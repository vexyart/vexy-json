# JSON Transformation

The Vexy JSON library provides powerful JSON transformation capabilities through its `transform` module. This module includes JSON normalization and AST optimization features.

## JSON Normalization

The JSON normalizer provides standardized JSON formatting with various normalization options.

### Basic Usage

```rust
use vexy_json_core::transform::{normalize, normalize_with_options, NormalizerOptions};

// Basic normalization with default options
let json = r#"{"b": 2, "a": 1, "c": null}"#;
let normalized = normalize(json).unwrap();
// Result: {"a": 1, "b": 2, "c": null}

// Custom normalization options
let options = NormalizerOptions {
    sort_keys: true,
    remove_null_values: true,
    remove_empty_containers: true,
    ..Default::default()
};
let normalized = normalize_with_options(json, options).unwrap();
// Result: {"a": 1, "b": 2}
```

### Normalization Options

The `NormalizerOptions` struct provides fine-grained control over normalization:

- `sort_keys`: Sort object keys alphabetically
- `remove_null_values`: Remove null values from objects
- `remove_empty_containers`: Remove empty objects and arrays
- `normalize_numbers`: Convert floats to integers when possible
- `prefer_integers`: Prefer integer representation for whole numbers
- `trim_strings`: Trim whitespace from string values
- `normalize_string_case`: Convert strings to lowercase
- `deduplicate_arrays`: Remove duplicate values from arrays
- `max_depth`: Maximum recursion depth for nested structures

### Specialized Normalizers

#### Canonical Normalizer

Produces deterministic JSON output suitable for hashing and comparison:

```rust
use vexy_json_core::transform::CanonicalNormalizer;

let normalizer = CanonicalNormalizer::new();
let canonical = normalizer.normalize(json).unwrap();
```

#### Cleanup Normalizer

Removes unnecessary elements and optimizes for size:

```rust
use vexy_json_core::transform::CleanupNormalizer;

let normalizer = CleanupNormalizer::new();
let cleaned = normalizer.normalize(json).unwrap();
```

## AST Optimization

The AST optimizer improves JSON structure performance through various optimization techniques.

### Basic Usage

```rust
use vexy_json_core::transform::{optimize, optimize_with_options, OptimizerOptions};

// Basic optimization with default options
let json = r#"{"count": 42.0, "items": [1, 2, 3]}"#;
let optimized = optimize(&json).unwrap();
// Numbers are optimized, strings may be interned

// Custom optimization options
let options = OptimizerOptions {
    intern_strings: true,
    min_intern_length: 5,
    min_intern_count: 2,
    optimize_numbers: true,
    remove_empty_containers: true,
    ..Default::default()
};
let optimized = optimize_with_options(&json, options).unwrap();
```

### Optimization Features

#### String Interning

Reduces memory usage by deduplicating repeated strings:

```rust
let options = OptimizerOptions {
    intern_strings: true,
    min_intern_length: 10,    // Only intern strings >= 10 chars
    min_intern_count: 3,      // Only intern strings appearing >= 3 times
    ..Default::default()
};
```

#### Number Optimization

Converts floats to integers when possible:

```rust
// Input: {"price": 19.0, "count": 42.5}
// Output: {"price": 19, "count": 42.5}
```

#### Container Optimization

Optimizes small objects and arrays:

```rust
let options = OptimizerOptions {
    optimize_small_objects: true,
    max_small_object_size: 4,
    collapse_single_arrays: true,
    remove_empty_containers: true,
    ..Default::default()
};
```

### Specialized Optimizers

#### Memory Optimizer

Optimizes for minimal memory usage:

```rust
use vexy_json_core::transform::MemoryOptimizer;

let optimized = MemoryOptimizer::minimize_memory(&json).unwrap();
```

#### Performance Optimizer

Optimizes for maximum performance:

```rust
use vexy_json_core::transform::PerformanceOptimizer;

let optimized = PerformanceOptimizer::maximize_performance(&json).unwrap();
```

### Optimization Statistics

Track optimization effectiveness:

```rust
use vexy_json_core::transform::AstOptimizer;

let mut optimizer = AstOptimizer::new();
let optimized = optimizer.optimize(&json).unwrap();
let stats = optimizer.stats();

println!("Interned strings: {}", stats.interner_stats.interned_strings);
println!("Saved bytes: {}", stats.interner_stats.saved_bytes);
```

## Advanced Usage

### Chaining Transformations

Combine normalization and optimization:

```rust
use vexy_json_core::{parse, transform::{normalize, optimize}};

let json = r#"{"z": 1.0, "a": 2.0, "b": null}"#;
let value = parse(json).unwrap();
let normalized = normalize(&value).unwrap();
let optimized = optimize(&normalized).unwrap();
```

### Custom Transformation Pipeline

Create custom transformation pipelines:

```rust
use vexy_json_core::transform::{NormalizerOptions, OptimizerOptions};

fn custom_transform(json: &str) -> Result<String, Error> {
    // First normalize
    let norm_options = NormalizerOptions {
        sort_keys: true,
        remove_null_values: true,
        ..Default::default()
    };
    let normalized = normalize_with_options(json, norm_options)?;
    
    // Then optimize
    let opt_options = OptimizerOptions {
        intern_strings: true,
        optimize_numbers: true,
        ..Default::default()
    };
    let optimized = optimize_with_options(&normalized, opt_options)?;
    
    Ok(optimized.to_string())
}
```

## Performance Considerations

### When to Use Normalization

- **Data deduplication**: When you need consistent JSON formatting
- **Comparison**: When comparing JSON structures
- **Storage**: When minimizing storage space
- **Hashing**: When creating content hashes

### When to Use Optimization

- **Memory-constrained environments**: Use MemoryOptimizer
- **Performance-critical applications**: Use PerformanceOptimizer
- **Large JSON datasets**: String interning provides significant benefits
- **Repeated processing**: Optimization overhead pays off over time

### Best Practices

1. **Profile before optimizing**: Measure actual performance impact
2. **Choose appropriate options**: Not all optimizations help every use case
3. **Consider trade-offs**: Memory savings vs. processing time
4. **Test thoroughly**: Ensure optimizations don't change semantics

## Error Handling

Both normalization and optimization can fail:

```rust
use vexy_json_core::transform::normalize;

match normalize(json) {
    Ok(normalized) => println!("Success: {}", normalized),
    Err(e) => eprintln!("Normalization failed: {}", e),
}
```

Common error scenarios:
- Invalid JSON input
- Circular references (when max_depth is exceeded)
- Memory allocation failures
- Serialization errors

## Integration with Other Features

### With Parsing

```rust
use vexy_json_core::{parse_with_options, transform::normalize, ParserOptions};

let options = ParserOptions {
    allow_comments: true,
    allow_trailing_commas: true,
    ..Default::default()
};

let parsed = parse_with_options(json, options)?;
let normalized = normalize(&parsed)?;
```

### With Streaming

```rust
use vexy_json_core::{streaming::parse_streaming, transform::optimize};

for value in parse_streaming(reader)? {
    let optimized = optimize(&value?)?;
    // Process optimized value
}
```

This transformation system provides powerful tools for JSON processing while maintaining the flexibility and performance that Vexy JSON is known for.