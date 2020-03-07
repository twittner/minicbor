//! Information about CBOR data types and tags.

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
    String,
    Array,
    Map,
    Tag,
    Break,
    Unknown(u8)
}

impl Type {
    pub(crate) fn read(n: u8) -> Self {
        match n {
            0x00 ..= 0x18        => Type::U8,
            0x19                 => Type::U16,
            0x1a                 => Type::U32,
            0x1b                 => Type::U64,
            0x20 ..= 0x38        => Type::I8,
            0x39                 => Type::I16,
            0x3a                 => Type::I32,
            0x3b                 => Type::I64,
            0x40 ..= 0x5b | 0x5f => Type::Bytes,
            0x60 ..= 0x7b | 0x7f => Type::String,
            0x80 ..= 0x9b | 0x9f => Type::Array,
            0xa0 ..= 0xbb | 0xbf => Type::Map,
            0xc9 ..= 0xdb        => Type::Tag,
            0xe0 ..= 0xf3 | 0xf8 => Type::Simple,
            0xf4 | 0xf5          => Type::Bool,
            0xf6                 => Type::Null,
            0xf7                 => Type::Undefined,
            0xf9                 => Type::F16,
            0xfa                 => Type::F32,
            0xfb                 => Type::F64,
            0xff                 => Type::Break,
            _                    => Type::Unknown(n)
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

    pub(crate) fn into(self) -> u64 {
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

