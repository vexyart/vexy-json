#!/usr/bin/env python3
# this_file: scripts/tests/fake-toolchain.py
"""Create deterministic fake Python artifacts and environments for shell tests."""

from __future__ import annotations

import io
import hashlib
import os
import stat
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path


def snapshot_tree(root: Path) -> str:
    """Return a stable digest of every working-tree file byte and symlink target."""
    digest = hashlib.sha256()
    for directory, directories, files in os.walk(root):
        directories[:] = sorted(name for name in directories if name != ".git")
        base = Path(directory)
        for name in sorted(files):
            path = base / name
            relative = path.relative_to(root).as_posix().encode()
            digest.update(relative)
            digest.update(b"\0")
            if path.is_symlink():
                digest.update(b"link\0")
                digest.update(os.readlink(path).encode())
            else:
                digest.update(b"file\0")
                digest.update(path.read_bytes())
            digest.update(b"\0")
    return digest.hexdigest()


def advance_remote(repo: Path, checkout: Path) -> None:
    """Advance the fixture's origin/main from an independent checkout."""
    remote = subprocess.run(
        ["git", "-C", str(repo), "remote", "get-url", "origin"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    subprocess.run(["git", "clone", "--branch", "main", remote, str(checkout)], check=True)
    subprocess.run(["git", "-C", str(checkout), "config", "user.name", "Remote Racer"], check=True)
    subprocess.run(
        ["git", "-C", str(checkout), "config", "user.email", "race@example.invalid"],
        check=True,
    )
    (checkout / "remote-race.txt").write_text("remote advanced during release\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(checkout), "add", "remote-race.txt"], check=True)
    subprocess.run(["git", "-C", str(checkout), "commit", "-m", "remote race"], check=True)
    subprocess.run(["git", "-C", str(checkout), "push", "origin", "main"], check=True)


def make_artifacts(dist: Path, version: str) -> None:
    """Create one minimal wheel and sdist with truthful core metadata."""
    dist.mkdir(parents=True, exist_ok=True)
    metadata = (
        "Metadata-Version: 2.3\n"
        "Name: vexy-json\n"
        f"Version: {version}\n"
        "Summary: shell-contract fixture\n\n"
    ).encode()
    wheel = dist / f"vexy_json-{version}-py3-none-any.whl"
    with zipfile.ZipFile(wheel, "w") as archive:
        archive.writestr(f"vexy_json-{version}.dist-info/METADATA", metadata)
        archive.writestr("vexy_json/__init__.py", f'__version__ = "{version}"\n')
    sdist = dist / f"vexy_json-{version}.tar.gz"
    with tarfile.open(sdist, "w:gz") as archive:
        info = tarfile.TarInfo(f"vexy_json-{version}/PKG-INFO")
        info.size = len(metadata)
        archive.addfile(info, io.BytesIO(metadata))


def make_venv(venv: Path) -> None:
    """Create a tiny interpreter facade that accepts the scripted smoke gates."""
    binary = venv / "bin" / "python"
    binary.parent.mkdir(parents=True, exist_ok=True)
    binary.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        "if [[ ${1:-} == -m && ${2:-} == pytest ]]; then exit 0; fi\n"
        "if [[ ${1:-} == - ]]; then cat >/dev/null; exit 0; fi\n"
        "printf 'fake Python received unsupported arguments: %s\\n' \"$*\" >&2\n"
        "exit 2\n",
        encoding="utf-8",
    )
    binary.chmod(binary.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    activate = venv / "bin" / "activate"
    activate.write_text("# fake activation file for workflow contract tests\n", encoding="utf-8")


def main(argv: list[str]) -> int:
    """Dispatch fixture creation."""
    if len(argv) == 4 and argv[1] == "artifacts":
        make_artifacts(Path(argv[2]), argv[3])
        return 0
    if len(argv) == 3 and argv[1] == "venv":
        make_venv(Path(argv[2]))
        return 0
    if len(argv) == 3 and argv[1] == "snapshot":
        print(snapshot_tree(Path(argv[2])))
        return 0
    if len(argv) == 4 and argv[1] == "advance-remote":
        advance_remote(Path(argv[2]), Path(argv[3]))
        return 0
    print(
        "usage: fake-toolchain.py artifacts DIST VERSION | venv PATH | "
        "snapshot ROOT | advance-remote REPO CHECKOUT",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
