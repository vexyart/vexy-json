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
        format!("{}0", number_slice)
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
            let _has_exponent = number_slice.contains('e') || number_slice.contains('E');

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
            // - If has exponent, allow it only if no decimal point
            if has_no_fract
                && is_finite
                && in_range
                && !is_neg_zero
                && (!has_decimal_point || has_trailing_decimal)
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
    let (sign, number_str) = if cleaned.starts_with('-') {
        (-1i64, &cleaned[1..])
    } else if cleaned.starts_with('+') {
        (1i64, &cleaned[1..])
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
        match cleaned.parse::<i64>() {
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
