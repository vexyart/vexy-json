// this_file: crates/core/src/optimization/zero_copy.rs

//! Zero-copy and minimal allocation parsing optimizations.

use crate::ast::{Number, Value};
use crate::error::{Error, Result};
use std::borrow::Cow;

/// Zero-copy string parsing that avoids allocations when possible.
#[inline]
pub fn parse_string_zero_copy(input: &str, start: usize, end: usize) -> Result<Cow<'_, str>> {
    if end - start < 2 {
        return Err(Error::UnterminatedString(start));
    }

    let content = &input[start + 1..end - 1]; // Remove quotes

    // Fast path: if no escapes, return borrowed string
    if !content.contains('\\') {
        return Ok(Cow::Borrowed(content));
    }

    // Slow path: allocate and unescape
    Ok(Cow::Owned(unescape_minimal(content)?))
}

/// Minimal escape processing optimized for common cases.
#[inline]
fn unescape_minimal(s: &str) -> Result<String> {
    let mut result = String::with_capacity(s.len());
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
                    // Unicode escape - simplified
                    let mut code = 0u32;
                    for _ in 0..4 {
                        if let Some(hex_char) = chars.next() {
                            if let Some(digit) = hex_char.to_digit(16) {
                                code = (code << 4) | digit;
                            } else {
                                return Err(Error::InvalidEscape(0));
                            }
                        } else {
                            return Err(Error::InvalidEscape(0));
                        }
                    }
                    if let Some(unicode_char) = std::char::from_u32(code) {
                        result.push(unicode_char);
                    } else {
                        return Err(Error::InvalidUnicode(0));
                    }
                }
                Some('x') => {
                    // ASCII hex escape
                    let mut code = 0u8;
                    for _ in 0..2 {
                        if let Some(hex_char) = chars.next() {
                            if let Some(digit) = hex_char.to_digit(16) {
                                code = (code << 4) | (digit as u8);
                            } else {
                                return Err(Error::InvalidEscape(0));
                            }
                        } else {
                            return Err(Error::InvalidEscape(0));
                        }
                    }
                    result.push(code as char);
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

/// Fast number parsing for common integer cases.
#[inline]
pub fn parse_number_fast(s: &str) -> Result<Value> {
    // Fast path for simple integers
    if s.len() <= 10 && !s.contains('.') && !s.contains('e') && !s.contains('E') {
        if let Ok(int_val) = s.parse::<i64>() {
            if int_val >= i32::MIN as i64 && int_val <= i32::MAX as i64 {
                return Ok(Value::Number(Number::Float(int_val as f64)));
            }
        }
    }

    // Standard float parsing
    match s.parse::<f64>() {
        Ok(f) => Ok(Value::Number(Number::Float(f))),
        Err(_) => Err(Error::InvalidNumber(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_no_escapes() {
        let result = parse_string_zero_copy("\"hello\"", 0, 7).unwrap();
        match result {
            Cow::Borrowed(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected borrowed string"),
        }
    }

    #[test]
    fn test_zero_copy_with_escapes() {
        let result = parse_string_zero_copy("\"hello\\nworld\"", 0, 14).unwrap();
        match result {
            Cow::Owned(s) => assert_eq!(s, "hello\nworld"),
            _ => panic!("Expected owned string"),
        }
    }

    #[test]
    fn test_fast_number_integer() {
        let result = parse_number_fast("42").unwrap();
        match result {
            Value::Number(n) => assert_eq!(n.as_f64(), 42.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_fast_number_float() {
        let result = parse_number_fast("3.14").unwrap();
        match result {
            Value::Number(n) => assert!((n.as_f64() - 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected number"),
        }
    }
}
