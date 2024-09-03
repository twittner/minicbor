use minicbor::{Encode, Decode};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Serialize, Deserialize};
use std::{borrow::Cow, iter, path::Path};
use std::time::Instant;

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct AddressBook<'a> {
    #[n(0)] timestamp: u64,
    #[b(1)] #[serde(borrow)] entries: Vec<Entry<'a>>,
    #[b(2)] #[serde(borrow)] style: Option<Style<'a>>,
    #[n(3)] rating: Option<f64>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct Entry<'a> {
    #[b(0)] #[serde(borrow)] firstname: Cow<'a, str>,
    #[b(1)] #[serde(borrow)] lastname: Cow<'a, str>,
    #[n(2)] birthday: u32,
    #[b(3)] #[serde(borrow)] addresses: Vec<Address<'a>>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct Address<'a> {
    #[b(0)] #[serde(borrow)] street: Cow<'a, str>,
    #[b(1)] #[serde(borrow)] houseno: Cow<'a, str>,
    #[n(2)] postcode: u32,
    #[b(3)] #[serde(borrow)] city: Cow<'a, str>,
    #[b(4)] #[serde(borrow)] country: Cow<'a, str>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
enum Style<'a> {
    #[n(0)] Version1,
    #[n(1)] Version2,
    #[n(2)] Version3(#[n(0)] bool, #[n(1)] u64),
    #[b(3)] Version4 {
        #[b(0)] #[serde(borrow)] path: Cow<'a, str>,
        #[n(1)] timestamp: u64
    }
}

fn bench(label: &str, mut f: impl FnMut() -> bool) {
    const ITERATIONS: u32 = 1000;
    let start = Instant::now();
    for _ in 0 .. ITERATIONS {
        assert!(f())
    }
    eprintln!("{label:24} {:0.2?}", start.elapsed() / ITERATIONS)
}

#[test]
fn packed_serde_cbor_vs_minicbor() {
    let book = gen_addressbook(16);
    let book_bytes_serde = serde_cbor::ser::to_vec_packed(&book).unwrap();
    let book_bytes_minicbor = minicbor::to_vec(&book).unwrap();

    println!();
    bench("encode/serde_cbor", || serde_cbor::ser::to_vec_packed(&book).is_ok());
    bench("encode/minicbor",   || minicbor::to_vec(&book).is_ok());
    bench("decode/serde_cbor", || serde_cbor::from_slice::<AddressBook>(&book_bytes_serde).is_ok());
    bench("decode/minicbor",   || minicbor::decode::<AddressBook>(&book_bytes_minicbor).is_ok());
}

#[test]
fn serde_encode() {
    let book = gen_addressbook(16);

    let mut buf = Vec::new();

    println!();
    bench("encode/serde_cbor", || {
        buf.clear();
        book.serialize(&mut serde_cbor::Serializer::new(&mut buf)).is_ok()
    });

    bench("encode/cbor4ii", || {
        buf.clear();
        cbor4ii::serde::to_writer(&mut buf, &book).is_ok()
    });

    bench("encode/minicbor-serde", || {
        buf.clear();
        book.serialize(&mut minicbor_serde::Serializer::new(&mut buf)).is_ok()
    });
}

#[test]
fn serde_decode() {
    let sample = Path::new("/tmp/sample.cbor");
    if !sample.is_file() {
        let book = gen_addressbook(16);
        let cbor = serde_cbor::ser::to_vec(&book).unwrap();
        std::fs::write(sample, &cbor).unwrap();
    }
    let sample = std::fs::read(sample).unwrap();

    println!();
    bench("decode/serde_cbor",     || serde_cbor::from_slice::<AddressBook>(&sample).is_ok());
    bench("decode/cbor4ii",        || cbor4ii::serde::from_slice::<AddressBook>(&sample).is_ok());
    bench("decode/minicbor-serde", || minicbor_serde::from_slice::<AddressBook>(&sample).is_ok());
}

fn gen_addressbook(n: usize) -> AddressBook<'static> {
    fn gen_string(g: &mut ThreadRng) -> Cow<'static, str> {
        Cow::Owned(iter::repeat_with(|| char::from(g.sample(Alphanumeric))).take(128).collect())
    }

    fn gen_address(g: &mut ThreadRng) -> Address<'static> {
        Address {
            street: gen_string(g),
            houseno: gen_string(g),
            postcode: g.gen(),
            city: gen_string(g),
            country: gen_string(g)
        }
    }

    fn gen_style(g: &mut ThreadRng) -> Option<Style<'static>> {
        let s = match g.gen_range(0 .. 5) {
            0 => return None,
            1 => Style::Version1,
            2 => Style::Version2,
            3 => Style::Version3(g.gen(), g.gen()),
            4 => Style::Version4 { path: gen_string(g), timestamp: g.gen() },
            _ => unreachable!()
        };
        Some(s)
    }

    fn gen_entry(g: &mut ThreadRng, n: usize) -> Entry<'static> {
        Entry {
            firstname: gen_string(g),
            lastname: gen_string(g),
            birthday: g.gen(),
            addresses: {
                let mut v = Vec::with_capacity(n);
                for _ in 0 .. n {
                    v.push(gen_address(g))
                }
                v
            }
        }
    }

    let mut g = rand::thread_rng();

    AddressBook {
        timestamp: g.gen(),
        entries: {
            let mut v = Vec::with_capacity(n);
            for _ in 0 .. n {
                v.push(gen_entry(&mut g, n))
            }
            v
        },
        style: gen_style(&mut g),
        rating: if g.gen() {
            Some(g.gen_range(-2342.42342 .. 234423.2342))
        } else {
            None
        }
    }
}
