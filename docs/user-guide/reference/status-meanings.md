# Status meanings

Where a state is shown, what it looks like, and what it means. Orange
and dashed mean *not decided yet*; red means *wrong now*; the accent
colour means *selected* and nothing else.

## On the canvas

| Mark | Meaning |
| --- | --- |
| *declared* (inspector, status line) | a rule with no formula yet — a rule is not a node of the canvas |
| bar at a block's left edge, entry arrow, word *Source* | a Source: the environment provides the value; nothing is missing |
| a block with no mapping block beside it | no formula: a Source (or, in a system, a block a binding realises — its left socket is filled) |
| hollow socket labelled `?` on a mapping block | an open position in the formula: a block dropped there fills it |
| red mark at a mapping block's formula line | the formula does not check |
| hollow ring socket | the concept's value form is *decide later* |
| dashed sink, *no domain* | the output has no timing domain |
| dashed sink | the output has no driver |
| solid sink | driven |
| sink with the red word *contested* | more than one driver |
| sink with the red word *ill-formed* | the driver's concept or domain does not fit the output |
| a domain name at a node's right edge | *Updates in* that domain |
| instance socket hollow | the required port is open (unbound) |
| red mark on an instance's component name | the source no longer keeps the component's promise |
| a gate mark on a binding link | the value is carried across timing domains |

## In the status line

| Text | Meaning |
| --- | --- |
| *Saved* / *Edited* | whether the project on disk matches what you see |
| *N concepts · M mappings* | counts |
| *N sources* | relationships the environment provides (read nothing, no formula) — a count, not an open item |
| *N not yet defined* (orange) | relationships that read something, or back a port, and have no formula yet; Sources are not counted |
| *N definitions do not check* (red) | invalid formulas |
| *N definitions not added* | formula drafts not yet added — saved with the project, not the project's |
| *not causal* (red) | an instantaneous cycle |
| *reads across domains* (red) | a value reads another domain's value without `sync` |
| *outputs incomplete* (orange) | a required output undriven, an output open, or a connection that does not fit |
| *feasible on Arduino Nano* / *incomplete on …* / *not feasible on …* | the last Deploy verdict for the chosen board |
| *Compiler 0.1.0* / *Connecting to the compiler* / *Compiler not connected* | the connection to the compiler service |

## In a relationship's inspector

| Line under the formula | Meaning |
| --- | --- |
| *Checking…* | the compiler has not answered for this text yet |
| *Valid definition* | checks |
| *Tilt has no representation yet.* (orange) | open |
| a red message | the first finding; the rest are listed under it |
| *Not saved: …* | the last commit was refused; the text is kept |
| *unsaved* in the section header | a draft differs from the saved formula |

## In an output's inspector

| Line | Meaning |
| --- | --- |
| *No timing domain yet: not part of the design's commitment until one is chosen.* | open |
| *Undriven — the design is incomplete without a driver.* | required, no driver |
| *Undriven.* | optional, no driver |
| *Driven by brightness.* | driven |
| *Still driven by a, b* | contested |
| *The connection does not fit: see the driver's findings below.* | the driver's concept or domain differs |

## On an instance's port

| Text | Meaning |
| --- | --- |
| *Open: nothing supplies it yet.* | required port without a binding |
| *Bound to lampA.brightness* | bound; *Show Binding* selects the link |
| *Direct* / *Carried across timing domains* (binding inspector) | whether the binding transports |

## On the Simulate page

| Text | Meaning |
| --- | --- |
| *Checking the design…* | the analysis for this revision has not arrived |
| a sentence with *Show* | something blocks stepping (see [Incomplete design](../troubleshooting/incomplete-design.md)) |
| *tilt divided by zero.* / *… produced a value that is not a number.* / *… needs a value for this step.* | the tick failed at that relationship |
| an empty trace cell | the value's domain did not activate at that tick |

## On the Deploy page

| Text | Meaning |
| --- | --- |
| *Choose a board to see whether this design fits it.* | no target chosen |
| *Checking Arduino Nano…* | the analysis is running |
| *Feasible on …* | placed |
| *Fits … so far — the binding is not finished.* | an output without a device or a device without an output |
| *Not feasible on …* | one requirement could not be placed; the dead end follows |

## In Explain: the formal ladder

`declared → open → invalid → type-valid → temporally valid →
clock-consistent`. *Output-complete* is a property of the whole design,
*feasible* of the design on one board; neither is a step a relationship
climbs.
