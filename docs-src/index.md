---
nav_title: Home
nav_order: 1
---

# JSON from the real world, parsed without ceremony

Vexy JSON accepts strict JSON and the useful deviations people actually write:
comments, trailing commas, unquoted keys, single quotes, implicit containers,
newline separators, and extended number formats. Its Rust engine is available
as a Rust library, native Python package, CLI, WebAssembly module, and C API.

```text
// Configuration, generated output, or a hand-edited file
service: 'preview'
retries: 0b11
hosts: [api, cache,]
```

The default parser turns that into an ordinary object. You can disable every
extension when an input boundary requires strict JSON.

## Install

=== "Rust"

    ```bash
    cargo add vexy-json
    ```

=== "Python"

    ```bash
    uv add vexy-json
    ```

=== "CLI"

    ```bash
    cargo install vexy-json-cli
    ```

Python installs the `vexy-json` distribution and imports it as `vexy_json`.

## Start here

- [Getting started](user/getting-started.md) — install, parse, configure, and
  handle errors.
- [Forgiving syntax](user/features.md) — the accepted extensions and how to
  switch them off.
- [Python API](user/api/python/index.md) — functions, options, files, and typed
  interfaces.
- [Rust API](user/api/rust.md) — core types and parser configuration.
- [Troubleshooting](user/guides/troubleshooting.md) — installation, parsing,
  WASM, and release diagnostics.
- [Interactive demo](demo/index.html) — parse examples in the browser.

## Choose the right surface

| Need | Package | Entry point |
| --- | --- | --- |
| Rust application | `vexy-json` | `vexy_json::parse` |
| Python application | `vexy-json` | `vexy_json.parse` |
| Terminal or shell pipeline | `vexy-json-cli` | `vexy-json` |
| Browser or Node.js | `vexy-json-wasm` | WASM bindings |
| C or C++ integration | `vexy-json-c-api` | C headers and library |

## Guarantees and boundaries

Vexy JSON returns normal JSON-compatible values. Forgiving syntax changes what
the parser accepts; it does not invent application-specific data. Parsing can
still fail for incomplete strings, invalid escapes, excessive nesting, and
inputs that remain ambiguous after the enabled recovery rules.

Use strict options at trust boundaries. Use forgiving defaults for
configuration, migrations, editor input, logs, and generated text where a
helpful parser is more valuable than a brittle one.

## Project links

- [Source and issues](https://github.com/vexyart/vexy-json)
- [Rust package](https://crates.io/crates/vexy-json)
- [Python package](https://pypi.org/project/vexy-json/)
- [Documentation](https://vexy.dev/vexy-json/)

Vexy JSON is available under MIT or Apache-2.0, at your option.
