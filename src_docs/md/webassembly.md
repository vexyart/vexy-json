# JavaScript and WebAssembly

The `crates/wasm` crate compiles the parser to WebAssembly with
`wasm-bindgen`. `wasm-pack build --target web` turns it into an ES module that
runs in browsers and in Node.js.

## Getting the module

The module is not on npm. Get it from a GitHub release
(`vexy-json-wasm-<version>.tar.gz`), from this site at
`https://vexy.dev/vexy-json/pkg/`, or build it with
`./scripts/build-wasm.sh release` (see [Installation](installation.md)).

A build produces these files:

| File | Content |
| --- | --- |
| `vexy_json_wasm.js` | ES module with the JavaScript glue code and the exported functions |
| `vexy_json_wasm_bg.wasm` | The compiled parser |
| `vexy_json_wasm.d.ts`, `vexy_json_wasm_bg.wasm.d.ts` | TypeScript declarations |
| `package.json` | Package metadata (`vexy-json-wasm`) |

## Loading the module

The default export is an `init` function. Call it once and wait for it before
calling any other export.

In a browser, `init()` fetches `vexy_json_wasm_bg.wasm` from the same
directory as the JavaScript file:

```html
<script type="module">
  import init, { parse_json } from "./pkg/vexy_json_wasm.js";

  await init();
  const json = parse_json("{name: 'Vexy', tags: [a, b,],}");
  console.log(JSON.parse(json)); // { name: "Vexy", tags: ["a", "b"] }
</script>
```

In Node.js, read the `.wasm` file yourself and pass its bytes to `init`, as
the repository's `crates/wasm/test.mjs` does:

```javascript
import { readFileSync } from "node:fs";
import init, { parse_json } from "./pkg/vexy_json_wasm.js";

await init(readFileSync("./pkg/vexy_json_wasm_bg.wasm"));
console.log(parse_json("a: 1, b: 2")); // {"a":1,"b":2}
```

Recent `wasm-bindgen` versions print a deprecation notice for this call form
and accept `init({ module_or_path: bytes })` instead.

## Exported functions

All parse functions return a **JSON string**, not a JavaScript object. Pass the
result to `JSON.parse` to get an object. On a parse error they throw a string
such as `Parse error: Expected string key but found UnquotedString at position 2`.

| Function | Returns | Notes |
| --- | --- | --- |
| `parse_json(input)` | `string` | Parses with the default options; compact JSON |
| `parse_json_with_options(input, allow_comments, allow_trailing_commas, allow_unquoted_keys, allow_single_quotes, implicit_top_level, newline_as_comma, enable_repair, max_depth)` | `string` | `max_depth` may be `undefined` (128) |
| `validate_json(input)` | `boolean` | `true` if `parse_json` would succeed |
| `stringify_value(input, indent)` | `string` | Parses, then prints compact JSON, or indented JSON when `indent` is given |
| `get_parser_options()` | `string` | JSON object describing each option, its default and description |
| `get_version_info()` | `string` | JSON object with `name`, `version`, `description`, `repository` |
| `parse_js(input)` | `string` | Older name for `parse_json` |
| `parse_with_options_js(input, …six booleans…)` | `string` | Older form; repair on, default depth |
| `is_valid(input)` | `boolean` | Older name for `validate_json` |
| `format(input)` | `string` | Same as `stringify_value(input, 2)` |

`stringify_value` always indents with two spaces; the value of `indent` only
switches indentation on.

```javascript
format("{a:[1,2]}");
// '{\n  "a": [\n    1,\n    2\n  ]\n}'

JSON.parse(get_version_info()).name;
// "vexy-json-wasm"
```

## Known limitation: escaped characters in strings

The WebAssembly functions build their output by printing the parsed value and
reading it back with `serde_json`. The printing step does not escape string
contents, so input with an escaped quote or an escaped control character in a
string fails:

```javascript
parse_json('{"a": "x\\ny"}');
// throws: control character (\u0000-\u001F) found while parsing a string …
parse_json('{"a": "say \\"hi\\""}');
// throws: expected `,` or `}` …
```

Strings without quotes, backslash escapes or control characters are not
affected.

## The browser demo

The [interactive demo](demo/index.html) loads `../pkg/vexy_json_wasm.js` from
this site, calls `parse_json` on the text in the left editor and shows the
result in the right editor, with the parse time. If the module fails to load,
it falls back to the browser's `JSON.parse`.

The source of the demo is `src_docs/md/demo/index.html` in the repository.
`scripts/build-wasm.sh` writes the module to `src_docs/md/pkg/`, and the
documentation build copies both to the published site.
