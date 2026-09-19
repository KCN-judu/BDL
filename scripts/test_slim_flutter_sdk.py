"""scripts/slim_flutter_sdk.py removes exactly the listed trees and keeps
what the Flutter tool reads (docs/project/ci.md)."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import slim_flutter_sdk as slim  # noqa: E402


def fake_sdk(root: Path) -> None:
    for rel in [
        "bin/flutter",
        "bin/internal/engine.version",
        ".git/HEAD",
        ".pub-preload-cache/x.tar.gz",
        "bin/cache/dart-sdk/bin/dart",
        "bin/cache/artifacts/engine/windows-x64/flutter_windows.dll",
        "bin/cache/artifacts/engine/windows-x64-profile/flutter_windows.dll",
        "bin/cache/artifacts/engine/windows-x64-release/flutter_windows.dll",
        "bin/cache/artifacts/engine/common/flutter_patched_sdk/x",
        "bin/cache/artifacts/engine/android-arm64/x",
        "bin/cache/artifacts/engine/android-x64-release/x",
        "bin/cache/flutter_web_sdk/x",
        "packages/flutter/lib/material.dart",
        "engine/src/flutter/x",
        "dev/bots/x",
        "examples/hello/x",
        "docs/x",
    ]:
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(b"0" * 10)


class Slim(unittest.TestCase):
    def test_removes_the_unused_trees_and_keeps_the_rest(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d) / "flutter"
            fake_sdk(root)
            self.assertEqual(slim.main(["slim", str(root)]), 0)
            gone = [
                "bin/cache/artifacts/engine/android-arm64",
                "bin/cache/artifacts/engine/android-x64-release",
                "bin/cache/flutter_web_sdk",
                "engine",
                "dev",
                "examples",
                "docs",
            ]
            kept = [
                "bin/flutter",
                "bin/internal/engine.version",
                ".git/HEAD",
                ".pub-preload-cache/x.tar.gz",
                "bin/cache/dart-sdk/bin/dart",
                "bin/cache/artifacts/engine/windows-x64/flutter_windows.dll",
                "bin/cache/artifacts/engine/windows-x64-profile/flutter_windows.dll",
                "bin/cache/artifacts/engine/windows-x64-release/flutter_windows.dll",
                "bin/cache/artifacts/engine/common/flutter_patched_sdk/x",
                "packages/flutter/lib/material.dart",
            ]
            for rel in gone:
                self.assertFalse((root / rel).exists(), rel)
            for rel in kept:
                self.assertTrue((root / rel).exists(), rel)
            # idempotent
            self.assertEqual(slim.main(["slim", str(root)]), 0)

    def test_refuses_a_directory_that_is_not_an_sdk(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(slim.main(["slim", d]), 2)


if __name__ == "__main__":
    unittest.main()
