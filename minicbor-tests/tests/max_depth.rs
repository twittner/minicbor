#![cfg(all(feature = "derive", feature = "alloc"))]
#![allow(dead_code)]

use minicbor::{Decode, Decoder, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map, max_depth(3))]
struct Nest {
    #[n(0)] value: u8,
    #[n(1)] next: Option<Box<Nest>>,
    #[n(2)] list: Vec<Nest>
}

impl Nest {
    fn chain(n: usize) -> Self {
        let mut this = Nest { value: 0, next: None, list: Vec::new() };
        for _ in 1 .. n {
            this = Nest { value: 0, next: Some(Box::new(this)), list: Vec::new() }
        }
        this
    }
}

fn decode<'a, T: Decode<'a, ()>>(bytes: &'a [u8]) -> Result<T, minicbor::decode::Error> {
    minicbor::decode(bytes)
}

#[test]
fn at_limit() {
    let bytes = minicbor::to_vec(Nest::chain(3)).unwrap();
    assert_eq!(Nest::chain(3), decode::<Nest>(&bytes).unwrap())
}

#[test]
fn beyond_limit() {
    let bytes = minicbor::to_vec(Nest::chain(4)).unwrap();
    let err   = decode::<Nest>(&bytes).unwrap_err();
    assert!(err.is_depth_limit_exceeded(), "{err}");
    assert!(err.position().is_some())
}

#[test]
fn nesting_through_collections_is_free() {
    let mut this = Nest { value: 0, next: None, list: Vec::new() };
    for _ in 0 .. 2 {
        this = Nest { value: 0, next: None, list: vec![this] }
    }
    let bytes = minicbor::to_vec(&this).unwrap();
    assert_eq!(this, decode::<Nest>(&bytes).unwrap());

    let deeper = Nest { value: 0, next: None, list: vec![this] };
    let bytes  = minicbor::to_vec(&deeper).unwrap();
    assert!(decode::<Nest>(&bytes).unwrap_err().is_depth_limit_exceeded())
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map)]
struct Unlimited {
    #[n(0)] next: Option<Box<Unlimited>>
}

#[test]
fn types_without_the_attribute_are_unconstrained() {
    let mut this = Unlimited { next: None };
    for _ in 0 .. 64 {
        this = Unlimited { next: Some(Box::new(this)) }
    }
    let bytes = minicbor::to_vec(&this).unwrap();
    assert_eq!(this, decode::<Unlimited>(&bytes).unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map, max_depth(1))]
struct Shallow {
    #[n(0)] next: Option<Box<Shallow>>
}

#[test]
fn max_depth_one_permits_a_single_value() {
    let one = Shallow { next: None };
    let bytes = minicbor::to_vec(&one).unwrap();
    assert_eq!(one, decode::<Shallow>(&bytes).unwrap());

    let two = Shallow { next: Some(Box::new(one)) };
    let bytes = minicbor::to_vec(&two).unwrap();
    assert!(decode::<Shallow>(&bytes).unwrap_err().is_depth_limit_exceeded())
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map, max_depth(8))]
struct Outer {
    #[n(0)] inner: Option<Box<Inner>>
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map, max_depth(2))]
struct Inner {
    #[n(0)] outer: Option<Box<Outer>>
}

#[test]
fn the_smaller_limit_constrains_its_subtree() {
    let ok = Outer {
        inner: Some(Box::new(Inner {
            outer: Some(Box::new(Outer { inner: None }))
        }))
    };
    let bytes = minicbor::to_vec(&ok).unwrap();
    assert_eq!(ok, decode::<Outer>(&bytes).unwrap());

    let too_deep = Outer {
        inner: Some(Box::new(Inner {
            outer: Some(Box::new(Outer {
                inner: Some(Box::new(Inner { outer: None }))
            }))
        }))
    };
    let bytes = minicbor::to_vec(&too_deep).unwrap();
    assert!(decode::<Outer>(&bytes).unwrap_err().is_depth_limit_exceeded())
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map, max_depth(4))]
struct Siblings {
    #[n(0)] a: Nest,
    #[n(1)] b: Nest
}

#[test]
fn the_budget_is_restored_for_siblings() {
    let v = Siblings {
        a: Nest::chain(2),
        b: Nest::chain(2)
    };
    let bytes = minicbor::to_vec(&v).unwrap();
    assert_eq!(v, decode::<Siblings>(&bytes).unwrap())
}

#[test]
fn an_enclosing_limit_caps_a_larger_inner_one() {
    #[derive(Debug, Encode, Decode)]
    #[cbor(map, max_depth(2))]
    struct Cap {
        #[n(0)] inner: Nest
    }

    let ok = Cap { inner: Nest::chain(1) };
    let bytes = minicbor::to_vec(&ok).unwrap();
    assert!(decode::<Cap>(&bytes).is_ok());

    let too_deep = Cap { inner: Nest::chain(2) };
    let bytes = minicbor::to_vec(&too_deep).unwrap();
    assert!(decode::<Cap>(&bytes).unwrap_err().is_depth_limit_exceeded())
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(max_depth(2))]
enum Tree {
    #[n(0)] Leaf(#[n(0)] u8),
    #[n(1)] Node(#[n(0)] Box<Tree>)
}

#[test]
fn enums_are_counted() {
    let ok = Tree::Node(Box::new(Tree::Leaf(1)));
    let bytes = minicbor::to_vec(&ok).unwrap();
    assert_eq!(ok, decode::<Tree>(&bytes).unwrap());

    let too_deep = Tree::Node(Box::new(Tree::Node(Box::new(Tree::Leaf(1)))));
    let bytes = minicbor::to_vec(&too_deep).unwrap();
    assert!(decode::<Tree>(&bytes).unwrap_err().is_depth_limit_exceeded())
}

#[test]
fn an_externally_lowered_budget_is_honoured() {
    let bytes = minicbor::to_vec(Nest::chain(3)).unwrap();

    let mut d = Decoder::new(&bytes);
    d.set_remaining_depth(2);
    assert!(d.decode::<Nest>().unwrap_err().is_depth_limit_exceeded());

    let mut d = Decoder::new(&bytes);
    d.set_remaining_depth(3);
    assert_eq!(Nest::chain(3), d.decode::<Nest>().unwrap())
}

#[test]
fn probing_does_not_leak_depth() {
    let bytes = minicbor::to_vec(Nest::chain(3)).unwrap();
    let mut d = Decoder::new(&bytes);
    assert!(d.probe().decode::<Nest>().is_ok());
    assert_eq!(u32::MAX, d.remaining_depth());
    assert_eq!(Nest::chain(3), d.decode::<Nest>().unwrap())
}

#[derive(Encode)]
#[cbor(map)]
struct WrittenOuter {
    #[n(0)] a: WrittenWrap,
    #[n(1)] b: Nest
}

#[derive(Encode)]
#[cbor(map)]
struct WrittenWrap {
    #[n(0)] t: WrittenTree
}

#[derive(Encode)]
enum WrittenTree {
    #[n(0)] Leaf(#[n(0)] u8),
    #[n(2)] Unknown(#[n(0)] u8)
}

#[derive(Decode)]
#[cbor(map, max_depth(4))]
struct ReadOuter {
    #[n(0)] a: Option<ReadWrap>,
    #[n(1)] b: Nest
}

#[derive(Decode)]
#[cbor(map, max_depth(4))]
struct ReadWrap {
    #[n(0)] t: ReadTree
}

#[derive(Decode)]
enum ReadTree {
    #[n(0)] Leaf(#[n(0)] u8),
    #[n(1)] Node(#[n(0)] u8)
}

// `ReadWrap` consumes budget and then fails on the unknown variant, so the
// recovery which skips the value has to hand that budget back before `b`.
#[test]
fn recovering_from_an_unknown_variant_restores_the_budget() {
    let written = WrittenOuter {
        a: WrittenWrap { t: WrittenTree::Unknown(1) },
        b: Nest::chain(3)
    };
    let bytes = minicbor::to_vec(&written).unwrap();
    let v: ReadOuter = decode(&bytes).unwrap();
    assert!(v.a.is_none());
    assert_eq!(Nest::chain(3), v.b)
}
