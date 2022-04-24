use minicbor::{Encode, Encoder, Decode, Decoder, decode, encode::{self, Write}};

mod unit {
    use super::*;

    pub(super) fn encode<C, W: Write>(_x: &Unit, _e: &mut Encoder<W>, _ctx: &mut C) -> Result<(), encode::Error<W::Error>> {
        unimplemented!()
    }

    pub(super) fn decode<C>(_d: &mut Decoder<'_>, _ctx: &mut C) -> Result<Unit, decode::Error> {
        unimplemented!()
    }

    pub(super) fn is_nil(_: &Unit) -> bool {
        true
    }

    pub(super) fn nil() -> Option<Unit> {
        Some(Unit(()))
    }
}

#[derive(Encode, Decode)]
struct Unit (#[n(0)] ());

#[derive(Encode, Decode)]
struct S0 { #[n(0)] field: Unit }

#[derive(Encode, Decode)]
struct S1 { #[cbor(n(0), with = "unit", has_nil)] field: Unit }

#[derive(Encode, Decode)]
struct S2 { #[n(0)] #[cbor(encode_with = "unit::encode")] field: Unit }

#[derive(Encode, Decode)]
struct S3 { #[n(0)] #[cbor(decode_with = "unit::decode")] field: Unit }

#[derive(Encode, Decode)]
struct T0 (#[n(0)] Unit);

#[derive(Encode, Decode)]
struct T1 (#[n(0)] #[cbor(with = "unit")] Unit);

#[derive(Encode, Decode)]
struct T2 (#[n(0)] #[cbor(encode_with = "unit::encode")] Unit);

#[derive(Encode, Decode)]
struct T3 (#[n(0)] #[cbor(decode_with = "unit::decode")] Unit);

mod generic {
    use super::*;

    pub(super) fn encode<C, T, W: Write>(_x: &T, _e: &mut Encoder<W>, _c: &mut C) -> Result<(), encode::Error<W::Error>> {
        unimplemented!()
    }

    pub(super) fn decode<C, T>(_d: &mut Decoder<'_>, _c: &mut C) -> Result<T, decode::Error> {
        unimplemented!()
    }
}

#[derive(Encode, Decode)]
struct Gen<T> (#[n(0)] T);

#[derive(Encode, Decode)]
struct GS0<T, U> {
    #[n(0)] field1: Gen<T>,
    #[n(1)] field2: U
}

#[derive(Encode, Decode)]
struct GS1<T, U> {
    #[n(0)] #[cbor(with = "generic")] field1: Gen<T>,
    #[n(1)] field2: U
}

#[derive(Encode, Decode)]
struct GS2<T, U> {
    #[n(0)] #[cbor(encode_with = "generic::encode")] field1: Gen<T>,
    #[n(1)] field2: U
}

#[derive(Encode, Decode)]
struct GS3<T, U> {
    #[n(0)] #[cbor(decode_with = "generic::decode")] field1: Gen<T>,
    #[n(1)] field2: U
}

mod borrow {
    use super::*;

    pub(super) fn encode<C, W: Write>(x: &str, e: &mut Encoder<W>, _c: &mut C) -> Result<(), encode::Error<W::Error>> {
        e.str(x)?.ok()
    }

    pub(super) fn decode<'a, C>(d: &mut Decoder<'a>, _c: &mut C) -> Result<&'a str, decode::Error> {
        d.str()
    }
}

#[derive(Encode, Decode)]
struct Borrowed<'a> (#[b(0)] &'a str);

#[derive(Encode, Decode)]
struct BS0<'a, T> {
    #[b(0)] field1: Borrowed<'a>,
    #[b(1)] field2: &'a str,
    #[n(2)] field3: T
}

#[derive(Encode, Decode)]
struct BS1<'a, T> {
    #[b(0)] #[cbor(with = "generic")] field1: Borrowed<'a>,
    #[b(1)] #[cbor(with = "borrow")] field2: &'a str,
    #[n(2)] field3: T
}

#[derive(Encode, Decode)]
struct BS2<'a, T> {
    #[b(0)] #[cbor(encode_with = "generic::encode")] field1: Borrowed<'a>,
    #[b(1)] #[cbor(encode_with = "borrow::encode")] field2: &'a str,
    #[n(2)] field3: T
}

#[derive(Encode, Decode)]
struct BS3<'a, T> {
    #[b(0)] #[cbor(decode_with = "generic::decode")] field1: Borrowed<'a>,
    #[b(1)] #[cbor(decode_with = "borrow::decode")] field2: &'a str,
    #[n(2)] field3: T
}

#[derive(Encode, Decode)]
#[cbor(transparent)]
struct W1(#[n(0)] Vec<u8>);

#[derive(Encode, Decode)]
#[cbor(transparent)]
struct W2<T>(#[n(0)] T);

#[derive(Encode, Decode)]
#[cbor(transparent)]
struct W3 { #[n(0)] inner: Vec<u8> }

#[derive(Encode, Decode)]
#[cbor(transparent)]
struct W4<'a, T>(#[b(0)] BS2<'a, T>);

#[derive(Encode, Decode)]
#[cbor(transparent)]
struct W5<'a, T> { #[b(0)] inner: BS2<'a, T> }

