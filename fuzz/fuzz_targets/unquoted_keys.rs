#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse_with_options, ParserOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test specifically for unquoted keys feature
        let options = ParserOptions {
            allow_unquoted_keys: true,
            allow_comments: false,
            allow_trailing_commas: false,
            allow_single_quotes: false,
            implicit_top_level: false,
            newline_as_comma: false,
            ..Default::default()
        };
        let _ = parse_with_options(s, options);
        
        // Test with strict options (should fail for unquoted keys)
        let strict_options = ParserOptions {
            allow_unquoted_keys: false,
            ..Default::default()
        };
        let _ = parse_with_options(s, strict_options);
    }
});