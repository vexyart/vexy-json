# Contributing

## Prerequisites

- Git
- A Rust toolchain with `cargo`, `rustfmt` and `clippy`
- [`uv`](https://docs.astral.sh/uv/) (provides `uv` and `uvx`) and `python3`
- For the WebAssembly module: the `wasm32-unknown-unknown` target and
  `wasm-pack`
- For the WebAssembly smoke test: Node.js

## Getting the code

```bash
git clone https://github.com/vexyart/vexy-json.git
cd vexy-json
```

## Running the full gate

`build.sh` in the repository root is the canonical check. Run it before you
open a pull request:

```bash
./build.sh
```

It performs these steps in order and stops at the first failure:

1. Checks that `git`, `cargo`, `rustc`, `uv`, `uvx` and `python3` are present
   and that the expected project files exist.
2. Runs `scripts/release-manifests.py check` to confirm that all manifests
   carry the same version.
3. Runs `cargo fmt --check` and `cargo clippy -D warnings` on the CLI crate.
4. Builds and tests the workspace (except the Python crate), including
   doctests and examples.
5. Builds the Python wheel with `uv build`, installs it in a temporary virtual
   environment, runs `crates/python/tests` with pytest and a smoke test that
   compares `vexy_json.__version__` with the workspace version.
6. Builds this documentation site in strict mode into a temporary directory.
7. Fails if the build changed any tracked or untracked file in the checkout.

The output is also written to `build.log.txt`. The last line is
`BUILD SUCCESS: …` or `BUILD FAILURE: …`.

## Working on the documentation

The documentation sources are Markdown files in `src_docs/md/`. The site
configuration is `src_docs/mkdocs.yml`. The site is built with ProperDocs and
the MaterialX theme into `docs/`, which GitHub Pages serves.

1. Edit or add a page in `src_docs/md/`. Add new pages to the `nav` list in
   `src_docs/mkdocs.yml`.
2. Preview the site:

    ```bash
    src_docs/build.sh serve
    ```

    The preview runs at `http://127.0.0.1:8000` and reloads on save.

3. Build the site:

    ```bash
    ./build.sh docs
    ```

    This runs `src_docs/build.sh`, which builds in strict mode: any warning,
    such as a broken link between pages, fails the build. The result replaces
    the contents of `docs/`.

Do not edit files in `docs/` by hand; the next build overwrites them. The
`.github/workflows/docs.yml` workflow rebuilds `docs/` on every push to `main`
that changes `src_docs/` or the WebAssembly sources, and commits the result.

### The demo and the WebAssembly bundle

The browser demo lives in `src_docs/md/demo/`, with its scripts and styles in
`src_docs/md/assets/`. The WebAssembly bundle it loads lives in
`src_docs/md/pkg/`. ProperDocs copies these files to `docs/` unchanged.

To rebuild the bundle after changing `crates/core` or `crates/wasm`:

1. Run `./scripts/build-wasm.sh release`. The script writes the new bundle
   to `src_docs/md/pkg/`.
2. Run the smoke test against it:

    ```bash
    mkdir -p crates/wasm/pkg
    cp src_docs/md/pkg/* crates/wasm/pkg/
    node crates/wasm/test.mjs
    ```

    The test prints `All tests passed!` at the end. Delete
    `crates/wasm/pkg/` afterwards.

3. Rebuild the site with `./build.sh docs`.

## Code guidelines

- Keep the forgiving behaviour behind `ParserOptions` so that every extension
  can be switched off.
- Add a test in `tests/` for each syntax change. The existing files are
  grouped by topic, such as `comment_handling.rs`, `number_formats.rs` and
  `newline_as_comma.rs`.
- When a change affects a binding, update its tests: `crates/python/tests/`
  for Python and `crates/wasm/test.mjs` for WebAssembly.
- Record user-visible changes in `CHANGELOG.md` under `Unreleased`.

## Releasing

Maintainers release with `./publish.sh`. According to the README, the script
verifies the repository boundary, checkpoints intended changes, runs the full
build, creates and verifies the release tag, then publishes the workspace
crates in dependency order followed by the Python package (built with
`uv build`, uploaded with `uv publish`).

Pushing the `v*` tag starts `.github/workflows/release.yml`, which tests the
tagged source, builds the CLI binaries, the macOS installer and the
WebAssembly archive, and attaches them to the GitHub release.

!!! warning "Interrupted releases"
    Registry uploads cannot be undone. If publication stops after the tag
    was created, check crates.io and PyPI before retrying, and resume only the
    uploads that are missing. Do not rerun the whole script: it derives the
    next version from the existing tags.
