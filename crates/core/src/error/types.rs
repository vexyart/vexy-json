// this_file: src/error/types.rs

use crate::error::terminal::{ColorScheme, TerminalFormatter};
use crate::error::Span;
use thiserror::Error;

/// Structured error codes for programmatic error handling.
///
/// Each error code has a unique identifier and provides context-aware
/// suggestions for fixing the associated error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// E1001: Unexpected character encountered
    UnexpectedCharacter,
    /// E1002: Unexpected end of input
    UnexpectedEndOfInput,
    /// E1003: Invalid number format
    InvalidNumberFormat,
    /// E1004: Invalid escape sequence in string
    InvalidEscapeSequence,
    /// E1005: Invalid unicode escape sequence
    InvalidUnicodeEscape,
    /// E1006: Unterminated string literal
    UnterminatedString,
    /// E1007: Trailing comma found
    TrailingComma,
    /// E1008: Expected specific token
    ExpectedToken,
    /// E1009: Maximum nesting depth exceeded
    DepthLimitExceeded,
    /// E1010: Custom error condition
    Custom,
    /// E1011: Error with additional context
    WithContext,
    /// E1012: JSON repair operation failed
    RepairFailed,
    /// E1013: Bracket mismatch detected
    BracketMismatch,
    /// E1014: Unbalanced brackets in structure
    UnbalancedBrackets,
    /// E1015: Maximum repair attempts exceeded
    MaxRepairsExceeded,
    /// E1016: Invalid UTF-8 sequence
    InvalidUtf8,
    /// E1017: Invalid chunk for parallel processing
    InvalidChunk,
}

impl ErrorCode {
    /// Returns the error code as a string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::UnexpectedCharacter => "E1001",
            ErrorCode::UnexpectedEndOfInput => "E1002",
            ErrorCode::InvalidNumberFormat => "E1003",
            ErrorCode::InvalidEscapeSequence => "E1004",
            ErrorCode::InvalidUnicodeEscape => "E1005",
            ErrorCode::UnterminatedString => "E1006",
            ErrorCode::TrailingComma => "E1007",
            ErrorCode::ExpectedToken => "E1008",
            ErrorCode::DepthLimitExceeded => "E1009",
            ErrorCode::Custom => "E1010",
            ErrorCode::WithContext => "E1011",
            ErrorCode::RepairFailed => "E1012",
            ErrorCode::BracketMismatch => "E1013",
            ErrorCode::UnbalancedBrackets => "E1014",
            ErrorCode::MaxRepairsExceeded => "E1015",
            ErrorCode::InvalidUtf8 => "E1016",
            ErrorCode::InvalidChunk => "E1017",
        }
    }

    /// Returns a brief description of the error code.
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::UnexpectedCharacter => "Unexpected character encountered",
            ErrorCode::UnexpectedEndOfInput => "Unexpected end of input",
            ErrorCode::InvalidNumberFormat => "Invalid number format",
            ErrorCode::InvalidEscapeSequence => "Invalid escape sequence",
            ErrorCode::InvalidUnicodeEscape => "Invalid unicode escape sequence",
            ErrorCode::UnterminatedString => "Unterminated string literal",
            ErrorCode::TrailingComma => "Trailing comma found",
            ErrorCode::ExpectedToken => "Expected specific token",
            ErrorCode::DepthLimitExceeded => "Maximum nesting depth exceeded",
            ErrorCode::Custom => "Custom error condition",
            ErrorCode::WithContext => "Error with additional context",
            ErrorCode::RepairFailed => "JSON repair operation failed",
            ErrorCode::BracketMismatch => "Bracket mismatch detected",
            ErrorCode::UnbalancedBrackets => "Unbalanced brackets in structure",
            ErrorCode::MaxRepairsExceeded => "Maximum repair attempts exceeded",
            ErrorCode::InvalidUtf8 => "Invalid UTF-8 sequence",
            ErrorCode::InvalidChunk => "Invalid chunk for parallel processing",
        }
    }

    /// Returns context-aware suggestions for fixing this error.
    pub fn suggestions(&self) -> Vec<&'static str> {
        match self {
            ErrorCode::UnexpectedCharacter => vec![
                "Check for typos in your JSON syntax",
                "Ensure proper quoting of strings",
                "Verify bracket and brace matching",
                "Remove any non-JSON characters",
            ],
            ErrorCode::UnexpectedEndOfInput => vec![
                "Check for unclosed strings, objects, or arrays",
                "Ensure the JSON document is complete",
                "Verify all brackets and braces are properly closed",
                "Add missing closing characters",
            ],
            ErrorCode::InvalidNumberFormat => vec![
                "Check for leading zeros in numbers",
                "Ensure decimal numbers have digits after the decimal point",
                "Verify exponential notation is properly formatted",
                "Remove any non-numeric characters from number literals",
            ],
            ErrorCode::InvalidEscapeSequence => vec![
                "Use valid escape sequences: \\n, \\t, \\r, \\\\, \\\", \\/",
                "For unicode escapes, use \\uXXXX format with 4 hex digits",
                "Remove or correct invalid escape sequences",
                "Double-check backslash usage in strings",
            ],
            ErrorCode::InvalidUnicodeEscape => vec![
                "Use exactly 4 hexadecimal digits after \\u",
                "Ensure hex digits are 0-9, A-F, or a-f",
                "For characters above U+FFFF, use surrogate pairs",
                "Consider using the actual Unicode character instead",
            ],
            ErrorCode::UnterminatedString => vec![
                "Add closing quote to string literal",
                "Check for unescaped quotes within strings",
                "Ensure proper escaping of special characters",
                "Verify string spans don't cross line boundaries",
            ],
            ErrorCode::TrailingComma => vec![
                "Remove trailing comma after last element",
                "Add another element after the comma",
                "Use a parser that allows trailing commas",
                "Check array and object syntax",
            ],
            ErrorCode::ExpectedToken => vec![
                "Check JSON syntax for missing colons or commas",
                "Ensure proper structure of objects and arrays",
                "Verify key-value pairs in objects",
                "Check for missing or extra punctuation",
            ],
            ErrorCode::DepthLimitExceeded => vec![
                "Reduce nesting depth of objects and arrays",
                "Flatten deeply nested structures",
                "Use references or IDs instead of deep nesting",
                "Consider alternative data organization",
            ],
            ErrorCode::Custom => vec![
                "Check the specific error message for details",
                "Verify input data meets expected format",
                "Consult documentation for specific requirements",
                "Review parser configuration and options",
            ],
            ErrorCode::WithContext => vec![
                "Review the wrapped error message for specific details",
                "Check the context information provided",
                "Address the underlying error cause",
                "Verify the parsing context is correct",
            ],
            ErrorCode::RepairFailed => vec![
                "Try manual correction of the JSON syntax",
                "Check for severe structural issues",
                "Verify the input is actually JSON-like",
                "Consider using a different parsing strategy",
            ],
            ErrorCode::BracketMismatch => vec![
                "Check for matching brackets: [], {}, ()",
                "Ensure proper nesting order",
                "Verify no extra or missing brackets",
                "Use a code editor with bracket matching",
            ],
            ErrorCode::UnbalancedBrackets => vec![
                "Count opening and closing brackets",
                "Add missing closing brackets",
                "Remove extra opening brackets",
                "Check for proper nesting structure",
            ],
            ErrorCode::MaxRepairsExceeded => vec![
                "Simplify the JSON structure",
                "Fix obvious syntax errors manually",
                "Use a more lenient parser",
                "Break down complex structures into smaller parts",
            ],
            ErrorCode::InvalidUtf8 => vec![
                "Ensure input is valid UTF-8 encoded text",
                "Check for encoding issues in the source file",
                "Use a different encoding or text editor",
                "Validate the input before parsing",
            ],
            ErrorCode::InvalidChunk => vec![
                "Use smaller chunk sizes for parallel processing",
                "Ensure JSON structure is well-formed",
                "Try sequential parsing instead of parallel",
                "Check for corrupted input data",
            ],
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error types that can occur during vexy_json parsing.
///
/// Each error variant contains positional information to help users
/// locate and fix parsing issues in their input. The error types
/// are designed to provide clear, actionable feedback.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Unexpected character encountered during parsing.
    /// Contains the character and its position in the input.
    #[error("Unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),

    /// Unexpected end of input while parsing.
    /// Contains the position where more input was expected.
    #[error("Unexpected end of input at position {0}")]
    UnexpectedEof(usize),

    /// Invalid number format encountered.
    /// Contains the position where the invalid number starts.
    #[error("Invalid number format at position {0}")]
    InvalidNumber(usize),

    /// Invalid escape sequence in string.
    /// Contains the position of the invalid escape.
    #[error("Invalid string escape sequence at position {0}")]
    InvalidEscape(usize),

    /// Invalid unicode escape sequence.
    /// Contains the position of the invalid unicode escape.
    #[error("Invalid Unicode escape at position {0}")]
    InvalidUnicode(usize),

    /// String literal was not properly terminated.
    /// Contains the position where the string started.
    #[error("Unterminated string starting at position {0}")]
    UnterminatedString(usize),

    /// Trailing comma found when not allowed by parser options.
    /// Contains the position of the trailing comma.
    #[error("Trailing comma at position {0}")]
    TrailingComma(usize),

    /// Expected a specific token or value but found something else.
    /// This is the most flexible error type for parser expectations.
    #[error("Expected {expected} but found {found} at position {position}")]
    Expected {
        /// Description of what was expected.
        expected: String,
        /// Description of what was actually found.
        found: String,
        /// Position in the input where the mismatch occurred.
        position: usize,
    },

    /// Maximum recursion depth exceeded while parsing nested structures.
    /// Contains the position where the limit was exceeded.
    #[error("Depth limit exceeded at position {0}")]
    DepthLimitExceeded(usize),

    /// Custom error with a descriptive message.
    /// Used for configuration errors or other non-positional issues.
    #[error("Custom error: {0}")]
    Custom(String),

    /// An error that wraps another error, adding context.
    #[error("{message}")]
    WithContext {
        /// A descriptive message for the context.
        message: String,
        /// The source error.
        #[source]
        source: Box<Error>,
    },

    /// JSON repair failed with the given error message.
    #[error("JSON repair failed: {0}")]
    RepairFailed(String),

    /// Bracket mismatch detected during parsing.
    #[error("Bracket mismatch at position {0}: expected {1}, found {2}")]
    BracketMismatch(usize, char, char),

    /// Unbalanced brackets detected.
    #[error("Unbalanced brackets: {0} extra opening, {1} extra closing")]
    UnbalancedBrackets(usize, usize),

    /// Maximum repair attempts exceeded.
    #[error("Maximum repair attempts exceeded ({0})")]
    MaxRepairsExceeded(usize),

    /// Invalid UTF-8 sequence encountered.
    #[error("Invalid UTF-8 sequence at position {0}")]
    InvalidUtf8(usize),

    /// Invalid chunk detected during parallel processing.
    #[error("Invalid chunk: {0}")]
    InvalidChunk(String),
}

impl Error {
    /// Returns the error code for this error.
    ///
    /// Error codes provide a structured way to identify and categorize
    /// different types of parsing errors programmatically.
    #[inline(always)]
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::UnexpectedChar(_, _) => ErrorCode::UnexpectedCharacter,
            Error::UnexpectedEof(_) => ErrorCode::UnexpectedEndOfInput,
            Error::InvalidNumber(_) => ErrorCode::InvalidNumberFormat,
            Error::InvalidEscape(_) => ErrorCode::InvalidEscapeSequence,
            Error::InvalidUnicode(_) => ErrorCode::InvalidUnicodeEscape,
            Error::UnterminatedString(_) => ErrorCode::UnterminatedString,
            Error::TrailingComma(_) => ErrorCode::TrailingComma,
            Error::Expected { .. } => ErrorCode::ExpectedToken,
            Error::DepthLimitExceeded(_) => ErrorCode::DepthLimitExceeded,
            Error::Custom(_) => ErrorCode::Custom,
            Error::WithContext { .. } => ErrorCode::WithContext,
            Error::RepairFailed(_) => ErrorCode::RepairFailed,
            Error::BracketMismatch(_, _, _) => ErrorCode::BracketMismatch,
            Error::UnbalancedBrackets(_, _) => ErrorCode::UnbalancedBrackets,
            Error::MaxRepairsExceeded(_) => ErrorCode::MaxRepairsExceeded,
            Error::InvalidUtf8(_) => ErrorCode::InvalidUtf8,
            Error::InvalidChunk(_) => ErrorCode::InvalidChunk,
        }
    }

    /// Returns the position in the input where the error occurred, if available.
    ///
    /// Most parsing errors have associated position information to help
    /// users locate the problematic input. Custom errors may not have
    /// position information and will return None.
    pub fn position(&self) -> Option<usize> {
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

    /// Returns a span covering the error location, if available.
    ///
    /// This provides more precise location information than just a position,
    /// including the range of characters that caused the error.
    pub fn span(&self) -> Option<Span> {
        self.position().map(Span::single)
    }

    /// Returns suggestions for fixing this error.
    ///
    /// Provides context-aware suggestions based on the error type and
    /// the specific situation that caused the error.
    pub fn suggestions(&self) -> Vec<&'static str> {
        self.code().suggestions()
    }

    /// Returns a detailed diagnostic message including error code and suggestions.
    ///
    /// This provides a comprehensive error report that includes the original
    /// error message, error code, and actionable suggestions for fixing the issue.
    /// For colored output, use `diagnostic_colored` or `diagnostic_with_formatter`.
    pub fn diagnostic(&self) -> String {
        let code = self.code();
        let suggestions = self.suggestions();

        let mut diagnostic = format!("[{}] {}", code.as_str(), self);

        if !suggestions.is_empty() {
            diagnostic.push_str("\n\nSuggestions:");
            for (i, suggestion) in suggestions.iter().enumerate() {
                diagnostic.push_str(&format!("\n  {}. {}", i + 1, suggestion));
            }
        }

        diagnostic
    }

    /// Returns a detailed diagnostic message with colored terminal formatting.
    ///
    /// This method provides the same comprehensive error report as `diagnostic()`
    /// but uses colored output when the terminal supports it. Colors enhance
    /// readability by highlighting error codes, messages, and suggestions.
    pub fn diagnostic_with_formatter(&self, formatter: &TerminalFormatter) -> String {
        let code = self.code();
        let suggestions = self.suggestions();

        // Format the error code with error color scheme
        let formatted_code = formatter.format_error_code(code.as_str());

        // Format the main error message
        let error_msg = formatter.format_text(&self.to_string(), ColorScheme::Error);

        let mut diagnostic = format!("[{formatted_code}] {error_msg}");

        if !suggestions.is_empty() {
            // Format suggestions header
            let suggestions_header = formatter.format_text("Suggestions:", ColorScheme::Info);
            diagnostic.push_str(&format!("\n\n{suggestions_header}"));

            for (i, suggestion) in suggestions.iter().enumerate() {
                let formatted_suggestion = formatter.format_suggestion(i + 1, suggestion);
                diagnostic.push_str(&format!("\n  {}. {}", i + 1, formatted_suggestion));
            }
        }

        diagnostic
    }

    /// Returns a detailed diagnostic message with colored output if terminal supports it.
    ///
    /// This is a convenience method that automatically detects terminal capabilities
    /// and uses colored output when available, falling back to plain text otherwise.
    pub fn diagnostic_colored(&self) -> String {
        let formatter = TerminalFormatter::new();
        self.diagnostic_with_formatter(&formatter)
    }

    /// Returns a detailed diagnostic message with plain text output (no colors).
    ///
    /// This is a convenience method for when you specifically want plain text output
    /// regardless of terminal capabilities.
    pub fn diagnostic_plain(&self) -> String {
        let formatter = TerminalFormatter::plain();
        self.diagnostic_with_formatter(&formatter)
    }

    /// Creates a new error with additional context.
    ///
    /// This allows wrapping an existing error with additional information
    /// while preserving the original error details.
    pub fn with_context<S: Into<String>>(self, message: S) -> Self {
        Error::WithContext {
            message: message.into(),
            source: Box::new(self),
        }
    }

    /// Creates a new error with a custom context message and span.
    ///
    /// This is useful when you need to provide additional context about
    /// what was being parsed when the error occurred.
    pub fn with_parsing_context<S: Into<String>>(self, context: S, _span: Span) -> Self {
        let context_msg = format!("While parsing {}: {}", context.into(), self);
        Error::WithContext {
            message: context_msg,
            source: Box::new(self),
        }
    }

    /// Checks if this error is related to string parsing.
    ///
    /// Useful for error categorization and specialized error handling.
    #[inline(always)]
    pub fn is_string_error(&self) -> bool {
        matches!(
            self,
            Error::InvalidEscape(_) | Error::InvalidUnicode(_) | Error::UnterminatedString(_)
        )
    }

    /// Checks if this error is related to number parsing.
    ///
    /// Useful for error categorization and specialized error handling.
    #[inline(always)]
    pub fn is_number_error(&self) -> bool {
        matches!(self, Error::InvalidNumber(_))
    }

    /// Checks if this error is a structural parsing error.
    ///
    /// Structural errors include bracket mismatches, unexpected tokens, etc.
    #[inline(always)]
    pub fn is_structural_error(&self) -> bool {
        matches!(
            self,
            Error::UnexpectedChar(_, _)
                | Error::UnexpectedEof(_)
                | Error::Expected { .. }
                | Error::TrailingComma(_)
                | Error::DepthLimitExceeded(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_as_str() {
        assert_eq!(ErrorCode::UnexpectedCharacter.as_str(), "E1001");
        assert_eq!(ErrorCode::UnexpectedEndOfInput.as_str(), "E1002");
        assert_eq!(ErrorCode::InvalidNumberFormat.as_str(), "E1003");
        assert_eq!(ErrorCode::InvalidEscapeSequence.as_str(), "E1004");
        assert_eq!(ErrorCode::InvalidUnicodeEscape.as_str(), "E1005");
        assert_eq!(ErrorCode::UnterminatedString.as_str(), "E1006");
        assert_eq!(ErrorCode::TrailingComma.as_str(), "E1007");
        assert_eq!(ErrorCode::ExpectedToken.as_str(), "E1008");
        assert_eq!(ErrorCode::DepthLimitExceeded.as_str(), "E1009");
        assert_eq!(ErrorCode::Custom.as_str(), "E1010");
        assert_eq!(ErrorCode::WithContext.as_str(), "E1011");
        assert_eq!(ErrorCode::RepairFailed.as_str(), "E1012");
        assert_eq!(ErrorCode::BracketMismatch.as_str(), "E1013");
        assert_eq!(ErrorCode::UnbalancedBrackets.as_str(), "E1014");
        assert_eq!(ErrorCode::MaxRepairsExceeded.as_str(), "E1015");
        assert_eq!(ErrorCode::InvalidUtf8.as_str(), "E1016");
        assert_eq!(ErrorCode::InvalidChunk.as_str(), "E1017");
    }

    #[test]
    fn test_error_code_description() {
        assert_eq!(ErrorCode::UnexpectedCharacter.description(), "Unexpected character encountered");
        assert_eq!(ErrorCode::InvalidNumberFormat.description(), "Invalid number format");
        assert_eq!(ErrorCode::TrailingComma.description(), "Trailing comma found");
        assert_eq!(ErrorCode::DepthLimitExceeded.description(), "Maximum nesting depth exceeded");
    }

    #[test]
    fn test_error_code_suggestions() {
        let suggestions = ErrorCode::UnexpectedCharacter.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"Check for typos in your JSON syntax"));

        let number_suggestions = ErrorCode::InvalidNumberFormat.suggestions();
        assert!(number_suggestions.contains(&"Check for leading zeros in numbers"));
        assert!(number_suggestions.contains(&"Ensure decimal numbers have digits after the decimal point"));
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::UnexpectedCharacter.to_string(), "E1001");
        assert_eq!(ErrorCode::InvalidNumberFormat.to_string(), "E1003");
    }

    #[test]
    fn test_error_code() {
        let error = Error::UnexpectedChar('x', 5);
        assert_eq!(error.code(), ErrorCode::UnexpectedCharacter);

        let error = Error::InvalidNumber(10);
        assert_eq!(error.code(), ErrorCode::InvalidNumberFormat);

        let error = Error::TrailingComma(20);
        assert_eq!(error.code(), ErrorCode::TrailingComma);

        let error = Error::Expected {
            expected: "value".to_string(),
            found: "EOF".to_string(),
            position: 15,
        };
        assert_eq!(error.code(), ErrorCode::ExpectedToken);
    }

    #[test]
    fn test_error_position() {
        let error = Error::UnexpectedChar('x', 5);
        assert_eq!(error.position(), Some(5));

        let error = Error::InvalidNumber(10);
        assert_eq!(error.position(), Some(10));

        let error = Error::Expected {
            expected: "value".to_string(),
            found: "EOF".to_string(),
            position: 15,
        };
        assert_eq!(error.position(), Some(15));

        let error = Error::Custom("Custom error".to_string());
        assert_eq!(error.position(), None);

        let error = Error::UnbalancedBrackets(1, 2);
        assert_eq!(error.position(), None);
    }

    #[test]
    fn test_error_span() {
        let error = Error::UnexpectedChar('x', 5);
        let span = error.span();
        assert_eq!(span, Some(Span::single(5)));

        let error = Error::Custom("Custom error".to_string());
        assert_eq!(error.span(), None);
    }

    #[test]
    fn test_error_suggestions() {
        let error = Error::UnexpectedChar('x', 5);
        let suggestions = error.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"Check for typos in your JSON syntax"));

        let error = Error::InvalidNumber(10);
        let suggestions = error.suggestions();
        assert!(suggestions.contains(&"Check for leading zeros in numbers"));
    }

    #[test]
    fn test_error_diagnostic() {
        let error = Error::UnexpectedChar('x', 5);
        let diagnostic = error.diagnostic();
        assert!(diagnostic.contains("[E1001]"));
        assert!(diagnostic.contains("Unexpected character"));
        assert!(diagnostic.contains("Suggestions:"));
        assert!(diagnostic.contains("Check for typos"));
    }

    #[test]
    fn test_error_with_context() {
        let base_error = Error::InvalidNumber(10);
        let wrapped_error = base_error.with_context("parsing array element");
        
        assert_eq!(wrapped_error.code(), ErrorCode::WithContext);
        assert_eq!(wrapped_error.position(), Some(10)); // Should delegate to source
        
        match wrapped_error {
            Error::WithContext { message, source } => {
                assert_eq!(message, "parsing array element");
                assert_eq!(*source.as_ref(), Error::InvalidNumber(10));
            }
            _ => panic!("Expected WithContext error"),
        }
    }

    #[test]
    fn test_error_with_parsing_context() {
        let base_error = Error::UnexpectedChar('x', 5);
        let wrapped_error = base_error.with_parsing_context("object key", Span::single(5));
        
        assert_eq!(wrapped_error.code(), ErrorCode::WithContext);
        
        match wrapped_error {
            Error::WithContext { message, source } => {
                assert!(message.contains("While parsing object key"));
                assert_eq!(*source.as_ref(), Error::UnexpectedChar('x', 5));
            }
            _ => panic!("Expected WithContext error"),
        }
    }

    #[test]
    fn test_error_is_string_error() {
        assert!(Error::InvalidEscape(5).is_string_error());
        assert!(Error::InvalidUnicode(10).is_string_error());
        assert!(Error::UnterminatedString(0).is_string_error());
        
        assert!(!Error::InvalidNumber(5).is_string_error());
        assert!(!Error::UnexpectedChar('x', 5).is_string_error());
    }

    #[test]
    fn test_error_is_number_error() {
        assert!(Error::InvalidNumber(5).is_number_error());
        
        assert!(!Error::InvalidEscape(5).is_number_error());
        assert!(!Error::UnexpectedChar('x', 5).is_number_error());
    }

    #[test]
    fn test_error_is_structural_error() {
        assert!(Error::UnexpectedChar('x', 5).is_structural_error());
        assert!(Error::UnexpectedEof(10).is_structural_error());
        assert!(Error::TrailingComma(15).is_structural_error());
        assert!(Error::DepthLimitExceeded(20).is_structural_error());
        assert!(Error::Expected {
            expected: "value".to_string(),
            found: "EOF".to_string(),
            position: 25,
        }.is_structural_error());
        
        assert!(!Error::InvalidNumber(5).is_structural_error());
        assert!(!Error::InvalidEscape(5).is_structural_error());
        assert!(!Error::Custom("error".to_string()).is_structural_error());
    }

    #[test]
    fn test_error_display_formatting() {
        let error = Error::UnexpectedChar('x', 5);
        assert_eq!(error.to_string(), "Unexpected character 'x' at position 5");

        let error = Error::InvalidNumber(10);
        assert_eq!(error.to_string(), "Invalid number format at position 10");

        let error = Error::Expected {
            expected: "value".to_string(),
            found: "EOF".to_string(),
            position: 15,
        };
        assert_eq!(error.to_string(), "Expected value but found EOF at position 15");

        let error = Error::Custom("Custom error message".to_string());
        assert_eq!(error.to_string(), "Custom error: Custom error message");
    }

    #[test]
    fn test_error_equality() {
        let error1 = Error::UnexpectedChar('x', 5);
        let error2 = Error::UnexpectedChar('x', 5);
        let error3 = Error::UnexpectedChar('y', 5);
        let error4 = Error::UnexpectedChar('x', 6);

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);

        let error5 = Error::InvalidNumber(10);
        let error6 = Error::InvalidNumber(10);
        let error7 = Error::InvalidNumber(11);

        assert_eq!(error5, error6);
        assert_ne!(error5, error7);
        assert_ne!(error1, error5);
    }

    #[test]
    fn test_nested_error_position() {
        let base_error = Error::InvalidNumber(10);
        let wrapped_error = base_error.with_context("parsing array element");
        
        // Position should delegate to the source error
        assert_eq!(wrapped_error.position(), Some(10));
        
        // Double wrapping should still work
        let double_wrapped = wrapped_error.with_context("parsing JSON document");
        assert_eq!(double_wrapped.position(), Some(10));
    }
}
