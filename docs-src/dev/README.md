---
nav_title: Development
nav_order: 8
---

# Development guide

The repository is a Cargo workspace containing the core parser, CLI, Python,
Serde, WebAssembly, C API, and test-support crates.

## Canonical commands

```bash
./build.sh
./install.sh
./publish.sh --dry-run
```

`build.sh` is the complete verification gate. It checks formatting and linting,
builds and tests the Rust workspace, builds and tests the Python wheel, and
renders this MkDocs site in strict mode. Its output is recorded in
`build.log.txt`.

`install.sh` installs the CLI under `~/.local` by default and builds a native
wheel matching the unmanaged system Python selected by uv.

`publish.sh` checkpoints intended dirty changes, runs `build.sh`, synchronizes
versions, runs `build.sh` again, creates and verifies release refs, then
publishes crates in dependency order and uploads Python artifacts with
`uv publish`.

## Contributor references

- [Contributing](contributing.md)
- [Architecture](design.md)
- [Build process](build-process.md)
- [Benchmarks](benchmarks.md)
- [Plugin development](plugin-development.md)
- [Plugin registry](plugin-registry.md)
- [Release process](release-process.md)
- [macOS packaging](packaging-macos.md)

Historical plans, debug fixtures, and completed migration notes remain under
`docs-src/internal/` as project evidence but are excluded from the public site.
