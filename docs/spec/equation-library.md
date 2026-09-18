---
kind: specification
area: language
status: current
---

# Equation library and the data core

What a formula may compute over beyond a single number: finite collections,
grouped values, optional values, and the reusable equations over them — `min`,
`clamp`, `any`, `all`, `map`, `filter`, `x in [a, b]`. This page fixes the
production boundary of the equation language: the data core, the equations
(their names, shapes and what their values must be able to do), how a use is
resolved, the ordering policy, and the diagnostics. It is the production
counterpart of BDL_FV Phases 9a, 9b and 9c (`fad79d9`, `ee2862e`, `6a8ee96`;
`POLYMORPHIC_EQUATION_LANGUAGE_NOTE.md`), consumed as written: nothing here is
richer than the formal development, and the formal minimality findings are
recommendations, not theorems about this implementation
(`docs/project/formal-correspondence.md`).

```text
formula   min(bIn, bMin)   any(temps, t => t < 30 K)   mode in [1, 2]
   │ bdl-elab: match the scheme against the arguments' closed types, check {Data, Eq, Ord}
   ▼
Core      app (app minF a) b — a closed term of the kernel, inlined (bdl-equations)
   │ bdl-check re-types it like any term; bdl-lower inlines the rule, ExecExpr::Fold
   ▼
generated Rust   list::fold(xs, init, |x, acc| …)      (runtime feature `collections`)
```

## 1. The data core

Every value a design carries is one of:

| kind           | kernel   | value form in `concept C : …`         | in a formula                          |
| -------------- | -------- | ------------------------------------- | ------------------------------------- |
| truth value    | `bool`   | `Bool`                                | `true`, `false`, `&&`, `!`            |
| quantity       | `q d`    | `Scalar`, `Angle`, `Temperature`, …   | `90 deg`, `+ - * /`, `<`              |
| count          | `nat`    | `Count`                               | `3` (a whole number)                  |
| concept value  | `sem s`  | (a concept's own name in a signature) | an input, a relationship's value      |
| optional value | `opt τ`  | `Option<R>`                           | `Some(e)`, `None`, `match`            |
| collection     | `list τ` | `List<R>`                             | `[a, b, c]`, `[]`                     |
| grouped value  | `τ × σ`  | `Pair<R₁, R₂>`                        | `(a, b)`; `(a, b, c)` = `(a, (b, c))` |

A concept's value form (`R`) is semantic-free data: the forms above minus
concept values. Every kind above is _data_: it may be remembered (`delay`),
transported (`sync`) and compared for equality. A rule (`x => …`) is not data
and not a value: it is given to an equation and inlined there.

The kernel's collection operators (`bdl-ir::Prim`): `nil`, `cons`, `length`,
`take`, `drop`, `reverse`, `head`, `toList`; the grouped-value operators `pair`,
`fst`, `snd`; and one term former, the **list recursor**
`fold f z [x₁, …, xₙ] = f x₁ (… (f xₙ z))`. `fold` is the only construct that
applies a function value during evaluation, and it is not recursion: a finite
list folds in exactly one step per element (FV `fold_total`). Nothing else — no
`map`, `filter`, `any` primitive, no `Set`, no record, no `while` — enters the
kernel; the rest is the library below.

Production deviations from the kernel, recorded here and in
`docs/architecture/ir.md`: `length` yields, and `take`/`drop` read, a
dimensionless `f64` (the kernel's `Nat`; ADR-0011) — a count is read as a whole
number towards zero, never below zero (`take 2.7 xs` is `take 2 xs`); `eq` at a
non-data type is refused by the checker (`type.equality_not_data`) rather than
made unwritable by a proof field.

## 2. Equality and order (Phase 9c)

- **Equality** is structural at every data kind: `a == b` on truth values,
  quantities of one dimension (exact `f64` equality), counts, optional values,
  collections (same length, same elements in order), grouped values, and concept
  values **of the same concept**. `Brightness == Opacity` is refused —
  `semantic.concept_mismatch` — whatever the representations; a concept's
  identity is never exchanged for another's. Rules cannot be compared
  (`type.equality_not_data`).
- **Order** is a property of _magnitudes_, not of data. `<`, `<=`, `>`, `>=`,
  `min`, `max`, `clamp`, `inRange`, `inInterval` are defined for
  - a quantity (of one dimension), and
  - a concept the designer **declared ordered** —
    `ordered concept Brightness : Scalar` in text, the _Order_ checkbox in
    Studio — represented by a quantity; two such values compare through their
    representation and `min`/`max`/`clamp` return the original concept value.

  Nothing else has an order: `Mode < Mode` (a numeric encoding is not a
  magnitude), `Pair < Pair`, `List < List`, `None < Some(x)`, `Bool < Bool` are
  refused with `semantic.no_order`, whose message says what has no order and
  what to do instead (declare the concept ordered when its values are
  magnitudes; compare a part; compare the length; take the optional value apart;
  choose with a rule). Declaring order is never inferred from the value form;
  `Concept::ordered` is stored in the project (`docs/spec/project-format.md`)
  and is an _edit_, not a refinement: it reopens every relationship that
  mentions the concept.

- **The comparator escape hatch.** `minBy(a, b, (x, y) => …)` and `maxBy` choose
  by a rule the designer writes and need no declaration (FV
  `minBy_recovers_min`: the comparator form loses nothing).

- **Concept values beside plain values.** Two concept values compare _as
  concepts_ (same concept, or `semantic.concept_mismatch`). A concept value
  beside a plain value of its own representation — `tilt < 10 deg`,
  `min(brightness, 0.5)`, `mode in [1, 2]` — is observed (`rep`) and compared as
  that representation, as arithmetic has always done (ADR-0013). A concept whose
  value form is a collection or a grouped value is observed deeply where an
  equation needs the collection (`all(readings, …)` over
  `Readings : List<Temperature>`), and a formula's result is observed deeply
  where the produced concept's form needs it (`(t, h)` for
  `Climate : Pair<Temperature, Scalar>`). The mixed case is the one path that
  does not consult the order declaration — ISS-0012 records it.

## 3. The equations

Each equation is a closed kernel term indexed by the kinds it is used at — a
_definitional family_ (`crates/bdl-equations`, one builder per row, the
combinator of `BDL/Surface/Stdlib.lean` in the designer's argument order). A use
site is resolved by **first-order matching** of the scheme against the
arguments' closed kinds, in argument order (FV `matchTy_sound`,
`matchTy_complete`): no unification of open types, no generalisation, no search,
and the kernel never sees a type variable. _Needs_ is the weakest capability the
scheme states: `Eq` for membership, `Ord` for order, nothing otherwise.

| equation                                                                                          | meaning                                                    | needs |
| ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | ----- |
| `id(x)`                                                                                           | the value itself                                           | —     |
| `const(x, y)`                                                                                     | always `x`                                                 | —     |
| `swap(pair)`                                                                                      | the parts exchanged                                        | —     |
| `min(a, b)`, `max(a, b)`                                                                          | the smaller / larger of two values of one ordered kind     | Ord   |
| `clamp(x, low, high)`                                                                             | `x` held between the bounds: `max(low, min(x, high))`      | Ord   |
| `inRange(x, low, high)`                                                                           | `low ≤ x ≤ high`                                           | Ord   |
| `inInterval(x, interval)`                                                                         | `inRange` with the bounds as a grouped value `(low, high)` | Ord   |
| `minBy(a, b, less)`, `maxBy`                                                                      | the value a rule ranks lower / higher                      | —     |
| `foldr(collection, start, step)`                                                                  | combine from the last element to the first                 | —     |
| `any(collection, condition)`                                                                      | some element satisfies the condition                       | —     |
| `all(collection, condition)`                                                                      | every element does (true of `[]`)                          | —     |
| `contains(x, collection)`, `oneOf`                                                                | membership; `x in [a, b]` is the same equation             | Eq    |
| `map(collection, rule)`                                                                           | the rule applied to every element                          | —     |
| `filter(collection, condition)`                                                                   | the elements that satisfy it, in order                     | —     |
| `append(first, second)`                                                                           | one collection followed by another                         | —     |
| `sum(quantities)`                                                                                 | the total, in the elements' dimension                      | —     |
| `zip(first, second)`                                                                              | paired element by element, as long as the shorter          | —     |
| `optElim(optional, default, rule)`                                                                | the rule on the value when present, else the default       | —     |
| `mapOpt(optional, rule)`                                                                          | the rule inside an optional value                          | —     |
| `getOrElse(optional, default)`                                                                    | the value when present, else the default                   | —     |
| `length`, `head`, `reverse`, `take(count, c)`, `drop(count, c)`, `cons(x, c)`, `toList(optional)` | the kernel's list operators                                | —     |
| `pair(a, b)`, `first(p)`, `second(p)`                                                             | the kernel's grouped-value operators; `(a, b)` is `pair`   | —     |

`sum` is dimension-generic: a dimension variable in the scheme is bound by the
elements' dimension and the result keeps it. A rule argument (`x => …`,
`(x, acc) => …`) is elaborated once the arguments before it have fixed what it
reads — the collection comes first in every shape above — and its body's kind
binds the remaining variables (`map` returns a collection of what the rule
produces). A rule may shadow an outer name; it may not contain `delay`/`sync`
(`formula.temporal.under_binder`) and is never a value on its own
(`formula.rule.not_a_value`).

A relationship of the design named like an equation wins: the call resolves to
the relationship, and the equation is not offered.

## 4. Sets, intervals, records, quantifiers, enums

Following FV D-95/D-96: a finite set is membership in a collection literal
(`x in [a, b, c]`; duplicates change nothing); an interval is a grouped value
`(low, high)` with `inInterval`; a record would be a right-nested grouped value
(no record syntax exists); finite quantification is `all`/`any` over a
collection (no `forall`/`exists` syntax exists — ISS-0001 and ISS-0010 list the
surface forms still open); user enums are unchanged (ISS-0005; the formal
recommendation is a tag paired with an optional payload). No `Set`, interval,
record or sum type exists in the kernel.

## 5. Diagnostics

| code                                    | when                                                               | message shape                                                               |
| --------------------------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| `formula.equation.arity`                | wrong number of arguments                                          | _min takes 2 values (a, b), but 3 are given here._ fix: _Write min(a, b)._  |
| `formula.equation.argument`             | an argument does not fit the shape the earlier ones fixed          | _any expects a collection for `collection`, but this is a quantity._        |
| `semantic.concept_mismatch`             | two concepts where one kind is needed, or `==`/`<` across concepts | _Brightness and Opacity are different concepts._                            |
| `dimension.mismatch`                    | two dimensions where one is needed                                 | _min mixes values with different physical dimensions: a length and a time._ |
| `semantic.no_order`                     | order asked of something without one                               | _Mode values can be compared for equality, but they have no default order._ |
| `type.equality_not_data`                | equality on rules                                                  | _Rules cannot be compared for equality._                                    |
| `formula.rule.expected` / `.unexpected` | a rule missing where needed, or given where a value is read        | _any needs a rule for `condition` here, written `x => …`._                  |
| `formula.rule.arity` / `.undetermined`  | wrong parameter count; the rule's inputs' kinds cannot be told yet | _The kind of `x` cannot be told here._                                      |
| `formula.rule.not_a_value`              | a rule outside an equation                                         | _A rule (`x => …`) is not a value on its own._                              |
| `formula.option.undetermined`           | `[]` or `None` whose kind nothing fixes                            | (as before)                                                                 |
| `type.fold_mismatch`                    | the checker's own fold rule (never reached from the surface)       | technical                                                                   |

Every message speaks of values, collections, grouped values, rules and concepts;
_scheme_, _fold_, _product_, _Data_ and _type variable_ appear only in the
technical line and the explanation layer.

## 6. Execution

The reference evaluator and the executable IR keep collections and grouped
values as structured values; `fold` is finite iteration from the last element,
never a stack of frames; a collection is a shared, immutable list so that the
library's `map`, `filter` and `append` are linear. Generated Rust represents a
collection as a `Vec` behind the runtime's `collections` feature (an allocator
on the target — ADR-0024; the manifest says `requires_allocator`) and a grouped
value as a tuple; `docs/architecture/codegen-rust.md` has the correspondence and
the known cost of `filter` there (ISS-0013). Collections and grouped values in
`delay`/`sync` follow the existing rules unchanged — a transported collection is
the source's collection at its last activation strictly before now — which is
what lets the Phase-9a lossless buffer be written as five ordinary declarations
(`docs/spec/runtime-semantics.md`).

## 7. What is deliberately absent

Type variables in Core IR; System F, higher-rank types, let-generalisation,
principal-type inference; user-defined type classes, `class`/`instance`/`trait`,
dictionaries; GADTs, dependent, row and existential types; general recursion,
fixpoints, loops; `Set`, interval and record types; general logical quantifiers;
new temporal primitives. Each is either derivable from the basis above or was
rejected in the formal development (`POLYMORPHIC_EQUATION_LANGUAGE_NOTE.md` §6).
