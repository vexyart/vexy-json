// this_file: src/error/mod.rs

//! Error handling module for vexy_json parser.
//!
//! This module provides comprehensive error handling capabilities including:
//! - Structured error codes with unique identifiers
//! - Context-aware error messages with actionable suggestions
//! - Enhanced span system with line/column information
//! - Intelligent error recovery strategies
//! - Diagnostic utilities for better error reporting

/// Error recovery analysis and strategy recommendation.
pub mod recovery;
/// Repair functionality for JSON error recovery.
pub mod repair;
/// Comprehensive error reporting with configurable formatting.
pub mod reporter;
/// Result type alias for convenience.
pub mod result;
/// Span for error reporting with enhanced line/column tracking.
pub mod span;
/// Terminal formatting and colored output for error reporting.
pub mod terminal;
/// Error type definitions and implementations with structured error codes.
pub mod types;

/// Utility traits and helper functions for error analysis.
pub mod utils;

/// Smart error recovery engine V2 with ML-based pattern recognition.
pub mod recovery_v2;

/// ML-based pattern recognition for error recovery.
pub mod ml_patterns;

// Re-export public API for backward compatibility
pub use result::{ParseResult, Result};
pub use span::{ContextWindow, EnhancedSpan, LineCol, Span};
pub use types::{Error, ErrorCode};
pub use utils::{ErrorHelper, ErrorSeverity, ErrorUtils};

// Re-export repair functionality
pub use repair::{EnhancedParseResult, ParsingTier, RepairAction, RepairType};

// Re-export recovery functionality
pub use recovery::{ErrorRecoveryAnalyzer, RecoveryStrategy};

// Re-export terminal formatting functionality
pub use terminal::{ColorScheme, TerminalFormatter};

// Re-export error reporting functionality
pub use reporter::{
    full_error_report, plain_error_report, quick_error_report, ErrorReporter, ReportConfig,
};
