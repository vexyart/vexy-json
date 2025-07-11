---
layout: page
title: Python Bindings
permalink: /python/
nav_order: 6
---

# vexy_json - Forgiving JSON Parser for Python

A Python library for parsing "forgiving" JSON, which is JSON that includes features like:

- Comments (single-line `//` and multi-line `/* */`)
- Trailing commas in arrays and objects
- Unquoted object keys
- Single-quoted strings
- Implicit top-level objects and arrays
- Newlines as comma separators

This is a Python binding for the Rust [vexy_json](https://github.com/twardoch/vexy_json) library, which is a port of the JavaScript [jsonic](https://github.com/jsonicjs/jsonic) library.

## Installation

```bash
pip install vexy_json
```

## Quick Start

```python
import vexy_json

# Parse forgiving JSON
result = vexy_json.parse('''
{
    // This is a comment
    name: "Alice",
    age: 30,
    active: true,  // trailing comma is OK
}
''')

print(result)
# Output: {'name': 'Alice', 'age': 30, 'active': True}
```

## Features

### Basic Parsing

```python
import vexy_json

# Standard JSON
data = vexy_json.parse('{"key": "value"}')

# Forgiving features
data = vexy_json.parse('''
{
    // Comments are allowed
    unquoted_key: "value",
    'single_quotes': true,
    trailing_comma: "ok",
}
''')
```

### Custom Options

```python
import vexy_json

# Parse with specific options
data = vexy_json.parse_with_options(
    'key: value',
    allow_comments=True,
    allow_trailing_commas=True,
    allow_unquoted_keys=True,
    allow_single_quotes=True,
    implicit_top_level=True,
    newline_as_comma=True,
    max_depth=128
)
# Output: {'key': 'value'}
```

### Validation

```python
import vexy_json

# Check if JSON is valid
if vexy_json.is_valid('{"valid": true}'):
    print("Valid JSON!")

if not vexy_json.is_valid('invalid json'):
    print("Invalid JSON!")
```

### Serialization

```python
import vexy_json

data = {'name': 'Alice', 'age': 30}

# Compact output
json_str = vexy_json.dumps(data)
print(json_str)
# Output: {"name":"Alice","age":30}

# Pretty printed output
json_str = vexy_json.dumps(data, indent=2)
print(json_str)
# Output:
# {
#   "age": 30,
#   "name": "Alice"
# }
```

## API Reference

### Functions

#### `parse(input: str) -> Any`

Parse a JSON string with all forgiving features enabled.

**Parameters:**
- `input` (str): The JSON string to parse

**Returns:**
- The parsed JSON as a Python object (dict, list, str, int, float, bool, or None)

**Raises:**
- `ValueError`: If the input is not valid JSON

#### `parse_with_options(input: str, **options) -> Any`

Parse a JSON string with custom options.

**Parameters:**
- `input` (str): The JSON string to parse
- `allow_comments` (bool): Allow single-line and multi-line comments (default: True)
- `allow_trailing_commas` (bool): Allow trailing commas (default: True)
- `allow_unquoted_keys` (bool): Allow unquoted object keys (default: True)
- `allow_single_quotes` (bool): Allow single-quoted strings (default: True)
- `implicit_top_level` (bool): Allow implicit top-level objects/arrays (default: True)
- `newline_as_comma` (bool): Treat newlines as commas (default: True)
- `max_depth` (int): Maximum nesting depth (default: 128)

**Returns:**
- The parsed JSON as a Python object

**Raises:**
- `ValueError`: If the input is not valid JSON

#### `is_valid(input: str) -> bool`

Check if a string is valid JSON/Vexy JSON.

**Parameters:**
- `input` (str): The JSON string to validate

**Returns:**
- `bool`: True if valid, False otherwise

#### `dumps(obj: Any, indent: Optional[int] = None) -> str`

Serialize a Python object to a JSON string.

**Parameters:**
- `obj`: The Python object to serialize
- `indent` (int, optional): Number of spaces for indentation

**Returns:**
- `str`: The JSON string representation

**Raises:**
- `TypeError`: If the object cannot be serialized

## Comparison with Standard Library

Unlike Python's built-in `json` module, vexy_json is forgiving and accepts non-standard JSON:

```python
import json
import vexy_json

forgiving_json = '''
{
    // Comment
    name: "Alice",
    'age': 30,
}
'''

# This will raise an exception
try:
    json.loads(forgiving_json)
except json.JSONDecodeError as e:
    print(f"json module failed: {e}")

# This works fine
result = vexy_json.parse(forgiving_json)
print(f"vexy_json parsed: {result}")
```

## Performance

vexy_json is implemented in Rust and should be competitive with other JSON parsers for most use cases. The forgiving features add minimal overhead.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.