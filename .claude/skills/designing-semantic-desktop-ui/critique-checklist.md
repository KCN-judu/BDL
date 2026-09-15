# Critique checklist

Answer every question in writing for the screen or component under review. A "yes" needs
a reason; a "no" needs a proposed change or an explicit acceptance with a reason. Do this
before declaring any UI work done and whenever asked to review a screen.

## Content

1. What is the single most important fact on this screen for the task at hand?
2. Can it be found in under one second, without reading? By which channel?
3. Is anything shown only because the internal model contains it (ids, revisions,
   protocol versions, enum names, counts nobody acts on)?
4. Is anything *missing* that the task needs (a fact the designer must go elsewhere for)?
5. Is every piece of prose specific to *this* object *now*? Which lines would be identical
   for every object? (Those are documentation.)

## Hierarchy

6. Is importance visible when you squint — by weight, colour, position, size, in that
   order of preference?
7. How many levels of hierarchy are there? Could it be one fewer?
8. Is secondary information actually secondary (lighter, later, smaller-not-tinier), or
   is it competing?
9. Does anything shout (bold + colour + size at once) that does not deserve it?

## Grouping

10. Are related things closer together than unrelated things? State the three gap sizes
    and confirm they are visibly distinct.
11. Is alignment doing grouping work — do labels share one edge, values another,
    numbers a third?
12. Is any border, box or card used where whitespace or a background tint would group
    more quietly?
13. Does any adjacent pair of marks create an unintended third shape (1 + 1 = 3)?

## Typography

14. Count the type sizes. Is the scale small and the same as the rest of Studio?
15. Is hierarchy visible through weight and colour, not size alone?
16. Are labels and values baseline-aligned? Numbers in tabular figures, right-aligned?
17. Is any text the user must act on below 11 pt, or in tertiary colour?
18. Is any punctuation (· — / | parentheses) doing layout work?

## Semantics

19. Does the interface show *meaning* (state of the object) or *metadata* (a description
    beside it)? Name each fact and which it is.
20. Can the object's state be understood from the object itself without the inspector?
21. Is compiler vocabulary leaking into ordinary UI (declRef, Grant, Κ, sync,
    SingleDriver, revision, protocol, bdld, id, enum names)?
22. Is any semantic distinction represented *only* by a badge, pill or word?
23. Is incomplete-but-valid visually distinct from wrong? Is anything red that is not
    actually wrong?
24. Are the words for this fact the same words used in the sheet, library, inspector,
    canvas and diagnostic?

## Interaction

25. Is every action located at or next to the object it affects?
26. Is feedback immediate and local for hover, drag, drop, commit, refusal?
27. Are destructive actions named ("Delete Tilt"), reversible, and never "OK"?
28. Is direct manipulation used where the thing is naturally spatial, and a form where
    it is naturally a value?
29. Are impossible actions constrained before failure (a link that cannot form, a
    disabled control) rather than diagnosed after?
30. Does every disabled control show its reason at rest, not only in a tooltip?
31. Is there any excise — a confirmation, a pre-step, a "select first" — that undo or a
    better default would remove?
32. What is the silent default? Could it be wrong without the user noticing?

## Consistency

33. Does the same semantic concept look identical across pages, panels, sheets and rows
    (one drawing of a socket, one set of words)?
34. Does each colour, shape and style keep one meaning? Check against the channel table
    in `semantic-ui.md`.
35. Does selection or hover ever erase or override a state encoding (dashed, hollow,
    mark)?

## Accessibility

36. Does meaning survive without colour (greyscale test)? Name the partner cue for each
    colour.
37. Can the whole task be done from the keyboard? Tab order matches visual order? Is
    every custom look-alike focusable?
38. Is keyboard focus visible (2 pt accent ring) and never shown on click?
39. Is contrast sufficient — text ≥ 4.5:1, non-text and identity hues ≥ 3:1, both themes?
40. Does any motion carry a fact that is lost with reduce-motion?
41. Do custom-painted objects expose semantics (labels in product language)?

## Platform

42. Is any custom control replacing a standard macOS control or arrangement (sheet,
    pop-up, segmented control, sidebar, inspector, contextual menu, menu bar) without a
    BDL reason written down?
43. Are Cancel/default placement, Return/Esc, ⌘-shortcuts and named destructive buttons
    as the HIG expects?
44. Would an experienced Mac user of Blender / Resolve / Figma / Xcode recognise the
    arrangement immediately?

## States

45. What does this look like when empty? When the object is incomplete? When the
    compiler refuses? When the daemon is disconnected or a request is pending? When
    the result is stale (older revision)?
46. Does the empty state name the first action and its shortcut?

## Report

For each violation: the question number, the file:line, the proposed change, and the
principle it follows (`principles.md` number or the reference file). Rank by the harm to
the task, not by how easy the fix is.
