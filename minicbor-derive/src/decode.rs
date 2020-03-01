use crate::{check_uniq, field_indices, index_number, is_option, variant_indices};
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

/// Create `Decode` impl for (tuple) structs.
fn on_struct(inp: &syn::DeriveInput, s: &syn::DataStruct, lt: syn::LifetimeDef) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;
    check_uniq(name.span(), field_indices(s.fields.iter())?)?;

    let g = add_lifetime(&inp.generics, lt);
    let (impl_generics , ..) = g.split_for_impl();
    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    let (field_names, field_types) = fields(s.fields.iter());
    let field_str = field_names.iter().map(|n| format!("{}::{}", name, n));
    let numbers = field_indices(s.fields.iter())?;
    let statements = gen_statements(&field_names, &field_types, &numbers);

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
            fn decode<__R777>(__d777: &mut minicbor::Decoder<'__b777, __R777>)
                -> Result<#name #typ_generics, minicbor::decode::Error<__R777::Error>>
            where
                __R777: minicbor::decode::Read<'__b777>
            {
                #statements
                #result
            }
        }
    })
}

/// Create `Decode` impl for enums.
fn on_enum(inp: &syn::DeriveInput, e: &syn::DataEnum, lt: syn::LifetimeDef) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;
    check_uniq(e.enum_token.span(), variant_indices(e.variants.iter())?)?;

    let g = add_lifetime(&inp.generics, lt);
    let (impl_generics , ..) = g.split_for_impl();
    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    let mut rows = Vec::new();
    for var in e.variants.iter() {
        let con = &var.ident;
        let idx = index_number(var.ident.span(), &var.attrs)?;
        check_uniq(con.span(), field_indices(var.fields.iter())?)?;
        let row = if let syn::Fields::Unit = &var.fields {
            quote!(#idx => { Ok(#name::#con) })
        } else {
            let (field_names, field_types) = fields(var.fields.iter());
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

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'__b777> for #name #typ_generics #where_clause {
            fn decode<__R777>(__d777: &mut minicbor::Decoder<'__b777, __R777>)
                -> Result<#name #typ_generics, minicbor::decode::Error<__R777::Error>>
            where
                __R777: minicbor::decode::Read<'__b777>
            {
                match __d777.u32()? {
                    #(#rows)*
                    n => Err(minicbor::decode::Error::UnknownVariant(n))
                }
            }
        }
    })
}

fn gen_statements(names: &Vec<syn::Ident>, types: &Vec<syn::Type>, numbers: &Vec<u32>) -> proc_macro2::TokenStream {
    let inits = types.iter().map(|ty| {
        if is_option(ty) {
            quote!(Some(None))
        } else {
            quote!(None)
        }
    });
    quote! {
        #(let mut #names : Option<#types> = #inits;)*

        if let Some(__len) = __d777.map()? {
            for _ in 0 .. __len {
                match __d777.u32()? {
                    #(#numbers => { #names = Some(minicbor::Decode::decode(__d777)?) })*
                    _          => __d777.skip()?
                }
            }
        } else {
            while minicbor::data::Type::Break != __d777.datatype()? {
                match __d777.u32()? {
                    #(#numbers => { #names = Some(minicbor::Decode::decode(__d777)?) })*
                    _          => __d777.skip()?
                }
            }
            __d777.skip()?
        }
    }
}

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

fn add_decode_bound(g: &mut syn::Generics) -> syn::Result<syn::LifetimeDef> {
    let bound: syn::TypeParamBound = syn::parse_str("minicbor::Decode<'__b777>")?;
    let lstatic: syn::Lifetime = syn::parse_str("'static")?;
    let mut lifetime: syn::LifetimeDef = syn::parse_str("'__b777")?;
    for l in g.lifetimes() {
        if l.lifetime == lstatic {
            continue
        }
        lifetime.bounds.push(l.lifetime.clone())
    }
    for t in g.type_params_mut() {
        t.bounds.push(bound.clone())
    }
    Ok(lifetime)
}

fn add_lifetime(g: &syn::Generics, l: syn::LifetimeDef) -> syn::Generics {
    let mut g2 = g.clone();
    g2.params = Some(l.into()).into_iter().chain(g2.params).collect();
    g2
}
