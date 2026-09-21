---
id: ADR-0034
status: superseded
date: 2026-09-20
area: studio
supersedes: []
superseded-by: [ADR-0044]
related: [ADR-0001, ADR-0003, ADR-0018, ADR-0019, ADR-0029, ADR-0032]
fv: [BDL/Core/Dependency.lean]
---

# ADR-0034: Canvas edges are signature edges and reference edges; _produces_ is the signature, _carried by_ is a value per tick

## Status

Superseded by ADR-0044 (2026-09-21): right on the same evidence about what an
edge is — dependency read off the analysis's `references`, never a client-side
parse; the signature as what dragging edits — and wrong about what a node is.
The concept node with an input socket for every producer and the produce edge
drew an object the kernel does not have (BDL_FV Phase 21, FVD-0163); the canvas
now draws Sem blocks and mapping blocks, and the read edges end at sockets. Left
as written.

Accepted (the reference-edges milestone, after the Source role of ADR-0032).

## Context

The canvas said "edges show dependency" and drew only the signature: a link from
a concept's output socket into each input socket of a relationship that reads
it, and from a relationship's output socket into the concept it produces. The
kernel's dependency is something else —
`dependsOn Δ a b ⇔ b ∈ (realizationOf Δ a).refs` (`docs/spec/kernel.md` §5):
which relationships a formula _names_. Nothing drew it. On one small design —
the Sources `TempSensor : () -> RoomTemp` and `ButtonInput : () -> ButtonHeld`,
the rule `AirConditionerCtrl : RoomTemp -> ButtonHeld -> SwitchState`, then the
value `acOn : () -> SwitchState = AirConditionerCtrl(TempSensor, ButtonInput)` —
four things went wrong at once:

1. Before `acOn` existed the rule looked wired and complete although nothing
   applied it: a rule is a function, and the design computes nothing until a
   value's formula applies it.
2. After `acOn` existed it was an isolated node with one edge to _SwitchState_,
   although it depends on the rule and both Sources. The design's real structure
   was invisible.
3. _Produces_ meant two things. The canvas drew every relationship whose output
   is _C_ into _C_'s input socket ("something produces this concept"); the
   Simulate probe listed only unit-domain relationships and said _No value
   declaration produces C_. A rule and a value were drawn alike, so the two
   readings contradicted each other on the same selection.
4. The creation sheet said _Dashed: declared, not yet defined…_ for a
   relationship with reads and _The environment provides it_ for one without —
   never that the first is a rule with no value of its own, nor that a `() -> C`
   with a formula is a computed value and not a Source.

The bounds: Studio computes no semantic judgment (ADR-0001), so what a formula
references must come from the compiler; signature edges are what the designer
edits by dragging (ADR-0032 §3, the Source has no input socket); the canvas
already draws the dependency graph's cut for a collapsed group — a group box's
aggregate input sockets and the crossing-in edges into them are `dependsOn` read
off the flat graph (ADR-0019) — so the expanded picture was the odd one out; one
channel carries one meaning (`docs/architecture/studio-ui.md` §7).

## Decision

1. **Two kinds of edge, told apart by where they end and how they are drawn,
   never by hue alone.** A _signature edge_ runs socket to socket in the
   concept's hue, 2 px: concept → a relationship that reads it, relationship →
   the concept it produces, value → the sink it drives. It is the interface
   (`Signature`), and dragging one edits the signature. A _reference edge_ runs
   from a relationship's output socket into the **formula line** of a
   relationship whose definition names it — the left end of the definition
   region, not a socket — in the neutral secondary text colour, 1.5 px, under
   the signature edges. It is `dependsOn`: the analysis's `references` for that
   relationship (the kernel's `refs`), never derived by Studio from the formula
   text. It cannot be dragged, dropped on, hovered or selected; the formula is
   edited in the inspector. The name of every relationship a node depends on is
   said to assistive technology with the node.
2. **The projection of dependency is the analysis, not the design.**
   `MappingAnalysis.references` (protocol 0.18, additive) carries
   `DependencyGraph::all` for the relationship: every declaration its elaborated
   realization references that exists in the design, ascending, each once,
   itself included when the formula names it. It is empty for a declared
   relationship, a Source, a definition that does not elaborate, and in a draft
   verdict (a draft changes nothing on the canvas). A self-reference draws no
   edge — memory through `delay` is the register mark of `studio-ui.md` §7,
   still _spec_. Between a commit and its analysis the canvas keeps the previous
   analysis's edges and drops any whose end no longer exists; a reference to a
   relationship the projection does not show (an instance's private relationship
   in the flat analysis) draws nothing. A hidden member of a collapsed group
   adds no reference edges of its own — the group's crossing-in edges stand for
   them — and an edge from a hidden member leaves from the group's aggregate
   output socket, as its signature edges do.
3. **The three shapes are distinguishable on the canvas without the formula.** A
   **rule** reads something: its input sockets are the shape, and the header
   carries the word _rule_ when no state word takes the slot (the precedence is
   _Source_ › _declared_ › port word › sink state › _required_ › _rule_). A
   **value** reads nothing and has a formula: no input socket, no header word. A
   **Source** reads nothing and has no formula: the marks of ADR-0032. The
   inspector's _Role_ row says _Source_ / _Rule_ / _Value_ (a port-backed
   relationship keeps its port word) with one sentence each; a defined
   relationship gets _Depends on_ (its `references`) and every relationship but
   a Source gets _Named in_ (the same edges read the other way) as rows of name
   links — the detail behind the neutral edge.
4. **One definition of _produces_.** _Produces_ is the signature: a relationship
   whose output is _C_ produces _C_, rule or value, and _C_'s input socket, the
   inspector's _Produced by_ row and the creation sheet's _Produces_ pop-up all
   mean exactly that. A value of _C_ per tick exists only where a value or a
   Source produces it: _C_ is **carried by** those. The Simulate probe for a
   concept is titled by that word — _Carried by_ over the values and Sources
   with their latest sample; with none, _Nothing carries C yet: no value or
   Source produces it_, and for each rule that produces _C_, _R is a rule; a
   value whose formula applies it would carry C_. The probe for a rule says _A
   rule: it has no value of its own. A value whose formula applies it is what
   the simulator samples_, then _Applied in …_ (the relationships whose
   `references` name it) or _No value applies it yet_.
5. **The creation sheet names the shape as the reads change**, in one sentence
   with its consequence. Nothing read: _Reads nothing: a Source. The environment
   provides C once per activation; a formula added later makes it a computed
   value instead._ Something read: _Reads X: a rule, a function to C. It has no
   value of its own — a value's formula applies it; dashed until its formula is
   added._ The sheet cannot create the third shape (a formula is attached
   later), and says how it arises.

## Alternatives

- _Keep signature edges only and stop claiming dependency_: rejected — problems
  1 and 2 are exactly the designer's question ("is this rule applied by
  anything?", "what does this value need?"), and the collapsed-group picture
  already draws the answer for members; leaving the expanded canvas without it
  would keep two contradictory pictures of one graph.
- _Replace signature edges by reference edges_: rejected — the signature is what
  dragging edits (ADR-0032 §3) and the interface others depend on; a declared
  rule has no formula and would lose every edge.
- _A dedicated "applies" socket row on the referencing node_: rejected — a
  socket is a drop target that edits the signature; a reference lives in the
  formula and is edited there, so a socket that cannot be dragged would lie
  about the gesture.
- _Dashed reference edges_: rejected — dashed already means _declared, not
  defined_ (one channel, one meaning).
- _Reference edges in the referenced concept's hue_: rejected — hue is the
  identity of a value at a socket; a reference to a rule is a reference to a
  function of no single concept, and a hued edge into a formula line would read
  as a second, undraggable port.
- _Deriving `refs` in Studio from the formula text_: rejected by ADR-0001; the
  compiler's `DependencyGraph` is the judgment, on the elaborated term, with
  dangling names filtered.
- _Distinguishing a rule by colour_: rejected — the header tint is the node
  category and hue is identity; the sockets are the shape and the word is the
  accessible cue, as for _Source_.

## Consequences

- The canvas is a picture of both the interface and the dependency; on the
  air-conditioner design `acOn` is joined to `AirConditionerCtrl`, `TempSensor`
  and `ButtonInput` by neutral edges into its formula line, the rule wears
  _rule_, and selecting _SwitchState_ on the canvas or in Simulate gives
  statements that agree.
- `docs/architecture/studio-ui.md` §2 and §7 carry the two edge kinds and the
  _rule_ word; `docs/spec/protocol.md` carries `MappingAnalysis.references` and
  0.18; the change fragment is
  `docs/changes/unreleased/2026-09-reference-edges.md`; the user guide's
  _Canvas_, _Relationships_, _Simulate_ and _Inspector_ pages follow. The
  glossary words are _rule_, _value_, _carried by_; _produces_ keeps its one
  meaning.
- Not drawn yet, deliberately: a self-reference (memory) and the delayed /
  instantaneous distinction (`instRefs`) — the register mark of §7; a reference
  edge across a lane once domains are drawn. Reference edges are not hit-tested,
  so a designer discovers what one means from the referencing node's _Depends
  on_ row, not from the edge.
