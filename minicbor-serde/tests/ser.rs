#![cfg(feature = "alloc")]

#[test]
fn to_vec_and_to_slice_are_equivalent() {
    let val = "Hello, world.";

    let mut b = [0u8; 256];
    let n = minicbor_serde::to_slice(&val, &mut b).unwrap();

    let v = minicbor_serde::to_vec(&val).unwrap();

    assert_eq!(v.as_slice(), &b[..n]);
}
