#!/usr/bin/env bash
# this_file: scripts/tests/workflow-contracts.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/vexy-json-contracts.XXXXXX")"
REAL_GIT="$(command -v git)"
REAL_PYTHON="$(command -v python3)"
REAL_TEE="$(command -v tee)"
SHIMS="$TEMP_DIR/shims"
TOOL_LOG="$TEMP_DIR/tools.log"
SUPPORT="$ROOT/scripts/tests/fake-toolchain.py"

cleanup() {
    rm -rf -- "$TEMP_DIR"
}
trap cleanup EXIT

fail() {
    printf 'workflow contract failure: %s\n' "$*" >&2
    exit 1
}

assert_contains() {
    local needle=$1
    local file=$2
    grep -F -- "$needle" "$file" >/dev/null || fail "$file does not contain: $needle"
}

assert_not_contains() {
    local needle=$1
    local file=$2
    if grep -F -- "$needle" "$file" >/dev/null; then
        fail "$file unexpectedly contains: $needle"
    fi
}

make_fixture() {
    local destination=$1
    local bare=$2
    mkdir -p "$destination" "$bare"
    tar -C "$ROOT" \
        --exclude=.git \
        --exclude=.codegraph \
        --exclude=.omc \
        --exclude=target \
        --exclude=build.log.txt \
        -cf - . | tar -C "$destination" -xf -
    "$REAL_GIT" init --bare "$bare" >/dev/null
    "$REAL_GIT" -C "$destination" init -b main >/dev/null
    "$REAL_GIT" -C "$destination" config user.name "Workflow Contract"
    "$REAL_GIT" -C "$destination" config user.email "workflow@example.invalid"
    "$REAL_GIT" -C "$destination" add .
    "$REAL_GIT" -C "$destination" commit -m fixture >/dev/null
    "$REAL_GIT" -C "$destination" tag v1.5.10
    "$REAL_GIT" -C "$destination" remote add origin "$bare"
    "$REAL_GIT" -C "$destination" push -u origin main --tags >/dev/null
}

make_shims() {
    mkdir -p "$SHIMS"
    cat >"$SHIMS/git" <<EOF
#!/usr/bin/env bash
printf 'git %s\n' "\$*" >>"\$TOOL_LOG"
exec "$REAL_GIT" "\$@"
EOF
    cat >"$SHIMS/rustc" <<'EOF'
#!/usr/bin/env bash
printf 'rustc %s\n' "$*" >>"$TOOL_LOG"
printf '%s\n' 'rustc 1.97.1 (workflow-contract)'
EOF
    cat >"$SHIMS/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >>"$TOOL_LOG"
case "${1:-}" in
    --version)
        printf '%s\n' 'cargo 1.97.1 (workflow-contract)'
        ;;
    install)
        root=""
        while (( $# > 0 )); do
            if [[ $1 == --root ]]; then root=$2; shift 2; else shift; fi
        done
        [[ -n "$root" ]]
        mkdir -p "$root/bin"
        printf '#!/usr/bin/env bash\nprintf "vexy-json 1.5.10\\n"\n' >"$root/bin/vexy-json"
        chmod +x "$root/bin/vexy-json"
        ;;
    publish)
        if [[ " $* " != *' --dry-run '* && ${ALLOW_FAKE_UPLOADS:-0} != 1 ]]; then
            printf '%s\n' 'non-dry Cargo upload forbidden by workflow fixture' >&2
            exit 90
        fi
        ;;
    *)
        if [[ -n ${BUILD_FAIL_STATUS:-} ]]; then
            exit "$BUILD_FAIL_STATUS"
        fi
        if [[ -n ${BUILD_INTERRUPT_MARKER:-} ]]; then
            : >"$BUILD_INTERRUPT_MARKER"
            while :; do
                sleep 0.05
            done
        fi
        ;;
esac
EOF
    cat >"$SHIMS/uv" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'uv %s\n' "$*" >>"$TOOL_LOG"
case "${1:-}" in
    --version)
        printf '%s\n' 'uv 0.12.3 (workflow-contract)'
        ;;
    build)
        out=""
        while (( $# > 0 )); do
            if [[ $1 == --out-dir ]]; then out=$2; shift 2; else shift; fi
        done
        version="$(sed -n '/^\[package\]/,/^\[/ s/^version = "\([^"]*\)"/\1/p' "$FIXTURE_ROOT/crates/python/Cargo.toml" | head -1)"
        "$REAL_PYTHON" "$SUPPORT" artifacts "$out" "$version"
        ;;
    venv)
        venv="${@: -1}"
        "$REAL_PYTHON" "$SUPPORT" venv "$venv"
        ;;
    pip)
        ;;
    publish)
        if [[ " $* " != *' --dry-run '* && ${ALLOW_FAKE_UPLOADS:-0} != 1 ]]; then
            printf '%s\n' 'non-dry Python upload forbidden by workflow fixture' >&2
            exit 91
        fi
        ;;
    *)
        printf 'unsupported fake uv command: %s\n' "$*" >&2
        exit 2
        ;;
esac
EOF
    cat >"$SHIMS/uvx" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'uvx %s\n' "$*" >>"$TOOL_LOG"
if [[ ${1:-} == --from ]]; then
    printf '%s\n' 'maturin 1.9.4 (workflow-contract)'
elif [[ ${1:-} == gitnextver@1.0.1 ]]; then
    [[ ${ALLOW_FAKE_UPLOADS:-0} == 1 ]]
    [[ "$*" == 'gitnextver@1.0.1 --directory . --verbose' ]]
    version="$(sed -n '/^\[package\]/,/^\[/ s/^version = "\([^"]*\)"/\1/p' "$FIXTURE_ROOT/Cargo.toml" | head -1)"
    git -C "$FIXTURE_ROOT" add Cargo.toml crates/*/Cargo.toml
    git -C "$FIXTURE_ROOT" commit -m "Release v$version" >/dev/null
    git -C "$FIXTURE_ROOT" tag "v$version"
    git -C "$FIXTURE_ROOT" push origin main "v$version" >/dev/null
elif [[ ${1:-} == --with && ${2:-} == hatch-vcs && ${3:-} == hatchling && ${4:-} == version ]]; then
    git -C "$FIXTURE_ROOT" describe --tags --exact-match | sed 's/^v//'
else
    printf 'unsupported fake uvx command: %s\n' "$*" >&2
    exit 2
fi
EOF
    cat >"$SHIMS/curl" <<'EOF'
#!/usr/bin/env bash
printf 'curl %s\n' "$*" >>"$TOOL_LOG"
exit 0
EOF
    cat >"$SHIMS/tee" <<EOF
#!/usr/bin/env bash
"$REAL_TEE" "\$@"
if [[ -n \${FAIL_TEE_STATUS:-} ]]; then
    exit "\$FAIL_TEE_STATUS"
fi
EOF
    chmod +x "$SHIMS"/*
}

bash -n "$ROOT/build.sh" "$ROOT/install.sh" "$ROOT/publish.sh"
"$REAL_PYTHON" -m py_compile \
    "$ROOT/scripts/release-manifests.py" \
    "$ROOT/scripts/check-python-artifacts.py" \
    "$SUPPORT"
"$REAL_PYTHON" "$ROOT/scripts/release-manifests.py" check >/dev/null

for retired in release.sh scripts/release.sh scripts/release.sh.backup \
    scripts/update-versions.sh scripts/test-version-system.sh; do
    [[ ! -e "$ROOT/$retired" ]] || fail "retired file remains: $retired"
done
for active in build.sh install.sh; do
    assert_not_contains gitnextver "$ROOT/$active"
    assert_not_contains 'git push' "$ROOT/$active"
    assert_not_contains 'git commit' "$ROOT/$active"
    assert_not_contains 'git tag' "$ROOT/$active"
    assert_not_contains 'cargo publish' "$ROOT/$active"
    assert_not_contains 'uv publish' "$ROOT/$active"
    assert_not_contains sudo "$ROOT/$active"
    assert_not_contains bindings/python "$ROOT/$active"
done
assert_not_contains bindings/python "$ROOT/publish.sh"
[[ "$(grep -Fc 'uvx gitnextver@1.0.1 --directory . --verbose' "$ROOT/publish.sh")" == 1 ]] \
    || fail "publish.sh must contain exactly one authorized gitnextver command"

make_shims
export PATH="$SHIMS:$PATH"
export TOOL_LOG REAL_PYTHON SUPPORT

DRY_REPO="$TEMP_DIR/repo with spaces dry"
DRY_ORIGIN="$TEMP_DIR/origin dry.git"
make_fixture "$DRY_REPO" "$DRY_ORIGIN"
export FIXTURE_ROOT="$DRY_REPO"
: >"$TOOL_LOG"
(
    cd "$TEMP_DIR"
    "$DRY_REPO/build.sh"
)
assert_contains 'BUILD SUCCESS' "$DRY_REPO/build.log.txt"
assert_not_contains 'BUILD FAILURE' "$DRY_REPO/build.log.txt"

BUILD_FAILURE_OUTPUT="$TEMP_DIR/build-failure.out"
set +e
BUILD_FAIL_STATUS=47 "$DRY_REPO/build.sh" >"$BUILD_FAILURE_OUTPUT" 2>&1
BUILD_FAILURE_STATUS=$?
set -e
[[ "$BUILD_FAILURE_STATUS" == 47 ]] \
    || fail "failed build exited with status $BUILD_FAILURE_STATUS, expected 47"
assert_contains 'BUILD FAILURE: exit status 47' "$BUILD_FAILURE_OUTPUT"
assert_not_contains 'BUILD SUCCESS' "$BUILD_FAILURE_OUTPUT"
assert_contains 'BUILD FAILURE: exit status 47' "$DRY_REPO/build.log.txt"
assert_not_contains 'BUILD SUCCESS' "$DRY_REPO/build.log.txt"

LOGGER_FAILURE_OUTPUT="$TEMP_DIR/logger-failure.out"
set +e
FAIL_TEE_STATUS=74 "$DRY_REPO/build.sh" >"$LOGGER_FAILURE_OUTPUT" 2>&1
LOGGER_FAILURE_STATUS=$?
set -e
[[ "$LOGGER_FAILURE_STATUS" == 74 ]] \
    || fail "logger failure exited with status $LOGGER_FAILURE_STATUS, expected 74"
assert_contains 'BUILD FAILURE: exit status 74' "$LOGGER_FAILURE_OUTPUT"
assert_not_contains 'BUILD SUCCESS' "$LOGGER_FAILURE_OUTPUT"
assert_contains 'BUILD FAILURE: exit status 74' "$DRY_REPO/build.log.txt"
assert_not_contains 'BUILD SUCCESS' "$DRY_REPO/build.log.txt"

INTERRUPT_MARKER="$TEMP_DIR/build-interrupt.ready"
INTERRUPT_OUTPUT="$TEMP_DIR/build-interrupt.out"
INTERRUPT_STATUS="$TEMP_DIR/build-interrupt.status"
export INTERRUPT_MARKER INTERRUPT_OUTPUT INTERRUPT_STATUS
BUILD_INTERRUPT_MARKER="$INTERRUPT_MARKER" "$REAL_PYTHON" - "$DRY_REPO" <<'PY'
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

repo = Path(sys.argv[1])
marker = Path(os.environ["INTERRUPT_MARKER"])
output_path = Path(os.environ["INTERRUPT_OUTPUT"])
status_path = Path(os.environ["INTERRUPT_STATUS"])
temporary_root = marker.parent / "interrupted-build-tmp"
temporary_root.mkdir()
environment = os.environ.copy()
environment["BUILD_INTERRUPT_MARKER"] = str(marker)
environment["TMPDIR"] = str(temporary_root)
process = subprocess.Popen(
    [str(repo / "build.sh")],
    cwd=repo.parent,
    env=environment,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
    start_new_session=True,
)

try:
    deadline = time.monotonic() + 5
    while not marker.exists():
        if process.poll() is not None:
            raise RuntimeError(f"build exited before interrupt gate: {process.returncode}")
        if time.monotonic() >= deadline:
            raise TimeoutError("build did not reach interrupt gate within 5 seconds")
        time.sleep(0.02)

    os.killpg(process.pid, signal.SIGINT)
    output, _ = process.communicate(timeout=5)
except BaseException:
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
    raise

output_path.write_text(output)
status_path.write_text(f"{process.returncode}\n")
if list(temporary_root.iterdir()):
    raise RuntimeError("interrupted build did not clean its temporary directory")
PY
[[ "$(<"$INTERRUPT_STATUS")" == 130 ]] \
    || fail "interrupted build exited with status $(<"$INTERRUPT_STATUS"), expected 130"
assert_contains 'BUILD FAILURE: exit status 130' "$INTERRUPT_OUTPUT"
assert_not_contains 'BUILD SUCCESS' "$INTERRUPT_OUTPUT"
assert_contains 'BUILD FAILURE: exit status 130' "$DRY_REPO/build.log.txt"
assert_not_contains 'BUILD SUCCESS' "$DRY_REPO/build.log.txt"

PREFIX="$TEMP_DIR/install prefix"
"$DRY_REPO/install.sh" --prefix "$PREFIX"
"$DRY_REPO/install.sh" --prefix "$PREFIX"
[[ -x "$PREFIX/bin/vexy-json" ]] || fail "repeatable install did not create CLI"
[[ -x "$PREFIX/share/vexy-json/venv/bin/python" ]] || fail "repeatable install did not create Python environment"

: >"$TOOL_LOG"
ALLOW_FAKE_UPLOADS=0 "$DRY_REPO/publish.sh" --dry-run >"$TEMP_DIR/dry.out" 2>"$TEMP_DIR/dry.err"
[[ "$(grep -c '^cargo publish .*--dry-run' "$TOOL_LOG")" == 6 ]] \
    || fail "dry run did not execute six Cargo package gates"
assert_contains 'uv build --no-sources' "$TOOL_LOG"
assert_contains 'uv publish --dry-run' "$TOOL_LOG"
assert_not_contains 'uvx gitnextver@1.0.1' "$TOOL_LOG"

touch "$DRY_REPO/untracked-blocker"
: >"$TOOL_LOG"
if ALLOW_FAKE_UPLOADS=0 "$DRY_REPO/publish.sh" --dry-run >"$TEMP_DIR/dirty.out" 2>"$TEMP_DIR/dirty.err"; then
    fail "dirty dry run unexpectedly succeeded"
fi
assert_contains 'completely clean tree' "$TEMP_DIR/dirty.err"
assert_not_contains 'cargo publish' "$TOOL_LOG"
assert_not_contains 'uv publish' "$TOOL_LOG"
rm "$DRY_REPO/untracked-blocker"

REAL_REPO="$TEMP_DIR/repo with spaces real"
REAL_ORIGIN="$TEMP_DIR/origin real.git"
make_fixture "$REAL_REPO" "$REAL_ORIGIN"
export FIXTURE_ROOT="$REAL_REPO"
: >"$TOOL_LOG"
CARGO_REGISTRY_TOKEN=fake UV_PUBLISH_TOKEN=fake ALLOW_FAKE_UPLOADS=1 \
    "$REAL_REPO/publish.sh" >"$TEMP_DIR/real.out" 2>"$TEMP_DIR/real.err"
[[ "$(grep -Fc 'uvx gitnextver@1.0.1 --directory . --verbose' "$TOOL_LOG")" == 1 ]] \
    || fail "real fixture did not invoke gitnextver exactly once"
[[ "$($REAL_GIT -C "$REAL_REPO" describe --tags --exact-match)" == v1.5.11 ]] \
    || fail "real fixture did not create v1.5.11"
"$REAL_PYTHON" "$REAL_REPO/scripts/release-manifests.py" check 1.5.11 >/dev/null
assert_contains 'RELEASE SUCCESS: v1.5.11' "$TEMP_DIR/real.out"

printf 'workflow contracts passed; artifacts retained until test exit under %s\n' "$TEMP_DIR"
