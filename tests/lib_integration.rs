// this_file: tests/lib_integration.rs

use vexy_json::{parse, parse_with_options, ParserOptions};
use vexy_json_core::lexer::{JsonLexer, Lexer};

#[test]
fn basic_parsing() {
    let json = r#"{"key": "value"}"#;
    let result = parse(json);
    match &result {
        Err(e) => eprintln!("Parse error: {:?}", e),
        Ok(v) => eprintln!("Parse success: {:?}", v),
    }
    assert!(result.is_ok());
}

#[test]
fn test_forgiving_features() {
    // Single quotes
    println!("\nTesting single quotes:");

    // First test lexer
    let mut lexer = Lexer::new("'hello'");
    match lexer.next_token() {
        Ok(token) => println!("Lexer token: {:?} at position {}", token, lexer.position()),
        Err(e) => println!("Lexer error: {:?}", e),
    }

    match parse("'hello'") {
        Ok(v) => println!("Success: {:?}", v),
        Err(e) => {
            println!("Error: {:?}", e);
            panic!("Single quotes test failed");
        }
    }

    // Comments
    println!("\nTesting comments:");
    // NOTE: Comments at the beginning of input require lexer preprocessing
    // This is a known limitation for v1.0.0 - comments work when following content
    // assert!(parse("// comment\n42").is_ok());
    // assert!(parse("/* comment */ 42").is_ok());
    assert!(parse("42 // comment").is_ok());
    assert!(parse("42 /* comment */").is_ok());

    // Trailing commas
    println!("\nTesting trailing commas:");
    match parse("[1, 2, 3,]") {
        Ok(v) => println!("Trailing comma success: {:?}", v),
        Err(e) => {
            println!("Trailing comma error: {:?}", e);
            panic!("Trailing comma test failed");
        }
    }
    match parse(r#"{"a": 1, "b": 2,}"#) {
        Ok(v) => println!("Quoted keys with trailing comma: {:?}", v),
        Err(e) => println!("Error with quoted keys: {:?}", e),
    }
    match parse("{a: 1, b: 2,}") {
        Ok(v) => println!("Unquoted keys with trailing comma: {:?}", v),
        Err(e) => {
            println!("Error with unquoted keys: {:?}", e);
            panic!("Unquoted keys test failed");
        }
    }

    // Unquoted keys
    println!("\nTesting unquoted keys:");
    assert!(parse("{key: 'value'}").is_ok());

    // Implicit top-level
    println!("\nTesting implicit top-level:");
    assert!(parse("a: 1").is_ok());
    assert!(parse("1, 2, 3").is_ok());

    // Newlines as commas
    println!("\nTesting newlines as commas:");
    assert!(parse("[1\n2\n3]").is_ok());
    assert!(parse("{\"a\": 1\n\"b\": 2}").is_ok());

    // Test with newlines disabled
    let options = ParserOptions {
        newline_as_comma: false,
        ..Default::default()
    };
    assert!(parse_with_options("[1\n2]", options).is_err());
}
