# Contents

- [minicbor](#minicbor)
- [minicbor-derive](#minicbor-derive)
- [minicbor-io](#minicbor-io)

# minicbor

## `0.9.0`

- **Breaking**: The `decode::Error::TypeMismatch` constructor changed to use a `data::Type`
  instead of a `u8` as its first parameter.
- Added feature flag `alloc` (implied by `std`), which enables most collections types in a `no_std`
  environment. See merge request #9 for details.
- Added `ByteArray` to support compact encoding of `u8`-arrays, similarly to the already existing
  `ByteSlice` and `ByteVec` types added in `minicbor-0.6.0`.
- Depend on `minicbor-derive-0.6.4`.

## `0.8.1`

- Depend on `minicbor-derive-0.6.3`.

## `0.8.0`

- **Breaking**: Change `data::Type` to distinguish between indefinite arrays, maps, bytes and
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

## `0.6.4`

- Improve hygiene (see merge request #7).

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

## `0.4.0`

- Depend on `minicbor-0.9.0`.

## `0.3.1`

- Depend on `minicbor-0.8.1`.

## `0.3.0`

- Depend on `minicbor-0.8.0`.

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


