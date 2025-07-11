#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse, parse_with_options, ParserOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test with default options (forgiving)
        let _ = parse(s);
        
        // Test with strict options
        let strict_options = ParserOptions {
            allow_comments: false,
            allow_trailing_commas: false,
            allow_unquoted_keys: false,
            allow_single_quotes: false,
            implicit_top_level: false,
            newline_as_comma: false,
            max_depth: 100,
            enable_repair: false,
            max_repairs: 0,
            fast_repair: false,
            report_repairs: false,
        };
        let _ = parse_with_options(s, strict_options);
        
        // Test with maximum forgiveness
        let forgiving_options = ParserOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            allow_unquoted_keys: true,
            allow_single_quotes: true,
            implicit_top_level: true,
            newline_as_comma: true,
            max_depth: 1000,
            enable_repair: true,
            max_repairs: 100,
            fast_repair: false,
            report_repairs: true,
        };
        let _ = parse_with_options(s, forgiving_options);
    }
});