use crate::Error;
use futures_io::AsyncRead;
use futures_util::AsyncReadExt;
use minicbor::Decode;
use std::io;

/// Wraps an [`AsyncRead`] and reads length-delimited CBOR values.
///
/// *Requires cargo feature* `"async-io"`.
#[derive(Debug)]
pub struct AsyncReader<R> {
    reader: R,
    buffer: Vec<u8>,
    max_len: usize,
    state: State
}

/// Read state.
#[derive(Debug)]
enum State {
    /// Reading length prefix.
    ReadLen([u8; 4], u8),
    /// Reading CBOR item bytes.
    ReadVal(usize)
}

impl State {
    /// Setup a new state.
    fn new() -> Self {
        State::ReadLen([0; 4], 0)
    }
}

impl<R> AsyncReader<R> {
    /// Create a new reader with a max. buffer size of 512KiB.
    pub fn new(reader: R) -> Self {
        Self::with_buffer(reader, Vec::new())
    }

    /// Create a new reader with a max. buffer size of 512KiB.
    pub fn with_buffer(reader: R, buffer: Vec<u8>) -> Self {
        Self { reader, buffer, max_len: 512 * 1024, state: State::new() }
    }

    /// Set the max. buffer size in bytes.
    ///
    /// If length values greater than this are decoded, an
    /// [`Error::InvalidLen`] will be returned.
    pub fn set_max_len(&mut self, val: u32) {
        self.max_len = val as usize
    }

    /// Get a reference to the inner reader.
    pub fn reader(&self) -> &R {
        &self.reader
    }

    /// Get a mutable reference to the inner reader.
    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Deconstruct this reader into the inner reader and the buffer.
    pub fn into_parts(self) -> (R, Vec<u8>) {
        (self.reader, self.buffer)
    }
}

impl<R: AsyncRead + Unpin> AsyncReader<R> {
    /// Read the next CBOR value and decode it.
    ///
    /// The value is assumed to be preceded by a `u32` (4 bytes in network
    /// byte order) denoting the length of the CBOR item in bytes.
    ///
    /// Reading 0 bytes when decoding the length prefix results in `Ok(None)`,
    /// otherwise either `Some` value or an error is returned.
    ///
    /// # Cancellation
    ///
    /// The future returned by `AsyncReader::read` can be dropped while still
    /// pending. Subsequent calls to `AsyncReader::read` will resume reading
    /// where the previous future left off.
    pub async fn read<'a, T: Decode<'a>>(&'a mut self) -> Result<Option<T>, Error> {
        loop {
            match self.state {
                State::ReadLen(buf, 4) => {
                    let len = u32::from_be_bytes(buf) as usize;
                    if len > self.max_len {
                        return Err(Error::InvalidLen)
                    }
                    self.buffer.clear();
                    self.buffer.resize(len, 0u8);
                    self.state = State::ReadVal(0)
                }
                State::ReadLen(ref mut buf, ref mut o) => {
                    let n = self.reader.read(&mut buf[usize::from(*o) ..]).await?;
                    if n == 0 {
                        return Ok(None)
                    }
                    *o += n as u8
                }
                State::ReadVal(o) if o >= self.buffer.len() => {
                    self.state = State::new();
                    return minicbor::decode(&self.buffer).map_err(Error::Decode).map(Some)
                }
                State::ReadVal(ref mut o) => {
                    let n = self.reader.read(&mut self.buffer[*o ..]).await?;
                    if n == 0 {
                        return Err(Error::Io(io::ErrorKind::UnexpectedEof.into()))
                    }
                    *o += n
                }
            }
        }
    }
}

