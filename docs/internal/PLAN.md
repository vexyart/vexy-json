# Vexy JSON Build & Release Fix Plan

## Summary
The build and release process is failing due to:
1. **Clippy warnings** (143 errors) that need to be fixed
2. **20 failing unit tests** in the core library
3. **Release script timeout** after 20 minutes

## Analysis of Issues

### 1. Clippy Warnings (High Priority)
The build is failing with 143 clippy errors. Key issues:

#### a. Formatting Issues
- `uninlined_format_args`: Variables can be used directly in format strings
- Files affected: `ast/visitor.rs`, `parallel.rs`

#### b. Code Quality Issues  
- `iter_kv_map`: Using `.iter()` then `.map()` on HashMap when `.keys()` would be cleaner
- `unnecessary_map_or`: Using `map_or(false, |x| ...)` when `is_some_and()` is clearer
- `while_let_on_iterator`: Using `while let Some(x) = iter.next()` instead of `for x in iter`
- `should_implement_trait`: Method named `default()` should implement Default trait
- `type_complexity`: Very complex types that should be factored into type definitions

#### c. Pattern Issues
- `manual_strip`: Manually stripping prefixes instead of using `strip_prefix()`
- `redundant_pattern_matching`: Using `if let Ok(_) = ...` instead of `.is_ok()`
- `collapsible_if`: Nested if statements that can be collapsed
- `redundant_closure`: Using closures that just call a function
- `new_without_default`: `new()` methods that should implement Default
- `let_and_return`: Unnecessary let bindings before return
- `unused_enumerate_index`: Using `.enumerate()` but not using the index

### 2. Failing Unit Tests (Critical)
20 tests are failing across multiple modules:

#### Parser Tests (7 failures)
- `parser::iterative` module has multiple failures in array/object parsing
- `parser::optimized_v2` has memory stats assertion failure

#### Streaming Tests (5 failures)  
- `streaming::event_parser` - incomplete JSON handling
- `streaming::ndjson` - line counting and parsing issues

#### Lazy Parsing Tests (4 failures)
- `lazy` module - unwrap on Err values, threshold issues

#### Other Test Failures (4 failures)
- `error::recovery_v2::test_bracket_matching` - assertion failure
- `lexer` tests - stats and error logging issues  
- `optimization::memory_pool_v2` - allocation stats
- `parallel_chunked` - NDJSON parsing
- `plugin::datetime` - custom format test

### 3. Release Script Issues
The release script times out after 20 minutes, likely due to the failing tests and compilation errors.

## Fix Strategy

### Phase 1: Fix Clippy Warnings (Blockers)
1. **Format string fixes** - Update all format! and write! macros to use inline variables
2. **Iterator improvements** - Replace iter().map() with keys(), use for loops instead of while let
3. **Pattern matching** - Use strip_prefix(), is_ok(), is_some_and() methods
4. **Trait implementations** - Add Default trait where needed
5. **Type simplification** - Create type aliases for complex types
6. **Code cleanup** - Remove unnecessary closures, let bindings, enumerate indices

### Phase 2: Fix Failing Tests
1. **Iterative parser** - Debug array/object parsing logic, fix comma/bracket expectations
2. **Streaming parsers** - Handle incomplete JSON properly, fix NDJSON line counting
3. **Lazy parser** - Add proper error handling instead of unwrap()
4. **Memory pools** - Fix allocation tracking and stats
5. **Recovery engine** - Fix bracket matching logic
6. **Lexer stats** - Update token counting logic

### Phase 3: Verify & Release
1. Run `./build.sh` to ensure all warnings are fixed
2. Run full test suite to ensure all tests pass
3. Re-run release script with proper versioning

## Implementation Order
1. Fix all clippy warnings first (they block compilation)
2. Fix failing tests module by module
3. Verify build and release process

## Estimated Time
- Clippy fixes: 1-2 hours
- Test fixes: 2-3 hours  
- Verification: 30 minutes