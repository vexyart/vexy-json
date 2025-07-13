//! AST builder for constructing JSON values efficiently
//!
//! This module provides a builder pattern for constructing AST nodes
//! with better ergonomics and performance optimizations.

use crate::ast::{Number, Value};
use crate::error::{Error, Result};
use rustc_hash::FxHashMap;

/// Builder for constructing JSON values with a fluent API
#[derive(Debug, Clone)]
pub struct ValueBuilder {
    stack: Vec<BuilderState>,
}

/// Internal state for the builder
#[derive(Debug, Clone)]
enum BuilderState {
    /// Building a single value
    Value(Value),
    /// Building an object
    Object {
        map: FxHashMap<String, Value>,
        current_key: Option<String>,
    },
    /// Building an array
    Array { vec: Vec<Value> },
}

impl ValueBuilder {
    /// Create a new value builder
    pub fn new() -> Self {
        ValueBuilder { stack: vec![] }
    }

    /// Start building an object
    pub fn object(mut self) -> Self {
        self.stack.push(BuilderState::Object {
            map: FxHashMap::default(),
            current_key: None,
        });
        self
    }

    /// Start building an array
    pub fn array(mut self) -> Self {
        self.stack.push(BuilderState::Array { vec: Vec::new() });
        self
    }

    /// Add a key to the current object (must be building an object)
    pub fn key<S: Into<String>>(mut self, key: S) -> Result<Self> {
        match self.stack.last_mut() {
            Some(BuilderState::Object { current_key, .. }) => {
                *current_key = Some(key.into());
                Ok(self)
            }
            _ => Err(Error::Custom(
                "Cannot add key when not building an object".to_string(),
            )),
        }
    }

    /// Add a string value
    pub fn string<S: Into<String>>(self, value: S) -> Result<Self> {
        self.add_value(Value::String(value.into()))
    }

    /// Add a number value (integer)
    pub fn integer(self, value: i64) -> Result<Self> {
        self.add_value(Value::Number(Number::Integer(value)))
    }

    /// Add a number value (float)
    pub fn float(self, value: f64) -> Result<Self> {
        self.add_value(Value::Number(Number::Float(value)))
    }

    /// Add a boolean value
    pub fn bool(self, value: bool) -> Result<Self> {
        self.add_value(Value::Bool(value))
    }

    /// Add a null value
    pub fn null(self) -> Result<Self> {
        self.add_value(Value::Null)
    }

    /// Add a pre-built value
    pub fn value(self, value: Value) -> Result<Self> {
        self.add_value(value)
    }

    /// End the current object or array
    pub fn end(mut self) -> Result<Self> {
        if self.stack.is_empty() {
            return Err(Error::Custom("No object or array to end".to_string()));
        }

        let completed = self.stack.pop().unwrap();
        let value = match completed {
            BuilderState::Object { map, .. } => Value::Object(map),
            BuilderState::Array { vec } => Value::Array(vec),
            BuilderState::Value(v) => v,
        };

        // If stack is empty, we're done
        if self.stack.is_empty() {
            self.stack.push(BuilderState::Value(value));
        } else {
            // Otherwise, add to parent
            self = self.add_value(value)?;
        }

        Ok(self)
    }

    /// Build the final value
    pub fn build(mut self) -> Result<Value> {
        // Auto-close any open structures
        while self.stack.len() > 1 {
            self = self.end()?;
        }

        match self.stack.pop() {
            Some(BuilderState::Value(v)) => Ok(v),
            Some(BuilderState::Object { map, .. }) => Ok(Value::Object(map)),
            Some(BuilderState::Array { vec }) => Ok(Value::Array(vec)),
            None => Err(Error::Custom("No value built".to_string())),
        }
    }

    /// Internal method to add a value
    fn add_value(mut self, value: Value) -> Result<Self> {
        match self.stack.last_mut() {
            Some(BuilderState::Object { map, current_key }) => match current_key.take() {
                Some(key) => {
                    map.insert(key, value);
                    Ok(self)
                }
                None => Err(Error::Custom(
                    "Object requires key before value".to_string(),
                )),
            },
            Some(BuilderState::Array { vec }) => {
                vec.push(value);
                Ok(self)
            }
            Some(BuilderState::Value(_)) => Err(Error::Custom(
                "Cannot add value to a completed value".to_string(),
            )),
            None => {
                self.stack.push(BuilderState::Value(value));
                Ok(self)
            }
        }
    }
}

impl Default for ValueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for building values
pub mod build {
    use super::*;

    /// Build an object with a closure
    pub fn object<F>(f: F) -> Result<Value>
    where
        F: FnOnce(ObjectBuilder) -> Result<ObjectBuilder>,
    {
        let builder = ObjectBuilder::new();
        f(builder)?.build()
    }

    /// Build an array with a closure
    pub fn array<F>(f: F) -> Result<Value>
    where
        F: FnOnce(ArrayBuilder) -> Result<ArrayBuilder>,
    {
        let builder = ArrayBuilder::new();
        f(builder)?.build()
    }
}

/// Specialized builder for objects
pub struct ObjectBuilder {
    map: FxHashMap<String, Value>,
}

impl ObjectBuilder {
    /// Create a new object builder
    pub fn new() -> Self {
        ObjectBuilder {
            map: FxHashMap::default(),
        }
    }

    /// Add a key-value pair
    pub fn insert<S: Into<String>>(mut self, key: S, value: Value) -> Self {
        self.map.insert(key.into(), value);
        self
    }

    /// Add a string value
    pub fn string<S: Into<String>, V: Into<String>>(self, key: S, value: V) -> Self {
        self.insert(key, Value::String(value.into()))
    }

    /// Add an integer value
    pub fn integer<S: Into<String>>(self, key: S, value: i64) -> Self {
        self.insert(key, Value::Number(Number::Integer(value)))
    }

    /// Add a float value
    pub fn float<S: Into<String>>(self, key: S, value: f64) -> Self {
        self.insert(key, Value::Number(Number::Float(value)))
    }

    /// Add a boolean value
    pub fn bool<S: Into<String>>(self, key: S, value: bool) -> Self {
        self.insert(key, Value::Bool(value))
    }

    /// Add a null value
    pub fn null<S: Into<String>>(self, key: S) -> Self {
        self.insert(key, Value::Null)
    }

    /// Build the object
    pub fn build(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized builder for arrays
pub struct ArrayBuilder {
    vec: Vec<Value>,
}

impl ArrayBuilder {
    /// Create a new array builder
    pub fn new() -> Self {
        ArrayBuilder { vec: Vec::new() }
    }

    /// Add a value
    pub fn push(mut self, value: Value) -> Self {
        self.vec.push(value);
        self
    }

    /// Add a string value
    pub fn string<S: Into<String>>(self, value: S) -> Self {
        self.push(Value::String(value.into()))
    }

    /// Add an integer value
    pub fn integer(self, value: i64) -> Self {
        self.push(Value::Number(Number::Integer(value)))
    }

    /// Add a float value
    pub fn float(self, value: f64) -> Self {
        self.push(Value::Number(Number::Float(value)))
    }

    /// Add a boolean value
    pub fn bool(self, value: bool) -> Self {
        self.push(Value::Bool(value))
    }

    /// Add a null value
    pub fn null(self) -> Self {
        self.push(Value::Null)
    }

    /// Build the array
    pub fn build(self) -> Result<Value> {
        Ok(Value::Array(self.vec))
    }
}

impl Default for ArrayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_builder_simple() {
        let value = ValueBuilder::new()
            .string("hello")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_value_builder_object() {
        let value = ValueBuilder::new()
            .object()
            .key("name")
            .unwrap()
            .string("John")
            .unwrap()
            .key("age")
            .unwrap()
            .integer(30)
            .unwrap()
            .key("active")
            .unwrap()
            .bool(true)
            .unwrap()
            .end()
            .unwrap()
            .build()
            .unwrap();

        match value {
            Value::Object(map) => {
                assert_eq!(map.get("name"), Some(&Value::String("John".to_string())));
                assert_eq!(map.get("age"), Some(&Value::Number(Number::Integer(30))));
                assert_eq!(map.get("active"), Some(&Value::Bool(true)));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_value_builder_array() {
        let value = ValueBuilder::new()
            .array()
            .integer(1)
            .unwrap()
            .integer(2)
            .unwrap()
            .integer(3)
            .unwrap()
            .end()
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(
            value,
            Value::Array(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
                Value::Number(Number::Integer(3)),
            ])
        );
    }

    #[test]
    fn test_value_builder_nested() {
        let value = ValueBuilder::new()
            .object()
            .key("user")
            .unwrap()
            .object()
            .key("name")
            .unwrap()
            .string("Alice")
            .unwrap()
            .key("scores")
            .unwrap()
            .array()
            .integer(100)
            .unwrap()
            .integer(95)
            .unwrap()
            .integer(87)
            .unwrap()
            .end()
            .unwrap()
            .end()
            .unwrap()
            .end()
            .unwrap()
            .build()
            .unwrap();

        match value {
            Value::Object(map) => match map.get("user") {
                Some(Value::Object(user_map)) => {
                    assert_eq!(
                        user_map.get("name"),
                        Some(&Value::String("Alice".to_string()))
                    );
                    match user_map.get("scores") {
                        Some(Value::Array(scores)) => {
                            assert_eq!(scores.len(), 3);
                        }
                        _ => panic!("Expected scores array"),
                    }
                }
                _ => panic!("Expected user object"),
            },
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_object_builder() {
        let value = ObjectBuilder::new()
            .string("name", "Bob")
            .integer("age", 25)
            .bool("active", false)
            .null("optional")
            .build()
            .unwrap();

        match value {
            Value::Object(map) => {
                assert_eq!(map.len(), 4);
                assert_eq!(map.get("name"), Some(&Value::String("Bob".to_string())));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_array_builder() {
        let value = ArrayBuilder::new()
            .string("hello")
            .integer(42)
            .float(3.14159)
            .bool(true)
            .null()
            .build()
            .unwrap();

        match value {
            Value::Array(vec) => {
                assert_eq!(vec.len(), 5);
                assert_eq!(vec[0], Value::String("hello".to_string()));
                assert_eq!(vec[4], Value::Null);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_build_helpers() {
        use build::*;

        let obj = object(|b| Ok(b.string("key", "value").integer("num", 42))).unwrap();

        let arr = array(|b| Ok(b.integer(1).integer(2).integer(3))).unwrap();

        assert!(matches!(obj, Value::Object(_)));
        assert!(matches!(arr, Value::Array(_)));
    }
}
