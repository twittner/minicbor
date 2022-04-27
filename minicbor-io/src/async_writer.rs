use crate::Error;
use futures_io::AsyncWrite;
use futures_util::AsyncWriteExt;
use minicbor::Encode;
use std::io;

/// Wraps an [`AsyncWrite`] and writes length-delimited CBOR values.
///
/// *Requires cargo feature* `"async-io"`.
#[derive(Debug)]
pub struct AsyncWriter<W> {
    writer: W,
    buffer: Vec<u8>,
    max_len: usize,
    state: State
}

/// Write state.
#[derive(Debug)]
enum State {
    /// Nothing is written at the moment.
    None,
    /// Writing buffer from offset.
    WriteFrom(usize)
}

impl<W> AsyncWriter<W> {
    /// Create a new writer with a max. buffer size of 512KiB.
    pub fn new(writer: W) -> Self {
        Self::with_buffer(writer, Vec::new())
    }

    /// Create a new writer with a max. buffer size of 512KiB.
    pub fn with_buffer(writer: W, buffer: Vec<u8>) -> Self {
        Self { writer, buffer, max_len: 512 * 1024, state: State::None }
    }

    /// Set the max. buffer size in bytes.
    ///
    /// If length values greater than this are encoded, an
    /// [`Error::InvalidLen`] will be returned.
    pub fn set_max_len(&mut self, val: u32) {
        self.max_len = val as usize
    }

    /// Get a reference to the inner writer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Get a mutable reference to the inner writer.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Deconstruct this writer into the inner writer and the buffer.
    pub fn into_parts(self) -> (W, Vec<u8>) {
        (self.writer, self.buffer)
    }
}

impl<W: AsyncWrite + Unpin> AsyncWriter<W> {
    /// Encode and write a CBOR value and return its size in bytes.
    ///
    /// The value will be preceded by a `u32` (4 bytes in network byte order),
    /// denoting the length of bytes constituting the serialised value.
    ///
    /// # Cancellation
    ///
    /// If the future returned by `AsyncWriter::write` is dropped while still
    /// pending, subsequent calls to `AsyncWriter::write` will discard any
    /// buffered data and instead encode, buffer and commit the new value.
    /// Cancelling a future thus cancels the transfer. However, it is also
    /// possible to resume the transfer by calling [`AsyncWriter::sync`]
    /// after cancellation, which is normally called implicitly by this method.
    pub async fn write<T: Encode<()>>(&mut self, val: T) -> Result<usize, Error> {
        self.write_with(val, &mut ()).await
    }

    /// Like [`AsyncWriter::write`] but accepting a user provided encoding context.
    pub async fn write_with<C, T: Encode<C>>(&mut self, val: T, ctx: &mut C) -> Result<usize, Error> {
        self.buffer.resize(4, 0u8);
        minicbor::encode_with(val, &mut self.buffer, ctx)?;
        if self.buffer.len() - 4 > self.max_len {
            return Err(Error::InvalidLen)
        }
        let prefix = (self.buffer.len() as u32 - 4).to_be_bytes();
        self.buffer[.. 4].copy_from_slice(&prefix);
        self.state = State::WriteFrom(0);

        self.sync().await?;

        Ok(self.buffer.len() - 4)
    }

    /// Commit any buffered data to the inner `AsyncWrite`.
    ///
    /// This method is implicitly called by [`AsyncWriter::write`]. The only
    /// reason to call it explicitly is to resume the write operation started
    /// by a previously unfinished, i.e. cancelled, `AsyncWriter::write` call.
    pub async fn sync(&mut self) -> Result<(), Error> {
        loop {
            match self.state {
                State::None => {
                    return Ok(())
                }
                State::WriteFrom(o) if o >= self.buffer.len() => {
                    self.state = State::None;
                    return Ok(())
                }
                State::WriteFrom(ref mut o) => {
                    let n = self.writer.write(&self.buffer[*o ..]).await?;
                    if n == 0 {
                        return Err(Error::Io(io::ErrorKind::WriteZero.into()))
                    }
                    *o += n
                }
            }
        }
    }

    /// Flush the inner `AsyncWrite`.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush().await?;
        Ok(())
    }
}
