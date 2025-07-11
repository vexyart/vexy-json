// this_file: crates/core/src/streaming/buffered/state.rs

/// Temporary state used during parsing of complex values.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub(super) struct TempParsingState {
    /// Buffer for accumulating string content
    pub(super) string_buffer: String,
    /// Whether we're inside a string literal
    pub(super) in_string: bool,
    /// Whether we're inside a comment
    pub(super) in_comment: bool,
    /// Comment type (single or multi-line)
    pub(super) comment_type: CommentType,
    /// Escape sequence state
    pub(super) escape_next: bool,
    /// Unicode escape accumulator
    pub(super) unicode_buffer: String,
    /// Bracket depth for object/array nesting
    pub(super) bracket_depth: i32,
}

/// Type of comment being parsed.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub(super) enum CommentType {
    None,
    SingleLine,
    MultiLine,
}

impl Default for CommentType {
    fn default() -> Self {
        CommentType::None
    }
}
