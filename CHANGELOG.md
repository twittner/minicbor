# Contents

- [minicbor](#minicbor)
- [minicbor-derive](#minicbor-derive)
- [minicbor-io](#minicbor-io)

# minicbor

## `0.16.0`

- ⚠️ **Breaking** ⚠️: The `Encode` and `Decode` traits are now parameterised by a context type and
  the context value is passed as another argument to `Encode::encode` and `Decode::decode`.
  Implementations of these traits that do not make use of the context need to be generic in the
  type variable and accept the context parameter, e.g. instead of

  ```rust
  struct T;

  impl Encode for T {
      fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> { ... }
  }

  impl<'b> Decode<'b> for T {
      fn decode(d: &mut Decoder<'b>) -> Result<Self, Error> { ... }
  }
  ```

  one would now write:

  ```rust
  struct T;

  impl<C> Encode<C> for T {
      fn encode<W: Write>(&self, e: &mut Encoder<W>, ctx: &mut C) -> Result<(), Error<W::Error>> { ... }
  }

  impl<'b, C> Decode<'b, C> for T {
      fn decode(d: &mut Decoder<'b>, ctx: &mut C) -> Result<Self, Error> { ... }
  }
  ```
- Several new methods have been added to `Decoder` and `Encoder` to work with contexts:

    - `Decoder::decode_with`
    - `Decoder::array_iter_with`
    - `Decoder::map_iter_with`
    - `Encoder::encode_with`

  These correspond to the existing variants without the `_with` suffix which do not accept a context
  and fix the context type to `()`. Note that generic implementations of `Decode` and `Encoder` must
  therefore use the versions which accept a context parameter.

  Other additions include the crate-level functions:

    - `encode_with`
    - `decode_with`
    - `to_vec_with`

## `0.15.0`

- ⚠️ **Breaking** ⚠️: The encoding of IP addresses changed (see commit fac39d5a). This affects the
  following types:

    - std::net::IpAddr
    - std::net::Ipv4Addr
    - std::net::Ipv6Addr
    - std::net::SocketAddr
    - std::net::SocketAddrV4
    - std::net::SocketAddrV6

  A new module `minicbor::legacy` is introduced which contains newtype wrappers for these types
  which continue to use the array-based encoding. Users can opt out of the new compact format by
  enabling the cargo feature `"legacy"` and importing the types from the legacy module.
- A new type `minicbor::data::Int` has been introduced (see merge request !20) to allow encoding
  and decoding of the whole CBOR integer range [-2<sup>64</sup>, 2<sup>64</sup> - 1].
- ⚠️ **Breaking** ⚠️: As a consequence of adding the new `Int` type, a new constructor
  `minicbor::data::Type::Int` has been added to denote those (signed) integers that do not fit
  into an `i64`. Similarly the new constructor `minicbor::decode::Token::Int` captures those values.

## `0.14.2`

- Bugfix release: Imports `alloc::string::ToString` when necessary (see issue #21) for details.

## `0.14.1`

- Maintenance release: Add position information to UTF-8 decoding errors.

## `0.14.0`

- ⚠️ **Breaking** ⚠️: `encode::Error` and `decode::Error` are now structs instead of enums.
  The actual error representation is hidden and errors are constructed with functions instead
  of creating enum values directly, for example `Error::Message("foo")` is now
  `Error::message("foo")`. This was done to support adding more information to error values,
  like the decoding position. For details see merge request !19.

## `0.13.2`

- Added `Decode` impl for `Box<str>` (see merge request !18 by @tailhook).

## `0.13.1`

- Bugfix: `Decoder::datatype` would sometimes report incorrect types for negative integers
  (see issue #18 and commit 0bd97b72 for details).

## `0.13.0`

- ⚠️ **Breaking** ⚠️: Removed the `Clone` impl of `decode::Error`.
- Added a new variant `decode::Error::Custom` (requires feature `std`) which contains an
  arbitrary `Box<dyn std::error::Error + Send + Sync>`.

## `0.12.1`

- Change `Tokenizer::token` to move to the end of decoding input when an error occurs. This is
  done because some errors do not cause consumption of the input, hence repeated calls to
  `Tokenizer::token` may not terminate.

## `0.12.0`

- Extend the optionality of fields beyond `Option`. This applies to derived impls of `Encode`
  and `Decode` which make use of newly added methods `Encode::is_nil` and `Decode::nil`
  instead of `Option::is_none` and `None`. See issue #10 and merge request !15 for details.

## `0.11.5`

- Accept non-preferred integer encodings (see issue #14 for details).
- Added `Decoder::{null, undefined}` methods.
- Added `data::Cbor` as identity element of `Encode` and `Decode`.

## `0.11.4`

- Bugfix: Decoding strings or bytes with a length of `u64::MAX` would cause an overflow of the
  internal decoder position value. This case is now properly handled. See issue #12 for details.
- Bugfix: The `partial-derive-support` feature did not re-export `minicbor-derive`, nor did it
  make the functionality of `minicbor::bytes` available. See merge request !14 by @dne1 for details.

## `0.11.3`

- Bugfix release: Version `0.11.2` added `Encode`/`Decode` impls for various atomic types without
  considering their availability on the target platform (cf. issue #11). In here we attempt to
  only offer impls for available atomic types (cf. merge request !13 for details).

## `0.11.2`

- Improves `minicbor::display` to be more robust when applied to malformed CBOR values
  (see commit c1294dd for details).
- Adds several `Encode`/`Decode` impls:
    - `core::num::Wrapping`
    - `core::sync::atomic::{AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize}`
    - `core::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize}`
    - `core::cell::{Cell, RefCell}`
    - `core::ops::{Bound, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive}`
    - `std::path::{Path, PathBuf}`
    - `std::time::SystemTime`

## `0.11.1`

- Depends on `minicbor-derive-0.7.1`.

## `0.11.0`

- Depends on `minicbor-derive-0.7.0`.

## `0.10.1`

- Small bugfix release (see commit 68963dc for details).

## `0.10.0`

- ⚠️ **Breaking** ⚠️: By default `Decoder::skip` is not available anymore. It can be enabled with
  feature flag `"alloc"` (implied by `"std"`) or `"partial-skip-support"`. The latter variant
  corresponds to the implementation of minicbor <= 0.9.1 but does not support skipping over
  indefinite-length arrays or maps inside of regular arrays or maps. The variant enabled by
  `"alloc"` supports skipping over arbitrary CBOR items. For more information see
  [feature flags][3] and issue #9.

## `0.9.1`

- Adds a few more trait impls to `ByteArray` and `ByteVec`. See commit b17fe67.

## `0.9.0`

- ⚠️ **Breaking** ⚠️: The encoding of `()` and `PhantomData` has changed. See commit b6b1f907.
- ⚠️ **Breaking** ⚠️: The `decode::Error::TypeMismatch` constructor changed to use a `data::Type`
  instead of a `u8` as its first parameter. See merge request !6 for details.
- Added feature flag `alloc` (implied by `std`), which enables most collections types in a `no_std`
  environment. See merge request !9 for details.
- Added `ByteArray` to support compact encoding of `u8`-arrays, similarly to the already existing
  `ByteSlice` and `ByteVec` types added in `minicbor-0.6.0`. See merge request !10 for details.
- Added `Write` impl for `alloc::vec::Vec` (see merge request !11 by @Hawk777).
- Depends on `minicbor-derive-0.6.4`.

## `0.8.1`

- Depends on `minicbor-derive-0.6.3`.

## `0.8.0`

- ⚠️ **Breaking** ⚠️: Change `data::Type` to distinguish between indefinite arrays, maps, bytes and
  strings, and regular ones by introducing constructors such as `Type::ArrayIndef`.
- Add new types `decode::Token` and `decode::Tokenizer` to allow decoding CBOR bytes
  generically as a mere sequence of tokens.
- Add a function `display` which displays CBOR bytes in [diagnostic notation][2].

## `0.7.2`

- Bugfix: `Type::read` used `0xc9` instead of `0xc0` when reading `Type::Tag`.
- Add README.md

## `0.7.1`

- Require `minicbor-derive-0.6.1`.

## `0.7.0`

- Require `minicbor-derive-0.6.0`.

## `0.6.0`

- Removes the `&[u8]` impl for `Decode` (see issue #4) and add a new module `minicbor::bytes`
  to support specialised encoding of CBOR bytes. This module provides the types `ByteSlice` and
  `ByteVec` which are substitutes for `&[u8]` and `Vec<u8>` respectively. See also the module
  documentation of `minicbor::bytes`.

## `0.5.1`

- Require `minicbor-derive-0.4.1`.

## `0.5.0`

- Require `minicbor-derive-0.4.0`.

## `0.4.1`

- Adds `Encoder::f16` to support encoding of `f32` values as half floats. Complements the existing
  `Decoder::f16` method and depends on the feature `half`.

## `0.4.0`

- Added `Encode` and `Decode` impls for tuples (see merge request !1 by @koushiro).

# minicbor-derive

## `0.10.0`

- Depends on `minicbor-0.16.0`.
- A new attribute `context_bound` has been added to allow constraining the generic context type of
  the derived `Encode` or `Decode` trait impl with a set of trait bounds.

## `0.9.0`

- Depends on `minicbor-0.14.0`.

## `0.8.0`

- Uses `Encode::is_nil` and `Decode::nil` instead of `Option::is_none` and `None` to generalise
  field optionality.
- Adds new attributes `is_nil`, `nil` and `has_nil` to enable integration of optional types which
  do not implement `Encode` or `Decode`.

## `0.7.2`

- Small bugfix release.

## `0.7.1`

- Small error reporting improvement (cf. 1b1cb41).

## `0.7.0`

- Major internal refactoring to make attribute parsing more flexible and robust.
- Adds as new attributes `encode_bound`, `decode_bound` and `bound` to allow overriding the
  generated type parameter bounds.

## `0.6.4`

- Improve hygiene (see merge request !7).

## `0.6.3`

- Improve macro hygiene.

## `0.6.2`

- Add README.md

## `0.6.1`

- Maintenance release.

## `0.6.0`

- Adds `#[cbor(index_only)]` attribute to support a more compact encoding for enums without fields
  (read [the documentation][1] for details).

## `0.5.0`

- When deriving, the attribute `#[cbor(with = "minicbor::bytes")]` can be used for `&[u8]` and
  `Option<&[u8]>` if direct use of `ByteSlice` is not desired.
  See also the section *"What about `&[u8]`?"* in `minicbor-derive`.

## `0.4.1`

- Adds `#[cbor(transparent)]` to allow newtypes to use the same CBOR encoding as their inner type.

## `0.4.0`

- Adds `#[cbor(encode_with)]`, `#[cbor(decode_with)` and `#[cbor(with)]` attributes to allow custom
  encode/decode functions which replace their trait counterparts or provide a way to handle types
  which do not implement these traits.

## `0.3.0`

- Added `#[cbor(map)]` and `#[cbor(array)]` attributes (see commit 40e8b240 for details).

# minicbor-io

## `0.11.0`

- Depends on `minicbor-0.16.0`.
- The following new methods have been added:

    - `Reader::read_with`
    - `AsyncReader::read_with`
    - `Writer::write_with`
    - `AsyncWriter::write_with`

  These accept an additional context parameter and the existing variants fix the context to the
  unit type.

## `0.10.0`

- Depends on `minicbor-0.15.0`.

## `0.9.0`

- Depends on `minicbor-0.14.0`.

## `0.8.0`

- Depends on `minicbor-0.13.0`.

## `0.7.0`

- Depends on `minicbor-0.12.0`.

## `0.6.0`

- Depends on `minicbor-0.11.0`.

## `0.5.0`

- Depends on `minicbor-0.10.0`.

## `0.4.0`

- Depends on `minicbor-0.9.0`.

## `0.3.1`

- Depends on `minicbor-0.8.1`.

## `0.3.0`

- Depends on `minicbor-0.8.0`.

## `0.2.3`

- Add README.md

## `0.2.2`

- Use same version for `minicbor` dependency in `dependencies` and `dev-dependencies`.

## `0.2.1`

- `Reader` and `AsyncReader` always return `UnexpectedEof` when reading 0 bytes while decoding a
  frame, unless at the very beginning of a frame, when not even the length prefix has been read,
  where `Ok(None)` would be returned instead. Previous versions returned `Ok(None)` while reading
  a partial length prefix.

## `0.1.2`

- Update dev-dependencies.

## `0.1.1`

- Fix link to documentation in `Cargo.toml`.

## `0.1.0`

- Initial release which provides some I/O utilities.

[1]: https://twittner.gitlab.io/minicbor/minicbor_derive/index.html#index_only
[2]: https://www.rfc-editor.org/rfc/rfc8949.html#section-8
[3]: https://twittner.gitlab.io/minicbor/minicbor/index.html#feature-flags

