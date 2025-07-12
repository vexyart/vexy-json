// this_file: benches/lexer_microbenchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use vexy_json_core::lexer::{FastLexer, JsonLexer, LexerConfig, LexerMode};

fn bench_tokenize_simple_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize simple json");

    let inputs = vec![
        ("empty object", "{}"),
        ("single key-value", r#"{"key": "value"}"#),
        ("multiple values", r#"{"a": 1, "b": true, "c": null}"#),
        ("simple array", "[1, 2, 3, 4, 5]"),
        ("mixed array", r#"[1, "two", true, null]"#),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("lexer", name), &input, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_tokenize_with_comments(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize with comments");

    let inputs = vec![
        ("single line comment", r#"{"key": "value"} // comment"#),
        ("multi line comment", r#"{"key": /* comment */ "value"}"#),
        (
            "mixed comments",
            r#"{
            // Single line comment
            "key1": "value1",
            /* Multi-line
               comment */
            "key2": "value2" // End comment
        }"#,
        ),
        (
            "nested comments",
            r#"{
            "data": {
                // Nested comment
                "nested": /* inline */ "value"
            }
        }"#,
        ),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("lexer", name), &input, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_tokenize_unquoted_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize unquoted keys");

    let inputs = vec![
        ("simple unquoted", r#"{key: "value"}"#),
        ("multiple unquoted", r#"{key1: "value1", key2: "value2"}"#),
        (
            "mixed quoted/unquoted",
            r#"{key1: "value1", "key2": "value2"}"#,
        ),
        ("nested unquoted", r#"{outer: {inner: "value"}}"#),
        (
            "special chars",
            r#"{$key: "value", _key: "value", key123: "value"}"#,
        ),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("lexer", name), &input, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_tokenize_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize numbers");

    let inputs = vec![
        ("integers", "[1, 42, 123, 0, -1, -42]"),
        ("floats", "[1.0, 3.14, -2.718, 0.0, 0.123]"),
        ("scientific", "[1e10, 1.23e-4, -4.56e+7, 1E10, 1.23E-4]"),
        ("mixed", "[1, 3.14, 1e10, -42, -2.718, 1.23e-4]"),
        (
            "large numbers",
            "[9007199254740991, 1.7976931348623157e+308, 5e-324]",
        ),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("lexer", name), &input, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_tokenize_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize strings");

    let long_strings = format!(
        r#"[{}]"#,
        (0..100)
            .map(|i| format!(r#""string_{}""#, i))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let inputs = vec![
        ("simple strings", r#"["hello", "world", "test"]"#),
        (
            "escaped strings",
            r#"["hello\nworld", "tab\there", "quote\"here"]"#,
        ),
        ("unicode strings", r#"["Hello 世界", "🚀✨🎉", "café"]"#),
        (
            "unicode escapes",
            r#"["\u0048\u0065\u006C\u006C\u006F", "\u4E16\u754C"]"#,
        ),
        ("mixed quotes", r#"["double", 'single', "mixed\"quotes"]"#),
        ("long strings", &long_strings),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("lexer", name), &input, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_tokenize_whitespace(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize whitespace");

    let inputs = vec![
        ("minimal whitespace", r#"{"key":"value"}"#),
        ("normal whitespace", r#"{ "key": "value" }"#),
        ("heavy whitespace", r#"  {   "key"   :   "value"   }  "#),
        ("mixed whitespace", "{\n\t\"key\": \"value\"\n}"),
        (
            "newlines as separators",
            "{\n\"key1\": \"value1\"\n\"key2\": \"value2\"\n}",
        ),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("lexer", name), &input, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_lexer_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer with options");

    let json = r#"{
        // Comment
        key: 'value',
        items: [1, 2, 3,],
        nested: {
            unquoted: true,
        }
    }"#;

    let options = vec![
        ("default", LexerConfig::default()),
        (
            "strict",
            LexerConfig {
                mode: LexerMode::Strict,
                ..Default::default()
            },
        ),
        (
            "forgiving",
            LexerConfig {
                mode: LexerMode::Forgiving,
                ..Default::default()
            },
        ),
    ];

    for (name, opts) in options {
        group.bench_with_input(BenchmarkId::new("options", name), &json, |b, json| {
            b.iter(|| {
                let mut lexer = FastLexer::new(black_box(json), opts.clone());
                let mut tokens = Vec::new();
                while let Ok(token) = lexer.next_token() {
                    tokens.push(token);
                    if lexer.is_eof() {
                        break;
                    }
                }
                tokens
            })
        });
    }

    group.finish();
}

fn bench_lexer_performance_characteristics(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer performance");

    // Test performance with different input sizes
    let sizes = vec![100, 1000, 10000];

    for size in sizes {
        let json = format!(
            "{{{}}}",
            (0..size)
                .map(|i| format!(r#""key{}": "value{}""#, i, i))
                .collect::<Vec<_>>()
                .join(", ")
        );

        group.bench_with_input(
            BenchmarkId::new("size", format!("{}_keys", size)),
            &json,
            |b, json| {
                b.iter(|| {
                    let mut lexer = FastLexer::new(black_box(json), LexerConfig::default());
                    let mut token_count = 0;
                    while let Ok(_) = lexer.next_token() {
                        token_count += 1;
                        if lexer.is_eof() {
                            break;
                        }
                    }
                    token_count
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    lexer_benches,
    bench_tokenize_simple_json,
    bench_tokenize_with_comments,
    bench_tokenize_unquoted_keys,
    bench_tokenize_numbers,
    bench_tokenize_strings,
    bench_tokenize_whitespace,
    bench_lexer_options,
    bench_lexer_performance_characteristics
);
criterion_main!(lexer_benches);
