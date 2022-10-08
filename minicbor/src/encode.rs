//! Traits and types for encoding CBOR.
//!
//! This module defines the trait [`Encode`] and the actual [`Encoder`].
//! It also defines a [`Write`] trait to store the encoded bytes.

mod encoder;
mod error;
pub mod write;

pub use encoder::Encoder;
pub use error::Error;
pub use write::Write;

/// A type that can be encoded to CBOR.
///
/// If this type's CBOR encoding is meant to be decoded by `Decode` impls
/// derived with [`minicbor_derive`] *it is advisable to only produce a
/// single CBOR data item*. Tagging, maps or arrays can and should be used
/// for multiple values.
pub trait Encode<C> {
    /// Encode a value of this type using the given `Encoder`.
    ///
    /// In addition to the encoder a user provided encoding context is given
    /// as another parameter. Most implementations of this trait do not need an
    /// encoding context and should be completely generic in the context
    /// type. In cases where a context is needed and the `Encode` impl type is
    /// meant to be combined with other types that require a different context
    /// type, it is preferrable to constrain the context type variable `C` with
    /// a trait bound instead of fixing the type.
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>>;

    /// Is this value of `Self` a nil value?
    ///
    /// This method is primarily used by `minicbor-derive`.
    ///
    /// Some types have a special value to denote the concept of "nothing", aka
    /// nil. An example is the `Option` type with its `None` value. This
    /// method--if overriden--allows checking if a value is such a special nil
    /// value.
    ///
    /// NB: A type implementing `Encode` with an overriden `Encode::is_nil`
    /// method should also override `Decode::nil` if it implements `Decode`
    /// at all.
    fn is_nil(&self) -> bool {
        false
    }
}

/// A type that can calculate its own CBOR encoding length.
pub trait CborLen {
    /// Compute the CBOR encoding length in bytes of this value.
    fn cbor_len(&self) -> usize;
}

impl<C, T: Encode<C> + ?Sized> Encode<C> for &T {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        (**self).encode(e, ctx)
    }
}

impl<T: CborLen> CborLen for &T {
    fn cbor_len(&self) -> usize {
        (**self).cbor_len()
    }
}

impl<C, T: Encode<C> + ?Sized> Encode<C> for &mut T {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        (**self).encode(e, ctx)
    }
}

impl<T: CborLen> CborLen for &mut T {
    fn cbor_len(&self) -> usize {
        (**self).cbor_len()
    }
}

#[cfg(feature = "alloc")]
impl<C, T: Encode<C> + ?Sized> Encode<C> for alloc::boxed::Box<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        (**self).encode(e, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<T: CborLen> CborLen for alloc::boxed::Box<T> {
    fn cbor_len(&self) -> usize {
        (**self).cbor_len()
    }
}

impl<C> Encode<C> for str {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.str(self)?.ok()
    }
}

impl CborLen for str {
    fn cbor_len(&self) -> usize {
        let n = self.len();
        n.cbor_len() + n
    }
}

impl<C, T: Encode<C>> Encode<C> for Option<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        if let Some(x) = self {
            x.encode(e, ctx)?;
        } else {
            e.null()?;
        }
        Ok(())
    }

    fn is_nil(&self) -> bool {
        self.is_none()
    }
}

impl<T: CborLen> CborLen for Option<T> {
    fn cbor_len(&self) -> usize {
        if let Some(x) = self {
            x.cbor_len()
        } else {
            1
        }
    }
}

impl<C, T: Encode<C>, E: Encode<C>> Encode<C> for Result<T, E> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?;
        match self {
            Ok(v)  => e.u32(0)?.encode_with(v, ctx)?.ok(),
            Err(v) => e.u32(1)?.encode_with(v, ctx)?.ok()
        }
    }
}

impl<T: CborLen, E: CborLen> CborLen for Result<T, E> {
    fn cbor_len(&self) -> usize {
        1 + match self {
            Ok(x)  => 1 + x.cbor_len(),
            Err(e) => 1 + e.cbor_len()
        }
    }
}

#[cfg(feature = "alloc")]
impl<C> Encode<C> for alloc::string::String {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.str(self)?.ok()
    }
}

#[cfg(feature = "alloc")]
impl CborLen for alloc::string::String {
    fn cbor_len(&self) -> usize {
        let n = self.len();
        n.cbor_len() + n
    }
}

#[cfg(feature = "alloc")]
impl<C, T> Encode<C> for alloc::borrow::Cow<'_, T>
where
    T: Encode<C> + alloc::borrow::ToOwned + ?Sized
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        self.as_ref().encode(e, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<T> CborLen for alloc::borrow::Cow<'_, T>
where
    T: CborLen + alloc::borrow::ToOwned + ?Sized
{
    fn cbor_len(&self) -> usize {
        self.as_ref().cbor_len()
    }
}

#[cfg(feature = "std")]
impl<C, T, S> Encode<C> for std::collections::HashSet<T, S>
where
    T: Encode<C>,
    S: std::hash::BuildHasher
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(self.len() as u64)?;
        for x in self {
            x.encode(e, ctx)?
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T, S> CborLen for std::collections::HashSet<T, S>
where
    T: CborLen,
    S: std::hash::BuildHasher
{
    fn cbor_len(&self) -> usize {
        self.len().cbor_len() + self.iter().map(|x| x.cbor_len()).sum::<usize>()
    }
}

#[cfg(feature = "std")]
impl<C, K, V, S> Encode<C> for std::collections::HashMap<K, V, S>
where
    K: Encode<C> + Eq + std::hash::Hash,
    V: Encode<C>,
    S: std::hash::BuildHasher
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.map(self.len() as u64)?;
        for (k, v) in self {
            k.encode(e, ctx)?;
            v.encode(e, ctx)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<K, V, S> CborLen for std::collections::HashMap<K, V, S>
where
    K: CborLen,
    V: CborLen,
    S: std::hash::BuildHasher
{
    fn cbor_len(&self) -> usize {
        self.len().cbor_len() + self.iter()
            .map(|(k, v)| k.cbor_len() + v.cbor_len())
            .sum::<usize>()
    }
}

#[cfg(feature = "alloc")]
impl<C, K, V> Encode<C> for alloc::collections::BTreeMap<K, V>
where
    K: Encode<C> + Eq + Ord,
    V: Encode<C>
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.map(self.len() as u64)?;
        for (k, v) in self {
            k.encode(e, ctx)?;
            v.encode(e, ctx)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<K, V> CborLen for std::collections::BTreeMap<K, V>
where
    K: CborLen,
    V: CborLen,
{
    fn cbor_len(&self) -> usize {
        self.len().cbor_len() + self.iter()
            .map(|(k, v)| k.cbor_len() + v.cbor_len())
            .sum::<usize>()
    }
}

impl<C, T> Encode<C> for core::marker::PhantomData<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.array(0)?.ok()
    }
}

impl<T> CborLen for core::marker::PhantomData<T> {
    fn cbor_len(&self) -> usize {
        1
    }
}

impl<C> Encode<C> for () {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.array(0)?.ok()
    }
}

impl CborLen for () {
    fn cbor_len(&self) -> usize {
        1
    }
}

impl<C, T: Encode<C>> Encode<C> for core::num::Wrapping<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl<T: CborLen> CborLen for core::num::Wrapping<T> {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

#[cfg(target_pointer_width = "32")]
impl<C> Encode<C> for usize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.u32(*self as u32)?.ok()
    }
}

#[cfg(target_pointer_width = "32")]
impl CborLen for usize {
    fn cbor_len(&self) -> usize {
        (*self as u32).cbor_len()
    }
}

#[cfg(target_pointer_width = "64")]
impl<C> Encode<C> for usize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.u64(*self as u64)?.ok()
    }
}

#[cfg(target_pointer_width = "64")]
impl CborLen for usize {
    fn cbor_len(&self) -> usize {
        (*self as u64).cbor_len()
    }
}

#[cfg(target_pointer_width = "32")]
impl<C> Encode<C> for isize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.i32(*self as i32)?.ok()
    }
}

#[cfg(target_pointer_width = "32")]
impl CborLen for isize {
    fn cbor_len(&self) -> usize {
        (*self as i32).cbor_len()
    }
}

#[cfg(target_pointer_width = "64")]
impl<C> Encode<C> for isize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.i64(*self as i64)?.ok()
    }
}

#[cfg(target_pointer_width = "64")]
impl CborLen for isize {
    fn cbor_len(&self) -> usize {
        (*self as i64).cbor_len()
    }
}

impl<C> Encode<C> for crate::data::Int {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.int(*self)?.ok()
    }
}

impl CborLen for crate::data::Int {
    fn cbor_len(&self) -> usize {
        self.value().cbor_len()
    }
}

macro_rules! encode_basic {
    ($($t:ident)*) => {
        $(
            impl<C> Encode<C> for $t {
                fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
                    e.$t(*self)?;
                    Ok(())
                }
            }
        )*
    }
}

encode_basic!(u8 i8 u16 i16 u32 i32 u64 i64 bool f32 f64 char);

impl CborLen for bool {
    fn cbor_len(&self) -> usize {
        1
    }
}

impl CborLen for char {
    fn cbor_len(&self) -> usize {
        (*self as u32).cbor_len()
    }
}

impl CborLen for u8 {
    fn cbor_len(&self) -> usize {
        if let 0 ..= 0x17 = self { 1 } else { 2 }
    }
}

impl CborLen for u16 {
    fn cbor_len(&self) -> usize {
        match self {
            0    ..= 0x17 => 1,
            0x18 ..= 0xff => 2,
            _             => 3
        }
    }
}

impl CborLen for u32 {
    fn cbor_len(&self) -> usize {
        match self {
            0     ..= 0x17   => 1,
            0x18  ..= 0xff   => 2,
            0x100 ..= 0xffff => 3,
            _                => 5
        }
    }
}

impl CborLen for u64 {
    fn cbor_len(&self) -> usize {
        match self {
            0        ..= 0x17        => 1,
            0x18     ..= 0xff        => 2,
            0x100    ..= 0xffff      => 3,
            0x1_0000 ..= 0xffff_ffff => 5,
            _                        => 9
        }
    }
}

impl CborLen for i8 {
    fn cbor_len(&self) -> usize {
        let x = if *self >= 0 { *self as u8 } else { (-1 - self) as u8 };
        x.cbor_len()
    }
}

impl CborLen for i16 {
    fn cbor_len(&self) -> usize {
        let x = if *self >= 0 { *self as u16 } else { (-1 - self) as u16 };
        x.cbor_len()
    }
}

impl CborLen for i32 {
    fn cbor_len(&self) -> usize {
        let x = if *self >= 0 { *self as u32 } else { (-1 - self) as u32 };
        x.cbor_len()
    }
}

impl CborLen for i64 {
    fn cbor_len(&self) -> usize {
        let x = if *self >= 0 { *self as u64 } else { (-1 - self) as u64 };
        x.cbor_len()
    }
}

impl CborLen for f32 {
    fn cbor_len(&self) -> usize {
        5
    }
}

impl CborLen for f64 {
    fn cbor_len(&self) -> usize {
        9
    }
}

macro_rules! encode_nonzero {
    ($($t:ty)*) => {
        $(
            impl<C> Encode<C> for $t {
                fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
                    self.get().encode(e, ctx)
                }
            }

            impl CborLen for $t {
                fn cbor_len(&self) -> usize {
                    self.get().cbor_len()
                }
            }
        )*
    }
}

encode_nonzero! {
    core::num::NonZeroU8
    core::num::NonZeroU16
    core::num::NonZeroU32
    core::num::NonZeroU64
    core::num::NonZeroI8
    core::num::NonZeroI16
    core::num::NonZeroI32
    core::num::NonZeroI64
}

#[cfg(any(atomic32, atomic64))]
macro_rules! encode_atomic {
    ($($t:ty)*) => {
        $(
            impl<C> Encode<C> for $t {
                fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
                    self.load(core::sync::atomic::Ordering::SeqCst).encode(e, ctx)?;
                    Ok(())
                }
            }

            impl CborLen for $t {
                fn cbor_len(&self) -> usize {
                    self.load(core::sync::atomic::Ordering::SeqCst).cbor_len()
                }
            }
        )*
    }
}

#[cfg(atomic32)]
encode_atomic! {
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
encode_atomic! {
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

macro_rules! encode_sequential {
    ($($t:ty)*) => {
        $(
            impl<C, T: Encode<C>> Encode<C> for $t {
                fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
                    e.array(self.len() as u64)?;
                    for x in self {
                        x.encode(e, ctx)?
                    }
                    Ok(())
                }
            }

            impl<T: CborLen> CborLen for $t {
                fn cbor_len(&self) -> usize {
                    let n = self.len();
                    n.cbor_len() + self.iter().map(|x| x.cbor_len()).sum::<usize>()
                }
            }
        )*
    }
}

encode_sequential!([T]);

#[cfg(feature = "alloc")]
encode_sequential! {
    alloc::vec::Vec<T>
    alloc::collections::VecDeque<T>
    alloc::collections::LinkedList<T>
    alloc::collections::BinaryHeap<T>
    alloc::collections::BTreeSet<T>
}

macro_rules! encode_arrays {
    ($($n:expr)*) => {
        $(
            impl<C, T: Encode<C>> Encode<C> for [T; $n] {
                fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
                    e.array($n)?;
                    for x in self {
                        x.encode(e, ctx)?
                    }
                    Ok(())
                }
            }

            impl<T: CborLen> CborLen for [T; $n] {
                fn cbor_len(&self) -> usize {
                    let n = self.len();
                    n.cbor_len() + self.iter().map(|x| x.cbor_len()).sum::<usize>()
                }
            }
        )*
    }
}

encode_arrays!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

macro_rules! encode_tuples {
    ($( $len:expr => { $($T:ident ($idx:tt))+ } )+) => {
        $(
            impl<Ctx, $($T: Encode<Ctx>),+> Encode<Ctx> for ($($T,)+) {
                fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut Ctx) -> Result<(), Error<W::Error>> {
                    e.array($len)?
                        $(.encode_with(&self.$idx, ctx)?)+
                        .ok()
                }
            }

            impl<$($T: CborLen),+> CborLen for ($($T,)+) {
                fn cbor_len(&self) -> usize {
                    $len.cbor_len() $(+ self.$idx.cbor_len())+
                }
            }
        )+
    }
}

encode_tuples! {
    1  => { A(0) }
    2  => { A(0) B(1) }
    3  => { A(0) B(1) C(2) }
    4  => { A(0) B(1) C(2) D(3) }
    5  => { A(0) B(1) C(2) D(3) E(4) }
    6  => { A(0) B(1) C(2) D(3) E(4) F(5) }
    7  => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) }
    8  => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) }
    9  => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) }
    10 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) }
    11 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) }
    12 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) }
    13 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) }
    14 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) }
    15 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) }
    16 => { A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) }
}

impl<C> Encode<C> for core::time::Duration {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode_with(self.as_secs(), ctx)?
            .encode_with(self.subsec_nanos(), ctx)?
            .ok()
    }
}

impl CborLen for core::time::Duration {
    fn cbor_len(&self) -> usize {
        1 + self.as_secs().cbor_len()
          + self.subsec_nanos().cbor_len()
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::time::SystemTime {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        match self.duration_since(std::time::UNIX_EPOCH) {
            Ok(d)  => d.encode(e, ctx),
            Err(e) => Err(Error::custom(e).with_message("when encoding system time"))
        }
    }
}

#[cfg(feature = "std")]
impl CborLen for std::time::SystemTime{
    fn cbor_len(&self) -> usize {
        self.duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.cbor_len())
            .unwrap_or(0)
    }
}

impl<C, T: Encode<C> + Copy> Encode<C> for core::cell::Cell<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        self.get().encode(e, ctx)
    }
}

impl<T: CborLen + Copy> CborLen for core::cell::Cell<T> {
    fn cbor_len(&self) -> usize {
        self.get().cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::cell::RefCell<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        if let Ok(v) = self.try_borrow() {
            v.encode(e, ctx)
        } else {
            Err(Error::message("could not borrow ref cell value"))
        }
    }
}

impl<T: CborLen> CborLen for core::cell::RefCell<T> {
    fn cbor_len(&self) -> usize {
        self.borrow().cbor_len()
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::path::Path {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        if let Some(s) = self.to_str() {
            e.str(s)?.ok()
        } else {
            Err(Error::message("non-utf-8 path values are not supported"))
        }
    }
}

#[cfg(feature = "std")]
impl CborLen for std::path::Path {
    fn cbor_len(&self) -> usize {
        self.to_str().map(|s| s.cbor_len()).unwrap_or(0)
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::path::PathBuf {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        self.as_path().encode(e, ctx)
    }
}

#[cfg(feature = "std")]
impl CborLen for std::path::PathBuf {
    fn cbor_len(&self) -> usize {
        self.as_path().cbor_len()
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::net::IpAddr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?;
        match self {
            std::net::IpAddr::V4(a) => e.u32(0)?.encode_with(a, ctx)?.ok(),
            std::net::IpAddr::V6(a) => e.u32(1)?.encode_with(a, ctx)?.ok()
        }
    }
}

#[cfg(feature = "std")]
impl CborLen for std::net::IpAddr {
    fn cbor_len(&self) -> usize {
        1 + match self {
            std::net::IpAddr::V4(a) => 1 + a.cbor_len(),
            std::net::IpAddr::V6(a) => 1 + a.cbor_len(),
        }
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::net::Ipv4Addr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.bytes(&self.octets())?.ok()
    }
}

#[cfg(feature = "std")]
impl CborLen for std::net::Ipv4Addr {
    fn cbor_len(&self) -> usize {
        5
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::net::Ipv6Addr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), Error<W::Error>> {
        e.bytes(&self.octets())?.ok()
    }
}

#[cfg(feature = "std")]
impl CborLen for std::net::Ipv6Addr {
    fn cbor_len(&self) -> usize {
        17
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::net::SocketAddr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?;
        match self {
            std::net::SocketAddr::V4(a) => e.u32(0)?.encode_with(a, ctx)?.ok(),
            std::net::SocketAddr::V6(a) => e.u32(1)?.encode_with(a, ctx)?.ok()
        }
    }
}

#[cfg(feature = "std")]
impl CborLen for std::net::SocketAddr {
    fn cbor_len(&self) -> usize {
        1 + match self {
            std::net::SocketAddr::V4(a) => 1 + a.cbor_len(),
            std::net::SocketAddr::V6(a) => 1 + a.cbor_len(),
        }
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::net::SocketAddrV4 {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode_with(self.ip(), ctx)?
            .encode_with(self.port(), ctx)?
            .ok()
    }
}

#[cfg(feature = "std")]
impl CborLen for std::net::SocketAddrV4 {
    fn cbor_len(&self) -> usize {
        1 + self.ip().cbor_len() + self.port().cbor_len()
    }
}

#[cfg(feature = "std")]
impl<C> Encode<C> for std::net::SocketAddrV6 {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode_with(self.ip(), ctx)?
            .encode_with(self.port(), ctx)?
            .ok()
    }
}

#[cfg(feature = "std")]
impl CborLen for std::net::SocketAddrV6 {
    fn cbor_len(&self) -> usize {
        1 + self.ip().cbor_len() + self.port().cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::ops::Range<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode_with(&self.start, ctx)?
            .encode_with(&self.end, ctx)?
            .ok()
    }
}

impl<T: CborLen> CborLen for core::ops::Range<T> {
    fn cbor_len(&self) -> usize {
        1 + self.start.cbor_len() + self.end.cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::ops::RangeFrom<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(1)?
            .encode_with(&self.start, ctx)?
            .ok()
    }
}

impl<T: CborLen> CborLen for core::ops::RangeFrom<T> {
    fn cbor_len(&self) -> usize {
        1 + self.start.cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::ops::RangeTo<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(1)?
            .encode_with(&self.end, ctx)?
            .ok()
    }
}

impl<T: CborLen> CborLen for core::ops::RangeTo<T> {
    fn cbor_len(&self) -> usize {
        1 + self.end.cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::ops::RangeToInclusive<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(1)?
            .encode_with(&self.end, ctx)?
            .ok()
    }
}

impl<T: CborLen> CborLen for core::ops::RangeToInclusive<T> {
    fn cbor_len(&self) -> usize {
        1 + self.end.cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::ops::RangeInclusive<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?
            .encode_with(self.start(), ctx)?
            .encode_with(self.end(), ctx)?
            .ok()
    }
}

impl<T: CborLen> CborLen for core::ops::RangeInclusive<T> {
    fn cbor_len(&self) -> usize {
        1 + self.start().cbor_len() + self.end().cbor_len()
    }
}

impl<C, T: Encode<C>> Encode<C> for core::ops::Bound<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.array(2)?;
        match self {
            core::ops::Bound::Included(v) => e.u32(0)?.encode_with(v, ctx)?.ok(),
            core::ops::Bound::Excluded(v) => e.u32(1)?.encode_with(v, ctx)?.ok(),
            core::ops::Bound::Unbounded   => e.u32(2)?.array(0)?.ok()
        }
    }
}

impl<T: CborLen> CborLen for core::ops::Bound<T> {
    fn cbor_len(&self) -> usize {
        1 + match self {
            core::ops::Bound::Included(v) => 1 + v.cbor_len(),
            core::ops::Bound::Excluded(v) => 1 + v.cbor_len(),
            core::ops::Bound::Unbounded   => 2
        }
    }
}

/// An encodable iterator writing its items as a CBOR array.
///
/// This type wraps any type implementing [`Iterator`] + [`Clone`] and encodes
/// the items produced by the iterator as a CBOR array.
#[derive(Debug)]
pub struct ArrayIter<I>(I);

impl<I> ArrayIter<I> {
    pub fn new(it: I) -> Self {
        ArrayIter(it)
    }
}

impl<C, I, T> Encode<C> for ArrayIter<I>
where
    I: Iterator<Item = T> + Clone,
    T: Encode<C>
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        let iter = self.0.clone();
        let (low, up) = iter.size_hint();
        let exact = Some(low) == up;
        if exact {
            e.array(low as u64)?;
        } else {
            e.begin_array()?;
        }
        for item in iter {
            item.encode(e, ctx)?;
        }
        if !exact {
            e.end()?;
        }
        Ok(())
    }
}

/// An encodable iterator writing its items as a CBOR map.
///
/// This type wraps any type implementing [`Iterator`] + [`Clone`] and encodes
/// the items produced by the iterator as a CBOR map.
#[derive(Debug)]
pub struct MapIter<I>(I);

impl<I> MapIter<I> {
    pub fn new(it: I) -> Self {
        MapIter(it)
    }
}

impl<C, I, K, V> Encode<C> for MapIter<I>
where
    I: Iterator<Item = (K, V)> + Clone,
    K: Encode<C>,
    V: Encode<C>
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> {
        let iter = self.0.clone();
        let (low, up) = iter.size_hint();
        let exact = Some(low) == up;
        if exact {
            e.map(low as u64)?;
        } else {
            e.begin_map()?;
        }
        for (k, v) in iter {
            k.encode(e, ctx)?;
            v.encode(e, ctx)?;
        }
        if !exact {
            e.end()?;
        }
        Ok(())
    }
}

