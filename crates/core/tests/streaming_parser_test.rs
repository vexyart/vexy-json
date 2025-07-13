// this_file: tests/streaming_parser_test.rs

use vexy_json_core::streaming::{StreamingParser, StreamingEvent, StreamingValueBuilder};
use vexy_json_core::parser::ParserOptions;
use vexy_json_core::ast::{Value, Number};

#[test]
fn test_streaming_parser_simple_object() {
    let mut parser = StreamingParser::new();
    let input = r#"{"key": "value"}"#;
    
    parser.feed(input).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartObject,
        StreamingEvent::ObjectKey("key".to_string()),
        StreamingEvent::String("value".to_string()),
        StreamingEvent::EndObject,
        StreamingEvent::EndOfInput,
    ]);
}

#[test]
fn test_streaming_parser_array() {
    let mut parser = StreamingParser::new();
    let input = r#"[1, 2, "three", true, null]"#;
    
    parser.feed(input).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartArray,
        StreamingEvent::Number("1".to_string()),
        StreamingEvent::Number("2".to_string()),
        StreamingEvent::String("three".to_string()),
        StreamingEvent::Bool(true),
        StreamingEvent::Null,
        StreamingEvent::EndArray,
        StreamingEvent::EndOfInput,
    ]);
}

#[test]
fn test_streaming_parser_nested() {
    let mut parser = StreamingParser::new();
    let input = r#"{"nested": {"key": [1, 2]}}"#;
    
    parser.feed(input).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartObject,
        StreamingEvent::ObjectKey("nested".to_string()),
        StreamingEvent::StartObject,
        StreamingEvent::ObjectKey("key".to_string()),
        StreamingEvent::StartArray,
        StreamingEvent::Number("1".to_string()),
        StreamingEvent::Number("2".to_string()),
        StreamingEvent::EndArray,
        StreamingEvent::EndObject,
        StreamingEvent::EndObject,
        StreamingEvent::EndOfInput,
    ]);
}

#[test]
fn test_streaming_parser_chunked() {
    let mut parser = StreamingParser::new();
    
    // Feed in chunks
    parser.feed(r#"{"key"#).unwrap();
    parser.feed(r#": "val"#).unwrap();
    parser.feed(r#"ue"}"#).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartObject,
        StreamingEvent::ObjectKey("key".to_string()),
        StreamingEvent::String("value".to_string()),
        StreamingEvent::EndObject,
        StreamingEvent::EndOfInput,
    ]);
}

#[test]
fn test_streaming_value_builder() {
    let mut builder = StreamingValueBuilder::new();
    
    // Build {"key": [1, 2, "three"]}
    builder.process_event(StreamingEvent::StartObject).unwrap();
    builder.process_event(StreamingEvent::ObjectKey("key".to_string())).unwrap();
    builder.process_event(StreamingEvent::StartArray).unwrap();
    builder.process_event(StreamingEvent::Number("1".to_string())).unwrap();
    builder.process_event(StreamingEvent::Number("2".to_string())).unwrap();
    builder.process_event(StreamingEvent::String("three".to_string())).unwrap();
    builder.process_event(StreamingEvent::EndArray).unwrap();
    builder.process_event(StreamingEvent::EndObject).unwrap();
    
    let value = builder.finish().unwrap().unwrap();
    
    match value {
        Value::Object(map) => {
            let array = map.get("key").unwrap();
            match array {
                Value::Array(vec) => {
                    assert_eq!(vec.len(), 3);
                    assert_eq!(vec[0], Value::Number(Number::Integer(1)));
                    assert_eq!(vec[1], Value::Number(Number::Integer(2)));
                    assert_eq!(vec[2], Value::String("three".to_string()));
                }
                _ => panic!("Expected array"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_streaming_parser_with_comments() {
    let mut options = ParserOptions::default();
    options.allow_comments = true;
    
    let mut parser = StreamingParser::with_options(options);
    let input = r#"{
        // Comment
        "key": /* inline comment */ "value"
    }"#;
    
    parser.feed(input).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartObject,
        StreamingEvent::ObjectKey("key".to_string()),
        StreamingEvent::String("value".to_string()),
        StreamingEvent::EndObject,
        StreamingEvent::EndOfInput,
    ]);
}

#[test]
fn test_streaming_parser_empty_containers() {
    let mut parser = StreamingParser::new();
    
    // Test empty object
    parser.feed("{}").unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartObject,
        StreamingEvent::EndObject,
        StreamingEvent::EndOfInput,
    ]);
    
    // Test empty array
    let mut parser = StreamingParser::new();
    parser.feed("[]").unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events, vec![
        StreamingEvent::StartArray,
        StreamingEvent::EndArray,
        StreamingEvent::EndOfInput,
    ]);
}

#[test]
fn test_streaming_parser_string_escapes() {
    let mut parser = StreamingParser::new();
    let input = r#"{"escaped": "Hello\nWorld\t\"quoted\""}"#;
    
    parser.feed(input).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events[2], StreamingEvent::String("Hello\nWorld\t\"quoted\"".to_string()));
}

#[test]
fn test_streaming_parser_unicode() {
    let mut parser = StreamingParser::new();
    let input = r#"{"unicode": "\u0048\u0065\u006C\u006C\u006F"}"#;
    
    parser.feed(input).unwrap();
    parser.finish().unwrap();
    
    let mut events = Vec::new();
    while let Some(event) = parser.next_event().unwrap() {
        events.push(event);
    }
    
    assert_eq!(events[2], StreamingEvent::String("Hello".to_string()));
}

#[test]
fn test_streaming_parser_iterator() {
    let mut parser = StreamingParser::new();
    parser.feed(r#"{"a": 1, "b": 2}"#).unwrap();
    parser.finish().unwrap();
    
    // Use iterator interface
    let events: Vec<StreamingEvent> = parser
        .map(|r| r.unwrap())
        .collect();
    
    assert_eq!(events.len(), 6); // Start, Key, Num, Key, Num, End, EOF
    assert_eq!(events[0], StreamingEvent::StartObject);
    assert_eq!(events[5], StreamingEvent::EndOfInput);
}