//! A [`Writer`] to encode length-delimited CBOR items.

use minicbor::{encode, Encode};
use std::{fmt, io};

/// Wraps [`std::io::Write`] and writes length-delimited CBOR values.
#[derive(Debug)]
pub struct Writer<W> {
    writer: W,
    buffer: Vec<u8>,
    max_len: usize
}

/// Possible write errors.
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured.
    Io(io::Error),
    /// An encoding error occured.
    Encode(encode::Error<io::Error>),
    /// The CBOR item is larger than the allowed max. buffer size.
    InvalidLen
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
    pub fn set_max_len(&mut self, val: usize) {
        self.max_len = val
    }

    /// Decompose this writer into the inner writer and the buffer.
    pub fn into_parts(self) -> (W, Vec<u8>) {
        (self.writer, self.buffer)
    }
}

impl<W: io::Write> Writer<W> {
    /// Encode and write a CBOR item and return the number of bytes written.
    pub fn write<T>(&mut self, val: T) -> Result<usize, Error>
    where
        T: Encode
    {
        self.buffer.clear();
        minicbor::encode(val, &mut self.buffer)?;
        if self.buffer.len() > self.max_len {
            return Err(Error::InvalidLen)
        }
        minicbor::encode(self.buffer.len(), &mut self.writer)?;
        self.writer.write_all(&self.buffer)?;
        Ok(self.buffer.len())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "i/o error: {}", e),
            Error::Encode(e) => write!(f, "encode error: {}", e),
            Error::InvalidLen => f.write_str("invalid length")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Encode(e) => Some(e),
            Error::InvalidLen => None
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<encode::Error<io::Error>> for Error {
    fn from(e: encode::Error<io::Error>) -> Self {
        Error::Encode(e)
    }
}

