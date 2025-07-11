// this_file: crates/core/src/optimization/mod.rs

//! Performance optimization modules for vexy_json parsing.
//!
//! This module contains optimized implementations of core parsing
//! functionality to improve performance while maintaining compatibility.

pub mod benchmarks;
pub mod memory_pool;
pub mod memory_pool_v2;
pub mod memory_pool_v3;
pub mod simd;
pub mod string_parser;
pub mod value_builder;
pub mod zero_copy;

pub use benchmarks::{BenchmarkResult, PerformanceMonitor};
pub use memory_pool::{MemoryPool, MemoryPoolStats, ScopedMemoryPool};
pub use memory_pool_v2::{OptimizedMemoryPool, PoolStats, ScopedOptimizedPool};
pub use memory_pool_v3::{
    with_pool, AllocationStats, CompactString, MemoryPoolV3, ScopedMemoryPoolV3, SmallVec,
    TypedArena,
};
pub use simd::{
    has_backslash_simd, parse_number_simd, skip_whitespace_simd, unescape_string_simd,
    validate_json_string_simd,
};
pub use string_parser::{
    extract_string_content, parse_number_optimized, unescape_string_optimized,
};
pub use value_builder::ValueBuilder;
pub use zero_copy::{parse_number_fast, parse_string_zero_copy};
