# Advanced Usage

This chapter covers advanced techniques for maximizing Vexy JSON's capabilities in complex scenarios.

## Streaming and Large JSON Processing

### Streaming Parser

For processing large JSON files without loading everything into memory:

```rust
use vexy_json::streaming::{StreamingParser, Event};
use std::fs::File;
use std::io::{BufReader, Read};

fn process_large_json(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut parser = StreamingParser::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        
        for event in parser.feed_chunk(&buffer[..bytes_read])? {
            match event {
                Event::ObjectStart => {
                    println!("Starting new object");
                }
                Event::Key(key) => {
                    println!("Processing key: {}", key);
                }
                Event::Value(value) => {
                    // Process individual values without keeping full structure
                    if value.is_string() {
                        println!("Found string: {}", value.as_str().unwrap());
                    }
                }
                Event::ArrayStart => {
                    println!("Starting array");
                }
                Event::ArrayEnd => {
                    println!("Array complete");
                }
                Event::ObjectEnd => {
                    println!("Object complete");
                }
            }
        }
    }
    
    parser.finalize()?;
    Ok(())
}
```

### Chunked Processing

Process JSON arrays in chunks:

```rust
use vexy_json::streaming::ChunkedParser;

fn process_json_array_chunks(json: &str) -> Result<(), Error> {
    let mut parser = ChunkedParser::new()
        .chunk_size(1000)  // Process 1000 items at a time
        .build();
    
    parser.parse_array(json, |chunk: Vec<Value>| {
        // Process each chunk
        println!("Processing {} items", chunk.len());
        for item in chunk {
            // Handle individual item
        }
        Ok(()) // Return Ok to continue, Err to stop
    })?;
    
    Ok(())
}
```

### NDJSON Support

Newline-Delimited JSON processing:

```rust
use vexy_json::streaming::NdjsonParser;

fn process_ndjson(input: &str) -> Result<(), Error> {
    let parser = NdjsonParser::new();
    
    for line_result in parser.parse_lines(input) {
        match line_result {
            Ok(value) => {
                println!("Parsed line: {:?}", value);
            }
            Err(e) => {
                eprintln!("Error on line {}: {}", e.line_number(), e.message());
                // Continue processing other lines
            }
        }
    }
    
    Ok(())
}
```

## Custom Error Handling and Recovery

### Advanced Error Recovery

```rust
use vexy_json::{Error, ErrorRecovery, VexyJson};

fn robust_parsing(json: &str) -> Result<Value, Error> {
    let recovery = ErrorRecovery::builder()
        .try_repair_quotes(true)
        .try_repair_commas(true)
        .try_repair_braces(true)
        .max_attempts(3)
        .build();
    
    match VexyJson::parse(json) {
        Ok(value) => Ok(value),
        Err(e) => {
            println!("Initial parse failed: {}", e.message());
            
            // Attempt automatic repair
            match recovery.attempt_repair(&e, json) {
                Ok(repaired_json) => {
                    println!("Successfully repaired JSON");
                    VexyJson::parse(&repaired_json)
                }
                Err(repair_error) => {
                    println!("Could not repair: {}", repair_error);
                    Err(e)
                }
            }
        }
    }
}
```

### Custom Error Types

```rust
use vexy_json::{Error, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("JSON parse error: {0}")]
    ParseError(#[from] vexy_json::Error),
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid value for {field}: expected {expected}, got {actual}")]
    InvalidValue {
        field: String,
        expected: String,
        actual: String,
    },
}

fn parse_config(json: &str) -> Result<Config, ConfigError> {
    let value = VexyJson::parse(json)?;
    
    let name = value.get("name")
        .and_then(|v| v.as_str())
        .ok_or(ConfigError::MissingField { 
            field: "name".to_string() 
        })?;
    
    let port = value.get("port")
        .and_then(|v| v.as_i64())
        .ok_or(ConfigError::MissingField { 
            field: "port".to_string() 
        })?;
    
    if port < 1 || port > 65535 {
        return Err(ConfigError::InvalidValue {
            field: "port".to_string(),
            expected: "1-65535".to_string(),
            actual: port.to_string(),
        });
    }
    
    Ok(Config {
        name: name.to_string(),
        port: port as u16,
    })
}
```

## Validation and Schema Support

### Custom Validation

```rust
use vexy_json::{Value, Map};
use serde_json::Value as JsonValue;

trait Validator {
    fn validate(&self, value: &Value) -> Result<(), ValidationError>;
}

struct SchemaValidator {
    required_fields: Vec<String>,
    field_types: Map<String, ValueType>,
}

impl Validator for SchemaValidator {
    fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        let obj = value.as_object()
            .ok_or(ValidationError::WrongType("object".to_string()))?;
        
        // Check required fields
        for field in &self.required_fields {
            if !obj.contains_key(field) {
                return Err(ValidationError::MissingField(field.clone()));
            }
        }
        
        // Check field types
        for (field, expected_type) in &self.field_types {
            if let Some(field_value) = obj.get(field) {
                if !expected_type.matches(field_value) {
                    return Err(ValidationError::TypeMismatch {
                        field: field.clone(),
                        expected: expected_type.clone(),
                        actual: ValueType::from(field_value),
                    });
                }
            }
        }
        
        Ok(())
    }
}
```

### JSON Schema Integration

```rust
use jsonschema::{JSONSchema, ValidationError};
use vexy_json::VexyJson;

fn validate_with_schema(json: &str, schema: &str) -> Result<Value, Vec<ValidationError>> {
    // Parse with Vexy JSON to allow comments in data
    let value = VexyJson::parse(json)?;
    
    // Convert to serde_json::Value for schema validation
    let json_value: serde_json::Value = value.into();
    
    // Parse schema (could also use Vexy JSON here)
    let schema_value: serde_json::Value = serde_json::from_str(schema)?;
    let compiled_schema = JSONSchema::compile(&schema_value)?;
    
    // Validate
    let validation_result = compiled_schema.validate(&json_value);
    match validation_result {
        Ok(()) => Ok(value),
        Err(errors) => Err(errors.collect()),
    }
}
```

## Transformation and Filtering

### Value Transformation

```rust
use vexy_json::{Value, Map};

trait Transform {
    fn transform(&self, value: Value) -> Value;
}

struct KeyTransformer {
    from_snake_case: bool,
    to_camel_case: bool,
}

impl Transform for KeyTransformer {
    fn transform(&self, value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut new_map = Map::new();
                for (key, val) in map {
                    let new_key = if self.from_snake_case && self.to_camel_case {
                        snake_to_camel(&key)
                    } else {
                        key
                    };
                    new_map.insert(new_key, self.transform(val));
                }
                Value::Object(new_map)
            }
            Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| self.transform(v)).collect())
            }
            other => other,
        }
    }
}

fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}
```

### Filtering and Extraction

```rust
use vexy_json::{Value, Path};

fn extract_values(value: &Value, path: &str) -> Vec<&Value> {
    let path_segments: Vec<&str> = path.split('.').collect();
    extract_recursive(value, &path_segments, 0)
}

fn extract_recursive<'a>(
    value: &'a Value, 
    path: &[&str], 
    index: usize
) -> Vec<&'a Value> {
    if index >= path.len() {
        return vec![value];
    }
    
    let segment = path[index];
    let mut results = Vec::new();
    
    match value {
        Value::Object(map) => {
            if segment == "*" {
                // Wildcard: process all values
                for val in map.values() {
                    results.extend(extract_recursive(val, path, index + 1));
                }
            } else if let Some(val) = map.get(segment) {
                results.extend(extract_recursive(val, path, index + 1));
            }
        }
        Value::Array(arr) => {
            if segment == "*" {
                // Wildcard: process all elements
                for val in arr {
                    results.extend(extract_recursive(val, path, index + 1));
                }
            } else if let Ok(idx) = segment.parse::<usize>() {
                if let Some(val) = arr.get(idx) {
                    results.extend(extract_recursive(val, path, index + 1));
                }
            }
        }
        _ => {}
    }
    
    results
}

// Usage
let json = r#"
{
    "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
    ]
}
"#;

let value = VexyJson::parse(json)?;
let names = extract_values(&value, "users.*.name");
// Returns references to "Alice" and "Bob"
```

## Plugin System and Extensions

### Custom Number Parsing

```rust
use vexy_json::plugins::{NumberPlugin, PluginManager};

struct CustomNumberPlugin;

impl NumberPlugin for CustomNumberPlugin {
    fn parse_number(&self, input: &str) -> Option<Number> {
        // Support custom formats like "1_000_000" or "1k"
        if input.contains('_') {
            let cleaned = input.replace('_', "");
            cleaned.parse().ok().map(Number::from)
        } else if input.ends_with('k') || input.ends_with('K') {
            let base = &input[..input.len() - 1];
            base.parse::<f64>().ok().map(|n| Number::from(n * 1000.0))
        } else if input.ends_with('M') {
            let base = &input[..input.len() - 1];
            base.parse::<f64>().ok().map(|n| Number::from(n * 1_000_000.0))
        } else {
            None
        }
    }
}

// Usage
let mut manager = PluginManager::new();
manager.register_number_plugin(Box::new(CustomNumberPlugin));

let config = Config::builder()
    .plugin_manager(manager)
    .build();

// Now can parse: {"memory": "512M", "count": "1_000"}
```

### Comment Preservation

```rust
use vexy_json::plugins::CommentPlugin;

struct CommentPreserver {
    comments: Vec<Comment>,
}

impl CommentPlugin for CommentPreserver {
    fn on_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
    }
}

// Parse with comment preservation
let mut preserver = CommentPreserver { comments: Vec::new() };
let mut manager = PluginManager::new();
manager.register_comment_plugin(Box::new(preserver));

let value = VexyJson::parse_with_plugins(json, &manager)?;
// Comments are now available in preserver.comments
```

## Memory Optimization

### Memory Pools

```rust
use vexy_json::memory::{MemoryPool, PoolConfig};

// Create a memory pool for reusing allocations
let pool = MemoryPool::new(PoolConfig {
    initial_capacity: 1024 * 1024, // 1MB
    max_capacity: 16 * 1024 * 1024, // 16MB
    chunk_size: 64 * 1024, // 64KB chunks
});

// Parse multiple JSON strings efficiently
for json_str in json_strings {
    let value = pool.parse(json_str)?;
    process_value(value);
    // Memory is automatically returned to pool when value is dropped
}
```

### Zero-Copy Parsing

```rust
use vexy_json::{Config, VexyJson};

let config = Config::builder()
    .zero_copy_strings(true)
    .build();

// String values will reference the original input
let value = VexyJson::parse_with_config(json, &config)?;

// IMPORTANT: Original json string must outlive the parsed value
// This won't work:
// let value = {
//     let json = get_json();
//     VexyJson::parse_with_config(&json, &config)?
// }; // json is dropped here, but value references it!
```

### Custom Allocators

```rust
use vexy_json::allocator::{Allocator, BumpAllocator};

// Use a bump allocator for temporary parsing
let bump = BumpAllocator::new(1024 * 1024); // 1MB arena

let config = Config::builder()
    .allocator(Box::new(bump))
    .build();

let value = VexyJson::parse_with_config(json, &config)?;
// All allocations use the bump allocator
// When bump is dropped, all memory is freed at once
```

## Parallel Processing

### Multi-threaded Parsing

```rust
use std::sync::Arc;
use std::thread;
use vexy_json::{Config, VexyJson};

fn parallel_parse(json_strings: Vec<String>) -> Vec<Result<Value, Error>> {
    let config = Arc::new(Config::permissive());
    let chunk_size = json_strings.len() / num_cpus::get();
    
    let mut handles = vec![];
    
    for chunk in json_strings.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let config = Arc::clone(&config);
        
        let handle = thread::spawn(move || {
            chunk.into_iter()
                .map(|json| VexyJson::parse_with_config(&json, &config))
                .collect::<Vec<_>>()
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.extend(handle.join().unwrap());
    }
    
    results
}
```

### Async Processing

```rust
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use vexy_json::VexyJson;

async fn parse_file_async(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).await?;
    
    // Parse in blocking thread pool to avoid blocking async runtime
    let value = tokio::task::spawn_blocking(move || {
        VexyJson::parse(&contents)
    }).await??;
    
    Ok(value)
}

async fn parallel_file_parsing(paths: Vec<&str>) -> Vec<Result<Value, Box<dyn std::error::Error>>> {
    let tasks: Vec<_> = paths.into_iter()
        .map(|path| parse_file_async(path))
        .collect();
    
    futures::future::join_all(tasks).await
}
```

## Integration Patterns

### Configuration Management

```rust
use vexy_json::{VexyJson, Value};
use std::path::Path;

pub struct ConfigManager {
    cache: std::collections::HashMap<String, Value>,
    watch_files: bool,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            watch_files: false,
        }
    }
    
    pub fn load_config<P: AsRef<Path>>(&mut self, path: P) -> Result<&Value, ConfigError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        if !self.cache.contains_key(&path_str) {
            let content = std::fs::read_to_string(&path)?;
            let value = VexyJson::parse(&content)
                .map_err(ConfigError::ParseError)?;
            self.cache.insert(path_str.clone(), value);
        }
        
        Ok(self.cache.get(&path_str).unwrap())
    }
    
    pub fn get<T>(&self, config_name: &str, key: &str) -> Option<T>
    where
        T: for<'a> TryFrom<&'a Value>,
    {
        self.cache.get(config_name)?
            .get(key)?
            .try_into()
            .ok()
    }
}
```

### Hot Reload

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct HotReloadConfig {
    config: Arc<RwLock<Value>>,
    _watcher: RecommendedWatcher,
}

impl HotReloadConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let initial_config = {
            let content = std::fs::read_to_string(&path)?;
            VexyJson::parse(&content)?
        };
        
        let config = Arc::new(RwLock::new(initial_config));
        let config_clone = Arc::clone(&config);
        let path_clone = path.as_ref().to_path_buf();
        
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        
        // Spawn background thread to handle file changes
        std::thread::spawn(move || {
            while let Ok(_event) = rx.recv() {
                if let Ok(content) = std::fs::read_to_string(&path_clone) {
                    if let Ok(new_config) = VexyJson::parse(&content) {
                        if let Ok(mut config_lock) = config_clone.write() {
                            *config_lock = new_config;
                            println!("Configuration reloaded");
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            config,
            _watcher: watcher,
        })
    }
    
    pub fn get(&self) -> Arc<RwLock<Value>> {
        Arc::clone(&self.config)
    }
}
```