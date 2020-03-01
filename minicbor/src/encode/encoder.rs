use crate::{SIGNED, BYTES, TEXT, ARRAY, MAP, TAGGED, SIMPLE};
use crate::data::Tag;
use crate::encode::{Error, Write};

#[derive(Debug, Clone)]
pub struct Encoder<W> { writer: W }

impl<W> AsRef<W> for Encoder<W> {
    fn as_ref(&self) -> &W {
        &self.writer
    }
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Encoder<W> {
        Encoder { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn u8(&mut self, x: u8) -> Result<&mut Self, Error<W::Error>> {
        if let 0 ..= 0x17 = x {
            self.put(&[x])
        } else {
            self.put(&[24, x])
        }
    }

    pub fn i8(&mut self, x: i8) -> Result<&mut Self, Error<W::Error>> {
        if x >= 0 {
            return self.u8(x as u8)
        }
        match (-1 - x) as u8 {
            n @ 0 ..= 0x17 => self.put(&[SIGNED | n]),
            n              => self.put(&[SIGNED | 24, n])
        }
    }

    pub fn u16(&mut self, x: u16) -> Result<&mut Self, Error<W::Error>> {
        match x {
            0    ..= 0x17 => self.put(&[x as u8]),
            0x18 ..= 0xff => self.put(&[24, x as u8]),
            _             => self.put(&[25])?.put(&x.to_be_bytes()[..])
        }
    }

    pub fn i16(&mut self, x: i16) -> Result<&mut Self, Error<W::Error>> {
        if x >= 0 {
            return self.u16(x as u16)
        }
        match (-1 - x) as u16 {
            n @ 0    ..= 0x17 => self.put(&[SIGNED | n as u8]),
            n @ 0x18 ..= 0xff => self.put(&[SIGNED | 24, n as u8]),
            n                 => self.put(&[SIGNED | 25])?.put(&n.to_be_bytes()[..])
        }
    }

    pub fn u32(&mut self, x: u32) -> Result<&mut Self, Error<W::Error>> {
        match x {
            0     ..= 0x17   => self.put(&[x as u8]),
            0x18  ..= 0xff   => self.put(&[24, x as u8]),
            0x100 ..= 0xffff => self.put(&[25])?.put(&(x as u16).to_be_bytes()[..]),
            _                => self.put(&[26])?.put(&x.to_be_bytes()[..])
        }
    }

    pub fn i32(&mut self, x: i32) -> Result<&mut Self, Error<W::Error>> {
        if x >= 0 {
            return self.u32(x as u32)
        }
        match (-1 - x) as u32 {
            n @ 0     ..= 0x17   => self.put(&[SIGNED | n as u8]),
            n @ 0x18  ..= 0xff   => self.put(&[SIGNED | 24, n as u8]),
            n @ 0x100 ..= 0xffff => self.put(&[SIGNED | 25])?.put(&(n as u16).to_be_bytes()[..]),
            n                    => self.put(&[SIGNED | 26])?.put(&n.to_be_bytes()[..])
        }
    }

    pub fn u64(&mut self, x: u64) -> Result<&mut Self, Error<W::Error>> {
        match x {
            0        ..= 0x17        => self.put(&[x as u8]),
            0x18     ..= 0xff        => self.put(&[24, x as u8]),
            0x100    ..= 0xffff      => self.put(&[25])?.put(&(x as u16).to_be_bytes()[..]),
            0x1_0000 ..= 0xffff_ffff => self.put(&[26])?.put(&(x as u32).to_be_bytes()[..]),
            _                        => self.put(&[27])?.put(&x.to_be_bytes()[..])
        }
    }

    pub fn i64(&mut self, x: i64) -> Result<&mut Self, Error<W::Error>> {
        if x >= 0 {
            return self.u64(x as u64)
        }
        match (-1 - x) as u64 {
            n @ 0        ..= 0x17        => self.put(&[SIGNED | n as u8]),
            n @ 0x18     ..= 0xff        => self.put(&[SIGNED | 24, n as u8]),
            n @ 0x100    ..= 0xffff      => self.put(&[SIGNED | 25])?.put(&(n as u16).to_be_bytes()[..]),
            n @ 0x1_0000 ..= 0xffff_ffff => self.put(&[SIGNED | 26])?.put(&(n as u32).to_be_bytes()[..]),
            n                            => self.put(&[SIGNED | 27])?.put(&n.to_be_bytes()[..])
        }
    }

    pub fn null(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[SIMPLE | 22])
    }

    pub fn undefined(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[SIMPLE | 23])
    }

    pub fn simple(&mut self, x: u8) -> Result<&mut Self, Error<W::Error>> {
        if x < 0x14 {
            self.put(&[SIMPLE | x])
        } else {
            self.put(&[SIMPLE | 24, x])
        }
    }

    pub fn f32(&mut self, x: f32) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[SIMPLE | 26])?.put(&x.to_be_bytes()[..])
    }

    pub fn f64(&mut self, x: f64) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[SIMPLE | 27])?.put(&x.to_be_bytes()[..])
    }

    pub fn bool(&mut self, x: bool) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[SIMPLE | if x { 0x15 } else { 0x14 }])
    }

    pub fn char(&mut self, x: char) -> Result<&mut Self, Error<W::Error>> {
        let mut buf = [0; 4];
        self.str(x.encode_utf8(&mut buf))
    }

    pub fn tag(&mut self, x: Tag) -> Result<&mut Self, Error<W::Error>> {
        self.type_len(TAGGED, x.into())
    }

    pub fn bytes(&mut self, x: &[u8]) -> Result<&mut Self, Error<W::Error>> {
        self.type_len(BYTES, x.len() as u64)?.put(x)
    }

    pub fn str(&mut self, x: &str) -> Result<&mut Self, Error<W::Error>> {
        self.type_len(TEXT, x.len() as u64)?.put(x.as_bytes())
    }

    pub fn array(&mut self, len: usize) -> Result<&mut Self, Error<W::Error>> {
        self.type_len(ARRAY, len as u64)
    }

    pub fn map(&mut self, len: usize) -> Result<&mut Self, Error<W::Error>> {
        self.type_len(MAP, len as u64)
    }

    pub fn begin_array(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[0x9f])
    }

    pub fn begin_bytes(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[0x5f])
    }

    pub fn begin_map(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[0xbf])
    }

    pub fn begin_str(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[0x7f])
    }

    pub fn end(&mut self) -> Result<&mut Self, Error<W::Error>> {
        self.put(&[0xff])
    }

    pub fn ok(&mut self) -> Result<(), Error<W::Error>> {
        Ok(())
    }

    fn put(&mut self, b: &[u8]) -> Result<&mut Self, Error<W::Error>> {
        self.writer.write_all(b).map_err(Error::Write)?;
        Ok(self)
    }

    fn type_len(&mut self, t: u8, x: u64) -> Result<&mut Self, Error<W::Error>> {
        match x {
            0        ..= 0x17        => self.put(&[t | x as u8]),
            0x18     ..= 0xff        => self.put(&[t | 24, x as u8]),
            0x100    ..= 0xffff      => self.put(&[t | 25])?.put(&(x as u16).to_be_bytes()[..]),
            0x1_0000 ..= 0xffff_ffff => self.put(&[t | 26])?.put(&(x as u32).to_be_bytes()),
            _                        => self.put(&[t | 27])?.put(&x.to_be_bytes()[..])
        }
    }
}

