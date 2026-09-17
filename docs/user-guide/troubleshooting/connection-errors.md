# Connections

Links on the canvas mean one of three things — *this relationship reads
that concept*, *this value drives that output*, *this port supplies that
port* — and each has rules. Some rules are enforced while you drag (the
link will not land); others are recorded and then reported, because the
tool would rather hold your intent and name the problem than refuse.

## The link will not land (forbidden cursor, no halo)

**What it means.** The two sockets cannot be joined: they carry different
concepts, or they are the same side (two inputs), or the same node — or
you are dragging onto a collapsed behavior's socket, which is a picture,
not a port.

**Why.** Socket colour is the concept. A relationship reads a concept; it
cannot read *Opacity* through a socket meant for *Brightness*, whatever
the numbers. For a behavior box, the link resolves to the member the
socket stands for; a small chooser appears if several qualify.

**What to do.** Drop on a socket of the same colour. If the concepts
really should be one, that is a design change: make the relationship
read the other concept, or bind the concept to the one you meant. In a
system, a private concept of an instance is a different concept from the
system's; share it in the component if it should be the same.

## *Light Output expects Brightness, but dimByTilt produces Tilt → Brightness.* (under *Drives* / *Driver*)

**What it means.** The connected relationship does not produce exactly
what the output accepts — a rule with inputs was connected, or a value of
another concept.

**Why.** A connection to an output converts nothing. Only a *value* — a
relationship without inputs — can be *the* brightness at a tick.

**What to do.** Connect the value that applies the rule (`brightness =
dimByTilt(tilt)`), or a value of the right concept; disconnect the wrong
one with *disconnect* in the output's *Driver* section. Code:
`output.type_mismatch`.

## *Light Output updates in the interaction domain, but its driver shown updates in display.*

**Why.** A connection transports nothing. The device refreshes at its
domain's rhythm and must read a value of that rhythm.

**What to do.** Bring the value into the output's domain first — a value
in *interaction* that reads the other through `sync` — then connect that.
Or set the output's *Updates in* to the driver's domain, if that is what
the product should do. Code: `output.clock_mismatch`.

## The sink says *contested*; *Light Output already has a final target.*; *brightness, indicator all connect to Light Output.*

**What it means.** Two or more values drive one output.

**Why.** One output, one driver. Two values for one light is a
contradiction, and BDL does not resolve it with a priority or a last
writer.

**What to do.** *Fixes* offers *Detach … from …* for each claimant and
*Create upstream combination mapping*. Combine the competing values in
one relationship — `if held then a else b` — and connect that. Code:
`output.multiple_drivers`.

## *Only a relationship without inputs can drive an output: connect the relationship that combines the sources.*

**What to do.** The rule cannot drive; the value that applies it can.
Same rule as above, stated at the source.

## *Replace the connection?* (a sheet)

**What it means.** You dropped a link on a required port or a bound
relationship that already has a source.

**What to do.** *Disconnect and Connect* to swap; *Cancel* to keep the
existing one. Nothing is ever swapped silently.

## *Carry across timing domains* (a sheet asking *Starts at*)

**What it means.** The two ends of the binding update in different
domains.

**What to do.** Enter the value the destination should see before the
source has ever produced one, in the destination's units, and confirm —
the binding carries the value across. Or cancel and give the destination
the source's domain. See [Carrying a value across timing domains](../workflows/cross-domain-transport.md).
Code when reported instead of asked: `system.binding_needs_transport`.

## *second.tiltValue expects Tilt, but lampA.brightness provides Brightness.*

**Why.** A binding converts nothing: the provided port must promise
exactly what the required port expects — same concept, input for input.

**What to do.** Bind a port or value of the right concept. Code:
`system.binding_type_mismatch`.

## *Takes its value from adaptiveLamp.brightness* and the formula field is gone

**What it means.** The relationship is bound to a port; the binding *is*
its definition.

**What to do.** If you want to write its formula yourself, *Disconnect*
the binding first (the inspector says so).

## A banner: *concept Tilt is still used by 2 mapping(s)* / *timing domain … is still used …* / *output … is still driven by …*

**What it means.** You tried to delete something other objects depend on.

**What to do.** Disconnect or delete the users first; the inspector names
them under the delete button.

## A banner about a port in use, an instance in use, or a component that cannot stand in

| Banner code | Meaning | What to do |
|---|---|---|
| `system_edit.port_in_use` | the port you tried to retire is bound on an instance | disconnect those bindings first |
| `system_edit.instance_in_use` | the instance you tried to delete has bound ports | *Disconnect its ports first; the component stays.* |
| `system_edit.not_substitutable` | *Replace with* — the instance uses a port the other component lacks or promises differently | compare the two components' *Ports*; restore the port or choose another component |
| `group_edit.already_grouped` | the relationship is already in another behavior | remove it there first |

## The component's name on an instance carries a red mark; *AdaptiveLamp's port brightness promises to update in main, but brightness updates in aux.*

**What it means.** The component's source no longer keeps its promise:
a port's relationship has a different domain, is missing, or a required
port has been given a definition.

**Why.** A consumer bound to the port relies on what the port advertises.

**What to do.** Fix the source (open it with *Edit Source*), or change
the port's contract to match. Codes: `component.port_clock_mismatch`,
`component.port_declaration_missing`, `component.required_port_realized`,
`component.port_signature_mismatch`.

## Related

[Physical outputs](../concepts/physical-outputs.md) ·
[Behavior systems](../concepts/behavior-systems.md) ·
[System projects](../studio/system-projects.md)
