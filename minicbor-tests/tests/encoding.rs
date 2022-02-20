use minicbor::{Encode, Decode};

const NULL: u8 = 0xf6;

#[test]
fn encode_as_array() {
    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(array)]
    struct T {
        #[n(0)] a: Option<u8>,
        #[n(2)] b: Option<u8>,
        #[n(5)] c: Option<u8>
    }

    // empty value => empty array
    let v = T { a: None, b: None, c: None };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x80][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // empty suffix is not encoded
    let v = T { a: Some(1), b: None, c: None };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x81, 1][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // gaps are filled with nulls
    let v = T { a: Some(1), b: Some(2), c: None };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x83, 1, NULL, 2][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // more gaps to fill
    let v = T { a: Some(1), b: Some(2), c: Some(3) };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x86, 1, NULL, 2, NULL, NULL, 3][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // and even more
    let v = T { a: Some(1), b: None, c: Some(3) };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x86, 1, NULL, NULL, NULL, NULL, 3][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // empty prefix is filled with nulls too
    let v = T { a: None, b: None, c: Some(3) };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x86, NULL, NULL, NULL, NULL, NULL, 3][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap())
}

#[test]
fn encode_as_map() {
    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(map)]
    struct T {
        #[n(0)] a: Option<u8>,
        #[n(2)] b: Option<u8>,
        #[n(5)] c: Option<u8>
    }

    // empty value => empty map
    let v = T { a: None, b: None, c: None };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa0][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // empty suffix is not encoded
    let v = T { a: Some(1), b: None, c: None };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa1, 0, 1][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // gaps are not encoded
    let v = T { a: Some(1), b: Some(2), c: None };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa2, 0, 1, 2, 2][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // gaps are not encoded
    let v = T { a: Some(1), b: Some(2), c: Some(3) };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa3, 0, 1, 2, 2, 5, 3][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // gaps are not encoded
    let v = T { a: Some(1), b: None, c: Some(3) };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa2, 0, 1, 5, 3][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    // gaps are not encoded
    let v = T { a: None, b: None, c: Some(3) };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa1, 5, 3][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap())
}

#[test]
fn mixed_encoding_1() {
    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(array)]
    struct T {
        #[n(0)] a: u8,
        #[n(1)] e: E
    }

    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(map)]
    enum E {
        #[n(0)] A,
        #[n(1)] B { #[n(0)] x: u8 },
        #[n(2)] #[cbor(array)] C { #[n(0)] z: u8 }
    }

    let v = T { a: 1, e: E::A };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x82, 1, 0x82, 0, 0xa0][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    let v = T { a: 1, e: E::B { x: 2 } };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x82, 1, 0x82, 1, 0xa1, 0, 2][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    let v = T { a: 1, e: E::C { z: 2 } };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0x82, 1, 0x82, 2, 0x81, 2][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap())
}

#[test]
fn mixed_encoding_2() {
    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(map)]
    struct T {
        #[n(0)] a: u8,
        #[n(1)] e: E
    }

    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(array)]
    enum E {
        #[n(0)] A,
        #[n(1)] B { #[n(0)] x: u8 },
        #[n(2)] #[cbor(map)] C { #[n(0)] z: u8 }
    }

    let v = T { a: 1, e: E::A };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa2, 0, 1, 1, 0x82, 0, 0x80][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    let v = T { a: 1, e: E::B { x: 2 } };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa2, 0, 1, 1, 0x82, 1, 0x81, 2][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap());

    let v = T { a: 1, e: E::C { z: 2 } };

    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(&[0xa2, 0, 1, 1, 0x82, 2, 0xa1, 0, 2][..], &bytes[..]);
    assert_eq!(v, minicbor::decode(&bytes).unwrap())
}

#[test]
fn index_only_enum() {
    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    #[cbor(index_only)]
    enum E {
        #[n(0)] A,
        #[n(1)] B
    }

    let bytes = minicbor::to_vec(&E::A).unwrap();
    assert_eq!(&[0][..], &bytes[..]);
    assert_eq!(E::A, minicbor::decode(&bytes).unwrap());

    let bytes = minicbor::to_vec(&E::B).unwrap();
    assert_eq!(&[1][..], &bytes[..]);
    assert_eq!(E::B, minicbor::decode(&bytes).unwrap());

    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(4).unwrap()
        .encode(E::A).unwrap()
        .encode(E::B).unwrap()
        .encode(32u8).unwrap()
        .encode("foo").unwrap();

    let mut d = minicbor::Decoder::new(e.as_ref());
    assert_eq!(Some(4), d.array().unwrap());
    assert_eq!(E::A, d.probe().decode().unwrap());
    assert_eq!(0, d.probe().u32().unwrap());
    d.skip().unwrap();
    assert_eq!(E::B, d.probe().decode().unwrap());
    assert_eq!(1, d.probe().u32().unwrap());
    d.skip().unwrap();
    assert_eq!(32u8, d.probe().decode().unwrap());
    d.skip().unwrap();
    assert_eq!("foo", d.probe().str().unwrap());
    d.skip().unwrap();
    assert!(d.skip().unwrap_err().is_end_of_input())
}

#[test]
fn regular_enum() {
    #[derive(Debug, Encode, Decode, PartialEq, Eq)]
    enum E {
        #[n(0)] A,
        #[n(1)] B
    }

    let bytes = minicbor::to_vec(&E::A).unwrap();
    assert_eq!(&[0x82, 0, 0x80][..], &bytes[..]);
    assert_eq!(E::A, minicbor::decode(&bytes).unwrap());

    let bytes = minicbor::to_vec(&E::B).unwrap();
    assert_eq!(&[0x82, 1, 0x80][..], &bytes[..]);
    assert_eq!(E::B, minicbor::decode(&bytes).unwrap());

    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(4).unwrap()
        .encode(E::A).unwrap()
        .encode(E::B).unwrap()
        .encode(32u8).unwrap()
        .encode("foo").unwrap();

    let mut d = minicbor::Decoder::new(e.as_ref());
    assert_eq!(Some(4), d.array().unwrap());
    assert_eq!(E::A, d.probe().decode().unwrap());
    assert_eq!(Some(2), d.probe().array().unwrap());
    d.skip().unwrap();
    assert_eq!(E::B, d.probe().decode().unwrap());
    assert_eq!(Some(2), d.probe().array().unwrap());
    d.skip().unwrap();
    assert_eq!(32u8, d.probe().decode().unwrap());
    d.skip().unwrap();
    assert_eq!("foo", d.probe().str().unwrap());
    d.skip().unwrap();
    assert!(d.skip().unwrap_err().is_end_of_input())
}

