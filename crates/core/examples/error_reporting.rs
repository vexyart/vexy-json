use vexy_json_core::error::reporter::{full_error_report, plain_error_report, quick_error_report};
use vexy_json_core::parse;

fn main() {
    // Example 1: Missing closing brace
    let json1 = r#"{"name": "Alice", "age": 30"#;
    match parse(json1) {
        Ok(_) => println!("JSON 1 parsed successfully"),
        Err(err) => {
            println!("=== Example 1: Missing closing brace ===");
            println!("\nQuick report:");
            println!("{}", quick_error_report(&err));
            println!("\nFull report:");
            println!("{}", full_error_report(&err, json1));
            println!("\n");
        }
    }

    // Example 2: Trailing comma
    let json2 = r#"{"items": [1, 2, 3,]}"#;
    match parse(json2) {
        Ok(_) => println!("JSON 2 parsed successfully"),
        Err(err) => {
            println!("=== Example 2: Trailing comma ===");
            println!("\nQuick report:");
            println!("{}", quick_error_report(&err));
            println!("\nFull report:");
            println!("{}", full_error_report(&err, json2));
            println!("\n");
        }
    }

    // Example 3: Invalid number format
    let json3 = r#"{"value": 123.45.67}"#;
    match parse(json3) {
        Ok(_) => println!("JSON 3 parsed successfully"),
        Err(err) => {
            println!("=== Example 3: Invalid number format ===");
            println!("\nQuick report:");
            println!("{}", quick_error_report(&err));
            println!("\nFull report:");
            println!("{}", full_error_report(&err, json3));
            println!("\n");
        }
    }

    // Example 4: Unquoted key
    let json4 = r#"{name: "Bob", age: 25}"#;
    match parse(json4) {
        Ok(_) => println!("JSON 4 parsed successfully"),
        Err(err) => {
            println!("=== Example 4: Unquoted key ===");
            println!("\nQuick report:");
            println!("{}", quick_error_report(&err));
            println!("\nFull report:");
            println!("{}", full_error_report(&err, json4));
            println!("\n");
        }
    }

    // Example 5: Plain text report (no colors)
    let json5 = r#"{"incomplete": "string"#;
    match parse(json5) {
        Ok(_) => println!("JSON 5 parsed successfully"),
        Err(err) => {
            println!("=== Example 5: Plain text report ===");
            println!("{}", plain_error_report(&err, json5));
            println!("\n");
        }
    }
}
