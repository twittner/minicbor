#![cfg(feature = "alloc")]

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Fields {
    a: u8,
    name_of_twenty_three_by: u8,
    name_of_twenty_four_byte: u8,
    #[serde(alias = "old")]
    new: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Variants {
    Unit,
    Newtype(u8),
    Struct { x: u8 },
    VariantNameOfTwentyFourB(u8),
}

#[test]
fn struct_field_names() {
    let f = Fields { a: 1, name_of_twenty_three_by: 2, name_of_twenty_four_byte: 3, new: 4 };
    let v = minicbor_serde::to_vec(&f).unwrap();
    assert_eq!(minicbor_serde::from_slice::<Fields>(&v).unwrap(), f)
}

#[test]
fn enum_variant_names() {
    for x in [Variants::Unit, Variants::Newtype(1), Variants::Struct { x: 2 }, Variants::VariantNameOfTwentyFourB(3)] {
        let v = minicbor_serde::to_vec(&x).unwrap();
        assert_eq!(minicbor_serde::from_slice::<Variants>(&v).unwrap(), x)
    }
}

#[test]
fn field_alias_and_unknown_field() {
    #[derive(Serialize)]
    struct Old {
        a: u8,
        name_of_twenty_three_by: u8,
        name_of_twenty_four_byte: u8,
        old: u8,
        gone: u8,
    }
    let o = Old { a: 1, name_of_twenty_three_by: 2, name_of_twenty_four_byte: 3, old: 4, gone: 5 };
    let v = minicbor_serde::to_vec(&o).unwrap();
    let f = minicbor_serde::from_slice::<Fields>(&v).unwrap();
    assert_eq!(f, Fields { a: 1, name_of_twenty_three_by: 2, name_of_twenty_four_byte: 3, new: 4 })
}

#[test]
fn unknown_variant() {
    let v = minicbor_serde::to_vec("Other").unwrap();
    let e = minicbor_serde::from_slice::<Variants>(&v).unwrap_err();
    assert!(e.to_string().contains("Other"), "{e}")
}

#[test]
fn truncated_field_name() {
    // map(1), text(5) with only 3 bytes following
    let input = b"\xa1\x65abc";
    assert!(minicbor_serde::from_slice::<Fields>(input).is_err())
}
