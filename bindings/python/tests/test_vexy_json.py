"""Tests for vexy_json Python bindings"""

import pytest
import vexy_json
import json
import tempfile
import os


class TestBasicParsing:
    """Test basic JSON parsing functionality"""

    def test_parse_simple_object(self):
        result = vexy_json.parse('{"name": "John", "age": 30}')
        assert result == {"name": "John", "age": 30}

    def test_parse_simple_array(self):
        result = vexy_json.parse('[1, 2, 3, "hello"]')
        assert result == [1, 2, 3, "hello"]

    def test_parse_nested_structure(self):
        input_json = """
        {
            "users": [
                {"name": "Alice", "age": 25},
                {"name": "Bob", "age": 30}
            ],
            "total": 2
        }
        """
        result = vexy_json.parse(input_json)
        expected = {
            "users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}],
            "total": 2,
        }
        assert result == expected

    def test_parse_primitives(self):
        assert vexy_json.parse("true") is True
        assert vexy_json.parse("false") is False
        assert vexy_json.parse("null") is None
        assert vexy_json.parse("42") == 42
        assert vexy_json.parse("3.14") == 3.14
        assert vexy_json.parse('"hello"') == "hello"


class TestForgivingFeatures:
    """Test forgiving JSON parsing features"""

    def test_comments(self):
        input_json = """
        {
            // Line comment
            "name": "Test",
            /* Block comment */
            "value": 42
        }
        """
        result = vexy_json.parse(input_json)
        assert result == {"name": "Test", "value": 42}

    def test_trailing_commas(self):
        result = vexy_json.parse('{"a": 1, "b": 2,}')
        assert result == {"a": 1, "b": 2}

        result = vexy_json.parse("[1, 2, 3,]")
        assert result == [1, 2, 3]

    def test_unquoted_keys(self):
        result = vexy_json.parse('{name: "John", age: 30}')
        assert result == {"name": "John", "age": 30}

    def test_single_quotes(self):
        result = vexy_json.parse("{'name': 'John', 'city': 'NYC'}")
        assert result == {"name": "John", "city": "NYC"}

    def test_implicit_object(self):
        result = vexy_json.parse('name: "John", age: 30')
        assert result == {"name": "John", "age": 30}

    def test_newline_as_comma(self):
        input_json = """
        {
            "a": 1
            "b": 2
            "c": 3
        }
        """
        result = vexy_json.parse(input_json)
        assert result == {"a": 1, "b": 2, "c": 3}

    def test_mixed_forgiving_features(self):
        input_json = """
        {
            // Configuration file
            server: 'localhost',
            port: 8080,
            features: {
                auth: true
                cache: false,
            },
            /* Database settings */
            database: {
                host: "db.example.com",
                name: 'myapp'
            }
        }
        """
        result = vexy_json.parse(input_json)
        expected = {
            "server": "localhost",
            "port": 8080,
            "features": {"auth": True, "cache": False},
            "database": {"host": "db.example.com", "name": "myapp"},
        }
        assert result == expected


class TestOptions:
    """Test parser options"""

    def test_default_options(self):
        opts = vexy_json.Options.default()
        assert opts.allow_comments is True
        assert opts.allow_trailing_commas is True
        assert opts.allow_unquoted_keys is True

    def test_strict_options(self):
        opts = vexy_json.Options.strict()
        assert opts.allow_comments is False
        assert opts.allow_trailing_commas is False
        assert opts.allow_unquoted_keys is False

    def test_custom_options(self):
        opts = vexy_json.Options(
            allow_comments=False, allow_trailing_commas=True, max_depth=50
        )
        assert opts.allow_comments is False
        assert opts.allow_trailing_commas is True
        assert opts.max_depth == 50

    def test_parse_with_strict_options(self):
        opts = vexy_json.Options.strict()

        # Valid JSON should work
        result = vexy_json.parse_with_options('{"valid": true}', opts)
        assert result == {"valid": True}

        # Invalid JSON should fail with strict options
        with pytest.raises(ValueError):
            vexy_json.parse_with_options("{unquoted: true}", opts)


class TestParser:
    """Test Parser class"""

    def test_parser_creation(self):
        parser = vexy_json.Parser()
        assert parser is not None

    def test_parser_with_options(self):
        opts = vexy_json.Options(allow_comments=False)
        parser = vexy_json.Parser(opts)

        # Should parse valid JSON
        result = parser.parse('{"valid": true}')
        assert result == {"valid": True}

    def test_parser_reuse(self):
        parser = vexy_json.Parser()

        # Parse multiple inputs with same parser
        results = []
        inputs = ['{"a": 1}', "[1, 2, 3]", '"hello"', "true"]

        for input_str in inputs:
            results.append(parser.parse(input_str))

        assert results == [{"a": 1}, [1, 2, 3], "hello", True]


class TestFileOperations:
    """Test file load/dump operations"""

    def test_load_file(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            f.write('{"test": true, "value": 42}')
            temp_path = f.name

        try:
            data = vexy_json.load(temp_path)
            assert data == {"test": True, "value": 42}
        finally:
            os.unlink(temp_path)

    def test_dump_file(self):
        data = {"name": "Test", "values": [1, 2, 3]}

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            temp_path = f.name

        try:
            vexy_json.dump(data, temp_path)

            # Read back with standard json to verify
            with open(temp_path, "r") as f:
                loaded = json.load(f)

            assert loaded == data
        finally:
            os.unlink(temp_path)

    def test_dump_with_indent(self):
        data = {"a": 1, "b": 2}

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            temp_path = f.name

        try:
            vexy_json.dump(data, temp_path, indent=2)

            with open(temp_path, "r") as f:
                content = f.read()

            # Check that content is indented
            assert '  "a"' in content or '  "b"' in content
        finally:
            os.unlink(temp_path)


class TestSerialization:
    """Test dumps functionality"""

    def test_dumps_basic(self):
        data = {"name": "Test", "value": 42}
        json_str = vexy_json.dumps(data)
        # Parse it back to verify
        assert json.loads(json_str) == data

    def test_dumps_with_indent(self):
        data = {"a": 1, "b": 2}
        json_str = vexy_json.dumps(data, indent=2)
        assert "\n" in json_str  # Should have newlines
        assert json.loads(json_str) == data

    def test_dumps_complex_types(self):
        data = {
            "string": "hello",
            "int": 42,
            "float": 3.14,
            "bool": True,
            "null": None,
            "list": [1, 2, 3],
            "dict": {"nested": True},
        }
        json_str = vexy_json.dumps(data)
        assert json.loads(json_str) == data


class TestErrorHandling:
    """Test error handling and repair"""

    def test_parse_error(self):
        # Completely invalid JSON
        with pytest.raises(ValueError):
            opts = vexy_json.Options(enable_repair=False)
            vexy_json.parse_with_options("{{{invalid}}}", opts)

    def test_repair_mode(self):
        # With repair enabled (default), should handle some errors
        result = vexy_json.parse('{"broken":')
        # Should repair to something valid
        assert isinstance(result, dict)


class TestCompatibility:
    """Test compatibility with standard json module"""

    def test_loads_alias(self):
        # loads should be an alias for parse
        result = vexy_json.loads('{"test": true}')
        assert result == {"test": True}

    def test_version(self):
        # Should have version info
        assert vexy_json.version() is not None
        assert vexy_json.__version__ is not None


if __name__ == "__main__":
    pytest.main([__file__])
