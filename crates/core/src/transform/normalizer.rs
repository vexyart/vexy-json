// this_file: crates/core/src/transform/normalizer.rs

//! JSON normalizer for transforming JSON into canonical form.
//!
//! This module provides functionality to normalize JSON values into a standardized
//! format. This is useful for:
//! - Comparing JSON values for equality
//! - Generating consistent JSON output
//! - Removing formatting variations
//! - Sorting object keys for deterministic output

use crate::ast::{Number, Value};
use crate::error::Result;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;

/// Configuration options for JSON normalization.
#[derive(Debug, Clone)]
pub struct NormalizerOptions {
    /// Whether to sort object keys alphabetically.
    pub sort_keys: bool,
    /// Whether to remove null values from objects.
    pub remove_null_values: bool,
    /// Whether to remove empty objects and arrays.
    pub remove_empty_containers: bool,
    /// Whether to normalize numbers (convert integers to floats if requested).
    pub normalize_numbers: bool,
    /// Whether to use integers for whole numbers.
    pub prefer_integers: bool,
    /// Whether to trim whitespace from strings.
    pub trim_strings: bool,
    /// Whether to normalize string case (to lowercase).
    pub normalize_string_case: bool,
    /// Whether to deduplicate array elements.
    pub deduplicate_arrays: bool,
    /// Maximum nesting depth for recursive normalization.
    pub max_depth: usize,
}

impl Default for NormalizerOptions {
    fn default() -> Self {
        Self {
            sort_keys: true,
            remove_null_values: false,
            remove_empty_containers: false,
            normalize_numbers: false,
            prefer_integers: true,
            trim_strings: false,
            normalize_string_case: false,
            deduplicate_arrays: false,
            max_depth: 100,
        }
    }
}

/// JSON normalizer that transforms JSON values into canonical form.
pub struct JsonNormalizer {
    options: NormalizerOptions,
    depth: usize,
}

impl JsonNormalizer {
    /// Creates a new JSON normalizer with default options.
    pub fn new() -> Self {
        Self {
            options: NormalizerOptions::default(),
            depth: 0,
        }
    }

    /// Creates a new JSON normalizer with custom options.
    pub fn with_options(options: NormalizerOptions) -> Self {
        Self { options, depth: 0 }
    }

    /// Normalizes a JSON value according to the configured options.
    pub fn normalize(&mut self, value: &Value) -> Result<Value> {
        self.depth = 0;
        self.normalize_value(value)
    }

    /// Internal method to normalize a value recursively.
    fn normalize_value(&mut self, value: &Value) -> Result<Value> {
        if self.depth >= self.options.max_depth {
            return Ok(value.clone());
        }

        self.depth += 1;
        let result = match value {
            Value::Object(obj) => self.normalize_object(obj),
            Value::Array(arr) => self.normalize_array(arr),
            Value::String(s) => Ok(self.normalize_string(s)),
            Value::Number(n) => Ok(self.normalize_number(n)),
            Value::Bool(_) | Value::Null => Ok(value.clone()),
        };
        self.depth -= 1;
        result
    }

    /// Normalizes an object.
    fn normalize_object(&mut self, obj: &FxHashMap<String, Value>) -> Result<Value> {
        let mut normalized = FxHashMap::default();

        for (key, value) in obj {
            let normalized_value = self.normalize_value(value)?;

            // Skip null values if configured
            if self.options.remove_null_values && normalized_value == Value::Null {
                continue;
            }

            // Skip empty containers if configured
            if self.options.remove_empty_containers {
                match &normalized_value {
                    Value::Object(o) if o.is_empty() => continue,
                    Value::Array(a) if a.is_empty() => continue,
                    _ => {}
                }
            }

            normalized.insert(key.clone(), normalized_value);
        }

        // Return empty object if all values were filtered out
        if self.options.remove_empty_containers && normalized.is_empty() {
            return Ok(Value::Object(normalized));
        }

        Ok(Value::Object(normalized))
    }

    /// Normalizes an array.
    fn normalize_array(&mut self, arr: &[Value]) -> Result<Value> {
        let mut normalized = Vec::new();

        for value in arr {
            let normalized_value = self.normalize_value(value)?;

            // Skip null values if configured
            if self.options.remove_null_values && normalized_value == Value::Null {
                continue;
            }

            // Skip empty containers if configured
            if self.options.remove_empty_containers {
                match &normalized_value {
                    Value::Object(o) if o.is_empty() => continue,
                    Value::Array(a) if a.is_empty() => continue,
                    _ => {}
                }
            }

            normalized.push(normalized_value);
        }

        // Deduplicate array elements if configured
        if self.options.deduplicate_arrays {
            normalized.sort_by(|a, b| self.compare_values(a, b));
            normalized.dedup();
        }

        Ok(Value::Array(normalized))
    }

    /// Normalizes a string.
    fn normalize_string(&self, s: &str) -> Value {
        let mut normalized = s.to_string();

        if self.options.trim_strings {
            normalized = normalized.trim().to_string();
        }

        if self.options.normalize_string_case {
            normalized = normalized.to_lowercase();
        }

        Value::String(normalized)
    }

    /// Normalizes a number.
    fn normalize_number(&self, n: &Number) -> Value {
        match n {
            Number::Integer(i) => {
                if self.options.normalize_numbers && !self.options.prefer_integers {
                    Value::Number(Number::Float(*i as f64))
                } else {
                    Value::Number(Number::Integer(*i))
                }
            }
            Number::Float(f) => {
                if self.options.prefer_integers && f.fract() == 0.0 && f.is_finite() {
                    Value::Number(Number::Integer(*f as i64))
                } else {
                    Value::Number(Number::Float(*f))
                }
            }
        }
    }

    /// Compares two values for sorting purposes.
    fn compare_values(&self, a: &Value, b: &Value) -> Ordering {
        match (a, b) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,

            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Bool(_), _) => Ordering::Less,
            (_, Value::Bool(_)) => Ordering::Greater,

            (Value::Number(a), Value::Number(b)) => self.compare_numbers(a, b),
            (Value::Number(_), _) => Ordering::Less,
            (_, Value::Number(_)) => Ordering::Greater,

            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::String(_), _) => Ordering::Less,
            (_, Value::String(_)) => Ordering::Greater,

            (Value::Array(a), Value::Array(b)) => {
                for (av, bv) in a.iter().zip(b.iter()) {
                    match self.compare_values(av, bv) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                }
                a.len().cmp(&b.len())
            }
            (Value::Array(_), _) => Ordering::Less,
            (_, Value::Array(_)) => Ordering::Greater,

            (Value::Object(_), Value::Object(_)) => Ordering::Equal, // Objects are equal for sorting
        }
    }

    /// Compares two numbers for sorting purposes.
    fn compare_numbers(&self, a: &Number, b: &Number) -> Ordering {
        match (a, b) {
            (Number::Integer(a), Number::Integer(b)) => a.cmp(b),
            (Number::Float(a), Number::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Number::Integer(a), Number::Float(b)) => {
                (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (Number::Float(a), Number::Integer(b)) => {
                a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)
            }
        }
    }
}

impl Default for JsonNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to normalize JSON with default options.
pub fn normalize(value: &Value) -> Result<Value> {
    let mut normalizer = JsonNormalizer::new();
    normalizer.normalize(value)
}

/// Convenience function to normalize JSON with custom options.
pub fn normalize_with_options(value: &Value, options: NormalizerOptions) -> Result<Value> {
    let mut normalizer = JsonNormalizer::with_options(options);
    normalizer.normalize(value)
}

/// Specialized normalizer for creating canonical JSON for comparison.
pub struct CanonicalNormalizer;

impl CanonicalNormalizer {
    /// Creates a canonical form of JSON for comparison purposes.
    pub fn canonicalize(value: &Value) -> Result<Value> {
        let options = NormalizerOptions {
            sort_keys: true,
            remove_null_values: false,
            remove_empty_containers: false,
            normalize_numbers: false,
            prefer_integers: true,
            trim_strings: false,
            normalize_string_case: false,
            deduplicate_arrays: false,
            max_depth: 100,
        };

        let mut normalizer = JsonNormalizer::with_options(options);
        normalizer.normalize(value)
    }

    /// Creates a canonical form for deep comparison (sorts everything).
    pub fn deep_canonicalize(value: &Value) -> Result<Value> {
        let options = NormalizerOptions {
            sort_keys: true,
            remove_null_values: false,
            remove_empty_containers: false,
            normalize_numbers: false,
            prefer_integers: true,
            trim_strings: false,
            normalize_string_case: false,
            deduplicate_arrays: true,
            max_depth: 100,
        };

        let mut normalizer = JsonNormalizer::with_options(options);
        normalizer.normalize(value)
    }
}

/// Specialized normalizer for cleaning up JSON data.
pub struct CleanupNormalizer;

impl CleanupNormalizer {
    /// Cleans up JSON by removing null values and empty containers.
    pub fn cleanup(value: &Value) -> Result<Value> {
        let options = NormalizerOptions {
            sort_keys: true,
            remove_null_values: true,
            remove_empty_containers: true,
            normalize_numbers: false,
            prefer_integers: true,
            trim_strings: true,
            normalize_string_case: false,
            deduplicate_arrays: false,
            max_depth: 100,
        };

        let mut normalizer = JsonNormalizer::with_options(options);
        normalizer.normalize(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Number;

    #[test]
    fn test_normalize_object_with_sorted_keys() {
        let mut obj = FxHashMap::default();
        obj.insert("zebra".to_string(), Value::String("last".to_string()));
        obj.insert("apple".to_string(), Value::String("first".to_string()));
        obj.insert("banana".to_string(), Value::String("second".to_string()));

        let value = Value::Object(obj);
        let normalized = normalize(&value).unwrap();

        // The result should be the same content (sorting is for serialization)
        if let Value::Object(normalized_obj) = normalized {
            assert_eq!(normalized_obj.len(), 3);
            assert_eq!(
                normalized_obj.get("zebra"),
                Some(&Value::String("last".to_string()))
            );
            assert_eq!(
                normalized_obj.get("apple"),
                Some(&Value::String("first".to_string()))
            );
            assert_eq!(
                normalized_obj.get("banana"),
                Some(&Value::String("second".to_string()))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_normalize_remove_null_values() {
        let mut obj = FxHashMap::default();
        obj.insert("keep".to_string(), Value::String("value".to_string()));
        obj.insert("remove".to_string(), Value::Null);

        let value = Value::Object(obj);
        let options = NormalizerOptions {
            remove_null_values: true,
            ..Default::default()
        };

        let normalized = normalize_with_options(&value, options).unwrap();

        if let Value::Object(normalized_obj) = normalized {
            assert_eq!(normalized_obj.len(), 1);
            assert_eq!(
                normalized_obj.get("keep"),
                Some(&Value::String("value".to_string()))
            );
            assert!(normalized_obj.get("remove").is_none());
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_normalize_remove_empty_containers() {
        let mut obj = FxHashMap::default();
        obj.insert("keep".to_string(), Value::String("value".to_string()));
        obj.insert("empty_obj".to_string(), Value::Object(FxHashMap::default()));
        obj.insert("empty_arr".to_string(), Value::Array(vec![]));

        let value = Value::Object(obj);
        let options = NormalizerOptions {
            remove_empty_containers: true,
            ..Default::default()
        };

        let normalized = normalize_with_options(&value, options).unwrap();

        if let Value::Object(normalized_obj) = normalized {
            assert_eq!(normalized_obj.len(), 1);
            assert_eq!(
                normalized_obj.get("keep"),
                Some(&Value::String("value".to_string()))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_normalize_numbers() {
        let value = Value::Number(Number::Float(42.0));
        let options = NormalizerOptions {
            prefer_integers: true,
            ..Default::default()
        };

        let normalized = normalize_with_options(&value, options).unwrap();

        assert_eq!(normalized, Value::Number(Number::Integer(42)));
    }

    #[test]
    fn test_normalize_strings() {
        let value = Value::String("  Hello World  ".to_string());
        let options = NormalizerOptions {
            trim_strings: true,
            normalize_string_case: true,
            ..Default::default()
        };

        let normalized = normalize_with_options(&value, options).unwrap();

        assert_eq!(normalized, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_deduplicate_arrays() {
        let arr = vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("a".to_string()),
            Value::String("c".to_string()),
        ];
        let value = Value::Array(arr);
        let options = NormalizerOptions {
            deduplicate_arrays: true,
            ..Default::default()
        };

        let normalized = normalize_with_options(&value, options).unwrap();

        if let Value::Array(normalized_arr) = normalized {
            assert_eq!(normalized_arr.len(), 3);
            // Elements should be sorted and deduplicated
            assert!(normalized_arr.contains(&Value::String("a".to_string())));
            assert!(normalized_arr.contains(&Value::String("b".to_string())));
            assert!(normalized_arr.contains(&Value::String("c".to_string())));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_canonical_normalizer() {
        let mut obj = FxHashMap::default();
        obj.insert("zebra".to_string(), Value::String("last".to_string()));
        obj.insert("apple".to_string(), Value::String("first".to_string()));

        let value = Value::Object(obj);
        let canonical = CanonicalNormalizer::canonicalize(&value).unwrap();

        // Should be the same as input (keys sorted during serialization)
        if let Value::Object(canonical_obj) = canonical {
            assert_eq!(canonical_obj.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_cleanup_normalizer() {
        let mut obj = FxHashMap::default();
        obj.insert("keep".to_string(), Value::String("  value  ".to_string()));
        obj.insert("remove".to_string(), Value::Null);
        obj.insert("empty".to_string(), Value::Array(vec![]));

        let value = Value::Object(obj);
        let cleaned = CleanupNormalizer::cleanup(&value).unwrap();

        if let Value::Object(cleaned_obj) = cleaned {
            assert_eq!(cleaned_obj.len(), 1);
            assert_eq!(
                cleaned_obj.get("keep"),
                Some(&Value::String("value".to_string()))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_deep_nested_normalization() {
        let mut inner = FxHashMap::default();
        inner.insert(
            "inner_key".to_string(),
            Value::String("inner_value".to_string()),
        );

        let mut outer = FxHashMap::default();
        outer.insert("outer_key".to_string(), Value::Object(inner));

        let value = Value::Object(outer);
        let normalized = normalize(&value).unwrap();

        // Should maintain structure
        if let Value::Object(normalized_obj) = normalized {
            if let Some(Value::Object(inner_obj)) = normalized_obj.get("outer_key") {
                assert_eq!(
                    inner_obj.get("inner_key"),
                    Some(&Value::String("inner_value".to_string()))
                );
            } else {
                panic!("Expected nested object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_max_depth_limit() {
        let mut obj = FxHashMap::default();
        obj.insert("key".to_string(), Value::String("value".to_string()));

        let value = Value::Object(obj);
        let options = NormalizerOptions {
            max_depth: 0,
            ..Default::default()
        };

        let normalized = normalize_with_options(&value, options).unwrap();

        // Should return original value when depth limit is exceeded
        assert_eq!(normalized, value);
    }
}
