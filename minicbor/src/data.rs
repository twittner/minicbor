//! Information about CBOR data types and tags.

use core::fmt;

/// CBOR data types.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Type {
    Bool,
    Null,
    Undefined,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    F32,
    F64,
    Simple,
    Bytes,
    BytesIndef,
    String,
    StringIndef,
    Array,
    ArrayIndef,
    Map,
    MapIndef,
    Tag,
    Break,
    Unknown(u8)
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Bool        => f.write_str("bool"),
            Type::Null        => f.write_str("null"),
            Type::Undefined   => f.write_str("undefined"),
            Type::U8          => f.write_str("u8"),
            Type::U16         => f.write_str("u16"),
            Type::U32         => f.write_str("u32"),
            Type::U64         => f.write_str("u64"),
            Type::I8          => f.write_str("i8"),
            Type::I16         => f.write_str("i16"),
            Type::I32         => f.write_str("i32"),
            Type::I64         => f.write_str("i64"),
            Type::F16         => f.write_str("f16"),
            Type::F32         => f.write_str("f32"),
            Type::F64         => f.write_str("f64"),
            Type::Simple      => f.write_str("simple"),
            Type::Bytes       => f.write_str("bytes"),
            Type::BytesIndef  => f.write_str("indefinite bytes"),
            Type::String      => f.write_str("string"),
            Type::StringIndef => f.write_str("indefinite string"),
            Type::Array       => f.write_str("array"),
            Type::ArrayIndef  => f.write_str("indefinite array"),
            Type::Map         => f.write_str("map"),
            Type::MapIndef    => f.write_str("indefinite map"),
            Type::Tag         => f.write_str("tag"),
            Type::Break       => f.write_str("break"),
            Type::Unknown(n)  => write!(f, "{:#x}", n)
        }
    }
}

/// CBOR data item tag.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Tag {
    DateTime,
    Timestamp,
    PosBignum,
    NegBignum,
    Decimal,
    Bigfloat,
    ToBase64Url,
    ToBase64,
    ToBase16,
    Cbor,
    Uri,
    Base64Url,
    Base64,
    Regex,
    Mime,
    Unassigned(u64)
}

impl Tag {
    pub(crate) fn from(n: u64) -> Self {
        match n {
            0x00 => Tag::DateTime,
            0x01 => Tag::Timestamp,
            0x02 => Tag::PosBignum,
            0x03 => Tag::NegBignum,
            0x04 => Tag::Decimal,
            0x05 => Tag::Bigfloat,
            0x15 => Tag::ToBase64Url,
            0x16 => Tag::ToBase64,
            0x17 => Tag::ToBase16,
            0x18 => Tag::Cbor,
            0x20 => Tag::Uri,
            0x21 => Tag::Base64Url,
            0x22 => Tag::Base64,
            0x23 => Tag::Regex,
            0x24 => Tag::Mime,
            _    => Tag::Unassigned(n)
        }
    }

    pub(crate) fn numeric(self) -> u64 {
        match self {
            Tag::DateTime      => 0x00,
            Tag::Timestamp     => 0x01,
            Tag::PosBignum     => 0x02,
            Tag::NegBignum     => 0x03,
            Tag::Decimal       => 0x04,
            Tag::Bigfloat      => 0x05,
            Tag::ToBase64Url   => 0x15,
            Tag::ToBase64      => 0x16,
            Tag::ToBase16      => 0x17,
            Tag::Cbor          => 0x18,
            Tag::Uri           => 0x20,
            Tag::Base64Url     => 0x21,
            Tag::Base64        => 0x22,
            Tag::Regex         => 0x23,
            Tag::Mime          => 0x24,
            Tag::Unassigned(n) => n
        }
    }
}

/// The neutral element w.r.t. [`Encode`](crate::Encode) and [`Decode`](crate::Decode).
///
/// This newtype merely wraps a byte slice which is assumed to be a CBOR value
/// and implements `Encode` and `Decode` as no-ops, i.e. the `Encode` impl
/// writes the byte slice as is and the `Decode` impl returns the byte slice
/// as is.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cbor<'b>(&'b [u8]);

impl<'b> From<&'b [u8]> for Cbor<'b> {
    fn from(cbor: &'b [u8]) -> Self {
        Cbor(cbor)
    }
}

impl<'b> From<Cbor<'b>> for &'b [u8] {
    fn from(cbor: Cbor<'b>) -> Self {
        cbor.0
    }
}

impl AsRef<[u8]> for Cbor<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl core::ops::Deref for Cbor<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl crate::Encode for Cbor<'_> {
    fn encode<W>(&self, e: &mut crate::Encoder<W>) -> Result<(), crate::encode::Error<W::Error>>
    where
        W: crate::encode::Write
    {
        e.put(self.0)?.ok()
    }
}

impl<'b> crate::Decode<'b> for Cbor<'b> {
    fn decode(d: &mut crate::Decoder<'b>) -> Result<Self, crate::decode::Error> {
        d.consume().map(Cbor)
    }
}

#[cfg(all(feature = "alloc", feature = "half"))]
impl core::fmt::Display for Cbor<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::display(self.0).fmt(f)
    }
}

