# Installation

Vexy JSON ships as several packages built from one Cargo workspace. Install the
one that matches the language you work in. All packages share one version
number.

## Rust library

Prerequisites: a Rust toolchain with Cargo.

1. Add the crate to your project:

    ```bash
    cargo add vexy-json
    ```

    Or add it to `Cargo.toml` by hand:

    ```toml
    [dependencies]
    vexy-json = "1"
    ```

2. Build the project with `cargo build`.

The `vexy-json` crate re-exports the parser from `vexy-json-core`. Its default
`serde` feature also pulls in `vexy-json-serde`. To build without it, turn off
default features:

```toml
[dependencies]
vexy-json = { version = "1", default-features = false }
```

The workspace publishes these crates to crates.io:

| Crate | Contents |
| --- | --- |
| `vexy-json` | High-level crate that re-exports the parser API |
| `vexy-json-core` | Lexer, parser, repair, streaming and transform modules |
| `vexy-json-cli` | The `vexy-json` command-line program |
| `vexy-json-serde` | `SerdeValue`, a Serde wrapper around `Value` |
| `vexy-json-wasm` | WebAssembly bindings (built with `wasm-pack`) |
| `vexy-json-c-api` | C ABI, built as a static and a dynamic library |

## Command-line tool

### With Cargo

1. Install the CLI crate:

    ```bash
    cargo install vexy-json-cli
    ```

2. Check the installation:

    ```bash
    vexy-json --version
    ```

    The command prints the program name and version, for example
    `vexy_json 1.5.13`.

### From a GitHub release

Each tagged release on the
[releases page](https://github.com/vexyart/vexy-json/releases) carries
prebuilt binaries:

- `vexy-json-<version>-linux-x86_64.tar.gz` and `…-linux-aarch64.tar.gz`
- `vexy-json-<version>-macos-x86_64.tar.gz`, `…-macos-aarch64.tar.gz`, a
  universal `…-macos.zip` and a `…-macos.dmg` installer
- `vexy-json-<version>-windows-x86_64.zip`

Unpack the archive and put the `vexy-json` binary (`vexy-json.exe` on Windows)
on your `PATH`.

### From a source checkout

`install.sh` in the repository root builds the CLI and installs it under
`~/.local` (the binary goes to `~/.local/bin`). It also builds and installs
the Python extension; see below.

```bash
./install.sh                 # CLI under ~/.local
./install.sh --prefix /opt   # CLI under /opt instead
```

The prefix can also come from the `VEXY_JSON_PREFIX` environment variable.

## Python package

Prerequisites: Python 3.8 or later. The PyPI distribution is named
`vexy-json`; the import name is `vexy_json`.

```bash
uv add vexy-json          # in a uv-managed project
uv pip install vexy-json  # into the active environment
pip install vexy-json     # with pip
```

Check the installation:

```bash
python -c "import vexy_json; print(vexy_json.__version__)"
```

The package contains a compiled extension module (`vexy_json._vexy_json`)
built from `crates/python` with maturin and PyO3. If no prebuilt wheel exists
for your platform, the installer builds it from the source distribution, which
needs a Rust toolchain.

`./install.sh` in a source checkout builds a wheel for the system Python
selected by `uv python find --system --no-managed-python` and installs it with
`uv pip install --system`.

## WebAssembly module

The WebAssembly bindings are not published to npm. Use one of these sources:

- **Release archive.** Each GitHub release has a
  `vexy-json-wasm-<version>.tar.gz` asset with the `wasm-pack` output (`pkg/`).
- **Hosted copy.** The documentation site serves the current build at
  `https://vexy.dev/vexy-json/pkg/vexy_json_wasm.js`. The demo loads it from
  there.
- **Build it yourself.** Prerequisites: Rust with the
  `wasm32-unknown-unknown` target and
  [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

    1. Run the build script from the repository root:

        ```bash
        ./scripts/build-wasm.sh release
        ```

    2. Collect the output from `src_docs/md/pkg/`: `vexy_json_wasm.js`,
       `vexy_json_wasm_bg.wasm`, the TypeScript declarations and
       `package.json`.

The build uses `wasm-pack build --target web`, so the module is an ES module
with a default `init` export. See [JavaScript and WebAssembly](webassembly.md).

## C and C++ library

Prerequisites: a Rust toolchain and a C or C++ compiler.

1. Build the library:

    ```bash
    cargo build --release -p vexy-json-c-api
    ```

    Cargo writes `libvexy_json_c_api.a` and the platform's dynamic library
    (`libvexy_json_c_api.dylib`, `.so` or `.dll`) to `target/release/`.

2. Add `crates/c-api/include` to your include path. It holds `vexy_json.h`,
   which C and C++ code can include.

3. Link against the library. See [C and C++](c-api.md) for an example.
