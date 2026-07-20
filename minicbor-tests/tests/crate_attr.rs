//! Tests for `#[cbor(crate = "...")]`.
//!
//! The private module `minicbor` below shadows the real crate with a dummy
//! type, so any generated code that reaches for the implicit `minicbor` path
//! would fail to compile. Derives in this file therefore have to rely on the
//! crate path supplied via `#[cbor(crate = "...")]`.

mod minicbor {
    #[allow(non_camel_case_types, dead_code)]
    pub struct DO_NOT_USE_THIS;
}

use ::minicbor as renamed_minicbor;

#[derive(renamed_minicbor::Encode, renamed_minicbor::Decode, renamed_minicbor::CborLen)]
#[cbor(crate = "renamed_minicbor")]
struct ReexportStruct {
    #[n(0)] a: u32,
    #[n(1)] b: Option<bool>,
}

#[derive(renamed_minicbor::Encode, renamed_minicbor::Decode, renamed_minicbor::CborLen)]
#[cbor(crate = "renamed_minicbor", map)]
enum ReexportEnum {
    #[n(0)] Unit,
    #[n(1)] Tuple(#[n(0)] u32),
    #[n(2)] Struct {
        #[n(0)] x: u32,
        #[cbor(n(1), skip_if = "Option::is_none")] y: Option<bool>,
    },
}

#[test]
fn roundtrip_struct() {
    let s = ReexportStruct { a: 7, b: Some(true) };
    let bytes = renamed_minicbor::to_vec(&s).unwrap();
    let back: ReexportStruct = renamed_minicbor::decode(&bytes).unwrap();
    assert_eq!(back.a, 7);
    assert_eq!(back.b, Some(true));
    assert_eq!(bytes.len(), renamed_minicbor::len(&s));
}

#[test]
fn roundtrip_enum() {
    for variant in [
        ReexportEnum::Unit,
        ReexportEnum::Tuple(42),
        ReexportEnum::Struct { x: 1, y: None },
        ReexportEnum::Struct { x: 1, y: Some(false) },
    ] {
        let bytes = renamed_minicbor::to_vec(&variant).unwrap();
        let back: ReexportEnum = renamed_minicbor::decode(&bytes).unwrap();
        assert!(matches!(
            (variant, back),
            (ReexportEnum::Unit, ReexportEnum::Unit)
                | (ReexportEnum::Tuple(_), ReexportEnum::Tuple(_))
                | (ReexportEnum::Struct { .. }, ReexportEnum::Struct { .. })
        ));
    }
}
