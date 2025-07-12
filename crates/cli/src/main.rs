// this_file: crates/cli/src/main.rs

//! Enhanced vexy_json CLI with comprehensive JSON processing capabilities.

use clap::{Args, Parser};
use colored::*;
use rayon::prelude::*;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use vexy_json_core::ast::Value;
use vexy_json_core::error::{EnhancedParseResult, ParsingTier, RepairType};
use vexy_json_core::{
    parse_with_detailed_repair_tracking, parse_with_fallback, ParallelConfig, ParallelParser,
    ParserOptions,
};

#[derive(Parser, Debug)]
#[clap(
    name = "vexy_json", 
    version = env!("VEXY_JSON_VERSION", env!("CARGO_PKG_VERSION")), 
    about = "A forgiving JSON parser and processor",
    long_about = "vexy_json processes JSON with forgiving syntax including comments, trailing commas, unquoted keys, and more."
)]
struct CliArgs {
    /// Input files to process (if none provided, reads from stdin)
    #[clap(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Output file (if not provided, writes to stdout)
    #[clap(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Pretty print with indentation
    #[clap(short = 'p', long = "pretty", conflicts_with = "compact")]
    pretty: bool,

    /// Number of spaces for indentation (default: 2)
    #[clap(long = "indent", default_value = "2")]
    indent: usize,

    /// Compact output (remove all unnecessary whitespace)
    #[clap(short = 'c', long = "compact")]
    compact: bool,

    /// Validate JSON without output
    #[clap(short = 'v', long = "validate")]
    validate: bool,

    /// Watch mode - monitor files for changes
    #[clap(short = 'w', long = "watch")]
    watch: bool,

    /// Process multiple files in parallel
    #[clap(short = 'j', long = "parallel")]
    parallel: bool,

    /// Show detailed error information
    #[clap(long = "verbose-errors")]
    verbose_errors: bool,

    /// Enable JSON repair functionality
    #[clap(short = 'r', long = "repair")]
    repair: bool,

    /// Show detailed repair information
    #[clap(long = "repair-details")]
    repair_details: bool,

    /// Enable fallback parsing (fast → forgiving → repair)
    #[clap(long = "fallback")]
    fallback: bool,

    /// Enable parallel parsing for large JSON files
    #[clap(long = "parallel-parse")]
    parallel_parse: bool,

    /// Minimum chunk size for parallel parsing (bytes)
    #[clap(long = "chunk-size", default_value = "65536")]
    chunk_size: usize,

    /// Maximum threads for parallel parsing (0 = auto)
    #[clap(long = "max-threads", default_value = "0")]
    max_threads: usize,

    /// Parse as NDJSON (newline-delimited JSON)
    #[clap(long = "ndjson")]
    ndjson: bool,

    /// Parser options
    #[clap(flatten)]
    parser_opts: ParserOptionsArgs,
}

#[derive(Args, Debug)]
struct ParserOptionsArgs {
    /// Disable comment parsing
    #[clap(long = "no-comments")]
    no_comments: bool,

    /// Disable trailing comma support
    #[clap(long = "no-trailing-commas")]
    no_trailing_commas: bool,

    /// Disable unquoted key support
    #[clap(long = "no-unquoted-keys")]
    no_unquoted_keys: bool,

    /// Disable single quote support
    #[clap(long = "no-single-quotes")]
    no_single_quotes: bool,

    /// Disable implicit top-level structures
    #[clap(long = "no-implicit-top-level")]
    no_implicit_top_level: bool,

    /// Disable newline as comma
    #[clap(long = "no-newline-as-comma")]
    no_newline_as_comma: bool,

    /// Maximum parsing depth
    #[clap(long = "max-depth", default_value = "128")]
    max_depth: usize,
}

#[derive(Error, Debug)]
enum CliError {
    #[error("Parse error in file '{file}' at line {line}, column {col}: {message}")]
    ParseError {
        file: String,
        line: usize,
        col: usize,
        message: String,
    },
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Watch error: {0}")]
    WatchError(#[from] notify::Error),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

type Result<T> = std::result::Result<T, CliError>;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let result = if args.watch {
        watch_mode(&args).await
    } else if args.files.is_empty() {
        process_stdin(&args).await
    } else if args.parallel && args.files.len() > 1 {
        process_files_parallel(&args)
    } else {
        process_files_sequential(&args).await
    };

    if let Err(e) = result {
        print_error(&e, &args);
        std::process::exit(1);
    }
}

async fn process_stdin(args: &CliArgs) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        if !args.validate {
            eprintln!("{}", "No input provided".yellow());
        }
        return Ok(());
    }

    process_content(&input, "<stdin>", args)?;
    Ok(())
}

async fn process_files_sequential(args: &CliArgs) -> Result<()> {
    for file in &args.files {
        process_single_file(file, args)?;
    }
    Ok(())
}

fn process_files_parallel(args: &CliArgs) -> Result<()> {
    let results: std::result::Result<Vec<_>, _> = args
        .files
        .par_iter()
        .map(|file| process_single_file(file, args))
        .collect();

    results?;
    Ok(())
}

fn process_single_file(file: &PathBuf, args: &CliArgs) -> Result<()> {
    if !file.exists() {
        return Err(CliError::FileNotFound(file.display().to_string()));
    }

    let content = fs::read_to_string(file)?;
    process_content(&content, &file.display().to_string(), args)?;
    Ok(())
}

fn process_content(content: &str, source: &str, args: &CliArgs) -> Result<()> {
    // Check if parallel parsing is requested
    if args.ndjson {
        // Parse as NDJSON
        return process_ndjson_content(content, source, args);
    } else if args.parallel_parse {
        // Use parallel parsing for large files
        return process_parallel_content(content, source, args);
    }

    let parser_options = create_parser_options(&args.parser_opts);

    // Choose parsing strategy based on CLI options
    if args.repair_details {
        // Use detailed repair tracking
        let result = parse_with_detailed_repair_tracking(content, parser_options);
        if result.errors.is_empty() {
            if args.validate {
                print_validation_result_with_repair(source, &result, args);
            } else {
                let formatted = format_output(&result.value, args);
                write_output(&formatted, args)?;
            }
            print_repair_summary(&result, args);
        } else {
            // Create error from the first error in the result
            let first_error = &result.errors[0];
            let cli_error = format_parse_error(first_error, source, content);
            return Err(cli_error);
        }
    } else {
        // Use fallback parsing by default (fast → forgiving → repair)
        let result = parse_with_fallback(content, parser_options);
        if result.errors.is_empty() {
            if args.validate {
                print_validation_result_with_repair(source, &result, args);
            } else {
                let formatted = format_output(&result.value, args);
                write_output(&formatted, args)?;
            }
            if args.repair_details {
                print_repair_info(&result, args);
            }
        } else {
            // Create error from the first error in the result
            let first_error = &result.errors[0];
            let cli_error = format_parse_error(first_error, source, content);
            return Err(cli_error);
        }
    }

    Ok(())
}

fn process_parallel_content(content: &str, source: &str, args: &CliArgs) -> Result<()> {
    let config = ParallelConfig {
        min_chunk_size: args.chunk_size,
        max_threads: args.max_threads,
        optimize_chunks: true,
    };

    let parser = ParallelParser::with_config(config);

    match parser.parse(content) {
        Ok(value) => {
            if args.validate {
                println!(
                    "{} {}",
                    "✓".green(),
                    format!("{} is valid JSON", source).green()
                );
            } else {
                let formatted = format_output(&value, args);
                write_output(&formatted, args)?;
            }
        }
        Err(e) => {
            let cli_error = format_parse_error(&e, source, content);
            return Err(cli_error);
        }
    }

    Ok(())
}

fn process_ndjson_content(content: &str, source: &str, args: &CliArgs) -> Result<()> {
    let config = ParallelConfig {
        min_chunk_size: args.chunk_size,
        max_threads: args.max_threads,
        optimize_chunks: true,
    };

    let parser = ParallelParser::with_config(config);

    match parser.parse_ndjson(content) {
        Ok(values) => {
            if args.validate {
                println!(
                    "{} {} parsed {} JSON objects",
                    "✓".green(),
                    source.green(),
                    values.len()
                );
            } else {
                for (i, value) in values.iter().enumerate() {
                    let formatted = format_output(value, args);
                    if i > 0 && args.pretty {
                        println!(); // Add blank line between objects in pretty mode
                    }
                    write_output(&formatted, args)?;
                    if !args.pretty {
                        println!(); // Add newline for NDJSON output
                    }
                }
            }
        }
        Err(e) => {
            let cli_error = format_parse_error(&e, source, content);
            return Err(cli_error);
        }
    }

    Ok(())
}

async fn watch_mode(args: &CliArgs) -> Result<()> {
    use notify::{EventKind, RecursiveMode, Watcher};
    use tokio::sync::mpsc;

    if args.files.is_empty() {
        eprintln!("{}", "Watch mode requires at least one file".red());
        return Ok(());
    }

    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if let Err(e) = tx.blocking_send(event) {
                eprintln!("Watch error: {}", e);
            }
        }
    })?;

    // Watch all specified files
    for file in &args.files {
        watcher.watch(file, RecursiveMode::NonRecursive)?;
    }

    println!(
        "{} {} files for changes... (Press Ctrl+C to stop)",
        "Watching".cyan().bold(),
        args.files.len()
    );

    // Process files initially
    for file in &args.files {
        if let Err(e) = process_single_file(file, args) {
            print_error(&e, args);
        }
    }

    while let Some(event) = rx.recv().await {
        if matches!(event.kind, EventKind::Modify(_)) {
            for path in event.paths {
                if args.files.contains(&path) {
                    println!("\n{} {}", "File changed:".yellow(), path.display());

                    // Small delay to ensure file write is complete
                    sleep(Duration::from_millis(100)).await;

                    if let Err(e) = process_single_file(&path, args) {
                        print_error(&e, args);
                    }
                }
            }
        }
    }

    Ok(())
}
fn create_parser_options(args: &ParserOptionsArgs) -> ParserOptions {
    ParserOptions {
        allow_comments: !args.no_comments,
        allow_trailing_commas: !args.no_trailing_commas,
        allow_unquoted_keys: !args.no_unquoted_keys,
        allow_single_quotes: !args.no_single_quotes,
        implicit_top_level: !args.no_implicit_top_level,
        newline_as_comma: !args.no_newline_as_comma,
        max_depth: args.max_depth,
        enable_repair: true,
        max_repairs: 100,
        fast_repair: false,
        report_repairs: true,
    }
}

fn print_validation_result_with_repair(
    source: &str,
    result: &EnhancedParseResult<Value>,
    args: &CliArgs,
) {
    let status = match result.parsing_tier {
        ParsingTier::Fast => "✓ Valid JSON (fast path)".green(),
        ParsingTier::Forgiving => "✓ Valid JSON (forgiving path)".yellow(),
        ParsingTier::Repair => "✓ Valid JSON (repaired)".cyan(),
    };

    if args.files.len() > 1 || args.watch {
        println!("{}: {}", source, status);
    } else {
        println!("{}", status);
    }
}

fn print_repair_info(result: &EnhancedParseResult<Value>, _args: &CliArgs) {
    if !result.repairs.is_empty() {
        println!("{}", "Repair actions performed:".yellow().bold());
        for action in &result.repairs {
            match action.action_type {
                RepairType::InsertBracket => {
                    println!(
                        "  • Added missing bracket at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::RemoveBracket => {
                    println!(
                        "  • Removed extra bracket at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::ReplaceBracket => {
                    println!(
                        "  • Replaced '{}' with '{}' at position {}: {}",
                        action.original, action.replacement, action.position, action.description
                    );
                }
                RepairType::BalanceQuotes => {
                    println!(
                        "  • Balanced quotes at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::InsertComma => {
                    println!(
                        "  • Added comma at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::RemoveComma => {
                    println!(
                        "  • Removed comma at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::InsertText => {
                    println!(
                        "  • Inserted text at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::ReplaceText => {
                    println!(
                        "  • Replaced text at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::ReplaceQuotes => {
                    println!(
                        "  • Replaced quotes at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::TypeCoercion => {
                    println!(
                        "  • Applied type coercion at position {}: {}",
                        action.position, action.description
                    );
                }
                RepairType::QuoteKey => {
                    println!(
                        "  • Added quotes to object key at position {}: {}",
                        action.position, action.description
                    );
                }
            }
        }
    }
}

fn print_repair_summary(result: &EnhancedParseResult<Value>, _args: &CliArgs) {
    println!("{}", "=== Repair Summary ===".cyan().bold());
    println!("Parsing tier: {:?}", result.parsing_tier);
    println!("Repair actions: {}", result.repairs.len());

    if !result.repairs.is_empty() {
        println!("\n{}", "Detailed repair actions:".yellow().bold());
        for (i, action) in result.repairs.iter().enumerate() {
            println!(
                "{}. {:?} at position {}: {}",
                i + 1,
                action.action_type,
                action.position,
                action.description
            );
        }
    }

    if !result.errors.is_empty() {
        println!("\n{}", "Errors encountered:".red().bold());
        for (i, error) in result.errors.iter().enumerate() {
            println!("{}. {}", i + 1, error);
        }
    }
}

fn format_output(value: &Value, args: &CliArgs) -> String {
    if args.compact {
        format_json_compact(value)
    } else if args.pretty {
        format_json_pretty(value, args.indent)
    } else {
        // Default: compact for single values, pretty for objects/arrays
        match value {
            Value::Object(_) | Value::Array(_) => format_json_pretty(value, args.indent),
            _ => format_json_compact(value),
        }
    }
}

fn format_json_compact(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", escape_json_string(s)),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_json_compact).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", escape_json_string(k), format_json_compact(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
    }
}

fn format_json_pretty(value: &Value, indent: usize) -> String {
    format_value_with_indent(value, 0, indent)
}

fn format_value_with_indent(value: &Value, current_indent: usize, indent_size: usize) -> String {
    let spaces = " ".repeat(current_indent);
    let inner_spaces = " ".repeat(current_indent + indent_size);

    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", escape_json_string(s)),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let mut result = "[\n".to_string();
                for (i, item) in arr.iter().enumerate() {
                    result.push_str(&inner_spaces);
                    result.push_str(&format_value_with_indent(
                        item,
                        current_indent + indent_size,
                        indent_size,
                    ));
                    if i < arr.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&spaces);
                result.push(']');
                result
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut result = "{\n".to_string();
                let mut entries: Vec<_> = obj.iter().collect();
                entries.sort_by_key(|(k, _)| *k);

                for (i, (key, value)) in entries.iter().enumerate() {
                    result.push_str(&inner_spaces);
                    result.push_str(&format!("\"{}\": ", escape_json_string(key)));
                    result.push_str(&format_value_with_indent(
                        value,
                        current_indent + indent_size,
                        indent_size,
                    ));
                    if i < entries.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&spaces);
                result.push('}');
                result
            }
        }
    }
}

fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            ch if ch.is_control() => {
                result.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => result.push(ch),
        }
    }
    result
}

fn write_output(content: &str, args: &CliArgs) -> Result<()> {
    if let Some(output_file) = &args.output {
        fs::write(output_file, content)?;
    } else {
        print!("{}", content);
        io::stdout().flush()?;
    }
    Ok(())
}

fn format_parse_error(error: &vexy_json_core::Error, file: &str, content: &str) -> CliError {
    // Try to extract position information from the error
    let error_str = error.to_string();

    // Parse error message to extract position if available
    if let Some(pos) = extract_position_from_error(&error_str) {
        let (line, col) = calculate_line_column(content, pos);
        CliError::ParseError {
            file: file.to_string(),
            line,
            col,
            message: error_str,
        }
    } else {
        CliError::ParseError {
            file: file.to_string(),
            line: 1,
            col: 1,
            message: error_str,
        }
    }
}

fn extract_position_from_error(error_str: &str) -> Option<usize> {
    // Try to extract position from error messages like "at position 42"
    if let Some(pos_str) = error_str.split("position ").nth(1) {
        if let Some(pos_str) = pos_str.split_whitespace().next() {
            pos_str.parse().ok()
        } else {
            None
        }
    } else {
        None
    }
}

fn calculate_line_column(content: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in content.char_indices() {
        if i >= position {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

fn print_error(error: &CliError, args: &CliArgs) {
    eprintln!("{}", "Error:".red().bold());

    if args.verbose_errors {
        eprintln!("{}", error);

        // If it's a parse error with context, show it
        if let CliError::ParseError {
            file, line, col, ..
        } = error
        {
            if file != "<stdin>" {
                if let Ok(content) = fs::read_to_string(file) {
                    print_error_context(&content, *line, *col);
                }
            }
        }
    } else {
        // Simple error message
        match error {
            CliError::ParseError { file, message, .. } => {
                eprintln!("{}: {}", file, message);
            }
            _ => eprintln!("{}", error),
        }
    }
}

fn print_error_context(content: &str, line: usize, col: usize) {
    let lines: Vec<&str> = content.lines().collect();
    let start = line.saturating_sub(2);
    let end = std::cmp::min(line + 2, lines.len());

    eprintln!();
    for (i, line_content) in lines[start..end].iter().enumerate() {
        let line_num = start + i + 1;
        let prefix = if line_num == line {
            format!("{:4} > ", line_num).red().bold()
        } else {
            format!("{:4}   ", line_num).white().dimmed()
        };

        eprintln!("{}{}", prefix, line_content);

        if line_num == line {
            let pointer = " ".repeat(6 + col.saturating_sub(1)) + "^";
            eprintln!("{}", pointer.red().bold());
        }
    }
}
