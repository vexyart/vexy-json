// this_file: src/error/span.rs

//! Span types for precise error location reporting with line/column information.

use crate::error::terminal::{ColorScheme, TerminalFormatter};

/// Represents a span of text in the input for error reporting.
///
/// Spans provide precise location information including both byte positions
/// and line/column coordinates for better error reporting and debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start position in the input (byte offset)
    pub start: usize,
    /// End position in the input (byte offset)
    pub end: usize,
}

/// Line and column position information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineCol {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// Enhanced span with line/column information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnhancedSpan {
    /// Byte span in the input
    pub span: Span,
    /// Line/column of the start position
    pub start_pos: LineCol,
    /// Line/column of the end position
    pub end_pos: LineCol,
}

/// Context window showing lines around an error location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextWindow {
    /// Lines of context around the error
    pub lines: Vec<String>,
    /// Index of the line containing the error (0-based within context)
    pub error_line: usize,
    /// Column where the error starts (1-based)
    pub error_column: usize,
    /// Length of the error span
    pub error_length: usize,
}

impl Span {
    /// Creates a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Creates a span for a single position.
    pub fn single(position: usize) -> Self {
        Span {
            start: position,
            end: position + 1,
        }
    }

    /// Returns the length of the span.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Checks if the span is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Checks if this span contains the given position.
    pub fn contains(&self, position: usize) -> bool {
        position >= self.start && position < self.end
    }

    /// Merges two spans into a single span covering both.
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Converts this span to an enhanced span with line/column information.
    pub fn to_enhanced(&self, input: &str) -> EnhancedSpan {
        let start_pos = byte_to_line_col(input, self.start);
        let end_pos = byte_to_line_col(input, self.end);

        EnhancedSpan {
            span: *self,
            start_pos,
            end_pos,
        }
    }

    /// Extracts the text covered by this span from the input.
    pub fn extract<'a>(&self, input: &'a str) -> &'a str {
        let start = self.start.min(input.len());
        let end = self.end.min(input.len());
        &input[start..end]
    }

    /// Returns a context window around this span.
    pub fn context_window(&self, input: &str, context_size: usize) -> ContextWindow {
        let start_line = byte_to_line_col(input, self.start);
        let end_line = byte_to_line_col(input, self.end);

        // Get lines around the error
        let lines: Vec<&str> = input.lines().collect();
        let context_start = start_line.line.saturating_sub(context_size + 1);
        let context_end = (end_line.line + context_size).min(lines.len());

        let context_lines: Vec<String> = lines[context_start..context_end]
            .iter()
            .map(|s| s.to_string())
            .collect();

        ContextWindow {
            lines: context_lines,
            error_line: start_line.line - context_start - 1,
            error_column: start_line.column,
            error_length: self.len(),
        }
    }
}

impl LineCol {
    /// Creates a new line/column position.
    pub fn new(line: usize, column: usize) -> Self {
        LineCol { line, column }
    }

    /// Creates a line/column position at the start of input.
    pub fn start() -> Self {
        LineCol { line: 1, column: 1 }
    }
}

impl EnhancedSpan {
    /// Creates a new enhanced span.
    pub fn new(span: Span, start_pos: LineCol, end_pos: LineCol) -> Self {
        EnhancedSpan {
            span,
            start_pos,
            end_pos,
        }
    }

    /// Returns a formatted location string for display.
    pub fn location_string(&self) -> String {
        if self.start_pos.line == self.end_pos.line {
            format!("{}:{}", self.start_pos.line, self.start_pos.column)
        } else {
            format!(
                "{}:{}-{}:{}",
                self.start_pos.line, self.start_pos.column, self.end_pos.line, self.end_pos.column
            )
        }
    }
}

impl ContextWindow {
    /// Formats the context window with syntax highlighting for the error.
    ///
    /// This method provides basic highlighting using simple markers for compatibility.
    /// For colored output, use `format_with_formatter` or `format_colored`.
    pub fn format_with_highlight(&self) -> String {
        let mut result = String::new();

        for (i, line) in self.lines.iter().enumerate() {
            // Add line number
            result.push_str(&format!("{:4} | ", i + 1));

            if i == self.error_line {
                // This is the error line - add highlighting
                let before = &line[..self.error_column.saturating_sub(1)];
                let error_start = self.error_column.saturating_sub(1);
                let error_end = (error_start + self.error_length).min(line.len());
                let error_text = &line[error_start..error_end];
                let after = &line[error_end..];

                result.push_str(before);
                result.push_str(&format!(">>{error_text}<<")); // Simple highlighting
                result.push_str(after);
                result.push('\n');

                // Add error pointer line
                result.push_str("     | ");
                result.push_str(&" ".repeat(self.error_column.saturating_sub(1)));
                result.push_str(&"^".repeat(self.error_length.max(1)));
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Formats the context window using the provided terminal formatter.
    ///
    /// This method provides colored highlighting when the formatter supports colors,
    /// and falls back to plain text when colors are not available.
    pub fn format_with_formatter(&self, formatter: &TerminalFormatter) -> String {
        let mut result = String::new();

        for (i, line) in self.lines.iter().enumerate() {
            let line_num = i + 1;

            if i == self.error_line {
                // This is the error line - add colored highlighting
                let before = &line[..self.error_column.saturating_sub(1)];
                let error_start = self.error_column.saturating_sub(1);
                let error_end = (error_start + self.error_length).min(line.len());
                let error_text = &line[error_start..error_end];
                let after = &line[error_end..];

                // Format line number with error style
                result.push_str(&formatter.format_line_number(line_num));

                // Format the line with error highlighting
                result.push_str(before);
                result.push_str(&formatter.colorize(error_text, ColorScheme::Error));
                result.push_str(after);
                result.push('\n');

                // Add error pointer line with colored arrows
                result.push_str(
                    &formatter.format_error_pointer(self.error_column, self.error_length.max(1)),
                );
                result.push('\n');
            } else {
                // Regular context line
                result.push_str(&formatter.format_line_number(line_num));
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Formats the context window with colored output if terminal supports it.
    ///
    /// This is a convenience method that automatically detects terminal capabilities
    /// and uses colored output when available.
    pub fn format_colored(&self) -> String {
        let formatter = TerminalFormatter::new();
        self.format_with_formatter(&formatter)
    }

    /// Formats the context window with plain text output (no colors).
    ///
    /// This is a convenience method for when you specifically want plain text output
    /// regardless of terminal capabilities.
    pub fn format_plain(&self) -> String {
        let formatter = TerminalFormatter::without_colors();
        self.format_with_formatter(&formatter)
    }
}

/// Converts a byte position to line/column coordinates.
fn byte_to_line_col(input: &str, byte_pos: usize) -> LineCol {
    let mut line = 1;
    let mut column = 1;
    let mut current_pos = 0;

    for ch in input.chars() {
        if current_pos >= byte_pos {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }

        current_pos += ch.len_utf8();
    }

    LineCol { line, column }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(5, 10);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 10);
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_single() {
        let span = Span::single(5);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 6);
        assert_eq!(span.len(), 1);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(5, 10);
        assert!(span.contains(5));
        assert!(span.contains(7));
        assert!(!span.contains(10));
        assert!(!span.contains(3));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(8, 15);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_byte_to_line_col() {
        let input = "hello\nworld\nfoo";
        assert_eq!(byte_to_line_col(input, 0), LineCol::new(1, 1));
        assert_eq!(byte_to_line_col(input, 5), LineCol::new(1, 6)); // '\n' position
        assert_eq!(byte_to_line_col(input, 6), LineCol::new(2, 1)); // 'w' position
        assert_eq!(byte_to_line_col(input, 12), LineCol::new(3, 1)); // 'f' position
    }

    #[test]
    fn test_enhanced_span() {
        let input = "hello\nworld\nfoo";
        let span = Span::new(6, 11); // "world"
        let enhanced = span.to_enhanced(input);

        assert_eq!(enhanced.start_pos, LineCol::new(2, 1));
        assert_eq!(enhanced.end_pos, LineCol::new(2, 6));
        assert_eq!(enhanced.location_string(), "2:1");
    }

    #[test]
    fn test_span_extract() {
        let input = "hello world";
        let span = Span::new(6, 11); // "world"
        assert_eq!(span.extract(input), "world");
    }
}
