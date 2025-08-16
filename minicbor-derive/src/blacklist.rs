use std::collections::HashSet;
use std::ops::Deref;

use crate::{collect_type_params, is_phantom_data, Mode};
use crate::{attrs::CustomCodec, fields::Fields};

#[derive(Default)]
pub(crate) struct Blacklist(HashSet<syn::Ident>);

impl Blacklist {
    /// Generate a blacklist of type parameters that should not have bounds attached.
    ///
    /// This includes:
    ///
    /// - Type parameters of fields with a custom encode, decode or cbor_len function.
    /// - Fields that are skipped over.
    /// - Fields with a `PhantomData` type.
    pub(crate) fn new(mode: Mode, fields: &Fields, g: &syn::Generics) -> Blacklist {
        // Start with custom encode/decode/cbor_len functions.
        let mut blacklist = collect_type_params(g, fields.fields().filter(|f| {
            match mode {
                Mode::Encode => f.attrs.codec().map(|c| c.is_encode()).unwrap_or(false),
                Mode::Decode => f.attrs.codec().map(|c| c.is_decode()).unwrap_or(false),
                Mode::Length => f.attrs.cbor_len().is_some()
                    || f.attrs.codec()
                        .map(|c| matches!(c, CustomCodec::Module(..)))
                        .unwrap_or(false)
            }
        }));
        if !blacklist.is_empty() {
            let others = collect_type_params(g, fields.fields().filter(|f| {
                match mode {
                    Mode::Encode => f.attrs.codec().map(|c| !c.is_encode()).unwrap_or(true),
                    Mode::Decode => f.attrs.codec().map(|c| !c.is_decode()).unwrap_or(true),
                    Mode::Length => f.attrs.cbor_len().is_none()
                        && f.attrs.codec()
                            .map(|c| !matches!(c, CustomCodec::Module(..)))
                            .unwrap_or(true)

                }
            }));
            blacklist.retain(|ident| !others.contains(ident));
        }

        // Extend the blacklist by type parameters only appearing in skipped fields.
        let skipped = collect_type_params(g, fields.skipped());
        if !skipped.is_empty() {
            let regular = collect_type_params(g, fields.fields());
            blacklist.extend(skipped.difference(&regular).cloned())
        }

        // And finally also by type parameters only appearing in `PhantomData`.
        let phantoms = collect_type_params(g, fields.fields().chain(fields.skipped()).filter(|f| {
            is_phantom_data(&f.typ)
        }));
        if !phantoms.is_empty() {
            let non_phantom = collect_type_params(g, fields.fields().chain(fields.skipped()).filter(|f| {
                !is_phantom_data(&f.typ)
            }));
            blacklist.extend(phantoms.difference(&non_phantom).cloned());
        }

        Self(blacklist)
    }

    /// Merge in another set of fields.
    ///
    /// Any types in positive position, i.e. not blacklisted in the given
    /// fields argument will be removed from the blacklist.
    ///
    /// Any types in negative position, i.e. blacklisted in the given fields
    /// argument will be add to the blacklist.
    pub(crate) fn merge(&mut self, m: Mode, f: &Fields, g: &syn::Generics) {
        let b = Blacklist::new(m, f, g);
        for t in collect_type_params(g, f.fields()).difference(&b) {
            self.0.remove(t);
        }
        for t in b.0 {
            self.0.insert(t);
        }
    }

    pub(crate) fn add<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = syn::Ident>
    {
        for id in it.into_iter() {
            self.0.insert(id);
        }
    }
}

impl From<Blacklist> for HashSet<syn::Ident> {
    fn from(b: Blacklist) -> Self {
        b.0
    }
}

impl Deref for Blacklist {
    type Target = HashSet<syn::Ident>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
