//! Modular lexer architecture for vexy_json parsing
//!
//! This module provides a trait-based lexer design that allows for
//! different lexer implementations optimized for various use cases:
//! - Fast lexer for production use
//! - Debug lexer with extensive logging
//! - Streaming lexer for incremental parsing

use crate::ast::Token;
use crate::error::{Result, Span};

/// Core lexer trait defining the interface for all lexer implementations
pub trait JsonLexer {
    /// Get the current position in the input
    fn position(&self) -> usize;

    /// Get the next token and its span
    fn next_token(&mut self) -> Result<(Token, Span)>;

    /// Peek at the next token without consuming it
    fn peek_token(&mut self) -> Result<&(Token, Span)>;

    /// Get the source text for a given span
    fn span_text(&self, span: &Span) -> &str;

    /// Get the current line and column position
    fn line_col(&self) -> (usize, usize);

    /// Check if we've reached the end of input
    fn is_eof(&self) -> bool;

    /// Get performance statistics (optional)
    fn stats(&self) -> LexerStats {
        LexerStats::default()
    }
}

/// Lexer performance statistics
#[derive(Debug, Default, Clone)]
pub struct LexerStats {
    /// Total tokens produced
    pub tokens_count: usize,
    /// Total bytes processed
    pub bytes_processed: usize,
    /// Time spent lexing (nanoseconds)
    pub time_ns: u64,
    /// Number of errors encountered
    pub errors_count: usize,
}

/// Lexer mode for different parsing scenarios
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerMode {
    /// Default mode with standard JSON tokenization
    Standard,
    /// Strict mode that rejects any non-standard JSON
    Strict,
    /// Forgiving mode with all extensions enabled
    Forgiving,
    /// Streaming mode optimized for incremental parsing
    Streaming,
}

/// Configuration for lexer behavior
#[derive(Debug, Clone)]
pub struct LexerConfig {
    /// Lexer mode
    pub mode: LexerMode,
    /// Maximum nesting depth
    pub max_depth: usize,
    /// Track line/column positions
    pub track_positions: bool,
    /// Collect performance statistics
    pub collect_stats: bool,
    /// Buffer size for streaming mode
    pub buffer_size: usize,
}

impl Default for LexerConfig {
    fn default() -> Self {
        LexerConfig {
            mode: LexerMode::Forgiving,
            max_depth: 128,
            track_positions: true,
            collect_stats: false,
            buffer_size: 8192,
        }
    }
}

// Re-export lexer implementations
pub mod debug_lexer;
pub mod fast_lexer;
pub mod logos_lexer;

pub use debug_lexer::DebugLexer;
pub use fast_lexer::FastLexer;
pub use logos_lexer::{Lexer, LogosLexer};

/// Create a lexer based on configuration
pub fn create_lexer<'a>(input: &'a str, config: LexerConfig) -> Box<dyn JsonLexer + 'a> {
    match config.mode {
        LexerMode::Standard | LexerMode::Forgiving => {
            if config.collect_stats || config.track_positions {
                Box::new(DebugLexer::new(input, config))
            } else {
                Box::new(FastLexer::new(input, config))
            }
        }
        LexerMode::Strict => Box::new(LogosLexer::new(input)),
        LexerMode::Streaming => Box::new(FastLexer::new(input, config)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_creation() {
        let input = r#"{"key": "value"}"#;

        // Test default configuration
        let config = LexerConfig::default();
        let lexer = create_lexer(input, config);
        assert_eq!(lexer.position(), 0);

        // Test strict mode
        let strict_config = LexerConfig {
            mode: LexerMode::Strict,
            ..Default::default()
        };
        let strict_lexer = create_lexer(input, strict_config);
        assert!(!strict_lexer.is_eof());
    }
}
