# Good semantic UI — worked examples

Each example names the fact, the encoding, the secondary cue, and why it beats the
metadata alternative. Some exist in Studio today; some are the agreed target.

## 1. Identity as hue, stable across renames (exists)

`Tilt` gets hue 0°, `Brightness` gets 137.5°, from their ids. Every socket, link, library
dot and chip for `Tilt` is that hue on every page. Rename `Tilt` to `Lean` and nothing
changes colour. The secondary cue is the name beside every socket.

*Why it is good:* the designer learns "the red one is Tilt" once and then reads the graph
preattentively; the nominal-typing rule (only same-hue sockets connect) becomes a
perceptual fact. *Metadata alternative:* a "type: Tilt" label on each socket — read, not
seen; and no stability guarantee.

## 2. Incomplete as dashed, not red (exists)

An undefined mapping has a dashed hairline outline and the word *declared* in the
header; its sockets are fully coloured and linkable. It is the same size and position as
a defined one.

*Why:* the paper's core idea — declare first, define later — is shown as a normal,
usable state. The dashed line (closure) still reads as a node; nothing suggests failure.
*Metadata alternative:* a red "undefined" badge — teaches the designer that their
workflow is an error.

## 3. The typing rule as a refused drop (exists in part; feedback to add)

Dragging from a `Tilt` output socket, only `Tilt` input sockets are legal targets. The
target halo appears only over a legal socket; release elsewhere discards. To add: all
compatible sockets gain a faint halo while dragging; an incompatible socket under the
pointer shows the blocked cursor; release on it retracts the link in 120 ms.

*Why:* the rule is never stated; it is felt at the pointer. Nothing to read, nothing to
undo. *Metadata alternative:* allow the link, then show "type mismatch: Tilt ≠
MotorAngle" — a diagnostic for a mistake the tool could have prevented.

## 4. The sheet that shows the node it will create (exists)

The New concept sheet renders, live, the concept node with a hollow socket while "Decide
later" is chosen and a filled one when a kind is chosen, with a caption stating what the
socket means. The New mapping sheet renders the dashed node with one socket per chosen
input, in their hues.

*Why:* responsive disclosure and preview together; the form teaches the encoding the
designer will meet on the canvas, with no sample text ("e.g. Tilt") standing in for
meaning. *Alternative:* a form with hints and a paragraph — the designer meets the
encoding for the first time after committing.

## 5. Memory as a mark on the link (target)

`Held` reads its own previous value. The link from `Held`'s output back to its input
carries a small register mark (two short bars across the curve) with the initial value
`false` beside it. Selecting the mapping shows "Remembers: Held, starting at false" in
the inspector.

*Why:* the temporal fact is on the dependency it belongs to; the cycle is visible as a
loop through time, not hidden or flagged; the initial value — which the kernel demands —
is where the eye already is. *Alternative:* a `delay` node in the graph (arithmetic-node
soup) or a "temporal: yes" row in the inspector.

## 6. Domains as lanes, crossings as gates (target)

Two labelled background lanes, *interaction 50 Hz* and *ambient 1 Hz*. Mappings sit in
one lane or outside both (domain-free). The link from `Temperature` (ambient) to
`heaterTarget` (interaction) passes through a small gate at the lane edge showing
`20 °C`, the initial value. A crossing without a gate is drawn broken at the boundary,
with an error-red gap and the fix in the datatip.

*Why:* containment says "belongs to"; the gate says "observed through a transport";
the strictly-before semantics get a place to be explained from. *Alternative:* a
"clock: ambient" row per node and a diagnostic list.

## 7. Ownership at the point of convergence (target)

The light output node has one input socket. `dimByTilt` drives it (filled socket, link).
Dragging `pulseWarm`'s output toward it: the socket does not highlight, the cursor shows
blocked, and a datatip says "driven by dimByTilt — combine the two brightness values
before connecting". The link retracts on release.

*Why:* single-driver is a constraint the designer meets at the exact place the conflict
would occur, with the fix. *Alternative:* allow two links and list a "SingleDriver
violation" somewhere else.

## 8. Feasibility as a board with leads (target)

The Deploy page shows the Nano with its pins. Each requirement (M1 PWM, IMU I²C) has a
lead to its assigned pin. The seventh PWM has no lead; beside it: "needs PWM · 6 PWM
pins on this board, all taken: D3 M1, D5 M2, …". Switching the board redraws leads;
the design canvas is untouched.

*Why:* feasibility is target-relative, so it is shown against the target; the blockers
are named on the thing that is blocked; the separation from typing is spatial (another
page). *Alternative:* a "hardware-feasible: false" pill on the project.

## 9. Values as numbers at the socket (target)

In Simulate at tick 12, `Tilt`'s socket shows `31.4` with `°` in the unit column;
`Held`'s ◇ is filled (true); `Brightness` shows `0.62`. Scrubbing the timeline changes
the numbers; nothing animates. In Monitor the same, and a value 2 s old has faded to
secondary with "2 s ago" on hover.

*Why:* the design canvas becomes the instrument; position and number are the accurate
channels; staleness is opacity with a text partner. *Alternative:* a separate table of
declId → value.

## 10. Disabled with the reason at rest (target)

"Delete Tilt" is disabled and under it, in secondary 11 pt: "used by dimByTilt". The
mapping name is a link that selects it.

*Why:* the gulf of evaluation is closed without a hover; the fix is one click away.
*Alternative:* disabled with a tooltip (exists today) — the reason is invisible until
the user guesses to hover.
