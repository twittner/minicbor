//!  Module providing types to support custom tags in CBOR
use core::ops::{Deref, DerefMut};
use minicbor::data::Tag;
use serde::{Deserialize, Serialize, de};

/// Enum alias for [`TagContainer`]
pub(crate) const TAG_CONTAINER_IDENTIFIER: &str = "$#minicbor_serde_tag_container#$";

/// Variant alias for [`TagContainer::Tag`]
pub(crate) const TAG_IDENTIFIER: &str = "$#minicbor_serde_tag#$";

/// Variant alias for [`TagContainer::NoTag`]
pub(crate) const NO_TAG_IDENTIFIER: &str = "$#minicbor_serde_no_tag#$";

/// Enum used to guide [`crate::de::Deserializer`] to deserialize a tagged
/// value.
///
/// [`de::Deserialize`] implementation for this enum renames the fields to
/// custom aliases to ensure that [`crate::de::Deserializer`] is able to drive
/// the [`minicbor::Decoder`] correctly.
///
/// /// [`ser::Serializer`] implementation for this enum renames the fields to
/// custom aliases to ensure that [`crate::de::Deserializer`] is able to drive
/// the [`minicbor::Decoder`] correctly.
enum TagContainer<T> {
    Tag(Tag, T),
    NoTag(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for TagContainer<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_enum(
            TAG_CONTAINER_IDENTIFIER,
            &[TAG_IDENTIFIER, NO_TAG_IDENTIFIER],
            TagContainerVisitor(core::marker::PhantomData),
        )
    }
}

struct TagContainerVisitor<T>(core::marker::PhantomData<T>);

impl<'de, T: Deserialize<'de>> de::Visitor<'de> for TagContainerVisitor<T> {
    type Value = TagContainer<T>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "enum TagContainer")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        use de::VariantAccess;
        let (variant, access) = data.variant::<&str>()?;
        match variant {
            TAG_IDENTIFIER => {
                let (tag, val) =
                    access.tuple_variant(2, TupleVisitor(core::marker::PhantomData))?;
                Ok(TagContainer::Tag(Tag::new(tag), val))
            }
            NO_TAG_IDENTIFIER => {
                let val = access.newtype_variant()?;
                Ok(TagContainer::NoTag(val))
            }
            _ => Err(de::Error::unknown_variant(
                variant,
                &[TAG_IDENTIFIER, NO_TAG_IDENTIFIER],
            )),
        }
    }
}

struct TupleVisitor<T>(core::marker::PhantomData<T>);

impl<'de, T: Deserialize<'de>> de::Visitor<'de> for TupleVisitor<T> {
    type Value = (u64, T);

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "tuple (u64, T)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let tag = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let val = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok((tag, val))
    }
}

/// Helper struct for formatting tag mismatch errors
struct TagMismatchError {
    found: Tag,
    expected: Tag,
}

impl TagMismatchError {
    fn new<A: Into<Tag>, B: Into<Tag>>(found: A, expected: B) -> Self {
        Self {
            found: found.into(),
            expected: expected.into(),
        }
    }
}

impl core::fmt::Display for TagMismatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "unexpected CBOR tag {}, expected {}",
            self.found, self.expected
        )
    }
}

/// Helper struct for formatting expected tag errors
struct ExpectedTagError {
    expected: Tag,
}

impl ExpectedTagError {
    fn new<A: Into<Tag>>(expected: A) -> Self {
        Self {
            expected: expected.into(),
        }
    }
}

impl core::fmt::Display for ExpectedTagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "expected CBOR tag {}", self.expected)
    }
}

impl<T: Serialize> Serialize for TagContainer<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTupleVariant;
        match self {
            TagContainer::Tag(tag, val) => {
                let mut tv = serializer.serialize_tuple_variant(
                    TAG_CONTAINER_IDENTIFIER,
                    0,
                    TAG_IDENTIFIER,
                    2,
                )?;
                tv.serialize_field(&tag.as_u64())?;
                tv.serialize_field(val)?;
                tv.end()
            }
            TagContainer::NoTag(val) => serializer.serialize_newtype_variant(
                TAG_CONTAINER_IDENTIFIER,
                1,
                NO_TAG_IDENTIFIER,
                val,
            ),
        }
    }
}

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

impl<'de, const TAG: u64, T: Deserialize<'de>> Deserialize<'de> for Required<TAG, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let container = TagContainer::deserialize(deserializer)?;
        match container {
            TagContainer::Tag(tag, val) if tag == Self::EXPECTED_TAG => Ok(Required(val)),
            TagContainer::Tag(tag, _) => Err(de::Error::custom(TagMismatchError::new(
                tag,
                Self::EXPECTED_TAG,
            ))),
            _ => Err(de::Error::custom(ExpectedTagError::new(Self::EXPECTED_TAG))),
        }
    }
}

impl<const TAG: u64, T: Serialize> Serialize for Required<TAG, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        TagContainer::Tag(Self::EXPECTED_TAG, &self.0).serialize(serializer)
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

impl<'de, const TAG: u64, T: Deserialize<'de>> Deserialize<'de> for Optional<TAG, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let container = TagContainer::deserialize(deserializer)?;
        match container {
            TagContainer::Tag(tag, val) if tag == Self::EXPECTED_TAG => {
                Ok(Optional(Some(tag), val))
            }
            TagContainer::NoTag(val) => Ok(Optional(None, val)),
            TagContainer::Tag(tag, _) => Err(de::Error::custom(TagMismatchError::new(
                tag,
                Self::EXPECTED_TAG,
            ))),
        }
    }
}

impl<const TAG: u64, T: Serialize> Serialize for Optional<TAG, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Some(tag) => TagContainer::Tag(tag, &self.1).serialize(serializer),
            None => TagContainer::NoTag(&self.1).serialize(serializer),
        }
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

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Any<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let container = TagContainer::deserialize(deserializer)?;
        match container {
            TagContainer::Tag(tag, val) => Ok(Any(Some(tag), val)),
            TagContainer::NoTag(val) => Ok(Any(None, val)),
        }
    }
}

impl<T: Serialize> Serialize for Any<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Some(tag) => TagContainer::Tag(tag, &self.1).serialize(serializer),
            None => TagContainer::NoTag(&self.1).serialize(serializer),
        }
    }
}
