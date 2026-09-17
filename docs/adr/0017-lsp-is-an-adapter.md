---
id: ADR-0017
status: accepted
date: 2026-09-15
area: ide
supersedes: []
superseded-by: []
related: []
fv: []
---
# ADR-0017: LSP is an adapter over a semantic-first IDE service

**Status**: accepted (2026-09-15)

## Context

BDL has two authoring surfaces — the Studio canvas and textual `.bdl`
files — that must share one language service. The obvious shortcut is to
make the language server *be* that service: define completion, hover,
diagnostics and rename in LSP's types and let Studio call them. That would
put text positions at the centre of a language whose visual surface has
no text, and would either force Studio to speak JSON-RPC and invent fake
ranges for nodes, or leave it with its own semantic layer. It would also
tie every internal API to one wire protocol's evolution.

## Decisions

1. **The IDE service is semantic-first and BDL-owned.** `bdl-ide-db`
   holds ground state (committed snapshot, overlays, documents, entity
   references, projections, stamps, cancellation) and produces immutable
   `AnalysisSnapshot`s; `bdl-ide` answers queries over a snapshot in
   BDL-owned types (`SemanticDiagnostic`, `SemanticHover`,
   `SemanticCompletion`, `SemanticEditPlan`, `SemanticAction`,
   `InvalidationPreview`). Identity is `EntityRef` (stable ids), location
   is a `ProjectionAnchor` (text range *or* visual element), and text
   ranges are UTF-8 bytes. No LSP type appears below `bdl-lsp`.

2. **`bdl-lsp` is a thin adapter.** It owns transport, position encoding
   and type rendering, and nothing else: no buffer store (`didOpen`/
   `didChange`/`didClose` are overlay updates on the shared host), no
   text search, no semantic logic. Custom requests are kept to three
   (`bdl/explainEntity`, `bdl/invalidationPreview`, `bdl/previewEdit`);
   simulation, hardware allocation, layout and canvas geometry stay out of
   LSP.

3. **Stack: `lsp-server` 0.10 + `lsp-types` 0.97.** Compared:

   | | `lsp-server` + `lsp-types` | `tower-lsp` | `tower-lsp-server` (fork) | `async-lsp` |
   |---|---|---|---|---|
   | maintenance | rust-analyzer's own; active | stalled since 2023 | active fork, 0.24 rc | active (oxalica) |
   | LSP version | 3.17 stable (pull diagnostics, position encodings); 3.18 behind `proposed` | 3.17 | 3.17 | 3.17 |
   | runtime coupling | none (sync loop over channels) | tokio | tokio | tower + an async runtime |
   | transport | stdio, TCP, **in-memory** (`Connection::memory()`) | stdio/TCP | stdio/TCP | any `AsyncRead/Write` |
   | cancellation | `$/cancelRequest` arrives on the same loop; we bind it to our own tokens | per-request futures | per-request futures | middleware |
   | custom requests | a `Request` impl per method | trait methods + `custom_method` | same | trait |
   | keeps LSP at the edge | yes — messages are plain values dispatched by method | the server *is* a trait object of LSP methods | same | same |

   `lsp-server` wins on the points that matter here: no runtime imposed on
   the core, an in-memory connection for end-to-end tests, and a plain
   message loop that makes the adapter obviously an adapter. The
   architectural precedent (rust-analyzer: `ide` crate with its own types,
   `rust-analyzer` binary as the LSP shell) is the one this repository
   follows. Compatibility: LSP 3.17 semantics; 3.18 features are not
   enabled (`lsp-types`' `proposed` flag is unstable) and none are needed
   for the MVP.

4. **Pull diagnostics are the model.** `textDocument/diagnostic` fits
   immutable snapshots: the client asks, the server answers from one
   world. `publishDiagnostics` is kept only for clients without pull
   support, confined to the notification path.

5. **Studio does not consume LSP.** Its projection is visual and its
   transport is protobuf (ADR-0002, ADR-0007). `bdld` owns the project's
   `IdeHost`; Studio's definition drafts are overlays on it; the
   `AnalyzeDefinitionDraft` request is "update overlay, take a snapshot,
   return the stamped verdict". Both adapters read the same `bdl-ide`
   results.

6. **GLSP concepts are borrowed, GLSP is not adopted.** The useful idea is
   structure-aware operations over a shared source model, separated from
   presentation operations; that is `SemanticEditPlan` over `EditOp`s.
   Replacing Flutter Studio or `bdld` with an Eclipse GLSP stack is out of
   scope and against ADR-0001/0002.

## Consequences

* Every editor (VS Code, Zed, Neovim, …) gets the same semantics as
  Studio, and a semantic feature is implemented once, in `bdl-ide`.
* Text positions exist in exactly one file (`bdl-lsp/src/position.rs`),
  tested against non-ASCII input in all three encodings.
* The workspace gains three crates; `bdld` depends on `bdl-ide`.
* Rename across surfaces is a `SemanticEditPlan` with model operations
  that a `WorkspaceEdit` cannot carry; the LSP adapter applies the text
  side and exposes the full plan through `bdl/previewEdit`. Applying the
  model side from an editor (a project-file save, or an editor command
  that talks to `bdld`) is follow-up work.
