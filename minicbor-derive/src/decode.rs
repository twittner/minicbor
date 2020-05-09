use crate::{check_uniq, field_indices, index_number, is_cow, is_option, variant_indices};
use crate::{Idx, lifetimes_to_constrain, is_str, is_byte_slice};
use quote::quote;
use syn::spanned::Spanned;

/// Entry point to derive `minicbor::Decode` on structs and enums.
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    let lifetime = match add_decode_bound(&mut input.generics) {
        Ok(lifetime) => lifetime,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error())
    };
    let result = match &input.data {
        syn::Data::Struct(s) => on_struct(&input, s, lifetime),
        syn::Data::Enum(e)   => on_enum(&input, e, lifetime),
        syn::Data::Union(u)  => {
            let msg = "deriving `minicbor::Decode` for a `union` is not supported";
            Err(syn::Error::new(u.union_token.span(), msg))
        }
    };
    proc_macro::TokenStream::from(result.unwrap_or_else(|e| e.to_compile_error()))
}

/// Create a `Decode` impl for (tuple) structs.
fn on_struct(inp: &syn::DeriveInput, s: &syn::DataStruct, mut lt: syn::LifetimeDef) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;
    let indices = field_indices(s.fields.iter())?;
    check_uniq(name.span(), indices.iter().cloned())?;

    let (field_names, field_types) = fields(s.fields.iter());
    for l in lifetimes_to_constrain(indices.iter().zip(field_types.iter())) {
        lt.bounds.push(l.clone())
    }
    let field_str = field_names.iter().map(|n| format!("{}::{}", name, n));
    let numbers = field_indices(s.fields.iter())?;
    let statements = gen_statements(&field_names, &field_types, &numbers);

    let g = add_lifetime(&inp.generics, lt);
    let (impl_generics , ..) = g.split_for_impl();
    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    let result = if let syn::Fields::Named(_) = s.fields {
        quote! {
            Ok(#name {
                #(#field_names : if let Some(x) = #field_names {
                    x
                } else {
                    return Err(minicbor::decode::Error::MissingValue(#numbers, #field_str))
                }),*
            })
        }
    } else if let syn::Fields::Unit = &s.fields {
        quote!(Ok(#name))
    } else {
        quote! {
            Ok(#name(#(if let Some(x) = #field_names {
                x
            } else {
                return Err(minicbor::decode::Error::MissingValue(#numbers, #field_str))
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
fn on_enum(inp: &syn::DeriveInput, e: &syn::DataEnum, mut lt: syn::LifetimeDef) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;
    check_uniq(e.enum_token.span(), variant_indices(e.variants.iter())?)?;

    let mut rows = Vec::new();
    for var in e.variants.iter() {
        let con = &var.ident;
        let idx = index_number(var.ident.span(), &var.attrs)?;
        let indices = field_indices(var.fields.iter())?;
        check_uniq(con.span(), indices.iter().cloned())?;
        let row = if let syn::Fields::Unit = &var.fields {
            quote!(#idx => {
                __d777.skip()?;
                Ok(#name::#con)
            })
        } else {
            let (field_names, field_types) = fields(var.fields.iter());
            for l in lifetimes_to_constrain(indices.iter().zip(field_types.iter())) {
                lt.bounds.push(l.clone())
            }
            let field_str = field_names.iter().map(|n| format!("{}::{}::{}", name, con, n));
            let numbers = field_indices(var.fields.iter())?;
            let statements = gen_statements(&field_names, &field_types, &numbers);
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

    let g = add_lifetime(&inp.generics, lt);
    let (impl_generics , ..) = g.split_for_impl();
    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'__b777> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'__b777>) -> Result<#name #typ_generics, minicbor::decode::Error> {
                if Some(2) != __d777.array()? {
                    return Err(minicbor::decode::Error::Message("expected enum (2-element array)"))
                }
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
// Then we iterate over all CBOR map elements and if an index `j` equal
// to `i` is found, we attempt to decode the next CBOR item as a value `v`
// of type `t`. If successful, we assign the result to `n` as `Some(v)`,
// otherwise we error, or -- if `t` is an option and the decoding failed
// because an unknown enum variant was decoded -- we skip the variant
// value and continue decoding.
//
// --------------------------------------------------------------------
// [1]: These variables will later be deconstructed in `on_enum` and
// `on_struct` and their inner value will be used to initialise a field.
// If not present, an error will be produced.
fn gen_statements(names: &[syn::Ident], types: &[syn::Type], numbers: &[Idx]) -> proc_macro2::TokenStream {
    assert_eq!(names.len(), types.len());
    assert_eq!(types.len(), numbers.len());

    let inits = types.iter().map(|ty| {
        if is_option(ty, |_| true) {
            quote!(Some(None))
        } else {
            quote!(None)
        }
    });

    let actions = numbers.iter().zip(names.iter().zip(types.iter())).map(|(ix, (name, ty))| {
        if is_option(ty, |_| true) {
            return quote! {
                match minicbor::Decode::decode(__d777) {
                    Ok(value) => #name = Some(value),
                    Err(minicbor::decode::Error::UnknownVariant(_)) => { __d777.skip()? }
                    Err(e) => return Err(e)
                }
            }
        }

        if ix.is_b() && is_cow(ty, |t| is_str(t) || is_byte_slice(t)) {
            return quote! {
                match minicbor::Decode::decode(__d777) {
                    Ok(value) => #name = Some(std::borrow::Cow::Borrowed(value)),
                    Err(minicbor::decode::Error::UnknownVariant(_)) => { __d777.skip()? }
                    Err(e) => return Err(e)
                }
            }
        }

        quote!({ #name = Some(minicbor::Decode::decode(__d777)?) })
    })
    .collect::<Vec<_>>();

    quote! {
        #(let mut #names : Option<#types> = #inits;)*

        if let Some(__len) = __d777.array()? {
            for i in 0 .. __len {
                match i {
                    #(#numbers => #actions)*
                    _          => __d777.skip()?
                }
            }
        } else {
            let mut i = 0;
            while minicbor::data::Type::Break != __d777.datatype()? {
                match i {
                    #(#numbers => #actions)*
                    _          => __d777.skip()?
                }
                i += 1
            }
            __d777.skip()?
        }
    }
}

/// Map fields to a list of field names and field types.
fn fields<'a, I>(iter: I) -> (Vec<syn::Ident>, Vec<syn::Type>)
where
    I: Iterator<Item = &'a syn::Field> + Clone
{
    let names = iter.clone().enumerate().map(|(i, f)| {
        match &f.ident {
            Some(n) => n.clone(),
            None    => quote::format_ident!("_{}", i)
        }
    })
    .collect();

    let types = iter.map(|f| f.ty.clone()).collect();

    (names, types)
}

/// Add a `minicbor::Decode` bound to every type parameter.
fn add_decode_bound(g: &mut syn::Generics) -> syn::Result<syn::LifetimeDef> {
    let bound: syn::TypeParamBound = syn::parse_str("minicbor::Decode<'__b777>")?;
    for t in g.type_params_mut() {
        t.bounds.push(bound.clone())
    }
    syn::parse_str("'__b777")
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
