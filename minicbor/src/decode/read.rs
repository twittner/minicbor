pub trait Read<'b> {
    type Error;

    fn current(&self) -> Result<u8, Self::Error>;
    fn next(&mut self) -> Result<u8, Self::Error>;
    fn take(&mut self, n: usize) -> Result<&'b [u8], Self::Error>;
}

impl<'b> Read<'b> for &'b [u8] {
    type Error = crate::EndOfSlice;

    fn current(&self) -> Result<u8, Self::Error> {
        if self.is_empty() {
            return Err(crate::EndOfSlice(()))
        }
        Ok(self[0])
    }

    fn next(&mut self) -> Result<u8, Self::Error> {
        if self.is_empty() {
            return Err(crate::EndOfSlice(()))
        }
        let (a, b) = self.split_at(1);
        *self = b;
        Ok(a[0])
    }

    fn take(&mut self, n: usize) -> Result<&'b [u8], Self::Error> {
        if self.len() < n {
            return Err(crate::EndOfSlice(()))
        }
        let (a, b) = self.split_at(n);
        *self = b;
        Ok(a)
    }
}

