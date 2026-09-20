# Textual BDL — overview

BDL has a text form. It is the same language as the canvas — every `concept`,
`mapping`, `clock`, `output`, `component` and `instance` you write is the same
object Studio draws, checked by the same compiler — and since this version a
whole project can be written as text, opened in Studio, edited on either side,
and saved back as text. This page says exactly what works today.

```bdl
// src/main.bdl — a lamp that dims with tilt

concept Tilt : Angle
concept Brightness : Scalar
clock interaction

mapping tilt : () -> Tilt @interaction         // supplied from outside
mapping brightness : () -> Brightness @interaction
brightness() = dimByTilt(tilt)

/// How bright the lamp is for a tilt.
mapping dimByTilt : Tilt -> Brightness
dimByTilt(t) = t / (90 deg)

output light : Brightness @interaction
drive light by brightness
```

## Current status

### Works today

- **Every project is text.** A project's design lives in its `src/**/*.bdl`
  files ([Project files](../reference/project-files.md)). Studio saves your
  canvas edits back into them as item-level changes — comments and layout of the
  text you did not touch stay as they were — and shows the same files in its
  **Code** view, where typing changes the design
  ([Design, Code and Split](../studio/code-view.md),
  [Authoring a project as text](../workflows/authoring-as-text.md)). Projects
  saved by older versions of Studio are converted to text the first time they
  open, keeping every identity and position.

- **The whole design has syntax**: concepts, relationships, timing domains
  (`clock`, `@domain`), physical outputs and their drivers, devices with fixed
  pins, components with required / provided / parameter ports and clock
  parameters, instances, bindings (with a transported initial value), exports,
  and `///` descriptions ([Syntax basics](syntax-basics.md)).
- **Stable identity.** A renamed, moved or re-ordered item keeps its identity —
  its layout position, its bindings, its place in a behavior group — as long as
  the tools can tell it is the same item (see _Identity_ below).
- **Editor support** through `bdl-lsp`, across every file of the project:
  diagnostics, hover, completion that knows its scope, go to definition,
  references, rename, document symbols, semantic tokens, formatting, inlay
  hints, code actions, and read-only views of the explanation, the kernel Core
  and the generated Rust ([Editor and language server](editor-and-lsp.md)). A
  reference VS Code extension ships in `editors/vscode`.
- **A command line.** `bdld check`, `bdld compile` and `bdld simulate` read the
  same files with the same loader ([Command line](../reference/cli.md)).
- **Parameter names are names.** `dimByTilt(t) = t / (90 deg)` reads its input
  as `t`; the concept's own name is not needed in the body, and renaming the
  concept leaves `t` alone.

### Not there yet

- **User enums.** `enum` items parse and are reported as _open_ (_enums are
  syntax only in this version; the design model has no sum types yet._); their
  constructors cannot be used in formulas.
- **Live two-way editing.** Studio and an editor do not watch each other. Studio
  notices that a file changed on disk when you save and offers to reload; an
  editor re-reads the project when a file is saved. Unsaved work on one side is
  not visible on the other.
- **Model-changing fixes from the editor** (choose a value form, connect a
  driver) are listed but disabled; make them in Studio or by editing the text.
- Workspace-wide symbol search.

## Identity

Every item has a stable identity, kept in `.bdl/identities.json` next to the
sources — a file the tools own and you never need to edit. When the files are
read again, each item is matched to its identity by its kind and name (and, for
the items inside a component, the component's name). So:

- **Editing a body, a signature or a description** keeps the identity.
- **Moving an item** to another file or another place in the file keeps it.
- **Renaming with the editor's rename** or **on the canvas** keeps it — both
  know which item you mean.
- **Retyping a name by hand** keeps it _when it is unambiguous_: one declaration
  of that kind disappeared from the file and one new one appeared with the same
  shape. Two renames of the same kind in the same file at once cannot be told
  apart; those items get fresh identities and a finding says so (_… could not be
  matched to an existing identity; it has a fresh one. Use the editor's rename
  to keep an identity._). Fresh identities lose nothing semantic — only canvas
  placement and group membership, which are keyed by identity.

## How text relates to Studio

One semantic model, two ways of writing it:

|                         | Studio                                            | Text                                                                  |
| ----------------------- | ------------------------------------------------- | --------------------------------------------------------------------- |
| concept                 | sheet: name, value form, description              | `concept Tilt : Angle`, a `///` line above                            |
| relationship            | sheet: reads, produces, domain; the formula field | `mapping f : A -> B @domain` then `f(a) = …`                          |
| formula                 | the same expression language                      | the same expression language                                          |
| timing domain           | the domain list                                   | `clock interaction`, `@interaction` on items                          |
| physical output, driver | output node, _Drives_                             | `output light : Brightness @interaction`, `drive light by brightness` |
| device                  | device sheet with pins                            | `device pwmLight : pwm_channel for light { pin 0 = D3 }`              |
| component, ports        | component context, port sheet                     | `component Lamp { … requires … provides … param … }`                  |
| instance, binding       | instance node, wire                               | `instance a : Lamp { … }`, `bind a.port = value`                      |
| behavior group          | group box                                         | not in the text: kept in `.bdl/authoring.json`                        |
| layout                  | the canvas                                        | not in the text: `ui/layout.json`                                     |
| findings                | inspector, with spans                             | LSP diagnostics, with the same spans, in the right file               |
| identity                | stable ids; names are labels                      | stable ids in `.bdl/identities.json`; names are how the text refers   |

The language server, the command line and Studio's compiler service (`bdld`)
share one loader and one compiler, so a design gets the same verdict on every
surface — tests in the repository hold them equal down to the generated code.

## Who should use it today

Anyone who prefers typing, wants a design under version control as readable
text, or wants to review a design as a diff. Timing, outputs, devices and
components are all there. Use Studio for the canvas, the simulation view and
deployment; the two work on the same files.

## Next

[Syntax basics](syntax-basics.md) ·
[Editor and language server](editor-and-lsp.md) ·
[Authoring a project as text](../workflows/authoring-as-text.md) ·
[Command line](../reference/cli.md)

_For language implementers:_ `docs/spec/textual-syntax.md` is the normative
grammar (§14 for project items); `docs/architecture/ide-service.md` explains the
text workspace and the LSP adapter; `docs/spec/project-format.md` the project
layout; the ADRs on the textual workspace and source identities (0020) and on
the one project with Design, Code and Split views (0023) record the rules.
