// this_file: benches/real_world_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use vexy_json::{parse, parse_with_options, ParserOptions};

fn collect_benchmark_files() -> Vec<(String, String, String)> {
    let mut files = Vec::new();

    // Small files (1-10KB)
    if let Ok(entries) = fs::read_dir("bench-data/small") {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let name = entry.file_name().to_string_lossy().to_string();
                files.push(("small".to_string(), name, content));
            }
        }
    }

    // Medium files (10KB-1MB)
    if let Ok(entries) = fs::read_dir("bench-data/medium") {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let name = entry.file_name().to_string_lossy().to_string();
                files.push(("medium".to_string(), name, content));
            }
        }
    }

    // Large files (1MB+) - only if they exist and are reasonable size
    if let Ok(entries) = fs::read_dir("bench-data/large") {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                // Skip files larger than 10MB for benchmarks
                if metadata.len() < 10 * 1024 * 1024 {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        files.push(("large".to_string(), name, content));
                    }
                }
            }
        }
    }

    files
}

fn bench_real_world_parsing(c: &mut Criterion) {
    let files = collect_benchmark_files();

    if files.is_empty() {
        // Create some fallback data if no files exist
        let fallback_files = vec![
            (
                "small".to_string(),
                "config.json".to_string(),
                r#"{"app": {"name": "test", "version": "1.0"}, "debug": true}"#.to_string(),
            ),
            (
                "medium".to_string(),
                "api_response.json".to_string(),
                format!(
                    "{{\"data\": [{}]}}",
                    (0..100)
                        .map(|i| format!(
                            r#"{{"id": {}, "name": "item{}", "value": {}}}"#,
                            i,
                            i,
                            i * 10
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
        ];

        for (category, filename, content) in fallback_files {
            let mut group = c.benchmark_group(format!("real_world_{category}"));
            group.bench_with_input(BenchmarkId::new("parse", filename), &content, |b, json| {
                b.iter(|| parse(black_box(json)))
            });
            group.finish();
        }
        return;
    }

    // Group files by category
    let mut small_files = Vec::new();
    let mut medium_files = Vec::new();
    let mut large_files = Vec::new();

    for (category, name, content) in files {
        match category.as_str() {
            "small" => small_files.push((name, content)),
            "medium" => medium_files.push((name, content)),
            "large" => large_files.push((name, content)),
            _ => {}
        }
    }

    // Benchmark small files
    if !small_files.is_empty() {
        let mut group = c.benchmark_group("real_world_small");
        for (name, content) in small_files {
            group.bench_with_input(BenchmarkId::new("parse", name), &content, |b, json| {
                b.iter(|| parse(black_box(json)))
            });
        }
        group.finish();
    }

    // Benchmark medium files
    if !medium_files.is_empty() {
        let mut group = c.benchmark_group("real_world_medium");
        for (name, content) in medium_files {
            group.bench_with_input(BenchmarkId::new("parse", name), &content, |b, json| {
                b.iter(|| parse(black_box(json)))
            });
        }
        group.finish();
    }

    // Benchmark large files
    if !large_files.is_empty() {
        let mut group = c.benchmark_group("real_world_large");
        // Use smaller sample size for large files
        group.sample_size(20);
        for (name, content) in large_files {
            group.bench_with_input(BenchmarkId::new("parse", name), &content, |b, json| {
                b.iter(|| parse(black_box(json)))
            });
        }
        group.finish();
    }
}

fn bench_parser_options_on_real_data(c: &mut Criterion) {
    let files = collect_benchmark_files();

    if files.is_empty() {
        return;
    }

    let mut group = c.benchmark_group("parser_options_real_data");

    // Take a few representative files
    let test_files: Vec<_> = files.into_iter().take(3).collect();

    let parser_options = vec![
        ("default", ParserOptions::default()),
        (
            "strict",
            ParserOptions {
                allow_comments: false,
                allow_trailing_commas: false,
                allow_unquoted_keys: false,
                allow_single_quotes: false,
                implicit_top_level: false,
                newline_as_comma: false,
                enable_repair: false,
                ..Default::default()
            },
        ),
        (
            "forgiving",
            ParserOptions {
                allow_comments: true,
                allow_trailing_commas: true,
                allow_unquoted_keys: true,
                allow_single_quotes: true,
                implicit_top_level: true,
                newline_as_comma: true,
                enable_repair: false,
                ..Default::default()
            },
        ),
        (
            "repair_enabled",
            ParserOptions {
                enable_repair: true,
                max_repairs: 10,
                ..Default::default()
            },
        ),
    ];

    for (category, filename, content) in test_files {
        for (option_name, options) in &parser_options {
            group.bench_with_input(
                BenchmarkId::new(format!("{category}_{option_name}"), filename.clone()),
                &content,
                |b, json| b.iter(|| parse_with_options(black_box(json), options.clone())),
            );
        }
    }

    group.finish();
}

fn bench_file_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_size_scaling");

    // Generate JSON files of different sizes
    let sizes = vec![
        (1_000, "1KB"),
        (10_000, "10KB"),
        (100_000, "100KB"),
        (1_000_000, "1MB"),
    ];

    for (target_size, name) in sizes {
        // Create JSON with approximately the target size
        let num_items = target_size / 50; // Rough estimate
        let json = format!(
            "{{\"items\": [{}]}}",
            (0..num_items)
                .map(|i| format!(r#"{{"id": {i}, "name": "item{i}", "data": "content{i}", "timestamp": "2023-12-01T12:00:00Z"}}"#))
                .collect::<Vec<_>>()
                .join(", ")
        );

        group.bench_with_input(BenchmarkId::new("size", name), &json, |b, json| {
            b.iter(|| parse(black_box(json)))
        });
    }

    group.finish();
}

fn bench_data_type_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_type_distribution");

    // Different data type distributions
    let distributions = vec![
        ("mostly_strings", generate_mostly_strings_json(1000)),
        ("mostly_numbers", generate_mostly_numbers_json(1000)),
        ("mostly_objects", generate_mostly_objects_json(100)),
        ("mostly_arrays", generate_mostly_arrays_json(100)),
        ("mixed_types", generate_mixed_types_json(500)),
    ];

    for (name, json) in distributions {
        group.bench_with_input(BenchmarkId::new("distribution", name), &json, |b, json| {
            b.iter(|| parse(black_box(json)))
        });
    }

    group.finish();
}

fn generate_mostly_strings_json(count: usize) -> String {
    let items: Vec<String> = (0..count)
        .map(|i| format!(r#""string_{i}_content_with_some_length""#))
        .collect();
    format!("[{}]", items.join(", "))
}

fn generate_mostly_numbers_json(count: usize) -> String {
    let items: Vec<String> = (0..count).map(|i| format!("{}", i as f64 + 0.5)).collect();
    format!("[{}]", items.join(", "))
}

fn generate_mostly_objects_json(count: usize) -> String {
    let items: Vec<String> = (0..count)
        .map(|i| {
            format!(
                r#"{{"id": {}, "name": "item{}", "value": {}, "active": {}}}"#,
                i,
                i,
                i * 10,
                i % 2 == 0
            )
        })
        .collect();
    format!("[{}]", items.join(", "))
}

fn generate_mostly_arrays_json(count: usize) -> String {
    let items: Vec<String> = (0..count)
        .map(|i| format!("[{}, {}, {}]", i, i + 1, i + 2))
        .collect();
    format!("[{}]", items.join(", "))
}

fn generate_mixed_types_json(count: usize) -> String {
    let items: Vec<String> = (0..count)
        .map(|i| match i % 5 {
            0 => format!(r#""string_{i}""#),
            1 => format!("{i}"),
            2 => format!("{}", i % 2 == 0),
            3 => "null".to_string(),
            _ => format!(r#"{{"nested": {i}}}"#),
        })
        .collect();
    format!("[{}]", items.join(", "))
}

criterion_group!(
    real_world_benches,
    bench_real_world_parsing,
    bench_parser_options_on_real_data,
    bench_file_size_scaling,
    bench_data_type_distribution
);
criterion_main!(real_world_benches);
