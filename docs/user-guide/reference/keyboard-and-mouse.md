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
| drag a concept's output socket onto an output | connect the relationship that drives it |
| drag socket → socket | link |
| drag a connected input socket away → empty canvas | disconnect |
| ⌫ / Delete | delete the selection (or disconnect a selected binding link) |
| double-click a concept or relationship | rename in place |
| double-click an instance | open its component's source |
| double-click a behavior's title | rename |
| right-click · Control-click | context menu |
| drag a Library row onto the canvas | insert a concept |

## Formula field

| Keys | Does |
| --- | --- |
| ⌘↩ | Add / Save the definition |
| Esc | Revert the draft; with completion open, close it first |
| Return | new line |
| ⌃Space | completion |
| ↑ / ↓ · Return / Tab | move in the completion list · accept |

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
