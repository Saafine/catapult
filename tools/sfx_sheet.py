#!/usr/bin/env python3
"""Contact sheet of the synthesized sounds: waveform + log-frequency
spectrogram per sound, from the `.env`/`.spec` files that
`cargo run -p hamsters-synth --release --bin sfx_render -- <dir>` writes.

usage: python3 tools/sfx_sheet.py <render_dir> <out.png> [--wide <name> <marks.json>] [name ...]

`--wide` adds a full-width panel on top (e.g. a capture of the in-game mix
analyzed with `sfx_render --analyze`), with the key presses from the
capture's JSON marked on the timeline.
"""
import json
import math
import os
import sys

from PIL import Image, ImageDraw, ImageFont

W, H = 360, 120  # must match sfx_render.rs
STOPS = [(0.0, (8, 6, 22)), (0.25, (60, 15, 110)), (0.5, (170, 40, 100)), (0.72, (240, 110, 50)), (0.9, (252, 205, 90)), (1.0, (255, 250, 220))]
LUT = []
for i in range(256):
    k = i / 255
    for (a, ca), (b, cb) in zip(STOPS, STOPS[1:]):
        if a <= k <= b:
            t = (k - a) / (b - a)
            LUT.append(tuple(int(ca[j] + (cb[j] - ca[j]) * t) for j in range(3)))
            break
DEFAULT = ["click", "lock", "slide", "slide_fast", "launch", "flap", "wind_high", "present_8",
           "firework", "rocket", "zap", "freeze", "kaboom", "record", "results_best", "results_crash"]


def font(size):
    for f in ["/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf"]:
        if os.path.exists(f):
            return ImageFont.truetype(f, size)
    return ImageFont.load_default()


def stats_for(d):
    rows = {}
    p = os.path.join(d, "stats.md")
    if not os.path.exists(p):
        return rows
    for line in open(p):
        c = [x.strip() for x in line.strip().strip("|").split("|")]
        if len(c) > 10 and c[1] == "0":
            rows[c[0]] = c
    return rows


def cell(d, name, st, title=None, tint=(24, 26, 34)):
    cw, ch = W + 20, H + 60 + 46
    im = Image.new("RGB", (cw, ch), tint)
    dr = ImageDraw.Draw(im)
    dr.text((10, 4), title or name, fill=(255, 236, 190), font=font(15))
    # Waveform envelope.
    env = [tuple(map(float, l.split())) for l in open(os.path.join(d, name + ".env"))]
    top, mid = 26, 26 + 26
    dr.rectangle([10, top, 10 + W, top + 52], fill=(14, 16, 22))
    dr.line([10, mid, 10 + W, mid], fill=(60, 64, 80))
    for x, (lo, hi) in enumerate(env):
        dr.line([10 + x, mid - hi * 26, 10 + x, mid - lo * 26], fill=(120, 200, 255))
    # Spectrogram.
    raw = open(os.path.join(d, name + ".spec"), "rb").read()
    sp = Image.new("RGB", (W, H))
    sp.putdata([LUT[b] for b in raw])
    im.paste(sp, (10, top + 58))
    f9 = font(9)
    for hz, lab in [(100, "100"), (1000, "1k"), (10000, "10k")]:
        import math
        k = math.log(hz / 40) / math.log(16000 / 40)
        y = top + 58 + int((1 - k) * (H - 1))
        dr.line([10, y, 16, y], fill=(255, 255, 255))
        dr.text((18, y - 6), lab, fill=(220, 220, 220), font=f9)
    stat = os.path.join(d, name + ".stat")
    if name in st:
        c = st[name]
        txt = f"{c[2]} s   peak {c[3]} dB   loud {c[4]} dB   bright {c[13]} Hz"
        dr.text((10, top + 58 + H + 4), txt, fill=(190, 196, 210), font=font(11))
    elif os.path.exists(stat):
        c = open(stat).read().strip().split("|")
        txt = f"{c[0]} s   peak {c[1]} dB   loud {c[2]} dB   bright {c[3]} Hz"
        dr.text((10, top + 58 + H + 4), txt, fill=(190, 196, 210), font=font(11))
    return im


def wide_panel(d, name, marks_path, width):
    raw = open(os.path.join(d, name + ".spec"), "rb").read()
    sw = len(raw) // H
    env = [tuple(map(float, l.split())) for l in open(os.path.join(d, name + ".env"))]
    run = json.load(open(marks_path))
    secs = run["secs"]
    im = Image.new("RGB", (width, H * 2 + 120), (24, 26, 34))
    dr = ImageDraw.Draw(im)
    dr.text((10, 4), "IN-GAME MIX, captured after the master limiter in headless Chrome (music + loops + one-shots): " + f"{secs:.1f} s, peak {20 * math.log10(max(run['peak'], 1e-9)):.1f} dBFS", fill=(255, 236, 190), font=font(15))
    x0, top = 10, 30
    scale = (width - 20) / sw
    dr.rectangle([x0, top, x0 + sw * scale, top + 70], fill=(14, 16, 22))
    mid = top + 35
    for x, (lo, hi) in enumerate(env):
        X = x0 + x * scale
        dr.line([X, mid - hi * 35, X, mid - lo * 35], fill=(120, 200, 255))
    sp = Image.new("RGB", (sw, H))
    sp.putdata([LUT[b] for b in raw])
    sp = sp.resize((int(sw * scale), H * 2 - 40))
    im.paste(sp, (x0, top + 76))
    labels = {"Digit2": "pick pilot", "P": "lock+perfect jump", "Space": "flap", "F": "fireworks", "Z": "freeze", "C": "crash dive", "KeyR": "restart", "V": "meteor drop"}
    f10 = font(10)
    last_x = -100
    for t, k in run["marks"]:
        X = x0 + t / secs * sw * scale
        dr.line([X, top, X, top + 76 + H * 2 - 40], fill=(255, 255, 255))
        y = top + 76 + H * 2 - 36 + (12 if X - last_x < 70 else 0)
        dr.text((X + 2, y), labels.get(k, k), fill=(255, 255, 255), font=f10)
        last_x = X
    return im


def main():
    args = sys.argv[1:]
    wide = None
    if "--wide" in args:
        i = args.index("--wide")
        wide = (args[i + 1], args[i + 2])
        del args[i:i + 3]
    pairs = []
    if "--compare" in args:
        i = args.index("--compare")
        pairs = args[i + 1].split(",")
        del args[i:i + 2]
    d, out = args[0], args[1]
    names = args[2:] or DEFAULT
    names = [n for n in names if os.path.exists(os.path.join(d, n + ".spec"))]
    st = stats_for(d)
    cols = 4
    cells = [cell(d, n, st) for n in names]
    cw, ch = cells[0].size
    rows = (len(cells) + cols - 1) // cols
    head = 54
    wp = wide_panel(d, wide[0], wide[1], cols * cw) if wide else None
    wh = wp.size[1] + 6 if wp else 0
    sheet = Image.new("RGB", (cols * cw + 10, rows * ch + head + wh + 6), (16, 17, 23))
    if wp:
        sheet.paste(wp, (5, head))
    head += wh
    if pairs:
        cmp_cells = []
        for p in pairs:
            cmp_cells.append(cell(d, "old_" + p, st, "BEFORE  " + p, (44, 24, 28)))
            cmp_cells.append(cell(d, p, st, "AFTER  " + p, (22, 38, 30)))
        crows = (len(cmp_cells) + cols - 1) // cols
        ext = Image.new("RGB", (sheet.size[0], sheet.size[1] + crows * ch + 30), (16, 17, 23))
        ext.paste(sheet, (0, 0))
        sheet = ext
        dr = ImageDraw.Draw(sheet)
        dr.text((12, head + 4), "BEFORE (previous build's synth) vs AFTER, same events", fill=(255, 255, 255), font=font(16))
        head += 30
        for i, c in enumerate(cmp_cells):
            sheet.paste(c, (5 + (i % cols) * cw, head + (i // cols) * ch))
        head += crows * ch
    dr = ImageDraw.Draw(sheet)
    dr.text((12, 8), "Santa's Flight School - synthesized soundscape (waveform + spectrogram 40 Hz-16 kHz, -90..0 dB)", fill=(255, 255, 255), font=font(18))
    dr.text((12, 32), "every sound is layered synthesis: noise + tonal, ADSR, SVF sweeps, FM/detuned voices, drive, echo, Freeverb; peak <= -1 dBFS, edges exactly 0", fill=(170, 176, 196), font=font(12))
    for i, c in enumerate(cells):
        sheet.paste(c, (5 + (i % cols) * cw, head + (i // cols) * ch))
    sheet.save(out)
    print(out, sheet.size)


main()
