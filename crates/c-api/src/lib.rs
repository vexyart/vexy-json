//! C API for the vexy_json JSON parser.
//!
//! This crate provides a C-compatible API that can be used from C/C++
//! applications and for creating language bindings.

use libc::{c_char, size_t};
use std::ffi::{CStr, CString};
use std::ptr;
use vexy_json_core::ast::Value;
use vexy_json_core::{parse, parse_with_options, ParserOptions};

/// Parser options for configuring vexy_json behavior
#[repr(C)]
pub struct VexyJsonParserOptions {
    pub allow_comments: bool,
    pub allow_trailing_commas: bool,
    pub allow_unquoted_keys: bool,
    pub allow_single_quotes: bool,
    pub implicit_top_level: bool,
    pub newline_as_comma: bool,
    pub max_depth: u32,
    pub enable_repair: bool,
    pub max_repairs: u32,
    pub fast_repair: bool,
    pub report_repairs: bool,
}

/// Result of parsing JSON
#[repr(C)]
pub struct VexyJsonParseResult {
    pub json: *mut c_char,
    pub error: *mut c_char,
}

/// A single repair action
#[repr(C)]
pub struct VexyJsonRepair {
    pub repair_type: *mut c_char,
    pub position: size_t,
    pub description: *mut c_char,
}

/// Detailed result including repairs
#[repr(C)]
pub struct VexyJsonDetailedResult {
    pub json: *mut c_char,
    pub error: *mut c_char,
    pub repairs: *mut VexyJsonRepair,
    pub repair_count: size_t,
}

/// Opaque parser handle
pub struct VexyJsonParser {
    options: ParserOptions,
}

/// Get the version of the vexy_json library
#[no_mangle]
pub extern "C" fn vexy_json_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// Parse JSON with default options
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parse(input: *const c_char) -> VexyJsonParseResult {
    if input.is_null() {
        return VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new("Input is null").unwrap().into_raw(),
        };
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            return VexyJsonParseResult {
                json: ptr::null_mut(),
                error: CString::new("Invalid UTF-8 input").unwrap().into_raw(),
            };
        }
    };

    match parse(input_str) {
        Ok(value) => match value_to_json_string(&value) {
            Ok(json_str) => VexyJsonParseResult {
                json: CString::new(json_str).unwrap().into_raw(),
                error: ptr::null_mut(),
            },
            Err(e) => VexyJsonParseResult {
                json: ptr::null_mut(),
                error: CString::new(format!("Serialization error: {e}"))
                    .unwrap()
                    .into_raw(),
            },
        },
        Err(e) => VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new(format!("{e}")).unwrap().into_raw(),
        },
    }
}

/// Parse JSON with custom options
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parse_with_options(
    input: *const c_char,
    options: *const VexyJsonParserOptions,
) -> VexyJsonParseResult {
    if input.is_null() {
        return VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new("Input is null").unwrap().into_raw(),
        };
    }

    if options.is_null() {
        return vexy_json_parse(input);
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            return VexyJsonParseResult {
                json: ptr::null_mut(),
                error: CString::new("Invalid UTF-8 input").unwrap().into_raw(),
            };
        }
    };

    let rust_options = c_options_to_rust(&*options);

    match parse_with_options(input_str, rust_options) {
        Ok(value) => match value_to_json_string(&value) {
            Ok(json_str) => VexyJsonParseResult {
                json: CString::new(json_str).unwrap().into_raw(),
                error: ptr::null_mut(),
            },
            Err(e) => VexyJsonParseResult {
                json: ptr::null_mut(),
                error: CString::new(format!("Serialization error: {e}"))
                    .unwrap()
                    .into_raw(),
            },
        },
        Err(e) => VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new(format!("{e}")).unwrap().into_raw(),
        },
    }
}

/// Parse JSON and get detailed information including repairs
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parse_detailed(
    input: *const c_char,
    options: *const VexyJsonParserOptions,
) -> VexyJsonDetailedResult {
    // For now, we'll implement this as a simple parse without repair tracking
    // TODO: Implement actual repair tracking
    let result = if options.is_null() {
        vexy_json_parse(input)
    } else {
        vexy_json_parse_with_options(input, options)
    };

    VexyJsonDetailedResult {
        json: result.json,
        error: result.error,
        repairs: ptr::null_mut(),
        repair_count: 0,
    }
}

/// Create a new parser instance
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parser_new(
    options: *const VexyJsonParserOptions,
) -> *mut VexyJsonParser {
    let rust_options = if options.is_null() {
        ParserOptions::default()
    } else {
        c_options_to_rust(&*options)
    };

    let parser = Box::new(VexyJsonParser {
        options: rust_options,
    });

    Box::into_raw(parser)
}

/// Parse JSON using a parser instance
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parser_parse(
    parser: *mut VexyJsonParser,
    input: *const c_char,
) -> VexyJsonParseResult {
    if parser.is_null() {
        return VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new("Parser is null").unwrap().into_raw(),
        };
    }

    if input.is_null() {
        return VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new("Input is null").unwrap().into_raw(),
        };
    }

    let parser_ref = &*parser;
    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            return VexyJsonParseResult {
                json: ptr::null_mut(),
                error: CString::new("Invalid UTF-8 input").unwrap().into_raw(),
            };
        }
    };

    match parse_with_options(input_str, parser_ref.options.clone()) {
        Ok(value) => match value_to_json_string(&value) {
            Ok(json_str) => VexyJsonParseResult {
                json: CString::new(json_str).unwrap().into_raw(),
                error: ptr::null_mut(),
            },
            Err(e) => VexyJsonParseResult {
                json: ptr::null_mut(),
                error: CString::new(format!("Serialization error: {e}"))
                    .unwrap()
                    .into_raw(),
            },
        },
        Err(e) => VexyJsonParseResult {
            json: ptr::null_mut(),
            error: CString::new(format!("{e}")).unwrap().into_raw(),
        },
    }
}

/// Free a parser instance
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parser_free(parser: *mut VexyJsonParser) {
    if !parser.is_null() {
        let _ = Box::from_raw(parser);
    }
}

/// Free a parse result
#[no_mangle]
pub unsafe extern "C" fn vexy_json_free_result(result: VexyJsonParseResult) {
    if !result.json.is_null() {
        let _ = CString::from_raw(result.json);
    }
    if !result.error.is_null() {
        let _ = CString::from_raw(result.error);
    }
}

/// Free a detailed result
#[no_mangle]
pub unsafe extern "C" fn vexy_json_free_detailed_result(result: VexyJsonDetailedResult) {
    if !result.json.is_null() {
        let _ = CString::from_raw(result.json);
    }
    if !result.error.is_null() {
        let _ = CString::from_raw(result.error);
    }
    // TODO: Free repairs array when implemented
}

/// Get default parser options
#[no_mangle]
pub extern "C" fn vexy_json_default_options() -> VexyJsonParserOptions {
    let rust_options = ParserOptions::default();
    rust_options_to_c(&rust_options)
}

/// Convert C options to Rust options
fn c_options_to_rust(options: &VexyJsonParserOptions) -> ParserOptions {
    ParserOptions {
        allow_comments: options.allow_comments,
        allow_trailing_commas: options.allow_trailing_commas,
        allow_unquoted_keys: options.allow_unquoted_keys,
        allow_single_quotes: options.allow_single_quotes,
        implicit_top_level: options.implicit_top_level,
        newline_as_comma: options.newline_as_comma,
        max_depth: options.max_depth as usize,
        enable_repair: options.enable_repair,
        max_repairs: options.max_repairs as usize,
        fast_repair: options.fast_repair,
        report_repairs: options.report_repairs,
    }
}

/// Convert Rust options to C options
fn rust_options_to_c(options: &ParserOptions) -> VexyJsonParserOptions {
    VexyJsonParserOptions {
        allow_comments: options.allow_comments,
        allow_trailing_commas: options.allow_trailing_commas,
        allow_unquoted_keys: options.allow_unquoted_keys,
        allow_single_quotes: options.allow_single_quotes,
        implicit_top_level: options.implicit_top_level,
        newline_as_comma: options.newline_as_comma,
        max_depth: options.max_depth as u32,
        enable_repair: options.enable_repair,
        max_repairs: options.max_repairs as u32,
        fast_repair: options.fast_repair,
        report_repairs: options.report_repairs,
    }
}

/// Convert a Value to a JSON string
fn value_to_json_string(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}
