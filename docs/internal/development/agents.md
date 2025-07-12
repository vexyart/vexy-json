---
title: AI Agent Development Guidelines
nav_order: 20
parent: Development
has_children: false
---

# AI Agent Development Guidelines

This document provides guidance for AI agents (Claude Code, etc.) when working with code in this repository.

## 1. Project Overview

`vexy_json` is a Rust port of the JavaScript library `the reference implementation`, a forgiving JSON parser. The reference JavaScript implementation is located in the `ref/the reference implementation/` directory.

## 2. Development Status

This project is in an active development phase. The core parsing engine is implemented, along with a comprehensive test suite, benchmarks, and WASM support. The focus is on achieving full API compatibility with `the reference implementation`, refining the idiomatic Rust API, and improving performance.

## 3. Rust Implementation

### 3.1. Module Organization

The Rust implementation is a cargo workspace organized into several crates:

-   `crates/core`: The core parsing engine.
    -   `src/lib.rs`: The main library crate root, exporting the public API.
    -   `src/parser.rs`: Contains the core recursive descent parsing logic.
    -   `src/lexer.rs`: The primary tokenizer for the input string.
    -   `src/ast/value.rs`: Defines the `Value` enum, which represents parsed JSON data.
    -   `src/error/mod.rs`: Implements custom error types for parsing failures.
-   `crates/cli`: The command-line interface.
    -   `src/main.rs`: The entry point for the CLI binary.
-   `crates/serde`: Provides `serde` integration for `vexy_json::Value`.
-   `crates/wasm`: Contains WebAssembly bindings to expose `vexy_json` to JavaScript environments.
-   `crates/test-utils`: Utility functions for testing.

### 3.2. Core Features

-   **Standard JSON Parsing (RFC 8259):** Full support for the official JSON specification.
-   **Forgiving Features:** Compatibility with `the reference implementation`'s non-standard features is a primary goal:
    -   Single-line (`//`) and multi-line (`/* */`) comments.
    -   Trailing commas in objects and arrays.
    -   Unquoted object keys (where unambiguous).
    -   Implicit top-level objects and arrays.
    -   Single-quoted strings.
    -   Newline characters as comma separators.

### 3.3. Architecture & Best Practices

-   **Error Handling:** Uses `Result<T, E>` and a custom `Error` enum (`src/error.rs`) for robust error handling with location information.
-   **Testing:**
    -   Unit and integration tests are located in the `tests/` directory, ported from `the reference implementation`'s test suite.
    -   The `examples/` directory contains numerous small, runnable programs for debugging specific features.
    -   Benchmarking is performed using `criterion.rs`, with benchmarks defined in the `benches/` directory.
-   **Extensibility:** The architecture uses Rust's traits and pattern matching for clarity and maintainability, avoiding a direct port of the JavaScript plugin system in favor of a more idiomatic approach.
-   **Performance:** The implementation aims for high performance, with ongoing benchmarking to compare against `serde_json` and `the reference implementation`.
-   **WASM Target:** A key feature is the ability to compile to WebAssembly, providing a performant `vexy_json` parser for web browsers and Node.js. The `wasm-pack` tool is used for building the WASM package.

## 4. Development Workflow

This project uses a specific workflow for development and testing. Adhere to the following commands.

### 4.1. Build and Test

**DO NOT** run `cargo build`, `cargo test`, or `cargo clippy` directly. Instead, use the provided build script, which handles all necessary steps, including formatting, linting, building, and testing.

```bash
./build.sh
```

After running the script, always review the output log to check for errors or warnings:

```bash
cat ./build.log.txt
```

### 4.2. Reference Implementation (the reference implementation)

When working with the reference JavaScript implementation in `ref/the reference implementation/`:

```bash
cd ref/the reference implementation

# Build the TypeScript code
npm run build

# Run all tests
npm test

# Run specific tests
npm run test-some -- <test-pattern>
```

---

# Consolidated Software Development Rules

## 5. Pre-Work Preparation

### 5.1. Before Starting Any Work
- **ALWAYS** read `WORK.md` in the main project folder for work progress
- Read `README.md` to understand the project
- STEP BACK and THINK HEAVILY STEP BY STEP about the task
- Consider alternatives and carefully choose the best option
- Check for existing solutions in the codebase before starting

### 5.2. Project Documentation to Maintain
- `README.md` - purpose and functionality
- `CHANGELOG.md` - past change release notes (accumulative)
- `PLAN.md` - detailed future goals, clear plan that discusses specifics
- `TODO.md` - flat simplified itemized `- [ ]`-prefixed representation of `PLAN.md`
- `docs/internal/WORK.md` - work progress updates

## 6. General Coding Principles

### 6.1. Core Development Approach
- Iterate gradually, avoiding major changes
- Focus on minimal viable increments and ship early
- Minimize confirmations and checks
- Preserve existing code/structure unless necessary
- Check often the coherence of the code you're writing with the rest of the code
- Analyze code line-by-line

### 6.2. Code Quality Standards
- Use constants over magic numbers
- Write explanatory docstrings/comments that explain what and WHY
- Explain where and how the code is used/referred to elsewhere
- Handle failures gracefully with retries, fallbacks, user guidance
- Address edge cases, validate assumptions, catch errors early
- Let the computer do the work, minimize user decisions
- Reduce cognitive load, beautify code
- Modularize repeated logic into concise, single-purpose functions
- Favor flat over nested structures

## 7. Tool Usage (When Available)

### 7.1. MCP Tools to Consult
- `codex` tool - for additional reasoning, summarization of files and second opinion
- `context7` tool - for most up-to-date software package documentation
- `sequentialthinking` tool - to think about the best way to solve tasks
- `perplexity_ask` - for up-to-date information or context

### 7.2. Additional Tools
- Use `tree` CLI app if available to verify file locations
- Check existing code with `.venv` folder to scan and consult dependency source code
- Run `DIR="."; uvx codetoprompt --compress --output "$DIR/llms.txt"  --respect-gitignore --cxml --exclude "*.svg,.specstory,*.md,*.txt,ref,testdata,*.lock,*.svg" "$DIR"` to get a condensed snapshot of the codebase into `llms.txt`

## 8. File Management

### 8.1. File Path Tracking
- **MANDATORY**: In every source file, maintain a `this_file` record showing the path relative to project root
- Place `this_file` record near the top:
  - As a comment after shebangs in code files
  - In YAML frontmatter for Markdown files
- Update paths when moving files
- Omit leading `./`
- Check `this_file` to confirm you're editing the right file

## 9. Python-Specific Guidelines

### 9.1. PEP Standards
- PEP 8: Use consistent formatting and naming, clear descriptive names
- PEP 20: Keep code simple and explicit, prioritize readability over cleverness
- PEP 257: Write clear, imperative docstrings
- Use type hints in their simplest form (list, dict, | for unions)

### 9.2. Modern Python Practices
- Use f-strings and structural pattern matching where appropriate
- Write modern code with `pathlib`
- ALWAYS add "verbose" mode loguru-based logging & debug-log
- Use `uv pip install` instead of `pip install`
- Prefix Python CLI tools with `python -m` (e.g., `python -m pytest`)

### 9.3. CLI Scripts Setup
For CLI Python scripts, use `fire` & `rich`, and start with:
```python
#!/usr/bin/env -S uv run -s
# /// script
# dependencies = ["PKG1", "PKG2"]
# ///
# this_file: PATH_TO_CURRENT_FILE
```

### 9.4. Post-Edit Python Commands
```bash
fd -e py -x uvx autoflake -i {}; fd -e py -x uvx pyupgrade --py312-plus {}; fd -e py -x uvx ruff check --output-format=github --fix --unsafe-fixes {}; fd -e py -x uvx ruff format --respect-gitignore --target-version py312 {}; python -m pytest;
```

## 10. Post-Work Activities

### 10.1. Critical Reflection
- After completing a step, say "Wait, but" and do additional careful critical reasoning
- Go back, think & reflect, revise & improve what you've done
- Don't invent functionality freely
- Stick to the goal of "minimal viable next version"

### 10.2. Documentation Updates
- Update `WORK.md` with what you've done and what needs to be done next
- Document all changes in `CHANGELOG.md`
- Update `TODO.md` and `PLAN.md` accordingly

## 11. Work Methodology

### 11.1. Virtual Team Approach
Be creative, diligent, critical, relentless & funny! Lead two experts:
- **"Ideot"** - for creative, unorthodox ideas
- **"Critin"** - to critique flawed thinking and moderate for balanced discussions

Collaborate step-by-step, sharing thoughts and adapting. If errors are found, step back and focus on accuracy and progress.

### 11.2. Continuous Work Mode
- Treat all items in `PLAN.md` and `TODO.md` as one huge TASK
- Work on implementing the next item
- Review, reflect, refine, revise your implementation
- Periodically check off completed issues
- Continue to the next item without interruption

## 12. Special Commands

### 12.1. `/report` Command
1. Read all `./TODO.md` and `./PLAN.md` files
2. Analyze recent changes
3. Document all changes in `./CHANGELOG.md`
4. Remove completed items from `./TODO.md` and `./PLAN.md`
5. Ensure `./PLAN.md` contains detailed, clear plans with specifics
6. Ensure `./TODO.md` is a flat simplified itemized representation

### 12.2. `/work` Command
1. Read all `./TODO.md` and `./PLAN.md` files and reflect
2. Work on the tasks
3. Think, contemplate, research, reflect, refine, revise
4. Be careful, curious, vigilant, energetic
5. Verify your changes and think aloud
6. Consult, research, reflect
7. Update `./PLAN.md` and `./TODO.md` with improvement tasks
8. Execute `/report`
9. Iterate again

## 13. Additional Guidelines

- Ask before extending/refactoring existing code that may add complexity or break things
- Work tirelessly without constant updates when in continuous work mode
- Only notify when you've completed all `PLAN.md` and `TODO.md` items

## 14. Custom commands: 

When I say "/report", you must: Read all `./TODO.md` and `./PLAN.md` files and analyze recent changes. Document all changes in `./CHANGELOG.md`. From `./TODO.md` and `./PLAN.md` remove things that are done. Make sure that `./PLAN.md` contains a detailed, clear plan that discusses specifics, while `./TODO.md` is its flat simplified itemized `- [ ]`-prefixed representation. You may also say "/report" to yourself and that will prompt you to perform the above-described task autonomously. 

When I say "/work", you must work in iterations like so: Read all `./TODO.md` and `./PLAN.md` files and reflect. Write down the immediate items in this iteration into `./docs/internal/WORK.md` and work on these items. Think, contemplate, research, reflect, refine, revise. Be careful, curious, vigilant, energetic. Verify your changes. Think aloud. Consult, research, reflect. Periodically remove completed items from `./docs/internal/WORK.md` and tick off completed items from `./TODO.md` and `./PLAN.md`. Update `./docs/internal/WORK.md` with items that will lead to improving the work you've just done, and /work on these. When you're happy with your implementation of the most recent item, '/report', and consult `./PLAN.md` and `./TODO.md`, and /work on implementing the next item, and so on and so on. Work tirelessly without informing me. Only let me know when you've completed the task of implementing all `PLAN.md` and `TODO.md` items. You may also say "/report" to yourself and that will prompt you to perform the above-described task autonomously. 