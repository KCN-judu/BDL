# Documentation research

What was studied before the user guide was written, organised by the
technique learned rather than by source. Sources are official manuals or
official training books; structure and method were analysed, no prose was
reused.

## Sources

| Source | Official URL | Studied |
|---|---|---|
| Blender Manual | https://docs.blender.org/manual/en/latest/ | top-level sections; *User Interface › Nodes › Node Parts* |
| Blender Manual — Writing Style Guide | https://docs.blender.org/manual/en/latest/contribute/manual/guides/writing_guide.html | the manual's own rules for contributors |
| The Beginner's Guide to DaVinci Resolve 17 (Blackmagic Design official training book) | https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-17-Beginners-Guide.pdf | front matter, Getting Started, Lesson 1 |
| Apple Human Interface Guidelines | https://developer.apple.com/design/human-interface-guidelines/ and …/writing | the Foundations / Patterns / Components split; the *Writing* page |
| Visual Studio Code docs | https://code.visualstudio.com/docs, …/docs/getstarted/getting-started, …/docs/getstarted/userinterface | landing page, the getting-started tutorial, the *User Interface* page |
| KiCad — Getting Started in KiCad 9.0 | https://docs.kicad.org/9.0/en/getting_started_in_kicad/getting_started_in_kicad.html | *Basic Concepts and Workflow*, *Electrical Rules Check* |
| Godot Engine docs | https://docs.godotengine.org/en/stable/ and …/getting_started/introduction/key_concepts_overview.html | the landing page's reader tiles; the key-concepts page |
| Figma Help — Guide to components | https://help.figma.com/hc/en-us/articles/360038662654-Guide-to-components-in-Figma | how a "main component vs instance" idea is introduced |

## Patterns observed

### Task-first onboarding

*Resolve, VS Code, KiCad.* The first substantial page is a task the reader
completes, not a description of the software. Resolve's Lesson 1 opens with
*Time* and *Goals*, then numbered steps in the second person; every step is
followed by one or two sentences saying what is now on screen and, only when
needed, why. Concepts (a *bin*, the *media pool*) are defined in the sentence
where they first matter — "Bins, like folders, …" — never in a glossary
first. KiCad's Getting Started is one long tutorial (project → schematic →
board → custom parts) with a short *Basic Concepts and Workflow* section
placed before the tutorial, three paragraphs, no jargon.

What BDL adopts: a *First behavior* tutorial as the central page; steps in
the imperative; each step followed by what the screen now shows; concepts
introduced at the moment they are needed; a short "what did you just
make" paragraph after each stage.

### Concept / reference separation

*Blender, Apple HIG, Godot.* Blender has three tiers: Getting Started;
reference of the interface (*User Interface*, *Editors*); feature and
workflow chapters. Apple separates *Foundations* (principles), *Patterns*
(task-oriented: how people accomplish something) and *Components*
(object-oriented: one control, its states). Godot separates Getting
Started, the Manual, and the class reference, and its *key concepts*
page is four concepts, one short section each, with the promise that
questions will be answered in the tutorials.

What BDL adopts: `concepts/` (ideas, one per page), `studio/` (the
interface, one panel or page per file — object-oriented), `workflows/`
(task-oriented recipes), `reference/` (tables). A page is one of these,
never two.

### Progressive disclosure

*Godot, VS Code, Blender.* Godot's landing page asks the reader to pick a
profile ("I've never made a game", "I know how to make a game, I want to
learn Godot") and routes accordingly. VS Code's *User Interface* page
starts with one annotated overview figure and six bold region names,
then goes deeper only through links. Blender's writing guide says to avoid
explaining the algorithm when a simpler explanation exists, and to put
implementation-level material where the reader has asked for it.

What BDL adopts: reader profiles on the landing page; every concept page
has a "Going deeper" tail that links to the technical documentation and
is labelled *for language implementers* / *formal reference*; formal
vocabulary (`DeclId`, elaboration, judgments) appears only in those tails
and in the Explain sections of the Studio pages.

### UI vocabulary

*Apple HIG Writing, Blender Node Parts, VS Code User Interface.* Regions
and controls are named in bold when introduced and then used with exactly
that name (VS Code: **Activity Bar**, **Primary Side Bar**). Blender's node
anatomy page names the parts (title, sockets, …) before any behaviour, and
enumerates visual encodings (socket colour = data type, socket shape =
data structure) as definition lists. Apple's guidance: one term per
concept, no synonyms, the app's own labels.

What BDL adopts: a terminology table (`reference/terminology.md`) that
fixes one word per idea and marks which words are Studio labels; UI
labels are written as they appear (*Updates in*, *Package as Reusable
Component…*); node anatomy explained parts-first with an encoding table
(hue = which concept, shape = value form, dashed = declared).

### Screenshot discipline

*Resolve, VS Code, Blender.* Resolve uses one annotated screenshot per
panel introduction, callouts naming each region, and otherwise small
crops beside the step they illustrate. VS Code uses one overview figure
per page and animated captures only for gestures. Blender captions every
figure with what it shows ("How a node appears when collapsed").
Blender's writing guide warns that details tied to a release rot.

What BDL adopts: no screenshots in the first version; figure *slots* with
a required state, crop, caption and alt text recorded in
`SCREENSHOT_PLAN.md` so images can be added by an automated capture
later; ASCII/annotated diagrams for anatomy where they are stable
(workspace, node, instance node).

### Error explanation

*KiCad ERC, Apple HIG.* KiCad's tutorial reproduces the checker's exact
message, then says why the rule exists ("to a human it is obvious … but
it is necessary to show it explicitly"), then the one fix, then re-runs
the check. Apple's writing guidance for errors: say what happened in the
person's terms, then what to do; no blame.

What BDL adopts: every troubleshooting entry is *what you see* → *what it
means in your design* → *why BDL insists* → *what to do*, with the
compiler's code in a final line for search. Diagnostics are never the
headline.

### Navigation

*Blender, KiCad, Godot.* A stable sidebar tree; every page ends with
"See also" (Blender) or "Where to go from here" (KiCad); tutorials say
what comes next. Godot's tiles route by reader, not by feature.

What BDL adopts: `README.md` as the single entry with routes by reader;
every tutorial ends with *Next*; every concept and workflow page ends
with *Related*; no page links to more than a handful of others in its body.

### Advanced-user pathways

*Blender, VS Code.* Blender keeps a *Contribute* and *Advanced* section
apart from the manual; VS Code keeps language-specific and setup material
in their own trees; both let an expert bypass the tutorial.

What BDL adopts: `textual/` for code-oriented users with an explicit
*Current status* section; `reference/` for lookups; pointers into
`docs/*.md` and `docs/decisions/` from "Going deeper" tails; the researcher
profile on the landing page routes straight there.

### Writing style

*Blender writing guide, Apple HIG Writing.* Short sentences; American
spelling; no first person; no weasel words ("probably fixes"); no
release-relative statements ("since version X"); no counts that rot ("23
modifiers"); do not copy tooltips; include *why* an option is useful.
Apple: second person for instructions, active voice, consistent terms,
plain language, define specialised terms in context.

What BDL adopts: second person for tutorials and workflows ("you"),
impersonal for concept and reference pages; imperative steps; one idea
per sentence; the *why* beside each *how*; no version-relative
statements; current facts only, planned work labelled as such.

## Documentation principles adopted for BDL

1. The tutorial is the front door; concepts are taught when the reader
   needs them and again, properly, in `concepts/`.
2. One page is one kind of page: concept, interface, workflow, reference,
   or troubleshooting.
3. Designer vocabulary first; the Studio label is the word; kernel and
   compiler vocabulary only under *Going deeper* or *Explain*.
4. A workflow page answers: goal, steps, what you just made in BDL terms,
   what to check when it does not work.
5. An error is explained as the reader's situation, then the rule, then
   the fix, with the code last.
6. Current behaviour only. Planned work is named as planned, in one
   sentence, never described as if it existed.
7. Examples build on one product — the tilt lamp — from the first tutorial
   through components and systems.
8. Figures are planned, captioned and alt-texted before they are taken;
   anatomy is drawn as a diagram where a screenshot would rot.
9. Every page ends with where to go next; the landing page routes by
   reader.
10. Every factual claim about the software has a verification row
    (`VERIFICATION.md`).

## Patterns deliberately rejected

* **A glossary before the tutorial.** Godot and Resolve define terms in
  the moment; BDL's terminology page is a reference, not the front door.
* **Documenting every option.** Blender's guide: lists that mirror the
  interface are cost without value. BDL documents what an option means
  and when to choose it, not each menu entry.
* **Screenshots as explanation.** A screenshot shows where; it does not
  say what a mark means. BDL explains meaning in text and reserves images
  for location.
* **The reference manual as first contact.** Resolve's 3,000-page
  reference manual is the last resort behind the training books; BDL's
  `docs/*.md` and ADRs play that role and are linked, not repeated.
* **Version-relative prose** ("new in", "since") and change logs inside the
  manual — Blender's guide leaves those to release notes.
* **Impersonal voice in tutorials.** Blender's UI-text guideline avoids
  "you"; Apple and Resolve use it for instructions. BDL follows Apple
  and Resolve for tutorials and workflows, Blender for concept and
  reference pages.
* **Marketing framing.** No page states that BDL is complete, verified
  end to end, or production-ready; the guide describes what the tool does.
