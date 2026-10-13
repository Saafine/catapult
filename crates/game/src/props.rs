//! How the pickups and hazards are drawn: presents (five designs, a rare
//! gold one), the firework rocket with its live flame, and the storm cloud
//! with its crackling lightning. The art is painted in tools/props_painted.py;
//! this module gives it life (bob, sway, glow, glints, flicker). Gameplay
//! (hit boxes, effects) stays in hamsters-core: nothing here changes it.

use hamsters_core::Entity;
use macroquad::prelude::*;

use crate::draw::{hash, lerp, OUTLINE};
use crate::sprites::{blit, blit_wh, Sprites};

/// The extra textures this module adds to the pack.
pub struct PropSprites {
    pub present_purple: Texture2D,
    pub present_gold: Texture2D,
    pub rocket_flame: Texture2D,
    pub bolt: Texture2D,
}

async fn tex(base: &str, name: &str) -> Texture2D {
    let path = format!("{base}/{name}.png");
    let t = load_texture(&path).await.unwrap_or_else(|e| panic!("missing sprite {path}: {e:?}"));
    t.set_filter(FilterMode::Linear);
    t
}

impl PropSprites {
    pub async fn load(base: &str) -> Self {
        PropSprites {
            present_purple: tex(base, "present_purple").await,
            present_gold: tex(base, "present_gold").await,
            rocket_flame: tex(base, "rocket_flame").await,
            bolt: tex(base, "bolt").await,
        }
    }
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

/// Roughly one present in twenty is wrapped in gold foil.
pub fn is_gold(e: &Entity) -> bool {
    hash((e.pos.x * 7.0) as i32 + (e.pos.y * 3.0) as i32, 83) < 0.05
}

/// A wrapped present floating in the sky: it bobs and sways, breathes a
/// soft halo so it pops off the busy background, and every couple of
/// seconds a star glint runs over its foil. The gold one is bigger, turns
/// slowly in a ring of light rays and trails orbiting sparkles.
pub fn present(sp: &Sprites, e: &Entity, p: Vec2, s: f32, t: f32, grow: f32, a: f32) {
    let seed = (e.pos.x * 7.0) as i32;
    let ph = e.pos.x * 1.37;
    let gold = is_gold(e);
    let size = if gold { 3.4 } else { 2.6 + hash(seed, 84) * 0.6 };
    let bob = (t * 2.6 + ph).sin() * 0.22 * s;
    let c = vec2(p.x, p.y + bob);
    // Squash a touch at the bottom of the bob, stretch at the top.
    let sq = 1.0 + (t * 2.6 + ph).cos() * 0.035;
    let tilt = (t * 1.7 + ph).sin() * 0.13;
    let breathe = 0.9 + (t * 3.2 + ph).sin() * 0.1;
    let (tex, glow) = if gold {
        (&sp.props.present_gold, rgba(1.0, 0.82, 0.3, 1.0))
    } else {
        match (hash(seed, 81) * 4.0) as usize % 4 {
            0 => (&sp.presents[0], rgba(1.0, 0.55, 0.45, 1.0)),
            1 => (&sp.presents[1], rgba(0.6, 1.0, 0.6, 1.0)),
            2 => (&sp.presents[2], rgba(0.6, 0.8, 1.0, 1.0)),
            _ => (&sp.props.present_purple, rgba(0.9, 0.6, 1.0, 1.0)),
        }
    };
    let w = size * s * grow;
    if gold {
        // Slow rotating rays behind the gold present.
        let n = 10;
        let r0 = 0.5 * w;
        let r1 = 1.25 * w * breathe;
        for k in 0..n {
            let ang = t * 0.6 + k as f32 * std::f32::consts::TAU / n as f32;
            let half = 0.11;
            let a0 = vec2((ang - half).cos(), (ang - half).sin());
            let a1 = vec2((ang + half).cos(), (ang + half).sin());
            draw_triangle(c + a0 * r0, c + a1 * r0, c + (a0 + a1) * 0.5 * r1, rgba(1.0, 0.92, 0.5, 0.22 * a));
        }
        blit(&sp.puff, c, 2.6 * w * breathe, 0.0, false, rgba(1.0, 0.85, 0.35, 0.45 * a));
    }
    // A white rim halo separates the box from the city, a coloured one gives it warmth.
    blit(&sp.puff, c, 1.9 * w * breathe, 0.0, false, rgba(1.0, 1.0, 1.0, 0.35 * a));
    blit(&sp.puff, c, 2.2 * w * breathe, 0.0, false, Color { a: 0.28 * a, ..glow });
    blit_wh(tex, c, w / sq, w * sq, tilt, false, rgba(1.0, 1.0, 1.0, a));
    // Star glint sweeping over the foil, once every ~2.4 s, out of step per present.
    let g = (t * 0.42 + hash(seed, 85)).fract();
    if g < 0.14 {
        let k = (g / 0.14 * std::f32::consts::PI).sin();
        let at = c + vec2(-0.28 * w + 0.5 * w * g / 0.14, -0.05 * w);
        blit(&sp.sparkle, at, 0.75 * w * k, t * 3.0, false, rgba(1.0, 1.0, 1.0, 0.95 * a));
    }
    if gold {
        for k in 0..4 {
            let ang = t * 2.2 + k as f32 * 1.571;
            let q = c + vec2(ang.cos() * 0.75 * w, ang.sin() * 0.45 * w);
            let tw = 0.6 + 0.4 * (t * 7.0 + k as f32 * 2.0).sin();
            blit(&sp.sparkle, q, 0.42 * w * tw, ang, false, rgba(1.0, 0.95, 0.6, 0.95 * a));
        }
    }
}

/// The firework rocket hovers nose-up-right with its engine running: a
/// flickering exhaust flame, a hot glow, sparks spitting off the flame tip
/// and smoke drifting down, with a faint engine shudder.
pub fn rocket(sp: &Sprites, e: &Entity, p: Vec2, s: f32, t: f32, grow: f32, a: f32) {
    let ph = e.pos.x * 1.13;
    let rot = 0.32 + (t * 1.8 + ph).sin() * 0.06;
    let bob = (t * 2.2 + ph).sin() * 0.2 * s;
    let shake = vec2((t * 41.0).sin(), (t * 37.0).cos()) * 0.025 * s;
    let c = vec2(p.x, p.y + bob) + shake;
    let w = 5.2 * s * grow;
    let dir = vec2(rot.cos(), -rot.sin());
    let nozzle = c - dir * (0.5 - 40.0 / 260.0) * w;
    let flick = 1.0 + (t * 31.0 + ph).sin() * 0.18 + (t * 17.0).sin() * 0.1;
    let fl = 2.6 * s * flick;
    // Smoke trail curling down and back behind the flame.
    for k in 0..4 {
        let kf = k as f32;
        let f = (t * 0.9 + kf * 0.25 + ph).fract();
        let q = nozzle - dir * (fl * 0.8 + f * 2.0 * s) + vec2(0.0, f * 1.6 * s);
        blit(&sp.puff, q, (0.7 + f * 1.4) * s, 0.0, false, rgba(0.88, 0.88, 0.94, 0.32 * a * (1.0 - f)));
    }
    // Hot glow round the whole rocket and brighter at the nozzle.
    blit(&sp.puff, c, 2.0 * w, 0.0, false, rgba(1.0, 0.6, 0.3, 0.2 * a));
    blit(&sp.puff, nozzle - dir * fl * 0.3, 3.2 * s * flick, 0.0, false, rgba(1.0, 0.75, 0.3, 0.55 * a));
    blit_wh(&sp.props.rocket_flame, nozzle - dir * fl * 0.5, fl, 1.25 * s * (0.9 + 0.1 * flick), rot, false, rgba(1.0, 1.0, 1.0, a));
    blit(&sp.firework, c, w, rot, false, rgba(1.0, 1.0, 1.0, a));
    // Sparks spitting off the flame.
    for k in 0..3 {
        let f = (t * 2.6 + k as f32 * 0.33 + ph).fract();
        let side = vec2(-dir.y, dir.x) * ((k as f32 - 1.0) * 0.5 + (t * 13.0 + k as f32).sin() * 0.3) * s * f;
        let q = nozzle - dir * (fl * 0.6 + f * 1.6 * s) + side;
        blit(&sp.sparkle, q, 0.7 * s * (1.0 - f), t * 8.0, false, rgba(1.0, 0.9, 0.5, a * (1.0 - f)));
    }
    // A gleam travelling along the lacquer.
    let g = (t * 0.5 + ph).fract();
    if g < 0.16 {
        let k = (g / 0.16 * std::f32::consts::PI).sin();
        blit(&sp.sparkle, c + dir * (g / 0.16 - 0.4) * w * 0.7 - vec2(-dir.y, dir.x) * 0.38 * s, 1.1 * s * k, t, false, rgba(1.0, 1.0, 1.0, 0.9 * a));
    }
}

/// A thunderhead: a dark violet aura around it, sleet streaking down under
/// it and, every couple of seconds, lightning cracking out of its belly
/// with a flash through the cloud. `threat` and `strike` come from the
/// caller's look-ahead at the dog's path; the aimed strike bolt is drawn by
/// [`strike_bolt`]. Returns the cloud's on-screen width.
pub fn storm(sp: &Sprites, e: &Entity, c: Vec2, s: f32, t: f32, grow: f32, a: f32, lit: f32) -> f32 {
    let seed = (e.pos.x * 7.0) as i32;
    let w = 7.4 * s * grow * (1.0 + lit * 0.2);
    let h = w * sp.stormcloud.height() / sp.stormcloud.width();
    // Menace: a dark aura that also cuts the cloud out of the bright sky.
    blit(&sp.puff, c + vec2(0.0, 0.3 * s), 1.75 * w, 0.0, false, rgba(0.18, 0.14, 0.32, 0.38 * a));
    // Sleet slanting down from the belly.
    for k in 0..9 {
        let kf = k as f32;
        let f = (t * 1.8 + kf * 0.37 + hash(seed + k, 92)).fract();
        let x = c.x + (kf / 8.0 - 0.5) * w * 0.75 + f * 0.6 * s;
        let y = c.y + h * 0.32 + f * 2.8 * s;
        draw_line(x, y, x - 0.25 * s, y + 0.7 * s, 0.09 * s, rgba(0.78, 0.84, 1.0, 0.7 * a * (1.0 - f)));
    }
    // Idle crackle: a bolt forks out of the belly for a split second, out of
    // step per cloud, and the cloud lights up from inside.
    let period = 1.7 + hash(seed, 93) * 0.9;
    let cyc = t / period + hash(seed, 94);
    let n = cyc.floor() as i32;
    let f = cyc.fract();
    let mut flash = 0.0;
    if f < 0.28 && e.alive {
        flash = (1.0 - f / 0.28) * (0.6 + 0.4 * (t * 60.0).sin().abs());
        let off = (hash(n + seed, 95) - 0.5) * w * 0.55;
        let len = (4.2 + hash(n + seed, 96) * 2.4) * s;
        let top = vec2(c.x + off, c.y + h * 0.28);
        let bw = len * 64.0 / 192.0;
        blit(&sp.puff, top + vec2(0.0, len * 0.4), len * 1.4, 0.0, false, rgba(1.0, 0.95, 0.6, 0.45 * flash * a));
        blit_wh(&sp.props.bolt, top + vec2(0.0, len * 0.5), bw, len, (hash(n + seed, 97) - 0.5) * 0.5, hash(n + seed, 98) > 0.5, rgba(1.0, 1.0, 1.0, flash.min(1.0) * a));
    }
    let glow = (flash * 0.8).max(lit);
    if glow > 0.0 {
        blit(&sp.puff, c, 1.3 * w, 0.0, false, rgba(1.0, 0.95, 0.7, 0.35 * glow * a));
    }
    let tint = lerp(rgba(1.0, 1.0, 1.0, a), rgba(1.0, 0.97, 0.85, a), glow * 0.8);
    blit_wh(&sp.stormcloud, c, w, h, 0.0, false, tint);
    // Inner light shining through the billows during a flash.
    if glow > 0.0 {
        blit(&sp.puff, c - vec2(0.0, h * 0.15), 0.8 * w, 0.0, false, rgba(1.0, 1.0, 0.85, 0.4 * glow * a));
    }
    w
}

/// Painted snow chunk: a blue-grey edge, an ice-blue body and a white lit top.
fn snow_chunk(c: Vec2, r: f32, rot: f32) {
    draw_ellipse(c.x + 0.1 * r, c.y + 0.45 * r, 1.1 * r, 0.42 * r, 0.0, rgba(0.32, 0.4, 0.66, 0.35));
    draw_ellipse(c.x, c.y, r + 0.12 * r + 1.0, 0.74 * r + 0.12 * r + 1.0, rot, rgba(0.56, 0.69, 0.82, 1.0));
    draw_ellipse(c.x, c.y, r, 0.74 * r, rot, rgba(0.8, 0.88, 0.96, 1.0));
    draw_ellipse(c.x - 0.18 * r, c.y - 0.2 * r, 0.72 * r, 0.46 * r, rot, WHITE);
}

/// Chunks of packed snow thrown up round the crash hole, back row first,
/// so the hole reads as punched into the snow rather than painted on it.
/// `size` is the crash size (1 splat, up to ~2.2 meteor), `sink` 0..1.
pub fn crash_rim(cam: &crate::Cam, x: f32, size: f32, sink: f32) {
    let s = cam.scale();
    let g = cam.to_screen(x, 0.0);
    let pop = (sink * 5.0).min(1.0);
    let pop = 1.0 - (1.0 - pop) * (1.0 - pop);
    let (rx, ry) = (1.95 * s * size, 0.5 * s * size);
    let n = 12;
    let mut order: Vec<usize> = (0..n).collect();
    let ang = |i: usize| i as f32 / n as f32 * std::f32::consts::TAU + 0.25;
    order.sort_by(|a, b| ang(*a).sin().partial_cmp(&ang(*b).sin()).unwrap());
    for i in order {
        let a = ang(i);
        let back = a.sin() < 0.0;
        let r = (0.26 + hash(i as i32, 61) * 0.22) * s * size.sqrt() * pop * if back { 0.75 } else { 0.85 };
        let lift = if back { r * 0.35 } else { -r * 0.15 };
        let c = vec2(g.x + a.cos() * (rx + r * 0.25), g.y - 0.08 * s + a.sin() * (ry + r * 0.3) - lift);
        snow_chunk(c, r, (hash(i as i32, 62) - 0.5) * 0.6);
    }
    // A few loose crumbs flung further out.
    for i in 0..6 {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let d = rx * (1.25 + hash(i, 63) * 0.5);
        let c = vec2(g.x + side * d, g.y + (hash(i, 64) - 0.3) * ry);
        snow_chunk(c, (0.1 + hash(i, 65) * 0.08) * s * size.sqrt() * pop, 0.0);
    }
}

/// The record flag: a brass pole with a ball finial and a gold-and-ink
/// checkered cloth that ripples in the wind. Returns the top of the pole.
pub fn checkered_flag(p: Vec2, s: f32, t: f32) -> Vec2 {
    let h = 2.8 * s;
    let pole = rgba(0.62, 0.42, 0.14, 1.0);
    draw_ellipse(p.x, p.y + 0.05 * s, 0.6 * s, 0.16 * s, 0.0, rgba(0.15, 0.2, 0.45, 0.3));
    draw_rectangle(p.x - 0.1 * s, p.y - h, 0.2 * s, h, OUTLINE);
    draw_rectangle(p.x - 0.06 * s, p.y - h, 0.12 * s, h, pole);
    draw_rectangle(p.x - 0.05 * s, p.y - h, 0.04 * s, h, rgba(1.0, 0.86, 0.5, 1.0));
    let top = vec2(p.x, p.y - h);
    draw_circle(top.x, top.y - 0.12 * s, 0.22 * s, OUTLINE);
    draw_circle(top.x, top.y - 0.12 * s, 0.16 * s, rgba(1.0, 0.8, 0.25, 1.0));
    draw_circle(top.x - 0.05 * s, top.y - 0.17 * s, 0.06 * s, WHITE);
    // Cloth: cols x rows checks; each column is displaced by a travelling wave
    // and shaded by its slope so the folds catch the light.
    let (cols, rows) = (5usize, 3usize);
    let (fw, fh) = (2.2 * s, 1.25 * s);
    let x0 = p.x + 0.08 * s;
    let y0 = top.y + 0.08 * s;
    let wave = |u: f32| (u * 5.0 - t * 6.0).sin() * 0.16 * s * u;
    let mut edge_top = Vec::new();
    let mut edge_bot = Vec::new();
    for ci in 0..cols {
        let (u0, u1) = (ci as f32 / cols as f32, (ci + 1) as f32 / cols as f32);
        let (w0, w1) = (wave(u0), wave(u1));
        let slope = (w1 - w0) / (fw / cols as f32);
        let shade = (0.85 - slope * 2.5).clamp(0.55, 1.15);
        for ri in 0..rows {
            let (v0, v1) = (ri as f32 / rows as f32, (ri + 1) as f32 / rows as f32);
            let droop = |u: f32, v: f32| v * fh + u * u * 0.12 * s;
            let a = vec2(x0 + u0 * fw, y0 + droop(u0, v0) + w0);
            let b = vec2(x0 + u1 * fw, y0 + droop(u1, v0) + w1);
            let c = vec2(x0 + u1 * fw, y0 + droop(u1, v1) + w1);
            let d = vec2(x0 + u0 * fw, y0 + droop(u0, v1) + w0);
            let col = if (ci + ri) % 2 == 0 { rgba(1.0 * shade, 0.8 * shade, 0.25 * shade, 1.0) } else { rgba(0.18 * shade, 0.12 * shade, 0.14 * shade, 1.0) };
            draw_triangle(a, b, c, col);
            draw_triangle(a, c, d, col);
        }
        edge_top.push(vec2(x0 + u0 * fw, y0 + u0 * u0 * 0.12 * s + w0));
        edge_bot.push(vec2(x0 + u0 * fw, y0 + fh + u0 * u0 * 0.12 * s + w0));
    }
    let wl = wave(1.0);
    edge_top.push(vec2(x0 + fw, y0 + 0.12 * s + wl));
    edge_bot.push(vec2(x0 + fw, y0 + fh + 0.12 * s + wl));
    for q in edge_top.windows(2).chain(edge_bot.windows(2)) {
        draw_line(q[0].x, q[0].y, q[1].x, q[1].y, 0.07 * s, OUTLINE);
    }
    let (l, r) = (edge_top.last().unwrap(), edge_bot.last().unwrap());
    draw_line(l.x, l.y, r.x, r.y, 0.07 * s, OUTLINE);
    top
}
