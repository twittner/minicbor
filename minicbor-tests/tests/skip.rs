use minicbor::{Decode, Decoder, Encode, Encoder};
use minicbor::data::{Tag, Type};
use minicbor::decode;
use minicbor::encode::{self, Write};
use quickcheck::{Arbitrary, Gen};
use std::collections::BTreeMap;

/// A simplified CBOR data model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Cbor {
    /// A primitive element.
    Int(u8),
    /// A CBOR array.
    Array(Vec<Cbor>),
    /// A CBOR array with indefinite length.
    ArrayIndef(Vec<Cbor>),
    /// A CBOR map.
    Map(BTreeMap<Cbor, Cbor>),
    /// A CBOR map with indefinite length.
    MapIndef(BTreeMap<Cbor, Cbor>),
    /// A tagged CBOR value.
    Tagged(Tag, Box<Cbor>),
    /// A CBOR string.
    String(String),
    /// A CBOR byte string.
    Bytes(Vec<u8>),
    /// An indefinite length CBOR string.
    StringIndef(Vec<String>),
    /// An indefinite length CBOR byte string.
    BytesIndef(Vec<Vec<u8>>)
}

quickcheck::quickcheck! {
    // Basic check to test encode-decode identity.
    fn identity(item: Cbor) -> bool {
        let bytes = minicbor::to_vec(&item).unwrap();
        let cbor: Cbor = minicbor::decode(&bytes).unwrap();
        cbor == item
    }

    // Encode prefix and suffix and when decoding, skip over the prefix and
    // check that the remainder matches the suffix.
    fn skip_prefix(prefix: Vec<Cbor>, suffix: Vec<Cbor>) -> bool {
        let mut bytes = Vec::new();
        let mut e = Encoder::new(&mut bytes);
        for c in &prefix {
            e.encode(c).unwrap();
        }
        let p = e.writer().len();
        for c in &suffix {
            e.encode(c).unwrap();
        }
        let mut d = Decoder::new(&bytes);
        for _ in 0 .. prefix.len() {
            d.skip().unwrap()
        }
        assert_eq!(p, d.position());
        let mut v = Vec::new();
        for _ in 0 .. suffix.len() {
            v.push(d.decode().unwrap())
        }
        suffix == v
    }
}

// Trait impls ///////////////////////////////////////////////////////////////////////////////////

impl<C> Encode<C> for Cbor {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), encode::Error<W::Error>> {
        match self {
            Cbor::Int(i) => e.u8(*i)?.ok(),
            Cbor::Array(a) => {
                e.array(a.len() as u64)?;
                for x in a {
                    x.encode(e, ctx)?;
                }
                Ok(())
            }
            Cbor::ArrayIndef(a) => {
                e.begin_array()?;
                for x in a {
                    x.encode(e, ctx)?;
                }
                e.end()?.ok()
            }
            Cbor::Map(m) => {
                e.map(m.len() as u64)?;
                for (k, v) in m {
                    k.encode(e, ctx)?;
                    v.encode(e, ctx)?;
                }
                Ok(())
            }
            Cbor::MapIndef(m) => {
                e.begin_map()?;
                for (k, v) in m {
                    k.encode(e, ctx)?;
                    v.encode(e, ctx)?;
                }
                e.end()?.ok()
            }
            Cbor::Tagged(t, v) => {
                e.tag(*t)?;
                e.encode_with(v, ctx)?.ok()
            }
            Cbor::String(s) => e.str(&s)?.ok(),
            Cbor::Bytes(b)  => e.bytes(&b)?.ok(),
            Cbor::StringIndef(v) => {
                e.begin_str()?;
                for s in v {
                    e.str(s)?;
                }
                e.end()?.ok()
            }
            Cbor::BytesIndef(v) => {
                e.begin_bytes()?;
                for b in v {
                    e.bytes(b)?;
                }
                e.end()?.ok()
            }
        }
    }
}

impl<'b, C> Decode<'b, C> for Cbor {
    fn decode(d: &mut Decoder<'b>, ctx: &mut C) -> Result<Self, decode::Error> {
        match d.datatype()? {
            Type::U8 => d.u8().map(Cbor::Int),
            Type::Array => {
                if let Some(n) = d.array()? {
                    let mut v = Vec::new();
                    for _ in 0 .. n {
                        v.push(Self::decode(d, ctx)?)
                    }
                    Ok(Cbor::Array(v))
                } else {
                    Err(decode::Error::type_mismatch(Type::Array).with_message("missing length"))
                }
            }
            Type::ArrayIndef => {
                if let None = d.array()? {
                    let mut v = Vec::new();
                    while Type::Break != d.datatype()? {
                        v.push(Self::decode(d, ctx)?)
                    }
                    d.skip()?;
                    Ok(Cbor::ArrayIndef(v))
                } else {
                    Err(decode::Error::type_mismatch(Type::ArrayIndef).with_message("unexpected length"))
                }
            }
            Type::Map => {
                if let Some(n) = d.map()? {
                    let mut m = BTreeMap::new();
                    for _ in 0 .. n {
                        let k = Self::decode(d, ctx)?;
                        let v = Self::decode(d, ctx)?;
                        m.insert(k, v);
                    }
                    Ok(Cbor::Map(m))
                } else {
                    Err(decode::Error::type_mismatch(Type::Array).with_message("missing length"))
                }
            }
            Type::MapIndef => {
                if let None = d.map()? {
                    let mut m = BTreeMap::new();
                    while Type::Break != d.datatype()? {
                        let k = Self::decode(d, ctx)?;
                        let v = Self::decode(d, ctx)?;
                        m.insert(k, v);
                    }
                    d.skip()?;
                    Ok(Cbor::MapIndef(m))
                } else {
                    Err(decode::Error::type_mismatch(Type::Array).with_message("unexpected length"))
                }
            }
            Type::Tag => {
                let t = d.tag()?;
                let v = Self::decode(d, ctx)?;
                Ok(Cbor::Tagged(t, Box::new(v)))
            }
            Type::String => {
                let s = d.str()?;
                Ok(Cbor::String(s.into()))
            }
            Type::Bytes => {
                let b = d.bytes()?;
                Ok(Cbor::Bytes(b.into()))
            }
            Type::StringIndef => {
                let mut v = Vec::new();
                for s in d.str_iter()? {
                    v.push(s?.into())
                }
                Ok(Cbor::StringIndef(v))
            }
            Type::BytesIndef => {
                let mut v = Vec::new();
                for b in d.bytes_iter()? {
                    v.push(b?.into())
                }
                Ok(Cbor::BytesIndef(v))
            }
            other => Err(decode::Error::type_mismatch(other).with_message("unknown type").at(d.position()))
        }
    }
}

impl Arbitrary for Cbor {
    fn arbitrary(g: &mut Gen) -> Self {
        if cfg!(feature = "__test-partial-skip-support") {
            gen_cbor(g, false, 3)
        } else {
            gen_cbor(g, true, 3)
        }
    }
}

fn gen_cbor(g: &mut Gen, indef: bool, rem: usize) -> Cbor {
    match g.choose(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) {
        Some(1) => Cbor::Int(Arbitrary::arbitrary(g)),
        Some(2) => {
            let n = rand::random::<usize>() % 5;
            let mut v = Vec::with_capacity(n);
            if rem > 0 {
                for _ in 0 .. n {
                    v.push(gen_cbor(g, indef, rem - 1))
                }
            }
            Cbor::Array(v)
        }
        Some(3) => {
            let n = rand::random::<usize>() % 5;
            let mut m = BTreeMap::new();
            if rem > 0 {
                for _ in 0 .. n {
                    let k = gen_cbor(g, indef, rem - 1);
                    let v = gen_cbor(g, indef, rem - 1);
                    m.insert(k, v);
                }
            }
            Cbor::Map(m)
        }
        Some(4) => {
            Cbor::String(Arbitrary::arbitrary(g))
        }
        Some(5) => {
            Cbor::Bytes(Arbitrary::arbitrary(g))
        }
        Some(6) => {
            Cbor::StringIndef(Arbitrary::arbitrary(g))
        }
        Some(7) => {
            Cbor::BytesIndef(Arbitrary::arbitrary(g))
        }
        Some(8) if indef => {
            let n = rand::random::<usize>() % 5;
            let mut v = Vec::with_capacity(n);
            if rem > 0 {
                for _ in 0 .. n {
                    v.push(gen_cbor(g, indef, rem - 1))
                }
            }
            Cbor::ArrayIndef(v)
        }
        Some(9) if indef => {
            let n = rand::random::<usize>() % 5;
            let mut m = BTreeMap::new();
            if rem > 0 {
                for _ in 0 .. n {
                    let k = gen_cbor(g, indef, rem - 1);
                    let v = gen_cbor(g, indef, rem - 1);
                    m.insert(k, v);
                }
            }
            Cbor::MapIndef(m)
        }
        _ => {
            Cbor::Tagged(Tag::Base64, Box::new(Cbor::String(Arbitrary::arbitrary(g))))
        }
    }
}

