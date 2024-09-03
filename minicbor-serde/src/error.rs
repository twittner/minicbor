use core::fmt;

use minicbor::{decode, encode};

/// Deserialisation error.
#[derive(Debug)]
pub struct DecodeError(DecodeErrorImpl);

/// Serialisation error.
#[derive(Debug)]
pub struct EncodeError<E>(EncodeErrorImpl<E>);

#[derive(Debug)]
enum DecodeErrorImpl {
    Decode(decode::Error),
    Custom(String)
}

#[derive(Debug)]
enum EncodeErrorImpl<E> {
    Encode(encode::Error<E>),
    Custom(String)
}

impl<E> From<encode::Error<E>> for EncodeError<E> {
    fn from(e: encode::Error<E>) -> Self {
        Self(EncodeErrorImpl::Encode(e))
    }
}

impl From<decode::Error> for DecodeError {
    fn from(e: decode::Error) -> Self {
        Self(DecodeErrorImpl::Decode(e))
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            DecodeErrorImpl::Decode(e) => e.fmt(f),
            DecodeErrorImpl::Custom(s) => f.write_str(s)
        }
    }
}

impl<E: fmt::Display> fmt::Display for EncodeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            EncodeErrorImpl::Encode(e) => e.fmt(f),
            EncodeErrorImpl::Custom(s) => f.write_str(s)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            DecodeErrorImpl::Decode(e) => e.source(),
            DecodeErrorImpl::Custom(_) => None
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for EncodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            EncodeErrorImpl::Encode(e) => e.source(),
            EncodeErrorImpl::Custom(_) => None
        }
    }
}

#[cfg(not(feature = "std"))]
impl serde::de::StdError for DecodeError {}

#[cfg(not(feature = "std"))]
impl<E: fmt::Debug + fmt::Display> serde::ser::StdError for EncodeError<E> {}

impl<E: serde::ser::StdError + 'static> serde::ser::Error for EncodeError<E> {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        #[cfg(feature = "alloc")]
        use alloc::string::ToString;

        Self(EncodeErrorImpl::Custom(msg.to_string()))
    }
}

impl serde::de::Error for DecodeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        #[cfg(feature = "alloc")]
        use alloc::string::ToString;

        Self(DecodeErrorImpl::Custom(msg.to_string()))
    }
}
