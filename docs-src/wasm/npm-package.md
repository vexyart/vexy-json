---
nav_title: NPM Package
nav_order: 2
---

# @twardoch/vexy_json-wasm

WebAssembly bindings for [vexy_json](https://github.com/vexyart/vexy-json), a forgiving JSON parser that's a Rust port of [the reference implementation](https://github.com/vexyart/vexy-json/tree/main/ref/the%20reference%20implementation).

## Installation

```bash
npm install @twardoch/vexy_json-wasm
```

## Usage

```javascript
import init, { parse_js, parse_with_options_js, is_valid, format } from '@twardoch/vexy_json-wasm';

// Initialize the WASM module
await init();

// Parse forgiving JSON
const result = parse_js('{ key: "value", trailing: true, }');
console.log(result); // {"key":"value","trailing":true}

// Parse with custom options
const customResult = parse_with_options_js(
  'key: value\nkey2: value2',
  true,  // allow_comments
  true,  // allow_trailing_commas
  true,  // allow_unquoted_keys
  true,  // allow_single_quotes
  true,  // implicit_top_level
  true   // newline_as_comma
);
console.log(customResult); // {"key":"value","key2":"value2"}

// Check if input is valid
console.log(is_valid('{"valid": true}')); // true
console.log(is_valid('invalid json')); // false

// Format JSON (parse and re-stringify)
const formatted = format('{ compact:true,data:[1,2,3] }');
console.log(formatted); // {"compact":true,"data":[1,2,3]}
```

## Features

vexy_json supports all standard JSON features plus:

- **Comments**: Single-line (`//`) and multi-line (`/* */`)
- **Trailing commas**: In objects and arrays
- **Unquoted keys**: Object keys without quotes
- **Single quotes**: For string values
- **Implicit top-level**: `key: value` → `{"key": "value"}`
- **Newlines as commas**: Line breaks can separate values

## API

### `parse_js(input: string): string`
Parse a JSON/Vexy JSON string with default options (all forgiving features enabled).

### `parse_with_options_js(input: string, ...options): string`
Parse with custom options:
- `allow_comments`: Enable single-line and multi-line comments
- `allow_trailing_commas`: Allow trailing commas in arrays and objects
- `allow_unquoted_keys`: Allow unquoted object keys
- `allow_single_quotes`: Allow single-quoted strings
- `implicit_top_level`: Convert top-level non-arrays/objects to valid JSON
- `newline_as_comma`: Treat newlines as commas

### `is_valid(input: string): boolean`
Check if the input is valid JSON/Vexy JSON.

### `format(input: string): string`
Parse and re-stringify JSON/Vexy JSON (currently outputs compact JSON).

## License

MIT OR Apache-2.0