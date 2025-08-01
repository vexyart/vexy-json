#!/usr/bin/env python3
"""Basic usage examples for vexy_json"""

import vexy_json


def main():
    print("vexy_json - Forgiving JSON Parser Examples")
    print("=" * 50)

    # Example 1: Basic parsing
    print("\n1. Basic JSON parsing:")
    json_str = '{"name": "Alice", "age": 30}'
    data = vexy_json.parse(json_str)
    print(f"Input:  {json_str}")
    print(f"Output: {data}")

    # Example 2: Comments
    print("\n2. JSON with comments:")
    json_with_comments = """
    {
        // User information
        "name": "Bob",
        "age": 25,
        /* Additional details */
        "city": "New York"
    }
    """
    data = vexy_json.parse(json_with_comments)
    print(f"Output: {data}")

    # Example 3: Trailing commas
    print("\n3. JSON with trailing commas:")
    json_trailing = '{"items": [1, 2, 3,], "total": 3,}'
    data = vexy_json.parse(json_trailing)
    print(f"Input:  {json_trailing}")
    print(f"Output: {data}")

    # Example 4: Unquoted keys
    print("\n4. JSON with unquoted keys:")
    json_unquoted = '{name: "Charlie", age: 35, active: true}'
    data = vexy_json.parse(json_unquoted)
    print(f"Input:  {json_unquoted}")
    print(f"Output: {data}")

    # Example 5: Single quotes
    print("\n5. JSON with single quotes:")
    json_single = "{'name': 'David', 'language': 'Python'}"
    data = vexy_json.parse(json_single)
    print(f"Input:  {json_single}")
    print(f"Output: {data}")

    # Example 6: Implicit object
    print("\n6. Implicit top-level object:")
    json_implicit = 'name: "Eve", role: "developer", experience: 5'
    data = vexy_json.parse(json_implicit)
    print(f"Input:  {json_implicit}")
    print(f"Output: {data}")

    # Example 7: Mixed features
    print("\n7. Mixed forgiving features:")
    json_mixed = """
    {
        // Config file
        server: 'localhost',
        port: 8080,
        features: {
            auth: true
            cache: false  // Missing comma
        },
        database: {
            host: "db.example.com",
            name: 'myapp',  // Trailing comma
        }
    }
    """
    data = vexy_json.parse(json_mixed)
    print(f"Output: {data}")

    # Example 8: Using Options
    print("\n8. Using custom options (strict mode):")
    strict_opts = vexy_json.Options.strict()
    try:
        # This will fail with strict options
        vexy_json.parse_with_options("{unquoted: true}", strict_opts)
    except ValueError as e:
        print(f"Expected error with strict mode: {e}")

    # Example 9: Serialization
    print("\n9. Serializing Python objects:")
    data = {"users": ["Alice", "Bob"], "count": 2, "active": True}
    json_output = vexy_json.dumps(data, indent=2)
    print(f"Serialized:\n{json_output}")

    # Example 10: Parser instance
    print("\n10. Using Parser instance:")
    parser = vexy_json.Parser()
    results = []
    for json_str in ['{"a": 1}', "[1, 2, 3]", '"hello"']:
        results.append(parser.parse(json_str))
    print(f"Parsed multiple inputs: {results}")


if __name__ == "__main__":
    main()
