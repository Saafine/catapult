#!/usr/bin/env bash
# Regenerate and rasterize every sprite of a pack:  PACK=calgary_xmas tools/gen_all.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export PACK="${PACK:-calgary_xmas}"
python3 tools/gen_svg.py        # terrain, pickups (base), effects
python3 tools/gen_chars.py      # pilots, Santa, elves (flat cartoon style)
python3 tools/gen_props.py      # pickups, props, sky, sled, tower (flat outlined style)
python3 tools/gen_styles.py     # gallery: dog style tests
python3 tools/gen_candidates.py # gallery: character candidates
node tools/render_sprites.js
python3 tools/build_gallery.py
