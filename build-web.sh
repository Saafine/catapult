#!/usr/bin/env bash
# Build the browser version and drop the wasm next to index.html.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown -p hamsters
cp target/wasm32-unknown-unknown/release/hamsters.wasm web/hamsters.wasm
python3 tools/build_gallery.py
# The active pack travels with the page: sprites and strings are fetched at startup.
pack=$(python3 -c "import json;print(json.load(open('packs/active.json'))['pack'])")
mkdir -p web/packs/"$pack"
cp packs/active.json web/packs/active.json
cp packs/"$pack"/pack.json web/packs/"$pack"/pack.json
rsync -a --delete packs/"$pack"/sprites/ web/packs/"$pack"/sprites/
if [ -d packs/"$pack"/sounds ]; then rsync -a --delete packs/"$pack"/sounds/ web/packs/"$pack"/sounds/; fi
ls -la web/hamsters.wasm
echo "serve with:  python3 tools/serve.py   (no-cache server on :8080)"
