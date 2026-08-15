#!/usr/bin/env python3
# this_file: scripts/tests/fake-toolchain.py
"""Create deterministic fake Python artifacts and environments for shell tests."""

from __future__ import annotations

import io
import os
import stat
import sys
import tarfile
import zipfile
from pathlib import Path


def make_artifacts(dist: Path, version: str) -> None:
    """Create one minimal wheel and sdist with truthful core metadata."""
    dist.mkdir(parents=True, exist_ok=True)
    metadata = (
        "Metadata-Version: 2.3\n"
        "Name: vexy_json\n"
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
    print("usage: fake-toolchain.py artifacts DIST VERSION | venv PATH", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
