// this_file: src/streaming/ndjson.rs

//! Newline-Delimited JSON (NDJSON) support for streaming parser.
//!
//! NDJSON is a format where each line is a valid JSON value, allowing
//! for streaming of multiple JSON objects without wrapping them in an array.

use super::{StreamingEvent, StreamingParser};
use crate::ast::Value;
use crate::error::{Error, Result};
use crate::parser::ParserOptions;

/// Parser for Newline-Delimited JSON streams
pub struct NdJsonParser {
    /// Line buffer
    line_buffer: String,
    /// Parser options
    #[allow(dead_code)]
    options: ParserOptions,
    /// Whether we've reached the end
    finished: bool,
    /// Current line number for error reporting
    line_number: usize,
}

impl NdJsonParser {
    /// Create a new NDJSON parser with default options
    pub fn new() -> Self {
        Self::with_options(ParserOptions::default())
    }

    /// Create a new NDJSON parser with custom options
    pub fn with_options(options: ParserOptions) -> Self {
        Self {
            line_buffer: String::new(),
            options,
            finished: false,
            line_number: 1,
        }
    }

    /// Feed a chunk of input to the parser
    pub fn feed(&mut self, chunk: &str) -> Result<Vec<Value>> {
        if self.finished {
            return Err(Error::Custom("Parser already finished".to_string()));
        }

        let mut results = Vec::new();

        for ch in chunk.chars() {
            if ch == '\n' {
                // Process complete line
                if !self.line_buffer.trim().is_empty() {
                    match self.parse_line(&self.line_buffer) {
                        Ok(value) => results.push(value),
                        Err(e) => {
                            return Err(Error::Custom(format!(
                                "Error on line {}: {}",
                                self.line_number, e
                            )));
                        }
                    }
                }
                self.line_buffer.clear();
                self.line_number += 1;
            } else {
                self.line_buffer.push(ch);
            }
        }

        Ok(results)
    }

    /// Parse a single line as JSON
    fn parse_line(&self, line: &str) -> Result<Value> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(Error::Custom("Empty line".to_string()));
        }

        // Use the regular parser for the line
        #[cfg(feature = "serde")]
        {
            crate::parse_with_options(trimmed, self.options.clone())
        }
        #[cfg(not(feature = "serde"))]
        {
            crate::parse(trimmed)
        }
    }

    /// Signal end of input and process any remaining buffer
    pub fn finish(&mut self) -> Result<Option<Value>> {
        self.finished = true;

        if !self.line_buffer.trim().is_empty() {
            Ok(Some(self.parse_line(&self.line_buffer)?))
        } else {
            Ok(None)
        }
    }

    /// Check if the parser has finished
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Get the current line number
    #[inline(always)]
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

/// Streaming NDJSON parser that emits events for each line
pub struct StreamingNdJsonParser {
    /// Line buffer
    line_buffer: String,
    /// Event queue for current line
    event_queue: Vec<StreamingEvent>,
    /// Parser options
    options: ParserOptions,
    /// Whether we've reached the end
    finished: bool,
    /// Current line number
    line_number: usize,
}

impl StreamingNdJsonParser {
    /// Create a new streaming NDJSON parser
    pub fn new() -> Self {
        Self::with_options(ParserOptions::default())
    }

    /// Create a new streaming NDJSON parser with custom options
    pub fn with_options(options: ParserOptions) -> Self {
        Self {
            line_buffer: String::new(),
            event_queue: Vec::new(),
            options,
            finished: false,
            line_number: 1,
        }
    }

    /// Feed a chunk of input
    pub fn feed(&mut self, chunk: &str) -> Result<()> {
        if self.finished {
            return Err(Error::Custom("Parser already finished".to_string()));
        }

        for ch in chunk.chars() {
            if ch == '\n' {
                // Process complete line
                if !self.line_buffer.trim().is_empty() {
                    self.start_line_parsing()?;
                }
                self.line_buffer.clear();
                self.line_number += 1;
            } else {
                self.line_buffer.push(ch);
            }
        }

        Ok(())
    }

    /// Start parsing a complete line
    fn start_line_parsing(&mut self) -> Result<()> {
        let mut parser = StreamingParser::with_options(self.options.clone());
        parser.feed(&self.line_buffer)?;
        parser.finish()?;

        // Collect all events from this line
        let mut line_events = Vec::new();
        while let Some(event) = parser.next_event()? {
            if matches!(event, StreamingEvent::EndOfInput) {
                break;
            }
            line_events.push(event);
        }

        // Add line separator event
        line_events.push(StreamingEvent::EndOfInput);

        self.event_queue.extend(line_events);
        Ok(())
    }

    /// Get the next event
    pub fn next_event(&mut self) -> Result<Option<StreamingEvent>> {
        if !self.event_queue.is_empty() {
            Ok(Some(self.event_queue.remove(0)))
        } else if self.finished {
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// Signal end of input
    pub fn finish(&mut self) -> Result<()> {
        if !self.line_buffer.trim().is_empty() {
            self.start_line_parsing()?;
        }
        self.finished = true;
        Ok(())
    }

    /// Check if the parser has finished
    pub fn is_finished(&self) -> bool {
        self.finished && self.event_queue.is_empty()
    }

    /// Get the current line number
    #[inline(always)]
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

/// Iterator interface for NDJSON values
pub struct NdJsonIterator {
    parser: NdJsonParser,
    buffer: String,
    input: Box<dyn Iterator<Item = std::result::Result<String, std::io::Error>>>,
}

impl NdJsonIterator {
    /// Create a new NDJSON iterator from a line iterator
    pub fn new<I>(input: I) -> Self
    where
        I: Iterator<Item = std::result::Result<String, std::io::Error>> + 'static,
    {
        Self {
            parser: NdJsonParser::new(),
            buffer: String::new(),
            input: Box::new(input),
        }
    }

    /// Create a new NDJSON iterator with custom options
    pub fn with_options<I>(input: I, options: ParserOptions) -> Self
    where
        I: Iterator<Item = std::result::Result<String, std::io::Error>> + 'static,
    {
        Self {
            parser: NdJsonParser::with_options(options),
            buffer: String::new(),
            input: Box::new(input),
        }
    }
}

impl Iterator for NdJsonIterator {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.input.next() {
                Some(Ok(line)) => {
                    self.buffer = line;
                    self.buffer.push('\n');

                    match self.parser.feed(&self.buffer) {
                        Ok(values) => {
                            if !values.is_empty() {
                                return Some(Ok(values.into_iter().next().unwrap()));
                            }
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Err(e)) => {
                    return Some(Err(Error::Custom(format!("IO error: {}", e))));
                }
                None => {
                    // End of input
                    match self.parser.finish() {
                        Ok(Some(value)) => return Some(Ok(value)),
                        Ok(None) => return None,
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
        }
    }
}

impl Default for NdJsonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StreamingNdJsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndjson_parser() {
        let mut parser = NdJsonParser::new();

        let input = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}"#;

        let values = parser.feed(input).unwrap();
        assert_eq!(values.len(), 3);

        // Check first value
        if let Value::Object(obj) = &values[0] {
            assert_eq!(
                obj.get("name").unwrap(),
                &Value::String("Alice".to_string())
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_streaming_ndjson() {
        let mut parser = StreamingNdJsonParser::new();

        parser
            .feed(
                r#"{"key": "value1"}
{"key": "value2"}"#,
            )
            .unwrap();
        parser.finish().unwrap();

        let mut events = Vec::new();
        while let Some(event) = parser.next_event().unwrap() {
            events.push(event);
        }

        // Should have events for two complete JSON objects
        assert!(!events.is_empty());
    }

    #[test]
    fn test_empty_lines() {
        let mut parser = NdJsonParser::new();

        let input = r#"{"valid": true}

{"also": "valid"}"#;

        let values = parser.feed(input).unwrap();
        assert_eq!(values.len(), 2);
    }
}
