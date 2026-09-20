# Reference material

Documents this repository carries for its readers but does not own.

## `paper/` — the BDL Design and Formalization Monograph (mirror)

The **BDL Design and Formalization Monograph** is the authoritative narrative
design record of BDL: for every construct, why it is there, what was tried
instead, which theorem or counterexample decided it, how production implements
it, and what remains open. Its canonical source is the `paper/` directory of
[KCN-judu/BDL_FV](https://github.com/KCN-judu/BDL_FV); `paper.md` is the only
file whose prose is edited, and `body.typ` and
`BDL_behavior_design_language.pdf` are generated from it by that repository's
pipeline (`build.sh`: pandoc → Typst).

`reference/paper/` is a **mirror**: a byte-for-byte copy of `BDL_FV/paper/` at
the commit recorded in [`paper-mirror.toml`](paper-mirror.toml) — the Markdown
source, the Typst files, the bibliography, the assets, the archived
conference-era manuscript (`paper/archive/`), the generated `body.typ` and the
generated PDF. Nothing in it is edited in this repository; a change to the
monograph is made in BDL_FV and mirrored afterwards, and the generated
artifacts here are copies of the canonical ones, not a second build. The
monograph's own `README.md` and `NOTES.md` are part of the mirror and say the
same from the other side.

This mirror is documentation. No build, test or Studio path reads it, and
nothing in this repository depends on BDL_FV being checked out (ADR-0010).

- Refresh: `just paper-mirror ../BDL_FV [commit]` replaces the directory with
  `git archive` of that commit and rewrites `paper-mirror.toml`.
- Verify: `just paper-mirror-check ../BDL_FV` compares every file with the
  recorded commit and fails on any difference. Optional tooling; not part of
  `just check` or CI.
- Read: [`paper/paper.md`](paper/paper.md) or the PDF. The engineering-binding
  extract of the earlier conference-era manuscript is
  `docs/background/paper-digest.md` (historical).
