---
layout: default
title: Home
nav_order: 1
---

# Vexy JSON Documentation

**A forgiving JSON parser that handles real-world JSON with comments, trailing commas, unquoted keys, and more.**

---

## 🚀 Quick Start

### Try It Now
- **[Interactive Demo](demo.html)** - Test Vexy JSON in your browser with WASM
- **[Legacy Tool](tool.html)** - Previous version of the web tool

### Installation
```bash
# Rust
cargo add vexy-json

# Python
pip install vexy-json

# CLI
cargo install vexy-json
```

---

## Quick Start (Rust)

```rust
use vexy_json::parse;

fn main() {
    let data = r#"{ key: 1, /* comment */ arr: [1,2,3,], hex: 0x10 }"#;
    let value = parse(data).unwrap();
    println!("{:?}", value);
}
```

## 📚 Documentation

### For Users
- **[Getting Started](user/getting-started.md)** - Installation and basic usage
- **[Features Overview](user/features.md)** - What makes Vexy JSON special
- **[API Documentation](user/api/)** - Language-specific guides
  - [Rust API](user/api/rust.md)
  - [Python API](user/api/python-bindings.md)
  - [WebAssembly](user/api/wasm.md)
  - [Streaming API](user/api/streaming-api.md)
- **[User Guides](user/guides/)**
  - [Migration Guide](user/guides/migration.md)
  - [Troubleshooting](user/guides/troubleshooting.md)
  - [JSON Repair](user/guides/json-repair.md)
- **[Reference](user/reference/)**
  - [Release Notes](user/reference/release-notes.md)

### For Developers
- **[Contributing](dev/contributing.md)** - How to contribute to the project
- **[Developer Guide](dev/developer-guide.md)** - Architecture and internals
- **[Plugin Development](dev/plugin-development.md)** - Creating plugins
- **[Build Process](dev/build-process.md)** - Building and testing
- **[Release Process](dev/release-process.md)** - How releases are made
- **[Benchmarks](dev/benchmarks.md)** - Performance comparisons

---

## ✨ Key Features

### 💬 Comments Support
```json
{
    // Single-line comments
    "name": "example",
    /* Multi-line
       comments */ 
    "value": 42
}
```

### 🏷️ Unquoted Keys
```json
{
    name: "No quotes needed",
    version: 1.0,
    active: true
}
```

### ➕ Trailing Commas
```json
{
    "items": [
        "first",
        "second",  // <- This comma is OK
    ],
    "done": true,  // <- And this one too
}
```

### 🔧 Error Recovery
```json
{
    "broken": "json,
    "gets": "fixed automatically"
}
```

---

## 🎯 Use Cases

- **Configuration Files** - More readable config with comments
- **API Development** - Forgiving parsing for client-side JSON
- **Data Migration** - Repair malformed JSON data
- **Developer Tools** - Build JSON editors and validators
- **Log Processing** - Handle JSON logs with comments

---

## 🌟 Performance

Vexy JSON is designed for both **correctness** and **speed**:

- ⚡ **Fast parsing** - Competitive with standard JSON parsers
- 🧠 **Smart recovery** - Fixes common JSON errors automatically  
- 🌐 **Multi-platform** - Rust, Python, WebAssembly, and C/C++ bindings
- 🔒 **Memory safe** - Built in Rust with comprehensive error handling

---

## 🔗 Links

- **[GitHub Repository](https://github.com/vexyart/vexy-json)** - Source code and issues
- **[Crates.io](https://crates.io/crates/vexy-json)** - Rust package
- **[PyPI](https://pypi.org/project/vexy-json/)** - Python package
- **[NPM](https://www.npmjs.com/package/@vexyart/vexy-json)** - WebAssembly package

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.
