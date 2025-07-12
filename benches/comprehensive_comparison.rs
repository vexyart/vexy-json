// this_file: benches/comprehensive_comparison.rs

use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use std::fs;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    vexy_json_time: Option<Duration>,
    vexy_json_success: bool,
    vexy_json_error: Option<String>,
    ref_impl_time: Option<Duration>,
    ref_impl_success: bool,
    ref_impl_error: Option<String>,
    input_size: usize,
    input_content: String,
}

#[derive(Debug)]
struct BenchmarkSuite {
    results: Vec<BenchmarkResult>,
    timestamp: DateTime<Utc>,
    environment: FxHashMap<String, String>,
}

impl BenchmarkSuite {
    fn new() -> Self {
        let mut environment = FxHashMap::default();

        // Collect environment information
        environment.insert(
            "rust_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        environment.insert(
            "timestamp".to_string(),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        );

        // Get system info
        if let Ok(output) = Command::new("uname").arg("-a").output() {
            if let Ok(uname) = String::from_utf8(output.stdout) {
                environment.insert("system".to_string(), uname.trim().to_string());
            }
        }

        // Get CPU info (macOS)
        if let Ok(output) = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
        {
            if let Ok(cpu) = String::from_utf8(output.stdout) {
                environment.insert("cpu".to_string(), cpu.trim().to_string());
            }
        }

        Self {
            results: Vec::new(),
            timestamp: Utc::now(),
            environment,
        }
    }

    fn run_vexy_json_benchmark(
        &self,
        _name: &str,
        content: &str,
    ) -> (Option<Duration>, bool, Option<String>) {
        let start = Instant::now();

        match vexy_json::parse(content) {
            Ok(_) => (Some(start.elapsed()), true, None),
            Err(e) => (Some(start.elapsed()), false, Some(e.to_string())),
        }
    }

    fn run_ref_impl_benchmark(
        &self,
        _name: &str,
        content: &str,
    ) -> (Option<Duration>, bool, Option<String>) {
        let bun_path = format!("{}/.bun/bin/bun", std::env::var("HOME").unwrap_or_default());
        let jsonic_path = "/usr/local/bin/jsonic";

        let start = Instant::now();

        let mut command = Command::new(&bun_path);
        command
            .arg("--bun")
            .arg(jsonic_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match command.spawn() {
            Ok(mut child) => {
                // Write input to stdin
                if let Some(stdin) = child.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    if stdin.write_all(content.as_bytes()).is_err() {
                        return (
                            Some(start.elapsed()),
                            false,
                            Some("Failed to write to stdin".to_string()),
                        );
                    }
                }

                match child.wait_with_output() {
                    Ok(output) => {
                        let duration = start.elapsed();
                        let success = output.status.success();
                        let error = if !success && !output.stderr.is_empty() {
                            Some(String::from_utf8_lossy(&output.stderr).to_string())
                        } else {
                            None
                        };
                        (Some(duration), success, error)
                    }
                    Err(e) => (Some(start.elapsed()), false, Some(e.to_string())),
                }
            }
            Err(e) => (None, false, Some(format!("Failed to spawn jsonic: {e}"))),
        }
    }

    fn benchmark_file(&mut self, name: &str, file_path: &str) {
        println!("Benchmarking: {name}");

        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read {file_path}: {e}");
                return;
            }
        };

        // Run multiple iterations for better accuracy
        const ITERATIONS: usize = 100;
        let mut vexy_json_times = Vec::new();
        let mut ref_impl_times = Vec::new();
        let mut vexy_json_successes = 0;
        let mut ref_impl_successes = 0;
        let mut vexy_json_last_error = None;
        let mut jsonic_last_error = None;

        // Warm up
        for _ in 0..10 {
            let _ = self.run_vexy_json_benchmark(name, &content);
            let _ = self.run_ref_impl_benchmark(name, &content);
        }

        // Benchmark vexy_json
        for _ in 0..ITERATIONS {
            let (time, success, error) = self.run_vexy_json_benchmark(name, &content);
            if let Some(t) = time {
                vexy_json_times.push(t);
            }
            if success {
                vexy_json_successes += 1;
            }
            if let Some(e) = error {
                vexy_json_last_error = Some(e);
            }
        }

        // Benchmark jsonic
        for _ in 0..ITERATIONS {
            let (time, success, error) = self.run_ref_impl_benchmark(name, &content);
            if let Some(t) = time {
                ref_impl_times.push(t);
            }
            if success {
                ref_impl_successes += 1;
            }
            if let Some(e) = error {
                jsonic_last_error = Some(e);
            }
        }

        // Calculate averages
        let vexy_json_avg = if !vexy_json_times.is_empty() {
            Some(vexy_json_times.iter().sum::<Duration>() / vexy_json_times.len() as u32)
        } else {
            None
        };

        let jsonic_avg = if !ref_impl_times.is_empty() {
            Some(ref_impl_times.iter().sum::<Duration>() / ref_impl_times.len() as u32)
        } else {
            None
        };

        let result = BenchmarkResult {
            name: name.to_string(),
            vexy_json_time: vexy_json_avg,
            vexy_json_success: vexy_json_successes > ITERATIONS / 2,
            vexy_json_error: vexy_json_last_error,
            ref_impl_time: jsonic_avg,
            ref_impl_success: ref_impl_successes > ITERATIONS / 2,
            ref_impl_error: jsonic_last_error,
            input_size: content.len(),
            input_content: if content.len() > 200 {
                format!("{}...", &content[..200])
            } else {
                content
            },
        };

        println!(
            "  vexy_json: {:?} (success: {})",
            result.vexy_json_time, result.vexy_json_success
        );
        println!(
            "  jsonic: {:?} (success: {})",
            result.ref_impl_time, result.ref_impl_success
        );

        self.results.push(result);
    }

    fn generate_jekyll_report(&self) -> String {
        let mut markdown = String::new();

        // Jekyll front matter
        markdown.push_str("---\n");
        markdown.push_str("layout: default\n");
        markdown.push_str("title: Benchmark Results\n");
        markdown.push_str("nav_order: 6\n");
        markdown.push_str("---\n\n");

        // Title and introduction
        markdown.push_str("# Benchmark Results\n\n");
        markdown.push_str("Comprehensive performance comparison between vexy_json (Rust) and jsonic (JavaScript) parsers.\n\n");

        // Metadata
        markdown.push_str("## Test Environment\n\n");
        markdown.push_str(&format!(
            "- **Generated**: {}\n",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        for (key, value) in &self.environment {
            markdown.push_str(&format!(
                "- **{}**: {}\n",
                key.replace("_", " ").to_uppercase(),
                value
            ));
        }
        markdown.push('\n');

        // Summary statistics
        markdown.push_str("## Summary\n\n");
        let total_tests = self.results.len();
        let vexy_json_successes = self.results.iter().filter(|r| r.vexy_json_success).count();
        let ref_impl_successes = self.results.iter().filter(|r| r.ref_impl_success).count();

        markdown.push_str(&format!("- **Total test cases**: {total_tests}\n"));
        markdown.push_str(&format!(
            "- **vexy_json success rate**: {:.1}% ({}/{})\n",
            (vexy_json_successes as f64 / total_tests as f64) * 100.0,
            vexy_json_successes,
            total_tests
        ));
        markdown.push_str(&format!(
            "- **jsonic success rate**: {:.1}% ({}/{})\n",
            (ref_impl_successes as f64 / total_tests as f64) * 100.0,
            ref_impl_successes,
            total_tests
        ));

        // Performance comparison
        let mut vexy_json_faster_count = 0;
        let mut jsonic_faster_count = 0;
        let mut speed_ratios = Vec::new();

        for result in &self.results {
            if let (Some(vexy_json_time), Some(ref_impl_time)) =
                (result.vexy_json_time, result.ref_impl_time)
            {
                if vexy_json_time < ref_impl_time {
                    vexy_json_faster_count += 1;
                } else {
                    jsonic_faster_count += 1;
                }

                let ratio = ref_impl_time.as_nanos() as f64 / vexy_json_time.as_nanos() as f64;
                speed_ratios.push(ratio);
            }
        }

        if !speed_ratios.is_empty() {
            let avg_speedup = speed_ratios.iter().sum::<f64>() / speed_ratios.len() as f64;
            markdown.push_str(&format!(
                "- **Average vexy_json speedup**: {avg_speedup:.2}x\n"
            ));
            markdown.push_str(&format!(
                "- **vexy_json faster in**: {}/{} cases\n",
                vexy_json_faster_count,
                speed_ratios.len()
            ));
        }

        markdown.push('\n');

        // Detailed results table
        markdown.push_str("## Detailed Results\n\n");
        markdown.push_str("| Test Case | Input Size | vexy_json Time | vexy_json Success | jsonic Time | jsonic Success | Speedup |\n");
        markdown.push_str("|-----------|------------|------------|---------------|-------------|----------------|----------|\n");

        for result in &self.results {
            let vexy_json_time_str = match result.vexy_json_time {
                Some(time) => format!("{:.3}ms", time.as_secs_f64() * 1000.0),
                None => "N/A".to_string(),
            };

            let ref_impl_time_str = match result.ref_impl_time {
                Some(time) => format!("{:.3}ms", time.as_secs_f64() * 1000.0),
                None => "N/A".to_string(),
            };

            let speedup_str = match (result.vexy_json_time, result.ref_impl_time) {
                (Some(vexy_json), Some(jsonic)) => {
                    let ratio = jsonic.as_nanos() as f64 / vexy_json.as_nanos() as f64;
                    format!("{ratio:.2}x")
                }
                _ => "N/A".to_string(),
            };

            let vexy_json_success_icon = if result.vexy_json_success {
                "✅"
            } else {
                "❌"
            };
            let ref_impl_success_icon = if result.ref_impl_success { "✅" } else { "❌" };

            markdown.push_str(&format!(
                "| {} | {} bytes | {} | {} | {} | {} | {} |\n",
                result.name,
                result.input_size,
                vexy_json_time_str,
                vexy_json_success_icon,
                ref_impl_time_str,
                ref_impl_success_icon,
                speedup_str
            ));
        }

        markdown.push('\n');

        // Error analysis
        let vexy_json_errors: Vec<_> = self
            .results
            .iter()
            .filter(|r| !r.vexy_json_success && r.vexy_json_error.is_some())
            .collect();

        let ref_impl_errors: Vec<_> = self
            .results
            .iter()
            .filter(|r| !r.ref_impl_success && r.ref_impl_error.is_some())
            .collect();

        if !vexy_json_errors.is_empty() || !ref_impl_errors.is_empty() {
            markdown.push_str("## Error Analysis\n\n");

            if !vexy_json_errors.is_empty() {
                markdown.push_str("### vexy_json Errors\n\n");
                for result in vexy_json_errors {
                    markdown.push_str(&format!(
                        "**{}**: {}\n\n",
                        result.name,
                        result.vexy_json_error.as_ref().unwrap()
                    ));
                }
            }

            if !ref_impl_errors.is_empty() {
                markdown.push_str("### jsonic Errors\n\n");
                for result in ref_impl_errors {
                    markdown.push_str(&format!(
                        "**{}**: {}\n\n",
                        result.name,
                        result.ref_impl_error.as_ref().unwrap()
                    ));
                }
            }
        }

        // Test cases details
        markdown.push_str("## Test Cases\n\n");
        for result in &self.results {
            markdown.push_str(&format!("### {}\n\n", result.name));
            markdown.push_str("```json\n");
            markdown.push_str(&result.input_content);
            markdown.push_str("\n```\n\n");
        }

        // Methodology
        markdown.push_str("## Methodology\n\n");
        markdown.push_str("- Each test case is run 100 times after 10 warm-up runs\n");
        markdown.push_str("- Times are averaged across all successful runs\n");
        markdown.push_str("- vexy_json is tested via direct Rust function calls\n");
        markdown.push_str(&format!(
            "- jsonic is tested via `{} --bun {} < input`\n",
            format!("{}/.bun/bin/bun", std::env::var("HOME").unwrap_or_default()),
            "/usr/local/bin/jsonic"
        ));
        markdown.push_str("- Speedup is calculated as `ref_impl_time / vexy_json_time`\n");
        markdown
            .push_str("- Success is determined by whether parsing completes without errors\n\n");

        markdown.push_str("---\n\n");
        markdown.push_str(
            "*This report was automatically generated by the vexy_json benchmark suite.*\n",
        );

        markdown
    }
}
fn main() {
    println!("Running comprehensive vexy_json vs jsonic benchmark...");

    let mut suite = BenchmarkSuite::new();

    // Test cases to benchmark
    let test_cases = vec![
        ("simple", "benches/data/simple.json"),
        ("array", "benches/data/array.json"),
        ("nested", "benches/data/nested.json"),
        ("large_array", "benches/data/large_array.json"),
        ("deeply_nested", "benches/data/deeply_nested.json"),
        ("config", "benches/data/config.json"),
        ("forgiving", "benches/data/forgiving.json"),
        ("comments", "benches/data/comments.json"),
        ("unquoted_keys", "benches/data/unquoted_keys.json"),
        ("trailing_commas", "benches/data/trailing_commas.json"),
        ("implicit_object", "benches/data/implicit_object.json"),
        ("broken", "benches/data/broken.json"),
    ];

    // Run all benchmarks
    for (name, file_path) in test_cases {
        suite.benchmark_file(name, file_path);
    }
    // Generate and save Jekyll report
    let report = suite.generate_jekyll_report();

    match fs::write("docs/benchmarks.md", report) {
        Ok(_) => println!("\nBenchmark report saved to docs/benchmarks.md"),
        Err(e) => eprintln!("Failed to write report: {e}"),
    }

    println!("Benchmark complete!");
}
