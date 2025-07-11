// this_file: crates/core/src/streaming/buffered/buffer.rs

// use crate::ast::Token;
use crate::error::{Error, Result};
use crate::streaming::buffered::BufferedStreamingParser;
use std::io::{BufRead, Read};

impl<R: Read> BufferedStreamingParser<R> {
    /// Fills the input and token buffers by reading more data.
    pub(super) fn fill_buffers(&mut self) -> Result<()> {
        if self.end_of_input {
            return Ok(());
        }

        // Read more data into the input buffer
        let mut temp_buffer = String::new();
        match self.reader.read_line(&mut temp_buffer) {
            Ok(0) => {
                // End of input reached
                self.end_of_input = true;
                self.process_remaining_input()?;
            }
            Ok(_) => {
                self.input_buffer.push_str(&temp_buffer);
                self.tokenize_buffer()?;
            }
            Err(_e) => return Err(Error::InvalidNumber(self.position)), // Convert IO error
        }

        Ok(())
    }

    /// Tokenizes the current input buffer.
    fn tokenize_buffer(&mut self) -> Result<()> {
        // Use a simple approach: split on basic delimiters and process each part
        let buffer_content = self.input_buffer.clone();
        self.input_buffer.clear();

        // For simplicity, just split on whitespace and structural characters
        // In a production implementation, this would be more sophisticated
        let mut current_token = String::new();

        for ch in buffer_content.chars() {
            match ch {
                ' ' | '\t' | '\r' => {
                    if !current_token.is_empty() {
                        self.add_token_if_valid(&current_token)?;
                        current_token.clear();
                    }
                }
                '{' | '}' | '[' | ']' | ',' | ':' | '\n' => {
                    if !current_token.is_empty() {
                        self.add_token_if_valid(&current_token)?;
                        current_token.clear();
                    }
                    // Add the structural character as a token
                    self.add_token_if_valid(&ch.to_string())?;
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        // Add any remaining token
        if !current_token.is_empty() {
            self.add_token_if_valid(&current_token)?;
        }

        Ok(())
    }

    /// Adds a token to the buffer if it's valid and there's space.
    fn add_token_if_valid(&mut self, token_str: &str) -> Result<()> {
        if self.token_buffer.len() < self.config.token_buffer_size {
            let token = self.classify_token(token_str)?;
            self.token_buffer.push_back((token, token_str.to_string()));
        }
        Ok(())
    }

    /// Processes any remaining input when end of stream is reached.
    pub(super) fn process_remaining_input(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            self.tokenize_buffer()?;
        }
        Ok(())
    }
}
