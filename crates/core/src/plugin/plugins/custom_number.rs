//! Custom number format parsing plugin

use crate::ast::{Number, Value};
use crate::error::{Error, Result};
use crate::plugin::ParserPlugin;
use std::any::Any;

/// Custom number format plugin for parsing hex, octal, binary, and other formats
pub struct CustomNumberFormatPlugin {
    /// Enable hex number parsing (0x prefix)
    parse_hex: bool,
    /// Enable octal number parsing (0o prefix)
    parse_octal: bool,
    /// Enable binary number parsing (0b prefix)
    parse_binary: bool,
    /// Enable underscore separators in numbers
    allow_underscores: bool,
    /// Enable infinity and NaN
    allow_special_floats: bool,
}

impl CustomNumberFormatPlugin {
    /// Create a new custom number format plugin
    pub fn new() -> Self {
        CustomNumberFormatPlugin {
            parse_hex: true,
            parse_octal: true,
            parse_binary: true,
            allow_underscores: true,
            allow_special_floats: true,
        }
    }

    /// Parse a custom number format
    fn parse_custom_number(&self, s: &str) -> Option<Number> {
        let s = if self.allow_underscores {
            s.replace('_', "")
        } else {
            s.to_string()
        };

        // Check for special float values
        if self.allow_special_floats {
            match s.to_lowercase().as_str() {
                "infinity" | "inf" | "+infinity" | "+inf" => {
                    return Some(Number::Float(f64::INFINITY));
                }
                "-infinity" | "-inf" => {
                    return Some(Number::Float(f64::NEG_INFINITY));
                }
                "nan" => {
                    return Some(Number::Float(f64::NAN));
                }
                _ => {}
            }
        }

        // Check for hex prefix
        if self.parse_hex && (s.starts_with("0x") || s.starts_with("0X")) {
            if let Ok(n) = i64::from_str_radix(&s[2..], 16) {
                return Some(Number::Integer(n));
            }
        }

        // Check for octal prefix
        if self.parse_octal && (s.starts_with("0o") || s.starts_with("0O")) {
            if let Ok(n) = i64::from_str_radix(&s[2..], 8) {
                return Some(Number::Integer(n));
            }
        }

        // Check for binary prefix
        if self.parse_binary && (s.starts_with("0b") || s.starts_with("0B")) {
            if let Ok(n) = i64::from_str_radix(&s[2..], 2) {
                return Some(Number::Integer(n));
            }
        }

        // Try parsing with underscores removed
        if self.allow_underscores
            && s.contains(|c: char| {
                c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-'
            })
        {
            if let Ok(n) = s.parse::<i64>() {
                return Some(Number::Integer(n));
            }
            if let Ok(f) = s.parse::<f64>() {
                return Some(Number::Float(f));
            }
        }

        None
    }
}

impl Default for CustomNumberFormatPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserPlugin for CustomNumberFormatPlugin {
    fn name(&self) -> &str {
        "custom_number_format"
    }

    fn on_number(&mut self, value: &str, _path: &str) -> Result<Value> {
        // First try custom formats
        if let Some(number) = self.parse_custom_number(value) {
            return Ok(Value::Number(number));
        }

        // Fall back to standard parsing
        if let Ok(n) = value.parse::<i64>() {
            Ok(Value::Number(Number::Integer(n)))
        } else if let Ok(f) = value.parse::<f64>() {
            Ok(Value::Number(Number::Float(f)))
        } else {
            Err(Error::InvalidNumber(0))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_numbers() {
        let mut plugin = CustomNumberFormatPlugin::new();

        let value = plugin.on_number("0xFF", "$").unwrap();
        assert_eq!(value, Value::Number(Number::Integer(255)));

        let value = plugin.on_number("0x10", "$").unwrap();
        assert_eq!(value, Value::Number(Number::Integer(16)));
    }

    #[test]
    fn test_octal_numbers() {
        let mut plugin = CustomNumberFormatPlugin::new();

        let value = plugin.on_number("0o755", "$").unwrap();
        assert_eq!(value, Value::Number(Number::Integer(493))); // 7*64 + 5*8 + 5
    }

    #[test]
    fn test_binary_numbers() {
        let mut plugin = CustomNumberFormatPlugin::new();

        let value = plugin.on_number("0b1010", "$").unwrap();
        assert_eq!(value, Value::Number(Number::Integer(10)));
    }

    #[test]
    fn test_underscores() {
        let mut plugin = CustomNumberFormatPlugin::new();

        let value = plugin.on_number("1_000_000", "$").unwrap();
        assert_eq!(value, Value::Number(Number::Integer(1000000)));

        let value = plugin.on_number("3.14_159", "$").unwrap();
        if let Value::Number(Number::Float(f)) = value {
            assert!((f - 3.14159).abs() < 0.00001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_special_floats() {
        let mut plugin = CustomNumberFormatPlugin::new();

        let value = plugin.on_number("Infinity", "$").unwrap();
        if let Value::Number(Number::Float(f)) = value {
            assert!(f.is_infinite() && f.is_sign_positive());
        } else {
            panic!("Expected float");
        }

        let value = plugin.on_number("-Infinity", "$").unwrap();
        if let Value::Number(Number::Float(f)) = value {
            assert!(f.is_infinite() && f.is_sign_negative());
        } else {
            panic!("Expected float");
        }

        let value = plugin.on_number("NaN", "$").unwrap();
        if let Value::Number(Number::Float(f)) = value {
            assert!(f.is_nan());
        } else {
            panic!("Expected float");
        }
    }
}
