# Editor and language server

`bdl-lsp` is a language server for `.bdl` files. It speaks the Language Server
Protocol over standard input and output, so any editor with LSP support can use
it. A reference VS Code extension is in `editors/vscode` (see its README); for
other editors you configure the client to launch the binary.

## Launching

Build it with the rest of the workspace (`cargo build --workspace`, or
`just build`); the binary is `target/debug/bdl-lsp`. Run it with no arguments;
it logs to standard error (set `RUST_LOG` to change the level).

Point your editor's LSP client at that binary for files with the `.bdl`
extension. In VS Code, install the extension from `editors/vscode` and set
`bdl.serverPath` to the binary.

## Which project it works on

At start-up the server looks for a **project root** — a folder containing
`bdl.toml`:

1. `initializationOptions.projectRoot`, if your client sends it (the VS Code
   extension sends the nearest such folder above the active file);
2. otherwise the first workspace folder;
3. otherwise the older `rootUri`.

**The project** (sources under `src/`) is the workspace: every `src/**/*.bdl`
file is known to the server whether or not you have it open, and every answer is
about the whole project — a definition in another file, references across files,
a rename that touches three files. An open buffer replaces the file on disk
while it is open; nothing is written by the server except the identity sidecar
(`.bdl/identities.json`) after a save or an outside change, so the next tool to
open the project agrees on identities. Saving is the editor's ordinary save of
the `.bdl` file.

**A project saved by an older version of Studio** (a `design/*.json` file, no
`src/`) is converted to `.bdl` sources the first time any tool opens it, the
language server included; from then on it is an ordinary workspace.

**No project**: the server starts empty and open documents alone populate it.

## What the server answers

| Feature              | LSP request                                                                                  | Notes                                                                                                                                                                                                                                                                                                                                                                                                         |
| -------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Diagnostics**      | `textDocument/diagnostic` (pull); `publishDiagnostics` only for clients without pull support | the same findings as Studio, with the same spans, in the file they belong to; _open_ findings come as information, not errors; findings about the source files (an unknown concept, a duplicate item, a second driver) are errors on the item                                                                                                                                                                 |
| **Hover**            | `textDocument/hover`                                                                         | what a name is: a concept's value form and description, a relationship's signature and status, a component's ports, an instance's arguments and bound ports, a binding's ends                                                                                                                                                                                                                                 |
| **Completion**       | `textDocument/completion`                                                                    | knows its scope: at an item start the items allowed there (a component body offers `requires`, `provides`, `use`, …); after `instance x :` the components; inside the braces the clock parameters and parameters not yet given; after `bind a.` the instance's ports; after `@` the domains in scope; in a body the body's own relationships and inputs, units after a number, `delay` / `sync` where allowed |
| **Go to definition** | `textDocument/definition`                                                                    | by identity, across files — a concept from its use in a component, a component from its instance, a relationship from a call in a body                                                                                                                                                                                                                                                                        |
| **References**       | `textDocument/references`                                                                    | every place the entity is named, in every file, including uses inside formula bodies                                                                                                                                                                                                                                                                                                                          |
| **Rename**           | `textDocument/prepareRename`, `textDocument/rename`                                          | renames the entity everywhere it is named, in every file, as text edits — concepts, relationships, components, ports, instances; parameter names are left alone                                                                                                                                                                                                                                               |
| **Formatting**       | `textDocument/formatting`                                                                    | canonical spacing and indentation; keeps item order, blank-line grouping, comments and your line breaks (a multi-line body keeps its relative indentation); a file with a syntax error is left untouched                                                                                                                                                                                                      |
| **Inlay hints**      | `textDocument/inlayHint`                                                                     | `: Tilt` after a parameter whose name is not the concept's; `sync interaction → display, init 0` after a binding that crosses domains                                                                                                                                                                                                                                                                         |
| **Document symbols** | `textDocument/documentSymbol`                                                                | the file's items                                                                                                                                                                                                                                                                                                                                                                                              |
| **Semantic tokens**  | `textDocument/semanticTokens/full`                                                           | concept, mapping, output, clock, device, unit, keyword, parameter, constructor, number, comment, operator                                                                                                                                                                                                                                                                                                     |
| **Code actions**     | `textDocument/codeAction`                                                                    | the compiler's _fixes_ for findings in the range and context actions at the cursor. An action that changes only text carries its edit; one that would change the model or needs a choice is listed as **disabled** with the reason                                                                                                                                                                            |
| **Cancellation**     | `$/cancelRequest`                                                                            | honoured; a cancelled request answers _RequestCancelled_                                                                                                                                                                                                                                                                                                                                                      |

Positions are negotiated in UTF-8, UTF-16 or UTF-32 as the client prefers;
documents are synchronised in full on every change. Saving a file or changing
one outside the editor (`workspace/didChangeWatchedFiles`) makes the server
re-read the project.

Custom requests for tools built on the server: `bdl/explainEntity` (the
explanation of the entity at a position — the same content as Studio's
_Explain_), `bdl/virtualDocument` (a read-only rendering: `explain` of the
entity at a position or of the whole project, `core` — the design in kernel
terms, `rust` — the generated core or why nothing is generated yet),
`bdl/invalidationPreview` (what a model edit would reopen), and
`bdl/previewEdit` (the full semantic plan of a rename). In VS Code these are the
commands _BDL: Explain Entity at Cursor_, _BDL: Show Kernel Core_ and _BDL: Show
Generated Rust_.

## What the editor cannot do today

- Apply model-changing fixes (they are disabled in the editor; edit the text or
  use Studio).
- See Studio's unsaved edits, or show its own to Studio, before a save
  ([Authoring a project as text](../workflows/authoring-as-text.md)).
- Search workspace symbols.

## Related

[Overview and current status](overview.md) · [Syntax basics](syntax-basics.md) ·
[Authoring a project as text](../workflows/authoring-as-text.md)

_For language implementers:_ `docs/architecture/ide-service.md` (the text
workspace, overlays, projections, the LSP adapter) and the ADR on the LSP as an
adapter.
