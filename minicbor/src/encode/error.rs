use core::fmt;

/// Encoding errors.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Error<W> {
    /// Write error.
    Write(W)
}

impl<W: fmt::Display> fmt::Display for Error<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Write(e) => write!(f, "write error: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl<W: std::error::Error + 'static> std::error::Error for Error<W> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Write(e) => Some(e)
        }
    }
}

