---
nav_title: Lean Minimalization
nav_order: 2
---

# LEAN.md

## vexy_json: Definitive Lean/Minimalization Checklist & Rationale

This actionable document is for reducing the vexy_json codebase to the absolutely minimal, efficient, and dependency-free parser crate, suitable for distribution or embedding.

---
### SECTION 1 — **REMOVE ENTIRELY / DEAD CODE**

These files are **unused or legacy** and can be deleted with no impact to correctness or API:

- `src/lexer2.rs` — Verified as unused code via `grep` and `search_files` tool. Remove immediately.

### KEEP but ensure that these are clearly marked 

- `examples/` directory: Contains various debug and test examples. These are not part of the core library and can be removed for a lean distribution.
- `benches/` directory: Contains benchmarking code. Not essential for the core library. Remove for a lean distribution.
- `docs/pkg/` directory: Contains WASM build output and related files. These are build artifacts and should not be part of a minimal source distribution.
- `scripts/` directory: Contains build and test scripts. These are development utilities and not part of the core library.
- `target/` directory: Contains build output and temporary files. Not part of the source distribution.


---
### SECTION 2 — **OPTIONAL via FEATURE-GATE/SECONDARY**

Keep behind a feature-flag:

- `src/wasm.rs` — WASM/Web export only. Feature-gated as "wasm" in `Cargo.toml`.
- `src/serde_impl.rs` — Serde interop only. Feature-gated as "serde" in `Cargo.toml`.
- `src/main.rs` — CLI entry point. Feature-gated as "cli" in `Cargo.toml`.
- `src/bin/harness.rs` — A binary harness, not part of the core library. Can be removed for a pure library/embedding.

---
### SECTION 3 — **KEEP: ABSOLUTELY ESSENTIAL**

The following files are always required for the core crate:

- `src/lib.rs` — *Entrypoint and API.*
- `src/parser.rs` — *Parser logic (references only `src/lexer.rs`).*
- `src/lexer.rs` — *Lexical analyzer (the **only** live lexer, used in API/tests/benches).* 
- `src/value.rs` — *Result and value types. Merge with lib.rs for amalgam builds only.*
- `src/error.rs` — *Error/result types.*

---
### SECTION 4 — **TESTS**

- Retain `tests/` for development and CI. *Exclude from binary/dist releases.*

---
### SUMMARY CHECKLIST

- [x] Remove: `src/lexer2.rs` (Done)
- [ ] KEEP `examples/`, `benches/`, `docs/pkg/`, `scripts/`, `target/` directories. (Conceptual: These are excluded from lean distribution by build process, not by deletion)
- [x] Confirm `src/lexer2.rs` is deleted. (Confirmed by command output)
- [x] Ensure `src/bin/harness.rs` is removed or feature-gated. (Removed)
- [x] Feature-gate: `src/wasm.rs`, `src/serde_impl.rs`, `src/main.rs`. (`src/main.rs` feature-gated via `Cargo.toml`, `src/wasm.rs` and `src/serde_impl.rs` already feature-gated as confirmed by file content)
- [ ] Keep only: `src/lib.rs`, `src/parser.rs`, `src/lexer.rs`, `src/value.rs`, `src/error.rs`. (Confirmed, no action needed)
- [ ] Exclude tests/ from binary/dist. (Conceptual: Handled by build process)

---
### UNAFFECTED: Cargo.toml, README.md, most of docs/

---
## TRADEOFFS

- Eliminates non-essential code, reducing binary size and attack surface.
- Simplifies codebase, lowering audit and maintenance costs.
- Improves clarity for contributors by removing dead or legacy code.
- Allows selective compilation of features (WASM, Serde, CLI) based on project needs.

---
*This document should be periodically re-audited for dead/unused modules via `git grep` or IDE autoref hints, and updated as refactors or new feature gates are added.*