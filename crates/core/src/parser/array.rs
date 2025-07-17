// this_file: src/parser/array.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result};
use crate::parser::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_array(&mut self) -> Result<Value> {
        self.check_depth()?;
        self.state.depth += 1;

        if !matches!(self.current_token, Some((Token::LeftBracket, _))) {
            return Err(Error::Expected {
                expected: "[".to_string(),
                found: format!("{:?}", self.current_token),
                position: self.lexer.position(),
            });
        }
        self.advance()?;

        let mut array = Vec::new();

        loop {
            self.skip_comments_and_newlines()?;

            if let Some((Token::RightBracket, _)) = self.current_token {
                self.advance()?;
                break;
            }

            // Check if we have a separator (comma or newline) which means null value
            if self.is_separator() {
                array.push(Value::Null);
                self.advance()?;
                // Check for consecutive separators
                while self.is_separator() {
                    array.push(Value::Null);
                    self.advance()?;
                }
                // After handling separators, continue to next iteration to either
                // parse the next value or handle end of array
                continue;
            }

            // Parse value (only reached if not a separator)
            let value = self.parse_value()?;
            array.push(value);

            // Check for separator or end (only after parsing a value)
            // Skip comments first, but don't skip newlines yet - we need to check if newlines are separators
            self.skip_comments()?;

            match self.current_token {
                Some((Token::Comma, _)) => {
                    self.advance()?;
                    self.skip_comments_and_newlines()?;
                    // Check for trailing comma
                    if matches!(self.current_token, Some((Token::RightBracket, _)))
                        && !self.options.allow_trailing_commas
                    {
                        return Err(Error::TrailingComma(self.lexer.position()));
                    }
                }
                Some((Token::Newline, _)) if self.options.newline_as_comma => {
                    self.advance()?;
                    self.skip_comments_and_newlines()?;
                    // Check for trailing newline
                    if matches!(self.current_token, Some((Token::RightBracket, _)))
                        && !self.options.allow_trailing_commas
                    {
                        return Err(Error::TrailingComma(self.lexer.position()));
                    }
                }
                Some((Token::RightBracket, _)) => {
                    continue;
                }
                _ => {
                    // If we have newline_as_comma enabled, try skipping comments and newlines
                    // to see if we find a separator or end token after comments
                    if self.options.newline_as_comma {
                        // Save the current state in case we need to restore
                        let _saved_pos = self.lexer.position();
                        let saved_token = self.current_token;

                        // Skip any additional comments and newlines
                        self.skip_comments_and_newlines()?;

                        match self.current_token {
                            Some((Token::RightBracket, _)) => {
                                // Found the end after skipping comments/newlines
                                continue;
                            }
                            Some((Token::Number, _))
                            | Some((Token::String, _))
                            | Some((Token::UnquotedString, _))
                            | Some((Token::True, _))
                            | Some((Token::False, _))
                            | Some((Token::Null, _))
                            | Some((Token::LeftBrace, _))
                            | Some((Token::LeftBracket, _)) => {
                                // Found a value after comments/newlines, which means the newlines were separators
                                // Continue to next iteration to parse this value
                                continue;
                            }
                            _ => {
                                // TODO: Restore state and return error with the original token
                                // self.lexer.set_position(saved_pos);
                                self.current_token = saved_token;
                                return Err(Error::Expected {
                                    expected: ", or ] or newline".to_string(),
                                    found: format!("{:?}", self.current_token),
                                    position: self.lexer.position(),
                                });
                            }
                        }
                    } else {
                        return Err(Error::Expected {
                            expected: ", or ] or newline".to_string(),
                            found: format!("{:?}", self.current_token),
                            position: self.lexer.position(),
                        });
                    }
                }
            }
        }

        self.state.depth -= 1;
        Ok(Value::Array(array))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Number, Value};
    use crate::parser::{Parser, ParserOptions};
    
    // Helper functions for creating test values
    fn n(num: i64) -> Value {
        Value::Number(Number::Integer(num))
    }
    
    fn s(s: &str) -> Value {
        Value::String(s.to_string())
    }
    
    fn arr(items: Vec<Value>) -> Value {
        Value::Array(items)
    }

    #[test]
    fn test_parse_empty_array() {
        let input = "[]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn test_parse_simple_array() {
        let input = "[1, 2, 3]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_mixed_type_array() {
        let input = r#"[1, "hello", true, null]"#;
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![
            n(1),
            s("hello"),
            Value::Bool(true),
            Value::Null
        ]));
    }

    #[test]
    fn test_parse_nested_arrays() {
        let input = "[[1, 2], [3, 4]]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![
            arr(vec![n(1), n(2)]),
            arr(vec![n(3), n(4)])
        ]));
    }

    #[test]
    fn test_parse_array_with_trailing_comma() {
        let input = "[1, 2, 3,]";
        let mut options = ParserOptions::default();
        options.allow_trailing_commas = true;
        let mut parser = Parser::new(input, options);
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_array_trailing_comma_not_allowed() {
        let input = "[1, 2, 3,]";
        let mut options = ParserOptions::default();
        options.allow_trailing_commas = false;
        let mut parser = Parser::new(input, options);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_with_newlines_as_commas() {
        let input = "[1\n2\n3]";
        let mut options = ParserOptions::default();
        options.newline_as_comma = true;
        let mut parser = Parser::new(input, options);
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_array_sparse_consecutive_commas() {
        let input = "[1,,3]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), Value::Null, n(3)]));
    }

    #[test]
    fn test_parse_array_starting_with_comma() {
        let input = "[,1,2]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![Value::Null, n(1), n(2)]));
    }

    #[test]
    fn test_parse_array_multiple_consecutive_commas() {
        let input = "[1,,,4]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), Value::Null, Value::Null, n(4)]));
    }

    #[test]
    fn test_parse_array_with_comments() {
        let input = r#"[1, /* comment */ 2, 3]"#;
        let mut options = ParserOptions::default();
        options.allow_comments = true;
        let mut parser = Parser::new(input, options);
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_array_with_line_comments() {
        let input = "[\n  1, // comment\n  2,\n  3\n]";
        let mut options = ParserOptions::default();
        options.allow_comments = true;
        let mut parser = Parser::new(input, options);
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_array_with_trailing_newline_comma() {
        let input = "[1\n2\n3\n]";
        let mut options = ParserOptions::default();
        options.newline_as_comma = true;
        options.allow_trailing_commas = true;
        let mut parser = Parser::new(input, options);
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_array_deeply_nested() {
        let input = "[[[[1]]]]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![
            arr(vec![
                arr(vec![
                    arr(vec![n(1)])
                ])
            ])
        ]));
    }

    #[test]
    fn test_parse_array_with_objects() {
        let input = r#"[{"a": 1}, {"b": 2}]"#;
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            
            // Check first object
            if let Value::Object(obj1) = &arr[0] {
                assert_eq!(obj1.get("a").unwrap(), &n(1));
            } else {
                panic!("Expected object at index 0");
            }
            
            // Check second object
            if let Value::Object(obj2) = &arr[1] {
                assert_eq!(obj2.get("b").unwrap(), &n(2));
            } else {
                panic!("Expected object at index 1");
            }
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_array_depth_limit() {
        let input = "[".repeat(1000) + &"]".repeat(1000);
        let mut options = ParserOptions::default();
        options.max_depth = 100;
        let mut parser = Parser::new(&input, options);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_whitespace_handling() {
        let input = "[\n  1,\n  2,\n  3\n]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3)]));
    }

    #[test]
    fn test_parse_array_mixed_separators() {
        let input = "[1, 2\n3, 4]";
        let mut options = ParserOptions::default();
        options.newline_as_comma = true;
        let mut parser = Parser::new(input, options);
        let result = parser.parse().unwrap();
        assert_eq!(result, arr(vec![n(1), n(2), n(3), n(4)]));
    }

    #[test]
    fn test_parse_array_error_unclosed() {
        let input = "[1, 2, 3";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_error_invalid_token() {
        let input = "[1, 2, @]";
        let mut parser = Parser::new(input, ParserOptions::default());
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_error_missing_separator() {
        let input = "[1 2 3]";
        let mut options = ParserOptions::default();
        options.newline_as_comma = false; // Ensure newlines are not treated as commas
        let mut parser = Parser::new(input, options);
        let result = parser.parse();
        // This should fail because whitespace is not a valid separator
        assert!(result.is_err());
    }
}
