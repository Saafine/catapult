#!/usr/bin/env python3
"""Painted pickups and props, after the designers' key art: every surface is
lit from the top left with a gradient, outlined in a deeper shade of its own
colour, with gloss and foil highlights painted on top. Each file carries
data-paint="done", so tools/paint.py only adds the grain.

Called from the end of tools/gen_props.py:
presents (five designs, one of them the rare gold one), the firework rocket
and its flame, the storm cloud and its lightning bolt, the Christmas tree,
the lamppost, the candy cane and the gift pile."""
import math, os, random
from paint import light, shadow, edge, mix

PACK = os.environ.get("PACK", "calgary_xmas")
OUT = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)

def write(name, w, h, body, defs="", vb=None, scale=None):
    vb = vb or (w, h)
    sc = f' data-scale="{scale}"' if scale else ""
    open(os.path.join(OUT, name + ".svg"), "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {vb[0]} {vb[1]}" data-paint="done"{sc}>\n'
        f'<defs>{defs}</defs>\n{body}\n</svg>')

def lg(gid, stops, x1=0, y1=0, x2=0, y2=1, units=""):
    s = "".join(f'<stop offset="{o}" stop-color="{c}"/>' if len(st) == 2 else f'<stop offset="{o}" stop-color="{c}" stop-opacity="{st[2]}"/>'
                for st in stops for (o, c) in [st[:2]])
    u = f' gradientUnits="{units}"' if units else ""
    return f'<linearGradient id="{gid}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}"{u}>{s}</linearGradient>'

def rg(gid, stops, cx=0.5, cy=0.5, r=0.5, fx=None, fy=None):
    s = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{a}"/>' for o, c, a in stops)
    f = f' fx="{fx}" fy="{fy}"' if fx is not None else ""
    return f'<radialGradient id="{gid}" cx="{cx}" cy="{cy}" r="{r}"{f}>{s}</radialGradient>'

def lit(gid, c, x2=0, y2=1):
    return lg(gid, [(0, light(c)), (0.45, c), (1, shadow(c))], x2=x2, y2=y2)

def P(d, fill, stroke=None, w=0, extra=""):
    s = f' stroke="{stroke}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"' if stroke else ""
    return f'<path d="{d}" fill="{fill}"{s} {extra}/>'

def C(cx, cy, r, fill, stroke=None, w=0, extra=""):
    s = f' stroke="{stroke}" stroke-width="{w}"' if stroke else ""
    return f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{r:.1f}" fill="{fill}"{s} {extra}/>'

def E(cx, cy, rx, ry, fill, rot=0, extra=""):
    return f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="{rx:.1f}" ry="{ry:.1f}" fill="{fill}" transform="rotate({rot} {cx:.1f} {cy:.1f})" {extra}/>'

def poly(pts):
    return "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts) + " Z"

def star_d(cx, cy, r1, r2, n=5, rot=-90):
    pts = []
    for i in range(n * 2):
        r = r1 if i % 2 == 0 else r2
        a = math.radians(rot + i * 180 / n)
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return poly(pts)

BLUR = lambda gid, sd: f'<filter id="{gid}" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="{sd}"/></filter>'

# ---------------------------------------------------------------- presents
def present(name, box, rib, pattern=None, pat_col="#ffffff", W=110, H=70, D=34, lid=20, gold=False):
    """A wrapped box in three-quarter view (front, right side and top lit
    differently), a lid that overhangs, a satin ribbon crossing every face
    and a big bow on top with sheen. Authored on a 192 unit grid."""
    defs = ""
    k = 19 / 34  # how steeply the side recedes
    dx, dy = D, -D * k
    x0 = (192 - W - D) / 2 + 2
    yb = 182                       # bottom of the front face
    yt = yb - H                    # top of the box front (under the lid)
    ov = 6                         # lid overhang
    lx0, lx1, ly0, ly1 = x0 - ov, x0 + W + ov, yt - lid + 6, yt + 6
    BE, RE = edge(box), edge(rib)
    if gold:
        BE = "#9a5a12"
    # Gradients: front lit top-left, side in shade, top brightest.
    if gold:
        defs += lg(f"{name}f", [(0, "#fff3b0"), (0.25, "#ffd34a"), (0.5, "#f2a91c"), (0.62, "#ffe27a"), (1, "#c97a12")], x2=1, y2=1)
        defs += lg(f"{name}s", [(0, "#e9a523"), (0.5, "#b8720f"), (1, "#8f520a")])
        defs += lg(f"{name}t", [(0, "#fffbe0"), (1, "#ffd85a")], x2=1, y2=0)
    else:
        defs += lg(f"{name}f", [(0, light(box)), (0.4, box), (1, shadow(box))])
        defs += lg(f"{name}s", [(0, shadow(box)), (1, mix(shadow(box), "#2a1f4a", 0.25))])
        defs += lg(f"{name}t", [(0, mix(light(box), "#ffffff", 0.25)), (1, light(box))], x2=1, y2=0)
    defs += lg(f"{name}r", [(0, mix(light(rib), "#ffffff", 0.35)), (0.5, rib), (1, shadow(rib))])
    defs += lg(f"{name}rs", [(0, shadow(rib)), (1, mix(shadow(rib), "#2a1f4a", 0.2))])
    defs += lg(f"{name}rv", [(0, shadow(rib)), (0.35, light(rib)), (0.5, mix(light(rib), "#ffffff", 0.5)), (0.65, rib), (1, shadow(rib))], x2=1, y2=0)
    defs += lg(f"{name}gl", [(0, "#ffffff", 0.75), (1, "#ffffff", 0)], x2=1, y2=1)
    # Pattern tiles (dots / stars / stripes) drawn in face-local units.
    pat = ""
    if pattern == "dots":
        pat = f'<pattern id="{name}p" width="26" height="26" patternUnits="userSpaceOnUse"><circle cx="7" cy="7" r="5" fill="{pat_col}"/><circle cx="20" cy="20" r="5" fill="{pat_col}"/></pattern>'
    elif pattern == "stars":
        pat = f'<pattern id="{name}p" width="34" height="30" patternUnits="userSpaceOnUse"><path d="{star_d(9, 9, 7, 3)}" fill="{pat_col}"/><path d="{star_d(26, 24, 5, 2.2)}" fill="{pat_col}"/></pattern>'
    elif pattern == "stripes":
        pat = f'<pattern id="{name}p" width="26" height="26" patternUnits="userSpaceOnUse" patternTransform="rotate(35)"><rect x="0" y="0" width="11" height="26" fill="{pat_col}"/></pattern>'
    elif pattern == "flakes":
        f = ""
        for a in range(0, 180, 60):
            f += f'<path d="M{10 - 7 * math.cos(math.radians(a)):.1f},{10 - 7 * math.sin(math.radians(a)):.1f} L{10 + 7 * math.cos(math.radians(a)):.1f},{10 + 7 * math.sin(math.radians(a)):.1f}" stroke="{pat_col}" stroke-width="2.2" stroke-linecap="round"/>'
        pat = f'<pattern id="{name}p" width="30" height="30" patternUnits="userSpaceOnUse"><g>{f}</g><circle cx="25" cy="25" r="2.5" fill="{pat_col}"/></pattern>'
    defs += pat
    b = ""
    # Soft contact shadow so the box sits on something even in mid-air.
    b += E(x0 + W / 2 + D / 2, yb + 2, W / 2 + D / 2 + 4, 7, "#1a2350", extra='opacity="0.18"')

    # ---- box body: side face, front face
    side = poly([(x0 + W, yt), (x0 + W + dx, yt + dy), (x0 + W + dx, yb + dy), (x0 + W, yb)])
    front = f"M{x0},{yt} L{x0 + W},{yt} L{x0 + W},{yb - 6} Q{x0 + W},{yb} {x0 + W - 6},{yb} L{x0 + 6},{yb} Q{x0},{yb} {x0},{yb - 6} Z"
    b += P(side, f"url(#{name}s)")
    b += P(front, f"url(#{name}f)")
    if pattern:
        b += P(front, f"url(#{name}p)", extra='opacity="0.85"')
        b += f'<g transform="matrix(1,{-k:.4f},0,1,{x0 + W},{yt})"><rect x="0" y="0" width="{dx}" height="{H}" fill="url(#{name}p)" opacity="0.55"/></g>'
    # Bottom shading on the front, a cool bounce light on its lower edge.
    b += P(f"M{x0},{yb - 18} L{x0 + W},{yb - 18} L{x0 + W},{yb - 6} Q{x0 + W},{yb} {x0 + W - 6},{yb} L{x0 + 6},{yb} Q{x0},{yb} {x0},{yb - 6} Z", "#1a1450", extra='opacity="0.16"')
    # Ribbon: vertical band on the front, and the band around the side.
    rw = 22
    rx = x0 + W / 2 - rw / 2
    b += P(f"M{rx},{yt} L{rx + rw},{yt} L{rx + rw},{yb} L{rx},{yb} Z", f"url(#{name}rv)")
    sm = 0.5  # side band position along the depth
    sx0, sx1 = x0 + W + dx * (sm - 0.17), x0 + W + dx * (sm + 0.17)
    b += P(poly([(sx0, yt + dy * (sm - 0.17)), (sx1, yt + dy * (sm + 0.17)), (sx1, yb + dy * (sm + 0.17)), (sx0, yb + dy * (sm - 0.17))]), f"url(#{name}rs)")
    # Outlines of the body.
    b += P(side, "none", BE, 4.5)
    b += P(front, "none", BE, 4.5)
    b += P(f"M{rx},{yt} L{rx},{yb} M{rx + rw},{yt} L{rx + rw},{yb}", "none", RE, 2.5)
    # Foil gloss: a diagonal streak across the front face.
    b += P(f"M{x0 + 8},{yt + 10} L{x0 + 30},{yt + 10} L{x0 + 12},{yb - 10} L{x0 + 6},{yb - 18} Z", f"url(#{name}gl)", extra='opacity="0.7"')
    b += P(f"M{x0 + 36},{yt + 10} L{x0 + 42},{yt + 10} L{x0 + 26},{yb - 22} L{x0 + 22},{yb - 26} Z", "#ffffff", extra='opacity="0.35"')

    # ---- lid: front band, side band, top
    lside = poly([(lx1, ly0), (lx1 + dx, ly0 + dy), (lx1 + dx, ly1 + dy), (lx1, ly1)])
    lfront = f"M{lx0},{ly0} L{lx1},{ly0} L{lx1},{ly1 - 3} Q{lx1},{ly1} {lx1 - 3},{ly1} L{lx0 + 3},{ly1} Q{lx0},{ly1} {lx0},{ly1 - 3} Z"
    ltop = poly([(lx0, ly0), (lx0 + dx, ly0 + dy), (lx1 + dx, ly0 + dy), (lx1, ly0)])
    b += P(f"M{lx0 + 2},{ly1} L{lx1 + 2},{ly1} L{lx1 + dx},{ly1 + dy + 4} L{lx1 + dx},{ly1 + dy + 8} L{lx1},{ly1 + 8} L{lx0 + 2},{ly1 + 8} Z", "#1a1450", extra='opacity="0.22"')
    b += P(lside, f"url(#{name}s)")
    b += P(lfront, f"url(#{name}f)")
    b += P(ltop, f"url(#{name}t)")
    if pattern:
        b += P(lfront, f"url(#{name}p)", extra='opacity="0.85"')
        b += f'<g transform="matrix(1,0,{dx / D:.3f},{dy / D:.4f},{lx0},{ly0})"><rect x="0" y="0" width="{lx1 - lx0}" height="{D}" fill="url(#{name}p)" opacity="0.6"/></g>'
    # Ribbon over the lid: front band, along the top (front to back and
    # left to right), and the side band.
    b += P(f"M{rx},{ly0} L{rx + rw},{ly0} L{rx + rw},{ly1} L{rx},{ly1} Z", f"url(#{name}rv)")
    b += P(poly([(rx, ly0), (rx + dx, ly0 + dy), (rx + rw + dx, ly0 + dy), (rx + rw, ly0)]), f"url(#{name}r)")
    tm0, tm1 = sm - 0.17, sm + 0.17
    b += P(poly([(lx0 + dx * tm0, ly0 + dy * tm0), (lx1 + dx * tm0, ly0 + dy * tm0), (lx1 + dx * tm1, ly0 + dy * tm1), (lx0 + dx * tm1, ly0 + dy * tm1)]), f"url(#{name}r)")
    sx0, sx1 = lx1 + dx * tm0, lx1 + dx * tm1
    b += P(poly([(sx0, ly0 + dy * tm0), (sx1, ly0 + dy * tm1), (sx1, ly1 + dy * tm1), (sx0, ly1 + dy * tm0)]), f"url(#{name}rs)")
    b += P(lside, "none", BE, 4.5)
    b += P(ltop, "none", BE, 4.5)
    b += P(lfront, "none", BE, 4.5)
    b += P(f"M{lx0 + 4},{ly0 + 3} L{lx1 - 2},{ly0 + 3}", "none", "#ffffff", 2.5, 'opacity="0.6"')

    # ---- bow: two satin loops, ribbon tails, a knot; sheen on every loop.
    cx, cy = rx + rw / 2 + dx * 0.5, ly0 + dy * 0.5 + 2
    s = 1.22 + (W - 110) / 300
    def loop(sg):
        return (f"M{cx},{cy} C{cx + sg * 14 * s},{cy - 40 * s} {cx + sg * 60 * s},{cy - 50 * s} {cx + sg * 58 * s},{cy - 20 * s} "
                f"C{cx + sg * 56 * s},{cy - 2 * s} {cx + sg * 30 * s},{cy + 4 * s} {cx},{cy} Z")
    def hole(sg):
        return (f"M{cx + sg * 8 * s},{cy - 6 * s} C{cx + sg * 18 * s},{cy - 30 * s} {cx + sg * 46 * s},{cy - 36 * s} {cx + sg * 46 * s},{cy - 20 * s} "
                f"C{cx + sg * 44 * s},{cy - 8 * s} {cx + sg * 26 * s},{cy - 4 * s} {cx + sg * 8 * s},{cy - 6 * s} Z")
    def tail(sg, ln):
        ex, ey = cx + sg * 30 * s, cy + ln * s
        return (f"M{cx - sg * 4},{cy} C{cx + sg * 8 * s},{cy + 14 * s} {ex - sg * 6},{ey - 16 * s} {ex - sg * 10 * s},{ey} "
                f"L{ex - sg * 1 * s},{ey - 7 * s} L{ex + sg * 6 * s},{ey + 2 * s} C{ex + sg * 4 * s},{ey - 18 * s} {cx + sg * 22 * s},{cy + 10 * s} {cx + sg * 8},{cy - 2} Z")
    b += P(tail(-1, 36), f"url(#{name}r)", RE, 3.5)
    b += P(tail(1, 30), f"url(#{name}rs)", RE, 3.5)
    for sg in (-1, 1):
        b += P(loop(sg), f"url(#{name}r)", RE, 4)
        b += P(hole(sg), shadow(shadow(rib)), extra='opacity="0.75"')
        # sheen along the top of the loop
        b += P(f"M{cx + sg * 10 * s},{cy - 16 * s} C{cx + sg * 20 * s},{cy - 38 * s} {cx + sg * 44 * s},{cy - 46 * s} {cx + sg * 52 * s},{cy - 34 * s}",
               "none", "#ffffff", 4 * s, 'opacity="0.75"')
    b += E(cx, cy - 2, 12 * s, 10 * s, f"url(#{name}r)", extra=f'stroke="{RE}" stroke-width="4"')
    b += E(cx - 3, cy - 6, 5 * s, 3 * s, "#ffffff", -20, 'opacity="0.8"')
    if gold:
        # Embossed sparkle glints on the foil.
        for (gx, gy, r) in ((x0 + 18, yt + 22, 9), (x0 + W - 14, yb - 22, 7), (lx1 + dx - 8, ly0 + dy + 6, 6)):
            b += P(star_d(gx, gy, r, r * 0.22, 4, -90), "#ffffff", extra='opacity="0.95"')
    write(name, 96, 96, b, defs, vb=(192, 192), scale=3)

# ---------------------------------------------------------------- rocket
def firework():
    """A chunky toy rocket in painted metal: red lacquered body with chrome
    bands and rivets, a porthole, a gold nose cone, three swept blue fins
    and a steel nozzle at the tail (the flame is the rocket_flame sprite).
    Points right; the nozzle mouth sits at x 40, y 60 of 260x120."""
    defs = ""
    RED, BLUE = "#e33b2e", "#2f6fd6"
    defs += lg("rb", [(0, "#8a1a14"), (0.12, "#e65a48"), (0.28, "#ffb3a0"), (0.36, "#ef5040"), (0.7, RED), (1, "#7d1612")])
    defs += lg("ch", [(0, "#5d6b7a"), (0.15, "#eef4fa"), (0.3, "#ffffff"), (0.45, "#a9b6c4"), (0.75, "#6c7a8a"), (1, "#3d4856")])
    defs += lg("nz", [(0, "#3a4250"), (0.2, "#c6d0da"), (0.35, "#f4f8fc"), (0.6, "#7d8a99"), (1, "#2e3542")])
    defs += lg("gd", [(0, "#a8620e"), (0.18, "#ffd24a"), (0.32, "#fff6c8"), (0.5, "#f5b52a"), (0.8, "#c77d12"), (1, "#8a4c08")])
    defs += lg("fn", [(0, light(BLUE)), (0.5, BLUE), (1, shadow(BLUE))])
    defs += lg("fn2", [(0, shadow(BLUE)), (1, mix(shadow(BLUE), "#101830", 0.4))])
    defs += rg("gl", [(0, "#e8f7ff", 1), (0.5, "#7ec8f5", 1), (1, "#2a6aa8", 1)], cx=0.38, cy=0.35, r=0.7)
    b = ""
    CY, R = 60, 27
    # Back fin (seen edge-on, behind the body).
    b += P("M84,52 L54,26 L44,26 L60,56 Z", "url(#fn2)", "#1b3c80", 3)
    # Nozzle bell at the tail.
    b += P(f"M76,{CY - 18} L76,{CY + 18} L46,{CY + 24} Q38,{CY} 46,{CY - 24} Z", "url(#nz)", "#2a3340", 4)
    b += E(45, CY, 6, 23, "#1a1f28")
    b += E(45, CY, 3.5, 17, "#ff9a3a", extra='opacity="0.9"')
    # Body: a cylinder with a lacquer highlight.
    body = f"M74,{CY - R} L196,{CY - R} L196,{CY + R} L74,{CY + R} Q66,{CY} 74,{CY - R} Z"
    b += P(body, "url(#rb)")
    b += P(f"M90,{CY - R + 6} L190,{CY - R + 6}", "none", "#ffffff", 4, 'opacity="0.55"')
    # Gold star decal.
    b += P(star_d(124, CY + 2, 15, 6.5), "url(#gd)", "#8a4c08", 2.5)
    # Chrome bands with rivets.
    for x in (84, 168):
        b += f'<rect x="{x}" y="{CY - R - 1}" width="12" height="{2 * R + 2}" rx="3" fill="url(#ch)" stroke="#3d4856" stroke-width="3"/>'
        for yy in (-17, -6, 5, 16):
            b += C(x + 6, CY + yy, 1.8, "#ffffff" if yy < 0 else "#55606e")
    # Porthole.
    b += C(148, CY - 2, 14, "url(#ch)", "#3d4856", 3)
    b += C(148, CY - 2, 9.5, "url(#gl)", "#1c4a7a", 2)
    b += E(144, CY - 6, 4, 2.5, "#ffffff", -30)
    b += P(body, "none", "#7d1612", 4.5)
    # Nose cone: gold ogive with a bright streak and a tip.
    nose = f"M194,{CY - R - 1} C222,{CY - R} 246,{CY - 12} 256,{CY} C246,{CY + 12} 222,{CY + R} 194,{CY + R + 1} Z"
    b += P(nose, "url(#gd)", "#8a4c08", 4.5)
    b += P(f"M202,{CY - R + 6} C222,{CY - R + 6} 238,{CY - 14} 248,{CY - 4}", "none", "#fffbe0", 3.5, 'opacity="0.8"')
    b += f'<rect x="190" y="{CY - R - 3}" width="10" height="{2 * R + 6}" rx="3" fill="url(#ch)" stroke="#3d4856" stroke-width="3"/>'
    # Top and bottom fins, swept back past the tail.
    top = f"M80,{CY - R + 2} L120,{CY - R + 2} C108,{CY - R - 10} 84,{CY - R - 26} 52,{CY - R - 30} C58,{CY - R - 18} 64,{CY - R - 6} 80,{CY - R + 2} Z"
    bot = f"M80,{CY + R - 2} L120,{CY + R - 2} C108,{CY + R + 10} 84,{CY + R + 26} 52,{CY + R + 30} C58,{CY + R + 18} 64,{CY + R + 6} 80,{CY + R - 2} Z"
    b += P(top, "url(#fn)", "#1b3c80", 4)
    b += P(bot, "url(#fn2)", "#1b3c80", 4)
    b += P(f"M62,{CY - R - 24} C80,{CY - R - 18} 98,{CY - R - 8} 110,{CY - R}", "none", "#bfe0ff", 3, 'opacity="0.8"')
    write("firework", 260, 120, b, defs, scale=3)

def rocket_flame():
    """Exhaust flame, pointing left from its right edge (the nozzle)."""
    defs = BLUR("fb", 4)
    defs += lg("fo", [(0, "#ff3d1a", 0), (0.35, "#ff5a1f"), (1, "#ffb02e")], x2=1, y2=0)
    defs += lg("fm", [(0, "#ffd23a", 0), (0.4, "#ffd23a"), (1, "#fff3b0")], x2=1, y2=0)
    b = ""
    b += P("M128,10 C96,8 50,18 4,32 C50,46 96,56 128,54 Z", "#ff7a2a", extra='filter="url(#fb)" opacity="0.8"')
    b += P("M128,14 C100,12 60,22 14,32 C60,42 100,52 128,50 Z", "url(#fo)")
    b += P("M128,20 C108,19 80,26 46,32 C80,38 108,45 128,44 Z", "url(#fm)")
    b += P("M128,25 C116,25 102,29 84,32 C102,35 116,39 128,39 Z", "#ffffff")
    write("rocket_flame", 128, 64, b, defs, scale=2)

# ---------------------------------------------------------------- storm
def stormcloud():
    """A big grumpy thunderhead: stacked billows lit lavender on top and
    bruised slate underneath, an electric glow leaking between them, a
    furrowed brow over glowing eyes and a gritted scowl."""
    W, H = 224, 144
    defs = ""
    defs += rg("bb", [(0, "#9294b8", 1), (0.55, "#5a5d82", 1), (1, "#33365a", 1)], cx=0.4, cy=0.3, r=0.75, fx=0.35, fy=0.22)
    defs += rg("bf", [(0, "#c3c4dc", 1), (0.5, "#7d80a3", 1), (1, "#474a6d", 1)], cx=0.4, cy=0.3, r=0.75, fx=0.35, fy=0.2)
    defs += lg("belly", [(0, "#4a4d72"), (1, "#2a2c48")])
    defs += rg("zap", [(0, "#fff7b0", 1), (0.4, "#ffd23a", 0.8), (1, "#ffd23a", 0)])
    defs += rg("eye", [(0, "#ffffff", 1), (0.45, "#fff27a", 1), (1, "#ffb21a", 1)])
    defs += BLUR("cb", 3)
    b = ""
    back = [(52, 70, 30), (92, 52, 38), (140, 50, 36), (178, 68, 30), (118, 40, 30)]
    front = [(36, 96, 28), (74, 88, 34), (116, 84, 38), (158, 88, 34), (192, 98, 26)]
    # Outline halo (deeper shade of the cloud).
    for (x, y, r) in back + front:
        b += C(x, y, r + 3.5, "#2a2c48")
    belly = "M18,98 Q12,124 36,124 Q52,134 72,125 Q92,136 114,126 Q136,136 156,126 Q176,134 192,124 Q212,124 206,98 Z"
    b += P(belly, "#2a2c48", "#2a2c48", 7)
    for (x, y, r) in back:
        b += C(x, y, r, "url(#bb)")
    # Electric glow leaking out between the billows.
    b += E(116, 66, 40, 16, "url(#zap)", extra='opacity="0.75"')
    b += E(64, 78, 20, 10, "url(#zap)", extra='opacity="0.6"')
    for (x, y, r) in front:
        b += C(x, y, r, "url(#bf)")
    b += P(belly, "url(#belly)")
    # Lavender rim light on the tops of the billows.
    for (x, y, r) in front + back[1:3]:
        a0, a1 = math.radians(200), math.radians(290)
        b += P(f"M{x + r * 0.86 * math.cos(a0):.1f},{y + r * 0.86 * math.sin(a0):.1f} A{r * 0.86:.1f},{r * 0.86:.1f} 0 0 1 {x + r * 0.86 * math.cos(a1):.1f},{y + r * 0.86 * math.sin(a1):.1f}",
               "none", "#e6e4ff", 4, 'opacity="0.55"')
    # Belly folds.
    b += P("M30,108 Q56,100 82,110 Q108,102 134,110 Q160,102 196,108", "none", "#7d80a3", 3, 'opacity="0.45"')
    # Face: a heavy furrowed brow, glowing slit eyes, gritted scowl.
    b += P("M76,74 L110,88 L106,94 L72,82 Z", "#25263f", "#25263f", 3)
    b += P("M156,74 L122,88 L126,94 L160,82 Z", "#25263f", "#25263f", 3)
    b += P("M84,88 Q96,84 106,96 Q94,102 84,96 Z", "url(#eye)", "#7a3a10", 2.5, 'filter="none"')
    b += P("M148,88 Q136,84 126,96 Q138,102 148,96 Z", "url(#eye)", "#7a3a10", 2.5)
    b += C(98, 94, 3.2, "#2a1408") + C(134, 94, 3.2, "#2a1408")
    b += P("M98,114 Q116,104 134,114 L132,120 Q116,112 100,120 Z", "#fdfbff", "#25263f", 3)
    b += P("M108,109 L108,117 M116,107 L116,116 M124,109 L124,117", "none", "#7d80a3", 2)
    write("stormcloud", W, H, b, defs, scale=3)

def bolt():
    """One forked lightning bolt hanging from its top centre, yellow-white
    with an amber edge and a soft glow (64x192)."""
    defs = BLUR("lb", 5)
    main = "M30,2 L44,2 L36,56 L50,56 L26,118 L36,118 L14,190 L20,124 L8,124 L22,66 L10,66 Z"
    fork = "M38,70 L56,96 L48,98 L60,128 L42,100 L48,98 Z"
    b = ""
    b += P(main, "#ffe45a", "#ffd23a", 10, 'filter="url(#lb)" opacity="0.9"')
    b += P(fork, "#ffe45a", "#ffd23a", 8, 'filter="url(#lb)" opacity="0.8"')
    b += P(fork, "#ffe65a", "#c86a10", 3)
    b += P(main, "#ffe65a", "#c86a10", 3.5)
    b += P("M33,8 L38,8 L30,60 L42,60 L24,108 L28,70 L18,70 Z", "#fffbe6")
    write("bolt", 64, 192, b, defs, scale=2)

# ---------------------------------------------------------------- set dressing
def candycane():
    """Peppermint cane: a round, glossy tube with red spiral stripes and a
    holly bow (64x128)."""
    defs = lg("cw", [(0, "#c9d6ea"), (0.25, "#ffffff"), (0.45, "#ffffff"), (1, "#a8b8d4")], x2=1, y2=0)
    defs += lg("cr", [(0, "#a8141c"), (0.3, "#ff5a52"), (0.5, "#e8262c"), (1, "#8a0e16")], x2=1, y2=0)
    path = "M22,124 L22,42 C22,12 54,12 54,40 L54,50"
    defs += f'<clipPath id="cc"><path d="{path}" fill="none" stroke="#000" stroke-width="18" stroke-linecap="round"/></clipPath>'
    defs += '<mask id="cm"><path d="M22,124 L22,42 C22,12 54,12 54,40 L54,50" fill="none" stroke="#fff" stroke-width="17" stroke-linecap="round"/></mask>'
    b = P(path, "none", "#8fa0c0", 22)
    b += P(path, "none", "url(#cw)", 17)
    stripes = ""
    for y in range(-20, 140, 18):
        stripes += P(f"M0,{y + 14} L70,{y - 16} L70,{y - 8} L0,{y + 22} Z", "url(#cr)")
    b += f'<g mask="url(#cm)">{stripes}</g>'
    b += P("M18,120 L18,44 C18,26 30,18 38,18", "none", "#ffffff", 3.5, 'opacity="0.85"')
    # Holly bow.
    b += P("M22,74 C8,62 2,74 8,82 C14,88 20,80 22,74 Z", "#3fa046", "#1c5a26", 2.5)
    b += P("M22,74 C36,62 44,72 38,80 C32,88 24,80 22,74 Z", "#3fa046", "#1c5a26", 2.5)
    b += C(18, 76, 4, "#e8262c", "#8a0e16", 1.8) + C(25, 78, 4, "#e8262c", "#8a0e16", 1.8)
    b += C(17, 75, 1.3, "#ffffff")
    write("candycane", 64, 128, b, defs, scale=2)

def xmas_tree():
    """A decorated spruce: four tiers of drooping boughs lit from the left,
    snow sitting on every bough tip, a gold garland, glossy baubles and a
    glowing star (256x336, base at the bottom centre)."""
    rnd = random.Random(11)
    G = "#2f8f4a"
    defs = lg("tg", [(0, "#5cc46a"), (0.5, G), (1, "#1c5c3a")], x2=1, y2=0.3)
    defs += lg("tr", [(0, "#a86a3a"), (1, "#5e3417")], x2=1, y2=0)
    defs += lg("sn", [(0, "#ffffff"), (1, "#d6e6f6")])
    defs += rg("st", [(0, "#fffbe0", 1), (0.5, "#ffd23a", 0.6), (1, "#ffd23a", 0)])
    defs += lg("sg", [(0, "#fff6c8"), (0.4, "#ffd24a"), (1, "#d18a12")])
    b = ""
    b += E(128, 326, 80, 9, "#1a2a50", extra='opacity="0.2"')
    b += f'<rect x="114" y="284" width="28" height="44" rx="5" fill="url(#tr)" stroke="#4a2a12" stroke-width="4"/>'
    tiers = [(312, 120, 82), (250, 100, 78), (190, 78, 70), (130, 54, 64)]
    for i, (yb, w, h) in enumerate(tiers):
        top = yb - h - 20
        n = 5 if w > 90 else 4 if w > 60 else 3
        d = f"M128,{top} "
        # Right edge down to the scalloped bottom, back up the left.
        d += f"C{128 + w * 0.3},{top + h * 0.4} {128 + w * 0.8},{yb - 24} {128 + w},{yb - 6} "
        pts = [128 + w - (2 * w) * (j + 1) / n for j in range(n)]
        px = 128 + w
        for x in pts:
            mx = (px + x) / 2
            d += f"Q{mx},{yb + 8} {x},{yb - 6} "
            px = x
        d += f"C{128 - w * 0.8},{yb - 24} {128 - w * 0.3},{top + h * 0.4} 128,{top} Z"
        b += P(d, "url(#tg)", "#164a2c", 4.5)
        # Needle strokes.
        for k in range(10 + i * 2):
            x = 128 + (rnd.random() - 0.5) * w * 1.4
            y = top + h * 0.35 + rnd.random() * (h * 0.6)
            if abs(x - 128) < (y - top) / (h + 10) * w * 0.85:
                b += P(f"M{x:.1f},{y:.1f} l{(x - 128) * 0.12:.1f},{10 + rnd.random() * 6:.1f}", "none", "#1c5c3a" if x > 128 else "#6fd27c", 2.5, 'opacity="0.7"')
        # Snow on the bough tips and along the top edge on the lit side.
        px = 128 + w
        for j, x in enumerate([128 + w] + pts[:-1]):
            sx = x - (2 * w) / n * 0.5
            b += P(f"M{sx - 14},{yb - 4} Q{sx},{yb - 16} {sx + 14},{yb - 4} Q{sx},{yb + 2} {sx - 14},{yb - 4} Z", "url(#sn)", "#8fb0d0", 2.5)
        b += P(f"M128,{top + 2} C{128 - w * 0.2},{top + h * 0.3} {128 - w * 0.5},{top + h * 0.6} {128 - w * 0.62},{top + h * 0.72} "
               f"Q{128 - w * 0.4},{top + h * 0.62} {128 - w * 0.1},{top + h * 0.3} Z", "url(#sn)", "#8fb0d0", 2)
    # Garland swags.
    for (y, w) in ((262, 92), (204, 70), (146, 48)):
        b += P(f"M{128 - w},{y - 30} Q128,{y + 6} {128 + w},{y - 40}", "none", "#9a5a10", 6)
        b += P(f"M{128 - w},{y - 30} Q128,{y + 6} {128 + w},{y - 40}", "none", "#ffd24a", 3.5, 'stroke-dasharray="4 3"')
    # Baubles: glossy spheres.
    cols = ["#e8262c", "#2f6fd6", "#ffc22a", "#e04fa0", "#e8262c", "#9b5de5", "#ffc22a", "#2f6fd6", "#e8262c"]
    spots = [(92, 296), (160, 290), (128, 270), (78, 236), (176, 232), (112, 214), (150, 176), (100, 168), (130, 118)]
    for i, ((x, y), c) in enumerate(zip(spots, cols)):
        defs += rg(f"bl{i}", [(0, "#ffffff", 1), (0.25, light(c), 1), (0.7, c, 1), (1, shadow(shadow(c)), 1)], cx=0.4, cy=0.35, r=0.65, fx=0.35, fy=0.3)
        b += P(f"M{x},{y - 11} L{x},{y - 16}", "none", "#9a5a10", 2)
        b += C(x, y, 10, f"url(#bl{i})", edge(c), 2.5)
        b += E(x - 3.5, y - 4, 3.5, 2.2, "#ffffff", -30, 'opacity="0.9"')
    # Star with a glow.
    b += C(128, 46, 40, "url(#st)")
    b += P(star_d(128, 48, 30, 13), "url(#sg)", "#a86a0e", 4)
    b += P(star_d(124, 44, 12, 5), "#fffbe6", extra='opacity="0.85"')
    write("xmas_tree", 256, 336, b, defs, scale=2)

def lamppost():
    """Victorian street lamp: fluted dark-green iron post, a lantern with a
    warm glowing pane and a snow cap, a wreath with a red bow (96x288)."""
    IRON = "#2f4a4a"
    defs = lg("ir", [(0, "#1c2e30"), (0.3, "#6d8f8a"), (0.5, IRON), (1, "#142022")], x2=1, y2=0)
    defs += lg("gw", [(0, "#fffbe0"), (0.6, "#ffd86a"), (1, "#ffb02e")])
    defs += rg("halo", [(0, "#fff2b0", 0.9), (0.4, "#ffd86a", 0.35), (1, "#ffd86a", 0)])
    defs += lg("sn", [(0, "#ffffff"), (1, "#d6e6f6")])
    defs += lg("wr", [(0, "#5cc46a"), (1, "#1c5c3a")])
    b = ""
    b += E(48, 284, 30, 4, "#1a2a50", extra='opacity="0.25"')
    b += P("M28,286 L68,286 L64,268 L56,262 L40,262 L32,268 Z", "url(#ir)", "#0f1a1b", 3)
    b += f'<rect x="42" y="96" width="12" height="170" rx="3" fill="url(#ir)" stroke="#0f1a1b" stroke-width="3"/>'
    for y in (120, 200, 250):
        b += f'<rect x="38" y="{y}" width="20" height="8" rx="3" fill="url(#ir)" stroke="#0f1a1b" stroke-width="2.5"/>'
    # Wreath on the post.
    b += C(48, 160, 17, "none", "#1c5c3a", 11) + C(48, 160, 17, "none", "url(#wr)", 7)
    for a in range(0, 360, 45):
        b += C(48 + 17 * math.cos(math.radians(a)), 160 + 17 * math.sin(math.radians(a)), 2.6, "#e8262c")
    b += P("M48,176 L36,190 L42,192 L48,180 L54,192 L60,190 Z", "#e8262c", "#8a0e16", 2)
    b += P("M38,170 C40,162 46,168 48,174 C50,168 56,162 58,170 C56,178 50,176 48,174 C46,176 40,178 38,170 Z", "#e8262c", "#8a0e16", 2)
    # Lantern.
    b += C(48, 52, 46, "url(#halo)")
    b += P("M36,96 L60,96 L56,86 L40,86 Z", "url(#ir)", "#0f1a1b", 3)
    b += P("M30,86 L66,86 L72,40 L24,40 Z", "url(#gw)", "#0f1a1b", 3.5)
    b += P("M48,40 L48,86 M30,63 L68,63", "none", "#2f4a4a", 3)
    b += P("M30,44 L36,44 L33,80 L30,80 Z", "#ffffff", extra='opacity="0.6"')
    b += P("M18,42 L78,42 L66,24 L30,24 Z", "url(#ir)", "#0f1a1b", 3.5)
    b += C(48, 16, 7, "url(#ir)", "#0f1a1b", 3)
    # Snow cap on the lantern roof.
    b += P("M16,40 Q14,30 28,26 Q48,16 68,26 Q82,30 80,40 Q72,34 60,38 Q48,32 36,38 Q24,34 16,40 Z", "url(#sn)", "#8fb0d0", 2.5)
    write("lamppost", 96, 288, b, defs, scale=2)

def gift_pile():
    """Three wrapped boxes stacked at the tower base, using the present
    art for the boxes (192x128)."""
    b = ""
    defs = ""
    for name, x, y, sz in (("present_green", -6, 34, 96), ("present_purple", 96, 38, 90), ("present_red", 42, -8, 104)):
        src = open(os.path.join(OUT, name + ".svg")).read()
        inner = src.split("</defs>", 1)[1].rsplit("</svg>", 1)[0]
        d = src.split("<defs>", 1)[1].split("</defs>", 1)[0]
        defs += d
        b += f'<g transform="translate({x},{y}) scale({sz / 192:.4f})">{inner}</g>'
    write("gift_pile", 192, 128, b, defs, scale=2)

def run():
    present("present_red", "#e23a30", "#ffcf3a", pattern="flakes", pat_col="#ff8a7a")
    present("present_green", "#2fa353", "#e8262c", pattern="dots", pat_col="#eaffe8", W=100, H=78, D=30)
    present("present_blue", "#3577e0", "#f4f8ff", pattern="stars", pat_col="#bfe0ff", W=118, H=62, D=36)
    present("present_purple", "#9b4fd6", "#3fe0c0", pattern="stripes", pat_col="#c58cf0", W=90, H=88, D=28)
    present("present_gold", "#f5b52a", "#e8262c", gold=True, W=108, H=72, D=34)
    firework()
    rocket_flame()
    stormcloud()
    bolt()
    candycane()
    xmas_tree()
    lamppost()
    gift_pile()

if __name__ == "__main__":
    run()
    print("painted props written")
