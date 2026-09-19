#!/usr/bin/env python3
"""Check the user guide's screenshots against their manifest and ledger.

The manifest (`docs/user-guide/screenshots/manifest.json`) says what each
screenshot shows and where it goes; the harness
(`apps/studio/test/docs_screenshots_test.dart`, `just docs-shots`) captures
them and writes the ledger (`docs/user-guide/screenshots/captured.json`)
with the commit and the hashes of the fixture and of the manifest entry
each image came from.  This script, run by `just docs-check`, verifies
without rendering anything that

* every manifest entry has its image on disk, at the size the ledger
  recorded;
* every page the entry names embeds the image with the manifest's alt text
  and carries its caption;
* every image embedded anywhere in the guide is a manifest output (no
  stray or hand-made screenshots);
* every entry is in the ledger and is not stale: the fixture and the
  entry hash still match what was captured — a changed fixture or scene
  means `just docs-shots` must run again;
* every entry is listed in `docs/user-guide/SCREENSHOT_PLAN.md`.

A ledger entry captured from a dirty checkout is reported as a warning
(`--strict` makes it an error, for CI).

Exit status 1 with one line per problem; 0 and "screenshots: valid"
otherwise.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

# The alt text may hold one level of balanced brackets (`Tilt(0.785398 [rad])`).
IMAGE = re.compile(r"!\[((?:[^\[\]]|\[[^\[\]]*\])*)\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
CODE = re.compile(r"```.*?```|`[^`\n]*`", re.DOTALL)


def images(text: str):
    """Image embeds outside code spans and fences."""
    return IMAGE.finditer(CODE.sub("", text))


def hash_tree(root: Path) -> str:
    """The harness's fixture hash: path and content of every file, sorted."""
    h = hashlib.sha256()
    for f in sorted(p for p in root.rglob("*") if p.is_file()):
        h.update(f"{f.relative_to(root).as_posix()}\n".encode())
        h.update(f.read_bytes())
        h.update(b"\n")
    return h.hexdigest()


def canonical(v: object) -> str:
    """The harness's canonical JSON: sorted keys, no whitespace."""
    if isinstance(v, dict):
        return "{" + ",".join(f"{json.dumps(k)}:{canonical(v[k])}" for k in sorted(v)) + "}"
    if isinstance(v, list):
        return "[" + ",".join(canonical(x) for x in v) + "]"
    return json.dumps(v, ensure_ascii=False)


def hash_entry(entry: dict) -> str:
    scene = {k: entry.get(k) for k in ["fixture", "view", "steps", "expect", "crop", "output"]}
    return hashlib.sha256(canonical(scene).encode()).hexdigest()


def png_size(path: Path) -> tuple[int, int] | None:
    data = path.read_bytes()[:24]
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big")


def check(root: Path, strict: bool) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    guide = root / "docs/user-guide"
    manifest_path = guide / "screenshots/manifest.json"
    ledger_path = guide / "screenshots/captured.json"
    plan_path = guide / "SCREENSHOT_PLAN.md"

    def error(path: Path, what: str) -> None:
        errors.append(f"{path.relative_to(root)}: {what}")

    if not manifest_path.exists():
        error(manifest_path, "missing")
        return errors, warnings
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    ledger = json.loads(ledger_path.read_text(encoding="utf-8")) if ledger_path.exists() else {}
    plan = plan_path.read_text(encoding="utf-8") if plan_path.exists() else ""
    assets = root / manifest["assets"]
    fixtures = root / manifest["fixtures"]

    outputs: dict[str, dict] = {}
    ids: set[str] = set()
    for entry in manifest["screenshots"]:
        sid = entry["id"]
        if sid in ids:
            error(manifest_path, f"{sid}: duplicate id")
        ids.add(sid)
        for key in ["id", "pages", "view", "steps", "expect", "crop", "output", "caption", "alt", "purpose"]:
            if key not in entry:
                error(manifest_path, f"{sid}: no `{key}`")
        if not entry.get("alt", "").strip():
            error(manifest_path, f"{sid}: empty alt text")
        if not entry.get("caption", "").strip():
            error(manifest_path, f"{sid}: empty caption")
        output = entry["output"]
        if output in outputs:
            error(manifest_path, f"{sid}: output {output} also produced by {outputs[output]['id']}")
        outputs[output] = entry

        image = assets / output
        if not image.exists():
            error(manifest_path, f"{sid}: {output} not captured (run `just docs-shots`)")
        fixture = entry.get("fixture")
        if fixture is not None and not (fixtures / fixture).is_dir():
            error(manifest_path, f"{sid}: fixture {fixture} does not exist")

        # the pages
        for page in entry.get("pages", []):
            page_path = guide / page
            if not page_path.exists():
                error(manifest_path, f"{sid}: page {page} does not exist")
                continue
            text = page_path.read_text(encoding="utf-8")
            rel = Path(*([".."] * (len(Path(page).parts) - 1))) / Path(manifest["assets"]).name / output
            found = [m for m in images(text) if m.group(2) == rel.as_posix()]
            if not found:
                error(page_path, f"does not embed {rel.as_posix()} ({sid})")
                continue
            for m in found:
                if " ".join(m.group(1).split()) != " ".join(entry["alt"].split()):
                    error(page_path, f"{sid}: alt text differs from the manifest")
            if " ".join(entry["caption"].split()) not in " ".join(text.split()):
                error(page_path, f"{sid}: caption not on the page")

        # the ledger
        rec = ledger.get(sid)
        if rec is None:
            error(ledger_path, f"{sid}: not captured yet (run `just docs-shots`)")
            continue
        if rec.get("entry_sha256") != hash_entry(entry):
            error(ledger_path, f"{sid}: stale — the manifest entry changed since capture")
        if fixture is not None and (fixtures / fixture).is_dir():
            if rec.get("fixture_sha256") != hash_tree(fixtures / fixture):
                error(ledger_path, f"{sid}: stale — fixture {fixture} changed since capture")
        if image.exists():
            size = png_size(image)
            if size is None:
                error(image, "not a PNG")
            elif size != (rec.get("width"), rec.get("height")):
                error(image, f"{sid}: size {size} differs from the ledger's {(rec.get('width'), rec.get('height'))}")
        if rec.get("dirty"):
            msg = f"{ledger_path.relative_to(root)}: {sid}: captured from a dirty checkout at {rec.get('commit')}"
            (errors if strict else warnings).append(msg)

        if sid not in plan:
            error(plan_path, f"{sid} is not listed")

    for sid in ledger:
        if sid not in ids:
            error(ledger_path, f"{sid}: not in the manifest (remove it and its image)")

    # every image in the guide is a manifest output
    known = {(assets / o).resolve() for o in outputs}
    for page_path in sorted(guide.rglob("*.md")):
        text = page_path.read_text(encoding="utf-8")
        for m in images(text):
            target = m.group(2)
            if "://" in target:
                continue
            resolved = (page_path.parent / target).resolve()
            if not resolved.exists():
                error(page_path, f"image {target} does not exist")
            elif resolved not in known:
                error(page_path, f"image {target} is not a manifest output")
    for image in sorted(assets.rglob("*.png")) if assets.exists() else []:
        if image.resolve() not in known:
            error(image, "not a manifest output (remove it or add an entry)")

    return errors, warnings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--strict", action="store_true", help="a capture from a dirty checkout is an error")
    args = parser.parse_args()
    errors, warnings = check(args.root, args.strict)
    for w in warnings:
        print(f"warning: {w}")
    for e in errors:
        print(e)
    if errors:
        print(f"screenshots: {len(errors)} problem(s)")
        return 1
    print("screenshots: valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
