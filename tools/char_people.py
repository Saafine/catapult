"""Santa, the elves and the reindeer for the 2026-10 character pass, painted
like the pilots (brush.Art.paint: lit gradient, deep same-hue outline, core
shadow, bounce light, fur tufts)."""
import math
from brush import Art, ellipse, circle, tone, mix, deep, shape, xform, fur_marks, bent_limb, shift_path

SKIN = "#ffcfae"
RED = "#e0372e"
FUR = "#ffffff"


def toon_eye(art, x, y, r, look=(1, 0), happy=0.3, skin=SKIN):
    """Glossy dark oval eye with two highlights and a happy lower lid."""
    d = ellipse(x, y, r * 0.78, r)
    art.paint(d, "#2a1712", w=max(2.0, r * 0.28), ink="#1a0c08", k=r * 0.5, hi=0, core=0, bounce=0, drop=0,
              lights=[(ellipse(x + look[0] * r * 0.2, y + r * 0.55, r * 0.5, r * 0.28), "#8a5a3a", 0.8, r * 0.15)])
    art.fill(ellipse(x - r * 0.22 + look[0], y - r * 0.38, r * 0.3, r * 0.26), "#ffffff")
    art.fill(circle(x + r * 0.26 + look[0], y + r * 0.3, r * 0.12), "#ffffff", 0.9)
    if happy:
        by = y + r * (1 - happy * 1.4)
        art.fill(f"M{x - r * 1.1:.1f},{y + r * 1.3:.1f} L{x - r * 1.1:.1f},{by + 2:.1f} Q{x:.1f},{by - r * 0.4:.1f} {x + r * 1.1:.1f},{by + 2:.1f} "
                 f"L{x + r * 1.1:.1f},{y + r * 1.3:.1f} Z", skin)


def blush(art, x, y, r):
    art.blend(ellipse(x, y, r, r * 0.62), "#ff6b6b", 0.55, max(1.5, r * 0.35), "normal")


# ---------------------------------------------------------------- Santa
def santa():
    """Santa waving from the start gate: a round jolly figure, beard in
    curly tufts, hat flopping back with the pompom, gold buckle. 192x256."""
    art = Art()
    BOOT = "#2c2430"
    for x in (72, 122):
        d = shape([(x - 18, 236), (x - 14, 222), (x + 8, 220), (x + 26, 230), (x + 28, 244), (x - 18, 246)])
        art.paint(d, BOOT, w=3.4, ink="#120c14", k=6, hi=0.4, core=0.4, bounce=0.6, bounce_col="#6a7aa8")
    # Left arm resting on the belly, behind the coat edge.
    arm = shape(bent_limb([(46, 132), (24, 170), (50, 196)], [32, 28, 26]))
    art.paint(arm, RED, w=3.8, k=10)
    coat = shape([(62, 116), (96, 106), (132, 110), (158, 128), (174, 170), (180, 214), (168, 232), (96, 238), (28, 232),
                  (16, 212), (22, 168), (38, 132)])
    art.paint(coat, RED, w=4.2, k=18, hi=0.55, core=0.5, bounce=0.5,
              fur=[(fur_marks(130, 170, 30, 40, 10, ang=100, ln=14, seed=3), tone(RED, -0.15), 0.35, 2.4)])
    art.paint(shape([(90, 140), (104, 140), (106, 224), (88, 224)], {1: (4, 4, 0.2), 3: (4, 4, 0.2)}, seed=2), FUR, w=3.0, k=6, hi=0, core=0.45)
    hem = shape([(14, 212), (96, 224), (182, 212), (184, 232), (96, 248), (12, 234)], {0: (5, 6, 0.2), 1: (5, 6, 0.2), 3: (5, 6, 0.2), 4: (5, 6, 0.2)}, seed=4)
    art.paint(hem, FUR, w=3.6, k=8, hi=0.4, core=0.5)
    belt = shape([(18, 178), (96, 190), (176, 178), (178, 198), (96, 210), (16, 198)])
    art.paint(belt, "#2a2028", w=3.2, ink="#100a10", k=6, hi=0.3, core=0.3, bounce=0.5, bounce_col="#6a7aa8")
    art.paint(shape([(82, 182), (110, 182), (112, 208), (80, 208)]), "#ffcf3a", w=3.2, k=6, hi=0.7, core=0.4, bounce=0,
              lights=[(ellipse(88, 188, 6, 3), "#ffffff", 0.9, 1)])
    art.fill(shape([(89, 189), (103, 189), (104, 201), (88, 201)]), "#2a2028")
    # Left mitten on the belly.
    art.paint(shape([(40, 186), (56, 180), (68, 188), (66, 204), (52, 210), (40, 202)]), "#2f8a52", w=3.4, k=6)
    # Right arm waving, cuff and mitten.
    rarm = shape(bent_limb([(150, 132), (172, 104), (166, 70)], [34, 30, 26]))
    art.paint(rarm, RED, w=3.8, k=10)
    art.paint(shape([(150, 74), (182, 70), (184, 84), (150, 88)], {0: (3, 4, 0.2), 2: (3, 4, 0.2)}, seed=7), FUR, w=3.0, k=5, core=0.4)
    mit = shape([(152, 72), (148, 50), (158, 36), (170, 40), (174, 30), (184, 34), (182, 52), (180, 72)])
    art.paint(mit, "#2f8a52", w=3.4, k=7)
    # Head.
    art.paint(circle(96, 82, 34), SKIN, w=3.6, ink=deep("#e89a7a"), k=10, hi=0.5, core=0.35)
    blush(art, 76, 92, 9); blush(art, 116, 92, 9)
    toon_eye(art, 82, 80, 7.5, happy=0.4); toon_eye(art, 110, 80, 7.5, happy=0.4)
    beard = shape([(56, 86), (72, 104), (96, 112), (120, 104), (136, 86), (148, 112), (142, 148), (120, 172), (96, 182),
                   (72, 172), (50, 148), (44, 112)], {5: (2, 8, 0.3), 6: (3, 9, 0.3), 7: (2, 9, 0.3), 8: (2, 9, 0.3), 9: (3, 9, 0.3),
                                                        10: (2, 8, 0.3)}, seed=9)
    art.paint(beard, FUR, w=3.6, k=12, hi=0.4, core=0.55,
              fur=[(fur_marks(96, 142, 36, 26, 16, ang=95, ln=14, seed=5, curl=0.6), "#aebcdc", 0.8, 2.4)])
    # Smile peeking out of the beard.
    art.paint("M84,118 Q96,132 110,118 Q96,124 84,118 Z", "#8a2024", w=2.6, ink="#4a1014", k=3, hi=0, core=0, bounce=0, drop=0)
    for side in (-1, 1):
        mo = shape([(96, 104), (96 + 14 * side, 98), (96 + 28 * side, 104), (96 + 34 * side, 96), (96 + 32 * side, 112),
                    (96 + 16 * side, 116)][::(1 if side > 0 else -1)])
        art.paint(mo, FUR, w=3.2, k=6, hi=0.4, core=0.45)
    art.paint(circle(96, 98, 9), "#ff8a80", w=3.0, ink="#b84a48", k=4, hi=0.6, core=0.3, bounce=0,
              lights=[(circle(93, 95, 3), "#ffffff", 0.9, 1)])
    for side in (-1, 1):
        art.paint(shape(xform([(-12, 4), (-4, -6), (10, -6), (14, 2), (2, 2)], 0, 0, side, 1, 0, 96 + 15 * side, 66)), FUR, w=2.6, k=4, core=0.4)
    # Hat flopping back to the left with the pompom.
    cone = shape([(26, 58), (38, 30), (70, 12), (106, 10), (132, 24), (142, 60), (54, 60), (40, 54)])
    cone = shift_path(cone, 0, -12)
    art.paint(cone, RED, w=4.0, k=12, hi=0.55, core=0.5)
    brim = shape([(50, 62), (96, 50), (142, 62), (144, 78), (96, 68), (48, 78)], {0: (3, 5, 0.2), 1: (3, 5, 0.2), 3: (3, 5, 0.2), 4: (3, 5, 0.2)}, seed=3)
    art.paint(shift_path(brim, 0, -12), FUR, w=3.6, k=7, core=0.5)
    pom = shape([(24 + 13 * math.cos(a), 62 + 13 * math.sin(a)) for a in [i * math.pi / 5 for i in range(10)]],
                {i: (1, 4, 0.2) for i in range(10)}, seed=6)
    art.paint(shift_path(pom, 0, -12), FUR, w=3.4, k=6, core=0.5)
    return art


# ---------------------------------------------------------------- elves
def elf(tunic, stripe, cowboy=False):
    """A chibi elf on 256x352 (saved at half size): big head with pointy
    ears and freckles, zigzag tunic, striped stockings, curly shoes, one arm
    waving. The cowboy one is a Calgary elf in a white Stampede hat."""
    art = Art()
    SHOE = "#2f8a52" if not cowboy else "#7a4a2a"
    for x in (110, 146):
        leg = f"M{x - 12},236 L{x - 12},300 C{x - 12},310 {x + 12},310 {x + 12},300 L{x + 12},236 Z"
        art.paint(leg, "#ffffff", w=3.8, k=6, core=0.4, hi=0.3)
        lid = art.clip_id(leg)
        for y in range(244, 300, 16):
            art.add(f'<path d="M{x - 14},{y} L{x + 14},{y - 5} L{x + 14},{y + 4} L{x - 14},{y + 9} Z" fill="{stripe}" clip-path="url(#{lid})"/>')
        art.overlay(leg, k=6, hi=0.3, core=0.4, bounce=0.4)
        art.stroke(leg, "#5a6a8a", 3.8)
    for side in (-1, 1):
        x0 = 128 + 4 * side
        pts = [(x0, 298), (x0 + 30 * side, 296), (x0 + 58 * side, 286), (x0 + 72 * side, 270), (x0 + 66 * side, 292),
               (x0 + 50 * side, 316), (x0, 320)]
        art.paint(shape(pts if side > 0 else pts[::-1]), SHOE, w=3.8, k=8)
        art.paint(circle(x0 + 72 * side, 266, 7), "#ffd23a", w=3.0, k=3, core=0.3)
    # Arms: left on the hip, right waving.
    art.paint(shape(bent_limb([(92, 164), (70, 196), (80, 222)], [26, 24, 22])), tunic, w=3.8, k=8)
    art.paint(circle(82, 226, 13), SKIN, w=3.6, ink=deep("#e89a7a"), k=5)
    art.paint(shape(bent_limb([(164, 160), (194, 140), (206, 104)], [26, 24, 22])), tunic, w=3.8, k=8)
    hand = shape([(196, 106), (194, 88), (200, 78), (208, 84), (214, 76), (222, 84), (218, 104), (208, 112)])
    art.paint(hand, SKIN, w=3.6, ink=deep("#e89a7a"), k=5)
    tun = "M90,148 C96,134 160,134 166,148 L184,236 L170,226 L156,242 L142,226 L128,242 L114,226 L100,242 L86,226 L72,236 Z"
    art.paint(tun, tunic, w=4.0, k=14, hi=0.55, core=0.5)
    art.paint(shape([(88, 188), (128, 196), (170, 188), (172, 206), (128, 214), (86, 206)]), "#3a2a22", w=3.2, ink="#1a100c", k=5, hi=0.3, core=0.3)
    art.paint(shape([(116, 186), (140, 186), (140, 212), (116, 212)]), "#ffcf3a", w=3.0, k=4, hi=0.7, core=0.3, bounce=0)
    coll = shape([(92, 144), (110, 156), (128, 150), (146, 156), (164, 144), (150, 134), (106, 134)], {0: (2, 4, 0.2), 1: (2, 4, 0.2), 2: (2, 4, 0.2), 3: (2, 4, 0.2)})
    art.paint(coll, stripe, w=3.4, k=6)
    # Head.
    ear_col = SKIN
    for side in (-1, 1):
        pts = [(128 + 46 * side, 108), (128 + 76 * side, 64), (128 + 82 * side, 58), (128 + 66 * side, 92), (128 + 50 * side, 92)]
        art.paint(shape(pts if side > 0 else pts[::-1]), ear_col, w=3.6, ink=deep("#e89a7a"), k=6)
    art.paint(circle(128, 104, 54), SKIN, w=4.0, ink=deep("#e89a7a"), k=14, hi=0.5, core=0.4)
    hair = shape([(76, 92), (90, 62), (128, 52), (166, 62), (180, 92), (166, 80), (152, 88), (140, 76), (124, 86), (112, 76),
                  (98, 88)], seed=4)
    art.paint(hair, "#d9692a" if not cowboy else "#8a4a24", w=3.4, k=8)
    toon_eye(art, 108, 110, 11, happy=0.25); toon_eye(art, 148, 110, 11, happy=0.25)
    for side in (-1, 1):
        art.stroke(f"M{128 + 30 * side},{90} q{-8 * side},-6 {-18 * side},-2", "#6a3a1e", 3.6)
    blush(art, 92, 128, 11); blush(art, 164, 128, 11)
    for (x, y) in ((88, 124), (96, 130), (160, 124), (168, 130)):
        art.fill(circle(x, y, 1.8), "#c8704a", 0.8)
    art.paint("M110,130 Q128,152 146,130 Q128,138 110,130 Z", "#8a2024", w=3.0, ink="#4a1014", k=3, hi=0, core=0, bounce=0, drop=0)
    art.fill(ellipse(128, 141, 8, 3.5), "#ff7d84")
    art.paint(ellipse(128, 120, 7, 5.5), "#ff9f86", w=2.6, ink=deep("#e89a7a"), k=3, hi=0.6, core=0.2, bounce=0, drop=0)
    if cowboy:
        brim = shape([(46, 74), (80, 70), (128, 72), (176, 70), (210, 74), (196, 84), (160, 90), (96, 90), (60, 84)])
        art.paint(brim, "#f6f7fb", w=4.0, k=8, core=0.5)
        crown = shape([(88, 74), (86, 40), (102, 18), (128, 26), (154, 18), (170, 40), (168, 74)])
        art.paint(crown, "#ffffff", w=4.0, k=12, core=0.5)
        art.paint(shape([(88, 62), (168, 62), (168, 74), (88, 74)]), "#c8372e", w=2.8, k=4, hi=0.4, core=0.3, bounce=0)
    else:
        cone = shape([(76, 74), (92, 36), (140, 8), (190, 12), (200, 30), (186, 48), (180, 62)])
        art.paint(cone, tunic, w=4.0, k=12, hi=0.55, core=0.5)
        art.paint(circle(196, 48, 11), "#ffd23a", w=3.4, k=4, core=0.3, lights=[(circle(192, 44, 4), "#ffffff", 0.9, 1.5)])
        band = shape([(72, 70), (128, 50), (184, 62), (186, 80), (128, 68), (74, 88)])
        art.paint(band, stripe, w=3.6, k=6)
    return art


# ---------------------------------------------------------------- reindeer
def reindeer():
    """The reindeer pickup, leaping left: warm brown coat, cream belly and
    muzzle, glowing red nose, big glossy eye, branching antlers, a red
    collar with a bell. 224x176."""
    art = Art()
    COAT, LEG, CREAM, ANT = "#b8743f", "#8e5530", "#f4dfbf", "#e2b878"
    # Far legs (behind the body).
    for pts in ([(150, 122), (176, 140), (204, 150)], [(78, 126), (56, 146), (30, 150)]):
        art.paint(shape(bent_limb(pts, [22, 16, 13])), tone(LEG, -0.06), w=3.4, k=6)
    # Antlers.
    for (x, y, s) in ((82, 48, 1.0), (104, 44, 0.85)):
        main = [(x, y), (x - 6 * s, y - 20 * s), (x - 4 * s, y - 40 * s)]
        art.paint(shape(bent_limb(main, [10, 8, 6])), ANT, w=3.0, ink=deep(ANT), k=4)
        for (a, b, c, d) in ((-4, -14, -18, -22), (-6, -28, 8, -38), (-5, -36, -16, -46)):
            art.paint(shape(bent_limb([(x + a * s, y + b * s), (x + c * s, y + d * s)], [7, 5])), ANT, w=2.8, ink=deep(ANT), k=3)
    tail = shape([(178, 92), (196, 80), (204, 90), (190, 104)], {0: (2, 4, 0.3), 1: (2, 4, 0.3)})
    art.paint(tail, CREAM, w=3.2, k=5)
    body = shape([(70, 96), (112, 80), (160, 82), (186, 98), (184, 126), (160, 142), (112, 146), (80, 136)], {1: (2, 4, 0.2), 2: (2, 4, 0.2)}, seed=3)
    art.paint(body, COAT, w=4.0, k=14, hi=0.55, core=0.45,
              fur=[(fur_marks(130, 104, 40, 14, 14, ang=180, ln=12, seed=2), tone(COAT, -0.2), 0.4, 2.2)])
    bid = art.clip_id(body)
    art.add(f'<path d="{shape([(80, 130), (130, 124), (176, 120), (190, 160), (70, 160)])}" fill="{CREAM}" clip-path="url(#{bid})"/>')
    art.stroke(body, deep(COAT), 4.0)
    # Near legs, stretched in the leap.
    for pts in ([(160, 126), (186, 146), (214, 152)], [(92, 128), (66, 150), (40, 158)]):
        art.paint(shape(bent_limb(pts, [24, 17, 14])), LEG, w=3.4, k=6)
        hx, hy = pts[-1]
        art.paint(ellipse(hx + (4 if hx > 100 else -4), hy, 8, 6), "#3a2620", w=2.6, k=3, core=0.3)
    # Head and neck.
    neck = shape([(70, 108), (66, 86), (84, 70), (100, 84), (96, 104)])
    art.paint(neck, COAT, w=3.8, k=10)
    for side in (0,):
        art.paint(shape([(96, 60), (112, 46), (120, 56), (104, 70)]), COAT, w=3.4, k=5)
    head = shape([(54, 66), (74, 50), (96, 54), (104, 72), (96, 92), (74, 100), (44, 104), (30, 96), (32, 82)])
    art.paint(head, COAT, w=4.0, k=12, hi=0.55, core=0.4)
    hid = art.clip_id(head)
    art.add(f'<path d="{ellipse(40, 94, 24, 16)}" fill="{CREAM}" clip-path="url(#{hid})"/>')
    art.overlay(head, k=10, hi=0.3, core=0.3)
    art.stroke(head, deep(COAT), 4.0)
    art.glow(circle(28, 92, 20), "#ff5a4a", 0.55, 6)
    art.paint(circle(28, 92, 11), "#ff3b3b", w=3.2, ink="#9a1010", k=5, hi=0.6, core=0.4, bounce=0,
              lights=[(circle(24, 88, 4), "#ffffff", 0.95, 1.2)])
    toon_eye(art, 72, 72, 8, look=(-1, 0), happy=0.25, skin=COAT)
    art.stroke("M62,58 q8,-6 18,-2", "#5a3018", 3.4)
    blush(art, 56, 86, 7)
    art.stroke("M38,104 Q48,110 58,104", "#4a2014", 3.2)
    art.paint(shape([(96, 92), (112, 104), (126, 110), (124, 120), (108, 116), (92, 104)]), "#e2453a", w=3.0, k=4)
    art.paint(circle(118, 124, 7), "#ffd23a", w=2.8, k=3, core=0.3, lights=[(circle(116, 121, 2.5), "#ffffff", 0.9, 1)])
    return art


# ---------------------------------------------------------------- sled
def sled():
    """The red sled from the key art, 192x96: a lacquered red deck with a
    gold pinstripe and a nose that curls up at the front (right), steel
    runners curling over the front with a brass bell, a rope handle. The
    deck top (y 46-68) is where the dog lies, as before."""
    art = Art()
    RED = "#e3392f"
    STEEL = "#7f93a8"
    # Struts.
    for x in (44, 92, 140):
        art.paint(shape(bent_limb([(x, 62), (x + 4, 82)], [9, 8])), STEEL, w=2.6, ink="#26313d", k=3, core=0.4, hi=0.5)
    # Runner with the front curl.
    run = [(8, 82), (6, 90), (20, 92), (160, 92), (182, 88), (190, 72), (184, 58), (172, 56), (168, 64), (176, 66), (180, 74),
           (174, 82), (160, 84), (22, 84)]
    art.paint(shape(run), STEEL, w=3.0, ink="#26313d", k=4, hi=0.7, core=0.45, bounce=0.6,
              lights=[("M24,88 L158,88", "#ffffff", 0.0, 1)])
    art.stroke("M24,87.5 L160,87.5", "#e8f2ff", 1.8, 0.9)
    # Deck: lacquered slats with a raised nose.
    deck = shape([(12, 46), (90, 44), (150, 44), (172, 40), (186, 34), (190, 44), (182, 58), (160, 68), (90, 69), (14, 69), (6, 58)])
    art.paint(deck, RED, w=3.4, k=8, hi=0.65, core=0.5, bounce=0.5,
              lights=[("M18,50 C80,47 140,47 172,42", "#ffffff", 0.0, 1)])
    did = art.clip_id(deck)
    for x in (54, 102, 148):
        art.add(f'<path d="M{x},44 L{x - 2},70" stroke="{deep(RED)}" stroke-width="2.4" opacity="0.7" clip-path="url(#{did})"/>')
    art.add(f'<path d="M14,62 L150,62 C166,62 178,52 184,42" fill="none" stroke="#ffd25a" stroke-width="2.6" clip-path="url(#{did})"/>')
    art.add(f'<path d="M16,50 C80,47 140,47 172,41" fill="none" stroke="#ffb0a0" stroke-width="3" opacity="0.85" clip-path="url(#{did})"/>')
    # Brass bell on the curl and the rope handle at the back.
    art.paint(circle(184, 62, 6), "#ffcf3a", w=2.4, k=3, core=0.4, lights=[(circle(182, 60, 2), "#ffffff", 0.9, 0.8)])
    art.stroke("M10,52 C-2,42 2,28 16,30 C24,31 24,40 18,46", "#8a6a3a", 5)
    art.stroke("M10,52 C-2,42 2,28 16,30 C24,31 24,40 18,46", "#d9b276", 3)
    return art
