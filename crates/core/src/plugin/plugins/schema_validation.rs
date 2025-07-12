//! JSON Schema validation plugin

use crate::ast::Value;
use crate::error::{Error, Result};
use crate::plugin::ParserPlugin;
use std::any::Any;
// use rustc_hash::FxHashMap;

/// JSON Schema validation plugin
pub struct SchemaValidationPlugin {
    /// The JSON schema to validate against
    schema: Value,
    /// Validation errors collected
    errors: Vec<String>,
}

impl SchemaValidationPlugin {
    /// Create a new schema validation plugin
    pub fn new(schema: Value) -> Self {
        SchemaValidationPlugin {
            schema,
            errors: Vec::new(),
        }
    }

    /// Get validation errors
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Validate a value against a schema
    fn validate_against_schema(&self, value: &Value, schema: &Value, path: &str) -> Result<()> {
        if let Value::Object(schema_obj) = schema {
            // Check type
            if let Some(Value::String(expected_type)) = schema_obj.get("type") {
                let actual_type = match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                };

                if expected_type != actual_type {
                    return Err(Error::Custom(format!(
                        "Type mismatch at {path}: expected {expected_type}, got {actual_type}"
                    )));
                }
            }

            // Validate object properties
            if let (Value::Object(obj), Some(Value::Object(properties))) =
                (value, schema_obj.get("properties"))
            {
                for (key, prop_schema) in properties {
                    let prop_path = format!("{path}.{key}");
                    if let Some(prop_value) = obj.get(key) {
                        self.validate_against_schema(prop_value, prop_schema, &prop_path)?;
                    } else if let Some(Value::Array(required)) = schema_obj.get("required") {
                        if required
                            .iter()
                            .any(|v| matches!(v, Value::String(s) if s == key))
                        {
                            return Err(Error::Custom(format!(
                                "Missing required property: {prop_path}"
                            )));
                        }
                    }
                }
            }

            // Validate array items
            if let (Value::Array(arr), Some(items_schema)) = (value, schema_obj.get("items")) {
                for (i, item) in arr.iter().enumerate() {
                    let item_path = format!("{path}[{i}]");
                    self.validate_against_schema(item, items_schema, &item_path)?;
                }
            }

            // Validate number constraints
            if let Value::Number(n) = value {
                let n_val = n.as_f64();

                if let Some(Value::Number(min)) = schema_obj.get("minimum") {
                    if n_val < min.as_f64() {
                        return Err(Error::Custom(format!(
                            "Value at {} is below minimum: {} < {}",
                            path,
                            n_val,
                            min.as_f64()
                        )));
                    }
                }

                if let Some(Value::Number(max)) = schema_obj.get("maximum") {
                    if n_val > max.as_f64() {
                        return Err(Error::Custom(format!(
                            "Value at {} exceeds maximum: {} > {}",
                            path,
                            n_val,
                            max.as_f64()
                        )));
                    }
                }
            }

            // Validate string constraints
            if let Value::String(s) = value {
                if let Some(Value::Number(min_len)) = schema_obj.get("minLength") {
                    if s.len() < min_len.as_f64() as usize {
                        return Err(Error::Custom(format!(
                            "String at {} is too short: {} < {}",
                            path,
                            s.len(),
                            min_len.as_f64()
                        )));
                    }
                }

                if let Some(Value::Number(max_len)) = schema_obj.get("maxLength") {
                    if s.len() > max_len.as_f64() as usize {
                        return Err(Error::Custom(format!(
                            "String at {} is too long: {} > {}",
                            path,
                            s.len(),
                            max_len.as_f64()
                        )));
                    }
                }

                if let Some(Value::String(pattern)) = schema_obj.get("pattern") {
                    let re = regex::Regex::new(pattern)
                        .map_err(|e| Error::Custom(format!("Invalid regex pattern: {e}")))?;
                    if !re.is_match(s) {
                        return Err(Error::Custom(format!(
                            "String at {path} does not match pattern: {pattern}"
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

impl ParserPlugin for SchemaValidationPlugin {
    fn name(&self) -> &str {
        "schema_validation"
    }

    fn validate(&self, value: &Value, path: &str) -> Result<()> {
        self.validate_against_schema(value, &self.schema, path)
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
    use crate::parse;

    #[test]
    fn test_schema_validation() {
        let schema = parse(
            r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number", "minimum": 0 }
            },
            "required": ["name"]
        }"#,
        )
        .unwrap();

        let plugin = SchemaValidationPlugin::new(schema);

        // Valid object
        let valid = parse(r#"{"name": "John", "age": 30}"#).unwrap();
        assert!(plugin.validate(&valid, "$").is_ok());

        // Invalid - missing required field
        let invalid = parse(r#"{"age": 30}"#).unwrap();
        assert!(plugin.validate(&invalid, "$").is_err());

        // Invalid - wrong type
        let invalid = parse(r#"{"name": 123, "age": 30}"#).unwrap();
        assert!(plugin.validate(&invalid, "$").is_err());

        // Invalid - negative age
        let invalid = parse(r#"{"name": "John", "age": -5}"#).unwrap();
        assert!(plugin.validate(&invalid, "$").is_err());
    }
}
