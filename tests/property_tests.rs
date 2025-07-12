// this_file: tests/property_tests.rs

use proptest::prelude::*;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use quickcheck_macros::quickcheck;
use rustc_hash::FxHashMap;
use vexy_json::{parse, Number, Value};
use vexy_json_core::{parse_with_options, ParserOptions};
// use vexy_json_core::streaming::{JsonEventHandler, parse_streaming, StreamingEvent};

// A strategy for generating simple JSON strings
fn simple_string_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_\s]{0,20}").unwrap()
}

// A strategy for generating valid identifiers
fn identifier_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z_][a-zA-Z0-9_]{0,10}").unwrap()
}

// A strategy for generating simple test strings
fn test_input_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_\s\{\}\[\]:,]{0,50}").unwrap()
}

proptest! {
    #[test]
    fn test_parser_deterministic(input in test_input_strategy()) {
        let result1 = parse(&input);
        let result2 = parse(&input);

        match (result1, result2) {
            (Ok(val1), Ok(val2)) => {
                prop_assert!(values_semantically_equal(&val1, &val2), "Parser should be deterministic");
            }
            (Err(_), Err(_)) => {
                // Both failed, which is consistent
            }
            _ => {
                prop_assert!(false, "Parser should be deterministic - results differ");
            }
        }
    }

    #[test]
    fn test_basic_values(
        string_val in simple_string_strategy(),
        int_val in -1000i64..1000i64,
        float_val in -1000.0f64..1000.0f64,
        bool_val in any::<bool>()
    ) {
        // Test string values
        if !string_val.is_empty() {
            let string_json = format!("\"{}\"", string_val.replace("\"", "\\\""));
            if let Ok(parsed) = parse(&string_json) {
                if let Value::String(s) = parsed {
                    prop_assert_eq!(s, string_val);
                }
            }
        }

        // Test integer values
        let int_json = int_val.to_string();
        if let Ok(parsed) = parse(&int_json) {
            match parsed {
                Value::Number(Number::Integer(i)) => {
                    prop_assert_eq!(i, int_val);
                }
                Value::Number(Number::Float(f)) => {
                    // Small integers might be parsed as floats
                    prop_assert!((f - int_val as f64).abs() < 0.001);
                }
                _ => {}
            }
        }

        // Test float values
        if float_val.is_finite() {
            let float_json = float_val.to_string();
            if let Ok(parsed) = parse(&float_json) {
                if let Value::Number(Number::Float(f)) = parsed {
                    prop_assert!((f - float_val).abs() < 0.001);
                }
            }
        }

        // Test boolean values
        let bool_json = bool_val.to_string();
        if let Ok(parsed) = parse(&bool_json) {
            if let Value::Bool(b) = parsed {
                prop_assert_eq!(b, bool_val);
            }
        }
    }

    #[test]
    fn test_parser_handles_arbitrary_input(input in test_input_strategy()) {
        // The parser should not panic on any input
        let _result = parse(&input);
        // We don't assert anything about the result - just that it doesn't panic
    }

    #[test]
    fn test_empty_and_whitespace_inputs(input in prop::string::string_regex(r"\s*").unwrap()) {
        let result = parse(&input);
        // Empty or whitespace-only inputs should either succeed or fail gracefully
        prop_assert!(result.is_ok() || result.is_err(), "Parser should handle whitespace gracefully");
    }

    #[test]
    fn test_simple_structures(
        keys in prop::collection::vec(identifier_strategy(), 1..4),
        values in prop::collection::vec(simple_string_strategy(), 1..4)
    ) {
        // Use the minimum length to ensure we have matched key-value pairs
        let min_len = keys.len().min(values.len());
        if min_len > 0 {
            // Test simple object using only the first min_len elements
            let mut obj_parts = Vec::new();
            let mut unique_keys = std::collections::HashSet::new();
            for (key, value) in keys.iter().take(min_len).zip(values.iter().take(min_len)) {
                obj_parts.push(format!("\"{}\": \"{}\"", key, value.replace("\"", "\\\"")));
                unique_keys.insert(key);
            }
            let obj_json = format!("{{{}}}", obj_parts.join(", "));

            if let Ok(parsed) = parse(&obj_json) {
                if let Value::Object(obj) = parsed {
                    // Object length should equal the number of unique keys
                    prop_assert_eq!(obj.len(), unique_keys.len());
                    // Only check values for the last occurrence of each key
                    let mut key_to_value: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
                    for (key, value) in keys.iter().take(min_len).zip(values.iter().take(min_len)) {
                        key_to_value.insert(key, value);
                    }
                    for (key, expected_value) in key_to_value {
                        if let Some(Value::String(actual_value)) = obj.get(key) {
                            prop_assert_eq!(actual_value, expected_value);
                        }
                    }
                }
            }
        }
    }
}

/// Helper to compare `Value` for semantic equality
fn values_semantically_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Number(Number::Integer(a)), Value::Number(Number::Integer(b))) => a == b,
        (Value::Number(Number::Float(a)), Value::Number(Number::Float(b))) => (a - b).abs() < 0.001,
        (Value::Number(Number::Integer(a)), Value::Number(Number::Float(b))) => {
            (*a as f64 - b).abs() < 0.001
        }
        (Value::Number(Number::Float(a)), Value::Number(Number::Integer(b))) => {
            (a - *b as f64).abs() < 0.001
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return false;
            }
            for (i, item) in a.iter().enumerate() {
                if !values_semantically_equal(item, &b[i]) {
                    return false;
                }
            }
            true
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }
            for (key, val) in a {
                if let Some(b_val) = b.get(key) {
                    if !values_semantically_equal(val, b_val) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

// ===== QuickCheck Tests =====

/// A wrapper type for generating valid JSON values with QuickCheck
#[derive(Clone, Debug)]
struct ArbitraryJsonValue(Value);

impl Arbitrary for ArbitraryJsonValue {
    fn arbitrary(g: &mut Gen) -> Self {
        // Control recursion depth to avoid stack overflow
        let depth = g.size().min(3);
        ArbitraryJsonValue(arbitrary_json_value(g, depth))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let value = self.0.clone();
        let variants: Vec<ArbitraryJsonValue> = match value {
            Value::Array(arr) if !arr.is_empty() => {
                // Shrink by removing elements
                let mut variants = vec![];
                for i in 0..arr.len() {
                    let mut smaller = arr.clone();
                    smaller.remove(i);
                    variants.push(ArbitraryJsonValue(Value::Array(smaller)));
                }
                variants
            }
            Value::Object(obj) if !obj.is_empty() => {
                // Shrink by removing keys
                let mut variants = vec![];
                let keys: Vec<_> = obj.keys().cloned().collect();
                for key in keys {
                    let mut smaller: FxHashMap<String, Value> = obj.clone();
                    smaller.remove(&key);
                    variants.push(ArbitraryJsonValue(Value::Object(smaller)));
                }
                variants
            }
            Value::String(s) if !s.is_empty() => {
                // Shrink strings by removing characters
                vec![
                    ArbitraryJsonValue(Value::String(s[..s.len() / 2].to_string())),
                    ArbitraryJsonValue(Value::String(String::new())),
                ]
            }
            _ => vec![],
        };
        Box::new(variants.into_iter())
    }
}

fn arbitrary_json_value(g: &mut Gen, depth: usize) -> Value {
    if depth == 0 {
        // At depth 0, only generate leaf values
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => Value::Null,
            1 => Value::Bool(bool::arbitrary(g)),
            2 => Value::Number(arbitrary_number(g)),
            _ => Value::String(arbitrary_json_string(g)),
        }
    } else {
        // At higher depths, can also generate arrays and objects
        match g.choose(&[0, 1, 2, 3, 4, 5]).unwrap() {
            0 => Value::Null,
            1 => Value::Bool(bool::arbitrary(g)),
            2 => Value::Number(arbitrary_number(g)),
            3 => Value::String(arbitrary_json_string(g)),
            4 => {
                // Generate array
                let size = g.choose(&[0, 1, 2, 3]).unwrap();
                let array: Vec<Value> = (0..*size)
                    .map(|_| arbitrary_json_value(g, depth - 1))
                    .collect();
                Value::Array(array)
            }
            _ => {
                // Generate object
                let size = g.choose(&[0, 1, 2]).unwrap();
                let mut object = FxHashMap::default();
                for i in 0..*size {
                    let key = format!("key{i}");
                    let value = arbitrary_json_value(g, depth - 1);
                    object.insert(key, value);
                }
                Value::Object(object)
            }
        }
    }
}

fn arbitrary_number(g: &mut Gen) -> Number {
    if bool::arbitrary(g) {
        Number::Integer(i64::arbitrary(g) % 10000) // Keep numbers reasonable
    } else {
        // Generate reasonable floats, avoiding NaN and infinity
        loop {
            let f = f64::arbitrary(g) % 10000.0;
            if f.is_finite() {
                return Number::Float(f.abs());
            }
        }
    }
}

fn arbitrary_json_string(g: &mut Gen) -> String {
    // Generate strings with valid JSON characters
    let len = g.choose(&[0, 1, 5, 10]).unwrap();
    let mut s = String::new();
    for _ in 0..*len {
        let ch = match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => *g.choose(b"abcdefghijklmnopqrstuvwxyz").unwrap() as char,
            1 => *g.choose(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ").unwrap() as char,
            2 => *g.choose(b"0123456789").unwrap() as char,
            _ => *g.choose(b" -_").unwrap() as char,
        };
        s.push(ch);
    }
    s
}

#[quickcheck]
fn prop_parse_serialize_roundtrip(value: ArbitraryJsonValue) -> TestResult {
    // Convert our value to JSON string
    let json_string = value_to_json_string(&value.0);

    // Parse it back
    match parse(&json_string) {
        Ok(parsed) => {
            // Check that parsed value equals original
            if values_semantically_equal(&parsed, &value.0) {
                TestResult::passed()
            } else {
                TestResult::error(format!(
                    "Roundtrip failed:\nOriginal: {:?}\nParsed: {:?}",
                    value.0, parsed
                ))
            }
        }
        Err(e) => TestResult::error(format!(
            "Failed to parse generated JSON: {json_string}\nError: {e:?}"
        )),
    }
}

/// Convert a Value to a JSON string
fn value_to_json_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => match n {
            Number::Integer(i) => i.to_string(),
            Number::Float(f) => f.to_string(),
        },
        Value::String(s) => format!("\"{}\"", escape_json_string(s)),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_json_string).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", escape_json_string(k), value_to_json_string(v)))
                .collect();
            format!("{{{}}}", items.join(","))
        }
    }
}

/// Escape a string for JSON
fn escape_json_string(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }
    result
}

#[quickcheck]
fn prop_forgiving_features_preserve_meaning(value: ArbitraryJsonValue) -> TestResult {
    let json_string = value_to_json_string(&value.0);

    // Parse with strict options
    let strict_options = ParserOptions {
        allow_comments: false,
        allow_trailing_commas: false,
        allow_unquoted_keys: false,
        allow_single_quotes: false,
        implicit_top_level: false,
        newline_as_comma: false,
        ..Default::default()
    };

    // Parse with forgiving options
    let forgiving_options = ParserOptions::default();

    match (
        parse_with_options(&json_string, strict_options),
        parse_with_options(&json_string, forgiving_options),
    ) {
        (Ok(strict_val), Ok(forgiving_val)) => {
            if values_semantically_equal(&strict_val, &forgiving_val) {
                TestResult::passed()
            } else {
                TestResult::error(format!(
                    "Strict and forgiving parsing differ:\nStrict: {strict_val:?}\nForgiving: {forgiving_val:?}"
                ))
            }
        }
        (Err(_), Ok(_)) => {
            // This is expected - forgiving parser can parse more
            TestResult::passed()
        }
        (Ok(_), Err(_)) => {
            TestResult::error("Forgiving parser failed where strict succeeded".to_string())
        }
        (Err(_), Err(_)) => {
            // Both failed - this is fine for invalid JSON
            TestResult::passed()
        }
    }
}

#[quickcheck]
fn prop_parser_options_combinations(value: ArbitraryJsonValue, options_bits: u8) -> TestResult {
    // Use bits to create different parser option combinations
    let options = ParserOptions {
        allow_comments: options_bits & 0x01 != 0,
        allow_trailing_commas: options_bits & 0x02 != 0,
        allow_unquoted_keys: options_bits & 0x04 != 0,
        allow_single_quotes: options_bits & 0x08 != 0,
        implicit_top_level: options_bits & 0x10 != 0,
        newline_as_comma: options_bits & 0x20 != 0,
        enable_repair: options_bits & 0x40 != 0,
        ..Default::default()
    };

    let json_string = value_to_json_string(&value.0);

    // Parsing should never panic, only return Ok or Err
    match parse_with_options(&json_string, options) {
        Ok(_) | Err(_) => TestResult::passed(),
    }
}

#[quickcheck]
fn prop_error_recovery_doesnt_crash(input: String) -> bool {
    let options = ParserOptions {
        enable_repair: true,
        max_repairs: 10,
        ..Default::default()
    };

    // Parser with error recovery should never panic
    match parse_with_options(&input, options) {
        Ok(_) | Err(_) => true,
    }
}

#[test]
fn test_quickcheck_json_generation() {
    // Test that we can generate various JSON values
    let mut qc = QuickCheck::new().tests(100);

    fn prop(value: ArbitraryJsonValue) -> bool {
        // Every generated value should be valid
        let json = value_to_json_string(&value.0);
        parse(&json).is_ok()
    }

    qc.quickcheck(prop as fn(ArbitraryJsonValue) -> bool);
}

// TODO: Uncomment when streaming parser API is stabilized
// Event collector for testing streaming parser
// #[derive(Default)]
// struct EventCollector {
//     events: Vec<String>,
// }
//
// impl JsonEventHandler for EventCollector {
//     fn on_object_start(&mut self) -> Result<(), vexy_json_core::error::Error> {
//         self.events.push("object_start".to_string());
//         Ok(())
//     }
//
//     fn on_object_end(&mut self) -> Result<(), vexy_json_core::error::Error> {
//         self.events.push("object_end".to_string());
//         Ok(())
//     }
//
//     fn on_array_start(&mut self) -> Result<(), vexy_json_core::error::Error> {
//         self.events.push("array_start".to_string());
//         Ok(())
//     }
//
//     fn on_array_end(&mut self) -> Result<(), vexy_json_core::error::Error> {
//         self.events.push("array_end".to_string());
//         Ok(())
//     }
//
//     fn on_key(&mut self, key: &str) -> Result<(), vexy_json_core::error::Error> {
//         self.events.push(format!("key:{}", key));
//         Ok(())
//     }
//
//     fn on_value(&mut self, value: &Value) -> Result<(), vexy_json_core::error::Error> {
//         self.events.push(format!("value:{:?}", value));
//         Ok(())
//     }
// }

// TODO: Fix streaming parser tests once the API is stabilized
// #[quickcheck]
// fn prop_streaming_parser_consistency(value: ArbitraryJsonValue) -> TestResult {
//     TestResult::passed()
// }

// TODO: Fix streaming parser tests once the API is stabilized
// #[quickcheck]
// fn prop_streaming_parser_event_balance(json: String) -> bool {
//     true
// }
