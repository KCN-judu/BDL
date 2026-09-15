# Perception: Gestalt and visual channels

Sources: the Gestalt school (Wertheimer, Köhler, Koffka), Palmer's later additions, the
Nielsen Norman Group's practical articles on the principles, and Colin Ware,
*Information Visualization: Perception for Design*. Own words.

## Gestalt — how the eye groups before the mind reads

| Principle | What the eye does | Use it for | Studio example |
|---|---|---|---|
| **Proximity** | things close together are one group | the strongest and cheapest grouping tool; beats similarity | label 8 pt from its field, 16 pt from the next row |
| **Similarity** | things that look alike belong together | shared shape/colour for shared category | all sockets of `Tilt` are the same hue; all mapping headers the same tint |
| **Common region** | things inside one enclosure are one group | a tinted background, stronger than a stroke, quieter too | a clock-domain lane; a context region; the preview box in a sheet |
| **Uniform connectedness** | things joined by a line are one unit — stronger than proximity or similarity | dependency; membership by link | a link makes concept and mapping one relationship |
| **Continuity** | the eye follows smooth paths | flows should be smooth curves, not elbows; aligned edges read as lines | Bézier links with horizontal tangents; the label column edge |
| **Closure** | the eye completes shapes | partial outlines still read as objects — dashed still reads as a node | dashed = incomplete, not broken |
| **Figure / ground** | one thing is the object, the rest is background | the canvas is ground; nodes are figure; the sheet's barrier pushes the window to ground | keep the canvas and grid quiet so nodes pop |
| **Common fate** | things that move together are one thing | drag moves the selection as one | multi-select drag |
| **Symmetry / order** | balanced, regular arrangements read as intentional | the module grid | fixed row heights, fixed columns |

Ordering that matters in practice (NN/g): **common region and connectedness beat
proximity, which beats similarity**. So a coloured link claims two objects belong
together more strongly than putting them side by side; an enclosure claims membership
more strongly than a shared colour. Use the strong ones for the strong facts (dependency,
domain) and the weak ones for the weak facts (category).

## Ware — what pops out and what does not

**Preattentive attributes** are read in parallel across the whole field in under ~200 ms:
hue, luminance, orientation, size, elongation, curvature, enclosure, added marks (a
mark on a line), spatial position, motion/flicker, and stereo depth. Shape in general is
*not* preattentive; a few strongly different shapes (○ ◇ □) are, when few.

Rules that follow:

- **The fact you want found first must be on a preattentive channel.** A dashed outline,
  a hollow socket, a coloured link, a mark on a link, an enclosure — yes. A word in the
  header, a number in a status line — no; those are for the second look.
- **Conjunctions do not pop out.** "Red *and* dashed" cannot be scanned for; only one
  attribute at a time. So one fact ↔ one attribute; do not require the eye to combine.
- **Hue for categories, not quantities.** Hue distinguishes ~6–8 categories reliably
  without a legend, ~12 with labels. More concepts than that need the name as the
  reliable cue and the hue as the fast-but-fallible one — which is why every socket has
  a label and every identity hue has a name partner.
- **Luminance contrast carries edges and text.** Hue contrast alone does not. A 2 px link
  in a light hue on a white canvas vanishes; constrain lightness so every identity hue
  has enough luminance contrast with the canvas in both themes.
- **Position and length are the most accurate channels for quantities**; then slope,
  angle, area; colour saturation last. Values in simulation and monitoring are shown as
  numbers and positions (plots), never as colour intensity.
- **Visual working memory holds ~3–4 objects.** A screen that asks the designer to
  compare five things in different places has already lost; bring them into one region
  or one table.
- **Motion captures attention absolutely.** Use it only for a transient answer (a link
  retracting on refusal, a selection change) and never to carry a fact — the reduced-
  motion user would lose it, and everyone else would be distracted.
- **Design the visual query.** Ask "what will the designer *look for* on this screen?"
  and make that the popout. Everything else is allowed to require reading.

## Colour, specifically

- Neutral chrome with low-saturation category tints keeps the identity hues the only
  saturated thing on the canvas — which is what makes them findable.
- Green/orange/red status colours are semantic words with a colour partner, not
  fills. Red is reserved for a fact that is wrong; orange for a decision still open;
  green for settled. None of these is an identity hue's job.
- Two themes: every colour must be defined for light and dark; identity hues shift
  lightness with the theme (0.48 light / 0.62 dark) to keep contrast, never hue.
- Check every hue pair that will sit adjacent (sockets on one node) for
  distinguishability, and every hue against the canvas for contrast.

## Checks

- Cover the text. Can you still tell which things belong together, which are
  incomplete, which are selected?
- What pops out first? Is that the most important fact?
- Is any group made with a border where a tint or 8 pt of space would do?
- Is any fact encoded as a conjunction of two attributes?
