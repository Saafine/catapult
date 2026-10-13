//! macroquad runtime for the Christmas dog launch game.
//!
//! Everything platform-flavoured lives here: window, input, drawing, later
//! sound. The rules live in `hamsters-core`. Builds native and wasm from the
//! same source.

use hamsters_core::{predict_arc, EntityKind, Event, Game, Phase, LIP, PILOTS, PILOT_RADIUS, RAMP_TOP};
use macroquad::prelude::*;

mod anim;
mod city;
mod draw;
mod hud;
mod orders;
mod pack;
mod particles;
mod post;
mod props;
mod sfx;
mod sprites;
mod fxtex;
mod vfx;
mod ui;

use city::City;
use draw::lerp;
use pack::Pack;
use serde::{Deserialize, Serialize};
use particles::Particles;
use sfx::Sfx;
use sprites::Sprites;

/// Pixels per world meter at zoom 1.
pub const PPM: f32 = 30.0;
const FIXED_DT: f32 = 1.0 / 120.0;

fn window_conf() -> Conf {
    Conf { window_title: "Santa's Flight School".to_owned(), window_width: 1024, window_height: 576, high_dpi: true, sample_count: 4, platform: post::platform(), ..Default::default() }
}

/// Camera that frames the tower while aiming, follows the sled down the ramp,
/// then tracks the dog with look-ahead and zooms out with speed.
pub struct Cam {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub shake: Vec2,
    pub shake_amp: f32,
    /// The dog's velocity smoothed for framing; see `target_for`.
    pub lead: Vec2,
    /// Launch punch: 1 right at the lip, decays to 0. Zooms in briefly.
    pub punch: f32,
    /// 0..1 how close the ground is in time while falling; drives the
    /// punch-in before impact.
    pub near: f32,
    /// The PERFECT-jump beat: 1 at the lip, decays in game time. Punches in
    /// on the dog and centres it while the world slows down.
    pub beat: f32,
    /// Game-time clock for the shake noise, so shakes slow down in slow motion.
    pub shake_clock: f32,
    /// 0..1 how excited the bystanders are (set by the app each frame).
    pub cheer: f32,
}

/// Seconds the pre-launch look-back glides from the target flag to the tower.
pub const PREVIEW_TIME: f32 = 1.3;

impl Cam {
    /// Where the camera wants to be: (x, y, zoom, near). `preview` is the
    /// look-back before a launch: (target x, progress 0..1).
    /// `lead` is the dog's velocity as the camera sees it: smoothed over a
    /// few tenths of a second, so a wiggle kick hops the dog on screen
    /// instead of yanking the whole world (and every parallax layer) with it.
    fn target_for(g: &Game, preview: Option<(f32, f32)>, lead: Vec2) -> (f32, f32, f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        match g.phase {
            Phase::AimPower { .. } => {
                // Frame the whole ski jump: the start gate on the tower, the
                // inrun, the lip and a strip of ground: about 38 m of world
                // height, centred a little under the middle of the run.
                let zoom = (sh / 38.0 / PPM).clamp(0.4, 1.0);
                let _ = sw;
                let (tx, ty) = (RAMP_TOP.x * 0.5 + 1.0, (RAMP_TOP.y + 2.5) * 0.5);
                if let Some((flag_x, k)) = preview {
                    // The look-back: start on the flag, glide left along the
                    // pennants and settle on the tower as the needle wakes.
                    let e = 1.0 - (1.0 - k.clamp(0.0, 1.0)).powi(3);
                    let (px, py, pz) = (flag_x + 6.0, 6.0, 0.75);
                    return (px + (tx - px) * e, py + (ty - py) * e, pz + (zoom - pz) * e, 0.0);
                }
                (tx, ty, zoom, 0.0)
            }
            Phase::Landed => {
                // The quiet after landing: close on the dog, a strip of snow
                // under it, and nothing else moving. Tighter still on a crash,
                // so the sink into the hole fills the frame.
                let zoom = if g.crashed { 1.8 } else { 1.3 };
                let s = PPM * zoom;
                (g.pos.x + if g.crashed { 0.3 } else { 2.0 }, if g.crashed { 0.30 * sh / s } else { 0.42 * sh / s }, zoom, 0.0)
            }
            Phase::Sliding { t, .. } => {
                // Tight on the sled, and pushing in over the last stretch so
                // the lip arrives with tension: the dog grows in the frame as
                // the take-off comes up, with lead room ahead of it.
                let k = t.clamp(0.0, 1.0);
                let push = k * k * (3.0 - 2.0 * k);
                let zoom = 1.1 + 0.28 * push;
                let s = PPM * zoom;
                (g.pos.x + (0.14 - 0.03 * push) * sw / s, (0.30 * sh / s + 1.0).max(g.pos.y - (0.02 + 0.02 * push) * sh / s), zoom, 0.0)
            }
            _ => {
                let speed = lead.length();
                // Stay tight for the first second after the lip, then ease out
                // with speed and altitude.
                // Hold the close framing for half a second after the lip, then
                // ease out over the next second.
                let fresh = (1.0 - ((g.flight_time - 0.5) / 1.0).clamp(0.0, 1.0)).powi(2);
                // Wide for the arc: the faster and higher the flight, the more
                // of the parabola is on screen.
                let wide = (1.0 / (1.0 + speed / 60.0 + g.pos.y / 170.0)).clamp(0.62, 1.0);
                let mut zoom = wide + (1.15 - wide) * fresh;
                // Tight for the impact: in the last second before the snow the
                // camera punches in so the stomp press has something to read.
                let near = if g.phase == Phase::Flying && g.vel.y < 0.0 {
                    let h = (g.pos.y - PILOT_RADIUS).max(0.0);
                    let grav = if g.glide > 0.0 { hamsters_core::GRAVITY * hamsters_core::GLIDE_GRAVITY_SCALE } else { hamsters_core::GRAVITY };
                    let tg = (g.vel.y + (g.vel.y * g.vel.y + 2.0 * grav * h).sqrt()) / grav;
                    (1.0 - tg / 1.1).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                zoom += (1.15 - zoom) * near * near;
                let s = PPM * zoom;
                // The camera rises only half as fast as the dog, so the dog
                // visibly climbs towards the top of the frame at the apex and
                // sinks back on the way down; clamped so it never leaves the
                // safe band between the top HUD and the wiggle strip.
                let base_y = 0.36 * sh / s;
                // During the hold the dog sits about a third from the top;
                // afterwards the camera follows altitude loosely so the arc shows.
                let tight_y = g.pos.y + 0.05 * sh / s;
                // Vertical lead: while climbing the dog sits low in the frame
                // (48% from the top) so the sky ahead shows; while falling it
                // rides high (30%) so the ground ahead shows.
                // Kept under the banner slot (24% from the top) either way.
                let frac = 0.53 + (lead.y / 12.0).clamp(-1.0, 1.0) * 0.07;
                // Screen y grows downwards, so the camera centre sits below
                // the dog (smaller world y) to put the dog high in the frame.
                let loose_y = g.pos.y - (0.5 - frac) * sh / s;
                let follow_y = tight_y * fresh + loose_y * (1.0 - fresh);
                // Hard clamp: the dog's hat never enters the top HUD band.
                let min_y = g.pos.y + 2.6 - (0.5 * sh - 150.0) / s;
                let max_y = g.pos.y + (0.5 * sh - 150.0) / s;
                // Lead room: the dog flies at about a third from the left.
                let lookahead = if g.phase == Phase::Flying { (0.16 - 0.08 * near) * sw / s } else { 0.10 * sw / s };
                (g.pos.x + lookahead, base_y.max(follow_y).clamp(min_y, max_y), zoom, near)
            }
        }
    }

    fn follow(&mut self, g: &Game, dt: f32, preview: Option<(f32, f32)>) {
        // Smooth the velocity the framing reads: a flap is a step of 14 m/s
        // that would otherwise move the camera target by a tenth of the
        // screen in one frame.
        let kl = 1.0 - (-dt * 2.5).exp();
        self.lead += (vec2(g.vel.x, g.vel.y) - self.lead) * kl;
        let (tx, ty, tz, near) = Self::target_for(g, preview, self.lead);
        self.near = near;
        // After landing the pull-in is a slow breath, not a snap.
        // A crash snaps the camera onto the hole; a soft landing is a slow breath.
        let rate = if g.phase == Phase::Landed { if g.crashed { 9.0 } else { 1.8 } } else { 7.0 };
        let k = 1.0 - (-dt * rate).exp();
        // Feed-forward: a smoothed follow trails a moving target by
        // speed / rate, which let a fast dog run off the top of the frame;
        // aim ahead by that much so the composition holds at any speed.
        // Off the lip (before any wiggle) it reads the true velocity, since
        // the smoothed copy is still catching up with the launch; later the
        // smoothed one, so a wiggle hops the dog and not the world.
        let fresh = if g.phase == Phase::Flying { (1.0 - (g.flight_time - 0.8) / 0.8).clamp(0.0, 1.0) } else { 0.0 };
        let ffv = self.lead + (vec2(g.vel.x, g.vel.y) - self.lead) * fresh;
        let (tx, ty) = if matches!(g.phase, Phase::Flying | Phase::Sliding { .. }) {
            (tx + ffv.x / rate * 0.8, ty + ffv.y / rate.min(6.0) * 0.9)
        } else {
            (tx, ty)
        };
        // The PERFECT beat: a quick punch-in centred on the dog, released slowly.
        self.beat = (self.beat - dt * 0.9).max(0.0);
        let b = self.beat;
        let be = if b > 0.88 { (1.0 - b) / 0.12 } else { b / 0.88 };
        let be = be * be * (3.0 - 2.0 * be);
        // Framed a little low and ahead, so the head clears the banner slot.
        let zs = PPM * tz * (1.0 + 0.22 * be);
        let (fx, fy) = (g.pos.x + 0.06 * screen_width() / zs, g.pos.y + 0.09 * screen_height() / zs);
        let (tx, ty) = (tx + (fx - tx) * 0.8 * be, ty + (fy - ty) * 0.8 * be);
        let tz = tz * (1.0 + 0.22 * be);
        // The punch-in before impact closes faster than the cruise.
        let kz = if near > 0.0 || be > 0.0 || (g.phase == Phase::Landed && g.crashed) { 1.0 - (-dt * 10.0).exp() } else { k };
        // Vertical follow lags a little so each wiggle kick hops the dog on screen.
        let ky = 1.0 - (-dt * rate.min(6.0)).exp();
        let (k, ky) = if be > 0.0 { (k.max(1.0 - (-dt * 12.0).exp()), ky.max(1.0 - (-dt * 12.0).exp())) } else { (k, ky) };
        self.x += (tx - self.x) * k;
        self.y += (ty - self.y) * ky;
        // Hard clamp after smoothing: the dog's hat never enters the HUD band.
        if matches!(g.phase, Phase::Flying | Phase::Sliding { .. }) {
            let s = PPM * self.zoom;
            let min_y = g.pos.y + 2.8 - (0.5 * screen_height() - 130.0) / s;
            if self.y < min_y {
                self.y = min_y;
            }
        }
        self.punch = (self.punch - dt * 2.0).max(0.0);
        // Punch eases in and out rather than snapping.
        let env = (self.punch * std::f32::consts::PI).sin().max(0.0);
        let tz = tz * (1.0 + 0.10 * env);
        self.zoom += (tz - self.zoom) * kz;
        // Shake: a hard hit, then a fast exponential settle; smooth noise
        // on a game-time clock, so a slow-motion crash shakes in slow motion.
        self.shake_amp = (self.shake_amp - dt * (6.0 + 4.5 * self.shake_amp)).max(0.0);
        self.shake_clock += dt;
        let c = self.shake_clock;
        let nx = (c * 53.0).sin() * 0.5 + (c * 31.0 + 1.3).sin() * 0.3 + (c * 89.0 + 2.1).sin() * 0.2;
        let ny = (c * 47.0 + 0.7).sin() * 0.5 + (c * 29.0 + 2.9).sin() * 0.3 + (c * 97.0 + 0.4).sin() * 0.2;
        self.shake = vec2(nx, ny) * self.shake_amp;
    }

    pub fn to_screen(&self, wx: f32, wy: f32) -> Vec2 {
        let s = self.scale();
        vec2(screen_width() * 0.5 + (wx - self.x) * s + self.shake.x, screen_height() * 0.5 - (wy - self.y) * s + self.shake.y)
    }

    pub fn to_screen_parallax(&self, wx: f32, wy: f32, factor: f32) -> Vec2 {
        self.to_screen_parallax_xy(wx, wy, factor, factor)
    }

    pub fn to_screen_parallax_xy(&self, wx: f32, wy: f32, fx: f32, fy: f32) -> Vec2 {
        let s = self.scale();
        vec2(
            screen_width() * 0.5 + (wx - self.x * fx) * s + self.shake.x * fx,
            screen_height() * 0.5 - (wy - self.y * fy) * s + self.shake.y * fy,
        )
    }

    pub fn scale(&self) -> f32 {
        PPM * self.zoom
    }
}

/// Short-lived popup triggered by a core event.
pub struct Fx {
    pub pos: Vec2,
    pub age: f32,
    pub life: f32,
    pub color: Color,
    pub text: String,
    /// Screen-centred banner rather than a world popup.
    pub banner: bool,
}

/// The sled after it leaves the lip: a free-falling prop.
#[derive(Clone, Serialize, Deserialize)]
pub struct Score {
    pub distance: f32,
    pub pilot: usize,
    pub crashed: bool,
}

const SCORES_KEY: &str = "santas_flight_school_scores";
const BEST_PATH_KEY: &str = "santas_flight_school_best_path";

/// The flight path of the best run so far: the ghost the player races.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BestPath {
    pub pilot: usize,
    pub path: Vec<(f32, f32)>,
}

fn load_best_path() -> BestPath {
    let raw = quad_storage::STORAGE.lock().ok().and_then(|s| s.get(BEST_PATH_KEY));
    raw.and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_default()
}

fn save_best_path(best: &BestPath) {
    if let Ok(json) = serde_json::to_string(best) {
        if let Ok(mut s) = quad_storage::STORAGE.lock() {
            s.set(BEST_PATH_KEY, &json);
        }
    }
}

fn load_scores() -> Vec<Score> {
    let raw = quad_storage::STORAGE.lock().ok().and_then(|s| s.get(SCORES_KEY));
    raw.and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_default()
}

const TAUGHT_KEY: &str = "santas_flight_school_taught";
const MUSIC_KEY: &str = "santas_flight_school_music";

/// Whether the pack music plays; on until the player presses M.
#[derive(Serialize, Deserialize)]
struct MusicSetting {
    on: bool,
}

impl Default for MusicSetting {
    fn default() -> Self {
        MusicSetting { on: true }
    }
}
const DAILY_KEY: &str = "santas_flight_school_daily";
const RECENT_KEY: &str = "santas_flight_school_recent";

/// Rules the player has already used once; each is taught with one prompt
/// at the moment it matters, then never mentioned again.
pub const TEACH_PUSH: u8 = 1;
pub const TEACH_JUMP: u8 = 2;
pub const TEACH_FLAP: u8 = 4;
pub const TEACH_STOMP: u8 = 8;
/// How many saved flights before the stomp is worth teaching.
const TEACH_STOMP_AFTER: usize = 3;
/// How many recent flights the results card charts.
pub const RECENT_RUNS: usize = 8;

/// Today's course: one seeded flight per pilot per calendar day, so which
/// dog to fly this course is a real decision. Resets when the day changes.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Daily {
    pub day: i64,
    pub flown: [Option<f32>; 3],
}

/// Days since the epoch, from the platform clock (works in the browser).
fn today() -> i64 {
    (miniquad::date::now() / 86400.0).floor() as i64
}

fn load_json<T: for<'de> Deserialize<'de> + Default>(key: &str) -> T {
    let raw = quad_storage::STORAGE.lock().ok().and_then(|s| s.get(key));
    raw.and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_default()
}

fn save_json<T: Serialize>(key: &str, value: &T) {
    if let Ok(json) = serde_json::to_string(value) {
        if let Ok(mut s) = quad_storage::STORAGE.lock() {
            s.set(key, &json);
        }
    }
}

fn save_scores(scores: &[Score]) {
    if let Ok(json) = serde_json::to_string(scores) {
        if let Ok(mut s) = quad_storage::STORAGE.lock() {
            s.set(SCORES_KEY, &json);
        }
    }
}

/// A fireworks shell waiting to burst: where, in how long, what colour.
struct Shell {
    pos: Vec2,
    delay: f32,
    color: Color,
}

/// Real seconds the crash plays in slow motion before time returns to normal.
const CRASH_SLOW: f32 = 1.1;
/// Real seconds of the slow-motion beat after a PERFECT jump.
const BEAT_SLOW: f32 = 1.0;

struct LooseSled {
    pos: Vec2,
    vel: Vec2,
    rot: f32,
    spin: f32,
}

pub struct App {
    pub game: Game,
    pub cam: Cam,
    acc: f32,
    pub fx: Vec<Fx>,
    pub particles: Particles,
    pub pilot_idx: usize,
    pub flap_anim: f32,
    /// Timer for the powerless paw-slap when the wiggle bar is empty.
    pub limp_flap: f32,
    /// Extra speed echoes right after a reindeer or rocket kick.
    pub boost_echo: f32,
    /// One-frame scale punch on the dog for each present.
    pub pickup_pop: f32,
    /// Best distance before this run, for the "short of best" line.
    pub best_before: f32,
    /// Electrified silhouette timer after a zap.
    pub zap_flash: f32,
    /// Best distance per pilot this session, for the results ladder.
    pub bests: [f32; 3],
    /// Last 10 m mark passed, so the distance counter can pop on each.
    pub last_mark: i32,
    pub dist_pulse: f32,
    /// 0..1 how far the crashed dog has sunk into its hole.
    pub crash_sink: f32,
    /// How big the crash hole is: 1 for a splat, more for a meteor.
    crash_size: f32,
    /// Rotation of the meteor ball, radians; keeps spinning while falling.
    meteor_spin: f32,
    /// Real seconds of crash slow-motion left; time creeps back to normal as it runs out.
    crash_slow: f32,
    /// Real seconds of the PERFECT-jump slow-motion beat left.
    beat_slow: f32,
    /// Procedural secondary motion for the dog: springs and the cape chain.
    dog_anim: anim::DogAnim,
    /// This frame's time without the slow-motion beats: the camera operator
    /// keeps working at full speed while the world slows down.
    cam_dt: f32,
    /// Game seconds since the run ended (respects slow motion), for the hold and the card.
    pub landed_for: f32,
    /// Fireworks shells in the air, waiting to burst.
    shells: Vec<Shell>,
    /// Next crackle burst while riding a rocket.
    rocket_burst: f32,
    /// Next snow spurt while the crashed dog sinks in.
    sink_spurt: f32,
    /// Dev-only: jump automatically at the perfect moment after a P launch.
    pub auto_jump: bool,
    /// Best distances so far, longest first, with the pilot that flew them.
    pub scores: Vec<Score>,
    was_frozen: bool,
    pub squash: f32,
    /// Tuck pose timer after a stomp press, decays fast.
    pub tuck: f32,
    pub tumble: f32,
    pub trail: Vec<Vec2>,
    trail_timer: f32,
    emit_timer: f32,
    loose_sled: Option<LooseSled>,
    /// Banners wait their turn here; only one shows at a time.
    banner_queue: Vec<Fx>,
    /// Seconds of frozen simulation after the lip (hit-stop).
    hitstop: f32,
    /// White flash alpha after the lip.
    flash: f32,
    /// Red flash alpha after a storm hit.
    red_flash: f32,
    /// Present chain: count and time since the last one.
    combo: u32,
    combo_timer: f32,
    /// Extra stretch right after the lip.
    launch_stretch: f32,
    /// Sonic-boom ring already fired this flight.
    boomed: bool,
    /// (x, size) of snow craters left by bounces.
    craters: Vec<(f32, f32)>,
    /// Short-lived reaction face and its remaining time.
    reaction: Option<(draw::Face, f32)>,
    /// Dev slow-motion multiplier.
    time_scale: f32,
    /// Squash-stretch pulse on each wiggle press, decays fast.
    wiggle_pulse: f32,
    /// Present counter pulse on a catch.
    pub counter_pulse: f32,
    /// Predicted flight for the current power and jump: the aim arc.
    pub aim_path: Vec<Vec2>,
    /// Frames since the aim arc was last recomputed.
    aim_frame: u32,
    /// The jump value the aim arc was computed for, to notice a change.
    aim_jump: Option<f32>,
    /// This run's flight path, one sample per 2 m.
    run_path: Vec<Vec2>,
    /// The best run's path, drawn as a ghost with a flag at its end.
    pub best_path: BestPath,
    /// Seconds left of the pre-launch look-back at the target flag; 0 when off.
    pub preview: f32,
    /// Bitflags of rules already taught; see `TEACH_*`.
    pub taught: u8,
    /// Today's course bookkeeping.
    pub daily: Daily,
    /// This run flies today's seeded course for its pilot.
    pub daily_run: bool,
    /// The last few flights in order, newest last, for the trend strip.
    pub recent: Vec<Score>,
    /// Where the look-back starts: the target's x.
    preview_x: f32,
    /// Distance of the previous run, for picking the next rung to beat.
    pub last_distance: f32,
    /// Fired once when this run passes the target, so the HUD line flips.
    pub best_beaten: bool,
    /// Live scoreboard rank of the current distance and a pulse when it changes.
    pub last_rank: usize,
    pub rank_pulse: f32,
    /// Seconds until the next breath puff while the dog sits in the snow.
    breath_timer: f32,
    pub sfx: Sfx,
    pub font: Font,
    pub display: Font,
    pub sprites: Sprites,
    pub pack: Pack,
    /// Calgary's streets under the flight path, when the pack has them.
    pub city: Option<City>,
    /// The district the dog is flying over, for the welcome banners.
    district: Option<usize>,
    /// Santa's orders: the three active challenges, rank and cape.
    pub orders: orders::Orders,
}

impl App {
    fn new(font: Font, display: Font, sprites: Sprites, pack: Pack, sfx: Sfx, city: Option<City>) -> Self {
        let seed = (get_time() * 1e6) as u64 | 1;
        let mut game = Game::new(PILOTS[0], seed);
        let scores = load_scores();
        if let Some(top) = scores.first() {
            game.best_distance = top.distance;
        }
        let (x, y, zoom, _) = Cam::target_for(&game, None, Vec2::ZERO);
        let mut app = App {
            game,
            cam: Cam { x, y, zoom, shake: Vec2::ZERO, shake_amp: 0.0, lead: Vec2::ZERO, punch: 0.0, near: 0.0, beat: 0.0, shake_clock: 0.0, cheer: 0.0 },
            acc: 0.0,
            fx: Vec::new(),
            particles: Particles::default(),
            pilot_idx: 0,
            flap_anim: 0.0,
            limp_flap: 0.0,
            boost_echo: 0.0,
            pickup_pop: 0.0,
            best_before: 0.0,
            zap_flash: 0.0,
            bests: [0.0; 3],
            last_mark: 0,
            dist_pulse: 0.0,
            crash_sink: 0.0,
            crash_size: 1.0,
            meteor_spin: 0.0,
            crash_slow: 0.0,
            beat_slow: 0.0,
            dog_anim: anim::DogAnim::new(),
            cam_dt: 0.0,
            landed_for: 0.0,
            shells: Vec::new(),
            rocket_burst: 0.0,
            sink_spurt: 0.0,
            auto_jump: false,
            scores,
            was_frozen: false,
            squash: 0.0,
            tuck: 0.0,
            tumble: 0.0,
            trail: Vec::new(),
            trail_timer: 0.0,
            emit_timer: 0.0,
            loose_sled: None,
            banner_queue: Vec::new(),
            hitstop: 0.0,
            flash: 0.0,
            red_flash: 0.0,
            combo: 0,
            combo_timer: 0.0,
            launch_stretch: 0.0,
            boomed: false,
            craters: Vec::new(),
            reaction: None,
            time_scale: 1.0,
            wiggle_pulse: 0.0,
            counter_pulse: 0.0,
            aim_path: Vec::new(),
            aim_frame: 0,
            aim_jump: None,
            run_path: Vec::new(),
            best_path: load_best_path(),
            preview: 0.0,
            taught: load_json::<u8>(TAUGHT_KEY),
            daily: load_json::<Daily>(DAILY_KEY),
            daily_run: false,
            recent: load_json::<Vec<Score>>(RECENT_KEY),
            preview_x: 0.0,
            last_distance: 0.0,
            best_beaten: false,
            last_rank: 0,
            rank_pulse: 0.0,
            breath_timer: 0.0,
            sfx,
            font,
            display,
            sprites,
            pack,
            city,
            district: None,
            orders: load_json::<orders::Orders>(orders::ORDERS_KEY),
        };
        let cape = app.orders.cape;
        app.orders.wear(cape);
        app.start_run();
        app.start_preview();
        app
    }

    /// Seed the run that is about to start: today's course if this pilot
    /// has not flown it yet, otherwise a fresh random course.
    fn start_run(&mut self) {
        let day = today();
        if self.daily.day != day {
            self.daily = Daily { day, flown: [None; 3] };
            save_json(DAILY_KEY, &self.daily);
        }
        let best = self.game.best_distance;
        // Bank the orders the last flight finished; the next one gets new ones.
        if self.game.phase == Phase::Landed {
            self.orders.bank(self.game.orders_done);
            save_json(orders::ORDERS_KEY, &self.orders);
        }
        let i = self.pilot_idx;
        self.daily_run = self.daily.flown[i].is_none();
        let seed = if self.daily_run { (day as u64) ^ ((i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)) } else { (get_time() * 1e6) as u64 | 1 };
        self.game = Game::new(PILOTS[i], seed);
        self.game.best_distance = best;
        self.game.set_orders(self.orders.goals());
        self.district = None;
        self.dog_anim = anim::DogAnim::new();
    }

    /// Mark a rule as used; the prompt for it never shows again. Works on
    /// the field alone so event handlers can call it while `game` is borrowed.
    fn teach(taught: &mut u8, bit: u8) {
        if *taught & bit == 0 {
            *taught |= bit;
            save_json(TAUGHT_KEY, taught);
        }
    }

    pub fn taught(&self, bit: u8) -> bool {
        self.taught & bit != 0
    }

    /// Whether the stomp prompt may show: only once there is a board to care about.
    pub fn stomp_teachable(&self) -> bool {
        self.scores.len() >= TEACH_STOMP_AFTER && !self.taught(TEACH_STOMP)
    }

    /// Today's distance for a pilot, if that course has been flown.
    pub fn today_flown(&self, idx: usize) -> Option<f32> {
        self.daily.flown.get(idx).copied().flatten()
    }

    /// Mean distance of the recent flights, if any.
    pub fn recent_avg(&self) -> Option<f32> {
        if self.recent.is_empty() {
            None
        } else {
            Some(self.recent.iter().map(|s| s.distance).sum::<f32>() / self.recent.len() as f32)
        }
    }

    /// The look-back: when there is a flag to chase, the camera opens on it
    /// and glides back to the tower before the needle starts.
    fn start_preview(&mut self) {
        let flag_x = self.target().map(|(_, d)| d).or_else(|| self.ghost_flag_x());
        if let Some(x) = flag_x {
            self.preview = PREVIEW_TIME;
            self.preview_x = x;
            let (cx, cy, cz, _) = Cam::target_for(&self.game, Some((x, 0.0)), Vec2::ZERO);
            self.cam.x = cx;
            self.cam.y = cy;
            self.cam.zoom = cz;
        }
    }

    fn preview_state(&self) -> Option<(f32, f32)> {
        (self.preview > 0.0).then(|| (self.preview_x, 1.0 - self.preview / PREVIEW_TIME))
    }

    fn handle_input(&mut self) {
        #[allow(unused_mut)]
        let mut action = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Up)
            || is_mouse_button_pressed(MouseButton::Left)
            || touches().iter().any(|t| t.phase == TouchPhase::Started);

        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::Z) {
            // Dev: freeze the dog now, for screenshots of the frost look.
            self.game.frozen = hamsters_core::FREEZE_TIME * 3.0;
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::C) {
            // Dev: a steep dive from 30 m that must crash, for screenshots.
            self.game.phase = Phase::Flying;
            self.game.pos = hamsters_core::math::Vec2::new(40.0, 30.0);
            self.game.vel = hamsters_core::math::Vec2::new(12.0, -30.0);
            self.game.entities.clear();
            self.preview = 0.0;
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::V) {
            // Dev: a drop from 320 m into an empty sky, for the meteor.
            self.game.dev_place(hamsters_core::math::Vec2::new(120.0, 320.0), hamsters_core::math::Vec2::new(14.0, 0.0));
            self.preview = 0.0;
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::G) {
            // Dev: glide low into the next district of the city, for screenshots.
            if let Some(city) = &self.city {
                let x = self.game.pos.x;
                if let Some(d) = city.districts.iter().find(|d| d.from > x + 1.0) {
                    self.game.dev_place(hamsters_core::math::Vec2::new(d.from + 4.0, 9.0), hamsters_core::math::Vec2::new(13.0, 3.0));
                    self.preview = 0.0;
                }
            }
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::J) {
            // Dev: skip 55 m ahead, gliding low.
            let x = self.game.pos.x.max(0.0) + 55.0;
            self.game.dev_place(hamsters_core::math::Vec2::new(x, 9.0), hamsters_core::math::Vec2::new(13.0, 3.0));
            self.preview = 0.0;
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::N) {
            // Dev: hop 150 m ahead into a freshly laid stretch of course.
            let x = self.game.pos.x.max(0.0) + 150.0;
            self.game.dev_course_at(hamsters_core::math::Vec2::new(x, 32.0), hamsters_core::math::Vec2::new(30.0, 6.0));
            self.preview = 0.0;
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::F) {
            // Dev: a fireworks display at the dog.
            let at = vec2(self.game.pos.x, self.game.pos.y);
            for (i, (dx, dy)) in [(6.0, -4.0), (14.0, -2.0), (20.0, -8.0)].iter().enumerate() {
                self.shells.push(Shell { pos: at + vec2(*dx, *dy), delay: 0.1 + 0.28 * i as f32, color: FIREWORK_COLORS[i % FIREWORK_COLORS.len()] });
            }
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::T) {
            // Slow motion for frame-by-frame capture.
            self.time_scale = if self.time_scale < 1.0 { 1.0 } else { 0.25 };
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::P) {
            if let Phase::AimPower { .. } = self.game.phase {
                // Lock a 90% push and let auto_jump press at the lip.
                self.preview = 0.0;
                self.auto_jump = true;
                self.game.phase = Phase::AimPower { meter: 0.9 };
                self.game.action();
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Phase::AimPower { .. } = self.game.phase {
                let (mx, my) = mouse_position();
                if let Some(i) = hud::picker_hit(vec2(mx, my)) {
                    self.sfx.select();
                    self.pilot_idx = i;
                    self.game.set_pilot(PILOTS[i]);
                    self.start_run();
                    action = false;
                }
            }
        }
        // The look-back is not skippable by a press: the needle starting is the go cue.
        if self.preview > 0.0 {
            action = false;
        }
        if self.auto_jump {
            if let Phase::Sliding { t, .. } = self.game.phase {
                if t >= 0.965 {
                    self.game.action();
                    self.auto_jump = false;
                }
            } else if self.game.phase == Phase::Flying {
                self.auto_jump = false;
            }
        }
        if action && self.game.phase == Phase::Landed {
            action = false;
        }
        if action {
            let before = self.game.flaps;
            self.game.action();
            if self.game.phase == Phase::Flying && self.game.flaps == before && self.game.stamina < hamsters_core::FLAP_COST {
                // Empty wiggle bar: the paws still slap, but nothing happens.
                self.limp_flap = 1.0;
            }
        }
        // Restart: the button on the results card, or R / Enter. Never a
        // stray click, and not in the first half second after landing.
        if is_key_pressed(KeyCode::K) && matches!(self.game.phase, Phase::AimPower { .. }) {
            // Change capes before a launch; ranks unlock more.
            self.orders.next_cape();
            save_json(orders::ORDERS_KEY, &self.orders);
        }
        if is_key_pressed(KeyCode::M) {
            let on = self.sfx.toggle_music();
            save_json(MUSIC_KEY, &MusicSetting { on });
        }
        let mut restart = is_key_pressed(KeyCode::R) && !matches!(self.game.phase, Phase::Flying | Phase::Sliding { .. });
        let hold = hud::result_hold(self.game.crashed) as f64;
        if self.game.phase == Phase::Landed && self.landed_for > hold as f32 + 0.3 {
            let (mx, my) = mouse_position();
            let clicked = is_mouse_button_pressed(MouseButton::Left) || touches().iter().any(|t| t.phase == TouchPhase::Started);
            let touch_at = touches().first().map(|t| t.position);
            let at = touch_at.unwrap_or(vec2(mx, my));
            if clicked && hud::restart_button_rect().contains(at) {
                restart = true;
            }
        }
        // Enter works right away, like R: the hold is for the eyes, not a lock.
        if self.game.phase == Phase::Landed && is_key_pressed(KeyCode::Enter) {
            restart = true;
        }
        if restart {
            if self.game.phase == Phase::Landed {
                self.orders.bank(self.game.orders_done);
                save_json(orders::ORDERS_KEY, &self.orders);
            }
            self.sfx.click();
            self.game.reset();
            self.start_run();
            self.loose_sled = None;
            self.craters.clear();
            self.start_preview();
        }
        let pick = if is_key_pressed(KeyCode::Key1) {
            Some(0)
        } else if is_key_pressed(KeyCode::Key2) {
            Some(1)
        } else if is_key_pressed(KeyCode::Key3) {
            Some(2)
        } else {
            None
        };
        if let Some(i) = pick {
            if matches!(self.game.phase, Phase::AimPower { .. }) {
                self.sfx.select();
                self.pilot_idx = i;
                self.game.set_pilot(PILOTS[i]);
                self.start_run();
            }
        }
    }

    fn update(&mut self, frame_dt: f32) {
        // The lip hangs for a heartbeat: half speed from just before the lip
        // until the jump press, so the one right instant can be seen.
        let at_lip = matches!(self.game.phase, Phase::Sliding { t, jump: None, .. } if t > 0.97);
        let sim_dt = if at_lip { frame_dt * 0.5 } else { frame_dt };
        if self.preview > 0.0 {
            // The world waits while the camera looks back; the needle is still.
            self.preview = (self.preview - frame_dt).max(0.0);
        } else if self.hitstop > 0.0 {
            // Hit-stop is counted in real time, so a slow-motion beat
            // doesn't stretch the freeze frame.
            self.hitstop -= self.cam_dt.max(frame_dt);
        } else {
            self.acc += sim_dt.min(0.1);
            while self.acc >= FIXED_DT {
                self.game.step(FIXED_DT);
                self.acc -= FIXED_DT;
            }
        }
        self.update_aim_path();
        self.flash = (self.flash - frame_dt * 5.0).max(0.0);
        self.red_flash = (self.red_flash - frame_dt * 3.0).max(0.0);
        self.launch_stretch = (self.launch_stretch - frame_dt * 4.5).max(0.0);
        self.combo_timer -= frame_dt;
        if self.combo_timer <= 0.0 {
            self.combo = 0;
        }
        // One label at a time: the next queued label starts only when the
        // current one is past 45% of its life, so two never overlap.
        let busy = self.fx.iter().any(|f| f.age < f.life * 0.7);
        if !busy && !self.banner_queue.is_empty() {
            let next = self.banner_queue.remove(0);
            self.fx.push(next);
        }
        // Drop stale queued labels so a burst of pickups doesn't replay for seconds.
        if self.banner_queue.len() > 3 {
            let keep: Vec<Fx> = self.banner_queue.drain(..).filter(|f| f.banner).collect();
            self.banner_queue = keep;
        }
        // Crossing into a new part of town gets a welcome banner.
        if let (Some(city), Phase::Flying) = (&self.city, self.game.phase) {
            if let Some(d) = city.district_index(self.game.pos.x) {
                if self.district != Some(d) {
                    self.district = Some(d);
                    let at = vec2(self.game.pos.x, self.game.pos.y);
                    self.banner_queue.push(Fx { pos: at, age: 0.0, life: 1.6, color: Color::new(1.0, 0.96, 0.86, 1.0), text: city.districts[d].name.clone(), banner: true });
                }
            }
        }
        let events: Vec<Event> = self.game.events.drain(..).collect();
        for ev in events {
            self.on_event(ev);
        }
        let cam_dt = if self.beat_slow > 0.0 || self.crash_slow > 0.0 { self.cam_dt.max(frame_dt) } else { frame_dt };
        self.cam.follow(&self.game, cam_dt, self.preview_state());
        // Secondary motion: frozen with the world during hit-stop.
        let anim_dt = if self.hitstop > 0.0 || self.preview > 0.0 { 0.0 } else { frame_dt };
        self.dog_anim.update(&self.game, self.flap_anim, self.tumble, anim_dt);
        // The crowd: calm while aiming, on its toes once the sled goes, wild
        // for the take-off, settling during the flight.
        let cheer = match self.game.phase {
            Phase::AimPower { .. } => 0.0,
            Phase::Sliding { t, .. } => 0.4 + 0.6 * t.min(1.0),
            Phase::Flying => (1.0 - (self.game.flight_time - 2.0) / 3.0).clamp(0.3, 1.0),
            Phase::Landed => 0.3,
        };
        self.cam.cheer += (cheer - self.cam.cheer) * (1.0 - (-frame_dt * 4.0).exp());
        self.particles.update(frame_dt);
        let airspeed = if self.game.phase == Phase::Flying { self.game.vel.length() } else { 0.0 };
        self.sfx.set_wind(airspeed, self.game.rocket > 0.0, frame_dt);
        let slide_level = match self.game.phase {
            Phase::Sliding { t, .. } => 0.25 + 0.75 * t.min(1.0),
            _ => 0.0,
        };
        self.sfx.set_slide(slide_level, frame_dt);
        let burn = if self.game.phase != Phase::Flying { 0.0 } else if self.game.rocket > 0.0 { 1.0 } else { self.game.meteor() };
        self.sfx.set_burn(burn, frame_dt);
        self.sfx.update_music(frame_dt, self.game.phase == Phase::Landed);
        // Breath in cold air while the dog sits where it landed.
        if self.game.phase == Phase::Landed && !self.game.crashed {
            self.breath_timer -= frame_dt;
            if self.breath_timer <= 0.0 {
                self.breath_timer = 0.9;
                let head = vec2(self.game.pos.x + 0.9, self.game.pos.y + 0.6);
                self.particles.emit_puffs(head, 2, Color::new(1.0, 1.0, 1.0, 0.5));
            }
        }

        self.flap_anim = (self.flap_anim - frame_dt * 3.3).max(0.0);
        self.limp_flap = (self.limp_flap - frame_dt * 4.0).max(0.0);
        self.boost_echo = (self.boost_echo - frame_dt).max(0.0);
        self.pickup_pop = (self.pickup_pop - frame_dt * 8.0).max(0.0);
        self.zap_flash = (self.zap_flash - frame_dt).max(0.0);
        self.dist_pulse = (self.dist_pulse - frame_dt * 5.0).max(0.0);
        let frozen_now = self.game.frozen > 0.0;
        if frozen_now {
            self.particles.emit_frost(vec2(self.game.pos.x, self.game.pos.y), frame_dt);
        }
        if self.was_frozen && !frozen_now {
            self.sfx.thaw();
            let at = vec2(self.game.pos.x, self.game.pos.y);
            self.particles.emit_shatter(at);
            self.particles.emit_confetti(at, 24);
            self.particles.emit_sparkles(at, 16);
            self.particles.emit_ring(at, Color::new(0.7, 0.9, 1.0, 1.0));
        }
        self.was_frozen = frozen_now;
        if self.game.phase == Phase::Landed {
            self.landed_for += frame_dt;
        }
        if self.game.crashed {
            // Slow sink, head first, with the snow spurting up around the rim.
            self.crash_sink = (self.crash_sink + frame_dt * 1.1).min(1.0);
            self.sink_spurt -= frame_dt;
            if self.sink_spurt <= 0.0 && self.crash_sink < 1.0 {
                self.sink_spurt = 0.16;
                let x = self.game.pos.x;
                self.particles.emit_snow(vec2(x, 0.3), vec2(0.0, 5.0 + 6.0 * (1.0 - self.crash_sink)), 6);
            }
        }
        // Fireworks shells burst on their own clock.
        for sh in &mut self.shells {
            sh.delay -= frame_dt;
            self.particles.shell_rising(sh.pos, sh.delay);
        }
        let mut burst: Vec<Shell> = Vec::new();
        self.shells.retain_mut(|sh| {
            if sh.delay <= 0.0 {
                burst.push(Shell { pos: sh.pos, delay: 0.0, color: sh.color });
                false
            } else {
                true
            }
        });
        for sh in burst {
            self.particles.emit_fireworks(sh.pos, sh.color, 36);
            self.cam.shake_amp = self.cam.shake_amp.max(3.0);
            self.sfx.fireworks_pop();
        }
        // Riding a rocket: small crackling bursts keep going off behind the dog.
        if self.game.phase == Phase::Flying && self.game.rocket > 0.0 {
            self.rocket_burst -= frame_dt;
            if self.rocket_burst <= 0.0 {
                self.rocket_burst = 0.22;
                let g = &self.game;
                let back = vec2(g.pos.x - 2.5, g.pos.y + 1.0);
                let col = FIREWORK_COLORS[(self.game.flaps as usize + self.shells.len() + (get_time() * 7.0) as usize) % FIREWORK_COLORS.len()];
                self.particles.emit_fireworks(back, col, 10);
            }
        }
        if self.game.phase == Phase::Flying {
            let mark = (self.game.distance() / 10.0) as i32;
            if mark > self.last_mark {
                self.last_mark = mark;
                self.dist_pulse = 1.0;
            }
            // The live rank pops when the dog overtakes a scoreboard entry.
            let rank = self.live_rank();
            if rank != self.last_rank {
                if self.last_rank != 0 {
                    self.rank_pulse = 0.5;
                }
                self.last_rank = rank;
            }
            if !self.best_beaten {
                if let Some((_, d)) = self.target() {
                    if self.game.distance() >= d {
                        self.best_beaten = true;
                        self.dist_pulse = 1.0;
                        self.sfx.record();
                        self.firework_display(5);
                    }
                }
            }
        } else if !matches!(self.game.phase, Phase::Landed) {
            self.last_mark = 0;
            self.last_rank = 0;
        }
        self.rank_pulse = (self.rank_pulse - frame_dt).max(0.0);
        self.wiggle_pulse = (self.wiggle_pulse - frame_dt * 12.0).max(0.0);
        self.counter_pulse = (self.counter_pulse - frame_dt * 4.0).max(0.0);
        self.squash = (self.squash - frame_dt * 5.0).max(0.0);
        self.tuck = (self.tuck - frame_dt * 3.0).max(0.0);
        self.tumble = (self.tumble - frame_dt * 1.6).max(0.0);
        // The meteor: the ball spins faster the harder it falls, sheds
        // flame, and rattles the camera all the way down.
        let m = self.game.meteor();
        if m > 0.0 {
            self.meteor_spin += frame_dt * (6.0 + 22.0 * m);
            let gp = vec2(self.game.pos.x, self.game.pos.y);
            let gv = vec2(self.game.vel.x, self.game.vel.y);
            if m > 0.3 {
                self.particles.emit_flame(gp - gv * 0.02, gv * 0.6);
                self.particles.emit_flame(gp + vec2(0.0, 0.8), gv * 0.3);
            }
            self.cam.shake_amp = self.cam.shake_amp.max(2.0 + 8.0 * m);
        } else {
            self.meteor_spin = 0.0;
        }
        // A missed jump: the dog keeps tumbling all the way down the building.
        if self.game.botched && self.game.phase == Phase::Flying && self.tumble <= 0.0 {
            self.tumble = 1.0;
        }
        if let Some((_, tt)) = &mut self.reaction {
            *tt -= frame_dt;
            if *tt <= 0.0 {
                self.reaction = None;
            }
        }
        for f in &mut self.fx {
            f.age += frame_dt;
        }
        self.fx.retain(|f| f.age < f.life);

        if let Some(sl) = &mut self.loose_sled {
            sl.vel.y -= 22.0 * frame_dt;
            sl.pos += sl.vel * frame_dt;
            sl.rot += sl.spin * frame_dt;
            if sl.pos.y < 0.4 {
                sl.pos.y = 0.4;
                sl.vel = Vec2::ZERO;
                sl.spin = 0.0;
            }
        }

        let g = &self.game;
        let hp = vec2(g.pos.x, g.pos.y);
        if g.phase == Phase::Flying && self.run_path.last().map_or(true, |p| hp.x - p.x >= 2.0) {
            self.run_path.push(hp);
        }
        self.trail_timer -= frame_dt;
        if g.phase == Phase::Flying && self.trail_timer <= 0.0 {
            self.trail_timer = 0.03;
            self.trail.push(hp);
            if self.trail.len() > 14 {
                self.trail.remove(0);
            }
        } else if g.phase != Phase::Flying {
            self.trail.clear();
        }

        self.emit_timer -= frame_dt;
        if self.emit_timer <= 0.0 {
            self.emit_timer = 0.04;
            if g.phase == Phase::Flying {
                if g.rocket > 0.0 {
                    self.particles.emit_flame(hp - vec2(1.2, 0.0), vec2(g.vel.x, g.vel.y));
                }
                if g.glide > 0.0 {
                    self.particles.emit_wind(hp, vec2(g.vel.x, g.vel.y));
                }
            }
            if g.phase == Phase::Flying && g.vel.length() > 25.0 {
                self.particles.emit_wind(hp + vec2(2.0, 0.0), vec2(g.vel.x, g.vel.y));
                if !self.boomed && g.flight_time > 0.3 {
                    self.boomed = true;
                    self.cam.shake_amp = 5.0;
                }
            }
            if matches!(g.phase, Phase::Sliding { .. }) {
                // Snow spray off the runners.
                self.particles.emit_spray(hp - vec2(0.6, 1.1), vec2(g.vel.x, g.vel.y));
            }
        }
    }

    /// Keep the aim arc in step with the meter while aiming and with the
    /// jump press while sliding; it goes away at the lip.
    fn update_aim_path(&mut self) {
        match self.game.phase {
            Phase::AimPower { meter } => {
                self.aim_frame += 1;
                if self.aim_frame % 3 == 1 || self.aim_path.is_empty() {
                    let power = Game::meter_value(meter);
                    self.aim_path = predict_arc(self.game.pilot, power, 1.0).into_iter().map(|p| vec2(p.x, p.y)).collect();
                }
                self.aim_jump = None;
            }
            Phase::Sliding { power, jump, .. } => {
                if jump != self.aim_jump || self.aim_path.is_empty() {
                    self.aim_jump = jump;
                    self.aim_path = predict_arc(self.game.pilot, power, jump.unwrap_or(1.0)).into_iter().map(|p| vec2(p.x, p.y)).collect();
                }
            }
            _ => {
                self.aim_path.clear();
                self.aim_jump = None;
            }
        }
    }

    /// Predicted landing distance for the current aim, if any.
    pub fn aim_distance(&self) -> Option<f32> {
        self.aim_path.last().map(|p| p.x)
    }

    /// Where the ghost's flag stands, if there is a best run to race.
    pub fn ghost_flag_x(&self) -> Option<f32> {
        self.best_path.path.last().map(|p| p.0)
    }

    /// The next rung to beat: the lowest-ranked scoreboard entry above the
    /// last run, as (rank, distance). A run that already sits at #1 chases
    /// itself plus ten percent. `None` with an empty board.
    pub fn target(&self) -> Option<(usize, f32)> {
        let best = self.scores.first()?.distance;
        if self.last_distance >= best - 0.01 {
            return Some((1, best * 1.1));
        }
        self.scores
            .iter()
            .enumerate()
            .filter(|(_, s)| s.distance > self.last_distance)
            .last()
            .map(|(i, s)| (i + 1, s.distance))
    }

    /// Where the current distance would rank on the scoreboard, 1-based.
    pub fn live_rank(&self) -> usize {
        let d = self.game.distance();
        self.scores.iter().filter(|s| s.distance > d).count() + 1
    }

    /// A pilot's longest flight on the scoreboard, if any.
    pub fn pilot_best(&self, idx: usize) -> Option<f32> {
        self.scores.iter().filter(|s| s.pilot == idx).map(|s| s.distance).fold(None, |m, d| Some(m.map_or(d, |m: f32| m.max(d))))
    }

    fn on_event(&mut self, ev: Event) {
        let g = &self.game;
        let at = vec2(g.pos.x, g.pos.y);
        match ev {
            Event::Pushed => {
                // The first gesture of a run: browsers allow audio from here.
                self.sfx.start_music();
                self.sfx.lock();
                Self::teach(&mut self.taught, TEACH_PUSH);
                self.particles.emit_snow(vec2(RAMP_TOP.x, RAMP_TOP.y + 0.3), vec2(-2.0, 2.0), 6);
            }
            Event::Jumped { quality, late } => {
                if quality >= 0.5 {
                    Self::teach(&mut self.taught, TEACH_JUMP);
                }
                let l = &self.pack.labels;
                let (text, color) = if quality >= hamsters_core::JUMP_PERFECT_FROM {
                    (l.jump_perfect.clone(), Color::new(1.0, 0.86, 0.3, 1.0))
                } else if late {
                    (l.jump_late.clone(), Color::new(1.0, 0.55, 0.35, 1.0))
                } else if quality >= 0.5 {
                    (l.jump_good.clone(), Color::new(0.6, 0.95, 0.7, 1.0))
                } else if quality > 0.05 {
                    (l.jump_early.clone(), Color::new(1.0, 0.7, 0.4, 1.0))
                } else {
                    (l.jump_missed.clone(), Color::new(1.0, 0.45, 0.45, 1.0))
                };
                self.fx.retain(|f| !f.banner);
                self.banner_queue.clear();
                self.fx.push(Fx { pos: at, age: 0.0, life: 1.0, color, text, banner: true });
                if quality >= hamsters_core::JUMP_PERFECT_FROM {
                    self.sfx.jump();
                    self.particles.emit_sparkles(at, 30);
                    self.particles.emit_confetti(at, 16);
                }
            }
            Event::Launched => {
                self.sfx.launch();
                self.particles.emit_snow(vec2(LIP.x, LIP.y + 0.3), vec2(g.vel.x * 0.3, 6.0), 40);
                self.particles.emit_clods(vec2(LIP.x, LIP.y + 0.2), vec2(g.vel.x * 0.4, 8.0), 70);
                self.particles.emit_burst_behind(at, 24);
                self.particles.emit_ring(at, WHITE);
                let power = (g.vel.length() / g.pilot.launch_speed).clamp(0.0, 1.0);
                self.cam.shake_amp = 4.0 + 8.0 * power;
                self.cam.punch = 1.0;
                self.hitstop = 0.1;
                self.flash = 0.7;
                self.launch_stretch = 1.0;
                let perfect = !g.botched && g.jump_q >= hamsters_core::JUMP_PERFECT_FROM;
                self.dog_anim.launch(perfect);
                if perfect {
                    // The money shot: the world drops to slow motion while
                    // the camera punches in on the dog leaving the lip.
                    self.beat_slow = BEAT_SLOW;
                    self.cam.beat = 1.0;
                    self.cam.punch = 0.0;
                }
                self.pickup_pop = 0.4;
                self.best_before = g.best_distance;
                self.boomed = false;
                self.best_beaten = false;
                self.run_path = vec![at];
                // The sled keeps going without the dog.
                self.loose_sled = Some(LooseSled { pos: vec2(LIP.x, LIP.y + 0.4), vel: vec2(g.vel.x * 0.45, g.vel.y * 0.4), rot: 0.0, spin: -6.0 });
                if g.botched {
                    // No fanfare for a miss: the sled tips over the edge and
                    // the dog goes down the side of the building in a panic.
                    self.cam.punch = 0.0;
                    self.hitstop = 0.0;
                    self.flash = 0.0;
                    self.launch_stretch = 0.0;
                    self.cam.shake_amp = 2.0;
                    self.tumble = 1.0;
                    self.sfx.botch();
                    self.reaction = Some((draw::Face::Panic, 3.0));
                    self.particles.emit_snow(vec2(LIP.x, LIP.y + 0.3), vec2(1.0, 1.5), 8);
                }
            }
            Event::Flap => {
                Self::teach(&mut self.taught, TEACH_FLAP);
                self.sfx.flap();
                self.sfx.gust();
                self.flap_anim = 1.0;
                self.wiggle_pulse = 1.0;
                self.dog_anim.flap(g.flap_lift());
                // A wiggle at speed bites more air: bigger puff, more spray.
                let lift = g.flap_lift();
                self.particles.emit_puffs(at - vec2(0.0, 1.6), (8.0 + 8.0 * lift) as usize, Color::new(1.0, 1.0, 1.0, 0.95));
                self.particles.emit_snow(at - vec2(0.0, 1.2), vec2(0.0, -5.0 - 4.0 * lift), (8.0 + 6.0 * lift) as usize);
                self.particles.emit_sparkles(at + vec2(0.0, 0.8), 4);
            }
            Event::WigglePower => {
                self.sfx.power();
                self.particles.emit_sparkles(at, 24);
                self.particles.emit_confetti(at, 20);
                self.particles.emit_ring(at, Color::new(1.0, 0.86, 0.3, 1.0));
                self.flash = 0.45;
                self.banner_queue.push(Fx { pos: at, age: 0.0, life: 1.4, color: Color::new(1.0, 0.86, 0.3, 1.0), text: self.pack.labels.wiggle_power.clone(), banner: true });
                self.cam.shake_amp = 6.0;
            }
            Event::Hit(kind) => {
                let gold = Color::new(1.0, 0.86, 0.30, 1.0);
                let l = &self.pack.labels;
                let (color, text) = match kind {
                    EntityKind::Trampoline => (Color::new(1.0, 0.55, 0.45, 1.0), l.trampoline.as_str()),
                    EntityKind::Reindeer => (Color::new(1.0, 0.75, 0.55, 1.0), l.reindeer.as_str()),
                    EntityKind::Wings => (Color::new(0.85, 0.95, 1.0, 1.0), l.wings.as_str()),
                    EntityKind::Firework => (Color::new(1.0, 0.72, 0.35, 1.0), l.firework.as_str()),
                    EntityKind::Present => (gold, l.present.as_str()),
                    EntityKind::Sleigh => (Color::new(1.0, 0.5, 0.5, 1.0), l.sleigh.as_str()),
                    EntityKind::Ufo => (Color::new(0.6, 1.0, 0.7, 1.0), l.ufo.as_str()),
                    EntityKind::Updraft => (Color::new(0.75, 0.92, 1.0, 1.0), l.updraft.as_str()),
                    EntityKind::ColdWind => (Color::new(0.6, 0.85, 1.0, 1.0), l.coldwind.as_str()),
                    EntityKind::Stormcloud => (Color::new(0.95, 0.9, 0.5, 1.0), l.stormcloud.as_str()),
                };
                self.reaction = Some(match kind {
                    EntityKind::Present => (draw::Face::Whee, 0.35),
                    EntityKind::Trampoline | EntityKind::Firework | EntityKind::Sleigh | EntityKind::Ufo => (draw::Face::Scream, 0.6),
                    EntityKind::Stormcloud => (draw::Face::Squint, 0.6),
                    EntityKind::ColdWind => (draw::Face::Panic, hamsters_core::FREEZE_TIME),
                    _ => (draw::Face::Whee, 0.4),
                });
                match kind {
                    EntityKind::Trampoline => self.sfx.trampoline(),
                    EntityKind::Reindeer => self.sfx.reindeer(),
                    EntityKind::Wings => self.sfx.wings(),
                    EntityKind::Firework => self.sfx.firework(),
                    EntityKind::Sleigh => self.sfx.sleigh(),
                    EntityKind::Ufo => self.sfx.ufo(),
                    EntityKind::Updraft => self.sfx.updraft(),
                    EntityKind::ColdWind => self.sfx.freeze(),
                    EntityKind::Stormcloud => self.sfx.zap(),
                    EntityKind::Present => {}
                }
                match kind {
                    EntityKind::Trampoline => {
                        self.dog_anim.impact(1.0);
                        self.particles.emit_ring(at, color);
                        self.particles.emit_snow(vec2(at.x, 0.3), vec2(0.0, 4.0), 30);
                        self.squash = 1.0;
                        self.cam.shake_amp = 10.0;
                        self.cam.punch = 0.8;
                        self.hitstop = 0.07;
                    }
                    EntityKind::Reindeer => {
                        self.particles.emit_feathers(at, 8);
                        self.particles.emit_sparkles(at, 10);
                        self.particles.emit_burst_behind(at, 16);
                        self.cam.shake_amp = 6.0;
                        self.cam.punch = 1.0;
                        self.hitstop = 0.05;
                        self.boost_echo = 0.8;
                    }
                    EntityKind::Present => {
                        self.particles.emit_sparkles(at, 8);
                        self.particles.emit_confetti(at, 5);
                        self.particles.emit_ring_sized(at, gold, 1.0, 0.14);
                        self.pickup_pop = 1.0;
                        self.dog_anim.pop();
                    }
                    EntityKind::Firework => {
                        self.particles.emit_ring(at, color);
                        self.cam.shake_amp = 7.0;
                        self.cam.punch = 0.7;
                        // A proper display: three shells climb ahead of the dog and burst in turn.
                        let dir = vec2(g.vel.x, g.vel.y).normalize_or_zero();
                        for (i, (dx, dy)) in [(5.0, 3.0), (11.0, 6.0), (17.0, 2.0)].iter().enumerate() {
                            let pos = at + vec2(*dx + dir.x * 3.0, *dy + dir.y.max(0.0) * 2.0);
                            self.shells.push(Shell { pos, delay: 0.18 + 0.28 * i as f32, color: FIREWORK_COLORS[(self.shells.len() + i) % FIREWORK_COLORS.len()] });
                        }
                    }
                    EntityKind::Wings => self.particles.emit_sparkles(at, 12),
                    EntityKind::ColdWind => {
                        self.particles.emit_freeze(at);
                        self.particles.emit_snow(at, vec2(-8.0, 0.0), 30);
                        self.particles.emit_sparkles(at, 10);
                        self.hitstop = 0.08;
                        self.flash = 0.35;
                        self.cam.shake_amp = 8.0;
                        self.fx.retain(|f| !f.banner);
                        self.banner_queue.clear();
                        self.fx.push(Fx { pos: at, age: 0.0, life: 1.0, color, text: text.to_owned(), banner: true });
                    }
                    EntityKind::Updraft => {
                        self.particles.emit_snow(at - vec2(0.0, 2.0), vec2(0.0, 14.0), 30);
                        self.particles.emit_puffs(at - vec2(0.0, 2.5), 14, Color::new(1.0, 1.0, 1.0, 0.9));
                        self.cam.shake_amp = 4.0;
                        self.boost_echo = 0.6;
                    }
                    EntityKind::Sleigh | EntityKind::Ufo => {
                        self.particles.emit_ring(at, color);
                        self.particles.emit_confetti(at, 30);
                        self.particles.emit_sparkles(at, 20);
                        self.cam.shake_amp = 10.0;
                    }
                    EntityKind::Stormcloud => {
                        self.particles.emit_zap(at);
                        self.dog_anim.impact(0.8);
                        self.particles.emit_snow(at, vec2(-4.0, -2.0), 16);
                        self.particles.emit_sparkles(at, 14);
                        self.squash = 1.0;
                        self.cam.shake_amp = 12.0;
                        self.hitstop = 0.12;
                        self.red_flash = 0.7;
                    }
                }
                let banner = matches!(kind, EntityKind::Sleigh | EntityKind::Ufo);
                let mut text = text.to_owned();
                if kind == EntityKind::Present {
                    self.combo += 1;
                    self.sfx.present(self.combo);
                    self.combo_timer = 1.3;
                    self.counter_pulse = 1.0;
                    text = String::new();
                    // A hot "xN" stamp rising off the pickup point, hotter with each link.
                    if self.combo > 1 {
                        let heat = ((self.combo - 2) as f32 / 4.0).min(1.0);
                        let col = if heat < 0.5 { lerp(gold, Color::new(1.0, 0.5, 0.15, 1.0), heat * 2.0) } else { lerp(Color::new(1.0, 0.5, 0.15, 1.0), Color::new(1.0, 0.3, 0.3, 1.0), heat * 2.0 - 1.0) };
                        self.fx.retain(|f| f.banner || !f.text.starts_with('x'));
                        self.fx.push(Fx { pos: at, age: 0.0, life: 0.55, color: col, text: format!("x{}", self.combo), banner: false });
                    }
                }
                // Every event label is a banner in the one low slot; a zap
                // clears the slot so it lands on the flash frame.
                let fx = Fx { pos: at, age: 0.0, life: if banner { 1.2 } else { 0.55 }, color, text, banner: true };
                if kind == EntityKind::Stormcloud {
                    // The zap lands on the dog: red stamp at the body, white
                    // flash, then an electrified white/black silhouette.
                    self.fx.retain(|f| !f.banner && !f.text.starts_with('x'));
                    self.banner_queue.clear();
                    self.fx.push(Fx { pos: at, age: 0.0, life: 0.9, color: Color::new(1.0, 0.92, 0.2, 1.0), text: self.pack.labels.stormcloud.clone(), banner: false });
                    self.flash = 0.5;
                    self.zap_flash = 0.16;
                    self.red_flash = 1.0;
                    self.cam.shake_amp = 16.0;
                } else if kind != EntityKind::Present && kind != EntityKind::ColdWind {
                    self.banner_queue.push(fx);
                }
            }
            Event::StompArmed => {
                Self::teach(&mut self.taught, TEACH_STOMP);
                // The tuck: a quick crouch and a puff, no words.
                self.tuck = 1.0;
                self.dog_anim.impact(0.4);
                self.particles.emit_puffs(at - vec2(0.0, 1.0), 6, Color::new(1.0, 1.0, 1.0, 0.8));
            }
            Event::GroundBounce { speed, stomped } => {
                self.sfx.bounce(speed, stomped);
                self.dog_anim.impact(speed / 14.0);
                if stomped {
                    // A stomped bounce: one clean white ring and a big spray.
                    self.particles.emit_ring(vec2(at.x, 0.6), WHITE);
                    self.particles.emit_snow(vec2(at.x, 0.2), vec2(0.0, 6.0), 40);
                    self.particles.emit_clods(vec2(at.x, 0.2), vec2(0.0, 7.0), 12);
                    self.squash = 1.0;
                    self.cam.shake_amp = (speed * 0.45).min(12.0);
                    self.cam.punch = 0.6;
                    self.hitstop = 0.05;
                } else if speed > 6.0 {
                    if speed > 12.0 {
                        let word = if speed > 20.0 { "SMASH!" } else { "BOUNCE!" };
                        self.fx.retain(|f| !f.banner);
                        self.banner_queue.clear();
                        self.fx.push(Fx { pos: vec2(at.x, 2.5), age: 0.0, life: 0.7, color: Color::new(0.85, 0.92, 1.0, 1.0), text: word.into(), banner: true });
                    }
                    let n = (speed * 0.9) as usize;
                    self.particles.emit_snow(vec2(at.x, 0.2), vec2(0.0, 3.0), n.min(28));
                    self.squash = 1.0;
                    self.cam.shake_amp = (speed * 0.35).min(10.0);
                    if speed > 12.0 && self.game.phase == Phase::Flying {
                        self.tumble = 1.0;
                    }
                }
            }
            Event::Meteor => {
                self.sfx.meteor();
                self.reaction = Some((draw::Face::Scream, 4.0));
                self.banner_queue.push(Fx { pos: at, age: 0.0, life: 1.1, color: Color::new(1.0, 0.45, 0.10, 1.0), text: self.pack.labels.meteor.clone(), banner: true });
                self.particles.emit_ring(at, Color::new(1.0, 0.6, 0.2, 1.0));
                self.particles.emit_burst_behind(at, 24);
            }
            Event::Crashed { speed } => {
                self.sfx.crash_impact(speed);
                self.fx.retain(|f| !f.banner);
                self.banner_queue.clear();
                // How much harder than an ordinary splat: 0 at the crash
                // threshold, 1 for a full meteor.
                let big = ((speed - 35.0) / 35.0).clamp(0.0, 1.0);
                let big = big * big * (3.0 - 2.0 * big);
                let label = if big > 0.5 { self.pack.labels.crash_meteor.clone() } else { self.pack.labels.crash.clone() };
                let color = if big > 0.5 { Color::new(1.0, 0.45, 0.10, 1.0) } else { Color::new(1.0, 0.92, 0.2, 1.0) };
                self.fx.push(Fx { pos: vec2(at.x, 2.5), age: 0.0, life: 1.2 + 0.6 * big, color, text: label, banner: true });
                self.particles.emit_crash(at.x, big);
                self.crash_size = 1.0 + 1.2 * big;
                self.craters.push((at.x, 1.4 + 1.6 * big));
                self.hitstop = 0.12 + 0.2 * big;
                self.flash = 0.12 + 0.5 * big;
                self.cam.shake_amp = (speed * 0.6).min(20.0) + 30.0 * big;
                self.crash_sink = 0.0;
                self.sink_spurt = 0.0;
                // The crash is the payoff of a bad flight: play it in slow motion.
                self.crash_slow = CRASH_SLOW;
                if big > 0.5 {
                    // A meteor gets the deeper beat on top: a fifth of speed at the hit.
                    self.beat_slow = BEAT_SLOW;
                }
            }
            Event::Milestone(i) => {
                self.sfx.milestone();
                self.firework_display(3);
                let name = self.pack.milestone(i);
                self.particles.emit_confetti(at, 40);
                self.particles.emit_sparkles(at, 20);
                self.banner_queue.push(Fx { pos: at, age: 0.0, life: 2.0, color: Color::new(0.75, 0.95, 1.0, 1.0), text: name.to_owned(), banner: true });
            }
            Event::CloseShave => {
                // Skimmed a storm cloud: a quick stamp and a crackle of sparks.
                self.sfx.gust();
                self.particles.emit_sparkles(at, 14);
                self.particles.emit_ring_sized(at, Color::new(0.95, 0.9, 0.5, 1.0), 0.8, 0.2);
                self.fx.retain(|f| f.banner || f.text != "CLOSE SHAVE!");
                self.fx.push(Fx { pos: at, age: 0.0, life: 0.8, color: Color::new(0.95, 0.9, 0.5, 1.0), text: "CLOSE SHAVE!".into(), banner: false });
            }
            Event::MissionComplete(i) => {
                // An order done mid-flight: a gold ribbon, confetti, and the
                // row on the tracker flashes.
                self.orders.ticked_at[i] = Some(get_time());
                self.sfx.milestone();
                self.particles.emit_confetti(at, 36);
                self.particles.emit_sparkles(at, 18);
                let what = self.orders.goals()[i].map(hamsters_core::missions::describe).unwrap_or_default().to_uppercase();
                self.banner_queue.push(Fx { pos: at, age: 0.0, life: 1.8, color: hud::GOLD, text: format!("DONE: {what}!"), banner: true });
            }
            Event::Landed { .. } => {
                self.sfx.land();
                self.particles.emit_snow(vec2(at.x, 0.2), vec2(0.0, 2.0), 8);
                self.particles.emit_clods(vec2(at.x, 0.2), vec2(0.0, 6.0), 10);
                if !g.crashed {
                    self.particles.emit_ring(vec2(at.x, 0.6), hud::GOLD);
                }
                self.landed_for = 0.0;
                self.breath_timer = 0.4;
                // Tongue out, catching breath, for the whole hold.
                self.reaction = Some((draw::Face::Whee, hud::result_hold(g.crashed)));
                self.last_distance = g.distance();
                self.sfx.queue_results(hud::result_hold(g.crashed), g.crashed, g.distance() > 0.0 && g.distance() >= self.best_before);
                self.bests[self.pilot_idx] = self.bests[self.pilot_idx].max(g.distance());
                if g.distance() > 0.0 && g.distance() >= self.best_before {
                    let mut path: Vec<(f32, f32)> = self.run_path.iter().map(|p| (p.x, p.y)).collect();
                    path.push((g.distance(), PILOT_RADIUS));
                    self.best_path = BestPath { pilot: self.pilot_idx, path };
                    save_best_path(&self.best_path);
                }
                self.scores.push(Score { distance: g.distance(), pilot: self.pilot_idx, crashed: g.crashed });
                self.scores.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap_or(std::cmp::Ordering::Equal));
                self.scores.truncate(10);
                save_scores(&self.scores);
                if self.daily_run {
                    self.daily.flown[self.pilot_idx] = Some(g.distance());
                    save_json(DAILY_KEY, &self.daily);
                }
                self.recent.push(Score { distance: g.distance(), pilot: self.pilot_idx, crashed: g.crashed });
                if self.recent.len() > RECENT_RUNS {
                    let extra = self.recent.len() - RECENT_RUNS;
                    self.recent.drain(..extra);
                }
                save_json(RECENT_KEY, &self.recent);
                // The rank stamp drops onto the pennant that now marks this run.
                // A crash keeps its SPLAT banner and the hole; no stamp on top.
                let rank = if self.game.crashed { None } else { self.scores.iter().position(|s| (s.distance - self.game.distance()).abs() < 0.01) };
                if rank == Some(0) {
                    self.firework_display(6);
                }
                if let Some(rank) = rank {
                    self.fx.retain(|f| !f.banner);
                    self.banner_queue.clear();
                    self.fx.push(Fx { pos: vec2(at.x, 3.0), age: 0.0, life: 1.6, color: hud::GOLD, text: format!("#{}", rank + 1), banner: false });
                }
                // Orders finished this flight may lift the flight-school rank.
                let gained = self.game.orders_done.iter().filter(|d| **d).count() as u32;
                let (before, after) = (hamsters_core::missions::rank_for(self.orders.done), hamsters_core::missions::rank_for(self.orders.done + gained));
                if after > before {
                    let title = hamsters_core::missions::RANKS[after].1;
                    self.banner_queue.push(Fx { pos: at, age: 0.0, life: 2.0, color: hud::GOLD, text: format!("{title}! NEW CAPE"), banner: true });
                }
            }
        }
        // Craters where the dog hit the snow.
        if let Event::GroundBounce { speed, .. } = ev {
            if speed > 8.0 {
                self.craters.push((at.x, (speed / 30.0).clamp(0.3, 1.0)));
                if self.craters.len() > 24 {
                    self.craters.remove(0);
                }
            }
        }
    }

    fn render(&self) {
        let g = &self.game;
        let cam = &self.cam;
        let sp = &self.sprites;
        let t = get_time() as f32;
        // World into the post-processing scene buffer; the HUD stays outside.
        let post_on = post::begin();

        // While aiming the sun sits further left, clear of the pilot picker.
        let aim_k = match g.phase {
            Phase::AimPower { .. } => 1.0,
            Phase::Sliding { t, .. } => (1.0 - t * 1.5).clamp(0.0, 1.0),
            _ => 0.0,
        };
        draw::sky(sp, cam, aim_k);
        draw::clouds(sp, cam);
        draw::jet_streamers(cam, t);
        draw::mountains(sp, cam);
        if let Some(c) = &self.city {
            c.skyline(cam);
        }
        post::haze(cam);
        draw::hills(sp, cam, if g.phase == Phase::Flying { g.vel.length() } else { 0.0 });
        draw::midground(sp, cam);
        if let Some(c) = &self.city {
            c.street(cam, g.pos.x, t);
        }
        if g.crashed {
            // Head-first in the snow: drawn before the ground so it disappears into it.
            let k = self.crash_sink;
            let ease = 1.0 - (1.0 - k) * (1.0 - k);
            draw::dog_buried(sp, cam, g.pos.x, self.pilot_idx, ease, t, self.crash_size);
        }
        draw::ground(sp, cam, &self.font, &self.pack.towns, self.city.as_ref());
        for (cx, size) in &self.craters {
            vfx::snow_crater(cam, *cx, *size);
        }
        draw::landing_zone(cam, g.landing_x, t);
        // Every past flight is a pennant to pass; the best is the ghost's
        // checkered flag when there is one, else a taller gold pennant.
        let flagged = !self.best_path.path.is_empty();
        for (i, sc) in self.scores.iter().enumerate() {
            let gold = (sc.distance - g.best_distance).abs() < 0.01;
            if gold && flagged {
                continue;
            }
            draw::pennant(cam, &self.display, sc.distance, i + 1, gold, t);
        }
        // The ghost is last run's record; when this run just set it, the flag
        // would pop onto the landing spot mid-hold, so it waits for the restart.
        let fresh_record = g.phase == Phase::Landed && g.distance() > 0.0 && g.distance() >= self.best_before;
        if !self.best_path.path.is_empty() && !fresh_record {
            draw::ghost_path(sp, cam, &self.best_path.path, self.best_path.pilot, t);
        }
        if !self.aim_path.is_empty() {
            let sweet = match g.phase {
                Phase::AimPower { meter } => Game::meter_value(meter) > 0.82,
                Phase::Sliding { power, .. } => power > 0.82,
                _ => false,
            };
            draw::aim_arc(cam, &self.aim_path, sweet);
        }
        draw::spectators(sp, cam, g.pos.x, t);
        draw::start_area(sp, cam, &self.display, &self.pack.tower_sign);
        anim::roof_crowd(sp, cam, g.pos.x, t);

        for e in &g.entities {
            draw::entity(sp, cam, e, t, if g.phase == Phase::Flying { Some((vec2(g.pos.x, g.pos.y), vec2(g.vel.x, g.vel.y))) } else { None });
        }

        let speed = g.vel.length();
        if g.phase == Phase::Flying {
            vfx::motion_trail(cam, &self.trail, vec2(g.pos.x, g.pos.y), speed, draw::scarf_color(self.pilot_idx), PILOT_RADIUS * cam.scale());
        }

        self.particles.draw_behind(sp, cam);
        if g.phase == Phase::Flying {
            draw::shadow(cam, g.pos.x, g.pos.y);
        }
        if let Some(sl) = &self.loose_sled {
            draw::sled(sp, cam, sl.pos, sl.rot);
        }

        let hp = cam.to_screen(g.pos.x, g.pos.y);
        let r = PILOT_RADIUS * cam.scale();
        let sliding = matches!(g.phase, Phase::Sliding { .. });
        // Body follows the velocity vector; steeper when diving, so the arc
        // reads in the character itself.
        let tilt = if (g.phase == Phase::Flying || sliding) && speed > 2.0 {
            let ang = g.vel.y.atan2(g.vel.x);
            let k = if sliding { 0.9 } else { 1.05 };
            // Nose up while rising, nose down while falling, a touch more than physics.
            let bias = if sliding { 0.0 } else { (g.vel.y / 14.0).clamp(-1.0, 1.0) * 0.28 };
            // Capped near the lip so the launch pose reads as a leap, not a smear.
            let cap = if g.phase == Phase::Flying && g.flight_time < 0.6 { 0.45 } else { 1.0 };
            (ang * k + bias + self.wiggle_pulse * 0.18).clamp(-cap, cap)
        } else {
            0.0
        };
        if sliding || matches!(g.phase, Phase::AimPower { .. }) {
            // Dog rides the sled; the sled sits just below the dog.
            let (c, s) = (tilt.cos(), tilt.sin());
            let off = vec2(-0.1, -0.85);
            let sled_pos = vec2(g.pos.x + off.x * c - off.y * s, g.pos.y + off.x * s + off.y * c);
            draw::sled(sp, cam, sled_pos, tilt);
        }
        let stretch = if g.phase == Phase::Flying {
            (((speed - 12.0) / 90.0).clamp(0.0, 0.18) + self.launch_stretch * 0.08).min(0.25) - self.wiggle_pulse * 0.32 - self.squash * 0.22 - self.tuck * 0.18 + self.pickup_pop * 0.14
        } else if sliding {
            // Crouched on the sled.
            -0.14
        } else {
            0.0
        };
        // Face and limbs follow the state of the flight.
        // Face and body pose follow the state of the flight.  A wiggle press
        // plays a two-frame paw cycle (up, then slapped down) over the pose.
        let flap_frame = if self.flap_anim > 0.55 {
            Some(draw::Limbs::FlapUp)
        } else if self.flap_anim > 0.0 || self.limp_flap > 0.0 {
            Some(draw::Limbs::FlapDown)
        } else {
            None
        };
        let (face, limbs) = if sliding {
            // Always happy on the sled: open-mouthed joy on the ride.
            (draw::Face::Whee, draw::Limbs::Seated)
        } else if g.phase == Phase::Flying {
            let (face, body) = if g.frozen > 0.0 {
                (draw::Face::Panic, draw::Limbs::Land)
            } else if g.flight_time < 0.6 {
                // A PERFECT take-off is pure joy; anything else is a yell.
                let joy = !g.botched && g.jump_q >= hamsters_core::JUMP_PERFECT_FROM;
                (if joy { draw::Face::Whee } else { draw::Face::Scream }, draw::Limbs::Spread)
            } else if g.meteor() > 0.2 {
                (draw::Face::Scream, draw::Limbs::Tumble)
            } else if self.tumble > 0.0 {
                (draw::Face::Dizzy, draw::Limbs::Tumble)
            } else if let Some((face, _)) = self.reaction {
                (face, if speed > 30.0 { draw::Limbs::Dive } else { draw::Limbs::Spread })
            } else if self.squash > 0.5 || self.tuck > 0.4 {
                (draw::Face::Squint, draw::Limbs::Land)
            } else if g.rocket > 0.0 || speed > 26.0 || (g.vel.y > 6.0 && speed > 16.0) {
                (draw::Face::Scream, draw::Limbs::Dive)
            } else if g.stamina < hamsters_core::FLAP_COST * 3.0 {
                // Nearly gassed: tired face, legs hanging, no matter the direction.
                (draw::Face::Panic, draw::Limbs::Land)
            } else if g.pos.y < 9.0 && g.vel.y < -4.0 {
                (draw::Face::Squint, draw::Limbs::Land)
            } else if g.vel.y < -9.0 && g.glide <= 0.0 {
                (draw::Face::Panic, draw::Limbs::Spread)
            } else {
                (draw::Face::Idle, draw::Limbs::Spread)
            };
            let body = match (flap_frame, body) {
                (Some(f), draw::Limbs::Spread) | (Some(f), draw::Limbs::Land) => f,
                (Some(f), draw::Limbs::Dive) if self.limp_flap <= 0.0 => f,
                _ => body,
            };
            (face, body)
        } else {
            (draw::Face::Idle, draw::Limbs::Seated)
        };
        // Blink while sitting still on the start gate.
        let face = if face == draw::Face::Idle && self.dog_anim.blinking() && !matches!(g.phase, Phase::Flying) { draw::Face::Grit } else { face };
        if speed > 14.0 && g.phase == Phase::Flying && g.flight_time > 0.3 {
            let k_speed = ((speed - 14.0) / 26.0).clamp(0.0, 1.0);
            let dir = vec2(g.vel.x, -g.vel.y).normalize_or_zero();
            let side = vec2(-dir.y, dir.x);
            // Two or three tapered comet strokes off the trailing edge.
            let n = 2 + (k_speed > 0.5) as i32;
            for i in 0..n {
                let lane = (i as f32 - (n as f32 - 1.0) / 2.0) * r * 0.9;
                let start = hp - dir * r * 1.15 + side * lane;
                let len = r * (0.8 + 1.2 * k_speed) * (0.8 + 0.4 * draw::hash(i + (t * 12.0) as i32, 77));
                let end = start - dir * len;
                let w = r * (0.16 + 0.1 * k_speed);
                draw_triangle(start + side * (w + 2.0), start - side * (w + 2.0), end, draw::OUTLINE);
                draw_triangle(start + side * w, start - side * w, end, Color::new(1.0, 1.0, 1.0, 0.85));
            }
        }

        if g.phase == Phase::Flying && speed > 18.0 {
            let dir = vec2(g.vel.x, -g.vel.y).normalize_or_zero();
            let k_speed = ((speed - 18.0) / 30.0).clamp(0.0, 1.0);
            let n = if g.rocket > 0.0 || self.boost_echo > 0.0 { 4 } else { 1 + (k_speed * 1.5) as i32 };
            let scarf = draw::scarf_color(self.pilot_idx);
            for i in 1..=n {
                let a = (0.22 - 0.05 * i as f32) * k_speed.max(if g.rocket > 0.0 || self.boost_echo > 0.0 { 0.8 } else { 0.0 });
                let ghost = hp - dir * r * (1.9 * i as f32);
                let tint = Color::new(scarf.r * 0.6 + 0.4, scarf.g * 0.6 + 0.4, scarf.b * 0.6 + 0.4, a);
                draw::dog_echo(sp, ghost, r * (1.0 - 0.05 * i as f32), self.pilot_idx, limbs, face, tilt, tint);
            }
        }
        if g.crashed {
            vfx::crash_hole(cam, g.pos.x, self.crash_size, t);
            props::crash_rim(cam, g.pos.x, self.crash_size, self.crash_sink);
            // Paws and tail above the rim: redraw the buried body clipped to above the hole.
            if self.crash_sink >= 1.0 {
                draw::dog_paws_up(sp, cam, g.pos.x, self.pilot_idx, t);
            }
        }
        if g.phase != Phase::Landed {
            draw::dog(
                sp,
                hp,
                r,
                self.pilot_idx,
                draw::Pose {
                    tilt,
                    flap: self.flap_anim,
                    gliding: g.glide > 0.0,
                    rocketing: g.rocket > 0.0,
                    flying: g.phase == Phase::Flying,
                    stretch,
                    spin: -(1.0 - self.tumble) * std::f32::consts::TAU * if self.tumble > 0.0 { 1.0 } else { 0.0 } - self.meteor_spin,
                    face,
                    limbs,
                    trail: ((speed - 8.0) / 30.0).clamp(0.0, 1.0),
                    t,
                    pips: -1,
                    frozen: g.frozen > 0.0,
                    ball: g.meteor(),
                    rig: self.dog_anim.rig(),
                },
            );
        }
        if self.zap_flash > 0.0 && g.phase == Phase::Flying {
            // Two-beat electrified silhouette: white, then black.
            let white = ((self.zap_flash * 25.0) as i32) % 2 == 0;
            let tint = if white { Color::new(1.0, 1.0, 1.0, 0.95) } else { Color::new(0.05, 0.02, 0.1, 0.95) };
            draw::dog_echo(sp, hp, r, self.pilot_idx, limbs, face, tilt, tint);
        }
        draw::foreground(sp, cam, if g.phase == Phase::Flying { speed } else { 0.0 });

        self.particles.draw_front(sp, cam);
        if g.phase == Phase::Flying && g.flight_time > 0.4 {
            vfx::speed_lines(cam, vec2(g.vel.x, g.vel.y), hp, t);
        }
        // Off the lip the near clouds smear over the dog: keep them thin for a second.
        let cloud_cap = if g.phase == Phase::Flying && g.flight_time < 1.0 { 0.35 } else { 1.0 };
        draw::near_clouds(sp, cam, if g.phase == Phase::Flying { speed } else { 0.0 }, cloud_cap);
        draw::gusts(sp, cam, if g.phase == Phase::Flying { speed } else { 0.0 }, t);
        if !(g.phase == Phase::Flying && g.flight_time < 0.5) {
            let settle = if g.phase == Phase::Landed { (self.landed_for / hud::result_hold(g.crashed)).clamp(0.0, 1.0) } else { 0.0 };
            let calm = if g.phase == Phase::Flying && g.flight_time < 2.5 { 0.5 } else { 1.0 - 0.6 * settle };
            draw::snowfall(sp, cam, if g.phase == Phase::Flying { vec2(g.vel.x, g.vel.y) } else { Vec2::ZERO }, calm);
        }
        post::end(cam);
        if self.flash > 0.0 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(1.0, 1.0, 1.0, self.flash * 0.8));
        }
        if self.red_flash > 0.0 {
            // Red edge vignette fading to the centre.
            let (sw, sh) = (screen_width(), screen_height());
            let a = self.red_flash * 0.75;
            let bw = sw * 0.22;
            for i in 0..10 {
                let k = 1.0 - i as f32 / 10.0;
                let col = Color::new(0.95, 0.12, 0.12, a * k * k);
                let d = bw / 10.0;
                draw_rectangle(i as f32 * d, 0.0, d + 1.0, sh, col);
                draw_rectangle(sw - (i as f32 + 1.0) * d, 0.0, d + 1.0, sh, col);
                draw_rectangle(0.0, i as f32 * d * 0.5, sw, d * 0.5 + 1.0, Color::new(0.95, 0.12, 0.12, a * k * k * 0.6));
                draw_rectangle(0.0, sh - (i as f32 + 1.0) * d * 0.5, sw, d * 0.5 + 1.0, Color::new(0.95, 0.12, 0.12, a * k * k * 0.6));
            }
        }

        // Speed lines.
        for f in &self.fx {
            if hud::draw_fx(self, f, hp, r) {
                continue;
            }
            let k = f.age / f.life;
            let mut c = f.color;
            c.a = if k < 0.7 { 1.0 } else { (1.0 - k) / 0.3 };
            if f.banner {
                // One slot, low centre: a tilted ribbon that slides in from the
                // left, holds, then fades up and away.
                let slide = hud::ease_out_back(f.age / 0.12);
                let out = ((k - 0.7) / 0.3).clamp(0.0, 1.0);
                let a = 1.0 - out * out;
                // The ribbon lives in the sky band under the top HUD, never
                // over the ground, the pennants or the dog's landing.
                let size = 44;
                let d = measure_text(&f.text, Some(&self.display), size, 1.0);
                let (w, h) = (d.width + 96.0, 60.0);
                let cx = -w + (screen_width() / 2.0 + w) * slide;
                let cy = screen_height() * 0.24 - 20.0 * out;
                let tilt = -3.0f32.to_radians();
                let rib = hud::ribbon_color(f.color);
                hud::ribbon(cx, cy, w, h, rib, tilt, a);
                let (sn, cs) = (tilt.sin(), tilt.cos());
                let o = vec2(cx - d.width / 2.0 * cs + d.height / 2.0 * sn, cy - d.width / 2.0 * sn - d.height / 2.0 * cs);
                // Ink text on the pale ice ribbon, paper text on the strong ones.
                let on_ice = rib.b > 0.9 && rib.r < 0.8;
                let fill = if on_ice { hud::INK } else { hud::PAPER };
                let edge = if on_ice { hud::PAPER } else { hud::INK };
                let edge = Color::new(edge.r, edge.g, edge.b, a);
                for (dx, dy) in [(3.0, 0.0), (-3.0, 0.0), (0.0, 3.0), (0.0, -3.0), (2.0, 2.0), (-2.0, 2.0), (2.0, -2.0), (-2.0, -2.0)] {
                    draw_text_ex(&f.text, o.x + dx, o.y + dy, TextParams { font: Some(&self.display), font_size: size, rotation: tilt, color: edge, ..Default::default() });
                }
                draw_text_ex(&f.text, o.x, o.y, TextParams { font: Some(&self.display), font_size: size, rotation: tilt, color: Color::new(fill.r, fill.g, fill.b, a), ..Default::default() });
            } else {
                // Anchored above the dog's hat, drifting up, always fully on screen.
                let combo = f.text.contains('x');
                let zap = f.text == self.pack.labels.stormcloud;
                // Combo stamps slam in at three times size and snap to rest.
                let pop = if combo {
                    (if k < 0.1 { 3.0 - 2.0 * (k / 0.1) } else { 1.0 }) * (1.0 + 0.1 * self.combo as f32)
                } else if zap {
                    if k < 0.1 { 2.4 - 1.4 * (k / 0.1) } else { 1.0 }
                } else if k < 0.2 {
                    1.35 - 0.35 * (k / 0.2)
                } else {
                    1.0
                };
                let rank = f.text.starts_with('#');
                // Combo and zap stamps sit above the hat, never on the muzzle;
                // other stamps sit below the dog.
                let anchor = if rank { cam.to_screen(f.pos.x, f.pos.y) } else if combo || zap { hp + vec2(0.0, -r * 3.4) } else { hp + vec2(0.0, r * 2.4) };
                let x = anchor.x.clamp(130.0, screen_width() - 130.0);
                let size = if combo { 48.0 } else if zap { 64.0 } else { 40.0 };
                // Never into the HUD band, whatever the slam scale.
                let top = 100.0 + size * pop.min(3.0);
                // The rank stamp snaps DOWN onto its pennant; everything else drifts up.
                let drift = if rank { -90.0 * (1.0 - (1.0 - (k / 0.25).min(1.0)).powi(3)) + 90.0 } else { 60.0 * k };
                let y = (anchor.y - drift).clamp(top, screen_height() - 60.0);
                hud::text_shout(&self.display, &f.text, x, y, size as u16, pop.min(3.0), if combo || zap { hud::GOLD } else { WHITE }.with_alpha(c.a), hud::Align::Center);
            }
        }

        // Sky flicker: a storm within a second ahead darkens the frame in pulses.
        if g.phase == Phase::Flying {
            let soon = g.entities.iter().any(|e| {
                e.kind == EntityKind::Stormcloud && e.alive && e.pos.x > g.pos.x && e.pos.x - g.pos.x < g.vel.x.max(10.0) * 0.5 && (e.pos.y - g.pos.y).abs() < 7.0
            });
            if soon {
                // Only in the last half second before a strike, and only at
                // the leading edge: the rest of the frame stays clean.
                let beat = 0.3 + 0.3 * (t * 8.0).sin().max(0.0);
                let wdt = 70.0;
                for i in 0..8 {
                    let k = i as f32 / 8.0;
                    draw_rectangle(screen_width() - wdt + k * wdt, 0.0, wdt / 8.0 + 1.0, screen_height(), Color::new(0.95, 0.2, 0.2, beat * k * k));
                }
            }
        }
        // One pointer at the screen edge for the nearest storm cloud ahead.
        if g.phase == Phase::Flying {
            let nearest = g
                .entities
                .iter()
                .filter(|e| e.kind == EntityKind::Stormcloud && e.alive && e.pos.x > g.pos.x && e.pos.x - g.pos.x < 70.0)
                .min_by(|a, b| a.pos.x.partial_cmp(&b.pos.x).unwrap());
            if let Some(e) = nearest {
                let p = cam.to_screen(e.pos.x, e.pos.y);
                if p.x > screen_width() - 40.0 {
                    let y = p.y.clamp(100.0, screen_height() - 100.0);
                    let x = screen_width() - 48.0;
                    let pulse = 1.0 + (t * 8.0).sin() * 0.12;
                    draw_circle(x - 14.0, y + 4.0, 31.0, Color::new(hud::INK.r, hud::INK.g, hud::INK.b, 0.25));
                    draw_circle(x - 14.0, y, 31.0, hud::INK);
                    draw_circle(x - 14.0, y, 28.0, hud::PAPER);
                    sprites::blit(&sp.stormcloud, vec2(x - 14.0, y), 42.0 * pulse, 0.0, false, WHITE);
                    draw_triangle(vec2(x + 12.0, y - 15.0), vec2(x + 12.0, y + 15.0), vec2(x + 34.0, y), hud::INK);
                    draw_triangle(vec2(x + 16.0, y - 9.0), vec2(x + 16.0, y + 9.0), vec2(x + 29.0, y), hud::BERRY);
                }
            }
        }
        if !post_on {
            draw_texture_ex(&sp.vignette, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), ..Default::default() });
        }
        hud::render(self);
        orders::draw(self);
    }
}

/// Shell colours, cycled so a display is never one colour.
const FIREWORK_COLORS: [Color; 6] = [
    Color::new(1.0, 0.22, 0.28, 1.0),
    Color::new(1.0, 0.84, 0.28, 1.0),
    Color::new(0.30, 0.62, 1.0, 1.0),
    Color::new(0.35, 1.0, 0.50, 1.0),
    Color::new(1.0, 0.42, 0.85, 1.0),
    Color::new(0.88, 0.94, 1.0, 1.0),
];

impl App {
    /// Time-scale for this frame: the crash runs at a third speed and eases
    /// back to normal over `CRASH_SLOW` real seconds. Ticks the timer.
    fn slow_motion(&mut self, real_dt: f32) -> f32 {
        // The PERFECT beat: a fifth of speed off the lip, held a moment,
        // then easing back to full speed.
        let beat = if self.beat_slow > 0.0 {
            self.beat_slow = (self.beat_slow - real_dt).max(0.0);
            let k = (self.beat_slow / (BEAT_SLOW * 0.6)).min(1.0);
            0.2 + 0.8 * (1.0 - k) * (1.0 - k)
        } else {
            1.0
        };
        if self.crash_slow <= 0.0 {
            return beat;
        }
        self.crash_slow = (self.crash_slow - real_dt).max(0.0);
        let k = self.crash_slow / CRASH_SLOW;
        (0.3 + 0.7 * (1.0 - k) * (1.0 - k)).min(beat)
    }

    /// A fireworks display of `n` shells ahead of and above the dog, bursting
    /// in turn (records and milestones).
    fn firework_display(&mut self, n: usize) {
        let at = vec2(self.game.pos.x, self.game.pos.y.max(8.0));
        let lead = vec2(self.game.vel.x, 0.0) * 0.35;
        for i in 0..n {
            let fi = i as f32;
            let off = vec2(4.0 + 7.0 * fi - 3.0 * (i % 2) as f32, 5.0 + 4.0 * ((fi * 1.7).sin() + 1.0));
            let color = FIREWORK_COLORS[(self.shells.len() + i * 2 + 1) % FIREWORK_COLORS.len()];
            self.shells.push(Shell { pos: at + lead + off, delay: 0.35 + 0.32 * fi, color });
        }
    }

    /// Current present chain length, for the HUD counter.
    pub fn combo_now(&self) -> u32 {
        self.combo
    }
}

/// Width of the airborne dog sprite on screen for a given pilot radius.

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/Fredoka.ttf")).expect("bundled font");
    let display = load_ttf_font_from_bytes(include_bytes!("../assets/LilitaOne.ttf")).expect("bundled font");
    let pack = Pack::load_active().await;
    let sprites = Sprites::load(&pack.sprite_dir()).await;
    let city = City::load(&pack.pack_dir(), &pack.sprite_dir()).await;
    let mut sfx = Sfx::load(&pack.sound_dir()).await;
    sfx.set_music_enabled(load_json::<MusicSetting>(MUSIC_KEY).on);
    let mut app = App::new(font, display, sprites, pack, sfx, city);
    loop {
        app.handle_input();
        let real = get_frame_time();
        let dt = real * app.time_scale * app.slow_motion(real);
        app.cam_dt = real * app.time_scale;
        app.update(dt);
        app.render();
        next_frame().await;
    }
}
