// this_file: crates/core/src/lazy/number.rs

use crate::ast::Value;
use crate::error::{Error, Result, Span};
use crate::lazy::LazyParser;

impl<'a> LazyParser<'a> {
    /// Parses a number value from a span.
    pub(super) fn parse_number(&self, span: Span) -> Result<Value> {
        let number_str = &self.input[span.start..span.end];

        if let Ok(int_val) = number_str.parse::<i64>() {
            Ok(Value::Number(crate::ast::Number::Integer(int_val)))
        } else if let Ok(float_val) = number_str.parse::<f64>() {
            Ok(Value::Number(crate::ast::Number::Float(float_val)))
        } else {
            Err(Error::InvalidNumber(span.start))
        }
    }
}
