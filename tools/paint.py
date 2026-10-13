#!/usr/bin/env python3
"""Paint pass: restyles every generated SVG of a pack after the designers'
"painted cartoon" key art (the Falkbuilt rooftop): no single dark ink line,
each shape is outlined in a deeper shade of its own colour, fills are lit
from above (light top, colour, cool shade at the bottom), snow is white
falling to ice blue with a blue-grey edge, flat black shade patches become
cool blue shadows, and a light paint grain and mottle sit over everything.

Runs after the generators and before rasterizing:  python3 tools/paint.py
Sprites drawn in the new style by hand carry data-paint="done" on the root
and only get the grain; data-paint="skip" leaves a file untouched."""
import colorsys, os, re, sys
import xml.etree.ElementTree as ET

PACK = os.environ.get("PACK", "calgary_xmas")
SVG_DIR = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
NS = "http://www.w3.org/2000/svg"
ET.register_namespace("", NS)
ET.register_namespace("xlink", "http://www.w3.org/1999/xlink")
Q = lambda t: f"{{{NS}}}{t}"

# The old line colours: dark browns and navies the generators used as ink.
INKS = {"#3a2618", "#4a2e1e", "#2b3a5a", "#1f2340", "#3a2a1e", "#2a2234", "#4a3a30"}
# Particles, glows and overlays the game tints itself: leave them alone.
SKIP = {"snowflake", "puff", "sparkle", "ring", "confetti", "vignette", "star", "updraft", "coldwind"}
# Gallery-only experiments are not game art.
SKIP_PREFIX = ("style_", "cand_")
SHAPES = {"path", "rect", "circle", "ellipse", "polygon", "polyline", "line"}
LINE_INK = "#4b2c22"   # warm dark brown for lone detail lines (faces, mullions)
SNOW_EDGE = "#8fb0d0"

def rgb(h):
    h = h.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))

def hexc(r, g, b):
    return "#%02x%02x%02x" % tuple(max(0, min(255, round(v * 255))) for v in (r, g, b))

def hsv(h):
    return colorsys.rgb_to_hsv(*rgb(h))

def from_hsv(h, s, v):
    return hexc(*colorsys.hsv_to_rgb(h % 1.0, max(0, min(1, s)), max(0, min(1, v))))

def lum(h):
    r, g, b = rgb(h)
    return 0.299 * r + 0.587 * g + 0.114 * b

def mix(a, b, t):
    ra, rb = rgb(a), rgb(b)
    return hexc(*(x + (y - x) * t for x, y in zip(ra, rb)))

def hue_toward(h, target, k):
    d = ((target - h + 0.5) % 1.0) - 0.5
    return h + d * k

def soften(c):
    """Pull the old poster primaries towards the key art's warmer paint."""
    h, s, v = hsv(c)
    if s > 0.7:
        s = 0.7 + (s - 0.7) * 0.6
    return from_hsv(h, s, min(1.0, v * 1.02))

def edge(c):
    """Outline: the shape's own colour, deeper, warmer and more saturated."""
    if lum(c) > 0.86 and hsv(c)[1] < 0.18:
        return SNOW_EDGE
    h, s, v = hsv(c)
    if s < 0.12:  # greys and whites keep a cool edge
        return from_hsv(0.6, 0.22, v * 0.5)
    return from_hsv(hue_toward(h, 0.0, 0.08), s * 1.1 + 0.12, v * 0.52)

def light(c):
    if lum(c) > 0.86 and hsv(c)[1] < 0.18:
        return "#ffffff"
    h, s, v = hsv(c)
    return from_hsv(hue_toward(h, 0.13, 0.08), s * 0.8, v + (1 - v) * 0.45 + 0.04)

def shadow(c):
    if lum(c) > 0.86 and hsv(c)[1] < 0.18:
        return "#cddff0"
    h, s, v = hsv(c)
    # Warm paints (orange to yellow-green) shade towards red-orange, the rest
    # towards blue, so yellows never turn green in shadow.
    target = 0.03 if 0.04 < h < 0.22 else (0.97 if (h <= 0.04 or h > 0.88) else 0.62)
    return from_hsv(hue_toward(h, target, 0.12), min(1, s * 1.08 + 0.04), v * 0.8)

def is_hex(c):
    return bool(c) and re.fullmatch(r"#[0-9a-fA-F]{3}([0-9a-fA-F]{3})?", c) is not None

def norm(c):
    c = c.lower()
    return "#" + "".join(ch * 2 for ch in c[1:]) if len(c) == 4 else c

class Painter:
    def __init__(self, root):
        self.root = root
        self.defs = root.find(Q("defs"))
        if self.defs is None:
            self.defs = ET.Element(Q("defs"))
            root.insert(0, self.defs)
        self.grads = {}
        self.solids = []   # (element, parent, colour) that get painted volume
        self.blurs = {}
        self.base_of = {}  # url id -> representative colour
        for el in self.defs.iter():
            tag = el.tag.split("}")[-1]
            if tag in ("linearGradient", "radialGradient"):
                stops = [s.get("stop-color") for s in el if s.get("stop-color")]
                if stops:
                    self.base_of[el.get("id")] = stops[len(stops) // 2]
            elif tag == "pattern":
                for ch in el.iter():
                    f = ch.get("fill")
                    if ch is not el and is_hex(f):
                        self.base_of[el.get("id")] = f
                        break

    def grad(self, c):
        c = norm(c)
        if c not in self.grads:
            gid = f"pt{len(self.grads)}"
            g = ET.SubElement(self.defs, Q("linearGradient"), {"id": gid, "x1": "0", "y1": "0", "x2": "0", "y2": "1"})
            for off, col in ((0, light(c)), (0.42, c), (1, shadow(c))):
                ET.SubElement(g, Q("stop"), {"offset": str(off), "stop-color": col})
            self.grads[c] = gid
        return f"url(#{self.grads[c]})"

    def paint(self, parent):
        kids = list(parent)
        self.parent = parent
        for i, el in enumerate(kids):
            tag = el.tag.split("}")[-1]
            if tag in ("defs", "text", "style", "filter", "pattern", "clipPath", "mask"):
                continue
            if tag == "g":
                self.paint(el)
                continue
            if tag not in SHAPES:
                continue
            self.parent = parent
            self.shape(el, kids[i + 1] if i + 1 < len(kids) else None)

    def shape(self, el, nxt):
        fill = el.get("fill")
        stroke = el.get("stroke")
        op = float(el.get("opacity", "1") or 1)
        ink = is_hex(stroke) and norm(stroke) in INKS
        # Flat black shade patches become cool blue shadow.
        if is_hex(fill) and norm(fill) == "#000000":
            el.set("fill", "#1c2b5e")
            return
        base = None
        if is_hex(fill):
            if norm(fill) in INKS:
                el.set("fill", "#3b2219")
            else:
                base = soften(norm(fill))
                if op >= 0.99 and el.get("fill-opacity") is None:
                    el.set("fill", self.grad(base))
                    self.solids.append((el, self.parent, base))
                else:
                    el.set("fill", base)
        elif fill and fill.startswith("url(#"):
            base = self.base_of.get(fill[5:-1])
            base = norm(base) if base and is_hex(base) else None
            if base and op >= 0.99:
                self.solids.append((el, self.parent, base))
        if not ink:
            if is_hex(stroke) and (fill in (None, "none")):
                el.set("stroke", soften(norm(stroke)))
            return
        w = float(el.get("stroke-width", "1") or 1)
        if el.get("data-ink") == "keep":
            # Eyes and teeth keep a dark rim even though they are white.
            el.set("stroke", LINE_INK)
            el.set("stroke-width", f"{max(1.4, w * 0.8):.2f}")
            return
        if base:
            el.set("stroke", edge(base))
            el.set("stroke-width", f"{max(1.4, w * 0.62):.2f}")
            return
        # A fill-less ink line: either the dark under-stroke of an outlined
        # line (the next sibling repeats the geometry in colour) or a lone
        # detail line.
        if nxt is not None and fill in (None, "none") and self.same_geom(el, nxt):
            inner = nxt.get("stroke")
            if is_hex(inner) and norm(inner) not in INKS:
                iw = float(nxt.get("stroke-width", "1") or 1)
                el.set("stroke", edge(soften(norm(inner))))
                el.set("stroke-width", f"{iw + max(1.4, (w - iw) * 0.6):.2f}")
                return
        el.set("stroke", LINE_INK)
        el.set("stroke-width", f"{max(1.2, w * 0.75):.2f}")


    GEOM = ("d", "x", "y", "width", "height", "cx", "cy", "r", "rx", "ry", "points")

    @staticmethod
    def size_of(el):
        """Smaller side of the shape's box, or None when it can't be told."""
        tag = el.tag.split("}")[-1]
        try:
            if tag == "rect":
                return min(float(el.get("width")), float(el.get("height")))
            if tag == "circle":
                return 2 * float(el.get("r"))
            if tag == "ellipse":
                return 2 * min(float(el.get("rx")), float(el.get("ry")))
            if tag in ("path", "polygon"):
                d = el.get("d") or el.get("points") or ""
                if re.search(r"[a-z]", d.replace("e-", "")):
                    return None  # relative commands: skip
                nums = [float(v) for v in re.findall(r"-?\d+(?:\.\d+)?", d)]
                if len(nums) < 4:
                    return None
                xs, ys = nums[0::2], nums[1::2]
                return min(max(xs) - min(xs), max(ys) - min(ys))
        except (TypeError, ValueError):
            return None
        return None

    def blur(self, sd):
        sd = round(sd * 2) / 2
        if sd not in self.blurs:
            fid = f"pv{len(self.blurs)}"
            f = ET.SubElement(self.defs, Q("filter"), {"id": fid, "x": "-30%", "y": "-30%", "width": "160%", "height": "160%"})
            ET.SubElement(f, Q("feGaussianBlur"), {"stdDeviation": str(sd)})
            self.blurs[sd] = fid
        return self.blurs[sd]

    def volume(self):
        """Paint each solid shape round: a soft shadow along its lower-right
        inner edge and soft light along its upper-left, clipped inside it,
        the way the key art models every brick, sill and parcel."""
        for i, (el, parent, base) in enumerate(self.solids):
            size = self.size_of(el)
            if not size or size < 9:
                continue
            tag = el.tag.split("}")[-1]
            k = max(1.5, min(16.0, size * 0.13))
            geom = {a: el.get(a) for a in self.GEOM if el.get(a) is not None}
            tr = el.get("transform", "")
            cid = f"pvc{i}"
            cp = ET.SubElement(self.defs, Q("clipPath"), {"id": cid})
            ET.SubElement(cp, Q(tag), dict(geom, **({"transform": tr} if tr else {})))
            g = ET.Element(Q("g"), {"clip-path": f"url(#{cid})"})
            o = k * 0.55
            for col, dx, op in ((shadow(shadow(base)), -o, "0.55"), (light(base), o, "0.6")):
                ET.SubElement(g, Q(tag), dict(geom, **{"fill": "none", "stroke": col, "stroke-width": f"{k * 1.6:.1f}", "opacity": op,
                    "filter": f"url(#{self.blur(k * 0.45)})", "transform": f"translate({dx:.1f} {dx:.1f}) {tr}".strip()}))
            idx = list(parent).index(el)
            parent.insert(idx + 1, g)
            # Keep the outline crisp on top of the soft paint.
            if el.get("stroke") not in (None, "none"):
                line = ET.Element(Q(tag), dict(geom, **{"fill": "none", "stroke": el.get("stroke"), "stroke-width": el.get("stroke-width", "1"),
                    "stroke-linejoin": "round", "stroke-linecap": "round"}, **({"transform": tr} if tr else {})))
                parent.insert(idx + 2, line)

    @staticmethod
    def same_geom(a, b):
        keys = ("d", "x", "y", "width", "height", "cx", "cy", "r", "rx", "ry", "points", "x1", "y1", "x2", "y2", "transform")
        return a.tag == b.tag and all(a.get(k) == b.get(k) for k in keys)

def grain(root, w, h):
    """Wrap the art in a light paint grain plus a soft large-scale mottle."""
    defs = root.find(Q("defs"))
    fine = 0.75 if max(w, h) < 700 else 0.55
    f = ET.SubElement(defs, Q("filter"), {"id": "paintgrain", "x": "-5%", "y": "-5%", "width": "110%", "height": "110%", "color-interpolation-filters": "sRGB"})
    ET.SubElement(f, Q("feTurbulence"), {"type": "fractalNoise", "baseFrequency": f"{fine}", "numOctaves": "2", "seed": "7", "result": "fine"})
    ET.SubElement(f, Q("feTurbulence"), {"type": "fractalNoise", "baseFrequency": f"{12 / max(w, h) * 3:.4f}", "numOctaves": "3", "seed": "3", "result": "soft"})
    # Grey noise at low contrast around mid grey, alpha forced to 1.
    ET.SubElement(f, Q("feColorMatrix"), {"in": "fine", "type": "matrix", "result": "fg",
        "values": "0.22 0 0 0 0.39  0.22 0 0 0 0.39  0.22 0 0 0 0.39  0 0 0 0 1"})
    ET.SubElement(f, Q("feColorMatrix"), {"in": "soft", "type": "matrix", "result": "sg",
        "values": "0.5 0 0 0 0.25  0.5 0 0 0 0.25  0.5 0 0 0 0.25  0 0 0 0 1"})
    ET.SubElement(f, Q("feComposite"), {"in": "fg", "in2": "sg", "operator": "arithmetic", "k1": "0", "k2": "0.5", "k3": "0.5", "k4": "0", "result": "noise"})
    ET.SubElement(f, Q("feComposite"), {"in": "noise", "in2": "SourceAlpha", "operator": "in", "result": "masked"})
    ET.SubElement(f, Q("feBlend"), {"in": "masked", "in2": "SourceGraphic", "mode": "soft-light", "result": "lit"})
    ET.SubElement(f, Q("feComposite"), {"in": "lit", "in2": "SourceAlpha", "operator": "in"})
    g = ET.Element(Q("g"), {"filter": "url(#paintgrain)"})
    for ch in list(root):
        if ch.tag != Q("defs"):
            root.remove(ch)
            g.append(ch)
    root.append(g)

def process(path):
    name = os.path.basename(path)[:-4]
    if name in SKIP or name.startswith(SKIP_PREFIX):
        return False
    src = open(path).read()
    if "data-painted" in src:
        return False  # already processed in this run
    # Browsers forgive a bare & in sign text; XML does not.
    root = ET.fromstring(re.sub(r"&(?!#?\w+;)", "&amp;", src))
    mode = root.get("data-paint")
    if mode == "skip":
        return False
    w = float(root.get("width")); h = float(root.get("height"))
    p = Painter(root)
    if mode != "done":
        p.paint(root)
        p.volume()
    if p.defs is None or root.find(Q("defs")) is None:
        root.insert(0, ET.Element(Q("defs")))
    if not os.environ.get("NOGRAIN"): grain(root, w, h)
    root.set("data-painted", "1")
    open(path, "w").write(ET.tostring(root, encoding="unicode"))
    return True

if __name__ == "__main__":
    only = re.compile(sys.argv[1]) if len(sys.argv) > 1 else None
    n = 0
    for f in sorted(os.listdir(SVG_DIR)):
        if f.endswith(".svg") and (not only or only.search(f)):
            n += process(os.path.join(SVG_DIR, f))
    print("painted", n, "svgs")
