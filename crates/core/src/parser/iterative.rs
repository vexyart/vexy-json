// this_file: crates/core/src/parser/iterative.rs

//! Stack-based iterative parser implementation for Vexy JSON.
//!
//! This parser uses an explicit stack instead of recursion to avoid stack overflow
//! on deeply nested JSON structures. It's particularly useful for parsing large
//! JSON documents with deep nesting.

use crate::ast::{Number, Token, Value};
use crate::error::{Error, Result, Span};
use crate::lexer::{JsonLexer, Lexer};
use crate::parser::ParserOptions;
use rustc_hash::FxHashMap;

/// Parsing context for the iterative parser stack.
#[derive(Debug, Clone)]
enum ParseContext {
    /// Parsing a value at the top level
    Value,
    /// Parsing an object - need to parse key-value pairs
    Object {
        /// The object being built
        object: FxHashMap<String, Value>,
        /// The current key being parsed (None if expecting a key)
        current_key: Option<String>,
        /// Whether we're expecting a key (true) or value (false)
        expecting_key: bool,
    },
    /// Parsing an array - need to parse elements
    Array {
        /// The array being built
        array: Vec<Value>,
    },
}

/// A stack-based iterative parser for JSON.
///
/// This parser uses an explicit stack to track parsing context, making it
/// suitable for parsing deeply nested JSON without stack overflow concerns.
pub struct IterativeParser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<(Token, Span)>,
    options: ParserOptions,
    /// Stack for tracking parsing context
    parse_stack: Vec<ParseContext>,
    /// The final result value
    result: Option<Value>,
}

impl<'a> IterativeParser<'a> {
    /// Creates a new iterative parser.
    pub fn new(input: &'a str, options: ParserOptions) -> Self {
        Self {
            lexer: Lexer::new(input),
            current_token: None,
            options,
            parse_stack: Vec::new(),
            result: None,
        }
    }

    /// Parses the input and returns the parsed value.
    pub fn parse(&mut self) -> Result<Value> {
        self.advance()?;

        // Start with a top-level value context
        self.parse_stack.push(ParseContext::Value);

        // Main parsing loop
        while !self.parse_stack.is_empty() {
            match self.parse_stack.last() {
                Some(ParseContext::Value) => {
                    self.parse_stack.pop();
                    self.parse_value_and_push()?;
                }
                Some(ParseContext::Object { .. }) => {
                    self.parse_object_step()?;
                }
                Some(ParseContext::Array { .. }) => {
                    self.parse_array_step()?;
                }
                None => unreachable!(),
            }
        }

        self.expect_eof()?;

        self.result
            .take()
            .ok_or_else(|| Error::Custom("No result produced".to_string()))
    }

    /// Advances to the next token, skipping comments if allowed.
    fn advance(&mut self) -> Result<()> {
        loop {
            let (token, span) = self.lexer.next_token_with_span()?;
            self.current_token = Some((token, span));

            match &self.current_token.as_ref().unwrap().0 {
                Token::SingleLineComment | Token::MultiLineComment => {
                    if self.options.allow_comments {
                        continue; // Skip comments
                    } else {
                        return Err(Error::Custom("Comments are not allowed".to_string()));
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Gets the current token without consuming it.
    fn peek(&self) -> Option<&Token> {
        self.current_token.as_ref().map(|(token, _)| token)
    }

    /// Gets the current token's span.
    fn current_span(&self) -> Span {
        self.current_token
            .as_ref()
            .map(|(_, span)| *span)
            .unwrap_or(Span { start: 0, end: 0 })
    }

    /// Expects the end of input.
    fn expect_eof(&self) -> Result<()> {
        if let Some((Token::Eof, _)) = &self.current_token {
            Ok(())
        } else {
            Err(Error::Expected {
                expected: "end of input".to_string(),
                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                position: self.current_span().start,
            })
        }
    }

    /// Parses a JSON value and potentially pushes contexts onto the stack.
    fn parse_value(&mut self) -> Result<Value> {
        match self.peek() {
            Some(Token::LeftBrace) => {
                self.advance()?;
                self.parse_stack.push(ParseContext::Object {
                    object: FxHashMap::default(),
                    current_key: None,
                    expecting_key: true,
                });
                // Don't return a value - the object context will handle it
                Ok(Value::Null)
            }
            Some(Token::LeftBracket) => {
                self.advance()?;
                self.parse_stack
                    .push(ParseContext::Array { array: Vec::new() });
                // Don't return a value - the array context will handle it
                Ok(Value::Null)
            }
            Some(Token::String) => self.parse_string(),
            Some(Token::Number) => self.parse_number(),
            Some(Token::True) => {
                self.advance()?;
                Ok(Value::Bool(true))
            }
            Some(Token::False) => {
                self.advance()?;
                Ok(Value::Bool(false))
            }
            Some(Token::Null) => {
                self.advance()?;
                Ok(Value::Null)
            }
            Some(Token::UnquotedString) if self.options.allow_unquoted_keys => {
                self.parse_unquoted_string()
            }
            Some(token) => Err(Error::Expected {
                expected: "value".to_string(),
                found: format!("{token:?}"),
                position: self.current_span().start,
            }),
            None => Err(Error::Expected {
                expected: "value".to_string(),
                found: "EOF".to_string(),
                position: self.current_span().start,
            }),
        }
    }

    /// Parses a value and pushes it or creates a new parsing context.
    fn parse_value_and_push(&mut self) -> Result<()> {
        match self.peek() {
            Some(Token::LeftBrace) => {
                self.advance()?;
                self.parse_stack.push(ParseContext::Object {
                    object: FxHashMap::default(),
                    current_key: None,
                    expecting_key: true,
                });
            }
            Some(Token::LeftBracket) => {
                self.advance()?;
                self.parse_stack
                    .push(ParseContext::Array { array: Vec::new() });
            }
            _ => {
                let value = self.parse_value()?;
                self.push_value(value)?;
            }
        }
        Ok(())
    }

    /// Parses one step of an object.
    fn parse_object_step(&mut self) -> Result<()> {
        // Check current stack depth
        if self.parse_stack.len() >= self.options.max_depth {
            return Err(Error::DepthLimitExceeded(self.current_span().start));
        }

        let context = self.parse_stack.last().unwrap().clone();

        if let ParseContext::Object {
            mut object,
            current_key,
            expecting_key,
        } = context
        {
            // Handle empty object or immediate close
            if expecting_key && object.is_empty() && matches!(self.peek(), Some(Token::RightBrace)) {
                self.advance()?;
                self.parse_stack.pop();
                let value = Value::Object(object);
                self.push_value(value)?;
                return Ok(());
            }

            if expecting_key {
                // Parse key
                let key = self.parse_object_key()?;

                // Expect colon
                if !matches!(self.peek(), Some(Token::Colon)) {
                    return Err(Error::Expected {
                        expected: "colon".to_string(),
                        found: format!("{:?}", self.peek()),
                        position: self.current_span().start,
                    });
                }
                self.advance()?;

                // Update context to expecting value
                self.parse_stack.pop();
                self.parse_stack.push(ParseContext::Object {
                    object,
                    current_key: Some(key),
                    expecting_key: false,
                });

                // Push value parsing context
                self.parse_stack.push(ParseContext::Value);
            } else {
                // We just parsed a value, add it to the object
                if let Some(key) = current_key {
                    if let Some(value) = self.result.take() {
                        object.insert(key, value);
                    }

                    // Check for continuation
                    match self.peek() {
                        Some(Token::Comma) => {
                            self.advance()?;

                            // Handle trailing comma
                            if let Some(Token::RightBrace) = self.peek() {
                                if self.options.allow_trailing_commas {
                                    self.advance()?;
                                    self.parse_stack.pop();
                                    let value = Value::Object(object);
                                    self.push_value(value)?;
                                    return Ok(());
                                } else {
                                    return Err(Error::Custom(
                                        "Trailing comma not allowed".to_string(),
                                    ));
                                }
                            }

                            // Continue parsing next key
                            self.parse_stack.pop();
                            self.parse_stack.push(ParseContext::Object {
                                object,
                                current_key: None,
                                expecting_key: true,
                            });
                        }
                        Some(Token::RightBrace) => {
                            self.advance()?;
                            self.parse_stack.pop();
                            let value = Value::Object(object);
                            self.push_value(value)?;
                        }
                        Some(Token::Newline) if self.options.newline_as_comma => {
                            self.advance()?;

                            // Handle trailing newline
                            if let Some(Token::RightBrace) = self.peek() {
                                self.advance()?;
                                self.parse_stack.pop();
                                let value = Value::Object(object);
                                self.push_value(value)?;
                                return Ok(());
                            }

                            // Continue parsing next key
                            self.parse_stack.pop();
                            self.parse_stack.push(ParseContext::Object {
                                object,
                                current_key: None,
                                expecting_key: true,
                            });
                        }
                        Some(token) => {
                            return Err(Error::Expected {
                                expected: "comma or closing brace".to_string(),
                                found: format!("{token:?}"),
                                position: self.current_span().start,
                            });
                        }
                        None => {
                            return Err(Error::Expected {
                                expected: "comma or closing brace".to_string(),
                                found: "EOF".to_string(),
                                position: self.current_span().start,
                            });
                        }
                    }
                } else {
                    return Err(Error::Custom("Missing key in object context".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Parses one step of an array.
    fn parse_array_step(&mut self) -> Result<()> {
        // Check current stack depth
        if self.parse_stack.len() >= self.options.max_depth {
            return Err(Error::DepthLimitExceeded(self.current_span().start));
        }

        let context = self.parse_stack.last().unwrap().clone();

        if let ParseContext::Array { mut array } = context {
            // Check if this is the first step (no value parsed yet)
            if self.result.is_none() {
                // Handle empty array
                if matches!(self.peek(), Some(Token::RightBracket)) {
                    self.advance()?;
                    self.parse_stack.pop();
                    let value = Value::Array(array);
                    self.push_value(value)?;
                    return Ok(());
                }
                // Parse first element
                self.parse_stack.pop();
                self.parse_stack.push(ParseContext::Array { array });
                self.parse_stack.push(ParseContext::Value);
                return Ok(());
            }

            // If we just parsed a value, add it to the array
            if let Some(value) = self.result.take() {
                array.push(value);

                // Check for continuation
                match self.peek() {
                    Some(Token::Comma) => {
                        self.advance()?;

                        // Handle trailing comma
                        if let Some(Token::RightBracket) = self.peek() {
                            if self.options.allow_trailing_commas {
                                self.advance()?;
                                self.parse_stack.pop();
                                let value = Value::Array(array);
                                self.push_value(value)?;
                                return Ok(());
                            } else {
                                return Err(Error::Custom(
                                    "Trailing comma not allowed".to_string(),
                                ));
                            }
                        }

                        // Continue parsing next element
                        self.parse_stack.pop();
                        self.parse_stack.push(ParseContext::Array { array });
                        self.parse_stack.push(ParseContext::Value);
                    }
                    Some(Token::RightBracket) => {
                        self.advance()?;
                        self.parse_stack.pop();
                        let value = Value::Array(array);
                        self.push_value(value)?;
                    }
                    Some(Token::Newline) if self.options.newline_as_comma => {
                        self.advance()?;

                        // Handle trailing newline
                        if let Some(Token::RightBracket) = self.peek() {
                            self.advance()?;
                            self.parse_stack.pop();
                            let value = Value::Array(array);
                            self.push_value(value)?;
                            return Ok(());
                        }

                        // Continue parsing next element
                        self.parse_stack.pop();
                        self.parse_stack.push(ParseContext::Array { array });
                        self.parse_stack.push(ParseContext::Value);
                    }
                    Some(token) => {
                        return Err(Error::Expected {
                            expected: "comma or closing bracket".to_string(),
                            found: format!("{token:?}"),
                            position: self.current_span().start,
                        });
                    }
                    None => {
                        return Err(Error::Expected {
                            expected: "comma or closing bracket".to_string(),
                            found: "EOF".to_string(),
                            position: self.current_span().start,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Parses an object key (string or unquoted string).
    fn parse_object_key(&mut self) -> Result<String> {
        match self.peek() {
            Some(Token::String) => {
                if let Some((Token::String, span)) = self.current_token {
                    let key = self.parse_string_from_span(span)?;
                    self.advance()?;
                    if let Value::String(s) = key {
                        Ok(s)
                    } else {
                        unreachable!("parse_string_from_span should return a String")
                    }
                } else {
                    unreachable!("Token::String should be present")
                }
            }
            Some(Token::UnquotedString) if self.options.allow_unquoted_keys => {
                if let Some((Token::UnquotedString, span)) = self.current_token {
                    let key = self.lexer.span_text(&span).to_string();
                    self.advance()?;
                    Ok(key)
                } else {
                    unreachable!("Token::UnquotedString should be present")
                }
            }
            Some(token) => Err(Error::Expected {
                expected: "string key".to_string(),
                found: format!("{token:?}"),
                position: self.current_span().start,
            }),
            None => Err(Error::Expected {
                expected: "string key".to_string(),
                found: "EOF".to_string(),
                position: self.current_span().start,
            }),
        }
    }

    /// Parses a JSON string.
    fn parse_string(&mut self) -> Result<Value> {
        if let Some((Token::String, span)) = self.current_token {
            let value = self.parse_string_from_span(span)?;
            self.advance()?;
            Ok(value)
        } else {
            Err(Error::Expected {
                expected: "string".to_string(),
                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                position: self.current_span().start,
            })
        }
    }

    /// Parses an unquoted string.
    fn parse_unquoted_string(&mut self) -> Result<Value> {
        if let Some((Token::UnquotedString, span)) = self.current_token {
            let value = self.lexer.span_text(&span).to_string();
            self.advance()?;
            Ok(Value::String(value))
        } else {
            Err(Error::Expected {
                expected: "unquoted string".to_string(),
                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                position: self.current_span().start,
            })
        }
    }

    /// Parses a string from a span with escape sequence processing.
    fn parse_string_from_span(&self, span: Span) -> Result<Value> {
        let text = self.lexer.span_text(&span);

        // Remove quotes
        let content = if (text.starts_with('"') && text.ends_with('"')) ||
            (text.starts_with('\'') && text.ends_with('\'') && self.options.allow_single_quotes)
        {
            &text[1..text.len() - 1]
        } else {
            return Err(Error::Custom("Invalid string format".to_string()));
        };

        // Process escape sequences
        let mut result = String::new();
        let mut chars = content.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\u{0008}'),
                    Some('f') => result.push('\u{000C}'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('u') => {
                        // Unicode escape sequence
                        let hex: String = chars.by_ref().take(4).collect();
                        if hex.len() != 4 {
                            return Err(Error::Custom("Invalid unicode escape".to_string()));
                        }
                        match u32::from_str_radix(&hex, 16) {
                            Ok(code) => {
                                if let Some(unicode_char) = char::from_u32(code) {
                                    result.push(unicode_char);
                                } else {
                                    return Err(Error::Custom(
                                        "Invalid unicode code point".to_string(),
                                    ));
                                }
                            }
                            Err(_) => {
                                return Err(Error::Custom("Invalid unicode escape".to_string()))
                            }
                        }
                    }
                    Some(ch) => {
                        return Err(Error::Custom(format!("Invalid escape sequence: \\{ch}")))
                    }
                    None => return Err(Error::Custom("Incomplete escape sequence".to_string())),
                }
            } else {
                result.push(ch);
            }
        }

        Ok(Value::String(result))
    }

    /// Parses a JSON number.
    fn parse_number(&mut self) -> Result<Value> {
        if let Some((Token::Number, span)) = self.current_token {
            let text = self.lexer.span_text(&span);

            // Try to parse as integer first
            if let Ok(int_val) = text.parse::<i64>() {
                self.advance()?;
                return Ok(Value::Number(Number::Integer(int_val)));
            }

            // Try to parse as float
            if let Ok(float_val) = text.parse::<f64>() {
                self.advance()?;
                return Ok(Value::Number(Number::Float(float_val)));
            }

            Err(Error::InvalidNumber(span.start))
        } else {
            Err(Error::Expected {
                expected: "number".to_string(),
                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                position: self.current_span().start,
            })
        }
    }

    /// Pushes a value to the result or appropriate context.
    fn push_value(&mut self, value: Value) -> Result<()> {
        self.result = Some(value);
        Ok(())
    }

}

/// Parses JSON using the stack-based iterative parser.
///
/// This function provides a stack-based iterative parser that can handle
/// deeply nested JSON structures without stack overflow concerns.
///
/// # Arguments
///
/// * `input` - The JSON string to parse
/// * `options` - Parser configuration options
///
/// # Examples
///
/// ```
/// use vexy_json_core::parser::iterative::parse_iterative;
/// use vexy_json_core::parser::ParserOptions;
///
/// let json = r#"{"key": "value", "numbers": [1, 2, 3]}"#;
/// let options = ParserOptions::default();
/// let result = parse_iterative(json, options).unwrap();
/// ```
pub fn parse_iterative(input: &str, options: ParserOptions) -> Result<Value> {
    let mut parser = IterativeParser::new(input, options);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let result = parse_iterative("null", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_parse_boolean() {
        let result = parse_iterative("true", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = parse_iterative("false", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        let result = parse_iterative("42", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Number(Number::Integer(42)));

        let result = parse_iterative("3.14", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Number(Number::Float(3.14)));
    }

    #[test]
    fn test_parse_string() {
        let result = parse_iterative(r#""hello""#, ParserOptions::default()).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let result = parse_iterative("[1, 2, 3]", ParserOptions::default()).unwrap();
        assert_eq!(
            result,
            Value::Array(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
                Value::Number(Number::Integer(3)),
            ])
        );
    }

    #[test]
    fn test_parse_object() {
        let result = parse_iterative(r#"{"key": "value"}"#, ParserOptions::default()).unwrap();
        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_parse_nested() {
        let json = r#"{"array": [1, 2, {"nested": true}]}"#;
        let result = parse_iterative(json, ParserOptions::default()).unwrap();

        let mut nested_obj = FxHashMap::default();
        nested_obj.insert("nested".to_string(), Value::Bool(true));

        let mut expected = FxHashMap::default();
        expected.insert(
            "array".to_string(),
            Value::Array(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
                Value::Object(nested_obj),
            ]),
        );

        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_parse_deeply_nested() {
        // Test with deep nesting that would cause stack overflow with recursive parser
        let mut json = String::new();
        json.push('[');
        for _ in 0..100 {
            json.push('[');
        }
        json.push_str("42");
        for _ in 0..100 {
            json.push(']');
        }
        json.push(']');

        let result = parse_iterative(&json, ParserOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_depth_limit() {
        let mut options = ParserOptions::default();
        options.max_depth = 2;

        let json = r#"{"a": {"b": {"c": "too deep"}}}"#;
        let result = parse_iterative(json, options);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::DepthLimitExceeded(_)));
    }

    #[test]
    fn test_empty_containers() {
        let result = parse_iterative("{}", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Object(FxHashMap::default()));

        let result = parse_iterative("[]", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn test_with_comments() {
        let json = r#"{"key": "value", /* comment */ "number": 42}"#;
        let result = parse_iterative(json, ParserOptions::default()).unwrap();

        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        expected.insert("number".to_string(), Value::Number(Number::Integer(42)));

        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_with_trailing_comma() {
        let json = r#"{"key": "value", "number": 42,}"#;
        let result = parse_iterative(json, ParserOptions::default()).unwrap();

        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        expected.insert("number".to_string(), Value::Number(Number::Integer(42)));

        assert_eq!(result, Value::Object(expected));
    }
}
