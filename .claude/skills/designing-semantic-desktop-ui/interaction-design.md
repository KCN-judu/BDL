# Interaction design

Sources: Don Norman, *The Design of Everyday Things*; Alan Cooper et al., *About Face*;
Jenifer Tidwell et al., *Designing Interfaces*; Steve Krug, *Don't Make Me Think*. Own
words, applied to a desktop tool with a canvas.

## Norman's vocabulary — use it to name the problem

- **Affordance** — what an object *can* do (a socket can be dragged from). Real, not
  perceived.
- **Signifier** — what tells the user the affordance exists (the socket's dot on the
  node edge, the crosshair cursor on hover). Every affordance the designer needs must
  have a signifier at rest or on hover; a hidden gesture is a hidden feature.
- **Mapping** — the spatial correspondence between control and effect. Left is input,
  right is output; the inspector edits the selected node; a delete button sits with the
  thing it deletes. Good mapping needs no label.
- **Constraint** — what the object refuses to do. Physical (a link only reaches a
  socket), semantic (only the same identity), logical (one driver per output). The
  kernel's rules become constraints *in the gesture*, so the mistake cannot be made.
- **Feedback** — the immediate, proportionate answer to every action: hover halo, drag
  ghost, drop snap, refused-link retraction, commit note. Delayed or global feedback (a
  banner for a local mistake) breaks the loop.
- **Conceptual model** — the story the user tells about how it works. The UI's shapes
  must tell the kernel's story: declarations that exist before definitions, edges that
  are dependency, domains that are containers, outputs that have one owner.
- **Gulf of execution / evaluation** — "what can I do?" / "what happened?". Every
  screen should shrink both: visible actions, visible results.
- **Slips vs mistakes** — slips are handled by undo and forgiving targets; mistakes by a
  conceptual model that matches. Do not treat a slip with a confirmation dialog.

## Cooper — goal-directed, for the perpetual intermediate

- **Posture.** Studio is a *sovereign* application: full screen, long sessions,
  dense, conservative colour, rich keyboard. Not a transient utility, not a web page.
- **Perpetual intermediates.** Most users are neither novices nor experts most of the
  time. Design the default path for them; give novices sheets that teach by showing and
  experts keyboard, direct manipulation and no lectures. Do not put the novice's
  explanation in the intermediate's face on every visit.
- **Object–verb.** Select, then act. The inspector is the verb list for the selection;
  the contextual menu repeats it; the menu bar holds the complete set with shortcuts.
- **Direct manipulation** for structure (drag to link, drag to move, drag away to
  disconnect); forms for properties (name, kind, formula). Never a form for something
  that is naturally a drag, never a drag for something that is naturally a value.
- **Modeless feedback.** Rich, continuous, non-interrupting: the state word on the node,
  the outcome line in the inspector, the status cell. Modal alerts are a design failure
  except for real loss of data.
- **Eliminate excise.** Any step that serves the software rather than the goal — a
  confirmation, a save-before-you-can, a "select a concept first" — is excise. Undo
  instead of confirm; autosave-style dirtiness; disabled controls that explain
  themselves at rest.
- **Considerate software** remembers (recent projects, panel widths, last page), offers
  good defaults but never a *wrong* silent default (a mapping that produces the first
  concept in the list), and does not ask what it can infer.
- **Inflection.** Frequent actions are one gesture away; rare ones are in the menu; the
  toolbar shows the frequent verbs, not every verb.

## Tidwell — the patterns that apply

| Pattern | What it is | In Studio |
|---|---|---|
| **Canvas plus palette** | a work surface with a source of new objects | canvas + library (later: drag from library) |
| **Master–detail / two-panel selector** | list on one side, selection's detail on the other | library ↔ inspector, with the canvas as the third, spatial view of the same selection |
| **Inspector (property panel)** | edits the selection; sections; disclosure | the inspector: sections, form rows, one `EditOp` per control |
| **Overview plus detail** | a small whole and a large part | status-line counts ↔ canvas; frame-all; later a minimap only if the canvas outgrows the window |
| **Progressive disclosure** | show more only when the user asks or the state needs it | dimension row appears when kind = quantity; technical details behind the diagnostic |
| **Responsive disclosure** | the form changes as choices are made | the sheet's live node preview |
| **Datatips** | hover reveals detail | full name, initial value, sample age — never something needed at rest |
| **Preview** | show the result before committing | the node preview in sheets; the consequence line before an edit |
| **Good defaults / forgiving format** | sensible prefill; accept loose input | "Decide later" as default kind; formula parser tolerant of spacing |
| **Input hint / structured format** | a hint inside or beside the field | "expression over Tilt, Held" |
| **Escape hatch** | always a way back | Esc clears selection, Cancel on the left, undo |
| **Local tools** | tools appear near the object | contextual menu on a node; remove × on a chip |
| **Clear entry points** | an empty screen says what to do first | empty canvas → "add a concept from the library" |
| **Alternative views** | same objects, different arrangement | pages: Design / Simulate / Deploy / Monitor over the same canvas |

Patterns to avoid here: wizards (excise), dashboards (metadata soup), modal tours,
notification toasts (snackbars) for anything that could be a state on the object.

## Krug — obviousness

- **Don't make me think.** Every element answers "what is this, what can I do with it,
  what state is it in" without reading. If a tooltip is needed to understand a control,
  the control is wrong (a tooltip may still add the shortcut).
- **People scan, satisfice and muddle through.** Design for the scan: a visual hierarchy
  that ranks, obvious clickables, clearly defined regions, little text.
- **Conventions are your friend.** Reinvent nothing the platform already answers.
- **Omit needless words.** Halve the prose, then halve it again. Instructions that are
  the same for every object are documentation.
- **The trunk test.** Dropped into any screen: which page am I on, what is selected,
  what can I do, how do I get back? All four must be answerable from the screen.

## Feedback and state rules for Studio

- Hover: 6 % overlay or a 4 pt halo; pressed: 12 %; focus: 2 pt accent ring (keyboard
  only); selected: accent at 20–25 %; disabled: 40 % opacity, and *the reason visible at
  rest* (a line under the button, not only a tooltip).
- A refused action answers where the pointer is: the link retracts, the socket shows a
  blocked cursor and the reason in a datatip, the drop target never highlights.
- A request in flight is a small indeterminate indicator in the status line and, if the
  object is affected, a lighter treatment of that object — never a blocking overlay.
- An error from the compiler is a banner above the content when it is global (stale
  revision, connection) and a mark on the object when it is local (a formula that does
  not type).
- After a semantic edit the inspector says, in one line, what kind of change it was and
  which other objects it reopened — ideally *before* the commit for edits with
  consequences ("changing this re-checks 2 mappings").

## Checks

- For every affordance on the screen: what is its signifier? Where is its feedback?
- For every kernel rule this screen touches: is it a constraint in the gesture, a local
  refusal, or a diagnostic afterwards? Move it up the list if you can.
- Which step here is excise? Remove it or replace it with undo.
- Is any prose the same for every object? Then it is documentation, not interface.
