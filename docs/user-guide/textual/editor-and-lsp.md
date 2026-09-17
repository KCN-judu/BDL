# Editor and language server

`bdl-lsp` is a language server for `.bdl` files. It speaks the Language
Server Protocol over standard input and output, so any editor with LSP
support can use it. There is no editor extension or syntax-highlighting
grammar shipped for any particular editor yet; you configure your editor
to launch the binary.

## Launching

Build it with the rest of the workspace (`cargo build --workspace`, or
`just build`); the binary is `target/debug/bdl-lsp`. Run it with no
arguments; it logs to standard error (set `RUST_LOG` to change the level).

Point your editor's LSP client at that binary for files with the `.bdl`
extension.

## Which project it checks against

At start-up the server looks for a **project root** — a folder containing
`bdl.toml`:

1. `initializationOptions.projectRoot`, if your client sends it;
2. otherwise the first workspace folder;
3. otherwise the older `rootUri`.

If that folder holds a BDL project, the server loads it as the
**committed state**: the concepts, relationships, domains, outputs and
devices Studio saved there. If not, it starts with an empty design named
after the folder, and open documents alone populate it.

An open `.bdl` document is an **overlay** on that state. Its `concept`
and `mapping` items are matched by name to the project's; a name the
project does not have becomes a new entity for as long as the document
is open. Every query answers about *project + open documents*. Nothing
is written back: closing the document or the editor leaves the project
as Studio saved it.

## What the server answers

| Feature | LSP request | Notes |
|---|---|---|
| **Diagnostics** | `textDocument/diagnostic` (pull); `publishDiagnostics` only for clients without pull support | the same findings as Studio, with the same spans; *open* findings (a concept without a value form) come as information, not errors |
| **Hover** | `textDocument/hover` | what a name is, its value form, its status, its description |
| **Completion** | `textDocument/completion` | inputs, relationships, units after a number, keywords, `delay` / `sync` where allowed, library templates for new concepts — ranked by the compiler |
| **Go to definition** | `textDocument/definition` | by identity — the declaration of the entity under the cursor |
| **References** | `textDocument/references` | every place the entity is named |
| **Rename** | `textDocument/prepareRename`, `textDocument/rename` | renames the entity everywhere it is named, as text edits; parameter names are not touched |
| **Document symbols** | `textDocument/documentSymbol` | the file's concepts and relationships |
| **Semantic tokens** | `textDocument/semanticTokens/full` | concept, mapping, output, clock, device, unit, keyword, parameter, constructor, number, comment, operator |
| **Code actions** | `textDocument/codeAction` | the compiler's *fixes* for findings in the range, and context actions at the cursor. An action that changes only text carries its edit; an action that would change the project model (choose a value form, connect a driver, choose a domain) or needs a choice is listed as **disabled** with the reason — make it in Studio |
| **Cancellation** | `$/cancelRequest` | honoured; a cancelled request answers *RequestCancelled* |

Positions are negotiated in UTF-8, UTF-16 or UTF-32 as the client
prefers; documents are synchronised in full on every change.

Three custom requests exist for tools built on the server:
`bdl/explainEntity` (the explanation of the entity at a position — the
same content as Studio's *Explain*), `bdl/invalidationPreview` (what a
model edit would reopen), and `bdl/previewEdit` (the full semantic plan
of a rename at a position).

## What the editor cannot do today

* Save a project. A `.bdl` file is analysed against the project; it is
  not the project.
* Author timing domains, outputs, devices, behaviors or components: they
  have no syntax.
* Apply model-changing fixes (they are disabled in the editor).
* Format a file, show inlay hints, search workspace symbols.

## Related

[Overview and current status](overview.md) · [Syntax basics](syntax-basics.md)

*For language implementers:* `docs/IDE_SERVICE_ARCHITECTURE.md`
(overlays, projections, the LSP adapter) and ADR-0017.
