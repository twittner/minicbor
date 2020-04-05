/// Like [`std::borrow::Cow`] but specialised for `str/String`.
///
/// This type exists to support direct borrowing of `&str` slices
/// from the decoding input.
#[derive(Debug, Clone)]
pub enum String<'a> {
    /// A borrowed slice.
    Borrowed(&'a str),
    /// An owned string.
    Owned(std::string::String)
}

impl String<'_> {
    /// Is this string owned?
    pub fn is_owned(&self) -> bool {
        if let String::Owned(_) = self {
            true
        } else {
            false
        }
    }

    /// Is this string borrowed?
    pub fn is_borrowed(&self) -> bool {
        !self.is_owned()
    }

    /// Get a mutable reference to an owned string.
    ///
    /// If the string is not already owned, the `&str` is copied
    /// into a string first.
    pub fn to_mut(&mut self) -> &mut std::string::String {
        if let String::Owned(v) = self {
            return v
        }
        *self = String::Owned(self.as_ref().into());
        if let String::Owned(v) = self {
            return v
        }
        unreachable!()
    }

    /// Extract the owned string of this value.
    ///
    /// If the string is not already owned, the &str is copied
    /// into a string first.
    pub fn into_owned(self) -> std::string::String {
        if let String::Owned(v) = self {
            return v
        }
        self.as_ref().into()
    }
}

impl PartialEq<String<'_>> for String<'_> {
    fn eq(&self, other: &String<'_>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for String<'_> {}

impl AsRef<str> for String<'_> {
    fn as_ref(&self) -> &str {
        match self {
            String::Borrowed(s) => s,
            String::Owned(s) => s.as_ref()
        }
    }
}

impl<'a> From<&'a str> for String<'a> {
    fn from(b: &'a str) -> Self {
        String::Borrowed(b)
    }
}

impl<'a> From<&'a std::string::String> for String<'a> {
    fn from(b: &'a std::string::String) -> Self {
        String::Borrowed(b.as_ref())
    }
}

impl From<std::string::String> for String<'_> {
    fn from(b: std::string::String) -> Self {
        String::Owned(b)
    }
}

