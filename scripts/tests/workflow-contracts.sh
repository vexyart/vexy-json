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

capture_repo_state() {
    local repo=$1
    local destination=$2
    "$REAL_PYTHON" "$SUPPORT" snapshot "$repo" >"$destination.files"
    "$REAL_GIT" -C "$repo" status --porcelain=v1 --untracked-files=all >"$destination.status"
    "$REAL_GIT" -C "$repo" ls-files --stage >"$destination.index"
    "$REAL_GIT" -C "$repo" rev-parse HEAD >"$destination.head"
    "$REAL_GIT" -C "$repo" show-ref | LC_ALL=C sort >"$destination.refs"
}

assert_repo_state_same() {
    local before=$1
    local after=$2
    local component
    for component in files status index head refs; do
        cmp -s "$before.$component" "$after.$component" \
            || fail "dry run mutated repository $component state"
    done
}

clone_fixture() {
    local destination=$1
    local origin=$2
    "$REAL_GIT" clone --branch main "$origin" "$destination" >/dev/null
    "$REAL_GIT" -C "$destination" config user.name "Workflow Contract"
    "$REAL_GIT" -C "$destination" config user.email "workflow@example.invalid"
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
    "$REAL_PYTHON" "$destination/scripts/release-manifests.py" sync 1.5.10 >/dev/null
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
if [[ \${REQUIRE_PAGER_DISABLED:-0} == 1 ]]; then
    [[ \${GIT_PAGER:-} == cat && \${PAGER:-} == cat ]] || {
        printf '%s\n' 'publish workflow did not disable interactive pagers' >&2
        exit 87
    }
fi
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
        arguments=" $* "
        package=""
        while (( $# > 0 )); do
            if [[ $1 == -p ]]; then package=$2; shift 2; else shift; fi
        done
        if [[ $arguments == *' --dry-run '* && \
              -n ${FAIL_PREMATURE_DEPENDENT_DRY_RUN:-} && \
              $package != vexy-json-core && \
              ! -e ${FAKE_CORE_PUBLISHED_MARKER:-/nonexistent} ]]; then
            printf '%s\n' "internal dependency for $package is not published" >&2
            exit 86
        fi
        if [[ $arguments != *' --dry-run '* && ${ALLOW_FAKE_UPLOADS:-0} != 1 ]]; then
            printf '%s\n' 'non-dry Cargo upload forbidden by workflow fixture' >&2
            exit 90
        fi
        if [[ $arguments != *' --dry-run '* && $package == vexy-json-core && \
              -n ${FAKE_CORE_PUBLISHED_MARKER:-} ]]; then
            : >"$FAKE_CORE_PUBLISHED_MARKER"
        fi
        ;;
    *)
        if [[ -n ${RACE_REMOTE_MARKER:-} && ! -e $RACE_REMOTE_MARKER ]]; then
            "$REAL_PYTHON" "$SUPPORT" advance-remote "$FIXTURE_ROOT" "$RACE_REMOTE_WORK"
            : >"$RACE_REMOTE_MARKER"
        fi
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
    python)
        [[ ${2:-} == find && ${3:-} == --system && ${4:-} == --no-managed-python ]]
        printf '%s\n' "$REAL_PYTHON"
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
elif [[ ${1:-} == --with-requirements ]]; then
    site_dir=""
    while (( $# > 0 )); do
        if [[ $1 == --site-dir ]]; then site_dir=$2; shift 2; else shift; fi
    done
    [[ -n "$site_dir" ]]
    mkdir -p "$site_dir"
    printf '%s\n' '<!doctype html><title>fixture docs</title>' >"$site_dir/index.html"
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
assert_contains 'mkdocs build --strict' "$ROOT/build.sh"
assert_contains 'documentation_gate' "$ROOT/build.sh"
assert_contains 'uv pip install --system --python "$SYSTEM_PYTHON" --reinstall "$WHEEL"' "$ROOT/install.sh"
assert_contains 'uv build --python "$SYSTEM_PYTHON" --no-sources' "$ROOT/install.sh"
assert_contains 'uv publish "$PY_WHEEL" "$PY_SDIST"' "$ROOT/publish.sh"
assert_contains 'CARGO_HTTP_MULTIPLEXING=false' "$ROOT/publish.sh"
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
[[ "$(grep -Fc 'uv pip install --system --python' "$TOOL_LOG")" == 2 ]] \
    || fail "repeatable install did not use uv pip install --system twice"

printf '\nworkflow contract modification\n' >>"$DRY_REPO/README.md"
printf '%s\n' 'checkpoint me' >"$DRY_REPO/checkpoint-new.txt"
rm "$DRY_REPO/TODO.md"
mkdir -p "$DRY_REPO/.omc" "$DRY_REPO/.codegraph"
printf '%s\n' 'keep local' >"$DRY_REPO/.omc/state.json"
printf '%s\n' 'keep index' >"$DRY_REPO/.codegraph/index.bin"
capture_repo_state "$DRY_REPO" "$TEMP_DIR/dry-before"
: >"$TOOL_LOG"
ALLOW_FAKE_UPLOADS=0 REQUIRE_PAGER_DISABLED=1 \
    GIT_PAGER=/usr/bin/false PAGER=/usr/bin/false \
    "$DRY_REPO/publish.sh" --dry-run >"$TEMP_DIR/dry.out" 2>"$TEMP_DIR/dry.err"
capture_repo_state "$DRY_REPO" "$TEMP_DIR/dry-after"
assert_repo_state_same "$TEMP_DIR/dry-before" "$TEMP_DIR/dry-after"
assert_contains 'Would checkpoint these non-ignored changes:' "$TEMP_DIR/dry.out"
assert_contains 'checkpoint-new.txt' "$TEMP_DIR/dry.out"
assert_contains 'README.md' "$TEMP_DIR/dry.out"
assert_contains 'TODO.md' "$TEMP_DIR/dry.out"
assert_not_contains '.omc/' "$TEMP_DIR/dry.out"
assert_not_contains '.codegraph/' "$TEMP_DIR/dry.out"
assert_not_contains 'cargo publish' "$TOOL_LOG"
assert_not_contains 'uv build' "$TOOL_LOG"
assert_not_contains 'uvx gitnextver@1.0.1' "$TOOL_LOG"

AHEAD_REPO="$TEMP_DIR/repo local ahead"
AHEAD_ORIGIN="$TEMP_DIR/origin local ahead.git"
make_fixture "$AHEAD_REPO" "$AHEAD_ORIGIN"
printf '\nlocal retry\n' >>"$AHEAD_REPO/README.md"
"$REAL_GIT" -C "$AHEAD_REPO" add README.md
"$REAL_GIT" -C "$AHEAD_REPO" commit -m "local retry" >/dev/null
export FIXTURE_ROOT="$AHEAD_REPO"
ALLOW_FAKE_UPLOADS=0 "$AHEAD_REPO/publish.sh" --dry-run \
    >"$TEMP_DIR/ahead.out" 2>"$TEMP_DIR/ahead.err"
assert_contains 'repository=local-ahead' "$TEMP_DIR/ahead.out"

DETACHED_REPO="$TEMP_DIR/repo detached"
DETACHED_ORIGIN="$TEMP_DIR/origin detached.git"
make_fixture "$DETACHED_REPO" "$DETACHED_ORIGIN"
"$REAL_GIT" -C "$DETACHED_REPO" checkout --detach >/dev/null
if "$DETACHED_REPO/publish.sh" --dry-run >"$TEMP_DIR/detached.out" 2>"$TEMP_DIR/detached.err"; then
    fail "detached preflight unexpectedly succeeded"
fi
assert_contains 'attached branch' "$TEMP_DIR/detached.err"

NONMAIN_REPO="$TEMP_DIR/repo nonmain"
NONMAIN_ORIGIN="$TEMP_DIR/origin nonmain.git"
make_fixture "$NONMAIN_REPO" "$NONMAIN_ORIGIN"
"$REAL_GIT" -C "$NONMAIN_REPO" switch -c feature >/dev/null
if "$NONMAIN_REPO/publish.sh" --dry-run >"$TEMP_DIR/nonmain.out" 2>"$TEMP_DIR/nonmain.err"; then
    fail "non-main preflight unexpectedly succeeded"
fi
assert_contains 'requires branch main' "$TEMP_DIR/nonmain.err"

UPSTREAM_REPO="$TEMP_DIR/repo wrong upstream"
UPSTREAM_ORIGIN="$TEMP_DIR/origin wrong upstream.git"
make_fixture "$UPSTREAM_REPO" "$UPSTREAM_ORIGIN"
"$REAL_GIT" -C "$UPSTREAM_REPO" push origin main:alternate >/dev/null
"$REAL_GIT" -C "$UPSTREAM_REPO" branch --set-upstream-to=origin/alternate main >/dev/null
if "$UPSTREAM_REPO/publish.sh" --dry-run >"$TEMP_DIR/upstream.out" 2>"$TEMP_DIR/upstream.err"; then
    fail "wrong-upstream preflight unexpectedly succeeded"
fi
assert_contains 'requires upstream origin/main' "$TEMP_DIR/upstream.err"

CONFLICT_REPO="$TEMP_DIR/repo conflict"
CONFLICT_ORIGIN="$TEMP_DIR/origin conflict.git"
make_fixture "$CONFLICT_REPO" "$CONFLICT_ORIGIN"
"$REAL_GIT" -C "$CONFLICT_REPO" switch -c conflict-side >/dev/null
printf '%s\n' 'side' >"$CONFLICT_REPO/conflict.txt"
"$REAL_GIT" -C "$CONFLICT_REPO" add conflict.txt
"$REAL_GIT" -C "$CONFLICT_REPO" commit -m side >/dev/null
"$REAL_GIT" -C "$CONFLICT_REPO" switch main >/dev/null
printf '%s\n' 'main' >"$CONFLICT_REPO/conflict.txt"
"$REAL_GIT" -C "$CONFLICT_REPO" add conflict.txt
"$REAL_GIT" -C "$CONFLICT_REPO" commit -m main >/dev/null
if "$REAL_GIT" -C "$CONFLICT_REPO" merge conflict-side >/dev/null 2>&1; then
    fail "conflict fixture merge unexpectedly succeeded"
fi
if "$CONFLICT_REPO/publish.sh" --dry-run >"$TEMP_DIR/conflict.out" 2>"$TEMP_DIR/conflict.err"; then
    fail "conflicted preflight unexpectedly succeeded"
fi
assert_contains 'unresolved conflicts' "$TEMP_DIR/conflict.err"

REMOTE_REPO="$TEMP_DIR/repo remote ahead"
REMOTE_ORIGIN="$TEMP_DIR/origin remote ahead.git"
make_fixture "$REMOTE_REPO" "$REMOTE_ORIGIN"
"$REAL_PYTHON" "$SUPPORT" advance-remote "$REMOTE_REPO" "$TEMP_DIR/remote-ahead-writer"
if "$REMOTE_REPO/publish.sh" --dry-run >"$TEMP_DIR/remote-ahead.out" 2>"$TEMP_DIR/remote-ahead.err"; then
    fail "remote-ahead preflight unexpectedly succeeded"
fi
assert_contains 'ahead of or diverged from local HEAD' "$TEMP_DIR/remote-ahead.err"

DIVERGED_REPO="$TEMP_DIR/repo diverged"
DIVERGED_ORIGIN="$TEMP_DIR/origin diverged.git"
make_fixture "$DIVERGED_REPO" "$DIVERGED_ORIGIN"
printf '\nlocal divergence\n' >>"$DIVERGED_REPO/README.md"
"$REAL_GIT" -C "$DIVERGED_REPO" add README.md
"$REAL_GIT" -C "$DIVERGED_REPO" commit -m "local divergence" >/dev/null
"$REAL_PYTHON" "$SUPPORT" advance-remote "$DIVERGED_REPO" "$TEMP_DIR/diverged-writer"
if "$DIVERGED_REPO/publish.sh" --dry-run >"$TEMP_DIR/diverged.out" 2>"$TEMP_DIR/diverged.err"; then
    fail "diverged preflight unexpectedly succeeded"
fi
assert_contains 'ahead of or diverged from local HEAD' "$TEMP_DIR/diverged.err"

REAL_REPO="$TEMP_DIR/repo with spaces real"
REAL_ORIGIN="$TEMP_DIR/origin real.git"
make_fixture "$REAL_REPO" "$REAL_ORIGIN"
export FIXTURE_ROOT="$REAL_REPO"
printf '\nreal checkpoint modification\n' >>"$REAL_REPO/README.md"
printf '%s\n' 'checkpointed untracked file' >"$REAL_REPO/checkpoint-new.txt"
rm "$REAL_REPO/TODO.md"
mkdir -p "$REAL_REPO/.omc" "$REAL_REPO/.codegraph"
printf '%s\n' 'local omc state' >"$REAL_REPO/.omc/state.json"
printf '%s\n' 'local codegraph index' >"$REAL_REPO/.codegraph/index.bin"
: >"$TOOL_LOG"
CORE_PUBLISHED_MARKER="$TEMP_DIR/core-published"
if ! CARGO_REGISTRY_TOKEN=fake UV_PUBLISH_TOKEN=fake ALLOW_FAKE_UPLOADS=1 \
    REQUIRE_PAGER_DISABLED=1 GIT_PAGER=/usr/bin/false PAGER=/usr/bin/false \
    FAIL_PREMATURE_DEPENDENT_DRY_RUN=1 FAKE_CORE_PUBLISHED_MARKER="$CORE_PUBLISHED_MARKER" \
    "$REAL_REPO/publish.sh" >"$TEMP_DIR/real.out" 2>"$TEMP_DIR/real.err"; then
    cat "$TEMP_DIR/real.out" >&2
    cat "$TEMP_DIR/real.err" >&2
    fail "real publish fixture failed"
fi
[[ "$(grep -Fc 'uvx gitnextver@1.0.1 --directory . --verbose' "$TOOL_LOG")" == 1 ]] \
    || fail "real fixture did not invoke gitnextver exactly once"
[[ "$($REAL_GIT -C "$REAL_REPO" describe --tags --exact-match)" == v1.5.11 ]] \
    || fail "real fixture did not create v1.5.11"
"$REAL_PYTHON" "$REAL_REPO/scripts/release-manifests.py" check 1.5.11 >/dev/null
assert_contains 'RELEASE SUCCESS: v1.5.11' "$TEMP_DIR/real.out"
assert_contains 'Checkpoint commit:' "$TEMP_DIR/real.out"
RELEASE_COMMIT="$($REAL_GIT -C "$REAL_REPO" rev-parse HEAD)"
CHECKPOINT_COMMIT="$($REAL_GIT -C "$REAL_REPO" rev-parse "$RELEASE_COMMIT^")"
[[ "$($REAL_GIT -C "$REAL_REPO" show -s --format=%s "$CHECKPOINT_COMMIT")" == \
   'chore: checkpoint work before release' ]] \
    || fail "real flow did not create the deterministic checkpoint commit"
assert_contains 'real checkpoint modification' <("$REAL_GIT" -C "$REAL_REPO" show "$CHECKPOINT_COMMIT:README.md")
[[ "$($REAL_GIT -C "$REAL_REPO" show "$CHECKPOINT_COMMIT:checkpoint-new.txt")" == \
   'checkpointed untracked file' ]] \
    || fail "checkpoint omitted the untracked file"
if "$REAL_GIT" -C "$REAL_REPO" cat-file -e "$CHECKPOINT_COMMIT:TODO.md" 2>/dev/null; then
    fail "checkpoint did not preserve the tracked deletion"
fi
[[ -f "$REAL_REPO/.omc/state.json" && -f "$REAL_REPO/.codegraph/index.bin" ]] \
    || fail "ignored local state was removed"
"$REAL_GIT" -C "$REAL_REPO" check-ignore -q .omc/state.json \
    || fail ".omc is not ignored"
"$REAL_GIT" -C "$REAL_REPO" check-ignore -q .codegraph/index.bin \
    || fail ".codegraph is not ignored"
[[ -z "$($REAL_GIT -C "$REAL_REPO" ls-files '.omc/**' '.codegraph/**')" ]] \
    || fail "local state directories were checkpointed"
[[ -z "$($REAL_GIT -C "$REAL_REPO" status --porcelain=v1 --untracked-files=all)" ]] \
    || fail "real release fixture was not clean after checkpoint and release"
awk '/^cargo publish / && $0 !~ / --dry-run / {
    for (field = 1; field <= NF; field++) {
        if ($field == "-p") print $(field + 1)
    }
}' "$TOOL_LOG" >"$TEMP_DIR/actual-publish-order.txt"
printf '%s\n' vexy-json-core vexy-json-serde vexy-json-cli vexy-json-wasm \
    vexy-json-c-api vexy-json >"$TEMP_DIR/expected-publish-order.txt"
cmp -s "$TEMP_DIR/expected-publish-order.txt" "$TEMP_DIR/actual-publish-order.txt" \
    || fail "Cargo publication order changed"

RACE_REPO="$TEMP_DIR/repo remote race"
RACE_ORIGIN="$TEMP_DIR/origin remote race.git"
make_fixture "$RACE_REPO" "$RACE_ORIGIN"
export FIXTURE_ROOT="$RACE_REPO"
printf '\nrace checkpoint\n' >>"$RACE_REPO/README.md"
RACE_REMOTE_MARKER="$TEMP_DIR/remote-race.marker"
RACE_REMOTE_WORK="$TEMP_DIR/remote-race-writer"
: >"$TOOL_LOG"
set +e
CARGO_REGISTRY_TOKEN=fake UV_PUBLISH_TOKEN=fake ALLOW_FAKE_UPLOADS=1 \
    RACE_REMOTE_MARKER="$RACE_REMOTE_MARKER" RACE_REMOTE_WORK="$RACE_REMOTE_WORK" \
    "$RACE_REPO/publish.sh" >"$TEMP_DIR/race.out" 2>"$TEMP_DIR/race.err"
RACE_STATUS=$?
set -e
(( RACE_STATUS != 0 )) || fail "remote race unexpectedly released"
assert_contains 'Live origin/main changed after preflight' "$TEMP_DIR/race.err"
assert_contains 'Checkpoint commit' "$TEMP_DIR/race.err"
assert_contains 'was preserved' "$TEMP_DIR/race.err"
assert_not_contains 'uvx gitnextver@1.0.1' "$TOOL_LOG"
if "$REAL_GIT" -C "$RACE_REPO" show-ref --verify --quiet refs/tags/v1.5.11; then
    fail "remote race created the target tag"
fi

"$ROOT/scripts/tests/release-workflow-contracts.sh"
printf 'workflow contracts passed; artifacts retained until test exit under %s\n' "$TEMP_DIR"
