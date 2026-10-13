"""Downtown Calgary on the horizon, hand-painted after the key art in two
depths: `skyline` (the near layer with the landmark towers, every one
recognisable) and `skyline_far` (a pale, hazy second rank of towers and
cranes behind it). Imported by tools/gen_city.py, which writes the files.

Every tower is a painted shape: top-lit gradient fill, an outline in a deeper
shade of its own colour, curtain-wall or punched-window grids, warm lit
offices with a soft glow, painted sky reflections (soft diagonal bands and a
reflected cloud), snow on every roof and ledge, and the haze of distance
rising from the bottom.
"""
import math, random

SNOW_EDGE = "#8fb0d0"
WARM, WARM_GLOW = "#ffd98a", "#ffb84a"


def _rgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))
def mixc(a, b, t):
    ra, rb = _rgb(a), _rgb(b)
    return "#%02x%02x%02x" % tuple(int(x + (y - x) * t) for x, y in zip(ra, rb))
def deep(c, k=0.62):
    """Outline: the fill's own colour, deeper and a touch warmer."""
    r, g, b = _rgb(c)
    return "#%02x%02x%02x" % (int(r * k * 1.02), int(g * k), int(b * k * 0.98))
def lift(c, k=0.35):
    return mixc(c, "#ffffff", k)


class Canvas:
    def __init__(self, W, H, haze, seed=1, fog=0.0):
        self.W, self.H = W, H
        self.fog = fog
        self.G = H  # ground line
        self.haze = haze
        self.defs, self.body = [], []
        self.n = 0
        self.blurs = set()
        self.rnd = random.Random(seed)

    def uid(self, p):
        self.n += 1
        return f"{p}{self.n}"

    def blur(self, sd):
        fid = "sb" + str(sd).replace(".", "_")
        if fid not in self.blurs:
            self.blurs.add(fid)
            self.defs.append(f'<filter id="{fid}" x="-100%" y="-100%" width="300%" height="300%"><feGaussianBlur stdDeviation="{sd}"/></filter>')
        return f'filter="url(#{fid})"'

    def lgrad(self, stops, x1=0, y1=0, x2=1, y2=0):
        gid = self.uid("lg")
        s = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{a}"/>' for o, c, a in stops)
        self.defs.append(f'<linearGradient id="{gid}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">{s}</linearGradient>')
        return f"url(#{gid})"

    def clip(self, d):
        cid = self.uid("cp")
        self.defs.append(f'<clipPath id="{cid}"><path d="{d}"/></clipPath>')
        return cid

    def add(self, s):
        self.body.append(s)

    def svg(self, attrs=""):
        return "".join(self.body), "".join(self.defs)

    # ---------------------------------------------------------------- pieces
    def snow(self, x0, x1, y, h=10, drip=True):
        """A lumpy painted snow cap on a ledge whose top is at y."""
        rnd = self.rnd
        n = max(1, round((x1 - x0) / 22))
        step = (x1 - x0) / n
        d = f"M{x0 - 3:.1f},{y + 3:.1f} "
        for i in range(n):
            xa = x0 + i * step
            d += f"Q{xa + step * 0.5:.1f},{y - h * rnd.uniform(0.7, 1.2):.1f} {xa + step:.1f},{y - h * rnd.uniform(0.0, 0.25):.1f} "
        d += f"L{x1 + 3:.1f},{y + 3:.1f} "
        # Soft underside with a couple of drips.
        if drip and x1 - x0 > 30:
            for k in range(max(1, int((x1 - x0) / 60))):
                cx = x1 - (k + 0.5) * (x1 - x0) / max(1, int((x1 - x0) / 60)) + rnd.uniform(-8, 8)
                d += f"L{cx + 4:.1f},{y + 3:.1f} Q{cx:.1f},{y + 3 + h * 0.7:.1f} {cx - 4:.1f},{y + 3:.1f} "
        d += "Z"
        fill = self.lgrad([(0, "#ffffff", 1), (0.6, "#f4f8ff", 1), (1, "#cfe0f2", 1)], 0, 0, 0, 1)
        self.add(f'<path d="{d}" fill="{fill}" stroke="{SNOW_EDGE}" stroke-width="2" stroke-linejoin="round"/>')

    def glow_rect(self, x, y, w, h, op=0.9):
        self.add(f'<rect x="{x - 2:.1f}" y="{y - 2:.1f}" width="{w + 4:.1f}" height="{h + 4:.1f}" fill="{WARM_GLOW}" opacity="{op * 0.55:.2f}" {self.blur(2.5)}/>')
        self.add(f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" fill="{WARM}" opacity="{op:.2f}"/>')

    def body_fill(self, d, col, x0, x1, light_left=True):
        """Top-lit, side-lit painted fill of a tower silhouette."""
        a, b = (lift(col, 0.28), deep(col, 0.86)) if light_left else (deep(col, 0.86), lift(col, 0.28))
        side = self.lgrad([(0, a, 1), (0.45, col, 1), (1, b, 1)])
        self.add(f'<path d="{d}" fill="{side}"/>')

    def reflections(self, x0, x1, y0, y1, strength=1.0):
        """Painted sky in the glass: a bright top, soft diagonal bands and a
        reflected cloud."""
        w, h = x1 - x0, y1 - y0
        rnd = self.rnd
        top = self.lgrad([(0, "#f2f8ff", 0.55 * strength), (0.35, "#e0eeff", 0.12 * strength), (1, "#ffffff", 0)], 0, 0, 0, 1)
        self.add(f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="{top}"/>')
        for k in range(2):
            bx = x0 + w * rnd.uniform(-0.2, 0.5)
            by = y0 + h * rnd.uniform(0.15, 0.6)
            bw = w * rnd.uniform(0.25, 0.45)
            sl = w * 1.4
            self.add(f'<path d="M{bx:.1f},{by + sl:.1f} L{bx + sl:.1f},{by:.1f} L{bx + sl + bw:.1f},{by:.1f} L{bx + bw:.1f},{by + sl:.1f} Z" '
                     f'fill="#ffffff" opacity="{0.18 * strength:.2f}" {self.blur(5)}/>')
        cy = y0 + h * rnd.uniform(0.2, 0.5)
        self.add(f'<ellipse cx="{x0 + w * rnd.uniform(0.3, 0.8):.1f}" cy="{cy:.1f}" rx="{w * 0.5:.1f}" ry="{min(40, h * 0.05):.1f}" fill="#ffffff" opacity="{0.28 * strength:.2f}" {self.blur(9)}/>')

    def curtain(self, x0, x1, y0, y1, col, floor=20, mull=14, lit=0.05, rows_lit=0.15):
        """Curtain wall: floor bands, mullions, lit offices in runs."""
        rnd = self.rnd
        s = ""
        for y in range(int(y0) + floor, int(y1), floor):
            s += f'<path d="M{x0},{y} H{x1}" stroke="{deep(col, 0.7)}" stroke-width="2" opacity="0.45"/>'
            s += f'<path d="M{x0},{y + 2} H{x1}" stroke="#ffffff" stroke-width="1" opacity="0.25"/>'
        x = x0 + mull
        while x < x1 - 2:
            s += f'<path d="M{x:.1f},{y0} V{y1}" stroke="{lift(col, 0.4)}" stroke-width="1.2" opacity="0.35"/>'
            x += mull
        self.add(s)
        for y in range(int(y0) + floor, int(y1) - floor, floor):
            if rnd.random() < rows_lit:
                a = x0 + mull * rnd.randint(0, 3)
                b = min(x1 - 3, a + mull * rnd.randint(2, 6))
                self.glow_rect(a + 2, y + 4, b - a - 4, floor - 8, 0.85)
            else:
                xx = x0
                while xx < x1 - mull:
                    if rnd.random() < lit:
                        self.glow_rect(xx + 2, y + 4, mull - 4, floor - 8, 0.85)
                    xx += mull

    def punched(self, x0, x1, y0, y1, col, fw=14, fh=18, gx=10, gy=12, lit=0.12, glass="#3d5d86"):
        """Punched windows in a stone or concrete wall, each with a sill of snow."""
        rnd = self.rnd
        cols = max(1, int((x1 - x0 - gx) // (fw + gx)))
        pad = (x1 - x0 - cols * fw - (cols - 1) * gx) / 2
        y = y0 + gy
        while y + fh < y1 - 4:
            for i in range(cols):
                x = x0 + pad + i * (fw + gx)
                if rnd.random() < lit:
                    self.glow_rect(x, y, fw, fh, 0.95)
                else:
                    self.add(f'<rect x="{x:.1f}" y="{y:.1f}" width="{fw}" height="{fh}" fill="{glass}"/>'
                             f'<rect x="{x:.1f}" y="{y:.1f}" width="{fw}" height="{fh * 0.4:.1f}" fill="#bcd8f2" opacity="0.45"/>')
                self.add(f'<rect x="{x - 1:.1f}" y="{y + fh - 1:.1f}" width="{fw + 2}" height="3" fill="#ffffff" opacity="0.9"/>')
            y += fh + gy

    def base_haze(self, x0, x1, y0, y1, k=1.0):
        g = self.lgrad([(0, self.haze, 0), (0.55, self.haze, 0.35 * k), (1, self.haze, 0.85 * k)], 0, 0, 0, 1)
        self.add(f'<rect x="{x0}" y="{y0}" width="{x1 - x0}" height="{y1 - y0}" fill="{g}"/>')

    def outline(self, d, col, w=3):
        self.add(f'<path d="{d}" fill="none" stroke="{deep(col, 0.55)}" stroke-width="{w}" stroke-linejoin="round"/>')

    def tower(self, d, bbox, col, kind="glass", light_left=True, floor=20, mull=14, lit=0.05, rows_lit=0.15,
              ledges=(), refl=1.0, haze=1.0, extra=None, w=3):
        """Paint one tower: silhouette `d` inside bbox (x0, y0, x1, y1)."""
        x0, y0, x1, y1 = bbox
        cid = self.clip(d)
        self.body_fill(d, col, x0, x1, light_left)
        self.add(f'<g clip-path="url(#{cid})">')
        if kind == "glass":
            self.curtain(x0, x1, y0, y1, col, floor, mull, lit, rows_lit)
            self.reflections(x0, x1, y0, y1, refl)
        elif kind == "stone":
            self.punched(x0 + 6, x1 - 6, y0 + 10, y1, col, lit=max(lit, 0.1))
            top = self.lgrad([(0, "#ffffff", 0.35), (0.3, "#ffffff", 0.0)], 0, 0, 0, 1)
            self.add(f'<rect x="{x0}" y="{y0}" width="{x1 - x0}" height="{y1 - y0}" fill="{top}"/>')
        if extra:
            extra(x0, y0, x1, y1)
        # Shade side: a soft cool band down the dark edge.
        sx = x1 - (x1 - x0) * 0.18 if light_left else x0
        self.add(f'<rect x="{sx:.1f}" y="{y0}" width="{(x1 - x0) * 0.18:.1f}" height="{y1 - y0}" fill="#1a2a55" opacity="0.13"/>')
        self.base_haze(x0, x1, y0 + (y1 - y0) * 0.45, y1, haze)
        if self.fog:
            self.add(f'<rect x="{x0}" y="{y0}" width="{x1 - x0}" height="{y1 - y0}" fill="{self.haze}" opacity="{self.fog}"/>')
        self.add("</g>")
        self.outline(d, col, w)
        for (a, b, y) in ledges:
            self.snow(a, b, y)

    def crane(self, x, h, jib_l=200, jib_r=90, col="#f0b429", op=1.0):
        """A tower crane with a lattice mast, painted yellow, snow on the jib."""
        G = self.G
        top = G - h
        s = f'<g opacity="{op}">'
        mw = 16
        s += f'<rect x="{x - mw / 2}" y="{top}" width="{mw}" height="{h}" fill="none" stroke="{deep(col, 0.7)}" stroke-width="7"/>'
        s += f'<rect x="{x - mw / 2}" y="{top}" width="{mw}" height="{h}" fill="none" stroke="{col}" stroke-width="3.5"/>'
        y = top
        lat = ""
        while y < G - 16:
            lat += f"M{x - mw / 2},{y} L{x + mw / 2},{y + 16} M{x + mw / 2},{y} L{x - mw / 2},{y + 16} "
            y += 16
        s += f'<path d="{lat}" stroke="{deep(col, 0.75)}" stroke-width="1.6" fill="none"/>'
        jy = top + 4
        s += f'<path d="M{x - jib_l},{jy} L{x + jib_r},{jy} L{x + jib_r},{jy + 12} L{x - jib_l},{jy + 12} Z" fill="{col}" stroke="{deep(col, 0.6)}" stroke-width="3"/>'
        zig = "".join(f"M{x - jib_l + i * 14},{jy} L{x - jib_l + i * 14 + 7},{jy + 12} L{x - jib_l + i * 14 + 14},{jy} " for i in range(int((jib_l + jib_r) / 14)))
        s += f'<path d="{zig}" stroke="{deep(col, 0.7)}" stroke-width="1.4" fill="none"/>'
        s += f'<path d="M{x},{top - 34} L{x - jib_l * 0.8},{jy} M{x},{top - 34} L{x + jib_r},{jy}" stroke="{deep(col, 0.5)}" stroke-width="2"/>'
        s += f'<path d="M{x - 6},{top} L{x},{top - 36} L{x + 6},{top} Z" fill="{col}" stroke="{deep(col, 0.6)}" stroke-width="2.5"/>'
        s += f'<rect x="{x + jib_r - 34}" y="{jy + 12}" width="30" height="22" fill="#9aa6b8" stroke="#5d6a80" stroke-width="2.5"/>'
        s += f'<rect x="{x + 6}" y="{jy + 12}" width="20" height="18" rx="3" fill="#ffffff" stroke="#8a96aa" stroke-width="2.5"/>'
        hx = x - jib_l * 0.6
        s += f'<path d="M{hx},{jy + 12} L{hx},{jy + 140}" stroke="#4a5568" stroke-width="2"/>'
        s += f'<rect x="{hx - 16}" y="{jy + 140}" width="32" height="20" fill="#c9a26a" stroke="#7a5a30" stroke-width="2.5"/>'
        s += f'<circle cx="{x}" cy="{top - 36}" r="4" fill="#ff4a4a"/>'
        s += "</g>"
        self.add(s)
        self.snow(x - jib_l, x + jib_r, jy, 5, drip=False)


# ====================================================================== NEAR LAYER
def near():
    """The landmark rank: Bankers Hall, the Calgary Tower, Telus Sky, the Bow,
    Brookfield Place, Suncor, Eighth Avenue Place and Centennial Place, with
    older stone blocks and parkades along the bottom."""
    W, H = 2800, 1000
    c = Canvas(W, H, "#c9dcf2", seed=42)
    G = H
    rnd = c.rnd

    def flat(x, w, h, col, kind="glass", step=None, **kw):
        y = G - h
        d = f"M{x},{G} L{x},{y} L{x + w},{y} L{x + w},{G} Z"
        if step:  # a setback penthouse
            sw_, sh_ = step
            d = f"M{x},{G} L{x},{y} L{x + (w - sw_) / 2},{y} L{x + (w - sw_) / 2},{y - sh_} L{x + (w + sw_) / 2},{y - sh_} L{x + (w + sw_) / 2},{y} L{x + w},{y} L{x + w},{G} Z"
        led = [(x, x + w, y)] + ([(x + (w - step[0]) / 2, x + (w + step[0]) / 2, y - step[1])] if step else [])
        c.tower(d, (x, y - (step[1] if step else 0), x + w, G), col, kind, ledges=led, **kw)

    # ---- back row of the near layer: mid towers that fill the gaps
    for (x, w, h, col, kind) in ((40, 130, 470, "#8fb0d6", "glass"), (230, 110, 560, "#a7b9cf", "stone"),
                                 (1000, 100, 520, "#9fbad8", "glass"), (1560, 110, 600, "#86a7d0", "glass"),
                                 (2330, 130, 520, "#a3b8d2", "glass"), (2560, 120, 440, "#b2c0d0", "stone")):
        flat(x, w, h, col, kind, step=(w * 0.5, 26), haze=1.3, refl=0.7)
    c.crane(360, 700, 220, 90)
    c.crane(2480, 640, 160, 110)

    # ---- Bankers Hall: twin towers, salmon granite with dark glass bands,
    #      stepped tops under copper peaks.
    def bankers(x, w, h):
        y = G - h
        col = "#d7aa8e"
        st = 18
        d = (f"M{x},{G} L{x},{y + 2 * st} L{x + st},{y + 2 * st} L{x + st},{y + st} L{x + 2 * st},{y + st} L{x + 2 * st},{y} "
             f"L{x + w - 2 * st},{y} L{x + w - 2 * st},{y + st} L{x + w - st},{y + st} L{x + w - st},{y + 2 * st} L{x + w},{y + 2 * st} L{x + w},{G} Z")
        def bands(x0, y0, x1, y1):
            for yy in range(int(y0) + 60, G, 26):
                c.add(f'<rect x="{x0}" y="{yy}" width="{x1 - x0}" height="12" fill="#3d4f6e" opacity="0.85"/>')
                c.add(f'<rect x="{x0}" y="{yy}" width="{x1 - x0}" height="4" fill="#a9c8ec" opacity="0.5"/>')
                if rnd.random() < 0.3:
                    a = x0 + rnd.uniform(4, (x1 - x0) * 0.5)
                    c.glow_rect(a, yy + 2, rnd.uniform(20, 50), 8, 0.9)
            for xx in range(int(x0) + 14, int(x1), 14):
                c.add(f'<path d="M{xx},{y0 + 60} V{G}" stroke="#b78a70" stroke-width="2" opacity="0.6"/>')
            c.reflections(x0, x1, y0 + 60, y1, 0.6)
        c.tower(d, (x, y, x + w, G), col, kind="none", extra=bands,
                ledges=[(x, x + st, y + 2 * st), (x + st, x + 2 * st, y + st), (x + w - 2 * st, x + w - st, y + st), (x + w - st, x + w, y + 2 * st)])
        # The copper crown.
        cx = x + w / 2
        crown = f"M{x + 2 * st - 4},{y + 2} L{cx},{y - 70} L{x + w - 2 * st + 4},{y + 2} Z"
        c.tower(crown, (x + 2 * st - 4, y - 70, x + w - 2 * st + 4, y + 2), "#5fa08a", kind="none", haze=0.2)
        c.add(f'<path d="M{cx},{y - 70} L{cx},{y - 96}" stroke="#4a5568" stroke-width="3"/><circle cx="{cx}" cy="{y - 98}" r="4" fill="#ff5a4a"/>')
        c.add(f'<path d="M{cx - 10},{y - 54} L{cx},{y - 70} L{cx + 10},{y - 54} Q{cx},{y - 58} {cx - 10},{y - 54} Z" fill="#ffffff" stroke="{SNOW_EDGE}" stroke-width="1.5"/>')
    bankers(420, 150, 710)
    bankers(600, 150, 680)

    # ---- The Calgary Tower: fluted concrete shaft, flared pod, red top, torch.
    cx = 880
    top = G - 690
    shaft = f"M{cx - 46},{G} C{cx - 36},{G - 260} {cx - 26},{top + 120} {cx - 24},{top + 70} L{cx + 24},{top + 70} C{cx + 26},{top + 120} {cx + 36},{G - 260} {cx + 46},{G} Z"
    def flutes(x0, y0, x1, y1):
        for k in (-14, -5, 5, 14):
            c.add(f'<path d="M{cx + k * 1.8},{G} L{cx + k},{top + 70}" stroke="#a89f90" stroke-width="2.5" opacity="0.7"/>')
        # The elevator lights running up the shaft.
        for yy in range(top + 110, G, 70):
            c.glow_rect(cx - 3, yy, 6, 10, 0.9)
    c.tower(shaft, (cx - 46, top + 70, cx + 46, G), "#e4ddd0", kind="none", extra=flutes, haze=1.1)
    # Pod: underside flare, glass observation band, red crown, roof.
    under = f"M{cx - 26},{top + 76} L{cx - 96},{top + 34} L{cx + 96},{top + 34} L{cx + 26},{top + 76} Z"
    c.tower(under, (cx - 96, top + 34, cx + 96, top + 76), "#ece6dc", kind="none", haze=0)
    band = f"M{cx - 96},{top + 34} L{cx - 102},{top + 6} L{cx + 102},{top + 6} L{cx + 96},{top + 34} Z"
    def obs(x0, y0, x1, y1):
        for xx in range(int(x0) + 6, int(x1), 12):
            c.add(f'<path d="M{xx},{y0} V{y1}" stroke="#d8e8f8" stroke-width="1.5" opacity="0.6"/>')
        for xx in range(int(x0) + 20, int(x1) - 20, 36):
            c.glow_rect(xx, y0 + 8, 18, 12, 0.85)
        c.reflections(x0, x1, y0, y1, 1.0)
    c.tower(band, (cx - 102, top + 6, cx + 102, top + 34), "#4f7fb6", kind="none", extra=obs, haze=0)
    red = f"M{cx - 102},{top + 6} L{cx - 90},{top - 22} L{cx + 90},{top - 22} L{cx + 102},{top + 6} Z"
    def stripe(x0, y0, x1, y1):
        c.add(f'<rect x="{x0}" y="{y0 + 10}" width="{x1 - x0}" height="7" fill="#ffffff" opacity="0.85"/>')
    c.tower(red, (cx - 102, top - 22, cx + 90, top + 6), "#d8343a", kind="none", extra=stripe, haze=0)
    roof = f"M{cx - 90},{top - 22} L{cx - 60},{top - 40} L{cx + 60},{top - 40} L{cx + 90},{top - 22} Z"
    c.tower(roof, (cx - 90, top - 40, cx + 90, top - 22), "#e9e4dc", kind="none", haze=0)
    c.snow(cx - 60, cx + 60, top - 40, 9)
    # Christmas lights along the pod's rim.
    cols = ["#ff4a4a", "#ffd23a", "#3fbf7a", "#4aa3ff", "#ff8ad8"]
    for i, xx in enumerate(range(cx - 96, cx + 97, 12)):
        yy = top + 36 + 4 * math.sin(i * 1.3)
        c.add(f'<circle cx="{xx}" cy="{yy:.1f}" r="7" fill="{cols[i % 5]}" opacity="0.45" {c.blur(3)}/><circle cx="{xx}" cy="{yy:.1f}" r="3.5" fill="{cols[i % 5]}"/>')
    # The torch, lit for Christmas, with a glow.
    c.add(f'<path d="M{cx - 10},{top - 40} L{cx - 6},{top - 70} L{cx + 6},{top - 70} L{cx + 10},{top - 40} Z" fill="#b8b0a0" stroke="#6e6658" stroke-width="2.5"/>')
    c.add(f'<rect x="{cx - 18}" y="{top - 82}" width="36" height="14" rx="3" fill="#d7d1c4" stroke="#6e6658" stroke-width="2.5"/>')
    c.add(f'<ellipse cx="{cx}" cy="{top - 130}" rx="46" ry="62" fill="#ffb03a" opacity="0.45" {c.blur(14)}/>')
    flame = f"M{cx},{top - 196} C{cx - 30},{top - 150} {cx - 26},{top - 100} {cx - 14},{top - 82} L{cx + 14},{top - 82} C{cx + 26},{top - 100} {cx + 30},{top - 150} {cx},{top - 196} Z"
    c.add(f'<path d="{flame}" fill="{c.lgrad([(0, "#ffb02e", 1), (0.6, "#ff7a1a", 1), (1, "#e2401a", 1)], 0, 0, 0, 1)}" stroke="#b8361a" stroke-width="3"/>')
    c.add(f'<path d="M{cx},{top - 164} C{cx - 14},{top - 132} {cx - 12},{top - 100} {cx - 6},{top - 86} L{cx + 6},{top - 86} C{cx + 12},{top - 100} {cx + 14},{top - 132} {cx},{top - 164} Z" fill="#ffe27a"/>')

    # ---- Telus Sky: a plain office base, then the residential floors that
    #      shift side to side towards the top; the LED artwork glows on it.
    tx, tw = 1100, 150
    base_h = 470
    pts = [(tx, G), (tx, G - base_h)]
    right = [(tx + tw, G), (tx + tw, G - base_h)]
    steps = 14
    for k in range(steps):
        off = 26 * math.sin(k / steps * math.pi * 1.2)
        narrow = k * 1.8
        y = G - base_h - k * 26
        pts += [(tx + off + narrow, y), (tx + off + narrow, y - 26)]
        right += [(tx + tw + off - narrow, y), (tx + tw + off - narrow, y - 26)]
    ttop = G - base_h - steps * 26
    d = "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts + right[::-1]) + " Z"
    def leds(x0, y0, x1, y1):
        cols = ["#ff5a7a", "#ffd23a", "#4ae0c0", "#7a9aff"]
        for k in range(steps):
            off = 26 * math.sin(k / steps * math.pi * 1.2)
            y = G - base_h - k * 26
            c.add(f'<rect x="{tx + off + k * 1.8:.1f}" y="{y - 4}" width="{tw - k * 3.6:.1f}" height="4" fill="#ffffff" opacity="0.55"/>')
        # A sweeping LED ribbon down the face.
        c.add(f'<path d="M{tx + 20},{G - 120} C{tx + 140},{G - 260} {tx + 10},{G - 360} {tx + 120},{G - 500}" stroke="{cols[0]}" stroke-width="10" fill="none" opacity="0.75" {c.blur(2)}/>')
        c.add(f'<path d="M{tx + 30},{G - 140} C{tx + 150},{G - 280} {tx + 20},{G - 380} {tx + 130},{G - 520}" stroke="{cols[2]}" stroke-width="6" fill="none" opacity="0.7"/>')
    c.tower(d, (tx - 10, ttop - 26, tx + tw + 40, G), "#7fa8d6", kind="glass", extra=leds, floor=22, mull=15, lit=0.07)
    c.snow(tx + 26 * math.sin((steps - 1) / steps * math.pi * 1.2) + (steps - 1) * 1.8, tx + tw + 26 * math.sin((steps - 1) / steps * math.pi * 1.2) - (steps - 1) * 1.8, ttop - 26, 9)

    # ---- The Bow: a crescent of glass under a silver diagrid, the sky garden notches.
    x0, x1, h = 1280, 1500, 850
    yt = G - h
    bow = (f"M{x0 + 6},{G} C{x0 - 10},{G - 300} {x0 - 6},{yt + 200} {x0 + 10},{yt + 30} L{x0 + 10},{yt} L{x1 - 50},{yt} L{x1 - 50},{yt + 18} "
           f"L{x1 - 10},{yt + 18} C{x1 + 6},{yt + 200} {x1 + 10},{G - 300} {x1 - 6},{G} Z")
    def diagrid(a, b, c_, d_):
        g = ""
        cell = 110
        for k in range(-10, 20):
            y0_ = G - k * cell
            g += f"M{x0},{y0_} L{x1},{y0_ - 220} M{x0},{y0_ - 220} L{x1},{y0_} "
        c.add(f'<path d="{g}" stroke="#4a6588" stroke-width="7" fill="none" opacity="0.55"/>')
        c.add(f'<path d="{g}" stroke="#eef5ff" stroke-width="3.5" fill="none" opacity="0.85"/>')
        for yy in range(yt + 210, G, 220):
            c.add(f'<path d="M{x0},{yy} H{x1}" stroke="#eef5ff" stroke-width="2.5" opacity="0.6"/>')
    c.tower(bow, (x0 - 10, yt, x1 + 10, G), "#6d9ccc", kind="glass", light_left=False, extra=diagrid, floor=18, mull=16, lit=0.04, rows_lit=0.1, refl=1.2,
            ledges=[(x0 + 10, x1 - 50, yt), (x1 - 50, x1 - 10, yt + 18)])

    # ---- Brookfield Place: the tallest, slim blue-green glass with a lit spine.
    bx, bw, bh = 1610, 150, 930
    by = G - bh
    bp = f"M{bx},{G} L{bx},{by + 30} L{bx + 20},{by + 30} L{bx + 20},{by} L{bx + bw - 20},{by} L{bx + bw - 20},{by + 30} L{bx + bw},{by + 30} L{bx + bw},{G} Z"
    def spine(a, b, c_, d_):
        for k in range(6):
            xx = bx + 25 + k * 20
            c.add(f'<path d="M{xx},{by} V{G}" stroke="#d6f0ff" stroke-width="2.5" opacity="0.45"/>')
        c.add(f'<rect x="{bx + bw / 2 - 10}" y="{by + 30}" width="20" height="{bh - 30}" fill="#eaf6ff" opacity="0.35"/>')
    c.tower(bp, (bx, by, bx + bw, G), "#5b93b8", kind="glass", extra=spine, floor=20, mull=12, lit=0.05, refl=1.2,
            ledges=[(bx + 20, bx + bw - 20, by), (bx, bx + 20, by + 30), (bx + bw - 20, bx + bw, by + 30)])
    c.add(f'<path d="M{bx + bw / 2},{by} L{bx + bw / 2},{by - 50}" stroke="#4a5568" stroke-width="3"/><circle cx="{bx + bw / 2}" cy="{by - 52}" r="5" fill="#ff4a4a"/>')

    # ---- Suncor Energy Centre: twin towers of red granite with slanted, notched crowns.
    def suncor(x, w, h, flip=False):
        y = G - h
        sl = 70
        if not flip:
            d = f"M{x},{G} L{x},{y} L{x + w * 0.45},{y + sl * 0.45} L{x + w * 0.45},{y + sl * 0.3} L{x + w},{y + sl} L{x + w},{G} Z"
        else:
            d = f"M{x},{G} L{x},{y + sl} L{x + w * 0.55},{y + sl * 0.3} L{x + w * 0.55},{y + sl * 0.45} L{x + w},{y} L{x + w},{G} Z"
        def strips(a, b, c_, d_):
            for xx in range(int(x) + 12, int(x + w) - 8, 18):
                c.add(f'<rect x="{xx}" y="{y}" width="9" height="{h}" fill="#2f3f5e" opacity="0.8"/>')
                c.add(f'<rect x="{xx}" y="{y}" width="3" height="{h}" fill="#9fc0e6" opacity="0.5"/>')
            for yy in range(y + 90, G, 22):
                if rnd.random() < 0.25:
                    xx = int(x) + 12 + 18 * rnd.randint(0, int(w / 18) - 2)
                    c.glow_rect(xx, yy, 9, 14, 0.9)
            c.reflections(x, x + w, y, G, 0.5)
        c.tower(d, (x, y, x + w, G), "#b7615a", kind="none", extra=strips)
        # Snow along the slope.
        if not flip:
            c.add(f'<path d="M{x - 2},{y - 2} L{x + w * 0.45},{y + sl * 0.45 - 2} M{x + w * 0.45},{y + sl * 0.3 - 2} L{x + w + 2},{y + sl - 2}" stroke="#ffffff" stroke-width="7" stroke-linecap="round"/>')
        else:
            c.add(f'<path d="M{x - 2},{y + sl - 2} L{x + w * 0.55},{y + sl * 0.3 - 2} M{x + w * 0.55},{y + sl * 0.45 - 2} L{x + w + 2},{y - 2}" stroke="#ffffff" stroke-width="7" stroke-linecap="round"/>')
    suncor(1830, 150, 800)
    suncor(1990, 110, 640, flip=True)

    # ---- Eighth Avenue Place: two blue towers with faceted crystal tops.
    def eap(x, w, h):
        y = G - h
        d = (f"M{x},{G} L{x},{y + 60} L{x + w * 0.25},{y + 10} L{x + w * 0.5},{y + 40} L{x + w * 0.62},{y} "
             f"L{x + w},{y + 50} L{x + w},{G} Z")
        def facets(a, b, c_, d_):
            c.add(f'<path d="M{x + w * 0.25},{y + 10} L{x + w * 0.4},{y + 140} L{x + w * 0.62},{y} M{x + w * 0.4},{y + 140} L{x},{y + 60} M{x + w * 0.4},{y + 140} L{x + w},{y + 50}" stroke="#e6f2ff" stroke-width="3" fill="none" opacity="0.8"/>')
            c.add(f'<path d="M{x + w * 0.4},{y + 140} L{x + w * 0.62},{y} L{x + w},{y + 50} Z" fill="#ffffff" opacity="0.18"/>')
        c.tower(d, (x, y, x + w, G), "#5f87c2", kind="glass", extra=facets, floor=20, mull=14)
        c.add(f'<path d="M{x},{y + 58} L{x + w * 0.25},{y + 8} L{x + w * 0.5},{y + 38} L{x + w * 0.62},{y - 2} L{x + w},{y + 48}" stroke="#ffffff" stroke-width="6" fill="none" stroke-linejoin="round" stroke-linecap="round"/>')
    eap(2140, 140, 780)
    eap(2290, 110, 620)

    # ---- Centennial Place and a dark older tower on the right.
    flat(2440, 120, 560, "#4f6b92", "glass", step=(60, 40), lit=0.07)
    flat(2600, 150, 480, "#c8b79a", "stone")
    flat(30, 110, 360, "#c9b08a", "stone")

    # ---- Front rank: older sandstone and brick blocks, a parkade, rooftop trees.
    for (x, w, h, col) in ((0, 170, 250, "#c7a27e"), (160, 210, 300, "#b98a6a"), (370, 140, 200, "#d9c6a2"),
                           (760, 160, 230, "#a87a5e"), (1010, 120, 280, "#d2bf9a"), (1500, 120, 210, "#c49a74"),
                           (1760, 130, 260, "#e0d0b0"), (2080, 120, 220, "#b48468"), (2420, 200, 280, "#cdb592"),
                           (2660, 140, 230, "#a88063")):
        flat(x, w, h, col, "stone", haze=1.4)
    # A rooftop Christmas tree and a lit star on two of them.
    for (tx_, ty_) in ((260, G - 300), (2510, G - 280)):
        c.add(f'<path d="M{tx_},{ty_ - 70} L{tx_ + 30},{ty_} L{tx_ - 30},{ty_} Z" fill="#2f8f5a" stroke="#1d5a38" stroke-width="3"/>')
        for k in range(5):
            c.add(f'<circle cx="{tx_ - 14 + k * 7}" cy="{ty_ - 12 - (k % 2) * 22}" r="3.5" fill="{cols[k]}"/>')
        c.add(f'<circle cx="{tx_}" cy="{ty_ - 74}" r="12" fill="#ffd23a" opacity="0.6" {c.blur(4)}/><circle cx="{tx_}" cy="{ty_ - 74}" r="6" fill="#ffe27a"/>')
    # Ground haze over everything at the bottom.
    g = c.lgrad([(0, c.haze, 0), (1, c.haze, 0.9)], 0, 0, 0, 1)
    c.add(f'<rect x="0" y="{G - 120}" width="{W}" height="120" fill="{g}"/>')
    body, defs = c.svg()
    return W, H, body, defs


# ====================================================================== FAR LAYER
def far():
    """The second rank, far behind: pale towers in the haze with a few lit
    windows and cranes, and the Bow's crescent and Brookfield's slab repeated
    as ghosts so the city feels deep."""
    W, H = 3000, 760
    c = Canvas(W, H, "#d6e4f5", seed=7, fog=0.3)
    G = H
    rnd = c.rnd
    x = 0
    pal = ["#9fb7d8", "#aec2de", "#b7c6dc", "#a3bbdc", "#c2cede"]
    while x < W - 60:
        w = rnd.randint(70, 150)
        h = rnd.randint(220, 640) if 700 < x < 2300 else rnd.randint(160, 380)
        col = rnd.choice(pal)
        y = G - h
        kind = rnd.random()
        if kind < 0.2:
            d = f"M{x},{G} L{x},{y} L{x + w},{y + 40} L{x + w},{G} Z"
            led = []
        elif kind < 0.35:
            d = f"M{x},{G} L{x},{y + 30} L{x + w / 2},{y} L{x + w},{y + 30} L{x + w},{G} Z"
            led = []
        else:
            d = f"M{x},{G} L{x},{y} L{x + w},{y} L{x + w},{G} Z"
            led = [(x, x + w, y)]
        c.tower(d, (x, y, x + w, G), col, kind="glass", floor=18, mull=13, lit=0.04, rows_lit=0.08, refl=0.6, haze=1.4, ledges=led, w=2.5)
        x += w + rnd.randint(-30, 20)
    c.crane(620, 560, 180, 80, op=0.75)
    c.crane(1900, 600, 200, 90, op=0.75)
    c.crane(2650, 420, 150, 70, op=0.75)
    g = c.lgrad([(0, c.haze, 0), (1, c.haze, 0.9)], 0, 0, 0, 1)
    c.add(f'<rect x="0" y="{G - 140}" width="{W}" height="140" fill="{g}"/>')
    body, defs = c.svg()
    return W, H, body, defs
