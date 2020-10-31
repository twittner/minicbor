use crate::{check_uniq, field_indices, index_number, is_cow, is_option, variant_indices};
use crate::{Idx, lifetimes_to_constrain, is_str, is_byte_slice, encoding, Encoding};
use crate::{collect_type_params, CustomCodec, custom_codec};
use crate::find_cbor_attr;
use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;

/// Entry point to derive `minicbor::Decode` on structs and enums.
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    let result = match &input.data {
        syn::Data::Struct(_) => on_struct(&mut input),
        syn::Data::Enum(_)   => on_enum(&mut input),
        syn::Data::Union(u)  => {
            let msg = "deriving `minicbor::Decode` for a `union` is not supported";
            Err(syn::Error::new(u.union_token.span(), msg))
        }
    };
    proc_macro::TokenStream::from(result.unwrap_or_else(|e| e.to_compile_error()))
}

/// Create a `Decode` impl for (tuple) structs.
fn on_struct(inp: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let data =
        if let syn::Data::Struct(data) = &inp.data {
            data
        } else {
            unreachable!("`derive_from` matched against `syn::Data::Struct`")
        };

    let name = &inp.ident;
    let indices = field_indices(data.fields.iter())?;
    check_uniq(name.span(), indices.iter().cloned())?;

    if find_cbor_attr(inp.attrs.iter(), "index_only", false)?.is_some() {
        return Err(syn::Error::new(inp.span(), "index_only is not supported on structs"))
    }

    let (field_names, field_types, decode_fns) = fields(data.fields.iter())?;
    let mut lifetime = decode_lifetime()?;
    for l in lifetimes_to_constrain(indices.iter().zip(field_types.iter())) {
        lifetime.bounds.push(l.clone())
    }
    // Collect type parameters which should not have an `Decode` bound added.
    let blacklist = collect_type_params(&inp.generics, data.fields.iter().zip(&decode_fns).filter_map(|(f, ff)| {
        if let Some(CustomCodec::Decode(_)) | Some(CustomCodec::Both(_)) = ff {
            Some(f)
        } else {
            None
        }
    }));
    add_decode_bound(&blacklist, inp.generics.type_params_mut())?;
    let g = add_lifetime(&inp.generics, lifetime);
    let (impl_generics , ..) = g.split_for_impl();
    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    // If transparent, just forward the decode call to the inner type.
    if find_cbor_attr(inp.attrs.iter(), "transparent", false)?.is_some() {
        return make_transparent_impl(inp, data, impl_generics, typ_generics, where_clause)
    }

    let field_str = field_names.iter().map(|n| format!("{}::{}", name, n));
    let encoding = inp.attrs.iter().filter_map(encoding).next().unwrap_or_default();
    let statements = gen_statements(&field_names, &field_types, &indices, &decode_fns, encoding)?;

    let result = if let syn::Fields::Named(_) = data.fields {
        quote! {
            Ok(#name {
                #(#field_names : if let Some(x) = #field_names {
                    x
                } else {
                    return Err(minicbor::decode::Error::MissingValue(#indices, #field_str))
                }),*
            })
        }
    } else if let syn::Fields::Unit = &data.fields {
        quote!(Ok(#name))
    } else {
        quote! {
            Ok(#name(#(if let Some(x) = #field_names {
                x
            } else {
                return Err(minicbor::decode::Error::MissingValue(#indices, #field_str))
            }),*))
        }
    };

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'__b777> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'__b777>) -> Result<#name #typ_generics, minicbor::decode::Error> {
                #statements
                #result
            }
        }
    })
}

/// Create a `Decode` impl for enums.
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

    let enum_encoding = inp.attrs.iter().filter_map(encoding).next().unwrap_or_default();
    let mut blacklist = HashSet::new();
    let mut lifetime = decode_lifetime()?;
    let mut rows = Vec::new();
    for var in data.variants.iter() {
        let con = &var.ident;
        let idx = index_number(var.ident.span(), &var.attrs)?;
        let indices = field_indices(var.fields.iter())?;
        check_uniq(con.span(), indices.iter().cloned())?;
        let row = if let syn::Fields::Unit = &var.fields {
            if index_only {
                quote!(#idx => Ok(#name::#con),)
            } else {
                quote!(#idx => {
                    __d777.skip()?;
                    Ok(#name::#con)
                })
            }
        } else {
            let (field_names, field_types, decode_fns) = fields(var.fields.iter())?;
            for l in lifetimes_to_constrain(indices.iter().zip(field_types.iter())) {
                lifetime.bounds.push(l.clone())
            }
            let field_str = field_names.iter().map(|n| format!("{}::{}::{}", name, con, n));
            let numbers = field_indices(var.fields.iter())?;
            let encoding = var.attrs.iter().filter_map(encoding).next().unwrap_or(enum_encoding);
            // Collect type parameters which should not have an `Decode` bound added.
            blacklist.extend(collect_type_params(&inp.generics, var.fields.iter().zip(&decode_fns).filter_map(|(f, ff)| {
                if let Some(CustomCodec::Decode(_)) | Some(CustomCodec::Both(_)) = ff {
                    Some(f)
                } else {
                    None
                }
            })));
            let statements = gen_statements(&field_names, &field_types, &numbers, &decode_fns, encoding)?;
            if let syn::Fields::Named(_) = var.fields {
                quote! {
                    #idx => {
                        #statements
                        Ok(#name::#con {
                            #(#field_names : if let Some(x) = #field_names {
                                x
                            } else {
                                return Err(minicbor::decode::Error::MissingValue(#numbers, #field_str))
                            }),*
                        })
                    }
                }
            } else {
                quote! {
                    #idx => {
                        #statements
                        Ok(#name::#con(#(if let Some(x) = #field_names {
                            x
                        } else {
                            return Err(minicbor::decode::Error::MissingValue(#numbers, #field_str))
                        }),*))
                    }
                }
            }
        };
        rows.push(row)
    }

    add_decode_bound(&blacklist, inp.generics.type_params_mut())?;
    let g = add_lifetime(&inp.generics, lifetime);
    let (impl_generics , ..) = g.split_for_impl();
    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();


    let check = if index_only {
        quote!()
    } else {
        quote! {
            if Some(2) != __d777.array()? {
                return Err(minicbor::decode::Error::Message("expected enum (2-element array)"))
            }
        }
    };

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'__b777> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'__b777>) -> Result<#name #typ_generics, minicbor::decode::Error> {
                #check
                match __d777.u32()? {
                    #(#rows)*
                    n => Err(minicbor::decode::Error::UnknownVariant(n))
                }
            }
        }
    })
}

/// Generate decoding statements for every item.
//
// For every name `n`, type `t` and index `i` we declare a local mutable
// variable `n` with type `Option<t>` and set it to `None` if `t` is not
// an `Option`, otherwise to `Some(None)`. [1]
//
// Then -- depending on the selected encoding -- we iterate over all CBOR
// map or array elements and if an index `j` equal to `i` is found, we
// attempt to decode the next CBOR item as a value `v` of type `t`. If
// successful, we assign the result to `n` as `Some(v)`, otherwise we
// error, or -- if `t` is an option and the decoding failed because an
// unknown enum variant was decoded -- we skip the variant value and
// continue decoding.
//
// --------------------------------------------------------------------
// [1]: These variables will later be deconstructed in `on_enum` and
// `on_struct` and their inner value will be used to initialise a field.
// If not present, an error will be produced.
fn gen_statements
    ( names: &[syn::Ident]
    , types: &[syn::Type]
    , numbers: &[Idx]
    , decode_fns: &[Option<CustomCodec>]
    , encoding: Encoding
    ) -> syn::Result<proc_macro2::TokenStream>
{
    assert_eq!(names.len(), types.len());
    assert_eq!(types.len(), numbers.len());
    assert_eq!(numbers.len(), decode_fns.len());

    let default_decode_fn: syn::ExprPath = syn::parse_str("minicbor::Decode::decode")?;

    let inits = types.iter().map(|ty| {
        if is_option(ty, |_| true) {
            quote!(Some(None))
        } else {
            quote!(None)
        }
    });

    let actions = numbers.iter().zip(names.iter().zip(types.iter().zip(decode_fns)))
        .map(|(ix, (name, (ty, ff)))| {
            let decode_fn = ff.as_ref()
                .and_then(|ff| ff.to_decode_path())
                .unwrap_or_else(|| default_decode_fn.clone());
            if is_option(ty, |_| true) {
                return quote! {
                    match #decode_fn(__d777) {
                        Ok(value) => #name = Some(value),
                        Err(minicbor::decode::Error::UnknownVariant(_)) => { __d777.skip()? }
                        Err(e) => return Err(e)
                    }
                }
            }
            if ix.is_b() && is_cow(ty, |t| is_str(t) || is_byte_slice(t)) {
                return quote! {
                    match #decode_fn(__d777) {
                        Ok(value) => #name = Some(std::borrow::Cow::Borrowed(value)),
                        Err(minicbor::decode::Error::UnknownVariant(_)) => { __d777.skip()? }
                        Err(e) => return Err(e)
                    }
                }
            }
            quote!({ #name = Some(#decode_fn(__d777)?) })
    })
    .collect::<Vec<_>>();

    Ok(match encoding {
        Encoding::Array => quote! {
            #(let mut #names : Option<#types> = #inits;)*

            if let Some(__len777) = __d777.array()? {
                for __i777 in 0 .. __len777 {
                    match __i777 {
                        #(#numbers => #actions)*
                        _          => __d777.skip()?
                    }
                }
            } else {
                let mut __i777 = 0;
                while minicbor::data::Type::Break != __d777.datatype()? {
                    match __i777 {
                        #(#numbers => #actions)*
                        _          => __d777.skip()?
                    }
                    __i777 += 1
                }
                __d777.skip()?
            }
        },
        Encoding::Map => quote! {
            #(let mut #names : Option<#types> = #inits;)*

            if let Some(__len777) = __d777.map()? {
                for _ in 0 .. __len777 {
                    match __d777.u32()? {
                        #(#numbers => #actions)*
                        _          => __d777.skip()?
                    }
                }
            } else {
                while minicbor::data::Type::Break != __d777.datatype()? {
                    match __d777.u32()? {
                        #(#numbers => #actions)*
                        _          => __d777.skip()?
                    }
                }
                __d777.skip()?
            }
        }
    })
}

/// Map fields to a list of field names and field types.
fn fields<'a, I>(iter: I) -> syn::Result<(Vec<syn::Ident>, Vec<syn::Type>, Vec<Option<CustomCodec>>)>
where
    I: Iterator<Item = &'a syn::Field> + Clone
{
    let names = iter.clone()
        .enumerate()
        .map(|(i, f)| {
            match &f.ident {
                Some(n) => n.clone(),
                None    => quote::format_ident!("_{}", i)
            }
        })
        .collect();

    let types = iter.clone()
        .map(|f| f.ty.clone())
        .collect();

    let decode_fns = iter
        .map(|f| {
            for a in &f.attrs {
                if let Some(ff) = custom_codec(a)? {
                    if ff.is_decode() {
                        return Ok(Some(ff))
                    }
                }
            }
            Ok(None)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok((names, types, decode_fns))
}

/// Generate the '__b777 lifetime.
fn decode_lifetime() -> syn::Result<syn::LifetimeDef> {
    syn::parse_str("'__b777")
}

/// Add a `minicbor::Decode` bound to every type parameter.
fn add_decode_bound<'a, I>(blacklist: &HashSet<syn::TypeParam>, iter: I) -> syn::Result<()>
where
    I: Iterator<Item = &'a mut syn::TypeParam>
{
    let bound: syn::TypeParamBound = syn::parse_str("minicbor::Decode<'__b777>")?;
    for t in iter.filter(|t| !blacklist.contains(t)) {
        t.bounds.push(bound.clone())
    }
    Ok(())
}

/// Return a modified clone of `syn::Generics` with the given lifetime
/// parameter put before the other type and lifetime parameters.
///
/// This will be used later when splitting the parameters so that the
/// additional lifetime is only present in the `impl` parameter section.
fn add_lifetime(g: &syn::Generics, l: syn::LifetimeDef) -> syn::Generics {
    let mut g2 = g.clone();
    g2.params = Some(l.into()).into_iter().chain(g2.params).collect();
    g2
}

/// Forward the decoding because of a `#[cbor(transparent)]` attribute.
fn make_transparent_impl
    ( input: &syn::DeriveInput
    , data: &syn::DataStruct
    , impl_generics: syn::ImplGenerics
    , typ_generics: syn::TypeGenerics
    , where_clause: Option<&syn::WhereClause>
    ) -> syn::Result<proc_macro2::TokenStream>
{
    if data.fields.len() != 1 {
        let msg = "#[cbor(transparent)] requires a struct with one field";
        return Err(syn::Error::new(input.ident.span(), msg))
    }

    let field = data.fields.iter().next().expect("struct has one field");

    if let Some(a) = find_cbor_attr(field.attrs.iter(), "decode_with", true)? {
        let msg = "#[cbor(decode_with)] not allowed within #[cbor(transparent)]";
        return Err(syn::Error::new(a.span(), msg))
    }

    if let Some(a) = find_cbor_attr(field.attrs.iter(), "with", true)? {
        let msg = "#[cbor(with)] not allowed within #[cbor(transparent)]";
        return Err(syn::Error::new(a.span(), msg))
    }

    let name = &input.ident;

    let call =
        if let Some(id) = &field.ident {
            quote! {
                Ok(#name { #id: minicbor::Decode::decode(__d777)? })
            }
        } else {
            quote! {
                Ok(#name(minicbor::Decode::decode(__d777)?))
            }
        };

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'__b777> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'__b777>) -> Result<#name #typ_generics, minicbor::decode::Error> {
                #call
            }
        }
    })
}

