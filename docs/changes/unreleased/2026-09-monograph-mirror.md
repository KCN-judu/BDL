# `reference/paper/` is the mirror of the BDL Design and Formalization Monograph

- Date: 2026-09-20
- Area: process
- Affected: developers, coding agents
- Related: ADR-0010

## What changed

- `reference/paper/` no longer holds the conference-era manuscript (_BDL: A
  Behavior Design Language_, 2026-09-15) as the current paper. It is now a
  byte-for-byte mirror of `paper/` in `KCN-judu/BDL_FV` at commit
  `cbb3ec1dc46de7d1fb67c022b714c3b052ad7dcf`: the **BDL Design and Formalization
  Monograph** — `paper.md` (the one edited source), `main.typ`, the generated
  `body.typ` and `BDL_behavior_design_language.pdf`, `references.bib`,
  `assets/`, `build.sh`, the monograph's own `README.md` and `NOTES.md`, and
  `archive/paper-2026-09-conference-manuscript.md`, where the earlier manuscript
  now lives as history. Every file is identical to the canonical one; nothing
  was excluded.
- The canonical authority is `BDL_FV/paper/`. The mirror is never edited here: a
  change to the monograph is made in BDL_FV and mirrored afterwards, and the
  generated `body.typ` and PDF are copies of the canonical artifacts, not a
  second build. `reference/README.md` says so; `reference/paper-mirror.toml`
  records the source commit and the production snapshot the monograph describes
  (`6be778b`).
- `scripts/paper_mirror.py` (`just paper-mirror-check`, `just paper-mirror`)
  compares or refreshes the mirror from a local BDL_FV checkout. It is optional
  reference tooling: not in `just check`, not in CI, and no build, test or
  Studio path reads the mirror or needs BDL_FV present (ADR-0010).
- Current documentation points at the monograph by that name: the repository
  README, the engineering front door, the getting-started guide, the spec and
  architecture pages that said "the paper". `docs/background/paper-digest.md` is
  marked as the historical digest of the archived manuscript.
- No production semantics changed: no compiler, runtime, Studio, protocol, model
  or formal-correspondence content was touched.

## Compatibility and migration

Nothing to migrate. A link to `reference/paper/paper.md` now opens the
monograph; the earlier manuscript is
`reference/paper/archive/paper-2026-09-conference-manuscript.md` (and Git
history). Historical records that cite the earlier paper keep their wording.

## Evidence

`python3 scripts/paper_mirror.py check ../BDL_FV` — ten files identical to
BDL_FV `cbb3ec1` `paper/` (sha-256 per file); `just docs-check`;
`python3 scripts/preflight.py fast`.
