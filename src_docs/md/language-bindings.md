# Language Bindings

Vexy JSON provides native-feeling APIs for multiple programming languages, each optimized for their respective ecosystems.

## Rust (Native)

Rust is the primary implementation language, offering the most complete feature set and best performance.

### Installation

```toml
[dependencies]
vexy-json = "1.2.4"

# For bleeding edge features
vexy-json = { git = "https://github.com/vexyart/vexy-json.git" }
```

### Integration Patterns

#### Error Handling with `?` Operator

```rust
use vexy_json::{VexyJson, Error};

fn parse_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let value = VexyJson::parse(&content)?;
    
    Ok(Config {
        name: value.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field")?,
        port: value.get("port")
            .and_then(|v| v.as_i64())
            .unwrap_or(8080) as u16,
    })
}
```

#### Serde Integration

```rust
use serde::{Deserialize, Serialize};
use vexy_json::serde::{from_str, to_string};

#[derive(Deserialize, Serialize)]
struct Config {
    name: String,
    debug: bool,
    features: Vec<String>,
}

// Deserialize from Vexy JSON
let config: Config = from_str(json_with_comments)?;

// Serialize back to standard JSON
let json = to_string(&config)?;
```

#### Custom Types

```rust
use vexy_json::{Value, Map};

impl From<MyStruct> for Value {
    fn from(s: MyStruct) -> Self {
        let mut map = Map::new();
        map.insert("field1".to_string(), Value::String(s.field1));
        map.insert("field2".to_string(), Value::Number(s.field2.into()));
        Value::Object(map)
    }
}
```

### Advanced Features

#### Streaming for Large Files

```rust
use vexy_json::streaming::{StreamingParser, Event};

fn process_large_json(reader: impl Read) -> Result<(), Error> {
    let mut parser = StreamingParser::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        
        for event in parser.feed_chunk(&buffer[..bytes_read])? {
            match event {
                Event::ObjectStart => println!("Starting object"),
                Event::Key(key) => println!("Key: {}", key),
                Event::Value(value) => println!("Value: {:?}", value),
                Event::ObjectEnd => println!("Ending object"),
                _ => {}
            }
        }
    }
    
    parser.finalize()?;
    Ok(())
}
```

## Python

Python bindings provide a familiar API similar to the standard `json` module.

### Installation

```bash
pip install vexy-json
```

For development versions:

```bash
pip install git+https://github.com/vexyart/vexy-json.git
```

### Integration Patterns

#### Drop-in Replacement for `json`

```python
# Instead of: import json
import vexy_json as json

# All standard json functions work
data = json.loads('{"name": "test"}')
text = json.dumps(data)

# Plus forgiving features
config = json.loads('''
{
    // Configuration with comments
    "debug": true,
    "features": [
        "logging",
        "metrics", // Easy to add/remove
    ]
}
''')
```

#### Django Integration

```python
# settings.py
import vexy_json

def load_config():
    with open('config/app.json') as f:
        return vexy_json.load(f)

# Custom JSON field
from django.db import models

class ConfigField(models.JSONField):
    def to_python(self, value):
        if isinstance(value, str):
            return vexy_json.loads(value)
        return value
```

#### FastAPI Integration

```python
from fastapi import FastAPI, HTTPException
import vexy_json

app = FastAPI()

@app.post("/parse")
async def parse_json(data: str):
    try:
        result = vexy_json.parse(data)
        return {"parsed": result, "success": True}
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

### Error Handling

```python
import vexy_json
import logging

def safe_parse_config(filename):
    try:
        with open(filename) as f:
            return vexy_json.load(f)
    except FileNotFoundError:
        logging.error(f"Config file {filename} not found")
        return {}
    except vexy_json.ParseError as e:
        logging.error(f"JSON parse error in {filename}: {e}")
        if e.suggestion:
            logging.info(f"Suggestion: {e.suggestion}")
        return {}
```

### Async Support

```python
import asyncio
import vexy_json

async def parse_async(json_string):
    """Parse JSON in thread pool to avoid blocking"""
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, vexy_json.parse, json_string)

# Usage
result = await parse_async(large_json_string)
```

## JavaScript/WebAssembly

JavaScript bindings work in both Node.js and browsers via WebAssembly.

### Installation

```bash
npm install vexy-json-wasm
```

### Browser Setup

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { VexyJson } from './node_modules/vexy-json-wasm/vexy_json_wasm.js';
        
        async function main() {
            await init();
            
            const result = VexyJson.parse(`{
                "name": "Browser App",
                // This comment is OK
                "version": "1.0.0"
            }`);
            
            console.log(result);
        }
        
        main();
    </script>
</head>
<body>
    <h1>Vexy JSON Browser Example</h1>
</body>
</html>
```

### Node.js Setup

```javascript
// CommonJS
const { VexyJson } = require('vexy-json-wasm');

// ES Modules
import { VexyJson } from 'vexy-json-wasm';

const config = `
{
    // Server configuration
    "host": "localhost",
    "port": 3000,
    "features": {
        "cors": true,
        "compression": true, // Trailing comma OK
    }
}
`;

const parsed = VexyJson.parse(config);
console.log(parsed);
```

### Integration Patterns

#### Express.js Middleware

```javascript
import express from 'express';
import { VexyJson } from 'vexy-json-wasm';

const app = express();

// Custom JSON parser middleware
app.use('/api', (req, res, next) => {
    if (req.headers['content-type'] === 'application/vexy-json') {
        let body = '';
        req.on('data', chunk => body += chunk);
        req.on('end', () => {
            try {
                req.body = VexyJson.parse(body);
                next();
            } catch (error) {
                res.status(400).json({
                    error: 'Invalid JSON',
                    message: error.message,
                    line: error.line,
                    column: error.column
                });
            }
        });
    } else {
        next();
    }
});
```

#### React Hook

```javascript
import { useState, useEffect } from 'react';
import { VexyJson } from 'vexy-json-wasm';

function useVexyJson(jsonString) {
    const [data, setData] = useState(null);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);
    
    useEffect(() => {
        if (!jsonString) {
            setLoading(false);
            return;
        }
        
        try {
            const parsed = VexyJson.parse(jsonString);
            setData(parsed);
            setError(null);
        } catch (err) {
            setError(err);
            setData(null);
        } finally {
            setLoading(false);
        }
    }, [jsonString]);
    
    return { data, error, loading };
}

// Usage
function ConfigEditor({ configText }) {
    const { data, error, loading } = useVexyJson(configText);
    
    if (loading) return <div>Parsing...</div>;
    if (error) return <div>Error: {error.message}</div>;
    return <div>Config: {JSON.stringify(data, null, 2)}</div>;
}
```

### TypeScript Support

```typescript
import { VexyJson, Config, ParseError } from 'vexy-json-wasm';

interface AppConfig {
    name: string;
    version: string;
    features: string[];
}

function parseAppConfig(jsonString: string): AppConfig {
    try {
        const result = VexyJson.parse(jsonString) as AppConfig;
        
        // Runtime validation
        if (!result.name || !result.version) {
            throw new Error('Missing required fields');
        }
        
        return result;
    } catch (error) {
        if (error instanceof ParseError) {
            console.error(`Parse error at ${error.line}:${error.column}: ${error.message}`);
        }
        throw error;
    }
}
```

## C/C++

C and C++ bindings provide low-level access with minimal overhead.

### Installation

#### Using vcpkg

```bash
vcpkg install vexy-json
```

#### Building from Source

```bash
git clone https://github.com/vexyart/vexy-json.git
cd vexy-json
cargo build --release --features c-api
```

### C Integration

#### Basic Usage

```c
#include "vexy_json.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    const char* json = "{\n"
                       "  \"name\": \"Test App\",\n"
                       "  // Comment here\n"
                       "  \"version\": \"1.0.0\"\n"
                       "}";
    
    VexyJsonResult* result = vexy_json_parse(json);
    
    if (vexy_json_result_is_ok(result)) {
        printf("Parse successful!\n");
        
        // Access parsed data
        VexyJsonValue* root = vexy_json_result_value(result);
        
        if (vexy_json_value_is_object(root)) {
            VexyJsonValue* name = vexy_json_object_get(root, "name");
            if (name && vexy_json_value_is_string(name)) {
                const char* name_str = vexy_json_string_value(name);
                printf("Name: %s\n", name_str);
            }
        }
    } else {
        printf("Parse error: %s\n", vexy_json_result_error_message(result));
    }
    
    vexy_json_result_free(result);
    return 0;
}
```

#### Error Handling

```c
VexyJsonResult* result = vexy_json_parse(json);

if (!vexy_json_result_is_ok(result)) {
    VexyJsonError* error = vexy_json_result_error(result);
    
    printf("Error: %s\n", vexy_json_error_message(error));
    printf("Line: %zu, Column: %zu\n", 
           vexy_json_error_line(error),
           vexy_json_error_column(error));
    
    const char* suggestion = vexy_json_error_suggestion(error);
    if (suggestion) {
        printf("Suggestion: %s\n", suggestion);
    }
}
```

### C++ Integration

#### Modern C++ Style

```cpp
#include "vexy_json.hpp"
#include <iostream>
#include <fstream>

using namespace vexy_json;

class ConfigManager {
    Config config_;
    
public:
    ConfigManager() {
        config_.allow_comments = true;
        config_.allow_trailing_commas = true;
    }
    
    bool load_from_file(const std::string& filename) {
        std::ifstream file(filename);
        if (!file) return false;
        
        std::string content((std::istreambuf_iterator<char>(file)),
                           std::istreambuf_iterator<char>());
        
        try {
            auto value = parse_with_config(content, config_);
            process_config(value);
            return true;
        } catch (const ParseError& e) {
            std::cerr << "Parse error: " << e.what() << std::endl;
            std::cerr << "Location: " << e.line() << ":" << e.column() << std::endl;
            return false;
        }
    }
    
private:
    void process_config(const Value& value) {
        if (auto obj = value.as_object()) {
            for (const auto& [key, val] : *obj) {
                std::cout << key << ": " << val.to_string() << std::endl;
            }
        }
    }
};
```

#### RAII Wrapper

```cpp
class JsonParser {
    VexyJsonConfig config_;
    
public:
    JsonParser() {
        config_.allow_comments = true;
        config_.allow_trailing_commas = true;
        config_.max_depth = 128;
    }
    
    std::optional<Value> parse(const std::string& json) noexcept {
        try {
            return parse_with_config(json, config_);
        } catch (...) {
            return std::nullopt;
        }
    }
    
    Value parse_or_throw(const std::string& json) {
        return parse_with_config(json, config_);
    }
};
```

## Platform-Specific Considerations

### Memory Management

#### Rust
- Automatic via RAII and `Drop` trait
- Zero-copy options available
- Memory pools for high-performance scenarios

#### Python
- Automatic via reference counting
- Large objects released immediately when not referenced
- Compatible with Python's garbage collector

#### JavaScript/WASM
- Automatic via WASM/JavaScript boundary
- Manually call `free()` on large objects if needed
- Memory usage tracked by browser developer tools

#### C/C++
- Manual memory management required
- Always call `vexy_json_result_free()`
- Use RAII wrappers in C++ for safety

### Performance Characteristics

| Language | Parse Speed | Memory Usage | Startup Cost |
|----------|-------------|--------------|--------------|
| Rust | Fastest | Lowest | None |
| C/C++ | Fast | Low | None |
| Python | Medium | Medium | Low |
| JavaScript | Medium | Medium | WASM load |

### Threading

#### Rust
```rust
use std::sync::Arc;
use vexy_json::{Config, VexyJson};

let config = Arc::new(Config::permissive());
let json = Arc::new(json_string.clone());

std::thread::spawn(move || {
    let result = VexyJson::parse_with_config(&json, &config);
    // Process result
});
```

#### Python
```python
import threading
import vexy_json

def worker(json_strings):
    for json_str in json_strings:
        result = vexy_json.parse(json_str)
        # Process result

threads = []
for chunk in json_chunks:
    t = threading.Thread(target=worker, args=(chunk,))
    threads.append(t)
    t.start()
```

#### JavaScript
```javascript
// Web Workers for heavy parsing
const worker = new Worker('parser-worker.js');
worker.postMessage({ json: largeJsonString });
worker.onmessage = (e) => {
    const { result, error } = e.data;
    if (error) {
        console.error('Parse error:', error);
    } else {
        console.log('Parsed:', result);
    }
};
```