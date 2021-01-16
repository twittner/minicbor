# 0.2.2 (minicbor-io)

Use same version for `minicbor` dependency in `dependencies` and
`dev-dependencies`.

# 0.2.1 (minicbor-io)

`Reader` and `AsyncReader` always return `UnexpectedEof` when reading 0 bytes
while decoding a frame, unless at the very beginning of a frame, when not
even the length prefix has been read, where `Ok(None)` would be returned
instead. Previous versions returned `Ok(None)` while reading a partial length
prefix.

# 0.7.1 (minicbor)
# 0.6.1 (minicbor-derive)

Maintenance release.

# 0.1.2 (minicbor-io)

Update dev-dependencies.

# 0.1.1 (minicbor-io)

Fix link to documentation in `Cargo.toml`.

# 0.1.0 (minicbor-io)

Initial release of [`minicbor-io`][2] which provides some I/O utilities.

[2]: https://twittner.gitlab.io/minicbor/minicbor_io/index.html

# 0.7.0 (minicbor)
# 0.6.0 (minicbor-derive)

Adds `#[cbor(index_only)]` attribute to support a more compact encoding for
enums without fields (see [[1]] for details).

[1]: https://twittner.gitlab.io/minicbor/minicbor_derive/index.html#index_only

# 0.6.0 (minicbor)
# 0.5.0 (minicbor-derive)

Removes the `&[u8]` impl for `Decode` (see issue #4) and add a new module
`minicbor::bytes` to support specialised encoding of CBOR bytes. This
module provides the types `ByteSlice` and `ByteVec` which are substitutes
for `&[u8]` and `Vec<u8>` respectively. When deriving, the attribute
`#[cbor(with = "minicbor::bytes")]` can be used for `&[u8]` and
`Option<&[u8]>` if direct use of `ByteSlice` is not desired. See also
the module documentation of `minicbor::bytes` and the section
*"What about `&[u8]`?"* in `minicbor-derive`.

# 0.5.1 (minicbor)
# 0.4.1 (minicbor-derive)

Adds `#[cbor(transparent)]` to allow newtypes to use the same CBOR encoding as
their inner type.

# 0.5.0 (minicbor)
# 0.4.0 (minicbor-derive)

Adds `#[cbor(encode_with)]`, `#[cbor(decode_with)` and `#[cbor(with)]` attributes
to allow custom encode/decode functions which replace their trait counterparts or
provide a way to handle types which do not implement these traits.

# 0.4.1 (minicbor)

Adds `Encoder::f16` to support encoding of `f32` values as half floats.
Complements the existing `Decoder::f16` method and depends on the feature `half`.

