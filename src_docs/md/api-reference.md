# API Reference

Complete API documentation for Vexy JSON across all supported languages.

## Rust API

### Core Functions

#### `VexyJson::parse(input: &str) -> Result<Value, Error>`

Parses JSON string with default permissive configuration.

```rust
use vexy_json::VexyJson;

let result = VexyJson::parse(r#"{"name": "test"}"#)?;
```

#### `VexyJson::parse_strict(input: &str) -> Result<Value, Error>`

Parses JSON string with strict RFC 8259 compliance.

```rust
let result = VexyJson::parse_strict(r#"{"name": "test"}"#)?;
```

#### `VexyJson::parse_with_config(input: &str, config: &Config) -> Result<Value, Error>`

Parses JSON string with custom configuration.

```rust
use vexy_json::{VexyJson, Config};

let config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(false)
    .build();

let result = VexyJson::parse_with_config(input, &config)?;
```

### Configuration

#### `Config::builder() -> ConfigBuilder`

Creates a new configuration builder.

```rust
let config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(true)
    .allow_unquoted_keys(true)
    .allow_single_quotes(true)
    .allow_implicit_objects(true)
    .zero_copy_strings(false)
    .max_depth(128)
    .build();
```

#### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `allow_comments` | `bool` | `true` | Allow `//` and `/* */` comments |
| `allow_trailing_commas` | `bool` | `true` | Allow trailing commas in arrays/objects |
| `allow_unquoted_keys` | `bool` | `true` | Allow unquoted object keys |
| `allow_single_quotes` | `bool` | `true` | Allow single-quoted strings |
| `allow_implicit_objects` | `bool` | `true` | Allow top-level key-value pairs |
| `zero_copy_strings` | `bool` | `false` | Use string slices instead of owned strings |
| `max_depth` | `usize` | `128` | Maximum nesting depth |

### Value Types

#### `Value` Enum

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

#### Value Methods

```rust
impl Value {
    pub fn is_null(&self) -> bool;
    pub fn is_bool(&self) -> bool;
    pub fn is_number(&self) -> bool;
    pub fn is_string(&self) -> bool;
    pub fn is_array(&self) -> bool;
    pub fn is_object(&self) -> bool;
    
    pub fn as_bool(&self) -> Option<bool>;
    pub fn as_i64(&self) -> Option<i64>;
    pub fn as_f64(&self) -> Option<f64>;
    pub fn as_str(&self) -> Option<&str>;
    pub fn as_array(&self) -> Option<&Vec<Value>>;
    pub fn as_object(&self) -> Option<&Map<String, Value>>;
    
    pub fn get(&self, key: &str) -> Option<&Value>;
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value>;
    
    pub fn to_string(&self) -> String;
    pub fn to_pretty_string(&self) -> String;
}
```

### Error Handling

#### `Error` Struct

```rust
pub struct Error {
    // Error details
}

impl Error {
    pub fn message(&self) -> &str;
    pub fn line(&self) -> usize;
    pub fn column(&self) -> usize;
    pub fn position(&self) -> usize;
    pub fn suggestion(&self) -> Option<&str>;
    pub fn try_repair(&self) -> Result<Value, Error>;
}
```

### Streaming API

#### `StreamingParser`

```rust
use vexy_json::streaming::StreamingParser;

let mut parser = StreamingParser::new();
parser.feed_chunk(b"{")?;
parser.feed_chunk(b"\"key\":")?;
parser.feed_chunk(b"\"value\"}")?;
let result = parser.finalize()?;
```

## Python API

### Core Functions

#### `parse(json_str: str, **kwargs) -> Any`

```python
import vexy_json

# Basic parsing
result = vexy_json.parse('{"name": "test"}')

# With options
result = vexy_json.parse(
    json_str,
    allow_comments=True,
    allow_trailing_commas=True,
    allow_unquoted_keys=False
)
```

#### `parse_strict(json_str: str) -> Any`

```python
result = vexy_json.parse_strict('{"name": "test"}')
```

#### `loads(json_str: str, **kwargs) -> Any`

Alias for `parse` to match `json.loads` interface.

```python
import vexy_json as json

data = json.loads('{"name": "test"}')
```

### Configuration Options

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `allow_comments` | `bool` | `True` | Allow comments in JSON |
| `allow_trailing_commas` | `bool` | `True` | Allow trailing commas |
| `allow_unquoted_keys` | `bool` | `True` | Allow unquoted object keys |
| `allow_single_quotes` | `bool` | `True` | Allow single-quoted strings |
| `max_depth` | `int` | `128` | Maximum nesting depth |

### Error Handling

```python
import vexy_json

try:
    result = vexy_json.parse(invalid_json)
except vexy_json.ParseError as e:
    print(f"Error at line {e.line}, column {e.column}: {e.message}")
    if e.suggestion:
        print(f"Suggestion: {e.suggestion}")
```

## JavaScript/WASM API

### Initialization

```javascript
import init, { VexyJson } from 'vexy-json-wasm';

// Initialize WASM module
await init();
```

### Core Functions

#### `VexyJson.parse(jsonString: string): any`

```javascript
const result = VexyJson.parse('{"name": "test"}');
```

#### `VexyJson.parseStrict(jsonString: string): any`

```javascript
const result = VexyJson.parseStrict('{"name": "test"}');
```

#### `VexyJson.parseWithConfig(jsonString: string, config: Config): any`

```javascript
const config = {
    allowComments: true,
    allowTrailingCommas: true,
    allowUnquotedKeys: false,
    allowSingleQuotes: false,
    maxDepth: 64
};

const result = VexyJson.parseWithConfig(jsonString, config);
```

### Configuration Interface

```typescript
interface Config {
    allowComments?: boolean;
    allowTrailingCommas?: boolean;
    allowUnquotedKeys?: boolean;
    allowSingleQuotes?: boolean;
    allowImplicitObjects?: boolean;
    maxDepth?: number;
}
```

### Error Handling

```javascript
try {
    const result = VexyJson.parse(invalidJson);
} catch (error) {
    console.log(`Error: ${error.message}`);
    console.log(`Location: line ${error.line}, column ${error.column}`);
    
    if (error.suggestion) {
        console.log(`Suggestion: ${error.suggestion}`);
    }
}
```

### TypeScript Support

```typescript
import { VexyJson, Config, ParseError } from 'vexy-json-wasm';

interface MyData {
    name: string;
    version: string;
}

const result = VexyJson.parse(jsonString) as MyData;
```

## C/C++ API

### C API

#### Include Headers

```c
#include "vexy_json.h"
```

#### Core Functions

```c
// Parse JSON string
VexyJsonResult* vexy_json_parse(const char* json);

// Parse with configuration
VexyJsonResult* vexy_json_parse_with_config(
    const char* json, 
    const VexyJsonConfig* config
);

// Free result
void vexy_json_result_free(VexyJsonResult* result);

// Check if parsing succeeded
bool vexy_json_result_is_ok(const VexyJsonResult* result);

// Get error message
const char* vexy_json_result_error_message(const VexyJsonResult* result);
```

#### Configuration

```c
typedef struct {
    bool allow_comments;
    bool allow_trailing_commas;
    bool allow_unquoted_keys;
    bool allow_single_quotes;
    size_t max_depth;
} VexyJsonConfig;

VexyJsonConfig config = {
    .allow_comments = true,
    .allow_trailing_commas = true,
    .allow_unquoted_keys = false,
    .allow_single_quotes = false,
    .max_depth = 128
};
```

### C++ API

#### Include Headers

```cpp
#include "vexy_json.hpp"
```

#### Core Functions

```cpp
namespace vexy_json {
    // Parse JSON string
    Value parse(const std::string& json);
    
    // Parse with configuration
    Value parse_with_config(const std::string& json, const Config& config);
    
    // Parse strictly
    Value parse_strict(const std::string& json);
}
```

#### Configuration

```cpp
vexy_json::Config config;
config.allow_comments = true;
config.allow_trailing_commas = true;
config.allow_unquoted_keys = false;
config.max_depth = 128;

auto result = vexy_json::parse_with_config(json_string, config);
```

#### Value Access

```cpp
vexy_json::Value value = vexy_json::parse(json);

// Type checking
if (value.is_object()) {
    auto obj = value.as_object();
    if (obj.contains("name")) {
        std::string name = obj["name"].as_string();
    }
}

// Array access
if (value.is_array()) {
    auto arr = value.as_array();
    for (const auto& item : arr) {
        // Process each item
    }
}
```

## Performance APIs

### Benchmarking

#### Rust

```rust
use vexy_json::benchmark::{Benchmark, BenchmarkConfig};

let config = BenchmarkConfig::new()
    .with_iterations(1000)
    .with_warmup(100);

let results = Benchmark::run(&config, json_samples)?;
println!("Average parse time: {:?}", results.average_time());
```

#### Python

```python
import vexy_json

# Time a single parse
elapsed = vexy_json.time_parse(json_string)

# Benchmark multiple iterations
results = vexy_json.benchmark(json_string, iterations=1000)
print(f"Average: {results.average_ms}ms")
```

### Memory Management

#### Zero-Copy Parsing (Rust)

```rust
use vexy_json::{Config, VexyJson};

let config = Config::builder()
    .zero_copy_strings(true)
    .build();

// String values will reference original input
let value = VexyJson::parse_with_config(json, &config)?;
```

#### Memory Pool (Rust)

```rust
use vexy_json::memory::{MemoryPool, PoolConfig};

let pool = MemoryPool::new(PoolConfig::default());
let value = pool.parse(json)?;
// Memory is automatically returned to pool when value is dropped
```

## Serialization

### Rust

```rust
use vexy_json::{Value, to_string, to_pretty_string};

let value = Value::Object(map);

// Compact output
let json = to_string(&value)?;

// Pretty-printed output
let pretty_json = to_pretty_string(&value)?;
```

### Python

```python
import vexy_json

data = {"name": "test", "items": [1, 2, 3]}

# Serialize to JSON
json_str = vexy_json.dumps(data)

# Pretty print
pretty_json = vexy_json.dumps(data, indent=2)
```

### JavaScript

```javascript
// Use standard JSON.stringify
const jsonString = JSON.stringify(data, null, 2);
```