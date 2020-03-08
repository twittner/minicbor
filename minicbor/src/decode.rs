//! Traits and types for decoding CBOR.
//!
//! This module defines the trait [`Decode`] and the actual [`Decoder`].

mod decoder;
mod error;

pub use decoder::Decoder;
pub use decoder::{ArrayIter, BytesIter, MapIter, StrIter};
pub use error::Error;

/// A type that can be decoded from CBOR.
pub trait Decode<'b>: Sized {
    /// Decode a value using the given `Decoder`.
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error>;
}

impl<'a, 'b: 'a> Decode<'b> for &'a [u8] {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.bytes()
    }
}

impl<'a, 'b: 'a> Decode<'b> for &'a str {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str()
    }
}

#[cfg(feature = "std")]
impl<'a, 'b: 'a> Decode<'b> for std::borrow::Cow<'a, [u8]> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.bytes().map(std::borrow::Cow::Borrowed)
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for String {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str().map(String::from)
    }
}

#[cfg(feature = "std")]
impl<'a, 'b: 'a> Decode<'b> for std::borrow::Cow<'a, str> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str().map(std::borrow::Cow::Borrowed)
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for Option<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        if crate::data::Type::Null == d.datatype()? {
            d.skip()?;
            return Ok(None)
        }
        T::decode(d).map(Some)
    }
}

#[cfg(feature = "std")]
impl<'b, T: Decode<'b>> Decode<'b> for Vec<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let mut v = Vec::new();
        let iter: ArrayIter<T> = d.array_iter()?;
        for x in iter {
            v.push(x?)
        }
        Ok(v)
    }
}

#[cfg(feature = "std")]
impl<'b, K, V> Decode<'b> for std::collections::HashMap<K, V>
where
    K: Decode<'b> + Eq + std::hash::Hash,
    V: Decode<'b>
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let mut m = std::collections::HashMap::new();
        let iter: MapIter<K, V> = d.map_iter()?;
        for x in iter {
            let (k, v) = x?;
            m.insert(k, v);
        }
        Ok(m)
    }
}

#[cfg(feature = "std")]
impl<'b, K, V> Decode<'b> for std::collections::BTreeMap<K, V>
where
    K: Decode<'b> + Eq + Ord,
    V: Decode<'b>
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let mut m = std::collections::BTreeMap::new();
        let iter: MapIter<K, V> = d.map_iter()?;
        for x in iter {
            let (k, v) = x?;
            m.insert(k, v);
        }
        Ok(m)
    }
}

#[cfg(target_pointer_width = "32")]
impl<'b> Decode<'b> for usize {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.u32().map(|n| n as usize)
    }
}

#[cfg(target_pointer_width = "64")]
impl<'b> Decode<'b> for usize {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.u64().map(|n| n as usize)
    }
}

#[cfg(target_pointer_width = "32")]
impl<'b> Decode<'b> for isize {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.i32().map(|n| n as isize)
    }
}

#[cfg(target_pointer_width = "64")]
impl<'b> Decode<'b> for isize {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.i64().map(|n| n as isize)
    }
}

macro_rules! decode_impls {
    ($($t:ident)*) => {
        $(
            impl<'b> $crate::decode::Decode<'b> for $t {
                fn decode(d: &mut $crate::decode::Decoder<'b>) -> Result<Self, Error> {
                    d.$t()
                }
            }
        )*
    }
}

decode_impls!(u8 i8 u16 i16 u32 i32 u64 i64 bool f32 f64 char);

macro_rules! decode_arrays {
    ($($n:expr)*) => {
        $(
            impl<'b, T> $crate::decode::Decode<'b> for [T; $n]
            where
                T: $crate::decode::Decode<'b> + Default,
            {
                fn decode(d: &mut $crate::decode::Decoder<'b>) -> Result<Self, Error> {
                    let iter: $crate::decode::ArrayIter<T> = d.array_iter()?;
                    let mut a: [T; $n] = ::core::default::Default::default();
                    let mut i = 0;
                    for x in iter {
                        if i >= a.len() {
                            let msg = "array lengths do not match";
                            return Err($crate::decode::Error::Overflow(i as u64, msg))
                        }
                        a[i] = x?;
                        i += 1;
                    }
                    debug_assert_eq!(i, a.len());
                    Ok(a)
                }
            }
        )*
    }
}

decode_arrays!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

#[cfg(feature = "smallvec")]
macro_rules! decode_smallvecs {
    ($($n:expr)*) => {
        $(
            impl<'b, T> $crate::decode::Decode<'b> for smallvec::SmallVec::<[T; $n]>
            where
                T: $crate::decode::Decode<'b>
            {
                fn decode(d: &mut $crate::decode::Decoder<'b>) -> Result<Self, Error> {
                    let iter: $crate::decode::ArrayIter<T> = d.array_iter()?;
                    let mut v = smallvec::SmallVec::<[T; $n]>::new();
                    for x in iter {
                        v.push(x?)
                    }
                    Ok(v)
                }
            }
        )*
    }
}

#[cfg(feature = "smallvec")]
decode_smallvecs!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

