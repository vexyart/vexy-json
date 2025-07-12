// this_file: src/streaming/simple_lexer.rs

//! Simple streaming lexer that works with the existing Token enum.
//!
//! This lexer processes input incrementally and emits tokens compatible
//! with the existing vexy_json token structure.

use crate::ast::Token;
use crate::error::{Error, Result, Span};

/// A simple streaming lexer for vexy_json
#[derive(Debug, Clone)]
pub struct SimpleStreamingLexer {
    /// Current position in the overall input stream
    position: usize,
    /// Current lexer state
    state: LexerState,
    /// Pending tokens ready to be consumed
    pending_tokens: Vec<(Token, Span)>,
    /// Options for parsing
    options: crate::parser::ParserOptions,
}

/// Internal lexer state for incremental parsing
#[derive(Debug, Clone)]
enum LexerState {
    /// Normal state, looking for next token
    Normal,
    /// Inside a string literal
    InString {
        quote_char: char,
        escape: bool,
        start_pos: usize,
        content: String,
    },
    /// Inside a number
    InNumber { start_pos: usize, content: String },
    /// Inside an identifier (could be keyword or unquoted string)
    InIdentifier { start_pos: usize, content: String },
    /// Inside a single-line comment
    InSingleLineComment { start_pos: usize },
    /// Inside a multi-line comment
    InMultiLineComment { start_pos: usize, star_seen: bool },
    /// Potential comment start (seen /)
    PotentialComment { start_pos: usize },
}

impl SimpleStreamingLexer {
    /// Create a new streaming lexer with default options
    pub fn new() -> Self {
        Self::with_options(crate::parser::ParserOptions::default())
    }

    /// Create a new streaming lexer with custom options
    pub fn with_options(options: crate::parser::ParserOptions) -> Self {
        Self {
            position: 0,
            state: LexerState::Normal,
            pending_tokens: Vec::new(),
            options,
        }
    }

    /// Feed a character to the lexer
    pub fn feed_char(&mut self, ch: char) -> Result<()> {
        match self.state.clone() {
            LexerState::Normal => self.process_normal(ch)?,
            LexerState::InString {
                quote_char,
                escape,
                start_pos,
                content,
            } => {
                self.process_string(ch, quote_char, escape, start_pos, content)?;
            }
            LexerState::InNumber { start_pos, content } => {
                self.process_number(ch, start_pos, content)?;
            }
            LexerState::InIdentifier { start_pos, content } => {
                self.process_identifier(ch, start_pos, content)?;
            }
            LexerState::InSingleLineComment { start_pos } => {
                self.process_single_line_comment(ch, start_pos)?;
            }
            LexerState::InMultiLineComment {
                start_pos,
                star_seen,
            } => {
                self.process_multi_line_comment(ch, start_pos, star_seen)?;
            }
            LexerState::PotentialComment { start_pos } => {
                self.process_potential_comment(ch, start_pos)?;
            }
        }

        self.position += ch.len_utf8();
        Ok(())
    }

    /// Feed a string to the lexer
    pub fn feed_str(&mut self, s: &str) -> Result<()> {
        for ch in s.chars() {
            self.feed_char(ch)?;
        }
        Ok(())
    }

    /// Process a character in normal state
    fn process_normal(&mut self, ch: char) -> Result<()> {
        match ch {
            // Whitespace
            ' ' | '\t' | '\r' => {
                // Skip whitespace
            }
            '\n' => {
                if self.options.newline_as_comma {
                    self.emit_token(Token::Newline, self.position, self.position + 1);
                }
            }
            // Structural characters
            '{' => self.emit_token(Token::LeftBrace, self.position, self.position + 1),
            '}' => self.emit_token(Token::RightBrace, self.position, self.position + 1),
            '[' => self.emit_token(Token::LeftBracket, self.position, self.position + 1),
            ']' => self.emit_token(Token::RightBracket, self.position, self.position + 1),
            ':' => self.emit_token(Token::Colon, self.position, self.position + 1),
            ',' => self.emit_token(Token::Comma, self.position, self.position + 1),
            // String literals
            '"' => {
                self.state = LexerState::InString {
                    quote_char: '"',
                    escape: false,
                    start_pos: self.position,
                    content: String::new(),
                };
            }
            '\'' if self.options.allow_single_quotes => {
                self.state = LexerState::InString {
                    quote_char: '\'',
                    escape: false,
                    start_pos: self.position,
                    content: String::new(),
                };
            }
            // Comments
            '/' if self.options.allow_comments => {
                self.state = LexerState::PotentialComment {
                    start_pos: self.position,
                };
            }
            // Numbers
            '-' | '0'..='9' => {
                self.state = LexerState::InNumber {
                    start_pos: self.position,
                    content: ch.to_string(),
                };
            }
            // Identifiers (for keywords and unquoted strings)
            'a'..='z' | 'A'..='Z' | '_' => {
                self.state = LexerState::InIdentifier {
                    start_pos: self.position,
                    content: ch.to_string(),
                };
            }
            _ => {
                return Err(Error::UnexpectedChar(ch, self.position));
            }
        }
        Ok(())
    }

    /// Process a character inside a string
    fn process_string(
        &mut self,
        ch: char,
        quote_char: char,
        escape: bool,
        start_pos: usize,
        mut content: String,
    ) -> Result<()> {
        if escape {
            // Handle escape sequences
            content.push('\\');
            content.push(ch);
            self.state = LexerState::InString {
                quote_char,
                escape: false,
                start_pos,
                content,
            };
        } else if ch == '\\' {
            self.state = LexerState::InString {
                quote_char,
                escape: true,
                start_pos,
                content,
            };
        } else if ch == quote_char {
            // End of string - emit string token
            self.emit_token(Token::String, start_pos, self.position + 1);
            self.state = LexerState::Normal;
        } else {
            content.push(ch);
            self.state = LexerState::InString {
                quote_char,
                escape: false,
                start_pos,
                content,
            };
        }
        Ok(())
    }

    /// Process a character inside a number
    fn process_number(&mut self, ch: char, start_pos: usize, mut content: String) -> Result<()> {
        match ch {
            '0'..='9' | '.' | 'e' | 'E' | '+' | '-' => {
                content.push(ch);
                self.state = LexerState::InNumber { start_pos, content };
            }
            _ => {
                // End of number
                self.emit_token(Token::Number, start_pos, self.position);
                self.state = LexerState::Normal;
                // Reprocess this character in normal state
                self.position -= ch.len_utf8();
                return self.feed_char(ch);
            }
        }
        Ok(())
    }

    /// Process a character inside an identifier
    fn process_identifier(
        &mut self,
        ch: char,
        start_pos: usize,
        mut content: String,
    ) -> Result<()> {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                content.push(ch);
                self.state = LexerState::InIdentifier { start_pos, content };
            }
            _ => {
                // End of identifier
                let token = match content.as_str() {
                    "true" => Token::True,
                    "false" => Token::False,
                    "null" => Token::Null,
                    _ => {
                        if self.options.allow_unquoted_keys {
                            Token::UnquotedString
                        } else {
                            return Err(Error::UnexpectedChar(
                                content.chars().next().unwrap(),
                                start_pos,
                            ));
                        }
                    }
                };
                self.emit_token(token, start_pos, self.position);
                self.state = LexerState::Normal;
                // Reprocess this character in normal state
                self.position -= ch.len_utf8();
                return self.feed_char(ch);
            }
        }
        Ok(())
    }

    /// Process a character inside a single-line comment
    fn process_single_line_comment(&mut self, ch: char, start_pos: usize) -> Result<()> {
        if ch == '\n' {
            // End of comment
            self.emit_token(Token::SingleLineComment, start_pos, self.position);
            self.state = LexerState::Normal;
            // Process newline in normal state
            self.position -= ch.len_utf8();
            return self.feed_char(ch);
        }
        Ok(())
    }

    /// Process a character inside a multi-line comment
    fn process_multi_line_comment(
        &mut self,
        ch: char,
        start_pos: usize,
        star_seen: bool,
    ) -> Result<()> {
        if star_seen && ch == '/' {
            // End of comment
            self.emit_token(Token::MultiLineComment, start_pos, self.position + 1);
            self.state = LexerState::Normal;
        } else {
            self.state = LexerState::InMultiLineComment {
                start_pos,
                star_seen: ch == '*',
            };
        }
        Ok(())
    }

    /// Process a potential comment start
    fn process_potential_comment(&mut self, ch: char, start_pos: usize) -> Result<()> {
        match ch {
            '/' => {
                // Single-line comment
                self.state = LexerState::InSingleLineComment { start_pos };
            }
            '*' => {
                // Multi-line comment
                self.state = LexerState::InMultiLineComment {
                    start_pos,
                    star_seen: false,
                };
            }
            _ => {
                // Not a comment, emit error (division not supported in JSON)
                return Err(Error::UnexpectedChar('/', start_pos));
            }
        }
        Ok(())
    }

    /// Emit a token
    fn emit_token(&mut self, token: Token, start: usize, end: usize) {
        self.pending_tokens.push((token, Span { start, end }));
    }

    /// Get the next token if available
    pub fn next_token(&mut self) -> Option<(Token, Span)> {
        if self.pending_tokens.is_empty() {
            None
        } else {
            Some(self.pending_tokens.remove(0))
        }
    }

    /// Check if there are pending tokens
    #[inline(always)]
    pub fn has_tokens(&self) -> bool {
        !self.pending_tokens.is_empty()
    }

    /// Finish lexing and emit any remaining tokens
    pub fn finish(&mut self) -> Result<()> {
        match &self.state {
            LexerState::Normal => Ok(()),
            LexerState::InString { start_pos, .. } => Err(Error::UnterminatedString(*start_pos)),
            LexerState::InNumber { start_pos, .. } => {
                self.emit_token(Token::Number, *start_pos, self.position);
                Ok(())
            }
            LexerState::InIdentifier { start_pos, content } => {
                let token = match content.as_str() {
                    "true" => Token::True,
                    "false" => Token::False,
                    "null" => Token::Null,
                    _ => Token::UnquotedString,
                };
                self.emit_token(token, *start_pos, self.position);
                Ok(())
            }
            LexerState::InSingleLineComment { start_pos } => {
                self.emit_token(Token::SingleLineComment, *start_pos, self.position);
                Ok(())
            }
            LexerState::InMultiLineComment { start_pos, .. } => Err(Error::Custom(format!(
                "Unterminated comment at position {start_pos}"
            ))),
            LexerState::PotentialComment { start_pos } => {
                Err(Error::UnexpectedChar('/', *start_pos))
            }
        }
    }
}

impl Default for SimpleStreamingLexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = SimpleStreamingLexer::new();
        lexer.feed_str("{\"key\": true}").unwrap();
        lexer.finish().unwrap();

        assert_eq!(lexer.next_token().unwrap().0, Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap().0, Token::String);
        assert_eq!(lexer.next_token().unwrap().0, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().0, Token::True);
        assert_eq!(lexer.next_token().unwrap().0, Token::RightBrace);
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_incremental_string() {
        let mut lexer = SimpleStreamingLexer::new();
        lexer.feed_str("\"hel").unwrap();
        assert!(!lexer.has_tokens());

        lexer.feed_str("lo\"").unwrap();
        assert!(lexer.has_tokens());

        assert_eq!(lexer.next_token().unwrap().0, Token::String);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = SimpleStreamingLexer::new();
        lexer.feed_str("123.45").unwrap();
        lexer.finish().unwrap();

        assert_eq!(lexer.next_token().unwrap().0, Token::Number);
    }

    #[test]
    fn test_keywords() {
        let mut lexer = SimpleStreamingLexer::new();
        lexer.feed_str("true false null").unwrap();
        lexer.finish().unwrap();

        assert_eq!(lexer.next_token().unwrap().0, Token::True);
        assert_eq!(lexer.next_token().unwrap().0, Token::False);
        assert_eq!(lexer.next_token().unwrap().0, Token::Null);
    }
}
