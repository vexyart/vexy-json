---
nav_title: Migration Guide
nav_order: 10
---

# Migration Guide: vexy_json v2.0.0

This document provides comprehensive guidance for upgrading to vexy_json v2.0.0 from previous versions.

## Migrating from v1.x to v2.0.0

### Overview

Version 2.0.0 is a major release that introduces powerful new features while maintaining backward compatibility for most existing code. The core parsing API remains unchanged, but new APIs have been added for streaming, parallel processing, and plugins.

### ✅ Backward Compatible Changes

The following APIs work exactly as before:
- `parse(input: &str) -> Result<Value>`
- `parse_with_options(input: &str, options: ParserOptions) -> Result<Value>`
- All `Value` enum methods and traits
- All `ParserOptions` fields
- CLI basic functionality

### 🚀 New Features to Adopt

#### Streaming API
If you're parsing large files, consider migrating to the streaming API:

**Before (v1.x):**
```rust
let large_json = std::fs::read_to_string("huge.json")?;
let value = parse(&large_json)?; // Uses lots of memory
```

**After (v2.0.0):**
```rust
use vexy_json::StreamingParser;

let mut parser = StreamingParser::new();
let file = std::fs::File::open("huge.json")?;
let reader = std::io::BufReader::new(file);

for line in reader.lines() {
    parser.feed(&line?)?;
}
parser.finish()?;

// Process events incrementally
while let Some(event) = parser.next_event()? {
    // Handle events with minimal memory usage
}
```

#### Parallel Processing
For batch operations, use the new parallel API:

**Before (v1.x):**
```rust
let mut results = Vec::new();
for json in json_files {
    results.push(parse(&json));
}
```

**After (v2.0.0):**
```rust
use vexy_json::parse_parallel;

let results = parse_parallel(json_files); // Automatically uses multiple cores
```

### ⚠️ Minor Breaking Changes

1. **Error Enum Reorganization**
   - Some error variants have been renamed for clarity
   - Add explicit imports if you match on specific error types:
   ```rust
   use vexy_json::Error::{UnexpectedChar, InvalidNumber};
   ```

2. **Feature Flags**
   - `wasm-bindgen` feature renamed to `wasm`
   - `full` feature now includes streaming and parallel features

3. **WASM JavaScript API**
   - Now uses consistent camelCase:
   - `parse_json` → `parseJson`
   - `parse_json_with_options` → `parseJsonWithOptions`

### 📦 Dependency Updates

If you depend on specific versions of vexy_json's dependencies:
- `serde`: Now requires 1.0.190+
- `wasm-bindgen`: Updated to 0.2.90
- New dependencies: `rayon`, `crossbeam-channel`, `simd-json`

### 🔧 CLI Changes

The CLI has been significantly enhanced. Update scripts that use vexy_json:

**New capabilities:**
```bash
# Watch mode
vexy_json --watch input.json -o output.json

# Batch processing
vexy_json --batch ./data/ --output-dir ./processed/

# Pretty printing with options
vexy_json --pretty --sort-keys --indent 4 input.json
```

---

# Migration Guide: vexy_json v0.2.0

This section covers the earlier v0.2.0 refactor for historical reference.

## Summary

The refactor focused on **internal improvements** while maintaining **full backward compatibility** for the public API. Most users should be able to upgrade without any code changes.

## ✅ No Breaking Changes

The following public APIs remain **unchanged** and fully compatible:

- `parse(input: &str) -> Result<Value>`
- `parse_with_options(input: &str, options: ParserOptions) -> Result<Value>`
- `ParserOptions` struct and all its fields
- `Value` enum and all its variants
- `Error` enum and existing error types
- WASM bindings and JavaScript API

## ✨ New Features Added

### Enhanced Error Handling

**New exports available:**
```rust
use vexy_json::{ParseResult, Error};

// New type alias for semantic clarity
fn parse_config() -> ParseResult<Config> {
    // ParseResult<T> is equivalent to Result<T, Error>
    // but provides semantic clarity for parsing operations
}

// Enhanced error context (automatically available)
match parse(input) {
    Err(error) => {
        // New error methods available
        if error.is_string_error() { /* handle string errors */ }
        if error.is_number_error() { /* handle number errors */ }
        if error.is_structural_error() { /* handle syntax errors */ }
    }
}
```

### Enhanced WASM API

**New JavaScript functions:**
```javascript
// Enhanced error objects with more information
try {
    const result = vexy_json.parse_json(input);
} catch (error) {
    console.log(error.message);        // Error description
    console.log(error.position);       // Character position (if available)
    console.log(error.isStringError);  // Error categorization
    console.log(error.isNumberError);
    console.log(error.isStructuralError);
}
```

## 🔧 Internal Improvements

The following improvements enhance performance and maintainability without affecting the public API:

### Architecture
- **Modular error system**: Enhanced error types with source chain support
- **Property-based testing**: Comprehensive test coverage with `proptest`
- **Better WASM integration**: Enhanced JavaScript error objects

### Performance
- **Optimized WASM bindings**: Latest wasm-bindgen with smaller bundle size
- **Enhanced CI/CD**: Multi-toolchain testing and security audits

### Development Experience
- **Enhanced error messages**: More precise error positioning and context
- **Better documentation**: Comprehensive API docs and examples
- **Improved CI/CD**: Enhanced testing matrix and security audits

## 📚 Recommended Usage Patterns

### For Rust Users

```rust
use vexy_json::{parse, ParseResult, ParserOptions};

// Recommended: Use the new ParseResult type for clarity
fn parse_config_file(content: &str) -> ParseResult<Config> {
    let options = ParserOptions::default(); // All forgiving features enabled
    let value = parse_with_options(content, options)?;
    // Convert value to your config struct...
    Ok(config)
}

// Error handling with enhanced categorization
match parse(input) {
    Ok(value) => println!("Parsed: {}", value),
    Err(error) => {
        if error.is_string_error() {
            eprintln!("String parsing error at position {:?}: {}", 
                     error.position(), error);
        } else {
            eprintln!("Parse error: {}", error);
        }
    }
}
```

### For JavaScript Users

```javascript
// Enhanced error handling with structured error objects
try {
    const result = vexy_json.parse_json(jsonString);
    console.log('Parsed:', result);
} catch (error) {
    console.error(`Parse error at position ${error.position}: ${error.message}`);
    
    // Enhanced error categorization
    if (error.isStringError) {
        console.log('This is a string-related parsing error');
    }
}
```

## 🚀 Future Compatibility

This refactor establishes a solid foundation for future enhancements:

- **Enhanced error reporting**: Better error context and source chains
- **Modular architecture**: Clean separation enables targeted optimizations
- **Comprehensive testing**: Property-based tests ensure robust behavior
- **Security auditing**: Automated dependency and security checks

## 📞 Support

If you encounter any issues during migration:

1. **Check compatibility**: Ensure you're not using any undocumented internal APIs
2. **Update imports**: Make sure you're importing from the main `vexy_json` crate
3. **Test thoroughly**: Run your existing test suite to verify behavior
4. **Report issues**: File bug reports with specific reproduction cases

## 📈 Benefits Summary

After migration, you'll benefit from:

- ✅ **Same API**: No code changes required for most users
- ✅ **Better errors**: More precise error reporting and categorization  
- ✅ **Enhanced WASM**: Better JavaScript integration with structured errors
- ✅ **Improved performance**: Optimized internal architecture
- ✅ **Future-proof**: Foundation for upcoming features and optimizations

The refactor maintains the reliability you expect while providing a foundation for continued improvements.