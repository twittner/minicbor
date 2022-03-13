use minicbor::{Encode, Encoder, Decode, Decoder};
use minicbor::data::{Int, Type};
use quickcheck::quickcheck;
use std::marker::PhantomData;

fn identity<T: Encode + Eq + for<'a> Decode<'a>>(arg: T) -> bool {
    let vec = minicbor::to_vec(&arg).unwrap();
    let mut dec = Decoder::new(&vec);
    let val = dec.decode().unwrap();
    assert_eq!(dec.position(), vec.len());
    arg == val
}

#[test]
fn u8() {
    quickcheck(identity as fn(u8) -> bool)
}

#[test]
fn u16() {
    quickcheck(identity as fn(u16) -> bool)
}

#[test]
fn u32() {
    quickcheck(identity as fn(u32) -> bool)
}

#[test]
fn u64() {
    quickcheck(identity as fn(u64) -> bool)
}

#[test]
fn i8() {
    quickcheck(identity as fn(i8) -> bool)
}

#[test]
fn i16() {
    quickcheck(identity as fn(i16) -> bool)
}

#[test]
fn i32() {
    quickcheck(identity as fn(i32) -> bool)
}

#[test]
fn i64() {
    quickcheck(identity as fn(i64) -> bool)
}

#[test]
fn int() {
    assert!(identity(Int::try_from(-2_i128.pow(64)).unwrap()));
    assert!(identity(Int::from(-1_i64)));
    assert!(identity(Int::from(0_i64)));
    assert!(identity(Int::from(1_i64)));
    assert!(identity(Int::try_from(2_i128.pow(64) - 1).unwrap()))
}

#[test]
fn nonzero_u8() {
    quickcheck(identity as fn(core::num::NonZeroU8) -> bool)
}

#[test]
fn nonzero_u16() {
    quickcheck(identity as fn(core::num::NonZeroU16) -> bool)
}

#[test]
fn nonzero_u32() {
    quickcheck(identity as fn(core::num::NonZeroU32) -> bool)
}

#[test]
fn nonzero_u64() {
    quickcheck(identity as fn(core::num::NonZeroU64) -> bool)
}

#[test]
fn f16() {
    fn property(arg: f32) -> bool {
        let mut e = Encoder::new(Vec::new());
        e.f16(arg).unwrap();
        let mut d = Decoder::new(e.as_ref());
        let val = d.f16().unwrap();
        assert_eq!(d.position(), e.as_ref().len());
        half::f16::from_f32(arg).to_f32().to_bits() == val.to_bits()
    }
    quickcheck(property as fn(f32) -> bool)
}

#[test]
fn f32() {
    fn property(arg: f32) -> bool {
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        let val = dec.f32().unwrap();
        assert_eq!(dec.position(), vec.len());
        arg.to_bits() == val.to_bits()
    }
    quickcheck(property as fn(f32) -> bool)
}

#[test]
fn f64() {
    fn property(arg: f64) -> bool {
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        let val = dec.f64().unwrap();
        assert_eq!(dec.position(), vec.len());
        arg.to_bits() == val.to_bits()
    }
    quickcheck(property as fn(f64) -> bool)
}

#[test]
fn bool() {
    quickcheck(identity as fn(bool) -> bool)
}

#[test]
fn char() {
    quickcheck(identity as fn(char) -> bool)
}

#[test]
fn option() {
    quickcheck(identity as fn(Option<char>) -> bool)
}

#[test]
fn option_unit() {
    quickcheck(identity as fn(Option<()>) -> bool)
}

#[test]
fn phantom_data() {
    let p: PhantomData<fn()> = PhantomData;
    assert!(identity(p))
}

#[test]
fn result() {
    quickcheck(identity as fn(Result<u64, String>) -> bool)
}

#[test]
fn result_unit_ok() {
    quickcheck(identity as fn(Result<(), String>) -> bool)
}

#[test]
fn result_unit_err() {
    quickcheck(identity as fn(Result<u64, ()>) -> bool)
}

#[test]
fn result_option() {
    quickcheck(identity as fn(Result<Option<()>, String>) -> bool)
}

#[test]
fn string() {
    quickcheck(identity as fn(String) -> bool)
}

#[test]
fn bytes() {
    quickcheck(identity as fn(Vec<u8>) -> bool)
}

#[test]
fn byte_slice() {
    use minicbor::bytes::ByteSlice;

    fn property(arg: Vec<u8>) -> bool {
        let arg: &ByteSlice = arg.as_slice().into();
        let vec = minicbor::to_vec(arg).unwrap();
        let mut dec = Decoder::new(&vec);
        assert_eq!(Some(Type::Bytes), dec.datatype().ok());
        dec.set_position(0);
        let val: &ByteSlice = dec.decode().unwrap();
        assert_eq!(dec.position(), vec.len());
        arg == val
    }

    quickcheck(property as fn(Vec<u8>) -> bool)
}

#[test]
fn byte_array() {
    use minicbor::bytes::ByteArray;

    let arg = ByteArray::from([1,2,3,4,5,6,7,8]);
    let vec = minicbor::to_vec(&arg).unwrap();
    let mut dec = Decoder::new(&vec);
    assert_eq!(Some(Type::Bytes), dec.datatype().ok());
    dec.set_position(0);
    let val: ByteArray<8> = dec.decode().unwrap();
    assert_eq!(arg, val);
    assert_eq!(dec.position(), vec.len())
}

#[test]
fn byte_vec() {
    use minicbor::bytes::ByteVec;

    fn property(arg: Vec<u8>) -> bool {
        let arg = ByteVec::from(arg);
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        assert_eq!(Some(Type::Bytes), dec.datatype().ok());
        dec.set_position(0);
        let val: ByteVec = dec.decode().unwrap();
        assert_eq!(dec.position(), vec.len());
        arg == val
    }

    quickcheck(property as fn(Vec<u8>) -> bool)
}

#[test]
fn vecdeque() {
    quickcheck(identity as fn(std::collections::VecDeque<u32>) -> bool)
}

#[test]
fn linkedlist() {
    quickcheck(identity as fn(std::collections::LinkedList<String>) -> bool)
}

#[test]
fn binaryheap() {
    use std::collections::{BTreeSet, BinaryHeap};
    use std::iter::FromIterator;

    fn property(arg: BinaryHeap<i32>) -> bool {
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        let val: BinaryHeap<i32> = dec.decode().unwrap();
        assert_eq!(dec.position(), vec.len());
        let a = BTreeSet::from_iter(arg.into_iter());
        let b = BTreeSet::from_iter(val.into_iter());
        a == b
    }

    quickcheck(property as fn(std::collections::BinaryHeap<i32>) -> bool)
}

#[test]
fn hashset() {
    quickcheck(identity as fn(std::collections::HashSet<u32>) -> bool)
}

#[test]
fn btreeset() {
    quickcheck(identity as fn(std::collections::BTreeSet<u32>) -> bool)
}

#[test]
fn boxed() {
    quickcheck(identity as fn(x: Box<u32>) -> bool)
}

#[test]
fn duration() {
    quickcheck(identity as fn(std::time::Duration) -> bool)
}

#[test]
fn ip() {
    quickcheck(identity as fn(std::net::IpAddr) -> bool)
}

#[test]
fn ipv4() {
    quickcheck(identity as fn(std::net::Ipv4Addr) -> bool)
}

#[test]
fn ipv6() {
    quickcheck(identity as fn(std::net::Ipv6Addr) -> bool)
}

#[test]
fn socketaddr() {
    quickcheck(identity as fn(std::net::SocketAddr) -> bool)
}

#[test]
fn socketaddrv4() {
    quickcheck(identity as fn(std::net::SocketAddrV4) -> bool)
}

#[test]
fn socketaddrv6() {
    fn property(mut x: std::net::SocketAddrV6) -> bool {
        x.set_flowinfo(0);
        x.set_scope_id(0);
        identity(x)
    }
    quickcheck(property as fn(std::net::SocketAddrV6) -> bool)
}

#[test]
fn tuple1() {
    quickcheck(identity as fn((bool,)) -> bool)
}

#[test]
fn tuple2() {
    quickcheck(identity as fn((bool, u8)) -> bool)
}

#[test]
fn tuple3() {
    quickcheck(identity as fn((bool, u8, char)) -> bool)
}

#[test]
fn tuple4() {
    quickcheck(identity as fn((bool, u8, char, String)) -> bool)
}

#[test]
fn tuple5() {
    quickcheck(identity as fn((bool, u8, char, String, Option<i16>)) -> bool)
}

#[test]
fn tuple6() {
    quickcheck(identity as fn((bool, u8, char, String, Option<i16>, Vec<u32>)) -> bool)
}

#[test]
fn tuple7() {
    quickcheck(identity as fn((bool, u8, char, String, Option<i16>, Vec<u32>, u8)) -> bool)
}

#[test]
fn tuple8() {
    quickcheck(identity as fn((bool, u8, char, String, Option<i16>, Vec<u32>, u8, i64)) -> bool)
}

#[test]
fn array1() {
    fn property(a: bool) -> bool {
        identity([a])
    }
    quickcheck(property as fn(bool) -> bool)
}

#[test]
fn array2() {
    fn property(a: bool, b: bool) -> bool {
        identity([a, b])
    }
    quickcheck(property as fn(bool, bool) -> bool)
}

#[test]
fn array3() {
    fn property(a: bool, b: bool, c: bool) -> bool {
        identity([a, b, c])
    }
    quickcheck(property as fn(bool, bool, bool) -> bool)
}

#[test]
fn array4() {
    fn property(a: bool, b: bool, c: bool, d: bool) -> bool {
        identity([a, b, c, d])
    }
    quickcheck(property as fn(bool, bool, bool, bool) -> bool)
}

#[test]
fn bound_u32() {
    quickcheck(identity as fn(core::ops::Bound<u32>) -> bool)
}

#[test]
fn bound_i32() {
    quickcheck(identity as fn(core::ops::Bound<i32>) -> bool)
}

#[test]
fn path_buf() {
    quickcheck(identity as fn(std::path::PathBuf) -> bool)
}

#[test]
fn path() {
    fn property(arg: std::path::PathBuf) -> bool {
        let vec = minicbor::to_vec(arg.as_path()).unwrap();
        let mut dec = Decoder::new(&vec);
        let val = dec.decode::<&std::path::Path>().unwrap();
        assert_eq!(dec.position(), vec.len());
        arg == val
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn range() {
    quickcheck(identity as fn(core::ops::Range<i32>) -> bool)
}

#[test]
fn range_from() {
    quickcheck(identity as fn(core::ops::RangeFrom<i32>) -> bool)
}

#[test]
fn range_inclusive() {
    quickcheck(identity as fn(core::ops::RangeInclusive<i32>) -> bool)
}

#[test]
fn range_to() {
    quickcheck(identity as fn(core::ops::RangeTo<i32>) -> bool)
}

#[test]
fn range_to_inclusive() {
    quickcheck(identity as fn(core::ops::RangeToInclusive<i32>) -> bool)
}

#[test]
fn wrapping() {
    quickcheck(identity as fn(core::num::Wrapping<i32>) -> bool)
}

#[test]
fn cell() {
    fn property(n: u32) -> bool {
        identity(core::cell::Cell::new(n))
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn refcell() {
    fn property(n: u32) -> bool {
        identity(core::cell::RefCell::new(n))
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn system_time() {
    fn property(t: std::time::SystemTime) -> bool {
        if t < std::time::UNIX_EPOCH {
            minicbor::to_vec(t).is_err()
        } else {
            identity(t)
        }
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn atomic_u64() {
    fn property(n: u64) -> bool {
        let arg = core::sync::atomic::AtomicU64::new(n);
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        let val = dec.decode::<core::sync::atomic::AtomicU64>().unwrap();
        assert_eq!(dec.position(), vec.len());
        n == val.load(core::sync::atomic::Ordering::SeqCst)
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn atomic_i64() {
    fn property(n: i64) -> bool {
        let arg = core::sync::atomic::AtomicI64::new(n);
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        let val = dec.decode::<core::sync::atomic::AtomicI64>().unwrap();
        assert_eq!(dec.position(), vec.len());
        n == val.load(core::sync::atomic::Ordering::SeqCst)
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn atomic_bool() {
    fn property(n: bool) -> bool {
        let arg = core::sync::atomic::AtomicBool::new(n);
        let vec = minicbor::to_vec(&arg).unwrap();
        let mut dec = Decoder::new(&vec);
        let val = dec.decode::<core::sync::atomic::AtomicBool>().unwrap();
        assert_eq!(dec.position(), vec.len());
        n == val.load(core::sync::atomic::Ordering::SeqCst)
    }
    quickcheck(property as fn(_) -> bool)
}

#[test]
fn null() {
    let mut buf = [0; 1];
    let mut enc = Encoder::new(buf.as_mut());
    enc.null().unwrap();
    let mut dec = Decoder::new(buf.as_ref());
    assert!(dec.null().is_ok());
    assert_eq!(1, dec.position());
}

#[test]
fn undefined() {
    let mut buf = [0; 1];
    let mut enc = Encoder::new(buf.as_mut());
    enc.undefined().unwrap();
    let mut dec = Decoder::new(buf.as_ref());
    assert!(dec.undefined().is_ok());
    assert_eq!(1, dec.position())
}

