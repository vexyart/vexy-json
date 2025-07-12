// this_file: benches/parser_microbenchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use vexy_json::{parse, parse_with_options, ParserOptions};
use vexy_json_core::lexer::{FastLexer, JsonLexer, LexerConfig};

fn bench_parse_small_object(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse small object");

    let inputs = vec![
        ("empty", "{}"),
        ("single key-value", r#"{"key": "value"}"#),
        ("two keys", r#"{"key1": "value1", "key2": "value2"}"#),
        (
            "mixed types",
            r#"{"str": "text", "num": 42, "bool": true, "null": null}"#,
        ),
        ("nested small", r#"{"outer": {"inner": "value"}}"#),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("parser", name), &input, |b, json| {
            b.iter(|| parse(black_box(json)))
        });
    }

    group.finish();
}

fn bench_parse_large_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse large array");

    // Create arrays of different sizes
    let arrays = vec![
        (
            "10 elements",
            (0..10)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "100 elements",
            (0..100)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "1000 elements",
            (0..1000)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "mixed types 100",
            (0..100)
                .map(|i| match i % 4 {
                    0 => i.to_string(),
                    1 => format!("\"string_{i}\""),
                    2 => "true".to_string(),
                    _ => "null".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
    ];

    for (name, elements) in arrays {
        let json = format!("[{elements}]");
        group.bench_with_input(BenchmarkId::new("parser", name), &json, |b, json| {
            b.iter(|| parse(black_box(json)))
        });
    }

    group.finish();
}

fn bench_parse_deeply_nested(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse deeply nested");

    let depths = vec![5, 10, 20, 50];

    for depth in depths {
        // Create nested object
        let mut nested_obj = String::new();
        for i in 0..depth {
            nested_obj.push_str(&format!("{{\"level_{i}\": "));
        }
        nested_obj.push_str("\"deep_value\"");
        for _ in 0..depth {
            nested_obj.push('}');
        }

        // Create nested array
        let mut nested_arr = String::new();
        for _ in 0..depth {
            nested_arr.push('[');
        }
        nested_arr.push_str("\"deep_value\"");
        for _ in 0..depth {
            nested_arr.push(']');
        }

        group.bench_with_input(
            BenchmarkId::new("nested_object", format!("depth_{depth}")),
            &nested_obj,
            |b, json| b.iter(|| parse(black_box(json))),
        );

        group.bench_with_input(
            BenchmarkId::new("nested_array", format!("depth_{depth}")),
            &nested_arr,
            |b, json| b.iter(|| parse(black_box(json))),
        );
    }

    group.finish();
}

fn bench_parse_with_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse with options");

    let json = r#"{
        // This is a comment
        unquoted_key: "value",
        "quoted_key": 'single quotes',
        "array": [1, 2, 3,],
        "nested": {
            another_key: true,
        },
    }"#;

    let options = vec![
        (
            "strict",
            ParserOptions {
                allow_comments: false,
                allow_trailing_commas: false,
                allow_unquoted_keys: false,
                allow_single_quotes: false,
                implicit_top_level: false,
                newline_as_comma: false,
                ..Default::default()
            },
        ),
        ("forgiving", ParserOptions::default()),
        (
            "comments_only",
            ParserOptions {
                allow_comments: true,
                allow_trailing_commas: false,
                allow_unquoted_keys: false,
                allow_single_quotes: false,
                implicit_top_level: false,
                newline_as_comma: false,
                ..Default::default()
            },
        ),
        (
            "unquoted_keys",
            ParserOptions {
                allow_comments: false,
                allow_trailing_commas: false,
                allow_unquoted_keys: true,
                allow_single_quotes: false,
                implicit_top_level: false,
                newline_as_comma: false,
                ..Default::default()
            },
        ),
    ];

    for (name, opts) in options {
        group.bench_with_input(BenchmarkId::new("options", name), &json, |b, json| {
            b.iter(|| parse_with_options(black_box(json), opts.clone()))
        });
    }

    group.finish();
}

fn bench_parse_component_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse component steps");

    let json = r#"{"name": "test", "value": 42, "active": true, "data": [1, 2, 3]}"#;

    // Benchmark lexer only
    group.bench_with_input(BenchmarkId::new("lexer", "only"), &json, |b, json| {
        b.iter(|| {
            let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
            let mut token_count = 0;
            while lexer.next_token().is_ok() {
                token_count += 1;
                if lexer.is_eof() {
                    break;
                }
            }
            token_count
        })
    });

    // Benchmark full parsing
    group.bench_with_input(BenchmarkId::new("full", "parsing"), &json, |b, json| {
        b.iter(|| parse(black_box(json)))
    });

    group.finish();
}

fn bench_parse_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse error handling");

    let deeply_nested = {
        let mut s = (0..200).fold(String::new(), |mut s, _| {
            s.push_str("{\"a\":");
            s
        });
        s.push('1');
        s.push_str(&"}".repeat(200));
        s
    };

    let invalid_inputs = vec![
        ("unterminated_string", r#"{"key": "unterminated"#),
        ("missing_comma", r#"{"key1": "value1" "key2": "value2"}"#),
        ("trailing_comma_strict", r#"{"key": "value",}"#),
        ("unquoted_key_strict", r#"{key: "value"}"#),
        ("invalid_number", r#"{"key": 123.45.67}"#),
        ("deeply_nested", &deeply_nested),
    ];

    for (name, input) in invalid_inputs {
        group.bench_with_input(BenchmarkId::new("error", name), &input, |b, json| {
            b.iter(|| {
                let _ = parse(black_box(json));
            })
        });
    }

    group.finish();
}

fn bench_parse_repair_mode(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse repair mode");

    let broken_json = r#"{"incomplete": "missing quote, "trailing": "comma",}"#;

    let repair_options = vec![
        (
            "no_repair",
            ParserOptions {
                enable_repair: false,
                ..Default::default()
            },
        ),
        (
            "repair_enabled",
            ParserOptions {
                enable_repair: true,
                max_repairs: 10,
                fast_repair: false,
                ..Default::default()
            },
        ),
        (
            "fast_repair",
            ParserOptions {
                enable_repair: true,
                max_repairs: 10,
                fast_repair: true,
                ..Default::default()
            },
        ),
    ];

    for (name, opts) in repair_options {
        group.bench_with_input(BenchmarkId::new("repair", name), &broken_json, |b, json| {
            b.iter(|| parse_with_options(black_box(json), opts.clone()))
        });
    }

    group.finish();
}

fn bench_parse_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse memory efficiency");

    let sizes = vec![("1KB", 1024), ("10KB", 10 * 1024), ("100KB", 100 * 1024)];

    for (name, size) in sizes {
        // Create a large JSON with many small objects
        let num_objects = size / 50; // Rough estimate
        let json = format!(
            "{{{}}}",
            (0..num_objects)
                .map(|i| format!(r#""key{i}": "value{i}", "num{i}": {i}"#))
                .collect::<Vec<_>>()
                .join(", ")
        );

        group.bench_with_input(BenchmarkId::new("size", name), &json, |b, json| {
            b.iter(|| parse(black_box(json)))
        });
    }

    group.finish();
}

fn bench_parse_string_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse string heavy");

    let string_types = vec![
        (
            "short_strings",
            (0..100)
                .map(|i| format!(r#""str{i}": "value{i}""#))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "long_strings",
            (0..10)
                .map(|i| format!(r#""str{}": "{}""#, i, "x".repeat(100)))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "unicode_strings",
            (0..50)
                .map(|i| format!(r#""str{i}": "Hello 世界 {i} 🌍""#))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "escaped_strings",
            (0..50)
                .map(|i| format!(r#""str{i}": "line1\nline2\ttab\"quote\\slash""#))
                .collect::<Vec<_>>()
                .join(", "),
        ),
    ];

    for (name, content) in string_types {
        let json = format!("{{{content}}}");
        group.bench_with_input(BenchmarkId::new("strings", name), &json, |b, json| {
            b.iter(|| parse(black_box(json)))
        });
    }

    group.finish();
}

criterion_group!(
    parser_benches,
    bench_parse_small_object,
    bench_parse_large_array,
    bench_parse_deeply_nested,
    bench_parse_with_options,
    bench_parse_component_steps,
    bench_parse_error_handling,
    bench_parse_repair_mode,
    bench_parse_memory_efficiency,
    bench_parse_string_heavy
);
criterion_main!(parser_benches);
