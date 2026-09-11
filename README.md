# Santa's Flight School

A from-scratch, Christmas-themed take on the 2007 Flash launch game
*Flight of the Hamsters*, written in Rust. Pick a dog, ride a sled down the
elves' ski-jump ramp, fly as far as you can, wiggle for lift, and bounce off
whatever Santa's helpers left in the way. The crate names still say
"hamsters" because that is where the project started.

## Layout

```
crates/core   hamsters-core  pure rules and physics, no platform code, has tests
crates/game   hamsters       macroquad runtime: window, input, drawing, particles
crates/game/assets/sprites   rendered PNG sprites (embedded into the binary)
crates/game/assets/*.ttf     Fredoka and Lilita One, both under the OFL
assets/svg    generated SVG sources for every sprite
tools/        gen_svg.py writes the SVGs; render_sprites.js rasterizes them
web/          index.html + macroquad's JS glue; the wasm is copied here on build
```

## Art pipeline

Every sprite is authored as SVG by `tools/gen_svg.py` (gradients, outlines,
seamless parallax tiles), then rasterized at 2x with transparent background by
`tools/render_sprites.js`, which needs Playwright and a Chrome:

```sh
python3 tools/gen_svg.py
NODE_PATH=/path/to/node_modules node tools/render_sprites.js
```

The PNGs are committed, so this only runs when the art changes. Adding a
sprite means: a `write(...)` call in `gen_svg.py`, a field in
`crates/game/src/sprites.rs`, and a `blit` call wherever it is drawn.

The split matters: `hamsters-core` compiles anywhere and knows nothing about
the browser. If a future feature needs something the browser can't do, build
the native binary from the same code. If you outgrow macroquad, the core moves
to another engine untouched.

## Browser build (works on this machine today)

```sh
./build-web.sh
python3 -m http.server 8080 -d web
# open http://localhost:8080
```

`web/mq_js_bundle.js` is miniquad's `js/gl.js`, copied from the crate that
Cargo.lock pins. If you bump macroquad, copy the new one from
`~/.cargo/registry/src/*/miniquad-<version>/js/gl.js`, otherwise the wasm
may import GL functions the old glue doesn't define.

## Native build (needs system packages once)

```sh
sudo apt install build-essential libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
cargo run --release -p hamsters
```

## Tests

```sh
cargo test -p hamsters-core
```

## Controls

| key             | action                                                   |
|-----------------|----------------------------------------------------------|
| 1 / 2 / 3       | choose a dog (before launch), or click a card            |
| space / click   | lock the push, then lock the jump angle at the lip       |
| space / click   | in the air: wiggle the paws for lift (costs stamina)     |
| R               | new run (keeps best distance)                            |

Wiggling refills slowly in the air and jumps up when you grab a present. A
wiggle never pushes you above a modest climb rate, so mashing keeps you
airborne rather than launching you into orbit.

## Easter eggs

- Every 25 wiggles in one run refills the stamina bar with a "WIGGLE POWER"
  burst, so a determined wiggler can keep climbing.
- Santa's sleigh cruises the upper sky and gives a huge boost.
- Above 400 m the sky turns to space; a UFO up there tows you even higher.
  Altitude milestones pop as banners and the results card records the best.

## Screenshots without a GPU

`cargo build ... --features devkeys` adds a `P` key that does a perfect
launch, `C` for a dive that crashes, `F` for a fireworks display and `T` for
slow motion. A Playwright script that drives the page through a flight lives in
the session scratchpad; the feature is never part of the normal build.

## Where to add things

- New pickup or obstacle: add a variant to `EntityKind` in `crates/core/src/lib.rs`,
  give it a radius, an effect in `apply_hit`, and a spawn weight in `spawn_ahead`.
  Then add a match arm in `entity()` in `crates/game/src/draw.rs`.
- New dog: add an entry to `PILOTS`, a colour set in `tools/gen_svg.py`, and a
  paw tint in `draw.rs`.
- The ramp shape is `ramp_point` in the core; the tower and ramp visuals are
  `start_area` in `draw.rs`.
- Sounds and particles: react to `Event`s in `App::on_event` in `main.rs`.
- Sprites: replace the body of a `draw::*` function with `draw_texture_ex`.
- Saves, leaderboards: the runtime crate is the place; the core stays pure.

## Toolchain notes for this machine

There is no gcc here, so `~/.cargo/config.toml` points the host linker at a
Zig shim in `~/.local/bin/zigcc`. That is enough for build scripts, tests and
the wasm target. Native windowed builds still need the apt line above.


## Asset packs

All art and theme strings live in `packs/<name>/` (see `packs/README.md`).
`packs/active.json` picks the pack the game loads at startup; `HAMSTERS_PACK=<name>`
overrides it for a native run. The first pack, `calgary_xmas`, is Christmas over
Calgary: the launch tower is the Calgary Tower with its cauldron lit, downtown
(The Bow, Bankers Hall, the Saddledome, the Peace Bridge) sits in the treeline
layer, the Rockies are the far layer, the crew wear white Stampede hats, and the
distance signposts name Cochrane, Airdrie, Okotoks, Canmore, Banff and friends.

Regenerate a pack's art with `PACK=<name> tools/gen_all.sh` (terrain from
`gen_svg.py`, characters from `gen_chars.py`, pickups and props from
`gen_props.py`, then rasterized by `render_sprites.js`); `./build-web.sh` copies
the active pack next to the page.

## Gallery

`http://localhost:8080/gallery/` lists every sprite in the active pack, plus the
character candidates (`tools/gen_candidates.py`) and dog style tests
(`tools/gen_styles.py`) at the top for picking.


## How a run works

1. **Push**: a swinging power gauge; press to lock it. The gold end is the sweet spot.
2. **Ride**: the sled drops down the ski-jump inrun on the Falkbuilt roof. Press once on the
   way down, as late as you can before the lip: the last stretch of the ramp glows.
   A press right at the lip is a PERFECT JUMP with a forward boost; early presses
   earn less; no press means the sled just drops off the lip at reduced speed.
3. **Flap**: in the air each press flaps the ears for lift and drains the FLAP bar
   (12 presses, slow regen, presents refill it). Every 25 flaps refills it fully.
4. **Land**: bounces below the crash speed roll out; hitting the snow faster than
   `CRASH_SPEED` ends the run head-first in a hole (SPLAT!). Pickups have a hitbox
   `PICKUP_REACH` times their drawn size; storm clouds keep their real size.
5. **Results**: the card shows this run's distance and a scoreboard of the ten
   longest flights (saved in the browser's local storage via `web/quad_storage.js`).
   Only the RESTART button, R or Enter start a new run; stray clicks do nothing.

Boosters: presents (small hop, stamina), reindeer (kick), rockets, Chinook
updrafts (a column of warm wind), trampolines, Santa's sleigh and the UFO.
Hazards: storm clouds (zap) and, past 120 m, Arctic cold winds that freeze the
dog into an ice block for `FREEZE_TIME` seconds (no flapping, heavy drag).
