use minicbor::data::Type;
use minicbor::decode::{Decoder, ErrorKind};
use minicbor::encode::Encoder;
use quickcheck::{quickcheck, TestResult};

#[test]
fn trigger_length_overflow_str() {
    let input = b"\x7B\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF";
    let mut d = Decoder::new(&input[..]);
    assert!(matches!(d.str().map_err(|e| e.kind()), Err(ErrorKind::EndOfInput)))
}

#[test]
fn trigger_length_overflow_bytes() {
    let input = b"\x5B\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF";
    let mut d = Decoder::new(&input[..]);
    assert!(matches!(d.bytes().map_err(|e| e.kind()), Err(ErrorKind::EndOfInput)))
}

quickcheck! {
    fn u8_cbor_read_as_u16(n: u8) -> bool {
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        u16::from(d.probe().u8().unwrap()) == d.u16().unwrap()
    }

    fn u8_cbor_read_as_u32(n: u8) -> bool {
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        u32::from(d.probe().u8().unwrap()) == d.u32().unwrap()
    }

    fn u8_cbor_read_as_u64(n: u8) -> bool {
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        u64::from(d.probe().u8().unwrap()) == d.u64().unwrap()
    }

    fn u8_cbor_read_as_i8(n: u8) -> TestResult {
        if n > i8::MAX as u8 {
            return TestResult::discard()
        }
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        let b = d.probe().i8().unwrap() as u8 == d.u8().unwrap();
        TestResult::from_bool(b)
    }

    fn u8_cbor_read_as_i16(n: u8) -> TestResult {
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        let b = d.probe().i16().unwrap() as u8 == d.u8().unwrap();
        TestResult::from_bool(b)
    }

    fn u8_cbor_read_as_i32(n: u8) -> TestResult {
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        let b = d.probe().i32().unwrap() as u8 == d.u8().unwrap();
        TestResult::from_bool(b)
    }

    fn u8_cbor_read_as_i64(n: u8) -> TestResult {
        let mut v = [0; 2];
        minicbor::encode(n, v.as_mut()).unwrap();
        let mut d = Decoder::new(&v);
        let b = d.probe().i64().unwrap() as u8 == d.u8().unwrap();
        TestResult::from_bool(b)
    }

    fn u16_cbor_read_as_u8(n: u8) -> bool {
        let mut v = [0; 3];
        v[0] = 0x19;
        v[2] = n;
        let mut d = Decoder::new(&v);
        u16::from(d.probe().u8().unwrap()) == d.u16().unwrap()
    }

    fn u16_cbor_read_as_u32(n: u16) -> bool {
        let mut v = [0; 3];
        v[0] = 0x19;
        v[1 .. 3].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        u32::from(d.probe().u16().unwrap()) == d.u32().unwrap()
    }

    fn u16_cbor_read_as_u64(n: u16) -> bool {
        let mut v = [0; 3];
        v[0] = 0x19;
        v[1 .. 3].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        u64::from(d.probe().u16().unwrap()) == d.u64().unwrap()
    }

    fn u16_cbor_read_as_i8(n: u8) -> TestResult {
        if n > i8::MAX as u8 {
            return TestResult::discard()
        }
        let mut v = [0; 3];
        v[0] = 0x19;
        v[2] = n;
        let mut d = Decoder::new(&v);
        let b = d.probe().i8().unwrap() as u16 == d.u16().unwrap();
        TestResult::from_bool(b)
    }

    fn u16_cbor_read_as_i16(n: u16) -> TestResult {
        if n > i16::MAX as u16 {
            return TestResult::discard()
        }
        let mut v = [0; 3];
        v[0] = 0x19;
        v[1 .. 3].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i16().unwrap() as u16 == d.u16().unwrap();
        TestResult::from_bool(b)
    }

    fn u16_cbor_read_as_i32(n: u16) -> TestResult {
        let mut v = [0; 3];
        v[0] = 0x19;
        v[1 .. 3].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i32().unwrap() as u16 == d.u16().unwrap();
        TestResult::from_bool(b)
    }

    fn u16_cbor_read_as_i64(n: u16) -> TestResult {
        let mut v = [0; 3];
        v[0] = 0x19;
        v[1 .. 3].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i64().unwrap() as u16 == d.u16().unwrap();
        TestResult::from_bool(b)
    }

    fn u32_cbor_read_as_u8(n: u8) -> bool {
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[4] = n;
        let mut d = Decoder::new(&v);
        u32::from(d.probe().u8().unwrap()) == d.u32().unwrap()
    }

    fn u32_cbor_read_as_u16(n: u16) -> bool {
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[3 .. 5].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        u32::from(d.probe().u16().unwrap()) == d.u32().unwrap()
    }

    fn u32_cbor_read_as_u64(n: u32) -> bool {
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[1 .. 5].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        u64::from(d.probe().u32().unwrap()) == d.u64().unwrap()
    }

    fn u32_cbor_read_as_i8(n: u8) -> TestResult {
        if n > i8::MAX as u8 {
            return TestResult::discard()
        }
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[4] = n;
        let mut d = Decoder::new(&v);
        let b = d.probe().i8().unwrap() as u32 == d.u32().unwrap();
        TestResult::from_bool(b)
    }

    fn u32_cbor_read_as_i16(n: u16) -> TestResult {
        if n > i16::MAX as u16 {
            return TestResult::discard()
        }
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[3 .. 5].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i16().unwrap() as u32 == d.u32().unwrap();
        TestResult::from_bool(b)
    }

    fn u32_cbor_read_as_i32(n: u32) -> TestResult {
        if n > i32::MAX as u32 {
            return TestResult::discard()
        }
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[1 .. 5].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i32().unwrap() as u32 == d.u32().unwrap();
        TestResult::from_bool(b)
    }

    fn u32_cbor_read_as_i64(n: u32) -> TestResult {
        let mut v = [0; 5];
        v[0] = 0x1a;
        v[1 .. 5].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i64().unwrap() as u32 == d.u32().unwrap();
        TestResult::from_bool(b)
    }

    fn u64_cbor_read_as_u8(n: u8) -> bool {
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[8] = n;
        let mut d = Decoder::new(&v);
        u64::from(d.probe().u8().unwrap()) == d.u64().unwrap()
    }

    fn u64_cbor_read_as_u16(n: u16) -> bool {
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[7 .. 9].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        u64::from(d.probe().u16().unwrap()) == d.u64().unwrap()
    }

    fn u64_cbor_read_as_u32(n: u32) -> bool {
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[5 .. 9].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        u64::from(d.probe().u32().unwrap()) == d.u64().unwrap()
    }

    fn u64_cbor_read_as_i8(n: u8) -> TestResult {
        if n > i8::MAX as u8 {
            return TestResult::discard()
        }
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[8] = n;
        let mut d = Decoder::new(&v);
        let b = d.probe().i8().unwrap() as u64 == d.u64().unwrap();
        TestResult::from_bool(b)
    }

    fn u64_cbor_read_as_i16(n: u16) -> TestResult {
        if n > i16::MAX as u16 {
            return TestResult::discard()
        }
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[7 .. 9].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i16().unwrap() as u64 == d.u64().unwrap();
        TestResult::from_bool(b)
    }

    fn u64_cbor_read_as_i32(n: u32) -> TestResult {
        if n > i32::MAX as u32 {
            return TestResult::discard()
        }
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[5 .. 9].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i32().unwrap() as u64 == d.u64().unwrap();
        TestResult::from_bool(b)
    }

    fn u64_cbor_read_as_i64(n: u64) -> TestResult {
        if n > i64::MAX as u64 {
            return TestResult::discard()
        }
        let mut v = [0; 9];
        v[0] = 0x1b;
        v[1 .. 9].copy_from_slice(&n.to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = d.probe().i64().unwrap() as u64 == d.u64().unwrap();
        TestResult::from_bool(b)
    }

    fn i8_cbor_read_as_i16(n: i8) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 2];
        v[0] = 0x38;
        v[1] = (-1 - n) as u8;
        let mut d = Decoder::new(&v);
        let b = i16::from(d.probe().i8().unwrap()) == d.i16().unwrap();
        TestResult::from_bool(b)
    }

    fn i8_cbor_read_as_i32(n: i8) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 2];
        v[0] = 0x38;
        v[1] = (-1 - n) as u8;
        let mut d = Decoder::new(&v);
        let b = i32::from(d.probe().i8().unwrap()) == d.i32().unwrap();
        TestResult::from_bool(b)
    }

    fn i8_cbor_read_as_i64(n: i8) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 2];
        v[0] = 0x38;
        v[1] = (-1 - n) as u8;
        let mut d = Decoder::new(&v);
        let b = i64::from(d.probe().i8().unwrap()) == d.i64().unwrap();
        TestResult::from_bool(b)
    }

    fn i16_cbor_read_as_i8(n: i8) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 3];
        v[0] = 0x39;
        v[2] = (-1 - n) as u8;
        let mut d = Decoder::new(&v);
        let b = i16::from(d.probe().i8().unwrap()) == d.i16().unwrap();
        TestResult::from_bool(b)
    }

    fn i16_cbor_read_as_i32(n: i16) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 3];
        v[0] = 0x39;
        v[1 .. 3].copy_from_slice(&((-1 - n) as u16).to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = i32::from(d.probe().i16().unwrap()) == d.i32().unwrap();
        TestResult::from_bool(b)
    }

    fn i16_cbor_read_as_i64(n: i16) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 3];
        v[0] = 0x39;
        v[1 .. 3].copy_from_slice(&((-1 - n) as u16).to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = i64::from(d.probe().i16().unwrap()) == d.i64().unwrap();
        TestResult::from_bool(b)
    }

    fn i32_cbor_read_as_i8(n: i8) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 5];
        v[0] = 0x3a;
        v[4] = (-1 - n) as u8;
        let mut d = Decoder::new(&v);
        let b = i32::from(d.probe().i8().unwrap()) == d.i32().unwrap();
        TestResult::from_bool(b)
    }

    fn i32_cbor_read_as_i16(n: i16) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 5];
        v[0] = 0x3a;
        v[3 .. 5].copy_from_slice(&((-1 - n) as u16).to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = i32::from(d.probe().i16().unwrap()) == d.i32().unwrap();
        TestResult::from_bool(b)
    }

    fn i32_cbor_read_as_i64(n: i32) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 5];
        v[0] = 0x3a;
        v[1 .. 5].copy_from_slice(&((-1 - n) as u32).to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = i64::from(d.probe().i32().unwrap()) == d.i64().unwrap();
        TestResult::from_bool(b)
    }

    fn i64_cbor_read_as_i8(n: i8) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 9];
        v[0] = 0x3b;
        v[8] = (-1 - n) as u8;
        let mut d = Decoder::new(&v);
        let b = i64::from(d.probe().i8().unwrap()) == d.i64().unwrap();
        TestResult::from_bool(b)
    }

    fn i64_cbor_read_as_i16(n: i16) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 9];
        v[0] = 0x3b;
        v[7 .. 9].copy_from_slice(&((-1 - n) as u16).to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = i64::from(d.probe().i16().unwrap()) == d.i64().unwrap();
        TestResult::from_bool(b)
    }

    fn i64_cbor_read_as_i32(n: i32) -> TestResult {
        if n == 0 {
            return TestResult::discard()
        }
        let n = if n > 0 { -n } else { n };
        let mut v = [0; 9];
        v[0] = 0x3b;
        v[5 .. 9].copy_from_slice(&((-1 - n) as u32).to_be_bytes()[..]);
        let mut d = Decoder::new(&v);
        let b = i64::from(d.probe().i32().unwrap()) == d.i64().unwrap();
        TestResult::from_bool(b)
    }
}

#[test]
fn type_of_negative_ints() {
    let mut buf = [0u8; 1024];
    let mut enc = Encoder::new(&mut buf[..]);

    enc.i8(-0x80).unwrap();
    enc.i16(-0x81).unwrap();
    enc.i16(-0x80_00).unwrap();
    enc.i32(-0x80_01).unwrap();
    enc.i32(-0x80_00_00_00).unwrap();
    enc.i64(-0x80_00_00_01).unwrap();
    enc.i64(-0x80_00_00_00_00_00_00_00).unwrap();

    let mut dec = Decoder::new(&buf);

    assert_eq!(Some(Type::I8), dec.datatype().ok());
    assert_eq!(Some(-0x80), dec.i8().ok());
    assert_eq!(Some(Type::I16), dec.datatype().ok());
    assert_eq!(Some(-0x81), dec.i16().ok());
    assert_eq!(Some(Type::I16), dec.datatype().ok());
    assert_eq!(Some(-0x80_00), dec.i16().ok());
    assert_eq!(Some(Type::I32), dec.datatype().ok());
    assert_eq!(Some(-0x80_01), dec.i32().ok());
    assert_eq!(Some(Type::I32), dec.datatype().ok());
    assert_eq!(Some(-0x80_00_00_00), dec.i32().ok());
    assert_eq!(Some(Type::I64), dec.datatype().ok());
    assert_eq!(Some(-0x80_00_00_01), dec.i64().ok());
    assert_eq!(Some(Type::I64), dec.datatype().ok());
    assert_eq!(Some(-0x80_00_00_00_00_00_00_00), dec.i64().ok());
}

quickcheck! {
    fn type_of_i8(i: i8)   -> bool { check_type_of(i64::from(i)) }
    fn type_of_i16(i: i16) -> bool { check_type_of(i64::from(i)) }
    fn type_of_i32(i: i32) -> bool { check_type_of(i64::from(i)) }
    fn type_of_i64(i: i64) -> bool { check_type_of(i) }
}

fn check_type_of(i: i64) -> bool {
    let mut b = [0u8; 16];
    minicbor::encode(i, b.as_mut()).unwrap();
    let mut d = Decoder::new(&b);
    match d.datatype().unwrap() {
        Type::U8  => Some(i) == d.u8().ok().map(i64::from),
        Type::U16 => Some(i) == d.u16().ok().map(i64::from),
        Type::U32 => Some(i) == d.u32().ok().map(i64::from),
        Type::U64 => Some(i) == d.u64().ok().and_then(|n| i64::try_from(n).ok()),
        Type::I8  => Some(i) == d.i8().ok().map(i64::from),
        Type::I16 => Some(i) == d.i16().ok().map(i64::from),
        Type::I32 => Some(i) == d.i32().ok().map(i64::from),
        Type::I64 => Some(i) == d.i64().ok(),
        _         => false
    }
}
