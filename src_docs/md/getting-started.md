# Getting Started

Welcome to Vexy JSON! This guide will help you get up and running with the forgiving JSON parser across different programming languages.

## What is Vexy JSON?

Vexy JSON is a lenient JSON parser that accepts both standard JSON and common extensions that developers actually use:

- **Comments**: Single-line `//` and multi-line `/* */` 
- **Trailing commas**: In arrays and objects
- **Unquoted keys**: When unambiguous
- **Single quotes**: For strings
- **Implicit objects**: Top-level key-value pairs

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
vexy-json = "1.2.4"
```

Or use cargo:

```bash
cargo add vexy-json
```

### Python

```bash
pip install vexy-json
```

### JavaScript/Node.js

```bash
npm install vexy-json-wasm
```

### C/C++

Download the latest release from GitHub or build from source:

```bash
git clone https://github.com/vexyart/vexy-json.git
cd vexy-json
cargo build --release --features c-api
```

## Your First Parse

### Rust

```rust
use vexy_json::VexyJson;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
    {
        "name": "My App",
        "version": "1.0.0", // Version comment
        "features": [
            "fast",
            "reliable", // Trailing comma OK
        ]
    }
    "#;
    
    let value = VexyJson::parse(json)?;
    println!("Parsed: {:?}", value);
    Ok(())
}
```

### Python

```python
import vexy_json

json_string = '''
{
    "name": "My App",
    "version": "1.0.0", // Version comment
    "features": [
        "fast",
        "reliable", // Trailing comma OK
    ]
}
'''

try:
    result = vexy_json.parse(json_string)
    print(f"Parsed: {result}")
except Exception as e:
    print(f"Error: {e}")
```

### JavaScript

```javascript
import { VexyJson } from 'vexy-json-wasm';

const jsonString = `
{
    "name": "My App",
    "version": "1.0.0", // Version comment
    "features": [
        "fast",
        "reliable", // Trailing comma OK
    ]
}
`;

try {
    const result = VexyJson.parse(jsonString);
    console.log('Parsed:', result);
} catch (error) {
    console.error('Error:', error);
}
```

### C++

```cpp
#include "vexy_json.hpp"
#include <iostream>

int main() {
    const char* json = R"(
    {
        "name": "My App",
        "version": "1.0.0", // Version comment
        "features": [
            "fast",
            "reliable", // Trailing comma OK
        ]
    }
    )";
    
    try {
        auto result = vexy_json::parse(json);
        std::cout << "Parsed successfully!" << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
    
    return 0;
}
```

## Configuration Options

Vexy JSON can be configured to be more or less strict:

### Rust Configuration

```rust
use vexy_json::{VexyJson, Config};

let config = Config::builder()
    .allow_comments(true)
    .allow_trailing_commas(true)
    .allow_unquoted_keys(true)
    .allow_single_quotes(true)
    .build();

let value = VexyJson::parse_with_config(json, &config)?;
```

### Error Handling

Vexy JSON provides detailed error information:

```rust
use vexy_json::VexyJson;

match VexyJson::parse(invalid_json) {
    Ok(value) => println!("Success: {:?}", value),
    Err(e) => {
        println!("Error at line {}, column {}: {}", 
                 e.line(), e.column(), e.message());
        if let Some(suggestion) = e.suggestion() {
            println!("Suggestion: {}", suggestion);
        }
    }
}
```

## Next Steps

- Learn about [Core Features](core-features.md) that make Vexy JSON forgiving
- Check the [API Reference](api-reference.md) for complete function documentation
- Explore [Language Bindings](language-bindings.md) for platform-specific guides
- See [Advanced Usage](advanced-usage.md) for streaming and performance optimization

## Common Issues

### WASM Loading (JavaScript)

If you encounter issues loading the WASM module:

```javascript
import init, { VexyJson } from 'vexy-json-wasm';

async function main() {
    await init(); // Initialize WASM
    const result = VexyJson.parse(jsonString);
}
```

### Missing Dependencies (Python)

If installation fails, ensure you have the required build tools:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Windows
# Install Visual Studio Build Tools
```

### Cargo Build Issues (Rust)

For the latest features, use the git dependency:

```toml
[dependencies]
vexy-json = { git = "https://github.com/vexyart/vexy-json.git" }
```