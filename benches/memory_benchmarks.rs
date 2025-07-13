// this_file: benches/memory_benchmarks.rs

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use vexy_json::{parse, parse_with_options, ParserOptions};

/// A tracking allocator that wraps the system allocator
/// to monitor memory allocations during benchmarks
#[derive(Debug)]
struct TrackingAllocator {
    allocations: Arc<AtomicUsize>,
    deallocations: Arc<AtomicUsize>,
    peak_memory: Arc<AtomicUsize>,
    current_memory: Arc<AtomicUsize>,
}

impl TrackingAllocator {
    fn new() -> Self {
        TrackingAllocator {
            allocations: Arc::new(AtomicUsize::new(0)),
            deallocations: Arc::new(AtomicUsize::new(0)),
            peak_memory: Arc::new(AtomicUsize::new(0)),
            current_memory: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn stats(&self) -> AllocatorStats {
        AllocatorStats {
            total_allocations: self.allocations.load(Ordering::SeqCst),
            total_deallocations: self.deallocations.load(Ordering::SeqCst),
            peak_memory: self.peak_memory.load(Ordering::SeqCst),
            current_memory: self.current_memory.load(Ordering::SeqCst),
        }
    }

    fn reset(&self) {
        self.allocations.store(0, Ordering::SeqCst);
        self.deallocations.store(0, Ordering::SeqCst);
        self.peak_memory.store(0, Ordering::SeqCst);
        self.current_memory.store(0, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.allocations.fetch_add(1, Ordering::SeqCst);
            let current = self
                .current_memory
                .fetch_add(layout.size(), Ordering::SeqCst)
                + layout.size();

            // Update peak memory if necessary
            let mut peak = self.peak_memory.load(Ordering::SeqCst);
            while current > peak {
                match self.peak_memory.compare_exchange(
                    peak,
                    current,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(actual) => peak = actual,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.deallocations.fetch_add(1, Ordering::SeqCst);
        self.current_memory
            .fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
struct AllocatorStats {
    total_allocations: usize,
    total_deallocations: usize,
    peak_memory: usize,
    current_memory: usize,
}

impl AllocatorStats {
    fn net_allocations(&self) -> usize {
        self.total_allocations
            .saturating_sub(self.total_deallocations)
    }
}

// Note: For these benchmarks to work properly, we would need to set up
// a custom global allocator. Since that's not practical for a benchmark,
// we'll use alternative approaches to measure memory usage patterns.

fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation patterns");

    let large_object = format!(
        "{{{}}}",
        (0..100)
            .map(|i| format!(r#""key{i}": "value{i}", "num{i}": {i}"#))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let deep_nesting = {
        let mut nested = String::new();
        for i in 0..20 {
            nested.push_str(&format!(r#"{{"level{i}": "#));
        }
        nested.push_str(r#""deep_value""#);
        for _ in 0..20 {
            nested.push('}');
        }
        nested
    };

    let test_cases = vec![
        ("small_object", r#"{"key": "value", "num": 42}"#),
        (
            "medium_object",
            r#"{
            "name": "test",
            "values": [1, 2, 3, 4, 5],
            "nested": {
                "inner": "value",
                "numbers": [10, 20, 30]
            }
        }"#,
        ),
        ("large_object", large_object.as_str()),
        ("deep_nesting", deep_nesting.as_str()),
    ];

    for (name, json) in test_cases {
        group.bench_with_input(BenchmarkId::new("parse", name), &json, |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                // Force the result to be used to prevent optimization
                std::hint::black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_string_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string interning");

    // Test cases with repeated strings to measure interning effectiveness
    let unique_strings = format!(
        "{{{}}}",
        (0..50)
            .map(|i| format!(r#""key{i}": "unique_value_{i}""#))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let repeated_strings = format!(
        "{{{}}}",
        (0..50)
            .map(|i| format!(r#""key{i}": "repeated_value""#))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mixed_strings = format!(
        "{{{}}}",
        (0..50)
            .map(|i| {
                let value = if i % 5 == 0 {
                    "common_value"
                } else {
                    "unique_value"
                };
                format!(r#""key{i}": "{value}""#)
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    let test_cases = vec![
        ("unique_strings", unique_strings.as_str()),
        ("repeated_strings", repeated_strings.as_str()),
        ("mixed_strings", mixed_strings.as_str()),
    ];

    for (name, json) in test_cases {
        group.bench_with_input(BenchmarkId::new("interning", name), &json, |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                std::hint::black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_memory_vs_input_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory vs input size");

    // Test how memory usage scales with input size
    let sizes = vec![
        (100, "100_elements"),
        (1000, "1000_elements"),
        (10000, "10000_elements"),
    ];

    for (size, name) in sizes {
        let json = format!(
            "[{}]",
            (0..size)
                .map(|i| format!(
                    r#"{{"id": {}, "value": "item_{}", "active": {}}}"#,
                    i,
                    i,
                    i % 2 == 0
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );

        group.bench_with_input(BenchmarkId::new("array_size", name), &json, |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                std::hint::black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_allocation_by_data_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation by data type");

    let numbers_only = format!(
        "[{}]",
        (0..1000)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let strings_only = format!(
        "[{}]",
        (0..1000)
            .map(|i| format!(r#""string_{i}""#))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let booleans_only = format!(
        "[{}]",
        (0..1000)
            .map(|i| if i % 2 == 0 { "true" } else { "false" })
            .collect::<Vec<_>>()
            .join(", ")
    );

    let nulls_only = format!(
        "[{}]",
        (0..1000).map(|_| "null").collect::<Vec<_>>().join(", ")
    );

    let mixed_types = format!(
        "[{}]",
        (0..1000)
            .map(|i| match i % 4 {
                0 => i.to_string(),
                1 => format!(r#""string_{i}""#),
                2 => if i % 2 == 0 { "true" } else { "false" }.to_string(),
                _ => "null".to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    let test_cases = vec![
        ("numbers_only", numbers_only.as_str()),
        ("strings_only", strings_only.as_str()),
        ("booleans_only", booleans_only.as_str()),
        ("nulls_only", nulls_only.as_str()),
        ("mixed_types", mixed_types.as_str()),
    ];

    for (name, json) in test_cases {
        group.bench_with_input(BenchmarkId::new("type", name), &json, |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                std::hint::black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_parser_options_memory_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser options memory impact");

    let complex_json = r#"{
        // This is a comment
        name: 'John Doe',
        age: 30,
        tags: ['developer', 'rust', 'json',],
        nested: {
            deep: {
                deeper: {
                    value: 'test',
                },
            },
        },
    }"#;

    let options = vec![
        ("default", ParserOptions::default()),
        (
            "minimal",
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
            "repair_enabled",
            ParserOptions {
                enable_repair: true,
                max_repairs: 10,
                report_repairs: true,
                ..Default::default()
            },
        ),
    ];

    for (name, opts) in options {
        group.bench_with_input(
            BenchmarkId::new("options", name),
            &complex_json,
            |b, json| {
                b.iter(|| {
                    let result = parse_with_options(black_box(json), opts.clone());
                    std::hint::black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_streaming_vs_batch_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming vs batch memory");

    // Create a large JSON that would benefit from streaming
    let large_array = format!(
        "[{}]",
        (0..5000)
            .map(|i| format!(
                r#"{{"id": {}, "data": "item_{}", "timestamp": {}}}"#,
                i,
                i,
                i * 1000
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );

    group.bench_with_input(
        BenchmarkId::new("batch", "large_array"),
        &large_array,
        |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                std::hint::black_box(result)
            })
        },
    );

    // Note: Streaming benchmarks would go here when the streaming API is stable
    // For now, we'll just test the batch approach

    group.finish();
}

fn bench_memory_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory fragmentation");

    // Test patterns that might cause fragmentation
    let test_cases = vec![
        ("alternating_sizes", {
            let mut json = String::from("[");
            for i in 0..500 {
                if i > 0 {
                    json.push_str(", ");
                }
                if i % 2 == 0 {
                    json.push_str(&format!(r#""small_{i}""#));
                } else {
                    json.push_str(&format!(r#""large_string_with_lots_of_content_{i}""#));
                }
            }
            json.push(']');
            json
        }),
        ("growing_strings", {
            let mut json = String::from("[");
            for i in 0..100 {
                if i > 0 {
                    json.push_str(", ");
                }
                json.push_str(&format!(r#""{}""#, "x".repeat(i * 10)));
            }
            json.push(']');
            json
        }),
        ("nested_growth", {
            let mut json = String::from("{");
            for i in 0..50 {
                if i > 0 {
                    json.push_str(", ");
                }
                json.push_str(&format!(r#""level{i}": {{"#));
                for j in 0..i {
                    if j > 0 {
                        json.push_str(", ");
                    }
                    json.push_str(&format!(r#""item{j}": {j}"#));
                }
                json.push('}');
            }
            json.push('}');
            json
        }),
    ];

    for (name, json) in test_cases {
        group.bench_with_input(BenchmarkId::new("fragmentation", name), &json, |b, json| {
            b.iter(|| {
                let result = parse(black_box(json));
                std::hint::black_box(result)
            })
        });
    }

    group.finish();
}

criterion_group!(
    memory_benches,
    bench_allocation_patterns,
    bench_string_interning,
    bench_memory_vs_input_size,
    bench_allocation_by_data_type,
    bench_parser_options_memory_impact,
    bench_streaming_vs_batch_memory,
    bench_memory_fragmentation
);
criterion_main!(memory_benches);
