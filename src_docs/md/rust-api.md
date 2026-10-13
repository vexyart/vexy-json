# Rust API

The `vexy-json` crate is a thin facade. It re-exports the parser from
`vexy-json-core` and, with its default `serde` feature, the items of
`vexy-json-serde`. Generated API documentation for each crate is on
[docs.rs/vexy-json](https://docs.rs/vexy-json).

## Parsing functions

```rust
pub fn parse(input: &str) -> vexy_json::Result<Value>;
pub fn parse_with_options(input: &str, options: ParserOptions) -> vexy_json::Result<Value>;
```

`parse` uses `ParserOptions::default()`, which turns on every syntax
extension. `parse_with_options` takes the options by value. Neither function
repairs input; for that, see `parse_with_fallback` in
[Repairing incomplete input](repair.md).

```rust
use vexy_json::{parse, parse_with_options, ParserOptions};

fn main() -> vexy_json::Result<()> {
    let value = parse("{name: 'Vexy', enabled: true,}")?;
    println!("{value}"); // {"name": "Vexy", "enabled": true}

    let options = ParserOptions { allow_comments: false, ..ParserOptions::default() };
    let value = parse_with_options("[1, 2, 3]", options)?;
    println!("{value}"); // [1, 2, 3]
    Ok(())
}
```

`vexy_json::Result<T>` is an alias for `std::result::Result<T, vexy_json::Error>`.

## `Value`

`Value` is the parsed document:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(FxHashMap<String, Value>),
}

pub enum Number {
    Integer(i64),
    Float(f64),
}
```

`Object` uses `FxHashMap` from the `rustc-hash` crate, so member order is not
kept.

### Reading values

| Method | Returns |
| --- | --- |
| `is_null()`, `is_bool()`, `is_number()`, `is_string()`, `is_array()`, `is_object()` | `bool` |
| `as_bool()` | `Option<bool>` |
| `as_i64()` | `Option<i64>` |
| `as_f64()` | `Option<f64>` |
| `as_str()` | `Option<&str>` |
| `as_array()` | `Option<&Vec<Value>>` |
| `as_object()` | `Option<&FxHashMap<String, Value>>` |

`Value` implements `Index<&str>` and `Index<usize>`. Indexing never panics: a
missing key, an index out of range or indexing the wrong type returns
`Value::Null`.

```rust
use vexy_json::{parse, Value};

let value = parse("{user: {name: 'Ada', roles: [admin, dev]}}").unwrap();

assert_eq!(value["user"]["name"].as_str(), Some("Ada"));
assert_eq!(value["user"]["roles"][1].as_str(), Some("dev"));
assert!(value["user"]["email"].is_null());

if let Value::Object(map) = &value["user"] {
    println!("{} members", map.len()); // 2 members
}
```

`Number::as_f64()` converts either variant to `f64`.

### Printing values

`Value` implements `Display`, which writes JSON-like text with a space after
each `:` and `,`. `Display` does not escape string contents: a string that
contains `"` or a newline is written as is. For guaranteed valid JSON output,
serialize with Serde (see below) or use the `vexy-json` CLI, which escapes
strings.

## `ParserOptions`

`ParserOptions` is a plain struct with public fields and a `Default`
implementation. The fields and their defaults are listed in
[Parser options](options.md). Use struct update syntax to change only some
fields:

```rust
use vexy_json::ParserOptions;

let options = ParserOptions {
    max_depth: 512,
    ..ParserOptions::default()
};
```

With the core crate's `serde` feature, `ParserOptions` also implements
`Serialize` and `Deserialize`, and missing fields take their defaults.

## Errors

`vexy_json::Error` (also exported as `ParseError`) is an enum built with
`thiserror`. Its `Display` text includes a byte position:

| Variant | Message |
| --- | --- |
| `UnexpectedChar(char, usize)` | `Unexpected character 'x' at position N` |
| `UnexpectedEof(usize)` | `Unexpected end of input at position N` |
| `InvalidNumber(usize)` | `Invalid number format at position N` |
| `InvalidEscape(usize)` | `Invalid string escape sequence at position N` |
| `InvalidUnicode(usize)` | `Invalid Unicode escape at position N` |
| `UnterminatedString(usize)` | `Unterminated string starting at position N` |
| `TrailingComma(usize)` | `Trailing comma at position N` |
| `Expected { expected, found, position }` | `Expected X but found Y at position N` |
| `DepthLimitExceeded(usize)` | `Depth limit exceeded at position N` |
| `Custom(String)` | `Custom error: …` |
| `RepairFailed(String)` | `JSON repair failed: …` |
| `BracketMismatch(usize, char, char)` | `Bracket mismatch at position N: expected X, found Y` |
| `UnbalancedBrackets(usize, usize)` | `Unbalanced brackets: N extra opening, M extra closing` |
| `MaxRepairsExceeded(usize)` | `Maximum repair attempts exceeded (N)` |

Positions are zero-based byte offsets into the input string. `Span` (start
and end byte offsets) is exported for code that works with tokens.

## Streaming

### Newline-delimited JSON

`NdJsonParser` parses one value per line. Feed it text in chunks of any size;
each call to `feed` returns the values completed so far, and `finish` returns
the last value if the input did not end with a newline.

```rust
use vexy_json::NdJsonParser;

fn main() -> vexy_json::Result<()> {
    let mut parser = NdJsonParser::new();
    let values = parser.feed("{\"id\": 1}\n{id: 2}\n")?;
    assert_eq!(values.len(), 2);
    let rest = parser.finish()?;
    assert!(rest.is_empty());
    Ok(())
}
```

`NdJsonParser::with_options(options)` applies custom `ParserOptions` to each
line. `line_number()` reports the current line.

### Event stream

`StreamingParser` produces `StreamingEvent` values as input arrives:
`StartObject`, `EndObject`, `StartArray`, `EndArray`, `ObjectKey(String)`,
`Null`, `Bool(bool)`, `Number(String)`, `String(String)` and `EndOfInput`.

```rust
use vexy_json::{StreamingEvent, StreamingParser};

fn main() -> vexy_json::Result<()> {
    let mut parser = StreamingParser::new();
    parser.feed(r#"{"name": "#)?;
    parser.feed(r#""Alice", "age": 30}"#)?;
    parser.finish()?;

    while let Some(event) = parser.next_event()? {
        match event {
            StreamingEvent::ObjectKey(key) => println!("key {key}"),
            StreamingEvent::EndOfInput => break,
            other => println!("{other:?}"),
        }
    }
    Ok(())
}
```

`StreamingValueBuilder` turns a sequence of events back into a `Value`. The
repository's `examples/streaming_example.rs` shows these types together; run
it with `cargo run --example streaming_example`.

## Serde

With the default `serde` feature, `vexy_json` re-exports `SerdeValue`, a
newtype around `Value` that implements `Serialize` and `Deserialize`, with
`From` conversions in both directions.

`Value` itself derives `Serialize` and `Deserialize` when the core crate's
`serde` feature is on. The derive keeps the enum structure, so
`serde_json::to_string(&value)` produces tagged output such as
`{"Object":{"a":{"Number":{"Integer":1}}}}`, not plain JSON.

## Other exports

`vexy-json-core` exports further entry points that the facade crate does not
re-export. Import them from `vexy_json_core` directly:

- Alternative parsers: `parse_iterative`, `parse_recursive`,
  `parse_optimized`, `parse_optimized_v2`, `parse_optimized_v3` and their
  `_with_options` and `_with_stats` variants.
- Repair: `parse_with_fallback`, `parse_with_detailed_repair_tracking`,
  `JsonRepairer`, `EnhancedParseResult`, `ParsingTier`, `RepairAction`.
- Large inputs: `parse_parallel`, `parse_ndjson_parallel`, `ParallelParser`,
  `ParallelConfig`, `parse_lazy` and `LazyParser`.
- Transformation: `normalize`, `optimize`, `JsonNormalizer`,
  `CanonicalNormalizer`, `AstOptimizer`.

These functions are public but are not covered by this guide; read their
doc comments on docs.rs before relying on them.
