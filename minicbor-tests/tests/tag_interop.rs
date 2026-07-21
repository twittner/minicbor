use minicbor::data::{Tag, Tagged};
use minicbor_serde::tag::{Any, Optional, Required};

const TEST_TAG: u64 = 42;
const TEST_TAG2: u64 = 20;

#[test]
fn minicbor_to_serde_required() {
    let value = Tagged::<TEST_TAG, u32>::new(123);
    let bytes = minicbor::to_vec(&value).unwrap();
    let decoded: Required<TEST_TAG, u32> = minicbor_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.into_value(), 123);
}

#[test]
fn minicbor_to_serde_optional_with_tag() {
    let value = Tagged::<TEST_TAG, String>::new("hello".to_string());
    let bytes = minicbor::to_vec(&value).unwrap();
    let decoded: Optional<TEST_TAG, String> = minicbor_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.tag().map(Tag::as_u64), Some(TEST_TAG));
    assert_eq!(decoded.into_value(), "hello");
}

#[test]
fn minicbor_to_serde_any_with_tag() {
    let value = Tagged::<TEST_TAG, bool>::new(true);
    let bytes = minicbor::to_vec(&value).unwrap();
    let decoded: Any<bool> = minicbor_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.tag().map(Tag::as_u64), Some(TEST_TAG));
    assert_eq!(decoded.into_value(), true);
}

#[test]
fn minicbor_untagged_to_serde_optional() {
    let value = 456u32;
    let bytes = minicbor::to_vec(&value).unwrap();
    let decoded: Optional<TEST_TAG, u32> = minicbor_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.tag().map(Tag::as_u64), None);
    assert_eq!(decoded.into_value(), 456);
}

#[test]
fn minicbor_untagged_to_serde_any() {
    let value = "test".to_string();
    let bytes = minicbor::to_vec(&value).unwrap();
    let decoded: Any<String> = minicbor_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.tag().map(Tag::as_u64), None);
    assert_eq!(decoded.into_value(), "test");
}

#[test]
fn minicbor_wrong_tag_to_serde_required_fails() {
    let value = Tagged::<TEST_TAG, u32>::new(42);
    let bytes = minicbor::to_vec(&value).unwrap();
    let result = minicbor_serde::from_slice::<Required<TEST_TAG2, u32>>(&bytes);
    assert!(result.is_err());
}

#[test]
fn serde_required_to_minicbor() {
    let value = Required::<TEST_TAG, u32>::new(123);
    let bytes = minicbor_serde::to_vec(&value).unwrap();
    let decoded: Tagged<TEST_TAG, u32> = minicbor::decode(&bytes).unwrap();
    assert_eq!(decoded.tag().as_u64(), TEST_TAG);
    assert_eq!(*decoded.value(), 123);
}

#[test]
fn serde_optional_with_tag_to_minicbor() {
    let value = Optional::<TEST_TAG, String>::tagged("hello".to_string());
    let bytes = minicbor_serde::to_vec(&value).unwrap();
    let decoded: Tagged<TEST_TAG, String> = minicbor::decode(&bytes).unwrap();
    assert_eq!(decoded.tag().as_u64(), TEST_TAG);
    assert_eq!(*decoded.value(), "hello");
}

#[test]
fn serde_optional_without_tag_to_minicbor() {
    let value = Optional::<TEST_TAG, u32>::untagged(456);
    let bytes = minicbor_serde::to_vec(&value).unwrap();
    let decoded: u32 = minicbor::decode(&bytes).unwrap();
    assert_eq!(decoded, 456);
}

#[test]
fn serde_any_with_tag_to_minicbor() {
    let value = Any::tagged(Tag::new(TEST_TAG), true);
    let bytes = minicbor_serde::to_vec(&value).unwrap();
    let decoded: Tagged<TEST_TAG, bool> = minicbor::decode(&bytes).unwrap();
    assert_eq!(decoded.tag().as_u64(), TEST_TAG);
    assert_eq!(*decoded.value(), true);
}

#[test]
fn serde_any_without_tag_to_minicbor() {
    let value = Any::untagged("test".to_string());
    let bytes = minicbor_serde::to_vec(&value).unwrap();
    let decoded: String = minicbor::decode(&bytes).unwrap();
    assert_eq!(decoded, "test");
}
