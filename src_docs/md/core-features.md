# Core Features

Vexy JSON extends standard JSON with developer-friendly features while maintaining full backward compatibility. This chapter explores what makes Vexy JSON "forgiving" and how these features improve your development experience.

## Standard JSON Support

Vexy JSON is fully compliant with [RFC 8259](https://tools.ietf.org/html/rfc8259) and passes all standard JSON test suites.

```json
{
  "string": "Hello, World!",
  "number": 42,
  "float": 3.14159,
  "boolean": true,
  "null": null,
  "array": [1, 2, 3],
  "object": {"nested": "value"}
}
```

## Forgiving Extensions

### Comments

Both single-line and multi-line comments are supported:

```javascript
{
  // Single-line comment
  "name": "My Application",
  
  /* Multi-line
     comment block */
  "version": "1.0.0",
  
  "config": {
    "debug": true // Inline comment
  }
}
```

**Use cases:**
- Configuration files with explanations
- JSON templates with documentation
- Development and debugging

### Trailing Commas

Trailing commas are allowed in arrays and objects:

```javascript
{
  "items": [
    "first",
    "second",
    "third", // ← This trailing comma is OK
  ],
  "config": {
    "setting1": true,
    "setting2": false, // ← This too
  }, // ← And this
}
```

**Benefits:**
- Easier version control diffs
- Simpler code generation
- Less error-prone editing

### Unquoted Keys

Object keys can be unquoted when they're valid identifiers:

```javascript
{
  name: "Unquoted key",
  version: "1.0.0",
  "special-key": "Needs quotes due to hyphen",
  "2key": "Needs quotes due to starting with number"
}
```

**Rules for unquoted keys:**
- Must start with letter, `_`, or `$`
- Can contain letters, digits, `_`, or `$`
- Cannot be reserved words
- Case-sensitive

### Single-Quoted Strings

Strings can use single quotes instead of double quotes:

```javascript
{
  "double": "This uses double quotes",
  'single': 'This uses single quotes',
  'mixed': "You can mix both styles",
  'escaped': 'Use \' to escape single quotes'
}
```

### Implicit Top-Level Objects

Top-level key-value pairs without braces:

```javascript
// Instead of: {"name": "value", "other": "data"}
name: "value",
other: "data"
```

### Number Formats

Extended number format support:

```javascript
{
  "hex": 0xFF,           // Hexadecimal
  "octal": 0o755,        // Octal  
  "binary": 0b1010,      // Binary
  "scientific": 1.23e-4, // Scientific notation
  "infinity": Infinity,   // Infinity values
  "nan": NaN             // Not a Number
}
```

### Newlines as Separators

Newlines can act as comma separators:

```javascript
{
  name: "My App"
  version: "1.0.0"
  settings: {
    debug: true
    port: 8080
  }
}
```

## Configuration Control

You can control which features are enabled:

### Rust

```rust
use vexy_json::{Config, VexyJson};

// Strict mode (standard JSON only)
let strict_config = Config::strict();

// Permissive mode (all extensions)
let permissive_config = Config::permissive();

// Custom configuration
let custom_config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(true)
    .allow_unquoted_keys(false)
    .allow_single_quotes(false)
    .build();

let result = VexyJson::parse_with_config(json, &custom_config)?;
```

### Python

```python
import vexy_json

# Parse with specific features
result = vexy_json.parse(
    json_string,
    allow_comments=True,
    allow_trailing_commas=True,
    allow_unquoted_keys=False
)
```

### JavaScript

```javascript
import { VexyJson } from 'vexy-json-wasm';

const config = {
    allowComments: true,
    allowTrailingCommas: true,
    allowUnquotedKeys: false,
    allowSingleQuotes: false
};

const result = VexyJson.parseWithConfig(jsonString, config);
```

## Error Handling and Recovery

Vexy JSON provides detailed error information and can often suggest fixes:

```rust
use vexy_json::VexyJson;

let broken_json = r#"
{
  "name": "Test",
  "items": [1, 2, 3,] // Error: trailing comma in strict mode
}
"#;

match VexyJson::parse_strict(broken_json) {
    Err(e) => {
        println!("Error: {}", e.message());
        println!("Location: line {}, column {}", e.line(), e.column());
        
        if let Some(suggestion) = e.suggestion() {
            println!("Suggestion: {}", suggestion);
            // Output: "Remove trailing comma or enable permissive mode"
        }
        
        // Try auto-repair
        if let Ok(repaired) = e.try_repair() {
            println!("Auto-repaired: {:?}", repaired);
        }
    }
    Ok(value) => println!("Parsed: {:?}", value),
}
```

## Performance Characteristics

### Zero-Copy Parsing

For string values, Vexy JSON can parse without copying:

```rust
use vexy_json::{Config, VexyJson};

let config = Config::builder()
    .zero_copy_strings(true)
    .build();

// String values reference the original input
let value = VexyJson::parse_with_config(json, &config)?;
```

### Streaming Support

Large JSON can be parsed incrementally:

```rust
use vexy_json::streaming::StreamingParser;

let mut parser = StreamingParser::new();
parser.feed_chunk(chunk1)?;
parser.feed_chunk(chunk2)?;
let result = parser.finalize()?;
```

## Comparison with Standard JSON

| Feature | Standard JSON | Vexy JSON |
|---------|---------------|-----------|
| Comments | ❌ | ✅ |
| Trailing commas | ❌ | ✅ |
| Unquoted keys | ❌ | ✅ |
| Single quotes | ❌ | ✅ |
| Hex numbers | ❌ | ✅ |
| Implicit objects | ❌ | ✅ |
| Error recovery | ❌ | ✅ |
| Performance | Fast | Faster* |

*In many cases due to optimizations

## Best Practices

### When to Use Forgiving Features

**Configuration Files:**
```javascript
// config.json
{
  // Database configuration
  database: {
    host: "localhost",
    port: 5432,
    name: "myapp", // Trailing comma for easy editing
  },
  
  // Feature flags
  features: {
    newUI: true,
    analytics: false, // Easy to toggle
  }
}
```

**Development and Testing:**
```javascript
// test-data.json
{
  users: [
    {name: "Alice", age: 30},
    {name: "Bob", age: 25}, // Easy to add/remove users
  ],
  
  // Test configuration
  test_config: {
    timeout: 5000,
    retries: 3, // Clear what each value means
  }
}
```

### When to Stay Strict

**Data Exchange:**
- APIs between systems
- Public data formats
- Long-term storage

**Compliance:**
- When standards adherence is required
- Interoperability with strict parsers
- Security-sensitive contexts

### Migration Strategy

1. **Start permissive** during development
2. **Add strictness** as code matures
3. **Use validation** for external inputs
4. **Configure per use case**

```rust
// Development
let dev_config = Config::permissive();

// Production API
let api_config = Config::strict();

// Configuration files
let config_file_config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(true)
    .build();
```