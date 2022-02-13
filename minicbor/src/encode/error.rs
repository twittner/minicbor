use core::fmt;

/// The various kinds of errors that may occur during encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Error writing bytes to a `Write` impl.
    Write,
    /// Generic error message.
    Message,
    /// Custom error.
    #[cfg(feature = "std")]
    Custom
}

/// Encoding errors.
#[derive(Debug)]
pub struct Error<E> {
    err: ErrorImpl<E>,
    msg: &'static str
}

impl<E> Error<E> {
    /// Construct an error with a generic message.
    pub fn message(msg: &'static str) -> Self {
        Error { err: ErrorImpl::Message, msg }
    }

    /// A write error happened.
    pub fn write(e: E) -> Self {
        Error { err: ErrorImpl::Write(e), msg: "" }
    }

    /// A custom error.
    ///
    /// *Requires feature* `"std"`.
    #[cfg(feature = "std")]
    pub fn custom(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error { err: ErrorImpl::Custom(err), msg: "" }
    }

    /// Add a message to this error value.
    pub fn with_message(mut self, msg: &'static str) -> Self {
        self.msg = msg;
        self
    }

    /// Get the kind of error that happened.
    pub fn kind(&self) -> ErrorKind {
        match &self.err {
            ErrorImpl::Write(_)  => ErrorKind::Write,
            ErrorImpl::Message   => ErrorKind::Message,
            #[cfg(feature = "std")]
            ErrorImpl::Custom(_) => ErrorKind::Custom
        }
    }
}

/// Internal error representation.
#[derive(Debug)]
enum ErrorImpl<E> {
    /// Error writing bytes to a `Write` impl.
    Write(E),
    /// Generic error message.
    Message,
    /// Custom error.
    #[cfg(feature = "std")]
    Custom(Box<dyn std::error::Error + Send + Sync>)
}

impl<W: fmt::Display> fmt::Display for Error<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.err {
            ErrorImpl::Message  => write!(f, "{}", self.msg),
            ErrorImpl::Write(e) =>
                if self.msg.is_empty() {
                    write!(f, "write error: {}", e)
                } else {
                    write!(f, "write error: {}, {}", e, self.msg)
                }
            #[cfg(feature = "std")]
            ErrorImpl::Custom(e) =>
                if self.msg.is_empty() {
                    write!(f, "{}", e)
                } else {
                    write!(f, "{}, {}", e, self.msg)
                }
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.err {
            ErrorImpl::Message  => None,
            ErrorImpl::Write(e) => Some(e),
            #[cfg(feature = "std")]
            ErrorImpl::Custom(e) => Some(&**e)
        }
    }
}

