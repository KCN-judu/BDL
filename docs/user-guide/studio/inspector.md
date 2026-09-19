# Inspector

The inspector (right) shows the selected object as a form of sections, in
designer vocabulary. Every field is an edit the compiler accepts; nothing in the
inspector is decorative. When nothing is selected it says what can be selected.

Below the sections, **Last change** states the consequence of your most recent
edit as a sentence about _other_ objects — _Nothing else needs rechecking._ or
_This change affects dimByTilt, warmPulse; they will be checked again._ — and
**Explain**, collapsed, holds the formal detail.

## Concept

| Section           | Fields                                                                                                    | Notes                                                                                                             |
| ----------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Meaning**       | Name, Meaning                                                                                             |                                                                                                                   |
| **Value**         | Quantity / On / off / Count / Decide later; Unit (a list of quantity kinds, the symbol in its own column) | choosing a value form the first time rechecks nothing; changing a chosen one says which relationships it rechecks |
| **Relationships** | _Produced by_, _Used by_                                                                                  | names are links that select the relationship                                                                      |
| **Delete …**      |                                                                                                           | disabled while used; the users are named under the button                                                         |

## Relationship

| Section          | Fields                                                                                                                                                                                                                                     | Notes                                        |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------- |
| **Meaning**      | Name, Meaning, _Role_ — _Source_, _Relationship_, or the port it backs; a Source adds one sentence                                                                                                                                         | the role is read off the design, never set   |
| **Reads**        | chips with the concepts' glyphs, removable; a pop-up to add                                                                                                                                                                                | an edit; dependents are rechecked            |
| **Produces**     | a pop-up with the glyph — titled **Provides** for a Source                                                                                                                                                                                 | an edit                                      |
| **Relationship** | the formula field and its verdict line; _Add definition_ / _Save definition_ / _Revert_ / _Detach definition_; findings under the field; for a Source, _Realization: Provided by the environment; no device is bound yet._ above the field | see [Formula editor](formula-editor.md)      |
| **Timing**       | _Updates in_ — a domain, or _Any timing domain_; timing findings                                                                                                                                                                           | see [Timing](../concepts/timing.md)          |
| **Drives**       | the output this value drives, or none; connection findings                                                                                                                                                                                 | only a relationship without inputs can drive |
| **Fixes**        | actions the tool offers for findings on this relationship: a button when ready, a pop-up when it needs a choice, the reason when blocked                                                                                                   | applied as ordinary, undoable edits          |
| **Delete …**     |                                                                                                                                                                                                                                            |                                              |

A relationship can also show _In group …_ with a _Show Group_ link, _Takes its
value from …_ with _Show Binding_ when it is bound to a port (with _Disconnect
it to define the relationship yourself._), and — in a component's source — the
port it backs (_requires_ / _provides_ / _parameter_). Ports are declared from
the component's own inspector.

A **Source** — a relationship that reads nothing and has no formula — is
inspected the same way. Its Role row says _Source_; its output section is
_Provides_; its Relationship section says the environment provides the value and
no device is bound yet, and keeps the formula field: add a formula and the same
relationship is computed inside the design instead. Nothing here says _sensor_
or names a part — which device provides the value is decided on the
[Deploy page](deploy.md), and today no device kind provides one.

## Physical output

| Section      | Fields                                                                                                                                                                                                      | Notes                                                   |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| **Meaning**  | Name, Meaning                                                                                                                                                                                               |                                                         |
| **Output**   | _Accepts_, _Updates in_, _Required_                                                                                                                                                                         |                                                         |
| **Driver**   | the state (_Driven by brightness._, _Undriven — the design is incomplete without a driver._, …); claimants with _disconnect_; a _Connect_ pop-up of the design's relationships (those _with inputs_ marked) | see [Physical outputs](../concepts/physical-outputs.md) |
| **Fixes**    | _Detach … from …_, _Create upstream combination mapping_, _Connect a driver to …_                                                                                                                           |                                                         |
| **Delete …** |                                                                                                                                                                                                             |                                                         |

## Timing domain

Timing domains have no inspector page: rename them in place in the sidebar and
delete them there while unused. Their meaning is in the relationships and
outputs that _update in_ them.

## Behaviors, components, instances, bindings

They have inspectors of their own; [System projects](system-projects.md)
describes them.

## Explain

The disclosure at the end of every inspector. It shows the object's identity
number, the formal interface and, for a relationship with a formula, the
inferred type and the Core term; the status word; the diagnostic codes and
technical text behind each finding; the revision; and the kind of the last
change (_refinement_ / _edit_) with what it invalidated. This is the one place
in Studio where compiler vocabulary appears. Nothing above it depends on it.

## Related

[Workspace](workspace.md) · [Formula editor](formula-editor.md) ·
[Status meanings](../reference/status-meanings.md)
