#!/usr/bin/env python3
"""Hand-painted key pieces in the style of the designers' key art (the
Falkbuilt rooftop): the Falkbuilt building with its giant wooden letters,
the sled, the sky (sun, clouds). Shapes are lit from the top left, outlined
in a deeper shade of their own colour, snow is white over ice blue.
Each file carries data-paint="done", so tools/paint.py only adds the grain.
Runs after gen_props.py and overwrites those files."""
import base64, math, os, random
from paint import edge, light, shadow, mix
from brush import Art, ellipse, circle, limb, person_eyes, cheeks

PACK = os.environ.get("PACK", "calgary_xmas")
HERE = os.path.dirname(__file__)
OUT = os.path.join(HERE, "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)

SNOW, SNOW_SH, SNOW_EDGE = "#ffffff", "#cfe0f2", "#8fb0d0"

def write(name, w, h, body, defs=""):
    open(os.path.join(OUT, name + ".svg"), "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" data-paint="done">\n'
        f'<defs>{defs}</defs>\n{body}\n</svg>')

def lg(gid, stops, x2=0, y2=1):
    s = "".join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
    return f'<linearGradient id="{gid}" x1="0" y1="0" x2="{x2}" y2="{y2}">{s}</linearGradient>'

def rg(gid, stops, cx=0.5, cy=0.5, r=0.5):
    s = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{a}"/>' for o, c, a in stops)
    return f'<radialGradient id="{gid}" cx="{cx}" cy="{cy}" r="{r}">{s}</radialGradient>'

def lit(gid, c):
    """Top-lit material gradient for one paint colour."""
    return lg(gid, [(0, light(c)), (0.45, c), (1, shadow(c))])

def P(d, fill, stroke=None, w=0, extra=""):
    s = f' stroke="{stroke}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"' if stroke else ""
    return f'<path d="{d}" fill="{fill}"{s} {extra}/>'

def R(x, y, w, h, fill, stroke=None, sw=0, rx=0, extra=""):
    s = f' stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round"' if stroke else ""
    return f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{rx}" fill="{fill}"{s} {extra}/>'

def snow_bank(x0, x1, y, h, rnd, step=34, lump=0.5):
    """A soft snow blanket whose base is the line y, lumpy on top, white
    falling to ice blue, with a blue-grey edge."""
    pts = []
    x = x0
    while x < x1:
        pts.append((x, y - h * (0.65 + lump * rnd.random())))
        x += step * (0.7 + 0.6 * rnd.random())
    pts.append((x1, y - h * 0.6))
    d = f"M{x0},{y + 2} L{x0},{pts[0][1]:.1f} "
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        mx = (ax + bx) / 2
        d += f"Q{mx:.1f},{min(ay, by) - h * 0.45:.1f} {bx:.1f},{by:.1f} "
    d += f"L{x1},{y + 2} Z"
    return d

# ---------------------------------------------------------------- Falkbuilt
def tower():
    """The Falkbuilt factory after the key art: red brick under a heavy
    slate-steel cornice buried in snow, tall arched windows, and FALKBUILT in
    giant orange wooden block letters on steel posts along the roof, each
    capped with snow. 1200x480 at 30 px per metre; the roof line (row 120)
    carries the ski jump drawn by the game, and the letters keep left of
    x 760 so the inrun clears them where it comes down to the roof."""
    W, H, ROOF = 1200, 480, 120
    rnd = random.Random(4)
    BRICK, BRICK_HI, BRICK_LO, MORTAR = "#c85a48", "#d9705b", "#a9473a", "#e9a08a"
    STEEL, STEEL_TOP, STEEL_LO = "#56697a", "#8aa0b0", "#3c4b59"
    GLASS_TOP, GLASS_BOT = "#bfe2fb", "#5f98cf"
    FRAME = "#dfe6ee"
    WOOD = "#f0943c"
    font = base64.b64encode(open(os.path.join(HERE, "..", "crates", "game", "assets", "LilitaOne.ttf"), "rb").read()).decode()
    defs = f'<style>@font-face{{font-family:"LilitaGen";src:url(data:font/ttf;base64,{font});}}</style>'
    defs += lg("wall", [(0, "#d36553"), (0.5, BRICK), (1, "#b04c3d")])
    defs += lg("glass", [(0, GLASS_TOP), (0.55, "#86b9e6"), (1, GLASS_BOT)])
    defs += lg("steel", [(0, STEEL_TOP), (0.3, STEEL), (1, STEEL_LO)])
    defs += lg("snow", [(0, SNOW), (0.55, SNOW), (1, SNOW_SH)])
    defs += lg("woodf", [(0, "#ffc27a"), (0.35, "#f8a24a"), (1, "#e07e2c")])
    defs += lg("pilaster", [(0, "#b8503f"), (0.25, "#d86b57"), (0.6, BRICK), (1, "#a9473a")], x2=1, y2=0)
    # Wood grain for the letters: long wavy strokes and a few knots.
    grain = ""
    for i in range(14):
        y = 6 + i * 7.5
        grain += f'<path d="M0,{y:.1f} C40,{y - 3:.1f} 80,{y + 3:.1f} 120,{y:.1f} S200,{y - 2:.1f} 240,{y:.1f}" fill="none" stroke="#c96a22" stroke-width="{1.2 + (i % 3) * 0.5}" opacity="0.45"/>'
    defs += f'<pattern id="grain" width="240" height="110" patternUnits="userSpaceOnUse">{grain}<ellipse cx="60" cy="40" rx="7" ry="3" fill="none" stroke="#b85c1c" stroke-width="1.5" opacity="0.5"/><ellipse cx="170" cy="78" rx="6" ry="2.5" fill="none" stroke="#b85c1c" stroke-width="1.5" opacity="0.5"/></pattern>'
    b = ""

    # ---- letters on the roof: steel posts and braces behind, then the
    # extruded wooden block letters, then their snow caps.
    TX0, TX1, BASE, SIZE = 22, 764, ROOF - 12, 126
    cap_top = BASE - SIZE * 0.70
    tattr = f'font-family="LilitaGen" font-size="{SIZE}" textLength="{TX1 - TX0}" lengthAdjust="spacingAndGlyphs"'
    for x in range(70, TX1, 86):
        b += R(x - 5, cap_top + 10, 10, ROOF - cap_top - 6, "url(#steel)", STEEL_LO, 2)
        b += P(f"M{x},{cap_top + 24} L{x + 46},{ROOF - 6}", "none", STEEL_LO, 9) + P(f"M{x},{cap_top + 24} L{x + 46},{ROOF - 6}", "none", STEEL, 5)
    # Extrusion: the letters stepped down-right in burnt orange.
    depth = 12
    side = "#b8601f"
    b += f'<text x="{TX0 + depth * 0.8}" y="{BASE + depth * 0.55}" {tattr} fill="{side}" stroke="#7d3c12" stroke-width="7" stroke-linejoin="round">FALKBUILT</text>'
    for k in range(depth, 0, -1):
        b += f'<text x="{TX0 + k * 0.8:.1f}" y="{BASE + k * 0.55:.1f}" {tattr} fill="{mix(side, "#9a4c17", k / depth)}">FALKBUILT</text>'
    # Face: lit gradient, grain, a darker rim and a bright top-left edge.
    b += f'<text x="{TX0}" y="{BASE}" {tattr} fill="url(#woodf)" stroke="#a5521a" stroke-width="4" stroke-linejoin="round" paint-order="stroke">FALKBUILT</text>'
    b += f'<text x="{TX0}" y="{BASE}" {tattr} fill="url(#grain)">FALKBUILT</text>'
    b += f'<text x="{TX0 - 1.5}" y="{BASE - 2}" {tattr} fill="none" stroke="#ffe2b0" stroke-width="1.6" opacity="0.7" clip-path="url(#tophalf)">FALKBUILT</text>'
    defs += f'<clipPath id="tophalf"><rect x="0" y="0" width="{W}" height="{cap_top + 26}"/></clipPath>'
    # Steel strap plates bolting each letter to its frame: two rows of
    # plates with bolt heads, clipped to the glyphs.
    defs += f'<clipPath id="glyphs"><text x="{TX0}" y="{BASE}" {tattr}>FALKBUILT</text></clipPath>'
    straps = ""
    for row_y in (BASE - SIZE * 0.2,):
        for x in range(70, TX1, 86):
            straps += R(x - 8, row_y - 4, 16, 8, "#9fb1c0", "#6d7f8f", 1.2, 2)
            straps += f'<circle cx="{x - 4}" cy="{row_y:.1f}" r="1.7" fill="#eef3f8"/><circle cx="{x + 4}" cy="{row_y:.1f}" r="1.7" fill="#eef3f8"/>'
    b += f'<g clip-path="url(#glyphs)">{straps}</g>'
    # Snow caps: the top band of the glyphs, swollen and lifted, so every
    # letter wears a pillow of snow exactly as wide as its top.
    defs += f'<clipPath id="capband"><rect x="0" y="{cap_top - 30}" width="{W}" height="36"/></clipPath>'
    defs += f'<clipPath id="capshade"><rect x="0" y="{cap_top - 2}" width="{W}" height="10"/></clipPath>'
    b += f'<g clip-path="url(#capband)" transform="translate(0 -5)"><text x="{TX0}" y="{BASE}" {tattr} fill="{SNOW_EDGE}" stroke="{SNOW_EDGE}" stroke-width="15" stroke-linejoin="round">FALKBUILT</text></g>'
    b += f'<g clip-path="url(#capband)" transform="translate(0 -5)"><text x="{TX0}" y="{BASE}" {tattr} fill="url(#snow)" stroke="#ffffff" stroke-width="11" stroke-linejoin="round">FALKBUILT</text></g>'
    b += f'<g clip-path="url(#capshade)" transform="translate(0 -5)"><text x="{TX0}" y="{BASE}" {tattr} fill="{SNOW_SH}" stroke="{SNOW_SH}" stroke-width="11" stroke-linejoin="round" opacity="0.9">FALKBUILT</text></g>'

    # ---- the wall ----
    WALL_TOP = ROOF + 26
    b += R(0, WALL_TOP, W, H - WALL_TOP, "url(#wall)")
    # Brick courses: staggered blocks with lighter mortar, each brick
    # nudged lighter or darker so the wall reads hand painted.
    bh, bw = 22, 54
    bricks = ""
    for r, y in enumerate(range(WALL_TOP, H, bh)):
        off = (bw // 2) if r % 2 else 0
        for x in range(-off, W, bw):
            c = rnd.choice((BRICK, BRICK, BRICK_HI, BRICK_LO, "#c2523f"))
            bricks += R(x + 2, y + 2, bw - 4, bh - 4, c, rx=3)
            bricks += R(x + 4, y + 2.5, bw - 8, 2.5, "#f0a08a", rx=1, extra='opacity="0.45"')
            bricks += R(x + 3, y + bh - 5, bw - 6, 2.5, "#7a2e26", rx=1, extra='opacity="0.35"')
            if rnd.random() < 0.18:
                bricks += R(x + 6, y + 4, bw * 0.4, 3, "#ffffff", rx=1.5, extra='opacity="0.18"')
    b += R(0, WALL_TOP, W, H - WALL_TOP, MORTAR) + bricks
    # Soft shade under the cornice and toward the ground.
    b += R(0, WALL_TOP, W, 40, "#4a1824", extra='opacity="0.22"')
    b += R(0, H - 60, W, 60, "#3a1630", extra='opacity="0.12"')

    # Pilasters between the bays, slightly proud of the wall.
    bays = [0, 150, 300, 450, 600, 750, 900, 1050, 1200]
    for x in bays:
        x0 = max(0, x - 18); x1 = min(W, x + 18)
        b += R(x0, WALL_TOP, x1 - x0, H - WALL_TOP, "url(#pilaster)", "#8e3a2f", 2)
        for y in range(WALL_TOP + 8, H, bh):
            b += P(f"M{x0 + 2},{y} L{x1 - 2},{y}", "none", MORTAR, 2, 'opacity="0.7"')

    # Arched windows: two storeys per bay; a few warm with Christmas inside.
    warm = {(1, 1), (3, 0), (4, 1), (6, 0)}
    def window(cx, top, w, h, lit_):
        r = w / 2
        out = f"M{cx - r},{top + h} L{cx - r},{top + r} A{r},{r} 0 0,1 {cx + r},{top + r} L{cx + r},{top + h} Z"
        ri = r - 9
        inn = f"M{cx - ri},{top + h - 7} L{cx - ri},{top + r + 2} A{ri},{ri} 0 0,1 {cx + ri},{top + r + 2} L{cx + ri},{top + h - 7} Z"
        s = ""
        # Brick arch voussoirs around the head.
        for k in range(9):
            a0 = math.pi * (1 - k / 9); a1 = math.pi * (1 - (k + 1) / 9)
            ro, rin = r + 12, r
            pts = [(cx + ro * math.cos(a0), top + r - ro * math.sin(a0)), (cx + ro * math.cos(a1), top + r - ro * math.sin(a1)),
                   (cx + rin * math.cos(a1), top + r - rin * math.sin(a1)), (cx + rin * math.cos(a0), top + r - rin * math.sin(a0))]
            s += P("M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts) + " Z", BRICK_HI if k % 2 else "#d0644f", "#93402f", 1.5)
        s += P(f"M{cx - 9},{top - 13} L{cx + 9},{top - 13} L{cx + 6},{top + 6} L{cx - 6},{top + 6} Z", "#e4dccf", "#9d9187", 2)
        s += P(f"M{cx - 9},{top - 13} L{cx + 9},{top - 13}", "none", "#ffffff", 2.5)
        s += P(out, FRAME, "#7c8a99", 3)
        s += P(inn, "#ffd88a" if lit_ else "url(#glass)", "#6f8296", 2)
        if lit_:
            s += P(inn, "#ff9a3c", extra='opacity="0.25"')
            s += f'<circle cx="{cx}" cy="{top + h * 0.55}" r="{w * 0.16}" fill="none" stroke="#2f8f5a" stroke-width="6"/>'
            s += f'<circle cx="{cx}" cy="{top + h * 0.55 - w * 0.16}" r="3.5" fill="#e23b3b"/>'
        else:
            # Glass glint: two soft diagonal bands.
            s += f'<g clip-path="url(#win{int(cx)}_{int(top)})"><path d="M{cx - ri},{top + h * 0.75} L{cx + ri},{top + h * 0.15} L{cx + ri},{top + h * 0.3} L{cx - ri},{top + h * 0.9} Z" fill="#ffffff" opacity="0.28"/></g>'
        # Mullions: centre bar, transom, and the arch spokes.
        s += P(f"M{cx},{top + 10} L{cx},{top + h - 7} M{cx - ri},{top + r + 2} L{cx + ri},{top + r + 2} M{cx - ri},{top + h * 0.68} L{cx + ri},{top + h * 0.68}", "none", "#6f8296", 6)
        s += P(f"M{cx},{top + 10} L{cx},{top + h - 7} M{cx - ri},{top + r + 2} L{cx + ri},{top + r + 2} M{cx - ri},{top + h * 0.68} L{cx + ri},{top + h * 0.68}", "none", FRAME, 3)
        # Stone sill with snow on it.
        s += R(cx - r - 8, top + h - 2, w + 16, 12, "#cfc6bb", "#8d8378", 2, 2)
        s += P(snow_bank(cx - r - 8, cx + r + 8, top + h - 1, 9, rnd, 14, 0.3), "url(#snow)", SNOW_EDGE, 1.5)
        return s, inn
    clips = ""
    for i in range(8):
        cx = 75 + i * 150
        for row, (top, h) in enumerate(((WALL_TOP + 26, 132), (WALL_TOP + 196, 116))):
            if row == 1 and i == 5:
                continue  # the door bay
            s, inn = window(cx, top, 82, h, (i, row) in warm)
            clips += f'<clipPath id="win{int(cx)}_{int(top)}"><path d="{inn}"/></clipPath>'
            b += s
    defs += clips
    # Front door with a wreath under a little steel canopy.
    dx, dtop = 825, WALL_TOP + 200
    b += P(f"M{dx - 44},{H - 22} L{dx - 44},{dtop + 44} A44,44 0 0,1 {dx + 44},{dtop + 44} L{dx + 44},{H - 22} Z", "#7a4a2c", "#4e2c18", 3)
    b += P(f"M{dx - 34},{H - 22} L{dx - 34},{dtop + 46} A34,34 0 0,1 {dx + 34},{dtop + 46} L{dx + 34},{H - 22} Z", "#8f5a36", "#5c341d", 2)
    b += P(f"M{dx},{dtop + 14} L{dx},{H - 22}", "none", "#5c341d", 3)
    b += f'<circle cx="{dx}" cy="{dtop + 70}" r="16" fill="none" stroke="#2c7d4e" stroke-width="9"/><circle cx="{dx}" cy="{dtop + 70}" r="16" fill="none" stroke="#49a86b" stroke-width="4" stroke-dasharray="5 6"/>'
    b += P(f"M{dx - 6},{dtop + 84} L{dx},{dtop + 76} L{dx + 6},{dtop + 84}", "#e23b3b", "#9e2626", 1.5)
    b += R(dx - 66, dtop - 6, 132, 16, "url(#steel)", STEEL_LO, 2, 3)
    b += P(snow_bank(dx - 66, dx + 66, dtop - 5, 12, rnd, 18), "url(#snow)", SNOW_EDGE, 1.5)

    # Copper downspouts at both ends with a snow cap on each hopper.
    for x in (14, W - 14):
        b += R(x - 6, ROOF + 26, 12, H - ROOF - 50, "url(#copper)", "#6c3b1d", 2, 3)
        b += R(x - 10, ROOF + 24, 20, 14, "url(#copper)", "#6c3b1d", 2, 2)
        for y in range(ROOF + 70, H - 30, 90):
            b += R(x - 9, y, 18, 6, "#3c4b59", rx=2)
    defs += lg("copper", [(0, "#d58a4f"), (0.5, "#b36a37"), (1, "#7f4422")], x2=1, y2=0)
    # Stone base course and a snow drift along the foot of the wall.
    b += R(0, H - 26, W, 26, "#9d9187", "#6d635b", 2)
    for x in range(0, W, 70):
        b += P(f"M{x},{H - 26} L{x},{H}", "none", "#7d736b", 2)
    b += P(snow_bank(0, W, H, 18, rnd, 46), "url(#snow)", SNOW_EDGE, 2)

    # ---- the cornice: a deep slate-steel gutter box with rivets and
    # brackets, a lit top lip, and the roof's snow blanket above it.
    b += R(-4, ROOF - 4, W + 8, 30, "url(#steel)", STEEL_LO, 3, 4)
    b += R(-4, ROOF - 4, W + 8, 6, STEEL_TOP, extra='opacity="0.8"')
    for x in range(30, W, 60):
        b += f'<circle cx="{x}" cy="{ROOF + 13}" r="3.6" fill="{STEEL_LO}"/><circle cx="{x - 1}" cy="{ROOF + 12}" r="1.6" fill="{STEEL_TOP}"/>'
    for x in bays[1:-1]:
        b += P(f"M{x - 14},{ROOF + 26} L{x + 14},{ROOF + 26} L{x + 10},{ROOF + 44} L{x - 10},{ROOF + 44} Z", "url(#steel)", STEEL_LO, 2)
    # Christmas lights sagging along the gutter.
    wire = f"M0,{ROOF + 30} " + " ".join(f"Q{x + 37.5},{ROOF + 44} {x + 75},{ROOF + 30}" for x in range(0, W, 75))
    b += P(wire, "none", "#2c3a2c", 2.5)
    for i, x in enumerate(range(12, W, 25)):
        k = ((x % 75) / 75.0)
        y = ROOF + 30 + 4 * 14 * k * (1 - k) * 0.95 + 5
        col = ["#ff5a4a", "#ffd23a", "#4fd08a", "#5ab0ff"][i % 4]
        b += f'<circle cx="{x}" cy="{y:.1f}" r="7" fill="{col}" opacity="0.3"/><ellipse cx="{x}" cy="{y:.1f}" rx="3.6" ry="4.6" fill="{col}" stroke="{edge(col)}" stroke-width="1.2"/>'
    # Rooftop plant under the take-off: a ribbed air handler and a vent
    # stack, each wearing snow.
    hx, hw_, hh_ = 840, 120, 44
    b += R(hx, ROOF - hh_, hw_, hh_ + 2, "url(#steel)", STEEL_LO, 2.5, 4)
    for k in range(1, 8):
        b += P(f"M{hx + k * hw_ / 8:.1f},{ROOF - hh_ + 8} L{hx + k * hw_ / 8:.1f},{ROOF - 4}", "none", STEEL_LO, 2, 'opacity="0.6"')
    b += f'<circle cx="{hx + 30}" cy="{ROOF - hh_ / 2:.0f}" r="13" fill="{STEEL_LO}"/><circle cx="{hx + 30}" cy="{ROOF - hh_ / 2:.0f}" r="9" fill="#2a353f"/>'
    b += P(snow_bank(hx - 4, hx + hw_ + 4, ROOF - hh_ + 2, 12, rnd, 22), "url(#snow)", SNOW_EDGE, 1.5)
    vx = 1040
    b += R(vx - 9, ROOF - 52, 18, 54, "url(#steel)", STEEL_LO, 2)
    b += R(vx - 15, ROOF - 60, 30, 10, "url(#steel)", STEEL_LO, 2, 3)
    b += P(snow_bank(vx - 17, vx + 17, ROOF - 59, 9, rnd, 10), "url(#snow)", SNOW_EDGE, 1.2)
    # Snow on the roof: a deep blanket over the cornice, drifting over the
    # front edge, and lumps between the letter posts.
    b += P(snow_bank(-4, W + 4, ROOF + 2, 16, rnd, 40), "url(#snow)", SNOW_EDGE, 2.5)
    b += P(f"M-4,{ROOF + 2} L{W + 4},{ROOF + 2} L{W + 4},{ROOF + 8} L-4,{ROOF + 8} Z", "url(#snow)")
    # Overhang lip and a few icicles.
    for x in range(40, W, 97):
        L = 10 + (x * 7) % 16
        b += P(f"M{x - 4},{ROOF + 8} L{x + 4},{ROOF + 8} L{x},{ROOF + 8 + L} Z", "#e8f4ff", SNOW_EDGE, 1.2)
    write("tower", W, H, b, defs)

tower()

# ---------------------------------------------------------------- sky
def cloud(name, w, h, seed):
    """A painted cumulus heap: a flat cool base, billows piled toward the
    middle along a domed envelope, each top lobe lit white on its upper
    left; the body falls from white to an ice-blue belly, rimmed in a
    deeper blue, edges softened."""
    r = random.Random(seed)
    base_y = h * 0.86
    top_y = h * 0.08
    lobes = []
    n = 9
    for i in range(n):
        t = (i + 0.5) / n
        env = math.sin(math.pi * t) ** 0.9 * r.uniform(0.75, 1.0)
        rr = h * (0.14 + 0.16 * env) * r.uniform(0.9, 1.1)
        x = w * (0.08 + 0.84 * t) + r.uniform(-w * 0.02, w * 0.02)
        y = max(top_y + rr, base_y - (base_y - top_y) * env * 0.92 - rr * 0.1 + rr)
        lobes.append((x, min(y, base_y - rr * 0.35), rr))
    # A second, smaller tier on top of the middle for a towering heap.
    for i in range(r.randint(2, 3)):
        x = w * r.uniform(0.32, 0.62)
        rr = h * r.uniform(0.17, 0.24)
        lobes.append((x, top_y + rr + r.uniform(0, h * 0.06), rr))
    left = min(x - rr for x, y, rr in lobes)
    right = max(x + rr for x, y, rr in lobes)
    body_rect = f'<rect x="{w * 0.1:.1f}" y="{base_y - h * 0.3:.1f}" width="{w * 0.8:.1f}" height="{h * 0.3:.1f}" rx="{h * 0.15:.1f}"/>'
    def shape(fill, dx=0, dy=0, k=1.0):
        return (f'<g fill="{fill}" transform="translate({dx} {dy})">' + "".join(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{rr * k:.1f}"/>' for x, y, rr in lobes)
                + body_rect + "</g>")
    clip = f'<clipPath id="above"><rect x="0" y="0" width="{w}" height="{base_y:.1f}"/></clipPath>'
    b = f'<g filter="url(#soft)"><g clip-path="url(#above)">'
    b += shape("#a3b6de", 0, 1.5, 1.035)
    b += shape("url(#cbody)")
    for x, y, rr in sorted(lobes, key=lambda l: l[1]):
        b += f'<circle cx="{x - rr * 0.14:.1f}" cy="{y - rr * 0.16:.1f}" r="{rr * 0.8:.1f}" fill="url(#clit)"/>'
    for x, y, rr in lobes:
        if y - rr < base_y - h * 0.35:
            b += f'<ellipse cx="{x - rr * 0.36:.1f}" cy="{y - rr * 0.46:.1f}" rx="{rr * 0.3:.1f}" ry="{rr * 0.16:.1f}" fill="#ffffff" transform="rotate(-25 {x - rr * 0.36:.1f} {y - rr * 0.46:.1f})"/>'
    # Belly shade over everything low.
    b += f'<rect x="0" y="{base_y - h * 0.32:.1f}" width="{w}" height="{h * 0.32:.1f}" fill="url(#belly)" clip-path="url(#cshape)"/>'
    b += "</g></g>"
    defs = (f'<linearGradient id="cbody" gradientUnits="userSpaceOnUse" x1="0" y1="{top_y:.1f}" x2="0" y2="{base_y:.1f}">'
            '<stop offset="0" stop-color="#ffffff"/><stop offset="0.55" stop-color="#f1f5fc"/><stop offset="1" stop-color="#c3d1ee"/></linearGradient>')
    defs += (f'<linearGradient id="belly" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#b7c8ea" stop-opacity="0"/>'
             '<stop offset="1" stop-color="#a9bce4" stop-opacity="0.75"/></linearGradient>') + clip
    defs += '<clipPath id="cshape">' + "".join(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{rr:.1f}"/>' for x, y, rr in lobes) + body_rect + '</clipPath>'
    defs += rg("clit", [(0, "#ffffff", 1), (0.6, "#ffffff", 0.85), (1, "#ffffff", 0)], cx=0.42, cy=0.36, r=0.62)
    defs += '<filter id="soft" x="-5%" y="-10%" width="110%" height="120%"><feGaussianBlur stdDeviation="1.4"/></filter>'
    write(name, w, h, b, defs)

for nm, w, h, sd in (("cloud_a", 320, 160, 1), ("cloud_b", 260, 140, 2), ("cloud_c", 400, 180, 3)):
    cloud(nm, w, h, sd)

def sun():
    """A painted winter sun: a white-hot core in a pale gold disc that melts
    into its own corona (the game adds the wide halo and rays)."""
    defs = rg("corona", [(0, "#fffbe8", 1), (0.36, "#fff3c4", 0.85), (0.5, "#ffe7a0", 0.38), (0.75, "#ffd98a", 0.12), (1, "#ffd98a", 0)])
    defs += rg("disc", [(0, "#ffffff", 1), (0.55, "#fffbea", 1), (0.85, "#fff0b8", 1), (1, "#ffe08c", 1)], cx=0.45, cy=0.42, r=0.6)
    defs += '<filter id="sb" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="2.2"/></filter>'
    b = '<circle cx="128" cy="128" r="127" fill="url(#corona)"/>'
    b += '<circle cx="128" cy="128" r="50" fill="#fff2b8" opacity="0.8" filter="url(#sb)"/>'
    b += '<circle cx="128" cy="128" r="46" fill="url(#disc)" filter="url(#sb)"/>'
    write("sun", 256, 256, b, defs)
sun()

# ---------------------------------------------------------------- terrain
# The Rockies, foothills, snow hills and the ground crust live in
# terrain.py (treeline and meadow too); written here so gen_all.sh keeps
# its order.
import terrain
terrain.build_all()

def sled():
    """The key art's red sled, painted with the character brush (lacquered
    deck, gold pinstripe, curled steel runners, brass bell); drawn in
    tools/char_people.py."""
    import char_people
    open(os.path.join(OUT, "sled.svg"), "w").write(char_people.sled().svg(192, 96).replace('data-paint="done"', 'data-paint="skip"'))
sled()

def reindeer():
    """The reindeer pickup, leaping left; drawn with the pilots in
    tools/char_people.py."""
    import char_people
    open(os.path.join(OUT, "reindeer.svg"), "w").write(char_people.reindeer().svg(224, 176).replace('data-paint="done"', 'data-paint="skip"'))
reindeer()

print("painted pieces written")
