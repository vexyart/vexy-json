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
