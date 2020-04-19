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

