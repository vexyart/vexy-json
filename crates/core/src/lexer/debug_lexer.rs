//! Debug lexer implementation with extensive logging
//!
//! This lexer provides detailed debugging information including:
//! - Token flow tracing
//! - Position tracking
//! - Performance metrics
//! - Error context

use crate::ast::Token;
use crate::error::{Result, Span};
use crate::lexer::{JsonLexer, LexerConfig, LexerStats, LogosLexer};
use std::fmt::Write;
use std::time::Instant;

/// Debug lexer that wraps another lexer and adds logging
pub struct DebugLexer<'a> {
    /// Inner lexer (usually LogosLexer)
    inner: LogosLexer<'a>,
    /// Configuration
    config: LexerConfig,
    /// Debug log
    log: Vec<DebugEvent>,
    /// Statistics
    stats: LexerStats,
    /// Start time
    start_time: Instant,
    /// Token count for debugging
    token_count: usize,
}

/// Debug event for logging
#[derive(Debug, Clone)]
enum DebugEvent {
    TokenProduced {
        token: Token,
        span: Span,
        text: String,
        line_col: (usize, usize),
        timestamp_ns: u64,
    },
    Error {
        error: String,
        position: usize,
        timestamp_ns: u64,
    },
    #[allow(dead_code)]
    StateChange {
        description: String,
        timestamp_ns: u64,
    },
}

impl<'a> DebugLexer<'a> {
    /// Create a new debug lexer
    pub fn new(input: &'a str, config: LexerConfig) -> Self {
        DebugLexer {
            inner: LogosLexer::new(input),
            config,
            log: Vec::new(),
            stats: LexerStats::default(),
            start_time: Instant::now(),
            token_count: 0,
        }
    }

    /// Log a debug event
    fn log_event(&mut self, event: DebugEvent) {
        self.log.push(event);
    }

    /// Get elapsed time in nanoseconds
    fn elapsed_ns(&self) -> u64 {
        self.start_time.elapsed().as_nanos() as u64
    }

    /// Format the debug log as a string
    pub fn format_log(&self) -> String {
        let mut output = String::new();

        writeln!(&mut output, "=== DebugLexer Log ===").unwrap();
        writeln!(&mut output, "Total tokens: {}", self.token_count).unwrap();
        writeln!(&mut output, "Total time: {}ns", self.elapsed_ns()).unwrap();
        writeln!(&mut output, "\nEvents:").unwrap();

        for (i, event) in self.log.iter().enumerate() {
            match event {
                DebugEvent::TokenProduced {
                    token,
                    span,
                    text,
                    line_col,
                    timestamp_ns,
                } => {
                    writeln!(
                        &mut output,
                        "[{:4}] {:>8}ns | Token: {:?} at {}:{} (span: {}-{}) text: {:?}",
                        i, timestamp_ns, token, line_col.0, line_col.1, span.start, span.end, text
                    )
                    .unwrap();
                }
                DebugEvent::Error {
                    error,
                    position,
                    timestamp_ns,
                } => {
                    writeln!(
                        &mut output,
                        "[{:4}] {:>8}ns | ERROR at {}: {}",
                        i, timestamp_ns, position, error
                    )
                    .unwrap();
                }
                DebugEvent::StateChange {
                    description,
                    timestamp_ns,
                } => {
                    writeln!(
                        &mut output,
                        "[{:4}] {:>8}ns | State: {}",
                        i, timestamp_ns, description
                    )
                    .unwrap();
                }
            }
        }

        writeln!(&mut output, "\n=== Statistics ===").unwrap();
        writeln!(&mut output, "Tokens produced: {}", self.stats.tokens_count).unwrap();
        writeln!(
            &mut output,
            "Bytes processed: {}",
            self.stats.bytes_processed
        )
        .unwrap();
        writeln!(
            &mut output,
            "Errors encountered: {}",
            self.stats.errors_count
        )
        .unwrap();
        writeln!(
            &mut output,
            "Average ns/token: {}",
            if self.stats.tokens_count > 0 {
                self.elapsed_ns() / self.stats.tokens_count as u64
            } else {
                0
            }
        )
        .unwrap();

        output
    }

    /// Print the debug log to stderr
    pub fn print_log(&self) {
        eprintln!("{}", self.format_log());
    }
}

impl<'a> JsonLexer for DebugLexer<'a> {
    fn position(&self) -> usize {
        self.inner.position()
    }

    fn next_token(&mut self) -> Result<(Token, Span)> {
        let start_ns = self.elapsed_ns();

        match self.inner.next_token() {
            Ok((token, span)) => {
                self.token_count += 1;
                self.stats.tokens_count += 1;
                self.stats.bytes_processed = span.end;

                let text = self.inner.span_text(&span).to_string();
                let line_col = self.inner.line_col();

                self.log_event(DebugEvent::TokenProduced {
                    token,
                    span,
                    text,
                    line_col,
                    timestamp_ns: self.elapsed_ns() - start_ns,
                });

                Ok((token, span))
            }
            Err(e) => {
                self.stats.errors_count += 1;

                self.log_event(DebugEvent::Error {
                    error: e.to_string(),
                    position: self.inner.position(),
                    timestamp_ns: self.elapsed_ns() - start_ns,
                });

                Err(e)
            }
        }
    }

    fn peek_token(&mut self) -> Result<&(Token, Span)> {
        self.inner.peek_token()
    }

    fn span_text(&self, span: &Span) -> &str {
        self.inner.span_text(span)
    }

    fn line_col(&self) -> (usize, usize) {
        self.inner.line_col()
    }

    fn is_eof(&self) -> bool {
        self.inner.is_eof()
    }

    fn stats(&self) -> LexerStats {
        let mut stats = self.stats.clone();
        stats.time_ns = self.elapsed_ns();
        stats
    }
}

impl<'a> Drop for DebugLexer<'a> {
    fn drop(&mut self) {
        if self.config.collect_stats && !self.log.is_empty() {
            eprintln!("\n=== DebugLexer Final Report ===");
            eprintln!(
                "Processed {} tokens in {}µs",
                self.stats.tokens_count,
                self.elapsed_ns() / 1000
            );
            if self.stats.errors_count > 0 {
                eprintln!("Encountered {} errors", self.stats.errors_count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::LexerMode;

    #[test]
    fn test_debug_lexer_logging() {
        let config = LexerConfig {
            mode: LexerMode::Standard,
            collect_stats: true,
            ..Default::default()
        };

        let mut lexer = DebugLexer::new(r#"{"debug": true}"#, config);

        // Consume all tokens
        while !lexer.is_eof() {
            let _ = lexer.next_token();
        }

        let log = lexer.format_log();

        // Verify log contains expected elements
        assert!(log.contains("Token: LeftBrace"));
        assert!(log.contains("Token: String"));
        assert!(log.contains("Token: Colon"));
        assert!(log.contains("Token: True"));
        assert!(log.contains("Token: RightBrace"));
        assert!(log.contains("Total tokens:"));
        assert!(log.contains("Statistics"));
    }

    #[test]
    fn test_debug_lexer_error_logging() {
        let config = LexerConfig {
            mode: LexerMode::Strict,
            collect_stats: true,
            ..Default::default()
        };

        let mut lexer = DebugLexer::new("{invalid}", config);

        // This should produce an error in strict mode (unquoted string)
        let _ = lexer.next_token(); // {
        let result = lexer.next_token(); // invalid - should error

        assert!(result.is_err());

        let log = lexer.format_log();
        assert!(log.contains("ERROR"));
        assert_eq!(lexer.stats.errors_count, 1);
    }

    #[test]
    fn test_debug_lexer_performance_stats() {
        let config = LexerConfig {
            mode: LexerMode::Standard,
            collect_stats: true,
            ..Default::default()
        };

        let input = r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#;
        let mut lexer = DebugLexer::new(input, config);

        while !lexer.is_eof() {
            let _ = lexer.next_token();
        }

        let stats = lexer.stats();
        assert!(stats.tokens_count > 0);
        assert!(stats.time_ns > 0);
        assert_eq!(stats.bytes_processed, input.len());

        // Print the log for manual inspection
        // lexer.print_log();
    }
}
