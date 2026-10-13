# Vexy JSON

Vexy JSON is a forgiving JSON parser written in Rust. It reads standard JSON and
the relaxed syntax found in hand-written configuration files and in text
produced by language models, and it returns ordinary JSON values.

The same parser is available as a Rust library, a command-line tool, a Python
package, a WebAssembly module and a C library.

## What it accepts

Standard JSON parses unchanged. On top of that, the default settings accept:

- `//`, `/* … */` and `#` comments
- trailing commas in arrays and objects
- unquoted object keys and unquoted single-word string values
- single-quoted strings
- implicit top-level objects (`a: 1, b: 2`) and arrays (`1, 2, 3`)
- newlines in place of commas
- hexadecimal (`0x1F`), octal (`0o17`) and binary (`0b101`) integers, and
  underscores in numbers (`1_000`)

Each extension can be switched off. See [Accepted syntax](syntax.md) and
[Parser options](options.md).

## One example

This input is not valid JSON:

```text
// service settings
{
  name: 'cache',
  ports: [8080, 8081,],
  retries: 0x3
}
```

Vexy JSON turns it into this value:

```json
{
  "name": "cache",
  "ports": [8080, 8081],
  "retries": 3
}
```

The comment is dropped, the unquoted key and the single-quoted string become
ordinary strings, the trailing comma is ignored and the hexadecimal number
becomes a decimal integer.

## Ways to use it

| Interface | Package | Entry point |
| --- | --- | --- |
| Rust library | `vexy-json` on crates.io | `vexy_json::parse` |
| Command line | `vexy-json-cli` on crates.io | `vexy-json` binary |
| Python | `vexy-json` on PyPI | `import vexy_json` |
| Browser and Node.js | built with `wasm-pack` from `crates/wasm` | `parse_json` |
| C and C++ | `vexy-json-c-api` (static and dynamic library) | `vexy_json_parse` |

Start with [Installation](installation.md) and the [Quickstart](quickstart.md).

## Try it in the browser

The [interactive demo](demo/index.html) runs the WebAssembly build of the
parser in your browser. Type relaxed JSON on the left and the normalized JSON
appears on the right. An older version of the demo is kept at
[demo/legacy.html](demo/legacy.html).

## Project links

- Source code: [github.com/vexyart/vexy-json](https://github.com/vexyart/vexy-json)
- Rust crate: [crates.io/crates/vexy-json](https://crates.io/crates/vexy-json)
- Python package: [pypi.org/project/vexy-json](https://pypi.org/project/vexy-json/)
- Changes: [CHANGELOG.md](https://github.com/vexyart/vexy-json/blob/main/CHANGELOG.md)

Vexy JSON is licensed under either the MIT License or the Apache License 2.0,
at your option.
