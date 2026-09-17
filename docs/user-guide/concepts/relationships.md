# Relationships

A **relationship** says how one concept follows from others. `dimByTilt` reads
_Tilt_ and produces _Brightness_; `adaptBrightness` reads _Brightness_ and
_Ambient light_ and produces _Brightness_. A relationship has a name, a
**signature** — the concepts it _reads_ and the one it _produces_ — and, once
you write it, a **formula**.

In the sidebar and the creation sheet, relationships are labelled _Mappings_.
Same thing.

## Signature first, formula second

Creating a relationship only fixes its signature. That is enough for the rest of
the design to be built around it: other relationships can apply it, an output
can wait for it, the simulator can list what it still needs. The formula comes
when you are ready, in the inspector's _Relationship_ section. Until then the
node is dashed and _declared_.

The formula is checked against the signature as you type, and the check is in
your own terms: _Brightness is a dimensionless quantity, but this formula
produces an angle._

## Three shapes of relationship

| Shape              | Reads                | Formula | What it is                                                                                                              |
| ------------------ | -------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------- |
| **Rule**           | one or more concepts | yes     | a function from concepts to a concept: `dimByTilt : Tilt → Brightness = Tilt / 90 deg`                                  |
| **Input**          | nothing              | none    | a value that arrives from outside — a sensor, a switch. In simulation you type it; on a device the hardware supplies it |
| **Computed value** | nothing              | yes     | a value of the design: `brightness = dimByTilt(tilt)`                                                                   |

Only a relationship that _reads nothing_ is a value; a rule is something you
apply. This matters in two places: a physical output can only be driven by a
value, and a formula applies a rule by calling it — `dimByTilt(tilt)` — and
names a value by writing it — `tilt`.

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

A link from a concept's output socket to a relationship's input socket means
_this relationship reads that concept_. Drawing one edits the signature;
dragging one away from an input and dropping it on empty canvas removes the
concept from the signature. Links show _what is read_, never the arithmetic, and
never an order of execution: the canvas is a picture of dependency, not a
flowchart.

## What the inspector shows

- **Meaning** — name and description.
- **Reads** / **Produces** — the signature, as chips and a pop-up with the
  concepts' glyphs. Changing either is an edit; the inspector says which
  relationships will be rechecked.
- **Relationship** — the formula editor: _Add definition_ / _Save definition_ /
  _Revert_ / _Detach definition_, the verdict line, and any findings about the
  formula.
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
language's naming rule.

## Related

[Concepts](concepts.md) · [Incomplete designs](incomplete-designs.md) ·
[Formula editor](../studio/formula-editor.md) ·
[Formula language](../reference/formula-language.md)
