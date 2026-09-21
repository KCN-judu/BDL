#!/usr/bin/env python3
"""The production mirror of the BDL Design and Formalization Monograph.

`reference/paper/` is a byte-for-byte copy of `paper/monograph/` in KCN-judu/BDL_FV at
the commit recorded in `reference/paper-mirror.toml` (reference/README.md).
This script keeps it that way from a local BDL_FV checkout; it is optional
reference tooling — nothing in the build, the tests or CI runs it, and this
repository never depends on BDL_FV being present (ADR-0010).

    paper_mirror.py check <path-to-BDL_FV>            # compare with the recorded commit
    paper_mirror.py refresh <path-to-BDL_FV> [commit] # re-mirror (default: BDL_FV HEAD)
"""

from __future__ import annotations

import hashlib
import io
import re
import shutil
import subprocess
import sys
import tarfile
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MIRROR = ROOT / "reference" / "paper"
PROVENANCE = ROOT / "reference" / "paper-mirror.toml"
# The monograph's directory in BDL_FV (it moved from `paper/` to `paper/monograph/`
# when the core-calculus paper joined it).
SOURCE_DIR = "paper/monograph"


def recorded_commit() -> str:
    m = re.search(r'^commit = "([0-9a-f]{40})"', PROVENANCE.read_text(), re.M)
    if not m:
        sys.exit(f"paper-mirror: no commit in {PROVENANCE}")
    return m.group(1)


def canonical_tree(fv: Path, commit: str) -> dict[str, bytes]:
    """`paper/monograph/` at `commit`, path → bytes, straight from the object store."""
    try:
        data = subprocess.run(
            ["git", "-C", str(fv), "archive", "--format=tar", commit, SOURCE_DIR],
            check=True,
            capture_output=True,
        ).stdout
    except subprocess.CalledProcessError as e:
        sys.exit(f"paper-mirror: git archive failed in {fv}: {e.stderr.decode().strip()}")
    files: dict[str, bytes] = {}
    with tarfile.open(fileobj=io.BytesIO(data)) as tar:
        for member in tar.getmembers():
            if member.isfile():
                f = tar.extractfile(member)
                assert f is not None
                files[member.name.removeprefix(SOURCE_DIR + "/")] = f.read()
    return files


def mirror_tree() -> dict[str, bytes]:
    return {
        p.relative_to(MIRROR).as_posix(): p.read_bytes() for p in sorted(MIRROR.rglob("*")) if p.is_file()
    }


def digest(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()[:12]


def check(fv: Path) -> int:
    commit = recorded_commit()
    want = canonical_tree(fv, commit)
    have = mirror_tree()
    problems = 0
    for path in sorted(set(want) | set(have)):
        if path not in have:
            print(f"missing in mirror: {path}")
            problems += 1
        elif path not in want:
            print(f"not in canonical: {path}")
            problems += 1
        elif want[path] != have[path]:
            print(f"differs: {path} (mirror {digest(have[path])}, canonical {digest(want[path])})")
            problems += 1
        else:
            print(f"identical: {path}")
    if problems:
        print(f"paper-mirror: {problems} difference(s) from BDL_FV {commit[:7]}")
        return 1
    print(f"paper-mirror: {len(want)} files identical to BDL_FV {commit[:7]} {SOURCE_DIR}/")
    return 0


def refresh(fv: Path, commit: str | None) -> int:
    full = subprocess.run(
        ["git", "-C", str(fv), "rev-parse", commit or "HEAD"], check=True, capture_output=True, text=True
    ).stdout.strip()
    files = canonical_tree(fv, full)
    if MIRROR.exists():
        shutil.rmtree(MIRROR)
    for path, data in files.items():
        out = MIRROR / path
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(data)
    (MIRROR / "build.sh").chmod(0o755)
    text = PROVENANCE.read_text()
    text = re.sub(r'^commit = "[0-9a-f]{40}"', f'commit = "{full}"', text, flags=re.M)
    text = re.sub(r'^mirrored = "[^"]*"', f'mirrored = "{date.today().isoformat()}"', text, flags=re.M)
    PROVENANCE.write_text(text)
    print(f"paper-mirror: {len(files)} files mirrored from BDL_FV {full[:7]} {SOURCE_DIR}/")
    return check(fv)


def main(argv: list[str]) -> int:
    if len(argv) < 3 or argv[1] not in ("check", "refresh"):
        print(__doc__)
        return 2
    fv = Path(argv[2]).expanduser().resolve()
    if not (fv / ".git").exists() and not (fv / "paper").exists():
        sys.exit(f"paper-mirror: {fv} is not a BDL_FV checkout")
    if argv[1] == "check":
        return check(fv)
    return refresh(fv, argv[3] if len(argv) > 3 else None)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
