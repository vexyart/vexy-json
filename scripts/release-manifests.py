#!/usr/bin/env python3
# this_file: scripts/release-manifests.py
"""Validate and synchronize the release-critical Cargo manifest graph."""

from __future__ import annotations

import os
import re
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFESTS = (
    Path("Cargo.toml"),
    Path("crates/core/Cargo.toml"),
    Path("crates/cli/Cargo.toml"),
    Path("crates/wasm/Cargo.toml"),
    Path("crates/serde/Cargo.toml"),
    Path("crates/test-utils/Cargo.toml"),
    Path("crates/c-api/Cargo.toml"),
    Path("crates/python/Cargo.toml"),
)
EXPECTED_DEPENDENCIES = {
    Path("Cargo.toml"): {"vexy-json-core", "vexy-json-serde"},
    Path("crates/core/Cargo.toml"): set(),
    Path("crates/cli/Cargo.toml"): {"vexy-json-core"},
    Path("crates/wasm/Cargo.toml"): {"vexy-json-core"},
    Path("crates/serde/Cargo.toml"): {"vexy-json-core"},
    Path("crates/test-utils/Cargo.toml"): {"vexy-json-core"},
    Path("crates/c-api/Cargo.toml"): {"vexy-json-core"},
    Path("crates/python/Cargo.toml"): {"vexy-json-core"},
}
PUBLISHABLE = {
    "vexy-json",
    "vexy-json-core",
    "vexy-json-cli",
    "vexy-json-wasm",
    "vexy-json-serde",
    "vexy-json-c-api",
}
PRIVATE = {"vexy-json-test-utils", "vexy-json-python"}
INTERNAL = PUBLISHABLE | PRIVATE
REQUIRED_METADATA = {"description", "license", "repository", "homepage", "readme"}
SEMVER = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
TAG = re.compile(r"^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
SECTION = re.compile(r"^\s*\[\[?([^]]+)]]?\s*$")
STRING = re.compile(r'^\s*([A-Za-z0-9_-]+)\s*=\s*"([^"]*)"\s*$')
BOOLEAN = re.compile(r"^\s*([A-Za-z0-9_-]+)\s*=\s*(true|false)\s*$")
PACKAGE_VERSION = re.compile(r'^(\s*version\s*=\s*)"[^"]*"(\s*)$')
DEPENDENCY_SECTION = re.compile(r"^(?:.*\.)?dependencies\.([^.]*)$")


class ManifestError(RuntimeError):
    """Report a release manifest contract violation."""


def scalar_fields(lines: list[str], wanted_section: str) -> dict[str, str | bool]:
    """Read the simple string and boolean fields used by the release contract."""
    fields: dict[str, str | bool] = {}
    section = ""
    for line in lines:
        match = SECTION.match(line)
        if match:
            section = match.group(1)
            continue
        if section != wanted_section:
            continue
        if match := STRING.match(line):
            fields[match.group(1)] = match.group(2)
        elif match := BOOLEAN.match(line):
            fields[match.group(1)] = match.group(2) == "true"
    return fields


def dependency_fields(lines: list[str]) -> dict[str, dict[str, str | bool]]:
    """Read internal dependency tables without accepting inline/path ambiguity."""
    dependencies: dict[str, dict[str, str | bool]] = {}
    current: str | None = None
    for line in lines:
        match = SECTION.match(line)
        if match:
            dependency = DEPENDENCY_SECTION.match(match.group(1))
            current = dependency.group(1) if dependency else None
            continue
        if current not in INTERNAL:
            continue
        if match := STRING.match(line):
            dependencies.setdefault(current, {})[match.group(1)] = match.group(2)
        elif match := BOOLEAN.match(line):
            dependencies.setdefault(current, {})[match.group(1)] = match.group(2) == "true"
    return dependencies


def load_manifests() -> dict[Path, list[str]]:
    """Load exactly the workspace manifests admitted to release synchronization."""
    loaded: dict[Path, list[str]] = {}
    for relative in MANIFESTS:
        path = ROOT / relative
        if not path.is_file():
            raise ManifestError(f"missing release manifest: {relative}")
        loaded[relative] = path.read_text(encoding="utf-8").splitlines(keepends=True)
    return loaded


def validate(expected: str | None = None) -> str:
    """Validate package policy and return the single synchronized version."""
    versions: set[str] = set()
    packages: set[str] = set()
    for relative, lines in load_manifests().items():
        package = scalar_fields(lines, "package")
        name = package.get("name")
        version = package.get("version")
        if not isinstance(name, str) or not isinstance(version, str):
            raise ManifestError(f"{relative}: [package] name/version must be strings")
        if not SEMVER.fullmatch(version):
            raise ManifestError(f"{relative}: package version is not canonical SemVer: {version}")
        packages.add(name)
        versions.add(version)

        if name in PUBLISHABLE:
            missing = sorted(field for field in REQUIRED_METADATA if not package.get(field))
            if missing:
                raise ManifestError(f"{relative}: publishable package lacks {', '.join(missing)}")
            text = "".join(lines)
            if not re.search(r"^\s*publish\s*=\s*\[\s*\"crates-io\"\s*]\s*$", text, re.M):
                raise ManifestError(f"{relative}: publishable package must allow only crates-io")
        elif name in PRIVATE:
            if package.get("publish") is not False:
                raise ManifestError(f"{relative}: private package must set publish = false")
        else:
            raise ManifestError(f"{relative}: package is outside the release allowlist: {name}")

        dependencies = dependency_fields(lines)
        if set(dependencies) != EXPECTED_DEPENDENCIES[relative]:
            raise ManifestError(
                f"{relative}: internal dependency graph differs; "
                f"expected={sorted(EXPECTED_DEPENDENCIES[relative])}, "
                f"actual={sorted(dependencies)}"
            )
        for dependency, fields in dependencies.items():
            if not fields.get("path"):
                raise ManifestError(f"{relative}: internal dependency {dependency} lacks path")
            if fields.get("version") != f"={version}":
                raise ManifestError(
                    f"{relative}: internal dependency {dependency} must require ={version}"
                )

    if packages != INTERNAL:
        missing = sorted(INTERNAL - packages)
        extra = sorted(packages - INTERNAL)
        raise ManifestError(f"package allowlist mismatch; missing={missing}, extra={extra}")
    if len(versions) != 1:
        raise ManifestError(f"workspace package versions differ: {sorted(versions)}")
    version = versions.pop()
    if expected is not None and version != expected:
        raise ManifestError(f"workspace version {version} does not equal expected {expected}")
    return version


def replace_version(lines: list[str], version: str) -> list[str]:
    """Replace package and internal dependency versions in one manifest."""
    result: list[str] = []
    section = ""
    package_replaced = False
    dependency_replaced: set[str] = set()

    def finish_section() -> None:
        dependency = DEPENDENCY_SECTION.match(section)
        if dependency and dependency.group(1) in INTERNAL:
            name = dependency.group(1)
            if name not in dependency_replaced:
                result.append(f'version = "={version}"\n')
                dependency_replaced.add(name)

    for line in lines:
        match = SECTION.match(line)
        if match:
            finish_section()
            section = match.group(1)
            result.append(line)
            continue
        if section == "package" and PACKAGE_VERSION.match(line):
            if package_replaced:
                raise ManifestError("duplicate [package] version field")
            result.append(PACKAGE_VERSION.sub(rf'\g<1>"{version}"\g<2>', line))
            package_replaced = True
            continue
        dependency = DEPENDENCY_SECTION.match(section)
        if dependency and dependency.group(1) in INTERNAL and PACKAGE_VERSION.match(line):
            name = dependency.group(1)
            result.append(PACKAGE_VERSION.sub(rf'\g<1>"={version}"\g<2>', line))
            dependency_replaced.add(name)
            continue
        result.append(line)
    finish_section()
    if not package_replaced:
        raise ManifestError("missing [package] version field")
    return result


def atomic_write(path: Path, lines: list[str]) -> None:
    """Replace a manifest atomically while preserving its mode."""
    mode = path.stat().st_mode
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, delete=False
    ) as handle:
        handle.writelines(lines)
        temporary = Path(handle.name)
    os.chmod(temporary, mode)
    os.replace(temporary, path)


def synchronize(version: str) -> None:
    """Synchronize the explicit manifest allowlist to one exact version."""
    if not SEMVER.fullmatch(version):
        raise ManifestError(f"target version is not canonical SemVer: {version}")
    loaded = load_manifests()
    for relative, lines in loaded.items():
        atomic_write(ROOT / relative, replace_version(lines, version))
    validate(version)


def next_patch(tags: list[str]) -> str:
    """Return the patch after the highest canonical v-prefixed SemVer tag."""
    versions = [tuple(map(int, match.groups())) for tag in tags if (match := TAG.fullmatch(tag))]
    if not versions:
        raise ManifestError("no canonical vMAJOR.MINOR.PATCH tag found")
    major, minor, patch = max(versions)
    return f"{major}.{minor}.{patch + 1}"


def main(argv: list[str]) -> int:
    """Run the requested manifest operation."""
    try:
        command = argv[1] if len(argv) > 1 else "check"
        if command == "check" and len(argv) in (2, 3):
            print(validate(argv[2] if len(argv) == 3 else None))
        elif command == "current-version" and len(argv) == 2:
            print(validate())
        elif command == "sync" and len(argv) == 3:
            synchronize(argv[2])
            print(argv[2])
        elif command == "next-patch" and len(argv) == 2:
            print(next_patch([line.strip() for line in sys.stdin if line.strip()]))
        else:
            print(
                "usage: release-manifests.py check [VERSION] | current-version | "
                "sync VERSION | next-patch",
                file=sys.stderr,
            )
            return 2
    except (ManifestError, OSError) as error:
        print(f"release manifest error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
