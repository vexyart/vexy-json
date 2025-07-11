#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse, ParserOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Create various number input formats for fuzzing
        let number_formats = vec![
            s.to_string(),                           // Raw input as number
            format!("{{\"num\": {}}}", s),          // Number as object value
            format!("[{}]", s),                     // Number in array
            format!("{{key: {}}}", s),              // Number with unquoted key
            format!("{}.0", s),                     // Force float format
            format!("{}e10", s),                    // Scientific notation
            format!("0x{}", s),                     // Hex format (should fail in strict JSON)
            format!("0o{}", s),                     // Octal format (should fail in strict JSON)
            format!("0b{}", s),                     // Binary format (should fail in strict JSON)
            format!("+{}", s),                      // Positive sign
            format!("-{}", s),                      // Negative sign
        ];
        
        for number_input in number_formats {
            // Test with default forgiving options
            let _ = parse(&number_input);
            
            // Test with strict JSON compliance
            let strict_options = ParserOptions {
                allow_comments: false,
                allow_trailing_commas: false,
                allow_unquoted_keys: false,
                allow_single_quotes: false,
                implicit_top_level: false,
                newline_as_comma: false,
                max_depth: 64,
                enable_repair: false,
                max_repairs: 0,
                fast_repair: false,
                report_repairs: false,
            };
            let _ = vexy_json::parse_with_options(&number_input, strict_options);
        }
        
        // Test edge cases
        let edge_cases = vec![
            format!("{{\"big\": {}}}", "1".repeat(1000)),    // Very large number
            format!("{{\"precision\": {}}}", format!("{}.{}", s, "1".repeat(100))), // High precision
        ];
        
        for edge_case in edge_cases {
            let _ = parse(&edge_case);
        }
    }
});