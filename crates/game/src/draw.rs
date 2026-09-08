//! Scene drawing with sprites: sky, parallax layers, ground dressing,
//! the workshop tower and ramp, the dog, pickups.

use hamsters_core::{ramp_point, Entity, EntityKind, PILOT_RADIUS, RAMP_TOP, SPACE_ALTITUDE};
use macroquad::prelude::*;

use crate::sprites::{blit, blit_pivot, blit_standing, blit_wh, Sprites};
use crate::Cam;

pub const SEED: Color = Color::new(1.0, 0.86, 0.30, 1.0);
pub const OUTLINE: Color = Color::new(0.12, 0.14, 0.25, 1.0);
const WOOD: Color = Color::new(0.70, 0.46, 0.25, 1.0);
const WOOD_LIGHT: Color = Color::new(0.82, 0.60, 0.38, 1.0);
const WOOD_DARK: Color = Color::new(0.40, 0.25, 0.12, 1.0);
const SNOW: Color = Color::new(0.97, 0.98, 1.0, 1.0);

/// Fur mid-tone per dog, used to tint the greyscale paw sprite.

// World sizes of sprites, in meters (width).
const DOG_W: f32 = 3.3;
const DOG_FLY_SCALE: f32 = 1.8;
const TOWER_W: f32 = 9.0;

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

// ---------------------------------------------------------------------------
// Sky
// ---------------------------------------------------------------------------

/// Christmas twilight that turns to space with altitude, with stars, moon and
/// aurora ribbons.
pub fn sky(sp: &Sprites, cam: &Cam) {
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;
    let space = space_factor(cam);
    // Each 75 m band of distance deepens the night a step.
    let band = ((cam.x / 150.0).floor() * 0.02).clamp(0.0, 0.08);
    let space = (space + band).min(1.0);
    // Bright winter day: deep cyan up top, pale at the horizon; the sky
    // only goes dark on the way into space.
    let top = lerp(Color::new(0.24, 0.62, 0.95, 1.0), Color::new(0.01, 0.01, 0.05, 1.0), space);
    let mid = lerp(Color::new(0.55, 0.82, 0.98, 1.0), Color::new(0.03, 0.04, 0.12, 1.0), space);
    let low = lerp(Color::new(0.88, 0.96, 1.0, 1.0), Color::new(0.10, 0.14, 0.32, 1.0), space);
    let strips = 72;
    let h = sh / strips as f32;
    for i in 0..strips {
        let k = i as f32 / (strips - 1) as f32;
        let c = if k < 0.5 { lerp(top, mid, (k / 0.5).powf(1.2)) } else { lerp(mid, low, ((k - 0.5) / 0.5).powf(1.6)) };
        draw_rectangle(0.0, i as f32 * h, sw, h + 1.0, c);
    }
    // Stars only once the sky darkens towards space.
    let star_a = space;
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
    let mx = sw * 0.78 - cam.x * 0.02 * cam.zoom;
    let my = sh * 0.20 + cam.y * 0.03 * cam.zoom;
    if space < 1.0 {
        blit(&sp.sun, vec2(mx, my), 230.0, t * 0.05, false, Color::new(1.0, 1.0, 1.0, 1.0 - space));
    }
    if space > 0.0 {
        blit(&sp.moon, vec2(mx - 200.0, my + 40.0), 200.0, 0.0, false, Color::new(1.0, 1.0, 1.0, space));
    }
    // Aurora fades out as we leave the atmosphere.
    aurora(cam, t, 1.0 - space);
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
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;
    // Snow thins with height so the sky opens up as you climb.
    let a = (1.0 - space_factor(cam)) * (1.0 - (cam.y / 160.0).clamp(0.0, 0.65)) * calm;
    if a <= 0.0 {
        return;
    }
    let speed = vel.length();
    let streak = ((speed - 18.0) / 30.0).clamp(0.0, 1.0);
    let dir = if speed > 1.0 { vec2(vel.x, -vel.y) / speed } else { vec2(1.0, 0.0) };
    let rot = -dir.y.atan2(dir.x);
    for i in 0..110 {
        let depth = 0.4 + hash(i, 31) * 0.6;
        let speed = 25.0 + 40.0 * depth;
        let x = (hash(i, 32) * sw + (t * 12.0 * depth + hash(i, 33) * 90.0).sin() * 30.0 - cam.x * 6.0 * depth * cam.zoom).rem_euclid(sw);
        let y = (hash(i, 34) * sh + t * speed + cam.y * 4.0 * depth * cam.zoom).rem_euclid(sh);
        let size = 4.0 + 8.0 * depth;
        // At speed the flakes smear into short lines along the flight direction.
        let len = size * (1.0 + streak * 6.0 * depth);
        blit_wh(&sp.snowflake, vec2(x, y), len, size, rot, false, Color::new(1.0, 1.0, 1.0, a * (0.35 + 0.5 * depth)));
    }
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
pub fn near_clouds(sp: &Sprites, cam: &Cam, speed: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let s = cam.scale();
    let factor = 1.35;
    const PERIOD: f32 = 300.0;
    let a = ((speed - 10.0) / 30.0).clamp(0.0, 1.0) * 0.5;
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
    let tint = lerp(Color::new(0.62, 0.66, 0.92, 1.0), WHITE, (factor / 0.8).clamp(0.0, 1.0));
    // Near layers sink out of frame as the camera climbs, far ones barely move.
    let factor_y = factor * factor * 0.9;
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
    let scroll = cam.x * factor * cam.scale();
    let passes: &[(f32, f32)] = if smear > 1.0 { &[(-1.0, 0.35), (1.0, 0.35), (0.0, 0.6)] } else { &[(0.0, 1.0)] };
    for (off, alpha) in passes {
        let mut x = -(scroll.rem_euclid(w)) - w + off * smear;
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
    layer(&sp.mountains_far, cam, 0.10, 200.0, 30.0, Color::new(0.46, 0.47, 0.70, 1.0), Color::new(0.42, 0.45, 0.68, 1.0));
    layer(&sp.mountains_near, cam, 0.18, 160.0, 8.0, Color::new(0.40, 0.42, 0.64, 1.0), Color::new(0.37, 0.40, 0.62, 1.0));
}

pub fn hills(sp: &Sprites, cam: &Cam, speed: f32) {
    let smear = ((speed - 22.0) / 10.0).clamp(0.0, 4.0) * 2.5;
    layer_smear(&sp.hills_near, cam, 0.45, 260.0, 0.0, Color::new(0.77, 0.82, 0.93, 1.0), Color::new(0.72, 0.78, 0.92, 1.0), smear * 0.6);
    meadow(sp, cam, 0.80);
    layer_smear(&sp.treeline, cam, 0.80, 105.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.0), Color::new(0.0, 0.0, 0.0, 0.0), smear);
    // Lower the treeline's contrast so airborne pickups read against it.
    let sh = screen_height();
    let sw = screen_width();
    let horizon = cam.to_screen_parallax(0.0, 0.0, 0.8 * 0.8 * 0.9).y;
    let top = horizon - 105.0 * cam.zoom.powf(0.4);
    if top < sh {
        draw_rectangle(0.0, top.max(0.0), sw, (horizon - top.max(0.0)).max(0.0), Color::new(0.35, 0.38, 0.62, 0.40));
    }
}

fn meadow(sp: &Sprites, cam: &Cam, factor: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let zz = cam.zoom.powf(0.4);
    let factor_y = factor * factor * 0.9;
    let horizon = cam.to_screen_parallax(0.0, 0.0, factor_y).y - 4.0;
    if horizon > sh {
        return;
    }
    let tw = 260.0 * zz;
    let th = tw * sp.meadow.height() / sp.meadow.width();
    let scroll = cam.x * factor * cam.scale();
    let mut x = -(scroll.rem_euclid(tw)) - tw;
    while x < sw {
        let mut y = horizon;
        while y < sh {
            draw_texture_ex(&sp.meadow, x, y, WHITE, DrawTextureParams { dest_size: Some(vec2(tw + 1.0, th + 1.0)), ..Default::default() });
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

pub fn ground(sp: &Sprites, cam: &Cam, font: &Font, towns: &[String]) {
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
    while cam.to_screen(wx, 0.0).x < sw {
        let x = cam.to_screen(wx, 0.0).x;
        draw_texture_ex(&sp.ground, x, top, WHITE, DrawTextureParams { dest_size: Some(vec2(tw + 1.0, th)), ..Default::default() });
        wx += GROUND_TILE_W;
    }
    if top + th < sh {
        draw_rectangle(0.0, top + th - 1.0, sw, sh - (top + th) + 1.0, Color::new(0.20, 0.21, 0.35, 1.0));
    }

    let left = left_world - 8.0;
    let right = cam.x + sw * 0.5 / s + 8.0;
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
    // Distance posts every 25 m.
    let first = (left / 25.0).floor().max(1.0) as i32;
    let last = (right / 25.0).ceil() as i32;
    for k in first..=last {
        let m = k as f32 * 25.0;
        let p = cam.to_screen(m, 0.0);
        let big = k % 4 == 0;
        let w = if big { 3.4 } else { 2.7 } * s;
        let h = w * sp.signpost.height() / sp.signpost.width();
        blit_standing(&sp.signpost, vec2(p.x, p.y + 0.02 * s), w, false, WHITE);
        let scale = (w * 0.30 / 24.0).clamp(0.4, 1.8);
        let label = if big && !towns.is_empty() {
            format!("{} {}", towns[(k / 4).rem_euclid(towns.len() as i32) as usize], m as i32)
        } else {
            format!("{}", m as i32)
        };
        let d = measure_text(&label, Some(font), 24, scale);
        draw_text_ex(
            &label,
            p.x - d.width / 2.0,
            p.y - h * 0.70 + d.height * 0.5,
            TextParams { font: Some(font), font_size: 24, font_scale: scale, color: OUTLINE, ..Default::default() },
        );
    }
}

/// Elves' workshop tower, the ski-jump ramp, Santa and cheering elves.
pub fn start_area(sp: &Sprites, cam: &Cam, font: &Font, label: &str) {
    let s = cam.scale();
    let sw = screen_width();
    let ground_y = cam.to_screen(0.0, 0.0).y;
    let visible = |wx: f32, half_w: f32| {
        let x = cam.to_screen(wx, 0.0).x;
        x > -half_w * s && x < sw + half_w * s
    };
    if !visible(-8.0, 30.0) {
        return;
    }
    let t = get_time() as f32;

    // Tower: its platform top sits at RAMP_TOP.y.
    let tower_h = TOWER_W * sp.tower.height() / sp.tower.width();
    let platform_frac = 128.0 / 800.0;
    let tower_x = RAMP_TOP.x - 1.5;
    let tower_top_y = RAMP_TOP.y + platform_frac * tower_h;
    let tp = cam.to_screen(tower_x, tower_top_y);
    draw_ellipse(tp.x, ground_y + 0.08 * s, TOWER_W * 0.55 * s, 0.3 * s, 0.0, Color::new(0.2, 0.25, 0.5, 0.3));
    draw_texture_ex(
        &sp.tower,
        tp.x - TOWER_W * s / 2.0,
        tp.y,
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(TOWER_W * s, tower_h * s)), ..Default::default() },
    );
    // Sign text.
    let sign = cam.to_screen(tower_x - (44.0 / 320.0) * TOWER_W, tower_top_y - (193.0 / 800.0) * tower_h);
    let board_w = TOWER_W * s * (164.0 / 320.0) * 0.9;
    let base = measure_text(label, Some(font), 20, 1.0);
    let scale = (board_w / base.width).clamp(0.2, 1.4);
    let d = measure_text(label, Some(font), 20, scale);
    draw_text_ex(label, sign.x - d.width / 2.0, sign.y + d.height / 2.0, TextParams { font: Some(font), font_size: 20, font_scale: scale, color: OUTLINE, ..Default::default() });

    // Ramp: struts first, then the deck.
    let mut pts: Vec<Vec2> = (0..=40).map(|i| { let p = ramp_point(i as f32 / 40.0); vec2(p.x, p.y) }).collect();
    // Extend the deck a little past the lip.
    pts.push(vec2(0.9, 9.35));
    for i in (0..pts.len()).step_by(5) {
        let p = pts[i];
        if p.x > -1.0 {
            continue;
        }
        let a = cam.to_screen(p.x, p.y - 0.5);
        let b = cam.to_screen(p.x, 0.0);
        draw_line(a.x, a.y, b.x, b.y, 0.34 * s, Color::new(0.30, 0.18, 0.08, 1.0));
        draw_line(a.x, a.y, b.x, b.y, 0.22 * s, WOOD_DARK);
        draw_line(a.x - 0.06 * s, a.y, b.x - 0.06 * s, b.y, 0.06 * s, Color::new(1.0, 1.0, 1.0, 0.15));
        if i + 5 < pts.len() {
            let q = pts[i + 5];
            let c = cam.to_screen(q.x, (q.y - 0.5) * 0.55);
            let d2 = cam.to_screen(p.x, (p.y - 0.5) * 0.35);
            draw_line(c.x, c.y, d2.x, d2.y, 0.22 * s, WOOD_DARK);
        }
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y);
        let b = cam.to_screen(w[1].x, w[1].y);
        draw_line(a.x, a.y, b.x, b.y, 0.86 * s, Color::new(0.30, 0.18, 0.08, 1.0));
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y);
        let b = cam.to_screen(w[1].x, w[1].y);
        draw_line(a.x, a.y, b.x, b.y, 0.72 * s, WOOD);
        draw_line(a.x, a.y - 0.14 * s, b.x, b.y - 0.14 * s, 0.14 * s, WOOD_LIGHT);
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y + 0.28);
        let b = cam.to_screen(w[1].x, w[1].y + 0.28);
        draw_line(a.x, a.y, b.x, b.y, 0.3 * s, Color::new(0.85, 0.89, 0.98, 1.0));
        draw_line(a.x, a.y - 0.06 * s, b.x, b.y - 0.06 * s, 0.16 * s, SNOW);
    }
    // Candy-cane rail along the ramp.
    for (i, w) in pts.windows(2).enumerate() {
        let a = cam.to_screen(w[0].x, w[0].y + 0.4);
        let b = cam.to_screen(w[1].x, w[1].y + 0.4);
        let c = if i % 2 == 0 { Color::new(0.92, 0.32, 0.32, 1.0) } else { Color::new(1.0, 0.97, 0.95, 1.0) };
        draw_line(a.x, a.y, b.x, b.y, 0.09 * s, c);
    }

    // Santa waving on the platform.
    let sp_pos = cam.to_screen(RAMP_TOP.x - 4.6, RAMP_TOP.y + 0.05);
    let bob = (t * 2.0).sin() * 0.05 * s;
    blit_standing(&sp.santa, vec2(sp_pos.x, sp_pos.y + bob), 2.4 * s, false, WHITE);
    // Elves cheering near the landing zone.
    for (i, (wx, tex)) in [(4.0, &sp.elf_a), (6.5, &sp.elf_b), (-22.0, &sp.elf_b)].iter().enumerate() {
        let p = cam.to_screen(*wx, 0.0);
        let hop = ((t * 5.0 + i as f32 * 1.7).sin().max(0.0)) * 0.3 * s;
        blit_standing(tex, vec2(p.x, p.y + 0.05 * s - hop), 1.5 * s, false, WHITE);
    }
    // A decorated tree and gifts by the tower foot.
    let p = cam.to_screen(-24.0, 0.0);
    blit_standing(&sp.xmas_tree, vec2(p.x, p.y + 0.05 * s), 4.6 * s, false, WHITE);
    let p = cam.to_screen(-20.5, 0.0);
    blit_standing(&sp.gift_pile, vec2(p.x, p.y + 0.05 * s), 2.6 * s, false, WHITE);
    let p = cam.to_screen(9.0, 0.0);
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
    let factor_y = factor * factor * 0.9;
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
        let hop = (t * 9.0 + k as f32).sin().max(0.0) * 0.5 * s * near;
        let tex = if hash(k, 142) > 0.5 { &sp.elf_a } else { &sp.elf_b };
        blit_standing(tex, vec2(p.x, p.y + 0.05 * s - hop), 1.5 * s, hash(k, 143) > 0.5, WHITE);
        if hash(k, 144) > 0.6 {
            let p2 = cam.to_screen(wx + 2.2, 0.0);
            blit_standing(&sp.snowman, vec2(p2.x, p2.y + 0.05 * s - hop * 0.3), 1.9 * s, false, WHITE);
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
    /// Encased in ice after a cold wind.
    pub frozen: bool,
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
        Limbs::Seated => (0.0, -120.0, 0.0),
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
pub fn dog_buried(sp: &Sprites, cam: &Cam, x: f32, idx: usize, sink: f32) {
    let s = cam.scale();
    let r = PILOT_RADIUS * s;
    let w = DOG_W * r * DOG_FLY_SCALE;
    // Hole rim on the snow.
    let g = cam.to_screen(x, 0.0);
    draw_ellipse(g.x, g.y, 1.9 * s, 0.5 * s, 0.0, Color::new(0.2, 0.18, 0.3, 0.9));
    draw_ellipse(g.x, g.y - 0.12 * s, 1.9 * s, 0.42 * s, 0.0, Color::new(0.08, 0.06, 0.14, 1.0));
    // Body upside down; goes down with the sink and stays with the paws up.
    let depth = 2.6 * sink;
    let p = cam.to_screen(x, PILOT_RADIUS + 0.4 - depth);
    let tilt = std::f32::consts::PI;
    let u = w / 256.0;
    let (hx, hy, _) = head_seat(Limbs::Land);
    let head = vec2(p.x + (hx * tilt.cos() + hy * tilt.sin()) * u, p.y + (-hx * tilt.sin() + hy * tilt.cos()) * u);
    ears(sp, idx, head, u, w, tilt, 0.4, WHITE);
    blit_wh(body_tex(sp, idx, Limbs::Land), p, w, w, tilt, false, WHITE);
    blit_wh(head_tex(sp, idx, Face::Squint), head, w, w, tilt, false, WHITE);
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
    let (c, s) = (tilt.cos(), tilt.sin());
    let rel = |x: f32, y: f32| vec2(head.x + (x * c + y * s) * u, head.y + (-x * s + y * c) * u);
    let ew = w * 160.0 / 256.0;
    let left = rel(-60.0, -50.0);
    let right = rel(60.0, -50.0);
    blit_pivot(&sp.dogs_ear[idx.min(2)], left, vec2(0.92, 0.5), ew, tilt + ang, false, tint);
    blit_pivot(&sp.dogs_ear[idx.min(2)], right, vec2(0.92, 0.5), ew, tilt - ang, true, tint);
}

pub fn dog(sp: &Sprites, p: Vec2, r: f32, idx: usize, pose: Pose) {
    let idx = idx.min(2);
    let scale = if pose.flying { DOG_FLY_SCALE } else { 1.0 };
    let w = DOG_W * r * scale;
    let r = r * scale;
    let tilt = pose.tilt + pose.spin;
    let (c, s) = (tilt.cos(), tilt.sin());
    let at = |x: f32, y: f32| vec2(p.x + (x * c - y * s) * r, p.y - (x * s + y * c) * r);

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

    // Each wiggle press hops the whole figure up and slaps it back down.
    let hop = (pose.flap * std::f32::consts::PI).sin();
    let p = vec2(p.x, p.y - hop * r * 0.35);
    // Squash / stretch applies to the whole figure along its heading.
    let k = pose.stretch.clamp(-0.35, 0.35) - if pose.flap > 0.0 && pose.flap < 0.55 { 0.08 } else { 0.0 };
    let sx = 1.0 + k;
    let sy = 1.0 - k;
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
    let (hx2, hy2) = (hx * sx, (hy - bob) * sy);
    let head = vec2(p.x + (hx2 * c + hy2 * s) * u, p.y + (-hx2 * s + hy2 * c) * u);
    let breed_lean = match idx {
        2 => (pose.t * 18.0).sin() * 0.06 * if pose.flying { 1.0 } else { 0.0 },
        _ => 0.0,
    };
    ears(sp, idx, head, u, w, tilt + hrot + breed_lean, ear_angle(&pose), WHITE);
    blit_wh(body_tex(sp, idx, pose.limbs), p, w * sx, w * sy, tilt, false, WHITE);
    blit_wh(head_tex(sp, idx, pose.face), head, w * sx, w * sy, tilt + hrot + breed_lean, false, WHITE);
    if pose.frozen {
        // An ice block around the whole dog, wobbling as it tumbles.
        let centre = (p + head) * 0.5;
        blit(&sp.icecube, centre, w * 1.25, tilt * 0.5 + (pose.t * 3.0).sin() * 0.05, false, Color::new(1.0, 1.0, 1.0, 0.92));
    }
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
    let seed = (e.pos.x * 7.0) as i32;
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
        EntityKind::Firework => {
            let flick = 1.0 + (t * 30.0 + e.pos.x).sin() * 0.25;
            blit(&sp.puff, vec2(p.x - 2.0 * s, p.y), 1.6 * s * flick, 0.0, false, Color::new(1.0, 0.6, 0.2, 0.8 * a));
            blit(&sp.puff, vec2(p.x - 2.4 * s, p.y), 1.0 * s * flick, 0.0, false, Color::new(1.0, 0.9, 0.4, 0.8 * a));
            blit(&sp.puff, p, 5.0 * s, 0.0, false, Color::new(1.0, 0.7, 0.4, 0.2 * a));
            blit(&sp.firework, p, 4.4 * s * grow, (t * 2.0 + e.pos.x).sin() * 0.05, false, tint);
        }
        EntityKind::Present => {
            let bob = (t * 3.0 + e.pos.x).sin() * 0.2 * s;
            let c = vec2(p.x, p.y + bob);
            let glow = (t * 4.0 + e.pos.x).sin() * 0.15 + 0.85;
            blit(&sp.puff, c, 4.0 * s * glow, 0.0, false, Color::new(1.0, 0.9, 0.5, 0.32 * a));
            blit(&sp.sparkle, c, 3.2 * s * glow, t * 0.8, false, Color::new(1.0, 0.95, 0.7, 0.55 * a));
            let tex = &sp.presents[(hash(seed, 81) * 3.0) as usize % 3];
            blit(tex, c, 2.1 * s * grow, (t * 1.3 + e.pos.x).sin() * 0.2, false, tint);
        }
        EntityKind::Sleigh => {
            let bob = (t * 2.5 + e.pos.x).sin() * 0.2 * s;
            blit(&sp.sparkle, vec2(p.x - 2.5 * s, p.y + bob + 0.5 * s), 1.4 * s, t * 2.0, false, Color::new(1.0, 0.95, 0.6, 0.5 * a));
            blit(&sp.sleigh, vec2(p.x, p.y + bob), 6.5 * s * grow, 0.05, false, tint);
        }
        EntityKind::Stormcloud => {
            let bob = (t * 1.5 + e.pos.x).sin() * 0.2 * s;
            let c = vec2(p.x, p.y + bob);
            // Falling snow under it.
            for k in 0..4 {
                let fy = ((t * 1.5 + k as f32 * 0.7 + e.pos.x).rem_euclid(1.0)) * 2.4 * s;
                let fx = (k as f32 - 1.5) * 0.7 * s;
                draw_circle(c.x + fx, c.y + 1.2 * s + fy, 0.09 * s, Color::new(1.0, 1.0, 1.0, 0.7 * a));
            }
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
            let swell = 1.0 + lit * 0.25;
            if lit > 0.0 && e.alive {
                blit(&sp.puff, c, 7.8 * s * swell, 0.0, false, Color::new(1.0, 0.25, 0.25, 0.5 * lit * a));
            }
            blit(&sp.stormcloud, c, 5.6 * s * grow * swell, 0.0, false, lerp(tint, Color::new(1.0, 0.95, 0.92, a), lit * 0.9));
            if pre > 0.0 && e.alive {
                for k in 0..6 {
                    let ang = t * 9.0 + k as f32 * 1.05;
                    let rr = 2.6 * s;
                    let q = c + vec2(ang.cos() * rr, ang.sin() * rr * 0.6);
                    blit(&sp.sparkle, q, 0.8 * s, ang, false, Color::new(1.0, 0.95, 0.5, pre * a));
                }
            }
            if strike && threat > 0.0 && e.alive {
                // A glowing branched bolt aimed at the dog, held for the whole strike.
                let target = dog_at.map(|d| cam.to_screen(d.x, d.y)).unwrap_or(vec2(c.x, c.y + 6.0 * s));
                let dir = (target - c).normalize_or_zero();
                let len = (target - c).length().min(9.0 * s);
                let side = vec2(-dir.y, dir.x);
                let mut pt = c + dir * 1.2 * s;
                let steps = 7;
                let jag = (t * 30.0).floor();
                for k in 0..steps {
                    let f = (k + 1) as f32 / steps as f32;
                    let off = hash(k + jag as i32, 91) - 0.5;
                    let nx = c + dir * (1.2 * s + len * f) + side * off * 1.1 * s;
                    draw_line(pt.x, pt.y, nx.x, nx.y, 0.42 * s, Color::new(0.5, 0.7, 1.0, 0.35 * a));
                    draw_line(pt.x, pt.y, nx.x, nx.y, 0.22 * s, OUTLINE);
                    draw_line(pt.x, pt.y, nx.x, nx.y, 0.11 * s, Color::new(1.0, 0.98, 0.7, a));
                    if k == 2 || k == 4 {
                        let b = nx + (dir * 0.5 + side * (if k == 2 { 0.8 } else { -0.8 })) * 1.3 * s;
                        draw_line(nx.x, nx.y, b.x, b.y, 0.14 * s, OUTLINE);
                        draw_line(nx.x, nx.y, b.x, b.y, 0.07 * s, Color::new(1.0, 0.98, 0.7, a));
                    }
                    pt = nx;
                }
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
pub fn dog_paws_up(sp: &Sprites, cam: &Cam, x: f32, idx: usize) {
    let s = cam.scale();
    let r = PILOT_RADIUS * s;
    let w = DOG_W * r * DOG_FLY_SCALE;
    let p = cam.to_screen(x, PILOT_RADIUS + 0.4 - 2.6);
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
        g.y - dh + 0.2 * s,
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(w, dh)), source: Some(src), flip_y: true, ..Default::default() },
    );
}
