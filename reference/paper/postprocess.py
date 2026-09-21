#!/usr/bin/env python3
"""Post-process the Pandoc-generated body.typ into the paper's environments.

* multi-letter math words emitted letter-by-letter (`sans(s o m e)`,
  `upright(g r a n t)`, `italic(e v)`) become upright words;
* `#strong[Theorem N (name; `lean`).] body` paragraphs become `#thm(...)`
  blocks (kind, number, name, italic body);
* `#emph[Proof.] … $square.stroked.tiny$` paragraphs become `#proof[...]`;
* `#emph[Figure N. …]` paragraphs become `#figcaption[...]`.
The source of truth is paper/template/; each paper's build.sh copies it beside
its main.typ and runs it on body.typ, which then imports env.typ.  Never edit
the copies or the output.
"""
import re, sys
p = sys.argv[1]
s = open(p).read()

def words(m):
    fn, body = m.group(1), m.group(2)
    w = body.replace(" ", "")
    return f'{fn}("{w}")'
s = re.sub(r'\b(sans|upright|italic|bold)\(([A-Za-z](?: [A-Za-z])+)\)', words, s)

paras = s.split("\n\n")
out = []
for para in paras:
    m = re.match(r'#strong\[(Theorem|Proposition|Lemma|Corollary|Definition)(?: (\d+))? \((.*?)\)\.\]\s*(.*)', para, flags=re.S)
    if m:
        kind, num, name, body = m.groups()
        out.append(f'#thm("{kind}", "{num or ""}")[{name}][{body}]')
        continue
    m = re.match(r'#emph\[Proof\.\]\s*(.*?)\s*\$square\.stroked\.tiny\$\s*$', para, flags=re.S)
    if m:
        out.append(f'#proof[{m.group(1)}]')
        continue
    m = re.match(r'#emph\[(Figure \d+\..*)\]\s*$', para, flags=re.S)
    if m:
        out.append(f'#figcaption[{m.group(1)}]')
        continue
    out.append(para)
open(p, "w").write('#import "env.typ": *\n\n' + "\n\n".join(out))
