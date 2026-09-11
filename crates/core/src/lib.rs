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
/// Presents are the thing to chase, so their reach is wider still: a present
/// that brushes an ear counts.
pub const PRESENT_REACH: f32 = 2.8;
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

/// Where the ski-jump lip is: the pilot leaves the ramp here. The take-off
/// table pokes just past the front corner of the building.
pub const LIP: Vec2 = Vec2::new(0.0, 14.0);
/// Where the sled waits: the start gate at the top of the inrun tower,
/// which stands on the back of the factory roof.
pub const RAMP_TOP: Vec2 = Vec2::new(-36.0, 25.0);
/// Height of the factory roof the ski jump stands on, meters.
pub const ROOF_Y: f32 = 12.0;
/// Where the straight inrun ends and the curved transition into the
/// take-off begins.
pub const KICKER_X: f32 = -13.0;
/// Height of the inrun at `KICKER_X`.
pub const KICKER_Y: f32 = 15.0;
/// Seconds the slide down the ramp takes at full power.
pub const SLIDE_TIME: f32 = 1.7;
/// Upward angle of the take-off table at the lip, degrees.
pub const LIP_ANGLE_DEG: f32 = 40.0;

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
/// Missing the jump: the sled topples off the roof edge at this crawl and
/// the dog tumbles down the side of the building; no flapping on the way.
pub const MISSED_JUMP_SPEED: f32 = 0.06;
/// After the lip the sled coasts along the deck for this much extra ramp
/// progress; a press in the coast still jumps, but later is worse, and the
/// end of the deck is the miss. This is what makes "too late" possible.
pub const LIP_COAST: f32 = 0.10;
/// How far along the deck the sled coasts past the lip, meters.
pub const LIP_DECK: f32 = 0.9;

/// Upward kick from one paw wiggle, m/s.
pub const FLAP_IMPULSE: f32 = 20.0;
/// Forward kick from one paw wiggle, m/s.
pub const FLAP_FORWARD: f32 = 4.0;
/// Stamina cost of one wiggle. Stamina is 0..1.
pub const FLAP_COST: f32 = 0.08;
/// Stamina regained per second in the air.
pub const FLAP_REGEN: f32 = 0.04;
/// Stamina regained per present.
pub const FLAP_PRESENT_BONUS: f32 = 0.2;
/// Minimum gap between wiggles, so holding the key isn't a jetpack.
pub const FLAP_COOLDOWN: f32 = 0.16;
/// A wiggle never pushes vertical speed above this (at lift 1).
pub const FLAP_MAX_VY: f32 = 26.0;
/// Wiggles work like a wing: their lift grows with airspeed. Lift is 1 at
/// this speed, `FLAP_LIFT_MIN` when hovering and `FLAP_LIFT_MAX` at full
/// tilt; the kick, its cap and how fast wiggles may repeat all follow it.
pub const FLAP_SPEED_REF: f32 = 30.0;
pub const FLAP_LIFT_MIN: f32 = 0.6;
pub const FLAP_LIFT_MAX: f32 = 2.2;
/// Every this many wiggles in one run, stamina refills: the wiggle easter egg.
pub const WIGGLE_STREAK: u32 = 25;

/// The meteor: a long fall with no wiggling. From this fall speed the dog
/// starts curling into a ball, and at `METEOR_FULL` it is a rolling
/// fireball. A dive sheds most of its vertical drag (`DIVE_DRAG` is what is
/// left at a full dive) and gravity pulls `DIVE_PULL` harder, so a fall
/// from space keeps gaining speed all the way down and hits harder than
/// any flight can.
pub const METEOR_FROM: f32 = 28.0;
pub const METEOR_FULL: f32 = 55.0;
/// Seconds of uninterrupted free fall (falling, no wiggle) before the
/// meteor can begin; it then builds in over `METEOR_RAMP` seconds more.
pub const METEOR_AFTER: f32 = 5.0;
pub const METEOR_RAMP: f32 = 1.0;
pub const DIVE_DRAG: f32 = 0.12;
pub const DIVE_PULL: f32 = 0.5;

/// Altitude milestones that fire [`Event::Milestone`].
pub const MILESTONES: [(f32, &str); 4] = [
    (60.0, "HIGH FLYER!"),
    (160.0, "ABOVE THE CLOUDS!"),
    (400.0, "SPACE!"),
    (700.0, "ORBIT!"),
];
/// Altitude at which the sky is fully space.
pub const SPACE_ALTITUDE: f32 = 400.0;

/// The stomp: a press while falling close to the snow arms a tucked landing
/// for this many seconds. An armed landing bounces harder, keeps its pace,
/// and survives a harder impact.
pub const STOMP_WINDOW: f32 = 0.22;
/// How far above the snow a press still counts as a stomp, meters.
pub const STOMP_REACH: f32 = 7.0;
/// Extra restitution from a stomped landing.
pub const STOMP_BONUS: f32 = 0.35;
/// Crash threshold multiplier while a stomp is armed.
pub const STOMP_CRASH_SCALE: f32 = 1.4;

/// Ground surfaces repeat in segments this long, meters.
pub const SURFACE_SEGMENT: f32 = 45.0;
/// Frozen lakes and snow drifts only appear past this distance.
pub const SURFACE_FROM: f32 = 150.0;

/// The jet stream: a river of tailwind between these altitudes, meters.
pub const JET_LO: f32 = 110.0;
pub const JET_HI: f32 = 170.0;
/// Forward push at the centre of the jet stream, m/s per second.
pub const JET_PUSH: f32 = 9.0;
/// Storm clouds never spawn above this altitude: the top of the sky is calm.
pub const STORM_CEILING: f32 = 260.0;

/// What the ground is made of at a given distance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    /// Plain snow: normal bounce and roll-out.
    Snow,
    /// A frozen lake: a rolling pilot keeps almost all speed and slides far.
    Ice,
    /// A deep drift: soft, swallows bounces, forgives a hard landing.
    Drift,
}

/// Integer hash to 0..1, deterministic and rng-free.
fn hash01(i: i32, salt: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

/// The surface under distance `x`: segments of [`SURFACE_SEGMENT`] meters,
/// hashed, so every run agrees on where the lakes are.
pub fn surface_at(x: f32) -> Surface {
    if x < SURFACE_FROM {
        return Surface::Snow;
    }
    let seg = (x / SURFACE_SEGMENT).floor() as i32;
    let r = hash01(seg, 7);
    if r < 0.18 {
        Surface::Ice
    } else if r < 0.28 {
        Surface::Drift
    } else {
        Surface::Snow
    }
}

/// Strength of the jet stream at an altitude: a smooth bump, 1 at the
/// centre of the band, 0 outside it.
pub fn jet_strength(alt: f32) -> f32 {
    let k = ((alt - JET_LO) / (JET_HI - JET_LO)).clamp(0.0, 1.0);
    let bump = (k * std::f32::consts::PI).sin();
    bump * bump
}

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
    /// The pilot tucked for a stomp just above the snow.
    StompArmed,
    Hit(EntityKind),
    /// A bounce off the ground; `stomped` when the landing was armed.
    GroundBounce { speed: f32, stomped: bool },
    /// Hit the ground so hard the run ends in a hole.
    Crashed { speed: f32 },
    /// The fall became a meteor: the pilot has curled into a rolling ball.
    Meteor,
    /// The jump off the ramp; quality 0..1, 1 for a press right at the lip,
    /// 0 for a missed jump. `late` when the press came after the lip.
    Jumped { quality: f32, late: bool },
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
    /// Seconds the stomp stays armed; 0 when not tucked.
    pub stomp: f32,
    /// The push power this run locked, 0..1; for the report card.
    pub push: f32,
    /// Jump quality earned this run, 0..1.
    pub jump_q: f32,
    /// Ramp progress at the jump press (1 is the lip, past 1 the coast);
    /// `1 + LIP_COAST` when the run coasted off unpressed.
    pub jump_t: f32,
    /// Total wiggles this run, for stats.
    pub flaps: u32,
    pub entities: Vec<Entity>,
    pub presents: u32,
    pub max_height: f32,
    pub flight_time: f32,
    pub best_distance: f32,
    /// True when the run ended head-first in the snow.
    pub crashed: bool,
    /// The jump was missed: the dog is falling off the roof and cannot flap.
    pub botched: bool,
    /// The meteor event has fired for this fall.
    was_meteor: bool,
    /// Seconds spent falling without a wiggle; the meteor clock.
    pub fall_time: f32,
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

/// Point on the ski jump for `t` in 0..1, like a real inrun: a straight
/// slope down from the start gate on the tower, a curved transition that
/// dips to just above the roof, and a short take-off table that throws the
/// sled up off the front corner of the building.
pub fn ramp_point(t: f32) -> Vec2 {
    const STRAIGHT: f32 = 0.66;
    let dir = {
        let d = Vec2::new(KICKER_X - RAMP_TOP.x, KICKER_Y - RAMP_TOP.y);
        let l = d.length();
        Vec2::new(d.x / l, d.y / l)
    };
    if t <= STRAIGHT {
        let k = t / STRAIGHT;
        Vec2::new(RAMP_TOP.x + (KICKER_X - RAMP_TOP.x) * k, RAMP_TOP.y + (KICKER_Y - RAMP_TOP.y) * k)
    } else {
        // Cubic transition: leaves along the inrun slope, bottoms out over
        // the roof, and lifts into the take-off at `LIP_ANGLE_DEG`.
        let k = (t - STRAIGHT) / (1.0 - STRAIGHT);
        let p0 = Vec2::new(KICKER_X, KICKER_Y);
        let p1 = Vec2::new(KICKER_X + dir.x * 4.5, KICKER_Y + dir.y * 4.5);
        let a = LIP_ANGLE_DEG.to_radians();
        let p2 = Vec2::new(LIP.x - a.cos() * 3.0, LIP.y - a.sin() * 3.0);
        let p3 = LIP;
        let u = 1.0 - k;
        Vec2::new(
            u * u * u * p0.x + 3.0 * u * u * k * p1.x + 3.0 * u * k * k * p2.x + k * k * k * p3.x,
            u * u * u * p0.y + 3.0 * u * u * k * p1.y + 3.0 * u * k * k * p2.y + k * k * k * p3.y,
        )
    }
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
            stomp: 0.0,
            push: 0.0,
            jump_q: 0.0,
            jump_t: 0.0,
            flaps: 0,
            entities: Vec::new(),
            presents: 0,
            max_height: 0.0,
            flight_time: 0.0,
            best_distance: 0.0,
            crashed: false,
            botched: false,
            was_meteor: false,
            fall_time: 0.0,
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

    /// Jump quality for a press at ramp progress `t`: nothing before the
    /// window, rising to 1 at the lip, then falling back to 0 across the
    /// coast so a late press is as bad as an early one.
    pub fn jump_quality(t: f32) -> f32 {
        if t > 1.0 {
            return (1.0 - (t - 1.0) / LIP_COAST).clamp(0.0, 1.0);
        }
        let k = ((t - JUMP_WINDOW_START) / (1.0 - JUMP_WINDOW_START)).clamp(0.0, 1.0);
        k * k * (3.0 - 2.0 * k)
    }

    /// Where the sled is at ramp progress `t`, including the coast past the lip.
    pub fn slide_point(t: f32) -> Vec2 {
        if t <= 1.0 {
            ramp_point(Self::slide_ease(t))
        } else {
            let d = Self::ramp_dir(1.0);
            let k = ((t - 1.0) / LIP_COAST).clamp(0.0, 1.0) * LIP_DECK;
            Vec2::new(LIP.x + d.x * k, LIP.y + d.y * k)
        }
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

    /// Launch velocity for a push `power` and jump `quality`: full ramp speed
    /// along the lip tangent plus the jump boost; a miss drops off slowly.
    fn launch_velocity(pilot: &Pilot, power: f32, quality: f32) -> Vec2 {
        let dir = Self::ramp_dir(1.0);
        let launch = pilot.launch_speed * (0.25 + 0.75 * power);
        let speed = if quality > 0.0 { launch } else { launch * MISSED_JUMP_SPEED };
        let ja = JUMP_ANGLE_DEG.to_radians();
        let boost = JUMP_BOOST * quality;
        Vec2::new(dir.x * speed + ja.cos() * boost, dir.y * speed + ja.sin() * boost)
    }

    /// Leave the deck from wherever the sled is (the lip, or somewhere along
    /// the coast) with the velocity the press earned.
    fn jump(&mut self, power: f32, quality: f32) {
        self.vel = Self::launch_velocity(&self.pilot, power, quality);
        // A missed jump is a fall, not a flight: the run ends under the roof edge.
        self.botched = quality <= 0.0;
        self.phase = Phase::Flying;
        self.events.push(Event::Launched);
    }

    pub fn action(&mut self) {
        match self.phase {
            Phase::AimPower { meter } => {
                let power = Self::meter_value(meter);
                self.push = power;
                self.phase = Phase::Sliding { power, t: 0.0, jump: None };
                self.events.push(Event::Pushed);
            }
            Phase::Sliding { power, t, jump: None } => {
                // One press per ride: as close to the lip as possible. Before
                // the lip the press is remembered and fires at the lip; in the
                // coast it fires right now, from wherever the sled got to.
                let q = Self::jump_quality(t).max(0.05);
                self.jump_q = q;
                self.jump_t = t;
                if t > 1.0 {
                    self.events.push(Event::Jumped { quality: q, late: true });
                    self.jump(power, q);
                } else {
                    self.phase = Phase::Sliding { power, t, jump: Some(q) };
                    self.events.push(Event::Jumped { quality: q, late: false });
                }
            }
            Phase::Sliding { .. } => {}
            Phase::Flying => self.flap(),
            // Landed: the results card owns the restart button.
            Phase::Landed => {}
        }
    }

    /// Put the flying pilot somewhere with a velocity and an empty sky
    /// ahead; for screenshots and tests, not play.
    pub fn dev_place(&mut self, pos: Vec2, vel: Vec2) {
        self.phase = Phase::Flying;
        self.pos = pos;
        self.vel = vel;
        self.entities.clear();
        self.spawned_to = pos.x + 600.0;
    }

    /// How much of a meteor the pilot is right now: 0 in normal flight, 1
    /// when falling at `METEOR_FULL` or faster, curled into a ball.
    pub fn meteor(&self) -> f32 {
        if self.phase != Phase::Flying || self.vel.y >= 0.0 || self.frozen > 0.0 || self.fall_time < METEOR_AFTER {
            return 0.0;
        }
        let k = ((-self.vel.y - METEOR_FROM) / (METEOR_FULL - METEOR_FROM)).clamp(0.0, 1.0);
        let clock = ((self.fall_time - METEOR_AFTER) / METEOR_RAMP).clamp(0.0, 1.0);
        k * k * (3.0 - 2.0 * k) * clock * clock * (3.0 - 2.0 * clock)
    }

    /// How much a wiggle does right now: lift grows with airspeed, from
    /// `FLAP_LIFT_MIN` near a hover to `FLAP_LIFT_MAX` at speed.
    pub fn flap_lift(&self) -> f32 {
        let speed = self.vel.length();
        (speed / FLAP_SPEED_REF).clamp(FLAP_LIFT_MIN, FLAP_LIFT_MAX)
    }

    /// Wiggle the paws for a bit of lift. Costs stamina; ignored on cooldown,
    /// when exhausted, or while rolling on the ground. Faster flight makes
    /// each wiggle lift more and lets them come quicker.
    pub fn flap(&mut self) {
        if self.phase != Phase::Flying || self.frozen > 0.0 || self.botched {
            return;
        }
        // Falling close to the snow: the press is a tuck, not a flap. It
        // costs nothing and arms a springier landing.
        if self.vel.y < 0.0 && self.pos.y < PILOT_RADIUS + STOMP_REACH && self.pos.y > PILOT_RADIUS + 0.01 {
            self.stomp = STOMP_WINDOW;
            self.events.push(Event::StompArmed);
            return;
        }
        if self.flap_cooldown > 0.0
            || self.stamina < FLAP_COST
            || (self.pos.y <= PILOT_RADIUS + 0.01 && self.vel.y <= 0.0)
        {
            return;
        }
        let p = self.pilot.flap_power * self.flap_lift();
        let boosted = self.vel.y + FLAP_IMPULSE * p;
        self.vel.y = boosted.min(self.vel.y.max(FLAP_MAX_VY * self.flap_lift()));
        self.vel.x += FLAP_FORWARD * p;
        self.stamina -= FLAP_COST;
        // Faster air, quicker wiggles.
        self.flap_cooldown = FLAP_COOLDOWN / self.flap_lift().max(1.0);
        // A wiggle is not free fall: the meteor clock starts over.
        self.fall_time = 0.0;
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
                if jump.is_some() && t2 >= 1.0 {
                    // A press before the lip fires at the lip.
                    self.pos = Vec2::new(LIP.x, LIP.y + PILOT_RADIUS);
                    self.jump(power, jump.unwrap_or(0.0));
                } else if t2 >= 1.0 + LIP_COAST {
                    // Coasted off the end of the deck without a press: a miss.
                    self.jump_q = 0.0;
                    self.jump_t = 1.0 + LIP_COAST;
                    self.events.push(Event::Jumped { quality: 0.0, late: false });
                    self.jump(power, 0.0);
                } else {
                    let p = Self::slide_point(t2);
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
        self.stomp = (self.stomp - dt).max(0.0);
        if self.frozen > 0.0 {
            // A block of ice flies badly: heavy drag, no lift of any kind.
            self.frozen = (self.frozen - dt).max(0.0);
            self.vel.x *= (1.0 - 0.45 * dt).max(0.0);
        }
        self.stamina = (self.stamina + FLAP_REGEN * dt).min(1.0);

        // The meteor clock: free fall is falling with no wiggle.
        if self.vel.y < 0.0 && self.rocket <= 0.0 {
            self.fall_time += dt;
        } else {
            self.fall_time = 0.0;
        }
        // Forces. A dive pulls harder the faster it falls: the meteor.
        let dive = self.meteor();
        let g_scale = if self.glide > 0.0 { GLIDE_GRAVITY_SCALE } else { 1.0 };
        self.vel.y -= GRAVITY * g_scale * (1.0 + DIVE_PULL * dive) * dt;
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
        // A ball falls clean: most of the vertical drag goes with the dive.
        let drag_y = drag * (1.0 - (1.0 - DIVE_DRAG) * dive);
        self.vel.x *= (1.0 - drag * dt).max(0.0);
        self.vel.y *= (1.0 - drag_y * dt).max(0.0);
        let m = self.meteor();
        if m >= 0.5 && !self.was_meteor {
            self.was_meteor = true;
            self.events.push(Event::Meteor);
        } else if m <= 0.0 {
            self.was_meteor = false;
        }
        // The jet stream: a tailwind river high up, felt as a steady pull.
        self.vel.x += JET_PUSH * jet_strength(self.altitude()) * dt;

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
            let surface = surface_at(self.pos.x);
            if self.vel.y < 0.0 {
                let impact = -self.vel.y;
                let soft = self.entities.iter().any(|e| e.alive && e.kind == EntityKind::Trampoline && (e.pos.x - self.pos.x).abs() < 3.5);
                let armed = self.stomp > 0.0;
                let mut crash_at = CRASH_SPEED;
                if armed {
                    crash_at *= STOMP_CRASH_SCALE;
                }
                if surface == Surface::Drift {
                    crash_at *= 1.6;
                }
                if impact > crash_at && !soft {
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
                let mut restitution = self.pilot.bounce;
                if armed {
                    restitution = (restitution + STOMP_BONUS).min(0.95);
                }
                if surface == Surface::Drift {
                    restitution *= 0.4;
                }
                self.vel.y = impact * restitution;
                let grip = match (armed, surface) {
                    (true, _) => 1.0,
                    (false, Surface::Ice) => 0.99,
                    (false, _) => self.pilot.ground_grip,
                };
                self.vel.x *= grip;
                self.stomp = 0.0;
                if self.vel.y < 2.0 {
                    self.vel.y = 0.0;
                } else {
                    self.events.push(Event::GroundBounce { speed: impact, stomped: armed });
                }
            }
            if self.vel.y == 0.0 {
                let friction = if surface == Surface::Ice { 0.15 } else { 1.2 };
                self.vel.x *= (1.0 - friction * dt).max(0.0);
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
            let reach = match e.kind {
                EntityKind::Stormcloud => 1.0,
                EntityKind::Present => PRESENT_REACH,
                _ => PICKUP_REACH,
            };
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
        // A rocket above the arc's peak: the first flight's big moment.
        let peak = at(landing * 0.5);
        push(EntityKind::Firework, Vec2::new(peak.x, peak.y + 3.5));
        push(EntityKind::Trampoline, Vec2::new(landing + 3.0, EntityKind::Trampoline.radius() * 0.6));
    }

    /// The arc a typical launch flies: sweet-spot push, perfect jump.
    fn median_arc(&self) -> Vec<Vec2> {
        predict_arc(self.pilot, 0.85, 1.0)
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
            } else if roll < 0.84 {
                // Fireworks are the crowd-pleaser: the biggest share after presents.
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
            // Every third rocket comes as a fan of three, climbing to the right.
            if kind == EntityKind::Firework && self.rng.unit() < 0.35 {
                for k in 1..3 {
                    let kf = k as f32;
                    self.entities.push(Entity { kind, pos: Vec2::new(x + kf * 3.2, y + kf * 2.4), alive: true, dead_for: 0.0 });
                }
            }
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
                // Half the clouds sit on the arc the dog is flying (the dodge);
                // the rest are scattered through the sky so no altitude is
                // ever the one storm band.
                let on_path = self.phase == Phase::Flying && self.rng.unit() < 0.5;
                let y = if on_path {
                    self.projected_y(x + 2.0).clamp(8.0, STORM_CEILING) + self.rng.range(-8.0, 8.0)
                } else {
                    let u = self.rng.unit();
                    8.0 + (STORM_CEILING - 8.0) * u * u
                };
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

/// Samples of the flight path for a push `power` and jump `quality` in an
/// empty world, one sample per 2 m of travel, ending where the pilot lands.
/// This is what the aim arc draws and what the opening chain is laid on.
pub fn predict_arc(pilot: Pilot, power: f32, quality: f32) -> Vec<Vec2> {
    let mut g = Game {
        pilot,
        phase: Phase::Flying,
        pos: Vec2::new(LIP.x, LIP.y + PILOT_RADIUS),
        vel: Game::launch_velocity(&pilot, power, quality),
        glide: 0.0,
        frozen: 0.0,
        rocket: 0.0,
        stamina: 1.0,
        flap_cooldown: 0.0,
        stomp: 0.0,
        push: 0.0,
        jump_q: 0.0,
        jump_t: 0.0,
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
            botched: false,
            was_meteor: false,
            fall_time: 0.0,
    };
    let mut out = vec![g.pos];
    let mut steps = 0;
    while g.phase == Phase::Flying && steps < 3000 {
        g.step(1.0 / 60.0);
        steps += 1;
        if g.pos.x - out.last().unwrap().x >= 2.0 {
            out.push(g.pos);
        }
        // Stop at the first touchdown: the arc ends where the dog lands.
        if g.pos.y <= PILOT_RADIUS + 1e-3 && g.vel.y <= 0.0 {
            break;
        }
    }
    out.push(Vec2::new(g.pos.x, PILOT_RADIUS));
    out
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
        // The inrun drops from the tower, bottoms out above the roof, then
        // rises into the lip.
        assert!(ramp_point(0.9).y < LIP.y);
        assert!(ramp_point(0.4).y < RAMP_TOP.y && ramp_point(0.4).y > LIP.y);
        for i in 0..=100 {
            assert!(ramp_point(i as f32 / 100.0).y > ROOF_Y + 0.5, "ramp dips into the roof at {}", i);
        }
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
    fn a_fall_from_space_becomes_a_meteor_and_hits_harder_than_a_flight() {
        let mut g = Game::new(PILOTS[0], 1);
        // An empty sky: a pickup on the way down would interrupt the fall.
        g.dev_place(Vec2::new(200.0, 350.0), Vec2::new(10.0, 0.0));
        let mut peak = 0.0f32;
        let mut t = 0.0f32;
        let mut meteor_at = None;
        for _ in 0..4000 {
            g.step(1.0 / 120.0);
            t += 1.0 / 120.0;
            peak = peak.max(-g.vel.y);
            if meteor_at.is_none() && g.events.contains(&Event::Meteor) {
                meteor_at = Some(t);
            }
            if g.phase == Phase::Landed {
                break;
            }
        }
        let meteor_at = meteor_at.expect("the fall became a meteor");
        assert!(meteor_at >= METEOR_AFTER, "meteor at {meteor_at}s");
        let impact = g.events.iter().find_map(|e| if let Event::Crashed { speed } = e { Some(*speed) } else { None }).expect("crashed");
        // Well past the old drag ceiling of about 45 m/s.
        assert!(impact > 70.0, "impact {} peak {}", impact, peak);
    }

    #[test]
    fn a_wiggle_lifts_more_and_repeats_faster_at_speed() {
        let mut slow = Game::new(PILOTS[0], 1);
        slow.entities.clear();
        slow.phase = Phase::Flying;
        slow.pos = Vec2::new(50.0, 40.0);
        slow.vel = Vec2::new(12.0, 0.0);
        let mut fast = slow.clone();
        fast.vel = Vec2::new(70.0, 0.0);
        slow.flap();
        fast.flap();
        assert!(fast.vel.y > slow.vel.y * 1.5, "fast {} slow {}", fast.vel.y, slow.vel.y);
        assert!(fast.flap_cooldown < slow.flap_cooldown);
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
    fn a_press_after_the_lip_is_late_and_the_deck_end_is_a_miss() {
        let just_late = fly(1.0, 1.02);
        let very_late = fly(1.0, 1.08);
        assert!(just_late.events.iter().any(|e| matches!(e, Event::Jumped { late: true, .. })));
        assert!(just_late.distance() > very_late.distance(), "{} vs {}", just_late.distance(), very_late.distance());
        let none = fly(1.0, 2.0);
        assert!(none.events.contains(&Event::Jumped { quality: 0.0, late: false }));
        assert!(very_late.distance() > none.distance());
        assert!(Game::jump_quality(1.0 + LIP_COAST) == 0.0);
        assert!(Game::jump_quality(1.05) > 0.4 && Game::jump_quality(1.05) < 0.6);
        // The sled leaves from the end of the deck, not the lip, on a miss.
        assert!(Game::slide_point(1.0 + LIP_COAST).x > LIP.x + 0.5);
    }

    #[test]
    fn predicted_arc_lands_where_a_real_flight_does() {
        let path = predict_arc(PILOTS[0], 1.0, 1.0);
        let real = fly(1.0, 0.999).distance();
        let predicted = path.last().unwrap().x;
        assert!(path.len() > 10);
        assert!(path[..path.len() - 1].windows(2).all(|w| w[1].x > w[0].x));
        // The prediction ignores bounces, so it lands at the first touchdown.
        assert!(predicted > 30.0 && predicted <= real + 1.0, "predicted {predicted} real {real}");
        assert!(predict_arc(PILOTS[0], 0.3, 1.0).last().unwrap().x < predicted);
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
    fn ice_keeps_more_speed_than_snow_on_a_roll_out() {
        // Find one ice segment and one snow segment past the surface start.
        let mut ice = None;
        let mut snow = None;
        let mut x = SURFACE_FROM + SURFACE_SEGMENT / 2.0;
        while (ice.is_none() || snow.is_none()) && x < 20_000.0 {
            match surface_at(x) {
                Surface::Ice if ice.is_none() => ice = Some(x),
                Surface::Snow if snow.is_none() => snow = Some(x),
                _ => {}
            }
            x += SURFACE_SEGMENT;
        }
        let (ice, snow) = (ice.expect("an ice segment"), snow.expect("a snow segment"));
        let roll = |x0: f32| {
            let mut g = Game::new(PILOTS[0], 2);
            g.entities.clear();
            g.phase = Phase::Flying;
            g.pos = Vec2::new(x0, PILOT_RADIUS);
            g.vel = Vec2::new(12.0, 0.0);
            for _ in 0..60 {
                g.entities.clear();
                g.step(1.0 / 120.0);
            }
            g.vel.x
        };
        assert!(roll(ice) > roll(snow), "ice {} vs snow {}", roll(ice), roll(snow));
        assert_eq!(surface_at(10.0), Surface::Snow);
    }

    #[test]
    fn a_stomp_bounces_higher_and_keeps_pace() {
        let land = |stomp: bool| {
            let mut g = Game::new(PILOTS[0], 2);
            g.entities.clear();
            g.phase = Phase::Flying;
            g.pos = Vec2::new(40.0, 2.6);
            g.vel = Vec2::new(20.0, -12.0);
            if stomp {
                g.flap();
                assert!(g.stomp > 0.0);
                assert!(g.events.contains(&Event::StompArmed));
            }
            for _ in 0..40 {
                g.entities.clear();
                g.step(1.0 / 120.0);
                if g.vel.y > 0.0 {
                    break;
                }
            }
            g
        };
        let plain = land(false);
        let stomped = land(true);
        assert!(stomped.vel.y > plain.vel.y, "stomped {} vs plain {}", stomped.vel.y, plain.vel.y);
        assert!(stomped.vel.x > plain.vel.x);
        assert!(stomped.events.iter().any(|e| matches!(e, Event::GroundBounce { stomped: true, .. })));
        assert_eq!(stomped.stamina, 1.0, "a stomp costs no stamina");
    }

    #[test]
    fn the_jet_stream_pulls_forward() {
        let cruise = |alt: f32| {
            let mut g = Game::new(Pilot { drag: 0.0, ..PILOTS[0] }, 2);
            g.entities.clear();
            g.phase = Phase::Flying;
            g.glide = 1.0e6; // hold altitude roughly: light gravity
            g.pos = Vec2::new(200.0, alt + PILOT_RADIUS);
            g.vel = Vec2::new(20.0, 0.0);
            for _ in 0..60 {
                g.entities.clear();
                g.pos.y = alt + PILOT_RADIUS;
                g.step(1.0 / 120.0);
            }
            g.vel.x
        };
        assert!(cruise(140.0) > cruise(40.0) + 3.0, "jet {} vs low {}", cruise(140.0), cruise(40.0));
        assert_eq!(jet_strength(40.0), 0.0);
        assert!(jet_strength(140.0) > 0.99);
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
