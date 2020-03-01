use crate::{check_uniq, field_indices, index_number, is_option, variant_indices};
use quote::quote;
use syn::spanned::Spanned;

/// Entry point to derive `minicbor::Encode` on structs and enums.
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    if let Err(e) = add_encode_bound(&mut input.generics.type_params_mut()) {
        return proc_macro::TokenStream::from(e.to_compile_error())
    }
    let result = match &input.data {
        syn::Data::Struct(s) => on_struct(&input, s),
        syn::Data::Enum(e)   => on_enum(&input, e),
        syn::Data::Union(u)  => {
            let msg = "deriving `minicbor::Encode` for a `union` is not supported";
            Err(syn::Error::new(u.union_token.span(), msg))
        }
    };
    proc_macro::TokenStream::from(result.unwrap_or_else(|e| e.to_compile_error()))
}

/// Create `Encode` impl for (tuple) structs.
fn on_struct(inp: &syn::DeriveInput, s: &syn::DataStruct) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;
    check_uniq(name.span(), field_indices(s.fields.iter())?)?;
    let statements = encode_struct_fields(s.fields.iter())?;
    let (impl_generics, typ_generics, where_clause) = inp.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics minicbor::Encode for #name #typ_generics #where_clause {
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                #(#statements)*
            }
        }
    })
}

/// Create `Encode` impl for enums.
fn on_enum(inp: &syn::DeriveInput, e: &syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;
    check_uniq(e.enum_token.span(), variant_indices(e.variants.iter())?)?;
    let mut rows = Vec::new();
    for var in e.variants.iter() {
        let con = &var.ident;
        let idx = index_number(var.ident.span(), &var.attrs)?;
        check_uniq(con.span(), field_indices(var.fields.iter())?)?;
        let row = match &var.fields {
            syn::Fields::Unit => quote! {
                #name::#con => {
                    __e777.u32(#idx)?;
                    Ok(())
                }
            },
            syn::Fields::Named(fields) => {
                let idents = fields.named.iter().map(|f| f.ident.clone().unwrap());
                let statements = encode_enum_fields(fields.named.iter())?;
                quote! {
                    #name::#con{#(#idents,)*} => {
                        __e777.u32(#idx)?;
                        #(#statements)*
                    }
                }
            }
            syn::Fields::Unnamed(fields) => {
                let idents = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    quote::format_ident!("_{}", i)
                });
                let statements = encode_enum_fields(fields.unnamed.iter())?;
                quote! {
                    #name::#con(#(#idents,)*) => {
                        __e777.u32(#idx)?;
                        #(#statements)*
                    }
                }
            }
        };
        rows.push(row)
    }

    let (impl_generics, typ_generics, where_clause) = inp.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics minicbor::Encode for #name #typ_generics #where_clause {
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                match self {
                    #(#rows)*
                }
            }
        }
    })
}

fn encode_struct_fields<'a, I>(iter: I) -> syn::Result<Vec<proc_macro2::TokenStream>>
where
    I: Iterator<Item = &'a syn::Field>
{
    let mut num_fields = 0usize;
    let mut has_option = false;

    let encode = iter.enumerate().map(|(i, f)| -> syn::Result<_> {
        num_fields += 1;
        let num = index_number(f.span(), &f.attrs)?;
        let needs_if = is_option(&f.ty);
        has_option |= needs_if;
        Ok(match &f.ident {
            Some(name) if needs_if => quote! {
                if let Some(x) = &self.#name {
                    __e777.u32(#num)?;
                    x.encode(__e777)?
                }
            },
            Some(name) => quote! {
                __e777.u32(#num)?;
                self.#name.encode(__e777)?;
            },
            None if needs_if => {
                let i = syn::Index::from(i);
                quote! {
                    if let Some(x) = &self.#i {
                        __e777.u32(#num)?;
                        x.encode(__e777)?
                    }
                }
            }
            None => {
                let i = syn::Index::from(i);
                quote! {
                    __e777.u32(#num)?;
                    self.#i.encode(__e777)?;
                }
            }
        })
    })
    .collect::<Vec<_>>();

    encode_as_map(encode, if has_option { None } else { Some(num_fields) })
}

fn encode_enum_fields<'a, I>(iter: I) -> syn::Result<Vec<proc_macro2::TokenStream>>
where
    I: Iterator<Item = &'a syn::Field>
{
    let mut num_fields = 0usize;
    let mut has_option = false;

    let encode = iter.enumerate().map(|(i, f)| -> syn::Result<_> {
        num_fields += 1;
        let num = index_number(f.span(), &f.attrs)?;
        let needs_if = is_option(&f.ty);
        has_option |= needs_if;
        Ok(match &f.ident {
            Some(name) if needs_if => quote! {
                if let Some(x) = #name {
                    __e777.u32(#num)?;
                    x.encode(__e777)?
                }
            },
            Some(name) => quote! {
                __e777.u32(#num)?;
                #name.encode(__e777)?;
            },
            None if needs_if => {
                let i = quote::format_ident!("_{}", i);
                quote! {
                    if let Some(x) = #i {
                        __e777.u32(#num)?;
                        x.encode(__e777)?
                    }
                }
            }
            None => {
                let i = quote::format_ident!("_{}", i);
                quote! {
                    __e777.u32(#num)?;
                    #i.encode(__e777)?;
                }
            }
        })
    })
    .collect::<Vec<_>>();

    encode_as_map(encode, if has_option { None } else { Some(num_fields) })
}

fn encode_as_map<I>(iter: I, len: Option<usize>) -> syn::Result<Vec<proc_macro2::TokenStream>>
where
    I: IntoIterator<Item = syn::Result<proc_macro2::TokenStream>>
{
    let mut statements = Vec::new();
    if let Some(n) = len {
        statements.push(quote!(__e777.map(#n)?;))
    } else {
        statements.push(quote!(__e777.begin_map()?;))
    }
    for e in iter.into_iter() {
        statements.push(e?)
    }
    if len.is_none() {
        statements.push(quote!(__e777.end()?;))
    }
    statements.push(quote!(Ok(())));

    Ok(statements)
}

fn add_encode_bound<'a, I>(iter: I) -> syn::Result<()>
where
    I: Iterator<Item = &'a mut syn::TypeParam>
{
    let bound: syn::TypeParamBound = syn::parse_str("minicbor::Encode")?;
    for p in iter {
        p.bounds.push(bound.clone())
    }
    Ok(())
}

