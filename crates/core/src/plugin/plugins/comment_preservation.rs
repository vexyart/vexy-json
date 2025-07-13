//! Comment preservation plugin

use crate::ast::Value;
use crate::error::Result;
use crate::plugin::ParserPlugin;
use rustc_hash::FxHashMap;
use std::any::Any;

/// Comment information
#[derive(Debug, Clone)]
pub struct Comment {
    /// Comment text (without delimiters)
    pub text: String,
    /// Line number where comment appears
    pub line: usize,
    /// Column number where comment starts
    pub column: usize,
    /// Whether it's a single-line (//) or multi-line (/* */) comment
    pub is_multiline: bool,
}

/// Comment preservation plugin that stores comments during parsing
pub struct CommentPreservationPlugin {
    /// Comments mapped by their location
    comments: FxHashMap<String, Vec<Comment>>,
    /// All comments in order
    all_comments: Vec<Comment>,
    /// Current line number
    current_line: usize,
    /// Current column number
    current_column: usize,
}

impl CommentPreservationPlugin {
    /// Create a new comment preservation plugin
    pub fn new() -> Self {
        CommentPreservationPlugin {
            comments: FxHashMap::default(),
            all_comments: Vec::new(),
            current_line: 1,
            current_column: 1,
        }
    }

    /// Add a comment
    pub fn add_comment(&mut self, text: String, path: &str, is_multiline: bool) {
        let comment = Comment {
            text,
            line: self.current_line,
            column: self.current_column,
            is_multiline,
        };

        self.all_comments.push(comment.clone());
        self.comments
            .entry(path.to_string())
            .or_default()
            .push(comment);
    }

    /// Get all comments
    pub fn all_comments(&self) -> &[Comment] {
        &self.all_comments
    }

    /// Get comments for a specific path
    pub fn comments_at(&self, path: &str) -> Option<&[Comment]> {
        self.comments.get(path).map(|v| v.as_slice())
    }

    /// Update position tracking
    pub fn update_position(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            } else {
                self.current_column += 1;
            }
        }
    }

    /// Convert comments to a value that can be attached to the AST
    pub fn comments_to_value(&self) -> Value {
        let mut result = Vec::new();

        for comment in &self.all_comments {
            let mut obj = FxHashMap::default();
            obj.insert("text".to_string(), Value::String(comment.text.clone()));
            obj.insert(
                "line".to_string(),
                Value::Number(crate::ast::Number::Integer(comment.line as i64)),
            );
            obj.insert(
                "column".to_string(),
                Value::Number(crate::ast::Number::Integer(comment.column as i64)),
            );
            obj.insert("multiline".to_string(), Value::Bool(comment.is_multiline));
            result.push(Value::Object(obj));
        }

        Value::Array(result)
    }
}

impl Default for CommentPreservationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserPlugin for CommentPreservationPlugin {
    fn name(&self) -> &str {
        "comment_preservation"
    }

    fn on_parse_end(&mut self, _value: &Value) -> Result<()> {
        // Optionally, we could attach comments to the parsed value here
        // For now, comments are available through the plugin's methods
        Ok(())
    }

    fn transform_value(&mut self, value: &mut Value, path: &str) -> Result<()> {
        // If there are comments for this path, we could attach them as metadata
        // For now, we just track them separately
        match value {
            Value::Object(obj) => {
                // Add a special _comments field if there are comments
                if let Some(comments) = self.comments_at(path) {
                    if !comments.is_empty() && !obj.contains_key("_comments") {
                        let comment_values: Vec<Value> = comments
                            .iter()
                            .map(|c| Value::String(c.text.clone()))
                            .collect();
                        obj.insert("_comments".to_string(), Value::Array(comment_values));
                    }
                }

                // Recurse into object values
                for (key, val) in obj.iter_mut() {
                    let child_path = format!("{path}.{key}");
                    self.transform_value(val, &child_path)?;
                }
            }
            Value::Array(arr) => {
                // Recurse into array elements
                for (i, val) in arr.iter_mut().enumerate() {
                    let child_path = format!("{path}[{i}]");
                    self.transform_value(val, &child_path)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_storage() {
        let mut plugin = CommentPreservationPlugin::new();

        plugin.add_comment("This is a test comment".to_string(), "$.foo", false);
        plugin.add_comment("Another comment".to_string(), "$.bar", true);

        assert_eq!(plugin.all_comments().len(), 2);
        assert_eq!(plugin.comments_at("$.foo").unwrap().len(), 1);
        assert_eq!(
            plugin.comments_at("$.foo").unwrap()[0].text,
            "This is a test comment"
        );
    }

    #[test]
    fn test_position_tracking() {
        let mut plugin = CommentPreservationPlugin::new();

        plugin.update_position("hello\nworld");
        assert_eq!(plugin.current_line, 2);
        assert_eq!(plugin.current_column, 6); // "world" is 5 chars + 1

        plugin.add_comment("test".to_string(), "$", false);
        let comment = &plugin.all_comments()[0];
        assert_eq!(comment.line, 2);
        assert_eq!(comment.column, 6);
    }

    #[test]
    fn test_comments_to_value() {
        let mut plugin = CommentPreservationPlugin::new();

        plugin.add_comment("Comment 1".to_string(), "$", false);
        plugin.update_position("\n");
        plugin.add_comment("Comment 2".to_string(), "$", true);

        let value = plugin.comments_to_value();
        if let Value::Array(arr) = value {
            assert_eq!(arr.len(), 2);

            if let Value::Object(obj) = &arr[0] {
                assert_eq!(
                    obj.get("text"),
                    Some(&Value::String("Comment 1".to_string()))
                );
                assert_eq!(obj.get("multiline"), Some(&Value::Bool(false)));
            }
        } else {
            panic!("Expected array");
        }
    }
}
