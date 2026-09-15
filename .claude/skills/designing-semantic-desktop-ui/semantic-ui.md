# Semantic UI: embodying meaning in structure

BDL's unusual requirement: the interface must *be* the semantics, not describe them.
This file holds the rules for choosing an encoding and the channel table every encoding
must respect. The BDL-specific assignment of facts to encodings is in `bdl-contract.md`.

## The question to ask for every semantic fact

In this order, and stop at the first that works:

1. **Can shape express it?** Shape is read before text and survives colour-blindness.
   (Socket shape for representation; node silhouette for output vs concept.)
2. **Can colour express it?** Only if the channel is free (hue = identity; nothing else
   gets hue) and a non-colour partner exists.
3. **Can connection express it?** A link is the strongest grouping cue there is; a
   missing link, a refused link, a link with a mark, a link through a gate all say
   something without a word.
4. **Can spatial grouping express it?** Left/right for source/sink; proximity for
   membership; columns for stages.
5. **Can containment express it?** A region says "these belong to this" — domain, context.
6. **Can an interaction constraint express it?** If it cannot be done, the gesture
   refuses. This is the purest embodiment: the rule is never stated, only felt.
7. **Can the object's own state express it?** Dashed, hollow, faded, filled — the object
   looks like what it is.
8. **Only then**: a word on the object, then a line in the inspector, then a badge, then
   prose in the explanation layer.

"Semantics as visual structure" beats "semantics as metadata displayed beside the
structure" because the designer perceives the former while looking at the work, and
must read the latter after looking away from it.

## Distinguish three layers and keep them apart

| Layer | Holds | Rule |
|---|---|---|
| **Structure** (canvas) | identity, relationships, regions, incompleteness, ownership, memory, crossings, values | perceivable without selection; no compiler words |
| **Properties** (inspector) | everything editable about the selection; the state ladder; consequences of the next edit | product language; one control per `EditOp` |
| **Explanation** (on demand) | why, diagnostics in full, kernel vocabulary, technical details | opens from the object; never required to operate |

A fact may appear in structure *and* properties when properties add detail (the ladder
behind the dashed outline). It may not appear as a badge on the structure *and* a word
in the properties *and* a status-line count with nothing added at each step.

## Channel table — one channel, one meaning

| Channel | Meaning in Studio | Secondary cue required | Forbidden uses |
|---|---|---|---|
| **hue** (saturated) | semantic identity of a concept | the concept's name beside the socket / in the row | category, status, decoration, user colour |
| **hue** (low-saturation header tint) | node category (concept / mapping / context / output / transport) | the node's silhouette and title | anything else |
| **socket shape** | representation category: ○ quantity ◇ on-off □ count | the kind word in the inspector | status, direction |
| **fill vs hollow** | bound vs not-yet-chosen (concept), driven vs undriven (output socket) | *open* / *undriven* word | selection, hover |
| **dashed boundary** | declared, not defined (incomplete-but-valid) | *declared* state word | error, disabled, hover, drag ghost |
| **solid boundary** | defined | — | — |
| **accent** | selection, keyboard focus, the default button | outline weight (2 px) | status, links, highlights, brand |
| **error red** | a fact that is wrong now (type mismatch, loop, second driver, no pin, connection failed) | an icon + a sentence with the fix | incomplete, pending, warning, delete buttons' fill |
| **open orange** (text/tint only) | a decision still to be made (open representation, undefined mapping, unpinned requirement) | the word itself | fills, links, errors |
| **settled green** (text/tint only) | reached, checked, connected | the word itself | fills, "success" toasts |
| **register mark** on a link | the value crosses a tick (delay / hold / previous) | initial value text at the mark; inspector row | decoration |
| **gate** on a link at a lane edge | transport (`sync`) with initial value | the value text; inspector row | — |
| **region tint** (large, labelled) | clock-domain lane or context membership | the label; the domain/context row in the inspector | grouping for tidiness, selection |
| **position x** | data flow / stage: sources left, sinks right | link direction by construction | priority, order of evaluation |
| **opacity** | age / staleness of a value; disabled | age text on hover; disabled reason at rest | hierarchy of static text (use colour tokens) |
| **weight (type)** | what to look for (names, titles) | — | status |
| **motion** | a transient answer (retract, snap, 120 ms) | the resulting state is visible at rest | carrying a fact; attention grabbing; loading data |
| **identity-hue fill on a chip / toggle** | *membership*: this concept is in the signature being edited (sheet toggles, Reads chips) — not selection | the socket glyph inside the chip; the preview node's sockets | selection (which is accent), status |

Adding an encoding means adding a row here and in `bdl-contract.md` §2 in the same
change. Reusing a channel for a second meaning is refused at self-review.

## Rules

1. **Do not make every state a badge.** A badge is metadata beside structure. Use the
   object's state (dashed, hollow, faded), then a state word in the header. Pills are
   for the inspector's state ladder and the last-change note, not the canvas.
2. **Incomplete is not wrong.** The paper's whole point is that a declared-but-undefined
   relationship is a valid design object. Dashed and *open*, never red, never faded as
   if disabled, never hidden.
3. **Show the rule by refusing, not by reporting.** Identity typing, single driver,
   causality and clock crossings are constraints in the gesture first, diagnostics second.
4. **Ownership is shown at the point of convergence.** Two things wanting one output is
   visible where the links meet the output socket, not in a list somewhere else.
5. **Time is a mark on the edge, not a kind of node.** Memory and transport are
   properties of a dependency; give them a glyph on the link and a row in the inspector,
   never a node per `delay`.
6. **Domains and contexts are places.** Containment, labelled, tinted. A node's domain
   is where it sits, and moving it is an edit the compiler must accept.
7. **Identity is compiler-assigned and immutable in the UI.** No colour picker for
   concepts; the hue is derived from the id and therefore stable across renames, pages
   and sessions.
8. **Numbers are numbers.** Simulation and telemetry values are figures with units at the
   object, positions on a plot, or fill/empty for booleans — never colour intensity.
9. **The same fact looks the same everywhere.** Socket glyph in library row, chip,
   pop-up item, sheet toggle and canvas are one drawing (`NodePainter` or a widget that
   mirrors it exactly).
10. **Words for the same fact are the same words** in sheet, inspector, library, status
    line and diagnostic (see `bdl-contract.md` §4).

## Testing an encoding

- Cover the labels: is the fact still perceivable?
- Set the display to greyscale: is the fact still perceivable (shape, dash, fill, mark,
  region)?
- Zoom out to 0.25×: is the fact still perceivable?
- Show it to the channel table: is the channel free? Is the secondary cue present?
- Ask: is this a *state of the object* or *metadata about the object*? If metadata,
  can it become state?
