#!/usr/bin/env python3
"""Painted landscape layers after the designers' key art: the Rockies
(far range and front range), forested foothills, snowy rolling hills with
spruce groves, the prairie treeline, the meadow fill and the ground crust.

Every tile is seamless (periodic noise, things near an edge are drawn on
both sides). Aerial perspective is baked in: far layers are bluer, paler
and softer, near layers carry saturated paint, crisp edges and detail.
Light comes from the upper left: lit faces warm white-lavender, shadow
faces cool blue-violet; outlines are a deeper shade of each fill.

Called from gen_paint.py (and gen_svg.py for the treeline/meadow).
Each file carries data-paint="done", so tools/paint.py only adds grain."""
import math, os, random
from paint import mix

PACK = os.environ.get("PACK", "calgary_xmas")
HERE = os.path.dirname(__file__)
OUT = os.path.join(HERE, "..", "packs", PACK, "svg")


# Distant layers are drawn at most ~250 px tall, so they rasterise at 1x
# (half the download and texture memory, no visible loss).
SCALE_1X = {"mountains_far", "mountains_near", "hills_far"}


def write(name, w, h, body, defs=""):
    os.makedirs(OUT, exist_ok=True)
    sc = ' data-scale="1"' if name in SCALE_1X else ""
    open(os.path.join(OUT, name + ".svg"), "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" data-paint="done"{sc}>\n'
        f'<defs>{defs}</defs>\n{body}\n</svg>')


def lg(gid, stops, x2=0, y2=1, x1=0, y1=0):
    s = ""
    for st in stops:
        a = f' stop-opacity="{st[2]}"' if len(st) > 2 else ""
        s += f'<stop offset="{st[0]}" stop-color="{st[1]}"{a}/>'
    return f'<linearGradient id="{gid}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">{s}</linearGradient>'


def rg(gid, stops, cx=0.5, cy=0.5, r=0.5):
    s = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{a}"/>' for o, c, a in stops)
    return f'<radialGradient id="{gid}" cx="{cx}" cy="{cy}" r="{r}">{s}</radialGradient>'


def poly(pts, close=True):
    d = "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts)
    return d + (" Z" if close else "")


def smooth_path(pts, close_to=None):
    """Quadratic-smoothed path through pts; close_to=(W,H) closes down to the bottom."""
    d = f"M{pts[0][0]:.1f},{pts[0][1]:.1f} "
    for k in range(1, len(pts) - 1):
        x0, y0 = pts[k]; x1, y1 = pts[k + 1]
        d += f"Q{x0:.1f},{y0:.1f} {(x0 + x1) / 2:.1f},{(y0 + y1) / 2:.1f} "
    d += f"L{pts[-1][0]:.1f},{pts[-1][1]:.1f} "
    if close_to:
        W, H = close_to
        d += f"L{W},{H} L0,{H} Z"
    return d


class Periodic:
    """Seamless 1-D noise on [0, W): a sum of sines with integer cycle counts."""
    def __init__(self, W, rnd, octaves):
        self.W = W
        self.terms = []
        for cycles, amp in octaves:
            for _ in range(2):
                self.terms.append((amp * rnd.uniform(0.5, 1.0), cycles + rnd.randint(0, max(1, cycles // 3)), rnd.uniform(0, math.tau)))

    def __call__(self, x):
        return sum(a * math.sin(math.tau * k * x / self.W + p) for a, k, p in self.terms)


def wrap_copies(x, W, margin):
    """Offsets at which something at x must be drawn so the tile is seamless."""
    out = [0]
    if x < margin:
        out.append(W)
    if x > W - margin:
        out.append(-W)
    return out


# --------------------------------------------------------------- mountains
def mountains(name, W, H, seed, n_peaks, pal, height=0.86, jag=1.0, detail=1.0, base=0.10):
    """A Rockies range. Summits are sharp and rocky; each casts a shadow face
    down its right flank with a zig-zag spur; snow fills the high bowls and
    runs down couloirs as fingers, turning blue where it sits in shadow;
    strata strokes give the rock its grain; a haze rises from the foot."""
    r = random.Random(seed)
    peaks = []
    for i in range(n_peaks):
        cx = (i + r.uniform(0.15, 0.85)) * W / n_peaks
        hh = r.uniform(0.55, 1.0) * H * height
        w = r.uniform(0.45, 0.85) * W / n_peaks
        peaks.append((cx, w, hh, r.uniform(-0.35, 0.35)))
        # A shoulder peak beside most summits.
        if r.random() < 0.75:
            side = r.choice((-1, 1))
            peaks.append((cx + side * w * r.uniform(0.35, 0.6), w * 0.55, hh * r.uniform(0.55, 0.8), r.uniform(-0.3, 0.3)))
    rough = Periodic(W, r, [(37, 1.0), (71, 0.6), (131, 0.35), (233, 0.2)])
    def ridge_h(x):
        best = 0.0
        for cx, w, hh, skew in peaks:
            for off in (-W, 0, W):
                dx = x - (cx + off)
                sw = w * (1 + skew) if dx > 0 else w * (1 - skew)
                t = abs(dx) / sw
                if t < 1:
                    best = max(best, hh * (1 - t) ** 1.45)
        # Rocky jaggedness, stronger up high.
        k = 0.35 + 0.65 * best / (H * height)
        return best + H * base + rough(x) * 7 * jag * k
    step = 4
    xs = list(range(0, W + 1, step))
    hs = [ridge_h(x) for x in xs]
    ridge = [(x, H - h) for x, h in zip(xs, hs)]
    sil = poly(ridge + [(W, H), (0, H)])
    defs = lg("mbody", [(0, pal["lit"]), (0.55, pal["body"]), (1, pal["foot"])])
    defs += lg("msnow", [(0, "#ffffff"), (0.6, pal["snow"]), (1, pal["snow_lo"])])
    defs += f'<clipPath id="sil"><path d="{sil}"/></clipPath>'
    defs += lg("haze", [(0, pal["haze"], 0), (0.55, pal["haze"], 0.35), (1, pal["haze"], 0.95)])
    defs += lg("shadeg", [(0, pal["shadow"], 0.95), (0.6, pal["shadow"], 0.85), (1, pal["reflect"], 0.6)])
    b = f'<path d="{sil}" fill="url(#mbody)"/>'
    b += '<g clip-path="url(#sil)">'

    # Snow: from the ridge down to a ragged snow line, deeper on high ground,
    # with couloir fingers hanging from the bowls.
    snowline = Periodic(W, r, [(9, 1.0), (23, 0.5), (61, 0.3)])
    depth = []
    for x, h in zip(xs, hs):
        lim = H * (0.5 + 0.07 * snowline(x) / 1.8)
        depth.append(max(0.0, (h - lim) * 0.55) + (3 if h > lim * 0.92 else 0))
    for _ in range(int(n_peaks * 7 * detail)):
        fx = r.uniform(0, W)
        i0 = int(fx / step)
        if hs[i0] < H * 0.45:
            continue
        L = r.uniform(30, 90) * (hs[i0] / H)
        fw = r.uniform(5, 14)
        for j in range(len(xs)):
            for off in (-W, 0, W):
                t = abs(xs[j] - fx - off) / fw
                if t < 1 and depth[j] > 0:
                    depth[j] += L * (1 - t) ** 1.5
    top = [(x, y - 3) for x, y in ridge]
    bot = [(x, y + d) for (x, y), d in zip(ridge, depth)]
    # Each snow run as its own polygon so snow-free stretches stay rock.
    runs, cur = [], []
    for i in range(len(xs)):
        if depth[i] > 0.5:
            cur.append(i)
        elif cur:
            runs.append(cur); cur = []
    if cur:
        runs.append(cur)
    for run in runs:
        pts = [top[i] for i in run] + [bot[i] for i in reversed(run)]
        b += f'<path d="{poly(pts)}" fill="url(#msnow)" stroke="{pal["snow_edge"]}" stroke-width="1.6" stroke-linejoin="round"/>'

    # Shadow faces: from each summit, a zig-zag spur down to the foot; the
    # face between the spur and the next valley to the right is in shade.
    maxima = [i for i in range(2, len(hs) - 2) if hs[i] >= max(hs[i - 2:i + 3]) and hs[i] > H * (base + 0.18)]
    # Keep only distinct summits (local maxima of a smoothed ridge).
    sm = [sum(hs[max(0, i - 6):i + 7]) / len(hs[max(0, i - 6):i + 7]) for i in range(len(hs))]
    summits = []
    for i in maxima:
        lo, hi = max(0, i - 10), min(len(hs), i + 11)
        if sm[i] >= max(sm[lo:hi]) - 1.0 and (not summits or i - summits[-1] > 8):
            summits.append(i)
    shade = ""
    spurs = ""
    for i in summits:
        sx, sy = ridge[i]
        # Valley to the right: walk down until the ridge rises again a lot.
        j = i
        while j < len(hs) - 1 and (hs[j + 1] <= hs[j] + 2.5 or hs[j] > hs[i] * 0.97):
            j += 1
        vx = xs[j]
        span = vx - sx
        fall = H - sy
        pts = [(sx, sy)]
        n = 9
        drift = r.uniform(0.15, 0.45)
        for k in range(1, n + 1):
            t = k / n
            jitter = (r.uniform(-1, 1) * 9 + (k % 2) * 12) * detail
            pts.append((sx + span * drift * t + jitter, sy + fall * t))
        ryj = ridge[j][1]
        pts.append((vx - span * 0.12, ryj + (H - ryj) * 0.45))
        pts.append((vx - span * 0.3, H))
        back = [ridge[q] for q in range(j, i, -1)]
        for off in wrap_copies(sx, W, span + 20):
            shade += f'<path d="{poly([(x + off, y) for x, y in pts + back])}" fill="url(#shadeg)"/>'
            # The spur crest: a light stroke along the edge of the lit face.
            spurs += f'<path d="{poly([(x + off - 2, y) for x, y in pts[:-1]], False)}" fill="none" stroke="{pal["crest"]}" stroke-width="{2.6 * detail:.1f}" opacity="0.8" stroke-linejoin="round"/>'
        # Secondary gullies on the lit face, left of the summit.
        for g in range(int(2 * detail) + 1):
            gx = sx - r.uniform(0.15, 0.7) * span
            gi = max(0, min(len(xs) - 1, int(gx / step)))
            gy = ridge[gi][1] + r.uniform(8, 20)
            if gy > H * 0.9:
                continue
            gpts = [(gx, gy)]
            for k in range(1, 6):
                gpts.append((gx + k * r.uniform(1, 6) + (k % 2) * 6, gy + (H - gy) * 0.6 * k / 5))
            for off in wrap_copies(gx, W, 60):
                spurs += f'<path d="{poly([(x + off, y) for x, y in gpts], False)}" fill="none" stroke="{pal["shadow"]}" stroke-width="{3.2 * detail:.1f}" opacity="0.45" stroke-linecap="round" stroke-linejoin="round"/>'
    b += shade + spurs

    # Strata: short strokes slanting with the bedding of the front ranges.
    strata = ""
    for _ in range(int(W / 14 * detail)):
        x = r.uniform(0, W)
        hi = ridge_h(x)
        y = r.uniform(H - hi + 14, H * 0.85)
        L = r.uniform(14, 44) * detail
        ang = r.uniform(-0.12, 0.22)
        high = y < H - hi * 0.55
        col = "#ffffff" if high and r.random() < 0.6 else pal["shadow"]
        op = r.uniform(0.45, 0.75) if col == "#ffffff" else r.uniform(0.2, 0.38)
        for off in wrap_copies(x, W, 50):
            strata += f'<path d="M{x + off:.1f},{y:.1f} q{L * 0.5:.1f},{L * ang - 3:.1f} {L:.1f},{L * ang:.1f}" fill="none" stroke="{col}" stroke-width="{r.uniform(1.6, 3.0):.1f}" opacity="{op:.2f}" stroke-linecap="round"/>'
    b += strata
    # Treeline at the foot of the near range: dark spruce pricks.
    if pal.get("trees"):
        tc = pal["trees"]
        for _ in range(int(W / 5)):
            x = r.uniform(0, W)
            hi = ridge_h(x)
            y = H - r.uniform(0, min(hi - H * base * 0.5, H * 0.2))
            th = r.uniform(8, 16)
            for off in wrap_copies(x, W, 20):
                strata += ""
                b += f'<path d="M{x + off:.1f},{y - th:.1f} L{x + off + th * 0.28:.1f},{y:.1f} L{x + off - th * 0.28:.1f},{y:.1f} Z" fill="{tc}" opacity="0.8"/>'
    b += f'<rect x="0" y="{H * 0.45:.0f}" width="{W}" height="{H * 0.55:.0f}" fill="url(#haze)"/>'
    b += "</g>"
    # Lit ridge: a bright stroke where the ridge climbs toward the light.
    lit_d = ""
    for (x0, y0), (x1, y1) in zip(ridge, ridge[1:]):
        if y1 < y0 - 0.3:
            lit_d += f"M{x0:.1f},{y0 + 1.5:.1f} L{x1:.1f},{y1 + 1.5:.1f} "
    b += f'<path d="{lit_d}" stroke="#ffffff" stroke-width="2.4" opacity="0.75" fill="none" stroke-linecap="round"/>'
    b += f'<path d="{poly(ridge, False)}" fill="none" stroke="{pal["edge"]}" stroke-width="{pal.get("edge_w", 2.4)}" stroke-linejoin="round"/>'
    write(name, W, H, b, defs)


# ----------------------------------------------------------------- spruces
def spruce(x, y, h, rnd, pal, tiers=None, snow=1.0, lean=0.0):
    """A painted snowy spruce standing at (x, y): a stack of drooping,
    scalloped branch tiers lit from the left, a blue-white snow load on
    each tier, the outline a deeper green."""
    body, lit, dark, edge, snow_c, snow_e = pal
    tiers = tiers or max(3, min(7, int(h / 22)))
    w = h * rnd.uniform(0.3, 0.38)
    s = f'<rect x="{x - h * 0.035:.1f}" y="{y - h * 0.14:.1f}" width="{h * 0.07:.1f}" height="{h * 0.16:.1f}" fill="#5e3e2a"/>'
    for k in range(tiers):
        t0 = k / tiers
        t1 = (k + 1) / tiers
        ytop = y - h + h * 0.86 * t0 * 0.92
        ybot = y - h * 0.1 - h * 0.88 * (1 - t1) * 0.95
        hw = w * (0.32 + 0.68 * t1)
        tx = x + lean * (1 - t0) * h * 0.08
        # Drooping tier: concave sides, a scalloped hem.
        hem = ""
        n = 4
        for j in range(1, n + 1):
            px = tx + hw - 2 * hw * j / n
            mx = px + hw / n
            lift = h * 0.025 if j < n else 0
            hem += f"Q{mx:.1f},{ybot + h * 0.04:.1f} {px:.1f},{ybot - lift:.1f} "
        d = f"M{tx:.1f},{ytop:.1f} Q{tx + hw * 0.35:.1f},{ytop + (ybot - ytop) * 0.6:.1f} {tx + hw:.1f},{ybot:.1f} " + hem + f"Q{tx - hw * 0.35:.1f},{ytop + (ybot - ytop) * 0.6:.1f} {tx:.1f},{ytop:.1f} Z"
        s += f'<path d="{d}" fill="{body}" stroke="{edge}" stroke-width="{max(1.2, h * 0.018):.1f}" stroke-linejoin="round"/>'
        # Shadow right half, light left sliver.
        s += f'<path d="M{tx:.1f},{ytop:.1f} Q{tx + hw * 0.35:.1f},{ytop + (ybot - ytop) * 0.6:.1f} {tx + hw:.1f},{ybot:.1f} L{tx + hw * 0.15:.1f},{ybot + h * 0.02:.1f} Z" fill="{dark}" opacity="0.75"/>'
        s += f'<path d="M{tx:.1f},{ytop:.1f} Q{tx - hw * 0.3:.1f},{ytop + (ybot - ytop) * 0.55:.1f} {tx - hw * 0.8:.1f},{ybot - h * 0.01:.1f} L{tx - hw * 0.45:.1f},{ybot - h * 0.015:.1f} Z" fill="{lit}" opacity="0.8"/>'
        if snow > 0:
            sy = ytop + (ybot - ytop) * 0.42
            sw_ = hw * 0.62
            s += (f'<path d="M{tx:.1f},{ytop + 1:.1f} Q{tx + sw_ * 0.5:.1f},{sy - 2:.1f} {tx + sw_:.1f},{sy + 3:.1f} '
                  f'Q{tx + sw_ * 0.55:.1f},{sy + 1:.1f} {tx + sw_ * 0.2:.1f},{sy + 4:.1f} Q{tx - sw_ * 0.3:.1f},{sy - 1:.1f} {tx - sw_ * 1.05:.1f},{sy + 2:.1f} '
                  f'Q{tx - sw_ * 0.5:.1f},{sy - 4:.1f} {tx:.1f},{ytop + 1:.1f} Z" fill="{snow_c}" stroke="{snow_e}" stroke-width="{max(0.8, h * 0.01):.1f}" opacity="{snow:.2f}"/>')
    return s


def bare_tree(x, y, h, rnd, col, edge):
    """A winter aspen: pale trunk, a fan of thin branches."""
    s = f'<path d="M{x - h * 0.03:.1f},{y:.1f} L{x - h * 0.012:.1f},{y - h:.1f} L{x + h * 0.012:.1f},{y - h:.1f} L{x + h * 0.03:.1f},{y:.1f} Z" fill="{col}" stroke="{edge}" stroke-width="1.2"/>'
    for k in range(7):
        by = y - h * (0.35 + 0.09 * k)
        side = 1 if k % 2 else -1
        L = h * (0.32 - 0.03 * k) * rnd.uniform(0.7, 1.1)
        s += f'<path d="M{x:.1f},{by:.1f} Q{x + side * L * 0.5:.1f},{by - L * 0.2:.1f} {x + side * L:.1f},{by - L * 0.75:.1f}" fill="none" stroke="{edge}" stroke-width="{max(1, h * 0.014):.1f}" stroke-linecap="round"/>'
    return s


GREEN_NEAR = ("#2f7d58", "#56a97c", "#1c5640", "#174634", "#ffffff", "#9db8d6")
GREEN_MID = ("#3f7f6c", "#5f9d88", "#2d6457", "#285449", "#f4f8ff", "#a9bfd9")


# --------------------------------------------------------------- foothills
def forest_band(W, ridge_y, rnd, cover, depth_n, col_top, col_bot, tip, gid, density=5, tip_h=(9, 16)):
    """A forest cloak hanging from a ridge: a serrated crown of spruce tips
    with snow dust along the top, a dark mass falling to a ragged lower
    edge, and fainter rows of tips inside so it reads as many trees."""
    out = ""
    x = 0
    runs, cur = [], []
    for xi in range(0, W + 1, density):
        if cover(xi) > 0:
            cur.append(xi)
        elif cur:
            runs.append(cur); cur = []
    if cur:
        runs.append(cur)
    for run in runs:
        if len(run) < 3:
            continue
        top, bot = [], []
        tips = ""
        for k, xi in enumerate(run):
            ry = ridge_y(xi)
            dep = 30 + 70 * (0.5 + 0.5 * depth_n(xi)) + rnd.uniform(-6, 6)
            th = rnd.uniform(*tip_h)
            top.append((xi - density * 0.5, ry + 3))
            top.append((xi, ry + 3 - th))
            bot.append((xi, ry + dep))
            tips += f"M{xi - th * 0.12:.1f},{ry + 3 - th * 0.55:.1f} L{xi:.1f},{ry + 3 - th:.1f} L{xi + th * 0.1:.1f},{ry + 3 - th * 0.6:.1f} "
        top.append((run[-1] + density * 0.5, ridge_y(run[-1]) + 3))
        pts = top + list(reversed(bot))
        out += f'<path d="{poly(pts)}" fill="url(#{gid})"/>'
        # Inner rows of crowns.
        for row in (1, 2, 3):
            d = ""
            for xi in run[row % 2::2]:
                ry = ridge_y(xi)
                dep = 30 + 70 * (0.5 + 0.5 * depth_n(xi))
                yy = ry + dep * row / 4.2
                th = rnd.uniform(6, 11)
                d += f"M{xi - th * 0.3:.1f},{yy:.1f} L{xi:.1f},{yy - th:.1f} L{xi + th * 0.3:.1f},{yy:.1f} "
            out += f'<path d="{d}" fill="none" stroke="{tip}" stroke-width="1.6" opacity="{0.5 - row * 0.1:.2f}" stroke-linejoin="round"/>'
        out += f'<path d="{tips}" fill="#f4f8ff" opacity="0.9"/>'
    return out


def foothills(name, W, H, seed):
    """Forested foothills at mid distance: two rounded ridges, the back one
    bluer, each cloaked in spruce forest with snowy clearings between, lit
    on the left shoulders and shaded on the lee."""
    r = random.Random(seed)
    defs = lg("fb", [(0, "#f2f7fe"), (0.6, "#dde8f7"), (1, "#c8d7ee")])
    defs += lg("fb2", [(0, "#e6eefb"), (1, "#c9d7ef")])
    defs += lg("for1", [(0, "#58798f"), (0.5, "#6d8ca3"), (1, "#a9bfd8", 0.2)])
    defs += lg("for2", [(0, "#486a80"), (0.45, "#5a7d92"), (1, "#9fb6d2", 0.15)])
    defs += lg("fh", [(0, "#d6e5f6", 0), (1, "#d6e5f6", 0.85)])
    b = ""
    for li, (base_y, amp, gid, fill, edge, cov_bias) in enumerate(((0.36, 0.12, "for1", "url(#fb2)", "#a9bbdc", 0.3), (0.58, 0.14, "for2", "url(#fb)", "#93a9cf", 0.25))):
        prof = Periodic(W, r, [(3, 1.0), (7, 0.5), (15, 0.22), (31, 0.08)])
        cover = Periodic(W, r, [(9, 1.0), (21, 0.6), (47, 0.3)])
        dn = Periodic(W, r, [(17, 1.0), (43, 0.5)])
        step = 4
        xs = list(range(0, W + 1, step))
        ys = [H * base_y - prof(x) * H * amp for x in xs]
        def ry(x, ys=ys):
            x %= W
            i = int(x / step)
            k = (x - i * step) / step
            return ys[i] + (ys[min(i + 1, len(ys) - 1)] - ys[i]) * k
        pts = list(zip(xs, ys))
        d = smooth_path(pts, (W, H))
        cid = f"fc{li}"
        defs += f'<clipPath id="{cid}"><path d="{d}"/></clipPath>'
        b += f'<path d="{d}" fill="{fill}"/><g clip-path="url(#{cid})">'
        # Lee shade: where the ridge falls to the right, a soft blue wash.
        lee = ""
        i = 1
        while i < len(pts) - 1:
            if ys[i] <= ys[i - 1] and ys[i] < ys[i + 1]:
                j = i + 1
                while j < len(pts) - 1 and ys[j + 1] >= ys[j]:
                    j += 1
                if xs[j] - xs[i] > 24:
                    seg = [(xs[q], ys[q]) for q in range(i, j + 1)]
                    lee += f'<path d="{poly(seg + [(x - 30, y + 70) for x, y in reversed(seg)])}"/>'
                i = j
            i += 1
        b += f'<g fill="#9fb3da" opacity="0.5" filter="url(#soft{li})">{lee}</g>'
        defs += f'<filter id="soft{li}" x="-5%" y="-50%" width="110%" height="200%"><feGaussianBlur stdDeviation="9"/></filter>'
        # Forest: thousands of tiny crowns where a 2-D density is positive,
        # so patches taper and fray at their lower edges.
        dn2 = Periodic(W, r, [(23, 1.0), (57, 0.6)])
        cols = ("#46677d", "#52758b", "#5f8399") if li else ("#5d7d95", "#6a8aa2", "#7896ad")
        buckets = {c: "" for c in cols}
        tips = ""
        maxd = 120 if li else 90
        trees = []
        for x in range(0, W, 4 if li else 3):
            cv = cover(x) + cov_bias
            if cv < -0.6:
                continue
            for d0 in range(0, maxd, 5 if li else 4):
                dd = d0 + r.uniform(-2, 2)
                val = cv + 0.45 * dn2(x + dd * 3) / 1.6 - dd / maxd * 1.2 + r.uniform(-0.25, 0.25)
                if val <= 0:
                    continue
                xx = x + r.uniform(-1.8, 1.8)
                y = ry(xx) + dd + 4
                th = (r.uniform(9, 15) if li else r.uniform(6, 10)) * (1.0 - 0.25 * dd / maxd)
                trees.append((y, xx, th))
        trees.sort()
        for y, x, th in trees:
            c = cols[min(2, int(r.random() * 2 + (y - ry(x)) / maxd * 1.5))]
            for o in wrap_copies(x, W, 8):
                xx = x + o
                buckets[c] += f"M{xx:.1f},{y - th:.1f} L{xx + th * 0.3:.1f},{y:.1f} L{xx - th * 0.3:.1f},{y:.1f} Z "
                tips += f"M{xx:.1f},{y - th:.1f} L{xx + th * 0.12:.1f},{y - th * 0.6:.1f} L{xx - th * 0.14:.1f},{y - th * 0.55:.1f} Z "
        for c in cols:
            b += f'<path d="{buckets[c]}" fill="{c}"/>'
        b += f'<path d="{tips}" fill="#f2f7ff" opacity="0.85"/>'
        # Snowy clearings and a few lone trees in the open.
        for _ in range(60):
            x = r.uniform(0, W)
            if cover(x) + cov_bias > 0:
                continue
            y = ry(x) + r.uniform(20, 120)
            th = r.uniform(8, 14) if li else r.uniform(6, 10)
            for o in wrap_copies(x, W, 10):
                b += f'<path d="M{x + o:.1f},{y - th:.1f} L{x + o + th * 0.3:.1f},{y:.1f} L{x + o - th * 0.3:.1f},{y:.1f} Z" fill="{"#4b6d83" if li else "#6382a0"}"/>'
        b += f'<rect x="0" y="{H * (0.3 + li * 0.25):.0f}" width="{W}" height="{H:.0f}" fill="url(#fh)"/>'
        b += "</g>"
        lit_d = ""
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            if y1 < y0 - 0.15:
                lit_d += f"M{x0:.1f},{y0 + 2:.1f} L{x1:.1f},{y1 + 2:.1f} "
        b += f'<path d="{lit_d}" stroke="#ffffff" stroke-width="3" fill="none" opacity="0.9"/>'
        b += f'<path d="{smooth_path(pts)}" fill="none" stroke="{edge}" stroke-width="2.5"/>'
    write(name, W, H, b, defs)


# ------------------------------------------------------------- snow hills
def snow_hills(name, W, H, seed):
    """Near rolling hills: sculpted snow with wind-cut crests, lit left
    shoulders, deep blue lee shadows, and spruce groves in clusters of
    different sizes with a few bare aspens, so the tile never reads as a
    row of identical trees."""
    r = random.Random(seed)
    prof = Periodic(W, r, [(4, 1.0), (9, 0.45), (19, 0.2), (41, 0.07)])
    step = 8
    xs = list(range(0, W + 1, step))
    ys = [H * 0.55 - prof(x) * H * 0.2 for x in xs]
    pts = list(zip(xs, ys))
    d = smooth_path(pts, (W, H))
    def y_at(x):
        x %= W
        i = int(x / step)
        k = (x - i * step) / step
        return ys[i] + (ys[min(i + 1, len(ys) - 1)] - ys[i]) * k
    defs = lg("hb", [(0, "#ffffff"), (0.35, "#eef4fd"), (1, "#c8d8f0")])
    defs += f'<clipPath id="hc"><path d="{d}"/></clipPath>'
    defs += lg("lee", [(0, "#8fa9d8", 0.55), (1, "#8fa9d8", 0.1)])
    b = f'<path d="{d}" fill="url(#hb)"/><g clip-path="url(#hc)">'
    # Lee shadows: a soft blue wash right of every crest.
    i = 1
    while i < len(pts) - 1:
        if ys[i] < ys[i - 1] and ys[i] <= ys[i + 1]:
            # crest at i: shadow until the next trough
            j = i + 1
            while j < len(pts) - 1 and ys[j + 1] >= ys[j]:
                j += 1
            span = xs[j] - xs[i]
            if span > 30:
                sh = [(xs[i], ys[i])] + [(xs[q], ys[q]) for q in range(i, j + 1)]
                sh += [(xs[j] - span * 0.1, ys[j] + 40), (xs[i] + span * 0.25, ys[i] + 70)]
                b += f'<path d="{smooth_path(sh)} Z" fill="#a9bfe4" opacity="0.55"/>'
            i = j
        i += 1
    # Wind-sculpted drift lines following the contour, offset down.
    for off, c, sw, op in ((8, "#ffffff", 10, 0.9), (26, "#d4e2f5", 18, 0.55), (52, "#bccfec", 14, 0.4)):
        b += f'<path d="{smooth_path([(x, y + off) for x, y in pts])}" fill="none" stroke="{c}" stroke-width="{sw}" opacity="{op}"/>'
    for _ in range(40):
        x = r.uniform(0, W); y = y_at(x) + r.uniform(30, H * 0.5); rx = r.uniform(50, 150)
        for o in wrap_copies(x, W, rx):
            b += f'<path d="M{x + o - rx:.0f},{y:.0f} Q{x + o:.0f},{y - rx * 0.16:.0f} {x + o + rx:.0f},{y:.0f} Q{x + o:.0f},{y - rx * 0.05:.0f} {x + o - rx:.0f},{y:.0f} Z" fill="#b9cceb" opacity="0.6"/>'
            b += f'<path d="M{x + o - rx * 0.9:.0f},{y - 2:.0f} Q{x + o - rx * 0.1:.0f},{y - rx * 0.17:.0f} {x + o + rx * 0.6:.0f},{y - rx * 0.06:.0f}" fill="none" stroke="#ffffff" stroke-width="4" opacity="0.8" stroke-linecap="round"/>'
    b += "</g>"
    lit_d = ""
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        if y1 < y0 - 0.2:
            lit_d += f"M{x0:.1f},{y0 + 3:.1f} L{x1:.1f},{y1 + 3:.1f} "
    b += f'<path d="{lit_d}" stroke="#ffffff" stroke-width="4" fill="none"/>'
    b += f'<path d="{smooth_path(pts)}" fill="none" stroke="#8aa6cf" stroke-width="3" stroke-linejoin="round"/>'
    # Groves: clusters of 1..9 spruces with varied heights, gaps between.
    things = []
    x = r.uniform(0, 80)
    while x < W:
        kind = r.random()
        if kind < 0.72:
            n = r.choice((1, 2, 3, 3, 4, 5, 6, 8))
            hmax = r.uniform(80, 150)
            for k in range(n):
                tx = x + k * r.uniform(14, 30)
                th = hmax * (1 - 0.5 * abs(k - n / 2) / max(1, n / 2)) * r.uniform(0.75, 1.05)
                things.append(("s", tx, th))
            x += n * 24 + r.uniform(120, 420)
        elif kind < 0.9:
            for k in range(r.randint(2, 4)):
                things.append(("a", x + k * r.uniform(10, 22), r.uniform(60, 100)))
            x += r.uniform(160, 380)
        else:
            x += r.uniform(250, 500)
    things.sort(key=lambda t: -t[2] * 0 + r.random())
    trees = ""
    for kind, tx, th in things:
        y = y_at(tx) + r.uniform(4, 14)
        for o in wrap_copies(tx, W, 80):
            if kind == "s":
                trees += spruce(tx + o, y, th, random.Random(int(tx * 7)), GREEN_NEAR, lean=r.uniform(-0.3, 0.3))
            else:
                trees += bare_tree(tx + o, y, th, random.Random(int(tx * 5)), "#e9e2d6", "#8d7f74")
            # A blue snow shadow at the foot.
            trees += f'<ellipse cx="{tx + o + th * 0.12:.1f}" cy="{y + 2:.1f}" rx="{th * 0.3:.1f}" ry="{th * 0.05:.1f}" fill="#9fb6df" opacity="0.6"/>'
    b += trees
    write(name, W, H, b, defs)


# ---------------------------------------------------------------- treeline
def treeline(name="treeline", W=8192, H=340, seed=21):
    """The prairie edge behind the launch field: a dense, uneven spruce and
    aspen belt with clearings, a few farmhouses with lit windows, a red barn
    and a grain elevator, drawn near enough to carry full paint."""
    r = random.Random(seed)
    b = ""
    # Back row: a softer, bluer forest mass for depth.
    back = Periodic(W, r, [(13, 1.0), (37, 0.5), (90, 0.25)])
    pts = [(x, H - 95 - back(x) * 22) for x in range(0, W + 1, 8)]
    b += f'<path d="{smooth_path(pts, (W, H))}" fill="#7f9fbe" stroke="#6d8bb0" stroke-width="3"/>'
    for x in range(0, W, 11):
        y = H - 95 - back(x) * 22 + 6
        th = r.uniform(26, 44)
        b += f'<path d="M{x:.1f},{y - th:.1f} L{x + th * 0.3:.1f},{y + 4:.1f} L{x - th * 0.3:.1f},{y + 4:.1f} Z" fill="#7393b4"/>'
        b += f'<path d="M{x:.1f},{y - th:.1f} L{x + th * 0.15:.1f},{y - th * 0.55:.1f} L{x - th * 0.15:.1f},{y - th * 0.5:.1f} Z" fill="#eef4ff" opacity="0.9"/>'
    # Front belt.
    items = []
    x = 0.0
    landmarks = {int(W * 0.13): "farm", int(W * 0.38): "barn", int(W * 0.57): "elevator", int(W * 0.79): "farm2"}
    lm_x = sorted(landmarks)
    li = 0
    while x < W:
        if li < len(lm_x) and x > lm_x[li]:
            items.append((landmarks[lm_x[li]], x + 40))
            x += 300
            li += 1
            continue
        roll = r.random()
        if roll < 0.08:
            x += r.uniform(60, 160)  # clearing
            continue
        if roll < 0.2:
            items.append(("aspen", x))
            x += r.uniform(14, 26)
            continue
        items.append(("spruce", x))
        x += r.uniform(16, 34)
    trees = ""
    bld = ""
    for kind, x in items:
        y = H - r.uniform(0, 8)
        for o in wrap_copies(x, W, 200):
            xx = x + o
            if kind == "spruce":
                th = r.uniform(95, 190) if r.random() < 0.85 else r.uniform(190, 230)
                trees += spruce(xx, y, th, random.Random(int(x)), GREEN_NEAR, lean=r.uniform(-0.4, 0.4))
            elif kind == "aspen":
                trees += bare_tree(xx, y, r.uniform(90, 150), random.Random(int(x)), "#ece6da", "#8a7c70")
            elif kind in ("farm", "farm2"):
                bld += farmhouse(xx, H, "#b7543f" if kind == "farm" else "#4f78a8")
            elif kind == "barn":
                bld += barn(xx, H)
            elif kind == "elevator":
                bld += elevator(xx, H)
    b += bld + trees
    # Snow along the foot.
    foot = Periodic(W, r, [(40, 1.0), (97, 0.5)])
    fp = [(x, H - 12 - foot(x) * 5) for x in range(0, W + 1, 8)]
    b += f'<path d="{smooth_path(fp, (W, H))}" fill="#ffffff" stroke="#a9c0de" stroke-width="2.5"/>'
    write(name, W, H, b)


def _roof_snow(x0, x1, y0, apex_x, apex_y):
    return f'<path d="M{x0 - 8},{y0 + 2} L{apex_x},{apex_y - 6} L{x1 + 8},{y0 + 2} L{x1 - 6},{y0 + 8} L{apex_x},{apex_y + 6} L{x0 + 6},{y0 + 8} Z" fill="#ffffff" stroke="#9db8d6" stroke-width="2" stroke-linejoin="round"/>'


def farmhouse(x, H, wall):
    w, h = 150, 92
    x0, y0 = x - w / 2, H - h
    s = f'<rect x="{x0:.1f}" y="{y0:.1f}" width="{w}" height="{h}" fill="{wall}" stroke="{mix(wall, "#000000", 0.4)}" stroke-width="3"/>'
    s += f'<rect x="{x0:.1f}" y="{y0:.1f}" width="{w}" height="18" fill="#000000" opacity="0.15"/>'
    s += f'<rect x="{x0 + w * 0.72:.1f}" y="{y0 - 70:.1f}" width="18" height="50" fill="#8a6a58" stroke="#5e4436" stroke-width="2.5"/>'
    s += f'<path d="M{x0 - 12:.1f},{y0:.1f} L{x:.1f},{y0 - 64:.1f} L{x0 + w + 12:.1f},{y0:.1f} Z" fill="#6b4e5e" stroke="#4a3442" stroke-width="3" stroke-linejoin="round"/>'
    s += _roof_snow(x0 - 12, x0 + w + 12, y0, x, y0 - 64)
    for wx in (x0 + 20, x0 + w - 48):
        s += f'<rect x="{wx:.1f}" y="{y0 + 26:.1f}" width="28" height="30" rx="3" fill="#ffd57a" stroke="#8a5a2a" stroke-width="2.5"/>'
        s += f'<rect x="{wx - 6:.1f}" y="{y0 + 22:.1f}" width="40" height="38" rx="8" fill="#ffcf6a" opacity="0.25"/>'
        s += f'<path d="M{wx + 14:.1f},{y0 + 26:.1f} L{wx + 14:.1f},{y0 + 56:.1f} M{wx:.1f},{y0 + 41:.1f} L{wx + 28:.1f},{y0 + 41:.1f}" stroke="#8a5a2a" stroke-width="2"/>'
    s += f'<rect x="{x - 14:.1f}" y="{H - 46:.1f}" width="28" height="46" rx="3" fill="#6a3f1c" stroke="#40240e" stroke-width="2.5"/>'
    s += f'<circle cx="{x:.1f}" cy="{H - 52:.1f}" r="9" fill="none" stroke="#2c7d4e" stroke-width="5"/>'
    return s


def barn(x, H):
    w, h = 190, 120
    x0, y0 = x - w / 2, H - h
    red, edge = "#c0463a", "#7e2a22"
    s = f'<path d="M{x0:.1f},{H} L{x0:.1f},{y0:.1f} L{x0 + 30:.1f},{y0 - 48:.1f} L{x:.1f},{y0 - 70:.1f} L{x0 + w - 30:.1f},{y0 - 48:.1f} L{x0 + w:.1f},{y0:.1f} L{x0 + w:.1f},{H} Z" fill="{red}" stroke="{edge}" stroke-width="3.5" stroke-linejoin="round"/>'
    for k in range(1, 9):
        s += f'<path d="M{x0 + k * w / 9:.1f},{y0 - 40 + abs(k - 4.5) * 8:.1f} L{x0 + k * w / 9:.1f},{H}" stroke="{edge}" stroke-width="1.5" opacity="0.5"/>'
    s += f'<path d="M{x0 - 10:.1f},{y0 + 4:.1f} L{x0 + 30:.1f},{y0 - 48:.1f} L{x:.1f},{y0 - 70:.1f} L{x0 + w - 30:.1f},{y0 - 48:.1f} L{x0 + w + 10:.1f},{y0 + 4:.1f} L{x0 + w - 4:.1f},{y0 + 8:.1f} L{x0 + w - 32:.1f},{y0 - 36:.1f} L{x:.1f},{y0 - 56:.1f} L{x0 + 32:.1f},{y0 - 36:.1f} L{x0 + 4:.1f},{y0 + 8:.1f} Z" fill="#ffffff" stroke="#9db8d6" stroke-width="2" stroke-linejoin="round"/>'
    s += f'<rect x="{x - 36:.1f}" y="{H - 74:.1f}" width="72" height="74" fill="#a63a30" stroke="#f3ece0" stroke-width="5"/>'
    s += f'<path d="M{x - 36:.1f},{H - 74:.1f} L{x + 36:.1f},{H} M{x + 36:.1f},{H - 74:.1f} L{x - 36:.1f},{H}" stroke="#f3ece0" stroke-width="4"/>'
    s += f'<rect x="{x - 14:.1f}" y="{y0 - 34:.1f}" width="28" height="24" fill="#ffd57a" stroke="#f3ece0" stroke-width="3"/>'
    return s


def elevator(x, H):
    """A prairie grain elevator: tall timber house with a peaked cupola."""
    w, h = 90, 230
    x0, y0 = x - w / 2, H - h
    c, edge = "#c9563f", "#7e2f22"
    s = f'<rect x="{x0 - 50:.1f}" y="{H - 70:.1f}" width="60" height="70" fill="#b44e3a" stroke="{edge}" stroke-width="3"/>'
    s += f'<rect x="{x0:.1f}" y="{y0:.1f}" width="{w}" height="{h}" fill="{c}" stroke="{edge}" stroke-width="3.5"/>'
    s += f'<rect x="{x0 + w - 24:.1f}" y="{y0:.1f}" width="24" height="{h}" fill="#000000" opacity="0.18"/>'
    s += f'<rect x="{x0:.1f}" y="{y0:.1f}" width="14" height="{h}" fill="#ffffff" opacity="0.12"/>'
    s += f'<rect x="{x - 22:.1f}" y="{y0 - 46:.1f}" width="44" height="46" fill="{c}" stroke="{edge}" stroke-width="3"/>'
    s += f'<path d="M{x - 30:.1f},{y0 - 44:.1f} L{x:.1f},{y0 - 74:.1f} L{x + 30:.1f},{y0 - 44:.1f} Z" fill="#ffffff" stroke="#9db8d6" stroke-width="2.5" stroke-linejoin="round"/>'
    s += f'<path d="M{x0 - 6:.1f},{y0 + 2:.1f} L{x0 + w + 6:.1f},{y0 + 2:.1f} L{x0 + w:.1f},{y0 + 9:.1f} L{x0:.1f},{y0 + 9:.1f} Z" fill="#ffffff" stroke="#9db8d6" stroke-width="2"/>'
    s += f'<rect x="{x0 + 8:.1f}" y="{y0 + 30:.1f}" width="{w - 16}" height="30" fill="#f3ece0" stroke="{edge}" stroke-width="2"/>'
    s += f'<text x="{x:.1f}" y="{y0 + 53:.1f}" font-family="sans-serif" font-weight="bold" font-size="20" text-anchor="middle" fill="#7e2f22">ALTA</text>'
    return s


def meadow(name="meadow", W=512, H=256, seed=33):
    """Open snow between the treeline and the street: soft wind ripples,
    blue hollows, and lit drift lips (seamless both ways)."""
    r = random.Random(seed)
    body = f'<rect x="0" y="0" width="{W}" height="{H}" fill="url(#mg)"/>'
    for i in range(12):
        x = r.uniform(0, W); y = r.uniform(0, H); rx = r.uniform(60, 170); ry = rx * r.uniform(0.14, 0.22)
        for dx in (0, W, -W):
            for dy in (0, H, -H):
                cx, cy = x + dx, y + dy
                body += f'<path d="M{cx - rx:.0f},{cy:.0f} Q{cx:.0f},{cy - ry * 2:.0f} {cx + rx:.0f},{cy:.0f} Z" fill="#cbd9f1"/>'
                body += f'<path d="M{cx - rx * 0.9:.0f},{cy - 2:.0f} Q{cx - rx * 0.2:.0f},{cy - ry * 2:.0f} {cx + rx * 0.5:.0f},{cy - ry * 0.9:.0f}" fill="none" stroke="#ffffff" stroke-width="5" stroke-linecap="round"/>'
    defs = lg("mg", [(0, "#f2f6fd"), (1, "#e6eefa")])
    write(name, W, H, body, defs)


# ------------------------------------------------------------------ ground
def drift(x, y, rx, h, lee=0.35):
    """One wind-sculpted drift seen from the side: a long rounded windward
    slope, a short steep lee face in blue shadow, a bright lip on the crest."""
    cx = x + rx * (1 - 2 * lee)  # crest sits toward the lee (right) end
    e = x + rx
    wind = f"M{x - rx:.1f},{y:.1f} C{x - rx * 0.5:.1f},{y - h * 0.35:.1f} {cx - (cx - x + rx) * 0.45:.1f},{y - h * 1.02:.1f} {cx:.1f},{y - h:.1f}"
    lee_c = f"C{cx + (e - cx) * 0.3:.1f},{y - h * 0.98:.1f} {e - (e - cx) * 0.25:.1f},{y - h * 0.15:.1f} {e:.1f},{y:.1f}"
    s = f'<ellipse cx="{x + rx * 0.2:.1f}" cy="{y + h * 0.12:.1f}" rx="{rx * 1.05:.1f}" ry="{h * 0.3:.1f}" fill="#8ea3d6" opacity="0.4" filter="url(#gsoft)"/>'
    s += f'<path d="{wind} {lee_c} Z" fill="url(#drift)"/>'
    s += f'<path d="M{cx:.1f},{y - h:.1f} {lee_c} L{cx + (e - cx) * 0.1:.1f},{y:.1f} Q{cx - (e - cx) * 0.15:.1f},{y - h * 0.55:.1f} {cx:.1f},{y - h:.1f} Z" fill="#a9bde6" opacity="0.85"/>'
    s += f'<path d="M{x - rx * 0.55:.1f},{y - h * 0.3:.1f} C{x - rx * 0.1:.1f},{y - h * 0.7:.1f} {cx - (cx - x + rx) * 0.4:.1f},{y - h * 1.0:.1f} {cx:.1f},{y - h + 1:.1f}" fill="none" stroke="#ffffff" stroke-width="{max(2, h * 0.14):.1f}" stroke-linecap="round"/>'
    return s


def ground():
    """Ground tile (seamless, 512x256, the snow surface at row 44): a
    wind-sculpted crust with a soft blue undercut, then the snowfield in
    front falling away in drifts with lit lips, blue lee shadows and glints
    of sparkle."""
    W, H = 512, 256
    r = random.Random(7)
    surf = Periodic(W, r, [(2, 1.0), (5, 0.4), (9, 0.15)])
    under = Periodic(W, r, [(3, 1.0), (7, 0.4)])
    defs = lg("deep", [(0, "#edf3fc"), (0.35, "#d3dff4"), (1, "#a9bae4")])
    defs += lg("crust", [(0, "#ffffff"), (0.6, "#ffffff"), (1, "#e7eefa")])
    defs += lg("drift", [(0, "#ffffff"), (0.7, "#f1f6fd"), (1, "#d6e2f5")])
    defs += '<filter id="gsoft" x="-20%" y="-60%" width="140%" height="220%"><feGaussianBlur stdDeviation="4"/></filter>'
    b = f'<rect x="0" y="40" width="{W}" height="{H - 40}" fill="url(#deep)"/>'
    # Drifts in rows, smaller and bluer toward the top (further away).
    for row, (y0, n, rxr, hr) in enumerate(((104, 4, (40, 70), (12, 18)), (156, 3, (60, 95), (17, 26)), (220, 3, (80, 120), (24, 34)))):
        xs = sorted(r.uniform(0, W) for _ in range(n))
        for x in xs:
            rx = r.uniform(*rxr); h = r.uniform(*hr); y = y0 + r.uniform(-8, 8)
            for o in wrap_copies(x, W, rx * 1.3):
                b += drift(x + o, y, rx, h, r.uniform(0.25, 0.4))
    # The crust: a blurred blue undercut, then the white blanket with its lip.
    pts = [(x, 44 - surf(x) * 3.5) for x in range(0, W + 1, 4)]
    low = [(x, 76 + under(x) * 3) for x in range(0, W + 1, 4)]
    b += f'<path d="{poly([(x, y + 5) for x, y in low] + [(W, 70), (0, 70)])}" fill="#93abd8" opacity="0.7" filter="url(#gsoft)"/>'
    b += f'<path d="{smooth_path(pts)} L{W},{low[-1][1]:.1f} ' + " ".join(f"L{x:.1f},{y:.1f}" for x, y in reversed(low)) + ' Z" fill="url(#crust)"/>'
    b += f'<path d="{smooth_path(low)}" fill="none" stroke="#c2d2ee" stroke-width="3"/>'
    b += f'<path d="{smooth_path([(x, y + 3) for x, y in pts])}" fill="none" stroke="#ffffff" stroke-width="4"/>'
    # Wind ripples on the crust face.
    for k in range(8):
        x = r.uniform(0, W); L = r.uniform(30, 70); y = 56 + r.uniform(0, 12)
        for o in wrap_copies(x, W, L):
            b += f'<path d="M{x + o:.0f},{y:.0f} q{L / 2:.0f},-4 {L:.0f},0" fill="none" stroke="#dbe5f5" stroke-width="2.5" stroke-linecap="round"/>'
    # Sparkle: tiny four-point glints and dots on the lit snow.
    for k in range(30):
        x = r.uniform(0, W)
        y = r.choice((r.uniform(42, 70), r.uniform(84, 250)))
        s_ = r.uniform(2.5, 5) if y < 80 else r.uniform(1.5, 3.5)
        for o in wrap_copies(x, W, 10):
            xx = x + o
            if r.random() < 0.45:
                b += f'<path d="M{xx:.1f},{y - s_ * 2:.1f} L{xx + s_ * 0.3:.1f},{y:.1f} L{xx:.1f},{y + s_ * 2:.1f} L{xx - s_ * 0.3:.1f},{y:.1f} Z M{xx - s_ * 2:.1f},{y:.1f} L{xx:.1f},{y - s_ * 0.3:.1f} L{xx + s_ * 2:.1f},{y:.1f} L{xx:.1f},{y + s_ * 0.3:.1f} Z" fill="#ffffff"/>'
            else:
                b += f'<circle cx="{xx:.1f}" cy="{y:.1f}" r="{s_ * 0.55:.1f}" fill="#ffffff" opacity="0.85"/>'
    write("ground", W, H, b, defs)


# ------------------------------------------------------------------- build
FAR = dict(lit="#d3defa", body="#b2c3ea", foot="#c4d4f0", shadow="#93a6d8", reflect="#c9d5f2", crest="#e9effc",
           snow="#f6f9ff", snow_lo="#dfe8f8", snow_edge="#c3d1ee", haze="#dce9f9", edge="#a4b6e0", edge_w=2)
NEAR = dict(lit="#b4c4ec", body="#8a9fd4", foot="#9fb3df", shadow="#5f72b2", reflect="#97a8dc", crest="#d6e0f8",
            snow="#ffffff", snow_lo="#e3ecf9", snow_edge="#9fb4dd", haze="#d2e2f6", edge="#6e83bf", edge_w=2.6,
            trees="#5c7aa3")


def build_all():
    mountains("mountains_far", 4096, 512, 31, 11, FAR, height=0.82, jag=0.8, detail=0.7, base=0.16)
    mountains("mountains_near", 4096, 512, 17, 8, NEAR, height=0.9, jag=1.2, detail=1.0, base=0.08)
    foothills("hills_far", 4096, 384, 5)
    snow_hills("hills_near", 4096, 320, 9)
    ground()


if __name__ == "__main__":
    build_all()
    treeline()
    meadow()
    print("terrain written")
