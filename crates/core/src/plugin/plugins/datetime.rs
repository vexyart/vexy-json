//! Date/time parsing plugin

use crate::ast::Value;
use crate::error::Result;
use crate::plugin::ParserPlugin;
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use std::any::Any;

/// Date/time parsing plugin that converts ISO 8601 strings to structured date objects
pub struct DateTimePlugin {
    /// Whether to parse dates in strings
    parse_dates: bool,
    /// Custom date formats to try
    custom_formats: Vec<String>,
    /// Whether to preserve original string
    preserve_original: bool,
}

impl DateTimePlugin {
    /// Create a new datetime plugin with default settings
    pub fn new() -> Self {
        DateTimePlugin {
            parse_dates: true,
            custom_formats: vec![
                "%Y-%m-%d".to_string(),
                "%Y-%m-%d %H:%M:%S".to_string(),
                "%Y-%m-%dT%H:%M:%S%.fZ".to_string(),
                "%Y-%m-%dT%H:%M:%S%.f%:z".to_string(),
            ],
            preserve_original: true,
        }
    }

    /// Add a custom date format
    pub fn add_format(&mut self, format: &str) {
        self.custom_formats.push(format.to_string());
    }

    /// Try to parse a string as a date
    fn try_parse_date(&self, s: &str) -> Option<DateTime<Utc>> {
        // Try ISO 8601 first
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }

        // Try custom formats
        for format in &self.custom_formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
                return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
            }
        }

        None
    }

    /// Convert a datetime to a structured value
    fn datetime_to_value(&self, dt: DateTime<Utc>, original: &str) -> Value {
        let mut obj = rustc_hash::FxHashMap::default();

        obj.insert(
            "year".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.year() as i64)),
        );
        obj.insert(
            "month".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.month() as i64)),
        );
        obj.insert(
            "day".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.day() as i64)),
        );
        obj.insert(
            "hour".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.hour() as i64)),
        );
        obj.insert(
            "minute".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.minute() as i64)),
        );
        obj.insert(
            "second".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.second() as i64)),
        );
        obj.insert(
            "timestamp".to_string(),
            Value::Number(crate::ast::Number::Integer(dt.timestamp())),
        );
        obj.insert("iso8601".to_string(), Value::String(dt.to_rfc3339()));

        if self.preserve_original {
            obj.insert("_original".to_string(), Value::String(original.to_string()));
        }

        Value::Object(obj)
    }
}

impl Default for DateTimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserPlugin for DateTimePlugin {
    fn name(&self) -> &str {
        "datetime"
    }

    fn transform_value(&mut self, value: &mut Value, _path: &str) -> Result<()> {
        if !self.parse_dates {
            return Ok(());
        }

        match value {
            Value::String(s) => {
                if let Some(dt) = self.try_parse_date(s) {
                    *value = self.datetime_to_value(dt, s);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.transform_value(item, _path)?;
                }
            }
            Value::Object(obj) => {
                for (_, v) in obj.iter_mut() {
                    self.transform_value(v, _path)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::parse;

    #[test]
    fn test_datetime_parsing() {
        let mut plugin = DateTimePlugin::new();

        // Test ISO 8601 parsing
        let mut value = Value::String("2023-12-25T10:30:00Z".to_string());
        plugin.transform_value(&mut value, "$").unwrap();

        if let Value::Object(obj) = value {
            assert_eq!(
                obj.get("year"),
                Some(&Value::Number(crate::ast::Number::Integer(2023)))
            );
            assert_eq!(
                obj.get("month"),
                Some(&Value::Number(crate::ast::Number::Integer(12)))
            );
            assert_eq!(
                obj.get("day"),
                Some(&Value::Number(crate::ast::Number::Integer(25)))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_custom_format() {
        let mut plugin = DateTimePlugin::new();

        // Test custom format
        let mut value = Value::String("2023-12-25".to_string());
        plugin.transform_value(&mut value, "$").unwrap();

        if let Value::Object(obj) = value {
            assert_eq!(
                obj.get("year"),
                Some(&Value::Number(crate::ast::Number::Integer(2023)))
            );
            assert!(obj.contains_key("_original"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_non_date_string() {
        let mut plugin = DateTimePlugin::new();

        // Non-date string should remain unchanged
        let mut value = Value::String("hello world".to_string());
        plugin.transform_value(&mut value, "$").unwrap();

        assert_eq!(value, Value::String("hello world".to_string()));
    }
}
