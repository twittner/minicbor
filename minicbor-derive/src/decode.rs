use std::collections::BTreeSet;

use quote::quote;
use syn::spanned::Spanned;

use crate::attrs::codec::PathOrDefault;
use crate::blacklist::Blacklist;
use crate::{collect_type_params, Mode};
use crate::{add_bound_to_type_params, is_cow, is_option, is_str, is_byte_slice};
use crate::{add_typeparam, gen_ctx_param};
use crate::attrs::{Attributes, CustomCodec, Encoding, Level};
use crate::fields::{Field, Fields};
use crate::variants::Variants;
use crate::lifetimes::{gen_lifetime, lifetimes_to_constrain, add_lifetime};

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

    let name   = &inp.ident;
    let attrs  = Attributes::try_from_iter(Level::Struct, inp.attrs.iter())?;
    let fields = Fields::try_from(name.span(), data.fields.iter(), &[&attrs])?;

    let mut lifetime = gen_lifetime();
    for l in lifetimes_to_constrain(fields.fields().map(|f| (&f.index, f.attrs.borrow(), &f.typ))) {
        if !lifetime.bounds.iter().any(|b| *b == l) {
            lifetime.bounds.push(l.clone())
        }
    }

    let blacklist = Blacklist::full(Mode::Decode, &fields, &inp.generics);
    let bound  = gen_decode_bound();
    let params = inp.generics.type_params_mut();
    add_bound_to_type_params(Mode::Decode, bound, params, &blacklist, fields.fields().attributes());

    // Collect type parameters which require a `Default` bound.
    let default_types: BTreeSet<syn::Ident> =
        collect_type_params(&inp.generics, fields.fields().chain(fields.skipped()).filter(|f| {
            f.attrs.default() || f.attrs.skip() || f.attrs.skip_if_codec()
        }));

    let blacklist = Blacklist::empty()
        .with_mode(Mode::Decode, &fields, &inp.generics)
        .with_phantoms(&fields, &inp.generics);

    let bound  = gen_default_bound();
    let params = inp.generics.type_params_mut().filter(|p| default_types.contains(&p.ident));
    add_bound_to_type_params(Mode::Decode, bound, params, &blacklist,
        fields.fields().chain(fields.skipped()).filter_map(|f| {
            (f.attrs.default() || f.attrs.skip() || f.attrs.skip_if_codec()).then_some(&f.attrs)
        })
    );

    let generics = add_lifetime(&inp.generics, lifetime);
    let generics = add_typeparam(&generics, gen_ctx_param(), attrs.context_bound());
    let impl_generics = generics.split_for_impl().0;

    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    // If transparent, just forward the decode call to the inner type.
    if attrs.transparent() {
        if fields.fields().len() != 1 {
            let msg = "#[cbor(transparent)] requires a struct with one field";
            return Err(syn::Error::new(inp.ident.span(), msg))
        }
        let f = fields.fields().next().expect("struct has 1 field");
        return make_transparent_impl(&inp.ident, f, impl_generics, typ_generics, where_clause)
    }

    let statements = gen_statements(&fields, attrs.encoding().unwrap_or_default(), false)?;

    let result = if let syn::Fields::Named(_) = data.fields {
        let defs      = defs(fields.fields());
        let nils      = nils(fields.fields());
        let indices   = fields.fields().indices();
        let idents    = fields.fields().idents();
        let field_str = fields.fields().idents().map(|n| format!("{name}::{n}"));
        let skipped   = fields.skipped().idents();
        let funs      = fields.fields().indices().map(|i| {
            if i.is_num() {
                quote!(missing_value)
            } else {
                quote!(missing_value_str)
            }
        });
        quote! {
            Ok(#name {
                #(#idents : if let Some(x) = #idents {
                    x
                } else if let Some(def) = #defs {
                    def
                } else if let Some(z) = #nils {
                    z
                } else {
                    return Err(minicbor::decode::Error::#funs(#indices).with_message(#field_str).at(__p777))
                },)*
                #(#skipped : Default::default(),)*
            })
        }
    } else if let syn::Fields::Unit = data.fields {
        quote!(Ok(#name))
    } else {
        let expr = field_inits(&name.to_string(), &fields);
        quote! {
            Ok(#name(#expr))
        }
    };

    let tag = decode_tag(&attrs);

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'bytes, Ctx> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'bytes>, __ctx777: &mut Ctx) -> core::result::Result<#name #typ_generics, minicbor::decode::Error> {
                #tag
                let __p777 = __d777.position();
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

    let name          = &inp.ident;
    let enum_attrs    = Attributes::try_from_iter(Level::Enum, inp.attrs.iter())?;
    let enum_encoding = enum_attrs.encoding().unwrap_or_default();
    let index_only    = enum_attrs.index_only();
    let flat          = enum_attrs.flat();
    let variants      = Variants::try_from(name.span(), data.variants.iter(), &enum_attrs)?;

    let mut decode_blacklist = Blacklist::empty();
    let mut field_attrs = Vec::new();
    let mut default_blacklist = Blacklist::empty();
    let mut defaults = BTreeSet::new();
    let mut default_attrs = Vec::new();
    let mut lifetime = gen_lifetime();
    let mut num_rows = Vec::new();
    let mut str_rows = Vec::new();
    for ((var, idx), attrs) in data.variants.iter().zip(variants.indices.iter()).zip(&variants.attrs) {
        let fields = Fields::try_from(var.ident.span(), var.fields.iter(), &[attrs, &enum_attrs])?;
        let encoding = attrs.encoding().unwrap_or(enum_encoding);
        let con = &var.ident;
        let tag = decode_tag(attrs);
        let row = if let syn::Fields::Unit = var.fields {
            if index_only | flat {
                quote!(#idx => Ok(#name::#con),)
            } else {
                quote!(#idx => {
                    #tag
                    __d777.skip()?;
                    Ok(#name::#con)
                })
            }
        } else {
            for l in lifetimes_to_constrain(fields.fields().map(|f| (&f.index, f.attrs.borrow(), &f.typ))) {
                if !lifetime.bounds.iter().any(|b| *b == l) {
                    lifetime.bounds.push(l.clone())
                }
            }
            decode_blacklist.merge(&fields, &inp.generics, Blacklist::full(Mode::Decode, &fields, &inp.generics));
            default_blacklist.merge(&fields, &inp.generics, Blacklist::empty()
                .with_mode(Mode::Decode, &fields, &inp.generics)
                .with_phantoms(&fields, &inp.generics));
            defaults.extend(
                collect_type_params(&inp.generics, fields.fields().chain(fields.skipped()).filter(|f| {
                    f.attrs.default() || f.attrs.skip() || f.attrs.skip_if_codec()
                }))
            );
            let statements = gen_statements(&fields, encoding, flat)?;
            if let syn::Fields::Named(_) = var.fields {
                let defs      = defs(fields.fields());
                let nils      = nils(fields.fields());
                let indices   = fields.fields().indices();
                let idents    = fields.fields().idents();
                let field_str = fields.fields().idents().map(|n| format!("{name}::{con}::{n}"));
                let skipped   = fields.skipped().idents();
                let funs      = fields.fields().indices().map(|i| {
                    if i.is_num() {
                        quote!(missing_value)
                    } else {
                        quote!(missing_value_str)
                    }
                });
                quote! {
                    #idx => {
                        #tag
                        #statements
                        Ok(#name::#con {
                            #(#idents : if let Some(x) = #idents {
                                x
                            } else if let Some(def) = #defs {
                                def
                            } else if let Some(z) = #nils {
                                z
                            } else {
                                return Err(minicbor::decode::Error::#funs(#indices).with_message(#field_str).at(__p777))
                            },)*
                            #(#skipped : Default::default(),)*
                        })
                    }
                }
            } else {
                let pref = format!("{name}::{con}");
                let expr = field_inits(&pref, &fields);
                quote! {
                    #idx => {
                        #tag
                        #statements
                        Ok(#name::#con(#expr))
                    }
                }
            }
        };
        field_attrs.extend(fields.fields().attributes().cloned());
        default_attrs.extend(fields.fields().attributes().chain(fields.skipped().attributes()).cloned());
        if idx.is_str() {
            str_rows.push(row)
        } else {
            num_rows.push(row)
        }
    }

    let bound  = gen_decode_bound();
    let params = inp.generics.type_params_mut();
    add_bound_to_type_params(Mode::Decode, bound, params, &decode_blacklist, &field_attrs);

    let bound  = gen_default_bound();
    let params = inp.generics.type_params_mut().filter(|p| defaults.contains(&p.ident));
    add_bound_to_type_params(Mode::Decode, bound, params, &default_blacklist,
        default_attrs.iter().filter(|a| {
            a.default() || a.skip() || a.skip_if_codec()
        })
    );

    let generics = add_lifetime(&inp.generics, lifetime);
    let generics = add_typeparam(&generics, gen_ctx_param(), enum_attrs.context_bound());
    let impl_generics = generics.split_for_impl().0;

    let (_, typ_generics, where_clause) = inp.generics.split_for_impl();

    let check = if index_only {
        quote! {
            let __p778 = __d777.position();
        }
    } else if flat {
        quote! {
            let __p777 = __d777.position();
            let Some(__len777) = __d777.array()? else {
                return Err(minicbor::decode::Error::message("flat enum requires definite-length array").at(__p777))
            };
            if __len777 == 0 {
                return Err(minicbor::decode::Error::message("flat enum requires non-empty array").at(__p777))
            }
            let __p778 = __d777.position();
        }
    } else {
        quote! {
            let __p777 = __d777.position();
            if Some(2) != __d777.array()? {
                return Err(minicbor::decode::Error::message("expected enum (2-element array)").at(__p777))
            }
            let __p778 = __d777.position();
        }
    };

    let tag = decode_tag(&enum_attrs);

    let match_fragement = if str_rows.is_empty() {
        quote! {
            match __d777.i64()? {
                #(#num_rows)*
                n => Err(minicbor::decode::Error::unknown_variant(n).at(__p778))
            }
        }
    } else {
        quote! {
            if matches!(__d777.datatype()?, minicbor::data::Type::String) {
                match __d777.str()? {
                    #(#str_rows)*
                    s => Err(minicbor::__minicbor_cfg! {
                        'std {
                            minicbor::decode::Error::unknown_variant_str(s.to_string()).at(__p778)
                        }
                        'alloc {
                            minicbor::decode::Error::unknown_variant_str(s.to_string()).at(__p778)
                        }
                        'otherwise {
                            minicbor::decode::Error::unknown_variant_str().at(__p778)
                        }
                    })
                }
            } else {
                match __d777.i64()? {
                    #(#num_rows)*
                    n => Err(minicbor::decode::Error::unknown_variant(n).at(__p778))
                }
            }
        }
    };

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'bytes, Ctx> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'bytes>, __ctx777: &mut Ctx) -> core::result::Result<#name #typ_generics, minicbor::decode::Error> {
                #tag
                #check
                #match_fragement
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
fn gen_statements(fields: &Fields, encoding: Encoding, flat: bool) -> syn::Result<proc_macro2::TokenStream> {
    let default_decode_fn: syn::ExprPath = syn::parse_quote!(minicbor::Decode::decode);

    let actions = fields.fields().map(|field| {
        let decode_fn = field.attrs.codec()
            .and_then(CustomCodec::to_decode_path)
            .unwrap_or_else(|| default_decode_fn.clone());

        let unknown_var_err =
            if let Some(cd) = field.attrs.codec() {
                if let Some(expr) = cd.to_nil_expr() {
                    let ty = &field.typ;
                    let nil = match expr {
                        PathOrDefault::Path(p) => quote!(#p()),
                        PathOrDefault::Default => quote!(Some(Default::default()))
                    };
                    quote! {
                        Err(e) if e.is_unknown_variant() && {
                            let __nil777: Option<#ty> = #nil;
                            __nil777.is_some()
                        } => {
                            __d777.skip()?
                        }
                    }
                } else if is_option(&field.typ, |_| true) {
                    quote! {
                        Err(e) if e.is_unknown_variant() => __d777.skip()?,
                    }
                } else {
                    quote!()
                }
            } else if is_option(&field.typ, |_| true) {
                quote! {
                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                }
            } else {
                let ty = &field.typ;
                quote! {
                    Err(e) if e.is_unknown_variant() && <#ty as minicbor::Decode::<Ctx>>::nil().is_some() => {
                        __d777.skip()?
                    }
                }
            };

            let value =
                if (field.attrs.borrow().is_some() || field.index.is_b())
                    && is_cow(&field.typ, |t| is_str(t) || is_byte_slice(t))
                {
                    quote!(minicbor::__minicbor_cfg! {
                        'std { Some(std::borrow::Cow::Borrowed(__v777)) }
                        'alloc { Some(alloc::borrow::Cow::Borrowed(__v777)) }
                        'otherwise { Some(__v777) }
                    })
                } else {
                    quote!(Some(__v777))
                };

            let tag  = decode_tag(&field.attrs);
            let name = &field.ident;

            quote! {{
                #tag
                match #decode_fn(__d777, __ctx777) {
                    Ok(__v777) => #name = #value,
                    #unknown_var_err
                    Err(e) => return Err(e)
                }
            }}
    })
    .collect::<Vec<_>>();

    let inits = fields.fields().types().map(|ty| {
        if is_option(ty, |_| true) {
            quote!(Some(None))
        } else {
            quote!(None)
        }
    });

    let idents  = fields.fields().idents();
    let types   = fields.fields().types();
    let indices = fields.fields().indices().collect::<Vec<_>>();

    Ok(match encoding {
        Encoding::Array if flat => quote! {
            #(let mut #idents : core::option::Option<#types> = #inits;)*

            for __i777 in 0 .. __len777 - 1 {
                match __i777 {
                    #(#indices => #actions)*
                    _          => __d777.skip()?
                }
            }
        },
        Encoding::Array => quote! {
            #(let mut #idents : core::option::Option<#types> = #inits;)*

            if let Some(__len777) = __d777.array()? {
                for __i777 in 0 .. __len777 {
                    match __i777 {
                        #(#indices => #actions)*
                        _          => __d777.skip()?
                    }
                }
            } else {
                let mut __i777 = 0;
                while minicbor::data::Type::Break != __d777.datatype()? {
                    match __i777 {
                        #(#indices => #actions)*
                        _          => __d777.skip()?
                    }
                    __i777 += 1
                }
                __d777.skip()?
            }
        },
        Encoding::Map if fields.has_str_index() => {
            let (numerics, strings): (Vec<_>, Vec<_>) = fields
                .fields()
                .partition(|f| f.index.is_num());

            let num_inits = numerics.iter().map(|f| {
                if is_option(&f.typ, |_| true) {
                    quote!(Some(None))
                } else {
                    quote!(None)
                }
            });
            let num_idents  = numerics.iter().map(|f| &f.ident);
            let num_types   = numerics.iter().map(|f| &f.typ);
            let num_indices = numerics.iter().map(|f| &f.index).collect::<Vec<_>>();

            let str_inits = strings.iter().map(|f| {
                if is_option(&f.typ, |_| true) {
                    quote!(Some(None))
                } else {
                    quote!(None)
                }
            });
            let str_idents  = strings.iter().map(|f| &f.ident);
            let str_types   = strings.iter().map(|f| &f.typ);
            let str_indices = strings.iter().map(|f| &f.index).collect::<Vec<_>>();

            let (num_actions, str_actions): (Vec<_>, Vec<_>) = actions
                .into_iter()
                .zip(fields.fields())
                .partition(|(_, f)| f.index.is_num());
            let num_actions = num_actions.into_iter().map(|(a, _)| a).collect::<Vec<_>>();
            let str_actions = str_actions.into_iter().map(|(a, _)| a).collect::<Vec<_>>();

            quote! {
                #(let mut #num_idents : core::option::Option<#num_types> = #num_inits;)*
                #(let mut #str_idents : core::option::Option<#str_types> = #str_inits;)*

                if let Some(__len777) = __d777.map()? {
                    for _ in 0 .. __len777 {
                        if matches!(__d777.datatype()?, minicbor::data::Type::String) {
                            match __d777.str()? {
                                #(#str_indices => #str_actions)*
                                _              => __d777.skip()?
                            }
                        } else {
                            match __d777.i64()? {
                                #(#num_indices => #num_actions)*
                                _              => __d777.skip()?
                            }
                        }
                    }
                } else {
                    while minicbor::data::Type::Break != __d777.datatype()? {
                        if matches!(__d777.datatype()?, minicbor::data::Type::String) {
                            match __d777.str()? {
                                #(#str_indices => #str_actions)*
                                _              => __d777.skip()?
                            }
                        } else {
                            match __d777.i64()? {
                                #(#num_indices => #num_actions)*
                                _              => __d777.skip()?
                            }
                        }
                    }
                    __d777.skip()?
                }
            }
        }
        Encoding::Map => quote! {
            #(let mut #idents : core::option::Option<#types> = #inits;)*

            if let Some(__len777) = __d777.map()? {
                for _ in 0 .. __len777 {
                    match __d777.i64()? {
                        #(#indices => #actions)*
                        _          => __d777.skip()?
                    }
                }
            } else {
                while minicbor::data::Type::Break != __d777.datatype()? {
                    match __d777.i64()? {
                        #(#indices => #actions)*
                        _          => __d777.skip()?
                    }
                }
                __d777.skip()?
            }
        }
    })
}

/// Forward the decoding because of a `#[cbor(transparent)]` attribute.
fn make_transparent_impl
    ( name: &syn::Ident
    , field: &Field
    , impl_generics: syn::ImplGenerics
    , typ_generics: syn::TypeGenerics
    , where_clause: Option<&syn::WhereClause>
    ) -> syn::Result<proc_macro2::TokenStream>
{
    let default_decode_fn: syn::ExprPath = syn::parse_quote!(minicbor::Decode::decode);
    let default_nil_fn: syn::ExprPath = syn::parse_quote!(minicbor::Decode::<Ctx>::nil);

    let decode_fn = field.attrs.codec()
        .filter(|cc| cc.is_decode())
        .and_then(CustomCodec::to_decode_path)
        .unwrap_or_else(|| default_decode_fn.clone());

    let decode_call =
        if (field.attrs.borrow().is_some() || field.index.is_b())
            && is_cow(&field.typ, |t| is_str(t) || is_byte_slice(t))
        {
            quote!(minicbor::__minicbor_cfg! {
                'std {
                    match #decode_fn(__d777, __ctx777) {
                        Ok(v)  => std::borrow::Cow::Borrowed(v),
                        Err(e) => return Err(e)
                    }
                }
                'alloc {
                    match #decode_fn(__d777, __ctx777) {
                        Ok(v)  => alloc::borrow::Cow::Borrowed(v),
                        Err(e) => return Err(e)
                    }
                }
                'otherwise {
                    #decode_fn(__d777, __ctx777)?
                }
            })
        } else {
            quote! {
                #decode_fn(__d777, __ctx777)?
            }
        };

    let decode_body =
        if field.is_name {
            let id = &field.ident;
            quote! {
                Ok(#name { #id: #decode_call })
            }
        } else {
            quote! {
                Ok(#name(#decode_call))
            }
        };

    let nil_impl =
        if let Some(codec) = field.attrs.codec().filter(|cc| cc.is_decode()) {
            if let Some(expr) = codec.to_nil_expr() {
                let nil = match expr {
                    PathOrDefault::Path(p) => quote!(#p()),
                    PathOrDefault::Default => quote!(Some(Default::default()))
                };
                if field.is_name {
                    let id = &field.ident;
                    quote! {
                        fn nil() -> core::option::Option<Self> {
                            #nil.map(|v| Self { #id: v })
                        }
                    }
                } else {
                    quote! {
                        fn nil() -> core::option::Option<Self> {
                            #nil.map(Self)
                        }
                    }
                }
            } else {
                // without a `nil()` do not override the default impl
                quote!()
            }
        } else if field.is_name { // no custom codec => forward to inner type
            let id = &field.ident;
            quote! {
                fn nil() -> core::option::Option<Self> {
                    #default_nil_fn().map(|v| Self { #id: v })
                }
            }
        } else { // no custom codec => forward to inner type
            quote! {
                fn nil() -> core::option::Option<Self> {
                    #default_nil_fn().map(Self)
                }
            }
        };

    Ok(quote! {
        impl #impl_generics minicbor::Decode<'bytes, Ctx> for #name #typ_generics #where_clause {
            fn decode(__d777: &mut minicbor::Decoder<'bytes>, __ctx777: &mut Ctx) -> core::result::Result<#name #typ_generics, minicbor::decode::Error> {
                #decode_body
            }

            #nil_impl
        }
    })
}

fn gen_decode_bound() -> syn::TypeParamBound {
    syn::parse_quote!(minicbor::Decode<'bytes, Ctx>)
}

fn gen_default_bound() -> syn::TypeParamBound {
    syn::parse_quote!(Default)
}

fn defs<'a, T>(fields: T) -> Vec<proc_macro2::TokenStream>
where
    T: IntoIterator<Item = &'a Field>
{
    fields.into_iter()
        .map(|f| {
            if f.attrs.default() {
                quote!(Some(Default::default()))
            } else {
                quote!(None)
            }
        })
        .collect()
}

fn nils<'a, T>(fields: T) -> Vec<proc_macro2::TokenStream>
where
    T: IntoIterator<Item = &'a Field>
{
    fields.into_iter().map(nil).collect()
}

fn nil(f: &Field) -> proc_macro2::TokenStream {
    if let Some(d) = f.attrs.codec() {
        match d.to_nil_expr() {
            Some(PathOrDefault::Path(p)) => quote!(#p()),
            Some(PathOrDefault::Default) => quote!(Some(Default::default())),
            None => if is_option(&f.typ, |_| true) {
                quote!(Some(None))
            } else {
                quote!(None)
            }
        }
    } else {
        let ty = &f.typ;
        quote!(<#ty as minicbor::Decode::<Ctx>>::nil())
    }
}

fn decode_tag(a: &Attributes) -> proc_macro2::TokenStream {
    if let Some(t) = a.tag() {
        quote! {
            let __p777 = __d777.position();
            let __t777 = __d777.tag()?;
            if #t != __t777.as_u64() {
                return Err(minicbor::__minicbor_cfg! {
                    'std {
                        minicbor::decode::Error::tag_mismatch(__t777)
                            .with_message(format!("expected tag {}", #t))
                            .at(__p777)
                    }
                    'alloc {
                        minicbor::decode::Error::tag_mismatch(__t777)
                            .with_message(alloc::format!("expected tag {}", #t))
                            .at(__p777)
                    }
                    'otherwise {
                        minicbor::decode::Error::tag_mismatch(__t777).at(__p777)
                    }
                });
            }
        }
    } else {
        quote!()
    }
}

fn field_inits(name: &str, fields: &Fields) -> proc_macro2::TokenStream {
    let mut fragments = Vec::new();
    for field in fields.fields() {
        let nil = nil(field);
        let idt = &field.ident;
        let idx = &field.index;
        let str = format!("{name}::{idt}");
        let def = if field.attrs.default() {
            quote!(Some(Default::default()))
        } else {
            quote!(None)
        };
        let fun = if field.index.is_num() {
            quote!(missing_value)
        } else {
            quote!(missing_value_str)
        };
        fragments.push((field.pos, quote! {
            if let Some(x) = #idt {
                x
            } else if let Some(def) = #def {
                def
            } else if let Some(z) = #nil {
                z
            } else {
                return Err(minicbor::decode::Error::#fun(#idx).with_message(#str).at(__p777))
            },
        }))
    }
    for skipped in fields.skipped() {
        fragments.push((skipped.pos, quote!(Default::default(),)))
    }
    fragments.sort_unstable_by_key(|(k, _)| *k);
    let mut expr = quote!();
    expr.extend(fragments.into_iter().map(|(_, f)| f));
    expr
}
