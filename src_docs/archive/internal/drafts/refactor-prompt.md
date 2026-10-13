Read @llms.txt which contains the snapshot of the entire codebase.

Analyze the entire #codebase 

Update REFACTOR.md so that it becomes a very detailed plan of refactoring the code, under the following principles:


1. **Production-grade Quality** – Aim for clean, idiomatic, _boring_ Rust. No clever macros where straightforward code is clearer.
2. **Parity With Reference Implementation** – Behaviour must remain 100 % compatible with the original JavaScript `the reference implementation` test-suite unless a conscious deviation is documented.
3. **Incremental, Review-friendly Commits** – Small, atomic commits that each compile and keep the test-suite green.
4. **Minimal Public-API Breakage** – The current crate is already used in downstream code and WASM builds; any unavoidable breaking change must be sign-posted in the CHANGELOG and guarded by a semver bump.
5. **Performance Awareness** – Never regress the existing Criterion benchmarks by more than 3 % unless the change gives a functional or maintainability win that clearly outweighs the cost.
6. **Great DX** – Improve docs, examples and error messages as you touch code; run `./build.sh` locally before pushing.
7. **Security & Safety First** – Eliminate `unsafe` (currently none), check for `TODO: unwrap` / `expect`, replace with fallible code paths.

The refactor will be delivered as a _series of pull-requests_ structured around themes so that reviewers can digest them easily.

Below is a **detailed, step-by-step playbook** you – the engineer – should follow. Feel free to adjust the ordering if downstream work uncovers hidden coupling, but _always_ keep commits small and the repo green.

---

## 1. On-boarding (½ day)

- Clone the repo, run `./build.sh`, open `./build.log.txt` – ensure you start from a clean, reproducible state.
- Scan `docs/internal/CLAUDE.md`, `IMPLEMENTATION_SUMMARY.md`, `PLAN.md` to understand design intent.
- Run the benchmarks (`cargo bench --bench parsing`) and note baseline numbers in a personal scratchpad.
- Create a new branch `refactor/phase-1-module-layout` for the first PR.

## 2. Restructure the Module Tree (1 day)

Goal: make the crate’s public surface and internal structure obvious at a glance.

1.1 **Move binaries into `src/bin/`**  
 Currently we have `main.rs` and `bin/harness.rs`; place both under `src/bin/` and use descriptive names (`cli.rs`, `harness.rs`). Adjust Cargo manifest `[bin]` sections accordingly.

1.2 **Introduce `src/ast/`**  
 Create a dedicated module for the concrete syntax tree (tokens) and abstract syntax tree (Value) to localise parsing artefacts. File split suggestion:

- `src/ast/mod.rs` – re-exports
- `src/ast/token.rs` – existing `Token` enum + helper impls
- `src/ast/value.rs` – existing `Value`, `Number`, conversions, feature-gated `serde`

  1.3 **Isolate Error Handling**  
   Move `error.rs` into `src/error/mod.rs`; create sub-modules:

- `kind.rs` – the `Error` enum
- `position.rs` – a lightweight `Span { start: usize, end: usize }`

  1.4 **Public API Barrel File**  
   `lib.rs` should become a concise _index_ that re-exports public types; the heavy doc-comment with README inclusion can move to `docs/api.md`.

Deliverables: new folder structure, imports updated, tests & benchmarks still pass.

## 3. Simplify the Lexer (2-3 days)

The current lexer contains duplicated state machines and ad-hoc look-ahead logic. Steps:

2.1 **Extract Config** – Config flags like `allow_single_quotes` belong in `ParserOptions` only; remove duplication from lexer. The lexer should tokenise _regardless_ of permissiveness; the parser decides if a token is legal in context.

2.2 **Use `logos`** – Evaluate replacing the handwritten lexer with the `logos` crate (MIT licensed, no runtime deps). Benchmark; accept if equal or faster and code is clearer.

2.3 **Remove `lexer2.rs`** – It’s an experiment that has diverged; either promote it (if chosen) or delete.

2.4 **Canonical Token Stream** – Ensure every character of input maps to exactly one token stream position; add invariant tests (property test with `quickcheck`) that `iter::sum(token.len()) == input.len()` apart from whitespace.

## 4. Parser Clean-up (3 days)

3.1 **Introduce `ParserState` struct** instead of many boolean fields to group stateful data (`depth`, `lexer_offset`, etc.).

3.2 **Tail-recursion removal** – Replace deep recursion on arrays/objects with an explicit stack to honour `max_depth` without risking stack overflow.

3.3 **Improve Error Reporting** – Switch from raw `usize` positions to the `Span` type; implement `fmt::Display` to highlight offending slice with a caret.

3.4 **Config Validation** – Add `ParserOptions::validate()` that returns `Result<(), ConfigError>`; e.g. `newline_as_comma=false` + `implicit_top_level=true` is ambiguously specified – decide policy and enforce.

3.5 **Property-based tests** – Port `the reference implementation` round-trip tests; generate random forgiving JSON, parse, serialise back to canonical JSON, compare using serde_json Value.

## 5. Error & Result Type Revamp (1 day)

- Implement the `thiserror` crate for boilerplate.
- Provide an `Error::source()` chain so WASM callers can access root cause.
- Export a `type ParseResult<T = Value> = core::result::Result<T, Error>` alias.

## 6. WASM Bindings Overhaul (½ day)

- Re-generate with `wasm-bindgen` 0.2.latest; enable `weak-refs` for memory leaks fix.
- Expose `parse_with_options(json, options)` where `options` is a JS object; derive `serde_wasm_bindgen` for bridging.

## 7. Benchmark & CI Pipeline (1 day)

- Move Criterion benches under `benches/` root, use `cargo bench --workspace`.
- GitHub Actions matrix: `stable`, `beta`, `nightly`, plus `wasm32-unknown-unknown` build.
- Add `cargo udeps` and `cargo deny` checks.

## 8. Documentation Pass (1½ days)

- Update code comments to **explain why** not just what.
- Auto-generate docs via `cargo doc --workspace --no-deps` in CI; deploy to `gh-pages`.
- Write a migration guide if any `pub` items are renamed.

## 9. Release Planning (½ day)

- Bump version to `0.2.0` following semver since internal layout changed.
- Update `CHANGELOG.md` with highlights: _module re-org_, _logos lexer_, _better error messages_.

---

### 9.1. Deliverable Checklist per PR

1. `./build.sh` green locally.
2. All tests & benches pass on CI.
3. Coverage ≥ 90 % for touched code (grcov).
4. Added / updated docs where public API changed.
5. CHANGELOG entry under _Unreleased_.

---

## 10. Nice-to-have Stretch Goals (do **not** block v0.2.0)

- Plug a _streaming serializer_ to avoid building intermediate `Value`s for large input.
- Explore `simd-utf8` for lexing speed-ups.
- Accept `Cow<str>` input to allow zero-copy parse in some contexts.

---

### 10.1. Final Notes

_Treat the refactor as paving the road for long-term maintainability rather than chasing micro-optimisations._ When in doubt choose readability – but back it up with benchmark data.
