mod encoder;
mod error;
mod iter;
pub mod write;

pub use encoder::Encoder;
pub use error::Error;
pub use iter::{Iter, ExactSizeIter};
pub use write::Write;

pub trait Encode {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>>;
}

impl<T: Encode + ?Sized> Encode for &T {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        (**self).encode(e)
    }
}

impl<T: Encode + ?Sized> Encode for &mut T {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        (**self).encode(e)
    }
}

impl Encode for [u8] {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.bytes(self).map(|_| ())
    }
}

impl Encode for str {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.str(self).map(|_| ())
    }
}

#[cfg(feature = "std")]
impl Encode for String {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.str(self).map(|_| ())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        if let Some(x) = self {
            x.encode(e)?;
        } else {
            e.null()?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T> Encode for std::borrow::Cow<'_, T>
where
    T: Encode + Clone
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        self.as_ref().encode(e)
    }
}

#[cfg(feature = "std")]
impl<T: Encode> Encode for Vec<T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.array(self.len())?;
        for x in self {
            x.encode(e)?
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<K, V> Encode for std::collections::HashMap<K, V>
where
    K: Encode + Eq + std::hash::Hash,
    V: Encode
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.map(self.len())?;
        for (k, v) in self {
            k.encode(e)?;
            v.encode(e)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<K, V> Encode for std::collections::BTreeMap<K, V>
where
    K: Encode + Eq + Ord,
    V: Encode
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.map(self.len())?;
        for (k, v) in self {
            k.encode(e)?;
            v.encode(e)?;
        }
        Ok(())
    }
}

#[cfg(target_pointer_width = "32")]
impl Encode for usize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.u32(*self as u32)?;
        Ok(())
    }
}

#[cfg(target_pointer_width = "64")]
impl Encode for usize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.u64(*self as u64)?;
        Ok(())
    }
}

#[cfg(target_pointer_width = "32")]
impl Encode for isize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.i32(*self as i32)?;
        Ok(())
    }
}

#[cfg(target_pointer_width = "64")]
impl Encode for isize {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        e.i64(*self as i64)?;
        Ok(())
    }
}

macro_rules! encode_impls {
    ($($t:ident)*) => {
        $(
            impl $crate::encode::Encode for $t {
                fn encode<W>(&self, e: &mut $crate::encode::Encoder<W>) -> Result<(), Error<W::Error>>
                where
                    W: $crate::encode::Write
                {
                    e.$t(*self)?;
                    Ok(())
                }
            }
        )*
    }
}

encode_impls!(u8 i8 u16 i16 u32 i32 u64 i64 bool f32 f64 char);

macro_rules! encode_arrays {
    ($($n:expr)*) => {
        $(
            impl<T> $crate::encode::Encode for [T; $n]
            where
                T: $crate::encode::Encode
            {
                fn encode<W>(&self, e: &mut $crate::encode::Encoder<W>) -> Result<(), Error<W::Error>>
                where
                    W: $crate::encode::Write
                {
                    e.array($n)?;
                    for x in self {
                        x.encode(e)?
                    }
                    Ok(())
                }
            }
        )*
    }
}

encode_arrays!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

#[cfg(feature = "smallvec")]
macro_rules! encode_smallvecs {
    ($($n:expr)*) => {
        $(
            impl<T> $crate::encode::Encode for smallvec::SmallVec::<[T; $n]>
            where
                T: $crate::encode::Encode
            {
                fn encode<W>(&self, e: &mut $crate::encode::Encoder<W>) -> Result<(), Error<W::Error>>
                where
                    W: $crate::encode::Write
                {
                    e.array(self.len())?;
                    for x in self {
                        x.encode(e)?
                    }
                    Ok(())
                }
            }
        )*
    }
}

#[cfg(feature = "smallvec")]
encode_smallvecs!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

