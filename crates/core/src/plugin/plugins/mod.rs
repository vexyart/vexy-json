//! Built-in plugins for Vexy JSON parser

mod comment_preservation;
mod custom_number;
mod datetime;
mod schema_validation;

pub use comment_preservation::CommentPreservationPlugin;
pub use custom_number::CustomNumberFormatPlugin;
pub use datetime::DateTimePlugin;
pub use schema_validation::SchemaValidationPlugin;
