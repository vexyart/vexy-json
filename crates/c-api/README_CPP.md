# Vexy JSON C++ Header-Only Wrapper

This directory contains a modern C++ header-only wrapper for the Vexy JSON parser, providing an idiomatic C++ interface with RAII, exceptions, and STL integration.

## Features

- **Header-only**: Just include `vexy_json.hpp` - no additional C++ files to compile
- **RAII**: Automatic memory management with smart pointers
- **Exception safety**: Strong exception guarantee with proper error handling
- **Modern C++**: Uses C++17 features like `std::string_view` and `std::optional`
- **Fluent API**: Builder pattern for parser options
- **Zero-copy where possible**: Efficient string handling

## Requirements

- C++17 or later compiler
- The vexy_json C library (linked separately)

## Installation

1. Include the `vexy_json.hpp` header in your project
2. Link against the vexy_json C library

## Quick Start

```cpp
#include "vexy_json.hpp"

// Simple parsing
std::string json = vexy_json::parse(R"({"key": "value"})");

// Parsing with options
auto options = vexy_json::ParserOptions()
    .allowComments()
    .allowTrailingCommas()
    .enableRepair();
    
std::string result = vexy_json::parse(input, options);

// Using a parser instance
vexy_json::Parser parser(options);
std::string result = parser.parseToString(input);

// Detailed parsing with repair information
auto detailed = vexy_json::parseDetailed(input, options);
std::cout << "JSON: " << detailed.json() << "\n";
for (const auto& repair : detailed.repairs()) {
    std::cout << "Repair: " << repair.description << "\n";
}
```

## API Reference

### Namespace `vexy_json`

All C++ wrapper functionality is in the `vexy_json` namespace. This is consistent with the `vexy_json` Rust crate name.

### Classes

#### `ParserOptions`
Configuration for the parser with a fluent builder interface:
- `allowComments()` - Allow // and /* */ comments
- `allowTrailingCommas()` - Allow trailing commas in arrays/objects
- `allowUnquotedKeys()` - Allow unquoted object keys
- `allowSingleQuotes()` - Allow single-quoted strings
- `implicitTopLevel()` - Allow implicit top-level objects
- `newlineAsComma()` - Treat newlines as commas
- `maxDepth(uint32_t)` - Set maximum nesting depth
- `enableRepair()` - Enable automatic error repair
- `maxRepairs(uint32_t)` - Set maximum number of repairs
- `fastRepair()` - Use fast repair mode
- `reportRepairs()` - Include repair information in results

#### `Parser`
Main parser class for repeated parsing with the same options:
- `Parser()` - Create with default options
- `Parser(const ParserOptions&)` - Create with custom options
- `parse(std::string_view)` - Parse and return ParseResult
- `parseToString(std::string_view)` - Parse and return JSON string directly

#### `ParseResult`
Result of parsing operation:
- `hasError()` - Check if parsing failed
- `error()` - Get error message (throws if no error)
- `json()` - Get parsed JSON string (throws on error)

#### `DetailedParseResult`
Extended result with repair information:
- All methods from `ParseResult`
- `repairs()` - Get vector of repairs made

#### `Repair`
Information about a single repair:
- `type` - Type of repair made
- `position` - Position in input where repair was made
- `description` - Human-readable description

#### `ParseError`
Exception thrown on parse errors (inherits from `std::runtime_error`)

### Free Functions

- `parse(std::string_view)` - Quick parse with default options
- `parse(std::string_view, const ParserOptions&)` - Quick parse with options
- `parseDetailed(std::string_view, const ParserOptions&)` - Parse with repair info
- `version()` - Get vexy_json library version

## Examples

See `examples/cpp_example.cpp` for comprehensive usage examples.

## Building the Examples

```bash
# Assuming you have built the vexy_json C library
g++ -std=c++17 examples/cpp_example.cpp -lvexy_json -o cpp_example
./cpp_example
```

## Thread Safety

The `Parser` class is thread-safe for parsing (multiple threads can call `parse()` on the same parser instance). However, creating parsers and modifying options should be synchronized if done from multiple threads.

## Performance Tips

1. Reuse `Parser` instances when parsing multiple documents with the same options
2. Use `std::string_view` when possible to avoid string copies
3. Enable fast repair mode for better performance when repair accuracy is less critical
4. Consider using the C API directly for maximum performance in hot paths