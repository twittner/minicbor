use crate::attrs::{Attributes, Kind, Level};
use crate::attrs::idx::{self, Index};
use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct Variants {
    /// CBOR indices of variants
    pub indices: Vec<Index>,
    /// variant attributes
    pub attrs: Vec<Attributes>
}

impl Variants {
    pub fn try_from<'a, I>(span: Span, iter: I, parent: &Attributes) -> syn::Result<Self>
    where
        I: IntoIterator<Item = &'a syn::Variant>
    {
        let mut indices = Vec::new();
        let mut attrs   = Vec::new();

        let parent_encoding = parent.effective_encoding();

        let text_keys = parent.text_keys();

        for v in iter.into_iter() {
            let attr = Attributes::try_from_iter(Level::Variant, &v.attrs)?;
            if !text_keys && attr.key().is_some() {
                return Err(syn::Error::new(v.ident.span(), "`#[cbor(key = \"...\")]` requires `#[cbor(text_keys)]` on the enum"))
            }
            let idex = if let Some(i) = attr.index() {
                i.clone()
            } else if text_keys {
                let key = attr.key()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.ident.to_string());
                Index::Str(key)
            } else {
                return Err(syn::Error::new(v.ident.span(), "missing `#[n(...)]`, `#[b(...)]`, or `#[s(...)]` attribute"))
            };
            if idex.is_str() && attr.encoding().unwrap_or(parent_encoding).is_array() {
                let span = attr.span(Kind::Index).unwrap_or_else(|| v.ident.span());
                return Err(syn::Error::new(span, "array encoding does not support constructors with string indices"))
            }
            if parent.flat() {
                if attr.tag().is_some() {
                    let span = attr.span(Kind::Tag).unwrap_or_else(|| v.ident.span());
                    return Err(syn::Error::new(span, "flat enum does not support tags on constructors"))
                }
                if attr.encoding().unwrap_or(parent_encoding).is_map() {
                    let span = attr.span(Kind::Encoding).unwrap_or_else(|| v.ident.span());
                    return Err(syn::Error::new(span, "flat enum does not support map encoding"))
                }
            }
            indices.push(idex.clone());
            attrs.push(attr);
        }

        idx::check_uniq(span, &indices)?;

        Ok(Variants { indices, attrs })
    }
}
