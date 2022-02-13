use core::{fmt, str};
use crate::data::Type;

/// The various kinds of errors that may occur during decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Decoding has (unexpectedly) reached the end of the input slice.
    EndOfInput,
    /// Data item to decode is not a valid `char`.
    InvalidChar,
    /// Decoding a string failed because it is invalid UTF-8.
    Utf8,
    /// A numeric value exceeds its value range.
    Overflow,
    /// An unexpected type was encountered.
    TypeMismatch,
    /// An unknown enum variant was encountered.
    UnknownVariant,
    /// A value was missing at the specified index.
    MissingValue,
    /// Generic error message.
    Message,
    /// Custom error.
    #[cfg(feature = "std")]
    Custom
}

/// Decoding errors.
#[derive(Debug)]
pub struct Error {
    err: ErrorImpl,
    msg: &'static str,
    pos: Option<usize>
}

impl Error {
    /// The premature end of input bytes has been detected.
    pub fn end_of_input() -> Self {
        Error { err: ErrorImpl::EndOfInput, msg: "", pos: None }
    }

    /// A type error.
    pub fn type_mismatch(ty: Type) -> Self {
        Error { err: ErrorImpl::TypeMismatch(ty), msg: "", pos: None }
    }

    /// An unknown enum variant (denoted by the given index) was encountered.
    pub fn unknown_variant(idx: u32) -> Self {
        Error { err: ErrorImpl::UnknownVariant(idx), msg: "", pos: None }
    }

    /// A value, expected at the given index, was missing.
    pub fn missing_value(idx: u32) -> Self {
        Error { err: ErrorImpl::MissingValue(idx), msg: "", pos: None }
    }

    /// Construct an error with a generic message.
    pub fn message(msg: &'static str) -> Self {
        Error { err: ErrorImpl::Message, msg, pos: None }
    }

    /// A custom error.
    ///
    /// *Requires feature* `"std"`.
    #[cfg(feature = "std")]
    pub fn custom(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error { err: ErrorImpl::Custom(err), msg: "", pos: None }
    }

    pub(crate) fn invalid_char(item: u32) -> Self {
        Error { err: ErrorImpl::InvalidChar(item), msg: "", pos: None }
    }

    pub(crate) fn utf8(err: str::Utf8Error) -> Self {
        Error { err: ErrorImpl::Utf8(err), msg: "", pos: None }
    }

    pub(crate) fn overflow(item: u64) -> Self {
        Error { err: ErrorImpl::Overflow(item), msg: "", pos: None }
    }

    /// Set the decoding position where the error happened.
    pub fn at(mut self, pos: usize) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Add a message to this error value.
    pub fn with_message(mut self, msg: &'static str) -> Self {
        self.msg = msg;
        self
    }

    /// Get the kind of error that happened.
    pub fn kind(&self) -> ErrorKind {
        match &self.err {
            ErrorImpl::EndOfInput        => ErrorKind::EndOfInput,
            ErrorImpl::InvalidChar(_)    => ErrorKind::InvalidChar,
            ErrorImpl::Utf8(_)           => ErrorKind::Utf8,
            ErrorImpl::Overflow(_)       => ErrorKind::Overflow,
            ErrorImpl::TypeMismatch(_)   => ErrorKind::TypeMismatch,
            ErrorImpl::UnknownVariant(_) => ErrorKind::UnknownVariant,
            ErrorImpl::MissingValue(_)   => ErrorKind::MissingValue,
            ErrorImpl::Message           => ErrorKind::Message,
            #[cfg(feature = "std")]
            ErrorImpl::Custom(_)         => ErrorKind::Custom
        }
    }
}

/// Internal error representation.
#[derive(Debug)]
enum ErrorImpl {
    /// Decoding has (unexpectedly) reached the end of the input slice.
    EndOfInput,
    /// Data item to decode is not a valid `char`.
    InvalidChar(u32),
    /// Decoding a string failed because it is invalid UTF-8.
    Utf8(str::Utf8Error),
    /// A numeric value exceeds its value range.
    Overflow(u64),
    /// An unexpected type was encountered.
    TypeMismatch(Type),
    /// An unknown enum variant was encountered.
    UnknownVariant(u32),
    /// A value was missing at the specified index.
    MissingValue(u32),
    /// Generic error message.
    Message,
    /// Custom error.
    #[cfg(feature = "std")]
    Custom(Box<dyn std::error::Error + Send + Sync>)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.err {
            ErrorImpl::EndOfInput =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "end of input bytes"),
                    ("", Some(p)) => write!(f, "end of input bytes at position {}", p),
                    (_, None)     => write!(f, "end of input bytes, {}", self.msg),
                    (_, Some(p))  => write!(f, "end of input bytes at position {}, {}", p, self.msg)
                }
            ErrorImpl::InvalidChar(n) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "invalid char: {:#x?}", n),
                    ("", Some(p)) => write!(f, "invalid char: {:#x?} at position {}", n, p),
                    (_, None)     => write!(f, "invalid char: {:#x?}, {}", n, self.msg),
                    (_, Some(p))  => write!(f, "invalid char: {:#x?} at position {}, {}", n, p, self.msg)
                }
            ErrorImpl::Utf8(e) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "invalid utf-8: {}", e),
                    ("", Some(p)) => write!(f, "invalid utf-8: {} at position {}", e, p),
                    (_, None)     => write!(f, "invalid utf-8: {}, {}", e, self.msg),
                    (_, Some(p))  => write!(f, "invalid utf-8: {} at position {}, {}", e, p, self.msg)
                }
            ErrorImpl::Overflow(n) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "overflow: {} overflows target type", n),
                    ("", Some(p)) => write!(f, "overflow: {} overflows target type at position {}", n, p),
                    (_, None)     => write!(f, "overflow: {} overflows target type {}", n, self.msg),
                    (_, Some(p))  => write!(f, "overflow: {} overflows target type at position {}, {}", n, p, self.msg)
                }
            ErrorImpl::TypeMismatch(t) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "unexpected type {}", t),
                    ("", Some(p)) => write!(f, "unexpected type {} at position {}", t, p),
                    (_, None)     => write!(f, "unexpected type {}, {}", t, self.msg),
                    (_, Some(p))  => write!(f, "unexpected type {} at position {}, {}", t, p, self.msg)
                }
            ErrorImpl::UnknownVariant(n) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "unknown enum variant {}", n),
                    ("", Some(p)) => write!(f, "unknown enum variant {} at position {}", n, p),
                    (_, None)     => write!(f, "unknown enum variant {}, {}", n, self.msg),
                    (_, Some(p))  => write!(f, "unknown enum variant {} at position {}, {}", n, p, self.msg)
                }
            ErrorImpl::MissingValue(n) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "missing value at index {}", n),
                    ("", Some(p)) => write!(f, "missing value at index {}, at position {}", n, p),
                    (_, None)     => write!(f, "missing value at index {}, {}", n, self.msg),
                    (_, Some(p))  => write!(f, "missing value at index {}, at position {}, {}", n, p, self.msg)
                }
            ErrorImpl::Message =>
                if let Some(p) = self.pos {
                    write!(f, "{}: {}", p, self.msg)
                } else {
                    write!(f, "{}", self.msg)
                }
            #[cfg(feature = "std")]
            ErrorImpl::Custom(e) =>
                match (self.msg, self.pos) {
                    ("", None)    => write!(f, "{}", e),
                    ("", Some(p)) => write!(f, "{}: {}", p, e),
                    (_, None)     => write!(f, "{}, {}", e, self.msg),
                    (_, Some(p))  => write!(f, "{}: {}, {}", p, e, self.msg)
                }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.err {
            ErrorImpl::Custom(e) => Some(&**e),
            ErrorImpl::Utf8(e) => Some(e),
            ErrorImpl::EndOfInput
            | ErrorImpl::InvalidChar(_)
            | ErrorImpl::Overflow(_)
            | ErrorImpl::TypeMismatch(_)
            | ErrorImpl::UnknownVariant(_)
            | ErrorImpl::MissingValue(_)
            | ErrorImpl::Message
            => None
        }
    }
}
