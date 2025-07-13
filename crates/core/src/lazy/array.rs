// this_file: crates/core/src/lazy/array.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result, Span};
use crate::lazy::{LazyParser, LazyValue};
use std::sync::Arc;

/// A lazy JSON array that parses elements on-demand.
#[derive(Debug, Clone)]
pub struct LazyArray {
    /// Vector of lazy values
    pub(super) elements: Vec<LazyValue>,
    /// The original input for deferred parsing
    #[allow(dead_code)]
    pub(super) input: Arc<str>,
    /// Parser options
    #[allow(dead_code)]
    pub(super) options: crate::parser::ParserOptions,
}

impl LazyArray {
    /// Creates a new lazy array.
    pub fn new(input: Arc<str>, options: crate::parser::ParserOptions) -> Self {
        LazyArray {
            elements: Vec::new(),
            input,
            options,
        }
    }

    /// Adds an element to the lazy array.
    pub fn push(&mut self, value: LazyValue) {
        self.elements.push(value);
    }

    /// Gets an element by index, evaluating it lazily if needed.
    pub fn get(&self, index: usize) -> Option<Result<Value>> {
        self.elements.get(index).map(|lazy_val| lazy_val.evaluate())
    }

    /// Gets an element without evaluating it.
    pub fn get_lazy(&self, index: usize) -> Option<&LazyValue> {
        self.elements.get(index)
    }

    /// Returns the length of the array.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Checks if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns an iterator over the lazy values.
    pub fn iter(&self) -> impl Iterator<Item = &LazyValue> {
        self.elements.iter()
    }

    /// Forces evaluation of all elements and returns a regular Value::Array.
    pub fn evaluate_all(&self) -> Result<Value> {
        let mut resolved_arr = Vec::new();
        for lazy_val in &self.elements {
            resolved_arr.push(lazy_val.evaluate()?);
        }
        Ok(Value::Array(resolved_arr))
    }
}

impl<'a> LazyParser<'a> {
    /// Parses an array, potentially deferring large arrays.
    pub(super) fn parse_array(&mut self, _start_span: Span) -> Result<Value> {
        // For now, always parse immediately
        // TODO: Implement proper lazy parsing with lexer checkpointing
        self.parse_array_immediate()
    }



    /// Parses an array immediately using standard parsing.
    pub(super) fn parse_array_immediate(&mut self) -> Result<Value> {
        let mut array = Vec::new();
        let mut first = true;

        loop {
            // Skip newlines
            loop {
                let (next_token, _) = self.peek_token()?;
                if next_token == Token::Newline {
                    self.next_token()?;
                } else {
                    break;
                }
            }

            // Check for end of array
            let (next_token, _) = self.peek_token()?;
            if next_token == Token::RightBracket {
                self.next_token()?;
                break;
            }

            // Handle comma between elements
            if !first {
                let (token, span) = self.next_token()?;
                match token {
                    Token::Comma => {}
                    Token::Newline if self.options.newline_as_comma => {}
                    Token::RightBracket if self.options.allow_trailing_commas => break,
                    _ => {
                        return Err(Error::Expected {
                            expected: "comma or ]".to_string(),
                            found: format!("{token:?}"),
                            position: span.start,
                        });
                    }
                }
            }
            first = false;

            // Parse value
            let (value_token, value_span) = self.next_token()?;
            let value = self.parse_value(value_token, value_span)?;
            array.push(value);
        }

        Ok(Value::Array(array))
    }
}
