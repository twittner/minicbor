#![allow(clippy::unusual_byte_groupings)]

use crate::{ARRAY, BREAK, BYTES, MAP, SIMPLE, TAGGED, TEXT, SIGNED, UNSIGNED};
use crate::data::{Tag, Type};
use crate::decode::{Decode, Error};
use core::{char, f32, i8, i16, i32, i64};
use core::{convert::TryInto, marker, str};

// Convert an expression of an unsigned int type to a signed int type.
//
// This is used when decoding signed int types whose representation
// is -1 - unsigned int. Turning the unsigned int into an int may
// overflow which is what we check here.
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

// Cast an expression of an unsigned int type to a signed int type.
//
// This is used when a CBOR type is parsed as an unsigned int first
// but later turned into a signed int type, i.e. to support decoding
// unsigned types that are not too large when actually decoding a
// signed int.
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

/// A non-allocating CBOR decoder.
#[derive(Debug, Clone)]
pub struct Decoder<'b> {
    buf: &'b [u8],
    pos: usize
}

impl<'b> Decoder<'b> {
    /// Construct a `Decoder` for the given byte slice.
    pub fn new(bytes: &'b [u8]) -> Self {
        Decoder { buf: bytes, pos: 0 }
    }

    /// Decode any type that implements [`Decode`].
    pub fn decode<T: Decode<'b>>(&mut self) -> Result<T, Error> {
        T::decode(self)
    }

    /// Get the current decode position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Set the current decode position.
    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos
    }

    /// Get a decoding probe to look ahead what is coming next.
    ///
    /// This will not affect the decoding state of `self` and after the
    /// returned `Probe` has been dropped, decoding can continue from
    /// its current position as if `probe` was never called.
    pub fn probe<'a>(&'a mut self) -> Probe<'a, 'b> {
        Probe {
            decoder: self.clone(),
            _marker: marker::PhantomData
        }
    }

    /// Decode a `bool` value.
    pub fn bool(&mut self) -> Result<bool, Error> {
        match self.read()? {
            0xf4 => Ok(false),
            0xf5 => Ok(true),
            b    => Err(Error::TypeMismatch(Type::read(b), "expected bool"))
        }
    }

    /// Decode a `u8` value.
    pub fn u8(&mut self) -> Result<u8, Error> {
        match self.read()? {
            n @ 0 ..= 0x17 => Ok(n),
            0x18           => self.read(),
            b              => Err(Error::TypeMismatch(Type::read(b), "expected u8"))
        }
    }

    /// Decode a `u16` value.
    pub fn u16(&mut self) -> Result<u16, Error> {
        match self.read()? {
            n @ 0 ..= 0x17 => Ok(u16::from(n)),
            0x18           => self.read().map(u16::from),
            0x19           => self.read_slice(2).map(read_u16),
            b              => Err(Error::TypeMismatch(Type::read(b), "expected u16"))
        }
    }

    /// Decode a `u32` value.
    pub fn u32(&mut self) -> Result<u32, Error> {
        match self.read()? {
            n @ 0 ..= 0x17 => Ok(u32::from(n)),
            0x18           => self.read().map(u32::from),
            0x19           => self.read_slice(2).map(read_u16).map(u32::from),
            0x1a           => self.read_slice(4).map(read_u32),
            b              => Err(Error::TypeMismatch(Type::read(b), "expected u32"))
        }
    }

    /// Decode a `u64` value.
    pub fn u64(&mut self) -> Result<u64, Error> {
        let n = self.read()?;
        self.unsigned(n)
    }

    /// Decode an `i8` value.
    pub fn i8(&mut self) -> Result<i8, Error> {
        match self.read()? {
            n @ 0x00 ..= 0x17 => Ok(n as i8),
            0x18              => u_as_i!(self.read()?, i8, i8::MAX as u8, "u8->i8"),
            n @ 0x20 ..= 0x37 => Ok(-1 - (n - 0x20) as i8),
            0x38              => u_to_i!(self.read()?, i8, i8::MAX as u8, "u8->i8"),
            b                 => Err(Error::TypeMismatch(Type::read(b), "expected i8"))
        }
    }

    /// Decode an `i16` value.
    pub fn i16(&mut self) -> Result<i16, Error> {
        match self.read()? {
            n @ 0x00 ..= 0x17 => Ok(i16::from(n)),
            0x18              => self.read().map(i16::from),
            0x19              => u_as_i!(self.read_slice(2).map(read_u16)?, i16, i16::MAX as u16, "u16->i16"),
            n @ 0x20 ..= 0x37 => Ok(-1 - i16::from(n - 0x20)),
            0x38              => self.read().map(|n| -1 - i16::from(n)),
            0x39              => u_to_i!(self.read_slice(2).map(read_u16)?, i16, i16::MAX as u16, "u16->i16"),
            b                 => Err(Error::TypeMismatch(Type::read(b), "expected i16"))
        }
    }

    /// Decode an `i32` value.
    pub fn i32(&mut self) -> Result<i32, Error> {
        match self.read()? {
            n @ 0x00 ..= 0x17 => Ok(i32::from(n)),
            0x18              => self.read().map(i32::from),
            0x19              => self.read_slice(2).map(read_u16).map(i32::from),
            0x1a              => u_as_i!(self.read_slice(4).map(read_u32)?, i32, i32::MAX as u32, "u32->i32"),
            n @ 0x20 ..= 0x37 => Ok(-1 - i32::from(n - 0x20)),
            0x38              => self.read().map(|n| -1 - i32::from(n)),
            0x39              => self.read_slice(2).map(read_u16).map(|n| -1 - i32::from(n)),
            0x3a              => u_to_i!(self.read_slice(4).map(read_u32)?, i32, i32::MAX as u32, "u32->i32"),
            b                 => Err(Error::TypeMismatch(Type::read(b), "expected i32"))
        }
    }

    /// Decode an `i64` value.
    pub fn i64(&mut self) -> Result<i64, Error> {
        match self.read()? {
            n @ 0x00 ..= 0x17 => Ok(i64::from(n)),
            0x18              => self.read().map(i64::from),
            0x19              => self.read_slice(2).map(read_u16).map(i64::from),
            0x1a              => self.read_slice(4).map(read_u32).map(i64::from),
            0x1b              => u_as_i!(self.read_slice(8).map(read_u64)?, i64, i64::MAX as u64, "u64->i64"),
            n @ 0x20 ..= 0x37 => Ok(-1 - i64::from(n - 0x20)),
            0x38              => self.read().map(|n| -1 - i64::from(n)),
            0x39              => self.read_slice(2).map(read_u16).map(|n| -1 - i64::from(n)),
            0x3a              => self.read_slice(4).map(read_u32).map(|n| -1 - i64::from(n)),
            0x3b              => u_to_i!(self.read_slice(8).map(read_u64)?, i64, i64::MAX as u64, "u64->i64"),
            b                 => Err(Error::TypeMismatch(Type::read(b), "expected i64"))
        }
    }

    /// Decode a half float (`f16`) and return it in an `f32`.
    ///
    /// Only available when the feature `half` is present.
    #[cfg(feature = "half")]
    pub fn f16(&mut self) -> Result<f32, Error> {
        let b = self.read()?;
        if 0xf9 != b {
            return Err(Error::TypeMismatch(Type::read(b), "expected f16"))
        }
        let mut n = [0; 2];
        n.copy_from_slice(self.read_slice(2)?);
        Ok(half::f16::from_bits(u16::from_be_bytes(n)).to_f32())
    }

    /// Decode an `f32` value.
    pub fn f32(&mut self) -> Result<f32, Error> {
        match self.current()? {
            #[cfg(feature = "half")]
            0xf9 => self.f16(),
            0xfa => {
                self.read()?;
                let mut n = [0; 4];
                n.copy_from_slice(self.read_slice(4)?);
                Ok(f32::from_be_bytes(n))
            }
            b => Err(Error::TypeMismatch(Type::read(b), "expected f32"))
        }
    }

    /// Decode an `f64` value.
    pub fn f64(&mut self) -> Result<f64, Error> {
        match self.current()? {
            #[cfg(feature = "half")]
            0xf9 => self.f16().map(f64::from),
            0xfa => self.f32().map(f64::from),
            0xfb => {
                self.read()?;
                let mut n = [0; 8];
                n.copy_from_slice(self.read_slice(8)?);
                Ok(f64::from_be_bytes(n))
            }
            b => Err(Error::TypeMismatch(Type::read(b), "expected f64"))
        }
    }

    /// Decode a `char` value.
    pub fn char(&mut self) -> Result<char, Error> {
        let n = self.u32()?;
        char::from_u32(n).ok_or(Error::InvalidChar(n))
    }

    /// Decode a byte slice.
    ///
    /// This only decodes byte slices of definite lengths.
    /// See [`Decoder::bytes_iter`] for indefinite byte slice support.
    pub fn bytes(&mut self) -> Result<&'b [u8], Error> {
        let b = self.read()?;
        if BYTES != type_of(b) || info_of(b) == 31 {
            return Err(Error::TypeMismatch(Type::read(b), "expected bytes (definite length)"))
        }
        let n = u64_to_usize(self.unsigned(info_of(b))?)?;
        self.read_slice(n)
    }

    /// Iterate over byte slices.
    ///
    /// This supports indefinite byte slices by returing a byte slice on each
    /// iterator step. If a single definite slice is decoded the iterator will
    /// only yield one item.
    pub fn bytes_iter(&mut self) -> Result<BytesIter<'_, 'b>, Error> {
        let b = self.read()?;
        if BYTES != type_of(b) {
            return Err(Error::TypeMismatch(Type::read(b), "expected bytes"))
        }
        match info_of(b) {
            31 => Ok(BytesIter { decoder: self, len: None }),
            n  => {
                let len = u64_to_usize(self.unsigned(n)?)?;
                Ok(BytesIter { decoder: self, len: Some(len) })
            }
        }
    }

    /// Decode a string slice.
    ///
    /// This only decodes string slices of definite lengths.
    /// See [`Decoder::str_iter`] for indefinite string slice support.
    pub fn str(&mut self) -> Result<&'b str, Error> {
        let b = self.read()?;
        if TEXT != type_of(b) || info_of(b) == 31 {
            return Err(Error::TypeMismatch(Type::read(b), "expected text (definite length)"))
        }
        let n = u64_to_usize(self.unsigned(info_of(b))?)?;
        let d = self.read_slice(n)?;
        str::from_utf8(d).map_err(Error::from)
    }

    /// Iterate over string slices.
    ///
    /// This supports indefinite string slices by returing a string slice on
    /// each iterator step. If a single definite slice is decoded the iterator
    /// will only yield one item.
    pub fn str_iter(&mut self) -> Result<StrIter<'_, 'b>, Error> {
        let b = self.read()?;
        if TEXT != type_of(b) {
            return Err(Error::TypeMismatch(Type::read(b), "expected text"))
        }
        match info_of(b) {
            31 => Ok(StrIter { decoder: self, len: None }),
            n  => {
                let len = u64_to_usize(self.unsigned(n)?)?;
                Ok(StrIter { decoder: self, len: Some(len) })
            }
        }
    }

    /// Begin decoding an array.
    ///
    /// CBOR arrays are heterogenous collections and may be of indefinite
    /// length. If the length is known it is returned as a `Some`, for
    /// indefinite arrays a `None` is returned.
    pub fn array(&mut self) -> Result<Option<u64>, Error> {
        let b = self.read()?;
        if ARRAY != type_of(b) {
            return Err(Error::TypeMismatch(Type::read(b), "expected array"))
        }
        match info_of(b) {
            31 => Ok(None),
            n  => Ok(Some(self.unsigned(n)?))
        }
    }

    /// Iterate over all array elements.
    ///
    /// This supports indefinite and definite length arrays and uses the
    /// [`Decode`] trait to decode each element. Consequently *only
    /// homogenous arrays are supported by this method*.
    pub fn array_iter<T>(&mut self) -> Result<ArrayIter<'_, 'b, T>, Error>
    where
        T: Decode<'b>
    {
        let len = self.array()?;
        Ok(ArrayIter { decoder: self, len, _mark: marker::PhantomData })
    }

    /// Begin decoding a map.
    ///
    /// CBOR maps are heterogenous collections (both in keys and in values)
    /// and may be of indefinite length. If the length is known it is returned
    /// as a `Some`, for indefinite maps a `None` is returned.
    pub fn map(&mut self) -> Result<Option<u64>, Error> {
        let b = self.read()?;
        if MAP != type_of(b) {
            return Err(Error::TypeMismatch(Type::read(b), "expected map"))
        }
        match info_of(b) {
            31 => Ok(None),
            n  => Ok(Some(self.unsigned(n)?))
        }
    }

    /// Iterate over all map entries.
    ///
    /// This supports indefinite and definite length maps and uses the
    /// [`Decode`] trait to decode each key and value. Consequently *only
    /// homogenous maps are supported by this method*.
    pub fn map_iter<K, V>(&mut self) -> Result<MapIter<'_, 'b, K, V>, Error>
    where
        K: Decode<'b>,
        V: Decode<'b>
    {
        let len = self.map()?;
        Ok(MapIter { decoder: self, len, _mark: marker::PhantomData })
    }

    /// Decode a CBOR tag.
    pub fn tag(&mut self) -> Result<Tag, Error> {
        let b = self.read()?;
        if TAGGED != type_of(b) {
            return Err(Error::TypeMismatch(Type::read(b), "expected tag"))
        }
        self.unsigned(info_of(b)).map(Tag::from)
    }

    /// Decode a CBOR simple value.
    pub fn simple(&mut self) -> Result<u8, Error> {
        match self.read()? {
            n @ SIMPLE ..= 0xf3 => Ok(n - SIMPLE),
            0xf8                => self.read(),
            n                   => Err(Error::TypeMismatch(Type::read(n), "expected simple value"))
        }
    }

    /// Inspect the CBOR type at the current position.
    pub fn datatype(&self) -> Result<Type, Error> {
        self.current().map(Type::read)
    }

    /// Skip over the current CBOR value.
    pub fn skip(&mut self) -> Result<(), Error> {
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
                        nrounds = nrounds.saturating_add(n.saturating_mul(2))
                    } else {
                        irounds = irounds.saturating_add(1)
                    }
                TAGGED ..= 0xdb => {
                    self.read().and_then(|n| self.unsigned(info_of(n)))?;
                    nrounds = nrounds.saturating_add(1)
                }
                SIMPLE ..= 0xfb => {
                    self.read().and_then(|n| self.unsigned(info_of(n)))?;
                }
                BREAK => {
                    self.read()?;
                    irounds = irounds.saturating_sub(1)
                }
                other => return Err(Error::TypeMismatch(Type::read(other), "unknown type"))
            }
            nrounds = nrounds.saturating_sub(1)
        }

        Ok(())
    }

    /// Decode a `u64` value beginning with `b`.
    fn unsigned(&mut self, b: u8) -> Result<u64, Error> {
        match b {
            n @ 0 ..= 0x17 => Ok(u64::from(n)),
            0x18           => self.read().map(u64::from),
            0x19           => self.read_slice(2).map(read_u16).map(u64::from),
            0x1a           => self.read_slice(4).map(read_u32).map(u64::from),
            0x1b           => self.read_slice(8).map(read_u64),
            _              => Err(Error::TypeMismatch(Type::read(b), "expected u64"))
        }
    }

    /// Get the byte at the current position.
    fn current(&self) -> Result<u8, Error> {
        if let Some(b) = self.buf.get(self.pos) {
            return Ok(*b)
        }
        Err(Error::EndOfInput)
    }

    /// Consume and return the byte at the current position.
    fn read(&mut self) -> Result<u8, Error> {
        if let Some(b) = self.buf.get(self.pos) {
            self.pos += 1;
            return Ok(*b)
        }
        Err(Error::EndOfInput)
    }

    /// Consume and return *n* bytes starting at the current position.
    fn read_slice(&mut self, n: usize) -> Result<&'b [u8], Error> {
        if let Some(b) = self.buf.get(self.pos .. self.pos + n) {
            self.pos += n;
            return Ok(b)
        }
        Err(Error::EndOfInput)
    }
}

/// An iterator over byte slices.
///
/// Returned from [`Decoder::bytes_iter`].
#[derive(Debug)]
pub struct BytesIter<'a, 'b> {
    decoder: &'a mut Decoder<'b>,
    len: Option<usize>
}

impl<'a, 'b> Iterator for BytesIter<'a, 'b> {
    type Item = Result<&'b [u8], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.read().map(|_| None).transpose(),
                Ok(_)     => Some(self.decoder.bytes()),
                Err(e)    => Some(Err(e))
            }
            Some(0) => None,
            Some(n) => {
                self.len = Some(0);
                Some(self.decoder.read_slice(n))
            }
        }
    }
}

/// An iterator over string slices.
///
/// Returned from [`Decoder::str_iter`].
#[derive(Debug)]
pub struct StrIter<'a, 'b> {
    decoder: &'a mut Decoder<'b>,
    len: Option<usize>
}

impl<'a, 'b> Iterator for StrIter<'a, 'b> {
    type Item = Result<&'b str, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.read().map(|_| None).transpose(),
                Ok(_)     => Some(self.decoder.str()),
                Err(e)    => Some(Err(e))
            }
            Some(0) => None,
            Some(n) => {
                self.len = Some(0);
                Some(self.decoder.read_slice(n).and_then(|d| str::from_utf8(d).map_err(Error::from)))
            }
        }
    }
}

/// An iterator over array elements.
///
/// Returned from [`Decoder::array_iter`].
#[derive(Debug)]
pub struct ArrayIter<'a, 'b, T> {
    decoder: &'a mut Decoder<'b>,
    len: Option<u64>,
    _mark: marker::PhantomData<&'a T>
}

impl<'a, 'b, T: Decode<'b>> Iterator for ArrayIter<'a, 'b, T> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.read().map(|_| None).transpose(),
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

/// An iterator over map entries.
///
/// Returned from [`Decoder::map_iter`].
#[derive(Debug)]
pub struct MapIter<'a, 'b, K, V> {
    decoder: &'a mut Decoder<'b>,
    len: Option<u64>,
    _mark: marker::PhantomData<&'a (K, V)>
}

impl<'a, 'b, K, V> Iterator for MapIter<'a, 'b, K, V>
where
    K: Decode<'b>,
    V: Decode<'b>
{
    type Item = Result<(K, V), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        fn pair<'b, K, V>(d: &mut Decoder<'b>) -> Result<(K, V), Error>
        where
            K: Decode<'b>,
            V: Decode<'b>
        {
            Ok((K::decode(d)?, V::decode(d)?))
        }
        match self.len {
            None => match self.decoder.current() {
                Ok(BREAK) => self.decoder.read().map(|_| None).transpose(),
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

/// A decoding probe to to look ahead what comes next.
///
/// A `Probe` derefs to [`Decoder`] and thus can be used like one without
/// affecting the decoder from which it was created.
//
// The current implementation just clones `Decoder` as it is very cheap
// to do so. `Probe` is nevertheless introduced to discourage use of
// `Decoder::clone` in client code for this purpose so that it stays
// independent of the current implementation.
// With a more heavyweight `Decoder`, `Probe` could only store a reference
// and the current position which it restores in a `Drop` impl.
#[derive(Debug)]
pub struct Probe<'a, 'b> {
    decoder: Decoder<'b>,
    _marker: marker::PhantomData<&'a mut ()>
}

impl<'b> core::ops::Deref for Probe<'_, 'b> {
    type Target = Decoder<'b>;

    fn deref(&self) -> &Self::Target {
        &self.decoder
    }
}

impl<'b> core::ops::DerefMut for Probe<'_, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decoder
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

/// Get the major type info of the given byte (highest 3 bits).
fn type_of(b: u8) -> u8 {
    b & 0b111_00000
}

/// Get the additionl type info of the given byte (lowest 5 bits).
fn info_of(b: u8) -> u8 {
    b & 0b000_11111
}

fn u64_to_usize(n: u64) -> Result<usize, Error> {
    n.try_into().map_err(|_| Error::Overflow(n, "u64->usize"))
}

