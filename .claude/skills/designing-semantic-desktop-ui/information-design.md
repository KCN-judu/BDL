# Information design

Sources: Edward Tufte, *The Visual Display of Quantitative Information* and *Envisioning
Information*; Colin Ware on channels (see `gestalt.md`). Own words.

## Data-ink and its enemies

- **Data-ink ratio.** Of all the ink on the screen, how much carries information? Erase
  non-data ink (decorative shadows, gradients, frames, icons that repeat a word). Then
  erase *redundant* data-ink (the same fact shown three ways in one place). What is left
  is the design.
- **Chartjunk in a UI** looks like: badges restating a state the outline already shows,
  a "status panel" listing what the canvas already displays, a card border around every
  section, an icon plus a label plus a colour for one category.
- **1 + 1 = 3.** Two adjacent marks create a third visual object between them (the gap,
  the moiré, the border-on-border). Every separator you add adds a shape to parse.
  Hairlines between sections only; no double borders (a box in a box).
- **Smallest effective difference.** Make distinctions as subtle as they can be while
  still being seen: a hairline at 12 %, a tint at 6 %, secondary text instead of a
  different size. Loud differences are spent on the few facts that deserve them.

## Density is a virtue when it is organised

- A dense inspector or table that the eye can scan beats a sparse one that needs
  scrolling. Density comes from small consistent type, tabular alignment, and removing
  decoration — not from cramming.
- **Micro/macro readings**: the canvas should read at two scales — zoomed out, the
  topology (what connects to what, which regions exist, what is dashed) and zoomed in,
  the detail (names, values, marks). Design both; test both zoom levels.
- **Small multiples**: when the same object appears in several states (simulation ticks,
  the same requirement on several boards), repeat one structure with the same encodings
  rather than inventing a new view per case.

## Layering and separation

- Rank facts into layers and give each a lighter treatment: primary in ink, secondary
  in 50 %, tertiary in 25 % or on hover. A reader should be able to attend to one layer
  and ignore the rest.
- Separate by *kind of ink*: structure in lines and fills, labels in text, values in
  tabular numbers, status in a word. Mixing kinds in one element (a coloured, bordered
  badge with an icon and a word) collapses the layers.
- Colour to *label* (identity), to *measure* (never in this product — values are
  numbers and positions), to *represent reality* (a board picture), to *enliven* (only
  the hero). Do not spend colour on decoration in a working view.

## Comparison and context

- A number alone is weak; a number beside what it is compared with is a fact: "7 PWM
  needed · 6 available", "0.62 now · 0.58 previous tick", "2 of 5 outputs driven".
- Tables of facts: fixed columns, comparable rows, the comparison down a column (values
  align so differences are visible), units in their own column.
- Diagnostics are comparisons too: what was required versus what is present, named on
  both sides.

## Text and graphic together

- Put the label where the thing is: the socket label beside the socket, the initial
  value at the delay mark, the pin name on the pin. A legend is a failure of placement.
- Integrate explanation with structure: the explanation layer opens *from* the object
  (a diagnostic anchored under the formula; a datatip on the gate), not from a separate
  page.

## Checks

- What could be erased without losing a fact? Erase it.
- Is any fact shown more than once in the same view for no overview↔detail reason?
- Are values numbers in aligned columns, not colour intensities?
- Does the canvas read at both zoom extremes?
- Is there a legend? Why is the label not on the object?
