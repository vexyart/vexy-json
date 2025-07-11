// this_file: crates/core/src/optimization/benchmarks.rs

//! Benchmarking utilities for SIMD-accelerated string parsing optimizations.
//!
//! This module provides comprehensive benchmarking capabilities to measure
//! and compare the performance of SIMD vs scalar implementations across
//! various string processing operations.

use crate::optimization::simd::*;
use std::time::{Duration, Instant};

/// Performance monitoring and benchmarking utilities for SIMD optimizations.
pub struct PerformanceMonitor {
    /// Collection of SIMD operation execution times
    pub simd_times: Vec<Duration>,
    /// Collection of scalar operation execution times
    pub scalar_times: Vec<Duration>,
    /// Test data strings used for benchmarking
    pub test_data: Vec<String>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default test data.
    pub fn new() -> Self {
        Self {
            simd_times: Vec::new(),
            scalar_times: Vec::new(),
            test_data: Self::generate_test_data(),
        }
    }

    /// Generate comprehensive test data for benchmarking.
    fn generate_test_data() -> Vec<String> {
        vec![
            // Small strings
            "hello".to_string(),
            "world test".to_string(),
            "short\\nstring".to_string(),
            
            // Medium strings
            "This is a medium-length string for testing SIMD performance with some escape sequences \\n and \\t".to_string(),
            "Another medium string with numbers 123 and symbols !@#$%".to_string(),
            "Path\\\\to\\\\some\\\\file\\\\with\\\\backslashes".to_string(),
            
            // Large strings
            "a".repeat(1000),
            format!("{}\\n{}", "large string part 1 ".repeat(50), "large string part 2 ".repeat(50)),
            format!("{}\\\\{}", "path component ".repeat(100), "file.txt".repeat(10)),
            
            // Strings with many escape sequences
            "\\n\\t\\r\\\\\\\"\\'\\/\\b\\f".repeat(20),
            
            // JSON-like strings
            r#"{"name": "John", "age": 30, "city": "New York"}"#.to_string(),
            r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#.to_string(),
            
            // Whitespace-heavy strings
            "   \t\n\r   hello   \t\n\r   world   \t\n\r   ".to_string(),
            " ".repeat(100) + "content" + &" ".repeat(100),
            
            // Number strings
            "42".to_string(),
            "-123456".to_string(),
            "0".to_string(),
            "999999999999999999".to_string(),
        ]
    }

    /// Benchmark backslash detection performance.
    pub fn benchmark_backslash_detection(&mut self) -> BenchmarkResult {
        let mut simd_total = Duration::new(0, 0);
        let mut scalar_total = Duration::new(0, 0);
        let iterations = 1000;

        for test_string in &self.test_data {
            // Benchmark SIMD version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = has_backslash_simd(test_string);
            }
            simd_total += start.elapsed();

            // Benchmark scalar version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = test_string.contains('\\');
            }
            scalar_total += start.elapsed();
        }

        BenchmarkResult {
            operation: "Backslash Detection".to_string(),
            simd_time: simd_total,
            scalar_time: scalar_total,
            speedup: scalar_total.as_nanos() as f64 / simd_total.as_nanos() as f64,
            iterations,
        }
    }

    /// Benchmark string validation performance.
    pub fn benchmark_string_validation(&mut self) -> BenchmarkResult {
        let mut simd_total = Duration::new(0, 0);
        let mut scalar_total = Duration::new(0, 0);
        let iterations = 1000;

        for test_string in &self.test_data {
            // Benchmark SIMD version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = validate_json_string_simd(test_string);
            }
            simd_total += start.elapsed();

            // Benchmark scalar version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = validate_json_string_scalar(test_string);
            }
            scalar_total += start.elapsed();
        }

        BenchmarkResult {
            operation: "String Validation".to_string(),
            simd_time: simd_total,
            scalar_time: scalar_total,
            speedup: scalar_total.as_nanos() as f64 / simd_total.as_nanos() as f64,
            iterations,
        }
    }

    /// Benchmark whitespace skipping performance.
    pub fn benchmark_whitespace_skipping(&mut self) -> BenchmarkResult {
        let mut simd_total = Duration::new(0, 0);
        let mut scalar_total = Duration::new(0, 0);
        let iterations = 1000;

        for test_string in &self.test_data {
            // Benchmark SIMD version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = skip_whitespace_simd(test_string);
            }
            simd_total += start.elapsed();

            // Benchmark scalar version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = skip_whitespace_scalar(test_string);
            }
            scalar_total += start.elapsed();
        }

        BenchmarkResult {
            operation: "Whitespace Skipping".to_string(),
            simd_time: simd_total,
            scalar_time: scalar_total,
            speedup: scalar_total.as_nanos() as f64 / simd_total.as_nanos() as f64,
            iterations,
        }
    }

    /// Benchmark string unescaping performance.
    pub fn benchmark_string_unescaping(&mut self) -> BenchmarkResult {
        let mut simd_total = Duration::new(0, 0);
        let mut scalar_total = Duration::new(0, 0);
        let iterations = 100; // Fewer iterations for complex operations

        for test_string in &self.test_data {
            // Benchmark SIMD version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = unescape_string_simd(test_string);
            }
            simd_total += start.elapsed();

            // Benchmark scalar version
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = unescape_string_scalar(test_string);
            }
            scalar_total += start.elapsed();
        }

        BenchmarkResult {
            operation: "String Unescaping".to_string(),
            simd_time: simd_total,
            scalar_time: scalar_total,
            speedup: scalar_total.as_nanos() as f64 / simd_total.as_nanos() as f64,
            iterations,
        }
    }

    /// Run all benchmarks and return comprehensive results.
    pub fn run_all_benchmarks(&mut self) -> Vec<BenchmarkResult> {
        vec![
            self.benchmark_backslash_detection(),
            self.benchmark_string_validation(),
            self.benchmark_whitespace_skipping(),
            self.benchmark_string_unescaping(),
        ]
    }
}

/// Results from a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmarked operation
    pub operation: String,
    /// Time taken by SIMD implementation
    pub simd_time: Duration,
    /// Time taken by scalar implementation
    pub scalar_time: Duration,
    /// Performance speedup ratio (scalar_time / simd_time)
    pub speedup: f64,
    /// Number of iterations performed in the benchmark
    pub iterations: u32,
}

impl BenchmarkResult {
    /// Display the benchmark results in a formatted way.
    pub fn display(&self) {
        println!("=== {} Benchmark ===", self.operation);
        println!("SIMD Time:   {:?}", self.simd_time);
        println!("Scalar Time: {:?}", self.scalar_time);
        println!("Speedup:     {:.2}x", self.speedup);
        println!("Iterations:  {}", self.iterations);
        println!();
    }
}

// Scalar implementations for comparison
fn validate_json_string_scalar(s: &str) -> bool {
    for byte in s.bytes() {
        if byte < 0x20 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
            return false;
        }
    }
    true
}

fn skip_whitespace_scalar(s: &str) -> usize {
    let mut i = 0;
    for byte in s.bytes() {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            _ => break,
        }
    }
    i
}

fn unescape_string_scalar(s: &str) -> Result<String, crate::error::Error> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                match escaped {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0C'),
                    'u' => {
                        // Unicode escape sequence
                        let hex_chars: String = chars.by_ref().take(4).collect();
                        if hex_chars.len() == 4 {
                            if let Ok(code) = u32::from_str_radix(&hex_chars, 16) {
                                if let Some(unicode_char) = std::char::from_u32(code) {
                                    result.push(unicode_char);
                                    continue;
                                }
                            }
                        }
                        return Err(crate::error::Error::InvalidEscape(0));
                    }
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(!monitor.test_data.is_empty());
        assert!(monitor.simd_times.is_empty());
        assert!(monitor.scalar_times.is_empty());
    }

    #[test]
    fn test_benchmark_backslash_detection() {
        let mut monitor = PerformanceMonitor::new();
        let result = monitor.benchmark_backslash_detection();

        assert_eq!(result.operation, "Backslash Detection");
        assert!(result.simd_time.as_nanos() > 0);
        assert!(result.scalar_time.as_nanos() > 0);
        assert!(result.speedup > 0.0);
        assert_eq!(result.iterations, 1000);
    }

    #[test]
    fn test_benchmark_string_validation() {
        let mut monitor = PerformanceMonitor::new();
        let result = monitor.benchmark_string_validation();

        assert_eq!(result.operation, "String Validation");
        assert!(result.simd_time.as_nanos() > 0);
        assert!(result.scalar_time.as_nanos() > 0);
        assert!(result.speedup > 0.0);
        assert_eq!(result.iterations, 1000);
    }

    #[test]
    fn test_benchmark_whitespace_skipping() {
        let mut monitor = PerformanceMonitor::new();
        let result = monitor.benchmark_whitespace_skipping();

        assert_eq!(result.operation, "Whitespace Skipping");
        assert!(result.simd_time.as_nanos() > 0);
        assert!(result.scalar_time.as_nanos() > 0);
        assert!(result.speedup > 0.0);
        assert_eq!(result.iterations, 1000);
    }

    #[test]
    fn test_benchmark_string_unescaping() {
        let mut monitor = PerformanceMonitor::new();
        let result = monitor.benchmark_string_unescaping();

        assert_eq!(result.operation, "String Unescaping");
        assert!(result.simd_time.as_nanos() > 0);
        assert!(result.scalar_time.as_nanos() > 0);
        assert!(result.speedup > 0.0);
        assert_eq!(result.iterations, 100);
    }

    #[test]
    fn test_run_all_benchmarks() {
        let mut monitor = PerformanceMonitor::new();
        let results = monitor.run_all_benchmarks();

        assert_eq!(results.len(), 4);
        assert_eq!(results[0].operation, "Backslash Detection");
        assert_eq!(results[1].operation, "String Validation");
        assert_eq!(results[2].operation, "Whitespace Skipping");
        assert_eq!(results[3].operation, "String Unescaping");
    }

    #[test]
    fn test_scalar_implementations() {
        // Test scalar validation
        assert!(validate_json_string_scalar("hello world"));
        assert!(!validate_json_string_scalar("hello\x00world"));

        // Test scalar whitespace skipping
        assert_eq!(skip_whitespace_scalar("hello"), 0);
        assert_eq!(skip_whitespace_scalar("  hello"), 2);
        assert_eq!(skip_whitespace_scalar("\t\n\r hello"), 4);

        // Test scalar unescaping
        assert_eq!(unescape_string_scalar("hello").unwrap(), "hello");
        assert_eq!(
            unescape_string_scalar("hello\\nworld").unwrap(),
            "hello\nworld"
        );
        assert_eq!(
            unescape_string_scalar("path\\\\to\\\\file").unwrap(),
            "path\\to\\file"
        );
    }
}
