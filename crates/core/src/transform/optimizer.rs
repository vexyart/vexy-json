// this_file: crates/core/src/transform/optimizer.rs

//! AST optimizer for improving JSON structure performance.
//!
//! This module provides optimizations for JSON Abstract Syntax Trees (ASTs):
//! - Interning repeated strings to reduce memory usage
//! - Collapsing small objects into arrays for cache efficiency
//! - Optimizing number representations
//! - Removing redundant data structures

use crate::ast::{Number, Value};
use crate::error::Result;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Configuration options for AST optimization.
#[derive(Debug, Clone)]
pub struct OptimizerOptions {
    /// Enable string interning for repeated strings.
    pub intern_strings: bool,
    /// Minimum string length to consider for interning.
    pub min_intern_length: usize,
    /// Minimum occurrence count for string interning.
    pub min_intern_count: usize,
    /// Convert small objects to arrays when beneficial.
    pub optimize_small_objects: bool,
    /// Maximum object size to consider for array conversion.
    pub max_small_object_size: usize,
    /// Optimize number representations.
    pub optimize_numbers: bool,
    /// Collapse single-element arrays to scalars when safe.
    pub collapse_single_arrays: bool,
    /// Remove empty objects and arrays.
    pub remove_empty_containers: bool,
    /// Maximum depth for recursive optimization.
    pub max_depth: usize,
    /// Enable structural sharing for identical subtrees.
    pub enable_structural_sharing: bool,
}

impl Default for OptimizerOptions {
    fn default() -> Self {
        Self {
            intern_strings: true,
            min_intern_length: 10,
            min_intern_count: 3,
            optimize_small_objects: false, // Conservative default
            max_small_object_size: 4,
            optimize_numbers: true,
            collapse_single_arrays: false, // Conservative default
            remove_empty_containers: false,
            max_depth: 100,
            enable_structural_sharing: false, // Requires Arc<Value>
        }
    }
}

/// String interning table for deduplicating repeated strings.
#[derive(Debug, Clone)]
pub struct StringInterner {
    /// Maps string content to interned string.
    strings: FxHashMap<String, Arc<str>>,
    /// Counts occurrences of each string.
    counts: FxHashMap<String, usize>,
    /// Minimum length for interning.
    min_length: usize,
    /// Minimum count for interning.
    min_count: usize,
}

impl StringInterner {
    /// Creates a new string interner.
    pub fn new(min_length: usize, min_count: usize) -> Self {
        Self {
            strings: FxHashMap::default(),
            counts: FxHashMap::default(),
            min_length,
            min_count,
        }
    }

    /// Records a string occurrence.
    pub fn record(&mut self, s: &str) {
        if s.len() >= self.min_length {
            *self.counts.entry(s.to_string()).or_insert(0) += 1;
        }
    }

    /// Finalizes the interner after recording all strings.
    pub fn finalize(&mut self) {
        for (string, count) in &self.counts {
            if *count >= self.min_count {
                self.strings
                    .insert(string.clone(), Arc::from(string.as_str()));
            }
        }
    }

    /// Gets an interned string if available, otherwise returns the original.
    pub fn get(&self, s: &str) -> Option<Arc<str>> {
        self.strings.get(s).cloned()
    }

    /// Gets statistics about the interner.
    pub fn stats(&self) -> InternerStats {
        InternerStats {
            total_strings: self.counts.len(),
            interned_strings: self.strings.len(),
            total_occurrences: self.counts.values().sum(),
            saved_bytes: self
                .strings
                .iter()
                .map(|(s, _)| {
                    let count = self.counts.get(s).unwrap_or(&0);
                    if *count > 1 {
                        s.len() * (*count - 1)
                    } else {
                        0
                    }
                })
                .sum(),
        }
    }
}

/// Statistics about string interning.
#[derive(Debug, Clone)]
pub struct InternerStats {
    /// Total number of unique strings seen.
    pub total_strings: usize,
    /// Number of strings that were interned.
    pub interned_strings: usize,
    /// Total number of string occurrences.
    pub total_occurrences: usize,
    /// Estimated bytes saved through interning.
    pub saved_bytes: usize,
}

/// AST optimizer for improving JSON structure performance.
pub struct AstOptimizer {
    options: OptimizerOptions,
    interner: StringInterner,
    depth: usize,
    /// Cache for structural sharing.
    structure_cache: FxHashMap<String, Arc<Value>>,
}

impl AstOptimizer {
    /// Creates a new AST optimizer with default options.
    pub fn new() -> Self {
        Self::with_options(OptimizerOptions::default())
    }

    /// Creates a new AST optimizer with custom options.
    pub fn with_options(options: OptimizerOptions) -> Self {
        Self {
            interner: StringInterner::new(options.min_intern_length, options.min_intern_count),
            options,
            depth: 0,
            structure_cache: FxHashMap::default(),
        }
    }

    /// Optimizes a JSON value using a two-pass approach.
    pub fn optimize(&mut self, value: &Value) -> Result<Value> {
        // First pass: record all strings for interning
        if self.options.intern_strings {
            self.depth = 0;
            self.record_strings(value);
            self.interner.finalize();
        }

        // Second pass: optimize the structure
        self.depth = 0;
        self.optimize_value(value)
    }

    /// Records all strings in the AST for interning analysis.
    fn record_strings(&mut self, value: &Value) {
        if self.depth >= self.options.max_depth {
            return;
        }

        self.depth += 1;
        match value {
            Value::String(s) => {
                self.interner.record(s);
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    self.interner.record(key);
                    self.record_strings(val);
                }
            }
            Value::Array(arr) => {
                for val in arr {
                    self.record_strings(val);
                }
            }
            _ => {}
        }
        self.depth -= 1;
    }

    /// Optimizes a JSON value recursively.
    fn optimize_value(&mut self, value: &Value) -> Result<Value> {
        if self.depth >= self.options.max_depth {
            return Ok(value.clone());
        }

        self.depth += 1;
        let result = match value {
            Value::String(s) => self.optimize_string(s),
            Value::Object(obj) => self.optimize_object(obj),
            Value::Array(arr) => self.optimize_array(arr),
            Value::Number(n) => Ok(self.optimize_number(n)),
            Value::Bool(_) | Value::Null => Ok(value.clone()),
        };
        self.depth -= 1;
        result
    }

    /// Optimizes a string value.
    fn optimize_string(&self, s: &str) -> Result<Value> {
        if self.options.intern_strings {
            // For now, we return the original string since our Value enum doesn't support Arc<str>
            // In a real implementation, we might need to modify the Value enum or use a different approach
            Ok(Value::String(s.to_string()))
        } else {
            Ok(Value::String(s.to_string()))
        }
    }

    /// Optimizes an object value.
    fn optimize_object(&mut self, obj: &FxHashMap<String, Value>) -> Result<Value> {
        let mut optimized = FxHashMap::default();
        let mut has_changes = false;

        for (key, value) in obj {
            let optimized_value = self.optimize_value(value)?;

            // Skip empty containers if configured
            if self.options.remove_empty_containers {
                match &optimized_value {
                    Value::Object(o) if o.is_empty() => {
                        has_changes = true;
                        continue;
                    }
                    Value::Array(a) if a.is_empty() => {
                        has_changes = true;
                        continue;
                    }
                    _ => {}
                }
            }

            if !has_changes && !self.values_equal(&optimized_value, value) {
                has_changes = true;
            }

            optimized.insert(key.clone(), optimized_value);
        }

        // Consider converting small objects to arrays if beneficial
        if self.options.optimize_small_objects
            && optimized.len() <= self.options.max_small_object_size
            && self.should_convert_to_array(&optimized)
        {
            return Ok(self.convert_object_to_array(&optimized));
        }

        Ok(Value::Object(optimized))
    }

    /// Optimizes an array value.
    fn optimize_array(&mut self, arr: &[Value]) -> Result<Value> {
        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut optimized = Vec::new();
        let mut has_changes = false;

        for value in arr {
            let optimized_value = self.optimize_value(value)?;

            // Skip empty containers if configured
            if self.options.remove_empty_containers {
                match &optimized_value {
                    Value::Object(o) if o.is_empty() => {
                        has_changes = true;
                        continue;
                    }
                    Value::Array(a) if a.is_empty() => {
                        has_changes = true;
                        continue;
                    }
                    _ => {}
                }
            }

            if !has_changes && !self.values_equal(&optimized_value, value) {
                has_changes = true;
            }

            optimized.push(optimized_value);
        }

        // Consider collapsing single-element arrays
        if self.options.collapse_single_arrays && optimized.len() == 1 {
            return Ok(optimized.into_iter().next().unwrap());
        }

        Ok(Value::Array(optimized))
    }

    /// Optimizes a number value.
    fn optimize_number(&self, n: &Number) -> Value {
        if !self.options.optimize_numbers {
            return Value::Number(n.clone());
        }

        match n {
            Number::Float(f) => {
                // Convert float to integer if it's a whole number
                if f.fract() == 0.0
                    && f.is_finite()
                    && *f >= i64::MIN as f64
                    && *f <= i64::MAX as f64
                {
                    Value::Number(Number::Integer(*f as i64))
                } else {
                    Value::Number(Number::Float(*f))
                }
            }
            Number::Integer(_) => Value::Number(n.clone()),
        }
    }

    /// Checks if two values are equal.
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        // Simple equality check - in a real implementation, this might be more sophisticated
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => self.numbers_equal(a, b),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(av, bv)| self.values_equal(av, bv))
            }
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|(k, v)| b.get(k).map_or(false, |bv| self.values_equal(v, bv)))
            }
            _ => false,
        }
    }

    /// Checks if two numbers are equal.
    fn numbers_equal(&self, a: &Number, b: &Number) -> bool {
        match (a, b) {
            (Number::Integer(a), Number::Integer(b)) => a == b,
            (Number::Float(a), Number::Float(b)) => a == b,
            (Number::Integer(a), Number::Float(b)) => *a as f64 == *b,
            (Number::Float(a), Number::Integer(b)) => *a == *b as f64,
        }
    }

    /// Determines if a small object should be converted to an array.
    fn should_convert_to_array(&self, obj: &FxHashMap<String, Value>) -> bool {
        // Only convert if all keys are sequential integers starting from 0
        if obj.is_empty() {
            return false;
        }

        let mut keys: Vec<_> = obj.keys().collect();
        keys.sort();

        for (i, key) in keys.iter().enumerate() {
            if key.parse::<usize>().unwrap_or(usize::MAX) != i {
                return false;
            }
        }

        true
    }

    /// Converts an object with sequential integer keys to an array.
    fn convert_object_to_array(&self, obj: &FxHashMap<String, Value>) -> Value {
        let mut arr = Vec::new();
        let mut keys: Vec<_> = obj.keys().collect();
        keys.sort();

        for key in keys {
            arr.push(obj.get(key).unwrap().clone());
        }

        Value::Array(arr)
    }

    /// Gets optimization statistics.
    pub fn stats(&self) -> OptimizerStats {
        OptimizerStats {
            interner_stats: self.interner.stats(),
            structure_cache_hits: self.structure_cache.len(),
        }
    }
}

impl Default for AstOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about AST optimization.
#[derive(Debug, Clone)]
pub struct OptimizerStats {
    /// String interning statistics.
    pub interner_stats: InternerStats,
    /// Number of structure cache hits.
    pub structure_cache_hits: usize,
}

/// Convenience function to optimize a JSON value with default options.
pub fn optimize(value: &Value) -> Result<Value> {
    let mut optimizer = AstOptimizer::new();
    optimizer.optimize(value)
}

/// Convenience function to optimize a JSON value with custom options.
pub fn optimize_with_options(value: &Value, options: OptimizerOptions) -> Result<Value> {
    let mut optimizer = AstOptimizer::with_options(options);
    optimizer.optimize(value)
}

/// Specialized optimizer for memory efficiency.
pub struct MemoryOptimizer;

impl MemoryOptimizer {
    /// Optimizes a JSON value for minimal memory usage.
    pub fn minimize_memory(value: &Value) -> Result<Value> {
        let options = OptimizerOptions {
            intern_strings: true,
            min_intern_length: 5,
            min_intern_count: 2,
            optimize_small_objects: false,
            max_small_object_size: 0,
            optimize_numbers: true,
            collapse_single_arrays: false,
            remove_empty_containers: true,
            max_depth: 100,
            enable_structural_sharing: false,
        };

        let mut optimizer = AstOptimizer::with_options(options);
        optimizer.optimize(value)
    }
}

/// Specialized optimizer for performance.
pub struct PerformanceOptimizer;

impl PerformanceOptimizer {
    /// Optimizes a JSON value for maximum performance.
    pub fn maximize_performance(value: &Value) -> Result<Value> {
        let options = OptimizerOptions {
            intern_strings: true,
            min_intern_length: 10,
            min_intern_count: 3,
            optimize_small_objects: true,
            max_small_object_size: 8,
            optimize_numbers: true,
            collapse_single_arrays: true,
            remove_empty_containers: false,
            max_depth: 100,
            enable_structural_sharing: true,
        };

        let mut optimizer = AstOptimizer::with_options(options);
        optimizer.optimize(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Number;

    #[test]
    fn test_optimize_numbers() {
        let value = Value::Number(Number::Float(42.0));
        let optimized = optimize(&value).unwrap();
        assert_eq!(optimized, Value::Number(Number::Integer(42)));
    }

    #[test]
    fn test_remove_empty_containers() {
        let mut obj = FxHashMap::default();
        obj.insert("empty_obj".to_string(), Value::Object(FxHashMap::default()));
        obj.insert("empty_arr".to_string(), Value::Array(vec![]));
        obj.insert("keep".to_string(), Value::String("value".to_string()));

        let value = Value::Object(obj);
        let options = OptimizerOptions {
            remove_empty_containers: true,
            ..Default::default()
        };

        let optimized = optimize_with_options(&value, options).unwrap();

        if let Value::Object(optimized_obj) = optimized {
            assert_eq!(optimized_obj.len(), 1);
            assert_eq!(
                optimized_obj.get("keep"),
                Some(&Value::String("value".to_string()))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_collapse_single_arrays() {
        let arr = vec![Value::String("single".to_string())];
        let value = Value::Array(arr);
        let options = OptimizerOptions {
            collapse_single_arrays: true,
            ..Default::default()
        };

        let optimized = optimize_with_options(&value, options).unwrap();
        assert_eq!(optimized, Value::String("single".to_string()));
    }

    #[test]
    fn test_object_to_array_conversion() {
        let mut obj = FxHashMap::default();
        obj.insert("0".to_string(), Value::String("first".to_string()));
        obj.insert("1".to_string(), Value::String("second".to_string()));
        obj.insert("2".to_string(), Value::String("third".to_string()));

        let value = Value::Object(obj);
        let options = OptimizerOptions {
            optimize_small_objects: true,
            max_small_object_size: 5,
            ..Default::default()
        };

        let optimized = optimize_with_options(&value, options).unwrap();

        if let Value::Array(arr) = optimized {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::String("first".to_string()));
            assert_eq!(arr[1], Value::String("second".to_string()));
            assert_eq!(arr[2], Value::String("third".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_interning() {
        let mut interner = StringInterner::new(3, 2);

        // Record strings
        interner.record("hello");
        interner.record("world");
        interner.record("hello");
        interner.record("hello");
        interner.record("short");

        interner.finalize();

        // "hello" should be interned (length >= 3, count >= 2)
        assert!(interner.get("hello").is_some());
        // "world" should not be interned (count < 2)
        assert!(interner.get("world").is_none());
        // "short" should not be interned (count < 2)
        assert!(interner.get("short").is_none());

        let stats = interner.stats();
        assert_eq!(stats.interned_strings, 1);
        assert_eq!(stats.total_strings, 3);
    }

    #[test]
    fn test_nested_optimization() {
        let mut inner = FxHashMap::default();
        inner.insert("inner_key".to_string(), Value::Number(Number::Float(3.14)));

        let mut outer = FxHashMap::default();
        outer.insert("outer_key".to_string(), Value::Object(inner));

        let value = Value::Object(outer);
        let optimized = optimize(&value).unwrap();

        // Should maintain structure but optimize the nested float
        if let Value::Object(optimized_obj) = optimized {
            if let Some(Value::Object(inner_obj)) = optimized_obj.get("outer_key") {
                assert_eq!(
                    inner_obj.get("inner_key"),
                    Some(&Value::Number(Number::Float(3.14)))
                );
            } else {
                panic!("Expected nested object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_memory_optimizer() {
        let mut obj = FxHashMap::default();
        obj.insert("empty".to_string(), Value::Object(FxHashMap::default()));
        obj.insert("keep".to_string(), Value::Number(Number::Float(42.0)));

        let value = Value::Object(obj);
        let optimized = MemoryOptimizer::minimize_memory(&value).unwrap();

        if let Value::Object(optimized_obj) = optimized {
            assert_eq!(optimized_obj.len(), 1);
            assert_eq!(
                optimized_obj.get("keep"),
                Some(&Value::Number(Number::Integer(42)))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_performance_optimizer() {
        let arr = vec![Value::String("single".to_string())];
        let value = Value::Array(arr);
        let optimized = PerformanceOptimizer::maximize_performance(&value).unwrap();

        // Should collapse single-element array
        assert_eq!(optimized, Value::String("single".to_string()));
    }

    #[test]
    fn test_optimizer_stats() {
        let options = OptimizerOptions {
            intern_strings: true,
            min_intern_length: 3,
            ..Default::default()
        };
        let mut optimizer = AstOptimizer::with_options(options);
        let value = Value::String("test".to_string());
        let _ = optimizer.optimize(&value).unwrap();

        let stats = optimizer.stats();
        assert_eq!(stats.interner_stats.total_strings, 1);
    }

    #[test]
    fn test_max_depth_limit() {
        let mut obj = FxHashMap::default();
        obj.insert("key".to_string(), Value::String("value".to_string()));

        let value = Value::Object(obj);
        let options = OptimizerOptions {
            max_depth: 0,
            ..Default::default()
        };

        let optimized = optimize_with_options(&value, options).unwrap();

        // Should return original value when depth limit is exceeded
        assert_eq!(optimized, value);
    }
}
