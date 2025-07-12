// this_file: tests/advanced_features.rs

//! Advanced forgiving JSON feature tests
//!
//! This module tests advanced features that are specific to forgiving JSON parsing,
//! including potential future features and edge cases that push the boundaries of the parser.

use rustc_hash::FxHashMap;
use vexy_json::{parse, parse_with_options, ParserOptions, Value};

/// Helper functions for creating test values
#[allow(dead_code)]
fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut map = FxHashMap::default();
    for (k, v) in pairs {
        map.insert(k.to_string(), v.clone());
    }
    Value::Object(map)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn b(val: bool) -> Value {
    Value::Bool(val)
}

#[allow(dead_code)]
fn null() -> Value {
    Value::Null
}

/// Tests for complex nested structures
mod complex_structures {
    use super::*;

    #[test]
    fn test_deeply_nested_objects() {
        let input = r#"
        {
            app: {
                config: {
                    database: {
                        connection: {
                            host: "localhost",
                            port: 5432,
                            ssl: {
                                enabled: true,
                                cert: "/path/to/cert"
                            }
                        }
                    }
                }
            }
        }
        "#;

        let result = parse(input).unwrap();
        assert_eq!(
            result["app"]["config"]["database"]["connection"]["host"],
            s("localhost")
        );
        assert_eq!(
            result["app"]["config"]["database"]["connection"]["port"],
            n(5432)
        );
        assert_eq!(
            result["app"]["config"]["database"]["connection"]["ssl"]["enabled"],
            Value::Bool(true)
        );
    }

    #[test]
    fn test_mixed_structure_complexity() {
        let input = r#"
        {
            users: [
                {
                    name: "Alice",
                    permissions: ["read", "write"],
                    settings: {
                        theme: "dark",
                        notifications: true
                    }
                },
                {
                    name: "Bob",
                    permissions: ["read"],
                    settings: {
                        theme: "light",
                        notifications: false
                    }
                }
            ],
            global_settings: {
                timeout: 30000,
                retry_attempts: 3
            }
        }
        "#;

        let result = parse(input).unwrap();

        // Test user array access
        if let Value::Array(users) = &result["users"] {
            assert_eq!(users.len(), 2);
            assert_eq!(users[0]["name"], s("Alice"));
            assert_eq!(users[1]["name"], s("Bob"));

            // Test nested permissions array
            if let Value::Array(perms) = &users[0]["permissions"] {
                assert_eq!(perms.len(), 2);
                assert_eq!(perms[0], s("read"));
                assert_eq!(perms[1], s("write"));
            }
        }

        assert_eq!(result["global_settings"]["timeout"], n(30000));
    }

    #[test]
    fn test_asymmetric_nesting() {
        // Objects with varying nesting depths
        let input = r#"
        {
            simple: "value",
            moderate: {
                nested: "value"
            },
            complex: {
                level1: {
                    level2: {
                        level3: {
                            deep: "value"
                        }
                    }
                }
            }
        }
        "#;

        let result = parse(input).unwrap();
        assert_eq!(result["simple"], s("value"));
        assert_eq!(result["moderate"]["nested"], s("value"));
        assert_eq!(
            result["complex"]["level1"]["level2"]["level3"]["deep"],
            s("value")
        );
    }
}

/// Tests for edge cases in value parsing
mod value_edge_cases {
    use super::*;

    #[test]
    fn test_boundary_numbers() {
        // Test numbers at various boundaries
        assert_eq!(parse("0").unwrap(), n(0));
        assert_eq!(parse("-0").unwrap(), n(0));
        assert_eq!(parse("1").unwrap(), n(1));
        assert_eq!(parse("-1").unwrap(), n(-1));

        // Large integers
        assert_eq!(parse("9007199254740991").unwrap(), n(9007199254740991)); // Max safe integer in JS
        assert_eq!(parse("-9007199254740991").unwrap(), n(-9007199254740991));

        // Very small floats
        let small_result = parse("1e-300").unwrap();
        assert!(matches!(
            small_result,
            Value::Number(vexy_json::Number::Float(_))
        ));

        // Very large floats
        let large_result = parse("1e300").unwrap();
        assert!(matches!(
            large_result,
            Value::Number(vexy_json::Number::Float(_))
        ));
    }

    #[test]
    fn test_special_float_values() {
        // Test special float representations
        assert_eq!(parse("0.0").unwrap(), f(0.0));
        assert_eq!(parse("-0.0").unwrap(), f(-0.0));

        // Very precise decimals
        assert_eq!(parse("0.1").unwrap(), f(0.1));
        assert_eq!(parse("0.123456789").unwrap(), f(0.123456789));

        // Scientific notation edge cases
        assert_eq!(parse("1e0").unwrap(), n(1));
        assert_eq!(parse("1E0").unwrap(), n(1));
        assert_eq!(parse("1.0e0").unwrap(), f(1.0));
    }

    #[test]
    fn test_string_edge_cases() {
        // Empty and whitespace strings
        assert_eq!(parse("\"\"").unwrap(), s(""));
        assert_eq!(parse("\" \"").unwrap(), s(" "));
        assert_eq!(parse("\"\\t\"").unwrap(), s("\t"));
        assert_eq!(parse("\"\\n\"").unwrap(), s("\n"));

        // Strings with JSON-like content
        assert_eq!(
            parse("\"{\\\"nested\\\": \\\"value\\\"}\"").unwrap(),
            s("{\"nested\": \"value\"}")
        );
        assert_eq!(parse("\"[1, 2, 3]\"").unwrap(), s("[1, 2, 3]"));

        // Very long strings
        let long_string = "a".repeat(1000);
        let long_input = format!("\"{long_string}\"");
        assert_eq!(parse(&long_input).unwrap(), s(&long_string));
    }

    #[test]
    fn test_identifier_edge_cases() {
        // Various identifier formats
        assert_eq!(parse("a").unwrap(), s("a"));
        assert_eq!(parse("_").unwrap(), s("_"));
        assert_eq!(parse("$").unwrap(), s("$"));
        assert_eq!(parse("a1").unwrap(), s("a1"));
        assert_eq!(parse("_private").unwrap(), s("_private"));
        assert_eq!(parse("$global").unwrap(), s("$global"));

        // Identifiers that look like keywords
        assert_eq!(parse("TRUE").unwrap(), s("TRUE"));
        assert_eq!(parse("False").unwrap(), s("False"));
        assert_eq!(parse("NULL").unwrap(), s("NULL"));
        assert_eq!(parse("undefined").unwrap(), s("undefined"));
    }
}

/// Tests for whitespace and formatting tolerance
mod formatting_tolerance {
    use super::*;

    #[test]
    fn test_unicode_whitespace() {
        // Various Unicode whitespace characters
        let unicode_spaces = [
            "\u{0020}", // Regular space
            "\u{00A0}", // Non-breaking space
            "\u{2000}", // En quad
            "\u{2001}", // Em quad
            "\u{2002}", // En space
            "\u{2003}", // Em space
            "\u{2009}", // Thin space
            "\u{200A}", // Hair space
        ];

        for space in &unicode_spaces {
            let input = format!("{space}{{{space}key{space}: {space}value{space}}}{space}");
            let result = parse(&input);
            // Some unicode spaces might not be recognized, but shouldn't crash
            if result.is_ok() {
                assert_eq!(result.unwrap()["key"], s("value"));
            }
        }
    }

    #[test]
    fn test_mixed_line_endings() {
        // Different line ending styles
        let inputs = vec![
            "a:1\nb:2",   // Unix (LF)
            "a:1\r\nb:2", // Windows (CRLF)
            "a:1\rb:2",   // Classic Mac (CR)
            "a:1\n\rb:2", // Mixed
        ];

        for input in inputs {
            let result = parse(input).unwrap();
            assert_eq!(result["a"], n(1));
            assert_eq!(result["b"], n(2));
        }
    }

    #[test]
    fn test_extreme_formatting() {
        // Extremely verbose formatting
        let input = r#"


        {


            key1    :    "value1"    ,


            key2    :    "value2"


        }


        "#;

        let result = parse(input).unwrap();
        assert_eq!(result["key1"], s("value1"));
        assert_eq!(result["key2"], s("value2"));
    }

    #[test]
    fn test_minimal_formatting() {
        // Minimal whitespace
        let input = "{key1:\"value1\",key2:\"value2\"}";
        let result = parse(input).unwrap();
        assert_eq!(result["key1"], s("value1"));
        assert_eq!(result["key2"], s("value2"));
    }
}

/// Tests for complex comment scenarios
mod advanced_comments {
    use super::*;

    #[test]
    fn test_comment_preservation_boundaries() {
        // Comments at critical parsing boundaries
        let inputs = vec![
            "//comment\n{a:1}",
            "{//comment\na:1}",
            "{a://comment\n1}",
            "{a:1//comment\n}",
            "{a:1}//comment",
        ];

        for input in inputs {
            let result = parse(input).unwrap();
            assert_eq!(result["a"], n(1));
        }
    }

    #[test]
    fn test_nested_multiline_comments() {
        // Nested multi-line comment-like content
        let input = r#"
        {
            /* outer comment
               /* inner comment-like text */
               still in outer comment */
            key: "value"
        }
        "#;

        let result = parse(input).unwrap();
        assert_eq!(result["key"], s("value"));
    }

    #[test]
    fn test_comments_with_special_characters() {
        // Comments containing various special characters
        let input = r#"
        {
            // Comment with "quotes" and 'apostrophes'
            key1: "value1",
            /* Comment with {braces} and [brackets] and :colons */
            key2: "value2",
            # Comment with $pecial ch@racters and numb3rs
            key3: "value3"
        }
        "#;

        let result = parse(input).unwrap();
        assert_eq!(result["key1"], s("value1"));
        assert_eq!(result["key2"], s("value2"));
        assert_eq!(result["key3"], s("value3"));
    }

    #[test]
    fn test_comment_line_endings() {
        // Comments with different line ending styles
        let inputs = vec![
            "// comment\na:1",
            "// comment\r\na:1",
            "// comment\ra:1",
            "# comment\na:1",
            "# comment\r\na:1",
        ];

        for input in inputs {
            let result = parse(input).unwrap();
            assert_eq!(result["a"], n(1));
        }
    }
}

/// Tests for performance and stress scenarios
mod stress_tests {
    use super::*;

    #[test]
    fn test_wide_objects() {
        // Objects with many keys
        let mut input = String::from("{");
        for i in 0..100 {
            if i > 0 {
                input.push(',');
            }
            input.push_str(&format!("key{i}: {i}"));
        }
        input.push('}');

        let result = parse(&input).unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(obj.len(), 100);
            assert_eq!(obj["key0"], n(0));
            assert_eq!(obj["key99"], n(99));
        }
    }

    #[test]
    fn test_wide_arrays() {
        // Arrays with many elements
        let mut input = String::from("[");
        for i in 0..100 {
            if i > 0 {
                input.push(',');
            }
            input.push_str(&i.to_string());
        }
        input.push(']');

        let result = parse(&input).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 100);
            assert_eq!(arr[0], n(0));
            assert_eq!(arr[99], n(99));
        }
    }

    #[test]
    fn test_deeply_nested_arrays() {
        // Deeply nested array structure
        let mut input = String::new();
        let depth = 20;

        // Create nested structure: [[[[...]]]]]
        for _ in 0..depth {
            input.push('[');
        }
        input.push_str("\"deep\"");
        for _ in 0..depth {
            input.push(']');
        }

        let result = parse(&input);
        assert!(result.is_ok(), "Deep nesting should be handled");

        // Navigate to the deep value
        let mut current = &result.unwrap();
        for _ in 0..depth {
            if let Value::Array(arr) = current {
                current = &arr[0];
            }
        }
        assert_eq!(*current, s("deep"));
    }

    #[test]
    fn test_alternating_nested_structures() {
        // Alternating object/array nesting
        let input = r#"
        {
            level1: [
                {
                    level2: [
                        {
                            level3: [
                                {
                                    value: "nested"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

        let result = parse(input).unwrap();
        let value = &result["level1"][0]["level2"][0]["level3"][0]["value"];
        assert_eq!(*value, s("nested"));
    }
}

/// Tests for parser configuration edge cases
mod configuration_edge_cases {
    use super::*;

    #[test]
    fn test_zero_max_depth() {
        // Test with max_depth = 0 (should allow only primitives)
        let options = ParserOptions {
            max_depth: 0,
            ..Default::default()
        };

        // Primitives should work
        assert!(parse_with_options("42", options.clone()).is_ok());
        assert!(parse_with_options("\"string\"", options.clone()).is_ok());
        assert!(parse_with_options("true", options.clone()).is_ok());

        // Structures should fail
        assert!(parse_with_options("{}", options.clone()).is_err());
        assert!(parse_with_options("[]", options.clone()).is_err());
    }

    #[test]
    fn test_minimal_max_depth() {
        // Test with max_depth = 1 (allows one level of nesting)
        let options = ParserOptions {
            max_depth: 1,
            ..Default::default()
        };

        // Single level should work
        assert!(parse_with_options("{a: 1}", options.clone()).is_ok());
        assert!(parse_with_options("[1, 2, 3]", options.clone()).is_ok());

        // Double nesting should fail
        assert!(parse_with_options("{a: {b: 1}}", options.clone()).is_err());
        assert!(parse_with_options("[[1]]", options.clone()).is_err());
    }

    #[test]
    fn test_all_features_disabled() {
        // Test with all forgiving features disabled
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

        // Standard JSON should work
        assert!(parse_with_options(r#"{"key": "value"}"#, strict_options.clone()).is_ok());
        assert!(parse_with_options(r#"[1, 2, 3]"#, strict_options.clone()).is_ok());
        assert!(parse_with_options("42", strict_options.clone()).is_ok());

        // All forgiving features should fail
        assert!(parse_with_options("// comment", strict_options.clone()).is_err());
        assert!(parse_with_options("{key: value}", strict_options.clone()).is_err());
        assert!(parse_with_options("{'key': 'value'}", strict_options.clone()).is_err());
        assert!(parse_with_options(r#"{"key": "value",}"#, strict_options.clone()).is_err());
        assert!(parse_with_options("key: value", strict_options).is_err());
    }
}

/// Tests for Unicode and internationalization
mod unicode_tests {
    use super::*;

    #[test]
    fn test_unicode_strings() {
        // Various Unicode characters in strings
        assert_eq!(parse("\"Hello 世界\"").unwrap(), s("Hello 世界"));
        assert_eq!(parse("\"🚀 rocket\"").unwrap(), s("🚀 rocket"));
        assert_eq!(parse("\"café\"").unwrap(), s("café"));
        assert_eq!(parse("\"naïve\"").unwrap(), s("naïve"));
        assert_eq!(parse("\"Москва\"").unwrap(), s("Москва"));
        assert_eq!(parse("\"العربية\"").unwrap(), s("العربية"));
    }

    #[test]
    fn test_unicode_keys() {
        // Unicode characters in object keys
        let input = r#"{
            "世界": "world",
            "🚀": "rocket",
            "café": "coffee"
        }"#;

        let result = parse(input).unwrap();
        assert_eq!(result["世界"], s("world"));
        assert_eq!(result["🚀"], s("rocket"));
        assert_eq!(result["café"], s("coffee"));
    }

    #[test]
    fn test_unicode_escape_sequences() {
        // Unicode escape sequences
        assert_eq!(parse("\"\\u4e16\\u754c\"").unwrap(), s("世界")); // "世界"
        assert_eq!(
            parse("\"\\u0048\\u0065\\u006c\\u006c\\u006f\"").unwrap(),
            s("Hello")
        );

        // Mixed Unicode and ASCII
        assert_eq!(parse("\"Hello \\u4e16\\u754c\"").unwrap(), s("Hello 世界"));
    }

    #[test]
    fn test_unicode_normalization() {
        // Different Unicode normalization forms (if supported)
        let cafe_nfc = "café"; // NFC form
        let cafe_nfd = "cafe\u{0301}"; // NFD form (e + combining acute accent)

        // Both should parse successfully (though they may not be equal)
        assert!(parse(&format!("\"{cafe_nfc}\"")).is_ok());
        assert!(parse(&format!("\"{cafe_nfd}\"")).is_ok());
    }
}
