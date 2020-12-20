//! A [`Reader]` to decode length-delimited CBOR items.

use futures_io::AsyncRead;
use futures_util::AsyncReadExt;
use minicbor::{decode, Decode};
use std::{fmt, io};

#[derive(Debug)]
pub struct Len(u32);

impl Len {
    pub fn val(self) -> u32 {
        self.0
    }
}

/// Wraps [`std::io::Read`] and reads length-delimited CBOR values.
#[derive(Debug)]
pub struct Reader<R> {
    reader: R,
    buffer: Vec<u8>
}

/// Possible read errors.
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured.
    Io(io::Error),
    /// A decoding error occured.
    Decode(decode::Error)
}

impl<R> Reader<R> {
    /// Create a new reader.
    pub fn new(reader: R) -> Self {
        Self::with_buffer(reader, Vec::new())
    }

    /// Create a new reader.
    pub fn with_buffer(reader: R, buffer: Vec<u8>) -> Self {
        Self { reader, buffer }
    }

    /// Decompose this reader into the inner reader and the buffer.
    pub fn into_parts(self) -> (R, Vec<u8>) {
        (self.reader, self.buffer)
    }
}

impl<R: AsyncRead + Unpin> Reader<R> {
    /// Read the length preceding the CBOR value.
    pub async fn read_len(&mut self) -> Result<Option<Len>, Error> {
        let mut buf = [0; 4];
        match self.reader.read_exact(buf.as_mut()).await {
            Ok(()) => Ok(Some(Len(u32::from_be_bytes(buf)))),
            Err(e) =>
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    Ok(None)
                } else {
                    Err(e.into())
                }
        }
    }

    /// Read the next CBOR value and decode it.
    pub async fn read_val<'a, T>(&'a mut self, n: Len) -> Result<T, Error>
    where
        T: Decode<'a>
    {
        self.buffer.clear();
        self.buffer.resize(n.val() as usize, 0u8);
        self.reader.read_exact(&mut self.buffer).await?;
        minicbor::decode(&self.buffer).map_err(Error::Decode)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "i/o error: {}", e),
            Error::Decode(e) => write!(f, "decode error: {}", e)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Decode(e) => Some(e)
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

