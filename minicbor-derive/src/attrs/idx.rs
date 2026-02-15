use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt, quote};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Index {
    Num(Idx),
    Str(String)
}

impl Index {
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }

    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num(_))
    }

    pub fn is_b(&self) -> bool {
        matches!(self, Self::Num(Idx::B(_)))
    }

    pub fn unwrap_numeric(&self) -> Idx {
        if let Self::Num(idx) = self {
            return *idx
        }
        panic!("Index is not numeric")
    }

    pub fn to_method(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Num(_) => quote!(i64),
            Self::Str(_) => quote!(str)
        }
    }
}

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Num(i) => i.to_tokens(tokens),
            Self::Str(s) => tokens.append(proc_macro2::Literal::string(s))
        }
    }
}

/// The index attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Idx {
    /// A regular, non-borrowing index.
    N(i64),
    /// An index which indicates that the value borrows from the decoding input.
    B(i64)
}

impl ToTokens for Idx {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(proc_macro2::Literal::i64_unsuffixed(self.val()))
    }
}

impl Idx {
    /// Get the numeric index value.
    pub fn val(self) -> i64 {
        match self {
            Idx::N(i) => i,
            Idx::B(i) => i
        }
    }

    /// Get value in bytewise lexicographic order.
    pub fn bytewise_lexicographic(self) -> impl Ord {
        (self.val() < 0, self.val().unsigned_abs())
    }
}

/// Check that there are no duplicate `Idx` values in `iter`.
pub fn check_uniq<'a, I>(s: Span, iter: I) -> syn::Result<()>
where
    I: IntoIterator<Item = &'a Index>
{
    let mut numeric = HashSet::new();
    let mut strings = HashSet::new();
    let mut num_ctr = 0;
    let mut str_ctr = 0;

    for idx in iter {
        match idx {
            Index::Num(i) => {
                numeric.insert(i.val());
                num_ctr += 1;
            }
            Index::Str(s) => {
                strings.insert(s);
                str_ctr += 1
            }
        }
    }

    if num_ctr != numeric.len() {
        return Err(syn::Error::new(s, "duplicate index numbers"))
    }
    if str_ctr != strings.len() {
        return Err(syn::Error::new(s, "duplicate index names"))
    }

    Ok(())
}

