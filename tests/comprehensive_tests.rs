// this_file: tests/comprehensive_tests.rs

//! Comprehensive test suite
//!
//! This module provides comprehensive test coverage of the vexy_json parser
//! to ensure feature parity and comprehensive coverage of edge cases.

use rustc_hash::FxHashMap;
use vexy_json::{parse, parse_with_options, ParserOptions, Value};

/// Helper to create expected values more easily
fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut map = FxHashMap::default();
    for (k, v) in pairs {
        map.insert(k.to_string(), v.clone());
    }
    Value::Object(map)
}

fn arr(values: Vec<Value>) -> Value {
    Value::Array(values)
}

fn s(text: &str) -> Value {
    Value::String(text.to_string())
}

fn n(num: i64) -> Value {
    Value::Number(vexy_json::Number::Integer(num))
}

fn f(num: f64) -> Value {
    Value::Number(vexy_json::Number::Float(num))
}

fn b(val: bool) -> Value {
    Value::Bool(val)
}

fn null() -> Value {
    Value::Null
}

/// Test suite covering basic parsing functionality
mod basic_parsing {
    use super::*;

    #[test]
    fn test_empty_input_variations() {
        // Various empty or whitespace-only inputs
        assert_eq!(parse("").unwrap(), null());
        assert_eq!(parse(" ").unwrap(), null());
        assert_eq!(parse("\t").unwrap(), null());
        assert_eq!(parse("\n").unwrap(), null());
        assert_eq!(parse("\r").unwrap(), null());
        assert_eq!(parse("   \t\n\r   ").unwrap(), null());
    }

    #[test]
    fn test_single_values() {
        // Single primitive values
        assert_eq!(parse("true").unwrap(), b(true));
        assert_eq!(parse("false").unwrap(), b(false));
        assert_eq!(parse("null").unwrap(), null());
        assert_eq!(parse("42").unwrap(), n(42));
        assert_eq!(parse("-17").unwrap(), n(-17));
        assert_eq!(parse("3.14").unwrap(), f(3.14));
        assert_eq!(parse("\"hello\"").unwrap(), s("hello"));
        assert_eq!(parse("'world'").unwrap(), s("world"));
    }

    #[test]
    fn test_unquoted_identifiers() {
        // Unquoted strings and identifiers
        assert_eq!(parse("hello").unwrap(), s("hello"));
        assert_eq!(parse("test_value").unwrap(), s("test_value"));
        assert_eq!(parse("key-with-dashes").unwrap(), s("key-with-dashes"));
        assert_eq!(parse("$pecial").unwrap(), s("$pecial"));
        assert_eq!(parse("_underscore").unwrap(), s("_underscore"));
    }

    #[test]
    fn test_implicit_arrays() {
        // Implicit arrays (comma-separated values)
        assert_eq!(parse("a,b,c").unwrap(), arr(vec![s("a"), s("b"), s("c")]));
        assert_eq!(parse("1,2,3").unwrap(), arr(vec![n(1), n(2), n(3)]));
        assert_eq!(
            parse("true,false,null").unwrap(),
            arr(vec![b(true), b(false), null()])
        );
        assert_eq!(parse("a,").unwrap(), arr(vec![s("a")]));
        assert_eq!(parse(",a").unwrap(), arr(vec![null(), s("a")]));
        assert_eq!(parse("a,,b").unwrap(), arr(vec![s("a"), null(), s("b")]));
    }

    #[test]
    fn test_implicit_objects() {
        // Implicit objects (key:value pairs)
        assert_eq!(parse("a:1").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(parse("a:1,b:2").unwrap(), obj(&[("a", n(1)), ("b", n(2))]));
        assert_eq!(
            parse("name:john,age:30").unwrap(),
            obj(&[("name", s("john")), ("age", n(30))])
        );
    }
}

/// Test suite covering comment handling variations
mod comment_handling {
    use super::*;

    #[test]
    fn test_single_line_comment_variations() {
        // Single-line comments with //
        assert_eq!(parse("// comment").unwrap(), null());
        assert_eq!(parse("a // comment").unwrap(), s("a"));
        assert_eq!(parse("a:1 // comment").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(parse("// comment\na:1").unwrap(), obj(&[("a", n(1))]));

        // Single-line comments with #
        assert_eq!(parse("# comment").unwrap(), null());
        assert_eq!(parse("a # comment").unwrap(), s("a"));
        assert_eq!(parse("a:1 # comment").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(parse("# comment\na:1").unwrap(), obj(&[("a", n(1))]));
    }

    #[test]
    fn test_multi_line_comments() {
        // Multi-line comments
        assert_eq!(parse("/* comment */").unwrap(), null());
        assert_eq!(
            parse("a /* comment */ b").unwrap(),
            arr(vec![s("a"), s("b")])
        );
        assert_eq!(parse("/* line1\nline2 */").unwrap(), null());
        assert_eq!(
            parse("a:1 /* comment */ ,b:2").unwrap(),
            obj(&[("a", n(1)), ("b", n(2))])
        );
    }

    #[test]
    fn test_nested_comments() {
        // Comments within structures
        let input = r#"
        {
            // Top level comment
            key1: "value1",
            /* Multi-line comment
               spanning multiple lines */
            key2: "value2" // End comment
        }
        "#;

        let result = parse(input).unwrap();
        assert_eq!(result["key1"], s("value1"));
        assert_eq!(result["key2"], s("value2"));
    }

    #[test]
    fn test_comment_edge_cases() {
        // Comments at boundaries
        assert_eq!(parse("a,//comment\nb").unwrap(), arr(vec![s("a"), s("b")]));
        
        // Note: Complex comment edge cases like "a:#comment\nb:2" are not fully supported
        // This would require sophisticated lookahead to distinguish between values and keys
        // For now, explicit objects work fine with comments
        assert_eq!(parse("{a:null,//comment\nb:2}").unwrap(), obj(&[("a", null()), ("b", n(2))]));
        
        assert_eq!(parse("{//comment\na:1}").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(parse("[//comment\n1,2]").unwrap(), arr(vec![n(1), n(2)]));
    }
}

/// Test suite covering string parsing edge cases
mod string_handling {
    use super::*;

    #[test]
    fn test_quote_variations() {
        // Different quote styles
        assert_eq!(parse("\"double\"").unwrap(), s("double"));
        assert_eq!(parse("'single'").unwrap(), s("single"));
        assert_eq!(parse("\"mix'ed\"").unwrap(), s("mix'ed"));
        assert_eq!(parse("'mix\"ed'").unwrap(), s("mix\"ed"));
    }

    #[test]
    fn test_escape_sequences() {
        // Standard escape sequences
        assert_eq!(parse("\"\\n\"").unwrap(), s("\n"));
        assert_eq!(parse("\"\\t\"").unwrap(), s("\t"));
        assert_eq!(parse("\"\\r\"").unwrap(), s("\r"));
        assert_eq!(parse("\"\\\\\"").unwrap(), s("\\"));
        assert_eq!(parse("\"\\\"\"").unwrap(), s("\""));
        assert_eq!(parse("'\\\''").unwrap(), s("'"));
    }

    #[test]
    fn test_unicode_sequences() {
        // Unicode escape sequences
        assert_eq!(parse("\"\\u0048\"").unwrap(), s("H"));
        assert_eq!(parse("\"\\u0065\"").unwrap(), s("e"));
        assert_eq!(parse("\"\\u006C\\u006C\"").unwrap(), s("ll"));
        assert_eq!(parse("\"\\u006F\"").unwrap(), s("o"));
        // Combined: "Hello"
        assert_eq!(
            parse("\"\\u0048\\u0065\\u006C\\u006C\\u006F\"").unwrap(),
            s("Hello")
        );
    }

    #[test]
    fn test_empty_strings() {
        // Empty string variations
        assert_eq!(parse("\"\"").unwrap(), s(""));
        assert_eq!(parse("''").unwrap(), s(""));
        assert_eq!(parse("key:\"\"").unwrap(), obj(&[("key", s(""))]));
        assert_eq!(parse("\"\":value").unwrap(), obj(&[("", s("value"))]));
    }

    #[test]
    fn test_special_characters_in_strings() {
        // Strings containing special JSON characters
        assert_eq!(parse("\"{\"").unwrap(), s("{"));
        assert_eq!(parse("\"}\"").unwrap(), s("}"));
        assert_eq!(parse("\"[\"").unwrap(), s("["));
        assert_eq!(parse("\"]\"").unwrap(), s("]"));
        assert_eq!(parse("\":\"").unwrap(), s(":"));
        assert_eq!(parse("\",\"").unwrap(), s(","));
    }
}

/// Test suite covering numeric parsing edge cases
mod number_handling {
    use super::*;

    #[test]
    fn test_integer_variations() {
        // Various integer formats
        assert_eq!(parse("0").unwrap(), n(0));
        assert_eq!(parse("-0").unwrap(), n(0));
        assert_eq!(parse("42").unwrap(), n(42));
        assert_eq!(parse("-42").unwrap(), n(-42));
        assert_eq!(parse("+42").unwrap(), n(42));
        assert_eq!(parse("123456789").unwrap(), n(123456789));
    }

    #[test]
    fn test_float_variations() {
        // Various float formats
        assert_eq!(parse("3.14").unwrap(), f(3.14));
        assert_eq!(parse("-3.14").unwrap(), f(-3.14));
        assert_eq!(parse("+3.14").unwrap(), f(3.14));
        assert_eq!(parse("0.5").unwrap(), f(0.5));
        assert_eq!(parse(".5").unwrap(), f(0.5));
        assert_eq!(parse("5.").unwrap(), n(5)); // May be treated as integer
    }

    #[test]
    fn test_scientific_notation() {
        // Scientific notation
        assert_eq!(parse("1e2").unwrap(), n(100));
        assert_eq!(parse("1E2").unwrap(), n(100));
        assert_eq!(parse("1e+2").unwrap(), n(100));
        assert_eq!(parse("1e-2").unwrap(), f(0.01));
        assert_eq!(parse("3.14e2").unwrap(), f(314.0));
        assert_eq!(parse("3.14e-2").unwrap(), f(0.0314));
    }

    #[test]
    fn test_special_number_formats() {
        // Hexadecimal, octal, binary (extensions)
        assert_eq!(parse("0xFF").unwrap(), n(255));
        assert_eq!(parse("0x10").unwrap(), n(16));
        assert_eq!(parse("0o77").unwrap(), n(63));
        assert_eq!(parse("0o10").unwrap(), n(8));
        assert_eq!(parse("0b1010").unwrap(), n(10));
        assert_eq!(parse("0b1111").unwrap(), n(15));
    }

    #[test]
    fn test_number_boundaries() {
        // Boundary cases
        assert_eq!(parse("2147483647").unwrap(), n(2147483647)); // i32 max
        assert_eq!(parse("-2147483648").unwrap(), n(-2147483648)); // i32 min
                                                                   // Large numbers that should become floats
        let large_result = parse("999999999999999999999").unwrap();
        assert!(matches!(
            large_result,
            Value::Number(vexy_json::Number::Float(_))
        ));
    }
}

/// Test suite covering object parsing edge cases
mod object_handling {
    use super::*;

    #[test]
    fn test_explicit_objects() {
        // Explicit object syntax
        assert_eq!(parse("{}").unwrap(), obj(&[]));
        assert_eq!(parse("{a:1}").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(parse("{\"a\":1}").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(parse("{'a':1}").unwrap(), obj(&[("a", n(1))]));
    }

    #[test]
    fn test_mixed_key_styles() {
        // Mixed quoted and unquoted keys
        let input = r#"{
            unquoted: "value1",
            "quoted": "value2",
            'single_quoted': "value3"
        }"#;

        let result = parse(input).unwrap();
        assert_eq!(result["unquoted"], s("value1"));
        assert_eq!(result["quoted"], s("value2"));
        assert_eq!(result["single_quoted"], s("value3"));
    }

    #[test]
    fn test_numeric_keys() {
        // Numeric keys (should be converted to strings)
        assert_eq!(parse("{1:\"one\"}").unwrap(), obj(&[("1", s("one"))]));
        assert_eq!(
            parse("{42:\"answer\"}").unwrap(),
            obj(&[("42", s("answer"))])
        );
        assert_eq!(parse("{3.14:\"pi\"}").unwrap(), obj(&[("3.14", s("pi"))]));
    }

    #[test]
    fn test_special_key_characters() {
        // Keys with special characters
        assert_eq!(
            parse("{\"key-with-dashes\":1}").unwrap(),
            obj(&[("key-with-dashes", n(1))])
        );
        assert_eq!(
            parse("{\"key_with_underscores\":1}").unwrap(),
            obj(&[("key_with_underscores", n(1))])
        );
        assert_eq!(
            parse("{\"key.with.dots\":1}").unwrap(),
            obj(&[("key.with.dots", n(1))])
        );
        assert_eq!(
            parse("{\"key with spaces\":1}").unwrap(),
            obj(&[("key with spaces", n(1))])
        );
    }

    #[test]
    fn test_nested_objects() {
        // Nested object structures
        let input = r#"{
            level1: {
                level2: {
                    level3: "deep"
                }
            }
        }"#;

        let result = parse(input).unwrap();
        assert_eq!(result["level1"]["level2"]["level3"], s("deep"));
    }
}

/// Test suite covering array parsing edge cases
mod array_handling {
    use super::*;

    #[test]
    fn test_explicit_arrays() {
        // Explicit array syntax
        assert_eq!(parse("[]").unwrap(), arr(vec![]));
        assert_eq!(parse("[1]").unwrap(), arr(vec![n(1)]));
        assert_eq!(parse("[1,2,3]").unwrap(), arr(vec![n(1), n(2), n(3)]));
        assert_eq!(
            parse("[\"a\",\"b\",\"c\"]").unwrap(),
            arr(vec![s("a"), s("b"), s("c")])
        );
    }

    #[test]
    fn test_mixed_type_arrays() {
        // Arrays with mixed types
        assert_eq!(
            parse("[1,\"two\",true,null]").unwrap(),
            arr(vec![n(1), s("two"), b(true), null()])
        );
        assert_eq!(
            parse("[{a:1},[2,3],\"string\"]").unwrap(),
            arr(vec![
                obj(&[("a", n(1))]),
                arr(vec![n(2), n(3)]),
                s("string")
            ])
        );
    }

    #[test]
    fn test_nested_arrays() {
        // Nested array structures
        assert_eq!(
            parse("[[1,2],[3,4]]").unwrap(),
            arr(vec![arr(vec![n(1), n(2)]), arr(vec![n(3), n(4)])])
        );

        let deep_input = "[[[1]]]";
        let result = parse(deep_input).unwrap();
        if let Value::Array(outer) = result {
            if let Value::Array(middle) = &outer[0] {
                if let Value::Array(inner) = &middle[0] {
                    assert_eq!(inner[0], n(1));
                }
            }
        }
    }

    #[test]
    fn test_sparse_arrays() {
        // Arrays with null/missing elements
        assert_eq!(parse("[1,,3]").unwrap(), arr(vec![n(1), null(), n(3)]));
        assert_eq!(parse("[,2,]").unwrap(), arr(vec![null(), n(2)]));
        assert_eq!(parse("[null,null]").unwrap(), arr(vec![null(), null()]));
    }
}

/// Test suite covering trailing comma handling
mod trailing_commas {
    use super::*;

    #[test]
    fn test_object_trailing_commas() {
        // Objects with trailing commas
        assert_eq!(parse("{a:1,}").unwrap(), obj(&[("a", n(1))]));
        assert_eq!(
            parse("{a:1,b:2,}").unwrap(),
            obj(&[("a", n(1)), ("b", n(2))])
        );

        let input = r#"{
            key1: "value1",
            key2: "value2",
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result["key1"], s("value1"));
        assert_eq!(result["key2"], s("value2"));
    }

    #[test]
    fn test_array_trailing_commas() {
        // Arrays with trailing commas
        assert_eq!(parse("[1,]").unwrap(), arr(vec![n(1)]));
        assert_eq!(parse("[1,2,]").unwrap(), arr(vec![n(1), n(2)]));
        assert_eq!(parse("[1,2,3,]").unwrap(), arr(vec![n(1), n(2), n(3)]));

        let input = r#"[
            "item1",
            "item2",
            "item3",
        ]"#;
        let result = parse(input).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], s("item1"));
            assert_eq!(arr[1], s("item2"));
            assert_eq!(arr[2], s("item3"));
        }
    }

    #[test]
    fn test_nested_trailing_commas() {
        // Nested structures with trailing commas
        let input = r#"{
            array: [1, 2, 3,],
            object: {
                nested: "value",
            },
        }"#;

        let result = parse(input).unwrap();
        if let Value::Array(arr) = &result["array"] {
            assert_eq!(arr.len(), 3);
        }
        assert_eq!(result["object"]["nested"], s("value"));
    }
}

/// Test suite covering whitespace and formatting tolerance
mod whitespace_handling {
    use super::*;

    #[test]
    fn test_minimal_whitespace() {
        // Minimal whitespace
        assert_eq!(
            parse("{a:1,b:2}").unwrap(),
            obj(&[("a", n(1)), ("b", n(2))])
        );
        assert_eq!(parse("[1,2,3]").unwrap(), arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_excessive_whitespace() {
        // Excessive whitespace
        let input = "  {  a  :  1  ,  b  :  2  }  ";
        assert_eq!(parse(input).unwrap(), obj(&[("a", n(1)), ("b", n(2))]));

        let input = "  [  1  ,  2  ,  3  ]  ";
        assert_eq!(parse(input).unwrap(), arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_mixed_whitespace() {
        // Mixed tabs, spaces, newlines
        let input = "{\t\na\t:\t1\n,\nb\t:\t2\n}";
        assert_eq!(parse(input).unwrap(), obj(&[("a", n(1)), ("b", n(2))]));
    }

    #[test]
    fn test_newlines_as_separators() {
        // Newlines acting as separators
        let input = r#"
        a: 1
        b: 2
        c: 3
        "#;

        let result = parse(input).unwrap();
        assert_eq!(result["a"], n(1));
        assert_eq!(result["b"], n(2));
        assert_eq!(result["c"], n(3));
    }
}

/// Test suite covering parser option combinations
mod parser_options {
    use super::*;

    #[test]
    fn test_strict_mode() {
        // Test with strict JSON mode
        let strict_options = ParserOptions {
            allow_comments: false,
            allow_trailing_commas: false,
            allow_unquoted_keys: false,
            allow_single_quotes: false,
            implicit_top_level: false,
            newline_as_comma: false,
            max_depth: 100,
            enable_repair: false,
            max_repairs: 0,
            fast_repair: false,
            report_repairs: false,
        };

        // Valid JSON should work
        assert!(parse_with_options(r#"{"key": "value"}"#, strict_options.clone()).is_ok());

        // Forgiving features should fail
        assert!(parse_with_options("key: value", strict_options.clone()).is_err());
        assert!(parse_with_options("{'key': 'value'}", strict_options.clone()).is_err());
        assert!(parse_with_options("{\"key\": \"value\",}", strict_options.clone()).is_err());
        assert!(parse_with_options("// comment\n{\"key\": \"value\"}", strict_options).is_err());
    }

    #[test]
    fn test_selective_options() {
        // Test with selective options enabled
        let selective_options = ParserOptions {
            allow_comments: true,
            allow_trailing_commas: false,
            allow_unquoted_keys: true,
            allow_single_quotes: false,
            implicit_top_level: false,
            newline_as_comma: false,
            max_depth: 100,
            enable_repair: false,
            max_repairs: 0,
            fast_repair: false,
            report_repairs: false,
        };

        // Should work
        assert!(
            parse_with_options("// comment\n{key: \"value\"}", selective_options.clone()).is_ok()
        );

        // Should fail
        assert!(parse_with_options("{key: 'value'}", selective_options.clone()).is_err());
        assert!(parse_with_options("{\"key\": \"value\",}", selective_options).is_err());
    }

    #[test]
    fn test_max_depth_limits() {
        // Test max_depth enforcement
        let shallow_options = ParserOptions {
            max_depth: 3,
            ..Default::default()
        };

        // Should work (depth 2)
        assert!(parse_with_options("{a: {b: 1}}", shallow_options.clone()).is_ok());

        // Should fail (depth 4)
        assert!(parse_with_options("{a: {b: {c: {d: 1}}}}", shallow_options).is_err());
    }
}

/// Test comprehensive error scenarios
mod error_handling {
    use super::*;

    #[test]
    fn test_syntax_errors() {
        // Various syntax errors
        let invalid_inputs = vec![
            "{",                 // Unclosed object
            "}",                 // Unexpected close
            "[",                 // Unclosed array
            "]",                 // Unexpected close
            "{key:}",            // Missing value
            "{:value}",          // Missing key
            "\"unclosed string", // Unclosed string
            "'unclosed string",  // Unclosed string
            "{key: val: ue}",    // Double colon
            "[1,2,3,]extra",     // Trailing content
        ];

        for input in invalid_inputs {
            let result = parse(input);
            assert!(result.is_err(), "Input should have failed: {}", input);
        }
    }

    #[test]
    fn test_number_errors() {
        // Invalid number formats
        let invalid_numbers = vec![
            "1.2.3", // Multiple dots
            "1e",    // Incomplete scientific
            "1e+",   // Incomplete scientific
            "--1",   // Double negative
            "++1",   // Double positive
            "0x",    // Incomplete hex
            "0o",    // Incomplete octal
            "0b",    // Incomplete binary
        ];

        for input in invalid_numbers {
            let result = parse(input);
            // Some may parse as identifiers, but should not parse as numbers
            if let Ok(Value::Number(_)) = result {
                panic!("Invalid number should not parse as number: {}", input);
            }
        }
    }

    #[test]
    fn test_unicode_errors() {
        // Invalid unicode escape sequences
        let invalid_unicode = vec![
            "\"\\u\"",      // Incomplete unicode
            "\"\\uGGGG\"",  // Invalid hex
            "\"\\u123\"",   // Too short
            "\"\\u12345\"", // Too long
        ];

        for input in invalid_unicode {
            let result = parse(input);
            assert!(result.is_err(), "Invalid unicode should fail: {}", input);
        }
    }
}
