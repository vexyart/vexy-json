// this_file: crates/core/src/error/reporter.rs

//! Advanced error reporting with configurable output formatting and context.
//!
//! This module provides comprehensive error reporting functionality including
//! colored terminal output, context sections, suggestions, and recovery strategies.
//! The ErrorReporter can generate detailed reports for debugging and user feedback.

use crate::error::recovery::ErrorRecoveryAnalyzer;
use crate::error::terminal::{ColorScheme, TerminalFormatter};
use crate::error::{Error, Span};

/// Configuration for error report formatting and content.
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Whether to include context lines around the error
    pub include_context: bool,
    /// Number of context lines to show before and after the error
    pub context_lines: usize,
    /// Whether to include error code and description
    pub include_error_code: bool,
    /// Whether to include suggestions for fixing the error
    pub include_suggestions: bool,
    /// Whether to include recovery strategies if available
    pub include_recovery: bool,
    /// Maximum width for line wrapping in suggestions
    pub max_line_width: usize,
    /// Whether to use colored output (auto-detected if None)
    pub use_colors: Option<bool>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_context: true,
            context_lines: 2,
            include_error_code: true,
            include_suggestions: true,
            include_recovery: true,
            max_line_width: 80,
            use_colors: None, // Auto-detect
        }
    }
}

impl ReportConfig {
    /// Creates a minimal report configuration with just the error message.
    pub fn minimal() -> Self {
        Self {
            include_context: false,
            context_lines: 0,
            include_error_code: false,
            include_suggestions: false,
            include_recovery: false,
            max_line_width: 80,
            use_colors: None,
        }
    }

    /// Creates a comprehensive report configuration with all sections enabled.
    pub fn comprehensive() -> Self {
        Self {
            include_context: true,
            context_lines: 3,
            include_error_code: true,
            include_suggestions: true,
            include_recovery: true,
            max_line_width: 100,
            use_colors: None,
        }
    }

    /// Sets whether to use colored output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = Some(use_colors);
        self
    }

    /// Sets the number of context lines to display.
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Sets the maximum line width for text wrapping.
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_line_width = width;
        self
    }
}

/// Advanced error reporter with configurable formatting and content.
///
/// The ErrorReporter provides comprehensive error reporting functionality
/// including colored terminal output, context sections, actionable suggestions,
/// and recovery strategies. It can generate reports suitable for both
/// development debugging and end-user error messages.
pub struct ErrorReporter {
    config: ReportConfig,
    formatter: TerminalFormatter,
}

impl ErrorReporter {
    /// Creates a new error reporter with the specified configuration.
    pub fn new(config: ReportConfig) -> Self {
        let formatter = match config.use_colors {
            Some(true) => TerminalFormatter::colored(),
            Some(false) => TerminalFormatter::plain(),
            None => TerminalFormatter::new(), // Auto-detect
        };

        Self { config, formatter }
    }


    /// Creates a new error reporter for minimal output.
    pub fn minimal() -> Self {
        Self::new(ReportConfig::minimal())
    }

    /// Creates a new error reporter for comprehensive output.
    pub fn comprehensive() -> Self {
        Self::new(ReportConfig::comprehensive())
    }

    /// Generates a complete error report for the given error and input.
    ///
    /// This method produces a comprehensive error report including all configured
    /// sections such as context, error codes, suggestions, and recovery strategies.
    pub fn generate_report(&self, error: &Error, input: &str) -> String {
        let mut report = String::new();

        // Main error header
        self.add_error_header(&mut report, error);

        // Context section (if error has position and context is enabled)
        if self.config.include_context {
            if let Some(span) = error.span() {
                self.add_context_section(&mut report, &span, input);
            }
        }

        // Error code section
        if self.config.include_error_code {
            self.add_error_code_section(&mut report, error);
        }

        // Suggestions section
        if self.config.include_suggestions {
            self.add_suggestions_section(&mut report, error);
        }

        // Recovery section
        if self.config.include_recovery {
            self.add_recovery_section(&mut report, error, input);
        }

        report
    }

    /// Generates a quick error report with just the essential information.
    pub fn quick_report(&self, error: &Error) -> String {
        let error_msg = self
            .formatter
            .format_text(&error.to_string(), ColorScheme::Error);
        let code = error.code();
        let formatted_code = self.formatter.format_error_code(code.as_str());

        format!("[{formatted_code}] {error_msg}")
    }

    /// Adds the main error header to the report.
    fn add_error_header(&self, report: &mut String, error: &Error) {
        let error_header = self.formatter.format_text("Error:", ColorScheme::Error);
        let error_msg = self
            .formatter
            .format_text(&error.to_string(), ColorScheme::Error);

        report.push_str(&format!("{error_header} {error_msg}\n"));
    }

    /// Adds the context section showing the error location in the input.
    fn add_context_section(&self, report: &mut String, span: &Span, input: &str) {
        let context = span.context_window(input, self.config.context_lines);

        if !context.lines.is_empty() {
            report.push('\n');
            let context_header = self.formatter.format_text("Context:", ColorScheme::Info);
            report.push_str(&format!("{context_header}\n"));

            let formatted_context = context.format_with_formatter(&self.formatter);
            report.push_str(&formatted_context);
        }
    }

    /// Adds the error code section with code description.
    fn add_error_code_section(&self, report: &mut String, error: &Error) {
        let code = error.code();

        report.push('\n');
        let code_header = self.formatter.format_text("Error Code:", ColorScheme::Info);
        let formatted_code = self.formatter.format_error_code(code.as_str());
        let description = self
            .formatter
            .format_text(code.description(), ColorScheme::Info);

        report.push_str(&format!("{code_header} {formatted_code} - {description}\n"));
    }

    /// Adds the suggestions section with actionable recommendations.
    fn add_suggestions_section(&self, report: &mut String, error: &Error) {
        let suggestions = error.suggestions();

        if !suggestions.is_empty() {
            report.push('\n');
            let suggestions_header = self
                .formatter
                .format_text("Suggestions:", ColorScheme::Info);
            report.push_str(&format!("{suggestions_header}\n"));

            for (i, suggestion) in suggestions.iter().enumerate() {
                let wrapped_suggestion = self.wrap_text(suggestion, self.config.max_line_width);
                let formatted_suggestion =
                    self.formatter.format_suggestion(i + 1, &wrapped_suggestion);
                report.push_str(&formatted_suggestion);
                report.push('\n');
            }
        }
    }

    /// Adds the recovery section with automatic repair strategies.
    fn add_recovery_section(&self, report: &mut String, error: &Error, input: &str) {
        let analyzer = ErrorRecoveryAnalyzer::new();
        let strategies = analyzer.analyze_error(error, input);

        if !strategies.is_empty() {
            report.push('\n');
            let recovery_header = self
                .formatter
                .format_text("Automatic Recovery:", ColorScheme::Success);
            report.push_str(&format!("{recovery_header}\n"));

            for (i, strategy) in strategies.iter().enumerate() {
                let strategy_desc = self
                    .formatter
                    .format_text(&strategy.description(), ColorScheme::Success);
                report.push_str(&format!("  {}. {}\n", i + 1, strategy_desc));
            }

            // Check if any strategy has low confidence
            let min_confidence = strategies
                .iter()
                .map(|s| s.confidence())
                .fold(1.0, f32::min);
            if min_confidence < 0.8 {
                let warning = self.formatter.format_text(
                    "Note: Automatic recovery may not be perfect. Manual review recommended.",
                    ColorScheme::Warning,
                );
                report.push_str(&format!("\n{warning}\n"));
            }
        }
    }

    /// Wraps text to the specified width, preserving words.
    fn wrap_text(&self, text: &str, max_width: usize) -> String {
        if text.len() <= max_width {
            return text.to_string();
        }

        let mut wrapped = String::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > max_width && !current_line.is_empty() {
                wrapped.push_str(&current_line);
                wrapped.push('\n');
                current_line.clear();
            }

            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }

        if !current_line.is_empty() {
            wrapped.push_str(&current_line);
        }

        wrapped
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new(ReportConfig::default())
    }
}

/// Convenience function for generating a quick error report.
///
/// This function provides a simple way to generate a basic error report
/// without needing to create an ErrorReporter instance.
pub fn quick_error_report(error: &Error) -> String {
    let reporter = ErrorReporter::minimal();
    reporter.quick_report(error)
}

/// Convenience function for generating a comprehensive error report.
///
/// This function provides a simple way to generate a full error report
/// with all available information and formatting.
pub fn full_error_report(error: &Error, input: &str) -> String {
    let reporter = ErrorReporter::comprehensive();
    reporter.generate_report(error, input)
}

/// Convenience function for generating a plain text error report.
///
/// This function generates an error report without any color formatting,
/// suitable for logging or non-terminal output.
pub fn plain_error_report(error: &Error, input: &str) -> String {
    let config = ReportConfig::comprehensive().with_colors(false);
    let reporter = ErrorReporter::new(config);
    reporter.generate_report(error, input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_report_config_creation() {
        let config = ReportConfig::default();
        assert!(config.include_context);
        assert_eq!(config.context_lines, 2);
        assert!(config.include_suggestions);

        let minimal = ReportConfig::minimal();
        assert!(!minimal.include_context);
        assert!(!minimal.include_suggestions);

        let comprehensive = ReportConfig::comprehensive();
        assert!(comprehensive.include_context);
        assert!(comprehensive.include_suggestions);
        assert!(comprehensive.include_recovery);
    }

    #[test]
    fn test_error_reporter_creation() {
        let reporter = ErrorReporter::default();
        assert!(reporter.config.include_suggestions);

        let minimal = ErrorReporter::minimal();
        assert!(!minimal.config.include_suggestions);

        let comprehensive = ErrorReporter::comprehensive();
        assert!(comprehensive.config.include_recovery);
    }

    #[test]
    fn test_quick_error_report() {
        let error = Error::UnexpectedChar('x', 5);
        let report = quick_error_report(&error);
        assert!(report.contains("E1001"));
        assert!(report.contains("Unexpected character"));
    }

    #[test]
    fn test_text_wrapping() {
        let config = ReportConfig::default().with_max_width(10);
        let reporter = ErrorReporter::new(config);

        let wrapped = reporter.wrap_text("This is a very long text that should be wrapped", 10);
        let lines: Vec<&str> = wrapped.lines().collect();
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| line.len() <= 10));
    }

    #[test]
    fn test_comprehensive_report() {
        let error = Error::InvalidNumber(15);
        let input = "{ \"value\": 123.45.67 }";

        let report = full_error_report(&error, input);
        assert!(report.contains("Error:"));
        assert!(report.contains("Context:"));
        assert!(report.contains("Error Code:"));
        assert!(report.contains("Suggestions:"));
    }
}
