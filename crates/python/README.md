# this_file: crates/python/README.md

# vexy_json Python Bindings

Python bindings for the vexy_json library - a forgiving JSON parser written in Rust.

## Installation

```bash
pip install vexy_json
```

## Usage

```python
import vexy_json

# Parse forgiving JSON
result = vexy_json.parse('{"key": "value", trailing: true,}')
print(result)  # {'key': 'value', 'trailing': True}

# Use NumPy integration
import numpy as np
arr = vexy_json.loads_numpy('[1, 2, 3, 4, 5]')
print(type(arr))  # <class 'numpy.ndarray'>
```

## Features

- Standard JSON parsing with forgiving extensions
- Comments (single-line and multi-line)
- Trailing commas in arrays and objects
- Unquoted object keys
- Single-quoted strings
- Implicit top-level objects and arrays
- NumPy integration for efficient array parsing
- Streaming parser for large files
- pandas DataFrame integration
- JSON repair functionality

For more information, see the [main vexy_json documentation](https://github.com/twardoch/vexy_json).