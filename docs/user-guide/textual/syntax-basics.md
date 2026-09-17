# Syntax basics

What a `.bdl` file looks like and the forms that are checked today.
Everything here parses; the *Checked?* notes say whether it is also
checked and runs. The normative grammar is `docs/TEXTUAL_SYNTAX.md`.

## A file

A file is a sequence of **items**. Items start with a keyword and end
where the next keyword starts; newlines carry no meaning; `//` and
`/* … */` are comments.

```bdl
concept Tilt : Angle
concept Brightness : Scalar
concept Held : Bool
concept Pulse                       // value form decided later

mapping dimByTilt : Tilt -> Brightness
dimByTilt(tilt) =
  tilt / (90 deg)

mapping chooseBrightness : Held -> Tilt -> Brightness
chooseBrightness(held, tilt) =
  if held then dimByTilt(tilt) else 0

mapping level : Brightness           // reads nothing: a value
level() = 0.5
```

## Concepts

```
concept Name : Type
concept Name
```

`Type` names the value form: `Bool`, `Count`, or a quantity kind —
`Scalar` (no unit), `Angle`, `Length`, `Time`, `Mass`, `Current`,
`Temperature`, `Amount`, `Luminous`, and derived kinds such as `Speed`,
`Frequency`, `Force`, `Pressure`, `Voltage`, `Illuminance`. Leaving the
type off declares a concept whose value form is decided later.

## Relationships

```
mapping name : A -> B -> C          // the signature: reads A and B, produces C
name(a, b) = expr                   // the definition; optional
```

The definition repeats the name and gives one parameter per input. A
relationship that reads nothing is written `mapping level : Brightness`
and defined `level() = …`. Without a definition, the relationship is
*declared* — an input, or work still to do.

**Checked?** Yes. Today the body is checked against the *concept's* name:
`tilt` resolves to `Tilt` because they spell the same (case-insensitively);
another parameter name is reported as unknown.

## Expressions

| Form | Example | Checked? |
|---|---|---|
| numbers, with a unit | `90 deg`, `2.5 s`, `300 lx`, `0.5` | yes — units are checked as dimensions; unknown units are reported |
| arithmetic | `a + b`, `a - b`, `a * b`, `a / b`, `-a` | yes |
| comparison | `<`, `<=`, `>`, `>=`, `==`, `!=` (no chaining: write `a < b && b < c`) | yes |
| logic | `&&`, `\|\|`, `!` | yes |
| conditional | `if c then a else b` | yes |
| call a relationship | `dimByTilt(tilt)` | yes — with the right number of arguments; a rule as a *value* or a partial call is refused |
| name a value | `tilt` | yes |
| block with `let` | `{ let x = …; let y = …; x + y }` | yes |
| options | `Some(e)`, `None` | yes |
| `match` | on true/false, options, whole numbers, and a concept through its value | yes, with exhaustiveness checking |
| memory | `delay(init, e)` | yes — in a value's formula; at the top level or in a `let`, an `if` branch or a `match` scrutinee |
| cross-domain read | `sync(domain, init, e)` | yes — same placement rule |
| `enum` items and their constructors | `enum Mode { Off, Manual(Brightness) }` | parses; reported as *open* — not usable in formulas yet |
| generic types in a signature | `Option<Brightness>` | parses; meaning decided by the checker |

Units attach to numbers only: `90 deg` is a literal; `x deg` is a syntax
error. A dimensioned expression gets its dimension from arithmetic:
`(tilt + offset) / (90 deg)`.

## Patterns

`_`, a name, `true` / `false`, a whole number, `Some(p)`, `None`. No
guards, ranges or or-patterns.

## Where Studio and text differ

The formula field in Studio is the expression language above — nothing
more, nothing less. What the file adds is the items around it. What the
file lacks is everything Studio has beyond concepts and relationships:
timing domains, physical outputs, devices, behaviors and components have
no text form yet; the words `clock`, `output`, `context`, `component`,
`require` are reserved for them.

## Related

[Overview and current status](overview.md) ·
[Editor and language server](editor-and-lsp.md) ·
[Formula language](../reference/formula-language.md)
