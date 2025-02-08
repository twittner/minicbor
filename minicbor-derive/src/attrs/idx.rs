use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt};
use std::collections::HashSet;

/// The index attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Idx {
    /// A regular, non-borrowing index.
    N(i32),
    /// An index which indicates that the value borrows from the decoding input.
    B(i32)
}

impl ToTokens for Idx {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(proc_macro2::Literal::i32_unsuffixed(self.val()))
    }
}

impl Idx {
    /// Test if `Idx` is the `B` variant.
    pub fn is_b(self) -> bool {
        matches!(self, Idx::B(_))
    }

    /// Get the numeric index value.
    pub fn val(self) -> i32 {
        match self {
            Idx::N(i) => i,
            Idx::B(i) => i
        }
    }

    /// Get a property of the index that is sorted as needed to produce the encoded items:
    /// They are sorted in their deterministic encoding.
    ///
    /// This produces the correct sequence both for CBOR arrays (where all items are emitted in
    /// ascending order) and for CBOR maps (where keys need to be sorted in bytewise lexicographic
    /// ordering).
    ///
    /// This is used for pre-sorting items for serialization.
    pub fn val_for_sorting(self) -> impl Ord {
        (self.val() < 0, self.val().unsigned_abs())
    }
}

/// Check that there are no duplicate `Idx` values in `iter`.
pub fn check_uniq<I>(s: Span, iter: I) -> syn::Result<()>
where
    I: IntoIterator<Item = Idx>
{
    let mut set = HashSet::new();
    let mut ctr = 0;
    for u in iter {
        set.insert(u.val());
        ctr += 1;
    }
    if ctr != set.len() {
        return Err(syn::Error::new(s, "duplicate index numbers"))
    }
    Ok(())
}

