#!/usr/bin/env python3
"""Generates the BDL language mark: a constructivist drafting compass whose
stalk, pencil leg and needle leg form a lambda.  Pure geometry -> SVG, so
the mark is reproducible and every variant shares one definition."""
import math, sys

# ---- palette (constructivist: black, red, cream, one blue) ----------------
BLACK = "#141414"
RED = "#E5322D"
CREAM = "#F2EBDD"
BLUE = "#2B5DD1"

# ---- geometry (viewBox 0 0 256 256) ---------------------------------------
T = (92, 22)     # top of the stalk (knob)
H = (116, 104)   # hinge: where the needle leg branches
B = (192, 238)   # pencil tip
L = (34, 232)    # needle tip

def sub(a, b): return (a[0]-b[0], a[1]-b[1])
def add(a, b): return (a[0]+b[0], a[1]+b[1])
def mul(a, k): return (a[0]*k, a[1]*k)
def norm(a):
    l = math.hypot(*a); return (a[0]/l, a[1]/l)
def perp(a): return (-a[1], a[0])
def fmt(p): return f"{p[0]:.1f},{p[1]:.1f}"

def tapered(a, b, wa, wb):
    """Quadrilateral around segment a->b, width wa at a and wb at b."""
    n = perp(norm(sub(b, a)))
    pts = [add(a, mul(n, wa/2)), add(b, mul(n, wb/2)), add(b, mul(n, -wb/2)), add(a, mul(n, -wa/2))]
    return " ".join(fmt(p) for p in pts)

def lerp(a, b, t): return add(a, mul(sub(b, a), t))

# The long stroke T->B is one straight line through H (lambda's long stroke).
# Assert collinearity by projecting H onto T->B.
d = norm(sub(B, T)); tH = (sub(H, T)[0]*d[0] + sub(H, T)[1]*d[1])
H = add(T, mul(d, tH))

lead_start = lerp(H, B, 0.82)            # pencil lead (red) begins here
needle_start = lerp(H, L, 0.86)          # steel needle (thin) begins here

def svg(variant):
    """variant: 'mark' (transparent), 'icon' (cream tile), 'mono' (single colour)."""
    black, red, blue = (BLACK, RED, BLUE) if variant != "mono" else ("currentColor",)*3
    parts = []
    if variant == "icon":
        parts.append(f'<rect width="256" height="256" rx="56" fill="{CREAM}"/>')
        # suprematist ground: one red quarter-disc behind the hinge
        parts.append(f'<circle cx="{H[0]:.1f}" cy="{H[1]:.1f}" r="78" fill="{RED}" opacity="0.14"/>')
    # arc the pencil would draw: centre = needle tip, radius = |B-L|
    r = math.hypot(*sub(B, L))
    a0 = math.atan2(B[1]-L[1], B[0]-L[0]) - math.radians(5)  # just past the pencil tip
    a1 = a0 - math.radians(28)                                # a short sweep: the circle is just begun
    p0 = add(L, (r*math.cos(a0), r*math.sin(a0))); p1 = add(L, (r*math.cos(a1), r*math.sin(a1)))
    parts.append(f'<path d="M {fmt(p0)} A {r:.1f} {r:.1f} 0 0 0 {fmt(p1)}" fill="none" stroke="{blue}" stroke-width="4.5" stroke-linecap="round"/>')
    # legs
    parts.append(f'<polygon points="{tapered(T, H, 11, 24)}" fill="{black}"/>')            # stalk
    parts.append(f'<polygon points="{tapered(H, lead_start, 24, 9)}" fill="{black}"/>')    # pencil leg
    parts.append(f'<polygon points="{tapered(lead_start, B, 9, 2)}" fill="{red}"/>')       # lead
    parts.append(f'<polygon points="{tapered(H, needle_start, 22, 6)}" fill="{black}"/>')  # needle leg
    parts.append(f'<polygon points="{tapered(needle_start, L, 6, 1)}" fill="{black}"/>')   # needle
    # hinge and knob: the two circles are the mark's "colour blocks"
    parts.append(f'<circle cx="{H[0]:.1f}" cy="{H[1]:.1f}" r="19" fill="{red}"/>')
    parts.append(f'<circle cx="{H[0]:.1f}" cy="{H[1]:.1f}" r="6" fill="{CREAM if variant != "mono" else "none"}"/>')
    parts.append(f'<circle cx="{T[0]:.1f}" cy="{T[1]:.1f}" r="12" fill="{black}"/>')
    body = "\n  ".join(parts)
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">\n  {body}\n</svg>\n'

for v in ("mark", "icon", "mono"):
    with open(f"bdl-{v}.svg", "w") as f:
        f.write(svg(v))
print("ok")
