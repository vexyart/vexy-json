---
layout: default
title: Home
nav_order: 1
---

# Welcome to vexy_json v2.0.0

`vexy_json` is a forgiving JSON parser written in Rust, inspired by the JavaScript library `jsonic`. It provides robust, flexible parsing for both strict and non-standard JSON, supporting features like comments, unquoted keys, trailing commas, implicit arrays/objects, and more. vexy_json is available as a Rust library, CLI tool, and WebAssembly module for browser/Node.js usage.

## 🚀 New in Version 2.0.0

- **Streaming API**: Process large JSON files incrementally with minimal memory usage
- **Parallel Processing**: Multi-threaded JSON processing for batch operations
- **Plugin Architecture**: Extensible framework for custom transformations and validators
- **Enhanced CLI**: Watch mode, batch processing, and advanced formatting options
- **NDJSON Support**: Native support for newline-delimited JSON streams

## Key Features

- Forgiving parsing: comments, unquoted keys, trailing commas, implicit arrays/objects
- Extended number formats: hex, octal, binary, underscores
- Strict mode for RFC 8259 compliance
- WebAssembly/JavaScript support
- Interactive web tool
- **NEW**: Streaming parser for large files
- **NEW**: Parallel processing capabilities
- **NEW**: Plugin system for extensibility

## Try It Online

🚀 **[Launch the Web Tool](tool.html)** — Parse forgiving JSON in your browser!

## Quick Start (Rust)

```rust
use vexy_json::parse;

fn main() {
    let data = r#"{ key: 1, /* comment */ arr: [1,2,3,], hex: 0x10 }"#;
    let value = parse(data).unwrap();
    println!("{:?}", value);
}
```

## Documentation

- [Usage Guide](usage.md)
- [Forgiving Features](features.md)
- [Web Tool](web-tool.md)
- [WASM API Reference](wasm.md)
- [Troubleshooting](troubleshooting.md)

## Contributing

See [Contributing](contributing.md) for how to help improve vexy_json.

📦 **[Download the latest CLI release](https://github.com/twardoch/vexy_json/releases/latest)** - Get the `vexy_json` command-line interface for your platform.

## Features

- **Forgiving Parsing**: Handles comments, trailing commas, unquoted keys, and implicit top-level objects/arrays.
- **Rust Idiomatic API**: Designed with Rust's ownership, borrowing, and error handling principles in mind.
- **Performance**: Optimized for speed and efficiency.
- **Serde Integration**: Seamlessly convert `vexy_json::Value` to and from other data formats using the `serde` framework.
- **WebAssembly (WASM) Bindings**: Use `vexy_json` directly in JavaScript environments.
- **Interactive Web Tool**: Browser-based parser with real-time feedback and sharing capabilities.
- **Compatibility**: Aims for API compatibility with the original `jsonic.js` where appropriate.

## Getting Started

To use `vexy_json` in your Rust project, add it to your `Cargo.toml`:

```toml
[dependencies]
vexy_json = "2.0.0" # Replace with the latest version
```

Then, you can parse JSON-like strings:

```rust
use vexy_json::parse;

fn main() {
    let json_str = r#"
        {
            // This is a comment
            name: "John Doe",
            age: 30,
            hobbies: [
                "reading",
                "hiking", // Trailing comma
            ],
        }
    "#;

    match parse(json_str) {
        Ok(value) => {
            println!("Parsed successfully: {:?}", value);
        }
        Err(e) => {
            eprintln!("Parsing error: {}", e);
        }
    }
}
```

## Documentation

- [API Reference](api/)
- [Usage Guide](usage/)
- [Forgiving Features](features/)
- [Streaming API](streaming-api/)
- [Release Notes](release-notes/)
- [Contributing](contributing/)

## Project Status

`vexy_json` v2.0.0 is production-ready with comprehensive features including streaming parsing, parallel processing, and a plugin architecture. We welcome contributions to expand the ecosystem!

## License

`vexy_json` is distributed under the MIT License. See the [LICENSE](https://github.com/twardoch/vexy_json/blob/main/LICENSE) file for more details.
