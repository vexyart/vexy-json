//! Logos-based lexer implementation
//!
//! This is the existing lexer implementation using the logos crate,
//! adapted to implement the JsonLexer trait.

use crate::ast::Token;
use crate::error::{Error, Result, Span};
use crate::lexer::JsonLexer;
use logos::Logos;

/// Logos-based lexer implementation
pub struct LogosLexer<'a> {
    lexer: logos::Lexer<'a, Token>,
    input: &'a str,
    peeked: Option<(Token, Span)>,
    line: usize,
    column: usize,
    last_newline_pos: usize,
}

impl<'a> LogosLexer<'a> {
    /// Creates a new logos-based lexer
    pub fn new(input: &'a str) -> Self {
        LogosLexer {
            lexer: Token::lexer(input),
            input,
            peeked: None,
            line: 1,
            column: 1,
            last_newline_pos: 0,
        }
    }

    fn update_position(&mut self, span: &Span) {
        let text = &self.input[self.last_newline_pos..span.start];
        for ch in text.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.last_newline_pos = span.start;
            } else {
                self.column += 1;
            }
        }
    }

    // Backward compatibility methods

    /// Get next token with span (backward compatibility)
    pub fn next_token_with_span(&mut self) -> Result<(Token, Span)> {
        self.next_token()
    }

    /// Peek at next token with span (backward compatibility)
    pub fn peek_with_span(&mut self) -> Result<&(Token, Span)> {
        self.peek_token()
    }

    /// Get current span (backward compatibility)
    pub fn span(&self) -> logos::Span {
        self.lexer.span()
    }
}

impl<'a> JsonLexer for LogosLexer<'a> {
    fn position(&self) -> usize {
        self.lexer.span().start
    }

    fn next_token(&mut self) -> Result<(Token, Span)> {
        if let Some(peeked) = self.peeked.take() {
            return Ok(peeked);
        }

        match self.lexer.next() {
            Some(token_result) => {
                let logos_span = self.lexer.span();
                let span = Span::new(logos_span.start, logos_span.end);
                self.update_position(&span);

                match token_result {
                    Ok(token) => Ok((token, span)),
                    Err(_) => Err(Error::UnexpectedChar(
                        self.lexer.slice().chars().next().unwrap_or(' '),
                        span.start,
                    )),
                }
            }
            None => {
                let pos = self.lexer.span().start;
                Ok((Token::Eof, Span::new(pos, pos)))
            }
        }
    }

    fn peek_token(&mut self) -> Result<&(Token, Span)> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    fn span_text(&self, span: &Span) -> &str {
        &self.input[span.start..span.end]
    }

    fn line_col(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    fn is_eof(&self) -> bool {
        self.peeked
            .as_ref()
            .map(|(t, _)| *t == Token::Eof)
            .unwrap_or(false)
            || self.lexer.remainder().is_empty()
    }
}

/// Type alias for backward compatibility with the original Lexer API.
pub type Lexer<'a> = LogosLexer<'a>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logos_lexer() {
        let input = r#"{"key": "value"}"#;
        let mut lexer = LogosLexer::new(input);

        // Test first token
        let (token, span) = lexer.next_token().unwrap();
        assert_eq!(token, Token::LeftBrace);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 1);

        // Test peek
        let peeked_token = lexer.peek_token().unwrap().0;
        assert_eq!(peeked_token, Token::String);

        // Peek again should return same token
        let peeked_token2 = lexer.peek_token().unwrap().0;
        assert_eq!(peeked_token, peeked_token2);

        // Next should consume the peeked token
        let (token, _) = lexer.next_token().unwrap();
        assert_eq!(token, Token::String);
    }

    #[test]
    fn test_line_col_tracking() {
        let input = "{\n  \"key\": \"value\"\n}";
        let mut lexer = LogosLexer::new(input);

        // First token
        lexer.next_token().unwrap();
        assert_eq!(lexer.line_col(), (1, 1));

        // After newline
        lexer.next_token().unwrap(); // Newline
        lexer.next_token().unwrap(); // String
        let (line, _col) = lexer.line_col();
        assert_eq!(line, 2);
    }
}
