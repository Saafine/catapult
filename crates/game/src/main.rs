//! macroquad runtime for the Christmas dog launch game.
//!
//! Everything platform-flavoured lives here: window, input, drawing, later
//! sound. The rules live in `hamsters-core`. Builds native and wasm from the
//! same source.

use hamsters_core::{EntityKind, Event, Game, Phase, LIP, PILOTS, PILOT_RADIUS, RAMP_TOP};
use macroquad::prelude::*;

mod draw;
mod hud;
mod pack;
mod particles;
mod sprites;

use draw::lerp;
use pack::Pack;
use serde::{Deserialize, Serialize};
use particles::Particles;
use sprites::Sprites;

/// Pixels per world meter at zoom 1.
pub const PPM: f32 = 30.0;
const FIXED_DT: f32 = 1.0 / 120.0;

fn window_conf() -> Conf {
    Conf { window_title: "Santa's Flight School".to_owned(), window_width: 1024, window_height: 576, high_dpi: true, ..Default::default() }
}

/// Camera that frames the tower while aiming, follows the sled down the ramp,
/// then tracks the dog with look-ahead and zooms out with speed.
pub struct Cam {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub shake: Vec2,
    pub shake_amp: f32,
    /// Launch punch: 1 right at the lip, decays to 0. Zooms in briefly.
    pub punch: f32,
}

impl Cam {
    fn target_for(g: &Game) -> (f32, f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        match g.phase {
            Phase::AimPower { .. } => {
                // Frame Santa on the tower top just under the HUD and keep a
                // strip of ground at the bottom: about 27 m of world height.
                let zoom = (sh / 27.5 / PPM).clamp(0.5, 1.0);
                let _ = sw;
                (RAMP_TOP.x * 0.5 + 2.0, 11.6, zoom)
            }
            Phase::Sliding { .. } => {
                // Tight on the sled: the dog fills about a fifth of the frame.
                let zoom = 1.15;
                let s = PPM * zoom;
                (g.pos.x + 0.14 * sw / s, (0.30 * sh / s + 1.0).max(g.pos.y - 0.02 * sh / s), zoom)
            }
            _ => {
                let speed = g.vel.length();
                // Stay tight for the first second after the lip, then ease out
                // with speed and altitude.
                // Hold the close framing for half a second after the lip, then
                // ease out over the next second.
                let fresh = (1.0 - ((g.flight_time - 0.5) / 1.0).clamp(0.0, 1.0)).powi(2);
                let wide = (1.0 / (1.0 + speed / 60.0 + g.pos.y / 170.0)).clamp(0.9, 1.0);
                let zoom = wide + (1.15 - wide) * fresh;
                let s = PPM * zoom;
                // The camera rises only half as fast as the dog, so the dog
                // visibly climbs towards the top of the frame at the apex and
                // sinks back on the way down; clamped so it never leaves the
                // safe band between the top HUD and the wiggle strip.
                let base_y = 0.36 * sh / s;
                // During the hold the dog sits about a third from the top;
                // afterwards the camera follows altitude loosely so the arc shows.
                let tight_y = g.pos.y - 0.02 * sh / s;
                // Vertical lead: while climbing the dog sits low in the frame
                // (48% from the top) so the sky ahead shows; while falling it
                // rides high (30%) so the ground ahead shows.
                let frac = 0.40 + (g.vel.y / 12.0).clamp(-1.0, 1.0) * 0.10;
                // Screen y grows downwards, so the camera centre sits below
                // the dog (smaller world y) to put the dog high in the frame.
                let loose_y = g.pos.y - (0.5 - frac) * sh / s;
                let follow_y = tight_y * fresh + loose_y * (1.0 - fresh);
                // Hard clamp: the dog's hat never enters the top HUD band.
                let min_y = g.pos.y + 2.6 - (0.5 * sh - 95.0) / s;
                let max_y = g.pos.y + (0.5 * sh - 150.0) / s;
                let lookahead = if g.phase == Phase::Flying { 0.30 * sw / s } else { 0.10 * sw / s };
                (g.pos.x + lookahead, base_y.max(follow_y).clamp(min_y, max_y), zoom)
            }
        }
    }

    fn follow(&mut self, g: &Game, dt: f32) {
        let (tx, ty, tz) = Self::target_for(g);
        let k = 1.0 - (-dt * 7.0).exp();
        // Vertical follow lags a little so each wiggle kick hops the dog on screen.
        let ky = 1.0 - (-dt * 6.0).exp();
        self.x += (tx - self.x) * k;
        self.y += (ty - self.y) * ky;
        // Hard clamp after smoothing: the dog's hat never enters the HUD band.
        if matches!(g.phase, Phase::Flying | Phase::Sliding { .. }) {
            let s = PPM * self.zoom;
            let min_y = g.pos.y + 2.8 - (0.5 * screen_height() - 100.0) / s;
            if self.y < min_y {
                self.y = min_y;
            }
        }
        self.punch = (self.punch - dt * 2.0).max(0.0);
        // Punch eases in and out rather than snapping.
        let env = (self.punch * std::f32::consts::PI).sin().max(0.0);
        let tz = tz * (1.0 + 0.10 * env);
        self.zoom += (tz - self.zoom) * k;
        self.shake_amp = (self.shake_amp - dt * 30.0).max(0.0);
        let t = get_time() as f32;
        self.shake = vec2((t * 61.0).sin(), (t * 47.0).cos()) * self.shake_amp;
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

fn load_scores() -> Vec<Score> {
    let raw = quad_storage::STORAGE.lock().ok().and_then(|s| s.get(SCORES_KEY));
    raw.and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_default()
}

fn save_scores(scores: &[Score]) {
    if let Ok(json) = serde_json::to_string(scores) {
        if let Ok(mut s) = quad_storage::STORAGE.lock() {
            s.set(SCORES_KEY, &json);
        }
    }
}

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
    /// Dev-only: jump automatically at the perfect moment after a P launch.
    pub auto_jump: bool,
    /// Best distances so far, longest first, with the pilot that flew them.
    pub scores: Vec<Score>,
    was_frozen: bool,
    pub squash: f32,
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
    /// When the run ended, for the results tick-up.
    pub landed_at: f64,
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
    /// Seconds the stamina arc stays visible after a wiggle.
    arc_show: f32,
    pub font: Font,
    pub display: Font,
    pub sprites: Sprites,
    pub pack: Pack,
}

impl App {
    fn new(font: Font, display: Font, sprites: Sprites, pack: Pack) -> Self {
        let seed = (get_time() * 1e6) as u64 | 1;
        let mut game = Game::new(PILOTS[0], seed);
        let scores = load_scores();
        if let Some(top) = scores.first() {
            game.best_distance = top.distance;
        }
        let (x, y, zoom) = Cam::target_for(&game);
        App {
            game,
            cam: Cam { x, y, zoom, shake: Vec2::ZERO, shake_amp: 0.0, punch: 0.0 },
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
            auto_jump: false,
            scores,
            was_frozen: false,
            squash: 0.0,
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
            landed_at: 0.0,
            boomed: false,
            craters: Vec::new(),
            reaction: None,
            time_scale: 1.0,
            wiggle_pulse: 0.0,
            counter_pulse: 0.0,
            arc_show: 0.0,
            font,
            display,
            sprites,
            pack,
        }
    }

    fn handle_input(&mut self) {
        #[allow(unused_mut)]
        let mut action = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Up)
            || is_mouse_button_pressed(MouseButton::Left)
            || touches().iter().any(|t| t.phase == TouchPhase::Started);

        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::T) {
            // Slow motion for frame-by-frame capture.
            self.time_scale = if self.time_scale < 1.0 { 1.0 } else { 0.25 };
        }
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::P) {
            if let Phase::AimPower { .. } = self.game.phase {
                self.auto_jump = true;
                self.game.phase = Phase::Sliding { power: 0.9, t: 0.0, jump: None };
                self.game.action();
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Phase::AimPower { .. } = self.game.phase {
                let (mx, my) = mouse_position();
                if let Some(i) = hud::picker_hit(vec2(mx, my)) {
                    self.pilot_idx = i;
                    self.game.set_pilot(PILOTS[i]);
                    action = false;
                }
            }
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
        let mut restart = is_key_pressed(KeyCode::R) && !matches!(self.game.phase, Phase::Flying | Phase::Sliding { .. });
        if self.game.phase == Phase::Landed && get_time() - self.landed_at > 0.5 {
            let (mx, my) = mouse_position();
            let clicked = is_mouse_button_pressed(MouseButton::Left) || touches().iter().any(|t| t.phase == TouchPhase::Started);
            let touch_at = touches().first().map(|t| t.position);
            let at = touch_at.unwrap_or(vec2(mx, my));
            if (clicked && hud::restart_button_rect().contains(at)) || is_key_pressed(KeyCode::Enter) {
                restart = true;
            }
        }
        if restart {
            self.game.reset();
            self.loose_sled = None;
            self.craters.clear();
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
            self.pilot_idx = i;
            self.game.set_pilot(PILOTS[i]);
        }
    }

    fn update(&mut self, frame_dt: f32) {
        if self.hitstop > 0.0 {
            self.hitstop -= frame_dt;
        } else {
            self.acc += frame_dt.min(0.1);
            while self.acc >= FIXED_DT {
                self.game.step(FIXED_DT);
                self.acc -= FIXED_DT;
            }
        }
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
            if next.banner {
                self.cam.shake_amp = self.cam.shake_amp.max(4.0);
            }
            self.fx.push(next);
        }
        // Drop stale queued labels so a burst of pickups doesn't replay for seconds.
        if self.banner_queue.len() > 3 {
            let keep: Vec<Fx> = self.banner_queue.drain(..).filter(|f| f.banner).collect();
            self.banner_queue = keep;
        }
        let events: Vec<Event> = self.game.events.drain(..).collect();
        for ev in events {
            self.on_event(ev);
        }
        self.cam.follow(&self.game, frame_dt);
        self.particles.update(frame_dt);

        self.flap_anim = (self.flap_anim - frame_dt * 3.3).max(0.0);
        self.limp_flap = (self.limp_flap - frame_dt * 4.0).max(0.0);
        self.boost_echo = (self.boost_echo - frame_dt).max(0.0);
        self.pickup_pop = (self.pickup_pop - frame_dt * 8.0).max(0.0);
        self.zap_flash = (self.zap_flash - frame_dt).max(0.0);
        self.dist_pulse = (self.dist_pulse - frame_dt * 5.0).max(0.0);
        let frozen_now = self.game.frozen > 0.0;
        if self.was_frozen && !frozen_now {
            let at = vec2(self.game.pos.x, self.game.pos.y);
            self.particles.emit_confetti(at, 24);
            self.particles.emit_sparkles(at, 16);
            self.particles.emit_ring(at, Color::new(0.7, 0.9, 1.0, 1.0));
        }
        self.was_frozen = frozen_now;
        if self.game.crashed {
            self.crash_sink = (self.crash_sink + frame_dt * 2.8).min(1.0);
        }
        if self.game.phase == Phase::Flying {
            let mark = (self.game.distance() / 10.0) as i32;
            if mark > self.last_mark {
                self.last_mark = mark;
                self.dist_pulse = 1.0;
            }
        } else if !matches!(self.game.phase, Phase::Landed) {
            self.last_mark = 0;
        }
        self.wiggle_pulse = (self.wiggle_pulse - frame_dt * 12.0).max(0.0);
        self.counter_pulse = (self.counter_pulse - frame_dt * 4.0).max(0.0);
        self.arc_show = (self.arc_show - frame_dt).max(0.0);
        self.squash = (self.squash - frame_dt * 5.0).max(0.0);
        self.tumble = (self.tumble - frame_dt * 1.6).max(0.0);
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
                self.particles.emit_snow(hp - vec2(0.6, 1.1), vec2(-g.vel.x * 0.2, 3.0), 2);
            }
        }
    }

    fn on_event(&mut self, ev: Event) {
        let g = &self.game;
        let at = vec2(g.pos.x, g.pos.y);
        match ev {
            Event::Pushed => {
                self.particles.emit_snow(vec2(RAMP_TOP.x, RAMP_TOP.y + 0.3), vec2(-2.0, 2.0), 6);
            }
            Event::Jumped { quality } => {
                let l = &self.pack.labels;
                let (text, color) = if quality >= hamsters_core::JUMP_PERFECT_FROM {
                    (l.jump_perfect.clone(), Color::new(1.0, 0.86, 0.3, 1.0))
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
                    self.particles.emit_sparkles(at, 30);
                    self.particles.emit_confetti(at, 16);
                }
            }
            Event::Launched => {
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
                self.pickup_pop = 0.4;
                self.best_before = g.best_distance;
                self.boomed = false;
                // The sled keeps going without the dog.
                self.loose_sled = Some(LooseSled { pos: vec2(LIP.x, LIP.y + 0.4), vel: vec2(g.vel.x * 0.45, g.vel.y * 0.4), rot: 0.0, spin: -6.0 });
            }
            Event::Flap => {
                self.flap_anim = 1.0;
                self.wiggle_pulse = 1.0;
                self.arc_show = 1.2;
                self.particles.emit_puffs(at - vec2(0.0, 1.6), 16, Color::new(1.0, 1.0, 1.0, 0.95));
                self.particles.emit_snow(at - vec2(0.0, 1.2), vec2(0.0, -7.0), 14);
                self.particles.emit_sparkles(at + vec2(0.0, 0.8), 4);
            }
            Event::WigglePower => {
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
                    EntityKind::Trampoline => {
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
                    }
                    EntityKind::Firework => {
                        self.particles.emit_ring(at, color);
                        self.cam.shake_amp = 7.0;
                        self.cam.punch = 0.7;
                    }
                    EntityKind::Wings => self.particles.emit_sparkles(at, 12),
                    EntityKind::ColdWind => {
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
            Event::GroundBounce { speed } => {
                if speed > 6.0 {
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
            Event::Crashed { speed } => {
                self.fx.retain(|f| !f.banner);
                self.banner_queue.clear();
                self.fx.push(Fx { pos: vec2(at.x, 2.5), age: 0.0, life: 1.2, color: Color::new(1.0, 0.92, 0.2, 1.0), text: self.pack.labels.crash.clone(), banner: true });
                self.particles.emit_snow(vec2(at.x, 0.2), vec2(0.0, 9.0), 60);
                self.particles.emit_clods(vec2(at.x, 0.2), vec2(0.0, 10.0), 40);
                self.particles.emit_ring(vec2(at.x, 0.5), WHITE);
                self.craters.push((at.x, 1.4));
                self.hitstop = 0.12;
                self.flash = 0.6;
                self.cam.shake_amp = (speed * 0.6).min(20.0);
                self.crash_sink = 0.0;
            }
            Event::Milestone(i) => {
                let name = self.pack.milestone(i);
                self.particles.emit_confetti(at, 40);
                self.particles.emit_sparkles(at, 20);
                self.banner_queue.push(Fx { pos: at, age: 0.0, life: 2.0, color: Color::new(0.75, 0.95, 1.0, 1.0), text: name.to_owned(), banner: true });
                self.cam.shake_amp = 4.0;
            }
            Event::Landed { .. } => {
                self.particles.emit_snow(vec2(at.x, 0.2), vec2(0.0, 2.0), 8);
                self.landed_at = get_time();
                self.bests[self.pilot_idx] = self.bests[self.pilot_idx].max(g.distance());
                self.scores.push(Score { distance: g.distance(), pilot: self.pilot_idx, crashed: g.crashed });
                self.scores.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap_or(std::cmp::Ordering::Equal));
                self.scores.truncate(10);
                save_scores(&self.scores);
            }
        }
        // Craters where the dog hit the snow.
        if let Event::GroundBounce { speed } = ev {
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

        draw::sky(sp, cam);
        draw::clouds(sp, cam);
        draw::mountains(sp, cam);
        draw::hills(sp, cam, if g.phase == Phase::Flying { g.vel.length() } else { 0.0 });
        draw::midground(sp, cam);
        if g.crashed {
            // Head-first in the snow: drawn before the ground so it disappears into it.
            let k = self.crash_sink;
            let ease = 1.0 - (1.0 - k) * (1.0 - k);
            draw::dog_buried(sp, cam, g.pos.x, self.pilot_idx, ease);
        }
        draw::ground(sp, cam, &self.font, &self.pack.towns);
        for (cx, size) in &self.craters {
            let p = cam.to_screen(*cx, 0.0);
            let sc = cam.scale();
            draw_ellipse(p.x, p.y + 0.12 * sc, 2.8 * sc * size, 0.7 * sc * size, 0.0, Color::new(0.5, 0.57, 0.82, 0.6));
            draw_ellipse(p.x, p.y + 0.02 * sc, 2.1 * sc * size, 0.45 * sc * size, 0.0, Color::new(0.78, 0.83, 0.96, 0.85));
            draw_ellipse(p.x - 1.2 * sc * size, p.y - 0.15 * sc, 0.7 * sc * size, 0.3 * sc * size, 0.0, Color::new(1.0, 1.0, 1.0, 0.9));
            draw_ellipse(p.x + 1.2 * sc * size, p.y - 0.15 * sc, 0.7 * sc * size, 0.3 * sc * size, 0.0, Color::new(1.0, 1.0, 1.0, 0.9));
        }
        draw::landing_zone(cam, g.landing_x, t);
        draw::spectators(sp, cam, g.pos.x, t);
        draw::start_area(sp, cam, &self.display, &self.pack.tower_sign);

        for e in &g.entities {
            draw::entity(sp, cam, e, t, if g.phase == Phase::Flying { Some((vec2(g.pos.x, g.pos.y), vec2(g.vel.x, g.vel.y))) } else { None });
        }

        let speed = g.vel.length();
        let n = self.trail.len();
        if speed > 12.0 {
            for (i, p) in self.trail.iter().enumerate() {
                let k = (i + 1) as f32 / (n + 1) as f32;
                let sp_ = cam.to_screen(p.x, p.y);
                let a = 0.22 * k * ((speed - 12.0) / 30.0).clamp(0.0, 1.0);
                draw_circle(sp_.x, sp_.y, PILOT_RADIUS * cam.scale() * 0.6 * k, Color::new(1.0, 1.0, 1.0, a));
            }
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
            (((speed - 12.0) / 90.0).clamp(0.0, 0.18) + self.launch_stretch * 0.08).min(0.25) - self.wiggle_pulse * 0.32 - self.squash * 0.22 + self.pickup_pop * 0.14
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
            (draw::Face::Whee, draw::Limbs::Seated)
        } else if g.phase == Phase::Flying {
            let (face, body) = if g.frozen > 0.0 {
                (draw::Face::Panic, draw::Limbs::Land)
            } else if g.flight_time < 0.6 {
                (draw::Face::Scream, draw::Limbs::Spread)
            } else if self.tumble > 0.0 {
                (draw::Face::Dizzy, draw::Limbs::Tumble)
            } else if let Some((face, _)) = self.reaction {
                (face, if speed > 30.0 { draw::Limbs::Dive } else { draw::Limbs::Spread })
            } else if self.squash > 0.5 {
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
            let gp = cam.to_screen(g.pos.x, 0.0);
            let s = cam.scale();
            draw_ellipse(gp.x, gp.y, 1.9 * s, 0.5 * s, 0.0, Color::new(0.2, 0.18, 0.3, 0.95));
            draw_ellipse(gp.x, gp.y - 0.1 * s, 1.7 * s, 0.38 * s, 0.0, Color::new(0.06, 0.05, 0.12, 1.0));
            // Paws and tail above the rim: redraw the buried body clipped to above the hole.
            if self.crash_sink >= 1.0 {
                draw::dog_paws_up(sp, cam, g.pos.x, self.pilot_idx);
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
                    spin: -(1.0 - self.tumble) * std::f32::consts::TAU * if self.tumble > 0.0 { 1.0 } else { 0.0 },
                    face,
                    limbs,
                    trail: ((speed - 8.0) / 30.0).clamp(0.0, 1.0),
                    t,
                    pips: -1,
                    frozen: g.frozen > 0.0,
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
        draw::near_clouds(sp, cam, if g.phase == Phase::Flying { speed } else { 0.0 });
        draw::gusts(sp, cam, if g.phase == Phase::Flying { speed } else { 0.0 }, t);
        if !(g.phase == Phase::Flying && g.flight_time < 0.5) {
            let calm = if g.phase == Phase::Flying && g.flight_time < 2.5 { 0.5 } else { 1.0 };
            draw::snowfall(sp, cam, if g.phase == Phase::Flying { vec2(g.vel.x, g.vel.y) } else { Vec2::ZERO }, calm);
        }
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
            let k = f.age / f.life;
            let mut c = f.color;
            c.a = if k < 0.7 { 1.0 } else { (1.0 - k) / 0.3 };
            if f.banner {
                // One slot, top-centre under the presents counter, on a plate; slams in.
                let pop = if k < 0.12 { 1.9 - 0.9 * (k / 0.12) } else { 1.0 };
                let y = screen_height() * 0.80;
                let d = measure_text(&f.text, Some(&self.display), 52, pop);
                hud::rounded(screen_width() / 2.0 - d.width / 2.0 - 18.0, y - d.height - 6.0, d.width + 36.0, d.height + 20.0, 12.0, Color::new(0.1, 0.06, 0.14, 0.55 * c.a.min(1.0)));
                hud::text_outlined_scaled(&self.display, &f.text, screen_width() / 2.0, y, 52, pop, c, hud::Align::Center);
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
                // Combos rise from where the present was; other stamps sit below the dog.
                let anchor = if combo { hp + vec2(-r * 2.2, -r * 2.6) } else if zap { hp + vec2(0.0, -r * 3.4) } else { hp + vec2(0.0, r * 2.4) };
                let x = anchor.x.clamp(130.0, screen_width() - 130.0);
                let size = if combo { 48.0 } else if zap { 64.0 } else { 40.0 };
                // Never into the HUD band, whatever the slam scale.
                let top = 100.0 + size * pop.min(3.0);
                let y = (anchor.y - 60.0 * k).clamp(top, screen_height() - 60.0);
                hud::text_outlined_scaled(&self.display, &f.text, x, y, size as u16, pop.min(3.0), c, hud::Align::Center);
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
                    hud::rounded(x - 44.0, y - 24.0, 62.0, 48.0, 10.0, Color::new(0.1, 0.06, 0.14, 0.7));
                    sprites::blit(&sp.stormcloud, vec2(x - 16.0, y), 44.0 * pulse, 0.0, false, WHITE);
                    draw_triangle(vec2(x + 18.0, y - 12.0), vec2(x + 18.0, y + 12.0), vec2(x + 34.0, y), Color::new(0.95, 0.25, 0.25, 1.0));
                }
            }
        }
        draw_texture_ex(&sp.vignette, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), ..Default::default() });
        hud::render(self);
    }
}

impl App {
    /// Current present chain length, for the HUD counter.
    pub fn combo_now(&self) -> u32 {
        self.combo
    }
}

/// Width of the airborne dog sprite on screen for a given pilot radius.

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/Poppins-SemiBold.ttf")).expect("bundled font");
    let display = load_ttf_font_from_bytes(include_bytes!("../assets/Poppins-Bold.ttf")).expect("bundled font");
    let pack = Pack::load_active().await;
    let sprites = Sprites::load(&pack.sprite_dir()).await;
    let mut app = App::new(font, display, sprites, pack);
    loop {
        app.handle_input();
        let dt = get_frame_time() * app.time_scale;
        app.update(dt);
        app.render();
        next_frame().await;
    }
}
