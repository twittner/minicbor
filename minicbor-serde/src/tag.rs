//!  Module providing types to support custom tags in CBOR
use core::ops::{Deref, DerefMut};
use minicbor::data::Tag;

/// Requires unique tag to be present during deserialization
///
/// Tags will always be emitted during serialization
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Required<const TAG: u64, T>(T);

impl<const TAG: u64, T> Required<TAG, T> {
    const EXPECTED_TAG: Tag = Tag::new(TAG);

    /// Creates a new tagged [`Required`]
    pub const fn new(val: T) -> Self {
        Self(val)
    }

    /// Returns the associated tag
    pub const fn tag(&self) -> Tag {
        Self::EXPECTED_TAG
    }

    /// Returns the inner value, while consuming self
    pub fn into_value(self) -> T {
        self.0
    }
}

impl<const TAG: u64, T> Deref for Required<TAG, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const TAG: u64, T> DerefMut for Required<TAG, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Accepts an unique tag during deserialization, if present
///
/// Tag will be emitted during serialization, if present
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Optional<const TAG: u64, T>(Option<Tag>, T);

impl<const TAG: u64, T> Optional<TAG, T> {
    const EXPECTED_TAG: Tag = Tag::new(TAG);

    /// Creates a new tagged [`Optional`]
    pub const fn tagged(val: T) -> Self {
        Self(Some(Self::EXPECTED_TAG), val)
    }

    /// Creates a new untagged [`Optional`]
    pub const fn untagged(val: T) -> Self {
        Self(None, val)
    }

    /// Returns the associated tag (if any)
    pub const fn tag(&self) -> Option<Tag> {
        self.0
    }

    /// Returns the inner value, while consuming self
    pub fn into_value(self) -> T {
        self.1
    }
}

impl<const TAG: u64, T> Deref for Optional<TAG, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<const TAG: u64, T> DerefMut for Optional<TAG, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

/// Accepts any tag during deserialization, if present
///
/// Tag will be emitted during serialization, if present
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Any<T>(Option<Tag>, T);

impl<T> Any<T> {
    /// Creates a new tagged [`Any`]
    pub fn tagged<N: Into<Tag>>(tag: N, val: T) -> Self {
        Self(Some(tag.into()), val)
    }

    /// Creates a new untagged [`Any`]
    pub const fn untagged(val: T) -> Self {
        Self(None, val)
    }

    /// Returns the associated tag (if any)
    pub const fn tag(&self) -> Option<Tag> {
        self.0
    }

    /// Returns the inner value, while consuming self
    pub fn into_value(self) -> T {
        self.1
    }
}

impl<T> Deref for Any<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T> DerefMut for Any<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}
