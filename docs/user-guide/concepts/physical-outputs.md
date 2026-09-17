# Physical outputs

A **physical output** is where a value leaves the design for the world: a
light, a motor, a display, a buzzer. It is not a concept and not a
relationship. It **accepts** one concept, **updates in** one timing
domain, and is **driven** by exactly one value of the design.

Do not confuse it with a relationship's *output* — the concept a
relationship produces. `dimByTilt` produces *Brightness*; *Light Output*
is a physical output that accepts *Brightness*. The first is a meaning
inside the design; the second is the lamp.

## The rules, in designer terms

| Rule | Why |
|---|---|
| An output accepts **one concept**. | The lamp shows a brightness, not "a number". Connecting an *Opacity* would be a mistake the tool can catch. |
| An output has **one timing domain**, and its driver must update in that domain. | The physical device refreshes at the domain's rhythm; a driver on another rhythm would leave the output reading a value from an unrelated instant. |
| An output is driven by a **value** — a relationship that reads nothing. | A rule is something you apply; only a value can be *the* brightness at this tick. `brightness = dimByTilt(tilt)` drives; `dimByTilt` cannot. |
| An output has **at most one driver**. | Two relationships driving one light is a contradiction in the design. BDL names it and does not pick a winner — no priorities, no last-writer. Combine the two values in a relationship and connect that. |
| A **required** output must be driven for the design to be executable; an optional one may stay undriven. | A lamp without its light is unfinished; an optional status LED is not. |

## The states of an output

| State | You see | Meaning |
|---|---|---|
| **open** | dashed sink, *no timing domain yet* | the output exists but has no domain; nothing can drive it yet and nothing is wrong |
| **undriven** | dashed sink; inspector: *Undriven — the design is incomplete without a driver.* (required) or *Undriven.* (optional) | waiting for a driver |
| **driven** | solid sink; *Driven by brightness.* | complete |
| **ill-formed** | the sink shows the word *ill-formed*; the driver carries a finding: wrong concept, wrong domain, or a rule with inputs | the connection was recorded, and it is wrong |
| **contested** | the sink shows the word *contested*; inspector lists every claimant | two or more drivers |

Findings about a connection belong to the *connection*: the driving
relationship itself stays valid, and the finding appears under that
relationship's **Drives** section and under the output's **Driver**
section.

## On the canvas

An output is a **sink node** at the right edge of the canvas: a boundary
bar and one input socket in the accepted concept's colour. Drag from a
value's output socket onto the sink to connect; drag the link away from
the sink and release on empty canvas to disconnect. Only a relationship's
output socket can land on a sink; a concept row cannot drive the world by
itself, and nothing reads from a sink.

## In the inspector

* **Meaning** — name and description.
* **Output** — *Accepts* (the concept), *Updates in*, *Required*.
* **Driver** — the state line, the claimants with *disconnect* links, and a
  **Connect** pop-up listing the design's relationships (those with inputs
  are marked *has inputs*; connecting one is recorded and then reported).
* **Fixes** — for a contested output, *Detach … from …* and *Create
  upstream combination mapping*; for an undriven one, *Connect a driver
  to …* with a choice.

## From output to device

Which piece of hardware realises an output — a PWM channel, a digital
output, an H-bridge — is not part of the design. It is a **device**
binding, edited on the Deploy page, and the board it is placed on is a
per-session choice. Changing either never changes the design's verdicts.
See [Deploy](../studio/deploy.md).

## Going deeper

*For language implementers.* An output is an entry of `Ω` with an
`OutputSpec { accepts, clock }`; a drive edge is `β d = o`. The
well-formedness of a drive (`DriveWF`: exact type view and exact domain),
the single-driver rule (`SingleDriver`) and completeness
(`CompleteOutputs`) are checked by `bdl-output` after the semantic passes
and never move a mapping down the status ladder (ADR-0015).
`docs/guides/deployment-walkthrough.md` follows one value from a relationship to
a pin. DI-20 records why only nullary relationships drive.

## Related

[Relationships](relationships.md) · [Timing](timing.md) ·
[Deploy](../studio/deploy.md) ·
[Troubleshooting: connections](../troubleshooting/connection-errors.md)
