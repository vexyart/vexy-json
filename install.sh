#!/usr/bin/env bash
# this_file: install.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
PREFIX="${VEXY_JSON_PREFIX:-${HOME:?HOME is required}/.local}"
TEMP_DIR=""

cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf -- "$TEMP_DIR"
    fi
}
trap cleanup EXIT

usage() {
    printf 'Usage: %s [--prefix PATH]\n' "$0"
}

while (( $# > 0 )); do
    case "$1" in
        --prefix)
            (( $# >= 2 )) || {
                printf '%s\n' '--prefix requires a path' >&2
                exit 2
            }
            PREFIX=$2
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'Unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

for tool in cargo rustc uv python3 git; do
    command -v "$tool" >/dev/null 2>&1 || {
        printf 'Required tool is missing: %s\n' "$tool" >&2
        exit 1
    }
done

for required in crates/cli/Cargo.toml crates/python/pyproject.toml scripts/check-python-artifacts.py; do
    [[ -e "$ROOT/$required" ]] || {
        printf 'Canonical project path is missing: %s\n' "$required" >&2
        exit 1
    }
done

mkdir -p "$PREFIX"
PREFIX="$(cd "$PREFIX" && pwd -P)"
[[ -w "$PREFIX" ]] || {
    printf 'Install prefix is not writable: %s\n' "$PREFIX" >&2
    exit 1
}

TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/vexy-json-install.XXXXXX")"
DIST="$TEMP_DIR/python-dist"
VENV="$PREFIX/share/vexy-json/venv"
PYTHON="$VENV/bin/python"
CLI="$PREFIX/bin/vexy-json"
mkdir -p "$DIST" "$PREFIX/bin" "$(dirname "$VENV")"

git -C "$ROOT" status --porcelain=v1 --untracked-files=all >"$TEMP_DIR/before.status"
git -C "$ROOT" diff --no-ext-diff --binary HEAD -- >"$TEMP_DIR/before.diff"

VERSION="$(python3 "$ROOT/scripts/release-manifests.py" check)"
SYSTEM_PYTHON="$(uv python find --system --no-managed-python)"
printf 'Installing Vexy JSON %s\n' "$VERSION"
printf 'rustc: %s\n' "$(rustc --version)"
printf 'cargo: %s\n' "$(cargo --version)"
printf 'uv: %s\n' "$(uv --version)"
printf 'python builder: %s\n' "$(python3 --version 2>&1)"
printf 'system Python: %s\n' "$SYSTEM_PYTHON"

CARGO_TARGET_DIR="$TEMP_DIR/cargo-target" cargo install \
    --path "$ROOT/crates/cli" \
    --root "$PREFIX" \
    --force
[[ -x "$CLI" ]] || {
    printf 'CLI installation did not create %s\n' "$CLI" >&2
    exit 1
}

(
    cd "$ROOT/crates/python"
    uv build --python "$SYSTEM_PYTHON" --no-sources --out-dir "$DIST"
)
python3 "$ROOT/scripts/check-python-artifacts.py" "$DIST" "$VERSION" >"$TEMP_DIR/artifacts.txt"
WHEEL="$(sed -n '1p' "$TEMP_DIR/artifacts.txt")"

# Install the native extension into the interpreter selected as the system
# Python by uv. The temporary environment below independently smoke-tests the
# exact wheel without affecting the checkout.
uv pip install --system --python "$SYSTEM_PYTHON" --reinstall "$WHEEL"
uv venv --clear --python "$SYSTEM_PYTHON" "$VENV"
uv pip install --python "$PYTHON" "$WHEEL"

"$CLI" --version
"$PYTHON" - "$VERSION" <<'PY'
import sys

import vexy_json

expected = sys.argv[1]
assert vexy_json.__version__ == expected, (vexy_json.__version__, expected)
assert vexy_json.parse("{installed: true,}") == {"installed": True}
print(f"Python import/parse smoke passed: vexy_json {vexy_json.__version__}")
PY

git -C "$ROOT" status --porcelain=v1 --untracked-files=all >"$TEMP_DIR/after.status"
git -C "$ROOT" diff --no-ext-diff --binary HEAD -- >"$TEMP_DIR/after.diff"
if ! cmp -s "$TEMP_DIR/before.status" "$TEMP_DIR/after.status" || \
    ! cmp -s "$TEMP_DIR/before.diff" "$TEMP_DIR/after.diff"; then
    printf 'Install changed the checkout. Before/after status follows.\n' >&2
    diff -u "$TEMP_DIR/before.status" "$TEMP_DIR/after.status" >&2 || true
    exit 1
fi

printf '\nInstallation complete.\n'
printf 'Prefix: %s\n' "$PREFIX"
printf 'CLI: %s\n' "$CLI"
printf 'Python package: installed into the system Python with uv pip install --system\n'
printf 'Add the CLI for this shell with: export PATH="%s/bin:$%s"\n' "$PREFIX" PATH
