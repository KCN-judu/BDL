# Authoring a project as text

**Goal:** write the tilt lamp as `.bdl` files, check it from your editor, open
the same project in Studio, and keep both sides agreeing.

**You need:** the workspace built (`cargo build --workspace`), an editor with
`bdl-lsp` configured
([Editor and language server](../textual/editor-and-lsp.md)), Studio running.

## Steps

1. **Create the project.** In Studio's welcome page choose _New Project…_ and
   pick a folder. Or from a terminal make the folder by hand:

   ```text
   lamp/
   ├── bdl.toml            schema_version = 2, name = "lamp", compiler_version = "…"
   └── src/main.bdl
   ```

   Studio shows an empty system canvas; on disk there is `src/main.bdl` with a
   comment, and the two tool-owned sidecars
   ([Project files](../reference/project-files.md)).

2. **Write the design.** In your editor, `src/main.bdl`:

   ```bdl
   concept Tilt : Angle
   concept Brightness : Scalar
   clock interaction

   mapping tilt : Tilt @interaction
   mapping brightness : Brightness @interaction
   brightness() = dimByTilt(tilt)

   /// How bright the lamp is for a tilt.
   mapping dimByTilt : Tilt -> Brightness
   dimByTilt(t) = t / (90 deg)

   output light : Brightness @interaction
   drive light = brightness
   ```

   As you type, findings appear on the line they concern; hovering `Brightness`
   in the signature shows _concept Brightness : Scalar_. Split the vocabulary
   into `src/concepts.bdl` if you like — items may live in any file under
   `src/`.

3. **Save.** The editor writes the file; the server re-reads the project and
   writes `.bdl/identities.json`.

4. **Check from the command line** (optional):

   ```bash
   bdld check lamp
   ```

   prints
   `lamp: 2 concept(s), 3 relationship(s), 1 output(s); checks, outputs complete`
   — or each finding with its file and line and a non-zero exit
   ([Command line](../reference/cli.md)).

5. **Open it in Studio.** _Open Project…_, choose the folder. The canvas shows
   the three relationships, the output and its driver, with the description on
   `dimByTilt`'s sheet. Simulate as usual
   ([First simulation](../getting-started/first-simulation.md)).

6. **Edit on the canvas and save (⌘S).** Rename _Tilt_ to _Lean_ in the sheet,
   save. On disk, `main.bdl` now reads `concept Lean : Angle`,
   `mapping dimByTilt : Lean -> Brightness` — and everything else, comments
   included, is as you wrote it. The item kept its identity: its position on the
   canvas is where you left it.

7. **Edit the text while Studio has the project open.** Add
   `concept Glow : Scalar` in your editor and save. Then in Studio make any edit
   and press ⌘S. Studio refuses with a banner — _1 changed on disk since the
   project was opened: src/main.bdl_ — and two buttons: **Reload from disk**
   takes the file's version (Studio's unsaved edits are dropped); **Overwrite**
   writes Studio's version over the file. Choose _Reload from disk_: _Glow_
   appears on the canvas, every other item keeps its identity and place.

## What BDL means by this

The text is the design; Studio is a view of the same design. There is no import
or export step and no second copy: Studio reads `src/**/*.bdl` and, when you
save, writes back only the items you changed, as text edits. Behavior groups and
canvas positions are not part of the design, so they live beside the sources
(`.bdl/authoring.json`, `ui/layout.json`) and are keyed by identity, which is
why a rename or a move of an item keeps its place.

Two tools never write the same file at once: Studio checks the files'
modification times before saving and asks, rather than merging.

## If it does not work

- **Studio's banner says a file changed but you did not edit it.** A formatter
  or your editor's save touched it. _Reload from disk_ is safe when you have no
  unsaved canvas edits.
- **After retyping several names by hand, positions on the canvas were lost.**
  Two renames of the same kind in one file at once cannot be told apart; the
  items got fresh identities (a finding on each says so). Rename one at a time,
  or use the editor's rename.
- **An item is a finding in the editor but fine in Studio** (or the reverse):
  one side has unsaved changes. Save on both sides; the two read the same files.
- **`bdld check` exits 2 with _error:_** — the folder has no `bdl.toml`, or it
  is an older project that has both `src/` files and a `design/*.json` file;
  keep one of the two.

## Next

[Syntax basics](../textual/syntax-basics.md) ·
[Composing components](composing-components.md) — the same design as components
and instances, in text as `component` / `instance` / `bind`.
