# String-Keyed CBOR Maps

This branch adds two features for encoding structs and enums as CBOR maps
with string keys instead of integer indices.

Based on [issue #52](https://github.com/twittner/minicbor/issues/52) and
the `#[s("...")]` string index support from @twittner's
[PR #53](https://github.com/twittner/minicbor/pull/53).

## `#[s("...")]` per-field string index

Works like `#[n(0)]`, but uses a string key:

```rust
use minicbor::{Encode, Decode};

#[derive(Encode, Decode)]
#[cbor(map)]
struct SensorData {
    #[s("temperature")]
    temperature: i16,
    #[s("humidity")]
    humidity: u8,
    #[n(2)]
    flags: u8, // numeric and string indices can coexist
}
```

Wire format (CBOR diagnostic notation):

```
{"temperature": 23, "humidity": 65, 2: 7}
```

## `#[cbor(text_keys)]` for automatic string keys

Applies string keys to all fields automatically, using their Rust
identifiers. No per-field annotations needed:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    active: bool,
}
```

Wire format:

```
{"temperature": 23, "humidity": 65, "active": true}
```

`text_keys` implies map encoding, so `#[cbor(map)]` is not required.

### Renaming keys

Use `#[cbor(key = "...")]` to override individual key names:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct SensorData {
    #[cbor(key = "t")]
    temperature: i16,
    #[cbor(key = "h")]
    humidity: u8,
    active: bool,
}
```

Wire format:

```
{"t": 23, "h": 65, "active": true}
```

### Optional fields

`Option<T>` fields that are `None` are omitted from the map:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct Config {
    name: u8,
    #[cbor(key = "interval_ms")]
    interval: Option<u32>,
}

// Config { name: 1, interval: None }      encodes as {"name": 1}
// Config { name: 1, interval: Some(500) } encodes as {"name": 1, "interval_ms": 500}
```

### Skipping fields

`#[cbor(skip)]` excludes fields from encoding. They get `Default::default()`
on decode:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct State {
    value: u16,
    #[cbor(skip)]
    cached: u32, // not encoded, defaults to 0 on decode
}
```

### Nested structs

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct Inner { a: u8 }

#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct Outer {
    inner: Inner,
    b: u16,
}

// Outer { inner: Inner { a: 1 }, b: 2 }
// encodes as {"inner": {"a": 1}, "b": 2}
```

### Enums

Unit variants encode as a plain string, non-unit variants as a
single-entry map:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
enum Command {
    Reset,
    SetInterval { #[s("ms")] ms: u32 },
}

// Command::Reset                        encodes as "Reset"
// Command::SetInterval { ms: 500 }      encodes as {"SetInterval": {"ms": 500}}
```

### Generics

Trait bounds are derived automatically:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct Wrapper<T> {
    value: T,
    label: u8,
}
```

### Collections

Standard collection types work as field values:

```rust
use std::collections::BTreeMap;

#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct DataSet {
    readings: Vec<u16>,
    metadata: BTreeMap<u32, u16>,
    name: String,
}
```

## Decode behavior

- Unknown keys are skipped (forward compatible)
- Missing optional fields default to `None` (backward compatible)
- Missing required fields produce a decode error
- Key order does not matter
- Definite and indefinite-length maps are both supported

## Compile-time checks

The following produce compile errors:

- `text_keys` on a tuple struct (fields have no names)
- `text_keys` combined with `transparent` or `index_only`
- `key = "..."` without `text_keys` on the struct or enum
- Two fields with the same key name

## `no_std` and `no_alloc`

Both features work on `no_std` and `no_alloc` targets.
`Encoder::str()` writes directly, `Decoder::str()` borrows from the
input buffer. No heap allocation anywhere.
