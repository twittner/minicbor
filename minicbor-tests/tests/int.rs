#![cfg(feature = "std")]

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

    fn u128_id(n: u128) -> bool {
        let v = minicbor::to_vec(n).unwrap();
        assert_eq!(minicbor::len(n), v.len());
        let m = minicbor::decode::<u128>(&v).unwrap();
        n == m
    }

    fn i128_id(n: i128) -> bool {
        let v = minicbor::to_vec(n).unwrap();
        assert_eq!(minicbor::len(n), v.len());
        let m = minicbor::decode::<i128>(&v).unwrap();
        n == m
    }

    fn u128_compat_u64(n: u64) -> bool {
        // u64 encoded values must round-trip through u128 decoding.
        let v = minicbor::to_vec(n).unwrap();
        let m = minicbor::decode::<u128>(&v).unwrap();
        u128::from(n) == m
    }

    fn i128_compat_i64(n: i64) -> bool {
        let v = minicbor::to_vec(n).unwrap();
        let m = minicbor::decode::<i128>(&v).unwrap();
        i128::from(n) == m
    }
}

#[test]
fn u128_edges() {
    fn roundtrip(n: u128) {
        let v = minicbor::to_vec(n).unwrap();
        assert_eq!(minicbor::len(n), v.len());
        assert_eq!(n, minicbor::decode::<u128>(&v).unwrap());
    }
    roundtrip(0);
    roundtrip(1);
    roundtrip(u64::MAX as u128);
    roundtrip(u64::MAX as u128 + 1);
    roundtrip(u128::MAX);
}

#[test]
fn i128_edges() {
    fn roundtrip(n: i128) {
        let v = minicbor::to_vec(n).unwrap();
        assert_eq!(minicbor::len(n), v.len());
        assert_eq!(n, minicbor::decode::<i128>(&v).unwrap());
    }
    roundtrip(0);
    roundtrip(1);
    roundtrip(-1);
    roundtrip(i64::MAX as i128);
    roundtrip(i64::MIN as i128);
    roundtrip(u64::MAX as i128);
    roundtrip(-(u64::MAX as i128) - 1); // -2^64, lowest Int-representable value
    roundtrip(u64::MAX as i128 + 1);     // first value requiring bignum
    roundtrip(-(u64::MAX as i128) - 2);  // first negative value requiring bignum
    roundtrip(i128::MAX);
    roundtrip(i128::MIN);
}

#[test]
fn u128_bignum_encoding() {
    // u128::MAX -> tag 2 (0xc2), bytes(16) -> 0x50, then 16 x 0xff
    let v = minicbor::to_vec(u128::MAX).unwrap();
    assert_eq!(v[0], 0xc2);
    assert_eq!(v[1], 0x50);
    assert_eq!(&v[2..], &[0xff; 16]);
}

#[test]
fn i128_negative_bignum_encoding() {
    // i128::MIN = -2^127. -1 - i128::MIN = 2^127 - 1.
    // Encoded as tag 3 (0xc3) + bstr(16) + big-endian bytes of (2^127 - 1).
    let v = minicbor::to_vec(i128::MIN).unwrap();
    assert_eq!(v[0], 0xc3);
    assert_eq!(v[1], 0x50);
    let mut expected = [0xff_u8; 16];
    expected[0] = 0x7f;
    assert_eq!(&v[2..], &expected);
}

#[test]
fn u128_decodes_with_leading_zero_bytes() {
    // Non-canonical encoding: tag 2 + bstr containing 16 bytes, first three zero.
    let mut v = vec![0xc2, 0x50, 0x00, 0x00, 0x00];
    v.extend_from_slice(&[0xab; 13]);
    let n = minicbor::decode::<u128>(&v).unwrap();
    let mut expected = [0_u8; 16];
    expected[3..].copy_from_slice(&[0xab; 13]);
    assert_eq!(n, u128::from_be_bytes(expected));
}

#[test]
fn u128_rejects_negative_bignum() {
    let v = minicbor::to_vec(-1i128).unwrap();
    assert_eq!(v[0], 0x20); // -1 encodes as native signed, not bignum
    // Force-construct a negative bignum and confirm u128 rejects it.
    let v = vec![0xc3, 0x42, 0x01, 0x00];
    assert!(minicbor::decode::<u128>(&v).is_err());
}

#[test]
fn i128_rejects_overflowing_positive_bignum() {
    // 17-byte bignum, all 0xff, overflows u128 capacity.
    let mut v = vec![0xc2, 0x51];
    v.extend_from_slice(&[0xff; 17]);
    assert!(minicbor::decode::<i128>(&v).is_err());
    // 16-byte positive bignum equal to 2^127 must overflow i128.
    let mut v = vec![0xc2, 0x50];
    v.push(0x80);
    v.extend_from_slice(&[0; 15]);
    assert!(minicbor::decode::<i128>(&v).is_err());
}
