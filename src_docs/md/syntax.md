# Accepted syntax

Vexy JSON accepts everything that RFC 8259 JSON allows. This page lists the
extensions it accepts on top of that, with the default
[parser options](options.md). Each example shows the input and the compact
JSON that `vexy-json -c` prints for it. Objects are stored in a hash map, so
the real output can list keys in a different order.

## Comments

Line comments start with `//` or `#` and run to the end of the line. Block
comments are enclosed in `/*` and `*/`.

```text
{
  // line comment
  a: 1, # another line comment
  /* block
     comment */
  b: 2
}
```

```json
{"a":1,"b":2}
```

Option: `allow_comments`. CLI: `--no-comments`.

## Trailing commas

A comma after the last element of an array or the last member of an object is
ignored.

```text
{"items": [1, 2, 3,], "done": true,}
```

```json
{"items":[1,2,3],"done":true}
```

Option: `allow_trailing_commas`. CLI: `--no-trailing-commas`.

## Unquoted keys

Object keys can be written without quotes. Letters, digits, `_`, `$` and `-`
work in unquoted keys.

```text
{name: "Vexy", $id: 7, max-size: 10}
```

```json
{"name":"Vexy","$id":7,"max-size":10}
```

Option: `allow_unquoted_keys`. CLI: `--no-unquoted-keys`.

## Unquoted string values

A single bare word in value position becomes a string. Only `true`, `false`
and `null` keep their JSON meaning.

```text
{mode: fast, level: hello-world, missing: nil}
```

```json
{"mode":"fast","level":"hello-world","missing":"nil"}
```

A value made of several words separated by spaces is rejected; quote it
instead.

## Single-quoted strings

Strings can be enclosed in single quotes. Inside them, `\'` is an escaped
quote and double quotes need no escaping.

```text
{'greeting': 'it\'s "fine"'}
```

```json
{"greeting":"it's \"fine\""}
```

Option: `allow_single_quotes`. CLI: `--no-single-quotes`.

## Implicit top-level object or array

The outer braces or brackets can be left out. Key-value pairs at the top
level form an object; a comma-separated list of values forms an array.

```text
host: 'localhost', port: 8080
```

```json
{"host":"localhost","port":8080}
```

```text
1, 2, 3
```

```json
[1,2,3]
```

Option: `implicit_top_level`. CLI: `--no-implicit-top-level`.

## Newlines as separators

A newline can take the place of a comma between array elements and between
object members.

```text
[
  "red"
  "green"
]
```

```json
["red","green"]
```

Combined with implicit top-level objects, this reads a simple `key: value`
file:

```text
a: 1
b: 2
```

```json
{"a":1,"b":2}
```

Option: `newline_as_comma`. CLI: `--no-newline-as-comma`.

## Number formats

Besides standard JSON numbers, the lexer reads:

| Input | Value | Notes |
| --- | --- | --- |
| `0x1F` | `31` | hexadecimal |
| `0o17` | `15` | octal |
| `0b101` | `5` | binary |
| `1_000` | `1000` | underscores between digits |
| `.5` | `0.5` | no leading zero |
| `1.` | `1` | no digits after the point |
| `+3` | `3` | explicit plus sign |
| `-0x10` | `-16` | sign on a prefixed number |

Integers that fit in a signed 64-bit integer become `Number::Integer`; other
numbers become `Number::Float`. `NaN` and `Infinity` are not numbers in Vexy
JSON: as bare words they become the strings `"NaN"` and `"Infinity"`.

These formats have no on/off option.

## Empty array slots

An empty slot between commas becomes `null`:

```text
[1,,2]
```

```json
[1,null,2]
```

## Duplicate keys

When a key appears twice in one object, the last value wins:

```text
{a: 1, a: 2}
```

```json
{"a":2}
```

## What is still rejected

- More than one top-level value, such as `{"a":1}{"b":2}`. Use NDJSON mode
  (`--ndjson` or `NdJsonParser`) for one value per line.
- Nesting deeper than `max_depth` (default 128) levels.
- Input that ends inside an object, array or string. The CLI can close some
  truncated input with `--repair`; see [Repairing incomplete input](repair.md).
- Mismatched brackets, such as `[1, 2}`.

The library parses empty input to `null`. The CLI prints `No input provided`
for empty standard input and exits with status `0`.
