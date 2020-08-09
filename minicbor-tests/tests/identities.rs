use minicbor::{Encode, Decode};
use quickcheck::quickcheck;

fn identity<T: Encode + Eq + for<'a> Decode<'a>>(arg: T) -> bool {
    let vec = minicbor::to_vec(&arg).unwrap();
    let val = minicbor::decode(&vec).unwrap();
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
        let mut e = minicbor::Encoder::new(Vec::new());
        e.f16(arg).unwrap();
        let val = minicbor::Decoder::new(e.as_ref()).f16().unwrap();
        half::f16::from_f32(arg).to_f32().to_bits() == val.to_bits()
    }
    quickcheck(property as fn(f32) -> bool)
}

#[test]
fn f32() {
    fn property(arg: f32) -> bool {
        let vec = minicbor::to_vec(&arg).unwrap();
        let val: f32 = minicbor::decode(&vec).unwrap();
        arg.to_bits() == val.to_bits()
    }
    quickcheck(property as fn(f32) -> bool)
}

#[test]
fn f64() {
    fn property(arg: f64) -> bool {
        let vec = minicbor::to_vec(&arg).unwrap();
        let val: f64 = minicbor::decode(&vec).unwrap();
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
fn string() {
    quickcheck(identity as fn(String) -> bool)
}

#[test]
fn bytes() {
    quickcheck(identity as fn(Vec<u8>) -> bool)
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
        let val: BinaryHeap<i32> = minicbor::decode(&vec).unwrap();
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

