# Contributing

Thank you for your interest in contributing to Vexy JSON! This guide will help you get started with development, testing, and submitting contributions.

## Development Setup

### Prerequisites

- **Rust**: Latest stable version (1.70.0+)
- **Git**: For version control
- **Node.js**: For JavaScript/WASM testing (16.0+)
- **Python**: For Python bindings testing (3.8+)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/vexyart/vexy-json.git
cd vexy-json

# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add required targets for WASM
rustup target add wasm32-unknown-unknown

# Install wasm-pack for WebAssembly builds
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Project Structure

```
vexy-json/
├── crates/
│   ├── core/           # Core parsing engine
│   ├── cli/            # Command-line interface  
│   ├── python/         # Python bindings
│   ├── wasm/           # WebAssembly bindings
│   ├── c-api/          # C/C++ bindings
│   └── serde/          # Serde integration
├── tests/              # Integration tests
├── benches/            # Benchmarks
├── examples/           # Example programs
├── docs-src/           # Documentation source
└── scripts/            # Build and utility scripts
```

## Build and Test

### Important: Use the Build Script

**DO NOT** run `cargo build`, `cargo test`, or `cargo clippy` directly. Always use the provided build script:

```bash
# Run complete build, format, lint, and test
./build.sh

# Check the output for any issues
cat ./build.log.txt
```

The build script ensures:
- Proper formatting with `rustfmt`
- Linting with `clippy`
- All crates build successfully
- Tests pass across all language bindings
- Documentation builds correctly

### Development Workflow

```bash
# Make your changes
vim src/some_file.rs

# Test your changes
./build.sh

# Check for any issues
cat ./build.log.txt

# If tests fail, fix issues and repeat
```

### Running Specific Tests

While the build script runs all tests, you can run specific test suites:

```bash
# Core Rust tests only
cargo test -p vexy-json-core

# Python binding tests
cd crates/python && python -m pytest

# JavaScript/WASM tests  
cd crates/wasm && npm test

# C/C++ tests
cd crates/c-api && make test
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench parser_comparison

# View benchmark results
open target/criterion/report/index.html
```

## Code Style and Standards

### Rust Code Style

We follow standard Rust conventions with some additions:

```rust
// Use descriptive function names
fn parse_object_with_recovery() -> Result<Value, Error> {
    // Implementation
}

// Prefer explicit error types
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedCharacter { char: char, position: usize },
    UnterminatedString { start: usize },
    InvalidNumber { text: String },
}

// Document public APIs thoroughly
/// Parses a JSON string with the given configuration.
/// 
/// # Arguments
/// 
/// * `input` - The JSON string to parse
/// * `config` - Parsing configuration options
/// 
/// # Examples
/// 
/// ```rust
/// use vexy_json::{VexyJson, Config};
/// 
/// let config = Config::permissive();
/// let value = VexyJson::parse_with_config(json, &config)?;
/// ```
/// 
/// # Errors
/// 
/// Returns `ParseError` if the input is not valid JSON according
/// to the provided configuration.
pub fn parse_with_config(input: &str, config: &Config) -> Result<Value, ParseError> {
    // Implementation
}
```

### Error Handling Patterns

```rust
// Use Result types consistently
pub fn parse_number(input: &str) -> Result<Number, NumberParseError> {
    // Implementation
}

// Provide helpful error context
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedCharacter { char, position } => {
                write!(f, "Unexpected character '{}' at position {}", char, position)
            }
            ParseError::UnterminatedString { start } => {
                write!(f, "Unterminated string starting at position {}", start)
            }
        }
    }
}

// Include suggestions when possible
impl ParseError {
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            ParseError::UnexpectedCharacter { char: ',', .. } => {
                Some("Try enabling trailing comma support")
            }
            ParseError::UnterminatedString { .. } => {
                Some("Check for missing closing quote")
            }
            _ => None,
        }
    }
}
```

### Testing Standards

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_object_parsing() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = VexyJson::parse(json).unwrap();
        
        assert_eq!(result.get("name").unwrap().as_str().unwrap(), "test");
        assert_eq!(result.get("value").unwrap().as_i64().unwrap(), 42);
    }

    #[test]
    fn test_error_handling() {
        let invalid_json = r#"{"name": "unclosed string}"#;
        let error = VexyJson::parse(invalid_json).unwrap_err();
        
        assert!(matches!(error, ParseError::UnterminatedString { .. }));
        assert!(error.suggestion().is_some());
    }

    #[test]
    fn test_forgiving_features() {
        let json_with_comments = r#"
        {
            // This is a comment
            "name": "test",
            "items": [1, 2, 3,] // Trailing comma
        }
        "#;
        
        let result = VexyJson::parse(json_with_comments).unwrap();
        assert!(result.get("name").is_some());
    }
}
```

## Adding New Features

### Feature Development Process

1. **Design Discussion**: Open an issue to discuss the feature
2. **Implementation**: Create a feature branch
3. **Testing**: Add comprehensive tests
4. **Documentation**: Update relevant docs
5. **Benchmarking**: Measure performance impact
6. **Review**: Submit pull request

### Example: Adding a New Forgiving Feature

Let's walk through adding support for hexadecimal numbers:

#### 1. Core Implementation

```rust
// In crates/core/src/parser/number.rs

pub fn parse_number(input: &str, config: &Config) -> Result<Number, ParseError> {
    // Existing decimal number parsing...
    
    // Add hexadecimal support
    if config.allow_hex_numbers() && input.starts_with("0x") || input.starts_with("0X") {
        return parse_hex_number(&input[2..]);
    }
    
    // Rest of implementation...
}

fn parse_hex_number(hex_str: &str) -> Result<Number, ParseError> {
    match i64::from_str_radix(hex_str, 16) {
        Ok(value) => Ok(Number::Integer(value)),
        Err(_) => Err(ParseError::InvalidHexNumber {
            text: hex_str.to_string(),
        }),
    }
}
```

#### 2. Configuration Support

```rust
// In crates/core/src/config.rs

#[derive(Debug, Clone)]
pub struct Config {
    allow_comments: bool,
    allow_trailing_commas: bool,
    allow_hex_numbers: bool, // New field
    // Other fields...
}

impl ConfigBuilder {
    pub fn allow_hex_numbers(mut self, enable: bool) -> Self {
        self.allow_hex_numbers = enable;
        self
    }
}
```

#### 3. Add Tests

```rust
// In tests/number_formats.rs

#[test]
fn test_hexadecimal_numbers() {
    let json = r#"{"value": 0xFF, "another": 0x1A2B}"#;
    
    let config = Config::builder()
        .allow_hex_numbers(true)
        .build();
    
    let result = VexyJson::parse_with_config(json, &config).unwrap();
    
    assert_eq!(result.get("value").unwrap().as_i64().unwrap(), 255);
    assert_eq!(result.get("another").unwrap().as_i64().unwrap(), 6699);
}

#[test]
fn test_hex_numbers_disabled() {
    let json = r#"{"value": 0xFF}"#;
    
    let config = Config::strict();
    let error = VexyJson::parse_with_config(json, &config).unwrap_err();
    
    assert!(matches!(error, ParseError::InvalidNumber { .. }));
}
```

#### 4. Update Language Bindings

```python
# In crates/python/src/lib.rs

#[pyfunction]
fn parse_with_options(
    json: &str,
    allow_comments: Option<bool>,
    allow_hex_numbers: Option<bool>, // New parameter
) -> PyResult<PyObject> {
    let config = Config::builder()
        .allow_comments(allow_comments.unwrap_or(true))
        .allow_hex_numbers(allow_hex_numbers.unwrap_or(false))
        .build();
    
    // Implementation...
}
```

#### 5. Documentation

```markdown
<!-- In docs/core-features.md -->

### Hexadecimal Numbers

Vexy JSON supports hexadecimal number literals:

```javascript
{
  "color": 0xFF0000,  // Red in hex
  "mask": 0x0F0F      // Bit mask
}
```

Enable with:

```rust
let config = Config::builder()
    .allow_hex_numbers(true)
    .build();
```
```

#### 6. Benchmarking

```rust
// In benches/number_parsing.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hex_number_parsing(c: &mut Criterion) {
    let json_with_hex = r#"{"values": [0xFF, 0x1234, 0xABCD]}"#;
    
    c.bench_function("parse hex numbers", |b| {
        b.iter(|| {
            VexyJson::parse(black_box(json_with_hex)).unwrap()
        })
    });
}

criterion_group!(benches, bench_hex_number_parsing);
criterion_main!(benches);
```

## Submitting Changes

### Pull Request Process

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/your-username/vexy-json.git
   cd vexy-json
   git remote add upstream https://github.com/vexyart/vexy-json.git
   ```

2. **Create Feature Branch**:
   ```bash
   git checkout -b feature/hex-number-support
   ```

3. **Make Changes**:
   - Implement your feature
   - Add tests
   - Update documentation
   - Run `./build.sh` to verify everything works

4. **Commit Changes**:
   ```bash
   git add .
   git commit -m "feat: add hexadecimal number support
   
   - Implement hex number parsing in core parser
   - Add configuration option for hex numbers
   - Include comprehensive tests
   - Update documentation with examples
   - Add benchmarks for performance testing"
   ```

5. **Push and Create PR**:
   ```bash
   git push origin feature/hex-number-support
   ```
   
   Then create a pull request on GitHub.

### Commit Message Format

We use conventional commits:

```
type(scope): brief description

Longer description explaining what and why, not how.

- Bullet points for multiple changes
- Reference issues with #123
- Breaking changes noted in footer

BREAKING CHANGE: describe what breaks
Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

### Pull Request Checklist

Before submitting your PR, ensure:

- [ ] `./build.sh` passes without errors
- [ ] New features have comprehensive tests
- [ ] Breaking changes are documented
- [ ] Performance impact is measured
- [ ] Documentation is updated
- [ ] Commit messages follow conventions
- [ ] Code follows project style guidelines

## Testing Guidelines

### Test Categories

1. **Unit Tests**: Test individual functions
2. **Integration Tests**: Test feature combinations
3. **Compatibility Tests**: Test against real-world JSON
4. **Performance Tests**: Benchmark critical paths
5. **Regression Tests**: Prevent previously fixed bugs

### Writing Good Tests

```rust
#[test]
fn test_specific_behavior() {
    // Arrange: Set up test data
    let json = r#"{"test": "data"}"#;
    let config = Config::permissive();
    
    // Act: Perform the operation
    let result = VexyJson::parse_with_config(json, &config);
    
    // Assert: Check the results
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.get("test").unwrap().as_str().unwrap(), "data");
}

#[test]
fn test_error_conditions() {
    let invalid_json = r#"{"invalid": }"#;
    
    let error = VexyJson::parse(invalid_json).unwrap_err();
    
    // Test specific error type
    assert!(matches!(error, ParseError::UnexpectedCharacter { .. }));
    
    // Test error message quality
    assert!(error.to_string().contains("unexpected"));
    
    // Test suggestions when available
    if let Some(suggestion) = error.suggestion() {
        assert!(!suggestion.is_empty());
    }
}
```

### Cross-Platform Testing

Test on multiple platforms when possible:

```bash
# Linux (CI handles this automatically)
./build.sh

# macOS
./build.sh

# Windows (if available)
build.bat
```

## Documentation

### Documentation Types

1. **API Documentation**: Rust docs (`cargo doc`)
2. **User Guide**: Markdown in `docs-src/`
3. **Examples**: Runnable code in `examples/`
4. **README**: Project overview

### Writing Documentation

```rust
/// Parses a JSON string into a Value tree.
///
/// This function accepts both standard JSON and Vexy JSON extensions
/// based on the provided configuration.
///
/// # Arguments
///
/// * `input` - The JSON string to parse
/// * `config` - Configuration controlling which features are enabled
///
/// # Returns
///
/// Returns `Ok(Value)` on successful parsing, or `Err(ParseError)`
/// if the input cannot be parsed.
///
/// # Examples
///
/// ```rust
/// use vexy_json::{VexyJson, Config};
///
/// // Parse standard JSON
/// let value = VexyJson::parse(r#"{"name": "test"}"#)?;
///
/// // Parse with comments enabled
/// let config = Config::builder().allow_comments(true).build();
/// let value = VexyJson::parse_with_config(r#"
///     {
///         // This is a comment
///         "name": "test"
///     }
/// "#, &config)?;
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The input is not valid JSON
/// - Enabled features are used but disabled in config
/// - Maximum parsing depth is exceeded
pub fn parse_with_config(input: &str, config: &Config) -> Result<Value, ParseError> {
    // Implementation
}
```

## Performance Considerations

### Benchmarking New Features

Always benchmark performance-critical changes:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_new_feature(c: &mut Criterion) {
    let test_data = setup_test_data();
    
    c.bench_function("new feature", |b| {
        b.iter(|| {
            new_feature_function(black_box(&test_data))
        })
    });
}

// Compare with baseline
fn bench_baseline(c: &mut Criterion) {
    let test_data = setup_test_data();
    
    c.bench_function("baseline", |b| {
        b.iter(|| {
            baseline_function(black_box(&test_data))
        })
    });
}
```

### Memory Usage

Monitor memory usage for new features:

```rust
#[cfg(test)]
fn test_memory_usage() {
    let initial_memory = get_memory_usage();
    
    // Run your code
    let result = expensive_operation();
    
    let final_memory = get_memory_usage();
    let memory_used = final_memory - initial_memory;
    
    // Assert reasonable memory usage
    assert!(memory_used < expected_maximum);
}
```

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community discussion
- **Discord**: Real-time chat (link in README)

### Mentorship

New contributors can request mentorship:

1. Comment on a "good first issue"
2. Tag `@maintainers` in the issue
3. We'll assign a mentor to help guide you

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [JSON Specification](https://tools.ietf.org/html/rfc8259)
- [WebAssembly Documentation](https://webassembly.org/)

Thank you for contributing to Vexy JSON! 🚀