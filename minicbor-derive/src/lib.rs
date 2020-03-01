extern crate proc_macro;

mod decode;
mod encode;

use proc_macro2::Span;
use syn::spanned::Spanned;

#[proc_macro_derive(Decode, attributes(n))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    decode::derive_from(input)
}

#[proc_macro_derive(Encode, attributes(n))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    encode::derive_from(input)
}

fn is_option(ty: &syn::Type) -> bool {
    let options = &[
        &["Option"][..],
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

fn field_indices<'a, I>(iter: I) -> syn::Result<Vec<u32>>
where
    I: Iterator<Item = &'a syn::Field>
{
    iter.map(|f| index_number(f.span(), &f.attrs)).collect()
}

fn variant_indices<'a, I>(iter: I) -> syn::Result<Vec<u32>>
where
    I: Iterator<Item = &'a syn::Variant>
{
    iter.map(|v| index_number(v.span(), &v.attrs)).collect()
}

