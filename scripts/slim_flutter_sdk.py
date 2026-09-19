#!/usr/bin/env python3
"""Trim a freshly installed Flutter SDK to what this repository builds with,
before CI caches it (docs/project/ci.md, "Windows Flutter SDK").

The stable Windows archive is 3.5 GB / 23 000 files unpacked, and
restoring it from the Actions cache costs about a minute per job, most of
it extraction.  BDL builds a desktop app and runs `flutter test`: the
Android engine artifacts (830 MB), the web SDK, the engine sources and the
framework's `dev/`, `examples/` and `docs/` trees (15 000 files) are never
read.  Removing them before the cache is saved makes every later restore
smaller and faster; the Flutter tool re-downloads an artifact only when a
command needs it, and none of ours do.

Kept on purpose: `.git` (the tool reads the version from it), the Windows
engine artifacts in every mode (the tool checks all three exist for a
desktop build), `.pub-preload-cache` (seeds the first `pub get`).

Usage: ``python3 scripts/slim_flutter_sdk.py [FLUTTER_ROOT]`` — the root
defaults to ``$FLUTTER_ROOT``.  Prints what was removed and the size
before and after; exits non-zero when the root is not a Flutter SDK.
"""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path

# Relative to the SDK root; a trailing `*` is a prefix match on the last
# path component.
PRUNE = (
    "bin/cache/artifacts/engine/android-*",
    "bin/cache/flutter_web_sdk",
    "engine",
    "dev",
    "examples",
    "docs",
)


def measure(root: Path) -> tuple[int, int]:
    size = files = 0
    for dirpath, _dirs, names in os.walk(root):
        for n in names:
            try:
                size += (Path(dirpath) / n).stat().st_size
            except OSError:
                continue
            files += 1
    return size, files


def targets(root: Path) -> list[Path]:
    out: list[Path] = []
    for pattern in PRUNE:
        parent, _, last = pattern.rpartition("/")
        base = root / parent if parent else root
        if last.endswith("*"):
            if base.is_dir():
                out.extend(sorted(p for p in base.iterdir() if p.name.startswith(last[:-1])))
        else:
            p = base / last
            if p.exists():
                out.append(p)
    return out


def main(argv: list[str]) -> int:
    root = Path(argv[1] if len(argv) > 1 else os.environ.get("FLUTTER_ROOT", ""))
    if not root or not (root / "bin" / "flutter").exists():
        print(f"not a Flutter SDK: {root or '(no FLUTTER_ROOT)'}", file=sys.stderr)
        return 2
    before = measure(root)
    removed = 0
    for p in targets(root):
        size, files = measure(p) if p.is_dir() else (p.stat().st_size, 1)
        shutil.rmtree(p) if p.is_dir() else p.unlink()
        removed += size
        print(f"removed {p.relative_to(root)}: {size / 1e6:.0f} MB, {files} files")
    after = measure(root)
    print(
        f"Flutter SDK {root}: {before[0] / 1e9:.2f} GB / {before[1]} files"
        f" -> {after[0] / 1e9:.2f} GB / {after[1]} files ({removed / 1e6:.0f} MB removed)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
