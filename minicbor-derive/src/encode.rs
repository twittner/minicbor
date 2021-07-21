#![allow(clippy::type_complexity)]

use crate::{check_uniq, field_indices, Idx, index_number, is_option, variant_indices};
use crate::{collect_type_params, encoding, Encoding, custom_codec, CustomCodec};
use crate::find_cbor_attr;
use quote::quote;
use std::{collections::HashSet, convert::TryInto};
use syn::spanned::Spanned;

/// Entry point to derive `minicbor::Encode` on structs and enums.
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    let result = match &input.data {
        syn::Data::Struct(_) => on_struct(&mut input),
        syn::Data::Enum(_)   => on_enum(&mut input),
        syn::Data::Union(u)  => {
            let msg = "deriving `minicbor::Encode` for a `union` is not supported";
            Err(syn::Error::new(u.union_token.span(), msg))
        }
    };
    proc_macro::TokenStream::from(result.unwrap_or_else(|e| e.to_compile_error()))
}

/// Create an `Encode` impl for (tuple) structs.
fn on_struct(inp: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let data =
        if let syn::Data::Struct(data) = &inp.data {
            data
        } else {
            unreachable!("`derive_from` matched against `syn::Data::Struct`")
        };

    let name = &inp.ident;
    check_uniq(name.span(), field_indices(data.fields.iter())?)?;

    let fields = sorted_fields(data.fields.iter())?;
    let encoding = inp.attrs.iter().find_map(encoding).unwrap_or_default();

    if find_cbor_attr(inp.attrs.iter(), "index_only", false)?.is_some() {
        return Err(syn::Error::new(inp.span(), "index_only is not supported on structs"))
    }

    // Collect type parameters which should not have an `Encode` bound added.
    let blacklist = collect_type_params(&inp.generics, fields.iter().filter_map(|(.., f, ff)| {
        if let Some(CustomCodec::Encode(_)) | Some(CustomCodec::Both(_)) = ff {
            Some(*f)
        } else {
            None
        }
    }));
    add_encode_bound(&blacklist, inp.generics.type_params_mut())?;

    // If transparent, just forward the encode call to the inner type.
    if find_cbor_attr(inp.attrs.iter(), "transparent", false)?.is_some() {
        return make_transparent_impl(inp, data)
    }

    let (impl_generics, typ_generics, where_clause) = inp.generics.split_for_impl();

    let statements = encode_fields(&fields, true, encoding)?;

    Ok(quote! {
        impl #impl_generics minicbor::Encode for #name #typ_generics #where_clause {
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                #statements
            }
        }
    })
}

/// Create an `Encode` impl for enums.
fn on_enum(inp: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let data =
        if let syn::Data::Enum(data) = &inp.data {
            data
        } else {
            unreachable!("`derive_from` matched against `syn::Data::Enum`")
        };

    let name = &inp.ident;
    check_uniq(data.enum_token.span(), variant_indices(data.variants.iter())?)?;

    let index_only = find_cbor_attr(inp.attrs.iter(), "index_only", false)?.is_some();

    let enum_encoding = inp.attrs.iter().find_map(encoding).unwrap_or_default();
    let mut blacklist = HashSet::new();
    let mut rows = Vec::new();
    for var in data.variants.iter() {
        let con = &var.ident;
        let idx = index_number(var.ident.span(), &var.attrs)?;
        check_uniq(con.span(), field_indices(var.fields.iter())?)?;
        let encoding = var.attrs.iter().find_map(encoding).unwrap_or(enum_encoding);
        let row = match &var.fields {
            syn::Fields::Unit => match encoding {
                Encoding::Array | Encoding::Map if index_only => quote! {
                    #name::#con => {
                        __e777.u32(#idx)?;
                        Ok(())
                    }
                },
                Encoding::Array => quote! {
                    #name::#con => {
                        __e777.array(2)?;
                        __e777.u32(#idx)?;
                        __e777.array(0)?;
                        Ok(())
                    }
                },
                Encoding::Map => quote! {
                    #name::#con => {
                        __e777.array(2)?;
                        __e777.u32(#idx)?;
                        __e777.map(0)?;
                        Ok(())
                    }
                }
            }
            syn::Fields::Named(f) if index_only => {
                return Err(syn::Error::new(f.span(), "index_only enums must not have fields"))
            }
            syn::Fields::Named(fields) => {
                let idents = fields.named.iter().map(|f| f.ident.clone().unwrap());
                let fields = sorted_fields(fields.named.iter())?;
                // Collect type parameters which should not have an `Encode` bound added.
                blacklist.extend(collect_type_params(&inp.generics, fields.iter().filter_map(|(.., f, ff)| {
                    if let Some(CustomCodec::Encode(_)) | Some(CustomCodec::Both(_)) = ff {
                        Some(*f)
                    } else {
                        None
                    }
                })));
                let statements = encode_fields(&fields, false, encoding)?;
                quote! {
                    #name::#con{#(#idents,)*} => {
                        __e777.array(2)?;
                        __e777.u32(#idx)?;
                        #statements
                    }
                }
            }
            syn::Fields::Unnamed(f) if index_only => {
                return Err(syn::Error::new(f.span(), "index_only enums must not have fields"))
            }
            syn::Fields::Unnamed(fields) => {
                let idents = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    quote::format_ident!("_{}", i)
                });
                let fields = sorted_fields(fields.unnamed.iter())?;
                // Collect type parameters which should not have an `Encode` bound added.
                blacklist.extend(collect_type_params(&inp.generics, fields.iter().filter_map(|(.., f, ff)| {
                    if let Some(CustomCodec::Encode(_)) | Some(CustomCodec::Both(_)) = ff {
                        Some(*f)
                    } else {
                        None
                    }
                })));
                let statements = encode_fields(&fields, false, encoding)?;
                quote! {
                    #name::#con(#(#idents,)*) => {
                        __e777.array(2)?;
                        __e777.u32(#idx)?;
                        #statements
                    }
                }
            }
        };
        rows.push(row)
    }

    add_encode_bound(&blacklist, inp.generics.type_params_mut())?;
    let (impl_generics, typ_generics, where_clause) = inp.generics.split_for_impl();

    let body = if rows.is_empty() {
        quote! {
            unreachable!("empty type")
        }
    } else {
        quote! {
            match self {
                #(#rows)*
            }
        }
    };

    Ok(quote! {
        impl #impl_generics minicbor::Encode for #name #typ_generics #where_clause {
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                #body
            }
        }
    })
}

/// The encoding logic of fields.
///
/// We first generate code to determine at runtime the number of fields to
/// encode so that we can use regular map or array containers instead of
/// indefinite ones. Since this value depends on optional values being `Some`
/// we can not calculate this number statically but have the generate code
/// with runtime tests.
///
/// Then the actual field encoding happens which is slightly different
/// depending on the encoding.
///
/// NB: The `fields` parameter is assumed to be sorted by index.
fn encode_fields
    ( fields: &[(usize, Idx, &syn::Field, Option<CustomCodec>)]
    , has_self: bool
    , encoding: Encoding
    ) -> syn::Result<proc_macro2::TokenStream>
{
    let default_encode_fn: syn::ExprPath = syn::parse_str("minicbor::Encode::encode")?;

    let mut max_index = None;
    let mut tests = Vec::new();

    match encoding {
        // Under array encoding the number of elements is the highest
        // index + 1. To determine the highest index we start from the
        // highest field whose type is not an `Option`, because it is
        // certain that all fields up to this point need to be encoded.
        // For each `Option` that follows the highest index is updated
        // if the value is a `Some`.
        Encoding::Array => {
            for f in fields.iter().rev() {
                let (i, n, f, _) = f;
                let n = n.val();
                if !is_option(&f.ty, |_| true) {
                    max_index = Some(n);
                    break
                }
                tests.push(match &f.ident {
                    Some(name) if has_self => quote! {
                        if self.#name.is_some() {
                            __max_index777 = Some(#n)
                        }
                    },
                    Some(name) => quote! {
                        if #name.is_some() {
                            __max_index777 = Some(#n)
                        }
                    },
                    None if has_self => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if self.#i.is_some() {
                                __max_index777 = Some(#n)
                            }
                        }
                    }
                    None => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            if #i.is_some() {
                                __max_index777 = Some(#n)
                            }
                        }
                    }
                })
            }
            tests.reverse()
        }
        // Under map encoding the number of map entries is the number
        // of fields minus those which are an `Option` whose value is
        // `None`. Further down we define the total number of fields
        // and here for each `Option` we check if it is `None` and if
        // so substract 1 from the total.
        Encoding::Map => {
            for f in fields {
                let (i, _, f, _) = f;
                if !is_option(&f.ty, |_| true) {
                    continue
                }
                tests.push(match &f.ident {
                    Some(name) if has_self => quote! {
                        if self.#name.is_none() {
                            __max_fields777 -= 1
                        }
                    },
                    Some(name) => quote! {
                        if #name.is_none() {
                            __max_fields777 -= 1
                        }
                    },
                    None if has_self => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if self.#i.is_none() {
                                __max_fields777 -= 1
                            }
                        }
                    }
                    None => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            if #i.is_none() {
                                __max_fields777 -= 1
                            }
                        }
                    }
                })
            }
        }
    }

    let mut statements = Vec::new();

    match encoding {
        // Under map encoding each field is encoded with its index.
        // If the field type is an `Option` and `None`, neither the
        // index nor the field value are encoded.
        Encoding::Map => for (i, n, f, ff) in fields {
            let is_opt = is_option(&f.ty, |_| true);
            let encode_fn = ff.as_ref()
                .and_then(|ff| ff.to_encode_path())
                .unwrap_or_else(|| default_encode_fn.clone());
            let statement = match &f.ident {

                // struct

                Some(name) if has_self && is_opt => quote! {
                    if let Some(x) = &self.#name {
                        __e777.u32(#n)?;
                        #encode_fn(x, __e777)?
                    }
                },
                Some(name) if has_self => quote! {
                    __e777.u32(#n)?;
                    #encode_fn(&self.#name, __e777)?;
                },

                // tuple struct

                Some(name) if is_opt => quote! {
                    if let Some(x) = #name {
                        __e777.u32(#n)?;
                        #encode_fn(x, __e777)?
                    }
                },
                Some(name) => quote! {
                    __e777.u32(#n)?;
                    #encode_fn(#name, __e777)?;
                },

                // enum struct

                None if has_self && is_opt => {
                    let i = syn::Index::from(*i);
                    quote! {
                        if let Some(x) = &self.#i {
                            __e777.u32(#n)?;
                            #encode_fn(x, __e777)?
                        }
                    }
                }
                None if has_self => {
                    let i = syn::Index::from(*i);
                    quote! {
                        __e777.u32(#n)?;
                        #encode_fn(&self.#i, __e777)?;
                    }
                }

                // enum tuple

                None if is_opt => {
                    let i = quote::format_ident!("_{}", i);
                    quote! {
                        if let Some(x) = #i {
                            __e777.u32(#n)?;
                            #encode_fn(x, __e777)?
                        }
                    }
                }
                None => {
                    let i = quote::format_ident!("_{}", i);
                    quote! {
                        __e777.u32(#n)?;
                        #encode_fn(#i, __e777)?;
                    }
                }
            };
            statements.push(statement)
        }
        // Under array encoding only field values are encoded and their
        // index is represented as the array position. Gaps between indexes
        // need to be filled with null.
        // We do not encode the suffix of `Option` fields which are `None`
        // so we check for each field if it is still below the max. index,
        // otherwise we do not encode it.
        Encoding::Array => {
            let mut first = true;
            let mut k = 0;
            for (i, n, f, ff) in fields {
                let is_opt = is_option(&f.ty, |_| true);
                let encode_fn = ff.as_ref()
                    .and_then(|ff| ff.to_encode_path())
                    .unwrap_or_else(|| default_encode_fn.clone());
                let gaps = if first {
                    first = false;
                    n.val() - k
                } else {
                    n.val() - k - 1
                };
                let statement = match &f.ident {

                    // struct

                    Some(name) if has_self && is_opt && gaps > 0 => quote! {
                        if Some(#n) <= __max_index777 {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(&self.#name, __e777)?
                        }
                    },
                    Some(name) if has_self && is_opt => quote! {
                        if Some(#n) <= __max_index777 {
                            #encode_fn(&self.#name, __e777)?
                        }
                    },
                    Some(name) if has_self && gaps > 0 => quote! {
                        for _ in 0 .. #gaps {
                            __e777.null()?;
                        }
                        #encode_fn(&self.#name, __e777)?;
                    },
                    Some(name) if has_self => quote! {
                        #encode_fn(&self.#name, __e777)?;
                    },

                    // enum struct

                    Some(name) if is_opt && gaps > 0 => quote! {
                        if Some(#n) <= __max_index777 {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(#name, __e777)?
                        }
                    },
                    Some(name) if is_opt => quote! {
                        if Some(#n) <= __max_index777 {
                            #encode_fn(#name, __e777)?
                        }
                    },
                    Some(name) if gaps > 0 => quote! {
                        for _ in 0 .. #gaps {
                            __e777.null()?;
                        }
                        #encode_fn(#name, __e777)?;
                    },
                    Some(name) => quote! {
                        #encode_fn(#name, __e777)?;
                    },

                    // tuple struct

                    None if has_self && is_opt && gaps > 0 => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if Some(#n) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #encode_fn(&self.#i, __e777)?
                            }
                        }
                    }
                    None if has_self && is_opt => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if Some(#n) <= __max_index777 {
                                #encode_fn(&self.#i, __e777)?
                            }
                        }
                    }
                    None if has_self && gaps > 0 => {
                        let i = syn::Index::from(*i);
                        quote! {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(&self.#i, __e777)?;
                        }
                    }
                    None if has_self => {
                        let i = syn::Index::from(*i);
                        quote! {
                            #encode_fn(&self.#i, __e777)?;
                        }
                    }

                    // enum tuple

                    None if is_opt && gaps > 0 => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            if Some(#n) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #encode_fn(#i, __e777)?
                            }
                        }
                    }
                    None if is_opt => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            if Some(#n) <= __max_index777 {
                                #encode_fn(#i, __e777)?
                            }
                        }
                    }
                    None if gaps > 0 => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(#i, __e777)?;
                        }
                    }
                    None => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            #encode_fn(#i, __e777)?;
                        }
                    }
                };
                statements.push(statement);
                k = n.val()
            }
        }
    }

    let max_fields: u32 = fields.len().try_into()
        .map_err(|_| {
            let msg = "more than 2^32 fields are not supported";
            syn::Error::new(proc_macro2::Span::call_site(), msg)
        })?;

    let max_index =
        if let Some(i) = max_index {
            quote!(Some(#i))
        } else {
            quote!(None)
        };

    match encoding {
        Encoding::Array => Ok(quote! {
            let mut __max_index777: core::option::Option<u32> = #max_index;

            #(#tests)*

            if let Some(__i777) = __max_index777 {
                __e777.array(u64::from(__i777) + 1)?;
                #(#statements)*
            } else {
                __e777.array(0)?;
            }

            Ok(())
        }),
        Encoding::Map => Ok(quote! {
            let mut __max_fields777 = #max_fields;

            #(#tests)*

            __e777.map(u64::from(__max_fields777))?;

            #(#statements)*

            Ok(())
        })
    }
}

/// Add a `minicbor::Encode` bound to every type parameter.
///
/// However if the blacklist contains the type parameter, do not add the bound.
fn add_encode_bound<'a, I>(blacklist: &HashSet<syn::TypeParam>, iter: I) -> syn::Result<()>
where
    I: Iterator<Item = &'a mut syn::TypeParam>
{
    let bound: syn::TypeParamBound = syn::parse_str("minicbor::Encode")?;
    for p in iter.filter(|t| !blacklist.contains(t)) {
        p.bounds.push(bound.clone())
    }
    Ok(())
}

/// Collect the field indexes and sort the fields by index number.
fn sorted_fields<'a, I>(iter: I) -> syn::Result<Vec<(usize, Idx, &'a syn::Field, Option<CustomCodec>)>>
where
    I: Iterator<Item = &'a syn::Field>
{
    let mut fields = iter.enumerate()
        .map(|(i, f)| {
            index_number(f.span(), &f.attrs)
                .and_then(move |n| {
                    for a in &f.attrs {
                        if let Some(ff) = custom_codec(a)? {
                            if ff.is_encode() {
                                return Ok((i, n, f, Some(ff)))
                            }
                        }
                    }
                    Ok((i, n, f, None))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    fields.sort_unstable_by_key(|(_, n, _, _)| n.val());
    Ok(fields)
}

/// Forward the encoding because of a `#[cbor(transparent)]` attribute.
fn make_transparent_impl
    ( input: &syn::DeriveInput
    , data: &syn::DataStruct
    ) -> syn::Result<proc_macro2::TokenStream>
{
    if data.fields.len() != 1 {
        let msg = "#[cbor(transparent)] requires a struct with one field";
        return Err(syn::Error::new(input.ident.span(), msg))
    }

    let field = data.fields.iter().next().expect("struct has one field");

    if let Some(a) = find_cbor_attr(field.attrs.iter(), "encode_with", true)? {
        let msg = "#[cbor(encode_with)] not allowed within #[cbor(transparent)]";
        return Err(syn::Error::new(a.span(), msg))
    }

    if let Some(a) = find_cbor_attr(field.attrs.iter(), "with", true)? {
        let msg = "#[cbor(with)] not allowed within #[cbor(transparent)]";
        return Err(syn::Error::new(a.span(), msg))
    }

    let ident =
        if let Some(id) = &field.ident {
            quote!(#id)
        } else {
            let id = syn::Index::from(0);
            quote!(#id)
        };

    let (impl_generics, typ_generics, where_clause) = input.generics.split_for_impl();

    let name = &input.ident;

    Ok(quote! {
        impl #impl_generics minicbor::Encode for #name #typ_generics #where_clause {
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                self.#ident.encode(__e777)
            }
        }
    })
}

