//! Course composition: the sky ahead of the dog is laid out in beats of
//! hand-authored patterns instead of a uniform scatter.
//!
//! Every beat picks a pattern (an arc of presents, a ladder up to a rocket,
//! a storm gate with presents threaded through it...) from a table weighted
//! by the Calgary district below and the altitude of the flight, and places
//! it relative to the line the dog would fly if it did nothing. "On the
//! line" is free, "above the line" costs wiggles, "next to a storm" is the
//! risk-reward line. Beats are spaced in flight *time*, not meters, so a
//! fast run is as busy as a slow one: a good run has a peak every second or
//! two. The sky above the line gets its own sprinkle (caches of presents,
//! sleighs, loose storm clouds), so climbing is never into empty blue.

use crate::math::Vec2;
use crate::rng::Rng;
use crate::{Entity, EntityKind, Game, Phase, FLAP_IMPULSE, GLIDE_GRAVITY_SCALE, GRAVITY, PILOT_RADIUS, SPACE_ALTITUDE, STORM_CEILING};

/// Calgary's districts along the course. The distances match
/// `packs/calgary_xmas/city.json` (a test checks it), so the course and the
/// street under it change together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Inglewood,
    StephenAvenue,
    TheBow,
    Kensington,
    RedMile,
    Suburbs,
    Prairie,
}

pub const ZONES: [(f32, Zone, &str); 7] = [
    (22.0, Zone::Inglewood, "Inglewood"),
    (190.0, Zone::StephenAvenue, "Stephen Avenue"),
    (400.0, Zone::TheBow, "the Bow"),
    (480.0, Zone::Kensington, "Kensington"),
    (650.0, Zone::RedMile, "the Red Mile"),
    (850.0, Zone::Suburbs, "the suburbs"),
    (1100.0, Zone::Prairie, "the prairie"),
];

/// The district under distance `x` (Inglewood before the first one).
pub fn zone_at(x: f32) -> Zone {
    ZONES.iter().rev().find(|(from, _, _)| x >= *from).map(|z| z.1).unwrap_or(Zone::Inglewood)
}

/// The name of the district that starts exactly at `x`, for orders like
/// "Reach the Bow".
pub fn district_name_at(x: f32) -> Option<&'static str> {
    ZONES.iter().find(|(from, _, _)| (*from - x).abs() < 0.5).map(|z| z.2)
}

/// One hand-authored piece of course.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    /// A rainbow of presents along the line, on it or a few wiggles above.
    PresentArc,
    /// Presents climbing steeply off the line to a rocket at the top.
    Ladder,
    /// Two storm clouds stacked with a gap, presents threaded through it.
    StormGate,
    /// A storm cloud on the line with presents skimming its top edge and a
    /// reindeer behind: the close-shave line.
    StormBait,
    /// Three rockets fanned up and to the right.
    RocketFan,
    /// A Chinook under the line throwing the dog up a column of presents.
    ChinookLift,
    /// An elf trampoline on the street with an arc of presents above it.
    TrampolineHop,
    /// A scatter of presents well above the line, for climbers.
    HighCache,
    /// An Arctic gust on the line with presents above and below: pick a side.
    ColdSnap,
    /// Reindeer in a rising relay; each kick carries the dog to the next.
    ReindeerRelay,
    /// Up in space: a ring of presents around a UFO.
    Constellation,
}

/// How generous the course is with boosters after `secs` of flight past
/// the launch relay: 1 at first, a half after 25 s, a third after 50 s.
/// Presents never thin out, so late flight is still busy, but the energy to
/// stay up gets scarce and every run, however good, comes down somewhere.
/// It runs on flight time, not distance: thinning by distance was a wall
/// that every flight hit at the same place, so a great launch's head start
/// was worth nothing by the end. On a clock, the head start is kept.
pub fn booster_rate(secs: f32) -> f32 {
    1.0 / (1.0 + secs.max(0.0) / BOOST_HALF_LIFE)
}
pub const BOOST_HALF_LIFE: f32 = 25.0;

impl CourseState {
    /// [`booster_rate`] for a pattern laid at `x`, and none at all before
    /// the launch relay ends.
    pub fn boost_rate(&self, x: f32) -> f32 {
        match self.boost_t0 {
            Some(t0) if x >= self.boost_from => booster_rate(self.now - t0),
            _ => 0.0,
        }
    }
}

/// Pattern weights for a district. Low flight adds ground patterns, space
/// swaps the table for its own.
fn weights(zone: Zone, line: f32, boost: f32) -> Vec<(Pattern, f32)> {
    use Pattern::*;
    if line > SPACE_ALTITUDE - 60.0 {
        // Space is the reward for a big boost, not a launch pad: presents
        // and the odd UFO, no rockets, so every space trip comes back down.
        return vec![(Constellation, 3.0), (HighCache, 2.0), (PresentArc, 2.0)];
    }
    let mut w = match zone {
        // The old town: friendly, lots of presents and rockets, first storms as bait.
        Zone::Inglewood => vec![(PresentArc, 4.0), (RocketFan, 2.5), (Ladder, 2.0), (TrampolineHop, 2.0), (HighCache, 1.0), (ChinookLift, 1.0), (StormBait, 1.0)],
        // Downtown: climb between the towers, Chinooks between buildings, the first gates.
        Zone::StephenAvenue => vec![(Ladder, 3.0), (PresentArc, 3.0), (ChinookLift, 2.0), (StormGate, 2.0), (RocketFan, 2.0), (HighCache, 1.5), (StormBait, 1.5)],
        // Over the frozen river: Arctic air and low arcs across the ice.
        Zone::TheBow => vec![(ColdSnap, 3.0), (PresentArc, 3.0), (StormGate, 1.5), (ReindeerRelay, 1.0), (TrampolineHop, 1.0)],
        // Kensington: reindeer relays and caches up high.
        Zone::Kensington => vec![(ReindeerRelay, 3.0), (PresentArc, 2.0), (Ladder, 2.0), (HighCache, 2.0), (StormBait, 2.0), (StormGate, 1.5)],
        // The Red Mile is a party: fireworks everywhere, and storms to dodge.
        Zone::RedMile => vec![(RocketFan, 4.0), (StormGate, 2.5), (StormBait, 2.5), (PresentArc, 2.0), (Ladder, 1.0)],
        // Backyard trampolines in every yard, storm cells over the cul-de-sacs.
        Zone::Suburbs => vec![(TrampolineHop, 3.0), (StormGate, 3.0), (StormBait, 2.5), (PresentArc, 2.0), (ColdSnap, 1.0), (ChinookLift, 1.0)],
        // Open prairie: Chinooks and big weather.
        Zone::Prairie => vec![(ChinookLift, 4.0), (StormGate, 3.0), (StormBait, 2.5), (PresentArc, 2.0), (ColdSnap, 1.5), (ReindeerRelay, 1.0), (HighCache, 1.0)],
    };
    if line > 22.0 {
        // The trampoline stands on the street; from this high it is scenery.
        w.retain(|(p, _)| *p != TrampolineHop);
    } else if line < 6.0 {
        // Skimming the snow: a trampoline or a Chinook is the way back up.
        w.push((TrampolineHop, 3.0));
        w.push((ChinookLift, 2.0));
    }
    for (p, k) in w.iter_mut() {
        if matches!(p, RocketFan | ReindeerRelay | ChinookLift) {
            *k *= boost;
        }
    }
    if boost <= 0.0 {
        // The launch relay's stretch: its line stays clear, the sky above
        // it is for climbers.
        return vec![(HighCache, 1.0)];
    }
    w
}

/// Pick a pattern from weights, never the same one twice in a row.
fn pick(rng: &mut Rng, w: &[(Pattern, f32)], last: Option<Pattern>) -> Pattern {
    let w: Vec<(Pattern, f32)> = w.iter().copied().filter(|(p, _)| Some(*p) != last).collect();
    let total: f32 = w.iter().map(|(_, k)| k).sum();
    let mut r = rng.unit() * total;
    for (p, k) in &w {
        if r < *k {
            return *p;
        }
        r -= k;
    }
    w.last().map(|(p, _)| *p).unwrap_or(Pattern::PresentArc)
}

/// The line the dog would fly from `pos` with `vel` if it did nothing:
/// gravity, drag and any running wings or rocket, no bounces. Sampled at
/// 30 Hz for up to `secs`; flat along the snow after touchdown.
pub fn project_line(pos: Vec2, vel: Vec2, drag: f32, glide: f32, rocket: f32, secs: f32) -> Vec<Vec2> {
    project_line_with_lift(pos, vel, drag, glide, rocket, 0.0, secs)
}

/// [`project_line`] for a dog whose ears are working: `lift` m/s² of the
/// average upward pull the player's recent wiggling has been giving.
pub fn project_line_with_lift(pos: Vec2, vel: Vec2, drag: f32, glide: f32, rocket: f32, lift: f32, secs: f32) -> Vec<Vec2> {
    let dt = 1.0 / 30.0;
    let (mut p, mut v, mut glide, mut rocket) = (pos, vel, glide, rocket);
    let mut out = vec![p];
    let mut t = 0.0;
    while t < secs {
        let g = if glide > 0.0 { GRAVITY * GLIDE_GRAVITY_SCALE } else { GRAVITY } - lift;
        v.y -= g * dt;
        if rocket > 0.0 {
            v.x += 60.0 * dt;
            v.y += 12.0 * dt;
            rocket -= dt;
        }
        glide -= dt;
        let air = (1.0 - (p.y - PILOT_RADIUS).max(0.0) / 900.0).clamp(0.15, 1.0);
        let k = (1.0 - drag * v.length() * air * dt).max(0.0);
        v = v * k;
        p = p + v * dt;
        if p.y < PILOT_RADIUS {
            p.y = PILOT_RADIUS;
            v.y = 0.0;
            v.x = v.x.max(6.0);
        }
        out.push(p);
        t += dt;
    }
    out
}

/// Height of a projected line at `x`: interpolated inside it, carried on
/// along the last slope past its end (never below the snow).
pub fn line_y(line: &[Vec2], x: f32) -> f32 {
    if line.is_empty() {
        return 20.0;
    }
    if x <= line[0].x {
        return line[0].y;
    }
    for w in line.windows(2) {
        if x <= w[1].x && w[1].x > w[0].x {
            let k = (x - w[0].x) / (w[1].x - w[0].x);
            return w[0].y + (w[1].y - w[0].y) * k;
        }
    }
    let n = line.len();
    let (a, b) = (line[n.saturating_sub(8)], line[n - 1]);
    let slope = if b.x - a.x > 0.5 { ((b.y - a.y) / (b.x - a.x)).clamp(-3.0, 1.0) } else { -1.0 };
    (b.y + slope * (x - b.x)).max(PILOT_RADIUS)
}

/// Spawner state that persists between beats.
#[derive(Clone, Debug, Default)]
pub struct CourseState {
    pub last: Option<Pattern>,
    /// Patterns laid this run, in order; for tests and the play analysis.
    pub laid: Vec<(f32, Pattern)>,
    /// Where the last UFO was put; they are rationed.
    pub last_ufo_x: f32,
    /// Course boosters begin here: before it the launch relay is the only
    /// source of speed, so the launch decides how the flight begins.
    pub boost_from: f32,
    /// Flight time when the course came within reach of `boost_from`, and
    /// the flight time now; the booster clock.
    pub boost_t0: Option<f32>,
    pub now: f32,
    /// The flight the launch relay was laid on (sweet push, perfect jump).
    pub relay_line: Vec<Vec2>,
    /// Wiggles per second lately (a ~1.5 s moving average); the course
    /// reads the player's flapping when it projects the line.
    pub flap_rate: f32,
}

/// How much of the recent wiggling the line projection trusts to go on.
const LIFT_TRUST: f32 = 0.25;
/// Seconds the flap-rate average looks back.
pub const FLAP_RATE_WINDOW: f32 = 1.5;

/// Seconds of flight between the end of one pattern and the next one.
const BEAT_GAP: (f32, f32) = (0.45, 1.05);
/// Patterns are laid just past the right edge of the widest flight framing
/// (about 44 m ahead of the dog at zoom 0.62), so the line they are placed
/// on is projected only a second or two ahead and still true when the dog
/// gets there; nothing ever pops in on screen.
pub const LAY_AHEAD: f32 = 48.0;
/// At high speed, lay this many seconds of flight ahead instead.
const LOOKAHEAD_SECS: f32 = 0.6;

impl Game {
    /// How far ahead of the dog the course exists, meters.
    pub(crate) fn course_lookahead(&self) -> f32 {
        LAY_AHEAD.max(self.vel.x * LOOKAHEAD_SECS)
    }

    /// Lay beats until the course reaches the lookahead.
    pub(crate) fn lay_course(&mut self) {
        let target = self.pos.x + self.course_lookahead();
        self.course.now = self.flight_time;
        if self.course.boost_t0.is_none() && target >= self.course.boost_from && self.phase == Phase::Flying {
            self.course.boost_t0 = Some(self.flight_time);
        }
        if self.spawned_to >= target {
            return;
        }
        let flying = self.phase == Phase::Flying;
        let line = if flying {
            // Lay the course where this player is heading: a dog that keeps
            // its ears going is flying a flatter line than a ballistic one.
            let lift = (self.course.flap_rate * FLAP_IMPULSE * self.pilot.flap_power * self.flap_lift() * LIFT_TRUST).clamp(0.0, GRAVITY * 0.9);
            project_line_with_lift(self.pos, self.vel, self.pilot.drag, self.glide, self.rocket, lift, 4.5)
        } else {
            Vec::new()
        };
        let speed = if flying { self.vel.x.max(18.0) } else { 30.0 };
        while self.spawned_to < target {
            let x0 = self.spawned_to;
            let relay = std::mem::take(&mut self.course.relay_line);
            let boost_from = self.course.boost_from;
            // Over the relay stretch, things hang off the relay's own line,
            // unless the dog has left it far behind (climbed or dropped away).
            let on_relay = x0 < boost_from && !relay.is_empty() && (line.is_empty() || (line_y(&line, x0) - line_y(&relay, x0)).abs() < 12.0);
            let ly = |x: f32| {
                if on_relay && x < boost_from {
                    line_y(&relay, x)
                } else if line.is_empty() {
                    24.0
                } else {
                    line_y(&line, x)
                }
            };
            let l0 = ly(x0);
            let zone = zone_at(x0);
            let table = if x0 < boost_from && !on_relay {
                // Off the relay: presents on the dog's own line, still no boosters.
                vec![(Pattern::PresentArc, 2.0), (Pattern::HighCache, 1.0)]
            } else {
                weights(zone, l0, self.course.boost_rate(x0))
            };
            let pattern = pick(&mut self.rng, &table, self.course.last);
            let len = self.place(pattern, x0, &ly);
            self.course.last = Some(pattern);
            self.course.laid.push((x0, pattern));
            self.sky_sprinkle(x0, len, &ly);
            // Spaced in time: a fast dog meets patterns as often as a slow one.
            let gap = (speed * self.rng.range(BEAT_GAP.0, BEAT_GAP.1)).clamp(8.0, 140.0);
            self.filler(x0 + len, gap, &ly);
            self.course.relay_line = relay;
            self.separate_presents_from_storms();
            self.spawned_to = x0 + len + gap;
        }
    }

    /// Put the flying dog at `pos` with `vel` in a fresh stretch of course
    /// (past the launch relay, boosters on); for screenshots, not play.
    pub fn dev_course_at(&mut self, pos: Vec2, vel: Vec2) {
        self.dev_place(pos, vel);
        self.spawned_to = pos.x + 6.0;
        self.course.boost_from = self.course.boost_from.min(pos.x);
        self.course.boost_t0 = Some(self.flight_time);
        self.lay_course();
    }

    fn put(&mut self, kind: EntityKind, x: f32, y: f32) {
        let y = if kind == EntityKind::Trampoline { EntityKind::Trampoline.radius() * 0.6 } else { y.max(2.2) };
        self.entities.push(Entity { kind, pos: Vec2::new(x, y), alive: true, dead_for: 0.0 });
    }

    /// Put one pattern down at `x0` along the line `ly`; returns its length.
    fn place(&mut self, p: Pattern, x0: f32, ly: &dyn Fn(f32) -> f32) -> f32 {
        use EntityKind::*;
        let far = ((x0 - 150.0) / 900.0).clamp(0.0, 1.0);
        match p {
            Pattern::PresentArc => {
                // On the line, one wiggle up, or two: the choice is the skill.
                let lift = [0.5, 3.5, 6.5, 9.5][self.rng.range_i(0, 3) as usize];
                let n = self.rng.range_i(5, 7);
                let span = 2.6 * (n - 1) as f32;
                for k in 0..n {
                    let u = k as f32 / (n - 1) as f32;
                    let x = x0 + span * u;
                    self.put(Present, x, ly(x) + lift + (u * std::f32::consts::PI).sin() * 2.5);
                }
                span
            }
            Pattern::Ladder => {
                let steps = 6;
                for k in 0..steps {
                    let x = x0 + k as f32 * 3.0;
                    self.put(Present, x, ly(x0) + 1.5 + k as f32 * 2.6);
                }
                let top_x = x0 + steps as f32 * 3.0 + 2.0;
                let prize = if self.rng.unit() > self.course.boost_rate(x0) {
                    Present
                } else if self.rng.unit() < 0.7 {
                    Firework
                } else {
                    Reindeer
                };
                self.put(prize, top_x, ly(x0) + 1.5 + steps as f32 * 2.6 + 1.5);
                top_x - x0
            }
            Pattern::StormGate => {
                // The gap sits on the line or just above it; later gates come in pairs.
                let pairs = if far > 0.35 && self.rng.unit() < 0.5 { 2 } else { 1 };
                let mut x = x0;
                for i in 0..pairs {
                    let mid = ly(x + 6.0) + if i == 0 { self.rng.range(-1.0, 4.0) } else { self.rng.range(2.0, 7.0) };
                    let half = 7.6 - 0.6 * far;
                    self.put(Stormcloud, x + 6.0, mid + half);
                    if mid - half > 3.0 {
                        self.put(Stormcloud, x + 6.0, mid - half);
                    }
                    for k in 0..5 {
                        self.put(Present, x + k as f32 * 3.0, mid);
                    }
                    x += 18.0;
                }
                x - x0
            }
            Pattern::StormBait => {
                let y = ly(x0 + 5.0) + self.rng.range(-1.0, 1.0);
                self.put(Stormcloud, x0 + 5.0, y);
                // Presents skim the top edge: grabbing them is a close shave.
                for k in 0..4 {
                    let x = x0 + 1.0 + k as f32 * 2.6;
                    self.put(Present, x, y + 6.8 + (k as f32 * 0.9).sin());
                }
                if self.rng.unit() < self.course.boost_rate(x0) {
                    self.put(Reindeer, x0 + 15.0, y + 6.0);
                }
                15.0
            }
            Pattern::RocketFan => {
                let lift = if self.rng.unit() < 0.5 { 0.5 } else { 5.0 };
                for k in 0..3 {
                    let kf = k as f32;
                    self.put(Firework, x0 + kf * 3.2, ly(x0) + lift + kf * 2.4);
                }
                // A short trail of presents in the wake.
                for k in 0..3 {
                    let x = x0 + 10.0 + k as f32 * 2.6;
                    self.put(Present, x, ly(x) + lift + 6.0);
                }
                15.0
            }
            Pattern::ChinookLift => {
                let base = ly(x0 + 3.0);
                self.put(Updraft, x0 + 3.0, (base - 4.0).max(5.0));
                for k in 0..5 {
                    self.put(Present, x0 + 6.0 + k as f32 * 1.8, base + 5.0 + k as f32 * 3.4);
                }
                15.0
            }
            Pattern::TrampolineHop => {
                self.put(Trampoline, x0 + 2.0, 0.0);
                for k in 0..5 {
                    let u = k as f32 / 4.0;
                    self.put(Present, x0 + 6.0 + u * 12.0, 8.0 + (u * std::f32::consts::PI).sin() * 7.0);
                }
                18.0
            }
            Pattern::HighCache => {
                let base = ly(x0) + self.rng.range(14.0, 24.0);
                let n = self.rng.range_i(4, 6);
                for k in 0..n {
                    let x = x0 + k as f32 * 3.4;
                    let y = base + self.rng.range(-3.0, 3.0);
                    self.put(Present, x, y);
                }
                if self.rng.unit() < 0.5 * self.course.boost_rate(x0) && base < SPACE_ALTITUDE - 100.0 {
                    self.put(Firework, x0 + n as f32 * 3.4 + 2.0, base + 2.0);
                }
                n as f32 * 3.4
            }
            Pattern::ColdSnap => {
                let y = ly(x0 + 6.0);
                let wy = y + self.rng.range(-1.0, 1.0);
                self.put(ColdWind, x0 + 6.0, wy);
                for k in 0..3 {
                    let x = x0 + 2.0 + k as f32 * 3.0;
                    self.put(Present, x, y + 9.0 + k as f32);
                    if y - 9.0 > 3.0 {
                        self.put(Present, x, y - 9.0);
                    }
                }
                12.0
            }
            Pattern::ReindeerRelay => {
                let n = if far > 0.3 { 3 } else { 2 };
                let y = ly(x0) + self.rng.range(0.0, 3.0);
                for k in 0..n {
                    let kf = k as f32;
                    self.put(Reindeer, x0 + kf * 16.0, y + kf * 9.0);
                }
                16.0 * (n - 1) as f32
            }
            Pattern::Constellation => {
                let c = Vec2::new(x0 + 8.0, ly(x0 + 8.0) + self.rng.range(2.0, 8.0));
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::TAU / 8.0;
                    self.put(Present, c.x + a.cos() * 7.0, c.y + a.sin() * 7.0);
                }
                // A UFO tows hard: at most one every few hundred meters.
                if x0 - self.course.last_ufo_x > 300.0 {
                    self.course.last_ufo_x = x0;
                    self.put(Ufo, c.x, c.y);
                }
                16.0
            }
        }
    }

    /// Loose singles in the gap after a pattern, within a wiggle or a drop
    /// of the line, so the frame ahead of the dog is never bare sky.
    fn filler(&mut self, from: f32, gap: f32, ly: &dyn Fn(f32) -> f32) {
        if gap < 14.0 {
            return;
        }
        let n = 1 + (gap / 30.0) as i32;
        let boost = self.course.boost_rate(from);
        let calm = from < self.course.boost_from;
        for k in 0..n.min(3) {
            let x = from + gap * (k as f32 + self.rng.range(0.3, 0.7)) / n as f32;
            let y = ly(x) + self.rng.range(-11.0, 12.0);
            let r = self.rng.unit();
            let kind = if !calm && r < 0.08 * boost && y < SPACE_ALTITUDE - 100.0 {
                EntityKind::Firework
            } else if !calm && r > 0.9 {
                EntityKind::Stormcloud
            } else {
                EntityKind::Present
            };
            self.put(kind, x, y);
        }
    }

    /// Above the line: never empty. Loose storm clouds thicken with
    /// distance, the odd sleigh cruises the upper sky, UFOs live in space.
    fn sky_sprinkle(&mut self, x0: f32, len: f32, ly: &dyn Fn(f32) -> f32) {
        let x = x0 + len * 0.5;
        let l = ly(x);
        let storm_p = if x < self.course.boost_from {
            0.0
        } else if x < 250.0 {
            0.3
        } else {
            (0.45 + (x - 250.0) / 1200.0).min(0.8)
        };
        if self.rng.unit() < storm_p {
            // Off the line but in the frame (the flight camera shows about
            // 15 m above and below the dog): weather to steer around.
            let side = if self.rng.unit() < 0.6 { 1.0 } else { -1.0 };
            let y = (l + side * self.rng.range(10.0, 22.0)).clamp(8.0, STORM_CEILING);
            self.last_storm_x = x;
            let sx = x + self.rng.range(-4.0, 4.0);
            self.put(EntityKind::Stormcloud, sx, y);
        }
        if x >= self.course.boost_from {
            // One wiggle up: a pair or three of presents just above the line.
            if self.rng.unit() < 0.5 {
                let y = l + self.rng.range(9.0, 15.0);
                let n = self.rng.range_i(2, 3);
                for k in 0..n {
                    self.put(EntityKind::Present, x + k as f32 * 2.4, y + k as f32 * 1.1);
                }
            }
            // And below it, for the dog that holds its ears and drops.
            if l > 14.0 && self.rng.unit() < 0.3 {
                let y = l - self.rng.range(7.0, 10.0);
                self.put(EntityKind::Present, x + 6.0, y);
                self.put(EntityKind::Present, x + 8.4, y - 0.8);
            }
        }
        if self.rng.unit() < 0.07 && x >= self.course.boost_from {
            let y = (l + self.rng.range(25.0, 60.0)).clamp(40.0, 320.0);
            self.put(EntityKind::Sleigh, x + 3.0, y);
        }
        if l > SPACE_ALTITUDE - 100.0 && x - self.course.last_ufo_x > 300.0 && self.rng.unit() < 0.3 {
            self.course.last_ufo_x = x;
            let uy = l + self.rng.range(10.0, 50.0);
            self.put(EntityKind::Ufo, x + 5.0, uy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    /// A flight placed past the launch relay, cruising, with an empty sky
    /// ahead for the course to fill.
    fn cruise(x: f32, y: f32, vx: f32, seed: u64) -> Game {
        let mut g = Game::new(PILOTS[0], seed);
        g.phase = Phase::Flying;
        g.entities.clear();
        g.spawned_to = x;
        g.pos = Vec2::new(x, y);
        g.vel = Vec2::new(vx, 0.0);
        g.course.boost_t0 = Some(0.0);
        g
    }

    /// Lay the course over a stretch of town without flying through it.
    fn lay_over(g: &mut Game, from: f32, to: f32, y: f32) {
        let mut x = from;
        while x < to {
            g.pos = Vec2::new(x, y);
            g.vel = Vec2::new(30.0, 0.0);
            // Wings on and no drag: the projected line stays near `y`.
            g.glide = 1.0e6;
            g.pilot.drag = 0.0;
            g.lay_course();
            x += 10.0;
        }
    }

    #[test]
    fn zones_match_the_calgary_pack() {
        let json = include_str!("../../../packs/calgary_xmas/city.json");
        let froms: Vec<f32> = json
            .match_indices("\"from\"")
            .filter_map(|(i, _)| {
                let rest = &json[i + 6..];
                let rest = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
                let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(rest.len());
                rest[..end].parse().ok()
            })
            .collect();
        let ours: Vec<f32> = ZONES.iter().map(|z| z.0).collect();
        for f in &ours {
            assert!(froms.contains(f), "zone at {f} m is not a district in city.json ({froms:?})");
        }
        assert_eq!(zone_at(10.0), Zone::Inglewood);
        assert_eq!(zone_at(420.0), Zone::TheBow);
        assert_eq!(zone_at(5000.0), Zone::Prairie);
        assert_eq!(district_name_at(400.0), Some("the Bow"));
    }

    #[test]
    fn the_course_is_patterns_and_districts_have_their_own_flavour() {
        let count = |from: f32, to: f32, p: Pattern| {
            let mut n = 0;
            let mut total = 0;
            for seed in 0..12 {
                let mut g = cruise(from, 30.0, 30.0, seed);
                lay_over(&mut g, from, to, 30.0);
                n += g.course.laid.iter().filter(|(x, q)| *x >= from && *q == p).count();
                total += g.course.laid.iter().filter(|(x, _)| *x >= from).count();
            }
            n as f32 / total.max(1) as f32
        };
        // Fireworks on the Red Mile, Arctic gusts over the frozen Bow,
        // Chinooks on the prairie, more than anywhere else.
        assert!(count(650.0, 850.0, Pattern::RocketFan) > count(400.0, 480.0, Pattern::RocketFan));
        assert!(count(400.0, 480.0, Pattern::ColdSnap) > count(190.0, 400.0, Pattern::ColdSnap));
        assert!(count(1100.0, 1500.0, Pattern::ChinookLift) > count(650.0, 850.0, Pattern::ChinookLift));
        // Never the same pattern twice in a row.
        let mut g = cruise(300.0, 30.0, 30.0, 3);
        lay_over(&mut g, 300.0, 1300.0, 30.0);
        assert!(g.course.laid.len() > 20);
        assert!(g.course.laid.windows(2).all(|w| w[0].1 != w[1].1));
    }

    #[test]
    fn beats_come_every_second_or_two_whatever_the_speed() {
        for vx in [15.0f32, 40.0, 90.0] {
            let mut g = cruise(300.0, 40.0, vx, 1);
            g.pilot.drag = 0.0;
            let mut t = 0.0;
            while t < 10.0 {
                g.glide = 1.0e6;
                g.pos.y = 40.0;
                g.vel = Vec2::new(vx, 0.0);
                // Count the beats, don't fly into them.
                g.entities.retain(|e| e.pos.x > g.pos.x + 20.0);
                g.step(1.0 / 60.0);
                t += 1.0 / 60.0;
            }
            let start = 300.0 + LAY_AHEAD;
            let end = g.pos.x;
            let beats = g.course.laid.iter().filter(|(x, _)| *x > start && *x < end).count() as f32;
            let secs = (end - start) / vx;
            let per_10s = beats / secs * 10.0;
            // Measured: about 6 at 15 m/s, 9 at 40, 13 at 90. Faster flight is
            // busier by design; slow flight still meets a beat every 2 s.
            assert!((4.5..=14.0).contains(&per_10s), "{per_10s} beats per 10 s at {vx} m/s");
        }
    }

    #[test]
    fn a_storm_gate_threads_presents_between_two_clouds() {
        let mut g = cruise(300.0, 40.0, 30.0, 1);
        let line = |_: f32| 40.0;
        g.place(Pattern::StormGate, 300.0, &line);
        let storms: Vec<Vec2> = g.entities.iter().filter(|e| e.kind == EntityKind::Stormcloud).map(|e| e.pos).collect();
        let presents: Vec<Vec2> = g.entities.iter().filter(|e| e.kind == EntityKind::Present).map(|e| e.pos).collect();
        assert!(storms.len() >= 2 && presents.len() >= 5);
        let hit = EntityKind::Stormcloud.radius() + PILOT_RADIUS;
        for p in &presents {
            for s in &storms {
                // The dog can fly the presents' line without touching a cloud,
                // and close enough to skim one.
                assert!((*p - *s).length() > hit + 1.0, "present {p:?} too close to storm {s:?}");
            }
        }
        assert!(storms.iter().any(|s| s.y > 40.0) && storms.iter().any(|s| s.y < 40.0));
    }

    #[test]
    fn the_sky_above_the_line_is_never_empty() {
        let mut above = 0;
        let mut on = 0;
        for seed in 0..6 {
            let mut g = cruise(300.0, 30.0, 30.0, seed);
            lay_over(&mut g, 300.0, 700.0, 30.0);
            // The wing-glide line sags about 6 m over the lay-ahead.
            above += g.entities.iter().filter(|e| e.pos.y > 30.0 + 10.0).count();
            on += g.entities.iter().filter(|e| (e.pos.y - 27.0).abs() < 10.0).count();
        }
        assert!(above >= 6 * 6, "only {above} things above the line");
        assert!(on > above, "the line itself should be busiest ({on} on, {above} above)");
    }

    #[test]
    fn space_has_no_rockets_and_ufos_are_rationed() {
        let mut g = cruise(600.0, 500.0, 40.0, 2);
        lay_over(&mut g, 600.0, 2600.0, 500.0);
        assert_eq!(g.entities.iter().filter(|e| e.kind == EntityKind::Firework).count(), 0);
        let ufos = g.entities.iter().filter(|e| e.kind == EntityKind::Ufo).count();
        assert!((1..=7).contains(&ufos), "{ufos} UFOs over 2 km of space");
    }

    #[test]
    fn boosters_thin_out_on_the_flight_clock_not_the_map() {
        assert!(booster_rate(0.0) > booster_rate(25.0) && booster_rate(25.0) > booster_rate(60.0));
        assert!((booster_rate(BOOST_HALF_LIFE) - 0.5).abs() < 1e-4);
        let mut c = CourseState { boost_from: 200.0, boost_t0: Some(5.0), now: 5.0, ..Default::default() };
        assert_eq!(c.boost_rate(150.0), 0.0, "no course boosters inside the launch relay");
        assert_eq!(c.boost_rate(900.0), 1.0, "a fresh course is generous wherever it starts");
        c.now = 30.0;
        assert!((c.boost_rate(900.0) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn a_perfect_launch_rides_the_relay_and_a_weak_one_misses_it() {
        for (i, pilot) in PILOTS.iter().enumerate() {
            let fly = |power: f32, press: f32| {
                let mut g = Game::new(*pilot, 11 + i as u64);
                g.phase = Phase::AimPower { meter: power };
                g.action();
                let mut pressed = false;
                let mut relay = 0;
                while g.phase != Phase::Landed && g.flight_time < 60.0 {
                    if let Phase::Sliding { t, .. } = g.phase {
                        if !pressed && t >= press {
                            g.action();
                            pressed = true;
                        }
                    }
                    let before = g.pos.x < g.course.boost_from;
                    g.step(1.0 / 120.0);
                    if before {
                        relay = g.stats.rockets + g.stats.reindeer;
                    }
                }
                (relay, g.distance())
            };
            let (perfect, far) = fly(0.85, 0.985);
            let (weak, near) = fly(0.5, 0.62);
            assert_eq!(perfect, 2, "{}: the relay's rocket and reindeer", pilot.name);
            assert_eq!(weak, 0, "{}", pilot.name);
            assert!(far > near * 2.0, "{}: perfect {far} vs weak {near}", pilot.name);
        }
    }
}
