# this_file: crates/python/vexy_json.pyi

"""
Type stubs for vexy_json Python bindings.

This file provides type hints for the vexy_json Python module, which is implemented in Rust.
"""

from typing import Any, Dict, List, Union, Optional, IO, Iterator, ContextManager
from typing_extensions import Literal
import numpy as np
import pandas as pd

# JSON Value Types
JSONValue = Union[None, bool, int, float, str, List['JSONValue'], Dict[str, 'JSONValue']]

# File-like object type
FileObject = Union[IO[str], IO[bytes]]

def parse_json(input: str) -> JSONValue:
    """
    Parse a JSON string with default options (all forgiving features enabled).
    
    Args:
        input: The JSON string to parse
        
    Returns:
        The parsed JSON as a Python object (dict, list, str, int, float, bool, or None)
        
    Raises:
        ValueError: If the input is not valid JSON
        
    Example:
        >>> import vexy_json
        >>> result = vexy_json.parse('{"key": "value", trailing: true,}')
        >>> print(result)
        {'key': 'value', 'trailing': True}
    """
    ...

def parse_with_options_py(
    input: str,
    allow_comments: bool = True,
    allow_trailing_commas: bool = True,
    allow_unquoted_keys: bool = True,
    allow_single_quotes: bool = True,
    implicit_top_level: bool = True,
    newline_as_comma: bool = True,
    max_depth: int = 128,
    enable_repair: bool = True,
    max_repairs: int = 100,
    fast_repair: bool = False,
    report_repairs: bool = True,
) -> JSONValue:
    """
    Parse a JSON string with custom options.
    
    Args:
        input: The JSON string to parse
        allow_comments: Allow single-line and multi-line comments. Defaults to True.
        allow_trailing_commas: Allow trailing commas in arrays and objects. Defaults to True.
        allow_unquoted_keys: Allow unquoted object keys. Defaults to True.
        allow_single_quotes: Allow single-quoted strings. Defaults to True.
        implicit_top_level: Allow implicit top-level objects/arrays. Defaults to True.
        newline_as_comma: Treat newlines as commas. Defaults to True.
        max_depth: Maximum nesting depth. Defaults to 128.
        enable_repair: Enable JSON repair functionality. Defaults to True.
        max_repairs: Maximum number of repairs to attempt. Defaults to 100.
        fast_repair: Prefer speed over repair quality. Defaults to False.
        report_repairs: Report all repairs made. Defaults to True.
        
    Returns:
        The parsed JSON as a Python object
        
    Raises:
        ValueError: If the input is not valid JSON
        
    Example:
        >>> import vexy_json
        >>> result = vexy_json.parse_with_options('key: value', implicit_top_level=True)
        >>> print(result)
        {'key': 'value'}
    """
    ...

def is_valid(input: str) -> bool:
    """
    Check if a string is valid JSON/Vexy JSON.
    
    Args:
        input: The JSON string to validate
        
    Returns:
        True if the input is valid, False otherwise
        
    Example:
        >>> import vexy_json
        >>> vexy_json.is_valid('{"valid": true}')
        True
        >>> vexy_json.is_valid('invalid json')
        False
    """
    ...

def dumps(obj: Any, indent: Optional[int] = None) -> str:
    """
    Dumps a Python object to a JSON string.
    
    Args:
        obj: The Python object to serialize
        indent: Number of spaces for indentation. If None, output is compact.
        
    Returns:
        The JSON string representation
        
    Raises:
        TypeError: If the object cannot be serialized to JSON
        
    Example:
        >>> import vexy_json
        >>> data = {'key': 'value', 'number': 42}
        >>> vexy_json.dumps(data)
        '{"key":"value","number":42}'
        >>> vexy_json.dumps(data, indent=2)
        '{\n  "key": "value",\n  "number": 42\n}'
    """
    ...

def load(fp: FileObject, **kwargs: Any) -> JSONValue:
    """
    Load JSON from a file-like object.
    
    Args:
        fp: A file-like object supporting .read()
        **kwargs: Additional arguments passed to parse_with_options
        
    Returns:
        The parsed JSON as a Python object
        
    Raises:
        ValueError: If the content is not valid JSON
        
    Example:
        >>> import vexy_json
        >>> with open('data.json', 'r') as f:
        ...     result = vexy_json.load(f)
    """
    ...

def dump(obj: Any, fp: FileObject, indent: Optional[int] = None) -> None:
    """
    Dump JSON to a file-like object.
    
    Args:
        obj: The Python object to serialize
        fp: A file-like object supporting .write()
        indent: Number of spaces for indentation
        
    Raises:
        TypeError: If the object cannot be serialized
        
    Example:
        >>> import vexy_json
        >>> data = {'key': 'value'}
        >>> with open('output.json', 'w') as f:
        ...     vexy_json.dump(data, f, indent=2)
    """
    ...

def loads_numpy(input: str, dtype: Optional[str] = None) -> np.ndarray:
    """
    Parse JSON array directly to NumPy array (if NumPy is available).
    
    Args:
        input: The JSON array string to parse
        dtype: NumPy dtype for the array. Defaults to auto-detection.
        
    Returns:
        The parsed array as a NumPy array
        
    Raises:
        ValueError: If the input is not a valid JSON array
        ImportError: If NumPy is not available
        
    Example:
        >>> import vexy_json
        >>> arr = vexy_json.loads_numpy('[1, 2, 3, 4, 5]')
        >>> print(type(arr))
        <class 'numpy.ndarray'>
    """
    ...

def loads_numpy_zerocopy(input: str, dtype: Optional[str] = None) -> np.ndarray:
    """
    Parse JSON array with zero-copy optimization for numeric data.
    
    Args:
        input: The JSON array string to parse
        dtype: Target dtype for the array
        
    Returns:
        The parsed array with zero-copy optimization when possible
        
    Example:
        >>> import vexy_json
        >>> arr = vexy_json.loads_numpy_zerocopy('[1.0, 2.0, 3.0]', dtype='float64')
    """
    ...

def loads_dataframe(input: str, orient: str = "records") -> pd.DataFrame:
    """
    Convert JSON object to pandas DataFrame (if pandas is available).
    
    Args:
        input: The JSON string to parse (should be an object or array of objects)
        orient: DataFrame orientation. Defaults to 'records'.
        
    Returns:
        The parsed JSON as a DataFrame
        
    Example:
        >>> import vexy_json
        >>> df = vexy_json.loads_dataframe('[{"a": 1, "b": 2}, {"a": 3, "b": 4}]')
        >>> print(type(df))
        <class 'pandas.core.frame.DataFrame'>
    """
    ...

class StreamingParser:
    """
    Streaming JSON parser with context manager support.
    
    This class provides a streaming JSON parser that can be used with Python's
    context manager protocol (`with` statement) for efficient processing of large
    JSON files or streams.
    
    Example:
        >>> import vexy_json
        >>> with vexy_json.StreamingParser() as parser:
        ...     for item in parser.parse_stream(file_handle):
        ...         print(item)
    """
    
    def __init__(
        self,
        allow_comments: bool = True,
        allow_trailing_commas: bool = True,
        allow_unquoted_keys: bool = True,
        allow_single_quotes: bool = True,
        implicit_top_level: bool = True,
        newline_as_comma: bool = True,
        max_depth: int = 128,
        enable_repair: bool = True,
        max_repairs: int = 100,
        fast_repair: bool = False,
        report_repairs: bool = True,
    ) -> None:
        """
        Create a new streaming parser.
        
        Args:
            allow_comments: Allow single-line and multi-line comments
            allow_trailing_commas: Allow trailing commas in arrays and objects
            allow_unquoted_keys: Allow unquoted object keys
            allow_single_quotes: Allow single-quoted strings
            implicit_top_level: Allow implicit top-level objects/arrays
            newline_as_comma: Treat newlines as commas
            max_depth: Maximum nesting depth
            enable_repair: Enable JSON repair functionality
            max_repairs: Maximum number of repairs to attempt
            fast_repair: Prefer speed over repair quality
            report_repairs: Report all repairs made
        """
        ...
    
    def __enter__(self) -> 'StreamingParser':
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[type] = None,
        exc_value: Optional[BaseException] = None,
        traceback: Optional[Any] = None,
    ) -> bool:
        """Context manager exit."""
        ...
    
    def parse_stream(self, fp: FileObject) -> Iterator[JSONValue]:
        """
        Parse a stream of JSON objects.
        
        Args:
            fp: A file-like object supporting .read() or .readline()
            
        Returns:
            Iterator of parsed JSON objects
            
        Example:
            >>> with vexy_json.StreamingParser() as parser:
            ...     for item in parser.parse_stream(file_handle):
            ...         process(item)
        """
        ...
    
    def parse_lines(self, fp: FileObject) -> Iterator[JSONValue]:
        """
        Parse lines from a file as individual JSON objects (NDJSON format).
        
        Args:
            fp: A file-like object supporting .readline()
            
        Returns:
            Iterator of parsed JSON objects
            
        Example:
            >>> with vexy_json.StreamingParser() as parser:
            ...     for item in parser.parse_lines(file_handle):
            ...         process(item)
        """
        ...

# Convenience aliases
parse = parse_json
parse_with_options = parse_with_options_py
loads = parse_json

# Module metadata
__version__: str
__author__: str
__description__: str