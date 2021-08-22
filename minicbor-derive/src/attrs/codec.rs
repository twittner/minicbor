/// Custom encode/decode functions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CustomCodec {
    /// Custom encode function.
    ///
    /// Assumed to be of a type equivalent to:
    ///
    ///   `fn<T, W: Write>(&T, &mut Encoder<W>) -> Result<(), Error<W::Error>>`
    ///
    /// Declared with `#[cbor(encode_with = "...")]`.
    Encode(syn::ExprPath),
    /// Custom decode function.
    ///
    /// Assumed to be of a type equivalent to:
    ///
    ///   `fn<T>(&mut Decoder<'_>) -> Result<T, Error>`
    ///
    /// Declared with `#[cbor(decode_with = "...")]`.
    Decode(syn::ExprPath),
    /// The combination of `encode_with` and `decode_with`.
    Both {
        encode: syn::ExprPath,
        decode: syn::ExprPath
    },
    /// A module which contains custom encode/decode functions.
    ///
    /// The module is assumed to contain two functions named `encode` and
    /// `decode` whose types match those declared with
    /// `#[cbor(encode_with = "...")]` or `#[cbor(decode_with = "...")]`
    /// respectively. Declared with `#[cbor(with = "...")]`.
    Module(syn::ExprPath)
}

impl CustomCodec {
    /// Is this a custom codec from `encode_with` or `with`?
    pub fn is_encode(&self) -> bool {
        !matches!(self, CustomCodec::Decode(_))
    }

    /// Is this a custom codec from `decode_with` or `with`?
    pub fn is_decode(&self) -> bool {
        !matches!(self, CustomCodec::Encode(_))
    }

    /// Extract the encode function unless this `CustomCodec` does not declare one.
    pub fn to_encode_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Encode(p) => Some(p.clone()),
            CustomCodec::Both { encode, .. } => Some(encode.clone()),
            CustomCodec::Decode(_) => None,
            CustomCodec::Module(p) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("encode", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
        }
    }

    /// Extract the decode function unless this `CustomCodec` does not declare one.
    pub fn to_decode_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Encode(_) => None,
            CustomCodec::Decode(p) => Some(p.clone()),
            CustomCodec::Both { decode, .. } => Some(decode.clone()),
            CustomCodec::Module(p) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("decode", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
        }
    }
}

