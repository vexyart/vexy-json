---
nav_title: Architecture
nav_order: 9
---

# Architecture

Vexy JSON keeps one parsing engine at the centre of every package. The
`vexy-json-core` crate owns tokenization, parser options, values, errors,
streaming, and repair. The root Rust crate provides the ergonomic public API;
the CLI, Python, WASM, C API, and Serde crates adapt that API at their language
or runtime boundaries.

This arrangement keeps accepted syntax and error semantics consistent across
surfaces. New parsing behaviour belongs in the core with shared tests. Binding
crates should translate native values and errors without creating a second
parser or silently changing defaults.

## Design constraints

- Standard JSON must remain valid input.
- Forgiving features are explicit parser options and can be disabled.
- Repair is bounded and must not be presented as schema validation.
- Library APIs return structured errors; command-line tools turn them into
  actionable diagnostics and exit statuses.
- Distribution packages are built and tested from their packaged artifacts,
  not only from the source checkout.

See the [Rust API](../user/api/rust.md), [Python API](../user/api/python/index.md),
[build process](build-process.md), and [plugin development](plugin-development.md)
for the concrete interfaces around this architecture.
