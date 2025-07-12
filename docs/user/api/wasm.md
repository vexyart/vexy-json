---
layout: default
title: WebAssembly API Reference
nav_order: 8
---


# WebAssembly (WASM) API Reference

`vexy_json` provides WebAssembly bindings for use in JavaScript environments (browsers, Node.js). The WASM module exposes parsing functions that mirror the Rust API, including forgiving features and strict mode.

## Usage

```js
import init, { parse_json, parse_json_with_options } from './pkg/vexy_json_wasm.js';

await init();
const result = parse_json_with_options('{a:1}', { allow_comments: true });
console.log(result); // { a: 1 }
```

## API

- `parse_json(input: string): any` — Parse with default forgiving options
- `parse_json_with_options(input: string, options: object): any` — Parse with custom options
- `get_parser_options(): object` — Get default options

## Options

All forgiving features can be toggled via options (see [features.md](features.md)).

## Recent Fixes

- As of v1.2.4, parsed objects are returned as plain JavaScript objects, not Maps. See [Troubleshooting](troubleshooting.md).

> **📝 Note**: Version 1.2.4 includes a critical fix for object conversion. Previous versions incorrectly returned JavaScript Maps instead of plain objects for parsed JSON. If you're experiencing issues where `{a:1}` returns `{}`, please upgrade to version 1.2.4 or later. See [Troubleshooting](troubleshooting.md) for details.

To use the WASM bindings, you need to enable the `wasm` feature in your `Cargo.toml`:

```toml
[dependencies]
vexy_json = { version = "2.0.0", features = ["wasm"] }
```

After building your Rust project with the `wasm` feature (e.g., using `wasm-pack`), you can import the generated JavaScript module.

## Available JavaScript Functions

The following functions are exposed to JavaScript:

### `init()`

```javascript
init(): Promise<void>
```

Initializes the WebAssembly module. This function should be called once when the WASM module is loaded to set up proper panic handling for better debugging experience. It returns a Promise that resolves when the WASM module is ready.

**Example:**

```javascript
import init from './pkg/vexy_json_wasm.js';

async function run() {
  await init();
  console.log("vexy_json WASM module loaded.");
  // Now you can use other vexy_json functions
}
run();
```

### `parse_json(input: string)`

```javascript
parse_json(input: string): any
```

Parses a JSON-like string into a JavaScript value using default parser options. This is the main parsing function for WebAssembly usage. It accepts relaxed JSON syntax including comments, unquoted keys, trailing commas, and more.

- `input`: The JSON string to parse (supports forgiving syntax).
- Returns: The successfully parsed value converted to a native JavaScript type (object, array, string, number, boolean, null).
- Throws: A `ParseError` object if a parsing error occurs.

**Example:**

```javascript
import { parse_json } from './pkg/vexy_json_wasm.js';

try {
  const result = parse_json(`{
    // This is a comment
    key: 'single quotes work',
    trailing: 'commas allowed',
  }`);
  console.log(result);
  // Output: { key: 'single quotes work', trailing: 'commas allowed' }
} catch (e) {
  console.error(`Parse Error: ${e.message} at position ${e.position}`);
}
```

### `parse_json_with_options(input: string, options: object)`

```javascript
parse_json_with_options(input: string, options: object): any
```

Parses a JSON string with custom parser options. This function allows fine-grained control over which forgiving features to enable.

- `input`: The JSON string to parse.
- `options`: A JavaScript object with parser configuration properties (see `get_parser_options()` for available properties).
- Returns: The successfully parsed value.
- Throws: A `ParseError` object if a parsing error occurs.

**Example:**

```javascript
import { parse_json_with_options } from './pkg/vexy_json_wasm.js';

// Strict JSON mode
const strictOptions = {
  allowComments: false,
  allowTrailingCommas: false,
  allowUnquotedKeys: false,
  allowSingleQuotes: false,
  implicitTopLevel: false,
  newlineAsComma: false
};

try {
  const result = parse_json_with_options('{"key": "value"}', strictOptions);
  console.log(result);
} catch (e) {
  console.error(`Strict Parse Error: ${e.message}`);
}

// Enable only specific features
const customOptions = {
  allowUnquotedKeys: true,
  implicitTopLevel: true
};

try {
  const result = parse_json_with_options('key: "value"', customOptions);
  console.log(result);
} catch (e) {
  console.error(`Custom Parse Error: ${e.message}`);
}
```

### `validate_json(input: string)`

```javascript
validate_json(input: string): boolean
```

Validates if a JSON string can be successfully parsed. This is a lightweight function that checks syntax validity without constructing the full value tree. Useful for input validation.

- `input`: The JSON string to validate.
- Returns: `true` if the input is valid and can be parsed, `false` otherwise.

**Example:**

```javascript
import { validate_json } from './pkg/vexy_json_wasm.js';

console.log(validate_json('{"key": "value"}')); // true
console.log(validate_json('{key: "value"}'));   // true (unquoted keys allowed by default)
console.log(validate_json('{invalid'));         // false
```

### `get_parser_options()`

```javascript
get_parser_options(): object
```

Returns the current default configuration for the parser as a JavaScript object. This object can be modified and passed to `parse_json_with_options`.

- Returns: A JavaScript object with all available parser options and their default values. The keys are camelCase (e.g., `allowComments`).

**Example:**

```javascript
import { get_parser_options, parse_json_with_options } from './pkg/vexy_json_wasm.js';

const defaultOptions = get_parser_options();
console.log(defaultOptions.allowComments); // true

// Modify specific options
const modifiedOptions = { ...defaultOptions, allowComments: false };
const result = parse_json_with_options('// comment\n{"a":1}', modifiedOptions); // Will throw error if comments are disabled
```

### `stringify_value(value: any)`

```javascript
stringify_value(value: any): string
```

Converts a JavaScript value (typically obtained from a `parse_json` operation) back to a compact JSON string representation.

- `value`: The JavaScript value to stringify.
- Returns: A compact JSON string representation.
- Throws: An error if the value cannot be serialized.

**Example:**

```javascript
import { parse_json, stringify_value } from './pkg/vexy_json_wasm.js';

const parsed = parse_json('{key: "value", num: 42}');
const jsonString = stringify_value(parsed); // '{"key":"value","num":42}'
console.log(jsonString);
```

### `get_version_info()`

```javascript
get_version_info(): object
```

Returns version and build information for the `vexy_json` library. Useful for debugging and compatibility checking.

- Returns: A JavaScript object with properties like `version`, `description`, `authors`, `homepage`, `repository`, and `license`.

**Example:**

```javascript
import { get_version_info } from './pkg/vexy_json_wasm.js';

const info = get_version_info();
console.log(`vexy_json v${info.version} - ${info.description}`);
```

## `ParseError` Class

When a parsing error occurs in `parse_json` or `parse_json_with_options`, a `ParseError` object is thrown. This class provides structured error information.

```javascript
class ParseError {
  readonly message: string;
  readonly position: number;
}
```

- `message`: A string describing what went wrong.
- `position`: The character position in the input string where the error occurred (0-indexed).

**Example (Error Handling):**

```javascript
import { parse_json } from './pkg/vexy_json_wasm.js';

try {
  parse_json('{invalid json');
} catch (e) {
  if (e instanceof Error && e.message.startsWith('Parse Error:')) { // Basic check for ParseError
    console.error(`Caught vexy_json ParseError: ${e.message} at position ${e.position}`);
  } else {
    console.error(`Caught unexpected error: ${e}`);
  }
}
```