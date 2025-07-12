//! AST visitor pattern for traversing and transforming JSON values
//!
//! This module provides a visitor pattern implementation for traversing
//! the JSON AST, allowing for analysis, transformation, and validation.

use crate::ast::{Number, Value};
use crate::error::Result;
use rustc_hash::FxHashMap;
use std::fmt;

/// Visitor trait for traversing JSON values
pub trait Visitor {
    /// Visit a value (dispatch method)
    fn visit_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Null => self.visit_null(),
            Value::Bool(b) => self.visit_bool(*b),
            Value::Number(n) => self.visit_number(n),
            Value::String(s) => self.visit_string(s),
            Value::Array(arr) => self.visit_array(arr),
            Value::Object(obj) => self.visit_object(obj),
        }
    }

    /// Visit a null value
    fn visit_null(&mut self) -> Result<()> {
        Ok(())
    }

    /// Visit a boolean value
    fn visit_bool(&mut self, _value: bool) -> Result<()> {
        Ok(())
    }

    /// Visit a number value
    fn visit_number(&mut self, _value: &Number) -> Result<()> {
        Ok(())
    }

    /// Visit a string value
    fn visit_string(&mut self, _value: &str) -> Result<()> {
        Ok(())
    }

    /// Visit an array value
    fn visit_array(&mut self, array: &[Value]) -> Result<()> {
        for value in array {
            self.visit_value(value)?;
        }
        Ok(())
    }

    /// Visit an object value
    fn visit_object(&mut self, object: &FxHashMap<String, Value>) -> Result<()> {
        for value in object.values() {
            self.visit_value(value)?;
        }
        Ok(())
    }
}

/// Mutable visitor trait for transforming JSON values
pub trait MutVisitor {
    /// Visit and potentially transform a value
    fn visit_value_mut(&mut self, value: &mut Value) -> Result<()> {
        match value {
            Value::Null => self.visit_null_mut(),
            Value::Bool(b) => self.visit_bool_mut(b),
            Value::Number(n) => self.visit_number_mut(n),
            Value::String(s) => self.visit_string_mut(s),
            Value::Array(arr) => self.visit_array_mut(arr),
            Value::Object(obj) => self.visit_object_mut(obj),
        }
    }

    /// Visit a null value
    fn visit_null_mut(&mut self) -> Result<()> {
        Ok(())
    }

    /// Visit a boolean value
    fn visit_bool_mut(&mut self, _value: &mut bool) -> Result<()> {
        Ok(())
    }

    /// Visit a number value
    fn visit_number_mut(&mut self, _value: &mut Number) -> Result<()> {
        Ok(())
    }

    /// Visit a string value
    fn visit_string_mut(&mut self, _value: &mut String) -> Result<()> {
        Ok(())
    }

    /// Visit an array value
    fn visit_array_mut(&mut self, array: &mut Vec<Value>) -> Result<()> {
        for value in array {
            self.visit_value_mut(value)?;
        }
        Ok(())
    }

    /// Visit an object value
    fn visit_object_mut(&mut self, object: &mut FxHashMap<String, Value>) -> Result<()> {
        for value in object.values_mut() {
            self.visit_value_mut(value)?;
        }
        Ok(())
    }
}

/// Path-aware visitor that tracks the current path in the JSON structure
pub trait PathVisitor {
    /// Visit a value with its path
    fn visit_value_with_path(&mut self, value: &Value, path: &JsonPath) -> Result<()> {
        match value {
            Value::Null => self.visit_null_with_path(path),
            Value::Bool(b) => self.visit_bool_with_path(*b, path),
            Value::Number(n) => self.visit_number_with_path(n, path),
            Value::String(s) => self.visit_string_with_path(s, path),
            Value::Array(arr) => self.visit_array_with_path(arr, path),
            Value::Object(obj) => self.visit_object_with_path(obj, path),
        }
    }

    /// Visit a null value with path
    fn visit_null_with_path(&mut self, _path: &JsonPath) -> Result<()> {
        Ok(())
    }

    /// Visit a boolean value with path
    fn visit_bool_with_path(&mut self, _value: bool, _path: &JsonPath) -> Result<()> {
        Ok(())
    }

    /// Visit a number value with path
    fn visit_number_with_path(&mut self, _value: &Number, _path: &JsonPath) -> Result<()> {
        Ok(())
    }

    /// Visit a string value with path
    fn visit_string_with_path(&mut self, _value: &str, _path: &JsonPath) -> Result<()> {
        Ok(())
    }

    /// Visit an array value with path
    fn visit_array_with_path(&mut self, array: &[Value], path: &JsonPath) -> Result<()> {
        for (i, value) in array.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(PathSegment::Index(i));
            self.visit_value_with_path(value, &child_path)?;
        }
        Ok(())
    }

    /// Visit an object value with path
    fn visit_object_with_path(
        &mut self,
        object: &FxHashMap<String, Value>,
        path: &JsonPath,
    ) -> Result<()> {
        for (key, value) in object {
            let mut child_path = path.clone();
            child_path.push(PathSegment::Key(key.clone()));
            self.visit_value_with_path(value, &child_path)?;
        }
        Ok(())
    }
}

/// JSON path representation
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPath {
    segments: Vec<PathSegment>,
}

/// Path segment in a JSON structure
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// Object key
    Key(String),
    /// Array index
    Index(usize),
}

impl JsonPath {
    /// Create a new empty path
    pub fn new() -> Self {
        JsonPath {
            segments: Vec::new(),
        }
    }

    /// Create a root path
    pub fn root() -> Self {
        Self::new()
    }

    /// Push a segment to the path
    pub fn push(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    /// Pop the last segment
    pub fn pop(&mut self) -> Option<PathSegment> {
        self.segments.pop()
    }

}

impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "$")?;
        for segment in &self.segments {
            match segment {
                PathSegment::Key(key) => write!(f, ".{}", key)?,
                PathSegment::Index(idx) => write!(f, "[{}]", idx)?,
            }
        }
        Ok(())
    }
}

impl Default for JsonPath {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to walk a value with a visitor
pub fn walk<V: Visitor>(value: &Value, visitor: &mut V) -> Result<()> {
    visitor.visit_value(value)
}

/// Helper function to walk a value with a mutable visitor
pub fn walk_mut<V: MutVisitor>(value: &mut Value, visitor: &mut V) -> Result<()> {
    visitor.visit_value_mut(value)
}

/// Helper function to walk a value with a path-aware visitor
pub fn walk_with_path<V: PathVisitor>(value: &Value, visitor: &mut V) -> Result<()> {
    let path = JsonPath::root();
    visitor.visit_value_with_path(value, &path)
}

// Example visitor implementations

/// A visitor that counts occurrences of each JSON value type.
///
/// This visitor can be used to gather statistics about a JSON document,
/// counting how many nulls, booleans, numbers, strings, arrays, and objects it contains.
#[derive(Debug, Default)]
pub struct CountingVisitor {
    /// Number of null values encountered.
    pub null_count: usize,
    /// Number of boolean values encountered.
    pub bool_count: usize,
    /// Number of numeric values encountered.
    pub number_count: usize,
    /// Number of string values encountered.
    pub string_count: usize,
    /// Number of arrays encountered.
    pub array_count: usize,
    /// Number of objects encountered.
    pub object_count: usize,
}

impl Visitor for CountingVisitor {
    fn visit_null(&mut self) -> Result<()> {
        self.null_count += 1;
        Ok(())
    }

    fn visit_bool(&mut self, _value: bool) -> Result<()> {
        self.bool_count += 1;
        Ok(())
    }

    fn visit_number(&mut self, _value: &Number) -> Result<()> {
        self.number_count += 1;
        Ok(())
    }

    fn visit_string(&mut self, _value: &str) -> Result<()> {
        self.string_count += 1;
        Ok(())
    }

    fn visit_array(&mut self, array: &[Value]) -> Result<()> {
        self.array_count += 1;
        for value in array {
            self.visit_value(value)?;
        }
        Ok(())
    }

    fn visit_object(&mut self, object: &FxHashMap<String, Value>) -> Result<()> {
        self.object_count += 1;
        for value in object.values() {
            self.visit_value(value)?;
        }
        Ok(())
    }
}

/// Visitor that collects all string values from a JSON document.
#[derive(Debug, Default)]
pub struct StringCollector {
    /// All string values found in the document.
    pub strings: Vec<String>,
}

impl Visitor for StringCollector {
    fn visit_string(&mut self, value: &str) -> Result<()> {
        self.strings.push(value.to_string());
        Ok(())
    }
}

/// Visitor that finds values at specific paths
pub struct PathFinder {
    target_path: String,
    results: Vec<Value>,
}

impl PathFinder {
    /// Create a new path finder for the given path
    pub fn new(path: &str) -> Self {
        PathFinder {
            target_path: path.to_string(),
            results: Vec::new(),
        }
    }

    /// Get the found values
    pub fn results(self) -> Vec<Value> {
        self.results
    }
}

impl PathVisitor for PathFinder {
    fn visit_value_with_path(&mut self, value: &Value, path: &JsonPath) -> Result<()> {
        if path.to_string() == self.target_path {
            self.results.push(value.clone());
        }

        // Continue traversing
        match value {
            Value::Array(arr) => self.visit_array_with_path(arr, path)?,
            Value::Object(obj) => self.visit_object_with_path(obj, path)?,
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::builder::ObjectBuilder;

    #[test]
    fn test_counting_visitor() {
        let value = ObjectBuilder::new()
            .string("name", "test")
            .integer("count", 42)
            .bool("active", true)
            .null("optional")
            .build()
            .unwrap();

        let mut visitor = CountingVisitor::default();
        walk(&value, &mut visitor).unwrap();

        assert_eq!(visitor.object_count, 1);
        assert_eq!(visitor.string_count, 1);
        assert_eq!(visitor.number_count, 1);
        assert_eq!(visitor.bool_count, 1);
        assert_eq!(visitor.null_count, 1);
    }

    #[test]
    fn test_string_collector() {
        let value = Value::Array(vec![
            Value::String("hello".to_string()),
            Value::Object({
                let mut map = FxHashMap::default();
                map.insert("key".to_string(), Value::String("world".to_string()));
                map
            }),
            Value::String("rust".to_string()),
        ]);

        let mut visitor = StringCollector::default();
        walk(&value, &mut visitor).unwrap();

        assert_eq!(visitor.strings, vec!["hello", "world", "rust"]);
    }

    #[test]
    fn test_path_visitor() {
        let value = ObjectBuilder::new()
            .insert(
                "user",
                ObjectBuilder::new()
                    .string("name", "Alice")
                    .integer("age", 30)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let mut finder = PathFinder::new("$.user.name");
        walk_with_path(&value, &mut finder).unwrap();

        let results = finder.results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], Value::String("Alice".to_string()));
    }

    #[test]
    fn test_mut_visitor() {
        let mut value = Value::Array(vec![
            Value::String("hello".to_string()),
            Value::String("world".to_string()),
        ]);

        struct UppercaseVisitor;
        impl MutVisitor for UppercaseVisitor {
            fn visit_string_mut(&mut self, value: &mut String) -> Result<()> {
                *value = value.to_uppercase();
                Ok(())
            }
        }

        let mut visitor = UppercaseVisitor;
        walk_mut(&mut value, &mut visitor).unwrap();

        match value {
            Value::Array(arr) => {
                assert_eq!(arr[0], Value::String("HELLO".to_string()));
                assert_eq!(arr[1], Value::String("WORLD".to_string()));
            }
            _ => panic!("Expected array"),
        }
    }
}
