# From sensor to output

**Goal.** Extend the lamp from
[Your first behavior](../getting-started/first-behavior.md) with a second
sensor: in a bright room the lamp need only be half as bright. This is the
design in `examples/smart_lamp`; you can open that project instead of building
it and read along.

The pattern is the one every sensor-driven behavior follows:

```text
input (declared, in a domain)  →  rule(s)  →  value (defined, in the domain)  →  physical output
```

## Steps

1. **A concept for the new sensor.** Sidebar → _Concepts_ **+**: `AmbientLight`,
   Quantity, unit **illuminance** (`lx`). (The Library tab's _Ambient Light_
   template gives the same.)
2. **The input.** _Mappings_ **+**: `ambient`, reads nothing, produces
   AmbientLight. Leave it declared. **Timing › Updates in**: _interaction_.
3. **The rule.** _Mappings_ **+**: `adaptBrightness`, reads **Brightness** and
   **AmbientLight**, produces **Brightness**. Formula:

   ```text
   if AmbientLight > 300 lx then Brightness / 2 else Brightness
   ```

   _Add definition._ A rule may read and produce the same concept: this one
   takes a brightness and returns an adjusted brightness.

4. **Rewire the value.** Select `brightness` and change its formula to

   ```text
   adaptBrightness(dimByTilt(tilt), ambient)
   ```

   _Save definition._ The output is still driven by `brightness`; nothing else
   changes.

5. **Simulate.** ⌘2. Two inputs now: `tilt` and `ambient`. Set tilt `1.5708`,
   ambient `100`, **Step**: `Brightness(1)`. Set ambient `500`, **Step**:
   `Brightness(0.5)`.

## What BDL means by this

- `ambient` and `tilt` are the design's **inputs**: declared values in the
  _interaction_ domain. What supplies them is outside the design.
- `dimByTilt` and `adaptBrightness` are **rules** with no domain of their own;
  they run when `brightness` runs.
- `brightness` is the **one value** the light shows, and the **only driver** of
  _light_. Everything the lamp does is legible from this one formula.
- The comparison `AmbientLight > 300 lx` is checked for dimension like any
  arithmetic: comparing an illuminance with `300` (no unit) would be refused.

## Variations

- **A switch instead of a sensor.** A concept _Held_ with value form **On /
  off** (◇), an input `held`, a rule `chooseBrightness` reading Held and Tilt
  with the formula `if Held then dimByTilt(Tilt) else 0`, and the value
  `brightness = chooseBrightness(held, tilt)`. Inside a rule you name the
  concepts it reads; inside a value you name the design's values and apply its
  rules.
- **A count.** A concept _Presses_ with value form **Count** (□); counts compare
  and add like numbers but carry no unit, and cannot be matched against number
  patterns.
- **Memory.** `acc = delay(0, acc + x)` — see [Timing](../concepts/timing.md).

## If it does not work

- A red line under the formula naming two dimensions that do not fit — a unit is
  missing on a number (`300` instead of `300 lx`), or the two sides of a
  comparison measure different things.
  [Types, units and concepts](../troubleshooting/type-and-concept-errors.md).
- Simulation refuses with _ambient needs a value_ — a new input has no value
  yet; type one.
- The light's cell in the trace is empty — `brightness` lost its domain, or the
  output's domain differs. [Timing](../troubleshooting/timing-errors.md).

## Related

[A second output](multi-output-behavior.md) ·
[Formula language](../reference/formula-language.md)
