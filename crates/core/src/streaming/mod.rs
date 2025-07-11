// this_file: src/streaming/mod.rs

//! Streaming parser implementation for vexy_json.
//!
//! This module provides a streaming JSON parser that can process input
//! incrementally, making it suitable for parsing large files or real-time
//! data streams without loading the entire content into memory.

mod buffered;
pub mod event_parser;
mod ndjson;
mod simple_lexer;

pub use buffered::{
    parse_streaming, parse_streaming_with_config, BufferedStreamingConfig, BufferedStreamingParser,
    StreamingEventIterator,
};
pub use event_parser::{
    EventDrivenParser, EventParserConfig, JsonEventHandler, ParserContext as EventParserContext,
    ParserState as EventParserState,
};
pub use ndjson::{NdJsonIterator, NdJsonParser, StreamingNdJsonParser};
pub use simple_lexer::SimpleStreamingLexer;

#[cfg(feature = "async")]
pub use event_parser::AsyncEventDrivenParser;

use crate::ast::{Token, Value};
use crate::error::{Error, Result};
use rustc_hash::FxHashMap;

/// Events emitted by the streaming parser
#[derive(Debug, Clone, PartialEq)]
pub enum StreamingEvent {
    /// Start of a JSON object
    StartObject,
    /// End of a JSON object
    EndObject,
    /// Start of a JSON array
    StartArray,
    /// End of a JSON array
    EndArray,
    /// Object key (in objects, before the corresponding value)
    ObjectKey(String),
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Number value (as string to preserve precision)
    Number(String),
    /// String value
    String(String),
    /// End of input
    EndOfInput,
}

/// State of the streaming parser
#[derive(Debug)]
pub struct StreamingParser {
    /// Streaming lexer
    lexer: SimpleStreamingLexer,
    /// Parser state stack for nested structures
    state_stack: Vec<ParserContext>,
    /// Current parser state
    current_state: ParserState,
    /// Parser options
    options: crate::parser::ParserOptions,
    /// Event queue
    event_queue: Vec<StreamingEvent>,
    /// Whether parsing is complete
    finished: bool,
    /// Current token being processed
    current_token: Option<(Token, crate::error::Span)>,
}

/// Internal parser state
#[derive(Debug, Clone)]
enum ParserState {
    /// Expecting a value (could be any JSON value)
    ExpectingValue,
    /// Inside an object, expecting key or closing brace
    InObject { expecting_key: bool },
    /// Inside an array, expecting value or closing bracket
    InArray { #[allow(dead_code)] first_element: bool },
    /// Between values (handling whitespace/commas)
    BetweenValues,
}

/// Context for nested structures
#[derive(Debug, Clone)]
enum ParserContext {
    Object { expecting_key: bool },
    Array { first_element: bool },
}

impl StreamingParser {
    /// Create a new streaming parser with default options
    pub fn new() -> Self {
        Self::with_options(crate::parser::ParserOptions::default())
    }

    /// Create a new streaming parser with custom options
    pub fn with_options(options: crate::parser::ParserOptions) -> Self {
        let lexer = SimpleStreamingLexer::with_options(options.clone());
        Self {
            lexer,
            state_stack: Vec::new(),
            current_state: ParserState::ExpectingValue,
            options,
            event_queue: Vec::new(),
            finished: false,
            current_token: None,
        }
    }

    /// Feed a chunk of input to the parser
    pub fn feed(&mut self, chunk: &str) -> Result<()> {
        if self.finished {
            return Err(Error::Custom("Parser already finished".to_string()));
        }

        // Feed to lexer
        self.lexer.feed_str(chunk)?;

        // Process any available tokens
        self.process_tokens()?;

        Ok(())
    }

    /// Process tokens from the lexer
    fn process_tokens(&mut self) -> Result<()> {
        loop {
            // Get next token if we don't have one
            if self.current_token.is_none() {
                self.current_token = self.lexer.next_token();
            }

            // If no token available, we're done for now
            let Some((token, _span)) = self.current_token.clone() else {
                break;
            };

            // Skip comments
            if matches!(token, Token::SingleLineComment | Token::MultiLineComment) {
                self.current_token = None;
                continue;
            }

            // Process token based on current state
            let consumed = match &self.current_state {
                ParserState::ExpectingValue => self.process_value(token)?,
                ParserState::InObject { expecting_key } => {
                    if *expecting_key {
                        self.process_object_key(token)?
                    } else {
                        self.process_value(token)?
                    }
                }
                ParserState::InArray { .. } => self.process_value(token)?,
                ParserState::BetweenValues => self.process_between_values(token)?,
            };

            if consumed {
                self.current_token = None;
            } else {
                // Token not consumed, stop processing
                break;
            }
        }
        Ok(())
    }

    /// Process a value token
    fn process_value(&mut self, token: Token) -> Result<bool> {
        match token {
            Token::LeftBrace => {
                self.event_queue.push(StreamingEvent::StartObject);
                self.state_stack.push(ParserContext::Object {
                    expecting_key: true,
                });
                self.current_state = ParserState::InObject {
                    expecting_key: true,
                };
                Ok(true)
            }
            Token::LeftBracket => {
                self.event_queue.push(StreamingEvent::StartArray);
                self.state_stack.push(ParserContext::Array {
                    first_element: true,
                });
                self.current_state = ParserState::InArray {
                    first_element: true,
                };
                Ok(true)
            }
            Token::String => {
                // Note: actual string content would need to be extracted from lexer
                self.event_queue
                    .push(StreamingEvent::String("".to_string()));
                self.transition_after_value();
                Ok(true)
            }
            Token::Number => {
                // Note: actual number content would need to be extracted from lexer
                self.event_queue
                    .push(StreamingEvent::Number("0".to_string()));
                self.transition_after_value();
                Ok(true)
            }
            Token::True => {
                self.event_queue.push(StreamingEvent::Bool(true));
                self.transition_after_value();
                Ok(true)
            }
            Token::False => {
                self.event_queue.push(StreamingEvent::Bool(false));
                self.transition_after_value();
                Ok(true)
            }
            Token::Null => {
                self.event_queue.push(StreamingEvent::Null);
                self.transition_after_value();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Process an object key
    fn process_object_key(&mut self, token: Token) -> Result<bool> {
        match token {
            Token::String => {
                // Note: actual string content would need to be extracted from lexer
                self.event_queue
                    .push(StreamingEvent::ObjectKey("".to_string()));
                self.current_state = ParserState::ExpectingValue;
                Ok(true)
            }
            Token::UnquotedString if self.options.allow_unquoted_keys => {
                // Note: actual string content would need to be extracted from lexer
                self.event_queue
                    .push(StreamingEvent::ObjectKey("".to_string()));
                self.current_state = ParserState::ExpectingValue;
                Ok(true)
            }
            Token::RightBrace => {
                // Empty object or trailing comma
                self.event_queue.push(StreamingEvent::EndObject);
                self.state_stack.pop();
                self.transition_after_value();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Process tokens between values
    fn process_between_values(&mut self, token: Token) -> Result<bool> {
        match token {
            Token::Comma => {
                // Move to next value
                if let Some(context) = self.state_stack.last() {
                    match context {
                        ParserContext::Object { .. } => {
                            self.current_state = ParserState::InObject {
                                expecting_key: true,
                            };
                        }
                        ParserContext::Array { .. } => {
                            self.current_state = ParserState::InArray {
                                first_element: false,
                            };
                        }
                    }
                }
                Ok(true)
            }
            Token::RightBrace => {
                if matches!(self.state_stack.last(), Some(ParserContext::Object { .. })) {
                    self.event_queue.push(StreamingEvent::EndObject);
                    self.state_stack.pop();
                    self.transition_after_value();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Token::RightBracket => {
                if matches!(self.state_stack.last(), Some(ParserContext::Array { .. })) {
                    self.event_queue.push(StreamingEvent::EndArray);
                    self.state_stack.pop();
                    self.transition_after_value();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    /// Transition state after processing a value
    fn transition_after_value(&mut self) {
        if self.state_stack.is_empty() {
            self.current_state = ParserState::BetweenValues;
        } else {
            self.current_state = ParserState::BetweenValues;
        }
    }

    /// Get the next event from the parser
    pub fn next_event(&mut self) -> Result<Option<StreamingEvent>> {
        // Process more tokens if needed
        self.process_tokens()?;

        // Return queued event if available
        if !self.event_queue.is_empty() {
            Ok(Some(self.event_queue.remove(0)))
        } else if self.finished && self.state_stack.is_empty() {
            Ok(Some(StreamingEvent::EndOfInput))
        } else {
            Ok(None)
        }
    }

    /// Signal end of input
    pub fn finish(&mut self) -> Result<()> {
        self.lexer.finish()?;
        self.process_tokens()?;

        if !self.state_stack.is_empty() {
            return Err(Error::Custom("Unexpected end of input".to_string()));
        }

        self.finished = true;
        Ok(())
    }

    /// Check if the parser has finished
    pub fn is_finished(&self) -> bool {
        self.finished && self.event_queue.is_empty()
    }
}

/// Iterator interface for streaming events
impl Iterator for StreamingParser {
    type Item = Result<StreamingEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_event() {
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Builder pattern for constructing values from events
pub struct StreamingValueBuilder {
    stack: Vec<BuilderState>,
    root: Option<Value>,
}

enum BuilderState {
    Object(FxHashMap<String, Value>, Option<String>),
    Array(Vec<Value>),
}

impl StreamingValueBuilder {
    /// Create a new value builder
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            root: None,
        }
    }

    /// Process a streaming event
    pub fn process_event(&mut self, event: StreamingEvent) -> Result<()> {
        match event {
            StreamingEvent::StartObject => {
                self.stack
                    .push(BuilderState::Object(FxHashMap::default(), None));
            }
            StreamingEvent::StartArray => {
                self.stack.push(BuilderState::Array(Vec::new()));
            }
            StreamingEvent::EndObject => {
                if let Some(BuilderState::Object(map, _)) = self.stack.pop() {
                    self.add_value(Value::Object(map))?;
                } else {
                    return Err(Error::Custom("Unexpected EndObject".to_string()));
                }
            }
            StreamingEvent::EndArray => {
                if let Some(BuilderState::Array(vec)) = self.stack.pop() {
                    self.add_value(Value::Array(vec))?;
                } else {
                    return Err(Error::Custom("Unexpected EndArray".to_string()));
                }
            }
            StreamingEvent::ObjectKey(key) => {
                if let Some(BuilderState::Object(_, ref mut pending_key)) = self.stack.last_mut() {
                    *pending_key = Some(key);
                } else {
                    return Err(Error::Custom("ObjectKey outside of object".to_string()));
                }
            }
            StreamingEvent::Null => self.add_value(Value::Null)?,
            StreamingEvent::Bool(b) => self.add_value(Value::Bool(b))?,
            StreamingEvent::Number(n) => {
                // Parse number string to Value::Number
                let value = if n.contains('.') || n.contains('e') || n.contains('E') {
                    Value::Number(crate::ast::Number::Float(
                        n.parse()
                            .map_err(|_| Error::Custom(format!("Invalid number: {}", n)))?,
                    ))
                } else {
                    Value::Number(crate::ast::Number::Integer(
                        n.parse()
                            .map_err(|_| Error::Custom(format!("Invalid number: {}", n)))?,
                    ))
                };
                self.add_value(value)?;
            }
            StreamingEvent::String(s) => self.add_value(Value::String(s))?,
            StreamingEvent::EndOfInput => {
                if !self.stack.is_empty() {
                    return Err(Error::Custom("Unexpected end of input".to_string()));
                }
            }
        }
        Ok(())
    }

    /// Add a value to the current container
    fn add_value(&mut self, value: Value) -> Result<()> {
        if self.stack.is_empty() {
            if self.root.is_some() {
                return Err(Error::Custom("Multiple root values".to_string()));
            }
            self.root = Some(value);
        } else {
            match self.stack.last_mut().unwrap() {
                BuilderState::Object(map, pending_key) => {
                    if let Some(key) = pending_key.take() {
                        map.insert(key, value);
                    } else {
                        return Err(Error::Custom("Value without key in object".to_string()));
                    }
                }
                BuilderState::Array(vec) => {
                    vec.push(value);
                }
            }
        }
        Ok(())
    }

    /// Get the final built value
    pub fn finish(self) -> Result<Option<Value>> {
        if !self.stack.is_empty() {
            return Err(Error::Custom("Incomplete JSON structure".to_string()));
        }
        Ok(self.root)
    }
}

impl Default for StreamingValueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_parser_creation() {
        let parser = StreamingParser::new();
        assert!(!parser.is_finished());
    }

    #[test]
    fn test_value_builder() {
        let mut builder = StreamingValueBuilder::new();

        // Build a simple object: {"key": "value"}
        builder.process_event(StreamingEvent::StartObject).unwrap();
        builder
            .process_event(StreamingEvent::ObjectKey("key".to_string()))
            .unwrap();
        builder
            .process_event(StreamingEvent::String("value".to_string()))
            .unwrap();
        builder.process_event(StreamingEvent::EndObject).unwrap();

        let value = builder.finish().unwrap().unwrap();
        match value {
            Value::Object(map) => {
                assert_eq!(map.get("key").unwrap(), &Value::String("value".to_string()));
            }
            _ => panic!("Expected object"),
        }
    }
}
