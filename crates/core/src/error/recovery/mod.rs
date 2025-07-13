// this_file: crates/core/src/error/recovery/mod.rs

use crate::error::{Error, ErrorCode, Span};
use rustc_hash::FxHashMap;
/// Context-aware error recovery rules.
pub mod context;
/// Recovery strategy patterns for error correction.
pub mod strategies;

use context::ContextRule;
pub use strategies::RecoveryStrategy;

/// Analyzes parsing errors and suggests recovery strategies.
#[derive(Debug)]
pub struct ErrorRecoveryAnalyzer {
    /// Common patterns and their recovery strategies
    patterns: FxHashMap<String, Vec<RecoveryStrategy>>,
    /// Context-aware recovery rules
    context_rules: Vec<ContextRule>,
}

impl ErrorRecoveryAnalyzer {
    /// Creates a new error recovery analyzer with default rules.
    pub fn new() -> Self {
        let mut analyzer = ErrorRecoveryAnalyzer {
            patterns: FxHashMap::default(),
            context_rules: Vec::new(),
        };

        analyzer.init_default_patterns();
        analyzer.init_context_rules();
        analyzer
    }

    /// Analyzes an error and suggests recovery strategies.
    pub fn analyze_error(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        match error.code() {
            ErrorCode::UnexpectedCharacter => self.analyze_unexpected_character(error, input),
            ErrorCode::UnexpectedEndOfInput => self.analyze_unexpected_eof(error, input),
            ErrorCode::InvalidNumberFormat => self.analyze_invalid_number(error, input),
            ErrorCode::InvalidEscapeSequence => self.analyze_invalid_escape(error, input),
            ErrorCode::InvalidUnicodeEscape => self.analyze_invalid_unicode(error, input),
            ErrorCode::UnterminatedString => self.analyze_unterminated_string(error, input),
            ErrorCode::TrailingComma => self.analyze_trailing_comma(error, input),
            ErrorCode::ExpectedToken => self.analyze_expected_token(error, input),
            ErrorCode::DepthLimitExceeded => self.analyze_excessive_nesting(error, input),
            ErrorCode::BracketMismatch => self.analyze_mismatched_brackets(error, input),
            ErrorCode::UnbalancedBrackets => self.analyze_unbalanced_brackets(error, input),
            ErrorCode::RepairFailed => self.analyze_repair_failed(error, input),
            ErrorCode::MaxRepairsExceeded => self.analyze_max_repairs_exceeded(error, input),
            ErrorCode::InvalidChunk => self.analyze_invalid_chunk(error, input),
            _ => self.analyze_context_rules(error, input),
        }
    }

    /// Analyzes errors using context-aware rules.
    fn analyze_context_rules(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();
        if let Some(span) = error.span() {
            let context_str = self.get_context(input, &span, 50); // Get a larger context

            for rule in &self.context_rules {
                if self.matches_context_pattern(&context_str, &rule.pattern) {
                    // For now, just add the strategy. More sophisticated logic could
                    // involve confidence levels and combining strategies.
                    strategies.push(rule.strategy.clone());
                }
            }
        }
        strategies
    }

    /// Checks if the given context string matches the provided regex pattern.
    fn matches_context_pattern(&self, context: &str, pattern: &str) -> bool {
        // This is a placeholder. A real implementation would use a regex engine.
        // For now, a simple substring check or basic pattern matching.
        context.contains(pattern)
    }

    /// Initializes default error patterns and their recovery strategies.
    fn init_default_patterns(&mut self) {
        // Common syntax errors
        self.patterns.insert(
            "missing_quote".to_string(),
            vec![RecoveryStrategy::FixQuoting {
                span: Span::new(0, 0), // Will be filled by analysis
                quote_char: '"',
            }],
        );

        self.patterns.insert(
            "missing_comma".to_string(),
            vec![RecoveryStrategy::AddComma { position: 0 }],
        );

        self.patterns.insert(
            "trailing_comma".to_string(),
            vec![RecoveryStrategy::RemoveTrailingComma { position: 0 }],
        );

        self.patterns.insert(
            "unquoted_key".to_string(),
            vec![RecoveryStrategy::QuoteKey {
                span: Span::new(0, 0),
            }],
        );
    }

    /// Initializes context-aware recovery rules.
    fn init_context_rules(&mut self) {
        // Rule for detecting missing closing braces
        self.context_rules.push(ContextRule {
            pattern: r"^\s*$".to_string(), // End of input
            strategy: RecoveryStrategy::AddClosing {
                position: 0,
                delimiter: '}',
            },
            confidence: 0.8,
        });

        // Rule for detecting missing closing brackets
        self.context_rules.push(ContextRule {
            pattern: r"^\s*$".to_string(), // End of input
            strategy: RecoveryStrategy::AddClosing {
                position: 0,
                delimiter: ']',
            },
            confidence: 0.8,
        });
    }

    /// Analyzes unexpected character errors.
    fn analyze_unexpected_character(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let char_at_pos = input.chars().nth(span.start).unwrap_or('\0');
            let context = self.get_context(input, &span, 10);

            match char_at_pos {
                ',' => {
                    // Likely trailing comma
                    strategies.push(RecoveryStrategy::RemoveTrailingComma {
                        position: span.start,
                    });
                }
                '\'' => {
                    // Single quote instead of double quote
                    strategies.push(RecoveryStrategy::ReplaceText {
                        span,
                        replacement: "\"".to_string(),
                    });
                }
                _ => {
                    // Check if it's an unquoted key
                    if self.looks_like_unquoted_key(&context) {
                        strategies.push(RecoveryStrategy::QuoteKey { span });
                    } else {
                        // Generic character removal
                        strategies.push(RecoveryStrategy::RemoveText { span });
                    }
                }
            }
        }

        strategies
    }

    /// Analyzes unexpected EOF errors.
    fn analyze_unexpected_eof(&self, _error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();
        let input_len = input.len();

        // Count unclosed brackets and braces
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => bracket_count -= 1,
                _ => {}
            }
        }

        // Add missing closing delimiters
        for _ in 0..brace_count {
            strategies.push(RecoveryStrategy::AddClosing {
                position: input_len,
                delimiter: '}',
            });
        }

        for _ in 0..bracket_count {
            strategies.push(RecoveryStrategy::AddClosing {
                position: input_len,
                delimiter: ']',
            });
        }

        // If we're in a string, add closing quote
        if in_string {
            strategies.push(RecoveryStrategy::AddClosing {
                position: input_len,
                delimiter: '"',
            });
        }

        strategies
    }

    /// Analyzes invalid string errors.
    fn analyze_invalid_string(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let string_content = span.extract(input);

            // Check for unescaped quotes
            if string_content.contains('"') {
                strategies.push(RecoveryStrategy::ReplaceText {
                    span,
                    replacement: string_content.replace('"', "\\\""),
                });
            }

            // Check for unescaped newlines
            if string_content.contains('\n') {
                strategies.push(RecoveryStrategy::ReplaceText {
                    span,
                    replacement: string_content.replace('\n', "\\n"),
                });
            }

            // Check for missing quotes
            if !string_content.starts_with('"') || !string_content.ends_with('"') {
                strategies.push(RecoveryStrategy::FixQuoting {
                    span,
                    quote_char: '"',
                });
            }
        }

        strategies
    }

    /// Analyzes invalid number errors.
    fn analyze_invalid_number(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let number_text = span.extract(input);

            // Try to fix common number format issues
            if let Ok(corrected) = self.fix_number_format(number_text) {
                strategies.push(RecoveryStrategy::RepairNumber {
                    span,
                    corrected_value: corrected,
                });
            }

            // If it looks like a quoted number, suggest removing quotes
            if number_text.starts_with('"') && number_text.ends_with('"') {
                let unquoted = &number_text[1..number_text.len() - 1];
                if unquoted.parse::<f64>().is_ok() {
                    strategies.push(RecoveryStrategy::ReplaceText {
                        span,
                        replacement: unquoted.to_string(),
                    });
                }
            }
        }

        strategies
    }

    /// Analyzes invalid escape sequence errors.
    fn analyze_invalid_escape(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let escape_sequence = span.extract(input);

            // Common escape sequence fixes
            let fixed_escape = match escape_sequence {
                "\\'" => "\\'", // Single quote doesn't need escaping in JSON
                "\\`" => "`",   // Backtick doesn't need escaping
                _ => {
                    // Try to fix by doubling the backslash
                    &format!("\\{escape_sequence}")
                }
            };

            strategies.push(RecoveryStrategy::ReplaceText {
                span,
                replacement: fixed_escape.to_string(),
            });
        }

        strategies
    }

    /// Analyzes trailing comma errors.
    fn analyze_trailing_comma(&self, error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            strategies.push(RecoveryStrategy::RemoveTrailingComma {
                position: span.start,
            });
        }

        strategies
    }

    /// Analyzes missing comma errors.
    fn _analyze_missing_comma(&self, error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            strategies.push(RecoveryStrategy::AddComma {
                position: span.start,
            });
        }

        strategies
    }

    /// Analyzes unquoted key errors.
    fn _analyze_unquoted_key(&self, error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            strategies.push(RecoveryStrategy::QuoteKey { span });
        }

        strategies
    }

    /// Analyzes invalid comment errors.
    fn _analyze_invalid_comment(&self, error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            strategies.push(RecoveryStrategy::FixComment { span });
        }

        strategies
    }

    /// Analyzes excessive nesting errors.
    fn analyze_excessive_nesting(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "Consider restructuring the JSON to reduce nesting depth".to_string(),
                "Break down complex nested structures into separate objects".to_string(),
                "Use references or IDs instead of deeply nested objects".to_string(),
            ],
        }]
    }

    /// Analyzes duplicate key errors.
    fn _analyze_duplicate_key(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "Remove or rename one of the duplicate keys".to_string(),
                "Merge the values if appropriate".to_string(),
                "Use an array if multiple values are intended".to_string(),
            ],
        }]
    }

    /// Analyzes mismatched brackets errors.
    fn analyze_mismatched_brackets(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let char_at_pos = input.chars().nth(span.start).unwrap_or('\0');

            match char_at_pos {
                ']' => {
                    strategies.push(RecoveryStrategy::ReplaceText {
                        span,
                        replacement: "}".to_string(),
                    });
                }
                '}' => {
                    strategies.push(RecoveryStrategy::ReplaceText {
                        span,
                        replacement: "]".to_string(),
                    });
                }
                _ => {
                    strategies.push(RecoveryStrategy::ManualIntervention {
                        suggestions: vec![
                            "Check for missing opening bracket or brace".to_string(),
                            "Verify that all brackets and braces are properly matched".to_string(),
                        ],
                    });
                }
            }
        }

        strategies
    }

    /// Analyzes repair failed errors.
    fn analyze_repair_failed(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "The automatic repair system could not fix this JSON".to_string(),
                "Please check the JSON syntax manually".to_string(),
                "Consider using a JSON validator to identify specific issues".to_string(),
            ],
        }]
    }

    /// Analyzes parsing failed errors.
    fn _analyze_parsing_failed(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "The JSON structure is too malformed to parse".to_string(),
                "Check for fundamental syntax errors".to_string(),
                "Verify the JSON is complete and well-formed".to_string(),
            ],
        }]
    }

    /// Analyzes invalid unicode escape errors.
    fn analyze_invalid_unicode(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let unicode_escape = span.extract(input);

            // Try to fix common Unicode escape issues
            if let Some(hex_part) = unicode_escape.strip_prefix("\\u") {
                if hex_part.len() < 4 {
                    // Pad with zeros
                    let padded = format!("\\u{hex_part:0>4}");
                    strategies.push(RecoveryStrategy::ReplaceText {
                        span,
                        replacement: padded,
                    });
                } else if hex_part.len() > 4 {
                    // Truncate to 4 digits
                    let truncated = format!("\\u{}", &hex_part[..4]);
                    strategies.push(RecoveryStrategy::ReplaceText {
                        span,
                        replacement: truncated,
                    });
                }
            }
        }

        strategies
    }

    /// Analyzes unterminated string errors.
    fn analyze_unterminated_string(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            // Add closing quote at the end of the span
            strategies.push(RecoveryStrategy::AddClosing {
                position: span.end,
                delimiter: '"',
            });

            // Also try the general string analysis
            strategies.extend(self.analyze_invalid_string(error, input));
        }

        strategies
    }

    /// Analyzes expected token errors.
    fn analyze_expected_token(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();

        if let Some(span) = error.span() {
            let context = self.get_context(input, &span, 20);

            // Check if we're missing a comma
            if self.looks_like_missing_comma(&context) {
                strategies.push(RecoveryStrategy::AddComma {
                    position: span.start,
                });
            }

            // Check if we need a colon for object key-value pairs
            if context.contains('"') && !context.contains(':') {
                strategies.push(RecoveryStrategy::InsertText {
                    position: span.start,
                    text: ":".to_string(),
                });
            }

            // Generic suggestions
            strategies.push(RecoveryStrategy::ManualIntervention {
                suggestions: vec![
                    "Check for missing punctuation (comma, colon, quote)".to_string(),
                    "Verify proper JSON syntax around this position".to_string(),
                ],
            });
        }

        strategies
    }

    /// Analyzes unbalanced brackets errors.
    fn analyze_unbalanced_brackets(&self, error: &Error, input: &str) -> Vec<RecoveryStrategy> {
        // Similar to analyze_unexpected_eof but focused on bracket balancing
        self.analyze_unexpected_eof(error, input)
    }

    /// Analyzes max repairs exceeded errors.
    fn analyze_max_repairs_exceeded(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "Too many repair attempts were needed".to_string(),
                "The JSON may be too corrupted for automatic repair".to_string(),
                "Consider manually reviewing and fixing the JSON structure".to_string(),
                "Break down the JSON into smaller, more manageable pieces".to_string(),
            ],
        }]
    }

    /// Analyzes custom errors.
    #[allow(dead_code)]
    fn analyze_custom_error(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "A custom error occurred during parsing".to_string(),
                "Check the specific error message for more details".to_string(),
                "This may be a domain-specific validation error".to_string(),
            ],
        }]
    }

    /// Analyzes context errors.
    #[allow(dead_code)]
    fn analyze_context_error(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        // For context errors, try to analyze the underlying error
        // This is a wrapper error, so we need to extract the inner error if possible
        // For now, provide general suggestions
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "An error occurred in a specific parsing context".to_string(),
                "Check the surrounding JSON structure".to_string(),
                "Verify proper nesting and syntax".to_string(),
            ],
        }]
    }

    /// Checks if the context suggests a missing comma.
    fn looks_like_missing_comma(&self, context: &str) -> bool {
        // Look for patterns like: "value" "key" or } "key" or ] "key"
        let trimmed = context.trim();

        // Pattern: quoted value followed by quoted key (missing comma between array/object elements)
        if trimmed.contains('"') {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 4 {
                let between = parts[parts.len() - 3].trim();
                // If there's nothing but whitespace between two quoted strings, likely missing comma
                return between.is_empty() || between.chars().all(|c| c.is_whitespace());
            }
        }

        // Pattern: closing bracket/brace followed by opening quote
        if trimmed.contains('}') || trimmed.contains(']') {
            return trimmed.matches('"').count() >= 2;
        }

        false
    }

    /// Gets context around a span for analysis.
    fn get_context(&self, input: &str, span: &Span, context_size: usize) -> String {
        let start = span.start.saturating_sub(context_size);
        let end = (span.end + context_size).min(input.len());
        input[start..end].to_string()
    }

    /// Checks if the given context looks like an unquoted key.
    fn looks_like_unquoted_key(&self, context: &str) -> bool {
        // Simple heuristic: if it contains alphanumeric characters followed by a colon
        context.contains(':') && context.chars().any(|c| c.is_alphanumeric())
    }

    /// Attempts to fix common number format issues.
    fn fix_number_format(&self, number_text: &str) -> Result<String, ()> {
        // Remove leading/trailing whitespace
        let trimmed = number_text.trim();

        // Try to parse as-is first
        if trimmed.parse::<f64>().is_ok() {
            return Ok(trimmed.to_string());
        }

        // Try removing common invalid characters
        let cleaned = trimmed.replace(" ", "").replace("_", "").replace(",", "");

        if cleaned.parse::<f64>().is_ok() {
            return Ok(cleaned);
        }

        // Try fixing decimal separators
        if cleaned.replace(",", ".").parse::<f64>().is_ok() {
            return Ok(cleaned.replace(",", "."));
        }

        Err(())
    }

    /// Analyze invalid UTF-8 errors and suggest fixes
    #[allow(dead_code)]
    fn analyze_invalid_utf8(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "Invalid UTF-8 encoding detected".to_string(),
                "Ensure input is valid UTF-8 encoded text".to_string(),
                "Check file encoding and convert to UTF-8".to_string(),
                "Use a text editor that supports UTF-8".to_string(),
            ],
        }]
    }

    /// Analyze invalid chunk errors and suggest fixes
    fn analyze_invalid_chunk(&self, _error: &Error, _input: &str) -> Vec<RecoveryStrategy> {
        vec![RecoveryStrategy::ManualIntervention {
            suggestions: vec![
                "Invalid chunk during parallel processing".to_string(),
                "Try using smaller chunk sizes".to_string(),
                "Use sequential parsing instead of parallel".to_string(),
                "Check for corrupted input data".to_string(),
            ],
        }]
    }
}

impl Default for ErrorRecoveryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_analyzer_creation() {
        let analyzer = ErrorRecoveryAnalyzer::new();
        assert!(!analyzer.patterns.is_empty());
        assert!(!analyzer.context_rules.is_empty());
    }

    #[test]
    fn test_unexpected_eof_analysis() {
        let analyzer = ErrorRecoveryAnalyzer::new();
        let input = r#"{"key": "value""#; // Missing closing brace
        let error = Error::UnexpectedEof(input.len());

        let strategies = analyzer.analyze_error(&error, input);
        assert!(!strategies.is_empty());

        // Should suggest adding closing brace
        assert!(strategies
            .iter()
            .any(|s| matches!(s, RecoveryStrategy::AddClosing { delimiter: '}', .. })));
    }

    #[test]
    fn test_trailing_comma_analysis() {
        let analyzer = ErrorRecoveryAnalyzer::new();
        let input = r#"{"key": "value",}"#;
        let error = Error::TrailingComma(16);

        let strategies = analyzer.analyze_error(&error, input);
        assert!(!strategies.is_empty());

        // Should suggest removing trailing comma
        assert!(strategies
            .iter()
            .any(|s| matches!(s, RecoveryStrategy::RemoveTrailingComma { .. })));
    }

    #[test]
    fn test_recovery_strategy_description() {
        let strategy = RecoveryStrategy::AddComma { position: 10 };
        assert_eq!(strategy.description(), "Add missing comma");

        let strategy = RecoveryStrategy::FixQuoting {
            span: Span::new(0, 5),
            quote_char: '"',
        };
        assert_eq!(strategy.description(), "Fix quoting with '\"'");
    }

    #[test]
    fn test_recovery_strategy_confidence() {
        let strategy = RecoveryStrategy::AddClosing {
            position: 10,
            delimiter: '}',
        };
        assert_eq!(strategy.confidence(), 0.9);

        let strategy = RecoveryStrategy::ManualIntervention {
            suggestions: vec![],
        };
        assert_eq!(strategy.confidence(), 0.3);
    }

    #[test]
    fn test_number_format_fix() {
        let analyzer = ErrorRecoveryAnalyzer::new();

        // Test valid number
        assert_eq!(
            analyzer.fix_number_format("123.45"),
            Ok("123.45".to_string())
        );

        // Test number with spaces
        assert_eq!(
            analyzer.fix_number_format(" 123.45 "),
            Ok("123.45".to_string())
        );

        // Test number with commas
        assert_eq!(
            analyzer.fix_number_format("1,234.56"),
            Ok("1234.56".to_string())
        );

        // Test invalid number
        assert!(analyzer.fix_number_format("abc").is_err());
    }
}
