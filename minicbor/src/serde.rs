//! Support for (de-)serialising CBOR with [serde].
//!
//! In contrast to [`minicbor_derive`], this serde-based implementation makes no
//! attempt to be particularly clever with regards to forward and backward
//! compatibility, nor does it use integers instead of strings for struct field
//! names or enum constructors. If those features are important, consider using
//! [`minicbor_derive`] instead.
//!
//! [serde]: https://serde.rs/

mod de;
mod ser;

pub use de::{Deserializer, from_slice};
pub use ser::Serializer;

#[cfg(feature = "std")]
pub use ser::to_vec;
