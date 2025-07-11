//! Comprehensive benchmark suite for vexy_json parser performance.
//!
//! This benchmark suite measures various aspects of parser performance including:
//! - Basic parsing speed
//! - Memory pool effectiveness
//! - SIMD optimization impact
//! - Large file handling
//! - Error recovery performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use vexy_json_core::{
    parse, parse_optimized, parse_optimized_v2,
    parse_optimized_with_options, parse_v2_with_stats, parse_with_stats, ParserOptions,
};

/// Sample JSON documents for benchmarking
mod samples {
    pub const SIMPLE_OBJECT: &str = r#"{"name": "John", "age": 30, "city": "New York"}"#;

    pub const NESTED_OBJECT: &str = r#"{
        "user": {
            "id": 12345,
            "profile": {
                "name": "John Doe",
                "email": "john@example.com",
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        }
    }"#;

    pub const ARRAY_OF_OBJECTS: &str = r#"[
        {"id": 1, "name": "Item 1", "value": 10.5},
        {"id": 2, "name": "Item 2", "value": 20.0},
        {"id": 3, "name": "Item 3", "value": 30.5},
        {"id": 4, "name": "Item 4", "value": 40.0},
        {"id": 5, "name": "Item 5", "value": 50.5}
    ]"#;

    pub const STRING_HEAVY: &str = r#"{
        "title": "The quick brown fox jumps over the lazy dog",
        "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        "content": "This is a longer text that contains various characters and needs to be parsed efficiently by the JSON parser. It includes some special characters like \"quotes\" and \n newlines.",
        "tags": ["json", "parsing", "benchmark", "performance", "optimization"]
    }"#;

    pub const NUMBER_HEAVY: &str = r#"{
        "integers": [1, 2, 3, 4, 5, 10, 20, 30, 40, 50, 100, 200, 300, 400, 500],
        "floats": [1.1, 2.2, 3.3, 4.4, 5.5, 10.1, 20.2, 30.3, 40.4, 50.5],
        "scientific": [1e10, 2.5e-10, 3.14159e0, 6.022e23, 1.602e-19],
        "mixed": [42, 3.14, -17, 0, 999999999, 0.0000001, -273.15]
    }"#;

    pub const MALFORMED_JSON: &str = r#"{'name': 'John', age: 30, "items": [1, 2, 3,]}"#;
}

/// Benchmarks basic parsing performance
fn bench_basic_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_parsing");

    group.bench_function("simple_object", |b| {
        b.iter(|| {
            let result = parse(black_box(samples::SIMPLE_OBJECT));
            assert!(result.is_ok());
        })
    });

    group.bench_function("nested_object", |b| {
        b.iter(|| {
            let result = parse(black_box(samples::NESTED_OBJECT));
            assert!(result.is_ok());
        })
    });

    group.bench_function("array_of_objects", |b| {
        b.iter(|| {
            let result = parse(black_box(samples::ARRAY_OF_OBJECTS));
            assert!(result.is_ok());
        })
    });

    group.finish();
}

/// Benchmarks optimized parser with memory pooling
fn bench_optimized_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimized_parsing");

    group.bench_function("simple_object_optimized", |b| {
        b.iter(|| {
            let result = parse_optimized(black_box(samples::SIMPLE_OBJECT));
            assert!(result.is_ok());
        })
    });

    group.bench_function("string_heavy_optimized", |b| {
        b.iter(|| {
            let result = parse_optimized(black_box(samples::STRING_HEAVY));
            assert!(result.is_ok());
        })
    });

    group.bench_function("number_heavy_optimized", |b| {
        b.iter(|| {
            let result = parse_optimized(black_box(samples::NUMBER_HEAVY));
            assert!(result.is_ok());
        })
    });

    group.finish();
}

/// Benchmarks optimized parser v2 with adaptive memory pooling
fn bench_optimized_v2_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimized_v2_parsing");

    group.bench_function("simple_object_optimized_v2", |b| {
        b.iter(|| {
            let result = parse_optimized_v2(black_box(samples::SIMPLE_OBJECT));
            assert!(result.is_ok());
        })
    });

    group.bench_function("string_heavy_optimized_v2", |b| {
        b.iter(|| {
            let result = parse_optimized_v2(black_box(samples::STRING_HEAVY));
            assert!(result.is_ok());
        })
    });

    group.bench_function("number_heavy_optimized_v2", |b| {
        b.iter(|| {
            let result = parse_optimized_v2(black_box(samples::NUMBER_HEAVY));
            assert!(result.is_ok());
        })
    });

    // Test with many small strings to see adaptive pooling in action
    let small_strings_json = r#"{
        "a": "x", "b": "y", "c": "z", "d": "w",
        "e": "1", "f": "2", "g": "3", "h": "4",
        "i": "5", "j": "6", "k": "7", "l": "8"
    }"#;

    group.bench_function("small_strings_optimized_v2", |b| {
        b.iter(|| {
            let result = parse_optimized_v2(black_box(small_strings_json));
            assert!(result.is_ok());
        })
    });

    group.finish();
}

/// Benchmarks memory pool effectiveness
fn bench_memory_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool");

    // Create a JSON document with many repeated strings
    let repeated_strings = r#"{
        "users": [
            {"type": "admin", "status": "active", "role": "admin"},
            {"type": "user", "status": "active", "role": "user"},
            {"type": "admin", "status": "inactive", "role": "admin"},
            {"type": "user", "status": "active", "role": "user"},
            {"type": "moderator", "status": "active", "role": "moderator"}
        ]
    }"#;

    group.bench_function("with_pooling_v1", |b| {
        b.iter(|| {
            let (_, stats, memory_stats) = parse_with_stats(black_box(repeated_strings)).unwrap();
            assert!(stats.pooled_allocations > 0);
            assert!(memory_stats.total_used > 0);
        })
    });

    group.bench_function("with_pooling_v2", |b| {
        b.iter(|| {
            let (_, stats, memory_stats) =
                parse_v2_with_stats(black_box(repeated_strings)).unwrap();
            // v2 may bypass some allocations
            assert!(stats.pooled_allocations > 0 || stats.bypassed_allocations > 0);
            assert!(memory_stats.total_bytes > 0);
        })
    });

    group.bench_function("without_pooling", |b| {
        b.iter(|| {
            let result = parse(black_box(repeated_strings));
            assert!(result.is_ok());
        })
    });

    group.finish();
}

/// Benchmarks parser performance with different input sizes
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    // Generate JSON arrays of different sizes
    let sizes = vec![10, 100, 1000, 10000];

    for size in sizes {
        let json = generate_array(size);

        group.bench_with_input(BenchmarkId::new("parse", size), &json, |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                assert!(result.is_ok());
            })
        });

        group.bench_with_input(
            BenchmarkId::new("parse_optimized", size),
            &json,
            |b, json| {
                b.iter(|| {
                    let result = parse_optimized(black_box(json));
                    assert!(result.is_ok());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_optimized_v2", size),
            &json,
            |b, json| {
                b.iter(|| {
                    let result = parse_optimized_v2(black_box(json));
                    assert!(result.is_ok());
                })
            },
        );
    }

    group.finish();
}

/// Benchmarks error recovery performance
fn bench_error_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery");

    let options = ParserOptions {
        allow_trailing_commas: true,
        allow_single_quotes: true,
        allow_comments: false,
        allow_unquoted_keys: true,
        implicit_top_level: false,
        newline_as_comma: false,
        max_depth: 128,
        enable_repair: true,
        max_repairs: 20,
        fast_repair: false,
        report_repairs: true,
    };

    group.bench_function("malformed_json_recovery", |b| {
        b.iter(|| {
            let result =
                parse_optimized_with_options(black_box(samples::MALFORMED_JSON), options.clone());
            assert!(result.is_ok());
        })
    });

    group.finish();
}

/// Benchmarks real-world JSON files if available
fn bench_real_world(c: &mut Criterion) {
    // Try to load some real-world JSON files for benchmarking
    let test_files = vec![
        ("small", "benches/data/small.json"),
        ("medium", "benches/data/medium.json"),
        ("large", "benches/data/large.json"),
    ];

    let mut group = c.benchmark_group("real_world");

    for (name, path) in test_files {
        if let Ok(content) = fs::read_to_string(path) {
            group.bench_function(name, |b| {
                b.iter(|| {
                    let result = parse_optimized(black_box(&content));
                    assert!(result.is_ok());
                })
            });
        }
    }

    group.finish();
}

/// Generates a JSON array with the specified number of elements
fn generate_array(size: usize) -> String {
    let mut json = String::from("[");

    for i in 0..size {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"id":{},"name":"Item {}","value":{}.5}}"#,
            i,
            i,
            i * 10
        ));
    }

    json.push(']');
    json
}

criterion_group!(
    benches,
    bench_basic_parsing,
    bench_optimized_parsing,
    bench_optimized_v2_parsing,
    bench_memory_pool,
    bench_scaling,
    bench_error_recovery, // TODO: Fix error recovery for optimized parser
    bench_real_world
);

criterion_main!(benches);
