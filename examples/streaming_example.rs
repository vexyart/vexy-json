// this_file: examples/streaming_example.rs

//! Example demonstrating the streaming parser capabilities.

use vexy_json::{NdJsonParser, StreamingEvent, StreamingParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Streaming Parser Example ===\n");

    // Example 1: Basic streaming parser
    println!("1. Basic Streaming Parser:");
    let mut parser = StreamingParser::new();

    // Feed JSON in chunks
    parser.feed(r#"{"name": "#)?;
    parser.feed(r#""Alice", "age": 30"#)?;
    parser.feed(r#", "active": true}"#)?;
    parser.finish()?;

    // Process events
    while let Some(event) = parser.next_event()? {
        match event {
            StreamingEvent::StartObject => println!("  → Start Object"),
            StreamingEvent::EndObject => println!("  → End Object"),
            StreamingEvent::ObjectKey(key) => println!("  → Key: '{key}'"),
            StreamingEvent::String(s) => println!("  → String: '{s}'"),
            StreamingEvent::Number(n) => println!("  → Number: {n}"),
            StreamingEvent::Bool(b) => println!("  → Boolean: {b}"),
            StreamingEvent::Null => println!("  → Null"),
            StreamingEvent::EndOfInput => {
                println!("  → End of Input");
                break;
            }
            _ => {}
        }
    }

    println!();

    // Example 2: NDJSON Parser
    println!("2. NDJSON Parser:");
    let mut ndjson_parser = NdJsonParser::new();

    let ndjson_input = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;

    let values = ndjson_parser.feed(ndjson_input)?;

    println!("  Parsed {} JSON objects:", values.len());
    for (i, value) in values.iter().enumerate() {
        println!("    {}. {}", i + 1, value);
    }

    println!();

    // Example 3: Incremental processing
    println!("3. Incremental Processing:");
    let mut incremental_parser = StreamingParser::new();

    let chunks = [r#"["#, r#"1, "#, r#"2, "#, r#"3"#, r#"]"#];

    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Feeding chunk {}: '{}'", i + 1, chunk);
        incremental_parser.feed(chunk)?;

        // Process any available events
        while let Some(event) = incremental_parser.next_event()? {
            match event {
                StreamingEvent::StartArray => println!("    → Array started"),
                StreamingEvent::EndArray => println!("    → Array ended"),
                StreamingEvent::Number(n) => println!("    → Number: {n}"),
                StreamingEvent::EndOfInput => break,
                _ => {}
            }
        }
    }

    incremental_parser.finish()?;

    // Process final events
    while let Some(event) = incremental_parser.next_event()? {
        if matches!(event, StreamingEvent::EndOfInput) {
            println!("    → Parsing complete");
            break;
        }
    }

    println!("\n=== Streaming Example Complete ===");
    Ok(())
}
