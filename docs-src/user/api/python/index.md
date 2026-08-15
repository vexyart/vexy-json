---
nav_title: Python API
nav_order: 1
---

# Python API

Install the `vexy-json` distribution and import `vexy_json`:

```bash
uv add vexy-json
```

```python
import vexy_json
```

The package ships type information (`py.typed`) and a native Rust extension.

## Parsing

### `parse(input)` / `loads(input)`

Parse a string with forgiving defaults. Both names return JSON-compatible
Python values: `dict`, `list`, `str`, `int`, `float`, `bool`, or `None`.

```python
value = vexy_json.loads("{name: 'Ada', active: true,}")
assert value == {"name": "Ada", "active": True}
```

Invalid input raises `ValueError`.

### `parse_with_options(input, **options)`

| Option | Default | Purpose |
| --- | ---: | --- |
| `allow_comments` | `True` | Accept line and block comments |
| `allow_trailing_commas` | `True` | Accept a final comma |
| `allow_unquoted_keys` | `True` | Accept identifier-like object keys |
| `allow_single_quotes` | `True` | Accept single-quoted strings |
| `implicit_top_level` | `True` | Infer a top-level object or array |
| `newline_as_comma` | `True` | Treat line boundaries as separators |
| `max_depth` | `128` | Bound nesting |
| `enable_repair` | `True` | Enable structural repair |
| `max_repairs` | `100` | Bound repair attempts |
| `fast_repair` | `False` | Prefer faster repair heuristics |
| `report_repairs` | `True` | Retain repair reporting behavior |

```python
strict = vexy_json.parse_with_options(
    '{"answer": 42}',
    allow_comments=False,
    allow_trailing_commas=False,
    allow_unquoted_keys=False,
    allow_single_quotes=False,
    implicit_top_level=False,
    newline_as_comma=False,
    enable_repair=False,
)
```

### `is_valid(input)`

Return `True` when the string parses under Vexy JSON defaults. This checks
syntax, not an application schema.

## Serialization

### `dumps(obj, indent=None)`

Serialize a JSON-compatible Python value to text. Set `indent` for readable
output.

### `load(fp, **options)` and `dump(obj, fp, indent=None)`

Read from or write to a text or binary file-like object:

```python
from pathlib import Path
import vexy_json

with Path("input.json").open() as source:
    value = vexy_json.load(source, allow_comments=True)

with Path("output.json").open("w") as destination:
    vexy_json.dump(value, destination, indent=2)
```

## Streaming and NDJSON

`StreamingParser.parse_lines(fp)` yields one parsed value per NDJSON line.
`parse_stream(fp)` provides the package's stream-oriented interface. Use the
context manager so resources are released predictably:

```python
import vexy_json

with open("events.ndjson") as source:
    with vexy_json.StreamingParser() as parser:
        for event in parser.parse_lines(source):
            process(event)
```

## Optional data integrations

`loads_numpy`, `loads_numpy_zerocopy`, and `loads_dataframe` require their
corresponding NumPy or pandas runtime dependency. Import failures are reported
when the optional library is unavailable.

## Package identity

These names intentionally differ:

- PyPI distribution and installer name: `vexy-json`
- Python module and native extension namespace: `vexy_json`

Python packaging normalizes hyphens and underscores for resolution, but the
hyphenated metadata keeps the public project name consistent across PyPI,
crates.io, the CLI, and this documentation.
