# Python

The `vexy-json` package on PyPI wraps the Rust parser in a compiled extension
module. Import it as `vexy_json`. It needs Python 3.8 or later. See
[Installation](installation.md).

## Parsing a string

```python
import vexy_json

value = vexy_json.parse("{name: 'Vexy', enabled: true,}")
assert value == {"name": "Vexy", "enabled": True}
```

`vexy_json.parse(text)` and its alias `vexy_json.loads(text)` take a `str`
and return Python objects:

| JSON | Python |
| --- | --- |
| object | `dict` |
| array | `list` |
| string | `str` |
| integer that fits in 64 bits | `int` |
| other number | `float` |
| `true`, `false` | `True`, `False` |
| `null` | `None` |

Input that cannot be parsed raises `ValueError` with the parser's message:

```python
vexy_json.parse("{a: ")
# ValueError: Parse error: Expected , or } or newline but found Eof at position 4
```

## Parsing with options

`parse_with_options` takes the input and any of the
[parser options](options.md) as keyword arguments:

```python
vexy_json.parse_with_options("key: value", implicit_top_level=True)
# {'key': 'value'}

vexy_json.parse_with_options('{"a": 1, }', allow_trailing_commas=False)
# ValueError: Parse error: Trailing comma at position 10
```

The keywords and their defaults are `allow_comments=True`,
`allow_trailing_commas=True`, `allow_unquoted_keys=True`,
`allow_single_quotes=True`, `implicit_top_level=True`,
`newline_as_comma=True`, `max_depth=128`, `enable_repair=True`,
`max_repairs=100`, `fast_repair=False` and `report_repairs=True`. The three
repair keywords have no effect on these functions; see
[Repairing incomplete input](repair.md).

## Checking input

`is_valid(text)` returns `True` if `parse` would succeed and `False`
otherwise. It never raises.

```python
vexy_json.is_valid('{"valid": true}')  # True
vexy_json.is_valid('{"key":}')         # False
```

## Files

`load(fp, **options)` reads all text from a file-like object with `.read()`
and parses it. Without keyword arguments it behaves like `parse`; with them,
like `parse_with_options`.

```python
with open("settings.conf", encoding="utf-8") as fp:
    settings = vexy_json.load(fp, allow_comments=True)
```

## Writing JSON

`dumps(obj, indent=None)` converts a Python object to a JSON string.
`dump(obj, fp, indent=None)` writes that string to a file-like object with
`.write()`.

```python
vexy_json.dumps({"key": "value", "number": 42})
# '{"key": "value", "number": 42}'

print(vexy_json.dumps({"key": "value"}, indent=2))
# {
#   "key": "value"
# }
```

Supported input types are `dict` with `str` keys, `list`, `str`, `int`,
`float`, `bool` and `None`. Other types, including tuples, raise `TypeError`.

!!! warning "Escaping in compact output"
    Without `indent`, `dumps` does not escape string contents. A string that
    contains `"` or a newline produces invalid JSON. With `indent`, strings are
    escaped. Use the standard library's `json.dumps` when your data may contain
    such characters.

## Streaming

`StreamingParser` reads values one at a time from a file-like object. Its
constructor takes the same keyword options as `parse_with_options`, and it
works as a context manager.

`parse_lines(fp)` reads newline-delimited JSON: one value per line.

```python
import io
import vexy_json

data = io.StringIO('{"a": 1}\n{b: 2}\n')
with vexy_json.StreamingParser() as parser:
    for item in parser.parse_lines(data):
        print(item)
# {'a': 1}
# {'b': 2}
```

`parse_stream(fp)` is meant for values that follow each other without line
structure. In version 1.5.13 it returned only the first value of
`{"a": 1} {b: 2}`, so prefer `parse_lines` when your data has one value per
line.

## NumPy and pandas

These functions import NumPy or pandas when called. If the library is missing,
they raise `ValueError` with an installation hint.

| Function | Input | Result |
| --- | --- | --- |
| `loads_numpy(text, dtype=None)` | a JSON array | `numpy.ndarray` |
| `loads_numpy_zerocopy(text, dtype=None)` | a JSON array | `numpy.ndarray` |
| `loads_dataframe(text)` | a JSON array of objects | `pandas.DataFrame` |

```python
arr = vexy_json.loads_numpy("[1, 2, 3]")
arr.dtype  # dtype('float64')

arr = vexy_json.loads_numpy("[1, 2, 3]", dtype="float32")
arr.dtype  # dtype('float32')

df = vexy_json.loads_dataframe("[{a: 1, b: 'x'}, {a: 2, b: 'y'}]")
df.shape   # (2, 2)
```

Input that is not an array raises
`ValueError: Input must be a JSON array for NumPy conversion`.

## Module attributes

| Name | Content |
| --- | --- |
| `__version__` | Package version, for example `"1.5.13"` |
| `__author__` | `"Adam Twardoch"` |
| `__description__` | Short description of the bindings |
| `__all__` | The public names listed on this page |

The package ships type stubs (`__init__.pyi`) and a `py.typed` marker, so type
checkers see the function signatures.
