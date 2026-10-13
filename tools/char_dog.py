"""The three dog pilots, drawn for the 2026-10 character pass.

Buddy (key "biscuit") is an orange-and-white beagle: smooth domed skull,
white blaze and muzzle, droopy flews, long soft ears, brown eyes.
Nanook is a husky: charcoal-blue cap with a widow's peak, white mask and
brow spots, ice-blue eyes, a fluffy cheek ruff, pointed fluffy ears and a
plume tail. Pixie is a Yorkshire terrier: golden face with a scruffy fringe,
a topknot tied with a pink bow, a beard, steel-blue saddle, feathered ears.

Heads are 256x256, three-quarter view facing right; the game seats the head
at the canvas centre and pivots the ears at (68,78) and (188,78) (draw.rs
head_seat / ears). Painted with brush.Art.paint / overlay: lit gradient,
outline in a deeper shade of each fill, core shadow, bounce and rim light.
"""
import math
from brush import (Art, ellipse, circle, tone, mix, deep, shape, xform, fur_marks, bent_limb, shift_path)

LINE = "#3b1f16"      # facial feature lines (mouth, lids) on warm dogs

BREEDS = {
    "biscuit": dict(kind="beagle", fur="#ec8a32", dark="#b85a1c", white="#fffaf1", ear="#c8692a", ear_in="#9c4a1e",
                    iris=("#2a1206", "#7a3f18", "#d99550"), nose="#2c1a16", collar="#e0392f", tag="#ffcc33",
                    brow="#a2521e", line="#3b1f16", body="#ec8a32", saddle="#d8742a", leg_top="#ec8a32", sock="#fffaf1"),
    "nanook":  dict(kind="husky", fur="#6f7c99", dark="#464f68", white="#ffffff", ear="#5d6886", ear_in="#f2f4fa",
                    iris=("#0c3a74", "#2a86dc", "#a6ecff"), nose="#1f2028", collar="#3a86d6", tag="#bfe6ff",
                    brow="#3c4560", line="#262a3a", body="#6f7c99", saddle="#4f5a76", leg_top="#6f7c99", sock="#ffffff"),
    "pixie":   dict(kind="terrier", fur="#e7ad5c", dark="#b47a36", white="#f9dca6", ear="#d99a48", ear_in="#f6c98a",
                    iris=("#1e0d06", "#5e3216", "#c08040"), nose="#2a1a14", collar="#ff4f9a", tag="#ffd23a",
                    brow="#fff0c8", line="#3a2014", lashes=True, body="#7483a2", saddle="#5d6a86", leg_top="#e7ad5c", sock="#f2c47c"),
}

SCLERA = "#ffffff"

# ---------------------------------------------------------------- head silhouettes
# Clockwise point lists; tuft specs per segment index: (count, depth, lean).
HEADS = {
    "beagle": ([(60, 48), (98, 22), (146, 20), (182, 34), (200, 56), (206, 72), (226, 80), (248, 84), (262, 96), (264, 116),
                (254, 130), (238, 138), (232, 156), (216, 170), (192, 176), (164, 184), (130, 192), (96, 188), (64, 174),
                (44, 150), (36, 116), (42, 80)],
               {16: (2, 7, 0.3), 17: (2, 8, 0.3), 18: (2, 6, 0.3)}),
    "husky":  ([(58, 48), (96, 22), (146, 18), (184, 32), (202, 54), (210, 70), (232, 76), (256, 84), (268, 96), (270, 114),
                (260, 126), (242, 132), (232, 146), (212, 158), (186, 166), (166, 182), (140, 196), (104, 200), (70, 188),
                (44, 164), (32, 128), (38, 84)],
               {14: (2, 8, 0.3), 15: (3, 12, 0.35), 16: (3, 13, 0.35), 17: (3, 13, 0.35), 18: (3, 11, 0.3), 19: (2, 8, 0.3)}),
    "terrier": ([(60, 50), (98, 26), (144, 24), (178, 38), (194, 58), (200, 74), (220, 82), (240, 88), (254, 100), (254, 116),
                 (246, 128), (244, 150), (230, 170), (206, 184), (178, 190), (150, 192), (118, 194), (84, 186), (54, 166),
                 (38, 130), (42, 86)],
                {10: (2, 8, 0.4), 11: (3, 12, 0.5), 12: (3, 13, 0.5), 13: (3, 11, 0.45), 14: (2, 9, 0.4), 16: (2, 8, 0.3),
                 17: (2, 9, 0.3), 18: (2, 9, 0.3), 19: (2, 7, 0.3)}),
}

# Where the features sit per breed: nose centre, near eye, far eye, mouth anchor.
FEAT = {
    "beagle":  dict(nose=(244, 100), ne=(136, 100, 26, 30), fe=(190, 88, 15, 25), mouth=(248, 122)),
    "husky":   dict(nose=(252, 98), ne=(136, 98, 25, 28), fe=(190, 86, 15, 24), mouth=(254, 120)),
    "terrier": dict(nose=(238, 104), ne=(134, 102, 27, 31), fe=(186, 92, 17, 27), mouth=(240, 124)),
}


# The head is drawn a little larger than the canvas allows and scaled into
# it, so the ear pivots of draw.rs land at (67,72) / (201,72) in head space.
HEAD_WRAP = "translate(128 128) scale(0.9) translate(-134 -128)"


def head_path(kind):
    pts, tf = HEADS[kind]
    return shape(pts, tf, seed=11)


# ---------------------------------------------------------------- eyes
def eye(art, c, cx, cy, rx, ry, look=(0, 0), pupil=0.5, iris=0.82, lid=0.0, tilt=0.0, lower=0.0,
        lidcol=None, lowcol=None, far=False, sparkle=1.0):
    """One big painted eye. `lid` 0..1 closes the upper lid (tilted by
    `tilt`, positive drops the inner corner), `lower` pushes a happy cheek
    up from below."""
    d = ellipse(cx, cy, rx, ry)
    cid = art.clip_id(d)
    gid = art.uid("sc")
    art.defs.append(f'<linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#b9c4e0"/>'
                    f'<stop offset="0.35" stop-color="#f4f7ff"/><stop offset="1" stop-color="#ffffff"/></linearGradient>')
    art.add(f'<path d="{d}" fill="url(#{gid})"/>')
    ir = min(rx, ry) * iris
    px, py = cx + look[0], cy + look[1]
    i0, i1, i2 = c["iris"]
    rg = art.uid("ir")
    art.defs.append(f'<radialGradient id="{rg}" cx="0.55" cy="0.72" r="0.62"><stop offset="0" stop-color="{i2}"/>'
                    f'<stop offset="0.55" stop-color="{i1}"/><stop offset="1" stop-color="{i0}"/></radialGradient>')
    g = f'<path d="{ellipse(px, py, ir, ir * 1.06)}" fill="url(#{rg})"/>'
    g += f'<path d="{ellipse(px, py, ir, ir * 1.06)}" fill="none" stroke="{i0}" stroke-width="{max(1.5, ir * 0.12):.1f}" opacity="0.8"/>'
    pr = ir * pupil
    g += f'<path d="{ellipse(px, py + pr * 0.04, pr, pr * 1.08)}" fill="#120806"/>'
    # Lid shadow across the top of the eyeball.
    g += f'<path d="{ellipse(cx, cy - ry * 0.95, rx * 1.3, ry * 0.55)}" fill="#24183a" opacity="0.32" filter="url(#{art.blur(3)})"/>'
    # Highlights: a big soft window, a crisp dot, a tiny bounce below.
    hs = ir * sparkle
    g += f'<path d="{ellipse(px - ir * 0.32, py - ir * 0.36, hs * 0.38, hs * 0.32)}" fill="#ffffff"/>'
    g += f'<path d="{circle(px + ir * 0.36, py + ir * 0.34, hs * 0.14)}" fill="#ffffff" opacity="0.95"/>'
    g += f'<path d="{ellipse(px + ir * 0.05, py + ir * 0.62, ir * 0.38, ir * 0.12)}" fill="{i2}" opacity="0.55" filter="url(#{art.blur(1.2)})"/>'
    lw = max(3.0, rx * 0.16)
    lid_line = ""
    if lid > 0:
        ly = cy - ry + lid * 2 * ry
        dl = -tilt * rx * (1 if not far else -1)   # inner corner: right for the near eye, left for the far one
        y_l, y_r = ly - dl, ly + dl
        col = lidcol or c["fur"]
        lp = (f"M{cx - rx - 6:.1f},{cy - ry - 8:.1f} L{cx + rx + 6:.1f},{cy - ry - 8:.1f} L{cx + rx + 6:.1f},{y_r:.1f} "
              f"Q{cx:.1f},{(y_l + y_r) / 2 + ry * 0.12:.1f} {cx - rx - 6:.1f},{y_l:.1f} Z")
        g += f'<path d="{lp}" fill="{col}"/>'
        g += f'<path d="{lp}" fill="none" stroke="#000" stroke-width="6" opacity="0.18" filter="url(#{art.blur(2)})" transform="translate(0 3)"/>'
        lid_line = f"M{cx - rx - 2:.1f},{y_l + (y_r - y_l) * -0.05:.1f} Q{cx:.1f},{(y_l + y_r) / 2 + ry * 0.12:.1f} {cx + rx + 2:.1f},{y_r:.1f}"
    if lower > 0:
        by = cy + ry - lower * 2 * ry
        col = lowcol or c["white"]
        bp = f"M{cx - rx - 6:.1f},{cy + ry + 8:.1f} L{cx - rx - 6:.1f},{by + 4:.1f} Q{cx:.1f},{by - ry * 0.25:.1f} {cx + rx + 6:.1f},{by + 4:.1f} L{cx + rx + 6:.1f},{cy + ry + 8:.1f} Z"
        g += f'<path d="{bp}" fill="{col}"/>'
        g += f'<path d="M{cx - rx:.1f},{by + 3:.1f} Q{cx:.1f},{by - ry * 0.25:.1f} {cx + rx:.1f},{by + 3:.1f}" fill="none" stroke="{c["line"]}" stroke-width="2.4" opacity="0.7"/>'
    art.add(f'<g clip-path="url(#{cid})">{g}</g>')
    ink = c["line"]
    art.stroke(d, ink, 3.0)
    # Heavy upper lash line with a flick at the outer corner.
    if lid_line:
        art.stroke(lid_line, ink, lw)
    else:
        a0, a1 = (200, 340)
        pts = [(cx + rx * math.cos(math.radians(a)), cy + ry * math.sin(math.radians(a))) for a in range(a0, a1 + 1, 20)]
        art.stroke("M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts), ink, lw)
    if not c.get("lashes"):
        return
    ox = cx + (rx * 0.92 if far else -rx * 0.92)
    oy = cy - ry * 0.42 + (lid * ry * 1.4 if lid else 0)
    fx = 9 if far else -9
    art.stroke(f"M{ox:.1f},{oy:.1f} q{fx * 0.6:.1f},-2 {fx:.1f},-8", ink, lw * 0.7)


def shut_eye(art, c, cx, cy, rx, ry, far=False, happy=False):
    """Squeezed shut: a thick > or < chevron (or a happy arc)."""
    ink = c["line"]
    if happy:
        art.stroke(f"M{cx - rx * 0.9:.1f},{cy + 4:.1f} Q{cx:.1f},{cy - ry * 0.75:.1f} {cx + rx * 0.9:.1f},{cy + 4:.1f}", ink, 6)
        return
    s = 1 if not far else -1
    art.stroke(f"M{cx - rx * 0.8 * s:.1f},{cy - ry * 0.55:.1f} Q{cx + rx * 0.2 * s:.1f},{cy - ry * 0.05:.1f} {cx + rx * 0.75 * s:.1f},{cy + 2:.1f} "
               f"Q{cx + rx * 0.2 * s:.1f},{cy + ry * 0.1:.1f} {cx - rx * 0.8 * s:.1f},{cy + ry * 0.55:.1f}", ink, 6)
    # Squeeze creases.
    art.stroke(f"M{cx - rx * 1.1 * s:.1f},{cy - ry * 0.2:.1f} l{-7 * s},-3 M{cx - rx * 1.1 * s:.1f},{cy + ry * 0.25:.1f} l{-7 * s},3", ink, 2.6, 0.7)


def spiral_eye(art, c, cx, cy, rx, ry, turn=0):
    d = ellipse(cx, cy, rx, ry)
    art.paint(d, SCLERA, w=3.0, ink=c["line"], k=6, hi=0, core=0.4, bounce=0, drop=0)
    pts = []
    for i in range(0, 80):
        a = math.radians(i * 13 + turn)
        r = min(rx, ry) * 0.85 * (1 - i / 86)
        pts.append((cx + math.cos(a) * r, cy + math.sin(a) * r * ry / rx * 0.95))
    art.stroke("M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts), "#3a2050", 3.2)


def brow(art, col, ink, cx, cy, L, rot, arch=8, t=9, far=False, tufty=False):
    """A tapered fur brow. `rot` > 0 lifts the inner end."""
    s = -1 if not far else 1    # inner end is +x for the near eye
    pts_top, pts_bot = [], []
    for i in range(7):
        u = i / 6 * 2 - 1
        th = t * (0.35 + 0.65 * (1 - abs(u) ** 1.6)) * (1.15 if u * -s > 0 else 0.9)
        x = cx + u * L
        y = cy - arch * (1 - u * u)
        pts_top.append((x, y - th / 2)); pts_bot.append((x, y + th / 2))
    ring = pts_top + pts_bot[::-1]
    ring = xform(ring, cx, cy, rot=rot * s)
    tufts = {i: (2, 4, 0.5) for i in range(0, 6)} if tufty else None
    art.paint(shape(ring, tufts, seed=5), col, w=2.6, ink=ink, k=4, hi=0.4, core=0.3, bounce=0, drop=0.6)


# ---------------------------------------------------------------- nose and mouths
def nose(art, c, nx, ny, s=1.0):
    d = shape([(nx - 18 * s, ny - 6 * s), (nx - 6 * s, ny - 13 * s), (nx + 12 * s, ny - 12 * s), (nx + 19 * s, ny - 3 * s),
               (nx + 12 * s, ny + 9 * s), (nx, ny + 14 * s), (nx - 12 * s, ny + 8 * s)])
    art.paint(d, c["nose"], w=3.2, ink="#0c0606", k=7, hi=0.25, core=0.5, bounce=0.7, bounce_col="#7d8fb8",
              lights=[(ellipse(nx - 4 * s, ny - 7 * s, 9 * s, 4 * s), "#ffffff", 0.85, 1.2)], light="#8a7a8a")
    art.stroke(f"M{nx - 9 * s:.1f},{ny + 2 * s:.1f} q4,-3 8,1 M{nx + 6 * s:.1f},{ny + 1 * s:.1f} q4,-2 7,2", "#000000", 2.6, 0.6)


def mouth(art, c, kind, mx, my, chin_col):
    """Mouth shapes, anchored at (mx, my) = just under the nose."""
    ink = c["line"]
    M = "#6a1418"
    TONGUE = "#ff7a86"
    art.stroke(f"M{mx},{my - 6} L{mx - 1},{my + 4}", ink, 3.4)   # philtrum
    def open_mouth(d, tongue, teeth=None, tongue_out=None):
        cid = art.clip_id(d)
        art.add(f'<path d="{d}" fill="{M}"/>')
        g = f'<path d="{tongue}" fill="{TONGUE}"/>'
        g += f'<path d="{tongue}" fill="none" stroke="#c83c52" stroke-width="2" opacity="0.6"/>'
        g += f'<path d="{shift_path(d, 0, -10)}" fill="none" stroke="#2a0608" stroke-width="10" opacity="0.4" filter="url(#{art.blur(3)})"/>'
        if teeth:
            g += f'<path d="{teeth}" fill="#ffffff" stroke="#b9c3da" stroke-width="1.5"/>'
        art.add(f'<g clip-path="url(#{cid})">{g}</g>')
        art.stroke(d, ink, 3.8)
        if tongue_out:
            art.paint(tongue_out, TONGUE, w=3, ink="#a8243c", k=6, hi=0.5, core=0.35, bounce=0, drop=0,
                      lights=[(ellipse(0, 0, 0, 0), "#fff", 0, 1)])
    if kind == "smile":
        # Happy open smile: a D-shaped mouth with the tongue, a cheek dimple.
        d = (f"M{mx - 44},{my + 8} Q{mx - 22},{my + 16} {mx - 1},{my + 4} C{mx + 2},{my + 26} {mx - 12},{my + 40} {mx - 25},{my + 39} "
             f"C{mx - 38},{my + 38} {mx - 46},{my + 24} {mx - 44},{my + 8} Z")
        open_mouth(d, ellipse(mx - 22, my + 38, 16, 10))
        art.stroke(f"M{mx - 50},{my + 2} Q{mx - 48},{my + 8} {mx - 43},{my + 11}", ink, 3.0)
    elif kind == "laugh":
        d = (f"M{mx - 48},{my + 4} Q{mx - 24},{my + 18} {mx},{my + 2} C{mx + 6},{my + 34} {mx - 10},{my + 52} {mx - 26},{my + 51} "
             f"C{mx - 44},{my + 50} {mx - 54},{my + 28} {mx - 48},{my + 4} Z")
        tongue_out = (f"M{mx - 34},{my + 40} C{mx - 36},{my + 60} {mx - 24},{my + 72} {mx - 12},{my + 66} C{mx - 2},{my + 60} {mx - 6},{my + 44} {mx - 12},{my + 40} Z")
        open_mouth(d, ellipse(mx - 24, my + 48, 18, 10), teeth=f"M{mx - 40},{my + 9} L{mx - 34},{my + 18} L{mx - 30},{my + 11} Z M{mx - 9},{my + 10} L{mx - 6},{my + 18} L{mx - 2},{my + 7} Z")
        art.paint(tongue_out, TONGUE, w=3, ink="#a8243c", k=6, hi=0.5, core=0.35, bounce=0, drop=0)
        art.stroke(f"M{mx - 23},{my + 46} L{mx - 21},{my + 60}", "#c83c52", 2.4, 0.8)
        art.stroke(f"M{mx - 54},{my - 2} Q{mx - 53},{my + 4} {mx - 48},{my + 8}", ink, 3.0)
    elif kind == "smirk":
        # Sure of itself: a lopsided grin with a row of teeth.
        d = f"M{mx - 46},{my + 12} Q{mx - 24},{my + 18} {mx - 1},{my + 2} C{mx},{my + 18} {mx - 12},{my + 26} {mx - 24},{my + 26} C{mx - 36},{my + 26} {mx - 44},{my + 20} {mx - 46},{my + 12} Z"
        open_mouth(d, ellipse(mx - 20, my + 30, 12, 6), teeth=f"M{mx - 46},{my + 8} Q{mx - 24},{my + 16} {mx},{my - 2} L{mx},{my + 12} Q{mx - 24},{my + 24} {mx - 46},{my + 16} Z")
        art.stroke(f"M{mx - 52},{my + 4} Q{mx - 50},{my + 12} {mx - 44},{my + 15}", ink, 3.0)
    elif kind == "clench":
        d = f"M{mx - 46},{my + 10} Q{mx - 24},{my + 6} {mx},{my + 4} Q{mx + 2},{my + 16} {mx - 2},{my + 26} Q{mx - 24},{my + 30} {mx - 44},{my + 24} Q{mx - 48},{my + 16} {mx - 46},{my + 10} Z"
        art.paint(d, "#ffffff", w=3.6, ink=ink, k=5, hi=0, core=0.5, bounce=0, drop=0)
        art.stroke(f"M{mx - 45},{my + 17} Q{mx - 22},{my + 18} {mx},{my + 15}", "#8a94b0", 2.2)
        art.stroke(f"M{mx - 34},{my + 9} L{mx - 34},{my + 27} M{mx - 22},{my + 7} L{mx - 22},{my + 28} M{mx - 10},{my + 6} L{mx - 10},{my + 27}", "#8a94b0", 2.0)
    elif kind == "wobble":
        d = f"M{mx - 38},{my + 18} Q{mx - 32},{my + 8} {mx - 24},{my + 16} Q{mx - 16},{my + 24} {mx - 8},{my + 14} Q{mx - 4},{my + 30} {mx - 22},{my + 32} Q{mx - 38},{my + 32} {mx - 38},{my + 18} Z"
        open_mouth(d, ellipse(mx - 22, my + 32, 10, 6))
    elif kind == "scream":
        d = (f"M{mx - 44},{my + 6} C{mx - 30},{my + 2} {mx - 10},{my - 2} {mx + 2},{my + 4} C{mx + 8},{my + 40} {mx - 6},{my + 66} {mx - 22},{my + 66} "
             f"C{mx - 40},{my + 66} {mx - 52},{my + 36} {mx - 44},{my + 6} Z")
        open_mouth(d, ellipse(mx - 22, my + 62, 20, 12), teeth=f"M{mx - 40},{my + 4} L{mx - 35},{my + 14} L{mx - 31},{my + 4} Z M{mx - 8},{my + 2} L{mx - 5},{my + 12} L{mx - 1},{my + 2} Z")
        # Uvula.
        art.fill(ellipse(mx - 20, my + 16, 4, 6), "#3a0408", 0.8)
    elif kind == "tongue":
        art.stroke(f"M{mx - 46},{my + 16} Q{mx - 30},{my + 22} {mx - 18},{my + 12} Q{mx - 8},{my + 6} {mx},{my + 4}", ink, 4)
        tongue = f"M{mx - 30},{my + 16} C{mx - 36},{my + 36} {mx - 26},{my + 52} {mx - 14},{my + 48} C{mx - 4},{my + 44} {mx - 8},{my + 26} {mx - 14},{my + 13} Z"
        art.paint(tongue, TONGUE, w=3, ink="#a8243c", k=6, hi=0.5, core=0.35, bounce=0, drop=0)
        art.stroke(f"M{mx - 21},{my + 22} L{mx - 19},{my + 40}", "#c83c52", 2.4, 0.8)


# ---------------------------------------------------------------- the head
FACES = ("idle", "scream", "panic", "squint", "dizzy", "grit", "whee")

EXPR = {
    #          look      pupil iris  lid   tilt  lower brows(lift, rot)  mouth     scale
    "idle":   ((4, 2),   0.50, 0.84, 0.0,  0.0,  0.14, (0, 4),        "smile",  1.0),
    "whee":   ((5, -4),  0.52, 0.86, 0.0,  0.0,  0.22, (-10, 6),      "laugh",  1.04),
    "grit":   ((7, 2),   0.52, 0.86, 0.30, 0.32, 0.16, (6, -20),      "smirk",  1.0),
    "panic":  ((-3, 2),  0.30, 0.56, 0.0,  0.0,  0.0,  (-12, 16),     "wobble", 1.08),
    "scream": ((0, 0),   0.26, 0.50, 0.0,  0.0,  0.0,  (-18, 8),      "scream", 1.12),
    "squint": (None,     0,    0,    0,    0,    0,    (6, -16),      "clench", 1.0),
    "dizzy":  (None,     0,    0,    0,    0,    0,    (-6, 0),       "tongue", 1.0),
}


def head_svg(c, face):
    kind = c["kind"]
    art = Art()
    H = head_path(kind)
    f = FEAT[kind]
    fur, white = c["fur"], c["white"]
    ink = deep(c["dark"])
    hid = art.clip_id(H)
    # Drop line (heavier outline on the shadow side), then the fur base.
    art.add(f'<path d="{shift_path(H, 1.4, 2.4)}" fill="{ink}"/>')
    art.paint(H, fur, w=0, k=16, hi=0, core=0, bounce=0, drop=0)
    def inside(fn):
        start = len(art.body); fn(); inner = "".join(art.body[start:]); del art.body[start:]
        art.add(f'<g clip-path="url(#{hid})">{inner}</g>')
    nx, ny = f["nose"]
    if kind == "beagle":
        blaze = shape([(146, 8), (158, 8), (164, 40), (170, 66), (176, 96), (186, 120), (208, 110), (226, 84), (270, 78),
                       (270, 270), (70, 270), (96, 196), (124, 176), (150, 160), (162, 136), (164, 110), (160, 70), (152, 40)],
                      {11: (2, 7, 0.3), 12: (2, 7, 0.3), 13: (2, 6, 0.3)}, seed=4)
        def m():
            art.paint(blaze, white, w=0, k=10, hi=0, core=0, bounce=0, drop=0)
            # Darker red-brown on the crown and the back of the skull.
            art.blend(ellipse(70, 70, 60, 46), c["dark"], 0.55, 14, "multiply")
            art.add(f'<path d="{fur_marks(80, 80, 44, 44, 26, ang=120, ln=10, seed=2)}" fill="none" stroke="{c["dark"]}" stroke-width="2.4" stroke-linecap="round" opacity="0.55"/>')
            art.add(f'<path d="{fur_marks(110, 50, 40, 18, 12, ang=160, ln=9, seed=5)}" fill="none" stroke="#ffd6a0" stroke-width="2.2" stroke-linecap="round" opacity="0.55"/>')
        inside(m)
    elif kind == "husky":
        mask = shape([(20, 152), (50, 138), (80, 126), (98, 104), (104, 76), (118, 58), (140, 52), (158, 60), (166, 90),
                      (174, 60), (194, 50), (212, 62), (222, 80), (240, 74), (280, 74), (280, 280), (10, 280)],
                     {0: (2, 8, 0.3), 1: (2, 8, 0.3), 2: (2, 6, 0.3)}, seed=7)
        stripe = shape([(158, 24), (174, 24), (176, 60), (170, 92), (164, 64)])
        def m():
            art.paint(mask, white, w=0, k=10, hi=0, core=0, bounce=0, drop=0)
            art.paint(stripe, c["fur"], w=0, hi=0, core=0, bounce=0, drop=0)
            # Brow spots above each eye, the husky trademark.
            art.paint(ellipse(126, 50, 13, 8), white, w=0, hi=0, core=0, bounce=0, drop=0)
            art.paint(ellipse(196, 42, 9, 6), white, w=0, hi=0, core=0, bounce=0, drop=0)
            # Dark cap shading and fur on the crown.
            art.blend(ellipse(80, 60, 70, 40), c["dark"], 0.5, 14, "multiply")
            art.add(f'<path d="{fur_marks(84, 70, 46, 34, 26, ang=130, ln=10, seed=3)}" fill="none" stroke="{c["dark"]}" stroke-width="2.4" stroke-linecap="round" opacity="0.6"/>')
            art.add(f'<path d="{fur_marks(80, 160, 40, 26, 18, ang=140, ln=11, seed=8)}" fill="none" stroke="#b9c6e0" stroke-width="2.2" stroke-linecap="round" opacity="0.8"/>')
        inside(m)
    else:  # terrier
        muzzle = shape([(176, 120), (196, 100), (220, 92), (262, 94), (262, 230), (170, 230), (164, 160)],
                       {5: (2, 8, 0.4), 6: (2, 8, 0.4)}, seed=3)
        def m():
            # Golden face, lighter muzzle, darker steel at the back of the skull.
            art.paint(muzzle, white, w=0, hi=0, core=0, bounce=0, drop=0)
            art.blend(ellipse(48, 120, 30, 60), c["dark"], 0.5, 14, "multiply")
            art.add(f'<path d="{fur_marks(90, 130, 44, 50, 34, ang=110, ln=12, seed=4)}" fill="none" stroke="{c["dark"]}" stroke-width="2.4" stroke-linecap="round" opacity="0.6"/>')
            art.add(f'<path d="{fur_marks(200, 150, 30, 24, 16, ang=100, ln=12, seed=9)}" fill="none" stroke="#c99a58" stroke-width="2.2" stroke-linecap="round" opacity="0.7"/>')
        inside(m)
    # Muzzle form: light along the bridge, shade under the eye and the jaw.
    cl = art.clip_id(H)
    art.blend(ellipse(nx - 26, ny - 8, 30, 9), "#ffffff", 0.6, 6, "screen", cl)
    art.blend(ellipse(150, 146, 30, 16), "#7c84c4", 0.35, 10, "multiply", cl)
    art.blend(ellipse(196, 188, 60, 16), "#7c84c4", 0.5, 9, "multiply", cl)
    # Muzzle pad line (where the snout meets the cheek).
    art.stroke(f"M{nx - 44},{ny - 20} Q{nx - 54},{ny + 4} {nx - 44},{ny + 30}", "#9aa6c6", 3, 0.45)
    art.overlay(H, k=18, hi=0.55, core=0.42, bounce=0.6, rim=0.45)
    art.stroke(H, ink, 4.6)
    if kind == "terrier":
        terrier_hair(art, c)
    E = EXPR[face]
    look, pupil, iris_s, lid, tilt, lower, (blift, brot), mk, sc = E
    ex, ey, erx, ery = f["ne"]; fx, fy, frx, fry = f["fe"]
    lidc_near = c["white"] if kind == "husky" else c["fur"]
    lidc_far = c["white"] if kind == "husky" else c["fur"]
    if kind == "husky":
        # Husky eyeliner: a dark rim around each eye.
        for (x, y, rx, ry) in ((ex, ey, erx, ery), (fx, fy, frx, fry)):
            art.blend(ellipse(x, y, rx * sc + 7, ry * sc + 6), c["dark"], 0.55, 2.5, "multiply", cl)
    if face == "squint":
        shut_eye(art, c, ex, ey, erx, ery)
        shut_eye(art, c, fx, fy, frx * 0.9, fry, far=True)
    elif face == "dizzy":
        spiral_eye(art, c, ex, ey, erx, ery, 0)
        spiral_eye(art, c, fx, fy, frx, fry, 140)
    else:
        eye(art, c, ex, ey, erx * sc, ery * sc, look, pupil, iris_s, lid, tilt, lower, lidc_near, c["white"] if kind != "husky" else c["white"])
        eye(art, c, fx, fy, frx * sc, fry * sc, (look[0] - 1, look[1]), pupil, iris_s, lid, tilt, lower, lidc_far, lidc_far, far=True)
    # Brows: lifted by blift (negative = higher), rotated by brot (+ lifts the inner end).
    bcol = c["brow"]
    bink = deep(c["dark"]) if kind != "terrier" else deep(c["fur"])
    tuf = kind == "terrier"
    if face == "dizzy":
        brow(art, bcol, bink, ex - 2, ey - ery - 16, 20, 14, 6, 9, tufty=tuf)
        brow(art, bcol, bink, fx + 2, fy - fry - 12, 13, -18, 5, 8, far=True, tufty=tuf)
    else:
        brow(art, bcol, bink, ex - 2, ey - ery * sc - 13 + blift, 22, brot, 7, 12, tufty=tuf)
        brow(art, bcol, bink, fx + 1, fy - fry * sc - 10 + blift, 14, brot, 5, 10, far=True, tufty=tuf)
    nose(art, c, nx, ny)
    mx, my = f["mouth"]
    mouth(art, c, mk, mx, my, c["white"])
    if kind == "terrier":
        terrier_bow(art, c)
    # Extras that sell the feeling.
    if face == "panic":
        for (x, y, s) in ((48, 62, 1.0), (30, 96, 0.7)):
            d = f"M{x},{y - 16 * s} C{x + 10 * s},{y - 2 * s} {x + 11 * s},{y + 9 * s} {x},{y + 10 * s} C{x - 11 * s},{y + 9 * s} {x - 10 * s},{y - 2 * s} {x},{y - 16 * s} Z"
            art.paint(d, "#9fdcff", w=2.6, ink="#3a78b8", k=5, hi=0.6, core=0.3, bounce=0, drop=0,
                      lights=[(ellipse(x - 3 * s, y, 3 * s, 5 * s), "#ffffff", 0.9, 1)])
    if face in ("idle", "whee", "grit"):
        # Rosy cheek blush on the muzzle.
        art.blend(ellipse(ex + 30, ey + 36, 18, 10), "#ff6f7a", 0.45, 5, "normal", cl)
    if face == "whee":
        for (x, y, s) in ((20, 40, 1.0), (236, 30, 0.8)):
            d = f"M{x},{y - 10 * s} L{x + 3 * s},{y - 3 * s} L{x + 10 * s},{y} L{x + 3 * s},{y + 3 * s} L{x},{y + 10 * s} L{x - 3 * s},{y + 3 * s} L{x - 10 * s},{y} L{x - 3 * s},{y - 3 * s} Z"
            art.paint(d, "#ffe066", w=2.2, ink="#c98a10", k=3, hi=0.5, core=0, bounce=0, drop=0)
    return art


def terrier_hair(art, c):
    """Pixie's silky fringe, parted in the middle and falling to both sides,
    and the topknot fountain the bow ties up."""
    hair = mix(c["fur"], "#ffe2a6", 0.3)
    ink = deep(c["fur"])
    left = shape([(146, 22), (114, 22), (84, 34), (64, 54), (60, 76), (72, 66), (82, 74), (92, 62), (104, 70), (112, 56),
                  (126, 58), (136, 44), (150, 36)], seed=12)
    right = shape([(150, 22), (176, 28), (194, 46), (204, 70), (192, 62), (184, 70), (174, 54), (162, 52), (154, 38)], seed=13)
    for d, cx in ((left, 104), (right, 176)):
        art.paint(d, hair, w=3.0, ink=ink, k=7, hi=0.6, core=0.35, bounce=0, drop=0.5,
                  fur=[(fur_marks(cx, 42, 30, 12, 12, ang=200 if cx < 150 else -20, ln=14, seed=cx), c["dark"], 0.45, 2.0)])
    knot = shape([(132, 32), (126, 16), (116, 6), (132, 10), (140, 2), (148, 12), (162, 6), (154, 20), (150, 32)], seed=3)
    art.paint(knot, hair, w=3.0, ink=ink, k=6, hi=0.6, core=0.3, bounce=0, drop=0.5)


def terrier_bow(art, c):
    col = c["collar"]
    ink = deep(col)
    bx, by = 141, 28
    for side in (-1, 1):
        pts = [(bx, by), (bx + 24 * side, by - 14), (bx + 32 * side, by - 2), (bx + 26 * side, by + 14), (bx, by + 4)]
        if side < 0:
            pts = pts[::-1]
        art.paint(shape(pts), col, w=3.0, ink=ink, k=7, hi=0.6, core=0.45, bounce=0.4, drop=0.6)
        for (x, y) in ((bx + 17 * side, by - 5), (bx + 24 * side, by + 7)):
            art.fill(circle(x, y, 2.6), "#ffffff", 0.9)
    art.paint(ellipse(bx, by, 8, 8), tone(col, -0.05), w=3.0, ink=ink, k=4, hi=0.6, core=0.4, bounce=0, drop=0.6)


# ---------------------------------------------------------------- ears
def ear_svg(c):
    """Ear sprite, 160x72: the root at the right edge (game pivot at 92%
    width), the flap reaching left."""
    art = Art()
    kind = c["kind"]
    col = c["ear"]
    if kind == "beagle":
        pts = [(156, 22), (120, 10), (78, 6), (40, 10), (14, 22), (6, 40), (16, 58), (44, 64), (84, 56), (124, 46), (156, 46)]
        d = shape(pts)
        art.paint(d, col, w=4, k=12, hi=0.6, core=0.5, bounce=0.5,
                  fur=[(fur_marks(80, 34, 56, 14, 22, ang=180, ln=14, seed=2), c["ear_in"], 0.4, 2.4)])
        art.stroke("M148,34 C118,30 80,32 44,42", c["ear_in"], 3, 0.55)
        art.blend(ellipse(20, 40, 22, 26), c["ear_in"], 0.6, 8, "multiply", art.clip_id(d))
    elif kind == "husky":
        pts = [(158, 14), (110, 8), (22, 34), (110, 62), (158, 58)]
        d = shape(pts, {1: (3, 5, 0.3), 2: (3, 5, 0.3)}, seed=4)
        # Make the tip pointed: rebuild with a sharp corner.
        d = (f"M158,12 C130,6 92,10 50,26 C34,32 22,36 12,36 C24,40 40,46 56,50 C94,60 130,64 158,60 Z")
        art.paint(d, col, w=4, k=12, hi=0.6, core=0.45, bounce=0.5)
        inner = "M150,22 C124,18 92,22 56,32 C46,35 36,36 28,36 C40,40 52,43 62,44 C96,50 126,52 150,50 Z"
        art.paint(shape([(150, 22), (110, 20), (40, 36), (110, 50), (150, 50)], {1: (3, 5, 0.4), 2: (3, 6, 0.4), 3: (3, 5, 0.4)}, seed=6),
                  c["ear_in"], w=0, k=6, hi=0.3, core=0.4, bounce=0, drop=0)
        art.stroke(d, deep(col), 4)
    else:
        pts = [(156, 20), (118, 12), (70, 12), (30, 22), (10, 38), (26, 52), (70, 58), (118, 52), (156, 50)]
        d = shape(pts, {2: (2, 7, 0.4), 3: (2, 8, 0.4), 4: (2, 10, 0.4), 5: (3, 9, 0.4), 6: (3, 8, 0.4)}, seed=8)
        art.paint(d, col, w=4, k=11, hi=0.6, core=0.45, bounce=0.5,
                  fur=[(fur_marks(70, 36, 54, 14, 26, ang=180, ln=16, seed=5), deep(col), 0.4, 2.2),
                       (fur_marks(80, 28, 50, 8, 14, ang=180, ln=14, seed=7), "#ffe2a8", 0.6, 2.2)])
    return art


# ---------------------------------------------------------------- bodies
# Body sprites are 256x256 with the figure position at the centre. The
# flying poses are upright (the game rotates the whole figure along the
# flight path): a 3/4 front view with the chest to the right. Dive and the
# sled pose are side views facing right. Torso centres and paw targets match
# the rig the game was built with (head_seat in draw.rs).
POSES = ("seated", "dive", "spread", "flap_a", "flap_b", "tumble", "land")

UPRIGHT = [(106, 70), (150, 68), (172, 86), (180, 118), (176, 150), (162, 176), (140, 190), (116, 192), (92, 180),
           (80, 154), (78, 120), (86, 88)]
BIB = [(118, 66), (166, 66), (174, 100), (170, 142), (156, 172), (134, 186), (118, 172), (114, 132), (116, 96)]
SIDE_DIVE = [(70, 106), (110, 96), (150, 94), (184, 100), (202, 116), (200, 140), (182, 156), (140, 162), (100, 160),
             (68, 150), (56, 128)]
SIDE_SEAT = [(58, 150), (88, 130), (130, 124), (166, 128), (186, 144), (182, 170), (160, 186), (120, 192), (80, 188),
             (56, 172)]

BODY = {
    #         torso pts, transform (sx, sy, rot, dx, dy), hind legs, front legs, tail base+angle, collar (x, y, rot), sole
    "seated": (SIDE_SEAT, (1, 1, 0, 0, 0), [(78, 176, 70, 206), (98, 182, 106, 208)], [(160, 166, 198, 202), (146, 174, 176, 208)],
               (60, 152, -30), (172, 132, 60), False),
    "dive":   (SIDE_DIVE, (1, 1, 0, 0, 0), [(80, 142, 34, 156), (86, 120, 40, 108)], [(178, 118, 224, 104), (174, 144, 220, 152)],
               (60, 118, -70), (182, 104, 70), False),
    "spread": (UPRIGHT, (1, 1, 0, 0, 0), [(100, 168, 64, 200), (156, 168, 194, 200)], [(96, 100, 52, 62), (160, 98, 204, 62)],
               (90, 178, -40), (136, 82, 0), True),
    "flap_a": (UPRIGHT, (0.96, 1.06, 0, 0, 2), [(102, 172, 90, 204), (154, 172, 168, 204)], [(98, 104, 46, 44), (158, 102, 210, 44)],
               (92, 180, -30), (136, 84, 0), True),
    "flap_b": (UPRIGHT, (1.08, 0.9, 0, 0, -4), [(104, 156, 96, 192), (152, 156, 160, 192)], [(92, 124, 76, 188), (164, 122, 180, 188)],
               (90, 166, -50), (136, 82, 0), True),
    "tumble": (UPRIGHT, (1, 1, 20, 0, 0), [(98, 164, 58, 150), (160, 156, 204, 178)], [(100, 102, 70, 56), (162, 98, 200, 68)],
               (92, 180, -90), (146, 84, 20), True),
    "land":   (UPRIGHT, (1.14, 0.84, 0, 0, 14), [(86, 168, 44, 184), (170, 168, 212, 184)], [(96, 140, 58, 164), (160, 140, 198, 164)],
               (80, 176, -20), (136, 104, 0), True),
}


def _tx(pts, t):
    sx, sy, rot, dx, dy = t
    return xform(pts, 128, 128, sx, sy, rot, dx, dy)


def paw(art, c, x, y, ang, col, r=17, sole=False):
    """A round mitten paw pointing along `ang` (degrees), toe grooves at the
    tip; with `sole` the pink pads face the viewer."""
    loc = [(-10, -15), (6, -17), (20, -12), (27, 0), (20, 12), (6, 17), (-10, 15)]
    s = r / 17
    pts = xform([(px * s, py * s) for px, py in loc], 0, 0, 1, 1, ang, x, y)
    d = shape(pts)
    art.paint(d, col, w=3.6, ink=deep(c["dark"]) if col != c["white"] else "#56648a", k=7, hi=0.55, core=0.5, bounce=0.6)
    ca, sa = math.cos(math.radians(ang)), math.sin(math.radians(ang))
    P = lambda u, v: (x + (u * ca - v * sa) * s, y + (u * sa + v * ca) * s)
    if sole:
        pad = "#ff9fb0"
        (a, b) = P(2, 0)
        art.paint(ellipse(a, b, 8 * s, 9 * s), pad, w=2.4, ink="#b8506a", k=4, hi=0.5, core=0.3, bounce=0, drop=0)
        for v in (-10, 0, 10):
            (a, b) = P(17 if v else 20, v)
            art.paint(circle(a, b, 4.2 * s), pad, w=2.2, ink="#b8506a", k=3, hi=0.5, core=0.2, bounce=0, drop=0)
    else:
        for v in (-6, 6):
            (a0, b0), (a1, b1) = P(26, v * 0.9), P(15, v * 1.1)
            art.stroke(f"M{a0:.1f},{b0:.1f} L{a1:.1f},{b1:.1f}", "#56648a" if col == c["white"] else deep(col), 2.6)


def leg(art, c, x0, y0, x1, y1, front, sole=False):
    L = max(1, math.hypot(x1 - x0, y1 - y0))
    nx, ny = -(y1 - y0) / L, (x1 - x0) / L
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2
    out = 1 if (nx * (mx - 128) + ny * (my - 128)) > 0 else -1
    widths = [36, 27, 24] if front else [44, 32, 26]
    ring = bent_limb([(x0, y0), (x1, y1)], widths, bend=out * L * 0.1)
    d = shape(ring, seed=int(x0 + y0))
    fur, sock = c["leg_top"], c["sock"]
    ink = deep(c["dark"])
    art.add(f'<path d="{shift_path(d, 1.0, 1.8)}" fill="{ink}"/>')
    art.paint(d, fur, w=0, hi=0, core=0, bounce=0, drop=0)
    # Sock: the lower part of the leg in the sock colour, with a fur edge.
    t0 = 0.5 if c["kind"] != "terrier" else 0.42
    sx_, sy_ = x0 + (x1 - x0) * t0, y0 + (y1 - y0) * t0
    cid = art.clip_id(d)
    ang = math.degrees(math.atan2(y1 - y0, x1 - x0))
    sock_pts = xform([(0, -30), (80, -30), (80, 30), (0, 30)], 0, 0, 1, 1, ang, sx_, sy_)
    sd = shape(sock_pts, {3: (3, 6, 0.0)}, seed=int(x1))
    art.add(f'<path d="{sd}" fill="{sock}" clip-path="url(#{cid})"/>')
    art.add(f'<path d="{fur_marks(mx, my, L * 0.25, 8, 6, ang=ang, ln=10, seed=int(x0))}" fill="none" stroke="{deep(fur)}" '
            f'stroke-width="2" stroke-linecap="round" opacity="0.35" clip-path="url(#{cid})"/>')
    art.overlay(d, k=11, hi=0.5, core=0.45, bounce=0.55)
    art.stroke(d, ink, 3.8)
    paw(art, c, x1 + (x1 - x0) / L * 4, y1 + (y1 - y0) / L * 4, ang, sock, r=17 if front else 19, sole=sole)


def tail(art, c, x, y, ang):
    kind = c["kind"]
    if kind == "beagle":
        loc = [(0, 0), (-10, -28), (-4, -58)]
        pts = xform(loc, 0, 0, 1, 1, ang + 30, x, y)
        ring = bent_limb(pts, [20, 15, 12])
        d = shape(ring)
        art.paint(d, c["body"], w=0, hi=0, core=0, bounce=0, drop=0)
        cid = art.clip_id(d)
        tip = shape(xform([(-30, -40), (20, -40), (20, -90), (-30, -90)], 0, 0, 1, 1, ang + 30, x, y), {3: (3, 6, 0)})
        art.add(f'<path d="{tip}" fill="{c["white"]}" clip-path="url(#{cid})"/>')
        art.overlay(d, k=8)
        art.stroke(d, deep(c["dark"]), 3.8)
    elif kind == "husky":
        loc = [(6, 6), (-18, -6), (-34, -32), (-30, -62), (-8, -78), (14, -70), (6, -56), (-10, -46), (-8, -24), (12, -6)]
        pts = xform(loc, 0, 0, 1, 1, ang + 40, x, y)
        d = shape(pts, {1: (2, 7, 0.4), 2: (3, 8, 0.4), 3: (2, 8, 0.4), 4: (2, 7, 0.4)}, seed=21)
        art.paint(d, c["body"], w=0, hi=0, core=0, bounce=0, drop=0)
        cid = art.clip_id(d)
        under = shape(xform([(14, -70), (6, -56), (-10, -46), (-8, -24), (12, -6), (30, -30)], 0, 0, 1, 1, ang + 40, x, y))
        art.add(f'<path d="{under}" fill="{c["white"]}" clip-path="url(#{cid})"/>')
        art.overlay(d, k=10)
        art.stroke(d, deep(c["dark"]), 3.8)
    else:
        loc = [(0, 0), (-6, -16), (-4, -30)]
        pts = xform(loc, 0, 0, 1, 1, ang + 30, x, y)
        d = shape(bent_limb(pts, [18, 14, 10]))
        art.paint(d, c["body"], w=3.8, ink=deep(c["saddle"]), k=7)
        tx, ty = pts[-1]
        tuft = shape(xform([(0, 6), (-8, -4), (-2, -16), (6, -4), (10, 6)], 0, 0, 1, 1, ang + 30, tx, ty), {1: (2, 5, 0.3), 2: (2, 5, 0.3)})
        art.paint(tuft, c["body"], w=3.4, ink=deep(c["saddle"]), k=6)


def collar(art, c, x, y, rot):
    band = shape(xform([(-32, -6), (0, 4), (32, -8), (33, 6), (0, 20), (-31, 8)], 0, 0, 1, 1, rot, x, y))
    art.paint(band, c["collar"], w=3.4, k=6, hi=0.6, core=0.45, bounce=0.4)
    tx, ty = xform([(2, 26)], 0, 0, 1, 1, rot, x, y)[0]
    art.stroke(f"M{x:.1f},{y + 12:.1f} L{tx:.1f},{ty - 6:.1f}", "#8a6a20", 2.4)
    if c["kind"] == "beagle":
        d = shape([(tx - 9, ty - 6), (tx - 3, ty - 4), (tx + 3, ty - 4), (tx + 9, ty - 6), (tx + 7, ty), (tx + 9, ty + 6),
                   (tx + 3, ty + 4), (tx - 3, ty + 4), (tx - 9, ty + 6), (tx - 7, ty)])     # bone tag
    elif c["kind"] == "husky":
        d = ellipse(tx, ty, 9, 9)
    else:
        d = (f"M{tx},{ty + 9} C{tx - 12},{ty} {tx - 10},{ty - 10} {tx - 4},{ty - 9} C{tx - 1},{ty - 9} {tx},{ty - 6} {tx},{ty - 5} "
             f"C{tx},{ty - 6} {tx + 1},{ty - 9} {tx + 4},{ty - 9} C{tx + 10},{ty - 10} {tx + 12},{ty} {tx},{ty + 9} Z")  # heart
    art.paint(d, c["tag"], w=2.8, ink=deep(c["tag"]), k=5, hi=0.6, core=0.4, bounce=0, drop=0.5,
              lights=[(ellipse(tx - 3, ty - 3, 3, 2), "#ffffff", 0.9, 0.8)])


def body_svg(c, pose):
    pts, t, hind, front, (tx, ty, ta), (cx, cy, cr), sole = BODY[pose]
    art = Art()
    kind = c["kind"]
    T = _tx(pts, t)
    if pts is UPRIGHT:
        tf = {1: (2, 6, 0.3), 2: (2, 6, 0.3), 8: (2, 7, 0.3), 9: (2, 7, 0.3), 10: (2, 6, 0.3)}
        if kind == "terrier":
            tf.update({4: (2, 10, 0.4), 5: (3, 12, 0.4), 6: (2, 12, 0.4), 7: (3, 12, 0.4)})
        if kind == "husky":
            tf.update({3: (2, 8, 0.3), 4: (2, 8, 0.3), 9: (3, 9, 0.3)})
    else:
        tf = {0: (2, 6, 0.3), 9: (2, 7, 0.3)} if pose == "dive" else {0: (2, 6, 0.3), 8: (2, 7, 0.3)}
        if kind == "terrier":
            tf.update({6: (3, 11, 0.4), 7: (3, 11, 0.4)})
    torso = shape(T, tf, seed=31)
    tail(art, c, tx, ty, ta)
    for (x0, y0, x1, y1) in hind:
        leg(art, c, x0, y0, x1, y1, False, sole=sole)
    ink = deep(c["dark"]) if kind != "terrier" else deep(c["saddle"])
    art.add(f'<path d="{shift_path(torso, 1.4, 2.4)}" fill="{ink}"/>')
    art.paint(torso, c["body"], w=0, hi=0, core=0, bounce=0, drop=0)
    tid = art.clip_id(torso)
    upright = pts is UPRIGHT
    if upright:
        bib = shape(_tx(BIB, t), {6: (2, 7, 0.3), 7: (2, 7, 0.3), 8: (2, 6, 0.3)}, seed=5)
        belly_ang = 90 + t[2]
    else:
        # Side view: white chest and belly along the underside.
        if pose == "dive":
            bib = shape([(86, 150), (140, 146), (180, 132), (196, 106), (220, 110), (220, 190), (80, 190)], {0: (3, 6, 0.3), 1: (3, 6, 0.3)}, seed=6)
        else:
            bib = shape([(70, 180), (120, 176), (160, 164), (176, 136), (200, 136), (200, 210), (60, 210)], {0: (3, 6, 0.3), 1: (3, 6, 0.3)}, seed=6)
        belly_ang = 0
    chest = c["white"] if kind != "terrier" else c["leg_top"]
    art.add(f'<path d="{bib}" fill="{chest}" clip-path="url(#{tid})"/>')
    if kind == "husky":
        # Darker saddle on the back.
        sad = ellipse(96, 120, 34, 60) if upright else ellipse(120, 96, 70, 22)
        art.blend(sad, c["saddle"], 0.55, 12, "multiply", tid)
    if kind == "terrier":
        art.add(f'<path d="{fur_marks(110, 120, 26, 50, 22, ang=95, ln=14, seed=8)}" fill="none" stroke="#9fb0cc" stroke-width="2.2" '
                f'stroke-linecap="round" opacity="0.6" clip-path="url(#{tid})"/>')
    art.add(f'<path d="{fur_marks(110, 130, 34, 40, 18, ang=belly_ang + 20, ln=11, seed=4)}" fill="none" stroke="{deep(c["body"])}" '
            f'stroke-width="2.2" stroke-linecap="round" opacity="0.3" clip-path="url(#{tid})"/>')
    art.overlay(torso, k=16, hi=0.5, core=0.3, bounce=0.5, rim=0.25)
    art.stroke(torso, ink, 4.6)
    # Belly button curl and a chest fur flick.
    if upright:
        bx, by = _tx([(142, 150)], t)[0]
        art.stroke(f"M{bx:.1f},{by:.1f} q5,3 2,8", "#9aa6c6", 2.6, 0.7)
    for (x0, y0, x1, y1) in front:
        leg(art, c, x0, y0, x1, y1, True)
    collar(art, c, cx, cy, cr)
    return art


def portrait_svg(name, c, head_inner, ear_inner, body_inner):
    """Picker / results portrait: a head-and-shoulders bust that fills the
    square so it reads at 24-90 px."""
    d_ear, b_ear = ear_inner
    d_body, b_body = body_inner
    d_head, b_head = head_inner
    ears = (f'<g transform="translate(68 78) rotate(-72) translate(-147 -36)">{b_ear}</g>'
            f'<g transform="translate(188 78) scale(-1 1) rotate(-72) translate(-147 -36)">{b_ear}</g>')
    comp = (f'<g transform="translate(118 132) scale(0.98) translate(-128 -128)">'
            f'<g transform="translate(0 92)">{b_body}</g>{ears}<g>{b_head}</g></g>')
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" data-paint="skip">\n'
            f'<defs>{d_ear}{d_body}{d_head}</defs>\n{comp}\n</svg>')
