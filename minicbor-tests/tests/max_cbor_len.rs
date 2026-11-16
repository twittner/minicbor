use minicbor::{Encode, CborLen, MaxCborLen};

// --- Structs ---

#[derive(Encode, CborLen, MaxCborLen)]
#[cbor(array)]
struct SimpleArray {
    #[n(0)] x: u8,
    #[n(1)] y: u32,
    #[n(2)] z: bool,
}

#[derive(Encode, MaxCborLen)]
#[cbor(map)]
struct SimpleMap {
    #[n(0)] x: u8,
    #[n(1)] y: u32,
}

#[derive(Encode, MaxCborLen)]
#[cbor(transparent)]
struct Transparent(#[n(0)] u32);

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct WithOption {
    #[n(0)] a: u8,
    #[n(1)] b: Option<u32>,
}

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct WithGap {
    #[n(0)] a: u8,
    #[n(3)] b: u32,
}

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct Nested {
    #[n(0)] inner: SimpleArray,
    #[n(1)] val: u8,
}

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct WithArray {
    #[n(0)] id: u8,
    #[n(1)] data: [u32; 3],
}

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct TupleStruct(#[n(0)] u8, #[n(1)] u32);

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct Tagged {
    #[n(0)] #[cbor(tag(1))] x: u32,
}

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct WithSkip {
    #[n(0)] a: u8,
    #[cbor(skip)] #[allow(dead_code)] b: u32,
    #[n(1)] c: bool,
}

// --- Enums ---

#[derive(Encode, MaxCborLen)]
#[cbor(index_only)]
#[allow(dead_code)]
enum IndexOnly {
    #[n(0)] A,
    #[n(1)] B,
    #[n(2)] C,
}

#[derive(Encode, MaxCborLen)]
#[allow(dead_code)]
enum WithFields {
    #[n(0)] Unit,
    #[n(1)] One { #[n(0)] x: u8 },
    #[n(2)] Two { #[n(0)] a: u32, #[n(1)] b: bool },
}

#[derive(Encode, MaxCborLen)]
#[allow(dead_code)]
enum WithTupleVariant {
    #[n(0)] A,
    #[n(1)] B(#[n(0)] u8, #[n(1)] u32),
}

#[derive(Encode, MaxCborLen)]
#[cbor(flat)]
#[allow(dead_code)]
enum Flat {
    #[n(0)] A,
    #[n(1)] B(#[n(0)] u8, #[n(1)] u32),
}

#[derive(Encode, MaxCborLen)]
#[cbor(map)]
#[allow(dead_code)]
enum MapEnum {
    #[n(0)] X { #[n(0)] a: u8 },
    #[n(1)] Y { #[n(0)] a: u32, #[n(1)] b: bool },
}

// --- Generic ---

#[derive(Encode, MaxCborLen)]
#[cbor(array)]
struct GenericStruct<T> {
    #[n(0)] val: T,
}

// --- Tests ---

#[test]
fn primitive_max_sizes() {
    assert_eq!(bool::MAX_CBOR_LEN, 1);
    assert_eq!(u8::MAX_CBOR_LEN, 2);
    assert_eq!(u16::MAX_CBOR_LEN, 3);
    assert_eq!(u32::MAX_CBOR_LEN, 5);
    assert_eq!(u64::MAX_CBOR_LEN, 9);
    assert_eq!(i8::MAX_CBOR_LEN, 2);
    assert_eq!(i16::MAX_CBOR_LEN, 3);
    assert_eq!(i32::MAX_CBOR_LEN, 5);
    assert_eq!(i64::MAX_CBOR_LEN, 9);
    assert_eq!(f32::MAX_CBOR_LEN, 5);
    assert_eq!(f64::MAX_CBOR_LEN, 9);
}

#[test]
fn simple_array_struct() {
    // array(3) header = 1, u8 max = 2, u32 max = 5, bool = 1 => 1 + 2 + 5 + 1 = 9
    assert_eq!(SimpleArray::MAX_CBOR_LEN, 9);
}

#[test]
fn simple_map_struct() {
    // map(2) header = 1
    // field 0: index 0 key = 1, u8 max = 2  => 3
    // field 1: index 1 key = 1, u32 max = 5 => 6
    // total = 1 + 3 + 6 = 10
    assert_eq!(SimpleMap::MAX_CBOR_LEN, 10);
}

#[test]
fn transparent_struct() {
    assert_eq!(Transparent::MAX_CBOR_LEN, u32::MAX_CBOR_LEN);
}

#[test]
fn option_field() {
    // array(2) = 1, u8 max = 2, Option<u32> max = max(1, 5) = 5
    // total = 1 + 2 + 5 = 8
    assert_eq!(WithOption::MAX_CBOR_LEN, 8);
}

#[test]
fn gap_in_indices() {
    // Fields at indices 0 and 3.
    // array(4) header = 1, field 0: u8 = 2, gaps at 1,2 = 2 nulls, field 3: u32 = 5
    // total = 1 + 2 + 2 + 5 = 10
    assert_eq!(WithGap::MAX_CBOR_LEN, 10);
}

#[test]
fn nested_struct() {
    // array(2) = 1, SimpleArray::MAX = 9, u8 = 2 => 12
    assert_eq!(Nested::MAX_CBOR_LEN, 12);
}

#[test]
fn index_only_enum() {
    // max index value cbor_len: indices 0,1,2 all fit in 1 byte
    assert_eq!(IndexOnly::MAX_CBOR_LEN, 1);
}

#[test]
fn enum_with_fields() {
    // Variant 0 (Unit): 1 (array(2)) + 1 (idx 0) + 0 (tag) + 1 (empty array body for unit)
    //   = 3
    // Variant 1 (One {x: u8}): 1 + 1 + 0 + array(1)=1 + u8=2 = 5
    // Variant 2 (Two {a: u32, b: bool}): 1 + 1 + 0 + array(2)=1 + u32=5 + bool=1 = 9
    // Max = 9
    assert_eq!(WithFields::MAX_CBOR_LEN, 9);
}

#[test]
fn flat_enum() {
    // Variant A: 1 (array header for len 1) + 1 (idx 0) = 2
    // Variant B: 1 (array header of 3 elements) + u8=2 + u32=5 + 1 (idx 1) = 9
    // Max = 9
    assert_eq!(Flat::MAX_CBOR_LEN, 9);
}

#[test]
fn generic_struct() {
    assert_eq!(<GenericStruct<u8>>::MAX_CBOR_LEN, 1 + 2); // array(1) + u8
    assert_eq!(<GenericStruct<u64>>::MAX_CBOR_LEN, 1 + 9); // array(1) + u64
}

#[test]
fn encoding_fits_in_max() {
    // Encode some values and check the actual encoded size is <= MAX_CBOR_LEN
    let val = SimpleArray { x: 255, y: 0xFFFF_FFFF, z: true };
    let mut buf = [0u8; 128];
    let len = minicbor::encode(&val, buf.as_mut_slice()).map(|_| {
        minicbor::encode::CborLen::<()>::cbor_len(&val, &mut ())
    }).unwrap();
    assert!(len <= SimpleArray::MAX_CBOR_LEN, "actual {len} > max {}", SimpleArray::MAX_CBOR_LEN);

    // Also with smaller values
    let val = SimpleArray { x: 0, y: 0, z: false };
    let len = minicbor::encode::CborLen::<()>::cbor_len(&val, &mut ());
    assert!(len <= SimpleArray::MAX_CBOR_LEN);
}

#[test]
fn array_max_size() {
    // [u8; 4]: header for 4 = 1, 4 * 2 = 8. Total = 9
    assert_eq!(<[u8; 4]>::MAX_CBOR_LEN, 9);
}

#[test]
fn tuple_max_size() {
    // (u8, u32): header for 2 = 1, u8 = 2, u32 = 5. Total = 8
    assert_eq!(<(u8, u32)>::MAX_CBOR_LEN, 8);
}

#[test]
fn struct_with_array_field() {
    // array(2) = 1, u8 = 2, [u32; 3] = 1 (array header) + 3*5 = 16
    // total = 1 + 2 + 16 = 19
    assert_eq!(WithArray::MAX_CBOR_LEN, 19);
}

#[test]
fn tuple_struct() {
    // array(2) = 1, u8 = 2, u32 = 5. Total = 8
    assert_eq!(TupleStruct::MAX_CBOR_LEN, 8);
}

#[test]
fn tagged_field() {
    // array(1) = 1, tag(1) = 1, u32 = 5. Total = 7
    assert_eq!(Tagged::MAX_CBOR_LEN, 7);
}

#[test]
fn skipped_field() {
    // Skipped fields don't contribute to encoded size.
    // array(2) = 1, u8 = 2, bool = 1. Total = 4
    assert_eq!(WithSkip::MAX_CBOR_LEN, 4);
}

#[test]
fn enum_with_tuple_variant() {
    // Variant A (Unit): 1 (array(2)) + 1 (idx) + 1 (empty body) = 3
    // Variant B: 1 (array(2)) + 1 (idx) + array(2)=1 + u8=2 + u32=5 = 10
    // Max = 10
    assert_eq!(WithTupleVariant::MAX_CBOR_LEN, 10);
}

#[test]
fn map_encoded_enum() {
    // Variant X: 1 (array(2)) + 1 (idx) + map(1)=1 + (idx0=1 + u8=2) = 6
    // Variant Y: 1 (array(2)) + 1 (idx) + map(2)=1 + (idx0=1 + u32=5) + (idx1=1 + bool=1) = 11
    // Max = 11
    assert_eq!(MapEnum::MAX_CBOR_LEN, 11);
}

#[test]
fn nonzero_max_size() {
    assert_eq!(core::num::NonZeroU8::MAX_CBOR_LEN, u8::MAX_CBOR_LEN);
    assert_eq!(core::num::NonZeroU32::MAX_CBOR_LEN, u32::MAX_CBOR_LEN);
    assert_eq!(core::num::NonZeroI64::MAX_CBOR_LEN, i64::MAX_CBOR_LEN);
}

#[test]
fn result_max_size() {
    // array(2) + discriminant(1) + max(u8=2, u32=5) = 1 + 1 + 5 = 7
    assert_eq!(<Result<u8, u32>>::MAX_CBOR_LEN, 7);
}

#[test]
fn duration_max_size() {
    // array(2) + u64=9 + u32=5 = 1 + 9 + 5 = 15
    assert_eq!(core::time::Duration::MAX_CBOR_LEN, 15);
}
