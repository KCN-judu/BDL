#!/usr/bin/env python3
"""Validate BDL's engineering records (docs/project/governance.md).

Checks, without any third-party dependency:

* decisions (`docs/decisions/NNNN-*.md`): frontmatter present with the required
  fields, known status and area, id matches the file name and the title,
  ids unique, `supersedes` / `superseded-by` name existing records and agree
  in both directions, a `superseded` record names its replacement, every id
  is in the decision index;
* proposals (`docs/proposals/NNNN-*.md`): same discipline; an accepted
  proposal names its ADR;
* issues (`docs/issues/NNNN-*.md`): known state and area, unique ids, a
  resolved or deferred issue has a resolution, `resolved-by` names existing
  records, every id is in the issue index;
* change fragments (`docs/changes/unreleased/*.md`): the header bullets and
  the three sections;
* pages (`docs/{spec,architecture,evidence,guides,background,archive,project}/*.md`):
  a `kind` / `area` / `status` header whose kind matches the folder;
* the front door (`docs/README.md`) links every page and no `.md` sits
  loose at the top of `docs/`;
* every relative Markdown link under `docs/` and in `README.md` resolves.

Exit status 1 with one line per problem; 0 and "engineering records:
valid" otherwise.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

DECISION_STATUSES = {"accepted", "superseded", "rejected", "withdrawn"}
PROPOSAL_STATUSES = {"draft", "discussion", "accepted", "rejected", "withdrawn", "superseded"}
ISSUE_STATES = {"open", "deferred", "resolved"}
AREAS = {
    "language",
    "textual",
    "compiler",
    "runtime",
    "codegen",
    "persistence",
    "protocol",
    "daemon",
    "ide",
    "studio",
    "behavior-systems",
    "deployment",
    "formal",
    "process",
}
DECISION_FIELDS = {"id", "status", "date", "area", "supersedes", "superseded-by"}
PROPOSAL_FIELDS = {"id", "status", "date", "area", "related-issues", "superseded-by"}
ISSUE_FIELDS = {"id", "state", "area", "opened", "resolved-by"}
PAGE_FOLDERS = {
    "spec": "specification",
    "architecture": "architecture",
    "evidence": "evidence",
    "guides": "guide",
    "background": "background",
    "archive": "archive",
    "project": "project",
}
PAGE_STATUSES = {"current", "archived"}
CHANGE_BULLETS = ("- Date:", "- Area:", "- Affected:", "- Related:")
CHANGE_SECTIONS = ("## What changed", "## Compatibility and migration", "## Evidence")
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
LINK = re.compile(r"(?<!!)\[[^\]]*\]\(([^)\s]+)\)")


def parse_value(value: str):
    value = value.strip()
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        if not inner:
            return []
        return [part.strip().strip("\"'") for part in inner.split(",")]
    return value.strip("\"'")


def parse_frontmatter(text: str) -> dict[str, object] | None:
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None
    data: dict[str, object] = {}
    for line in lines[1:end]:
        if not line.strip() or line.lstrip().startswith("#") or ":" not in line:
            continue
        key, value = line.split(":", 1)
        data[key.strip()] = parse_value(value)
    return data


def as_list(value) -> list[str]:
    if isinstance(value, list):
        return value
    if value in (None, ""):
        return []
    return [str(value)]


class Validator:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.errors: list[str] = []
        self.known_ids: set[str] = set()

    def error(self, path: Path | None, message: str) -> None:
        where = f"{path.relative_to(self.root)}: " if path else ""
        self.errors.append(f"{where}{message}")

    # ---- records ---------------------------------------------------------

    def records(self, folder: str) -> list[tuple[Path, dict[str, object], str]]:
        out = []
        for path in sorted((self.root / folder).glob("[0-9][0-9][0-9][0-9]-*.md")):
            text = path.read_text(encoding="utf-8")
            meta = parse_frontmatter(text)
            if meta is None:
                self.error(path, "missing or malformed frontmatter")
                continue
            out.append((path, meta, text))
        return out

    def check_common(self, path: Path, meta: dict[str, object], prefix: str, fields: set[str]) -> str:
        missing = sorted(fields - meta.keys())
        if missing:
            self.error(path, f"missing fields {', '.join(missing)}")
        record_id = str(meta.get("id", ""))
        number = path.name[:4]
        if record_id != f"{prefix}-{number}":
            self.error(path, f"id {record_id!r} does not match the file name ({prefix}-{number})")
        title = re.search(r"^# (\S+):", path.read_text(encoding="utf-8"), re.M)
        if not title or title.group(1) != record_id:
            self.error(path, f"title heading must start with '# {record_id}:'")
        area = str(meta.get("area", ""))
        if area not in AREAS:
            self.error(path, f"unknown area {area!r}")
        for key in ("date", "opened"):
            if key in meta and not DATE.match(str(meta[key])):
                self.error(path, f"{key} must be YYYY-MM-DD")
        if record_id in self.known_ids:
            self.error(path, f"duplicate id {record_id}")
        self.known_ids.add(record_id)
        return record_id

    def check_index(self, index: Path, ids: list[str]) -> None:
        text = index.read_text(encoding="utf-8") if index.exists() else ""
        for record_id in ids:
            if record_id not in text:
                self.error(index, f"{record_id} is missing from the index")

    def check_decisions(self) -> None:
        recs = self.records("docs/decisions")
        by_id: dict[str, dict[str, object]] = {}
        paths: dict[str, Path] = {}
        for path, meta, _ in recs:
            record_id = self.check_common(path, meta, "ADR", DECISION_FIELDS)
            if str(meta.get("status")) not in DECISION_STATUSES:
                self.error(path, f"unknown decision status {meta.get('status')!r}")
            by_id[record_id] = meta
            paths[record_id] = path
        for record_id, meta in by_id.items():
            path = paths[record_id]
            for field in ("supersedes", "superseded-by"):
                for target in as_list(meta.get(field)):
                    if target not in by_id:
                        self.error(path, f"{field} names unknown record {target}")
            for old in as_list(meta.get("supersedes")):
                if old in by_id and record_id not in as_list(by_id[old].get("superseded-by")):
                    self.error(paths[old], f"must list {record_id} in superseded-by")
            for new in as_list(meta.get("superseded-by")):
                if new in by_id and record_id not in as_list(by_id[new].get("supersedes")):
                    self.error(paths[new], f"must list {record_id} in supersedes")
                if str(meta.get("status")) != "superseded":
                    self.error(path, "has superseded-by but status is not 'superseded'")
            if str(meta.get("status")) == "superseded" and not as_list(meta.get("superseded-by")):
                self.error(path, "status 'superseded' needs a superseded-by record")
        self.check_index(self.root / "docs/decisions/README.md", list(by_id))

    def check_proposals(self) -> None:
        recs = self.records("docs/proposals")
        ids = []
        for path, meta, text in recs:
            record_id = self.check_common(path, meta, "PRP", PROPOSAL_FIELDS)
            ids.append(record_id)
            status = str(meta.get("status"))
            if status not in PROPOSAL_STATUSES:
                self.error(path, f"unknown proposal status {status!r}")
            if status == "accepted" and not re.search(r"ADR-\d{4}", text):
                self.error(path, "an accepted proposal must name the ADR that records it")
        self.check_index(self.root / "docs/proposals/README.md", ids)

    def check_issues(self) -> None:
        recs = self.records("docs/issues")
        ids = []
        for path, meta, text in recs:
            record_id = self.check_common(path, meta, "ISS", ISSUE_FIELDS)
            ids.append(record_id)
            state = str(meta.get("state"))
            if state not in ISSUE_STATES:
                self.error(path, f"unknown issue state {state!r}")
            resolved_by = as_list(meta.get("resolved-by"))
            if state == "resolved" and not resolved_by:
                self.error(path, "a resolved issue must name what resolved it in resolved-by")
            for target in resolved_by:
                if re.match(r"(ADR|PRP|ISS)-\d{4}$", target) and target not in self.known_ids:
                    self.error(path, f"resolved-by names unknown record {target}")
                elif "/" in target and not (self.root / target.split("#")[0]).exists():
                    self.error(path, f"resolved-by path does not exist: {target}")
            if state in ("resolved", "deferred"):
                body = text.split("## Resolution", 1)
                if len(body) < 2 or len(body[1].strip()) < 10:
                    self.error(path, f"a {state} issue needs a Resolution section")
        self.check_index(self.root / "docs/issues/README.md", ids)

    def check_changes(self) -> None:
        folder = self.root / "docs/changes/unreleased"
        for path in sorted(folder.glob("*.md")):
            text = path.read_text(encoding="utf-8")
            if not text.startswith("# "):
                self.error(path, "a change fragment starts with a '# title' line")
            for bullet in CHANGE_BULLETS:
                if bullet not in text:
                    self.error(path, f"missing header bullet {bullet!r}")
            for section in CHANGE_SECTIONS:
                if section not in text:
                    self.error(path, f"missing section {section!r}")
            if not re.match(r"\d{4}-\d{2}-[a-z0-9-]+\.md$", path.name):
                self.error(path, "fragment file names are YYYY-MM-slug.md")

    # ---- indexes and links ------------------------------------------------

    def check_pages(self) -> None:
        for folder, kind in PAGE_FOLDERS.items():
            for path in sorted((self.root / "docs" / folder).glob("*.md")):
                if path.name in {"README.md", "TEMPLATE.md"}:
                    continue
                meta = parse_frontmatter(path.read_text(encoding="utf-8"))
                if meta is None:
                    self.error(path, "missing kind/area/status header")
                    continue
                if str(meta.get("kind")) != kind:
                    self.error(path, f"kind must be {kind!r} in docs/{folder}/")
                if str(meta.get("area")) not in AREAS:
                    self.error(path, f"unknown area {meta.get('area')!r}")
                if str(meta.get("status")) not in PAGE_STATUSES:
                    self.error(path, f"unknown status {meta.get('status')!r}")
                if folder == "archive" and str(meta.get("status")) != "archived":
                    self.error(path, "pages under docs/archive/ are status: archived")

    def check_front_door(self) -> None:
        front = self.root / "docs/README.md"
        text = front.read_text(encoding="utf-8") if front.exists() else ""
        for path in sorted((self.root / "docs").glob("*.md")):
            if path.name != "README.md":
                self.error(path, "loose page at the top of docs/: move it into its kind's folder")
        for folder in PAGE_FOLDERS:
            for path in sorted((self.root / "docs" / folder).glob("*.md")):
                if path.name in {"README.md", "TEMPLATE.md"}:
                    continue
                rel = f"{folder}/{path.name}"
                if f"({rel})" not in text and f"({rel}#" not in text:
                    self.error(front, f"{rel} is not registered in the front door")

    def check_links(self) -> None:
        files = [self.root / "README.md"] + sorted((self.root / "docs").rglob("*.md"))
        for path in files:
            if not path.exists():
                continue
            text = path.read_text(encoding="utf-8")
            for match in LINK.finditer(text):
                target = match.group(1)
                if "://" in target or target.startswith("mailto:") or target.startswith("#"):
                    continue
                target = target.split("#", 1)[0]
                if not target:
                    continue
                resolved = (path.parent / target).resolve()
                if not resolved.exists():
                    self.error(path, f"broken link {match.group(1)}")

    def run(self) -> list[str]:
        self.check_decisions()
        self.check_proposals()
        self.check_issues()
        self.check_changes()
        self.check_pages()
        self.check_front_door()
        self.check_links()
        return self.errors


def validate(root: Path) -> list[str]:
    return Validator(root).run()


def main() -> int:
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]
    errors = validate(root)
    for error in errors:
        print(error)
    if errors:
        print(f"engineering records: {len(errors)} problem(s)")
        return 1
    print("engineering records: valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
