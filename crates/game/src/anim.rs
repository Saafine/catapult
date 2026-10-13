//! Procedural animation for the dog rig and the bystanders: damped springs
//! for the head, ears, lean and squash/stretch, a simulated cape chain that
//! whips in the airstream, anticipation on the sled and follow-through on
//! every kick. Presentation only: it reads the game state, never writes it.
//!
//! `DogAnim` is stepped once per frame from `App::update`; `DogAnim::rig`
//! hands `draw::dog` a plain `Rig` of offsets to apply to the sprites.

use hamsters_core::{Game, Phase};
use macroquad::prelude::*;

use crate::sprites::blit_foot;

/// Damped spring on one value. `freq` is in Hz, `damp` the damping ratio:
/// below 1 it overshoots and wobbles, which is where follow-through lives.
#[derive(Clone, Copy, Default)]
pub struct Spring {
    pub x: f32,
    pub v: f32,
}

impl Spring {
    pub fn new(x: f32) -> Self {
        Self { x, v: 0.0 }
    }

    pub fn step(&mut self, target: f32, freq: f32, damp: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let w = std::f32::consts::TAU * freq;
        let n = ((dt * w / 0.4).ceil() as i32).clamp(1, 12);
        let h = dt / n as f32;
        for _ in 0..n {
            let a = w * w * (target - self.x) - 2.0 * damp * w * self.v;
            self.v += a * h;
            self.x += self.v * h;
        }
    }
}

/// Points along the cape, collar first.
pub const CAPE_N: usize = 9;
/// Cape length in figure widths.
const CAPE_LEN: f32 = 0.82;
/// Figure width in meters on the sled (`draw::DOG_W` times the pilot radius);
/// in flight it grows by `draw::DOG_FLY_SCALE`.
const FIG_W: f32 = 3.3;
const FLY_GROW: f32 = 0.8;

/// What `draw::dog` needs from the animation this frame. All offsets are
/// relative to the pose the game picked; `on == false` draws the old rig.
#[derive(Clone, Copy)]
pub struct Rig {
    pub on: bool,
    /// 0 on the sled, 1 in flight (overshoots on the launch pop).
    pub grow: f32,
    /// Extra body rotation, radians, counter-clockwise.
    pub lean: f32,
    /// Extra squash/stretch along the heading (multipliers).
    pub sx: f32,
    pub sy: f32,
    /// Whole-figure lift in figure widths (sled rattle, breathing).
    pub hop: f32,
    /// Extra head rotation and offset (256-canvas px, x forward, y down).
    pub head: f32,
    pub head_off: Vec2,
    /// Ear angles relative to the head: 0 = straight out like wings,
    /// positive hangs down, negative sweeps up and back.
    pub ear_l: f32,
    pub ear_r: f32,
    /// Ear length multiplier: the wind pulls them long.
    pub ear_len: f32,
    /// Cape chain from the collar, in figure widths, y up.
    pub cape: [Vec2; CAPE_N],
    /// Cape flutter speed, for the hem ripple.
    pub flutter: f32,
}

impl Default for Rig {
    fn default() -> Self {
        Self {
            on: false,
            grow: 0.0,
            lean: 0.0,
            sx: 1.0,
            sy: 1.0,
            hop: 0.0,
            head: 0.0,
            head_off: Vec2::ZERO,
            ear_l: 1.25,
            ear_r: 1.25,
            ear_len: 1.0,
            cape: [Vec2::ZERO; CAPE_N],
            flutter: 0.0,
        }
    }
}

pub struct DogAnim {
    grow: Spring,
    squash: Spring,
    lean: Spring,
    head: Spring,
    head_y: Spring,
    head_x: Spring,
    ear_l: Spring,
    ear_r: Spring,
    ear_len: Spring,
    cape: [Vec2; CAPE_N],
    cape_prev: [Vec2; CAPE_N],
    last_vel: Vec2,
    clock: f32,
    blink: f32,
    hop: f32,
    flutter: f32,
    /// 0..1: how coiled the dog is on the sled before the lip.
    crouch: Spring,
}

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let k = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

impl DogAnim {
    pub fn new() -> Self {
        let mut cape = [Vec2::ZERO; CAPE_N];
        for (i, c) in cape.iter_mut().enumerate() {
            *c = vec2(-0.8, -0.6) * (CAPE_LEN * i as f32 / (CAPE_N - 1) as f32);
        }
        Self {
            grow: Spring::new(0.0),
            squash: Spring::new(0.0),
            lean: Spring::new(0.0),
            head: Spring::new(0.0),
            head_y: Spring::new(0.0),
            head_x: Spring::new(0.0),
            ear_l: Spring::new(1.25),
            ear_r: Spring::new(1.25),
            ear_len: Spring::new(1.0),
            cape,
            cape_prev: cape,
            last_vel: Vec2::ZERO,
            clock: 0.0,
            blink: 2.0,
            hop: 0.0,
            flutter: 0.0,
            crouch: Spring::new(0.0),
        }
    }

    /// A wiggle press: anticipation squash, the body pitches up, the head
    /// nods and the ears get thrown into the up-stroke.
    pub fn flap(&mut self, lift: f32) {
        let k = 0.7 + 0.3 * lift.min(2.0);
        self.squash.v -= 4.5 * k;
        self.lean.v += 5.0 * k;
        self.head.v -= 5.0 * k;
        self.head_y.v += 120.0 * k;
        self.ear_l.v -= 22.0 * k;
        self.ear_r.v -= 22.0 * k;
    }

    /// Leaving the lip: a long stretch along the take-off, the head whips
    /// back, the ears and cape are flung behind, and the figure pops up to
    /// its flying size with an overshoot.
    pub fn launch(&mut self, perfect: bool) {
        let k = if perfect { 1.25 } else { 0.9 };
        self.squash.x = 0.0;
        self.squash.v = 9.0 * k;
        self.grow.v = 7.0 * k;
        self.head.v = 6.0 * k;
        self.head_x.v = -260.0 * k;
        self.lean.v = 3.0 * k;
        self.ear_l.v -= 30.0 * k;
        self.ear_r.v -= 30.0 * k;
        self.crouch = Spring::new(0.0);
    }

    /// A hit or a bounce: squash flat by `k` (0..1), the head drops in.
    pub fn impact(&mut self, k: f32) {
        let k = k.clamp(0.0, 1.5);
        self.squash.v -= 9.0 * k;
        self.head_y.v += 260.0 * k;
        self.head.v -= 4.0 * k;
        self.ear_l.v += 18.0 * k;
        self.ear_r.v += 18.0 * k;
    }

    /// A small pop: the dog swells for a present.
    pub fn pop(&mut self) {
        self.squash.v += 2.5;
        self.ear_l.v -= 8.0;
        self.ear_r.v -= 8.0;
    }

    /// Shown briefly with eyes half shut, on the title and the sled.
    pub fn blinking(&self) -> bool {
        self.blink < 0.0
    }

    /// `flap` is the 1..0 wiggle animation timer; `dt` is game time
    /// (respects slow motion) and 0 during hit-stop, so a freeze frame
    /// freezes the rig too.
    pub fn update(&mut self, g: &Game, flap: f32, tumble: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let dt = dt.min(0.1);
        self.clock += dt;
        let c = self.clock;
        let vel = vec2(g.vel.x, g.vel.y);
        let speed = vel.length();
        let sk = ((speed - 4.0) / 26.0).clamp(0.0, 1.0);
        let flying = g.phase == Phase::Flying;
        let fall = ((-vel.y - 6.0) / 24.0).clamp(0.0, 1.0);

        // Blinks: every 2.5..4.5 s, 0.13 s shut.
        self.blink -= dt;
        if self.blink < -0.13 {
            self.blink = 2.5 + 2.0 * (c * 7.31).sin().abs();
        }

        let (grow_t, sq_t, lean_t, mut head_t, hy_t, hx_t);
        let (mut el_t, mut er_t, mut len_t);
        let mut hop = 0.0;
        let air: Vec2;
        let mut flutter_amp = 2.0 + 34.0 * sk;
        let mut flutter_rate = 7.0 + 13.0 * sk;
        match g.phase {
            Phase::AimPower { .. } | Phase::Landed => {
                // Idle life: breathing, a slow look around, ears swaying, a
                // breeze in the cape.
                let breathe = (c * 2.3).sin();
                grow_t = 0.0;
                sq_t = 0.025 * breathe;
                lean_t = 0.04 * (c * 0.9).sin();
                let glance = smooth(0.6, 0.9, (c * 0.23).sin());
                head_t = 0.07 * (c * 0.7).sin() + 0.18 * glance;
                hy_t = -4.0 * breathe;
                hx_t = 0.0;
                el_t = 1.2 + 0.10 * (c * 1.3).sin();
                er_t = 1.2 + 0.10 * (c * 1.1 + 1.0).sin();
                len_t = 1.0;
                hop = 0.006 * breathe;
                air = vec2(-0.35 - 0.35 * (c * 0.8).sin().max(0.0), 0.0);
                flutter_amp = 1.2;
                flutter_rate = 4.0;
                self.crouch.step(0.0, 3.0, 1.0, dt);
            }
            Phase::Sliding { t, jump, .. } => {
                // Riding: leaning into the slope, rattled by the runners,
                // ears and cape streaming; coiling down over the last stretch
                // and fully coiled once the jump is pressed.
                let coil = if jump.is_some() { 1.0 } else { 0.85 * smooth(0.62, 0.97, t) };
                self.crouch.step(coil, 5.0, 0.6, dt);
                let cr = self.crouch.x;
                grow_t = 0.0;
                sq_t = 0.05 - 0.20 * cr;
                lean_t = -0.10 - 0.16 * cr;
                head_t = -0.06 - 0.14 * cr + 0.05 * (c * 13.0).sin() * sk;
                hy_t = 18.0 * cr;
                hx_t = 10.0 * cr;
                let wind = smooth(0.0, 0.7, sk);
                el_t = 1.2 + (0.05 - 1.2) * wind + 0.18 * (c * 23.0).sin() * wind;
                er_t = 1.2 + (-2.8 - 1.2) * wind + 0.2 * (c * 19.0 + 1.0).sin() * wind;
                len_t = 1.0 + 0.25 * wind;
                let rattle = (c * 53.0).sin() * 0.6 + (c * 37.0 + 1.0).sin() * 0.4;
                hop = 0.012 * rattle * sk.max(0.2) * (1.0 - 0.7 * cr);
                air = -vel / FIG_W;
            }
            Phase::Flying => {
                grow_t = 1.0;
                // Stretch along the heading in a dive.
                sq_t = 0.16 * fall + 0.05 * sk;
                lean_t = 0.0;
                // Head looks where it's going: tucked into a dive, up on a climb.
                head_t = -0.22 * fall + 0.12 * ((vel.y - 4.0) / 12.0).clamp(0.0, 1.0);
                hy_t = 0.0;
                hx_t = 14.0 * fall;
                // Ears: out like wings, fluttering; one beat per wiggle; swept
                // back by the airstream at speed.
                let u = (1.0 - flap).clamp(0.0, 1.0);
                let stroke = if flap > 0.0 { -1.15 * (u * std::f32::consts::TAU).sin() } else { 0.0 };
                let flut = (c * 9.0).sin() * 0.10 + (c * 23.0).sin() * 0.05 * sk;
                // A wiggle spreads the ears back into wings for the beat.
                let beat = if flap > 0.0 { 1.0 - 0.85 * smooth(0.0, 0.3, flap) } else { 1.0 };
                let stream = smooth(24.0, 44.0, speed).max(fall * 0.8).max(if g.rocket > 0.0 { 0.9 } else { 0.0 }) * beat;
                el_t = (-0.15 + stroke) * (1.0 - stream) + (-0.05 + 0.12 * (c * 27.0).sin()) * stream + flut;
                er_t = (-0.15 + stroke) * (1.0 - stream) + (-2.7 + 0.15 * (c * 31.0).sin()) * stream + flut;
                len_t = 1.0 + 0.2 * stream;
                if g.frozen > 0.0 {
                    el_t = 0.5;
                    er_t = 0.5;
                    len_t = 0.95;
                    flutter_amp *= 0.2;
                } else if tumble > 0.0 || g.botched {
                    el_t = 0.3 + 1.3 * (c * 15.0).sin();
                    er_t = 0.3 + 1.3 * (c * 17.0 + 2.0).sin();
                    head_t = 0.25 * (c * 11.0).sin();
                }
                air = -vel / (FIG_W * (1.0 + FLY_GROW));
                self.crouch.step(0.0, 4.0, 1.0, dt);
            }
        }

        self.grow.step(grow_t, 2.6, 0.38, dt);
        self.squash.step(sq_t, 3.6, 0.32, dt);
        self.lean.step(lean_t, 3.0, 0.45, dt);
        self.head.step(head_t, 3.4, 0.35, dt);
        self.head_y.step(hy_t, 4.0, 0.35, dt);
        self.head_x.step(hx_t, 3.5, 0.4, dt);
        self.ear_l.step(el_t, 6.2, 0.28, dt);
        self.ear_r.step(er_t, 5.6, 0.30, dt);
        self.ear_len.step(len_t, 3.0, 0.5, dt);
        self.hop = hop;
        self.flutter = flutter_rate;

        // Cape: a verlet chain in the dog's moving frame. The airstream
        // drags every link back, the frame's own acceleration (a flap, the
        // launch, a bounce) throws the cloth the other way, and a travelling
        // flutter ripples it towards the hem.
        let fig = FIG_W * (1.0 + FLY_GROW * self.grow.x.clamp(0.0, 1.3));
        let vfig = vel / fig;
        let mut dv = vfig - self.last_vel;
        if dv.length() > 2.5 {
            dv = dv.normalize() * 2.5;
        }
        self.last_vel = vfig;
        // Cloth falls with the dog in flight (the frame's own fall cancels
        // it); on the sled it hangs, a little floaty.
        let grav = vec2(0.0, -9.8 / fig) * if flying { 1.0 } else { 0.55 };
        let n = ((dt / (1.0 / 120.0)).ceil() as i32).clamp(1, 12);
        let h = dt / n as f32;
        let drag = 9.0;
        let seg = CAPE_LEN / (CAPE_N - 1) as f32;
        for i in 1..CAPE_N {
            // The frame's velocity change, spread over this frame.
            self.cape_prev[i] += dv * h;
        }
        for step in 0..n {
            let tt = c - dt + h * (step as f32 + 1.0);
            for i in 1..CAPE_N {
                let q = self.cape[i];
                let v = (q - self.cape_prev[i]) / h;
                let k = i as f32 / (CAPE_N - 1) as f32;
                // Flutter across the chain direction.
                let along = (q - self.cape[i - 1]).normalize_or_zero();
                let side = vec2(-along.y, along.x);
                let fl = (tt * flutter_rate - k * 4.2).sin() * flutter_amp * k;
                let a = (air - v) * drag + side * fl + grav;
                self.cape_prev[i] = q;
                self.cape[i] = q + v * h * 0.995 + a * h * h;
            }
            self.cape[0] = Vec2::ZERO;
            for _ in 0..4 {
                for i in 1..CAPE_N {
                    let d = self.cape[i] - self.cape[i - 1];
                    let l = d.length().max(1e-5);
                    let corr = d * ((l - seg) / l);
                    if i == 1 {
                        self.cape[i] -= corr;
                    } else {
                        self.cape[i] -= corr * 0.5;
                        self.cape[i - 1] += corr * 0.5;
                    }
                }
            }
        }
    }

    pub fn rig(&self) -> Rig {
        let s = self.squash.x.clamp(-0.4, 0.45);
        Rig {
            on: true,
            grow: self.grow.x.clamp(-0.1, 1.35),
            lean: self.lean.x.clamp(-0.6, 0.6),
            sx: 1.0 + s,
            sy: 1.0 - 0.8 * s,
            hop: self.hop,
            head: self.head.x.clamp(-0.7, 0.7),
            head_off: vec2(self.head_x.x.clamp(-60.0, 40.0), self.head_y.x.clamp(-40.0, 50.0)),
            ear_l: self.ear_l.x,
            ear_r: self.ear_r.x,
            ear_len: self.ear_len.x.clamp(0.8, 1.4),
            cape: self.cape,
            flutter: self.flutter,
        }
    }
}

/// Santa on the start platform: rocking his waving arm, breathing, and
/// hopping when the dog sets off (`excite` 0..1).
pub fn santa_wave(tex: &Texture2D, foot: Vec2, w: f32, t: f32, excite: f32) {
    let e = excite.clamp(0.0, 1.0);
    let breathe = (t * 2.1).sin();
    let rate = 3.2 + 3.0 * e;
    let wave = (t * rate).sin();
    let period = 0.62;
    let u = (t / period).fract();
    let lift = if e > 0.05 { (u * std::f32::consts::PI).sin().powi(2) * 0.16 * e } else { 0.0 };
    let landing = if e > 0.05 { (1.0 - (u / 0.18).min(1.0)) * 0.10 * e } else { 0.0 };
    let h = w * tex.height() / tex.width();
    let sx = 1.0 - 0.012 * breathe + landing + 0.02 * e * wave.abs();
    let sy = 1.0 + 0.02 * breathe - landing + 0.04 * lift;
    let lean = -0.07 * wave * (0.6 + 0.4 * e);
    blit_foot(tex, vec2(foot.x, foot.y - lift * h), w, sx, sy, lean, false, WHITE);
}

/// Draw the cape from `collar` along the rig's chain: a tapered ribbon
/// with a deeper-shade outline, pleats fanning out from the collar, a
/// swallow-cut hem whose two corners trail and ripple, and a gold trim.
/// `w` is the figure width in pixels.
pub fn cape(collar: Vec2, w: f32, rig: &Rig, fill: Color, t: f32) {
    let pts: Vec<Vec2> = rig.cape.iter().map(|q| collar + vec2(q.x, -q.y) * w).collect();
    let n = pts.len();
    let last = n - 1;
    let (w0, w1) = (0.05 * w, 0.24 * w);
    // Cross-sections: `rows[i]` runs from the left edge to the right edge
    // in five points, so pleats can be shaded lane by lane.
    let lanes = 5;
    let mut rows: Vec<Vec<Vec2>> = Vec::with_capacity(n);
    for i in 0..n {
        let a = pts[i.saturating_sub(1)];
        let b = pts[(i + 1).min(last)];
        let dir = (b - a).normalize_or_zero();
        let side = vec2(-dir.y, dir.x);
        let k = i as f32 / last as f32;
        let ripple = 1.0 + 0.16 * k * (t * rig.flutter * 0.9 - k * 6.0).sin();
        let half = (w0 + (w1 - w0) * k.powf(0.7)) * ripple;
        let mut row = Vec::with_capacity(lanes);
        for j in 0..lanes {
            let u = j as f32 / (lanes - 1) as f32 * 2.0 - 1.0;
            // Pleats: the cloth bunches in and out across its width.
            let pleat = 0.05 * w * k * (u * 3.0 * std::f32::consts::PI + t * rig.flutter * 0.5).sin() * (1.0 - u.abs());
            let mut q = pts[i] + side * (half * u) + dir * pleat;
            if i == last {
                // Swallow-cut hem: the corners trail, the middle tucks in.
                let tail = (u.abs() * 1.4 - 0.4) * 0.07 * w + 0.025 * w * (t * rig.flutter + u * 2.0).sin() * u.abs();
                q += dir * tail;
            }
            row.push(q);
        }
        rows.push(row);
    }
    let dark = Color::new(fill.r * 0.55, fill.g * 0.42, fill.b * 0.45, 1.0);
    let shade = Color::new(fill.r * 0.80, fill.g * 0.68, fill.b * 0.70, 1.0);
    let light = Color::new((fill.r * 1.08 + 0.10).min(1.0), (fill.g * 1.1 + 0.14).min(1.0), (fill.b * 1.1 + 0.14).min(1.0), 1.0);
    let quad = |a: Vec2, b: Vec2, c: Vec2, d: Vec2, col: Color| {
        draw_triangle(a, b, c, col);
        draw_triangle(a, c, d, col);
    };
    // Outline: every cell grown a little away from the spine, in the deep shade.
    let o = 2.5;
    for i in 0..last {
        for j in 0..lanes - 1 {
            let g = |p: Vec2, c: Vec2| p + (p - c).normalize_or_zero() * o;
            let (c0, c1) = (pts[i], pts[i + 1]);
            quad(g(rows[i][j], c0), g(rows[i + 1][j], c1), g(rows[i + 1][j + 1], c1), g(rows[i][j + 1], c0), dark);
        }
    }
    // Lanes 0 and 2 catch the light (top-lit), 1 and 3 fold into shade.
    for i in 0..last {
        for j in 0..lanes - 1 {
            let k = (i as f32 + 0.5) / last as f32;
            let col = match j {
                0 => lerp_col(fill, light, 0.5 * k + 0.2),
                1 => lerp_col(fill, shade, 0.9 * k),
                2 => lerp_col(fill, light, 0.25 * k),
                _ => lerp_col(fill, shade, 0.6 + 0.4 * k),
            };
            quad(rows[i][j], rows[i + 1][j], rows[i + 1][j + 1], rows[i][j + 1], col);
        }
    }
    // Gold trim along the hem.
    let gold = Color::new(1.0, 0.82, 0.30, 1.0);
    let gold_dark = Color::new(0.72, 0.45, 0.10, 1.0);
    let back = |q: Vec2, p: Vec2, f: f32| q + (p - q) * f;
    for j in 0..lanes - 1 {
        let (a, b) = (rows[last][j], rows[last][j + 1]);
        let (a2, b2) = (back(a, rows[last - 1][j], 0.42), back(b, rows[last - 1][j + 1], 0.42));
        quad(a2, a, b, b2, gold_dark);
        let (a3, b3) = (back(a, rows[last - 1][j], 0.30), back(b, rows[last - 1][j + 1], 0.30));
        let (a4, b4) = (back(a, rows[last - 1][j], 0.10), back(b, rows[last - 1][j + 1], 0.10));
        quad(a3, a4, b4, b3, gold);
    }
}

fn lerp_col(a: Color, b: Color, k: f32) -> Color {
    let k = k.clamp(0.0, 1.0);
    Color::new(a.r + (b.r - a.r) * k, a.g + (b.g - a.g) * k, a.b + (b.b - a.b) * k, 1.0)
}

/// Elves on the Falkbuilt roof under the inrun: they wave up at the gate
/// while the dog waits, turn to follow the sled and jump for joy as it
/// passes, then keep cheering the flight.
pub fn roof_crowd(sp: &crate::sprites::Sprites, cam: &crate::Cam, dog_x: f32, t: f32) {
    let s = cam.scale();
    for (i, (wx, b)) in [(-29.6, false), (-24.4, true), (-17.2, false)].iter().enumerate() {
        let p = cam.to_screen(*wx, hamsters_core::ROOF_Y - 0.25);
        if p.x < -2.0 * s || p.x > screen_width() + 2.0 * s || p.y < -3.0 * s || p.y > screen_height() + 3.0 * s {
            continue;
        }
        let tex = if *b { &sp.elf_b } else { &sp.elf_a };
        let near = ((9.0 - (dog_x - wx).abs()) / 9.0).clamp(0.0, 1.0);
        let excite = (0.35 * cam.cheer).max(near).max(if cam.cheer > 0.9 { 0.8 } else { 0.0 });
        // Facing the sled: before it passes they look up the hill, after it they turn to the lip.
        let flip = dog_x < *wx;
        crate::draw::cheering(tex, p, 1.45 * s, flip, t, 0.9 + i as f32 * 2.3, excite.max(0.15), WHITE);
    }
}

