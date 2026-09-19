# Reference edges, the word _rule_, one meaning of _produces_ (ADR-0034)

- Date: 2026-09-20
- Area: studio, protocol
- Affected: designers, protocol clients, developers
- Related: ADR-0034, ADR-0032, ADR-0019, ADR-0001

## What changed

- **The canvas draws what a formula references.** Beside the signature edges
  (socket to socket, in the concept's hue — what dragging edits) a _reference
  edge_ runs from the output socket of every relationship a definition names
  into the **formula line** of the relationship naming it: the kernel's
  `dependsOn`, neutral grey, thinner, under the signature edges, never a drop
  target. On `acOn = AirConditionerCtrl(TempSensor, ButtonInput)` the value is
  joined to the rule and both Sources; a rule nothing applies has no edge into
  any formula line. A relationship that names itself (memory through `delay`)
  draws no edge yet.
- **A rule wears the word _rule_** in its header when no state word takes the
  slot (_Source_ › _declared_ › port word › sink state › _required_ › _rule_);
  its input sockets are the shape. A value (no reads, a formula) carries no
  word. Assistive technology hears _name, rule, reads …_ or _name, value_, then
  _depends on …_.
- **The inspector's _Role_ row** says _Source_ / _Rule_ / _Value_ (a port-backed
  relationship keeps its port word) with one sentence; a defined relationship
  gets a _Depends on_ row and every relationship but a Source a _Named in_ row
  of name links.
- **_Produces_ means the signature everywhere** — a concept's input socket, the
  inspector's _Produced by_, the sheet's _Produces_. A value of a concept per
  tick is what _carries_ it: the Simulate probe for a concept is _Carried by_
  over its values and Sources, or _Nothing carries C yet: no value or Source
  produces it_ plus _R is a rule; a value whose formula applies it would carry
  C_; the probe for a rule is _A rule: it has no value of its own. A value whose
  formula applies it is what the simulator samples_, then _Applied in …_ or _No
  value applies it yet_. The old _No value declaration produces C_ and _A
  relationship: it is applied inside other relationships…_ are gone.
- **The creation sheet names the shape as the reads change**: _Reads nothing: a
  Source. The environment provides C once per activation; a formula added later
  makes it a computed value instead._ / _Reads X: a rule, a function to C. It
  has no value of its own — a value's formula applies it; dashed until its
  formula is added._

## Compatibility and migration

- Designers: nothing to do; every project means what it did. The canvas shows
  more edges; a picture that relied on a rule looking "complete" now shows
  whether anything applies it.
- Project files: nothing — no file carries an edge or a role.
- Protocol clients: protocol **0.18**, additive — `MappingAnalysis.references`
  (`repeated uint64`), the declarations a relationship's elaborated realization
  references, ascending, each once; empty for a declared relationship, a Source,
  a definition that does not elaborate, and in a `DefinitionDraftAnalysis`. A
  0.17 client ignores the field.
- Developers: `analysis_to_pb` fills `references` from `DependencyGraph::all`;
  Studio `LinkShape.reference`, `NodeShape.rule` / `dependsOn` / `formulaEntry`,
  `buildScene(refs:)`, `AppState.refsOf` / `referrersOf`; l10n keys `ruleWord`,
  `roleRule`, `roleValue`, `ruleExplanation`, `valueExplanation`, `dependsOn`,
  `namedIn`, `carriedBy`, `noValueCarries`, `ruleProducesNoValue`,
  `aRuleNoValueOfItsOwn`, `appliedIn`, `noValueAppliesItYet`,
  `sheetSourceShape`, `sheetRuleShape`, `ruleNodeSemantics`,
  `valueNodeSemantics`, `dependsOnList` (four keys removed).

## Evidence

`crates/bdl-protocol/src/convert.rs` `analysis_carries_the_realization_refs`;
`apps/studio/test/reference_edges_test.dart` (geometry, inspector, Simulate
probe, creation sheet); `apps/studio/test/source_role_test.dart` (the Source
sheet's sentence).
