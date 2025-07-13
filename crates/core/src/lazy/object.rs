// this_file: crates/core/src/lazy/object.rs

use crate::ast::{Token, Value};
use crate::error::{Error, Result, Span};
use crate::lazy::{LazyParser, LazyValue};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// A lazy JSON object that parses keys and values on-demand.
#[derive(Debug, Clone)]
pub struct LazyObject {
    /// Map of field names to their lazy values
    pub(super) fields: FxHashMap<String, LazyValue>,
    /// The original input for deferred parsing
    #[allow(dead_code)]
    pub(super) input: Arc<str>,
    /// Parser options
    #[allow(dead_code)]
    pub(super) options: crate::parser::ParserOptions,
}

impl LazyObject {
    /// Creates a new lazy object.
    pub fn new(input: Arc<str>, options: crate::parser::ParserOptions) -> Self {
        LazyObject {
            fields: FxHashMap::default(),
            input,
            options,
        }
    }

    /// Adds a field to the lazy object.
    pub fn insert(&mut self, key: String, value: LazyValue) {
        self.fields.insert(key, value);
    }

    /// Gets a field by key, evaluating it lazily if needed.
    pub fn get(&self, key: &str) -> Option<Result<Value>> {
        self.fields.get(key).map(|lazy_val| lazy_val.evaluate())
    }

    /// Gets a field without evaluating it.
    pub fn get_lazy(&self, key: &str) -> Option<&LazyValue> {
        self.fields.get(key)
    }

    /// Lists all available keys without evaluating values.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }

    /// Checks if a key exists without evaluating the value.
    pub fn contains_key(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Forces evaluation of all fields and returns a regular Value::Object.
    pub fn evaluate_all(&self) -> Result<Value> {
        let mut resolved_obj = FxHashMap::default();
        for (key, lazy_val) in &self.fields {
            resolved_obj.insert(key.clone(), lazy_val.evaluate()?);
        }
        Ok(Value::Object(resolved_obj))
    }
}

impl<'a> LazyParser<'a> {
    /// Parses an object, potentially deferring large objects.
    pub(super) fn parse_object(&mut self, _start_span: Span) -> Result<Value> {
        // For now, always parse immediately
        // TODO: Implement proper lazy parsing with lexer checkpointing
        self.parse_object_immediate()
    }



    /// Parses an object immediately using standard parsing.
    pub(super) fn parse_object_immediate(&mut self) -> Result<Value> {
        let mut object = FxHashMap::default();
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

            // Check for end of object
            let (next_token, _) = self.peek_token()?;
            if next_token == Token::RightBrace {
                self.next_token()?;
                break;
            }

            // Handle comma between elements
            if !first {
                let (token, span) = self.next_token()?;
                match token {
                    Token::Comma => {}
                    Token::Newline if self.options.newline_as_comma => {}
                    Token::RightBrace if self.options.allow_trailing_commas => break,
                    _ => {
                        return Err(Error::Expected {
                            expected: "comma or }".to_string(),
                            found: format!("{token:?}"),
                            position: span.start,
                        });
                    }
                }
            }
            first = false;

            // Parse key
            let (key_token, key_span) = self.next_token()?;
            let key = match key_token {
                Token::String => self.parse_string_content(key_span)?,
                Token::UnquotedString if self.options.allow_unquoted_keys => {
                    self.input[key_span.start..key_span.end].to_string()
                }
                _ => {
                    return Err(Error::Expected {
                        expected: "string key".to_string(),
                        found: format!("{key_token:?}"),
                        position: key_span.start,
                    });
                }
            };

            // Expect colon
            let (colon_token, colon_span) = self.next_token()?;
            match colon_token {
                Token::Colon => {}
                _ => {
                    return Err(Error::Expected {
                        expected: "colon".to_string(),
                        found: format!("{colon_token:?}"),
                        position: colon_span.start,
                    });
                }
            }

            // Parse value
            let (value_token, value_span) = self.next_token()?;
            let value = self.parse_value(value_token, value_span)?;

            object.insert(key, value);
        }

        Ok(Value::Object(object))
    }
}
