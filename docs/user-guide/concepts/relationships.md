# Relationships

A **relationship** is what you create with **+** by _Mappings_: a name, a
**signature** — the concepts it _reads_ and the one it _produces_ — and, once
you write it, a **formula**. It is one of two things. A relationship that
**reads nothing** is a **block** (a _Sem block_): one value of its concept in
the design, one per tick — `tilt`, `brightness`. A relationship that **reads
something** is a **rule**: a template from concepts to a concept — `dimByTilt`
reads _Tilt_ and produces _Brightness_; `adaptBrightness` reads _Brightness_ and
_Ambient light_ and produces _Brightness_. A rule has no value of its own; a
block's formula applies it, and that application is what the canvas draws as the
block's **mapping block**. Rule : mapping block = template : application.

In the sidebar and the creation sheet, relationships are labelled _Mappings_.
Same thing.

## Signature first, formula second

Creating a relationship only fixes its signature. That is enough for the rest of
the design to be built around it: other relationships can apply it, an output
can wait for it, the simulator can list what it still needs. The formula comes
when you are ready, in the inspector's _Relationship_ section. Until then a
relationship that reads something — a rule — is _declared_ (the inspector and
the status line say so); one that reads nothing — a block on the canvas — is a
**Source** — complete as it is, provided by the environment.

The formula is checked against the signature as you type, and the check is in
your own terms: _Brightness is a dimensionless quantity, but this formula
produces an angle._

## Three shapes of relationship

| Shape              | Reads                | Formula | What it is                                                                                                                                                  |
| ------------------ | -------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rule**           | one or more concepts | yes     | a template: a function from concepts to a concept, `dimByTilt : Tilt → Brightness = Tilt / 90 deg`; applied by blocks' formulas, never a node on the canvas |
| **Source**         | nothing              | none    | a block the environment provides — from a sensor, a switch, an analog line. In simulation you type it; on a device the hardware supplies it                 |
| **Computed value** | nothing              | yes     | a block with a formula, drawn with its mapping block: `brightness = dimByTilt(tilt)`                                                                        |

Only a relationship that _reads nothing_ — a block — has a value; a rule is
something you apply. This matters in three places: a physical output can only be
driven by a block; a formula applies a rule by calling it — `dimByTilt(tilt)` —
and names a block by writing it — `tilt`; and only a block gives its concept a
value at each tick. A rule _produces_ its concept in the sense of its signature,
but a value of _Brightness_ exists only in a block whose formula applies the
rule. Until `brightness` exists, `dimByTilt` is a function nobody calls, and no
Brightness is anywhere in the design.

The creation sheet says which shape you are about to make as you change the
reads: with nothing read, _Reads nothing: a Source…_; with something read,
_Reads Tilt: a rule, a function to Brightness. It has no value of its own — a
value's formula applies it…_. A computed value is a Source you then give a
formula.

Only a value shows in the simulator: a rule has no value per tick, so it has no
column in the trace. A rule that nothing applies is a legal state, and the tool
says so: the finding _dimByTilt is a rule nothing applies yet._ on the rule, in
the inspector's _Relationship_ section and on the Simulate page, with the fix
**Add a value that applies dimByTilt** — which creates
`dimByTiltValue : () -> Brightness = dimByTilt(tilt)` when exactly one value
produces each concept the rule reads, asks you to choose when several do, and
says why when none does. A rule is not on the canvas, so the canvas shows
nothing for this; the finding lives with the rule in the inspector and on the
Simulate page. It is not an error: nothing is wrong, something is still to be
written.

## The formula

Formulas are short expressions over the names in scope: the concepts the
relationship reads, the values and rules of the design, numbers with units, and
a few keywords.

```text
Tilt / 90 deg
if AmbientLight > 300 lx then Brightness / 2 else Brightness
adaptBrightness(dimByTilt(tilt), ambient)
delay(0, acc + x)
```

A formula is checked for **dimension** (units must work out), for **meaning**
(the result must be the concept the signature promises), and for **shape** (a
rule cannot be used where a value is expected, and the other way round). The
full language — operators, `if`, `let`, `match`, memory with `delay`, values
from other domains with `sync` — is in
[Formula language](../reference/formula-language.md); the editor is described in
[Formula editor](../studio/formula-editor.md).

## Links on the canvas

Three kinds of link, all socket to socket in a concept's colour, told apart by
where they end. From a block's output socket into a **mapping block**'s input
socket: _this formula reads that block_ — `brightness`'s mapping block receives
one from `tilt`. From a mapping block's output socket into the block beside it:
_this formula defines that block_ — the one definition, drawn as a short joint.
From a block's output socket into an output: _this block drives that output_.
The read links are the **dependency**, read off the compiler's analysis of the
formula, never off the text as you type it; they follow the formula. A rule
(`dimByTilt`) is not on the canvas: the mapping block that applies it names it
in its header, and the rule's own signature is edited in its inspector, in the
sidebar. No link shows the arithmetic or an order of execution: the canvas is a
picture of values and of dependency, not a flowchart.

## What the inspector shows

- **Meaning** — name, description and _Role_: _Source_, _Rule_ or _Value_ (or
  the port the relationship backs), with one sentence.
- **Reads** / **Produces** — the signature. For a rule, _Reads_ is the parameter
  editor: chips with the concepts' glyphs, a pop-up to add one; for a block
  there is nothing to read in the signature, and _Produces_ (titled _Provides_
  for a Source) is its concept. Changing either is an edit; the inspector says
  which relationships will be rechecked.
- **Relationship** — the formula editor: _Add definition_ / _Save definition_ /
  _Revert_ / _Detach definition_, the verdict line, and any findings about the
  formula; then, for a block, _Reads_ (the blocks the formula names — the
  canvas's links into it), _Applies_ (the rules it applies) and _Read by_ (the
  blocks whose formulas name it); for a rule, _Depends on_ and _Named in_ — as
  names you can click; then the findings about the relationship's place in the
  design — a rule nothing applies, with its fix beside it.
- **Timing** — _Updates in_: the timing domain, or _Any timing domain_ for a
  pure rule. See [Timing](timing.md).
- **Drives** — the physical output this value is the final target of, if any.
  See [Physical outputs](physical-outputs.md).
- **Fixes** — actions the tool offers for findings on this relationship.
- **Explain** — collapsed by default: the formal detail.

## Going deeper

_For language implementers._ A relationship is a declaration with an interface
(the signature elaborated over the concepts' representations) and an optional
realization (the formula elaborated to a Core term, a lambda over `rep` of each
input ending in `mk` of the output). A declaration without a realization is the
language's unresolved declaration, a first-class state.
`docs/architecture/compiler-pipeline.md` passes 3–6;
`docs/spec/textual-syntax.md` §11 for what elaborates; ADR-0013 for the formula
language's naming rule. The links on the canvas are the kernel's `dependsOn`
(`docs/spec/kernel.md` §5) — `refs` of the realization, carried per relationship
in the analysis (`MappingAnalysis.references`, ADR-0034, ADR-0044).

## Related

[Concepts](concepts.md) · [Incomplete designs](incomplete-designs.md) ·
[Formula editor](../studio/formula-editor.md) ·
[Formula language](../reference/formula-language.md) ·
[Simulate](../studio/simulate.md)
