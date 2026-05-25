use std::cmp::Ordering;

use crate::attrs::{Attributes, Idx, Kind, Level};
use crate::attrs::idx::{self, Index};
use proc_macro2::Span;
use syn::{Ident, Type};
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub struct Fields {
    fields: Vec<Field>,
    skipped: Vec<Field>,
    has_str_index: bool
}

#[derive(Debug, Clone)]
pub struct Field {
    /// field position
    pub pos: usize,
    /// field identifier
    pub ident: Ident,
    /// does the field hava a name or is the identifier generated
    pub is_name: bool,
    /// CBOR index
    pub index: Index,
    /// field type
    pub typ: Type,
    /// field attributes
    pub attrs: Attributes,
    /// the original syn field
    pub orig: syn::Field
}

impl Fields {
    pub fn try_from<'a, I>(span: Span, iter: I, parents: &[&Attributes]) -> syn::Result<Self>
    where
        I: IntoIterator<Item = &'a syn::Field>
    {
        let mut fields  = Vec::new();
        let mut skipped = Vec::new();
        let mut has_str_index = false;

        let encoding = parents.iter().find_map(|p| p.encoding()).unwrap_or_default();

        for (pos, f) in iter.into_iter().enumerate() {
            let attrs = Attributes::try_from_iter(Level::Field, &f.attrs)?;
            let index = if attrs.skip() {
                debug_assert!(attrs.index().is_none());
                Index::Num(Idx::N(i64::MAX))
            } else if let Some(i) = attrs.index() {
                debug_assert!(!attrs.skip());
                i.clone()
            } else if parents.last().map(|p| p.transparent()).unwrap_or(false) {
                Index::Num(Idx::N(i64::MAX))
            } else {
                let s = f.ident.as_ref().map(|i| i.span()).unwrap_or_else(|| f.ty.span());
                return Err(syn::Error::new(s, "missing `#[n(...)]`, `#[b(...)]`, or `#[s(...)]` attribute"))
            };
            if let Index::Str(_) = index && encoding.is_array() {
                let s = attrs.span(Kind::Index)
                    .or_else(|| f.ident.as_ref().map(|i| i.span()))
                    .unwrap_or_else(|| f.ty.span());
                return Err(syn::Error::new(s, "array encoding does not support fields with string indices"))
            }
            if let Index::Num(idx) = index && idx.val().is_negative() && encoding.is_array() {
                let s = attrs.span(Kind::Index)
                    .or_else(|| f.ident.as_ref().map(|i| i.span()))
                    .unwrap_or_else(|| f.ty.span());
                return Err(syn::Error::new(s, "array encoding does not support fields with negative indices"))
            }
            let (ident, is_name) = match &f.ident {
                Some(n) => (n.clone(), true),
                None    => (quote::format_ident!("_{}", pos), false)
            };

            has_str_index |= index.is_str();

            let typ  = f.ty.clone();
            let skip = attrs.skip();
            let fld  = Field { pos, index, ident, is_name, typ, attrs, orig: f.clone() };

            if skip {
                skipped.push(fld)
            } else {
                fields.push(fld)
            }
        }

        // Sort field indices by putting strings after numbers:
        fields.sort_unstable_by(|a, b| match (&a.index, &b.index) {
            (Index::Num(i), Index::Num(j)) => i.bytewise_lexicographic().cmp(&j.bytewise_lexicographic()),
            (Index::Str(a), Index::Str(b)) => a.cmp(b),
            (Index::Str(_), Index::Num(_)) => Ordering::Greater,
            (Index::Num(_), Index::Str(_)) => Ordering::Less
        });

        idx::check_uniq(span, fields.iter().map(|f| &f.index))?;

        Ok(Fields { fields, skipped, has_str_index })
    }

    pub fn fields(&self) -> FieldIter<'_> {
        FieldIter(&self.fields, 0)
    }

    pub fn skipped(&self) -> FieldIter<'_> {
        FieldIter(&self.skipped, 0)
    }

    pub fn has_str_index(&self) -> bool {
        self.has_str_index
    }

    /// Order all identifiers by position and replace skipped ones with `_`.
    ///
    /// To be used when matching identifiers by position, e.g. in tuples.
    pub fn match_idents(&self) -> Vec<syn::Ident> {
        let idents  = self.fields().positions().zip(self.fields().idents().cloned());
        let skipped = self.skipped().positions().zip(self.skipped().idents().map(|_| quote::format_ident!("_")));
        let mut all = idents.chain(skipped).collect::<Vec<_>>();
        all.sort_unstable_by_key(|(p, _)| *p);
        all.into_iter().map(|(_, i)| i).collect()
    }
}

#[derive(Debug, Clone)]
pub struct FieldIter<'a>(&'a [Field], usize);

impl<'a> Iterator for FieldIter<'a> {
    type Item = &'a Field;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.0.get(self.1) {
            self.1 += 1;
            return Some(x)
        }
        None
    }
}

impl ExactSizeIterator for FieldIter<'_> {
    fn len(&self) -> usize {
        self.0.len() - self.1
    }
}

impl<'a> FieldIter<'a> {
    pub fn attributes(&self) -> impl Iterator<Item = &'a Attributes> + Clone + use<'a> {
        self.clone().map(|f| &f.attrs)
    }

    pub fn idents(&self) -> impl Iterator<Item = &'a Ident> + Clone + use<'a> {
        self.clone().map(|f| &f.ident)
    }

    pub fn types(&self) -> impl Iterator<Item = &'a Type> + use<'a> {
        self.clone().map(|f| &f.typ)
    }

    pub fn indices(&self) -> impl Iterator<Item = &'a Index> + use<'a> {
        self.clone().map(|f| &f.index)
    }

    pub fn positions(&self) -> impl Iterator<Item = usize> + use<'a> {
        self.clone().map(|f| f.pos)
    }
}
