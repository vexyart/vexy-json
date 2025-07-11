// this_file: src/parser/state.rs

//! Parser state management for better error reporting and parsing control.
use crate::error::Span;

/// Parser state tracking for better error reporting and debugging.
#[derive(Debug, Clone, Default)]
pub struct ParserState {
    /// Current position in the input stream
    pub position: usize,
    /// Current parsing depth (for nested structures)
    pub depth: usize,
    /// The span of the current token
    pub span: Span,
}

impl ParserState {
    /// Creates a new parser state.
    pub fn new() -> Self {
        ParserState::default()
    }
}
