#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e
cd "$(dirname "$0")/.."
echo "Starting build process for vexy_json..."

{
    echo "Building the vexy_json project..."
    # Build the project in release mode for optimized binaries
    /Users/adam/.cargo/bin/cargo build --release

    echo "Running tests..."
    # Run all unit and integration tests
    /Users/adam/.cargo/bin/cargo test

    echo "Running linter (clippy)..."
    # Run clippy to catch common mistakes and improve code quality
    # Note: Currently allowing missing_docs warnings as there are 80 pending
    /Users/adam/.cargo/bin/cargo clippy -- -D warnings -A missing_docs

    echo "Checking code formatting..."
    # Check if code is formatted according to rustfmt rules
    /Users/adam/.cargo/bin/cargo fmt --check

    echo "Running examples..."
    # Test the example programs
    /Users/adam/.cargo/bin/cargo run --example test_single_quote
    /Users/adam/.cargo/bin/cargo run --example test_implicit_array

    echo "Building documentation..."
    # Build the documentation
    /Users/adam/.cargo/bin/cargo doc --no-deps

    echo "Build and verification complete."
    echo ""
    echo "Library built at: ./target/release/libvexy_json.rlib"
    echo "Documentation at: ./target/doc/vexy_json/index.html"
    echo ""
    echo "To use vexy_json in your project, add to Cargo.toml:"
    echo '  vexy_json = { path = "'$(pwd)'" }'
    echo ""
    echo "Example usage:"
    echo "  use vexy_json::parse;"
    echo "  let value = parse(\"'hello', 'world'\").unwrap();"

} >build.log.txt 2>&1

echo "Build log created in: build.log.txt"
echo ""
echo "Quick test - parsing implicit array:"
echo "'a', 'b', 'c'" | /Users/adam/.cargo/bin/cargo run --example test_implicit_array 2>/dev/null | grep -A1 "'a'" || true
