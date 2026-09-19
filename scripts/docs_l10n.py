#!/usr/bin/env python3
"""Gettext catalogs for the user guide (docs/project/localization-style.md).

English Markdown under ``docs/user-guide/`` is canonical.  This script

* ``extract`` — segments every guide page into translatable blocks (a
  paragraph, a heading, a list item, a table cell, a quote line) and writes
  ``locale/user-guide/user-guide.pot``.  Code fences, front matter, HTML
  comments, image-only lines and table delimiter rows are never extracted;
* ``merge`` — brings each locale's ``user-guide.po`` up to date with the
  POT, keeping existing translations and dropping messages the English
  no longer has;
* ``render`` — writes the localized pages under
  ``locale/user-guide/<lang>/…`` mirroring the source tree, for every page
  that has at least one translated block.  An untranslated block stays
  English and the page carries a notice; a link to a page with no
  translation at all points at the English original.

Locales are exactly zh_Hans and ja.  ``python3 scripts/docs_l10n.py all``
runs the three steps; ``just docs-l10n`` is the same.
"""

from __future__ import annotations

import os
import re
import sys
from collections import OrderedDict
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "docs/user-guide"
# Where the catalogs and pages live in the tree.  `--out` writes them
# elsewhere (a freshness check's scratch copy); links and the generated
# notice are still computed as if the pages sat at CANONICAL, so the two
# renderings compare byte for byte.
CANONICAL = ROOT / "locale/user-guide"
OUT = CANONICAL
POT = OUT / "user-guide.pot"
LANGS = ("zh_Hans", "ja")
# Working documents of the guide's authors, not pages of the guide.
INTERNAL = {"DOCUMENTATION_RESEARCH.md", "SCREENSHOT_PLAN.md", "STYLE_GUIDE.md", "VERIFICATION.md"}

STRINGS = {
    "zh_Hans": {
        "name": "简体中文",
        "language": "语言",
        "partial": "本页尚未完全翻译；未翻译的段落以英文显示。",
        "generated": "由 scripts/docs_l10n.py 从 {src} 生成；请编辑 {po}，不要编辑本文件。",
    },
    "ja": {
        "name": "日本語",
        "language": "言語",
        "partial": "このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。",
        "generated": "scripts/docs_l10n.py が {src} から生成しました。{po} を編集してください。このファイルは編集しないでください。",
    },
}

FENCE = re.compile(r"^\s*(`{3,}|~{3,})")
HEADING = re.compile(r"^(#{1,6}\s+)(.*?)(\s+#+)?\s*$")
LIST_ITEM = re.compile(r"^(\s*(?:[-*+]|\d+[.)])\s+)(.*)$")
TABLE_DELIM = re.compile(r"^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$")
QUOTE = re.compile(r"^(\s*>\s?)(.*)$")
IMAGE_ONLY = re.compile(r"^\s*!\[[^\]]*\]\([^)]*\)\s*$")
HTML = re.compile(r"^\s*<")
LINK = re.compile(r"(!?\[[^\]]*\]\()([^)\s]+)(\)|\s)")


@dataclass
class Block:
    """One piece of a page: either literal lines or a translatable text."""

    literal: list[str] = field(default_factory=list)
    prefix: str = ""
    text: str = ""
    line: int = 0
    cells: list[str] | None = None  # a table row: its cells
    row: str = ""  # the row's original text, for reconstruction


def split_cells(row: str) -> list[str]:
    body = row.strip()
    if body.startswith("|"):
        body = body[1:]
    if body.endswith("|"):
        body = body[:-1]
    return [c.strip() for c in re.split(r"(?<!\\)\|", body)]


def segment(lines: list[str]) -> list[Block]:
    blocks: list[Block] = []
    i = 0
    n = len(lines)
    in_fence: str | None = None
    front = False
    while i < n:
        line = lines[i]
        if i == 0 and line.strip() == "---":
            front = True
            blocks.append(Block(literal=[line]))
            i += 1
            continue
        if front:
            blocks.append(Block(literal=[line]))
            if line.strip() == "---":
                front = False
            i += 1
            continue
        m = FENCE.match(line)
        if in_fence:
            blocks.append(Block(literal=[line]))
            if m and m.group(1)[0] == in_fence[0] and len(m.group(1)) >= len(in_fence):
                in_fence = None
            i += 1
            continue
        if m:
            in_fence = m.group(1)
            blocks.append(Block(literal=[line]))
            i += 1
            continue
        if not line.strip() or HTML.match(line) or IMAGE_ONLY.match(line) or TABLE_DELIM.match(line):
            blocks.append(Block(literal=[line]))
            i += 1
            continue
        if line.startswith("<!--"):
            blocks.append(Block(literal=[line]))
            i += 1
            continue
        h = HEADING.match(line)
        if h:
            blocks.append(Block(prefix=h.group(1), text=h.group(2), line=i + 1))
            i += 1
            continue
        if line.lstrip().startswith("|") and line.rstrip().endswith("|"):
            blocks.append(Block(cells=split_cells(line), row=line, line=i + 1))
            i += 1
            continue
        q = QUOTE.match(line)
        if q:
            text = [q.group(2)]
            prefix = q.group(1)
            i += 1
            while i < n:
                q2 = QUOTE.match(lines[i])
                if not q2 or not q2.group(2).strip():
                    break
                text.append(q2.group(2))
                i += 1
            blocks.append(Block(prefix=prefix, text=" ".join(t.strip() for t in text), line=i))
            continue
        li = LIST_ITEM.match(line)
        if li:
            prefix, text = li.group(1), [li.group(2)]
            start = i
            i += 1
            indent = len(prefix)
            while i < n:
                nxt = lines[i]
                if not nxt.strip() or LIST_ITEM.match(nxt) or FENCE.match(nxt):
                    break
                if len(nxt) - len(nxt.lstrip()) < indent:
                    break
                text.append(nxt.strip())
                i += 1
            blocks.append(Block(prefix=prefix, text=" ".join(text), line=start + 1))
            continue
        # A paragraph: up to a blank line or a structural line.
        text = [line.strip()]
        start = i
        i += 1
        while i < n:
            nxt = lines[i]
            if (
                not nxt.strip()
                or FENCE.match(nxt)
                or HEADING.match(nxt)
                or LIST_ITEM.match(nxt)
                or QUOTE.match(nxt)
                or HTML.match(nxt)
                or (nxt.lstrip().startswith("|") and nxt.rstrip().endswith("|"))
            ):
                break
            text.append(nxt.strip())
            i += 1
        blocks.append(Block(text=" ".join(text), line=start + 1))
    return blocks


def pages() -> list[Path]:
    return sorted(
        p for p in SOURCE.rglob("*.md") if p.name not in INTERNAL and "assets" not in p.parts
    )


def po_escape(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def po_unescape(s: str) -> str:
    out = []
    i = 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s):
            nxt = s[i + 1]
            out.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}.get(nxt, nxt))
            i += 2
        else:
            out.append(c)
            i += 1
    return "".join(out)


def read_po(path: Path) -> OrderedDict[str, str]:
    """msgid → msgstr, header (msgid "") included under ""."""
    entries: OrderedDict[str, str] = OrderedDict()
    if not path.exists():
        return entries
    msgid: list[str] | None = None
    msgstr: list[str] | None = None
    target: list[str] | None = None

    def flush() -> None:
        if msgid is not None and msgstr is not None:
            entries["".join(msgid)] = "".join(msgstr)

    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line.startswith("msgid "):
            flush()
            msgid = [po_unescape(line[6:].strip()[1:-1])]
            msgstr = None
            target = msgid
        elif line.startswith("msgstr "):
            msgstr = [po_unescape(line[7:].strip()[1:-1])]
            target = msgstr
        elif line.startswith('"') and target is not None:
            target.append(po_unescape(line[1:-1]))
        elif not line:
            flush()
            msgid = msgstr = target = None
    flush()
    return entries


def write_po(path: Path, header: str, entries: list[tuple[str, list[str], str]]) -> None:
    out = ['msgid ""', 'msgstr ""']
    out += [po_escape(part + "\n") for part in header.rstrip("\n").split("\n")]
    out.append("")
    for msgid, refs, msgstr in entries:
        for r in refs:
            out.append(f"#: {r}")
        out.append("msgid " + po_escape(msgid))
        out.append("msgstr " + po_escape(msgstr))
        out.append("")
    path.write_text("\n".join(out).rstrip("\n") + "\n", encoding="utf-8")


def catalog() -> OrderedDict[str, list[str]]:
    """Every translatable text of the guide with its references, in page order."""
    cat: OrderedDict[str, list[str]] = OrderedDict()
    for page in pages():
        rel = page.relative_to(ROOT).as_posix()
        for b in segment(page.read_text(encoding="utf-8").splitlines()):
            texts = b.cells if b.cells is not None else ([b.text] if b.text else [])
            for t in texts:
                if t and not t.isspace() and not re.fullmatch(r"[-:\s|]+", t):
                    cat.setdefault(t, []).append(f"{rel}:{b.line}")
    return cat


HEADER = (
    "Project-Id-Version: BDL user guide\n"
    "MIME-Version: 1.0\n"
    "Content-Type: text/plain; charset=UTF-8\n"
    "Content-Transfer-Encoding: 8bit\n"
    "PO-Revision-Date: 2026-09-19 00:00+0000\n"
    "Last-Translator: BDL maintainers\n"
    "Language-Team: BDL maintainers\n"
)


def extract() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    cat = catalog()
    write_po(POT, HEADER, [(k, v, "") for k, v in cat.items()])
    print(f"docs-l10n: {len(cat)} messages → {POT.relative_to(ROOT)}")


def merge() -> None:
    cat = catalog()
    for lang in LANGS:
        po = OUT / lang / "user-guide.po"
        po.parent.mkdir(parents=True, exist_ok=True)
        old = read_po(po)
        header = old.get("", HEADER + f"Language: {lang}\n")
        if "Language:" not in header:
            header += f"Language: {lang}\n"
        entries = [(k, v, old.get(k, "")) for k, v in cat.items()]
        write_po(po, header, entries)
        done = sum(1 for _, _, s in entries if s)
        print(f"docs-l10n: {po.relative_to(ROOT)}: {done}/{len(entries)} translated")


def rewrite_links(text: str, src_dir: Path, out_dir: Path, rendered: set[Path]) -> str:
    def fix(m: re.Match[str]) -> str:
        target = m.group(2)
        if re.match(r"^[a-z]+:", target) or target.startswith("#"):
            return m.group(0)
        path, _, anchor = target.partition("#")
        resolved = (src_dir / path).resolve()
        if resolved in rendered:
            return m.group(0)
        new = os.path.relpath(resolved, out_dir)
        return f"{m.group(1)}{new}{'#' + anchor if anchor else ''}{m.group(3)}"

    return LINK.sub(fix, text)


def page_texts(blocks: list[Block]) -> list[str]:
    return [t for b in blocks for t in (b.cells or ([b.text] if b.text else [])) if t]


def translated_enough(blocks: list[Block], tr: dict[str, str]) -> bool:
    texts = page_texts(blocks)
    if not texts:
        return False
    title = next((b.text for b in blocks if b.prefix.startswith("#")), None)
    done = sum(1 for t in texts if tr.get(t))
    return bool(title and tr.get(title)) and done * 2 >= len(texts)


def render() -> None:
    all_pages = pages()
    for lang in LANGS:
        po = OUT / lang / "user-guide.po"
        tr = read_po(po)
        tr.pop("", None)
        s = STRINGS[lang]
        other_tr = {o: read_po(OUT / o / "user-guide.po") for o in LANGS if o != lang}
        # A page is rendered once its title and at least half of its blocks
        # are translated; below that a reader is better served by the English
        # page, so links point there instead.
        rendered: dict[Path, list[Block]] = {}
        for page in all_pages:
            blocks = segment(page.read_text(encoding="utf-8").splitlines())
            if translated_enough(blocks, tr):
                rendered[page] = blocks
        rendered_out = {
            (OUT / lang / p.relative_to(SOURCE)).resolve(): p for p in rendered
        }
        rendered_src = {p.resolve() for p in rendered}
        # Drop pages no longer rendered.
        for stale in (OUT / lang).rglob("*.md"):
            if stale.resolve() not in rendered_out:
                stale.unlink()
        count = 0
        for page, blocks in rendered.items():
            rel = page.relative_to(SOURCE)
            out_path = OUT / lang / rel
            # relative links are computed from the canonical location
            canon_path = CANONICAL / lang / rel
            out_path.parent.mkdir(parents=True, exist_ok=True)
            partial = False
            lines: list[str] = []

            def t(text: str) -> str:
                nonlocal partial
                got = tr.get(text, "")
                if not got:
                    partial = True
                    return text
                return got

            for b in blocks:
                if b.cells is not None:
                    lines.append("| " + " | ".join(t(c) if c else "" for c in b.cells) + " |")
                elif b.text:
                    lines.append(b.prefix + t(b.text))
                else:
                    lines.extend(b.literal)
            body = "\n".join(lines)
            body = rewrite_links(body, page.parent, canon_path.parent, rendered_src)
            src_rel = page.relative_to(ROOT).as_posix()
            po_rel = (CANONICAL / lang / "user-guide.po").relative_to(ROOT).as_posix()
            english = os.path.relpath(page, canon_path.parent)
            links = [f"[English]({english})"]
            for other in LANGS:
                if other == lang:
                    links.append(STRINGS[lang]["name"])
                else:
                    sibling = CANONICAL / other / rel
                    if translated_enough(blocks, other_tr[other]):
                        links.append(
                            f"[{STRINGS[other]['name']}]({os.path.relpath(sibling, canon_path.parent)})"
                        )
                    else:
                        links.append(STRINGS[other]["name"])
            head = [
                "<!-- " + s["generated"].format(src=src_rel, po=po_rel) + " -->",
                "",
                f"> {s['language']}: " + " · ".join(links),
            ]
            if partial:
                head += [">", f"> {s['partial']}"]
            out_path.write_text("\n".join(head) + "\n\n" + body + "\n", encoding="utf-8")
            count += 1
        print(f"docs-l10n: {lang}: {count} page(s) rendered")


def main(argv: list[str]) -> int:
    global OUT, POT
    args = list(argv[1:])
    # `--out DIR`: write the catalogs and pages under DIR instead of
    # locale/user-guide (a freshness check renders into a scratch copy and
    # compares, never touching the working tree)
    if "--out" in args:
        i = args.index("--out")
        if i + 1 >= len(args):
            print(__doc__)
            return 2
        OUT = Path(args[i + 1]).resolve()
        POT = OUT / "user-guide.pot"
        del args[i : i + 2]
    steps = args or ["all"]
    for step in steps:
        if step in ("extract", "all"):
            extract()
        if step in ("merge", "all"):
            merge()
        if step in ("render", "all"):
            render()
        if step not in ("extract", "merge", "render", "all"):
            print(__doc__)
            return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
