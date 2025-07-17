// this_file: src/ast/value.rs

//! Value types and implementations for the vexy_json AST.
//!
//! This module defines the core value types that represent parsed JSON data
//! in the vexy_json AST. It supports all standard JSON types plus the extensions
//! provided by vexy_json's forgiving syntax.

use rustc_hash::FxHashMap;
use std::fmt;
use std::ops::Index;

/// Represents any valid JSON value.
///
/// This enum can hold all JSON data types: null, boolean, number, string,
/// array, and object.
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Represents any valid JSON value.
pub enum Value {
    /// JSON null value.
    Null,
    /// JSON boolean value (true or false).
    Bool(bool),
    /// JSON numeric value (integer or floating point).
    Number(Number),
    /// JSON string value.
    String(String),
    /// JSON array containing a sequence of values.
    Array(Vec<Value>),
    /// JSON object containing key-value pairs.
    Object(FxHashMap<String, Value>),
}

/// Represents a JSON number, which can be either an integer or floating point.
///
/// This allows for more efficient representation and operations when the number
/// is a whole number that fits in an i64.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Number {
    /// An integer value that fits in an i64.
    Integer(i64),
    /// A floating point value.
    Float(f64),
}

impl Number {
    /// Converts the number to an f64 value.
    #[inline(always)]
    pub fn as_f64(&self) -> f64 {
        match self {
            Number::Integer(i) => *i as f64,
            Number::Float(f) => *f,
        }
    }
}

impl Value {
    /// Returns true if the value is null.
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if the value is a boolean.
    #[inline(always)]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Returns true if the value is a number.
    #[inline(always)]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns true if the value is a string.
    #[inline(always)]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns true if the value is an array.
    #[inline(always)]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Returns true if the value is an object.
    #[inline(always)]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// If the value is a boolean, returns the associated bool.
    /// Returns None otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If the value is a number, tries to return it as an i64.
    /// Returns None if the value is not a number or cannot be represented as i64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(Number::Integer(i)) => Some(*i),
            Value::Number(Number::Float(f)) => {
                if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                    Some(*f as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// If the value is a number, returns it as an f64.
    /// Returns None if the value is not a number.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(Number::Integer(i)) => Some(*i as f64),
            Value::Number(Number::Float(f)) => Some(*f),
            _ => None,
        }
    }

    /// If the value is a string, returns the associated str.
    /// Returns None otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the value is an array, returns a reference to the associated vector.
    /// Returns None otherwise.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// If the value is an object, returns a reference to the associated map.
    /// Returns None otherwise.
    pub fn as_object(&self) -> Option<&FxHashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{k}\": {v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(i) => write!(f, "{i}"),
            Number::Float(fl) => write!(f, "{fl}"),
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Value::Object(map) => map.get(index).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Value::Array(arr) => arr.get(index).unwrap_or(&Value::Null),
            _ => &Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_is_type_checkers() {
        let null_val = Value::Null;
        let bool_val = Value::Bool(true);
        let num_val = Value::Number(Number::Integer(42));
        let str_val = Value::String("hello".to_string());
        let arr_val = Value::Array(vec![Value::Null]);
        let obj_val = Value::Object(FxHashMap::default());

        // Test is_null
        assert!(null_val.is_null());
        assert!(!bool_val.is_null());
        assert!(!num_val.is_null());
        assert!(!str_val.is_null());
        assert!(!arr_val.is_null());
        assert!(!obj_val.is_null());

        // Test is_bool
        assert!(!null_val.is_bool());
        assert!(bool_val.is_bool());
        assert!(!num_val.is_bool());
        assert!(!str_val.is_bool());
        assert!(!arr_val.is_bool());
        assert!(!obj_val.is_bool());

        // Test is_number
        assert!(!null_val.is_number());
        assert!(!bool_val.is_number());
        assert!(num_val.is_number());
        assert!(!str_val.is_number());
        assert!(!arr_val.is_number());
        assert!(!obj_val.is_number());

        // Test is_string
        assert!(!null_val.is_string());
        assert!(!bool_val.is_string());
        assert!(!num_val.is_string());
        assert!(str_val.is_string());
        assert!(!arr_val.is_string());
        assert!(!obj_val.is_string());

        // Test is_array
        assert!(!null_val.is_array());
        assert!(!bool_val.is_array());
        assert!(!num_val.is_array());
        assert!(!str_val.is_array());
        assert!(arr_val.is_array());
        assert!(!obj_val.is_array());

        // Test is_object
        assert!(!null_val.is_object());
        assert!(!bool_val.is_object());
        assert!(!num_val.is_object());
        assert!(!str_val.is_object());
        assert!(!arr_val.is_object());
        assert!(obj_val.is_object());
    }

    #[test]
    fn test_value_as_bool() {
        let true_val = Value::Bool(true);
        let false_val = Value::Bool(false);
        let null_val = Value::Null;
        let num_val = Value::Number(Number::Integer(42));

        assert_eq!(true_val.as_bool(), Some(true));
        assert_eq!(false_val.as_bool(), Some(false));
        assert_eq!(null_val.as_bool(), None);
        assert_eq!(num_val.as_bool(), None);
    }

    #[test]
    fn test_value_as_i64() {
        let int_val = Value::Number(Number::Integer(42));
        let float_val = Value::Number(Number::Float(3.14));
        let float_int_val = Value::Number(Number::Float(5.0));
        let large_float_val = Value::Number(Number::Float(1e20));
        let null_val = Value::Null;

        assert_eq!(int_val.as_i64(), Some(42));
        assert_eq!(float_val.as_i64(), None); // Has fractional part
        assert_eq!(float_int_val.as_i64(), Some(5)); // No fractional part
        assert_eq!(large_float_val.as_i64(), None); // Too large for i64
        assert_eq!(null_val.as_i64(), None);
    }

    #[test]
    fn test_value_as_f64() {
        let int_val = Value::Number(Number::Integer(42));
        let float_val = Value::Number(Number::Float(3.14));
        let null_val = Value::Null;
        let str_val = Value::String("hello".to_string());

        assert_eq!(int_val.as_f64(), Some(42.0));
        assert_eq!(float_val.as_f64(), Some(3.14));
        assert_eq!(null_val.as_f64(), None);
        assert_eq!(str_val.as_f64(), None);
    }

    #[test]
    fn test_value_as_str() {
        let str_val = Value::String("hello".to_string());
        let null_val = Value::Null;
        let num_val = Value::Number(Number::Integer(42));

        assert_eq!(str_val.as_str(), Some("hello"));
        assert_eq!(null_val.as_str(), None);
        assert_eq!(num_val.as_str(), None);
    }

    #[test]
    fn test_value_as_array() {
        let arr_val = Value::Array(vec![Value::Null, Value::Bool(true)]);
        let null_val = Value::Null;
        let str_val = Value::String("hello".to_string());

        assert_eq!(arr_val.as_array(), Some(&vec![Value::Null, Value::Bool(true)]));
        assert_eq!(null_val.as_array(), None);
        assert_eq!(str_val.as_array(), None);
    }

    #[test]
    fn test_value_as_object() {
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), Value::String("value".to_string()));
        let obj_val = Value::Object(map.clone());
        let null_val = Value::Null;
        let arr_val = Value::Array(vec![]);

        assert_eq!(obj_val.as_object(), Some(&map));
        assert_eq!(null_val.as_object(), None);
        assert_eq!(arr_val.as_object(), None);
    }

    #[test]
    fn test_value_index_str() {
        let mut map = FxHashMap::default();
        map.insert("name".to_string(), Value::String("John".to_string()));
        map.insert("age".to_string(), Value::Number(Number::Integer(30)));
        let obj_val = Value::Object(map);

        assert_eq!(obj_val["name"], Value::String("John".to_string()));
        assert_eq!(obj_val["age"], Value::Number(Number::Integer(30)));
        assert_eq!(obj_val["missing"], Value::Null);

        // Test indexing non-object returns null
        let arr_val = Value::Array(vec![]);
        assert_eq!(arr_val["any"], Value::Null);
    }

    #[test]
    fn test_value_index_usize() {
        let arr_val = Value::Array(vec![
            Value::String("first".to_string()),
            Value::Number(Number::Integer(42)),
            Value::Bool(true),
        ]);

        assert_eq!(arr_val[0], Value::String("first".to_string()));
        assert_eq!(arr_val[1], Value::Number(Number::Integer(42)));
        assert_eq!(arr_val[2], Value::Bool(true));
        assert_eq!(arr_val[99], Value::Null); // Out of bounds

        // Test indexing non-array returns null
        let obj_val = Value::Object(FxHashMap::default());
        assert_eq!(obj_val[0], Value::Null);
    }

    #[test]
    fn test_number_as_f64() {
        let int_num = Number::Integer(42);
        let float_num = Number::Float(3.14);

        assert_eq!(int_num.as_f64(), 42.0);
        assert_eq!(float_num.as_f64(), 3.14);
    }

    #[test]
    fn test_value_display() {
        let null_val = Value::Null;
        let bool_val = Value::Bool(true);
        let int_val = Value::Number(Number::Integer(42));
        let float_val = Value::Number(Number::Float(3.14));
        let str_val = Value::String("hello".to_string());

        assert_eq!(null_val.to_string(), "null");
        assert_eq!(bool_val.to_string(), "true");
        assert_eq!(int_val.to_string(), "42");
        assert_eq!(float_val.to_string(), "3.14");
        assert_eq!(str_val.to_string(), "\"hello\"");
    }

    #[test]
    fn test_value_display_array() {
        let arr_val = Value::Array(vec![
            Value::Number(Number::Integer(1)),
            Value::Number(Number::Integer(2)),
            Value::Number(Number::Integer(3)),
        ]);

        assert_eq!(arr_val.to_string(), "[1, 2, 3]");

        let empty_arr = Value::Array(vec![]);
        assert_eq!(empty_arr.to_string(), "[]");
    }

    #[test]
    fn test_value_display_object() {
        let mut map = FxHashMap::default();
        map.insert("name".to_string(), Value::String("John".to_string()));
        let obj_val = Value::Object(map);

        assert_eq!(obj_val.to_string(), "{\"name\": \"John\"}");

        let empty_obj = Value::Object(FxHashMap::default());
        assert_eq!(empty_obj.to_string(), "{}");
    }

    #[test]
    fn test_number_display() {
        let int_num = Number::Integer(42);
        let float_num = Number::Float(3.14);

        assert_eq!(int_num.to_string(), "42");
        assert_eq!(float_num.to_string(), "3.14");
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::Number(Number::Integer(42));
        let val2 = Value::Number(Number::Integer(42));
        let val3 = Value::Number(Number::Integer(43));

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_number_equality() {
        let int1 = Number::Integer(42);
        let int2 = Number::Integer(42);
        let int3 = Number::Integer(43);
        let float1 = Number::Float(42.0);

        assert_eq!(int1, int2);
        assert_ne!(int1, int3);
        assert_ne!(int1, float1); // Different variants are not equal
    }

    #[test]
    fn test_as_i64_edge_cases() {
        // Test max and min i64 values
        let max_int = Value::Number(Number::Integer(i64::MAX));
        let min_int = Value::Number(Number::Integer(i64::MIN));
        
        assert_eq!(max_int.as_i64(), Some(i64::MAX));
        assert_eq!(min_int.as_i64(), Some(i64::MIN));

        // Test float edge cases
        let max_float = Value::Number(Number::Float(i64::MAX as f64));
        let min_float = Value::Number(Number::Float(i64::MIN as f64));
        let too_large = Value::Number(Number::Float(1e20));
        let negative_zero = Value::Number(Number::Float(-0.0));

        assert_eq!(max_float.as_i64(), Some(i64::MAX));
        assert_eq!(min_float.as_i64(), Some(i64::MIN));
        assert_eq!(too_large.as_i64(), None);
        assert_eq!(negative_zero.as_i64(), Some(0));
    }
}
