// this_file: crates/core/src/optimization/value_builder.rs

//! Optimized value building to reduce allocations during parsing.

use crate::ast::{Number, Value};
use rustc_hash::FxHashMap;

/// A builder for constructing JSON values efficiently with pre-allocation.
pub struct ValueBuilder {
    // Pre-allocated capacity hints for better performance
    object_capacity: usize,
    array_capacity: usize,
}

impl ValueBuilder {
    /// Create a new ValueBuilder with default capacity hints.
    pub fn new() -> Self {
        Self {
            object_capacity: 8, // Start with reasonable defaults
            array_capacity: 16,
        }
    }

    /// Create a new ValueBuilder with specific capacity hints.
    pub fn with_capacity(object_capacity: usize, array_capacity: usize) -> Self {
        Self {
            object_capacity,
            array_capacity,
        }
    }

    /// Build an object with pre-allocated capacity.
    #[inline]
    pub fn build_object(&self) -> FxHashMap<String, Value> {
        FxHashMap::with_capacity_and_hasher(self.object_capacity, Default::default())
    }

    /// Build an array with pre-allocated capacity.
    #[inline]
    pub fn build_array(&self) -> Vec<Value> {
        Vec::with_capacity(self.array_capacity)
    }

    /// Build a string value, optimizing for the common case of no escape sequences.
    #[inline]
    pub fn build_string(content: &str) -> Value {
        // For small strings, check if we need unescaping
        if content.len() < 64 && !content.contains('\\') {
            Value::String(content.to_string())
        } else {
            // Use optimized unescaping for larger or escaped strings
            match super::string_parser::unescape_string_optimized(content) {
                Ok(unescaped) => Value::String(unescaped),
                Err(_) => Value::String(content.to_string()), // Fallback
            }
        }
    }

    /// Build a number value with optimized parsing.
    #[inline]
    pub fn build_number(s: &str) -> Result<Value, crate::error::Error> {
        let num = super::string_parser::parse_number_optimized(s)?;
        // Check if it's actually an integer
        let number_value = if num.fract() == 0.0
            && num.is_finite()
            && num >= i64::MIN as f64
            && num <= i64::MAX as f64
        {
            Number::Integer(num as i64)
        } else {
            Number::Float(num)
        };
        Ok(Value::Number(number_value))
    }
}

impl Default for ValueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_object() {
        let builder = ValueBuilder::new();
        let obj = builder.build_object();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_build_array() {
        let builder = ValueBuilder::new();
        let arr = builder.build_array();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_build_string() {
        let value = ValueBuilder::build_string("hello");
        match value {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_build_number() {
        let value = ValueBuilder::build_number("42").unwrap();
        match value {
            Value::Number(Number::Integer(42)) => {}
            Value::Number(Number::Float(f)) => assert_eq!(f, 42.0),
            _ => panic!("Expected number value"),
        }
    }
}
