# Vexy JSON for Python

Parse configuration files, hand-edited data, and LLM-generated JSON without
pre-cleaning the text first. Vexy JSON accepts strict JSON plus comments,
trailing commas, unquoted keys, single quotes, implicit containers, newline
separators, and extended number formats. A Rust engine performs the parsing and
returns ordinary Python values.

## Install

```bash
uv add vexy-json
# or: python -m pip install vexy-json
```

The PyPI distribution is named `vexy-json`; Python imports use `vexy_json`.

## Parse

```python
import vexy_json

settings = vexy_json.parse("""
{
  // readable configuration
  environment: 'preview',
  retries: 3,
  regions: ['eu', 'us',],
}
""")

assert settings["environment"] == "preview"
```

Use keyword options when an input boundary must be stricter:

```python
strict = vexy_json.parse_with_options(
    '{"ready": true}',
    allow_comments=False,
    allow_trailing_commas=False,
    allow_unquoted_keys=False,
    allow_single_quotes=False,
    implicit_top_level=False,
    newline_as_comma=False,
    enable_repair=False,
)
```

The package also provides `load`, `loads`, `dump`, `dumps`, `is_valid`, an
NDJSON-capable `StreamingParser`, and optional NumPy and pandas conversion
helpers. It ships inline typing information for editors and type checkers.

## Documentation

Complete guides, option tables, Rust documentation, and troubleshooting are at
[vexy.dev/vexy-json](https://vexy.dev/vexy-json/).

Source and issue tracking live at
[github.com/vexyart/vexy-json](https://github.com/vexyart/vexy-json).

Licensed under MIT or Apache-2.0, at your option.
