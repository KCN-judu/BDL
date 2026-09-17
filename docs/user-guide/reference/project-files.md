# Project files

A project is a folder of plain files. Studio reads and writes them; you
can put the folder under version control, copy it, or diff it. The
authoritative description for tool builders is `docs/PROJECT_FORMAT.md`.

```
lamp/
├── bdl.toml                 the manifest: name, schema version, kind (flat, system or text)
├── design/
│   ├── project.bdl.json     a flat project's design
│   └── system.bdl.json      a system project's design (one of the two exists, never both)
└── ui/
    └── layout.json          where things are on the canvas — not part of the design
```

A **text project** (`kind = "text"`) has no design JSON; its design is
the `.bdl` files:

```
lamp/
├── bdl.toml                 kind = "text"
├── src/
│   ├── concepts.bdl         any number of .bdl files, any names, read in path order
│   └── main.bdl
├── .bdl/
│   ├── identities.json      the stable identity of every item — owned by the tools
│   └── authoring.json       behavior groups — owned by the tools
└── ui/
    └── layout.json          as above
```

## What is in the design file

Everything that means something: concepts (name, description, value
form), relationships (name, description, reads, produces, formula, timing
domain, the output it drives), timing domains, physical outputs (name,
what it accepts, domain, required), devices (name, kind, output, fixed
pins), and the counters that allocate identities. In a system project
also: components (each with its own design and its ports), instances,
bindings, behavior groups, and the table that gives each instance its own
identities.

Every object has a stable numeric identity that never changes and is
never reused; the **name is a label**. Renaming touches nothing but the
label. Two objects with the same name in two projects have nothing to do
with each other.

## What is in the layout file

Positions of concepts, relationships, outputs, instances and behavior
boxes; collapse state; pan and zoom per canvas. A project opens without
this file; deleting it loses nothing but placement.

## What is not saved

* the chosen **board** on the Deploy page — a session preference;
* the **simulation** — inputs, periods and trace;
* the **analysis** — every verdict is recomputed on open;
* the **revision history** — undo is per session.

Formula **drafts** are not in the project either; Studio keeps unsaved
drafts on its side, by project path, and restores them when you reopen.

## Saving

**⌘S** or **Save** writes the design file first, then the manifest, each
through a temporary file and an atomic rename, so an interrupted save
never leaves a half-written project. Newer file versions are refused
rather than misread; older ones are migrated on open. A text project
writes the changed source files, then the sidecars, the layout and the
manifest, the same way.

## Text projects

The `.bdl` files are the design ([Syntax basics](../textual/syntax-basics.md)).
Studio and the language server read them with one loader and write them
back as item-level edits: a save changes the items you changed and
leaves the rest of the text — comments, blank lines, your ordering — as
it was. New items Studio creates go to the end of `src/main.bdl` (or of the
first file in path order when there is no `main.bdl`), or to the end of
the component body they belong to.

`.bdl/identities.json` records which identity each item has, by its
kind and name, together with the counters that hand out new ones. It is
written on open when the sources needed identities the file did not
have, after every save, and by the language server after a save in the
editor. Keep it under version control with the sources; do not edit it.
Deleting it gives every item a fresh identity — nothing semantic is lost,
but canvas positions and group membership, which are keyed by identity,
are.

`.bdl/authoring.json` holds behavior groups, which the text does not
express.

Saving from Studio checks that no source file changed on disk since it
was read; if one did, Studio refuses and offers to reload or overwrite
([Authoring a project as text](../workflows/authoring-as-text.md)).
