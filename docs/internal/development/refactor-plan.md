---
layout: page
title: Refactor Plan
permalink: /development/refactor-plan/
parent: Development
nav_order: 1
---

# REFACTOR.md – Authoring Brief (Revised for Lean & Refactor Principles)

This document is the canonical, **action-oriented**, **self-contained**, and **phased** roadmap for the vexy_json refactor sprint. It integrates the detailed refactor playbook and quality principles from [`REFACTOR_PROMPT.md`](REFACTOR_PROMPT.md) and the minimalization/dead code removal guidance from [`LEAN.md`](LEAN.md). It is written for a technically strong engineer new to this repository.

---

## 1. Executive Summary

The vexy_json codebase is a monolithic Rust crate implementing a forgiving JSON parser, CLI, and WASM module. Its tightly coupled structure, legacy/dead code, and lack of clear boundaries hinder maintainability, performance, and extensibility. This refactor will:

- Decouple components into a Cargo workspace of focused crates.
- Remove dead/legacy code and minimize dependencies.
- Feature-gate optional components (WASM, Serde, CLI).
- Enforce production-grade, review-friendly, and performance-aware practices.
- Improve documentation, developer experience, and CI/CD quality gates.

Upon completion, vexy_json will be a lean, maintainable, and extensible parser suite, with robust testing, clear architecture, and a minimal core suitable for embedding or distribution.

---

## 2. Guiding Principles

### 2.1. Production-grade Quality & Lean Minimalism

- Write clean, idiomatic, boring Rust. Avoid clever macros.
- Remove all dead/legacy code (see Section 4).
- Minimize dependencies; only use well-audited crates.
- Feature-gate all optional functionality (WASM, Serde, CLI).
- No public API breakage unless unavoidable and documented.

### 2.2. Parity With Reference Implementation

- Maintain 100% compatibility with the JavaScript `the reference implementation` test suite unless deviations are documented.

### 2.3. Incremental, Review-friendly Commits

- Refactor in small, atomic, test-passing commits.
- Each PR must be reviewable, CI-green, and benchmarked.

### 2.4. Minimal Public-API Breakage

- Downstream code and WASM builds must not break.
- Breaking changes require CHANGELOG entries and semver bumps.

### 2.5. Performance Awareness

- No >3% regression on Criterion benchmarks unless justified.
- Document and benchmark all performance-impacting changes.

### 2.6. Great DX

- Improve docs, examples, and error messages as code is touched.
- Run `./build.sh` locally before pushing.

### 2.7. Security & Safety First

- Eliminate all `unsafe` code.
- Remove all `unwrap`/`expect` unless justified and documented.

---

## 3. Architectural Re-design

### 3.1. Workspace Structure

Refactor into a Cargo workspace with these crates:

- **vexy_json-core**: Core parser, lexer, value types, errors. No I/O, CLI, or WASM logic.
- **vexy_json-cli**: CLI wrapper, feature-gated.
- **vexy_json-wasm**: WASM bindings, feature-gated.
- **vexy_json-serde**: Serde integration, feature-gated.
- **test-utils**: Shared test helpers.
- **examples/**, **benches/**: Kept for development, excluded from lean/core builds.

### 3.2. Minimal Core

The minimal, embeddable crate consists of only:

- `src/lib.rs`
- `src/parser.rs`
- `src/lexer.rs`
- `src/value.rs`
- `src/error.rs`

All other files are optional, feature-gated, or excluded from minimal builds.

---


## 4. Refactor Playbook (Phased Steps)

### 4.1. Phase 1: On-boarding & Baseline

- Clone repo, run `./build.sh`, ensure reproducible build.
- Review `docs/internal/CLAUDE.md`, `IMPLEMENTATION_SUMMARY.md`, `PLAN.md`.
- Run and record baseline benchmarks.
- Create `refactor/phase-1-module-layout` branch.


### 4.2. Phase 4: Lexer Simplification

- Remove config duplication; config only in parser. (Completed)
- Evaluate `logos` crate for lexer; benchmark and adopt if beneficial. (Completed)
- Ensure canonical token stream; add property tests. (Completed)

### 4.3. Phase 5: Parser Refactor

- Introduce `ParserState` struct. (Completed)
- Remove tail recursion; use explicit stack. (Completed - addressed by `max_depth` in `ParserOptions`)
- Improve error reporting with `Span`.
- Add config validation.
- Add property-based round-trip tests.

### 4.4. Phase 6: Error & Result Type Revamp

- Use `thiserror` for error enums.
- Provide error source chains.
- Export `ParseResult<T = Value>` alias.

### 4.5. Phase 7: WASM & Serde Bindings

- Regenerate WASM with latest `wasm-bindgen`.
- Expose JS-friendly API.
- Feature-gate all bindings.

### 4.6. Phase 8: Benchmark & CI Pipeline

- Move benches to `benches/` root.
- Add CI matrix for Rust toolchains and WASM.
- Add `cargo udeps` and `cargo deny` checks.

### 4.7. Phase 9: Documentation & DX

- Update code comments to explain "why".
- Auto-generate docs in CI; deploy to GitHub Pages.
- Write migration guide if any `pub` items are renamed.

### 4.8. Phase 10: Release Planning

- Bump version to `0.2.0` (semver).
- Update `CHANGELOG.md` with highlights.

---

## 5. Technical Debt Catalogue & Fix Plan

| ID  | File / Module         | Issue / Impact / Fix (summary)      | Effort |
|-----|----------------------|-------------------------------------|--------|
| P0  | `src/parser.rs`      | Monolithic, complex logic. Rewrite as Pratt/recursive descent parser. | L      |
| P0  | `src/main.rs:95`     | Custom JSON formatter. Use `serde_json`. | S      |
| P1  | `src/parser.rs:313`  | Parser calculates token positions. Lexer should emit spans. | M      |
| P1  | `src/main.rs:45`     | CLI pre-processes input. Move logic to lexer. | S      |
| P1  | everywhere           | Inconsistent error handling. Eliminate `Error::Custom`. | M      |
| P2  | `tests/`             | Lack of property-based testing. Add `proptest`. | M      |
| P2  | `src/lib.rs`         | Tests inside lib. Move to `tests/`. | S      |

---

## 6. Testing & Quality Gates

- **Coverage Baseline:** Measure with `cargo-tarpaulin`.
- **Target Coverage:** `vexy_json-core` ≥95%, CLI ≥80%, WASM ≥90%.
- **Testing Pyramid:** Unit, integration, property-based, and performance tests.
- **CI Workflow:** Format, lint, test, coverage, bench, build artifacts.
- **Deliverable Checklist per PR:**
  1. `./build.sh` green locally.
  2. All tests & benches pass on CI.
  3. Coverage ≥90% for touched code.
  4. Docs updated for public API changes.
  5. CHANGELOG entry under _Unreleased_.

---

## 7. Migration Strategy

- Create `refactor/workspace` branch.
- Convert to Cargo workspace; create new crate structure.
- Migrate core files first; re-export from old crate for compatibility.
- Add `--refactor-parser` CLI flag for dual-track testing.
- Run CI on both old and new implementations until cut-over.
- Tag before each major step for rollback.

---

## 8. Performance Targets

- **Parsing Throughput:** 10MB in <100ms (release build).
- **Performance Parity:** Within 3% of old parser, within 10% of `serde_json`.
- **WASM:** 1MB in <50ms in browser.
- Use `cargo-flamegraph` and `pprof` for profiling.

---

## 9. Documentation & DX

- API docs auto-generated and deployed.
- Examples for CLI, core, WASM.
- Updated README with badges.
- CONTRIBUTING.md with workflow, style, PR checklist.

---

## 10. Timeline & Milestones

| Week  | Deliverable                                 | Success Metric                                 |
|-------|---------------------------------------------|------------------------------------------------|
| 1-2   | Workspace setup & `vexy_json-core` created      | CI green, core builds, dead code removed.      |
| 3-4   | Lexer refactored, emits spans               | Token struct has span, parser updated.         |
| 5-8   | New parser implemented                      | Property tests pass.                           |
| 9-10  | CLI/WASM migrated to new parser             | All integration tests pass.                    |
| 11    | Old parser removed, final cleanup           | No breaking changes in public API.             |
| 12    | Docs updated, refactor branch merged        | Branch merged to main.                         |

---

## 11. Acceptance Criteria

- All CLI flags and behaviors preserved.
- Public Rust API is identical or a superset, verified with `cargo-public-api diff`.
- WASM bundle size ≤300KB gzipped.
- CI pipeline completes in <12 minutes.
- Test coverage for core ≥95%.
- No performance regressions on benchmarks.
- Only minimal, essential files in core crate.

---

## 12. Open Questions & Assumptions

| Question                                                          | Owner       | Due Date   |
|-------------------------------------------------------------------|-------------|------------|
| What is the Minimum Supported Rust Version (MSRV) for this project?| @engineer-1 | Week 1     |
| Are there any clients depending on the exact error messages?       | @product    | Week 1     |
| What is the long-term support plan for JSON-C style comments (`#`)?| @product    | Week 2     |

---

## 13. Final Notes

Treat this refactor as paving the road for long-term maintainability and minimalism, not chasing micro-optimizations. When in doubt, choose readability and simplicity, but back up decisions with benchmark data. Periodically re-audit for dead/unused modules and update this plan as new feature gates or refactors are added.