# Syntax basics

What a `.bdl` file looks like and the forms that are checked today.
Everything here parses; the *Checked?* notes say whether it is also
checked and runs. The normative grammar is `docs/spec/textual-syntax.md`.

## A file

A file is a sequence of **items**. Items start with a keyword and end
where the next keyword starts; newlines carry no meaning; `//` and
`/* … */` are comments. A project is any number of files under `src/`,
read in path order; an item may refer to anything declared in any file.

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
mapping level : Brightness @main    // reads nothing: a value, in the timing domain `main`
level() = 0.5
```

The definition repeats the name and gives one parameter per input. A
relationship that reads nothing is written `mapping level : Brightness`
and defined `level() = …`. Without a definition, the relationship is
*declared* — an input, or work still to do. `@domain` after the
signature puts the relationship in a timing domain
([Timing domains](../concepts/timing.md)); without it the
relationship serves any domain.

**Checked?** Yes. The parameter names are the names the body uses:
`dimByTilt(t) = t / (90 deg)` reads its input as `t`. Renaming the
concept does not change `t`; renaming the parameter is an edit of the
definition.

## Descriptions

A run of `///` lines directly above an item is its description — the
text Studio shows in the sheet and in *Explain*. Ordinary `//` and
`/* … */` comments are yours and are kept where you wrote them.

```bdl
/// How bright the lamp is for a tilt.
mapping dimByTilt : Tilt -> Brightness
```

## Timing domains, outputs, drivers, devices

```bdl
clock interaction
clock display

output light : Brightness @interaction              // required: the design is incomplete until driven
output indicator : Brightness @interaction optional // may stay undriven
drive light = brightness                            // the relationship that is light's final driver

device pwmLight : pwm_channel for light { pin 0 = D3 }
device imu : i2c_sensor
```

A physical output names the concept it accepts and its domain;
`optional` marks one that may stay undriven. `drive` names one
relationship as an output's driver — a second `drive` of the same
output is a finding, and the first stays. A device has a kind —
`pwm_channel`, `digital_output`, `h_bridge_channel`, `i2c_sensor`,
`quadrature_encoder`, `uart` — optionally the output it realises (`for
light`) and pins fixed by position (`pin 0 = D3`).

**Checked?** Yes: drivers are checked as in Studio (one driver, right
concept and domain); devices take part in deployment analysis.

## Components, instances, bindings

```bdl
component AdaptiveLamp {
  use concept Tilt              // a concept of the project, shared
  use concept Brightness
  use concept Gain
  param clock main              // a domain the instance supplies
  clock blink                   // a domain private to each instance

  requires tiltValue : Tilt @main       // a port someone must bind
  param gain : Gain                     // a constant each instance sets

  mapping dimByTilt : Tilt -> Brightness
  dimByTilt(t) = t / (90 deg)

  provides brightness : Brightness @main
  brightness() = dimByTilt(tiltValue) * gain
}

instance lampA : AdaptiveLamp { main = interaction, gain = 2 }
instance lampB : AdaptiveLamp { main = interaction, gain = 1 }

mapping tiltValue : Tilt @interaction
mapping slow : Brightness @display

bind lampA.tiltValue = tiltValue          // a required port reads a relationship
bind lampB.tiltValue = tiltValue
bind slow = lampA.brightness init 0       // crosses domains: needs a starting value

export lampB.tiltValue as tiltIn          // an unbound port becomes an input of the whole system
```

Inside a component body the ordinary items apply (`concept`, `mapping`,
`clock`, `output`, `drive`, `device`); a concept declared *in* the body
is private to each instance, a `use concept` is the project's concept
shared by all. `requires`, `provides` and `param` declare ports — each is
a relationship of the body with a contract other components can rely on
([Components](../concepts/components.md)). An instance gives every clock
parameter a domain and every `param` a constant, in braces. `bind`
connects an instance's port to a relationship or to another instance's
port; when the two sides update in different domains, `init` gives the
value before the source has one.

**Checked?** Yes — composition is validated as in Studio: a port bound
to the wrong concept, an instance missing a parameter, a domain crossing
without `init`, are findings on the line concerned.

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
| a relationship's parameter | `t` in `dimByTilt(t) = t / (90 deg)` | yes — the parameter name is the name; the concept's name is not needed |
| generic types in a signature | `Option<Brightness>` | parses; meaning decided by the checker |

Units attach to numbers only: `90 deg` is a literal; `x deg` is a syntax
error. A dimensioned expression gets its dimension from arithmetic:
`(tilt + offset) / (90 deg)`.

## Patterns

`_`, a name, `true` / `false`, a whole number, `Some(p)`, `None`. No
guards, ranges or or-patterns.

## Where Studio and text differ

The formula field in Studio is the expression language above — nothing
more, nothing less. What the file adds is the items around it. Two
things Studio has that the text does not carry: **behavior groups** and
**canvas layout** — both are authoring metadata, kept in files beside
the sources ([Project files](../reference/project-files.md)). `enum`
items parse but are reported as *open*: the design model has no sum
types yet. The words `context` and `require` are reserved.

## Related

[Overview and current status](overview.md) ·
[Editor and language server](editor-and-lsp.md) ·
[Authoring a project as text](../workflows/authoring-as-text.md) ·
[Formula language](../reference/formula-language.md)
