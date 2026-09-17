---
kind: background
area: ide
status: current
---
# IDE service — research context

Architectural precedents the IDE service draws on, summarised in our own
words as lessons, not as descriptions of those projects. Each entry ends
with what BDL took from it and what it deliberately did not.

## rust-analyzer: an internal IDE API with an LSP shell

The best-known shape for a language service in Rust is a layered one: a
database of inputs (files, configuration), a semantic layer computing
facts on demand, an `ide` crate whose API is expressed in the language's
own terms (a `FileId` and a byte offset in, a structured result out — no
protocol types), and a separate binary that speaks LSP by translating
those results. The IDE API is the product; LSP is one client of it, and
the test suite drives the IDE API directly with fixtures rather than
through JSON-RPC.

Two further ideas travel with that shape. First, *analysis snapshots*:
every request runs against an immutable view of all inputs, and a change
to any input cancels in-flight requests that depended on it, so results
never mix worlds. Second, *positions are not identity*: the IDE tracks
things by stable references and maps to text ranges at the edge.

**Taken**: the crate split (`bdl-ide-db` / `bdl-ide` / `bdl-lsp`), BDL-owned
result types, immutable stamped snapshots, cancellation scoped to what a
request depends on, tests against the IDE API first.
**Not taken**: the incremental query engine underneath (Salsa). BDL's
whole-project analysis is a couple of milliseconds at hundreds of
declarations; the API is shaped so an engine can be added if profiling
ever demands it, and not before.

## LSP 3.17: pull diagnostics, position encodings, cancellation

Two protocol additions make a snapshot-based service fit LSP cleanly.
*Pull diagnostics* (`textDocument/diagnostic`) let the client ask and the
server answer from one consistent world, instead of the server pushing
whenever it feels like it; a `resultId` lets a client skip unchanged
reports. *Position encodings* (`general.positionEncodings`) let a server
say "I count bytes" and avoid UTF-16 arithmetic altogether when the client
agrees — and make the conversion an explicit, negotiated, testable thing
when it does not. Cancellation (`$/cancelRequest`) is best-effort by
design; the protocol expects servers to answer cancelled requests with an
error, and clients to tolerate a late answer, which is why a server needs
its own staleness check beyond cancellation.

**Taken**: pull diagnostics as the model with push confined to a
fallback; one `LineIndex` that owns the negotiated encoding; cancellation
tokens plus stamps as the correctness backstop.

## GLSP: operations over a shared source model

Graphical language server frameworks make one distinction sharply: an
*operation* changes the source model (create a node, connect an edge, set
a property) and is validated and executed by the server, while a *layout
change* is presentation and never touches the model. The diagram is a
projection the server computes; the client renders it and sends
operations back. Actions and their applicability are computed
server-side, so a client never invents a fix.

**Taken**: the semantic/presentation split as a rule of the IDE service
(`SemanticEditPlan` over `EditOp`s versus layout, which never creates a
revision); server-computed actions with explicit applicability
(`Ready | NeedsChoice | Blocked`); a visual projection derived from the
model rather than from geometry.
**Not taken**: the GLSP stack itself. The product is Flutter + `bdld`
over protobuf; borrowing the operation model costs nothing, replacing the
stack would cost everything.

## Hybrid textual/graphical language servers

Systems that offer both a text and a diagram surface over one model face
one recurring problem: which surface is the truth when they disagree? The
robust answers keep a *single* model with stable identities, treat both
text and diagram as projections with explicit binding steps, and make
"the text says X but the model has Y" an ordinary diagnostic rather than a
silent resolution. Identity binding by name is the weak point — a rename
in text looks like delete-plus-create unless the binding is stable across
edits — which is why such systems push rename through the model's own
rename operation rather than through text substitution.

**Taken**: one model, two projections; an explicit binding pass for the
textual surface with binding faults as diagnostics; rename as a planned
model operation with projected text edits, never text substitution.
**Open**: persisted textual identities (so that a name change *typed*
into a file is recognised as a rename) are future work; today a typed
rename of a declaration is a new declaration in the overlay until the
model is renamed through the tool.

## Projectional and shared-model editing

Projectional editors (structure editors over an AST) show that authoring
surfaces can be arbitrarily different — text-like, tabular, graphical —
while the artefact stays one tree with stable node identities, and that
the hard part is not rendering but *editing* through the projection
without losing identity. The cost they pay is that free text entry must be
reconciled back into the tree.

**Taken**: identity lives in the model (`EntityRef` over stable ids,
ADR-0008); a projection map from identity to surface location is what
makes every query surface-independent. BDL keeps free-text editing of
formulas and files (with a lossless CST, ADR-0014) rather than a pure
structure editor, and reconciles by binding.

## Incomplete-program tooling as a design influence

Tooling built for programs that are being written — holes in typed
languages, partial parses with recovery, "unresolved" as a first-class
state — treats incompleteness as the normal case, not a failure mode:
parsing is total, typing proceeds around holes, and the IDE keeps
answering questions about the parts that are known. BDL's kernel is
already built this way (an unresolved declaration is a legal declaration;
refinement versus edit is formal), so the IDE service must not be the
place where "incomplete" turns into "error".

**Taken**: `SemanticSeverity::Open` distinct from `Error` on every
surface; hover, references, rename, navigation and diagnostics all work
on unresolved mappings, unbound concepts and partially wired outputs;
binding faults for unsupported constructs are *open* when the language
merely does not have them yet.

## Summary of lessons

1. Define the language service in the language's own terms; adapt at
   the edge.
2. Queries run on immutable, stamped snapshots; changes cancel and
   stamps decide.
3. Identity is stable and surface-independent; positions and nodes are
   projections.
4. Operations on the model are data; presentation changes are not
   operations.
5. Incomplete is legal, and the service must say so on every surface.
6. Add an incremental engine when measurement asks for it, not before.
