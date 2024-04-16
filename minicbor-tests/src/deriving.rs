#![cfg(feature = "derive")]

use arbitrary::Arbitrary;
use minicbor::{Encode, Decode, CborLen};

/// Types that can reset their skipped fields to default values.
///
/// Used in tests to ensure fields marked as `skip` contain their default value
/// to compared them with decoded values.
pub trait ResetSkipped {
    fn reset_skipped(&mut self) {}
}

macro_rules! gen_modules {
    ($name : ident, $array_or_map : tt) => {
        pub mod $name {
            use super::*;

            pub mod tuples {
                use super::*;

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Plain;

                impl ResetSkipped for Plain {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct0(#[n(0)] pub bool);

                impl ResetSkipped for TupleStruct0 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct1(#[n(1)] pub bool);

                impl ResetSkipped for TupleStruct1 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct1Skipped(#[cbor(skip)] pub char, #[n(0)] bool);

                impl ResetSkipped for TupleStruct1Skipped {
                    fn reset_skipped(&mut self) {
                        self.0 = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct2(#[n(0)] pub bool, #[n(1)] pub char);

                impl ResetSkipped for TupleStruct2 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct2Rev(#[n(1)] pub char, #[n(0)] pub bool);

                impl ResetSkipped for TupleStruct2Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct2Skipped(#[n(0)] pub bool, #[cbor(skip)] pub char);

                impl ResetSkipped for TupleStruct2Skipped {
                    fn reset_skipped(&mut self) {
                        self.1 = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct TupleStruct2AllSkipped(#[cbor(skip)] pub bool, #[cbor(skip)] pub char);

                impl ResetSkipped for TupleStruct2AllSkipped {
                    fn reset_skipped(&mut self) {
                        self.0 = Default::default();
                        self.1 = Default::default()
                    }
                }
            }

            pub mod records {
                use super::*;

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Plain {}

                impl ResetSkipped for Plain {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct0 {
                    #[n(0)] pub a: bool
                }

                impl ResetSkipped for Struct0 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct1 {
                    #[n(1)] pub a: bool
                }

                impl ResetSkipped for Struct1 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct1Skipped {
                    #[cbor(skip)] pub a: char,
                    #[n(0)] pub b: bool
                }

                impl ResetSkipped for Struct1Skipped {
                    fn reset_skipped(&mut self) {
                        self.a = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct2 {
                    #[n(0)] pub a: bool,
                    #[n(1)] pub b: char
                }

                impl ResetSkipped for Struct2 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct2Rev {
                    #[n(1)] pub a: char,
                    #[n(0)] pub b: bool
                }

                impl ResetSkipped for Struct2Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct2Skipped {
                    #[n(0)] pub a: bool,
                    #[cbor(skip)] pub b: char
                }

                impl ResetSkipped for Struct2Skipped {
                    fn reset_skipped(&mut self) {
                        self.b = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub struct Struct2AllSkipped {
                    #[cbor(skip)] pub a: bool,
                    #[cbor(skip)] pub b: char
                }

                impl ResetSkipped for Struct2AllSkipped {
                    fn reset_skipped(&mut self) {
                        self.a = Default::default();
                        self.b = Default::default()
                    }
                }
            }

            pub mod enums {
                use super::*;

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum1 {
                    #[n(0)] A
                }

                impl ResetSkipped for Enum1 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum2 {
                    #[n(0)] A,
                    #[n(1)] B
                }

                impl ResetSkipped for Enum2 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum2Rev {
                    #[n(1)] B,
                    #[n(0)] A
                }

                impl ResetSkipped for Enum2Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum4 {
                    #[n(0)] A,
                    #[n(1)] B,
                    #[n(2)] C(#[n(0)] char),
                    #[n(3)] D(#[n(1)] char)
                }

                impl ResetSkipped for Enum4 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum4Rec {
                    #[n(0)] A,
                    #[n(1)] B,
                    #[n(2)] C { #[n(0)] a: char },
                    #[n(3)] D { #[n(1)] b: char }
                }

                impl ResetSkipped for Enum4Rec {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum4Rev {
                    #[n(3)] A,
                    #[n(2)] B,
                    #[n(1)] C(#[n(0)] char),
                    #[n(0)] D(#[n(1)] char)
                }

                impl ResetSkipped for Enum4Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum4RecRev {
                    #[n(3)] A,
                    #[n(2)] B,
                    #[n(1)] C { #[n(0)] a: char },
                    #[n(0)] D { #[n(1)] b: char }
                }

                impl ResetSkipped for Enum4RecRev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum8 {
                    #[n(0)] A,
                    #[n(1)] B,
                    #[n(2)] C(#[n(0)] char),
                    #[n(3)] D(#[n(1)] char),
                    #[n(6)] E(#[n(1)] char, #[n(0)] bool),
                    #[n(8)] F(#[cbor(skip)] char, #[n(0)] bool),
                    #[n(9)] G(#[n(0)] char, #[cbor(skip)] bool),
                    #[n(7)] H(#[n(3)] char, #[cbor(skip)] bool),
                }

                impl ResetSkipped for Enum8 {
                    fn reset_skipped(&mut self) {
                        match self {
                            Self::F(v, _) => *v = Default::default(),
                            Self::G(_, v) => *v = Default::default(),
                            Self::H(_, v) => *v = Default::default(),
                            _             => {}
                        }
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map)]
                pub enum Enum8Rec {
                    #[n(0)] A,
                    #[n(1)] B,
                    #[n(2)] C { #[n(0)] a: char },
                    #[n(3)] D { #[n(1)] a: char },
                    #[n(6)] E { #[n(1)] a: char, #[n(0)] b: bool },
                    #[n(8)] F { #[cbor(skip)] a: char, #[n(0)] b: bool },
                    #[n(9)] G { #[n(0)] a: char, #[cbor(skip)] b: bool },
                    #[n(7)] H { #[n(3)] a: char, #[cbor(skip)] b: bool }
                }

                impl ResetSkipped for Enum8Rec {
                    fn reset_skipped(&mut self) {
                        match self {
                            Self::F {a, ..} => *a = Default::default(),
                            Self::G {b, ..} => *b = Default::default(),
                            Self::H {b, ..} => *b = Default::default(),
                            _             => {}
                        }
                    }
                }
            }
        }
    }
}

gen_modules!(array, array);
gen_modules!(map, map);

