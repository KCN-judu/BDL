# Troubleshooting — start here

Behavior Designer reports problems as sentences about _your_ objects, in the
place they belong: under the formula field, under _Timing_ or _Drives_ in the
inspector, on the Simulate page's readiness list, on the Deploy verdict, or as a
banner when an action was refused. Each entry in this section takes one such
sentence, says what it means in the design, why BDL insists, and what to do. The
compiler's code for the finding is given last, for search, and can be read in
the inspector's _Explain_.

## Find your situation

| You see                                                                                                                                                                                                                                                                                              | Go to                                                                                                   |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| a **dashed** node, _declared_, _not yet defined_; _needs a value before simulation can step_; _Checked once … is decided_; _no final target yet_; _Fits … so far — the binding is not finished_                                                                                                      | [Incomplete design](incomplete-design.md) — none of these is an error                                   |
| a **red line under the formula**: _… is a dimensionless quantity, but this formula produces …_; _… is not something this mapping reads or can call_; _… is not a unit_; _… reads Tilt here, but this is …_; _… reads nothing; it is a value, not something to apply_                                 | [Types, units and concepts](type-and-concept-errors.md)                                                 |
| _reads across domains_; _… updates in a different timing domain from …_; _… depends on its own current value_; _These relationships depend on each other in the same instant_; _… remembers a value over time but has no timing domain_; _`delay` can only be used in a relationship without inputs_ | [Timing](timing-errors.md)                                                                              |
| a link will not land; _contested_; _… already has a final target_; _… expects …, but … produces …_; _Replace the connection?_; _Carry across timing domains_; _… expects …, but … provides …_; a banner about a port in use or a component that cannot stand in                                      | [Connections](connection-errors.md)                                                                     |
| _Not feasible on …_; _Nothing on … can carry …_; _The pin chosen by hand … cannot carry …_; _No device on … for …_; _Not connected to an output_                                                                                                                                                     | [Deployment](deployment-errors.md)                                                                      |
| a **banner** after an action: _a concept named … already exists_; _concept … is still used by …_; _the project has moved on_                                                                                                                                                                         | the action was refused, nothing changed; read the banner — it names the reason — and _Dismiss_          |
| in a `.bdl` file, the Code view or `bdld check`: _… is not a concept of this project_; _… is declared twice_; _… has a fresh one_; banners _This file does not build yet …_ and _… changed on disk since the project was opened_; _A name is one word, without spaces_                               | [Source files](text-project-errors.md)                                                                  |
| _Compiler not connected_ in the status line                                                                                                                                                                                                                                                          | [Install and launch](../getting-started/install-and-launch.md): Studio does nothing semantic on its own |

## Three habits

1. **Read the sentence.** It names the object and the rule. The explanation
   under it (or in _Explain_) names the alternative.
2. **Look for a fix.** The inspector's **Fixes** section offers the actions the
   tool can take for a finding — _Choose what Temperature is represented by_,
   _Connect a driver to light_, _Detach … from …_ — as ordinary, undoable edits.
3. **Orange is not red.** Orange and dashed mean _not decided yet_; red means
   _wrong now_. Only red stops anything.
