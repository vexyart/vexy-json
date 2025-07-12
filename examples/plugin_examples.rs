// this_file: examples/plugin_examples.rs

//! Examples demonstrating Vexy JSON plugin usage
//!
//! This example shows how to use the built-in plugins to extend
//! Vexy JSON's parsing capabilities.

use vexy_json::{parse, parse_with_options, ParserOptions};
use vexy_json_core::plugin::plugins::{
    CommentPreservationPlugin, CustomNumberFormatPlugin, DateTimePlugin, SchemaValidationPlugin,
};
use vexy_json_core::plugin::ParserPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Vexy JSON Plugin Examples");
    println!("========================");

    // Example 1: Schema Validation Plugin
    println!("\n1. Schema Validation Plugin");
    println!("---------------------------");

    let json_data = r#"{
        "name": "John Doe",
        "age": 30,
        "email": "john@example.com",
        "active": true
    }"#;

    let schema = parse(
        r#"{
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "number", "minimum": 0},
            "email": {"type": "string", "format": "email"},
            "active": {"type": "boolean"}
        },
        "required": ["name", "age"]
    }"#,
    )?;

    let validator = SchemaValidationPlugin::new(schema);

    let parsed = parse(json_data)?;
    match validator.validate(&parsed, "$") {
        Ok(()) => println!("✅ Schema validation passed"),
        Err(e) => println!("❌ Schema validation failed: {e}"),
    }

    // Example 2: Date/Time Parsing Plugin
    println!("\n2. Date/Time Parsing Plugin");
    println!("---------------------------");

    let datetime_json = r#"{
        "created_at": "2023-12-25T10:30:00Z",
        "updated_at": "2023-12-25",
        "birthday": "1990-01-15T08:00:00-05:00"
    }"#;

    let mut datetime_plugin = DateTimePlugin::new();
    let mut parsed_datetime = parse(datetime_json)?;
    datetime_plugin.transform_value(&mut parsed_datetime, "$")?;

    println!("Original JSON: {datetime_json}");
    println!("Transformed with datetime plugin:");
    println!("{parsed_datetime:#}");

    // Example 3: Custom Number Format Plugin
    println!("\n3. Custom Number Format Plugin");
    println!("------------------------------");

    let mut number_plugin = CustomNumberFormatPlugin::new();

    let hex_result = number_plugin.on_number("0xFF", "$")?;
    println!("Hex 0xFF parsed as: {hex_result:?}");

    let binary_result = number_plugin.on_number("0b1010", "$")?;
    println!("Binary 0b1010 parsed as: {binary_result:?}");

    let underscore_result = number_plugin.on_number("1_000_000", "$")?;
    println!("Number 1_000_000 parsed as: {underscore_result:?}");

    let infinity_result = number_plugin.on_number("Infinity", "$")?;
    println!("Infinity parsed as: {infinity_result:?}");

    // Example 4: Comment Preservation Plugin
    println!("\n4. Comment Preservation Plugin");
    println!("------------------------------");

    let _commented_json = r#"{
        "name": "Jane Doe",
        "age": 25,
        "email": "jane@example.com",
        "phone": "+1-555-0123"
    }"#;

    let mut comment_plugin = CommentPreservationPlugin::new();

    // In a real scenario, you would integrate this with the parser
    // For demonstration, we'll manually add some comments
    comment_plugin.add_comment("User information".to_string(), "$.name", false);
    comment_plugin.add_comment("Contact details".to_string(), "$.email", true);
    comment_plugin.add_comment("Primary phone".to_string(), "$.phone", false);

    let comments = comment_plugin.comments_to_value();
    println!("Preserved comments: {comments:#}");

    // Example 5: Combining Multiple Plugins
    println!("\n5. Combining Multiple Plugins");
    println!("-----------------------------");

    let complex_json = r#"{
        "app": {
            "name": "MyApp",
            "version": "1.0.0",
            "debug": true
        },
        "database": {
            "host": "localhost",
            "port": 5555,
            "timeout": 30000,
            "created_at": "2023-12-01T12:00:00Z"
        },
        "features": {
            "max_users": 1000,
            "cache_size": 16777216
        }
    }"#;

    println!("Complex JSON with multiple formats:");
    println!("{complex_json}");

    // Parse with standard options
    let options = ParserOptions::default();

    let mut complex_parsed = parse_with_options(complex_json, options)?;

    // Apply datetime transformation
    let mut datetime_plugin = DateTimePlugin::new();
    datetime_plugin.transform_value(&mut complex_parsed, "$")?;

    println!("\nParsed and transformed:");
    println!("{complex_parsed:#}");

    // Example 6: Plugin Performance Comparison
    println!("\n6. Plugin Performance Comparison");
    println!("--------------------------------");

    let test_json = r#"{"timestamp": "2023-12-01T12:00:00Z", "value": 12345}"#;

    let start = std::time::Instant::now();
    let _without_plugins = parse(test_json)?;
    let without_plugins_time = start.elapsed();

    let start = std::time::Instant::now();
    let mut with_plugins = parse(test_json)?;
    let mut datetime_plugin = DateTimePlugin::new();
    datetime_plugin.transform_value(&mut with_plugins, "$")?;
    let with_plugins_time = start.elapsed();

    println!("Without plugins: {without_plugins_time:?}");
    println!("With plugins: {with_plugins_time:?}");
    println!(
        "Overhead: {:?}",
        with_plugins_time.saturating_sub(without_plugins_time)
    );

    println!("\n🎉 All plugin examples completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation_example() {
        let json_data = r#"{"name": "John", "age": 30}"#;
        let schema = parse(
            r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number", "minimum": 0}
            },
            "required": ["name", "age"]
        }"#,
        )
        .unwrap();

        let validator = SchemaValidationPlugin::new(schema);

        let parsed = parse(json_data).unwrap();
        validator.validate(&parsed, "$").unwrap();
    }

    #[test]
    fn test_datetime_plugin_example() {
        let mut plugin = DateTimePlugin::new();
        let mut value = parse(r#"{"date": "2023-12-25T10:30:00Z"}"#).unwrap();

        plugin.transform_value(&mut value, "$").unwrap();

        // Check that the date was transformed
        if let Some(date_obj) = value.as_object().and_then(|o| o.get("date")) {
            assert!(date_obj.is_object());
            assert!(date_obj.as_object().unwrap().contains_key("year"));
        }
    }

    #[test]
    fn test_custom_number_plugin_example() {
        let mut plugin = CustomNumberFormatPlugin::new();

        let hex_result = plugin.on_number("0xFF", "$").unwrap();
        assert!(hex_result.is_number());

        let binary_result = plugin.on_number("0b1010", "$").unwrap();
        assert!(binary_result.is_number());

        let underscore_result = plugin.on_number("1_000_000", "$").unwrap();
        assert!(underscore_result.is_number());
    }

    #[test]
    fn test_comment_preservation_example() {
        let mut plugin = CommentPreservationPlugin::new();

        plugin.add_comment("Test comment".to_string(), "$.test", false);

        let comments = plugin.comments_to_value();
        assert!(comments.is_array());
        assert_eq!(comments.as_array().unwrap().len(), 1);
    }
}
