// this_file: src/error/result.rs

use super::types::Error;

/// Convenience type alias for Results using vexy_json's Error type.
///
/// This makes function signatures more concise throughout the codebase
/// while maintaining type safety. Most parsing functions return this type.
pub type Result<T> = std::result::Result<T, Error>;

/// Convenience type alias for parsing operations.
pub type ParseResult<T = crate::ast::Value> = std::result::Result<T, Error>;
