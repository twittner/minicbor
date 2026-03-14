#![cfg(feature = "std")]

#[cfg(test)]
mod tests {
    use minicbor::{CborLen, Encode, Decode, Encoder, Decoder};

    fn encode_to_buf<T: Encode<()>>(val: &T, buf: &mut [u8]) -> usize {
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut *buf);
        e.encode(val).unwrap();
        buf_len - e.writer().len()
    }

    #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
    #[cbor(text_keys)]
    struct Simple {
        x: u8,
        y: u16,
    }

    #[test]
    fn roundtrip_simple() {
        let val = Simple { x: 1, y: 300 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Simple = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn decode_reordered_keys() {
        let mut buf = [0u8; 64];
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(2).unwrap()
         .str("y").unwrap().u16(300).unwrap()
         .str("x").unwrap().u8(1).unwrap();
        let len = buf_len - e.writer().len();
        let decoded: Simple = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(decoded, Simple { x: 1, y: 300 });
    }

    #[test]
    fn decode_unknown_keys_skipped() {
        let mut buf = [0u8; 128];
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(3).unwrap()
         .str("x").unwrap().u8(1).unwrap()
         .str("unknown_field").unwrap().str("ignored").unwrap()
         .str("y").unwrap().u16(300).unwrap();
        let len = buf_len - e.writer().len();
        let decoded: Simple = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(decoded, Simple { x: 1, y: 300 });
    }

    #[test]
    fn renamed_keys() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Renamed {
            #[cbor(key = "t")]
            temperature: i16,
        }
        let val = Renamed { temperature: -5 };
        let mut buf = [0u8; 32];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa1); // map(1)
        assert_eq!(buf[1], 0x61); // text(1)
        assert_eq!(buf[2], b't');
        let decoded: Renamed = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn optional_field_absent() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithOpt {
            required: u8,
            optional: Option<u16>,
        }
        let val = WithOpt { required: 42, optional: None };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa1); // map(1)
        let decoded: WithOpt = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn optional_field_present() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithOpt {
            required: u8,
            optional: Option<u16>,
        }
        let val = WithOpt { required: 42, optional: Some(100) };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa2); // map(2)
        let decoded: WithOpt = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn nested_text_keys() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Inner { a: u8 }
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Outer { inner: Inner, b: u16 }
        let val = Outer { inner: Inner { a: 1 }, b: 2 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Outer = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn missing_required_field_errors() {
        let mut buf = [0u8; 64];
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(1).unwrap().str("x").unwrap().u8(1).unwrap();
        let len = buf_len - e.writer().len();
        assert!(minicbor::decode::<Simple>(&buf[..len]).is_err());
    }

    #[test]
    fn enum_unit_variant() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        enum Command { Reset, Stop }
        let val = Command::Reset;
        let mut buf = [0u8; 32];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Command = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn enum_struct_variant() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        enum Command {
            Reset,
            SetInterval { #[s("ms")] ms: u32 },
        }
        let val = Command::SetInterval { ms: 500 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Command = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn skip_field() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithSkip {
            name: u8,
            #[cbor(skip)]
            cached: u32,
            value: u16,
        }
        let val = WithSkip { name: 1, cached: 9999, value: 42 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa2); // map(2)
        let decoded: WithSkip = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(decoded.name, 1);
        assert_eq!(decoded.cached, 0);
        assert_eq!(decoded.value, 42);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn vec_field() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithVec { label: u8, items: Vec<u16> }
        let val = WithVec { label: 1, items: vec![10, 20, 30] };
        let mut buf = [0u8; 128];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithVec = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn btreemap_field() {
        use std::collections::BTreeMap;
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithMap { label: u8, data: BTreeMap<u32, u16> }
        let mut data = BTreeMap::new();
        data.insert(1, 100);
        data.insert(2, 200);
        let val = WithMap { label: 1, data };
        let mut buf = [0u8; 128];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithMap = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn string_field() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithString { id: u8, name: String }
        let val = WithString { id: 1, name: "hello".into() };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithString = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn empty_struct() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Empty {}
        let val = Empty {};
        let mut buf = [0u8; 16];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa0); // map(0)
        assert_eq!(len, 1);
        let decoded: Empty = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn all_optional_all_none() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct AllOpt { a: Option<u8>, b: Option<u16> }
        let val = AllOpt { a: None, b: None };
        let mut buf = [0u8; 16];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa0); // map(0)
        let decoded: AllOpt = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn generic_struct() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Wrapper<T> { value: T, label: u8 }
        let val: Wrapper<u16> = Wrapper { value: 1000, label: 5 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Wrapper<u16> = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn encode_buffer_too_small() {
        let val = Simple { x: 1, y: 300 };
        let expected_len = minicbor::len(&val);
        for size in 0..expected_len {
            let mut buf = vec![0u8; size];
            assert!(minicbor::encode(&val, buf.as_mut_slice()).is_err());
        }
        let mut buf = vec![0u8; expected_len];
        minicbor::encode(&val, buf.as_mut_slice()).unwrap();
    }

    #[test]
    fn decode_truncated() {
        let val = Simple { x: 1, y: 300 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        for trunc in 1..len {
            assert!(minicbor::decode::<Simple>(&buf[..len - trunc]).is_err());
        }
    }

    #[test]
    fn decode_non_string_key_errors() {
        let mut buf = [0u8; 64];
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(1).unwrap().u32(0).unwrap().u8(1).unwrap();
        let len = buf_len - e.writer().len();
        assert!(minicbor::decode::<Simple>(&buf[..len]).is_err());
    }

    #[test]
    fn text_keys_wire_format() {
        let val = Simple { x: 42, y: 1000 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let mut d = Decoder::new(&buf[..len]);
        let n = d.map().unwrap().unwrap();
        assert_eq!(n, 2);
        let mut keys = vec![];
        for _ in 0..n {
            keys.push(d.str().unwrap().to_string());
            d.skip().unwrap();
        }
        keys.sort();
        assert_eq!(keys, vec!["x", "y"]);
    }

    // --- Missing from old test suite ---

    #[test]
    fn borrowed_str_field() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Borrowed<'a> {
            #[cbor(n(0))]
            name: &'a str,
            value: u8,
        }
        let val = Borrowed { name: "hello", value: 42 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Borrowed = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn btreemap_field_empty() {
        use std::collections::BTreeMap;
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithMap { label: u8, data: BTreeMap<u32, u16> }
        let val = WithMap { label: 7, data: BTreeMap::new() };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithMap = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_simple() {
        let val = Simple { x: 1, y: 300 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_renamed() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Renamed { #[cbor(key = "t")] temperature: i16 }
        let val = Renamed { temperature: -5 };
        let mut buf = [0u8; 32];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_optional_absent() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Opt { required: u8, optional: Option<u16> }
        let val = Opt { required: 42, optional: None };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_optional_present() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Opt { required: u8, optional: Option<u16> }
        let val = Opt { required: 42, optional: Some(1000) };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_nested() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Inner { a: u8 }
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Outer { inner: Inner, b: u16 }
        let val = Outer { inner: Inner { a: 1 }, b: 2 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_enum_unit() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        enum Cmd { Reset, Stop }
        let val = Cmd::Reset;
        let mut buf = [0u8; 32];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn cbor_len_enum_struct() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        enum Cmd { Reset, SetInterval { #[s("ms")] ms: u32 } }
        let val = Cmd::SetInterval { ms: 500 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn decode_indefinite_map_works() {
        // The str-index codebase supports indefinite-length maps
        let buf = [
            0xbf,                           // begin indefinite map
            0x61, b'x', 0x01,              // "x": 1
            0x61, b'y', 0x19, 0x01, 0x2c,  // "y": 300
            0xff,                           // break
        ];
        let decoded: Simple = minicbor::decode(&buf).unwrap();
        assert_eq!(decoded, Simple { x: 1, y: 300 });
    }

    #[test]
    fn encode_buffer_too_small_with_optional() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Opt { a: u8, b: Option<u32> }
        let val = Opt { a: 1, b: Some(1000) };
        let expected_len = minicbor::len(&val);
        let mut buf = vec![0u8; expected_len - 1];
        assert!(minicbor::encode(&val, buf.as_mut_slice()).is_err());
        let mut buf = vec![0u8; expected_len];
        minicbor::encode(&val, buf.as_mut_slice()).unwrap();

        let val_none = Opt { a: 1, b: None };
        let expected_len_none = minicbor::len(&val_none);
        assert!(expected_len_none < expected_len);
        let mut buf = vec![0u8; expected_len_none - 1];
        assert!(minicbor::encode(&val_none, buf.as_mut_slice()).is_err());
        let mut buf = vec![0u8; expected_len_none];
        minicbor::encode(&val_none, buf.as_mut_slice()).unwrap();
    }

    #[test]
    fn enum_tuple_variant() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        enum Value {
            None,
            Int(#[s("v")] u32),
            Pair(#[s("a")] u8, #[s("b")] u16),
        }
        let val = Value::None;
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::decode::<Value>(&buf[..len]).unwrap(), val);

        let val = Value::Int(42);
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::decode::<Value>(&buf[..len]).unwrap(), val);
        assert_eq!(minicbor::len(&val), len);

        let val = Value::Pair(1, 300);
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(minicbor::decode::<Value>(&buf[..len]).unwrap(), val);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn generic_struct_with_vec() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Collection<T> { items: Vec<T>, count: u8 }
        let val: Collection<u16> = Collection { items: vec![1, 2, 3], count: 3 };
        let mut buf = [0u8; 128];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Collection<u16> = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn hashmap_field_roundtrip() {
        use std::collections::HashMap;
        #[derive(Encode, Decode, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithHashMap { label: u8, data: HashMap<u32, u16> }
        let mut data = HashMap::new();
        data.insert(10, 1000);
        data.insert(20, 2000);
        let val = WithHashMap { label: 3, data };
        let mut buf = [0u8; 128];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithHashMap = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn skip_field_only_skipped_remain() {
        #[derive(Encode, Decode, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct AllButOne {
            keep: u8,
            #[cbor(skip)]
            drop1: u16,
            #[cbor(skip)]
            drop2: bool,
        }
        let val = AllButOne { keep: 7, drop1: 1000, drop2: true };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa1); // map(1)
        let decoded: AllButOne = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(decoded.keep, 7);
        assert_eq!(decoded.drop1, 0);
        assert_eq!(decoded.drop2, false);
    }

    #[test]
    fn skip_with_vec() {
        #[derive(Encode, Decode, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Mixed {
            items: Vec<u8>,
            #[cbor(skip)]
            cache: Vec<u8>,
            count: u16,
        }
        let val = Mixed { items: vec![1, 2, 3], cache: vec![99, 98], count: 42 };
        let mut buf = [0u8; 128];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Mixed = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(decoded.items, vec![1, 2, 3]);
        assert_eq!(decoded.cache, Vec::<u8>::new());
        assert_eq!(decoded.count, 42);
    }

    #[test]
    fn text_keys_implies_map() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct ImpliedMap { a: u8, b: u16 }
        let val = ImpliedMap { a: 1, b: 2 };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        assert_eq!(buf[0], 0xa2); // map(2), not array
        let decoded: ImpliedMap = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn vec_field_empty() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct WithVec { label: u8, items: Vec<u16> }
        let val = WithVec { label: 5, items: vec![] };
        let mut buf = [0u8; 64];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithVec = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }

    #[test]
    fn vec_of_text_keys_structs() {
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Item { id: u8, value: u16 }
        #[derive(Encode, Decode, CborLen, PartialEq, Debug)]
        #[cbor(text_keys)]
        struct Container { name: u8, items: Vec<Item> }
        let val = Container {
            name: 1,
            items: vec![Item { id: 0, value: 100 }, Item { id: 1, value: 200 }],
        };
        let mut buf = [0u8; 256];
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Container = minicbor::decode(&buf[..len]).unwrap();
        assert_eq!(val, decoded);
        assert_eq!(minicbor::len(&val), len);
    }
}
