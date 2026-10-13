# C and C++

The `vexy-json-c-api` crate exposes the parser through a C ABI. Cargo builds
it as a static library and a dynamic library named `vexy_json_c_api`. The
declarations are in `crates/c-api/include/vexy_json.h`.

## Building and linking

Prerequisites: a Rust toolchain and a C compiler.

1. Build the library in release mode:

    ```bash
    cargo build --release -p vexy-json-c-api
    ```

2. Compile your program against the header and link the static library. On
   macOS:

    ```bash
    cc -I crates/c-api/include example.c \
      target/release/libvexy_json_c_api.a \
      -framework CoreFoundation -o example
    ```

    Other platforms need other system libraries. This command prints the
    list for your platform:

    ```bash
    cargo rustc -p vexy-json-c-api --lib --crate-type staticlib -- --print native-static-libs
    ```

3. Run `./example`.

## A complete example

```c
#include <stdio.h>
#include "vexy_json.h"

int main(void) {
    printf("vexy_json %s\n", vexy_json_version());

    VexyJsonParseResult r = vexy_json_parse("{a: 1, b: [true, 'x'],}");
    if (r.error) {
        fprintf(stderr, "error: %s\n", r.error);
    } else {
        printf("%s\n", r.json);
    }
    vexy_json_free_result(r);

    VexyJsonParserOptions options = vexy_json_default_options();
    options.allow_unquoted_keys = false;
    r = vexy_json_parse_with_options("{a: 1}", &options);
    printf("error: %s\n", r.error);
    /* error: Expected string key but found UnquotedString at position 2 */
    vexy_json_free_result(r);
    return 0;
}
```

## Functions

| Function | Purpose |
| --- | --- |
| `const char* vexy_json_version(void)` | Library version; do not free |
| `VexyJsonParserOptions vexy_json_default_options(void)` | Options with every extension on |
| `VexyJsonParseResult vexy_json_parse(const char* input)` | Parse with default options |
| `VexyJsonParseResult vexy_json_parse_with_options(const char* input, const VexyJsonParserOptions* options)` | Parse with options; `NULL` options means defaults |
| `VexyJsonDetailedResult vexy_json_parse_detailed(const char* input, const VexyJsonParserOptions* options)` | Same parse, in the detailed result type |
| `VexyJsonParser vexy_json_parser_new(const VexyJsonParserOptions* options)` | Create a reusable parser handle |
| `VexyJsonParseResult vexy_json_parser_parse(VexyJsonParser parser, const char* input)` | Parse with a handle's options |
| `void vexy_json_parser_free(VexyJsonParser parser)` | Free a handle |
| `void vexy_json_free_result(VexyJsonParseResult result)` | Free both strings of a result |
| `void vexy_json_free_detailed_result(VexyJsonDetailedResult result)` | Free a detailed result |

`VexyJsonParserOptions` has the fields listed in [Parser options](options.md).
`max_depth` and `max_repairs` are `uint32_t`; the other fields are `bool`.

## Results and memory

A `VexyJsonParseResult` holds two strings:

- `json`: the parsed value, or `NULL` on error
- `error`: the error message, or `NULL` on success

Exactly one of them is set. A `NULL` input gives the error `Input is null`;
input that is not UTF-8 gives `Invalid UTF-8 input`.

The library allocates both strings. Release every result with
`vexy_json_free_result` exactly once, and do not use its pointers afterwards.
Do not pass them to `free()`.

A parser handle is not thread-safe. Create one handle per thread, and free it
with `vexy_json_parser_free`.

## Output format

The `json` string is produced by Serde's derived serialization of the Rust
`Value` enum, so it keeps the enum variant names instead of plain JSON:

```text
{"Object":{"a":{"Number":{"Integer":1}},"b":{"Array":[{"Bool":true},{"String":"x"}]}}}
```

Each value is an object with one key naming its kind: `Null`, `Bool`,
`Number` (containing `Integer` or `Float`), `String`, `Array` or `Object`.
Read it with a JSON library and unwrap one level per value. Member order in
`Object` is not kept.

## Current limitations

- `vexy_json_parse_detailed` does not track repairs yet: `repairs` is always
  `NULL` and `repair_count` is `0`.
- The repair fields of `VexyJsonParserOptions` have no effect; the C functions
  never repair input.
- `crates/c-api/include/vexy_json.hpp`, a C++17 wrapper, refers to type names
  that the C header does not define (for example `vexy_json_parse_result`), so
  it does not compile with the current header. From C++, include
  `vexy_json.h` directly; it has `extern "C"` guards.
