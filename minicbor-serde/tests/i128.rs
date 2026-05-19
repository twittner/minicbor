#![cfg(feature = "alloc")]

use serde::{Deserialize, Serialize};

fn roundtrip_u128(n: u128) {
    let v = minicbor_serde::to_vec(n).unwrap();
    let m: u128 = minicbor_serde::from_slice(&v).unwrap();
    assert_eq!(n, m, "u128 round-trip failed for {n}");

    // Wire format must match the raw `minicbor` encoding.
    let direct = minicbor::to_vec(n).unwrap();
    assert_eq!(v, direct, "serde encoding diverges from minicbor for u128 {n}");
}

fn roundtrip_i128(n: i128) {
    let v = minicbor_serde::to_vec(n).unwrap();
    let m: i128 = minicbor_serde::from_slice(&v).unwrap();
    assert_eq!(n, m, "i128 round-trip failed for {n}");

    let direct = minicbor::to_vec(n).unwrap();
    assert_eq!(v, direct, "serde encoding diverges from minicbor for i128 {n}");
}

#[test]
fn u128_roundtrip() {
    roundtrip_u128(0);
    roundtrip_u128(1);
    roundtrip_u128(u64::MAX as u128);
    roundtrip_u128(u64::MAX as u128 + 1);
    roundtrip_u128(u128::MAX);
}

#[test]
fn i128_roundtrip() {
    roundtrip_i128(0);
    roundtrip_i128(1);
    roundtrip_i128(-1);
    roundtrip_i128(i64::MIN as i128);
    roundtrip_i128(i64::MAX as i128);
    roundtrip_i128(u64::MAX as i128);
    roundtrip_i128(-(u64::MAX as i128) - 1);
    roundtrip_i128(u64::MAX as i128 + 1);
    roundtrip_i128(-(u64::MAX as i128) - 2);
    roundtrip_i128(i128::MAX);
    roundtrip_i128(i128::MIN);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrap {
    u: u128,
    i: i128,
}

#[test]
fn struct_with_128_fields() {
    let w = Wrap { u: u128::MAX, i: i128::MIN };
    let v = minicbor_serde::to_vec(&w).unwrap();
    let r: Wrap = minicbor_serde::from_slice(&v).unwrap();
    assert_eq!(w, r);
}

#[test]
fn deserialize_any_routes_bignums() {
    // Custom value type that calls `deserialize_any` and supports the full
    // 128-bit range. (Serde's built-in `Content` buffer used by untagged enums
    // only stores up to 64 bits — see serde-rs/serde#2912 — so we can't test
    // bignum dispatch through an untagged enum here.)
    use serde::de;
    use std::fmt;

    #[derive(Debug, PartialEq)]
    enum Any {
        U128(u128),
        I128(i128),
    }

    struct AnyVisitor;
    impl<'de> de::Visitor<'de> for AnyVisitor {
        type Value = Any;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a 128-bit integer")
        }

        fn visit_u128<E: de::Error>(self, v: u128) -> Result<Any, E> { Ok(Any::U128(v)) }
        fn visit_i128<E: de::Error>(self, v: i128) -> Result<Any, E> { Ok(Any::I128(v)) }
        fn visit_u64<E: de::Error>(self, v: u64)   -> Result<Any, E> { Ok(Any::U128(u128::from(v))) }
        fn visit_i64<E: de::Error>(self, v: i64)   -> Result<Any, E> { Ok(Any::I128(i128::from(v))) }
        fn visit_u32<E: de::Error>(self, v: u32)   -> Result<Any, E> { Ok(Any::U128(u128::from(v))) }
        fn visit_u16<E: de::Error>(self, v: u16)   -> Result<Any, E> { Ok(Any::U128(u128::from(v))) }
        fn visit_u8<E: de::Error>(self, v: u8)     -> Result<Any, E> { Ok(Any::U128(u128::from(v))) }
    }

    impl<'de> Deserialize<'de> for Any {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            d.deserialize_any(AnyVisitor)
        }
    }

    // Large positive: encoded as positive bignum, deserialize_any must route to visit_u128.
    let n = u64::MAX as u128 + 12345;
    let v = minicbor::to_vec(n).unwrap();
    assert_eq!(Any::U128(n), minicbor_serde::from_slice::<Any>(&v).unwrap());

    // Large negative: encoded as negative bignum, deserialize_any must route to visit_i128.
    let n: i128 = -(u64::MAX as i128) - 100;
    let v = minicbor::to_vec(n).unwrap();
    assert_eq!(Any::I128(n), minicbor_serde::from_slice::<Any>(&v).unwrap());

    // Small unsigned: encoded as native CBOR u8, must reach visit_u8/u64.
    let v = minicbor::to_vec(7u128).unwrap();
    assert_eq!(Any::U128(7), minicbor_serde::from_slice::<Any>(&v).unwrap());
}
