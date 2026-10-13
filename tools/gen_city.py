#!/usr/bin/env python3
"""Calgary street scenery (repainted by tools/paint.py in the key-art style): building facades for
each district the flight passes over (Inglewood, Stephen Avenue downtown,
the Bow and the Peace Bridge, Kensington, 17th Avenue and the Saddledome,
the suburbs, the prairie), the downtown skyline for the far layer, street
props, vehicles and Calgarians.

Drawn from street photos (Wikimedia Commons) and exaggerated for fun. Every
building is drawn at M px per metre; the real size in metres goes into
packs/<pack>/city.json together with the district layout, so the game
needs no size table of its own.

Writes SVGs into packs/<pack>/svg (render with tools/render_sprites.js).
"""
import base64, json, math, os, random
from brush import Art, SKIN, ellipse, circle, limb, person_eyes, cheeks

PACK = os.environ.get("PACK", "calgary_xmas")
ROOT = os.path.join(os.path.dirname(__file__), "..")
OUT = os.path.join(ROOT, "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)
INK = "#3a2618"
LW = 6
M = 30  # px per metre for buildings
SNOW, SNOW_SH = "#ffffff", "#dfe8f5"
GLASS, GLASS_HI, WARM = "#8ec9f2", "#cfeaff", "#ffd98a"
FONT = base64.b64encode(open(os.path.join(ROOT, "crates", "game", "assets", "LilitaOne.ttf"), "rb").read()).decode()

# name -> (width m, height m) of what the sprite covers; feeds city.json.
SIZES = {}


# Shared painted defs every city sprite may use: soft glows, the warm lit
# interior, the daylit glass with sky in it, and the facade light overlay.
COMMON_DEFS = (
    '<filter id="fglow" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="7"/></filter>'
    '<filter id="fsoft" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="2.5"/></filter>'
    '<linearGradient id="gLit" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff4c8"/>'
    '<stop offset="0.55" stop-color="#ffd27a"/><stop offset="1" stop-color="#f2a04a"/></linearGradient>'
    '<linearGradient id="gGlass" x1="0" y1="0" x2="0.4" y2="1"><stop offset="0" stop-color="#eaf6ff"/>'
    '<stop offset="0.45" stop-color="#a8d4f4"/><stop offset="1" stop-color="#6e9fd2"/></linearGradient>'
    '<linearGradient id="gWall" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff3d6" stop-opacity="0.30"/>'
    '<stop offset="0.22" stop-color="#ffffff" stop-opacity="0"/><stop offset="0.72" stop-color="#1c2b5e" stop-opacity="0"/>'
    '<stop offset="1" stop-color="#1c2b5e" stop-opacity="0.18"/></linearGradient>'
    '<linearGradient id="gSide" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#ffffff" stop-opacity="0.10"/>'
    '<stop offset="0.25" stop-color="#ffffff" stop-opacity="0"/><stop offset="0.8" stop-color="#1c2b5e" stop-opacity="0"/>'
    '<stop offset="1" stop-color="#1c2b5e" stop-opacity="0.16"/></linearGradient>')
# Facades (bld_*, house_*) get the key art's top light and grounded shade.
OVERLAY = ("bld_", "house_")
# name -> (fx, fy): where a chimney's smoke leaves, as fractions of the sprite.
SMOKE = {}


def write(name, w, h, body, defs="", metres=None, scale=1.5, painted=False):
    """`metres` is the sprite's world width; the height follows its aspect.
    `painted` marks art drawn with tools/brush.py, which paint.py leaves be."""
    defs = COMMON_DEFS + defs
    if name.startswith(OVERLAY):
        defs += '<mask id="fmask" mask-type="alpha"><use href="#fbody"/></mask>'
        body = (f'<g id="fbody">{body}</g>'
                f'<rect x="0" y="0" width="{int(w)}" height="{int(h)}" fill="url(#gWall)" opacity="0.97" mask="url(#fmask)"/>'
                f'<rect x="0" y="0" width="{int(w)}" height="{int(h)}" fill="url(#gSide)" opacity="0.97" mask="url(#fmask)"/>')
    style = ""
    if "LilitaGen" in body:
        style = f'<style>@font-face{{font-family:"LilitaGen";src:url(data:font/ttf;base64,{FONT});}}</style>'
    open(os.path.join(OUT, name + ".svg"), "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{int(w)}" height="{int(h)}" viewBox="0 0 {int(w)} {int(h)}" data-scale="{scale}"{' data-paint="done"' if painted else ''}>\n'
        f'<defs>{style}{defs}</defs>\n{body}\n</svg>')
    wm = metres if metres is not None else w / M
    SIZES[name] = (round(wm, 2), round(wm * h / w, 2))


# ------------------------------------------------------------------ helpers
def ol(w=LW, c=INK):
    return f'stroke="{c}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'
def line(d, w=LW, c=INK, op=1):
    return f'<path d="{d}" fill="none" {ol(w, c)} opacity="{op}"/>'
def shape(d, fill, w=LW, c=INK):
    if w == 0:
        return f'<path d="{d}" fill="{fill}"/>'
    return f'<path d="{d}" fill="{fill}" {ol(w, c)}/>'
def rect(x, y, w, h, fill, rx=0, sw=LW, c=INK, op=1):
    s = f' {ol(sw, c)}' if sw else ''
    return f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{rx}" fill="{fill}" opacity="{op}"{s}/>'
def circ(cx, cy, r, fill, w=LW, c=INK):
    s = f' {ol(w, c)}' if w else ''
    return f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{r:.1f}" fill="{fill}"{s}/>'
def ell(cx, cy, rx, ry, fill, w=LW, rot=0, c=INK):
    s = f' {ol(w, c)}' if w else ''
    return f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="{rx:.1f}" ry="{ry:.1f}" fill="{fill}" transform="rotate({rot} {cx:.1f} {cy:.1f})"{s}/>'
def poly(pts, fill, w=LW, c=INK):
    d = "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts) + " Z"
    return shape(d, fill, w, c)
def shade(d, a=0.18):
    return f'<path d="{d}" fill="#000000" opacity="{a}"/>'
def gloss(x, y, w, h, a=0.35):
    """A diagonal light streak across a pane."""
    return f'<path d="M{x + w * 0.15:.1f},{y + h:.1f} L{x + w * 0.55:.1f},{y:.1f} L{x + w * 0.8:.1f},{y:.1f} L{x + w * 0.4:.1f},{y + h:.1f} Z" fill="#ffffff" opacity="{a}"/>'
def text(x, y, s, size, fill, anchor="middle", stroke=None, sw=0, length=None, rot=0, op=1):
    st = f' stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round" paint-order="stroke"' if stroke else ''
    ln = f' textLength="{length}" lengthAdjust="spacingAndGlyphs"' if length else ''
    tr = f' transform="rotate({rot} {x} {y})"' if rot else ''
    return f'<text x="{x:.1f}" y="{y:.1f}" font-family="LilitaGen" font-size="{size}" text-anchor="{anchor}" fill="{fill}" opacity="{op}"{st}{ln}{tr}>{s}</text>'

def darker(hexcol, k=0.82):
    r, g, b = int(hexcol[1:3], 16), int(hexcol[3:5], 16), int(hexcol[5:7], 16)
    return "#%02x%02x%02x" % (int(r * k), int(g * k), int(b * k))
def lighter(hexcol, k=0.3):
    r, g, b = int(hexcol[1:3], 16), int(hexcol[3:5], 16), int(hexcol[5:7], 16)
    return "#%02x%02x%02x" % (int(r + (255 - r) * k), int(g + (255 - g) * k), int(b + (255 - b) * k))

def _bricks(pid, base, mortar, bw, bh, per_row, k_var, r):
    """Chunky painted masonry after the key art: light mortar, each block a
    rounded slab with a lit top edge, a shaded bottom edge and its own
    slightly different tone. The first (empty) rect carries the base colour
    for tools/paint.py's outline maths."""
    rnd = random.Random(pid)
    W, H = bw * per_row, bh * 2
    out = f'<pattern id="{pid}" width="{W}" height="{H}" patternUnits="userSpaceOnUse"><rect width="0" height="0" fill="{base}"/>'
    out += f'<rect width="{W}" height="{H}" fill="{mortar}"/>'
    g = max(1.0, bh * 0.1)
    for row in range(2):
        off = 0 if row == 0 else bw / 2
        for i in range(-1, per_row + 1):
            x = i * bw + off
            y = row * bh
            c = lighter(base, rnd.uniform(0, k_var)) if rnd.random() < 0.5 else darker(base, 1 - rnd.uniform(0, k_var))
            out += f'<rect x="{x + g:.1f}" y="{y + g:.1f}" width="{bw - 2 * g:.1f}" height="{bh - 2 * g:.1f}" rx="{r}" fill="{c}"/>'
            out += f'<rect x="{x + g + 1:.1f}" y="{y + g:.1f}" width="{bw - 2 * g - 2:.1f}" height="{max(1.2, bh * 0.16):.1f}" rx="{r / 2}" fill="#ffffff" opacity="0.28"/>'
            out += f'<rect x="{x + g:.1f}" y="{y + bh - g - max(1.2, bh * 0.2):.1f}" width="{bw - 2 * g:.1f}" height="{max(1.2, bh * 0.2):.1f}" rx="{r / 2}" fill="#3a1a2a" opacity="0.18"/>'
    return out + "</pattern>"

def brick_pattern(pid, base, mortar=None, bw=24, bh=12):
    mortar = mortar or lighter(base, 0.45)
    return _bricks(pid, base, mortar, bw, bh, 4, 0.1, 2)
def block_pattern(pid, base, bw=60, bh=30):
    """Sandstone ashlar: big soft blocks with pale joints."""
    return _bricks(pid, base, lighter(base, 0.35), bw, bh, 2, 0.06, 4)
def siding_pattern(pid, base, step=12):
    return (f'<pattern id="{pid}" width="40" height="{step}" patternUnits="userSpaceOnUse">'
            f'<rect width="40" height="{step}" fill="{base}"/><path d="M0,{step - 1} H40" stroke="{darker(base, 0.8)}" stroke-width="2"/></pattern>')

def snowcap(x0, x1, y, h=12, sw=4):
    """A scalloped snow cap sitting on a ledge whose top is at y."""
    n = max(1, round((x1 - x0) / 28))
    step = (x1 - x0) / n
    d = f"M{x0 - 2:.1f},{y + 4:.1f} L{x0 - 2:.1f},{y:.1f} "
    for i in range(n):
        xa = x0 + i * step
        d += f"Q{xa + step / 2:.1f},{y - h * (1.1 if i % 2 else 0.8):.1f} {xa + step:.1f},{y:.1f} "
    d += f"L{x1 + 2:.1f},{y:.1f} L{x1 + 2:.1f},{y + 4:.1f} Z"
    return shape(d, SNOW, sw) + icicles(x0, x1, y + 4, h)

def icicles(x0, x1, y, h=12):
    """Icicles under a ledge, a few long, most short; seeded by position."""
    if x1 - x0 < 50:
        return ""
    rnd = random.Random(int(x0 * 7 + y * 13 + x1))
    b = ""
    x = x0 + rnd.uniform(4, 14)
    while x < x1 - 4:
        if rnd.random() < 0.6:
            L = rnd.uniform(0.5, 1.6) * max(6, h)
            w = rnd.uniform(2.5, 4.5)
            b += f'<path d="M{x - w:.1f},{y - 1:.1f} L{x + w:.1f},{y - 1:.1f} L{x + 0.5:.1f},{y + L:.1f} Z" fill="#d9efff" opacity="0.95"/>'
            b += f'<path d="M{x - w * 0.4:.1f},{y:.1f} L{x:.1f},{y:.1f} L{x + 0.2:.1f},{y + L * 0.7:.1f} Z" fill="#ffffff" opacity="0.9"/>'
        x += rnd.uniform(8, 20)
    return b

def lights(x0, x1, y, sag=12, step=22, r=4.5):
    d = f"M{x0},{y} "
    seg = 66
    xs = list(range(int(x0), int(x1), seg))
    for x in xs:
        d += f"Q{x + seg / 2},{y + sag} {min(x + seg, x1)},{y} "
    b = line(d, 2.5, "#2b4a2b")
    cols = ["#ff4a4a", "#ffd23a", "#3fbf7a", "#4aa3ff", "#ff8ad8"]
    for i, x in enumerate(range(int(x0) + step // 2, int(x1), step)):
        k = ((x - x0) % seg) / seg
        yy = y + sag * 4 * k * (1 - k) * 0.5 + 5
        b += circ(x, yy, r, cols[i % len(cols)], 2.5)
    return b

def wreath(cx, cy, r=12):
    b = f'<circle cx="{cx}" cy="{cy}" r="{r * 0.68:.1f}" fill="none" stroke="{INK}" stroke-width="{r * 0.64 + 6:.1f}"/>'
    b += f'<circle cx="{cx}" cy="{cy}" r="{r * 0.68:.1f}" fill="none" stroke="#2e9e5b" stroke-width="{r * 0.64:.1f}"/>'
    for a in range(0, 360, 72):
        b += circ(cx + math.cos(math.radians(a)) * r * 0.7, cy + math.sin(math.radians(a)) * r * 0.7, 2.6, "#e6382f", 0)
    b += shape(f"M{cx - 6},{cy + r - 2} L{cx},{cy + r - 6} L{cx + 6},{cy + r - 2} L{cx + 4},{cy + r + 8} L{cx},{cy + r + 2} L{cx - 4},{cy + r + 8} Z", "#e6382f", 2.5)
    return b

def interior(ix, iy, iw, ih, seed):
    """What you see through a lit window: curtains tied back, and a little
    Christmas tree, a lamp, a cat on the sill or somebody waving."""
    rnd = random.Random(seed)
    b = f'<rect x="{ix:.1f}" y="{iy + ih * 0.62:.1f}" width="{iw:.1f}" height="{ih * 0.38:.1f}" fill="#d8843a" opacity="0.35"/>'
    what = rnd.random()
    cx, base = ix + iw * rnd.uniform(0.35, 0.65), iy + ih
    if what < 0.34 and iw > 22:
        r = min(iw * 0.32, ih * 0.22)
        b += f'<path d="M{cx:.1f},{base - ih * 0.7:.1f} L{cx + r:.1f},{base - 2:.1f} L{cx - r:.1f},{base - 2:.1f} Z" fill="#2f7a4a" opacity="0.92"/>'
        for k in range(4):
            b += f'<circle cx="{cx + rnd.uniform(-r, r) * 0.6:.1f}" cy="{base - ih * rnd.uniform(0.1, 0.5):.1f}" r="2.2" fill="{["#ff4a4a", "#ffd23a", "#4aa3ff", "#ff8ad8"][k]}"/>'
        b += f'<circle cx="{cx:.1f}" cy="{base - ih * 0.7:.1f}" r="3" fill="#ffe27a"/>'
    elif what < 0.6:
        b += f'<path d="M{cx - 7:.1f},{iy + ih * 0.35:.1f} L{cx + 7:.1f},{iy + ih * 0.35:.1f} L{cx + 4:.1f},{iy + ih * 0.22:.1f} L{cx - 4:.1f},{iy + ih * 0.22:.1f} Z" fill="#e6382f" opacity="0.85"/>'
        b += f'<path d="M{cx:.1f},{iy + ih * 0.35:.1f} L{cx:.1f},{base - 4:.1f}" stroke="#7a4a2a" stroke-width="2" opacity="0.7"/>'
        b += f'<circle cx="{cx:.1f}" cy="{iy + ih * 0.33:.1f}" r="9" fill="#fff6c8" opacity="0.8" filter="url(#fsoft)"/>'
    elif what < 0.78:
        # A cat on the sill, tail hanging.
        b += f'<path d="M{cx - 8:.1f},{base:.1f} C{cx - 9:.1f},{base - 12:.1f} {cx + 9:.1f},{base - 12:.1f} {cx + 8:.1f},{base:.1f} Z" fill="#4a3a4a" opacity="0.85"/>'
        b += f'<circle cx="{cx:.1f}" cy="{base - 13:.1f}" r="5" fill="#4a3a4a" opacity="0.85"/>'
        b += f'<path d="M{cx - 4.5:.1f},{base - 15:.1f} L{cx - 3:.1f},{base - 21:.1f} L{cx - 0.5:.1f},{base - 16:.1f} M{cx + 4.5:.1f},{base - 15:.1f} L{cx + 3:.1f},{base - 21:.1f} L{cx + 0.5:.1f},{base - 16:.1f}" fill="#4a3a4a" opacity="0.85"/>'
    elif iw > 20:
        # Somebody at the window, waving.
        b += f'<path d="M{cx - 9:.1f},{base:.1f} C{cx - 10:.1f},{base - ih * 0.3:.1f} {cx + 10:.1f},{base - ih * 0.3:.1f} {cx + 9:.1f},{base:.1f} Z" fill="#9a5a4a" opacity="0.6"/>'
        b += f'<circle cx="{cx:.1f}" cy="{base - ih * 0.36:.1f}" r="{min(8, iw * 0.2):.1f}" fill="#9a5a4a" opacity="0.6"/>'
        b += f'<path d="M{cx + 7:.1f},{base - ih * 0.2:.1f} L{cx + 13:.1f},{base - ih * 0.5:.1f}" stroke="#9a5a4a" stroke-width="4" stroke-linecap="round" opacity="0.6"/>'
    # Curtains tied back at both sides.
    cc = rnd.choice(["#c8433a", "#2f7a5a", "#f4e6c8", "#8a3a6a"])
    cw = iw * 0.24
    b += f'<path d="M{ix:.1f},{iy:.1f} L{ix + cw:.1f},{iy:.1f} Q{ix + cw * 0.4:.1f},{iy + ih * 0.45:.1f} {ix + cw * 0.5:.1f},{iy + ih * 0.6:.1f} L{ix:.1f},{iy + ih * 0.66:.1f} Z" fill="{cc}" opacity="0.9"/>'
    b += f'<path d="M{ix + iw:.1f},{iy:.1f} L{ix + iw - cw:.1f},{iy:.1f} Q{ix + iw - cw * 0.4:.1f},{iy + ih * 0.45:.1f} {ix + iw - cw * 0.5:.1f},{iy + ih * 0.6:.1f} L{ix + iw:.1f},{iy + ih * 0.66:.1f} Z" fill="{cc}" opacity="0.9"/>'
    return b

def sky_in_glass(ix, iy, iw, ih, seed):
    """Daylit glass: the sky's gradient, a reflected cloud and a light streak."""
    rnd = random.Random(seed)
    b = f'<ellipse cx="{ix + iw * rnd.uniform(0.3, 0.7):.1f}" cy="{iy + ih * rnd.uniform(0.25, 0.45):.1f}" rx="{iw * 0.42:.1f}" ry="{max(3, ih * 0.08):.1f}" fill="#ffffff" opacity="0.6" filter="url(#fsoft)"/>'
    return b + gloss(ix, iy, iw, ih, 0.32)

def window(x, y, w, h, frame="#f4efe3", lit=False, arch=False, cross=True, sill=True, snow=True, glass=GLASS, fw=4):
    """A sash window: frame, glass (day sky or a warm lit room), glazing bars, sill."""
    b = ""
    seed = int(x * 31 + y * 17)
    if lit:
        b += f'<rect x="{x - 6:.1f}" y="{y - 6:.1f}" width="{w + 12:.1f}" height="{h + 12:.1f}" fill="#ffc86a" opacity="0.55" filter="url(#fglow)"/>'
    pane = "url(#gLit)" if lit else ("url(#gGlass)" if glass == GLASS else glass)
    if arch:
        d = f"M{x},{y + h} L{x},{y + w / 2} A{w / 2},{w / 2} 0 0 1 {x + w},{y + w / 2} L{x + w},{y + h} Z"
        b += shape(d, frame, 4)
        ix, iy, iw, ih = x + fw, y + fw, w - 2 * fw, h - 2 * fw
        di = f"M{ix},{iy + ih} L{ix},{iy + iw / 2} A{iw / 2},{iw / 2} 0 0 1 {ix + iw},{iy + iw / 2} L{ix + iw},{iy + ih} Z"
        b += shape(di, pane, 0)
        if lit:
            b += interior(ix, iy + iw * 0.4, iw, ih - iw * 0.4, seed)
        else:
            b += sky_in_glass(ix, iy + iw * 0.3, iw, ih - iw * 0.3, seed)
        if cross:
            b += line(f"M{x + w / 2},{iy + 2} L{x + w / 2},{iy + ih} M{ix},{iy + ih * 0.55} L{ix + iw},{iy + ih * 0.55}", 3, frame)
        b += line(d, 4)
    else:
        b += rect(x, y, w, h, frame, 2, 4)
        ix, iy, iw, ih = x + fw, y + fw, w - 2 * fw, h - 2 * fw
        b += rect(ix, iy, iw, ih, pane, 0, 0)
        if lit:
            b += interior(ix, iy, iw, ih, seed)
        else:
            b += sky_in_glass(ix, iy, iw, ih, seed)
        if cross:
            b += line(f"M{x + w / 2},{iy} L{x + w / 2},{iy + ih} M{ix},{iy + ih * 0.5} L{ix + iw},{iy + ih * 0.5}", 3, frame)
        b += rect(x, y, w, h, "none", 2, 4)
    if sill:
        b += rect(x - 4, y + h - 1, w + 8, 7, darker(frame, 0.9), 2, 3)
        if snow:
            b += snowcap(x - 4, x + w + 4, y + h - 1, 6, 3)
    return b

def garland(x0, x1, y, sag=10):
    """A fir swag with red bows and warm bulbs, hung along a shop sign."""
    n = max(1, round((x1 - x0) / 60))
    step = (x1 - x0) / n
    d = f"M{x0:.1f},{y:.1f} " + " ".join(f"Q{x0 + (i + 0.5) * step:.1f},{y + sag * 2:.1f} {x0 + (i + 1) * step:.1f},{y:.1f}" for i in range(n))
    b = f'<path d="{d}" fill="none" stroke="#1d5a38" stroke-width="11" stroke-linecap="round"/>'
    b += f'<path d="{d}" fill="none" stroke="#2f8f5a" stroke-width="7" stroke-linecap="round" stroke-dasharray="3 4"/>'
    for i in range(n + 1):
        bx = x0 + i * step
        b += shape(f"M{bx - 8:.1f},{y - 6:.1f} L{bx:.1f},{y:.1f} L{bx - 8:.1f},{y + 6:.1f} Z M{bx + 8:.1f},{y - 6:.1f} L{bx:.1f},{y:.1f} L{bx + 8:.1f},{y + 6:.1f} Z", "#e6382f", 2.5)
        b += circ(bx, y, 3.5, "#c72d26", 2)
    for i in range(n):
        for k in (0.3, 0.7):
            t = k
            bx = x0 + (i + t) * step
            by = y + sag * 2 * 2 * t * (1 - t)
            b += f'<circle cx="{bx:.1f}" cy="{by + 4:.1f}" r="7" fill="#ffd77a" opacity="0.6" filter="url(#fsoft)"/>' + circ(bx, by + 4, 3, "#fff2b0", 0)
    return b

def chimney(name, W, H, x, y, w=30, h=50, col="#a5523a"):
    """A brick chimney standing on a roof line at y; its smoke is animated
    in the game (city.json carries the anchor)."""
    SMOKE[name] = (round((x + w / 2) / W, 3), round((y - h) / H, 3))
    b = rect(x, y - h, w, h + 4, col, 2, 4)
    for yy in range(int(y - h) + 10, int(y), 10):
        b += line(f"M{x + 2},{yy} L{x + w - 2},{yy}", 1.5, darker(col, 0.75), 0.7)
    b += rect(x - 4, y - h - 8, w + 8, 10, darker(col, 0.85), 2, 4)
    b += snowcap(x - 4, x + w + 4, y - h - 8, 7, 3)
    return b

def awning(x, y, w, h, c1, c2="#ffffff", n=None):
    n = n or max(3, int(w / 26))
    sw = w / n
    b = ""
    for i in range(n):
        b += rect(x + i * sw, y, sw, h, c1 if i % 2 == 0 else c2, 0, 0)
    sc = f"M{x},{y + h} " + " ".join(f"Q{x + (i + 0.5) * sw:.1f},{y + h + 12} {x + (i + 1) * sw:.1f},{y + h}" for i in range(n))
    b += shape(sc + f" L{x + w},{y + h} Z", c1, 0)
    for i in range(n):
        if i % 2 == 1:
            b += shape(f"M{x + i * sw:.1f},{y + h} Q{x + (i + 0.5) * sw:.1f},{y + h + 12} {x + (i + 1) * sw:.1f},{y + h} Z", c2, 0)
    b += line(f"M{x},{y} L{x + w},{y} L{x + w},{y + h} " + " ".join(f"Q{x + w - (i + 0.5) * sw:.1f},{y + h + 12} {x + w - (i + 1) * sw:.1f},{y + h}" for i in range(n)) + f" L{x},{y} Z", 4)
    b += snowcap(x, x + w, y, 10, 3)
    return b

def shopfront(x, y, w, h, frame="#3b3f4a", goods=None, door_at=None, lit=True):
    """A storefront: glazed bays on a kick plate, warm interior, a door."""
    b = ""
    if lit:
        b += f'<rect x="{x - 10}" y="{y - 4}" width="{w + 20}" height="{h + 10}" fill="#ffc060" opacity="0.5" filter="url(#fglow)"/>'
    b += rect(x, y, w, h, frame, 0, 4)
    kick = 26
    gx, gy, gw, gh = x + 8, y + 8, w - 16, h - 16 - kick
    b += rect(gx, gy, gw, gh, "url(#gLit)" if lit else "url(#gGlass)", 0, 0)
    if lit:
        # The shop inside: a back wall with shelves, pendant lamps glowing.
        b += rect(gx, gy + gh * 0.18, gw, 4, "#a8642e", 0, 0, op=0.35) + rect(gx, gy + gh * 0.42, gw, 4, "#a8642e", 0, 0, op=0.35)
        for k in range(max(1, int(gw / 60))):
            lx = gx + (k + 0.5) * gw / max(1, int(gw / 60))
            b += f'<path d="M{lx:.1f},{gy} L{lx:.1f},{gy + gh * 0.16:.1f}" stroke="#5a3a2a" stroke-width="1.5" opacity="0.8"/>'
            b += f'<circle cx="{lx:.1f}" cy="{gy + gh * 0.2:.1f}" r="13" fill="#fff6c8" opacity="0.85" filter="url(#fsoft)"/>'
            b += f'<path d="M{lx - 8:.1f},{gy + gh * 0.2:.1f} Q{lx:.1f},{gy + gh * 0.2 - 12:.1f} {lx + 8:.1f},{gy + gh * 0.2:.1f} Z" fill="#2f6b4a" opacity="0.9"/>'
    if goods:
        b += goods(gx, gy, gw, gh)
    b += gloss(gx, gy, gw * 0.5, gh, 0.25) + gloss(gx + gw * 0.5, gy, gw * 0.5, gh, 0.2)
    n = max(1, round(gw / 70))
    for i in range(1, n):
        xx = gx + gw * i / n
        b += line(f"M{xx:.1f},{gy} L{xx:.1f},{gy + gh}", 4, frame)
    b += rect(gx, gy, gw, gh, "none", 0, 3)
    if door_at is not None:
        dx = x + door_at
        b += rect(dx, y + 10, 44, h - 10, frame, 0, 4) + rect(dx + 6, y + 18, 32, h - 34, WARM if lit else GLASS, 0, 3)
        b += circ(dx + 34, y + h * 0.6, 3, "#ffd23a", 0)
        b += wreath(dx + 22, y + 34, 10)
    if lit and w > 80:
        b += garland(x + 4, x + w - 4, y + 9, 6)
    return b

def bare_tree(tx, ground, th, col="#4a3a30", lights_on=False):
    b = line(f"M{tx},{ground} L{tx},{ground - th}", 8, col)
    pts = []
    for k in range(5):
        y = ground - th * (0.42 + 0.12 * k)
        sp = th * (0.2 - 0.03 * k)
        b += line(f"M{tx},{y:.1f} L{tx - sp:.1f},{y - th * 0.12:.1f} M{tx},{y:.1f} L{tx + sp:.1f},{y - th * 0.1:.1f}", 4, col)
        b += line(f"M{tx - sp:.1f},{y - th * 0.12:.1f} L{tx - sp - 8:.1f},{y - th * 0.2:.1f} M{tx + sp:.1f},{y - th * 0.1:.1f} L{tx + sp + 7:.1f},{y - th * 0.18:.1f}", 3, col)
        pts += [(tx - sp, y - th * 0.12), (tx + sp, y - th * 0.1), (tx - sp * 0.5, y - th * 0.06), (tx + sp * 0.5, y - th * 0.05)]
    if lights_on:
        for (x, y) in pts:
            b += circ(x, y, 3.2, "#fff2a8", 1.5, "#c99a1e")
    return b

def magpie_shape(x, y, s=1.0, flip=False):
    """A tiny magpie perched at (x, y) = feet; used on roofs inside facades."""
    f = -1 if flip else 1
    p = lambda dx, dy: f"{x + f * dx * s:.1f},{y + dy * s:.1f}"
    b = shape(f"M{p(-4, -6)} L{p(-30, -2)} L{p(-28, -8)} Z", "#1f3b5a", 2)  # tail
    b += shape(f"M{p(-6, -4)} C{p(-8, -16)} {p(4, -20)} {p(10, -14)} C{p(14, -10)} {p(8, -2)} {p(-6, -4)} Z", "#1b1b22", 2)
    b += shape(f"M{p(-4, -8)} C{p(0, -14)} {p(6, -12)} {p(6, -8)} Z", "#ffffff", 0)
    b += circ(x + f * 10 * s, y - 18 * s, 5 * s, "#1b1b22", 2) + shape(f"M{p(14, -19)} L{p(19, -17)} L{p(14, -16)} Z", "#1b1b22", 0)
    b += line(f"M{p(0, -3)} L{p(0, 0)} M{p(4, -3)} L{p(4, 0)}", 2)
    return b


# ================================================================ INGLEWOOD
def ingle_a():
    """The 1912 block on 9 Av SE (ingle2): tan brick between red brick
    pilasters, arched top floor, a pediment with the date, shops below."""
    W, H, T = 450, 430, 40
    defs = brick_pattern("tan", "#dcbf8f") + brick_pattern("red", "#b5482f")
    b = chimney("bld_ingle_a", W, H, 380, T + 28, 30, 26)
    b += rect(0, T + 40, W, H - T - 40, "url(#tan)", 0, 0)
    # Pilasters.
    xs = [4, 74, 144, 214, 284, 354, 424]
    for x in xs:
        b += rect(x, T + 40, 22, 280, "url(#red)", 0, 3)
    # Floors: arched windows on top, square on the middle floor.
    for i in range(6):
        bx = xs[i] + 22
        lit = (i * 5) % 3 == 0
        b += window(bx + 9, T + 70, 30, 68, "#efe6cf", lit=lit, arch=True)
        b += window(bx + 9, T + 172, 30, 58, "#efe6cf", lit=(i * 7) % 4 == 1)
        if i in (1, 4):
            b += wreath(bx + 24, T + 196, 10)
    for y in (T + 152, T + 250):
        b += rect(0, y, W, 9, "#efe0b8", 0, 3)
    # Cornice with dentils and a pediment carrying the date.
    b += rect(-2, T + 26, W + 4, 22, "#8a3a28", 0, 4)
    for x in range(6, W, 14):
        b += rect(x, T + 46, 7, 7, "#efe0b8", 0, 0)
    b += poly([(160, T + 28), (225, T - 26), (290, T + 28)], "#8a3a28", 4)
    b += poly([(186, T + 22), (225, T - 6), (264, T + 22)], "#efe0b8", 3)
    b += text(225, T + 18, "1912", 20, "#8a3a28")
    b += snowcap(-2, W + 2, T + 26, 14)
    b += line(f"M160,{T + 26} L225,{T - 28} L290,{T + 26}", 10, SNOW) + line(f"M160,{T + 24} L225,{T - 32} L290,{T + 24}", 3)
    b += lights(8, W - 8, T + 60, 10)
    # Shopfronts: BAKERY on the left, RECORDS on the right, arched door between.
    def bread(gx, gy, gw, gh):
        s = ""
        for k in range(4):
            s += ell(gx + 20 + k * 30, gy + gh - 12, 14, 8, "#d08a3a", 3)
        s += rect(gx + 6, gy + gh * 0.35, gw - 12, 5, "#8a5a30", 0, 0)
        for k in range(3):
            s += ell(gx + 26 + k * 34, gy + gh * 0.35 - 7, 12, 7, "#e0a050", 3)
        return s
    def records(gx, gy, gw, gh):
        s = ""
        for k in range(4):
            cx = gx + 22 + k * 32
            s += rect(cx - 14, gy + gh - 40, 28, 30, ["#e6382f", "#3577e0", "#ffd23a", "#2fa353"][k], 1, 2.5)
            s += circ(cx, gy + gh - 25, 9, "#1b1b22", 0) + circ(cx, gy + gh - 25, 3, "#ffd23a", 0)
        return s
    b += rect(0, 300 + T - 8, W, 30, "#2b3a4a", 0, 4)
    b += text(80, 300 + T + 14, "BAKERY", 22, "#ffd98a")
    b += text(370, 300 + T + 14, "RECORDS", 22, "#ffd98a")
    b += shopfront(6, 322 + T - 8, 150, H - 314 - T, goods=bread)
    b += shopfront(294, 322 + T - 8, 150, H - 314 - T, goods=records)
    b += rect(156, 314 + T, 138, H - 314 - T, "url(#tan)", 0, 0)
    b += window(190, 330 + T, 70, H - 330 - T, "#3b3f4a", lit=True, arch=True, cross=False, sill=False)
    b += line(f"M225,{340 + T} L225,{H}", 3, "#3b3f4a")
    b += wreath(225, 352 + T, 12)
    b += rect(0, T + 40, W, H - T - 40, "none", 0, LW)
    b += magpie_shape(330, T + 24, 1.0, flip=True)
    write("bld_ingle_a", W, H, b, defs)

def ingle_b():
    """A live-music bar with a vertical blade sign; dark red brick."""
    W, H, T = 300, 330, 40
    defs = brick_pattern("dred", "#9c3a2c")
    b = chimney("bld_ingle_b", W, H, 240, T + 2, 28, 28, "#8a3a2c")
    b += rect(0, T + 20, W, H - T - 20, "url(#dred)", 0, 0)
    # Stepped parapet.
    b += poly([(0, T + 20), (0, T), (90, T), (90, T - 14), (210, T - 14), (210, T), (W, T), (W, T + 20)], "#7a2c22", 4)
    b += snowcap(0, 90, T, 10) + snowcap(90, 210, T - 14, 10) + snowcap(210, W, T, 10)
    b += rect(0, T + 20, W, 12, "#e8d5b0", 0, 3)
    for i, x in enumerate((26, 96, 166)):
        b += window(x, T + 60, 46, 70, "#e8d5b0", lit=i == 1)
    # Marquee board over the door.
    b += rect(10, T + 160, 200, 50, "#fff6e0", 4, 4)
    for x in range(18, 206, 14):
        b += circ(x, T + 164, 3, "#ffd23a", 0) + circ(x, T + 206, 3, "#ffd23a", 0)
    b += text(110, T + 184, "LIVE TONITE", 16, "#2b3a4a")
    b += text(110, T + 203, "THE CHINOOKS", 16, "#e6382f")
    b += shopfront(10, T + 214, 200, H - T - 214, door_at=140, goods=lambda gx, gy, gw, gh:
                   "".join(circ(gx + 20 + k * 26, gy + gh - 18, 9, "#3a2a4a", 0) + rect(gx + 12 + k * 26, gy + gh - 10, 16, 10, "#3a2a4a", 2, 0) for k in range(5)))
    # Blade sign.
    bx, by = 228, T + 34
    b += rect(bx + 22, by - 10, 8, 14, "#3b3f4a", 0, 3)
    b += rect(bx, by, 54, 196, "#1f2a44", 8, 5)
    b += rect(bx + 6, by + 6, 42, 184, "none", 6, 2.5, "#ffd23a")
    for k, ch in enumerate("BLUES"):
        b += text(bx + 27, by + 40 + k * 36, ch, 34, "#4aa3ff", stroke="#cfe8ff", sw=0)
    for y in range(int(by) + 8, int(by) + 192, 16):
        b += circ(bx + 3, y, 2.5, "#ffd23a", 0) + circ(bx + 51, y, 2.5, "#ffd23a", 0)
    b += line(f"M0,{T + 20} L0,{H} M{W},{T + 20} L{W},{H}", LW)
    write("bld_ingle_b", W, H, b, defs)

def ingle_c():
    """A narrow red-brick three-storey with an oriel window and an antiques shop."""
    W, H, T = 270, 440, 30
    defs = brick_pattern("red2", "#b44a33")
    b = chimney("bld_ingle_c", W, H, 40, T + 8, 26, 20, "#7a5a4a")
    b += rect(0, T + 20, W, H - T - 20, "url(#red2)", 0, 0)
    b += rect(-2, T + 6, W + 4, 20, "#e8d5b0", 0, 4)
    for x in range(8, W, 16):
        b += rect(x, T + 24, 8, 8, "#e8d5b0", 0, 0)
    b += snowcap(-2, W + 2, T + 6, 12)
    # Oriel bay across two floors.
    b += rect(70, T + 60, 130, 200, "#e8d5b0", 4, 5)
    for (x, w) in ((80, 34), (118, 34), (156, 34)):
        b += window(x, T + 72, w, 70, "#ffffff", lit=x == 118, sill=False)
        b += window(x, T + 168, w, 70, "#ffffff", lit=x == 80, sill=False)
    b += rect(64, T + 252, 142, 14, "#8a5a3a", 3, 4) + snowcap(64, 206, T + 252, 8, 3)
    b += rect(64, T + 50, 142, 12, "#8a5a3a", 3, 4) + snowcap(64, 206, T + 50, 8, 3)
    for x in (16, 222):
        b += window(x, T + 80, 32, 60, "#e8d5b0")
        b += window(x, T + 176, 32, 60, "#e8d5b0", lit=x == 222)
    # Shop.
    b += rect(0, T + 276, W, 34, "#1f5a3a", 0, 4)
    b += text(W / 2, T + 302, "ANTIQUES", 26, "#ffd23a", stroke="#7a5a10", sw=2)
    def antiques(gx, gy, gw, gh):
        s = rect(gx + 14, gy + 10, 26, gh - 14, "#8a5a30", 4, 3) + circ(gx + 27, gy + 26, 9, "#fff6e0", 2.5)
        s += line(f"M{gx + 27},{gy + 26} L{gx + 27},{gy + 20} M{gx + 27},{gy + 26} L{gx + 32},{gy + 28}", 2)
        s += shape(f"M{gx + 70},{gy + gh} L{gx + 76},{gy + gh - 22} L{gx + 112},{gy + gh - 22} L{gx + 118},{gy + gh} Z", "#e6382f", 3)
        s += ell(gx + 94, gy + gh - 34, 22, 12, "#d08a3a", 3) + circ(gx + 118, gy + gh - 44, 8, "#d08a3a", 3)
        return s
    b += shopfront(0, T + 310, W, H - T - 310, door_at=200, goods=antiques)
    b += line(f"M0,{T + 20} L0,{H} M{W},{T + 20} L{W},{H}", LW)
    write("bld_ingle_c", W, H, b, defs)

def ingle_d():
    """An old wooden false-front hardware store with a porch."""
    W, H, T = 240, 280, 30
    defs = siding_pattern("teal", "#4fa3a0", 10)
    b = poly([(0, T + 30), (40, T + 30), (40, T), (200, T), (200, T + 30), (W, T + 30), (W, H), (0, H)], "url(#teal)", 0)
    b += line(f"M0,{H} L0,{T + 30} L40,{T + 30} L40,{T} L200,{T} L200,{T + 30} L{W},{T + 30} L{W},{H}", LW)
    b += snowcap(40, 200, T, 12) + snowcap(0, 40, T + 30, 8) + snowcap(200, W, T + 30, 8)
    b += rect(36, T + 16, 168, 44, "#fff6e0", 4, 4)
    b += text(120, T + 47, "HARDWARE", 28, "#c72d26")
    b += window(30, T + 80, 50, 60, "#ffffff", lit=True)
    b += window(160, T + 80, 50, 60, "#ffffff")
    # Porch roof on posts.
    b += poly([(-6, T + 160), (W + 6, T + 160), (W - 6, T + 180), (6, T + 180)], "#8a5a3a", 4)
    b += snowcap(-6, W + 6, T + 160, 10)
    for x in (14, 120, 222):
        b += rect(x - 5, T + 180, 10, H - T - 180, "#efe6cf", 0, 3)
    def tools(gx, gy, gw, gh):
        s = line(f"M{gx + 14},{gy + gh} L{gx + 30},{gy + 10}", 5, "#8a5a30") + shape(f"M{gx + 24},{gy + 6} L{gx + 40},{gy + 10} L{gx + 34},{gy + 22} Z", "#9aa3b0", 3)
        s += rect(gx + 46, gy + gh - 30, 26, 30, "#e6382f", 3, 3) + rect(gx + 50, gy + gh - 40, 18, 12, "#e6382f", 2, 3)
        return s
    b += shopfront(26, T + 186, 90, H - T - 186, goods=tools)
    b += rect(140, T + 190, 44, H - T - 190, "#7a4a2a", 2, 4) + wreath(162, T + 210, 10)
    # A snow shovel and a barrel on the porch.
    b += line(f"M206,{H - 4} L224,{H - 70}", 5, "#8a5a30") + poly([(198, H - 2), (214, H - 2), (210, H - 20), (200, H - 20)], "#e6382f", 3)
    write("bld_ingle_d", W, H, b, defs)


# ================================================================ DOWNTOWN
def sand_a():
    """A Stephen Avenue sandstone block: round-arched windows, rusticated base, cafe."""
    W, H, T = 390, 510, 30
    defs = block_pattern("sand", "#e3c995") + block_pattern("sand2", "#d5b880", 40, 20)
    b = rect(0, T + 30, W, H - T - 30, "url(#sand)", 0, 0)
    # Parapet with the date plate and a heavy cornice on brackets.
    b += rect(-4, T + 22, W + 8, 24, "#c9ad7c", 0, 4)
    for x in range(10, W, 30):
        b += poly([(x, T + 46), (x + 12, T + 46), (x + 9, T + 58), (x + 3, T + 58)], "#c9ad7c", 3)
    b += rect(150, T - 6, 90, 30, "#c9ad7c", 4, 4) + text(195, T + 16, "1891", 20, "#7a5a30")
    b += snowcap(-4, W + 4, T + 22, 14) + snowcap(150, 240, T - 6, 10)
    for row, y in enumerate((T + 80, T + 190, T + 290)):
        for i in range(4):
            x = 26 + i * 92
            b += window(x, y, 50, 80, "#7a5a40", lit=(row * 4 + i) % 5 == 2, arch=True)
        b += rect(0, y + 92, W, 8, "#cfb07a", 0, 3)
    b += lights(6, W - 6, T + 66, 10)
    # Rusticated ground floor with the cafe and its awning.
    b += rect(0, T + 380, W, H - T - 380, "url(#sand2)", 0, 0)
    def cafe(gx, gy, gw, gh):
        s = ""
        for k in range(3):
            cx = gx + 30 + k * 60
            s += circ(cx, gy + gh - 34, 10, "#5a3a2a", 0) + rect(cx - 12, gy + gh - 24, 24, 24, ["#3577e0", "#e6382f", "#2fa353"][k], 6, 0)
        s += rect(gx + 6, gy + gh - 10, gw - 12, 6, "#8a5a30", 0, 0)
        return s
    b += shopfront(14, T + 400, 240, H - T - 400, door_at=180, goods=cafe)
    b += awning(8, T + 376, 252, 30, "#2a5fb8", "#2a5fb8")
    b += text(134, T + 400, "CAFE", 22, "#ffffff")
    b += rect(280, T + 396, 90, H - T - 396, "#7a5a40", 2, 4) + rect(290, T + 406, 70, H - T - 412, WARM, 0, 3)
    b += line(f"M325,{T + 406} L325,{H - 6}", 3, "#7a5a40") + wreath(325, T + 424, 12)
    b += rect(0, T + 30, W, H - T - 30, "none", 0, LW)
    write("bld_sand_a", W, H, b, defs)

def sand_b():
    """Red brick over sandstone with a white bay window and a western-wear shop."""
    W, H, T = 330, 480, 30
    defs = brick_pattern("red3", "#bf5a3c") + block_pattern("sand3", "#e8d2a6", 40, 20)
    b = rect(0, T + 24, W, H - T - 24, "url(#red3)", 0, 0)
    b += rect(-4, T + 12, W + 8, 22, "#e8d2a6", 0, 4) + snowcap(-4, W + 4, T + 12, 12)
    # Bay window.
    b += rect(100, T + 70, 130, 220, "#f7f3ea", 4, 5)
    for y in (T + 82, T + 186):
        for x in (110, 150, 190):
            b += window(x, y, 32, 84, "#f7f3ea", lit=(x + y) % 3 == 0, sill=False)
    b += rect(94, T + 60, 142, 14, "#e8d2a6", 3, 4) + snowcap(94, 236, T + 60, 8, 3)
    b += rect(94, T + 284, 142, 16, "#e8d2a6", 3, 4)
    for x in (24, 256):
        for y in (T + 90, T + 196):
            b += window(x, y, 48, 76, "#e8d2a6", lit=(x == 24 and y > T + 100))
    # Sandstone shop level.
    b += rect(0, T + 316, W, H - T - 316, "url(#sand3)", 0, 0)
    b += rect(0, T + 316, W, 40, "#2b2234", 0, 4)
    b += text(W / 2, T + 346, "WHITE HATS", 28, "#ffffff")
    # A white cowboy hat on the sign.
    hx, hy = 34, T + 336
    b += ell(hx, hy + 6, 22, 6, "#ffffff", 3) + shape(f"M{hx - 12},{hy + 4} C{hx - 12},{hy - 14} {hx + 12},{hy - 14} {hx + 12},{hy + 4} Z", "#ffffff", 3)
    def hats(gx, gy, gw, gh):
        s = ""
        for k in range(3):
            cx, cy = gx + 34 + k * 62, gy + 40
            s += line(f"M{cx},{cy + 6} L{cx},{gy + gh}", 3, "#8a5a30")
            s += ell(cx, cy + 6, 24, 6, "#ffffff", 3) + shape(f"M{cx - 13},{cy + 4} C{cx - 13},{cy - 16} {cx + 13},{cy - 16} {cx + 13},{cy + 4} Z", "#ffffff", 3)
        for k in range(2):
            cx = gx + 60 + k * 70
            s += shape(f"M{cx},{gy + gh - 34} L{cx + 14},{gy + gh - 34} L{cx + 14},{gy + gh - 10} L{cx + 30},{gy + gh - 6} L{cx + 30},{gy + gh} L{cx},{gy + gh} Z", "#8a4a20", 3)
        return s
    b += shopfront(10, T + 358, 250, H - T - 358, goods=hats)
    b += rect(270, T + 362, 50, H - T - 362, "#2b2234", 2, 4) + rect(278, T + 370, 34, H - T - 376, WARM, 0, 3)
    b += rect(0, T + 24, W, H - T - 24, "none", 0, LW)
    write("bld_sand_b", W, H, b, defs)

def tower_glass():
    """A blue glass office tower with a peaked crown and a lobby Christmas tree."""
    W, H, T = 480, 1470, 30
    defs = ('<linearGradient id="tg" x1="0" x2="1" y1="0" y2="0"><stop offset="0" stop-color="#4f8fd0"/>'
            '<stop offset="0.55" stop-color="#6aaee6"/><stop offset="1" stop-color="#3f78b8"/></linearGradient>')
    top = T + 90
    b = poly([(0, top), (W / 2, T), (W, top), (W, H), (0, H)], "url(#tg)", 0)
    # Floor bands and mullions.
    for y in range(int(top) + 30, H - 230, 42):
        b += line(f"M0,{y} L{W},{y}", 3, "#2f5f96", 0.9)
    for x in range(30, W, 30):
        b += line(f"M{x},{top - (W / 2 - abs(x - W / 2)) * (90 / (W / 2))} L{x},{H - 230}", 2, "#cfe6ff", 0.5)
    # Sky reflections: two diagonal light bands.
    b += f'<path d="M60,{H - 230} L300,{top + 200} L380,{top + 200} L140,{H - 230} Z" fill="#ffffff" opacity="0.14"/>'
    b += f'<path d="M260,{H - 600} L470,{top + 320} L480,{top + 380} L320,{H - 600} Z" fill="#ffffff" opacity="0.10"/>'
    rnd = random.Random(7)
    for y in range(int(top) + 34, H - 240, 42):
        for x in range(30, W, 30):
            if rnd.random() < 0.06:
                b += rect(x + 3, y + 3, 24, 36, WARM, 0, 0, op=0.85)
    # Crown.
    b += poly([(0, top), (W / 2, T), (W, top)], "#2f5f96", 4)
    b += line(f"M{W / 2},{T} L{W / 2},{T - 26}", 4) + circ(W / 2, T - 30, 6, "#ff4a4a", 3)
    b += line(f"M0,{top} L{W / 2},{T} L{W},{top}", 12, SNOW) + line(f"M0,{top - 4} L{W / 2},{T - 4} L{W},{top - 4}", 3)
    # Lobby: two storeys of clear glass, a big tree, a revolving door.
    ly = H - 230
    b += rect(0, ly, W, 230, "#2f3a4a", 0, 0)
    b += rect(14, ly + 20, W - 28, 210, "#cfeaff", 0, 4)
    b += rect(14, ly + 20, W - 28, 80, "#fff6dc", 0, 0, op=0.7)
    tx = 120
    for (yy, ww) in ((ly + 210, 70), (ly + 160, 56), (ly + 112, 42)):
        b += poly([(tx, yy - 70), (tx + ww, yy), (tx - ww, yy)], "#2fa353", 4)
    b += f'<path d="M{tx},{ly + 30} l6,14 l15,1 l-12,9 l4,15 l-13,-8 l-13,8 l4,-15 l-12,-9 l15,-1 Z" fill="#ffd23a" {ol(3)}/>'
    for (x, y, c) in ((100, ly + 150, "#e6382f"), (140, ly + 180, "#3577e0"), (110, ly + 196, "#ffd23a"), (130, ly + 120, "#ff7fa5")):
        b += circ(x, y, 6, c, 2.5)
    for x in range(14 + 70, W - 14, 70):
        b += line(f"M{x},{ly + 20} L{x},{H}", 4, "#2f3a4a")
    b += circ(330, H - 60, 56, "#9fd3ff", 4) + line(f"M274,{H - 60} L386,{H - 60} M330,{H - 116} L330,{H - 4}", 4, "#2f3a4a")
    b += rect(-4, ly - 8, W + 8, 16, "#2f3a4a", 2, 4) + snowcap(-4, W + 4, ly - 8, 10, 3)
    b += line(f"M0,{top} L0,{H} M{W},{top} L{W},{H}", LW)
    write("bld_tower_glass", W, H, b, defs, scale=1.0)

def tower_concrete():
    """A 1970s beige precast tower with ribbon windows and an antenna."""
    W, H, T = 420, 1230, 60
    b = rect(0, T + 50, W, H - T - 50, "#d9ccb2", 0, 0)
    rnd = random.Random(11)
    for y in range(T + 80, H - 200, 50):
        b += rect(14, y, W - 28, 26, "url(#gGlass)", 2, 0)
        b += sky_in_glass(14, y, W - 28, 26, y)
        for x in range(14, W - 28, 40):
            if rnd.random() < 0.14:
                b += rect(x + 2, y + 2, 36, 22, "url(#gLit)", 0, 0) + interior(x + 2, y + 2, 36, 22, x * y)
            b += line(f"M{x},{y} L{x},{y + 26}", 2, "#d9ccb2")
        b += rect(10, y + 26, W - 20, 5, "#c4b493", 0, 0) + icicles(10, W - 10, y + 31, 6)
    b += f'<rect x="{W - 90}" y="{T + 50}" width="90" height="{H - T - 250}" fill="#000" opacity="0.08"/>'
    # Mechanical penthouse and antenna.
    b += rect(80, T, W - 160, 52, "#c4b493", 0, 4) + snowcap(80, W - 80, T, 10)
    b += line(f"M{W / 2},{T} L{W / 2},{T - 56}", 5) + circ(W / 2, T - 58, 7, "#ff4a4a", 3)
    b += line(f"M{W / 2 - 18},{T - 30} L{W / 2 + 18},{T - 30}", 3)
    b += rect(-4, T + 46, W + 8, 14, "#c4b493", 0, 4) + snowcap(-4, W + 4, T + 46, 10)
    # Colonnade at the street.
    ly = H - 200
    b += rect(0, ly, W, 200, "#3a3f4a", 0, 0) + rect(10, ly + 30, W - 20, 170, "url(#gLit)", 0, 3)
    b += rect(10, ly + 30, W - 20, 60, "#fff1c9", 0, 0, op=0.6)
    for x in range(10, W, 70):
        b += rect(x, ly + 20, 22, 180, "#d9ccb2", 0, 4)
    b += rect(-4, ly, W + 8, 22, "#c4b493", 0, 4) + snowcap(-4, W + 4, ly, 10)
    b += lights(10, W - 10, ly + 24, 10)
    b += rect(0, T + 50, W, H - T - 50, "none", 0, LW)
    write("bld_tower_concrete", W, H, b, scale=1.0)

def city_hall():
    """Old City Hall (1911): sandstone, a clock tower, red pyramid roofs."""
    W, H = 660, 920
    defs = block_pattern("ch", "#dcc391", 50, 25)
    base = 430
    b = ""
    # Wings with red hip roofs.
    for (x0, x1) in ((40, 280), (380, 620)):
        b += poly([(x0 - 10, base), (x0 + 30, base - 80), (x1 - 30, base - 80), (x1 + 10, base)], "#c8433a", 5)
        b += line(f"M{x0 + 30},{base - 80} L{x1 - 30},{base - 80}", 10, SNOW) + line(f"M{x0 + 30},{base - 84} L{x1 - 30},{base - 84}", 3)
        cx = (x0 + x1) / 2
        b += poly([(cx - 22, base - 40), (cx, base - 70), (cx + 22, base - 40), (cx + 22, base - 10), (cx - 22, base - 10)], "#efe2c4", 4)
        b += window(cx - 10, base - 44, 20, 28, "#efe2c4", sill=False)
    # Corner turrets with pointy red caps.
    for x in (40, 620):
        b += rect(x - 26, base - 50, 52, H - base + 50, "url(#ch)", 0, 5)
        b += poly([(x - 32, base - 50), (x, base - 140), (x + 32, base - 50)], "#c8433a", 5)
        b += line(f"M{x},{base - 140} L{x},{base - 162}", 4) + circ(x, base - 164, 5, "#ffd23a", 3)
    b += rect(40, base, 580, H - base, "url(#ch)", 0, 0)
    for row, y in enumerate((base + 60, base + 200)):
        for i, x in enumerate((90, 160, 230, 430, 500, 570)):
            b += window(x - 20, y, 40, 90, "#efe2c4", lit=(row + i) % 4 == 0, arch=row == 0)
    b += rect(36, base + 170, 588, 12, "#c9ad7c", 0, 3) + rect(36, base, 588, 14, "#c9ad7c", 0, 4)
    # Clock tower.
    tx0, tx1 = 270, 390
    b += rect(tx0, 70, tx1 - tx0, H - 70, "url(#ch)", 0, 5)
    b += rect(tx0 - 8, 56, tx1 - tx0 + 16, 22, "#c9ad7c", 0, 4)
    for x in range(int(tx0) - 6, int(tx1) + 6, 20):
        b += rect(x, 30, 12, 28, "#c9ad7c", 0, 3)
    b += snowcap(tx0 - 8, tx1 + 8, 56, 10)
    b += line(f"M330,30 L330,4", 4) + shape("M330,6 L352,12 L330,18 Z", "#e6382f", 3)
    b += circ(330, 130, 38, "#fffaf0", 5) + circ(330, 130, 30, "none", 2)
    for k in range(12):
        a = math.radians(k * 30)
        b += line(f"M{330 + math.cos(a) * 26:.1f},{130 + math.sin(a) * 26:.1f} L{330 + math.cos(a) * 30:.1f},{130 + math.sin(a) * 30:.1f}", 3)
    b += line("M330,130 L330,106 M330,130 L348,136", 4)
    for y in (200, 300):
        b += window(304, y, 24, 70, "#efe2c4", arch=True, sill=False) + window(332, y, 24, 70, "#efe2c4", arch=True, sill=False, lit=y == 300)
    # Entrance arch, steps and a wreath.
    b += f'<path d="M290,{H} L290,{H - 110} A40,40 0 0 1 370,{H - 110} L370,{H} Z" fill="#5a3a2a" {ol(5)}/>'
    b += f'<path d="M302,{H} L302,{H - 104} A28,28 0 0 1 358,{H - 104} L358,{H} Z" fill="{WARM}"/>'
    b += wreath(330, H - 160, 18)
    for k in range(3):
        b += rect(270 - k * 12, H - 14 + 0 - k * 0, 120 + k * 24, 14, "#c9ad7c", 0, 3) if k == 0 else ""
    b += line(f"M40,{base} L40,{H} M620,{base} L620,{H}", LW)
    write("bld_city_hall", W, H, b, defs, scale=1.0)

def wonderland():
    """Jaume Plensa's Wonderland: a giant white wire head on the Bow plaza."""
    W, H = 240, 360
    head = "M120,20 C60,20 34,74 38,140 C40,196 56,240 80,262 C88,282 92,300 92,316 L148,316 C148,300 152,282 160,262 C184,240 200,196 202,140 C206,74 180,20 120,20 Z"
    defs = f'<clipPath id="hd"><path d="{head}"/></clipPath>'
    b = f'<path d="{head}" fill="#ffffff" opacity="0.28"/>'
    g = ""
    for x in range(30, 214, 12):
        g += f'<path d="M{x},10 C{x + (x - 120) * 0.15:.1f},120 {x + (x - 120) * 0.1:.1f},220 {x},330" fill="none" stroke="#ffffff" stroke-width="2.2"/>'
    for y in range(20, 330, 12):
        g += f'<path d="M20,{y} Q120,{y + 10} 220,{y}" fill="none" stroke="#ffffff" stroke-width="2.2"/>'
    b += f'<g clip-path="url(#hd)">{g}</g>'
    # Closed eyes, nose and lips traced in the mesh.
    b += line("M78,132 Q92,140 106,132 M134,132 Q148,140 162,132", 4, "#9aa7bd") + line("M120,140 L114,180 L126,182", 4, "#9aa7bd")
    b += line("M104,206 Q120,214 136,206", 4, "#9aa7bd")
    b += line(head[:-2], 5, "#5a6b86")
    b += rect(70, 316, 100, 44, "#9aa3b0", 4, 5) + snowcap(70, 170, 316, 10)
    b += snowcap(86, 154, 24, 8, 3)
    write("wonderland", W, H, b, defs, metres=7.0)

def ctrain():
    """A Calgary CTrain car: white body, red skirt, toques in the windows."""
    W, H = 760, 190
    b = ""
    # Pantograph.
    b += line("M330,52 L360,18 L400,52 M340,52 L372,22 L392,52", 4) + line("M340,16 L392,16", 6)
    # Body with sloped noses.
    body = "M40,58 L720,58 C744,60 756,92 758,130 L758,160 L2,160 L2,130 C4,92 16,60 40,58 Z"
    b += shape(body, "#f4f6fa", LW)
    b += shape("M2,128 L758,128 L758,160 L2,160 Z", "#d8343a", 0)
    b += line("M2,128 L758,128", 4)
    b += rect(120, 50, 520, 12, "#c9ced8", 4, 4)
    # Windows with passengers.
    rnd = random.Random(3)
    hats = ["#e6382f", "#3577e0", "#2fa353", "#ffd23a", "#ffffff", "#ff7fa5"]
    for i, x in enumerate(range(70, 700, 64)):
        if i in (2, 7):
            # Doors.
            b += rect(x, 70, 52, 88, "#5a6372", 4, 4) + rect(x + 6, 78, 18, 40, "#9fd3ff", 2, 3) + rect(x + 28, 78, 18, 40, "#9fd3ff", 2, 3)
            continue
        b += rect(x, 72, 52, 44, "#2c3e55", 6, 4)
        if rnd.random() < 0.8:
            hx = x + 26 + rnd.uniform(-8, 8)
            b += circ(hx, 104, 11, "#ffe0c4", 2.5)
            hc = rnd.choice(hats)
            if hc == "#ffffff":
                b += ell(hx, 94, 16, 4, "#ffffff", 2) + shape(f"M{hx - 8},{94} C{hx - 8},{82} {hx + 8},{82} {hx + 8},{94} Z", "#ffffff", 2)
            else:
                b += shape(f"M{hx - 11},{100} C{hx - 11},{86} {hx + 11},{86} {hx + 11},{100} Z", hc, 2) + circ(hx, 87, 4, "#ffffff", 1.5)
        b += gloss(x + 4, 74, 44, 40, 0.25)
    # Cab windows and the destination sign.
    b += shape("M10,118 C12,92 22,74 40,72 L56,72 L56,118 Z", "#2c3e55", 4)
    b += shape("M750,118 C748,92 738,74 720,72 L704,72 L704,118 Z", "#2c3e55", 4)
    b += rect(640, 64, 70, 14, "#1b1b22", 2, 3) + text(675, 76, "SOMERSET", 11, "#ffb02e")
    b += text(170, 150, "CTRAIN", 20, "#ffffff")
    b += circ(18, 140, 6, "#fff6a8", 3) + circ(742, 140, 6, "#ff4a4a", 3)
    # Bogies.
    for x in (110, 650):
        b += rect(x - 50, 156, 100, 16, "#3a3f4a", 4, 4)
        b += circ(x - 28, 174, 13, "#2a2f3a", 4) + circ(x + 28, 174, 13, "#2a2f3a", 4)
    b += snowcap(120, 640, 50, 8, 3)
    write("ctrain", W, H, b, metres=24.0)

def plus15():
    """A +15 skywalk: a glazed steel tube with commuters, 15 feet up."""
    W, H = 300, 110
    b = rect(0, 18, W, 84, "#7f8a9c", 6, LW)
    b += rect(8, 28, W - 16, 56, "#cfeaff", 2, 3)
    b += gloss(8, 28, 120, 56, 0.35) + gloss(160, 28, 120, 56, 0.3)
    for x in range(8, W, 46):
        b += line(f"M{x},28 L{x},84", 4, "#7f8a9c")
    # Commuters in the tube.
    for (x, c, hat) in ((70, "#3577e0", "#e6382f"), (150, "#2fa353", "#ffffff"), (210, "#8a5a30", "#ffd23a")):
        b += rect(x - 8, 60, 16, 24, c, 4, 2.5) + circ(x, 52, 8, "#ffe0c4", 2.5)
        b += shape(f"M{x - 8},{50} C{x - 8},{40} {x + 8},{40} {x + 8},{50} Z", hat, 2)
    b += snowcap(-2, W + 2, 18, 10)
    b += circ(W - 30, 90, 17, "#2a5fb8", 4) + text(W - 30, 96, "+15", 15, "#ffffff")
    write("plus15", W, H, b, metres=10.0)

def stephen_lamp():
    """Stephen Avenue's dark green lamp standard with a blue banner and wreath."""
    W, H = 120, 420
    b = rect(52, 80, 16, 330, "#1f4a3a", 3, 4)
    b += rect(40, 396, 40, 22, "#1f4a3a", 4, 4)
    b += line("M60,84 C30,70 24,50 30,36 M60,84 C90,70 96,50 90,36", 6, "#1f4a3a")
    for x in (30, 90):
        b += poly([(x - 14, 36), (x + 14, 36), (x + 9, 8), (x - 9, 8)], "#1f4a3a", 4)
        b += poly([(x - 10, 33), (x + 10, 33), (x + 6, 12), (x - 6, 12)], "#ffe27a", 0)
        b += circ(x, 4, 5, "#1f4a3a", 3)
    b += line("M68,120 L108,120", 4, "#1f4a3a")
    b += shape("M70,124 L106,124 L106,250 L88,236 L70,250 Z", "#2a6fd6", 4)
    for (a, bb) in (((88, 156), (88, 204)), ((70, 180), (106, 180)), ((76, 166), (100, 194)), ((76, 194), (100, 166))):
        b += line(f"M{a[0]},{a[1]} L{bb[0]},{bb[1]}", 4, "#ffffff")
    b += wreath(60, 290, 16)
    write("stephen_lamp", W, H, b, metres=1.6)

def temp_sign():
    """A bank's time-and-temperature sign mid-Chinook: +12 in December."""
    W, H = 150, 300
    b = rect(66, 120, 18, 176, "#5a6372", 3, 4) + rect(50, 286, 50, 12, "#5a6372", 3, 4)
    b += rect(6, 10, 138, 116, "#1b1b22", 10, 5)
    b += rect(14, 18, 122, 48, "#2a0a0a", 4, 0) + text(75, 56, "+12°C", 36, "#ff9a2e")
    b += rect(14, 72, 122, 46, "#2a0a0a", 4, 0) + text(75, 106, "DEC 24", 26, "#ff9a2e")
    write("temp_sign", W, H, b, metres=2.4)


# ================================================================ BOW RIVER
def peace_bridge():
    """The Peace Bridge: a red helix tube with a glass roof over the Bow."""
    W, H = 1620, 250
    b = ""
    top, bot = 40, 196
    # Abutments.
    for x in (0, W - 90):
        b += rect(x, bot - 10, 90, H - bot + 10, "#b8b0a0", 4, 5)
    # Deck.
    b += rect(40, bot, W - 80, 22, "#e9edf3", 4, LW)
    b += snowcap(40, W - 40, bot, 8, 3)
    # Back members of the helix (darker), glass roof, then front members.
    step = 54
    for x in range(40, W - 40, step):
        b += line(f"M{x},{bot} C{x + 10},{top + 40} {x + 40},{top} {x + step * 1.6:.0f},{top}", 12, "#7a1b22")
    b += rect(60, top - 6, W - 120, 22, "#bfe3ff", 10, 4)
    b += gloss(60, top - 6, 600, 22, 0.4)
    b += snowcap(60, W - 60, top - 6, 10, 3)
    for x in range(40 + step // 2, W - 40, step):
        d = f"M{x},{top} C{x + 30},{top} {x + 50},{bot - 50} {x + step * 1.4:.0f},{bot}"
        b += line(d, 16, INK) + line(d, 10, "#d6283a")
    # Top and bottom chords.
    b += line(f"M50,{top + 14} L{W - 50},{top + 14}", 14, INK) + line(f"M50,{top + 14} L{W - 50},{top + 14}", 8, "#d6283a")
    b += line(f"M40,{bot - 4} L{W - 40},{bot - 4}", 12, INK) + line(f"M40,{bot - 4} L{W - 40},{bot - 4}", 6, "#d6283a")
    # Railing glass.
    b += rect(50, bot - 30, W - 100, 26, "#cfeaff", 0, 0, op=0.5)
    write("peace_bridge", W, H, b, metres=54.0, scale=1.0)


# ================================================================ KENSINGTON
def plaza():
    """The Plaza Theatre: art deco stucco, a red PLAZA blade, a hopeful marquee."""
    W, H, T = 360, 380, 50
    b = poly([(0, T + 40), (60, T + 40), (60, T + 14), (120, T + 14), (120, T - 10), (240, T - 10), (240, T + 14), (300, T + 14), (300, T + 40), (W, T + 40), (W, H), (0, H)], "#f1e6cc", LW)
    for x in range(140, 230, 18):
        b += rect(x, T + 2, 8, 70, "#e0cfa8", 2, 0)
    b += snowcap(120, 240, T - 10, 12) + snowcap(60, 120, T + 14, 10) + snowcap(240, 300, T + 14, 10)
    b += snowcap(0, 60, T + 40, 8) + snowcap(300, W, T + 40, 8)
    for x in (30, 290):
        b += window(x, T + 80, 40, 52, "#c9a86a", lit=x == 290)
    # Blade sign.
    b += rect(150, T + 84, 60, 130, "#d6283a", 6, 5)
    for k, ch in enumerate("PLAZA"):
        b += text(180, T + 108 + k * 24, ch, 24, "#ffffff")
    for y in range(T + 90, T + 210, 14):
        b += circ(155, y, 2.5, "#ffe27a", 0) + circ(205, y, 2.5, "#ffe27a", 0)
    # Marquee.
    b += poly([(14, T + 220), (W - 14, T + 220), (W - 4, T + 290), (4, T + 290)], "#2b2234", 5)
    b += rect(24, T + 228, W - 48, 54, "#fffaf0", 3, 3)
    b += text(W / 2, T + 252, "IT'S A WONDERFUL", 20, "#1b1b22", length=260)
    b += text(W / 2, T + 276, "CHINOOK", 22, "#d6283a")
    for x in range(12, W - 8, 16):
        b += circ(x, T + 294, 3, "#ffe27a", 2, "#c99a1e")
    b += snowcap(14, W - 14, T + 220, 10)
    # Doors, posters and a ticket booth.
    b += rect(20, T + 300, W - 40, H - T - 300, "#7a2c22", 0, 4)
    for x in (34, 286):
        b += rect(x, T + 310, 40, 56, "#fff6e0", 2, 3) + rect(x + 6, T + 316, 28, 30, ["#3577e0", "#2fa353"][x > 100], 2, 0)
    b += rect(110, T + 306, 140, H - T - 306, "#5a2018", 2, 4)
    for x in (116, 186):
        b += rect(x, T + 312, 58, H - T - 316, WARM, 2, 3)
    write("bld_plaza", W, H, b)

def kens_a():
    """A painted-brick coffee shop with a striped awning and fairy lights."""
    W, H, T = 270, 300, 30
    defs = brick_pattern("blue", "#a5cfe0")
    b = rect(0, T + 20, W, H - T - 20, "url(#blue)", 0, 0)
    b += rect(-4, T + 8, W + 8, 18, "#ffffff", 0, 4) + snowcap(-4, W + 4, T + 8, 12)
    for x in (24, 110, 196):
        b += window(x, T + 48, 50, 66, "#ffffff", lit=x == 110)
    b += lights(6, W - 6, T + 32, 8)
    def mugs(gx, gy, gw, gh):
        s = ""
        for k in range(3):
            cx = gx + 30 + k * 56
            s += circ(cx, gy + gh - 34, 10, "#5a3a2a", 0) + rect(cx - 12, gy + gh - 24, 24, 24, ["#e6382f", "#ffd23a", "#2fa353"][k], 6, 0)
            s += rect(cx + 10, gy + gh - 40, 14, 12, "#ffffff", 3, 2)
        return s
    b += shopfront(8, T + 150, 190, H - T - 150, goods=mugs)
    b += awning(4, T + 136, 198, 24, "#2a9d8f")
    b += rect(210, T + 156, 50, H - T - 156, "#2a9d8f", 2, 4) + rect(218, T + 164, 34, H - T - 170, WARM, 0, 3) + wreath(235, T + 180, 10)
    # Hanging coffee cup sign.
    b += line(f"M212,{T + 124} L262,{T + 124}", 4)
    b += shape(f"M220,{T + 128} L254,{T + 128} L248,{T + 162} L226,{T + 162} Z", "#ffffff", 4) + rect(222, T + 138, 30, 8, "#2a9d8f", 0, 0)
    b += line(f"M230,{T + 120} Q226,{T + 108} 232,{T + 98} M242,{T + 120} Q238,{T + 106} 244,{T + 96}", 3, "#9aa7bd")
    b += rect(0, T + 20, W, H - T - 20, "none", 0, LW)
    write("bld_kens_a", W, H, b, defs)

def kens_b():
    """A six-storey condo: glass balconies, a snowed-in barbecue, a red flag."""
    W, H, T = 450, 650, 20
    b = rect(0, T + 20, W, H - T - 20, "#e9e6df", 0, 0)
    b += rect(300, T + 20, 150, H - T - 20, "#6b7380", 0, 0)
    b += rect(250, T + 20, 50, H - T - 20, "#c08a5a", 0, 0)
    for k in range(6):
        y = T + 50 + k * 92
        for x in (20, 140):
            b += window(x, y, 96, 70, "#ffffff", lit=(k + x) % 3 == 0, cross=False, sill=False)
        b += window(320, y, 110, 70, "#c9ced8", lit=k % 2 == 1, cross=False, sill=False)
        # Balcony slab and glass rail.
        b += rect(10, y + 70, 240, 10, "#c9ced8", 0, 4) + rect(14, y + 44, 232, 26, "#cfeaff", 0, 3, op=0.85)
        b += snowcap(10, 250, y + 70, 6, 3)
        if k == 1:
            b += line(f"M40,{y + 44} L40,{y - 10}", 3) + shape(f"M40,{y - 10} L80,{y - 4} L40,{y + 10} Z", "#d6283a", 3)
        if k == 3:
            b += rect(170, y + 28, 50, 30, "#3a3f4a", 6, 3) + snowcap(170, 220, y + 28, 8, 3) + line(f"M176,{y + 58} L176,{y + 70} M214,{y + 58} L214,{y + 70}", 3)
        if k == 2:
            b += lights(14, 246, y + 40, 6, 18, 3.5)
        if k == 4:
            b += circ(60, y + 52, 12, SNOW, 3) + circ(60, y + 34, 9, SNOW, 3) + shape(f"M60,{y + 34} L72,{y + 36} L60,{y + 38} Z", "#ff7a1a", 2)
    b += rect(-4, T + 8, W + 8, 16, "#3a3f4a", 0, 4) + snowcap(-4, W + 4, T + 8, 10)
    ly = H - 100
    b += rect(0, ly, W, 100, "#3a3f4a", 0, 0)
    b += shopfront(10, ly + 10, 280, 90, goods=lambda gx, gy, gw, gh: "".join(circ(gx + 30 + k * 70, gy + gh - 14, 12, "none", 4) + circ(gx + 62 + k * 70, gy + gh - 14, 12, "none", 4) + line(f"M{gx + 30 + k * 70},{gy + gh - 14} L{gx + 46 + k * 70},{gy + gh - 34} L{gx + 62 + k * 70},{gy + gh - 14}", 4, "#e6382f") for k in range(3)))
    b += rect(310, ly + 16, 120, 84, "#5a6372", 2, 4) + rect(320, ly + 26, 100, 74, WARM, 0, 3)
    b += rect(0, T + 20, W, H - T - 20, "none", 0, LW)
    write("bld_kens_b", W, H, b, scale=1.0)

def kens_c():
    """A green pub with gold letters and a chalkboard out front."""
    W, H, T = 300, 320, 40
    b = chimney("bld_kens_c", W, H, 236, T + 8, 28, 28)
    b += rect(0, T + 20, W, H - T - 20, "#2f6b4a", 0, 0)
    b += rect(-4, T + 6, W + 8, 20, "#1f4a33", 0, 4) + snowcap(-4, W + 4, T + 6, 12)
    for x in (30, 125, 220):
        b += window(x, T + 56, 50, 70, "#efe6cf", lit=x != 125)
    b += rect(10, T + 150, W - 20, 40, "#1f4a33", 2, 4)
    b += text(W / 2, T + 180, "THE DOG & SLED", 24, "#ffd23a", stroke="#7a5a10", sw=1.5)
    b += shopfront(10, T + 192, 190, H - T - 192, frame="#1f4a33",
                   goods=lambda gx, gy, gw, gh: "".join(rect(gx + 18 + k * 40, gy + gh - 40, 20, 34, "#ffb02e", 3, 2.5) + rect(gx + 18 + k * 40, gy + gh - 44, 20, 8, "#ffffff", 3, 2) for k in range(4)))
    b += rect(212, T + 198, 64, H - T - 198, "#1f4a33", 2, 4) + rect(220, T + 206, 48, H - T - 212, WARM, 0, 3) + wreath(244, T + 222, 10)
    b += rect(0, T + 20, W, H - T - 20, "none", 0, LW)
    write("bld_kens_c", W, H, b)

def chalkboard():
    W, H = 90, 110
    b = poly([(10, 106), (24, 6), (66, 6), (80, 106)], "#8a5a30", 4)
    b += rect(22, 14, 46, 64, "#2a2f2a", 3, 3)
    b += text(45, 34, "SOUP", 13, "#ffffff") + text(45, 52, "TODAY", 11, "#ffd23a") + text(45, 72, "+12°", 16, "#ff9a2e")
    write("chalkboard", W, H, b, metres=0.9)


# ================================================================ 17 AVENUE
def red_a():
    """A 17 Av pub on the Red Mile with a GO FLAMES GO neon and fans in the windows."""
    W, H, T = 330, 380, 40
    defs = brick_pattern("rm", "#c23a2f") + '<filter id="neon" x="-20%" y="-50%" width="140%" height="200%"><feGaussianBlur stdDeviation="5"/></filter>'
    b = rect(0, T + 20, W, H - T - 20, "url(#rm)", 0, 0)
    b += rect(-4, T + 6, W + 8, 20, "#1b1b22", 0, 4) + snowcap(-4, W + 4, T + 6, 12)
    b += rect(30, T + 44, 270, 70, "#1b1b22", 8, 4)
    b += text(165, T + 92, "GO FLAMES GO", 36, "#ff6a3a", op=0.8).replace('opacity="0.8"', 'opacity="0.8" filter="url(#neon)"')
    b += text(165, T + 92, "GO FLAMES GO", 36, "#fff2e0", stroke="#ff6a3a", sw=3)
    for x in (40, 140, 240):
        b += window(x, T + 134, 50, 60, "#efe6cf", lit=True)
    b += rect(0, T + 210, W, 22, "#1b1b22", 0, 4)
    b += text(W / 2, T + 228, "C OF RED", 18, "#ff6a3a")
    def fans(gx, gy, gw, gh):
        s = ""
        for k in range(5):
            cx = gx + 20 + k * 40
            s += rect(cx - 13, gy + gh - 34, 26, 34, "#d6283a", 6, 2.5) + rect(cx - 13, gy + gh - 22, 26, 5, "#ffffff", 0, 0)
            s += circ(cx, gy + gh - 44, 10, "#ffe0c4", 2.5)
            if k % 2 == 0:
                s += line(f"M{cx - 10},{gy + gh - 30} L{cx - 18},{gy + gh - 54}", 6, "#d6283a") + circ(cx - 18, gy + gh - 56, 4, "#ffe0c4", 2)
        return s
    b += shopfront(8, T + 234, 220, H - T - 234, goods=fans)
    b += rect(238, T + 238, 80, H - T - 238, "#1b1b22", 2, 4) + rect(246, T + 246, 64, H - T - 252, WARM, 0, 3)
    b += rect(0, T + 20, W, H - T - 20, "none", 0, LW)
    write("bld_red_a", W, H, b, defs)

def red_b():
    """A late-night donair shop, because Alberta."""
    W, H, T = 300, 300, 40
    # The donair spit's exhaust, going all night.
    b = chimney("bld_red_b", W, H, 250, T + 8, 22, 28, "#9aa3b0")
    b += rect(0, T + 20, W, H - T - 20, "#3aa6a0", 0, 0)
    b += rect(-4, T + 6, W + 8, 20, "#ffd23a", 0, 4) + snowcap(-4, W + 4, T + 6, 12)
    b += rect(20, T + 40, 260, 70, "#fff6e0", 8, 5)
    b += text(160, T + 90, "DONAIRS", 40, "#e6382f", stroke="#7a1b12", sw=2)
    # A donair on the sign.
    b += shape(f"M30,{T + 98} L48,{T + 50} L70,{T + 98} Z", "#f2d29a", 4) + shape(f"M38,{T + 76} L48,{T + 54} L60,{T + 76} Z", "#a8502a", 0)
    b += line(f"M40,{T + 70} L56,{T + 70}", 3, "#2fa353")
    b += rect(70, T + 124, 160, 26, "#1b1b22", 4, 4) + text(150, T + 144, "OPEN LATE", 18, "#ff9a2e")
    b += shopfront(10, T + 160, 190, H - T - 160, goods=lambda gx, gy, gw, gh: rect(gx + 10, gy + gh - 30, gw - 20, 30, "#c9ced8", 2, 3) + "".join(circ(gx + 30 + k * 36, gy + gh - 40, 10, "#ffe0c4", 2.5) for k in range(4)))
    b += rect(210, T + 166, 70, H - T - 166, "#2a6f6a", 2, 4) + rect(218, T + 174, 54, H - T - 180, WARM, 0, 3)
    b += rect(0, T + 20, W, H - T - 20, "none", 0, LW)
    write("bld_red_b", W, H, b)

def red_c():
    """Brick with a rooftop patio that is OPEN, heaters on, in December."""
    W, H, T = 360, 430, 80
    defs = brick_pattern("br", "#a05a3c")
    b = rect(0, T + 20, W, H - T - 20, "url(#br)", 0, 0)
    # Rooftop patio: rail, umbrellas under snow, heaters, diners in toques.
    for (x, c) in ((70, "#e6382f"), (250, "#2a5fb8")):
        b += line(f"M{x},{T + 20} L{x},{T - 40}", 4)
        b += shape(f"M{x - 54},{T - 30} Q{x},{T - 66} {x + 54},{T - 30} Z", c, 4) + snowcap(x - 40, x + 40, T - 46, 8, 3)
    for x in (160,):
        b += line(f"M{x},{T + 20} L{x},{T - 40}", 5, "#9aa3b0") + shape(f"M{x - 16},{T - 40} L{x + 16},{T - 40} L{x + 8},{T - 56} L{x - 8},{T - 56} Z", "#9aa3b0", 3)
        b += f'<ellipse cx="{x}" cy="{T - 34}" rx="20" ry="8" fill="#ff9a2e" opacity="0.6"/>'
    for (x, hc) in ((110, "#3fbf7a"), (210, "#ffd23a"), (300, "#e6382f")):
        b += rect(x - 12, T - 6, 24, 26, "#5a6372", 6, 2.5) + circ(x, T - 16, 10, "#ffe0c4", 2.5)
        b += shape(f"M{x - 10},{T - 18} C{x - 10},{T - 32} {x + 10},{T - 32} {x + 10},{T - 18} Z", hc, 2) + circ(x, T - 31, 3.5, "#ffffff", 1.5)
    b += rect(-2, T, W + 4, 22, "#cfeaff", 0, 4, op=0.9)
    for x in range(0, W, 30):
        b += line(f"M{x},{T} L{x},{T + 22}", 3, "#7f8a9c")
    b += line(f"M-2,{T} L{W + 2},{T}", 5, "#7f8a9c")
    b += rect(60, T + 34, 240, 34, "#fff6e0", 4, 4) + text(180, T + 60, "PATIO OPEN!", 24, "#2fa353")
    for y in (T + 90, T + 180):
        for x in (30, 130, 230):
            b += window(x, y, 90 if x != 130 else 90, 64, "#efe6cf", lit=(x + y) % 3 == 1)
    b += shopfront(10, T + 268, 240, H - T - 268, door_at=180)
    b += rect(262, T + 268, 88, H - T - 268, "#5a3a2a", 0, 4)
    b += rect(0, T + 20, W, H - T - 20, "none", 0, LW)
    write("bld_red_c", W, H, b, defs)

def saddledome():
    """The Saddledome: the saddle roof that dips in the middle, wearing a Santa hat."""
    W, H = 1980, 720
    b = ""
    roof_top = "M40,120 C420,150 760,290 990,300 C1220,290 1560,150 1940,120"
    roof_bot = "M1900,210 C1560,230 1220,370 990,380 C760,370 420,230 80,210"
    # Concrete bowl below the roof.
    b += shape("M150,250 C500,310 800,420 990,428 C1180,420 1480,310 1830,250 L1830,720 L150,720 Z", "#d7d1c4", LW)
    for k, y in enumerate(range(470, 700, 46)):
        b += line(f"M150,{y} L1830,{y}", 3, "#b8b0a0")
    for x in range(200, 1830, 90):
        b += line(f"M{x},{460 if 600 < x < 1380 else 330} L{x},720", 4, "#b8b0a0")
    # Glass entrance with the name.
    b += rect(760, 520, 460, 200, "#2f3a4a", 4, LW)
    b += rect(774, 534, 432, 186, "#9fd3ff", 0, 3) + gloss(774, 534, 200, 186, 0.3)
    b += rect(774, 534, 432, 60, "#fff6dc", 0, 0, op=0.6)
    for x in range(774, 1206, 54):
        b += line(f"M{x},534 L{x},720", 4, "#2f3a4a")
    b += rect(700, 450, 580, 60, "#1b1b22", 8, LW)
    b += text(990, 496, "SADDLEDOME", 46, "#ff4a3a", stroke="#ffd2c4", sw=1.5)
    # The roof: a thick curved slab, red band, snow on top.
    b += shape(roof_top + " L1900,210 " + roof_bot[roof_bot.index('C'):] + " Z", "#ece7dc", LW)
    b += line("M60,170 C440,190 770,330 990,340 C1210,330 1540,190 1920,170", 14, "#d6283a")
    b += line(roof_top, 18, SNOW) + line("M40,112 C420,142 760,282 990,292 C1220,282 1560,142 1940,112", 4)
    # Santa hat on the right horn.
    hx, hy = 1860, 120
    b += shape(f"M{hx - 90},{hy + 6} C{hx - 70},{hy - 120} {hx + 30},{hy - 160} {hx + 120},{hy - 110} C{hx + 70},{hy - 100} {hx + 40},{hy - 60} {hx + 70},{hy + 6} Z", "#e6382f", LW)
    b += rect(hx - 104, hy - 6, 190, 34, "#ffffff", 16, LW) + circ(hx + 124, hy - 108, 26, "#ffffff", LW)
    # GO FLAMES GO banners on the bowl.
    for x in (330, 1560):
        b += rect(x, 360, 90, 200, "#d6283a", 4, 5)
        for k, word in enumerate(("GO", "FLAMES", "GO")):
            b += text(x + 45, 410 + k * 56, word, 24 if word != "FLAMES" else 19, "#ffffff")
    write("saddledome", W, H, b, metres=66.0, scale=1.0)

def cone():
    W, H = 60, 80
    b = poly([(6, 74), (54, 74), (36, 6), (24, 6)], "#ff7a1a", 4)
    b += poly([(18, 44), (42, 44), (39, 32), (21, 32)], "#ffffff", 0)
    b += rect(0, 70, 60, 10, "#ff7a1a", 3, 4)
    write("cone", W, H, b, metres=0.55)

def roadwork():
    """Calgary has two seasons: winter and construction."""
    W, H = 200, 260
    b = line("M100,120 L100,256", 8, "#5a6372") + line("M70,256 L130,256", 8, "#5a6372")
    b += poly([(100, 8), (178, 86), (100, 164), (22, 86)], "#ff9a1a", LW)
    b += text(100, 82, "ROAD", 30, "#1b1b22") + text(100, 114, "WORK", 30, "#1b1b22")
    b += rect(30, 172, 140, 60, "#ffffff", 4, 5)
    b += text(100, 196, "SEASON:", 18, "#1b1b22") + text(100, 222, "ALL YEAR", 22, "#e6382f")
    write("roadwork", W, H, b, metres=2.0)


# ================================================================ SUBURBS
def house(name, siding, door, trim="#ffffff", flip=False):
    """The suburban two-storey with the double garage out front. Every one is
    the same house; only the door changes."""
    W, H, T = 400, 330, 40
    defs = siding_pattern("sd" + name[-1], siding, 12)
    g = chimney(name, W, H, 320, T + 44, 30, 60)
    if flip:
        fx, fy = SMOKE[name]
        SMOKE[name] = (round(1 - fx, 3), fy)
    # Main house, gable roof.
    g += rect(150, T + 80, 230, H - T - 80, f"url(#sd{name[-1]})", 0, LW)
    g += poly([(130, T + 90), (265, T - 10), (400, T + 90)], "#5a5f6a", LW)
    g += line(f"M134,{T + 86} L265,{T - 14} L396,{T + 86}", 12, SNOW) + line(f"M134,{T + 82} L265,{T - 18} L396,{T + 82}", 3)
    g += lights(150, 380, T + 96, 8, 20, 4)
    g += window(180, T + 116, 60, 56, trim, lit=False) + window(290, T + 116, 60, 56, trim, lit=True)
    # Garage, front-attached, with a snowy apron.
    g += rect(0, T + 150, 170, H - T - 150, f"url(#sd{name[-1]})", 0, LW)
    g += poly([(-8, T + 160), (85, T + 110), (178, T + 160)], "#5a5f6a", LW)
    g += line(f"M-4,{T + 156} L85,{T + 106} L174,{T + 156}", 10, SNOW) + line(f"M-4,{T + 152} L85,{T + 102} L174,{T + 152}", 3)
    g += rect(16, T + 176, 138, H - T - 176, "#f4f4f2", 4, 4)
    for y in range(T + 196, H, 22):
        g += line(f"M16,{y} L154,{y}", 3, "#c9ced8")
    for x in range(30, 150, 24):
        g += rect(x, T + 184, 16, 8, "#9fd3ff", 1, 2)
    # Door, porch light, wreath.
    g += rect(250, T + 214, 50, H - T - 214, door, 3, 4) + circ(290, T + 260, 3, "#ffd23a", 0) + wreath(275, T + 236, 10)
    g += rect(240, T + 204, 70, 10, "#5a5f6a", 2, 3) + snowcap(240, 310, T + 204, 6, 3)
    g += window(320, T + 220, 46, 46, trim, lit=True)
    if flip:
        g = f'<g transform="translate({W} 0) scale(-1 1)">{g}</g>'
    write(name, W, H, g, defs)

def comm_sign():
    """Every new community: Something Ridge Park Hills."""
    W, H = 330, 150
    b = rect(10, 70, 310, 76, "#b8a890", 6, LW)
    for (x, y, w) in ((24, 84, 60), (96, 80, 50), (160, 86, 70), (244, 82, 60), (40, 116, 70), (126, 112, 60), (200, 118, 80)):
        b += rect(x, y, w, 22, "#a8987e", 8, 3)
    b += snowcap(10, 320, 70, 8)
    b += rect(30, 8, 270, 82, "#4a3a2a", 6, LW)
    b += text(165, 42, "SAGE RIDGE", 26, "#ffd98a") + text(165, 76, "PARK HILLS", 26, "#ffd98a")
    b += snowcap(30, 300, 8, 10)
    b += rect(60, 100, 210, 30, "#e6382f", 3, 4) + text(165, 122, "PHASE 47 NOW SELLING", 15, "#ffffff")
    write("comm_sign", W, H, b, metres=7.0)

def blue_ring():
    """Travelling Light: the giant blue ring with a streetlamp through it."""
    W, H = 480, 540
    b = rect(140, 506, 200, 30, "#b8b0a0", 4, LW) + snowcap(140, 340, 506, 8, 3)
    # Back half of the ring.
    b += f'<circle cx="240" cy="260" r="210" fill="none" stroke="{INK}" stroke-width="56"/>'
    b += f'<circle cx="240" cy="260" r="210" fill="none" stroke="#2a7fd6" stroke-width="44"/>'
    b += f'<circle cx="240" cy="260" r="222" fill="none" stroke="#5aa8f0" stroke-width="8" stroke-dasharray="300 1100" transform="rotate(200 240 260)"/>'
    # Legs into the pad.
    b += line("M190,462 L180,506 M290,462 L300,506", 22) + line("M190,462 L180,506 M290,462 L300,506", 12, "#2a7fd6")
    # The streetlamp inside the ring.
    b += line("M240,506 L240,140 C240,110 260,96 300,96", 14) + line("M240,506 L240,140 C240,110 260,96 300,96", 7, "#9aa3b0")
    b += shape("M290,88 L350,92 C352,104 340,110 300,108 Z", "#9aa3b0", 4) + shape("M300,106 L342,104 L338,112 L304,112 Z", "#ffe27a", 2)
    b += snowcap(160, 320, 52, 10, 3)
    write("blue_ring", W, H, b, metres=16.0)

def drive_in():
    """A 1960s drive-in with a towering SHAKES sign."""
    W, H, T = 450, 420, 20
    b = ""
    # Tall sign.
    b += rect(40, 120, 18, H - 120, "#5a6372", 2, 4)
    b += rect(0, T, 150, 120, "#ffffff", 10, LW)
    for x in range(6, 144, 18):
        b += rect(x, T + 92, 9, 9, "#1b1b22", 0, 0) + rect(x + 9, T + 101, 9, 9, "#1b1b22", 0, 0)
    b += text(75, T + 52, "DRIVE-IN", 30, "#2a5fb8", stroke="#ffffff", sw=0)
    b += shape(f"M14,{T + 64} Q75,{T + 50} 136,{T + 64} L136,{T + 84} Q75,{T + 70} 14,{T + 84} Z", "#e6382f", 3)
    b += text(75, T + 82, "SINCE 1964", 14, "#ffffff")
    b += rect(10, T + 140, 130, 50, "#e6382f", 6, LW) + text(75, T + 176, "SHAKES", 30, "#ffffff")
    # Low building with red and blue stripes, order windows.
    b += rect(160, 220, 290, H - 220, "#ffffff", 0, LW)
    b += rect(160, 220, 290, 20, "#e6382f", 0, 4) + rect(160, 240, 290, 14, "#2a5fb8", 0, 4)
    b += snowcap(156, 454, 220, 12)
    for x in (180, 280, 380):
        b += rect(x, 280, 60, 70, WARM, 4, 4) + rect(x - 4, 350, 68, 8, "#9aa3b0", 2, 3)
    b += circ(310, 300, 12, "#ffe0c4", 2.5) + shape("M298,296 C298,280 322,280 322,296 Z", "#ffffff", 2)
    b += text(310, 380, "ORDER HERE", 16, "#2a5fb8")
    write("drive_in", W, H, b, metres=15.0)

def car(name, body, roof_snow=True, plug=True):
    """A sedan on a winter morning: snow on the roof, block heater plugged in."""
    W, H = 300, 130
    b = ""
    if plug:
        b += rect(6, 70, 14, 56, "#7f8a9c", 3, 3) + rect(2, 70, 22, 12, "#9aa3b0", 2, 3)
        b += line("M20,80 C40,120 50,120 62,96", 4, "#1b1b22")
    b += shape("M50,96 L60,66 C66,58 80,56 96,56 L120,30 C126,24 136,22 150,22 L210,22 C222,22 232,26 240,34 L264,58 C282,60 292,66 294,80 L296,100 C296,106 292,110 286,110 L56,110 C50,110 48,104 50,96 Z", body, LW)
    b += shape("M128,36 L150,32 L174,32 L174,58 L104,58 Z", "#9fd3ff", 4) + shape("M184,32 L208,32 C218,32 226,36 232,44 L244,58 L184,58 Z", "#9fd3ff", 4)
    b += gloss(110, 34, 60, 22, 0.35)
    b += rect(176, 64, 22, 6, darker(body), 2, 2)
    b += circ(100, 110, 20, "#2a2f3a", LW) + circ(100, 110, 8, "#9aa3b0", 3)
    b += circ(240, 110, 20, "#2a2f3a", LW) + circ(240, 110, 8, "#9aa3b0", 3)
    b += ell(284, 76, 8, 6, "#fff6a8", 3) + rect(48, 74, 12, 10, "#ff4a4a", 3, 3)
    # Door seam, mirror, chrome bumpers, and the road salt and slush every
    # Calgary car wears from November to April.
    b += line("M178,58 L178,104 M110,62 L104,104", 2.5, darker(body, 0.7))
    b += shape("M238,46 L252,44 L254,56 L242,58 Z", body, 3)
    b += rect(272, 96, 26, 8, "#d8dee8", 3, 3) + rect(44, 96, 22, 8, "#d8dee8", 3, 3)
    rnd = random.Random(name)
    for _ in range(26):
        x, y = rnd.uniform(60, 290), rnd.uniform(80, 96)
        if abs(x - 100) > 24 and abs(x - 240) > 24:
            b += f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{rnd.uniform(1.5, 4):.1f}" fill="#9a8f84" opacity="0.6"/>'
    if body == "#ffc22a":
        b += rect(150, 8, 46, 16, "#1b1b22", 4, 3) + text(173, 21, "TAXI", 12, "#ffd23a")
    b += wreath(286, 92, 7)
    if roof_snow:
        b += shape("M130,24 Q150,6 172,14 Q196,4 214,22 Z", SNOW, 4)
    write(name, W, H, b, metres=4.8)

def pickup():
    """A pickup with the family's Christmas tree in the box."""
    W, H = 360, 170
    b = ""
    # Tree in the box.
    b += poly([(40, 90), (100, 24), (170, 90)], "#2fa353", LW) + line("M30,96 L178,96", 8, "#c9a26a")
    b += shape("M60,74 Q100,60 140,74", "none", 0)
    b += line("M60,72 L140,72", 3, "#e6382f")
    b += shape("M14,96 L196,96 L196,136 L14,136 Z", "#5a6b86", LW)
    b += shape("M196,136 L196,70 C196,62 202,58 210,58 L260,58 C270,58 278,64 284,72 L300,96 L340,100 C350,102 354,110 354,118 L354,136 Z", "#5a6b86", LW)
    b += shape("M212,66 L258,66 C266,66 272,70 276,76 L288,96 L212,96 Z", "#9fd3ff", 4) + gloss(214, 66, 60, 30, 0.3)
    b += circ(80, 140, 22, "#2a2f3a", LW) + circ(80, 140, 9, "#9aa3b0", 3)
    b += circ(296, 140, 22, "#2a2f3a", LW) + circ(296, 140, 9, "#9aa3b0", 3)
    b += ell(348, 112, 6, 5, "#fff6a8", 3)
    b += rect(14, 104, 30, 14, "#ffffff", 2, 3) + text(29, 116, "YYC", 10, "#2a5fb8")
    b += shape("M210,60 Q232,46 262,58 Z", SNOW, 4)
    write("pickup", W, H, b, metres=5.6)


# ================================================================ PRAIRIE
def grain_elevator():
    """A red wooden prairie grain elevator with the name in white."""
    W, H = 260, 700
    b = poly([(40, 120), (210, 120), (210, H), (40, H)], "#c23a2f", LW)
    b += poly([(30, 124), (125, 40), (220, 124)], "#8a2a22", LW)
    b += line("M34,120 L125,36 L216,120", 10, SNOW) + line("M34,116 L125,32 L216,116", 3)
    b += poly([(90, 40), (90, 0), (160, 0), (160, 40), (125, 40)], "#c23a2f", 5)
    b += snowcap(88, 162, 2, 8, 3)
    for y in range(140, H, 20):
        b += line(f"M40,{y} L210,{y}", 1.5, "#8a2a22", 0.6)
    b += text(125, 200, "ALBERTA", 30, "#ffffff", length=150)
    b += text(125, 236, "POOL", 26, "#ffffff")
    # Annex and the driveway shed.
    b += poly([(210, 400), (260, 430), (260, H), (210, H)], "#a8322a", LW)
    b += rect(70, H - 120, 110, 120, "#7a2a22", 2, 4)
    b += line(f"M70,{H - 120} L180,{H} M180,{H - 120} L70,{H}", 4, "#c23a2f")
    b += magpie_shape(140, 2, 1.4)
    write("grain_elevator", W, H, b, metres=8.0)

def pumpjack():
    """An oil pumpjack in a Santa hat. Base and beam are separate sprites so
    the game can nod the beam about the bearing at the top of the A-frame."""
    W, H = 330, 240
    b = rect(10, 220, 310, 16, "#7f8a9c", 3, 4)
    b += poly([(140, 222), (176, 60), (212, 222)], "none", LW).replace('fill="none"', 'fill="none"')
    b += line("M140,222 L176,64 L212,222", 12) + line("M140,222 L176,64 L212,222", 6, "#e0b83a")
    b += line("M158,140 L194,140", 6, "#e0b83a")
    # Gearbox, crank and counterweight.
    b += rect(220, 160, 60, 60, "#5a6372", 6, LW) + circ(250, 170, 22, "#e0b83a", 5) + circ(250, 170, 6, INK, 0)
    # Wellhead.
    b += rect(20, 160, 24, 60, "#7f8a9c", 3, 4) + rect(14, 186, 36, 10, "#e6382f", 2, 3)
    b += circ(176, 64, 9, "#5a6372", 4)
    write("pumpjack_base", W, H, b, metres=11.0)
    W, H = 330, 120
    # Beam pivots at (176, 60) in this canvas, matching the base bearing.
    b = rect(40, 52, 230, 18, "#e0b83a", 4, LW)
    b += shape("M40,40 C20,40 10,60 14,90 L34,90 C36,70 40,64 48,64 Z", "#e0b83a", LW)
    b += line("M24,90 L24,120", 4)
    b += rect(250, 48, 36, 30, "#5a6372", 4, 5)
    # Santa hat on the horsehead.
    b += shape("M14,42 C18,10 44,0 64,12 C52,16 46,26 48,42 Z", "#e6382f", 5)
    b += rect(8, 36, 46, 12, "#ffffff", 6, 4) + circ(66, 12, 8, "#ffffff", 4)
    b += circ(176, 60, 7, INK, 0)
    write("pumpjack_beam", W, H, b, metres=11.0)

def cow():
    """A Hereford in a toque."""
    W, H = 240, 170
    b = ""
    for x in (60, 84, 170, 194):
        b += rect(x - 9, 110, 18, 54, "#8a3a20", 6, 5) + rect(x - 10, 150, 20, 14, "#3a2a20", 3, 4)
    b += shape("M40,70 C40,46 70,40 110,42 L170,42 C206,42 214,62 212,90 C210,116 196,126 170,126 L80,126 C54,126 40,110 40,90 Z", "#b0502a", LW)
    b += shape("M110,126 C116,108 140,104 150,126 Z", "#ffffff", 4)
    b += line("M212,70 C226,72 230,96 222,110", 6) + circ(222, 114, 6, "#3a2a20", 3)
    # Head, white face.
    b += ell(46, 70, 34, 30, "#b0502a", LW)
    b += shape("M28,58 C30,44 62,44 64,58 L62,92 C60,104 32,104 30,92 Z", "#ffffff", 4)
    b += ell(46, 96, 20, 12, "#ffc7b0", 4) + circ(40, 96, 2.5, INK, 0) + circ(52, 96, 2.5, INK, 0)
    b += line("M34,70 Q38,64 42,70 M50,70 Q54,64 58,70", 4)
    b += ell(14, 56, 14, 7, "#b0502a", 4, -20) + ell(78, 56, 14, 7, "#b0502a", 4, 20)
    # Toque.
    b += shape("M22,50 C22,22 70,22 70,50 Z", "#2a5fb8", LW) + rect(18, 44, 56, 12, "#ffffff", 6, 4) + circ(46, 22, 9, "#ffffff", 4)
    write("cow", W, H, b, metres=2.6)


# ================================================================ PEOPLE AND CRITTERS
CINK = "#4a2e1e"  # character ink, as in gen_chars.py
def person(name, coat, pants, hat, extra=None, skin=SKIN, shorts=False, arms_up=False, metres=1.25, mouth="smile",
           canvas=None, companion=None):
    """A Calgarian in the elves' chibi proportions, painted like the key
    art: big round head with shiny eyes and rosy cheeks, a puffy coat with
    quilted bands, boots. `hat` is a toque colour or "cowboy"; `extra(art)`
    paints a held item on top. `canvas=(W, dx)` widens the sprite and shifts
    the person right by dx, so `companion(art)` can paint a dog on a leash
    beside them."""
    art = Art()
    def paint():
        _paint_person(art, coat, pants, hat, extra, skin, shorts, arms_up, mouth)
    if canvas:
        W, dx = canvas
        art.group(paint, f"translate({dx} 0)")
        if companion:
            companion(art)
        metres = metres * W / 128
    else:
        W = 128
        paint()
    H = 210
    write(name, W, H, f'<g transform="translate(0 2)">{"".join(art.body)}</g>', "".join(art.defs), metres=metres, painted=True)

def _paint_person(art, coat, pants, hat, extra, skin, shorts, arms_up, mouth):
    BOOT = "#3a2a22"
    for x in (50, 78):
        if shorts:
            art.part(f"M{x - 9},160 L{x - 9},192 L{x + 9},192 L{x + 9},160 Z", skin, w=3.6)
        else:
            art.part(f"M{x - 11},156 L{x - 11},192 L{x + 11},192 L{x + 11},156 Z", pants, w=3.6)
        art.part(f"M{x - 14},190 L{x + 12},190 C{x + 22},190 {x + 24},206 {x + 14},206 L{x - 14},206 C{x - 18},206 {x - 18},190 {x - 14},190 Z", BOOT, w=3.6)
    if shorts:
        art.part("M38,146 L90,146 L92,170 L36,170 Z", pants, w=3.6)
    coat_d = "M30,100 C30,84 46,80 64,80 C82,80 98,84 98,100 L102,160 C90,168 38,168 26,160 Z"
    art.part(coat_d, coat, shades=[(ellipse(92, 140, 22, 40), None, 0.55, 7)], lights=[(ellipse(44, 104, 12, 18), None, 0.7, 5)], w=3.8)
    art.stroke("M64,84 L64,162", w=3)
    art.stroke("M32,122 Q64,130 96,122 M30,142 Q64,150 98,142", w=2.4, op=0.5)
    mitt = "#ffffff" if coat != "#ffffff" else BOOT
    for sx in (-1, 1):
        if arms_up:
            art.part(limb(64 + sx * 28, 98, 64 + sx * 46, 46, 20, 16), coat, w=3.6)
            art.part(circle(64 + sx * 47, 40, 10), mitt, w=3.4, rim=False)
        else:
            art.part(limb(64 + sx * 28, 98, 64 + sx * 40, 140, 20, 16), coat, w=3.6)
            art.part(circle(64 + sx * 40, 146, 9), BOOT, w=3.4, rim=False)
    art.part("M40,84 Q64,96 88,84 L88,96 Q64,108 40,96 Z", "#e6382f", w=3.4, rim=False)
    art.part(circle(64, 54, 31), skin, shades=[(ellipse(84, 70, 14, 22), None, 0.4, 6)], lights=[(ellipse(52, 36, 14, 8), None, 0.8, 4)], w=3.8)
    person_eyes(art, 52, 76, 54, 7)
    cheeks(art, 44, 84, 66, 7)
    if mouth == "sing":
        art.part(ellipse(64, 76, 7, 9), "#7a2a2a", w=2.6, rim=False)
        art.fill(ellipse(64, 81, 4, 3), "#ff8a8a")
    else:
        art.part("M54,70 Q64,82 74,70 Q64,74 54,70 Z", "#7a2a2a", w=2.8, rim=False)
    if hat == "cowboy":
        art.part("M14,38 C14,30 30,28 46,28 L82,28 C98,28 114,30 114,38 C114,48 90,52 64,52 C38,52 14,48 14,38 Z", "#f4f4f8",
                 shades=[(ellipse(64, 52, 50, 6), "#a9b6cc", 0.8, 3)], w=3.6)
        art.part("M40,32 C40,8 50,0 64,0 C78,0 88,8 88,32 Z", "#ffffff", shades=[(ellipse(84, 20, 10, 18), "#b9c6da", 0.7, 4)], w=3.6)
        art.fill("M40,26 L88,26 L88,33 L40,33 Z", "#2b1d2e")
    else:
        art.part("M34,48 C32,16 96,16 94,48 Z", hat, shades=[(ellipse(84, 40, 14, 14), None, 0.5, 5)], w=3.6)
        art.part("M32,42 Q64,34 96,42 L96,52 Q64,44 32,52 Z", "#ffffff", w=3.4, rim=False)
        art.part(circle(64, 16, 9), "#ffffff", shades=[(circle(68, 20, 6), "#b9c6da", 0.7, 2)], w=3.4, rim=False)
    if extra:
        extra(art)

def people():
    def coffee(art):
        art.part("M96,122 L112,122 L110,144 L98,144 Z", "#e6382f", w=3, rim=False)
        art.part("M94,118 L114,118 L114,124 L94,124 Z", "#ffffff", w=2.6, rim=False)
    def shovel(art):
        art.stroke("M100,50 L112,180", w=6)
        art.stroke("M100,50 L112,180", "#c9915a", 3)
        art.part("M98,176 L130,170 L134,206 L104,208 Z", "#3577e0", w=3.4)
    def skates(art):
        art.stroke("M34,208 L64,208 M64,208 L94,208", "#9aa3b0", 4)
    person("ped_hat", "#2a4a7a", "#3a3f4a", "cowboy", extra=coffee)
    person("ped_fan", "#d6283a", "#2a2f3a", "#d6283a", arms_up=True)
    person("ped_shorts", "#ff8a1a", "#2fa353", "#2fa353", extra=shovel, shorts=True)
    person("ped_kid", "#ff7fa5", "#ff7fa5", "#ffd23a", metres=0.95)
    person("ped_skater", "#2fa353", "#3a3f4a", "#e6382f", extra=skates)

    def bags(art):
        for (x, col, gift) in ((22, "#e6382f", True), (106, "#2f8f5a", False)):
            art.stroke(f"M{x - 6},132 Q{x},118 {x + 6},132", "#7a4a2a", 3)
            art.part(f"M{x - 14},132 L{x + 14},132 L{x + 16},170 L{x - 16},170 Z", col, w=3.4)
            if gift:
                art.part(f"M{x - 8},120 L{x + 8},120 L{x + 8},134 L{x - 8},134 Z", "#ffd23a", w=2.8, rim=False)
                art.stroke(f"M{x},120 L{x},134", "#e6382f", 3)
            else:
                art.part(ellipse(x + 4, 128, 6, 9), "#e9c27a", w=2.6, rim=False)  # a baguette poking out
    person("ped_granny", "#7a4a9a", "#3a3f4a", "#ff7fa5", extra=bags, metres=1.2)

    def foam_finger(art):
        # The Flames jersey: white hem stripes, the flaming C on the chest.
        art.stroke("M30,146 Q64,154 98,146", "#ffffff", 5)
        art.stroke("M29,154 Q64,162 99,154", "#ffd23a", 3)
        art.part(circle(64, 124, 12), "#ffffff", w=2.6, rim=False)
        art.stroke("M70,118 A7.5,7.5 0 1 0 70,130", "#d6283a", 4.5)
        art.fill("M66,112 Q72,106 70,100 Q76,106 74,114 Z", "#ffb02e")
        # A big yellow foam finger: we're number one.
        art.part("M96,52 L96,32 Q96,26 102,26 L104,26 L104,8 Q109,2 114,8 L114,26 L118,26 Q124,26 124,32 L124,52 Z", "#ffd23a",
                 lights=[(ellipse(104, 30, 6, 14), None, 0.6, 3)], w=3.4)
        art.stroke("M104,36 L116,36 M104,43 L116,43", "#c99a1e", 2)
    person("ped_jersey", "#d6283a", "#2a2f3a", "#ffd23a", extra=foam_finger, arms_up=True, mouth="sing")

    def stick(art):
        art.stroke("M98,60 L84,196", w=7)
        art.stroke("M98,60 L84,196", "#d9a45a", 4)
        art.part("M84,190 L110,190 Q116,190 116,198 L116,204 L80,204 Z", "#1b1b22", w=3)
        art.stroke("M86,150 L92,150", "#ffffff", 4)
    person("ped_hockey", "#2a5fb8", "#2a2f3a", "#e6382f", extra=stick, metres=0.95)

    def songbook(art):
        art.part("M44,118 L64,124 L84,118 L84,140 L64,146 L44,140 Z", "#c8433a", w=3.2)
        art.stroke("M64,124 L64,146", "#7a1b12", 2)
        art.part("M104,30 C104,22 112,22 112,28 L112,4 L118,6", "#3b2219", w=0, flat=True, rim=False)
        art.part(ellipse(102, 30, 6, 4.5), "#3b2219", w=0, flat=True, rim=False)
        art.stroke("M107,30 L107,6 Q114,8 118,14", w=3)
    person("ped_caroler", "#2f8f5a", "#3a3f4a", "#ffffff", extra=songbook, mouth="sing")

    def dachshund(art):
        """A wiener dog in a red Christmas sweater on a long leash."""
        COAT = "#a8602e"
        art.stroke("M104,146 Q140,186 180,170", "#e6382f", 3)
        for x in (180, 196, 236, 250):
            art.part(limb(x, 176, x + 1, 200, 10, 9), COAT if x in (196, 250) else "#8a4a22", w=3)
        art.part("M168,172 C168,156 186,152 210,152 L240,152 C258,152 266,162 264,176 C262,188 250,190 236,190 L190,190 C176,190 168,184 168,172 Z",
                 COAT, shades=[(ellipse(220, 194, 40, 8), None, 0.6, 5)], lights=[(ellipse(210, 156, 30, 5), None, 0.6, 4)], w=3.6)
        art.part("M196,152 L238,152 C246,152 250,160 250,170 L250,186 L192,186 L190,166 C190,158 192,152 196,152 Z", "#d6283a", w=3.2, rim=False)
        art.stroke("M196,164 L248,164 M196,176 L248,176", "#ffffff", 3)
        art.stroke("M262,166 Q276,150 274,140", w=5)
        art.stroke("M262,166 Q276,150 274,140", COAT, 3)
        art.part(ellipse(172, 160, 18, 15), COAT, lights=[(ellipse(168, 152, 8, 4), None, 0.7, 3)], w=3.6)
        art.part("M156,160 C150,162 146,168 150,172 C156,176 164,172 166,166 Z", COAT, w=3.2, rim=False)
        art.fill(circle(150, 166, 3.5), "#2a1a14")
        art.part("M176,150 C190,150 192,170 184,178 C178,180 174,170 176,150 Z", "#7a3a1a", w=3, rim=False)
        art.fill(circle(166, 156, 3), "#2a1a14")
        art.fill(circle(167, 155, 1), "#ffffff")
    person("ped_walker", "#2a9d8f", "#3a3f4a", "#ffd23a", canvas=(280, 0), companion=dachshund)

def inflatable_santa():
    W, H = 160, 240
    b = line("M20,230 L56,170 M140,230 L104,170", 2, "#9aa3b0")
    b += ell(80, 166, 62, 64, "#e6382f", LW, 0, CINK)
    b += rect(22, 160, 116, 14, "#1b1b22", 6, 4, CINK) + rect(70, 156, 20, 22, "#ffd23a", 3, 3, CINK)
    b += shape("M80,100 C50,100 40,130 52,150 C62,164 98,164 108,150 C120,130 110,100 80,100 Z", "#ffffff", 4, CINK)
    b += circ(80, 76, 38, "#ffe0c4", LW, CINK)
    b += line("M64,72 Q70,66 76,72 M84,72 Q90,66 96,72", 4, CINK) + circ(80, 84, 6, "#ff8a8a", 3, CINK)
    b += shape("M44,62 C44,20 110,4 128,40 C112,36 108,52 116,62 Z", "#e6382f", LW, CINK)
    b += rect(38, 54, 84, 16, "#ffffff", 8, 4, CINK) + circ(130, 40, 11, "#ffffff", 4, CINK)
    b += ell(30, 120, 14, 30, "#e6382f", 5, -30, CINK) + ell(130, 120, 14, 30, "#e6382f", 5, 30, CINK)
    b += ell(80, 232, 50, 6, "#000000", 0) .replace('fill="#000000"', 'fill="#000000" opacity="0.15"')
    b += gloss(40, 120, 30, 60, 0.3)
    write("inflatable_santa", W, H, b, metres=2.4)

def deer():
    """A mule deer, regular visitor of every Calgary cul-de-sac: big ears,
    white rump, a soft painted coat."""
    W, H = 220, 200
    art = Art()
    COAT, BELLY = "#b07d4c", "#ead6b8"
    for x, near in ((150, False), (60, False), (170, True), (80, True)):
        art.part(limb(x, 118, x + 2, 190, 16, 11), COAT if near else "#94643a", w=3.6)
        art.part(ellipse(x + 3, 192, 8, 5), "#3a2a22", w=3, rim=False)
    body = "M40,90 C40,70 70,64 110,66 L160,66 C186,66 196,82 194,104 C192,124 176,130 156,130 L70,130 C52,130 40,118 40,104 Z"
    art.part(body, COAT, shades=[(ellipse(120, 136, 80, 20), None, 0.6, 8)], lights=[(ellipse(110, 70, 50, 10), None, 0.7, 6)], w=4)
    art.fill(ellipse(116, 122, 46, 9), BELLY, 0.9)
    art.part(ellipse(188, 96, 13, 17), "#ffffff", w=3.6, rim=False)
    art.fill(ellipse(196, 112, 4, 6), "#1b1b22")
    art.part("M58,92 C52,60 50,40 56,30 L82,30 C84,50 80,74 78,96 Z", COAT, w=4)
    for d, rot in (("M30,14", -30), ("M78,14", 30)):
        cx = int(d[1:3])
        art.part(ellipse(cx, 14, 20, 10), COAT, w=3.6, extra=f'transform="rotate({rot} {cx} 14)"')
        art.fill(ellipse(cx, 14, 12, 5), "#e8b8a0", 0.8, f'transform="rotate({rot} {cx} 14)"')
    art.part(ellipse(54, 36, 26, 20), COAT, lights=[(ellipse(48, 26, 12, 6), None, 0.7, 3)], w=4)
    art.part(ellipse(32, 44, 13, 10), BELLY, w=3.4, rim=False)
    art.fill(circle(24, 42, 4.5), "#1b1b22")
    person_eyes(art, 50, 50, 30, 6)
    cheeks(art, 44, 44, 42, 5)
    write("deer", W, H, "".join(art.body), "".join(art.defs), metres=2.1, painted=True)

def magpie():
    W, H = 96, 64
    b = magpie_shape(56, 58, 1.6)
    write("magpie", W, H, b, metres=0.6)

def jackrabbit():
    """A white-tailed jackrabbit in its winter white."""
    W, H = 120, 122
    art = Art()
    WHITE = "#f7f8fc"
    for x, r in ((86, -10), (100, 12)):
        art.part(ellipse(x, 22, 7, 22), WHITE, w=3.4, extra=f'transform="rotate({r} {x} 22)"')
        art.fill(ellipse(x, 22, 3, 14), "#ffc2cc", 0.9, f'transform="rotate({r} {x} 22)"')
        art.fill(ellipse(x, 4, 4, 6), "#2a2a33", 1, f'transform="rotate({r} {x} 22)"')
    art.part("M24,96 C14,70 30,50 60,50 C86,50 100,66 98,88 C96,100 86,104 70,104 L34,104 C28,104 24,100 24,96 Z", WHITE,
             shades=[(ellipse(60, 108, 40, 12), None, 0.8, 6)], w=4)
    art.part(ellipse(92, 54, 19, 17), WHITE, shades=[(ellipse(98, 66, 12, 6), None, 0.6, 4)], w=4)
    person_eyes(art, 96, 96, 50, 5)
    art.fill(circle(110, 58, 3), "#ff9aa8")
    art.part(circle(22, 80, 10), "#ffffff", w=3.4, rim=False)
    write("jackrabbit", W, H, f'<g transform="translate(0 12)">{"".join(art.body)}</g>', "".join(art.defs), metres=0.7, painted=True)

def lit_tree():
    W, H = 200, 300
    b = bare_tree(100, 296, 280, lights_on=True)
    write("lit_tree", W, H, b, metres=5.0)


# ================================================================ DISTRICT GATES
def gate(name, title, tag, board="#2f6b4a", ink="#ffe9b0", tag_ink="#ffffff", w=340):
    """A neighbourhood sign at the start of a district, so the street
    begins somewhere instead of popping in: a painted board on posts, a
    wreath, lights and snow, with a line for the locals."""
    W, H = w, 250
    b = ""
    for x in (40, W - 40):
        b += rect(x - 9, 100, 18, H - 104, "#5a4030", 3, 4) + rect(x - 14, H - 14, 28, 12, "#7f8a9c", 3, 3)
    b += rect(10, 30, W - 20, 104, board, 12, LW)
    b += rect(22, 42, W - 44, 80, "none", 8, 3, lighter(board, 0.35))
    b += text(W / 2, 88, title, 34 if len(title) < 13 else 28, ink, stroke=darker(board, 0.6), sw=3, length=min(W - 70, 22 * len(title)))
    b += text(W / 2, 114, tag, 15, tag_ink, length=min(W - 70, 9.2 * len(tag)))
    b += snowcap(6, W - 6, 30, 14)
    b += lights(20, W - 20, 136, 8, 22, 4)
    b += wreath(W / 2, 22, 14)
    write(name, W, H, b, metres=W / 340 * 4.6)

def hwy_sign():
    """The green Trans-Canada guide sign heading out west."""
    W, H = 360, 300
    b = ""
    for x in (60, W - 60):
        b += rect(x - 8, 150, 16, H - 154, "#9aa3b0", 2, 4)
    b += rect(10, 20, W - 20, 150, "#1f7a4a", 10, LW)
    b += rect(20, 30, W - 40, 130, "none", 6, 4, "#ffffff")
    b += poly([(42, 46), (78, 46), (82, 70), (60, 86), (38, 70)], "#ffffff", 3) + text(60, 72, "1", 22, "#1f7a4a")
    b += text(100, 68, "WEST", 22, "#ffffff", anchor="start")
    b += text(40, 112, "Banff", 26, "#ffffff", anchor="start") + text(W - 40, 112, "128", 26, "#ffffff", anchor="end")
    b += text(40, 146, "Vancouver", 26, "#ffffff", anchor="start") + text(W - 40, 146, "970", 26, "#ffffff", anchor="end")
    b += snowcap(6, W - 6, 20, 14)
    write("hwy_sign", W, H, b, metres=5.0)


# ================================================================ SKYLINE (two depths)
def skyline():
    """Downtown Calgary on the horizon in two painted depths (tools/city_skyline.py):
    the landmark rank (Calgary Tower with its torch lit, Bankers Hall, Telus Sky,
    the Bow, Brookfield Place, Suncor, Eighth Avenue Place) and a hazy far rank."""
    import city_skyline
    W, H, body, defs = city_skyline.near()
    write("skyline", W, H, body, defs, metres=80.0, scale=1.0, painted=True)
    W, H, body, defs = city_skyline.far()
    write("skyline_far", W, H, body, defs, metres=80.0, scale=1.0, painted=True)


# ================================================================ run
for fn in (ingle_a, ingle_b, ingle_c, ingle_d, sand_a, sand_b, tower_glass, tower_concrete, city_hall, wonderland,
           ctrain, plus15, stephen_lamp, temp_sign, peace_bridge, plaza, kens_a, kens_b, kens_c, chalkboard,
           red_a, red_b, red_c, saddledome, cone, roadwork, comm_sign, blue_ring, drive_in, pickup, grain_elevator,
           pumpjack, cow, people, inflatable_santa, deer, magpie, jackrabbit, lit_tree, skyline):
    fn()
gate("gate_ingle", "INGLEWOOD", "CALGARY'S OLDEST · EST. 1875", "#7a2c22")
gate("gate_stephen", "STEPHEN AVE", "PEDESTRIAN MALL · NO FLYING DOGS", "#2a4a7a")
gate("gate_bow", "THE BOW", "ICE THICKNESS: PROBABLY FINE", "#3a6f9a")
gate("gate_kens", "KENSINGTON", "A COFFEE EVERY 10 METRES", "#2a7a6f")
gate("gate_red", "THE RED MILE", "17 AV SW · HONK IF THE FLAMES WIN", "#b8262c", w=380)
hwy_sign()
house("house_a", "#e3d3b3", "#c8433a")
house("house_b", "#c9cdd3", "#2f8f63", flip=True)
house("house_c", "#d9c7b0", "#2a5fb8")
car("car_red", "#d6283a")
car("car_blue", "#2a6fd6", plug=False)
car("car_taxi", "#ffc22a", plug=False)
car("car_white", "#eef1f6", plug=False)

# ------------------------------------------------------------------ layout
# Districts from the launch building westward. Facades repeat in a seeded
# order inside each district; landmarks sit at fixed distances; props are
# scattered in front of the facades with weights. All x are metres.
def fac(*names):
    return [dict({"sprite": n, "w": SIZES[n][0]}, **({"smoke": list(SMOKE[n])} if n in SMOKE else {})) for n in names]
def prop(n, weight=1.0, w=None):
    # Calgarians are drawn a size up so they read next to the flying dog.
    k = 1.25 if n.startswith("ped_") else 1.0
    return {"sprite": n, "w": w if w is not None else round(SIZES[n][0] * k, 2), "weight": weight}
def mark(n, x, w=None, lift=0.0):
    return {"sprite": n, "x": x, "w": w if w is not None else SIZES[n][0], "lift": lift}

city = {
    "skyline": {"sprite": "skyline", "x": 260.0, "factor": 0.08, "height": 420.0},
    "traffic": [{"sprite": n, "w": SIZES[n][0]} for n in ("car_blue", "car_taxi", "car_white", "pickup", "car_blue", "car_taxi")],
    "skyline_far": {"sprite": "skyline_far", "x": 240.0, "factor": 0.05, "height": 300.0},
    "districts": [
        {"name": "INGLEWOOD", "landed": "in Inglewood, 9 Av SE", "street": "9 Av SE", "from": 22, "to": 190, "ground": "street",
         "facades": fac("bld_ingle_a", "bld_ingle_b", "bld_ingle_c", "bld_ingle_d"),
         "props": [prop("lit_tree", 1.0), prop("ped_hat", 1.0), prop("ped_kid", 0.6), prop("ped_walker", 1.0), prop("ped_granny", 0.8), prop("ped_caroler", 0.6), prop("car_red", 0.8), prop("magpie", 0.8), prop("jackrabbit", 0.4)],
         "landmarks": [mark("gate_ingle", 23)]},
        {"name": "STEPHEN AVENUE", "landed": "on Stephen Avenue, downtown", "street": "Downtown", "from": 190, "to": 400, "ground": "street",
         "facades": fac("bld_sand_a", "bld_tower_glass", "bld_sand_b", "bld_tower_concrete"),
         "props": [prop("stephen_lamp", 2.0), prop("ped_hat", 1.2), prop("ped_fan", 0.4), prop("ped_caroler", 1.0), prop("ped_granny", 0.8), prop("ped_walker", 0.6), prop("lit_tree", 0.8), prop("magpie", 0.5)],
         "landmarks": [mark("gate_stephen", 191), mark("bld_city_hall", 198), mark("wonderland", 300), mark("temp_sign", 330)],
         "train": {"sprite": "ctrain", "w": SIZES["ctrain"][0], "speed": 7.0},
         "skywalk": {"sprite": "plus15", "lift": 4.6}},
        {"name": "THE BOW", "landed": "on the frozen Bow", "street": "Peace Bridge", "from": 400, "to": 480, "ground": "river",
         "facades": [],
         "props": [prop("ped_skater", 1.4), prop("ped_hockey", 1.0), prop("ped_walker", 0.4), prop("magpie", 0.3)],
         "landmarks": [mark("gate_bow", 401), mark("peace_bridge", 412, lift=1.2)]},
        {"name": "KENSINGTON", "landed": "in Kensington", "street": "Kensington Rd NW", "from": 480, "to": 650, "ground": "street",
         "facades": fac("bld_kens_a", "bld_plaza", "bld_kens_c", "bld_kens_b"),
         "props": [prop("lit_tree", 1.2), prop("chalkboard", 0.6), prop("ped_hat", 0.8), prop("ped_kid", 0.6), prop("ped_walker", 1.0), prop("ped_caroler", 0.6), prop("ped_granny", 0.5), prop("car_blue", 0.6), prop("magpie", 0.6)],
         "landmarks": [mark("gate_kens", 481)]},
        {"name": "THE RED MILE", "landed": "on the Red Mile, 17 Av SW", "street": "17 Av SW", "from": 650, "to": 850, "ground": "street",
         "facades": fac("bld_red_a", "bld_red_b", "bld_red_c"),
         "props": [prop("ped_fan", 1.6), prop("ped_jersey", 1.6), prop("cone", 1.2), prop("roadwork", 0.4), prop("car_red", 0.5), prop("magpie", 0.4)],
         "landmarks": [mark("gate_red", 651), mark("saddledome", 760)]},
        {"name": "THE SUBURBS", "landed": "in Sage Ridge Park Hills, phase 47", "street": "Sage Ridge Park Hills", "from": 850, "to": 1100, "ground": "street", "gap": [2.5, 4.0],
         "facades": fac("house_a", "house_b", "house_c"),
         "props": [prop("inflatable_santa", 1.0), prop("ped_shorts", 1.0), prop("ped_hockey", 0.8), prop("ped_walker", 0.5), prop("deer", 0.6), prop("pickup", 0.6), prop("car_blue", 0.5), prop("jackrabbit", 0.5)],
         "landmarks": [mark("comm_sign", 856), mark("drive_in", 1010), mark("blue_ring", 1070)]},
        {"name": "THE PRAIRIE", "landed": "out on the prairie, Hwy 1", "street": "Hwy 1 West", "from": 1100, "to": 100000, "ground": "snow",
         "facades": [],
         "props": [prop("cow", 1.0), prop("magpie", 0.4), prop("jackrabbit", 0.4)],
         "landmarks": [mark("hwy_sign", 1102), mark("grain_elevator", 1160), mark("grain_elevator", 1700)],
         "pumpjacks": {"base": "pumpjack_base", "beam": "pumpjack_beam", "w": SIZES["pumpjack_base"][0], "every": 140, "from": 1230}},
    ],
}
json.dump(city, open(os.path.join(ROOT, "packs", PACK, "city.json"), "w"), indent=1)
print(f"city: {len(SIZES)} sprites, {len(city['districts'])} districts")
