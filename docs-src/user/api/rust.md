---
nav_title: Rust API
nav_order: 2
---

# Rust API

```bash
cargo add vexy-json
```

The crate name in `Cargo.toml` is `vexy-json`; Rust code refers to it as
`vexy_json`.

## Core entry points

```rust
pub fn parse(input: &str) -> Result<Value, Error>
pub fn parse_with_options(input: &str, options: ParserOptions) -> Result<Value, Error>
```

`parse` uses forgiving defaults. `parse_with_options` accepts an owned
`ParserOptions` value.

```rust
use vexy_json::{parse_with_options, ParserOptions, Result};

fn main() -> Result<()> {
    let options = ParserOptions {
        max_depth: 64,
        max_repairs: 10,
        ..ParserOptions::default()
    };
    let value = parse_with_options("name: 'Ada'", options)?;
    assert_eq!(value["name"].as_str(), Some("Ada"));
    Ok(())
}
```

## `ParserOptions`

The public fields control comments, trailing commas, unquoted keys, single
quotes, implicit containers, newline separators, maximum depth, and repair
behavior. Defaults enable forgiving parsing with `max_depth = 128` and
`max_repairs = 100`.

Build strict behavior by disabling every syntax extension and repair. There is
no hidden global parser configuration; options travel with the parse call.

## `Value` and `Number`

`Value` represents null, boolean, number, string, array, and object values.
Use `is_null`, `is_bool`, `is_number`, `is_string`, `is_array`, and `is_object`
for inspection, then `as_bool`, `as_i64`, `as_f64`, `as_str`, `as_array`, and
`as_object` for typed access.

`Number` distinguishes integer and floating-point storage. Conversions return
`Option` when the requested representation is not available.

## Errors

Parsing returns `vexy_json::Result<T>`, whose error carries the failure category
and position information where available. Propagate it with `?` in libraries or
render it for users at the application boundary.

```rust
match vexy_json::parse("{message: 'unfinished}") {
    Ok(value) => println!("{value}"),
    Err(error) => eprintln!("Could not parse input: {error}"),
}
```

## Serde

The default `serde` feature exposes Serde integration through the companion
`vexy-json-serde` crate. Disable default features when you only need the core
value model:

```toml
[dependencies]
vexy-json = { version = "1", default-features = false }
```

## Streaming and NDJSON

The crate re-exports `StreamingParser`, `StreamingEvent`,
`StreamingValueBuilder`, `NdJsonParser`, and `SimpleStreamingLexer`. Prefer the
NDJSON parser when records are independently delimited by lines. Streaming APIs
have their own completion contract; drain events until the iterator ends rather
than assuming `EndOfInput` repeats.

See [Troubleshooting](../guides/troubleshooting.md) for malformed input and
installation diagnostics.
