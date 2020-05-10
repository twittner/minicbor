use crate::{check_uniq, field_indices, Idx, index_number, is_option, variant_indices};
use crate::{encoding, Encoding};
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

    let fields = sorted_fields(s.fields.iter())?;
    let encoding = inp.attrs.iter().filter_map(encoding).next().unwrap_or_default();
    let statements = encode_fields(&fields, true, encoding);
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
    let enum_encoding = inp.attrs.iter().filter_map(encoding).next().unwrap_or_default();
    let mut rows = Vec::new();
    for var in e.variants.iter() {
        let con = &var.ident;
        let idx = index_number(var.ident.span(), &var.attrs)?;
        check_uniq(con.span(), field_indices(var.fields.iter())?)?;
        let encoding = var.attrs.iter().filter_map(encoding).next().unwrap_or(enum_encoding);
        let row = match &var.fields {
            syn::Fields::Unit => match encoding {
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
            syn::Fields::Named(fields) => {
                let idents = fields.named.iter().map(|f| f.ident.clone().unwrap());
                let fields = sorted_fields(fields.named.iter())?;
                let statements = encode_fields(&fields, false, encoding);
                quote! {
                    #name::#con{#(#idents,)*} => {
                        __e777.array(2)?;
                        __e777.u32(#idx)?;
                        #statements
                    }
                }
            }
            syn::Fields::Unnamed(fields) => {
                let idents = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    quote::format_ident!("_{}", i)
                });
                let fields = sorted_fields(fields.unnamed.iter())?;
                let statements = encode_fields(&fields, false, encoding);
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

fn encode_fields(fields: &[(usize, Idx, &syn::Field)], has_self: bool, encoding: Encoding) -> proc_macro2::TokenStream {
    let mut max_index = None;
    let mut tests = Vec::new();

    match encoding {
        Encoding::Array => {
            for j in (0 .. fields.len()).rev() {
                let (i, n, f) = fields[j];
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
                        let i = syn::Index::from(i);
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
        Encoding::Map => {
            for j in 0 .. fields.len() {
                let (i, _, f) = fields[j];
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
                        let i = syn::Index::from(i);
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
        Encoding::Map => for (i, n, f) in fields {
            let is_opt = is_option(&f.ty, |_| true);
            let statement = match &f.ident {

                // struct

                Some(name) if has_self && is_opt => quote! {
                    if let Some(x) = &self.#name {
                        __e777.u32(#n)?;
                        x.encode(__e777)?
                    }
                },
                Some(name) if has_self => quote! {
                    __e777.u32(#n)?;
                    self.#name.encode(__e777)?;
                },

                // tuple struct

                Some(name) if is_opt => quote! {
                    if let Some(x) = #name {
                        __e777.u32(#n)?;
                        x.encode(__e777)?
                    }
                },
                Some(name) => quote! {
                    __e777.u32(#n)?;
                    #name.encode(__e777)?;
                },

                // enum struct

                None if has_self && is_opt => {
                    let i = syn::Index::from(*i);
                    quote! {
                        if let Some(x) = &self.#i {
                            __e777.u32(#n)?;
                            x.encode(__e777)?
                        }
                    }
                }
                None if has_self => {
                    let i = syn::Index::from(*i);
                    quote! {
                        __e777.u32(#n)?;
                        self.#i.encode(__e777)?;
                    }
                }

                // enum tuple

                None if is_opt => {
                    let i = quote::format_ident!("_{}", i);
                    quote! {
                        if let Some(x) = #i {
                            __e777.u32(#n)?;
                            x.encode(__e777)?
                        }
                    }
                }
                None => {
                    let i = quote::format_ident!("_{}", i);
                    quote! {
                        __e777.u32(#n)?;
                        #i.encode(__e777)?;
                    }
                }
            };
            statements.push(statement)
        }
        Encoding::Array => {
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

                    // struct

                    Some(name) if has_self && is_opt && gaps > 0 => quote! {
                        if Some(#n) <= __max_index777 {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            self.#name.encode(__e777)?
                        }
                    },
                    Some(name) if has_self && is_opt && gaps == 0 => quote! {
                        if Some(#n) <= __max_index777 {
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

                    // enum struct

                    Some(name) if is_opt && gaps > 0 => quote! {
                        if Some(#n) <= __max_index777 {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #name.encode(__e777)?
                        }
                    },
                    Some(name) if is_opt && gaps == 0 => quote! {
                        if Some(#n) <= __max_index777 {
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

                    // tuple struct

                    None if has_self && is_opt && gaps > 0 => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if Some(#n) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                self.#i.encode(__e777)?
                            }
                        }
                    }
                    None if has_self && is_opt && gaps == 0 => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if Some(#n) <= __max_index777 {
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

                    // enum tuple

                    None if is_opt && gaps > 0 => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            if Some(#n) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #i.encode(__e777)?
                            }
                        }
                    }
                    None if is_opt && gaps == 0 => {
                        let i = quote::format_ident!("_{}", i);
                        quote! {
                            if Some(#n) <= __max_index777 {
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
        }
    }

    let max_fields = fields.len() as u32;

    let max_index =
        if let Some(i) = max_index {
            quote!(Some(#i))
        } else {
            quote!(None)
        };

    match encoding {
        Encoding::Array => quote! {
            let mut __max_index777: Option<u32> = #max_index;

            #(#tests)*

            if let Some(__i777) = __max_index777 {
                __e777.array(__i777 as u64 + 1)?;
                #(#statements)*
            } else {
                __e777.array(0)?;
            }

            Ok(())
        },
        Encoding::Map => quote! {
            let mut __max_fields777 = #max_fields;

            #(#tests)*

            __e777.map(__max_fields777 as u64)?;

            #(#statements)*

            Ok(())
        }
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

fn sorted_fields<'a, I>(iter: I) -> syn::Result<Vec<(usize, Idx, &'a syn::Field)>>
where
    I: Iterator<Item = &'a syn::Field>
{
    let mut fields = iter.enumerate()
        .map(|(i, f)| index_number(f.span(), &f.attrs).map(move |n| (i, n, f)))
        .collect::<Result<Vec<_>, _>>()?;
    fields.sort_unstable_by_key(|(_, n, _)| n.val());
    Ok(fields)
}
