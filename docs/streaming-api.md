# Streaming Parser API Documentation

## Overview

The vexy_json streaming parser provides an event-driven API for parsing JSON incrementally, making it suitable for:
- Processing large JSON files without loading them entirely into memory
- Real-time parsing of JSON data streams
- Parsing newline-delimited JSON (NDJSON) files
- Building custom JSON processing pipelines

## Core Components

### StreamingParser

The main streaming parser that processes input incrementally and emits parsing events.

```rust
use vexy_json::{StreamingParser, StreamingEvent};

let mut parser = StreamingParser::new();
parser.feed(r#"{"key": "value"}"#)?;
parser.finish()?;

while let Some(event) = parser.next_event()? {
    match event {
        StreamingEvent::StartObject => println!("Object started"),
        StreamingEvent::ObjectKey(key) => println!("Key: {}", key),
        StreamingEvent::String(s) => println!("String: {}", s),
        StreamingEvent::EndObject => println!("Object ended"),
        StreamingEvent::EndOfInput => break,
        _ => {}
    }
}
```

### StreamingEvent

Events emitted by the streaming parser:

```rust
pub enum StreamingEvent {
    StartObject,           // {
    EndObject,             // }
    StartArray,            // [
    EndArray,              // ]
    ObjectKey(String),     // "key":
    Null,                  // null
    Bool(bool),            // true/false
    Number(String),        // 42, 3.14
    String(String),        // "text"
    EndOfInput,            // End of parsing
}
```

### StreamingValueBuilder

Utility for building Value objects from streaming events:

```rust
use vexy_json::{StreamingParser, StreamingValueBuilder};

let mut parser = StreamingParser::new();
let mut builder = StreamingValueBuilder::new();

parser.feed(r#"{"name": "Alice", "age": 30}"#)?;
parser.finish()?;

while let Some(event) = parser.next_event()? {
    builder.process_event(event)?;
}

let value = builder.finish()?.unwrap();
println!("{}", value); // {"name": "Alice", "age": 30}
```

## NDJSON Support

### NdJsonParser

Parser for newline-delimited JSON where each line is a separate JSON value:

```rust
use vexy_json::NdJsonParser;

let mut parser = NdJsonParser::new();
let input = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;

let values = parser.feed(input)?;
println!("Parsed {} objects", values.len());

for value in values {
    println!("{}", value);
}
```

### StreamingNdJsonParser

Event-based NDJSON parser:

```rust
use vexy_json::StreamingNdJsonParser;

let mut parser = StreamingNdJsonParser::new();
parser.feed(r#"{"a": 1}
{"b": 2}"#)?;
parser.finish()?;

while let Some(event) = parser.next_event()? {
    // Process events for each line
    println!("{:?}", event);
}
```

## Parser Options

Both streaming parsers support the same options as the regular parser:

```rust
use vexy_json::{StreamingParser, ParserOptions};

let options = ParserOptions {
    allow_comments: true,
    allow_trailing_commas: true,
    allow_unquoted_keys: true,
    allow_single_quotes: true,
    implicit_top_level: true,
    newline_as_comma: true,
    max_depth: 128,
};

let mut parser = StreamingParser::with_options(options);
```

## Usage Patterns

### Pattern 1: Event Processing

```rust
fn process_json_stream(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = StreamingParser::new();
    parser.feed(input)?;
    parser.finish()?;
    
    while let Some(event) = parser.next_event()? {
        match event {
            StreamingEvent::ObjectKey(key) => {
                println!("Found key: {}", key);
            }
            StreamingEvent::String(s) => {
                println!("Found string: {}", s);
            }
            StreamingEvent::EndOfInput => break,
            _ => {}
        }
    }
    
    Ok(())
}
```

### Pattern 2: Incremental Processing

```rust
fn process_chunks(chunks: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = StreamingParser::new();
    
    for chunk in chunks {
        parser.feed(chunk)?;
        
        // Process available events after each chunk
        while let Some(event) = parser.next_event()? {
            if matches!(event, StreamingEvent::EndOfInput) {
                break;
            }
            // Handle event...
        }
    }
    
    parser.finish()?;
    
    // Process final events
    while let Some(event) = parser.next_event()? {
        if matches!(event, StreamingEvent::EndOfInput) {
            break;
        }
        // Handle final events...
    }
    
    Ok(())
}
```

### Pattern 3: Building Custom Values

```rust
fn build_filtered_object(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut parser = StreamingParser::new();
    let mut builder = StreamingValueBuilder::new();
    
    parser.feed(input)?;
    parser.finish()?;
    
    while let Some(event) = parser.next_event()? {
        // Filter events or transform them
        match event {
            StreamingEvent::ObjectKey(key) if key.starts_with("_") => {
                // Skip private keys
                continue;
            }
            _ => builder.process_event(event)?,
        }
    }
    
    Ok(builder.finish()?.unwrap_or(Value::Null))
}
```

## Error Handling

The streaming parser uses the same error types as the regular parser:

```rust
use vexy_json::{StreamingParser, Error};

let mut parser = StreamingParser::new();

match parser.feed("invalid json") {
    Ok(()) => println!("Chunk processed"),
    Err(Error::UnexpectedChar(ch, pos)) => {
        println!("Unexpected character '{}' at position {}", ch, pos);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## Performance Considerations

1. **Memory Usage**: The streaming parser uses minimal memory, only buffering incomplete tokens
2. **Latency**: Events are emitted as soon as complete tokens are available
3. **Throughput**: Designed for high-throughput scenarios with large datasets
4. **Buffering**: Internal buffers are automatically managed and kept minimal

## Limitations

1. **Token Values**: Due to the existing Token enum design, string and number content extraction is simplified in the current implementation
2. **Error Recovery**: The parser currently fails fast on errors rather than attempting recovery
3. **Async Support**: Async/await support is planned but not yet implemented

## Examples

See `examples/streaming_example.rs` for a complete working example demonstrating all streaming parser features.