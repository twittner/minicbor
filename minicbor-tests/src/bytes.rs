#![cfg(feature = "std")]

use minicbor::bytes::{ByteArray, ByteSlice, ByteVec};
use minicbor::{Decode, Encode};
use std::borrow::Cow;

#[derive(Encode, Decode)]
struct Slice<'a> {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: &'a [u8],
}

#[derive(Encode, Decode)]
struct OptSlice<'a> {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: Option<&'a [u8]>,
}

#[derive(Encode, Decode)]
struct CowSlice<'a> {
    #[n(0)]
    field: Cow<'a, ByteSlice>,
}

#[derive(Encode, Decode)]
struct CowSlice2<'a> {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: Cow<'a, [u8]>,
}

#[derive(Encode, Decode)]
struct CowSlice3<'a> {
    #[b(0)]
    #[cbor(with = "minicbor::bytes")]
    field: Cow<'a, [u8]>,
}

#[derive(Encode, Decode)]
struct Vector {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: Vec<u8>,
}

#[derive(Encode, Decode)]
struct CowVector<'a> {
    #[n(0)]
    field: Cow<'a, ByteVec>,
}

#[derive(Encode, Decode)]
struct Array {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: [u8; 64],
}

#[derive(Encode, Decode)]
struct GenericArray<const N: usize> {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: [u8; N],
}

#[derive(Encode, Decode)]
struct GenericCowArray<'a, const N: usize> {
    #[n(0)]
    field: Cow<'a, ByteArray<N>>,
}
