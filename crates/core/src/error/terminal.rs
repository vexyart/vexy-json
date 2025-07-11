// this_file: crates/core/src/error/terminal.rs

//! Terminal output formatting with ANSI color support for enhanced error reporting.
//!
//! This module provides colored terminal output capabilities for better error visibility
//! and user experience. It includes automatic color detection, fallback support for
//! non-color terminals, and a comprehensive color scheme for different error types.

use std::env;
use std::io::{self, IsTerminal};

/// ANSI color codes for terminal formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiColors;

impl AnsiColors {
    // Reset and control codes
    /// ANSI reset code to clear all formatting.
    pub const RESET: &'static str = "\x1b[0m";
    /// ANSI code for bold text.
    pub const BOLD: &'static str = "\x1b[1m";
    /// ANSI code for dim text.
    pub const DIM: &'static str = "\x1b[2m";
    /// ANSI code for italic text.
    pub const ITALIC: &'static str = "\x1b[3m";
    /// ANSI code for underlined text.
    pub const UNDERLINE: &'static str = "\x1b[4m";

    // Text colors
    /// ANSI code for red text.
    pub const RED: &'static str = "\x1b[31m";
    /// ANSI code for green text.
    pub const GREEN: &'static str = "\x1b[32m";
    /// ANSI code for yellow text.
    pub const YELLOW: &'static str = "\x1b[33m";
    /// ANSI code for blue text.
    pub const BLUE: &'static str = "\x1b[34m";
    /// ANSI code for magenta text.
    pub const MAGENTA: &'static str = "\x1b[35m";
    /// ANSI code for cyan text.
    pub const CYAN: &'static str = "\x1b[36m";
    /// ANSI code for white text.
    pub const WHITE: &'static str = "\x1b[37m";
    /// ANSI code for gray text.
    pub const GRAY: &'static str = "\x1b[90m";

    // Bright text colors
    /// ANSI code for bright red text.
    pub const BRIGHT_RED: &'static str = "\x1b[91m";
    /// ANSI code for bright green text.
    pub const BRIGHT_GREEN: &'static str = "\x1b[92m";
    /// ANSI code for bright yellow text.
    pub const BRIGHT_YELLOW: &'static str = "\x1b[93m";
    /// ANSI code for bright blue text.
    pub const BRIGHT_BLUE: &'static str = "\x1b[94m";
    /// ANSI code for bright magenta text.
    pub const BRIGHT_MAGENTA: &'static str = "\x1b[95m";
    /// ANSI code for bright cyan text.
    pub const BRIGHT_CYAN: &'static str = "\x1b[96m";
    /// ANSI code for bright white text.
    pub const BRIGHT_WHITE: &'static str = "\x1b[97m";

    // Background colors
    /// ANSI code for red background.
    pub const BG_RED: &'static str = "\x1b[41m";
    /// ANSI code for green background.
    pub const BG_GREEN: &'static str = "\x1b[42m";
    /// ANSI code for yellow background.
    pub const BG_YELLOW: &'static str = "\x1b[43m";
    /// ANSI code for blue background.
    pub const BG_BLUE: &'static str = "\x1b[44m";
    /// ANSI code for magenta background.
    pub const BG_MAGENTA: &'static str = "\x1b[45m";
    /// ANSI code for cyan background.
    pub const BG_CYAN: &'static str = "\x1b[46m";
    /// ANSI code for white background.
    pub const BG_WHITE: &'static str = "\x1b[47m";
}

/// Color scheme for different types of output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    /// Error messages and critical issues
    Error,
    /// Warning messages and non-critical issues  
    Warning,
    /// Informational messages and suggestions
    Info,
    /// Success messages and positive feedback
    Success,
    /// Highlighted text and important content
    Highlight,
    /// Secondary text and metadata
    Secondary,
    /// Line numbers and structural elements
    LineNumbers,
    /// Error codes and identifiers
    ErrorCode,
    /// Context and background information
    Context,
}

impl ColorScheme {
    /// Returns the ANSI color code for this color scheme.
    pub fn color_code(&self) -> &'static str {
        match self {
            ColorScheme::Error => AnsiColors::BRIGHT_RED,
            ColorScheme::Warning => AnsiColors::BRIGHT_YELLOW,
            ColorScheme::Info => AnsiColors::BRIGHT_BLUE,
            ColorScheme::Success => AnsiColors::BRIGHT_GREEN,
            ColorScheme::Highlight => AnsiColors::BRIGHT_CYAN,
            ColorScheme::Secondary => AnsiColors::GRAY,
            ColorScheme::LineNumbers => AnsiColors::BLUE,
            ColorScheme::ErrorCode => AnsiColors::MAGENTA,
            ColorScheme::Context => AnsiColors::WHITE,
        }
    }

    /// Returns the ANSI code with bold formatting.
    pub fn bold_color_code(&self) -> String {
        format!("{}{}", AnsiColors::BOLD, self.color_code())
    }
}

/// Terminal formatter that handles colored output with fallback support.
#[derive(Debug, Clone)]
pub struct TerminalFormatter {
    /// Whether colors are enabled for this formatter
    colors_enabled: bool,
    /// Whether to force color output regardless of terminal detection
    force_colors: bool,
}

impl Default for TerminalFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalFormatter {
    /// Creates a new terminal formatter with automatic color detection.
    pub fn new() -> Self {
        let colors_enabled = Self::should_use_colors();
        Self {
            colors_enabled,
            force_colors: false,
        }
    }

    /// Creates a new formatter that always uses colors.
    pub fn with_colors() -> Self {
        Self {
            colors_enabled: true,
            force_colors: true,
        }
    }

    /// Creates a new formatter that always uses colors (alias for with_colors).
    pub fn colored() -> Self {
        Self::with_colors()
    }

    /// Creates a new formatter that never uses colors.
    pub fn without_colors() -> Self {
        Self {
            colors_enabled: false,
            force_colors: false,
        }
    }

    /// Creates a new formatter that never uses colors (alias for without_colors).
    pub fn plain() -> Self {
        Self::without_colors()
    }

    /// Detects whether the current environment supports colored output.
    ///
    /// This checks several factors:
    /// - Whether stdout is a terminal (TTY)
    /// - Environment variables like NO_COLOR, FORCE_COLOR, TERM
    /// - Platform-specific terminal detection
    pub fn should_use_colors() -> bool {
        // Check NO_COLOR environment variable (standard)
        if env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Check FORCE_COLOR environment variable
        if env::var("FORCE_COLOR").is_ok() {
            return true;
        }

        // Check if we're in a terminal
        if !io::stderr().is_terminal() {
            return false;
        }

        // Check TERM environment variable
        match env::var("TERM") {
            Ok(term) => {
                // Don't use colors for dumb terminals
                if term == "dumb" {
                    return false;
                }
                // Most modern terminals support colors
                true
            }
            Err(_) => {
                // No TERM variable, probably not a color terminal
                false
            }
        }
    }

    /// Formats text with the specified color scheme.
    pub fn colorize(&self, text: &str, scheme: ColorScheme) -> String {
        if !self.colors_enabled {
            return text.to_string();
        }

        format!("{}{}{}", scheme.color_code(), text, AnsiColors::RESET)
    }

    /// Formats text with the specified color scheme (alias for colorize).
    pub fn format_text(&self, text: &str, scheme: ColorScheme) -> String {
        self.colorize(text, scheme)
    }

    /// Formats text with bold styling and color.
    pub fn bold_colorize(&self, text: &str, scheme: ColorScheme) -> String {
        if !self.colors_enabled {
            return text.to_string();
        }

        format!("{}{}{}", scheme.bold_color_code(), text, AnsiColors::RESET)
    }

    /// Formats text with underline styling and color.
    pub fn underline_colorize(&self, text: &str, scheme: ColorScheme) -> String {
        if !self.colors_enabled {
            return text.to_string();
        }

        format!(
            "{}{}{}{}",
            AnsiColors::UNDERLINE,
            scheme.color_code(),
            text,
            AnsiColors::RESET
        )
    }

    /// Creates an error pointer line with colored arrows.
    pub fn format_error_pointer(&self, column: usize, length: usize) -> String {
        if !self.colors_enabled {
            let spaces = " ".repeat(column.saturating_sub(1));
            let arrows = "^".repeat(length.max(1));
            return format!("     | {}{}", spaces, arrows);
        }

        let spaces = " ".repeat(column.saturating_sub(1));
        let arrows = "^".repeat(length.max(1));
        let colored_arrows = self.colorize(&arrows, ColorScheme::Error);

        format!(
            "{}     | {}{}",
            self.colorize("", ColorScheme::LineNumbers),
            spaces,
            colored_arrows
        )
    }

    /// Formats a line number with appropriate styling.
    pub fn format_line_number(&self, line_num: usize) -> String {
        let line_str = format!("{:4}", line_num);
        if !self.colors_enabled {
            return format!("{} | ", line_str);
        }

        format!("{} | ", self.colorize(&line_str, ColorScheme::LineNumbers))
    }

    /// Formats an error code with appropriate styling.
    pub fn format_error_code(&self, code: &str) -> String {
        if !self.colors_enabled {
            return format!("[{}]", code);
        }

        let formatted_code = format!("[{}]", code);
        self.bold_colorize(&formatted_code, ColorScheme::ErrorCode)
    }

    /// Formats a suggestion with appropriate styling.
    pub fn format_suggestion(&self, index: usize, suggestion: &str) -> String {
        if !self.colors_enabled {
            return format!("  {}. {}", index, suggestion);
        }

        let number = format!("{}.", index);
        let colored_number = self.colorize(&number, ColorScheme::Info);
        let colored_suggestion = self.colorize(suggestion, ColorScheme::Context);

        format!("  {} {}", colored_number, colored_suggestion)
    }

    /// Returns whether colors are currently enabled.
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Enables or disables colors for this formatter.
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        if !self.force_colors {
            self.colors_enabled = enabled;
        }
    }
}

/// Utility functions for common terminal formatting operations.
pub struct TerminalUtils;

impl TerminalUtils {
    /// Creates a horizontal separator line.
    pub fn separator(formatter: &TerminalFormatter, width: usize) -> String {
        let line = "─".repeat(width);
        formatter.colorize(&line, ColorScheme::Secondary)
    }

    /// Creates a section header with appropriate styling.
    pub fn section_header(formatter: &TerminalFormatter, title: &str) -> String {
        formatter.bold_colorize(title, ColorScheme::Highlight)
    }

    /// Formats a key-value pair for display.
    pub fn key_value(formatter: &TerminalFormatter, key: &str, value: &str) -> String {
        let colored_key = formatter.colorize(key, ColorScheme::Info);
        let colored_value = formatter.colorize(value, ColorScheme::Context);
        format!("{}: {}", colored_key, colored_value)
    }

    /// Formats a file path with appropriate styling.
    pub fn file_path(formatter: &TerminalFormatter, path: &str) -> String {
        formatter.underline_colorize(path, ColorScheme::Info)
    }

    /// Formats a location string (line:column) with appropriate styling.
    pub fn location(formatter: &TerminalFormatter, location: &str) -> String {
        formatter.colorize(location, ColorScheme::Secondary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_codes() {
        assert_eq!(ColorScheme::Error.color_code(), AnsiColors::BRIGHT_RED);
        assert_eq!(ColorScheme::Warning.color_code(), AnsiColors::BRIGHT_YELLOW);
        assert_eq!(ColorScheme::Info.color_code(), AnsiColors::BRIGHT_BLUE);
        assert_eq!(ColorScheme::Success.color_code(), AnsiColors::BRIGHT_GREEN);
    }

    #[test]
    fn test_formatter_without_colors() {
        let formatter = TerminalFormatter::without_colors();
        assert!(!formatter.colors_enabled());

        let result = formatter.colorize("test", ColorScheme::Error);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_formatter_with_colors() {
        let formatter = TerminalFormatter::with_colors();
        assert!(formatter.colors_enabled());

        let result = formatter.colorize("test", ColorScheme::Error);
        assert!(result.contains("test"));
        assert!(result.contains(AnsiColors::BRIGHT_RED));
        assert!(result.contains(AnsiColors::RESET));
    }

    #[test]
    fn test_bold_colorize() {
        let formatter = TerminalFormatter::with_colors();
        let result = formatter.bold_colorize("test", ColorScheme::Error);

        assert!(result.contains("test"));
        assert!(result.contains(AnsiColors::BOLD));
        assert!(result.contains(AnsiColors::BRIGHT_RED));
        assert!(result.contains(AnsiColors::RESET));
    }

    #[test]
    fn test_format_error_code() {
        let formatter = TerminalFormatter::without_colors();
        let result = formatter.format_error_code("E1001");
        assert_eq!(result, "[E1001]");

        let formatter = TerminalFormatter::with_colors();
        let result = formatter.format_error_code("E1001");
        assert!(result.contains("[E1001]"));
    }

    #[test]
    fn test_format_line_number() {
        let formatter = TerminalFormatter::without_colors();
        let result = formatter.format_line_number(42);
        assert_eq!(result, "  42 | ");
    }

    #[test]
    fn test_format_suggestion() {
        let formatter = TerminalFormatter::without_colors();
        let result = formatter.format_suggestion(1, "Fix the error");
        assert_eq!(result, "  1. Fix the error");
    }
}
