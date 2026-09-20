# The BDL Design and Formalization Monograph

This directory holds the monograph: the living technical record of BDL,
written as one argument in seven Parts — why behavior needs a design medium;
the architecture; one interactive physical product walked through the whole
lifecycle; the formal model derived in dependency order; the toolchain by
responsibility; Studio and the IDE as a design argument; the evidence, the
rejected designs and the open agenda — with the reference apparatus in the
appendices (Appendix J maps the previous revision's sections to this one). It
is not a paper and is not prepared for publication; the directory
keeps its historical name because renaming it would break history. The
reader-facing name is _the monograph_ or _the record_.

## Files

- `paper.md` — the canonical source. Everything is edited here.
- `main.typ` — the document chrome: a single-column monograph layout, the
  title page (revision date, the formal and production snapshots), the table
  of contents (three levels), the bibliography.
- `body.typ` — **generated** from `paper.md` by Pandoc; never edited by hand.
- `BDL_behavior_design_language.pdf` — **generated** by Typst from `main.typ`.
- `references.bib` — the bibliography.
- `assets/` — figures.
- `NOTES.md` — the dated changelog of the document's revisions (its entries
  are historical and keep the vocabulary and identifiers of their date).
- `archive/` — historical snapshots. `paper-2026-09-conference-manuscript.md`
  is the conference-style manuscript the monograph grew out of; it is kept as
  it was submitted, with its own claims, vocabulary and the old decision
  numbers, and is not updated. The monograph is authoritative wherever the two
  differ.
- `build.sh` — the build.

## Build

```bash
./build.sh
```

The script regenerates `body.typ` from `paper.md` with Pandoc (verified with
3.11), patches the symbol names Pandoc emits for pre-0.13 Typst
(`bracket.l.double`, `angle.l`, `gt.tri`, …) to their current names, and
compiles `main.typ` with Typst (0.13 or later; verified with 0.15) to the PDF.
Without Pandoc the existing `body.typ` is used; without Typst the script stops
after regenerating it. The build is deterministic: the same `paper.md` yields
the same `body.typ`. No external Typst packages are used; fonts fall back to the
bundled Libertinus family.

## Conventions

- Production is described as of one pinned commit, stated in the preface
  and in Appendix F, and the formal repository's canonical copy of that hash is
  the `snapshot` field of `docs/project/production-correspondence.md`.
- Formal decisions are cited as `FVD-NNNN`, formal open items as `FVI-NNNN`,
  production records as `ADR`/`PRP`/`ISS-NNNN`; a formal result and the
  production record it concerns are cited together. The map from the retired
  `D-NN` numbers is Appendix E.
- Theorem names are Lean identifiers and never change to track a document id.
- Every substantive claim carries one of the strength labels of the preface;
  the full vocabulary is Appendix I.

## Canonical source and the production mirror

There is one BDL monograph. Its canonical source is this directory,
`KCN-judu/BDL_FV/paper/`: `paper.md` is the only file whose prose is edited,
and `body.typ` and the PDF are generated from it here. The production
repository `KCN-judu/BDL` may carry a copy under `reference/paper/` as a
reference mirror for readers of that repository. The mirror is a copy of a
committed revision of this directory, refreshed after a revision lands here; it
is never edited in place, never carries prose this directory does not, and is
not a second authority — a change proposed against the mirror is made here and
mirrored, not the other way round. Nothing in the production repository decides
what the monograph says; production's own records (`docs/`) are cited by it.
