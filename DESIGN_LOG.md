# Design log

Rounds of a designer-panel loop. Three consulting personas review the build,
each proposes three improvements and a UI style; the improvements are
implemented and the UI is reworked to the synthesis of the three styles.

## Round 1 (2026-09-09)

### Jaakko Iisalo (trajectory, launch feel)
1. **Live aim arc.** `predict_arc(pilot, power, quality)` in the core; a dotted
   arc off the lip swings with the power needle and freezes on lock. The dial
   shows the predicted landing (`~N m`).
2. **The honest lip.** The sled coasts along the deck for `LIP_COAST` after the
   lip; jump quality falls from 1 to 0 across it, so a press can now be TOO LATE.
   Time runs at half speed in the last 3% of the ramp until the press.
3. **Ghost of your best.** The best run's path is saved and drawn as dots, ending
   in a checkered flag with the pilot's head. The best readout pulses gold as you
   close on the flag.

### Zach Gage (the loop, information design)
1. **History on the ground.** A pennant at every scoreboard distance; a live
   "N m to #k" line and a "#k" rank pulse in the HUD; the 25-tick wiggle row is
   replaced by "N to WIGGLE POWER" only when it is close.
2. **Report card.** The results card grades PUSH, JUMP (perfect / early by s /
   late by s / missed) and AIR, and gives one coach tip for the weakest skill
   (`tip_push`, `tip_jump`, `tip_air` in the pack).
3. **The next rung.** Before launch the target is the scoreboard entry just above
   the last run, not the all-time best; picker cards show each pilot's record.

### Nifflas (bounce, atmosphere, discovery)
1. **The stomp.** A press while falling near the snow arms a tuck: the landing
   bounces harder, keeps its pace and survives a harder impact.
2. **Frozen lakes and drifts.** Past 150 m the ground has hashed 45 m segments
   of ice (rolls for ever) and drifts (soft, forgiving).
3. **The jet stream.** A tailwind river between 110 and 170 m with visible
   streamers; storms never spawn above 260 m, so the top of the sky is calm.

### UI: "Paper Christmas Card"
All three asked for the same thing: opaque cream paper plates with a thick ink
outline and a hard offset shadow, like the FLIGHT SCHOOL sign; Lilita One for
numbers and shouts, Fredoka for labels; berry ribbons, pine buttons, gold only
for the sweet spot and records, ice for height. See `crates/game/src/hud.rs`.

## Round 2 (2026-09-11)

All three designers asked to keep the paper look and fix its hierarchy: opaque
plates in flight, solid slate for secondary text, no gold text on paper, fewer
and smaller elements over the sky.

### Jaakko Iisalo
1. **Wide arc, tight impact.** The flight camera zooms out to 0.62 so the
   parabola reads as a shape, then punches in over the last 1.1 s before the
   snow so the stomp window is framed.
2. **The look-back.** Before each launch the camera sits on the target flag and
   glides back along the pennants to the tower; the needle starting is the cue.
3. **Savour the splat** (merged with Nifflas's "quiet after landing", below).

### Zach Gage
1. **Teach one rule when needed.** PRESS AT THE LIP, PRESS TO FLAP and PRESS TO
   STOMP appear in the world the first time each matters, then never again.
2. **Today's course.** One seeded run per dog per day, so pilot choice is a
   decision; after the three the seeds go random and nothing nags.
3. **The last eight flights.** A trend strip on the results card so a run that
   misses the top ten still has a home.

### Nifflas
1. **First sounds.** A procedural palette with no assets: flap, bounce, present
   chime rising with the combo, milestone arpeggio, launch whoosh, landing, and
   a wind bed that follows airspeed.
2. **The long afternoon.** The sun sinks and warms with distance; the horizon
   goes peach then violet, stars come out, and the aurora becomes dusk's reward.
3. **The quiet after landing.** A 1.6 s hold with the dog alone in the snow,
   breath in the cold air, the rank stamp settling on the pennant, before the
   postcard arrives.

## Player feedback pass (2026-09-11)

Direct requests from playing the round-2 build, not a designer round:

- **Sounds were bad.** The audio is now synthesized in `crates/synth` with
  real envelopes, filters and FM bells, 19 sounds, and any of them can be
  replaced by dropping a file in `packs/<pack>/sounds/`.
- **Launch from the Falkbuilt factory.** The Calgary Tower is replaced by a
  caricature of Falkbuilt's Calgary building (glass corner, brick pillars,
  glowing FALKBUILT roof letters); the ramp leaves its flat roof.
- **Watch the crash.** Slow motion on impact, the camera on the hole, the dog
  stuck head-first then sliding in with paws kicking, a 3 s hold before the
  postcard.
- **More and better fireworks.** New rocket sprite, three-shell colour bursts
  on pickup, crackles during the ride, rockets the most common booster.
- **Background music and better wind.** Five public-domain carols recorded by
  the US Air Force and US Army bands (credits in the sounds folder) play as a
  playlist; the wind is three layered loops mixed by airspeed.

### Second feedback pass (2026-09-11, afternoon)

- **The rooftop is the run-up.** The ramp geometry changed: the sled starts at
  the far end of the Falkbuilt roof (y = 12 m), runs flat along it and rides a
  short kicker at the roof edge; there is no ski-jump structure any more.
  Constants `RAMP_TOP`, `ROOF_Y`, `KICKER_X`, `LIP` in the core.
- **A missed jump is a fall.** No press before the edge tips the sled over at
  a crawl; the dog tumbles down the side of the building with no flapping and
  splats below. `Game::botched`.
- **Frost, not an ice cube.** A frozen dog is tinted icy blue, shivers, grows
  icicles and frost glints.
- **Cape.** The dog wears a cape in its scarf colour that streams against its
  motion and hangs while seated.
- **Fireworks redrawn** as comet-tailed streaks with a white flash, willow
  strands and smoke.
- **Wiggle is stronger** (impulse 14, cap 18 m/s, forward 2.6 m/s).
- **Storm clouds scatter** through the sky instead of sitting on one band.
- **Removed:** the town signposts, the ghost-run dots.
- **One looping track**, a sled-slide sound on the roof, and `tools/serve.py`
  serves with no-cache headers so the browser never shows a stale build.

### Third feedback pass (2026-09-11, evening)

- **The Falkbuilt building, from the photo.** The sprite is now the street-corner
  view: the short face recedes on the left, a red-brick corner pillar, the tan
  band, two storeys of blue glass on white mullions with the dark canopy, a
  brick base, and glowing FALKBUILT letters on both faces at the corner.
- **A real ski jump on the roof.** `ramp_point` is an inrun again: a wooden
  trestle tower at the back of the roof with the start gate 13 m up, a straight
  slope on a truss, a cubic transition that bottoms out just above the roof
  and a take-off table curling up 40 degrees off the front corner. Constants
  `RAMP_TOP`, `KICKER_X`, `KICKER_Y`, `LIP`, `LIP_ANGLE_DEG` in the core.
- **Removed the aim-arc dots.** Only the landing ring on the snow remains; the
  dotted flight path off the lip read as clutter.
- **Wiggle no longer moves the world.** The camera frames on a smoothed copy
  of the dog's velocity (`Cam::lead`), so a flap hops the dog on screen but
  the parallax layers stay put.
- **Altitude you can see.** Vertical parallax of the scenery is decoupled from
  distance (`draw::para_y`): the landscape sinks as the dog climbs, the far
  ridge reaches the bottom edge around 60 m and is gone by 90 m; layers wash
  out with height and the sky blue deepens from 20 m up.
- **Wiggle is stronger again** (impulse 20, forward 4, cap 26 m/s).
- **The second press is spelled out.** Every ride carries a paper plate over
  the sled with a SPACE key cap and a mouse, "TO JUMP"; over the last stretch
  it becomes a gold JUMP! shout with the key cap under it. A missed press
  shows "PRESS SPACE OR CLICK AT THE LIP" under the banner during the fall,
  and the report-card tip names the keys. `hud::jump_prompt`, `hud::key_caps`.
- **Wiggles work like a wing.** Lift scales with airspeed (`Game::flap_lift`,
  0.6 near a hover, 1 at 30 m/s, 2.2 at 66 m/s and above): the kick, its
  cap and the repeat rate all follow it, so a fast flight climbs on wiggles
  and a stall barely flutters. Puff and spray grow with it.
- **The meteor.** Five seconds of free fall with no wiggling (`METEOR_AFTER`,
  the `fall_time` clock) and the dog curls into a rolling fireball
  (`Game::meteor`); a wiggle or a rocket resets the clock. From there a
  dive sheds most of its vertical drag and gravity pulls half again as hard,
  so a drop from space hits at over 70 m/s instead of the old 45 m/s drag
  ceiling. The impact scales with speed: a wider scorched crater, a bigger
  flash, hit-stop and shake, and a KABOOM! banner instead of SPLAT!.
- **Presents are easier to catch**: their reach is 2.8x (`PRESENT_REACH`),
  other pickups stay at 1.7x.
- **Mountains no longer jitter.** Scenery tiles are phased at the screen
  centre in tile units, so a zoom change scales the layer in place instead
  of sliding it sideways.

### Calgary's streets (2026-10-08)

The user asked to fly over real Calgary streets, recreated in a funny way so
people who live there feel at home. Drawn from ~25 street photos on
Wikimedia Commons (Inglewood's 9 Av SE, Stephen Avenue, the CTrain on 7 Av,
the +15s, Wonderland, Old City Hall, the Peace Bridge, Kensington, 17 Av SW,
the Saddledome, the Bow, the Calgary Tower).

- **Districts along the course** (`tools/gen_city.py` -> `packs/<pack>/city.json`,
  `crates/game/src/city.rs`): Inglewood from 22 m, Stephen Avenue 190 m, the
  Bow 400 m, Kensington 480 m, the Red Mile 650 m, the suburbs 850 m, the
  prairie from 1100 m. Facades stand on the street behind the snowbank;
  the ground turns into asphalt with a yellow centre line, and into the
  frozen Bow with a skating oval under the Peace Bridge.
- **Jokes for locals**: the Plaza marquee "IT'S A WONDERFUL CHINOOK", a +12 °C
  time-and-temperature sign on Dec 24, ROAD WORK "SEASON: ALL YEAR", a
  rooftop "PATIO OPEN!" in December, a donair shop, GO FLAMES GO on the Red
  Mile, the Saddledome in a Santa hat, cars plugged in by block heaters, a
  guy in shorts shovelling, identical suburban houses with only the door
  colour changed behind a "SAGE RIDGE PARK HILLS · PHASE 47 NOW SELLING"
  sign, a mule deer, the blue ring with its streetlamp, a 1964 drive-in,
  magpies everywhere, a white jackrabbit, a CTrain to SOMERSET with toques
  and white hats in the windows, ALBERTA POOL grain elevators, pumpjacks
  nodding in Santa hats, cows in toques.
- **Skyline**: one wide far layer (Calgary Tower with its torch lit, Bankers
  Hall, the Bow, Brookfield Place, Telus Sky, Suncor, Eighth Avenue Place,
  two cranes), centred over downtown, visible from the launch and towering
  over the Rockies on a climb.
- **Welcome banners** name each district as the dog flies in; the results
  card says where it came down ("landed on the Red Mile, 17 Av SW").
- People on the sidewalk hop and turn to the dog as it passes; critters hop.
- `render_sprites.js` honours `data-scale` (big facades render at 1x/1.5x)
  and `ONLY=<regex>`. Dev keys: `G` next district, `J` skip 55 m.

### Replay value: course, pacing, orders (2026-10-08, gameplay team)

Play-analysed headlessly (`crates/core/examples/playtest.rs`, scripted
players over hundreds of seeds; numbers in the team report). Found: mashing
beat careful play (endless stamina from the 25-wiggle refill plus presents,
and wiggles that kept adding speed), the launch had no effect on distance,
and long flights were 50-80% dead air with gaps up to 20 s.

- **Hand-authored patterns** (`crates/core/src/course.rs`): beats of present
  arcs, ladders up to a rocket, storm gates with presents threaded through,
  storm bait (close-shave line), rocket fans, Chinook lifts, trampoline hops,
  high caches, cold snaps, reindeer relays, constellations in space; weighted
  by Calgary district (fireworks on the Red Mile, Arctic gusts over the Bow,
  Chinooks on the prairie, trampolines in the suburbs). Laid relative to the
  line the dog is flying, just past the right edge of the frame, spaced in
  flight time (a beat every 1-2 s). Loose presents fill the gaps; storms,
  presents and the odd sleigh fill the sky above and below the line.
- **The launch relay**: the opening is simulated on a sweet-spot push with a
  perfect jump: presents, a rocket at the top of the arc, presents along the
  line the rocket really throws, a reindeer, more presents. Course boosters
  start where the relay ends. A clean launch rides it into town; a weak one
  passes under it.
- **Ears**: the forward kick fades out at a stall and at top speed (wiggles
  lift, boosters push); lift at a stall 0.25 (was 0.6); ears need air
  (weaker above 200 m, so space ends as a meteor); the 25-wiggle streak tops
  up 0.4 instead of refilling. Rockets have a top speed (150 m/s). Presents
  nudge 1.2/2.2 m/s and give 0.12 stamina (were 3/3.5 and 0.2).
- **Landing**: a stomp now survives an ordinary descent (crash x1.6, was
  x1.4) but needs 9 m/s of pace to spring, so landing well is a skill and
  crawling along the snow is not.
- **Boosters thin on the flight clock** (half after 25 s), not by distance,
  so every flight ends but a good start is not erased.
- **Close shaves**: flying past a storm cloud within 7.5 m pays stamina.
- **Santa's orders** (`missions.rs`, `crates/game/src/orders.rs`): three
  per-flight orders at a time from a 30-order book, then an endless one
  ("Catch 10 presents", "Reach the Bow", "Fly past 3 storm clouds", "Stomp a
  landing", "300 m without a storm hit"...). Completing one shows a DONE
  ribbon mid-flight; orders bank at the next launch; ranks (CADET to SANTA'S
  ACE) unlock capes, K changes cape before a launch.
- Pilots rebalanced: Pixie drag 0.0115 and ears 1.5, Nanook drag 0.0085.


### The ten-team studio pass (2026-10-08)

"The game looks cheap": ten teams worked in parallel, one domain each, in
separate worktrees merged 3-way.

- **Characters**: every dog redrawn; Buddy beagle, Nanook husky, Pixie
  Yorkie with seven distinct faces; Santa, elves, reindeer, sled, cloth cape
  (`tools/char_dog.py`, `tools/char_people.py`).
- **Environment**: painted Rockies, forested foothills layer, sculpted snow,
  sun halo and rays, horizon haze, far range over a cloud sea at altitude,
  bolted ramp with floodlight masts (`tools/terrain.py`). The moon waits
  until the sun has gone (space 0.7+), never both in one sky.
- **City**: two-depth painted downtown skyline, lit interiors, chimney
  smoke, traffic, five new Calgarians, district gate signs
  (`tools/city_skyline.py`).
- **Props**: 3/4-view presents (rare gold one), painted rocket with flame,
  angry thunderheads, all animated in `props.rs`.
- **VFX**: rewritten particles with colour ramps; five firework shell types
  on records and milestones; layered crash; four-depth snow
  (`vfx.rs`, `fxtex.rs`).
- **Rendering**: sprites load with bled alpha and mipmaps (no dark fringes),
  up to 2x supersampling, bloom, sun shafts, grade, vignette, WebGL2 with
  WebGL1 fallback; HUD drawn after post (`post.rs`, dev key O cycles it).
- **UI**: game logo, painted UI kit (`tools/gen_ui.py`, `ui.rs`), big picker
  cards with stat bars, banners that pick a slot clear of the dog, a new
  results card, painted loading page.
- **Animation**: spring rig, verlet cape, Santa and roof elves react,
  aim-ahead camera, PERFECT slow-motion beat (`anim.rs`).
- **Audio**: layered synthesis with variants, speed-tracked sled hiss, music
  ducking, a master limiter in `web/quad_snd.js`.
- **Gameplay**: see the section above (authored course, orders, capes).

Headless SwiftShader captures run the post-processed build at ~6 fps, which
throws the real-time camera off in stills; press O first for framing shots.
