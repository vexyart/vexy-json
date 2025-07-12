---
nav_title: Forgiving Features
nav_order: 5
---

a: 1, b: 2

# Forgiving Features

`vexy_json` is a forgiving JSON parser, handling common deviations from strict JSON (RFC 8259). Below are the supported forgiving features, enhanced in v2.0.0 with streaming, parallel processing, and plugin capabilities:

## Comments

- Single-line: `// ...` and `# ...`
- Multi-line: `/* ... */`

Comments are ignored anywhere whitespace is allowed.

**Example:**

```json
{
  // This is a single-line comment
  age: 30, # Another single-line comment
  /* Multi-line
     comment */
  name: "Alice"
}
```

## Unquoted Keys

Object keys can be unquoted if they are valid identifiers.

```json
{ name: "vexy_json", version: 1.0 }
```

## Trailing Commas

Trailing commas are allowed in arrays and objects.

```json
{
  a: 1,
  b: 2,
}
```

## Implicit Top-Level Objects and Arrays

You can omit brackets for top-level arrays or objects:

```json
apple, banana, cherry
# Interpreted as ["apple", "banana", "cherry"]


# Interpreted as {"a": 1, "b": 2}
```

## Newlines as Comma Separators

When enabled, newlines can act as value separators, like commas, in arrays and objects.

```json
[
  1
  2
  3
]
```

```json
{
  key1: "value1"
  key2: "value2"
}
```

## Extended Number Formats

- Hexadecimal: `0xFF`
- Octal: `0o77`
- Binary: `0b1010`
- Underscores: `1_000_000`

## Single-Quoted Strings

Both single and double quotes are supported for strings.

```json
{ key: 'value', other: "also ok" }
```

## Strict Mode

All forgiving features can be disabled for strict RFC 8259 compliance.

These forgiving features make `vexy_json` a flexible parser for configurations, data files, and other scenarios where strict JSON adherence might be relaxed.

## New in v2.0.0: Advanced Features

### Streaming Parser
Process large JSON files incrementally:
- Memory-efficient parsing for gigabyte-sized files
- Event-driven API for fine-grained control
- Support for incremental data feeds

### Parallel Processing
Leverage multiple CPU cores:
- Automatic work distribution across threads
- Intelligent chunk boundary detection
- Linear scalability with core count

### Plugin Architecture
Extend vexy_json with custom functionality:
- Transform values during parsing
- Add custom validation rules
- Implement domain-specific logic

### NDJSON Support
Native support for newline-delimited JSON:
- Process streaming data sources
- Handle log files and data exports
- Efficient line-by-line parsing

For detailed API documentation on these features, see the [API Reference](api/).

