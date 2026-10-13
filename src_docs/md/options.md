# Parser options

Every interface reads the same set of options. In Rust they are the fields of
`vexy_json::ParserOptions`; the Python, WebAssembly and C bindings expose the
same names. With the defaults, every syntax extension is on.

## Option reference

| Option | Type | Default | Effect when `true` |
| --- | --- | --- | --- |
| `allow_comments` | bool | `true` | Accept `//`, `#` and `/* … */` comments |
| `allow_trailing_commas` | bool | `true` | Accept a comma after the last array element or object member |
| `allow_unquoted_keys` | bool | `true` | Accept object keys without quotes |
| `allow_single_quotes` | bool | `true` | Accept strings in single quotes |
| `implicit_top_level` | bool | `true` | Accept `a: 1` as `{"a": 1}` and `1, 2` as `[1, 2]` |
| `newline_as_comma` | bool | `true` | Accept a newline in place of a comma |
| `max_depth` | integer | `128` | Maximum nesting depth of arrays and objects |
| `enable_repair` | bool | `true` | Let `parse_with_fallback` try bracket repair after a parse error |
| `max_repairs` | integer | `100` | Upper limit on repairs that the repairer attempts |
| `fast_repair` | bool | `false` | Use the repairer without its cache |
| `report_repairs` | bool | `true` | Reserved; the parser does not read it in this version |

The three repair options affect only `parse_with_fallback` (Rust) and the
functions built on it. Plain `parse` and `parse_with_options` never repair
input. See [Repairing incomplete input](repair.md).

## How strict mode works

Switching the six syntax options off makes the parser behave like a strict
JSON parser. Take this input:

```text
{a: 1}
```

With the defaults it parses to `{"a": 1}`. With `allow_unquoted_keys` set to
`false`, the parser stops at the key and reports:

```text
Expected string key but found UnquotedString at position 1
```

The position is a zero-based byte offset into the input.

!!! note "`allow_comments` also controls the lexer mode"
    When `allow_comments` is `false` but another syntax option is still on,
    the lexer runs in strict mode. Strict mode rejects unquoted words and
    single-quoted strings even if `allow_unquoted_keys` or
    `allow_single_quotes` is `true`: with `--no-comments`, the input
    `{a: 1}` fails with `Unexpected character 'a' at position 1`. Trailing
    commas and prefixed numbers such as `0x10` are still accepted.

## Setting options

=== "Rust"

    ```rust
    use vexy_json::{parse_with_options, ParserOptions};

    let strict = ParserOptions {
        allow_comments: false,
        allow_trailing_commas: false,
        allow_unquoted_keys: false,
        allow_single_quotes: false,
        implicit_top_level: false,
        newline_as_comma: false,
        ..ParserOptions::default()
    };

    assert!(parse_with_options("{a: 1}", strict.clone()).is_err());
    assert!(parse_with_options(r#"{"a": [1, 2]}"#, strict).is_ok());
    ```

=== "Python"

    ```python
    import vexy_json

    vexy_json.parse_with_options('{"a": 1, }', allow_trailing_commas=True)
    # {'a': 1}

    vexy_json.parse_with_options('{"a": 1, }', allow_trailing_commas=False)
    # ValueError: Parse error: Trailing comma at position 10
    ```

    Every option is a keyword argument with the default shown in the table.
    `vexy_json.load(fp, **options)` accepts the same keywords.

=== "Command line"

    ```bash
    printf '{"a":1, }' | vexy-json --no-trailing-commas
    ```

    ```text
    Error:
    <stdin>: Trailing comma at position 9
    ```

    The CLI has one `--no-…` flag for each syntax option and `--max-depth`.
    It always runs with `enable_repair` set to `false`; its own `--repair`
    flag works differently (see [Command line](cli.md)).

=== "JavaScript"

    ```javascript
    import init, { parse_json_with_options } from "./pkg/vexy_json_wasm.js";
    await init();

    // input, allow_comments, allow_trailing_commas, allow_unquoted_keys,
    // allow_single_quotes, implicit_top_level, newline_as_comma,
    // enable_repair, max_depth (undefined means 128)
    const json = parse_json_with_options(
      "{a: 1}", true, true, false, true, true, true, true, undefined,
    );
    // throws "Parse error: Expected string key but found UnquotedString at position 2"
    ```

    The WebAssembly function takes positional arguments. `max_repairs` is
    fixed at 100 and `fast_repair` and `report_repairs` at `false`.

=== "C"

    ```c
    VexyJsonParserOptions options = vexy_json_default_options();
    options.allow_unquoted_keys = false;
    VexyJsonParseResult result = vexy_json_parse_with_options("{a: 1}", &options);
    /* result.json is NULL, result.error holds the message */
    vexy_json_free_result(result);
    ```

## Depth limit

`max_depth` protects against stack exhaustion on deeply nested input. A
document with 200 nested arrays fails with the default limit:

```text
Depth limit exceeded at position 129
```

Raise the limit with `max_depth` (Rust, Python, C), the ninth argument of
`parse_json_with_options` (JavaScript) or `--max-depth` (CLI).
