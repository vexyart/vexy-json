// this_file: examples/recursive_parser.rs

//! Example demonstrating the recursive descent parser

use vexy_json_core::ast::{Number, Value};
use vexy_json_core::parser::recursive::parse_recursive;
use vexy_json_core::parser::ParserOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧮 Recursive Descent Parser Demo");
    println!("==================================");

    // Test basic parsing
    println!("\n1. Basic Values");
    println!("---------------");

    let result = parse_recursive("null", ParserOptions::default())?;
    println!("null -> {:?}", result);
    assert_eq!(result, Value::Null);

    let result = parse_recursive("true", ParserOptions::default())?;
    println!("true -> {:?}", result);
    assert_eq!(result, Value::Bool(true));

    let result = parse_recursive("42", ParserOptions::default())?;
    println!("42 -> {:?}", result);
    assert_eq!(result, Value::Number(Number::Integer(42)));

    let result = parse_recursive(r#""hello""#, ParserOptions::default())?;
    println!(r#""hello" -> {:?}"#, result);
    assert_eq!(result, Value::String("hello".to_string()));

    // Test collections
    println!("\n2. Collections");
    println!("--------------");

    let result = parse_recursive("[1, 2, 3]", ParserOptions::default())?;
    println!("[1, 2, 3] -> {:?}", result);

    let result = parse_recursive(r#"{"key": "value"}"#, ParserOptions::default())?;
    println!(r#"{{"key": "value"}} -> {:?}"#, result);

    // Test nested structures
    println!("\n3. Nested Structures");
    println!("--------------------");

    let nested_json =
        r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}], "total": 2}"#;

    let result = parse_recursive(nested_json, ParserOptions::default())?;
    println!("Nested JSON parsed successfully!");
    if let Value::Object(obj) = &result {
        if let Some(Value::Array(users)) = obj.get("users") {
            println!("Users count: {}", users.len());
        }
    }

    // Test forgiving features
    println!("\n4. Forgiving Features");
    println!("---------------------");

    let options = ParserOptions::default();

    // Comments
    let _result = parse_recursive(
        r#"{"key": "value", /* comment */ "num": 42}"#,
        options.clone(),
    )?;
    println!("JSON with comments parsed successfully!");

    // Trailing commas
    let _result = parse_recursive(r#"{"key": "value", "num": 42,}"#, options.clone())?;
    println!("JSON with trailing comma parsed successfully!");

    // Unquoted keys
    let _result = parse_recursive(r#"{key: "value", num: 42}"#, options.clone())?;
    println!("JSON with unquoted keys parsed successfully!");

    // Test error handling
    println!("\n5. Error Handling");
    println!("-----------------");

    match parse_recursive(r#"{"key": "value"#, ParserOptions::default()) {
        Ok(_) => println!("Should have failed!"),
        Err(e) => println!("Correctly caught error: {}", e),
    }

    println!("\n🎉 All tests passed! The recursive descent parser is working correctly.");

    Ok(())
}
