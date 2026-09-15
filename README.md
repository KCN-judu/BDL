# BDL — Behavior Design Language (engineering implementation)

Engineering implementation of **BDL**, a behavior design language for industrial
designers in which typed semantic relationships (`?f : Tilt -> Brightness`) are
first-class design artifacts, an unresolved relationship is a legal state of the
design, and every designer-facing construct elaborates onto a small formal kernel.

The kernel is fixed by the mechanized development in
[KCN-judu/BDL_FV](https://github.com/KCN-judu/BDL_FV) (Lean 4, Phases 0–7, no
`sorry`). This repository builds what that development deliberately did not:
the elaborator, the tooling, and the execution path.

## Status

Project initialisation. No code yet — implementation language and delivery form
are still to be decided (see `docs/03-open-questions.md`).

## Layout

```
docs/
  01-paper-digest.md     paper summary oriented at implementation (zh)
  02-kernel-spec.md      exact kernel contract transcribed from the Lean sources (zh)
  03-open-questions.md   decisions the paper leaves to the implementation (zh)
reference/
  paper/                 the paper (Typst PDF + canonical markdown source)
```

## Reading order

1. `docs/01-paper-digest.md` — what BDL is and what the three layers are
2. `docs/02-kernel-spec.md` — the definitions the implementation must agree with
3. `docs/03-open-questions.md` — what has to be decided before writing code
4. `reference/paper/paper.md` — the paper itself, for anything not covered above
