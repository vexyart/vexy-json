# Repairing incomplete input

Text from language models, interrupted downloads and log tails often stops in
the middle of a JSON document. Vexy JSON has two separate repair mechanisms:
the `--repair` option of the command-line tool, and the `parse_with_fallback`
function of the Rust library. They follow different rules.

## The CLI `--repair` option

`--repair` only adds closing brackets and braces at the end of the input. It
never invents keys, values or string contents. If the input cannot be closed
without inventing data, the CLI refuses and writes nothing to standard output.

### What happens

Take this truncated input:

```text
{"a": [1, {"b": 2
```

1. The normal parse fails because the input ends inside three open
   structures.
2. The CLI scans the input from the start. It skips quoted strings (single or
   double quotes, with `\` escapes) and comments, and keeps a stack of the
   `{` and `[` that are still open. Here the stack is `{`, `[`, `{`.
3. It checks that closing is safe (see the list below).
4. It appends the matching closers in reverse order: `}`, `]`, `}`.
5. It parses the result again. If that parse fails, it reports an unsafe
   repair and writes nothing.
6. It writes the value and prints a warning to standard error:

```bash
printf '{"a": [1, {"b": 2' | vexy-json -r -c
```

```text
{"a":[1,{"b":2}]}
```

```text
WARNING: unexpected EOF at byte 17; emitted partial/lossy JSON.
Repair actions: inserted closing '}' at byte 17; inserted closing ']' at byte 17; inserted closing '}' at byte 17.
Missing upstream bytes were not recovered; use --lms when complete data matters.
```

The exit status is `0`. With `--validate`, the CLI prints
`Valid JSON (partial/lossy repair)` instead of the value.

### When the CLI refuses

The CLI does not repair, and exits with status `1`, when:

| Situation | Example input | Reason given |
| --- | --- | --- |
| Input ends inside a string | `{"a":"tru` | closing it would fabricate string contents |
| Last token is incomplete | `[1, 2, tr` | the final token is incomplete |
| Input ends after a comma or colon | `{"a":1,` | no complete value can be closed safely |
| Input ends inside a comment | `[1, 2 // note` | closing would change the comment |
| A closer does not match its opener | `[1, 2}` or `{"a": 1}}` | mismatched or unmatched closing delimiter |
| Nothing is left open | `{"a": 1} x` | input did not end inside an open JSON structure |
| Input ends inside a block comment | `[1, 2 /* x` | EOF occurred inside a block comment |

The input is safe to close only when its last non-blank character is a
bracket or brace, a quote that ends a string, the last digit of a complete
number, or the end of `true`, `false` or `null`.

The message names the byte where the problem was found:

```text
Error:
Cannot safely repair <stdin> at byte 9: EOF occurred inside a quoted string; closing it would fabricate string contents
```

### Limits

- `--repair` works on a single document. It cannot be combined with
  `--ndjson`, `--watch`, `--parallel-parse` or `--lms`.
- The repaired value is incomplete by definition: whatever came after the
  cut is lost. Treat it as partial data.
- When the source is the `lms` command, use `--lms` instead. It captures the
  whole output through a regular file, so nothing is cut off in the first
  place.

## The Rust `parse_with_fallback` function

`vexy_json_core::parse_with_fallback` tries up to three parsers in turn and
tells you which one succeeded:

1. **Fast tier.** `serde_json` parses the input as strict JSON.
2. **Forgiving tier.** If that fails, the Vexy JSON parser runs with the given
   `ParserOptions`.
3. **Repair tier.** If that fails and `enable_repair` is `true`, a bracket
   repairer (`JsonRepairer`) balances brackets and the result is parsed again.

```rust
use vexy_json::ParserOptions;
use vexy_json_core::{parse_with_fallback, ParsingTier};

let result = parse_with_fallback(r#"{"a": [1, 2"#, ParserOptions::default());

assert!(result.errors.is_empty());
assert!(matches!(result.parsing_tier, ParsingTier::Repair));
println!("{}", result.value);          // {"a": [1, 2]}
println!("{:?}", result.repairs.len()); // number of RepairAction records
```

The function returns an `EnhancedParseResult<Value>` with four public fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `value` | `Value` | The parsed value; `Value::Null` when every tier failed |
| `errors` | `Vec<Error>` | Empty on success |
| `repairs` | `Vec<RepairAction>` | Repairs that were applied |
| `parsing_tier` | `ParsingTier` | `Fast`, `Forgiving` or `Repair` |

Each `RepairAction` has `action_type`, `position`, `original`, `replacement`
and `description` fields. For `{"a": 1` the repairer records one action that
inserts `}`.

Always check `errors`. The repairer can record an action and still fail: for
`{"a": [1, 2}` it inserts brackets, but the result does not parse, so `value`
is `Value::Null` and `errors` holds one error.

The `vexy-json` CLI also calls `parse_with_fallback`, but with
`enable_repair` set to `false`, so its third tier never runs; `--repair` is the
CLI's own mechanism described above.

## Python, JavaScript and C

The Python functions, the WebAssembly functions and the C functions call
`parse` or `parse_with_options`, which never repair. Their `enable_repair`
arguments are accepted but have no effect. To repair input from these
languages, run it through `vexy-json --repair` first.
