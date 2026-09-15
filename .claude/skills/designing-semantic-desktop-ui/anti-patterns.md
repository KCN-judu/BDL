# Anti-patterns

Each with *why it is harmful*, so the reason can be applied to cases not listed. Most of
these are habits from web dashboards and mobile apps; a desktop tool for professional
work has different economics: long sessions, dense information, expert repetition,
spatial memory.

| Anti-pattern | What it looks like | Why it is harmful |
|---|---|---|
| **Card soup** | every group in its own bordered, shadowed, rounded card | borders and shadows are ink that carries no fact; cards fragment the grid, waste space, and make every group equally important so nothing is |
| **Badge soup** | pills for state, kind, count, category on every row | a badge is metadata beside the object; many badges become a legend the eye must decode; the object's own state (dashed, hollow, filled) is lost under labels |
| **Metadata dashboards** | KPI tiles, counters, status panels summarising the document | they duplicate what the objects already show, invite decoration, and pull attention from the work surface; a professional tool is the document, not a report about it |
| **One border around every group** | hairlines/boxes as the only grouping tool | 1 + 1 = 3: each border adds a shape; grouping by proximity and alignment is quieter and stronger; borders should be the last resort, and then hairlines between sections only |
| **Punctuation-based layout** | `r12 · 2 concepts · declared`, `Tilt (rad)`, `— Edited` | forces the reader to parse instead of see; hides the fact that these are columns; demotes with parentheses instead of colour |
| **Arbitrary shadows** | drop shadows on panels, nodes, buttons for "depth" | shadow is an elevation cue; if nothing is elevated it is noise, and it makes flat macOS chrome read as a web page |
| **Decorative gradients** | gradient headers, buttons, backgrounds | spend the colour budget on nothing; break the neutral ground that lets identity hues pop; date the product |
| **Excessive colour** | coloured headers, coloured icons, coloured counts | hue is the identity channel; every other colour steals from it; saturated chrome makes the accent and the error colour invisible |
| **Tiny secondary text** | 9–10 pt for anything the user needs | "secondary" should mean lighter, not unreadable; below 11 pt contrast and legibility fail, and the user leans in |
| **More type sizes than necessary** | 5–7 sizes on one screen | hierarchy becomes ambiguous; the reader cannot rank six sizes; weight and colour would have done it with three |
| **Giant hero regions in productivity views** | a big empty banner, illustration or title above the work | a sovereign application's screen is for work; a hero pushes the work below the fold and repeats itself every day; heroes belong on a launcher, if anywhere |
| **Icon-only actions with unclear meaning** | a row of glyph buttons, no label, tooltip only | recognition fails; the user hovers to learn; a word costs 40 pt and is searchable, accessible and unambiguous |
| **Custom controls replacing standard desktop controls** | a homemade dropdown, toggle, tab bar, dialog | learnability and keyboard access are lost; the platform already solved it; custom is justified only for what the platform lacks (nodes, sockets, lanes) |
| **Compiler terms shown to ordinary designers** | `declRef`, `Grant`, `Κ`, `SingleDriver`, `revision 12`, `protocol 0.1.0`, `id 3`, enum names | the designer's model is product behaviour; kernel words force a translation on every read and signal that the tool is for the implementer, not for them |
| **Status panels duplicating the object** | a list of "undefined mappings" beside a canvas full of dashed nodes | the same fact twice, with no added detail, doubles the ink and halves the trust in each copy; an overview is only justified if it adds navigation or comparison |
| **Red for incomplete-but-valid** | an undefined mapping outlined in red, an open concept with a warning icon | it teaches the designer that the paper's central idea (declare first, define later) is an error; red must be reserved for what is actually wrong, or it stops meaning anything |
| **One colour, many meanings** | orange for "open", "not found", "warning", "pending" | a channel with several meanings cannot be scanned; the reader must read the label anyway, so the colour did nothing but add noise |
| **Arithmetic node per primitive** | `add`, `mul`, `clamp` nodes wired into trees | the canvas stops being a dependency graph and becomes an unreadable expression tree; formulas are properties of a mapping, edited as text in the inspector |
| **Generic dashboard layouts for professional tools** | sidebar of "modules", grid of tiles, top nav | professional tools are canvas + inspector + library with pages by workflow; a dashboard has no work surface and no selection model |
| **Prose that lectures** | the same explanatory paragraph under a control for every object | the intermediate user reads it once and then pays for it forever; specific, dynamic one-liners at the moment of consequence replace it |
| **Silent wrong defaults** | a mapping that "produces" the first concept in the list | a default that is often wrong is worse than no default; the user commits an error without an act of choice |
| **Hover that hides** | hovering a row swaps a fact for a button | information available at rest must stay available; hover may *add*, never remove |
| **Selection that overrides state** | selecting a dashed node draws it solid | the state encoding is more important than the selection encoding; selection must change colour or weight, not the dash |
| **Toasts and snackbars** | transient messages for results | they vanish before they are read, cannot be reviewed, and put the answer far from the object; a state on the object or a line in the inspector persists |
| **Modal confirmation for reversible actions** | "Are you sure?" before a delete that undo covers | excise; trains the user to click through; the named destructive button plus undo is the confirmation |
| **Legend instead of labels** | a colour key in a corner | labels belong on the object; a legend forces a lookup for each read |
| **Auto-layout that rearranges** | a "tidy" button that moves everything | spatial memory is the expert's main asset in a node editor; nothing moves unless the user moves it |
| **Motion as information** | a pulsing badge meaning "live", a shake meaning "refused" | reduce-motion users lose the fact; everyone else is distracted; motion may only echo a state that is also visible at rest |
