---
layout: page
title: Python API Design
permalink: /design/python-api/
parent: Design
nav_order: 1
---

# Python API Design for vexy_json

## Overview

This document outlines the design for Python bindings for the vexy_json library, drawing from PyO3 best practices and existing Python JSON parser APIs (json, orjson, ujson).

## Core Design Principles

1. **Idiomatic Python**: API should feel natural to Python developers
2. **Performance First**: Minimize Python/Rust round-trips
3. **Compatibility**: Similar to standard json library where possible
4. **Extensibility**: Support for streaming and advanced features

## API Structure

### Basic Functions (Similar to json module)

```python
import vexy_json

# Basic parsing - similar to json.loads()
def loads(s: str, *, 
          allow_comments: bool = True,
          allow_trailing_commas: bool = True,
          allow_unquoted_keys: bool = True,
          allow_single_quotes: bool = True,
          implicit_top_level: bool = True,
          newline_as_comma: bool = True,
          max_depth: int = 64) -> Any:
    """Parse a JSON string with forgiving features."""
    pass

# Formatting - similar to json.dumps()
def dumps(obj: Any, *, 
          indent: Optional[int] = None,
          ensure_ascii: bool = True) -> str:
    """Format a Python object as JSON string."""
    pass

# Validation
def is_valid(s: str) -> bool:
    """Check if string is valid JSON/Vexy JSON."""
    pass

# File operations
def load(fp: TextIO, **kwargs) -> Any:
    """Load JSON from file object."""
    pass

def dump(obj: Any, fp: TextIO, **kwargs) -> None:
    """Dump JSON to file object."""
    pass
```

### Options Class (For Advanced Configuration)

```python
class ParserOptions:
    """Configuration options for vexy_json parser."""
    
    def __init__(self, 
                 allow_comments: bool = True,
                 allow_trailing_commas: bool = True,
                 allow_unquoted_keys: bool = True,
                 allow_single_quotes: bool = True,
                 implicit_top_level: bool = True,
                 newline_as_comma: bool = True,
                 max_depth: int = 64):
        pass
    
    @classmethod
    def strict(cls) -> 'ParserOptions':
        """Create strict JSON parser options."""
        pass
    
    @classmethod
    def forgiving(cls) -> 'ParserOptions':
        """Create forgiving parser options (default)."""
        pass

def parse_with_options(s: str, options: ParserOptions) -> Any:
    """Parse with explicit options object."""
    pass
```

### Streaming Parser

```python
class StreamingParser:
    """Event-based streaming JSON parser."""
    
    def __init__(self, options: Optional[ParserOptions] = None):
        pass
    
    def feed(self, data: str) -> Iterator[StreamingEvent]:
        """Feed data and yield events."""
        pass
    
    def close(self) -> Iterator[StreamingEvent]:
        """Close parser and yield remaining events."""
        pass

class StreamingEvent:
    """Base class for streaming events."""
    pass

class StartObject(StreamingEvent):
    pass

class EndObject(StreamingEvent):
    pass

class StartArray(StreamingEvent):
    pass

class EndArray(StreamingEvent):
    pass

class ObjectKey(StreamingEvent):
    def __init__(self, key: str):
        self.key = key

class NullValue(StreamingEvent):
    pass

class BoolValue(StreamingEvent):
    def __init__(self, value: bool):
        self.value = value

class NumberValue(StreamingEvent):
    def __init__(self, value: str):
        self.value = value

class StringValue(StreamingEvent):
    def __init__(self, value: str):
        self.value = value

class EndOfInput(StreamingEvent):
    pass
```

### NDJSON Support

```python
class NdJsonParser:
    """Newline-delimited JSON parser."""
    
    def __init__(self, options: Optional[ParserOptions] = None):
        pass
    
    def parse_lines(self, lines: Iterable[str]) -> Iterator[Any]:
        """Parse NDJSON lines."""
        pass
    
    def parse_file(self, file_path: str) -> Iterator[Any]:
        """Parse NDJSON file."""
        pass

def parse_ndjson(s: str, **kwargs) -> List[Any]:
    """Parse NDJSON string to list of objects."""
    pass
```

### Error Handling

```python
class VexyJsonError(Exception):
    """Base exception for vexy_json errors."""
    pass

class ParseError(VexyJsonError):
    """JSON parsing error."""
    
    def __init__(self, message: str, line: int, column: int):
        self.message = message
        self.line = line
        self.column = column
        super().__init__(f"{message} at line {line}, column {column}")

class ValidationError(VexyJsonError):
    """JSON validation error."""
    pass
```

### Python-Specific Features

```python
# Dict/List builders for streaming
class StreamingValueBuilder:
    """Build Python objects from streaming events."""
    
    def __init__(self):
        pass
    
    def process_event(self, event: StreamingEvent) -> Optional[Any]:
        """Process event and return completed value if any."""
        pass

# Async support (future enhancement)
async def loads_async(s: str, **kwargs) -> Any:
    """Async version of loads."""
    pass

# Iterator support
def iter_objects(s: str, **kwargs) -> Iterator[Any]:
    """Iterate over top-level objects in string."""
    pass

def iter_arrays(s: str, **kwargs) -> Iterator[Any]:
    """Iterate over top-level arrays in string."""
    pass
```

## Key Design Decisions

### 1. Function Naming and Signatures

- **`loads()`** instead of `parse()` for consistency with `json` module
- **Keyword-only arguments** for options to prevent positional confusion
- **Boolean defaults** match vexy_json's forgiving nature

### 2. Error Handling

- **Custom exception hierarchy** with position information
- **Graceful error recovery** in streaming mode
- **Validation separate from parsing** for performance

### 3. Performance Optimizations

- **Bytes handling** like orjson for performance
- **Streaming events** minimize memory allocation
- **Bulk operations** in Rust rather than Python loops

### 4. Python Integration

- **File object support** for `load()`/`dump()`
- **Iterator protocol** for streaming
- **Type hints** for better IDE support
- **Docstrings** following Python conventions

### 5. API Extensions

- **`is_valid()`** for validation without parsing
- **Options classes** for complex configuration
- **NDJSON support** for line-oriented JSON
- **Streaming builder** for event-to-object conversion

## Implementation Strategy

1. **Phase 1**: Core `loads()`, `dumps()`, `is_valid()` functions
2. **Phase 2**: `ParserOptions` class and advanced parsing
3. **Phase 3**: Streaming parser with events
4. **Phase 4**: NDJSON support and file operations
5. **Phase 5**: Performance optimizations and async support

## Compatibility Notes

- **Standard library compatibility**: `loads()` and `dumps()` work as drop-in replacements
- **orjson inspiration**: Performance-focused design with bytes handling
- **ujson similarity**: Simple API with performance benefits
- **vexy_json extensions**: Forgiving features as the key differentiator

This design balances Python idioms with the performance benefits of Rust, providing a comprehensive JSON parsing solution that extends beyond standard JSON capabilities.