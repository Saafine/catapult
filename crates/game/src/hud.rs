//! Screen-space UI: snow-capped wooden panels, meters, pilot picker, results.

use hamsters_core::{Game, Phase, PILOTS};
use macroquad::prelude::*;

use crate::draw::lerp;
use crate::sprites::blit;
use crate::{draw, App};

const GLASS: Color = Color::new(0.06, 0.08, 0.18, 0.62);
const GLASS_EDGE: Color = Color::new(1.0, 1.0, 1.0, 0.22);
const GOLD: Color = Color::new(0.96, 0.78, 0.36, 1.0);
const CREAM: Color = Color::new(0.98, 0.98, 1.0, 1.0);
const DIM: Color = Color::new(0.80, 0.84, 0.95, 0.9);

#[derive(Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

fn aligned_x(font: &Font, s: &str, x: f32, size: u16, scale: f32, align: Align) -> f32 {
    let d = measure_text(s, Some(font), size, scale);
    match align {
        Align::Left => x,
        Align::Center => x - d.width / 2.0,
        Align::Right => x - d.width,
    }
}

pub fn text(font: &Font, s: &str, x: f32, y: f32, size: u16, color: Color, align: Align) {
    let x = aligned_x(font, s, x, size, 1.0, align);
    draw_text_ex(s, x, y, TextParams { font: Some(font), font_size: size, color, ..Default::default() });
}

pub fn text_outlined(font: &Font, s: &str, x: f32, y: f32, size: u16, color: Color, align: Align) {
    text_outlined_scaled(font, s, x, y, size, 1.0, color, align);
}

/// Outlined text with a runtime scale; keeps the glyph atlas stable by
/// rasterizing at one size and scaling the quads.
pub fn text_outlined_scaled(font: &Font, s: &str, x: f32, y: f32, size: u16, scale: f32, color: Color, align: Align) {
    let x = aligned_x(font, s, x, size, scale, align);
    // Soft shadow instead of a hard outline.
    let o = Color::new(0.03, 0.04, 0.12, 0.55 * color.a.min(1.0));
    let t = 1.6 * scale;
    for (dx, dy) in [(t, t), (t * 1.6, t * 1.6), (0.0, t)] {
        draw_text_ex(s, x + dx, y + dy, TextParams { font: Some(font), font_size: size, font_scale: scale, color: o, ..Default::default() });
    }
    let mut fill = color;
    fill.a = fill.a.min(1.0);
    draw_text_ex(s, x, y, TextParams { font: Some(font), font_size: size, font_scale: scale, color: fill, ..Default::default() });
}

/// Rounded rectangle as one triangle fan.
pub fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    let centers = [(x + w - r, y + r, -90.0f32), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, 90.0), (x + r, y + r, 180.0)];
    let mut pts: Vec<Vec2> = Vec::with_capacity(28);
    for (cx, cy, start) in centers {
        for i in 0..=6 {
            let a = (start + i as f32 * 15.0).to_radians();
            pts.push(vec2(cx + a.cos() * r, cy + a.sin() * r));
        }
    }
    let c = vec2(x + w / 2.0, y + h / 2.0);
    for i in 0..pts.len() {
        draw_triangle(c, pts[i], pts[(i + 1) % pts.len()], color);
    }
}

/// Frosted glass panel: dark translucent fill, a fine light edge, a gold hairline.
pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    panel_alpha(x, y, w, h, 1.0)
}

/// A panel that can go translucent so nothing in play is ever hidden.
pub fn panel_alpha(x: f32, y: f32, w: f32, h: f32, alpha: f32) {
    rounded(x + 2.0, y + 3.0, w, h, 12.0, Color::new(0.0, 0.0, 0.05, 0.25 * alpha));
    rounded(x, y, w, h, 12.0, Color::new(GLASS.r, GLASS.g, GLASS.b, GLASS.a * alpha));
    rounded(x + 1.0, y + 1.0, w - 2.0, 1.5, 1.0, GLASS_EDGE);
    rounded(x + 1.0, y + 1.0, 1.5, h - 2.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.12));
    rounded(x + 12.0, y + h - 4.0, w - 24.0, 2.0, 1.0, Color::new(0.96, 0.78, 0.36, 0.7));
}

fn picker_cards() -> [Rect; 3] {
    let w = 200.0;
    let h = 56.0;
    let gap = 10.0;
    let x = screen_width() - w - 12.0;
    let y0 = 104.0;
    [0, 1, 2].map(|i| Rect::new(x, y0 + i as f32 * (h + gap), w, h))
}

pub fn picker_hit(p: Vec2) -> Option<usize> {
    picker_cards().iter().position(|r| r.contains(p))
}

pub fn render(app: &App) {
    let g = &app.game;
    let font = &app.font;
    let big = &app.display;
    let sw = screen_width();
    let sh = screen_height();

    let aiming = matches!(g.phase, Phase::AimPower { .. } | Phase::Sliding { .. });
    if !aiming && g.phase != Phase::Landed {
        let flying = g.phase == Phase::Flying;
        let pa = if flying { 0.55 } else { 1.0 };
        panel_alpha(12.0, 12.0, 240.0, if flying { 74.0 } else { 50.0 }, pa);
        let dp = 1.0 + 0.18 * (app.dist_pulse * std::f32::consts::PI).sin().max(0.0);
        text_outlined_scaled(big, &format!("{:.1} m", g.distance()), 24.0, 48.0, 34, dp, CREAM, Align::Left);
        if g.best_distance > 0.0 {
            text(font, &format!("best {:.1}", g.best_distance), 240.0, 45.0, 15, DIM, Align::Right);
        }
        if flying {
            // Wiggle stamina lives in this plank so nothing on the ground is hidden.
            text_outlined(big, &app.pack.labels.flap, 24.0, 74.0, 13, CREAM, Align::Left);
            let x = 92.0;
            let y = 60.0;
            let w = 148.0;
            let left = ((g.stamina / hamsters_core::FLAP_COST).floor() as i32).clamp(0, 12);
            text(font, &format!("{}/12", left), x + w + 8.0, 74.0, 13, if left < 3 { Color::new(1.0, 0.4, 0.4, 1.0) } else { DIM }, Align::Left);
            rounded(x, y, w, 16.0, 6.0, Color::new(0.02, 0.03, 0.10, 0.7));
            let col = if g.stamina < hamsters_core::FLAP_COST { Color::new(0.9, 0.3, 0.3, 1.0) } else { Color::new(0.45, 0.9, 0.6, 1.0) };
            let low = left < 3;
            let col = if low { lerp(col, Color::new(1.0, 0.25, 0.25, 1.0), 0.5 + 0.5 * (get_time() as f32 * 9.0).sin()) } else { col };
            if g.stamina > 0.02 {
                rounded(x, y, (w * g.stamina).max(10.0), 16.0, 6.0, col);
            }
            // Twelve notches: one wiggle press each, so the cost is countable.
            for i in 1..12 {
                let nx = x + w * i as f32 / 12.0;
                draw_rectangle(nx - 0.75, y + 1.0, 1.5, 14.0, Color::new(0.02, 0.03, 0.10, 0.75));
            }
            let streak = g.flaps % hamsters_core::WIGGLE_STREAK;
            let n = hamsters_core::WIGGLE_STREAK as usize;
            let tw = (w - (n as f32 - 1.0) * 1.5) / n as f32;
            for i in 0..n {
                let c = if (i as u32) < streak { draw::SEED } else { Color::new(1.0, 1.0, 1.0, 0.22) };
                draw_rectangle(x + i as f32 * (tw + 1.5), y + 18.0, tw, 3.0, c);
            }
        }

        if g.phase == Phase::Flying {
            panel_alpha(sw - 172.0, 12.0, 160.0, 50.0, pa);
            text_outlined(big, &format!("{:.0} m", g.altitude()), sw - 86.0, 48.0, 28, CREAM, Align::Right);
            text(font, "HEIGHT", sw - 80.0, 46.0, 12, DIM, Align::Left);
        }

        let pulse = 1.0 + 0.5 * (app.counter_pulse * std::f32::consts::PI).sin().max(0.0);
        panel_alpha(sw / 2.0 - 66.0, 12.0, 132.0, 50.0, pa);
        blit(&app.sprites.presents[0], vec2(sw / 2.0 - 36.0, 37.0), 30.0 * pulse, 0.1, false, WHITE);
        let combo = if app.counter_pulse > 0.0 && app.combo_now() > 1 { format!("{}  x{}", g.presents, app.combo_now()) } else { format!("{}", g.presents) };
        text_outlined_scaled(big, &combo, sw / 2.0 - 14.0, 47.0, 28, pulse, if app.counter_pulse > 0.0 { GOLD } else { CREAM }, Align::Left);
    }

    match g.phase {
        Phase::AimPower { meter } => {
            picker(app);
            power_dial(app, sw / 2.0, sh - 44.0, Game::meter_value(meter), true);
            text_outlined(font, "space or click to lock the push", sw / 2.0, sh - 12.0, 16, CREAM, Align::Center);
        }
        Phase::Sliding { power, t, jump } => {
            power_dial(app, sw / 2.0, sh - 44.0, power, false);
            if jump.is_none() {
                jump_timer(app, t);
            }
        }
        Phase::Flying => {
            // Active boosts: small icon plus a draining bar, no words.
            let mut by = sh - 36.0;
            if g.glide > 0.0 {
                blit(&app.sprites.wings, vec2(40.0, by), 40.0, 0.0, false, WHITE);
                let frac = (g.glide / (3.5 * g.pilot.glide_bonus)).clamp(0.0, 1.0);
                rounded(64.0, by - 5.0, 120.0, 10.0, 4.0, Color::new(0.15, 0.08, 0.10, 0.7));
                rounded(64.0, by - 5.0, (120.0 * frac).max(8.0), 10.0, 4.0, Color::new(0.75, 0.9, 1.0, 1.0));
                by -= 30.0;
            }
            if g.rocket > 0.0 {
                blit(&app.sprites.firework, vec2(40.0, by), 40.0, 0.0, false, WHITE);
                let frac = (g.rocket / 1.2).clamp(0.0, 1.0);
                rounded(64.0, by - 5.0, 120.0, 10.0, 4.0, Color::new(0.15, 0.08, 0.10, 0.7));
                rounded(64.0, by - 5.0, (120.0 * frac).max(8.0), 10.0, 4.0, Color::new(1.0, 0.72, 0.35, 1.0));
            }
        }
        Phase::Landed => {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.03, 0.02, 0.08, 0.5));
            let r = results_rect();
            let (x, y, w, h) = (r.x, r.y, r.w, r.h);
            let new_best = g.distance() >= g.best_distance - 0.01 && g.distance() > 0.0;
            let t = get_time() as f32;
            if new_best {
                for i in 0..16 {
                    let a = i as f32 * std::f32::consts::TAU / 16.0 + t * 0.3;
                    let a2 = a + 0.09;
                    let c = vec2(sw / 2.0, y + h / 2.0);
                    let rr = 460.0;
                    draw_triangle(c, c + vec2(a.cos(), a.sin()) * rr, c + vec2(a2.cos(), a2.sin()) * rr, Color::new(1.0, 0.85, 0.3, 0.09));
                }
            }
            panel(x, y, w, h);
            rounded(x + w / 2.0 - 106.0, y - 14.0, 212.0, 28.0, 8.0, Color::new(0.80, 0.18, 0.22, 1.0));
            rounded(x + w / 2.0 - 104.0, y - 12.0, 208.0, 1.5, 1.0, Color::new(1.0, 1.0, 1.0, 0.35));
            text_outlined(big, if g.crashed { &app.pack.labels.crash } else { &app.pack.labels.flight_over }, x + w / 2.0, y + 8.0, 22, CREAM, Align::Center);
            // Left: the pilot and this run's distance, big.
            let portrait = if new_best { &app.sprites.dogs_heads[app.pilot_idx][6] } else { &app.sprites.dogs[app.pilot_idx] };
            let lx = x + 150.0;
            blit(portrait, vec2(lx, y + 96.0), 116.0, 0.06, false, WHITE);
            let tick = ((get_time() - app.landed_at) / 1.1).clamp(0.0, 1.0) as f32;
            let ease = 1.0 - (1.0 - tick).powi(3);
            text_outlined(big, &format!("{:.1} m", g.distance() * ease), lx, y + 208.0, 64, CREAM, Align::Center);
            if new_best && tick >= 1.0 {
                let pulse = 1.0 + (t * 6.0).sin() * 0.06;
                text_outlined_scaled(big, "NEW BEST!", lx, y + 244.0, 28, pulse, draw::SEED, Align::Center);
            } else if tick >= 1.0 {
                let short = app.best_before - g.distance();
                text(font, &format!("{:.1} m short of your best", short.max(0.0)), lx, y + 240.0, 15, Color::new(1.0, 0.85, 0.5, 1.0), Align::Center);
            }
            // Right: the scoreboard, distance only.
            let sx = x + 316.0;
            let sw_ = w - 340.0;
            text_outlined(big, &app.pack.labels.scoreboard, sx + sw_ / 2.0, y + 44.0, 18, GOLD, Align::Center);
            rounded(sx, y + 54.0, sw_, 1.5, 1.0, Color::new(1.0, 0.86, 0.3, 0.5));
            let rows = app.scores.iter().take(6);
            let mut ry = y + 80.0;
            let mut highlighted = false;
            for (i, sc) in rows.enumerate() {
                let mine = !highlighted && (sc.distance - g.distance()).abs() < 0.01 && sc.pilot == app.pilot_idx;
                if mine {
                    highlighted = true;
                    rounded(sx - 6.0, ry - 17.0, sw_ + 12.0, 26.0, 6.0, Color::new(1.0, 0.86, 0.3, 0.16));
                }
                let col = if mine { GOLD } else if i == 0 { CREAM } else { DIM };
                text_outlined(big, &format!("{}.", i + 1), sx + 4.0, ry, 15, col, Align::Left);
                blit(&app.sprites.dogs[sc.pilot.min(2)], vec2(sx + 44.0, ry - 5.0), 22.0, 0.0, false, WHITE);
                let pname = app.pack.pilots.get(sc.pilot).map(|p| p.name.as_str()).unwrap_or("?");
                text(font, pname, sx + 62.0, ry, 14, col, Align::Left);
                text_outlined(big, &format!("{:.1} m", sc.distance), sx + sw_ - 4.0, ry, 16, col, Align::Right);
                if sc.crashed {
                    text(font, "splat", sx + sw_ - 78.0, ry, 11, Color::new(1.0, 0.5, 0.5, 0.9), Align::Right);
                }
                ry += 26.0;
            }
            if app.scores.is_empty() {
                text(font, "no flights yet", sx + sw_ / 2.0, y + 100.0, 14, DIM, Align::Center);
            }
            // Restart button, pulsing once the numbers have settled.
            let b = restart_button_rect();
            let armed = get_time() - app.landed_at > 0.5;
            let pulse = if armed { 1.0 + 0.04 * (t * 5.0).sin() } else { 1.0 };
            let (bw, bh) = (b.w * pulse, b.h * pulse);
            let (bx, by) = (b.x + (b.w - bw) / 2.0, b.y + (b.h - bh) / 2.0);
            rounded(bx + 2.0, by + 4.0, bw, bh, 14.0, Color::new(0.0, 0.0, 0.05, 0.35));
            rounded(bx, by, bw, bh, 14.0, if armed { Color::new(0.16, 0.66, 0.36, 1.0) } else { Color::new(0.25, 0.3, 0.4, 1.0) });
            rounded(bx + 3.0, by + 3.0, bw - 6.0, bh * 0.45, 10.0, Color::new(1.0, 1.0, 1.0, 0.18));
            text_outlined_scaled(big, &app.pack.labels.restart, bx + bw / 2.0, by + bh / 2.0 + 9.0, 24, 1.0, WHITE, Align::Center);
            text(font, "or press R", b.x + b.w / 2.0, b.y + b.h + 18.0, 12, DIM, Align::Center);
        }
    }
}

/// The results card, centred.
fn results_rect() -> Rect {
    let (sw, sh) = (screen_width(), screen_height());
    let (w, h) = (640.0, 330.0);
    Rect::new((sw - w) / 2.0, (sh - h) / 2.0 - 10.0, w, h)
}

/// Where the Restart button sits on the results card (for hit-testing).
pub fn restart_button_rect() -> Rect {
    let r = results_rect();
    let (w, h) = (220.0, 52.0);
    Rect::new(r.x + r.w / 2.0 - w / 2.0, r.y + r.h - h - 22.0, w, h)
}

/// A big semicircular power gauge: colour zones, tick marks, a gold sweet
/// spot and a swinging needle. `live` while the meter is still bouncing.
fn power_dial(app: &App, cx: f32, cy: f32, value: f32, live: bool) {
    let big = &app.display;
    let font = &app.font;
    let r = 96.0;
    let t = get_time() as f32;
    // Half-disc plate with a flat base just under the hub.
    let plate = |rad: f32, col: Color, dy: f32| {
        let n = 40;
        for i in 0..n {
            let a0 = std::f32::consts::PI * i as f32 / n as f32;
            let a1 = std::f32::consts::PI * (i + 1) as f32 / n as f32;
            draw_triangle(vec2(cx, cy + dy), vec2(cx - a0.cos() * rad, cy + dy - a0.sin() * rad), vec2(cx - a1.cos() * rad, cy + dy - a1.sin() * rad), col);
        }
        draw_rectangle(cx - rad, cy + dy, 2.0 * rad, 18.0, col);
    };
    plate(r + 24.0, Color::new(0.0, 0.0, 0.05, 0.3), 5.0);
    plate(r + 20.0, Color::new(0.06, 0.08, 0.18, 0.92), 0.0);
    draw_line(cx - r - 20.0, cy + 18.0, cx + r + 20.0, cy + 18.0, 2.0, Color::new(1.0, 0.86, 0.3, 0.5));
    // Zones from left (0) to right (1) along the upper half.
    let zone = |a0: f32, a1: f32, col: Color, rad: f32, wd: f32| {
        let n = 24;
        for i in 0..n {
            let k0 = a0 + (a1 - a0) * i as f32 / n as f32;
            let k1 = a0 + (a1 - a0) * (i + 1) as f32 / n as f32;
            let (p0, p1) = (std::f32::consts::PI * (1.0 - k0), std::f32::consts::PI * (1.0 - k1));
            let (o0, o1) = (vec2(cx + p0.cos() * rad, cy - p0.sin() * rad), vec2(cx + p1.cos() * rad, cy - p1.sin() * rad));
            let (i0, i1) = (vec2(cx + p0.cos() * (rad - wd), cy - p0.sin() * (rad - wd)), vec2(cx + p1.cos() * (rad - wd), cy - p1.sin() * (rad - wd)));
            draw_triangle(o0, o1, i0, col);
            draw_triangle(o1, i1, i0, col);
        }
    };
    zone(0.0, 1.0, Color::new(0.02, 0.03, 0.10, 0.9), r, 26.0);
    zone(0.0, 0.5, Color::new(0.35, 0.75, 0.5, 0.9), r - 3.0, 20.0);
    zone(0.5, 0.82, Color::new(0.95, 0.75, 0.3, 0.9), r - 3.0, 20.0);
    let glow = 0.75 + 0.25 * (t * 6.0).sin();
    zone(0.82, 1.0, Color::new(1.0, 0.86 * glow, 0.2, 1.0), r - 3.0, 20.0);
    zone(0.82, 1.0, Color::new(1.0, 1.0, 1.0, 0.25 * glow), r + 2.0, 6.0);
    // Ticks.
    for i in 0..=10 {
        let k = i as f32 / 10.0;
        let a = std::f32::consts::PI * (1.0 - k);
        let (c, sn) = (a.cos(), a.sin());
        let len = if i % 5 == 0 { 14.0 } else { 7.0 };
        draw_line(cx + c * (r - 30.0), cy - sn * (r - 30.0), cx + c * (r - 30.0 - len), cy - sn * (r - 30.0 - len), 2.5, Color::new(1.0, 1.0, 1.0, 0.7));
    }
    // Needle with a little overshoot wobble while live.
    let wob = if live { (t * 40.0).sin() * 0.006 } else { 0.0 };
    let a = std::f32::consts::PI * (1.0 - (value + wob).clamp(0.0, 1.0));
    let tip = vec2(cx + a.cos() * (r - 8.0), cy - a.sin() * (r - 8.0));
    let side = vec2(-a.sin(), -a.cos()) * 5.0;
    draw_triangle(vec2(cx, cy) + side, vec2(cx, cy) - side, tip, Color::new(0.03, 0.04, 0.12, 0.9));
    draw_triangle(vec2(cx, cy) + side * 0.6, vec2(cx, cy) - side * 0.6, tip, if value > 0.82 { GOLD } else { CREAM });
    draw_circle(cx, cy, 12.0, Color::new(0.03, 0.04, 0.12, 1.0));
    draw_circle(cx, cy, 8.0, CREAM);
    // Readout.
    let sweet = value > 0.82;
    let pulse = if sweet { 1.0 + 0.08 * (t * 10.0).sin() } else { 1.0 };
    text_outlined_scaled(big, &format!("{:.0}", value * 100.0), cx, cy - 34.0, 40, pulse, if sweet { GOLD } else { CREAM }, Align::Center);
    text(font, if sweet { "SWEET SPOT" } else { "POWER" }, cx, cy - 8.0, 13, if sweet { GOLD } else { DIM }, Align::Center);
    let _ = live;
}

/// The ramp ride as a timing challenge: a bar from the top of the ramp to
/// the lip with a gold JUMP zone at the end, a marker for where the sled is,
/// and a glowing stretch of the real ramp so the eye stays on the sled.
fn jump_timer(app: &App, t: f32) {
    let big = &app.display;
    let font = &app.font;
    let cam = &app.cam;
    let sw = screen_width();
    let time = get_time() as f32;
    let q = Game::jump_quality(t);
    let in_zone = t >= hamsters_core::JUMP_WINDOW_START;
    // Glow on the last stretch of the ramp, pulsing faster near the lip.
    let s = cam.scale();
    let n = 26;
    for i in 0..=n {
        let e = hamsters_core::JUMP_WINDOW_START + (1.0 - hamsters_core::JUMP_WINDOW_START) * i as f32 / n as f32;
        let w = hamsters_core::ramp_point(e.powf(2.4));
        let p = cam.to_screen(w.x, w.y + 0.25);
        let k = i as f32 / n as f32;
        let gold = k > 0.84;
        let pulse = 0.55 + 0.45 * (time * (if gold { 14.0 } else { 6.0 }) - k * 8.0).sin();
        let col = if gold { Color::new(1.0, 0.86, 0.3, 0.9 * pulse) } else { Color::new(1.0, 1.0, 1.0, 0.35 * pulse) };
        draw_circle(p.x, p.y, (0.12 + 0.12 * k) * s, col);
    }
    let _ = (big, font, sw, q, in_zone);
}

fn picker(app: &App) {
    let font = &app.font;
    let cards = picker_cards();
    text_outlined(font, "pick your pilot  (1 / 2 / 3)", cards[0].x + cards[0].w, cards[0].y - 14.0, 18, CREAM, Align::Right);
    for (i, h) in PILOTS.iter().enumerate() {
        let r = cards[i];
        let selected = i == app.pilot_idx;
        panel(r.x, r.y, r.w, r.h);
        if selected {
            rounded(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, 10.0, Color::new(0.96, 0.78, 0.36, 0.14));
            draw_rectangle_lines(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0, 2.0, GOLD);
        }
        blit(&app.sprites.dogs[i], vec2(r.x + 30.0, r.y + r.h / 2.0), 50.0, 0.0, false, WHITE);
        let c = if selected { WHITE } else { DIM };
        let (pname, word) = app.pack.pilots.get(i).map(|t| (t.name.as_str(), t.word.as_str())).unwrap_or((h.name, "far"));
        text_outlined(&app.display, pname, r.x + 60.0, r.y + 26.0, 22, c, Align::Left);
        text(font, word, r.x + 60.0, r.y + 46.0, 15, DIM, Align::Left);
        let _ = h;
    }
}
