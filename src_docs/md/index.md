# Vexy JSON Documentation

## TLDR

**Vexy JSON** is a forgiving JSON parser built in Rust that extends standard JSON with developer-friendly features while maintaining high performance and full compatibility with existing JSON. Unlike strict parsers, Vexy JSON accepts trailing commas, comments, unquoted keys, and more - making configuration files and development workflows smoother.

**Key Features:**
- 🚀 **High Performance**: Blazing fast parsing with zero-copy optimizations
- 🛡️ **Forgiving**: Accepts non-standard but commonly used JSON extensions
- 🌐 **Multi-Language**: Rust native with Python, JavaScript/WASM, and C/C++ bindings
- 🔧 **Developer Friendly**: Detailed error messages with repair suggestions
- 📦 **Production Ready**: Battle-tested with comprehensive test suite

## Table of Contents

### [1. Getting Started](getting-started.md)
Learn how to install and start using Vexy JSON in your projects. Covers installation for Rust, Python, JavaScript, and C/C++, plus your first parsing examples.

### [2. Core Features](core-features.md)
Discover what makes Vexy JSON special - from standard JSON compatibility to forgiving extensions like comments, trailing commas, and unquoted keys.

### [3. API Reference](api-reference.md)
Complete API documentation for all parsing functions, configuration options, and return types across all supported languages.

### [4. Language Bindings](language-bindings.md)
Detailed guides for using Vexy JSON in Python, JavaScript (Node.js and browsers), C/C++, and integration patterns for each ecosystem.

### [5. Advanced Usage](advanced-usage.md)
Advanced parsing techniques including streaming, custom error handling, validation, transformation, and plugin development.

### [6. Performance Guide](performance-guide.md)
Optimization strategies, benchmarking results, memory management, and tips for maximizing parsing performance in your applications.

### [7. Troubleshooting](troubleshooting.md)
Common issues, error messages, debugging techniques, and solutions for platform-specific problems.

### [8. Contributing](contributing.md)
How to contribute to Vexy JSON development, including setting up the development environment, testing, and submitting pull requests.

### [9. Migration & Compatibility](migration-compatibility.md)
Migrating from other JSON parsers, compatibility matrices, and strategies for gradual adoption in existing codebases.

---

## Quick Example

```rust
use vexy_json::VexyJson;

// Parse forgiving JSON with comments and trailing commas
let json = r#"
{
    "name": "Vexy JSON", // This is a comment
    "features": [
        "fast",
        "forgiving",
        "multi-language", // Trailing comma is OK
    ],
    unquoted_key: "also works",
}
"#;

let parsed = VexyJson::parse(json)?;
println!("{:?}", parsed);
```

## Community & Support

- **GitHub**: [vexyart/vexy-json](https://github.com/vexyart/vexy-json)
- **Issues**: Report bugs and request features
- **Discussions**: Community Q&A and showcases
- **Crates.io**: [vexy-json](https://crates.io/crates/vexy-json)

---

*Get started with [installation and basic usage](getting-started.md) or jump to any section using the navigation above.*