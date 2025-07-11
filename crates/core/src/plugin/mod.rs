//! Plugin system for extending Vexy JSON parser functionality.
//!
//! This module provides a flexible plugin architecture that allows:
//! - Custom value transformations
//! - Validation during parsing
//! - Schema enforcement
//! - Custom number/date formats
//! - Comment preservation
//! - And more custom extensions

use crate::ast::Value;
use crate::error::{Error, Result};
use rustc_hash::FxHashMap;
use std::any::Any;
use std::sync::{Arc, RwLock};

/// Trait for parser plugins
pub trait ParserPlugin: Send + Sync {
    /// Unique name of the plugin
    fn name(&self) -> &str;

    /// Called when parsing starts
    fn on_parse_start(&mut self, _input: &str) -> Result<()> {
        Ok(())
    }

    /// Called when parsing completes
    fn on_parse_end(&mut self, _value: &Value) -> Result<()> {
        Ok(())
    }

    /// Transform a value during parsing
    fn transform_value(&mut self, _value: &mut Value, _path: &str) -> Result<()> {
        Ok(())
    }

    /// Validate a value during parsing
    fn validate(&self, _value: &Value, _path: &str) -> Result<()> {
        Ok(())
    }

    /// Called for each key in an object
    fn on_object_key(&mut self, _key: &str, _path: &str) -> Result<()> {
        Ok(())
    }

    /// Called for each string value
    fn on_string(&mut self, value: &str, _path: &str) -> Result<String> {
        Ok(value.to_string())
    }

    /// Called for each number value
    fn on_number(&mut self, value: &str, _path: &str) -> Result<Value> {
        Ok(Value::Number(crate::ast::Number::Float(
            value.parse().map_err(|_| Error::InvalidNumber(0))?,
        )))
    }

    /// Get plugin-specific data
    fn as_any(&self) -> &dyn Any;

    /// Get mutable plugin-specific data
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Hook types for plugin system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginHook {
    /// Before parsing starts
    BeforeParse,
    /// After parsing completes
    AfterParse,
    /// When a value is created
    OnValue,
    /// When an object key is encountered
    OnObjectKey,
    /// When a string is parsed
    OnString,
    /// When a number is parsed
    OnNumber,
    /// During validation
    OnValidate,
}

/// Plugin registry for managing plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Vec<Box<dyn ParserPlugin>>,
    /// Hook mappings
    hooks: FxHashMap<PluginHook, Vec<usize>>,
    /// Plugin lookup by name
    plugin_map: FxHashMap<String, usize>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        PluginRegistry {
            plugins: Vec::new(),
            hooks: FxHashMap::default(),
            plugin_map: FxHashMap::default(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn ParserPlugin>) -> Result<()> {
        let name = plugin.name().to_string();

        if self.plugin_map.contains_key(&name) {
            return Err(Error::Custom(format!(
                "Plugin '{}' already registered",
                name
            )));
        }

        let index = self.plugins.len();
        self.plugins.push(plugin);
        self.plugin_map.insert(name, index);

        // Register hooks for this plugin
        self.register_hooks(index);

        Ok(())
    }

    /// Register hooks for a plugin
    fn register_hooks(&mut self, plugin_index: usize) {
        // All plugins get these hooks by default
        let hooks = vec![
            PluginHook::BeforeParse,
            PluginHook::AfterParse,
            PluginHook::OnValue,
            PluginHook::OnValidate,
        ];

        for hook in hooks {
            self.hooks
                .entry(hook)
                .or_insert_with(Vec::new)
                .push(plugin_index);
        }
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn ParserPlugin> {
        self.plugin_map.get(name).map(|&idx| &*self.plugins[idx])
    }

    /// Get a mutable plugin by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn ParserPlugin> {
        if let Some(&idx) = self.plugin_map.get(name) {
            Some(&mut *self.plugins[idx])
        } else {
            None
        }
    }

    /// Execute hook for all registered plugins
    pub fn execute_hook<F>(&mut self, hook: PluginHook, mut f: F) -> Result<()>
    where
        F: FnMut(&mut dyn ParserPlugin) -> Result<()>,
    {
        if let Some(indices) = self.hooks.get(&hook).cloned() {
            for idx in indices {
                f(&mut *self.plugins[idx])?;
            }
        }
        Ok(())
    }

    /// Transform a value through all plugins
    pub fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.transform_value(value, path)?;
        }
        Ok(())
    }

    /// Validate a value through all plugins
    pub fn validate(&self, value: &Value, path: &str) -> Result<()> {
        for plugin in &self.plugins {
            plugin.validate(value, path)?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe plugin registry
pub type SharedPluginRegistry = Arc<RwLock<PluginRegistry>>;

/// Create a shared plugin registry
pub fn create_shared_registry() -> SharedPluginRegistry {
    Arc::new(RwLock::new(PluginRegistry::new()))
}

// Re-export plugin implementations
pub mod plugins;

pub use plugins::{
    CommentPreservationPlugin, CustomNumberFormatPlugin, DateTimePlugin, SchemaValidationPlugin,
};

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        transform_count: usize,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            TestPlugin {
                name: name.to_string(),
                transform_count: 0,
            }
        }
    }

    impl ParserPlugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn transform_value(&mut self, _value: &mut Value, _path: &str) -> Result<()> {
            self.transform_count += 1;
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();

        let plugin = Box::new(TestPlugin::new("test"));
        registry.register(plugin).unwrap();

        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_duplicate_plugin() {
        let mut registry = PluginRegistry::new();

        registry
            .register(Box::new(TestPlugin::new("test")))
            .unwrap();
        let result = registry.register(Box::new(TestPlugin::new("test")));

        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_hooks() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(TestPlugin::new("test")))
            .unwrap();

        let mut count = 0;
        registry
            .execute_hook(PluginHook::BeforeParse, |plugin| {
                count += 1;
                plugin.on_parse_start("test")
            })
            .unwrap();

        assert_eq!(count, 1);
    }
}
