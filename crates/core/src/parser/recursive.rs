// this_file: crates/core/src/parser/recursive.rs

//! Clean recursive descent parser implementation for Vexy JSON.
//!
//! This parser provides a clean, textbook-style recursive descent implementation
//! that is easy to understand and maintain. It serves as an alternative to the
//! main parser for educational purposes and as a reference implementation.

use crate::ast::{Number, Token, Value};
use crate::error::{Error, Result, Span};
use crate::lexer::{JsonLexer, Lexer};
use crate::parser::ParserOptions;
use rustc_hash::FxHashMap;

/// A clean recursive descent parser for JSON.
///
/// This parser follows the classic recursive descent pattern with one method
/// per grammar rule. It's designed to be readable and maintainable rather than
/// optimized for maximum performance.
pub struct RecursiveDescentParser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<(Token, Span)>,
    options: ParserOptions,
    depth: usize,
}

impl<'a> RecursiveDescentParser<'a> {
    /// Creates a new recursive descent parser.
    pub fn new(input: &'a str, options: ParserOptions) -> Self {
        Self {
            lexer: Lexer::new(input),
            current_token: None,
            options,
            depth: 0,
        }
    }

    /// Parses the input and returns the parsed value.
    pub fn parse(&mut self) -> Result<Value> {
        self.advance()?;
        let value = self.parse_value()?;
        self.expect_eof()?;
        Ok(value)
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

    /// Consumes the current token if it matches the expected token.
    fn expect(&mut self, expected: Token) -> Result<()> {
        if let Some((token, _)) = &self.current_token {
            if std::mem::discriminant(token) == std::mem::discriminant(&expected) {
                self.advance()?;
                Ok(())
            } else {
                Err(Error::Expected {
                    expected: format!("{:?}", expected),
                    found: format!("{:?}", token),
                    position: self.current_span().start,
                })
            }
        } else {
            Err(Error::Expected {
                expected: format!("{:?}", expected),
                found: "EOF".to_string(),
                position: self.current_span().start,
            })
        }
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

    /// Checks if the current nesting depth is within limits.
    fn check_depth(&self) -> Result<()> {
        if self.depth >= self.options.max_depth {
            Err(Error::DepthLimitExceeded(self.current_span().start))
        } else {
            Ok(())
        }
    }

    /// Parses a JSON value.
    ///
    /// Grammar:
    /// ```
    /// value = object | array | string | number | boolean | null
    /// ```
    fn parse_value(&mut self) -> Result<Value> {
        match self.peek() {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::String) => self.parse_string(),
            Some(Token::Number) => self.parse_number(),
            Some(Token::True) => self.parse_true(),
            Some(Token::False) => self.parse_false(),
            Some(Token::Null) => self.parse_null(),
            Some(Token::UnquotedString) if self.options.allow_unquoted_keys => {
                self.parse_unquoted_string()
            }
            Some(token) => Err(Error::Expected {
                expected: "value".to_string(),
                found: format!("{:?}", token),
                position: self.current_span().start,
            }),
            None => Err(Error::Expected {
                expected: "value".to_string(),
                found: "EOF".to_string(),
                position: self.current_span().start,
            }),
        }
    }

    /// Parses a JSON object.
    ///
    /// Grammar:
    /// ```
    /// object = "{" [pair ("," pair)*] "}"
    /// pair = string ":" value
    /// ```
    fn parse_object(&mut self) -> Result<Value> {
        self.check_depth()?;
        self.depth += 1;

        self.expect(Token::LeftBrace)?;
        let mut object = FxHashMap::default();

        // Handle empty object
        if let Some(Token::RightBrace) = self.peek() {
            self.advance()?;
            self.depth -= 1;
            return Ok(Value::Object(object));
        }

        loop {
            // Parse key
            let key = self.parse_object_key()?;

            // Expect colon
            self.expect(Token::Colon)?;

            // Parse value
            let value = self.parse_value()?;
            object.insert(key, value);

            // Check for continuation
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance()?;

                    // Handle trailing comma
                    if let Some(Token::RightBrace) = self.peek() {
                        if self.options.allow_trailing_commas {
                            break;
                        } else {
                            return Err(Error::Custom("Trailing comma not allowed".to_string()));
                        }
                    }
                }
                Some(Token::RightBrace) => break,
                Some(Token::Newline) if self.options.newline_as_comma => {
                    self.advance()?;

                    // Handle trailing newline
                    if let Some(Token::RightBrace) = self.peek() {
                        break;
                    }
                }
                Some(token) => {
                    return Err(Error::Expected {
                        expected: "comma or closing brace".to_string(),
                        found: format!("{:?}", token),
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
        }

        self.expect(Token::RightBrace)?;
        self.depth -= 1;
        Ok(Value::Object(object))
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
                found: format!("{:?}", token),
                position: self.current_span().start,
            }),
            None => Err(Error::Expected {
                expected: "string key".to_string(),
                found: "EOF".to_string(),
                position: self.current_span().start,
            }),
        }
    }

    /// Parses a JSON array.
    ///
    /// Grammar:
    /// ```
    /// array = "[" [value ("," value)*] "]"
    /// ```
    fn parse_array(&mut self) -> Result<Value> {
        self.check_depth()?;
        self.depth += 1;

        self.expect(Token::LeftBracket)?;
        let mut array = Vec::new();

        // Handle empty array
        if let Some(Token::RightBracket) = self.peek() {
            self.advance()?;
            self.depth -= 1;
            return Ok(Value::Array(array));
        }

        loop {
            // Parse value
            array.push(self.parse_value()?);

            // Check for continuation
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance()?;

                    // Handle trailing comma
                    if let Some(Token::RightBracket) = self.peek() {
                        if self.options.allow_trailing_commas {
                            break;
                        } else {
                            return Err(Error::Custom("Trailing comma not allowed".to_string()));
                        }
                    }
                }
                Some(Token::RightBracket) => break,
                Some(Token::Newline) if self.options.newline_as_comma => {
                    self.advance()?;

                    // Handle trailing newline
                    if let Some(Token::RightBracket) = self.peek() {
                        break;
                    }
                }
                Some(token) => {
                    return Err(Error::Expected {
                        expected: "comma or closing bracket".to_string(),
                        found: format!("{:?}", token),
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

        self.expect(Token::RightBracket)?;
        self.depth -= 1;
        Ok(Value::Array(array))
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
        let content = if text.starts_with('"') && text.ends_with('"') {
            &text[1..text.len() - 1]
        } else if text.starts_with('\'') && text.ends_with('\'') && self.options.allow_single_quotes
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
                        return Err(Error::Custom(format!("Invalid escape sequence: \\{}", ch)))
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

    /// Parses a JSON boolean true.
    fn parse_true(&mut self) -> Result<Value> {
        self.expect(Token::True)?;
        Ok(Value::Bool(true))
    }

    /// Parses a JSON boolean false.
    fn parse_false(&mut self) -> Result<Value> {
        self.expect(Token::False)?;
        Ok(Value::Bool(false))
    }

    /// Parses a JSON null.
    fn parse_null(&mut self) -> Result<Value> {
        self.expect(Token::Null)?;
        Ok(Value::Null)
    }
}

/// Parses JSON using the recursive descent parser.
///
/// This function provides a clean, textbook-style recursive descent parser
/// that is easy to understand and maintain. It's designed for educational
/// purposes and as a reference implementation.
///
/// # Arguments
///
/// * `input` - The JSON string to parse
/// * `options` - Parser configuration options
///
/// # Examples
///
/// ```
/// use vexy_json_core::parser::recursive::parse_recursive;
/// use vexy_json_core::parser::ParserOptions;
///
/// let json = r#"{"key": "value", "number": 42}"#;
/// let options = ParserOptions::default();
/// let result = parse_recursive(json, options).unwrap();
/// ```
pub fn parse_recursive(input: &str, options: ParserOptions) -> Result<Value> {
    let mut parser = RecursiveDescentParser::new(input, options);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let result = parse_recursive("null", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_parse_boolean() {
        let result = parse_recursive("true", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = parse_recursive("false", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        let result = parse_recursive("42", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Number(Number::Integer(42)));

        let result = parse_recursive("3.14", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Number(Number::Float(3.14)));
    }

    #[test]
    fn test_parse_string() {
        let result = parse_recursive(r#""hello""#, ParserOptions::default()).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let result = parse_recursive("[1, 2, 3]", ParserOptions::default()).unwrap();
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
        let result = parse_recursive(r#"{"key": "value"}"#, ParserOptions::default()).unwrap();
        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_parse_nested() {
        let json = r#"{"array": [1, 2, {"nested": true}]}"#;
        let result = parse_recursive(json, ParserOptions::default()).unwrap();

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
    fn test_parse_with_comments() {
        let json = r#"{"key": "value", /* comment */ "number": 42}"#;
        let result = parse_recursive(json, ParserOptions::default()).unwrap();

        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        expected.insert("number".to_string(), Value::Number(Number::Integer(42)));

        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_parse_with_trailing_comma() {
        let json = r#"{"key": "value", "number": 42,}"#;
        let result = parse_recursive(json, ParserOptions::default()).unwrap();

        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        expected.insert("number".to_string(), Value::Number(Number::Integer(42)));

        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_parse_with_unquoted_keys() {
        let json = r#"{key: "value", number: 42}"#;
        let result = parse_recursive(json, ParserOptions::default()).unwrap();

        let mut expected = FxHashMap::default();
        expected.insert("key".to_string(), Value::String("value".to_string()));
        expected.insert("number".to_string(), Value::Number(Number::Integer(42)));

        assert_eq!(result, Value::Object(expected));
    }

    #[test]
    fn test_depth_limit() {
        let mut options = ParserOptions::default();
        options.max_depth = 2;

        let json = r#"{"a": {"b": {"c": "too deep"}}}"#;
        let result = parse_recursive(json, options);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::DepthLimitExceeded(_)));
    }

    #[test]
    fn test_escape_sequences() {
        let json = r#""hello\nworld\t\"quote\"""#;
        let result = parse_recursive(json, ParserOptions::default()).unwrap();
        assert_eq!(result, Value::String("hello\nworld\t\"quote\"".to_string()));
    }

    #[test]
    fn test_unicode_escape() {
        let json = r#""\u0041\u0042\u0043""#;
        let result = parse_recursive(json, ParserOptions::default()).unwrap();
        assert_eq!(result, Value::String("ABC".to_string()));
    }

    #[test]
    fn test_empty_containers() {
        let result = parse_recursive("{}", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Object(FxHashMap::default()));

        let result = parse_recursive("[]", ParserOptions::default()).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }
}
