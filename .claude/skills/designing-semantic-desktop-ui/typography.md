# Typography

Sources: Ellen Lupton, *Thinking with Type*; Apple HIG macOS text styles; Refactoring UI
on hierarchy. Studio's fixed sizes are in `docs/STUDIO_UI.md` §3 and `mac/theme.dart`.

## Hierarchy with few levels

Lupton's point: hierarchy is *contrast that the reader can rank at a glance*, and it is
made with several tools — size, weight, colour, case, position, spacing — not size alone.
Two or three levels are enough for almost any screen; more levels mean the reader has to
work out which of six sizes is which.

Studio's scale (the HIG macOS table, trimmed):

| Role | Size | Weight | Colour | Used for |
|---|---|---|---|---|
| body | 13 | regular | primary (85 %) | fields, rows, node titles (12.5 in the painter), buttons |
| headline | 13 | semibold | primary | window/sheet titles, node titles |
| subheadline | 11 | regular / semibold | secondary (50 %) | section titles, form labels, captions under previews, socket labels |
| caption | 10 | medium | secondary / status | state words on nodes, page bar labels |
| monospace | 13 Menlo | regular | primary | formulas only |

Rules:

- **Do not add a size.** If a fourth size seems needed, change weight or colour instead.
  Titles bigger than 13 pt exist only on the welcome screen (the hero) and nowhere in a
  working page.
- **Secondary means lighter, not tinier.** Demote with the secondary colour at the same
  size before dropping to 11; never below 10; never below 11 for anything the designer
  must read to act.
- **Weight ranks; colour recedes.** Semibold is for the thing you look for (a name, a
  title). Secondary colour is for what you read after you found it (a label, a unit).
  Bold + coloured + larger on the same element is shouting.
- **Case:** sentence case everywhere; no uppercase section headers (Finder's sidebar
  headers are semibold secondary, not caps).

## Labels and values

- A form is two columns: labels right-aligned in the 78 pt column, values left-aligned
  next to them, baseline-aligned. The label is secondary colour at 11; the value is
  primary at 13. The eye reads the value column; the label column is the index.
- A label that merely repeats the value's obvious meaning can go ("Name" above a name
  field in a sheet titled "New concept" is borderline; keep it because the platform
  form idiom expects it).
- Units and symbols live in their own column in secondary colour, never appended with a
  separator or wrapped in parentheses.
- Numbers that sit in columns use tabular figures (`kTabularFigures`) so digits align.

## Line length and density

- Body text in inspectors and captions: 45–60 characters per line. The 290 pt inspector
  at 11–13 pt gives that naturally; do not widen text blocks beyond it.
- Do not justify; left-aligned ragged right.
- Explanatory prose in an inspector is two lines at most, and only when it says
  something *specific to this object now* ("used by 2 mappings; changing re-checks
  them"). Static paragraphs that are the same for every object are documentation, not UI.

## Readability of names

Concept and mapping names are the most-read text in the product. Give them:

- the primary colour and semibold where they are the subject (node header, library row);
- a colour dot or socket glyph beside them so identity is recognisable before reading;
- ellipsis truncation with the full name in a tooltip, never wrapping in a row.

## Checks

- Count the distinct font sizes on the screen. More than four (13, 12.5, 11, 10) is wrong.
- Is anything important shown at 10 pt tertiary? Then it is not shown.
- Is hierarchy visible if you squint? Weight and colour should still rank the elements.
- Are labels and values on one baseline? Are numbers right-aligned in tabular figures?
