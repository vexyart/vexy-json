// this_file: src/ast/mod.rs

//! Abstract Syntax Tree components for vexy_json parsing.
//!
//! This module contains the core AST types used throughout the vexy_json parser:
//! - `Token`: Lexical tokens produced by the lexer
//! - `Value`: Parsed JSON values with support for all vexy_json extensions
//! - `Number`: Numeric value representation supporting integers and floats
//!
//! These types form the foundation of the parsing pipeline, from lexical analysis
//! through to final value construction.

pub mod builder;
pub mod token;
pub mod value;
pub mod visitor;

// Re-export all public types for convenient access
pub use builder::{ArrayBuilder, ObjectBuilder, ValueBuilder};
pub use token::Token;
pub use value::{Number, Value};
pub use visitor::{walk, walk_mut, walk_with_path, JsonPath, MutVisitor, PathVisitor, Visitor};
