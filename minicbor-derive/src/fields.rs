use crate::attrs::{Attributes, Idx, Kind, Level};
use crate::attrs::idx;
use proc_macro2::Span;
use syn::{Ident, Type};
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub struct Fields {
    fields: Vec<Field>,
    skipped: Vec<Field>
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
    pub index: Idx,
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

        let encoding = parents.iter().find_map(|p| p.encoding()).unwrap_or_default();

        for (pos, f) in iter.into_iter().enumerate() {
            let attrs = Attributes::try_from_iter(Level::Field, &f.attrs)?;
            let index = if attrs.skip() {
                debug_assert!(attrs.index().is_none());
                Idx::N(i64::MAX)
            } else if let Some(i) = attrs.index() {
                debug_assert!(!attrs.skip());
                i
            } else if parents.last().map(|p| p.transparent()).unwrap_or(false) {
                Idx::N(i64::MAX)
            } else {
                let s = f.ident.as_ref().map(|i| i.span()).unwrap_or_else(|| f.ty.span());
                return Err(syn::Error::new(s, "missing `#[n(...)]` or `#[b(...)]` attribute"))
            };
            if index.val().is_negative() && encoding.is_array() {
                let s = attrs.span(Kind::Index)
                    .or_else(|| f.ident.as_ref().map(|i| i.span()))
                    .unwrap_or_else(|| f.ty.span());
                return Err(syn::Error::new(s, "array encoding does not support fields with negative indices"))
            }
            let (ident, is_name) = match &f.ident {
                Some(n) => (n.clone(), true),
                None    => (quote::format_ident!("_{}", pos), false)
            };
            let typ  = f.ty.clone();
            let skip = attrs.skip();
            let fld  = Field { pos, index, ident, is_name, typ, attrs, orig: f.clone() };

            if skip {
                skipped.push(fld)
            } else {
                fields.push(fld)
            }
        }

        fields.sort_unstable_by_key(|f| f.index.bytewise_lexicographic());
        idx::check_uniq(span, fields.iter().map(|f| f.index))?;

        Ok(Fields { fields, skipped })
    }

    pub fn fields(&self) -> FieldIter<'_> {
        FieldIter(&self.fields, 0)
    }

    pub fn skipped(&self) -> FieldIter<'_> {
        FieldIter(&self.skipped, 0)
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

    pub fn indices(&self) -> impl Iterator<Item = Idx> + use<'a> {
        self.clone().map(|f| f.index)
    }

    pub fn positions(&self) -> impl Iterator<Item = usize> + use<'a> {
        self.clone().map(|f| f.pos)
    }
}
