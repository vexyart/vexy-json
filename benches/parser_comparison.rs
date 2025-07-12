// this_file: benches/parser_comparison.rs

//! Benchmarks comparing different parser implementations.
//!
//! This benchmark suite compares the performance of:
//! - Original parser (optimized)
//! - Optimized parser V2 (optimized_v2)
//! - Recursive descent parser (recursive)
//! - Stack-based iterative parser (iterative)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use vexy_json_core::{
    parse_iterative, parse_optimized_v2_with_options, parse_optimized_with_options,
    parse_recursive, parse_with_options, ParserOptions,
};

// Test data for benchmarking
struct TestData {
    name: &'static str,
    json: &'static str,
}

const TEST_DATA: &[TestData] = &[
    TestData {
        name: "simple_object",
        json: r#"{"name": "John", "age": 30, "city": "New York"}"#,
    },
    TestData {
        name: "small_array",
        json: r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#,
    },
    TestData {
        name: "nested_object",
        json: r#"{"user": {"profile": {"name": "Alice", "settings": {"theme": "dark", "notifications": true}}}}"#,
    },
    TestData {
        name: "array_of_objects",
        json: r#"[{"id": 1, "name": "Item 1"}, {"id": 2, "name": "Item 2"}, {"id": 3, "name": "Item 3"}]"#,
    },
    TestData {
        name: "with_comments",
        json: r#"{"key": "value", /* comment */ "number": 42, "flag": true}"#,
    },
    TestData {
        name: "with_trailing_commas",
        json: r#"{"key": "value", "number": 42, "array": [1, 2, 3,],}"#,
    },
    TestData {
        name: "unquoted_keys",
        json: r#"{key: "value", number: 42, flag: true}"#,
    },
    TestData {
        name: "mixed_types",
        json: r#"{"string": "hello", "number": 3.14, "boolean": true, "null": null, "array": [1, 2, 3], "object": {"nested": "value"}}"#,
    },
];

// Deep nesting test data
fn generate_deep_object(depth: usize) -> String {
    let mut json = String::new();
    for _ in 0..depth {
        json.push_str(r#"{"nested": "#);
    }
    json.push_str("\"value\"");
    for _ in 0..depth {
        json.push('}');
    }
    json
}

fn generate_deep_array(depth: usize) -> String {
    let mut json = String::new();
    for _ in 0..depth {
        json.push('[');
    }
    json.push_str("42");
    for _ in 0..depth {
        json.push(']');
    }
    json
}

fn bench_parser_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_comparison");

    let options = ParserOptions::default();

    for test_data in TEST_DATA {
        // Benchmark original parser
        group.bench_with_input(
            BenchmarkId::new("original", test_data.name),
            test_data.json,
            |b, json| {
                b.iter(|| {
                    parse_with_options(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        // Benchmark optimized parser
        group.bench_with_input(
            BenchmarkId::new("optimized", test_data.name),
            test_data.json,
            |b, json| {
                b.iter(|| {
                    parse_optimized_with_options(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        // Benchmark optimized parser V2
        group.bench_with_input(
            BenchmarkId::new("optimized_v2", test_data.name),
            test_data.json,
            |b, json| {
                b.iter(|| {
                    parse_optimized_v2_with_options(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        // Benchmark recursive descent parser
        group.bench_with_input(
            BenchmarkId::new("recursive", test_data.name),
            test_data.json,
            |b, json| {
                b.iter(|| {
                    parse_recursive(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        // Benchmark iterative parser
        group.bench_with_input(
            BenchmarkId::new("iterative", test_data.name),
            test_data.json,
            |b, json| {
                b.iter(|| {
                    parse_iterative(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_deep_nesting(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nesting");

    let options = ParserOptions::default();
    let depths = [10, 20, 50, 100];

    for depth in depths {
        let deep_object = generate_deep_object(depth);
        let deep_array = generate_deep_array(depth);

        // Deep object benchmarks
        group.bench_with_input(
            BenchmarkId::new("recursive_deep_object", depth),
            &deep_object,
            |b, json| {
                b.iter(|| {
                    parse_recursive(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterative_deep_object", depth),
            &deep_object,
            |b, json| {
                b.iter(|| {
                    parse_iterative(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        // Deep array benchmarks
        group.bench_with_input(
            BenchmarkId::new("recursive_deep_array", depth),
            &deep_array,
            |b, json| {
                b.iter(|| {
                    parse_recursive(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterative_deep_array", depth),
            &deep_array,
            |b, json| {
                b.iter(|| {
                    parse_iterative(black_box(json), black_box(options.clone()))
                        .expect("Parse should succeed")
                });
            },
        );
    }

    group.finish();
}

fn bench_forgiving_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("forgiving_features");

    let forgiving_options = ParserOptions::default();
    let strict_options = ParserOptions {
        allow_comments: false,
        allow_trailing_commas: false,
        allow_unquoted_keys: false,
        allow_single_quotes: false,
        implicit_top_level: false,
        newline_as_comma: false,
        ..ParserOptions::default()
    };

    let test_cases = [
        (
            "comments",
            r#"{"key": "value", /* comment */ "number": 42}"#,
        ),
        ("trailing_commas", r#"{"key": "value", "number": 42,}"#),
        ("unquoted_keys", r#"{key: "value", number: 42}"#),
        ("single_quotes", r#"{'key': 'value', 'number': 42}"#),
        (
            "newline_as_comma",
            "{\n\"key\": \"value\"\n\"number\": 42\n}",
        ),
    ];

    for (feature, json) in test_cases {
        // Test with forgiving options
        group.bench_with_input(BenchmarkId::new("forgiving", feature), json, |b, json| {
            b.iter(|| {
                let _ = parse_recursive(black_box(json), black_box(forgiving_options.clone()));
            });
        });

        // Test with strict options (some will fail, that's expected)
        group.bench_with_input(BenchmarkId::new("strict", feature), json, |b, json| {
            b.iter(|| {
                let _ = parse_recursive(black_box(json), black_box(strict_options.clone()));
            });
        });
    }

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let options = ParserOptions::default();

    // Large JSON structures to test memory allocation patterns
    let large_object = {
        let mut json = String::from("{");
        for i in 0..1000 {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(r#""key{i}": "value{i}""#));
        }
        json.push('}');
        json
    };

    let large_array = {
        let mut json = String::from("[");
        for i in 0..1000 {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(r#"{{"id": {i}, "value": "item{i}"}}"#));
        }
        json.push(']');
        json
    };

    // Test memory usage with different parsers
    group.bench_function("optimized_large_object", |b| {
        b.iter(|| {
            parse_optimized_with_options(black_box(&large_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("optimized_v2_large_object", |b| {
        b.iter(|| {
            parse_optimized_v2_with_options(black_box(&large_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("recursive_large_object", |b| {
        b.iter(|| {
            parse_recursive(black_box(&large_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("iterative_large_object", |b| {
        b.iter(|| {
            parse_iterative(black_box(&large_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("optimized_large_array", |b| {
        b.iter(|| {
            parse_optimized_with_options(black_box(&large_array), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("optimized_v2_large_array", |b| {
        b.iter(|| {
            parse_optimized_v2_with_options(black_box(&large_array), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("recursive_large_array", |b| {
        b.iter(|| {
            parse_recursive(black_box(&large_array), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("iterative_large_array", |b| {
        b.iter(|| {
            parse_iterative(black_box(&large_array), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    let options = ParserOptions::default();

    let invalid_json_cases = [
        ("missing_quote", r#"{"key": "value, "number": 42}"#),
        ("missing_comma", r#"{"key": "value" "number": 42}"#),
        ("missing_colon", r#"{"key" "value", "number": 42}"#),
        ("unclosed_object", r#"{"key": "value", "number": 42"#),
        ("unclosed_array", r#"[1, 2, 3, 4, 5"#),
        ("trailing_comma_strict", r#"{"key": "value",}"#),
    ];

    for (error_type, json) in invalid_json_cases {
        group.bench_with_input(
            BenchmarkId::new("recursive_error", error_type),
            json,
            |b, json| {
                b.iter(|| {
                    let _ = parse_recursive(black_box(json), black_box(options.clone()));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterative_error", error_type),
            json,
            |b, json| {
                b.iter(|| {
                    let _ = parse_iterative(black_box(json), black_box(options.clone()));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_comparison,
    bench_deep_nesting,
    bench_forgiving_features,
    bench_memory_usage,
    bench_error_handling
);
criterion_main!(benches);
