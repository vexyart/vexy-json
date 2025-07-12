---
layout: default
title: API Reference
nav_order: 3
permalink: /api/
---

# API Reference v2.0.0

This section provides detailed documentation for the `vexy_json` Rust library v2.0.0. The API is designed to be intuitive and idiomatic for Rust developers, with powerful new features for streaming, parallel processing, and extensibility.

## `vexy_json::parse`

```rust
pub fn parse(input: &str) -> Result<Value, Error>
```

Parses a JSON-like string into a `vexy_json::Value` enum using default parser options. This is the primary entry point for using the library.

- `input`: The string slice containing the JSON-like data to parse.
- Returns:
    - `Ok(Value)`: If parsing is successful, returns a `Value` enum representing the parsed data.
    - `Err(Error)`: If an error occurs during parsing, returns an `Error` detailing the issue.

## `vexy_json::parse_with_options`

```rust
pub fn parse_with_options(input: &str, options: ParserOptions) -> Result<Value, Error>
```

Parses a JSON-like string into a `vexy_json::Value` enum with custom parser options. This allows fine-grained control over which forgiving features are enabled.

- `input`: The string slice containing the JSON-like data to parse.
- `options`: A `ParserOptions` struct configuring the parser's behavior.
- Returns:
    - `Ok(Value)`: If parsing is successful, returns a `Value` enum representing the parsed data.
    - `Err(Error)`: If an error occurs during parsing, returns an `Error` detailing the issue.

## `vexy_json::ParserOptions`

This struct defines the configurable options for the `vexy_json` parser.

```rust
pub struct ParserOptions {
    pub allow_comments: bool,
    pub allow_trailing_commas: bool,
    pub allow_unquoted_keys: bool,
    pub allow_single_quotes: bool,
    pub implicit_top_level: bool,
    pub newline_as_comma: bool,
    pub max_depth: usize,
}
```

- `allow_comments`: If `true`, allows single-line (`//`, `#`) and multi-line (`/* */`) comments. Default: `true`.
- `allow_trailing_commas`: If `true`, allows trailing commas in arrays and objects. Default: `true`.
- `allow_unquoted_keys`: If `true`, allows object keys without quotes (e.g., `key: "value"`). Default: `true`.
- `allow_single_quotes`: If `true`, allows strings to be enclosed in single quotes (`'`). Default: `true`.
- `implicit_top_level`: If `true`, attempts to parse input not wrapped in `{}` or `[]` as an implicit top-level object or array. Default: `true`.
- `newline_as_comma`: If `true`, treats newlines as comma separators in arrays and objects. Default: `true`.
- `max_depth`: Maximum recursion depth for nested structures to prevent stack overflow. Default: `128`.

`ParserOptions` implements `Default`, so you can create a default instance and then modify specific fields:

```rust
use vexy_json::ParserOptions;

let mut options = ParserOptions::default();
options.allow_comments = false; // Disable comments
options.max_depth = 64; // Set a custom max depth
```

## `vexy_json::Value` Enum

This enum represents the different types of JSON values that `vexy_json` can parse.

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

- `Null`: Represents a JSON `null` value.
- `Bool(bool)`: Represents a JSON boolean (`true` or `false`).
- `Number(Number)`: Represents a JSON numeric value. See `vexy_json::Number` for details.
- `String(String)`: Represents a JSON string.
- `Array(Vec<Value>)`: Represents a JSON array, a vector of `Value` enums.
- `Object(HashMap<String, Value>)`: Represents a JSON object, a hash map of string keys to `Value` enums.

### `Value` Helper Methods

The `Value` enum provides several helper methods for type checking and value extraction:

- `is_null() -> bool`
- `is_bool() -> bool`
- `is_number() -> bool`
- `is_string() -> bool`
- `is_array() -> bool`
- `is_object() -> bool`
- `as_bool() -> Option<bool>`
- `as_i64() -> Option<i64>`: Returns `None` if the number cannot be represented as `i64`.
- `as_f64() -> Option<f64>`
- `as_str() -> Option<&str>`
- `as_array() -> Option<&Vec<Value>>`
- `as_object() -> Option<&HashMap<String, Value>>`

## `vexy_json::Number` Enum

This enum represents a JSON number, which can be either an integer or a floating-point number.

```rust
pub enum Number {
    Integer(i64),
    Float(f64),
}
```

- `Integer(i64)`: An integer value that fits in an `i64`.
- `Float(f64)`: A floating-point value.

## `vexy_json::Error` Enum

This enum defines the types of errors that can occur during parsing.

```rust
pub enum Error {
    UnexpectedChar(char, usize),
    UnexpectedEof(usize),
    InvalidNumber(usize),
    InvalidEscape(usize),
    InvalidUnicode(usize),
    UnterminatedString(usize),
    TrailingComma(usize),
    Expected {
        expected: String,
        found: String,
        position: usize,
    },
    DepthLimitExceeded(usize),
    Custom(String),
}
```

- `UnexpectedChar(char, usize)`: Encountered an unexpected character during parsing at a given position.
- `UnexpectedEof(usize)`: Reached the end of the input unexpectedly at a given position.
- `InvalidNumber(usize)`: An invalid number format was encountered at a given position.
- `InvalidEscape(usize)`: An invalid escape sequence was found in a string at a given position.
- `InvalidUnicode(usize)`: An invalid Unicode escape sequence was found at a given position.
- `UnterminatedString(usize)`: A string literal was not properly terminated, starting at a given position.
- `TrailingComma(usize)`: A trailing comma was found where not allowed (though typically allowed by `vexy_json`'s forgiving nature, this error might occur in strict modes or specific contexts) at a given position.
- `Expected { expected: String, found: String, position: usize }`: The parser expected a specific token or value but found something else at a given position.
- `DepthLimitExceeded(usize)`: The maximum recursion depth was exceeded while parsing nested structures at a given position.
- `Custom(String)`: A custom error with a descriptive message.

### `Error` Helper Methods

- `position() -> Option<usize>`: Returns the character position in the input where the error occurred, if available.

## Serde Integration

`vexy_json` provides optional integration with the `serde` serialization framework. When the `serde` feature is enabled in your `Cargo.toml`, `vexy_json::Value` and `vexy_json::Number` implement the `Serialize` and `Deserialize` traits. This allows easy conversion between `vexy_json::Value` and other data formats supported by Serde (e.g., `serde_json::Value`).

To enable this feature, add `serde` to your `vexy_json` dependency in `Cargo.toml`:

```toml
[dependencies]
vexy_json = { version = "2.0.0", features = ["serde"] }
```

**Example:**

```rust
use vexy_json::{parse, Value};
use serde_json; // Requires `serde_json` crate

fn main() {
    let json_str = r#"{ "name": "Alice", "age": 30 }"#;
    let vexy_json_value: Value = parse(json_str).unwrap();

    // Convert vexy_json::Value to serde_json::Value
    let serde_value: serde_json::Value = serde_json::to_value(vexy_json_value).unwrap();
    println!("Converted to serde_json::Value: {}", serde_value);

    // Convert serde_json::Value back to vexy_json::Value
    let new_vexy_json_value: Value = serde_json::from_value(serde_value).unwrap();
    println!("Converted back to vexy_json::Value: {:?}", new_vexy_json_value);
}
```

## WebAssembly (WASM) Bindings

`vexy_json` offers WebAssembly bindings, allowing it to be used directly in JavaScript environments (e.g., web browsers, Node.js). This is enabled via the `wasm` feature.

To enable this feature, add `wasm` to your `vexy_json` dependency in `Cargo.toml`:

```toml
[dependencies]
vexy_json = { version = "2.0.0", features = ["wasm"] }
```

For detailed documentation on the WebAssembly API, including JavaScript examples, please refer to the [WASM API Reference](wasm/).

## Streaming API (New in v2.0.0)

`vexy_json` v2.0.0 introduces a powerful streaming parser for processing large JSON files incrementally.

### `vexy_json::StreamingParser`

```rust
pub struct StreamingParser { /* ... */ }

impl StreamingParser {
    pub fn new() -> Self;
    pub fn with_options(options: ParserOptions) -> Self;
    pub fn feed(&mut self, input: &str) -> Result<(), Error>;
    pub fn finish(&mut self) -> Result<(), Error>;
    pub fn next_event(&mut self) -> Result<Option<StreamingEvent>, Error>;
}
```

Example usage:
```rust
use vexy_json::{StreamingParser, StreamingEvent};

let mut parser = StreamingParser::new();
parser.feed(r#"{"key": "value"}"#)?;
parser.finish()?;

while let Some(event) = parser.next_event()? {
    match event {
        StreamingEvent::StartObject => println!("Object started"),
        StreamingEvent::ObjectKey(key) => println!("Key: {}", key),
        StreamingEvent::String(s) => println!("String: {}", s),
        StreamingEvent::EndObject => println!("Object ended"),
        StreamingEvent::EndOfInput => break,
        _ => {}
    }
}
```

### `vexy_json::StreamingEvent`

```rust
pub enum StreamingEvent {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    ObjectKey(String),
    Null,
    Bool(bool),
    Number(String),
    String(String),
    EndOfInput,
}
```

## Parallel Processing (New in v2.0.0)

`vexy_json` v2.0.0 includes parallel processing capabilities for batch operations using the `rayon` crate.

### `vexy_json::parse_parallel`

```rust
pub fn parse_parallel<I>(inputs: I) -> Vec<Result<Value, Error>>
where
    I: IntoParallelIterator,
    I::Item: AsRef<str>,
```

Process multiple JSON strings in parallel:

```rust
use vexy_json::parse_parallel;

let json_strings = vec![
    r#"{"id": 1, "name": "Alice"}"#,
    r#"{"id": 2, "name": "Bob"}"#,
    r#"{"id": 3, "name": "Charlie"}"#,
];

let results = parse_parallel(json_strings);
for (i, result) in results.iter().enumerate() {
    match result {
        Ok(value) => println!("Parsed {}: {:?}", i, value),
        Err(e) => eprintln!("Error parsing {}: {}", i, e),
    }
}
```

### `vexy_json::ParallelOptions`

```rust
pub struct ParallelOptions {
    pub parser_options: ParserOptions,
    pub num_threads: Option<usize>,
    pub chunk_size: Option<usize>,
}
```

## Plugin System (New in v2.0.0)

`vexy_json` v2.0.0 introduces a plugin architecture for extending parsing capabilities.

### `vexy_json::Plugin` Trait

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn transform(&self, value: &mut Value) -> Result<(), Error>;
    fn validate(&self, value: &Value) -> Result<(), Error> {
        Ok(())
    }
}
```

Example plugin implementation:

```rust
use vexy_json::{Plugin, Value, Error};

struct DateNormalizerPlugin;

impl Plugin for DateNormalizerPlugin {
    fn name(&self) -> &str {
        "date-normalizer"
    }
    
    fn transform(&self, value: &mut Value) -> Result<(), Error> {
        // Transform date strings to ISO format
        match value {
            Value::String(s) => {
                if is_date_string(s) {
                    *s = normalize_date(s)?;
                }
            }
            Value::Object(map) => {
                for (_, v) in map.iter_mut() {
                    self.transform(v)?;
                }
            }
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    self.transform(v)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

### `vexy_json::parse_with_plugins`

```rust
pub fn parse_with_plugins(
    input: &str,
    options: ParserOptions,
    plugins: &[Box<dyn Plugin>]
) -> Result<Value, Error>
```

Usage example:
```rust
use vexy_json::{parse_with_plugins, ParserOptions};

let plugins: Vec<Box<dyn Plugin>> = vec![
    Box::new(DateNormalizerPlugin),
    Box::new(ValidationPlugin::new(schema)),
];

let value = parse_with_plugins(input, ParserOptions::default(), &plugins)?;
```

## NDJSON Support (New in v2.0.0)

### `vexy_json::NdJsonParser`

```rust
pub struct NdJsonParser { /* ... */ }

impl NdJsonParser {
    pub fn new() -> Self;
    pub fn with_options(options: ParserOptions) -> Self;
    pub fn feed(&mut self, input: &str) -> Result<Vec<Value>, Error>;
}
```

Example:
```rust
use vexy_json::NdJsonParser;

let mut parser = NdJsonParser::new();
let input = r#"{"id": 1}
{"id": 2}
{"id": 3}"#;

let values = parser.feed(input)?;
println!("Parsed {} objects", values.len());
```

