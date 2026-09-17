# Project files

A project is a folder of plain files. Studio reads and writes them; you
can put the folder under version control, copy it, or diff it. The
authoritative description for tool builders is `docs/PROJECT_FORMAT.md`.

```
lamp/
├── bdl.toml                 the manifest: name, schema version, kind (flat or system)
├── design/
│   ├── project.bdl.json     a plain project's design
│   └── system.bdl.json      a system project's design (one of the two exists, never both)
└── ui/
    └── layout.json          where things are on the canvas — not part of the design
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
rather than misread; older ones are migrated on open.

## Textual files

A `.bdl` text file is **not** part of a project. It is a document a
language server checks against a project ([Textual BDL](../textual/overview.md)).
