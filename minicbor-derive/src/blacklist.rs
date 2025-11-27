use std::collections::HashSet;
use std::ops::Deref;

use crate::{collect_type_params, is_phantom_data, Mode};
use crate::{attrs::CustomCodec, fields::Fields};

pub(crate) struct Blacklist(HashSet<syn::Ident>);

impl Blacklist {
    pub(crate) fn empty() -> Self {
        Self(HashSet::new())
    }

    /// Generate a blacklist of type parameters that should not have bounds attached.
    ///
    /// This includes:
    ///
    /// - Type parameters of fields with a custom encode, decode or cbor_len function.
    /// - Fields that are skipped over.
    /// - Fields with a `PhantomData` type.
    pub(crate) fn full(mode: Mode, fields: &Fields, g: &syn::Generics) -> Self {
        Self::empty()
            .with_mode(mode, fields, g)
            .with_skipped(fields, g)
            .with_phantoms(fields, g)
    }

    pub(crate) fn with_mode(mut self, mode: Mode, fields: &Fields, g: &syn::Generics) -> Self {
        let mut blacklist = collect_type_params(g, fields.fields().filter(|f| {
            match mode {
                Mode::Encode => f.attrs.codec().map(|c| !c.require_encode_bound()).unwrap_or(false),
                Mode::Decode => f.attrs.codec().map(|c| !c.require_decode_bound()).unwrap_or(false),
                Mode::Length => f.attrs.cbor_len().is_some()
                    || f.attrs.codec()
                        .map(|c| matches!(c, CustomCodec::Module(..)))
                        .unwrap_or(false)
            }
        }));
        if !blacklist.is_empty() {
            let others = collect_type_params(g, fields.fields().filter(|f| {
                match mode {
                    Mode::Encode => f.attrs.codec().map(|c| c.require_encode_bound()).unwrap_or(true),
                    Mode::Decode => f.attrs.codec().map(|c| c.require_decode_bound()).unwrap_or(true),
                    Mode::Length => f.attrs.cbor_len().is_none()
                        && f.attrs.codec()
                            .map(|c| !matches!(c, CustomCodec::Module(..)))
                            .unwrap_or(true)

                }
            }));
            blacklist.retain(|ident| !others.contains(ident));
        }
        self.0.extend(blacklist);
        self
    }

    /// Extend the blacklist by type parameters only appearing in skipped fields.
    pub(crate) fn with_skipped(mut self, fields: &Fields, g: &syn::Generics) -> Self {
        let skipped = collect_type_params(g, fields.skipped());
        if !skipped.is_empty() {
            let regular = collect_type_params(g, fields.fields());
            self.0.extend(skipped.difference(&regular).cloned())
        }
        self
    }

    pub(crate) fn with_phantoms(mut self, fields: &Fields, g: &syn::Generics) -> Self {
        let phantoms = collect_type_params(g, fields.fields().chain(fields.skipped()).filter(|f| {
            is_phantom_data(&f.typ)
        }));
        if !phantoms.is_empty() {
            let non_phantom = collect_type_params(g, fields.fields().chain(fields.skipped()).filter(|f| {
                !is_phantom_data(&f.typ)
            }));
            self.0.extend(phantoms.difference(&non_phantom).cloned());
        }
        self
    }


    /// Merge in another set of fields.
    ///
    /// Any types in positive position, i.e. not blacklisted in the given
    /// fields argument will be removed from the blacklist.
    ///
    /// Any types in negative position, i.e. blacklisted in the given fields
    /// argument will be add to the blacklist.
    pub(crate) fn merge(&mut self, f: &Fields, g: &syn::Generics, b: Blacklist) -> &mut Self {
        for t in collect_type_params(g, f.fields().chain(f.skipped())).difference(&b) {
            self.0.remove(t);
        }
        for t in b.0 {
            self.0.insert(t);
        }
        self
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
