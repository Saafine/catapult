//! Procedural effect textures (soft glow, painted smoke puffs, sparks,
//! twinkle stars, bokeh flakes, snow clods, ice shards, shock rings) and the
//! additive-blend material. Built once on first use, so no pack files are
//! needed and every effect shares the same few textures.

use std::cell::RefCell;
use std::rc::Rc;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

pub struct FxTex {
    /// Gaussian glow, white.
    pub glow: Texture2D,
    /// Soft round dot, white (snow powder, far flakes).
    pub soft: Texture2D,
    /// Out-of-focus flake: a soft disc with a brighter rim.
    pub bokeh: Texture2D,
    /// Painted cartoon puffs: top-lit, a blue-grey deeper-shade edge.
    pub puffs: [Texture2D; 3],
    /// Soft noisy mist blob, top-lit, no edge.
    pub mist: Texture2D,
    /// A streak with a hot head at the right end (u = 1), white.
    pub spark: Texture2D,
    /// Four-point twinkle, white.
    pub star: Texture2D,
    /// Shockwave ring: a bright rim fading inward, white.
    pub ring: Texture2D,
    /// Painted snow clods.
    pub chunks: [Texture2D; 3],
    /// Painted ice shard (points up).
    pub shard: Texture2D,
    /// Gaussian across v, flat along u: ribbons and bolts.
    pub strip: Texture2D,
    /// A thin line tapering at both ends: wind streaks, speed lines.
    pub line: Texture2D,
    /// Blend: src * alpha + dst.
    pub add: Material,
}

thread_local! {
    static FX: RefCell<Option<Rc<FxTex>>> = const { RefCell::new(None) };
}

/// The shared effect textures; built on the first call (needs a GL context).
pub fn fx() -> Rc<FxTex> {
    FX.with(|c| c.borrow_mut().get_or_insert_with(|| Rc::new(FxTex::build())).clone())
}

/// Run `f` with additive blending, then restore the default material.
pub fn additive<F: FnOnce()>(f: F) {
    let t = fx();
    gl_use_material(&t.add);
    f();
    gl_use_default_material();
}

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn h2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

fn vnoise(x: f32, y: f32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - xi as f32, y - yi as f32);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = h2(xi, yi, seed);
    let b = h2(xi + 1, yi, seed);
    let c = h2(xi, yi + 1, seed);
    let d = h2(xi + 1, yi + 1, seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    let mut s = 0.0;
    let mut amp = 0.5;
    let mut f = 1.0;
    for o in 0..4 {
        s += amp * vnoise(x * f, y * f, seed + o);
        amp *= 0.5;
        f *= 2.0;
    }
    s
}

/// Build a texture from `f(nx, ny) -> [r, g, b, a]` with nx, ny in -1..1.
fn make<F: Fn(f32, f32) -> [f32; 4]>(w: u16, h: u16, f: F) -> Texture2D {
    let mut bytes = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            let nx = (x as f32 + 0.5) / w as f32 * 2.0 - 1.0;
            let ny = (y as f32 + 0.5) / h as f32 * 2.0 - 1.0;
            let c = f(nx, ny);
            for v in c {
                bytes.push((v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            }
        }
    }
    let t = Texture2D::from_rgba8(w, h, &bytes);
    t.set_filter(FilterMode::Linear);
    t
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// A painted cartoon puff: a union of lit spheres with a deeper-shade rim.
fn puff_tex(seed: u32) -> Texture2D {
    let mut circles = Vec::new();
    let n = 6;
    for i in 0..n {
        let a = i as f32 / n as f32 * 6.283 + h2(i, 1, seed) * 0.8;
        let d = if i == 0 { 0.0 } else { 0.22 + 0.18 * h2(i, 2, seed) };
        let r = if i == 0 { 0.5 } else { 0.28 + 0.14 * h2(i, 3, seed) };
        circles.push((a.cos() * d, a.sin() * d * 0.85 + 0.04, r));
    }
    let light = vec3(-0.45, -0.75, 0.55).normalize();
    make(128, 128, |x, y| {
        let mut best = -1.0f32;
        let mut nrm = vec3(0.0, 0.0, 1.0);
        for &(cx, cy, r) in &circles {
            let dx = (x - cx) / r;
            let dy = (y - cy) / r;
            let d2 = dx * dx + dy * dy;
            let inside = 1.0 - d2.sqrt();
            if inside > best {
                best = inside;
                nrm = vec3(dx, dy, (1.0 - d2).max(0.0).sqrt());
            }
        }
        let wob = (fbm(x * 3.0 + 7.0, y * 3.0, seed) - 0.5) * 0.14;
        let field = best + wob;
        let lam = nrm.dot(light).max(0.0);
        let shade = mix3([0.70, 0.78, 0.93], [1.0, 1.0, 1.0], smooth(0.05, 0.75, lam));
        let edge = smooth(0.0, 0.16, field);
        let col = mix3([0.56, 0.64, 0.82], shade, 0.35 + 0.65 * edge);
        let a = smooth(-0.02, 0.07, field);
        [col[0], col[1], col[2], a]
    })
}

fn chunk_tex(seed: u32) -> Texture2D {
    let p = [h2(1, 7, seed) * 6.28, h2(2, 7, seed) * 6.28, h2(3, 7, seed) * 6.28];
    make(48, 48, |x, y| {
        let r = (x * x + y * y).sqrt();
        let th = y.atan2(x);
        let rr = 0.74 + 0.11 * (3.0 * th + p[0]).sin() + 0.07 * (5.0 * th + p[1]).sin() + 0.04 * (7.0 * th + p[2]).sin();
        let d = rr - r;
        let a = smooth(-0.03, 0.04, d);
        let up = ((y + 1.0) * 0.5).clamp(0.0, 1.0);
        let mut col = mix3([1.0, 1.0, 1.0], [0.74, 0.83, 0.96], smooth(0.2, 1.0, up));
        let hl = (-((x + 0.3).powi(2) + (y + 0.35).powi(2)) * 14.0).exp();
        col = mix3(col, [1.0, 1.0, 1.0], hl);
        let edge = smooth(0.0, 0.16, d);
        col = mix3([0.47, 0.57, 0.77], col, edge);
        [col[0], col[1], col[2], a]
    })
}

fn shard_tex() -> Texture2D {
    // A kite pointing up, convex, so the distance is the min over edges.
    let pts = [vec2(0.0, -0.95), vec2(0.42, -0.05), vec2(0.0, 0.95), vec2(-0.32, 0.1)];
    make(32, 48, |x, y| {
        let q = vec2(x, y);
        let mut d = f32::MAX;
        for i in 0..4 {
            let a = pts[i];
            let b = pts[(i + 1) % 4];
            let e = (b - a).normalize();
            let nrm = vec2(e.y, -e.x);
            d = d.min((q - a).dot(nrm));
        }
        let a = smooth(-0.04, 0.03, d);
        let left = x < 0.04 * y;
        let mut col = if left { [0.93, 0.98, 1.0] } else { [0.62, 0.82, 0.97] };
        col = mix3(col, [0.80, 0.92, 1.0], smooth(-1.0, 1.0, y) * 0.4);
        let edge = smooth(0.0, 0.1, d);
        col = mix3([0.30, 0.50, 0.78], col, edge);
        [col[0], col[1], col[2], a]
    })
}

impl FxTex {
    fn build() -> FxTex {
        let glow = make(128, 128, |x, y| {
            let r2 = x * x + y * y;
            [1.0, 1.0, 1.0, (-r2 * 4.6).exp() * smooth(1.0, 0.85, r2.sqrt())]
        });
        let soft = make(32, 32, |x, y| {
            let r = (x * x + y * y).sqrt();
            [1.0, 1.0, 1.0, smooth(1.0, 0.35, r)]
        });
        let bokeh = make(128, 128, |x, y| {
            let r = (x * x + y * y).sqrt();
            [1.0, 1.0, 1.0, smooth(1.0, 0.86, r) * (0.7 + 0.3 * smooth(0.5, 0.92, r))]
        });
        let mist = make(128, 128, |x, y| {
            let r = (x * x + y * y).sqrt();
            let n = fbm(x * 2.2 + 3.0, y * 2.2 + 1.0, 91);
            let a = smooth(1.0, 0.25, r + (0.5 - n) * 0.5);
            let col = mix3([1.0, 1.0, 1.0], [0.78, 0.85, 0.96], smooth(-0.6, 0.9, y));
            [col[0], col[1], col[2], a * 0.95]
        });
        let spark = make(64, 16, |x, y| {
            let u = (x + 1.0) * 0.5;
            let along = u.clamp(0.06, 0.84);
            let k = (along - 0.06) / 0.78;
            let rad = 0.18 + 0.82 * k;
            let dx = (u - along) / 0.13;
            let dy = y / rad;
            let core = (-(dx * dx + dy * dy) * 2.4).exp();
            [1.0, 1.0, 1.0, core * (0.15 + 0.85 * k * k)]
        });
        let star = make(64, 64, |x, y| {
            let arm = |a: f32, b: f32| (-a.abs() * 3.2).exp() * (-(b * b) * 260.0).exp();
            let (dx, dy) = ((x + y) * 0.7071, (x - y) * 0.7071);
            let r2 = x * x + y * y;
            let v = arm(x, y) + arm(y, x) + 0.35 * (arm(dx * 1.8, dy) + arm(dy * 1.8, dx)) + (-r2 * 18.0).exp() + 0.35 * (-r2 * 5.0).exp();
            [1.0, 1.0, 1.0, v.min(1.0) * smooth(1.0, 0.9, r2.sqrt())]
        });
        let ring = make(256, 256, |x, y| {
            let r = (x * x + y * y).sqrt();
            let e = 0.86;
            let rim = (-((r - e) / 0.045).powi(2)).exp();
            let inner = if r < e { 0.32 * smooth(0.25, e, r) } else { 0.0 };
            [1.0, 1.0, 1.0, (rim + inner).min(1.0) * smooth(1.0, 0.96, r)]
        });
        let strip = make(4, 64, |_x, y| [1.0, 1.0, 1.0, (-(y * y) * 5.0).exp() * smooth(1.0, 0.85, y.abs())]);
        let line = make(64, 8, |x, y| {
            let u = (x + 1.0) * 0.5;
            [1.0, 1.0, 1.0, (std::f32::consts::PI * u).sin().powf(1.5) * (-(y * y) * 3.0).exp()]
        });
        let add = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)),
                    alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("additive material");
        FxTex {
            glow,
            soft,
            bokeh,
            puffs: [puff_tex(11), puff_tex(23), puff_tex(37)],
            mist,
            spark,
            star,
            ring,
            chunks: [chunk_tex(5), chunk_tex(17), chunk_tex(29)],
            shard: shard_tex(),
            strip,
            line,
            add,
        }
    }
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
varying lowp vec4 color;
varying lowp vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}"#;
