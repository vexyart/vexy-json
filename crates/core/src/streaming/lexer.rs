// this_file: src/streaming/lexer.rs

//! Incremental lexer for streaming parser.
//!
//! This lexer can process input character by character and maintain state
//! across chunk boundaries, making it suitable for streaming scenarios.

use crate::ast::Token;
use crate::error::{Error, Result, Span};

/// State of the incremental lexer
#[derive(Debug, Clone)]
pub struct StreamingLexer {
    /// Current position in the overall input stream
    position: usize,
    /// Buffer for incomplete tokens
    buffer: String,
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
    },
    /// Inside a number
    InNumber {
        start_pos: usize,
        has_dot: bool,
        has_exp: bool,
    },
    /// Inside an identifier (could be keyword or unquoted string)
    InIdentifier {
        start_pos: usize,
    },
    /// Inside a single-line comment
    InSingleLineComment {
        start_pos: usize,
    },
    /// Inside a multi-line comment
    InMultiLineComment {
        start_pos: usize,
        star_seen: bool,
    },
    /// Potential comment start (seen /)
    PotentialComment {
        start_pos: usize,
    },
}

impl StreamingLexer {
    /// Create a new streaming lexer with default options
    pub fn new() -> Self {
        Self::with_options(crate::parser::ParserOptions::default())
    }

    /// Create a new streaming lexer with custom options
    pub fn with_options(options: crate::parser::ParserOptions) -> Self {
        Self {
            position: 0,
            buffer: String::new(),
            state: LexerState::Normal,
            pending_tokens: Vec::new(),
            options,
        }
    }

    /// Feed a character to the lexer
    pub fn feed_char(&mut self, ch: char) -> Result<()> {
        match self.state.clone() {
            LexerState::Normal => self.process_normal(ch)?,
            LexerState::InString { quote_char, escape, start_pos } => {
                self.process_string(ch, quote_char, escape, start_pos)?;
            }
            LexerState::InNumber { start_pos, has_dot, has_exp } => {
                self.process_number(ch, start_pos, has_dot, has_exp)?;
            }
            LexerState::InIdentifier { start_pos } => {
                self.process_identifier(ch, start_pos)?;
            }
            LexerState::InSingleLineComment { start_pos } => {
                self.process_single_line_comment(ch, start_pos)?;
            }
            LexerState::InMultiLineComment { start_pos, star_seen } => {
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
                    // Emit comma token for newline
                    self.emit_token(Token::Comma, self.position, self.position + 1);
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
                self.buffer.clear();
                self.state = LexerState::InString {
                    quote_char: '"',
                    escape: false,
                    start_pos: self.position,
                };
            }
            '\'' if self.options.allow_single_quotes => {
                self.buffer.clear();
                self.state = LexerState::InString {
                    quote_char: '\'',
                    escape: false,
                    start_pos: self.position,
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
                self.buffer.clear();
                self.buffer.push(ch);
                self.state = LexerState::InNumber {
                    start_pos: self.position,
                    has_dot: false,
                    has_exp: false,
                };
            }
            // Identifiers (for keywords and unquoted strings)
            'a'..='z' | 'A'..='Z' | '_' => {
                self.buffer.clear();
                self.buffer.push(ch);
                self.state = LexerState::InIdentifier {
                    start_pos: self.position,
                };
            }
            _ => {
                return Err(Error::UnexpectedCharacter(ch, self.position));
            }
        }
        Ok(())
    }

    /// Process a character inside a string
    fn process_string(&mut self, ch: char, quote_char: char, escape: bool, start_pos: usize) -> Result<()> {
        if escape {
            // Handle escape sequences
            self.buffer.push('\\');
            self.buffer.push(ch);
            self.state = LexerState::InString {
                quote_char,
                escape: false,
                start_pos,
            };
        } else if ch == '\\' {
            self.state = LexerState::InString {
                quote_char,
                escape: true,
                start_pos,
            };
        } else if ch == quote_char {
            // End of string
            self.emit_token(
                Token::String(self.buffer.clone()),
                start_pos,
                self.position + 1,
            );
            self.state = LexerState::Normal;
        } else {
            self.buffer.push(ch);
        }
        Ok(())
    }

    /// Process a character inside a number
    fn process_number(&mut self, ch: char, start_pos: usize, has_dot: bool, has_exp: bool) -> Result<()> {
        match ch {
            '0'..='9' => {
                self.buffer.push(ch);
            }
            '.' if !has_dot && !has_exp => {
                self.buffer.push(ch);
                self.state = LexerState::InNumber {
                    start_pos,
                    has_dot: true,
                    has_exp,
                };
            }
            'e' | 'E' if !has_exp => {
                self.buffer.push(ch);
                self.state = LexerState::InNumber {
                    start_pos,
                    has_dot,
                    has_exp: true,
                };
            }
            '+' | '-' if self.buffer.ends_with('e') || self.buffer.ends_with('E') => {
                self.buffer.push(ch);
            }
            _ => {
                // End of number
                self.emit_number_token(start_pos)?;
                self.state = LexerState::Normal;
                // Reprocess this character in normal state
                self.position -= ch.len_utf8();
                return self.feed_char(ch);
            }
        }
        Ok(())
    }

    /// Process a character inside an identifier
    fn process_identifier(&mut self, ch: char, start_pos: usize) -> Result<()> {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                self.buffer.push(ch);
            }
            _ => {
                // End of identifier
                self.emit_identifier_token(start_pos)?;
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
            self.emit_token(
                Token::Comment(self.buffer.clone()),
                start_pos,
                self.position,
            );
            self.state = LexerState::Normal;
            // Process newline in normal state
            self.position -= ch.len_utf8();
            return self.feed_char(ch);
        } else {
            self.buffer.push(ch);
        }
        Ok(())
    }

    /// Process a character inside a multi-line comment
    fn process_multi_line_comment(&mut self, ch: char, start_pos: usize, star_seen: bool) -> Result<()> {
        if star_seen && ch == '/' {
            // End of comment
            self.buffer.pop(); // Remove the *
            self.emit_token(
                Token::Comment(self.buffer.clone()),
                start_pos,
                self.position + 1,
            );
            self.state = LexerState::Normal;
        } else {
            if ch == '*' {
                self.state = LexerState::InMultiLineComment {
                    start_pos,
                    star_seen: true,
                };
            } else {
                self.state = LexerState::InMultiLineComment {
                    start_pos,
                    star_seen: false,
                };
            }
            self.buffer.push(ch);
        }
        Ok(())
    }

    /// Process a potential comment start
    fn process_potential_comment(&mut self, ch: char, start_pos: usize) -> Result<()> {
        match ch {
            '/' => {
                // Single-line comment
                self.buffer.clear();
                self.state = LexerState::InSingleLineComment { start_pos };
            }
            '*' => {
                // Multi-line comment
                self.buffer.clear();
                self.state = LexerState::InMultiLineComment {
                    start_pos,
                    star_seen: false,
                };
            }
            _ => {
                // Not a comment, emit division operator (not supported in JSON)
                return Err(Error::UnexpectedCharacter('/', start_pos));
            }
        }
        Ok(())
    }

    /// Emit a token
    fn emit_token(&mut self, token: Token, start: usize, end: usize) {
        self.pending_tokens.push((token, Span { start, end }));
    }

    /// Emit a number token
    fn emit_number_token(&mut self, start_pos: usize) -> Result<()> {
        let token = Token::Number(self.buffer.clone());
        self.emit_token(token, start_pos, self.position);
        Ok(())
    }

    /// Emit an identifier token (could be keyword or unquoted string)
    fn emit_identifier_token(&mut self, start_pos: usize) -> Result<()> {
        let token = match self.buffer.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Null,
            _ => {
                if self.options.allow_unquoted_keys {
                    Token::UnquotedString(self.buffer.clone())
                } else {
                    return Err(Error::UnexpectedCharacter(
                        self.buffer.chars().next().unwrap(),
                        start_pos,
                    ));
                }
            }
        };
        self.emit_token(token, start_pos, self.position);
        Ok(())
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
    pub fn has_tokens(&self) -> bool {
        !self.pending_tokens.is_empty()
    }

    /// Finish lexing and emit any remaining tokens
    pub fn finish(&mut self) -> Result<()> {
        match &self.state {
            LexerState::Normal => Ok(()),
            LexerState::InString { start_pos, .. } => {
                Err(Error::UnterminatedString(*start_pos))
            }
            LexerState::InNumber { start_pos, .. } => {
                self.emit_number_token(*start_pos)
            }
            LexerState::InIdentifier { start_pos } => {
                self.emit_identifier_token(*start_pos)
            }
            LexerState::InSingleLineComment { start_pos } => {
                self.emit_token(
                    Token::Comment(self.buffer.clone()),
                    *start_pos,
                    self.position,
                );
                Ok(())
            }
            LexerState::InMultiLineComment { start_pos, .. } => {
                Err(Error::Custom(format!("Unterminated comment at position {}", start_pos), *start_pos))
            }
            LexerState::PotentialComment { start_pos } => {
                Err(Error::UnexpectedCharacter('/', *start_pos))
            }
        }
    }
}

impl Default for StreamingLexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = StreamingLexer::new();
        lexer.feed_str("{\"key\": \"value\"}").unwrap();
        lexer.finish().unwrap();

        assert_eq!(lexer.next_token().unwrap().0, Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap().0, Token::String("key".to_string()));
        assert_eq!(lexer.next_token().unwrap().0, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().0, Token::String("value".to_string()));
        assert_eq!(lexer.next_token().unwrap().0, Token::RightBrace);
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_incremental_string() {
        let mut lexer = StreamingLexer::new();
        lexer.feed_str("\"hel").unwrap();
        assert!(!lexer.has_tokens());
        
        lexer.feed_str("lo\"").unwrap();
        assert!(lexer.has_tokens());
        
        assert_eq!(lexer.next_token().unwrap().0, Token::String("hello".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = StreamingLexer::new();
        lexer.feed_str("123.45e-6").unwrap();
        lexer.finish().unwrap();

        assert_eq!(lexer.next_token().unwrap().0, Token::Number("123.45e-6".to_string()));
    }
}