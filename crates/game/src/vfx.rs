//! Drawn effects that are not particles: multi-depth snowfall with bokeh
//! flakes, the dog's motion ribbon and speed lines, the meteor fireball,
//! lightning bolts and the crash hole / bounce craters. All use the shared
//! procedural textures in `fxtex` and additive blending for light.

use macroquad::prelude::*;

use crate::draw::{hash, space_factor};
use crate::fxtex::{additive, fx};
use crate::particles::fire_ramp;
use crate::sprites::{blit, blit_wh, Sprites};
use crate::Cam;

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

/// Screen-space snow in four depths: a fine far haze, mid flakes, near
/// crystal flakes that tumble, and a few huge out-of-focus bokeh flakes in the
/// very foreground. Each depth has its own fall speed, sway and parallax, and
/// at speed every flake smears along the flight direction.
pub fn snowfall(sp: &Sprites, cam: &Cam, vel: Vec2, calm: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let time = get_time() as f32;
    // Snow thins with height so the sky opens up as you climb.
    let a = (1.0 - space_factor(cam)) * (1.0 - (cam.y / 160.0).clamp(0.0, 0.65)) * calm;
    if a <= 0.0 {
        return;
    }
    let tx = fx();
    let speed = vel.length();
    let streak = ((speed - 16.0) / 30.0).clamp(0.0, 1.0);
    let dir = if speed > 1.0 { vec2(vel.x, -vel.y) / speed } else { vec2(1.0, 0.0) };
    let rot = -dir.y.atan2(dir.x);
    let s = cam.scale();
    // (count, salt, size px lo..hi, fall px/s lo..hi, sway px, parallax, alpha)
    let layers: [(i32, u32, f32, f32, f32, f32, f32, f32, f32); 4] = [
        (150, 300, 2.0, 3.5, 18.0, 30.0, 8.0, 0.12, 0.55),
        (90, 310, 4.5, 8.0, 40.0, 65.0, 22.0, 0.45, 0.85),
        (26, 320, 13.0, 22.0, 85.0, 130.0, 40.0, 1.1, 0.95),
        (7, 330, 50.0, 95.0, 130.0, 190.0, 60.0, 2.2, 0.2),
    ];
    for (li, &(n, salt, s0, s1, f0, f1, sway, par, alpha)) in layers.iter().enumerate() {
        let margin = s1 * 2.0;
        let (wrap_w, wrap_h) = (sw + margin * 2.0, sh + margin * 2.0);
        for i in 0..n {
            let h = |k: u32| hash(i, salt + k);
            let size = s0 + (s1 - s0) * h(1);
            let fall = f0 + (f1 - f0) * h(2);
            let sx = (time * (0.6 + h(3)) + h(4) * 40.0).sin() * sway;
            let x = (h(5) * wrap_w + sx - cam.x * s * par).rem_euclid(wrap_w) - margin;
            let y = (h(6) * wrap_h + time * fall + cam.y * s * par).rem_euclid(wrap_h) - margin;
            let p = vec2(x, y);
            let len = size * (1.0 + streak * (1.0 + 1.6 * par.min(1.5)));
            match li {
                0 | 1 => {
                    let c = rgba(1.0, 1.0, 1.0, a * alpha * (0.6 + 0.4 * h(7)));
                    if li == 1 {
                        // A faint cool rim so mid flakes read on white cloud too.
                        blit_wh(&tx.soft, p + vec2(0.0, 0.8), len * 1.15, size * 1.15, rot, false, rgba(0.55, 0.65, 0.85, c.a * 0.35));
                    }
                    blit_wh(&tx.soft, p, len, size, rot, false, c);
                }
                2 => {
                    let spin = time * (h(8) - 0.5) * 3.0 + h(9) * 6.0;
                    let c = rgba(1.0, 1.0, 1.0, a * alpha);
                    if streak > 0.2 {
                        blit_wh(&tx.soft, p, len, size * 0.6, rot, false, rgba(1.0, 1.0, 1.0, a * alpha * 0.8));
                    } else {
                        blit_wh(&sp.snowflake, p, size, size, spin, false, c);
                    }
                }
                _ => {
                    let c = rgba(0.94, 0.97, 1.0, a * alpha * (0.6 + 0.4 * h(7)));
                    blit_wh(&tx.bokeh, p, len, size, rot, false, c);
                }
            }
        }
    }
}

/// A tapered wind ribbon behind the dog at speed, white with an additive
/// core in the pilot's scarf colour, plus two thin wingtip contrails.
pub fn motion_trail(cam: &Cam, trail: &[Vec2], head: Vec2, speed: f32, tint: Color, r: f32) {
    let k = ((speed - 12.0) / 28.0).clamp(0.0, 1.0);
    if k <= 0.0 || trail.len() < 3 {
        return;
    }
    let mut pts: Vec<Vec2> = trail.iter().map(|p| cam.to_screen(p.x, p.y)).collect();
    pts.push(cam.to_screen(head.x, head.y));
    let tx = fx();
    let n = pts.len();
    let ribbon = |width: f32, off: f32, col: Color, alpha: f32| {
        let mut vertices = Vec::with_capacity(n * 2);
        let mut indices: Vec<u16> = Vec::with_capacity((n - 1) * 6);
        for i in 0..n {
            let prev = pts[i.saturating_sub(1)];
            let next = pts[(i + 1).min(n - 1)];
            let tng = (next - prev).normalize_or_zero();
            let nrm = vec2(-tng.y, tng.x);
            let u = i as f32 / (n - 1) as f32;
            let w = width * u.powf(0.7);
            let c = Color::new(col.r, col.g, col.b, alpha * u * u);
            let base = pts[i] + nrm * off * u;
            vertices.push(Vertex::new(base.x + nrm.x * w, base.y + nrm.y * w, 0.0, u, 0.0, c));
            vertices.push(Vertex::new(base.x - nrm.x * w, base.y - nrm.y * w, 0.0, u, 1.0, c));
            if i + 1 < n {
                let j = (i * 2) as u16;
                indices.extend_from_slice(&[j, j + 1, j + 2, j + 1, j + 3, j + 2]);
            }
        }
        draw_mesh(&Mesh { vertices, indices, texture: Some(tx.strip.clone()) });
    };
    ribbon(r * 1.1, 0.0, rgba(1.0, 1.0, 1.0, 1.0), 0.55 * k);
    ribbon(r * 0.22, r * 1.1, rgba(1.0, 1.0, 1.0, 1.0), 0.7 * k);
    ribbon(r * 0.22, -r * 0.9, rgba(1.0, 1.0, 1.0, 1.0), 0.7 * k);
    additive(|| ribbon(r * 0.55, 0.0, Color::new(tint.r * 0.6 + 0.4, tint.g * 0.6 + 0.4, tint.b * 0.6 + 0.4, 1.0), 0.6 * k));
}

/// Anime speed lines raking past the frame along the flight direction,
/// kept clear of a band around the dog.
pub fn speed_lines(cam: &Cam, vel: Vec2, dog: Vec2, t: f32) {
    let speed = vel.length();
    let k = ((speed - 24.0) / 26.0).clamp(0.0, 1.0);
    if k <= 0.0 {
        return;
    }
    let _ = cam;
    let tx = fx();
    let (sw, sh) = (screen_width(), screen_height());
    let dir = vec2(vel.x, -vel.y) / speed;
    let side = vec2(-dir.y, dir.x);
    let ang = vel.y.atan2(vel.x);
    let centre = vec2(sw * 0.5, sh * 0.5);
    let diag = (sw * sw + sh * sh).sqrt() * 0.5;
    for i in 0..22 {
        let lane = (hash(i, 501) * 2.0 - 1.0) * diag;
        let q = centre + side * lane;
        // Keep the dog's own band clear.
        if ((q - dog).dot(side)).abs() < 90.0 {
            continue;
        }
        let rate = 1.4 + hash(i, 502) * 1.4;
        let ph = (t * rate + hash(i, 503)).fract();
        let along = (0.5 - ph) * diag * 2.6;
        let len = (110.0 + 230.0 * hash(i, 504)) * (0.5 + 0.5 * k);
        let p = q + dir * along;
        let a = (std::f32::consts::PI * ph).sin() * 0.5 * k;
        blit_wh(&tx.line, p, len, 2.0 + 3.0 * hash(i, 505), ang, false, rgba(1.0, 1.0, 1.0, a));
    }
}

/// The meteor fireball behind the curled dog: a hot halo and a flame tail
/// streaming up from the fall. `w` is the figure width, `b` 0..1.
pub fn fireball_back(p: Vec2, w: f32, b: f32, t: f32) {
    let tx = fx();
    let flick = 1.0 + (t * 37.0).sin() * 0.1 + (t * 23.0).cos() * 0.07;
    // Painted body first so the fire has a shape on the bright sky: puffs
    // whose blue-grey rims turn deep red-brown under the fire tint, a smoke
    // wake at the tip of the tail, then a yellow heart.
    for i in 0..9 {
        let k = i as f32 / 8.0;
        let wob = (t * 13.0 + i as f32 * 1.7).sin() * 0.14 * w * (0.3 + k);
        let tail = vec2(p.x + wob, p.y - (0.35 + 2.1 * k) * w * b);
        let c = if k > 0.8 { rgba(0.45, 0.40, 0.42, 1.0) } else { fire_ramp(0.35 + 0.6 * k) };
        let a = if k > 0.8 { 0.5 } else { 0.95 - 0.5 * k };
        blit(&tx.puffs[i % 3], tail, (1.45 - 0.75 * k) * w * b * flick, t * 2.0 + i as f32, i % 2 == 0, rgba(c.r, c.g, c.b, a * b));
    }
    let c = fire_ramp(0.42);
    blit(&tx.puffs[0], p, 1.75 * w * b * flick, t * 1.5, false, rgba(c.r, c.g, c.b, 0.95 * b));
    let c = fire_ramp(0.2);
    blit(&tx.puffs[1], p - vec2(0.0, 0.1 * w), 1.3 * w * b * flick, -t * 2.0, true, rgba(c.r, c.g, c.b, 0.9 * b));
    additive(|| {
        blit(&tx.glow, p, 3.0 * w * b * flick, 0.0, false, rgba(1.0, 0.45, 0.1, 0.3 * b));
        blit(&tx.glow, p, 1.4 * w * b * flick, 0.0, false, rgba(1.0, 0.75, 0.3, 0.35 * b));
    });
}

/// Fire in front of the dog, so it burns inside the ball.
pub fn fireball_front(p: Vec2, w: f32, b: f32, t: f32) {
    let tx = fx();
    let flick = 1.0 + (t * 41.0).sin() * 0.1;
    let c = fire_ramp(0.3);
    blit(&tx.mist, p, 1.5 * w * b * flick, -t * 2.5, true, rgba(c.r, c.g, c.b, 0.35 * b));
    additive(|| {
        blit(&tx.glow, p, 1.1 * w * b, 0.0, false, rgba(1.0, 0.85, 0.5, 0.3 * b));
    });
}

/// A branching lightning bolt from `from` to `to` in screen space: a wide
/// additive blue glow, a deep-violet edge, a white-hot core and a flare at
/// both ends. Re-jags 30 times a second. `s` is pixels per metre.
pub fn lightning(from: Vec2, to: Vec2, s: f32, t: f32, a: f32) {
    let seed = (t * 30.0).floor() as i32;
    let bolt = |a0: Vec2, a1: Vec2, levels: i32, salt: u32| -> Vec<Vec2> {
        let mut pts = vec![a0, a1];
        let mut amp = (a1 - a0).length() * 0.22;
        for l in 0..levels {
            let mut next = Vec::with_capacity(pts.len() * 2);
            for i in 0..pts.len() - 1 {
                let (p, q) = (pts[i], pts[i + 1]);
                let d = q - p;
                let nrm = vec2(-d.y, d.x).normalize_or_zero();
                let off = (hash(seed * 64 + i as i32 + l * 1000, salt) - 0.5) * 2.0 * amp;
                next.push(p);
                next.push((p + q) * 0.5 + nrm * off);
            }
            next.push(*pts.last().unwrap());
            pts = next;
            amp *= 0.55;
        }
        pts
    };
    let main = bolt(from, to, 4, 91);
    let mut bolts = vec![(main.clone(), 1.0f32)];
    for (bi, idx) in [4usize, 9].iter().enumerate() {
        if let Some(&root) = main.get(*idx) {
            let d = (to - from).normalize_or_zero();
            let turn = if bi == 0 { 0.7f32 } else { -0.6 };
            let bd = vec2(d.x * turn.cos() - d.y * turn.sin(), d.x * turn.sin() + d.y * turn.cos());
            let end = root + bd * (to - from).length() * 0.32;
            bolts.push((bolt(root, end, 3, 97 + bi as u32), 0.55));
        }
    }
    let tx = fx();
    let seg = |p: Vec2, q: Vec2, w: f32, c: Color| {
        let d = q - p;
        let len = d.length() + w * 0.5;
        blit_wh(&tx.strip, (p + q) * 0.5, len, w, (-d.y).atan2(d.x), false, c);
    };
    additive(|| {
        for (pts, wk) in &bolts {
            for i in 0..pts.len() - 1 {
                seg(pts[i], pts[i + 1], 1.6 * s * wk, rgba(0.45, 0.6, 1.0, 0.55 * a));
            }
        }
        blit(&tx.glow, to, 4.5 * s, 0.0, false, rgba(0.8, 0.85, 1.0, 0.8 * a));
        blit(&tx.glow, from, 3.0 * s, 0.0, false, rgba(0.8, 0.85, 1.0, 0.6 * a));
    });
    for (pts, wk) in &bolts {
        for i in 0..pts.len() - 1 {
            let (p, q) = (pts[i], pts[i + 1]);
            draw_line(p.x, p.y, q.x, q.y, 0.24 * s * wk, rgba(0.30, 0.28, 0.72, a));
        }
    }
    for (pts, wk) in &bolts {
        for i in 0..pts.len() - 1 {
            let (p, q) = (pts[i], pts[i + 1]);
            draw_line(p.x, p.y, q.x, q.y, 0.11 * s * wk, rgba(1.0, 0.99, 0.85, a));
        }
    }
    additive(|| {
        for (pts, wk) in &bolts {
            for i in 0..pts.len() - 1 {
                seg(pts[i], pts[i + 1], 0.5 * s * wk, rgba(1.0, 1.0, 0.9, 0.7 * a));
            }
        }
    });
}

/// The crash hole on the snow: a thrown-up lip of snow lumps with
/// blue-grey undersides, a deep ice-blue throat lit on its far wall, and for
/// a meteor (`size` > 1.2) a scorched ring with embers glowing in the rim.
pub fn crash_hole(cam: &Cam, x: f32, size: f32, t: f32) {
    let s = cam.scale();
    let g = cam.to_screen(x, 0.0);
    let (rx, ry) = (1.9 * s * size, 0.5 * s * size);
    let heat = ((size - 1.2) / 1.0).clamp(0.0, 1.0);
    let edge = rgba(0.50, 0.60, 0.80, 1.0);
    if heat > 0.0 {
        draw_ellipse(g.x, g.y + 0.06 * s, rx * 1.75, ry * 1.9, 0.0, rgba(0.30, 0.22, 0.20, 0.55 * heat));
        draw_ellipse(g.x, g.y + 0.04 * s, rx * 1.45, ry * 1.55, 0.0, rgba(0.42, 0.30, 0.24, 0.65 * heat));
    }
    // Snow lip: a raised ring around the hole.
    draw_ellipse(g.x, g.y + 0.08 * s, rx * 1.24, ry * 1.42, 0.0, edge);
    draw_ellipse(g.x, g.y, rx * 1.2, ry * 1.3, 0.0, mix(rgba(0.98, 0.99, 1.0, 1.0), rgba(0.85, 0.72, 0.62, 1.0), heat * 0.6));
    // Throat: the far wall catches light, the near side drops into shade.
    draw_ellipse(g.x, g.y, rx, ry, 0.0, rgba(0.36, 0.46, 0.74, 1.0));
    draw_ellipse(g.x, g.y + ry * 0.22, rx * 0.94, ry * 0.8, 0.0, rgba(0.17, 0.21, 0.42, 1.0));
    draw_ellipse(g.x, g.y + ry * 0.35, rx * 0.72, ry * 0.55, 0.0, rgba(0.09, 0.10, 0.24, 1.0));
    // Lumps of thrown snow along the near lip and the sides.
    for i in 0..9 {
        let k = i as f32 / 8.0;
        let ang = 0.05 + k * (std::f32::consts::PI - 0.1);
        let lx = g.x + ang.cos() * rx * 1.12;
        let ly = g.y + ang.sin() * ry * 1.2;
        let lw = (0.32 + 0.22 * hash(i, 611)) * s * size;
        let lh = lw * (0.5 + 0.15 * hash(i, 612));
        draw_ellipse(lx, ly + lh * 0.35, lw * 1.08, lh * 1.05, 0.0, edge);
        let c = mix(rgba(1.0, 1.0, 1.0, 1.0), rgba(0.88, 0.76, 0.66, 1.0), heat * 0.5);
        draw_ellipse(lx, ly, lw, lh, 0.0, c);
        draw_ellipse(lx - lw * 0.25, ly - lh * 0.3, lw * 0.35, lh * 0.3, 0.0, rgba(1.0, 1.0, 1.0, 0.9));
    }
    if heat > 0.0 {
        let tx = fx();
        let pulse = 0.8 + 0.2 * (t * 5.0).sin();
        additive(|| {
            blit_wh(&tx.glow, vec2(g.x, g.y + ry * 0.2), rx * 2.2, ry * 2.6, 0.0, false, rgba(1.0, 0.45, 0.1, 0.55 * heat * pulse));
            for i in 0..7 {
                let ang = hash(i, 621) * std::f32::consts::TAU;
                let e = vec2(g.x + ang.cos() * rx * 0.85, g.y + ang.sin() * ry * 0.8);
                let fl = 0.6 + 0.4 * (t * (6.0 + 3.0 * hash(i, 622)) + i as f32).sin();
                blit(&tx.glow, e, 0.9 * s * size * fl, 0.0, false, rgba(1.0, 0.6, 0.2, 0.7 * heat));
            }
        });
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

/// A bounce mark in the snow: a shallow ice-blue bowl with a white splash
/// rim and a few thrown clumps.
pub fn snow_crater(cam: &Cam, x: f32, size: f32) {
    let s = cam.scale();
    let p = cam.to_screen(x, 0.0);
    let (rx, ry) = (2.4 * s * size, 0.55 * s * size);
    draw_ellipse(p.x, p.y + 0.1 * s, rx * 1.1, ry * 1.2, 0.0, rgba(0.55, 0.64, 0.84, 0.55));
    draw_ellipse(p.x, p.y + 0.04 * s, rx, ry, 0.0, rgba(0.98, 0.99, 1.0, 0.95));
    draw_ellipse(p.x, p.y + 0.08 * s, rx * 0.8, ry * 0.62, 0.0, rgba(0.70, 0.79, 0.95, 0.95));
    draw_ellipse(p.x, p.y + 0.12 * s, rx * 0.55, ry * 0.38, 0.0, rgba(0.58, 0.68, 0.90, 0.95));
    for i in 0..4 {
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let d = (1.05 + 0.35 * (i / 2) as f32) * rx;
        let lw = (0.28 + 0.12 * hash(i + (x as i32) * 7, 631)) * s * size;
        let (lx, ly) = (p.x + side * d, p.y - 0.05 * s + (i / 2) as f32 * 0.12 * s);
        draw_ellipse(lx, ly + lw * 0.2, lw * 1.08, lw * 0.5, 0.0, rgba(0.55, 0.64, 0.84, 0.8));
        draw_ellipse(lx, ly, lw, lw * 0.45, 0.0, rgba(1.0, 1.0, 1.0, 1.0));
    }
}
