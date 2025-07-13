// this_file: crates/core/tests/streaming_buffered_test.rs

//! Tests for the improved buffered streaming parser

use std::io::Cursor;
use vexy_json_core::streaming::{BufferedStreamingParser, BufferedStreamingConfig, StreamingEvent};

#[test]
fn test_buffered_lexer_strings_with_spaces() {
    let json = r#"{"message": "hello world", "value": 42}"#;
    let reader = Cursor::new(json);
    let mut parser = BufferedStreamingParser::new(reader);
    
    let mut events = Vec::new();
    
    // Manually iterate to see what happens
    loop {
        match parser.next_event() {
            Ok(Some(event)) => {
                println!("Got event: {:?}", event);
                events.push(event);
            }
            Ok(None) => {
                println!("No more events");
                break;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Error getting next event: {:?}", e);
            }
        }
    }
    
    // Debug: print actual events to see what's happening
    println!("Actual events: {:#?}", events);
    println!("Number of events: {}", events.len());
    println!("End of input: {}", parser.is_end_of_input());
    
    // Check that we have proper events
    assert!(events.contains(&StreamingEvent::StartObject));
    assert!(events.contains(&StreamingEvent::ObjectKey("message".to_string())));
    assert!(events.contains(&StreamingEvent::String("hello world".to_string())));
    assert!(events.contains(&StreamingEvent::ObjectKey("value".to_string())));
    assert!(events.contains(&StreamingEvent::Number("42".to_string())));
    assert!(events.contains(&StreamingEvent::EndObject));
}

#[test]
fn test_buffered_lexer_escape_sequences() {
    let json = r#"{"escaped": "hello\nworld", "quote": "say \"hi\""}"#;
    let reader = Cursor::new(json);
    let mut parser = BufferedStreamingParser::new(reader);
    
    let events = parser.collect_events().unwrap();
    
    // Check that we parse strings with escape sequences
    assert!(events.contains(&StreamingEvent::StartObject));
    assert!(events.contains(&StreamingEvent::ObjectKey("escaped".to_string())));
    assert!(events.contains(&StreamingEvent::ObjectKey("quote".to_string())));
    assert!(events.contains(&StreamingEvent::EndObject));
    
    // Count events to ensure proper structure
    let start_objects = events.iter().filter(|e| matches!(e, StreamingEvent::StartObject)).count();
    let end_objects = events.iter().filter(|e| matches!(e, StreamingEvent::EndObject)).count();
    assert_eq!(start_objects, end_objects);
}

#[test]
fn test_buffered_lexer_comments_forgiving_mode() {
    let json = r#"{
        // This is a comment
        "value": 42,
        /* Multi-line 
           comment */
        "name": "test"
    }"#;
    let reader = Cursor::new(json);
    
    let mut config = BufferedStreamingConfig::default();
    config.parser_options.allow_comments = true;
    
    let mut parser = BufferedStreamingParser::with_config(reader, config);
    let events = parser.collect_events().unwrap();
    
    // Should parse successfully with comments ignored
    assert!(events.contains(&StreamingEvent::StartObject));
    assert!(events.contains(&StreamingEvent::ObjectKey("value".to_string())));
    assert!(events.contains(&StreamingEvent::Number("42".to_string())));
    assert!(events.contains(&StreamingEvent::ObjectKey("name".to_string())));
    assert!(events.contains(&StreamingEvent::String("test".to_string())));
    assert!(events.contains(&StreamingEvent::EndObject));
}

#[test]
fn test_buffered_lexer_unquoted_keys() {
    let json = r#"{name: "test", value: 42}"#;
    let reader = Cursor::new(json);
    
    let mut config = BufferedStreamingConfig::default();
    config.parser_options.allow_unquoted_keys = true;
    
    let mut parser = BufferedStreamingParser::with_config(reader, config);
    let events = parser.collect_events().unwrap();
    
    // Should parse unquoted keys successfully
    assert!(events.contains(&StreamingEvent::StartObject));
    assert!(events.contains(&StreamingEvent::ObjectKey("name".to_string())));
    assert!(events.contains(&StreamingEvent::String("test".to_string())));
    assert!(events.contains(&StreamingEvent::ObjectKey("value".to_string())));
    assert!(events.contains(&StreamingEvent::Number("42".to_string())));
    assert!(events.contains(&StreamingEvent::EndObject));
}

#[test]
fn test_buffered_lexer_numbers() {
    let json = r#"[42, -3.14, 1.23e-4, 0, 1000000]"#;
    let reader = Cursor::new(json);
    let mut parser = BufferedStreamingParser::new(reader);
    
    let events = parser.collect_events().unwrap();
    
    assert!(events.contains(&StreamingEvent::StartArray));
    assert!(events.contains(&StreamingEvent::Number("42".to_string())));
    assert!(events.contains(&StreamingEvent::Number("-3.14".to_string())));
    assert!(events.contains(&StreamingEvent::Number("1.23e-4".to_string())));
    assert!(events.contains(&StreamingEvent::Number("0".to_string())));
    assert!(events.contains(&StreamingEvent::Number("1000000".to_string())));
    assert!(events.contains(&StreamingEvent::EndArray));
}

#[test]
fn test_buffered_lexer_nested_structures() {
    let json = r#"{"outer": {"inner": [1, {"deep": true}]}, "list": [1, 2, 3]}"#;
    let reader = Cursor::new(json);
    let mut parser = BufferedStreamingParser::new(reader);
    
    let events = parser.collect_events().unwrap();
    
    // Verify structure integrity
    let start_objects = events.iter().filter(|e| matches!(e, StreamingEvent::StartObject)).count();
    let end_objects = events.iter().filter(|e| matches!(e, StreamingEvent::EndObject)).count();
    let start_arrays = events.iter().filter(|e| matches!(e, StreamingEvent::StartArray)).count();
    let end_arrays = events.iter().filter(|e| matches!(e, StreamingEvent::EndArray)).count();
    
    assert_eq!(start_objects, end_objects);
    assert_eq!(start_arrays, end_arrays);
    assert_eq!(start_objects, 3); // main object + inner object + deep object
    assert_eq!(start_arrays, 2); // inner array + list array
}

#[test]
fn test_buffered_lexer_parse_to_value() {
    let json = r#"{"items": [1, 2, 3], "metadata": {"count": 3, "valid": true}}"#;
    let reader = Cursor::new(json);
    let mut parser = BufferedStreamingParser::new(reader);
    
    let value = parser.parse_to_value().unwrap();
    
    match value {
        vexy_json_core::ast::Value::Object(obj) => {
            assert!(obj.contains_key("items"));
            assert!(obj.contains_key("metadata"));
            
            if let Some(vexy_json_core::ast::Value::Array(arr)) = obj.get("items") {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected items to be an array");
            }
            
            if let Some(vexy_json_core::ast::Value::Object(meta)) = obj.get("metadata") {
                assert!(meta.contains_key("count"));
                assert!(meta.contains_key("valid"));
            } else {
                panic!("Expected metadata to be an object");
            }
        }
        _ => panic!("Expected root to be an object"),
    }
}

#[test] 
fn test_buffered_lexer_empty_input() {
    let json = "";
    let reader = Cursor::new(json);
    let mut parser = BufferedStreamingParser::new(reader);
    
    let events = parser.collect_events().unwrap();
    assert!(events.is_empty());
}

#[test]
fn test_buffered_lexer_single_values() {
    // Test single number
    let reader = Cursor::new("42");
    let mut parser = BufferedStreamingParser::new(reader);
    let events = parser.collect_events().unwrap();
    assert!(events.contains(&StreamingEvent::Number("42".to_string())));
    
    // Test single string
    let reader = Cursor::new("\"hello\"");
    let mut parser = BufferedStreamingParser::new(reader);
    let events = parser.collect_events().unwrap();
    assert!(events.contains(&StreamingEvent::String("hello".to_string())));
    
    // Test single boolean
    let reader = Cursor::new("true");
    let mut parser = BufferedStreamingParser::new(reader);
    let events = parser.collect_events().unwrap();
    assert!(events.contains(&StreamingEvent::Bool(true)));
    
    // Test null
    let reader = Cursor::new("null");
    let mut parser = BufferedStreamingParser::new(reader);
    let events = parser.collect_events().unwrap();
    assert!(events.contains(&StreamingEvent::Null));
}