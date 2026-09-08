#!/usr/bin/env python3
"""Character candidates in the user's reference style (flat cartoon, thick
dark-brown line, flat fills, closed-happy eyes, orange blush, no shading).
Five dogs, five elves, five Santas -> assets/svg/cand_*.svg (256x320)."""
import os, math

PACK = os.environ.get("PACK", "calgary_xmas")
OUT = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)
W, H = 256, 320
INK = "#4a2e1e"
LW = 6
BLUSH = "#ff8a3c"

def write(name, body, defs=""):
    open(os.path.join(OUT, name + ".svg"), "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">\n<defs>{defs}</defs>\n{body}\n</svg>')
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

# ---------------------------------------------------------------- shared face bits
def happy_eyes(lx, rx, y, w=16):
    return line(f"M{lx - w},{y + 6} Q{lx},{y - 12} {lx + w},{y + 6}", 5) + line(f"M{rx - w},{y + 6} Q{rx},{y - 12} {rx + w},{y + 6}", 5)
def dot_eyes(lx, rx, y, r=9, iris=None):
    s = ""
    for ex in (lx, rx):
        if iris:
            s += f'<circle cx="{ex}" cy="{y}" r="{r + 4}" fill="#ffffff" {ol(4)}/><circle cx="{ex}" cy="{y}" r="{r}" fill="{iris}"/>'
        s += f'<circle cx="{ex}" cy="{y}" r="{r * 0.75 if iris else r}" fill="{INK}"/><circle cx="{ex + r * 0.35}" cy="{y - r * 0.35}" r="{r * 0.3}" fill="#ffffff"/>'
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
def small_smile(cx, y, w=18):
    return line(f"M{cx - w},{y} Q{cx},{y + 16} {cx + w},{y}", 5)
def dog_nose(cx, y, r=13):
    return f'<path d="M{cx - r},{y - r * 0.5} Q{cx},{y - r} {cx + r},{y - r * 0.5} Q{cx + r * 0.9},{y + r * 0.7} {cx},{y + r} Q{cx - r * 0.9},{y + r * 0.7} {cx - r},{y - r * 0.5} Z" fill="{INK}"/><ellipse cx="{cx - r * 0.35}" cy="{y - r * 0.35}" rx="{r * 0.3}" ry="{r * 0.18}" fill="#ffffff" opacity="0.7"/>'

def santa_hat(cx, top, w, tilt=-8, red="#e63c3c"):
    return (f'<g transform="rotate({tilt} {cx} {top + w * 0.5})">'
            + shape(f"M{cx - w*0.5},{top + w*0.5} C{cx - w*0.45},{top + w*0.1} {cx - w*0.15},{top - w*0.05} {cx + w*0.1},{top - w*0.02} C{cx + w*0.4},{top} {cx + w*0.55},{top - w*0.3} {cx + w*0.66},{top - w*0.36} C{cx + w*0.52},{top - w*0.05} {cx + w*0.5},{top + w*0.25} {cx + w*0.5},{top + w*0.5} Z", red)
            + shape(f"M{cx - w*0.56},{top + w*0.62} C{cx - w*0.56},{top + w*0.4} {cx - w*0.36},{top + w*0.36} {cx - w*0.28},{top + w*0.46} C{cx - w*0.16},{top + w*0.34} {cx},{top + w*0.34} {cx + w*0.06},{top + w*0.46} C{cx + w*0.16},{top + w*0.34} {cx + w*0.34},{top + w*0.34} {cx + w*0.4},{top + w*0.46} C{cx + w*0.5},{top + w*0.36} {cx + w*0.6},{top + w*0.42} {cx + w*0.56},{top + w*0.62} C{cx + w*0.5},{top + w*0.74} {cx - w*0.5},{top + w*0.74} {cx - w*0.56},{top + w*0.62} Z", "#ffffff")
            + circ(cx + w*0.68, top - w*0.36, w*0.15, "#ffffff") + '</g>')

def motion_marks(x, y, flip=False):
    s = -1 if flip else 1
    return line(f"M{x},{y - 22} C{x - 10 * s},{y - 10} {x - 10 * s},{y + 10} {x},{y + 22}", 5) + line(f"M{x + 14 * s},{y - 16} C{x + 6 * s},{y - 6} {x + 6 * s},{y + 6} {x + 14 * s},{y + 16}", 5)

# ---------------------------------------------------------------- dogs
def dog(name, fur, ear, ear_fur, eyes="happy", muzzle=None, markings="", collar="#e63c3c", tag="bell", ears_out=True, hat=True, iris=None, spots=False, body_len=0):
    b = ""
    # Ears flung out like wings (drawn behind the head).
    if ear == "floppy":
        b += ell(48, 78, 62, 22, ear_fur, rot=-18) + ell(208, 78, 62, 22, ear_fur, rot=18)
        b += motion_marks(-6, 70) + motion_marks(262, 70, True)
    elif ear == "long":
        b += ell(40, 90, 72, 24, ear_fur, rot=-24) + ell(216, 90, 72, 24, ear_fur, rot=24)
        b += motion_marks(-12, 84) + motion_marks(268, 84, True)
    elif ear == "upright":
        b += shape("M74,86 C54,60 44,30 60,18 C76,10 96,40 106,64 Z", ear_fur) + shape("M182,86 C202,60 212,30 196,18 C180,10 160,40 150,64 Z", ear_fur)
        b += f'<path d="M76,74 C66,58 62,40 68,32 C78,42 88,58 92,68 Z" fill="#f7b8c0"/><path d="M180,74 C190,58 194,40 188,32 C178,42 168,58 164,68 Z" fill="#f7b8c0"/>'
    elif ear == "pointy":
        b += shape("M80,84 C66,60 60,28 70,16 C78,10 90,40 100,64 Z", ear_fur) + shape("M176,84 C190,60 196,28 186,16 C178,10 166,40 156,64 Z", ear_fur)
        b += f'<path d="M82,72 C74,56 72,40 76,32 C82,44 90,58 94,66 Z" fill="#ffffff"/><path d="M174,72 C182,56 184,40 180,32 C174,44 166,58 162,66 Z" fill="#ffffff"/>'
    # Body: small, under the chin, with dangling legs and a tail.
    by = 196
    b += line(f"M96,{by + 40} C74,{by + 30} 62,{by + 8} 70,{by - 10}", 12) + line(f"M96,{by + 40} C74,{by + 30} 62,{by + 8} 70,{by - 10}", 5, fur) if False else ""
    b += shape(f"M76,{by - 8} C62,{by + 4} 56,{by - 20} 62,{by - 34} C70,{by - 26} 82,{by - 14} 84,{by - 4} Z", fur)  # tail
    b += shape(f"M92,{by} C88,{by + 30} 88,{by + 60 + body_len} 96,{by + 76 + body_len} L160,{by + 76 + body_len} C168,{by + 60 + body_len} 168,{by + 30} 164,{by} Z", fur)
    b += ell(128, by + 48 + body_len * 0.5, 22, 18, muzzle or "#ffffff", 0)
    for x in (104, 152):
        b += shape(f"M{x - 12},{by + 40 + body_len} L{x - 12},{by + 92 + body_len} C{x - 12},{by + 104 + body_len} {x + 12},{by + 104 + body_len} {x + 12},{by + 92 + body_len} L{x + 12},{by + 40 + body_len} Z", fur)
        b += line(f"M{x - 4},{by + 100 + body_len} L{x - 4},{by + 92 + body_len} M{x + 4},{by + 100 + body_len} L{x + 4},{by + 92 + body_len}", 4)
    if spots:
        b += f'<circle cx="112" cy="{by + 22}" r="7" fill="{INK}"/><circle cx="150" cy="{by + 62}" r="6" fill="{INK}"/>'
    # Collar and tag.
    b += shape(f"M90,{by - 2} Q128,{by + 16} 166,{by - 2} L168,{by + 12} Q128,{by + 30} 88,{by + 12} Z", collar, 5)
    if tag == "bell":
        b += circ(128, by + 24, 10, "#ffd54a", 4) + line(f"M124,{by + 28} L132,{by + 28}", 3)
    else:
        b += circ(128, by + 24, 10, "#c9ced8", 4)
    # Head: big rounded square, flat.
    head = "M60,74 C60,44 92,36 128,36 C164,36 196,44 196,74 L200,148 C202,186 172,204 128,204 C84,204 54,186 56,148 Z"
    b += shape(head, fur)
    b += markings
    b += blush(80, 176, 126)
    b += freckles(80, 176, 126)
    if eyes == "happy":
        b += happy_eyes(100, 156, 112)
    elif eyes == "dot":
        b += dot_eyes(100, 156, 112, 8)
    else:
        b += dot_eyes(100, 156, 112, 9, iris)
    b += dog_nose(128, 126, 14)
    b += open_smile(128, 150, 30, 22)
    if hat:
        b += santa_hat(128, 6, 96, tilt=-10)
    write(name, b)

# 1 Buddy: the reference dog, cream with long dark floppy ears.
dog("cand_dog_1", "#ffe1b3", "floppy", "#5a3a2a", eyes="happy")
# 2 Biscuit the corgi: orange, upright ears, white blaze and muzzle, green collar.
dog("cand_dog_2", "#f2a648", "upright", "#f2a648", eyes="open", iris="#5a3418", collar="#2f8f63",
    markings=f'<path d="M118,40 L138,40 L136,120 Q128,126 120,120 Z" fill="#fff3e0"/>' + ell(128, 150, 44, 28, "#fff3e0", 0))
# 3 Nanook the husky: grey with white mask, pointy ears, blue eyes.
dog("cand_dog_3", "#9aa6c2", "pointy", "#9aa6c2", eyes="open", iris="#3f7fd6", collar="#d63a3a",
    markings='<path d="M72,110 C84,84 104,74 128,74 C152,74 172,84 184,110 C190,150 170,190 128,190 C86,190 66,150 72,110 Z" fill="#ffffff"/>')
# 4 Pretzel the dachshund: chocolate, very long ears, longer body, closed eyes.
dog("cand_dog_4", "#b0703f", "long", "#6a3d22", eyes="happy", collar="#3a86d6", tag="tag", body_len=22, muzzle="#e8c39a")
# 5 Domino the dalmatian: white with spots and a black eye patch, one black ear.
dog("cand_dog_5", "#ffffff", "floppy", "#2b1a12", eyes="dot", collar="#e63c3c", spots=True,
    markings=f'<ellipse cx="156" cy="112" rx="24" ry="28" fill="{INK}"/><circle cx="82" cy="60" r="9" fill="{INK}"/><circle cx="176" cy="180" r="7" fill="{INK}"/>')

# ---------------------------------------------------------------- elves
def elf(name, tunic, stripe, hat_col, hair, skin="#ffe0c4", accessory="", beard=False, glasses=False, pigtails=False, hat_droop=False, hat_big=False):
    b = ""
    cx = 128
    # legs with stripes, curly shoes.
    for x in (110, 146):
        b += shape(f"M{x - 12},214 L{x - 12},272 C{x - 12},280 {x + 12},280 {x + 12},272 L{x + 12},214 Z", "#ffffff", 5)
        for y in range(220, 270, 14):
            b += f'<rect x="{x - 12}" y="{y}" width="24" height="7" fill="{stripe}"/>'
        b += line(f"M{x - 12},214 L{x - 12},272 C{x - 12},280 {x + 12},280 {x + 12},272 L{x + 12},214", 5)
    b += shape("M78,286 C66,286 56,278 58,268 C70,274 84,272 98,276 L122,276 L122,292 L84,292 Z", "#2f8f63", 5)
    b += shape("M178,286 C190,286 200,278 198,268 C186,274 172,272 158,276 L134,276 L134,292 L172,292 Z", "#2f8f63", 5)
    # tunic with a scalloped hem, belt.
    b += shape("M84,140 L92,108 L164,108 L172,140 L178,214 C166,206 160,222 150,214 C140,224 132,208 128,214 C124,208 116,224 106,214 C96,222 90,206 78,214 Z", tunic, 5)
    b += f'<rect x="80" y="170" width="96" height="12" rx="4" fill="#3a2a1e"/><rect x="118" y="167" width="20" height="18" rx="3" fill="#ffd54a" {ol(4)}/>'
    # arms: one waving, one holding an accessory.
    b += line("M92,120 C70,130 60,150 58,166", 20) + line("M92,120 C70,130 60,150 58,166", 10, tunic)
    b += line("M164,120 C186,110 196,92 200,74", 20) + line("M164,120 C186,110 196,92 200,74", 10, tunic)
    b += circ(58, 170, 11, skin, 5) + circ(202, 70, 11, skin, 5)
    b += accessory
    # head, ears, hair.
    b += shape("M92,64 C86,52 84,40 90,34 L104,52 Z M164,64 C170,52 172,40 166,34 L152,52 Z", skin, 5)
    b += circ(cx, 72, 40, skin, 5)
    if hair == "ginger":
        b += shape("M92,66 C94,44 108,34 128,36 C148,34 162,44 164,66 C156,58 146,54 128,58 C110,54 100,58 92,66 Z", "#e07a2a", 5)
    elif hair == "blonde":
        b += shape("M90,70 C92,44 108,32 128,34 C148,32 164,44 166,70 C156,60 146,56 128,60 C110,56 100,60 90,70 Z", "#ffd76a", 5)
        if pigtails:
            b += circ(80, 84, 14, "#ffd76a", 5) + circ(176, 84, 14, "#ffd76a", 5)
    elif hair == "black":
        b += shape("M90,68 C92,46 108,34 128,36 C148,34 164,46 166,68 C160,58 150,56 128,60 C106,56 96,58 90,68 Z", "#2b1a12", 5)
    if beard:
        b += shape("M96,84 C92,116 108,132 128,134 C148,132 164,116 160,84 C150,96 106,96 96,84 Z", "#ffffff", 5)
        b += shape("M108,86 Q128,98 148,86 Q138,94 128,90 Q118,94 108,86 Z", "#ffffff", 4)
    b += blush(104, 152, 84, 9, 6)
    if hat_big:
        b += happy_eyes(114, 142, 76, 10)
    else:
        b += dot_eyes(114, 142, 74, 6)
    if glasses:
        b += f'<circle cx="114" cy="74" r="12" fill="none" {ol(4)}/><circle cx="142" cy="74" r="12" fill="none" {ol(4)}/>' + line("M126,74 L130,74", 4)
    if not beard:
        b += small_smile(cx, 90, 12)
    # hat: pointed, with a brim and a bell.
    if hat_big:
        b += shape("M74,72 C86,10 160,-6 196,14 C200,22 196,34 194,44 L192,80 Z", hat_col, 5) + circ(198, 12, 9, "#ffd54a", 4)
        b += shape("M70,70 Q128,54 194,72 L194,88 Q128,72 70,88 Z", stripe, 5)
    elif hat_droop:
        b += shape("M92,52 C100,22 140,12 176,20 C196,26 210,44 214,66 C222,90 210,110 196,118 C200,100 196,80 184,70 C176,60 170,60 168,50 Z", hat_col, 5) + circ(196, 120, 9, "#ffd54a", 4)
        b += shape("M88,50 Q128,36 170,50 L170,64 Q128,50 88,64 Z", stripe, 5)
    else:
        b += shape("M92,50 C104,14 148,4 176,14 C184,18 184,30 178,44 L172,50 Z", hat_col, 5) + circ(182, 44, 9, "#ffd54a", 4)
        b += shape("M88,48 Q128,34 172,48 L172,62 Q128,48 88,62 Z", stripe, 5)
    write(name, b)

elf("cand_elf_1", "#3fa66b", "#e63c3c", "#3fa66b", "ginger")
elf("cand_elf_2", "#ff7fa3", "#ffffff", "#ff7fa3", "blonde", pigtails=True,
    accessory=shape("M28,150 L72,150 L72,192 L28,192 Z", "#3a86d6", 5) + line("M50,150 L50,192 M28,171 L72,171", 4, "#ffd54a") + shape("M40,150 Q50,134 60,150", "#ffd54a", 4))
elf("cand_elf_3", "#3a86d6", "#ffffff", "#3a86d6", "black", beard=True, glasses=True,
    accessory=line("M202,80 L214,124", 12) + line("M202,80 L214,124", 6, "#c9ced8") + shape("M196,64 L212,60 L216,74 L204,78 Z", "#c9ced8", 4))
elf("cand_elf_4", "#7a5bd6", "#c9bdfd", "#7a5bd6", "ginger", hat_droop=True,
    accessory=shape("M188,60 L216,60 L212,96 L192,96 Z", "#ffffff", 5) + shape("M196,62 L208,62 L207,72 L197,72 Z", "#8a5a30", 0) + line("M216,70 C228,70 228,86 214,88", 5))
elf("cand_elf_5", "#e63c3c", "#ffffff", "#3fa66b", "blonde", hat_big=True,
    accessory=line("M206,64 L220,120", 12) + line("M206,64 L220,120", 6, "#ffffff") + line("M208,72 L211,84 M213,92 L216,104", 6, "#e63c3c") + line("M206,64 C200,50 214,44 220,54", 6, "#e63c3c"))

# ---------------------------------------------------------------- santas
def santa(name, coat="#e63c3c", beard_len=1.0, slim=False, glasses=False, sunglasses=False, goggles=False, jacket=None, accessory="", wave=True, present=False):
    b = ""
    bw = 44 if slim else 62
    # boots
    for x in (104, 152):
        b += shape(f"M{x - 16},250 L{x - 16},284 C{x - 16},294 {x + 20},294 {x + 20},284 L{x + 16},250 Z", "#2a2234", 5)
    # body
    b += shape(f"M{128 - bw},252 C{128 - bw - 12},200 {128 - bw + 4},132 128,128 C{128 + bw - 4},132 {128 + bw + 12},200 {128 + bw},252 Z", jacket or coat, 5)
    b += f'<rect x="{128 - bw - 8}" y="240" width="{2 * bw + 16}" height="16" rx="8" fill="#ffffff" {ol(4)}/>'
    if not jacket:
        b += f'<rect x="118" y="130" width="20" height="110" rx="9" fill="#ffffff" {ol(4)}/>'
    b += f'<rect x="{128 - bw - 4}" y="192" width="{2 * bw + 8}" height="16" rx="5" fill="#2a2234" {ol(4)}/><rect x="116" y="188" width="24" height="24" rx="4" fill="#ffd54a" {ol(4)}/>'
    # arms
    L = 128 - bw + 6; R = 128 + bw - 6
    if wave:
        b += line(f"M{R},150 C{R + 24},130 {R + 26},100 {R + 18},72", 22) + line(f"M{R},150 C{R + 24},130 {R + 26},100 {R + 18},72", 12, jacket or coat)
        b += circ(R + 18, 66, 12, "#ffffff", 5) + circ(R + 18, 50, 13, "#ffffff", 5)
    else:
        b += line(f"M{R},150 C{R + 30},160 {R + 30},190 {R + 8},210", 22) + line(f"M{R},150 C{R + 30},160 {R + 30},190 {R + 8},210", 12, jacket or coat)
        b += circ(R + 6, 214, 12, "#ffffff", 5) + circ(R + 2, 228, 13, "#ffffff", 5)
    b += line(f"M{L},150 C{L - 30},164 {L - 30},196 {L - 10},212", 22) + line(f"M{L},150 C{L - 30},164 {L - 30},196 {L - 10},212", 12, jacket or coat)
    b += circ(L - 10, 216, 12, "#ffffff", 5) + circ(L - 12, 230, 13, "#ffffff", 5)
    b += accessory
    if present:
        b += shape("M14,214 L62,214 L62,258 L14,258 Z", "#3fa66b", 5) + line("M38,214 L38,258 M14,236 L62,236", 5, "#ffd54a") + shape("M28,214 Q38,196 48,214", "#ffd54a", 4)
    # beard, head
    bl = 60 * beard_len
    b += shape(f"M84,96 C80,{112 + bl * 0.6} 96,{124 + bl} 128,{128 + bl} C160,{124 + bl} 176,{112 + bl * 0.6} 172,96 C160,112 96,112 84,96 Z", "#ffffff", 5)
    b += circ(128, 80, 42, "#ffe0c4", 5)
    b += blush(104, 152, 92, 10, 6)
    b += circ(128, 96, 9, "#ff8a8a", 4)
    b += shape("M102,102 C108,116 122,116 128,106 C134,116 148,116 154,102 C146,108 136,106 128,102 C120,106 110,108 102,102 Z", "#ffffff", 4)
    if sunglasses:
        b += f'<rect x="98" y="66" width="26" height="16" rx="6" fill="{INK}"/><rect x="132" y="66" width="26" height="16" rx="6" fill="{INK}"/>' + line("M124,72 L132,72", 4)
    else:
        b += dot_eyes(112, 144, 74, 5)
        b += line("M102,64 Q112,58 122,64 M134,64 Q144,58 154,64", 5, "#ffffff")
        if glasses:
            b += f'<circle cx="112" cy="74" r="12" fill="none" {ol(4)}/><circle cx="144" cy="74" r="12" fill="none" {ol(4)}/>' + line("M124,74 L132,74", 4)
    b += santa_hat(128, 8, 96, tilt=8, red=coat)
    if goggles:
        b += f'<rect x="86" y="40" width="84" height="14" rx="7" fill="#5a3a2a" {ol(4)}/>' + circ(110, 46, 13, "#9fd3ff", 5) + circ(146, 46, 13, "#9fd3ff", 5)
    write(name, b)

santa("cand_santa_1")
santa("cand_santa_2", goggles=True, jacket="#8a5a30",
      accessory=shape("M96,118 Q128,140 160,118 L164,138 Q128,160 92,138 Z", "#2f8f63", 5) + shape("M150,136 L164,176 L188,168 L172,132 Z", "#2f8f63", 5))
santa("cand_santa_3", slim=True, sunglasses=True, beard_len=0.7)
santa("cand_santa_4", glasses=True, beard_len=1.6, wave=False,
      accessory=shape("M190,196 L236,190 L242,236 L196,242 Z", "#c9915a", 5) + shape("M200,206 L228,202 L230,214 L202,218 Z", "#ffffff", 3) + f'<rect x="206" y="184" width="16" height="10" rx="3" fill="#c9ced8" {ol(3)}/>')
santa("cand_santa_5", beard_len=2.4, wave=False, present=True)
print("candidates written")
