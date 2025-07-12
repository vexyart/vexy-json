// this_file: src/lib.rs
// Main vexy_json library that re-exports core functionality

//! # vexy_json
//!
//! A forgiving JSON parser with support for relaxed JSON syntax.
//!
//! This crate provides a JSON parser that accepts relaxed JSON syntax including:
//! - Comments (single-line and multi-line)
//! - Trailing commas
//! - Unquoted object keys
//! - Single-quoted strings
//! - And more forgiving features
//!
//! ## Quick Start
//!
//! ```rust
//! use vexy_json::parse;
//!
//! let result = parse(r#"{"key": "value"}"#).unwrap();
//! ```

// Re-export core functionality
pub use vexy_json_core::{parse, parse_with_options, Error, Lexer, ParserOptions, Result};

// Re-export streaming functionality
pub use vexy_json_core::{
    NdJsonParser, SimpleStreamingLexer, StreamingEvent, StreamingParser, StreamingValueBuilder,
};

// Re-export AST types
pub use vexy_json_core::ast::{Number, Token, Value};

// Re-export error types
pub use vexy_json_core::error::{Error as ParseError, Result as ParseResult, Span};

// Re-export serde functionality if feature is enabled
#[cfg(feature = "serde")]
pub use vexy_json_serde::*;
