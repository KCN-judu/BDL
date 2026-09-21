# Keyboard and mouse

Shortcuts are listed for macOS. On Windows the same keys are shown with
Ctrl in the interface, but the bindings are not yet active there — use
the toolbar and menus.

## Application

| Keys | Does |
| --- | --- |
| ⌘S | Save the project — everything as it is, unfinished formulas and text that does not build included |
| ⌘W | Close the project — asks _Save changes?_ when it is edited |
| ⌘Q | Quit — the same question first |
| ⌘Z / ⇧⌘Z | Undo / Redo — design edits and behavior-group edits, one history |
| ⌘1 · ⌘2 · ⌘3 · ⌘4 | Design · Simulate · Deploy · Monitor |

_New Project…_ (⌘N) and _Open Project…_ (⌘O) on the project manager show
their shortcuts but are reached by clicking today.

## Canvas

| Do | Result |
| --- | --- |
| drag empty canvas, left → right | select the nodes wholly inside (window) |
| drag empty canvas, right → left | select the nodes inside or touched (crossing) |
| ⌘-drag / ⇧-drag a rectangle | add its nodes to the selection / remove them |
| middle-button drag · Space + drag · two fingers on a trackpad | pan |
| scroll wheel · pinch · ⌘ + two fingers | zoom about the pointer |
| Home · ⌘0 | frame all |
| click | select one |
| ⌘-click (Ctrl on Windows and Linux) | add to / remove from the selection |
| ⇧-click | select the chain of connections from the active node, when there is one |
| ⌘A | select every node in view |
| Esc | cancel the gesture in progress; then clear the selection |
| ← → ↑ ↓ (⇧: one point) | nudge the selection |
| drag a node | move the selection (layout only) |
| drag a block's output socket onto an output | make it the driver |
| drag a block's output socket onto a block with no formula, onto a `?` socket, or onto a mapping block with one | the formula gets the block's name (a text edit) |
| drag socket → socket | link (a binding, a drive) |
| drag a driven output's socket, or a mapping block's input socket, away → empty canvas | disconnect (a read link: the name becomes `?`) |
| ⌫ / Delete | delete the selection (or disconnect a selected link) |
| double-click a block or its mapping block | rename the block in place |
| double-click an instance | open its component's source |
| double-click a behavior's title | rename |
| right-click · Control-click | context menu |
| drag a Library row onto the canvas | insert a concept |

## Formula field

| Keys | Does |
| --- | --- |
| ⌘↩ | Add / Save the definition (in the formula sheet: and close it) |
| Esc | Revert the draft; with completion open, close it first; in the formula sheet, close it with the draft kept |
| ⌘E | open the formula sheet on this relationship |
| Return | new line |
| ⌃Space | completion (in the Formula view it also opens as you type a name) |
| ↑ / ↓ · Return / Tab | move in the completion list · accept |

## Formula view

| Keys | Does |
| --- | --- |
| ← → | the previous / next place — out of a denominator, past a parenthesis, into the next part |
| ↑ ↓ | the row above / below (a numerator from its denominator, a branch from the next) |
| Home / End | the ends of the enclosing part; again, the ends of the formula |
| Tab / ⇧Tab | the next / previous empty slot |
| ) , | leave the parentheses / the next argument |
| letters, digits | type into the slot or the name or number at the caret; a space after a number starts its unit |
| + − \* / < > = & \| | the operator after the part at the caret, with a slot for the other side; `=` after `<` or `>` makes `<=` / `>=` |
| ! | negate the part; in a slot, with `-`, a sign |
| ( | apply the name before the caret (`clamp` → `clamp(?, ?, ?)`), or group a slot |
| ⌫ / ⌦ | a character; the last character of a value leaves a slot; a slot goes with its operator; a whole structure after the caret |
| click | place the caret and select the part |

## Code view

| Keys | Does |
| --- | --- |
| ⌃Space | completion at the caret |
| ↑ / ↓ · Return / Tab · Esc | move in the completion list · accept · close it |
| rest the pointer on a name | its card; typing or moving away hides it |
| ⌘-click a name · F12 | go to where it is declared (another file opens) |
| ⇧F12 | list every place that names it; Esc or × closes the list |
| ⌥⇧F · _Format_ | lay the file out the canonical way, as one edit |

## Inline rename (canvas, library rows)

| Keys | Does |
| --- | --- |
| Return | commit |
| Esc | keep the old name |
| click elsewhere | commit what was typed |

## Sheets

Return submits, Esc cancels; Cancel is left of the primary button.
