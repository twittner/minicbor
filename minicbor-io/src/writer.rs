use crate::Error;
use minicbor::Encode;
use std::io;

/// Wraps a [`std::io::Write`] and writes length-delimited CBOR values.
#[derive(Debug)]
pub struct Writer<W> {
    writer: W,
    buffer: Vec<u8>,
    max_len: usize
}

impl<W> Writer<W> {
    /// Create a new writer with a max. buffer size of 512KiB.
    pub fn new(writer: W) -> Self {
        Self::with_buffer(writer, Vec::new())
    }

    /// Create a new writer with a max. buffer size of 512KiB.
    pub fn with_buffer(writer: W, buffer: Vec<u8>) -> Self {
        Self { writer, buffer, max_len: 512 * 1024 }
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

impl<W: io::Write> Writer<W> {
    /// Encode and write a CBOR value and return its size in bytes.
    pub fn write<T: Encode>(&mut self, val: T) -> Result<usize, Error> {
        self.buffer.resize(4, 0u8);
        minicbor::encode(val, &mut self.buffer)?;
        if self.buffer.len() - 4 > self.max_len {
            return Err(Error::InvalidLen)
        }
        let prefix = (self.buffer.len() as u32 - 4).to_be_bytes();
        self.buffer[.. 4].copy_from_slice(&prefix);
        self.writer.write_all(&self.buffer)?;
        Ok(self.buffer.len() - 4)
    }

    /// Flush the inner `Write`r.
    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()?;
        Ok(())
    }
}
