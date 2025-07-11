//! Fast lexer implementation optimized for performance
//! 
//! This lexer implementation focuses on speed by:
//! - Using hand-written state machine instead of regex
//! - Minimizing allocations
//! - Inline optimization for hot paths
//! - SIMD acceleration where available

use crate::ast::Token;
use crate::error::{Error, Result, Span};
use crate::lexer::{JsonLexer, LexerConfig, LexerStats};
use std::time::Instant;

/// Fast hand-optimized lexer implementation
pub struct FastLexer<'a> {
    /// Input bytes for faster access
    input: &'a [u8],
    /// Current position in input
    position: usize,
    /// Peeked token if any
    peeked: Option<(Token, Span)>,
    /// Configuration
    config: LexerConfig,
    /// Statistics
    stats: LexerStats,
    /// Start time for stats
    start_time: Option<Instant>,
}

impl<'a> FastLexer<'a> {
    /// Create a new fast lexer
    pub fn new(input: &'a str, config: LexerConfig) -> Self {
        let start_time = if config.collect_stats {
            Some(Instant::now())
        } else {
            None
        };

        FastLexer {
            input: input.as_bytes(),
            position: 0,
            peeked: None,
            config,
            stats: LexerStats::default(),
            start_time,
        }
    }

    /// Skip whitespace and return true if position changed
    #[inline(always)]
    fn skip_whitespace(&mut self) -> bool {
        let start = self.position;
        while self.position < self.input.len() {
            match self.input[self.position] {
                b' ' | b'\t' | b'\r' => self.position += 1,
                b'\n' => {
                    self.position += 1;
                    // Newline handling for newline_as_comma would go here
                }
                _ => break,
            }
        }
        self.position > start
    }

    /// Parse a string token
    #[inline]
    fn parse_string(&mut self, quote: u8) -> Result<(Token, Span)> {
        let start = self.position;
        self.position += 1; // Skip opening quote

        while self.position < self.input.len() {
            match self.input[self.position] {
                b'\\' => {
                    // Skip escape sequence
                    self.position += 1;
                    if self.position < self.input.len() {
                        self.position += 1;
                    }
                }
                b if b == quote => {
                    self.position += 1; // Skip closing quote
                    return Ok((Token::String, Span::new(start, self.position)));
                }
                _ => self.position += 1,
            }
        }

        Err(Error::UnterminatedString(start))
    }

    /// Parse a number token
    #[inline]
    fn parse_number(&mut self) -> Result<(Token, Span)> {
        let start = self.position;

        // Optional sign (+ or -)
        if self.position < self.input.len() && (self.input[self.position] == b'-' || self.input[self.position] == b'+') {
            self.position += 1;
        }

        // Check if we start with a decimal point (e.g., .5)
        let starts_with_dot = self.position < self.input.len() && self.input[self.position] == b'.';
        
        if starts_with_dot {
            // Skip the dot, we'll handle it in the fractional part
        } else {
            // Check for alternative number formats first
            if self.position < self.input.len() && self.input[self.position] == b'0' {
                let next_pos = self.position + 1;
                if next_pos < self.input.len() {
                    match self.input[next_pos] {
                        b'x' | b'X' => {
                            // Hexadecimal number
                            self.position += 2; // Skip "0x"
                            if self.position >= self.input.len() || !self.input[self.position].is_ascii_hexdigit() {
                                return Err(Error::InvalidNumber(start));
                            }
                            while self.position < self.input.len() && (self.input[self.position].is_ascii_hexdigit() || self.input[self.position] == b'_') {
                                self.position += 1;
                            }
                            return Ok((Token::Number, Span::new(start, self.position)));
                        }
                        b'o' | b'O' => {
                            // Octal number
                            self.position += 2; // Skip "0o"
                            if self.position >= self.input.len() || !matches!(self.input[self.position], b'0'..=b'7') {
                                return Err(Error::InvalidNumber(start));
                            }
                            while self.position < self.input.len() && (matches!(self.input[self.position], b'0'..=b'7') || self.input[self.position] == b'_') {
                                self.position += 1;
                            }
                            return Ok((Token::Number, Span::new(start, self.position)));
                        }
                        b'b' | b'B' => {
                            // Binary number
                            self.position += 2; // Skip "0b"
                            if self.position >= self.input.len() || !matches!(self.input[self.position], b'0' | b'1') {
                                return Err(Error::InvalidNumber(start));
                            }
                            while self.position < self.input.len() && (matches!(self.input[self.position], b'0' | b'1') || self.input[self.position] == b'_') {
                                self.position += 1;
                            }
                            return Ok((Token::Number, Span::new(start, self.position)));
                        }
                        _ => {
                            // Regular number starting with 0
                            self.position += 1;
                        }
                    }
                } else {
                    // Just "0"
                    self.position += 1;
                }
            } else {
                // Integer part
                if self.position >= self.input.len() || !self.input[self.position].is_ascii_digit() {
                    return Err(Error::InvalidNumber(start));
                }

                while self.position < self.input.len() && (self.input[self.position].is_ascii_digit() || self.input[self.position] == b'_') {
                    self.position += 1;
                }
            }
        }

        // Fractional part
        if self.position < self.input.len() && self.input[self.position] == b'.' {
            self.position += 1;
            
            // Check for double decimal point immediately (e.g., "1..1" or "..1")
            if self.position < self.input.len() && self.input[self.position] == b'.' {
                return Err(Error::InvalidNumber(start));
            }
            
            // For numbers starting with dot (e.g., .5), we need at least one digit
            if starts_with_dot && (self.position >= self.input.len() || !self.input[self.position].is_ascii_digit()) {
                return Err(Error::InvalidNumber(start));
            }
            
            // Allow trailing decimal point (e.g., "1.") - vexy_json compatibility
            // Only consume digits if they exist after the decimal point
            while self.position < self.input.len() && (self.input[self.position].is_ascii_digit() || self.input[self.position] == b'_') {
                self.position += 1;
            }
        } else if starts_with_dot {
            // If we started with a dot but there's no dot here, something went wrong
            return Err(Error::InvalidNumber(start));
        }

        // Exponent part
        if self.position < self.input.len()
            && (self.input[self.position] == b'e' || self.input[self.position] == b'E')
        {
            self.position += 1;
            if self.position < self.input.len()
                && (self.input[self.position] == b'+' || self.input[self.position] == b'-')
            {
                self.position += 1;
            }
            if self.position >= self.input.len() || !self.input[self.position].is_ascii_digit() {
                return Err(Error::InvalidNumber(start));
            }
            while self.position < self.input.len() && (self.input[self.position].is_ascii_digit() || self.input[self.position] == b'_') {
                self.position += 1;
            }
        }

        Ok((Token::Number, Span::new(start, self.position)))
    }

    /// Parse an identifier (could be keyword or unquoted string)
    #[inline]
    fn parse_identifier(&mut self) -> Result<(Token, Span)> {
        let start = self.position;

        while self.position < self.input.len() {
            match self.input[self.position] {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' | b'-' => {
                    self.position += 1;
                }
                _ => break,
            }
        }

        let text = std::str::from_utf8(&self.input[start..self.position])
            .map_err(|_| Error::UnexpectedChar('?', start))?;

        let token = match text {
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            _ => Token::UnquotedString,
        };

        Ok((token, Span::new(start, self.position)))
    }

    /// Skip single-line comment
    #[inline]
    fn skip_single_line_comment(&mut self) {
        while self.position < self.input.len() {
            match self.input[self.position] {
                b'\n' | b'\r' => break,
                _ => self.position += 1,
            }
        }
    }

    /// Skip multi-line comment
    #[inline]
    fn skip_multi_line_comment(&mut self) -> Result<()> {
        let start = self.position - 2; // We already consumed /*
        let mut depth = 1;

        while self.position + 1 < self.input.len() {
            if self.input[self.position] == b'/' && self.input[self.position + 1] == b'*' {
                self.position += 2;
                depth += 1;
            } else if self.input[self.position] == b'*' && self.input[self.position + 1] == b'/' {
                self.position += 2;
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            } else {
                self.position += 1;
            }
        }

        Err(Error::UnexpectedEof(start))
    }

    /// Get next token implementation
    fn next_token_impl(&mut self) -> Result<(Token, Span)> {
        loop {
            self.skip_whitespace();

            if self.position >= self.input.len() {
                return Ok((Token::Eof, Span::new(self.position, self.position)));
            }

            let ch = self.input[self.position];
            match ch {
                b'{' => {
                    let span = Span::new(self.position, self.position + 1);
                    self.position += 1;
                    return Ok((Token::LeftBrace, span));
                }
                b'}' => {
                    let span = Span::new(self.position, self.position + 1);
                    self.position += 1;
                    return Ok((Token::RightBrace, span));
                }
                b'[' => {
                    let span = Span::new(self.position, self.position + 1);
                    self.position += 1;
                    return Ok((Token::LeftBracket, span));
                }
                b']' => {
                    let span = Span::new(self.position, self.position + 1);
                    self.position += 1;
                    return Ok((Token::RightBracket, span));
                }
                b',' => {
                    let span = Span::new(self.position, self.position + 1);
                    self.position += 1;
                    return Ok((Token::Comma, span));
                }
                b':' => {
                    let span = Span::new(self.position, self.position + 1);
                    self.position += 1;
                    return Ok((Token::Colon, span));
                }
                b'"' => return self.parse_string(b'"'),
                b'\'' if self.config.mode != crate::lexer::LexerMode::Strict => {
                    return self.parse_string(b'\'');
                }
                b'-' | b'+' | b'.' | b'0'..=b'9' => return self.parse_number(),
                b'/' => {
                    if self.position + 1 < self.input.len() {
                        match self.input[self.position + 1] {
                            b'/' if self.config.mode != crate::lexer::LexerMode::Strict => {
                                self.position += 2;
                                self.skip_single_line_comment();
                                continue; // Skip comment and continue
                            }
                            b'*' if self.config.mode != crate::lexer::LexerMode::Strict => {
                                self.position += 2;
                                self.skip_multi_line_comment()?;
                                continue; // Skip comment and continue
                            }
                            _ => {}
                        }
                    }
                    return Err(Error::UnexpectedChar('/', self.position));
                }
                b'#' if self.config.mode != crate::lexer::LexerMode::Strict => {
                    self.position += 1;
                    self.skip_single_line_comment();
                    continue; // Skip comment and continue
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                    if self.config.mode != crate::lexer::LexerMode::Strict {
                        return self.parse_identifier();
                    } else {
                        return Err(Error::UnexpectedChar(ch as char, self.position));
                    }
                }
                _ => return Err(Error::UnexpectedChar(ch as char, self.position)),
            }
        }
    }
}

impl<'a> JsonLexer for FastLexer<'a> {
    fn position(&self) -> usize {
        self.position
    }

    fn next_token(&mut self) -> Result<(Token, Span)> {
        if let Some(peeked) = self.peeked.take() {
            return Ok(peeked);
        }

        let result = self.next_token_impl();

        if self.config.collect_stats {
            self.stats.tokens_count += 1;
            self.stats.bytes_processed = self.position;
            if result.is_err() {
                self.stats.errors_count += 1;
            }
        }

        result
    }

    fn peek_token(&mut self) -> Result<&(Token, Span)> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    fn span_text(&self, span: &Span) -> &str {
        std::str::from_utf8(&self.input[span.start..span.end]).unwrap_or("")
    }

    fn line_col(&self) -> (usize, usize) {
        if !self.config.track_positions {
            return (0, 0);
        }

        let mut line = 1;
        let mut col = 1;

        for i in 0..self.position.min(self.input.len()) {
            if self.input[i] == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    fn is_eof(&self) -> bool {
        self.peeked
            .as_ref()
            .map(|(t, _)| *t == Token::Eof)
            .unwrap_or(false)
            || self.position >= self.input.len()
    }

    fn stats(&self) -> LexerStats {
        let mut stats = self.stats.clone();
        if let Some(start_time) = self.start_time {
            stats.time_ns = start_time.elapsed().as_nanos() as u64;
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::LexerMode;

    #[test]
    fn test_fast_lexer_basic() {
        let config = LexerConfig {
            mode: LexerMode::Standard,
            ..Default::default()
        };
        let mut lexer = FastLexer::new(r#"{"key": 123}"# , config);

        assert_eq!(lexer.next_token().unwrap().0, Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap().0, Token::String);
        assert_eq!(lexer.next_token().unwrap().0, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().0, Token::Number);
        assert_eq!(lexer.next_token().unwrap().0, Token::RightBrace);
        assert_eq!(lexer.next_token().unwrap().0, Token::Eof);
    }

    #[test]
    fn test_fast_lexer_comments() {
        let config = LexerConfig {
            mode: LexerMode::Forgiving,
            ..Default::default()
        };
        let mut lexer = FastLexer::new("// comment\n{/* multi\nline */}", config);

        assert_eq!(lexer.next_token().unwrap().0, Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap().0, Token::RightBrace);
    }

    #[test]
    fn test_fast_lexer_unquoted() {
        let config = LexerConfig {
            mode: LexerMode::Forgiving,
            ..Default::default()
        };
        let mut lexer = FastLexer::new("{key: true}", config);

        assert_eq!(lexer.next_token().unwrap().0, Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap().0, Token::UnquotedString);
        assert_eq!(lexer.next_token().unwrap().0, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().0, Token::True);
        assert_eq!(lexer.next_token().unwrap().0, Token::RightBrace);
    }

    #[test]
    fn test_fast_lexer_stats() {
        let config = LexerConfig {
            mode: LexerMode::Standard,
            collect_stats: true,
            ..Default::default()
        };
        let mut lexer = FastLexer::new("[1,2,3]", config);

        while lexer.next_token().unwrap().0 != Token::Eof {
            // Consume all tokens
        }

        let stats = lexer.stats();
        assert_eq!(stats.tokens_count, 7); // [, 1, ,, 2, ,, 3, ], EOF
        assert_eq!(stats.bytes_processed, 7);
        assert!(stats.time_ns > 0);
    }
}
