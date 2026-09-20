"""scripts/gen_library_l10n.py: the catalog cannot drift from the library
silently, and the generator is deterministic and read-only under --check."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_library_l10n as g  # noqa: E402


def catalog(items: dict) -> dict:
    return {"$schema_version": 1, "items": items}


ENTRY = {
    "zh-Hans": {"name": "温度", "description": "…", "tags": "温度"},
    "ja": {"name": "温度", "description": "…", "tags": "温度"},
}


class Validation(unittest.TestCase):
    def test_the_standard_catalog_is_complete(self) -> None:
        all_items = g.items()
        l10n = json.loads(g.L10N.read_text(encoding="utf-8"))
        g.validate(all_items, l10n)
        self.assertEqual(len(all_items), 22)
        self.assertEqual(len(l10n["items"]), 22)

    def test_a_missing_item_is_refused(self) -> None:
        items = [{"id": "std.a", "name": "A", "description": "", "keywords": []}]
        with self.assertRaises(SystemExit) as e:
            g.validate(items, catalog({}))
        self.assertIn("std.a: no entry", str(e.exception))

    def test_an_orphan_entry_is_refused(self) -> None:
        items = [{"id": "std.a", "name": "A", "description": "", "keywords": []}]
        with self.assertRaises(SystemExit) as e:
            g.validate(items, catalog({"std.a": ENTRY, "std.gone": ENTRY}))
        self.assertIn("std.gone: not an item", str(e.exception))

    def test_an_empty_field_or_a_missing_locale_is_refused(self) -> None:
        items = [{"id": "std.a", "name": "A", "description": "", "keywords": []}]
        blank = {"zh-Hans": {"name": " ", "description": "d", "tags": "t"}}
        with self.assertRaises(SystemExit) as e:
            g.validate(items, catalog({"std.a": blank}))
        msg = str(e.exception)
        self.assertIn("zh-Hans name is empty", msg)
        self.assertIn("no ja entry", msg)

    def test_check_is_deterministic_and_touches_nothing(self) -> None:
        before = {p: p.read_bytes() for p in [g.DART, *(g.ARB_DIR / n for n in g.ARBS.values())]}
        for _ in range(2):
            r = subprocess.run([sys.executable, str(Path(g.__file__)), "--check"], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        after = {p: p.read_bytes() for p in before}
        self.assertEqual(before, after)
        # the same inputs render the same outputs
        all_items = g.items()
        l10n = json.loads(g.L10N.read_text(encoding="utf-8"))
        self.assertEqual(g.dart(all_items), g.dart(all_items))
        self.assertEqual(g.strings_for("ja", all_items, l10n), g.strings_for("ja", all_items, l10n))


if __name__ == "__main__":
    unittest.main()
