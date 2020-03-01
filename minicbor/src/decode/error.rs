use core::{fmt, str};

/// Decoding errors.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Error<R> {
    /// Read error.
    Read(R),
    /// Data item to decode is not a valid `char`.
    InvalidChar,
    /// Decoding a string failed because of invalid UTF-8 encoding.
    Utf8(str::Utf8Error),
    /// A numeric value exceeds its value range.
    Overflow(u64, &'static str),
    /// An unexpected type was encountered.
    TypeMismatch(u8, &'static str),
    /// An unknown enum variant.
    /// This error can only occur when deriving [`Decode`].
    UnknownVariant(u32),
    /// A value was missing at the specified index.
    /// This error can only occur when deriving [`Decode`].
    MissingValue(u32, &'static str)
}

impl<R: fmt::Display> fmt::Display for Error<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Read(e)            => write!(f, "read error: {}", e),
            Error::InvalidChar        => f.write_str("invalid char"),
            Error::Utf8(e)            => write!(f, "invalid utf-8: {}", e),
            Error::Overflow(n, m)     => write!(f, "{}: {} overflows target type", m, n),
            Error::TypeMismatch(t, m) => write!(f, "type mismatch: {}, {}", t, m),
            Error::UnknownVariant(n)  => write!(f, "unknown enum variant {}", n),
            Error::MissingValue(n, s) => write!(f, "missing value at index {} for {}", n, s)
        }
    }
}

#[cfg(feature = "std")]
impl<R: std::error::Error + 'static> std::error::Error for Error<R> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Read(e) => Some(e),
            Error::Utf8(e) => Some(e),
            | Error::InvalidChar
            | Error::Overflow(..)
            | Error::TypeMismatch(..)
            | Error::UnknownVariant(_)
            | Error::MissingValue(..)
            => None
        }
    }
}

impl<R> From<str::Utf8Error> for Error<R> {
    fn from(e: str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}


