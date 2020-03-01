use crate::{ARRAY, BREAK, BYTES, MAP, SIMPLE, TAGGED, TEXT, SIGNED, UNSIGNED};
use crate::data::{Tag, Type};
use crate::decode::{Decode, Error, Read};
use core::{f32, i8, i16, i32, i64};
use core::{convert::TryInto, str, marker};

macro_rules! u_to_i {
    ($val: expr, $typ: ty, $max: expr, $msg: expr) => {{
        let val = $val; // evaluate only once
        if val > $max {
            Err(Error::Overflow(u64::from(val), $msg))
        } else {
            Ok(-1 - val as $typ)
        }
    }}
}

macro_rules! u_as_i {
    ($val: expr, $typ: ty, $max: expr, $msg: expr) => {{
        let val = $val; // evaluate only once
        if val > $max {
            Err(Error::Overflow(u64::from(val), $msg))
        } else {
            Ok(val as $typ)
        }
    }}
}

#[derive(Debug, Clone)]
pub struct Decoder<'b, R: Read<'b>> {
    reader: R,
    _mark: marker::PhantomData<&'b ()>
}

impl<'b> Decoder<'b, &'b [u8]> {
    pub fn from_slice(b: &'b [u8]) -> Self {
        Decoder::new(b)
    }
}

impl<'b, R: Read<'b>> Decoder<'b, R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader, _mark: marker::PhantomData }
    }

    pub fn bool(&mut self) -> Result<bool, Error<R::Error>> {
        match self.next()? {
            0xf4 => Ok(false),
            0xf5 => Ok(true),
            b    => Err(Error::TypeMismatch(b, "expected bool"))
        }
    }

    pub fn u8(&mut self) -> Result<u8, Error<R::Error>> {
        match self.next()? {
            n @ 0 ..= 0x17 => Ok(n),
            0x18           => self.next(),
            b              => Err(Error::TypeMismatch(b, "expected u8"))
        }
    }

    pub fn u16(&mut self) -> Result<u16, Error<R::Error>> {
        match self.next()? {
            n @ 0 ..= 0x17 => Ok(u16::from(n)),
            0x18           => self.next().map(u16::from),
            0x19           => self.take(2).map(read_u16),
            b              => Err(Error::TypeMismatch(b, "expected u16"))
        }
    }

    pub fn u32(&mut self) -> Result<u32, Error<R::Error>> {
        match self.next()? {
            n @ 0 ..= 0x17 => Ok(u32::from(n)),
            0x18           => self.next().map(u32::from),
            0x19           => self.take(2).map(read_u16).map(u32::from),
            0x1a           => self.take(4).map(read_u32),
            b              => Err(Error::TypeMismatch(b, "expected u32"))
        }
    }

    pub fn u64(&mut self) -> Result<u64, Error<R::Error>> {
        let n = self.next()?;
        self.unsigned(n)
    }

    pub fn i8(&mut self) -> Result<i8, Error<R::Error>> {
        match self.next()? {
            n @ 0x00 ..= 0x17 => Ok(n as i8),
            0x18              => u_as_i!(self.next()?, i8, i8::MAX as u8, "u8->i8"),
            n @ 0x20 ..= 0x37 => Ok(-1 - (n - 0x20) as i8),
            0x38              => u_to_i!(self.next()?, i8, i8::MAX as u8, "u8->i8"),
            b                 => Err(Error::TypeMismatch(b, "expected i8"))
        }
    }

    pub fn i16(&mut self) -> Result<i16, Error<R::Error>> {
        match self.next()? {
            n @ 0x00 ..= 0x17 => Ok(i16::from(n)),
            0x18              => self.next().map(i16::from),
            0x19              => u_as_i!(self.take(2).map(read_u16)?, i16, i16::MAX as u16, "u16->i16"),
            n @ 0x20 ..= 0x37 => Ok(-1 - i16::from(n - 0x20)),
            0x38              => self.next().map(|n| -1 - i16::from(n)),
            0x39              => u_to_i!(self.take(2).map(read_u16)?, i16, i16::MAX as u16, "u16->i16"),
            b                 => Err(Error::TypeMismatch(b, "expected i16"))
        }
    }

    pub fn i32(&mut self) -> Result<i32, Error<R::Error>> {
        match self.next()? {
            n @ 0x00 ..= 0x17 => Ok(i32::from(n)),
            0x18              => self.next().map(i32::from),
            0x19              => self.take(2).map(read_u16).map(i32::from),
            0x1a              => u_as_i!(self.take(4).map(read_u32)?, i32, i32::MAX as u32, "u32->i32"),
            n @ 0x20 ..= 0x37 => Ok(-1 - i32::from(n - 0x20)),
            0x38              => self.next().map(|n| -1 - i32::from(n)),
            0x39              => self.take(2).map(read_u16).map(|n| -1 - i32::from(n)),
            0x3a              => u_to_i!(self.take(4).map(read_u32)?, i32, i32::MAX as u32, "u32->i32"),
            b                 => Err(Error::TypeMismatch(b, "expected i32"))
        }
    }

    pub fn i64(&mut self) -> Result<i64, Error<R::Error>> {
        match self.next()? {
            n @ 0x00 ..= 0x17 => Ok(i64::from(n)),
            0x18              => self.next().map(i64::from),
            0x19              => self.take(2).map(read_u16).map(i64::from),
            0x1a              => self.take(4).map(read_u32).map(i64::from),
            0x1b              => u_as_i!(self.take(8).map(read_u64)?, i64, i64::MAX as u64, "u64->i64"),
            n @ 0x20 ..= 0x37 => Ok(-1 - i64::from(n - 0x20)),
            0x38              => self.next().map(|n| -1 - i64::from(n)),
            0x39              => self.take(2).map(read_u16).map(|n| -1 - i64::from(n)),
            0x3a              => self.take(4).map(read_u32).map(|n| -1 - i64::from(n)),
            0x3b              => u_to_i!(self.take(8).map(read_u64)?, i64, i64::MAX as u64, "u64->i64"),
            b                 => Err(Error::TypeMismatch(b, "expected i64"))
        }
    }

    #[cfg(feature = "half")]
    pub fn f16(&mut self) -> Result<f32, Error<R::Error>> {
        let b = self.next()?;
        if 0xf9 != b {
            return Err(Error::TypeMismatch(b, "expected f16"))
        }
        let mut n = [0; 2];
        n.copy_from_slice(self.take(2)?);
        Ok(half::f16::from_bits(u16::from_be_bytes(n)).to_f32())
    }

    pub fn f32(&mut self) -> Result<f32, Error<R::Error>> {
        match self.current()? {
            #[cfg(feature = "half")]
            0xf9 => self.f16(),
            0xfa => {
                self.next()?;
                let mut n = [0; 4];
                n.copy_from_slice(self.take(4)?);
                Ok(f32::from_be_bytes(n))
            }
            b => Err(Error::TypeMismatch(b, "expected f32"))
        }
    }

    pub fn f64(&mut self) -> Result<f64, Error<R::Error>> {
        match self.current()? {
            #[cfg(feature = "half")]
            0xf9 => self.f16().map(f64::from),
            0xfa => self.f32().map(f64::from),
            0xfb => {
                self.next()?;
                let mut n = [0; 8];
                n.copy_from_slice(self.take(8)?);
                Ok(f64::from_be_bytes(n))
            }
            b => Err(Error::TypeMismatch(b, "expected f64"))
        }
    }

    pub fn char(&mut self) -> Result<char, Error<R::Error>> {
        let b = self.next()?;
        if TEXT != type_of(b) {
            return Err(Error::TypeMismatch(b, "expected text"))
        }
        if info_of(b) > 4 {
            return Err(Error::InvalidChar)
        }
        str::from_utf8(self.take(usize::from(info_of(b)))?)?
            .chars()
            .next()
            .ok_or(Error::InvalidChar)
    }

    pub fn bytes(&mut self) -> Result<&'b [u8], Error<R::Error>> {
        let b = self.next()?;
        if BYTES != type_of(b) || info_of(b) == 31 {
            return Err(Error::TypeMismatch(b, "expected bytes (definite length)"))
        }
        let n = u64_to_usize(self.unsigned(info_of(b))?)?;
        self.take(n)
    }

    pub fn bytes_iter(&mut self) -> Result<BytesIter<'_, 'b, R>, Error<R::Error>> {
        let b = self.next()?;
        if BYTES != type_of(b) {
            return Err(Error::TypeMismatch(b, "expected bytes"))
        }
        match info_of(b) {
            31 => Ok(BytesIter { decoder: self, len: None }),
            n  => {
                let len = u64_to_usize(self.unsigned(n)?)?;
                Ok(BytesIter { decoder: self, len: Some(len) })
            }
        }
    }

    pub fn str(&mut self) -> Result<&'b str, Error<R::Error>> {
        let b = self.next()?;
        if TEXT != type_of(b) || info_of(b) == 31 {
            return Err(Error::TypeMismatch(b, "expected text (definite length)"))
        }
        let n = u64_to_usize(self.unsigned(info_of(b))?)?;
        let d = self.take(n)?;
        str::from_utf8(d).map_err(Error::from)
    }

    pub fn str_iter(&mut self) -> Result<StrIter<'_, 'b, R>, Error<R::Error>> {
        let b = self.next()?;
        if TEXT != type_of(b) {
            return Err(Error::TypeMismatch(b, "expected text"))
        }
        match info_of(b) {
            31 => Ok(StrIter { decoder: self, len: None }),
            n  => {
                let len = u64_to_usize(self.unsigned(n)?)?;
                Ok(StrIter { decoder: self, len: Some(len) })
            }
        }
    }

    pub fn array(&mut self) -> Result<Option<u64>, Error<R::Error>> {
        let b = self.next()?;
        if ARRAY != type_of(b) {
            return Err(Error::TypeMismatch(b, "expected array"))
        }
        match info_of(b) {
            31 => Ok(None),
            n  => Ok(Some(self.unsigned(n)?))
        }
    }

    pub fn array_iter<T>(&mut self) -> Result<ArrayIter<'_, 'b, R, T>, Error<R::Error>>
    where
        T: Decode<'b>
    {
        let len = self.array()?;
        Ok(ArrayIter { decoder: self, len, _mark: marker::PhantomData })
    }

    pub fn map(&mut self) -> Result<Option<u64>, Error<R::Error>> {
        let b = self.next()?;
        if MAP != type_of(b) {
            return Err(Error::TypeMismatch(b, "expected map"))
        }
        match info_of(b) {
            31 => Ok(None),
            n  => Ok(Some(self.unsigned(n)?))
        }
    }

    pub fn map_iter<K, V>(&mut self) -> Result<MapIter<'_, 'b, R, K, V>, Error<R::Error>>
    where
        K: Decode<'b>,
        V: Decode<'b>
    {
        let len = self.map()?;
        Ok(MapIter { decoder: self, len, _mark: marker::PhantomData })
    }

    pub fn tag(&mut self) -> Result<Tag, Error<R::Error>> {
        let b = self.next()?;
        if TAGGED != type_of(b) {
            return Err(Error::TypeMismatch(b, "expected tag"))
        }
        self.unsigned(info_of(b)).map(Tag::from)
    }

    pub fn simple(&mut self) -> Result<u8, Error<R::Error>> {
        match self.next()? {
            n @ SIMPLE ..= 0xf3 => Ok(n - SIMPLE),
            0xf8                => self.next(),
            n                   => Err(Error::TypeMismatch(n, "expected simple value"))
        }
    }

    pub fn datatype(&self) -> Result<Type, Error<R::Error>> {
        let b = self.current()?;
        Type::read(b).ok_or(Error::TypeMismatch(b, "expected known data type"))
    }

    pub fn skip(&mut self) -> Result<(), Error<R::Error>> {
        let mut nrounds = 1u64; // number of iterations over array and map elements
        let mut irounds = 0u64; // number of indefinite iterations

        while nrounds > 0 || irounds > 0 {
            match self.current()? {
                UNSIGNED ..= 0x1b => { self.u64()?; }
                SIGNED   ..= 0x3b => { self.i64()?; }
                BYTES    ..= 0x5f => { for _ in self.bytes_iter()? {} }
                TEXT     ..= 0x7f => { for _ in self.str_iter()? {} }
                ARRAY    ..= 0x9f =>
                    if let Some(n) = self.array()? {
                        nrounds = nrounds.saturating_add(n)
                    } else {
                        irounds = irounds.saturating_add(1)
                    }
                MAP ..= 0xbf =>
                    if let Some(n) = self.map()? {
                        nrounds = nrounds.saturating_add(2 * n)
                    } else {
                        irounds = irounds.saturating_add(1)
                    }
                TAGGED ..= 0xdb => { self.next().and_then(|n| self.unsigned(info_of(n)))?; }
                SIMPLE ..= 0xfb => { self.next().and_then(|n| self.unsigned(info_of(n)))?; }
                BREAK => {
                    self.next()?;
                    irounds = irounds.saturating_sub(1)
                }
                other => return Err(Error::TypeMismatch(other, "unknown type"))
            }
            nrounds = nrounds.saturating_sub(1)
        }

        Ok(())
    }

    fn unsigned(&mut self, b: u8) -> Result<u64, Error<R::Error>> {
        match b {
            n @ 0 ..= 0x17 => Ok(u64::from(n)),
            0x18           => self.next().map(u64::from),
            0x19           => self.take(2).map(read_u16).map(u64::from),
            0x1a           => self.take(4).map(read_u32).map(u64::from),
            0x1b           => self.take(8).map(read_u64),
            _              => Err(Error::TypeMismatch(b, "expected u64"))
        }
    }

    fn next(&mut self) -> Result<u8, Error<R::Error>> {
        self.reader.next().map_err(Error::Read)
    }

    fn current(&self) -> Result<u8, Error<R::Error>> {
        self.reader.current().map_err(Error::Read)
    }

    fn take(&mut self, n: usize) -> Result<&'b [u8], Error<R::Error>> {
        self.reader.take(n).map_err(Error::Read)
    }
}

#[derive(Debug)]
pub struct BytesIter<'a, 'b, R: Read<'b>> {
    decoder: &'a mut Decoder<'b, R>,
    len: Option<usize>
}

impl<'a, 'b, R: Read<'b>> Iterator for BytesIter<'a, 'b, R> {
    type Item = Result<&'b [u8], Error<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.next().map(|_| None).transpose(),
                Ok(_)     => Some(self.decoder.bytes()),
                Err(e)    => Some(Err(e))
            }
            Some(0) => None,
            Some(n) => {
                self.len = Some(0);
                Some(self.decoder.take(n))
            }
        }
    }
}

#[derive(Debug)]
pub struct StrIter<'a, 'b, R: Read<'b>> {
    decoder: &'a mut Decoder<'b, R>,
    len: Option<usize>
}

impl<'a, 'b, R: Read<'b>> Iterator for StrIter<'a, 'b, R> {
    type Item = Result<&'b str, Error<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.next().map(|_| None).transpose(),
                Ok(_)     => Some(self.decoder.str()),
                Err(e)    => Some(Err(e))
            }
            Some(0) => None,
            Some(n) => {
                self.len = Some(0);
                Some(self.decoder.take(n).and_then(|d| str::from_utf8(d).map_err(Error::from)))
            }
        }
    }
}

#[derive(Debug)]
pub struct ArrayIter<'a, 'b, R: Read<'b>, T> {
    decoder: &'a mut Decoder<'b, R>,
    len: Option<u64>,
    _mark: marker::PhantomData<&'a T>
}

impl<'a, 'b, R: Read<'b>, T: Decode<'b>> Iterator for ArrayIter<'a, 'b, R, T> {
    type Item = Result<T, Error<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.next().map(|_| None).transpose(),
                Ok(_)     => Some(T::decode(&mut self.decoder)),
                Err(e)    => Some(Err(e))
            }
            Some(0) => None,
            Some(n) => {
                self.len = Some(n - 1);
                Some(T::decode(&mut self.decoder))
            }
        }
    }
}

#[derive(Debug)]
pub struct MapIter<'a, 'b, R: Read<'b>, K, V> {
    decoder: &'a mut Decoder<'b, R>,
    len: Option<u64>,
    _mark: marker::PhantomData<&'a (K, V)>
}

impl<'a, 'b, R, K, V> Iterator for MapIter<'a, 'b, R, K, V>
where
    K: Decode<'b>,
    V: Decode<'b>,
    R: Read<'b>
{
    type Item = Result<(K, V), Error<R::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        fn pair<'b, R, K, V>(d: &mut Decoder<'b, R>) -> Result<(K, V), Error<R::Error>>
        where
            K: Decode<'b>,
            V: Decode<'b>,
            R: Read<'b>
        {
            Ok((K::decode(d)?, V::decode(d)?))
        }
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.next().map(|_| None).transpose(),
                Ok(_)  => Some(pair(&mut self.decoder)),
                Err(e) => Some(Err(e))
            }
            Some(0) => None,
            Some(n) => {
                self.len = Some(n - 1);
                Some(pair(&mut self.decoder))
            }
        }
    }
}

fn read_u16(b: &[u8]) -> u16 {
    let mut n = [0; 2];
    n.copy_from_slice(b);
    u16::from_be_bytes(n)
}

fn read_u32(b: &[u8]) -> u32 {
    let mut n = [0; 4];
    n.copy_from_slice(b);
    u32::from_be_bytes(n)
}

fn read_u64(b: &[u8]) -> u64 {
    let mut n = [0; 8];
    n.copy_from_slice(b);
    u64::from_be_bytes(n)
}

fn type_of(b: u8) -> u8 {
    b & 0b111_00000
}

fn info_of(b: u8) -> u8 {
    b & 0b000_11111
}

fn u64_to_usize<E>(n: u64) -> Result<usize, Error<E>> {
    n.try_into().map_err(|_| Error::Overflow(n, "u64->usize"))
}

