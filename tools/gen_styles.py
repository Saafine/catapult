#!/usr/bin/env python3
"""Five candidate art styles for the corgi pilot, written to assets/svg/style_N_*.svg.
All five share the same brief: Biscuit the corgi, Santa hat, green scarf, front view.
Only the drawing style changes."""
import os, math

PACK = os.environ.get("PACK", "calgary_xmas")
OUT = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)
W, H = 256, 320

def write(name, body, defs=""):
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">\n<defs>{defs}</defs>\n{body}\n</svg>'
    open(os.path.join(OUT, name + ".svg"), "w").write(svg)

def lg(id, stops, x1=0, y1=0, x2=0, y2=1):
    s = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{a if a is not None else 1}"/>' for o, c, a in [(x[0], x[1], x[2] if len(x) > 2 else None) for x in stops])
    return f'<linearGradient id="{id}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">{s}</linearGradient>'
def rg(id, stops, cx=0.35, cy=0.3, r=0.8):
    s = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{a if a is not None else 1}"/>' for o, c, a in [(x[0], x[1], x[2] if len(x) > 2 else None) for x in stops])
    return f'<radialGradient id="{id}" cx="{cx}" cy="{cy}" r="{r}">{s}</radialGradient>'
def ol(c, w):
    return f'stroke="{c}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'

FUR, FURD, FURL, CREAM = "#f0a64f", "#c9823a", "#ffc97a", "#fff3e0"
GREEN, GREEND = "#2f8f63", "#1c5c3e"
RED, REDD = "#e63c3c", "#8f1e2a"
INK = "#2b1a12"

# ---------------------------------------------------------------- 2 flat modern
def flat_modern():
    # No outlines; every form is two flat tones split by a diagonal light.
    d = lg("split", [(0, FUR), (0.5, FUR), (0.5, FURD), (1, FURD)], x1=0, y1=0, x2=1, y2=1)
    d += lg("splitr", [(0, RED), (0.5, RED), (0.5, REDD), (1, REDD)], x1=0, y1=0, x2=1, y2=1)
    d += lg("splitg", [(0, GREEN), (0.5, GREEN), (0.5, GREEND), (1, GREEND)], x1=0, y1=0, x2=1, y2=1)
    b = ""
    # legs
    for x in (86, 146):
        b += f'<rect x="{x}" y="236" width="28" height="56" rx="14" fill="url(#split)"/>'
    # body
    b += '<rect x="66" y="176" width="124" height="96" rx="48" fill="url(#split)"/>'
    b += '<ellipse cx="128" cy="240" rx="40" ry="26" fill="#fff3e0"/>'
    # scarf
    b += '<rect x="74" y="172" width="108" height="26" rx="13" fill="url(#splitg)"/><path d="M150,190 L166,236 L188,226 L170,186 Z" fill="#2f8f63"/>'
    # ears
    b += '<path d="M56,120 L40,44 L100,86 Z" fill="#c9823a"/><path d="M200,120 L216,44 L156,86 Z" fill="#f0a64f"/>'
    b += '<path d="M62,112 L52,62 L92,90 Z" fill="#f7c8b8"/><path d="M194,112 L204,62 L164,90 Z" fill="#f7c8b8"/>'
    # head
    b += '<circle cx="128" cy="128" r="66" fill="url(#split)"/>'
    b += '<path d="M128,62 C150,62 170,84 172,110 L128,110 Z" fill="#fff3e0" opacity="0.0"/>'
    b += '<ellipse cx="128" cy="152" rx="36" ry="26" fill="#fff3e0"/>'
    b += '<path d="M118,64 L138,64 L134,120 L122,120 Z" fill="#fff3e0"/>'
    # eyes, nose
    b += '<ellipse cx="102" cy="122" rx="9" ry="12" fill="#2b1a12"/><ellipse cx="154" cy="122" rx="9" ry="12" fill="#2b1a12"/>'
    b += '<circle cx="105" cy="118" r="3" fill="#ffffff"/><circle cx="157" cy="118" r="3" fill="#ffffff"/>'
    b += '<path d="M116,142 L140,142 L128,156 Z" fill="#2b1a12"/>'
    b += '<path d="M120,164 Q128,170 136,164" fill="none" stroke="#2b1a12" stroke-width="3" stroke-linecap="round"/>'
    b += '<circle cx="86" cy="150" r="7" fill="#ffb0a0" opacity="0.6"/><circle cx="170" cy="150" r="7" fill="#ffb0a0" opacity="0.6"/>'
    # hat
    b += '<path d="M70,84 C90,36 150,26 186,30 L196,22 C188,60 176,74 178,84 Z" fill="url(#splitr)"/><rect x="62" y="76" width="122" height="20" rx="10" fill="#ffffff"/><circle cx="198" cy="20" r="12" fill="#ffffff"/>'
    write("style_2_flat", b, d)

# ---------------------------------------------------------------- 3 rubber-hose cartoon
def rubber_hose():
    O = "#111111"
    b = ""
    # hose legs and gloved paws
    for sx in (-1, 1):
        x0 = 128 + sx * 26
        b += f'<path d="M{x0},240 C{x0 + sx*6},262 {x0 + sx*4},280 {x0 + sx*14},296" fill="none" stroke="{O}" stroke-width="20" stroke-linecap="round"/><path d="M{x0},240 C{x0 + sx*6},262 {x0 + sx*4},280 {x0 + sx*14},296" fill="none" stroke="{FUR}" stroke-width="12" stroke-linecap="round"/>'
        b += f'<ellipse cx="{x0 + sx*18}" cy="300" rx="22" ry="12" fill="#ffffff" {ol(O, 5)}/><path d="M{x0 + sx*10},304 L{x0 + sx*10},296 M{x0 + sx*20},305 L{x0 + sx*20},296" stroke="{O}" stroke-width="3" stroke-linecap="round"/>'
    # pear body
    b += f'<path d="M84,182 C60,220 60,262 128,262 C196,262 196,220 172,182 Z" fill="{FUR}" {ol(O, 6)}/>'
    b += f'<ellipse cx="128" cy="234" rx="34" ry="22" fill="{CREAM}" {ol(O, 4)}/>'
    # arms: hoses out to gloves
    b += f'<path d="M84,196 C52,196 40,214 30,236" fill="none" stroke="{O}" stroke-width="20" stroke-linecap="round"/><path d="M84,196 C52,196 40,214 30,236" fill="none" stroke="{FUR}" stroke-width="12" stroke-linecap="round"/>'
    b += f'<path d="M172,196 C204,196 216,214 226,236" fill="none" stroke="{O}" stroke-width="20" stroke-linecap="round"/><path d="M172,196 C204,196 216,214 226,236" fill="none" stroke="{FUR}" stroke-width="12" stroke-linecap="round"/>'
    for x in (24, 232):
        b += f'<circle cx="{x}" cy="246" r="18" fill="#ffffff" {ol(O, 5)}/><path d="M{x-6},252 L{x-6},240 M{x+2},254 L{x+2},240 M{x+9},250 L{x+9},242" stroke="{O}" stroke-width="3" stroke-linecap="round"/>'
    # scarf
    b += f'<path d="M78,176 Q128,200 178,176 L182,196 Q128,222 74,196 Z" fill="{GREEN}" {ol(O, 5)}/><path d="M150,198 L164,238 L190,230 L172,192 Z" fill="{GREEN}" {ol(O, 5)}/>'
    # floppy ears with bounce
    b += f'<path d="M78,100 C40,70 24,40 44,30 C64,24 88,60 100,88 Z" fill="{FURD}" {ol(O, 6)}/><path d="M178,100 C216,70 232,40 212,30 C192,24 168,60 156,88 Z" fill="{FURD}" {ol(O, 6)}/>'
    # head: big round, cheeks
    b += f'<path d="M52,130 C52,70 90,54 128,54 C166,54 204,70 204,130 C204,178 170,200 128,200 C86,200 52,178 52,130 Z" fill="{FUR}" {ol(O, 6)}/>'
    b += f'<ellipse cx="128" cy="160" rx="46" ry="30" fill="{CREAM}" {ol(O, 4)}/>'
    # pie-cut eyes
    for ex in (100, 156):
        b += f'<ellipse cx="{ex}" cy="118" rx="20" ry="26" fill="#ffffff" {ol(O, 5)}/>'
        b += f'<path d="M{ex + 4},128 L{ex + 4},112 A16,16 0 1,0 {ex + 16},124 Z" fill="{O}"/>'
    b += f'<path d="M78,88 Q100,78 118,90 M138,90 Q156,78 178,88" fill="none" stroke="{O}" stroke-width="5" stroke-linecap="round"/>'
    # nose and wide grin with tongue
    b += f'<ellipse cx="128" cy="146" rx="16" ry="11" fill="{O}"/><ellipse cx="122" cy="142" rx="5" ry="3" fill="#ffffff"/>'
    b += f'<path d="M92,164 Q128,200 164,164 Z" fill="#7a1a22" {ol(O, 4)}/><path d="M108,172 Q128,190 148,172 Q140,196 116,196 Z" fill="#ff7f8f"/>'
    b += f'<path d="M92,164 Q128,178 164,164" fill="none" stroke="#ffffff" stroke-width="7" stroke-linecap="round"/>'
    # hat, tilted, with a polka pompom
    b += f'<path d="M64,86 C80,30 150,14 196,26 L210,10 C204,56 190,72 186,88 Z" fill="{RED}" {ol(O, 6)}/><rect x="56" y="78" width="140" height="22" rx="11" fill="#ffffff" {ol(O, 5)}/><circle cx="212" cy="12" r="14" fill="#ffffff" {ol(O, 5)}/>'
    write("style_3_rubberhose", b)

# ---------------------------------------------------------------- 4 kawaii sticker
def kawaii():
    P_FUR, P_FURD, P_CREAM = "#ffc46b", "#f2a94a", "#fff8ee"
    ST = "#ffffff"   # sticker border
    G = "#8a8a8a"
    shape = "M40,150 C40,80 78,54 128,54 C178,54 216,80 216,150 C216,206 180,236 128,236 C76,236 40,206 40,150 Z"
    body = "M84,222 C70,250 70,300 128,300 C186,300 186,250 172,222 Z"
    b = ""
    # sticker border: white fat outline behind everything
    b += f'<path d="{shape}" fill="{ST}" {ol(ST, 26)}/><path d="{body}" fill="{ST}" {ol(ST, 26)}/>'
    b += f'<path d="M56,96 C40,64 46,40 66,44 C80,48 92,70 96,90 Z M200,96 C216,64 210,40 190,44 C176,48 164,70 160,90 Z" fill="{ST}" {ol(ST, 26)}/>'
    b += f'<path d="{shape}" fill="none" {ol(G, 18)}/><path d="{body}" fill="none" {ol(G, 18)}/>'
    b += f'<path d="M56,96 C40,64 46,40 66,44 C80,48 92,70 96,90 Z M200,96 C216,64 210,40 190,44 C176,48 164,70 160,90 Z" fill="none" {ol(G, 18)}/>'
    # legs nubs
    b += f'<ellipse cx="100" cy="296" rx="18" ry="10" fill="{P_FUR}"/><ellipse cx="156" cy="296" rx="18" ry="10" fill="{P_FUR}"/>'
    b += f'<path d="{body}" fill="{P_FUR}"/><ellipse cx="128" cy="268" rx="30" ry="20" fill="{P_CREAM}"/>'
    # scarf as a soft band
    b += f'<path d="M82,222 Q128,244 174,222 L178,240 Q128,262 78,240 Z" fill="#8fd6b0"/><path d="M148,238 L160,270 L182,262 L168,232 Z" fill="#8fd6b0"/>'
    # ears
    b += f'<path d="M56,96 C40,64 46,40 66,44 C80,48 92,70 96,90 Z" fill="{P_FURD}"/><path d="M200,96 C216,64 210,40 190,44 C176,48 164,70 160,90 Z" fill="{P_FURD}"/>'
    b += f'<path d="M64,86 C56,68 60,54 70,56 C78,60 84,72 86,84 Z" fill="#ffd6d0"/><path d="M192,86 C200,68 196,54 186,56 C178,60 172,72 170,84 Z" fill="#ffd6d0"/>'
    # head
    b += f'<path d="{shape}" fill="{P_FUR}"/>'
    b += f'<ellipse cx="128" cy="176" rx="46" ry="30" fill="{P_CREAM}"/>'
    # tiny wide-set dot eyes, blush, w mouth
    b += '<circle cx="90" cy="140" r="7" fill="#3a2a2a"/><circle cx="166" cy="140" r="7" fill="#3a2a2a"/>'
    b += '<circle cx="93" cy="137" r="2.2" fill="#ffffff"/><circle cx="169" cy="137" r="2.2" fill="#ffffff"/>'
    b += '<ellipse cx="74" cy="164" rx="12" ry="7" fill="#ffb3c1" opacity="0.8"/><ellipse cx="182" cy="164" rx="12" ry="7" fill="#ffb3c1" opacity="0.8"/>'
    b += '<ellipse cx="128" cy="166" rx="9" ry="6" fill="#3a2a2a"/>'
    b += '<path d="M116,180 Q122,188 128,180 Q134,188 140,180" fill="none" stroke="#3a2a2a" stroke-width="3" stroke-linecap="round"/>'
    # small hat perched on top
    b += f'<path d="M92,78 C104,40 150,30 178,36 L188,26 C184,56 176,66 178,78 Z" fill="#ff6b6b" {ol(ST, 10)}/><path d="M92,78 C104,40 150,30 178,36 L188,26 C184,56 176,66 178,78 Z" fill="#ff6b6b"/><rect x="84" y="72" width="102" height="18" rx="9" fill="#ffffff" {ol(G, 3)}/><circle cx="190" cy="24" r="11" fill="#ffffff" {ol(G, 3)}/>'
    write("style_4_kawaii", b)

# ---------------------------------------------------------------- 5 pixel art
PIX = """
..........rrrrr..........
.........rrrrrrr....ww...
........rrrrrrrrr..wwww..
.......rrrrrrrrrrrr.ww...
......wwwwwwwwwwwwww.....
.oo...wwwwwwwwwwwww..oo..
.odo..ooooooooooooo..odo.
.oddo.offffffffffffo.oddo
.oddooffffffffffffffooddo
..oddoffffcffffcffffoddo.
..oddoffffcffffcffffoddo.
...odffffcwkffcwkffffdo..
...offffffkkffffkkfffffo.
...offffcccccccccccffffo.
...offffccckkkkcccccffffo
....offfcccckkkkcccfffo..
....offffccccccccccfffo..
.....offfccccpccccffo....
......offffcccccffffo....
.......ooofffffffooo.....
.........ooooooooo.......
.......ogggggggggo.......
......ogggggggggggo......
.....ogggggggggggggo.....
....offooooooooooooffo...
...offffoffffffffoffffo..
...offffoffcccccffoffffo.
...ofoooffcccccccffooofo.
....o..offcccccccffo..o..
.......offfcccccfffo.....
.......ofoffffffofo......
.......ofo.ffff.ofo......
.......ooo.oooo.ooo......
"""
def pixel():
    pal = {"o": "#2b1a12", "f": "#f0a64f", "d": "#c9823a", "c": "#fff3e0", "w": "#ffffff", "k": "#1a1216", "r": "#e63c3c", "g": "#2f8f63", "p": "#ff7f8f"}
    rows = [r for r in PIX.strip("\n").split("\n")]
    cw = len(rows[0]); ch = len(rows)
    s = W / cw
    b = f'<g shape-rendering="crispEdges" transform="translate(0 {(H - ch * s) / 2:.1f})">'
    for y, r in enumerate(rows):
        for x, ch_ in enumerate(r):
            if ch_ in pal:
                b += f'<rect x="{x * s:.2f}" y="{y * s:.2f}" width="{s + 0.05:.2f}" height="{s + 0.05:.2f}" fill="{pal[ch_]}"/>'
    b += "</g>"
    write("style_5_pixel", b)

flat_modern(); rubber_hose(); kawaii(); pixel()
print("styles written")
