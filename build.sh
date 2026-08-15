#!/usr/bin/env bash
# this_file: build.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
LOG="$ROOT/build.log.txt"
TEMP_DIR=""
BUILD_COMPLETED=0

cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf -- "$TEMP_DIR"
    fi
}

finish() {
    local status=$?
    local message
    trap - EXIT
    if (( status == 0 && BUILD_COMPLETED == 1 )); then
        message='BUILD SUCCESS: Rust workspace and canonical Python package passed.'
    else
        if (( status == 0 )); then
            status=1
        fi
        message="BUILD FAILURE: exit status $status; inspect $LOG"
    fi
    if ! printf '\n%s\n' "$message" >>"$LOG"; then
        if (( status == 0 )); then
            status=1
        fi
        printf '\nBUILD FAILURE: could not append final status to %s\n' "$LOG" >&2 || true
    else
        printf '\n%s\n' "$message" || {
            if (( status == 0 )); then
                status=1
            fi
        }
    fi
    cleanup
    exit "$status"
}

handle_signal() {
    local status=$1
    trap - INT TERM
    exit "$status"
}

require_tool() {
    local tool=$1
    command -v "$tool" >/dev/null 2>&1 || {
        printf 'Required tool is missing: %s\n' "$tool" >&2
        return 1
    }
}

verify_layout() {
    local required
    for required in \
        Cargo.toml \
        crates/cli/Cargo.toml \
        crates/python/Cargo.toml \
        crates/python/pyproject.toml \
        crates/python/tests \
        mkdocs.yml \
        requirements-docs.txt \
        scripts/release-manifests.py \
        scripts/check-python-artifacts.py; do
        [[ -e "$ROOT/$required" ]] || {
            printf 'Canonical project path is missing: %s\n' "$required" >&2
            return 1
        }
    done
}

record_tree() {
    local destination=$1
    git -C "$ROOT" status --porcelain=v1 --untracked-files=all >"$destination.status"
    git -C "$ROOT" diff --no-ext-diff --binary HEAD -- >"$destination.diff"
}

verify_tree_unchanged() {
    local before=$1
    local after=$2
    if ! cmp -s "$before.status" "$after.status" || ! cmp -s "$before.diff" "$after.diff"; then
        printf 'Build changed the checkout. Before/after status follows.\n' >&2
        diff -u "$before.status" "$after.status" >&2 || true
        return 1
    fi
}

python_gate() {
    local expected=$1
    local dist="$TEMP_DIR/python-dist"
    local venv="$TEMP_DIR/python-venv"
    local python="$venv/bin/python"
    local artifact_list="$TEMP_DIR/python-artifacts.txt"
    local wheel

    mkdir -p "$dist"
    (
        cd "$ROOT/crates/python"
        uv build --no-sources --out-dir "$dist"
    )
    python3 "$ROOT/scripts/check-python-artifacts.py" "$dist" "$expected" >"$artifact_list"
    wheel="$(sed -n '1p' "$artifact_list")"
    [[ -f "$wheel" ]] || {
        printf 'Python wheel inspection did not return a file.\n' >&2
        return 1
    }

    uv venv --clear --python python3 "$venv"
    uv pip install --python "$python" "$wheel" pytest
    "$python" -m pytest "$ROOT/crates/python/tests" -q
    "$python" - "$expected" <<'PY'
import sys

import vexy_json

expected = sys.argv[1]
assert vexy_json.__version__ == expected, (vexy_json.__version__, expected)
assert vexy_json.parse("{answer: 42,}") == {"answer": 42}
print(f"Python smoke passed: vexy_json {vexy_json.__version__}")
PY
}

documentation_gate() {
    local site="$TEMP_DIR/docs-site"
    uvx --with-requirements "$ROOT/requirements-docs.txt" \
        mkdocs build --strict --config-file "$ROOT/mkdocs.yml" --site-dir "$site"
    [[ -f "$site/index.html" ]] || {
        printf 'Documentation build did not create index.html.\n' >&2
        return 1
    }
    printf 'Documentation build passed: %s\n' "$ROOT/mkdocs.yml"
}

run_build() {
    local tool
    for tool in git cargo rustc uv uvx python3; do
        require_tool "$tool"
    done
    verify_layout

    record_tree "$TEMP_DIR/before"

    printf 'Repository: %s\n' "$ROOT"
    printf 'rustc: %s\n' "$(rustc --version)"
    printf 'cargo: %s\n' "$(cargo --version)"
    printf 'uv: %s\n' "$(uv --version)"
    printf 'python: %s\n' "$(python3 --version 2>&1)"
    printf 'native extension builder: %s\n' "$(uvx --from 'maturin>=1.0,<2.0' maturin --version)"

    local version
    version="$(python3 "$ROOT/scripts/release-manifests.py" check)"
    printf 'Synchronized workspace version: %s\n' "$version"

    cargo fmt --manifest-path "$ROOT/Cargo.toml" -p vexy-json-cli -- --check
    cargo clippy --manifest-path "$ROOT/Cargo.toml" -p vexy-json-cli --all-targets --all-features --no-deps -- -D warnings
    cargo build --manifest-path "$ROOT/Cargo.toml" --workspace --exclude vexy-json-python --all-features
    cargo test --manifest-path "$ROOT/Cargo.toml" --workspace --exclude vexy-json-python --all-features
    cargo test --manifest-path "$ROOT/Cargo.toml" --workspace --exclude vexy-json-python --all-features --doc
    cargo build --manifest-path "$ROOT/Cargo.toml" --workspace --exclude vexy-json-python --all-features --examples
    python_gate "$version"
    documentation_gate

    record_tree "$TEMP_DIR/after"
    verify_tree_unchanged "$TEMP_DIR/before" "$TEMP_DIR/after"
}

trap finish EXIT
trap 'handle_signal 130' INT
trap 'handle_signal 143' TERM
: >"$LOG"
TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/vexy-json-build.XXXXXX")"
run_build 2>&1 | tee -a "$LOG"
BUILD_COMPLETED=1
