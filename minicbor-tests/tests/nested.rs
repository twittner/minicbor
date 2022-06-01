#![cfg(feature = "std")]

use minicbor::data::Type;
use minicbor::decode::{self, Decode, Decoder, Token, Tokenizer};
use minicbor::encode::{self, Write, Encode, Encoder};
use quickcheck::{Arbitrary, Gen};
use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq, Eq)]
enum C {
    E(u32),
    A(Vec<C>),
    M(BTreeMap<u32, C>)
}

impl Arbitrary for C {
    fn arbitrary(_: &mut Gen) -> Self {
        let mut g = Gen::new(3);
        match g.choose(&[1, 2, 3]).unwrap() {
            1 => C::E(Arbitrary::arbitrary(&mut g)),
            2 => C::A(Arbitrary::arbitrary(&mut g)),
            _ => C::M(Arbitrary::arbitrary(&mut g))
        }
    }
}

impl<Ctx> Encode<Ctx> for C {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut Ctx) -> Result<(), encode::Error<W::Error>> {
        match self {
            C::E(n)  => e.u32(*n)?.ok(),
            C::A(xs) => e.encode_with(xs, ctx)?.ok(),
            C::M(xs) => e.encode_with(xs, ctx)?.ok()
        }
    }
}

impl<'b, Ctx> Decode<'b, Ctx> for C {
    fn decode(d: &mut Decoder<'b>, ctx: &mut Ctx) -> Result<Self, decode::Error> {
        match d.datatype()? {
            Type::Array => d.decode_with(ctx).map(C::A),
            Type::Map   => d.decode_with(ctx).map(C::M),
            _           => d.decode_with(ctx).map(C::E)
        }
    }
}

quickcheck::quickcheck! {
    fn can_skip_array_element(c: C) -> bool {
        let c = C::A(vec![c, C::E(42)]);
        let b = minicbor::to_vec(&c).unwrap();
        let mut d = Decoder::new(&b);
        assert_eq!(Some(2), d.array().unwrap());
        d.skip().unwrap();
        C::E(42) == d.decode().unwrap()
    }

    fn can_skip_map_entry(c: C) -> bool {
        let c = C::M(BTreeMap::from_iter(vec![(0, c), (1, C::E(42))]));
        let b = minicbor::to_vec(&c).unwrap();
        let mut d = Decoder::new(&b);
        assert_eq!(Some(2), d.map().unwrap());
        d.skip().unwrap();
        d.skip().unwrap();
        d.skip().unwrap();
        C::E(42) == d.decode().unwrap()
    }
}

struct Rec<'b>(Tokenizer<'b>);

impl fmt::Display for Rec<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn gen(it: &mut Tokenizer<'_>, f: &mut fmt::Formatter) -> fmt::Result {
            match it.next() {
                Some(Ok(Token::Array(n))) => {
                    f.write_str("[")?;
                    for i in 0 .. n {
                        gen(it, f)?;
                        if i < n - 1 {
                            f.write_str(", ")?
                        }
                    }
                    f.write_str("]")
                }
                Some(Ok(Token::Map(n))) => {
                    f.write_str("{")?;
                    for i in 0 .. n {
                        gen(it, f)?;
                        f.write_str(": ")?;
                        gen(it, f)?;
                        if i < n - 1 {
                            f.write_str(", ")?
                        }
                    }
                    f.write_str("}")
                }
                Some(Ok(t))  => t.fmt(f),
                Some(Err(e)) => write!(f, "decoding error: {}", e),
                None         => Ok(())
            }
        }
        let mut this = self.0.clone();
        gen(&mut this, f)
    }
}

quickcheck::quickcheck! {
    fn display(c: C) -> bool {
        let b = minicbor::to_vec(&c).unwrap();
        let t = Tokenizer::new(&b);
        let r = Rec(t.clone());
        format!("{}", t) == format!("{}", r)
    }
}
