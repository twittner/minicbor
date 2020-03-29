//! Procedural macros to derive minicbor's `Encode` and `Decode` traits.
//!
//! Deriving is supported for `struct`s and `enum`s. The encoding is optimised
//! for forward and backward compatibility and the overall approach is
//! influenced by [Google's Protocol Buffers][1].
//!
//! The goal is that ideally a change to a type still allows older software,
//! which is unaware of the changes, to decode values of the changed type
//! (forward compatibility) and newer software, which knows about the changes,
//! to decode values of types which have been encoded by older software and
//! which therefore do not include the changes made to the type (backward
//! compatibility).
//!
//! In order to reach this goal the encoding has the following characteristics:
//!
//! 1. The encoding does not contain any names, i.e. no field names, type names
//! or variant names. Instead every field and every constructor needs to be
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
//! Item **1**. ensures that names can be changed freely without compatibility
//! concerns. Item **2**. ensures that new fields do not affect older software.
//! Item **3**. ensures that newer software can stop producing optional values.
//! Item **4**. ensures that enums can get new variants that older software is
//! not aware of. By "fields" we mean the elements of structs and tuple
//! structs as well as enum structs and enum tuples. In addition it is a
//! compatible change to turn a unit variant into a struct or tuple variant if
//! all fields are optional.
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

// Structs are encoded as:
//
//      struct_encoding =
//          begin_map
//              n_0 item_0
//              n_1 item_1
//              ...
//              n_k item_k
//          end
//
// `n_0`, `n_1` etc. denote the field indices. `begin_map` means the start of
// an indefinite map which terminates at `end`. Optional fields whose value is
// `None` are not encoded. Enums are encoded as:
//
//      enum_encoding =
//          array(2) n_var map(0) // unit constructor
//          | array(2) n_var <<struct_encoding>>
//
// `n_var` denotes the variant index. `array(2)` means a 2-element array and
// `map(0)` an empty map.
//
// While the enum encoding costs as much as 2 bytes extra for unit constructors,
// those are import for compatibility. The array is required so that enums in
// unknown fields can be skipped over. The empty map is required so we can skip
// over unknown variants as we can not know when to expect a unit constructor
// if we do not even know the variant.

extern crate proc_macro;

mod decode;
mod encode;

use proc_macro2::Span;
use syn::spanned::Spanned;

/// Derive the `minicbor::Decode` trait for a struct or enum.
///
/// See the [crate] documentation for details.
#[proc_macro_derive(Decode, attributes(n))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    decode::derive_from(input)
}

/// Derive the `minicbor::Encode` trait for a struct or enum.
///
/// See the [crate] documentation for details.
#[proc_macro_derive(Encode, attributes(n))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    encode::derive_from(input)
}

/// Check if the given type is an `Option`.
fn is_option(ty: &syn::Type) -> bool {
    let options = &[
        &["Option"][..],
        &["option", "Option"][..],
        &["std", "option", "Option"][..],
        &["core", "option", "Option"][..]
    ];
    if let syn::Type::Path(tp) = ty {
        for o in options {
            if tp.path.segments.iter().zip(o.iter()).all(|(a, b)| a.ident == b) {
                return true
            }
        }
    }
    false
}

/// Get the index number from the list of attributes.
///
/// The first attribute `n` will be used and its argument must be an
/// unsigned integer literal that fits into a `u32`.
fn index_number(s: Span, attrs: &[syn::Attribute]) -> syn::Result<u32> {
    for a in attrs {
        if a.path.is_ident("n") {
            let lit: syn::LitInt = a.parse_args()?;
            return lit.base10_digits().parse().map_err(|_| {
                syn::Error::new(a.tokens.span(), "expected `u32` value")
            })
        }
    }
    Err(syn::Error::new(s, "missing `#[n(...)]` attribute"))
}

/// Check that there are no duplicate elements in `iter`.
fn check_uniq<I>(s: Span, iter: I) -> syn::Result<()>
where
    I: IntoIterator<Item = u32>
{
    let mut set = std::collections::HashSet::new();
    let mut ctr = 0;
    for u in iter {
        set.insert(u);
        ctr += 1;
    }
    if ctr != set.len() {
        return Err(syn::Error::new(s, "duplicate index numbers"))
    }
    Ok(())
}

/// Get the index number of every field.
fn field_indices<'a, I>(iter: I) -> syn::Result<Vec<u32>>
where
    I: Iterator<Item = &'a syn::Field>
{
    iter.map(|f| index_number(f.span(), &f.attrs)).collect()
}

/// Get the index number of every variant.
fn variant_indices<'a, I>(iter: I) -> syn::Result<Vec<u32>>
where
    I: Iterator<Item = &'a syn::Variant>
{
    iter.map(|v| index_number(v.span(), &v.attrs)).collect()
}

