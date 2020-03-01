//! A small [CBOR] codec suitable for no_std environments.
//!
//! [CBOR]: https://tools.ietf.org/html/rfc7049

#![forbid(unsafe_code, unused_imports, unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod data;
pub mod decode;
pub mod encode;

const UNSIGNED: u8 = 0x00;
const SIGNED: u8   = 0x20;
const BYTES: u8    = 0x40;
const TEXT: u8     = 0x60;
const ARRAY: u8    = 0x80;
const MAP: u8      = 0xa0;
const TAGGED: u8   = 0xc0;
const SIMPLE: u8   = 0xe0;
const BREAK: u8    = 0xff;

pub use decode::{Decode, Decoder};
pub use encode::{Encode, Encoder};

#[cfg(feature = "derive")]
pub use minicbor_derive::*;

/// An error indicating the end of a slice.
#[derive(Debug)]
pub struct EndOfSlice(());

impl core::fmt::Display for EndOfSlice {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("end of slice")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EndOfSlice {}

pub fn from_slice<'b, T>(b: &'b [u8]) -> Result<T, decode::Error<EndOfSlice>>
where
    T: Decode<'b>
{
    let mut d = Decoder::new(b);
    T::decode(&mut d)
}

#[cfg(not(feature = "std"))]
pub fn to_slice<'b, T>(x: T, b: &'b mut [u8]) -> Result<&'b [u8], encode::Error<EndOfSlice>>
where
    T: Encode
{
    let mut e = Encoder::new(b);
    x.encode(&mut e)?;
    Ok(e.into_inner())
}

#[cfg(feature = "std")]
pub fn to_slice<'b, T>(x: T, b: &'b mut [u8]) -> Result<&'b [u8], encode::Error<std::io::Error>>
where
    T: Encode
{
    let mut e = Encoder::new(b);
    x.encode(&mut e)?;
    Ok(e.into_inner())
}

#[cfg(feature = "std")]
pub fn to_vec<T>(x: T) -> Result<Vec<u8>, encode::Error<std::io::Error>>
where
    T: Encode
{
    let mut e = Encoder::new(Vec::new());
    x.encode(&mut e)?;
    Ok(e.into_inner())
}

