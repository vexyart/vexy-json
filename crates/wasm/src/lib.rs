use serde_json::Value;
use vexy_json_core::{parse, parse_with_options, ParserOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Parse a JSON/Vexy JSON string and return the result as a JSON string
#[wasm_bindgen]
pub fn parse_json(input: &str) -> Result<String, JsValue> {
    match parse(input) {
        Ok(value) => {
            // Convert to serde_json::Value for proper JSON serialization
            let json_value: Value = serde_json::from_str(&value.to_string())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&json_value).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {e}"))),
    }
}

/// Parse a JSON/Vexy JSON string with custom options
#[wasm_bindgen]
pub fn parse_json_with_options(
    input: &str,
    allow_comments: bool,
    allow_trailing_commas: bool,
    allow_unquoted_keys: bool,
    allow_single_quotes: bool,
    implicit_top_level: bool,
    newline_as_comma: bool,
    enable_repair: bool,
    max_depth: Option<u32>,
) -> Result<String, JsValue> {
    let options = ParserOptions {
        allow_comments,
        allow_trailing_commas,
        allow_unquoted_keys,
        allow_single_quotes,
        implicit_top_level,
        newline_as_comma,
        max_depth: max_depth.unwrap_or(128) as usize,
        enable_repair,
        fast_repair: false,
        max_repairs: 100,
        report_repairs: false,
    };

    match parse_with_options(input, options) {
        Ok(value) => {
            // Convert to serde_json::Value for proper JSON serialization
            let json_value: Value = serde_json::from_str(&value.to_string())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&json_value).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {e}"))),
    }
}

/// Validate if a string is valid JSON/Vexy JSON
#[wasm_bindgen]
pub fn validate_json(input: &str) -> bool {
    parse(input).is_ok()
}

/// Get parser options as a JSON object
#[wasm_bindgen]
pub fn get_parser_options() -> Result<String, JsValue> {
    let options = serde_json::json!({
        "allow_comments": {
            "default": true,
            "description": "Allow // and /* */ style comments"
        },
        "allow_trailing_commas": {
            "default": true,
            "description": "Allow trailing commas in arrays and objects"
        },
        "allow_unquoted_keys": {
            "default": true,
            "description": "Allow unquoted object keys"
        },
        "allow_single_quotes": {
            "default": true,
            "description": "Allow single-quoted strings"
        },
        "implicit_top_level": {
            "default": true,
            "description": "Allow implicit top-level objects"
        },
        "newline_as_comma": {
            "default": true,
            "description": "Treat newlines as commas"
        },
        "enable_repair": {
            "default": true,
            "description": "Enable automatic error repair"
        },
        "max_depth": {
            "default": 128,
            "description": "Maximum nesting depth"
        }
    });

    serde_json::to_string(&options).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Stringify a JSON value with pretty printing
#[wasm_bindgen]
pub fn stringify_value(input: &str, indent: Option<u32>) -> Result<String, JsValue> {
    // First parse with vexy_json to handle forgiving syntax
    let value = parse(input).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Convert to serde_json::Value
    let json_value: Value =
        serde_json::from_str(&value.to_string()).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Stringify with or without pretty printing
    if let Some(_indent_size) = indent {
        // TODO: Implement custom indentation size
        serde_json::to_string_pretty(&json_value).map_err(|e| JsValue::from_str(&e.to_string()))
    } else {
        serde_json::to_string(&json_value).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Get version information
#[wasm_bindgen]
pub fn get_version_info() -> Result<String, JsValue> {
    let version_info = serde_json::json!({
        "version": env!("VEXY_JSON_VERSION", env!("CARGO_PKG_VERSION")),
        "name": env!("CARGO_PKG_NAME"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
    });

    serde_json::to_string(&version_info).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Legacy function names for backward compatibility
#[wasm_bindgen]
pub fn parse_js(input: &str) -> Result<String, JsValue> {
    parse_json(input)
}

#[wasm_bindgen]
pub fn parse_with_options_js(
    input: &str,
    allow_comments: bool,
    allow_trailing_commas: bool,
    allow_unquoted_keys: bool,
    allow_single_quotes: bool,
    implicit_top_level: bool,
    newline_as_comma: bool,
) -> Result<String, JsValue> {
    parse_json_with_options(
        input,
        allow_comments,
        allow_trailing_commas,
        allow_unquoted_keys,
        allow_single_quotes,
        implicit_top_level,
        newline_as_comma,
        true, // enable_repair
        None, // max_depth
    )
}

#[wasm_bindgen]
pub fn is_valid(input: &str) -> bool {
    validate_json(input)
}

#[wasm_bindgen]
pub fn format(input: &str) -> Result<String, JsValue> {
    stringify_value(input, Some(2))
}
