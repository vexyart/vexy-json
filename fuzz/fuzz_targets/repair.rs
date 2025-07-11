#![no_main]

use libfuzzer_sys::fuzz_target;
use vexy_json::{parse_with_options, ParserOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test repair functionality with various levels
        let repair_options = ParserOptions {
            enable_repair: true,
            max_repairs: 100,
            fast_repair: false,
            report_repairs: true,
            ..Default::default()
        };
        let _ = parse_with_options(s, repair_options);
        
        // Test fast repair
        let fast_repair_options = ParserOptions {
            enable_repair: true,
            max_repairs: 10,
            fast_repair: true,
            report_repairs: false,
            ..Default::default()
        };
        let _ = parse_with_options(s, fast_repair_options);
        
        // Test limited repairs
        let limited_repair_options = ParserOptions {
            enable_repair: true,
            max_repairs: 1,
            fast_repair: false,
            report_repairs: true,
            ..Default::default()
        };
        let _ = parse_with_options(s, limited_repair_options);
    }
});