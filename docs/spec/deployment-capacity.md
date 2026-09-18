---
kind: specification
area: deployment
status: current
---

# Collections at deployment: bounds, capacity and memory

What a design's collections need of a target with finite memory, how the
toolchain computes it, and what it refuses. The list semantics of
[equation-library.md](equation-library.md) §1 is untouched by everything here: a
list is unbounded data (FV Phase 9a, D-83), a deployment is finite, and the gap
between the two is a **validation obligation** — decided on the lowered plan and
the deployment schedule, never in typing and never by a silent policy of the
generated code (ADR-0024, ADR-0027).

Implemented in `crates/bdl-exec-ir/src/bounds.rs` (the static bounds),
`crates/bdl-reactive/src/capacity.rs` (the window model) and
`crates/bdl-compiler/src/collections.rs` (the report and the diagnostics);
reachable through `CompileOptions { memory, schedule }` and
`bdld compile --bounded-memory --period domain=N`.

## 1. Three questions, three answers

| Question                                           | Answer                                                                                              | Where                       |
| -------------------------------------------------- | --------------------------------------------------------------------------------------------------- | --------------------------- |
| How large can each list the design carries get?    | a static, sound upper bound per declaration and per state cell — §2                                 | `bdl-exec-ir::bounds`       |
| How many values cross between two domains at once? | the window model of FV `Validation/Capacity.lean` over the deployment schedule — §3                 | `bdl-reactive::capacity`    |
| Does the design fit the target?                    | a readiness class, byte estimates, and `deployment.*` diagnostics; refusal on a bounded target — §4 | `bdl-compiler::collections` |

## 2. Static bounds

Every list position of every value gets a **bound**:

| Bound                 | Meaning                                                                                               |
| --------------------- | ----------------------------------------------------------------------------------------------------- |
| `finite { elements }` | at most this many elements, whatever the inputs                                                       |
| `input`               | as large as a list the host supplies (an unresolved declaration of list type, or built from one)      |
| `unbounded`           | the design itself grows it without bound over time (`cons x (delay [] log)`: one more per activation) |

The analysis is an abstract interpretation of the executable IR (after the
equation library has been inlined, so `map`, `filter`, `append`, `zip` are folds
and need no special cases):

- `[]` is 0; `cons` adds one; `take k` with a constant `k` narrows to `k`;
  `drop`, `reverse` keep the bound; `head`/`toList` move between a list and an
  option; a conditional takes the larger branch; a pair or option carries its
  parts.
- A **fold** is run twice abstractly: an accumulator that does not change is
  stable, one that grows by a constant per element grows by that constant times
  the list's bound (so `map`/`filter`/`append` of an input-sized list are
  input-sized and of a 3-element list at most 3 more), anything else is
  unbounded.
- A **state cell** is the fixpoint of its operand: iterated, widened to
  `unbounded` when still moving, then narrowed once — so
  `take cap (cons x (delay [] log))` is bounded by `cap` for any `cap`, and
  `cons x (delay [] log)` is unbounded.

**Soundness (tested):** every bound is an upper bound of every length the
reference evaluator produces on every tick of every corpus case
(`crates/bdl-compiler/tests/collections.rs`,
`every_bound_holds_on_every_tick_of_every_corpus_case`). **Not tight:** `take`
is the one narrowing operator, so a design bounds its state by writing the
bound; an analysis that cannot see a bound reports `unbounded`, never a guess.

Byte estimates follow the generated core's storage: a `Vec` header of 24 bytes
plus `elements × element size`, `f64`/`u64` 8 bytes, `bool` 1, an option one
word more, a grouped value the sum. `state_bytes_max` sums the cells;
`tick_bytes_max` sums one tick's declaration values. Both are `None` when
anything is not finite. They are estimates of the core's own storage, not of a
target's heap fragmentation or stack.

## 3. Windows and capacity

The **window** from a source domain to a destination domain at tick `t` is the
source's activations since the destination's last activation strictly before `t`
(FV `windowTicks`). Under the periodic schedules production has
(`Schedule { periods }`, a domain activates at `t` iff `t mod period = 0`):

| Production                                       | FV `Validation/Capacity.lean`  | Strength                                                                                                   |
| ------------------------------------------------ | ------------------------------ | ---------------------------------------------------------------------------------------------------------- |
| `window_ticks`, `window_len`                     | `windowTicks`, `windowLen`     | transcribed; tested on examples                                                                            |
| `capacity_sufficient(s, src, dst, cap, horizon)` | `CapacitySufficient`           | transcribed (decidable by enumeration)                                                                     |
| `required_capacity(s, src, dst, horizon)`        | `requiredCapacity`             | `requiredCapacity_sufficient` is the property test `required_capacity_is_sufficient`                       |
| `required_capacity_periodic(s, src, dst)`        | `periodic_capacity_sufficient` | the exact bound at every horizon (the windows repeat with the periods' lcm); tested `≤ period(dst)`        |
| `drop_oldest`, `drop_newest`                     | `dropOldest`, `dropNewest`     | `sufficient_capacity_preserves` is the property test `policies_are_the_identity_under_sufficient_capacity` |

`required` for a crossing is what the destination sees as new at most: with
`fast` every tick and `slow` every third, three values; with `fast` every second
tick, two.

## 4. The report and its diagnostics

`collections_report(exec_ir, schedule)` classifies the program:

| Readiness       | Meaning                                                  | Target needs                                               |
| --------------- | -------------------------------------------------------- | ---------------------------------------------------------- |
| `scalar_only`   | no list anywhere; the core is allocation-free and `Copy` | nothing                                                    |
| `bounded`       | every list's size is fixed by the design                 | an allocator; `state_bytes_max` + `tick_bytes_max` of heap |
| `input_bounded` | some list is as large as the host supplies               | an allocator; the platform adapter bounds its inputs       |
| `unbounded`     | a remembered collection grows without bound              | not deployable on finite memory as written                 |

and, for every list-carrying `sync` cell, the crossing's `required` capacity
(when a schedule is given) beside the bound the design gives that list.

Diagnostics, in the designer's words:

| Code                              | When                                                                                                                                          | Severity                                                                 |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `deployment.unbounded_list_state` | a state cell is `unbounded` — "`log` keeps every value it has ever received."                                                                 | warning on a host; **error** under `MemoryPolicy::Bounded` (no artefact) |
| `deployment.list_input_unbounded` | an unresolved declaration of list type — its size is the platform's to bound                                                                  | warning, under `Bounded` only                                            |
| `deployment.window_capacity`      | a synced list keeps fewer values than the crossing needs — "Between two activations of slow, fast produces up to 3 values, but logD keeps 2." | warning (the design chose to keep the newest; said, not silent)          |

The manifest of a list-carrying core records the same facts
(`collections.cells[].bound`, `input_slots`, `state_bytes_max`,
`tick_bytes_max`, `unbounded`;
[codegen-rust.md](../architecture/codegen-rust.md)).

## 5. Overflow: explicit, or refused

FV shows that under sufficient capacity every overflow policy is the identity on
the window and that under insufficient capacity every policy changes the trace;
only refusing the deployment keeps the unbounded semantics. Production follows
exactly that:

- **No policy lives in the generated code.** A core never drops a value.
- **A policy is written in the design.** `take cap (cons x …)` keeps the newest
  `cap` values — FV's `dropOldest` — and the analysis reads that as the bound; a
  design that wants the oldest writes that. What a design keeps is what it says
  it keeps.
- **A deployment that cannot hold what the design keeps is refused** on a
  bounded-memory target (`deployment.unbounded_list_state`), and one that keeps
  fewer values than a crossing produces is told so
  (`deployment.window_capacity`).

The **deployable window** is therefore the Phase-9a construction with the log
bounded and the count kept apart:

```text
count  @fast := 1 + delay 0 count
log    @fast := take cap (cons x (delay [] log))
logD   @slow := sync fast [] log
seen   @slow := sync fast 0 count
cursor @slow := delay 0 seen
window @slow := reverse (take (seen − cursor) logD)
```

With `cap ≥ required(fast → slow)` its `window` equals the unbounded
construction's at every tick — the production reading of FV
`bounded_buffer_agrees`, differentially tested across the reference evaluator,
the executable-IR interpreter and the generated core (corpus `bounded_buffer`
against `buffer`, 7 and 3 000 ticks); with `cap` smaller the oldest values of a
window are the ones missing (corpus `overflowing_buffer`). Nothing in the
language changed for this: no buffer, ring or capacity is a kernel notion, and
the unbounded form stays the formal construction it is.

## 6. Embedded targets: allocator, memory, failure

What a list-carrying core requires of the first targets. These are **deployment
recommendations** from the generated code's shape and the targets' documented
memory models; no target has run a BDL core yet (roadmap priority 1).

| Requirement                | Host (std)                                                                                                                                                       | RP2040 / RP2350                                                                                                           | ESP32-S3                                                           |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| allocator                  | the system allocator                                                                                                                                             | a `#[global_allocator]` the firmware declares (`embedded-alloc`'s linked-list or TLSF heap over a static arena)           | `esp-alloc` over a static arena, or the PSRAM/heap the HAL exposes |
| heap size                  | virtual memory                                                                                                                                                   | a fixed arena of the firmware's choosing; the report's `state_bytes_max` + `tick_bytes_max` is the core's floor           | the same; PSRAM on boards that have it                             |
| out-of-memory              | abort                                                                                                                                                            | `alloc::alloc::handle_alloc_error` → panic → the firmware's panic handler (reset or halt): a fault, never a dropped value | the same                                                           |
| ownership                  | the `State` record and one `Tick`                                                                                                                                | identical: the core owns its state and returns one `Tick` per step; nothing is shared, nothing is retained by the runtime | identical                                                          |
| allocation per tick        | one per list a due declaration builds, one per list cell written (`cons`, `take`, folds); zero for unchanged cells and for `length`/`head`/`take`/`==`/`<` reads | the same, on the arena; bounded by the report when the design is `bounded`                                                | the same                                                           |
| capacity growth at runtime | a `Vec` grows as its list does                                                                                                                                   | the same: a list grows to its bound and no further; an `unbounded` design is refused (`--bounded-memory`)                 | the same                                                           |

The generated core is `no_std` and `forbid(unsafe_code)`; the allocator, the
panic handler and the arena are the platform adapter's, outside the core.

### Readiness matrix

| Feature                                | Host  | RP2040 / RP2350                                                 | ESP32-S3                                        |
| -------------------------------------- | ----- | --------------------------------------------------------------- | ----------------------------------------------- |
| scalar-only design                     | READY | READY (allocation-free, `Copy`)                                 | READY                                           |
| finite static lists (`bounded`)        | READY | READY WITH ALLOCATOR                                            | READY WITH ALLOCATOR                            |
| dynamic lists (`input_bounded`)        | READY | REQUIRES CAPACITY VALIDATION (the adapter bounds its inputs)    | REQUIRES CAPACITY VALIDATION                    |
| list state (`delay`/`sync` of a list)  | READY | READY WITH ALLOCATOR when `bounded`; NOT READY when `unbounded` | the same                                        |
| cross-domain buffer (the window above) | READY | READY WITH ALLOCATOR under a validated `--period` schedule      | READY WITH ALLOCATOR under a validated schedule |

READY here means the generated core and its manifest carry what the adapter
needs; "READY WITH ALLOCATOR" means the adapter must declare one; nothing has
been flashed. `docs/project/roadmap.md` priority 1 is where this becomes
evidence.

## 7. Correspondence and claims

| Claim                                                                           | Strength                                                                                                                                                   |
| ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| a window never exceeds the required capacity; the periodic bound is one period  | formally proved in FV (`requiredCapacity_sufficient`, `periodic_capacity_sufficient`); production functions property-tested against them                   |
| under sufficient capacity the bounded window equals the unbounded one           | formally proved in FV (`bounded_buffer_agrees`) for the policies as functions; production differentially tested on the bounded design across three engines |
| every static bound holds at runtime                                             | tested implementation property (every corpus case, every tick)                                                                                             |
| `map`/`filter`/`append`/`sum` are linear in the generated core, `zip` quadratic | performance observation (`collections_cost_measurement`, docs/evidence/testing.md)                                                                         |
| the allocator and OOM behaviour on RP2040/ESP32-S3                              | deployment recommendation only                                                                                                                             |
