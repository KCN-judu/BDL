# Types, units and concepts

Findings under the formula field, in red. The formula was checked as you typed;
the relationship becomes _invalid_ only if you save it as it is. The rules
behind these are in [Concepts](../concepts/concepts.md) and
[Relationships](../concepts/relationships.md).

## _Brightness is a dimensionless quantity, but this formula produces an angle._

**What it means.** The signature promises one concept; the formula's result has
a different value form or dimension. Here a _Tilt_ was returned where a
_Brightness_ (a plain number) was expected.

**Why.** A relationship produces exactly the concept it promises. The result is
not scaled or reinterpreted.

**What to do.** Make the arithmetic come out in the promised dimension —
`Tilt / 90 deg` divides an angle by an angle — or change _Produces_ if the
relationship really produces something else. Code: `realization.type_mismatch`.

## _This expression adds values with different physical dimensions: an angle and a time._

**What it means.** Two operands of `+`, `-`, a comparison, or `if` branches do
not have the same dimension.

**Why.** An angle plus a time has no meaning; BDL checks dimensions like a
physicist does.

**What to do.** Usually a unit is missing on a number: `Tilt > 45` should be
`Tilt > 45 deg`. Multiplication and division combine dimensions freely;
addition, subtraction and comparison need the same one. Code:
`dimension.mismatch`.

## _`foobar` is not a unit._ — _Units available: rad deg s ms min h m mm cm km kg g A mA K cd mol Hz N Pa kPa W V mV lx._

**What to do.** Use one of the listed units, or write the quantity without a
unit if it really is a plain number. Code: `formula.unit.unknown`.

## _`x` is not something this mapping reads or can call._

**What it means.** A name in the formula is neither a concept this relationship
reads nor a relationship of the design.

**What to do.** Inside a **rule**, name the concepts it reads (_Tilt_, as in the
_Reads_ section). Inside a **value**, name the design's values (`tilt`) and
apply its rules (`dimByTilt(tilt)`). Check the spelling — completion (⌃Space)
lists what is in scope. Code: `formula.name.unknown`.

## _dimByTilt does not read AmbientLight._

**What it means.** The formula names a concept the relationship does not read.

**Why.** A rule sees only its inputs; that is what makes it a rule.

**What to do.** Add the concept under _Reads_ (the fix _Connect AmbientLight to
dimByTilt as an input_ does it), or drop it from the formula. Code:
`formula.name.not_an_input`.

## _dimByTilt reads Tilt here, but this is Brightness._

**What it means.** You applied a rule to a value of the wrong concept.

**Why.** Concepts are identities: a _Brightness_ is never a _Tilt_, even if both
are plain numbers.

**What to do.** Pass a Tilt — an input of this relationship, or a value that
produces Tilt. Code: `formula.call.argument_type`.

## _tilt reads nothing; it is a value, not something to apply._

**What to do.** Write `tilt`, not `tilt()`. Code: `formula.call.arity`.

## _dimByTilt reads Tilt; give it those values._

**What it means.** A rule was named as if it were a value.

**Why.** A rule with inputs can only be applied; it cannot be stored or passed
along.

**What to do.** Write `dimByTilt(tilt)`. Code:
`formula.mapping.needs_arguments`.

## _Only a named relationship can be applied._ / _Tilt is a concept, not a relationship; it cannot be applied._

**What to do.** Calls apply a relationship of the design to its inputs; the
result of a call cannot be called again (`f(x)(y)`), and a concept has nothing
to apply. Code: `formula.call.not_a_relationship`.

## A number pattern in `match` is refused — _This pattern is a plain number, but the value matched is a count._

**What to do.** Counts cannot be matched against number patterns; write
`if x == 10 then … else …`. Code: `formula.unsupported` / `dimension.mismatch`.

## _… is not exhaustive_ / an arm is unreachable

**What to do.** Add the missing case (`_ =>` catches the rest), or remove the
arm that can never match. Codes: `formula.match.non_exhaustive` (error),
`formula.match.unreachable` (warning).

## A syntax error

**What it means.** The text does not parse; the finding points at the place and
says what was expected (_comparisons cannot be chained — write
`a < b && b < c`_, _a unit can only follow a number, as in `90 deg`_).

**What to do.** Fix the text; the check re-runs as you type. Code:
`formula.parse.unexpected_token`.

## Related

[Formula language](../reference/formula-language.md) ·
[Formula editor](../studio/formula-editor.md)
