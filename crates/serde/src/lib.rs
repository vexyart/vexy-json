use serde::{Deserialize, Serialize};
use vexy_json_core::ast::Value;

// This is a placeholder for Serde integration.
// Actual implementation would involve implementing Serialize/Deserialize for vexy_json_core::Value
// or providing helper functions for conversion.

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SerdeValue(Value);

impl From<Value> for SerdeValue {
    fn from(value: Value) -> Self {
        SerdeValue(value)
    }
}

impl From<SerdeValue> for Value {
    fn from(serde_value: SerdeValue) -> Self {
        serde_value.0
    }
}
