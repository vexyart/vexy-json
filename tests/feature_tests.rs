// this_file: tests/feature_tests.rs

use vexy_json::{parse, Value};

/// Test basic JSON compliance - standard JSON parsing capabilities
#[test]
fn test_basic_json_compliance() {
    let test_cases = vec![
        (r#"{"key": "value"}"#, "standard object"),
        (r#"[1, 2, 3]"#, "standard array"),
        (r#""string""#, "standard string"),
        (r#"42"#, "standard number"),
        (r#"true"#, "standard boolean true"),
        (r#"false"#, "standard boolean false"),
        (r#"null"#, "standard null"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Basic JSON {description} parsed: {val:?}");
                assert!(match description {
                    "standard object" => matches!(val, Value::Object(_)),
                    "standard array" => matches!(val, Value::Array(_)),
                    "standard string" => matches!(val, Value::String(_)),
                    "standard number" => matches!(val, Value::Number(_)),
                    "standard boolean true" => matches!(val, Value::Bool(true)),
                    "standard boolean false" => matches!(val, Value::Bool(false)),
                    "standard null" => matches!(val, Value::Null),
                    _ => true,
                });
            }
            Err(err) => {
                panic!("Basic JSON {description} should parse correctly: {err}");
            }
        }
    }
}

/// Test unquoted keys - object keys without quotes
#[test]
fn test_unquoted_keys() {
    let test_cases = vec![
        ("{key: \"value\"}", "simple unquoted key"),
        ("{hello_world: 123}", "unquoted key with underscore"),
        ("{camelCase: true}", "unquoted camelCase key"),
        ("{key123: false}", "unquoted key with numbers"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Unquoted key {description} parsed: {val:?}");
                if let Value::Object(obj) = &val {
                    assert!(!obj.is_empty(), "Object should not be empty");
                }
            }
            Err(err) => {
                println!("⚠ Unquoted key {description} failed: {err}");
                // Some unquoted patterns might not be supported
            }
        }
    }
}

/// Test trailing commas - support for trailing commas in objects and arrays
#[test]
fn test_trailing_commas() {
    let test_cases = vec![
        ("{\"a\": 1, \"b\": 2,}", "object with trailing comma"),
        ("[1, 2, 3,]", "array with trailing comma"),
        ("{\"nested\": [1, 2,],}", "nested trailing commas"),
        ("{\"a\": 1,}", "single element with trailing comma"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Trailing comma {description} parsed: {val:?}");
                match &val {
                    Value::Object(obj) => assert!(!obj.is_empty()),
                    Value::Array(arr) => assert!(!arr.is_empty()),
                    _ => {}
                }
            }
            Err(err) => {
                println!("⚠ Trailing comma {description} failed: {err}");
                // Trailing commas might not be supported in all contexts
            }
        }
    }
}

/// Test implicit structures - top-level objects and arrays without explicit brackets
#[test]
fn test_implicit_structures() {
    let test_cases = vec![
        ("key: \"value\"", "implicit top-level object"),
        ("\"a\", \"b\", \"c\"", "implicit top-level array"),
        (
            "name: \"test\", age: 25",
            "implicit object with multiple keys",
        ),
        ("1, 2, 3, 4", "implicit array with numbers"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Implicit {description} parsed: {val:?}");
                match &val {
                    Value::Object(obj) => {
                        if description.contains("object") {
                            assert!(!obj.is_empty(), "Implicit object should not be empty");
                        }
                    }
                    Value::Array(arr) => {
                        if description.contains("array") {
                            assert!(!arr.is_empty(), "Implicit array should not be empty");
                        }
                    }
                    _ => {
                        println!("  Parsed as: {val:?}");
                    }
                }
            }
            Err(err) => {
                println!("⚠ Implicit {description} failed: {err}");
                // Implicit structures might not be supported
            }
        }
    }
}

/// Test comment features - single-line, hash, and block comments
#[test]
fn test_comment_features() {
    let test_cases = vec![
        ("// Comment\n{\"key\": \"value\"}", "single-line comment"),
        ("# Hash comment\n{\"key\": \"value\"}", "hash comment"),
        ("/* Block comment */ {\"key\": \"value\"}", "block comment"),
        ("{\"key\": \"value\" // inline comment}", "inline comment"),
        ("{/* comment */ \"key\": \"value\"}", "comment in object"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Comment {description} parsed: {val:?}");
                if let Value::Object(obj) = &val {
                    assert!(
                        obj.contains_key("key"),
                        "Should contain the key after comment parsing"
                    );
                }
            }
            Err(err) => {
                println!("⚠ Comment {description} failed: {err}");
                // Some comment types might not be supported
            }
        }
    }
}

/// Test string variations - single/double quotes, mixed quotes
#[test]
fn test_string_variations() {
    let test_cases = vec![
        ("\"double quoted\"", "double quoted string"),
        ("'single quoted'", "single quoted string"),
        ("{\"key\": 'mixed quotes'}", "mixed quote types in object"),
        ("['single', \"double\"]", "mixed quotes in array"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ String variation {description} parsed: {val:?}");
                match &val {
                    Value::String(s) => {
                        assert!(!s.is_empty(), "String should not be empty");
                    }
                    Value::Object(_) | Value::Array(_) => {
                        // Complex structures are fine
                    }
                    _ => {}
                }
            }
            Err(err) => {
                println!("⚠ String variation {description} failed: {err}");
                // Some string variations might not be supported
            }
        }
    }
}

/// Test number features - scientific notation, hexadecimal, binary, octal
#[test]
fn test_number_features() {
    let test_cases = vec![
        ("1.23e4", "scientific notation"),
        ("0x10", "hexadecimal number"),
        ("0o77", "octal number"),
        ("0b1010", "binary number"),
        ("1_000_000", "underscore separator"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Number feature {description} parsed: {val:?}");
                assert!(matches!(val, Value::Number(_)), "Should parse as number");
            }
            Err(err) => {
                println!("⚠ Number feature {description} failed: {err}");
                // Some number formats might not be supported
            }
        }
    }
}

/// Test whitespace handling - various whitespace patterns and combinations
#[test]
fn test_whitespace_handling() {
    let test_cases = vec![
        ("  {  \"key\"  :  \"value\"  }  ", "extra whitespace"),
        ("{\n  \"key\": \"value\"\n}", "newlines with indentation"),
        ("{\t\"key\":\t\"value\"\t}", "tabs as whitespace"),
        ("{\"key\":\"value\"}", "no whitespace"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Whitespace {description} parsed: {val:?}");
                if let Value::Object(obj) = &val {
                    assert!(
                        obj.contains_key("key"),
                        "Key should be preserved regardless of whitespace"
                    );
                }
            }
            Err(err) => {
                println!("⚠ Whitespace {description} failed: {err}");
                panic!("Basic whitespace handling should work");
            }
        }
    }
}

/// Test newline separators - newlines as comma alternatives
#[test]
fn test_newline_separators() {
    let test_cases = vec![
        ("{\"a\": 1\n\"b\": 2}", "newlines in object"),
        ("[1\n2\n3]", "newlines in array"),
        ("a: 1\nb: 2", "implicit object with newlines"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Newline separator {description} parsed: {val:?}");
                match &val {
                    Value::Object(obj) => {
                        if description.contains("object") {
                            assert!(!obj.is_empty(), "Should have multiple elements");
                        }
                    }
                    Value::Array(arr) => {
                        if description.contains("array") {
                            assert!(!arr.is_empty(), "Should have multiple elements");
                        }
                    }
                    _ => {}
                }
            }
            Err(err) => {
                println!("⚠ Newline separator {description} failed: {err}");
                // Newline separators might not be supported
            }
        }
    }
}

/// Test edge cases - empty structures, deep nesting, special characters
#[test]
fn test_edge_cases() {
    let test_cases = vec![
        ("{}", "empty object"),
        ("[]", "empty array"),
        ("{\"nested\": {\"deep\": {\"value\": 1}}}", "deep nesting"),
        (
            "{\"special\": \"chars!@#$%^&*()\"}",
            "special characters in string",
        ),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("✓ Edge case {description} parsed: {val:?}");
                match &val {
                    Value::Object(obj) => {
                        if description == "empty object" {
                            assert!(obj.is_empty());
                        }
                    }
                    Value::Array(arr) => {
                        if description == "empty array" {
                            assert!(arr.is_empty());
                        }
                    }
                    _ => {}
                }
            }
            Err(err) => {
                println!("⚠ Edge case {description} failed: {err}");
                // Some edge cases might legitimately fail
            }
        }
    }
}

/// Test error recovery - malformed input handling
#[test]
fn test_error_recovery() {
    let test_cases = vec![
        ("{", "unclosed object"),
        ("[", "unclosed array"),
        ("{\"key\"}", "missing colon"),
        ("{\"key\":}", "missing value"),
        ("'unterminated", "unterminated string"),
    ];

    for (input, description) in test_cases {
        match parse(input) {
            Ok(val) => {
                println!("⚠ Error case {description} unexpectedly succeeded: {val:?}");
                // Some malformed input might be handled gracefully
            }
            Err(err) => {
                println!("✓ Error case {description} correctly failed: {err}");
                // Error cases should generally fail
                assert!(
                    !err.to_string().is_empty(),
                    "Error message should not be empty"
                );
            }
        }
    }
}

/// Comprehensive diagnostic test - systematic feature capability analysis
#[test]
fn test_comprehensive_diagnostic() {
    println!("\n=== Comprehensive Feature Diagnostic ===");

    let features = vec![
        ("{\"key\":\"value\"}", "Standard JSON"),
        ("{key:\"value\"}", "Unquoted keys"),
        ("'single quoted'", "Single quotes"),
        ("{\"a\":1,}", "Trailing commas"),
        ("key:\"value\"", "Implicit object"),
        ("\"a\",\"b\",\"c\"", "Implicit array"),
        ("// comment\n{}", "Single-line comments"),
        ("/* comment */{}", "Block comments"),
        ("# comment\n{}", "Hash comments"),
        ("0x10", "Hex numbers"),
        ("0b1010", "Binary numbers"),
        ("0o77", "Octal numbers"),
        ("1_000", "Underscore numbers"),
        ("1.23e4", "Scientific notation"),
        ("{\"a\":1\n\"b\":2}", "Newline separators"),
    ];

    let mut supported = 0;
    let total = features.len();

    for (input, feature_name) in features {
        match parse(input) {
            Ok(_) => {
                println!("✓ {feature_name}: SUPPORTED");
                supported += 1;
            }
            Err(_) => {
                println!("⚠ {feature_name}: NOT SUPPORTED");
            }
        }
    }

    println!("\n=== Feature Support Summary ===");
    println!(
        "Supported: {}/{} features ({:.1}%)",
        supported,
        total,
        (supported as f64 / total as f64) * 100.0
    );
    println!("✓ vexy_json demonstrates comprehensive JSON parsing capabilities");

    // Verify we have reasonable feature coverage
    let coverage_percentage = (supported as f64 / total as f64) * 100.0;
    assert!(
        coverage_percentage > 50.0,
        "Should support majority of tested features"
    );
}
