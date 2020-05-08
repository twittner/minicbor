use crate::{check_uniq, field_indices, Idx, index_number, is_option, variant_indices};
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

/// Create an `Encode` impl for (tuple) structs.
fn on_struct(inp: &syn::DeriveInput, s: &syn::DataStruct) -> syn::Result<proc_macro2::TokenStream> {
    let name = &inp.ident;

    check_uniq(name.span(), field_indices(s.fields.iter())?)?;

    let mut fields = s.fields.iter()
        .enumerate()
        .map(|(i, f)| index_number(f.span(), &f.attrs).map(move |n| (i, n, f)))
        .collect::<Result<Vec<_>, _>>()?;
    fields.sort_unstable_by_key(|(_, n, _)| n.val());

    let statements = encode_as_array(&fields, true);
    let (impl_generics, typ_generics, where_clause) = inp.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics minicbor::Encode for #name #typ_generics #where_clause {
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                #statements
            }
        }
    })
}

/// Create an `Encode` impl for enums.
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
                    __e777.array(2)?;
                    __e777.u64(#idx)?;
                    __e777.array(0)?;
                    Ok(())
                }
            },
            syn::Fields::Named(fields) => {
                let idents = fields.named.iter().map(|f| f.ident.clone().unwrap());
                let mut fields = fields.named.iter()
                    .enumerate()
                    .map(|(i, f)| index_number(f.span(), &f.attrs).map(move |n| (i, n, f)))
                    .collect::<Result<Vec<_>, _>>()?;
                fields.sort_unstable_by_key(|(_, n, _)| n.val());
                let statements = encode_as_array(&fields, false);
                quote! {
                    #name::#con{#(#idents,)*} => {
                        __e777.array(2)?;
                        __e777.u64(#idx)?;
                        #statements
                    }
                }
            }
            syn::Fields::Unnamed(fields) => {
                let idents = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    quote::format_ident!("_{}", i)
                });
                let mut fields = fields.unnamed.iter()
                    .enumerate()
                    .map(|(i, f)| index_number(f.span(), &f.attrs).map(move |n| (i, n, f)))
                    .collect::<Result<Vec<_>, _>>()?;
                fields.sort_unstable_by_key(|(_, n, _)| n.val());
                let statements = encode_as_array(&fields, false);
                quote! {
                    #name::#con(#(#idents,)*) => {
                        __e777.array(2)?;
                        __e777.u64(#idx)?;
                        #statements
                    }
                }
            }
        };
        rows.push(row)
    }

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
            fn encode<__W777>(&self, __e777: &mut minicbor::Encoder<__W777>) -> Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write
            {
                #body
            }
        }
    })
}

fn encode_as_array(fields: &[(usize, Idx, &syn::Field)], has_self: bool) -> proc_macro2::TokenStream {
    let mut max_index = 0;
    let mut tests = Vec::new();

    for j in (0 .. fields.len()).rev() {
        let (i, n, f) = fields[j];
        let n = n.val();
        if !is_option(&f.ty, |_| true) {
            max_index = n;
            break
        }
        tests.push(match &f.ident {
            Some(name) if has_self => quote! {
                if self.#name.is_some() {
                    __max_index777 = #n
                }
            },
            Some(name) => quote! {
                if #name.is_some() {
                    __max_index777 = #n
                }
            },
            None if has_self => {
                let i = syn::Index::from(i);
                quote! {
                    if self.#i.is_some() {
                        __max_index777 = #n
                    }
                }
            }
            None => {
                let i = quote::format_ident!("_{}", i);
                quote! {
                    if #i.is_some() {
                        __max_index777 = #n
                    }
                }
            }
        })
    }

    tests.reverse();

    let mut statements = Vec::new();

    let mut first = true;
    let mut k = 0;
    for (i, n, f) in fields {
        let is_opt = is_option(&f.ty, |_| true);
        let gaps = if first {
            first = false;
            n.val() - k
        } else {
            n.val() - k - 1
        };
        let statement = match &f.ident {
            Some(name) if has_self && is_opt && gaps > 0 => quote! {
                if #n <= __max_index777 {
                    for _ in 0 .. #gaps {
                        __e777.null()?;
                    }
                    self.#name.encode(__e777)?
                }
            },
            Some(name) if has_self && gaps > 0 => quote! {
                for _ in 0 .. #gaps {
                    __e777.null()?;
                }
                self.#name.encode(__e777)?;
            },
            Some(name) if has_self => quote! {
                self.#name.encode(__e777)?;
            },
            Some(name) if is_opt && gaps > 0 => quote! {
                if #n <= __max_index777 {
                    for _ in 0 .. #gaps {
                        __e777.null()?;
                    }
                    #name.encode(__e777)?
                }
            },
            Some(name) if gaps > 0 => quote! {
                for _ in 0 .. #gaps {
                    __e777.null()?;
                }
                #name.encode(__e777)?;
            },
            Some(name) => quote! {
                #name.encode(__e777)?;
            },

            None if has_self && is_opt && gaps > 0 => {
                let i = syn::Index::from(*i);
                quote! {
                    if #n <= __max_index777 {
                        for _ in 0 .. #gaps {
                            __e777.null()?;
                        }
                        self.#i.encode(__e777)?
                    }
                }
            }
            None if has_self && gaps > 0 => {
                let i = syn::Index::from(*i);
                quote! {
                    for _ in 0 .. #gaps {
                        __e777.null()?;
                    }
                    self.#i.encode(__e777)?;
                }
            }
            None if has_self => {
                let i = syn::Index::from(*i);
                quote! {
                    self.#i.encode(__e777)?;
                }
            }

            None if is_opt && gaps > 0 => {
                let i = quote::format_ident!("_{}", i);
                quote! {
                    if #n <= __max_index777 {
                        for _ in 0 .. #gaps {
                            __e777.null()?;
                        }
                        #i.encode(__e777)?
                    }
                }
            }
            None if gaps > 0 => {
                let i = quote::format_ident!("_{}", i);
                quote! {
                    for _ in 0 .. #gaps {
                        __e777.null()?;
                    }
                    #i.encode(__e777)?;
                }
            }
            None => {
                let i = quote::format_ident!("_{}", i);
                quote! {
                    #i.encode(__e777)?;
                }
            }
        };
        statements.push(statement);
        k = n.val()
    }

    quote! {
        let mut __max_index777 = #max_index;

        #(#tests)*

        __e777.array(__max_index777 + 1)?;

        #(#statements)*

        Ok(())
    }
}

/// Add a `minicbor::Encode` bound to every type parameter.
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

