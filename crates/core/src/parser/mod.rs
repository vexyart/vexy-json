// this_file: src/parser/mod.rs

/// Array parsing functionality.
pub mod array;
/// Boolean value parsing.
pub mod boolean;
/// Stack-based iterative parser implementation.
pub mod iterative;
/// Null value parsing.
pub mod null;
/// Number parsing with integer and float support.
pub mod number;
/// Object parsing with key-value pairs.
pub mod object;
pub mod optimized;
pub mod optimized_v2;
/// Clean recursive descent parser implementation.
pub mod recursive;
/// Parser state management.
pub mod state;
/// String parsing with escape sequence handling.
pub mod string;

use self::boolean::{parse_false, parse_true};
use self::null::parse_null;
use self::number::parse_number_token;
use self::string::parse_string_token;
use crate::ast::{Number, Token, Value};
use crate::error::repair::{EnhancedParseResult, ParsingTier};
use crate::error::{Error, Result, Span};
use crate::lexer::{JsonLexer, Lexer, FastLexer, LexerConfig, LexerMode};
use crate::optimization::ValueBuilder;
use crate::repair::JsonRepairer;
pub use iterative::{parse_iterative, IterativeParser};
pub use optimized::{
    parse_optimized, parse_optimized_with_options, parse_with_stats, OptimizedParser,
};
pub use optimized_v2::{
    parse_optimized_v2, parse_optimized_v2_with_options, parse_v2_with_stats, OptimizedParserV2,
};
pub use recursive::{parse_recursive, RecursiveDescentParser};
use rustc_hash::FxHashMap;
pub use state::ParserState;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Configuration options for the vexy_json parser.
///
/// These options control which forgiving features are enabled during parsing.
/// By default, all forgiving features are enabled.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ParserOptions {
    /// Whether to allow single-line and multi-line comments.
    pub allow_comments: bool,
    /// Whether to allow trailing commas in arrays and objects.
    pub allow_trailing_commas: bool,
    /// Whether to allow unquoted object keys (e.g., {key: "value"}).
    pub allow_unquoted_keys: bool,
    /// Whether to allow single-quoted strings (e.g., 'value').
    pub allow_single_quotes: bool,
    /// Whether to allow implicit top-level objects and arrays.
    /// When enabled, `key: value` becomes `{key: value}` and `1, 2, 3` becomes `[1, 2, 3]`.
    pub implicit_top_level: bool,
    /// Whether to treat newlines as commas in arrays and objects.
    pub newline_as_comma: bool,
    /// Maximum nesting depth for objects and arrays to prevent stack overflow.
    pub max_depth: usize,
    /// Enable JSON repair functionality for bracket mismatches.
    pub enable_repair: bool,
    /// Maximum number of repairs to attempt.
    pub max_repairs: usize,
    /// Prefer speed over repair quality.
    pub fast_repair: bool,
    /// Report all repairs made.
    pub report_repairs: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            allow_unquoted_keys: true,
            allow_single_quotes: true,
            implicit_top_level: true,
            newline_as_comma: true,
            max_depth: 128,
            enable_repair: true,
            max_repairs: 100,
            fast_repair: false,
            report_repairs: true,
        }
    }
}

/// The vexy_json parser.
///
/// Parses tokens from a Lexer into a Value tree structure.
/// Supports both strict JSON and various forgiving extensions.
pub struct Parser<'a> {
    pub(super) lexer: Box<dyn JsonLexer + 'a>,
    pub(super) original_input: &'a str,
    pub(super) options: ParserOptions,
    pub(super) current_token: Option<(Token, Span)>,
    /// Offset of the current lexer within the original input.
    /// This is 0 when the lexer is working on the full original input,
    /// but becomes non-zero when we create a new lexer from a slice.
    pub(super) state: ParserState,
    /// Value builder for optimized object and array construction
    #[allow(dead_code)]
    pub(super) value_builder: ValueBuilder,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with the given input and options.
    pub fn new(input: &'a str, options: ParserOptions) -> Self {
        // Determine if we need forgiving features
        let needs_forgiving = options.allow_comments || 
                            options.allow_trailing_commas || 
                            options.allow_unquoted_keys || 
                            options.allow_single_quotes ||
                            options.implicit_top_level ||
                            options.newline_as_comma;
        
        // Create appropriate lexer based on options
        let lexer: Box<dyn JsonLexer + 'a> = if needs_forgiving {
            // Use FastLexer with forgiving mode for non-strict parsing
            let config = LexerConfig {
                mode: if options.allow_comments { LexerMode::Forgiving } else { LexerMode::Strict },
                collect_stats: false,
                buffer_size: 8192,
                max_depth: options.max_depth,
                track_positions: true,
            };
            Box::new(FastLexer::new(input, config))
        } else {
            // Use LogosLexer for strict parsing
            Box::new(Lexer::new(input))
        };
        
        Parser {
            lexer,
            original_input: input,
            options,
            current_token: None, // Will be populated by first advance()
            state: ParserState::new(),
            value_builder: ValueBuilder::new(),
        }
    }

    /// Parses the input and returns a Value.
    ///
    /// This is the main entry point for parsing. It handles:
    /// - Empty input (returns null)
    /// - Single values
    /// - Implicit arrays (when multiple comma-separated values are found)
    /// - Implicit objects (when key:value pairs are found at top level)
    pub fn parse(&mut self) -> Result<Value> {
        self.advance()?;
        self.skip_comments()?;

        // Handle empty input - check if we have only whitespace/newlines
        if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
            return Ok(Value::Null);
        }

        // Check if input is only newlines and whitespace (effectively empty)
        // NOTE: This check is only meaningful if we're at the start of the input
        // and haven't consumed any actual values yet
        // TEMPORARILY DISABLED - this seems to be causing issues
        // if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Newline) && self.is_only_whitespace_and_newlines() {
        //     return Ok(Value::Null);
        // }

        // Skip leading newlines when they appear after comments - they should not start implicit arrays
        // This is different from commas which can legitimately start implicit arrays
        if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Newline)
            && self.options.newline_as_comma
        {
            self.skip_comments_and_newlines()?;
            if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                return Ok(Value::Null);
            }
        }

        // Check if it starts with a separator (implicit array with null first element)
        if self.is_separator() && self.options.implicit_top_level {
            let mut array = vec![Value::Null];
            self.advance()?;

            loop {
                self.skip_comments_and_newlines()?;
                if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                    break;
                }

                // Check for consecutive separators (which mean null values)
                if self.is_separator() {
                    array.push(Value::Null);
                    self.advance()?;
                    continue;
                }

                array.push(self.parse_value()?);

                self.skip_comments_and_newlines()?;
                if self.is_separator() {
                    self.advance()?;
                } else if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                    break;
                } else {
                    return Err(Error::Expected {
                        expected: ", or newline or end of input".to_string(),
                        found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                        position: self.lexer.position(),
                    });
                }
            }

            return Ok(Value::Array(array));
        }

        // Try to parse as a regular value first (with implicit object support if enabled)
        // However, if we start with explicit braces/brackets, parse as regular value
        let is_explicit_structure = matches!(
            self.current_token.as_ref().map(|(t, _)| t),
            Some(&Token::LeftBrace) | Some(&Token::LeftBracket)
        );
        
        let first_value = if self.options.implicit_top_level && !is_explicit_structure {
            self.parse_value_or_implicit()?
        } else {
            self.parse_value()?
        };

        // Check for trailing content
        self.skip_comments()?;

        match self.current_token.as_ref().map(|(t, _)| t) {
            Some(&Token::Eof) => Ok(first_value),
            _ if is_explicit_structure => {
                // For explicit JSON structures (arrays/objects), check if there's a trailing comma
                // that should start an implicit array
                if self.options.implicit_top_level 
                    && matches!(self.current_token.as_ref().map(|(t, _)| t), Some(&Token::Comma)) {
                    // Treat the explicit structure as the first element of an implicit array
                    let mut array = vec![first_value];
                    self.advance()?;

                    loop {
                        self.skip_comments_and_newlines()?;
                        if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                            break;
                        }

                        // Check for consecutive separators (which mean null values)
                        if self.is_separator() {
                            array.push(Value::Null);
                            self.advance()?;
                            continue;
                        }

                        array.push(self.parse_value()?);

                        self.skip_comments_and_newlines()?;
                        if self.is_separator() {
                            self.advance()?;
                        } else if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                            break;
                        } else {
                            return Err(Error::Expected {
                                expected: ", or newline or end of input".to_string(),
                                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                                position: self.lexer.position(),
                            });
                        }
                    }

                    Ok(Value::Array(array))
                } else {
                    // For explicit JSON structures (arrays/objects), require end of input
                    Err(Error::Expected {
                        expected: "end of input".to_string(),
                        found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                        position: self.lexer.position(),
                    })
                }
            }
            Some(&Token::Comma) | Some(&Token::Newline)
                if matches!(
                    self.current_token.as_ref().map(|(t, _)| t),
                    Some(&Token::Comma)
                ) || (self.options.newline_as_comma
                    && matches!(
                        self.current_token.as_ref().map(|(t, _)| t),
                        Some(&Token::Newline)
                    )) =>
            {
                // Check if this is just trailing newlines/whitespace by advancing and checking
                if self.options.newline_as_comma
                    && matches!(
                        self.current_token.as_ref().map(|(t, _)| t),
                        Some(&Token::Newline)
                    )
                {
                    self.advance()?;
                    self.skip_comments_and_newlines()?;

                    // If we reach EOF after skipping newlines/comments, the newline was trailing
                    if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                        return Ok(first_value);
                    } else {
                        // There's content after the newline, so this is a real separator
                        // We need to create an implicit array
                        let mut array = vec![first_value];
                        array.push(self.parse_value()?);

                        loop {
                            self.skip_comments_and_newlines()?;
                            if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                                break;
                            }

                            if self.is_separator() {
                                self.advance()?;
                                self.skip_comments_and_newlines()?;
                                if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof)
                                {
                                    break;
                                }
                            }

                            array.push(self.parse_value()?);
                        }

                        return Ok(Value::Array(array));
                    }
                }

                // It's an implicit array (for commas)
                if self.options.implicit_top_level {
                    let mut array = vec![first_value];
                    self.advance()?;

                    loop {
                        self.skip_comments_and_newlines()?;
                        if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                            break;
                        }

                        // Check for consecutive separators (which mean null values)
                        if self.is_separator() {
                            array.push(Value::Null);
                            self.advance()?;
                            continue;
                        }

                        array.push(self.parse_value()?);

                        self.skip_comments_and_newlines()?;
                        if self.is_separator() {
                            self.advance()?;
                        } else if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                            break;
                        } else {
                            return Err(Error::Expected {
                                expected: ", or newline or end of input".to_string(),
                                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                                position: self.lexer.position(),
                            });
                        }
                    }

                    Ok(Value::Array(array))
                } else {
                    Err(Error::Expected {
                        expected: "end of input".to_string(),
                        found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                        position: self.lexer.position(),
                    })
                }
            }
            _ => {
                // Check if this is another value in an implicit array (space-separated)
                if self.options.implicit_top_level && self.is_value_token() {
                    // Create an implicit array with the first value and continue parsing
                    let mut array = vec![first_value];
                    
                    // Parse the remaining values
                    loop {
                        // Check for consecutive separators (which mean null values)
                        if self.is_separator() {
                            array.push(Value::Null);
                            self.advance()?;
                            self.skip_comments_and_newlines()?;
                            if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                                break;
                            }
                            continue;
                        }
                        
                        array.push(self.parse_value()?);
                        self.skip_comments()?;
                        
                        if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                            break;
                        }
                        
                        // Check if there's a separator
                        if self.is_separator() {
                            self.advance()?;
                            self.skip_comments_and_newlines()?;
                            if self.current_token.as_ref().map(|(t, _)| t) == Some(&Token::Eof) {
                                break;
                            }
                        } else if !self.is_value_token() {
                            return Err(Error::Expected {
                                expected: "value, separator, or end of input".to_string(),
                                found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                                position: self.lexer.position(),
                            });
                        }
                    }
                    
                    Ok(Value::Array(array))
                } else {
                    Err(Error::Expected {
                        expected: "end of input".to_string(),
                        found: format!("{:?}", self.current_token.as_ref().map(|(t, _)| t)),
                        position: self.lexer.position(),
                    })
                }
            }
        }
    }

    pub(super) fn advance(&mut self) -> Result<()> {
        loop {
            let (token, span) = self.lexer.next_token()?;
            self.state.span = span; // Update parser state with the current token's span
            self.current_token = Some((token, span));

            match self.current_token.as_ref().map(|(t, _)| t) {
                Some(&Token::SingleLineComment) | Some(&Token::MultiLineComment) => {
                    if self.options.allow_comments {
                        continue;
                    } else {
                        return Err(Error::Custom("Comments are not allowed".to_string()));
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    pub(super) fn skip_comments(&mut self) -> Result<()> {
        while matches!(
            self.current_token.as_ref().map(|(t, _)| t),
            Some(&Token::SingleLineComment) | Some(&Token::MultiLineComment)
        ) {
            self.advance()?;
        }
        Ok(())
    }

    /// Check if the current token can start a value
    fn is_value_token(&self) -> bool {
        matches!(
            self.current_token.as_ref().map(|(t, _)| t),
            Some(&Token::String) 
            | Some(&Token::UnquotedString) 
            | Some(&Token::Number) 
            | Some(&Token::LeftBrace) 
            | Some(&Token::LeftBracket) 
            | Some(&Token::True) 
            | Some(&Token::False) 
            | Some(&Token::Null)
        )
    }


    /// Skips comments and optionally newlines if newline_as_comma is enabled.
    pub(super) fn skip_comments_and_newlines(&mut self) -> Result<()> {
        let mut just_had_single_line_comment = false;
        
        loop {
            match self.current_token.as_ref().map(|(t, _)| t) {
                Some(&Token::SingleLineComment) => {
                    just_had_single_line_comment = true;
                    self.advance()?;
                }
                Some(&Token::MultiLineComment) => {
                    just_had_single_line_comment = false;
                    self.advance()?;
                }
                Some(&Token::Newline) if self.options.newline_as_comma || just_had_single_line_comment => {
                    just_had_single_line_comment = false;
                    self.advance()?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Checks if the current token is a separator (comma or newline when newline_as_comma is enabled).
    pub(super) fn is_separator(&self) -> bool {
        matches!(
            self.current_token.as_ref().map(|(t, _)| t),
            Some(&Token::Comma)
        ) || (self.options.newline_as_comma
            && matches!(
                self.current_token.as_ref().map(|(t, _)| t),
                Some(&Token::Newline)
            ))
    }

    /// Checks if the input contains only whitespace, newlines, and comments (effectively empty).
    #[allow(dead_code)]
    fn is_only_whitespace_and_newlines(&mut self) -> bool {
        // Create a temporary lexer to peek without modifying the main lexer's state
        let current_pos = self.lexer.position();
        let remaining_input = &self.original_input[current_pos..];
        
        // Create same type of lexer as the main parser
        let needs_forgiving = self.options.allow_comments || 
                            self.options.allow_trailing_commas || 
                            self.options.allow_unquoted_keys || 
                            self.options.allow_single_quotes ||
                            self.options.implicit_top_level ||
                            self.options.newline_as_comma;
        
        let mut temp_lexer: Box<dyn JsonLexer> = if needs_forgiving {
            let config = LexerConfig {
                mode: if self.options.allow_comments { LexerMode::Forgiving } else { LexerMode::Strict },
                collect_stats: false,
                buffer_size: 8192,
                max_depth: self.options.max_depth,
                track_positions: true,
            };
            Box::new(FastLexer::new(remaining_input, config))
        } else {
            Box::new(Lexer::new(remaining_input))
        };

        loop {
            match temp_lexer.next_token() {
                Ok((Token::Eof, _)) => return true,
                Ok((Token::Newline, _)) => continue,
                Ok((Token::SingleLineComment, _)) => continue,
                Ok((Token::MultiLineComment, _)) => continue,
                Ok((_, _)) => return false,
                Err(_) => return false, // Any lexer error means it's not just whitespace
            }
        }
    }

    fn parse_value_or_implicit(&mut self) -> Result<Value> {
        self.skip_comments()?;

        // Check if it's an implicit object (key:value pattern)
        if self.options.implicit_top_level {
            match self.current_token {
                Some((Token::UnquotedString, _))
                | Some((Token::String, _))
                | Some((Token::Number, _)) => {
                    // We don't need to calculate token positions manually anymore - the lexer provides spans

                    // Read the potential key
                    let potential_key = match self.current_token {
                        Some((Token::String, span)) => {
                            // Use the helper function to parse the string
                            match parse_string_token(self.original_input, span, &self.options)? {
                                Value::String(s) => s,
                                _ => {
                                    unreachable!("parse_string_token should always return a String")
                                }
                            }
                        }
                        Some((Token::UnquotedString, span)) => {
                            // Use the span information directly - no quotes to remove
                            self.original_input[span.start..span.end].to_string()
                        }
                        Some((Token::Number, span)) => {
                            // Use the span information directly
                            self.original_input[span.start..span.end].to_string()
                        }
                        _ => unreachable!(),
                    };

                    // Save the current token info before advancing
                    let key_token = self.current_token;

                    // Advance past the key token
                    self.advance()?;
                    self.skip_comments_and_newlines()?;

                    if let Some((Token::Colon, _)) = self.current_token {
                        // It's an implicit object
                        let mut object = FxHashMap::default();

                        // Parse first key-value pair
                        self.advance()?; // Skip colon
                        let value = self.parse_value()?;
                        object.insert(potential_key, value);

                        // Continue parsing object pairs
                        loop {
                            self.skip_comments_and_newlines()?;

                            if let Some((Token::Eof, _)) = self.current_token {
                                break;
                            }

                            if self.is_separator() {
                                self.advance()?;
                                self.skip_comments_and_newlines()?;

                                if let Some((Token::Eof, _)) = self.current_token {
                                    break;
                                }
                            }

                            // Parse next key
                            let key = match self.current_token {
                                Some((Token::String, span)) => {
                                    // Use the helper function to parse the string
                                    let k = match parse_string_token(
                                        self.original_input,
                                        span,
                                        &self.options,
                                    )? {
                                        Value::String(s) => s,
                                        _ => unreachable!(
                                            "parse_string_token should always return a String"
                                        ),
                                    };
                                    self.advance()?;
                                    k
                                }
                                Some((Token::UnquotedString, span)) => {
                                    // Use the span information directly - no quotes to remove
                                    let k = self.original_input[span.start..span.end].to_string();
                                    self.advance()?;
                                    k
                                }
                                Some((Token::Number, span)) => {
                                    // Use the span information directly
                                    let k = self.original_input[span.start..span.end].to_string();
                                    self.advance()?;
                                    k
                                }
                                _ => break,
                            };

                            // Expect colon
                            self.skip_comments_and_newlines()?;
                            if !matches!(self.current_token, Some((Token::Colon, _))) {
                                return Err(Error::Expected {
                                    expected: ":".to_string(),
                                    found: format!("{:?}", self.current_token),
                                    position: self.lexer.position(),
                                });
                            }
                            self.advance()?;

                            // Parse value
                            let value = self.parse_value()?;
                            object.insert(key, value);
                        }

                        return Ok(Value::Object(object));
                    } else {
                        // Not an implicit object, parse the original token as a value
                        let value = match key_token {
                            Some((Token::String, span)) => {
                                // Use the helper function to parse the string
                                parse_string_token(self.original_input, span, &self.options)?
                            }
                            Some((Token::UnquotedString, span)) => {
                                // Handle unquoted strings as values
                                let s = self.original_input[span.start..span.end].to_string();
                                Value::String(s)
                            }
                            Some((Token::Number, span)) => {
                                // Use the same number parsing logic as parse_number_token
                                parse_number_token(self.original_input, span)?
                            }
                            _ => unreachable!(),
                        };

                        // Don't advance again - we've already advanced past the token
                        return Ok(value);
                    }
                }
                _ => {}
            }
        }

        // Parse as regular value
        self.parse_value()
    }

    pub(super) fn parse_value(&mut self) -> Result<Value> {
        self.skip_comments_and_newlines()?;

        match self.current_token {
            Some((Token::Null, _)) => {
                self.advance()?;
                parse_null()
            }
            Some((Token::True, _)) => {
                self.advance()?;
                parse_true()
            }
            Some((Token::False, _)) => {
                self.advance()?;
                parse_false()
            }
            Some((Token::String, span)) => {
                let value = parse_string_token(self.original_input, span, &self.options)?;
                self.advance()?;
                Ok(value)
            }
            Some((Token::UnquotedString, span)) => {
                // Handle unquoted strings as values - extract from span
                let s = self.original_input[span.start..span.end].to_string();
                self.advance()?;
                Ok(Value::String(s))
            }
            Some((Token::Number, span)) => {
                let value = parse_number_token(self.original_input, span)?;
                self.advance()?;
                Ok(value)
            }
            Some((Token::LeftBrace, _)) => self.parse_object(),
            Some((Token::LeftBracket, _)) => self.parse_array(),
            None => {
                // If we reached EOF where a value is expected, treat it as null (likely comment)
                if self.options.allow_comments {
                    Ok(Value::Null)
                } else {
                    Err(Error::Expected {
                        expected: "value".to_string(),
                        found: "EOF".to_string(),
                        position: self.lexer.position(),
                    })
                }
            }
            Some((Token::Eof, _)) => {
                // If we reached EOF where a value is expected, treat it as null (likely comment)
                if self.options.allow_comments {
                    Ok(Value::Null)
                } else {
                    Err(Error::Expected {
                        expected: "value".to_string(),
                        found: "EOF".to_string(),
                        position: self.lexer.position(),
                    })
                }
            }
            _ => Err(Error::Expected {
                expected: "value".to_string(),
                found: format!("{:?}", self.current_token),
                position: self.lexer.position(),
            }),
        }
    }

    pub(super) fn check_depth(&self) -> Result<()> {
        if self.state.depth >= self.options.max_depth {
            Err(Error::DepthLimitExceeded(self.lexer.position()))
        } else {
            Ok(())
        }
    }
}

/// Parses a JSON string with default options (all forgiving features enabled).
///
/// # Examples
///
/// ```
/// use vexy_json_core::parse;
///
/// // Standard JSON
/// let result = parse(r#"{"key": "value"}"#);
/// assert!(result.is_ok());
///
/// // With forgiving features - unquoted keys
/// let result = parse(r#"{key: "value"}"#);
/// assert!(result.is_ok());
/// ```
pub fn parse(input: &str) -> Result<Value> {
    let mut parser = Parser::new(input, ParserOptions::default());
    parser.parse()
}

/// Parses a JSON string with custom options.
///
/// # Arguments
///
/// * `input` - The JSON string to parse
/// * `options` - Parser configuration options
///
/// # Examples
///
/// ```
/// use vexy_json_core::{parse_with_options, ParserOptions};
///
/// let mut options = ParserOptions::default();
/// options.allow_comments = false;
///
/// let result = parse_with_options(r#"{"key": "value"}"#, options);
/// assert!(result.is_ok());
/// ```
pub fn parse_with_options(input: &str, options: ParserOptions) -> Result<Value> {
    let mut parser = Parser::new(input, options);
    parser.parse()
}

/// Enhanced parsing with three-tier fallback strategy (serde_json → vexy_json → repair)
///
/// This function implements a progressive parsing strategy:
/// 1. First tries serde_json for maximum performance on valid JSON
/// 2. Falls back to vexy_json for forgiving parsing of non-standard JSON
/// 3. Finally attempts repair for malformed JSON (bracket imbalances, etc.)
///
/// Returns an `EnhancedParseResult` that includes information about which
/// parsing tier was used and any repairs that were applied.
pub fn parse_with_fallback(input: &str, options: ParserOptions) -> EnhancedParseResult<Value> {
    // Tier 1: Try serde_json for maximum performance on valid JSON
    if let Ok(serde_value) = serde_json::from_str::<serde_json::Value>(input) {
        // Convert serde_json::Value to vexy_json::Value
        let vexy_json_value = convert_serde_to_vexy_json(serde_value);
        return EnhancedParseResult::success(vexy_json_value, ParsingTier::Fast);
    }

    // Tier 2: Try vexy_json for forgiving parsing
    match parse_with_options(input, options.clone()) {
        Ok(value) => EnhancedParseResult::success(value, ParsingTier::Forgiving),
        Err(error) => {
            // Tier 3: Try repair if enabled
            if options.enable_repair {
                parse_with_repair(input, &options)
            } else {
                EnhancedParseResult::failure(Value::Null, vec![error], ParsingTier::Forgiving)
            }
        }
    }
}

/// Parse with repair functionality for bracket mismatches
fn parse_with_repair(input: &str, options: &ParserOptions) -> EnhancedParseResult<Value> {
    let mut repairer = if options.fast_repair {
        JsonRepairer::new_without_cache(options.max_repairs)
    } else {
        JsonRepairer::new(options.max_repairs)
    };

    match repairer.repair(input) {
        Ok((repaired_json, repairs)) => {
            // Try to parse the repaired JSON with vexy_json
            match parse_with_options(&repaired_json, options.clone()) {
                Ok(value) => {
                    EnhancedParseResult::success_with_repairs(value, repairs, ParsingTier::Repair)
                }
                Err(error) => {
                    // Even repair failed - return best effort
                    EnhancedParseResult::failure_with_repairs(
                        Value::Null,
                        vec![error],
                        repairs,
                        ParsingTier::Repair,
                    )
                }
            }
        }
        Err(repair_error) => EnhancedParseResult::failure(
            Value::Null,
            vec![Error::RepairFailed(repair_error)],
            ParsingTier::Repair,
        ),
    }
}

/// Convert serde_json::Value to vexy_json::Value
fn convert_serde_to_vexy_json(serde_value: serde_json::Value) -> Value {
    match serde_value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(Number::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Value::Number(Number::Float(f))
            } else {
                Value::Number(Number::Float(0.0))
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let converted: Vec<Value> = arr.into_iter().map(convert_serde_to_vexy_json).collect();
            Value::Array(converted)
        }
        serde_json::Value::Object(obj) => {
            let converted: FxHashMap<String, Value> = obj
                .into_iter()
                .map(|(k, v)| (k, convert_serde_to_vexy_json(v)))
                .collect();
            Value::Object(converted)
        }
    }
}

/// Enhanced parsing function that reports all repairs made
pub fn parse_with_detailed_repair_tracking(
    input: &str,
    options: ParserOptions,
) -> EnhancedParseResult<Value> {
    let mut repairer = JsonRepairer::new(options.max_repairs);

    match repairer.repair_with_detailed_tracking(input) {
        Ok((repaired_json, repairs)) => match parse_with_options(&repaired_json, options) {
            Ok(value) => {
                EnhancedParseResult::success_with_repairs(value, repairs, ParsingTier::Repair)
            }
            Err(error) => EnhancedParseResult::failure_with_repairs(
                Value::Null,
                vec![error],
                repairs,
                ParsingTier::Repair,
            ),
        },
        Err(repair_error) => EnhancedParseResult::failure(
            Value::Null,
            vec![Error::RepairFailed(repair_error)],
            ParsingTier::Repair,
        ),
    }
}
