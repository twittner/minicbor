//! Traits and types for decoding CBOR.
//!
//! This module defines the trait [`Decode`] and the actual [`Decoder`].

mod decoder;
mod error;

pub use decoder::{Decoder, Probe};
pub use decoder::{ArrayIter, BytesIter, MapIter, StrIter};
pub use error::Error;

#[cfg(feature = "half")]
mod tokens;

#[cfg(feature = "half")]
pub use tokens::{Token, Tokenizer};

/// A type that can be decoded from CBOR.
pub trait Decode<'b>: Sized {
    /// Decode a value using the given `Decoder`.
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error>;

    /// If possible, return a nil value of `Self`.
    ///
    /// This method is primarily used by `minicbor-derive` and allows
    /// creating a special value denoting the absence of a "real" value if
    /// no CBOR value is present. The canonical example of a type where
    /// this is sensible is the `Option` type, whose `Decode::nil` method
    /// would return `Some(None)`.
    ///
    /// With the exception of `Option<_>` all types `T` are considered
    /// mandatory by default, i.e. `T::nil()` returns `None`. Missing values
    /// of `T` therefore cause decoding errors in derived `Decode`
    /// implementations.
    ///
    /// NB: A type implementing `Decode` with an overriden `Decode::nil`
    /// method should also override `Encode::is_nil` if it implements `Encode`
    /// at all.
    fn nil() -> Option<Self> {
        None
    }
}

#[cfg(feature = "alloc")]
impl<'b, T: Decode<'b>> Decode<'b> for alloc::boxed::Box<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        T::decode(d).map(alloc::boxed::Box::new)
    }
}

impl<'a, 'b: 'a> Decode<'b> for &'a str {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str()
    }
}

#[cfg(feature = "alloc")]
impl<'b, T> Decode<'b> for alloc::borrow::Cow<'_, T>
where
    T: alloc::borrow::ToOwned + ?Sized,
    T::Owned: Decode<'b>
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(alloc::borrow::Cow::Owned)
    }
}

#[cfg(feature = "alloc")]
impl<'b> Decode<'b> for alloc::string::String {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str().map(alloc::string::String::from)
    }
}

#[cfg(feature = "alloc")]
impl<'b> Decode<'b> for alloc::boxed::Box<str> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str().map(Into::into)
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for Option<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        if crate::data::Type::Null == d.datatype()? {
            d.limited_skip()?;
            return Ok(None)
        }
        T::decode(d).map(Some)
    }

    fn nil() -> Option<Self> {
        Some(None)
    }
}

impl<'b, T, E> Decode<'b> for Result<T, E>
where
    T: Decode<'b>,
    E: Decode<'b>
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        if Some(2) != d.array()? {
            return Err(Error::message("expected enum (2-element array)").at(p))
        }
        let p = d.position();
        match d.u32()? {
            0 => T::decode(d).map(Ok),
            1 => E::decode(d).map(Err),
            n => Err(Error::unknown_variant(n).at(p))
        }
    }
}

#[cfg(feature = "alloc")]
impl<'b, T> Decode<'b> for alloc::collections::BinaryHeap<T>
where
    T: Decode<'b> + Ord
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let iter: ArrayIter<T> = d.array_iter()?;
        let mut v = alloc::collections::BinaryHeap::new();
        for x in iter {
            v.push(x?)
        }
        Ok(v)
    }
}

#[cfg(feature = "std")]
impl<'b, T, S> Decode<'b> for std::collections::HashSet<T, S>
where
    T: Decode<'b> + Eq + std::hash::Hash,
    S: std::hash::BuildHasher + std::default::Default
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let iter: ArrayIter<T> = d.array_iter()?;
        let mut v = std::collections::HashSet::default();
        for x in iter {
            v.insert(x?);
        }
        Ok(v)
    }
}

#[cfg(feature = "alloc")]
impl<'b, T> Decode<'b> for alloc::collections::BTreeSet<T>
where
    T: Decode<'b> + Ord
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let iter: ArrayIter<T> = d.array_iter()?;
        let mut v = alloc::collections::BTreeSet::new();
        for x in iter {
            v.insert(x?);
        }
        Ok(v)
    }
}

#[cfg(feature = "std")]
impl<'b, K, V, S> Decode<'b> for std::collections::HashMap<K, V, S>
where
    K: Decode<'b> + Eq + std::hash::Hash,
    V: Decode<'b>,
    S: std::hash::BuildHasher + std::default::Default
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let mut m = std::collections::HashMap::default();
        let iter: MapIter<K, V> = d.map_iter()?;
        for x in iter {
            let (k, v) = x?;
            m.insert(k, v);
        }
        Ok(m)
    }
}

#[cfg(feature = "alloc")]
impl<'b, K, V> Decode<'b> for alloc::collections::BTreeMap<K, V>
where
    K: Decode<'b> + Eq + Ord,
    V: Decode<'b>
{
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let mut m = alloc::collections::BTreeMap::new();
        let iter: MapIter<K, V> = d.map_iter()?;
        for x in iter {
            let (k, v) = x?;
            m.insert(k, v);
        }
        Ok(m)
    }
}

impl<'b, T> Decode<'b> for core::marker::PhantomData<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        if Some(0) != d.array()? {
            return Err(Error::message("expected phantom data, i.e. an empty array").at(p))
        }
        Ok(core::marker::PhantomData)
    }
}

impl<'b> Decode<'b> for () {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        if Some(0) != d.array()? {
            return Err(Error::message("expected unit, i.e. an empty array").at(p))
        }
        Ok(())
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::num::Wrapping<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(core::num::Wrapping)
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

impl<'b> Decode<'b> for crate::data::Int {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.int()
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
                    let p = d.position();
                    <$t>::new(Decode::decode(d)?).ok_or_else(|| Error::message($msg).at(p))
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

#[cfg(any(atomic32, atomic64))]
macro_rules! decode_atomic {
    ($($t:ty)*) => {
        $(
            impl<'b> Decode<'b> for $t {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    d.decode().map(<$t>::new)
                }
            }
        )*
    }
}

#[cfg(atomic32)]
decode_atomic! {
    core::sync::atomic::AtomicBool
    core::sync::atomic::AtomicU8
    core::sync::atomic::AtomicU16
    core::sync::atomic::AtomicU32
    core::sync::atomic::AtomicUsize
    core::sync::atomic::AtomicI8
    core::sync::atomic::AtomicI16
    core::sync::atomic::AtomicI32
    core::sync::atomic::AtomicIsize
}

#[cfg(atomic64)]
decode_atomic! {
    core::sync::atomic::AtomicBool
    core::sync::atomic::AtomicU8
    core::sync::atomic::AtomicU16
    core::sync::atomic::AtomicU32
    core::sync::atomic::AtomicU64
    core::sync::atomic::AtomicUsize
    core::sync::atomic::AtomicI8
    core::sync::atomic::AtomicI16
    core::sync::atomic::AtomicI32
    core::sync::atomic::AtomicI64
    core::sync::atomic::AtomicIsize
}

#[cfg(feature = "alloc")]
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

#[cfg(feature = "alloc")]
decode_sequential! {
    alloc::vec::Vec<T>, push
    alloc::collections::VecDeque<T>, push_back
    alloc::collections::LinkedList<T>, push_back
}

macro_rules! decode_arrays {
    ($($n:expr)*) => {
        $(
            impl<'b, T: Decode<'b> + Default> Decode<'b> for [T; $n] {
                fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
                    let p = d.position();
                    let iter: ArrayIter<T> = d.array_iter()?;
                    let mut a: [T; $n] = Default::default();
                    let mut i = 0;
                    for x in iter {
                        if i >= a.len() {
                            let msg = concat!("array has more than ", $n, " elements");
                            return Err(Error::message(msg).at(p))
                        }
                        a[i] = x?;
                        i += 1;
                    }
                    if i < a.len() {
                        let msg = concat!("array has less than ", $n, " elements");
                        return Err(Error::message(msg).at(p))
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
                    let p = d.position();
                    let n = d.array()?;
                    if n != Some($len) {
                        return Err(Error::message(concat!("invalid ", $len, "-tuple length")).at(p))
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
        $(let mut $x : core::option::Option<$t> = None;)*

        let p = $d.position();

        match $d.array()? {
            Some(n) => for i in 0 .. n {
                match i {
                    $($n => $x = Some(Decode::decode($d)?),)*
                    _    => $d.limited_skip()?
                }
            }
            None => {
                let mut i = 0;
                while $d.datatype()? != crate::data::Type::Break {
                    match i {
                        $($n => $x = Some(Decode::decode($d)?),)*
                        _    => $d.limited_skip()?
                    }
                    i += 1
                }
                $d.limited_skip()?
            }
        }

        $(let $x = if let Some(x) = $x {
            x
        } else {
            return Err(Error::missing_value($n).at(p).with_message($msg))
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
impl<'b> Decode<'b> for std::time::SystemTime {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        std::time::UNIX_EPOCH
            .checked_add(d.decode()?)
            .ok_or_else(|| Error::message("duration value can not represent system time").at(p))
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::cell::Cell<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(core::cell::Cell::new)
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::cell::RefCell<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(core::cell::RefCell::new)
    }
}

#[cfg(feature = "std")]
impl<'a, 'b: 'a> Decode<'b> for &'a std::path::Path {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.str().map(std::path::Path::new)
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for Box<std::path::Path> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(std::path::PathBuf::into_boxed_path)
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::path::PathBuf {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        d.decode().map(std::path::Path::to_path_buf)
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::IpAddr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        if Some(2) != d.array()? {
            return Err(Error::message("expected enum (2-element array)").at(p))
        }
        let p = d.position();
        match d.u32()? {
            0 => Ok(std::net::Ipv4Addr::decode(d)?.into()),
            1 => Ok(std::net::Ipv6Addr::decode(d)?.into()),
            n => Err(Error::unknown_variant(n).at(p))
        }
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::Ipv4Addr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let octets: crate::bytes::ByteArray<4> = Decode::decode(d)?;
        Ok(<[u8; 4]>::from(octets).into())
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::Ipv6Addr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let octets: crate::bytes::ByteArray<16> = Decode::decode(d)?;
        Ok(<[u8; 16]>::from(octets).into())
    }
}

#[cfg(feature = "std")]
impl<'b> Decode<'b> for std::net::SocketAddr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        if Some(2) != d.array()? {
            return Err(Error::message("expected enum (2-element array)").at(p))
        }
        let p = d.position();
        match d.u32()? {
            0 => Ok(std::net::SocketAddrV4::decode(d)?.into()),
            1 => Ok(std::net::SocketAddrV6::decode(d)?.into()),
            n => Err(Error::unknown_variant(n).at(p))
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

impl<'b, T: Decode<'b>> Decode<'b> for core::ops::Range<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 start => T ; "Range::start"
            1 end   => T ; "Range::end"
        }
        Ok(core::ops::Range { start, end })
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::ops::RangeFrom<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 start => T ; "RangeFrom::start"
        }
        Ok(core::ops::RangeFrom { start })
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::ops::RangeTo<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 end => T ; "RangeTo::end"
        }
        Ok(core::ops::RangeTo { end })
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::ops::RangeToInclusive<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 end => T ; "RangeToInclusive::end"
        }
        Ok(core::ops::RangeToInclusive { end })
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::ops::RangeInclusive<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        decode_fields! { d |
            0 start => T ; "RangeInclusive::start"
            1 end   => T ; "RangeInclusive::end"
        }
        Ok(core::ops::RangeInclusive::new(start, end))
    }
}

impl<'b, T: Decode<'b>> Decode<'b> for core::ops::Bound<T> {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> {
        let p = d.position();
        if Some(2) != d.array()? {
            return Err(Error::message("expected enum (2-element array)").at(p))
        }
        let p = d.position();
        match d.u32()? {
            0 => d.decode().map(core::ops::Bound::Included),
            1 => d.decode().map(core::ops::Bound::Excluded),
            2 => d.limited_skip().map(|_| core::ops::Bound::Unbounded),
            n => Err(Error::unknown_variant(n).at(p))
        }
    }
}
