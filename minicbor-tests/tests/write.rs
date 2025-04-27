#![cfg(feature = "alloc")]

use minicbor::encode::write::{Cursor, Write};
use quickcheck::quickcheck;
use std::num::NonZeroUsize;

quickcheck! {
    fn cursor_slice(input: Vec<u8>, chunk_len: NonZeroUsize) -> bool {
        let mut o = vec![0; input.len()];
        let mut c = Cursor::new(&mut o[..]);
        for chunk in input.chunks(chunk_len.get()) {
            c.write_all(chunk).unwrap();
        }
        &c.get_ref()[.. c.position()] == &input
    }

    fn cursor_array(input: Vec<u8>, chunk_len: NonZeroUsize) -> bool {
        let mut c = Cursor::new([0; 128]);
        for chunk in input.chunks(chunk_len.get()) {
            c.write_all(chunk).unwrap();
        }
        &c.get_ref()[.. c.position()] == &input
    }

    fn cursor_box(input: Vec<u8>, chunk_len: NonZeroUsize) -> bool {
        let mut c = Cursor::new(vec![0u8; input.len()].into_boxed_slice());
        for chunk in input.chunks(chunk_len.get()) {
            c.write_all(chunk).unwrap();
        }
        &c.get_ref()[.. c.position()] == &input
    }
}
