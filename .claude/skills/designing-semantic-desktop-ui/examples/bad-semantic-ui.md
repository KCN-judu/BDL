# Bad semantic UI — what to recognise and refuse

Each example shows the habit, the harm, and the repair. Several are hypothetical
"obvious" designs a framework-trained hand would produce for BDL; a few are present in
Studio today and are marked.

## 1. The status card grid

A Design page header with four tiles: *Concepts 4 · Mappings 2 · Undefined 1 · Errors 0*.

*Harm:* metadata dashboard; duplicates the canvas (dashed nodes are the undefined
count); "Errors 0" is a KPI, not a state; the tiles take the top 80 pt of the work
surface forever. *Repair:* the status line's count cells (exist), each clickable to
select/frame the objects it counts; nothing above the canvas.

## 2. Badge soup on the node

A mapping node with pills: `declared` (orange), `2 inputs` (grey), `formula` (blue),
`Δ` (purple), `50 Hz` (teal).

*Harm:* five badges for facts that are already the node's shape (dashed outline, two
sockets, body line, the lane it sits in); colours stolen from the identity channel;
nothing pops out because everything does. *Repair:* dashed + *declared* word only; the
rest is structure or inspector.

## 3. Red means "not done yet" (avoid — Studio correctly avoids it)

Undefined mappings outlined in red with a ⚠ icon; open concepts with a red dot.

*Harm:* teaches that the signature-first workflow is an error; when a real type error
appears, red has already lost its meaning. *Repair:* dashed/hollow + *open/declared*;
red only for a contradiction.

## 4. Identity shown as a number (present: `identity 3` in the inspector section header)

*Harm:* the id is an implementation fact; the designer cannot use it; it invites
"what does 3 mean?" and makes the panel look like a debugger. *Repair:* remove; the hue
and the name are the identity. Expose ids in a technical-details disclosure if ever
needed.

## 5. Two vocabularies for one fact (present: sheet says *Quantity / On–off / Count /
Decide later*; inspector segmented says *none / quantity / boolean / count*)

*Harm:* the designer learns the words in the sheet and then meets different, more
technical ones in the inspector; "none" reads as "no type" rather than "not decided";
"boolean" is a programmer's word. *Repair:* one set of words everywhere, from the
product side.

## 6. Selection erases state (present: a selected undefined node is drawn with a solid
accent outline)

*Harm:* the most important state cue disappears exactly when the designer is looking at
the object. *Repair:* dashed in accent when selected.

## 7. The lecture under every control (present: static paragraphs "Changing the
signature is an edit…", "No definition yet. This is a legal state…", "Replacing or
detaching is an edit.")

*Harm:* same text for every object, read once, paid for forever; pushes the controls
apart; sounds like documentation. *Repair:* one dynamic line at the moment of
consequence: "Re-checks dimByTilt, lampTarget" appearing when the field is focused or
after the commit (the Last change note already does the latter).

## 8. A node per operator

`Tilt → [÷ 60] → [× 0.8] → [+ 0.2] → [clamp 0 1] → Brightness`.

*Harm:* the canvas becomes an expression tree; dependency between concepts — the
thing BDL cares about — is buried; every formula edit is a graph surgery. *Repair:* one
mapping node; the formula in the inspector; the body line shows a summary.

## 9. Compiler words in the status line (present: `r12`, `bdld 0.1.0`, `protocol 0.1.0`)

*Harm:* the designer sees the daemon's name and a revision counter with no persisted
meaning; the leading cell — the one a glance is for — is the least useful. *Repair:*
lead with the document state ("Saved" / "Edited", then the open-decision count),
"Compiler 0.1.0" only, and the protocol only when it mismatches.

## 10. Hover swaps a fact for a button (present: Recent rows replace the "2 h ago" column
with × on hover)

*Harm:* information at rest disappears under the pointer; the column jumps. *Repair:*
keep the time; put × in its own trailing column that appears on hover, or in the
contextual menu.

## 11. A silent wrong default (present: New mapping "Produces" defaults to the first
concept)

*Harm:* a mapping reading `Tilt` and producing `Tilt` is created without a choice; the
canvas then shows a plausible-looking self-loop. *Repair:* no default — the pop-up
reads "choose" and Create stays disabled until it is chosen; or default to the most
recently created concept that is not among the inputs, and say so in the preview.

## 12. Custom look-alikes that cannot take focus (present: `MacSegmented`,
`_ConceptToggle` use `GestureDetector` only)

*Harm:* keyboard users cannot choose a kind or an input; the focus ring standard is
violated silently. *Repair:* `FocusableActionDetector` with arrow-key movement and
Space/Return activation, the 2 pt ring.

## 13. Category hue that competes with identity

Concept headers in saturated teal, mapping headers in saturated blue, context headers
in purple.

*Harm:* the saturated header is the biggest coloured area on the node, so the eye reads
category before identity; sockets get lost. *Repair:* header tints at low saturation
(as today); identity hues the only saturated marks.

## 14. Legend in the corner

A key: "● quantity ◆ on-off ■ count ○ undecided".

*Harm:* every read requires a lookup; the sheet already teaches the glyphs at creation
and the inspector states the kind in words. *Repair:* no legend; the word in the
inspector is the secondary cue; the sheet's caption is the lesson.
