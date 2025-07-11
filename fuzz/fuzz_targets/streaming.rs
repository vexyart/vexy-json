#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json_core::streaming::{parse_streaming, BufferedStreamingConfig};
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test streaming parser with various buffer sizes
        let cursor = Cursor::new(s);
        let mut parser = parse_streaming(cursor);
        
        // Collect all events (or until error)
        let _ = parser.collect_events();
        
        // Test with small buffer
        let cursor = Cursor::new(s);
        let config = BufferedStreamingConfig {
            input_buffer_size: 16,
            token_buffer_size: 8,
            event_buffer_size: 4,
            ..Default::default()
        };
        let mut small_parser = parse_streaming_with_config(cursor, config);
        let _ = small_parser.collect_events();
        
        // Test parsing to value
        let cursor = Cursor::new(s);
        let mut value_parser = parse_streaming(cursor);
        let _ = value_parser.parse_to_value();
    }
});