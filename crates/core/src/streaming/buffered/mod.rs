// this_file: crates/core/src/streaming/buffered/mod.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result};
use crate::parser::ParserOptions;
use crate::streaming::{ParserContext, StreamingEvent};
// use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::io::{BufReader, Read};

pub mod buffer;
pub mod state;

use state::TempParsingState;

/// Configuration for the buffered streaming parser.
#[derive(Debug, Clone)]
pub struct BufferedStreamingConfig {
    /// Size of the input buffer in bytes (default: 8192)
    pub input_buffer_size: usize,
    /// Size of the token buffer (default: 1024)
    pub token_buffer_size: usize,
    /// Size of the event buffer (default: 512)
    pub event_buffer_size: usize,
    /// Whether to preserve raw string values for numbers
    pub preserve_number_precision: bool,
    /// Parser options
    pub parser_options: ParserOptions,
}

impl Default for BufferedStreamingConfig {
    fn default() -> Self {
        BufferedStreamingConfig {
            input_buffer_size: 8192,
            token_buffer_size: 1024,
            event_buffer_size: 512,
            preserve_number_precision: true,
            parser_options: ParserOptions::default(),
        }
    }
}

/// A buffered streaming JSON parser that processes input incrementally.
pub struct BufferedStreamingParser<R: Read> {
    /// Buffered reader for input
    reader: BufReader<R>,
    /// Configuration
    config: BufferedStreamingConfig,
    /// Input buffer for partial reads
    input_buffer: String,
    /// Token buffer for parsed tokens
    token_buffer: VecDeque<(Token, String)>,
    /// Event buffer for generated events
    event_buffer: VecDeque<StreamingEvent>,
    /// Parser state stack
    state_stack: Vec<ParserContext>,
    /// Current position in the input stream
    position: usize,
    /// Whether we've reached the end of input
    end_of_input: bool,
    /// Temporary state for parsing complex values
    #[allow(dead_code)]
    temp_state: TempParsingState,
}

impl<R: Read> BufferedStreamingParser<R> {
    /// Creates a new buffered streaming parser with default configuration.
    pub fn new(reader: R) -> Self {
        Self::with_config(reader, BufferedStreamingConfig::default())
    }

    /// Creates a new buffered streaming parser with custom configuration.
    pub fn with_config(reader: R, config: BufferedStreamingConfig) -> Self {
        let buffer_size = config.input_buffer_size;
        BufferedStreamingParser {
            reader: BufReader::with_capacity(buffer_size, reader),
            config,
            input_buffer: String::with_capacity(buffer_size),
            token_buffer: VecDeque::with_capacity(1024),
            event_buffer: VecDeque::with_capacity(512),
            state_stack: Vec::new(),
            position: 0,
            end_of_input: false,
            temp_state: TempParsingState::default(),
        }
    }

    /// Returns the next streaming event, if available.
    pub fn next_event(&mut self) -> Result<Option<StreamingEvent>> {
        // Return buffered events first
        if let Some(event) = self.event_buffer.pop_front() {
            return Ok(Some(event));
        }

        // If we've reached end of input and no more events, return None
        if self.end_of_input && self.token_buffer.is_empty() {
            return Ok(None);
        }

        // Try to fill buffers and generate more events
        self.fill_buffers()?;
        self.process_tokens()?;

        // Return the next event if available
        Ok(self.event_buffer.pop_front())
    }

    /// Classifies a token string into a Token enum variant.
    fn classify_token(&self, token_str: &str) -> Result<Token> {
        match token_str {
            "{" => Ok(Token::LeftBrace),
            "}" => Ok(Token::RightBrace),
            "[" => Ok(Token::LeftBracket),
            "]" => Ok(Token::RightBracket),
            "," => Ok(Token::Comma),
            ":" => Ok(Token::Colon),
            "\n" => Ok(Token::Newline),
            "null" => Ok(Token::Null),
            "true" => Ok(Token::True),
            "false" => Ok(Token::False),
            _ => {
                // Check if it's a string
                if (token_str.starts_with('"') && token_str.ends_with('"'))
                    || (token_str.starts_with('\'') && token_str.ends_with('\''))
                {
                    Ok(Token::String)
                }
                // Check if it's a number
                else if self.is_number(token_str) {
                    Ok(Token::Number)
                }
                // Check if it's an unquoted string
                else if self.is_unquoted_string(token_str) {
                    Ok(Token::UnquotedString)
                } else {
                    Err(Error::UnexpectedChar(
                        token_str.chars().next().unwrap_or('\0'),
                        self.position,
                    ))
                }
            }
        }
    }

    /// Checks if a string represents a valid number.
    #[inline(always)]
    fn is_number(&self, s: &str) -> bool {
        s.parse::<f64>().is_ok()
    }

    /// Checks if a string is a valid unquoted string.
    #[inline(always)]
    fn is_unquoted_string(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let first_char = s.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' && first_char != '$' {
            return false;
        }

        s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    }

    /// Converts a character to its corresponding token.
    #[allow(dead_code)]
    fn char_to_token(&self, ch: char) -> Result<Token> {
        match ch {
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            '[' => Ok(Token::LeftBracket),
            ']' => Ok(Token::RightBracket),
            ',' => Ok(Token::Comma),
            ':' => Ok(Token::Colon),
            '\n' => Ok(Token::Newline),
            _ => Err(Error::UnexpectedChar(ch, self.position)),
        }
    }

    /// Processes tokens from the token buffer and generates events.
    fn process_tokens(&mut self) -> Result<()> {
        while let Some((token, token_str)) = self.token_buffer.pop_front() {
            if self.event_buffer.len() >= self.config.event_buffer_size {
                // Event buffer is full, stop processing
                self.token_buffer.push_front((token, token_str));
                break;
            }

            let event = self.token_to_event(token, &token_str)?;
            if let Some(event) = event {
                self.event_buffer.push_back(event);
            }
        }

        Ok(())
    }

    /// Converts a token to a streaming event.
    fn token_to_event(&mut self, token: Token, token_str: &str) -> Result<Option<StreamingEvent>> {
        match token {
            Token::LeftBrace => {
                self.state_stack.push(ParserContext::Object {
                    expecting_key: true,
                });
                Ok(Some(StreamingEvent::StartObject))
            }
            Token::RightBrace => {
                if let Some(ParserContext::Object { .. }) = self.state_stack.pop() {
                    Ok(Some(StreamingEvent::EndObject))
                } else {
                    Err(Error::UnexpectedChar('}', self.position))
                }
            }
            Token::LeftBracket => {
                self.state_stack.push(ParserContext::Array {
                    first_element: true,
                });
                Ok(Some(StreamingEvent::StartArray))
            }
            Token::RightBracket => {
                if let Some(ParserContext::Array { .. }) = self.state_stack.pop() {
                    Ok(Some(StreamingEvent::EndArray))
                } else {
                    Err(Error::UnexpectedChar(']', self.position))
                }
            }
            Token::String => {
                let content = self.extract_string_content(token_str)?;

                // Check if this is an object key
                if let Some(ParserContext::Object { expecting_key }) = self.state_stack.last_mut() {
                    if *expecting_key {
                        *expecting_key = false;
                        Ok(Some(StreamingEvent::ObjectKey(content)))
                    } else {
                        Ok(Some(StreamingEvent::String(content)))
                    }
                } else {
                    Ok(Some(StreamingEvent::String(content)))
                }
            }
            Token::Number => {
                if self.config.preserve_number_precision {
                    Ok(Some(StreamingEvent::Number(token_str.to_string())))
                } else {
                    // Parse and reformat the number
                    Ok(Some(StreamingEvent::Number(token_str.to_string())))
                }
            }
            Token::True => Ok(Some(StreamingEvent::Bool(true))),
            Token::False => Ok(Some(StreamingEvent::Bool(false))),
            Token::Null => Ok(Some(StreamingEvent::Null)),
            Token::Colon => {
                // Colon doesn't generate an event but updates parser state
                if let Some(ParserContext::Object { expecting_key }) = self.state_stack.last_mut() {
                    *expecting_key = false; // Next value will be the object value, not a key
                }
                Ok(None)
            }
            Token::Comma => {
                // Comma doesn't generate an event but updates parser state
                match self.state_stack.last_mut() {
                    Some(ParserContext::Object { expecting_key }) => {
                        *expecting_key = true; // Next value will be a key
                    }
                    Some(ParserContext::Array { first_element }) => {
                        *first_element = false;
                    }
                    _ => {}
                }
                Ok(None)
            }
            Token::Newline => {
                // Newlines are generally ignored unless they serve as comma replacements
                if self.config.parser_options.newline_as_comma {
                    self.token_to_event(Token::Comma, token_str)
                } else {
                    Ok(None)
                }
            }
            Token::UnquotedString => {
                // Similar to string, but for unquoted identifiers
                if let Some(ParserContext::Object { expecting_key }) = self.state_stack.last_mut() {
                    if *expecting_key {
                        *expecting_key = false;
                        Ok(Some(StreamingEvent::ObjectKey(token_str.to_string())))
                    } else {
                        Ok(Some(StreamingEvent::String(token_str.to_string())))
                    }
                } else {
                    Ok(Some(StreamingEvent::String(token_str.to_string())))
                }
            }
            _ => Ok(None),
        }
    }

    /// Extracts the content from a quoted string token.
    fn extract_string_content(&self, token_str: &str) -> Result<String> {
        if token_str.len() < 2 {
            return Err(Error::UnterminatedString(self.position));
        }

        // Remove quotes
        let content = &token_str[1..token_str.len() - 1];

        // Basic unescaping (simplified)
        Ok(content
            .replace("\\\"", "\"")
            .replace("\\'", "'")
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r"))
    }

    /// Returns the current position in the input stream.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns whether the end of input has been reached.
    pub fn is_end_of_input(&self) -> bool {
        self.end_of_input
    }

    /// Returns the current parser state depth (nesting level).
    pub fn depth(&self) -> usize {
        self.state_stack.len()
    }

    /// Collects all remaining events into a vector.
    pub fn collect_events(&mut self) -> Result<Vec<StreamingEvent>> {
        let mut events = Vec::new();
        while let Some(event) = self.next_event()? {
            events.push(event);
        }
        Ok(events)
    }

    /// Parses the stream into a complete Value tree.
    pub fn parse_to_value(&mut self) -> Result<Value> {
        let mut value_stack: Vec<Value> = Vec::new();
        let mut key_stack: Vec<String> = Vec::new();

        while let Some(event) = self.next_event()? {
            match event {
                StreamingEvent::StartObject => {
                    value_stack.push(Value::Object(rustc_hash::FxHashMap::default()));
                }
                StreamingEvent::EndObject => {
                    if let Some(Value::Object(obj)) = value_stack.pop() {
                        if value_stack.is_empty() {
                            return Ok(Value::Object(obj));
                        }
                        // Add to parent structure
                        self.add_value_to_parent(
                            &mut value_stack,
                            &mut key_stack,
                            Value::Object(obj),
                        )?;
                    }
                }
                StreamingEvent::StartArray => {
                    value_stack.push(Value::Array(Vec::new()));
                }
                StreamingEvent::EndArray => {
                    if let Some(Value::Array(arr)) = value_stack.pop() {
                        if value_stack.is_empty() {
                            return Ok(Value::Array(arr));
                        }
                        // Add to parent structure
                        self.add_value_to_parent(
                            &mut value_stack,
                            &mut key_stack,
                            Value::Array(arr),
                        )?;
                    }
                }
                StreamingEvent::ObjectKey(key) => {
                    key_stack.push(key);
                }
                StreamingEvent::String(s) => {
                    let value = Value::String(s);
                    self.add_value_to_parent(&mut value_stack, &mut key_stack, value)?;
                }
                StreamingEvent::Number(n) => {
                    let value = if let Ok(int_val) = n.parse::<i64>() {
                        Value::Number(crate::ast::Number::Integer(int_val))
                    } else if let Ok(float_val) = n.parse::<f64>() {
                        Value::Number(crate::ast::Number::Float(float_val))
                    } else {
                        return Err(Error::InvalidNumber(self.position));
                    };
                    self.add_value_to_parent(&mut value_stack, &mut key_stack, value)?;
                }
                StreamingEvent::Bool(b) => {
                    let value = Value::Bool(b);
                    self.add_value_to_parent(&mut value_stack, &mut key_stack, value)?;
                }
                StreamingEvent::Null => {
                    let value = Value::Null;
                    self.add_value_to_parent(&mut value_stack, &mut key_stack, value)?;
                }
                StreamingEvent::EndOfInput => break,
            }
        }

        // Return the top-level value
        if let Some(value) = value_stack.pop() {
            Ok(value)
        } else {
            Err(Error::UnexpectedChar('\0', self.position))
        }
    }

    /// Helper to add a value to its parent container.
    fn add_value_to_parent(
        &self,
        value_stack: &mut Vec<Value>,
        key_stack: &mut Vec<String>,
        value: Value,
    ) -> Result<()> {
        if value_stack.is_empty() {
            return Ok(());
        }

        match value_stack.last_mut().unwrap() {
            Value::Object(ref mut obj) => {
                if let Some(key) = key_stack.pop() {
                    obj.insert(key, value);
                } else {
                    return Err(Error::UnexpectedChar('\0', self.position));
                }
            }
            Value::Array(ref mut arr) => {
                arr.push(value);
            }
            _ => return Err(Error::UnexpectedChar('\0', self.position)),
        }

        Ok(())
    }
}

/// Result of character processing during tokenization.
#[derive(Debug)]
#[allow(dead_code)]
enum TokenizeResult {
    /// Continue processing more characters
    Continue,
    /// A complete token has been formed
    TokenComplete(Token),
    /// Need more input data to complete the token
    NeedMoreData,
}

/// Iterator adapter for streaming events.
pub struct StreamingEventIterator<R: Read> {
    parser: BufferedStreamingParser<R>,
}

impl<R: Read> StreamingEventIterator<R> {
    /// Creates a new iterator from a buffered streaming parser.
    pub fn new(parser: BufferedStreamingParser<R>) -> Self {
        StreamingEventIterator { parser }
    }
}

impl<R: Read> Iterator for StreamingEventIterator<R> {
    type Item = Result<StreamingEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.next_event() {
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Creates a buffered streaming parser with default configuration.
pub fn parse_streaming<R: Read>(reader: R) -> BufferedStreamingParser<R> {
    BufferedStreamingParser::new(reader)
}

/// Creates a buffered streaming parser with custom configuration.
pub fn parse_streaming_with_config<R: Read>(
    reader: R,
    config: BufferedStreamingConfig,
) -> BufferedStreamingParser<R> {
    BufferedStreamingParser::with_config(reader, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_buffered_streaming_simple_object() {
        let json = r#"{"name": "test", "value": 42}"#;
        let reader = Cursor::new(json);
        let mut parser = BufferedStreamingParser::new(reader);

        let events = parser.collect_events().unwrap();

        assert!(events.contains(&StreamingEvent::StartObject));
        assert!(events.contains(&StreamingEvent::ObjectKey("name".to_string())));
        assert!(events.contains(&StreamingEvent::String("test".to_string())));
        assert!(events.contains(&StreamingEvent::ObjectKey("value".to_string())));
        assert!(events.contains(&StreamingEvent::Number("42".to_string())));
        assert!(events.contains(&StreamingEvent::EndObject));
    }

    #[test]
    fn test_buffered_streaming_array() {
        let json = r#"[1, 2, 3]"#;
        let reader = Cursor::new(json);
        let mut parser = BufferedStreamingParser::new(reader);

        let events = parser.collect_events().unwrap();

        assert!(events.contains(&StreamingEvent::StartArray));
        assert!(events.contains(&StreamingEvent::Number("1".to_string())));
        assert!(events.contains(&StreamingEvent::Number("2".to_string())));
        assert!(events.contains(&StreamingEvent::Number("3".to_string())));
        assert!(events.contains(&StreamingEvent::EndArray));
    }

    #[test]
    fn test_buffered_streaming_to_value() {
        let json = r#"{"items": [1, 2, 3], "count": 3}"#;
        let reader = Cursor::new(json);
        let mut parser = BufferedStreamingParser::new(reader);

        let value = parser.parse_to_value().unwrap();

        match value {
            Value::Object(obj) => {
                assert!(obj.contains_key("items"));
                assert!(obj.contains_key("count"));

                if let Some(Value::Array(arr)) = obj.get("items") {
                    assert_eq!(arr.len(), 3);
                }

                if let Some(Value::Number(crate::ast::Number::Integer(count))) = obj.get("count") {
                    assert_eq!(*count, 3);
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_custom_buffer_config() {
        let config = BufferedStreamingConfig {
            input_buffer_size: 4096,
            token_buffer_size: 256,
            event_buffer_size: 128,
            preserve_number_precision: false,
            parser_options: ParserOptions::default(),
        };

        let json = r#"{"test": 123.456}"#;
        let reader = Cursor::new(json);
        let mut parser = BufferedStreamingParser::with_config(reader, config);

        let value = parser.parse_to_value().unwrap();
        assert!(matches!(value, Value::Object(_)));
    }
}
