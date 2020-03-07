//! The [`Read`] trait definition and implementations.

/// A type that provides sequential access to bytes and byte slices.
pub trait Read<'b> {
    type Error;

    /// Get the byte at the current position.
    fn peek(&self) -> Result<u8, Self::Error>;

    /// Consume and return the byte at the current position.
    fn read(&mut self) -> Result<u8, Self::Error>;

    /// Consume and return *n* bytes starting at the current position.
    fn read_slice(&mut self, n: usize) -> Result<&'b [u8], Self::Error>;
}

impl<'b> Read<'b> for &'b [u8] {
    type Error = crate::EndOfSlice;

    fn peek(&self) -> Result<u8, Self::Error> {
        if self.is_empty() {
            return Err(crate::EndOfSlice(()))
        }
        Ok(self[0])
    }

    fn read(&mut self) -> Result<u8, Self::Error> {
        if self.is_empty() {
            return Err(crate::EndOfSlice(()))
        }
        let (a, b) = self.split_at(1);
        *self = b;
        Ok(a[0])
    }

    fn read_slice(&mut self, n: usize) -> Result<&'b [u8], Self::Error> {
        if self.len() < n {
            return Err(crate::EndOfSlice(()))
        }
        let (a, b) = self.split_at(n);
        *self = b;
        Ok(a)
    }
}

