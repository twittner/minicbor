#![cfg(feature = "std")]

//! Test forward and backward compatibility.

use minicbor::{Encode, Decode};
use quickcheck::{Arbitrary, Gen, quickcheck};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
struct Version1<'a> {
    #[n(0)] field_a: u32,
    #[n(1)] field_b: Option<Cow<'a, str>>
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
struct Version2<'a> {
    #[n(0)] field_a: u32,
    #[n(1)] field_b: Option<Cow<'a, str>>,
    #[n(2)] field_c: Option<bool>
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
struct Version3 {
    #[n(0)] field_a: u32,
    #[n(2)] field_c: Option<bool>
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
struct Version4 {
    #[n(0)] field_a: u32,
    #[n(2)] field_c: Option<bool>,
    #[n(3)] field_d: Option<Enum1>
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
enum Enum1 {
    #[n(0)] Con1
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
struct Version5 {
    #[n(0)] field_a: u32,
    #[n(2)] field_c: Option<bool>,
    #[n(3)] field_d: Option<Enum2>
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
enum Enum2 {
    #[n(0)] Con1(#[n(0)] Option<i64>)
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
struct Version6<'a> {
    #[n(0)] field_a: u32,
    #[n(2)] field_c: Option<bool>,
    #[n(3)] field_d: Option<Enum3<'a>>
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
enum Enum3<'a> {
    #[n(0)] Con1(#[n(0)] Option<i64>),
    #[n(1)] Con2 {
        #[n(0)] foo: u32,
        #[n(1)] bar: Option<Cow<'a, [u8]>>
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SomeVersion<'a> {
    V1(Version1<'a>),
    V2(Version2<'a>),
    V3(Version3),
    V4(Version4),
    V5(Version5),
    V6(Version6<'a>)
}

#[derive(Clone)]
struct Message(SomeVersion<'static>, Vec<u8>);

#[test]
fn compatibility() {
    // Any given version can be decoded as any of the 6 versions.
    // Common fields are equal and enum constructors match.
    fn property(m: Message) {
        match minicbor::decode::<Version1>(&m.1) {
            Ok(v1) => {
                let v = SomeVersion::V1(v1);
                assert!(m.0.overlaps(&v), "{:?} != {:?}", m.0, v)
            }
            Err(e) => panic!("failed decoding v{} as v1: {}", m.0.version(), e)
        }
        match minicbor::decode::<Version2>(&m.1) {
            Ok(v2) => {
                let v = SomeVersion::V2(v2);
                assert!(m.0.overlaps(&v), "{:?} != {:?}", m.0, v)
            }
            Err(e) => panic!("failed decoding v{} as v2: {}", m.0.version(), e)
        }
        match minicbor::decode::<Version3>(&m.1) {
            Ok(v3) => {
                let v = SomeVersion::V3(v3);
                assert!(m.0.overlaps(&v), "{:?} != {:?}", m.0, v)
            }
            Err(e) => panic!("failed decoding v{} as v3: {}", m.0.version(), e)
        }
        match minicbor::decode::<Version4>(&m.1) {
            Ok(v4) => {
                let v = SomeVersion::V4(v4);
                assert!(m.0.overlaps(&v), "{:?} != {:?}", m.0, v)
            }
            Err(e) => panic!("failed decoding v{} as v4: {}", m.0.version(), e)
        }
        match minicbor::decode::<Version5>(&m.1) {
            Ok(v5) => {
                let v = SomeVersion::V5(v5);
                assert!(m.0.overlaps(&v), "{:?} != {:?}", m.0, v)
            }
            Err(e) => panic!("failed decoding v{} as v5: {}", m.0.version(), e)
        }
        match minicbor::decode::<Version6>(&m.1) {
            Ok(v6) => {
                let v = SomeVersion::V6(v6);
                assert!(m.0.overlaps(&v), "{:?} != {:?}", m.0, v)
            }
            Err(e) => panic!("failed decoding v{} as v6: {}", m.0.version(), e)
        }
    }
    quickcheck(property as fn(Message))
}

impl<'a> SomeVersion<'a> {
    fn version(&self) -> u8 {
        match self {
            SomeVersion::V1(_) => 1,
            SomeVersion::V2(_) => 2,
            SomeVersion::V3(_) => 3,
            SomeVersion::V4(_) => 4,
            SomeVersion::V5(_) => 5,
            SomeVersion::V6(_) => 6
        }
    }

    fn encode(&self) -> Vec<u8> {
        match self {
            SomeVersion::V1(v) => minicbor::to_vec(v).unwrap(),
            SomeVersion::V2(v) => minicbor::to_vec(v).unwrap(),
            SomeVersion::V3(v) => minicbor::to_vec(v).unwrap(),
            SomeVersion::V4(v) => minicbor::to_vec(v).unwrap(),
            SomeVersion::V5(v) => minicbor::to_vec(v).unwrap(),
            SomeVersion::V6(v) => minicbor::to_vec(v).unwrap()
        }
    }

    // Test if common fields in different versions are equal and enum constructors match.
    fn overlaps(&self, other: &SomeVersion) -> bool {
        use SomeVersion::*;
        match (self, other) {
            (V1(a), V1(b)) => a == b,
            (V1(a), V2(b)) => a.field_a == b.field_a && a.field_b == b.field_b,
            (V1(a), V3(b)) => a.field_a == b.field_a,
            (V1(a), V4(b)) => a.field_a == b.field_a,
            (V1(a), V5(b)) => a.field_a == b.field_a,
            (V1(a), V6(b)) => a.field_a == b.field_a,
            (V2(_), V1(_)) => other.overlaps(self),
            (V2(a), V2(b)) => a == b,
            (V2(a), V3(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V2(a), V4(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V2(a), V5(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V2(a), V6(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V3(_), V1(_)) => other.overlaps(self),
            (V3(_), V2(_)) => other.overlaps(self),
            (V3(a), V3(b)) => a == b,
            (V3(a), V4(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V3(a), V5(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V3(a), V6(b)) => a.field_a == b.field_a && a.field_c == b.field_c,
            (V4(_), V1(_)) => other.overlaps(self),
            (V4(_), V2(_)) => other.overlaps(self),
            (V4(_), V3(_)) => other.overlaps(self),
            (V4(a), V4(b)) => a == b,
            (V4(a), V5(b)) => a.field_a == b.field_a && a.field_c == b.field_c &&
                (match (&a.field_d, &b.field_d) {
                    (Some(Enum1::Con1), Some(Enum2::Con1(_))) => true,
                    (None,              None)                 => true,
                    _                                         => false
                }),
            (V4(a), V6(b)) => a.field_a == b.field_a && a.field_c == b.field_c &&
                (match (&a.field_d, &b.field_d) {
                    (Some(Enum1::Con1), Some(Enum3::Con1(_)))  => true,
                    (None,              Some(Enum3::Con2{..})) => true,
                    (None,              None)                  => true,
                    _                                          => false
                }),
            (V5(_), V1(_)) => other.overlaps(self),
            (V5(_), V2(_)) => other.overlaps(self),
            (V5(_), V3(_)) => other.overlaps(self),
            (V5(_), V4(_)) => other.overlaps(self),
            (V5(a), V5(b)) => a == b,
            (V5(a), V6(b)) => a.field_a == b.field_a && a.field_c == b.field_c &&
                (match (&a.field_d, &b.field_d) {
                    (Some(Enum2::Con1(ia)), Some(Enum3::Con1(ib))) => ia == ib,
                    _                                              => true
                }),
            (V6(_), V1(_)) => other.overlaps(self),
            (V6(_), V2(_)) => other.overlaps(self),
            (V6(_), V3(_)) => other.overlaps(self),
            (V6(_), V4(_)) => other.overlaps(self),
            (V6(_), V5(_)) => other.overlaps(self),
            (V6(a), V6(b)) => a == b
        }
    }
}

impl Arbitrary for SomeVersion<'static> {
    fn arbitrary(g: &mut Gen) -> Self {
        match rand::random::<u8>() % 6 {
            0 => SomeVersion::V1(Arbitrary::arbitrary(g)),
            1 => SomeVersion::V2(Arbitrary::arbitrary(g)),
            2 => SomeVersion::V3(Arbitrary::arbitrary(g)),
            3 => SomeVersion::V4(Arbitrary::arbitrary(g)),
            4 => SomeVersion::V5(Arbitrary::arbitrary(g)),
            5 => SomeVersion::V6(Arbitrary::arbitrary(g)),
            _ => unreachable!()
        }
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Message")
            .field(&self.0)
            .field(&hex::encode(&self.1))
            .finish()
    }
}

impl Arbitrary for Message {
    fn arbitrary(g: &mut Gen) -> Self {
        let v = SomeVersion::arbitrary(g);
        let b = v.encode();
        Message(v, b)
    }
}

impl Arbitrary for Version1<'static> {
    fn arbitrary(g: &mut Gen) -> Self {
        Version1 {
            field_a: rand::random(),
            field_b: if rand::random() {
                Some(Cow::Owned(Arbitrary::arbitrary(g)))
            } else {
                None
            }
        }
    }
}

impl Arbitrary for Version2<'static> {
    fn arbitrary(g: &mut Gen) -> Self {
        Version2 {
            field_a: rand::random(),
            field_b: if rand::random() {
                Some(Cow::Owned(Arbitrary::arbitrary(g)))
            } else {
                None
            },
            field_c: Arbitrary::arbitrary(g)
        }
    }
}

impl Arbitrary for Version3 {
    fn arbitrary(g: &mut Gen) -> Self {
        Version3 {
            field_a: rand::random(),
            field_c: Arbitrary::arbitrary(g)
        }
    }
}

impl Arbitrary for Version4 {
    fn arbitrary(g: &mut Gen) -> Self {
        Version4 {
            field_a: rand::random(),
            field_c: Arbitrary::arbitrary(g),
            field_d: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Version5 {
    fn arbitrary(g: &mut Gen) -> Self {
        Version5 {
            field_a: rand::random(),
            field_c: Arbitrary::arbitrary(g),
            field_d: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Version6<'static> {
    fn arbitrary(g: &mut Gen) -> Self {
        Version6 {
            field_a: rand::random(),
            field_c: Arbitrary::arbitrary(g),
            field_d: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Enum1 {
    fn arbitrary(_: &mut Gen) -> Self {
        Enum1::Con1
    }
}

impl Arbitrary for Enum2 {
    fn arbitrary(g: &mut Gen) -> Self {
        Enum2::Con1(Arbitrary::arbitrary(g))
    }
}

impl Arbitrary for Enum3<'static> {
    fn arbitrary(g: &mut Gen) -> Self {
        if rand::random() {
            Enum3::Con1(Arbitrary::arbitrary(g))
        } else {
            Enum3::Con2 {
                foo: Arbitrary::arbitrary(g),
                bar: if rand::random() {
                    Some(Cow::Owned(Arbitrary::arbitrary(g)))
                } else {
                    None
                }
            }
        }
    }
}
