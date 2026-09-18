# Project files

A project is a folder of plain files. Studio reads and writes them; you
can put the folder under version control, copy it, or diff it. The
authoritative description for tool builders is `docs/spec/project-format.md`.

```text
lamp/
├── bdl.toml                 the manifest: name, schema version
├── src/
│   ├── concepts.bdl         the design — any number of .bdl files, any names, read in path order
│   └── main.bdl
├── .bdl/
│   ├── identities.json      the stable identity of every item — owned by the tools
│   └── authoring.json       behavior groups — owned by the tools
└── ui/
    └── layout.json          where things are on the canvas — not part of the design
```

There is one kind of project. Its design is the `.bdl` files
([Syntax basics](../textual/syntax-basics.md)); Studio shows them as a
graph, as text, or both ([Design, Code and Split](../studio/code-view.md)).

## What is in the design

Everything that means something: concepts (name, description, value
form), relationships (name, description, reads, produces, formula, timing
domain, the output it drives), timing domains, physical outputs (name,
what it accepts, domain, required), devices (name, kind, output, fixed
pins), components (each with its own design and its ports), instances,
bindings and exports.

Every object has a stable numeric identity that never changes and is
never reused. The **name is how the files spell it**: renaming an item
changes its name everywhere it is written and nothing else, because the
identity lives in `.bdl/identities.json`, matched to the item by its
kind and name. Two objects with the same name in two projects have
nothing to do with each other.

A name must be a name the files can spell: letters, digits and
underscores, not starting with a digit, not a word of the language
(`light`, `pwmLight`, `dimByTilt`). Studio refuses any other spelling
and shows the one that would work
([Source files](../troubleshooting/text-project-errors.md)).

## What is in the sidecars

`.bdl/identities.json` records which identity each item has, by its
kind and name, together with the counters that hand out new ones. It is
written on open when the sources needed identities the file did not
have, after every save, and by the language server after a save in the
editor. Keep it under version control with the sources; do not edit it.
Deleting it gives every item a fresh identity — nothing semantic is lost,
but canvas positions and group membership, which are keyed by identity,
are.

`.bdl/authoring.json` holds behavior groups, which the text does not
express. A group whose members are gone from the text loses them.

## What is in the layout file

Positions of concepts, relationships, outputs, instances and behavior
boxes; collapse state; pan and zoom per canvas. A project opens without
this file; deleting it loses nothing but placement. Every item that has
no position is given one when the project opens and whenever an item is
made — in the column of its kind, beside what it reads or produces —
and nothing that has a position is moved ([Canvas](../studio/canvas.md)).

## What is not saved

- the chosen **board** on the Deploy page — a session preference;
- the **simulation** — inputs, periods and trace;
- the **analysis** — every verdict is recomputed on open;
- the **revision history** — undo is per session;
- where you were — the view, the page, the open file, the component whose
  source was open — Studio notes that for itself, per user, and returns
  you there on the next open.

Everything you can see and edit is in the project. A formula you typed
and did not add — even one that does not check yet, even an empty field
— is saved as it is and comes back in the editor. Text in the Code view
that does not build yet is saved exactly as typed: the file on disk is
what you typed, and the design keeps showing the last version of it that
built until it builds again ([Design, Code and Split](../studio/code-view.md)).

## Saving

**⌘S** or **Save** writes the whole current state: the changed source
files as item-level edits — a save changes the items you changed and
leaves the rest of the text, comments, blank lines and your ordering as
it was; a file that does not build yet is written as typed — then the
sidecars (identities, groups, your unfinished formulas and the last good
text of a file that does not build), the layout and the manifest, each
through a temporary file and an atomic rename, so an interrupted save
never leaves a half-written project. Saving never needs a formula to
check or a file to build. _Saved_ in the status line means every one of
those files was written; until then the project is _Edited_. New
items made on the canvas go to the end of `src/main.bdl` (or of the first
file in path order when there is no `main.bdl`), or to the end of the
component body they belong to. Newer file versions are refused rather
than misread.

Saving checks that no source file changed on disk since it was read; if
one did, Studio refuses and offers to reload or overwrite
([Authoring a project as text](../workflows/authoring-as-text.md)).

## Older projects

Projects saved by earlier versions of Studio kept the design in a JSON
file (`design/project.bdl.json` or `design/system.bdl.json`) and said
so in `bdl.toml`. Opening one — in Studio, from the command line or
with the language server — converts it in place, once:

- the design is written to `src/main.bdl`;
- every item keeps its identity, so positions, colours and group
  membership are unchanged;
- a name the files cannot spell is respelled (`Light Output` becomes
  `Light_Output`, a word of the language gets a trailing `_`);
- the JSON file is renamed `….migrated` and is never read again; delete
  it when you are sure, or keep it as a record;
- `bdl.toml` is rewritten.

A folder that has both a JSON design file and `.bdl` files under `src/`
is refused rather than guessed at: keep one of the two. There is no way
back to the JSON form.

## Related

[Design, Code and Split](../studio/code-view.md) ·
[Authoring a project as text](../workflows/authoring-as-text.md) ·
[Textual BDL](../textual/overview.md)
