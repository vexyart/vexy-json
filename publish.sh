#!/usr/bin/env bash
# this_file: publish.sh

set -euo pipefail

# Release automation must never hand control to an interactive pager. In
# particular, `git diff --cached --check` runs immediately after preflight and
# otherwise may open `less`, leaving the operator at an unexplained `(END)`.
export GIT_PAGER=cat
export PAGER=cat

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
MODE="real"
TEMP_DIR=""
PHASE="startup"
TARGET_VERSION=""
TARGET_TAG=""
RELEASE_COMMIT=""
REMOTE_BASE=""
LATEST_VERSION=""
SNAPSHOT_COMMIT=""
CHECKPOINT_CREATED=0
CHECKPOINT_MESSAGE="chore: checkpoint work before release"
COMPLETED=()
MANIFEST_ALLOWLIST=(
    Cargo.toml
    crates/c-api/Cargo.toml
    crates/cli/Cargo.toml
    crates/core/Cargo.toml
    crates/python/Cargo.toml
    crates/serde/Cargo.toml
    crates/test-utils/Cargo.toml
    crates/wasm/Cargo.toml
)

cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf -- "$TEMP_DIR"
    fi
}

finish() {
    local status=$?
    if (( status != 0 )); then
        printf '\nRelease stopped during phase: %s\n' "$PHASE" >&2
        if (( ${#COMPLETED[@]} > 0 )); then
            printf 'Confirmed published artifacts:\n' >&2
            printf '  %s\n' "${COMPLETED[@]}" >&2
        fi
        if (( CHECKPOINT_CREATED == 1 )); then
            printf 'Checkpoint commit %s was preserved; no files were stashed, reset, cleaned, or discarded.\n' \
                "$SNAPSHOT_COMMIT" >&2
        elif [[ -n "$SNAPSHOT_COMMIT" ]]; then
            printf 'Checkpoint baseline %s was preserved; no files were stashed, reset, cleaned, or discarded.\n' \
                "$SNAPSHOT_COMMIT" >&2
        fi
        if [[ "$PHASE" == prepare-target || "$PHASE" == target-build || \
              "$PHASE" == release-boundary ]]; then
            printf 'Synchronized manifest changes were preserved for review; nothing was discarded.\n' >&2
        elif [[ "$PHASE" == create-release || "$PHASE" == verify-refs ]]; then
            printf 'Release refs were preserved. Inspect local and live origin refs; do not rewrite them automatically.\n' >&2
        elif [[ "$PHASE" == upload-* || "$PHASE" == registry-* ]]; then
            printf 'Registry publication may be irreversible or ambiguous. Check registry truth before any operator retry.\n' >&2
        fi
    fi
    cleanup
    exit "$status"
}
trap finish EXIT

usage() {
    printf 'Usage: %s [--dry-run]\n' "$0"
}

if (( $# > 1 )); then
    usage >&2
    exit 2
fi
case "${1:-}" in
    --dry-run)
        MODE="dry-run"
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    "")
        ;;
    *)
        printf 'Unknown argument: %s\n' "$1" >&2
        usage >&2
        exit 2
        ;;
esac

require_tool() {
    local tool=$1
    command -v "$tool" >/dev/null 2>&1 || {
        printf 'Required tool is missing: %s\n' "$tool" >&2
        return 1
    }
}

remote_tag_commit() {
    local tag=$1
    local refs
    refs="$(git -C "$ROOT" ls-remote --tags origin "refs/tags/$tag" "refs/tags/$tag^{}")"
    local peeled
    peeled="$(printf '%s\n' "$refs" | awk '$2 ~ /\^\{\}$/ {print $1}')"
    if [[ -n "$peeled" ]]; then
        printf '%s\n' "$peeled"
    else
        printf '%s\n' "$refs" | awk '$2 !~ /\^\{\}$/ {print $1}'
    fi
}

read_only_preflight() {
    PHASE="read-only-preflight"
    local tool
    for tool in git cargo rustc uv uvx python3; do
        require_tool "$tool"
    done
    if [[ "$MODE" == "real" ]]; then
        require_tool curl
        [[ -n "${CARGO_REGISTRY_TOKEN:-}" ]] || {
            printf 'CARGO_REGISTRY_TOKEN is required for a real release.\n' >&2
            return 1
        }
        [[ -n "${UV_PUBLISH_TOKEN:-}" ]] || {
            printf 'UV_PUBLISH_TOKEN is required for a real release.\n' >&2
            return 1
        }
    fi

    [[ -x "$ROOT/build.sh" ]] || {
        printf 'build.sh is missing or not executable.\n' >&2
        return 1
    }
    [[ -x "$ROOT/scripts/release-manifests.py" ]] || {
        printf 'release manifest helper is missing or not executable.\n' >&2
        return 1
    }
    [[ -z "$(git -C "$ROOT" ls-files -u)" ]] || {
        printf 'Publish cannot checkpoint a tree with unresolved conflicts.\n' >&2
        return 1
    }

    local branch upstream head repository_state
    branch="$(git -C "$ROOT" symbolic-ref --quiet --short HEAD)" || {
        printf 'Publish requires an attached branch.\n' >&2
        return 1
    }
    [[ "$branch" == "main" ]] || {
        printf 'Publish requires branch main; found %s.\n' "$branch" >&2
        return 1
    }
    upstream="$(git -C "$ROOT" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}')" || {
        printf 'Publish requires an upstream branch.\n' >&2
        return 1
    }
    [[ "$upstream" == "origin/main" ]] || {
        printf 'Publish requires upstream origin/main; found %s.\n' "$upstream" >&2
        return 1
    }
    git -C "$ROOT" remote get-url origin >/dev/null
    head="$(git -C "$ROOT" rev-parse HEAD)"
    REMOTE_BASE="$(git -C "$ROOT" ls-remote --exit-code origin refs/heads/main | awk 'NR == 1 {print $1}')"
    [[ -n "$REMOTE_BASE" ]] || {
        printf 'Live origin/main is missing.\n' >&2
        return 1
    }
    if [[ "$head" == "$REMOTE_BASE" ]]; then
        repository_state="equal"
    elif git -C "$ROOT" cat-file -e "$REMOTE_BASE^{commit}" 2>/dev/null && \
         git -C "$ROOT" merge-base --is-ancestor "$REMOTE_BASE" "$head"; then
        repository_state="local-ahead"
    else
        printf 'Live origin/main is ahead of or diverged from local HEAD. local=%s remote=%s\n' \
            "$head" "$REMOTE_BASE" >&2
        return 1
    fi

    local tags_file="$TEMP_DIR/tags.txt"
    git -C "$ROOT" tag --list 'v*' >"$tags_file"
    git -C "$ROOT" ls-remote --tags origin \
        | awk '$2 !~ /\^\{\}$/ {sub("refs/tags/", "", $2); print $2}' >>"$tags_file"
    local malformed_tags
    malformed_tags="$(grep -Ev '^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' "$tags_file" || true)"
    [[ -z "$malformed_tags" ]] || {
        printf 'Malformed v-prefixed release tags block prediction:\n%s\n' "$malformed_tags" >&2
        return 1
    }
    TARGET_VERSION="$(python3 "$ROOT/scripts/release-manifests.py" next-patch <"$tags_file")"
    TARGET_TAG="v$TARGET_VERSION"

    local major minor patch latest_tag local_latest remote_latest
    IFS=. read -r major minor patch <<<"$TARGET_VERSION"
    latest_tag="v$major.$minor.$((patch - 1))"
    LATEST_VERSION="${latest_tag#v}"
    local_latest="$(git -C "$ROOT" rev-parse "$latest_tag^{commit}")" || {
        printf 'Latest canonical tag is missing locally: %s\n' "$latest_tag" >&2
        return 1
    }
    remote_latest="$(remote_tag_commit "$latest_tag")"
    [[ -n "$remote_latest" && "$local_latest" == "$remote_latest" ]] || {
        printf 'Latest local and remote tag commits differ for %s.\n' "$latest_tag" >&2
        return 1
    }
    if git -C "$ROOT" show-ref --verify --quiet "refs/tags/$TARGET_TAG" || \
        [[ -n "$(remote_tag_commit "$TARGET_TAG")" ]]; then
        printf 'Predicted release tag already exists: %s\n' "$TARGET_TAG" >&2
        return 1
    fi

    printf 'Preflight passed: predicted=%s branch=%s repository=%s remote-base=%s\n' \
        "$TARGET_VERSION" "$branch" "$repository_state" "$REMOTE_BASE"
}

report_checkpoint_plan() {
    PHASE="dry-run-plan"
    local changes
    changes="$(git -C "$ROOT" status --short --untracked-files=all)"
    if [[ -n "$changes" ]]; then
        printf 'Would checkpoint these non-ignored changes:\n%s\n' "$changes"
    else
        printf 'No checkpoint commit would be needed.\n'
    fi
}

checkpoint_changes() {
    PHASE="checkpoint"
    git -C "$ROOT" add -A -- .
    git -C "$ROOT" diff --cached --check
    if git -C "$ROOT" diff --cached --quiet; then
        printf 'Checkpoint not needed: working tree already clean.\n'
    else
        git -C "$ROOT" commit -m "$CHECKPOINT_MESSAGE"
        CHECKPOINT_CREATED=1
    fi
    [[ -z "$(git -C "$ROOT" status --porcelain=v1 --untracked-files=all)" ]] || {
        printf 'Checkpoint did not produce a clean tree.\n' >&2
        return 1
    }
    SNAPSHOT_COMMIT="$(git -C "$ROOT" rev-parse HEAD)"
    printf 'Checkpoint commit: %s\n' "$SNAPSHOT_COMMIT"
}

python_package_gate() {
    local expected=$1
    local label=$2
    local dist="$TEMP_DIR/python-$label-dist"
    local venv="$TEMP_DIR/python-$label-venv"
    local python="$venv/bin/python"
    local artifacts="$TEMP_DIR/python-$label-artifacts.txt"
    mkdir -p "$dist"
    (
        cd "$ROOT/crates/python"
        uv build --no-sources --out-dir "$dist"
    )
    python3 "$ROOT/scripts/check-python-artifacts.py" "$dist" "$expected" >"$artifacts"
    PY_WHEEL="$(sed -n '1p' "$artifacts")"
    PY_SDIST="$(sed -n '2p' "$artifacts")"
    [[ -f "$PY_WHEEL" && -f "$PY_SDIST" ]]

    uv venv --clear --python python3 "$venv"
    uv pip install --python "$python" "$PY_WHEEL" pytest
    "$python" -m pytest "$ROOT/crates/python/tests" -q
    "$python" - "$expected" <<'PY'
import sys

import vexy_json

expected = sys.argv[1]
assert vexy_json.__version__ == expected, (vexy_json.__version__, expected)
assert vexy_json.parse("{release_gate: true,}") == {"release_gate": True}
print(f"Python package gate passed: vexy_json {vexy_json.__version__}")
PY
}

pre_mutation_gates() {
    PHASE="pre-mutation-gates"
    local current
    current="$(python3 "$ROOT/scripts/release-manifests.py" check)"
    [[ "$current" == "$LATEST_VERSION" ]] || {
        printf 'Cargo version %s does not equal latest release tag v%s.\n' \
            "$current" "$LATEST_VERSION" >&2
        return 1
    }
    "$ROOT/build.sh"
    cargo publish --manifest-path "$ROOT/Cargo.toml" --dry-run -p vexy-json-core
    python_package_gate "$current" current
    uv publish --dry-run "$PY_WHEEL" "$PY_SDIST"
}

assert_manifest_diff() {
    local actual="$TEMP_DIR/actual-manifests.txt"
    local expected="$TEMP_DIR/expected-manifests.txt"
    git -C "$ROOT" diff --name-only -- >"$actual"
    printf '%s\n' "${MANIFEST_ALLOWLIST[@]}" >"$expected"
    LC_ALL=C sort -o "$actual" "$actual"
    LC_ALL=C sort -o "$expected" "$expected"
    if ! cmp -s "$actual" "$expected"; then
        printf 'Release preparation changed files outside the exact manifest allowlist.\n' >&2
        diff -u "$expected" "$actual" >&2 || true
        return 1
    fi
    [[ -z "$(git -C "$ROOT" diff --cached --name-only)" ]] || {
        printf 'Release preparation unexpectedly staged files.\n' >&2
        return 1
    }
}

prepare_target() {
    PHASE="prepare-target"
    python3 "$ROOT/scripts/release-manifests.py" sync "$TARGET_VERSION" >/dev/null
    python3 "$ROOT/scripts/release-manifests.py" check "$TARGET_VERSION" >/dev/null
    assert_manifest_diff
    PHASE="target-build"
    "$ROOT/build.sh"
    assert_manifest_diff
}

verify_release_boundary() {
    PHASE="release-boundary"
    local live_remote head
    live_remote="$(git -C "$ROOT" ls-remote --exit-code origin refs/heads/main | awk 'NR == 1 {print $1}')"
    [[ "$live_remote" == "$REMOTE_BASE" ]] || {
        printf 'Live origin/main changed after preflight. before=%s now=%s\n' \
            "$REMOTE_BASE" "$live_remote" >&2
        return 1
    }
    head="$(git -C "$ROOT" rev-parse HEAD)"
    [[ -n "$SNAPSHOT_COMMIT" && "$head" == "$SNAPSHOT_COMMIT" ]] || {
        printf 'HEAD changed after checkpoint. snapshot=%s now=%s\n' "$SNAPSHOT_COMMIT" "$head" >&2
        return 1
    }
    git -C "$ROOT" merge-base --is-ancestor "$REMOTE_BASE" "$SNAPSHOT_COMMIT" || {
        printf 'Checkpoint commit is not descended from the preflight remote base.\n' >&2
        return 1
    }
    if git -C "$ROOT" show-ref --verify --quiet "refs/tags/$TARGET_TAG" || \
        [[ -n "$(remote_tag_commit "$TARGET_TAG")" ]]; then
        printf 'Predicted release tag appeared after preflight: %s\n' "$TARGET_TAG" >&2
        return 1
    fi
    assert_manifest_diff
    printf 'Release boundary verified: snapshot=%s remote-base=%s target=%s\n' \
        "$SNAPSHOT_COMMIT" "$REMOTE_BASE" "$TARGET_TAG"
}

verify_release_refs() {
    PHASE="verify-refs"
    [[ -z "$(git -C "$ROOT" status --porcelain=v1 --untracked-files=all)" ]] || {
        printf 'Release command left a dirty tree.\n' >&2
        return 1
    }
    RELEASE_COMMIT="$(git -C "$ROOT" rev-parse HEAD)"
    local local_tag branch_remote tag_remote new_tags
    local_tag="$(git -C "$ROOT" rev-parse "$TARGET_TAG^{commit}")" || {
        printf 'Expected local release tag is missing: %s\n' "$TARGET_TAG" >&2
        return 1
    }
    new_tags="$(comm -13 "$TEMP_DIR/tags-before.txt" <(git -C "$ROOT" tag --list 'v*' | LC_ALL=C sort))"
    [[ "$new_tags" == "$TARGET_TAG" ]] || {
        printf 'Release command created unexpected canonical tags: %s\n' "$new_tags" >&2
        return 1
    }
    branch_remote="$(git -C "$ROOT" ls-remote --exit-code origin refs/heads/main | awk 'NR == 1 {print $1}')"
    tag_remote="$(remote_tag_commit "$TARGET_TAG")"
    [[ "$local_tag" == "$RELEASE_COMMIT" && \
       "$branch_remote" == "$RELEASE_COMMIT" && \
       "$tag_remote" == "$RELEASE_COMMIT" ]] || {
        printf 'Release ref mismatch: HEAD=%s local-tag=%s remote-branch=%s remote-tag=%s\n' \
            "$RELEASE_COMMIT" "$local_tag" "$branch_remote" "$tag_remote" >&2
        return 1
    }
    python3 "$ROOT/scripts/release-manifests.py" check "$TARGET_VERSION" >/dev/null

    local hatch_version
    hatch_version="$(cd "$ROOT/crates/python" && uvx --with hatch-vcs hatchling version)"
    [[ "$hatch_version" == "$TARGET_VERSION" ]] || {
        printf 'Tag-derived Hatch version differs: expected=%s hatch=%s\n' "$TARGET_VERSION" "$hatch_version" >&2
        return 1
    }
    printf 'Verified release refs and tag-derived version: %s at %s\n' "$TARGET_TAG" "$RELEASE_COMMIT"
}

wait_for_crate() {
    local package=$1
    local version=$2
    local deadline=$((SECONDS + 180))
    while (( SECONDS < deadline )); do
        if curl --fail --silent --show-error \
            --header 'User-Agent: vexy-json-release-check' \
            "https://crates.io/api/v1/crates/$package/$version" >/dev/null; then
            return 0
        fi
        sleep 5
    done
    printf 'Timed out waiting for crates.io visibility: %s@%s\n' "$package" "$version" >&2
    return 1
}

publish_crate() {
    local package=$1
    PHASE="upload-crate-$package"
    cargo publish --manifest-path "$ROOT/Cargo.toml" --dry-run -p "$package"
    # Cargo/libcurl HTTP/2 uploads can fail mid-stream for registry writes.
    # Disable multiplexing for the irreversible upload while retaining the
    # ordinary dry-run verification above.
    if ! CARGO_HTTP_MULTIPLEXING=false \
        cargo publish --manifest-path "$ROOT/Cargo.toml" -p "$package"; then
        printf 'Cargo upload failed or is ambiguous: %s@%s\n' "$package" "$TARGET_VERSION" >&2
        return 1
    fi
    PHASE="registry-crate-$package"
    wait_for_crate "$package" "$TARGET_VERSION"
    COMPLETED+=("$package@$TARGET_VERSION")
}

wait_for_python() {
    local deadline=$((SECONDS + 180))
    while (( SECONDS < deadline )); do
        if curl --fail --silent --show-error \
            --header 'User-Agent: vexy-json-release-check' \
            "https://pypi.org/pypi/vexy-json/$TARGET_VERSION/json" >/dev/null; then
            return 0
        fi
        sleep 5
    done
    printf 'Timed out waiting for PyPI visibility: vexy-json@%s\n' "$TARGET_VERSION" >&2
    return 1
}

publish_all() {
    publish_crate vexy-json-core
    publish_crate vexy-json-serde
    publish_crate vexy-json-cli
    publish_crate vexy-json-wasm
    publish_crate vexy-json-c-api
    publish_crate vexy-json

    PHASE="python-final-gate"
    python_package_gate "$TARGET_VERSION" release
    local hatch_version
    hatch_version="$(cd "$ROOT/crates/python" && uvx --with hatch-vcs hatchling version)"
    [[ "$hatch_version" == "$TARGET_VERSION" ]]
    uv publish --dry-run "$PY_WHEEL" "$PY_SDIST"
    PHASE="upload-python"
    if ! uv publish "$PY_WHEEL" "$PY_SDIST"; then
        printf 'PyPI upload failed or is ambiguous: vexy-json@%s\n' "$TARGET_VERSION" >&2
        return 1
    fi
    PHASE="registry-python"
    wait_for_python
    COMPLETED+=("vexy-json-python@$TARGET_VERSION")
}

TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/vexy-json-release.XXXXXX")"
read_only_preflight

if [[ "$MODE" == "dry-run" ]]; then
    report_checkpoint_plan
    PHASE="dry-run-complete"
    printf '\nDRY RUN SUCCESS\n'
    printf 'Predicted next release: %s\n' "$TARGET_TAG"
    printf 'Real mode will synchronize the eight-manifest allowlist, run the target build, create refs once, verify live refs, then upload six Rust crates and the inspected Python artifacts.\n'
    exit 0
fi

checkpoint_changes
pre_mutation_gates
prepare_target
verify_release_boundary
git -C "$ROOT" tag --list 'v*' | LC_ALL=C sort >"$TEMP_DIR/tags-before.txt"
PHASE="create-release"
(
    cd "$ROOT"
    uvx gitnextver@1.0.1 --directory . --verbose
)
verify_release_refs
publish_all
PHASE="complete"

printf '\nRELEASE SUCCESS: %s at %s\n' "$TARGET_TAG" "$RELEASE_COMMIT"
printf 'Confirmed published artifacts:\n'
printf '  %s\n' "${COMPLETED[@]}"
