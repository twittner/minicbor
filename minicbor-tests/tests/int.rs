use minicbor::{Encoder, Decoder};
use minicbor::data::{Int, MAX_INT, MIN_INT};
use quickcheck::{quickcheck, TestResult};

fn identity<T: Into<Int>>(v: T) -> bool {
    let i = v.into();
    let mut b = [0u8; 16];
    let mut e = Encoder::new(b.as_mut());
    e.int(i).unwrap();
    let mut d = Decoder::new(&b);
    let j = d.int().unwrap();
    assert_eq!(i, j);
    assert_eq!(i128::from(i), i128::from(j));
    true
}

#[test]
fn min_int_max_int() {
    identity(MIN_INT);
    identity(MAX_INT);
}

#[test]
fn int_bounds() {
    assert!(Int::try_from(-2_i128.pow(64)).is_ok());
    assert!(Int::try_from(2_i128.pow(64) - 1).is_ok());
    assert!(Int::try_from(-2_i128.pow(64) - 1).is_err());
    assert!(Int::try_from(2_i128.pow(64)).is_err())
}

quickcheck! {
    fn to_from_i8(n: i8) -> bool {
        let i = Int::from(n);
        n == i8::try_from(i).unwrap()
    }

    fn to_from_i16(n: i16) -> bool {
        let i = Int::from(n);
        n == i16::try_from(i).unwrap()
    }

    fn to_from_i32(n: i32) -> bool {
        let i = Int::from(n);
        n == i32::try_from(i).unwrap()
    }

    fn to_from_i64(n: i64) -> bool {
        let i = Int::from(n);
        n == i64::try_from(i).unwrap()
    }

    fn to_from_i128(n: i128) -> TestResult {
        if let Ok(i) = Int::try_from(n) {
            TestResult::from_bool(n == i128::from(i))
        } else {
            TestResult::discard()
        }
    }

    fn to_from_u8(n: u8) -> bool {
        let i = Int::from(n);
        n == u8::try_from(i).unwrap()
    }

    fn to_from_u16(n: u16) -> bool {
        let i = Int::from(n);
        n == u16::try_from(i).unwrap()
    }

    fn to_from_u32(n: u32) -> bool {
        let i = Int::from(n);
        n == u32::try_from(i).unwrap()
    }

    fn to_from_u64(n: u64) -> bool {
        let i = Int::from(n);
        n == u64::try_from(i).unwrap()
    }

    fn to_from_u128(n: u128) -> TestResult {
        if let Ok(i) = Int::try_from(n) {
            TestResult::from_bool(n == u128::try_from(i).unwrap())
        } else {
            TestResult::discard()
        }
    }

    fn int_u8(n: u8) -> bool {
        identity(n)
    }

    fn int_u16(n: u16) -> bool {
        identity(n)
    }

    fn int_u32(n: u32) -> bool {
        identity(n)
    }

    fn int_u64_id(n: u64) -> bool {
        identity(n)
    }

    fn int_i8_id(n: i8) -> bool {
        identity(n)
    }

    fn int_i16_id(n: i16) -> bool {
        identity(n)
    }

    fn int_i32_id(n: i32) -> bool {
        identity(n)
    }

    fn int_i64_id(n: i64) -> bool {
        identity(n)
    }

    fn encode_i64_decode_int(i: i64) -> bool {
        let v = minicbor::to_vec(i).unwrap();
        let j = minicbor::decode::<Int>(&v).unwrap();
        i == i64::try_from(j).unwrap()
    }

    fn encode_int_decode_i64(n: i64) -> bool {
        let i = Int::from(n);
        assert_eq!(i64::try_from(i).ok(), Some(n));
        let v = minicbor::to_vec(i).unwrap();
        let j = minicbor::decode::<i64>(&v).unwrap();
        n == j
    }

    fn i128_as_int(n: i128) -> TestResult {
        let i = match Int::try_from(n) {
            Ok(i)  => i,
            Err(_) => return TestResult::discard()
        };
        assert_eq!(i128::from(i), n);
        let v = minicbor::to_vec(i).unwrap();
        let j = minicbor::decode::<Int>(&v).unwrap();
        TestResult::from_bool(n == i128::from(j))
    }
}
