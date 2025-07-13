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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences a raw pointer (`input`)
/// - Assumes `input` points to a valid null-terminated C string
/// - Returns raw pointers that must be freed using `vexy_json_free_result`
///
/// The caller must ensure:
/// - `input` is either null or points to a valid null-terminated UTF-8 string
/// - The returned `VexyJsonParseResult` is eventually freed using `vexy_json_free_result`
/// - The returned pointers in the result are not used after being freed
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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers (`input` and `options`)
/// - Assumes `input` points to a valid null-terminated C string
/// - Assumes `options` (if non-null) points to a valid `VexyJsonParserOptions` struct
/// - Returns raw pointers that must be freed using `vexy_json_free_result`
///
/// The caller must ensure:
/// - `input` is either null or points to a valid null-terminated UTF-8 string
/// - `options` is either null or points to a valid `VexyJsonParserOptions` struct
/// - The returned `VexyJsonParseResult` is eventually freed using `vexy_json_free_result`
/// - The returned pointers in the result are not used after being freed
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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers (`input` and `options`)
/// - Assumes `input` points to a valid null-terminated C string
/// - Assumes `options` (if non-null) points to a valid `VexyJsonParserOptions` struct
/// - Returns raw pointers that must be freed using `vexy_json_free_detailed_result`
///
/// The caller must ensure:
/// - `input` is either null or points to a valid null-terminated UTF-8 string
/// - `options` is either null or points to a valid `VexyJsonParserOptions` struct
/// - The returned `VexyJsonDetailedResult` is eventually freed using `vexy_json_free_detailed_result`
/// - The returned pointers in the result are not used after being freed
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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences a raw pointer (`options`) if non-null
/// - Assumes `options` (if non-null) points to a valid `VexyJsonParserOptions` struct
/// - Returns a raw pointer that must be freed using `vexy_json_parser_free`
///
/// The caller must ensure:
/// - `options` is either null or points to a valid `VexyJsonParserOptions` struct
/// - The returned parser pointer is eventually freed using `vexy_json_parser_free`
/// - The returned parser pointer is not used after being freed
/// - The returned parser pointer is not shared across threads without proper synchronization
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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers (`parser` and `input`)
/// - Assumes `parser` points to a valid `VexyJsonParser` instance created by `vexy_json_parser_new`
/// - Assumes `input` points to a valid null-terminated C string
/// - Returns raw pointers that must be freed using `vexy_json_free_result`
///
/// The caller must ensure:
/// - `parser` is either null or points to a valid `VexyJsonParser` created by `vexy_json_parser_new`
/// - `input` is either null or points to a valid null-terminated UTF-8 string
/// - The parser has not been freed or moved
/// - The returned `VexyJsonParseResult` is eventually freed using `vexy_json_free_result`
/// - The returned pointers in the result are not used after being freed
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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Takes ownership of the raw pointer `parser`
/// - Deallocates the memory pointed to by `parser`
///
/// The caller must ensure:
/// - `parser` is either null or points to a valid `VexyJsonParser` created by `vexy_json_parser_new`
/// - `parser` has not already been freed
/// - `parser` is not used after this call
/// - No other references to the parser exist
#[no_mangle]
pub unsafe extern "C" fn vexy_json_parser_free(parser: *mut VexyJsonParser) {
    if !parser.is_null() {
        let _ = Box::from_raw(parser);
    }
}

/// Free a parse result
///
/// # Safety
///
/// This function is unsafe because it:
/// - Takes ownership of the raw pointers in `result`
/// - Deallocates the memory pointed to by `result.json` and `result.error`
///
/// The caller must ensure:
/// - The pointers in `result` are either null or were returned by one of the parse functions
/// - The pointers have not already been freed
/// - The pointers are not used after this call
/// - The result struct was obtained from `vexy_json_parse`, `vexy_json_parse_with_options`, or `vexy_json_parser_parse`
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
///
/// # Safety
///
/// This function is unsafe because it:
/// - Takes ownership of the raw pointers in `result`
/// - Deallocates the memory pointed to by `result.json`, `result.error`, and `result.repairs`
///
/// The caller must ensure:
/// - The pointers in `result` are either null or were returned by `vexy_json_parse_detailed`
/// - The pointers have not already been freed
/// - The pointers are not used after this call
/// - If `result.repairs` is non-null, it points to an array of `result.repair_count` elements
/// - Each repair in the array has valid pointers that need to be freed
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
