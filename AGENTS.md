
After every iteration, /report and mark completed items as done in @PLAN.md and @TODO.md. Then run `./build.sh` and then check the `./build_logs`. Then /work on items from @TODO.md consulting on @PLAN.md. Then review reflect refine revise, and then continue to /work on @PLAN.md and @TODO.md until every single item and issue has been fixed. Iterate iterate iterate! Do not stop, do not ask for confirmation. Work! When you're finishing one task or item, say "Wait, but..." and go on to the next task/item. It’s CRUCIAL that we get to a solution that BUILDS everything correctly!

## 1. Project Overview

`vexy_json` is a Rust port of the JavaScript library `jsonic`, a forgiving JSON parser. The project is officially named "Vexy JSON". The reference JavaScript implementation is located in the `ref/jsonic/` directory.

## 2. Development Status

This project is in an active development phase, focusing on post-migration cleanup and feature refinement. The core parsing engine is implemented, along with a comprehensive test suite, benchmarks, and WASM support. The current focus is on:

-   **Removing `jsonic` references**: Cleaning up legacy naming from 50 files.
-   **Fixing test failures**: Specifically, `test_number_features` due to unsupported number formats (octal, binary, underscore separators).
-   **Resolving build warnings**: Addressing 3 unused variable warnings in `examples/recursive_parser.rs`.
-   **Reducing compilation warnings**: Aiming to reduce the current 24 warnings.

The long-term focus remains on achieving full API compatibility with `jsonic`, refining the idiomatic Rust API, and improving performance, alongside planned architectural improvements, performance enhancements, and testing infrastructure upgrades.

## 3. Rust Implementation

### 3.1. Module Organization

The Rust implementation is a cargo workspace organized into several crates:

-   `crates/core`: The core parsing engine.
    -   `src/lib.rs`: The main library crate root, exporting the public API.
    -   `src/parser/`: Contains the core recursive descent parsing logic, with modules like `array.rs`, `boolean.rs`, `iterative.rs`, `null.rs`, `number.rs`, `object.rs`, `optimized.rs`, `optimized_v2.rs`, `recursive.rs`, `state.rs`, and `string.rs`.
    -   `src/lexer/`: The primary tokenizer for the input string, with `debug_lexer.rs`, `fast_lexer.rs`, and `logos_lexer.rs`.
    -   `src/ast/`: Defines the `Value` enum, which represents parsed JSON data, along with `builder.rs`, `mod.rs`, `token.rs`, `value.rs`, and `visitor.rs`.
    -   `src/error/`: Implements custom error types for parsing failures, including `mod.rs`, `ml_patterns.rs`, `recovery_v2.rs`, `repair.rs`, `reporter.rs`, `result.rs`, `span.rs`, `terminal.rs`, `types.rs`, `utils.rs`, and the `recovery` subdirectory.
    -   `src/lazy/`: Contains lazy parsing components for `array.rs`, `mod.rs`, `number.rs`, `object.rs`, and `string.rs`.
    -   `src/optimization/`: Includes `benchmarks.rs`, `memory_pool.rs`, `memory_pool_v2.rs`, `memory_pool_v3.rs`, `mod.rs`, `simd.rs`, `string_parser.rs`, `value_builder.rs`, and `zero_copy.rs`.
    -   `src/plugin/`: For plugin-related functionalities, including `mod.rs` and the `plugins` subdirectory.
    -   `src/repair/`: Contains `mod.rs` and `advanced.rs`.
    -   `src/streaming/`: Includes `buffered`, `event_parser.rs`, `lexer.rs`, `mod.rs`, `ndjson.rs`, and `simple_lexer.rs`.
    -   `src/transform/`: Contains `mod.rs`, `normalizer.rs` and `optimizer.rs`.
    -   `src/parallel.rs`: For parallel parsing.
    -   `src/parallel_chunked.rs`: For chunked parallel parsing.
    -   `src/repair.rs`: Another repair module.
    -   `crates/core/benches/parser_benchmarks.rs`: Benchmarks for the parser.
    -   `crates/core/examples/advanced_repair.rs`: Example for advanced repair.
    -   `crates/core/examples/error_reporting.rs`: Example for error reporting.
-   `crates/cli`: The command-line interface.
    -   `src/main.rs`: The entry point for the CLI binary.
-   `crates/c-api`: Provides C and C++ bindings, including `examples/`, `include/` (with `vexy_json.h` and `vexy_json.hpp`), and `src/lib.rs`.
-   `crates/python`: Provides Python bindings, including `python/vexy_json/__init__.py`, `src/lib.rs`, and `tests/`.
-   `crates/serde`: Provides `serde` integration for `vexy_json::Value`, with `src/lib.rs`.
-   `crates/wasm`: Contains WebAssembly bindings to expose `vexy_json` to JavaScript environments, including `src/lib.rs` and `test.mjs`.
-   `crates/test-utils`: Utility functions for testing, with `src/lib.rs`.

### 3.2. Core Features

-   **Standard JSON Parsing (RFC 8259):** Full support for the official JSON specification.
-   **Forgiving Features:** Compatibility with `jsonic`'s non-standard features is a primary goal:
    -   Single-line (`//`) and multi-line (`/* */`) comments.
    -   Trailing commas in objects and arrays.
    -   Unquoted object keys (where unambiguous).
    -   Implicit top-level objects and arrays.
    -   Single-quoted strings.
    -   Newline characters as comma separators.

### 3.3. Architecture & Best Practices

-   **Error Handling:** Uses `Result<T, E>` and a custom `Error` enum (`src/error.rs`) for robust error handling with location information.
-   **Testing:**
    -   Unit and integration tests are located in the `tests/` directory, covering various aspects like `advanced_features.rs`, `basic_tests.rs`, `comma_handling.rs`, `comment_handling.rs`, `compat_tests.rs`, `comprehensive_tests.rs`, `error_handling.rs`, `feature_tests.rs`, `forgiving_features.rs`, `lexer_tests.rs`, `lib_integration.rs`, `newline_as_comma.rs`, `number_formats.rs`, `property_tests.rs`, `real_world_scenarios.rs`, and `string_handling.rs`. Many of these are ported from `jsonic`'s test suite.
    -   The `examples/` directory contains numerous small, runnable programs for debugging specific features, such as `debug_comma_one.rs`, `debug_comment_tokens.rs`, `recursive_parser.rs`, and `test_number_types.rs`.
    -   Benchmarking is performed using `criterion.rs`, with benchmarks defined in the `benches/` directory, including `benchmark.rs`, `comparison.rs`, `comprehensive_comparison.rs`, `lexer_microbenchmarks.rs`, `memory_benchmarks.rs`, `parser_comparison.rs`, `parser_microbenchmarks.rs`, `parsing.rs`, `performance_comparison.rs`, `profiling.rs`, `real_world_benchmarks.rs`, `simd_benchmarks.rs`, and `stack_overflow_test.rs`.
    -   Property-based tests are implemented using `proptest` in `tests/property_tests.rs`.
-   **Extensibility:** The architecture uses Rust's traits and pattern matching for clarity and maintainability, avoiding a direct port of the JavaScript plugin system in favor of a more idiomatic approach.
-   **Performance:** The implementation aims for high performance, with ongoing benchmarking to compare against `serde_json`.
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

### 4.2. Reference Implementation (jsonic)

When working with the reference JavaScript implementation in `ref/jsonic/`:

```bash
cd ref/jsonic

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
- Read `docs/internal/WORK.md` for work progress
- Read `README.md` to understand the project
- STEP BACK and THINK HEAVILY STEP BY STEP about the task
- Consider alternatives and carefully choose the best option
- Check for existing solutions in the codebase before starting

### 5.2. Project Documentation to Maintain
- `README.md` - purpose and functionality
- `CHANGELOG.md` - past change release notes (accumulative)
- `PLAN.md` - detailed future goals, clear plan that discusses specifics
- `TODO.md` - flat simplified itemized `- [ ]`-prefixed representation of `PLAN.md`
- `WORK.md` - work progress updates

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
- Update `TODO.md` and `docs/internal/PLAN.md` accordingly

## 11. Work Methodology

### 11.1. Virtual Team Approach
Be creative, diligent, critical, relentless & funny! Lead two experts:
- **"Ideot"** - for creative, unorthodox ideas
- **"Critin"** - to critique flawed thinking and moderate for balanced discussions

Collaborate step-by-step, sharing thoughts and adapting. If errors are found, step back and focus on accuracy and progress.

### 11.2. Continuous Work Mode
- Treat all items in `docs/internal/PLAN.md` and `TODO.md` as one huge TASK
- Work on implementing the next item
- Review, reflect, refine, revise your implementation
- Periodically check off completed issues
- Continue to the next item without interruption

## 12. Special Commands

### 12.1. `/report` Command
1. Read all `./TODO.md` and `./docs/internal/PLAN.md` files
2. Analyze recent changes
3. Document all changes in `./CHANGELOG.md`
4. Remove completed items from `./TODO.md` and `./docs/internal/PLAN.md`
5. Ensure `./docs/internal/PLAN.md` contains detailed, clear plans with specifics
6. Ensure `./TODO.md` is a flat simplified itemized representation

### 12.2. `/work` Command
1. Read all `./TODO.md` and `./docs/internal/PLAN.md` files and reflect
2. Work on the tasks
3. Think, contemplate, research, reflect, refine, revise
4. Be careful, curious, vigilant, energetic
5. Verify your changes and think aloud
6. Consult, research, reflect
7. Update `./docs/internal/PLAN.md` and `./TODO.md` with improvement tasks
8. Execute `/report`
9. Iterate again

## 13. Additional Guidelines

- Ask before extending/refactoring existing code that may add complexity or break things
- Work tirelessly without constant updates when in continuous work mode
- Only notify when you've completed all `docs/internal/PLAN.md` and `TODO.md` items

## 14. Custom commands: 

When I say "/report", you must: Read all `./TODO.md` and `./PLAN.md` files and analyze recent changes. Document all changes in `./CHANGELOG.md`. From `./TODO.md` and `./PLAN.md` remove things that are done. Make sure that `./PLAN.md` contains a detailed, clear plan that discusses specifics, while `./TODO.md` is its flat simplified itemized `- [ ]`-prefixed representation. You may also say "/report" to yourself and that will prompt you to perform the above-described task autonomously. 

When I say "/work", you must work in iterations like so: Read all `./TODO.md` and `./PLAN.md` files and reflect. Write down the immediate items in this iteration into `./WORK.md` and work on these items. Think, contemplate, research, reflect, refine, revise. Be careful, curious, vigilant, energetic. Verify your changes. Think aloud. Consult, research, reflect. Periodically remove completed items from `./WORK.md` and tick off completed items from `./TODO.md` and `./PLAN.md`. Update `./WORK.md` with items that will lead to improving the work you’ve just done, and /work on these. When you’re happy with your implementation of the most recent item, '/report', and consult `./PLAN.md` and `./TODO.md`, and /work on implementing the next item, and so on and so on. Work tirelessly without informing me. Only let me know when you’ve completed the task of implementing all `./PLAN.md` and `./TODO.md` items. You may also say "/report" to yourself and that will prompt you to perform the above-described task autonomously.

### 14.1. Development Workflow

This project uses a specific workflow for development and testing. Adhere to the following commands.

### 14.2. Build and Test

**DO NOT** run `cargo build`, `cargo test`, or `cargo clippy` directly. Instead, use the provided build script, which handles all necessary steps, including formatting, linting, building, and testing.

```bash
./build.sh
```

After running the script, always review the output log to check for errors or warnings:

```bash
cat ./build.log.txt
```

### 14.3. Reference Implementation (jsonic)

When working with the reference JavaScript implementation in `ref/jsonic/`:

```bash
cd ref/jsonic

# Build the TypeScript code
npm run build

# Run all tests
npm test

# Run specific tests
npm run test-some -- <test-pattern>
```


---

# Consolidated Software Development Rules

## 15. Pre-Work Preparation

### 15.1. Before Starting Any Work
- Read `docs/internal/WORK.md` for work progress
- Read `README.md` to understand the project
- STEP BACK and THINK HEAVILY STEP BY STEP about the task
- Consider alternatives and carefully choose the best option
- Check for existing solutions in the codebase before starting

### 15.2. Project Documentation to Maintain
- `README.md` - purpose and functionality
- `CHANGELOG.md` - past change release notes (accumulative)
- `PLAN.md` - detailed future goals, clear plan that discusses specifics
- `TODO.md` - flat simplified itemized `- [ ]`-prefixed representation of `PLAN.md`
- `WORK.md` - work progress updates

## 16. General Coding Principles

### 16.1. Core Development Approach
- Iterate gradually, avoiding major changes
- Focus on minimal viable increments and ship early
- Minimize confirmations and checks
- Preserve existing code/structure unless necessary
- Check often the coherence of the code you're writing with the rest of the code
- Analyze code line-by-line

### 16.2. Code Quality Standards
- Use constants over magic numbers
- Write explanatory docstrings/comments that explain what and WHY
- Explain where and how the code is used/referred to elsewhere
- Handle failures gracefully with retries, fallbacks, user guidance
- Address edge cases, validate assumptions, catch errors early
- Let the computer do the work, minimize user decisions
- Reduce cognitive load, beautify code
- Modularize repeated logic into concise, single-purpose functions
- Favor flat over nested structures

## 17. Tool Usage (When Available)

### 17.1. MCP Tools to Consult
- `codex` tool - for additional reasoning, summarization of files and second opinion
- `context7` tool - for most up-to-date software package documentation
- `sequentialthinking` tool - to think about the best way to solve tasks
- `perplexity_ask` - for up-to-date information or context

### 17.2. Additional Tools
- Use `tree` CLI app if available to verify file locations
- Check existing code with `.venv` folder to scan and consult dependency source code
- Run `DIR="."; uvx codetoprompt --compress --output "$DIR/llms.txt"  --respect-gitignore --cxml --exclude "*.svg,.specstory,*.md,*.txt,ref,testdata,*.lock,*.svg" "$DIR"` to get a condensed snapshot of the codebase into `llms.txt`

## 18. File Management

### 18.1. File Path Tracking
- **MANDATORY**: In every source file, maintain a `this_file` record showing the path relative to project root
- Place `this_file` record near the top:
  - As a comment after shebangs in code files
  - In YAML frontmatter for Markdown files
- Update paths when moving files
- Omit leading `./`
- Check `this_file` to confirm you're editing the right file

## 19. Python-Specific Guidelines

### 19.1. PEP Standards
- PEP 8: Use consistent formatting and naming, clear descriptive names
- PEP 20: Keep code simple and explicit, prioritize readability over cleverness
- PEP 257: Write clear, imperative docstrings
- Use type hints in their simplest form (list, dict, | for unions)

### 19.2. Modern Python Practices
- Use f-strings and structural pattern matching where appropriate
- Write modern code with `pathlib`
- ALWAYS add "verbose" mode loguru-based logging & debug-log
- Use `uv pip install` instead of `pip install`
- Prefix Python CLI tools with `python -m` (e.g., `python -m pytest`)

### 19.3. CLI Scripts Setup
For CLI Python scripts, use `fire` & `rich`, and start with:
```python
#!/usr/bin/env -S uv run -s
# /// script
# dependencies = ["PKG1", "PKG2"]
# ///
# this_file: PATH_TO_CURRENT_FILE
```

### 19.4. Post-Edit Python Commands
```bash
fd -e py -x uvx autoflake -i {}; fd -e py -x uvx pyupgrade --py312-plus {}; fd -e py -x uvx ruff check --output-format=github --fix --unsafe-fixes {}; fd -e py -x uvx ruff format --respect-gitignore --target-version py312 {}; python -m pytest;
```

## 20. Post-Work Activities

### 20.1. Critical Reflection
- After completing a step, say "Wait, but" and do additional careful critical reasoning
- Go back, think & reflect, revise & improve what you've done
- Don't invent functionality freely
- Stick to the goal of "minimal viable next version"

### 20.2. Documentation Updates
- Update `WORK.md` with what you've done and what needs to be done next
- Document all changes in `CHANGELOG.md`
- Update `TODO.md` and `docs/internal/PLAN.md` accordingly

## 21. Work Methodology

### 21.1. Virtual Team Approach
Be creative, diligent, critical, relentless & funny! Lead two experts:
- **"Ideot"** - for creative, unorthodox ideas
- **"Critin"** - to critique flawed thinking and moderate for balanced discussions

Collaborate step-by-step, sharing thoughts and adapting. If errors are found, step back and focus on accuracy and progress.

### 21.2. Continuous Work Mode
- Treat all items in `docs/internal/PLAN.md` and `TODO.md` as one huge TASK
- Work on implementing the next item
- Review, reflect, refine, revise your implementation
- Periodically check off completed issues
- Continue to the next item without interruption

## 22. Special Commands

### 22.1. `/report` Command
1. Read all `./TODO.md` and `./docs/internal/PLAN.md` files
2. Analyze recent changes
3. Document all changes in `./CHANGELOG.md`
4. Remove completed items from `./TODO.md` and `./docs/internal/PLAN.md`
5. Ensure `./docs/internal/PLAN.md` contains detailed, clear plans with specifics
6. Ensure `./TODO.md` is a flat simplified itemized representation

### 22.2. `/work` Command
1. Read all `./TODO.md` and `./docs/internal/PLAN.md` files and reflect
2. Work on the tasks
3. Think, contemplate, research, reflect, refine, revise
4. Be careful, curious, vigilant, energetic
5. Verify your changes and think aloud
6. Consult, research, reflect
7. Update `./docs/internal/PLAN.md` and `./TODO.md` with improvement tasks
8. Execute `/report`
9. Iterate again

## 23. Additional Guidelines

- Ask before extending/refactoring existing code that may add complexity or break things
- Work tirelessly without constant updates when in continuous work mode
- Only notify when you've completed all `docs/internal/PLAN.md` and `TODO.md` items

## 24. Custom commands: 

When I say "/report", you must: Read all `./TODO.md` and `./docs/internal/PLAN.md` files and analyze recent changes. Document all changes in `./CHANGELOG.md`. From `./TODO.md` and `./docs/internal/PLAN.md` remove things that are done. Make sure that `./PLAN.md` contains a detailed, clear plan that discusses specifics, while `./TODO.md` is its flat simplified itemized `- [ ]`-prefixed representation. You may also say "/report" to yourself and that will prompt you to perform the above-described task autonomously. 

When I say "/work", you must work in iterations like so: Read all `./TODO.md` and `./docs/internal/PLAN.md` files and reflect. Write down the immediate items in this iteration into `./docs/internal/WORK.md` and work on these items. Think, contemplate, research, reflect, refine, revise. Be careful, curious, vigilant, energetic. Verify your changes. Think aloud. Consult, research, reflect. Periodically remove completed items from `./docs/internal/WORK.md` and tick off completed items from `./TODO.md` and `./docs/internal/PLAN.md`. Update `./docs/internal/WORK.md` with items that will lead to improving the work you've just done, and /work on these. When you're happy with your implementation of the most recent item, '/report', and consult `./docs/internal/PLAN.md` and `./TODO.md`, and /work on implementing the next item, and so on and so on. Work tirelessly without informing me. Only let me know when you've completed the task of implementing all `./docs/internal/PLAN.M` and `./TODO.md` items. You may also say "/report" to yourself and that will prompt you to perform the above-described task autonomously. 