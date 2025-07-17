// this_file: src/parser/number.rs

use crate::ast::{Number, Value};
use crate::error::{Error, Result, Span};

#[inline]
pub(super) fn parse_number_token(original_input: &str, span: Span) -> Result<Value> {
    // Extract the number content from the span
    let number_slice = &original_input[span.start..span.end];

    // DEBUG
    // eprintln!("DEBUG parse_number_token: number_slice = '{}', span = {:?}", number_slice, span);

    // Check if it has a trailing decimal point BEFORE normalization
    let has_trailing_decimal = number_slice.ends_with('.');

    // Check for alternative number formats (hex, octal, binary)
    if let Some(parsed_int) = parse_alternative_number_format(number_slice, span)? {
        return Ok(Value::Number(Number::Integer(parsed_int)));
    }

    // Normalize number format for Rust's parser
    let normalized_number = if has_trailing_decimal {
        // Convert "1." to "1.0" for Rust's parser
        format!("{number_slice}0")
    } else {
        number_slice.to_string()
    };

    // Parse the number string
    let number_value = match normalized_number.parse::<f64>() {
        Ok(f) => {
            // Check if it's actually an integer and within i64 range
            // Don't treat scientific notation or decimal notation as integers even if they have no fractional part
            // Also preserve special float values like -0.0
            let has_no_fract = f.fract() == 0.0;
            let is_finite = f.is_finite();
            // Check if the float can be accurately represented as i64
            // Note: We need to be careful here because very large floats can be >= i64::MAX
            // but when converted to i64 they get clamped to i64::MAX
            // Also check that the conversion is lossless
            let in_range = f.is_finite()
                && f >= (i64::MIN as f64)
                && f <= (i64::MAX as f64)
                && (f as i64) as f64 == f;
            let is_neg_zero = f.is_sign_negative() && f == 0.0;

            // Check if the original string contains indicators that it should be a float
            let has_decimal_point = number_slice.contains('.');
            let has_exponent = number_slice.contains('e') || number_slice.contains('E');

            // For negative zero, return Integer(0) not Float(-0.0)
            if is_neg_zero && !has_decimal_point {
                return Ok(Value::Number(Number::Integer(0)));
            }

            // Only convert to integer if:
            // - No fractional part
            // - Is finite
            // - In i64 range
            // - Not negative zero (already handled above)
            // - Either no decimal point OR trailing decimal point (vexy_json compatibility)
            // - No exponent (scientific notation should be treated as float)
            if has_no_fract
                && is_finite
                && in_range
                && !is_neg_zero
                && (!has_decimal_point || has_trailing_decimal)
                && !has_exponent
            {
                Number::Integer(f as i64)
            } else {
                Number::Float(f)
            }
        }
        Err(_) => {
            return Err(Error::InvalidNumber(span.start));
        }
    };

    Ok(Value::Number(number_value))
}

/// Parse alternative number formats (hex, octal, binary, underscore separators)
/// Returns None if not an alternative integer format, Some(i64) if parsed successfully as an integer.
fn parse_alternative_number_format(input: &str, span: Span) -> Result<Option<i64>> {
    // Remove underscore separators first
    let cleaned = input.replace('_', "");

    // Handle signed numbers
    let (sign, number_str) = if let Some(rest) = cleaned.strip_prefix('-') {
        (-1i64, rest)
    } else if let Some(rest) = cleaned.strip_prefix('+') {
        (1i64, rest)
    } else {
        (1i64, cleaned.as_str())
    };
    

    // Parse different number formats
    let parsed_value = if number_str.starts_with("0x") || number_str.starts_with("0X") {
        // Hexadecimal
        let hex_str = &number_str[2..];
        if hex_str.is_empty() {
            return Err(Error::InvalidNumber(span.start));
        }
        match i64::from_str_radix(hex_str, 16) {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidNumber(span.start)),
        }
    } else if number_str.starts_with("0o") || number_str.starts_with("0O") {
        // Octal
        let octal_str = &number_str[2..];
        if octal_str.is_empty() {
            return Err(Error::InvalidNumber(span.start));
        }
        match i64::from_str_radix(octal_str, 8) {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidNumber(span.start)),
        }
    } else if number_str.starts_with("0b") || number_str.starts_with("0B") {
        // Binary
        let binary_str = &number_str[2..];
        if binary_str.is_empty() {
            return Err(Error::InvalidNumber(span.start));
        }
        match i64::from_str_radix(binary_str, 2) {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidNumber(span.start)),
        }
    } else if input.contains('_')
        && !input.contains('.')
        && !input.contains('e')
        && !input.contains('E')
    {
        // Number with underscore separators (decimal integer only)
        // If it contains a decimal or exponent, it's not a simple integer alternative format
        match number_str.parse::<i64>() {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidNumber(span.start)),
        }
    } else {
        // Not an alternative format
        return Ok(None);
    };

    // Apply sign and return
    let result = if sign == -1 {
        parsed_value
            .checked_neg()
            .ok_or(Error::InvalidNumber(span.start))?
    } else {
        parsed_value
    };

    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;

    #[test]
    fn test_parse_number_token_integers() {
        // Positive integers
        assert_eq!(parse_number_token("42", Span::new(0, 2)).unwrap(), Value::Number(Number::Integer(42)));
        assert_eq!(parse_number_token("0", Span::new(0, 1)).unwrap(), Value::Number(Number::Integer(0)));
        assert_eq!(parse_number_token("123", Span::new(0, 3)).unwrap(), Value::Number(Number::Integer(123)));
        
        // Negative integers
        assert_eq!(parse_number_token("-42", Span::new(0, 3)).unwrap(), Value::Number(Number::Integer(-42)));
        assert_eq!(parse_number_token("-0", Span::new(0, 2)).unwrap(), Value::Number(Number::Integer(0)));
    }

    #[test]
    fn test_parse_number_token_floats() {
        // Standard floats
        assert_eq!(parse_number_token("3.14", Span::new(0, 4)).unwrap(), Value::Number(Number::Float(3.14)));
        assert_eq!(parse_number_token("0.5", Span::new(0, 3)).unwrap(), Value::Number(Number::Float(0.5)));
        assert_eq!(parse_number_token("-1.5", Span::new(0, 4)).unwrap(), Value::Number(Number::Float(-1.5)));
        
        // Scientific notation
        assert_eq!(parse_number_token("1e10", Span::new(0, 4)).unwrap(), Value::Number(Number::Float(1e10)));
        assert_eq!(parse_number_token("1E10", Span::new(0, 4)).unwrap(), Value::Number(Number::Float(1E10)));
        assert_eq!(parse_number_token("1e-5", Span::new(0, 4)).unwrap(), Value::Number(Number::Float(1e-5)));
        assert_eq!(parse_number_token("2.5e3", Span::new(0, 5)).unwrap(), Value::Number(Number::Float(2500.0)));
    }

    #[test]
    fn test_parse_number_token_trailing_decimal() {
        // Trailing decimal should be treated as integer for vexy_json compatibility
        assert_eq!(parse_number_token("1.", Span::new(0, 2)).unwrap(), Value::Number(Number::Integer(1)));
        assert_eq!(parse_number_token("42.", Span::new(0, 3)).unwrap(), Value::Number(Number::Integer(42)));
        assert_eq!(parse_number_token("-5.", Span::new(0, 3)).unwrap(), Value::Number(Number::Integer(-5)));
    }

    #[test]
    fn test_parse_alternative_number_format_hex() {
        // Hexadecimal
        assert_eq!(parse_alternative_number_format("0x10", Span::new(0, 4)).unwrap(), Some(16));
        assert_eq!(parse_alternative_number_format("0X10", Span::new(0, 4)).unwrap(), Some(16));
        assert_eq!(parse_alternative_number_format("0xff", Span::new(0, 4)).unwrap(), Some(255));
        assert_eq!(parse_alternative_number_format("0xFF", Span::new(0, 4)).unwrap(), Some(255));
        assert_eq!(parse_alternative_number_format("-0x10", Span::new(0, 5)).unwrap(), Some(-16));
        assert_eq!(parse_alternative_number_format("+0x10", Span::new(0, 5)).unwrap(), Some(16));
        
        // Invalid hex
        assert!(parse_alternative_number_format("0x", Span::new(0, 2)).is_err());
        assert!(parse_alternative_number_format("0xg", Span::new(0, 3)).is_err());
    }

    #[test]
    fn test_parse_alternative_number_format_octal() {
        let span = Span::new(0, 4);
        
        // Octal
        assert_eq!(parse_alternative_number_format("0o10", span).unwrap(), Some(8));
        assert_eq!(parse_alternative_number_format("0O10", span).unwrap(), Some(8));
        assert_eq!(parse_alternative_number_format("0o777", span).unwrap(), Some(511));
        assert_eq!(parse_alternative_number_format("-0o10", span).unwrap(), Some(-8));
        assert_eq!(parse_alternative_number_format("+0o10", span).unwrap(), Some(8));
        
        // Invalid octal
        assert!(parse_alternative_number_format("0o", span).is_err());
        assert!(parse_alternative_number_format("0o8", span).is_err());
    }

    #[test]
    fn test_parse_alternative_number_format_binary() {
        let span = Span::new(0, 6);
        
        // Binary
        assert_eq!(parse_alternative_number_format("0b1010", span).unwrap(), Some(10));
        assert_eq!(parse_alternative_number_format("0B1010", span).unwrap(), Some(10));
        assert_eq!(parse_alternative_number_format("0b1111", span).unwrap(), Some(15));
        assert_eq!(parse_alternative_number_format("-0b1010", span).unwrap(), Some(-10));
        assert_eq!(parse_alternative_number_format("+0b1010", span).unwrap(), Some(10));
        
        // Invalid binary
        assert!(parse_alternative_number_format("0b", span).is_err());
        assert!(parse_alternative_number_format("0b2", span).is_err());
    }

    #[test]
    fn test_parse_alternative_number_format_underscore_separators() {
        // Underscore separators
        assert_eq!(parse_alternative_number_format("1_000", Span::new(0, 5)).unwrap(), Some(1000));
        assert_eq!(parse_alternative_number_format("1_000_000", Span::new(0, 9)).unwrap(), Some(1000000));
        assert_eq!(parse_alternative_number_format("-1_000", Span::new(0, 6)).unwrap(), Some(-1000));
        assert_eq!(parse_alternative_number_format("+1_000", Span::new(0, 6)).unwrap(), Some(1000));
        
        // Not alternative format (contains decimal or exponent)
        assert_eq!(parse_alternative_number_format("1.5", Span::new(0, 3)).unwrap(), None);
        assert_eq!(parse_alternative_number_format("1e5", Span::new(0, 3)).unwrap(), None);
        assert_eq!(parse_alternative_number_format("1E5", Span::new(0, 3)).unwrap(), None);
        assert_eq!(parse_alternative_number_format("1.5_0", Span::new(0, 5)).unwrap(), None);
    }

    #[test]
    fn test_parse_alternative_number_format_not_alternative() {
        let span = Span::new(0, 3);
        
        // Regular decimal numbers should return None
        assert_eq!(parse_alternative_number_format("123", span).unwrap(), None);
        assert_eq!(parse_alternative_number_format("3.14", span).unwrap(), None);
        assert_eq!(parse_alternative_number_format("1e10", span).unwrap(), None);
        assert_eq!(parse_alternative_number_format("-42", span).unwrap(), None);
    }

    #[test]
    fn test_parse_alternative_number_format_overflow() {
        let span = Span::new(0, 20);
        
        // Test overflow handling
        let max_hex = "0x7FFFFFFFFFFFFFFF"; // i64::MAX in hex
        assert_eq!(parse_alternative_number_format(max_hex, span).unwrap(), Some(i64::MAX));
        
        // This should cause an overflow error
        let overflow_hex = "0x8000000000000000"; // i64::MAX + 1
        assert!(parse_alternative_number_format(overflow_hex, span).is_err());
    }

    #[test]
    fn test_parse_number_token_edge_cases() {
        // Large numbers that should be floats
        let large_num = "999999999999999999999";
        assert!(matches!(
            parse_number_token(large_num, Span::new(0, large_num.len())).unwrap(),
            Value::Number(Number::Float(_))
        ));
        
        // Special float values - these might not be supported by all parsers
        // Comment out inf tests as they might not be standard JSON
        // assert!(matches!(
        //     parse_number_token("inf", Span::new(0, 3)).unwrap(),
        //     Value::Number(Number::Float(f)) if f.is_infinite() && f.is_sign_positive()
        // ));
        
        // assert!(matches!(
        //     parse_number_token("-inf", Span::new(0, 4)).unwrap(),
        //     Value::Number(Number::Float(f)) if f.is_infinite() && f.is_sign_negative()
        // ));
    }

    #[test]
    fn test_parse_number_token_invalid() {
        // Invalid number formats should return errors
        assert!(parse_number_token("abc", Span::new(0, 3)).is_err());
        // Note: 1.2.3 might actually parse as 1.2 followed by .3, depending on the parser
        // Let's test a clearly invalid format
        assert!(parse_number_token("1e", Span::new(0, 2)).is_err());
        assert!(parse_number_token("", Span::new(0, 0)).is_err());
    }
}
