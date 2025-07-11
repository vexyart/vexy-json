"""
Basic functionality tests for vexy_json Python bindings.
"""

import pytest
import vexy_json


class TestBasicParsing:
    """Test basic JSON parsing functionality."""

    def test_parse_simple_object(self):
        """Test parsing a simple JSON object."""
        result = vexy_json.parse('{"key": "value"}')
        assert result == {"key": "value"}

    def test_parse_simple_array(self):
        """Test parsing a simple JSON array."""
        result = vexy_json.parse("[1, 2, 3]")
        assert result == [1, 2, 3]

    def test_parse_null(self):
        """Test parsing null value."""
        result = vexy_json.parse("null")
        assert result is None

    def test_parse_boolean(self):
        """Test parsing boolean values."""
        assert vexy_json.parse("true") is True
        assert vexy_json.parse("false") is False

    def test_parse_numbers(self):
        """Test parsing various number formats."""
        assert vexy_json.parse("42") == 42
        assert vexy_json.parse("-42") == -42
        assert vexy_json.parse("3.14") == 3.14
        assert vexy_json.parse("-3.14") == -3.14
        assert vexy_json.parse("1e5") == 100000.0
        assert vexy_json.parse("1.5e2") == 150.0

    def test_parse_strings(self):
        """Test parsing string values."""
        assert vexy_json.parse('"hello"') == "hello"
        assert vexy_json.parse('"hello world"') == "hello world"
        assert vexy_json.parse('""') == ""

    def test_parse_nested_structures(self):
        """Test parsing nested objects and arrays."""
        complex_json = """
        {
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "metadata": {
                "count": 2,
                "active": true
            }
        }
        """
        result = vexy_json.parse(complex_json)
        expected = {
            "users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}],
            "metadata": {"count": 2, "active": True},
        }
        assert result == expected


class TestForgivingFeatures:
    """Test vexy_json's forgiving JSON features."""

    def test_comments(self):
        """Test single-line and multi-line comments."""
        json_with_comments = """
        {
            // This is a single-line comment
            "name": "Alice",
            /* This is a
               multi-line comment */
            "age": 30
        }
        """
        result = vexy_json.parse(json_with_comments)
        assert result == {"name": "Alice", "age": 30}

    def test_trailing_commas(self):
        """Test trailing commas in objects and arrays."""
        # Object with trailing comma
        result = vexy_json.parse('{"a": 1, "b": 2,}')
        assert result == {"a": 1, "b": 2}

        # Array with trailing comma
        result = vexy_json.parse("[1, 2, 3,]")
        assert result == [1, 2, 3]

    def test_unquoted_keys(self):
        """Test unquoted object keys."""
        result = vexy_json.parse('{key: "value", another_key: 42}')
        assert result == {"key": "value", "another_key": 42}

    def test_single_quotes(self):
        """Test single-quoted strings."""
        result = vexy_json.parse("{'key': 'value'}")
        assert result == {"key": "value"}

    def test_implicit_top_level(self):
        """Test implicit top-level objects and arrays."""
        # Implicit object
        result = vexy_json.parse('key: "value", number: 42')
        assert result == {"key": "value", "number": 42}

        # Implicit array
        result = vexy_json.parse("1, 2, 3")
        assert result == [1, 2, 3]

    def test_newline_as_comma(self):
        """Test newlines as comma separators."""
        json_with_newlines = """
        {
            "a": 1
            "b": 2
        }
        """
        result = vexy_json.parse(json_with_newlines)
        assert result == {"a": 1, "b": 2}

    def test_combined_features(self):
        """Test multiple forgiving features together."""
        forgiving_json = """
        {
            // Configuration object
            name: 'Alice'
            age: 30,
            /* Multi-line
               comment */
            active: true,
        }
        """
        result = vexy_json.parse(forgiving_json)
        assert result == {"name": "Alice", "age": 30, "active": True}


class TestCustomOptions:
    """Test parsing with custom options."""

    def test_disable_comments(self):
        """Test disabling comment support."""
        json_with_comment = '{"key": "value", // comment}'

        # Should work with comments enabled (default)
        result = vexy_json.parse(json_with_comment)
        assert result == {"key": "value"}

        # Should fail with comments disabled
        with pytest.raises(ValueError):
            vexy_json.parse_with_options(json_with_comment, allow_comments=False)

    def test_disable_trailing_commas(self):
        """Test disabling trailing comma support."""
        json_with_trailing = '{"a": 1, "b": 2,}'

        # Should work with trailing commas enabled (default)
        result = vexy_json.parse(json_with_trailing)
        assert result == {"a": 1, "b": 2}

        # Should fail with trailing commas disabled
        with pytest.raises(ValueError):
            vexy_json.parse_with_options(
                json_with_trailing, allow_trailing_commas=False
            )

    def test_disable_unquoted_keys(self):
        """Test disabling unquoted key support."""
        json_unquoted = '{key: "value"}'

        # Should work with unquoted keys enabled (default)
        result = vexy_json.parse(json_unquoted)
        assert result == {"key": "value"}

        # Should fail with unquoted keys disabled
        with pytest.raises(ValueError):
            vexy_json.parse_with_options(json_unquoted, allow_unquoted_keys=False)

    def test_disable_single_quotes(self):
        """Test disabling single quote support."""
        json_single_quotes = "{'key': 'value'}"

        # Should work with single quotes enabled (default)
        result = vexy_json.parse(json_single_quotes)
        assert result == {"key": "value"}

        # Should fail with single quotes disabled
        with pytest.raises(ValueError):
            vexy_json.parse_with_options(json_single_quotes, allow_single_quotes=False)

    def test_disable_implicit_top_level(self):
        """Test disabling implicit top-level support."""
        implicit_object = 'key: "value"'

        # Should work with implicit top-level enabled (default)
        result = vexy_json.parse(implicit_object)
        assert result == {"key": "value"}

        # Should fail with implicit top-level disabled
        with pytest.raises(ValueError):
            vexy_json.parse_with_options(implicit_object, implicit_top_level=False)

    def test_max_depth_limit(self):
        """Test maximum depth limitation."""
        # Create deeply nested structure
        deep_json = '{"a":' * 10 + "1" + "}" * 10

        # Should work with default max_depth (128)
        result = vexy_json.parse(deep_json)
        assert isinstance(result, dict)

        # Should fail with low max_depth
        with pytest.raises(ValueError):
            vexy_json.parse_with_options(deep_json, max_depth=5)


class TestValidation:
    """Test JSON validation functionality."""

    def test_is_valid_true_cases(self):
        """Test cases that should be valid."""
        valid_cases = [
            '{"key": "value"}',
            "[1, 2, 3]",
            "null",
            "true",
            "false",
            "42",
            '"string"',
            "{key: value}",  # unquoted key
            "{'key': 'value'}",  # single quotes
            '{"a": 1,}',  # trailing comma
        ]

        for case in valid_cases:
            assert vexy_json.is_valid(case), f"Should be valid: {case}"

    def test_is_valid_false_cases(self):
        """Test cases that should be invalid."""
        invalid_cases = [
            "",
            "{",
            "}",
            "[",
            "]",
            '{"key":}',
            '{:"value"}',
            "undefined",
            "{key value}",  # missing colon
        ]

        for case in invalid_cases:
            assert not vexy_json.is_valid(case), f"Should be invalid: {case}"


class TestErrorHandling:
    """Test error handling and exceptions."""

    def test_parse_error_exception(self):
        """Test that parse errors raise ValueError."""
        with pytest.raises(ValueError, match="Parse error"):
            vexy_json.parse("{invalid json}")

    def test_parse_with_options_error(self):
        """Test that parse_with_options errors raise ValueError."""
        with pytest.raises(ValueError, match="Parse error"):
            vexy_json.parse_with_options("{invalid}", allow_comments=False)

    def test_empty_input(self):
        """Test parsing empty input."""
        with pytest.raises(ValueError):
            vexy_json.parse("")

    def test_malformed_json(self):
        """Test various malformed JSON inputs."""
        malformed_cases = [
            "{",
            "}",
            "[",
            "]",
            '{"key":}',
            '{:"value"}',
            '{"key": "value"',
            '{"key" "value"}',
        ]

        for case in malformed_cases:
            with pytest.raises(ValueError):
                vexy_json.parse(case)


if __name__ == "__main__":
    pytest.main([__file__])
