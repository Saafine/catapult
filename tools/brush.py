"""Painting toolkit for the hand-painted sprites (characters, hero pieces),
after the designers' key art: every part is filled with a colour lit from
the top left, then soft blurred shadow and light shapes are painted inside
it (clipped to the part), then a dark warm outline goes on top.

    art = Art()
    art.part(d, "#e8852f", shades=[(d2, "#b5541f", 0.7)], lights=[(d3, "#ffd09a", 0.8)])
    art.svg(256, 256)  -> the svg text (data-paint="done")
"""
import colorsys

INK = "#40200f"

def _rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))

def _hex(r, g, b):
    return "#%02x%02x%02x" % tuple(max(0, min(255, round(v * 255))) for v in (r, g, b))

def tone(c, dv=0.0, ds=0.0, dh=0.0):
    """Shift a colour in HSV: value, saturation, hue."""
    h, s, v = colorsys.rgb_to_hsv(*_rgb(c))
    return _hex(*colorsys.hsv_to_rgb((h + dh) % 1, max(0, min(1, s + ds)), max(0, min(1, v + dv))))

def warm_light(c):
    if sum(_rgb(c)) > 2.7:
        return "#ffffff"
    return tone(c, 0.14, -0.14, 0.015)

def cool_shade(c):
    h, s, v = colorsys.rgb_to_hsv(*_rgb(c))
    if s < 0.12:  # whites and greys shade to blue-grey
        return _hex(*colorsys.hsv_to_rgb(0.6, 0.18, v * 0.78))
    # warm colours shade towards red, cool ones towards blue
    target = 0.0 if (h < 0.2 or h > 0.85) else 0.62
    d = ((target - h + 0.5) % 1) - 0.5
    return _hex(*colorsys.hsv_to_rgb((h + d * 0.12) % 1, min(1, s + 0.1), v * 0.7))

class Art:
    def __init__(self):
        self.defs = []
        self.body = []
        self.n = 0
        self.blurs = set()

    def uid(self, p="a"):
        self.n += 1
        return f"{p}{self.n}"

    def blur(self, sd):
        fid = f"bl{str(sd).replace('.', '_')}"
        if fid not in self.blurs:
            self.blurs.add(fid)
            self.defs.append(f'<filter id="{fid}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="{sd}"/></filter>')
        return fid

    def lit(self, c, cx=0.32, cy=0.25, r=0.85):
        gid = self.uid("g")
        self.defs.append(f'<radialGradient id="{gid}" cx="{cx}" cy="{cy}" r="{r}"><stop offset="0" stop-color="{warm_light(c)}"/>'
                         f'<stop offset="0.5" stop-color="{c}"/><stop offset="1" stop-color="{tone(c, -0.1, 0.05)}"/></radialGradient>')
        return f"url(#{gid})"

    def add(self, s):
        self.body.append(s)

    def part(self, d, fill, shades=(), lights=(), w=4.5, ink=INK, soft=7, flat=False, extra="", rim=True, tone_of=None):
        """One painted part. `shades`/`lights` are (path, colour, opacity)
        tuples, or (path, colour, opacity, blur) to change the softness;
        colour None picks the part's own shadow / light tone (derived from
        `tone_of` when the fill is a gradient url)."""
        col = tone_of or fill
        cid = self.uid("c")
        self.defs.append(f'<clipPath id="{cid}"><path d="{d}"/></clipPath>')
        base = fill if (flat or fill.startswith("url")) else self.lit(fill)
        out = f'<path d="{d}" fill="{base}" {extra}/>'
        g = ""
        for sh in shades:
            p, c, op = sh[:3]
            sd = sh[3] if len(sh) > 3 else soft
            g += f'<path d="{p}" fill="{c or cool_shade(col)}" opacity="{op}" filter="url(#{self.blur(sd)})"/>'
        for li in lights:
            p, c, op = li[:3]
            sd = li[3] if len(li) > 3 else soft
            g += f'<path d="{p}" fill="{c or warm_light(col)}" opacity="{op}" filter="url(#{self.blur(sd)})"/>'
        if rim and not flat:
            # A soft dark rim just inside the outline rounds the form.
            g += f'<path d="{d}" fill="none" stroke="{cool_shade(col)}" stroke-width="{w * 2.2:.1f}" opacity="0.35" filter="url(#{self.blur(max(1.5, w * 0.6))})"/>'
        if g:
            out += f'<g clip-path="url(#{cid})">{g}</g>'
        if w:
            out += f'<path d="{d}" fill="none" stroke="{ink}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"/>'
        self.add(out)

    def stroke(self, d, col=INK, w=4.5, op=1.0, extra=""):
        self.add(f'<path d="{d}" fill="none" stroke="{col}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round" opacity="{op}" {extra}/>')

    def fill(self, d, col, op=1.0, extra=""):
        self.add(f'<path d="{d}" fill="{col}" opacity="{op}" {extra}/>')

    def glow(self, d, col, op, sd):
        self.add(f'<path d="{d}" fill="{col}" opacity="{op}" filter="url(#{self.blur(sd)})"/>')

    def group(self, inner_fn, transform):
        """Run inner_fn to paint into a transformed group."""
        start = len(self.body)
        inner_fn()
        inner = "".join(self.body[start:])
        del self.body[start:]
        self.add(f'<g transform="{transform}">{inner}</g>')

    def svg(self, w, h, wrap=None):
        body = "".join(self.body)
        if wrap:
            body = f'<g transform="{wrap}">{body}</g>'
        return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" data-paint="done">\n'
                f'<defs>{"".join(self.defs)}</defs>\n{body}\n</svg>')

# ---------------------------------------------------------------- shape helpers
def ellipse(cx, cy, rx, ry):
    return f"M{cx - rx:.1f},{cy:.1f} A{rx:.1f},{ry:.1f} 0 1,0 {cx + rx:.1f},{cy:.1f} A{rx:.1f},{ry:.1f} 0 1,0 {cx - rx:.1f},{cy:.1f} Z"

def circle(cx, cy, r):
    return ellipse(cx, cy, r, r)

def limb(x0, y0, x1, y1, w0, w1):
    """A tapered limb from (x0,y0) (width w0) to (x1,y1) (width w1) with
    round ends."""
    import math
    dx, dy = x1 - x0, y1 - y0
    L = max(1e-3, math.hypot(dx, dy))
    nx, ny = -dy / L, dx / L
    a0 = (x0 + nx * w0 / 2, y0 + ny * w0 / 2); b0 = (x0 - nx * w0 / 2, y0 - ny * w0 / 2)
    a1 = (x1 + nx * w1 / 2, y1 + ny * w1 / 2); b1 = (x1 - nx * w1 / 2, y1 - ny * w1 / 2)
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2
    bulge = (w0 + w1) * 0.1
    return (f"M{a0[0]:.1f},{a0[1]:.1f} Q{mx + nx * ((w0 + w1) / 4 + bulge):.1f},{my + ny * ((w0 + w1) / 4 + bulge):.1f} {a1[0]:.1f},{a1[1]:.1f} "
            f"A{w1 / 2:.1f},{w1 / 2:.1f} 0 0,0 {b1[0]:.1f},{b1[1]:.1f} "
            f"Q{mx - nx * ((w0 + w1) / 4 + bulge):.1f},{my - ny * ((w0 + w1) / 4 + bulge):.1f} {b0[0]:.1f},{b0[1]:.1f} "
            f"A{w0 / 2:.1f},{w0 / 2:.1f} 0 0,0 {a0[0]:.1f},{a0[1]:.1f} Z")

# ---------------------------------------------------------------- people
SKIN = "#ffd6b8"

def person_eyes(art, lx, rx, y, r=9):
    """Big shiny key-art eyes for people (elves, Santa, Calgarians)."""
    for x in (lx, rx):
        art.part(ellipse(x, y, r, r * 1.2), "#ffffff", w=max(2.4, r * 0.33), rim=False)
        art.fill(circle(x + r * 0.2, y + r * 0.1, r * 0.62), "#2a1a14")
        art.fill(circle(x + r * 0.4, y - r * 0.22, r * 0.24), "#ffffff")

def cheeks(art, lx, rx, y, r=10):
    for x in (lx, rx):
        art.glow(ellipse(x, y, r, r * 0.65), "#ff8a7a", 0.75, max(1.5, r * 0.3))

# ---------------------------------------------------------------- character pass (2026-10)
# Rendered-cartoon painting for the pilots, Santa, elves and the reindeer:
# outlines in a deeper shade of each fill (heavier on the shadow side), a
# top-left lit gradient, a lit band inside the top-left edge, a soft core
# shadow inside the bottom-right edge with a thin cool bounce light from the
# snow right at the rim, and fur marks. Shapes are built from point lists
# (Catmull-Rom curves) with optional fur tufts along chosen edges.
import math, random as _random

def mix(a, b, t):
    ra, rb = _rgb(a), _rgb(b)
    return _hex(*(x + (y - x) * t for x, y in zip(ra, rb)))

def deep(c):
    """The outline colour for a fill: the same hue, much deeper."""
    h, s, v = colorsys.rgb_to_hsv(*_rgb(c))
    if s < 0.12:
        return _hex(*colorsys.hsv_to_rgb(0.61, 0.38, max(0.28, v * 0.42)))
    target = 0.0 if (h < 0.2 or h > 0.85) else 0.65
    d = ((target - h + 0.5) % 1) - 0.5
    return _hex(*colorsys.hsv_to_rgb((h + d * 0.15) % 1, min(1, s + 0.25), v * 0.4))

def bounce_of(c):
    """Cool light thrown up from the snow onto the underside of a form."""
    return tone(mix(c, "#cfe6ff", 0.5), 0.12)

def _cr_seg(p0, p1, p2, p3):
    c1 = (p1[0] + (p2[0] - p0[0]) / 6, p1[1] + (p2[1] - p0[1]) / 6)
    c2 = (p2[0] - (p3[0] - p1[0]) / 6, p2[1] - (p3[1] - p1[1]) / 6)
    return c1, c2

def _bez(p1, c1, c2, p2, t):
    u = 1 - t
    return (u * u * u * p1[0] + 3 * u * u * t * c1[0] + 3 * u * t * t * c2[0] + t * t * t * p2[0],
            u * u * u * p1[1] + 3 * u * u * t * c1[1] + 3 * u * t * t * c2[1] + t * t * t * p2[1])

def shape(pts, tufts=None, closed=True, seed=1):
    """A smooth path through `pts` (clockwise on screen for closed shapes).
    `tufts` maps a segment index i (pts[i] -> pts[i+1]) to (n, depth, lean):
    that edge becomes n curved fur tufts pointing outward, swept along the
    edge by `lean` (fraction of a tuft's width, negative sweeps backwards)."""
    tufts = tufts or {}
    rnd = _random.Random(seed)
    n = len(pts)
    P = lambda i: pts[i % n] if closed else pts[max(0, min(n - 1, i))]
    out = [f"M{pts[0][0]:.1f},{pts[0][1]:.1f}"]
    segs = n if closed else n - 1
    for i in range(segs):
        p0, p1, p2, p3 = P(i - 1), P(i), P(i + 1), P(i + 2)
        c1, c2 = _cr_seg(p0, p1, p2, p3)
        if i in tufts:
            k, dep, lean = tufts[i]
            pts_b = [_bez(p1, c1, c2, p2, j / k) for j in range(k + 1)]
            for j in range(k):
                a, b = pts_b[j], pts_b[j + 1]
                dx, dy = b[0] - a[0], b[1] - a[1]
                L = max(1e-3, math.hypot(dx, dy))
                nx, ny = dy / L, -dx / L
                dd = dep * (0.55 + 0.9 * rnd.random())
                mx, my = (a[0] + b[0]) / 2, (a[1] + b[1]) / 2
                tip = (mx + nx * dd + dx * lean, my + ny * dd + dy * lean)
                q1 = (a[0] + (tip[0] - a[0]) * 0.45 + nx * dd * 0.35, a[1] + (tip[1] - a[1]) * 0.45 + ny * dd * 0.35)
                q2 = (tip[0] + (b[0] - tip[0]) * 0.55 - nx * dd * 0.05, tip[1] + (b[1] - tip[1]) * 0.55 - ny * dd * 0.05)
                out.append(f"Q{q1[0]:.1f},{q1[1]:.1f} {tip[0]:.1f},{tip[1]:.1f} Q{q2[0]:.1f},{q2[1]:.1f} {b[0]:.1f},{b[1]:.1f}")
        else:
            out.append(f"C{c1[0]:.1f},{c1[1]:.1f} {c2[0]:.1f},{c2[1]:.1f} {p2[0]:.1f},{p2[1]:.1f}")
    if closed:
        out.append("Z")
    return " ".join(out)

def xform(pts, cx=0, cy=0, sx=1, sy=1, rot=0, dx=0, dy=0):
    """Scale and rotate points about (cx, cy), then move them."""
    c, s = math.cos(math.radians(rot)), math.sin(math.radians(rot))
    out = []
    for x, y in pts:
        x, y = (x - cx) * sx, (y - cy) * sy
        out.append((cx + x * c - y * s + dx, cy + x * s + y * c + dy))
    return out

def fur_marks(cx, cy, rx, ry, n, ang=90, ln=9, seed=3, curl=0.35):
    """Short curved fur strokes scattered in an ellipse, flowing along `ang`
    degrees (screen space). Returns one path string."""
    rnd = _random.Random(seed)
    out = []
    for _ in range(n):
        while True:
            u, v = rnd.uniform(-1, 1), rnd.uniform(-1, 1)
            if u * u + v * v <= 1:
                break
        x, y = cx + u * rx, cy + v * ry
        a = math.radians(ang + rnd.uniform(-18, 18))
        L = ln * rnd.uniform(0.7, 1.25)
        ex, ey = x + math.cos(a) * L, y + math.sin(a) * L
        qx, qy = (x + ex) / 2 - math.sin(a) * L * curl, (y + ey) / 2 + math.cos(a) * L * curl
        out.append(f"M{x:.1f},{y:.1f} Q{qx:.1f},{qy:.1f} {ex:.1f},{ey:.1f}")
    return " ".join(out)

def _paint(self, d, fill, w=4.0, ink=None, tf="", k=12, hi=0.55, core=0.55, bounce=0.6, shades=(), lights=(),
           fur=(), drop=1.0, grad=True, light=None, shade=None, bounce_col=None):
    """One part in the rendered-cartoon style (see the section comment).
    `k` is the depth of the edge light / core shadow in px. `fur` is a list
    of (path, colour, opacity, width) strokes clipped to the part."""
    T = lambda pre: f' transform="{pre} {tf}"' if (pre or tf) else ""
    ink = ink or deep(fill)
    lc, sc = light or warm_light(fill), shade or cool_shade(fill)
    if drop and w:
        self.add(f'<path d="{d}" fill="{ink}"{T(f"translate({0.9 * drop * w / 4:.2f} {1.5 * drop * w / 4:.2f})")}/>')
    if grad:
        gid = self.uid("lg")
        self.defs.append(f'<linearGradient id="{gid}" x1="0.25" y1="0" x2="0.7" y2="1"><stop offset="0" stop-color="{mix(fill, lc, 0.55)}"/>'
                         f'<stop offset="0.45" stop-color="{fill}"/><stop offset="1" stop-color="{tone(fill, -0.07, 0.04)}"/></linearGradient>')
        base = f"url(#{gid})"
    else:
        base = fill
    self.add(f'<path d="{d}" fill="{base}"{T("")}/>')
    cid = self.uid("c")
    self.defs.append(f'<clipPath id="{cid}"><path d="{d}"{T("")}/></clipPath>')
    g = ""
    if hi:
        g += (f'<path d="{d}" fill="none" stroke="{lc}" stroke-width="{k * 1.1:.1f}" opacity="{hi}" '
              f'filter="url(#{self.blur(round(k * 0.35, 1))})"{T(f"translate({k * 0.45:.1f} {k * 0.6:.1f})")}/>')
    for sh in shades:
        p, c, op = sh[:3]
        sd = sh[3] if len(sh) > 3 else 7
        g += f'<path d="{p}" fill="{c or sc}" opacity="{op}" filter="url(#{self.blur(sd)})"/>'
    if core:
        g += (f'<path d="{d}" fill="none" stroke="{sc}" stroke-width="{k * 2.2:.1f}" opacity="{core}" '
              f'filter="url(#{self.blur(round(k * 0.45, 1))})"{T(f"translate({-k * 0.55:.1f} {-k * 0.95:.1f})")}/>')
    for li in lights:
        p, c, op = li[:3]
        sd = li[3] if len(li) > 3 else 7
        g += f'<path d="{p}" fill="{c or lc}" opacity="{op}" filter="url(#{self.blur(sd)})"/>'
    for (p, c, op, fw) in fur:
        g += f'<path d="{p}" fill="none" stroke="{c}" stroke-width="{fw}" stroke-linecap="round" opacity="{op}"/>'
    if bounce:
        bid = self.uid("b")
        self.defs.append(f'<clipPath id="{bid}"><path d="{d}"{T(f"translate({k * 0.5:.1f} {k * 1.6:.1f})")}/></clipPath>')
        g += (f'<g clip-path="url(#{bid})"><path d="{d}" fill="none" stroke="{bounce_col or bounce_of(fill)}" stroke-width="{max(3.0, k * 0.55):.1f}" '
              f'opacity="{bounce}" filter="url(#{self.blur(1.2)})"{T("")}/></g>')
    if g:
        self.add(f'<g clip-path="url(#{cid})">{g}</g>')
    if w:
        self.add(f'<path d="{d}" fill="none" stroke="{ink}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"{T("")}/>')
    return cid

Art.paint = _paint

def _clipped(self, d, inner_fn, tf=""):
    """Paint whatever inner_fn adds clipped to the path d."""
    cid = self.uid("k")
    t = f' transform="{tf}"' if tf else ""
    self.defs.append(f'<clipPath id="{cid}"><path d="{d}"{t}/></clipPath>')
    start = len(self.body)
    inner_fn()
    inner = "".join(self.body[start:])
    del self.body[start:]
    self.add(f'<g clip-path="url(#{cid})">{inner}</g>')

Art.clipped = _clipped

def bent_limb(pts, widths, bend=0.0):
    """A chunky tapered limb along a polyline (hip, knee, ..., paw) with
    the given widths at each joint; `bend` bows the middle sideways (px)."""
    if bend and len(pts) == 2:
        (x0, y0), (x1, y1) = pts
        L = max(1e-3, math.hypot(x1 - x0, y1 - y0))
        nx, ny = -(y1 - y0) / L, (x1 - x0) / L
        pts = [pts[0], ((x0 + x1) / 2 + nx * bend, (y0 + y1) / 2 + ny * bend), pts[1]]
        widths = [widths[0], (widths[0] + widths[-1]) / 2, widths[-1]]
    left, right = [], []
    for i, (x, y) in enumerate(pts):
        a = pts[max(0, i - 1)]; b = pts[min(len(pts) - 1, i + 1)]
        dx, dy = b[0] - a[0], b[1] - a[1]
        L = max(1e-3, math.hypot(dx, dy))
        nx, ny = -dy / L, dx / L
        w = widths[i] / 2
        left.append((x + nx * w, y + ny * w)); right.append((x - nx * w, y - ny * w))
    # Round caps: a point beyond each end.
    (x0, y0), (x1, y1) = pts[0], pts[1]
    L = max(1e-3, math.hypot(x1 - x0, y1 - y0))
    cap0 = (x0 - (x1 - x0) / L * widths[0] * 0.45, y0 - (y1 - y0) / L * widths[0] * 0.45)
    (xa, ya), (xb, yb) = pts[-2], pts[-1]
    L = max(1e-3, math.hypot(xb - xa, yb - ya))
    cap1 = (xb + (xb - xa) / L * widths[-1] * 0.45, yb + (yb - ya) / L * widths[-1] * 0.45)
    ring = left + [cap1] + right[::-1] + [cap0]
    # Orient clockwise on screen (positive shoelace with y down).
    area = sum(ring[i][0] * ring[(i + 1) % len(ring)][1] - ring[(i + 1) % len(ring)][0] * ring[i][1] for i in range(len(ring)))
    if area < 0:
        ring = ring[::-1]
    return ring

import re as _re

def shift_path(d, dx, dy):
    """Translate an absolute-command path string by (dx, dy)."""
    toks = _re.findall(r"[A-Za-z]|-?\d*\.?\d+(?:e-?\d+)?", d)
    out, cmd, args = [], None, []
    nargs = {"M": 2, "L": 2, "T": 2, "C": 6, "S": 4, "Q": 4, "A": 7, "H": 1, "V": 1, "Z": 0}
    def flush():
        if cmd is None:
            return
        n = nargs.get(cmd.upper(), 0)
        if n == 0:
            out.append(cmd); return
        for i in range(0, len(args), n):
            chunk = [float(v) for v in args[i:i + n]]
            if cmd.isupper():
                if cmd == "A":
                    chunk[5] += dx; chunk[6] += dy
                elif cmd == "H":
                    chunk[0] += dx
                elif cmd == "V":
                    chunk[0] += dy
                else:
                    for j in range(0, n, 2):
                        chunk[j] += dx; chunk[j + 1] += dy
            out.append((cmd if i == 0 else ("L" if cmd == "M" else cmd)) + " ".join(f"{v:.1f}" if not float(v).is_integer() or cmd != "A" else f"{int(v)}" for v in chunk))
    for t in toks:
        if t.isalpha():
            flush(); cmd, args = t, []
            if t in "Zz":
                out.append("Z"); cmd = None
        else:
            args.append(t)
    flush()
    return " ".join(out)

def _overlay(self, d, k=16, hi=0.5, core=0.4, bounce=0.55, rim=0.0, shade="#7c84c4", light="#fff1cf", bounce_col="#bfe2ff"):
    """Form lighting over an already filled, multi-coloured part (a head
    with markings): multiply a cool core shadow into the bottom-right,
    screen a warm light band inside the top-left edge and a cool bounce
    along the bottom rim. Works across every colour inside the part.
    Each stroke carries its own clip: a clipped group would isolate the
    blend from the art underneath."""
    cid = self.uid("o")
    self.defs.append(f'<clipPath id="{cid}"><path d="{d}"/></clipPath>')
    if hi:
        self.add(self._fx(shift_path(d, k * 0.45, k * 0.6), light, k * 1.1, hi, round(k * 0.4, 1), "screen", cid))
    if core:
        self.add(self._fx(shift_path(d, -k * 0.5, -k * 0.95), shade, k * 2.2, core, round(k * 0.5, 1), "multiply", cid))
    if bounce:
        bid = self.uid("b")
        self.defs.append(f'<clipPath id="{bid}" clip-path="url(#{cid})"><path d="{shift_path(d, k * 0.4, k * 1.7)}"/></clipPath>')
        self.add(self._fx(d, bounce_col, max(3.0, k * 0.5), bounce, 1.4, "screen", bid))
    if rim:
        rid = self.uid("r")
        self.defs.append(f'<clipPath id="{rid}" clip-path="url(#{cid})"><path d="{shift_path(d, k * 0.9, -k * 0.3)}"/></clipPath>')
        self.add(self._fx(d, "#e8f6ff", max(2.5, k * 0.35), rim, 0.8, "normal", rid))

Art.overlay = _overlay

def _fx(self, d, col, sw, op, sd, mode, clip):
    return (f'<path d="{d}" fill="none" stroke="{col}" stroke-width="{sw:.1f}" opacity="{op}" '
            f'filter="url(#{self.blur(sd)})" clip-path="url(#{clip})" style="mix-blend-mode:{mode}"/>')

Art._fx = _fx
Art.overlay = _overlay

def _blend(self, d, col, op, sd, mode="multiply", clip=None):
    """A soft blended shape (shadow or light) over the art below, clipped
    to the clipPath id `clip` (see clip_id)."""
    c = f' clip-path="url(#{clip})"' if clip else ""
    self.add(f'<path d="{d}" fill="{col}" opacity="{op}" style="mix-blend-mode:{mode}" filter="url(#{self.blur(sd)})"{c}/>')

def _clip_id(self, d):
    cid = self.uid("ci")
    self.defs.append(f'<clipPath id="{cid}"><path d="{d}"/></clipPath>')
    return cid

Art.clip_id = _clip_id

Art.blend = _blend
