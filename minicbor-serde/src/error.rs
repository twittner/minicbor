use core::fmt;

use minicbor::{decode, encode};

/// Deserialisation error.
#[derive(Debug)]
pub struct DecodeError(decode::Error);

/// Serialisation error.
#[derive(Debug)]
pub struct EncodeError<E>(encode::Error<E>);

impl<E> From<encode::Error<E>> for EncodeError<E> {
    fn from(e: encode::Error<E>) -> Self {
        Self(e)
    }
}

impl From<decode::Error> for DecodeError {
    fn from(e: decode::Error) -> Self {
        Self(e)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<E: fmt::Display> fmt::Display for EncodeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl core::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.0.source()
    }
}

impl<E: core::error::Error + 'static> core::error::Error for EncodeError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.0.source()
    }
}

impl<E: core::error::Error + 'static> serde::ser::Error for EncodeError<E> {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        #[cfg(feature = "alloc")]
        return Self(encode::Error::message(_msg));
        #[cfg(not(feature = "alloc"))]
        Self(encode::Error::message("custom error"))
    }
}

impl serde::de::Error for DecodeError {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        #[cfg(feature = "alloc")]
        return Self(decode::Error::message(_msg));
        #[cfg(not(feature = "alloc"))]
        Self(decode::Error::message("custom error"))
    }
}
