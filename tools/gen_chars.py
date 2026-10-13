#!/usr/bin/env python3
"""Game characters, hand painted after the designers' key art: the pilots
(Buddy the orange-and-white beagle, Nanook the husky, Pixie the terrier)
with seven faces, seven body poses and a flapping ear each, portraits,
Santa and two elves. Heads are in three-quarter view facing right (the way
the dog flies): a big round skull, a long white muzzle with the nose at the
tip, two big eyes high on the face. Every part is painted with tools/brush.py
(lit fill, soft shadow and light inside, dark warm outline).
Run after gen_svg.py; writes into the pack svg folder."""
import math, os
from brush import Art

PACK = os.environ.get("PACK", "calgary_xmas")
OUT = os.path.join(os.path.dirname(__file__), "..", "packs", PACK, "svg")
os.makedirs(OUT, exist_ok=True)

def save(name, art, w, h, wrap=None):
    open(os.path.join(OUT, name + ".svg"), "w").write(art.svg(w, h, wrap))

import char_dog as dogs

def save_skip(name, art, w, h, wrap=None):
    """Character sprites carry their own painted finish (fur marks, lighting);
    paint.py's mottle would dirty them, so they opt out of it."""
    open(os.path.join(OUT, name + ".svg"), "w").write(art.svg(w, h, wrap).replace('data-paint="done"', 'data-paint="skip"'))

def svg_inner(name):
    s = open(os.path.join(OUT, name + ".svg")).read()
    defs = s.split("<defs>", 1)[1].split("</defs>", 1)[0]
    body = s.split("</defs>\n", 1)[1].rsplit("\n</svg>", 1)[0]
    return defs, body

def prefix_ids(defs, body, p):
    import re
    ids = re.findall(r'id="([^"]+)"', defs)
    for i in sorted(set(ids), key=len, reverse=True):
        defs = re.sub(rf'id="{i}"', f'id="{p}{i}"', defs)
        defs = defs.replace(f"url(#{i})", f"url(#{p}{i})")
        body = body.replace(f"url(#{i})", f"url(#{p}{i})")
    return defs, body

for n, c in dogs.BREEDS.items():
    save_skip(f"dog_{n}_ear", dogs.ear_svg(c), 160, 72)
    for f in dogs.FACES:
        save_skip(f"dog_{n}_head_{f}", dogs.head_svg(c, f), 256, 256, dogs.HEAD_WRAP)
    for p in dogs.POSES:
        save_skip(f"dog_{n}_body_{p}", dogs.body_svg(c, p), 256, 256)
    parts = [prefix_ids(*svg_inner(nm), f"p{k}_") for k, nm in enumerate((f"dog_{n}_head_idle", f"dog_{n}_ear", f"dog_{n}_body_seated"))]
    open(os.path.join(OUT, f"dog_{n}.svg"), "w").write(dogs.portrait_svg(n, c, parts[0], parts[1], parts[2]))

# ---------------------------------------------------------------- Santa, elves
import char_people as people
save_skip("santa", people.santa(), 192, 256)
save_skip("elf_a", people.elf("#3fa66b", "#e63c3c"), 128, 176, wrap="scale(0.5)")
save_skip("elf_b", people.elf("#3a86d6", "#e63c3c", cowboy=True), 128, 176, wrap="translate(128 0) scale(-0.5 0.5)")

# Gallery still shows the chosen dog under its old style slot.
open(os.path.join(OUT, "style_1_chibi.svg"), "w").write(open(os.path.join(OUT, "dog_biscuit.svg")).read())
print("chars written")
