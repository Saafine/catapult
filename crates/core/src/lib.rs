//! Pure game logic for the Christmas dog launch game.
//!
//! This crate knows nothing about windows, input devices, sound, or files.
//! It exposes a [`Game`] state machine that a runtime drives with
//! [`Game::step`] and a handful of input methods. Everything here is
//! deterministic given the same inputs and seed, which also makes it
//! trivially testable.

pub mod math;
pub mod rng;

use math::Vec2;
use rng::Rng;

// ---------------------------------------------------------------------------
// Tunables
// ---------------------------------------------------------------------------

/// World units are meters. Gravity is arcade-strong so flights feel snappy.
pub const GRAVITY: f32 = 22.0;
/// Boosters and presents are easy to touch: their hitbox is this much bigger
/// than they look. Hazards keep their drawn size.
pub const PICKUP_REACH: f32 = 1.7;
/// How long a cold wind keeps the pilot frozen (no flapping, heavy drag).
pub const FREEZE_TIME: f32 = 1.5;
/// Downward speed at which a landing stops being a bounce and becomes a crash.
pub const CRASH_SPEED: f32 = 17.0;
/// Fraction of gravity that applies while gliding on angel wings.
pub const GLIDE_GRAVITY_SCALE: f32 = 0.22;
/// How far ahead of the pilot we keep the world populated.
pub const SPAWN_LOOKAHEAD: f32 = 70.0;
/// Radius of the pilot's collision circle.
pub const PILOT_RADIUS: f32 = 1.0;
/// Below this ground speed the run ends.
pub const REST_SPEED: f32 = 1.5;

/// Where the ski-jump lip is: the pilot leaves the ramp here.
pub const LIP: Vec2 = Vec2::new(0.0, 9.0);
/// Where the sled waits at the top of the ramp.
pub const RAMP_TOP: Vec2 = Vec2::new(-16.0, 19.0);
/// Seconds the slide down the ramp takes at full power.
pub const SLIDE_TIME: f32 = 1.4;

/// Speed at which the power meter sweeps 0..1..0, in sweeps per second.
pub const POWER_METER_HZ: f32 = 0.9;
/// Speed at which the angle meter sweeps between its limits.
/// Jump timing: pressing in the last stretch of the ramp is a perfect jump.
pub const JUMP_WINDOW_START: f32 = 0.5;
pub const JUMP_PERFECT_FROM: f32 = 0.92;
/// Extra speed for a perfect jump, pushed forward at 45 degrees so a
/// good press flies farther, not just higher.
pub const JUMP_BOOST: f32 = 18.0;
pub const JUMP_ANGLE_DEG: f32 = 45.0;
/// Missing the jump: the sled just drops off the lip at a fraction of the speed.
pub const MISSED_JUMP_SPEED: f32 = 0.6;

/// Upward kick from one paw wiggle, m/s.
pub const FLAP_IMPULSE: f32 = 9.0;
/// Forward kick from one paw wiggle, m/s.
pub const FLAP_FORWARD: f32 = 1.2;
/// Stamina cost of one wiggle. Stamina is 0..1.
pub const FLAP_COST: f32 = 0.08;
/// Stamina regained per second in the air.
pub const FLAP_REGEN: f32 = 0.04;
/// Stamina regained per present.
pub const FLAP_PRESENT_BONUS: f32 = 0.2;
/// Minimum gap between wiggles, so holding the key isn't a jetpack.
pub const FLAP_COOLDOWN: f32 = 0.16;
/// A wiggle never pushes vertical speed above this.
pub const FLAP_MAX_VY: f32 = 12.0;
/// Every this many wiggles in one run, stamina refills: the wiggle easter egg.
pub const WIGGLE_STREAK: u32 = 25;

/// Altitude milestones that fire [`Event::Milestone`].
pub const MILESTONES: [(f32, &str); 4] = [
    (60.0, "HIGH FLYER!"),
    (160.0, "ABOVE THE CLOUDS!"),
    (400.0, "SPACE!"),
    (700.0, "ORBIT!"),
];
/// Altitude at which the sky is fully space.
pub const SPACE_ALTITUDE: f32 = 400.0;

// ---------------------------------------------------------------------------
// Pilots
// ---------------------------------------------------------------------------

/// Static stats for one playable dog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pilot {
    pub name: &'static str,
    pub breed: &'static str,
    /// Launch speed at full power, m/s.
    pub launch_speed: f32,
    /// Air drag coefficient. Higher = slows down faster.
    pub drag: f32,
    /// Ground bounce restitution, 0..1.
    pub bounce: f32,
    /// Horizontal speed kept on each ground contact, 0..1.
    pub ground_grip: f32,
    /// Multiplier on wings duration.
    pub glide_bonus: f32,
    /// Multiplier on wiggle strength. Light dogs wiggle better.
    pub flap_power: f32,
}

pub const PILOTS: [Pilot; 3] = [
    Pilot {
        name: "Biscuit",
        breed: "corgi",
        launch_speed: 70.0,
        drag: 0.011,
        bounce: 0.55,
        ground_grip: 0.85,
        glide_bonus: 1.0,
        flap_power: 1.0,
    },
    Pilot {
        name: "Nanook",
        breed: "husky",
        launch_speed: 82.0,
        drag: 0.007,
        bounce: 0.35,
        ground_grip: 0.75,
        glide_bonus: 0.7,
        flap_power: 0.7,
    },
    Pilot {
        name: "Pixie",
        breed: "chihuahua",
        launch_speed: 60.0,
        drag: 0.015,
        bounce: 0.72,
        ground_grip: 0.92,
        glide_bonus: 1.6,
        flap_power: 1.4,
    },
];

// ---------------------------------------------------------------------------
// Things in the world
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    /// Elves holding a trampoline on the ground. Bounces the pilot straight up, hard.
    Trampoline,
    /// Flying reindeer. Knocks the pilot forward and up.
    Reindeer,
    /// Angel wings pickup. Reduces gravity for a few seconds.
    Wings,
    /// Firework rocket pickup. Big forward burst.
    Firework,
    /// Wrapped present. Score and a bit of stamina.
    Present,
    /// Santa's sleigh, rare and high up. Huge boost.
    Sleigh,
    /// UFO, only in space. Tows the pilot even further.
    Ufo,
    /// A Chinook: a column of warm wind that throws the pilot upward.
    Updraft,
    /// An Arctic gust: touching it freezes the pilot solid for a moment.
    ColdWind,
    /// Grumpy storm cloud. Saps speed and stamina. Denser further out.
    Stormcloud,
}

impl EntityKind {
    pub fn radius(self) -> f32 {
        match self {
            EntityKind::Trampoline => 1.8,
            EntityKind::Reindeer => 1.5,
            EntityKind::Wings => 1.6,
            EntityKind::Firework => 1.4,
            EntityKind::Present => 0.9,
            EntityKind::Sleigh => 2.4,
            EntityKind::Ufo => 2.6,
            EntityKind::Updraft => 4.5,
            EntityKind::ColdWind => 3.2,
            EntityKind::Stormcloud => 2.2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub pos: Vec2,
    /// Consumed entities stay in the list (so the renderer can play a
    /// fade-out) but no longer collide.
    pub alive: bool,
    /// Seconds since consumed, for effects. Only meaningful when !alive.
    pub dead_for: f32,
}

// ---------------------------------------------------------------------------
// Game state machine
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Player is choosing push power. `meter` sweeps 0..1.
    AimPower { meter: f32 },
    /// Power locked, player is choosing the jump angle at the lip.
    /// Sled is sliding down the ramp. `t` is 0..1 along the ramp.
    /// Riding the ramp; `t` is 0 at the top and 1 at the lip. One press on
    /// the way down sets the jump quality (later is better); the launch
    /// itself always happens at the lip.
    Sliding { power: f32, t: f32, jump: Option<f32> },
    Flying,
    /// Run over. Waiting for the player to reset.
    Landed,
}

/// Events emitted during a step so the runtime can play sounds or particles
/// without the core knowing what those are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The sled started sliding.
    Pushed,
    /// The pilot left the lip.
    Launched,
    /// The pilot wiggled.
    Flap,
    /// Wiggle streak reached: stamina refilled.
    WigglePower,
    Hit(EntityKind),
    GroundBounce { speed: f32 },
    /// Hit the ground so hard the run ends in a hole.
    Crashed { speed: f32 },
    /// The jump off the ramp; quality 0..1, 1 for a press right at the lip,
    /// 0 for a missed jump.
    Jumped { quality: f32 },
    /// An altitude milestone was reached; index into [`MILESTONES`].
    Milestone(usize),
    Landed { distance: f32 },
}

#[derive(Clone, Debug)]
pub struct Game {
    pub pilot: Pilot,
    pub phase: Phase,
    pub pos: Vec2,
    pub vel: Vec2,
    /// Seconds of wings remaining.
    pub glide: f32,
    /// Seconds of freeze left after a cold wind; 0 when free.
    pub frozen: f32,
    /// Seconds of firework thrust remaining.
    pub rocket: f32,
    /// Wiggle stamina, 0..1.
    pub stamina: f32,
    /// Seconds until the next wiggle is allowed.
    pub flap_cooldown: f32,
    /// Total wiggles this run, for stats.
    pub flaps: u32,
    pub entities: Vec<Entity>,
    pub presents: u32,
    pub max_height: f32,
    pub flight_time: f32,
    pub best_distance: f32,
    /// True when the run ended head-first in the snow.
    pub crashed: bool,
    /// Highest milestone index reached this run, if any.
    pub milestone: Option<usize>,
    /// Where a sweet-spot launch first touches down: the landing-zone marker.
    pub landing_x: f32,
    pub events: Vec<Event>,
    rng: Rng,
    /// x coordinate up to which entities have been spawned.
    spawned_to: f32,
    last_storm_x: f32,
    seed: u64,
}

/// Point on the ramp for `t` in 0..1: a cubic curve from the top of the
/// tower down to the lip, dipping below the lip so the end kicks upward.
pub fn ramp_point(t: f32) -> Vec2 {
    let p0 = RAMP_TOP;
    let p1 = Vec2::new(-12.0, 10.0);
    let p2 = Vec2::new(-3.5, 4.2);
    let p3 = LIP;
    let u = 1.0 - t;
    let a = u * u * u;
    let b = 3.0 * u * u * t;
    let c = 3.0 * u * t * t;
    let d = t * t * t;
    Vec2::new(
        a * p0.x + b * p1.x + c * p2.x + d * p3.x,
        a * p0.y + b * p1.y + c * p2.y + d * p3.y,
    )
}

impl Game {
    pub fn new(pilot: Pilot, seed: u64) -> Self {
        let mut g = Game {
            pilot,
            phase: Phase::AimPower { meter: 0.0 },
            pos: Vec2::new(RAMP_TOP.x, RAMP_TOP.y + PILOT_RADIUS),
            vel: Vec2::ZERO,
            glide: 0.0,
            frozen: 0.0,
            rocket: 0.0,
            stamina: 1.0,
            flap_cooldown: 0.0,
            flaps: 0,
            entities: Vec::new(),
            presents: 0,
            max_height: 0.0,
            flight_time: 0.0,
            best_distance: 0.0,
            crashed: false,
            milestone: None,
            landing_x: 60.0,
            events: Vec::new(),
            rng: Rng::new(seed),
            spawned_to: 82.0,
            last_storm_x: 0.0,
            seed,
        };
        g.opening_chain();
        g.spawn_ahead();
        g
    }

    /// Start a new run with the same pilot, keeping the best distance.
    pub fn reset(&mut self) {
        let best = self.best_distance;
        self.seed = self.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        *self = Game::new(self.pilot, self.seed);
        self.best_distance = best;
        self.crashed = false;
    }

    pub fn set_pilot(&mut self, pilot: Pilot) {
        if matches!(self.phase, Phase::AimPower { .. }) {
            self.pilot = pilot;
        }
    }

    /// Distance travelled in meters.
    pub fn distance(&self) -> f32 {
        self.pos.x.max(0.0)
    }

    /// Altitude above the ground, meters.
    pub fn altitude(&self) -> f32 {
        (self.pos.y - PILOT_RADIUS).max(0.0)
    }

    /// Current jump angle in degrees, if in the angle phase or sliding.
    /// Jump quality for a press at ramp progress `t`: nothing before the
    /// window, rising to 1 at the lip.
    pub fn jump_quality(t: f32) -> f32 {
        let k = ((t - JUMP_WINDOW_START) / (1.0 - JUMP_WINDOW_START)).clamp(0.0, 1.0);
        k * k * (3.0 - 2.0 * k)
    }

    /// Ramp tangent direction at eased progress `e`.
    fn ramp_dir(e: f32) -> Vec2 {
        let a = ramp_point((e - 0.01).max(0.0));
        let b = ramp_point((e + 0.01).min(1.0));
        let d = b - a;
        let l = d.length().max(1e-4);
        Vec2::new(d.x / l, d.y / l)
    }

    fn slide_ease(t: f32) -> f32 {
        t.powf(2.4)
    }

    /// Leave the lip: full ramp speed along the lip tangent, plus the jump
    /// boost for the quality the press earned; a miss drops off slowly.
    fn jump(&mut self, power: f32, quality: f32) {
        self.pos = Vec2::new(LIP.x, LIP.y + PILOT_RADIUS);
        let dir = Self::ramp_dir(1.0);
        let launch = self.pilot.launch_speed * (0.25 + 0.75 * power);
        let speed = if quality > 0.0 { launch } else { launch * MISSED_JUMP_SPEED };
        let ja = JUMP_ANGLE_DEG.to_radians();
        let boost = JUMP_BOOST * quality;
        self.vel = Vec2::new(dir.x * speed + ja.cos() * boost, dir.y * speed + ja.sin() * boost);
        self.phase = Phase::Flying;
        self.events.push(Event::Launched);
    }

    pub fn action(&mut self) {
        match self.phase {
            Phase::AimPower { meter } => {
                self.phase = Phase::Sliding { power: Self::meter_value(meter), t: 0.0, jump: None };
                self.events.push(Event::Pushed);
            }
            Phase::Sliding { power, t, jump: None } => {
                // One press per ride: as late as possible, but before the lip.
                let q = Self::jump_quality(t).max(0.05);
                self.phase = Phase::Sliding { power, t, jump: Some(q) };
                self.events.push(Event::Jumped { quality: q });
            }
            Phase::Sliding { .. } => {}
            Phase::Flying => self.flap(),
            // Landed: the results card owns the restart button.
            Phase::Landed => {}
        }
    }

    /// Wiggle the paws for a bit of lift. Costs stamina; ignored on cooldown,
    /// when exhausted, or while rolling on the ground.
    pub fn flap(&mut self) {
        if self.phase != Phase::Flying
            || self.frozen > 0.0
            || self.flap_cooldown > 0.0
            || self.stamina < FLAP_COST
            || (self.pos.y <= PILOT_RADIUS + 0.01 && self.vel.y <= 0.0)
        {
            return;
        }
        let p = self.pilot.flap_power;
        let boosted = self.vel.y + FLAP_IMPULSE * p;
        self.vel.y = boosted.min(self.vel.y.max(FLAP_MAX_VY));
        self.vel.x += FLAP_FORWARD * p;
        self.stamina -= FLAP_COST;
        self.flap_cooldown = FLAP_COOLDOWN;
        self.flaps += 1;
        self.events.push(Event::Flap);
        if self.flaps % WIGGLE_STREAK == 0 {
            self.stamina = 1.0;
            self.vel.y = self.vel.y.max(0.0) + 8.0 * p;
            self.events.push(Event::WigglePower);
        }
    }

    /// Advance the simulation by `dt` seconds. Call with a fixed timestep.
    /// The runtime should take `events` after stepping.
    pub fn step(&mut self, dt: f32) {
        match self.phase {
            Phase::AimPower { meter } => {
                let m = (meter + dt * POWER_METER_HZ * 2.0) % 2.0;
                self.phase = Phase::AimPower { meter: m };
            }
            Phase::Sliding { power, t, jump } => {
                // Slower pushes take a little longer down the ramp.
                let speed = 1.0 / (SLIDE_TIME * (1.25 - 0.25 * power));
                let t2 = t + dt * speed;
                if t2 >= 1.0 {
                    // Off the lip with whatever jump was pressed; none is a miss.
                    if jump.is_none() {
                        self.events.push(Event::Jumped { quality: 0.0 });
                    }
                    self.jump(power, jump.unwrap_or(0.0));
                } else {
                    let p = ramp_point(Self::slide_ease(t2));
                    let prev = self.pos;
                    self.pos = Vec2::new(p.x, p.y + PILOT_RADIUS);
                    self.vel = (self.pos - prev) * (1.0 / dt);
                    self.phase = Phase::Sliding { power, t: t2, jump };
                }
            }
            Phase::Flying => self.step_flight(dt),
            Phase::Landed => {}
        }
        for e in &mut self.entities {
            if !e.alive {
                e.dead_for += dt;
            }
        }
    }

    /// Meter values sweep 0..2 internally; this folds them into a 0..1
    /// triangle wave so the needle bounces instead of snapping.
    pub fn meter_value(raw: f32) -> f32 {
        if raw <= 1.0 {
            raw
        } else {
            2.0 - raw
        }
    }

    fn step_flight(&mut self, dt: f32) {
        self.flight_time += dt;
        self.flap_cooldown = (self.flap_cooldown - dt).max(0.0);
        if self.frozen > 0.0 {
            // A block of ice flies badly: heavy drag, no lift of any kind.
            self.frozen = (self.frozen - dt).max(0.0);
            self.vel.x *= (1.0 - 0.45 * dt).max(0.0);
        }
        self.stamina = (self.stamina + FLAP_REGEN * dt).min(1.0);

        // Forces.
        let g_scale = if self.glide > 0.0 { GLIDE_GRAVITY_SCALE } else { 1.0 };
        self.vel.y -= GRAVITY * g_scale * dt;
        if self.rocket > 0.0 {
            self.vel.x += 60.0 * dt;
            self.vel.y += 12.0 * dt;
            self.rocket -= dt;
        }
        if self.glide > 0.0 {
            self.glide -= dt;
            self.vel.x += 1.5 * dt;
        }
        let speed = self.vel.length();
        // Thinner air up high: less drag, so space runs keep their speed.
        let air = (1.0 - self.altitude() / 900.0).clamp(0.15, 1.0);
        let drag = self.pilot.drag * speed * air;
        self.vel = self.vel * (1.0 - drag * dt).max(0.0);

        // Integrate.
        self.pos = self.pos + self.vel * dt;
        let alt = self.altitude();
        self.max_height = self.max_height.max(alt);
        for (i, (h, _)) in MILESTONES.iter().enumerate() {
            if alt >= *h && self.milestone.map_or(true, |m| i > m) {
                self.milestone = Some(i);
                self.events.push(Event::Milestone(i));
            }
        }

        // Ground.
        if self.pos.y < PILOT_RADIUS {
            self.pos.y = PILOT_RADIUS;
            if self.vel.y < 0.0 {
                let impact = -self.vel.y;
                let soft = self.entities.iter().any(|e| e.alive && e.kind == EntityKind::Trampoline && (e.pos.x - self.pos.x).abs() < 3.5);
                if impact > CRASH_SPEED && !soft {
                    // Straight into the snow: the run ends here, in a hole.
                    self.vel = Vec2::ZERO;
                    self.crashed = true;
                    self.phase = Phase::Landed;
                    let d = self.distance();
                    self.best_distance = self.best_distance.max(d);
                    self.events.push(Event::Crashed { speed: impact });
                    self.events.push(Event::Landed { distance: d });
                    return;
                }
                self.vel.y = impact * self.pilot.bounce;
                self.vel.x *= self.pilot.ground_grip;
                if self.vel.y < 2.0 {
                    self.vel.y = 0.0;
                } else {
                    self.events.push(Event::GroundBounce { speed: impact });
                }
            }
            if self.vel.y == 0.0 {
                self.vel.x *= (1.0 - 1.2 * dt).max(0.0);
                if self.vel.length() < REST_SPEED {
                    self.vel = Vec2::ZERO;
                    self.phase = Phase::Landed;
                    let d = self.distance();
                    self.best_distance = self.best_distance.max(d);
                    self.events.push(Event::Landed { distance: d });
                    return;
                }
            }
        }

        // Collisions.
        let hp = self.pos;
        for i in 0..self.entities.len() {
            let e = self.entities[i];
            if !e.alive {
                continue;
            }
            let reach = if e.kind == EntityKind::Stormcloud { 1.0 } else { PICKUP_REACH };
            let r = e.kind.radius() * reach + PILOT_RADIUS;
            if (e.pos - hp).length_sq() <= r * r {
                self.entities[i].alive = false;
                self.apply_hit(e.kind);
            }
        }

        self.spawn_ahead();
        let cutoff = self.pos.x - 80.0;
        self.entities.retain(|e| e.pos.x > cutoff);
    }

    fn apply_hit(&mut self, kind: EntityKind) {
        match kind {
            EntityKind::Trampoline => {
                self.vel.y = self.vel.y.abs().max(24.0) + 22.0;
                self.vel.x += 5.0;
            }
            EntityKind::Reindeer => {
                self.vel.x += 20.0;
                self.vel.y = self.vel.y.max(0.0) + 18.0;
            }
            EntityKind::Wings => {
                self.glide = 3.5 * self.pilot.glide_bonus;
                self.vel.y = self.vel.y.max(-2.0);
            }
            EntityKind::Firework => {
                self.rocket = 1.2;
                self.vel.x += 30.0;
            }
            EntityKind::Present => {
                self.presents += 1;
                self.stamina = (self.stamina + FLAP_PRESENT_BONUS).min(1.0);
                // A small nudge forward and up: every present is a little boost,
                // so a chain lifts the whole arc.
                self.vel.x += 3.0;
                self.vel.y += 3.5;
            }
            EntityKind::Sleigh => {
                self.vel.x += 30.0;
                self.vel.y = self.vel.y.max(0.0) + 28.0;
                self.stamina = 1.0;
            }
            EntityKind::ColdWind => {
                self.frozen = FREEZE_TIME;
                self.glide = 0.0;
                self.rocket = 0.0;
            }
            EntityKind::Updraft => {
                self.vel.y = self.vel.y.max(0.0) + 17.0;
                self.vel.x += 4.0;
                self.stamina = (self.stamina + 0.15).min(1.0);
            }
            EntityKind::Ufo => {
                self.vel.x += 40.0;
                self.vel.y = self.vel.y.max(0.0) + 35.0;
                self.stamina = 1.0;
            }
            EntityKind::Stormcloud => {
                self.vel = self.vel * 0.55;
                self.vel.y -= 7.0;
                self.stamina = (self.stamina - 0.35).max(0.0);
            }
        }
        self.events.push(Event::Hit(kind));
    }

    /// Presents never sit inside a storm cloud's reach: taking the bait must
    /// be a dodge, not a coin flip.
    fn separate_presents_from_storms(&mut self) {
        let storms: Vec<Vec2> = self.entities.iter().filter(|e| e.kind == EntityKind::Stormcloud).map(|e| e.pos).collect();
        for e in self.entities.iter_mut().filter(|e| e.kind != EntityKind::Stormcloud) {
            for sp in &storms {
                let d = e.pos - *sp;
                if d.length() < 6.5 {
                    let dir = if d.y >= 0.0 { 1.0 } else { -1.0 };
                    e.pos.y = sp.y + dir * 6.5;
                    if e.pos.y < 3.0 {
                        e.pos.y = sp.y + 6.5;
                    }
                }
            }
        }
    }

    /// The opening every run is laid on the arc a typical launch actually
    /// flies (sweet-spot push, 45 degree jump), so a first run is guaranteed
    /// to catch presents, a reindeer and wings before the first landing.
    fn opening_chain(&mut self) {
        let path = self.median_arc();
        let at = |x: f32| -> Vec2 {
            let mut best = path[0];
            for p in &path {
                if (p.x - x).abs() < (best.x - x).abs() {
                    best = *p;
                }
            }
            best
        };
        // Where the median arc meets the ground; everything is placed as a
        // fraction of that so it stays on the arc for every pilot.
        let landing = path.iter().find(|p| p.y < 2.0 && p.x > 20.0).map(|p| p.x).unwrap_or(80.0);
        self.landing_x = landing;
        let mut push = |kind: EntityKind, pos: Vec2| {
            self.entities.push(Entity { kind, pos, alive: true, dead_for: 0.0 });
        };
        // A short rising arc right off the lip so the very first frame of
        // flight already shows something to wiggle for.
        for k in 0..3 {
            let p = at(14.0 + k as f32 * 3.0);
            push(EntityKind::Present, Vec2::new(p.x, p.y + 2.2 + k as f32 * 0.6));
        }
        for k in 0..5 {
            let p = at(landing * (0.40 + k as f32 * 0.035));
            push(EntityKind::Present, Vec2::new(p.x, p.y + 0.3));
        }
        push(EntityKind::Reindeer, at(landing * 0.6));
        for k in 0..4 {
            let p = at(landing * (0.72 + k as f32 * 0.03));
            push(EntityKind::Present, Vec2::new(p.x, p.y + 0.3));
        }
        push(EntityKind::Trampoline, Vec2::new(landing + 3.0, EntityKind::Trampoline.radius() * 0.6));
    }

    /// Samples of the flight path for a sweet-spot push at 45 degrees, in
    /// an empty world, one sample per 2 m of travel.
    fn median_arc(&self) -> Vec<Vec2> {
        let mut g = Game {
            pilot: self.pilot,
            phase: Phase::Flying,
            pos: Vec2::new(LIP.x, LIP.y + PILOT_RADIUS),
            vel: Vec2::ZERO,
            glide: 0.0,
            frozen: 0.0,
            rocket: 0.0,
            stamina: 1.0,
            flap_cooldown: 0.0,
            flaps: 0,
            entities: Vec::new(),
            presents: 0,
            max_height: 0.0,
            flight_time: 0.0,
            best_distance: 0.0,
            milestone: None,
            landing_x: 0.0,
            events: Vec::new(),
            rng: Rng::new(1),
            spawned_to: 1.0e9,
            last_storm_x: 0.0,
            seed: 0,
            crashed: false,
        };
        let speed = self.pilot.launch_speed * (0.25 + 0.75 * 0.85);
        let a = 45.0f32.to_radians();
        g.vel = Vec2::new(a.cos(), a.sin()) * speed;
        let mut out = vec![g.pos];
        let mut steps = 0;
        while g.phase == Phase::Flying && steps < 5000 {
            g.step(1.0 / 120.0);
            steps += 1;
            if g.pos.x - out.last().unwrap().x >= 2.0 {
                out.push(g.pos);
            }
        }
        out
    }

    /// Where the dog will be, in height, when it reaches `x` on a plain
    /// ballistic arc from its current state; only meaningful while flying.
    fn projected_y(&self, x: f32) -> f32 {
        if self.phase != Phase::Flying || self.vel.x < 3.0 {
            return 22.0;
        }
        let t = ((x - self.pos.x) / self.vel.x).max(0.0);
        let g = if self.glide > 0.0 { GRAVITY * GLIDE_GRAVITY_SCALE } else { GRAVITY };
        self.pos.y + self.vel.y * t - 0.5 * g * t * t
    }

    fn spawn_ahead(&mut self) {
        fn x_far(x: f32) -> f32 {
            (x - 80.0).max(0.0)
        }
        let target = self.pos.x + SPAWN_LOOKAHEAD;
        while self.spawned_to < target {
            // Pickups thin out with distance while storm clouds thicken.
            let gap = self.rng.range(4.0, 11.0) * (1.0 + (x_far(self.spawned_to)) / 700.0);
            self.spawned_to += gap;
            let x = self.spawned_to;
            let roll = self.rng.unit();
            // No angel wings: rockets take their share. Cold winds only
            // past 120 m, once the player has the flap under control.
            let cold = x > 120.0 && roll >= 0.93;
            let kind = if cold {
                EntityKind::ColdWind
            } else if roll < 0.24 {
                EntityKind::Present
            } else if roll < 0.44 {
                EntityKind::Reindeer
            } else if roll < 0.56 {
                EntityKind::Trampoline
            } else if roll < 0.78 {
                EntityKind::Firework
            } else {
                EntityKind::Updraft
            };
            let sky = |rng: &mut Rng, lo: f32, hi: f32| {
                let u = rng.unit();
                lo + (hi - lo) * u * u
            };
            // Alternate slots between the low band (rolling, bouncing) and the
            // mid band (flying), so every screen has something in reach.
            // Three bands cycle by slot: low (rolling, bouncing), mid (the
            // usual arc) and high (what flapping and boosts can reach), so
            // the sky above the flight line is never empty.
            let slot = ((x / 10.0) as i32).rem_euclid(3);
            let (mid_slot, high_slot) = (slot == 0, slot == 2);
            let y = match kind {
                EntityKind::Trampoline => EntityKind::Trampoline.radius() * 0.6,
                EntityKind::Reindeer => if high_slot { self.rng.range(50.0, 110.0) } else if mid_slot { self.rng.range(14.0, 48.0) } else { sky(&mut self.rng, 5.0, 90.0) },
                EntityKind::Wings => if high_slot { self.rng.range(45.0, 100.0) } else if mid_slot { self.rng.range(12.0, 40.0) } else { sky(&mut self.rng, 5.0, 70.0) },
                EntityKind::Firework => if high_slot { self.rng.range(40.0, 90.0) } else if mid_slot { self.rng.range(10.0, 36.0) } else { sky(&mut self.rng, 4.0, 60.0) },
                EntityKind::Present => if mid_slot { self.rng.range(10.0, 40.0) } else { sky(&mut self.rng, 3.0, 80.0) },
                EntityKind::Updraft => if high_slot { self.rng.range(40.0, 120.0) } else { self.rng.range(12.0, 50.0) },
                EntityKind::ColdWind => self.projected_y(x + 2.0).clamp(10.0, 300.0) + self.rng.range(-4.0, 4.0),
                _ => 0.0,
            };
            self.entities.push(Entity { kind, pos: Vec2::new(x, y), alive: true, dead_for: 0.0 });
            if kind == EntityKind::Present {
                let n = self.rng.range_i(2, 5);
                for k in 1..=n {
                    let fx = x + k as f32 * 2.2;
                    let fy = y + (k as f32 * 0.9).sin() * 2.0;
                    self.entities.push(Entity { kind, pos: Vec2::new(fx, fy), alive: true, dead_for: 0.0 });
                }
            }
            // Past the opening, every other slot also drops a present in the
            // flight band so the descent is never empty.
            // A present high above the arc every third slot: bait for flappers.
            if x > 82.0 && high_slot {
                let y = if self.phase == Phase::Flying { (self.projected_y(x + 1.0) + self.rng.range(12.0, 30.0)).clamp(30.0, 400.0) } else { self.rng.range(45.0, 90.0) };
                self.entities.push(Entity { kind: EntityKind::Present, pos: Vec2::new(x + 1.0, y), alive: true, dead_for: 0.0 });
            }
            if x > 82.0 && self.rng.unit() < 0.9 {
                // Ladder just above the projected arc: reachable only by wiggling,
                // so the player learns "wiggle = reach" by wanting.
                let y = if self.phase == Phase::Flying { (self.projected_y(x + 3.0) + 2.5).clamp(6.0, 400.0) } else { self.rng.range(12.0, 40.0) };
                self.entities.push(Entity { kind: EntityKind::Present, pos: Vec2::new(x + 3.0, y + self.rng.range(-3.0, 3.0)), alive: true, dead_for: 0.0 });
                // Sometimes a short arc of three so the corridor has a line to chase.
                if self.rng.unit() < 0.4 {
                    for k in 1..3 {
                        let kf = k as f32;
                        self.entities.push(Entity { kind: EntityKind::Present, pos: Vec2::new(x + 3.0 + kf * 2.6, y + kf * 1.4), alive: true, dead_for: 0.0 });
                    }
                }
            }
            self.separate_presents_from_storms();
            // Storm clouds start around 80 m, roughly one per screen, and
            // get denser with distance.
            // Distance bands: the first 100 m is clean, then one cloud at a
            // time, then pairs and denser fields past 250 m.
            let storm_p = if x < 70.0 {
                0.0
            } else if x < 250.0 {
                0.22
            } else {
                (0.34 + (x - 250.0) / 500.0).min(0.7)
            };
            // Never more than 150 m without a storm in the flight band.
            let overdue = x > 70.0 && x - self.last_storm_x > 120.0;
            if self.rng.unit() < storm_p || overdue {
                self.last_storm_x = x;
                // On the arc the dog is projected to fly: integrate the current
                // velocity forward to the cloud's x.
                // A corridor around where the dog is heading in the next second or two.
                let centre = self.projected_y(x + 2.0).clamp(8.0, 380.0);
                let y = centre + self.rng.range(-6.0, 6.0);
                self.entities.push(Entity { kind: EntityKind::Stormcloud, pos: Vec2::new(x + 2.0, y), alive: true, dead_for: 0.0 });
            }
            // Rare high-altitude helpers: the sleigh cruises the upper sky,
            // the UFO only appears in space.
            if self.rng.unit() < 0.10 {
                let y = self.rng.range(90.0, 300.0);
                self.entities.push(Entity { kind: EntityKind::Sleigh, pos: Vec2::new(x + 3.0, y), alive: true, dead_for: 0.0 });
            }
            if self.rng.unit() < 0.12 {
                let y = self.rng.range(SPACE_ALTITUDE + 20.0, SPACE_ALTITUDE + 400.0);
                self.entities.push(Entity { kind: EntityKind::Ufo, pos: Vec2::new(x + 5.0, y), alive: true, dead_for: 0.0 });
            }
            self.separate_presents_from_storms();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fly(power: f32, jump_at: f32) -> Game {
        let mut g = Game::new(PILOTS[0], 7);
        g.phase = Phase::AimPower { meter: power };
        g.action();
        assert!(matches!(g.phase, Phase::Sliding { .. }));
        let mut steps = 0;
        let mut jumped = false;
        while g.phase != Phase::Landed && steps < 200_000 {
            // Pure physics: no pickups or hazards in the comparison.
            g.entities.clear();
            if let Phase::Sliding { t, .. } = g.phase {
                if !jumped && t >= jump_at {
                    g.action();
                    jumped = true;
                }
            }
            g.step(1.0 / 120.0);
            steps += 1;
        }
        g
    }

    #[test]
    fn a_launch_slides_then_flies_then_lands() {
        let g = fly(1.0, 0.95);
        assert_eq!(g.phase, Phase::Landed);
        assert!(g.distance() > 50.0, "distance was {}", g.distance());
        assert!(g.events.contains(&Event::Pushed));
        assert!(g.events.contains(&Event::Launched));
    }

    #[test]
    fn ramp_starts_at_top_and_ends_at_lip() {
        let a = ramp_point(0.0);
        let b = ramp_point(1.0);
        assert!((a.x - RAMP_TOP.x).abs() < 1e-4 && (a.y - RAMP_TOP.y).abs() < 1e-4);
        assert!((b.x - LIP.x).abs() < 1e-4 && (b.y - LIP.y).abs() < 1e-4);
        // The lip kicks up: the curve is below the lip just before the end.
        assert!(ramp_point(0.9).y < LIP.y);
    }

    #[test]
    fn more_power_flies_further_in_empty_world() {
        let mut lo = Game::new(PILOTS[0], 1);
        lo.entities.clear();
        lo.phase = Phase::Sliding { power: 0.3, t: 0.96, jump: None };
        let mut hi = lo.clone();
        hi.phase = Phase::Sliding { power: 1.0, t: 0.96, jump: None };
        for g in [&mut lo, &mut hi] {
            g.action();
            while g.phase != Phase::Landed {
                g.entities.clear();
                g.step(1.0 / 120.0);
            }
        }
        assert!(hi.distance() > lo.distance());
    }

    #[test]
    fn flapping_extends_flight_and_streak_refills() {
        let run = |flap: bool| {
            let mut g = Game::new(PILOTS[0], 3);
            g.entities.clear();
            g.phase = Phase::Sliding { power: 0.6, t: 0.96, jump: None };
            g.action();
            let mut t = 0.0;
            while g.phase != Phase::Landed {
                g.entities.clear();
                g.step(1.0 / 120.0);
                t += 1.0 / 120.0;
                if flap && t > 1.5 {
                    g.flap();
                }
            }
            g
        };
        let plain = run(false);
        let flapped = run(true);
        assert_eq!(plain.flaps, 0);
        assert!(flapped.flaps > 3);
        assert!(flapped.distance() > plain.distance() * 1.2);
        if flapped.flaps >= WIGGLE_STREAK {
            assert!(flapped.events.contains(&Event::WigglePower));
        }
    }

    #[test]
    fn a_late_jump_flies_farther_than_an_early_one_or_none() {
        let late = fly(1.0, 0.96).distance();
        let early = fly(1.0, 0.55).distance();
        let none = fly(1.0, 2.0).distance();
        assert!(late > early, "late {late} vs early {early}");
        assert!(early > none, "early {early} vs none {none}");
        assert!(late > 0.0 && none > 0.0);
        assert!(Game::jump_quality(0.99) > 0.95);
        assert_eq!(Game::jump_quality(0.3), 0.0);
    }

    #[test]
    fn a_hard_landing_ends_the_run_in_a_hole() {
        let mut g = Game::new(PILOTS[0], 3);
        g.entities.clear();
        g.phase = Phase::Flying;
        g.pos = Vec2::new(60.0, 30.0);
        g.vel = Vec2::new(10.0, -30.0);
        for _ in 0..120 {
            g.step(1.0 / 60.0);
            if g.phase == Phase::Landed {
                break;
            }
        }
        assert!(g.crashed, "a 30 m/s drop should crash");
        assert!(g.events.iter().any(|e| matches!(e, Event::Crashed { .. })));
        assert_eq!(g.phase, Phase::Landed);
    }

    #[test]
    fn world_is_populated_at_all_heights() {
        let mut g = Game::new(PILOTS[0], 5);
        let opening: Vec<_> = g.entities.iter().take(10).cloned().collect();
        // Fly a typical arc so the spawner runs ahead of the dog.
        g.phase = Phase::Flying;
        g.pos = Vec2::new(LIP.x, LIP.y + 1.0);
        g.vel = Vec2::new(30.0, 18.0);
        for _ in 0..(6 * 60) {
            g.step(1.0 / 60.0);
        }
        let n = g.entities.len();
        assert!(n >= 20, "only {n} entities after six seconds of flight");
        let low = g.entities.iter().filter(|e| e.pos.y < 15.0).count();
        let mid = g.entities.iter().filter(|e| e.pos.y >= 15.0 && e.pos.y < 60.0).count();
        assert!(low >= 3, "low band has {low}");
        assert!(mid >= 3, "mid band has {mid}");
        let g = Game { entities: opening, ..g };
        // The opening chain is always there and sits on the arc, off the ground.
        // Ten airborne pickups come before the trampoline on the ground.
        let opening: Vec<_> = g.entities.iter().take(10).collect();
        assert!(opening.len() >= 10, "opening has {}", opening.len());
        assert!(opening.iter().all(|e| e.pos.y > 6.0), "opening pickups must be on the arc, not the ground");
    }

    #[test]
    fn milestones_fire_in_order_when_climbing() {
        let mut g = Game::new(Pilot { drag: 0.0, ..PILOTS[0] }, 1);
        g.entities.clear();
        g.phase = Phase::Flying;
        g.pos = Vec2::new(10.0, 1.0);
        g.vel = Vec2::new(10.0, 200.0);
        let mut seen = Vec::new();
        for _ in 0..2000 {
            g.entities.clear();
            g.step(1.0 / 120.0);
            for e in g.events.drain(..) {
                if let Event::Milestone(i) = e {
                    seen.push(i);
                }
            }
            if g.phase == Phase::Landed {
                break;
            }
        }
        assert_eq!(seen, vec![0, 1, 2, 3]);
    }

    #[test]
    fn storm_clouds_get_denser_with_distance() {
        let mut g = Game::new(PILOTS[0], 9);
        g.entities.clear();
        g.phase = Phase::Flying;
        g.pos = Vec2::new(600.0, 30.0);
        g.vel = Vec2::new(40.0, 0.0);
        g.step(1.0 / 120.0);
        let far = g.entities.iter().filter(|e| e.kind == EntityKind::Stormcloud).count();
        let near = Game::new(PILOTS[0], 9).entities.iter().filter(|e| e.kind == EntityKind::Stormcloud).count();
        assert!(far > near, "far {far} near {near}");
        // A storm cloud costs speed.
        let mut h = Game::new(PILOTS[0], 1);
        h.vel = Vec2::new(30.0, 10.0);
        h.apply_hit(EntityKind::Stormcloud);
        assert!(h.vel.x < 30.0);
    }

    #[test]
    fn presents_stay_clear_of_storm_clouds() {
        let mut g = Game::new(PILOTS[0], 4);
        g.phase = Phase::Flying;
        g.pos = Vec2::new(300.0, 25.0);
        g.vel = Vec2::new(30.0, 0.0);
        for _ in 0..60 {
            g.entities.clear();
            g.pos.x += 20.0;
            g.spawn_ahead();
        }
        let storms: Vec<Vec2> = g.entities.iter().filter(|e| e.kind == EntityKind::Stormcloud).map(|e| e.pos).collect();
        for e in g.entities.iter().filter(|e| e.kind != EntityKind::Stormcloud) {
            for sp in &storms {
                assert!((e.pos - *sp).length() >= 6.0, "present at {:?} inside storm at {:?}", e.pos, sp);
            }
        }
    }

    #[test]
    fn reset_keeps_best() {
        let mut g = fly(1.0, 0.5);
        let d = g.distance();
        g.reset(); // the results card restart button
        assert_eq!(g.best_distance, d);
        assert!(matches!(g.phase, Phase::AimPower { .. }));
        assert_eq!(g.distance(), 0.0);
    }

    #[test]
    fn deterministic_for_same_seed() {
        let a = fly(0.8, 0.4);
        let b = fly(0.8, 0.4);
        assert_eq!(a.distance(), b.distance());
        assert_eq!(a.presents, b.presents);
    }
}
