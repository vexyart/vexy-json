// this_file: src/error/utils.rs

use super::repair::{RepairAction, RepairType};
use super::types::Error;
use rustc_hash::FxHashMap;

/// Severity levels for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - warnings or minor issues
    Low,
    /// Medium severity - issues that may cause problems
    Medium,
    /// High severity - critical issues that prevent parsing
    High,
}

impl ErrorSeverity {
    /// Returns the string representation of the severity level.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "low",
            ErrorSeverity::Medium => "medium",
            ErrorSeverity::High => "high",
        }
    }
}

/// Utility helper for enhanced error diagnostics and reporting.
pub struct ErrorHelper;

impl ErrorHelper {
    /// Determines the severity level of an error based on its type.
    pub fn severity(error: &Error) -> ErrorSeverity {
        match error {
            Error::TrailingComma(_) => ErrorSeverity::Low,
            Error::Custom(_) => ErrorSeverity::Low,
            Error::WithContext { .. } => ErrorSeverity::Medium,
            Error::InvalidEscape(_) | Error::InvalidUnicode(_) => ErrorSeverity::Medium,
            Error::UnexpectedChar(_, _)
            | Error::UnexpectedEof(_)
            | Error::InvalidNumber(_)
            | Error::UnterminatedString(_)
            | Error::Expected { .. }
            | Error::DepthLimitExceeded(_)
            | Error::RepairFailed(_)
            | Error::BracketMismatch(_, _, _)
            | Error::UnbalancedBrackets(_, _)
            | Error::MaxRepairsExceeded(_)
            | Error::InvalidUtf8(_)
            | Error::InvalidChunk(_) => ErrorSeverity::High,
        }
    }

    /// Creates a comprehensive error report with code, severity, and suggestions.
    pub fn create_report(error: &Error) -> String {
        let code = error.code();
        let severity = Self::severity(error);
        let suggestions = error.suggestions();

        let mut report = format!(
            "[{}] {} ({})\n{}",
            code.as_str(),
            severity.as_str().to_uppercase(),
            code.description(),
            error
        );

        if !suggestions.is_empty() {
            report.push_str("\n\nSuggestions:");
            for (i, suggestion) in suggestions.iter().enumerate() {
                report.push_str(&format!("\n  {}. {}", i + 1, suggestion));
            }
        }

        report
    }

    /// Checks if an error is recoverable through automatic repair.
    #[inline(always)]
    pub fn is_recoverable(error: &Error) -> bool {
        match error {
            Error::TrailingComma(_) => true,
            Error::BracketMismatch(_, _, _) => true,
            Error::UnbalancedBrackets(_, _) => true,
            Error::UnexpectedEof(_) => true,
            Error::Expected { .. } => true,
            Error::RepairFailed(_) => false,
            Error::MaxRepairsExceeded(_) => false,
            Error::Custom(_) => false,
            Error::DepthLimitExceeded(_) => false,
            Error::InvalidUtf8(_) => false,
            Error::InvalidChunk(_) => false,
            _ => true,
        }
    }

    /// Categorizes an error into a general category for reporting.
    pub fn categorize(error: &Error) -> &'static str {
        match error {
            Error::UnexpectedChar(_, _) | Error::UnexpectedEof(_) => "syntax",
            Error::InvalidNumber(_) => "number",
            Error::InvalidEscape(_) | Error::InvalidUnicode(_) | Error::UnterminatedString(_) => {
                "string"
            }
            Error::TrailingComma(_) => "formatting",
            Error::Expected { .. } => "structure",
            Error::DepthLimitExceeded(_) => "limits",
            Error::Custom(_) => "custom",
            Error::WithContext { .. } => "context",
            Error::RepairFailed(_) => "repair",
            Error::BracketMismatch(_, _, _) | Error::UnbalancedBrackets(_, _) => "brackets",
            Error::MaxRepairsExceeded(_) => "repair",
            Error::InvalidUtf8(_) => "encoding",
            Error::InvalidChunk(_) => "parallel",
        }
    }
}

/// Utility trait for error analysis and manipulation.
///
/// This trait provides methods for extracting useful information
/// from errors, particularly position data for debugging.
pub trait ErrorUtils {
    /// Returns the position in the input where the error occurred, if available.
    ///
    /// Most parsing errors have associated position information to help
    /// users locate the problematic input. Custom errors may not have
    /// position information and will return None.
    fn position(&self) -> Option<usize>;

    /// Checks if this error is related to string parsing.
    ///
    /// Useful for error categorization and specialized error handling.
    fn is_string_error(&self) -> bool;

    /// Checks if this error is related to number parsing.
    ///
    /// Useful for error categorization and specialized error handling.
    fn is_number_error(&self) -> bool;

    /// Checks if this error is a structural parsing error.
    ///
    /// Structural errors include bracket mismatches, unexpected tokens, etc.
    fn is_structural_error(&self) -> bool;
}

impl ErrorUtils for Error {
    fn position(&self) -> Option<usize> {
        match self {
            Error::UnexpectedChar(_, pos)
            | Error::UnexpectedEof(pos)
            | Error::InvalidNumber(pos)
            | Error::InvalidEscape(pos)
            | Error::InvalidUnicode(pos)
            | Error::UnterminatedString(pos)
            | Error::TrailingComma(pos)
            | Error::Expected { position: pos, .. }
            | Error::DepthLimitExceeded(pos)
            | Error::BracketMismatch(pos, _, _)
            | Error::InvalidUtf8(pos) => Some(*pos),
            Error::WithContext { source, .. } => source.position(),
            Error::Custom(_)
            | Error::RepairFailed(_)
            | Error::UnbalancedBrackets(_, _)
            | Error::MaxRepairsExceeded(_)
            | Error::InvalidChunk(_) => None,
        }
    }

    #[inline(always)]
    fn is_string_error(&self) -> bool {
        matches!(
            self,
            Error::InvalidEscape(_) | Error::InvalidUnicode(_) | Error::UnterminatedString(_)
        )
    }

    #[inline(always)]
    fn is_number_error(&self) -> bool {
        matches!(self, Error::InvalidNumber(_))
    }

    #[inline(always)]
    fn is_structural_error(&self) -> bool {
        matches!(
            self,
            Error::UnexpectedChar(_, _)
                | Error::UnexpectedEof(_)
                | Error::Expected { .. }
                | Error::TrailingComma(_)
                | Error::DepthLimitExceeded(_)
                | Error::BracketMismatch(_, _, _)
                | Error::UnbalancedBrackets(_, _)
        )
    }
}

/// Checks if an error is related to bracket or brace mismatches that might be repairable.
#[inline(always)]
pub fn is_bracket_mismatch_error(error: &Error) -> bool {
    match error {
        Error::Expected {
            expected, found, ..
        } => {
            // Check if this is a bracket-related expectation error
            expected.contains("}")
                || expected.contains("]")
                || expected.contains("{")
                || expected.contains("[")
                || found.contains("}")
                || found.contains("]")
                || found.contains("{")
                || found.contains("[")
        }
        Error::UnexpectedEof(_) => true, // Missing closing brackets
        Error::UnexpectedChar(ch, _) => {
            matches!(ch, '{' | '}' | '[' | ']')
        }
        Error::BracketMismatch(_, _, _) => true,
        Error::UnbalancedBrackets(_, _) => true,
        _ => false,
    }
}

/// Detects repairs made by comparing original and repaired JSON strings.
pub fn detect_repairs(original: &str, repaired: &str) -> Vec<RepairAction> {
    let mut repairs = Vec::new();

    // Simple diff-based repair detection
    // This is a simplified version - full implementation would use
    // more sophisticated diff algorithms

    let original_brackets = count_brackets(original);
    let repaired_brackets = count_brackets(repaired);

    for (bracket_char, orig_count) in original_brackets.iter() {
        let rep_count = repaired_brackets.get(bracket_char).unwrap_or(&0);

        if orig_count != rep_count {
            repairs.push(RepairAction {
                action_type: if rep_count > orig_count {
                    RepairType::InsertBracket
                } else {
                    RepairType::RemoveBracket
                },
                position: 0, // Would need actual position tracking in full implementation
                original: bracket_char.to_string(),
                replacement: format!("{} (count: {} → {})", bracket_char, orig_count, rep_count),
                description: format!("Balanced {} brackets", bracket_char),
            });
        }
    }

    // Check for new brackets that weren't in original
    for (bracket_char, rep_count) in repaired_brackets.iter() {
        if !original_brackets.contains_key(bracket_char) && *rep_count > 0 {
            repairs.push(RepairAction {
                action_type: RepairType::InsertBracket,
                position: 0,
                original: String::new(),
                replacement: bracket_char.to_string(),
                description: format!("Added {} brackets", bracket_char),
            });
        }
    }

    repairs
}

/// Counts the occurrences of each bracket type in the input string.
fn count_brackets(input: &str) -> FxHashMap<char, usize> {
    let mut counts = FxHashMap::default();
    for ch in input.chars() {
        match ch {
            '{' | '}' | '[' | ']' => {
                *counts.entry(ch).or_insert(0) += 1;
            }
            _ => {}
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_extraction() {
        let error = Error::UnexpectedChar('x', 42);
        assert_eq!(error.position(), Some(42));

        let custom_error = Error::Custom("test".to_string());
        assert_eq!(custom_error.position(), None);
    }

    #[test]
    fn test_error_categorization() {
        let string_error = Error::InvalidEscape(10);
        assert!(string_error.is_string_error());
        assert!(!string_error.is_number_error());
        assert!(!string_error.is_structural_error());

        let number_error = Error::InvalidNumber(5);
        assert!(!number_error.is_string_error());
        assert!(number_error.is_number_error());
        assert!(!number_error.is_structural_error());

        let structural_error = Error::UnexpectedEof(20);
        assert!(!structural_error.is_string_error());
        assert!(!structural_error.is_number_error());
        assert!(structural_error.is_structural_error());
    }

    #[test]
    fn test_bracket_mismatch_detection() {
        let bracket_error = Error::BracketMismatch(10, '{', '}');
        assert!(is_bracket_mismatch_error(&bracket_error));

        let eof_error = Error::UnexpectedEof(20);
        assert!(is_bracket_mismatch_error(&eof_error));

        let expected_error = Error::Expected {
            expected: "closing brace }".to_string(),
            found: "end of input".to_string(),
            position: 15,
        };
        assert!(is_bracket_mismatch_error(&expected_error));

        let string_error = Error::InvalidEscape(5);
        assert!(!is_bracket_mismatch_error(&string_error));
    }

    #[test]
    fn test_bracket_counting() {
        let input = "{[}]";
        let counts = count_brackets(input);
        assert_eq!(counts.get(&'{'), Some(&1));
        assert_eq!(counts.get(&'}'), Some(&1));
        assert_eq!(counts.get(&'['), Some(&1));
        assert_eq!(counts.get(&']'), Some(&1));
    }

    #[test]
    fn test_repair_detection() {
        let original = "{key: value";
        let repaired = "{key: value}";
        let repairs = detect_repairs(original, repaired);

        assert!(!repairs.is_empty());
        assert_eq!(repairs[0].action_type, RepairType::InsertBracket);
        assert!(repairs[0].description.contains("Added"));
    }
}
