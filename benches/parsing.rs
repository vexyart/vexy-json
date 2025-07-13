use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use vexy_json::{parse, parse_with_options, ParserOptions};

fn benchmark_simple_object(c: &mut Criterion) {
    let json = r#"{"name": "John", "age": 30, "active": true}"#;
    c.bench_function("parse simple object", |b| b.iter(|| parse(black_box(json))));
}

fn benchmark_nested_object(c: &mut Criterion) {
    let json = r#"{
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
    c.bench_function("parse nested object", |b| b.iter(|| parse(black_box(json))));
}

fn benchmark_array(c: &mut Criterion) {
    let json = r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#;
    c.bench_function("parse small array", |b| b.iter(|| parse(black_box(json))));
}

fn benchmark_large_array(c: &mut Criterion) {
    let json = format!(
        "[{}]",
        (0..1000)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    c.bench_function("parse large array", |b| b.iter(|| parse(black_box(&json))));
}

fn benchmark_forgiving_features(c: &mut Criterion) {
    // Test with comments
    let json_with_comments = r#"{
        // This is a comment
        "name": "John", // Another comment
        "age": 30,
        /* Multi-line
           comment */
        "active": true
    }"#;
    c.bench_function("parse with comments", |b| {
        b.iter(|| parse(black_box(json_with_comments)))
    });

    // Test with unquoted keys
    let json_unquoted = r#"{
        name: "John",
        age: 30,
        active: true,
        address: {
            street: "123 Main St",
            city: "Boston"
        }
    }"#;
    c.bench_function("parse unquoted keys", |b| {
        b.iter(|| parse(black_box(json_unquoted)))
    });

    // Test with trailing commas
    let json_trailing = r#"{
        "items": [1, 2, 3,],
        "data": {
            "a": 1,
            "b": 2,
        },
    }"#;
    c.bench_function("parse trailing commas", |b| {
        b.iter(|| parse(black_box(json_trailing)))
    });

    // Test with single quotes
    let json_single_quotes = r#"{
        'name': 'John',
        'age': 30,
        'tags': ['developer', 'rust', 'json']
    }"#;
    c.bench_function("parse single quotes", |b| {
        b.iter(|| parse(black_box(json_single_quotes)))
    });

    // Test implicit structures
    let implicit_object = "name: 'John', age: 30, active: true";
    c.bench_function("parse implicit object", |b| {
        b.iter(|| parse(black_box(implicit_object)))
    });

    let implicit_array = "'item1', 'item2', 'item3', 'item4'";
    c.bench_function("parse implicit array", |b| {
        b.iter(|| parse(black_box(implicit_array)))
    });
}

fn benchmark_options_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("options comparison");

    let json_all_features = r#"{
        // Comment here
        name: 'John', // Uses unquoted key and single quotes
        items: [1, 2, 3,], // Trailing comma
        /* Multi-line comment */
        data: { a: 1, b: 2, }
    }"#;

    // Benchmark with all features enabled (default)
    group.bench_with_input(
        BenchmarkId::new("all features", "enabled"),
        &json_all_features,
        |b, json| b.iter(|| parse(black_box(json))),
    );

    // Benchmark with strict JSON mode
    let strict_options = ParserOptions {
        allow_comments: false,
        allow_trailing_commas: false,
        allow_unquoted_keys: false,
        allow_single_quotes: false,
        implicit_top_level: false,
        newline_as_comma: false,
        max_depth: 64,
        enable_repair: false,
        max_repairs: 0,
        fast_repair: false,
        report_repairs: false,
    };

    let json_strict = r#"{"name": "John", "items": [1, 2, 3], "data": {"a": 1, "b": 2}}"#;

    group.bench_with_input(
        BenchmarkId::new("strict mode", "enabled"),
        &json_strict,
        |b, json| b.iter(|| parse_with_options(black_box(json), strict_options.clone())),
    );

    group.finish();
}

fn benchmark_real_world(c: &mut Criterion) {
    // Simulate a configuration file with mixed features
    let config_json = r#"{
        // Application configuration
        app: {
            name: 'MyApp',
            version: '1.2.3',
            debug: true,
        },
        
        // Server settings
        server: {
            host: 'localhost',
            port: 8080,
            ssl: {
                enabled: false,
                cert: '/path/to/cert.pem',
                key: '/path/to/key.pem',
            },
        },
        
        // Database configuration
        database: {
            type: 'postgres',
            host: 'db.example.com',
            port: 5432,
            credentials: {
                user: 'dbuser',
                password: 'secret', // TODO: Use env var
            },
            pool: {
                min: 5,
                max: 20,
                idle_timeout: 30000, // milliseconds
            },
        },
        
        // Feature flags
        features: [
            'new-ui',
            'analytics',
            'beta-api',
        ],
    }"#;

    c.bench_function("parse config file", |b| {
        b.iter(|| parse(black_box(config_json)))
    });
}

fn benchmark_edge_cases(c: &mut Criterion) {
    // Very deeply nested structure
    let mut deep_json = String::from("{\"a\":");
    for _ in 0..50 {
        deep_json.push_str("{\"b\":");
    }
    deep_json.push('1');
    for _ in 0..50 {
        deep_json.push('}');
    }
    deep_json.push('}');

    c.bench_function("parse deeply nested", |b| {
        b.iter(|| parse(black_box(&deep_json)))
    });

    // Large object with many keys
    let large_object = format!(
        "{{{}}}",
        (0..100)
            .map(|i| format!("\"key{i}\": {i}"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    c.bench_function("parse large object", |b| {
        b.iter(|| parse(black_box(&large_object)))
    });
}

criterion_group!(
    benches,
    benchmark_simple_object,
    benchmark_nested_object,
    benchmark_array,
    benchmark_large_array,
    benchmark_forgiving_features,
    benchmark_options_comparison,
    benchmark_real_world,
    benchmark_edge_cases
);
criterion_main!(benches);
