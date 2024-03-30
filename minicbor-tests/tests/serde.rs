#![cfg(feature = "serde")]

use serde::{Serialize, Deserialize};
use minicbor::serde::{Serializer, Deserializer};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum E {
    Unit,
    XXX,
    Tuple1(bool),
    Tuple2(bool, char),
    Struct{ b: bool, c: char}
}

#[test]
fn with_variant_names() {
    let mut s = Serializer::new(Vec::new());

    E::Unit.serialize(&mut s).unwrap();
    let mut d = Deserializer::from_slice(s.encoder().writer());
    assert_eq!(E::deserialize(&mut d).unwrap(), E::Unit);
    s.encoder_mut().writer_mut().clear();

    E::Tuple1(true).serialize(&mut s).unwrap();
    let mut d = Deserializer::from_slice(s.encoder().writer());
    assert_eq!(E::deserialize(&mut d).unwrap(), E::Tuple1(true));
    s.encoder_mut().writer_mut().clear();

    E::Tuple2(true, 'x').serialize(&mut s).unwrap();
    let mut d = Deserializer::from_slice(s.encoder().writer());
    assert_eq!(E::deserialize(&mut d).unwrap(), E::Tuple2(true, 'x'));
    s.encoder_mut().writer_mut().clear();

    E::Struct { b: true, c: 'x' }.serialize(&mut s).unwrap();
    let mut d = Deserializer::from_slice(s.encoder().writer());
    assert_eq!(E::deserialize(&mut d).unwrap(), E::Struct { b: true, c: 'x' });
    s.encoder_mut().writer_mut().clear();
}

