# Design, Code and Split

The Design page shows one project three ways. A control at the right end
of the bar above the canvas switches between them:

| View | What you see | What you do |
|---|---|---|
| **Design** | the canvas ([Canvas](canvas.md)) | draw the design |
| **Code** | the project's `.bdl` files in an editor | type the design ([Syntax basics](../textual/syntax-basics.md)) |
| **Split** | the canvas on the left, the editor on the right | both, on the same objects |

Switching changes nothing in the project: the graph and the text are two
pictures of the same design, kept in step by Studio's compiler service.
There is no *convert*, *import* or *export*.

## What the editor shows

The text of one file at a time. When the project has several `.bdl`
files, a pop-up at the top of the editor names them; a file that does
not build yet is listed with *— not built*.

The text is the project's files with everything you did on the canvas
written in: rename a concept on the canvas and the text shows the new
name at every place it is used, with your comments and blank lines
untouched. Add a relationship on the canvas and it appears at the end of
`src/main.bdl` (or of the first file, when there is no `main.bdl`), or at
the end of the component body it belongs to.

## Typing

Type as in any editor. A short moment after you stop, the file is read
as a design:

* **It builds.** The canvas changes to match — a new relationship
  appears as a node, placed for you beside what it reads; a renamed
  concept keeps its node, its colour, its place and its connections,
  because the identity of an item lives with its kind and name, not
  with its spelling in the text ([Identity](../textual/overview.md#identity)).
* **It does not build yet.** A banner above the editor says *This file
  does not build yet: the design shows the last version that did.* The
  canvas keeps showing the last version of the file that built, with its
  own banner: *Showing the last version that built; the text has changes
  that do not build yet.* Your text stays exactly as typed. The reasons
  are listed under the editor, one per line, with the line number; click
  one to put the cursor on it. An **×** is something the design cannot
  mean; a **hollow ring** is something the design has no meaning for yet
  and does not stop the file from building. Fix the text and both
  banners go.

Nothing you type is lost, and nothing you drew is lost: a file that does
not build never erases the graph.

**Undo** (⌘Z) undoes changes to the design — a relationship added in the
text is undone like one added on the canvas, and the text follows.
Changes that only touch comments or spacing are not steps of that
history.

## Split: one selection

Select a node on the canvas and the editor scrolls to its declaration.
Click inside an item's text and its node is selected on the canvas, so
the inspector shows it. The selection is the same object in both views.

## Saving

**⌘S** writes the files as the editor shows them, then the layout and
the sidecars ([Project files](../reference/project-files.md)). A file
that does not build yet is not written; fix it first.

## Not built yet

Findings underlined in the text (the list and the cursor jump stand in);
completion and hover in the editor (a code editor with the language
server has them — [Editor and language server](../textual/editor-and-lsp.md));
a *Format* command; creating a second source file from Studio (make it
in a code editor; it appears on reload).

## Related

[Canvas](canvas.md) · [Textual BDL](../textual/overview.md) ·
[Authoring a project as text](../workflows/authoring-as-text.md) ·
[Source files](../troubleshooting/text-project-errors.md)
