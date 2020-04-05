use criterion::{criterion_group, criterion_main, Criterion};
use minicbor::{Encode, Decode};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Serialize, Deserialize};
use std::borrow::Cow;

criterion_group!(benches, benchmark);
criterion_main!(benches);

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct AddressBook<'a> {
    #[n(1)] timestamp: u64,
    #[n(2)] entries: Vec<Entry<'a>>,
    #[n(3)] style: Option<Style<'a>>,
    #[n(4)] rating: Option<f64>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct Entry<'a> {
    #[n(1)] firstname: Cow<'a, str>,
    #[n(2)] lastname: Cow<'a, str>,
    #[n(3)] birthday: u32,
    #[n(4)] addresses: Vec<Address<'a>>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
struct Address<'a> {
    #[n(1)] street: Cow<'a, str>,
    #[n(2)] houseno: Cow<'a, str>,
    #[n(3)] postcode: u32,
    #[n(4)] city: Cow<'a, str>,
    #[n(5)] country: Cow<'a, str>
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
enum Style<'a> {
    #[n(1)] Version1,
    #[n(2)] Version2,
    #[n(3)] Version3(#[n(1)] bool, #[n(2)] u64),
    #[n(4)] Version4 {
        #[n(1)] path: Cow<'a, str>,
        #[n(2)] timestamp: u64
    }
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    let book = gen_addressbook(8);
    group.bench_function("serde_cbor", |b| b.iter(|| {
        serde_cbor::ser::to_vec_packed(&book).unwrap();
    }));
    group.bench_function("minicbor", |b| b.iter(|| {
        minicbor::to_vec(&book).unwrap();
    }));
    group.finish();

    let mut group = c.benchmark_group("decode");
    let book = gen_addressbook(8);
    let book_bytes_serde = serde_cbor::ser::to_vec_packed(&book).unwrap();
    let book_bytes_minicbor = minicbor::to_vec(&book).unwrap();
    group.bench_function("serde_cbor", |b| b.iter(|| {
        let _: AddressBook = serde_cbor::from_slice(&book_bytes_serde).unwrap();

    }));
    group.bench_function("minicbor", |b| b.iter(|| {
        let _: AddressBook = minicbor::decode(&book_bytes_minicbor).unwrap();
    }));
    group.finish();
}

fn gen_addressbook(n: usize) -> AddressBook<'static> {
    fn gen_string(g: &mut ThreadRng) -> Cow<'static, str> {
        Cow::Owned(Alphanumeric.sample_iter(g).take(128).collect())
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
        let s = match g.gen_range(0, 5) {
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
            Some(g.gen_range(-2342.42342, 234423.2342))
        } else {
            None
        }
    }
}
