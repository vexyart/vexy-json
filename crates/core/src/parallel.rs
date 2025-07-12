// this_file: crates/core/src/parallel.rs

//! Parallel parsing for large JSON files
//!
//! This module provides parallel parsing capabilities for large JSON files by:
//! - Splitting the input into logical chunks at JSON boundaries
//! - Parsing chunks in parallel using Rayon
//! - Merging results efficiently
//!
//! The parallel parser can handle:
//! - Large single JSON objects/arrays
//! - NDJSON (newline-delimited JSON) files
//! - JSON arrays with many elements

use crate::{
    ast::Value,
    error::{Error, Result},
    parse,
};
use rayon::prelude::*;

/// Configuration for parallel parsing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Minimum chunk size in bytes before enabling parallel parsing
    pub min_chunk_size: usize,
    /// Maximum number of parallel threads to use (0 = auto)
    pub max_threads: usize,
    /// Whether to enable chunk optimization (may use more memory)
    pub optimize_chunks: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 64 * 1024, // 64KB
            max_threads: 0,            // Auto-detect
            optimize_chunks: true,
        }
    }
}

/// Parallel JSON parser
pub struct ParallelParser {
    config: ParallelConfig,
}

impl ParallelParser {
    /// Create a new parallel parser with default configuration
    pub fn new() -> Self {
        Self {
            config: ParallelConfig::default(),
        }
    }

    /// Create a parallel parser with custom configuration
    pub fn with_config(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Parse a large JSON input using parallel processing
    pub fn parse(&self, input: &str) -> Result<Value> {
        // For small inputs, use regular parsing
        if input.len() < self.config.min_chunk_size {
            return parse(input);
        }

        // Detect input type and choose appropriate strategy
        match self.detect_input_type(input)? {
            InputType::Array => self.parse_large_array(input),
            InputType::Object => self.parse_large_object(input),
            InputType::NdJson => {
                let values = self.parse_ndjson(input)?;
                // For consistency, return a single array containing all NDJSON objects
                Ok(Value::Array(values))
            }
            InputType::Single => parse(input),
        }
    }

    /// Parse NDJSON (newline-delimited JSON) in parallel
    pub fn parse_ndjson(&self, input: &str) -> Result<Vec<Value>> {
        let lines: Vec<&str> = input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        if lines.len() < 2 {
            // Single line, parse normally
            let value = parse(input)?;
            return Ok(vec![value]);
        }

        // Configure thread pool if specified
        let result = if self.config.max_threads > 0 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(self.config.max_threads)
                .build()
                .map_err(|e| Error::Custom(format!("Failed to create thread pool: {e}")))?;

            pool.install(|| {
                lines
                    .par_iter()
                    .map(|line| parse(line))
                    .collect::<Result<Vec<_>>>()
            })
        } else {
            lines
                .par_iter()
                .map(|line| parse(line))
                .collect::<Result<Vec<_>>>()
        };

        result
    }

    /// Detect the type of JSON input
    fn detect_input_type(&self, input: &str) -> Result<InputType> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(Error::Custom("Empty input".to_string()));
        }

        // Check for NDJSON (multiple lines with JSON objects)
        if input.lines().filter(|line| !line.trim().is_empty()).count() > 1 {
            let first_line = input
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("");
            if first_line.trim_start().starts_with('{') {
                return Ok(InputType::NdJson);
            }
        }

        // Check first character to determine type
        match trimmed.chars().next().unwrap() {
            '[' => Ok(InputType::Array),
            '{' => Ok(InputType::Object),
            _ => Ok(InputType::Single),
        }
    }

    /// Parse a large JSON array in parallel
    fn parse_large_array(&self, input: &str) -> Result<Value> {
        let chunks = self.split_array_into_chunks(input)?;

        if chunks.len() <= 1 {
            // Single chunk, parse normally
            return parse(input);
        }

        // Parse chunks in parallel
        let parsed_chunks: Result<Vec<Vec<Value>>> = if self.config.max_threads > 0 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(self.config.max_threads)
                .build()
                .map_err(|e| Error::Custom(format!("Failed to create thread pool: {e}")))?;

            pool.install(|| {
                chunks
                    .par_iter()
                    .map(|chunk| self.parse_array_chunk(chunk))
                    .collect()
            })
        } else {
            chunks
                .par_iter()
                .map(|chunk| self.parse_array_chunk(chunk))
                .collect()
        };

        let chunks = parsed_chunks?;

        // Merge results
        let mut merged = Vec::new();
        for chunk in chunks {
            merged.extend(chunk);
        }

        Ok(Value::Array(merged))
    }

    /// Parse a large JSON object in parallel by parsing nested objects/arrays in parallel
    fn parse_large_object(&self, input: &str) -> Result<Value> {
        // For now, parse objects normally since they're harder to parallelize
        // In the future, we could parallelize parsing of large nested arrays/objects
        parse(input)
    }

    /// Split a JSON array into chunks that can be parsed in parallel
    fn split_array_into_chunks(&self, input: &str) -> Result<Vec<String>> {
        let trimmed = input.trim();

        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(Error::Custom("Input is not a JSON array".to_string()));
        }

        // Remove outer brackets
        let content = &trimmed[1..trimmed.len() - 1].trim();
        if content.is_empty() {
            return Ok(vec!["[]".to_string()]);
        }

        let elements = self.split_array_elements(content)?;
        let target_chunk_size = self.config.min_chunk_size;
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_size = 0;

        for element in elements {
            let element_size = element.len();

            if current_size + element_size > target_chunk_size && !current_chunk.is_empty() {
                // Create chunk
                chunks.push(format!("[{}]", current_chunk.join(",")));
                current_chunk.clear();
                current_size = 0;
            }

            current_chunk.push(element);
            current_size += element_size;
        }

        // Add remaining elements
        if !current_chunk.is_empty() {
            chunks.push(format!("[{}]", current_chunk.join(",")));
        }

        if chunks.is_empty() {
            chunks.push("[]".to_string());
        }

        Ok(chunks)
    }

    /// Split array content into individual elements
    fn split_array_elements(&self, content: &str) -> Result<Vec<String>> {
        let mut elements = Vec::new();
        let mut current_element = String::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let chars = content.chars().peekable();

        for ch in chars {
            if escape_next {
                current_element.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    current_element.push(ch);
                    escape_next = true;
                }
                '"' => {
                    current_element.push(ch);
                    in_string = !in_string;
                }
                '{' | '[' if !in_string => {
                    current_element.push(ch);
                    depth += 1;
                }
                '}' | ']' if !in_string => {
                    current_element.push(ch);
                    depth -= 1;
                }
                ',' if !in_string && depth == 0 => {
                    // End of element
                    let element = current_element.trim();
                    if !element.is_empty() {
                        elements.push(element.to_string());
                    }
                    current_element.clear();
                }
                _ => {
                    current_element.push(ch);
                }
            }
        }

        // Add the last element
        let element = current_element.trim();
        if !element.is_empty() {
            elements.push(element.to_string());
        }

        Ok(elements)
    }

    /// Parse a chunk of array elements
    fn parse_array_chunk(&self, chunk: &str) -> Result<Vec<Value>> {
        let parsed = parse(chunk)?;
        match parsed {
            Value::Array(elements) => Ok(elements),
            single => Ok(vec![single]),
        }
    }
}

impl Default for ParallelParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of JSON input detected
#[derive(Debug, Clone, PartialEq)]
enum InputType {
    /// Single JSON value
    Single,
    /// JSON array
    Array,
    /// JSON object
    Object,
    /// Newline-delimited JSON
    NdJson,
}

/// Convenience function for parallel parsing with default settings
pub fn parse_parallel(input: &str) -> Result<Value> {
    ParallelParser::new().parse(input)
}

/// Convenience function for parallel NDJSON parsing
pub fn parse_ndjson_parallel(input: &str) -> Result<Vec<Value>> {
    ParallelParser::new().parse_ndjson(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_input_fallback() {
        let parser = ParallelParser::new();
        let input = r#"{"test": true}"#;
        let result = parser.parse(input).unwrap();

        match result {
            Value::Object(_) => (),
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_ndjson_parsing() {
        let parser = ParallelParser::new();
        let input = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}"#;

        let results = parser.parse_ndjson(input).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_array_chunking() {
        let parser = ParallelParser::new();
        let input = r#"[1, 2, 3, 4, 5]"#;

        let chunks = parser.split_array_into_chunks(input).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_element_splitting() {
        let parser = ParallelParser::new();
        let content = r#"1, "hello", {"key": "value"}, [1, 2, 3]"#;

        let elements = parser.split_array_elements(content).unwrap();
        assert_eq!(elements.len(), 4);
        assert_eq!(elements[0], "1");
        assert_eq!(elements[1], r#""hello""#);
        assert_eq!(elements[2], r#"{"key": "value"}"#);
        assert_eq!(elements[3], "[1, 2, 3]");
    }

    #[test]
    fn test_input_type_detection() {
        let parser = ParallelParser::new();

        assert_eq!(
            parser.detect_input_type(r#"{"test": true}"#).unwrap(),
            InputType::Object
        );
        assert_eq!(
            parser.detect_input_type("[1, 2, 3]").unwrap(),
            InputType::Array
        );
        assert_eq!(parser.detect_input_type("42").unwrap(), InputType::Single);
        assert_eq!(
            parser
                .detect_input_type(
                    r#"{"a": 1}
{"b": 2}"#
                )
                .unwrap(),
            InputType::NdJson
        );
    }
}
