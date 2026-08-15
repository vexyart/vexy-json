#!/usr/bin/env bash
# this_file: scripts/tests/release-workflow-contracts.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
RELEASE="$ROOT/.github/workflows/release.yml"
WASM="$ROOT/.github/workflows/wasm.yml"
WASM_BUILD="$ROOT/.github/workflows/wasm-build.yml"
DEPLOY="$ROOT/.github/workflows/deploy.yml"
ROOT_MANIFEST="$ROOT/Cargo.toml"

fail() {
    printf 'release workflow contract failure: %s\n' "$*" >&2
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

for workflow in "$RELEASE" "$WASM" "$WASM_BUILD" "$DEPLOY"; do
    [[ -f "$workflow" ]] || fail "missing workflow: $workflow"
    RUBYOPT=-W0 ruby -e 'require "yaml"; YAML.parse_file(ARGV.fetch(0))' "$workflow"
    assert_contains '# this_file: .github/workflows/' "$workflow"
done

assert_contains 'python3 scripts/release-manifests.py check "$VERSION"' "$RELEASE"
assert_contains './build.sh' "$RELEASE"
assert_contains 'gh release create "$TAG"' "$RELEASE"
assert_contains 'gh release edit "$TAG" --draft=false' "$RELEASE"
assert_not_contains 'scripts/update-versions.sh' "$RELEASE"
assert_not_contains 'cargo publish' "$RELEASE"
assert_not_contains 'uv publish' "$RELEASE"
assert_not_contains 'maturin publish' "$RELEASE"
assert_not_contains 'npm publish' "$RELEASE"

assert_contains 'include = [' "$ROOT_MANIFEST"
assert_contains '"/src/**"' "$ROOT_MANIFEST"

assert_contains 'python3 scripts/release-manifests.py check "$VERSION"' "$WASM"
assert_contains 'cp README.md crates/wasm/pkg/README.md' "$WASM"
assert_contains 'cp LICENSE crates/wasm/pkg/LICENSE' "$WASM"
assert_contains 'npm pack --dry-run' "$WASM"
assert_contains 'npm publish --access public' "$WASM"
assert_not_contains 'crates/wasm/README.md' "$WASM"

publish_count="$(grep -hF 'npm publish' "$RELEASE" "$WASM" | wc -l | tr -d ' ')"
[[ "$publish_count" == 1 ]] || fail "expected one npm publisher; found $publish_count"

for workflow in "$WASM_BUILD" "$DEPLOY"; do
    assert_contains './scripts/build-wasm.sh release' "$workflow"
    assert_not_contains './build-wasm.sh' "$workflow"
    assert_not_contains 'vexy_json.js' "$workflow"
    assert_not_contains 'vexy_json_bg.wasm' "$workflow"
done

deploy_count="$(grep -hF 'actions/deploy-pages@' "$WASM_BUILD" "$DEPLOY" | wc -l | tr -d ' ')"
[[ "$deploy_count" == 1 ]] || fail "expected one GitHub Pages deployer; found $deploy_count"
assert_not_contains 'bundle install' "$DEPLOY"
assert_not_contains 'bundle exec jekyll' "$DEPLOY"

printf 'Release workflow contracts passed.\n'
