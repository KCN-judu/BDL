# Troubleshooting — start here

Behavior Designer reports problems as sentences about *your* objects, in
the place they belong: under the formula field, under *Timing* or
*Drives* in the inspector, on the Simulate page's readiness list, on the
Deploy verdict, or as a banner when an action was refused. Each entry in
this section takes one such sentence, says what it means in the design,
why BDL insists, and what to do. The compiler's code for the finding is
given last, for search, and can be read in the inspector's *Explain*.

## Find your situation

| You see | Go to |
|---|---|
| a **dashed** node, *declared*, *not yet defined*; *needs a value before simulation can step*; *Checked once … is decided*; *no final target yet*; *Fits … so far — the binding is not finished* | [Incomplete design](incomplete-design.md) — none of these is an error |
| a **red line under the formula**: *… is a dimensionless quantity, but this formula produces …*; *… is not something this mapping reads or can call*; *… is not a unit*; *… reads Tilt here, but this is …*; *… reads nothing; it is a value, not something to apply* | [Types, units and concepts](type-and-concept-errors.md) |
| *reads across domains*; *… updates in a different timing domain from …*; *… depends on its own current value*; *These relationships depend on each other in the same instant*; *… remembers a value over time but has no timing domain*; *`delay` can only be used in a relationship without inputs* | [Timing](timing-errors.md) |
| a link will not land; *contested*; *… already has a final target*; *… expects …, but … produces …*; *Replace the connection?*; *Carry across timing domains*; *… expects …, but … provides …*; a banner about a port in use or a component that cannot stand in | [Connections](connection-errors.md) |
| *Not feasible on …*; *Nothing on … can carry …*; *The pin chosen by hand … cannot carry …*; *No device on … for …*; *Not connected to an output* | [Deployment](deployment-errors.md) |
| a **banner** after an action: *a concept named … already exists*; *concept … is still used by …*; *the project has moved on* | the action was refused, nothing changed; read the banner — it names the reason — and *Dismiss* |
| *Compiler not connected* in the status line | [Install and launch](../getting-started/install-and-launch.md): Studio does nothing semantic on its own |

## Three habits

1. **Read the sentence.** It names the object and the rule. The
   explanation under it (or in *Explain*) names the alternative.
2. **Look for a fix.** The inspector's **Fixes** section offers the
   actions the tool can take for a finding — *Choose what Temperature is
   represented by*, *Connect a driver to Light Output*, *Detach … from …*
   — as ordinary, undoable edits.
3. **Orange is not red.** Orange and dashed mean *not decided yet*; red
   means *wrong now*. Only red stops anything.
