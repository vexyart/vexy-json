//! ML-based pattern recognition for error recovery
//!
//! This module implements machine learning-inspired pattern recognition
//! for common JSON parsing errors.

#![allow(dead_code)]

use crate::error::recovery_v2::{ErrorContext, RecoverySuggestion, SuggestionCategory};
use crate::error::{Error, Span};
use rustc_hash::FxHashMap;

/// ML-based pattern recognizer
pub struct MLPatternRecognizer {
    /// Feature extractors
    feature_extractors: Vec<Box<dyn FeatureExtractor>>,
    /// Trained patterns with their weights
    patterns: FxHashMap<String, TrainedPattern>,
}

/// Feature extractor trait
trait FeatureExtractor {
    /// Extract features from error context
    fn extract(&self, context: &ErrorContext) -> Vec<Feature>;
}

/// Represents a feature extracted from error context
#[derive(Debug, Clone)]
struct Feature {
    /// Feature name
    name: String,
    /// Feature value (normalized 0.0-1.0)
    value: f64,
    /// Feature weight (importance)
    weight: f64,
}

/// Trained pattern for error recognition
#[derive(Debug, Clone)]
struct TrainedPattern {
    /// Pattern ID
    id: String,
    /// Feature weights
    weights: FxHashMap<String, f64>,
    /// Bias term
    bias: f64,
    /// Success rate
    success_rate: f64,
    /// Number of times used
    usage_count: usize,
    /// Associated fix template
    fix_template: FixTemplate,
}

/// Template for generating fixes
#[derive(Debug, Clone)]
enum FixTemplate {
    /// Insert character at position
    InsertChar { char: char, offset: i32 },
    /// Insert string at position
    InsertString { string: String, offset: i32 },
    /// Replace a range with new text
    ReplaceRange {
        start: i32,
        end: i32,
        replacement: String,
    },
    /// Remove a range
    RemoveRange { start: i32, end: i32 },
    /// Complex operation with multiple sub-operations
    Complex(Vec<FixOperation>),
}

/// Single fix operation
#[derive(Debug, Clone)]
enum FixOperation {
    Insert {
        position: usize,
        text: String,
    },
    Delete {
        start: usize,
        end: usize,
    },
    Replace {
        start: usize,
        end: usize,
        text: String,
    },
}

/// Record of a successful fix
#[derive(Debug, Clone)]
struct SuccessfulFix {
    /// Error signature
    error_signature: String,
}

impl MLPatternRecognizer {
    /// Create a new ML pattern recognizer
    pub fn new() -> Self {
        let mut recognizer = MLPatternRecognizer {
            feature_extractors: Vec::new(),
            patterns: FxHashMap::default(),
        };

        // Initialize feature extractors
        recognizer.add_default_extractors();

        // Load pre-trained patterns
        recognizer.load_pretrained_patterns();

        recognizer
    }

    /// Add default feature extractors
    fn add_default_extractors(&mut self) {
        self.feature_extractors
            .push(Box::new(TokenPatternExtractor));
        self.feature_extractors
            .push(Box::new(CharacterDistributionExtractor));
        self.feature_extractors
            .push(Box::new(StructuralBalanceExtractor));
        self.feature_extractors.push(Box::new(ContextualExtractor));
        self.feature_extractors.push(Box::new(ErrorTypeExtractor));
    }

    /// Load pre-trained patterns
    fn load_pretrained_patterns(&mut self) {
        // Missing closing brace pattern
        self.patterns.insert(
            "missing_closing_brace".to_string(),
            TrainedPattern {
                id: "missing_closing_brace".to_string(),
                weights: [
                    ("unmatched_open_brace".to_string(), 0.9),
                    ("at_end_of_input".to_string(), 0.8),
                    ("in_object_context".to_string(), 0.7),
                ]
                .iter()
                .cloned()
                .collect(),
                bias: -0.1,
                success_rate: 0.95,
                usage_count: 0,
                fix_template: FixTemplate::InsertChar {
                    char: '}',
                    offset: 0,
                },
            },
        );

        // Missing closing bracket pattern
        self.patterns.insert(
            "missing_closing_bracket".to_string(),
            TrainedPattern {
                id: "missing_closing_bracket".to_string(),
                weights: [
                    ("unmatched_open_bracket".to_string(), 0.9),
                    ("at_end_of_input".to_string(), 0.8),
                    ("in_array_context".to_string(), 0.7),
                ]
                .iter()
                .cloned()
                .collect(),
                bias: -0.1,
                success_rate: 0.95,
                usage_count: 0,
                fix_template: FixTemplate::InsertChar {
                    char: ']',
                    offset: 0,
                },
            },
        );

        // Missing comma pattern
        self.patterns.insert(
            "missing_comma".to_string(),
            TrainedPattern {
                id: "missing_comma".to_string(),
                weights: [
                    ("consecutive_values".to_string(), 0.9),
                    ("after_string_or_number".to_string(), 0.8),
                    ("before_string_or_brace".to_string(), 0.7),
                ]
                .iter()
                .cloned()
                .collect(),
                bias: -0.2,
                success_rate: 0.85,
                usage_count: 0,
                fix_template: FixTemplate::InsertChar {
                    char: ',',
                    offset: 0,
                },
            },
        );

        // Unmatched quote pattern
        self.patterns.insert(
            "unmatched_quote".to_string(),
            TrainedPattern {
                id: "unmatched_quote".to_string(),
                weights: [
                    ("odd_quote_count".to_string(), 0.95),
                    ("unterminated_string_error".to_string(), 0.9),
                    ("at_end_of_line".to_string(), 0.6),
                ]
                .iter()
                .cloned()
                .collect(),
                bias: -0.1,
                success_rate: 0.9,
                usage_count: 0,
                fix_template: FixTemplate::InsertChar {
                    char: '"',
                    offset: 0,
                },
            },
        );
    }

    /// Recognize patterns and suggest fixes
    pub fn recognize_and_suggest(&mut self, context: &ErrorContext) -> Vec<RecoverySuggestion> {
        // Extract features
        let features = self.extract_features(context);

        // Score each pattern
        let mut pattern_scores: Vec<(String, f64)> = self
            .patterns
            .iter()
            .map(|(id, pattern)| {
                let score = self.calculate_pattern_score(pattern, &features);
                (id.clone(), score)
            })
            .collect();

        // Sort by score
        pattern_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Generate suggestions for high-scoring patterns
        let mut suggestions = Vec::new();
        for (pattern_id, score) in pattern_scores {
            if score > 0.5 {
                if let Some(pattern) = self.patterns.get(&pattern_id) {
                    if let Some(suggestion) = self.generate_suggestion(pattern, context, score) {
                        suggestions.push(suggestion);
                    }
                }
            }
        }

        suggestions
    }

    /// Extract features from context
    fn extract_features(&self, context: &ErrorContext) -> Vec<Feature> {
        let mut features = Vec::new();

        for extractor in &self.feature_extractors {
            features.extend(extractor.extract(context));
        }

        features
    }

    /// Calculate pattern score using logistic regression
    fn calculate_pattern_score(&self, pattern: &TrainedPattern, features: &[Feature]) -> f64 {
        let mut score = pattern.bias;

        for feature in features {
            if let Some(weight) = pattern.weights.get(&feature.name) {
                score += weight * feature.value;
            }
        }

        // Apply sigmoid function
        1.0 / (1.0 + (-score).exp())
    }

    /// Generate suggestion from pattern
    fn generate_suggestion(
        &self,
        pattern: &TrainedPattern,
        context: &ErrorContext,
        confidence: f64,
    ) -> Option<RecoverySuggestion> {
        let fixed_input = self.apply_fix_template(&pattern.fix_template, context)?;

        Some(RecoverySuggestion {
            description: format!("ML: {}", pattern.id.replace('_', " ")),
            confidence: confidence * pattern.success_rate,
            fixed_input,
            category: SuggestionCategory::Other,
            fix_location: Span {
                start: context.position,
                end: context.position,
            },
        })
    }

    /// Apply fix template to generate fixed input
    fn apply_fix_template(&self, template: &FixTemplate, context: &ErrorContext) -> Option<String> {
        match template {
            FixTemplate::InsertChar { char, offset } => {
                let position = (context.position as i32 + offset).max(0) as usize;
                let mut fixed = context.input.clone();
                if position <= fixed.len() {
                    fixed.insert(position, *char);
                    Some(fixed)
                } else {
                    fixed.push(*char);
                    Some(fixed)
                }
            }
            FixTemplate::InsertString { string, offset } => {
                let position = (context.position as i32 + offset).max(0) as usize;
                let mut fixed = context.input.clone();
                if position <= fixed.len() {
                    fixed.insert_str(position, string);
                    Some(fixed)
                } else {
                    fixed.push_str(string);
                    Some(fixed)
                }
            }
            FixTemplate::ReplaceRange {
                start,
                end,
                replacement,
            } => {
                let start_pos = (context.position as i32 + start).max(0) as usize;
                let end_pos = (context.position as i32 + end).max(0) as usize;
                let mut fixed = context.input.clone();
                fixed.replace_range(start_pos..end_pos, replacement);
                Some(fixed)
            }
            FixTemplate::RemoveRange { start, end } => {
                let start_pos = (context.position as i32 + start).max(0) as usize;
                let end_pos = (context.position as i32 + end).max(0) as usize;
                let mut fixed = context.input.clone();
                fixed.replace_range(start_pos..end_pos, "");
                Some(fixed)
            }
            FixTemplate::Complex(operations) => {
                let mut fixed = context.input.clone();
                for op in operations {
                    match op {
                        FixOperation::Insert { position, text } => {
                            if *position <= fixed.len() {
                                fixed.insert_str(*position, text);
                            }
                        }
                        FixOperation::Delete { start, end } => {
                            if *start <= fixed.len() && *end <= fixed.len() && start <= end {
                                fixed.replace_range(*start..*end, "");
                            }
                        }
                        FixOperation::Replace { start, end, text } => {
                            if *start <= fixed.len() && *end <= fixed.len() && start <= end {
                                fixed.replace_range(*start..*end, text);
                            }
                        }
                    }
                }
                Some(fixed)
            }
        }
    }

    /// Update patterns based on user feedback
    pub fn update_from_feedback(&mut self, pattern_id: &str, success: bool) {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.usage_count += 1;

            // Update success rate with exponential moving average
            let alpha = 0.1; // Learning rate
            pattern.success_rate =
                (1.0 - alpha) * pattern.success_rate + alpha * (if success { 1.0 } else { 0.0 });

            // Adjust weights if needed
            if !success && pattern.success_rate < 0.5 {
                // Reduce all weights slightly
                for weight in pattern.weights.values_mut() {
                    *weight *= 0.95;
                }
            }
        }
    }
}

// Feature Extractor Implementations

/// Extract token-based patterns
struct TokenPatternExtractor;

impl FeatureExtractor for TokenPatternExtractor {
    fn extract(&self, context: &ErrorContext) -> Vec<Feature> {
        let mut features = Vec::new();

        // Check for consecutive values without comma
        if context.tokens_before.len() >= 2 {
            let _last_two: Vec<_> = context.tokens_before.iter().rev().take(2).collect();
            // Simple heuristic for consecutive values
            features.push(Feature {
                name: "consecutive_values".to_string(),
                value: 0.5, // Would need actual token analysis
                weight: 1.0,
            });
        }

        features
    }
}

/// Extract character distribution features
struct CharacterDistributionExtractor;

impl FeatureExtractor for CharacterDistributionExtractor {
    fn extract(&self, context: &ErrorContext) -> Vec<Feature> {
        let mut features = Vec::new();

        // Count brackets and quotes
        let mut open_braces = 0;
        let mut close_braces = 0;
        let mut open_brackets = 0;
        let mut close_brackets = 0;
        let mut quotes = 0;

        for ch in context.input.chars() {
            match ch {
                '{' => open_braces += 1,
                '}' => close_braces += 1,
                '[' => open_brackets += 1,
                ']' => close_brackets += 1,
                '"' => quotes += 1,
                _ => {}
            }
        }

        features.push(Feature {
            name: "unmatched_open_brace".to_string(),
            value: if open_braces > close_braces { 1.0 } else { 0.0 },
            weight: 1.0,
        });

        features.push(Feature {
            name: "unmatched_open_bracket".to_string(),
            value: if open_brackets > close_brackets {
                1.0
            } else {
                0.0
            },
            weight: 1.0,
        });

        features.push(Feature {
            name: "odd_quote_count".to_string(),
            value: if quotes % 2 == 1 { 1.0 } else { 0.0 },
            weight: 1.0,
        });

        features
    }
}

/// Extract structural balance features
struct StructuralBalanceExtractor;

impl FeatureExtractor for StructuralBalanceExtractor {
    fn extract(&self, context: &ErrorContext) -> Vec<Feature> {
        let mut features = Vec::new();

        // Check if at end of input
        features.push(Feature {
            name: "at_end_of_input".to_string(),
            value: if context.position >= context.input.len() - 1 {
                1.0
            } else {
                0.0
            },
            weight: 1.0,
        });

        features
    }
}

/// Extract contextual features
struct ContextualExtractor;

impl FeatureExtractor for ContextualExtractor {
    fn extract(&self, context: &ErrorContext) -> Vec<Feature> {
        let mut features = Vec::new();

        // Check parsing context
        features.push(Feature {
            name: "in_object_context".to_string(),
            value: if context.parsing_context.contains("object") {
                1.0
            } else {
                0.0
            },
            weight: 1.0,
        });

        features.push(Feature {
            name: "in_array_context".to_string(),
            value: if context.parsing_context.contains("array") {
                1.0
            } else {
                0.0
            },
            weight: 1.0,
        });

        features
    }
}

/// Extract error type features
struct ErrorTypeExtractor;

impl FeatureExtractor for ErrorTypeExtractor {
    fn extract(&self, context: &ErrorContext) -> Vec<Feature> {
        let mut features = Vec::new();

        // Check error type
        match &context.error {
            Error::UnterminatedString(_) => {
                features.push(Feature {
                    name: "unterminated_string_error".to_string(),
                    value: 1.0,
                    weight: 1.0,
                });
            }
            Error::UnexpectedEof(_) | Error::UnexpectedChar(_, _) => {
                features.push(Feature {
                    name: "unexpected_token_error".to_string(),
                    value: 1.0,
                    weight: 1.0,
                });
            }
            _ => {}
        }

        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_pattern_recognition() {
        let mut recognizer = MLPatternRecognizer::new();

        // Test missing closing brace
        let context = ErrorContext {
            error: Error::UnexpectedEof(10),
            input: r#"{"name": "test""#.to_string(),
            position: 15,
            tokens_before: vec![],
            partial_ast: None,
            parsing_context: "in_object".to_string(),
        };

        let suggestions = recognizer.recognize_and_suggest(&context);
        assert!(!suggestions.is_empty());

        // Should suggest adding closing brace
        let first = &suggestions[0];
        assert!(first.fixed_input.ends_with('}'));
    }

    #[test]
    fn test_feature_extraction() {
        let recognizer = MLPatternRecognizer::new();

        let context = ErrorContext {
            error: Error::UnterminatedString(5),
            input: r#"{"key": "value"#.to_string(),
            position: 14,
            tokens_before: vec![],
            partial_ast: None,
            parsing_context: "in_string".to_string(),
        };

        let features = recognizer.extract_features(&context);

        // Should extract odd quote count feature
        let quote_feature = features
            .iter()
            .find(|f| f.name == "odd_quote_count")
            .expect("Should have odd quote count feature");

        assert_eq!(quote_feature.value, 1.0);
    }
}
