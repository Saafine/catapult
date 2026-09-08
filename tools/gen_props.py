#!/usr/bin/env python3
"""Pickups, props, sky bits, sled and tower in the flat outlined cartoon
style (thick dark line, saturated flat fills, one light patch, one shade
patch). Runs after gen_svg.py and overwrites those files in the pack."""
import os, re

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
def present(name, box, ribbon, lid):
    b = rect(12, 34, 72, 54, box, 6)
    b += shade("M12,70 L84,70 L84,82 Q84,88 78,88 L18,88 Q12,88 12,82 Z", 0.2)
    b += rect(6, 22, 84, 18, lid, 5)
    b += f'<rect x="40" y="22" width="16" height="66" fill="{ribbon}"/>' + line("M40,22 L40,88 M56,22 L56,88", 4)
    b += shape("M48,22 C36,22 26,10 34,6 C40,3 46,12 48,18 C50,12 56,3 62,6 C70,10 60,22 48,22 Z", ribbon, 5)
    b += gloss(24, 44, 8, 4)
    write(name, 96, 96, b)
present("present_red", "#e6382f", "#ffd23a", "#c72d26")
present("present_green", "#2fa353", "#ff7fa5", "#22853f")
present("present_blue", "#3577e0", "#ffffff", "#2a5fb8")

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

def firework():
    b = shape("M6,56 L44,26 L44,86 Z", "#ffd23a")
    b += rect(36, 30, 150, 52, "#3fbf7a", 22)
    b += shape("M180,30 C214,36 226,50 226,56 C226,62 214,76 180,82 Z", "#ffd23a")
    b += gloss(90, 42, 40, 6, 0)
    b += circ(110, 56, 15, "#ffd23a", 5) + f'<path d="M110,46 L114,54 L122,55 L116,60 L118,68 L110,64 L102,68 L104,60 L98,55 L106,54 Z" fill="#ffffff"/>'
    b += line("M20,44 L8,36 M18,68 L6,76", 5, "#ff6b5a")
    write("firework", 232, 112, b)
firework()

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

def stormcloud():
    b = shape("M60,124 C30,124 22,90 46,80 C40,52 78,40 96,58 C104,30 150,28 160,58 C186,48 210,70 196,92 C214,100 206,126 184,126 Z", "#5a6178")
    b += shape("M70,118 C50,118 46,96 62,90 C58,72 84,64 96,78 C102,60 136,58 144,78 C160,70 178,86 168,102 C180,108 174,118 160,118 Z", "#6f778f", 0)
    b += line("M70,84 L104,96 M162,84 L128,96", 7)
    b += circ(100, 102, 8, "#ffffff", 4) + circ(132, 102, 8, "#ffffff", 4) + circ(102, 104, 4, INK) + circ(130, 104, 4, INK)
    b += line("M104,120 Q116,110 128,120", 5)
    write("stormcloud", 224, 144, b)
stormcloud()

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

# ---------------------------------------------------------------- sky
def cloud(name, w, h, seed):
    import random
    r = random.Random(seed)
    cx, cy = w / 2, h * 0.6
    body = ""
    blobs = [(cx - w * 0.28, cy, w * 0.18), (cx - w * 0.08, cy - h * 0.22, w * 0.22), (cx + w * 0.16, cy - h * 0.1, w * 0.2), (cx + w * 0.32, cy + h * 0.02, w * 0.15)]
    for (x, y, rr) in blobs:
        body += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{rr:.0f}" fill="{INK}"/>'
    body += f'<rect x="{cx - w*0.44:.0f}" y="{cy - h*0.05:.0f}" width="{w*0.9:.0f}" height="{h*0.3:.0f}" rx="{h*0.15:.0f}" fill="{INK}"/>'
    for (x, y, rr) in blobs:
        body += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{rr - LW:.0f}" fill="#ffffff"/>'
    body += f'<rect x="{cx - w*0.44 + LW:.0f}" y="{cy - h*0.05 + LW:.0f}" width="{w*0.9 - 2*LW:.0f}" height="{h*0.3 - 2*LW:.0f}" rx="{h*0.15:.0f}" fill="#ffffff"/>'
    body += f'<rect x="{cx - w*0.4:.0f}" y="{cy + h*0.12:.0f}" width="{w*0.8:.0f}" height="{h*0.1:.0f}" rx="{h*0.05:.0f}" fill="#dfe8f5"/>'
    write(name, w, h, body)

def sun():
    b = ""
    for i in range(12):
        import math
        a = i * math.pi / 6
        x0, y0 = 128 + math.cos(a) * 92, 128 + math.sin(a) * 92
        x1, y1 = 128 + math.cos(a) * 122, 128 + math.sin(a) * 122
        b += line(f"M{x0:.0f},{y0:.0f} L{x1:.0f},{y1:.0f}", 14) + line(f"M{x0:.0f},{y0:.0f} L{x1:.0f},{y1:.0f}", 7, "#ffd23a")
    b += circ(128, 128, 80, "#ffd23a") + circ(128, 128, 62, "#ffe27a", 0) + gloss(104, 104, 18, 9)
    write("sun", 256, 256, b)
sun()

# ---------------------------------------------------------------- props
def xmas_tree():
    b = rect(114, 286, 28, 44, "#8a5a30", 6)
    for (y, w) in ((300, 118), (236, 96), (176, 72)):
        b += shape(f"M128,{y - 110} L{128 + w},{y} L{128 - w},{y} Z", "#2fa353")
        b += shape(f"M128,{y - 110} L{128 + w*0.55},{y - 46} Q128,{y - 30} {128 - w*0.55},{y - 46} Z", "#ffffff", 4)
    for (x, y, c) in ((100, 250, "#e6382f"), (156, 200, "#3577e0"), (118, 300, "#ffd23a"), (170, 290, "#ff7fa5"), (140, 150, "#e6382f")):
        b += circ(x, y, 9, c, 4)
    b += f'<path d="M128,20 L136,44 L162,46 L142,62 L148,88 L128,74 L108,88 L114,62 L94,46 L120,44 Z" fill="#ffd23a" {ol(5)}/>'
    write("xmas_tree", 256, 336, b)
xmas_tree()

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

def lamppost():
    b = rect(40, 250, 16, 38, "#3a4a66", 3)
    b += rect(30, 280, 36, 8, "#3a4a66", 3, 4)
    b += rect(44, 60, 8, 200, "#3a4a66", 2, 4)
    b += shape("M22,60 L74,60 L64,20 L32,20 Z", "#3a4a66")
    b += shape("M28,56 L68,56 L60,26 L36,26 Z", "#ffe27a", 0)
    b += circ(48, 12, 8, "#3a4a66", 4)
    b += shape("M22,58 Q48,64 74,58 L74,66 Q48,72 22,66 Z", "#ffffff", 4)
    write("lamppost", 96, 288, b)
lamppost()

def candycane():
    b = line("M20,120 L20,40 C20,14 52,14 52,40 L52,50", 22) + line("M20,120 L20,40 C20,14 52,14 52,40 L52,50", 13, "#ffffff")
    for y in (110, 92, 74, 56):
        b += line(f"M13,{y} L27,{y - 8}", 6, "#e6382f")
    b += line("M28,22 L38,32 M46,44 L58,48", 6, "#e6382f")
    write("candycane", 64, 128, b)
candycane()

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

def gift_pile():
    b = rect(12, 62, 68, 58, "#2fa353", 6) + f'<rect x="40" y="62" width="14" height="58" fill="#ffd23a"/>' + line("M40,62 L40,120 M54,62 L54,120", 3)
    b += rect(114, 66, 64, 54, "#3577e0", 6) + f'<rect x="140" y="66" width="12" height="54" fill="#ffffff"/>' + line("M140,66 L140,120 M152,66 L152,120", 3)
    b += rect(60, 26, 76, 60, "#e6382f", 6) + f'<rect x="90" y="26" width="16" height="60" fill="#ffd23a"/>' + line("M90,26 L90,86 M106,26 L106,86", 3)
    b += shape("M98,26 C88,26 80,14 88,10 C94,7 98,16 98,22 C98,16 102,7 108,10 C116,14 108,26 98,26 Z", "#ffd23a", 4)
    write("gift_pile", 192, 128, b)
gift_pile()

write("tuft", 96, 64, ell(48, 48, 40, 14, "#ffffff", 5) + ell(44, 42, 30, 9, "#ffffff", 0))

def sled():
    b = line("M14,78 Q6,86 18,88 L176,88 Q188,86 180,78", 12) + line("M14,78 Q6,86 18,88 L176,88 Q188,86 180,78", 6, "#c9ced8")
    b += rect(18, 40, 156, 30, "#3577e0", 10)
    b += shade("M18,58 L174,58 L174,62 Q174,70 166,70 L26,70 Q18,70 18,62 Z", 0.2)
    b += rect(30, 24, 132, 22, "#c9915a", 6)
    b += line("M40,24 L40,46 M80,24 L80,46 M120,24 L120,46", 4)
    b += line("M40,70 L44,80 M150,70 L146,80", 8)
    b += shape("M164,40 C186,36 190,20 178,14", "#c9ced8", 0) + line("M164,40 C186,36 190,20 178,14", 10) + line("M164,40 C186,36 190,20 178,14", 5, "#c9ced8")
    write("sled", 192, 96, b)
sled()

def tower():
    # Calgary Tower in flat style: tapered white shaft, flared collar, the
    # red-topped pod with its ring of windows, the torch on top. The deck,
    # sign and door stay where the game expects them.
    b = shape("M122,790 L140,320 L180,320 L198,790 Z", "#f1f3f8")
    b += shade("M160,320 L180,320 L198,790 L160,790 Z", 0.1)
    b += line("M150,330 L154,790 M166,330 L168,790", 3, "#c9d0de")
    b += rect(92, 736, 136, 64, "#f1f3f8", 8)
    b += rect(132, 756, 56, 44, "#6a3f1c", 6)
    # Flared collar under the pod.
    b += shape("M104,318 L216,318 L242,262 L78,262 Z", "#dfe3ee")
    # Pod: white lower ring, glass window band, red roof.
    b += shape("M58,262 C58,296 262,296 262,262 L266,236 L54,236 Z", "#f1f3f8")
    b += rect(60, 236, 200, 26, "#9fd3ff", 4, 4)
    for x in range(72, 250, 24):
        b += line(f"M{x},238 L{x},260", 3, "#5a7fc0")
    b += shape("M52,238 C52,200 268,200 268,238 Z", "#e6382f")
    b += shade("M52,238 C52,222 90,212 128,210 L128,238 Z", 0.12)
    b += rect(44, 176, 232, 30, "#f4f6fb", 10)
    b += rect(24, 168, 184, 50, "#c9915a", 10)
    b += rect(34, 176, 164, 34, "#fff6e0", 6, 4)
    b += rect(14, 128, 292, 30, "#c9915a", 10)
    b += shape("M14,126 Q40,110 70,126 Q100,110 130,126 Q160,110 190,126 Q220,110 250,126 Q280,110 306,126 L306,140 L14,140 Z", "#ffffff", 4)
    b += rect(24, 90, 12, 40, "#a86a3a", 3, 4) + rect(284, 90, 12, 40, "#a86a3a", 3, 4)
    b += rect(18, 94, 284, 10, "#a86a3a", 4, 4)
    b += rect(150, 30, 20, 64, "#f1f3f8", 4)
    b += shape("M130,36 L190,36 L182,56 L138,56 Z", "#5a6178")
    b += shape("M146,38 C138,20 150,12 152,2 C158,14 166,10 168,0 C176,14 186,22 174,38 Z", "#ff8c1a", 5)
    b += shape("M154,36 C150,26 158,20 160,14 C164,22 170,26 166,36 Z", "#ffe27a", 0)
    write("tower", 320, 800, b)
tower()

# Cloud sizes follow whatever gen_svg wrote so the game's aspect maths still holds.
src = open(os.path.join(os.path.dirname(__file__), "gen_svg.py")).read()
for m in re.finditer(r'cloud\("(cloud_[abc])",\s*\[.*?\],\s*(\d+),\s*(\d+)\)', src):
    cloud(m.group(1), int(m.group(2)), int(m.group(3)), hash(m.group(1)) % 100)
print("props written")
