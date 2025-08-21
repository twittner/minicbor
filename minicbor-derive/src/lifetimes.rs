use std::collections::BTreeSet;

use crate::attrs::Idx;
use crate::{is_byte_slice, is_str};

/// Generate the decode lifetime.
pub fn gen_lifetime() -> syn::Result<syn::LifetimeParam> {
    syn::parse_str("'bytes")
}

/// Return a modified clone of `syn::Generics` with the given lifetime
/// parameter put before the other type and lifetime parameters.
pub fn add_lifetime(g: &syn::Generics, l: syn::LifetimeParam) -> syn::Generics {
    let mut g2 = g.clone();
    g2.params = Some(l.into()).into_iter().chain(g2.params).collect();
    g2
}

/// Get the set of lifetimes which need to be constrained to the decoding input lifetime.
pub fn lifetimes_to_constrain<'a, I>(types: I) -> BTreeSet<syn::Lifetime>
where
    I: Iterator<Item = (&'a Idx, Option<&'a BTreeSet<syn::Lifetime>>, &'a syn::Type)>,
{
    // Get the lifetime of a reference if its type matches the predicate.
    fn tyref_lifetime(
        ty: &syn::Type,
        pred: impl FnOnce(&syn::Type) -> bool,
    ) -> Option<syn::Lifetime> {
        if let syn::Type::Reference(p) = ty {
            if pred(&p.elem) {
                return p.lifetime.clone();
            }
        }
        None
    }

    // Get all lifetimes of a type.
    fn get_lifetimes(
        ty: &syn::Type,
        set: &mut BTreeSet<syn::Lifetime>,
        filter: &BTreeSet<syn::Lifetime>,
    ) {
        match ty {
            syn::Type::Array(t) => get_lifetimes(&t.elem, set, filter),
            syn::Type::Slice(t) => get_lifetimes(&t.elem, set, filter),
            syn::Type::Paren(t) => get_lifetimes(&t.elem, set, filter),
            syn::Type::Group(t) => get_lifetimes(&t.elem, set, filter),
            syn::Type::Ptr(t) => get_lifetimes(&t.elem, set, filter),
            syn::Type::Reference(t) => {
                if let Some(l) = &t.lifetime {
                    if filter.is_empty() || filter.contains(l) {
                        set.insert(l.clone());
                    }
                }
                get_lifetimes(&t.elem, set, filter)
            }
            syn::Type::Tuple(t) => {
                for t in &t.elems {
                    get_lifetimes(t, set, filter)
                }
            }
            syn::Type::Path(t) => {
                for s in &t.path.segments {
                    if let syn::PathArguments::AngleBracketed(b) = &s.arguments {
                        for a in &b.args {
                            match a {
                                syn::GenericArgument::Type(t) => get_lifetimes(t, set, filter),
                                syn::GenericArgument::AssocType(b) => {
                                    get_lifetimes(&b.ty, set, filter)
                                }
                                syn::GenericArgument::Lifetime(l) => {
                                    if filter.is_empty() || filter.contains(l) {
                                        set.insert(l.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Get the lifetime of the given type if it is an `Option` whose inner type matches the predicate.
    fn option_lifetime(
        ty: &syn::Type,
        pred: impl FnOnce(&syn::Type) -> bool,
    ) -> Option<syn::Lifetime> {
        if let syn::Type::Path(t) = ty {
            if let Some(s) = t.path.segments.last() {
                if s.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(b) = &s.arguments {
                        if b.args.len() == 1 {
                            if let syn::GenericArgument::Type(syn::Type::Reference(ty)) = &b.args[0]
                            {
                                if pred(&ty.elem) {
                                    return ty.lifetime.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    let mut set = BTreeSet::new();
    for (i, l, t) in types {
        if let Some(l) = tyref_lifetime(t, is_str) {
            set.insert(l);
            continue;
        }
        if let Some(l) = tyref_lifetime(t, is_byte_slice) {
            set.insert(l);
            continue;
        }
        if let Some(l) = option_lifetime(t, is_str) {
            set.insert(l);
            continue;
        }
        if let Some(l) = option_lifetime(t, is_byte_slice) {
            set.insert(l);
            continue;
        }
        if let Some(l) = l {
            get_lifetimes(t, &mut set, l)
        }
        if i.is_b() {
            get_lifetimes(t, &mut set, &BTreeSet::new())
        }
    }
    set
}
