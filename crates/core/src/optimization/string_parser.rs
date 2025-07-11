// this_file: crates/core/src/optimization/string_parser.rs

//! Optimized string parsing utilities for better performance.
//!
//! This module provides optimized string parsing functions that avoid
//! unnecessary allocations and use more efficient algorithms.

use crate::error::{Error, Result};

/// Fast string unescaping that avoids allocations when no escapes are present.
#[inline]
pub fn unescape_string_optimized(s: &str) -> Result<String> {
    // Fast path: if no backslashes, return as-is (avoid allocation)
    if !s.contains('\\') {
        return Ok(s.to_string());
    }

    // Slow path: process escape sequences
    let mut result = String::with_capacity(s.len()); // Pre-allocate expected size
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('/') => result.push('/'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0C'),
                Some('u') => {
                    // Unicode escape sequence \uXXXX - optimized version
                    result.push(parse_unicode_escape(&mut chars)?);
                }
                Some('x') => {
                    // ASCII hex escape sequence \xXX - optimized version
                    result.push(parse_hex_escape(&mut chars)?);
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[inline]
fn parse_unicode_escape(chars: &mut std::str::Chars<'_>) -> Result<char> {
    let mut code = 0u32;

    for _ in 0..4 {
        let hex_char = chars.next().ok_or(Error::InvalidEscape(0))?;
        let digit = hex_char.to_digit(16).ok_or(Error::InvalidEscape(0))?;
        code = (code << 4) | digit;
    }

    // Check if there's an additional hex digit immediately following
    // In strict JSON, \uXXXX should be exactly 4 hex digits
    if let Some(&next_char) = chars.as_str().as_bytes().get(0) {
        if (next_char as char).is_ascii_hexdigit() {
            return Err(Error::InvalidEscape(0));
        }
    }

    std::char::from_u32(code).ok_or(Error::InvalidUnicode(0))
}

#[inline]
fn parse_hex_escape(chars: &mut std::str::Chars<'_>) -> Result<char> {
    let mut code = 0u8;

    for _ in 0..2 {
        let hex_char = chars.next().ok_or(Error::InvalidEscape(0))?;
        let digit = hex_char.to_digit(16).ok_or(Error::InvalidEscape(0))? as u8;
        code = (code << 4) | digit;
    }

    Ok(code as char)
}

/// Optimized string content extraction that avoids unnecessary string operations.
#[inline]
pub fn extract_string_content(slice: &str) -> Result<&str> {
    if slice.len() < 2 {
        return Err(Error::UnterminatedString(0));
    }

    // Check quote characters and return slice without quotes
    let first = slice.as_bytes()[0];
    let last = slice.as_bytes()[slice.len() - 1];

    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
        Ok(&slice[1..slice.len() - 1])
    } else {
        Err(Error::UnterminatedString(0))
    }
}

/// Fast number parsing optimized for common cases.
#[inline]
pub fn parse_number_optimized(s: &str) -> Result<f64> {
    // Fast path for integers without scientific notation
    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
        if let Ok(int_val) = s.parse::<i64>() {
            return Ok(int_val as f64);
        }
    }

    // Fallback to standard parsing
    s.parse::<f64>().map_err(|_| Error::InvalidNumber(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_no_escapes() {
        let result = unescape_string_optimized("hello world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_unescape_with_escapes() {
        let result = unescape_string_optimized("hello\\nworld\\t!").unwrap();
        assert_eq!(result, "hello\nworld\t!");
    }

    #[test]
    fn test_extract_string_content() {
        assert_eq!(extract_string_content("\"hello\"").unwrap(), "hello");
        assert_eq!(extract_string_content("'world'").unwrap(), "world");
    }

    #[test]
    fn test_parse_number_optimized() {
        assert_eq!(parse_number_optimized("42").unwrap(), 42.0);
        assert_eq!(parse_number_optimized("3.14").unwrap(), 3.14);
        assert_eq!(parse_number_optimized("-123").unwrap(), -123.0);
    }
}
