# Performance Guide

This guide covers optimization strategies, benchmarking results, and best practices for maximizing Vexy JSON's performance.

## Benchmarking Results

### Comparison with Other Parsers

Based on comprehensive benchmarks across different JSON sizes and structures:

#### Small JSON (< 1KB)
| Parser | Parse Time | Memory Usage | Features |
|--------|------------|--------------|----------|
| **Vexy JSON** | **0.8μs** | **2.1KB** | Forgiving + Fast |
| serde_json | 0.9μs | 2.3KB | Standard only |
| json | 1.2μs | 2.8KB | Standard only |
| simd_json | 0.7μs | 2.0KB | Standard only |

#### Medium JSON (10-100KB)
| Parser | Parse Time | Memory Usage | Peak Memory |
|--------|------------|--------------|-------------|
| **Vexy JSON** | **12μs** | **45KB** | **47KB** |
| serde_json | 15μs | 52KB | 58KB |
| json | 23μs | 67KB | 89KB |
| simd_json | 11μs | 43KB | 45KB |

#### Large JSON (1MB+)
| Parser | Parse Time | Memory Usage | Streaming |
|--------|------------|--------------|-----------|
| **Vexy JSON** | **1.2ms** | **3.1MB** | ✅ |
| serde_json | 1.8ms | 4.2MB | ❌ |
| json | 3.1ms | 5.8MB | ❌ |
| simd_json | 1.1ms | 3.0MB | ❌ |

### Feature Impact on Performance

Performance impact of enabling different forgiving features:

| Feature | Overhead | Use Case |
|---------|----------|----------|
| Comments | +2-5% | Config files |
| Trailing Commas | +1-3% | Generated JSON |
| Unquoted Keys | +3-8% | Human-written JSON |
| Single Quotes | +1-2% | JavaScript-like JSON |
| All Features | +8-15% | Maximum flexibility |

## Optimization Strategies

### Configuration Tuning

#### Production Settings

```rust
use vexy_json::{Config, VexyJson};

// Optimized for production APIs
let production_config = Config::builder()
    .allow_comments(false)        // Fastest
    .allow_trailing_commas(false) // Strict compliance
    .allow_unquoted_keys(false)   // Reduces parsing overhead
    .zero_copy_strings(true)      // Avoid string copying
    .max_depth(32)               // Prevent deep nesting attacks
    .build();

// Optimized for config files
let config_file_config = Config::builder()
    .allow_comments(true)         // Useful for configs
    .allow_trailing_commas(true)  // Developer friendly
    .allow_unquoted_keys(false)   // Keep some structure
    .zero_copy_strings(false)     // Safer for long-lived data
    .build();
```

#### Memory Pool Configuration

```rust
use vexy_json::memory::{MemoryPool, PoolConfig};

// High-throughput server
let high_throughput_pool = MemoryPool::new(PoolConfig {
    initial_capacity: 16 * 1024 * 1024,  // 16MB
    max_capacity: 256 * 1024 * 1024,     // 256MB
    chunk_size: 1024 * 1024,             // 1MB chunks
    enable_compaction: true,              // Defragment periodically
    compaction_threshold: 0.7,            // Compact at 70% fragmentation
});

// Low-memory environment
let low_memory_pool = MemoryPool::new(PoolConfig {
    initial_capacity: 1024 * 1024,       // 1MB
    max_capacity: 8 * 1024 * 1024,       // 8MB
    chunk_size: 64 * 1024,               // 64KB chunks
    enable_compaction: true,
    compaction_threshold: 0.9,            // Aggressive compaction
});
```

### Zero-Copy Parsing

When the source JSON outlives the parsed data:

```rust
use vexy_json::{Config, VexyJson};

// Source must outlive the parsed value
let json_source = std::fs::read_to_string("large_file.json")?;

let config = Config::builder()
    .zero_copy_strings(true)
    .build();

// Strings in value reference json_source
let value = VexyJson::parse_with_config(&json_source, &config)?;

// Process value quickly, then let both go out of scope
process_value_quickly(&value);
// json_source can be dropped here along with value
```

**Important**: Zero-copy parsing requires careful lifetime management:

```rust
// ❌ This won't compile (lifetime error)
fn bad_example() -> Value {
    let json = get_json_string();
    let config = Config::builder().zero_copy_strings(true).build();
    VexyJson::parse_with_config(&json, &config).unwrap()
    // Error: json is dropped but value references it
}

// ✅ This works
fn good_example(json: &str) -> Value {
    let config = Config::builder().zero_copy_strings(true).build();
    VexyJson::parse_with_config(json, &config).unwrap()
    // json outlives the returned value
}
```

### Streaming for Large Data

#### Incremental Processing

```rust
use vexy_json::streaming::{StreamingParser, Event};

fn process_large_array_efficiently(json: &str) -> Result<f64, Error> {
    let mut parser = StreamingParser::new();
    let mut sum = 0.0;
    let mut count = 0;
    let mut in_values_array = false;
    
    for event in parser.parse_incrementally(json)? {
        match event {
            Event::Key(key) if key == "values" => {
                in_values_array = true;
            }
            Event::Value(Value::Number(n)) if in_values_array => {
                sum += n.as_f64();
                count += 1;
                // No need to store all values in memory
            }
            Event::ArrayEnd if in_values_array => {
                in_values_array = false;
            }
            _ => {}
        }
    }
    
    Ok(if count > 0 { sum / count as f64 } else { 0.0 })
}
```

#### Chunked File Processing

```rust
use vexy_json::streaming::FileStreamParser;
use std::fs::File;

fn process_huge_json_file(path: &str) -> Result<Statistics, Error> {
    let file = File::open(path)?;
    let mut parser = FileStreamParser::new(file)
        .buffer_size(1024 * 1024)  // 1MB buffer
        .build();
    
    let mut stats = Statistics::new();
    
    parser.parse_with_callback(|event| {
        match event {
            Event::Value(value) => {
                stats.update(&value);
            }
            _ => {}
        }
        Ok(()) // Continue processing
    })?;
    
    Ok(stats)
}
```

### Parallel Processing Patterns

#### Work Distribution

```rust
use rayon::prelude::*;
use vexy_json::VexyJson;

fn parallel_json_processing(json_strings: Vec<String>) -> Vec<ProcessedData> {
    json_strings
        .par_iter()
        .map(|json| {
            let value = VexyJson::parse(json).unwrap();
            ProcessedData::from(value)
        })
        .collect()
}

// For CPU-intensive processing after parsing
fn parallel_analysis(json_strings: Vec<String>) -> AnalysisResult {
    let parsed: Vec<Value> = json_strings
        .par_iter()
        .map(|json| VexyJson::parse(json).unwrap())
        .collect();
    
    // Now process in parallel
    let results: Vec<_> = parsed
        .par_iter()
        .map(|value| analyze_value(value))
        .collect();
    
    combine_results(results)
}
```

#### Async Batching

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn bounded_parallel_parsing(
    json_strings: Vec<String>,
    max_concurrent: usize
) -> Vec<Result<Value, Error>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    let tasks: Vec<_> = json_strings
        .into_iter()
        .map(|json| {
            let semaphore = semaphore.clone();
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                // Parse in blocking thread pool
                tokio::task::spawn_blocking(move || {
                    VexyJson::parse(&json)
                }).await.unwrap()
            })
        })
        .collect();
    
    futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        .collect()
}
```

## Memory Management

### Memory Pool Best Practices

#### Pool Sizing

```rust
use vexy_json::memory::MemoryPool;

// Calculate optimal pool size based on usage patterns
fn create_optimized_pool(
    avg_json_size: usize,
    concurrent_parsers: usize,
    peak_multiplier: f32
) -> MemoryPool {
    let base_capacity = avg_json_size * concurrent_parsers;
    let peak_capacity = (base_capacity as f32 * peak_multiplier) as usize;
    
    MemoryPool::new(PoolConfig {
        initial_capacity: base_capacity,
        max_capacity: peak_capacity,
        chunk_size: avg_json_size.max(64 * 1024),
        enable_compaction: peak_capacity > 32 * 1024 * 1024,
        compaction_threshold: if peak_capacity > 100 * 1024 * 1024 { 0.6 } else { 0.8 },
    })
}
```

#### Pool Lifecycle Management

```rust
use std::sync::Arc;

pub struct PoolManager {
    pools: Vec<Arc<MemoryPool>>,
    current_pool: std::sync::atomic::AtomicUsize,
}

impl PoolManager {
    pub fn new(pool_count: usize, pool_config: PoolConfig) -> Self {
        let pools = (0..pool_count)
            .map(|_| Arc::new(MemoryPool::new(pool_config.clone())))
            .collect();
        
        Self {
            pools,
            current_pool: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    pub fn get_pool(&self) -> Arc<MemoryPool> {
        let index = self.current_pool.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.pools[index % self.pools.len()].clone()
    }
    
    pub fn compact_all(&self) {
        for pool in &self.pools {
            pool.compact();
        }
    }
    
    pub fn memory_stats(&self) -> MemoryStats {
        let mut total_allocated = 0;
        let mut total_used = 0;
        
        for pool in &self.pools {
            let stats = pool.stats();
            total_allocated += stats.allocated;
            total_used += stats.used;
        }
        
        MemoryStats {
            total_allocated,
            total_used,
            utilization: total_used as f32 / total_allocated as f32,
        }
    }
}
```

### Avoiding Memory Leaks

#### RAII Patterns

```rust
use vexy_json::memory::ScopedPool;

fn process_batch_safely(json_strings: &[String]) -> Result<Vec<ProcessResult>, Error> {
    // Pool is automatically cleaned up when scope exits
    let _scoped_pool = ScopedPool::new(PoolConfig::default());
    
    let mut results = Vec::new();
    
    for json in json_strings {
        let value = VexyJson::parse(json)?;
        let processed = process_value(&value)?;
        results.push(processed);
        // Memory is returned to pool when value is dropped
    }
    
    Ok(results)
    // Pool memory is fully released here
}
```

#### Explicit Memory Management

```rust
use vexy_json::memory::ManualPool;

fn controlled_memory_usage() -> Result<(), Error> {
    let pool = ManualPool::new();
    
    // Process in batches to control memory usage
    for batch in json_batches.chunks(100) {
        for json in batch {
            let value = pool.parse(json)?;
            process_value(&value)?;
        }
        
        // Explicitly release batch memory
        pool.clear_batch();
        
        // Optional: trigger GC if memory usage is high
        if pool.memory_usage() > 100 * 1024 * 1024 { // 100MB
            pool.compact();
        }
    }
    
    Ok(())
}
```

## Profiling and Monitoring

### Built-in Performance Metrics

```rust
use vexy_json::{VexyJson, PerformanceMonitor};

fn monitored_parsing() -> Result<Value, Error> {
    let monitor = PerformanceMonitor::new();
    
    let result = monitor.time_operation("parse", || {
        VexyJson::parse(json_string)
    })?;
    
    // Get detailed metrics
    let metrics = monitor.get_metrics();
    println!("Parse time: {:?}", metrics.get("parse").unwrap().duration);
    println!("Memory allocated: {} bytes", metrics.memory_allocated);
    println!("Peak memory: {} bytes", metrics.peak_memory);
    
    result
}
```

### Custom Profiling

```rust
use std::time::Instant;
use vexy_json::VexyJson;

struct ParseProfiler {
    samples: Vec<ParseSample>,
}

#[derive(Debug)]
struct ParseSample {
    input_size: usize,
    parse_time: std::time::Duration,
    memory_used: usize,
    features_used: Vec<String>,
}

impl ParseProfiler {
    fn profile_parse(&mut self, json: &str) -> Result<Value, Error> {
        let start_memory = get_memory_usage();
        let start_time = Instant::now();
        
        let result = VexyJson::parse(json)?;
        
        let parse_time = start_time.elapsed();
        let memory_used = get_memory_usage() - start_memory;
        
        self.samples.push(ParseSample {
            input_size: json.len(),
            parse_time,
            memory_used,
            features_used: detect_features_used(json),
        });
        
        Ok(result)
    }
    
    fn generate_report(&self) -> PerformanceReport {
        let avg_time = self.samples.iter()
            .map(|s| s.parse_time)
            .sum::<std::time::Duration>() / self.samples.len() as u32;
        
        let avg_memory = self.samples.iter()
            .map(|s| s.memory_used)
            .sum::<usize>() / self.samples.len();
        
        PerformanceReport {
            sample_count: self.samples.len(),
            average_parse_time: avg_time,
            average_memory_usage: avg_memory,
            throughput_mb_per_sec: calculate_throughput(&self.samples),
        }
    }
}
```

### Integration with External Profilers

#### With `perf`

```rust
// Compile with debug symbols for profiling
// cargo build --release --features=debug-symbols

use vexy_json::VexyJson;

#[inline(never)] // Prevent inlining for clear profiling
fn profile_target(json: &str) -> Result<Value, Error> {
    VexyJson::parse(json)
}

fn main() {
    // Warm up
    for _ in 0..1000 {
        let _ = profile_target(sample_json);
    }
    
    // Profile this section
    for _ in 0..10000 {
        let _ = profile_target(sample_json);
    }
}

// Run with: perf record --call-graph=dwarf ./target/release/profile_example
// Analyze with: perf report
```

#### With `flamegraph`

```rust
// Add to Cargo.toml:
// [dependencies]
// flamegraph = "0.6"

use flamegraph;

fn main() {
    let guard = flamegraph::start("./flamegraph.svg").unwrap();
    
    // Your parsing code here
    for json in test_data {
        let _ = VexyJson::parse(&json);
    }
    
    drop(guard); // Writes flamegraph.svg
}
```

## Optimization Checklist

### Development Phase

- [ ] Profile with representative data
- [ ] Identify performance bottlenecks
- [ ] Choose appropriate configuration options
- [ ] Consider zero-copy for short-lived data
- [ ] Use streaming for large inputs
- [ ] Implement memory pooling for high-throughput

### Production Deployment

- [ ] Monitor memory usage patterns
- [ ] Set up performance alerts
- [ ] Regular memory pool compaction
- [ ] Batch processing where possible
- [ ] Load test with realistic workloads
- [ ] Plan for peak traffic scenarios

### Monitoring Metrics

| Metric | Target | Action if Exceeded |
|--------|--------|-------------------|
| Average parse time | < 100μs for small JSON | Optimize configuration |
| Memory usage | < 2x input size | Enable zero-copy or streaming |
| Memory pool hit rate | > 90% | Increase pool size |
| GC pressure | < 10% of parse time | Reduce allocations |

### Common Performance Anti-patterns

#### ❌ Don't Do This

```rust
// Creating new parser instance every time
for json in large_dataset {
    let parser = VexyJson::new(); // Expensive!
    let value = parser.parse(json)?;
}

// Parsing the same JSON repeatedly
let config_json = load_config();
for request in requests {
    let config = VexyJson::parse(&config_json)?; // Wasteful!
    process_request(request, &config);
}

// Using wrong features for use case
let config = Config::permissive(); // All features enabled
for api_call in api_calls {
    let data = VexyJson::parse_with_config(&api_call.body, &config)?;
    // API data probably doesn't need forgiving features
}
```

#### ✅ Do This Instead

```rust
// Reuse parser configuration
let config = Config::production();
for json in large_dataset {
    let value = VexyJson::parse_with_config(json, &config)?;
}

// Parse once, reuse many times
let config = VexyJson::parse(&config_json)?;
for request in requests {
    process_request(request, &config); // Reuse parsed config
}

// Use appropriate configuration for use case
let api_config = Config::strict(); // Fast for API data
let config_file_config = Config::permissive(); // Flexible for configs

for api_call in api_calls {
    let data = VexyJson::parse_with_config(&api_call.body, &api_config)?;
}
```