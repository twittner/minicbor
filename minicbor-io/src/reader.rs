use crate::Error;
use minicbor::Decode;
use std::io;

/// Wraps a [`std::io::Read`] and reads length-delimited CBOR values.
#[derive(Debug)]
pub struct Reader<R> {
    reader: R,
    buffer: Vec<u8>,
    max_len: usize
}

impl<R> Reader<R> {
    /// Create a new reader with a max. buffer size of 512KiB.
    pub fn new(reader: R) -> Self {
        Self::with_buffer(reader, Vec::new())
    }

    /// Create a new reader with a max. buffer size of 512KiB.
    pub fn with_buffer(reader: R, buffer: Vec<u8>) -> Self {
        Self { reader, buffer, max_len: 512 * 1024 }
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

impl<R: io::Read> Reader<R> {
    /// Read the next CBOR value and decode it.
    ///
    /// The value is assumed to be preceded by a `u32` (4 bytes in network
    /// byte order) denoting the length of the CBOR item in bytes.
    ///
    /// Reading 0 bytes when decoding the length prefix results in `Ok(None)`,
    /// otherwise either `Some` value or an error is returned.
    pub fn read<'a, T: Decode<'a, ()>>(&'a mut self) -> Result<Option<T>, Error> {
        self.read_with(&mut ())
    }

    /// Like [`Reader::read`] but accepting a user provided decoding context.
    pub fn read_with<'a, C, T: Decode<'a, C>>(&'a mut self, ctx: &mut C) -> Result<Option<T>, Error> {
        let mut buf = [0; 4];
        let mut len = 0;
        while len < 4 {
            match self.reader.read(&mut buf[len ..]) {
                Ok(0) if len == 0 =>
                    return Ok(None),
                Ok(0) =>
                    return Err(Error::Io(io::ErrorKind::UnexpectedEof.into())),
                Ok(n) =>
                    len += n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted =>
                    continue,
                Err(e) =>
                    return Err(Error::Io(e))
            }
        }
        let len = u32::from_be_bytes(buf) as usize;
        if len > self.max_len {
            return Err(Error::InvalidLen)
        }
        self.buffer.clear();
        self.buffer.resize(len, 0u8);
        self.reader.read_exact(&mut self.buffer)?;
        minicbor::decode_with(&self.buffer, ctx).map_err(Error::Decode).map(Some)
    }
}

