# Architecture

This page describes how the repository is organized and how a string becomes
a value. It is meant for contributors and for users who need to know why the
interfaces behave as they do.

## Workspace layout

The repository is one Cargo workspace. The root package `vexy-json` is the
facade crate; the other crates live in `crates/`:

| Path | Crate | Role |
| --- | --- | --- |
| `crates/core` | `vexy-json-core` | Lexers, parsers, errors, repair, streaming, parallel and transform modules |
| `crates/cli` | `vexy-json-cli` | The `vexy-json` binary (`clap`, `tokio`, `rayon`, `notify`) |
| `crates/python` | `vexy-json-python` | PyO3 extension, packaged with maturin as `vexy-json` on PyPI |
| `crates/wasm` | `vexy-json-wasm` | `wasm-bindgen` exports for JavaScript |
| `crates/c-api` | `vexy-json-c-api` | C ABI and the `vexy_json.h` header |
| `crates/serde` | `vexy-json-serde` | `SerdeValue` wrapper |
| `crates/test-utils` | `vexy-json-test-utils` | Helpers for the test suite |
| `src/` | `vexy-json` | Re-exports from the core and Serde crates |

Other top-level directories:

- `tests/` — integration tests for the root crate
- `examples/` — small runnable programs (`cargo run --example simple`)
- `benches/` — Criterion benchmarks
- `fuzz/` — `cargo-fuzz` targets
- `scripts/` — WebAssembly build, release and packaging scripts
- `src_docs/` — this documentation site

All crates share one version number. `scripts/release-manifests.py check`
verifies that every manifest carries the same version.

## From text to value

Take the input `{a: 1, b: [2,],}` and follow it through `parse`.

### 1. Choosing a lexer

`Parser::new` looks at the options. If any of the six syntax options is on,
it uses `FastLexer`. The lexer runs in forgiving mode when `allow_comments` is
on and in strict mode when it is off. If all six syntax options are off, the
parser uses `Lexer`, a tokenizer generated with the `logos` crate.

### 2. Tokens

The lexer turns the text into tokens, each with a `Span` (start and end byte
offset). The parser reads the token text back through the span:

```text
LeftBrace  UnquotedString  Colon  Number  Comma
UnquotedString  Colon  LeftBracket  Number  Comma  RightBracket
Comma  RightBrace  Eof
```

Spaces are dropped. Comments arrive as `SingleLineComment` and
`MultiLineComment` tokens, which the parser skips. Newlines arrive as
`Newline` tokens, so that `newline_as_comma` can treat them as separators.

### 3. Parsing

The parser is a recursive descent parser. It keeps a depth counter in
`ParserState` and stops with `DepthLimitExceeded` when the depth reaches
`max_depth`. For the example it:

1. sees `{` and starts an object;
2. reads the unquoted key `a` (allowed by `allow_unquoted_keys`), the colon and
   the number `1`;
3. reads the key `b` and an array; inside it, the comma before `]` is a
   trailing comma (allowed by `allow_trailing_commas`);
4. accepts the comma before `}` for the same reason;
5. returns `Value::Object` with the members `a` → `Integer(1)` and
   `b` → `Array([Integer(2)])`.

At the top level, the parser checks for a key followed by `:` (implicit
object) or a value followed by a comma (implicit array) when
`implicit_top_level` is on. After the value, any further token other than the
end of input is an error.

### 4. Numbers

Number tokens are converted to `Number::Integer(i64)` when they are integers
that fit, including hexadecimal, octal, binary and underscore forms, and to
`Number::Float(f64)` otherwise.

## The fallback chain

`parse_with_fallback` wraps the parser in three tiers: `serde_json` for strict
JSON, then the forgiving parser, then a bracket repairer when `enable_repair`
is on. The result records which tier succeeded. The CLI calls this function
with repair turned off, and adds its own end-of-input repair on top (see
[Repairing incomplete input](repair.md)).

## How each interface converts the value

The core returns `Value`. Each interface converts it differently, which
explains some of their differences:

| Interface | Conversion |
| --- | --- |
| CLI | Its own formatter, with string escaping, compact or indented |
| Python | Builds `dict`, `list` and scalar objects directly |
| WebAssembly | Prints `Value` with `Display`, re-reads it with `serde_json`, prints compact JSON |
| C | Serializes `Value` with its derived Serde implementation (tagged form) |

Because `Display` does not escape strings, the WebAssembly round trip fails on
strings with escaped quotes or control characters. See
[Troubleshooting](troubleshooting.md).

## Build and publication

| Script or workflow | Purpose |
| --- | --- |
| `build.sh` | Full local gate: format, Clippy, build, tests, doctests, examples, Python wheel and tests, documentation |
| `build.sh docs` | Documentation only (calls `src_docs/build.sh`) |
| `scripts/build-wasm.sh` | Builds the WebAssembly module into `src_docs/md/pkg/` |
| `publish.sh` | Tags a release and publishes the crates and the Python package |
| `.github/workflows/ci.yml` | Tests on pushes and pull requests |
| `.github/workflows/release.yml` | On a `v*` tag: tests, then builds CLI binaries, the macOS installer and the WebAssembly archive for the GitHub release |
| `.github/workflows/docs.yml` | On pushes that change `src_docs/` or the WebAssembly sources: rebuilds this site into `docs/` |

GitHub Pages serves the `docs/` folder of the `main` branch at
`https://vexy.dev/vexy-json/`.
