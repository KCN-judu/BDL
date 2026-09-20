// BDL Design and Formalization Monograph - Typst layout.
// Primary editable source: paper.md -> body.typ (generated with Pandoc via build.sh).
// This file supplies the document chrome and a single-column monograph layout.

#set page(
  paper: "us-letter",
  margin: (top: 0.9in, bottom: 0.9in, left: 1.0in, right: 1.0in),
  numbering: "1",
  number-align: center,
)
#set text(font: "Libertinus Serif", size: 10pt, lang: "en")
#set par(justify: true, leading: 0.6em)
#set heading(numbering: none)
#set enum(indent: 1.15em, body-indent: 0.5em)
#set list(indent: 1.05em, body-indent: 0.5em)
#show raw: set text(font: "DejaVu Sans Mono", size: 0.88em)
#show table: set text(size: 8.6pt)
#show table.cell: set align(left + top)
#show table.cell: set par(justify: false)
#show table.cell.where(y: 0): set text(weight: "bold")
#set table(inset: (x: 4pt, y: 3pt), stroke: (x: none, y: 0.3pt))
// Pandoc wraps every table in a figure; let long tables break across pages,
// and let identifiers inside table cells break at their separators.
#show figure.where(kind: table): set block(breakable: true)
#show figure.where(kind: table): set figure(placement: none)
// A row never splits across pages: each cell body is an unbreakable box
// (the table itself still breaks between rows; the header row repeats).
#show table.cell: it => box(width: 100%, it)
#show raw.where(block: false): it => {
  show regex("[_./:]"): m => m.text + sym.zws
  it
}

#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(6pt)
  set text(font: "Libertinus Serif", size: 15pt, weight: "bold")
  it
  v(6pt)
}
#show heading.where(level: 2): set text(font: "Libertinus Serif", size: 11.5pt, weight: "bold")
#show heading.where(level: 3): set text(font: "Libertinus Serif", size: 10.2pt, weight: "bold", style: "italic")

// ---------------------------------------------------------------- title page
#align(center)[
  #v(2.2in)
  #text(font: "Libertinus Serif", size: 24pt, weight: "bold")[
    BDL Design and Formalization Monograph
  ]
  #v(10pt)
  #text(size: 13pt)[
    A Behavior Design Language for Interactive Physical Products: \
    Why Behavior Needs a Design Medium, the Architecture, \
    One Product Through the Lifecycle, the Derived Formal Model, \
    the Toolchain, Studio, and the Evidence
  ]
  #v(24pt)
  #text(size: 11pt)[ZHU ZHEHAO]
  #v(6pt)
  #text(size: 9.5pt, style: "italic")[Living technical record — revision of 2026-09-20]
  #v(4pt)
  #text(size: 9pt)[
    Formal development `KCN-judu/BDL_FV` — this working tree, through Phase 15 (previous commit `dce5ac4`) \
    Production `KCN-judu/BDL` at `081296df606d577eece7e269ed250b255547d497` (2026-09-20, protocol 0.24)
  ]
  #v(1.6in)
  #block(width: 88%)[
    #set par(justify: true)
    #set text(size: 9.3pt)
    This document is the authoritative narrative design record of BDL — a behavior-design language for interactive physical products — written as one argument, from the design problem through the architecture, one product's lifecycle, the derived formal model, the toolchain and the authoring environment to the evidence, and a bridge between its formal model and its production system. Its canonical source is `paper/` in `KCN-judu/BDL_FV`; the production repository carries a reference mirror that does not evolve on its own. It is not a paper, not a submission and not written to a page limit; it records what the language is, why each construct exists or was rejected, which theorem, counterexample or executed example supports each claim, how production implements it, where production deviates from the formal model, and what remains open — including the alternatives that failed. Every claim carries a strength label (formally proved; formally characterized under restricted hypotheses; mechanically executed example; counterexample; informed by FV; production implemented and tested; production architecture decision; proposal / not implemented; design recommendation; open empirical question). No usability claim in it has been tested with users.
  ]
]

#pagebreak()

// ---------------------------------------------------------------- contents
#outline(title: [Contents], indent: 1.2em, depth: 3)

#pagebreak()

#include "body.typ"
#bibliography("references.bib", style: "ieee", title: "References")
