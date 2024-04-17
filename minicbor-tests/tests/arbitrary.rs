#![cfg(all(feature = "derive", feature = "alloc"))]

use arbitrary::{Arbitrary, Unstructured};
use core::fmt::Debug;
use minicbor::{Encode, Decode, CborLen, Decoder};
use minicbor_tests::deriving;

const RUNS: usize = 100;

fn identity<T>()
where
    T: for<'a> Arbitrary<'a>
        + for<'a> Decode<'a, ()>
        + Debug
        + CborLen<()>
        + Encode<()>
        + Eq
        + deriving::ResetSkipped
{
    for _ in 0 .. RUNS {
        let seed  = rand::random::<[u8; 16]>();
        let mut u = Unstructured::new(&seed[..]);
        let mut val1: T = u.arbitrary().unwrap();

        let len = minicbor::len(&val1);
        let vec = minicbor::to_vec(&val1).unwrap();
        assert_eq!(len, vec.len(), "cbor length mismatch; input := {val1:?}");

        let mut dec = Decoder::new(&vec);
        let val2 = match dec.decode() {
            Ok(v)  => v,
            Err(e) => {
                panic!("decoding failed; error := {e}; input := {val1:?}")
            }
        };
        assert_eq!(dec.position(), vec.len());

        val1.reset_skipped();
        assert_eq!(val1, val2, "{val1:?} != {val2:?}, seed := {seed:x?}")
    }
}

#[test]
fn deriving_array_tuples() {
    identity::<deriving::array::tuples::Plain>();
    identity::<deriving::array::tuples::TupleStruct0>();
    identity::<deriving::array::tuples::TupleStruct1>();
    identity::<deriving::array::tuples::TupleStruct1Skipped>();
    identity::<deriving::array::tuples::TupleStruct2>();
    identity::<deriving::array::tuples::TupleStruct2Skipped>();
    identity::<deriving::array::tuples::TupleStruct2Rev>();
    identity::<deriving::array::tuples::TupleStruct2AllSkipped>();
}

#[test]
fn deriving_array_records() {
    identity::<deriving::array::records::Plain>();
    identity::<deriving::array::records::Struct0>();
    identity::<deriving::array::records::Struct1>();
    identity::<deriving::array::records::Struct1Skipped>();
    identity::<deriving::array::records::Struct2>();
    identity::<deriving::array::records::Struct2Skipped>();
    identity::<deriving::array::records::Struct2Rev>();
    identity::<deriving::array::records::Struct2AllSkipped>();
}

#[test]
fn deriving_array_enums() {
    identity::<deriving::array::enums::Enum1>();
    identity::<deriving::array::enums::Enum2>();
    identity::<deriving::array::enums::Enum2Rev>();
    identity::<deriving::array::enums::Enum4>();
    identity::<deriving::array::enums::Enum4Rec>();
    identity::<deriving::array::enums::Enum4RecRev>();
    identity::<deriving::array::enums::Enum8>();
    identity::<deriving::array::enums::Enum8Rec>();

}

#[test]
fn deriving_map_tuples() {
    identity::<deriving::map::tuples::Plain>();
    identity::<deriving::map::tuples::TupleStruct0>();
    identity::<deriving::map::tuples::TupleStruct1>();
    identity::<deriving::map::tuples::TupleStruct2>();
    identity::<deriving::map::tuples::TupleStruct2Rev>();
}

#[test]
fn deriving_map_records() {
    identity::<deriving::map::records::Plain>();
    identity::<deriving::map::records::Struct0>();
    identity::<deriving::map::records::Struct1>();
    identity::<deriving::map::records::Struct2>();
    identity::<deriving::map::records::Struct2Rev>();
}

#[test]
fn deriving_map_enums() {
    identity::<deriving::map::enums::Enum1>();
    identity::<deriving::map::enums::Enum2>();
    identity::<deriving::map::enums::Enum2Rev>();
    identity::<deriving::map::enums::Enum4>();
    identity::<deriving::map::enums::Enum4Rec>();
    identity::<deriving::map::enums::Enum4RecRev>();
    identity::<deriving::map::enums::Enum8>();
    identity::<deriving::map::enums::Enum8Rec>();
}

