use core::fmt;

/// Encoding errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error<W> {
    /// Error writing bytes to a `Write` impl.
    Write(W),
    /// Generic error message.
    Message(&'static str),
    /// Custom error.
    #[cfg(feature = "std")]
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

impl<W: fmt::Display> fmt::Display for Error<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Write(e) => write!(f, "write error: {}", e),
            Error::Message(m) => write!(f, "{}", m),
            #[cfg(feature = "std")]
            Error::Custom(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
impl<W: std::error::Error + 'static> std::error::Error for Error<W> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Custom(e) => Some(&**e),
            Error::Write(e) => Some(e),
            Error::Message(_) => None,
        }
    }
}
