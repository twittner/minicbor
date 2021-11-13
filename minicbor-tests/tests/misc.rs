use minicbor::decode::{Decoder, Error};
use quickcheck::{quickcheck, TestResult};

#[test]
fn trigger_length_overflow_str() {
    let input = b"\x7B\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF";
    let mut d = Decoder::new(&input[..]);
    assert!(matches!(d.str(), Err(Error::EndOfInput)))
}

#[test]
fn trigger_length_overflow_bytes() {
    let input = b"\x5B\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF";
    let mut d = Decoder::new(&input[..]);
    assert!(matches!(d.bytes(), Err(Error::EndOfInput)))
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
        u64::from(d.probe().u64().unwrap()) == d.u64().unwrap()
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

