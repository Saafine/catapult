#!/usr/bin/env python3
"""Pickups and props, drawn as outlined cartoon shapes; tools/paint.py
repaints them in the key-art style afterwards. The sled, building, sun and
clouds are hand painted in tools/gen_paint.py. Runs after gen_svg.py and
overwrites those files in the pack."""
import os

PACK = os.environ.get("PACK", "calgary_xmas")
OUT = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)
INK = "#3a2618"
LW = 6

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
def rect(x, y, w, h, fill, rx=8, sw=LW, c=INK):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" {ol(sw, c)}/>'
def gloss(cx, cy, rx, ry, rot=-30):
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="#ffffff" opacity="0.45" transform="rotate({rot} {cx} {cy})"/>'
def shade(d, a=0.18):
    return f'<path d="{d}" fill="#000000" opacity="{a}"/>'

# ---------------------------------------------------------------- pickups
def trampoline():
    b = ""
    for x in (44, 180):
        b += line(f"M{x},110 L{x - 14},168", 12) + line(f"M{x},110 L{x - 14},168", 6, "#8a8f9c")
        b += line(f"M{x},110 L{x + 14},168", 12) + line(f"M{x},110 L{x + 14},168", 6, "#8a8f9c")
    b += ell(112, 108, 104, 30, "#3a4a66")
    b += ell(112, 100, 92, 22, "#e6382f")
    b += ell(112, 100, 70, 14, "#ff6b5a", 4)
    b += gloss(80, 94, 20, 6, -10)
    for k in range(-3, 4):
        x = 112 + k * 26
        b += line(f"M{x},118 L{x},126", 4, "#c9ced8")
    write("trampoline", 224, 176, b)
trampoline()

def reindeer():
    b = ""
    # antlers
    b += line("M84,44 L70,14 M84,44 L96,10 M96,10 L110,20 M70,14 L58,24", 10) + line("M84,44 L70,14 M84,44 L96,10 M96,10 L110,20 M70,14 L58,24", 5, "#c9915a")
    b += line("M140,44 L126,14 M140,44 L152,10 M152,10 L166,20 M126,14 L114,24", 10) + line("M140,44 L126,14 M140,44 L152,10 M152,10 L166,20 M126,14 L114,24", 5, "#c9915a")
    # legs tucked back
    b += shape("M60,124 C40,128 24,140 22,156 C34,152 52,144 66,140 Z", "#a86a3a") + shape("M150,126 C170,130 190,142 194,158 C180,154 162,146 148,142 Z", "#a86a3a")
    b += ell(112, 112, 70, 40, "#b8743f")
    b += ell(118, 128, 44, 22, "#f2d9b8", 0)
    b += ell(72, 82, 40, 34, "#b8743f")
    b += ell(50, 96, 24, 18, "#c98a50")
    b += circ(34, 100, 12, "#ff3b3b") + gloss(30, 96, 4, 2.5)
    b += circ(80, 74, 7, INK) + f'<circle cx="82" cy="72" r="2.5" fill="#ffffff"/>'
    b += shape("M96,60 C96,44 86,34 78,40 C72,46 78,58 86,66 Z", "#b8743f", 5)
    b += shape("M100,120 Q112,134 124,120", "#ffffff", 0) + line("M100,120 Q112,134 124,120", 4)
    b += rect(96, 138, 40, 10, "#e6382f", 4, 4) + circ(116, 150, 6, "#ffd23a", 3)
    write("reindeer", 224, 176, b)
reindeer()

def wings():
    b = ""
    for sgn, x in ((1, 120), (-1, 120)):
        d = f"M{x},110 C{x + sgn*20},60 {x + sgn*80},30 {x + sgn*116},44 C{x + sgn*100},60 {x + sgn*106},70 {x + sgn*96},84 C{x + sgn*84},92 {x + sgn*80},104 {x + sgn*70},112 C{x + sgn*60},120 {x + sgn*40},134 {x},126 Z"
        b += shape(d, "#ffffff")
        b += line(f"M{x + sgn*30},112 C{x + sgn*46},96 {x + sgn*70},84 {x + sgn*96},80 M{x + sgn*20},124 C{x + sgn*40},112 {x + sgn*60},100 {x + sgn*80},100", 3.5, "#c9d2e6")
    b += ell(120, 40, 34, 10, "#ffd23a", 5) + ell(120, 40, 22, 5, "#fff2a8", 0)
    write("wings", 240, 160, b)
wings()

def sleigh():
    b = line("M24,150 Q10,160 20,166 L180,166 Q198,166 190,150", 12) + line("M24,150 Q10,160 20,166 L180,166 Q198,166 190,150", 6, "#ffd23a")
    b += shape("M14,80 C10,120 30,150 70,150 L150,150 C160,150 164,140 164,130 L160,90 C160,80 150,70 140,74 Q130,78 128,90 L126,124 L60,124 L56,80 C54,66 40,60 30,64 Z", "#e6382f")
    b += rect(60, 124, 66, 10, "#ffd23a", 3, 4)
    b += shade("M14,130 L164,130 L164,140 Q160,150 150,150 L70,150 Q40,148 24,130 Z", 0.2)
    b += rect(66, 96, 40, 30, "#2fa353", 4) + rect(100, 90, 34, 36, "#3577e0", 4)
    b += circ(96, 86, 26, "#ffe0c4") + shape("M74,92 C78,118 114,118 118,92 Z", "#ffffff", 5)
    b += circ(88, 82, 4, INK) + circ(104, 82, 4, INK) + circ(96, 90, 5, "#ff8a8a", 3)
    b += shape("M62,74 C70,40 120,30 136,44 L144,36 C140,60 132,66 130,76 Z", "#e6382f", 5) + rect(56, 70, 84, 14, "#ffffff", 7, 4) + circ(146, 34, 9, "#ffffff", 4)
    write("sleigh", 320, 176, b)
sleigh()

def ufo():
    b = ell(128, 78, 120, 34, "#c9d2e6")
    b += ell(128, 70, 112, 12, "#ffffff", 0)
    b += shape("M70,60 C80,20 176,20 186,60 Z", "#9fd3ff")
    b += gloss(110, 36, 22, 8, -20)
    for i, x in enumerate((50, 90, 128, 166, 206)):
        b += circ(x, 96 if i in (0, 4) else 104, 8, ["#3fbf7a", "#ff6b5a", "#ffd23a", "#3577e0", "#3fbf7a"][i], 4)
    b += circ(128, 46, 16, "#8fe3a8") + circ(122, 44, 3.5, INK) + circ(134, 44, 3.5, INK)
    write("ufo", 256, 128, b)
ufo()

def updraft():
    # A tall column of curling wind strokes with arrowheads pointing up.
    b = ""
    for k, (x, y0) in enumerate(((40, 236), (88, 250), (60, 120), (100, 130), (36, 30), (92, 40))):
        d = f"M{x},{y0} C{x - 26},{y0 - 30} {x + 26},{y0 - 60} {x},{y0 - 90}"
        b += line(d, 14, "#2b3a5a") + line(d, 7, "#dff2ff")
        b += shape(f"M{x - 12},{y0 - 84} L{x},{y0 - 104} L{x + 12},{y0 - 84} Z", "#dff2ff", 5, "#2b3a5a")
    for (x, y) in ((22, 200), (110, 180), (64, 70), (18, 90)):
        b += circ(x, y, 5, "#ffffff", 3, "#2b3a5a")
    write("updraft", 128, 256, b)
updraft()

def coldwind():
    # Horizontal icy streaks with pointed ends, blue-white, plus snowflakes.
    b = ""
    for k, (y, x0, x1) in enumerate(((30, 24, 232), (52, 10, 200), (74, 40, 246), (96, 18, 210))):
        d = f"M{x0},{y} C{x0 + 40},{y - 10} {x1 - 60},{y + 10} {x1},{y}"
        b += line(d, 16, "#2b3a5a") + line(d, 9, "#bfe6ff")
        b += shape(f"M{x0 + 4},{y - 9} L{x0 - 22},{y} L{x0 + 4},{y + 9} Z", "#bfe6ff", 5, "#2b3a5a")
    for (x, y) in ((70, 18), (150, 44), (110, 86), (200, 108)):
        b += line(f"M{x - 8},{y} L{x + 8},{y} M{x},{y - 8} L{x},{y + 8} M{x - 6},{y - 6} L{x + 6},{y + 6} M{x - 6},{y + 6} L{x + 6},{y - 6}", 3, "#ffffff")
    write("coldwind", 256, 128, b)
coldwind()

def icecube():
    # Translucent ice block with a bright edge, cracks and icicles.
    b = shape("M40,36 L216,36 L226,200 L30,200 Z", "#9fd8ff", 6, "#2b3a5a")
    b += f'<path d="M40,36 L216,36 L226,200 L30,200 Z" fill="#dff4ff" opacity="0.55"/>'
    b += line("M60,50 L70,110 L58,150", 4, "#ffffff") + line("M170,60 L150,120 L176,160", 4, "#ffffff")
    b += gloss(90, 70, 26, 10, -25)
    for (x, h) in ((60, 30), (110, 46), (160, 26), (200, 38)):
        b += shape(f"M{x - 12},200 L{x},{200 + h} L{x + 12},200 Z", "#bfe6ff", 5, "#2b3a5a")
    write("icecube", 256, 256, b)
icecube()

# ---------------------------------------------------------------- props
def pine_snow():
    b = rect(84, 276, 24, 44, "#8a5a30", 6)
    for (y, w) in ((286, 88), (226, 72), (166, 54)):
        b += shape(f"M96,{y - 104} L{96 + w},{y} L{96 - w},{y} Z", "#2c8a4a")
        b += shape(f"M96,{y - 104} L{96 + w*0.5},{y - 46} Q96,{y - 32} {96 - w*0.5},{y - 46} Z", "#ffffff", 4)
    write("pine_snow", 192, 320, b)
pine_snow()

def snowman():
    b = circ(80, 176, 44, "#ffffff") + circ(80, 108, 34, "#ffffff") + circ(80, 56, 26, "#ffffff")
    b += circ(72, 50, 3.5, INK) + circ(90, 50, 3.5, INK)
    b += shape("M84,58 L118,64 L84,68 Z", "#ff7a1a", 4)
    b += line("M66,72 Q80,80 94,72", 4)
    for y in (100, 116, 132):
        b += circ(80, y, 4, INK, 0)
    b += line("M46,110 L14,88 M114,110 L146,88", 8) + line("M46,110 L14,88 M114,110 L146,88", 4, "#8a5a30")
    b += rect(56, 14, 48, 22, "#2a2234", 3) + rect(64, 0, 32, 20, "#2a2234", 4)
    b += shape("M50,86 Q80,100 110,86 L112,98 Q80,112 48,98 Z", "#e6382f", 4)
    write("snowman", 160, 224, b)
snowman()

def fence():
    b = ""
    for x in (24, 92, 160, 228):
        b += rect(x - 10, 20, 20, 76, "#c9915a", 4)
        b += shape(f"M{x - 12},22 L{x},6 L{x + 12},22 Z", "#c9915a", 4)
    b += rect(0, 40, 256, 12, "#a86a3a", 3, 4) + rect(0, 70, 256, 12, "#a86a3a", 3, 4)
    b += shape("M0,36 Q32,28 64,36 Q96,28 128,36 Q160,28 192,36 Q224,28 256,36 L256,44 L0,44 Z", "#ffffff", 0)
    write("fence", 256, 96, b)
fence()

def signpost():
    b = rect(70, 80, 20, 90, "#a86a3a", 4)
    b += rect(6, 14, 148, 78, "#c9915a", 12)
    b += rect(18, 26, 124, 54, "#fff6e0", 6, 4)
    b += shape("M4,14 Q30,0 60,14 Q90,2 128,14 Q140,4 156,14 L156,22 L4,22 Z", "#ffffff", 4)
    write("signpost", 160, 176, b)
signpost()

write("tuft", 96, 64, ell(48, 48, 40, 14, "#ffffff", 5) + ell(44, 42, 30, 9, "#ffffff", 0))

# The pickups and the props the dog meets up close are hand painted:
# presents (five designs, one rare gold), the rocket and its flame, the storm
# cloud and its bolt, the tree, the lamppost, the candy cane, the gift pile.
import props_painted
props_painted.run()

print("props written")
