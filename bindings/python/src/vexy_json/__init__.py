"""
vexy_json - A forgiving JSON parser for Python

This module provides a fast, forgiving JSON parser that can handle:
- Comments (// and /* */)
- Trailing commas
- Unquoted keys
- Single quotes
- Missing commas (newline as comma)
- Automatic error repair

Example:
    >>> import vexy_json
    >>> data = vexy_json.parse('{ unquoted: true, /* comment */ trailing: "comma", }')
    >>> print(data)
    {'unquoted': True, 'trailing': 'comma'}
"""

from .vexy_json import (
    parse,
    parse_with_options,
    dumps,
    load,
    dump,
    version,
    Parser,
    Options,
    ParseError,
    ParseResult,
    Repair,
    __version__,
)

__all__ = [
    "parse",
    "parse_with_options",
    "dumps",
    "load",
    "dump",
    "version",
    "Parser",
    "Options",
    "ParseError",
    "ParseResult",
    "Repair",
    "__version__",
]

# Convenience functions that mirror standard json module API
loads = parse  # Alias for compatibility with json module