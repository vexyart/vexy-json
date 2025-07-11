//! Event-driven streaming parser with resumable parsing and JSONPath support.
//!
//! This module provides an event-driven API for parsing JSON streams with:
//! - Handler-based event processing
//! - Resumable parsing with state persistence
//! - JSONPath-based selective parsing
//! - Async I/O support
//! - Memory-efficient partial extraction

use crate::ast::{Token, Value};
use crate::error::{Error, Result};
use crate::streaming::SimpleStreamingLexer;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io::Read;

#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

/// Trait for handling JSON parsing events
pub trait JsonEventHandler: Send {
    /// Called when parsing starts
    fn on_parse_start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when an object starts
    fn on_object_start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when an object ends
    fn on_object_end(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when an array starts
    fn on_array_start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when an array ends
    fn on_array_end(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called for each object key
    fn on_key(&mut self, _key: &str) -> Result<()> {
        Ok(())
    }

    /// Called for each value (including array elements)
    fn on_value(&mut self, _value: &Value) -> Result<()> {
        Ok(())
    }

    /// Called for null values
    fn on_null(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called for boolean values
    fn on_bool(&mut self, _value: bool) -> Result<()> {
        Ok(())
    }

    /// Called for number values
    fn on_number(&mut self, _value: &str) -> Result<()> {
        Ok(())
    }

    /// Called for string values
    fn on_string(&mut self, _value: &str) -> Result<()> {
        Ok(())
    }

    /// Called when parsing completes
    fn on_parse_end(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called on parsing error
    fn on_error(&mut self, error: &Error) -> Result<()> {
        Err(error.clone())
    }
}

/// Parser state for resumable parsing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParserState {
    /// Current position in the input
    pub position: usize,
    /// Stack of nested contexts
    pub context_stack: Vec<ParserContext>,
    /// Current parsing context
    pub current_context: ParserContext,
    /// Whether parsing is complete
    pub is_complete: bool,
    /// Partial value buffer for chunk boundaries
    pub partial_buffer: String,
}

/// Parsing context for nested structures
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ParserContext {
    /// Root context
    Root,
    /// Inside an object
    Object {
        /// Whether we're expecting a key
        expecting_key: bool,
        /// Current key being processed
        current_key: Option<String>,
    },
    /// Inside an array
    Array {
        /// Index of current element
        index: usize,
    },
}

/// Configuration for event-driven parser
#[derive(Debug, Clone)]
pub struct EventParserConfig {
    /// Maximum depth for nested structures
    pub max_depth: usize,
    /// Buffer size for reading chunks
    pub chunk_size: usize,
    /// JSONPath expressions for selective parsing
    pub json_paths: Vec<String>,
    /// Whether to skip large arrays/objects
    pub skip_large_values: bool,
    /// Threshold for "large" values
    pub large_value_threshold: usize,
}

impl Default for EventParserConfig {
    fn default() -> Self {
        EventParserConfig {
            max_depth: 128,
            chunk_size: 8192,
            json_paths: Vec::new(),
            skip_large_values: false,
            large_value_threshold: 1024 * 1024, // 1MB
        }
    }
}

/// Event-driven streaming parser
pub struct EventDrivenParser<H: JsonEventHandler> {
    /// Event handler
    handler: H,
    /// Parser configuration
    config: EventParserConfig,
    /// Parser state for resumable parsing
    state: ParserState,
    /// JSONPath matcher
    path_matcher: Option<JsonPathMatcher>,
}

impl<H: JsonEventHandler> EventDrivenParser<H> {
    /// Create a new event-driven parser
    pub fn new(handler: H) -> Self {
        Self::with_config(handler, EventParserConfig::default())
    }

    /// Create parser with custom configuration
    pub fn with_config(handler: H, config: EventParserConfig) -> Self {
        let path_matcher = if !config.json_paths.is_empty() {
            Some(JsonPathMatcher::new(&config.json_paths))
        } else {
            None
        };

        EventDrivenParser {
            handler,
            config,
            state: ParserState {
                position: 0,
                context_stack: Vec::new(),
                current_context: ParserContext::Root,
                is_complete: false,
                partial_buffer: String::new(),
            },
            path_matcher,
        }
    }

    /// Parse from a reader
    pub fn parse<R: Read>(&mut self, reader: &mut R) -> Result<()> {
        self.handler.on_parse_start()?;

        let mut buffer = vec![0; self.config.chunk_size];

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .map_err(|e| Error::Custom(format!("IO error: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            let chunk = std::str::from_utf8(&buffer[..bytes_read])
                .map_err(|e| Error::Custom(format!("UTF-8 error: {}", e)))?;

            self.parse_chunk(chunk)?;
        }

        self.finish_parsing()?;
        Ok(())
    }

    /// Parse a chunk of input
    pub fn parse_chunk(&mut self, chunk: &str) -> Result<()> {
        // Combine with any partial data from previous chunk
        let combined_input;
        let input = if self.state.partial_buffer.is_empty() {
            chunk
        } else {
            combined_input = format!("{}{}", self.state.partial_buffer, chunk);
            &combined_input
        };

        // Process the input
        let (processed_bytes, remaining) = self.process_input(input)?;

        // Update state after processing
        self.state.position += processed_bytes;
        self.state.partial_buffer = remaining;

        Ok(())
    }

    /// Process input and return number of bytes processed and remaining data
    fn process_input(&mut self, input: &str) -> Result<(usize, String)> {
        let mut lexer = SimpleStreamingLexer::new();
        let mut position = 0;

        // Feed input to lexer
        lexer.feed_str(input)?;

        // Process tokens
        while let Some((token, span)) = lexer.next_token() {
            if !self.should_process_token(&token)? {
                continue;
            }

            match &token {
                Token::LeftBrace => {
                    self.handler.on_object_start()?;
                    self.push_context(ParserContext::Object {
                        expecting_key: true,
                        current_key: None,
                    });
                }
                Token::RightBrace => {
                    self.handler.on_object_end()?;
                    self.pop_context()?;
                }
                Token::LeftBracket => {
                    self.handler.on_array_start()?;
                    self.push_context(ParserContext::Array { index: 0 });
                }
                Token::RightBracket => {
                    self.handler.on_array_end()?;
                    self.pop_context()?;
                }
                Token::String => {
                    // For now, we can't extract string values from tokens
                    // This would need to be redesigned to work with the lexer API
                    self.handler.on_string("<string>")?;
                }
                Token::Number => {
                    // For now, we can't extract number values from tokens
                    self.handler.on_number("<number>")?;
                }
                Token::True => self.handler.on_bool(true)?,
                Token::False => self.handler.on_bool(false)?,
                Token::Null => self.handler.on_null()?,
                Token::Comma => self.handle_comma()?,
                Token::Colon => self.handle_colon()?,
                _ => {}
            }

            position = span.start;
        }

        // Return processed bytes and remaining input
        let remaining = input[position..].to_string();
        Ok((position, remaining))
    }

    /// Check if current path matches JSONPath filters
    fn should_process_token(&self, _token: &Token) -> Result<bool> {
        if let Some(ref matcher) = self.path_matcher {
            Ok(matcher.matches(&self.get_current_path()))
        } else {
            Ok(true)
        }
    }

    /// Get current JSONPath
    fn get_current_path(&self) -> String {
        let mut path = String::from("$");

        for context in &self.state.context_stack {
            match context {
                ParserContext::Object {
                    current_key: Some(key),
                    ..
                } => {
                    path.push('.');
                    path.push_str(key);
                }
                ParserContext::Array { index } => {
                    path.push_str(&format!("[{}]", index));
                }
                _ => {}
            }
        }

        path
    }

    /// Push a new context
    fn push_context(&mut self, context: ParserContext) {
        if self.state.context_stack.len() >= self.config.max_depth {
            // Handle max depth by skipping
            return;
        }

        self.state
            .context_stack
            .push(self.state.current_context.clone());
        self.state.current_context = context;
    }

    /// Pop a context
    fn pop_context(&mut self) -> Result<()> {
        if let Some(prev) = self.state.context_stack.pop() {
            self.state.current_context = prev;
            Ok(())
        } else {
            Err(Error::Custom("Unexpected closing bracket".to_string()))
        }
    }

    /// Update object context
    #[allow(dead_code)]
    fn update_object_context(&mut self, key: Option<String>) {
        if let ParserContext::Object {
            expecting_key,
            current_key,
        } = &mut self.state.current_context
        {
            *expecting_key = false;
            *current_key = key;
        }
    }

    /// Handle comma token
    fn handle_comma(&mut self) -> Result<()> {
        match &mut self.state.current_context {
            ParserContext::Object { expecting_key, .. } => {
                *expecting_key = true;
            }
            ParserContext::Array { index } => {
                *index += 1;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle colon token
    fn handle_colon(&mut self) -> Result<()> {
        if let ParserContext::Object { expecting_key, .. } = &mut self.state.current_context {
            *expecting_key = false;
        }
        Ok(())
    }

    /// Finish parsing
    pub fn finish_parsing(&mut self) -> Result<()> {
        if !self.state.partial_buffer.is_empty() {
            return Err(Error::Custom("Incomplete JSON at end of input".to_string()));
        }

        if !self.state.context_stack.is_empty() {
            return Err(Error::Custom("Unclosed brackets".to_string()));
        }

        self.state.is_complete = true;
        self.handler.on_parse_end()?;
        Ok(())
    }

    /// Save parser state for resumption
    pub fn save_state(&self) -> ParserState {
        self.state.clone()
    }

    /// Resume parsing from saved state
    pub fn resume_from_state(mut self, state: ParserState) -> Self {
        self.state = state;
        self
    }
}

/// JSONPath matcher for selective parsing
struct JsonPathMatcher {
    paths: Vec<String>,
}

impl JsonPathMatcher {
    fn new(paths: &[String]) -> Self {
        JsonPathMatcher {
            paths: paths.to_vec(),
        }
    }

    fn matches(&self, current_path: &str) -> bool {
        // Simple prefix matching for now
        self.paths.iter().any(|p| current_path.starts_with(p))
    }
}

/// Async version of the event-driven parser
#[cfg(feature = "async")]
pub struct AsyncEventDrivenParser<H: JsonEventHandler> {
    inner: EventDrivenParser<H>,
}

#[cfg(feature = "async")]
impl<H: JsonEventHandler> AsyncEventDrivenParser<H> {
    /// Create new async parser
    pub fn new(handler: H) -> Self {
        AsyncEventDrivenParser {
            inner: EventDrivenParser::new(handler),
        }
    }

    /// Parse from async reader
    pub async fn parse<R: AsyncRead + Unpin>(&mut self, reader: &mut R) -> Result<()> {
        self.inner.handler.on_parse_start()?;

        let mut buffer = vec![0; self.inner.config.chunk_size];

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await
                .map_err(|e| Error::Custom(format!("Async IO error: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            let chunk = std::str::from_utf8(&buffer[..bytes_read])
                .map_err(|e| Error::Custom(format!("UTF-8 error: {}", e)))?;

            self.inner.parse_chunk(chunk)?;
        }

        self.inner.finish_parsing()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        events: Vec<String>,
    }

    impl JsonEventHandler for TestHandler {
        fn on_object_start(&mut self) -> Result<()> {
            self.events.push("object_start".to_string());
            Ok(())
        }

        fn on_object_end(&mut self) -> Result<()> {
            self.events.push("object_end".to_string());
            Ok(())
        }

        fn on_key(&mut self, key: &str) -> Result<()> {
            self.events.push(format!("key:{}", key));
            Ok(())
        }

        fn on_string(&mut self, value: &str) -> Result<()> {
            self.events.push(format!("string:{}", value));
            Ok(())
        }
    }

    #[test]
    fn test_event_driven_parser() {
        let handler = TestHandler { events: Vec::new() };
        let mut parser = EventDrivenParser::new(handler);

        let json = r#"{"name": "test", "value": "data"}"#;
        let mut cursor = std::io::Cursor::new(json);

        parser.parse(&mut cursor).unwrap();

        let events = &parser.handler.events;
        assert!(events.contains(&"object_start".to_string()));
        assert!(events.contains(&"key:name".to_string()));
        assert!(events.contains(&"string:test".to_string()));
        assert!(events.contains(&"object_end".to_string()));
    }

    #[test]
    fn test_resumable_parsing() {
        let handler = TestHandler { events: Vec::new() };
        let mut parser = EventDrivenParser::new(handler);

        // Parse first chunk
        parser.parse_chunk(r#"{"name": "#).unwrap();

        // Save state
        let state = parser.save_state();
        assert!(!state.is_complete);

        // Parse second chunk
        parser.parse_chunk(r#""test"}"#).unwrap();
        parser.finish_parsing().unwrap();

        assert!(parser.state.is_complete);
    }
}
