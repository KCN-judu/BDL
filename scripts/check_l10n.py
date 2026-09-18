#!/usr/bin/env python3
"""Completeness and consistency checks for the localization layer.

Exit non-zero when:

* a key of the English catalog (``app_en.arb``) is missing from
  ``app_zh.arb`` or ``app_ja.arb`` — every P0/P1 key ships in all three
  locales (docs/project/localization-style.md);
* a translation carries a placeholder set different from the English
  message (an ICU ``{name}`` that would go unfilled or be unknown);
* a translation names a key the English catalog does not have;
* a translation renders a term the glossary marks ``translatable: false``
  in anything but its English spelling;
* a user-guide PO catalog is missing a locale directory or a POT message
  is absent from a PO (untranslated is fine; unknown is not);
* the glossary lists a locale other than en, zh-Hans, ja.

Usage: ``python3 scripts/check_l10n.py`` from the repository root.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ARB_DIR = ROOT / "apps/studio/lib/l10n"
GLOSSARY = ROOT / "locale/glossary.json"
GUIDE_LOCALE = ROOT / "locale/user-guide"
LOCALES = ["en", "zh-Hans", "ja"]
ARBS = {"en": "app_en.arb", "zh-Hans": "app_zh.arb", "ja": "app_ja.arb"}

PLACEHOLDER = re.compile(r"\{(\w+)(?:,|\})")


def load_arb(name: str) -> dict[str, str]:
    data = json.loads((ARB_DIR / name).read_text(encoding="utf-8"))
    return {k: v for k, v in data.items() if not k.startswith("@")}


def placeholders(message: str) -> set[str]:
    return set(PLACEHOLDER.findall(message))


def check_arbs(problems: list[str]) -> None:
    en = load_arb(ARBS["en"])
    for locale, name in ARBS.items():
        if locale == "en":
            continue
        cat = load_arb(name)
        for key in en:
            if key not in cat:
                problems.append(f"{name}: missing key {key}")
            elif placeholders(cat[key]) != placeholders(en[key]):
                problems.append(
                    f"{name}: {key} placeholders {sorted(placeholders(cat[key]))} "
                    f"!= English {sorted(placeholders(en[key]))}"
                )
        for key in cat:
            if key not in en:
                problems.append(f"{name}: key {key} is not in app_en.arb")
    thin = json.loads((ARB_DIR / "app_zh_Hans.arb").read_text(encoding="utf-8"))
    if set(thin) != {"@@locale"}:
        problems.append("app_zh_Hans.arb must hold only @@locale; translations live in app_zh.arb")


def check_glossary(problems: list[str]) -> None:
    g = json.loads(GLOSSARY.read_text(encoding="utf-8"))
    if g.get("locales") != LOCALES:
        problems.append(f"glossary locales must be exactly {LOCALES}")
    seen: set[str] = set()
    for term in g["terms"]:
        for field in ("id", "en", "context", "zh-Hans", "ja", "note", "translatable"):
            if field not in term:
                problems.append(f"glossary term {term.get('id')!r} lacks {field}")
        if term["id"] in seen:
            problems.append(f"glossary term {term['id']} is listed twice")
        seen.add(term["id"])
        if term.get("translatable") is False:
            for loc in ("zh-Hans", "ja"):
                if term.get(loc) not in (term["en"], "(unchanged)"):
                    problems.append(f"glossary term {term['id']} is not translatable but {loc} differs")


def parse_po(path: Path) -> set[str]:
    """The msgids of a PO/POT file (multi-line strings joined)."""
    ids: set[str] = set()
    current: list[str] | None = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("msgid "):
            current = [json.loads(line[6:])]
        elif line.startswith("msgstr"):
            if current is not None:
                ids.add("".join(current))
            current = None
        elif line.startswith('"') and current is not None:
            current.append(json.loads(line))
    ids.discard("")
    return ids


def check_guide(problems: list[str]) -> None:
    pot = GUIDE_LOCALE / "user-guide.pot"
    if not pot.exists():
        return
    ids = parse_po(pot)
    for loc in ("zh_Hans", "ja"):
        po = GUIDE_LOCALE / loc / "user-guide.po"
        if not po.exists():
            problems.append(f"{po.relative_to(ROOT)} is missing")
            continue
        got = parse_po(po)
        for m in sorted(ids - got)[:20]:
            problems.append(f"{po.relative_to(ROOT)}: POT message absent: {m[:60]!r}")
    for entry in GUIDE_LOCALE.iterdir():
        if entry.is_dir() and entry.name not in ("zh_Hans", "ja"):
            problems.append(f"unexpected user-guide locale {entry.name}")


def main() -> int:
    problems: list[str] = []
    check_arbs(problems)
    check_glossary(problems)
    check_guide(problems)
    for p in problems:
        print(f"l10n: {p}")
    if problems:
        print(f"l10n: {len(problems)} problem(s)")
        return 1
    print("l10n: catalogs complete and consistent")
    return 0


if __name__ == "__main__":
    sys.exit(main())
