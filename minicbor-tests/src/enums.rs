use minicbor::{Encode, Encoder, Decode, Decoder, decode, encode::{self, Write}};

mod unit {
    use super::*;

    pub(super) fn encode<C, W: Write>(_x: &Unit, _e: &mut Encoder<W>, _c: &mut C) -> Result<(), encode::Error<W::Error>> {
        unimplemented!()
    }

    pub(super) fn decode<C>(_d: &mut Decoder<'_>, _c: &mut C) -> Result<Unit, decode::Error> {
        unimplemented!()
    }
}

#[derive(Encode, Decode)]
struct Unit (#[n(0)] ());

#[derive(Encode, Decode)]
enum E0 {
    #[n(0)] A { #[n(0)] field: Unit },
    #[n(1)] B,
    #[n(2)] C(#[n(0)] Unit)
}

#[derive(Encode, Decode)]
enum E1 {
    #[n(0)] A { #[n(0)] #[cbor(with = "unit")] field: Unit },
    #[n(1)] B,
    #[n(2)] C(#[n(0)] #[cbor(with = "unit")] Unit)
}

#[derive(Encode, Decode)]
enum E2 {
    #[n(0)] A { #[n(0)] #[cbor(encode_with = "unit::encode")] field: Unit },
    #[n(1)] B,
    #[n(2)] C(#[n(0)] #[cbor(encode_with = "unit::encode")] Unit)
}

#[derive(Encode, Decode)]
enum E3 {
    #[n(0)] A { #[n(0)] #[cbor(decode_with = "unit::decode")] field: Unit },
    #[n(1)] B,
    #[n(2)] C(#[n(0)] #[cbor(decode_with = "unit::decode")] Unit)
}

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
enum Gen<T> { #[n(0)] A(#[n(0)] T) }

#[derive(Encode, Decode)]
enum GE0<T> {
    #[n(0)] A { #[n(0)] field: Gen<T> },
    #[n(1)] B,
    #[n(2)] C(#[n(0)] Gen<T>)
}

#[derive(Encode, Decode)]
enum GE1<T, U, Z> {
    #[n(0)] A { #[n(0)] #[cbor(with = "generic")] field: Gen<T> },
    #[n(1)] B,
    #[n(2)] C ( #[n(0)] #[cbor(with = "generic")] Gen<T> ),
    #[n(3)] D ( #[n(0)] U ),
    #[n(4)] E { #[n(0)] field: Z }
}

#[derive(Encode, Decode)]
enum GE2<T, U, Z> {
    #[n(0)] A { #[n(0)] #[cbor(encode_with = "generic::encode")] field: Gen<T> },
    #[n(1)] B,
    #[n(2)] C ( #[n(0)] #[cbor(encode_with = "generic::encode")] Gen<T> ),
    #[n(3)] D ( #[n(0)] U ),
    #[n(4)] E { #[n(0)] field: Z }
}

#[derive(Encode, Decode)]
enum GE3<T, U, Z> {
    #[n(0)] A { #[n(0)] #[cbor(decode_with = "generic::decode")] field: Gen<T> },
    #[n(1)] B,
    #[n(2)] C ( #[n(0)] #[cbor(decode_with = "generic::decode")] Gen<T> ),
    #[n(3)] D ( #[n(0)] U ),
    #[n(4)] E { #[n(0)] field: Z }
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
enum Borrowed<'a> { #[n(0)] A(#[b(0)] &'a str) }

#[derive(Encode, Decode)]
enum BE0<'a, T, U> {
    #[n(0)] A { #[b(0)] field: Borrowed<'a> },
    #[n(1)] B,
    #[n(2)] C ( #[b(0)] Borrowed<'a> ),
    #[n(3)] D ( #[n(0)] T ),
    #[n(4)] E { #[n(0)] field: U }
}

#[derive(Encode, Decode)]
enum BE1<'a, T, U> {
    #[n(0)] A {
        #[b(0)] #[cbor(with = "generic")] field1: Borrowed<'a>,
        #[b(1)] #[cbor(with = "borrow")] field2: &'a str
    },
    #[n(1)] B,
    #[n(2)] C (
        #[b(0)] #[cbor(with = "generic")] Borrowed<'a>,
        #[b(1)] #[cbor(with = "borrow")] &'a str
    ),
    #[n(3)] D ( #[n(0)] T ),
    #[n(4)] E { #[n(0)] field: U }
}

#[derive(Encode, Decode)]
enum BE2<'a, T, U> {
    #[n(0)] A {
        #[b(0)] #[cbor(encode_with = "generic::encode")] field1: Borrowed<'a>,
        #[b(1)] #[cbor(encode_with = "borrow::encode")] field2: &'a str
    },
    #[n(1)] B,
    #[n(2)] C (
        #[b(0)] #[cbor(encode_with = "generic::encode")] Borrowed<'a>,
        #[b(1)] #[cbor(encode_with = "borrow::encode")] &'a str
    ),
    #[n(3)] D ( #[n(0)] T ),
    #[n(4)] E { #[n(0)] field: U }
}

#[derive(Encode, Decode)]
enum BE3<'a, T, U> {
    #[n(0)] A {
        #[b(0)] #[cbor(decode_with = "generic::decode")] field1: Borrowed<'a>,
        #[b(1)] #[cbor(decode_with = "borrow::decode")] field2: &'a str
    },
    #[n(1)] B,
    #[n(2)] C (
        #[b(0)] #[cbor(decode_with = "generic::decode")] Borrowed<'a>,
        #[b(1)] #[cbor(decode_with = "borrow::decode")] &'a str
    ),
    #[n(3)] D ( #[n(0)] T ),
    #[n(4)] E { #[n(0)] field: U }
}

