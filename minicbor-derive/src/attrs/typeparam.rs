use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeParams {
    Encode(HashMap<syn::Ident, syn::TypeParam>),
    Decode(HashMap<syn::Ident, syn::TypeParam>),
    Length(HashMap<syn::Ident, syn::TypeParam>),
    All {
        encode: HashMap<syn::Ident, syn::TypeParam>,
        decode: HashMap<syn::Ident, syn::TypeParam>,
        length: HashMap<syn::Ident, syn::TypeParam>,
    },
}

impl TypeParams {
    pub fn try_merge(&mut self, s: proc_macro2::Span, other: Self) -> syn::Result<()> {
        match (&mut *self, other) {
            (Self::Encode(e1), Self::Encode(e2)) => try_merge(s, e1, e2)?,
            (Self::Decode(d1), Self::Decode(d2)) => try_merge(s, d1, d2)?,
            (Self::Length(l1), Self::Length(l2)) => try_merge(s, l1, l2)?,
            (Self::Encode(e), Self::Decode(d)) => {
                *self = Self::All {
                    encode: mem::take(e),
                    decode: d,
                    length: HashMap::new(),
                }
            }
            (Self::Encode(e), Self::Length(l)) => {
                *self = Self::All {
                    encode: mem::take(e),
                    decode: HashMap::new(),
                    length: l,
                }
            }
            (Self::Decode(d), Self::Encode(e)) => {
                *self = Self::All {
                    encode: e,
                    decode: mem::take(d),
                    length: HashMap::new(),
                }
            }
            (Self::Decode(d), Self::Length(l)) => {
                *self = Self::All {
                    encode: HashMap::new(),
                    decode: mem::take(d),
                    length: l,
                }
            }
            (Self::Length(l), Self::Encode(e)) => {
                *self = Self::All {
                    encode: e,
                    decode: HashMap::new(),
                    length: mem::take(l),
                }
            }
            (Self::Length(l), Self::Decode(d)) => {
                *self = Self::All {
                    encode: HashMap::new(),
                    decode: d,
                    length: mem::take(l),
                }
            }
            (
                Self::Encode(e1),
                Self::All {
                    encode: e2,
                    decode,
                    length,
                },
            ) => {
                try_merge(s, e1, e2)?;
                *self = Self::All {
                    encode: mem::take(e1),
                    decode,
                    length,
                }
            }
            (
                Self::Decode(d1),
                Self::All {
                    encode,
                    decode: d2,
                    length,
                },
            ) => {
                try_merge(s, d1, d2)?;
                *self = Self::All {
                    encode,
                    decode: mem::take(d1),
                    length,
                }
            }
            (
                Self::Length(l1),
                Self::All {
                    encode,
                    decode,
                    length: l2,
                },
            ) => {
                try_merge(s, l1, l2)?;
                *self = Self::All {
                    encode,
                    decode,
                    length: mem::take(l1),
                }
            }
            (Self::All { encode: e1, .. }, Self::Encode(e2)) => try_merge(s, e1, e2)?,
            (Self::All { decode: d1, .. }, Self::Decode(d2)) => try_merge(s, d1, d2)?,
            (Self::All { length: l1, .. }, Self::Length(l2)) => try_merge(s, l1, l2)?,
            (
                Self::All {
                    encode: e1,
                    decode: d1,
                    length: l1,
                },
                Self::All {
                    encode: e2,
                    decode: d2,
                    length: l2,
                },
            ) => {
                try_merge(s, e1, e2)?;
                try_merge(s, d1, d2)?;
                try_merge(s, l1, l2)?
            }
        }
        Ok(())
    }

    pub fn get_encode(&self, id: &syn::Ident) -> Option<&syn::TypeParam> {
        match self {
            Self::Decode(_) | Self::Length(_) => None,
            Self::Encode(e) => e.get(id),
            Self::All { encode, .. } => encode.get(id),
        }
    }

    pub fn get_decode(&self, id: &syn::Ident) -> Option<&syn::TypeParam> {
        match self {
            Self::Encode(_) | Self::Length(_) => None,
            Self::Decode(d) => d.get(id),
            Self::All { decode, .. } => decode.get(id),
        }
    }

    pub fn get_length(&self, id: &syn::Ident) -> Option<&syn::TypeParam> {
        match self {
            Self::Encode(_) | Self::Decode(_) => None,
            Self::Length(l) => l.get(id),
            Self::All { length, .. } => length.get(id),
        }
    }
}

fn try_merge<K, V>(s: proc_macro2::Span, a: &mut HashMap<K, V>, b: HashMap<K, V>) -> syn::Result<()>
where
    K: Eq + Hash,
{
    for (k, v) in b.into_iter() {
        if a.contains_key(&k) {
            return Err(syn::Error::new(s, "duplicate type parameter"));
        }
        a.insert(k, v);
    }
    Ok(())
}
