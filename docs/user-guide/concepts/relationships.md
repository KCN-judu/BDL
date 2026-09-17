# Relationships

A **relationship** says how one concept follows from others. `dimByTilt`
reads *Tilt* and produces *Brightness*; `adaptBrightness` reads
*Brightness* and *Ambient light* and produces *Brightness*. A relationship
has a name, a **signature** — the concepts it *reads* and the one it
*produces* — and, once you write it, a **formula**.

In the sidebar and the creation sheet, relationships are labelled
*Mappings*. Same thing.

## Signature first, formula second

Creating a relationship only fixes its signature. That is enough for the
rest of the design to be built around it: other relationships can apply
it, an output can wait for it, the simulator can list what it still
needs. The formula comes when you are ready, in the inspector's
*Relationship* section. Until then the node is dashed and *declared*.

The formula is checked against the signature as you type, and the check
is in your own terms: *Brightness is a dimensionless quantity, but this
formula produces an angle.*

## Three shapes of relationship

| Shape | Reads | Formula | What it is |
|---|---|---|---|
| **Rule** | one or more concepts | yes | a function from concepts to a concept: `dimByTilt : Tilt → Brightness = Tilt / 90 deg` |
| **Input** | nothing | none | a value that arrives from outside — a sensor, a switch. In simulation you type it; on a device the hardware supplies it |
| **Computed value** | nothing | yes | a value of the design: `brightness = dimByTilt(tilt)` |

Only a relationship that *reads nothing* is a value; a rule is something
you apply. This matters in two places: a physical output can only be
driven by a value, and a formula applies a rule by calling it —
`dimByTilt(tilt)` — and names a value by writing it — `tilt`.

## The formula

Formulas are short expressions over the names in scope: the concepts the
relationship reads, the values and rules of the design, numbers with
units, and a few keywords.

```
Tilt / 90 deg
if AmbientLight > 300 lx then Brightness / 2 else Brightness
adaptBrightness(dimByTilt(tilt), ambient)
delay(0, acc + x)
```

A formula is checked for **dimension** (units must work out), for
**meaning** (the result must be the concept the signature promises), and
for **shape** (a rule cannot be used where a value is expected, and the
other way round). The full language — operators, `if`, `let`, `match`,
memory with `delay`, values from other domains with `sync` — is in
[Formula language](../reference/formula-language.md); the editor is
described in [Formula editor](../studio/formula-editor.md).

## Links on the canvas

A link from a concept's output socket to a relationship's input socket
means *this relationship reads that concept*. Drawing one edits the
signature; dragging one away from an input and dropping it on empty
canvas removes the concept from the signature. Links show *what is read*,
never the arithmetic, and never an order of execution: the canvas is a
picture of dependency, not a flowchart.

## What the inspector shows

* **Meaning** — name and description.
* **Reads** / **Produces** — the signature, as chips and a pop-up with the
  concepts' glyphs. Changing either is an edit; the inspector says which
  relationships will be rechecked.
* **Relationship** — the formula editor: *Add definition* / *Save
  definition* / *Revert* / *Detach definition*, the verdict line, and any
  findings about the formula.
* **Timing** — *Updates in*: the timing domain, or *Any timing domain* for
  a pure rule. See [Timing](timing.md).
* **Drives** — the physical output this value is the final target of, if
  any. See [Physical outputs](physical-outputs.md).
* **Fixes** — actions the tool offers for findings on this relationship.
* **Explain** — collapsed by default: the formal detail.

## Going deeper

*For language implementers.* A relationship is a declaration with an
interface (the signature elaborated over the concepts' representations)
and an optional realization (the formula elaborated to a Core term, a
lambda over `rep` of each input ending in `mk` of the output). A
declaration without a realization is the language's unresolved
declaration, a first-class state. `docs/COMPILER_PIPELINE.md` passes 3–6;
`docs/TEXTUAL_SYNTAX.md` §11 for what elaborates; ADR-0013 for the
formula language's naming rule.

## Related

[Concepts](concepts.md) · [Incomplete designs](incomplete-designs.md) ·
[Formula editor](../studio/formula-editor.md) ·
[Formula language](../reference/formula-language.md)
