#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse, ParserOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Create various string input formats for fuzzing
        let string_formats = vec![
            format!("\"{}\"", s),              // Double quoted string
            format!("'{}'", s),                // Single quoted string (forgiving)
            format!("{{\"key\": \"{}\"}}", s), // String as object value
            format!("[\"{}\"]", s),            // String in array
            format!("{{key: \"{}\"}}", s),     // Unquoted key with string value
        ];
        
        for string_input in string_formats {
            // Test with default forgiving options
            let _ = parse(&string_input);
            
            // Test with single quotes enabled
            let single_quote_options = ParserOptions {
                allow_single_quotes: true,
                allow_unquoted_keys: true,
                ..Default::default()
            };
            let _ = vexy_json::parse_with_options(&string_input, single_quote_options);
        }
        
        // Test raw string parsing (edge case)
        let raw_string = format!("key: {}", s);
        let _ = parse(&raw_string);
    }
});