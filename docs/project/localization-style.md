---
kind: project
area: process
status: current
---

# Localization style

How BDL Studio and the user guide speak a language other than English, and what
never changes with the language. The authority for terms is the machine-readable
glossary [`locale/glossary.json`](../../locale/glossary.json); the decision is
[ADR-0031](../decisions/0031-locale-is-presentation-only.md).

## Supported locales

Exactly three: **English** (`en`), **Simplified Chinese** (`zh-Hans`) and
**Japanese** (`ja`). English is the canonical source of every string and every
page; the other two are translations of it. No other locale is offered, and a
system locale outside the three resolves to English (Traditional Chinese
included: it is not a supported locale, and a guess from `zh-Hans` would be
wrong more often than English is).

## English is canonical

- Studio's strings live in `apps/studio/lib/l10n/app_en.arb`; every other
  catalog carries the same keys and nothing else (`scripts/check_l10n.py`).
- The user guide is written in English under `docs/user-guide/`; the
  translations are gettext catalogs under `locale/user-guide/` and the rendered
  pages beside them are generated (`just docs-l10n`), never edited.
- A string or page missing in a translation shows English — the generated
  `AppLocalizations` class hierarchy falls back key by key; a guide page is
  rendered in a language only once its title and at least half of its blocks are
  translated, and a link to a page that is not rendered points at the English
  original. A raw key or an empty label is a bug.
- The engineering records under `docs/` (excluding the user guide) are English
  only and are not localized. Two were written in Chinese and are kept as
  English now: `docs/spec/kernel.md` and `docs/background/paper-digest.md`.

## What never changes with the locale

Changing the locale changes presentation and nothing else. Never translated,
transliterated or re-spelled:

| Kind                   | Examples                                                                              |
| ---------------------- | ------------------------------------------------------------------------------------- |
| BDL syntax             | `concept`, `mapping`, `delay`, `sync`, `output`, `all`, `any`, `map`, `filter`, `in`  |
| port kinds             | `requires`, `provides`, `parameter`                                                   |
| type notation          | `() -> B`, `Quantity(angle)`, `List<Reading>`, unit symbols `m`, `s`, `°C`, `rad`     |
| the empty product      | `()` — prose may say _empty product_ / 空积 / 空積, the notation stays `()`           |
| formal identities      | `SemanticId`, `DeclId`, Θ / Κ / β / Ω, kernel terms in the Explain panel              |
| protocol identities    | message and field names, enum values, `PROTOCOL_VERSION`                              |
| diagnostic codes       | `output.missing_driver`, `studio.not_connected` — identity, shown beside any sentence |
| names the user gave    | concepts, relationships, domains, outputs, components, files, boards, pins            |
| project files          | everything under the project folder; a locale never rewrites a byte of it             |
| product and file names | BDL, BDL Studio, Behavior Designer, `bdld`, `.bdl`, `bdl.toml`                        |

Diagnostics: the **code** is the identity and is stable across locales and
versions. Studio words a diagnostic itself only when the code alone determines
the sentence (`apps/studio/lib/l10n/diagnostics.dart`); every other diagnostic
keeps the sentence the compiler service composed, in English, with the code
beside it. [ISS-0015](../issues/0015-daemon-diagnostics-are-english.md) records
that remainder.

## Terminology

The glossary is the single authority. Each entry has the canonical English term,
the context it is used in, the zh-Hans and ja renderings, a translator note and
`translatable` (false for syntax, notation and identities). Rules:

- One term, one rendering, everywhere: a term rendered two ways in one locale is
  a defect, not a stylistic choice. Studio's strings, the guide and the glossary
  agree.
- **Source** (the role: a mapping without inputs and without a definition; a
  port's Source role) means _a value entering the behavior model from the
  environment_. Render it as 来源 / 入力元 — never as a sensor, a file, a signal
  source or source code. A `.bdl` file is a _source file_
  (源文件 / ソースファイル) and is a different term.
- Studio says _relationship_ for what the kernel and the protocol call a
  _mapping_; both renderings appear in the glossary and are used where the
  English uses them.
- A translated term that is ambiguous in the target language gets a translator
  note in the glossary and a `description` on the ARB key.

## Studio strings

- gen-l10n / ARB. Keys are camelCase; every key has an `@key` description; every
  placeholder is declared with its type; plurals use ICU `{count, plural, …}`
  (English `=1` and `other`; zh-Hans and ja `other` only — neither language
  inflects for number).
- No concatenation: a sentence with a variable part is one message with a
  placeholder, so word order is the translator's.
- Categories, for review and priority: **Interface** (labels, buttons, menus),
  **Tooltip**, **Diagnostic** (the sentences Studio words itself), **Guidance**
  (explanations, empty states), **Lifecycle** (save / close / reload questions).
  P0 — project manager, toolbar, save/close dialogs, page navigation, status
  line, preferences — and P1 — Design sidebar, Inspector, Formula Composer, Code
  view, Simulate, Deploy — are complete in all three locales; the completeness
  check fails CI on any missing key.
- Not localized in Studio (recorded, not forgotten): the Explain panel's formal
  vocabulary; the accessibility semantics of the canvas and the Formula Composer
  (P2); the compiler's own diagnostic sentences (ISS-0015).
- Layout: no hard-coded width around a translatable label; controls size to
  their text (the page bar's tabs, the toolbar's buttons, the sheet's buttons
  and segmented controls). Fonts are the platform's system fonts; nothing is
  bundled per script. Screenshots stay English.

## Punctuation and style

**zh-Hans** — full-width punctuation (，。：；？！「」（）) inside Chinese text;
a half-width space between Chinese and a Latin word, number or code span
(`Tilt / 90 deg`、`0.5`); no space before full-width punctuation; sentences end
in 。; UI labels have no trailing 。; 您 is not used — 你 throughout, as the
English uses _you_; verbs first for actions (打开项目、添加定义); keep the
English casing of product names.

**ja** — full-width punctuation (、。「」（）) inside Japanese text; a
half-width space between Japanese and a Latin word, number or code span; the
polite form (です・ます) in guidance and questions, plain noun phrases for
labels (保存、閉じる、環境設定); katakana loanwords where the glossary uses them
(コンポーネント、インスタンス、シミュレート), never invented ones; no
trailing 。 on labels; long vowel marks as the glossary spells them.

**Both** — dates and numbers as the platform formats them; keyboard shortcuts as
symbols (⌘S, ⇧⌘Z) unchanged; `…` (one character) for a menu item that opens a
sheet; product language, never compiler language (see
`docs/architecture/compiler-pipeline.md` for the diagnostic style Studio's
sentences follow).

## User guide

English Markdown → `locale/user-guide/user-guide.pot` → one
`locale/user-guide/<lang>/user-guide.po` per locale → rendered pages under
`locale/user-guide/<lang>/` mirroring the source tree. Units of translation are
a paragraph, a heading, a list item, a table cell, a quote line; code fences,
front matter, comments, image lines and table delimiters are never extracted.
Every rendered page carries a language line (English · 简体中文 ·日本語) and,
when any block is still English, a notice saying so. Links between translated
pages stay relative; links to assets and to untranslated pages point back into
`docs/user-guide/`. Screenshots are shared with the English guide (there are no
localized screenshot sets).

Translate the core workflow first — the guide's README, _Getting started_, the
Studio pages (workspace, canvas, inspector, formula editor, code view, simulate,
deploy) and the keyboard and project-file references — then the rest as it
stabilises.

## Reproducible build

```bash
cd apps/studio && flutter gen-l10n     # or: just studio-l10n — regenerate lib/l10n/app_localizations*.dart
python3 scripts/check_l10n.py          # every en key in zh and ja, placeholders match, glossary sound
just docs-l10n                         # POT → POs → locale/user-guide/<lang>/ (commit the result)
just l10n-check                        # the two above, and the rendered pages must be current
```

`flutter pub get` also regenerates the Dart classes (`generate: true` in
`pubspec.yaml`); the generated files are committed so a checkout builds without
the step. Translations are edited in the ARB files and the PO files only.

**Standard Library items** are the one exception to "edited in the ARB files":
their names, descriptions and search tags live in `locale/library/std.json` (by
item id and locale; English comes from `library/std/concepts.toml`), and
`just library-l10n` (`scripts/gen_library_l10n.py`) writes the `libItem_*` keys
into the three ARB catalogs and the id lookup
`apps/studio/lib/l10n/library_strings.dart`. Edit the JSON, regenerate, then
`just studio-l10n`; preflight's `l10n` check fails when the generated files are
behind. The library, the compiler service and the protocol carry English only
(ADR-0031, ADR-0032 amendment): an item is localized by id in Studio, and what
it creates — `RoomTemp`, `TempSensor` — is an identifier in every locale.

## Review

A change to a string or a page:

1. edits English first (`app_en.arb`, `docs/user-guide/`);
2. runs the completeness check; a new key is added to both translations in the
   same change (a placeholder English value is acceptable only with a
   `TODO(l10n)` in the translator note and a follow-up before release);
3. checks the glossary for every product term it introduces and adds the term
   when it is new;
4. is reviewed by someone who reads the target language for the P0/P1 surfaces;
   a machine or model draft is marked as such in the PO comment until reviewed.

Adding a locale is a decision, not a pull request: it needs an ADR that
supersedes ADR-0031's locale list, a glossary column, and both pipelines
extended — nothing in the code is written to make a fourth locale cheap.
