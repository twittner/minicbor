/// The encoding to use for structs and enum variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Encoding {
    #[default]
    Array,
    IndefiniteArray,
    Map
}

impl Encoding {
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array | Self::IndefiniteArray)
    }

    pub fn is_map(self) -> bool {
        matches!(self, Self::Map)
    }
}
