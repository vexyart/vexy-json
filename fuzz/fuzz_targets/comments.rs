#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse, ParserOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Create various comment input formats for fuzzing
        let comment_formats = vec![
            format!("// {}\n{{\"key\": \"value\"}}", s),     // Single-line comment before
            format!("{{\"key\": \"value\"}} // {}", s),      // Single-line comment after
            format!("/* {} */{{\"key\": \"value\"}}", s),    // Multi-line comment before
            format!("{{\"key\": \"value\"}}/* {} */", s),    // Multi-line comment after
            format!("{{\"key\": /* {} */ \"value\"}}", s),   // Comment inside object
            format!("[1, /* {} */ 2, 3]", s),               // Comment inside array
            format!("// {}\n// Another comment\n{{}}", s),   // Multiple single-line comments
            format!("/* {} */ /* Another */ {{}}", s),       // Multiple multi-line comments
            format!("{{// {}\n\"key\": \"value\"}}", s),     // Comment inside object (forgiving)
            format!("{{\"key\": \"value\", // {}\n}}", s),   // Comment before closing brace
        ];
        
        for comment_input in comment_formats {
            // Test with comments enabled (default)
            let _ = parse(&comment_input);
            
            // Test with comments enabled explicitly
            let comment_options = ParserOptions {
                allow_comments: true,
                ..Default::default()
            };
            let _ = vexy_json::parse_with_options(&comment_input, comment_options);
            
            // Test with comments disabled (should fail for most)
            let no_comment_options = ParserOptions {
                allow_comments: false,
                ..Default::default()
            };
            let _ = vexy_json::parse_with_options(&comment_input, no_comment_options);
        }
        
        // Test nested comment edge cases
        let nested_comments = vec![
            format!("/* outer /* {} */ outer */", s),        // Nested comments (should not work)
            format!("// /* {} */", s),                       // Mixed comment types
            format!("/* // {} */", s),                       // Mixed comment types
        ];
        
        for nested_input in nested_comments {
            let _ = parse(&nested_input);
        }
    }
});