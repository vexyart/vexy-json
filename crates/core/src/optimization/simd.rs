// this_file: crates/core/src/optimization/simd.rs

//! SIMD-accelerated string parsing optimizations.
//!
//! This module provides SIMD-accelerated versions of string parsing functions
//! that can significantly improve performance for large strings. All functions
//! include scalar fallbacks for non-SIMD architectures.

use crate::error::{Error, Result};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD-accelerated backslash detection for fast string escape analysis.
///
/// This function uses SIMD instructions to quickly scan for backslashes in strings,
/// which is the primary bottleneck in determining if a string needs unescaping.
/// Falls back to scalar implementation on non-x86_64 architectures.
#[inline]
pub fn has_backslash_simd(s: &str) -> bool {
    // Fast path for empty or very short strings
    if s.len() < 16 {
        return s.contains('\\');
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { has_backslash_sse2(s.as_bytes()) };
        }
    }

    // Fallback to scalar implementation
    s.contains('\\')
}

/// SIMD-accelerated string validation for JSON compliance.
///
/// This function uses SIMD to quickly validate that a string contains only
/// valid JSON characters, which can speed up the parsing process.
#[inline]
pub fn validate_json_string_simd(s: &str) -> bool {
    // Fast path for empty strings
    if s.is_empty() {
        return true;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { validate_json_string_sse2(s.as_bytes()) };
        }
    }

    // Fallback to scalar validation
    validate_json_string_scalar(s)
}

/// SIMD-accelerated whitespace skipping for faster tokenization.
///
/// This function uses SIMD instructions to quickly skip over whitespace
/// characters in the input, which is a common operation during parsing.
#[inline]
pub fn skip_whitespace_simd(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { skip_whitespace_sse2(s.as_bytes()) };
        }
    }

    // Fallback to scalar implementation
    skip_whitespace_scalar(s)
}

/// SIMD-accelerated number parsing for improved performance.
///
/// This function uses SIMD instructions to accelerate common number parsing
/// operations, particularly for integer detection and validation.
#[inline]
pub fn parse_number_simd(s: &str) -> Result<f64> {
    if s.is_empty() {
        return Err(Error::InvalidNumber(0));
    }

    // Fast path for simple integers using SIMD validation
    if s.len() <= 19 && is_simple_integer_simd(s) {
        // Use optimized integer parsing
        return parse_integer_fast(s);
    }

    // Fallback to standard parsing for complex numbers
    s.parse::<f64>().map_err(|_| Error::InvalidNumber(0))
}

// Private SIMD implementations for x86_64
#[cfg(target_arch = "x86_64")]
unsafe fn has_backslash_sse2(bytes: &[u8]) -> bool {
    let backslash_vec = _mm_set1_epi8(b'\\' as i8);
    let mut i = 0;

    // Process 16 bytes at a time
    while i + 16 <= bytes.len() {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(i) as *const __m128i);
        let cmp = _mm_cmpeq_epi8(chunk, backslash_vec);
        let mask = _mm_movemask_epi8(cmp);

        if mask != 0 {
            return true;
        }

        i += 16;
    }

    // Handle remaining bytes
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            return true;
        }
        i += 1;
    }

    false
}

#[cfg(target_arch = "x86_64")]
unsafe fn validate_json_string_sse2(bytes: &[u8]) -> bool {
    let mut i = 0;

    // Process 16 bytes at a time
    while i + 16 <= bytes.len() {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(i) as *const __m128i);

        // Check for control characters (0x00-0x1F)
        let control_mask = _mm_cmpgt_epi8(chunk, _mm_set1_epi8(0x1F));
        let control_result = _mm_movemask_epi8(control_mask);

        // If we found control characters, check if they're valid JSON escapes
        if control_result != 0xFFFF {
            // Fallback to scalar validation for this chunk
            for j in 0..16 {
                if i + j >= bytes.len() {
                    break;
                }
                let byte = bytes[i + j];
                if byte < 0x20 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
                    return false;
                }
            }
        }

        i += 16;
    }

    // Handle remaining bytes
    while i < bytes.len() {
        let byte = bytes[i];
        if byte < 0x20 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
            return false;
        }
        i += 1;
    }

    true
}

#[cfg(target_arch = "x86_64")]
unsafe fn skip_whitespace_sse2(bytes: &[u8]) -> usize {
    let space_vec = _mm_set1_epi8(b' ' as i8);
    let tab_vec = _mm_set1_epi8(b'\t' as i8);
    let newline_vec = _mm_set1_epi8(b'\n' as i8);
    let carriage_vec = _mm_set1_epi8(b'\r' as i8);

    let mut i = 0;

    // Process 16 bytes at a time
    while i + 16 <= bytes.len() {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(i) as *const __m128i);

        // Check for whitespace characters
        let space_cmp = _mm_cmpeq_epi8(chunk, space_vec);
        let tab_cmp = _mm_cmpeq_epi8(chunk, tab_vec);
        let newline_cmp = _mm_cmpeq_epi8(chunk, newline_vec);
        let carriage_cmp = _mm_cmpeq_epi8(chunk, carriage_vec);

        // Combine all whitespace comparisons
        let whitespace_mask = _mm_or_si128(
            _mm_or_si128(space_cmp, tab_cmp),
            _mm_or_si128(newline_cmp, carriage_cmp),
        );

        let mask = _mm_movemask_epi8(whitespace_mask);

        // If all bytes are whitespace, continue to next chunk
        if mask == 0xFFFF {
            i += 16;
            continue;
        }

        // Find first non-whitespace byte in this chunk
        let first_non_whitespace = mask.trailing_ones() as usize;
        return i + first_non_whitespace;
    }

    // Handle remaining bytes
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            _ => break,
        }
    }

    i
}

// Scalar fallback implementations
#[inline(always)]
fn validate_json_string_scalar(s: &str) -> bool {
    for byte in s.bytes() {
        if byte < 0x20 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
            return false;
        }
    }
    true
}

#[inline(always)]
fn skip_whitespace_scalar(s: &str) -> usize {
    let mut i = 0;
    for byte in s.bytes() {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            _ => break,
        }
    }
    i
}

fn is_simple_integer_simd(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let bytes = s.as_bytes();
    let mut start = 0;

    // Check for optional minus sign
    if bytes[0] == b'-' {
        if bytes.len() == 1 {
            return false;
        }
        start = 1;
    }

    // Check if all remaining characters are digits
    for &byte in &bytes[start..] {
        if !(b'0'..=b'9').contains(&byte) {
            return false;
        }
    }

    true
}

fn parse_integer_fast(s: &str) -> Result<f64> {
    // Fast path for integers without overflow checking
    let mut result = 0i64;
    let mut negative = false;
    let mut chars = s.chars();

    // Handle sign
    if let Some(first_char) = chars.next() {
        if first_char == '-' {
            negative = true;
        } else if first_char.is_ascii_digit() {
            result = (first_char as u8 - b'0') as i64;
        } else {
            return Err(Error::InvalidNumber(0));
        }
    }

    // Process remaining digits
    for ch in chars {
        if ch.is_ascii_digit() {
            result = result * 10 + (ch as u8 - b'0') as i64;
        } else {
            return Err(Error::InvalidNumber(0));
        }
    }

    if negative {
        result = -result;
    }

    Ok(result as f64)
}

/// SIMD-accelerated string unescaping with optimized common cases.
///
/// This function uses SIMD instructions to accelerate string unescaping
/// by quickly identifying regions that need processing and optimizing
/// the handling of common escape sequences.
#[inline]
pub fn unescape_string_simd(s: &str) -> Result<String> {
    // Fast path: if no backslashes, return as-is
    if !has_backslash_simd(s) {
        return Ok(s.to_string());
    }

    // Use SIMD-accelerated escape processing
    unescape_with_simd_scan(s)
}

/// Internal function that uses SIMD to scan for escape sequences
/// and processes them efficiently.
fn unescape_with_simd_scan(s: &str) -> Result<String> {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Use SIMD to find the next backslash
        let backslash_pos = find_next_backslash_simd(&bytes[i..]);

        if let Some(pos) = backslash_pos {
            // Copy the unescaped portion
            result.push_str(std::str::from_utf8(&bytes[i..i + pos]).unwrap());
            i += pos;

            // Process the escape sequence
            if i + 1 < bytes.len() {
                match bytes[i + 1] {
                    b'n' => result.push('\n'),
                    b't' => result.push('\t'),
                    b'r' => result.push('\r'),
                    b'\\' => result.push('\\'),
                    b'"' => result.push('"'),
                    b'\'' => result.push('\''),
                    b'/' => result.push('/'),
                    b'b' => result.push('\x08'),
                    b'f' => result.push('\x0C'),
                    b'u' => {
                        // Unicode escape sequence
                        if i + 5 < bytes.len() {
                            let hex_str = std::str::from_utf8(&bytes[i + 2..i + 6]).unwrap();
                            if let Ok(code) = u32::from_str_radix(hex_str, 16) {
                                if let Some(unicode_char) = std::char::from_u32(code) {
                                    result.push(unicode_char);
                                    i += 6;
                                    continue;
                                }
                            }
                        }
                        return Err(Error::InvalidEscape(i));
                    }
                    other => {
                        result.push('\\');
                        result.push(other as char);
                    }
                }
                i += 2;
            } else {
                result.push('\\');
                i += 1;
            }
        } else {
            // No more backslashes, copy the rest
            result.push_str(std::str::from_utf8(&bytes[i..]).unwrap());
            break;
        }
    }

    Ok(result)
}

/// Find the next backslash in a byte slice using SIMD when available.
fn find_next_backslash_simd(bytes: &[u8]) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { find_next_backslash_sse2(bytes) };
        }
    }

    // Fallback to scalar search
    bytes.iter().position(|&b| b == b'\\')
}

#[cfg(target_arch = "x86_64")]
unsafe fn find_next_backslash_sse2(bytes: &[u8]) -> Option<usize> {
    let backslash_vec = _mm_set1_epi8(b'\\' as i8);
    let mut i = 0;

    // Process 16 bytes at a time
    while i + 16 <= bytes.len() {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(i) as *const __m128i);
        let cmp = _mm_cmpeq_epi8(chunk, backslash_vec);
        let mask = _mm_movemask_epi8(cmp);

        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }

        i += 16;
    }

    // Handle remaining bytes
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            return Some(i);
        }
        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_backslash_simd() {
        assert!(!has_backslash_simd("hello world"));
        assert!(has_backslash_simd("hello\\nworld"));
        assert!(has_backslash_simd("path\\to\\file"));
        assert!(!has_backslash_simd(""));

        // Test with long strings
        let long_string = "a".repeat(1000);
        assert!(!has_backslash_simd(&long_string));

        let long_string_with_backslash = format!("{}\\n{}", "a".repeat(500), "b".repeat(500));
        assert!(has_backslash_simd(&long_string_with_backslash));
    }

    #[test]
    fn test_validate_json_string_simd() {
        assert!(validate_json_string_simd("hello world"));
        assert!(validate_json_string_simd("with\ttabs"));
        assert!(validate_json_string_simd("with\nnewlines"));
        assert!(!validate_json_string_simd("with\x00null"));
        assert!(!validate_json_string_simd("with\x01control"));
        assert!(validate_json_string_simd(""));
    }

    #[test]
    fn test_skip_whitespace_simd() {
        assert_eq!(skip_whitespace_simd("hello"), 0);
        assert_eq!(skip_whitespace_simd("  hello"), 2);
        assert_eq!(skip_whitespace_simd("\t\n\r hello"), 4);
        assert_eq!(skip_whitespace_simd("   "), 3);
        assert_eq!(skip_whitespace_simd(""), 0);
    }

    #[test]
    fn test_parse_number_simd() {
        assert_eq!(parse_number_simd("42").unwrap(), 42.0);
        assert_eq!(parse_number_simd("-123").unwrap(), -123.0);
        assert_eq!(parse_number_simd("0").unwrap(), 0.0);
        assert!(parse_number_simd("").is_err());
        assert!(parse_number_simd("abc").is_err());
    }

    #[test]
    fn test_unescape_string_simd() {
        assert_eq!(unescape_string_simd("hello").unwrap(), "hello");
        assert_eq!(
            unescape_string_simd("hello\\nworld").unwrap(),
            "hello\nworld"
        );
        assert_eq!(
            unescape_string_simd("path\\\\to\\\\file").unwrap(),
            "path\\to\\file"
        );
        assert_eq!(
            unescape_string_simd("quote\\\"test").unwrap(),
            "quote\"test"
        );

        // Test unicode escapes
        assert_eq!(
            unescape_string_simd("hello\\u0041world").unwrap(),
            "helloAworld"
        );

        // Test with long strings
        let long_string = "a".repeat(1000);
        assert_eq!(unescape_string_simd(&long_string).unwrap(), long_string);
    }
}
