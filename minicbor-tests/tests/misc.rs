use minicbor::decode::{Decoder, Error};

#[test]
fn trigger_length_overflow_str() {
    let input = b"\x7B\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF";
    let mut d = Decoder::new(&input[..]);
    assert!(matches!(d.str(), Err(Error::EndOfInput)))
}

#[test]
fn trigger_length_overflow_bytes() {
    let input = b"\x5B\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF";
    let mut d = Decoder::new(&input[..]);
    assert!(matches!(d.bytes(), Err(Error::EndOfInput)))
}

