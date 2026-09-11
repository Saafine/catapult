//! Tiny particle system in world units. Purely cosmetic.

use macroquad::prelude::*;

use crate::sprites::{blit, Sprites};
use crate::Cam;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Puff,
    Sparkle,
    Feather,
    Ring,
    Flame,
    Snow,
    Confetti,
    /// A firework spark: bright core with a short trail, twinkling out.
    Spark,
}

pub struct Particle {
    pub kind: Kind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub age: f32,
    pub life: f32,
    /// World size (width) at birth.
    pub size: f32,
    pub rot: f32,
    pub spin: f32,
    pub gravity: f32,
    pub color: Color,
    pub behind: bool,
}

#[derive(Default)]
pub struct Particles {
    pub list: Vec<Particle>,
    seed: u32,
}

impl Particles {
    fn rnd(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.seed >> 8) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.rnd()
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.list {
            p.age += dt;
            p.vel.y -= p.gravity * dt;
            p.vel *= (1.0 - 1.5 * dt).max(0.0);
            p.pos += p.vel * dt;
            p.rot += p.spin * dt;
        }
        self.list.retain(|p| p.age < p.life);
        if self.list.len() > 400 {
            let extra = self.list.len() - 400;
            self.list.drain(0..extra);
        }
    }

    pub fn emit_dust(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let a = self.range(0.6, 2.5);
            let sp = self.range(2.0, 8.0);
            let v = vec2(a.cos() * sp * if self.rnd() > 0.5 { 1.0 } else { -1.0 }, a.sin() * sp);
            let size = self.range(0.6, 1.6);
            let life = self.range(0.5, 1.0);
            self.list.push(Particle {
                kind: Kind::Puff, pos: at, vel: v, age: 0.0, life, size,
                rot: 0.0, spin: 0.0, gravity: 4.0,
                color: Color::new(0.80, 0.68, 0.50, 0.7), behind: false,
            });
        }
    }

    pub fn emit_puffs(&mut self, at: Vec2, n: usize, color: Color) {
        for _ in 0..n {
            let v = vec2(self.range(-2.0, 2.0), self.range(-4.0, -1.0));
            let size = self.range(0.4, 0.9);
            self.list.push(Particle {
                kind: Kind::Puff, pos: at, vel: v, age: 0.0, life: 0.45, size,
                rot: 0.0, spin: 0.0, gravity: 0.0, color, behind: true,
            });
        }
    }

    pub fn emit_flame(&mut self, at: Vec2, vel: Vec2) {
        let v = vec2(-vel.x * 0.3 + self.range(-3.0, 3.0), -vel.y * 0.3 + self.range(-3.0, 3.0));
        let size = self.range(0.5, 1.1);
        let c = if self.rnd() > 0.5 { Color::new(1.0, 0.55, 0.15, 0.85) } else { Color::new(1.0, 0.85, 0.3, 0.85) };
        self.list.push(Particle {
            kind: Kind::Flame, pos: at, vel: v, age: 0.0, life: 0.4, size,
            rot: 0.0, spin: 0.0, gravity: -2.0, color: c, behind: true,
        });
    }

    pub fn emit_wind(&mut self, at: Vec2, vel: Vec2) {
        let off = vec2(self.range(-1.5, 1.5), self.range(-1.5, 2.5));
        let v = vec2(-vel.x * 0.9, -vel.y * 0.9);
        self.list.push(Particle {
            kind: Kind::Puff, pos: at + off, vel: v, age: 0.0, life: 0.35, size: 0.35,
            rot: 0.0, spin: 0.0, gravity: 0.0, color: Color::new(1.0, 1.0, 1.0, 0.35), behind: true,
        });
    }

    pub fn emit_feathers(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let v = vec2(self.range(-7.0, 7.0), self.range(0.0, 9.0));
            let size = self.range(0.35, 0.6);
            let spin = self.range(-6.0, 6.0);
            let rot = self.range(0.0, 6.28);
            self.list.push(Particle {
                kind: Kind::Feather, pos: at, vel: v, age: 0.0, life: 1.4, size,
                rot, spin, gravity: 3.0, color: WHITE, behind: false,
            });
        }
    }

    pub fn emit_sparkles(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let a = self.range(0.0, 6.28);
            let sp = self.range(3.0, 9.0);
            let size = self.range(0.4, 0.9);
            let spin = self.range(-4.0, 4.0);
            self.list.push(Particle {
                kind: Kind::Sparkle, pos: at, vel: vec2(a.cos() * sp, a.sin() * sp), age: 0.0, life: 0.7,
                size, rot: 0.0, spin, gravity: 6.0, color: Color::new(1.0, 0.95, 0.5, 1.0), behind: false,
            });
        }
    }

    /// Snow spray: white puffs with a little upward toss.
    pub fn emit_snow(&mut self, at: Vec2, base: Vec2, n: usize) {
        for _ in 0..n {
            let v = base + vec2(self.range(-4.0, 4.0), self.range(0.0, 5.0));
            let size = self.range(0.2, 0.45);
            let life = self.range(0.5, 1.0);
            self.list.push(Particle {
                kind: Kind::Snow, pos: at, vel: v, age: 0.0, life, size,
                rot: 0.0, spin: 0.0, gravity: 6.0, color: Color::new(1.0, 1.0, 1.0, 0.9), behind: false,
            });
        }
    }

    /// Chunky snow clods that arc under gravity.
    pub fn emit_clods(&mut self, at: Vec2, base: Vec2, n: usize) {
        for _ in 0..n {
            let v = base + vec2(self.range(-9.0, 9.0), self.range(2.0, 12.0));
            let size = self.range(0.25, 0.7);
            let life = self.range(0.6, 1.1);
            self.list.push(Particle {
                kind: Kind::Snow, pos: at, vel: v, age: 0.0, life, size,
                rot: 0.0, spin: 0.0, gravity: 18.0, color: Color::new(1.0, 1.0, 1.0, 1.0), behind: false,
            });
        }
    }

    /// Big slow snow plume: fills a good part of the frame for a moment.
    pub fn emit_plume(&mut self, at: Vec2, base: Vec2, n: usize) {
        for _ in 0..n {
            let v = base + vec2(self.range(-5.0, 5.0), self.range(0.0, 6.0));
            let size = self.range(1.5, 3.2);
            let life = self.range(0.5, 0.9);
            self.list.push(Particle {
                kind: Kind::Puff, pos: at, vel: v, age: 0.0, life, size,
                rot: 0.0, spin: 0.0, gravity: 3.0, color: Color::new(1.0, 1.0, 1.0, 0.8), behind: true,
            });
        }
    }

    /// Coloured confetti that tumbles down.
    pub fn emit_confetti(&mut self, at: Vec2, n: usize) {
        let palette = [
            Color::new(1.0, 0.35, 0.35, 1.0), Color::new(0.35, 0.85, 0.5, 1.0),
            Color::new(1.0, 0.85, 0.3, 1.0), Color::new(0.4, 0.7, 1.0, 1.0), WHITE,
        ];
        for i in 0..n {
            let a = self.range(0.0, 6.28);
            let sp = self.range(4.0, 14.0);
            let size = self.range(0.25, 0.5);
            let spin = self.range(-10.0, 10.0);
            let color = palette[i % palette.len()];
            self.list.push(Particle {
                kind: Kind::Confetti, pos: at, vel: vec2(a.cos() * sp, a.sin() * sp + 4.0), age: 0.0, life: 1.6,
                size, rot: a, spin, gravity: 5.0, color, behind: false,
            });
        }
    }

    /// Launch burst drawn behind the dog, so the silhouette stays visible.
    pub fn emit_burst_behind(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            let a = self.range(0.0, 6.28);
            let sp = self.range(6.0, 16.0);
            let size = self.range(0.5, 1.1);
            let spin = self.range(-4.0, 4.0);
            self.list.push(Particle {
                kind: Kind::Sparkle, pos: at, vel: vec2(a.cos() * sp, a.sin() * sp), age: 0.0, life: 0.55,
                size, rot: 0.0, spin, gravity: 4.0, color: Color::new(1.0, 0.95, 0.6, 0.9), behind: true,
            });
        }
        self.list.push(Particle {
            kind: Kind::Ring, pos: at, vel: Vec2::ZERO, age: 0.0, life: 0.45, size: 2.0,
            rot: 0.0, spin: 0.0, gravity: 0.0, color: WHITE, behind: true,
        });
    }

    /// A fireworks shell bursting at `at`: a white flash, then a sphere of
    /// comet-tailed streaks in the shell colour that droop under gravity and
    /// twinkle out, with a few long-lived willow strands and a puff of smoke.
    pub fn emit_fireworks(&mut self, at: Vec2, color: Color, n: usize) {
        self.list.push(Particle {
            kind: Kind::Puff, pos: at, vel: Vec2::ZERO, age: 0.0, life: 0.18, size: 7.0,
            rot: 0.0, spin: 0.0, gravity: 0.0, color: Color::new(1.0, 1.0, 0.95, 0.95), behind: false,
        });
        for i in 0..3 {
            let a = self.range(0.0, 6.28);
            self.list.push(Particle {
                kind: Kind::Puff, pos: at + vec2(a.cos(), a.sin()) * 0.6, vel: vec2(a.cos(), a.sin()) * 1.5, age: 0.0, life: 2.2 + 0.3 * i as f32,
                size: 3.0, rot: 0.0, spin: 0.0, gravity: -0.4, color: Color::new(0.55, 0.55, 0.62, 0.35), behind: true,
            });
        }
        let n = n.max(24);
        for i in 0..n {
            // A sphere: even angles with a little jitter, near-equal speeds so
            // the burst reads as one round bloom; every sixth spark is a willow
            // strand that lingers and sags.
            let a = i as f32 / n as f32 * 6.2832 + self.range(-0.12, 0.12);
            let willow = i % 6 == 0;
            let sp = if willow { self.range(11.0, 14.0) } else { self.range(17.0, 21.0) };
            let life = if willow { self.range(1.9, 2.4) } else { self.range(1.1, 1.5) };
            let size = if willow { 0.30 } else { self.range(0.32, 0.42) };
            self.list.push(Particle {
                kind: Kind::Spark, pos: at, vel: vec2(a.cos() * sp, a.sin() * sp), age: 0.0, life,
                size, rot: a, spin: 0.0, gravity: if willow { 6.0 } else { 4.0 }, color, behind: false,
            });
        }
    }

    pub fn emit_ring(&mut self, at: Vec2, color: Color) {
        self.emit_ring_sized(at, color, 1.5, 0.5);
    }

    pub fn emit_ring_sized(&mut self, at: Vec2, color: Color, size: f32, life: f32) {
        // One ring alive at a time: a chain of pickups must not stack them.
        self.list.retain(|p| p.kind != Kind::Ring);
        self.list.push(Particle {
            kind: Kind::Ring, pos: at, vel: Vec2::ZERO, age: 0.0, life, size,
            rot: 0.0, spin: 0.0, gravity: 0.0, color, behind: false,
        });
    }

    pub fn draw_behind(&self, sp: &Sprites, cam: &Cam) {
        self.draw(sp, cam, true);
    }

    pub fn draw_front(&self, sp: &Sprites, cam: &Cam) {
        self.draw(sp, cam, false);
    }

    fn draw(&self, sp: &Sprites, cam: &Cam, behind: bool) {
        let s = cam.scale();
        for p in &self.list {
            if p.behind != behind {
                continue;
            }
            let k = p.age / p.life;
            let scr = cam.to_screen(p.pos.x, p.pos.y);
            let mut c = p.color;
            if p.kind == Kind::Spark {
                // White-hot for the first beat, then the shell colour; the
                // tail is a comet of shrinking dots along the recent path.
                let hot = (1.0 - p.age / 0.22).clamp(0.0, 1.0);
                let mut col = Color::new(c.r + (1.0 - c.r) * hot, c.g + (1.0 - c.g) * hot, c.b + (1.0 - c.b) * hot, 1.0);
                let tw = if k > 0.6 { 0.55 + 0.45 * (p.age * 40.0 + p.rot * 9.0).sin() } else { 1.0 };
                let fade = (1.0 - k * k) * tw;
                let head = p.size * s * 0.5;
                let segs = 9;
                for j in (1..=segs).rev() {
                    let back = j as f32 * 0.03 * (0.6 + 0.4 * (1.0 - k));
                    let q = cam.to_screen(p.pos.x - p.vel.x * back, p.pos.y - p.vel.y * back + 0.5 * p.gravity * back * back);
                    let kk = 1.0 - j as f32 / (segs as f32 + 1.0);
                    col.a = fade * (0.12 + 0.55 * kk);
                    draw_circle(q.x, q.y, head * (0.35 + 0.65 * kk), col);
                }
                col.a = fade;
                draw_circle(scr.x, scr.y, head, col);
                let mut glow = col;
                glow.a = fade * 0.35;
                draw_circle(scr.x, scr.y, head * 2.2, glow);
                continue;
            }
            let (tex, size) = match p.kind {
                Kind::Puff => {
                    c.a *= 1.0 - k;
                    (&sp.puff, p.size * (1.0 + k * 1.2))
                }
                Kind::Flame => {
                    c.a *= 1.0 - k;
                    (&sp.puff, p.size * (1.0 - k * 0.5))
                }
                Kind::Sparkle => {
                    c.a *= 1.0 - k * k;
                    (&sp.sparkle, p.size * (1.0 - k * 0.3))
                }
                Kind::Feather => {
                    c.a *= (1.0 - k).min(1.0) * 1.2;
                    (&sp.feather, p.size)
                }
                Kind::Ring => {
                    c.a *= 1.0 - k;
                    (&sp.ring, p.size * (1.0 + k * 4.0))
                }
                Kind::Snow => {
                    c.a *= 1.0 - k * k;
                    (&sp.snowflake, p.size * (1.0 + k * 0.6))
                }
                Kind::Confetti => {
                    c.a *= (1.5 - k).min(1.0);
                    (&sp.confetti, p.size)
                }
                Kind::Spark => unreachable!("drawn above"),
            };
            blit(tex, scr, size * s, p.rot, false, c);
        }
    }
}
