// this_file: crates/core/src/streaming/buffered/buffer.rs

// use crate::ast::Token;
use crate::error::{Error, Result};
use crate::streaming::buffered::BufferedStreamingParser;
use std::io::{Read};

impl<R: Read> BufferedStreamingParser<R> {
    /// Fills the input and token buffers by reading more data.
    pub(super) fn fill_buffers(&mut self) -> Result<()> {
        if self.end_of_input {
            return Ok(());
        }

        // Read more data from the reader
        let mut buffer = vec![0u8; 1024];
        match self.reader.read(&mut buffer) {
            Ok(0) => {
                // End of input reached
                self.end_of_input = true;
                // Flush any remaining tokens from the lexer
                let tokens = self.lexer.flush()?;
                self.token_buffer.extend(tokens);
            }
            Ok(n) => {
                // Convert bytes to string (handle UTF-8 properly)
                let input_str = std::str::from_utf8(&buffer[..n])
                    .map_err(|_| Error::InvalidUtf8(self.lexer.position()))?;
                
                // Accumulate input for token content extraction
                self.input_accumulator.push_str(input_str);
                
                // Feed to lexer
                let (tokens, _needs_more) = self.lexer.feed(input_str)?;
                self.token_buffer.extend(tokens);
            }
            Err(e) => {
                // Properly convert IO error
                return Err(Error::Custom(format!("IO error: {e}")));
            }
        }

        Ok(())
    }
}
