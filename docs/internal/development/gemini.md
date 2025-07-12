---
title: Gemini Development Guidelines
nav_order: 21
parent: Development
has_children: false
---

# Gemini Development Guidelines

This document provides guidance for Gemini AI when working with code in this repository.

## 1. Project Overview

`vexy_json` is a Rust port of the JavaScript library `the reference implementation`, a forgiving JSON parser. The reference JavaScript implementation is located in the `ref/the reference implementation/` directory.

## 2. Development Status

This project is in an active development phase. The core parsing engine is implemented, along with a comprehensive test suite, benchmarks, and WASM support. The focus is on achieving full API compatibility with `the reference implementation`, refining the idiomatic Rust API, and improving performance.

## 3. Rust Implementation

### 3.1. Module Organization

The Rust implementation is a cargo workspace organized into several crates:

-   `crates/core`: The core parsing engine.
    -   `src/lib.rs`: The main library crate root, exporting the public API.
    -   `src/parser.rs`: Contains the core recursive descent parsing logic.
    -   `src/lexer.rs`: The primary tokenizer for the input string.
    -   `src/ast/value.rs`: Defines the `Value` enum, which represents parsed JSON data.
    -   `src/error/mod.rs`: Implements custom error types for parsing failures.
-   `crates/cli`: The command-line interface.
    -   `src/main.rs`: The entry point for the CLI binary.
-   `crates/serde`: Provides `serde` integration for `vexy_json::Value`.
-   `crates/wasm`: Contains WebAssembly bindings to expose `vexy_json` to JavaScript environments.
-   `crates/test-utils`: Utility functions for testing.

### 3.2. Core Features

-   **Standard JSON Parsing (RFC 8259):** Full support for the official JSON specification.
-   **Forgiving Features:** Compatibility with `the reference implementation`'s non-standard features is a primary goal:
    -   Single-line (`//`) and multi-line (`/* */`) comments.
    -   Trailing commas in objects and arrays.
    -   Unquoted object keys (where unambiguous).
    -   Implicit top-level objects and arrays.
    -   Single-quoted strings.
    -   Newline characters as comma separators.

### 3.3. Architecture & Best Practices

-   **Error Handling:** Uses `Result<T, E>` and a custom `Error` enum (`src/error.rs`) for robust error handling with location information.
-   **Testing:**
    -   Unit and integration tests are located in the `tests/` directory, ported from `the reference implementation`'s test suite.
    -   The `examples/` directory contains numerous small, runnable programs for debugging specific features.
    -   Benchmarking is performed using `criterion.rs`, with benchmarks defined in the `benches/` directory.
-   **Extensibility:** The architecture uses Rust's traits and pattern matching for clarity and maintainability, avoiding a direct port of the JavaScript plugin system in favor of a more idiomatic approach.
-   **Performance:** The implementation aims for high performance, with ongoing benchmarking to compare against `serde_json` and `the reference implementation`.
-   **WASM Target:** A key feature is the ability to compile to WebAssembly, providing a performant `vexy_json` parser for web browsers and Node.js. The `wasm-pack` tool is used for building the WASM package.

## 4. Development Workflow

This project uses a specific workflow for development and testing. Please follow these guidelines:

### 4.1. Build and Test

**DO NOT** run `cargo build`, `cargo test`, or `cargo clippy` directly. Instead, use the provided build script, which handles all necessary steps, including formatting, linting, building, and testing.

```bash
./build.sh
```

After running the script, always review the output log to check for errors or warnings:

```bash
cat ./build.log.txt
```

### 4.2. Reference Implementation (the reference implementation)

When working with the reference JavaScript implementation in `ref/the reference implementation/`:

```bash
cd ref/the reference implementation

# Build the TypeScript code
npm run build

# Run all tests
npm test

# Run specific tests
npm run test-some -- <test-pattern>
```

## 5. Gemini-Specific Guidelines

### 5.1. Code Analysis
- Provide comprehensive code analysis and suggestions
- Focus on performance optimization opportunities
- Identify potential security vulnerabilities
- Suggest architectural improvements

### 5.2. Documentation
- Help maintain comprehensive documentation
- Create clear examples and usage patterns
- Explain complex algorithms and data structures
- Provide migration guides and tutorials

### 5.3. Testing
- Suggest comprehensive test cases
- Identify edge cases and boundary conditions
- Recommend property-based testing strategies
- Help with performance benchmarking

### 5.4. Best Practices
- Follow Rust idioms and conventions
- Prioritize safety and performance
- Maintain backward compatibility
- Consider cross-platform compatibility

## 6. Development Priorities

### 6.1. Current Focus
- JSON repair functionality integration
- Performance optimizations
- API stabilization
- Documentation improvements

### 6.2. Quality Assurance
- Comprehensive test coverage
- Performance regression testing
- Security audit considerations
- Cross-platform testing

### 6.3. Community
- Clear contribution guidelines
- Responsive issue handling
- Educational content creation
- Ecosystem integration