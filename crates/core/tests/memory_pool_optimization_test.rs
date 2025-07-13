// this_file: crates/core/tests/memory_pool_optimization_test.rs

//! Tests demonstrating memory pool optimizations

use vexy_json_core::parser::{
    parse_optimized, parse_optimized_v2, parse_optimized_v3, 
    parse_v3_with_stats, ParserOptions
};

#[test]
fn test_memory_pool_optimization_improvements() {
    let json = r#"{"users": [1, 2, 3], "metadata": {"count": 3}}"#;

    // Test the new optimized parser V3
    let result = parse_optimized_v3(json);
    if let Err(e) = &result {
        println!("V3 parser error: {:?}", e);
    }
    assert!(result.is_ok(), "V3 parser should parse successfully");

    // Test with statistics to verify optimizations
    let (value, stats, alloc_stats) = parse_v3_with_stats(json, ParserOptions::default()).unwrap();
    
    // Verify the value is parsed correctly
    match value {
        vexy_json_core::ast::Value::Object(obj) => {
            assert!(obj.contains_key("users"));
            assert!(obj.contains_key("metadata"));
        }
        _ => panic!("Expected root to be an object"),
    }

    // Verify optimization statistics
    println!("Memory Pool V3 Optimization Stats:");
    println!("  Values parsed: {}", stats.values_parsed);
    println!("  Objects parsed: {}", stats.objects_parsed);
    println!("  Arrays parsed: {}", stats.arrays_parsed);
    println!("  Small vector optimizations: {}", stats.small_vec_optimizations);
    println!("  Compact string optimizations: {}", stats.compact_string_optimizations);
    println!("  Pre-sized collections: {}", stats.presized_collections);
    
    // These optimizations should have been applied
    assert!(stats.objects_parsed >= 2); // root + metadata object
    assert!(stats.arrays_parsed >= 1); // users array
    assert!(stats.small_vec_optimizations > 0); // Small arrays should trigger this
    assert!(stats.presized_collections > 0); // Collections should be pre-sized
    // Note: compact_string_optimizations might be 0 if strings are not short enough
}

#[test]
fn test_collection_presizing_optimization() {
    // Test that small objects and arrays get pre-sized correctly
    let small_array_json = r#"[1, 2, 3]"#;
    let small_object_json = r#"{"a": 1, "b": 2}"#;
    
    let (_, array_stats, _) = parse_v3_with_stats(small_array_json, ParserOptions::default()).unwrap();
    let (_, object_stats, _) = parse_v3_with_stats(small_object_json, ParserOptions::default()).unwrap();
    
    // Both should have pre-sizing optimizations
    assert!(array_stats.presized_collections > 0);
    assert!(array_stats.small_vec_optimizations > 0);
    
    assert!(object_stats.presized_collections > 0);
}

#[test]
fn test_string_optimization_heuristics() {
    let json_with_short_strings = r#"{"id": "abc", "type": "user", "role": "admin"}"#;
    let json_with_long_strings = r#"{"description": "This is a very long string that should not trigger compact string optimization because it exceeds the threshold"}"#;
    
    let (_, short_stats, _) = parse_v3_with_stats(json_with_short_strings, ParserOptions::default()).unwrap();
    let (_, long_stats, _) = parse_v3_with_stats(json_with_long_strings, ParserOptions::default()).unwrap();
    
    // Short strings should trigger more compact optimizations
    assert!(short_stats.compact_string_optimizations >= 3); // "abc", "user", "admin"
    assert!(long_stats.compact_string_optimizations < short_stats.compact_string_optimizations);
}

#[test]
fn test_parser_comparison() {
    let json = r#"{"items": [1, 2, 3, 4, 5], "settings": {"enabled": true, "count": 5}}"#;
    
    // All parsers should produce the same result
    let result_v1 = parse_optimized(json).unwrap();
    let result_v2 = parse_optimized_v2(json).unwrap();
    let result_v3 = parse_optimized_v3(json).unwrap();
    
    assert_eq!(result_v1, result_v2);
    assert_eq!(result_v2, result_v3);
    
    // V3 should provide additional optimization statistics
    let (_, stats, _) = parse_v3_with_stats(json, ParserOptions::default()).unwrap();
    assert!(stats.values_parsed > 0);
    assert!(stats.presized_collections > 0);
}

#[test]
fn test_nested_structure_optimization() {
    let nested_json = r#"{
        "level1": {
            "level2": {
                "level3": {
                    "data": [1, 2, 3],
                    "metadata": {"type": "nested"}
                }
            }
        }
    }"#;
    
    let (value, stats, _) = parse_v3_with_stats(nested_json, ParserOptions::default()).unwrap();
    
    // Verify correct parsing
    match value {
        vexy_json_core::ast::Value::Object(obj) => {
            assert!(obj.contains_key("level1"));
        }
        _ => panic!("Expected root to be an object"),
    }
    
    // Should have multiple objects and show optimization usage
    assert!(stats.objects_parsed >= 4); // root + level1 + level2 + level3 + metadata
    assert!(stats.arrays_parsed >= 1); // data array
    assert!(stats.presized_collections >= stats.objects_parsed + stats.arrays_parsed);
}