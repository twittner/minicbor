# 0.5.1

Update `minicbor-derive` to v0.4.1 which adds the `#[cbor(transparent)]`
attribute.

# 0.5.0

Update `minicbor-derive` to v0.4.0 which adds `encode_with`, `decode_with`
and `with` attributes.

# 0.4.1

Add `Encoder::f16` to support encoding of `f32` values as half floats.
Complements the existing `Decoder::f16` method and depends on the feature
`half`.

