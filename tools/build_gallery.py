#!/usr/bin/env python3
"""Builds web/gallery/: copies every rendered sprite and writes a manifest,
so http://localhost:8080/gallery/ shows all current visual objects and the
dog style candidates."""
import json, os, shutil
root = os.path.join(os.path.dirname(__file__), "..")
pack = json.load(open(os.path.join(root, "packs", "active.json")))["pack"]
src = os.path.join(root, "packs", pack, "sprites")
dst = os.path.join(root, "web", "gallery", "sprites")
os.makedirs(dst, exist_ok=True)
names = sorted(f for f in os.listdir(src) if f.endswith(".png"))
for f in names:
    shutil.copy2(os.path.join(src, f), os.path.join(dst, f))
groups = {}
for f in names:
    key = ("dog candidates" if f.startswith("cand_dog") else
           "elf candidates" if f.startswith("cand_elf") else
           "santa candidates" if f.startswith("cand_santa") else
           "dog styles" if f.startswith("style_") else
           "dogs" if f.startswith("dog_") else
           "people" if f.split("_")[0] in ("santa", "elf") else
           "pickups & hazards" if f.split(".")[0] in ("present_red", "present_green", "present_blue", "trampoline", "reindeer", "wings", "firework", "sleigh", "ufo", "stormcloud") else
           "sky" if f.split(".")[0] in ("moon", "cloud_a", "cloud_b", "cloud_c", "snowflake", "star") or f.startswith("cloud") else
           "landscape" if any(f.startswith(p) for p in ("mountains", "hills", "treeline", "meadow", "ground", "village", "snowfield")) else
           "props" if f.split(".")[0] in ("tower", "sled", "xmas_tree", "pine_snow", "snowman", "lamppost", "candycane", "fence", "signpost", "gift_pile", "tuft") else
           "particles & fx")
    groups.setdefault(key, []).append(f)
order = ["dog candidates", "elf candidates", "santa candidates", "dog styles", "dogs", "people", "pickups & hazards", "props", "sky", "landscape", "particles & fx"]
manifest = [{"group": g, "files": groups[g]} for g in order if g in groups]
for g in groups:
    if g not in order:
        manifest.append({"group": g, "files": groups[g]})
json.dump(manifest, open(os.path.join(root, "web", "gallery", "manifest.json"), "w"), indent=1)
print(f"gallery: {len(names)} sprites in {len(manifest)} groups")
