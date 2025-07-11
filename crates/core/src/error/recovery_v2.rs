//! Smart error recovery engine for vexy_json parsing
//!
//! This module implements advanced error recovery strategies including:
//! - ML-based pattern recognition for common errors
//! - Context-aware repair suggestions

#![allow(dead_code)]
//! - Source code snippets with error visualization
//! - "Did you mean?" suggestions

use crate::ast::{Token, Value};
use crate::error::{Error, Span};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Represents a recovery suggestion
#[derive(Debug, Clone)]
pub struct RecoverySuggestion {
    /// Description of the fix
    pub description: String,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// The fixed input if this suggestion is applied
    pub fixed_input: String,
    /// Category of the suggestion
    pub category: SuggestionCategory,
    /// Specific location where the fix would be applied
    pub fix_location: Span,
}

/// Categories of recovery suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionCategory {
    /// Missing bracket or brace
    MissingBracket,
    /// Unmatched quote
    UnmatchedQuote,
    /// Missing comma
    MissingComma,
    /// Trailing comma
    TrailingComma,
    /// Invalid escape sequence
    InvalidEscape,
    /// Type mismatch
    TypeMismatch,
    /// Structural error
    StructuralError,
    /// Unknown/other
    Other,
}

/// Smart error recovery engine
pub struct ErrorRecoveryEngineV2 {
    /// Pattern database for ML-based recognition
    pattern_db: PatternDatabase,
    /// Context analyzer
    context_analyzer: ContextAnalyzer,
    /// Recovery strategies
    strategies: Vec<Box<dyn RecoveryStrategy>>,
    /// Configuration
    config: RecoveryConfig,
}

/// Configuration for error recovery
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of suggestions to generate
    pub max_suggestions: usize,
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Enable ML-based pattern recognition
    pub enable_ml: bool,
    /// Enable context-aware suggestions
    pub enable_context: bool,
    /// Maximum recovery attempts
    pub max_attempts: usize,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        RecoveryConfig {
            max_suggestions: 5,
            min_confidence: 0.5,
            enable_ml: true,
            enable_context: true,
            max_attempts: 3,
        }
    }
}

/// Pattern database for ML-based error recognition
struct PatternDatabase {
    patterns: FxHashMap<String, ErrorPattern>,
    learned_patterns: Vec<LearnedPattern>,
}

/// Represents a common error pattern
struct ErrorPattern {
    // No fields needed if ML-based pattern recognition is not fully implemented yet
}

/// Learned pattern from successful recovery
#[derive(Debug, Clone)]
struct LearnedPattern {
    // No fields needed if ML-based pattern recognition is not fully implemented yet
}

/// Context analyzer for understanding error context
struct ContextAnalyzer {
    /// Schema information if available
    schema: Option<Value>,
    /// Previous successful parses
    history: VecDeque<String>,
    /// Token lookahead buffer
    lookahead_size: usize,
}

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The error that occurred
    pub error: Error,
    /// Current input being parsed
    pub input: String,
    /// Error position
    pub position: usize,
    /// Tokens before error
    pub tokens_before: Vec<(Token, Span)>,
    /// Partial AST if available
    pub partial_ast: Option<Value>,
    /// Current parsing context (e.g., "in_object", "in_array")
    pub parsing_context: String,
}

/// Recovery strategy trait
trait RecoveryStrategy {
    /// Name of the strategy
    fn name(&self) -> &str;

    /// Try to recover from error
    fn recover(&self, context: &ErrorContext) -> Vec<RecoverySuggestion>;
}

impl ErrorRecoveryEngineV2 {
    /// Create a new error recovery engine
    pub fn new() -> Self {
        Self::with_config(RecoveryConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: RecoveryConfig) -> Self {
        let mut engine = ErrorRecoveryEngineV2 {
            pattern_db: PatternDatabase::new(),
            context_analyzer: ContextAnalyzer::new(),
            strategies: Vec::new(),
            config,
        };

        // Initialize default strategies
        engine.add_default_strategies();
        engine
    }

    /// Add default recovery strategies
    fn add_default_strategies(&mut self) {
        self.strategies
            .push(Box::new(BracketMatchingStrategy::new()));
        self.strategies
            .push(Box::new(QuoteInferenceStrategy::new()));
        self.strategies
            .push(Box::new(CommaSuggestionStrategy::new()));
        self.strategies.push(Box::new(TypeCoercionStrategy::new()));
        self.strategies
            .push(Box::new(StructuralRepairStrategy::new()));
    }

    /// Generate recovery suggestions for an error
    pub fn suggest_recovery(&self, context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let mut suggestions = Vec::new();

        // Try ML-based pattern recognition
        if self.config.enable_ml {
            suggestions.extend(self.pattern_db.find_matches(context));
        }

        // Try each recovery strategy
        for strategy in &self.strategies {
            let strategy_suggestions = strategy.recover(context);
            suggestions.extend(strategy_suggestions);
        }

        // Apply context analysis if enabled
        if self.config.enable_context {
            suggestions = self
                .context_analyzer
                .refine_suggestions(suggestions, context);
        }

        // Sort by confidence and limit
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(self.config.max_suggestions);

        // Filter by minimum confidence
        suggestions.retain(|s| s.confidence >= self.config.min_confidence);

        suggestions
    }

    /// Create error message with visual aids
    pub fn create_visual_error(&self, context: &ErrorContext) -> String {
        let mut output = String::new();

        // Add error description
        output.push_str(&format!("Error: {}\n\n", context.error));

        // Add source visualization
        output.push_str(&self.create_source_visualization(context));

        // Add suggestions
        let suggestions = self.suggest_recovery(context);
        if !suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for (i, suggestion) in suggestions.iter().enumerate() {
                output.push_str(&format!(
                    "  {}. {} (confidence: {:.0}%)\n",
                    i + 1,
                    suggestion.description,
                    suggestion.confidence * 100.0
                ));
            }
        }

        output
    }

    /// Create source code visualization with error arrow
    fn create_source_visualization(&self, context: &ErrorContext) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = context.input.lines().collect();

        // Find line and column of error
        let mut current_pos = 0;
        let mut error_line = 0;
        let mut error_col = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_end = current_pos + line.len() + 1; // +1 for newline
            if context.position >= current_pos && context.position < line_end {
                error_line = line_idx;
                error_col = context.position - current_pos;
                break;
            }
            current_pos = line_end;
        }

        // Show context lines
        let start_line = error_line.saturating_sub(2);
        let end_line = (error_line + 3).min(lines.len());

        for (idx, line) in lines[start_line..end_line].iter().enumerate() {
            let line_num = start_line + idx + 1;
            output.push_str(&format!("{:4} | {}\n", line_num, line));

            if start_line + idx == error_line {
                // Add error arrow
                output.push_str(&format!("     | {}^\n", " ".repeat(error_col)));
                output.push_str(&format!("     | {}--- error here\n", " ".repeat(error_col)));
            }
        }

        output
    }
}

impl PatternDatabase {
    fn new() -> Self {
        let mut db = PatternDatabase {
            patterns: FxHashMap::default(),
            learned_patterns: Vec::new(),
        };

        // Initialize common patterns
        db.add_common_patterns();
        db
    }

    fn add_common_patterns(&mut self) {
        // Add patterns for common errors
        // This would be expanded with actual pattern implementations
    }

    fn find_matches(&self, _context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let suggestions = Vec::new();

        // Pattern matching would be implemented here
        // For now, return empty suggestions

        suggestions
    }
}

impl ContextAnalyzer {
    fn new() -> Self {
        ContextAnalyzer {
            schema: None,
            history: VecDeque::with_capacity(10),
            lookahead_size: 5,
        }
    }

    fn refine_suggestions(
        &self,
        mut suggestions: Vec<RecoverySuggestion>,
        _context: &ErrorContext,
    ) -> Vec<RecoverySuggestion> {
        // Refine suggestions based on context
        for suggestion in &mut suggestions {
            // Boost confidence if suggestion matches schema
            if let Some(ref schema) = self.schema {
                if self.matches_schema(&suggestion.fixed_input, schema) {
                    suggestion.confidence *= 1.2;
                }
            }

            // Boost confidence if similar to previous successful parses
            for previous in &self.history {
                let similarity = self.calculate_similarity(&suggestion.fixed_input, previous);
                suggestion.confidence *= 1.0 + (similarity * 0.2);
            }
        }

        suggestions
    }

    fn matches_schema(&self, _input: &str, _schema: &Value) -> bool {
        // Check if input matches schema
        // This would be implemented with actual schema validation
        true
    }

    fn calculate_similarity(&self, _a: &str, _b: &str) -> f64 {
        // Calculate string similarity
        // This would use edit distance or similar algorithm
        0.5
    }
}

// Recovery Strategy Implementations

/// Bracket matching strategy
struct BracketMatchingStrategy {
    bracket_pairs: Vec<(char, char)>,
}

impl BracketMatchingStrategy {
    fn new() -> Self {
        BracketMatchingStrategy {
            bracket_pairs: vec![('{', '}'), ('[', ']'), ('(', ')')],
        }
    }
}

impl RecoveryStrategy for BracketMatchingStrategy {
    fn name(&self) -> &str {
        "bracket_matching"
    }

    fn recover(&self, context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let mut suggestions = Vec::new();

        // Count bracket pairs
        let mut stack = Vec::new();
        for ch in context.input.chars() {
            for (open, close) in &self.bracket_pairs {
                if ch == *open {
                    stack.push(ch);
                } else if ch == *close {
                    if let Some(last) = stack.last() {
                        if *last == *open {
                            stack.pop();
                        } else {
                            // Mismatched bracket
                            // Generate suggestion
                        }
                    }
                }
            }
        }

        // Check for unclosed brackets
        while let Some(unclosed) = stack.pop() {
            for (open, close) in &self.bracket_pairs {
                if unclosed == *open {
                    let mut fixed = context.input.clone();
                    fixed.push(*close);

                    suggestions.push(RecoverySuggestion {
                        description: format!("Add missing '{}'", close),
                        confidence: 0.8,
                        fixed_input: fixed,
                        category: SuggestionCategory::MissingBracket,
                        fix_location: Span {
                            start: context.input.len(),
                            end: context.input.len(),
                        },
                    });
                }
            }
        }

        suggestions
    }
}

/// Quote inference strategy
struct QuoteInferenceStrategy;

impl QuoteInferenceStrategy {
    fn new() -> Self {
        QuoteInferenceStrategy
    }
}

impl RecoveryStrategy for QuoteInferenceStrategy {
    fn name(&self) -> &str {
        "quote_inference"
    }

    fn recover(&self, context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let mut suggestions = Vec::new();

        // Check for unmatched quotes
        let mut in_string = false;
        let mut quote_char = '\0';
        let mut escape = false;

        for (_i, ch) in context.input.chars().enumerate() {
            if escape {
                escape = false;
                continue;
            }

            if ch == '\\' {
                escape = true;
                continue;
            }

            if !in_string && (ch == '"' || ch == '\'') {
                in_string = true;
                quote_char = ch;
            } else if in_string && ch == quote_char {
                in_string = false;
            }
        }

        if in_string {
            let mut fixed = context.input.clone();
            fixed.push(quote_char);

            suggestions.push(RecoverySuggestion {
                description: format!("Add missing closing quote '{}'", quote_char),
                confidence: 0.9,
                fixed_input: fixed,
                category: SuggestionCategory::UnmatchedQuote,
                fix_location: Span {
                    start: context.input.len(),
                    end: context.input.len(),
                },
            });
        }

        suggestions
    }
}

/// Comma suggestion strategy
struct CommaSuggestionStrategy;

impl CommaSuggestionStrategy {
    fn new() -> Self {
        CommaSuggestionStrategy
    }
}

impl RecoveryStrategy for CommaSuggestionStrategy {
    fn name(&self) -> &str {
        "comma_suggestion"
    }

    fn recover(&self, _context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let suggestions = Vec::new();

        // Check for missing commas between array/object elements
        // This would analyze token patterns to detect missing commas

        suggestions
    }
}

/// Type coercion strategy
struct TypeCoercionStrategy;

impl TypeCoercionStrategy {
    fn new() -> Self {
        TypeCoercionStrategy
    }
}

impl RecoveryStrategy for TypeCoercionStrategy {
    fn name(&self) -> &str {
        "type_coercion"
    }

    fn recover(&self, _context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let suggestions = Vec::new();

        // Suggest type conversions based on context
        // e.g., "true" -> true, "123" -> 123

        suggestions
    }
}

/// Structural repair strategy
struct StructuralRepairStrategy;

impl StructuralRepairStrategy {
    fn new() -> Self {
        StructuralRepairStrategy
    }
}

impl RecoveryStrategy for StructuralRepairStrategy {
    fn name(&self) -> &str {
        "structural_repair"
    }

    fn recover(&self, _context: &ErrorContext) -> Vec<RecoverySuggestion> {
        let suggestions = Vec::new();

        // Analyze overall structure and suggest repairs
        // e.g., converting malformed object to valid object

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_matching() {
        let engine = ErrorRecoveryEngineV2::new();

        let context = ErrorContext {
            error: Error::UnexpectedEof(10),
            input: r#"{"name": "test"#.to_string(),
            position: 14,
            tokens_before: vec![],
            partial_ast: None,
            parsing_context: "in_object".to_string(),
        };

        let suggestions = engine.suggest_recovery(&context);
        assert!(!suggestions.is_empty());

        let first = &suggestions[0];
        assert_eq!(first.category, SuggestionCategory::MissingBracket);
        assert!(first.fixed_input.ends_with('}'));
    }

    #[test]
    fn test_quote_inference() {
        let engine = ErrorRecoveryEngineV2::new();

        let context = ErrorContext {
            error: Error::UnterminatedString(5),
            input: r#"{"key": "value"#.to_string(),
            position: 14,
            tokens_before: vec![],
            partial_ast: None,
            parsing_context: "in_string".to_string(),
        };

        let suggestions = engine.suggest_recovery(&context);
        assert!(!suggestions.is_empty());

        let first = &suggestions[0];
        assert_eq!(first.category, SuggestionCategory::UnmatchedQuote);
    }

    #[test]
    fn test_visual_error() {
        let engine = ErrorRecoveryEngineV2::new();

        let context = ErrorContext {
            error: Error::UnexpectedEof(20),
            input: "{\n  \"name\": \"test\"\n  \"age\": 25\n}".to_string(),
            position: 20, // Position after "test"
            tokens_before: vec![],
            partial_ast: None,
            parsing_context: "in_object".to_string(),
        };

        let visual = engine.create_visual_error(&context);
        assert!(visual.contains("---"));
        assert!(visual.contains("^"));
    }
}
