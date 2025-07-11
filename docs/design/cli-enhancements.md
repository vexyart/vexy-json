---
layout: page
title: CLI Enhancements Design
permalink: /design/cli-enhancements/
parent: Design
nav_order: 2
---

# CLI Enhancements Design for vexy_json

## Overview

This document outlines the design for comprehensive CLI enhancements to the vexy_json command-line tool, building on the current basic implementation to provide a powerful and user-friendly JSON processing experience.

## Current State Analysis

**Existing CLI Features:**
- Basic stdin JSON parsing and compact output
- Comment-aware JSON processing (for non-comment content)
- Simple error reporting

**Limitations:**
- No file input/output options
- No pretty printing or formatting options
- No batch processing capabilities
- No watch mode for continuous monitoring
- Limited error context and reporting
- No query/filtering capabilities

## Enhancement Goals

1. **User Experience**: Make vexy_json the go-to CLI tool for JSON processing
2. **Feature Parity**: Match or exceed capabilities of popular JSON tools (jq, jsonlint)
3. **Rust Integration**: Leverage Rust's performance and safety for robust operations
4. **Flexibility**: Support various workflows from simple formatting to complex transformations

## Proposed CLI Interface

### Basic Usage (Enhanced)
```bash
# Current (unchanged for compatibility)
echo '{"key": "value"}' | vexy_json

# New file input/output
vexy_json input.json                    # Read from file, output to stdout
vexy_json input.json -o output.json     # Read from file, write to file
vexy_json -i input.json -o output.json  # Explicit input/output

# Multiple files
vexy_json file1.json file2.json         # Process multiple files
vexy_json *.json                        # Glob support
```

### Formatting Options
```bash
# Pretty printing (default when output is terminal)
vexy_json --pretty input.json
vexy_json -p input.json

# Compact output (default when piped)
vexy_json --compact input.json
vexy_json -c input.json

# Custom indentation
vexy_json --indent 4 input.json
vexy_json --indent tab input.json

# Sort keys
vexy_json --sort-keys input.json
```

### Validation and Analysis
```bash
# Validate only (exit code indicates success/failure)
vexy_json --validate input.json
vexy_json -v input.json

# Show statistics
vexy_json --stats input.json
# Output: {"objects": 5, "arrays": 3, "strings": 12, ...}

# Detailed error reporting
vexy_json --strict input.json    # Fail on any forgiving features
vexy_json --explain input.json   # Show what forgiving features were used
```

### Parser Options Control
```bash
# Disable specific forgiving features
vexy_json --no-comments input.json
vexy_json --no-trailing-commas input.json
vexy_json --no-unquoted-keys input.json
vexy_json --no-single-quotes input.json

# Enable specific features (when starting from strict mode)
vexy_json --strict --allow-comments input.json

# Newline as comma mode
vexy_json --newline-as-comma input.json
```

### Watch Mode
```bash
# Watch file for changes
vexy_json --watch input.json
vexy_json -w input.json

# Watch with auto-output
vexy_json -w input.json -o output.json

# Watch directory
vexy_json -w ./config/
```

### Batch Processing
```bash
# Process all JSON files in directory
vexy_json --batch ./data/ --output-dir ./processed/

# With transformation
vexy_json --batch ./data/ --pretty --sort-keys -o ./formatted/

# Parallel processing
vexy_json --parallel ./data/*.json
```

### Query and Filtering (Future Enhancement)
```bash
# Basic path extraction (jq-like)
vexy_json input.json --get ".users[0].name"

# Multiple paths
vexy_json input.json --get ".name" --get ".age"

# Simple filtering
vexy_json input.json --filter ".age > 30"
```

### Output Control
```bash
# Output to stderr instead of stdout
vexy_json --stderr input.json

# Silent mode (only exit codes)
vexy_json --silent input.json
vexy_json -s input.json

# Different output formats
vexy_json --output-format yaml input.json  # Future
vexy_json --output-format toml input.json  # Future
```

### Advanced Features
```bash
# Diff two JSON files (structural comparison)
vexy_json --diff file1.json file2.json

# Merge JSON files
vexy_json --merge base.json override.json

# Schema validation (future)
vexy_json --schema schema.json data.json

# Performance profiling
vexy_json --profile large-file.json
```

## Implementation Architecture

### Core Components

1. **CLI Parser (clap v4)**
   - Comprehensive argument parsing
   - Subcommands for complex operations
   - Environment variable support
   - Shell completion generation

2. **Input/Output Manager**
   - File handling with proper error recovery
   - Streaming support for large files
   - Memory-mapped files for performance
   - Progress bars for long operations

3. **Formatter Engine**
   - Pretty printing with configurable indentation
   - Compact output optimization
   - Key sorting algorithms
   - Color output support (when terminal detected)

4. **Validator Module**
   - Strict mode validation
   - Feature usage detection and reporting
   - Statistics collection
   - Error context extraction

5. **Watch System (notify crate)**
   - File system monitoring
   - Debouncing for rapid changes
   - Directory watching with filters
   - Change notification system

6. **Batch Processor**
   - Parallel processing with rayon
   - Progress tracking
   - Error aggregation
   - Transaction-like operations

### Error Handling Strategy

1. **Contextual Errors**
   ```
   Error at line 5, column 12:
     4 |     "name": "John",
     5 |     age: 30,
              ^^^
   Expected quoted key, found unquoted identifier 'age'
   
   Hint: Use --allow-unquoted-keys to permit this syntax
   ```

2. **Error Recovery**
   - Continue processing other files in batch mode
   - Provide partial output where possible
   - Suggest fixes for common issues

3. **Exit Codes**
   - 0: Success
   - 1: Parse error
   - 2: I/O error
   - 3: Validation error
   - 4: Invalid arguments

### Performance Considerations

1. **Streaming Architecture**
   - Process large files without loading entirely into memory
   - Incremental parsing for watch mode
   - Lazy evaluation where possible

2. **Parallel Processing**
   - Use rayon for multi-file operations
   - Configurable thread pool size
   - Work-stealing for load balancing

3. **Optimization Strategies**
   - SIMD operations for string processing
   - Memory pooling for repeated allocations
   - Zero-copy parsing where applicable

## Testing Strategy

### Unit Tests
- Each CLI option tested independently
- Error case coverage
- Edge cases (empty files, huge files, special characters)

### Integration Tests
- End-to-end command execution
- File I/O operations
- Pipe and redirection handling

### Performance Tests
- Benchmark against other JSON tools
- Memory usage profiling
- Large file handling

### Compatibility Tests
- Ensure backward compatibility
- Test on different platforms
- Shell integration testing

## Documentation Plan

### Man Page
- Comprehensive option documentation
- Examples for common use cases
- Troubleshooting section

### README Updates
- Quick start guide
- Feature comparison table
- Migration guide from other tools

### Interactive Help
- Context-sensitive help
- Did-you-mean suggestions
- Example snippets in error messages

## Migration Path

### Phase 1: Core Enhancements (Week 1-2)
- File I/O support
- Pretty printing
- Basic validation
- Enhanced error messages

### Phase 2: Advanced Features (Week 3-4)
- Watch mode
- Batch processing
- Parser option controls
- Statistics

### Phase 3: Power Features (Week 5-6)
- Parallel processing
- Query/filtering basics
- Diff/merge operations
- Performance optimizations

### Phase 4: Polish (Week 7-8)
- Documentation
- Shell completions
- Testing and benchmarking
- Release preparation

## Success Metrics

1. **Performance**: Process 1MB JSON in <100ms
2. **Usability**: 90% of operations require no manual reference
3. **Compatibility**: 100% backward compatibility maintained
4. **Reliability**: Zero panics in production use
5. **Adoption**: Featured in awesome-rust JSON tools section

## Open Questions

1. Should we implement a full jq-compatible query language?
2. How much functionality should be in the core vs. plugins?
3. Should we support YAML/TOML output in v1?
4. What level of JSON Schema support is needed?

## Conclusion

These CLI enhancements will transform vexy_json from a basic JSON parser into a comprehensive JSON processing toolkit. By focusing on user experience, performance, and flexibility, vexy_json can become the preferred choice for developers working with forgiving JSON formats.