//! A set of I/O utilities for working with CBOR encoded values.

#![forbid(unsafe_code)]

mod error;
mod reader;
mod writer;

#[cfg(feature = "async-io")]
mod async_reader;

#[cfg(feature = "async-io")]
mod async_writer;

pub use error::Error;
pub use reader::Reader;
pub use writer::Writer;

#[cfg(feature = "async-io")]
pub use async_reader::AsyncReader;

#[cfg(feature = "async-io")]
pub use async_writer::AsyncWriter;

/// Ensure we can safely cast a `u32` to a `usize`.
const __U32_FITS_INTO_USIZE: () =
    if std::mem::size_of::<u32>() > std::mem::size_of::<usize>() {
        panic!("This crate requires at least a 32-bit architecture.")
    };

