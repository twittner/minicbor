# String-Keyed CBOR Maps

This branch adds two complementary features for encoding structs and enums
as CBOR maps with human-readable string keys.

It builds on the `#[s("...")]` string index support introduced by
@twittner in [PR #53](https://github.com/twittner/minicbor/pull/53)
and extends it with a convenience macro that automatically derives
string keys from field and variant names.

## `#[s("...")]` — Per-field string index

Annotate individual fields or variants with a string index, just like
`#[n(0)]` assigns a numeric index:

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

## `#[cbor(text_keys)]` — Automatic string keys from field names

Instead of annotating every field, apply `text_keys` at the struct or
enum level. Each field is automatically keyed by its Rust identifier:

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

No `#[cbor(map)]` is needed — `text_keys` implies map encoding.

### Renaming keys

Override individual key names with `#[cbor(key = "...")]`:

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

`Option<T>` fields that are `None` are omitted from the map entirely:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct Config {
    name: u8,
    #[cbor(key = "interval_ms")]
    interval: Option<u32>,
}

// Config { name: 1, interval: None } encodes as: {"name": 1}
// Config { name: 1, interval: Some(500) } encodes as: {"name": 1, "interval_ms": 500}
```

### Skipping fields

Fields annotated with `#[cbor(skip)]` are excluded from encoding and
initialized with `Default::default()` on decode:

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

Inner structs with `text_keys` produce nested string-keyed maps:

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
// encodes as: {"inner": {"a": 1}, "b": 2}
```

### Enums

Unit variants encode as a plain string. Non-unit variants encode as a
single-entry map wrapping the variant's fields:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
enum Command {
    Reset,
    SetInterval { #[s("ms")] ms: u32 },
}

// Command::Reset           → "Reset"
// Command::SetInterval { ms: 500 } → {"SetInterval": {"ms": 500}}
```

### Generic structs

Works with generics — the derived bounds are added automatically:

```rust
#[derive(Encode, Decode)]
#[cbor(text_keys)]
struct Wrapper<T> {
    value: T,
    label: u8,
}
```

### Collection fields

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

- **Unknown keys** are silently skipped (forward compatible)
- **Missing optional fields** default to `None` (backward compatible)
- **Missing required fields** produce a decode error
- **Key order** does not matter — the decoder handles any order
- Both definite and indefinite-length maps are supported

## Compile-time validation

The following invalid usages produce compile errors:

- `#[cbor(text_keys)]` on a tuple struct (fields have no names)
- `#[cbor(text_keys)]` combined with `transparent` or `index_only`
- `#[cbor(key = "...")]` without `text_keys` on the struct/enum
- Two fields with the same key name

## `no_std` / `no_alloc`

Both features are fully compatible with `no_std` and `no_alloc` targets.
String keys are encoded directly by the `Encoder::str()` method and decoded
zero-copy via `Decoder::str()` — no heap allocation is needed at any point.
