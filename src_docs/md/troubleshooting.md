# Troubleshooting

Common issues, error messages, debugging techniques, and solutions for platform-specific problems.

## Common Error Messages

### Parse Errors

#### "Unexpected character at line X, column Y"

**Cause**: Invalid JSON syntax that even Vexy JSON's forgiving parser cannot handle.

```
Error: Unexpected character '}' at line 3, column 5
{
  "name": "test",
  } // Extra comma before closing brace
```

**Solutions**:
1. Check for syntax errors around the reported position
2. Enable more forgiving features:
   ```rust
   let config = Config::builder()
       .allow_trailing_commas(true)
       .build();
   ```
3. Use error recovery:
   ```rust
   match VexyJson::parse(json) {
       Err(e) => {
           if let Ok(repaired) = e.try_repair() {
               println!("Auto-repaired JSON");
               repaired
           } else {
               return Err(e);
           }
       }
       Ok(value) => value,
   }
   ```

#### "Unterminated string at line X"

**Cause**: Missing closing quote for a string value.

```json
{
  "name": "unclosed string,
  "age": 30
}
```

**Solutions**:
1. Check for unescaped quotes in strings
2. Verify proper string termination
3. Use raw strings in Rust for complex JSON:
   ```rust
   let json = r#"{"name": "string with \"quotes\""}"#;
   ```

#### "Invalid number format"

**Cause**: Number doesn't conform to JSON number specification.

```json
{
  "hex": 0xFF,     // Not standard JSON
  "decimal": .5,   // Missing leading zero
  "trailing": 1.   // Missing digits after decimal
}
```

**Solutions**:
1. Enable extended number support:
   ```rust
   let config = Config::builder()
       .allow_hex_numbers(true)
       .allow_leading_decimal_point(true)
       .build();
   ```
2. Fix the number format for strict JSON compliance

### Configuration Errors

#### "Feature not enabled"

**Cause**: Trying to parse JSON with features that are disabled.

**Solution**: Enable the required features:

```rust
// Enable all forgiving features
let config = Config::permissive();

// Or enable specific features
let config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(true)
    .allow_unquoted_keys(true)
    .build();
```

### Memory Errors

#### "Stack overflow during parsing"

**Cause**: JSON nesting exceeds maximum depth limit.

**Solutions**:
1. Increase max depth:
   ```rust
   let config = Config::builder()
       .max_depth(256)  // Default is 128
       .build();
   ```
2. Use streaming parser for deeply nested data:
   ```rust
   let mut parser = StreamingParser::new();
   // Process events instead of building full tree
   ```
3. Validate input depth before parsing

#### "Out of memory"

**Cause**: JSON too large for available memory.

**Solutions**:
1. Use streaming parser:
   ```rust
   let mut parser = StreamingParser::new();
   for chunk in json_chunks {
       parser.feed_chunk(chunk)?;
   }
   ```
2. Enable memory pool:
   ```rust
   let pool = MemoryPool::new(PoolConfig::default());
   let value = pool.parse(json)?;
   ```
3. Process in smaller chunks

## Platform-Specific Issues

### Rust

#### Compilation Issues

**"Could not find crate vexy-json"**

```bash
# Update Cargo.toml
[dependencies]
vexy-json = "1.2.4"

# Or use git version
vexy-json = { git = "https://github.com/vexyart/vexy-json.git" }

# Update registry
cargo update
```

**Linking errors on older Rust versions**

```bash
# Update Rust to latest stable
rustup update stable

# Or specify minimum version in Cargo.toml
[package]
rust-version = "1.70.0"
```

#### Runtime Issues

**"Cannot borrow as mutable"**

```rust
// ❌ This won't work
let value = VexyJson::parse(json)?;
let obj = value.as_object_mut().unwrap();
obj.insert("new_key".to_string(), Value::Bool(true));

// ✅ Use this instead
let mut value = VexyJson::parse(json)?;
if let Value::Object(ref mut obj) = value {
    obj.insert("new_key".to_string(), Value::Bool(true));
}
```

### Python

#### Installation Issues

**"Failed building wheel for vexy-json"**

```bash
# Install build dependencies
pip install --upgrade pip setuptools wheel

# Install Rust if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Try installing with verbose output
pip install vexy-json --verbose
```

**"Microsoft Visual C++ 14.0 is required" (Windows)**

```bash
# Install Visual Studio Build Tools
# Or use pre-built wheels
pip install --only-binary=vexy-json vexy-json
```

#### Runtime Issues

**"Module not found: vexy_json"**

```python
# Check installation
pip list | grep vexy

# Reinstall if necessary
pip uninstall vexy-json
pip install vexy-json

# Import correctly
import vexy_json  # Note: underscore, not hyphen
```

**Performance issues with large JSON**

```python
# Use streaming for large files
import vexy_json

def process_large_file(filename):
    with open(filename, 'r') as f:
        # Read in chunks instead of loading all at once
        chunk_size = 1024 * 1024  # 1MB chunks
        while True:
            chunk = f.read(chunk_size)
            if not chunk:
                break
            # Process chunk
```

### JavaScript/WASM

#### Browser Issues

**"WebAssembly module failed to load"**

```javascript
// Check WASM support
if (!WebAssembly) {
    console.error('WebAssembly not supported in this browser');
    // Provide fallback
}

// Proper initialization
import init, { VexyJson } from 'vexy-json-wasm';

async function initVexyJson() {
    try {
        await init();
        console.log('Vexy JSON WASM loaded successfully');
    } catch (error) {
        console.error('Failed to load WASM:', error);
    }
}
```

**CORS errors when loading WASM**

```javascript
// Serve WASM files with correct MIME type
// In your server configuration:
// .wasm files should be served as application/wasm

// For development, use a local server
// python -m http.server 8000
// Or use webpack dev server with proper configuration
```

**"TypeError: Cannot read property 'parse' of undefined"**

```javascript
// Ensure WASM is initialized before use
import init, { VexyJson } from 'vexy-json-wasm';

let vexyJsonReady = false;

init().then(() => {
    vexyJsonReady = true;
});

function parseJson(jsonString) {
    if (!vexyJsonReady) {
        throw new Error('VexyJson WASM not ready yet');
    }
    return VexyJson.parse(jsonString);
}
```

#### Node.js Issues

**"Cannot find module 'vexy-json-wasm'"**

```bash
# Install the package
npm install vexy-json-wasm

# Check Node.js version (requires Node 14+)
node --version

# Clear npm cache if issues persist
npm cache clean --force
```

**ES Module import issues**

```javascript
// Use dynamic import for CommonJS
async function loadVexyJson() {
    const { VexyJson } = await import('vexy-json-wasm');
    return VexyJson;
}

// Or configure package.json for ES modules
{
  "type": "module"
}
```

### C/C++

#### Compilation Issues

**"vexy_json.h: No such file or directory"**

```bash
# Build the C API
cargo build --release --features c-api

# Copy headers to include path
cp crates/c-api/include/* /usr/local/include/

# Link with the library
gcc -L./target/release -lvexy_json_c -o program program.c
```

**Linking errors**

```bash
# Make sure library is in linker path
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:./target/release

# Or copy to system library directory
sudo cp target/release/libvexy_json_c.so /usr/local/lib/
sudo ldconfig
```

#### Runtime Issues

**Segmentation fault**

```c
// Always check return values
VexyJsonResult* result = vexy_json_parse(json);
if (!result) {
    fprintf(stderr, "Failed to create result object\n");
    return -1;
}

if (!vexy_json_result_is_ok(result)) {
    fprintf(stderr, "Parse error: %s\n", 
            vexy_json_result_error_message(result));
    vexy_json_result_free(result);
    return -1;
}

// Always free resources
vexy_json_result_free(result);
```

**Memory leaks**

```cpp
// Use RAII in C++
class VexyJsonWrapper {
    VexyJsonResult* result_;
public:
    VexyJsonWrapper(const char* json) 
        : result_(vexy_json_parse(json)) {}
    
    ~VexyJsonWrapper() {
        if (result_) {
            vexy_json_result_free(result_);
        }
    }
    
    bool is_ok() const {
        return result_ && vexy_json_result_is_ok(result_);
    }
};
```

## Debugging Techniques

### Enabling Debug Output

#### Rust

```rust
use vexy_json::{VexyJson, Config};

// Enable detailed error messages
let config = Config::builder()
    .debug_mode(true)
    .build();

match VexyJson::parse_with_config(json, &config) {
    Err(e) => {
        println!("Error: {}", e.message());
        println!("Context: {}", e.context());
        println!("Position: {}:{}", e.line(), e.column());
        
        // Print surrounding text
        if let Some(context) = e.surrounding_text(5) {
            println!("Around error:\n{}", context);
        }
    }
    Ok(value) => println!("Success: {:?}", value),
}
```

#### Python

```python
import vexy_json
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)

try:
    result = vexy_json.parse(json_string, debug=True)
except vexy_json.ParseError as e:
    print(f"Detailed error: {e}")
    print(f"Error context: {e.context}")
    print(f"Surrounding text: {e.surrounding_text}")
```

### Step-by-Step Debugging

#### Incremental Parsing

```rust
use vexy_json::debug::{DebugParser, ParseStep};

fn debug_parse(json: &str) {
    let mut parser = DebugParser::new();
    
    for step in parser.parse_steps(json) {
        match step {
            ParseStep::StartObject => println!("Starting object"),
            ParseStep::Key(key) => println!("Found key: {}", key),
            ParseStep::Value(value) => println!("Found value: {:?}", value),
            ParseStep::Error(e) => {
                println!("Error at step {}: {}", parser.step_count(), e);
                break;
            }
        }
    }
}
```

#### Token-Level Debugging

```rust
use vexy_json::lexer::{Lexer, Token};

fn debug_tokenization(json: &str) {
    let lexer = Lexer::new(json);
    
    for (i, token) in lexer.tokenize().enumerate() {
        match token {
            Ok(t) => println!("Token {}: {:?}", i, t),
            Err(e) => {
                println!("Tokenization error at token {}: {}", i, e);
                break;
            }
        }
    }
}
```

### Performance Debugging

#### Memory Usage Tracking

```rust
use vexy_json::{VexyJson, MemoryTracker};

fn debug_memory_usage(json: &str) {
    let tracker = MemoryTracker::new();
    
    let start_memory = tracker.current_usage();
    let value = VexyJson::parse(json)?;
    let end_memory = tracker.current_usage();
    
    println!("Memory used: {} bytes", end_memory - start_memory);
    println!("Peak memory: {} bytes", tracker.peak_usage());
    
    // Detailed breakdown
    let breakdown = tracker.allocation_breakdown();
    for (category, size) in breakdown {
        println!("{}: {} bytes", category, size);
    }
}
```

#### Timing Analysis

```rust
use std::time::Instant;
use vexy_json::VexyJson;

fn debug_timing(json: &str) {
    let phases = [
        "tokenization",
        "parsing", 
        "value_construction",
        "validation"
    ];
    
    let mut timings = std::collections::HashMap::new();
    
    for phase in &phases {
        let start = Instant::now();
        
        // Run specific phase
        match *phase {
            "tokenization" => { /* tokenize only */ }
            "parsing" => { /* parse AST only */ }
            "value_construction" => { /* build Value tree */ }
            "validation" => { /* validate result */ }
            _ => {}
        }
        
        timings.insert(*phase, start.elapsed());
    }
    
    for (phase, duration) in timings {
        println!("{}: {:?}", phase, duration);
    }
}
```

## Error Recovery Strategies

### Automatic Repair

```rust
use vexy_json::{VexyJson, ErrorRecovery};

fn robust_config_loading(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    
    // Try standard parsing first
    match VexyJson::parse(&content) {
        Ok(value) => return Ok(Config::from_value(value)?),
        Err(original_error) => {
            println!("Initial parse failed: {}", original_error.message());
        }
    }
    
    // Try with more permissive settings
    let permissive_config = Config::permissive();
    match VexyJson::parse_with_config(&content, &permissive_config) {
        Ok(value) => return Ok(Config::from_value(value)?),
        Err(permissive_error) => {
            println!("Permissive parse failed: {}", permissive_error.message());
        }
    }
    
    // Try automatic repair
    let recovery = ErrorRecovery::builder()
        .fix_quotes(true)
        .fix_commas(true)
        .fix_brackets(true)
        .build();
    
    match recovery.attempt_repair(&content) {
        Ok(repaired_content) => {
            println!("Successfully repaired JSON");
            let value = VexyJson::parse(&repaired_content)?;
            Ok(Config::from_value(value)?)
        }
        Err(repair_error) => {
            println!("Could not repair JSON: {}", repair_error);
            Err(ConfigError::UnparseableJson)
        }
    }
}
```

### Graceful Degradation

```rust
fn load_config_with_fallback(primary_path: &str, fallback_path: &str) -> Config {
    // Try primary config
    if let Ok(config) = try_load_config(primary_path) {
        return config;
    }
    
    println!("Primary config failed, trying fallback");
    
    // Try fallback config
    if let Ok(config) = try_load_config(fallback_path) {
        return config;
    }
    
    println!("All configs failed, using defaults");
    
    // Use hardcoded defaults
    Config::default()
}

fn try_load_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    
    // Try multiple parsing strategies
    let strategies = [
        || VexyJson::parse_strict(&content),
        || VexyJson::parse(&content),
        || {
            let config = Config::builder()
                .allow_comments(true)
                .allow_trailing_commas(true)
                .repair_mode(true)
                .build();
            VexyJson::parse_with_config(&content, &config)
        },
    ];
    
    for strategy in &strategies {
        if let Ok(value) = strategy() {
            return Ok(Config::from_value(value)?);
        }
    }
    
    Err(ConfigError::AllStrategiesFailed)
}
```

## Frequently Asked Questions

### Q: Why is my JSON parsing slower than expected?

**A**: Common causes:
1. **Debug mode enabled**: Disable debug features in production
2. **Wrong configuration**: Use `Config::strict()` for standard JSON
3. **Large input**: Consider streaming parser
4. **Memory fragmentation**: Use memory pools
5. **Repeated parsing**: Cache parsed results

### Q: Can I parse JSON5 format?

**A**: Vexy JSON supports many JSON5 features:
- Comments (`//` and `/* */`)
- Trailing commas
- Unquoted keys (when unambiguous)
- Single-quoted strings

Not supported:
- Hexadecimal numbers (planned)
- Multiline strings (use string concatenation)
- Reserved word keys (use quotes)

### Q: How do I handle very large JSON files?

**A**: Use the streaming parser:
```rust
use vexy_json::streaming::StreamingParser;

let mut parser = StreamingParser::new();
// Process in chunks without loading full file
```

### Q: Is Vexy JSON thread-safe?

**A**: Yes, parsing operations are thread-safe. You can:
- Parse different JSON strings in parallel
- Share `Config` instances between threads
- Use memory pools with proper synchronization

### Q: How do I migrate from serde_json?

**A**: Vexy JSON provides compatibility layers:
```rust
// Replace this:
use serde_json::Value;
let value: Value = serde_json::from_str(json)?;

// With this:
use vexy_json::Value;
let value: Value = vexy_json::from_str(json)?;
```

Most APIs are drop-in compatible.