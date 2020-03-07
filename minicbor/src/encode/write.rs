//! The [`Write`] trait definition and implementations.
//!
//! If the feature `std` is present all `std::io::Write` impls
//! are made impls of [`Write`] too.

/// A type that accepts byte slices for writing.
pub trait Write {
    type Error;

    /// Write the whole byte slice.
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}

#[cfg(feature = "std")]
impl<W: std::io::Write> Write for W {
    type Error = std::io::Error;

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        std::io::Write::write_all(self, buf)
    }
}

#[cfg(not(feature = "std"))]
impl Write for &mut [u8] {
    type Error = crate::EndOfSlice;

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        if self.len() < buf.len() {
            return Err(crate::EndOfSlice(()))
        }
        let this = core::mem::replace(self, &mut []);
        let (prefix, suffix) = this.split_at_mut(buf.len());
        prefix.copy_from_slice(buf);
        *self = suffix;
        Ok(())
    }
}

