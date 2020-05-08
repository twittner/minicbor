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
//! used instead of the name, using either `n` or `b` as attribute names. For
//! the encoding it makes no difference which one to choose. For decoding, `b`
//! indicates that the value borrows from the decoding input, whereas `n`
//! produces non-borrowed values (except for implicit borrows).
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
//! If a type is annotated with `#[b(...)]`, all its lifetimes will be
//! constrained to the input lifetime.
//!
//! If the type is a `std::borrow::Cow<'_, str>` or `std::borrow::Cow<'_, [u8]>`
//! type, the generated code will decode the inner type and construct a
//! `Cow::Borrowed` variant, contrary to the `Cow` impl of `Decode` which
//! produces owned values.
//!
//! # CBOR encoding
//!
//! The CBOR values produced by a derived `Encode` implementation are of the
//! following format.
//!
//! ## Structs
//!
//! Each struct is encoded as a CBOR map with numeric keys:
//!
//! ```text
//! <<struct encoding>> =
//!     | `map(0)`         ; unit struct => empty map
//!     | `begin_map`      ; indefinite map otherwise
//!           `0` item_0
//!           `1` item_1
//!           ...
//!            n  item_n
//!       `end`
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
//! <<enum encoding>> = `array(2)` n <<struct encoding>>
//! ```

// While the enum encoding costs as much as 2 bytes extra for unit constructors,
// those are import for compatibility. The array is required so that enums in
// unknown fields can be skipped over. The empty map is required so we can skip
// over unknown variants as we can not know when to expect a unit constructor
// if we do not even know the variant.

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
#[proc_macro_derive(Decode, attributes(n, b))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    decode::derive_from(input)
}

/// Derive the `minicbor::Encode` trait for a struct or enum.
///
/// See the [crate] documentation for details.
#[proc_macro_derive(Encode, attributes(n, b))]
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
    N(u64),
    /// An index which indicates that the value borrows from the decoding input.
    B(u64)
}

impl ToTokens for Idx {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(proc_macro2::Literal::u64_suffixed(self.val()))
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
    fn val(self) -> u64 {
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

