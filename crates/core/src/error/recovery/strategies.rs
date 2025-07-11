// this_file: crates/core/src/error/recovery/strategies.rs

use crate::error::Span;

/// Strategies for recovering from different types of JSON parsing errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Insert missing character(s) at the specified position
    InsertText {
        /// The position where text should be inserted.
        position: usize,
        /// The text to insert.
        text: String,
    },
    /// Remove character(s) at the specified span
    RemoveText {
        /// The span of text to remove.
        span: Span,
    },
    /// Replace text in the specified span with new text
    ReplaceText {
        /// The span of text to replace.
        span: Span,
        /// The replacement text.
        replacement: String,
    },
    /// Add missing closing delimiter
    AddClosing {
        /// The position where the closing delimiter should be added.
        position: usize,
        /// The closing delimiter character (e.g., '}', ']').
        delimiter: char,
    },
    /// Fix quoted string issues
    FixQuoting {
        /// The span of the string to fix.
        span: Span,
        /// The quote character to use.
        quote_char: char,
    },
    /// Repair number formatting
    RepairNumber {
        /// The span of the malformed number.
        span: Span,
        /// The corrected number value.
        corrected_value: String,
    },
    /// Add missing comma between elements
    AddComma {
        /// The position where the comma should be added.
        position: usize,
    },
    /// Remove trailing comma
    RemoveTrailingComma {
        /// The position of the trailing comma to remove.
        position: usize,
    },
    /// Convert unquoted key to quoted key
    QuoteKey {
        /// The span of the unquoted key.
        span: Span,
    },
    /// Fix comment syntax
    FixComment {
        /// The span of the malformed comment.
        span: Span,
    },
    /// No automatic recovery possible - manual intervention required
    ManualIntervention {
        /// Suggested manual interventions.
        suggestions: Vec<String>,
    },
}

impl RecoveryStrategy {
    /// Returns a human-readable description of the recovery strategy.
    pub fn description(&self) -> String {
        match self {
            RecoveryStrategy::InsertText { text, .. } => {
                format!("Insert \"{}\"", text)
            }
            RecoveryStrategy::RemoveText { span } => {
                format!("Remove {} characters", span.len())
            }
            RecoveryStrategy::ReplaceText { replacement, .. } => {
                format!("Replace with \"{}\"", replacement)
            }
            RecoveryStrategy::AddClosing { delimiter, .. } => {
                format!("Add closing '{}'", delimiter)
            }
            RecoveryStrategy::FixQuoting { quote_char, .. } => {
                format!("Fix quoting with '{}'", quote_char)
            }
            RecoveryStrategy::RepairNumber {
                corrected_value, ..
            } => {
                format!("Repair number to \"{}\"", corrected_value)
            }
            RecoveryStrategy::AddComma { .. } => "Add missing comma".to_string(),
            RecoveryStrategy::RemoveTrailingComma { .. } => "Remove trailing comma".to_string(),
            RecoveryStrategy::QuoteKey { .. } => "Add quotes around key".to_string(),
            RecoveryStrategy::FixComment { .. } => "Fix comment syntax".to_string(),
            RecoveryStrategy::ManualIntervention { suggestions } => {
                format!("Manual intervention required: {}", suggestions.join("; "))
            }
        }
    }

    /// Returns the confidence level for this recovery strategy (0.0 to 1.0).
    pub fn confidence(&self) -> f32 {
        match self {
            RecoveryStrategy::InsertText { .. } => 0.8,
            RecoveryStrategy::RemoveText { .. } => 0.7,
            RecoveryStrategy::ReplaceText { .. } => 0.8,
            RecoveryStrategy::AddClosing { .. } => 0.9,
            RecoveryStrategy::FixQuoting { .. } => 0.8,
            RecoveryStrategy::RepairNumber { .. } => 0.7,
            RecoveryStrategy::AddComma { .. } => 0.8,
            RecoveryStrategy::RemoveTrailingComma { .. } => 0.9,
            RecoveryStrategy::QuoteKey { .. } => 0.8,
            RecoveryStrategy::FixComment { .. } => 0.6,
            RecoveryStrategy::ManualIntervention { .. } => 0.3,
        }
    }
}
