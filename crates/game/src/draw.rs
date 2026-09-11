//! Scene drawing with sprites: sky, parallax layers, ground dressing,
//! the workshop tower and ramp, the dog, pickups.

use hamsters_core::{ramp_point, Entity, EntityKind, LIP, PILOT_RADIUS, RAMP_TOP, ROOF_Y, SPACE_ALTITUDE};
use macroquad::prelude::*;

use crate::sprites::{blit, blit_pivot, blit_standing, blit_wh, Sprites};
use crate::Cam;

pub const SEED: Color = Color::new(1.0, 0.86, 0.30, 1.0);
pub const OUTLINE: Color = Color::new(0.12, 0.14, 0.25, 1.0);
const WOOD: Color = Color::new(0.70, 0.46, 0.25, 1.0);
const WOOD_LIGHT: Color = Color::new(0.82, 0.60, 0.38, 1.0);
const SNOW: Color = Color::new(0.97, 0.98, 1.0, 1.0);

/// Fur mid-tone per dog, used to tint the greyscale paw sprite.

// World sizes of sprites, in meters (width).
const DOG_W: f32 = 3.3;
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

const HAZE: Color = Color::new(0.74, 0.82, 0.97, 1.0);

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let k = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// Tint for everything lit by the sky: white at noon, cool violet at dusk.
pub fn daylight(cam: &Cam) -> Color {
    lerp(WHITE, Color::new(0.72, 0.68, 0.85, 1.0), dusk_factor(cam))
}

fn mul(a: Color, b: Color) -> Color {
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
    let day_top = lerp(Color::new(0.24, 0.62, 0.95, 1.0), Color::new(0.12, 0.20, 0.48, 1.0), dusk);
    let day_top = lerp(day_top, Color::new(0.10, 0.38, 0.85, 1.0), alt);
    let day_mid = lerp(Color::new(0.55, 0.82, 0.98, 1.0), Color::new(0.55, 0.45, 0.70, 1.0), smoothstep(0.3, 1.0, dusk));
    let day_mid = lerp(day_mid, Color::new(0.34, 0.66, 0.96, 1.0), alt);
    let peach = lerp(Color::new(0.88, 0.96, 1.0, 1.0), Color::new(1.0, 0.72, 0.47, 1.0), smoothstep(0.15, 0.6, dusk));
    let day_low = lerp(peach, Color::new(0.30, 0.26, 0.50, 1.0), smoothstep(0.6, 1.0, dusk));
    let top = lerp(day_top, Color::new(0.01, 0.01, 0.05, 1.0), space);
    let mid = lerp(day_mid, Color::new(0.03, 0.04, 0.12, 1.0), space);
    let low = lerp(day_low, Color::new(0.10, 0.14, 0.32, 1.0), space);
    let strips = 72;
    let h = sh / strips as f32;
    for i in 0..strips {
        let k = i as f32 / (strips - 1) as f32;
        let c = if k < 0.5 { lerp(top, mid, (k / 0.5).powf(1.2)) } else { lerp(mid, low, ((k - 0.5) / 0.5).powf(1.6)) };
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
        blit(&sp.sun, vec2(mx, my), 230.0, t * 0.05, false, Color::new(warm.r, warm.g, warm.b, 1.0 - space));
    }
    if space > 0.0 {
        blit(&sp.moon, vec2(mx - 200.0, my + 40.0), 200.0, 0.0, false, Color::new(1.0, 1.0, 1.0, space));
    }
    // Aurora is the reward of dusk: faint at noon, full in the evening, gone in space.
    aurora(cam, t, (1.0 - space) * (0.35 + 0.65 * dusk));
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
    let tint = lerp(mul(lerp(Color::new(0.62, 0.66, 0.92, 1.0), WHITE, (factor / 0.8).clamp(0.0, 1.0)), light), mul(HAZE, light), haze);
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
    let horizon = cam.to_screen_parallax(0.0, 0.0, para_y(0.8)).y;
    let top = horizon - 105.0 * cam.zoom.powf(0.4);
    if top < sh {
        draw_rectangle(0.0, top.max(0.0), sw, (horizon - top.max(0.0)).max(0.0), Color::new(0.35, 0.38, 0.62, 0.40));
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
    // The snow takes the evening light too, a little less than the far layers.
    let light = lerp(WHITE, daylight(cam), 0.7);
    while cam.to_screen(wx, 0.0).x < sw {
        let x = cam.to_screen(wx, 0.0).x;
        draw_texture_ex(&sp.ground, x, top, light, DrawTextureParams { dest_size: Some(vec2(tw + 1.0, th)), ..Default::default() });
        wx += GROUND_TILE_W;
    }
    if top + th < sh {
        draw_rectangle(0.0, top + th - 1.0, sw, sh - (top + th) + 1.0, Color::new(0.20, 0.21, 0.35, 1.0));
    }

    let left = left_world - 8.0;
    let right = cam.x + sw * 0.5 / s + 8.0;
    surfaces(cam, left, right);
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
    // No distance posts: the HUD carries the number and the pennants mark history.
    let _ = (font, towns);
}

/// The Falkbuilt factory with the ski jump on its roof, Santa and cheering elves.
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
        draw_line(a.x, a.y, b.x, b.y, (w + 0.16) * s, OUTLINE);
        draw_line(a.x, a.y, b.x, b.y, w * s, WOOD);
        draw_line(a.x - 0.1 * w * s, a.y, b.x - 0.1 * w * s, b.y, 0.25 * w * s, WOOD_LIGHT);
    };
    let beam = |x0: f32, y0: f32, x1: f32, y1: f32, w: f32| {
        let a = cam.to_screen(x0, y0);
        let b = cam.to_screen(x1, y1);
        draw_line(a.x, a.y, b.x, b.y, (w + 0.14) * s, OUTLINE);
        draw_line(a.x, a.y, b.x, b.y, w * s, WOOD);
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
    // Struts from the deck to the roof; the long ones get a knee brace.
    for x in [-32.5, -27.0, -19.0, -13.5, -8.0, -3.2] {
        let dy = deck_y(x);
        if dy - ROOF_Y > 0.9 {
            post(x, dy - 0.3, ROOF_Y - 0.3, 0.34);
            if dy - ROOF_Y > 4.0 {
                beam(x, ROOF_Y + 2.4, x - 2.2, dy - 0.9, 0.16);
                beam(x, ROOF_Y + 2.4, x + 2.2, dy - 0.9 - 1.1, 0.16);
            }
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
    // The deck: ink underside, wood, a light edge, then snow with two tracks.
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y);
        let b = cam.to_screen(w[1].x, w[1].y);
        draw_line(a.x, a.y, b.x, b.y, 0.9 * s, OUTLINE);
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y);
        let b = cam.to_screen(w[1].x, w[1].y);
        draw_line(a.x, a.y, b.x, b.y, 0.74 * s, WOOD);
        draw_line(a.x, a.y - 0.14 * s, b.x, b.y - 0.14 * s, 0.14 * s, WOOD_LIGHT);
    }
    for w in pts.windows(2) {
        let a = cam.to_screen(w[0].x, w[0].y + 0.3);
        let b = cam.to_screen(w[1].x, w[1].y + 0.3);
        draw_line(a.x, a.y, b.x, b.y, 0.34 * s, Color::new(0.85, 0.89, 0.98, 1.0));
        draw_line(a.x, a.y - 0.06 * s, b.x, b.y - 0.06 * s, 0.2 * s, SNOW);
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
            draw_line(f.x, f.y, a.x, a.y, 0.06 * s, OUTLINE);
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
    let bob = (t * 2.0).sin() * 0.05 * s;
    blit_standing(&sp.santa, vec2(sp_pos.x, sp_pos.y + bob), 2.4 * s, false, WHITE);
    // Elves cheering near the landing zone.
    for (i, (wx, tex)) in [(3.0, &sp.elf_a), (5.5, &sp.elf_b), (14.5, &sp.elf_b)].iter().enumerate() {
        let p = cam.to_screen(*wx, 0.0);
        let hop = ((t * 5.0 + i as f32 * 1.7).sin().max(0.0)) * 0.3 * s;
        blit_standing(tex, vec2(p.x, p.y + 0.05 * s - hop), 1.5 * s, false, WHITE);
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
        let h = 2.4 * s;
        draw_rectangle(p.x - 0.07 * s, p.y - h, 0.14 * s, h, OUTLINE);
        let wave = (t * 3.0).sin() * 0.12 * s;
        // Checkered flag: a 2x2 gold/ink grid, waving at the tip.
        let (fw, fh) = (1.5 * s, 0.9 * s);
        let (x0, y0) = (p.x + 0.07 * s, p.y - h);
        for cy in 0..2 {
            for cx in 0..2 {
                let gold = (cx + cy) % 2 == 0;
                let c = if gold { SEED } else { OUTLINE };
                let (ax, ay) = (x0 + fw * 0.5 * cx as f32, y0 + fh * 0.5 * cy as f32);
                let tip = if cx == 1 { wave } else { 0.0 };
                draw_triangle(vec2(ax, ay + tip * 0.5), vec2(ax + fw * 0.5, ay + tip), vec2(ax, ay + fh * 0.5 + tip * 0.5), c);
                draw_triangle(vec2(ax + fw * 0.5, ay + tip), vec2(ax + fw * 0.5, ay + fh * 0.5 + tip), vec2(ax, ay + fh * 0.5 + tip * 0.5), c);
            }
        }
        draw_rectangle_lines(x0, y0, fw, fh, 0.06 * s, OUTLINE);
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
    let squash = 1.0 - 0.28 * (1.0 - stuck);
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

    if pose.ball > 0.0 {
        // The meteor: a fireball around the curled dog, flickering, with a
        // tail of flame streaming up behind the fall. Sized from the figure
        // width so it wraps the whole dog.
        let b = pose.ball;
        let flick = 1.0 + (pose.t * 37.0).sin() * 0.12 + (pose.t * 23.0).cos() * 0.08;
        for i in 0..6 {
            let k = i as f32 / 5.0;
            let tail = vec2(p.x + (pose.t * 13.0 + i as f32).sin() * 0.12 * w, p.y - (0.45 + 1.4 * k) * w * b);
            let col = lerp(Color::new(1.0, 0.45, 0.10, 0.9 * b), Color::new(1.0, 0.85, 0.35, 0.0), k);
            blit(&sp.puff, tail, (1.3 - 0.8 * k) * w * b * flick, k * 3.0, i % 2 == 0, col);
        }
        blit(&sp.puff, p, 1.7 * w * b * flick, pose.t * 2.0, false, Color::new(1.0, 0.40, 0.08, 0.85 * b));
        blit(&sp.puff, p, 1.3 * w * b * flick, -pose.t * 3.0, true, Color::new(1.0, 0.72, 0.20, 0.9 * b));
        blit(&sp.puff, p, 0.95 * w * b, pose.t * 4.0, false, Color::new(1.0, 0.95, 0.65, 0.9 * b));
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
    // Frozen: the dog itself goes icy blue and shivers, with icicles hanging
    // off it and frost glinting around; no block hides the character.
    let tint = if pose.frozen { Color::new(0.70, 0.86, 1.0, 1.0) } else { WHITE };
    let shiver = if pose.frozen { vec2((pose.t * 57.0).sin(), (pose.t * 43.0).cos()) * 0.018 * w } else { Vec2::ZERO };
    let (p, head) = (p + shiver, head + shiver);
    cape(p, head, w, tilt, &pose, idx);
    ears(sp, idx, head, u, w, tilt + hrot + breed_lean, ear_angle(&pose), tint);
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
        let b = pose.ball;
        let flick = 1.0 + (pose.t * 41.0).sin() * 0.1;
        blit(&sp.puff, p, 1.35 * w / (1.0 - 0.45 * b) * b * flick, -pose.t * 2.5, true, Color::new(1.0, 0.55, 0.12, 0.45 * b));
        blit(&sp.puff, p, 0.9 * w / (1.0 - 0.45 * b) * b, pose.t * 3.5, false, Color::new(1.0, 0.9, 0.5, 0.35 * b));
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
    let fill = scarf_color(idx);
    let dark = Color::new(fill.r * 0.72, fill.g * 0.6, fill.b * 0.6, 1.0);
    // Ink outline as a slightly inflated copy, then the fill, then a shaded inner fold.
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let grow = |q: Vec2| c + (q - c) * 1.0 + (q - c).normalize_or_zero() * 2.5;
        draw_triangle(c, grow(a), grow(b), OUTLINE);
    }
    for i in 0..pts.len() {
        draw_triangle(c, pts[i], pts[(i + 1) % pts.len()], fill);
    }
    // A darker fold down the middle gives the cape volume.
    let mid0 = collar + side * 0.02 * w;
    let mid1 = collar + stream * len + side * (0.05 * w + (pose.t * flap_rate).sin() * 0.05 * w);
    draw_triangle(mid0, mid1 + side * 0.09 * w, mid1 - side * 0.05 * w, dark);
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
            // The rocket hangs nose-up-right with its fuse lit: a warm glow, a
            // sputtering spark at the tail and a thin smoke trail drifting down.
            let flick = 1.0 + (t * 30.0 + e.pos.x).sin() * 0.25;
            let rot = -0.35 + (t * 2.0 + e.pos.x).sin() * 0.05;
            let tail = vec2(p.x - 2.1 * s, p.y + 0.9 * s);
            for k in 0..3 {
                let kf = k as f32;
                let drift = ((t * 0.8 + kf * 0.9 + e.pos.x).rem_euclid(1.0)) * 1.8 * s;
                blit(&sp.puff, vec2(tail.x - 0.3 * s * kf, tail.y + drift), (0.6 + 0.3 * kf) * s, 0.0, false, Color::new(0.85, 0.85, 0.9, 0.25 * a * (1.0 - drift / (1.8 * s))));
            }
            blit(&sp.puff, p, 5.6 * s, 0.0, false, Color::new(1.0, 0.7, 0.4, 0.22 * a));
            blit(&sp.firework, p, 4.6 * s * grow, rot, false, tint);
            blit(&sp.sparkle, tail, 1.5 * s * flick, t * 9.0, false, Color::new(1.0, 0.95, 0.6, 0.95 * a));
            blit(&sp.sparkle, tail, 0.8 * s * flick, -t * 12.0, false, Color::new(1.0, 1.0, 1.0, 0.9 * a));
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
