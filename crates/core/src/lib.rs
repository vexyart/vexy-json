// this_file: src/lib.rs
#![warn(
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

//! Core parsing logic for vexy_json, a forgiving JSON parser.

/// Abstract Syntax Tree components for vexy_json parsing.
pub mod ast;

/// Error types and result type alias for the vexy_json parser.
pub mod error;

/// Lexical analyzer (tokenizer) for vexy_json parsing.
pub mod lexer;

/// Parser implementation for converting tokens to JSON values.
pub mod parser;

/// Streaming parser implementation for incremental parsing.
pub mod streaming;

/// Performance optimization modules.
pub mod optimization;

/// Lazy evaluation for large JSON structures.
pub mod lazy;

/// Plugin system for extending parser functionality.
pub mod plugin;

/// JSON repair functionality for bracket balancing.
pub mod repair;

/// JSON transformation utilities.
pub mod transform;

/// Parallel parsing for large files.
pub mod parallel;

/// Parallel chunked processing for large JSON files.
pub mod parallel_chunked;

#[cfg(feature = "serde")]
/// WebAssembly bindings for browser usage.
#[cfg(feature = "wasm")]
pub use ast::{Number, Token, Value};
pub use error::{EnhancedParseResult, ParsingTier, RepairAction, RepairType};
pub use error::{Error, ParseResult, Result};
pub use lazy::{
    parse_lazy, parse_lazy_with_options, parse_lazy_with_threshold, LazyArray, LazyObject,
    LazyParser, LazyValue,
};
pub use lexer::Lexer;
pub use parallel::{parse_ndjson_parallel, parse_parallel, ParallelConfig, ParallelParser};
pub use parser::{
    parse, parse_iterative, parse_optimized, parse_optimized_v2, parse_optimized_v2_with_options,
    parse_optimized_v3, parse_optimized_v3_with_options, parse_optimized_with_options, 
    parse_recursive, parse_v2_with_stats, parse_v3_with_stats,
    parse_with_detailed_repair_tracking, parse_with_fallback, parse_with_options, parse_with_stats,
    IterativeParser, Parser, ParserOptions, RecursiveDescentParser,
};
pub use repair::JsonRepairer;
pub use streaming::{
    parse_streaming, parse_streaming_with_config, BufferedStreamingConfig, BufferedStreamingParser,
    NdJsonParser, SimpleStreamingLexer, StreamingEvent, StreamingParser, StreamingValueBuilder,
};
pub use transform::{
    normalize, normalize_with_options, optimize, optimize_with_options, AstOptimizer,
    CanonicalNormalizer, CleanupNormalizer, InternerStats, JsonNormalizer, MemoryOptimizer,
    NormalizerOptions, OptimizerOptions, OptimizerStats, PerformanceOptimizer, StringInterner,
};
