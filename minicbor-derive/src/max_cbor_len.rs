use quote::quote;
use syn::spanned::Spanned;

use crate::attrs::{Attributes, Encoding, Level};
use crate::collect_type_params;
use crate::fields::Fields;
use crate::variants::Variants;

/// Entry point to derive `minicbor::MaxCborLen` on structs and enums.
///
/// This reuses the same field/variant/attribute infrastructure as `CborLen`,
/// but generates compile-time const expressions instead of runtime code.
/// The const helper fns `unsigned_cbor_len` and `signed_cbor_len` serve as
/// the const-compatible counterparts to `CborLen` impls for headers and indices.
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    let result = match &input.data {
        syn::Data::Struct(_) => on_struct(&mut input),
        syn::Data::Enum(_) => on_enum(&mut input),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "deriving `minicbor::MaxCborLen` for a `union` is not supported",
        )),
    };
    proc_macro::TokenStream::from(result.unwrap_or_else(|e| e.to_compile_error()))
}

fn on_struct(inp: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let data = if let syn::Data::Struct(d) = &inp.data {
        d
    } else {
        unreachable!()
    };

    let name = inp.ident.clone();
    let attrs = Attributes::try_from_iter(Level::Struct, inp.attrs.iter())?;
    let fields = Fields::try_from(name.span(), data.fields.iter(), &[&attrs])?;

    add_bounds(&mut inp.generics, &fields);
    let (ig, tg, wc) = inp.generics.split_for_impl();

    if attrs.transparent() {
        if fields.fields().len() != 1 {
            return Err(syn::Error::new(
                name.span(),
                "#[cbor(transparent)] requires exactly one field",
            ));
        }
        let ty = &fields.fields().next().unwrap().typ;
        return Ok(quote! {
            impl #ig minicbor::MaxCborLen for #name #tg #wc {
                const MAX_CBOR_LEN: usize = <#ty as minicbor::MaxCborLen>::MAX_CBOR_LEN;
            }
        });
    }

    let tag = tag_size(&attrs);
    let body = fields_size(&fields, attrs.encoding().unwrap_or_default())?;

    Ok(quote! {
        impl #ig minicbor::MaxCborLen for #name #tg #wc {
            const MAX_CBOR_LEN: usize = #tag + #body;
        }
    })
}

fn on_enum(inp: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let data = if let syn::Data::Enum(d) = &inp.data {
        d
    } else {
        unreachable!()
    };

    let name = inp.ident.clone();
    let enum_attrs = Attributes::try_from_iter(Level::Enum, inp.attrs.iter())?;
    let encoding = enum_attrs.encoding().unwrap_or_default();
    let index_only = enum_attrs.index_only();
    let flat = enum_attrs.flat();
    let variants = Variants::try_from(name.span(), data.variants.iter(), &enum_attrs)?;

    let mut all_fields = Vec::new();
    let mut sizes = Vec::new();

    for ((var, idx), va) in data
        .variants
        .iter()
        .zip(&variants.indices)
        .zip(&variants.attrs)
    {
        let fields = Fields::try_from(var.ident.span(), var.fields.iter(), &[va, &enum_attrs])?;
        all_fields.extend(fields.fields().cloned());
        let iv = idx.val();
        let tag = tag_size(va);
        let enc = va.encoding().unwrap_or(encoding);

        let size = match &var.fields {
            syn::Fields::Unit if index_only => quote! { minicbor::encode::signed_cbor_len(#iv) },
            syn::Fields::Unit if flat => quote! { 1 + minicbor::encode::signed_cbor_len(#iv) },
            syn::Fields::Unit => quote! { 1 + minicbor::encode::signed_cbor_len(#iv) + #tag + 1 },
            _ if index_only => {
                return Err(syn::Error::new(
                    var.ident.span(),
                    "index_only enums must not have fields",
                ));
            }
            _ => {
                let body = fields_size(&fields, enc)?;
                if flat {
                    quote! { #body + minicbor::encode::signed_cbor_len(#iv) }
                } else {
                    quote! { #body + #tag + 1 + minicbor::encode::signed_cbor_len(#iv) }
                }
            }
        };
        sizes.push(size);
    }

    let bound: syn::TypeParamBound = syn::parse_quote!(minicbor::MaxCborLen);
    let used = collect_type_params(&inp.generics, all_fields.iter());
    for p in inp.generics.type_params_mut() {
        if used.contains(&p.ident) && !p.bounds.iter().any(|b| *b == bound) {
            p.bounds.push(bound.clone());
        }
    }
    let (ig, tg, wc) = inp.generics.split_for_impl();
    let tag = tag_size(&enum_attrs);

    let body = sizes
        .into_iter()
        .rev()
        .reduce(|acc, v| quote! { minicbor::encode::const_max(#v, #acc) })
        .unwrap_or_else(|| quote!(0));

    Ok(quote! {
        impl #ig minicbor::MaxCborLen for #name #tg #wc {
            const MAX_CBOR_LEN: usize = #tag + #body;
        }
    })
}

/// Compute const expression for max encoded size of a field set.
///
/// Uses `<T as MaxCborLen>::MAX_CBOR_LEN` per field — the const analogue of
/// `CborLen::cbor_len()` — and `unsigned_cbor_len`/`signed_cbor_len` for
/// headers and index keys (the const analogues of the runtime CborLen impls
/// for integers).
fn fields_size(fields: &Fields, encoding: Encoding) -> syn::Result<proc_macro2::TokenStream> {
    let active: Vec<_> = fields.fields().collect();

    match encoding {
        Encoding::Map => {
            let n = active.len();
            let mut parts = vec![quote! { minicbor::encode::unsigned_cbor_len(#n as u64) }];
            for f in &active {
                let ty = &f.typ;
                let iv = f.index.val();
                let tag = tag_size(&f.attrs);
                parts.push(quote! {
                    minicbor::encode::signed_cbor_len(#iv) + #tag
                        + <#ty as minicbor::MaxCborLen>::MAX_CBOR_LEN
                });
            }
            Ok(quote! { #(#parts)+* })
        }
        Encoding::Array => {
            if active.is_empty() {
                return Ok(quote!(1));
            }
            let max_idx = active.iter().map(|f| f.index.val()).max().unwrap();
            let array_len = (max_idx + 1) as u64;
            let mut parts = vec![quote! { minicbor::encode::unsigned_cbor_len(#array_len) }];
            let mut sorted: Vec<_> = active.iter().map(|f| (f.index.val(), *f)).collect();
            sorted.sort_by_key(|(i, _)| *i);
            let mut prev = 0i64;
            for (idx, f) in sorted {
                let gap = (idx - prev) as usize;
                if gap > 0 {
                    parts.push(quote!(#gap));
                }
                let ty = &f.typ;
                let tag = tag_size(&f.attrs);
                parts.push(quote! { #tag + <#ty as minicbor::MaxCborLen>::MAX_CBOR_LEN });
                prev = idx + 1;
            }
            Ok(quote! { #(#parts)+* })
        }
    }
}

fn tag_size(a: &Attributes) -> proc_macro2::TokenStream {
    if let Some(t) = a.tag() {
        quote!(minicbor::encode::unsigned_cbor_len(#t))
    } else {
        quote!(0)
    }
}

/// Add `MaxCborLen` bounds to type params appearing in the given fields.
///
/// Reuses `collect_type_params` from the crate root to find which type
/// parameters appear in the field types, then adds the bound.
fn add_bounds(generics: &mut syn::Generics, fields: &Fields) {
    let used = collect_type_params(generics, fields.fields());
    let bound: syn::TypeParamBound = syn::parse_quote!(minicbor::MaxCborLen);
    for p in generics.type_params_mut() {
        if used.contains(&p.ident) && !p.bounds.iter().any(|b| *b == bound) {
            p.bounds.push(bound.clone());
        }
    }
}
