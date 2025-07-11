use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::hint::black_box;
use vexy_json::parse;

fn get_json_data(name: &str) -> String {
    fs::read_to_string(format!("benches/data/{}.json", name)).expect("Unable to read file")
}

fn comparison_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");
    let test_cases = &[
        "simple",
        "array",
        "nested",
        "large_array",
        "deeply_nested",
        "forgiving",
        "config",
    ];

    for &case in test_cases {
        let json_data = get_json_data(case);
        group.bench_with_input(BenchmarkId::new("vexy_json", case), &json_data, |b, data| {
            b.iter(|| parse(black_box(data)))
        });
    }
    group.finish();
}

criterion_group!(benches, comparison_benchmarks);
criterion_main!(benches);
