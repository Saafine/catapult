#!/usr/bin/env python3
"""Game characters in the picked flat cartoon style (thick dark-brown line,
flat fills, happy closed eyes, orange blush): three pilots with seven faces
and seven body poses each, portraits, Santa and two elves.
Run after gen_svg.py; writes into the same pack svg folder."""
import os, math

PACK = os.environ.get("PACK", "calgary_xmas")
OUT = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)
INK = "#4a2e1e"
LW = 6
BLUSH = "#ff8a3c"

def write(name, w, h, body, defs=""):
    open(os.path.join(OUT, name + ".svg"), "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n<defs>{defs}</defs>\n{body}\n</svg>')
def ol(w=LW, c=INK):
    return f'stroke="{c}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'
def line(d, w=LW, c=INK):
    return f'<path d="{d}" fill="none" {ol(w, c)}/>'
def shape(d, fill, w=LW, c=INK):
    return f'<path d="{d}" fill="{fill}" {ol(w, c)}/>'
def ell(cx, cy, rx, ry, fill, w=LW, rot=0, c=INK):
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" transform="rotate({rot} {cx} {cy})" {ol(w, c)}/>'
def circ(cx, cy, r, fill, w=LW, c=INK):
    return f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" {ol(w, c)}/>'

def happy_eyes(lx, rx, y, w=16):
    return line(f"M{lx - w},{y + 6} Q{lx},{y - 12} {lx + w},{y + 6}", 5) + line(f"M{rx - w},{y + 6} Q{rx},{y - 12} {rx + w},{y + 6}", 5)
def shut_eyes(lx, rx, y, w=14):
    # Squeezed shut: arcs bowing down.
    return line(f"M{lx - w},{y - 4} Q{lx},{y + 12} {lx + w},{y - 4}", 5) + line(f"M{rx - w},{y - 4} Q{rx},{y + 12} {rx + w},{y - 4}", 5)
def dot_eyes(lx, rx, y, r=9, white=False, lid=0.0):
    s = ""
    for ex in (lx, rx):
        if white:
            s += f'<circle cx="{ex}" cy="{y}" r="{r + 6}" fill="#ffffff" {ol(4)}/>'
        s += f'<circle cx="{ex}" cy="{y}" r="{r}" fill="{INK}"/><circle cx="{ex + r * 0.35}" cy="{y - r * 0.35}" r="{r * 0.3}" fill="#ffffff"/>'
        if lid > 0:
            s += f'<path d="M{ex - r - 8},{y - r - 8} L{ex + r + 8},{y - r - 8} L{ex + r + 8},{y - r + lid * 2 * r} L{ex - r - 8},{y - r + lid * 2 * r} Z" fill="{{fur}}"/>'
            s += line(f"M{ex - r - 6},{y - r + lid * 2 * r} L{ex + r + 6},{y - r + lid * 2 * r}", 5)
    return s
def spiral_eyes(lx, rx, y):
    s = ""
    for ex in (lx, rx):
        s += f'<circle cx="{ex}" cy="{y}" r="15" fill="#ffffff" {ol(4)}/>'
        s += line(f"M{ex},{y} m-9,0 a9,9 0 1,1 9,9 a5,5 0 1,1 -5,-5 a2,2 0 1,1 2,2", 3.5)
    return s
def blush(lx, rx, y, rx_=14, ry=8):
    return f'<ellipse cx="{lx}" cy="{y}" rx="{rx_}" ry="{ry}" fill="{BLUSH}"/><ellipse cx="{rx}" cy="{y}" rx="{rx_}" ry="{ry}" fill="{BLUSH}"/>'
def freckles(lx, rx, y):
    s = ""
    for ex in (lx, rx):
        for dx, dy in ((-8, -2), (0, 4), (8, -1)):
            s += f'<circle cx="{ex + dx}" cy="{y + dy}" r="1.6" fill="{INK}"/>'
    return s
def open_smile(cx, y, w=30, h=26, tongue=True):
    s = shape(f"M{cx - w},{y} Q{cx},{y + h * 2} {cx + w},{y} Q{cx},{y + 6} {cx - w},{y} Z", "#7a2a2a", 5)
    if tongue:
        s += f'<path d="M{cx - w * 0.45},{y + h * 0.55} Q{cx},{y + h * 1.7} {cx + w * 0.45},{y + h * 0.55} Z" fill="#ff6b5a"/>'
    return s
def scream_mouth(cx, y, w=30, h=34):
    return shape(f"M{cx - w},{y} C{cx - w},{y + h * 1.6} {cx + w},{y + h * 1.6} {cx + w},{y} Q{cx},{y + 6} {cx - w},{y} Z", "#7a2a2a", 5) + f'<path d="M{cx - w * 0.5},{y + h * 0.9} Q{cx},{y + h * 1.5} {cx + w * 0.5},{y + h * 0.9} Z" fill="#ff6b5a"/>'
def wavy_mouth(cx, y, w=26):
    return line(f"M{cx - w},{y} Q{cx - w * 0.5},{y - 10} {cx},{y} Q{cx + w * 0.5},{y + 10} {cx + w},{y}", 5)
def teeth_mouth(cx, y, w=26):
    return shape(f"M{cx - w},{y} L{cx + w},{y} L{cx + w * 0.8},{y + 14} L{cx - w * 0.8},{y + 14} Z", "#ffffff", 5) + line(f"M{cx - w * 0.5},{y} L{cx - w * 0.45},{y + 14} M{cx},{y} L{cx},{y + 14} M{cx + w * 0.5},{y} L{cx + w * 0.45},{y + 14}", 3)
def tongue_out(cx, y, w=22):
    return line(f"M{cx - w},{y} Q{cx},{y + 14} {cx + w},{y}", 5) + shape(f"M{cx - 4},{y + 6} Q{cx + 4},{y + 30} {cx + 18},{y + 20} Q{cx + 18},{y + 6} {cx + 12},{y + 4} Z", "#ff6b5a", 4)
def dog_nose(cx, y, r=13):
    return f'<path d="M{cx - r},{y - r * 0.5} Q{cx},{y - r} {cx + r},{y - r * 0.5} Q{cx + r * 0.9},{y + r * 0.7} {cx},{y + r} Q{cx - r * 0.9},{y + r * 0.7} {cx - r},{y - r * 0.5} Z" fill="{INK}"/><ellipse cx="{cx - r * 0.35}" cy="{y - r * 0.35}" rx="{r * 0.3}" ry="{r * 0.18}" fill="#ffffff" opacity="0.7"/>'
def sweat(x, y):
    return shape(f"M{x},{y - 14} C{x + 10},{y} {x + 10},{y + 8} {x},{y + 8} C{x - 10},{y + 8} {x - 10},{y} {x},{y - 14} Z", "#9fd3ff", 4)
def santa_hat(cx, top, w, tilt=-8, red="#e63c3c"):
    return (f'<g transform="rotate({tilt} {cx} {top + w * 0.5})">'
            + shape(f"M{cx - w*0.5},{top + w*0.5} C{cx - w*0.45},{top + w*0.1} {cx - w*0.15},{top - w*0.05} {cx + w*0.1},{top - w*0.02} C{cx + w*0.4},{top} {cx + w*0.55},{top - w*0.3} {cx + w*0.66},{top - w*0.36} C{cx + w*0.52},{top - w*0.05} {cx + w*0.5},{top + w*0.25} {cx + w*0.5},{top + w*0.5} Z", red)
            + shape(f"M{cx - w*0.56},{top + w*0.62} C{cx - w*0.56},{top + w*0.4} {cx - w*0.36},{top + w*0.36} {cx - w*0.28},{top + w*0.46} C{cx - w*0.16},{top + w*0.34} {cx},{top + w*0.34} {cx + w*0.06},{top + w*0.46} C{cx + w*0.16},{top + w*0.34} {cx + w*0.34},{top + w*0.34} {cx + w*0.4},{top + w*0.46} C{cx + w*0.5},{top + w*0.36} {cx + w*0.6},{top + w*0.42} {cx + w*0.56},{top + w*0.62} C{cx + w*0.5},{top + w*0.74} {cx - w*0.5},{top + w*0.74} {cx - w*0.56},{top + w*0.62} Z", "#ffffff")
            + circ(cx + w*0.68, top - w*0.36, w*0.15, "#ffffff") + '</g>')
def motion_marks(x, y, flip=False):
    s = -1 if flip else 1
    return line(f"M{x},{y - 22} C{x - 10 * s},{y - 10} {x - 10 * s},{y + 10} {x},{y + 22}", 5) + line(f"M{x + 14 * s},{y - 16} C{x + 6 * s},{y - 6} {x + 6 * s},{y + 6} {x + 14 * s},{y + 16}", 5)

# ---------------------------------------------------------------- pilots
PILOTS = {
    "biscuit": dict(fur="#ffe1b3", ear="#5a3a2a", collar="#e63c3c", muzzle=None),   # Buddy
    "nanook":  dict(fur="#b9c1d6", ear="#3a3f52", collar="#3a86d6", muzzle="#ffffff"),
    "pixie":   dict(fur="#f2b56a", ear="#b5652a", collar="#ff5aa0", muzzle="#fff1dc"),
}
FACES = ("idle", "scream", "panic", "squint", "dizzy", "grit", "whee")
POSES = ("seated", "dive", "spread", "flap_a", "flap_b", "tumble", "land")
HEAD = "M60,74 C60,44 92,36 128,36 C164,36 196,44 196,74 L200,148 C202,186 172,204 128,204 C84,204 54,186 56,148 Z"

def head(name, c, face):
    fur, earc = c["fur"], c["ear"]
    fly = face not in ("idle", "grit")
    b = ""
    # Ears are a separate sprite (dog_<n>_ear) so the game can flap them.
    b += shape(HEAD, fur)
    if c["muzzle"]:
        b += ell(128, 150, 46, 34, c["muzzle"], 0)
    b += blush(80, 176, 126) + freckles(80, 176, 126)
    if face == "idle":
        b += happy_eyes(100, 156, 112) + dog_nose(128, 126, 14) + open_smile(128, 150, 30, 22)
    elif face == "whee":
        b += happy_eyes(100, 156, 112) + dog_nose(128, 126, 14) + open_smile(128, 148, 34, 28)
    elif face == "scream":
        b += dot_eyes(100, 156, 112, 9, white=True) + dog_nose(128, 128, 14) + scream_mouth(128, 150, 30, 30)
    elif face == "panic":
        b += dot_eyes(100, 156, 112, 6) + dog_nose(128, 128, 14) + wavy_mouth(128, 160, 24) + sweat(186, 92)
    elif face == "squint":
        b += shut_eyes(100, 156, 112) + dog_nose(128, 128, 14) + teeth_mouth(128, 154, 26)
    elif face == "dizzy":
        b += spiral_eyes(100, 156, 112) + dog_nose(128, 128, 14) + tongue_out(128, 158, 22)
    elif face == "grit":
        b += dot_eyes(100, 156, 112, 9, white=True).replace("{fur}", fur) + line("M80,96 L118,104 M176,96 L138,104", 5) + dog_nose(128, 128, 14) + teeth_mouth(128, 154, 26)
    b = b.replace("{fur}", fur)
    b += santa_hat(128, 6, 96, tilt=-10 if fly else 8)
    write(f"dog_{name}_head_{face}", 256, 256, b)

def leg(x0, y0, x1, y1, fill, w=24):
    dx, dy = x1 - x0, y1 - y0
    L = max(1e-3, math.hypot(dx, dy)); nx, ny = -dy / L * w / 2, dx / L * w / 2
    d = f"M{x0 + nx:.1f},{y0 + ny:.1f} L{x1 + nx:.1f},{y1 + ny:.1f} A{w/2:.1f},{w/2:.1f} 0 0,1 {x1 - nx:.1f},{y1 - ny:.1f} L{x0 - nx:.1f},{y0 - ny:.1f} Z"
    s = shape(d, fill)
    ux, uy = dx / L, dy / L
    for k in (-0.3, 0.3):
        tx, ty = x1 + nx * k * 2, y1 + ny * k * 2
        s += line(f"M{tx - ux * 6:.1f},{ty - uy * 6:.1f} L{tx + ux * 3:.1f},{ty + uy * 3:.1f}", 3.5)
    return s

# Body per pose: (cx, cy, rx, ry), hind legs, front legs.
BODY = {
    "seated": ((128, 112, 38, 46), [(100, 142, 84, 178), (156, 142, 172, 178)], [(108, 122, 102, 190), (148, 122, 154, 190)]),
    "dive":   ((128, 128, 62, 34), [(80, 140, 32, 152), (86, 118, 40, 110)], [(176, 116, 224, 104), (172, 142, 220, 148)]),
    "spread": ((128, 128, 42, 46), [(100, 160, 60, 198), (156, 160, 196, 198)], [(98, 100, 50, 60), (158, 100, 206, 60)]),
    "flap_a": ((128, 134, 40, 44), [(102, 166, 88, 202), (154, 166, 168, 202)], [(100, 106, 44, 40), (156, 106, 212, 40)]),
    "flap_b": ((128, 124, 48, 36), [(104, 152, 96, 188), (152, 152, 160, 188)], [(92, 132, 74, 188), (164, 132, 182, 188)]),
    "tumble": ((128, 128, 42, 48), [(98, 160, 56, 148), (158, 150, 206, 178)], [(100, 100, 70, 56), (160, 96, 200, 70)]),
    "land":   ((128, 142, 56, 34), [(84, 164, 40, 180), (172, 164, 216, 180)], [(96, 140, 56, 160), (160, 140, 200, 160)]),
}
def body(name, c, pose):
    fur, col = c["fur"], c["collar"]
    (cx, cy, rx, ry), hind, front = BODY[pose]
    b = ""
    # Tail curl off the left hip.
    tx, ty = cx - rx * 0.9, cy + ry * 0.2
    b += shape(f"M{tx},{ty} C{tx - 14},{ty - 30} {tx - 40},{ty - 26} {tx - 36},{ty - 4} C{tx - 34},{ty + 8} {tx - 16},{ty + 10} {tx},{ty + 8} Z", fur)
    for (x0, y0, x1, y1) in hind:
        b += leg(x0, y0, x1, y1, fur, 24)
    b += ell(cx, cy, rx, ry, fur)
    b += ell(cx, cy + ry * 0.35, rx * 0.55, ry * 0.4, c["muzzle"] or "#ffffff", 0)
    for (x0, y0, x1, y1) in front:
        b += leg(x0, y0, x1, y1, fur, 24)
    # Collar and bell at the top of the body.
    top = cy - ry
    b += shape(f"M{cx - rx * 0.95},{top + 4} Q{cx},{top + 24} {cx + rx * 0.95},{top + 4} L{cx + rx * 0.95},{top + 18} Q{cx},{top + 38} {cx - rx * 0.95},{top + 18} Z", col, 5)
    b += circ(cx, top + 32, 10, "#ffd54a", 4) + line(f"M{cx - 4},{top + 36} L{cx + 4},{top + 36}", 3)
    write(f"dog_{name}_body_{pose}", 256, 256, b)

def ear(name, c):
    # One ear, root at the right edge (pivot), tip to the left. 160x72.
    b = ell(78, 36, 70, 26, c["ear"])
    b += f'<ellipse cx="66" cy="34" rx="44" ry="12" fill="{c["ear"]}" opacity="0"/>'
    write(f"dog_{name}_ear", 160, 72, b)

for n, c in PILOTS.items():
    ear(n, c)
    for f in FACES:
        head(n, c, f)
    for p in POSES:
        body(n, c, p)
    # Portrait: seated body under an idle head, head-heavy like the reference.
    hb = open(os.path.join(OUT, f"dog_{n}_head_idle.svg")).read().split("<defs></defs>\n")[1].rsplit("\n</svg>", 1)[0]
    bb = open(os.path.join(OUT, f"dog_{n}_body_seated.svg")).read().split("<defs></defs>\n")[1].rsplit("\n</svg>", 1)[0]
    ears = ell(58, 126, 22, 60, c["ear"], rot=6) + ell(198, 126, 22, 60, c["ear"], rot=-6)
    write(f"dog_{n}", 256, 256, f'<g transform="translate(0 92) scale(0.9) translate(14 0)">{bb}</g><g transform="translate(24 -8) scale(0.82)">{ears}{hb}</g>')
    if n == "biscuit":
        write("style_1_chibi", 256, 320, f'<g transform="translate(0 130)">{bb}</g><g transform="translate(0 0)">{ears}{hb}</g>')

# ---------------------------------------------------------------- Santa (pick 1) and elves (pick 1), at game canvas sizes
def arm(x0, y0, x1, y1, x2, y2, col, w=26):
    return (f'<path d="M{x0},{y0} L{x1},{y1} L{x2},{y2}" fill="none" stroke="{INK}" stroke-width="{w + LW*2}" stroke-linecap="round" stroke-linejoin="round"/>'
            f'<path d="M{x0},{y0} L{x1},{y1} L{x2},{y2}" fill="none" stroke="{col}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>')

def santa_body():
    coat = "#e63c3c"; bw = 62
    b = ""
    for x in (104, 152):
        b += shape(f"M{x - 16},250 L{x - 16},284 C{x - 16},294 {x + 20},294 {x + 20},284 L{x + 16},250 Z", "#2a2234", 5)
    b += shape(f"M{128 - bw},252 C{128 - bw - 12},200 {128 - bw + 4},132 128,128 C{128 + bw - 4},132 {128 + bw + 12},200 {128 + bw},252 Z", coat, 5)
    b += f'<rect x="{128 - bw - 8}" y="240" width="{2 * bw + 16}" height="16" rx="8" fill="#ffffff" {ol(4)}/>'
    b += f'<rect x="118" y="130" width="20" height="110" rx="9" fill="#ffffff" {ol(4)}/>'
    b += f'<rect x="{128 - bw - 4}" y="192" width="{2 * bw + 8}" height="16" rx="5" fill="#2a2234" {ol(4)}/><rect x="116" y="188" width="24" height="24" rx="4" fill="#ffd54a" {ol(4)}/>'
    L = 128 - bw + 6; R = 128 + bw - 6
    b += line(f"M{R},150 C{R + 24},130 {R + 26},100 {R + 18},72", 22) + line(f"M{R},150 C{R + 24},130 {R + 26},100 {R + 18},72", 12, coat)
    b += circ(R + 18, 66, 12, "#ffffff", 5) + circ(R + 18, 50, 13, "#ffffff", 5)
    b += line(f"M{L},150 C{L - 30},164 {L - 30},196 {L - 10},212", 22) + line(f"M{L},150 C{L - 30},164 {L - 30},196 {L - 10},212", 12, coat)
    b += circ(L - 10, 216, 12, "#ffffff", 5) + circ(L - 12, 230, 13, "#ffffff", 5)
    bl = 60
    b += shape(f"M84,96 C80,{112 + bl * 0.6} 96,{124 + bl} 128,{128 + bl} C160,{124 + bl} 176,{112 + bl * 0.6} 172,96 C160,112 96,112 84,96 Z", "#ffffff", 5)
    b += circ(128, 80, 42, "#ffe0c4", 5)
    b += blush(104, 152, 92, 10, 6)
    b += circ(128, 96, 9, "#ff8a8a", 4)
    b += shape("M102,102 C108,116 122,116 128,106 C134,116 148,116 154,102 C146,108 136,106 128,102 C120,106 110,108 102,102 Z", "#ffffff", 4)
    b += dot_eyes(112, 144, 74, 5)
    b += line("M102,64 Q112,58 122,64 M134,64 Q144,58 154,64", 5, "#ffffff")
    b += santa_hat(128, 8, 96, tilt=8)
    return b
write("santa", 192, 256, f'<g transform="translate(0 8) scale(0.75)">{santa_body()}</g>')

def elf_body(tunic, stripe, hat_col, cowboy=False):
    b = ""
    for x in (110, 146):
        b += shape(f"M{x - 12},214 L{x - 12},272 C{x - 12},280 {x + 12},280 {x + 12},272 L{x + 12},214 Z", "#ffffff", 5)
        for y in range(220, 270, 14):
            b += f'<rect x="{x - 12}" y="{y}" width="24" height="7" fill="{stripe}"/>'
        b += line(f"M{x - 12},214 L{x - 12},272 C{x - 12},280 {x + 12},280 {x + 12},272 L{x + 12},214", 5)
    b += shape("M78,286 C66,286 56,278 58,268 C70,274 84,272 98,276 L122,276 L122,292 L84,292 Z", "#2f8f63", 5)
    b += shape("M178,286 C190,286 200,278 198,268 C186,274 172,272 158,276 L134,276 L134,292 L172,292 Z", "#2f8f63", 5)
    b += shape("M84,140 L92,108 L164,108 L172,140 L178,214 C166,206 160,222 150,214 C140,224 132,208 128,214 C124,208 116,224 106,214 C96,222 90,206 78,214 Z", tunic, 5)
    b += f'<rect x="80" y="170" width="96" height="12" rx="4" fill="#3a2a1e"/><rect x="118" y="167" width="20" height="18" rx="3" fill="#ffd54a" {ol(4)}/>'
    b += line("M92,120 C70,130 60,150 58,166", 20) + line("M92,120 C70,130 60,150 58,166", 10, tunic)
    b += line("M164,120 C186,110 196,92 200,74", 20) + line("M164,120 C186,110 196,92 200,74", 10, tunic)
    b += circ(58, 170, 11, "#ffe0c4", 5) + circ(202, 70, 11, "#ffe0c4", 5)
    b += shape("M92,64 C86,52 84,40 90,34 L104,52 Z M164,64 C170,52 172,40 166,34 L152,52 Z", "#ffe0c4", 5)
    b += circ(128, 72, 40, "#ffe0c4", 5)
    b += shape("M92,66 C94,44 108,34 128,36 C148,34 162,44 164,66 C156,58 146,54 128,58 C110,54 100,58 92,66 Z", "#e07a2a", 5)
    b += blush(104, 152, 84, 9, 6) + dot_eyes(114, 142, 74, 6) + line("M116,90 Q128,102 140,90", 5)
    if cowboy:
        # White Stampede hat.
        b += shape("M70,52 C70,44 84,42 100,42 L156,42 C172,42 186,44 186,52 C186,62 164,66 128,66 C92,66 70,62 70,52 Z", "#f4f4f8", 5)
        b += shape("M98,44 C98,16 110,4 128,4 C146,4 158,16 158,44 Z", "#ffffff", 5) + line("M110,12 C118,20 138,20 146,12", 4, "#c9c9d4")
        b += f'<rect x="98" y="38" width="60" height="8" fill="#2b1d2e"/>'
    else:
        b += shape("M92,50 C104,14 148,4 176,14 C184,18 184,30 178,44 L172,50 Z", hat_col, 5) + circ(182, 44, 9, "#ffd54a", 4)
        b += shape("M88,48 Q128,34 172,48 L172,62 Q128,48 88,62 Z", stripe, 5)
    return b
write("elf_a", 128, 176, f'<g transform="translate(0 6) scale(0.5)">{elf_body("#3fa66b", "#e63c3c", "#3fa66b")}</g>')
write("elf_b", 128, 176, f'<g transform="translate(128 6) scale(-0.5 0.5)">{elf_body("#3a86d6", "#ffffff", "#3a86d6", cowboy=True)}</g>')
print("chars written")
