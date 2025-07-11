# vexy_json Python Bindings

Fast, forgiving JSON parser for Python - a port of the JavaScript library jsonic.

## Features

- 🚀 **Fast**: Written in Rust for maximum performance
- 🤝 **Forgiving**: Handles common JSON mistakes and non-standard syntax
- 💬 **Comments**: Supports `//` and `/* */` style comments
- 🔧 **Flexible**: Unquoted keys, trailing commas, single quotes, and more
- 🛠️ **Repairable**: Automatically fixes common JSON errors
- 🐍 **Pythonic**: Familiar API similar to the standard `json` module

## Installation

```bash
pip install vexy_json
```

### Building from source

```bash
cd bindings/python
pip install maturin
maturin develop
```

## Quick Start

```python
import vexy_json

# Parse forgiving JSON
data = vexy_json.parse('''
{
    // Comments are allowed
    name: "John",        // Unquoted keys
    'age': 30,          // Single quotes
    "city": "New York",
    hobbies: [
        "reading",
        "coding",       // Trailing commas
    ],
}
''')

print(data)
# {'name': 'John', 'age': 30, 'city': 'New York', 'hobbies': ['reading', 'coding']}
```

## API Reference

### Functions

#### `parse(input: str) -> Any`
Parse a JSON string with default forgiving options.

```python
data = vexy_json.parse('{"key": "value"}')
```

#### `parse_with_options(input: str, options: Options) -> Any`
Parse a JSON string with custom options.

```python
opts = vexy_json.Options(allow_comments=False)
data = vexy_json.parse_with_options(json_str, opts)
```

#### `dumps(obj: Any, indent: int = None, sort_keys: bool = False) -> str`
Serialize a Python object to JSON string.

```python
json_str = vexy_json.dumps({"key": "value"}, indent=2)
```

#### `load(filename: str, options: Options = None) -> Any`
Load JSON from a file.

```python
data = vexy_json.load("config.json")
```

#### `dump(obj: Any, filename: str, indent: int = None, sort_keys: bool = False)`
Save Python object as JSON to a file.

```python
vexy_json.dump(data, "output.json", indent=2)
```

### Classes

#### `Options`
Parser configuration options.

```python
opts = vexy_json.Options(
    allow_comments=True,         # Allow // and /* */ comments
    allow_trailing_commas=True,  # Allow trailing commas
    allow_unquoted_keys=True,    # Allow unquoted object keys
    allow_single_quotes=True,    # Allow single-quoted strings
    implicit_top_level=True,     # Allow implicit top-level objects
    newline_as_comma=True,       # Treat newlines as commas
    max_depth=128,              # Maximum nesting depth
    enable_repair=True,         # Enable automatic error repair
    max_repairs=100,            # Maximum repair attempts
    fast_repair=False,          # Use fast repair mode
    report_repairs=False        # Include repair info in results
)
```

Pre-configured options:
- `Options.default()` - All forgiving features enabled (default)
- `Options.strict()` - Standard JSON only

#### `Parser`
Reusable parser instance for better performance when parsing multiple documents.

```python
parser = vexy_json.Parser(options)
data = parser.parse(json_str)
```

## Examples

### Configuration Files

vexy_json is perfect for configuration files that need to be human-friendly:

```python
config = vexy_json.parse('''
{
    // Server configuration
    server: {
        host: 'localhost',
        port: 8080,
        workers: 4,
    },
    
    // Database settings
    database: {
        engine: 'postgresql',
        host: 'db.example.com',
        credentials: {
            user: 'app_user',
            password_env: 'DB_PASSWORD',  // Read from environment
        }
    },
    
    // Feature flags
    features: {
        new_ui: true
        analytics: false
        beta: ['feature1', 'feature2']
    }
}
''')
```

### Error Recovery

vexy_json can automatically fix common JSON errors:

```python
# Missing commas
fixed = vexy_json.parse('{"a": 1 "b": 2}')  # {'a': 1, 'b': 2}

# Unclosed strings
fixed = vexy_json.parse('{"name": "John')   # {'name': 'John'}

# Trailing commas
fixed = vexy_json.parse('[1, 2, 3,]')       # [1, 2, 3]
```

### Strict Mode

For standard JSON compliance:

```python
strict_parser = vexy_json.Parser(vexy_json.Options.strict())

# This will raise an error
try:
    strict_parser.parse('{unquoted: true}')
except ValueError as e:
    print(f"Invalid JSON: {e}")
```

## Performance

vexy_json is built with Rust and is designed to be fast:

- Written in Rust for native performance
- Efficient memory usage
- SIMD optimizations where available
- Minimal Python overhead

## Compatibility

- Python 3.8+
- Works on Linux, macOS, and Windows
- Thread-safe

## License

This project is licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.