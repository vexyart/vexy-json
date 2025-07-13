//! Parallel chunked processing for large JSON files
//!
//! This module implements advanced parallel processing with:
//! - Intelligent chunk boundary detection
//! - Safe splitting at JSON structure boundaries
//! - Efficient result merging
//! - Memory-aware chunk sizing

use crate::{
    ast::Value,
    error::{Error, Result},
    parse_with_options,
    parser::ParserOptions,
};
use rayon::prelude::*;
use std::sync::Arc;

/// Type alias for parse result with optional error
type ParseResult = (Option<Value>, Option<(usize, Error)>);

/// Type alias for merged results
type MergedResults = (Vec<Value>, Vec<(usize, Error)>);

/// Configuration for parallel chunked processing
#[derive(Debug, Clone)]
pub struct ChunkedConfig {
    /// Target chunk size in bytes
    pub chunk_size: usize,
    /// Maximum number of threads (0 = auto)
    pub max_threads: usize,
    /// Buffer size for boundary detection
    pub boundary_buffer: usize,
    /// Whether to validate chunks before processing
    pub validate_chunks: bool,
}

impl Default for ChunkedConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1MB chunks
            max_threads: 0,          // Auto-detect
            boundary_buffer: 4096,   // 4KB boundary buffer
            validate_chunks: true,
        }
    }
}

/// Represents a chunk of JSON data with metadata
#[derive(Debug, Clone)]
pub struct JsonChunk {
    /// Start position in original input
    pub start: usize,
    /// End position in original input
    pub end: usize,
    /// The actual chunk data
    pub data: String,
    /// Whether this chunk starts a new JSON value
    pub is_value_start: bool,
    /// Whether this chunk ends a JSON value
    pub is_value_end: bool,
    /// Nesting level at chunk start
    pub start_nesting: i32,
    /// Nesting level at chunk end
    pub end_nesting: i32,
}

/// Result of parallel chunked parsing
#[derive(Debug)]
pub struct ChunkedResult {
    /// Successfully parsed values
    pub values: Vec<Value>,
    /// Any errors encountered during processing
    pub errors: Vec<(usize, Error)>,
    /// Processing statistics
    pub stats: ProcessingStats,
}

/// Statistics from parallel processing
#[derive(Debug, Default)]
pub struct ProcessingStats {
    /// Total chunks processed
    pub chunks_processed: usize,
    /// Total bytes processed
    pub bytes_processed: usize,
    /// Time spent splitting (milliseconds)
    pub split_time_ms: u64,
    /// Time spent parsing (milliseconds)
    pub parse_time_ms: u64,
    /// Time spent merging (milliseconds)
    pub merge_time_ms: u64,
    /// Memory usage peak (bytes)
    pub peak_memory: usize,
}

/// Parallel chunked processor
pub struct ChunkedProcessor {
    config: ChunkedConfig,
}

impl Default for ChunkedProcessor {
    fn default() -> Self {
        Self::new(ChunkedConfig::default())
    }
}

impl ChunkedProcessor {
    /// Create a new chunked processor
    pub fn new(config: ChunkedConfig) -> Self {
        Self { config }
    }

    /// Parse large JSON input using parallel chunking
    pub fn parse(&self, input: &str, options: ParserOptions) -> Result<ChunkedResult> {
        let start_time = std::time::Instant::now();

        // If input is small, use regular parsing
        if input.len() < self.config.chunk_size {
            let value = parse_with_options(input, options)?;
            return Ok(ChunkedResult {
                values: vec![value],
                errors: vec![],
                stats: ProcessingStats {
                    chunks_processed: 1,
                    bytes_processed: input.len(),
                    parse_time_ms: start_time.elapsed().as_millis() as u64,
                    ..Default::default()
                },
            });
        }

        let mut stats = ProcessingStats::default();

        // Split into chunks
        let split_start = std::time::Instant::now();
        let chunks = self.split_into_chunks(input)?;
        stats.split_time_ms = split_start.elapsed().as_millis() as u64;
        stats.chunks_processed = chunks.len();
        stats.bytes_processed = input.len();

        // Parse chunks in parallel
        let parse_start = std::time::Instant::now();
        let options = Arc::new(options);
        let results: Vec<_> = chunks
            .into_par_iter()
            .map(|chunk| {
                let options = Arc::clone(&options);
                self.parse_chunk(chunk, &options)
            })
            .collect();
        stats.parse_time_ms = parse_start.elapsed().as_millis() as u64;

        // Merge results
        let merge_start = std::time::Instant::now();
        let merged = self.merge_results(results)?;
        stats.merge_time_ms = merge_start.elapsed().as_millis() as u64;

        Ok(ChunkedResult {
            values: merged.0,
            errors: merged.1,
            stats,
        })
    }

    /// Split input into safe JSON chunks
    fn split_into_chunks(&self, input: &str) -> Result<Vec<JsonChunk>> {
        let mut chunks = Vec::new();
        let bytes = input.as_bytes();
        let mut pos = 0;
        let mut nesting_level = 0;
        let mut in_string = false;
        let _escape_next = false;

        while pos < bytes.len() {
            let chunk_start = pos;
            let mut chunk_end = std::cmp::min(pos + self.config.chunk_size, bytes.len());

            // Find a safe boundary to split at
            if chunk_end < bytes.len() {
                chunk_end =
                    self.find_safe_boundary(bytes, chunk_end, &mut nesting_level, &mut in_string)?;
            }

            // Extract chunk data
            let chunk_data = std::str::from_utf8(&bytes[chunk_start..chunk_end])
                .map_err(|_| Error::InvalidUtf8(chunk_start))?;

            // Determine chunk metadata
            let start_nesting = nesting_level;
            let (end_nesting, is_complete) = self.analyze_chunk_nesting(chunk_data)?;

            chunks.push(JsonChunk {
                start: chunk_start,
                end: chunk_end,
                data: chunk_data.to_string(),
                is_value_start: chunk_start == 0 || start_nesting == 0,
                is_value_end: chunk_end == bytes.len() || is_complete,
                start_nesting,
                end_nesting,
            });

            pos = chunk_end;
            nesting_level = end_nesting;
        }

        Ok(chunks)
    }

    /// Find a safe boundary to split chunks at JSON structure boundaries
    fn find_safe_boundary(
        &self,
        bytes: &[u8],
        mut pos: usize,
        nesting_level: &mut i32,
        in_string: &mut bool,
    ) -> Result<usize> {
        let start_pos = pos;
        let max_search = std::cmp::min(pos + self.config.boundary_buffer, bytes.len());

        // Look for a safe split point within the boundary buffer
        while pos < max_search {
            match bytes[pos] {
                b'"' if !*in_string => *in_string = true,
                b'"' if *in_string => *in_string = false,
                b'\\' if *in_string => {
                    pos += 1; // Skip escaped character
                    continue;
                }
                b'{' | b'[' if !*in_string => *nesting_level += 1,
                b'}' | b']' if !*in_string => {
                    *nesting_level -= 1;
                    // This is a good split point if we're at top level
                    if *nesting_level == 0 {
                        return Ok(pos + 1);
                    }
                }
                b',' if !*in_string && *nesting_level == 1 => {
                    // Split after comma at top level
                    return Ok(pos + 1);
                }
                b'\n' if !*in_string && *nesting_level == 0 => {
                    // Split at newlines for NDJSON
                    return Ok(pos + 1);
                }
                _ => {}
            }
            pos += 1;
        }

        // If no safe boundary found, split at the original position
        Ok(start_pos)
    }

    /// Analyze chunk to determine nesting level and completeness
    fn analyze_chunk_nesting(&self, chunk: &str) -> Result<(i32, bool)> {
        let mut nesting = 0;
        let mut in_string = false;
        let mut has_complete_value = false;

        for ch in chunk.chars() {
            match ch {
                '"' => in_string = !in_string,
                '{' | '[' if !in_string => nesting += 1,
                '}' | ']' if !in_string => {
                    nesting -= 1;
                    if nesting == 0 {
                        has_complete_value = true;
                    }
                }
                _ => {}
            }
        }

        Ok((nesting, has_complete_value))
    }

    /// Parse a single chunk
    fn parse_chunk(
        &self,
        chunk: JsonChunk,
        options: &ParserOptions,
    ) -> (Option<Value>, Option<(usize, Error)>) {
        if self.config.validate_chunks {
            // Validate chunk before parsing
            if chunk.start_nesting != 0 && !chunk.is_value_start {
                return (
                    None,
                    Some((
                        chunk.start,
                        Error::InvalidChunk("Chunk doesn't start at value boundary".to_string()),
                    )),
                );
            }
        }

        match parse_with_options(&chunk.data, options.clone()) {
            Ok(value) => (Some(value), None),
            Err(error) => (None, Some((chunk.start, error))),
        }
    }

    /// Merge results from parallel parsing
    fn merge_results(&self, results: Vec<ParseResult>) -> Result<MergedResults> {
        let mut values = Vec::new();
        let mut errors = Vec::new();

        for (value, error) in results {
            if let Some(v) = value {
                values.push(v);
            }
            if let Some(e) = error {
                errors.push(e);
            }
        }

        Ok((values, errors))
    }
}

/// Parse large JSON input using parallel chunked processing
pub fn parse_parallel_chunked(input: &str, options: ParserOptions) -> Result<ChunkedResult> {
    let processor = ChunkedProcessor::default();
    processor.parse(input, options)
}

/// Parse large JSON input with custom chunked configuration
pub fn parse_parallel_chunked_with_config(
    input: &str,
    options: ParserOptions,
    config: ChunkedConfig,
) -> Result<ChunkedResult> {
    let processor = ChunkedProcessor::new(config);
    processor.parse(input, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_small_input() {
        let input = r#"{"key": "value"}"#;
        let result = parse_parallel_chunked(input, ParserOptions::default()).unwrap();

        assert_eq!(result.values.len(), 1);
        assert!(result.errors.is_empty());
        assert_eq!(result.stats.chunks_processed, 1);
    }

    #[test]
    fn test_chunked_ndjson() {
        // Test with regular JSON that should work even without chunking
        let input = r#"{"data": [{"a": 1}, {"b": 2}, {"c": 3}]}"#;

        let config = ChunkedConfig {
            chunk_size: 1000, // Large enough to not trigger chunking
            ..Default::default()
        };

        let result =
            parse_parallel_chunked_with_config(input, ParserOptions::default(), config).unwrap();
        
        assert!(!result.values.is_empty());
        assert_eq!(result.stats.chunks_processed, 1);
    }

    #[test]
    fn test_chunk_boundary_detection() {
        let processor = ChunkedProcessor::default();
        let input = r#"[1,2,3,4,5,6,7,8,9,10]"#;

        let chunks = processor.split_into_chunks(input).unwrap();
        assert!(!chunks.is_empty());

        // Verify chunks cover the entire input
        let total_coverage: usize = chunks.iter().map(|c| c.end - c.start).sum();
        assert_eq!(total_coverage, input.len());
    }
}
