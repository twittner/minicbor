/// The encoding to use for structs and enum variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Array(Len),
    Map(Len)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Len {
    /// Definite length collection.
    Def,
    /// Indefinite length collection.
    Indef
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Array(Len::Def)
    }
}

impl Encoding {
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn len(self) -> Len {
        match self {
            Self::Array(len) => len,
            Self::Map(len) => len
        }
    }
}

impl Len {
    pub fn is_def(self) -> bool {
        matches!(self, Self::Def)
    }

    pub fn is_indef(self) -> bool {
        matches!(self, Self::Indef)
    }
}
