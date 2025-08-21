#![cfg(feature = "derive")]

#[cfg(feature = "alloc")]
extern crate alloc;

use arbitrary::Arbitrary;
use minicbor::{CborLen, Decode, Encode};

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
                #[cbor($array_or_map, tag(1000))]
                pub struct Plain;

                impl ResetSkipped for Plain {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct TupleStruct0(#[cbor(n(0), tag(1001))] pub bool);

                impl ResetSkipped for TupleStruct0 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct TupleStruct1(#[cbor(n(1), tag(1001))] pub bool);

                impl ResetSkipped for TupleStruct1 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct TupleStruct1Skipped(
                    #[cbor(skip)] pub char,
                    #[cbor(n(0), tag(1001))] bool,
                );

                impl ResetSkipped for TupleStruct1Skipped {
                    fn reset_skipped(&mut self) {
                        self.0 = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct TupleStruct2(
                    #[cbor(n(0), tag(1001))] pub bool,
                    #[cbor(n(1), tag(1002))] pub char,
                );

                impl ResetSkipped for TupleStruct2 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct TupleStruct2Rev(
                    #[cbor(n(1), tag(1001))] pub char,
                    #[cbor(n(0), tag(1002))] pub bool,
                );

                impl ResetSkipped for TupleStruct2Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct TupleStruct2Skipped(
                    #[cbor(n(0), tag(1001))] pub bool,
                    #[cbor(skip)] pub char,
                );

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
                #[cbor($array_or_map, tag(1000))]
                pub struct Plain {}

                impl ResetSkipped for Plain {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct0 {
                    #[cbor(n(0), tag(1001))]
                    pub a: bool,
                }

                impl ResetSkipped for Struct0 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct1 {
                    #[cbor(n(1), tag(1001))]
                    pub a: bool,
                }

                impl ResetSkipped for Struct1 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct1Skipped {
                    #[cbor(skip)]
                    pub a: char,
                    #[cbor(n(0), tag(1001))]
                    pub b: bool,
                }

                impl ResetSkipped for Struct1Skipped {
                    fn reset_skipped(&mut self) {
                        self.a = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct2 {
                    #[cbor(n(0), tag(1001))]
                    pub a: bool,
                    #[cbor(n(1), tag(1002))]
                    pub b: char,
                }

                impl ResetSkipped for Struct2 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct2Rev {
                    #[cbor(n(1), tag(1001))]
                    pub a: char,
                    #[cbor(n(0), tag(1002))]
                    pub b: bool,
                }

                impl ResetSkipped for Struct2Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct2Skipped {
                    #[cbor(n(0), tag(1001))]
                    pub a: bool,
                    #[cbor(skip)]
                    pub b: char,
                }

                impl ResetSkipped for Struct2Skipped {
                    fn reset_skipped(&mut self) {
                        self.b = Default::default()
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub struct Struct2AllSkipped {
                    #[cbor(skip)]
                    pub a: bool,
                    #[cbor(skip)]
                    pub b: char,
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
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum1 {
                    #[cbor(n(0), tag(1001))]
                    A,
                }

                impl ResetSkipped for Enum1 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum2 {
                    #[cbor(n(0), tag(1001))]
                    A,
                    #[cbor(n(1), tag(1002))]
                    B,
                }

                impl ResetSkipped for Enum2 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum2Rev {
                    #[cbor(n(1), tag(1002))]
                    B,
                    #[cbor(n(0), tag(1001))]
                    A,
                }

                impl ResetSkipped for Enum2Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum4 {
                    #[cbor(n(0), tag(1001))]
                    A,
                    #[cbor(n(1), tag(1002))]
                    B,
                    #[cbor(n(2), tag(1003))]
                    C(#[cbor(n(0), tag(2000))] char),
                    #[cbor(n(3), tag(1004))]
                    D(#[cbor(n(1), tag(2001))] char),
                }

                impl ResetSkipped for Enum4 {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum4Rec {
                    #[cbor(n(0), tag(1001))]
                    A,
                    #[cbor(n(1), tag(1002))]
                    B,
                    #[cbor(n(2), tag(1003))]
                    C {
                        #[cbor(n(0), tag(2000))]
                        a: char,
                    },
                    #[cbor(n(3), tag(1004))]
                    D {
                        #[cbor(n(1), tag(2001))]
                        b: char,
                    },
                }

                impl ResetSkipped for Enum4Rec {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum4Rev {
                    #[cbor(n(3), tag(1004))]
                    A,
                    #[cbor(n(2), tag(1003))]
                    B,
                    #[cbor(n(1), tag(1002))]
                    C(#[cbor(n(0), tag(2000))] char),
                    #[cbor(n(0), tag(1001))]
                    D(#[cbor(n(1), tag(2001))] char),
                }

                impl ResetSkipped for Enum4Rev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum4RecRev {
                    #[cbor(n(3), tag(1004))]
                    A,
                    #[cbor(n(2), tag(1003))]
                    B,
                    #[cbor(n(1), tag(1002))]
                    C {
                        #[cbor(n(0), tag(2000))]
                        a: char,
                    },
                    #[cbor(n(0), tag(1001))]
                    D {
                        #[cbor(n(1), tag(2001))]
                        b: char,
                    },
                }

                impl ResetSkipped for Enum4RecRev {}

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum8 {
                    #[cbor(n(0), tag(1001))]
                    A,
                    #[cbor(n(1), tag(1002))]
                    B,
                    #[cbor(n(2), tag(1003))]
                    C(#[cbor(n(0), tag(2000))] char),
                    #[cbor(n(3), tag(1004))]
                    D(#[cbor(n(1), tag(2001))] char),
                    #[cbor(n(6), tag(1005))]
                    E(#[cbor(n(1), tag(2002))] char, #[cbor(n(0), tag(3000))] bool),
                    #[cbor(n(8), tag(1006))]
                    F(#[cbor(skip)] char, #[cbor(n(0), tag(3000))] bool),
                    #[cbor(n(9), tag(1007))]
                    G(#[cbor(n(0), tag(2003))] char, #[cbor(skip)] bool),
                    #[cbor(n(7), tag(1008))]
                    H(#[cbor(n(3), tag(2004))] char, #[cbor(skip)] bool),
                }

                impl ResetSkipped for Enum8 {
                    fn reset_skipped(&mut self) {
                        match self {
                            Self::F(v, _) => *v = Default::default(),
                            Self::G(_, v) => *v = Default::default(),
                            Self::H(_, v) => *v = Default::default(),
                            _ => {}
                        }
                    }
                }

                #[derive(Debug, Arbitrary, Encode, Decode, CborLen, PartialEq, Eq)]
                #[cbor($array_or_map, tag(1000))]
                pub enum Enum8Rec {
                    #[cbor(n(0), tag(1001))]
                    A,
                    #[cbor(n(1), tag(1002))]
                    B,
                    #[cbor(n(2), tag(1003))]
                    C {
                        #[cbor(n(0), tag(2000))]
                        a: char,
                    },
                    #[cbor(n(3), tag(1004))]
                    D {
                        #[cbor(n(1), tag(2001))]
                        a: char,
                    },
                    #[cbor(n(6), tag(1005))]
                    E {
                        #[cbor(n(1), tag(2002))]
                        a: char,
                        #[cbor(n(0), tag(3000))]
                        b: bool,
                    },
                    #[cbor(n(8), tag(1006))]
                    F {
                        #[cbor(skip)]
                        a: char,
                        #[cbor(n(0), tag(3000))]
                        b: bool,
                    },
                    #[cbor(n(9), tag(1007))]
                    G {
                        #[cbor(n(0), tag(2003))]
                        a: char,
                        #[cbor(skip)]
                        b: bool,
                    },
                    #[cbor(n(7), tag(1008))]
                    H {
                        #[cbor(n(3), tag(2004))]
                        a: char,
                        #[cbor(skip)]
                        b: bool,
                    },
                }

                impl ResetSkipped for Enum8Rec {
                    fn reset_skipped(&mut self) {
                        match self {
                            Self::F { a, .. } => *a = Default::default(),
                            Self::G { b, .. } => *b = Default::default(),
                            Self::H { b, .. } => *b = Default::default(),
                            _ => {}
                        }
                    }
                }
            }
        }
    };
}

gen_modules!(array, array);
gen_modules!(map, map);
