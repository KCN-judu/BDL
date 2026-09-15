# BDL brand assets

## The mark

A drafting compass in the constructivist manner — flat colour blocks, no
outlines, one diagonal — whose stalk, pencil leg and needle leg form a **λ**:
the hinge sits partway down the long stroke, exactly where lambda's short
stroke branches. The compass is the designer's instrument; the lambda is
the language's. The blue stroke is the circle the pencil has just begun.

```
bdl-mark.svg   transparent, full colour          — documents, hero, README
bdl-icon.svg   cream tile + pale red disc        — app icon (macOS/Windows)
bdl-mono.svg   currentColor                      — toolbars, small sizes
png/           rasters exported from bdl-icon.svg
gen_logo.py    the single source: geometry → all three SVGs
```

`apps/studio/lib/ui/brand/compass_mark.dart` paints the same geometry in
Flutter (same coordinates, no SVG runtime). Change `gen_logo.py` and the
painter together.

## Palette

| | hex | role |
|---|---|---|
| black | `#141414` | legs, knob |
| red | `#E5322D` | hinge, pencil lead |
| cream | `#F2EBDD` | tile, hinge pin |
| blue | `#2B5DD1` | the drawn arc |

## Regenerate

```bash
python3 assets/brand/gen_logo.py           # SVGs
# rasters: render bdl-icon.svg at 1024 px (headless Chrome or any SVG
# renderer), then resize into apps/studio/macos/Runner/Assets.xcassets/
# AppIcon.appiconset and apps/studio/windows/runner/resources/app_icon.ico
```

## Type

Wordmark *Behavior Designer* in Chakra Petch (SIL OFL), see
`apps/studio/assets/fonts/`.
