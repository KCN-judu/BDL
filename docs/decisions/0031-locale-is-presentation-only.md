---
id: ADR-0031
status: accepted
date: 2026-09-19
area: studio
supersedes: []
superseded-by: []
related: [ADR-0001, ADR-0030]
fv: []
---

# ADR-0031: Locale is presentation only — three locales, English canonical

## Status

Accepted (first internationalization milestone).

## Context

Studio and the user guide were English-only, with every label a Dart literal and
every explanation an English page. Designers in Simplified Chinese and Japanese
are the next audience, and the product's own principle — semantic truth is
Rust-only (ADR-0001), Studio renders projections — leaves an obvious line to
draw: a locale may change how a projection is worded and never what it projects.
The pressure was to get translations in without letting a language leak into a
project file, a diagnostic identity or the compiler.

## Decision

1. **Locale is an application preference, never project state.** It is stored in
   `preferences.json` in the app-support directory beside the recent-projects
   list, loaded at startup, saved on change, and applied live. No project file,
   `.bdl` source, sidecar, protocol message or daemon request carries or reads
   it.
2. **Changing the locale changes presentation only.** BDL syntax, type notation,
   `()`, formal identities, protocol identities, diagnostic codes and user-given
   names are rendered identically in every locale. A diagnostic's code is its
   identity; Studio words a sentence for a code only when the code alone
   determines it, and otherwise shows the compiler's English sentence with the
   code beside it.
3. **Exactly three locales — English, Simplified Chinese, Japanese — with
   English canonical.** Every string and page is authored in English; the other
   two are translations of it, complete for the P0/P1 surfaces, checked in CI.
   An unsupported system locale resolves to English. A fourth locale requires an
   ADR superseding this list.
4. **Two pipelines, both reproducible from the repository.** Studio: Flutter
   gen-l10n over ARB files with typed accessors, ICU plurals and declared
   placeholders, no string concatenation in widgets. Guide: English Markdown →
   gettext POT → PO per locale → rendered pages, generated and committed.
5. **One terminology authority**, `locale/glossary.json`, machine-readable,
   listing for each product term its canonical English, context, zh-Hans and ja
   renderings, translator note and whether it is translatable at all. Studio
   strings and guide translations follow it.

## Alternatives

- _Per-project language_ (a `language` field in `bdl.toml`): rejected — a
  project shared between a Chinese and a Japanese designer would fight over a
  presentation setting, and the file would carry state the compiler has no use
  for.
- _Localizing diagnostics in the daemon_ (a locale in the handshake): rejected
  for this milestone — it moves presentation into the semantic layer and makes
  the daemon's text depend on a client preference; the remainder is ISS-0015,
  where a structured-argument diagnostic would let Studio word every sentence
  itself.
- _A third-party Markdown-i18n tool_ (mdpo and similar): rejected — none
  installed cleanly on the supported platforms, and the segmentation rules
  (never extract a fence, a front-matter line or a table delimiter) are a
  hundred lines that the repository can own and test.
- _Traditional Chinese as a fourth locale or as a fallback target for zh-Hans_:
  rejected — not a supported locale; resolving it to English is honest,
  resolving it to Simplified is a guess.

## Consequences

- Every Studio surface reads `context.l10n.key`; a bare literal in
  `apps/studio/lib/ui/` is a defect. Helpers outside the widget tree take an
  `AppLocalizations`; the effect executor labels OS dialogs from the tracked
  preference.
- `scripts/check_l10n.py`, `just l10n-check` and CI fail on a key missing from a
  translation, a placeholder mismatch, a glossary term rendered where it must
  not be, or a rendered guide page that is out of date.
- Records: `docs/project/localization-style.md` (the rules), ISS-0015 (the
  daemon-owned English remainder), a change fragment
  (`docs/changes/unreleased/2026-09-internationalization.md`), a status row.
- The two Chinese engineering documents (`docs/spec/kernel.md`,
  `docs/background/paper-digest.md`) are kept in English from now on; the
  engineering records are not a localized surface.
