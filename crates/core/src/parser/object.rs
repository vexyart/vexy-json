// this_file: src/parser/object.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result};
use crate::parser::string::parse_string_token;
use crate::parser::Parser;
use rustc_hash::FxHashMap;

impl<'a> Parser<'a> {
    pub(super) fn parse_object(&mut self) -> Result<Value> {
        self.check_depth()?;
        self.state.depth += 1;

        if !matches!(self.current_token, Some((Token::LeftBrace, _))) {
            return Err(Error::Expected {
                expected: "{".to_string(),
                found: match &self.current_token {
                    Some((token, _)) => format!("{token:?}"),
                    None => "EOF".to_string(),
                },
                position: self.lexer.position(),
            });
        }
        self.advance()?;

        let mut object = FxHashMap::default();

        loop {
            self.skip_comments_and_newlines()?;

            if let Some((Token::RightBrace, _)) = self.current_token {
                self.advance()?;
                break;
            }

            // Check if we have a separator (comma or newline) which means skip it in objects
            if self.is_separator() {
                self.advance()?;
                continue;
            }

            // Parse key
            // Debug: log the current token and options
            #[cfg(all(feature = "wasm", target_arch = "wasm32"))]
            console::log_1(
                &format!(
                    "DEBUG: current_token = {:?}, allow_unquoted_keys = {}",
                    self.current_token, self.options.allow_unquoted_keys
                )
                .into(),
            );

            let key = match self.current_token {
                Some((Token::String, span)) => {
                    // Use the helper function to parse the string
                    let k = match parse_string_token(self.original_input, span, &self.options)? {
                        Value::String(s) => s,
                        _ => unreachable!("parse_string_token should always return a String"),
                    };
                    self.advance()?;
                    k
                }
                Some((Token::UnquotedString, span)) if self.options.allow_unquoted_keys => {
                    // Extract the unquoted string content from the span
                    let k = self.original_input[span.start..span.end].to_string();
                    self.advance()?;
                    k
                }
                Some((Token::Number, _)) => {
                    // For numbers as keys, convert to string
                    let current_pos = self.lexer.position();
                    let mut number_start = current_pos;
                    if current_pos > 0 {
                        number_start = current_pos - 1;
                    }

                    // Search backwards for the start of the number
                    while number_start > 0 {
                        let prev_pos = number_start - 1;
                        if let Some(c) = self.original_input.chars().nth(prev_pos) {
                            match c {
                                '0'..='9' => {
                                    number_start = prev_pos;
                                }
                                '.' => {
                                    number_start = prev_pos;
                                }
                                'e' | 'E' => {
                                    number_start = prev_pos;
                                }
                                // Hex digits can be part of hex numbers
                                'A'..='F' | 'a'..='f' => {
                                    number_start = prev_pos;
                                }
                                // Underscore separators can be part of numbers
                                '_' => {
                                    number_start = prev_pos;
                                }
                                // Prefix characters for hex, octal, binary (excluding b,B which are hex digits)
                                'x' | 'X' | 'o' | 'O' => {
                                    number_start = prev_pos;
                                }
                                '+' | '-' => {
                                    if prev_pos == 0 {
                                        number_start = 0;
                                        break;
                                    } else if prev_pos > 0 {
                                        if let Some(before_sign) =
                                            self.original_input.chars().nth(prev_pos - 1)
                                        {
                                            if before_sign == 'e' || before_sign == 'E' {
                                                number_start = prev_pos;
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                ',' | ']' | '}' | ':' | ' ' | '\t' | '\n' | '\r' => {
                                    break;
                                }
                                _ => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }

                    // Extract the number string
                    let number_str = &self.original_input[number_start..current_pos];
                    let k = number_str.to_string();

                    self.advance()?;
                    k
                }
                _ => {
                    return Err(Error::Expected {
                        expected: "string key".to_string(),
                        found: match &self.current_token {
                            Some((token, _)) => format!("{token:?}"),
                            None => "EOF".to_string(),
                        },
                        position: self.lexer.position(),
                    });
                }
            };

            // Parse colon
            self.skip_comments_and_newlines()?;
            if !matches!(self.current_token, Some((Token::Colon, _))) {
                return Err(Error::Expected {
                    expected: ":".to_string(),
                    found: match &self.current_token {
                        Some((token, _)) => format!("{token:?}"),
                        None => "EOF".to_string(),
                    },
                    position: self.lexer.position(),
                });
            }
            self.advance()?;

            // Skip comments and newlines after colon
            self.skip_comments_and_newlines()?;

            // Parse value
            let value = self.parse_value()?;
            object.insert(key, value);

            // Check for separator or end
            // Skip comments first, but don't skip newlines yet - we need to check if newlines are separators
            self.skip_comments()?;

            match self.current_token {
                Some((Token::Comma, _)) => {
                    self.advance()?;
                    self.skip_comments_and_newlines()?;
                    // Check for trailing comma
                    if matches!(self.current_token, Some((Token::RightBrace, _)))
                        && !self.options.allow_trailing_commas
                    {
                        return Err(Error::TrailingComma(self.lexer.position()));
                    }
                }
                Some((Token::Newline, _)) if self.options.newline_as_comma => {
                    self.advance()?;
                    self.skip_comments_and_newlines()?;
                    // Check for trailing newline
                    if matches!(self.current_token, Some((Token::RightBrace, _)))
                        && !self.options.allow_trailing_commas
                    {
                        return Err(Error::TrailingComma(self.lexer.position()));
                    }
                }
                Some((Token::RightBrace, _)) => continue,
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
                            Some((Token::RightBrace, _)) => {
                                // Found the end after skipping comments/newlines
                                continue;
                            }
                            Some((Token::String, _))
                            | Some((Token::UnquotedString, _))
                            | Some((Token::Number, _)) => {
                                // Found a key after comments/newlines, which means the newlines were separators
                                // Continue to next iteration to parse this key-value pair
                                continue;
                            }
                            _ => {
                                // TODO: Restore state and return error with the original token
                                // self.lexer.set_position(saved_pos);
                                self.current_token = saved_token;
                                return Err(Error::Expected {
                                    expected: ", or } or newline".to_string(),
                                    found: match &self.current_token {
                                        Some((token, _)) => format!("{token:?}"),
                                        None => "EOF".to_string(),
                                    },
                                    position: self.lexer.position(),
                                });
                            }
                        }
                    } else {
                        return Err(Error::Expected {
                            expected: ", or } or newline".to_string(),
                            found: match &self.current_token {
                                Some((token, _)) => format!("{token:?}"),
                                None => "EOF".to_string(),
                            },
                            position: self.lexer.position(),
                        });
                    }
                }
            }
        }

        self.state.depth -= 1;
        Ok(Value::Object(object))
    }
}
