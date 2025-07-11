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
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(i) => write!(f, "{}", i),
            Number::Float(fl) => write!(f, "{}", fl),
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
