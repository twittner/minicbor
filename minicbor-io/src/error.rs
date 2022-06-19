use core::convert::Infallible;
use std::{fmt, io};

/// Possible read/write errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error occured.
    Io(io::Error),
    /// A decoding error occured.
    Decode(minicbor::decode::Error),
    /// An encoding error occured.
    Encode(minicbor::encode::Error<Infallible>),
    /// The length preceding the CBOR value is not valid.
    InvalidLen
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "i/o error: {}", e),
            Error::Decode(e) => write!(f, "decode error: {}", e),
            Error::Encode(e) => write!(f, "encode error: {}", e),
            Error::InvalidLen => f.write_str("invalid length")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Decode(e) => Some(e),
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

impl From<minicbor::encode::Error<Infallible>> for Error {
    fn from(e: minicbor::encode::Error<Infallible>) -> Self {
        Error::Encode(e)
    }
}
