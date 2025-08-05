# Migration & Compatibility

This guide helps you migrate from other JSON parsers to Vexy JSON and provides compatibility information for existing codebases.

## Migration from Popular JSON Libraries

### From serde_json (Rust)

Vexy JSON provides a compatibility layer for easy migration from `serde_json`:

#### Drop-in Replacement

```rust
// Before: serde_json
use serde_json::{Value, from_str, to_string};

let value: Value = from_str(json_str)?;
let serialized = to_string(&value)?;

// After: vexy_json (with compatibility mode)
use vexy_json::compat::serde::{Value, from_str, to_string};

let value: Value = from_str(json_str)?;  // Now supports comments!
let serialized = to_string(&value)?;     // Same API
```

#### Gradual Migration

```rust
// 1. Start by importing both libraries
use serde_json;
use vexy_json;

fn parse_config_file(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    
    // Try Vexy JSON first (allows comments in config)
    match vexy_json::from_str(&content) {
        Ok(value) => Ok(Config::from_value(value)?),
        Err(_) => {
            // Fallback to serde_json for compatibility
            let value = serde_json::from_str(&content)?;
            Ok(Config::from_serde_value(value)?)
        }
    }
}

// 2. Gradually replace serde_json calls
fn parse_api_response(json: &str) -> Result<ApiResponse, ApiError> {
    // Use Vexy JSON but in strict mode for APIs
    let value = vexy_json::parse_strict(json)?;
    Ok(ApiResponse::from_value(value)?)
}

// 3. Eventually remove serde_json dependency
```

#### Feature Mapping

| serde_json Feature | Vexy JSON Equivalent | Notes |
|-------------------|---------------------|-------|
| `from_str()` | `vexy_json::from_str()` | Plus forgiving features |
| `to_string()` | `vexy_json::to_string()` | Identical output |
| `to_string_pretty()` | `vexy_json::to_string_pretty()` | Same formatting |
| `Value` enum | `vexy_json::Value` | Compatible variants |
| Custom deserializers | Serde integration crate | Full compatibility |

### From std::json (Python)

```python
# Before: standard json module
import json

with open('config.json') as f:
    data = json.load(f)

result = json.dumps(data, indent=2)

# After: vexy_json
import vexy_json as json  # Drop-in replacement

with open('config.json') as f:
    data = json.load(f)  # Now supports comments and trailing commas!

result = json.dumps(data, indent=2)  # Same output format
```

#### Django Integration

```python
# settings.py

# Before
import json
CONFIG = json.load(open('config.json'))

# After - supports comments in config files
import vexy_json as json
CONFIG = json.load(open('config.json'))

# Custom JSON field that accepts forgiving JSON
from django.db import models
import vexy_json

class ForgivingJSONField(models.JSONField):
    def to_python(self, value):
        if isinstance(value, str):
            return vexy_json.loads(value)
        return value
    
    def get_prep_value(self, value):
        if value is None:
            return value
        return vexy_json.dumps(value)
```

#### FastAPI Migration

```python
# Before: standard JSON parsing
from fastapi import FastAPI, HTTPException
import json

app = FastAPI()

@app.post("/api/data")
async def handle_data(request: Request):
    try:
        body = await request.body()
        data = json.loads(body.decode())
        return process_data(data)
    except json.JSONDecodeError as e:
        raise HTTPException(status_code=400, detail=str(e))

# After: Vexy JSON with better error messages
import vexy_json

@app.post("/api/data")
async def handle_data(request: Request):
    try:
        body = await request.body()
        data = vexy_json.loads(body.decode())
        return process_data(data)
    except vexy_json.ParseError as e:
        raise HTTPException(
            status_code=400, 
            detail={
                "error": e.message,
                "line": e.line,
                "column": e.column,
                "suggestion": e.suggestion
            }
        )
```

### From JSON.parse (JavaScript)

```javascript
// Before: standard JSON.parse
const data = JSON.parse(jsonString);

// After: Vexy JSON WASM
import { VexyJson } from 'vexy-json-wasm';

// Initialize WASM module once
await init();

// Drop-in replacement with better error handling
try {
    const data = VexyJson.parse(jsonString);
} catch (error) {
    console.error(`Parse error at ${error.line}:${error.column}: ${error.message}`);
    if (error.suggestion) {
        console.log(`Suggestion: ${error.suggestion}`);
    }
}
```

#### Node.js Configuration Loading

```javascript
// Before
const fs = require('fs');

function loadConfig(path) {
    const content = fs.readFileSync(path, 'utf8');
    return JSON.parse(content);
}

// After - supports comments in config files
const { VexyJson } = require('vexy-json-wasm');

function loadConfig(path) {
    const content = fs.readFileSync(path, 'utf8');
    return VexyJson.parse(content);  // Comments and trailing commas OK!
}
```

#### React Component Migration

```javascript
// Before
function ConfigEditor({ configText, onChange }) {
    const [data, setData] = useState(null);
    const [error, setError] = useState(null);
    
    useEffect(() => {
        try {
            const parsed = JSON.parse(configText);
            setData(parsed);
            setError(null);
        } catch (e) {
            setError(e.message);
            setData(null);
        }
    }, [configText]);
    
    return error ? <div>Error: {error}</div> : <ConfigDisplay data={data} />;
}

// After - better error reporting
import { VexyJson } from 'vexy-json-wasm';

function ConfigEditor({ configText, onChange }) {
    const [data, setData] = useState(null);
    const [error, setError] = useState(null);
    
    useEffect(() => {
        try {
            const parsed = VexyJson.parse(configText);
            setData(parsed);
            setError(null);
        } catch (e) {
            setError({
                message: e.message,
                line: e.line,
                column: e.column,
                suggestion: e.suggestion
            });
            setData(null);
        }
    }, [configText]);
    
    if (error) {
        return (
            <div className="error">
                <div>Error at line {error.line}, column {error.column}:</div>
                <div>{error.message}</div>
                {error.suggestion && <div>Suggestion: {error.suggestion}</div>}
            </div>
        );
    }
    
    return <ConfigDisplay data={data} />;
}
```

## Compatibility Matrices

### Language Version Support

| Language | Minimum Version | Recommended | Notes |
|----------|----------------|-------------|-------|
| Rust | 1.70.0 | Latest stable | MSRV policy |
| Python | 3.8 | 3.10+ | Async features require 3.9+ |
| Node.js | 14.0 | 18.0+ | WASM support required |
| Browser | ES2018 | ES2020+ | WebAssembly required |

### Platform Support

| Platform | Rust | Python | WASM | C/C++ |
|----------|------|--------|------|-------|
| Linux x64 | ✅ | ✅ | ✅ | ✅ |
| Linux ARM64 | ✅ | ✅ | ✅ | ✅ |
| macOS x64 | ✅ | ✅ | ✅ | ✅ |
| macOS ARM64 | ✅ | ✅ | ✅ | ✅ |
| Windows x64 | ✅ | ✅ | ✅ | ✅ |
| Windows ARM64 | ✅ | ⚠️ | ✅ | ⚠️ |
| FreeBSD | ✅ | ⚠️ | ✅ | ⚠️ |

✅ Fully supported  
⚠️ Community supported  
❌ Not supported  

### JSON Feature Compatibility

| Feature | Standard JSON | Vexy JSON (Default) | Vexy JSON (Strict) |
|---------|---------------|--------------------|--------------------|
| Objects | ✅ | ✅ | ✅ |
| Arrays | ✅ | ✅ | ✅ |
| Strings | ✅ | ✅ | ✅ |
| Numbers | ✅ | ✅ | ✅ |
| Booleans | ✅ | ✅ | ✅ |
| Null | ✅ | ✅ | ✅ |
| Comments | ❌ | ✅ | ❌ |
| Trailing commas | ❌ | ✅ | ❌ |
| Unquoted keys | ❌ | ✅ | ❌ |
| Single quotes | ❌ | ✅ | ❌ |
| Hex numbers | ❌ | ⚠️ | ❌ |
| Implicit objects | ❌ | ✅ | ❌ |

⚠️ Planned feature

## Migration Strategies

### Strategy 1: Gradual Replacement

Best for large codebases with many JSON parsing calls.

```rust
// Phase 1: Add both dependencies
[dependencies]
serde_json = "1.0"
vexy-json = "1.2"

// Phase 2: Create abstraction layer
mod json {
    pub use vexy_json::Value;
    
    pub fn parse_strict(s: &str) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(vexy_json::parse_strict(s)?)
    }
    
    pub fn parse_lenient(s: &str) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(vexy_json::parse(s)?)
    }
    
    // Fallback for compatibility
    pub fn parse_with_fallback(s: &str) -> Result<Value, Box<dyn std::error::Error>> {
        vexy_json::parse(s)
            .or_else(|_| serde_json::from_str(s).map(Value::from_serde))
            .map_err(|e| e.into())
    }
}

// Phase 3: Replace calls gradually
use crate::json;

// Old: serde_json::from_str(data)?
// New: json::parse_strict(data)?

// Phase 4: Remove serde_json dependency
```

### Strategy 2: Feature Flags

Good for libraries that want to offer both options.

```rust
// Cargo.toml
[features]
default = ["vexy-json"]
serde-json = ["serde_json"]
vexy-json = ["vexy_json"]

[dependencies]
serde_json = { version = "1.0", optional = true }
vexy_json = { version = "1.2", optional = true }

// lib.rs
#[cfg(feature = "vexy-json")]
pub use vexy_json as json;

#[cfg(feature = "serde-json")]
mod json {
    pub use serde_json::*;
    pub use serde_json::Value;
}
```

### Strategy 3: Runtime Configuration

Allow users to choose the parser at runtime.

```rust
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub enum JsonParser {
    Strict,
    Lenient,
    Compatible,
}

static PARSER_CONFIG: OnceLock<JsonParser> = OnceLock::new();

pub fn set_parser(parser: JsonParser) {
    PARSER_CONFIG.set(parser).ok();
}

pub fn parse_json(input: &str) -> Result<Value, ParseError> {
    match PARSER_CONFIG.get().unwrap_or(&JsonParser::Lenient) {
        JsonParser::Strict => vexy_json::parse_strict(input),
        JsonParser::Lenient => vexy_json::parse(input),
        JsonParser::Compatible => {
            vexy_json::parse(input)
                .or_else(|_| serde_json::from_str(input).map(Value::from))
        }
    }
}
```

## Handling Breaking Changes

### API Evolution

Vexy JSON follows semantic versioning:

- **Major versions** (2.0, 3.0): Breaking API changes
- **Minor versions** (1.1, 1.2): New features, backward compatible
- **Patch versions** (1.2.1, 1.2.2): Bug fixes only

### Migration Between Major Versions

#### From v1.x to v2.x (hypothetical)

```rust
// v1.x
let config = Config::new()
    .allow_comments(true)
    .allow_trailing_commas(true);

// v2.x - Builder pattern
let config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(true)
    .build();
```

Provide migration macros:

```rust
// Migration helper
macro_rules! config_v1_to_v2 {
    ($old_config:expr) => {
        Config::builder()
            .allow_comments($old_config.comments_enabled())
            .allow_trailing_commas($old_config.trailing_commas_enabled())
            .build()
    };
}
```

### Deprecation Process

1. **Announce**: Document in release notes
2. **Deprecate**: Add `#[deprecated]` attributes
3. **Provide alternatives**: Clear migration path
4. **Remove**: In next major version

```rust
#[deprecated(since = "1.5.0", note = "Use `Config::builder()` instead")]
pub fn new() -> ConfigBuilder {
    Config::builder()
}
```

## Performance Migration Considerations

### Benchmarking During Migration

Compare performance before and after migration:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_old_vs_new(c: &mut Criterion) {
    let test_data = load_test_data();
    
    c.bench_function("serde_json", |b| {
        b.iter(|| {
            serde_json::from_str::<serde_json::Value>(black_box(&test_data))
        })
    });
    
    c.bench_function("vexy_json_strict", |b| {
        b.iter(|| {
            vexy_json::parse_strict(black_box(&test_data))
        })
    });
    
    c.bench_function("vexy_json_permissive", |b| {
        b.iter(|| {
            vexy_json::parse(black_box(&test_data))
        })
    });
}
```

### Memory Usage Comparison

```rust
fn compare_memory_usage() {
    let json_data = generate_test_json(1024 * 1024); // 1MB
    
    // Measure serde_json memory usage
    let start_memory = get_memory_usage();
    let _serde_value: serde_json::Value = serde_json::from_str(&json_data).unwrap();
    let serde_memory = get_memory_usage() - start_memory;
    
    // Measure vexy_json memory usage
    let start_memory = get_memory_usage();
    let _vexy_value = vexy_json::parse(&json_data).unwrap();
    let vexy_memory = get_memory_usage() - start_memory;
    
    println!("serde_json memory: {} bytes", serde_memory);
    println!("vexy_json memory: {} bytes", vexy_memory);
    println!("Difference: {:.2}%", 
             (vexy_memory as f64 / serde_memory as f64 - 1.0) * 100.0);
}
```

## Testing Migration

### Compatibility Test Suite

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[test]
    fn test_json_roundtrip_compatibility() {
        let test_cases = load_json_test_suite();
        
        for test_case in test_cases {
            // Parse with old library
            let serde_result: Result<serde_json::Value, _> = 
                serde_json::from_str(&test_case.json);
            
            // Parse with new library
            let vexy_result = vexy_json::parse_strict(&test_case.json);
            
            match (serde_result, vexy_result) {
                (Ok(serde_val), Ok(vexy_val)) => {
                    // Both should succeed and produce equivalent results
                    assert_equivalent_values(&serde_val, &vexy_val);
                }
                (Err(_), Err(_)) => {
                    // Both should fail on the same invalid JSON
                }
                _ => {
                    panic!("Inconsistent parsing results for: {}", test_case.json);
                }
            }
        }
    }
    
    fn assert_equivalent_values(serde_val: &serde_json::Value, vexy_val: &vexy_json::Value) {
        // Implementation to compare values across different types
    }
}
```

### Real-World Data Testing

```rust
#[test]
fn test_with_real_world_json() {
    // Test with actual JSON from popular APIs
    let github_api_response = include_str!("../test_data/github_api.json");
    let npm_package_json = include_str!("../test_data/package.json");
    let config_file = include_str!("../test_data/config.json");
    
    for json_data in [github_api_response, npm_package_json, config_file] {
        let serde_result = serde_json::from_str::<serde_json::Value>(json_data);
        let vexy_result = vexy_json::parse_strict(json_data);
        
        match (serde_result, vexy_result) {
            (Ok(_), Ok(_)) => { /* Both succeed - good! */ }
            (Err(serde_err), Err(vexy_err)) => {
                // Both fail - check error quality
                assert!(vexy_err.to_string().len() >= serde_err.to_string().len(),
                        "Vexy JSON should provide at least as detailed errors");
            }
            _ => panic!("Inconsistent results"),
        }
    }
}
```

## Migration Checklist

### Pre-Migration Assessment

- [ ] Inventory all JSON parsing calls in codebase
- [ ] Identify performance-critical paths
- [ ] Document current error handling patterns
- [ ] Establish performance baselines
- [ ] Review JSON data formats used

### Migration Planning

- [ ] Choose migration strategy (gradual vs. all-at-once)
- [ ] Plan testing approach
- [ ] Identify rollback procedures
- [ ] Schedule migration phases
- [ ] Prepare team training materials

### Implementation

- [ ] Add Vexy JSON dependency
- [ ] Create abstraction layer (if gradual migration)
- [ ] Migrate non-critical paths first
- [ ] Update error handling code
- [ ] Add feature flags for A/B testing

### Validation

- [ ] Run existing test suite
- [ ] Add migration-specific tests
- [ ] Performance testing
- [ ] Memory usage validation
- [ ] Error handling verification

### Post-Migration

- [ ] Monitor production metrics
- [ ] Remove old dependencies
- [ ] Update documentation
- [ ] Train team on new features
- [ ] Plan future feature adoption

By following this guide, you can successfully migrate to Vexy JSON while maintaining compatibility and improving your JSON handling capabilities.