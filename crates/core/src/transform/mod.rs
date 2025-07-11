// this_file: crates/core/src/transform/mod.rs

//! JSON transformation utilities.
//!
//! This module provides utilities for transforming JSON data:
//! - Normalizing JSON into canonical forms
//! - Cleaning up JSON data
//! - Comparing JSON values

pub mod normalizer;
pub mod optimizer;

pub use normalizer::{
    normalize, normalize_with_options, CanonicalNormalizer, CleanupNormalizer, JsonNormalizer,
    NormalizerOptions,
};
pub use optimizer::{
    optimize, optimize_with_options, AstOptimizer, MemoryOptimizer, OptimizerOptions,
    OptimizerStats, PerformanceOptimizer, StringInterner, InternerStats,
};