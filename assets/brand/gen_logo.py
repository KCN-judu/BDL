#!/usr/bin/env python3
"""BDL language mark: a drafting compass reduced to straight lines.

Head upright.  Stalk and needle leg are one vertical bar; the pencil leg is
the single diagonal.  Bars have parallel edges and come to a point with a
short chamfer.  The lambda is what remains — a long stroke with a branch —
without tilting anything to get it.  Pure geometry -> SVG."""
import math

BLACK = "#141414"
RED = "#E5322D"
CREAM = "#F2EBDD"
BLUE = "#2B5DD1"

# ---- geometry (viewBox 0 0 256 256) ---------------------------------------
X = 150                     # the vertical bar's centre line
W = 20                      # bar width (parallel edges)
KNOB = (X, 18, 30, 30)      # knob: cx, top, width, height
HINGE = (X, 104, 36)        # hinge: cx, cy, side (square)
NEEDLE_TIP = (X, 242)
PENCIL_TIP = (52, 232)
LEAD = 0.80                 # fraction of the pencil leg after which it is red
CHAMFER = 1.5               # tip chamfer length, in bar widths

def sub(a, b): return (a[0]-b[0], a[1]-b[1])
def add(a, b): return (a[0]+b[0], a[1]+b[1])
def mul(a, k): return (a[0]*k, a[1]*k)
def norm(a):
    l = math.hypot(*a); return (a[0]/l, a[1]/l)
def perp(a): return (-a[1], a[0])
def lerp(a, b, t): return add(a, mul(sub(b, a), t))
def pts(ps): return " ".join(f"{p[0]:.1f},{p[1]:.1f}" for p in ps)

def bar(a, b, w, tip_at_b=False):
    """Parallel-edged bar a->b.  With tip_at_b the last CHAMFER*w of the bar
    closes to a point with two short straight cuts."""
    d = norm(sub(b, a)); n = perp(d)
    if not tip_at_b:
        return [add(a, mul(n, w/2)), add(b, mul(n, w/2)), add(b, mul(n, -w/2)), add(a, mul(n, -w/2))]
    c = add(b, mul(d, -CHAMFER*w))
    return [add(a, mul(n, w/2)), add(c, mul(n, w/2)), b, add(c, mul(n, -w/2)), add(a, mul(n, -w/2))]

def svg(variant):
    black, red = (BLACK, RED) if variant != "mono" else ("currentColor", "currentColor")
    hx, hy, hs = HINGE
    h = (hx, hy)
    parts = []
    if variant == "icon":
        parts.append(f'<rect width="256" height="256" rx="56" fill="{CREAM}"/>')
    # vertical bar: knob top -> needle tip (stalk and needle leg are one line)
    parts.append(f'<polygon points="{pts(bar((X, KNOB[1]), NEEDLE_TIP, W, tip_at_b=True))}" fill="{black}"/>')
    # knob block
    kx, ky, kw, kh = KNOB
    parts.append(f'<rect x="{kx - kw/2}" y="{ky}" width="{kw}" height="{kh}" fill="{black}"/>')
    # pencil leg: the one diagonal; black bar, then red lead with the chamfered tip
    lead_start = lerp(h, PENCIL_TIP, LEAD)
    parts.append(f'<polygon points="{pts(bar(h, lead_start, W - 2))}" fill="{black}"/>')
    parts.append(f'<polygon points="{pts(bar(lead_start, PENCIL_TIP, W - 2, tip_at_b=True))}" fill="{red}"/>')
    # hinge: a red square block, cream pin
    parts.append(f'<rect x="{hx - hs/2}" y="{hy - hs/2}" width="{hs}" height="{hs}" fill="{red}"/>')
    if variant != "mono":
        parts.append(f'<rect x="{hx - 5}" y="{hy - 5}" width="10" height="10" fill="{CREAM}"/>')
    body = "\n  ".join(parts)
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">\n  {body}\n</svg>\n'

for v in ("mark", "icon", "mono"):
    with open(f"bdl-{v}.svg", "w") as f:
        f.write(svg(v))
print("ok")
