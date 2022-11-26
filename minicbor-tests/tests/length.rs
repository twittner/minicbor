#![cfg(feature = "std")]

use minicbor::{CborLen, Encode, Decode};
use quickcheck::{Arbitrary, Gen, quickcheck};

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(array)]
enum SampleArrayEncoding<T> {
    #[n(0)] Unit,
    #[n(1)] Struct {
        #[n(0)] field1: String,
        #[n(1)] field2: bool
    },
    #[n(2)] TupleStruct(#[n(0)] u32, #[n(1)] String),
    #[n(3)] Generic(#[n(0)] T)
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(map)]
enum SampleMapEncoding<T> {
    #[n(0)] Unit,
    #[n(1)] Struct {
        #[n(0)] field1: String,
        #[n(1)] field2: bool
    },
    #[n(2)] TupleStruct(#[n(0)] u32, #[n(1)] String),
    #[n(3)] Generic(#[n(0)] T)
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(array)]
struct BytesArrayEncoding {
    #[cbor(n(0), with="minicbor::bytes")] array: [u8; 32],
    #[cbor(n(1), with="minicbor::bytes")] vector: Vec<u8>
}


#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(map)]
struct BytesMapEncoding {
    #[cbor(n(0), with="minicbor::bytes")] array: [u8; 32],
    #[cbor(n(1), with="minicbor::bytes")] vector: Vec<u8>
}

impl Arbitrary for SampleArrayEncoding<BytesArrayEncoding> {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleArrayEncoding::Unit,
            1 => SampleArrayEncoding::Struct {
                field1: Arbitrary::arbitrary(g),
                field2: Arbitrary::arbitrary(g)
            },
            2 => SampleArrayEncoding::TupleStruct(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            _ => SampleArrayEncoding::Generic(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for SampleArrayEncoding<BytesMapEncoding> {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleArrayEncoding::Unit,
            1 => SampleArrayEncoding::Struct {
                field1: Arbitrary::arbitrary(g),
                field2: Arbitrary::arbitrary(g)
            },
            2 => SampleArrayEncoding::TupleStruct(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            _ => SampleArrayEncoding::Generic(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for SampleMapEncoding<BytesArrayEncoding> {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleMapEncoding::Unit,
            1 => SampleMapEncoding::Struct {
                field1: Arbitrary::arbitrary(g),
                field2: Arbitrary::arbitrary(g)
            },
            2 => SampleMapEncoding::TupleStruct(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            _ => SampleMapEncoding::Generic(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for SampleMapEncoding<BytesMapEncoding> {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleMapEncoding::Unit,
            1 => SampleMapEncoding::Struct {
                field1: Arbitrary::arbitrary(g),
                field2: Arbitrary::arbitrary(g)
            },
            2 => SampleMapEncoding::TupleStruct(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            _ => SampleMapEncoding::Generic(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for BytesArrayEncoding {
    fn arbitrary(g: &mut Gen) -> Self {
        BytesArrayEncoding {
            array: [1; 32],
            vector: Arbitrary::arbitrary(g)
        }
    }
}

impl Arbitrary for BytesMapEncoding {
    fn arbitrary(g: &mut Gen) -> Self {
        BytesMapEncoding {
            array: [1; 32],
            vector: Arbitrary::arbitrary(g)
        }
    }
}

quickcheck! {
    fn sample_array_array(val: SampleArrayEncoding<BytesArrayEncoding>) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }

    fn sample_array_map(val: SampleArrayEncoding<BytesMapEncoding>) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }

    fn sample_map_map(val: SampleMapEncoding<BytesMapEncoding>) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }

    fn sample_map_array(val: SampleMapEncoding<BytesArrayEncoding>) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }
}
