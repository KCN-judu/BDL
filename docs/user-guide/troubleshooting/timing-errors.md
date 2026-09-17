# Timing

Findings about *when* values exist: under a relationship's **Timing**
section, in the status line, and on the Simulate page's readiness list.
The ideas are in [Timing](../concepts/timing.md).

## *These relationships depend on each other in the same instant: a, b. One of them must read the previous value instead.* (Simulate) / *a depends on its own current value.* (inspector) / status line *not causal*

**What it means.** Two or more values need each other within one tick —
`a` uses `b` and `b` uses `a`, or a value uses itself — so there is no
order in which they could be computed.

**Why.** Within a tick every value is computed once from the current
inputs. A cycle has no first value.

**What to do.** Decide which of them should use *last tick's* value and
write it with `delay`: `a = delay(0, b + 1)`. A cycle through a `delay` is
not instantaneous. If the cycle is a mistake, break it by changing what
one of the formulas reads. Code: `reactive.instantaneous_cycle`.

## *shown updates in a different timing domain from brightness. Choose how this relationship should observe the source value.* / status line *reads across domains*

**What it means.** A value's formula names a value from another timing
domain directly.

**Why.** Two domains share no instant; "the current brightness" has no
meaning in the display's tick.

**What to do.** Either read it through `sync(interaction, 0, brightness)`
— the last value the source produced strictly before this tick, with a
stated value to use before the source has ever run — or move the reader
into the source's domain (*Updates in*) and accept its timing. The
*Insert explicit sync* fix is listed but blocked today; write the `sync`
yourself. Code: `clock.cross_domain_reference`.

## *acc remembers a value over time but has no timing domain.*

**What it means.** A formula uses `delay` or `sync` but its relationship
is on *Any timing domain*.

**Why.** Remembering means reading a domain at its previous activation;
without a domain there is no "previous".

**What to do.** Set *Updates in* on the relationship. Code:
`clock.temporal_without_domain`.

## *`delay` can only be used in a relationship without inputs.*

**What it means.** `delay` or `sync` was written inside a rule (a
relationship with inputs).

**Why.** Memory belongs to a value as a whole — one value per activation
— not to a formula over its inputs.

**What to do.** Read the inputs through values (relationships without
inputs) and remember there: make the rule pure, and put the `delay` in
the value that applies it. Code: `formula.temporal.under_inputs`.

## *`delay` cannot be used inside a block's result or a match arm.*

**What to do.** Bind it first — `let previous = delay(0, x);` — or make it
the value the `match` looks at. Code: `formula.temporal.under_binder`.

## *`fast` is not a timing domain of this design.*

**What to do.** The first argument of `sync` is the *source's* domain,
spelled as in the sidebar's *Timing domains*; create it first if it does
not exist. Code: `formula.sync.unknown_domain`.

## The trace has an empty cell where you expected a value

**What it means.** The value's domain did not activate at that tick (the
*active* column names the ones that did). Not an error.

**What to do.** Nothing, or change that domain's period on the Simulate
page.

## An instance's *Timing* says a domain is missing — *second does not say which timing domain main is.*

**What to do.** Choose a system domain for each of the instance's timing
parameters in its inspector. Code: `system.clock_argument_missing`.

## Related

[Timing](../concepts/timing.md) ·
[Carrying a value across timing domains](../workflows/cross-domain-transport.md)
