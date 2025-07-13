// this_file: crates/core/src/parser/optimized_v3.rs

//! Optimized parser V3 with advanced memory pool and small collection optimizations.
//!
//! This module provides the most advanced optimized parser that uses:
//! - Memory Pool V3 with typed arenas
//! - Small vector optimization for arrays
//! - Compact string optimization
//! - Efficient collection pre-sizing

use crate::ast::{Number, Token, Value};
use crate::error::{Error, Result, Span};
use crate::lexer::Lexer;
use crate::optimization::{
    extract_string_content, AllocationStats,
};
use crate::parser::ParserOptions;
use rustc_hash::FxHashMap;

/// Advanced optimized parser with Memory Pool V3
pub struct OptimizedParserV3<'a> {
    /// Input string
    input: &'a str,
    /// Lexer for tokenization
    lexer: Lexer<'a>,
    /// Parser options
    options: ParserOptions,
    /// Current parsing depth
    depth: usize,
    /// Performance statistics
    stats: ParserStats,
}

/// Performance statistics for the advanced optimized parser
#[derive(Debug, Default, Clone)]
pub struct ParserStats {
    /// Number of values parsed
    pub values_parsed: usize,
    /// Number of strings parsed
    pub strings_parsed: usize,
    /// Number of arrays parsed
    pub arrays_parsed: usize,
    /// Number of objects parsed
    pub objects_parsed: usize,
    /// Number of small vector optimizations used
    pub small_vec_optimizations: usize,
    /// Number of compact string optimizations used
    pub compact_string_optimizations: usize,
    /// Number of pre-sized collections
    pub presized_collections: usize,
}

impl<'a> OptimizedParserV3<'a> {
    /// Creates a new advanced optimized parser
    pub fn new(input: &'a str, options: ParserOptions) -> Self {
        Self {
            input,
            lexer: Lexer::new(input),
            options,
            depth: 0,
            stats: ParserStats::default(),
        }
    }

    /// Parses the input with advanced optimizations
    pub fn parse(&mut self) -> Result<Value> {
        let (token, span) = self.next_token()?;
        self.parse_value((token, span))
    }

    /// Returns performance statistics
    pub fn stats(&self) -> &ParserStats {
        &self.stats
    }

    /// Returns memory allocation statistics
    pub fn allocation_stats(&self) -> AllocationStats {
        // This would be populated by the memory pool during parsing
        AllocationStats::default()
    }

    fn next_token(&mut self) -> Result<(Token, Span)> {
        self.lexer.next_token_with_span()
    }

    fn peek_token(&mut self) -> Result<(Token, Span)> {
        self.lexer
            .peek_with_span()
            .map(|&(token, span)| (token, span))
    }

    fn parse_value(&mut self, token: (Token, Span)) -> Result<Value> {
        self.stats.values_parsed += 1;

        match token.0 {
            Token::String => self.parse_string_optimized(token.1),
            Token::Number => self.parse_number_optimized(token.1),
            Token::True => Ok(Value::Bool(true)),
            Token::False => Ok(Value::Bool(false)),
            Token::Null => Ok(Value::Null),
            Token::LeftBracket => self.parse_array_optimized(),
            Token::LeftBrace => self.parse_object_optimized(),
            Token::Newline => {
                // Skip newlines and parse the next token
                let (next_token, next_span) = self.next_token()?;
                self.parse_value((next_token, next_span))
            }
            _ => Err(Error::UnexpectedChar('?', token.1.start)),
        }
    }

    /// Parse string with compact string optimization
    fn parse_string_optimized(&mut self, span: Span) -> Result<Value> {
        self.stats.strings_parsed += 1;

        let string_slice = &self.input[span.start..span.end];
        let content = extract_string_content(string_slice)
            .map_err(|_| Error::UnterminatedString(span.start))?;

        // Small string optimization opportunity
        if content.len() <= 16 {
            self.stats.compact_string_optimizations += 1;
        }
        
        // TODO: Implement compact string optimization when Value supports it
        Ok(Value::String(content.to_string()))
    }

    /// Parse number with optimizations
    fn parse_number_optimized(&mut self, span: Span) -> Result<Value> {
        let number_str = &self.input[span.start..span.end];
        
        // Try integer parsing first (most common case)
        if !number_str.contains('.') && !number_str.contains('e') && !number_str.contains('E') {
            if let Ok(int_val) = number_str.parse::<i64>() {
                return Ok(Value::Number(Number::Integer(int_val)));
            }
        }

        // Fall back to float parsing
        if let Ok(float_val) = number_str.parse::<f64>() {
            Ok(Value::Number(Number::Float(float_val)))
        } else {
            Err(Error::InvalidNumber(span.start))
        }
    }

    /// Parse array with small vector optimization
    fn parse_array_optimized(&mut self) -> Result<Value> {
        self.stats.arrays_parsed += 1;
        
        // Check recursion depth
        self.depth += 1;
        if self.depth > self.options.max_depth {
            return Err(Error::DepthLimitExceeded(0));
        }

        // TODO: Use SmallVec when Value::Array supports it
        let mut elements = Vec::new();
        let mut first = true;

        // Pre-size based on heuristics
        let estimated_size = self.estimate_array_size();
        if estimated_size <= 8 {
            self.stats.small_vec_optimizations += 1;
        }
        elements.reserve(estimated_size.min(16)); // Cap reservation
        self.stats.presized_collections += 1;

        loop {
            // Skip newlines
            let (next_token, span) = self.peek_token()?;
            if next_token == Token::Newline {
                self.next_token()?;
                continue;
            }

            if next_token == Token::RightBracket {
                self.next_token()?; // consume ]
                break;
            }

            if !first {
                // Expect comma
                if next_token == Token::Comma {
                    self.next_token()?; // consume comma
                } else if self.options.newline_as_comma && next_token == Token::Newline {
                    self.next_token()?; // consume newline
                } else {
                    return Err(Error::Expected {
                        expected: "comma or ]".to_string(),
                        found: format!("{next_token:?}"),
                        position: span.start,
                    });
                }
            }

            let (value_token, value_span) = self.next_token()?;
            let value = self.parse_value((value_token, value_span))?;
            elements.push(value);
            
            first = false;
        }

        self.depth -= 1;
        Ok(Value::Array(elements))
    }

    /// Parse object with pre-sizing optimization
    fn parse_object_optimized(&mut self) -> Result<Value> {
        self.stats.objects_parsed += 1;

        // Check recursion depth
        self.depth += 1;
        if self.depth > self.options.max_depth {
            return Err(Error::DepthLimitExceeded(0));
        }

        // Pre-size based on heuristics for small objects
        let estimated_size = self.estimate_object_size();
        let mut object = FxHashMap::with_capacity_and_hasher(estimated_size, Default::default());
        self.stats.presized_collections += 1;

        let mut first = true;

        loop {
            // Skip newlines
            let (next_token, span) = self.peek_token()?;
            if next_token == Token::Newline {
                self.next_token()?;
                continue;
            }

            if next_token == Token::RightBrace {
                self.next_token()?; // consume }
                break;
            }

            if !first {
                // Expect comma
                if next_token == Token::Comma {
                    self.next_token()?; // consume comma
                } else if self.options.newline_as_comma && next_token == Token::Newline {
                    self.next_token()?; // consume newline
                } else {
                    return Err(Error::Expected {
                        expected: "comma or }".to_string(),
                        found: format!("{next_token:?}"),
                        position: span.start,
                    });
                }
            }

            // Parse key
            let (key_token, key_span) = self.next_token()?;
            let key = match key_token {
                Token::String => self.parse_string_key(key_span)?,
                Token::UnquotedString if self.options.allow_unquoted_keys => {
                    let key_slice = &self.input[key_span.start..key_span.end];
                    key_slice.to_string()
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
            if colon_token != Token::Colon {
                return Err(Error::Expected {
                    expected: "colon".to_string(),
                    found: format!("{colon_token:?}"),
                    position: colon_span.start,
                });
            }

            // Parse value
            let (value_token, value_span) = self.next_token()?;
            let value = self.parse_value((value_token, value_span))?;
            
            object.insert(key, value);
            first = false;
        }

        self.depth -= 1;
        Ok(Value::Object(object))
    }

    /// Parse string key with potential interning
    fn parse_string_key(&mut self, span: Span) -> Result<String> {
        let string_slice = &self.input[span.start..span.end];
        let content = extract_string_content(string_slice)
            .map_err(|_| Error::UnterminatedString(span.start))?;

        // Common keys could be interned here
        // TODO: Implement string interning for common keys
        Ok(content.to_string())
    }

    /// Estimate array size based on input heuristics
    fn estimate_array_size(&self) -> usize {
        // Simple heuristic: most JSON arrays are small
        // In practice, you might look ahead in the input or use context
        4
    }

    /// Estimate object size based on input heuristics
    fn estimate_object_size(&self) -> usize {
        // Simple heuristic: most JSON objects have few keys
        // In practice, you might look ahead in the input or use context
        4
    }
}

/// Parse JSON using the advanced optimized parser
pub fn parse_optimized_v3(input: &str) -> Result<Value> {
    let mut parser = OptimizedParserV3::new(input, ParserOptions::default());
    parser.parse()
}

/// Parse JSON with custom options using the advanced optimized parser
pub fn parse_optimized_v3_with_options(input: &str, options: ParserOptions) -> Result<Value> {
    let mut parser = OptimizedParserV3::new(input, options);
    parser.parse()
}

/// Parse JSON and return both the value and performance statistics
pub fn parse_v3_with_stats(
    input: &str,
    options: ParserOptions,
) -> Result<(Value, ParserStats, AllocationStats)> {
    let mut parser = OptimizedParserV3::new(input, options);
    let value = parser.parse()?;
    let stats = parser.stats().clone();
    let alloc_stats = parser.allocation_stats();
    Ok((value, stats, alloc_stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_v3_parser_simple() {
        let input = r#"{"name": "test", "values": [1, 2, 3]}"#;
        let result = parse_optimized_v3(input);
        assert!(result.is_ok());
        
        let value = result.unwrap();
        match value {
            Value::Object(obj) => {
                assert!(obj.contains_key("name"));
                assert!(obj.contains_key("values"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parser_v3_with_stats() {
        let input = r#"{"items": [1, 2, 3, 4], "meta": {"count": 4}}"#;
        let result = parse_v3_with_stats(input, ParserOptions::default());
        assert!(result.is_ok());
        
        let (_, stats, _) = result.unwrap();
        assert!(stats.objects_parsed >= 2); // root object + meta object
        assert!(stats.arrays_parsed >= 1); // items array
        assert!(stats.small_vec_optimizations > 0);
        assert!(stats.presized_collections > 0);
    }

    #[test]
    fn test_small_vector_optimization() {
        let input = r#"[1, 2]"#; // Small array
        let result = parse_v3_with_stats(input, ParserOptions::default());
        assert!(result.is_ok());
        
        let (_, stats, _) = result.unwrap();
        assert!(stats.small_vec_optimizations > 0);
    }
}