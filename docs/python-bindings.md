# Python Bindings

Vexy JSON provides comprehensive Python bindings that offer all the functionality of the Rust library with a familiar Python API. The bindings are designed to be both performant and easy to use.

## Installation

```bash
pip install vexy_json
```

## Basic Usage

### Parsing JSON

```python
import vexy_json

# Standard JSON parsing
data = vexy_json.loads('{"name": "John", "age": 30}')
print(data)  # {'name': 'John', 'age': 30}

# Parse with forgiving features
data = vexy_json.loads('''
{
    name: "John",  // Unquoted keys and comments
    age: 30,       // Trailing comma is okay
}
''')
```

### JSON Compatibility

The Vexy JSON Python bindings provide full compatibility with the standard `json` module:

```python
import vexy_json

# Drop-in replacement for json.loads()
data = vexy_json.loads('{"key": "value"}')

# All standard json functions are available
json_str = vexy_json.dumps(data)
json_str = vexy_json.dumps(data, indent=2)

# File operations
with open('data.json', 'r') as f:
    data = vexy_json.load(f)

with open('output.json', 'w') as f:
    vexy_json.dump(data, f, indent=2)
```

## Advanced Parsing Options

### Custom Parser Options

```python
import vexy_json

# Parse with custom options
data = vexy_json.parse_with_options(
    json_string,
    allow_comments=True,
    allow_trailing_commas=True,
    allow_unquoted_keys=True,
    allow_single_quotes=True,
    implicit_top_level=True,
    newline_as_comma=True,
    max_depth=128,
    enable_repair=True,
    max_repairs=100,
    fast_repair=False,
    report_repairs=True
)
```

### Validation

```python
import vexy_json

# Check if JSON is valid
is_valid = vexy_json.is_valid('{"valid": true}')
print(is_valid)  # True

is_valid = vexy_json.is_valid('invalid json')
print(is_valid)  # False
```

## Streaming Support

### Streaming Parser with Context Manager

```python
import vexy_json

# Parse large JSON files efficiently
with vexy_json.StreamingParser() as parser:
    with open('large_file.json', 'r') as f:
        for item in parser.parse_stream(f):
            process(item)
```

### NDJSON Support

```python
import vexy_json

# Parse NDJSON (newline-delimited JSON)
with vexy_json.StreamingParser() as parser:
    with open('data.ndjson', 'r') as f:
        for item in parser.parse_lines(f):
            process(item)
```

### Custom Streaming Options

```python
import vexy_json

# Create streaming parser with custom options
parser = vexy_json.StreamingParser(
    allow_comments=True,
    allow_trailing_commas=True,
    enable_repair=True
)

with parser as p:
    for item in p.parse_stream(file_handle):
        process(item)
```

## NumPy Integration

### Direct Array Parsing

```python
import vexy_json
import numpy as np

# Parse JSON array directly to NumPy array
arr = vexy_json.loads_numpy('[1, 2, 3, 4, 5]')
print(type(arr))  # <class 'numpy.ndarray'>
print(arr.dtype)  # int64

# Specify dtype
arr = vexy_json.loads_numpy('[1.1, 2.2, 3.3]', dtype='float32')
print(arr.dtype)  # float32
```

### Zero-Copy Optimization

```python
import vexy_json

# Optimized parsing for numeric data
arr = vexy_json.loads_numpy_zerocopy('[1, 2, 3, 4, 5]', dtype='int64')
# Uses zero-copy when possible for better performance
```

### Mixed Data Types

```python
import vexy_json

# Handle mixed arrays
arr = vexy_json.loads_numpy('[1, 2.5, 3, 4.7]')
print(arr.dtype)  # float64 (automatically promoted)

# Non-numeric data falls back to object arrays
arr = vexy_json.loads_numpy('["a", "b", "c"]')
print(arr.dtype)  # object
```

## Pandas Integration

### DataFrame Conversion

```python
import vexy_json
import pandas as pd

# Parse JSON to DataFrame
json_data = '[{"name": "John", "age": 30}, {"name": "Jane", "age": 25}]'
df = vexy_json.loads_dataframe(json_data)
print(type(df))  # <class 'pandas.core.frame.DataFrame'>

# Specify orientation
df = vexy_json.loads_dataframe(json_data, orient='records')
```

## Error Handling

### Parse Errors

```python
import vexy_json

try:
    data = vexy_json.loads('invalid json')
except ValueError as e:
    print(f"Parse error: {e}")
```

### Repair Functionality

```python
import vexy_json

# Automatic repair of common JSON issues
try:
    data = vexy_json.loads('{"key": "value",}')  # Trailing comma
    print(data)  # Successfully parsed
except ValueError as e:
    print(f"Even repair failed: {e}")
```

## Performance Optimization

### Choosing the Right Function

```python
import vexy_json

# For standard JSON, use loads() for compatibility
data = vexy_json.loads(standard_json)

# For forgiving JSON, use parse_with_options()
data = vexy_json.parse_with_options(
    forgiving_json,
    allow_comments=True,
    allow_trailing_commas=True
)

# For numerical data, use NumPy integration
arr = vexy_json.loads_numpy(json_array)

# For tabular data, use pandas integration
df = vexy_json.loads_dataframe(json_records)
```

### Memory Efficiency

```python
import vexy_json

# Streaming for large files
with vexy_json.StreamingParser() as parser:
    for item in parser.parse_stream(large_file):
        # Process items one at a time
        # Memory usage stays constant
        process(item)
```

## Type Hints

The Python bindings include comprehensive type hints:

```python
from typing import Any, Dict, List, Optional, Union
import vexy_json

def process_json(json_str: str) -> Dict[str, Any]:
    return vexy_json.loads(json_str)

def safe_parse(json_str: str) -> Optional[Dict[str, Any]]:
    try:
        return vexy_json.loads(json_str)
    except ValueError:
        return None
```

## Best Practices

### Error Handling

```python
import vexy_json

def safe_parse_json(json_str: str, default=None):
    """Safely parse JSON with fallback."""
    try:
        return vexy_json.loads(json_str)
    except ValueError as e:
        print(f"JSON parse error: {e}")
        return default

# Usage
data = safe_parse_json(user_input, default={})
```

### Performance Tips

1. **Use appropriate functions**: Choose `loads()` for standard JSON, `parse_with_options()` for forgiving JSON
2. **Streaming for large files**: Use `StreamingParser` for files that don't fit in memory
3. **NumPy integration**: Use `loads_numpy()` for numeric arrays
4. **Pandas integration**: Use `loads_dataframe()` for tabular data
5. **Validate when necessary**: Use `is_valid()` to check JSON before parsing

### Memory Management

```python
import vexy_json

# For large datasets, prefer streaming
def process_large_json(filename):
    with vexy_json.StreamingParser() as parser:
        with open(filename, 'r') as f:
            for item in parser.parse_stream(f):
                yield process_item(item)

# This keeps memory usage constant regardless of file size
```

## Integration Examples

### With Requests

```python
import requests
import vexy_json

response = requests.get('https://api.example.com/data')
data = vexy_json.loads(response.text)
```

### With FastAPI

```python
from fastapi import FastAPI
import vexy_json

app = FastAPI()

@app.post("/parse-json")
async def parse_json(content: str):
    try:
        data = vexy_json.loads(content)
        return {"success": True, "data": data}
    except ValueError as e:
        return {"success": False, "error": str(e)}
```

### With Django

```python
from django.http import JsonResponse
import vexy_json

def parse_json_view(request):
    try:
        data = vexy_json.loads(request.body)
        # Process data
        return JsonResponse({"success": True})
    except ValueError as e:
        return JsonResponse({"error": str(e)}, status=400)
```

## Migration from Standard JSON

### Drop-in Replacement

```python
# Before
import json
data = json.loads(json_string)

# After
import vexy_json
data = vexy_json.loads(json_string)  # Same interface, more forgiving
```

### Gradual Migration

```python
import json
import vexy_json

def parse_json_fallback(json_str):
    """Try standard JSON first, fall back to Vexy JSON."""
    try:
        return json.loads(json_str)
    except json.JSONDecodeError:
        return vexy_json.loads(json_str)  # More forgiving parsing
```

## Advanced Features

### Custom Serialization

```python
import vexy_json
from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int

# Convert to dict first, then serialize
person = Person("John", 30)
json_str = vexy_json.dumps(person.__dict__)
```

### Configuration Management

```python
import vexy_json

# Parse configuration files with comments
config_str = '''
{
    // Database configuration
    "database": {
        "host": "localhost",
        "port": 5432,  // Default PostgreSQL port
        "name": "myapp",
    },
    
    // API settings
    "api": {
        "timeout": 30,
        "retries": 3,
    }
}
'''

config = vexy_json.loads(config_str)
```

This comprehensive Python API provides all the power of Vexy JSON with the familiar interface Python developers expect.