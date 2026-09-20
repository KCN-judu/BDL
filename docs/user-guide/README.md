# Behavior Designer — user guide

Behavior Designer is a desktop tool for designing **how a product behaves**:
what it senses, what it decides, and what it shows or moves — before any
firmware is written. You describe the behavior as named values and the
relationships between them; the tool checks the design as you build it, lets you
simulate it tick by tick, and tells you whether it fits a particular board.

The language behind the tool is called **BDL**. The canvas, the inspector and
the formula field are one way of writing it; the text form is another, for
people who prefer an editor — see [Textual BDL](textual/overview.md). Either way
you are describing behavior in a language; what the tool keeps out of your way
is the machine: which pin, which protocol, which HAL call are decided later, at
deployment.

## What problem it solves

A product's behavior is usually decided in sketches and spreadsheets and then
rediscovered in code, where the meaning of each number is lost. In Behavior
Designer a value has a **meaning** (a _Brightness_ is not an _Opacity_, even if
both are numbers between 0 and 1), a relationship between values is checked for
units and dimensions, timing is explicit, and a physical output can only be
driven by one relationship. Unfinished designs are normal: you can name a
relationship before you know its formula, and the tool tells you what is still
open rather than refusing to work.

## What you can do here

|                        |                                                                                                                                                   |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Design**             | name the concepts of your product, relate them with formulas, decide their timing, and connect the results to physical outputs                    |
| **Simulate**           | feed values in, step the design tick by tick, and watch every value — with the same evaluator that defines the language                           |
| **Deploy**             | choose a board and see whether the design's outputs can be placed on its pins, and why not if they cannot                                         |
| **Organize and reuse** | group related relationships into a behavior, package a behavior as a reusable component, and compose instances of components into a larger system |

Not yet in the tool: flashing a board, live values from a running device, and
importing components from other projects. The guide says so where it matters.

## Where to start

| If you are…                                                                  | Start here                                                                                                                                           |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| a product or industrial designer who wants to describe a behavior and try it | [What is BDL?](getting-started/what-is-bdl.md) → [Your first behavior](getting-started/first-behavior.md)                                            |
| an embedded prototyper who knows sensors and actuators                       | [Your first behavior](getting-started/first-behavior.md), then [From sensor to output](workflows/sensor-to-output.md) and [Deploy](studio/deploy.md) |
| a code-oriented user who would rather type BDL                               | [Textual BDL — overview](textual/overview.md), which says exactly what works today                                                                   |
| a researcher or language person who wants the semantics                      | the _Going deeper_ section at the end of each concept page, and [Terminology](reference/terminology.md) → the technical docs in `docs/`              |

## Contents

**Getting started** [What is BDL?](getting-started/what-is-bdl.md) ·
[Install and launch](getting-started/install-and-launch.md) ·
[Your first behavior](getting-started/first-behavior.md) ·
[Your first simulation](getting-started/first-simulation.md) ·
[Your first deployment check](getting-started/first-deployment.md) ·
[Your first board](getting-started/pico-demo.md)

**Concepts** — the ideas, one page each [Concepts](concepts/concepts.md) ·
[Relationships](concepts/relationships.md) ·
[Incomplete designs](concepts/incomplete-designs.md) ·
[Timing](concepts/timing.md) · [Physical outputs](concepts/physical-outputs.md)
· [Behavior groups](concepts/behavior-groups.md) ·
[Components](concepts/components.md) ·
[Behavior systems](concepts/behavior-systems.md)

**Studio** — the interface, one panel or page each
[Workspace](studio/workspace.md) · [Canvas](studio/canvas.md) ·
[Library](studio/library.md) · [Inspector](studio/inspector.md) ·
[Formula editor](studio/formula-editor.md) · [Simulate](studio/simulate.md) ·
[Deploy](studio/deploy.md) · [System projects](studio/system-projects.md) ·
[Design, Code and Split](studio/code-view.md)

**Workflows** — one goal each, on the same lamp
[From sensor to output](workflows/sensor-to-output.md) ·
[A second output](workflows/multi-output-behavior.md) ·
[Grouping a behavior](workflows/grouping-behavior.md) ·
[Packaging a behavior as a component](workflows/package-as-component.md) ·
[Composing components](workflows/composing-components.md) ·
[Carrying a value across timing domains](workflows/cross-domain-transport.md) ·
[Versioning a component](workflows/component-versioning.md) ·
[Authoring a project as text](workflows/authoring-as-text.md)

**Textual BDL** [Overview and current status](textual/overview.md) ·
[Syntax basics](textual/syntax-basics.md) ·
[Editor and language server](textual/editor-and-lsp.md) ·
[Authoring a project as text](workflows/authoring-as-text.md)

**Troubleshooting** [Start here](troubleshooting/README.md) ·
[Incomplete design](troubleshooting/incomplete-design.md) ·
[Types, units and concepts](troubleshooting/type-and-concept-errors.md) ·
[Timing](troubleshooting/timing-errors.md) ·
[Connections](troubleshooting/connection-errors.md) ·
[Deployment](troubleshooting/deployment-errors.md) ·
[Source files](troubleshooting/text-project-errors.md)

**Reference** [Terminology](reference/terminology.md) ·
[Keyboard and mouse](reference/keyboard-and-mouse.md) ·
[Status meanings](reference/status-meanings.md) ·
[Formula language](reference/formula-language.md) ·
[Project files](reference/project-files.md) · [Command line](reference/cli.md)

**About this guide** [Style guide](STYLE_GUIDE.md) ·
[Verification matrix](VERIFICATION.md) · [Screenshot plan](SCREENSHOT_PLAN.md) ·
[Documentation research](DOCUMENTATION_RESEARCH.md)

The technical documentation for people building the tool itself lives one level
up in `docs/` (architecture, compiler pipeline, protocol, formal notes). This
guide links to it from _Going deeper_ sections and does not repeat it.
