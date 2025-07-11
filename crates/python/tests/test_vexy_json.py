#!/usr/bin/env python3
# Test file for vexy_json renaming

import vexy_json
from vexy_json import VexyJSONParser, VexyJSONConfig


class VexyJSONWrapper:
    """A wrapper for Vexy JSON functionality"""

    def __init__(self):
        self.parser = VexyJSONParser()
        self.config = VexyJSONConfig()

    def parse(self, data):
        # Parse vexy_json data
        return self.parser.parse(data)


# Test with a string that contains "vexy_json"
test_string = "This is a vexy_json parser"
print(f"Vexy JSON version: {vexy_json.__version__}")
