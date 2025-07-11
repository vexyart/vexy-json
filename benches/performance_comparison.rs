// this_file: benches/performance_comparison.rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use vexy_json::parse as vexy_json_parse;

fn benchmark_parser_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_comparison");

    // Test cases
    let simple_json = r#"{"name": "John", "age": 30, "active": true}"#;
    let nested_json = r#"{
        "user": {
            "name": "John Doe",
            "email": "john@example.com",
            "metadata": {
                "created": "2023-01-01",
                "lastLogin": "2023-12-01",
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        }
    }"#;

    let large_array_json = format!(
        "[{}]",
        (0..1000)
            .map(|i| format!("{{\"id\":{}, \"value\":\"item{}\"}}", i, i))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Benchmark simple JSON
    group.bench_with_input(
        BenchmarkId::new("vexy_json", "simple"),
        &simple_json,
        |b, json| b.iter(|| vexy_json_parse(black_box(json))),
    );

    group.bench_with_input(
        BenchmarkId::new("serde_json", "simple"),
        &simple_json,
        |b, json| b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json))),
    );

    // Benchmark nested JSON
    group.bench_with_input(
        BenchmarkId::new("vexy_json", "nested"),
        &nested_json,
        |b, json| b.iter(|| vexy_json_parse(black_box(json))),
    );

    group.bench_with_input(
        BenchmarkId::new("serde_json", "nested"),
        &nested_json,
        |b, json| b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json))),
    );

    // Benchmark large array
    group.bench_with_input(
        BenchmarkId::new("vexy_json", "large_array"),
        &large_array_json,
        |b, json| b.iter(|| vexy_json_parse(black_box(json))),
    );

    group.bench_with_input(
        BenchmarkId::new("serde_json", "large_array"),
        &large_array_json,
        |b, json| b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json))),
    );

    group.finish();
}

fn benchmark_forgiving_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("forgiving_features");

    // Test vexy_json's forgiving features (serde_json can't parse these)
    let forgiving_json = r#"{
        // Comment
        name: 'John',
        age: 30,
        tags: ['rust', 'json',], // trailing comma
    }"#;

    let standard_json = r#"{"name": "John", "age": 30, "tags": ["rust", "json"]}"#;

    group.bench_with_input(
        BenchmarkId::new("vexy_json", "forgiving"),
        &forgiving_json,
        |b, json| b.iter(|| vexy_json_parse(black_box(json))),
    );

    group.bench_with_input(
        BenchmarkId::new("vexy_json", "standard"),
        &standard_json,
        |b, json| b.iter(|| vexy_json_parse(black_box(json))),
    );

    group.bench_with_input(
        BenchmarkId::new("serde_json", "standard"),
        &standard_json,
        |b, json| b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json))),
    );

    group.finish();
}

fn benchmark_string_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_parsing");

    // Test string-heavy workloads
    let string_heavy = generate_string_heavy_json();

    group.bench_with_input(
        BenchmarkId::new("vexy_json", "string_heavy"),
        &string_heavy,
        |b, json| b.iter(|| vexy_json_parse(black_box(json))),
    );

    group.bench_with_input(
        BenchmarkId::new("serde_json", "string_heavy"),
        &string_heavy,
        |b, json| b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(json))),
    );

    group.finish();
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Many small objects to test allocation patterns
    let small_objects: Vec<String> = (0..100)
        .map(|i| format!(r#"{{"id": {}, "name": "item{}", "active": true}}"#, i, i))
        .collect();

    group.bench_function("vexy_json_small_objects", |b| {
        b.iter(|| {
            for obj in &small_objects {
                let _ = vexy_json_parse(black_box(obj));
            }
        })
    });

    group.bench_function("serde_json_small_objects", |b| {
        b.iter(|| {
            for obj in &small_objects {
                let _ = serde_json::from_str::<serde_json::Value>(black_box(obj));
            }
        })
    });

    group.finish();
}

fn generate_string_heavy_json() -> String {
    let mut json = String::from("[");

    for i in 0..50 {
        json.push_str(&format!(
            r#"{{
                "title": "This is a long title for item number {} with lots of text to parse",
                "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation.",
                "content": "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
                "metadata": {{
                    "author": "Author Name {}",
                    "category": "Category {}",
                    "url": "https://example.com/very/long/url/path/that/requires/significant/string/processing/item/{}"
                }}
            }}"#,
            i, i, i % 10, i
        ));

        if i < 49 {
            json.push(',');
        }
    }

    json.push(']');
    json
}

criterion_group!(
    performance_benches,
    benchmark_parser_comparison,
    benchmark_forgiving_features,
    benchmark_string_parsing,
    benchmark_memory_allocation
);
criterion_main!(performance_benches);
