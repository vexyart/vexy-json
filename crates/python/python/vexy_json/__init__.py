"""
vexy_json - A forgiving JSON parser for Python.

This module provides Python bindings for the vexy_json library, which is a Rust port
of the JavaScript jsonic library. It allows parsing of "forgiving" JSON that includes
features like comments, trailing commas, unquoted keys, and more.

Features:
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
"""

from ._vexy_json import (
    parse_json as parse,
    parse_with_options_py as parse_with_options,
    is_valid,
    dumps,
    load,
    dump,
    loads_numpy,
    loads_numpy_zerocopy,
    loads_dataframe,
    StreamingParser,
    __version__,
    __author__,
    __description__,
)

# Re-export for convenience and standard library compatibility
loads = parse

__all__ = [
    "parse",
    "loads",
    "parse_with_options",
    "is_valid",
    "dumps",
    "load",
    "dump",
    "loads_numpy",
    "loads_numpy_zerocopy",
    "loads_dataframe",
    "StreamingParser",
    "__version__",
    "__author__",
    "__description__",
]
