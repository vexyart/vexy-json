// this_file: crates/core/src/streaming/buffered/lexer.rs

//! Buffered lexer for streaming JSON parsing
//!
//! This lexer can handle incremental input and properly tokenize across buffer boundaries.

use crate::ast::Token;
use crate::error::{Error, Result, Span};
use crate::lexer::{LexerConfig, LexerMode};

/// State of the lexer for resumable parsing
#[derive(Debug, Clone)]
pub enum LexerState {
    /// Normal state, expecting any token
    Normal,
    /// Inside a string, with quote character
    InString { quote: char, escape_next: bool },
    /// Inside a single-line comment
    InSingleLineComment,
    /// Inside a multi-line comment
    InMultiLineComment { star_seen: bool },
    /// Building a number
    InNumber,
    /// Building an identifier (for keywords and unquoted keys)
    InIdentifier,
}

/// Buffered lexer that can handle incremental input
pub struct BufferedLexer {
    /// Current state for resumable parsing
    state: LexerState,
    /// Buffer for the current token being built
    token_buffer: String,
    /// Position in the overall input stream
    position: usize,
    /// Start position of current token
    token_start: usize,
    /// Configuration
    config: LexerConfig,
}

impl BufferedLexer {
    /// Create a new buffered lexer
    pub fn new(config: LexerConfig) -> Self {
        Self {
            state: LexerState::Normal,
            token_buffer: String::new(),
            position: 0,
            token_start: 0,
            config,
        }
    }

    /// Feed a chunk of input to the lexer
    /// Returns tokens that were completed and whether more input is needed
    pub fn feed(&mut self, input: &str) -> Result<(Vec<(Token, Span)>, bool)> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match &self.state {
                LexerState::Normal => {
                    match ch {
                        // Whitespace
                        ' ' | '\t' | '\r' => {
                            // Skip whitespace
                        }
                        '\n' => {
                            tokens.push((Token::Newline, self.make_span(1)));
                        }
                        // Structural characters
                        '{' => tokens.push((Token::LeftBrace, self.make_span(1))),
                        '}' => tokens.push((Token::RightBrace, self.make_span(1))),
                        '[' => tokens.push((Token::LeftBracket, self.make_span(1))),
                        ']' => tokens.push((Token::RightBracket, self.make_span(1))),
                        ',' => tokens.push((Token::Comma, self.make_span(1))),
                        ':' => tokens.push((Token::Colon, self.make_span(1))),
                        
                        // Strings
                        '"' | '\'' => {
                            self.state = LexerState::InString { quote: ch, escape_next: false };
                            self.token_start = self.position;
                            self.token_buffer.clear();
                        }
                        
                        // Comments
                        '/' if self.config.mode == LexerMode::Forgiving => {
                            if let Some(&next_ch) = chars.peek() {
                                match next_ch {
                                    '/' => {
                                        chars.next(); // consume second /
                                        self.position += 1;
                                        self.state = LexerState::InSingleLineComment;
                                    }
                                    '*' => {
                                        chars.next(); // consume *
                                        self.position += 1;
                                        self.state = LexerState::InMultiLineComment { star_seen: false };
                                    }
                                    _ => {
                                        // Not a comment, treat as error
                                        return Err(Error::UnexpectedChar('/', self.position));
                                    }
                                }
                            } else {
                                // Need more input to determine if it's a comment
                                self.token_buffer.push(ch);
                                self.token_start = self.position;
                                self.position += 1;
                                return Ok((tokens, true));
                            }
                        }
                        
                        // Numbers
                        '-' | '0'..='9' => {
                            self.state = LexerState::InNumber;
                            self.token_start = self.position;
                            self.token_buffer.clear();
                            self.token_buffer.push(ch);
                        }
                        
                        // Identifiers (null, true, false, unquoted keys)
                        'a'..='z' | 'A'..='Z' | '_' => {
                            self.state = LexerState::InIdentifier;
                            self.token_start = self.position;
                            self.token_buffer.clear();
                            self.token_buffer.push(ch);
                        }
                        
                        _ => {
                            return Err(Error::UnexpectedChar(ch, self.position));
                        }
                    }
                }
                
                LexerState::InString { quote, escape_next } => {
                    if *escape_next {
                        // Just add the escaped character
                        self.token_buffer.push(ch);
                        self.state = LexerState::InString { quote: *quote, escape_next: false };
                    } else if ch == '\\' {
                        self.token_buffer.push(ch);
                        self.state = LexerState::InString { quote: *quote, escape_next: true };
                    } else if ch == *quote {
                        // String complete - increment position first to include the closing quote
                        self.position += 1;
                        tokens.push((Token::String, self.make_span_from_start()));
                        self.state = LexerState::Normal;
                        self.token_buffer.clear();
                        continue; // Skip the normal position increment
                    } else {
                        self.token_buffer.push(ch);
                    }
                }
                
                LexerState::InSingleLineComment => {
                    if ch == '\n' {
                        self.state = LexerState::Normal;
                        tokens.push((Token::Newline, self.make_span(1)));
                    }
                    // Otherwise, ignore characters in comment
                }
                
                LexerState::InMultiLineComment { star_seen } => {
                    if *star_seen && ch == '/' {
                        // End of multi-line comment
                        self.state = LexerState::Normal;
                    } else {
                        self.state = LexerState::InMultiLineComment { star_seen: ch == '*' };
                    }
                }
                
                LexerState::InNumber => {
                    if ch.is_numeric() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-' {
                        self.token_buffer.push(ch);
                    } else {
                        // Number complete
                        tokens.push((Token::Number, self.make_span_from_start()));
                        self.state = LexerState::Normal;
                        self.token_buffer.clear();
                        
                        // Reprocess this character
                        self.position -= 1;
                        continue;
                    }
                }
                
                LexerState::InIdentifier => {
                    if ch.is_alphanumeric() || ch == '_' {
                        self.token_buffer.push(ch);
                    } else {
                        // Identifier complete
                        let token = match self.token_buffer.as_str() {
                            "null" => Token::Null,
                            "true" => Token::True,
                            "false" => Token::False,
                            _ => {
                                // Unquoted key or error
                                if self.config.mode == LexerMode::Forgiving {
                                    Token::UnquotedString
                                } else {
                                    return Err(Error::UnexpectedChar(
                                        self.token_buffer.chars().next().unwrap_or(' '),
                                        self.token_start
                                    ));
                                }
                            }
                        };
                        tokens.push((token, self.make_span_from_start()));
                        self.state = LexerState::Normal;
                        self.token_buffer.clear();
                        
                        // Reprocess this character
                        self.position -= 1;
                        continue;
                    }
                }
            }
            
            self.position += 1;
        }
        
        // Check if we need more input
        let needs_more = !matches!(self.state, LexerState::Normal);
        
        Ok((tokens, needs_more))
    }
    
    /// Flush any remaining tokens (call at end of input)
    pub fn flush(&mut self) -> Result<Vec<(Token, Span)>> {
        match &self.state {
            LexerState::Normal => Ok(vec![]),
            LexerState::InString { .. } => Err(Error::UnterminatedString(self.token_start)),
            LexerState::InSingleLineComment => {
                self.state = LexerState::Normal;
                Ok(vec![])
            }
            LexerState::InMultiLineComment { .. } => {
                Err(Error::UnexpectedEof(self.position))
            }
            LexerState::InNumber => {
                let token = vec![(Token::Number, self.make_span_from_start())];
                self.state = LexerState::Normal;
                self.token_buffer.clear();
                Ok(token)
            }
            LexerState::InIdentifier => {
                let token = match self.token_buffer.as_str() {
                    "null" => Token::Null,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => {
                        if self.config.mode == LexerMode::Forgiving {
                            Token::UnquotedString
                        } else {
                            return Err(Error::UnexpectedChar(
                                self.token_buffer.chars().next().unwrap_or(' '),
                                self.token_start
                            ));
                        }
                    }
                };
                let tokens = vec![(token, self.make_span_from_start())];
                self.state = LexerState::Normal;
                self.token_buffer.clear();
                Ok(tokens)
            }
        }
    }
    
    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Create a span for a single character token
    fn make_span(&self, len: usize) -> Span {
        Span {
            start: self.position,
            end: self.position + len,
        }
    }
    
    /// Create a span from token start to current position
    fn make_span_from_start(&self) -> Span {
        Span {
            start: self.token_start,
            end: self.position,
        }
    }
}