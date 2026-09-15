# Principles — the compressed set

Everything in the other files reduces to these. If a decision cannot be traced to one of
them, it is probably a habit imported from a web framework.

## From the reader's side (Krug, Norman, Cooper)

1. **Recognition over interpretation.** The designer should not have to read to know
   what something is or what state it is in. Text is the fallback, not the default.
2. **The object is the interface.** State, ownership and consequence belong *on the
   thing* — on the node, socket, link, pin — not in a panel that describes the thing.
3. **Constrain before you diagnose.** If the kernel forbids it, make it undrawable or
   refuse it at the pointer with local feedback. A diagnostic afterwards is the last
   resort, and it must come with the fix.
4. **Feedback is immediate, local and proportionate.** Hover, drag, drop and commit each
   answer at the point of action within one frame; nothing modal for anything routine.
5. **Design for the perpetual intermediate.** A first-time path (sheets that teach by
   showing), a fast path (keyboard, direct manipulation), and no path that lectures.
6. **Undo is the safety net; naming is the confirmation.** Destructive actions are
   named ("Delete Tilt"), reversible, and never hide behind "OK".
7. **The conceptual model must match the kernel.** Refinement vs edit, declared vs
   defined, domain vs rate — the UI's words and shapes must not imply a model the
   compiler does not have (no execution order on the canvas, no arithmetic nodes).

## From perception (Gestalt, Ware, Tufte)

8. **One channel, one meaning.** Hue is identity. Shape is representation. Dashed is
   incomplete. Red is wrong. Accent is selection. Containment is domain or context. Do
   not reuse.
9. **Preattentive first.** The most important fact should pop out of a scan (colour,
   shape, enclosure, connection) before any reading begins.
10. **Redundant coding for anything that matters.** Colour never stands alone; give it
    shape, text or position as a partner.
11. **Proximity and alignment group; borders are the last resort.** Whitespace and a
    shared edge make a group. A box around it is what you draw when the spacing failed.
12. **Data ink.** Every pixel either carries a fact or organises facts. Shadows,
    gradients, separators and badges that do neither are removed.
13. **Layer and separate.** Secondary information is lighter, not smaller; tertiary
    information appears on hover or selection; it never shouts alongside the primary.
14. **Comparison beats a lone number.** A count, a state or a value gains meaning next to
    what it is compared with (needed vs available, now vs previous tick).

## From composition (Müller-Brockmann, Lupton, Refactoring UI)

15. **A grid you can feel, not see.** 8 pt base, fixed column widths, one gutter, label
    column right-aligned, values left-aligned, numbers right-aligned in tabular figures.
16. **A small type scale, hierarchy by weight and colour.** 13 / 11 / 10 for body /
    secondary / caption; bigger only for the title of a window or a hero. Hierarchy comes
    from weight, colour and position before size.
17. **Start with too much whitespace and remove.** Gaps inside an item < between items
    < between groups < between sections, and each ratio is visible.
18. **Colour restraint.** Neutral chrome; the accent for selection and one default
    action; semantic hues only where the contract assigns them.
19. **Repeat structures.** Anything with two or more rows sharing fields is a table with
    fixed columns; anything with two or more instances of a concept looks the same
    everywhere it appears.

## From the platform (Apple HIG, professional tools)

20. **Use the convention unless BDL has a reason.** Sidebar–content–inspector, sheets,
    pop-ups, disclosure, ⌘ shortcuts, named buttons, arrow-key lists. Custom controls
    exist only for what the platform has no control for (nodes, sockets, lanes, gates).
21. **The canvas keeps the structure; the inspector keeps the properties; selection
    connects them.** Nothing editable lives only on the canvas; nothing structural lives
    only in the inspector.
22. **Spatial memory is sacred.** Objects do not move unless the user moves them; panels
    do not rearrange themselves; the same object looks the same on every page.
23. **Keyboard equals mouse.** Every action has a keyboard route; focus is visible; the
    Tab order follows the visual order.
