# Inspector

The inspector (right) shows the selected object as a form of sections,
in designer vocabulary. Every field is an edit the compiler accepts;
nothing in the inspector is decorative. When nothing is selected it says
what can be selected.

Below the sections, **Last change** states the consequence of your most
recent edit as a sentence about *other* objects — *Nothing else needs
rechecking.* or *This change affects dimByTilt, warmPulse; they will be
checked again.* — and **Explain**, collapsed, holds the formal detail.

## Concept

| Section | Fields | Notes |
|---|---|---|
| **Meaning** | Name, Meaning | |
| **Value** | Quantity / On / off / Count / Decide later; Unit (a list of quantity kinds, the symbol in its own column) | choosing a value form the first time rechecks nothing; changing a chosen one says which relationships it rechecks |
| **Relationships** | *Produced by*, *Used by* | names are links that select the relationship |
| **Delete …** | | disabled while used; the users are named under the button |

## Relationship

| Section | Fields | Notes |
|---|---|---|
| **Meaning** | Name, Meaning | |
| **Reads** | chips with the concepts' glyphs, removable; a pop-up to add | an edit; dependents are rechecked |
| **Produces** | a pop-up with the glyph | an edit |
| **Relationship** | the formula field and its verdict line; *Add definition* / *Save definition* / *Revert* / *Detach definition*; findings under the field | see [Formula editor](formula-editor.md) |
| **Timing** | *Updates in* — a domain, or *Any timing domain*; timing findings | see [Timing](../concepts/timing.md) |
| **Drives** | the output this value drives, or none; connection findings | only a relationship without inputs can drive |
| **Fixes** | actions the tool offers for findings on this relationship: a button when ready, a pop-up when it needs a choice, the reason when blocked | applied as ordinary, undoable edits |
| **Delete …** | | |

In a system project a relationship can also show *In group …* with a
*Show Group* link, *Takes its value from …* with *Show Binding* when it is
bound to a port (with *Disconnect it to define the relationship
yourself.*), and — in a component's source — the port it backs
(*requires* / *provides* / *parameter*). Ports are declared from the
component's own inspector.

## Physical output

| Section | Fields | Notes |
|---|---|---|
| **Meaning** | Name, Meaning | |
| **Output** | *Accepts*, *Updates in*, *Required* | |
| **Driver** | the state (*Driven by brightness.*, *Undriven — the design is incomplete without a driver.*, …); claimants with *disconnect*; a *Connect* pop-up of the design's relationships (those *with inputs* marked) | see [Physical outputs](../concepts/physical-outputs.md) |
| **Fixes** | *Detach … from …*, *Create upstream combination mapping*, *Connect a driver to …* | |
| **Delete …** | | |

## Timing domain

Timing domains have no inspector page: rename them in place in the
sidebar and delete them there while unused. Their meaning is in the
relationships and outputs that *update in* them.

## In a system project

Behaviors, components, instances and bindings have inspectors of their
own; [System projects](system-projects.md) describes them.

## Explain

The disclosure at the end of every inspector. It shows the object's
identity number, the formal interface and, for a relationship with a
formula, the inferred type and the Core term; the status word; the
diagnostic codes and technical text behind each finding; the revision;
and the kind of the last change (*refinement* / *edit*) with what it
invalidated. This is the one place in Studio where compiler vocabulary
appears. Nothing above it depends on it.

## Related

[Workspace](workspace.md) · [Formula editor](formula-editor.md) ·
[Status meanings](../reference/status-meanings.md)
