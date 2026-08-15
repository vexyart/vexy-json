---
nav_title: Contributing
nav_order: 7
---

# Contributing

Vexy JSON welcomes focused fixes, tests, documentation improvements, and API
proposals. Open an issue before a large architectural change so the intended
behaviour and compatibility constraints are clear.

## Prepare a change

1. Fork and clone `vexyart/vexy-json`.
2. Create a short-lived branch from `main`.
3. Add or update tests that demonstrate the required behaviour.
4. Keep changes scoped; the workspace serves Rust, Python, CLI, WASM, and C API
   users from the same parser core.
5. Update user documentation when public behaviour changes.

## Run the canonical gate

Use the repository script instead of invoking individual Cargo commands:

```bash
./build.sh
cat build.log.txt
```

The gate formats and checks the Rust workspace, runs unit, integration, and
documentation tests, builds the Python source distribution and native wheel,
tests the installed wheel in an isolated environment, and builds this MkDocs
site in strict mode.

## Submit the pull request

Describe the problem, the chosen behaviour, and the verification you ran. Link
the relevant issue and call out changes to public syntax, error behaviour, or
package metadata. Avoid committing generated build output or local tool caches.

For implementation details, continue with the [build process](build-process.md),
[plugin development](plugin-development.md), or the public [Rust API](../user/api/rust.md).

Report defects and feature requests in the
[GitHub issue tracker](https://github.com/vexyart/vexy-json/issues).
