# BDL for VS Code

The reference editor integration of `bdl-lsp`. The extension is a thin
client: every language fact comes from the server (`crates/bdl-lsp`),
which runs the same loader and the same semantic model as Behavior
Designer and `bdld`.

## What it gives you

- Diagnostics as you type, across every file of a text project.
- Go to definition, find references and rename by identity — across files,
  for concepts, relationships, components, ports and instances.
- Completion that knows its scope (a component body offers the body's
  names; `bind inst.` offers the instance's ports).
- Hover, inlay hints (the concept a parameter reads, the transport of a
  binding that crosses timing domains), semantic highlighting, document
  symbols, formatting.
- Read-only views: **BDL: Explain Entity at Cursor**, **BDL: Show Kernel
  Core**, **BDL: Show Generated Rust**.

Until the server answers, a TextMate grammar (`syntaxes/`) colours
keywords, comments and units.

## Setup

1. Build the server: `cargo build -p bdl-lsp` (the binary is
   `target/debug/bdl-lsp`).
2. Set `bdl.serverPath` to that path, or put `bdl-lsp` on your `PATH`.
3. Open a text project (a folder with `bdl.toml` of kind `text` and
   `src/**/*.bdl`) as the workspace folder.

To run the extension from source:

```bash
cd editors/vscode && npm install && npm run compile
```

then press F5 in VS Code (Run Extension), or `npm run package` for a
`.vsix`.

## How it works

- The server is started with `initializationOptions.projectRoot` set to
  the nearest folder with `bdl.toml` above the active file (else the
  first workspace folder). One server per window.
- Files under `src/**/*.bdl`, `.bdl/identities.json` and `bdl.toml` are
  watched; a change outside an open buffer makes the server re-read the
  project and write the reconciled identity sidecar.
- Virtual documents are `bdl-explain://`, `bdl-core://` and
  `bdl-generated://` URIs served by `bdl/virtualDocument`. They are
  renderings, never sources of truth, and are not editable.

See `docs/user-guide/textual/editor-and-lsp.md` for the user guide and
`docs/architecture/ide-service.md` for the design.
