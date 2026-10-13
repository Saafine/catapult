#!/usr/bin/env python3
"""UI kit: the game logo, paper panels, ribbons, wax seals, medals, stamps,
buttons, the power dial and the HUD icons, painted like the key art (top-lit
gradient fills, outlines in a deeper shade of each fill, soft shading, a
light paint grain) on the cream paper Christmas-card identity of the HUD.

Writes packs/<PACK>/svg/ui_*.svg with data-paint="skip" (they carry their
own grain, so tools/paint.py leaves them alone). Rasterise with
  ONLY='^ui_' node tools/render_sprites.js

Slice contracts the game relies on (crates/game/src/ui.rs), in svg px; the
png is 2x:
  ui_panel*.svg      120x120, 9-slice, border 40, body inset 6/4/6/10
  ui_ribbon_*.svg    360x100, 3-slice, caps 96, body y 20..76
  ui_dial.svg        320x176, dial centre at (160,152)
  ui_needle.svg      28x124, pivot at (14,110)
"""
import base64, math, os, random

PACK = os.environ.get("PACK", "calgary_xmas")
ROOT = os.path.join(os.path.dirname(__file__), "..")
OUT = os.path.join(ROOT, "packs", PACK, "svg")
FONTS = os.path.join(ROOT, "crates", "game", "assets")

# Paint ramps: (light, mid, deep, line). The line is a deeper shade of the fill.
PAPER = ("#fffdf6", "#fbf0d8", "#efd8ac", "#a8733f")
BERRY = ("#ff7d74", "#e2373f", "#a51b29", "#6e0d18")
PINE = ("#7fdc8f", "#2fa35d", "#1b6b3c", "#0d4325")
GOLD = ("#fff1a6", "#f9c846", "#d48a1c", "#8a520c")
ICE = ("#f4fbff", "#c4e8fb", "#8cbfe3", "#4d7aa3")
SILVER = ("#ffffff", "#d9dee8", "#a3abbd", "#5d6578")
BRONZE = ("#ffd2a3", "#d98c4b", "#a2582a", "#5e2f12")
SNOW = ("#ffffff", "#eef6fd", "#c7dcef", "#8fb0d0")
WOOD = ("#ffc27a", "#f0913a", "#c0601d", "#7a3510")
INK = "#1e2440"


def font_face():
    out = []
    for fam, f in (("Lilita", "LilitaOne.ttf"), ("Fredoka", "Fredoka.ttf")):
        b = base64.b64encode(open(os.path.join(FONTS, f), "rb").read()).decode()
        out.append(f"@font-face{{font-family:'{fam}';src:url(data:font/ttf;base64,{b}) format('truetype');}}")
    return "<style>" + "".join(out) + "</style>"


def grain(fid="grain", fine=0.8, size=200):
    return f'''<filter id="{fid}" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
<feTurbulence type="fractalNoise" baseFrequency="{fine}" numOctaves="2" seed="7" result="fine"/>
<feTurbulence type="fractalNoise" baseFrequency="{36 / size:.4f}" numOctaves="3" seed="3" result="soft"/>
<feColorMatrix in="fine" type="matrix" result="fg" values="0.16 0 0 0 0.42  0.16 0 0 0 0.42  0.16 0 0 0 0.42  0 0 0 0 1"/>
<feColorMatrix in="soft" type="matrix" result="sg" values="0.3 0 0 0 0.35  0.3 0 0 0 0.35  0.3 0 0 0 0.35  0 0 0 0 1"/>
<feComposite in="fg" in2="sg" operator="arithmetic" k1="0" k2="0.5" k3="0.5" k4="0" result="noise"/>
<feComposite in="noise" in2="SourceAlpha" operator="in" result="masked"/>
<feBlend in="masked" in2="SourceGraphic" mode="soft-light" result="lit"/>
<feComposite in="lit" in2="SourceAlpha" operator="in"/></filter>'''


def vgrad(gid, ramp, stops=(0, 0.45, 1)):
    l, m, d = ramp[:3]
    return f'<linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1"><stop offset="{stops[0]}" stop-color="{l}"/><stop offset="{stops[1]}" stop-color="{m}"/><stop offset="{stops[2]}" stop-color="{d}"/></linearGradient>'


def rgrad(gid, ramp, cx=0.38, cy=0.3, r=0.75):
    l, m, d = ramp[:3]
    return f'<radialGradient id="{gid}" cx="{cx}" cy="{cy}" r="{r}"><stop offset="0" stop-color="{l}"/><stop offset="0.5" stop-color="{m}"/><stop offset="1" stop-color="{d}"/></radialGradient>'


def blur(fid, sd):
    return f'<filter id="{fid}" x="-40%" y="-40%" width="180%" height="180%"><feGaussianBlur stdDeviation="{sd}"/></filter>'


def svg(name, w, h, defs, body, use_grain=True, fonts=False, size=None):
    g = grain(size=size or max(w, h)) if use_grain else ""
    body = f'<g filter="url(#grain)">{body}</g>' if use_grain else body
    s = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" data-paint="skip">'
         f'{font_face() if fonts else ""}<defs>{g}{defs}</defs>{body}</svg>')
    open(os.path.join(OUT, name + ".svg"), "w").write(s)
    print("wrote", name)


def star_pts(cx, cy, r1, r2, n=5, rot=-90):
    pts = []
    for i in range(n * 2):
        r = r1 if i % 2 == 0 else r2
        a = math.radians(rot + i * 180 / n)
        pts.append(f"{cx + r * math.cos(a):.1f},{cy + r * math.sin(a):.1f}")
    return " ".join(pts)


def sparkle(cx, cy, r, col="#ffffff", op=1.0):
    """Four-point twinkle."""
    k = r * 0.22
    d = f"M{cx},{cy - r} Q{cx + k},{cy - k} {cx + r},{cy} Q{cx + k},{cy + k} {cx},{cy + r} Q{cx - k},{cy + k} {cx - r},{cy} Q{cx - k},{cy - k} {cx},{cy - r}Z"
    return f'<path d="{d}" fill="{col}" opacity="{op}"/>'


# ---------------------------------------------------------------------------
# Logo
# ---------------------------------------------------------------------------

def wing(side):
    """A snowy feathered wing fanning out from behind the ribbon, left (-1)
    or right (+1) of x=310: six primaries back to front, then the coverts."""
    rx, ry = 310 + side * 112, 96
    out = []
    feathers = [(56, 104, 30), (46, 130, 32), (35, 152, 33), (25, 170, 34), (15, 182, 34), (5, 186, 32)]
    for ang, length, w in feathers:
        a = math.radians(180 - ang) if side < 0 else math.radians(ang)
        rot = math.degrees(-a) if side > 0 else 180 - math.degrees(math.radians(180 - ang)) + 180
        # build the feather along +x then rotate into place
        hw = w / 2
        d = (f"M0,{-hw * 0.7:.1f} C{length * 0.4:.1f},{-hw * 1.05:.1f} {length - hw * 1.6:.1f},{-hw:.1f} {length - hw * 0.4:.1f},{-hw * 0.5:.1f} "
             f"Q{length + 2:.1f},0 {length - hw * 0.4:.1f},{hw * 0.6:.1f} C{length - hw * 1.6:.1f},{hw:.1f} {length * 0.4:.1f},{hw * 0.95:.1f} 0,{hw * 0.7:.1f} Z")
        deg = -ang if side > 0 else -(180 - ang)
        tr = f"translate({rx} {ry}) rotate({deg:.1f})"
        out.append(f'<g transform="{tr}"><path d="{d}" fill="#1d2a4a" opacity="0.22" transform="translate(0 6)" filter="url(#soft2)"/>'
                   f'<path d="{d}" fill="url(#gWing)" stroke="{SNOW[3]}" stroke-width="3.5" stroke-linejoin="round"/>'
                   f'<path d="M14,1 Q{length * 0.5:.1f},3 {length - 14:.1f},1" fill="none" stroke="#a9c6e0" stroke-width="2.4" stroke-linecap="round"/>'
                   f'<path d="M18,{-hw * 0.45:.1f} Q{length * 0.5:.1f},{-hw * 0.7:.1f} {length - 22:.1f},{-hw * 0.4:.1f}" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round" opacity="0.9"/></g>')
    # coverts: a rounded scalloped shoulder over the feather roots
    cx = rx - side * 4
    pts = []
    for k in range(7):
        t = k / 6
        x = cx + side * (10 + 70 * t)
        y = ry - 26 + 34 * t
        pts.append((x, y))
    d = f"M{cx},{ry - 40} C{cx + side * 50},{ry - 52} {cx + side * 92},{ry - 30} {cx + side * 86},{ry - 4} "
    for k in range(5, -1, -1):
        x0, y0 = pts[k + 1]
        x1, y1 = pts[k]
        d += f"Q{(x0 + x1) / 2 + side * 6:.1f},{max(y0, y1) + 14:.1f} {x1:.1f},{y1:.1f} "
    d += f"L{cx},{ry + 8} Z"
    out.append(f'<path d="{d}" fill="url(#gWing)" stroke="{SNOW[3]}" stroke-width="3.5" stroke-linejoin="round"/>')
    return "".join(out)


def logo():
    W, H = 620, 250
    defs = (vgrad("gWing", SNOW, (0, 0.5, 1)) + vgrad("gRib", BERRY) + vgrad("gRibD", ("#c22a35", "#8e1622", "#5e0c16"))
            + '<linearGradient id="gTitle" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff6b8"/><stop offset="0.38" stop-color="#ffd24a"/><stop offset="0.72" stop-color="#f7a02b"/><stop offset="1" stop-color="#e0661a"/></linearGradient>'
            + blur("soft4", 4) + blur("soft2", 2) + blur("soft8", 8)
            + '<clipPath id="cTitle"><text x="310" y="214" text-anchor="middle" font-family="Lilita" font-size="88" textLength="560" lengthAdjust="spacingAndGlyphs">FLIGHT SCHOOL</text></clipPath>')
    t = 'x="310" y="214" text-anchor="middle" font-family="Lilita" font-size="88" textLength="560" lengthAdjust="spacingAndGlyphs"'
    body = []
    # A soft warm glow behind everything so it lifts off any sky.
    body.append('<g transform="translate(0 18)">')
    body.append(wing(-1) + wing(1))
    # Ribbon tails behind "SANTA'S".
    for s in (-1, 1):
        x0 = 310 + s * 128
        body.append(f'<path d="M{x0},66 L{x0 + s * 62},66 L{x0 + s * 46},86 L{x0 + s * 62},106 L{x0},106 Z" fill="url(#gRibD)" stroke="{BERRY[3]}" stroke-width="4" stroke-linejoin="round"/>')
        body.append(f'<path d="M{x0},96 L{x0 - s * 18},106 L{x0},106 Z" fill="#4a0810"/>')
    body.append(f'<path d="M180,48 Q310,40 440,48 L440,96 Q310,88 180,96 Z" fill="#1d2a4a" opacity="0.3" transform="translate(0 6)" filter="url(#soft4)"/>')
    body.append(f'<path d="M180,48 Q310,40 440,48 L440,96 Q310,88 180,96 Z" fill="url(#gRib)" stroke="{BERRY[3]}" stroke-width="4" stroke-linejoin="round"/>')
    body.append('<path d="M188,56 Q310,48 432,56" fill="none" stroke="#ffb3a8" stroke-width="3" opacity="0.8"/>')
    # stitched edges
    body.append('<path d="M186,61 Q310,53 434,61 M186,84 Q310,76 434,84" fill="none" stroke="#ffd9d0" stroke-width="1.6" stroke-dasharray="5 4" opacity="0.7"/>')
    body.append(f'<text x="310" y="88" text-anchor="middle" font-family="Lilita" font-size="42" letter-spacing="3" fill="#fff8ec" stroke="{BERRY[3]}" stroke-width="7" paint-order="stroke" stroke-linejoin="round">SANTA&#8217;S</text>')
    body.append('</g>')
    # FLIGHT SCHOOL: soft drop, extrusion, deep outline, gold body, snow caps.
    body.append(f'<text {t} fill="#1d2a4a" opacity="0.35" stroke="#1d2a4a" stroke-width="18" stroke-linejoin="round" transform="translate(0 12)" filter="url(#soft4)">FLIGHT SCHOOL</text>')
    for dy in range(9, 0, -1):
        body.append(f'<text {t} fill="#7a2e0c" stroke="#5a1f08" stroke-width="14" stroke-linejoin="round" transform="translate(0 {dy})">FLIGHT SCHOOL</text>')
    body.append(f'<text {t} fill="url(#gTitle)" stroke="#7a3510" stroke-width="12" stroke-linejoin="round" paint-order="stroke">FLIGHT SCHOOL</text>')
    # Snow on the letter tops: a drippy band, clipped to the glyphs.
    random.seed(4)
    pts = []
    x = 20
    while x < 600:
        pts.append((x, 160 + random.uniform(-2, 3)))
        x += random.uniform(8, 16)
        if random.random() < 0.35:
            pts.append((x, 172 + random.uniform(0, 9)))
            x += 7
    d = "M20,120 L600,120 L600,165 " + " ".join(f"L{px:.0f},{py:.0f}" for px, py in reversed(pts)) + " Z"
    body.append(f'<g clip-path="url(#cTitle)"><path d="{d}" fill="{SNOW[2]}" transform="translate(0 4)"/><path d="{d}" fill="#ffffff"/>'
                f'<rect x="20" y="186" width="580" height="30" fill="#b8430f" opacity="0.25"/></g>')
    # Glints.
    for (gx, gy, gr) in ((92, 150, 9), (520, 176, 7), (300, 150, 6)):
        body.append(sparkle(gx, gy, gr, "#ffffff", 0.95))
    # Gold stars and twinkles around.
    for (sx_, sy_, r) in ((150, 40, 13), (470, 36, 11), (40, 136, 8), (584, 128, 9)):
        body.append(f'<polygon points="{star_pts(sx_, sy_ + 2, r, r * 0.45)}" fill="#8a520c" opacity="0.5"/>')
        body.append(f'<polygon points="{star_pts(sx_, sy_, r, r * 0.45)}" fill="url(#gTitle)" stroke="#8a520c" stroke-width="2.4" stroke-linejoin="round"/>')
    for (sx_, sy_, r) in ((112, 70, 7), (510, 74, 6), (24, 96, 5), (600, 92, 6)):
        body.append(sparkle(sx_, sy_, r, "#fffbe6"))
    svg("ui_logo", W, H, defs, "".join(body), fonts=True)


# ---------------------------------------------------------------------------
# Panels (9-slice)
# ---------------------------------------------------------------------------

def panel(name, ramp, line, rule1, rule2, thick=4.0):
    W = H = 120
    defs = vgrad("gP", ramp, (0, 0.5, 1)) + blur("sh", 3.2) + blur("in", 2.0)
    body = []
    # Soft blue-grey drop shadow below.
    body.append('<rect x="7" y="11" width="106" height="104" rx="20" fill="#1d2a4a" opacity="0.34" filter="url(#sh)"/>')
    body.append(f'<rect x="6" y="4" width="108" height="106" rx="20" fill="url(#gP)" stroke="{line}" stroke-width="{thick}"/>')
    # Inner bevel: light top edge, warm bottom shade.
    body.append('<rect x="10" y="8" width="100" height="98" rx="16" fill="none" stroke="#ffffff" stroke-width="2.5" opacity="0.75"/>')
    body.append(f'<path d="M12,98 Q60,104 108,98 L108,104 Q60,108 12,104 Z" fill="{line}" opacity="0.18" filter="url(#in)"/>')
    # Christmas-card double rule.
    body.append(f'<rect x="15" y="13" width="90" height="88" rx="12" fill="none" stroke="{rule1}" stroke-width="2.2"/>')
    if rule2:
        body.append(f'<rect x="19" y="17" width="82" height="80" rx="9" fill="none" stroke="{rule2}" stroke-width="1.2" opacity="0.9"/>')
    svg(name, W, H, defs, "".join(body), size=120)


def chip():
    """A small plain paper chip, 9-slice border 24 (for counters and rows)."""
    W = H = 72
    defs = vgrad("gP", PAPER, (0, 0.55, 1)) + blur("sh", 2.2)
    body = ('<rect x="5" y="8" width="62" height="58" rx="14" fill="#1d2a4a" opacity="0.32" filter="url(#sh)"/>'
            f'<rect x="4" y="3" width="64" height="61" rx="14" fill="url(#gP)" stroke="{PAPER[3]}" stroke-width="3.5"/>'
            '<path d="M14,8 L58,8" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" opacity="0.8"/>')
    svg("ui_chip", W, H, defs, body, size=72)


def dark_chip():
    W = H = 72
    defs = vgrad("gP", ("#3b4670", "#262e52", "#161b33"), (0, 0.5, 1)) + blur("sh", 2.2)
    body = ('<rect x="5" y="8" width="62" height="58" rx="14" fill="#0b0f20" opacity="0.35" filter="url(#sh)"/>'
            f'<rect x="4" y="3" width="64" height="61" rx="14" fill="url(#gP)" stroke="#0d1126" stroke-width="3.5"/>'
            '<path d="M14,8 L58,8" stroke="#7d8bc0" stroke-width="2" stroke-linecap="round" opacity="0.8"/>')
    svg("ui_chip_dark", W, H, defs, body, size=72)


# ---------------------------------------------------------------------------
# Ribbons (3-slice)
# ---------------------------------------------------------------------------

def ribbon(name, ramp):
    W, H = 360, 100
    l, m, d, line = ramp
    dark = (d, d, line)
    defs = vgrad("gR", ramp, (0, 0.5, 1)) + vgrad("gD", dark, (0, 0.5, 1)) + blur("sh", 3.5)
    body = []
    for s in (-1, 1):
        # Tail: hangs lower behind the body, swallow-tail notch outward.
        x0 = 180 + s * 150
        xa = 180 + s * 178
        body.append(f'<path d="M{x0},38 L{xa},38 L{xa - s * 18},62 L{xa},86 L{x0},86 Z" fill="#1d2a4a" opacity="0.3" transform="translate(0 5)" filter="url(#sh)"/>')
        body.append(f'<path d="M{x0},38 L{xa},38 L{xa - s * 18},62 L{xa},86 L{x0},86 Z" fill="url(#gD)" stroke="{line}" stroke-width="3.5" stroke-linejoin="round"/>')
        # The fold: a dark triangle where the body tucks under.
        xb = 180 + s * 132
        body.append(f'<path d="M{xb},76 L{x0},86 L{x0},76 Z" fill="{line}"/>')
    body.append('<rect x="48" y="25" width="264" height="56" fill="#1d2a4a" opacity="0.32" filter="url(#sh)"/>')
    body.append(f'<rect x="48" y="20" width="264" height="56" fill="url(#gR)" stroke="{line}" stroke-width="3.5"/>')
    body.append(f'<rect x="50" y="25" width="260" height="3" fill="#ffffff" opacity="0.45"/>')
    body.append(f'<rect x="50" y="68" width="260" height="6" fill="{line}" opacity="0.25"/>')
    body.append(f'<path d="M50,31 L310,31 M50,66 L310,66" stroke="#ffffff" stroke-width="1.4" stroke-dasharray="6 5" opacity="0.45"/>')
    svg(name, W, H, defs, "".join(body), size=360)


# ---------------------------------------------------------------------------
# Seals, medals, rosette, stamps
# ---------------------------------------------------------------------------

def blob(cx, cy, r, n=11, seed=1, amp=0.09):
    random.seed(seed)
    pts = []
    for i in range(n * 3):
        a = i / (n * 3) * math.tau
        k = 1 + amp * math.sin(a * n) + random.uniform(-0.03, 0.03)
        pts.append((cx + r * k * math.cos(a), cy + r * k * math.sin(a)))
    d = f"M{pts[0][0]:.1f},{pts[0][1]:.1f} " + " ".join(f"L{x:.1f},{y:.1f}" for x, y in pts[1:]) + " Z"
    return d


def seal(name, ramp, emblem="star"):
    W = H = 100
    l, m, d, line = ramp
    defs = rgrad("gS", ramp, 0.36, 0.3, 0.8) + rgrad("gI", (m, d, l), 0.6, 0.7, 0.8) + blur("sh", 2.5) + blur("b1", 1.2)
    body = []
    outer = blob(50, 50, 42, 11, 3)
    body.append(f'<path d="{outer}" fill="#1d2a4a" opacity="0.35" transform="translate(0 4)" filter="url(#sh)"/>')
    body.append(f'<path d="{outer}" fill="url(#gS)" stroke="{line}" stroke-width="3.5" stroke-linejoin="round"/>')
    # pressed inner disc
    body.append(f'<circle cx="50" cy="50" r="29" fill="url(#gI)" stroke="{line}" stroke-width="2" opacity="0.9"/>')
    body.append(f'<circle cx="50" cy="51" r="29" fill="none" stroke="{l}" stroke-width="2" opacity="0.6"/>')
    if emblem == "star":
        body.append(f'<polygon points="{star_pts(50, 52, 17, 7.5)}" fill="{line}" opacity="0.5"/>')
        body.append(f'<polygon points="{star_pts(50, 50, 17, 7.5)}" fill="{l}" opacity="0.85"/>')
    body.append(f'<ellipse cx="36" cy="30" rx="12" ry="6" fill="#ffffff" opacity="0.45" transform="rotate(-30 36 30)" filter="url(#b1)"/>')
    svg(name, W, H, defs, "".join(body), size=100)


def medal(name, ramp, rib_a, rib_b):
    W, H = 120, 160
    l, m, d, line = ramp
    defs = rgrad("gM", ramp, 0.36, 0.3, 0.8) + vgrad("gA", rib_a) + vgrad("gB", rib_b) + blur("sh", 3) + blur("b1", 1.4)
    body = []
    # V ribbon
    body.append(f'<path d="M30,4 L58,4 L72,70 L50,74 Z" fill="url(#gA)" stroke="{rib_a[3]}" stroke-width="3" stroke-linejoin="round"/>')
    body.append(f'<path d="M90,4 L62,4 L48,70 L70,74 Z" fill="url(#gB)" stroke="{rib_b[3]}" stroke-width="3" stroke-linejoin="round"/>')
    body.append('<path d="M44,6 L60,72 M76,6 L60,72" stroke="#ffffff" stroke-width="3" opacity="0.5"/>')
    body.append('<circle cx="60" cy="106" r="44" fill="#1d2a4a" opacity="0.35" transform="translate(0 5)" filter="url(#sh)"/>')
    body.append(f'<circle cx="60" cy="106" r="44" fill="url(#gM)" stroke="{line}" stroke-width="4"/>')
    body.append(f'<circle cx="60" cy="106" r="34" fill="none" stroke="{d}" stroke-width="3"/>')
    body.append(f'<circle cx="60" cy="107" r="34" fill="none" stroke="{l}" stroke-width="1.6" opacity="0.8"/>')
    body.append(f'<polygon points="{star_pts(60, 110, 22, 9.5)}" fill="{line}" opacity="0.45"/>')
    body.append(f'<polygon points="{star_pts(60, 107, 22, 9.5)}" fill="{l}" stroke="{d}" stroke-width="2" stroke-linejoin="round"/>')
    body.append('<ellipse cx="42" cy="84" rx="15" ry="8" fill="#ffffff" opacity="0.55" transform="rotate(-35 42 84)" filter="url(#b1)"/>')
    svg(name, W, H, defs, "".join(body), size=160)


def rosette():
    W, H = 170, 210
    defs = rgrad("gM", GOLD, 0.36, 0.3, 0.8) + vgrad("gR", BERRY) + vgrad("gG", PINE) + blur("sh", 3.2) + blur("b1", 1.5)
    body = []
    # tails
    body.append(f'<path d="M62,110 L36,200 L52,190 L64,206 L86,118 Z" fill="url(#gR)" stroke="{BERRY[3]}" stroke-width="3" stroke-linejoin="round"/>')
    body.append(f'<path d="M108,110 L134,200 L118,190 L106,206 L84,118 Z" fill="url(#gG)" stroke="{PINE[3]}" stroke-width="3" stroke-linejoin="round"/>')
    # pleated petals
    pts = []
    n = 26
    for i in range(n * 2):
        a = i / (n * 2) * math.tau
        r = 76 if i % 2 == 0 else 64
        pts.append(f"{85 + r * math.cos(a):.1f},{86 + r * math.sin(a):.1f}")
    pl = " ".join(pts)
    body.append(f'<polygon points="{pl}" fill="#1d2a4a" opacity="0.35" transform="translate(0 5)" filter="url(#sh)"/>')
    body.append(f'<polygon points="{pl}" fill="url(#gR)" stroke="{BERRY[3]}" stroke-width="3" stroke-linejoin="round"/>')
    for i in range(n):
        a = (i + 0.5) / n * math.tau
        body.append(f'<line x1="{85 + 40 * math.cos(a):.1f}" y1="{86 + 40 * math.sin(a):.1f}" x2="{85 + 70 * math.cos(a):.1f}" y2="{86 + 70 * math.sin(a):.1f}" stroke="{BERRY[3]}" stroke-width="1.6" opacity="0.6"/>')
    body.append(f'<circle cx="85" cy="86" r="50" fill="url(#gM)" stroke="{GOLD[3]}" stroke-width="4"/>')
    body.append(f'<circle cx="85" cy="86" r="41" fill="none" stroke="{GOLD[2]}" stroke-width="2.5" stroke-dasharray="3 3"/>')
    body.append('<ellipse cx="66" cy="62" rx="18" ry="9" fill="#ffffff" opacity="0.55" transform="rotate(-35 66 62)" filter="url(#b1)"/>')
    svg("ui_rosette", W, H, defs, "".join(body), size=210)


def stamp(name, ramp):
    """Postage stamp: perforated paper edge, coloured field, fine frame."""
    W, H = 240, 96
    l, m, d, line = ramp
    defs = vgrad("gS", ramp) + blur("sh", 2.6)
    # perforated outline: rectangle with semicircle bites
    x0, y0, x1, y1, r = 8, 8, 232, 86, 4.2
    holes = []
    nx, ny = 22, 8
    for i in range(nx + 1):
        x = x0 + (x1 - x0) * i / nx
        holes.append(f'<circle cx="{x:.1f}" cy="{y0}" r="{r}"/><circle cx="{x:.1f}" cy="{y1}" r="{r}"/>')
    for j in range(ny + 1):
        y = y0 + (y1 - y0) * j / ny
        holes.append(f'<circle cx="{x0}" cy="{y:.1f}" r="{r}"/><circle cx="{x1}" cy="{y:.1f}" r="{r}"/>')
    defs += f'<mask id="perf"><rect x="0" y="0" width="{W}" height="{H}" fill="white"/><g fill="black">{"".join(holes)}</g></mask>'
    body = []
    body.append(f'<g mask="url(#perf)"><rect x="{x0}" y="{y0 + 3}" width="{x1 - x0}" height="{y1 - y0}" fill="#1d2a4a" opacity="0.35" filter="url(#sh)"/>'
                f'<rect x="{x0}" y="{y0}" width="{x1 - x0}" height="{y1 - y0}" fill="#fffaf0"/></g>')
    body.append(f'<rect x="{x0 + 9}" y="{y0 + 9}" width="{x1 - x0 - 18}" height="{y1 - y0 - 18}" rx="3" fill="url(#gS)" stroke="{line}" stroke-width="3"/>')
    body.append(f'<rect x="{x0 + 14}" y="{y0 + 14}" width="{x1 - x0 - 28}" height="{y1 - y0 - 28}" rx="2" fill="none" stroke="{l}" stroke-width="1.5" opacity="0.8"/>')
    svg(name, W, H, defs, "".join(body), size=240)


# ---------------------------------------------------------------------------
# Button, dial, needle, small pieces
# ---------------------------------------------------------------------------

def button(name, ramp):
    W, H = 300, 100
    l, m, d, line = ramp
    defs = vgrad("gB", ramp, (0, 0.5, 1)) + blur("sh", 3.5) + blur("b1", 1.5)
    body = []
    body.append('<rect x="10" y="22" width="280" height="72" rx="36" fill="#1d2a4a" opacity="0.35" filter="url(#sh)"/>')
    body.append(f'<rect x="8" y="16" width="284" height="72" rx="36" fill="{line}"/>')  # lip
    body.append(f'<rect x="8" y="6" width="284" height="72" rx="36" fill="url(#gB)" stroke="{line}" stroke-width="4"/>')
    body.append(f'<rect x="26" y="12" width="248" height="26" rx="13" fill="#ffffff" opacity="0.35" filter="url(#b1)"/>')
    body.append(f'<rect x="14" y="10" width="272" height="64" rx="32" fill="none" stroke="{l}" stroke-width="2" opacity="0.6"/>')
    svg(name, W, H, defs, "".join(body), size=300)


def dial():
    W, H = 320, 176
    cx, cy = 160, 152
    defs = (vgrad("gF", PAPER, (0, 0.6, 1)) + vgrad("gRim", BERRY) + blur("sh", 3.5)
            + '<radialGradient id="gFace" cx="0.5" cy="0.9" r="0.9"><stop offset="0" stop-color="#fffdf6"/><stop offset="0.7" stop-color="#fbefd6"/><stop offset="1" stop-color="#ecd3a4"/></radialGradient>')

    def arc_band(a0, a1, ro, ri, fill, stroke=None, sw=0):
        p0 = math.pi * (1 - a0); p1 = math.pi * (1 - a1)
        large = 1 if abs(a1 - a0) > 0.5 else 0
        o0 = (cx + ro * math.cos(p0), cy - ro * math.sin(p0)); o1 = (cx + ro * math.cos(p1), cy - ro * math.sin(p1))
        i1 = (cx + ri * math.cos(p1), cy - ri * math.sin(p1)); i0 = (cx + ri * math.cos(p0), cy - ri * math.sin(p0))
        d = (f"M{o0[0]:.2f},{o0[1]:.2f} A{ro},{ro} 0 {large} 1 {o1[0]:.2f},{o1[1]:.2f} L{i1[0]:.2f},{i1[1]:.2f} "
             f"A{ri},{ri} 0 {large} 0 {i0[0]:.2f},{i0[1]:.2f} Z")
        st = f' stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round"' if stroke else ""
        return f'<path d="{d}" fill="{fill}"{st}/>'

    def half(r, extra):
        return f"M{cx - r},{cy + extra} L{cx - r},{cy} A{r},{r} 0 0 1 {cx + r},{cy} L{cx + r},{cy + extra} Z"

    body = []
    body.append(f'<path d="{half(146, 20)}" fill="#1d2a4a" opacity="0.38" transform="translate(0 5)" filter="url(#sh)"/>')
    body.append(f'<path d="{half(146, 20)}" fill="url(#gRim)" stroke="{BERRY[3]}" stroke-width="4" stroke-linejoin="round"/>')
    # rim studs
    for i in range(13):
        a = math.pi * i / 12
        x = cx - 137 * math.cos(a); y = cy - 137 * math.sin(a)
        body.append(f'<circle cx="{x:.1f}" cy="{y + 1:.1f}" r="3.6" fill="{BERRY[3]}"/><circle cx="{x:.1f}" cy="{y:.1f}" r="3.2" fill="#ffe08a"/><circle cx="{x - 1:.1f}" cy="{y - 1:.1f}" r="1.2" fill="#ffffff"/>')
    body.append(f'<path d="{half(128, 18)}" fill="url(#gFace)" stroke="{PAPER[3]}" stroke-width="3" stroke-linejoin="round"/>')
    # Wedge bands with their own top-lit paint.
    R0, R1 = 118, 88
    body.append(arc_band(0, 1, R0 + 3, R1 - 3, "#5a3a22"))
    defs += vgrad("gPine", PINE) + vgrad("gAmb", ("#ffd68a", "#f2a43a", "#c4721a")) + vgrad("gGold", GOLD)
    body.append(arc_band(0.0, 0.5, R0, R1, "url(#gPine)"))
    body.append(arc_band(0.5, 0.82, R0, R1, "url(#gAmb)"))
    for i in range(12):
        a0 = 0.82 + 0.18 * i / 12; a1 = 0.82 + 0.18 * (i + 1) / 12
        body.append(arc_band(a0, a1, R0, R1, "url(#gGold)" if i % 2 == 0 else "#fff7e6"))
    # inner highlight and ticks
    body.append(arc_band(0, 1, R0 - 1, R0 - 5, "#ffffff", None) .replace('fill="#ffffff"', 'fill="#ffffff" opacity="0.35"'))
    for k in range(0, 21):
        v = k / 20
        p = math.pi * (1 - v)
        r_in = R1 - (12 if k % 5 == 0 else 6)
        x0, y0 = cx + (R1 - 3) * math.cos(p), cy - (R1 - 3) * math.sin(p)
        x1, y1 = cx + r_in * math.cos(p), cy - r_in * math.sin(p)
        body.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke="#7a5a3a" stroke-width="{2.6 if k % 5 == 0 else 1.5}" stroke-linecap="round"/>')
    for k in (0.5, 0.82):
        p = math.pi * (1 - k)
        body.append(f'<line x1="{cx + (R0 + 3) * math.cos(p):.1f}" y1="{cy - (R0 + 3) * math.sin(p):.1f}" x2="{cx + (R1 - 3) * math.cos(p):.1f}" y2="{cy - (R1 - 3) * math.sin(p):.1f}" stroke="#5a3a22" stroke-width="3.5"/>')
    # a little star at the sweet spot
    p = math.pi * (1 - 0.91)
    sx_, sy_ = cx + (R0 + 16) * math.cos(p), cy - (R0 + 16) * math.sin(p)
    body.append(f'<polygon points="{star_pts(sx_, sy_, 10, 4.4)}" fill="#fff1a6" stroke="#8a520c" stroke-width="2" stroke-linejoin="round"/>')
    svg("ui_dial", W, H, defs, "".join(body), size=320)


def needle():
    W, H = 28, 124
    defs = '<linearGradient id="gN" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#ffffff"/><stop offset="0.6" stop-color="#f4ecdf"/><stop offset="1" stop-color="#c9bba8"/></linearGradient>' + blur("sh", 1.6)
    body = []
    d = "M14,8 L20,104 Q14,116 8,104 Z"
    body.append(f'<path d="{d}" fill="#1d2a4a" opacity="0.35" transform="translate(2 3)" filter="url(#sh)"/>')
    body.append(f'<path d="{d}" fill="url(#gN)" stroke="{BERRY[3]}" stroke-width="2.5" stroke-linejoin="round"/>')
    defs += f'<clipPath id="cN"><path d="{d}"/></clipPath>'
    stripes = "".join(f'<path d="M0,{y} L28,{y - 10} L28,{y - 3} L0,{y + 7} Z" fill="#e2373f"/>' for y in range(14, 120, 15))
    body.append(f'<g clip-path="url(#cN)">{stripes}<rect x="15" y="0" width="13" height="124" fill="#7a1018" opacity="0.18"/></g>')
    body.append(f'<circle cx="14" cy="110" r="12" fill="url(#gHub)" stroke="#8a520c" stroke-width="2.5"/><circle cx="11" cy="107" r="3.5" fill="#ffffff" opacity="0.8"/>')
    defs += rgrad("gHub", GOLD, 0.35, 0.3, 0.8)
    svg("ui_needle", W, H, defs, "".join(body), size=124)


def holly():
    W, H = 96, 64
    defs = vgrad("gL", PINE) + rgrad("gB", BERRY, 0.35, 0.3, 0.8) + blur("sh", 1.8)

    def leaf(cx, cy, rot, s=1):
        d = ("M0,0 C6,-8 12,-6 16,-12 C18,-6 24,-6 28,-10 C28,-4 34,-2 40,-4 C36,2 38,8 42,10 "
             "C34,10 30,14 28,18 C24,12 18,14 14,18 C14,10 6,8 0,0 Z")
        return (f'<g transform="translate({cx} {cy}) rotate({rot}) scale({s})">'
                f'<path d="{d}" fill="#1d2a4a" opacity="0.3" transform="translate(1 3)" filter="url(#sh)"/>'
                f'<path d="{d}" fill="url(#gL)" stroke="{PINE[3]}" stroke-width="2" stroke-linejoin="round"/>'
                f'<path d="M3,2 C14,4 26,4 40,4" fill="none" stroke="{PINE[3]}" stroke-width="1.6"/>'
                f'<path d="M6,-1 C14,-3 20,-3 30,-3" fill="none" stroke="#bff0c6" stroke-width="1.4" opacity="0.7"/></g>')
    body = leaf(46, 34, 200) + leaf(48, 30, -20)
    for (bx, by) in ((44, 34), (54, 38), (50, 26)):
        body += f'<circle cx="{bx}" cy="{by}" r="7.5" fill="url(#gB)" stroke="{BERRY[3]}" stroke-width="2"/><circle cx="{bx - 2.5}" cy="{by - 2.5}" r="2" fill="#ffffff" opacity="0.8"/>'
    svg("ui_holly", W, H, defs, body, size=96)


def snowcap(name, W, H, seed):
    random.seed(seed)
    defs = vgrad("gS", SNOW, (0, 0.6, 1)) + blur("sh", 1.8)
    # Drift along the top edge with rounded lumps and a few icicle drips.
    top = []
    x = 0
    while x <= W:
        top.append((x, 10 + random.uniform(-6, 2) * (1 if 6 < x < W - 6 else 0.2)))
        x += random.uniform(W / 30, W / 14)
    top.append((W, 14))
    bot = []
    x = W
    while x >= 0:
        y = H * 0.55 + random.uniform(-3, 3)
        if random.random() < 0.3 and 12 < x < W - 12:
            bot += [(x, y), (x - 4, H - random.uniform(0, 6)), (x - 8, y)]
            x -= 8
        else:
            bot.append((x, y))
        x -= random.uniform(W / 40, W / 18)
    bot.append((0, H * 0.5))
    pts = top + bot
    d = f"M{pts[0][0]:.1f},{pts[0][1]:.1f} " + " ".join(f"L{a:.1f},{b:.1f}" for a, b in pts[1:]) + " Z"
    body = (f'<path d="{d}" fill="#1d2a4a" opacity="0.25" transform="translate(0 3)" filter="url(#sh)"/>'
            f'<path d="{d}" fill="url(#gS)" stroke="{SNOW[3]}" stroke-width="2.2" stroke-linejoin="round"/>'
            f'<path d="M6,{12:.0f} ' + " ".join(f"L{a:.1f},{b + 3:.1f}" for a, b in top[1:-1]) + '" fill="none" stroke="#ffffff" stroke-width="2.5" opacity="0.9"/>')
    svg(name, W, H, defs, body, size=W)


def burst():
    """Gold starburst behind the combo count."""
    W = H = 120
    defs = rgrad("gB", ("#fff6c0", "#ffc93c", "#f08a1c"), 0.5, 0.5, 0.6) + blur("sh", 2.5)
    pts = []
    n = 14
    random.seed(9)
    for i in range(n * 2):
        a = i / (n * 2) * math.tau
        r = (54 + random.uniform(-3, 3)) if i % 2 == 0 else 38
        pts.append(f"{60 + r * math.cos(a):.1f},{60 + r * math.sin(a):.1f}")
    p = " ".join(pts)
    body = (f'<polygon points="{p}" fill="#1d2a4a" opacity="0.35" transform="translate(0 4)" filter="url(#sh)"/>'
            f'<polygon points="{p}" fill="url(#gB)" stroke="#a5450c" stroke-width="3.5" stroke-linejoin="round"/>'
            '<circle cx="60" cy="60" r="34" fill="none" stroke="#fff6c0" stroke-width="2" opacity="0.7"/>')
    svg("ui_burst", W, H, defs, body, size=120)


# ---------------------------------------------------------------------------
# Icons, 64x64
# ---------------------------------------------------------------------------

def icon(name, defs, body):
    svg(name, 64, 64, defs + blur("ish", 1.6), body, size=64)


def icons():
    sh = lambda d, dy=3: f'<path d="{d}" fill="#1d2a4a" opacity="0.3" transform="translate(1 {dy})" filter="url(#ish)"/>'
    # Distance flag: pole and berry pennant.
    flag = "M20,10 L52,20 L20,32 Z"
    icon("ui_ic_flag", vgrad("gF", BERRY) + vgrad("gP", WOOD),
         sh(flag) + f'<rect x="14" y="6" width="7" height="52" rx="3.5" fill="url(#gP)" stroke="{WOOD[3]}" stroke-width="2"/>'
         f'<path d="{flag}" fill="url(#gF)" stroke="{BERRY[3]}" stroke-width="2.5" stroke-linejoin="round"/>'
         '<path d="M24,14 L40,19" stroke="#ffd0c8" stroke-width="2" stroke-linecap="round"/>'
         f'<circle cx="17.5" cy="7" r="4.5" fill="#ffe08a" stroke="{GOLD[3]}" stroke-width="2"/>'
         f'<ellipse cx="18" cy="58" rx="14" ry="4" fill="{SNOW[1]}" stroke="{SNOW[3]}" stroke-width="2"/>')
    # FLAP: a floppy brown ear lifted like a wing, with motion arcs.
    ear = "M14,40 C10,24 22,8 40,8 C52,10 56,20 50,28 C44,36 36,40 30,52 C24,54 16,50 14,40 Z"
    icon("ui_ic_ear", vgrad("gE", ("#e7a463", "#c4733a", "#8a4a20")),
         sh(ear) + f'<path d="{ear}" fill="url(#gE)" stroke="#5e2f12" stroke-width="2.5" stroke-linejoin="round"/>'
         '<path d="M22,38 C22,26 30,16 42,14 C48,16 48,22 44,26 C38,30 32,34 28,44 Z" fill="#f2c9a0" opacity="0.8"/>'
         f'<path d="M50,40 Q58,44 58,52 M44,48 Q50,52 50,58" fill="none" stroke="{ICE[3]}" stroke-width="3" stroke-linecap="round"/>')
    # Altitude: snowy peak with an up arrow.
    mtn = "M4,56 L26,20 L36,34 L44,26 L60,56 Z"
    icon("ui_ic_alt", vgrad("gM", ("#a7b6d6", "#7489b5", "#4e5f8a")) + vgrad("gA", PINE),
         sh(mtn) + f'<path d="{mtn}" fill="url(#gM)" stroke="#36446a" stroke-width="2.5" stroke-linejoin="round"/>'
         '<path d="M18,33 L26,20 L33,30 L29,28 L25,33 L21,30 Z" fill="#ffffff" stroke="#8fb0d0" stroke-width="1.5" stroke-linejoin="round"/>'
         f'<path d="M46,4 L58,18 L51,18 L51,30 L41,30 L41,18 L34,18 Z" fill="url(#gA)" stroke="{PINE[3]}" stroke-width="2.5" stroke-linejoin="round"/>')
    # Star.
    icon("ui_ic_star", rgrad("gS", GOLD, 0.4, 0.3, 0.8),
         f'<polygon points="{star_pts(32, 35, 27, 12)}" fill="#1d2a4a" opacity="0.3" filter="url(#ish)"/>'
         f'<polygon points="{star_pts(32, 32, 27, 12)}" fill="url(#gS)" stroke="{GOLD[3]}" stroke-width="3" stroke-linejoin="round"/>'
         '<ellipse cx="25" cy="24" rx="6" ry="3.5" fill="#ffffff" opacity="0.7" transform="rotate(-30 25 24)"/>')
    # Restart: circular arrow in paper white.
    arc = "M48,22 A18,18 0 1 0 50,38"
    icon("ui_ic_restart", "",
         f'<path d="{arc}" fill="none" stroke="#0d4325" stroke-width="12" stroke-linecap="round"/>'
         f'<path d="{arc}" fill="none" stroke="#fff8ec" stroke-width="7" stroke-linecap="round"/>'
         '<path d="M38,8 L56,10 L52,28 Z" fill="#fff8ec" stroke="#0d4325" stroke-width="3" stroke-linejoin="round"/>')
    # Trophy.
    cup = "M18,8 L46,8 L44,28 C42,36 36,40 32,40 C28,40 22,36 20,28 Z"
    icon("ui_ic_trophy", vgrad("gT", GOLD),
         sh(cup) + f'<path d="M18,12 C6,12 8,28 20,28 M46,12 C58,12 56,28 44,28" fill="none" stroke="{GOLD[3]}" stroke-width="4"/>'
         f'<path d="{cup}" fill="url(#gT)" stroke="{GOLD[3]}" stroke-width="2.5" stroke-linejoin="round"/>'
         f'<rect x="28" y="40" width="8" height="8" fill="{GOLD[2]}" stroke="{GOLD[3]}" stroke-width="2"/>'
         f'<rect x="18" y="48" width="28" height="9" rx="2" fill="url(#gT)" stroke="{GOLD[3]}" stroke-width="2.5"/>'
         '<path d="M24,13 L26,28" stroke="#ffffff" stroke-width="3" stroke-linecap="round" opacity="0.7"/>')
    # Speed: a rocket-ish chevron with streaks.
    icon("ui_ic_speed", vgrad("gS", BERRY),
         f'<path d="M10,20 L26,20 M6,32 L24,32 M10,44 L26,44" stroke="{ICE[3]}" stroke-width="4" stroke-linecap="round"/>'
         + sh("M28,14 L58,32 L28,50 L34,32 Z") +
         f'<path d="M28,14 L58,32 L28,50 L34,32 Z" fill="url(#gS)" stroke="{BERRY[3]}" stroke-width="2.5" stroke-linejoin="round"/>'
         '<path d="M32,20 L50,31" stroke="#ffd0c8" stroke-width="2.4" stroke-linecap="round"/>')
    # Bounce: ball over a spring arc.
    icon("ui_ic_bounce", rgrad("gB", ICE, 0.35, 0.3, 0.8),
         f'<path d="M6,56 Q14,40 22,56 Q30,40 38,56 Q46,40 54,56" fill="none" stroke="{ICE[3]}" stroke-width="3" stroke-linecap="round" stroke-dasharray="3 5"/>'
         '<circle cx="34" cy="26" r="15" fill="#1d2a4a" opacity="0.3" transform="translate(1 3)" filter="url(#ish)"/>'
         f'<circle cx="34" cy="26" r="15" fill="url(#gB)" stroke="{ICE[3]}" stroke-width="2.5"/>'
         f'<path d="M20,26 Q34,34 48,26" fill="none" stroke="{BERRY[1]}" stroke-width="3"/>'
         '<circle cx="28" cy="20" r="4" fill="#ffffff" opacity="0.8"/>')
    # Jump: a kicker ramp with an arc and a star at the top.
    icon("ui_ic_jump", vgrad("gW", WOOD),
         sh("M4,56 L30,56 C24,46 14,40 4,38 Z") +
         f'<path d="M4,56 L30,56 C24,46 14,40 4,38 Z" fill="url(#gW)" stroke="{WOOD[3]}" stroke-width="2.5" stroke-linejoin="round"/>'
         f'<path d="M26,44 Q40,6 58,30" fill="none" stroke="{ICE[3]}" stroke-width="3" stroke-dasharray="4 4" stroke-linecap="round"/>'
         f'<polygon points="{star_pts(54, 24, 9, 4)}" fill="#ffd24a" stroke="{GOLD[3]}" stroke-width="2" stroke-linejoin="round"/>')
    # Push: a power bolt.
    bolt = "M36,4 L14,36 L30,36 L24,60 L50,24 L34,24 Z"
    icon("ui_ic_push", vgrad("gP", GOLD),
         sh(bolt) + f'<path d="{bolt}" fill="url(#gP)" stroke="{GOLD[3]}" stroke-width="2.5" stroke-linejoin="round"/>'
         '<path d="M34,10 L20,32" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" opacity="0.7"/>')
    # Lock / key hint: a pointing hand is too fiddly; a gift for presents.
    icon("ui_ic_gift", vgrad("gG", BERRY) + vgrad("gR", GOLD),
         sh("M10,26 L54,26 L54,58 L10,58 Z") +
         f'<rect x="10" y="26" width="44" height="32" rx="3" fill="url(#gG)" stroke="{BERRY[3]}" stroke-width="2.5"/>'
         f'<rect x="7" y="18" width="50" height="12" rx="3" fill="url(#gG)" stroke="{BERRY[3]}" stroke-width="2.5"/>'
         f'<rect x="28" y="18" width="8" height="40" fill="url(#gR)" stroke="{GOLD[3]}" stroke-width="1.5"/>'
         f'<path d="M32,18 C22,4 12,10 20,17 Z M32,18 C42,4 52,10 44,17 Z" fill="url(#gR)" stroke="{GOLD[3]}" stroke-width="2" stroke-linejoin="round"/>')


def main():
    os.makedirs(OUT, exist_ok=True)
    logo()
    panel("ui_panel", PAPER, PAPER[3], "#d65a5a", "#e8b84a")
    panel("ui_panel_gold", ("#fffbea", "#fff1c8", "#f3d48a"), "#b07a14", "#e2a72a", "#e2373f", thick=5.0)
    chip()
    dark_chip()
    for n, r in (("red", BERRY), ("green", PINE), ("gold", GOLD), ("ice", ICE)):
        ribbon("ui_ribbon_" + n, r)
    seal("ui_seal_red", BERRY)
    seal("ui_seal_green", PINE)
    seal("ui_seal_gold", GOLD)
    medal("ui_medal_gold", GOLD, BERRY, PINE)
    medal("ui_medal_silver", SILVER, ICE, BERRY)
    medal("ui_medal_bronze", BRONZE, PINE, BERRY)
    rosette()
    stamp("ui_stamp_red", BERRY)
    stamp("ui_stamp_gold", GOLD)
    stamp("ui_stamp_green", PINE)
    button("ui_button", PINE)
    button("ui_button_off", SILVER)
    dial()
    needle()
    holly()
    snowcap("ui_snow_wide", 720, 44, 11)
    snowcap("ui_snow_small", 260, 30, 12)
    burst()
    icons()


if __name__ == "__main__":
    main()
