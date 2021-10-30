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
    ///
    /// In addition, an optional custom `is_null` function can be declared which
    /// is assumed to be of a type equivalent to:
    ///
    ///   `fn<T>(&T) -> bool`
    ///
    /// Declared with `#[cbor(is_null = "...")]`
    Encode(Encode),
    /// Custom decode function.
    ///
    /// Assumed to be of a type equivalent to:
    ///
    ///   `fn<T>(&mut Decoder<'_>) -> Result<T, Error>`
    ///
    /// Declared with `#[cbor(decode_with = "...")]`.
    ///
    /// In addition, an optional custom `null` function can be declared which
    /// is assumed to be of a type equivalent to:
    ///
    ///   `fn<T>() -> Option<T>`
    ///
    /// Declared with `#[cbor(null = "...")]`
    Decode(Decode),
    /// The combination of `encode_with` + `is_null` and `decode_with` + `null`.
    Both(Box<Encode>, Box<Decode>),
    /// A module which contains custom encode/decode functions.
    ///
    /// The module is assumed to contain two functions named `encode` and
    /// `decode` whose types match those declared with
    /// `#[cbor(encode_with = "...")]` or `#[cbor(decode_with = "...")]`
    /// respectively. Declared with `#[cbor(with = "...")]`.
    ///
    /// Optionally, the attribute `has_null` can be added which means that
    /// the module contains functions `is_null` and `null` matching those
    /// declared with `is_null` and `null` when using `encode_with` and
    /// `decode_with`.
    Module(syn::ExprPath, bool)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Encode {
    pub encode: syn::ExprPath,
    pub is_null: Option<syn::ExprPath>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Decode {
    pub decode: syn::ExprPath,
    pub null: Option<syn::ExprPath>
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

    /// Does this codec support checking for optionality?
    pub fn has_is_null(&self) -> bool {
        matches!(self, CustomCodec::Encode(Encode { is_null: Some(_), .. }))
    }

    /// Does this codec support creating a default optional value?
    pub fn has_null(&self) -> bool {
        matches!(self, CustomCodec::Decode(Decode { null: Some(_), .. }))
    }

    /// Does this codec module support optionality semantics?
    pub fn has_module_null(&self) -> bool {
        matches!(self, CustomCodec::Module(_, true))
    }

    /// Extract the encode function unless this `CustomCodec` does not declare one.
    pub fn to_encode_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Encode(e)    => Some(e.encode.clone()),
            CustomCodec::Both(e, _)   => Some(e.encode.clone()),
            CustomCodec::Decode(_)    => None,
            CustomCodec::Module(p, _) => {
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
            CustomCodec::Decode(d)    => Some(d.decode.clone()),
            CustomCodec::Both(_, d)   => Some(d.decode.clone()),
            CustomCodec::Encode(_)    => None,
            CustomCodec::Module(p, _) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("decode", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
        }
    }

    /// Extrace the `is_null` function if possible.
    pub fn to_is_null_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Encode(e)       => e.is_null.clone(),
            CustomCodec::Both(e, _)      => e.is_null.clone(),
            CustomCodec::Module(p, true) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("is_null", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
            CustomCodec::Module(_, false) => None,
            CustomCodec::Decode(_)        => None
        }
    }

    /// Extrace the `null` function if possible.
    pub fn to_null_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Decode(d)       => d.null.clone(),
            CustomCodec::Both(_, d)      => d.null.clone(),
            CustomCodec::Module(p, true) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("null", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
            CustomCodec::Module(_, false) => None,
            CustomCodec::Encode(_)        => None
        }
    }

    /// Set the `is_null` function.
    pub fn set_is_null(&mut self, p: syn::ExprPath) {
        match self {
            CustomCodec::Encode(e)  => e.is_null = Some(p),
            CustomCodec::Both(e, _) => e.is_null = Some(p),
            CustomCodec::Module(..) => {}
            CustomCodec::Decode(_)  => {}
        }
    }

    /// Set the `null` function.
    pub fn set_null(&mut self, p: syn::ExprPath) {
        match self {
            CustomCodec::Decode(d)  => d.null = Some(p),
            CustomCodec::Both(_, d) => d.null = Some(p),
            CustomCodec::Module(..) => {}
            CustomCodec::Encode(_)  => {}
        }
    }

    /// Mark the custom codec module as supporting optionality.
    pub fn set_module_null(&mut self) {
        if let CustomCodec::Module(_, v) = self {
            *v = true
        }
    }
}

