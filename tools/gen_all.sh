#!/usr/bin/env bash
# Regenerate and rasterize every sprite of a pack:  PACK=calgary_xmas tools/gen_all.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export PACK="${PACK:-calgary_xmas}"
python3 tools/gen_svg.py        # treeline, meadow, base props, effects
python3 tools/gen_chars.py      # pilots, Santa, elves
python3 tools/gen_props.py      # pickups and props
python3 tools/gen_city.py       # Calgary streets: facades, landmarks, skyline, Calgarians; writes city.json
python3 tools/gen_paint.py      # hand-painted key-art pieces: Falkbuilt building, sled, sky, mountains, hills, ground
python3 tools/gen_ui.py         # UI kit: logo, paper panels, ribbons, seals, medals, dial, icons
python3 tools/gen_styles.py     # gallery: dog style tests
python3 tools/gen_candidates.py # gallery: character candidates
python3 tools/paint.py         # repaint every svg in the style of the painted key art
node tools/render_sprites.js
python3 tools/build_gallery.py
