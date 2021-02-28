//! Generic CBOR tokenisation.

use core::fmt;
use crate::Decoder;
use crate::data::{Tag, Type};
use crate::decode::Error;

/// Representation of possible CBOR tokens.
///
/// *Requires feature* `"half"`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'b> {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F16(f32),
    F32(f32),
    F64(f64),
    Bytes(&'b [u8]),
    String(&'b str),
    Array(u64),
    Map(u64),
    Tag(Tag),
    Simple(u8),
    Break,
    Null,
    Undefined,
    /// Start of indefinite byte string.
    BeginBytes,
    /// Start of indefinite text string.
    BeginString,
    /// Start of indefinite array.
    BeginArray,
    /// Start of indefinite map.
    BeginMap
}

/// An [`Iterator`] over CBOR tokens.
///
/// The `Iterator` implementation calls [`Tokenizer::token`] until
/// [`Error::EndOfInput`] is returned which is mapped to `None`.
///
/// *Requires feature* `"half"`.
#[derive(Debug)]
pub struct Tokenizer<'b> {
    decoder: Decoder<'b>
}

impl<'b> Iterator for Tokenizer<'b> {
    type Item = Result<Token<'b>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.token() {
            Ok(t) => Some(Ok(t)),
            Err(Error::EndOfInput) => None,
            Err(e) => Some(Err(e))
        }
    }
}

impl<'b> From<Decoder<'b>> for Tokenizer<'b> {
    fn from(d: Decoder<'b>) -> Self {
        Tokenizer { decoder: d }
    }
}

impl<'b> Tokenizer<'b> {
    /// Create a new Tokenizer for the given input bytes.
    pub fn new(bytes: &'b[u8]) -> Self {
        Tokenizer { decoder: Decoder::new(bytes) }
    }

    /// Decode the next token.
    ///
    /// Note that a sequence of tokens may not necessarily represent
    /// well-formed CBOR items.
    pub fn token(&mut self) -> Result<Token<'b>, Error> {
        match self.decoder.datatype()? {
            Type::Bool        => self.decoder.bool().map(Token::Bool),
            Type::U8          => self.decoder.u8().map(Token::U8),
            Type::U16         => self.decoder.u16().map(Token::U16),
            Type::U32         => self.decoder.u32().map(Token::U32),
            Type::U64         => self.decoder.u64().map(Token::U64),
            Type::I8          => self.decoder.i8().map(Token::I8),
            Type::I16         => self.decoder.i16().map(Token::I16),
            Type::I32         => self.decoder.i32().map(Token::I32),
            Type::I64         => self.decoder.i64().map(Token::I64),
            Type::F16         => self.decoder.f16().map(Token::F16),
            Type::F32         => self.decoder.f32().map(Token::F32),
            Type::F64         => self.decoder.f64().map(Token::F64),
            Type::Bytes       => self.decoder.bytes().map(Token::Bytes),
            Type::String      => self.decoder.str().map(Token::String),
            Type::Tag         => self.decoder.tag().map(Token::Tag),
            Type::Simple      => self.decoder.simple().map(Token::Simple),
            Type::Array       => self.decoder.array().map(|n| Token::Array(n.expect("array len"))),
            Type::Map         => self.decoder.map().map(|n| Token::Map(n.expect("map len"))),
            Type::BytesIndef  => { self.skip_byte(); Ok(Token::BeginBytes)  }
            Type::StringIndef => { self.skip_byte(); Ok(Token::BeginString) }
            Type::ArrayIndef  => { self.skip_byte(); Ok(Token::BeginArray)  }
            Type::MapIndef    => { self.skip_byte(); Ok(Token::BeginMap)    }
            Type::Null        => { self.skip_byte(); Ok(Token::Null)        }
            Type::Undefined   => { self.skip_byte(); Ok(Token::Undefined)   }
            Type::Break       => { self.skip_byte(); Ok(Token::Break)       }
            Type::Unknown(n)  => Err(Error::TypeMismatch(n, "unknown cbor type"))
        }
    }

    fn skip_byte(&mut self) {
        self.decoder.set_position(self.decoder.position() + 1)
    }
}

/// Pretty print a token.
///
/// The following syntax is used:
///
/// - Numeric values and booleans are displayed as literals, e.g.
///   `Token::U16(12)` becomes `12`.
/// - Text strings are displayed in double quotes.
/// - Byte strings are displayed in angle brackets and each byte is
///   base-16 encoded, e.g. `<01 02 ef>`.
/// - An array is displayed as `A[n]` where `n` denotes the number of elements.
///   The following `n` tokens are elements of this array.
/// - A map is displayed as `M[n]` where `n` denotes the number of pairs.
///   The following `n` tokens are entries of this map.
/// - Tags are displayed with `T(t)` where `t` is the [`Tag`].
/// - Simple values are displayed as `#n` where `n` denotes the numeric
///   value.
/// - Indefinite items start with:
///     * `?B[` for byte strings,
///     * `?S[` for text strings,
///     * `?A[` for arrays,
///     * `?M[` for maps,
///   and end with `]` when a `Token::Break` is encountered. All tokens
///   in between belong to the indefinite container.
/// - `Token::Null` is displayed as `NULL` and `Token::Undefined` as `UNDEFINED`.
impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Bool(b)     => write!(f, "{}", b),
            Token::U8(n)       => write!(f, "{}", n),
            Token::U16(n)      => write!(f, "{}", n),
            Token::U32(n)      => write!(f, "{}", n),
            Token::U64(n)      => write!(f, "{}", n),
            Token::I8(n)       => write!(f, "{}", n),
            Token::I16(n)      => write!(f, "{}", n),
            Token::I32(n)      => write!(f, "{}", n),
            Token::I64(n)      => write!(f, "{}", n),
            Token::F16(n)      => write!(f, "{}", n),
            Token::F32(n)      => write!(f, "{}", n),
            Token::F64(n)      => write!(f, "{}", n),
            Token::String(n)   => write!(f, "\"{}\"", n),
            Token::Array(n)    => write!(f, "A[{}]", n),
            Token::Map(n)      => write!(f, "M[{}]", n),
            Token::Tag(t)      => write!(f, "T({:?})", t),
            Token::Simple(n)   => write!(f, "#{}", n),
            Token::Break       => f.write_str("]"),
            Token::Null        => f.write_str("NULL"),
            Token::Undefined   => f.write_str("UNDEFINED"),
            Token::BeginBytes  => f.write_str("?B["),
            Token::BeginString => f.write_str("?S["),
            Token::BeginArray  => f.write_str("?A["),
            Token::BeginMap    => f.write_str("?M["),
            Token::Bytes(b)    => {
                f.write_str("<")?;
                let mut i = b.len();
                for x in *b {
                    if i > 1 {
                        write!(f, "{:02x} ", x)?
                    } else {
                        write!(f, "{:02x}", x)?
                    }
                    i -= 1;
                }
                f.write_str(">")
            }
        }
    }
}

