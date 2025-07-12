# Vexy JSON Plugin Registry

## Overview

Vexy JSON supports a plugin system that allows extending the parser with custom functionality. This document serves as a registry of available plugins and a guide for creating new ones.

## Built-in Plugins

### Schema Validation Plugin

**Location**: `crates/core/src/plugin/plugins/schema_validation.rs`  
**Purpose**: Validate JSON against a schema  
**Usage**:
```rust
use vexy_json_core::plugin::plugins::SchemaValidationPlugin;

let schema = parse(r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#)?;
let validator = SchemaValidationPlugin::new(schema);
validator.validate(&parsed_json, "$")?;
```

### DateTime Plugin

**Location**: `crates/core/src/plugin/plugins/datetime.rs`  
**Purpose**: Parse ISO 8601 dates and convert them to structured objects  
**Usage**:
```rust
use vexy_json_core::plugin::plugins::DateTimePlugin;

let mut datetime_plugin = DateTimePlugin::new();
datetime_plugin.transform_value(&mut value, "$")?;
```

### Custom Number Format Plugin

**Location**: `crates/core/src/plugin/plugins/custom_number.rs`  
**Purpose**: Parse non-standard number formats (hex, binary, underscores)  
**Usage**:
```rust
use vexy_json_core::plugin::plugins::CustomNumberFormatPlugin;

let mut number_plugin = CustomNumberFormatPlugin::new();
let result = number_plugin.on_number("0xFF", "$")?;
```

### Comment Preservation Plugin

**Location**: `crates/core/src/plugin/plugins/comment_preservation.rs`  
**Purpose**: Preserve comments during parsing  
**Usage**:
```rust
use vexy_json_core::plugin::plugins::CommentPreservationPlugin;

let mut comment_plugin = CommentPreservationPlugin::new();
comment_plugin.add_comment("Description".to_string(), "$.field", false);
```

## Creating Custom Plugins

### Plugin Trait

All plugins must implement the `ParserPlugin` trait:

```rust
use vexy_json_core::plugin::ParserPlugin;
use vexy_json_core::ast::Value;
use vexy_json_core::error::Result;
use std::any::Any;

struct MyPlugin;

impl ParserPlugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
        // Transform the value
        Ok(())
    }

    fn validate(&self, value: &Value, path: &str) -> Result<()> {
        // Validate the value
        Ok(())
    }

    fn on_number(&mut self, value: &str, path: &str) -> Result<Value> {
        // Parse custom number formats
        Ok(Value::String(value.to_string()))
    }

    fn on_string(&mut self, value: &str, path: &str) -> Result<String> {
        // Transform string values
        Ok(value.to_string())
    }

    fn on_parse_start(&mut self, input: &str) -> Result<()> {
        // Called when parsing starts
        Ok(())
    }

    fn on_parse_end(&mut self, value: &Value) -> Result<()> {
        // Called when parsing ends
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
```

### Plugin Hooks

#### Transform Hook
- **Purpose**: Modify parsed values after parsing
- **When called**: After a value is parsed
- **Use cases**: Date parsing, string transformations, data normalization

#### Validate Hook
- **Purpose**: Validate parsed values
- **When called**: After transformation
- **Use cases**: Schema validation, business rule validation

#### Number Hook
- **Purpose**: Parse custom number formats
- **When called**: During lexing when a number is encountered
- **Use cases**: Hex/binary numbers, special float values, units

#### String Hook
- **Purpose**: Transform string values
- **When called**: During lexing when a string is encountered
- **Use cases**: Escape sequence handling, encoding conversion

### Plugin Integration

Plugins can be integrated into the parser in several ways:

#### Direct Integration
```rust
use vexy_json_core::parser::Parser;
use vexy_json_core::plugin::ParserPluginManager;

let mut manager = ParserPluginManager::new();
manager.register(Box::new(MyPlugin));

let mut parser = Parser::new_with_plugins(manager);
let result = parser.parse(json_string)?;
```

#### Parser Options
```rust
use vexy_json::{parse_with_options, ParserOptions};

let options = ParserOptions {
    plugins: vec![Box::new(MyPlugin)],
    ..Default::default()
};

let result = parse_with_options(json_string, options)?;
```

## Plugin Best Practices

### 1. Error Handling
Always use proper error handling and return meaningful error messages:

```rust
fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
    match value {
        Value::String(s) => {
            // Transform string
            Ok(())
        }
        _ => Err(Error::Custom(format!("Expected string at {}", path)))
    }
}
```

### 2. Performance Considerations
- Avoid expensive operations in hot paths
- Use lazy evaluation where possible
- Cache computed values when appropriate

### 3. Path Handling
Use the provided path parameter for error reporting and validation:

```rust
fn validate(&self, value: &Value, path: &str) -> Result<()> {
    if let Value::Object(obj) = value {
        for (key, val) in obj {
            let child_path = format!("{}.{}", path, key);
            self.validate(val, &child_path)?;
        }
    }
    Ok(())
}
```

### 4. State Management
Keep plugin state minimal and avoid global state:

```rust
struct MyPlugin {
    config: MyConfig,
    // Avoid: static mut GLOBAL_STATE
}
```

### 5. Testing
Write comprehensive tests for your plugins:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vexy_json::parse;

    #[test]
    fn test_my_plugin() {
        let mut plugin = MyPlugin::new();
        let mut value = parse(r#"{"test": "value"}"#).unwrap();
        plugin.transform_value(&mut value, "$").unwrap();
        // Assert expected behavior
    }
}
```

## Community Plugins

### Submitting Plugins

To submit a plugin to the registry:

1. Create a plugin following the guidelines above
2. Add comprehensive documentation
3. Include examples and tests
4. Submit a pull request with:
   - Plugin code in `crates/core/src/plugin/plugins/`
   - Documentation update to this registry
   - Example usage in `examples/`

### Plugin Categories

#### Data Transformation
- DateTime parsing and formatting
- Number format conversion
- String encoding/decoding
- Unit conversion

#### Validation
- Schema validation
- Business rule validation
- Data integrity checks
- Format validation

#### Parsing Extensions
- Custom comment styles
- Extended number formats
- Alternative string delimiters
- Macro expansion

#### Integration
- Database connectivity
- API validation
- Configuration management
- Templating support

## Performance Benchmarks

Plugin performance is tracked in the benchmark suite. Expected overhead:

- **Schema Validation**: ~30-50μs per validation
- **DateTime Parsing**: ~20-30μs per date field
- **Custom Numbers**: ~5-10μs per number
- **Comment Preservation**: ~10-20μs per comment

## Security Considerations

### Safe Plugin Development

1. **Input Validation**: Always validate plugin inputs
2. **Memory Safety**: Use safe Rust patterns
3. **Error Boundaries**: Handle errors gracefully
4. **Resource Limits**: Avoid unbounded resource usage

### Plugin Sandboxing

Future versions may include plugin sandboxing for untrusted plugins.

## API Stability

The plugin API is considered stable as of v2.0.0. Breaking changes will follow semantic versioning.

## Contributing

See `CONTRIBUTING.md` for details on contributing new plugins or improving existing ones.