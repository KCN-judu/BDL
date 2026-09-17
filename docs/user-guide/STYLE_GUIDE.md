# User-guide style guide

For anyone adding to `docs/user-guide/`. Short enough to read before writing a
page; the principles behind it are in
[DOCUMENTATION_RESEARCH.md](DOCUMENTATION_RESEARCH.md).

## Who is reading

A designer, prototyper or embedded developer who has never seen the repository.
Assume they know their product and a little about sensors; assume nothing about
compilers, type theory, Rust, Lean, or this codebase. Write for them first; give
the language expert a _Going deeper_ tail, not the page.

## One page, one kind

| Directory          | Kind           | Voice                           | Shape                                                                                                           |
| ------------------ | -------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `getting-started/` | tutorial       | second person, imperative steps | Goal · Time · numbered steps with "what you see" after each · What you made · If something does not work · Next |
| `concepts/`        | idea           | impersonal, present tense       | what it is · the rules in designer terms · what the interface shows · Going deeper · Related                    |
| `studio/`          | interface      | impersonal                      | a diagram of the region · what each part means · gestures / fields as tables · Related                          |
| `workflows/`       | recipe         | second person                   | Goal · Steps · What BDL means by this · If it does not work · Next / Related                                    |
| `textual/`         | current status | impersonal                      | works today / not there yet, explicitly                                                                         |
| `troubleshooting/` | situation      | second person                   | the sentence you see → what it means → why BDL insists → what to do → code                                      |
| `reference/`       | lookup         | tables                          | no prose beyond a lead sentence                                                                                 |

Do not mix kinds. A tutorial that starts explaining the ladder has become a
concept page; a concept page with numbered steps has become a workflow. Link
instead.

## Terminology

Use the words in [reference/terminology.md](reference/terminology.md), one per
idea: _concept_, _relationship_, _value form_, _timing domain_, _physical
output_, _behavior_ (group), _component_, _instance_, _binding_, _finding_,
_fix_. Where Studio's label differs (_Mappings_, _New mapping_), say so once on
the page and use the guide's word after.

Never use in designer-facing text: `DeclId`, `SemanticId`, elaboration, IR,
judgment, flatten, freshen, realization, `Grant`, `Clocked`, or any code
identifier. If one must appear, explain the user-facing idea first and put the
term in a _Going deeper_ tail or an _Explain_ description.

Keep these apart, always: relationship ≠ execution step; canvas edge ≠ execution
order; behavior group ≠ component; a group box's socket ≠ a port; a
relationship's produced concept ≠ a physical output; same value form ≠ same
concept; fan-out ≠ copying a value.

## Tone and sentences

- Second person in tutorials, workflows and troubleshooting; impersonal
  elsewhere. No first person.
- Short sentences. One idea each. Active voice.
- The _why_ beside the _how_: a rule stated without its reason reads as
  arbitrary.
- No promotional words (_powerful_, _seamless_, _revolutionary_), no hedges
  (_probably_, _should work_), no release-relative phrases (_new in_, _since_).
- Unfinished states are described as states, not failures: _open_, _declared_,
  _waiting_ — never "error" unless it is one.

## UI labels and code

- A control or label as it appears on screen: **bold** when it is something to
  click or type in the step (**Add definition**, **+**), and _italic_ when
  quoting a label or message (_Updates in_, _Valid definition_). Match the
  on-screen spelling and capitalisation exactly.
- Names of objects in the example design and formula text: `code` (`dimByTilt`,
  `Tilt / 90 deg`).
- Keys: ⌘S, ⌘↩, ⌃Space, ⇧-drag, ⌫ — macOS symbols.
- Quote a compiler message only when it is the real one; check the source.
  Paraphrase otherwise and say so ("a red line saying …").

## Examples

One product, extended: the tilt lamp of the first tutorial, then ambient light,
an indicator, a second domain, a behavior, a component, a second instance, a
version. Every example must be reproducible with the current build; prefer
values that appear in `examples/smart_lamp` or the tests. Do not invent syntax;
if a form is not sure to elaborate, do not show it.

## Notes, tips, warnings

Use a blockquote for a _tip_ that is optional for the task. Use plain text —
never a box — for facts the reader needs. Do not use warning boxes for states
that are not dangerous.

## Screenshots and diagrams

No screenshot without an entry in [SCREENSHOT_PLAN.md](SCREENSHOT_PLAN.md)
stating the exact design state, crop, caption and alt text. Use ASCII diagrams
for anatomy (workspace, node, instance node, pages); they do not rot. A
screenshot shows _where_; the text says _what it means_.

## Current versus planned

Describe what the build does today. If a page must mention what does not exist,
say so in one sentence — _not built yet_, _has no syntax yet_ — and stop. Never
describe a planned feature as if it worked, and never document a roadmap item in
a tutorial or concept page. `textual/` pages carry a _Current status_ section
because that surface is changing.

## Cross-references

- Every page ends with _Related_ or _Next_.
- Link a concept the first time it matters on a page, not every time.
- _Going deeper_ links to `docs/*.md` and `docs/decisions/*` with a label — _for
  language implementers_, _formal reference_ — so a designer knows they may
  stop.
- Do not repeat another page's explanation; link it.

## Before you commit

- Every claim about behaviour checked against the code or a test, and a row
  added to [VERIFICATION.md](VERIFICATION.md) for a new page.
- Labels checked against the Dart sources (`apps/studio/lib/ui/`).
- Messages checked against the Rust sources.
- Read the page once as the designer (_can I do this?_) and once as the
  implementer (_is every sentence true?_).

## Formatting is mechanical

Run `just docs-fmt` before committing: Prettier lays out the Markdown (`-`
bullets, wrapped prose, aligned tables), bare fences become ` ```text `, and
markdownlint applies its fixes; `just docs-check` fails on what is left. Never
adjust spacing or wrapping by hand — the formatter owns it. Two things it cannot
repair: a `|` inside a table cell must be written `\|`, and a placeholder like
`<name>` goes in backticks.
