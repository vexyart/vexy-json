---
layout: default
title: Usage Guide
nav_order: 2
permalink: /usage/
---

# Usage Guide v2.0.0

This guide provides in-depth examples for using `vexy_json` v2.0.0 in Rust and JavaScript/WebAssembly, including the new streaming API, parallel processing, and plugin system.

## Basic Parsing (Rust)

The simplest way to use vexy_json is with the `parse` function:

```rust
use vexy_json::parse;

fn main() {
    let json_data = r#"{ key: "value", num: 123, // comment\n trailing: [1,2,3,], hex: 0xFF }"#;
    let value = parse(json_data).unwrap();
    println!("{:?}", value);
}
```

## Customizing Parsing with `ParserOptions`

For more control, use `parse_with_options` and configure `ParserOptions`:

```rust
use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    let input = "a:1, b:2";
    let options = ParserOptions {
        allow_comments: true,
        allow_unquoted_keys: true,
        allow_trailing_commas: true,
        allow_implicit_top_level: true,
        allow_newline_as_comma: true,
        allow_single_quoted_strings: true,
        allow_extended_numbers: true,
        ..Default::default()
    };
    let value = parse_with_options(input, &options).unwrap();
    println!("{:?}", value);
}
```

## WebAssembly/JavaScript Usage

See [docs/wasm.md](wasm.md) for full API details.

```js
import init, { parse_json_with_options } from './pkg/vexy_json_wasm.js';

await init();
const result = parse_json_with_options('{a:1}', { allow_comments: true });
console.log(result); // { a: 1 }
```

## Customizing Parsing with `ParserOptions`

For more control over the parsing behavior, you can use `parse_with_options` and configure `ParserOptions`.

```rust
use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    // Example: Strict JSON parsing (disabling all forgiving features)
    let mut strict_options = ParserOptions::default();
    strict_options.allow_comments = false;
    strict_options.allow_trailing_commas = false;
    strict_options.allow_unquoted_keys = false;
    strict_options.allow_single_quotes = false;
    strict_options.implicit_top_level = false;
    strict_options.newline_as_comma = false;

    let strict_json = r#"{"key": "value"}"#;
    match parse_with_options(strict_json, strict_options) {
        Ok(value) => println!("Parsed strictly: {:?}", value),
        Err(e) => eprintln!("Strict parsing error: {}", e),
    }

    // Example: Allowing only unquoted keys and implicit top-level
    let mut custom_options = ParserOptions::default();
    custom_options.allow_unquoted_keys = true;
    custom_options.implicit_top_level = true;
    custom_options.allow_comments = false; // Keep other defaults or explicitly set

    let custom_json = r#"myKey: "myValue", another: 42"#;
    match parse_with_options(custom_json, custom_options) {
        Ok(value) => println!("Parsed with custom options: {:?}", value),
        Err(e) => eprintln!("Custom parsing error: {}", e),
    }
}
```

## Handling Forgiving Features

`vexy_json` excels at parsing JSON with common relaxations. Here are examples of how it handles them:

### Comments

Both single-line (`//`, `#`) and multi-line (`/* ... */`) comments are ignored.

```rust
use vexy_json::parse;

fn main() {
    let json_with_comments = r#"
        {
            // This is a single-line comment
            "name": "Alice", /* This is a
                                multi-line comment */
            "age": 30, # Another comment style
        }
    "#;
    let value = parse(json_with_comments).unwrap();
    println!("Parsed with comments: {:?}", value);
}
```

### Trailing Commas

Trailing commas in arrays and objects are gracefully handled.

```rust
use vexy_json::parse;

fn main() {
    let json_with_trailing_comma = r#"
        [
            1,
            2,
            3, // Trailing comma here
        ]
    "#;
    let value = parse(json_with_trailing_comma).unwrap();
    println!("Parsed with trailing comma: {:#?}", value);

    let obj_with_trailing_comma = r#"
        {
            key1: "value1",
            key2: "value2", // Trailing comma here
        }
    "#;
    let obj_value = parse(obj_with_trailing_comma).unwrap();
    println!("Parsed object with trailing comma: {:#?}", obj_value);
}
```

### Unquoted Keys

Object keys do not need to be quoted, as long as they are valid identifiers.

```rust
use vexy_json::parse;

fn main() {
    let json_unquoted_keys = r#"{ firstName: "John", lastName: "Doe" }"#;
    let value = parse(json_unquoted_keys).unwrap();
    println!("Parsed with unquoted keys: {:#?}", value);
}
```

### Implicit Top-Level Objects and Arrays

You don't need to wrap your entire input in `{}` or `[]` if it's clearly an object or an array.

```rust
use vexy_json::parse;

fn main() {
    // Implicit object
    let implicit_obj = r#"name: "Bob", age: 25"#;
    let obj_value = parse(implicit_obj).unwrap();
    println!("Parsed implicit object: {:#?}", obj_value);

    // Implicit array
    let implicit_arr = r#""apple", "banana", "cherry""#;
    let arr_value = parse(implicit_arr).unwrap();
    println!("Parsed implicit array: {:#?}", arr_value);
}
```

### Newline as Comma

When the `newline_as_comma` option is enabled, newlines can act as implicit comma separators.

```rust
use vexy_json::{parse_with_options, ParserOptions};

fn main() {
    let mut options = ParserOptions::default();
    options.newline_as_comma = true;

    let json_with_newlines = r#"
        [
            1
            2
            3
        ]
    "#;
    let value = parse_with_options(json_with_newlines, options).unwrap();
    println!("Parsed with newlines as commas: {:#?}", value);

    let obj_with_newlines = r#"
        {
            key1: "value1"
            key2: "value2"
        }
    "#;
    let obj_value = parse_with_options(obj_with_newlines, options).unwrap();
    println!("Parsed object with newlines as commas: {:#?}", obj_value);
}
```

## Error Handling

`vexy_json` returns a `Result<Value, Error>` which allows for robust error handling. You should always check the `Result` to handle potential parsing issues.

```rust
use vexy_json::parse;

fn main() {
    let invalid_json = r#"{ key: "value }"#; // Missing closing quote
    match parse(invalid_json) {
        Ok(value) => println!("Parsed: {:?}", value),
        Err(e) => eprintln!("Parsing error: {}", e),
    }
}
```

For more details on error types, refer to the [API Reference](api/).

## Streaming API Usage (New in v2.0.0)

The streaming API is ideal for processing large JSON files without loading them entirely into memory.

### Basic Streaming Example

```rust
use vexy_json::{StreamingParser, StreamingEvent};

fn process_large_file(json_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = StreamingParser::new();
    parser.feed(json_content)?;
    parser.finish()?;
    
    let mut depth = 0;
    while let Some(event) = parser.next_event()? {
        match event {
            StreamingEvent::StartObject => {
                println!("{:indent$}Object {", "", indent = depth * 2);
                depth += 1;
            }
            StreamingEvent::EndObject => {
                depth -= 1;
                println!("{:indent$}}}", "", indent = depth * 2);
            }
            StreamingEvent::ObjectKey(key) => {
                print!("{:indent$}{}: ", "", key, indent = depth * 2);
            }
            StreamingEvent::String(s) => println!("\"{}\"", s),
            StreamingEvent::Number(n) => println!("{}", n),
            StreamingEvent::Bool(b) => println!("{}", b),
            StreamingEvent::Null => println!("null"),
            StreamingEvent::EndOfInput => break,
            _ => {}
        }
    }
    Ok(())
}
```

### Incremental Parsing

Perfect for network streams or reading files in chunks:

```rust
use vexy_json::StreamingParser;
use std::io::{BufReader, BufRead};
use std::fs::File;

fn parse_file_incrementally(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut parser = StreamingParser::new();
    
    for line in reader.lines() {
        parser.feed(&line?)?;
        
        // Process available events after each line
        while let Some(event) = parser.next_event()? {
            // Handle events...
        }
    }
    
    parser.finish()?;
    Ok(())
}
```

## Parallel Processing (New in v2.0.0)

Process multiple JSON files or strings in parallel for improved performance.

### Basic Parallel Parsing

```rust
use vexy_json::{parse_parallel, ParallelOptions};
use std::fs;

fn process_json_files(directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<String> = fs::read_dir(directory)?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()? == "json" {
                    fs::read_to_string(path).ok()
                } else {
                    None
                }
            })
        })
        .collect();
    
    let results = parse_parallel(files);
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("File {} parsed successfully", i),
            Err(e) => eprintln!("Error in file {}: {}", i, e),
        }
    }
    
    Ok(())
}
```

### Custom Parallel Options

```rust
use vexy_json::{parse_parallel_with_options, ParallelOptions, ParserOptions};

let mut parallel_opts = ParallelOptions::default();
parallel_opts.num_threads = Some(8);  // Use 8 threads
parallel_opts.chunk_size = Some(100); // Process 100 items per chunk

let mut parser_opts = ParserOptions::default();
parser_opts.allow_comments = true;
parser_opts.allow_trailing_commas = true;

parallel_opts.parser_options = parser_opts;

let results = parse_parallel_with_options(json_strings, parallel_opts);
```

## Plugin System (New in v2.0.0)

Extend vexy_json with custom functionality through plugins.

### Creating a Custom Plugin

```rust
use vexy_json::{Plugin, Value, Error};
use std::collections::HashMap;

// Plugin to redact sensitive information
struct RedactPlugin {
    sensitive_keys: Vec<String>,
}

impl Plugin for RedactPlugin {
    fn name(&self) -> &str {
        "redact-sensitive"
    }
    
    fn transform(&self, value: &mut Value) -> Result<(), Error> {
        match value {
            Value::Object(map) => {
                for key in &self.sensitive_keys {
                    if map.contains_key(key) {
                        map.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                    }
                }
                // Recursively process nested objects
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

// Usage
let plugin = RedactPlugin {
    sensitive_keys: vec!["password".to_string(), "api_key".to_string()],
};

let plugins: Vec<Box<dyn Plugin>> = vec![Box::new(plugin)];
let value = parse_with_plugins(json_str, ParserOptions::default(), &plugins)?;
```

### Validation Plugin Example

```rust
struct SchemaValidatorPlugin {
    required_fields: Vec<String>,
}

impl Plugin for SchemaValidatorPlugin {
    fn name(&self) -> &str {
        "schema-validator"
    }
    
    fn transform(&self, _value: &mut Value) -> Result<(), Error> {
        Ok(()) // No transformation needed
    }
    
    fn validate(&self, value: &Value) -> Result<(), Error> {
        if let Value::Object(map) = value {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(Error::Custom(
                        format!("Missing required field: {}", field)
                    ));
                }
            }
        }
        Ok(())
    }
}
```

## NDJSON (Newline-Delimited JSON) Support (New in v2.0.0)

Process streams of JSON objects separated by newlines.

```rust
use vexy_json::NdJsonParser;

fn process_log_file(log_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = NdJsonParser::new();
    let entries = parser.feed(log_content)?;
    
    println!("Processed {} log entries", entries.len());
    
    for (i, entry) in entries.iter().enumerate() {
        if let Some(timestamp) = entry.get("timestamp") {
            println!("Entry {}: {:?}", i, timestamp);
        }
    }
    
    Ok(())
}

// Example input:
// {"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "Server started"}
// {"timestamp": "2024-01-01T00:01:00Z", "level": "ERROR", "message": "Connection failed"}
// {"timestamp": "2024-01-01T00:02:00Z", "level": "INFO", "message": "Retry successful"}
```

## Advanced CLI Usage (New in v2.0.0)

The v2.0.0 CLI includes powerful new features:

### Watch Mode
```bash
# Watch a file for changes and reformat on save
vexy_json --watch config.json --output formatted-config.json

# Watch a directory
vexy_json --watch ./configs/ --output-dir ./formatted/
```

### Batch Processing
```bash
# Process multiple files in parallel
vexy_json --parallel *.json --output-dir ./processed/

# Apply transformations during batch processing
vexy_json --batch ./data/ --pretty --sort-keys --output-dir ./formatted/
```

### Plugin Usage
```bash
# Use built-in plugins
vexy_json input.json --plugin redact-passwords --plugin validate-schema

# Load custom plugin
vexy_json input.json --plugin-path ./my-plugin.wasm
```

For more details on the web tool, including its features and how to use it, refer to the [Web Tool documentation](web-tool.md).
