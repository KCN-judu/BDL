#!/usr/bin/env python3
"""The two Markdown normalisations Prettier does not do (`just docs-fmt`).

* A bare ``` fence gets the language `text` (a file tree, an ASCII
  diagram, a message excerpt), so markdownlint MD040 is satisfied. Fences
  that name a language and everything inside a fence are left alone.
* A table delimiter row written `|---|---|` becomes `| --- | --- |`, the
  spacing Prettier itself writes, so a table Prettier leaves unaligned
  (wider than the print width) still has one consistent style (MD060).

Usage: md_normalize.py FILE...   (rewrites in place, prints changed files)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

FENCE = re.compile(r"^(\s*)(`{3,}|~{3,})(.*)$")
DELIMITER = re.compile(r"^(\s*)\|(\s*:?-+:?\s*\|)+\s*$")


def space_delimiter(line: str) -> str:
    """`|---|:--:|` → `| --- | :--: |` (alignment colons and indent kept)."""
    indent = line[: len(line) - len(line.lstrip())]
    cells = line.strip().strip("|").split("|")
    return indent + "| " + " | ".join(c.strip() for c in cells) + " |\n"


def label(text: str) -> str:
    out = []
    open_marker: str | None = None
    for line in text.splitlines(keepends=True):
        if open_marker is None and DELIMITER.match(line.rstrip("\n")):
            out.append(space_delimiter(line))
            continue
        m = FENCE.match(line.rstrip("\n"))
        if m:
            indent, marker, rest = m.groups()
            if open_marker is None:
                open_marker = marker
                if rest.strip() == "":
                    line = f"{indent}{marker}text\n"
            elif marker[0] == open_marker[0] and len(marker) >= len(open_marker) and rest.strip() == "":
                open_marker = None
        out.append(line)
    return "".join(out)


def main(paths: list[str]) -> int:
    changed = 0
    for name in paths:
        path = Path(name)
        text = path.read_text(encoding="utf-8")
        new = label(text)
        if new != text:
            path.write_text(new, encoding="utf-8")
            changed += 1
            print(f"normalised: {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
