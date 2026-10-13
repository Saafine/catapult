//! Scene drawing with sprites: sky, parallax layers, ground dressing,
//! the workshop tower and ramp, the dog, pickups.

use hamsters_core::{ramp_point, Entity, EntityKind, LIP, PILOT_RADIUS, RAMP_TOP, ROOF_Y, SPACE_ALTITUDE};
use macroquad::prelude::*;

use crate::city::{City, Ground};
use crate::sprites::{blit, blit_foot, blit_pivot, blit_standing, blit_wh, Sprites};
use crate::Cam;

pub const SEED: Color = Color::new(1.0, 0.86, 0.30, 1.0);
pub const OUTLINE: Color = Color::new(0.29, 0.17, 0.13, 1.0);
// The ski jump is built from the same orange timber as the FALKBUILT
// letters in the key art: lit top edge, deeper orange outline, no ink.
const WOOD: Color = Color::new(0.93, 0.57, 0.24, 1.0);
const WOOD_LIGHT: Color = Color::new(1.0, 0.78, 0.48, 1.0);
const WOOD_SHADE: Color = Color::new(0.78, 0.42, 0.16, 1.0);
const WOOD_EDGE: Color = Color::new(0.52, 0.25, 0.09, 1.0);
const SNOW: Color = Color::new(0.97, 0.98, 1.0, 1.0);
const SNOW_SHADE: Color = Color::new(0.79, 0.87, 0.95, 1.0);

/// Fur mid-tone per dog, used to tint the greyscale paw sprite.

// World sizes of sprites, in meters (width).
const DOG_W: f32 = 3.3;
/// Mirrored in `anim::FLY_GROW` (DOG_FLY_SCALE - 1) for the cape's scale.
const DOG_FLY_SCALE: f32 = 1.8;
/// Width of the launch building (the Falkbuilt factory) in meters.
const TOWER_W: f32 = 40.0;

pub fn lerp(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

/// Cheap deterministic hash -> 0..1, for scattering scenery.
pub fn hash(i: i32, salt: u32) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// 0 at ground level, 1 in full space.
pub fn space_factor(cam: &Cam) -> f32 {
    ((cam.y - 120.0) / (SPACE_ALTITUDE - 120.0)).clamp(0.0, 1.0)
}

/// The long afternoon: 0 at the tower, 1 by about 450 m out. The sun sinks
/// and warms with distance, so a far flight looks like a journey.
pub fn dusk_factor(cam: &Cam) -> f32 {
    ((cam.x - 100.0) / 350.0).clamp(0.0, 1.0)
}

/// Vertical parallax of a scenery layer with horizontal parallax `factor`.
/// Deliberately much larger than the horizontal one for the far layers, so
/// climbing sinks the whole landscape down the screen and the flight reads
/// as height: at 30 m the mountains fill the lower half, at 60 m the far
/// ridge sits on the bottom edge, and by 90 m only sky and cloud are left.
pub fn para_y(factor: f32) -> f32 {
    0.25 + 0.6 * factor
}

/// How high the camera is for the scenery: 0 on the ground, 1 by 260 m.
/// Layers wash out towards the sky colour with it (aerial perspective).
pub fn height_haze(cam: &Cam) -> f32 {
    smoothstep(30.0, 260.0, cam.y) * 0.6
}

pub const HAZE: Color = Color::new(0.74, 0.82, 0.97, 1.0);

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let k = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// Tint for everything lit by the sky: white at noon, cool violet at dusk.
pub fn daylight(cam: &Cam) -> Color {
    lerp(WHITE, Color::new(0.72, 0.68, 0.85, 1.0), dusk_factor(cam))
}

pub fn mul(a: Color, b: Color) -> Color {
    Color::new(a.r * b.r, a.g * b.g, a.b * b.b, a.a)
}

// ---------------------------------------------------------------------------
// Sky
// ---------------------------------------------------------------------------

/// Christmas twilight that turns to space with altitude, with stars, moon and
/// aurora ribbons.
pub fn sky(sp: &Sprites, cam: &Cam, aim_k: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;
    let space = space_factor(cam);
    let dusk = dusk_factor(cam);
    // Bright winter day: deep cyan up top, pale at the horizon. With distance
    // the afternoon wears on: peach at the horizon, then violet, the top
    // deepening to evening blue. Space still wins with altitude.
    // The blue deepens with height well before space, so a climb shows in
    // the sky even while the landscape is still in frame.
    let alt = smoothstep(20.0, 220.0, cam.y) * 0.6;
    let day_top = lerp(Color::new(0.33, 0.61, 0.87, 1.0), Color::new(0.12, 0.20, 0.48, 1.0), dusk);
    let day_top = lerp(day_top, Color::new(0.10, 0.38, 0.85, 1.0), alt);
    let day_mid = lerp(Color::new(0.62, 0.81, 0.95, 1.0), Color::new(0.55, 0.45, 0.70, 1.0), smoothstep(0.3, 1.0, dusk));
    let day_mid = lerp(day_mid, Color::new(0.34, 0.66, 0.96, 1.0), alt);
    let peach = lerp(Color::new(0.88, 0.94, 0.99, 1.0), Color::new(1.0, 0.72, 0.47, 1.0), smoothstep(0.15, 0.6, dusk));
    let day_low = lerp(peach, Color::new(0.30, 0.26, 0.50, 1.0), smoothstep(0.6, 1.0, dusk));
    let top = lerp(day_top, Color::new(0.01, 0.01, 0.05, 1.0), space);
    let mid = lerp(day_mid, Color::new(0.03, 0.04, 0.12, 1.0), space);
    let low = lerp(day_low, Color::new(0.10, 0.14, 0.32, 1.0), space);
    // Painted winter sky: a saturated zenith, a clear cyan middle, and a
    // milky haze that thickens toward the landscape's horizon (wherever the
    // far range sits on screen), warmed on the side of the sun.
    let zen = lerp(top, Color::new(0.17, 0.43, 0.80, 1.0), 0.4 * (1.0 - dusk) * (1.0 - space));
    let strips = 96;
    let h = sh / strips as f32;
    for i in 0..strips {
        let k = i as f32 / (strips - 1) as f32;
        let c = if k < 0.5 { lerp(zen, mid, (k / 0.5).powf(1.1)) } else { lerp(mid, low, ((k - 0.5) / 0.5).powf(1.6)) };
        draw_rectangle(0.0, i as f32 * h, sw, h + 1.0, c);
    }
    // Stars come out with the dusk, and fully once the sky darkens towards space.
    let star_a = space.max(0.7 * dusk);
    let star_span = 0.55 + 0.45 * space;
    for i in 0..160 {
        let x = (hash(i, 11) * sw * 1.4 - cam.x * 0.02 * (0.5 + hash(i, 15))).rem_euclid(sw);
        let y = hash(i, 12) * sh * star_span;
        let tw = 0.6 + 0.4 * (t * (1.0 + hash(i, 16) * 2.0) + hash(i, 17) * 6.0).sin();
        let r = 0.8 + hash(i, 13) * 1.6;
        draw_circle(x, y, r, Color::new(1.0, 1.0, 1.0, star_a * tw * (0.5 + 0.5 * hash(i, 14))));
        if hash(i, 18) > 0.9 {
            blit(&sp.star, vec2(x, y), 10.0 + 8.0 * hash(i, 19), t * 0.3, false, Color::new(1.0, 1.0, 0.9, 0.35 * star_a * tw));
        }
    }
    // Sun by day, moon once we are up in the dark; both drift with the camera.
    // The sun sits left of the pilot picker while aiming (aim_k 1), and
    // further right in flight.
    let mx = sw * (0.78 - 0.23 * aim_k) - cam.x * 0.02 * cam.zoom;
    // The sun sinks towards the horizon and warms as the afternoon wears on.
    let my = sh * 0.20 + cam.y * 0.03 * cam.zoom + dusk * sh * 0.50;
    if space < 1.0 {
        let warm = lerp(WHITE, Color::new(1.0, 0.72, 0.45, 1.0), dusk);
        sun_halo(vec2(mx, my), t, warm, 1.0 - space);
        blit(&sp.sun, vec2(mx, my), 230.0, t * 0.05, false, Color::new(warm.r, warm.g, warm.b, 1.0 - space));
        crate::post::set_sun(vec2(mx, my), 115.0 * 0.8, warm, 1.0 - space);
    }
    if space > 0.0 {
        // The moon waits until the sun has nearly gone: never both in one sky.
        blit(&sp.moon, vec2(mx - 200.0, my + 40.0), 200.0, 0.0, false, Color::new(1.0, 1.0, 1.0, smoothstep(0.7, 0.95, space)));
    }
    // Aurora is the reward of dusk: faint at noon, full in the evening, gone in space.
    aurora(cam, t, (1.0 - space) * (0.1 + 0.9 * dusk));
    // Haze rising off the land toward the horizon, then (from high up) the
    // far horizon itself: a pale distant range over a sea of low cloud.
    sky_haze(cam, mx, dusk, space);
    far_horizon(sp, cam, dusk, space);
    // Earth glow at the bottom of the screen once in space.
    if space > 0.0 {
        let strips = 12;
        let hh = 160.0 / strips as f32;
        for i in 0..strips {
            let k = i as f32 / strips as f32;
            let a = 0.35 * space * k;
            draw_rectangle(0.0, sh - 160.0 + i as f32 * hh, sw, hh + 1.0, Color::new(0.35, 0.65, 1.0, a));
        }
    }
}

fn mesh_vertex(p: Vec2, c: Color) -> macroquad::models::Vertex {
    macroquad::models::Vertex { position: vec3(p.x, p.y, 0.0), uv: vec2(0.0, 0.0), color: c.into(), normal: vec4(0.0, 0.0, 1.0, 0.0) }
}

/// A soft radial glow: `c` at the centre falling off smoothly to nothing at
/// radius `r` (a fan of rings, so there are no visible bands).
pub fn glow(center: Vec2, r: f32, c: Color) {
    let segs = 48u16;
    let rings = 6u16;
    let mut vertices = Vec::with_capacity(1 + (segs as usize) * rings as usize);
    let mut indices: Vec<u16> = Vec::new();
    vertices.push(mesh_vertex(center, c));
    for ring in 1..=rings {
        let k = ring as f32 / rings as f32;
        let a = c.a * (1.0 - k).powf(2.2);
        for i in 0..segs {
            let ang = i as f32 / segs as f32 * std::f32::consts::TAU;
            vertices.push(mesh_vertex(center + vec2(ang.cos(), ang.sin()) * r * k, Color::new(c.r, c.g, c.b, a)));
        }
    }
    for i in 0..segs {
        indices.extend_from_slice(&[0, 1 + i, 1 + (i + 1) % segs]);
    }
    for ring in 1..rings {
        let a0 = 1 + (ring - 1) * segs;
        let b0 = 1 + ring * segs;
        for i in 0..segs {
            let j = (i + 1) % segs;
            indices.extend_from_slice(&[a0 + i, b0 + i, a0 + j, a0 + j, b0 + i, b0 + j]);
        }
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

/// The sun's atmosphere: a wide warm bloom in the sky around it, a tighter
/// bright corona, and a slow wheel of soft rays.
fn sun_halo(c: Vec2, t: f32, warm: Color, a: f32) {
    if a <= 0.0 {
        return;
    }
    glow(c, 520.0, Color::new(1.0, 0.95 * warm.g, 0.80 * warm.b, 0.30 * a));
    glow(c, 230.0, Color::new(1.0, 0.98 * warm.g, 0.88 * warm.b, 0.45 * a));
    let n = 14;
    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    for i in 0..n {
        let base = i as f32 / n as f32 * std::f32::consts::TAU + t * 0.015 + 0.3 * hash(i, 401);
        let len = 260.0 + 220.0 * hash(i, 402);
        let half = 0.035 + 0.04 * hash(i, 403);
        let al = a * (0.10 + 0.08 * (t * 0.4 + i as f32 * 1.7).sin().abs());
        let k = vertices.len() as u16;
        vertices.push(mesh_vertex(c, Color::new(1.0, 0.97 * warm.g, 0.85 * warm.b, al)));
        for s in [-1.0, 1.0] {
            let ang = base + s * half;
            vertices.push(mesh_vertex(c + vec2(ang.cos(), ang.sin()) * len, Color::new(1.0, 0.97 * warm.g, 0.85 * warm.b, 0.0)));
        }
        indices.extend_from_slice(&[k, k + 1, k + 2]);
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

/// Atmospheric haze: a milky band that thickens toward the land's horizon
/// (where the far range stands on screen), warmer under the sun and peach
/// as the afternoon goes on.
fn sky_haze(cam: &Cam, sun_x: f32, dusk: f32, space: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let horizon = cam.to_screen_parallax(0.0, 0.0, para_y(0.10)).y - 30.0 * cam.zoom.powf(0.4);
    let a = (1.0 - space) * 0.55;
    if a <= 0.0 || horizon - 260.0 > sh {
        return;
    }
    let cool = lerp(Color::new(0.90, 0.95, 1.0, 1.0), Color::new(1.0, 0.80, 0.62, 1.0), smoothstep(0.1, 0.7, dusk));
    let cols = 24;
    let rows = 10;
    let top = horizon - 260.0;
    let bottom = sh.max(horizon + 10.0);
    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    for c in 0..=cols {
        let x = sw * c as f32 / cols as f32;
        // Warmer and denser under the sun.
        let near_sun = (1.0 - ((x - sun_x).abs() / (sw * 0.6)).min(1.0)).powf(1.5);
        let col = lerp(cool, Color::new(1.0, 0.94, 0.80, 1.0), near_sun * 0.6 * (1.0 - dusk));
        for r in 0..=rows {
            let k = r as f32 / rows as f32;
            let y = top + (bottom - top) * k;
            let ramp = ((y - top) / (horizon - top)).clamp(0.0, 1.0);
            let al = a * ramp.powf(1.8) * (0.8 + 0.2 * near_sun);
            vertices.push(mesh_vertex(vec2(x, y), Color::new(col.r, col.g, col.b, al)));
        }
    }
    let stride = (rows + 1) as u16;
    for c in 0..cols as u16 {
        for r in 0..rows as u16 {
            let i0 = c * stride + r;
            let i1 = i0 + stride;
            indices.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1]);
        }
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

/// From high up the real horizon stays in view: a pale, far-off range of
/// peaks (procedural, never repeating) above a sea of low cloud. It sinks
/// slowly with altitude, so a climb still reads as height, and it is
/// hidden behind the nearer ranges at ground level.
fn far_horizon(sp: &Sprites, cam: &Cam, dusk: f32, space: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let show = smoothstep(25.0, 90.0, cam.y) * (1.0 - space * 0.85);
    if show <= 0.0 {
        return;
    }
    let base = sh * (0.66 + cam.y / 1300.0);
    if base - 120.0 > sh {
        return;
    }
    let light = daylight(cam);
    let far = mul(lerp(Color::new(0.70, 0.78, 0.95, 1.0), Color::new(0.80, 0.70, 0.82, 1.0), dusk), light);
    let foot = mul(lerp(Color::new(0.90, 0.94, 1.0, 1.0), Color::new(1.0, 0.84, 0.74, 1.0), dusk), light);
    let snow = mul(Color::new(0.97, 0.98, 1.0, 1.0), light);
    let shade = mul(Color::new(0.60, 0.68, 0.90, 1.0), light);
    let scroll = cam.x * 0.03 * crate::PPM * cam.zoom;
    let tri = |u: f32| 1.0 - 2.0 * (u.rem_euclid(1.0) - 0.5).abs();
    let peak = |x: f32| -> f32 {
        let u = (x + scroll) * 0.0035;
        0.62 * tri(u * 0.9).powf(1.7) + 0.33 * tri(u * 2.3 + 0.37).powf(1.4) + 0.05 * (u * 19.0).sin()
    };
    let ridge = |x: f32| -> f32 { base - 22.0 - 120.0 * peak(x) };
    let cols = 128;
    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let bottom = sh + 4.0;
    for c in 0..=cols {
        let x = sw * c as f32 / cols as f32;
        let y = ridge(x);
        let left = ridge(x - 6.0);
        // Faces climbing to the right are lit, falling ones in shade.
        let lit = if y < left { 1.0 } else { 0.0 };
        let face = lerp(shade, far, 0.35 + 0.65 * lit);
        let cap = smoothstep(0.42, 0.62, peak(x)) * 0.95;
        let top_c = lerp(face, lerp(shade, snow, 0.4 + 0.6 * lit), cap);
        vertices.push(mesh_vertex(vec2(x, y), Color::new(top_c.r, top_c.g, top_c.b, show)));
        vertices.push(mesh_vertex(vec2(x, base + 10.0), Color::new(foot.r, foot.g, foot.b, show)));
        vertices.push(mesh_vertex(vec2(x, bottom), Color::new(foot.r, foot.g, foot.b, show)));
    }
    for c in 0..cols as u16 {
        let i = c * 3;
        let j = i + 3;
        indices.extend_from_slice(&[i, j, i + 1, i + 1, j, j + 1, i + 1, j + 1, i + 2, i + 2, j + 1, j + 2]);
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
    // The cloud sea: rows of soft puffs along the foot of the range.
    let tint = mul(lerp(WHITE, Color::new(1.0, 0.88, 0.80, 1.0), dusk), light);
    for row in 0..3 {
        let y = base + 4.0 + row as f32 * 26.0;
        let size = 120.0 + row as f32 * 40.0;
        let par = 0.05 + row as f32 * 0.03;
        let shift = (cam.x * par * crate::PPM * cam.zoom).rem_euclid(size * 0.7);
        let mut i = -2;
        let mut x = -shift - size;
        while x < sw + size {
            let k = i + ((cam.x * par * crate::PPM * cam.zoom) / (size * 0.7)).floor() as i32;
            let w = size * (0.8 + 0.6 * hash(k, 410 + row));
            let yy = y - 14.0 * hash(k, 420 + row);
            blit_wh(&sp.puff, vec2(x, yy), w * 1.6, w * 0.55, 0.0, false, Color::new(tint.r, tint.g, tint.b, show * (0.55 + 0.15 * row as f32)));
            x += size * 0.7;
            i += 1;
        }
    }
}

fn aurora(cam: &Cam, t: f32, strength: f32) {
    if strength <= 0.0 {
        return;
    }
    let sw = screen_width();
    let sh = screen_height();
    let ribbons = [
        (Color::new(0.35, 1.0, 0.65, 1.0), 0.16, 0.9, 0.0),
        (Color::new(0.45, 0.85, 1.0, 1.0), 0.24, 1.3, 2.1),
        (Color::new(0.75, 0.45, 1.0, 1.0), 0.11, 0.7, 4.0),
    ];
    let cols = 40;
    let rows = 8;
    for (color, base, freq, phase) in ribbons {
        let mut vertices = Vec::with_capacity((cols + 1) * (rows + 1));
        let mut indices: Vec<u16> = Vec::with_capacity(cols * rows * 6);
        for c in 0..=cols {
            let x = sw * c as f32 / cols as f32;
            let wx = (x + cam.x * 0.15 * cam.zoom) * 0.004 * freq;
            let y0 = sh * base + (wx * 2.0 + t * 0.25 + phase).sin() * 40.0 + (wx * 5.0 - t * 0.4).sin() * 14.0;
            let len = 130.0 + (wx * 3.0 + t * 0.6 + phase).sin() * 50.0;
            let a = strength * (0.10 + 0.10 * (wx * 7.0 + t * 1.3).sin().abs());
            for r in 0..=rows {
                let k = r as f32 / rows as f32;
                let fade = (1.0 - (k - 0.35).abs() * 2.0).clamp(0.0, 1.0);
                vertices.push(macroquad::models::Vertex {
                    position: vec3(x, y0 + len * k, 0.0),
                    uv: vec2(0.0, 0.0),
                    color: Color::new(color.r, color.g, color.b, a * fade).into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
            }
        }
        let stride = (rows + 1) as u16;
        for c in 0..cols as u16 {
            for r in 0..rows as u16 {
                let i0 = c * stride + r;
                let i1 = i0 + stride;
                indices.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1]);
            }
        }
        draw_mesh(&Mesh { vertices, indices, texture: None });
    }
}

/// Falling snow in screen space, thinning out with altitude.
pub fn snowfall(sp: &Sprites, cam: &Cam, vel: Vec2, calm: f32) {
    crate::vfx::snowfall(sp, cam, vel, calm);
}

/// Clouds scattered through world space, parallax 0.5 in x.
pub fn clouds(sp: &Sprites, cam: &Cam) {
    cloud_field(sp, cam, 0.5, 10.0, 330.0, 150, 5.0, 8.0, Color::new(0.92, 0.95, 1.0, 0.95), 21);
    // Nearer, bigger clouds at real altitudes so the sky above the arc has
    // things to fly past.
    cloud_field(sp, cam, 0.85, 30.0, 320.0, 60, 7.0, 10.0, Color::new(1.0, 1.0, 1.0, 1.0), 61);
}

fn cloud_field(sp: &Sprites, cam: &Cam, factor: f32, y0: f32, y_span: f32, n: i32, size0: f32, size_span: f32, tint: Color, salt: u32) {
    let sw = screen_width();
    let sh = screen_height();
    const PERIOD: f32 = 400.0;
    let s = cam.scale();
    let vis_l = cam.x * factor - sw / s;
    let vis_r = cam.x * factor + sw / s;
    let first = (vis_l / PERIOD).floor() as i32;
    let last = (vis_r / PERIOD).ceil() as i32;
    for tile in first..=last {
        for i in 0..n {
            let k = i + tile * 1000;
            let wx = tile as f32 * PERIOD + hash(k, salt) * PERIOD;
            let wy = y0 + hash(i, salt + 1) * y_span;
            let size = (size0 + hash(i, salt + 2) * size_span) * s;
            let p = cam.to_screen_parallax_xy(wx, wy, factor, 1.0);
            if p.x < -size || p.x > sw + size || p.y < -size || p.y > sh + size {
                continue;
            }
            let tex = &sp.clouds[(hash(i, salt + 3) * 3.0) as usize % 3];
            blit(tex, p, size, 0.0, hash(i, salt + 4) > 0.5, tint);
        }
    }
}

/// Snow gusts streaming past in the near foreground at speed: soft
/// horizontal wisps at parallax 1.7 that lengthen with velocity.
pub fn gusts(sp: &Sprites, cam: &Cam, speed: f32, t: f32) {
    let k = ((speed - 16.0) / 28.0).clamp(0.0, 1.0);
    if k <= 0.0 {
        return;
    }
    let sw = screen_width();
    let sh = screen_height();
    let s = cam.scale();
    let factor = 1.7;
    const PERIOD: f32 = 120.0;
    let vis_l = cam.x * factor - sw / s;
    let vis_r = cam.x * factor + sw / s;
    let first = (vis_l / PERIOD).floor() as i32;
    let last = (vis_r / PERIOD).ceil() as i32;
    for tile in first..=last {
        for i in 0..14 {
            let kk = i + tile * 1000;
            let wx = tile as f32 * PERIOD + hash(kk, 171) * PERIOD;
            let wy = cam.y + (hash(i, 172) - 0.5) * sh / s * 1.2;
            let p = cam.to_screen_parallax_xy(wx, wy, factor, 1.0);
            if p.x < -200.0 || p.x > sw + 200.0 {
                continue;
            }
            let len = (60.0 + 220.0 * k) * (0.6 + hash(i, 173));
            let wisp = (t * 3.0 + i as f32).sin() * 0.5 + 0.5;
            blit_wh(&sp.puff, p, len, 14.0 + 10.0 * hash(i, 174), 0.0, false, Color::new(1.0, 1.0, 1.0, 0.34 * k * (0.5 + 0.5 * wisp)));
        }
    }
}

/// Wispy near clouds in front of the action, parallax > 1, so speed is visible
/// even high up where the ground is gone.
/// `cap` limits the cloud alpha (0..1) so they never hide the dog.
pub fn near_clouds(sp: &Sprites, cam: &Cam, speed: f32, cap: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let s = cam.scale();
    let factor = 1.35;
    const PERIOD: f32 = 300.0;
    let a = (((speed - 10.0) / 30.0).clamp(0.0, 1.0) * 0.5).min(cap);
    if a <= 0.0 {
        return;
    }
    let vis_l = cam.x * factor - sw / s;
    let vis_r = cam.x * factor + sw / s;
    let first = (vis_l / PERIOD).floor() as i32;
    let last = (vis_r / PERIOD).ceil() as i32;
    for tile in first..=last {
        for i in 0..40 {
            let k = i + tile * 1000;
            let wx = tile as f32 * PERIOD + hash(k, 121) * PERIOD;
            let wy = 14.0 + hash(i, 122) * 500.0;
            let size = (9.0 + hash(i, 123) * 10.0) * s;
            let p = cam.to_screen_parallax_xy(wx, wy, factor, 1.0);
            if p.x < -size || p.x > sw + size || p.y < -size || p.y > sh + size {
                continue;
            }
            let tex = &sp.clouds[(hash(i, 124) * 3.0) as usize % 3];
            blit_wh(tex, p, size * 1.6, size * 0.35, 0.0, hash(i, 125) > 0.5, Color::new(0.9, 0.93, 1.0, a));
        }
    }
}

// ---------------------------------------------------------------------------
// Parallax scenery layers
// ---------------------------------------------------------------------------

fn layer(tex: &Texture2D, cam: &Cam, factor: f32, height_px: f32, lift_px: f32, under: Color, fade: Color) {
    layer_smear(tex, cam, factor, height_px, lift_px, under, fade, 0.0);
}

/// `smear` is horizontal motion blur in pixels: the layer is drawn three
/// times, offset, at reduced alpha.
fn layer_smear(tex: &Texture2D, cam: &Cam, factor: f32, height_px: f32, lift_px: f32, under: Color, fade: Color, smear: f32) {
    let light = daylight(cam);
    let haze = height_haze(cam);
    // The painted layers carry their own aerial perspective; only a light
    // cool wash on the farthest ones here.
    let tint = lerp(mul(lerp(Color::new(0.88, 0.91, 1.0, 1.0), WHITE, (factor / 0.5).clamp(0.0, 1.0)), light), mul(HAZE, light), haze);
    let (under, fade) = (lerp(mul(under, light), mul(HAZE, light), haze), lerp(mul(fade, light), mul(HAZE, light), haze));
    // Layers sink out of frame as the camera climbs, the near ones first.
    let factor_y = para_y(factor);
    let sw = screen_width();
    let sh = screen_height();
    let zz = cam.zoom.powf(0.4);
    let h = height_px * zz;
    let w = h * tex.width() / tex.height();
    let horizon = cam.to_screen_parallax(0.0, 0.0, factor_y).y - lift_px * zz;
    let bottom = horizon;
    let top = bottom - h;
    if top > sh {
        return;
    }
    // Tile phase measured in tiles at the screen centre, so it does not
    // depend on the zoom: the layer is drawn at zoom^0.4 while the camera
    // scrolls at full zoom, and scrolling in pixels would slide the
    // mountains sideways every frame the zoom eases (a visible jitter).
    let tiles = cam.x * factor * crate::PPM / (height_px * tex.width() / tex.height());
    let phase = tiles.rem_euclid(1.0) * w;
    let passes: &[(f32, f32)] = if smear > 1.0 { &[(-1.0, 0.35), (1.0, 0.35), (0.0, 0.6)] } else { &[(0.0, 1.0)] };
    for (off, alpha) in passes {
        let mut x = sw * 0.5 - phase + off * smear;
        while x > -w {
            x -= w;
        }
        let c = Color::new(tint.r, tint.g, tint.b, *alpha);
        while x < sw {
            draw_texture_ex(tex, x, top, c, DrawTextureParams { dest_size: Some(vec2(w + 1.0, h)), ..Default::default() });
            x += w;
        }
    }
    if bottom < sh && under.a > 0.0 {
        let strips = 10;
        let hh = (sh - bottom + 1.0) / strips as f32;
        for i in 0..strips {
            let k = i as f32 / (strips - 1) as f32;
            draw_rectangle(0.0, bottom - 1.0 + i as f32 * hh, sw, hh + 1.0, lerp(under, fade, k));
        }
    }
}

pub fn mountains(sp: &Sprites, cam: &Cam) {
    // Three depths of the Rockies: the far range (palest, bluest), the
    // front range with its snowfields and shadow faces, then the forested
    // foothills; each sinks with height at its own rate (see para_y).
    layer(&sp.mountains_far, cam, 0.10, 250.0, 70.0, Color::new(0.84, 0.90, 0.97, 1.0), Color::new(0.80, 0.86, 0.96, 1.0));
    layer(&sp.mountains_near, cam, 0.18, 250.0, 22.0, Color::new(0.81, 0.88, 0.96, 1.0), Color::new(0.78, 0.85, 0.95, 1.0));
    layer(&sp.hills_far, cam, 0.30, 175.0, 0.0, Color::new(0.79, 0.84, 0.93, 1.0), Color::new(0.75, 0.81, 0.92, 1.0));
}

pub fn hills(sp: &Sprites, cam: &Cam, speed: f32) {
    let smear = ((speed - 22.0) / 10.0).clamp(0.0, 4.0) * 2.5;
    layer_smear(&sp.hills_near, cam, 0.45, 260.0, 0.0, Color::new(0.77, 0.82, 0.93, 1.0), Color::new(0.72, 0.78, 0.92, 1.0), smear * 0.6);
    meadow(sp, cam, 0.80);
    layer_smear(&sp.treeline, cam, 0.80, 105.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.0), Color::new(0.0, 0.0, 0.0, 0.0), smear);
    // Lower the treeline's contrast so airborne pickups read against it.
    let sh = screen_height();
    let sw = screen_width();
    let horizon = cam.to_screen_parallax(0.0, 0.0, para_y(0.8)).y;
    let top = horizon - 105.0 * cam.zoom.powf(0.4);
    if top < sh {
        draw_rectangle(0.0, top.max(0.0), sw, (horizon - top.max(0.0)).max(0.0), Color::new(0.40, 0.45, 0.68, 0.28));
    }
}

fn meadow(sp: &Sprites, cam: &Cam, factor: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let zz = cam.zoom.powf(0.4);
    let factor_y = para_y(factor);
    let horizon = cam.to_screen_parallax(0.0, 0.0, factor_y).y - 4.0;
    if horizon > sh {
        return;
    }
    let tw = 260.0 * zz;
    let th = tw * sp.meadow.height() / sp.meadow.width();
    // Phase in tiles at the screen centre, independent of zoom (see layer_smear).
    let phase = (cam.x * factor * crate::PPM / 260.0).rem_euclid(1.0) * tw;
    let light = lerp(daylight(cam), mul(HAZE, daylight(cam)), height_haze(cam));
    let mut x = sw * 0.5 - phase;
    while x > -tw {
        x -= tw;
    }
    while x < sw {
        let mut y = horizon;
        while y < sh {
            draw_texture_ex(&sp.meadow, x, y, light, DrawTextureParams { dest_size: Some(vec2(tw + 1.0, th + 1.0)), ..Default::default() });
            y += th;
        }
        x += tw;
    }
    // Cool shadow gradient towards the horizon and a warm-lit foreground.
    let strips = 8;
    let hh = 90.0 * zz / strips as f32;
    for i in 0..strips {
        let a = 0.35 * (1.0 - i as f32 / strips as f32);
        draw_rectangle(0.0, horizon + i as f32 * hh, sw, hh + 1.0, Color::new(0.55, 0.60, 0.85, a));
    }
}

// ---------------------------------------------------------------------------
// Ground and dressing
// ---------------------------------------------------------------------------

const GROUND_TILE_W: f32 = 8.0;
const GROUND_TILE_LIP: f32 = 44.0 / 256.0;

/// The jet stream: long thin streaks flowing right through the tailwind
/// band, so the river of air is visible before the pilot reaches it.
pub fn jet_streamers(cam: &Cam, t: f32) {
    use hamsters_core::{jet_strength, JET_HI, JET_LO, JET_PUSH};
    let sw = screen_width();
    let s = cam.scale();
    // Skip entirely when the band is off screen.
    let top = cam.to_screen(0.0, JET_HI + 6.0).y;
    let bottom = cam.to_screen(0.0, JET_LO - 6.0).y;
    if bottom < 0.0 || top > screen_height() {
        return;
    }
    let flow = JET_PUSH * 4.0;
    let n = 30;
    for i in 0..n {
        let k = i as f32 / n as f32;
        let y = JET_LO + (JET_HI - JET_LO) * (0.04 + 0.92 * hash(i, 301));
        let len = 6.0 + 10.0 * hash(i, 302);
        let period = 90.0 + 60.0 * hash(i, 303);
        // Each streak loops along a band anchored to the camera, in world
        // space, so they scroll with the flow and parallax with the scene.
        let phase = hash(i, 304) * period;
        let wx = cam.x - sw * 0.5 / s - 20.0 + ((t * flow + phase) % period) * (sw / s + 40.0) / period;
        let a = 0.25 * jet_strength(y) * (0.6 + 0.4 * hash(i, 305));
        let p0 = cam.to_screen(wx, y);
        let p1 = cam.to_screen(wx + len, y);
        let col = Color::new(1.0, 1.0, 1.0, a);
        draw_line(p0.x, p0.y, p1.x, p1.y, (0.08 + 0.06 * k) * s, col);
        // A brighter head so the direction reads.
        draw_circle(p1.x, p1.y, 0.09 * s, Color::new(1.0, 1.0, 1.0, a * 1.5));
    }
}

/// Frozen lakes and snow drifts: the surfaces from the core, drawn over
/// the ground tiles for every visible segment.
fn surfaces(cam: &Cam, left: f32, right: f32) {
    use hamsters_core::{surface_at, Surface, SURFACE_SEGMENT};
    let s = cam.scale();
    let gy = cam.to_screen(0.0, 0.0).y;
    let first = (left / SURFACE_SEGMENT).floor() as i32;
    let last = (right / SURFACE_SEGMENT).ceil() as i32;
    for k in first..=last {
        let x0 = k as f32 * SURFACE_SEGMENT;
        match surface_at(x0 + 1.0) {
            Surface::Snow => {}
            Surface::Ice => {
                // A frozen lake: a blue slab sunk into the snow, an ink edge,
                // a white glint along the top and three cracks.
                let a = cam.to_screen(x0 + 1.5, 0.0).x;
                let b = cam.to_screen(x0 + SURFACE_SEGMENT - 1.5, 0.0).x;
                let top = gy - 0.05 * s;
                let h = 0.85 * s;
                let ice = Color::new(0.62, 0.84, 0.97, 1.0);
                let deep = Color::new(0.42, 0.66, 0.90, 1.0);
                draw_rectangle(a, top, b - a, h, OUTLINE);
                draw_rectangle(a + 0.06 * s, top + 0.06 * s, b - a - 0.12 * s, h - 0.12 * s, ice);
                draw_rectangle(a + 0.06 * s, top + h * 0.55, b - a - 0.12 * s, h * 0.45 - 0.06 * s, deep);
                draw_rectangle(a + 0.3 * s, top + 0.12 * s, b - a - 0.6 * s, 0.07 * s, Color::new(1.0, 1.0, 1.0, 0.9));
                for c in 0..3 {
                    let cx = x0 + 6.0 + hash(k * 3 + c, 311) * (SURFACE_SEGMENT - 12.0);
                    let p = cam.to_screen(cx, 0.0);
                    let dx = (hash(k * 3 + c, 312) - 0.5) * 3.0 * s;
                    let dark = Color::new(0.16, 0.30, 0.55, 0.9);
                    draw_line(p.x - dx, top + 0.1 * s, p.x, top + 0.42 * s, 0.05 * s, dark);
                    draw_line(p.x, top + 0.42 * s, p.x + dx * 0.7, top + 0.7 * s, 0.05 * s, dark);
                    draw_line(p.x, top + 0.42 * s, p.x + dx * 0.4, top + 0.2 * s, 0.04 * s, dark);
                }
                // Snowy banks at both ends so the slab sits in the ground.
                draw_ellipse(a, top + 0.25 * s, 0.9 * s, 0.32 * s, 0.0, SNOW);
                draw_ellipse(b, top + 0.25 * s, 0.9 * s, 0.32 * s, 0.0, SNOW);
            }
            Surface::Drift => {
                // A deep drift: a low cosine hump, white with a blue lee side
                // and an ink outline along the crest, like the hills.
                let n = 28;
                let h = 1.3 * s;
                let w = SURFACE_SEGMENT - 4.0;
                let base = gy + 0.3 * s;
                let crest = |u: f32| gy - 0.15 * s - h * 0.5 * (1.0 - (u * std::f32::consts::TAU).cos());
                for i in 0..n {
                    let u0 = i as f32 / n as f32;
                    let u1 = (i + 1) as f32 / n as f32;
                    let p0 = cam.to_screen(x0 + 2.0 + w * u0, 0.0);
                    let p1 = cam.to_screen(x0 + 2.0 + w * u1, 0.0);
                    let (y0, y1) = (crest(u0), crest(u1));
                    draw_triangle(vec2(p0.x, base), vec2(p0.x, y0), vec2(p1.x, y1), SNOW);
                    draw_triangle(vec2(p0.x, base), vec2(p1.x, y1), vec2(p1.x, base), SNOW);
                    if u0 >= 0.5 {
                        let sh = Color::new(0.62, 0.72, 0.92, 0.85);
                        let (m0, m1) = (base + (y0 - base) * 0.55, base + (y1 - base) * 0.55);
                        draw_triangle(vec2(p0.x, base), vec2(p0.x, m0), vec2(p1.x, m1), sh);
                        draw_triangle(vec2(p0.x, base), vec2(p1.x, m1), vec2(p1.x, base), sh);
                    }
                    draw_line(p0.x, y0, p1.x, y1, 0.09 * s, OUTLINE);
                }
            }
        }
    }
}

pub fn ground(sp: &Sprites, cam: &Cam, font: &Font, towns: &[String], city: Option<&City>) {
    let sw = screen_width();
    let sh = screen_height();
    let s = cam.scale();
    let gy = cam.to_screen(0.0, 0.0).y;
    if gy > sh + 10.0 {
        return;
    }
    let tw = GROUND_TILE_W * s;
    let th = tw * sp.ground.height() / sp.ground.width();
    let top = gy - th * GROUND_TILE_LIP;
    let left_world = cam.x - sw * 0.5 / s;
    let first = (left_world / GROUND_TILE_W).floor();
    let mut wx = first * GROUND_TILE_W;
    // The snow takes the evening light too, a little less than the far layers.
    let light = lerp(WHITE, daylight(cam), 0.7);
    while cam.to_screen(wx, 0.0).x < sw {
        let x = cam.to_screen(wx, 0.0).x;
        draw_texture_ex(&sp.ground, x, top, light, DrawTextureParams { dest_size: Some(vec2(tw + 1.0, th)), ..Default::default() });
        wx += GROUND_TILE_W;
    }
    if top + th < sh {
        // Below the tile the snowfield carries on in its deepest blue.
        draw_rectangle(0.0, top + th - 1.0, sw, sh - (top + th) + 1.0, mul(Color::new(0.64, 0.71, 0.88, 1.0), light));
    }

    let left = left_world - 8.0;
    let right = cam.x + sw * 0.5 / s + 8.0;
    snow_dressing(cam, left, right, city, light);
    if let Some(c) = city {
        c.road(cam);
    }
    surfaces(cam, left, right);
    // City blocks bring their own street furniture; trees, snowmen and
    // fences stay on the open snow.
    let open_snow = |x: f32| city.map_or(true, |c| c.ground_at(x).map_or(true, |g| g == Ground::Snow));
    let shadow = |x: f32, w: f32| {
        draw_ellipse(x, gy + 0.06 * s, w * 0.5, 0.2 * s, 0.0, Color::new(0.2, 0.25, 0.5, 0.25));
    };
    // Big props every ~11 m (skipped around the launch area).
    let first = (left / 11.0).floor() as i32;
    let last = (right / 11.0).ceil() as i32;
    for k in first..=last {
        let r = hash(k, 41);
        if r < 0.30 || (-3..=1).contains(&k) {
            continue;
        }
        let wx = k as f32 * 11.0 + hash(k, 44) * 7.0;
        if !open_snow(wx) {
            continue;
        }
        let scale = 0.8 + hash(k, 42) * 0.7;
        let p = cam.to_screen(wx, 0.0);
        let foot = vec2(p.x, p.y + 0.05 * s);
        if r < 0.5 {
            shadow(p.x, 3.0 * scale * s);
            blit_standing(&sp.xmas_tree, foot, 4.0 * scale * s, hash(k, 43) > 0.5, WHITE);
        } else if r < 0.72 {
            shadow(p.x, 2.4 * scale * s);
            blit_standing(&sp.pine_snow, foot, 3.0 * scale * s, false, WHITE);
        } else if r < 0.86 {
            blit_standing(&sp.snowman, foot, 2.0 * scale * s, hash(k, 43) > 0.5, WHITE);
        } else {
            blit_standing(&sp.lamppost, foot, 1.3 * s, false, WHITE);
        }
    }
    // Fences and gift piles every ~17 m.
    let first = (left / 17.0).floor() as i32;
    let last = (right / 17.0).ceil() as i32;
    for k in first..=last {
        let r = hash(k, 61);
        if r < 0.45 || (-2..=1).contains(&k) {
            continue;
        }
        let wx = k as f32 * 17.0 + hash(k, 62) * 9.0;
        if !open_snow(wx) {
            continue;
        }
        let p = cam.to_screen(wx, 0.0);
        let foot = vec2(p.x, p.y + 0.04 * s);
        if r < 0.75 {
            blit_standing(&sp.fence, foot, 4.2 * s, false, WHITE);
        } else {
            blit_standing(&sp.gift_pile, foot, 2.4 * s, hash(k, 63) > 0.5, WHITE);
        }
    }
    // Ground rhythm: twin sled tracks and a marker post every 10 m so the
    // scroll speed reads even on empty snow.
    let track_a = gy + 0.9 * s;
    let track_b = gy + 1.25 * s;
    let first_d = (left / 2.5).floor() as i32;
    let last_d = (right / 2.5).ceil() as i32;
    for k in first_d..=last_d {
        let x0 = cam.to_screen(k as f32 * 2.5, 0.0).x;
        let x1 = cam.to_screen(k as f32 * 2.5 + 1.6, 0.0).x;
        draw_line(x0, track_a, x1, track_a, 0.12 * s, Color::new(0.62, 0.68, 0.88, 0.95));
        draw_line(x0 + 0.3 * s, track_b, x1 + 0.3 * s, track_b, 0.12 * s, Color::new(0.62, 0.68, 0.88, 0.95));
    }
    let first = (left / 10.0).floor() as i32;
    let last = (right / 10.0).ceil() as i32;
    for k in first..=last {
        if k <= 1 {
            continue;
        }
        let p = cam.to_screen(k as f32 * 10.0, 0.0);
        draw_rectangle(p.x - 0.09 * s, p.y - 0.9 * s, 0.18 * s, 0.9 * s, Color::new(0.42, 0.25, 0.12, 1.0));
        draw_rectangle(p.x - 0.16 * s, p.y - 1.0 * s, 0.32 * s, 0.22 * s, if k % 5 == 0 { Color::new(0.9, 0.25, 0.25, 1.0) } else { SNOW });
        // Drift shadow behind each post.
        draw_ellipse(p.x + 0.5 * s, p.y + 0.4 * s, 0.9 * s, 0.16 * s, 0.0, Color::new(0.55, 0.62, 0.85, 0.35));
    }
    // Small stuff every ~2.5 m.
    let first = (left / 1.6).floor() as i32;
    let last = (right / 1.6).ceil() as i32;
    for k in first..=last {
        let r = hash(k, 51);
        let wx = k as f32 * 1.6 + hash(k, 52) * 1.2;
        let p = cam.to_screen(wx, 0.0);
        let foot = vec2(p.x, p.y + 0.05 * s);
        if r < 0.10 {
            blit_standing(&sp.candycane, foot, 0.5 * s, hash(k, 54) > 0.5, WHITE);
        } else if r < 0.45 {
            blit_standing(&sp.tuft, foot, (0.9 + hash(k, 53) * 0.6) * s, hash(k, 54) > 0.5, WHITE);
        }
    }
    // No distance posts: the HUD carries the number and the pennants mark history.
    let _ = (font, towns);
}

/// Big wind-sculpted drifts and twinkling glints on the open snow, placed in
/// world space so the 8 m ground tile never reads as a repeat.
fn snow_dressing(cam: &Cam, left: f32, right: f32, city: Option<&City>, light: Color) {
    let s = cam.scale();
    let gy = cam.to_screen(0.0, 0.0).y;
    let open = |x: f32| city.map_or(true, |c| c.ground_at(x).map_or(true, |g| g == Ground::Snow));
    let white = mul(Color::new(1.0, 1.0, 1.0, 1.0), light);
    let body = mul(Color::new(0.95, 0.97, 1.0, 1.0), light);
    let lee = mul(Color::new(0.70, 0.78, 0.94, 1.0), light);
    let deep = mul(Color::new(0.55, 0.64, 0.88, 0.45), light);
    let first = (left / 13.0).floor() as i32;
    let last = (right / 13.0).ceil() as i32;
    for k in first..=last {
        if hash(k, 431) < 0.35 || (-4..=0).contains(&k) {
            continue;
        }
        let w = 4.0 + 6.0 * hash(k, 432);
        let x0 = k as f32 * 13.0 + hash(k, 433) * (13.0 - w);
        if !open(x0) || !open(x0 + w) {
            continue;
        }
        let h = (0.35 + 0.6 * hash(k, 434)) * s;
        let n = 20;
        let crest_u = 0.62 + 0.15 * hash(k, 435);
        // Long windward slope to the crest, short steep lee face.
        let prof = |u: f32| -> f32 {
            if u < crest_u {
                let v = u / crest_u;
                (v * std::f32::consts::FRAC_PI_2).sin().powf(1.6)
            } else {
                let v = (u - crest_u) / (1.0 - crest_u);
                (1.0 - v * v).max(0.0).powf(1.3)
            }
        };
        let base = gy + 0.12 * s;
        let a0 = cam.to_screen(x0, 0.0).x;
        let a1 = cam.to_screen(x0 + w, 0.0).x;
        draw_ellipse((a0 + a1) * 0.5 + 0.4 * s, base + 0.12 * s, (a1 - a0) * 0.55, 0.22 * s, 0.0, deep);
        for i in 0..n {
            let u0 = i as f32 / n as f32;
            let u1 = (i + 1) as f32 / n as f32;
            let (x_a, x_b) = (a0 + (a1 - a0) * u0, a0 + (a1 - a0) * u1);
            let (y_a, y_b) = (base - h * prof(u0), base - h * prof(u1));
            let c = if u0 >= crest_u { lee } else { body };
            draw_triangle(vec2(x_a, base), vec2(x_a, y_a), vec2(x_b, y_b), c);
            draw_triangle(vec2(x_a, base), vec2(x_b, y_b), vec2(x_b, base), c);
            if u0 < crest_u && u0 > 0.15 {
                draw_line(x_a, y_a + 0.04 * s, x_b, y_b + 0.04 * s, 0.09 * s, white);
            }
        }
        // A blue undercut below the crest on the lee side.
        let cx = a0 + (a1 - a0) * crest_u;
        draw_triangle(vec2(cx, base - h * 0.96), vec2(cx + (a1 - a0) * 0.12, base - h * 0.55), vec2(cx - 0.05 * s, base - h * 0.5), mul(Color::new(0.62, 0.71, 0.92, 0.8), light));
    }
    // Glints: the snow flashes as the light catches it.
    let t = get_time() as f32;
    let first = (left / 0.9).floor() as i32;
    let last = (right / 0.9).ceil() as i32;
    for k in first..=last {
        let wx = k as f32 * 0.9 + hash(k, 441) * 0.9;
        if !open(wx) {
            continue;
        }
        let tw = (t * (1.2 + hash(k, 442) * 1.5) + hash(k, 443) * 40.0).sin().max(0.0).powf(10.0);
        if tw < 0.05 {
            continue;
        }
        let p = cam.to_screen(wx, 0.0);
        let y = p.y + (0.05 + hash(k, 444) * 2.6) * s;
        let r = (0.12 + 0.18 * hash(k, 445)) * s * tw;
        let c = Color::new(1.0, 1.0, 1.0, 0.95 * tw);
        draw_line(p.x - r, y, p.x + r, y, 0.035 * s, c);
        draw_line(p.x, y - r * 1.4, p.x, y + r * 1.4, 0.035 * s, c);
        draw_circle(p.x, y, 0.045 * s, c);
    }
}

/// The Falkbuilt factory with the ski jump on its roof, Santa and cheering elves.
/// How a cheering bystander moves at time `t`: lift off the ground (as a
/// fraction of its height), squash/stretch and lean. Between jumps it shifts
/// its weight and breathes; every so often (own rhythm per `seed`) it
/// crouches, springs up stretched on a real arc and lands with a squash.
/// `excite` 0..1 scales the jumping; at 0 it only idles.
pub struct Cheer {
    pub lift: f32,
    pub sx: f32,
    pub sy: f32,
    pub lean: f32,
}

pub fn cheer(t: f32, seed: f32, excite: f32) -> Cheer {
    let breathe = 0.015 * (t * 2.6 + seed * 3.1).sin();
    let mut c = Cheer { lift: 0.0, sx: 1.0 - breathe * 0.5, sy: 1.0 + breathe, lean: 0.045 * (t * 1.7 + seed * 5.3).sin() };
    if excite <= 0.0 {
        return c;
    }
    // A jump every 1.4..2.6 s, offset per figure.
    let period = 1.4 + 1.2 * (seed * 0.618).fract();
    let u = ((t + seed * 7.3) / period).fract() * period;
    let (crouch, air, land) = (0.13, 0.42, 0.15);
    let e = excite.clamp(0.0, 1.0);
    if u < crouch {
        let k = smoothstep(0.0, 1.0, u / crouch);
        c.sy -= 0.15 * k * e;
        c.sx += 0.09 * k * e;
    } else if u < crouch + air {
        let k = (u - crouch) / air;
        c.lift = 4.0 * k * (1.0 - k) * 0.3 * e;
        // Stretched leaving the ground and reaching for it, round at the top.
        let st = 0.09 * (2.0 * k - 1.0).abs() * e;
        c.sy += st;
        c.sx -= st * 0.6;
        c.lean *= 0.3;
    } else if u < crouch + air + land {
        let k = 1.0 - (u - crouch - air) / land;
        let k = k * k;
        c.sy -= 0.17 * k * e;
        c.sx += 0.11 * k * e;
    }
    c
}

/// A bystander cheering in place (see [`cheer`]), feet at `foot`.
pub fn cheering(tex: &Texture2D, foot: Vec2, w: f32, flip: bool, t: f32, seed: f32, excite: f32, tint: Color) {
    let c = cheer(t, seed, excite);
    let h = w * tex.height() / tex.width();
    blit_foot(tex, vec2(foot.x, foot.y - c.lift * h), w, c.sx, c.sy, c.lean, flip, tint);
}

/// A square steel plate with four bolt heads, centred on a timber joint.
fn bolt_plate(cam: &Cam, x: f32, y: f32) {
    let s = cam.scale();
    let p = cam.to_screen(x, y);
    let h = 0.24 * s;
    draw_rectangle(p.x - h, p.y - h, 2.0 * h, 2.0 * h, Color::new(0.27, 0.33, 0.40, 1.0));
    draw_rectangle(p.x - h, p.y - h, 2.0 * h, 0.05 * s, Color::new(0.60, 0.68, 0.76, 1.0));
    for (dx, dy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        draw_circle(p.x + dx * h * 0.55, p.y + dy * h * 0.55, 0.045 * s, Color::new(0.80, 0.86, 0.92, 1.0));
    }
}

/// A ridge of snow lying along the top of a timber from (x0,y0) to (x1,y1).
fn timber_snow(cam: &Cam, x0: f32, y0: f32, x1: f32, y1: f32, th: f32) {
    let s = cam.scale();
    let a = cam.to_screen(x0, y0);
    let b = cam.to_screen(x1, y1);
    draw_line(a.x, a.y + 0.02 * s, b.x, b.y + 0.02 * s, th * 1.2 * s, Color::new(0.62, 0.72, 0.88, 1.0));
    draw_line(a.x, a.y - 0.02 * s, b.x, b.y - 0.02 * s, th * s, SNOW);
    draw_circle(a.x, a.y - 0.02 * s, th * 0.5 * s, SNOW);
    draw_circle(b.x, b.y - 0.02 * s, th * 0.5 * s, SNOW);
}

pub fn start_area(sp: &Sprites, cam: &Cam, font: &Font, label: &str) {
    let s = cam.scale();
    let sw = screen_width();
    let ground_y = cam.to_screen(0.0, 0.0).y;
    let visible = |wx: f32, half_w: f32| {
        let x = cam.to_screen(wx, 0.0).x;
        x > -half_w * s && x < sw + half_w * s
    };
    if !visible(-14.0, 48.0) {
        return;
    }
    let t = get_time() as f32;

    // The Falkbuilt building: low and wide, its flat roof (row 120 of the
    // 480 px sprite) at ROOF_Y carries the ski jump; the inrun tower stands
    // at the back over the letters and the take-off leaves the front corner.
    let tower_h = TOWER_W * sp.tower.height() / sp.tower.width();
    let platform_frac = 120.0 / 480.0;
    let tower_x = -21.0;
    let tower_top_y = ROOF_Y + platform_frac * tower_h;
    let tp = cam.to_screen(tower_x, tower_top_y);
    draw_ellipse(tp.x, ground_y + 0.08 * s, TOWER_W * 0.55 * s, 0.3 * s, 0.0, Color::new(0.2, 0.25, 0.5, 0.3));
    draw_texture_ex(
        &sp.tower,
        tp.x - TOWER_W * s / 2.0,
        tp.y,
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(TOWER_W * s, tower_h * s)), ..Default::default() },
    );
    // Sign text on the facade, for packs whose building has a blank board;
    // an empty label means the sprite carries its own lettering.
    if !label.is_empty() {
        let sign = cam.to_screen(tower_x, tower_top_y - (160.0 / 480.0) * tower_h);
        let board_w = TOWER_W * s * 0.5;
        let base = measure_text(label, Some(font), 20, 1.0);
        let scale = (board_w / base.width).clamp(0.2, 1.4);
        let d = measure_text(label, Some(font), 20, scale);
        draw_text_ex(label, sign.x - d.width / 2.0, sign.y + d.height / 2.0, TextParams { font: Some(font), font_size: 20, font_scale: scale, color: OUTLINE, ..Default::default() });
    }

    // The ski jump on the roof: a wooden trestle tower at the back with the
    // start gate on top, a straight inrun on a truss, the curved transition
    // dipping to the roof and the take-off table cantilevered off the front
    // corner. Snow on the deck, two tracks, a candy-cane rail.
    let mut pts: Vec<Vec2> = (0..=48).map(|i| { let p = ramp_point(i as f32 / 48.0); vec2(p.x, p.y) }).collect();
    // Extend the table a little past the lip along its tangent.
    let (a, b) = (ramp_point(0.98), ramp_point(1.0));
    let d = vec2(b.x - a.x, b.y - a.y).normalize_or_zero();
    pts.push(vec2(LIP.x, LIP.y) + d * 1.2);
    let deck_y = |x: f32| -> f32 {
        // Height of the deck above a world x by scanning the polyline.
        for w in pts.windows(2) {
            if (w[0].x <= x && x <= w[1].x) || (w[1].x <= x && x <= w[0].x) {
                let k = ((x - w[0].x) / (w[1].x - w[0].x)).clamp(0.0, 1.0);
                return w[0].y + (w[1].y - w[0].y) * k;
            }
        }
        RAMP_TOP.y
    };
    let post = |x: f32, top: f32, bottom: f32, w: f32| {
        let a = cam.to_screen(x, top);
        let b = cam.to_screen(x, bottom);
        draw_line(a.x, a.y, b.x, b.y, (w + 0.12) * s, WOOD_EDGE);
        draw_line(a.x, a.y, b.x, b.y, w * s, WOOD);
        draw_line(a.x + 0.25 * w * s, a.y, b.x + 0.25 * w * s, b.y, 0.4 * w * s, WOOD_SHADE);
        draw_line(a.x - 0.22 * w * s, a.y, b.x - 0.22 * w * s, b.y, 0.22 * w * s, WOOD_LIGHT);
    };
    let beam = |x0: f32, y0: f32, x1: f32, y1: f32, w: f32| {
        let a = cam.to_screen(x0, y0);
        let b = cam.to_screen(x1, y1);
        draw_line(a.x, a.y, b.x, b.y, (w + 0.1) * s, WOOD_EDGE);
        draw_line(a.x, a.y, b.x, b.y, w * s, WOOD_SHADE);
        draw_line(a.x, a.y - 0.2 * w * s, b.x, b.y - 0.2 * w * s, 0.45 * w * s, WOOD);
    };
    // Start tower: two legs, rungs and X bracing, standing on the roof.
    let (tl, tr) = (RAMP_TOP.x - 1.9, RAMP_TOP.y - 0.3);
    let top_y = RAMP_TOP.y - 0.35;
    let mut y = ROOF_Y + 1.6;
    while y < top_y - 0.8 {
        beam(tl, y, tl + 3.6, y, 0.22);
        beam(tl, y, tl + 3.6, (y + 2.2).min(top_y), 0.16);
        beam(tl + 3.6, y, tl, (y + 2.2).min(top_y), 0.16);
        y += 2.2;
    }
    post(tl, top_y, ROOF_Y - 0.3, 0.5);
    post(tl + 3.6, top_y, ROOF_Y - 0.3, 0.5);
    let _ = tr;
    // Steel bolt plates where the rungs meet the legs, and snow lying along
    // the top of every rung.
    let mut y = ROOF_Y + 1.6;
    while y < top_y - 0.8 {
        timber_snow(cam, tl + 0.25, y + 0.11, tl + 3.35, y + 0.11, 0.13);
        bolt_plate(cam, tl, y);
        bolt_plate(cam, tl + 3.6, y);
        y += 2.2;
    }
    // The inrun's shadow on the roof snow.
    let r0 = cam.to_screen(RAMP_TOP.x - 2.5, ROOF_Y + 0.12);
    let r1 = cam.to_screen(LIP.x - 0.5, ROOF_Y + 0.12);
    draw_rectangle(r0.x, r0.y - 0.05 * s, r1.x - r0.x, 0.35 * s, Color::new(0.45, 0.55, 0.85, 0.22));
    // Struts from the deck to the roof; the long ones get a knee brace.
    for x in [-32.5, -27.0, -19.0, -13.5, -8.0, -3.2] {
        let dy = deck_y(x);
        if dy - ROOF_Y > 0.9 {
            post(x, dy - 0.3, ROOF_Y - 0.3, 0.34);
            if dy - ROOF_Y > 4.0 {
                beam(x, ROOF_Y + 2.4, x - 2.2, dy - 0.9, 0.16);
                beam(x, ROOF_Y + 2.4, x + 2.2, dy - 0.9 - 1.1, 0.16);
                bolt_plate(cam, x, ROOF_Y + 2.4);
            }
            // Steel shoe on the roof and a bolted bracket under the deck.
            let f = cam.to_screen(x, ROOF_Y - 0.1);
            draw_rectangle(f.x - 0.32 * s, f.y - 0.3 * s, 0.64 * s, 0.3 * s, Color::new(0.24, 0.30, 0.36, 1.0));
            draw_rectangle(f.x - 0.28 * s, f.y - 0.28 * s, 0.56 * s, 0.08 * s, Color::new(0.55, 0.63, 0.70, 1.0));
            bolt_plate(cam, x, dy - 0.75);
        }
    }
    // Under-deck truss along the inrun: a lower chord and zigzag diagonals.
    let depth = |p: Vec2| (1.0f32).min((p.y - ROOF_Y - 0.5).max(0.0));
    for (i, w) in pts.windows(2).enumerate() {
        let (d0, d1) = (depth(w[0]), depth(w[1]));
        if d0 < 0.2 { continue; }
        beam(w[0].x, w[0].y - d0, w[1].x, w[1].y - d1, 0.2);
        if i % 2 == 0 { beam(w[0].x, w[0].y - d0, w[1].x, w[1].y - 0.05, 0.12); } else { beam(w[0].x, w[0].y - 0.05, w[1].x, w[1].y - d1, 0.12); }
    }
    // The deck: one long orange timber like the key art's ramp, outlined
    // in deep orange, shaded underneath, lit along the top, with wood grain;
    // then a snow blanket with a blue underside and two sled tracks.
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y);
        let b = cam.to_screen(w[1].x, w[1].y);
        draw_line(a.x, a.y, b.x, b.y, 0.92 * s, WOOD_EDGE);
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y);
        let b = cam.to_screen(w[1].x, w[1].y);
        draw_line(a.x, a.y, b.x, b.y, 0.8 * s, WOOD_SHADE);
        draw_line(a.x, a.y - 0.08 * s, b.x, b.y - 0.08 * s, 0.6 * s, WOOD);
        draw_line(a.x, a.y - 0.3 * s, b.x, b.y - 0.3 * s, 0.12 * s, WOOD_LIGHT);
    }
    for (i, w) in pts.windows(2).enumerate() {
        // Grain: short darker streaks staggered along the timber.
        if i % 3 == 0 {
            let a = cam.to_screen(w[0].x, w[0].y);
            let b = cam.to_screen(w[1].x, w[1].y);
            let off = if i % 2 == 0 { 0.08 } else { -0.12 } * s;
            draw_line(a.x, a.y + off, a.x + (b.x - a.x) * 0.7, a.y + (b.y - a.y) * 0.7 + off, 0.04 * s, WOOD_SHADE);
        }
    }
    // Plank butt joints across the side timber, each with a pair of bolts,
    // so the deck reads as built from lengths of lumber.
    let mut along = 0.0;
    let mut next = 1.4;
    for w in pts.windows(2) {
        let seg = vec2(w[1].x - w[0].x, w[1].y - w[0].y);
        let len = seg.length();
        while next < along + len {
            let k = (next - along) / len;
            let p = vec2(w[0].x, w[0].y) + seg * k;
            let n = vec2(-seg.y, seg.x) / len;
            let top = cam.to_screen(p.x + n.x * 0.36, p.y + n.y * 0.36);
            let bot = cam.to_screen(p.x - n.x * 0.36, p.y - n.y * 0.36);
            draw_line(top.x, top.y, bot.x, bot.y, 0.06 * s, WOOD_EDGE);
            draw_line(top.x + 0.06 * s, top.y, bot.x + 0.06 * s, bot.y, 0.03 * s, WOOD_LIGHT);
            for side in [-0.32f32, 0.32] {
                let b = cam.to_screen(p.x + seg.x / len * side, p.y + seg.y / len * side - 0.05);
                draw_circle(b.x, b.y, 0.075 * s, Color::new(0.30, 0.20, 0.14, 1.0));
                draw_circle(b.x - 0.02 * s, b.y - 0.02 * s, 0.03 * s, Color::new(0.85, 0.80, 0.72, 1.0));
            }
            next += 2.2;
        }
        along += len;
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y + 0.3);
        let b = cam.to_screen(w[1].x, w[1].y + 0.3);
        draw_line(a.x, a.y + 0.04 * s, b.x, b.y + 0.04 * s, 0.4 * s, Color::new(0.56, 0.69, 0.82, 1.0));
        draw_line(a.x, a.y, b.x, b.y, 0.34 * s, SNOW_SHADE);
        draw_line(a.x, a.y - 0.06 * s, b.x, b.y - 0.06 * s, 0.22 * s, SNOW);
        // Two sled tracks pressed into the snow.
        for off in [-0.06, 0.06] {
            draw_line(a.x, a.y + off * s, b.x, b.y + off * s, 0.03 * s, Color::new(0.62, 0.72, 0.9, 1.0));
        }
    }
    // Candy-cane rail along the whole run, on short posts.
    for (i, w) in pts.windows(2).enumerate() {
        let a = cam.to_screen(w[0].x, w[0].y + 1.1);
        let b = cam.to_screen(w[1].x, w[1].y + 1.1);
        let c = if i % 2 == 0 { Color::new(0.92, 0.32, 0.32, 1.0) } else { Color::new(1.0, 0.97, 0.95, 1.0) };
        draw_line(a.x, a.y, b.x, b.y, 0.1 * s, c);
        if i % 4 == 0 {
            let f = cam.to_screen(w[0].x, w[0].y + 0.3);
            draw_line(f.x, f.y, a.x, a.y, 0.06 * s, WOOD_EDGE);
        }
    }
    // Floodlight masts over the take-off, clear of the letters: timber
    // poles on the roof with lamp heads that glow warm (brighter as the
    // afternoon fades).
    let dusk = dusk_factor(cam);
    for (i, x) in [-11.2f32, -5.6].iter().enumerate() {
        let x = *x;
        let top = deck_y(x) + 4.2;
        let b = cam.to_screen(x, top);
        // A timber mast like the trestle, with a steel cross arm.
        post(x, top, ROOF_Y - 0.2, 0.26);
        bolt_plate(cam, x, (ROOF_Y + top) * 0.5);
        draw_line(b.x - 0.8 * s, b.y, b.x + 0.8 * s, b.y, 0.18 * s, Color::new(0.24, 0.30, 0.36, 1.0));
        draw_line(b.x - 0.8 * s, b.y - 0.06 * s, b.x + 0.8 * s, b.y - 0.06 * s, 0.05 * s, Color::new(0.62, 0.70, 0.78, 1.0));
        timber_snow(cam, x - 0.85, top + 0.12, x + 0.85, top + 0.12, 0.12);
        // Two lamp housings angled down the run, lenses lit, a warm bloom.
        for dx in [-0.5f32, 0.5] {
            let l = vec2(b.x + dx * s, b.y + 0.3 * s);
            let on = 0.5 + 0.35 * dusk + 0.08 * (t * 2.0 + i as f32).sin();
            glow(l + vec2(0.2 * s, 0.3 * s), 2.6 * s, Color::new(1.0, 0.88, 0.58, 0.5 * on));
            let hw = 0.32 * s;
            draw_triangle(vec2(l.x - hw, l.y - 0.22 * s), vec2(l.x + hw, l.y - 0.22 * s), vec2(l.x + hw * 1.25, l.y + 0.22 * s), Color::new(0.20, 0.26, 0.32, 1.0));
            draw_triangle(vec2(l.x - hw, l.y - 0.22 * s), vec2(l.x + hw * 1.25, l.y + 0.22 * s), vec2(l.x - hw * 0.75, l.y + 0.22 * s), Color::new(0.20, 0.26, 0.32, 1.0));
            draw_line(l.x - hw * 0.7, l.y + 0.2 * s, l.x + hw * 1.2, l.y + 0.2 * s, 0.14 * s, Color::new(1.0, 0.97, 0.82, 1.0));
        }
    }
    // Start platform behind the gate, with a rail, a red start beam and a
    // pennant on a pole; Santa waves from here.
    let (px0, px1, py) = (RAMP_TOP.x - 4.2, RAMP_TOP.x + 0.4, RAMP_TOP.y);
    beam(px0, py, px1, py, 0.62);
    let a = cam.to_screen(px0, py + 0.3);
    let b = cam.to_screen(px1, py + 0.3);
    draw_line(a.x, a.y, b.x, b.y, 0.3 * s, SNOW);
    post(px0 + 0.2, py + 1.5, py, 0.12);
    beam(px0 + 0.2, py + 1.5, px0 + 2.6, py + 1.5, 0.1);
    post(px0 + 0.2, py + 4.4, py + 1.5, 0.1);
    let fp = cam.to_screen(px0 + 0.2, py + 4.4);
    let wave = (t * 3.0).sin() * 0.12 * s;
    draw_triangle(fp, vec2(fp.x + 1.4 * s, fp.y + 0.35 * s + wave), vec2(fp.x, fp.y + 0.7 * s), Color::new(0.92, 0.32, 0.32, 1.0));
    // Start beam: a red bar across the track the sled sits behind.
    let g0 = cam.to_screen(RAMP_TOP.x + 0.9, RAMP_TOP.y + 0.15);
    draw_line(g0.x, g0.y, g0.x, g0.y - 0.9 * s, 0.1 * s, OUTLINE);
    draw_circle(g0.x, g0.y - 0.9 * s, 0.16 * s, Color::new(0.92, 0.32, 0.32, 1.0));

    // Santa waving from the start platform behind the sled.
    let sp_pos = cam.to_screen(RAMP_TOP.x - 2.4, RAMP_TOP.y + 0.05);
    crate::anim::santa_wave(&sp.santa, sp_pos, 2.4 * s, t, cam.cheer);
    // Elves cheering near the landing zone.
    for (i, (wx, tex)) in [(3.0, &sp.elf_a), (5.5, &sp.elf_b), (14.5, &sp.elf_b)].iter().enumerate() {
        let p = cam.to_screen(*wx, 0.0);
        cheering(tex, vec2(p.x, p.y + 0.05 * s), 1.5 * s, false, t, 0.37 + i as f32 * 1.91, 0.5 + 0.5 * cam.cheer, WHITE);
    }
    // A decorated tree, gifts and a lamppost on the ground past the corner.
    let p = cam.to_screen(10.0, 0.0);
    blit_standing(&sp.xmas_tree, vec2(p.x, p.y + 0.05 * s), 4.6 * s, false, WHITE);
    let p = cam.to_screen(7.6, 0.0);
    blit_standing(&sp.gift_pile, vec2(p.x, p.y + 0.05 * s), 2.6 * s, false, WHITE);
    let p = cam.to_screen(12.5, 0.0);
    blit_standing(&sp.lamppost, vec2(p.x, p.y + 0.05 * s), 1.3 * s, false, WHITE);
}

/// Mid-ground dressing on the plain between the hills and the village:
/// drifts, snowmen and small pines at parallax 0.65, so the band under the
/// flight line scrolls and reads as terrain.
pub fn midground(sp: &Sprites, cam: &Cam) {
    let sw = screen_width();
    let sh = screen_height();
    let s = cam.scale();
    let factor = 0.65;
    let factor_y = para_y(factor);
    let base = cam.to_screen_parallax(0.0, 0.0, factor_y).y + 40.0 * cam.zoom.powf(0.4);
    if base < -60.0 || base > sh + 60.0 {
        return;
    }
    let tint = Color::new(0.80, 0.84, 0.98, 1.0);
    let spacing = 4.0;
    let left = cam.x * factor - sw * 0.5 / s - 10.0;
    let right = cam.x * factor + sw * 0.5 / s + 10.0;
    let first = (left / spacing).floor() as i32;
    let last = (right / spacing).ceil() as i32;
    for k in first..=last {
        let r = hash(k, 161);
        if r < 0.4 {
            continue;
        }
        let wx = k as f32 * spacing + hash(k, 162) * 4.0;
        let x = sw * 0.5 + (wx - cam.x * factor) * s;
        let foot = vec2(x, base);
        let sc = cam.zoom.powf(0.4);
        if r < 0.6 {
            blit_standing(&sp.tuft, foot, (1.6 + hash(k, 163)) * 22.0 * sc, hash(k, 164) > 0.5, tint);
        } else if r < 0.75 {
            blit_standing(&sp.pine_snow, foot, (1.2 + hash(k, 163) * 0.6) * 30.0 * sc, false, tint);
        } else if r < 0.9 {
            blit_standing(&sp.fence, foot, 70.0 * sc, false, tint);
        } else {
            blit_standing(&sp.snowman, foot, 34.0 * sc, hash(k, 164) > 0.5, tint);
        }
    }
}

/// Elves and snowmen along the course that hop and wave as the dog passes.
pub fn spectators(sp: &Sprites, cam: &Cam, dog_x: f32, t: f32) {
    let s = cam.scale();
    let sw = screen_width();
    let ground_y = cam.to_screen(0.0, 0.0).y;
    if ground_y > screen_height() + s {
        return;
    }
    let left = cam.x - sw * 0.5 / s - 6.0;
    let right = cam.x + sw * 0.5 / s + 6.0;
    let first = (left / 18.0).floor().max(1.0) as i32;
    let last = (right / 18.0).ceil() as i32;
    for k in first..=last {
        let wx = k as f32 * 18.0 + 8.0 + hash(k, 141) * 6.0;
        let near = ((10.0 - (dog_x - wx).abs()) / 10.0).clamp(0.0, 1.0);
        let p = cam.to_screen(wx, 0.0);
        let tex = if hash(k, 142) > 0.5 { &sp.elf_a } else { &sp.elf_b };
        cheering(tex, vec2(p.x, p.y + 0.05 * s), 1.5 * s, hash(k, 143) > 0.5, t, hash(k, 145) * 10.0, near, WHITE);
        if hash(k, 144) > 0.6 {
            let p2 = cam.to_screen(wx + 2.2, 0.0);
            blit_standing(&sp.snowman, vec2(p2.x, p2.y + 0.05 * s), 1.9 * s, false, WHITE);
        }
    }
}

/// Near foreground: snowy pines and lampposts scrolling faster than the world,
/// sheared into streaks with speed so velocity is felt in the frame.
pub fn foreground(sp: &Sprites, cam: &Cam, speed: f32) {
    let s = cam.scale();
    let sw = screen_width();
    let sh = screen_height();
    let factor = 1.45;
    let ground_y = cam.to_screen_parallax_xy(0.0, -1.5, factor, 1.0).y;
    if ground_y < -2.0 * s {
        return;
    }
    let shear = ((speed - 22.0) / 30.0).clamp(0.0, 1.0);
    let spacing = 9.0;
    let left = cam.x * factor - sw * 0.5 / s - 12.0;
    let right = cam.x * factor + sw * 0.5 / s + 12.0;
    let first = (left / spacing).floor() as i32;
    let last = (right / spacing).ceil() as i32;
    for k in first..=last {
        let r = hash(k, 151);
        if r < 0.45 {
            continue;
        }
        let wx = k as f32 * spacing + hash(k, 152) * 5.0;
        // Skip the launch area so the ramp stays clear.
        if wx > -30.0 * factor && wx < 12.0 * factor {
            continue;
        }
        let p = cam.to_screen_parallax_xy(wx, -1.5, factor, 1.0);
        let tint = Color::new(0.55, 0.6, 0.8, 0.75);
        if r < 0.8 {
            let w = (2.0 + hash(k, 153) * 1.2) * s;
            let h = w * sp.pine_snow.height() / sp.pine_snow.width();
            blit_wh(&sp.pine_snow, vec2(p.x, p.y - h / 2.0), w * (1.0 + shear * 0.9), h, 0.0, false, tint);
        } else {
            let w = 1.1 * s;
            let h = w * sp.lamppost.height() / sp.lamppost.width();
            // Lamps come on as the afternoon fades: a warm disc around the head.
            let dusk = dusk_factor(cam);
            if dusk > 0.0 {
                draw_circle(p.x, p.y - h * 0.86, 2.5 * s, Color::new(1.0, 0.85, 0.5, 0.35 * dusk));
            }
            blit_wh(&sp.lamppost, vec2(p.x, p.y - h / 2.0), w * (1.0 + shear * 1.2), h, 0.0, false, tint);
        }
    }
    let _ = sh;
}

/// Landing-zone marker: a red flag and a snow ring where a sweet-spot push lands.
pub fn landing_zone(cam: &Cam, x: f32, t: f32) {
    let s = cam.scale();
    let p = cam.to_screen(x, 0.0);
    if p.x < -4.0 * s || p.x > screen_width() + 4.0 * s || p.y > screen_height() + s {
        return;
    }
    draw_ellipse(p.x, p.y + 0.05 * s, 3.2 * s, 0.6 * s, 0.0, Color::new(0.9, 0.25, 0.25, 0.18 + 0.08 * (t * 3.0).sin()));
    draw_ellipse(p.x, p.y + 0.05 * s, 2.2 * s, 0.4 * s, 0.0, Color::new(1.0, 1.0, 1.0, 0.35));
    for side in [-1.0f32, 1.0] {
        let fx = p.x + side * 2.6 * s;
        draw_rectangle(fx - 0.05 * s, p.y - 2.2 * s, 0.1 * s, 2.2 * s, Color::new(0.35, 0.38, 0.55, 1.0));
        let wave = (t * 4.0 + side).sin() * 0.15 * s;
        draw_triangle(vec2(fx, p.y - 2.2 * s), vec2(fx + side * 1.1 * s, p.y - 1.85 * s + wave), vec2(fx, p.y - 1.5 * s), Color::new(0.9, 0.25, 0.25, 1.0));
    }
}

/// A ranked pennant where a past flight ended: a pole, a triangular flag
/// with the rank on it. The best flight's pennant is gold, taller, and
/// waves slowly; the rest are ink-and-snow.
pub fn pennant(cam: &Cam, font: &Font, x: f32, rank: usize, gold: bool, t: f32) {
    let s = cam.scale();
    let p = cam.to_screen(x, 0.0);
    if p.x < -4.0 * s || p.x > screen_width() + 4.0 * s || p.y > screen_height() + s {
        return;
    }
    let h = if gold { 3.5 } else { 2.2 } * s;
    draw_rectangle(p.x - 0.06 * s, p.y - h, 0.12 * s, h, OUTLINE);
    let wave = (t * if gold { 2.0 } else { 4.0 } + rank as f32).sin() * 0.12 * s;
    let (fw, fh) = if gold { (1.6 * s, 0.95 * s) } else { (1.2 * s, 0.7 * s) };
    let (x0, y0) = (p.x + 0.06 * s, p.y - h);
    let fill = if gold { SEED } else { SNOW };
    draw_triangle(vec2(x0, y0), vec2(x0 + fw, y0 + fh * 0.5 + wave), vec2(x0, y0 + fh), OUTLINE);
    let k = 0.08 * s;
    draw_triangle(vec2(x0, y0 + k), vec2(x0 + fw - 2.0 * k, y0 + fh * 0.5 + wave), vec2(x0, y0 + fh - k), fill);
    let label = format!("{}", rank);
    let size = if gold { 0.5 } else { 0.4 } * s;
    let d = measure_text(&label, Some(font), size as u16, 1.0);
    let (tx, ty) = (x0 + fw * 0.32 - d.width / 2.0, y0 + fh * 0.5 + wave * 0.4 + d.height / 2.0);
    draw_text_ex(&label, tx, ty, TextParams { font: Some(font), font_size: size as u16, color: OUTLINE, ..Default::default() });
}

/// The aim mark: only a ring on the snow where the flight for the current
/// power and jump would land. The dotted arc off the lip is gone; it read
/// as clutter over the sky. Gold when the push is in the sweet spot.
pub fn aim_arc(cam: &Cam, path: &[Vec2], sweet: bool) {
    let s = cam.scale();
    let sw = screen_width();
    if let Some(last) = path.last() {
        let p = cam.to_screen(last.x, 0.0);
        if p.x > -2.0 * s && p.x < sw + 2.0 * s {
            let col = if sweet { Color::new(1.0, 0.86, 0.30, 0.9) } else { Color::new(0.98, 0.98, 1.0, 0.8) };
            draw_ellipse_lines(p.x, p.y, 0.5 * s, 0.22 * s, 0.0, 0.08 * s, Color::new(0.12, 0.14, 0.25, 0.7));
            draw_ellipse_lines(p.x, p.y, 0.45 * s, 0.19 * s, 0.0, 0.05 * s, col);
        }
    }
}

/// The ghost of the best run: its path as faint dots in the pilot's scarf
/// colour, and a gold checkered flag with that pilot's face where it ended.
pub fn ghost_path(sp: &Sprites, cam: &Cam, path: &[(f32, f32)], idx: usize, t: f32) {
    let s = cam.scale();
    let (sw, sh) = (screen_width(), screen_height());
    // Only the flag at the end marks the ghost: a trail of dots read as clutter.
    let _ = idx;
    if let Some((fx, _)) = path.last() {
        let p = cam.to_screen(*fx, 0.0);
        if p.x < -4.0 * s || p.x > sw + 4.0 * s || p.y > sh + s {
            return;
        }
        let top = crate::props::checkered_flag(p, s, t);
        let y0 = top.y;
        blit(&sp.dogs_heads[idx.min(2)][0], vec2(p.x, y0 - 0.8 * s), 1.5 * s, 0.0, false, WHITE);
    }
}

/// Soft ellipse on the ground under the dog; fades with altitude.
pub fn shadow(cam: &Cam, wx: f32, wy: f32) {
    let p = cam.to_screen(wx, 0.0);
    let s = cam.scale();
    let h = (wy - PILOT_RADIUS).max(0.0);
    let a = (0.32 * (1.0 - h / 70.0)).max(0.0);
    if a <= 0.0 || p.y > screen_height() + s {
        return;
    }
    let w = (1.1 + h * 0.03) * s;
    draw_ellipse(p.x, p.y + 0.05 * s, w, 0.28 * s, 0.0, Color::new(0.15, 0.2, 0.45, a));
}

// ---------------------------------------------------------------------------
// Dog
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum Face {
    Idle,
    Scream,
    Panic,
    Squint,
    Dizzy,
    #[allow(dead_code)]
    Grit,
    Whee,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Limbs {
    /// Holding the sled.
    Seated,
    /// Streamlined head-first dive (fast).
    Dive,
    /// Legs out like a star (apex, gentle flight).
    Spread,
    /// Wiggle frame 1: paws thrown up.
    FlapUp,
    /// Wiggle frame 2: paws slapped down.
    FlapDown,
    /// Flailing after a zap.
    Tumble,
    /// Legs braced wide for the ground.
    Land,
}

pub struct Pose {
    pub tilt: f32,
    pub flap: f32,
    pub gliding: bool,
    pub rocketing: bool,
    pub flying: bool,
    pub stretch: f32,
    pub spin: f32,
    pub face: Face,
    pub limbs: Limbs,
    /// 0..1 how fast the dog is going, for trailing cloth.
    #[allow(dead_code)]
    pub trail: f32,
    pub t: f32,
    /// Wiggle presses left (0..12), or -1 to hide the pips.
    #[allow(dead_code)]
    pub pips: i32,
    /// 0..1 how much of a meteor: curled into a rolling fireball.
    pub ball: f32,
    /// Encased in ice after a cold wind.
    pub frozen: bool,
    /// Procedural secondary motion from `anim::DogAnim` (springs, cape chain).
    pub rig: crate::anim::Rig,
}

pub fn head_tex<'a>(sp: &'a Sprites, idx: usize, face: Face) -> &'a Texture2D {
    let k = match face {
        Face::Idle => 0,
        Face::Scream => 1,
        Face::Panic => 2,
        Face::Squint => 3,
        Face::Dizzy => 4,
        Face::Grit => 5,
        Face::Whee => 6,
    };
    &sp.dogs_heads[idx.min(2)][k]
}

pub fn body_tex<'a>(sp: &'a Sprites, idx: usize, limbs: Limbs) -> &'a Texture2D {
    let k = match limbs {
        Limbs::Seated => 0,
        Limbs::Dive => 1,
        Limbs::Spread => 2,
        Limbs::FlapUp => 3,
        Limbs::FlapDown => 4,
        Limbs::Tumble => 5,
        Limbs::Land => 6,
    };
    &sp.dogs_bodies[idx.min(2)][k]
}

/// Where the head sits on each body sprite, in 256-canvas pixels from the
/// body centre, plus the head's own lean in radians.
fn head_seat(limbs: Limbs) -> (f32, f32, f32) {
    match limbs {
        Limbs::Seated => (52.0, -70.0, 0.0),
        Limbs::Dive => (40.0, -96.0, -0.3),
        Limbs::Spread => (0.0, -110.0, 0.0),
        Limbs::FlapUp => (0.0, -104.0, 0.1),
        Limbs::FlapDown => (0.0, -100.0, -0.1),
        Limbs::Tumble => (0.0, -110.0, 0.3),
        Limbs::Land => (0.0, -86.0, 0.0),
    }
}

/// Collar colour per pilot, used to tint speed echoes.
pub fn scarf_color(idx: usize) -> Color {
    [Color::new(0.9, 0.24, 0.24, 1.0), Color::new(0.23, 0.53, 0.84, 1.0), Color::new(1.0, 0.35, 0.63, 1.0)][idx.min(2)]
}

/// One tinted silhouette of the dog (body + head) used for speed echoes.
pub fn dog_echo(sp: &Sprites, p: Vec2, r: f32, idx: usize, limbs: Limbs, face: Face, tilt: f32, tint: Color) {
    let w = DOG_W * r * DOG_FLY_SCALE;
    let (c, s) = (tilt.cos(), tilt.sin());
    let (hx, hy, hrot) = head_seat(limbs);
    let u = w / 256.0;
    let head = vec2(p.x + (hx * c + hy * s) * u, p.y + (-hx * s + hy * c) * u);
    ears(sp, idx, head, u, w, tilt + hrot, -0.2, tint);
    blit_wh(body_tex(sp, idx, limbs), p, w, w, tilt, false, tint);
    blit_wh(head_tex(sp, idx, face), head, w, w, tilt + hrot, false, tint);
}

/// The dog after a crash: head-first in the snow, sinking, then only the
/// hind paws and tail poking out of the hole. `sink` is 0..1.
/// `size` scales the hole: 1 for an ordinary splat, up to about 2.2 for a
/// meteor, which also scorches the rim.
pub fn dog_buried(sp: &Sprites, cam: &Cam, x: f32, idx: usize, sink: f32, t: f32, size: f32) {
    let s = cam.scale();
    let r = PILOT_RADIUS * s;
    let w = DOG_W * r * DOG_FLY_SCALE;
    // Hole rim on the snow; a meteor's is wider, with a scorched ring.
    let g = cam.to_screen(x, 0.0);
    if size > 1.2 {
        let k = ((size - 1.2) / 1.0).clamp(0.0, 1.0);
        draw_ellipse(g.x, g.y + 0.1 * s, 3.4 * s * size, 0.9 * s * size, 0.0, Color::new(0.25, 0.18, 0.15, 0.55 * k));
        draw_ellipse(g.x, g.y + 0.05 * s, 2.6 * s * size, 0.7 * s * size, 0.0, Color::new(0.45, 0.30, 0.20, 0.6 * k));
    }
    draw_ellipse(g.x, g.y, 1.9 * s * size, 0.5 * s * size, 0.0, Color::new(0.2, 0.18, 0.3, 0.9));
    draw_ellipse(g.x, g.y - 0.12 * s, 1.9 * s * size, 0.42 * s * size, 0.0, Color::new(0.08, 0.06, 0.14, 1.0));
    // Two beats: first the dog is stuck head-first at the surface, squashed
    // and shuddering; then it slides down into the hole and stays with the
    // paws up. Stars orbit the tail end the whole time.
    let stuck = (sink / 0.5).clamp(0.0, 1.0);
    let slide = ((sink - 0.5) / 0.5).clamp(0.0, 1.0);
    let depth = 2.6 * slide * slide * size.max(1.0);
    let shudder = if slide <= 0.0 { (t * 38.0).sin() * 0.06 * (1.0 - stuck) } else { 0.0 };
    // The hit flattens the dog, then it jellies: stretched, squashed, settling.
    let squash = 1.0 - 0.42 * (1.0 - stuck).powf(1.5) * (stuck * 9.4).cos();
    let p = cam.to_screen(x + shudder, PILOT_RADIUS + 0.4 - depth);
    let tilt = std::f32::consts::PI + shudder * 2.0;
    let u = w / 256.0;
    let (hx, hy, _) = head_seat(Limbs::Land);
    let head = vec2(p.x + (hx * tilt.cos() + hy * tilt.sin()) * u, p.y + (-hx * tilt.sin() + hy * tilt.cos()) * u);
    ears(sp, idx, head, u, w, tilt, 0.4, WHITE);
    blit_wh(body_tex(sp, idx, Limbs::Land), p, w * (2.0 - squash), w * squash, tilt, false, WHITE);
    blit_wh(head_tex(sp, idx, Face::Dizzy), head, w * (2.0 - squash), w * squash, tilt, false, WHITE);
    // Dizzy stars circling above the hole.
    for i in 0..3 {
        let a = t * 4.0 + i as f32 * 2.09;
        let c = vec2(g.x + a.cos() * 1.3 * s, g.y - 2.2 * s + a.sin() * 0.35 * s);
        blit(&sp.sparkle, c, 0.7 * s, a * 1.5, false, Color::new(1.0, 0.9, 0.4, 0.95));
    }
}

/// Ear angle in radians for the left ear (right is mirrored): hanging when
/// not flying, out like wings in flight, and one up-then-down stroke per flap.
fn ear_angle(pose: &Pose) -> f32 {
    if !pose.flying {
        return 1.25;
    }
    let flutter = (pose.t * 9.0).sin() * 0.08;
    let stroke = (pose.flap * std::f32::consts::TAU).sin() * 0.9;
    -0.2 + flutter + stroke
}

/// Both ears, pivoting at the head's top corners.
fn ears(sp: &Sprites, idx: usize, head: Vec2, u: f32, w: f32, tilt: f32, ang: f32, tint: Color) {
    ears_lr(sp, idx, head, u, w, tilt, ang, ang, 1.0, tint);
}

/// Both ears with their own angles (see `anim::Rig::ear_l`) and a length
/// multiplier for the wind pulling them long.
fn ears_lr(sp: &Sprites, idx: usize, head: Vec2, u: f32, w: f32, tilt: f32, ang_l: f32, ang_r: f32, len: f32, tint: Color) {
    let (c, s) = (tilt.cos(), tilt.sin());
    let rel = |x: f32, y: f32| vec2(head.x + (x * c + y * s) * u, head.y + (-x * s + y * c) * u);
    let ew = w * 160.0 / 256.0 * len;
    let left = rel(-60.0, -50.0);
    let right = rel(60.0, -50.0);
    blit_pivot(&sp.dogs_ear[idx.min(2)], left, vec2(0.92, 0.5), ew, tilt + ang_l, false, tint);
    blit_pivot(&sp.dogs_ear[idx.min(2)], right, vec2(0.92, 0.5), ew, tilt - ang_r, true, tint);
}

pub fn dog(sp: &Sprites, p: Vec2, r: f32, idx: usize, pose: Pose) {
    let idx = idx.min(2);
    let rig = pose.rig;
    let scale = if rig.on { 1.0 + (DOG_FLY_SCALE - 1.0) * rig.grow } else if pose.flying { DOG_FLY_SCALE } else { 1.0 };
    let w = DOG_W * r * scale;
    let r = r * scale;
    let tilt = pose.tilt + pose.spin + rig.lean;
    let (c, s) = (tilt.cos(), tilt.sin());
    let at = |x: f32, y: f32| vec2(p.x + (x * c - y * s) * r, p.y - (x * s + y * c) * r);

    if pose.ball > 0.0 {
        // The meteor: a fireball around the curled dog, flickering, with a
        // tail of flame streaming up behind the fall. Sized from the figure
        // width so it wraps the whole dog.
        crate::vfx::fireball_back(p, w, pose.ball, pose.t);
    }
    if pose.rocketing {
        let flick = 1.0 + (pose.t * 40.0).sin() * 0.2;
        for (k, col) in [(1.6, Color::new(1.0, 0.55, 0.15, 0.9)), (1.0, Color::new(1.0, 0.85, 0.3, 0.9))] {
            let tail = at(-1.4 - k * 0.3, -0.05);
            blit(&sp.puff, tail, k * r * flick, 0.0, false, col);
        }
    }
    if pose.gliding {
        // Wings sit on the shoulders, behind the body.
        let top = at(-0.15, 0.35);
        let flap = (pose.t * 6.0).sin() * 0.15;
        blit_wh(&sp.wings, top, 4.6 * r, 4.6 * r * 160.0 / 240.0 * (1.0 + flap), tilt, false, WHITE);
    }

    // Curled up in the fireball the figure is about half size.
    let w = w * (1.0 - 0.45 * pose.ball);
    let r = r * (1.0 - 0.45 * pose.ball);
    // Each wiggle press hops the whole figure up and slaps it back down.
    let hop = (pose.flap * std::f32::consts::PI).sin();
    let p = vec2(p.x, p.y - hop * r * 0.35 - rig.hop * w);
    // Squash / stretch applies to the whole figure along its heading.
    let k = pose.stretch.clamp(-0.35, 0.35) - if pose.flap > 0.0 && pose.flap < 0.55 { 0.08 } else { 0.0 };
    let sx = ((1.0 + k) * rig.sx).clamp(0.6, 1.6);
    let sy = ((1.0 - k) * rig.sy).clamp(0.55, 1.4);
    // Soft drop shadow, then body, then head.
    blit_wh(&sp.puff, p + vec2(r * 0.1, r * 0.35), w * 1.1, w * 0.9, 0.0, false, Color::new(0.05, 0.06, 0.2, 0.28));
    // Wiggle frames roll the body: nose up on the Y, nose down on the tuck.
    let tilt = tilt + match pose.limbs {
        Limbs::FlapUp => 0.12,
        Limbs::FlapDown => -0.16,
        _ => 0.0,
    };
    let (c, s) = (tilt.cos(), tilt.sin());
    let (hx, hy, hrot) = head_seat(pose.limbs);
    let u = w / 256.0;
    let bob = (pose.flap * std::f32::consts::PI).sin() * 10.0;
    let (hx2, hy2) = ((hx + rig.head_off.x) * sx, (hy - bob + rig.head_off.y) * sy);
    let hrot = hrot + rig.head;
    let head = vec2(p.x + (hx2 * c + hy2 * s) * u, p.y + (-hx2 * s + hy2 * c) * u);
    let breed_lean = match idx {
        2 => (pose.t * 18.0).sin() * 0.06 * if pose.flying { 1.0 } else { 0.0 },
        _ => 0.0,
    };
    // Frozen: the dog itself goes icy blue and shivers, with icicles hanging
    // off it and frost glinting around; no block hides the character.
    let tint = if pose.frozen { Color::new(0.70, 0.86, 1.0, 1.0) } else { WHITE };
    let shiver = if pose.frozen { vec2((pose.t * 57.0).sin(), (pose.t * 43.0).cos()) * 0.018 * w } else { Vec2::ZERO };
    let (p, head) = (p + shiver, head + shiver);
    if rig.on {
        let collar = p + (head - p) * 0.5 - vec2(tilt.cos(), -tilt.sin()) * 0.04 * w;
        crate::anim::cape(collar, w, &rig, scarf_color(idx), pose.t);
        ears_lr(sp, idx, head, u, w, tilt + hrot + breed_lean, rig.ear_l, rig.ear_r, rig.ear_len, tint);
    } else {
        cape(p, head, w, tilt, &pose, idx);
        ears(sp, idx, head, u, w, tilt + hrot + breed_lean, ear_angle(&pose), tint);
    }
    blit_wh(body_tex(sp, idx, pose.limbs), p, w * sx, w * sy, tilt, false, tint);
    blit_wh(head_tex(sp, idx, pose.face), head, w * sx, w * sy, tilt + hrot + breed_lean, false, tint);
    if pose.frozen {
        // Body-local axes so the icicles hang from the underside however the dog tilts.
        let down = vec2(-tilt.sin(), tilt.cos());
        let right = vec2(tilt.cos(), tilt.sin());
        for k in 0..5 {
            let along = (k as f32 - 2.0) * 0.16 * w;
            let len = (0.08 + 0.05 * ((k * 7) % 3) as f32) * w;
            let base = p + right * along + down * 0.40 * w * sy;
            let half = 0.035 * w;
            let tip = base + down * len;
            draw_triangle(base - right * (half + 2.0) - down * 2.0, base + right * (half + 2.0) - down * 2.0, tip + down * 2.5, OUTLINE);
            draw_triangle(base - right * half, base + right * half, tip, Color::new(0.86, 0.95, 1.0, 1.0));
        }
        // Frost glints twinkling around the body.
        for k in 0..5 {
            let a = pose.t * 1.4 + k as f32 * 1.256;
            let c = p + vec2(a.cos() * 0.55 * w, a.sin() * 0.45 * w);
            let tw = 0.5 + 0.5 * (pose.t * 9.0 + k as f32 * 2.0).sin();
            blit(&sp.sparkle, c, (0.10 + 0.08 * tw) * w, a * 2.0, false, Color::new(0.85, 0.95, 1.0, 0.5 + 0.5 * tw));
        }
        // A little cloud of breath frost above the head.
        blit(&sp.puff, head - vec2(0.0, 0.32 * w) + shiver, 0.28 * w * (1.0 + 0.1 * (pose.t * 6.0).sin()), 0.0, false, Color::new(0.9, 0.97, 1.0, 0.45));
    }
    if pose.ball > 0.0 {
        // Fire in front of the dog too, so it burns inside the ball rather
        // than in front of a backdrop.
        crate::vfx::fireball_front(p, w / (1.0 - 0.45 * pose.ball), pose.ball, pose.t);
    }
}

/// A flying cape behind the dog: anchored at the collar, streaming against
/// the direction of travel, flaring and rippling with speed; it hangs down
/// while the dog sits on the sled. Drawn before the body so it stays behind.
fn cape(p: Vec2, head: Vec2, w: f32, tilt: f32, pose: &Pose, idx: usize) {
    let speed = pose.trail.clamp(0.0, 1.0);
    let facing = vec2(tilt.cos(), tilt.sin());
    let stream = if pose.flying {
        (-facing + vec2(0.0, 0.45 * (1.0 - speed))).normalize_or_zero()
    } else {
        vec2(-0.35, 1.0).normalize()
    };
    let side = vec2(-stream.y, stream.x);
    let collar = p + (head - p) * 0.5 - facing * 0.04 * w;
    let len = w * if pose.flying { 0.55 + 0.55 * speed } else { 0.42 };
    let (w0, w1) = (0.15 * w, (0.30 + 0.12 * speed) * w);
    let flap_rate = if pose.flying { 11.0 + 9.0 * speed } else { 3.0 };
    // Build the outline: left edge out, rippled hem, right edge back.
    let n = 7;
    let mut pts: Vec<Vec2> = Vec::with_capacity(2 * n + 2);
    for i in 0..=n {
        let k = i as f32 / n as f32;
        let ripple = (pose.t * flap_rate - k * 5.0).sin() * 0.07 * w * k;
        let half = w0 + (w1 - w0) * k;
        pts.push(collar + stream * (len * k) + side * (half + ripple));
    }
    for i in (0..=n).rev() {
        let k = i as f32 / n as f32;
        let ripple = (pose.t * flap_rate - k * 5.0 + 1.3).sin() * 0.07 * w * k;
        let half = w0 + (w1 - w0) * k;
        pts.push(collar + stream * (len * k) - side * (half + ripple));
    }
    let c = collar + stream * (len * 0.5);
    // An unlocked cape (Santa's orders, `orders.rs`) replaces the scarf colour.
    let fill = crate::orders::cape_fill().unwrap_or(scarf_color(idx));
    let dark = Color::new(fill.r * 0.72, fill.g * 0.6, fill.b * 0.6, 1.0);
    // Ink outline as a slightly inflated copy, then the fill, then a shaded inner fold.
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let grow = |q: Vec2| c + (q - c) * 1.0 + (q - c).normalize_or_zero() * 2.5;
        draw_triangle(c, grow(a), grow(b), Color::new(fill.r * 0.38, fill.g * 0.3, fill.b * 0.36, 1.0));
    }
    for i in 0..pts.len() {
        draw_triangle(c, pts[i], pts[(i + 1) % pts.len()], fill);
    }
    cape_cloth(&pts, collar, stream, side, len, w, pose.t * flap_rate, fill, dark);
}

/// Cloth shading over the cape outline built in `cape`: the darker lining
/// along the underside edge, fold valleys fanning out from the collar with
/// a lit crest beside each, and a gold hem trim. `pts` is the left edge
/// (collar to hem) followed by the right edge (hem to collar).
fn cape_cloth(pts: &[Vec2], collar: Vec2, stream: Vec2, side: Vec2, len: f32, w: f32, ph: f32, fill: Color, dark: Color) {
    let n = pts.len() / 2 - 1;
    let light = Color::new((fill.r * 1.25 + 0.12).min(1.0), (fill.g * 1.25 + 0.1).min(1.0), (fill.b * 1.2 + 0.1).min(1.0), 1.0);
    let lining = Color::new(fill.r * 0.55, fill.g * 0.42, fill.b * 0.5, 1.0);
    // Lining: a band inside the right (underside) edge.
    for i in 0..n {
        let (a, b) = (pts[n + 1 + i], pts[n + 2 + i]);
        let k_a = 1.0 - i as f32 / n as f32;
        let k_b = 1.0 - (i + 1) as f32 / n as f32;
        let ia = a + side * (0.03 + 0.07 * k_a) * w;
        let ib = b + side * (0.03 + 0.07 * k_b) * w;
        draw_triangle(a, b, ib, lining);
        draw_triangle(a, ib, ia, lining);
    }
    // Folds: valleys from the collar to the hem, each with a lit crest.
    for (j, &f) in [-0.45f32, 0.1, 0.6].iter().enumerate() {
        let wob = (ph + j as f32 * 1.7).sin();
        let top = collar + side * f * 0.10 * w;
        let end = collar + stream * len * (0.96 + 0.04 * wob) + side * (f * 0.34 * w + wob * 0.04 * w);
        let half = (0.045 + 0.015 * wob.abs()) * w;
        draw_triangle(top, end + side * half, end - side * half * 0.3, dark);
        draw_triangle(top + side * 0.01 * w, end + side * (half + 0.05 * w), end + side * half, Color::new(light.r, light.g, light.b, 0.75));
    }
    // Lit band along the leading (left, upper) edge.
    for i in 0..n {
        let (a, b) = (pts[i], pts[i + 1]);
        let ia = a - side * 0.035 * w;
        let ib = b - side * 0.035 * w;
        draw_triangle(a, b, ib, Color::new(light.r, light.g, light.b, 0.8));
        draw_triangle(a, ib, ia, Color::new(light.r, light.g, light.b, 0.8));
    }
    // Gold trim along the hem.
    let (h0, h1) = (pts[n], pts[n + 1]);
    let back = -stream * 0.05 * w;
    let gold = Color::new(1.0, 0.80, 0.25, 1.0);
    draw_triangle(h0, h1, h1 + back, gold);
    draw_triangle(h0, h1 + back, h0 + back, gold);
    draw_line(h0.x + back.x, h0.y + back.y, h1.x + back.x, h1.y + back.y, 0.012 * w, Color::new(0.75, 0.5, 0.1, 1.0));
}

/// The sled the dog rides down the ramp.
pub fn sled(sp: &Sprites, cam: &Cam, pos: Vec2, rot: f32) {
    let s = cam.scale();
    let p = cam.to_screen(pos.x, pos.y);
    blit(&sp.sled, p, 4.2 * s, rot, false, WHITE);
}

// ---------------------------------------------------------------------------
// Pickups and obstacles
// ---------------------------------------------------------------------------

/// `dog` is the pilot's position and velocity while flying, used to decide
/// which storm clouds are actually in the way.
pub fn entity(sp: &Sprites, cam: &Cam, e: &Entity, t: f32, dog: Option<(Vec2, Vec2)>) {
    let p = cam.to_screen(e.pos.x, e.pos.y);
    let s = cam.scale();
    if p.x < -6.0 * s || p.x > screen_width() + 6.0 * s || p.y < -6.0 * s || p.y > screen_height() + 6.0 * s {
        return;
    }
    let mut grow = 1.0;
    let mut a = 1.0;
    if !e.alive {
        // A struck storm cloud lingers so the player sees what hit them.
        let k = (e.dead_for * if e.kind == EntityKind::Stormcloud { 0.9 } else { 3.0 }).min(1.0);
        grow = 1.0 + k * 0.6;
        a = 1.0 - k;
        if a <= 0.0 {
            return;
        }
    }
    let tint = Color::new(1.0, 1.0, 1.0, a);
    // Ground shadow under airborne things so their height is legible.
    if e.kind != EntityKind::Trampoline && e.pos.y < 40.0 && e.alive {
        let gp = cam.to_screen(e.pos.x, 0.0);
        if gp.y < screen_height() + s {
            let k = 1.0 - e.pos.y / 40.0;
            draw_ellipse(gp.x, gp.y + 0.05 * s, (0.8 + 0.6 * k) * s, 0.22 * s, 0.0, Color::new(0.15, 0.2, 0.45, 0.22 * k * a));
        }
    }
    match e.kind {
        EntityKind::Trampoline => {
            let ground = cam.to_screen(e.pos.x, 0.0).y;
            draw_ellipse(p.x, ground + 0.05 * s, 2.0 * s, 0.3 * s, 0.0, Color::new(0.2, 0.25, 0.5, 0.25 * a));
            let bounce = 1.0 + (t * 6.0 + e.pos.x).sin().max(0.0) * 0.04;
            blit_wh(&sp.trampoline, vec2(p.x, ground - 1.6 * s * grow), 4.2 * s * grow, 4.2 * s * grow * 176.0 / 224.0 * bounce, 0.0, false, tint);
        }
        EntityKind::Reindeer => {
            let ph = t * 4.0 + e.pos.x;
            let bob = ph.sin() * 0.15 * s;
            blit(&sp.puff, vec2(p.x, p.y + bob), 5.5 * s, 0.0, false, Color::new(1.0, 0.8, 0.6, 0.18 * a));
            blit(&sp.reindeer, vec2(p.x, p.y + bob), 4.4 * s * grow, (ph * 0.5).cos() * 0.08, false, tint);
            // Rudolph's glow.
            blit(&sp.puff, vec2(p.x + 1.6 * s, p.y + bob - 0.1 * s), 1.2 * s * (1.0 + (t * 8.0).sin() * 0.2), 0.0, false, Color::new(1.0, 0.3, 0.3, 0.4 * a));
        }
        EntityKind::Wings => {
            let bob = (t * 2.0 + e.pos.x).sin() * 0.15 * s;
            let flap = 1.0 + (t * 5.0 + e.pos.x).sin() * 0.08;
            blit(&sp.puff, vec2(p.x, p.y + bob), 4.5 * s, 0.0, false, Color::new(1.0, 0.95, 0.7, 0.3 * a));
            blit_wh(&sp.wings, vec2(p.x, p.y + bob), 4.4 * s * grow, 4.4 * s * grow * 160.0 / 240.0 * flap, 0.0, false, tint);
        }
        EntityKind::Firework => crate::props::rocket(sp, e, p, s, t, grow, a),
        EntityKind::Present => crate::props::present(sp, e, p, s, t, grow, a),
        EntityKind::Sleigh => {
            let bob = (t * 2.5 + e.pos.x).sin() * 0.2 * s;
            blit(&sp.sparkle, vec2(p.x - 2.5 * s, p.y + bob + 0.5 * s), 1.4 * s, t * 2.0, false, Color::new(1.0, 0.95, 0.6, 0.5 * a));
            blit(&sp.sleigh, vec2(p.x, p.y + bob), 6.5 * s * grow, 0.05, false, tint);
        }
        EntityKind::Stormcloud => {
            let bob = (t * 1.5 + e.pos.x).sin() * 0.2 * s;
            let c = vec2(p.x, p.y + bob);
            // Only a cloud the dog is actually heading for charges: sample the
            // projected path over the next 1.6 s and look for a near pass.
            let mut threat = 0.0f32;
            let mut dog_at = None;
            if let Some((dp, dv)) = dog {
                let mut best = f32::MAX;
                for k in 0..=8 {
                    let tt = k as f32 * 0.2;
                    let q = vec2(dp.x + dv.x * tt, dp.y + dv.y * tt - 0.5 * 22.0 * 0.22 * tt * tt);
                    best = best.min((q - vec2(e.pos.x, e.pos.y)).length());
                }
                if e.pos.x > dp.x - 3.0 {
                    threat = (1.0 - (best - 4.0) / 8.0).clamp(0.0, 1.0);
                }
                dog_at = Some(dp);
            }
            // Charge rhythm while threatened: 0.6 s swell, 0.35 s held bolt, rest.
            let rate = 0.667;
            let cycle = (t * rate + e.pos.x * 0.1).rem_euclid(1.0);
            let charge = if cycle < 0.4 { cycle / 0.4 } else { 0.0 };
            let strike = cycle >= 0.4 && cycle < 0.63;
            let pre = charge * threat * (0.7 + 0.3 * (t * 20.0).sin());
            let lit = if strike && threat > 0.0 { threat } else { pre };
            if lit > 0.0 && e.alive {
                blit(&sp.puff, c, 9.0 * s * (1.0 + lit * 0.2), 0.0, false, Color::new(1.0, 0.25, 0.25, 0.5 * lit * a));
            }
            crate::props::storm(sp, e, c, s, t, grow, a, if e.alive { lit } else { 0.0 });
            if pre > 0.0 && e.alive {
                for k in 0..6 {
                    let ang = t * 9.0 + k as f32 * 1.05;
                    let rr = 3.2 * s;
                    let q = c + vec2(ang.cos() * rr, ang.sin() * rr * 0.6);
                    blit(&sp.sparkle, q, 0.9 * s, ang, false, Color::new(1.0, 0.95, 0.5, pre * a));
                }
            }
            if strike && threat > 0.0 && e.alive {
                // A glowing branched bolt aimed at the dog, held for the whole strike.
                let target = dog_at.map(|d| cam.to_screen(d.x, d.y)).unwrap_or(vec2(c.x, c.y + 6.0 * s));
                let dir = (target - c).normalize_or_zero();
                let len = (target - c).length().min(9.0 * s);
                crate::vfx::lightning(c + dir * 1.2 * s, c + dir * (1.2 * s + len), s, t, a);
            }
        }
        EntityKind::ColdWind => {
            // Streaks of icy air rushing left, with drifting flakes.
            let w = 7.5 * s * grow;
            let h = w * sp.coldwind.height() / sp.coldwind.width();
            let scroll = (t * 2.2).rem_euclid(1.0);
            for k in 0..2 {
                let off = (scroll + k as f32) * w - w;
                let x = p.x - off;
                let a = (1.0 - ((x - p.x).abs() / w)).clamp(0.0, 1.0);
                blit_wh(&sp.coldwind, vec2(x, p.y), w, h, 0.0, false, Color::new(1.0, 1.0, 1.0, tint.a * 0.9 * a.max(0.3)));
            }
            for k in 0..6 {
                let fx = ((t * 1.4 + k as f32 * 0.31 + e.pos.x).rem_euclid(1.0)) * w;
                let fy = ((k as f32 * 2.1 + e.pos.x).sin()) * h * 0.4;
                blit(&sp.snowflake, vec2(p.x + w * 0.5 - fx, p.y + fy), 0.35 * s, t, false, Color::new(0.85, 0.95, 1.0, 0.9 * tint.a));
            }
        }
        EntityKind::Updraft => {
            // A column of wind: two copies of the swirl sprite scrolling
            // upward, plus snow specks rising through it.
            let h = 9.0 * s * grow;
            let w = h * sp.updraft.width() / sp.updraft.height();
            let scroll = (t * 1.6).rem_euclid(1.0);
            for k in 0..2 {
                let off = (scroll + k as f32) * h - h;
                let y = p.y - off;
                let a = (1.0 - ((y - p.y).abs() / h)).clamp(0.0, 1.0);
                blit_wh(&sp.updraft, vec2(p.x, y), w, h, 0.0, false, Color::new(1.0, 1.0, 1.0, tint.a * 0.85 * a.max(0.35)));
            }
            for k in 0..8 {
                let fy = ((t * 2.0 + k as f32 * 0.37 + e.pos.x).rem_euclid(1.0)) * h;
                let fx = ((k as f32 * 1.7 + e.pos.x).sin()) * w * 0.35;
                draw_circle(p.x + fx, p.y + h * 0.5 - fy, 0.1 * s, Color::new(1.0, 1.0, 1.0, 0.8 * tint.a));
            }
        }
        EntityKind::Ufo => {
            let bob = (t * 3.0 + e.pos.x).sin() * 0.25 * s;
            let c = vec2(p.x, p.y + bob);
            // Tractor beam.
            draw_triangle(
                c + vec2(-1.2 * s, 0.4 * s),
                c + vec2(1.2 * s, 0.4 * s),
                c + vec2(0.0, 5.5 * s),
                Color::new(0.5, 1.0, 0.7, 0.18 * a * (0.8 + 0.2 * (t * 6.0).sin())),
            );
            blit(&sp.ufo, c, 5.5 * s * grow, (t * 1.5).sin() * 0.05, false, tint);
        }
    }
}

/// Just the hind paws of the buried dog sticking up out of the hole.
pub fn dog_paws_up(sp: &Sprites, cam: &Cam, x: f32, idx: usize, t: f32) {
    let s = cam.scale();
    let r = PILOT_RADIUS * s;
    let w = DOG_W * r * DOG_FLY_SCALE;
    // Legs kick: a quick alternating wobble and bob, slowing as the dog gives up.
    let kick = (t * 9.0).sin();
    let p = cam.to_screen(x + kick * 0.12, PILOT_RADIUS + 0.4 - 2.6 + (t * 18.0).sin().abs() * 0.12);
    // Source rect: the lower part of the land pose (legs), drawn flipped so it points up.
    let tex = body_tex(sp, idx, Limbs::Land);
    let (tw, th) = (tex.width(), tex.height());
    let src = Rect::new(0.0, th * 0.62, tw, th * 0.38);
    let dh = w * 0.38;
    let top = p.y + w / 2.0 - dh;
    let _ = top;
    // Flipped vertically: legs that pointed down now point up, above the rim.
    let g = cam.to_screen(x, 0.0);
    draw_texture_ex(
        tex,
        p.x - w / 2.0,
        g.y - dh + 0.2 * s - (t * 18.0).sin().abs() * 0.12 * s,
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(w, dh)), source: Some(src), flip_y: true, rotation: kick * 0.12, ..Default::default() },
    );
}
