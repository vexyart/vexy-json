---
nav_title: Forgiving Syntax
nav_order: 3
---

# Forgiving syntax

Strict JSON is always accepted. The default parser additionally accepts the
following extensions.

| Extension | Example | Option |
| --- | --- | --- |
| Line comments | `// note` | `allow_comments` |
| Block comments | `/* note */` | `allow_comments` |
| Trailing commas | `[1, 2,]` | `allow_trailing_commas` |
| Unquoted keys | `{port: 8080}` | `allow_unquoted_keys` |
| Single quotes | `{'mode': 'dev'}` | `allow_single_quotes` |
| Implicit top level | `port: 8080` | `implicit_top_level` |
| Newline separators | `1\n2\n3` | `newline_as_comma` |
| Extended numbers | `0xff`, `0o77`, `0b10`, `1_000` | enabled parser support |

## Comments

Comments are discarded rather than returned in the parsed value:

```text
{
  // deployment target
  region: 'eu-central',
  /* keep this low in development */
  workers: 2,
}
```

## Implicit containers

A top-level sequence of key/value pairs becomes an object:

```text
host: localhost
port: 8080
```

A top-level sequence of values becomes an array:

```text
'red'
'green'
'blue'
```

Explicit `{}` and `[]` remain preferable when the input could be ambiguous.

## Numbers

Extended integers accept hexadecimal, octal, and binary prefixes. Underscores
may separate digits for readability:

```text
permissions: 0o755
mask: 0xff
flags: 0b1010
budget: 1_000_000
```

## Repair is bounded

Repair handles selected structural mistakes; it is not a promise that every
broken string can be guessed correctly. Incomplete strings, invalid UTF-8,
partial tokens, or semantically ambiguous input may still fail. Configure
`max_repairs` and `max_depth` for untrusted inputs, or disable repair entirely.

## Strict parsing

For protocol boundaries, signatures, or canonical files, disable all forgiving
options and repair. See [Getting started](getting-started.md#control-accepted-syntax)
for complete Rust and Python examples.
