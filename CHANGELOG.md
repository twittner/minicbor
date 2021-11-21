# Contents

- [minicbor](#minicbor)
- [minicbor-derive](#minicbor-derive)
- [minicbor-io](#minicbor-io)

# minicbor

## `0.11.5`

- Accept non-preferred integer encodings (see issue #14 for details).
- Added `Decoder::{null, undefined}` methods.
- Added `data::Cbor` as identity element of `Encode` and `Decode`.

## `0.11.4`

- Bugfix: Decoding strings or bytes with a length of `u64::MAX` would cause an overflow of the
  internal decoder position value. This case is now properly handled. See issue #12 for details.
- Bugfix: The `partial-derive-support` feature did not re-export `minicbor-derive`, nor did it
  make the functionality of `minicbor::bytes` available. See merge request !14 for details.

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

# minicbor-derive

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

# minicbor-io

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

