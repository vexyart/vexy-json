//! Optimized parser implementation with adaptive memory pooling and branch prediction.
//!
//! This module provides a high-performance parser that uses adaptive memory pooling
//! to reduce allocation overhead and branch prediction hints for better CPU
//! pipeline utilization.

use crate::ast::{Number, Token, Value};
use crate::error::{Error, Result, Span};
use crate::lexer::{JsonLexer, Lexer};
use crate::optimization::{
    extract_string_content, parse_number_optimized, unescape_string_optimized, OptimizedMemoryPool,
    PoolStats,
};
use crate::parser::ParserOptions;
use rustc_hash::FxHashMap;

/// Branch prediction hints for hot paths
/// These are implemented as identity functions but the compiler
/// can use them for optimization hints
#[inline(always)]
#[cold]
#[allow(dead_code)]
fn cold_path() {
    // This function is marked cold to hint that it's unlikely to be called
}

#[inline(always)]
fn likely(b: bool) -> bool {
    // In release builds, this helps the compiler with branch prediction
    #[cfg(not(debug_assertions))]
    if !b {
        cold_path();
    }
    b
}

#[inline(always)]
fn unlikely(b: bool) -> bool {
    // In release builds, this helps the compiler with branch prediction
    #[cfg(not(debug_assertions))]
    if b {
        cold_path();
    }
    b
}

/// Optimized parser with adaptive memory pooling and performance enhancements.
pub struct OptimizedParserV2<'a> {
    /// Input string being parsed
    input: &'a str,
    /// Lexer for tokenization
    lexer: Lexer<'a>,
    /// Parser options
    options: ParserOptions,
    /// Optimized memory pool for allocations
    memory_pool: OptimizedMemoryPool,
    /// Current recursion depth
    depth: usize,
    /// Parser statistics
    stats: ParserStats,
}

/// Statistics collected during parsing
#[derive(Debug, Default, Clone)]
pub struct ParserStats {
    /// Number of allocations saved by pooling
    pub pooled_allocations: usize,
    /// Number of SIMD operations used
    pub simd_operations: usize,
    /// Number of branch predictions
    pub branch_predictions: usize,
    /// Number of small strings that bypassed pooling
    pub bypassed_allocations: usize,
}

impl<'a> OptimizedParserV2<'a> {
    /// Creates a new optimized parser for the given input.
    pub fn new(input: &'a str, options: ParserOptions) -> Self {
        let memory_pool = OptimizedMemoryPool::new();

        Self {
            input,
            lexer: Lexer::new(input),
            options,
            memory_pool,
            depth: 0,
            stats: ParserStats::default(),
        }
    }

    /// Creates a new optimized parser with configuration options.
    pub fn with_config(input: &'a str, options: ParserOptions, use_thread_local: bool) -> Self {
        let memory_pool = OptimizedMemoryPool::with_config(use_thread_local, true);

        Self {
            input,
            lexer: Lexer::new(input),
            options,
            memory_pool,
            depth: 0,
            stats: ParserStats::default(),
        }
    }

    /// Parses the input and returns the resulting JSON value.
    pub fn parse(&mut self) -> Result<Value> {
        let (token, span) = self.next_token()?;
        let value = self.parse_value((token, span))?;

        // Ensure we've consumed all input
        let (next_token, _) = self.peek_token()?;
        if next_token != Token::Eof {
            return Err(Error::Expected {
                expected: "end of input".to_string(),
                found: format!("{next_token:?}"),
                position: self.lexer.position(),
            });
        }

        // Analyze performance and potentially disable pooling if not beneficial
        let pool_stats = self.memory_pool.stats();
        if pool_stats.total_allocations > 100 && pool_stats.avg_allocation_size < 64 {
            // If average allocation is very small, disable pooling for better performance
            self.memory_pool.set_pooling_enabled(false);
        }

        Ok(value)
    }

    /// Returns parser statistics
    pub fn stats(&self) -> &ParserStats {
        &self.stats
    }

    /// Returns memory pool statistics
    pub fn memory_stats(&self) -> PoolStats {
        self.memory_pool.stats()
    }

    /// Gets the next token from the lexer
    #[inline]
    fn next_token(&mut self) -> Result<(Token, Span)> {
        self.lexer.next_token_with_span()
    }

    /// Peeks at the next token without consuming it
    #[inline]
    fn peek_token(&mut self) -> Result<(Token, Span)> {
        self.lexer
            .peek_with_span()
            .map(|&(token, span)| (token, span))
    }

    /// Parses a value based on the current token with branch prediction
    fn parse_value(&mut self, token: (Token, Span)) -> Result<Value> {
        self.stats.branch_predictions += 1;

        match token.0 {
            // Most common cases first for better branch prediction
            Token::String => {
                if likely(
                    !self.options.allow_single_quotes
                        || self.input.chars().nth(token.1.start).unwrap() == '"',
                ) {
                    self.parse_string_pooled(token.1)
                } else {
                    self.parse_string_pooled(token.1)
                }
            }
            Token::Number => self.parse_number_optimized(token.1),
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::True => Ok(Value::Bool(true)),
            Token::False => Ok(Value::Bool(false)),
            Token::Null => Ok(Value::Null),
            _ => Err(Error::UnexpectedChar(
                self.input.chars().nth(token.1.start).unwrap_or('\0'),
                token.1.start,
            )),
        }
    }

    /// Parses a string using the optimized memory pool
    fn parse_string_pooled(&mut self, span: Span) -> Result<Value> {
        let string_slice = &self.input[span.start..span.end];

        // Extract string content
        let content = extract_string_content(string_slice)
            .map_err(|_| Error::UnterminatedString(span.start))?;

        // Check if unescaping is needed
        if likely(!content.contains('\\')) {
            // Fast path: no escaping needed, use memory pool
            if let Some(pooled_str) = self.memory_pool.allocate_str(content) {
                let pool_stats = self.memory_pool.stats();
                if pool_stats.pooled_allocations > self.stats.pooled_allocations {
                    self.stats.pooled_allocations = pool_stats.pooled_allocations;
                } else {
                    self.stats.bypassed_allocations += 1;
                }
                Ok(Value::String(pooled_str.to_string()))
            } else {
                Ok(Value::String(content.to_string()))
            }
        } else {
            // Slow path: unescape the string
            let unescaped = unescape_string_optimized(content).map_err(|e| match e {
                Error::InvalidEscape(_) => Error::InvalidEscape(span.start),
                Error::InvalidUnicode(_) => Error::InvalidUnicode(span.start),
                other => other,
            })?;

            Ok(Value::String(unescaped))
        }
    }

    /// Parses a number using optimized routines
    fn parse_number_optimized(&mut self, span: Span) -> Result<Value> {
        let number_str = &self.input[span.start..span.end];

        // Use optimized number parsing
        match parse_number_optimized(number_str) {
            Ok(value) => Ok(Value::Number(Number::Float(value))),
            Err(_) => Err(Error::InvalidNumber(span.start)),
        }
    }

    /// Parses an object with optimized string handling
    fn parse_object(&mut self) -> Result<Value> {
        // Check recursion depth
        self.depth += 1;
        if unlikely(self.depth > self.options.max_depth) {
            return Err(Error::DepthLimitExceeded(0));
        }

        let mut object = FxHashMap::default();
        let mut first = true;

        loop {
            // Skip newlines
            loop {
                let (next_token, _) = self.peek_token()?;
                if next_token == Token::Newline {
                    self.next_token()?;
                } else {
                    break;
                }
            }

            // Check for end of object
            let (next_token, _) = self.peek_token()?;
            if next_token == Token::RightBrace {
                self.next_token()?;
                break;
            }

            // Handle comma between elements
            if !first {
                let (token, span) = self.next_token()?;
                match token {
                    Token::Comma => {}
                    Token::Newline if self.options.newline_as_comma => {}
                    Token::RightBrace if self.options.allow_trailing_commas => break,
                    _ => {
                        return Err(Error::Expected {
                            expected: "comma or }".to_string(),
                            found: format!("{token:?}"),
                            position: span.start,
                        });
                    }
                }
            }
            first = false;

            // Skip newlines after comma
            loop {
                let (next_token, _) = self.peek_token()?;
                if next_token == Token::Newline {
                    self.next_token()?;
                } else {
                    break;
                }
            }

            // Check for trailing comma before closing brace
            let (next_token, _) = self.peek_token()?;
            if next_token == Token::RightBrace && self.options.allow_trailing_commas {
                self.next_token()?;
                break;
            }

            // Parse key
            let (key_token, key_span) = self.next_token()?;
            let key = match key_token {
                Token::String => self.parse_string_key(key_span)?,
                Token::UnquotedString if self.options.allow_unquoted_keys => {
                    // Extract unquoted key directly from input
                    let key_str = &self.input[key_span.start..key_span.end];
                    key_str.to_string()
                }
                _ => {
                    return Err(Error::Expected {
                        expected: "string key".to_string(),
                        found: format!("{key_token:?}"),
                        position: key_span.start,
                    });
                }
            };

            // Expect colon
            let (colon_token, colon_span) = self.next_token()?;
            match colon_token {
                Token::Colon => {}
                _ => {
                    return Err(Error::Expected {
                        expected: "colon".to_string(),
                        found: format!("{colon_token:?}"),
                        position: colon_span.start,
                    });
                }
            }

            // Parse value
            let value_token = self.next_token()?;
            let value = self.parse_value(value_token)?;

            object.insert(key, value);
        }

        self.depth -= 1;
        Ok(Value::Object(object))
    }

    /// Parses a string key using the optimized memory pool
    fn parse_string_key(&mut self, span: Span) -> Result<String> {
        let string_slice = &self.input[span.start..span.end];
        let content = extract_string_content(string_slice)
            .map_err(|_| Error::UnterminatedString(span.start))?;

        // Keys often repeat, so pooling is very effective
        // Use a lower threshold for keys
        if content.len() >= 4 {
            // Pool even small keys
            if let Some(pooled_str) = self.memory_pool.allocate_str(content) {
                let pool_stats = self.memory_pool.stats();
                if pool_stats.pooled_allocations > self.stats.pooled_allocations {
                    self.stats.pooled_allocations = pool_stats.pooled_allocations;
                }
                Ok(pooled_str.to_string())
            } else {
                Ok(content.to_string())
            }
        } else {
            self.stats.bypassed_allocations += 1;
            Ok(content.to_string())
        }
    }

    /// Parses an array with optimized value handling
    fn parse_array(&mut self) -> Result<Value> {
        // Check recursion depth
        self.depth += 1;
        if unlikely(self.depth > self.options.max_depth) {
            return Err(Error::DepthLimitExceeded(0));
        }

        let mut array = Vec::new();
        let mut first = true;

        loop {
            // Skip newlines
            loop {
                let (next_token, _) = self.peek_token()?;
                if next_token == Token::Newline {
                    self.next_token()?;
                } else {
                    break;
                }
            }

            // Check for end of array
            let (next_token, _) = self.peek_token()?;
            if next_token == Token::RightBracket {
                self.next_token()?;
                break;
            }

            // Handle comma between elements
            if !first {
                let (token, span) = self.next_token()?;
                match token {
                    Token::Comma => {}
                    Token::Newline if self.options.newline_as_comma => {}
                    Token::RightBracket if self.options.allow_trailing_commas => break,
                    _ => {
                        return Err(Error::Expected {
                            expected: "comma or ]".to_string(),
                            found: format!("{token:?}"),
                            position: span.start,
                        });
                    }
                }
            }
            first = false;

            // Skip newlines after comma
            loop {
                let (next_token, _) = self.peek_token()?;
                if next_token == Token::Newline {
                    self.next_token()?;
                } else {
                    break;
                }
            }

            // Check for trailing comma before closing bracket
            let (next_token, _) = self.peek_token()?;
            if next_token == Token::RightBracket && self.options.allow_trailing_commas {
                self.next_token()?;
                break;
            }

            // Parse value
            let value_token = self.next_token()?;
            let value = self.parse_value(value_token)?;
            array.push(value);
        }

        self.depth -= 1;
        Ok(Value::Array(array))
    }
}

/// Parses JSON with optimized memory pool v2
pub fn parse_optimized_v2(input: &str) -> Result<Value> {
    let mut parser = OptimizedParserV2::new(input, ParserOptions::default());
    parser.parse()
}

/// Parses JSON with custom options and optimized memory pool v2
pub fn parse_optimized_v2_with_options(input: &str, options: ParserOptions) -> Result<Value> {
    let mut parser = OptimizedParserV2::new(input, options);
    parser.parse()
}

/// Parses and returns both the value and performance statistics
pub fn parse_v2_with_stats(input: &str) -> Result<(Value, ParserStats, PoolStats)> {
    let mut parser = OptimizedParserV2::new(input, ParserOptions::default());
    let value = parser.parse()?;
    let stats = parser.stats().clone();
    let memory_stats = parser.memory_stats();
    Ok((value, stats, memory_stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_parser_v2_simple() {
        let input = r#"{"name": "test", "value": 42}"#;
        let value = parse_optimized_v2(input).unwrap();

        match value {
            Value::Object(obj) => {
                assert_eq!(obj.get("name"), Some(&Value::String("test".to_string())));
                assert_eq!(obj.get("value"), Some(&Value::Number(Number::Float(42.0))));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parser_v2_with_stats() {
        let input = r#"{"items": ["a", "b", "c"], "count": 3}"#;
        let (value, stats, memory_stats) = parse_v2_with_stats(input).unwrap();

        // Should have some allocations
        assert!(stats.pooled_allocations > 0 || stats.bypassed_allocations > 0);
        assert!(memory_stats.total_bytes > 0);

        // Verify the parsed value
        match value {
            Value::Object(obj) => {
                assert!(obj.contains_key("items"));
                assert!(obj.contains_key("count"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_adaptive_pooling() {
        // Test with many small strings
        let mut small_items = Vec::new();
        for i in 0..50 {
            small_items.push(format!("\"x{i}\""));
        }
        let input = format!("[{}]", small_items.join(","));

        let (_, stats, memory_stats) = parse_v2_with_stats(&input).unwrap();

        // Most small strings should bypass pooling
        assert!(stats.bypassed_allocations > 0);

        // Average allocation size should be small
        if memory_stats.total_allocations > 0 {
            assert!(memory_stats.avg_allocation_size < 100);
        }
    }

    #[test]
    fn test_branch_prediction_stats() {
        let input = r#"[1, 2, 3, 4, 5]"#;
        let (_, stats, _) = parse_v2_with_stats(input).unwrap();

        // Should have branch predictions for each value
        assert!(stats.branch_predictions >= 5);
    }
}
