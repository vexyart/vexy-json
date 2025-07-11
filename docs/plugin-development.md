# Vexy JSON Plugin Development Guide

## Introduction

This guide will walk you through creating custom plugins for the Vexy JSON parser. Plugins allow you to extend the parser's functionality with custom transformations, validations, and parsing logic.

## Quick Start

Let's create a simple plugin that converts all string values to uppercase:

```rust
use vexy_json_core::plugin::ParserPlugin;
use vexy_json_core::ast::Value;
use vexy_json_core::error::Result;
use std::any::Any;

pub struct UppercasePlugin;

impl ParserPlugin for UppercasePlugin {
    fn name(&self) -> &str {
        "uppercase"
    }

    fn transform_value(&mut self, value: &mut Value, _path: &str) -> Result<()> {
        match value {
            Value::String(s) => {
                *s = s.to_uppercase();
            }
            Value::Object(obj) => {
                for (_, val) in obj.iter_mut() {
                    self.transform_value(val, _path)?;
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    self.transform_value(val, _path)?;
                }
            }
            _ => {}
        }
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

## Plugin Architecture

### Plugin Trait

The `ParserPlugin` trait defines the interface for all plugins:

```rust
pub trait ParserPlugin: Send + Sync {
    fn name(&self) -> &str;
    
    // Lifecycle hooks
    fn on_parse_start(&mut self, input: &str) -> Result<()> { Ok(()) }
    fn on_parse_end(&mut self, value: &Value) -> Result<()> { Ok(()) }
    
    // Value transformation
    fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> { Ok(()) }
    
    // Validation
    fn validate(&self, value: &Value, path: &str) -> Result<()> { Ok(()) }
    
    // Token-level hooks
    fn on_string(&mut self, value: &str, path: &str) -> Result<String> { Ok(value.to_string()) }
    fn on_number(&mut self, value: &str, path: &str) -> Result<Value> { 
        // Default implementation
        Ok(Value::String(value.to_string()))
    }
    fn on_object_key(&mut self, key: &str, path: &str) -> Result<()> { Ok(()) }
    
    // Type casting for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

### Plugin Execution Order

Plugins are executed in the following order:

1. **`on_parse_start`**: Called before parsing begins
2. **Token-level hooks**: Called during lexing/parsing
   - `on_string`: For string literals
   - `on_number`: For number literals
   - `on_object_key`: For object keys
3. **`transform_value`**: Called after parsing, traverses the AST
4. **`validate`**: Called after transformation
5. **`on_parse_end`**: Called after parsing completes

## Advanced Plugin Examples

### Configuration Plugin

A plugin that processes configuration files with environment variable substitution:

```rust
use std::env;
use std::collections::HashMap;
use regex::Regex;

pub struct ConfigPlugin {
    env_vars: HashMap<String, String>,
    prefix: String,
}

impl ConfigPlugin {
    pub fn new(prefix: &str) -> Self {
        let mut env_vars = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with(prefix) {
                env_vars.insert(key, value);
            }
        }
        
        ConfigPlugin {
            env_vars,
            prefix: prefix.to_string(),
        }
    }
    
    fn substitute_env_vars(&self, s: &str) -> String {
        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        re.replace_all(s, |caps: &regex::Captures| {
            let var_name = &caps[1];
            self.env_vars.get(var_name)
                .cloned()
                .unwrap_or_else(|| format!("${{{}}}", var_name))
        }).into_owned()
    }
}

impl ParserPlugin for ConfigPlugin {
    fn name(&self) -> &str {
        "config"
    }

    fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
        match value {
            Value::String(s) => {
                *s = self.substitute_env_vars(s);
            }
            Value::Object(obj) => {
                for (_, val) in obj.iter_mut() {
                    self.transform_value(val, path)?;
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    self.transform_value(val, path)?;
                }
            }
            _ => {}
        }
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

### Data Validation Plugin

A plugin that validates data against business rules:

```rust
use vexy_json_core::error::Error;

pub struct ValidationPlugin {
    rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    pub path_pattern: String,
    pub validator: Box<dyn Fn(&Value) -> Result<()> + Send + Sync>,
}

impl ValidationPlugin {
    pub fn new() -> Self {
        ValidationPlugin {
            rules: Vec::new(),
        }
    }
    
    pub fn add_rule<F>(&mut self, path_pattern: &str, validator: F) 
    where 
        F: Fn(&Value) -> Result<()> + Send + Sync + 'static 
    {
        self.rules.push(ValidationRule {
            path_pattern: path_pattern.to_string(),
            validator: Box::new(validator),
        });
    }
    
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob-style matching
        if pattern == "*" {
            return true;
        }
        
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }
        
        path == pattern
    }
}

impl ParserPlugin for ValidationPlugin {
    fn name(&self) -> &str {
        "validation"
    }

    fn validate(&self, value: &Value, path: &str) -> Result<()> {
        for rule in &self.rules {
            if self.matches_pattern(path, &rule.path_pattern) {
                (rule.validator)(value)?;
            }
        }
        
        // Recurse into nested values
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    let child_path = format!("{}.{}", path, key);
                    self.validate(val, &child_path)?;
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let child_path = format!("{}[{}]", path, i);
                    self.validate(val, &child_path)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Usage example
fn create_validation_plugin() -> ValidationPlugin {
    let mut plugin = ValidationPlugin::new();
    
    // Validate that age is a positive number
    plugin.add_rule("*.age", |value| {
        if let Value::Number(n) = value {
            if n.as_f64() < 0.0 {
                return Err(Error::Custom("Age must be positive".to_string()));
            }
        }
        Ok(())
    });
    
    // Validate email format
    plugin.add_rule("*.email", |value| {
        if let Value::String(s) = value {
            if !s.contains('@') {
                return Err(Error::Custom("Invalid email format".to_string()));
            }
        }
        Ok(())
    });
    
    plugin
}
```

### Macro Expansion Plugin

A plugin that expands custom macros in JSON:

```rust
use std::collections::HashMap;

pub struct MacroPlugin {
    macros: HashMap<String, Value>,
}

impl MacroPlugin {
    pub fn new() -> Self {
        MacroPlugin {
            macros: HashMap::new(),
        }
    }
    
    pub fn define_macro(&mut self, name: &str, value: Value) {
        self.macros.insert(name.to_string(), value);
    }
    
    fn expand_macro(&self, value: &Value) -> Option<Value> {
        if let Value::String(s) = value {
            if s.starts_with("$") {
                let macro_name = &s[1..];
                return self.macros.get(macro_name).cloned();
            }
        }
        None
    }
}

impl ParserPlugin for MacroPlugin {
    fn name(&self) -> &str {
        "macro"
    }

    fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
        // Try to expand macro first
        if let Some(expanded) = self.expand_macro(value) {
            *value = expanded;
            // Recursively process the expanded value
            self.transform_value(value, path)?;
            return Ok(());
        }
        
        // Process nested values
        match value {
            Value::Object(obj) => {
                for (_, val) in obj.iter_mut() {
                    self.transform_value(val, path)?;
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    self.transform_value(val, path)?;
                }
            }
            _ => {}
        }
        
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

## Testing Plugins

### Unit Testing

Create comprehensive unit tests for your plugins:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vexy_json::parse;

    #[test]
    fn test_uppercase_plugin() {
        let mut plugin = UppercasePlugin;
        let mut value = parse(r#"{"message": "hello world"}"#).unwrap();
        
        plugin.transform_value(&mut value, "$").unwrap();
        
        if let Value::Object(obj) = value {
            if let Some(Value::String(s)) = obj.get("message") {
                assert_eq!(s, "HELLO WORLD");
            } else {
                panic!("Expected string value");
            }
        } else {
            panic!("Expected object");
        }
    }
    
    #[test]
    fn test_config_plugin() {
        std::env::set_var("TEST_VAR", "test_value");
        
        let mut plugin = ConfigPlugin::new("TEST_");
        let mut value = parse(r#"{"config": "${TEST_VAR}"}"#).unwrap();
        
        plugin.transform_value(&mut value, "$").unwrap();
        
        if let Value::Object(obj) = value {
            if let Some(Value::String(s)) = obj.get("config") {
                assert_eq!(s, "test_value");
            } else {
                panic!("Expected string value");
            }
        } else {
            panic!("Expected object");
        }
    }
}
```

### Integration Testing

Test plugins with the full parser:

```rust
#[test]
fn test_plugin_integration() {
    use vexy_json::{parse_with_options, ParserOptions};
    
    let json = r#"{"name": "john", "age": 25}"#;
    let mut plugin = UppercasePlugin;
    
    // This would require parser integration
    // let options = ParserOptions::default().with_plugin(plugin);
    // let result = parse_with_options(json, options).unwrap();
    
    // For now, test manually
    let mut value = parse(json).unwrap();
    plugin.transform_value(&mut value, "$").unwrap();
    
    // Verify transformation
    assert_eq!(value.get("name").unwrap().as_str().unwrap(), "JOHN");
}
```

## Performance Considerations

### 1. Minimize Allocations

Avoid unnecessary allocations in hot paths:

```rust
// Good: Modify in place
fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
    if let Value::String(s) = value {
        s.make_ascii_uppercase(); // Modifies in place
    }
    Ok(())
}

// Avoid: Creating new strings
fn transform_value_slow(&mut self, value: &mut Value, path: &str) -> Result<()> {
    if let Value::String(s) = value {
        *s = s.to_uppercase(); // Creates new string
    }
    Ok(())
}
```

### 2. Use Efficient Data Structures

Choose appropriate data structures for your use case:

```rust
use rustc_hash::FxHashMap; // Faster than std::collections::HashMap
use indexmap::IndexMap;    // For ordered maps
use smallvec::SmallVec;    // For small vectors
```

### 3. Lazy Evaluation

Defer expensive operations until necessary:

```rust
pub struct LazyPlugin {
    cached_result: Option<Value>,
    input: String,
}

impl LazyPlugin {
    fn get_processed_value(&mut self) -> &Value {
        if self.cached_result.is_none() {
            self.cached_result = Some(self.expensive_computation());
        }
        self.cached_result.as_ref().unwrap()
    }
    
    fn expensive_computation(&self) -> Value {
        // Expensive operation here
        Value::String("computed".to_string())
    }
}
```

## Error Handling

### Custom Error Types

Create specific error types for your plugin:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Validation failed at {path}: {message}")]
    ValidationError { path: String, message: String },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Macro expansion failed: {macro_name}")]
    MacroError { macro_name: String },
}

impl From<PluginError> for vexy_json_core::error::Error {
    fn from(err: PluginError) -> Self {
        vexy_json_core::error::Error::Custom(err.to_string())
    }
}
```

### Error Recovery

Implement graceful error recovery:

```rust
fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
    match self.try_transform(value, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Log error but continue processing
            eprintln!("Warning: Plugin error at {}: {}", path, e);
            Ok(())
        }
    }
}
```

## Plugin Configuration

### Configuration Structs

Use configuration structs for complex plugins:

```rust
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub enabled: bool,
    pub max_depth: usize,
    pub custom_rules: Vec<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        PluginConfig {
            enabled: true,
            max_depth: 10,
            custom_rules: Vec::new(),
        }
    }
}

pub struct ConfigurablePlugin {
    config: PluginConfig,
}

impl ConfigurablePlugin {
    pub fn new(config: PluginConfig) -> Self {
        ConfigurablePlugin { config }
    }
}
```

### Builder Pattern

Use the builder pattern for complex plugin configuration:

```rust
pub struct PluginBuilder {
    config: PluginConfig,
}

impl PluginBuilder {
    pub fn new() -> Self {
        PluginBuilder {
            config: PluginConfig::default(),
        }
    }
    
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }
    
    pub fn add_rule(mut self, rule: String) -> Self {
        self.config.custom_rules.push(rule);
        self
    }
    
    pub fn build(self) -> ConfigurablePlugin {
        ConfigurablePlugin::new(self.config)
    }
}

// Usage
let plugin = PluginBuilder::new()
    .with_max_depth(5)
    .add_rule("validate_email".to_string())
    .build();
```

## Distribution and Packaging

### Cargo Features

Use Cargo features to make plugins optional:

```toml
[features]
default = ["builtin-plugins"]
builtin-plugins = ["datetime", "validation"]
datetime = ["chrono"]
validation = ["regex"]
```

### Plugin Crates

Create separate crates for complex plugins:

```toml
[package]
name = "vexy_json-plugin-myplugin"
version = "0.1.0"
edition = "2021"

[dependencies]
vexy_json-core = "2.0"
```

## Best Practices Summary

1. **Keep plugins focused**: Each plugin should have a single, clear purpose
2. **Use appropriate data structures**: Choose efficient collections and algorithms
3. **Handle errors gracefully**: Provide meaningful error messages and recovery
4. **Write comprehensive tests**: Test both success and failure cases
5. **Document your plugins**: Provide clear usage examples and API documentation
6. **Consider performance**: Profile your plugins and optimize hot paths
7. **Use configuration**: Make plugins configurable for different use cases
8. **Follow Rust conventions**: Use idiomatic Rust patterns and naming

## Next Steps

- Study the built-in plugins in `crates/core/src/plugin/plugins/`
- Create your own plugin following these patterns
- Submit your plugin to the community registry
- Contribute improvements to the plugin system

For more examples and detailed API documentation, see the `examples/plugin_examples.rs` file.