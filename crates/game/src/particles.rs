//! Particle system in world units. Purely cosmetic.
//!
//! Every effect is layered from a few procedural textures (`fxtex`): painted
//! puffs and mist drawn with normal blending, glows, sparks, embers and
//! glitter drawn additively on top. Particles carry a colour ramp (`c0` to
//! `c1` over their life), a size curve (`grow`), their own drag and gravity,
//! an optional start delay (negative age) and a parallax factor: firework
//! displays burst in a far layer (`FAR`) so they hang in the sky for seconds
//! instead of whipping off screen behind a fast dog.
//!
//! Daylight trick: a saturated colour added onto a bright sky turns white, so
//! sparks get a normal-blend coloured underlay (`UNDER`) and the additive
//! pass only adds the hot core and glow.

use std::cell::Cell;

use macroquad::prelude::*;

use crate::fxtex::{additive, fx, FxTex};
use crate::sprites::{blit, blit_wh, Sprites};
use crate::Cam;

/// Hard cap on live particles (wasm budget).
const MAX: usize = 1400;
/// Parallax of firework displays: they burst "far away" over the city.
const FAR: f32 = 0.08;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    /// Painted cartoon puff (flap puffs, snow plumes, dust rings).
    Puff,
    /// Soft noisy mist (snow mist, smoke, frost breath).
    Mist,
    /// Powder snow dot.
    Snow,
    /// Painted snow clod that spins and bounces.
    Chunk,
    /// Painted ice shard.
    Shard,
    Feather,
    Confetti,
    /// Pickup star: the pack sparkle sprite with a hot additive twinkle.
    Sparkle,
    /// Additive glow (flash cores).
    Glow,
    /// Additive fire with a white-yellow-orange-red ramp.
    Flame,
    /// A spark drawn as a comet streak along its velocity.
    Spark,
    /// Small additive dot (willow and rocket trails, cinders).
    Ember,
    /// Strobing four-point star.
    Glitter,
    /// Shockwave ring in the air. One alive at a time.
    Ring,
    /// Shockwave ring flat on the ground.
    Wave,
    /// Wind streak (white, normal blend) along its velocity.
    Streak,
}

const BOUNCE: u8 = 1;
const WILLOW: u8 = 2;
const CRACKLE: u8 = 4;
const STROBE: u8 = 8;
/// Coloured normal-blend underlay so the colour survives a bright sky.
const UNDER: u8 = 16;

pub struct Particle {
    pub kind: Kind,
    pub pos: Vec2,
    pub vel: Vec2,
    /// Negative while waiting to be born.
    pub age: f32,
    pub life: f32,
    /// World size (width) at birth.
    pub size: f32,
    /// Size multiplier reached at the end of life (eased out).
    pub grow: f32,
    pub rot: f32,
    pub spin: f32,
    pub gravity: f32,
    pub drag: f32,
    /// Colour at birth and at death.
    pub c0: Color,
    pub c1: Color,
    pub behind: bool,
    /// Parallax factor: 1 for world space, `FAR` for firework displays.
    pub par: f32,
    pub var: u8,
    pub flags: u8,
    pub timer: f32,
}

fn part(kind: Kind, pos: Vec2, vel: Vec2, life: f32, size: f32, c0: Color) -> Particle {
    Particle {
        kind, pos, vel, age: 0.0, life, size, grow: 1.0, rot: 0.0, spin: 0.0, gravity: 0.0, drag: 1.5,
        c0, c1: c0, behind: false, par: 1.0, var: 0, flags: 0, timer: 0.0,
    }
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

fn mixc(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

/// The fire ramp: white-hot, yellow, orange, deep red.
pub fn fire_ramp(k: f32) -> Color {
    let stops = [
        (0.0, rgba(1.0, 0.98, 0.85, 1.0)),
        (0.18, rgba(1.0, 0.86, 0.32, 1.0)),
        (0.45, rgba(1.0, 0.52, 0.10, 1.0)),
        (0.75, rgba(0.86, 0.20, 0.06, 1.0)),
        (1.0, rgba(0.45, 0.10, 0.08, 1.0)),
    ];
    for w in stops.windows(2) {
        if k <= w[1].0 {
            return mixc(w[0].1, w[1].1, ((k - w[0].0) / (w[1].0 - w[0].0)).max(0.0));
        }
    }
    stops[4].1
}

/// The second colour a shell shifts into as its stars burn down.
fn shift_color(c: Color) -> Color {
    if c.r > 0.9 && c.g < 0.5 {
        rgba(1.0, 0.82, 0.35, 1.0) // red -> gold
    } else if c.r > 0.9 && c.g > 0.75 && c.b < 0.5 {
        rgba(1.0, 0.45, 0.2, 1.0) // gold -> orange
    } else if c.b > 0.9 && c.r < 0.6 {
        rgba(0.85, 0.95, 1.0, 1.0) // blue -> silver
    } else if c.g > 0.9 && c.r < 0.6 {
        rgba(0.6, 0.9, 1.0, 1.0) // green -> aqua
    } else {
        rgba(0.75, 0.55, 1.0, 1.0) // pink -> violet
    }
}

#[derive(Clone, Copy)]
enum Shell {
    Peony,
    Willow,
    Crackle,
    Ring,
    Palm,
}

#[derive(Default)]
pub struct Particles {
    pub list: Vec<Particle>,
    seed: u32,
    /// Camera centre at the last draw, for converting into the far layer.
    cam: Cell<Vec2>,
    /// Pixels per metre and screen size at the last draw.
    view: Cell<(f32, f32, f32)>,
    shells: u32,
    frost_acc: f32,
}

impl Particles {
    fn rnd(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.seed >> 8) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.rnd()
    }

    fn dir(&mut self, a0: f32, a1: f32) -> Vec2 {
        let a = self.range(a0, a1);
        vec2(a.cos(), a.sin())
    }

    fn push(&mut self, p: Particle) {
        self.list.push(p);
    }

    /// A world point expressed in a parallax layer, so it appears where the
    /// world point is on screen right now.
    fn to_layer(&self, w: Vec2, par: f32) -> Vec2 {
        w - self.cam.get() * (1.0 - par)
    }

    /// Nudge a display burst into the open sky of the current frame: the
    /// upper middle of the screen, clear of the edges.
    fn in_sky(&self, w: Vec2) -> Vec2 {
        let (s, sw, sh) = self.view.get();
        if s <= 0.0 {
            return w;
        }
        let c = self.cam.get();
        let px = (sw * 0.5 + (w.x - c.x) * s).clamp(sw * 0.55, sw * 0.9);
        let py = (sh * 0.5 - (w.y - c.y) * s).clamp(sh * 0.15, sh * 0.45);
        vec2(c.x + (px - sw * 0.5) / s, c.y - (py - sh * 0.5) / s)
    }

    pub fn update(&mut self, dt: f32) {
        // Children spawned this frame: (what, pos, colour, parallax).
        let mut born: Vec<(u8, Vec2, Color, f32)> = Vec::new();
        for p in &mut self.list {
            p.age += dt;
            if p.age < 0.0 {
                continue;
            }
            p.vel.y -= p.gravity * dt;
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.pos += p.vel * dt;
            p.rot += p.spin * dt;
            if p.flags & BOUNCE != 0 && p.pos.y < 0.15 && p.vel.y < 0.0 {
                p.pos.y = 0.15;
                p.vel.y *= -0.3;
                p.vel.x *= 0.5;
                p.spin *= 0.5;
            }
            if p.flags & WILLOW != 0 {
                p.timer += dt;
                if p.timer > 0.07 {
                    p.timer = 0.0;
                    let k = p.age / p.life;
                    born.push((0, p.pos, mixc(p.c0, p.c1, k), p.par));
                }
            }
            if p.flags & CRACKLE != 0 && p.age >= p.life {
                born.push((1, p.pos, p.c1, p.par));
            }
        }
        self.list.retain(|p| p.age < p.life);
        for (what, pos, c, par) in born {
            if what == 0 {
                let v = vec2(self.range(-0.5, 0.5), self.range(-1.5, 0.0));
                let life = self.range(0.5, 0.9);
                let mut e = part(Kind::Ember, pos, v, life, self.range(0.18, 0.28), c);
                e.c1 = rgba(1.0, 0.45, 0.1, 0.0);
                e.gravity = 2.5;
                e.par = par;
                e.behind = true;
                self.push(e);
            } else {
                for _ in 0..2 {
                    let off = self.dir(0.0, 6.28) * self.range(0.2, 0.9);
                    let mut g = part(Kind::Glitter, pos + off, Vec2::ZERO, self.range(0.1, 0.22), self.range(0.7, 1.1), WHITE);
                    g.par = par;
                    g.behind = true;
                    g.rot = self.rnd();
                    self.push(g);
                }
                let mut g = part(Kind::Glow, pos, Vec2::ZERO, 0.12, 1.8, with_a(c, 0.8));
                g.par = par;
                g.behind = true;
                self.push(g);
            }
        }
        if self.list.len() > MAX {
            let extra = self.list.len() - MAX;
            self.list.drain(0..extra);
        }
    }

    pub fn emit_dust(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let d = self.dir(0.6, 2.5);
            let side = if self.rnd() > 0.5 { 1.0 } else { -1.0 };
            let v = vec2(d.x * side, d.y) * self.range(2.0, 8.0);
            let mut p = part(Kind::Puff, at, v, self.range(0.5, 1.0), self.range(0.6, 1.4), rgba(0.85, 0.76, 0.62, 0.7));
            p.gravity = 4.0;
            p.grow = 1.8;
            p.var = (self.rnd() * 3.0) as u8;
            self.push(p);
        }
    }

    /// Flap / breath puffs: painted puffs that bloom and thin out.
    pub fn emit_puffs(&mut self, at: Vec2, n: usize, color: Color) {
        for i in 0..n {
            let v = vec2(self.range(-2.5, 2.5), self.range(-4.5, -1.0));
            let mut p = part(Kind::Puff, at + vec2(self.range(-0.3, 0.3), 0.0), v, self.range(0.45, 0.7), self.range(0.45, 0.9), color);
            p.grow = 2.2;
            p.drag = 3.0;
            p.rot = self.range(0.0, 6.28);
            p.spin = self.range(-1.0, 1.0);
            p.var = (i % 3) as u8;
            p.behind = true;
            self.push(p);
        }
        if color.a > 0.8 && n >= 4 {
            // A wing-beat: a little white shock ring under the ears.
            let mut w = part(Kind::Wave, at, Vec2::ZERO, 0.3, 1.2, rgba(1.0, 1.0, 1.0, 0.7));
            w.grow = 3.0;
            w.behind = true;
            self.push(w);
        }
    }

    /// Fire: an additive flame tongue, with smoke and cinders peeling off.
    pub fn emit_flame(&mut self, at: Vec2, vel: Vec2) {
        let v = vec2(-vel.x * 0.3 + self.range(-3.0, 3.0), -vel.y * 0.3 + self.range(-3.0, 3.0));
        let mut p = part(Kind::Flame, at, v, self.range(0.3, 0.5), self.range(0.7, 1.3), WHITE);
        p.grow = 1.7;
        p.gravity = -3.0;
        p.drag = 2.0;
        p.rot = self.range(0.0, 6.28);
        p.spin = self.range(-3.0, 3.0);
        p.behind = true;
        self.push(p);
        if self.rnd() < 0.4 {
            let mut s = part(Kind::Mist, at - vel * 0.02, v * 0.6, self.range(0.9, 1.5), self.range(0.7, 1.1), rgba(0.50, 0.50, 0.56, 0.42));
            s.c1 = rgba(0.78, 0.80, 0.86, 0.0);
            s.grow = 3.0;
            s.gravity = -1.0;
            s.drag = 2.5;
            s.rot = self.range(0.0, 6.28);
            s.spin = self.range(-0.6, 0.6);
            s.behind = true;
            self.push(s);
        }
        if self.rnd() < 0.3 {
            let mut e = part(Kind::Ember, at, v + vec2(self.range(-5.0, 5.0), self.range(-3.0, 5.0)), self.range(0.4, 0.8), self.range(0.15, 0.3), rgba(1.0, 0.8, 0.3, 1.0));
            e.c1 = rgba(1.0, 0.3, 0.05, 0.0);
            e.gravity = 3.0;
            e.flags = UNDER;
            e.behind = true;
            self.push(e);
        }
    }

    /// Wind streaks raking past at speed.
    pub fn emit_wind(&mut self, at: Vec2, vel: Vec2) {
        for _ in 0..2 {
            let off = vec2(self.range(-2.5, 2.5), self.range(-2.0, 3.0));
            let v = vec2(-vel.x * 0.9, -vel.y * 0.9);
            let mut p = part(Kind::Streak, at + off, v, self.range(0.25, 0.4), self.range(0.08, 0.16), rgba(1.0, 1.0, 1.0, 0.55));
            p.drag = 0.5;
            p.behind = true;
            self.push(p);
        }
    }

    pub fn emit_feathers(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let v = vec2(self.range(-7.0, 7.0), self.range(0.0, 9.0));
            let mut p = part(Kind::Feather, at, v, 1.4, self.range(0.35, 0.6), WHITE);
            p.rot = self.range(0.0, 6.28);
            p.spin = self.range(-6.0, 6.0);
            p.gravity = 3.0;
            self.push(p);
        }
    }

    /// Pickup stars: gold sparkle sprites with hot twinkling cores.
    pub fn emit_sparkles(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let v = self.dir(0.0, 6.28) * self.range(3.0, 10.0);
            let mut p = part(Kind::Sparkle, at, v, self.range(0.55, 0.85), self.range(0.45, 0.95), rgba(1.0, 0.92, 0.45, 1.0));
            p.c1 = rgba(1.0, 0.6, 0.2, 1.0);
            p.spin = self.range(-4.0, 4.0);
            p.gravity = 6.0;
            self.push(p);
        }
        for _ in 0..(n / 2).max(2) {
            let v = self.dir(0.0, 6.28) * self.range(1.0, 5.0);
            let mut g = part(Kind::Glitter, at + v * 0.1, v, self.range(0.3, 0.6), self.range(0.4, 0.8), rgba(1.0, 0.95, 0.7, 1.0));
            g.rot = self.rnd();
            self.push(g);
        }
    }

    /// Snow spray: powder that blooms, with the odd ice glint.
    pub fn emit_snow(&mut self, at: Vec2, base: Vec2, n: usize) {
        for i in 0..n {
            let v = base + vec2(self.range(-4.0, 4.0), self.range(0.0, 5.0));
            let mut p = part(Kind::Snow, at, v, self.range(0.5, 1.0), self.range(0.22, 0.5), rgba(1.0, 1.0, 1.0, 0.95));
            p.c1 = rgba(0.82, 0.89, 1.0, 0.9);
            p.grow = 1.7;
            p.gravity = 6.0;
            self.push(p);
            if i % 6 == 0 {
                let mut g = part(Kind::Glitter, at, v * 0.8, self.range(0.3, 0.6), self.range(0.3, 0.55), rgba(0.85, 0.95, 1.0, 1.0));
                g.gravity = 5.0;
                g.rot = self.rnd();
                self.push(g);
            }
        }
    }

    /// The sled's rooster tail: powder kicked up and back off the runners,
    /// a soft trailing mist and the odd ice glint.
    pub fn emit_spray(&mut self, at: Vec2, vel: Vec2) {
        let speed = vel.length();
        let k = (speed / 25.0).clamp(0.3, 1.0);
        for _ in 0..3 {
            let v = vec2(-vel.x * self.range(0.15, 0.35), self.range(2.0, 6.0) * k) + vec2(self.range(-1.5, 1.5), 0.0);
            let mut p = part(Kind::Snow, at + vec2(self.range(-0.3, 0.3), 0.0), v, self.range(0.4, 0.7), self.range(0.25, 0.55) * (0.6 + 0.6 * k), WHITE);
            p.c1 = rgba(0.84, 0.9, 1.0, 0.85);
            p.grow = 2.0;
            p.gravity = 9.0;
            p.drag = 2.0;
            self.push(p);
        }
        if self.rnd() < 0.35 {
            let v = vec2(-vel.x * 0.1, 1.0);
            let mut m = part(Kind::Puff, at, v, self.range(0.5, 0.8), self.range(0.5, 0.8) * k, rgba(1.0, 1.0, 1.0, 0.75));
            m.grow = 2.4;
            m.drag = 3.0;
            m.rot = self.range(0.0, 6.28);
            m.var = (self.rnd() * 3.0) as u8;
            m.behind = true;
            self.push(m);
        }
        if self.rnd() < 0.25 {
            let mut g = part(Kind::Glitter, at, vec2(-vel.x * 0.2, self.range(1.0, 4.0)), self.range(0.25, 0.45), self.range(0.3, 0.5), rgba(0.9, 0.97, 1.0, 1.0));
            g.gravity = 6.0;
            g.rot = self.rnd();
            self.push(g);
        }
    }

    /// Chunky snow clods that arc and tumble, with powder between them.
    pub fn emit_clods(&mut self, at: Vec2, base: Vec2, n: usize) {
        for i in 0..n {
            let v = base + vec2(self.range(-9.0, 9.0), self.range(2.0, 12.0));
            if i % 2 == 0 {
                let mut p = part(Kind::Chunk, at, v, self.range(0.7, 1.2), self.range(0.28, 0.6), WHITE);
                p.gravity = 20.0;
                p.drag = 0.3;
                p.rot = self.range(0.0, 6.28);
                p.spin = self.range(-9.0, 9.0);
                p.var = (self.rnd() * 3.0) as u8;
                self.push(p);
            } else {
                let mut p = part(Kind::Snow, at, v, self.range(0.6, 1.1), self.range(0.3, 0.6), WHITE);
                p.c1 = rgba(0.84, 0.9, 1.0, 1.0);
                p.gravity = 16.0;
                p.drag = 0.8;
                p.grow = 1.4;
                self.push(p);
            }
        }
    }

    /// Big slow snow plume: painted puffs that fill a good part of the frame.
    pub fn emit_plume(&mut self, at: Vec2, base: Vec2, n: usize) {
        for i in 0..n {
            let v = base + vec2(self.range(-5.0, 5.0), self.range(0.0, 6.0));
            let mut p = part(Kind::Puff, at, v, self.range(0.8, 1.4), self.range(1.5, 3.0), rgba(1.0, 1.0, 1.0, 0.9));
            p.grow = 2.0;
            p.gravity = 3.0;
            p.drag = 2.0;
            p.rot = self.range(0.0, 6.28);
            p.spin = self.range(-0.8, 0.8);
            p.var = (i % 3) as u8;
            p.behind = true;
            self.push(p);
        }
    }

    /// Coloured confetti that flutters down, flipping like paper.
    pub fn emit_confetti(&mut self, at: Vec2, n: usize) {
        let palette = [
            rgba(1.0, 0.30, 0.30, 1.0), rgba(0.30, 0.85, 0.45, 1.0),
            rgba(1.0, 0.82, 0.25, 1.0), rgba(0.35, 0.65, 1.0, 1.0), WHITE, rgba(1.0, 0.5, 0.85, 1.0),
        ];
        for i in 0..n {
            let a = self.range(0.0, 6.28);
            let sp = self.range(4.0, 15.0);
            let mut p = part(Kind::Confetti, at, vec2(a.cos() * sp, a.sin() * sp + 5.0), self.range(1.4, 2.0), self.range(0.28, 0.5), palette[i % palette.len()]);
            p.rot = a;
            p.spin = self.range(-10.0, 10.0);
            p.gravity = 5.0;
            p.drag = 2.2;
            p.timer = self.range(0.0, 6.28);
            self.push(p);
        }
    }

    /// Launch burst drawn behind the dog, so the silhouette stays visible.
    pub fn emit_burst_behind(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let v = self.dir(0.0, 6.28) * self.range(8.0, 20.0);
            let mut p = part(Kind::Spark, at, v, self.range(0.35, 0.6), self.range(0.28, 0.45), rgba(1.0, 0.92, 0.55, 1.0));
            p.c1 = rgba(1.0, 0.6, 0.2, 1.0);
            p.gravity = 4.0;
            p.drag = 3.0;
            p.flags = UNDER;
            p.behind = true;
            self.push(p);
        }
        let mut g = part(Kind::Glow, at, Vec2::ZERO, 0.25, 6.0, rgba(1.0, 0.95, 0.7, 0.9));
        g.grow = 1.4;
        g.behind = true;
        self.push(g);
        self.list.retain(|p| p.kind != Kind::Ring);
        let mut r = part(Kind::Ring, at, Vec2::ZERO, 0.45, 2.0, WHITE);
        r.grow = 5.0;
        r.behind = true;
        self.push(r);
    }

    /// A fireworks burst at `at`. A full shell (`n` >= 24) blooms in the far
    /// layer as one of five shell types, cycling so a display varies: peony
    /// with a colour-shifting pistil, golden willow with dripping trails,
    /// crackle, a tilted ring, and a palm of thick comets. Smaller counts are
    /// the quick crackling pops behind a rocket ride, in world space.
    pub fn emit_fireworks(&mut self, at: Vec2, color: Color, n: usize) {
        if n < 24 {
            self.pop(at, color, n);
            return;
        }
        let style = [Shell::Peony, Shell::Willow, Shell::Crackle, Shell::Ring, Shell::Palm, Shell::Peony, Shell::Crackle][self.shells as usize % 7];
        self.shells += 1;
        self.shell(at, color, style);
    }

    /// The rising comet of a shell that bursts at `pos` in `delay` seconds:
    /// call every frame while the shell waits.
    pub fn shell_rising(&mut self, pos: Vec2, delay: f32) {
        let head = self.to_layer(self.in_sky(pos) - vec2(0.0, 22.0 * delay.max(0.0)), FAR);
        let jit = vec2(self.range(-0.15, 0.15), 0.0);
        let v = vec2(self.range(-0.6, 0.6), -1.5);
        let mut e = part(Kind::Ember, head + jit, v, self.range(0.35, 0.6), self.range(0.2, 0.32), rgba(1.0, 0.85, 0.5, 1.0));
        e.c1 = rgba(1.0, 0.4, 0.1, 0.0);
        e.gravity = 3.0;
        e.par = FAR;
        e.behind = true;
        e.flags = UNDER;
        self.push(e);
        let mut g = part(Kind::Glow, head, Vec2::ZERO, 0.05, 1.4, rgba(1.0, 0.95, 0.8, 1.0));
        g.par = FAR;
        g.behind = true;
        self.push(g);
    }

    fn pop(&mut self, at: Vec2, color: Color, n: usize) {
        let n = n.max(8) + 4;
        for i in 0..n {
            let a = i as f32 / n as f32 * 6.283 + self.range(-0.2, 0.2);
            let sp = self.range(9.0, 13.0);
            let mut p = part(Kind::Spark, at, vec2(a.cos(), a.sin()) * sp, self.range(0.45, 0.7), self.range(0.26, 0.34), color);
            p.c1 = shift_color(color);
            p.gravity = 5.0;
            p.drag = 2.4;
            p.rot = self.rnd();
            p.flags = UNDER | STROBE;
            self.push(p);
        }
        let mut g = part(Kind::Glow, at, Vec2::ZERO, 0.18, 4.0, with_a(color, 0.9));
        g.c1 = with_a(WHITE, 0.0);
        self.push(g);
    }

    fn shell(&mut self, at: Vec2, color: Color, style: Shell) {
        let at = self.to_layer(self.in_sky(at), FAR);
        let c2 = shift_color(color);
        // A display reads the same size on screen whatever the camera zoom.
        let z = (24.0 / self.view.get().0.max(1.0)).clamp(0.5, 1.8);
        let far = |mut p: Particle| {
            p.par = FAR;
            p.behind = true;
            p.pos = at + (p.pos - at) * z;
            p.size *= z;
            p.vel *= z;
            p.gravity *= z;
            p
        };
        // Flash: a white-hot core and a coloured bloom.
        let mut g = part(Kind::Glow, at, Vec2::ZERO, 0.2, 7.0, rgba(1.0, 1.0, 0.95, 1.0));
        g.grow = 1.3;
        self.push(far(g));
        let mut g = part(Kind::Glow, at, Vec2::ZERO, 0.7, 12.0, with_a(color, 0.55));
        g.c1 = with_a(c2, 0.0);
        g.grow = 1.4;
        self.push(far(g));
        // Burst smoke: a soft daylight backdrop that gives the colours contrast.
        for _ in 0..4 {
            let off = self.dir(0.0, 6.28) * self.range(0.5, 2.5);
            let mut s = part(Kind::Mist, at + off, off * 0.6, self.range(2.6, 3.4), self.range(4.0, 6.0), rgba(0.62, 0.66, 0.78, 0.30));
            s.c1 = rgba(0.85, 0.88, 0.95, 0.0);
            s.grow = 1.8;
            s.gravity = -0.3;
            s.drag = 0.8;
            s.rot = self.range(0.0, 6.28);
            s.spin = self.range(-0.2, 0.2);
            self.push(far(s));
        }
        let spark = |me: &mut Self, v: Vec2, life: f32, size: f32, c0: Color, c1: Color, grav: f32, drag: f32, flags: u8| {
            let mut p = part(Kind::Spark, at, v, life, size, c0);
            p.c1 = c1;
            p.gravity = grav;
            p.drag = drag;
            p.flags = flags | UNDER;
            p.rot = me.rnd();
            me.push(far(p));
        };
        match style {
            Shell::Peony => {
                let n = 54;
                for i in 0..n {
                    // A Fibonacci sphere seen from the side: an even round bloom with depth.
                    let a = i as f32 * 2.39996 + self.range(-0.05, 0.05);
                    let z = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
                    let r = (1.0 - z * z).sqrt();
                    let sp = 18.0 * self.range(0.95, 1.05);
                    let v = vec2(a.cos() * r, a.sin() * r) * sp;
                    let life = self.range(1.5, 1.9);
                    spark(self, v, life, 0.5, color, mixc(color, c2, 0.5), 3.5, 1.7, STROBE);
                }
                for i in 0..18 {
                    let a = i as f32 / 18.0 * 6.283;
                    let v = vec2(a.cos(), a.sin()) * self.range(7.0, 8.0);
                    let life = self.range(1.1, 1.3);
                    spark(self, v, life, 0.42, c2, WHITE, 3.0, 1.7, STROBE);
                }
            }
            Shell::Willow => {
                let gold = rgba(1.0, 0.80, 0.38, 1.0);
                for i in 0..40 {
                    let a = i as f32 / 40.0 * 6.283 + self.range(-0.08, 0.08);
                    let v = vec2(a.cos(), a.sin()) * self.range(15.0, 17.0);
                    let life = self.range(2.4, 3.0);
                    spark(self, v, life, 0.42, gold, rgba(1.0, 0.45, 0.12, 1.0), 5.5, 2.2, WILLOW);
                }
                for i in 0..10 {
                    let a = i as f32 / 10.0 * 6.283;
                    let v = vec2(a.cos(), a.sin()) * 5.0;
                    spark(self, v, 1.1, 0.45, color, c2, 3.0, 1.6, STROBE);
                }
            }
            Shell::Crackle => {
                for i in 0..46 {
                    let a = i as f32 / 46.0 * 6.283 + self.range(-0.1, 0.1);
                    let v = vec2(a.cos(), a.sin()) * self.range(15.0, 19.0);
                    let life = self.range(1.0, 1.3);
                    spark(self, v, life, 0.46, color, mixc(color, WHITE, 0.4), 4.0, 1.6, CRACKLE);
                }
            }
            Shell::Ring => {
                let tilt = self.range(-0.6, 0.6);
                let (ct, st) = (tilt.cos(), tilt.sin());
                for i in 0..40 {
                    let a = i as f32 / 40.0 * 6.283;
                    let e = vec2(a.cos() * 19.0, a.sin() * 7.5);
                    let v = vec2(e.x * ct - e.y * st, e.x * st + e.y * ct);
                    let life = self.range(1.4, 1.7);
                    spark(self, v, life, 0.5, color, c2, 3.0, 1.7, STROBE);
                }
                for _ in 0..16 {
                    let v = self.dir(0.0, 6.28) * self.range(1.0, 4.0);
                    let mut g = part(Kind::Glitter, at, v, self.range(0.8, 1.4), self.range(0.6, 1.0), WHITE);
                    g.c1 = c2;
                    g.gravity = 1.5;
                    g.rot = self.rnd();
                    self.push(far(g));
                }
            }
            Shell::Palm => {
                for i in 0..9 {
                    let a = i as f32 / 9.0 * 6.283 + 0.35;
                    let v = vec2(a.cos(), a.sin() + 0.25) * self.range(17.0, 19.0);
                    let life = self.range(1.8, 2.2);
                    spark(self, v, life, 0.8, color, c2, 6.0, 1.4, WILLOW);
                }
                for i in 0..24 {
                    let a = i as f32 / 24.0 * 6.283;
                    let v = vec2(a.cos(), a.sin()) * self.range(6.0, 9.0);
                    let life = self.range(0.6, 0.9);
                    spark(self, v, life, 0.36, WHITE, color, 3.0, 2.0, CRACKLE);
                }
            }
        }
    }

    pub fn emit_ring(&mut self, at: Vec2, color: Color) {
        self.emit_ring_sized(at, color, 1.5, 0.5);
    }

    pub fn emit_ring_sized(&mut self, at: Vec2, color: Color, size: f32, life: f32) {
        // One ring alive at a time: a chain of pickups must not stack them.
        self.list.retain(|p| p.kind != Kind::Ring);
        let mut r = part(Kind::Ring, at, Vec2::ZERO, life, size, color);
        r.grow = 5.0;
        self.push(r);
        let mut g = part(Kind::Glow, at, Vec2::ZERO, life * 0.5, size * 2.6, with_a(color, 0.85));
        g.c1 = with_a(WHITE, 0.0);
        self.push(g);
    }

    /// The crash, in layers: a flash, a ground shockwave and an air ring, a
    /// cone of powder, painted snow clods and ice that arc out and bounce, a
    /// rolling ground-hugging dust ring, a tall snow-mist column that hangs
    /// and drifts for seconds, and glints twinkling in it. A meteor (`big`
    /// up to 1) adds a fire core, cinders and a dark smoke column.
    pub fn emit_crash(&mut self, x: f32, big: f32) {
        let g0 = vec2(x, 0.3);
        let sc = 1.0 + 0.7 * big;
        let core = if big > 0.3 { rgba(1.0, 0.8, 0.45, 1.0) } else { rgba(0.92, 0.97, 1.0, 1.0) };
        let mut g = part(Kind::Glow, g0 + vec2(0.0, 0.6), Vec2::ZERO, 0.3, 4.5 * sc, with_a(core, 0.8));
        g.grow = 1.5;
        self.push(g);
        // Impact star: white streaks flung out from the hit, comic-style.
        for i in 0..18 {
            let a = 0.15 + i as f32 / 17.0 * 2.84 + self.range(-0.05, 0.05);
            let v = vec2(a.cos(), a.sin()) * self.range(22.0, 34.0) * sc;
            let mut p = part(Kind::Streak, g0 + vec2(a.cos(), a.sin()) * 0.6, v, self.range(0.18, 0.28), self.range(0.16, 0.26), WHITE);
            p.drag = 6.0;
            self.push(p);
        }
        // Ground shockwaves and an air ring.
        for (life, grow, a) in [(0.55, 6.0, 0.95), (0.9, 9.0, 0.6)] {
            let mut w = part(Kind::Wave, vec2(x, 0.1), Vec2::ZERO, life, 2.2 * sc, rgba(1.0, 1.0, 1.0, a));
            w.c1 = rgba(0.8, 0.88, 1.0, a);
            w.grow = grow;
            self.push(w);
        }
        self.list.retain(|p| p.kind != Kind::Ring);
        let ring = if big > 0.3 { rgba(1.0, 0.75, 0.4, 1.0) } else { WHITE };
        let mut r = part(Kind::Ring, g0 + vec2(0.0, 1.0), Vec2::ZERO, 0.45, 2.5 * sc, ring);
        r.grow = 5.0;
        self.push(r);
        // Powder eruption: a cone up and out, staggered so it keeps coming.
        let n = (70.0 + 60.0 * big) as usize;
        for i in 0..n {
            // Two lobes, up-left and up-right, so the dog stays in view.
            let d = if i % 2 == 0 { self.dir(0.3, 1.25) } else { self.dir(1.9, 2.84) };
            let sp = self.range(5.0, 15.0) * (1.0 + 0.35 * big);
            let mut p = part(Kind::Snow, g0, d * sp, self.range(1.0, 1.7), self.range(0.2, 0.5), WHITE);
            p.c1 = rgba(0.80, 0.87, 1.0, 0.9);
            p.grow = 1.8;
            p.gravity = 16.0;
            p.drag = 1.3;
            p.age = -0.12 * (i % 4) as f32 * self.rnd();
            self.push(p);
        }
        // Splash crown: jets of snow fanned up and out, each a line of
        // powder at staggered speeds so it reads as a spike.
        let jets = 10;
        for j in 0..jets {
            let side = (j % 2) as f32;
            let ang = 0.32 + (j / 2) as f32 * 0.22 + self.range(-0.06, 0.06);
            let ang = if side > 0.0 { std::f32::consts::PI - ang } else { ang };
            let d = vec2(ang.cos(), ang.sin());
            let top = self.range(11.0, 16.0) * (0.75 + 0.25 * ang.sin()) * (1.0 + 0.5 * big);
            for q in 0..9 {
                let f = 0.35 + 0.65 * q as f32 / 8.0;
                let mut p = part(Kind::Snow, g0, d * top * f, self.range(1.0, 1.4), (0.5 - 0.25 * f) * sc, WHITE);
                p.c1 = rgba(0.84, 0.9, 1.0, 0.9);
                p.grow = 1.6;
                p.gravity = 18.0;
                p.drag = 0.9;
                self.push(p);
            }
            let mut c = part(Kind::Chunk, g0, d * top * 1.05, self.range(1.8, 2.4), self.range(0.35, 0.55) * sc, WHITE);
            c.gravity = 18.0;
            c.drag = 0.4;
            c.spin = self.range(-10.0, 10.0);
            c.var = j as u8;
            c.flags = BOUNCE;
            self.push(c);
        }
        // Debris: painted clods and ice, a few asphalt bits from the street.
        let n = (20.0 + 22.0 * big) as usize;
        for i in 0..n {
            let d = self.dir(0.3, 2.84);
            let sp = self.range(5.0, 13.0) * (1.0 + 0.4 * big);
            let kind = if i % 5 == 0 { Kind::Shard } else { Kind::Chunk };
            let col = if i % 7 == 3 {
                rgba(0.42, 0.42, 0.50, 1.0)
            } else if big > 0.4 && i % 4 == 1 {
                rgba(0.62, 0.48, 0.42, 1.0)
            } else {
                WHITE
            };
            let at = g0 + vec2(self.range(-0.5, 0.5), 0.0);
            let mut p = part(kind, at, d * sp, self.range(2.0, 2.8), self.range(0.25, 0.6), col);
            p.gravity = 18.0;
            p.drag = 0.35;
            p.rot = self.range(0.0, 6.28);
            p.spin = self.range(-12.0, 12.0);
            p.var = (i % 3) as u8;
            p.flags = BOUNCE;
            self.push(p);
        }
        // Dust ring rolling out along the ground both ways.
        for i in 0..16 {
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            let v = vec2(side * self.range(5.0, 11.0) * sc, self.range(0.3, 1.2));
            let at = vec2(x + side * self.range(1.3, 2.0), 0.35);
            let mut p = part(Kind::Puff, at, v, self.range(1.4, 2.2), self.range(0.7, 1.2) * sc, WHITE);
            p.c1 = rgba(0.88, 0.92, 1.0, 0.9);
            p.grow = 2.4;
            p.drag = 2.4;
            p.rot = self.range(0.0, 6.28);
            p.spin = side * self.range(0.5, 1.5);
            p.var = (i % 3) as u8;
            self.push(p);
        }
        // The snow-mist column: rises, spreads and hangs over the hole.
        let n = (10.0 + 10.0 * big) as usize;
        for i in 0..n {
            let v = vec2(self.range(-3.0, 3.0), self.range(2.0, 6.0) * sc);
            let at = g0 + vec2(self.range(-1.8, 1.8), self.range(1.8, 3.0));
            let mut p = part(Kind::Puff, at, v, self.range(2.6, 3.6), self.range(0.9, 1.6) * sc, rgba(1.0, 1.0, 1.0, 0.9));
            p.c1 = rgba(0.86, 0.91, 1.0, 0.8);
            p.grow = 2.6;
            p.drag = 1.6;
            p.gravity = -0.6;
            p.rot = self.range(0.0, 6.28);
            p.spin = self.range(-0.5, 0.5);
            p.var = (i % 3) as u8;
            p.behind = true;
            p.age = -0.05 * (i % 4) as f32;
            self.push(p);
        }
        for _ in 0..(6.0 + 6.0 * big) as usize {
            let v = vec2(self.range(-2.0, 2.0), self.range(0.5, 2.0));
            let at = g0 + vec2(self.range(-3.0, 3.0), self.range(2.0, 4.0));
            let mut p = part(Kind::Mist, at, v, self.range(3.0, 4.0), self.range(2.5, 3.5) * sc, rgba(0.95, 0.97, 1.0, 0.45));
            p.c1 = rgba(0.9, 0.93, 1.0, 0.0);
            p.grow = 2.0;
            p.drag = 1.0;
            p.rot = self.range(0.0, 6.28);
            p.age = -0.15;
            self.push(p);
        }
        // Ice glints twinkling in the cloud.
        for _ in 0..24 {
            let off = vec2(self.range(-4.0, 4.0), self.range(0.5, 4.0)) * sc;
            let v = vec2(self.range(-1.0, 1.0), self.range(-0.5, 1.5));
            let mut p = part(Kind::Glitter, g0 + off, v, self.range(0.5, 1.0), self.range(0.45, 0.8), rgba(0.85, 0.95, 1.0, 1.0));
            p.age = -self.range(0.05, 0.9);
            p.rot = self.rnd();
            self.push(p);
        }
        if big > 0.05 {
            for _ in 0..(18.0 * big) as usize {
                let v = vec2(self.range(-6.0, 6.0), self.range(2.0, 10.0));
                let at = g0 + vec2(self.range(-1.5, 1.5), self.range(0.0, 1.0));
                let mut p = part(Kind::Flame, at, v, self.range(0.5, 0.9), self.range(1.4, 2.6), WHITE);
                p.grow = 1.8;
                p.gravity = -4.0;
                p.drag = 2.0;
                p.rot = self.range(0.0, 6.28);
                p.spin = self.range(-2.0, 2.0);
                self.push(p);
            }
            for _ in 0..(50.0 * big) as usize {
                let v = self.dir(0.3, 2.84) * self.range(10.0, 28.0);
                let mut p = part(Kind::Spark, g0, v, self.range(0.6, 1.3), self.range(0.22, 0.34), rgba(1.0, 0.85, 0.4, 1.0));
                p.c1 = rgba(1.0, 0.3, 0.05, 1.0);
                p.gravity = 14.0;
                p.drag = 1.2;
                p.flags = UNDER | BOUNCE;
                self.push(p);
            }
            for i in 0..(10.0 * big) as usize {
                let v = vec2(self.range(-2.0, 2.0), self.range(2.0, 6.0));
                let at = g0 + vec2(self.range(-1.0, 1.0), 1.0);
                let mut p = part(Kind::Mist, at, v, self.range(3.0, 4.0), self.range(2.0, 3.0), rgba(0.30, 0.26, 0.28, 0.75));
                p.c1 = rgba(0.62, 0.60, 0.64, 0.0);
                p.grow = 3.0;
                p.drag = 0.9;
                p.gravity = -1.2;
                p.rot = self.range(0.0, 6.28);
                p.age = -0.2 - 0.1 * i as f32;
                p.behind = true;
                self.push(p);
            }
        }
    }

    /// While frozen: frost glints and cold mist curling off the dog.
    pub fn emit_frost(&mut self, at: Vec2, dt: f32) {
        self.frost_acc += dt;
        while self.frost_acc > 0.07 {
            self.frost_acc -= 0.07;
            let off = self.dir(0.0, 6.28) * self.range(0.6, 1.6);
            let mut g = part(Kind::Glitter, at + off, off * 0.3 + vec2(0.0, -0.5), self.range(0.4, 0.8), self.range(0.35, 0.6), rgba(0.8, 0.95, 1.0, 1.0));
            g.rot = self.rnd();
            self.push(g);
            let at2 = at + vec2(self.range(-0.6, 0.6), -0.4);
            let v = vec2(self.range(-0.6, 0.6), -1.2);
            let mut m = part(Kind::Mist, at2, v, self.range(0.8, 1.2), self.range(0.8, 1.2), rgba(0.80, 0.92, 1.0, 0.35));
            m.c1 = rgba(0.9, 0.97, 1.0, 0.0);
            m.grow = 2.2;
            m.drag = 1.0;
            m.rot = self.range(0.0, 6.28);
            m.behind = true;
            self.push(m);
        }
    }

    /// Hit by the cold wind: a burst of ice and frost.
    pub fn emit_freeze(&mut self, at: Vec2) {
        self.emit_ring_sized(at, rgba(0.7, 0.9, 1.0, 1.0), 1.8, 0.5);
        for i in 0..16 {
            let v = self.dir(0.0, 6.28) * self.range(6.0, 14.0);
            let mut p = part(Kind::Shard, at, v, self.range(0.6, 0.9), self.range(0.3, 0.55), WHITE);
            p.drag = 3.0;
            p.rot = v.y.atan2(v.x) - 1.5708;
            p.spin = self.range(-3.0, 3.0);
            p.var = i as u8;
            self.push(p);
        }
        for _ in 0..6 {
            let v = self.dir(0.0, 6.28) * self.range(1.0, 4.0);
            let mut m = part(Kind::Mist, at, v, self.range(0.9, 1.3), self.range(1.5, 2.2), rgba(0.85, 0.95, 1.0, 0.55));
            m.c1 = rgba(0.9, 0.97, 1.0, 0.0);
            m.grow = 2.0;
            m.drag = 2.0;
            m.behind = true;
            self.push(m);
        }
    }

    /// The ice breaking off on thaw: shards that fall and tumble.
    pub fn emit_shatter(&mut self, at: Vec2) {
        for i in 0..24 {
            let v = self.dir(0.0, 6.28) * self.range(4.0, 13.0) + vec2(0.0, 3.0);
            let mut p = part(Kind::Shard, at + v * 0.05, v, self.range(0.9, 1.4), self.range(0.3, 0.6), WHITE);
            p.gravity = 16.0;
            p.drag = 0.6;
            p.rot = self.range(0.0, 6.28);
            p.spin = self.range(-10.0, 10.0);
            p.var = i as u8;
            self.push(p);
        }
        for _ in 0..14 {
            let v = self.dir(0.0, 6.28) * self.range(2.0, 7.0);
            let mut g = part(Kind::Glitter, at, v, self.range(0.4, 0.8), self.range(0.4, 0.8), rgba(0.85, 0.97, 1.0, 1.0));
            g.rot = self.rnd();
            self.push(g);
        }
    }

    /// Struck by lightning: an electric flash, crackling sparks, singed smoke.
    pub fn emit_zap(&mut self, at: Vec2) {
        let mut g = part(Kind::Glow, at, Vec2::ZERO, 0.2, 7.0, rgba(1.0, 1.0, 0.85, 1.0));
        g.c1 = rgba(0.6, 0.7, 1.0, 0.0);
        self.push(g);
        for _ in 0..30 {
            let v = self.dir(0.0, 6.28) * self.range(12.0, 30.0);
            let mut p = part(Kind::Spark, at, v, self.range(0.2, 0.45), self.range(0.2, 0.3), rgba(1.0, 1.0, 0.8, 1.0));
            p.c1 = rgba(0.55, 0.65, 1.0, 1.0);
            p.drag = 5.0;
            p.gravity = 4.0;
            p.flags = UNDER;
            self.push(p);
        }
        for _ in 0..6 {
            let v = vec2(self.range(-2.0, 2.0), self.range(1.0, 3.0));
            let mut m = part(Kind::Mist, at, v, self.range(0.9, 1.3), self.range(1.0, 1.6), rgba(0.25, 0.22, 0.30, 0.6));
            m.c1 = rgba(0.6, 0.6, 0.65, 0.0);
            m.grow = 2.5;
            m.drag = 1.5;
            m.rot = self.range(0.0, 6.28);
            m.behind = true;
            self.push(m);
        }
    }

    pub fn draw_behind(&self, sp: &Sprites, cam: &Cam) {
        self.cam.set(vec2(cam.x, cam.y));
        self.view.set((cam.scale(), screen_width(), screen_height()));
        self.draw(sp, cam, true);
    }

    pub fn draw_front(&self, sp: &Sprites, cam: &Cam) {
        self.draw(sp, cam, false);
    }

    fn draw(&self, sp: &Sprites, cam: &Cam, behind: bool) {
        let t = fx();
        let s = cam.scale();
        let live = || self.list.iter().filter(move |p| p.behind == behind && p.age >= 0.0);
        // Pass 1, normal blend: soft volumes and solid bits, far layer first.
        for far in [true, false] {
            for p in live().filter(|p| (p.par < 1.0) == far) {
                self.draw_normal(p, sp, &t, cam, s);
            }
        }
        // Pass 2, additive: glows, fire, spark cores, glitter.
        additive(|| {
            for p in live() {
                self.draw_add(p, &t, cam, s);
            }
        });
    }

    fn screen(&self, p: &Particle, cam: &Cam) -> Vec2 {
        if p.par < 1.0 {
            cam.to_screen_parallax_xy(p.pos.x, p.pos.y, p.par, p.par)
        } else {
            cam.to_screen(p.pos.x, p.pos.y)
        }
    }

    fn fade(p: &Particle, k: f32) -> f32 {
        let fin = match p.kind {
            Kind::Mist | Kind::Puff => (k / 0.12).min(1.0),
            Kind::Glow | Kind::Flame => (k / 0.04).min(1.0),
            _ => (k / 0.03).min(1.0),
        };
        let out = match p.kind {
            Kind::Puff | Kind::Mist => (1.0 - k).powf(1.4),
            Kind::Snow | Kind::Sparkle => 1.0 - k * k,
            Kind::Chunk | Kind::Shard | Kind::Feather => ((1.0 - k) / 0.3).min(1.0),
            Kind::Confetti => ((1.0 - k) / 0.25).min(1.0),
            Kind::Glow => (1.0 - k) * (1.0 - k),
            Kind::Ring | Kind::Wave => (1.0 - k).powf(1.5),
            Kind::Spark => {
                let strobe = p.flags & STROBE != 0 && k > 0.55 && (p.age * 28.0 + p.rot * 40.0).sin() < -0.2;
                (1.0 - k * k * k) * if strobe { 0.15 } else { 1.0 }
            }
            Kind::Glitter => {
                let tw = 0.5 + 0.5 * (p.age * 34.0 + p.rot * 60.0).sin();
                (1.0 - k) * (0.2 + 0.8 * tw)
            }
            _ => 1.0 - k,
        };
        fin * out
    }

    fn size(p: &Particle, k: f32) -> f32 {
        let e = 1.0 - (1.0 - k) * (1.0 - k);
        p.size * (1.0 + (p.grow - 1.0) * e)
    }

    /// Screen-space streak for a moving spark: centre, length, angle. The
    /// head sits at the particle; the tail trails `secs` of motion behind.
    fn streak(p: &Particle, scr: Vec2, s: f32, w: f32, secs: f32) -> (Vec2, f32, f32) {
        let v = p.vel;
        let len = (v.length() * s * secs).max(w * 1.6);
        let ang = v.y.atan2(v.x);
        let dir = vec2(v.x, -v.y).normalize_or_zero();
        (scr - dir * (len * 0.5 - w * 0.4), len, ang)
    }

    fn offscreen(scr: Vec2, w: f32) -> bool {
        let m = w * 2.0 + 200.0;
        scr.x < -m || scr.x > screen_width() + m || scr.y < -m || scr.y > screen_height() + m
    }

    fn draw_normal(&self, p: &Particle, sp: &Sprites, t: &FxTex, cam: &Cam, s: f32) {
        let k = p.age / p.life;
        let scr = self.screen(p, cam);
        let w = Self::size(p, k) * s;
        if Self::offscreen(scr, w) {
            return;
        }
        let mut c = mixc(p.c0, p.c1, k);
        c.a *= Self::fade(p, k);
        match p.kind {
            Kind::Puff => blit(&t.puffs[p.var as usize % 3], scr, w, p.rot, false, c),
            Kind::Mist => blit(&t.mist, scr, w, p.rot, false, c),
            Kind::Snow => {
                // Powder with a blue-grey underside so it reads on white snow.
                blit(&t.soft, scr + vec2(0.0, w * 0.12), w * 1.1, 0.0, false, Color::new(0.55, 0.64, 0.84, c.a * 0.45));
                blit(&t.soft, scr, w, 0.0, false, c);
            }
            Kind::Chunk => blit(&t.chunks[p.var as usize % 3], scr, w, p.rot, false, c),
            Kind::Shard => blit(&t.shard, scr, w * 0.67, p.rot, p.var % 2 == 1, c),
            Kind::Feather => blit(&sp.feather, scr, w, p.rot, false, c),
            Kind::Confetti => {
                // Paper flipping over: thin edge-on, darker on the back face.
                let flip = (p.age * 9.0 + p.timer).cos();
                let back = if flip < 0.0 { 0.7 } else { 1.0 };
                blit_wh(&sp.confetti, scr, w * flip.abs().max(0.12), w, p.rot, false, Color::new(c.r * back, c.g * back, c.b * back, c.a));
            }
            Kind::Sparkle => blit(&sp.sparkle, scr, w, p.rot, false, c),
            Kind::Ring => blit(&t.ring, scr, w, 0.0, false, with_a(c, c.a * 0.85)),
            Kind::Wave => {
                blit_wh(&t.ring, scr + vec2(0.0, w * 0.025), w * 1.03, w * 0.31, 0.0, false, Color::new(0.45, 0.56, 0.80, c.a * 0.75));
                blit_wh(&t.ring, scr, w, w * 0.28, 0.0, false, c);
            }
            Kind::Streak => {
                let (ctr, len, ang) = Self::streak(p, scr, s, w, 0.07);
                if p.size > 0.15 {
                    blit_wh(&t.line, ctr, len * 1.05, w * 1.8, ang, false, Color::new(0.5, 0.6, 0.82, c.a * 0.6));
                }
                blit_wh(&t.line, ctr, len, w.max(2.0), ang, false, c);
            }
            Kind::Spark if p.flags & UNDER != 0 => {
                // Painted star: a comet tail, then a round head with a
                // deeper-shade rim, the saturated colour and a pale hot core,
                // all opaque so they hold on a bright sky.
                let (ctr, len, ang) = Self::streak(p, scr, s, w, 0.13);
                let len = len.max(w * 3.5);
                let dir = vec2(p.vel.x, -p.vel.y).normalize_or_zero();
                let ctr = if dir == Vec2::ZERO { ctr } else { scr - dir * (len * 0.5 - w * 0.4) };
                blit_wh(&t.spark, ctr, len, w * 1.5, ang, false, with_a(c, c.a * 0.8));
                blit(&t.soft, scr, w * 2.0, 0.0, false, Color::new(c.r * 0.55, c.g * 0.5, c.b * 0.6, c.a * 0.9));
                blit(&t.soft, scr, w * 1.55, 0.0, false, c);
                let hot = (1.0 - p.age / 0.25).clamp(0.0, 1.0);
                blit(&t.soft, scr, w * 0.85, 0.0, false, with_a(mixc(c, WHITE, 0.6 + 0.4 * hot), c.a));
            }
            Kind::Ember if p.flags & UNDER != 0 => blit(&t.soft, scr, w * 1.6, 0.0, false, with_a(c, c.a * 0.7)),
            Kind::Flame => {
                // Painted fire body: the puff's blue-grey rim turns into a
                // deep red-brown edge under the fire tint, so the flame has
                // a shape on snow and sky; the additive pass only adds heat.
                let f = fire_ramp(0.12 + k * 0.85);
                let a = (1.0 - k).powf(1.2) * (k / 0.05).min(1.0);
                blit(&t.puffs[(p.rot * 3.0) as usize % 3], scr, w, p.rot, false, Color::new(f.r, f.g, f.b, 0.92 * a));
            }
            _ => {}
        }
    }

    fn draw_add(&self, p: &Particle, t: &FxTex, cam: &Cam, s: f32) {
        let k = p.age / p.life;
        let scr = self.screen(p, cam);
        let w = Self::size(p, k) * s;
        if Self::offscreen(scr, w) {
            return;
        }
        let a = Self::fade(p, k);
        let c = mixc(p.c0, p.c1, k);
        match p.kind {
            Kind::Glow => blit(&t.glow, scr, w, 0.0, false, with_a(c, c.a * a)),
            Kind::Flame => {
                blit(&t.glow, scr, w * 0.75, 0.0, false, with_a(fire_ramp(k * 0.6), 0.45 * a));
            }
            Kind::Spark if p.flags & UNDER != 0 => {
                // Over the painted star only a coloured halo is added, so the
                // sky never washes it out to white.
                blit(&t.glow, scr, w * 4.0, 0.0, false, with_a(c, 0.3 * a));
            }
            Kind::Spark => {
                let hot = (1.0 - p.age / 0.2).clamp(0.0, 1.0);
                let core = mixc(c, WHITE, hot * 0.8 + 0.25);
                let (ctr, len, ang) = Self::streak(p, scr, s, w, 0.085);
                blit_wh(&t.spark, ctr, len, w, ang, false, with_a(core, a));
                blit(&t.glow, scr, w * 3.2, 0.0, false, with_a(c, 0.45 * a));
            }
            Kind::Ember => blit(&t.glow, scr, w * 2.2, 0.0, false, with_a(c, c.a * a)),
            Kind::Glitter => {
                blit(&t.star, scr, w * 1.6, p.rot, false, with_a(c, a));
                blit(&t.glow, scr, w * 1.4, 0.0, false, with_a(c, 0.4 * a));
            }
            Kind::Sparkle => blit(&t.star, scr, w * 1.5, p.rot * 0.3, false, with_a(c, 0.8 * a)),
            _ => {}
        }
    }
}
