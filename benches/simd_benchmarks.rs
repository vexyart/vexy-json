use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use vexy_json_core::optimization::simd::*;

fn generate_test_strings() -> Vec<(&'static str, String)> {
    vec![
        ("small_no_escape", "hello world test string".to_string()),
        ("small_with_escape", "hello\\nworld\\ttest".to_string()),
        ("medium_no_escape", "a".repeat(100)),
        (
            "medium_with_escape",
            format!(
                "{}\\n{}\\t{}",
                "test".repeat(25),
                "data".repeat(25),
                "end".repeat(25)
            ),
        ),
        ("large_no_escape", "x".repeat(1000)),
        (
            "large_with_escape",
            format!("{}\\\\{}", "path".repeat(250), "file".repeat(250)),
        ),
        (
            "json_string",
            r#"{"name": "John Doe", "age": 30, "city": "New York"}"#.to_string(),
        ),
        ("unicode_heavy", "Hello 世界 🌍 Здравствуй мир".repeat(10)),
    ]
}

fn benchmark_backslash_detection(c: &mut Criterion) {
    let test_strings = generate_test_strings();
    let mut group = c.benchmark_group("backslash_detection");

    for (name, test_string) in &test_strings {
        group.bench_with_input(BenchmarkId::new("simd", name), test_string, |b, s| {
            b.iter(|| has_backslash_simd(black_box(s)))
        });
    }

    group.finish();
}

fn benchmark_string_validation(c: &mut Criterion) {
    let test_strings = generate_test_strings();
    let mut group = c.benchmark_group("string_validation");

    for (name, test_string) in &test_strings {
        group.bench_with_input(BenchmarkId::new("simd", name), test_string, |b, s| {
            b.iter(|| validate_json_string_simd(black_box(s)))
        });
    }

    group.finish();
}

fn benchmark_whitespace_skipping(c: &mut Criterion) {
    let large_whitespace = " ".repeat(1000);
    let test_cases = vec![
        ("no_whitespace", "hello_world"),
        ("leading_spaces", "    hello world"),
        ("leading_mixed", "\t\n\r hello world"),
        ("all_whitespace", "    \t\n\r    "),
        ("large_whitespace", large_whitespace.as_str()),
    ];

    let mut group = c.benchmark_group("whitespace_skipping");

    for (name, test_string) in test_cases {
        group.bench_with_input(BenchmarkId::new("simd", name), test_string, |b, s| {
            b.iter(|| skip_whitespace_simd(black_box(s)))
        });
    }

    group.finish();
}

fn benchmark_number_parsing(c: &mut Criterion) {
    let test_cases = vec![
        ("small_int", "42"),
        ("negative_int", "-123"),
        ("large_int", "9876543210"),
        ("float", "3.14159"),
        ("scientific", "1.23e-10"),
        ("zero", "0"),
    ];

    let mut group = c.benchmark_group("number_parsing");

    for (name, test_string) in test_cases {
        group.bench_with_input(BenchmarkId::new("simd", name), test_string, |b, s| {
            b.iter(|| parse_number_simd(black_box(s)))
        });
    }

    group.finish();
}

fn benchmark_string_unescaping(c: &mut Criterion) {
    let test_cases = vec![
        ("no_escape", "hello world test string"),
        ("simple_escapes", "hello\\nworld\\ttab\\rreturn"),
        ("quotes", r#"She said \"Hello\" to him"#),
        ("backslashes", "C:\\\\Users\\\\Documents\\\\file.txt"),
        ("unicode", "Hello \\u0041\\u0042\\u0043 World"),
        ("mixed", r#"Line1\\nLine2\\t\"quoted\"\\path\\to\\file"#),
    ];

    let mut group = c.benchmark_group("string_unescaping");

    for (name, test_string) in test_cases {
        group.bench_with_input(BenchmarkId::new("simd", name), test_string, |b, s| {
            b.iter(|| unescape_string_simd(black_box(s)))
        });
    }

    group.finish();
}

criterion_group!(
    simd_benches,
    benchmark_backslash_detection,
    benchmark_string_validation,
    benchmark_whitespace_skipping,
    benchmark_number_parsing,
    benchmark_string_unescaping
);

criterion_main!(simd_benches);
