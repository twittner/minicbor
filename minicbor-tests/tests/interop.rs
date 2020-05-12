use minicbor::{data, decode, Decoder, encode::{self, Write}, Encode, Encoder};
use quickcheck::{Arbitrary, Gen};
use rand::Rng;
use serde_cbor::Value;
use std::collections::BTreeMap;

#[test]
fn encode_minicbor_decode_serde() {
    fn property(input: Cbor) -> bool {
        let bytes = minicbor::to_vec(&input).unwrap();
        let output = serde_cbor::from_slice(&bytes).map(Cbor).unwrap();
        input == output
    }
    quickcheck::quickcheck(property as fn(Cbor) -> bool)
}

#[test]
fn encode_serde_decode_minicbor() {
    fn property(input: Cbor) {
        let bytes = serde_cbor::to_vec(&input.0).unwrap();
        let mut decoder = Decoder::new(&bytes);
        check(&mut decoder, input).unwrap();
    }
    quickcheck::quickcheck(property as fn(Cbor))
}

#[derive(Debug, Clone, PartialEq)]
struct Cbor(Value);

impl Arbitrary for Cbor {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Cbor(gen_value(g, 7))
    }
}

impl Encode for Cbor {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>> {
        encode_value(&self.0, e)
    }
}

// Decode using a known `Cbor` value as template.
fn check<'a>(d: &mut Decoder<'a>, c: Cbor) -> Result<(), decode::Error> {
    match c.0 {
        Value::Null => {
            assert_eq!(data::Type::Null, d.datatype()?);
            d.skip()?
        }
        Value::Bool(b)    => assert_eq!(b, d.bool()?),
        Value::Integer(i) => assert_eq!(i, d.i64()? as i128),
        Value::Float(f)   => assert_eq!(f, d.f64()?),
        Value::Bytes(b)   => assert_eq!(b, d.bytes()?),
        Value::Text(s)    => assert_eq!(s, d.str()?),
        Value::Array(a)   => {
            assert_eq!(Some(a.len() as u64), d.array()?);
            for x in a {
                check(d, Cbor(x))?
            }
        }
        Value::Map(m) => {
            assert_eq!(Some(m.len() as u64), d.map()?);
            for (k, v) in m {
                check(d, Cbor(k))?;
                check(d, Cbor(v))?
            }
        }
        _ => {}
    }
    Ok(())
}

// Generate an arbitrary `serde_cbor::Value`.
// (`rem` denotes the remaining recursion depth.)
fn gen_value<G: Gen>(g: &mut G, rem: usize) -> Value {
    match g.gen_range(0, 9) {
        0 => Value::Null,
        1 => Value::Bool(true),
        2 => Value::Bool(false),
        3 => Value::Integer(g.gen::<i128>() % std::i64::MAX as i128),
        4 => Value::Float(g.gen()),
        5 => Value::Bytes(Arbitrary::arbitrary(g)),
        6 => Value::Text(Arbitrary::arbitrary(g)),
        7 => Value::Array({
            let mut v = Vec::new();
            if rem > 0 {
                for _ in 0 .. g.gen_range(0, 12) {
                    v.push(gen_value(g, rem - 1))
                }
            }
            v
        }),
        _ => Value::Map({
            let mut m = BTreeMap::new();
            if rem > 0 {
                for _ in 0 .. g.gen_range(0, 12) {
                    m.insert(gen_value(g, rem - 1), gen_value(g, rem - 1));
                }
            }
            m
        })
    }
}

// Encode a `serde_cbor::Value` with a `minicbor::Encoder`.
fn encode_value<W>(val: &Value, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>>
where
    W: encode::Write
{
    match val {
        Value::Null       => { e.null()?; }
        Value::Bool(b)    => { e.bool(*b)?; }
        Value::Integer(i) => { e.i64(*i as i64)?; }
        Value::Float(f)   => { e.f64(*f)?; }
        Value::Bytes(b)   => { e.bytes(b)?; }
        Value::Text(s)    => { e.str(s)?; }
        Value::Array(a)   => {
            e.array(a.len() as u64)?;
            for x in a {
                encode_value(x, e)?
            }
        }
        Value::Map(m) => {
            e.map(m.len() as u64)?;
            for (k, v) in m {
                encode_value(k, e)?;
                encode_value(v, e)?
            }
        }
        _ => ()
    }
    Ok(())
}

