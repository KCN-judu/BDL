# Layout and grid

Sources: Josef Müller-Brockmann, *Grid Systems in Graphic Design*; Adam Wathan & Steve
Schoger, *Refactoring UI* (spacing, borders); Apple HIG layout guidance; the numbers
already fixed in `docs/STUDIO_UI.md` §3 and §3a. Summarised in my own words.

## What a grid is for

Müller-Brockmann's grid is an *ordering system*: a page divided into a fixed set of
fields (columns × rows) separated by gutters, with margins that are part of the design.
Its purpose is not decoration but predictability — the reader learns where things are
and stops searching. Three consequences for a desktop tool:

- **Every edge is shared.** Text, controls, headers and values sit on a small number of
  vertical lines. A new element does not get its own x; it joins an existing one.
- **The module repeats.** Rows are a fixed height, columns a fixed width; the same
  structure repeats down a list or across a table so the eye can predict the next item.
- **Whitespace is structural, not leftover.** The margin and the gutter are set on
  purpose and kept identical; they are what makes the grid perceptible without lines.

The grid is felt, not seen: hairlines and boxes that reveal it are a failure to trust it.

## The Studio grid (fixed — reuse, do not reinvent)

| Element | Value |
|---|---|
| base unit | 8 pt; 4 pt only for optical correction inside an item |
| sidebar / inspector | 220 pt / 290 pt, resizable, hideable |
| list row | 24 pt; control height 22 pt; text field 24 pt |
| panel header | 28 pt, 12 pt inset |
| form label column | 78 pt, right-aligned, baseline-aligned with the field |
| gutter (between columns) | 16 pt |
| gap inside an item | 4–8 pt |
| gap between items of a group | 16 pt |
| gap between groups / sections | 24 pt |
| corner radius | 5–7 pt controls and nodes, 10 pt sheets |
| separators | whitespace; a hairline only between sections |

`MacMetrics` holds these; `FormRow`, `MacTable`, `InspectorSection`, `PanelHeader` apply
them. A new component that needs a different number must argue for it in the doc.

## Spacing rules (Refactoring UI, adapted)

- **Establish the scale, then pick from it.** 4 · 8 · 16 · 24 · 32 · 48. Never 13 or 18.
- **Start with too much space.** Add whitespace generously first and remove until the
  groups read; dense-first layouts never get their groups back.
- **Ambiguous spacing is the most common bug.** A label must be visibly closer to its
  own field than to the field above; a section title closer to its content than to the
  previous section's last row. Check the ratios, not the absolute values.
- **Relative units do not scale.** A 3× gap between sections is right at one size and
  cavernous at another; use the scale's absolute steps.
- **Do not fill the width.** A sheet at 480 pt, a form column that stops where the
  longest sensible value ends, a table whose columns are as wide as their content.
  Stretching to fill is what makes desktop tools look like web pages.
- **Group with space, then with background, then — last — with a border.** A tinted
  region (common region) is stronger and quieter than a stroked box. If a border is
  needed, it is a hairline at ~12 % ink, never a 1 px solid line in a saturated colour.

## Columns and tables

Anything with two or more rows sharing fields is a table, even in an inspector:

- fixed column widths, one gutter, no vertical rules;
- text columns left-aligned, numbers right-aligned in tabular figures, units in their
  own column in secondary colour;
- the leading column is the one a glance needs (the name); state and counts trail;
- row height constant; a row that needs two lines is a sign the columns are wrong.

## Panels and regions

- **Sidebar–content–inspector** is a three-column grid with the content column flexible.
  The sidebar and inspector keep their widths across pages so spatial memory survives.
- **The canvas is its own grid**: nodes snap to a coarse module (24 pt in the painter)
  for alignment, but auto-placement respects reading order (sources left, sinks right).
- **The status line and page bar** are one-row grids of cells with `gapGroup` between
  cells; the leading cell is the fact a glance needs.
- **Sheets** are a form: a label column, a field column, a preview region, an action
  row with Cancel left of the default. Nothing centred except the sheet itself.

## Checks

- Can you name the x-coordinate of every left edge on the screen with ≤ 4 numbers?
- Are inside-item gaps visibly smaller than between-item gaps, which are visibly smaller
  than between-group gaps?
- Is any border doing work that 8 more points of space would do better?
- Does any element stretch to the container width without a reason?
