//! A [`Reader]` to decode length-delimited CBOR items.

use minicbor::{decode, Decode};
use std::{fmt, io};

/// Wraps [`std::io::Read`] and reads length-delimited CBOR values.
#[derive(Debug)]
pub struct Reader<R> {
    reader: R,
    buffer: Vec<u8>,
    max_len: usize
}

/// Possible read errors.
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured.
    Io(io::Error),
    /// A decoding error occured.
    Decode(decode::Error),
    /// The length preceding the CBOR value is not valid.
    InvalidLen
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
    pub fn set_max_len(&mut self, val: usize) {
        self.max_len = val
    }

    /// Decompose this reader into the inner reader and the buffer.
    pub fn into_parts(self) -> (R, Vec<u8>) {
        (self.reader, self.buffer)
    }
}

impl<R: io::Read> Reader<R> {
    /// Read the next CBOR item and decode it.
    pub fn read<'a, T>(&'a mut self) -> Result<T, Error>
    where
        T: Decode<'a>
    {
        let n = self.read_len()?;
        self.buffer.clear();
        self.buffer.resize(n, 0u8);
        self.reader.read_exact(&mut self.buffer)?;
        minicbor::decode(&self.buffer).map_err(Error::Decode)
    }

    /// Read the length preceding the CBOR value.
    fn read_len(&mut self) -> Result<usize, Error> {
        let mut buf = [0; 8];
        for i in 0 .. buf.len() {
            self.reader.read_exact(&mut buf[i .. i + 1])?;
            match minicbor::decode(&buf[.. i + 1]) {
                Ok(n) => {
                    if n > self.max_len {
                        return Err(Error::InvalidLen)
                    }
                    return Ok(n)
                }
                Err(e) if e.is_end_of_input() => continue,
                Err(e) => return Err(Error::Decode(e))
            }
        }
        Err(Error::InvalidLen)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "i/o error: {}", e),
            Error::Decode(e) => write!(f, "decode error: {}", e),
            Error::InvalidLen => f.write_str("invalid length")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Decode(e) => Some(e),
            Error::InvalidLen => None
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

