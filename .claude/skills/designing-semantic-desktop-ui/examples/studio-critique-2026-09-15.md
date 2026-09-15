# Applying the skill to BDL Studio — critique of 2026-09-15

A worked example of `critique-checklist.md` against the code at commit 998376a
(`apps/studio/lib/ui`). Question numbers refer to the checklist; principle numbers to
`principles.md`. Findings are ranked by harm to the designer's task, not by effort.
Nothing here has been implemented; it is the test of whether the skill produces
judgment. Re-run it after changes; do not treat it as a permanent list.

## Welcome (`welcome/welcome_page.dart`, `welcome/hero_mark.dart`)

**Task:** open the project I was working on, or start one, in under three seconds.

**Strengths**
- Start / Recent two-column layout; OS pickers; typed-path fallback only when the OS
  refuses (q42, q43, principle 20).
- Recent rows: name semibold, path tertiary, "when" in a fixed right-aligned column with
  tabular figures; missing projects greyed with an icon (q11, q16, principle 15).
- Empty Recent state is a sentence, not a hero (q46).

**Violations**
1. *Hover hides a fact* — `welcome_page.dart:264`: the "2 h ago" cell is replaced by × on
   hover. q30/q35, anti-pattern "hover that hides". Keep the time; add a trailing 22 pt
   column for × that appears on hover, or use the contextual menu.
2. *Compiler term in designer prose* — `:148` "Waiting for the compiler service (bdld)".
   q21. Say "Waiting for the compiler" and let the status line carry the detail.
3. *Orange overloaded* — `:254` "not found" in `t.open`. Orange means "a decision is
   open"; a missing folder is not that. q34, principle 8. Tertiary text; the folder-off
   icon is already the partner cue.
4. *Hero proportion* — the hero is an `AspectRatio(1.15)` filling the left column; at a
   1360 × 820 window it is ~450 pt tall and Start sits under it. Acceptable on a
   launcher (Xcode does the same), but keep it capped so Start is never below the
   fold at 760 pt height. q45. Not a change request; a constraint to keep.

**Keep:** the layout, the recent-row table, the OS-picker policy.

## Design canvas (`canvas/canvas_geometry.dart`, `canvas/node_canvas.dart`)

**Task:** see what exists, what depends on what, what is still open; connect things.

**Strengths**
- Socket + link hue = identity; hollow = unbound; dashed = undefined; header tint =
  category at low saturation; left → right; layout separate from semantics (q19, q20,
  q23, principles 2, 8).
- Link legality by identity enforced in `dropTarget` (q29, principle 3).
- Hover/selection/drag treatments follow the one standard; frame-all; Esc/Delete.

**Violations**
1. *Selection erases the state encoding* — `node_canvas.dart:401`
   `if (n.unresolved && !isSelected) dashed … else solid`. A selected undefined node is
   drawn solid. q35, node-editor rule 5. Draw the dashed outline in accent at 2 px.
2. *Identity hues fail contrast in light theme* — `tokens.dart` `conceptColor`:
   HSL(h, 0.55, 0.48) gives 2.0–2.5:1 against `#F2F2F4` for hues near 50–200° (ids 1,
   3, 4, 6, 9, 11 of the first twelve); dark theme is fine. q39, accessibility. Choose
   lightness per hue to hit a fixed relative luminance (or generate in OKLCH at a fixed
   L), and verify the worst three hues in both themes. Keep the golden-angle sequence.
3. *No feedback for an illegal drop* — `_onPanEnd`: an incompatible target simply does
   nothing; during the drag nothing shows which sockets are legal. q26, q29, principle
   3–4. While dragging: faint halo on every compatible socket, strong halo on the one
   under the pointer, blocked cursor over an incompatible one, 120 ms retract on
   release.
4. *Representation shown as text, not shape* — `canvas_geometry.dart:351–357`
   "representation not chosen" / "quantity rad" as a 10.5 pt tertiary subtitle on the
   concept node, while `STUDIO_UI.md` §2 specifies socket shape ○ ◇ □. q17, q19, q22.
   Implement the socket shape (also in library rows, chips, toggles via one drawing);
   drop the subtitle; the hollow ring + *open* already covers "not chosen".
5. *Three encodings of one fact on an open concept* — hollow socket, the word *open*,
   and the subtitle. q13, information-design (redundant data-ink). Keep two: the ring
   and the word.
6. *Arbitrary shadow* — `node_canvas.dart:375–378` a blurred drop shadow under every
   node. Blender's nodes are flat; macOS chrome is flat; the fill + hairline already
   separate figure from ground. q12, anti-pattern "arbitrary shadows". Remove, or keep
   only under a node being dragged (elevation with a reason).
7. *No empty state with a project open* — `design_page.dart:64` handles no project only;
   a project with zero nodes shows a bare grid. q46, Tidwell "clear entry point". One
   tertiary line at the centre: "Add a concept from the Library".
8. *Keyboard reach on the canvas* — only ⌫, Esc, Home/⌘0. No Tab between nodes, no
   arrow nudge, no keyboard route to link or rename. q37, accessibility. Add Tab/⇧Tab
   through nodes in reading order, arrows to nudge, Return to rename, a "Connect to…"
   command in the contextual menu.
9. *Undriven concepts are not distinguishable from driven ones* — a concept with no
   producing mapping is an external input (a sensor, a supplied trace in simulation),
   which matters for the state ladder and for Simulate. q4. Proposed: the concept's
   input socket is hollow-ringed when nothing produces it (fill = driven, as for
   outputs), consistent with the fill/hollow channel meaning "bound/driven".
10. *Painter exposes no semantics* — `_CanvasPainter` has no `semanticsBuilder`; the
    graph is invisible to VoiceOver. q41. Add semantics nodes per canvas node.

**Keep:** identity-by-hue with the name partner, dashed-not-red, drop legality by
identity, no auto-insert, no operator nodes, position-only layout.

## Concept inspector (`inspector.dart:141–248`)

**Task:** name, describe and fix the kind of a concept; see who uses it; delete it.

**Strengths**
- Form with right-aligned 78 pt labels; commit-on-blur/Return; sections with 11 pt
  semibold titles; "Used by" as a `MacTable` with *produces*/*reads*; named destructive
  button; disabled while used (q16, q27, q29).

**Violations**
1. *Internal id shown* — `:158` "identity 3" at 10 pt tertiary in the section header. q3,
   q21, anti-pattern "compiler terms". Remove.
2. *Two vocabularies for one fact* — `:279` segmented "none / quantity / boolean / count"
   vs the sheet's "Quantity / On–off / Count / Decide later". q24, contract §4. One set,
   the sheet's, in the same order.
3. *Parentheses as layout* — `:255` "angle (rad)" etc. in the dimension pop-up, while the
   sheet uses `detailOf` for a unit column. q18. Use `detailOf` here too; share the one
   dimension table between sheet and inspector.
4. *Static lecture* — `:191` and the "Open. Mappings may already use this concept…"
   paragraph are identical for every concept. q5, anti-pattern "prose that lectures".
   Replace with one dynamic line only when it applies: "Used by dimByTilt, lampTarget —
   changing the kind re-checks them." Nothing when unused.
5. *Disabled reason only in a tooltip* — `:237`. q30. Put "used by dimByTilt" under the
   button in secondary 11 pt, with the names as links that select the mapping.
6. *"Used by" rows lack the identity glyph the library rows have* — minor consistency
   (q33); add the socket dot in the output-concept hue as the library does.

## Mapping inspector (`inspector.dart:302–498`)

**Task:** see how far this relationship is from runnable; fix its signature; attach or
change its definition.

**Strengths**
- Reads as chips in identity hue with ×, Produces as a pop-up, formula in monospace,
  input hint "expression over Tilt, Held", Attach as a primary button, detach
  explicit (q25, q28, Tidwell input hint).

**Violations**
1. *State shown as a pill in a form row* — `:326` `FormRow(label: 'State', StatePill)`.
   The ladder (declared → defined → type-valid → temporally valid → clock-consistent →
   output-complete) is the designer's model of "how done"; a single pill hides where
   the object stands and what is missing next. q19, q22, contract §3. Replace with a
   ladder control: rungs in order, reached ones solid, the next one with one sentence on
   what is missing ("no definition yet"). Keep the header word on the canvas as the
   overview.
2. *Produces pop-up lacks the identity glyph* while Reads chips have it — `:379`. q33.
   Add the socket glyph in the pop-up's rows and value.
3. *Static lectures* — `:414`, `:426`, `:462`. q5. Replace with one dynamic consequence
   line at the moment of action ("Re-checks 2 dependents") — the Last change note
   already gives the after-the-fact version.
4. *Internal id* — `:344`. Remove.
5. *No home for diagnostics* — when `bdl-check` lands, a non-typing formula needs a
   place. q4, Xcode pattern. Reserve: the diagnostic sits directly under the formula
   field, error red icon + one product-language sentence with the fix, with a
   "Details" disclosure for the kernel vocabulary.
6. *Delete without confirmation* — `:490`. Acceptable: the button is named and undo
   exists (principle 6). Confirm that undo restores layout position too; if not, that is
   the fix, not a dialog.

## Creation sheets (`dialogs.dart`)

**Task:** create a concept or a mapping and understand what I just made.

**Strengths**
- Live node preview painted by the canvas painter; captions that state what the socket
  means; product vocabulary (Quantity / On–off / Count / Decide later; Reads /
  Produces; Meaning); unit in its own column; Cancel left of Create; Return submits;
  Create disabled until named (q24, q43, principle 5, Tidwell preview).

**Violations**
1. *Silent wrong default* — `:278` `_output = widget.concepts.first`. A mapping can be
   created producing an input it reads, without a choice. q32, anti-pattern. No default:
   pop-up reads "choose", Create disabled until chosen; the preview's output socket is a
   hollow ring meanwhile.
2. *Look-alikes without focus* — `MacSegmented` (`widgets.dart:264`) and `_ConceptToggle`
   (`dialogs.dart:379`) are `GestureDetector`s: not focusable, no ring, no arrow keys.
   q37, q38, accessibility. Wrap in `FocusableActionDetector`; arrows move the
   segment, Space toggles a concept.
3. *Sheet is a centred dialog, not a window sheet* — `showMacSheet` uses `showDialog`
   with a barrier. HIG sheets attach to the window's title bar. q42. Low priority
   (Flutter has no native sheet), but the barrier should be lighter and the dialog
   positioned at the top of the window to read as a sheet.
4. *Concept toggles use identity hue as "selected"* — deliberate and correct (membership,
   not selection), but undocumented. q34. Add the exception to `semantic-ui.md`'s table
   note: "a chip filled in its identity hue means *member of this signature*".

## Status line (`shell.dart:185–252`)

**Task:** at a glance — is my work saved, is anything open, is the compiler there?

**Strengths**
- Cells separated by whitespace with `gapGroup`; tabular figures; connection dot + text;
  plural handling; pending spinner small (q18, principle 15).

**Violations**
1. *Leading cell is a compiler counter* — `:203` `r12`. Revisions are session-only, not
   persisted, and mean nothing to a designer; the glance fact is the document state.
   q1–q3, q21. Lead with "Saved" / "Edited"; then "1 relationship awaiting definition"
   (clickable → select/frame those nodes, which makes it an overview rather than a
   duplicate); move the revision to a technical-details view.
2. *Daemon and protocol names* — `:219–220` "bdld 0.1.0", "protocol 0.1.0". q21. Show
   "Compiler 0.1.0"; show the protocol only on mismatch or failure, in the banner.
3. *The hovered/selected object's state phrase* promised in `STUDIO_UI.md` §1 is not
   there. q4. Add a cell: "dimByTilt — declared, reads Tilt".

## Shell, toolbar, page bar, library (`shell.dart`, `library.dart`)

1. *Punctuation layout* — `shell.dart:131` "— Edited". q18. A separate cell in tertiary,
   or the platform's dirty dot in the title.
2. *Page bar labels at 10 pt* — `:357`. q17. 11 pt; the icons are Material glyphs
   standing in for SF Symbols — acceptable until symbols are bundled, but each page
   button must keep its word.
3. *Settings button dispatches ConnectRequested* — `:319`. A mapping violation
   (Norman): the gear is not "reconnect". Make it project settings or remove until it is.
4. *Close in the toolbar duplicates ⊞ in the page bar* — two controls, one action. q12.
   Keep one (the page bar's, as in Resolve).
5. *Library trailing state word at 10 pt tertiary* — `library.dart:155`. q17. 11 pt
   secondary, right-aligned as a column.
6. *One glyph, two meanings in the library* — a hollow dot means "kind not chosen" on a
   concept row and "no definition" on a mapping row; a mapping row's dot is in its
   output concept's hue, so it looks like a concept row. q34. Mapping rows should show
   the mapping's silhouette (a small dashed/solid rectangle) rather than a socket dot,
   or a dashed ring for undefined; the socket dot stays a concept-only glyph.

## What to change first (ranked by harm to the task)

1. Dashed outline survives selection (`node_canvas.dart:401`) — one line, fixes a
   channel collision on the most important state.
2. Identity-hue contrast in the light theme (`tokens.dart` `conceptColor`) — the
   identity channel is unreadable for half the concepts.
3. Illegal-drop feedback and legal-target popout during a link drag.
4. Remove `identity N`; unify the representation vocabulary and the dimension table
   between sheet and inspector.
5. Disabled Delete shows its reason at rest.
6. Focusability of `MacSegmented` and concept toggles.
7. Socket shape by representation, replacing the subtitle text.
8. Status line: document state first, product names for the compiler, protocol on
   mismatch only.
9. Mapping sheet: no silent Produces default.
10. Empty-canvas entry point.

## What should not change

- The Resolve page structure, the Blender canvas anatomy, the HIG look; the three
  reference policy in ADR-0012.
- Socket and link hue as identity with the name as partner; hollow for unbound;
  dashed for undefined; red reserved for errors.
- Link legality enforced at the pointer by identity; no auto-insert; no operator nodes;
  layout kept out of the design file.
- The whitespace-not-punctuation rules of §3a, `MacTable`, `FormRow`, the 78 pt label
  column, the 8 pt grid.
- The interaction-state standard (`MacStates`, `MacInteractive`) — extend it to the
  look-alikes, do not fork it.
- Sheets that teach by showing the node; product vocabulary in sheets.
- OS pickers with a typed fallback only when refused.
- The token system and the two-theme discipline.
