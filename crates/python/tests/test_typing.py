# this_file: crates/python/tests/test_typing.py

"""
Tests for type hints and advanced Python features.
"""

import pytest
import io
import sys
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import numpy as np
    import pandas as pd

# Test basic functionality without importing numpy/pandas
def test_basic_functionality():
    """Test basic parsing functionality with type hints."""
    import vexy_json
    
    # Test basic parsing
    result = vexy_json.parse('{"key": "value"}')
    assert result == {"key": "value"}
    
    # Test loads alias
    result = vexy_json.loads('{"key": "value"}')
    assert result == {"key": "value"}
    
    # Test validation
    assert vexy_json.is_valid('{"valid": true}') is True
    assert vexy_json.is_valid('invalid') is False
    
    # Test dumps
    data = {"key": "value", "number": 42}
    json_str = vexy_json.dumps(data)
    assert "key" in json_str
    assert "value" in json_str
    
    # Test pretty printing
    pretty_str = vexy_json.dumps(data, indent=2)
    assert "\n" in pretty_str
    assert "  " in pretty_str


def test_file_operations():
    """Test file I/O operations with type hints."""
    import vexy_json
    
    # Test with StringIO
    json_data = '{"test": "data", "number": 123}'
    
    # Test load
    fp = io.StringIO(json_data)
    result = vexy_json.load(fp)
    assert result == {"test": "data", "number": 123}
    
    # Test dump
    output = io.StringIO()
    vexy_json.dump({"key": "value"}, output)
    output.seek(0)
    dumped = output.read()
    assert "key" in dumped
    assert "value" in dumped


def test_streaming_parser():
    """Test streaming parser with type hints."""
    import vexy_json
    
    # Test streaming parser creation
    parser = vexy_json.StreamingParser()
    assert parser is not None
    
    # Test context manager
    with vexy_json.StreamingParser() as parser:
        assert parser is not None
    
    # Test with file-like object
    json_lines = '{"line": 1}\n{"line": 2}\n{"line": 3}\n'
    fp = io.StringIO(json_lines)
    
    with vexy_json.StreamingParser() as parser:
        results = list(parser.parse_lines(fp))
        assert len(results) == 3
        assert results[0] == {"line": 1}
        assert results[1] == {"line": 2}
        assert results[2] == {"line": 3}


def test_parse_with_options():
    """Test parse_with_options with all parameter types."""
    import vexy_json
    
    # Test with all parameters
    result = vexy_json.parse_with_options(
        'key: "value", // comment\n',
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
        report_repairs=True,
    )
    assert result == {"key": "value"}


@pytest.mark.skipif(sys.version_info < (3, 9), reason="requires Python 3.9+")
def test_numpy_integration():
    """Test NumPy integration if available."""
    import vexy_json
    
    try:
        import numpy as np
        
        # Test loads_numpy
        arr = vexy_json.loads_numpy('[1, 2, 3, 4, 5]')
        assert isinstance(arr, np.ndarray)
        assert arr.tolist() == [1, 2, 3, 4, 5]
        
        # Test loads_numpy_zerocopy
        arr = vexy_json.loads_numpy_zerocopy('[1.0, 2.0, 3.0]')
        assert isinstance(arr, np.ndarray)
        assert arr.tolist() == [1.0, 2.0, 3.0]
        
        # Test with dtype specification
        arr = vexy_json.loads_numpy('[1, 2, 3]', dtype='float32')
        assert isinstance(arr, np.ndarray)
        assert arr.dtype == np.float32
        
    except ImportError:
        pytest.skip("NumPy not available")


@pytest.mark.skipif(sys.version_info < (3, 9), reason="requires Python 3.9+")
def test_pandas_integration():
    """Test pandas integration if available."""
    import vexy_json
    
    try:
        import pandas as pd
        
        # Test loads_dataframe
        df = vexy_json.loads_dataframe('[{"a": 1, "b": 2}, {"a": 3, "b": 4}]')
        assert isinstance(df, pd.DataFrame)
        assert df.shape == (2, 2)
        assert df.columns.tolist() == ["a", "b"]
        assert df.iloc[0]["a"] == 1
        assert df.iloc[1]["b"] == 4
        
    except ImportError:
        pytest.skip("pandas not available")


def test_error_handling():
    """Test error handling with proper exception types."""
    import vexy_json
    
    # Test ValueError for invalid JSON
    with pytest.raises(ValueError, match="Parse error"):
        vexy_json.parse('invalid json')
    
    # Test TypeError for non-serializable objects
    with pytest.raises(TypeError):
        vexy_json.dumps(object())


def test_module_metadata():
    """Test module metadata and version information."""
    import vexy_json
    
    # Test version information
    assert hasattr(vexy_json, '__version__')
    assert isinstance(vexy_json.__version__, str)
    
    # Test author information
    assert hasattr(vexy_json, '__author__')
    assert isinstance(vexy_json.__author__, str)
    
    # Test description
    assert hasattr(vexy_json, '__description__')
    assert isinstance(vexy_json.__description__, str)


def test_forgiving_features():
    """Test all forgiving JSON features."""
    import vexy_json
    
    # Test comments
    result = vexy_json.parse('{"key": "value" /* comment */}')
    assert result == {"key": "value"}
    
    # Test trailing commas
    result = vexy_json.parse('{"key": "value",}')
    assert result == {"key": "value"}
    
    # Test unquoted keys
    result = vexy_json.parse('{key: "value"}')
    assert result == {"key": "value"}
    
    # Test single quotes
    result = vexy_json.parse("{'key': 'value'}")
    assert result == {"key": "value"}
    
    # Test implicit top-level object
    result = vexy_json.parse('key: "value"')
    assert result == {"key": "value"}
    
    # Test implicit top-level array
    result = vexy_json.parse('"a", "b", "c"')
    assert result == ["a", "b", "c"]