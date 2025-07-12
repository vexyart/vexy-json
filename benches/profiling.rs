// this_file: benches/profiling.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vexy_json::parse;

fn profiling_heavy_workload(c: &mut Criterion) {
    // Create a large, complex JSON that will stress the parser
    let large_complex_json = generate_complex_json();

    c.bench_function("profiling_heavy_workload", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = parse(black_box(&large_complex_json));
            }
        })
    });
}

fn profiling_forgiving_features(c: &mut Criterion) {
    // Mix of all forgiving features
    let forgiving_json = r#"{
        // Configuration file with comments
        app_name: 'MyApp',
        version: "1.2.3",
        debug: true,
        
        /* Multi-line comment
           describing complex config */
        server: {
            host: 'localhost',
            port: 8080,
            endpoints: [
                '/api/v1',
                '/api/v2',
                '/health', // trailing comma
            ],
        },
        
        database: {
            type: 'postgres',
            url: "postgres://user:pass@localhost/db",
            pool_size: 10,
            timeout: 30000, // milliseconds
        },
        
        features: [
            'feature1',
            'feature2',
            'feature3',
        ], // trailing comma
    }"#;

    c.bench_function("profiling_forgiving_features", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _ = parse(black_box(forgiving_json));
            }
        })
    });
}

fn profiling_memory_allocation(c: &mut Criterion) {
    // Test parsing many small objects to stress allocation
    let small_objects: Vec<String> = (0..1000)
        .map(|i| format!("{{\"id\": {i}, \"name\": \"item{i}\", \"active\": true}}"))
        .collect();

    c.bench_function("profiling_memory_allocation", |b| {
        b.iter(|| {
            for obj in &small_objects {
                let _ = parse(black_box(obj));
            }
        })
    });
}

fn profiling_string_parsing(c: &mut Criterion) {
    // Heavy string parsing workload
    let string_heavy_json = generate_string_heavy_json();

    c.bench_function("profiling_string_parsing", |b| {
        b.iter(|| {
            for _ in 0..200 {
                let _ = parse(black_box(&string_heavy_json));
            }
        })
    });
}

fn generate_complex_json() -> String {
    let mut json = String::from("{");

    // Add many nested objects
    for i in 0..50 {
        json.push_str(&format!(
            r#""section{}": {{
                "id": {},
                "data": [
                    {{"key": "value{}", "number": {}}},
                    {{"key": "value{}", "number": {}}},
                    {{"key": "value{}", "number": {}}}
                ],
                "meta": {{
                    "created": "2023-01-{:02}",
                    "tags": ["tag1", "tag2", "tag3"],
                    "settings": {{
                        "enabled": {},
                        "threshold": {}
                    }}
                }}
            }}"#,
            i,
            i,
            i,
            i * 2,
            i,
            i * 3,
            i,
            i * 4,
            (i % 28) + 1,
            i % 2 == 0,
            i * 10
        ));

        if i < 49 {
            json.push(',');
        }
    }

    json.push('}');
    json
}

fn generate_string_heavy_json() -> String {
    let mut json = String::from("[");

    for i in 0..200 {
        json.push_str(&format!(
            r#"{{
                "title": "This is a long title for item number {} with lots of text",
                "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Item {}",
                "content": "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Content for item {}",
                "metadata": {{
                    "author": "Author Name {}",
                    "category": "Category {}",
                    "tags": ["tag{}", "tag{}", "tag{}"],
                    "url": "https://example.com/item/{}"
                }}
            }}"#,
            i, i, i, i, i % 10, i, (i + 1) % 10, (i + 2) % 10, i
        ));

        if i < 199 {
            json.push(',');
        }
    }

    json.push(']');
    json
}

criterion_group!(
    profiling_benches,
    profiling_heavy_workload,
    profiling_forgiving_features,
    profiling_memory_allocation,
    profiling_string_parsing
);
criterion_main!(profiling_benches);
