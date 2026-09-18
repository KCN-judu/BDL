# Formula language

What can be written in a relationship's formula — in Studio's field and
in a `.bdl` definition body, which share one grammar and one checker.

## Names

| In a **rule** (a relationship with inputs) | In a **value** (a relationship without inputs) |
| --- | --- |
| the concepts it reads, by name: `Tilt`, `AmbientLight` | the design's values: `tilt`, `ambient`, `brightness` |
| the design's rules, applied: `dimByTilt(Tilt)` | the design's rules, applied: `dimByTilt(tilt)` |

A rule sees only what it reads; a value sees the design's other values
and rules. A rule is always applied (`f(x)`), never named alone; a value
is always named alone, never called (`tilt`, not `tilt()`).

## Literals

| | Example |
| --- | --- |
| number | `0`, `0.5`, `1e-3`, `-2` |
| number with a unit | `90 deg`, `2.5 s`, `300 lx`, `25.4 mm` |
| on / off | `true`, `false` |

Units: `rad deg · s ms min h · m mm cm km · kg g · A mA · K · cd · mol ·
Hz · N · Pa kPa · W · V mV · lx`. A unit follows a number only; `x deg` is
a syntax error. A quantity's dimension comes from arithmetic — write
`Tilt / (90 deg)`, not `(Tilt) deg`.

## Operators, loosest to tightest

| | Operators | Notes |
| --- | --- | --- |
| or | `\|\|` | |
| and | `&&` | |
| equality | `==` `!=` | same dimension both sides; no chaining |
| comparison | `<` `<=` `>` `>=` `in` | same dimension both sides; no chaining (`a < b && b < c`); `x in xs` is membership, `x in lo .. hi` a closed range |
| range | `lo .. hi` | only after `in`; both ends must be comparable with the value: `angle in -45 deg .. 45 deg` |
| default | `x ?? d` | `x` when it has a value, `d` when it is absent |
| additive | `+` `-` | same dimension both sides |
| multiplicative | `*` `/` | dimensions combine: `m / s` is a speed |
| prefix | `!` `-` | |
| call | `f(x, y)` | |

Concepts and their values: a concept in a formula is used through its
value form — `Tilt / 90 deg` divides the angle — and the result must
have the value form of the concept the relationship produces.

## Forms

| Form | Example | Meaning |
| --- | --- | --- |
| conditional | `if Held then dimByTilt(Tilt) else 0` | both branches the same kind |
| block with bindings | `{ let half = Brightness / 2; let dark = AmbientLight < 10 lx; if dark then Brightness else half }` | `let` names a value for the rest of the block; may shadow |
| options | `Some(e)`, `None` | an optional value; `None` takes its kind from a sibling |
| match | `match reading { Some(v) => v, None => 0 }` | on true/false, options, whole numbers, or a concept's value; must be exhaustive (`_ =>` catches the rest) |
| memory | `delay(0, acc + x)` | last tick's value of the expression; `0` before there was one |
| across domains | `sync(interaction, 0, brightness)` | the source domain's last value strictly before this tick; `0` before the source has run |
| every / some element | `all reading in readings: reading < limit` · `any fault in faults: fault > 2` | true or false for a whole collection; the name after the word stands for one element inside the body only |
| transform / keep | `map reading in readings: reading / 2` · `filter reading in readings: reading in 10 K .. 40 K` | a new collection: each element transformed, or the elements that pass |

`delay` and `sync` belong in a **value's** formula (no inputs), at the top
level or in a `let`, an `if` branch or a `match` scrutinee — not in a
rule, a `match` arm or a block's result. Each written `delay` / `sync` is
its own memory cell. The relationship must have a timing domain.

## What is checked

| Check | Example finding |
| --- | --- |
| dimensions | *This expression adds values with different physical dimensions: an angle and a time.* |
| the produced concept | *Brightness is a dimensionless quantity, but this formula produces an angle.* |
| names in scope | *`x` is not something this mapping reads or can call.* — inside `all x in xs: …` the fix lists the locals in scope |
| binders and ranges | *all expects a collection after 'in'.* · *The body of 'filter' must be true or false.* · *This range endpoint must be an angle.* |
| concept identity in calls | *dimByTilt reads Tilt here, but this is Brightness.* |
| arity and shape | *dimByTilt reads Tilt; give it those values.* · *tilt reads nothing; it is a value, not something to apply.* |
| placement of memory | *`delay` can only be used in a relationship without inputs.* |
| exhaustiveness | *… non-exhaustive*; unreachable arms are warnings |
| units | *`foobar` is not a unit.* |

Not in the language: user-defined functions beyond the design's
relationships, loops, strings, lists, user enums (they parse in text but
cannot be used yet), unit casts on expressions.

## Related

[Formula editor](../studio/formula-editor.md) ·
[Types, units and concepts](../troubleshooting/type-and-concept-errors.md) ·
[Syntax basics](../textual/syntax-basics.md)
