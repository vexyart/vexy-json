// this_file: src/parser/string.rs

use crate::ast::Value;
use crate::error::{Error, Result, Span};
use crate::optimization::{extract_string_content, unescape_string_optimized};
use crate::parser::ParserOptions;

/// Optimized helper function for parsing string tokens into Values
#[inline]
pub(super) fn parse_string_token(
    original_input: &str,
    span: Span,
    options: &ParserOptions,
) -> Result<Value> {
    let string_slice = &original_input[span.start..span.end];

    // Validate quote characters
    if string_slice.len() < 2 {
        return Err(Error::UnterminatedString(span.start));
    }

    let first_char = string_slice.chars().next().unwrap();
    if first_char == '\'' && !options.allow_single_quotes {
        return Err(Error::UnexpectedChar('\'', span.start));
    }

    // Fast string content extraction
    let content =
        extract_string_content(string_slice).map_err(|_| Error::UnterminatedString(span.start))?;

    // Use optimized unescaping
    let unescaped = unescape_string_optimized(content).map_err(|e| match e {
        Error::InvalidEscape(_) => Error::InvalidEscape(span.start),
        Error::InvalidUnicode(_) => Error::InvalidUnicode(span.start),
        other => other,
    })?;

    Ok(Value::String(unescaped))
}
