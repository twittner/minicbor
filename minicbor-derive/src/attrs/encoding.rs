/// The encoding to use for structs and enum variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Encoding {
    #[default]
    Array,
    Map,
}

impl Encoding {
    pub fn is_array(self) -> bool {
        matches!(self, Self::Array)
    }

    pub fn is_map(self) -> bool {
        matches!(self, Self::Map)
    }
}
