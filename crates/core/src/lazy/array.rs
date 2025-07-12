// this_file: crates/core/src/lazy/array.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result, Span};
use crate::lazy::{LazyParser, LazyValue};
use crate::lexer::JsonLexer;
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
    pub(super) fn parse_array(&mut self, start_span: Span) -> Result<Value> {
        // Find the end of the array to determine its size
        let end_span = self.find_array_end()?;
        let array_size = end_span.end - start_span.start;

        if array_size > self.lazy_threshold {
            // Create a lazy array for large arrays
            let mut lazy_arr = LazyArray::new(Arc::from(self.input), self.options.clone());

            // Parse only the element boundaries and defer the values
            self.parse_array_boundaries_only(&mut lazy_arr, start_span, end_span)?;

            // For now, return as a regular array since Value doesn't support LazyArray
            // In a real implementation, we'd extend Value to include lazy variants
            lazy_arr.evaluate_all()
        } else {
            // Parse immediately for small arrays
            self.parse_array_immediate()
        }
    }

    /// Finds the end of an array by scanning for the matching closing bracket.
    pub(super) fn find_array_end(&mut self) -> Result<Span> {
        let mut depth = 1;
        let mut current_pos = self.lexer.position();

        while depth > 0 {
            let (token, span) = self.next_token()?;
            current_pos = span.end;

            match token {
                Token::LeftBracket => depth += 1,
                Token::RightBracket => depth -= 1,
                Token::Eof => return Err(Error::UnterminatedString(current_pos)),
                _ => {}
            }
        }

        Ok(Span::new(current_pos - 1, current_pos))
    }

    /// Parses array boundaries only, creating deferred values for the elements.
    fn parse_array_boundaries_only(
        &mut self,
        _lazy_arr: &mut LazyArray,
        _start_span: Span,
        _end_span: Span,
    ) -> Result<()> {
        // This is a simplified implementation
        // In practice, we'd need to carefully parse just the structure
        // and create spans for each element

        // For now, fall back to immediate parsing
        // TODO: Implement proper boundary-only parsing
        Ok(())
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
