//! Support for (de-)serialising CBOR with [serde].
//!
//! In contrast to [minicbor-derive], this serde-based implementation makes no
//! attempt to be particularly clever with regards to forward and backward
//! compatibility, nor does it use integers instead of strings for struct field
//! names or enum constructors. If those features are important, consider using
//! [minicbor-derive] instead.
//!
//! # Example
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
//! struct S {
//!     field: bool
//! }
//!
//! let s1 = S::default();
//!
//! let cbor = minicbor_serde::to_vec(&s1)?;
//! let s2: S = minicbor_serde::from_slice(&cbor)?;
//!
//! assert_eq!(s1, s2);
//!
//! let mut buf = Vec::new();
//! let mut ser = minicbor_serde::Serializer::new(&mut buf);
//! s1.serialize(&mut ser)?;
//!
//! let mut de = minicbor_serde::Deserializer::new(&buf);
//! let s3 = S::deserialize(&mut de)?;
//!
//! assert_eq!(s1, s3);
//!
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```
//!
//! [serde]: https://serde.rs/
//! [minicbor-derive]: https://twittner.gitlab.io/minicbor/minicbor_derive/

#[cfg(feature = "alloc")]
extern crate alloc;

mod de;
mod ser;
pub mod error;

pub use de::{Deserializer, from_slice};
pub use ser::Serializer;

#[cfg(feature = "alloc")]
pub use ser::to_vec;
