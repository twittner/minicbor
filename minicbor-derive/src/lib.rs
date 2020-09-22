//! Procedural macros to derive minicbor's `Encode` and `Decode` traits.
//!
//! Deriving is supported for `struct`s and `enum`s. The encoding is optimised
//! for forward and backward compatibility and the overall approach is
//! influenced by [Google's Protocol Buffers][1].
//!
//! The goal is that ideally a change to a type still allows older software,
//! which is unaware of the changes, to decode values of the changed type
//! (forward compatibility) and newer software, to decode values of types
//! encoded by older software, which do not include the changes made to the
//! type (backward compatibility).
//!
//! In order to reach this goal, the encoding has the following characteristics:
//!
//! 1. The encoding does not contain any names, i.e. no field names, type names
//! or variant names. Instead, every field and every constructor needs to be
//! annotated with an (unsigned) index number, e.g. `#[n(1)]`.
//!
//! 2. Unknown fields are ignored during decoding.
//!
//! 3. Optional types default to `None` if their value is not present during
//! decoding.
//!
//! 4. Optional enums default to `None` if an unknown variant is encountered
//! during decoding.
//!
//! Item **1** ensures that names can be changed freely without compatibility
//! concerns. Item **2** ensures that new fields do not affect older software.
//! Item **3** ensures that newer software can stop producing optional values.
//! Item **4** ensures that enums can get new variants that older software is
//! not aware of. By "fields" we mean the elements of structs and tuple structs
//! as well as enum structs and enum tuples. In addition, it is a compatible
//! change to turn a unit variant into a struct or tuple variant if all fields
//! are optional.
//!
//! From the above it should be obvious that *non-optional fields need to be
//! present forever*, so they should only be part of a type after careful
//! consideration.
//!
//! It should be emphasised that an `enum` itself can not be changed in a
//! compatible way. An unknown variant causes an error. It is only when they
//! are declared as an optional field type that unknown variants of an enum
//! are mapped to `None`. In other words, *only structs can be used as
//! top-level types in a forward and backward compatible way, enums can not.*
//!
//! # Example
//!
//! ```
//! use minicbor::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct Point {
//!     #[n(0)] x: f64,
//!     #[n(1)] y: f64
//! }
//!
//! #[derive(Encode, Decode)]
//! struct ConvexHull {
//!     #[n(0)] left: Point,
//!     #[n(1)] right: Point,
//!     #[n(2)] points: Vec<Point>,
//!     #[n(3)] state: Option<State>
//! }
//!
//! #[derive(Encode, Decode)]
//! enum State {
//!     #[n(0)] Start,
//!     #[n(1)] Search { #[n(0)] info: u64 }
//! }
//! ```
//!
//! In this example the following changes would be compatible in both
//! directions:
//!
//! - Renaming every identifier.
//!
//! - Adding optional fields to `Point`, `ConvexHull`, `State::Start` or
//! `State::Search`.
//!
//! - Adding more variants to `State` *iff* `State` is only decoded as part of
//! `ConvexHull`. Direct decoding of `State` would produce an `UnknownVariant`
//! error for those new variants.
//!
//! [1]: https://developers.google.com/protocol-buffers/
//!
//! # Attributes and borrowing
//!
//! Each field and variant needs to be annotated with an index number, which is
//! used instead of the name, using either **`n`** or **`b`** as attribute names.
//! For the encoding it makes no difference which one to choose. For decoding,
//! `b` indicates that the value borrows from the decoding input, whereas `n`
//! produces non-borrowed values (except for implicit borrows).
//!
//! ## Encoding format
//!
//! The actual CBOR encoding to use can be selected by attaching either the
//! **`#[cbor(array)]`** or **`#[cbor(map)]`** attribute to structs, enums or
//! enum variants. By default `#[cbor(array)]` is implied. The attribute
//! attached to an enum applies to all its variants but can be overriden per
//! variant with another such attribute.
//!
//! ## Implicit borrowing
//!
//! The following types implicitly borrow from the decoding input, which means
//! their lifetimes are constrained by the input lifetime:
//!
//! - `&'_ str`
//! - `&'_ [u8]`
//! - `Option<&'_ str>`
//! - `Option<&'_ [u8]>`
//!
//! ## Explicit borrowing
//!
//! If a type is annotated with **`#[b(...)]`**, all its lifetimes will be
//! constrained to the input lifetime.
//!
//! If the type is a `std::borrow::Cow<'_, str>` or `std::borrow::Cow<'_, [u8]>`
//! type, the generated code will decode the inner type and construct a
//! `Cow::Borrowed` variant, contrary to the `Cow` impl of `Decode` which
//! produces owned values.
//!
//! ## Other attributes
//!
//! ### `encode_with`, `decode_with` and `with`
//!
//! Fields in structs and enum variants may be annotated with
//! **`#[cbor(encode_with = "path")]`**, **`#[cbor(decode_with = "path")]`** or
//! **`#[cbor(with = "module-path")]`** where `path` is the full path to a
//! function which is used instead of `Encode::encode` to encode the field or
//! `Decode::decode` to decode the field respectively. The types of these
//! functions must be equivalent to `Encode::encode` or `Decode::decode`.
//! The `with` attribute combines the other two with `module-path` denoting the
//! full path to a module with two functions `encode` and `decode` as members,
//! which are used for encoding and decoding of the field. These three
//! attributes can either override an existing `Encode` or `Decode` impl or be
//! used for types which do not implement those traits at all.
//!
//! ### `transparent`
//!
//! A **`#[cbor(transparent)]`** attribute can be attached to structs with
//! exactly one field (aka newtypes). If present, the generated `Encode` and
//! `Decode` impls will just forward the `encode`/`decode` calls to the inner
//! type, i.e. the resulting CBOR representation will be identical to the one
//! of the inner type.
//!
//! # CBOR encoding
//!
//! The CBOR values produced by a derived `Encode` implementation are of the
//! following formats.
//!
//! ## Structs
//!
//! ### Array encoding
//!
//! By default or if a struct has the **`#[cbor(array)]`** attribute, it will
//! be represented as a CBOR array. Its index numbers are represened by the
//! position of the field value in this array. Any gaps between index numbers
//! are filled with CBOR NULL values and `Option`s which are `None` likewise
//! end up as NULLs in this array.
//!
//! ```text
//! <<struct-as-array encoding>> =
//!     `array(n)`
//!         item_0
//!         item_1
//!         ...
//!         item_n
//! ```
//!
//! ### Map encoding
//!
//! If a struct has the **`#[cbor(map)]`** attribute, then it will be
//! represented as a CBOR map with keys corresponding to the numeric index
//! value:
//!
//! ```text
//! <<struct-as-map encoding>> =
//!     `map(n)`
//!         `0` item_0
//!         `1` item_1
//!         ...
//!          n  item_n
//! ```
//!
//! Optional fields whose value is `None` are not encoded.
//!
//! ## Enums
//!
//! Each enum variant is encoded as a two-element array. The first element
//! is the variant index and the second the actual variant value:
//!
//! ```text
//! <<enum encoding>> =
//!     | `array(2)` n <<struct-as-array encoding>> ; if #[cbor(array)]
//!     | `array(2)` n <<struct-as-map encoding>>   ; if #[cbor(map)]
//! ```
//!
//! ## Which encoding to use?
//!
//! The map encoding needs to represent the indexes explicitly in the encoding
//! which costs at least one extra byte per field value, whereas the array
//! encoding does not need to encode the indexes. On the other hand, absent
//! values, i.e. `None`s and gaps between indexes are not encoded with maps but
//! need to be encoded explicitly with arrays as NULLs which need one byte each.
//! Which encoding to choose depends therefore on the nature of the type that
//! should be encoded:
//!
//! - *Dense types* are types which contain only few `Option`s or their `Option`s
//! are assumed to be `Some`s usually. They are best encoded as arrays.
//!
//! - *Sparse types* are types with many `Option`s and their `Option`s are usually
//! `None`s. They are best encoded as maps.
//!
//! When selecting the encoding, future changes to the type should be considered
//! as they may turn a dense type into a sparse one over time.

extern crate proc_macro;

mod decode;
mod encode;

use quote::{ToTokens, TokenStreamExt};
use proc_macro2::Span;
use syn::spanned::Spanned;
use std::collections::HashSet;

/// Derive the `minicbor::Decode` trait for a struct or enum.
///
/// See the [crate] documentation for details.
#[proc_macro_derive(Decode, attributes(n, b, cbor))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    decode::derive_from(input)
}

/// Derive the `minicbor::Encode` trait for a struct or enum.
///
/// See the [crate] documentation for details.
#[proc_macro_derive(Encode, attributes(n, b, cbor))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    encode::derive_from(input)
}

// Helpers ////////////////////////////////////////////////////////////////////

/// Check if the given type is an `Option` whose inner type matches the predicate.
fn is_option(ty: &syn::Type, pred: impl FnOnce(&syn::Type) -> bool) -> bool {
    if let syn::Type::Path(t) = ty {
        if let Some(s) = t.path.segments.last() {
            if s.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(b) = &s.arguments {
                    if b.args.len() == 1 {
                        if let syn::GenericArgument::Type(ty) = &b.args[0] {
                            return pred(ty)
                        }
                    }
                }
            }
        }
    }
    false
}

/// Check if the given type is a `Cow` whose inner type matches the predicate.
fn is_cow(ty: &syn::Type, pred: impl FnOnce(&syn::Type) -> bool) -> bool {
    if let syn::Type::Path(t) = ty {
        if let Some(s) = t.path.segments.last() {
            if s.ident == "Cow" {
                if let syn::PathArguments::AngleBracketed(b) = &s.arguments {
                    if b.args.len() == 2 {
                        if let syn::GenericArgument::Lifetime(_) = &b.args[0] {
                            if let syn::GenericArgument::Type(ty) = &b.args[1] {
                                return pred(ty)
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Check if the given type is a `&str`.
fn is_str(ty: &syn::Type) -> bool {
    if let syn::Type::Path(t) = ty {
        t.qself.is_none() && t.path.segments.len() == 1 && t.path.segments[0].ident == "str"
    } else {
        false
    }
}

/// Check if the given type is a `&[u8]`.
fn is_byte_slice(ty: &syn::Type) -> bool {
    if let syn::Type::Path(t) = ty {
        return t.qself.is_none() &&
            ((t.path.segments.len() == 1 && t.path.segments[0].ident == "ByteSlice")
                || (t.path.segments.len() == 2
                    && t.path.segments[0].ident == "bytes"
                    && t.path.segments[1].ident == "ByteSlice")
                || (t.path.segments.len() == 3
                    && t.path.segments[0].ident == "minicbor"
                    && t.path.segments[1].ident == "bytes"
                    && t.path.segments[2].ident == "ByteSlice"))
    }
    if let syn::Type::Slice(t) = ty {
        if let syn::Type::Path(t) = &*t.elem {
            t.qself.is_none() && t.path.segments.len() == 1 && t.path.segments[0].ident == "u8"
        } else {
            false
        }
    } else {
        false
    }
}

/// Get the lifetime of the given type if it is an `Option` whose inner type matches the predicate.
fn option_lifetime(ty: &syn::Type, pred: impl FnOnce(&syn::Type) -> bool) -> Option<syn::Lifetime> {
    if let syn::Type::Path(t) = ty {
        if let Some(s) = t.path.segments.last() {
            if s.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(b) = &s.arguments {
                    if b.args.len() == 1 {
                        if let syn::GenericArgument::Type(syn::Type::Reference(ty)) = &b.args[0] {
                            if pred(&*ty.elem) {
                                return ty.lifetime.clone()
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Get all lifetimes of a type.
fn get_lifetimes(ty: &syn::Type, set: &mut HashSet<syn::Lifetime>) {
    match ty {
        syn::Type::Array(t) => get_lifetimes(&t.elem, set),
        syn::Type::Slice(t) => get_lifetimes(&t.elem, set),
        syn::Type::Paren(t) => get_lifetimes(&t.elem, set),
        syn::Type::Group(t) => get_lifetimes(&t.elem, set),
        syn::Type::Ptr(t)   => get_lifetimes(&t.elem, set),
        syn::Type::Reference(t) => {
            if let Some(l) = &t.lifetime {
                set.insert(l.clone());
            }
            get_lifetimes(&t.elem, set)
        }
        syn::Type::Tuple(t) => {
            for t in &t.elems {
                get_lifetimes(t, set)
            }
        }
        syn::Type::Path(t) => {
            for s in &t.path.segments {
                if let syn::PathArguments::AngleBracketed(b) = &s.arguments {
                    for a in &b.args {
                        match a {
                            syn::GenericArgument::Type(t)     => get_lifetimes(t, set),
                            syn::GenericArgument::Binding(b)  => get_lifetimes(&b.ty, set),
                            syn::GenericArgument::Lifetime(l) => { set.insert(l.clone()); }
                            _                                 => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Get the lifetime of a reference if its type matches the predicate.
fn tyref_lifetime(ty: &syn::Type, pred: impl FnOnce(&syn::Type) -> bool) -> Option<syn::Lifetime> {
    if let syn::Type::Reference(p) = ty {
        if pred(&*p.elem) {
            return p.lifetime.clone()
        }
    }
    None
}

/// Get the set of lifetimes which need to be constrained to the decoding input lifetime.
fn lifetimes_to_constrain<'a, I>(types: I) -> HashSet<syn::Lifetime>
where
    I: Iterator<Item = (&'a Idx, &'a syn::Type)>
{
    let mut set = HashSet::new();
    for (i, t) in types {
        if let Some(l) = tyref_lifetime(t, is_str) {
            set.insert(l);
            continue
        }
        if let Some(l) = tyref_lifetime(t, is_byte_slice) {
            set.insert(l);
            continue
        }
        if let Some(l) = option_lifetime(t, is_str) {
            set.insert(l);
            continue
        }
        if let Some(l) = option_lifetime(t, is_byte_slice) {
            set.insert(l);
            continue
        }
        if i.is_b() {
            get_lifetimes(t, &mut set)
        }
    }
    set
}

/// The index attribute.
#[derive(Debug, Clone, Copy)]
enum Idx {
    /// A regular, non-borrowing index.
    N(u32),
    /// An index which indicates that the value borrows from the decoding input.
    B(u32)
}

impl ToTokens for Idx {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(proc_macro2::Literal::u32_unsuffixed(self.val()))
    }
}

impl Idx {
    /// Test if `Idx` is the `B` variant.
    fn is_b(self) -> bool {
        if let Idx::B(_) = self {
            true
        } else {
            false
        }
    }

    /// Get the numeric index value.
    fn val(self) -> u32 {
        match self {
            Idx::N(i) => i,
            Idx::B(i) => i
        }
    }
}

/// Get the index number from the list of attributes.
///
/// The first attribute `n` will be used and its argument must be an
/// unsigned integer literal that fits into a `u32`.
fn index_number(s: Span, attrs: &[syn::Attribute]) -> syn::Result<Idx> {
    for a in attrs {
        if a.path.is_ident("n") {
            let lit: syn::LitInt = a.parse_args()?;
            return lit.base10_digits().parse()
                .map_err(|_| syn::Error::new(a.tokens.span(), "expected `u32` value"))
                .map(Idx::N)
        }
        if a.path.is_ident("b") {
            let lit: syn::LitInt = a.parse_args()?;
            return lit.base10_digits().parse()
                .map_err(|_| syn::Error::new(a.tokens.span(), "expected `u32` value"))
                .map(Idx::B)
        }
    }
    Err(syn::Error::new(s, "missing `#[n(...)]` or `#[b(...)]` attribute"))
}

/// Check that there are no duplicate elements in `iter`.
fn check_uniq<I>(s: Span, iter: I) -> syn::Result<()>
where
    I: IntoIterator<Item = Idx>
{
    let mut set = HashSet::new();
    let mut ctr = 0;
    for u in iter {
        set.insert(u.val());
        ctr += 1;
    }
    if ctr != set.len() {
        return Err(syn::Error::new(s, "duplicate index numbers"))
    }
    Ok(())
}

/// Get the index number of every field.
fn field_indices<'a, I>(iter: I) -> syn::Result<Vec<Idx>>
where
    I: Iterator<Item = &'a syn::Field>
{
    iter.map(|f| index_number(f.span(), &f.attrs)).collect()
}

/// Get the index number of every variant.
fn variant_indices<'a, I>(iter: I) -> syn::Result<Vec<Idx>>
where
    I: Iterator<Item = &'a syn::Variant>
{
    iter.map(|v| index_number(v.span(), &v.attrs)).collect()
}

/// The encoding to use for structs and enum variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Array,
    Map
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::Array
    }
}

/// Determine attribute value of the `#[cbor(map|array)]` attribute.
fn encoding(a: &syn::Attribute) -> Option<Encoding> {
    match a.parse_meta().ok()? {
        syn::Meta::List(ml) if ml.path.is_ident("cbor") => {
            if let Some(syn::NestedMeta::Meta(syn::Meta::Path(arg))) = ml.nested.first() {
                if arg.is_ident("map") {
                    return Some(Encoding::Map)
                }
                if arg.is_ident("array") {
                    return Some(Encoding::Array)
                }
            }
        }
        _ => {}
    }
    None
}

/// Custom encode/decode functions.
enum CustomCodec {
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
    /// A module which contains custom encode/decode functions.
    ///
    /// The module is assumed to contain two functions named `encode` and
    /// `decode` whose types match those declared with
    /// `#[cbor(encode_with = "...")]` or `#[cbor(decode_with = "...")]`
    /// respectively. Declared with `#[cbor(with = "...")]`.
    Both(syn::ExprPath)
}

impl CustomCodec {
    /// Is this a custom codec from `encode_with` or `with`?
    fn is_encode(&self) -> bool {
        matches!(self, CustomCodec::Encode(_) | CustomCodec::Both(_))
    }

    /// Is this a custom codec from `decode_with` or `with`?
    fn is_decode(&self) -> bool {
        matches!(self, CustomCodec::Decode(_) | CustomCodec::Both(_))
    }

    /// Extract the encode function unless this `CustomCodec` does not declare one.
    fn to_encode_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Encode(p) => Some(p.clone()),
            CustomCodec::Decode(_) => None,
            CustomCodec::Both(p) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("encode", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
        }
    }

    /// Extract the decode function unless this `CustomCodec` does not declare one.
    fn to_decode_path(&self) -> Option<syn::ExprPath> {
        match self {
            CustomCodec::Encode(_) => None,
            CustomCodec::Decode(p) => Some(p.clone()),
            CustomCodec::Both(p) => {
                let mut p = p.clone();
                let ident = syn::Ident::new("decode", proc_macro2::Span::call_site());
                p.path.segments.push(ident.into());
                Some(p)
            }
        }
    }
}

/// Determine the attribute value of the `#[cbor(encode_with|decode_with|with)]` attribute.
fn custom_codec(a: &syn::Attribute) -> syn::Result<Option<CustomCodec>> {
    if let syn::Meta::List(ml) = a.parse_meta()? {
        if !ml.path.is_ident("cbor") {
            return Ok(None)
        }
        if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(arg))) = ml.nested.first() {
            if arg.path.is_ident("encode_with") {
                if let syn::Lit::Str(path) = &arg.lit {
                    return Ok(Some(CustomCodec::Encode(syn::parse_str(&path.value())?)))
                }
            }
            if arg.path.is_ident("decode_with") {
                if let syn::Lit::Str(path) = &arg.lit {
                    return Ok(Some(CustomCodec::Decode(syn::parse_str(&path.value())?)))
                }
            }
            if arg.path.is_ident("with") {
                if let syn::Lit::Str(path) = &arg.lit {
                    return Ok(Some(CustomCodec::Both(syn::parse_str(&path.value())?)))
                }
            }
        }
    }
    Ok(None)
}

/// Traverse all field types and collect all type parameters along the way.
fn collect_type_params<'a, I>(all: &syn::Generics, fields: I) -> HashSet<syn::TypeParam>
where
    I: Iterator<Item = &'a syn::Field>
{
    use syn::visit::Visit;

    struct Collector {
        all: Vec<syn::Ident>,
        found: HashSet<syn::TypeParam>
    }

    impl<'a> Visit<'a> for Collector {
        fn visit_field(&mut self, f: &'a syn::Field) {
            if let syn::Type::Path(ty) = &f.ty {
                if let Some(t) = ty.path.segments.first() {
                    if self.all.contains(&t.ident) {
                        self.found.insert(syn::TypeParam::from(t.ident.clone()));
                    }
                }
            }
            self.visit_type(&f.ty)
        }

        fn visit_path(&mut self, p: &'a syn::Path) {
            if p.leading_colon.is_none() && p.segments.len() == 1 {
                let id = &p.segments[0].ident;
                if self.all.contains(id) {
                    self.found.insert(syn::TypeParam::from(id.clone()));
                }
            }
            syn::visit::visit_path(self, p)
        }
    }

    let mut c = Collector {
        all: all.type_params().map(|tp| tp.ident.clone()).collect(),
        found: HashSet::new()
    };

    for f in fields {
        c.visit_field(f)
    }

    c.found
}

/// Check if the attribute matches the given identifier.
fn is_cbor_attr(a: &syn::Attribute, ident: &str) -> syn::Result<bool> {
    match a.parse_meta()? {
        syn::Meta::List(ml) if ml.path.is_ident("cbor") => {
            if let Some(syn::NestedMeta::Meta(syn::Meta::Path(arg))) = ml.nested.first() {
                if arg.is_ident(ident) {
                    return Ok(true)
                }
            }
            if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(arg))) = ml.nested.first() {
                if arg.path.is_ident(ident) {
                    return Ok(true)
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

/// Find any of the attributes that matches the given identifier.
fn find_cbor_attr<'a, I>(attrs: I, ident: &str) -> syn::Result<Option<&'a syn::Attribute>>
where
    I: Iterator<Item = &'a syn::Attribute>
{
    for a in attrs {
        if is_cbor_attr(a, ident)? {
            return Ok(Some(a))
        }
    }
    Ok(None)
}
