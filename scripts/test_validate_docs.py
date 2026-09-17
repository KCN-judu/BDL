"""Unit tests for scripts/validate_docs.py: each check fails on the mistake
it exists for and passes on a minimal correct tree.

Run with `python3 -m unittest scripts/test_validate_docs.py`.
"""

from __future__ import annotations

import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from validate_docs import validate  # noqa: E402

ADR = """---
id: ADR-0001
status: accepted
date: 2026-09-15
area: studio
supersedes: []
superseded-by: []
---
# ADR-0001: Something

## Decision
"""

ISSUE = """---
id: ISS-0001
state: open
area: language
opened: 2026-09-15
resolved-by: []
---
# ISS-0001: A problem

## Problem

## Resolution

Open.
"""

CHANGE = """# A change

- Date: 2026-09-17
- Area: process
- Affected: developers
- Related: none

## What changed

## Compatibility and migration

## Evidence
"""


class Tree:
    def __init__(self) -> None:
        self.dir = Path(tempfile.mkdtemp())
        self.root = self.dir / "repo"
        for folder in ("docs/decisions", "docs/proposals", "docs/issues", "docs/changes/unreleased"):
            (self.root / folder).mkdir(parents=True)
        self.write("README.md", "# BDL\n\n[docs](docs/README.md)\n")
        self.write("docs/README.md", "# Front door\n\n[decisions](decisions/README.md) [issues](issues/README.md) [proposals](proposals/README.md)\n")
        self.write("docs/decisions/README.md", "# ADRs\n\nADR-0001\n")
        self.write("docs/issues/README.md", "# Issues\n\nISS-0001\n")
        self.write("docs/proposals/README.md", "# Proposals\n")
        self.write("docs/decisions/0001-something.md", ADR)
        self.write("docs/issues/0001-a-problem.md", ISSUE)
        self.write("docs/changes/unreleased/2026-09-a-change.md", CHANGE)

    def write(self, rel: str, text: str) -> None:
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def errors(self) -> list[str]:
        return validate(self.root)

    def cleanup(self) -> None:
        shutil.rmtree(self.dir)


class ValidateDocsTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tree = Tree()

    def tearDown(self) -> None:
        self.tree.cleanup()

    def test_minimal_tree_is_valid(self) -> None:
        self.assertEqual(self.tree.errors(), [])

    def test_duplicate_decision_id_is_reported(self) -> None:
        self.tree.write("docs/decisions/0002-other.md", ADR.replace("0001-something", "0002-other"))
        errors = self.tree.errors()
        self.assertTrue(any("does not match the file name" in e for e in errors), errors)
        self.assertTrue(any("duplicate id ADR-0001" in e for e in errors), errors)

    def test_unknown_status_and_area_are_reported(self) -> None:
        self.tree.write("docs/decisions/0001-something.md", ADR.replace("status: accepted", "status: proposed").replace("area: studio", "area: misc"))
        errors = self.tree.errors()
        self.assertTrue(any("unknown decision status 'proposed'" in e for e in errors), errors)
        self.assertTrue(any("unknown area 'misc'" in e for e in errors), errors)

    def test_supersession_must_point_both_ways(self) -> None:
        new = ADR.replace("ADR-0001", "ADR-0002").replace("supersedes: []", "supersedes: [ADR-0001]")
        self.tree.write("docs/decisions/0002-newer.md", new)
        self.tree.write("docs/decisions/README.md", "# ADRs\n\nADR-0001 ADR-0002\n")
        errors = self.tree.errors()
        self.assertTrue(any("0001-something.md: must list ADR-0002 in superseded-by" in e for e in errors), errors)
        old = ADR.replace("superseded-by: []", "superseded-by: [ADR-0002]").replace("status: accepted", "status: superseded")
        self.tree.write("docs/decisions/0001-something.md", old)
        self.assertEqual(self.tree.errors(), [])

    def test_superseded_by_without_status_is_reported(self) -> None:
        new = ADR.replace("ADR-0001", "ADR-0002").replace("supersedes: []", "supersedes: [ADR-0001]")
        self.tree.write("docs/decisions/0002-newer.md", new)
        self.tree.write("docs/decisions/README.md", "# ADRs\n\nADR-0001 ADR-0002\n")
        self.tree.write("docs/decisions/0001-something.md", ADR.replace("superseded-by: []", "superseded-by: [ADR-0002]"))
        errors = self.tree.errors()
        self.assertTrue(any("status is not 'superseded'" in e for e in errors), errors)

    def test_missing_index_row_is_reported(self) -> None:
        self.tree.write("docs/decisions/README.md", "# ADRs\n")
        errors = self.tree.errors()
        self.assertTrue(any("ADR-0001 is missing from the index" in e for e in errors), errors)

    def test_resolved_issue_needs_a_resolution(self) -> None:
        self.tree.write("docs/issues/0001-a-problem.md", ISSUE.replace("state: open", "state: resolved"))
        errors = self.tree.errors()
        self.assertTrue(any("must name what resolved it" in e for e in errors), errors)
        fixed = ISSUE.replace("state: open", "state: resolved").replace("resolved-by: []", "resolved-by: [ADR-0001]").replace("Open.", "Resolved by ADR-0001, which chose X.")
        self.tree.write("docs/issues/0001-a-problem.md", fixed)
        self.assertEqual(self.tree.errors(), [])
        self.tree.write("docs/issues/0001-a-problem.md", fixed.replace("ADR-0001]", "ADR-0009]"))
        errors = self.tree.errors()
        self.assertTrue(any("unknown record ADR-0009" in e for e in errors), errors)

    def test_accepted_proposal_names_its_adr(self) -> None:
        prp = """---
id: PRP-0001
status: accepted
date: 2026-09-17
area: language
related-issues: []
superseded-by: []
---
# PRP-0001: A proposal

## Outcome

Accepted.
"""
        self.tree.write("docs/proposals/0001-a-proposal.md", prp)
        self.tree.write("docs/proposals/README.md", "# Proposals\n\nPRP-0001\n")
        errors = self.tree.errors()
        self.assertTrue(any("must name the ADR" in e for e in errors), errors)
        self.tree.write("docs/proposals/0001-a-proposal.md", prp.replace("Accepted.", "Accepted as ADR-0001."))
        self.assertEqual(self.tree.errors(), [])

    def test_change_fragment_shape(self) -> None:
        self.tree.write("docs/changes/unreleased/2026-09-bad.md", "# Bad\n\n- Date: x\n")
        errors = self.tree.errors()
        self.assertTrue(any("missing header bullet '- Area:'" in e for e in errors), errors)
        self.assertTrue(any("missing section '## What changed'" in e for e in errors), errors)

    def test_loose_top_level_page_is_reported(self) -> None:
        self.tree.write("docs/NEW_PAGE.md", "# New\n")
        errors = self.tree.errors()
        self.assertTrue(any("loose page at the top of docs/" in e for e in errors), errors)

    def test_pages_need_a_matching_kind_header_and_a_front_door_row(self) -> None:
        self.tree.write("docs/spec/thing.md", "---\nkind: architecture\narea: protocol\nstatus: current\n---\n# Thing\n")
        errors = self.tree.errors()
        self.assertTrue(any("kind must be 'specification'" in e for e in errors), errors)
        self.assertTrue(any("spec/thing.md is not registered" in e for e in errors), errors)
        self.tree.write("docs/spec/thing.md", "---\nkind: specification\narea: protocol\nstatus: current\n---\n# Thing\n")
        self.tree.write("docs/README.md", "# Front door\n\n[decisions](decisions/README.md) [issues](issues/README.md) [proposals](proposals/README.md) [thing](spec/thing.md)\n")
        self.assertEqual(self.tree.errors(), [])
        self.tree.write("docs/archive/old.md", "---\nkind: archive\narea: process\nstatus: current\n---\n# Old\n")
        self.tree.write("docs/README.md", "# Front door\n\n[decisions](decisions/README.md) [issues](issues/README.md) [proposals](proposals/README.md) [thing](spec/thing.md) [old](archive/old.md)\n")
        errors = self.tree.errors()
        self.assertTrue(any("status: archived" in e for e in errors), errors)

    def test_broken_relative_link_is_reported(self) -> None:
        self.tree.write("docs/issues/README.md", "# Issues\n\nISS-0001 [gone](../adr/0009-missing.md#x)\n")
        errors = self.tree.errors()
        self.assertTrue(any("broken link ../adr/0009-missing.md#x" in e for e in errors), errors)


if __name__ == "__main__":
    unittest.main()
