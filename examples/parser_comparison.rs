// this_file: examples/parser_comparison.rs

//! Example comparing different parser implementations

use std::time::Instant;
use vexy_json_core::ast::{Number, Value};
use vexy_json_core::{
    parse_iterative, parse_optimized_v2_with_options, parse_optimized_with_options,
    parse_recursive, parse_with_options, ParserOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Vexy JSON Parser Comparison Demo");
    println!("===============================");

    let options = ParserOptions::default();

    // Test data
    let test_cases = vec![
        ("Simple Object", r#"{"name": "John", "age": 30}"#),
        ("Array", r#"[1, 2, 3, 4, 5]"#),
        (
            "Nested",
            r#"{"user": {"profile": {"name": "Alice", "settings": {"theme": "dark"}}}}"#,
        ),
        (
            "With Comments",
            r#"{"key": "value", /* comment */ "number": 42}"#,
        ),
        ("Trailing Commas", r#"{"key": "value", "number": 42,}"#),
        ("Unquoted Keys", r#"{key: "value", number: 42}"#),
    ];

    println!("\n📊 Performance Comparison");
    println!("=========================");

    for (name, json) in &test_cases {
        println!("\n🔍 Testing: {name}");
        println!("JSON: {json}");

        // Test all parsers
        let parsers: Vec<(
            &str,
            fn(&str, ParserOptions) -> Result<Value, vexy_json_core::Error>,
        )> = vec![
            ("Original", parse_with_options),
            ("Optimized", parse_optimized_with_options),
            ("Optimized V2", parse_optimized_v2_with_options),
            ("Recursive", parse_recursive),
            ("Iterative", parse_iterative),
        ];

        for (parser_name, parser_fn) in parsers {
            let start = Instant::now();

            // Run parser multiple times for timing
            let mut result = None;
            for _ in 0..1000 {
                result = Some(parser_fn(json, options.clone())?);
            }

            let duration = start.elapsed();
            println!(
                "  {} parser: {:?} ({} iterations)",
                parser_name, duration, 1000
            );

            // Verify result (just once)
            if let Some(value) = result {
                match value {
                    Value::Object(obj) => println!("    → Object with {} keys", obj.len()),
                    Value::Array(arr) => println!("    → Array with {} elements", arr.len()),
                    Value::String(s) => println!("    → String: \"{s}\""),
                    Value::Number(Number::Integer(i)) => println!("    → Integer: {i}"),
                    Value::Number(Number::Float(f)) => println!("    → Float: {f}"),
                    Value::Bool(b) => println!("    → Boolean: {b}"),
                    Value::Null => println!("    → Null"),
                }
            }
        }
    }

    println!("\n🏗️ Deep Nesting Test");
    println!("====================");

    // Test deep nesting where iterative parser should excel
    let depths = vec![10, 50, 100, 200];

    for depth in depths {
        println!("\n🔢 Testing depth: {depth}");

        // Generate deep nested object
        let mut deep_json = String::new();
        for _ in 0..depth {
            deep_json.push_str(r#"{"nested": "#);
        }
        deep_json.push_str("\"value\"");
        for _ in 0..depth {
            deep_json.push('}');
        }

        println!("JSON length: {} characters", deep_json.len());

        // Test recursive parser (may fail on deep nesting)
        let start = Instant::now();
        match parse_recursive(&deep_json, options.clone()) {
            Ok(_) => {
                let duration = start.elapsed();
                println!("  Recursive parser: {duration:?} ✅");
            }
            Err(e) => {
                println!("  Recursive parser: Failed - {e} ❌");
            }
        }

        // Test iterative parser (should handle deep nesting better)
        let start = Instant::now();
        match parse_iterative(&deep_json, options.clone()) {
            Ok(_) => {
                let duration = start.elapsed();
                println!("  Iterative parser: {duration:?} ✅");
            }
            Err(e) => {
                println!("  Iterative parser: Failed - {e} ❌");
            }
        }
    }

    println!("\n🎯 Error Handling Test");
    println!("======================");

    let invalid_cases = vec![
        ("Missing Quote", r#"{"key": "value, "number": 42}"#),
        ("Missing Comma", r#"{"key": "value" "number": 42}"#),
        ("Unclosed Object", r#"{"key": "value", "number": 42"#),
        ("Unclosed Array", r#"[1, 2, 3, 4, 5"#),
    ];

    for (error_name, invalid_json) in invalid_cases {
        println!("\n⚠️ Testing: {error_name}");
        println!("JSON: {invalid_json}");

        // Test error handling across parsers
        let parsers: Vec<(
            &str,
            fn(&str, ParserOptions) -> Result<Value, vexy_json_core::Error>,
        )> = vec![
            ("Recursive", parse_recursive),
            ("Iterative", parse_iterative),
        ];

        for (parser_name, parser_fn) in parsers {
            match parser_fn(invalid_json, options.clone()) {
                Ok(_) => {
                    println!("  {parser_name} parser: Unexpectedly succeeded! 🤔");
                }
                Err(e) => {
                    println!("  {parser_name} parser: Correctly failed - {e} ✅");
                }
            }
        }
    }

    println!("\n🚀 Memory Usage Test");
    println!("===================");

    // Test with a large JSON structure
    let mut large_json = String::from("{");
    for i in 0..1000 {
        if i > 0 {
            large_json.push(',');
        }
        large_json.push_str(&format!(r#""key{i}": "value{i}""#));
    }
    large_json.push('}');

    println!("Testing large JSON with 1000 key-value pairs");
    println!("JSON size: {} bytes", large_json.len());

    let parsers: Vec<(
        &str,
        fn(&str, ParserOptions) -> Result<Value, vexy_json_core::Error>,
    )> = vec![
        ("Recursive", parse_recursive),
        ("Iterative", parse_iterative),
    ];

    for (parser_name, parser_fn) in parsers {
        let start = Instant::now();
        match parser_fn(&large_json, options.clone()) {
            Ok(value) => {
                let duration = start.elapsed();
                if let Value::Object(obj) = value {
                    println!(
                        "  {} parser: {:?} - {} keys ✅",
                        parser_name,
                        duration,
                        obj.len()
                    );
                } else {
                    println!(
                        "  {parser_name} parser: {duration:?} - unexpected type ⚠️"
                    );
                }
            }
            Err(e) => {
                println!("  {parser_name} parser: Failed - {e} ❌");
            }
        }
    }

    println!("\n🎉 Parser Comparison Complete!");
    println!("===================================");
    println!("Key Takeaways:");
    println!("• Iterative parser handles deep nesting better (no stack overflow)");
    println!("• Recursive parser may be slightly faster for shallow structures");
    println!("• Both parsers support all Vexy JSON forgiving features");
    println!("• Error handling is consistent across implementations");
    println!("• Choose based on your specific use case and data characteristics");

    Ok(())
}
