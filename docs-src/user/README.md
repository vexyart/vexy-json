---
nav_title: User Guide
nav_order: 2
---

# User guide

Vexy JSON turns relaxed JSON-like text into ordinary values without requiring
a cleanup pass. Choose a guide based on the surface you use:

- [Getting started](getting-started.md) for installation and first parses.
- [Forgiving syntax](features.md) for accepted extensions and strict settings.
- [Python API](api/python/index.md) for Python functions and options.
- [Rust API](api/rust.md) for Rust types and configuration.
- [Streaming API](api/streaming-api.md) for incremental processing.
- [WebAssembly API](api/wasm.md) for browser and Node.js integration.
- [JSON repair](guides/json-repair.md) for repair boundaries.
- [Migration](guides/migration.md) for adopting the parser.
- [Troubleshooting](guides/troubleshooting.md) for common failures.

The browser-based [interactive demo](../demo/index.html) is useful for quickly
checking how a sample is interpreted.

## Naming at a glance

| Surface | Name |
| --- | --- |
| Python distribution | `vexy-json` |
| Python import | `vexy_json` |
| Rust package | `vexy-json` |
| Rust crate | `vexy_json` |
| CLI executable | `vexy-json` |

## Safe adoption

Start with forgiving defaults for trusted configuration and generated text.
At external trust boundaries, explicitly disable syntax extensions and repair,
then validate the parsed value against your application schema.
