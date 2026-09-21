// Theorem, proof and caption environments shared by main.typ and body.typ.
// ---------------------------------------------------------------- environments
// Theorem-like statement: small-caps head, italic body, hanging from the head.
#let thm(kind, num, name, body) = block(
  above: 1.1em, below: 1.1em, width: 100%, breakable: true,
)[
  #set par(first-line-indent: 0em)
  #text(weight: "bold")[#smallcaps(kind)#if num != "" [ #num]]
  #if name != [] [ (#name).] else [.]
  #emph[#body]
]
// Proof: "Proof." in italics, the tombstone on the last line.
#let proof(body) = block(above: 0.6em, below: 1.1em, width: 100%, breakable: true)[
  #set par(first-line-indent: 0em)
  #emph[Proof.] #body #h(1fr) $square$
]
// Figure caption: small, set off from the figure.
#let figcaption(body) = block(above: 0.4em, below: 1.2em, width: 100%)[
  #set par(first-line-indent: 0em)
  #set text(size: 9pt)
  #body
]

