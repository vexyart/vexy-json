// this_file: crates/core/src/lazy/string.rs

use crate::ast::Value;
use crate::error::{Error, Result, Span};
use crate::lazy::LazyParser;

impl<'a> LazyParser<'a> {
    /// Parses a string value from a span.
    pub(super) fn parse_string(&mut self, span: Span) -> Result<Value> {
        let content = self.parse_string_content(span)?;
        Ok(Value::String(content))
    }

    /// Parses string content from a span.
    pub(super) fn parse_string_content(&self, span: Span) -> Result<String> {
        let string_slice = &self.input[span.start..span.end];

        if string_slice.len() < 2 {
            return Err(Error::UnterminatedString(span.start));
        }

        // Remove quotes and handle basic unescaping
        let content = &string_slice[1..string_slice.len() - 1];
        Ok(content.to_string())
    }
}
