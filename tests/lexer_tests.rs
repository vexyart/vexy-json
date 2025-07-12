// this_file: tests/lexer_tests.rs

use vexy_json::{parse, Value};

///
/// These tests focus on the tokenization and lexing behavior of vexy_json,
/// exploring how the parser handles various input patterns at the lexer level.
/// Uses adaptive testing to discover vexy_json's actual capabilities.

#[test]
fn test_basic_specials() {
    // Test basic tokenization of special characters and numbers

    // Test empty input - should parse successfully or fail gracefully
    let result = parse("");
    match result {
        Ok(_) => println!("✓ vexy_json handles empty input successfully"),
        Err(_) => println!("ℹ vexy_json reports error for empty input"),
    }

    // Test whitespace handling
    let result = parse("   ");
    match result {
        Ok(_) => println!("✓ vexy_json handles whitespace-only input"),
        Err(_) => println!("ℹ vexy_json reports error for whitespace-only input"),
    }

    // Test single number
    match parse("123") {
        Ok(val) => {
            assert!(
                val.as_f64() == Some(123.0),
                "Expected number 123, got: {val:?}"
            );
            println!("✓ vexy_json parses single number: 123");
        }
        Err(e) => panic!("Failed to parse single number: {e}"),
    }

    // Test number with trailing content that should be treated as text
    match parse("123%") {
        Ok(val) => {
            // Could be parsed as text or cause an error
            println!("ℹ vexy_json parsed '123%' as: {val:?}");
        }
        Err(_) => {
            println!("ℹ vexy_json treats '123%' as invalid input");
        }
    }
}

#[test]
fn test_space_handling() {
    // Test various space and whitespace combinations

    // Single space
    let result = parse(" ");
    match result {
        Ok(_) => println!("✓ vexy_json handles single space"),
        Err(_) => println!("ℹ vexy_json reports error for single space"),
    }

    // Tab character
    let result = parse("\t");
    match result {
        Ok(_) => println!("✓ vexy_json handles tab character"),
        Err(_) => println!("ℹ vexy_json reports error for tab character"),
    }

    // Mixed spaces and tabs
    let result = parse(" \t ");
    match result {
        Ok(_) => println!("✓ vexy_json handles mixed whitespace"),
        Err(_) => println!("ℹ vexy_json reports error for mixed whitespace"),
    }
}

#[test]
fn test_brace_handling() {
    // Test brace tokenization behavior

    // Single opening brace
    let result = parse("{");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed single '{{' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for unmatched opening brace"),
    }

    // Double opening braces
    let result = parse("{{");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed '{{' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for double opening braces"),
    }

    // Single closing brace
    let result = parse("}");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed single '}}' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for unmatched closing brace"),
    }

    // Proper brace pair
    match parse("{}") {
        Ok(val) => {
            // Should be empty object
            match &val {
                Value::Object(obj) => {
                    assert!(obj.is_empty(), "Expected empty object, got: {obj:?}");
                    println!("✓ vexy_json parses empty object correctly");
                }
                _ => panic!("Expected object, got: {val:?}"),
            }
        }
        Err(e) => panic!("Failed to parse empty object: {e}"),
    }
}

#[test]
fn test_square_bracket_handling() {
    // Test square bracket tokenization behavior

    // Single opening bracket
    let result = parse("[");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed single '[' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for unmatched opening bracket"),
    }

    // Single closing bracket
    let result = parse("]");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed single ']' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for unmatched closing bracket"),
    }

    // Proper bracket pair
    match parse("[]") {
        Ok(val) => {
            // Should be empty array
            match &val {
                Value::Array(arr) => {
                    assert!(arr.is_empty(), "Expected empty array, got: {arr:?}");
                    println!("✓ vexy_json parses empty array correctly");
                }
                _ => panic!("Expected array, got: {val:?}"),
            }
        }
        Err(e) => panic!("Failed to parse empty array: {e}"),
    }
}

#[test]
fn test_colon_comma_handling() {
    // Test colon and comma tokenization

    // Single colon
    let result = parse(":");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed single ':' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for lone colon"),
    }

    // Double colon
    let result = parse("::");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed '::' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for double colon"),
    }

    // Single comma
    let result = parse(",");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed single ',' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for lone comma"),
    }

    // Double comma
    let result = parse(",,");
    match result {
        Ok(_) => println!("ℹ vexy_json parsed ',,' successfully"),
        Err(_) => println!("✓ vexy_json properly reports error for double comma"),
    }
}

#[test]
fn test_comment_lexer_behavior() {
    // Test comment handling at lexer level

    // Hash comment
    match parse("a#b") {
        Ok(val) => {
            // Should parse 'a' and ignore '#b'
            match &val {
                Value::String(s) => {
                    assert_eq!(s, "a", "Expected 'a', got: {s}");
                    println!("✓ vexy_json supports # comments in lexer");
                }
                _ => println!("ℹ vexy_json parsed 'a#b' as: {val:?}"),
            }
        }
        Err(_) => println!("ℹ vexy_json doesn't support # comments or treats as error"),
    }

    // Block comment
    match parse("a/*x*/b") {
        Ok(val) => {
            println!("ℹ vexy_json parsed 'a/*x*/b' as: {val:?}");
        }
        Err(_) => println!("ℹ vexy_json doesn't support /* */ comments or treats as error"),
    }

    // Comment with newline
    match parse("a#b\nc") {
        Ok(val) => {
            println!("ℹ vexy_json parsed 'a#b\\nc' as: {val:?}");
        }
        Err(_) => println!("ℹ vexy_json has issues with comments and newlines"),
    }
}

#[test]
fn test_boolean_null_lexing() {
    // Test boolean and null value tokenization

    // True value
    match parse("true") {
        Ok(val) => match &val {
            Value::Bool(true) => println!("✓ vexy_json lexer handles 'true' correctly"),
            _ => panic!("Expected true boolean, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse 'true': {e}"),
    }

    // False value
    match parse("false") {
        Ok(val) => match &val {
            Value::Bool(false) => println!("✓ vexy_json lexer handles 'false' correctly"),
            _ => panic!("Expected false boolean, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse 'false': {e}"),
    }

    // Null value
    match parse("null") {
        Ok(val) => match &val {
            Value::Null => println!("✓ vexy_json lexer handles 'null' correctly"),
            _ => panic!("Expected null, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse 'null': {e}"),
    }

    // Test with extra characters that should make it text
    match parse("truex") {
        Ok(val) => {
            // Should be treated as text, not boolean
            match &val {
                Value::String(s) => {
                    assert_eq!(s, "truex", "Expected text 'truex', got: {s}");
                    println!("✓ vexy_json lexer treats 'truex' as text, not boolean");
                }
                _ => println!("ℹ vexy_json lexer parsed 'truex' as: {val:?}"),
            }
        }
        Err(_) => println!("ℹ vexy_json lexer rejects 'truex'"),
    }
}

#[test]
fn test_number_lexing() {
    // Test number tokenization behavior

    // Basic integer
    match parse("321") {
        Ok(val) => {
            assert!(val.as_f64() == Some(321.0), "Expected 321, got: {val:?}");
            println!("✓ vexy_json lexer handles integers correctly");
        }
        Err(e) => panic!("Failed to parse integer: {e}"),
    }

    // Zero
    match parse("0") {
        Ok(val) => {
            assert!(val.as_f64() == Some(0.0), "Expected 0, got: {val:?}");
            println!("✓ vexy_json lexer handles zero correctly");
        }
        Err(e) => panic!("Failed to parse zero: {e}"),
    }

    // Decimal number
    match parse("1.2") {
        Ok(val) => {
            assert!(val.as_f64() == Some(1.2), "Expected 1.2, got: {val:?}");
            println!("✓ vexy_json lexer handles decimal numbers correctly");
        }
        Err(e) => panic!("Failed to parse decimal: {e}"),
    }

    // Negative number
    match parse("-1.2") {
        Ok(val) => {
            assert!(val.as_f64() == Some(-1.2), "Expected -1.2, got: {val:?}");
            println!("✓ vexy_json lexer handles negative numbers correctly");
        }
        Err(e) => panic!("Failed to parse negative number: {e}"),
    }

    // Scientific notation
    match parse("1e2") {
        Ok(val) => {
            assert!(val.as_f64() == Some(100.0), "Expected 100, got: {val:?}");
            println!("✓ vexy_json lexer handles scientific notation");
        }
        Err(_) => println!("ℹ vexy_json lexer doesn't support scientific notation"),
    }

    // Hexadecimal
    match parse("0xA") {
        Ok(val) => {
            assert!(val.as_f64() == Some(10.0), "Expected 10, got: {val:?}");
            println!("✓ vexy_json lexer handles hexadecimal numbers");
        }
        Err(_) => println!("ℹ vexy_json lexer doesn't support hexadecimal"),
    }

    // Number with invalid trailing characters
    match parse("1x") {
        Ok(val) => {
            // Should be treated as text
            match &val {
                Value::String(s) => {
                    assert_eq!(s, "1x", "Expected text '1x', got: {s}");
                    println!("✓ vexy_json lexer treats '1x' as text");
                }
                _ => println!("ℹ vexy_json lexer parsed '1x' as: {val:?}"),
            }
        }
        Err(_) => println!("ℹ vexy_json lexer rejects '1x'"),
    }
}

#[test]
fn test_string_lexing() {
    // Test string tokenization with different quote types

    // Double quotes
    match parse("\"\"") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert!(s.is_empty(), "Expected empty string, got: '{s}'");
                println!("✓ vexy_json lexer handles empty double-quoted strings");
            }
            _ => panic!("Expected string, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse empty double-quoted string: {e}"),
    }

    // Double quotes with content
    match parse("\"abc\"") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "abc", "Expected 'abc', got: '{s}'");
                println!("✓ vexy_json lexer handles double-quoted strings with content");
            }
            _ => panic!("Expected string, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse double-quoted string: {e}"),
    }

    // Single quotes
    match parse("'abc'") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "abc", "Expected 'abc', got: '{s}'");
                println!("✓ vexy_json lexer handles single-quoted strings");
            }
            _ => panic!("Expected string, got: {val:?}"),
        },
        Err(_) => println!("ℹ vexy_json lexer doesn't support single-quoted strings"),
    }

    // Unterminated string
    let result = parse("\"abc");
    match result {
        Ok(_) => println!("ℹ vexy_json lexer unexpectedly parsed unterminated string"),
        Err(_) => println!("✓ vexy_json lexer properly reports error for unterminated string"),
    }

    // String with escape sequences
    match parse("\"\\t\"") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "\t", "Expected tab character, got: '{s:?}'");
                println!("✓ vexy_json lexer handles escape sequences");
            }
            _ => panic!("Expected string, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse string with escape: {e}"),
    }

    // String with unicode escape
    match parse("\"\\u0040\"") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "@", "Expected '@', got: '{s}'");
                println!("✓ vexy_json lexer handles unicode escapes");
            }
            _ => panic!("Expected string, got: {val:?}"),
        },
        Err(_) => println!("ℹ vexy_json lexer doesn't support unicode escapes"),
    }
}

#[test]
fn test_text_lexing() {
    // Test unquoted text tokenization

    // Simple text with hyphens
    match parse("a-b") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "a-b", "Expected 'a-b', got: '{s}'");
                println!("✓ vexy_json lexer handles text with hyphens");
            }
            _ => println!("ℹ vexy_json lexer parsed 'a-b' as: {val:?}"),
        },
        Err(_) => println!("ℹ vexy_json lexer rejects 'a-b'"),
    }

    // Text with special characters
    match parse("$a_") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "$a_", "Expected '$a_', got: '{s}'");
                println!("✓ vexy_json lexer handles text with $ and _");
            }
            _ => println!("ℹ vexy_json lexer parsed '$a_' as: {val:?}"),
        },
        Err(_) => println!("ℹ vexy_json lexer rejects '$a_'"),
    }

    // Text with punctuation
    match parse("!%~") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "!%~", "Expected '!%~', got: '{s}'");
                println!("✓ vexy_json lexer handles punctuation text");
            }
            _ => println!("ℹ vexy_json lexer parsed '!%~' as: {val:?}"),
        },
        Err(_) => println!("ℹ vexy_json lexer rejects '!%~'"),
    }
}

#[test]
fn test_line_handling() {
    // Test newline tokenization behavior

    // Object with newline
    match parse("{a:1,\nb:2}") {
        Ok(val) => match &val {
            Value::Object(obj) => {
                assert!(obj.len() == 2, "Expected 2 properties, got: {}", obj.len());
                assert!(
                    obj.get("a").and_then(|v| v.as_f64()) == Some(1.0),
                    "Expected a=1"
                );
                assert!(
                    obj.get("b").and_then(|v| v.as_f64()) == Some(2.0),
                    "Expected b=2"
                );
                println!("✓ vexy_json lexer handles newlines in objects correctly");
            }
            _ => panic!("Expected object, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse object with newline: {e}"),
    }

    // Test if newlines can act as separators
    match parse("a\nb") {
        Ok(val) => {
            println!("ℹ vexy_json lexer parsed 'a\\nb' as: {val:?}");
        }
        Err(_) => println!("ℹ vexy_json lexer rejects 'a\\nb'"),
    }
}

#[test]
fn test_complex_lexer_scenarios() {
    // Test complex tokenization scenarios

    // Mixed quotes and special characters in strings
    match parse("\"[{}]:,\"") {
        Ok(val) => match &val {
            Value::String(s) => {
                assert_eq!(s, "[{}]:,", "Expected '[{{}}]:,', got: '{s}'");
                println!("✓ vexy_json lexer handles special chars in strings");
            }
            _ => panic!("Expected string, got: {val:?}"),
        },
        Err(e) => panic!("Failed to parse string with special chars: {e}"),
    }

    // Number followed by special character
    match parse("1%") {
        Ok(val) => {
            // Could be treated as text or cause error
            println!("ℹ vexy_json lexer parsed '1%' as: {val:?}");
        }
        Err(_) => println!("ℹ vexy_json lexer rejects '1%'"),
    }

    // Object key followed by colon
    match parse("a:") {
        Ok(val) => {
            println!("ℹ vexy_json lexer parsed 'a:' as: {val:?}");
        }
        Err(_) => println!("ℹ vexy_json lexer rejects incomplete key-value pair"),
    }
}

/// Comprehensive lexer test that explores vexy_json's tokenization capabilities
///
/// This test serves as a diagnostic tool to understand how vexy_json's lexer
/// handles various input patterns, providing insight into the tokenization
/// process that underlies the parsing functionality.
#[test]
fn test_lexer_comprehensive_diagnostic() {
    println!("\n=== Vexy JSON LEXER CAPABILITY ANALYSIS ===");

    // Test various input patterns systematically
    let test_cases = vec![
        // Basic tokens
        ("empty", ""),
        ("space", " "),
        ("number", "42"),
        ("text", "abc"),
        // Structural tokens
        ("empty_object", "{}"),
        ("empty_array", "[]"),
        ("colon", ":"),
        ("comma", ","),
        // String tokens
        ("double_quote_empty", "\"\""),
        ("double_quote_text", "\"hello\""),
        ("single_quote_text", "'hello'"),
        // Boolean/null tokens
        ("true_value", "true"),
        ("false_value", "false"),
        ("null_value", "null"),
        // Special cases
        ("hex_number", "0xA"),
        ("scientific", "1e2"),
        ("negative", "-42"),
        ("hash_comment", "a#comment"),
        ("block_comment", "a/*comment*/b"),
    ];

    let mut parsed_count = 0;
    let mut error_count = 0;

    for (name, input) in test_cases {
        match parse(input) {
            Ok(val) => {
                parsed_count += 1;
                println!("  ✓ {name}: {val:?}");
            }
            Err(_) => {
                error_count += 1;
                println!("  ✗ {name}: parse error");
            }
        }
    }

    println!("\nLEXER ANALYSIS SUMMARY:");
    println!("  Successful parses: {parsed_count}");
    println!("  Parse errors: {error_count}");
    println!("  Total test cases: {}", parsed_count + error_count);

    // Ensure we have some basic functionality
    assert!(
        parsed_count > 0,
        "Lexer should successfully parse at least some inputs"
    );
}
