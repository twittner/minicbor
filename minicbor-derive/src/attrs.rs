//! Attribute handling.

pub mod codec;
pub mod encoding;
pub mod idx;

use std::collections::HashMap;
use std::fmt;
use syn::spanned::Spanned;

pub use codec::CustomCodec;
pub use encoding::Encoding;
pub use idx::Idx;

/// Recognised attributes.
#[derive(Debug, Clone)]
pub struct Attributes(Level, HashMap<Kind, Value>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Kind {
    Codec,
    Encoding,
    Index,
    IndexOnly,
    Transparent
}

#[derive(Debug, Clone)]
enum Value {
    Codec(CustomCodec, proc_macro2::Span),
    Encoding(Encoding, proc_macro2::Span),
    Index(Idx, proc_macro2::Span),
    Span(proc_macro2::Span)
}

#[derive(Debug, Copy, Clone)]
pub enum Level {
    Enum,
    Struct,
    Variant,
    Field
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Enum    => f.write_str("enum"),
            Level::Struct  => f.write_str("struct"),
            Level::Variant => f.write_str("variant"),
            Level::Field   => f.write_str("field")
        }
    }
}

impl Attributes {
    pub fn new(l: Level) -> Self {
        Attributes(l, HashMap::new())
    }

    pub fn try_from_iter<'a, I>(l: Level, attrs: I) -> syn::Result<Self>
    where
        I: IntoIterator<Item = &'a syn::Attribute>
    {
        let mut this = Attributes::new(l);
        for m in attrs.into_iter().map(|a| Attributes::try_from(l, a)) {
            let m = m?;
            for (k, v) in m.1.into_iter() {
                this.try_insert(k, v)?;
            }
        }
        Ok(this)
    }

    fn try_from(l: Level, a: &syn::Attribute) -> syn::Result<Attributes> {
        let mut attrs = Attributes::new(l);

        // #[n(...)]
        if a.path.is_ident("n") {
            let idx = parse_u32_arg(a).map(Idx::N)?;
            attrs.try_insert(Kind::Index, Value::Index(idx, a.tokens.span()))?;
            return Ok(attrs)
        }

        // #[b(...)]
        if a.path.is_ident("b") {
            let idx = parse_u32_arg(a).map(Idx::B)?;
            attrs.try_insert(Kind::Index, Value::Index(idx, a.tokens.span()))?;
            return Ok(attrs)
        }

        // #[cbor(...)]
        let cbor =
            if let syn::Meta::List(ml) = a.parse_meta()? {
                if !ml.path.is_ident("cbor") {
                    return Ok(Attributes::new(l))
                }
                ml
            } else {
                return Ok(Attributes::new(l))
            };

        for nested in &cbor.nested {
            match nested {
                syn::NestedMeta::Meta(syn::Meta::Path(arg)) =>
                    if arg.is_ident("index_only") {
                        attrs.try_insert(Kind::IndexOnly, Value::Span(nested.span()))?
                    } else if arg.is_ident("transparent") {
                        attrs.try_insert(Kind::Transparent, Value::Span(nested.span()))?
                    } else if arg.is_ident("map") {
                        attrs.try_insert(Kind::Encoding, Value::Encoding(Encoding::Map, nested.span()))?
                    } else if arg.is_ident("array") {
                        attrs.try_insert(Kind::Encoding, Value::Encoding(Encoding::Array, nested.span()))?
                    }
                syn::NestedMeta::Meta(syn::Meta::NameValue(arg)) =>
                    if arg.path.is_ident("encode_with") {
                        if let syn::Lit::Str(path) = &arg.lit {
                            let cc = CustomCodec::Encode(syn::parse_str(&path.value())?);
                            attrs.try_insert(Kind::Codec, Value::Codec(cc, nested.span()))?
                        }
                    } else if arg.path.is_ident("decode_with") {
                        if let syn::Lit::Str(path) = &arg.lit {
                            let cc = CustomCodec::Decode(syn::parse_str(&path.value())?);
                            attrs.try_insert(Kind::Codec, Value::Codec(cc, nested.span()))?
                        }
                    } else if arg.path.is_ident("with") {
                        if let syn::Lit::Str(path) = &arg.lit {
                            let cc = CustomCodec::Module(syn::parse_str(&path.value())?);
                            attrs.try_insert(Kind::Codec, Value::Codec(cc, nested.span()))?
                        }
                    }
                syn::NestedMeta::Meta(syn::Meta::List(arg)) =>
                    if arg.path.is_ident("n") {
                        if let Some(syn::NestedMeta::Lit(syn::Lit::Int(n))) = arg.nested.first() {
                            let idx = parse_int(n).map(Idx::N)?;
                            attrs.try_insert(Kind::Index, Value::Index(idx, a.tokens.span()))?;
                        } else {
                            return Err(syn::Error::new(arg.span(), "n expects a u32 argument"))
                        }
                    } else if arg.path.is_ident("b") {
                        if let Some(syn::NestedMeta::Lit(syn::Lit::Int(n))) = arg.nested.first() {
                            let idx = parse_int(n).map(Idx::B)?;
                            attrs.try_insert(Kind::Index, Value::Index(idx, a.tokens.span()))?;
                        } else {
                            return Err(syn::Error::new(arg.span(), "b expects a u32 argument"))
                        }
                    }
                syn::NestedMeta::Lit(_) => {}
            }
        }
        Ok(attrs)
    }

    pub fn encoding(&self) -> Option<Encoding> {
        self.get(Kind::Encoding).and_then(|v| v.encoding())
    }

    pub fn index(&self) -> Option<Idx> {
        self.get(Kind::Index).and_then(|v| v.index())
    }

    pub fn codec(&self) -> Option<&CustomCodec> {
        self.get(Kind::Codec).and_then(|v| v.codec())
    }

    pub fn transparent(&self) -> bool {
        self.contains_key(Kind::Transparent)
    }

    pub fn index_only(&self) -> bool {
        self.contains_key(Kind::IndexOnly)
    }

    fn contains_key(&self, k: Kind) -> bool {
        self.1.contains_key(&k)
    }

    fn get(&self, k: Kind) -> Option<&Value> {
        self.1.get(&k)
    }

    fn get_mut(&mut self, k: Kind) -> Option<&mut Value> {
        self.1.get_mut(&k)
    }

    fn try_insert(&mut self, key: Kind, val: Value) -> syn::Result<()> {
        match self.0 {
            Level::Struct => match key {
                Kind::Encoding | Kind::Transparent => {}
                Kind::Codec | Kind::Index | Kind::IndexOnly => {
                    let msg = format!("attribute is not supported on {}-level", self.0);
                    return Err(syn::Error::new(val.span(), msg))
                }
            }
            Level::Field => match key {
                Kind::Codec | Kind::Index => {}
                Kind::Encoding | Kind::IndexOnly | Kind::Transparent => {
                    let msg = format!("attribute is not supported on {}-level", self.0);
                    return Err(syn::Error::new(val.span(), msg))
                }
            }
            Level::Enum => match key {
                Kind::Encoding | Kind::IndexOnly => {}
                Kind::Codec | Kind::Index | Kind::Transparent => {
                    let msg = format!("attribute is not supported on {}-level", self.0);
                    return Err(syn::Error::new(val.span(), msg))
                }
            }
            Level::Variant => match key {
                Kind::Encoding | Kind::Index => {}
                Kind::Codec | Kind::IndexOnly | Kind::Transparent => {
                    let msg = format!("attribute is not supported on {}-level", self.0);
                    return Err(syn::Error::new(val.span(), msg))
                }
            }
        }
        if self.contains_key(key) {
            if let Some(Value::Codec(cc, _)) = self.get_mut(key) {
                let s = val.span();
                match (val, &cc) {
                    (Value::Codec(CustomCodec::Encode(e), _), CustomCodec::Decode(d)) => {
                        *cc = CustomCodec::Both { encode: e, decode: d.clone() };
                        return Ok(())
                    }
                    (Value::Codec(CustomCodec::Decode(d), _), CustomCodec::Encode(e)) => {
                        *cc = CustomCodec::Both { encode: e.clone(), decode: d };
                        return Ok(())
                    }
                    _ => return Err(syn::Error::new(s, "duplicate attribute"))
                }
            } else {
                return Err(syn::Error::new(val.span(), "duplicate attribute"))
            }
        }
        self.1.insert(key, val);
        Ok(())
    }
}

impl Value {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Value::Codec(_, s)    => *s,
            Value::Encoding(_, s) => *s,
            Value::Index(_, s)    => *s,
            Value::Span(s)        => *s
        }
    }

    fn index(&self) -> Option<Idx> {
        if let Value::Index(i, _) = self {
            Some(*i)
        } else {
            None
        }
    }

    fn codec(&self) -> Option<&CustomCodec> {
        if let Value::Codec(c, _) = self {
            Some(c)
        } else {
            None
        }
    }

    fn encoding(&self) -> Option<Encoding> {
        if let Value::Encoding(e, _) = self {
            Some(*e)
        } else {
            None
        }
    }
}

fn parse_u32_arg(a: &syn::Attribute) -> syn::Result<u32> {
    parse_int(&a.parse_args()?)
}

fn parse_int(n: &syn::LitInt) -> syn::Result<u32> {
    n.base10_digits()
     .parse()
     .map_err(|_| syn::Error::new(n.span(), "expected `u32` value"))
}
