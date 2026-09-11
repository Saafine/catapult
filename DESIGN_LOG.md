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
