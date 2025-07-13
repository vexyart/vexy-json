// this_file: benches/stack_overflow_test.rs

//! Benchmarks testing stack overflow scenarios and extreme deep nesting.
//!
//! This benchmark suite specifically tests scenarios where the iterative parser
//! should outperform recursive parsers by avoiding stack overflow issues.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use vexy_json_core::{parse_iterative, parse_recursive, ParserOptions};

// Generate extremely deep nested structures
fn generate_extreme_deep_object(depth: usize) -> String {
    let mut json = String::new();
    for _ in 0..depth {
        json.push_str(r#"{"level": "#);
    }
    json.push_str("\"bottom\"");
    for _ in 0..depth {
        json.push('}');
    }
    json
}

fn generate_extreme_deep_array(depth: usize) -> String {
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

fn generate_mixed_deep_structure(depth: usize) -> String {
    let mut json = String::new();
    for i in 0..depth {
        if i % 2 == 0 {
            json.push_str(r#"{"array": ["#);
        } else {
            json.push_str(r#"{"object": {"nested": "#);
        }
    }
    json.push_str("\"value\"");
    for i in 0..depth {
        if i % 2 == 0 {
            json.push_str("]}");
        } else {
            json.push_str("}}");
        }
    }
    json
}

fn bench_extreme_deep_nesting(c: &mut Criterion) {
    let mut group = c.benchmark_group("extreme_deep_nesting");

    let options = ParserOptions::default();

    // Test with progressively deeper nesting
    // Note: We start with lower depths for recursive parser to avoid actual stack overflow
    let depths = [50, 100, 200, 500, 1000, 2000];

    for depth in depths {
        let deep_object = generate_extreme_deep_object(depth);
        let deep_array = generate_extreme_deep_array(depth);
        let mixed_structure = generate_mixed_deep_structure(depth);

        // Only test recursive parser on smaller depths to avoid stack overflow
        if depth <= 200 {
            group.bench_with_input(
                BenchmarkId::new("recursive_extreme_object", depth),
                &deep_object,
                |b, json| {
                    b.iter(|| {
                        match parse_recursive(black_box(json), black_box(options.clone())) {
                            Ok(_) => {}
                            Err(_) => {} // Expected for very deep structures
                        }
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("recursive_extreme_array", depth),
                &deep_array,
                |b, json| {
                    b.iter(|| {
                        match parse_recursive(black_box(json), black_box(options.clone())) {
                            Ok(_) => {}
                            Err(_) => {} // Expected for very deep structures
                        }
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("recursive_mixed_structure", depth),
                &mixed_structure,
                |b, json| {
                    b.iter(|| {
                        match parse_recursive(black_box(json), black_box(options.clone())) {
                            Ok(_) => {}
                            Err(_) => {} // Expected for very deep structures
                        }
                    });
                },
            );
        }

        // Test iterative parser on all depths
        group.bench_with_input(
            BenchmarkId::new("iterative_extreme_object", depth),
            &deep_object,
            |b, json| {
                b.iter(|| {
                    match parse_iterative(black_box(json), black_box(options.clone())) {
                        Ok(_) => {}
                        Err(_) => {} // May fail due to depth limits, not stack overflow
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterative_extreme_array", depth),
            &deep_array,
            |b, json| {
                b.iter(|| {
                    match parse_iterative(black_box(json), black_box(options.clone())) {
                        Ok(_) => {}
                        Err(_) => {} // May fail due to depth limits, not stack overflow
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterative_mixed_structure", depth),
            &mixed_structure,
            |b, json| {
                b.iter(|| {
                    match parse_iterative(black_box(json), black_box(options.clone())) {
                        Ok(_) => {}
                        Err(_) => {} // May fail due to depth limits, not stack overflow
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_with_high_depth_limit(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_depth_limit");

    // Test with high depth limit to see how parsers handle it
    let mut high_depth_options = ParserOptions::default();
    high_depth_options.max_depth = 5000;

    let depths = [1000, 2000, 3000, 4000];

    for depth in depths {
        let deep_object = generate_extreme_deep_object(depth);

        // Only test smaller depths with recursive parser
        if depth <= 1000 {
            group.bench_with_input(
                BenchmarkId::new("recursive_high_depth", depth),
                &deep_object,
                |b, json| {
                    b.iter(|| {
                        match parse_recursive(
                            black_box(json),
                            black_box(high_depth_options.clone()),
                        ) {
                            Ok(_) => {}
                            Err(_) => {} // Expected for very deep structures
                        }
                    });
                },
            );
        }

        // Test iterative parser with all depths
        group.bench_with_input(
            BenchmarkId::new("iterative_high_depth", depth),
            &deep_object,
            |b, json| {
                b.iter(|| {
                    match parse_iterative(black_box(json), black_box(high_depth_options.clone())) {
                        Ok(_) => {}
                        Err(_) => {} // Should handle deeper structures better
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let options = ParserOptions::default();

    // Test structures that create many temporary objects
    let wide_object = {
        let mut json = String::from("{");
        for i in 0..10000 {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                r#""key{i}": {{"nested": "value{i}", "number": {i}}}"#
            ));
        }
        json.push('}');
        json
    };

    let wide_array = {
        let mut json = String::from("[");
        for i in 0..10000 {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(r#"[{i}, "item{i}", true, null]"#));
        }
        json.push(']');
        json
    };

    group.bench_function("iterative_wide_object", |b| {
        b.iter(|| {
            parse_iterative(black_box(&wide_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("iterative_wide_array", |b| {
        b.iter(|| {
            parse_iterative(black_box(&wide_array), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    // Compare with recursive parser on smaller structures
    let smaller_wide_object = {
        let mut json = String::from("{");
        for i in 0..1000 {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                r#""key{i}": {{"nested": "value{i}", "number": {i}}}"#
            ));
        }
        json.push('}');
        json
    };

    group.bench_function("recursive_smaller_wide_object", |b| {
        b.iter(|| {
            parse_recursive(black_box(&smaller_wide_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.bench_function("iterative_smaller_wide_object", |b| {
        b.iter(|| {
            parse_iterative(black_box(&smaller_wide_object), black_box(options.clone()))
                .expect("Parse should succeed")
        });
    });

    group.finish();
}

fn bench_pathological_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological_cases");

    let options = ParserOptions::default();

    // Alternating deep nesting between arrays and objects
    let alternating_structure = {
        let mut json = String::new();
        for i in 0..500 {
            if i % 2 == 0 {
                json.push_str(r#"{"array": ["#);
            } else {
                json.push_str(r#"{"object": {"key": "#);
            }
        }
        json.push_str("\"value\"");
        for i in 0..500 {
            if i % 2 == 0 {
                json.push_str("]}");
            } else {
                json.push_str("}}");
            }
        }
        json
    };

    // Very deep array with single elements
    let deep_single_array = {
        let mut json = String::new();
        for _ in 0..1000 {
            json.push('[');
        }
        json.push_str("42");
        for _ in 0..1000 {
            json.push(']');
        }
        json
    };

    // Test iterative parser with pathological cases
    group.bench_function("iterative_alternating_structure", |b| {
        b.iter(|| {
            match parse_iterative(
                black_box(&alternating_structure),
                black_box(options.clone()),
            ) {
                Ok(_) => {}
                Err(_) => {} // May fail due to depth limits
            }
        });
    });

    group.bench_function("iterative_deep_single_array", |b| {
        b.iter(|| {
            match parse_iterative(black_box(&deep_single_array), black_box(options.clone())) {
                Ok(_) => {}
                Err(_) => {} // May fail due to depth limits
            }
        });
    });

    // Test recursive parser with smaller versions
    let smaller_alternating = {
        let mut json = String::new();
        for i in 0..100 {
            if i % 2 == 0 {
                json.push_str(r#"{"array": ["#);
            } else {
                json.push_str(r#"{"object": {"key": "#);
            }
        }
        json.push_str("\"value\"");
        for i in 0..100 {
            if i % 2 == 0 {
                json.push_str("]}");
            } else {
                json.push_str("}}");
            }
        }
        json
    };

    group.bench_function("recursive_smaller_alternating", |b| {
        b.iter(|| {
            if parse_recursive(black_box(&smaller_alternating), black_box(options.clone())).is_ok()
            {
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_extreme_deep_nesting,
    bench_with_high_depth_limit,
    bench_memory_efficiency,
    bench_pathological_cases
);
criterion_main!(benches);
