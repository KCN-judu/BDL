# Carrying a value across timing domains

**Goal.** Show the lamp's brightness on a slow display that refreshes on its own
rhythm — and see why BDL asks for an initial value when a value crosses from one
timing domain to another.

There are two places this happens, with one rule behind them: inside a
**formula** (`sync`), and on a **binding** between an instance's port and
something in another domain (_Starts at_).

## In a formula

1. **A second domain.** _Timing domains_ **+**: `display`.
2. **A value in it.** _Mappings_ **+**: `shown`, reads nothing, produces
   Brightness, **Updates in** _display_.
3. **Read across.** Give `shown` the formula `brightness` and _Add definition_.
   The status line says **reads across domains**, and under the relationship's
   _Timing_ section: _shown updates in a different timing domain from
   brightness. Choose how this relationship should observe the source value._
4. **Carry it.** Change the formula to

   ```text
   sync(interaction, 0, brightness)
   ```

   _Save definition._ The finding is gone. `shown` now reads the last value
   `brightness` had at _interaction_'s most recent activation strictly before
   `display`'s tick, and `0` until _interaction_ has run.

5. **Simulate.** ⌘2. Set _display_ to _every 3_. Step ×10: `shown` is filled
   only on ticks 0, 3, 6, 9; at tick 0 it reads `Brightness(0)` — the initial
   value — and afterwards the brightness from the last _interaction_ tick
   _before_ its own.

## On a binding

With the system from [Composing components](composing-components.md):

1. **A value in another domain.** _Timing domains_ **+**: `aux`; _Mappings_
   **+**: `slow`, reads nothing, produces Brightness, updates in _aux_, no
   formula.
2. **Bind across.** Drag from `adaptiveLamp`'s _brightness_ socket (domain
   _main_) onto `slow`. A sheet: _Carry across timing domains_ — from _main_ to
   _aux_ — with a **Starts at** field. Enter `0` and confirm.
3. The link carries a gate mark; the binding's inspector says _Carried across
   timing domains: the destination sees the last value …_ with _Starts at 0_.
   Cancelling instead makes no binding: a transport is never inserted for you.

<!-- figure F15 -->

## What BDL means by this

- Two domains have **no shared instant**. "The current brightness" has no
  meaning in the display's tick; only "the brightness at _interaction_'s last
  tick before now" does. That is what `sync` and a transported binding read.
- **Strictly before**: a source value produced at the same global tick is not
  visible yet. This is what makes the result independent of the order in which a
  device happens to process two domains that fire together.
- The **initial value** is yours to choose, in the destination's units, because
  the destination may tick before the source ever has. BDL does not invent a
  zero.
- A transport is a **meaning-changing decision**, so it is always written (in
  the formula) or confirmed (in the sheet); the tool offers no automatic sync.

## If it does not work

- _reads across domains_ stays — the formula still names a value from another
  domain outside `sync`; every such read needs its own `sync`.
- `sync` refused _under inputs_ / _under a binder_ — `sync` (like `delay`)
  belongs in a value's formula, at the top level or in a `let`, an `if` branch
  or a `match` scrutinee; not in a rule with inputs, a `match` arm or a block's
  result.
- The domain name in `sync(...)` is unknown — it is the _source's_ domain,
  spelled as in the sidebar.

## Related

[Timing](../concepts/timing.md) ·
[Troubleshooting: timing](../troubleshooting/timing-errors.md)
