// this_file: crates/core/src/lazy/mod.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result, Span};
use crate::lexer::Lexer;
use crate::parser::ParserOptions;
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

/// Lazy array parsing and iteration.
pub mod array;
/// Lazy number parsing with deferred type conversion.
pub mod number;
/// Lazy object parsing with on-demand key access.
pub mod object;
/// Lazy string parsing with deferred escape processing.
pub mod string;

pub use array::LazyArray;
pub use object::LazyObject;

/// A lazily-evaluated JSON value that is parsed on-demand.
#[derive(Debug, Clone)]
pub enum LazyValue {
    /// An already-parsed value
    Resolved(Value),
    /// A deferred value with its position in the input
    Deferred {
        /// The original input string
        input: Arc<str>,
        /// Span of this value in the input
        span: Span,
        /// Parser options to use when evaluating
        options: ParserOptions,
        /// Cache for the resolved value
        cache: Arc<Mutex<Option<Value>>>,
    },
}

/// Parser for creating lazy JSON structures.
pub struct LazyParser<'a> {
    /// The input string being parsed
    pub(super) input: &'a str,
    /// Lexer for tokenization
    pub(super) lexer: Lexer<'a>,
    /// Parser options
    pub(super) options: ParserOptions,
    /// Threshold for lazy evaluation (minimum size in bytes)
    pub(super) lazy_threshold: usize,
}

impl LazyValue {
    /// Creates a new deferred lazy value.
    pub fn deferred(input: Arc<str>, span: Span, options: ParserOptions) -> Self {
        LazyValue::Deferred {
            input,
            span,
            options,
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a new resolved lazy value.
    pub fn resolved(value: Value) -> Self {
        LazyValue::Resolved(value)
    }

    /// Forces evaluation of the lazy value, returning the parsed result.
    pub fn evaluate(&self) -> Result<Value> {
        match self {
            LazyValue::Resolved(value) => Ok(value.clone()),
            LazyValue::Deferred {
                input,
                span,
                options,
                cache,
            } => {
                // Check cache first
                {
                    let cache_guard = cache.lock().unwrap();
                    if let Some(cached_value) = cache_guard.as_ref() {
                        return Ok(cached_value.clone());
                    }
                }

                // Parse the value
                let slice = &input[span.start..span.end];
                let mut parser = LazyParser::new(slice, options.clone());
                parser.set_lazy_threshold(0); // Force immediate parsing
                let value = parser.parse()?;

                // Convert LazyValue back to regular Value
                let resolved_value = Self::force_resolve(value)?;

                // Cache the result
                {
                    let mut cache_guard = cache.lock().unwrap();
                    *cache_guard = Some(resolved_value.clone());
                }

                Ok(resolved_value)
            }
        }
    }

    /// Recursively forces resolution of all lazy values in a structure.
    fn force_resolve(value: Value) -> Result<Value> {
        match value {
            Value::Object(obj) => {
                let mut resolved_obj = FxHashMap::default();
                for (key, val) in obj {
                    resolved_obj.insert(key, Self::force_resolve(val)?);
                }
                Ok(Value::Object(resolved_obj))
            }
            Value::Array(arr) => {
                let mut resolved_arr = Vec::new();
                for val in arr {
                    resolved_arr.push(Self::force_resolve(val)?);
                }
                Ok(Value::Array(resolved_arr))
            }
            other => Ok(other),
        }
    }

    /// Checks if the value is already resolved.
    pub fn is_resolved(&self) -> bool {
        matches!(self, LazyValue::Resolved(_))
    }

    /// Gets the resolved value if available, without forcing evaluation.
    pub fn try_get_resolved(&self) -> Option<Value> {
        match self {
            LazyValue::Resolved(value) => Some(value.clone()),
            LazyValue::Deferred { cache, .. } => cache.lock().unwrap().clone(),
        }
    }
}

impl<'a> LazyParser<'a> {
    /// Creates a new lazy parser.
    pub fn new(input: &'a str, options: ParserOptions) -> Self {
        LazyParser {
            input,
            lexer: Lexer::new(input),
            options,
            lazy_threshold: 1024, // Default threshold of 1KB
        }
    }

    /// Sets the threshold for lazy evaluation.
    /// Values smaller than this threshold will be parsed immediately.
    pub fn set_lazy_threshold(&mut self, threshold: usize) {
        self.lazy_threshold = threshold;
    }

    /// Parses the input with lazy evaluation.
    pub fn parse(&mut self) -> Result<Value> {
        let (token, span) = self.next_token()?;
        self.parse_value(token, span)
    }

    /// Gets the next token from the lexer.
    pub(super) fn next_token(&mut self) -> Result<(Token, Span)> {
        self.lexer.next_token_with_span()
    }

    /// Peeks at the next token without consuming it.
    pub(super) fn peek_token(&mut self) -> Result<(Token, Span)> {
        self.lexer
            .peek_with_span()
            .map(|&(token, span)| (token, span))
    }

    /// Parses a value, deciding whether to evaluate immediately or defer.
    pub(super) fn parse_value(&mut self, token: Token, span: Span) -> Result<Value> {
        match token {
            Token::LeftBrace => self.parse_object(span),
            Token::LeftBracket => self.parse_array(span),
            Token::String => self.parse_string(span),
            Token::Number => self.parse_number(span),
            Token::True => Ok(Value::Bool(true)),
            Token::False => Ok(Value::Bool(false)),
            Token::Null => Ok(Value::Null),
            _ => Err(Error::UnexpectedChar(
                self.input.chars().nth(span.start).unwrap_or('\0'),
                span.start,
            )),
        }
    }
}

/// Creates a lazy parser and parses the input with lazy evaluation.
pub fn parse_lazy(input: &str) -> Result<Value> {
    let mut parser = LazyParser::new(input, ParserOptions::default());
    parser.parse()
}

/// Creates a lazy parser with custom options and parses the input.
pub fn parse_lazy_with_options(input: &str, options: ParserOptions) -> Result<Value> {
    let mut parser = LazyParser::new(input, options);
    parser.parse()
}

/// Creates a lazy parser with a custom threshold and parses the input.
pub fn parse_lazy_with_threshold(input: &str, threshold: usize) -> Result<Value> {
    let mut parser = LazyParser::new(input, ParserOptions::default());
    parser.set_lazy_threshold(threshold);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_value_resolved() {
        let value = LazyValue::resolved(Value::String("test".to_string()));
        assert!(value.is_resolved());

        let result = value.evaluate().unwrap();
        assert_eq!(result, Value::String("test".to_string()));
    }

    #[test]
    fn test_lazy_parser_small_object() {
        let input = r#"{"name": "test", "value": 42}"#;
        let result = parse_lazy(input).unwrap();

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("name"), Some(&Value::String("test".to_string())));
                assert_eq!(
                    obj.get("value"),
                    Some(&Value::Number(crate::ast::Number::Integer(42)))
                );
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_lazy_parser_with_threshold() {
        let input = r#"{"small": "value"}"#;
        let mut parser = LazyParser::new(input, ParserOptions::default());
        parser.set_lazy_threshold(10); // Very small threshold

        let result = parser.parse().unwrap();
        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("small"), Some(&Value::String("value".to_string())));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_lazy_array() {
        let input = r#"[1, 2, 3]"#;
        let result = parse_lazy(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Number(crate::ast::Number::Integer(1)));
            }
            _ => panic!("Expected array"),
        }
    }
}
