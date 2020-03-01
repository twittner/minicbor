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


#[derive(Debug, Clone)]
pub struct SliceReader<'b> {
    buf: &'b [u8],
    pos: usize
}

impl<'b> SliceReader<'b> {
    pub fn new(buf: &'b [u8]) -> Self {
        SliceReader { buf, pos: 0 }
    }
}

impl<'b> Read<'b> for SliceReader<'b> {
    type Error = crate::EndOfSlice;

    fn current(&self) -> Result<u8, Self::Error> {
        if self.pos >= self.buf.len() {
            return Err(crate::EndOfSlice(()))
        }
        Ok(self.buf[self.pos])
    }

    fn next(&mut self) -> Result<u8, Self::Error> {
        if self.pos >= self.buf.len() {
            return Err(crate::EndOfSlice(()))
        }
        let b = self.buf[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn take(&mut self, n: usize) -> Result<&'b [u8], Self::Error> {
        if self.buf.len() - self.pos < n {
            return Err(crate::EndOfSlice(()))
        }
        let s = &self.buf[self.pos .. self.pos + n];
        self.pos += n;
        Ok(s)
    }
}

