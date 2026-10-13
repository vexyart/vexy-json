# Vexy JSON

**Documentation:** https://vexy.dev/vexy-json/

Vexy JSON is a forgiving JSON parser for Rust and Python. It accepts standard
JSON as well as practical extensions commonly found in configuration files and
LLM output, while still producing ordinary JSON values.

Supported extensions include comments, trailing commas, single-quoted strings,
unquoted object keys, implicit top-level objects and arrays, newline separators,
and binary, octal, hexadecimal, or underscore-separated numbers.

## Rust

Add the high-level crate:

```toml
[dependencies]
vexy-json = "1"
```

Parse forgiving JSON with the default options:

```rust
use vexy_json::parse;

fn main() -> vexy_json::Result<()> {
    let value = parse("{name: 'Vexy', enabled: true,}")?;
    println!("{value}");
    Ok(())
}
```

The workspace also publishes focused crates for the core parser, CLI, Serde
integration, WebAssembly bindings, and the C API.

Complete guides and API references are published at
[vexy.dev/vexy-json](https://vexy.dev/vexy-json/).

## Python

Install the native Python package with uv:

```bash
uv add vexy-json
```

```python
import vexy_json

value = vexy_json.parse("{name: 'Vexy', enabled: true,}")
assert value == {"name": "Vexy", "enabled": True}
```

## Command line

Install the CLI from crates.io:

```bash
cargo install vexy-json-cli
```

Then parse a file or standard input with `vexy-json`.

## Development

Run the repository's complete formatting, linting, Rust, and packaged-Python
gate with:

```bash
./build.sh
```

Do not invoke individual Cargo test or lint commands for release verification;
`build.sh` is the canonical gate and records its output in `build.log.txt`.

Install the Rust CLI under `~/.local` and the Python extension into the system
Python selected by `uv` with:

```bash
./install.sh
```

The installer resolves one unmanaged interpreter with `uv python find --system
--no-managed-python`, builds a matching native wheel, and installs it explicitly
with `uv pip install --system`. Pass `--prefix PATH` to change only the Rust CLI
installation prefix.

## Releasing

`./publish.sh` verifies the repository boundary, checkpoints intended changes,
runs the complete build, creates and verifies the release tag, then publishes
the workspace crates in dependency order followed by the Python package.
Python distributions are built with `uv build` and uploaded with `uv publish`.

If publication stops after a tag is created, check crates.io and PyPI before
retrying anything. Registry uploads are irreversible, and a transport error can
be ambiguous. Resume only the artifacts confirmed absent; do not rerun the full
script, because the next version is derived from existing tags.

The root Rust package deliberately includes only its manifest, license, README,
build script, Rust source, examples, tests, and benchmarks. Generated
documentation, installers, previous release archives, and local development
artifacts are not shipped to crates.io.

## License

Licensed under either the MIT License or the Apache License 2.0, at your option.
