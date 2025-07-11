// this_file: src/ast/token.rs
#![allow(missing_docs)]

//! Token types and definitions for the vexy_json lexer.
//!
//! This module defines all the token types that can be produced during
//! lexical analysis of vexy_json input. The tokens support both standard JSON
//! syntax and vexy_json's forgiving extensions like comments and unquoted strings.

/// Represents a token in the vexy_json language.
///
/// This enum is used by the lexer to break down the input string into meaningful units.
use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Logos)]
pub enum Token {
    /// Opening curly brace '{' for objects.
    #[token("{")]
    LeftBrace,
    /// Closing curly brace '}' for objects.
    #[token("}")]
    RightBrace,
    /// Opening square bracket '[' for arrays.
    #[token("[")]
    LeftBracket,
    /// Closing square bracket ']' for arrays.
    #[token("]")]
    RightBracket,
    /// Comma ',' separator.
    #[token(",")]
    Comma,
    /// Colon ':' separator between keys and values.
    #[token(":")]
    Colon,
    /// JSON null literal.
    #[token("null")]
    Null,
    /// JSON true literal.
    #[token("true")]
    True,
    /// JSON false literal.
    #[token("false")]
    False,

    /// Newline character '\n' or '\r' (used for newline-as-comma feature).
    #[token("\n")]
    #[token("\r")]
    Newline,

    /// Whitespace (spaces, tabs) - skipped during parsing.
    #[regex(r"[ \t]+", logos::skip)]

    /// String literal (double or single quoted).
    #[regex(r#""(?:[^"\\]|\\.)*""#)]
    #[regex(r#"'(?:[^'\\]|\\.)*'"#)]
    String,

    /// Unquoted string (used for object keys in forgiving mode). (Basic, will be refined)
    #[regex(r"[a-zA-Z_$][a-zA-Z0-9_$-]*")]
    UnquotedString,

    /// Numeric literal. (Basic, will be refined)
    #[regex(r"(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    #[regex(r"-(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    Number,

    /// Single-line comment starting with '//' or '#'.
    #[regex(r"//[^\r\n]*")]
    #[regex(r"#[^\r\n]*")]
    SingleLineComment,

    /// Multi-line comment enclosed in '/* */'.
    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    MultiLineComment,

    /// End of file/input. (Logos usually handles this implicitly)
    Eof,

    #[error]
    /// Represents a lexical error.
    Error, // Catch-all for lexing errors. Logos requires this.
}
