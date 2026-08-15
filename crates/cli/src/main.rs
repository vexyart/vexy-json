// this_file: crates/cli/src/main.rs

//! Enhanced vexy_json CLI with comprehensive JSON processing capabilities.

use clap::{Args, Parser};
use colored::*;
use rayon::prelude::*;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use vexy_json_core::ast::Value;
use vexy_json_core::{parse_with_fallback, ParallelConfig, ParallelParser, ParserOptions};

const MAX_LMS_CAPTURE_BYTES: u64 = 256 * 1024 * 1024;
const LMS_COMMAND_DESCRIPTION: &str = "lms ls --json --llm";

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

    /// Lossily close safely incomplete JSON at unexpected EOF
    #[clap(
        short = 'r',
        long = "repair",
        conflicts_with_all = ["watch", "ndjson", "parallel_parse"]
    )]
    repair: bool,

    /// Show detailed repair information
    #[clap(long = "repair-details", requires = "repair")]
    repair_details: bool,

    /// Losslessly capture and parse `lms ls --json --llm` via a regular file
    #[clap(
        long = "lms",
        conflicts_with_all = [
            "files",
            "watch",
            "parallel",
            "repair",
            "repair_details",
            "parallel_parse",
            "ndjson"
        ]
    )]
    lms: bool,

    /// Use fast → forgiving parser fallback (never implies --repair)
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
    #[error("Input from {input_name} is not valid UTF-8 at byte {byte}")]
    InvalidUtf8 { input_name: String, byte: usize },
    #[error("Cannot safely repair {input_name} at byte {byte}: {message}")]
    UnsafeRepair {
        input_name: String,
        byte: usize,
        message: String,
    },
    #[error("Failed to execute `lms ls --json --llm`: {0}")]
    LmsSpawn(std::io::Error),
    #[error("`lms ls --json --llm` exited unsuccessfully ({status})")]
    LmsExit { status: String },
    #[error("`lms ls --json --llm` captured no output")]
    EmptyLmsCapture,
    #[error("`lms ls --json --llm` capture is too large ({bytes} bytes; maximum 256 MiB)")]
    OversizedLmsCapture { bytes: u64 },
    #[error("Failed to remove the temporary lms capture: {0}")]
    TempCleanup(std::io::Error),
    #[error("Downstream output closed before all JSON was written (broken pipe)")]
    BrokenPipe,
}

type Result<T> = std::result::Result<T, CliError>;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let result = if args.lms {
        process_lms(&args).await
    } else if args.watch {
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
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    let input = decode_utf8(bytes, "<stdin>")?;

    if input.trim().is_empty() {
        if !args.validate {
            eprintln!("{}", "No input provided".yellow());
        }
        return Ok(());
    }

    process_content(&input, "<stdin>", args)?;
    Ok(())
}

async fn process_lms(args: &CliArgs) -> Result<()> {
    let mut capture = NamedTempFile::new()?;
    let child_stdout = capture.reopen()?;
    let status = Command::new("lms")
        .args(["ls", "--json", "--llm"])
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::inherit())
        .status()
        .map_err(CliError::LmsSpawn)?;

    if !status.success() {
        return Err(CliError::LmsExit {
            status: status.code().map_or_else(
                || "terminated by signal".to_string(),
                |code| code.to_string(),
            ),
        });
    }

    let captured_bytes = capture.as_file().metadata()?.len();
    if captured_bytes == 0 {
        return Err(CliError::EmptyLmsCapture);
    }
    if captured_bytes > MAX_LMS_CAPTURE_BYTES {
        return Err(CliError::OversizedLmsCapture {
            bytes: captured_bytes,
        });
    }

    capture.as_file_mut().seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::with_capacity(captured_bytes as usize);
    capture.as_file_mut().read_to_end(&mut bytes)?;
    capture.close().map_err(CliError::TempCleanup)?;

    let input = decode_utf8(bytes, "<lms regular-file capture>")?;
    process_content(&input, "<lms regular-file capture>", args)?;
    eprintln!(
        "Captured {captured_bytes} bytes from `{LMS_COMMAND_DESCRIPTION}` via a regular temporary file (lossless input path)."
    );
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

    let content = decode_utf8(fs::read(file)?, &file.display().to_string())?;
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

    let result = parse_with_fallback(content, parser_options.clone());
    if result.errors.is_empty() {
        return emit_value(&result.value, source, args, false);
    }

    let first_error = &result.errors[0];
    if !args.repair {
        return Err(format_parse_error(first_error, source, content));
    }

    let repair = repair_unexpected_eof(content, source)?;
    let repaired_result = parse_with_fallback(&repair.content, parser_options);
    if !repaired_result.errors.is_empty() {
        return Err(CliError::UnsafeRepair {
            input_name: source.to_string(),
            byte: content.len(),
            message:
                "closing the open structures did not produce valid JSON; no output was emitted"
                    .to_string(),
        });
    }

    emit_value(&repaired_result.value, source, args, true)?;
    eprintln!(
        "WARNING: unexpected EOF at byte {}; emitted partial/lossy JSON.",
        content.len()
    );
    eprintln!("Repair actions: {}.", repair.actions.join("; "));
    eprintln!("Missing upstream bytes were not recovered; use --lms when complete data matters.");
    Ok(())
}

fn emit_value(value: &Value, source: &str, args: &CliArgs, repaired: bool) -> Result<()> {
    if args.validate {
        let qualifier = if repaired {
            " (partial/lossy repair)"
        } else {
            ""
        };
        if args.files.len() > 1 || args.watch {
            println!("{source} is valid JSON{qualifier}");
        } else {
            println!("Valid JSON{qualifier}");
        }
    } else {
        write_output(&format_output(value, args), args)?;
    }
    Ok(())
}

struct EofRepair {
    content: String,
    actions: Vec<String>,
}

fn repair_unexpected_eof(content: &str, source: &str) -> Result<EofRepair> {
    let mut stack = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let bytes = content.as_bytes();
    let mut index = 0;
    let mut line_comment_at_eof = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }

        if byte == b'/' && index + 1 < bytes.len() {
            match bytes[index + 1] {
                b'/' => {
                    index += 2;
                    while index < bytes.len() && bytes[index] != b'\n' {
                        index += 1;
                    }
                    line_comment_at_eof = index == bytes.len();
                    continue;
                }
                b'*' => {
                    index += 2;
                    let mut closed = false;
                    while index + 1 < bytes.len() {
                        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
                            index += 2;
                            closed = true;
                            break;
                        }
                        index += 1;
                    }
                    if !closed {
                        return unsafe_repair(
                            source,
                            content.len(),
                            "EOF occurred inside a block comment",
                        );
                    }
                    continue;
                }
                _ => {}
            }
        }

        match byte {
            b'"' | b'\'' => quote = Some(byte),
            b'{' | b'[' => stack.push(byte),
            b'}' => pop_expected(&mut stack, b'{', source, index)?,
            b']' => pop_expected(&mut stack, b'[', source, index)?,
            _ => {}
        }
        index += 1;
    }

    if quote.is_some() || escaped {
        return unsafe_repair(
            source,
            content.len(),
            "EOF occurred inside a quoted string; closing it would fabricate string contents",
        );
    }
    if stack.is_empty() {
        return unsafe_repair(
            source,
            content.len(),
            "input did not end inside an open JSON structure",
        );
    }
    if line_comment_at_eof {
        return unsafe_repair(
            source,
            content.len(),
            "EOF occurred inside a line comment; appending closures would change comment contents",
        );
    }
    if !has_safe_eof_token(content) {
        return unsafe_repair(
            source,
            content.len(),
            "the final token is incomplete or no complete value can be closed safely",
        );
    }

    let mut repaired = content.to_string();
    let mut actions = Vec::with_capacity(stack.len());
    for opening in stack.iter().rev() {
        let closing = if *opening == b'{' { '}' } else { ']' };
        repaired.push(closing);
        actions.push(format!(
            "inserted closing '{closing}' at byte {}",
            content.len()
        ));
    }

    Ok(EofRepair {
        content: repaired,
        actions,
    })
}

fn pop_expected(stack: &mut Vec<u8>, expected: u8, source: &str, byte: usize) -> Result<()> {
    if stack.pop() == Some(expected) {
        return Ok(());
    }
    unsafe_repair(
        source,
        byte,
        "encountered mismatched or unmatched closing delimiter before EOF",
    )
}

fn unsafe_repair<T>(source: &str, byte: usize, message: &str) -> Result<T> {
    Err(CliError::UnsafeRepair {
        input_name: source.to_string(),
        byte,
        message: message.to_string(),
    })
}

fn has_safe_eof_token(content: &str) -> bool {
    let trimmed = content.trim_end();
    let Some(last) = trimmed.as_bytes().last().copied() else {
        return false;
    };

    match last {
        b'{' | b'[' | b'}' | b']' | b'"' | b'\'' => true,
        b'0'..=b'9' => trailing_scalar(trimmed).is_some_and(is_complete_json_number),
        b'e' => matches!(trailing_scalar(trimmed), Some("true") | Some("false")),
        b'l' => trailing_scalar(trimmed) == Some("null"),
        _ => false,
    }
}

fn trailing_scalar(content: &str) -> Option<&str> {
    let start = content
        .rfind(|character: char| {
            character.is_whitespace() || matches!(character, '[' | ']' | '{' | '}' | ',' | ':')
        })
        .map_or(0, |position| position + 1);
    let scalar = &content[start..];
    (!scalar.is_empty()).then_some(scalar)
}

fn is_complete_json_number(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = usize::from(bytes.first() == Some(&b'-'));
    if index >= bytes.len() {
        return false;
    }

    if bytes[index] == b'0' {
        index += 1;
    } else if bytes[index].is_ascii_digit() && bytes[index] != b'0' {
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
    } else {
        return false;
    }

    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == fraction_start {
            return false;
        }
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == exponent_start {
            return false;
        }
    }

    index == bytes.len()
}

fn decode_utf8(bytes: Vec<u8>, source: &str) -> Result<String> {
    String::from_utf8(bytes).map_err(|error| CliError::InvalidUtf8 {
        input_name: source.to_string(),
        byte: error.utf8_error().valid_up_to(),
    })
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
                    format!("{source} is valid JSON").green()
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
                eprintln!("Watch error: {e}");
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
        enable_repair: false,
        max_repairs: 100,
        fast_repair: false,
        report_repairs: false,
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
        let stdout = io::stdout();
        let mut output = stdout.lock();
        output
            .write_all(content.as_bytes())
            .map_err(map_output_error)?;
        output.flush().map_err(map_output_error)?;
    }
    Ok(())
}

fn map_output_error(error: std::io::Error) -> CliError {
    if error.kind() == std::io::ErrorKind::BrokenPipe {
        CliError::BrokenPipe
    } else {
        CliError::IoError(error)
    }
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
        eprintln!("{error}");

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
                eprintln!("{file}: {message}");
            }
            _ => eprintln!("{error}"),
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
            format!("{line_num:4} > ").red().bold()
        } else {
            format!("{line_num:4}   ").white().dimmed()
        };

        eprintln!("{prefix}{line_content}");

        if line_num == line {
            let pointer = " ".repeat(6 + col.saturating_sub(1)) + "^";
            eprintln!("{}", pointer.red().bold());
        }
    }
}
