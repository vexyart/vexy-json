#!/bin/bash -eu
# this_file: oss-fuzz/build.sh

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Navigate to the project directory
cd $SRC/vexy_json

# Build the project
cargo build --release

# Build fuzz targets
cd fuzz
cargo fuzz build

# Copy fuzz targets to the output directory
for target in $(cargo fuzz list); do
    cp target/x86_64-unknown-linux-gnu/release/$target $OUT/
done

# Copy corpus and dictionary files
if [ -d "corpus" ]; then
    for target in $(cargo fuzz list); do
        if [ -d "corpus/$target" ]; then
            cp -r corpus/$target $OUT/${target}_seed_corpus
        fi
    done
fi

# Copy dictionary files if they exist
if [ -f "dictionary.txt" ]; then
    cp dictionary.txt $OUT/
fi

# Create a comprehensive JSON dictionary for better fuzzing
cat > $OUT/json.dict << 'EOF'
# JSON structure tokens
"{"
"}"
"["
"]"
":"
","
"\""
"'"
"null"
"true"
"false"

# JSON escape sequences
"\n"
"\r"
"\t"
"\\"
"\""
"\'"
"\/"
"\b"
"\f"
"\u0000"

# Common JSON values
"0"
"1"
"-1"
"0.0"
"1.0"
"-1.0"
"1e10"
"-1e10"
"1.5e-10"
""
"string"
"test"
"key"
"value"
"data"
"items"
"id"
"name"

# Vexy JSON-specific extensions
"//"
"/*"
"*/"
"unquoted_key"
"trailing_comma"
"single_quotes"

# Common patterns
"key:value"
"\"key\":\"value\""
"'key':'value'"
"key:123"
"key:true"
"key:false"
"key:null"
"key:[1,2,3]"
"key:{\"nested\":\"value\"}"
EOF