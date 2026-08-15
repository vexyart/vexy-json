# this_file: crates/python/tests/test_vexy_json.py
"""Canonical public API and package-version tests."""

from importlib.metadata import version

import vexy_json


def test_runtime_version_matches_installed_metadata():
    """Require the native runtime and wheel metadata to expose one version."""
    assert vexy_json.__version__ == version("vexy_json")


def test_public_parser_handles_representative_forgiving_input():
    """Exercise the exported parser instead of stale compatibility classes."""
    assert vexy_json.parse("{answer: 42,}") == {"answer": 42}


def test_standard_loads_alias_uses_the_public_parser():
    """Keep the standard-library-compatible loads alias operational."""
    assert vexy_json.loads("['vexy', 'json',]") == ["vexy", "json"]
