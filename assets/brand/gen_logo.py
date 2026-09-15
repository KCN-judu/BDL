#!/usr/bin/env python3
"""BDL language mark: a drafting compass reduced to straight lines.

Two legs of equal length, opened symmetrically so the tips sit on one
horizontal line; the stalk continues the right leg upward, so the whole
instrument leans — a lambda.  Bars have parallel edges and come to a point
with a short chamfer; knob and hinge are square blocks aligned with the
stalk.  Two line directions only.  Pure geometry -> SVG."""
import math

BLACK = "#141414"
RED = "#E5322D"
CREAM = "#F2EBDD"
BLUE = "#2B5DD1"

# ---- geometry (viewBox 0 0 256 256) ---------------------------------------
H = (128.0, 102.0)          # hinge
THETA = math.radians(24)    # half opening angle, from vertical
LEG = 138.0                 # hinge -> tip, both legs
STALK = 60.0                # hinge -> knob centre, along the right leg's line
W = 20.0                    # bar width (parallel edges)
KNOB = 30.0                 # knob square side
HINGE = 36.0                # hinge square side
LEAD_LEN = 42.0             # length of the red lead, measured from the tip
CHAMFER = 1.5               # tip chamfer length, in bar widths
assert LEAD_LEN > CHAMFER * W + 4, "the lead must contain the whole chamfer or the polygon folds"

def add(a, b): return (a[0]+b[0], a[1]+b[1])
def sub(a, b): return (a[0]-b[0], a[1]-b[1])
def mul(a, k): return (a[0]*k, a[1]*k)
def perp(a): return (-a[1], a[0])
def lerp(a, b, t): return add(a, mul(sub(b, a), t))
def pts(ps): return " ".join(f"{p[0]:.1f},{p[1]:.1f}" for p in ps)

DR = (math.sin(THETA), math.cos(THETA))      # right leg direction (down-right)
DL = (-math.sin(THETA), math.cos(THETA))     # left leg direction (down-left)
PENCIL_TIP = add(H, mul(DR, LEG))
NEEDLE_TIP = add(H, mul(DL, LEG))
KNOB_C = sub(H, mul(DR, STALK))              # stalk continues the right leg upward
assert abs(PENCIL_TIP[1] - NEEDLE_TIP[1]) < 1e-9  # tips level

def bar(a, b, w, tip_at_b=False):
    d = sub(b, a); l = math.hypot(*d); d = (d[0]/l, d[1]/l); n = perp(d)
    if not tip_at_b:
        return [add(a, mul(n, w/2)), add(b, mul(n, w/2)), add(b, mul(n, -w/2)), add(a, mul(n, -w/2))]
    c = add(b, mul(d, -CHAMFER*w))
    return [add(a, mul(n, w/2)), add(c, mul(n, w/2)), b, add(c, mul(n, -w/2)), add(a, mul(n, -w/2))]

def square(c, side, d):
    """Square centred at c with one axis along d."""
    n = perp(d); h = side/2
    return [add(add(c, mul(d, h)), mul(n, h)), add(add(c, mul(d, h)), mul(n, -h)),
            add(add(c, mul(d, -h)), mul(n, -h)), add(add(c, mul(d, -h)), mul(n, h))]

def svg(variant):
    black, red = (BLACK, RED) if variant != "mono" else ("currentColor", "currentColor")
    parts = []
    if variant == "icon":
        parts.append(f'<rect width="256" height="256" rx="56" fill="{CREAM}"/>')
    # long stroke: knob -> pencil tip as one black chamfered bar, then the red
    # lead painted over its end (no abutting edges, hence no seams)
    lead_start = lerp(PENCIL_TIP, H, LEAD_LEN / LEG)
    parts.append(f'<polygon points="{pts(bar(KNOB_C, PENCIL_TIP, W, tip_at_b=True))}" fill="{black}"/>')
    parts.append(f'<polygon points="{pts(bar(lead_start, PENCIL_TIP, W, tip_at_b=True))}" fill="{red}"/>')
    # needle leg
    parts.append(f'<polygon points="{pts(bar(H, NEEDLE_TIP, W, tip_at_b=True))}" fill="{black}"/>')
    # knob and hinge blocks, aligned with the long stroke
    parts.append(f'<polygon points="{pts(square(KNOB_C, KNOB, DR))}" fill="{black}"/>')
    parts.append(f'<polygon points="{pts(square(H, HINGE, DR))}" fill="{red}"/>')
    if variant != "mono":
        parts.append(f'<polygon points="{pts(square(H, 10, DR))}" fill="{CREAM}"/>')
    body = "\n  ".join(parts)
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">\n  {body}\n</svg>\n'

for v in ("mark", "icon", "mono"):
    with open(f"bdl-{v}.svg", "w") as f:
        f.write(svg(v))
print("ok", PENCIL_TIP, NEEDLE_TIP, KNOB_C)
