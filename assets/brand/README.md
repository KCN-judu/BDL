# BDL brand assets

## The mark

A drafting compass reduced to straight lines, in the constructivist manner: flat
colour blocks, no outlines, no curves. Two legs of **equal length**, opened
symmetrically so the **tips sit on one horizontal line**; the stalk continues
the right leg upward, so the whole instrument leans. Bars have parallel edges
and come to a point with a short chamfer; the knob and hinge are square blocks
aligned with the stalk. Two line directions only. The **λ** — a long stroke with
a branch — is what a compass in use looks like. The compass is the designer's
instrument; the lambda is the language's.

```text
bdl-mark-light.svg  black body, transparent      — light themes, documents
bdl-mark-dark.svg   white body, transparent      — dark themes
bdl-icon.svg        black body on a cream tile   — app icon (macOS/Windows)
bdl-mono.svg        currentColor                 — toolbars, small sizes
png/           rasters exported from bdl-icon.svg
gen_logo.py    the single source: geometry → all three SVGs
```

`apps/studio/lib/ui/brand/compass_mark.dart` paints the same geometry in Flutter
(same coordinates, no SVG runtime). Change `gen_logo.py` and the painter
together.

## Palette

|       | hex       | role                                                   |
| ----- | --------- | ------------------------------------------------------ |
| black | `#141414` | legs, knob (light version)                             |
| white | `#F4F4F2` | legs, knob (dark version); hinge pin becomes `#1E1E1E` |
| red   | `#E5322D` | hinge, pencil lead                                     |
| cream | `#F2EBDD` | tile, hinge pin                                        |
| blue  | `#2B5DD1` | reserved (accent in UI)                                |

## Regenerate

```bash
python3 assets/brand/gen_logo.py           # SVGs
# rasters: render bdl-icon.svg at 1024 px (headless Chrome or any SVG
# renderer), then resize into apps/studio/macos/Runner/Assets.xcassets/
# AppIcon.appiconset and apps/studio/windows/runner/resources/app_icon.ico
```

## Type

Wordmark _Behavior Designer_ in Chakra Petch (SIL OFL), see
`apps/studio/assets/fonts/`.
