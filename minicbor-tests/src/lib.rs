// Some compile-time tests mostly testing derive functionality.

#![allow(unused)]

use minicbor::{Encode, Encoder, Decode, Decoder};
use std::borrow::Cow;

mod structs;
mod enums;

#[derive(Encode, Decode)] struct S0;
#[derive(Encode, Decode)] enum E0 {}
#[derive(Encode, Decode)] enum Ev { #[n(0)] V }

// implicit borrow of &str
#[derive(Decode)] struct S1<'a> { #[n(0)] field: &'a str }

fn s1_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> S1<'a> {
    d.decode().unwrap()
}

// no implicit borrow of Cow<'_, str>
#[derive(Decode)] struct S2<'a> { #[n(0)] field: Cow<'a, str> }

fn s2_is_free<'a, 'b>(d: &mut Decoder<'b>) -> S2<'a> {
    d.decode().unwrap()
}

// explicit borrow of Cow<'_, str>
#[derive(Decode)] struct S3<'a> { #[b(0)] field: Cow<'a, str> }

fn s3_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> S3<'a> {
    d.decode().unwrap()
}

// implicit borrow of Option<&str>
#[derive(Decode)] struct S4<'a> { #[n(0)] field: Option<&'a str> }

fn s4_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> S4<'a> {
    d.decode().unwrap()
}

// implicit borrow of &[u8]
#[derive(Decode)] struct B1<'a> { #[n(0)] field: &'a [u8] }

fn b1_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> B1<'a> {
    d.decode().unwrap()
}

// no implicit borrow of Cow<'_, [u8]>
#[derive(Decode)] struct B2<'a> { #[n(0)] field: Cow<'a, [u8]> }

fn b2_is_free<'a, 'b>(d: &mut Decoder<'b>) -> B2<'a> {
    d.decode().unwrap()
}

// explicit borrow of Cow<'_, [u8]>
#[derive(Decode)] struct B3<'a> { #[b(0)] field: Cow<'a, [u8]> }

fn b3_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> B3<'a> {
    d.decode().unwrap()
}

// implicit borrow of Option<&[u8]>
#[derive(Decode)] struct B4<'a> { #[n(0)] field: Option<&'a [u8]> }

fn b4_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> B4<'a> {
    d.decode().unwrap()
}

// explicit borrow
#[derive(Decode)] struct N1<'a> { #[b(0)] field: S1<'a> }

fn n1_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> N1<'a> {
    d.decode().unwrap()
}

// no implicit borrow of arbitrary types
#[derive(Decode)] struct N2<'a> { #[n(0)] field: S2<'a> }

fn n2_is_free<'a, 'b>(d: &mut Decoder<'b>) -> N2<'a> {
    d.decode().unwrap()
}

// implicit borrow for &str in enums
#[derive(Decode)]
enum E1<'a> {
    #[n(0)] A(#[n(0)] &'a str)
}

fn e1_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> E1<'a> {
    d.decode().unwrap()
}

// implicit borrow for &str in enums
#[derive(Decode)]
enum E2<'a> {
    #[n(0)] A { #[n(0)] field: &'a str }
}

fn e2_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> E2<'a> {
    d.decode().unwrap()
}

// no implicit borrow of arbitrary types
#[derive(Decode)]
enum E3<'a> {
    #[n(0)] A(#[n(0)] S2<'a>)
}

fn e3_is_free<'a, 'b>(d: &mut Decoder<'b>) -> E3<'a> {
    d.decode().unwrap()
}

// explicit borrow
#[derive(Decode)]
enum E4<'a> {
    #[b(0)] A(#[b(0)] S2<'a>)
}

fn e4_is_bound<'a, 'b: 'a>(d: &mut Decoder<'b>) -> E4<'a> {
    d.decode().unwrap()
}

