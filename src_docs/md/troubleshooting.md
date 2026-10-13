# Troubleshooting

## Reading error messages

Every parse error names a zero-based byte offset, `at position N`. The first
byte of the input is position `0`.

```text
Expected , or } or newline but found Eof at position 5
```

This message, for the input `{a: 1`, means the parser reached the end of the
input (`Eof`) at byte 5 while it still expected a comma, a closing brace or a
newline. Common messages:

| Message | Usual cause |
| --- | --- |
| `Expected , or } or newline but found Eof` | The input ends inside an object |
| `Expected , or ] or newline but found …` | A missing comma or a wrong closing bracket in an array |
| `Expected string key but found UnquotedString` | An unquoted key while `allow_unquoted_keys` is off |
| `Expected : but found RightBrace` | An unquoted value with spaces, such as `{a: two words}` |
| `Unexpected character 'a'` | A bare word while the lexer is in strict mode (see below) |
| `Trailing comma at position N` | A trailing comma while `allow_trailing_commas` is off |
| `Expected end of input but found …` | Two top-level values, such as `{"a":1}{"b":2}` |
| `Depth limit exceeded at position N` | Nesting deeper than `max_depth` |

To see the line and column in the CLI, add `--verbose-errors`. For a file it
also prints the lines around the error with a `^` under the column.

## "Unexpected character" after turning off comments

`--no-comments` (or `allow_comments: false`) puts the lexer in strict mode.
Strict mode rejects unquoted keys, unquoted values and single-quoted strings
even though their own options are still on. If you need to reject comments
but keep the other extensions, there is no combination of options for that in
this version; strip comments before parsing instead.

## Object keys come out in a different order

`Value::Object` is a hash map (`FxHashMap`). Key order is not kept in Rust, in
the CLI output, in the WebAssembly output or in the C output, and Python `dict`s
built by `vexy_json.parse` can list keys in a different order too. If order
matters, sort the keys after parsing.

## The CLI writes nothing for empty input

For empty standard input, `vexy-json` prints `No input provided` to standard
error and exits with status `0`. With `--validate` it prints nothing. The
library returns `null` for empty input.

## `--repair` refuses to repair

The CLI only closes open brackets and braces at the very end of the input.
It refuses when that would invent data, for example when the input stops in
the middle of a string, after a comma or inside a comment. The message says
which case applied. See [Repairing incomplete input](repair.md).

## WebAssembly: errors on strings with escapes

`parse_json` and the other WebAssembly functions fail on strings that contain
an escaped quote (`\"`) or a control character such as `\n`:

```text
control character (\u0000-\u001F) found while parsing a string at line 2 column 0
```

The parser itself accepts these strings; the failure happens when the
WebAssembly wrapper converts the result to a JSON string. Until this is fixed,
parse such input with the CLI, the Python package or the Rust library.

## Python: `dumps` output is not valid JSON

`vexy_json.dumps(obj)` without `indent` does not escape quotes or control
characters inside strings. Pass `indent=2` (which escapes) or use the standard
library's `json.dumps`.

## C: the output is wrapped in type names

The C functions return Serde's tagged form of the value, such as
`{"Object":{"a":{"Number":{"Integer":1}}}}`, not plain JSON. See
[C and C++](c-api.md#output-format).

## The demo shows "JSON.parse" behaviour

The [interactive demo](demo/index.html) falls back to the browser's
`JSON.parse` when the WebAssembly module fails to load, and then rejects every
extension. Open the browser console: a `Failed to load WASM` message names
the cause. Serve the demo over HTTP; browsers do not load WebAssembly modules
from `file://` pages.

## Very large or deeply nested input

- The default `max_depth` is 128. Raise it for legitimately deep data.
- For files with one JSON value per line, use `--ndjson` (CLI),
  `NdJsonParser` (Rust) or `StreamingParser.parse_lines` (Python) instead of
  parsing the whole file as one value.
- `--parallel-parse` splits one large input into chunks of at least
  `--chunk-size` bytes and parses them on several threads.

## Reporting a problem

Open an issue at
[github.com/vexyart/vexy-json/issues](https://github.com/vexyart/vexy-json/issues)
with the smallest input that shows the problem, the interface you used, the
version (`vexy-json --version` or `vexy_json.__version__`) and the full error
message.
