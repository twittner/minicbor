//! Traits and types for decoding CBOR.
//!
//! This module defines the trait [`Decode`] and the actual [`Decoder`].

mod decoder;
mod error;

pub use decoder::{Decoder, Probe};
pub use decoder::{ArrayIter, BytesIter, MapIter, StrIter};
pub use error::Error;

/// A type that can be decoded from CBOR.
pub trait Decode<'b>: Sized {
    /// Decode a value using the given `Decoder`.
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error>;
}

#[cfg(feature = "std")]
impl<'b, T: Decode<'b>> Decode<'b> for Box<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        T::decode(d).map(Box::new)
    }
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
impl<'b, T> Decode<'b> for std::borrow::Cow<'_, T>
where
    T: std::borrow::ToOwned + ?Sized,
    T::Owned: Decode<'b>
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(std::borrow::Cow::Owned)
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for String {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str().map(String::from)
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
impl<'b, T> Decode<'b> for std::collections::BinaryHeap<T>
where
    T: Decode<'b> + Ord
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let iter: ArrayIter<T> = d.array_iter()?;
        let mut v = std::collections::BinaryHeap::new();
        for x in iter {
            v.push(x?)
        }
        Ok(v)
    }
}

#[cfg(feature = "std")]
impl<'b, T> Decode<'b> for std::collections::HashSet<T>
where
    T: Decode<'b> + Eq + std::hash::Hash
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let iter: ArrayIter<T> = d.array_iter()?;
        let mut v = std::collections::HashSet::new();
        for x in iter {
            v.insert(x?);
        }
        Ok(v)
    }
}

#[cfg(feature = "std")]
impl<'b, T> Decode<'b> for std::collections::BTreeSet<T>
where
    T: Decode<'b> + Ord
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let iter: ArrayIter<T> = d.array_iter()?;
        let mut v = std::collections::BTreeSet::new();
        for x in iter {
            v.insert(x?);
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

impl<'b, T> Decode<'b> for core::marker::PhantomData<T> {
    fn decode(_: &mut Decoder<'b>) -> Result<Self, Error> {
        Ok(core::marker::PhantomData)
    }
}

impl<'b> Decode<'b> for () {
    fn decode(_: &mut Decoder<'b>) -> Result<Self, Error> {
        Ok(())
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

macro_rules! decode_basic {
    ($($t:ident)*) => {
        $(
            impl<'b> Decode<'b> for $t {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    d.$t()
                }
            }
        )*
    }
}

decode_basic!(u8 i8 u16 i16 u32 i32 u64 i64 bool f32 f64 char);

macro_rules! decode_nonzero {
    ($($t:ty, $msg:expr)*) => {
        $(
            impl<'b> Decode<'b> for $t {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    Ok(<$t>::new(Decode::decode(d)?).ok_or(Error::Message($msg))?)
                }
            }
        )*
    }
}

decode_nonzero! {
    core::num::NonZeroU8,  "unexpected 0 when decoding a `NonZeroU8`"
    core::num::NonZeroU16, "unexpected 0 when decoding a `NonZeroU16`"
    core::num::NonZeroU32, "unexpected 0 when decoding a `NonZeroU32`"
    core::num::NonZeroU64, "unexpected 0 when decoding a `NonZeroU64`"
    core::num::NonZeroI8,  "unexpected 0 when decoding a `NonZeroI8`"
    core::num::NonZeroI16, "unexpected 0 when decoding a `NonZeroI16`"
    core::num::NonZeroI32, "unexpected 0 when decoding a `NonZeroI32`"
    core::num::NonZeroI64, "unexpected 0 when decoding a `NonZeroI64`"
}

#[cfg(feature = "std")]
macro_rules! decode_sequential {
    ($($t:ty, $push:ident)*) => {
        $(
            impl<'b, T: Decode<'b>> Decode<'b> for $t {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    let iter: ArrayIter<T> = d.array_iter()?;
                    let mut v = <$t>::new();
                    for x in iter {
                        v.$push(x?)
                    }
                    Ok(v)
                }
            }
        )*
    }
}

#[cfg(feature = "std")]
decode_sequential! {
    Vec<T>, push
    std::collections::VecDeque<T>, push_back
    std::collections::LinkedList<T>, push_back
}

macro_rules! decode_arrays {
    ($($n:expr)*) => {
        $(
            impl<'b, T: Decode<'b> + Default> Decode<'b> for [T; $n] {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    let iter: ArrayIter<T> = d.array_iter()?;
                    let mut a: [T; $n] = Default::default();
                    let mut i = 0;
                    for x in iter {
                        if i >= a.len() {
                            let msg = concat!("array has more than ", $n, " elements");
                            return Err(Error::Message(msg))
                        }
                        a[i] = x?;
                        i += 1;
                    }
                    if i < a.len() {
                        let msg = concat!("array has less than ", $n, " elements");
                        return Err(Error::Message(msg))
                    }
                    Ok(a)
                }
            }
        )*
    }
}

decode_arrays!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

macro_rules! decode_tuples {
    ($( $len:expr => { $($T:ident)+ } )+) => {
        $(
            impl<'b, $($T: Decode<'b>),+> Decode<'b> for ($($T,)+) {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    let n = d.array()?;
                    if n != Some($len) {
                        return Err(Error::Message(concat!("invalid ", $len, "-tuple length")))
                    }
                    Ok(($($T::decode(d)?,)+))
                }
            }
        )+
    }
}

decode_tuples! {
    1  => { A }
    2  => { A B }
    3  => { A B C }
    4  => { A B C D }
    5  => { A B C D E }
    6  => { A B C D E F }
    7  => { A B C D E F G }
    8  => { A B C D E F G H }
    9  => { A B C D E F G H I }
    10 => { A B C D E F G H I J }
    11 => { A B C D E F G H I J K }
    12 => { A B C D E F G H I J K L }
    13 => { A B C D E F G H I J K L M }
    14 => { A B C D E F G H I J K L M N }
    15 => { A B C D E F G H I J K L M N O }
    16 => { A B C D E F G H I J K L M N O P }
}

macro_rules! decode_fields {
    ($d:ident | $($n:literal $x:ident => $t:ty ; $msg:literal)*) => {
        $(let mut $x = None;)*

        match $d.array()? {
            Some(n) => for i in 0 .. n {
                match i {
                    $($n => $x = Some(Decode::decode($d)?),)*
                    _    => $d.skip()?
                }
            }
            None => {
                let mut i = 0;
                while $d.datatype()? != crate::data::Type::Break {
                    match i {
                        $($n => $x = Some(Decode::decode($d)?),)*
                        _    => $d.skip()?
                    }
                    i += 1
                }
                $d.skip()?
            }
        }

        $(let $x = if let Some(x) = $x {
            x
        } else {
            return Err(Error::MissingValue($n, $msg))
        };)*
    }
}

impl<'b> Decode<'b> for core::time::Duration {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 secs  => u64 ; "Duration::secs"
            1 nanos => u32 ; "Duration::nanos"
        }
        Ok(core::time::Duration::new(secs, nanos))
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::IpAddr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        if Some(2) != d.array()? {
            return Err(Error::Message("expected enum (2-element array)"))
        }
        match d.u64()? {
            0 => Ok(std::net::Ipv4Addr::decode(d)?.into()),
            1 => Ok(std::net::Ipv6Addr::decode(d)?.into()),
            n => Err(Error::UnknownVariant(n))
        }
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::Ipv4Addr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let octets: [u8; 4] = Decode::decode(d)?;
        Ok(octets.into())
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::Ipv6Addr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let octets: [u8; 16] = Decode::decode(d)?;
        Ok(octets.into())
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::SocketAddr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        if Some(2) != d.array()? {
            return Err(Error::Message("expected enum (2-element array)"))
        }
        match d.u64()? {
            0 => Ok(std::net::SocketAddrV4::decode(d)?.into()),
            1 => Ok(std::net::SocketAddrV6::decode(d)?.into()),
            n => Err(Error::UnknownVariant(n))
        }
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::SocketAddrV4 {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 ip   => std::net::Ipv4Addr ; "SocketAddrV4::ip"
            1 port => u16                ; "SocketAddrV4::port"
        }
        Ok(std::net::SocketAddrV4::new(ip, port))
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::SocketAddrV6 {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 ip   => std::net::Ipv6Addr ; "SocketAddrV6::ip"
            1 port => u16                ; "SocketAddrV6::port"
        }
        Ok(std::net::SocketAddrV6::new(ip, port, 0, 0))
    }
}

