# Schema Validation Plugin

## Overview

The Schema Validation Plugin provides JSON Schema validation capabilities for Vexy JSON. It allows you to validate parsed JSON against a schema definition.

## Features

- **Type Validation**: Validates basic JSON types (string, number, boolean, array, object, null)
- **Property Validation**: Validates object properties and required fields
- **Array Validation**: Validates array items against schema
- **Number Constraints**: Supports minimum/maximum validation
- **String Constraints**: Supports minLength/maxLength and pattern validation
- **Nested Validation**: Recursive validation of nested objects and arrays

## Usage

### Basic Usage

```rust
use vexy_json::{parse, ParserOptions};
use vexy_json_core::plugin::plugins::SchemaValidationPlugin;

// Define a schema
let schema = parse(r#"{
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "number", "minimum": 0},
        "email": {"type": "string", "pattern": "^[^@]+@[^@]+\\.[^@]+$"}
    },
    "required": ["name", "age"]
}"#)?;

// Create the plugin
let validator = SchemaValidationPlugin::new(schema);

// Validate JSON data
let data = parse(r#"{"name": "John", "age": 30, "email": "john@example.com"}"#)?;
validator.validate(&data, "$")?;
```

### Advanced Schema Features

```rust
// Complex schema with nested objects
let schema = parse(r#"{
    "type": "object",
    "properties": {
        "user": {
            "type": "object",
            "properties": {
                "profile": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "minLength": 2},
                        "bio": {"type": "string", "maxLength": 500}
                    }
                }
            }
        },
        "tags": {
            "type": "array",
            "items": {"type": "string"}
        }
    }
}"#)?;

let validator = SchemaValidationPlugin::new(schema);

// This will validate the entire structure
let data = parse(r#"{
    "user": {
        "profile": {
            "name": "Alice",
            "bio": "Software developer"
        }
    },
    "tags": ["rust", "json", "parser"]
}"#)?;

validator.validate(&data, "$")?;
```

## Schema Format

The plugin supports a subset of JSON Schema specification:

### Type Validation
```json
{
    "type": "string"    // Valid types: string, number, boolean, array, object, null
}
```

### Object Validation
```json
{
    "type": "object",
    "properties": {
        "field1": {"type": "string"},
        "field2": {"type": "number"}
    },
    "required": ["field1"]
}
```

### Array Validation
```json
{
    "type": "array",
    "items": {"type": "string"}
}
```

### Number Constraints
```json
{
    "type": "number",
    "minimum": 0,
    "maximum": 100
}
```

### String Constraints
```json
{
    "type": "string",
    "minLength": 2,
    "maxLength": 50,
    "pattern": "^[a-zA-Z]+$"
}
```

## Error Handling

The plugin provides detailed error messages with path information:

```rust
// This will fail validation
let invalid_data = parse(r#"{"name": 123, "age": -5}"#)?;

match validator.validate(&invalid_data, "$") {
    Ok(()) => println!("Valid"),
    Err(e) => println!("Validation error: {}", e),
    // Output: "Type mismatch at $.name: expected string, got number"
}
```

## Performance

- **Validation Time**: ~30-50μs per validation for typical schemas
- **Memory Usage**: Minimal overhead, schema is parsed once
- **Regex Compilation**: Patterns are compiled once and cached

## Limitations

Current limitations (may be addressed in future versions):

- No support for `$ref` references
- Limited to basic JSON Schema features
- No support for `allOf`, `oneOf`, `anyOf`
- No support for `additionalProperties`
- No support for format validation (email, uri, etc.)

## Integration with Parser

The plugin can be integrated into the parsing pipeline:

```rust
use vexy_json::{parse_with_options, ParserOptions};

// Create parser options with validation
let options = ParserOptions {
    validate_schema: Some(schema),
    ..Default::default()
};

// Parse and validate in one step
let result = parse_with_options(json_data, options)?;
```

## Examples

### Configuration File Validation

```rust
let config_schema = parse(r#"{
    "type": "object",
    "properties": {
        "server": {
            "type": "object",
            "properties": {
                "host": {"type": "string"},
                "port": {"type": "number", "minimum": 1, "maximum": 65535}
            },
            "required": ["host", "port"]
        },
        "database": {
            "type": "object",
            "properties": {
                "url": {"type": "string", "pattern": "^(mysql|postgresql)://"},
                "timeout": {"type": "number", "minimum": 1}
            },
            "required": ["url"]
        }
    },
    "required": ["server"]
}"#)?;

let validator = SchemaValidationPlugin::new(config_schema);

// Validate configuration
let config = parse(r#"{
    "server": {
        "host": "localhost",
        "port": 8080
    },
    "database": {
        "url": "postgresql://localhost:5432/mydb",
        "timeout": 30
    }
}"#)?;

validator.validate(&config, "$")?;
```

### API Response Validation

```rust
let api_schema = parse(r#"{
    "type": "object",
    "properties": {
        "status": {"type": "string"},
        "data": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "id": {"type": "number"},
                    "name": {"type": "string", "minLength": 1}
                },
                "required": ["id", "name"]
            }
        },
        "pagination": {
            "type": "object",
            "properties": {
                "page": {"type": "number", "minimum": 1},
                "total": {"type": "number", "minimum": 0}
            }
        }
    },
    "required": ["status", "data"]
}"#)?;

let validator = SchemaValidationPlugin::new(api_schema);

// Validate API response
let response = parse(r#"{
    "status": "success",
    "data": [
        {"id": 1, "name": "Item 1"},
        {"id": 2, "name": "Item 2"}
    ],
    "pagination": {
        "page": 1,
        "total": 2
    }
}"#)?;

validator.validate(&response, "$")?;
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vexy_json::parse;

    #[test]
    fn test_basic_validation() {
        let schema = parse(r#"{"type": "string"}"#).unwrap();
        let validator = SchemaValidationPlugin::new(schema);
        
        let valid = parse(r#""hello""#).unwrap();
        assert!(validator.validate(&valid, "$").is_ok());
        
        let invalid = parse(r#"123"#).unwrap();
        assert!(validator.validate(&invalid, "$").is_err());
    }
    
    #[test]
    fn test_object_validation() {
        let schema = parse(r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }"#).unwrap();
        
        let validator = SchemaValidationPlugin::new(schema);
        
        let valid = parse(r#"{"name": "test"}"#).unwrap();
        assert!(validator.validate(&valid, "$").is_ok());
        
        let invalid = parse(r#"{}"#).unwrap();
        assert!(validator.validate(&invalid, "$").is_err());
    }
}
```

## Future Enhancements

Planned features for future versions:

- Support for `$ref` references
- Additional JSON Schema features
- Performance optimizations
- Better error messages
- Schema composition (`allOf`, `oneOf`, `anyOf`)
- Format validation
- Custom validation functions