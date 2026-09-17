# Textual BDL — overview

BDL has a text form. It is the same language as the canvas — every
`concept` and `mapping` you write is the same object Studio draws, and a
formula in a `.bdl` file is checked by the same compiler as the formula
field — but the text surface is younger than Studio, and this page says
exactly what it does today.

```bdl
// Lamp behaviour

concept Tilt : Angle
concept Brightness : Scalar

mapping dimByTilt : Tilt -> Brightness
dimByTilt(tilt) =
  tilt / (90 deg)
```

## Current status

**Works today**

* **Parsing** of the full v0.1 syntax — `concept`, `mapping`, `enum`
  items; the expression language (`if`, `let` blocks, `match`, calls,
  `Some` / `None`, `delay` / `sync`, units on numbers); comments; error
  recovery that keeps going after a mistake.
* **Checking** a `.bdl` file against a project through the language
  server: concepts and relationships in the file are matched by name to
  the project's, formulas are checked exactly as in
  Studio, and the same findings come back with their spans.
* **Editor support** through `bdl-lsp`: diagnostics, hover, completion,
  go to definition, references, rename, document symbols, semantic
  tokens, code actions ([Editor and language server](editor-and-lsp.md)).
* **One formula grammar**: what you type in Studio's formula field is
  the expression sub-language of the text form, so formulas move between
  the two without change.

**Not there yet**

* **A textual project.** A project is stored as Studio's JSON files
  ([Project files](../reference/project-files.md)). A `.bdl` file is an
  *overlay* the language server lays over that project: it is analysed,
  it is not saved into the project, and closing the editor forgets it.
  There is no way to author a whole project as text and open it in Studio.
* **Timing domains, physical outputs, devices, behaviors, components** —
  none has a textual syntax. The words `clock`, `output`, `context`,
  `component`, `require` are reserved for them. Findings about them
  (an output's driver, a domain crossing) are reported on the relationship
  they concern.
* **Parameter names as a binding layer.** `dimByTilt(tilt) = …` names its
  parameter `tilt`, but the body is checked against the *concept's* name:
  `tilt` resolves to *Tilt* because the two spell the same; any other
  parameter name is an unknown name. Rename does not touch parameter
  names.
* **User enums.** `enum` items parse and are reported as *open*; their
  constructors cannot be used in formulas yet.
* An editor extension or syntax grammar for any particular editor; a
  formatter; inlay hints.

## How text relates to Studio

One semantic model, two ways of writing it:

| | Studio | Text |
|---|---|---|
| concept | sheet: name, value form, unit | `concept Tilt : Angle` |
| relationship | sheet: reads, produces; the formula field | `mapping f : A -> B` then `f(a) = …` |
| formula | the same expression language | the same expression language |
| timing, outputs, devices, behaviors, components | yes | no syntax yet |
| findings | inspector, with spans | LSP diagnostics, with the same spans |
| identity | stable ids; names are labels | matched to the project by declared name |

The language server and Studio's compiler service (`bdld`) share the same
IDE service underneath, so a formula gets the same verdict on both
surfaces — a test in the repository holds them equal.

## Who should use it today

Someone who wants to **read** a design as text, **check** formulas from
an editor, or **explore** the language. Someone who needs to *deliver* a
design should use Studio, because only Studio saves projects and only
Studio has timing, outputs and deployment.

## Next

[Syntax basics](syntax-basics.md) · [Editor and language server](editor-and-lsp.md)

*For language implementers:* `docs/TEXTUAL_SYNTAX.md` is the normative
grammar; `docs/IDE_SERVICE_ARCHITECTURE.md` explains overlays and the
textual projection; ADR-0014 and ADR-0017 record the infrastructure
choices.
