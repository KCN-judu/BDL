# Accessibility

Sources: Apple HIG accessibility (macOS), WCAG 2.x contrast and keyboard criteria,
Flutter's `Semantics` model. A desktop tool used for hours must be usable by keyboard
alone, without colour, and without motion.

## Contrast

- Text: ≥ 4.5:1 against its background; large/bold ≥ 3:1. Studio's primary (85 %) and
  secondary (50 %) text pass on the chrome colours; tertiary (25 %) does *not* pass for
  anything the user must read — use it only for de-emphasis of things also available
  elsewhere.
- Non-text (icons, socket outlines, 2 px links, focus rings): ≥ 3:1. Identity hues are
  generated; constrain lightness per hue so the worst case (yellows, cyans on white)
  still clears 3:1, in both themes. Verify, don't assume.
- Hairlines at 12 % are decorative; nothing may depend on seeing them.
- Honour "Increase contrast": tokens should have a high-contrast variant or at least
  raise hairline and tertiary alphas.

## Keyboard

- Every pointer action has a keyboard route: select (Tab / arrows), act (Return, Space,
  ⌘-shortcut), link (a menu command "Connect to…" with a pop-up, not only a drag),
  move (arrow nudges), delete (⌫), frame (⌘0), find (⌘F), pages (⌘1–4), panels (⌥⌘S / I).
- Focus order follows visual order: toolbar → sidebar → content → inspector; within a
  form, top to bottom; within a sheet, first field autofocused, Return submits, Esc
  cancels.
- Custom look-alikes (`MacSegmented`, concept toggles, chips with ×, canvas nodes) must
  be focusable with `FocusableActionDetector` and respond to Space/Return/arrows. A
  control you cannot tab to is not a control.
- Focus is visible: the 2 pt accent ring, on keyboard focus only (not on click).

## Non-colour encoding

Every colour meaning in the channel table has a partner: name for hue, shape for
representation, dash for incompleteness, word for status, icon + text for error, weight
for selection outline, text for age. Test by switching the display to greyscale.

## Readable text

- No text below 10 pt; nothing the user must act on below 11 pt.
- System font, system rendering; no letter-spacing tricks; no ALL CAPS.
- Ellipsis with a tooltip for the full text; no clipping without a way to see the rest.

## Targets

- Interactive targets ≥ 22 × 22 pt (controls) and ≥ 20 pt hit radius for canvas sockets
  (10 pt visual, 10 pt hit radius today — acceptable with the halo feedback; do not go
  smaller).
- Remove buttons on chips (18 pt) are below target size; acceptable only because the
  same action exists in the inspector's pop-up and via ⌫ on a selected link.

## Motion

- 120–150 ms ease-out for state transitions; nothing longer; nothing looping.
- Honour reduce-motion: transitions become instant; nothing pulses; live values update
  without animation.
- Motion never carries a fact. If something is only noticeable because it moved, it
  also needs a resting encoding.

## Screen-reader semantics

- Wrap custom widgets in `Semantics` with a label that states the fact in product
  language: "Tilt, quantity, angle, open", "dimByTilt, declared, reads Tilt, produces
  Brightness", "link from Tilt to dimByTilt".
- Canvas nodes are focusable semantic nodes in reading order (left → right, top → bottom)
  even though they are painted in one `CustomPaint`; use `SemanticsNode`s via
  `CustomPainter.semanticsBuilder`.
- State changes announce (`liveRegion`) only for what the user did not initiate: an
  incoming project change, a connection change.
- Pop-ups, sheets and banners have roles; buttons have labels that are verbs with
  objects ("Delete Tilt", not "Delete").

## Checks

- Unplug the mouse. Complete: create a concept, create a mapping, link them, attach a
  formula, delete the mapping. Note every step that failed.
- Greyscale the screen. Name every fact you can no longer see.
- Enable reduce-motion. Did any information disappear?
- Run the contrast numbers for the three worst identity hues on both themes.
