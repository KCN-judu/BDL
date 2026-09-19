# A rule nothing applies is stated, and the value that applies it is offered

- Date: 2026-09-20
- Area: compiler, ide, studio, protocol
- Affected: designers, protocol clients, developers
- Related: ADR-0029 (the unit domain), ADR-0032 (the Source role)

## What changed

- **A new finding, `reactive.rule_unapplied`** (info, never an error), on every
  rule — a relationship with inputs — that no definition references and no
  output is driven through: _AirConditionerCtrl is a rule nothing applies yet._
  with the explanation naming the call when each read concept has exactly one
  producing value (_a value that applies it —
  `AirConditionerCtrl(TempSensor, ButtonInput)` — is what the simulator and an
  output can read_). It states what the simulator cannot show: a rule's value at
  a tick is a function, so it has no column; only a value that applies it does.
- **A new action, `rule.apply:<rule>`** (_Add a value that applies X_): creates
  `<lowerCamel rule> : () -> <output> = Rule(arg, …)` — ready when every read
  concept has one producing value; a choice (one option per argument
  combination, in declaration order) when one has several; blocked, with the
  reason, when one has none or the arguments update in two timing domains. The
  tool never guesses.
- **`MappingAnalysis.applied_by`** (in the compiler's analysis): who references
  each declaration, from the dependency graph; on the wire the same fact is the
  inverse of `MappingAnalysis.references` (0.18, ADR-0034), which Studio inverts
  — never a reading of formulas.
- **Studio**: the Simulate page's readiness area lists the note below the
  blockers (hollow dot: Step stays enabled) with the fix and a _Show_ link; the
  probe for a rule says it has no value per tick and links its appliers or
  offers the fix; the inspector's _Relationship_ section shows the finding with
  its fix beside it; the canvas draws an unapplied rule with a hollow output
  socket and, once defined, the header word _not applied_. A Fix chosen away
  from the inspector (`SemanticActionChosen`) selects the object, asks for its
  actions and applies the one of that kind when it arrives ready.
- **Studio, a Source with no value**: an on/off Source not yet given a value is
  a third state — the off | on control drawn empty with a dashed outline and _no
  value yet_ beside it — never a switch that reads _off_; one click on a segment
  gives that value; the number fields' hint says _no value yet_. Nothing is
  defaulted.

## Compatibility and migration

- Designers: nothing to do. A design with an unapplied rule was legal and is
  legal; it now says so and offers the value.
- Project files: nothing.
- Protocol clients: protocol **0.19**, additive — `CreateMapping.definition?`
  and `clock_id?` (a mapping declared and defined in one step, a refinement,
  `created_mapping` as before); a new diagnostic code and action id, both
  strings. An older client works unchanged.
- Developers:
  `EditOp::CreateMapping { definition: Option<Definition>, clock: Option<ClockId> }`
  (serde defaults; every struct literal gains the two fields);
  `bdl_compiler::MappingAnalysis.applied_by: BTreeSet<DeclId>` (not on the
  wire); `bdl_ide::actions` `apply_rule`; Studio `simulationNotes`,
  `appliersOf`, `isUnappliedRule`, `kApplyRuleActionKind`, `OfferedFix`,
  `FixItem`, `MacSegmented.undecided`, `DashedOutline`.

## Evidence

`crates/bdl-compiler/src/lib.rs`
(`a_rule_nothing_applies_is_stated_until_a_value_applies_it`),
`crates/bdl-ide/tests/rule_apply.rs`, `crates/bdl-daemon/tests/stdio_e2e.rs`,
`apps/studio/test/unapplied_rule_test.dart` (incl. the acceptance scenario
against bdld), `apps/studio/test/simulation_test.dart` (the three-state
control).
