use crate::Mode;
use crate::{add_bound_to_type_params, collect_type_params, is_option};
use crate::attrs::{Attributes, CustomCodec, Encoding, Level};
use crate::fields::Fields;
use crate::variants::Variants;
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

    let name     = &inp.ident;
    let attrs    = Attributes::try_from_iter(Level::Struct, inp.attrs.iter())?;
    let encoding = attrs.encoding().unwrap_or_default();
    let fields   = Fields::try_from(name.span(), data.fields.iter())?;

    let encode_fns: Vec<Option<CustomCodec>> = fields.attrs.iter()
        .map(|a| a.codec().cloned().filter(CustomCodec::is_encode))
        .collect();

    // Collect type parameters which should not have an `Encode` bound added,
    // i.e. from fields which have a custom encode function defined.
    let blacklist = {
        let iter = data.fields.iter()
            .zip(&encode_fns)
            .filter_map(|(f, ff)| ff.is_some().then(|| f));
        collect_type_params(&inp.generics, iter)
    };

    {
        let bound  = gen_encode_bound()?;
        let params = inp.generics.type_params_mut();
        add_bound_to_type_params(bound, params, &blacklist, &fields.attrs, Mode::Encode);
    }

    let (impl_generics, typ_generics, where_clause) = inp.generics.split_for_impl();

    // If transparent, just forward the encode call to the inner type.
    if attrs.transparent() {
        if fields.len() != 1 {
            let msg = "#[cbor(transparent)] requires a struct with one field";
            return Err(syn::Error::new(inp.ident.span(), msg))
        }
        let f = data.fields.iter().next().expect("struct has 1 field");
        let a = fields.attrs.first().expect("struct has 1 field");
        return make_transparent_impl(&inp.ident, f, a, impl_generics, typ_generics, where_clause)
    }

    let statements = encode_fields(&fields, true, encoding, &encode_fns)?;

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

    let name          = &inp.ident;
    let enum_attrs    = Attributes::try_from_iter(Level::Enum, inp.attrs.iter())?;
    let enum_encoding = enum_attrs.encoding().unwrap_or_default();
    let index_only    = enum_attrs.index_only();
    let variants      = Variants::try_from(name.span(), data.variants.iter())?;

    let mut blacklist = HashSet::new();
    let mut field_attrs = Vec::new();
    let mut rows = Vec::new();
    for ((var, idx), attrs) in data.variants.iter().zip(variants.indices.iter()).zip(&variants.attrs) {
        let fields = Fields::try_from(var.ident.span(), var.fields.iter())?;
        let encode_fns: Vec<Option<CustomCodec>> = fields.attrs.iter()
            .map(|a| a.codec().cloned().filter(CustomCodec::is_encode))
            .collect();
        // Collect type parameters which should not have an `Encode` bound added,
        // i.e. from fields which have a custom encode function defined.
        blacklist.extend({
            let iter = var.fields.iter()
                .zip(&encode_fns)
                .filter_map(|(f, ff)| ff.is_some().then(|| f));
            collect_type_params(&inp.generics, iter)
        });
        let con = &var.ident;
        let encoding = attrs.encoding().unwrap_or(enum_encoding);
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
            syn::Fields::Named(_) => {
                let statements = encode_fields(&fields, false, encoding, &encode_fns)?;
                let Fields { idents, .. } = fields;
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
            syn::Fields::Unnamed(_) => {
                let statements = encode_fields(&fields, false, encoding, &encode_fns)?;
                let Fields { idents, .. } = fields;
                quote! {
                    #name::#con(#(#idents,)*) => {
                        __e777.array(2)?;
                        __e777.u32(#idx)?;
                        #statements
                    }
                }
            }
        };
        field_attrs.extend_from_slice(&fields.attrs);
        rows.push(row)
    }

    {
        let bound  = gen_encode_bound()?;
        let params = inp.generics.type_params_mut();
        add_bound_to_type_params(bound, params, &blacklist, &field_attrs, Mode::Encode);
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
    ( fields: &Fields
    , has_self: bool
    , encoding: Encoding
    , encode_fns: &[Option<CustomCodec>]
    ) -> syn::Result<proc_macro2::TokenStream>
{
    let default_encode_fn: syn::ExprPath = syn::parse_str("minicbor::Encode::encode")?;

    let mut max_index = None;
    let mut tests = Vec::new();

    let iter = fields.pos.iter()
        .zip(fields.indices.iter()
            .zip(fields.idents.iter()
                .zip(fields.is_name.iter()
                    .zip(&fields.types))));

    match encoding {
        // Under array encoding the number of elements is the highest
        // index + 1. To determine the highest index we start from the
        // highest field whose type is not an `Option`, because it is
        // certain that all fields up to this point need to be encoded.
        // For each `Option` that follows the highest index is updated
        // if the value is a `Some`.
        Encoding::Array => {
            for field in iter.clone().rev() {
                let (i, (idx, (ident, (&is_name, typ)))) = field;
                let n = idx.val();
                if !is_option(typ, |_| true) {
                    max_index = Some(n);
                    break
                }
                let expr =
                    if has_self {
                        if is_name {
                            quote! {
                                if self.#ident.is_some() {
                                    __max_index777 = Some(#n)
                                }
                            }
                        } else {
                            let i = syn::Index::from(*i);
                            quote! {
                                if self.#i.is_some() {
                                    __max_index777 = Some(#n)
                                }
                            }
                        }
                    } else {
                        quote! {
                            if #ident.is_some() {
                                __max_index777 = Some(#n)
                            }
                        }
                    };
                tests.push(expr)
            }
            tests.reverse()
        }
        // Under map encoding the number of map entries is the number
        // of fields minus those which are an `Option` whose value is
        // `None`. Further down we define the total number of fields
        // and here for each `Option` we check if it is `None` and if
        // so substract 1 from the total.
        Encoding::Map => {
            for field in iter.clone() {
                let (i, (_idx, (ident, (&is_name, typ)))) = field;
                if !is_option(typ, |_| true) {
                    continue
                }
                let expr =
                    if has_self {
                        if is_name {
                            quote! {
                                if self.#ident.is_none() {
                                    __max_fields777 -= 1
                                }
                            }
                        } else {
                            let i = syn::Index::from(*i);
                            quote! {
                                if self.#i.is_none() {
                                    __max_fields777 -= 1
                                }
                            }
                        }
                    } else {
                        quote! {
                            if #ident.is_none() {
                                __max_fields777 -= 1
                            }
                        }
                    };
                tests.push(expr);
            }
        }
    }

    let mut statements = Vec::new();

    const IS_NAME: bool = true;
    const NO_NAME: bool = false;
    const HAS_SELF: bool = true;
    const NO_SELF: bool = false;
    const IS_OPT: bool = true;
    const NO_OPT: bool = false;
    const HAS_GAPS: bool = true;
    const NO_GAPS: bool = false;

    match encoding {
        // Under map encoding each field is encoded with its index.
        // If the field type is an `Option` and `None`, neither the
        // index nor the field value are encoded.
        Encoding::Map => for (field, encode_fn) in iter.zip(encode_fns) {
            let (i, (idx, (ident, (&is_name, typ)))) = field;
            let is_opt = is_option(typ, |_| true);
            let encode_fn = encode_fn.as_ref()
                .and_then(|f| f.to_encode_path())
                .unwrap_or_else(|| default_encode_fn.clone());
            let statement =
                match (is_name, has_self, is_opt) {
                    // struct
                    (IS_NAME, HAS_SELF, IS_OPT) => quote! {
                        if let Some(x) = &self.#ident {
                            __e777.u32(#idx)?;
                            #encode_fn(x, __e777)?
                        }
                    },
                    (IS_NAME, HAS_SELF, NO_OPT) => quote! {
                        __e777.u32(#idx)?;
                        #encode_fn(&self.#ident, __e777)?;
                    },
                    // tuple struct
                    (IS_NAME, NO_SELF, IS_OPT) => quote! {
                        if let Some(x) = #ident {
                            __e777.u32(#idx)?;
                            #encode_fn(x, __e777)?
                        }
                    },
                    (IS_NAME, NO_SELF, NO_OPT) => quote! {
                        __e777.u32(#idx)?;
                        #encode_fn(#ident, __e777)?;
                    },
                    // enum struct
                    (NO_NAME, HAS_SELF, IS_OPT) => {
                        let i = syn::Index::from(*i);
                        quote! {
                            if let Some(x) = &self.#i {
                                __e777.u32(#idx)?;
                                #encode_fn(x, __e777)?
                            }
                        }
                    }
                    (NO_NAME, HAS_SELF, NO_OPT) => {
                        let i = syn::Index::from(*i);
                        quote! {
                            __e777.u32(#idx)?;
                            #encode_fn(&self.#i, __e777)?;
                        }
                    }
                    // enum tuple
                    (NO_NAME, NO_SELF, IS_OPT) => quote! {
                        if let Some(x) = #ident {
                            __e777.u32(#idx)?;
                            #encode_fn(x, __e777)?
                        }
                    },
                    (NO_NAME, NO_SELF, NO_OPT) => quote! {
                        __e777.u32(#idx)?;
                        #encode_fn(#ident, __e777)?;
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
            for (field, encode_fn) in iter.zip(encode_fns) {
                let (i, (idx, (ident, (&is_name, typ)))) = field;
                let is_opt = is_option(typ, |_| true);
                let encode_fn = encode_fn.as_ref()
                    .and_then(|ff| ff.to_encode_path())
                    .unwrap_or_else(|| default_encode_fn.clone());
                let gaps = if first {
                    first = false;
                    idx.val() - k
                } else {
                    idx.val() - k - 1
                };

                let statement =
                    match (is_name, has_self, is_opt, gaps > 0) {
                        // struct
                        (IS_NAME, HAS_SELF, IS_OPT, HAS_GAPS) => quote! {
                            if Some(#idx) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #encode_fn(&self.#ident, __e777)?
                            }
                        },
                        (IS_NAME, HAS_SELF, IS_OPT, NO_GAPS) => quote! {
                            if Some(#idx) <= __max_index777 {
                                #encode_fn(&self.#ident, __e777)?
                            }
                        },
                        (IS_NAME, HAS_SELF, NO_OPT, HAS_GAPS) => quote! {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(&self.#ident, __e777)?;
                        },
                        (IS_NAME, HAS_SELF, NO_OPT, NO_GAPS) => quote! {
                            #encode_fn(&self.#ident, __e777)?;
                        },
                        // enum struct
                        (IS_NAME, NO_SELF, IS_OPT, HAS_GAPS) => quote! {
                            if Some(#idx) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #encode_fn(#ident, __e777)?
                            }
                        },
                        (IS_NAME, NO_SELF, IS_OPT, NO_GAPS) => quote! {
                            if Some(#idx) <= __max_index777 {
                                #encode_fn(#ident, __e777)?
                            }
                        },
                        (IS_NAME, NO_SELF, NO_OPT, HAS_GAPS) => quote! {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(#ident, __e777)?;
                        },
                        (IS_NAME, NO_SELF, NO_OPT, NO_GAPS) => quote! {
                            #encode_fn(#ident, __e777)?;
                        },
                        // tuple struct
                        (NO_NAME, HAS_SELF, IS_OPT, HAS_GAPS) => {
                            let i = syn::Index::from(*i);
                            quote! {
                                if Some(#idx) <= __max_index777 {
                                    for _ in 0 .. #gaps {
                                        __e777.null()?;
                                    }
                                    #encode_fn(&self.#i, __e777)?
                                }
                            }
                        }
                        (NO_NAME, HAS_SELF, IS_OPT, NO_GAPS) => {
                            let i = syn::Index::from(*i);
                            quote! {
                                if Some(#idx) <= __max_index777 {
                                    #encode_fn(&self.#i, __e777)?
                                }
                            }
                         }
                        (NO_NAME, HAS_SELF, NO_OPT, HAS_GAPS) => {
                            let i = syn::Index::from(*i);
                            quote! {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #encode_fn(&self.#i, __e777)?;
                            }
                        }
                        (NO_NAME, HAS_SELF, NO_OPT, NO_GAPS) => {
                            let i = syn::Index::from(*i);
                            quote! {
                                #encode_fn(&self.#i, __e777)?;
                            }
                        }
                        // enum tuple
                        (NO_NAME, NO_SELF, IS_OPT, HAS_GAPS) => quote! {
                            if Some(#idx) <= __max_index777 {
                                for _ in 0 .. #gaps {
                                    __e777.null()?;
                                }
                                #encode_fn(#ident, __e777)?
                            }
                        },
                        (NO_NAME, NO_SELF, IS_OPT, NO_GAPS) => quote! {
                            if Some(#idx) <= __max_index777 {
                                #encode_fn(#ident, __e777)?
                            }
                        },
                        (NO_NAME, NO_SELF, NO_OPT, HAS_GAPS) => quote! {
                            for _ in 0 .. #gaps {
                                __e777.null()?;
                            }
                            #encode_fn(#ident, __e777)?;
                        },
                        (NO_NAME, NO_SELF, NO_OPT, NO_GAPS) => quote! {
                            #encode_fn(#ident, __e777)?;
                        }
                    };
                statements.push(statement);
                k = idx.val()
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

/// Forward the encoding because of a `#[cbor(transparent)]` attribute.
fn make_transparent_impl
    ( name: &syn::Ident
    , field: &syn::Field
    , attrs: &Attributes
    , impl_generics: syn::ImplGenerics
    , typ_generics: syn::TypeGenerics
    , where_clause: Option<&syn::WhereClause>
    ) -> syn::Result<proc_macro2::TokenStream>
{
    if attrs.codec().map(CustomCodec::is_encode).unwrap_or(false) {
        let msg  = "`encode_with` or `with` not allowed with #[cbor(transparent)]";
        let span = field.ident.as_ref().map(|i| i.span()).unwrap_or_else(|| field.ty.span());
        return Err(syn::Error::new(span, msg))
    }

    let ident =
        if let Some(id) = &field.ident {
            quote!(#id)
        } else {
            let id = syn::Index::from(0);
            quote!(#id)
        };

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

fn gen_encode_bound() -> syn::Result<syn::TypeParamBound> {
    syn::parse_str("minicbor::Encode")
}

