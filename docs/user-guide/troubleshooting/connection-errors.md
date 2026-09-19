# Connections

Links on the canvas mean one of three things — _this relationship reads that
concept_, _this value drives that output_, _this port supplies that port_ — and
each has rules. Some rules are enforced while you drag (the link will not land);
others are recorded and then reported, because the tool would rather hold your
intent and name the problem than refuse.

## The link will not land (forbidden cursor, no halo)

**What it means.** The two sockets cannot be joined: they carry different
concepts, or they are the same side (two inputs), or the same node — or you are
dragging onto a collapsed behavior's socket, which is a picture, not a port.

**Why.** Socket colour is the concept. A relationship reads a concept; it cannot
read _Opacity_ through a socket meant for _Brightness_, whatever the numbers.
For a behavior box, the link resolves to the member the socket stands for; a
small chooser appears if several qualify.

**What to do.** Drop on a socket of the same colour. If the concepts really
should be one, that is a design change: make the relationship read the other
concept, or bind the concept to the one you meant. In a system, a private
concept of an instance is a different concept from the system's; share it in the
component if it should be the same.

## _light expects Brightness, but dimByTilt produces Tilt → Brightness._ (under _Drives_ / _Driver_)

**What it means.** The connected relationship does not produce exactly what the
output accepts — a rule with inputs was connected, or a value of another
concept.

**Why.** A connection to an output converts nothing. Only a _value_ — a
relationship without inputs — can be _the_ brightness at a tick.

**What to do.** Connect the value that applies the rule
(`brightness = dimByTilt(tilt)`), or a value of the right concept; disconnect
the wrong one with _disconnect_ in the output's _Driver_ section. Code:
`output.type_mismatch`.

## _light updates in the interaction domain, but its driver shown updates in display._

**Why.** A connection transports nothing. The device refreshes at its domain's
rhythm and must read a value of that rhythm.

**What to do.** Bring the value into the output's domain first — a value in
_interaction_ that reads the other through `sync` — then connect that. Or set
the output's _Updates in_ to the driver's domain, if that is what the product
should do. Code: `output.clock_mismatch`.

## The sink says _contested_; _light already has a final target._; _brightness, indicator all connect to light._

**What it means.** Two or more values drive one output.

**Why.** One output, one driver. Two values for one light is a contradiction,
and BDL does not resolve it with a priority or a last writer.

**What to do.** _Fixes_ offers _Detach … from …_ for each claimant and _Create
upstream combination mapping_. Combine the competing values in one relationship
— `if held then a else b` — and connect that. Code: `output.multiple_drivers`.

## _A rule cannot drive an output — connect the value that applies this rule._ (under _Drives_)

**What it means.** Not a finding: the selected relationship reads something, so
its _Drives_ section offers no output. The same rule as above, stated at the
source before anything is connected.

**What to do.** When exactly one value of the design applies the rule, the
section names it — _Show_ selects it; connect the output from there. Otherwise
write the value (`brightness = dimByTilt(tilt)`) and connect that. An output's
_Connect_ pop-up likewise lists only values of the concept it accepts; when
there is none, it says so in one sentence.

## _Replace the connection?_ (a sheet)

**What it means.** You dropped a link on a required port or a bound relationship
that already has a source.

**What to do.** _Disconnect and Connect_ to swap; _Cancel_ to keep the existing
one. Nothing is ever swapped silently.

## _Carry across timing domains_ (a sheet asking _Starts at_)

**What it means.** The two ends of the binding update in different domains.

**What to do.** Enter the value the destination should see before the source has
ever produced one, in the destination's units, and confirm — the binding carries
the value across. Or cancel and give the destination the source's domain. See
[Carrying a value across timing domains](../workflows/cross-domain-transport.md).
Code when reported instead of asked: `system.binding_needs_transport`.

## _second.tiltValue expects Tilt, but lampA.brightness provides Brightness._

**Why.** A binding converts nothing: the provided port must promise exactly what
the required port expects — same concept, input for input.

**What to do.** Bind a port or value of the right concept. Code:
`system.binding_type_mismatch`.

## _Takes its value from adaptiveLamp.brightness_ and the formula field is gone

**What it means.** The relationship is bound to a port; the binding _is_ its
definition.

**What to do.** If you want to write its formula yourself, _Disconnect_ the
binding first (the inspector says so).

## A banner: _concept Tilt is still used by 2 mapping(s)_ / _timing domain … is still used …_ / _output … is still driven by …_

**What it means.** You tried to delete something other objects depend on.

**What to do.** Disconnect or delete the users first; the inspector names them
under the delete button.

## A banner about a port in use, an instance in use, or a component that cannot stand in

| Banner code                     | Meaning                                                                                     | What to do                                                                        |
| ------------------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `system_edit.port_in_use`       | the port you tried to retire is bound on an instance                                        | disconnect those bindings first                                                   |
| `system_edit.instance_in_use`   | the instance you tried to delete has bound ports                                            | _Disconnect its ports first; the component stays._                                |
| `system_edit.not_substitutable` | _Replace with_ — the instance uses a port the other component lacks or promises differently | compare the two components' _Ports_; restore the port or choose another component |
| `group_edit.already_grouped`    | the relationship is already in another behavior                                             | remove it there first                                                             |

## The component's name on an instance carries a red mark; _AdaptiveLamp's port brightness promises to update in main, but brightness updates in aux._

**What it means.** The component's source no longer keeps its promise: a port's
relationship has a different domain, is missing, or a required port has been
given a definition.

**Why.** A consumer bound to the port relies on what the port advertises.

**What to do.** Fix the source (open it with _Edit Source_), or change the
port's contract to match. Codes: `component.port_clock_mismatch`,
`component.port_declaration_missing`, `component.required_port_realized`,
`component.port_signature_mismatch`.

## Related

[Physical outputs](../concepts/physical-outputs.md) ·
[Behavior systems](../concepts/behavior-systems.md) ·
[System projects](../studio/system-projects.md)
