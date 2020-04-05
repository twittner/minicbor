/// Like [`std::borrow::Cow`] but specialised for `[u8]/Vec<u8>`.
///
/// This type exists to support direct borrowing of `&[u8]` slices
/// from the decoding input.
#[derive(Debug, Clone)]
pub enum Bytes<'a> {
    /// A borrowed slice.
    Borrowed(&'a [u8]),
    /// An owned vector.
    Owned(Vec<u8>)
}

impl Bytes<'_> {
    /// Is this an owned vector?
    pub fn is_owned(&self) -> bool {
        if let Bytes::Owned(_) = self {
            true
        } else {
            false
        }
    }

    /// Is this a borrowed slice?
    pub fn is_borrowed(&self) -> bool {
        !self.is_owned()
    }

    /// Get a mutable reference to an owned vector.
    ///
    /// If the data is not already owned, the `&[u8]` is copied
    /// into a `Vec<u8>` first.
    pub fn to_mut(&mut self) -> &mut Vec<u8> {
        if let Bytes::Owned(v) = self {
            return v
        }
        *self = Bytes::Owned(self.as_ref().into());
        if let Bytes::Owned(v) = self {
            return v
        }
        unreachable!()
    }

    /// Extract the owned vector of this value.
    ///
    /// If the data is not already owned, the `&[u8]` is copied
    /// into a `Vec<u8>` first.
    pub fn into_owned(self) -> Vec<u8> {
        if let Bytes::Owned(v) = self {
            return v
        }
        self.as_ref().into()
    }
}

impl PartialEq<Bytes<'_>> for Bytes<'_> {
    fn eq(&self, other: &Bytes<'_>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Bytes<'_> {}

impl AsRef<[u8]> for Bytes<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Bytes::Borrowed(s) => s,
            Bytes::Owned(v) => v.as_ref()
        }
    }
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    fn from(b: &'a [u8]) -> Self {
        Bytes::Borrowed(b)
    }
}

impl<'a> From<&'a Vec<u8>> for Bytes<'a> {
    fn from(b: &'a Vec<u8>) -> Self {
        Bytes::Borrowed(b.as_ref())
    }
}

impl From<Vec<u8>> for Bytes<'_> {
    fn from(b: Vec<u8>) -> Self {
        Bytes::Owned(b)
    }
}

