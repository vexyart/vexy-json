#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse, parse_with_options, ParserOptions};

fuzz_target!(|data: &[u8]| {
    // Test with raw bytes (may include invalid UTF-8)
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse(s);
        
        // Test with repair enabled for malformed unicode
        let repair_options = ParserOptions {
            enable_repair: true,
            max_repairs: 50,
            ..Default::default()
        };
        let _ = parse_with_options(s, repair_options);
    }
    
    // Also test with potentially malformed UTF-8 strings
    // by creating a lossy conversion
    let lossy_string = String::from_utf8_lossy(data);
    let _ = parse(&lossy_string);
});