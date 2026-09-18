# Studio and the user guide in English, Simplified Chinese and Japanese

- Date: 2026-09-19
- Area: studio, process
- Affected: designers, developers
- Related: ADR-0031, ISS-0015

## What changed

- **A language preference.** _Preferences…_ (the gear at the right of the page
  bar, the link on the project manager) offers _System Default_, _English_,
  _简体中文_ and _日本語_. The choice is saved in `preferences.json` beside the
  recent-projects list — outside every project — and applies at once, without a
  restart. A system locale other than the three shows English.
- **Studio speaks all three.** The project manager, toolbar, save/close
  questions, page bar, status line, preferences, the Design sidebar, the
  Inspector, the Formula Composer, the Code view, Simulate and Deploy are
  translated. Studio's own error sentences (not connected, changed on disk,
  stale edit, …) are translated by code; the compiler's diagnostic sentences
  stay English with their code beside them (ISS-0015). The Explain panel's
  formal vocabulary stays English by design.
- **What never changes with the language:** BDL syntax and keywords, type
  notation and `()`, formal and protocol identities, diagnostic codes, the
  names you gave things, and every byte of a project file.
- **The user guide** has Simplified Chinese and Japanese editions of its core
  workflow — the front page, _Getting started_ (five pages), the Studio pages
  for the workspace, canvas, inspector, formula editor, code view, simulate and
  deploy, and the keyboard and project-file references — under
  `locale/user-guide/zh_Hans/` and `locale/user-guide/ja/`, each page with a
  language line and a notice when a passage is still English; other pages link
  to the English original.
- **For developers:** strings live in `apps/studio/lib/l10n/app_*.arb`
  (gen-l10n, `context.l10n.key`), terms in `locale/glossary.json`, guide
  catalogs in `locale/user-guide/*.po`; `just l10n-check` and CI fail on a key
  missing from a translation, a placeholder mismatch or a stale rendered page.
  `docs/project/localization-style.md` is the rule book. `docs/spec/kernel.md`
  and `docs/background/paper-digest.md` are now English.

## Compatibility and migration

Nothing to migrate. Projects are untouched by the language; `preferences.json`
is created on first change and ignored when absent or unreadable (defaults).
Protocol unchanged.

## Evidence

`apps/studio/test/l10n_test.dart` (locales, resolution, persistence, live
re-rendering, presentation-only invariants, layout in zh-Hans and ja),
`scripts/test_docs_l10n.py` (guide segmentation, PO round-trip, render
threshold, link rewriting), `scripts/check_l10n.py` over the real catalogs
(in `just docs-check` and CI, with `just docs-l10n` required to leave
`locale/` unchanged).
