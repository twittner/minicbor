//! Internal module that other crates should not rely on.

/// Wrap a default value in `Option::Some`.
pub fn some_default<T: Default>() -> Option<T> {
    Some(Default::default())
}
