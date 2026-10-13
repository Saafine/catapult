//! Screen-space UI: a painted paper Christmas card. Cream paper panels with
//! painted depth, ribbons, wax seals, medals and stamps from the UI kit
//! (`tools/gen_ui.py`, drawn by `ui.rs`), Lilita One for every number and
//! shout, Fredoka for labels. Every UI colour is defined in the palette below
//! and nowhere else.

use hamsters_core::{Game, Phase, PILOTS, PILOT_RADIUS, STOMP_REACH};
use macroquad::prelude::*;

use crate::draw::lerp;
use crate::sprites::blit;
use crate::ui::{self, Rib};
use crate::{App, Fx};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

pub const INK: Color = Color::new(0.118, 0.141, 0.251, 1.0);
pub const PAPER: Color = Color::new(1.0, 0.957, 0.878, 1.0);
pub const SNOW: Color = Color::new(0.957, 0.969, 0.984, 1.0);
pub const BERRY: Color = Color::new(0.902, 0.224, 0.275, 1.0);
pub const PINE: Color = Color::new(0.180, 0.620, 0.357, 1.0);
pub const GOLD: Color = Color::new(0.961, 0.773, 0.259, 1.0);
pub const ICE: Color = Color::new(0.749, 0.914, 1.0, 1.0);
pub const AMBER: Color = Color::new(0.941, 0.651, 0.227, 1.0);
/// Secondary text on paper: solid slate, never ink at alpha (that goes muddy).
pub const SLATE: Color = Color::new(0.353, 0.380, 0.471, 1.0);
/// Deeper shades for outlines on coloured art (the painted style never uses
/// one black line).
const BERRY_DEEP: Color = Color::new(0.43, 0.05, 0.09, 1.0);
const PINE_DEEP: Color = Color::new(0.05, 0.26, 0.15, 1.0);
const GOLD_DEEP: Color = Color::new(0.54, 0.32, 0.05, 1.0);
const ICE_DEEP: Color = Color::new(0.30, 0.48, 0.64, 1.0);
const PAPER_DEEP: Color = Color::new(0.66, 0.45, 0.25, 1.0);
const BROWN: Color = Color::new(0.29, 0.17, 0.13, 1.0);
const SHADOW_ALPHA: f32 = 0.25;
/// Seconds the dog sits in the snow before the results card arrives.
pub const RESULT_HOLD: f32 = 1.6;
/// Shorter after a crash: the SPLAT banner and the sinking fill the beat.
pub const RESULT_HOLD_CRASH: f32 = 3.0;

pub fn result_hold(crashed: bool) -> f32 {
    if crashed { RESULT_HOLD_CRASH } else { RESULT_HOLD }
}

pub fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

/// Ease-out-back: overshoots to about 1.1 then settles at 1.
pub fn ease_out_back(k: f32) -> f32 {
    ui::ease_out_back(k)
}

/// Every event shout gets one of four ribbon colours, by the hue of the
/// colour the event chose.
pub fn ribbon_color(c: Color) -> Color {
    if c.r > 0.85 && c.g > 0.7 && c.b < 0.6 {
        GOLD
    } else if c.g > c.r && c.g > c.b {
        PINE
    } else if c.r > c.g + 0.2 {
        BERRY
    } else {
        ICE
    }
}

/// The painted ribbon for an event colour, and the deep shade its text is
/// outlined in.
fn ribbon_kind(c: Color) -> (Rib, Color) {
    let r = ribbon_color(c);
    if r == GOLD {
        (Rib::Gold, GOLD_DEEP)
    } else if r == PINE {
        (Rib::Green, PINE_DEEP)
    } else if r == BERRY {
        (Rib::Red, BERRY_DEEP)
    } else {
        (Rib::Ice, ICE_DEEP)
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

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
    text_scaled(font, s, x, y, size, 1.0, color, align);
}

pub fn text_scaled(font: &Font, s: &str, x: f32, y: f32, size: u16, scale: f32, color: Color, align: Align) {
    let x = aligned_x(font, s, x, size, scale, align);
    draw_text_ex(s, x, y, TextParams { font: Some(font), font_size: size, font_scale: scale, color, ..Default::default() });
}

/// A shout: fill with a hard ink outline (eight offset draws), never a soft
/// shadow. The outline scales with the type size.
pub fn text_shout(font: &Font, s: &str, x: f32, y: f32, size: u16, scale: f32, fill: Color, align: Align) {
    let x = aligned_x(font, s, x, size, scale, align);
    let t = (size as f32 * 0.075).clamp(1.5, 4.0) * scale;
    let ink = fade(INK, fill.a.min(1.0));
    for (dx, dy) in [(t, 0.0), (-t, 0.0), (0.0, t), (0.0, -t), (t, t), (-t, t), (t, -t), (-t, -t)] {
        draw_text_ex(s, x + dx, y + dy, TextParams { font: Some(font), font_size: size, font_scale: scale, color: ink, ..Default::default() });
    }
    draw_text_ex(s, x, y, TextParams { font: Some(font), font_size: size, font_scale: scale, color: fade(fill, 1.0), ..Default::default() });
}

/// A number with its unit at 60% size on the same baseline: "35.5 m".
pub fn number_unit(font: &Font, num: &str, unit: &str, x: f32, y: f32, size: u16, scale: f32, color: Color, align: Align) {
    let small = (size as f32 * 0.6) as u16;
    let wn = measure_text(num, Some(font), size, scale).width;
    let wu = measure_text(unit, Some(font), small, scale).width;
    let gap = 3.0 * scale;
    let x0 = match align {
        Align::Left => x,
        Align::Center => x - (wn + gap + wu) / 2.0,
        Align::Right => x - (wn + gap + wu),
    };
    text_scaled(font, num, x0, y, size, scale, color, Align::Left);
    text_scaled(font, unit, x0 + wn + gap, y, small, scale, color, Align::Left);
}

/// A painted number with its unit: deep outline, soft drop, centred on `x`.
fn number_painted(font: &Font, num: &str, unit: &str, x: f32, y: f32, size: u16, scale: f32, fill: Color, edge: Color) {
    let small = (size as f32 * 0.55) as u16;
    let wn = ui::text_w(font, num, size, scale);
    let wu = ui::text_w(font, unit, small, scale);
    let gap = 5.0 * scale;
    let x0 = x - (wn + gap + wu) / 2.0;
    let t = (size as f32 * 0.06).clamp(2.0, 5.0) * scale;
    ui::text_painted(font, num, x0 + wn / 2.0, y, size, scale, fill, edge, t, 0.0);
    ui::text_painted(font, unit, x0 + wn + gap + wu / 2.0, y, small, scale, fill, edge, t * 0.8, 0.0);
}

// ---------------------------------------------------------------------------
// Paper shapes
// ---------------------------------------------------------------------------

/// Rounded rectangle as one triangle fan.
pub fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    fan(&tag_pts(x, y, w, h, r, 0.0), vec2(x + w / 2.0, y + h / 2.0), color);
}

fn fan(pts: &[Vec2], c: Vec2, color: Color) {
    for i in 0..pts.len() {
        draw_triangle(c, pts[i], pts[(i + 1) % pts.len()], color);
    }
}

/// Outline of a rounded rect; when `cut` is positive the top-left corner is
/// sliced off like a gift tag.
fn tag_pts(x: f32, y: f32, w: f32, h: f32, r: f32, cut: f32) -> Vec<Vec2> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pts = Vec::with_capacity(32);
    let arc = |pts: &mut Vec<Vec2>, cx: f32, cy: f32, start: f32| {
        for i in 0..=6 {
            let a = (start + i as f32 * 15.0).to_radians();
            pts.push(vec2(cx + a.cos() * r, cy + a.sin() * r));
        }
    };
    if cut > 0.0 {
        pts.push(vec2(x + cut, y));
    } else {
        arc(&mut pts, x + r, y + r, 180.0);
    }
    arc(&mut pts, x + w - r, y + r, -90.0);
    arc(&mut pts, x + w - r, y + h - r, 0.0);
    arc(&mut pts, x + r, y + h - r, 90.0);
    if cut > 0.0 {
        pts.push(vec2(x, y + cut));
    }
    pts
}

/// A swallow-tailed ribbon, tilted, with an ink outline.
pub fn ribbon(cx: f32, cy: f32, w: f32, h: f32, color: Color, tilt: f32, alpha: f32) {
    let (s, c) = (tilt.sin(), tilt.cos());
    let rot = |p: Vec2| vec2(cx + p.x * c - p.y * s, cy + p.x * s + p.y * c);
    let shape = |w: f32, h: f32, col: Color, dy: f32| {
        let notch = h * 0.45;
        let pts = [
            vec2(-w / 2.0, -h / 2.0),
            vec2(w / 2.0, -h / 2.0),
            vec2(w / 2.0 - notch, 0.0),
            vec2(w / 2.0, h / 2.0),
            vec2(-w / 2.0, h / 2.0),
            vec2(-w / 2.0 + notch, 0.0),
        ]
        .map(|p| rot(p + vec2(0.0, dy)));
        fan(&pts, rot(vec2(0.0, dy)), col);
    };
    shape(w + 6.0, h + 6.0, fade(INK, SHADOW_ALPHA * alpha), 5.0);
    shape(w + 6.0, h + 6.0, fade(INK, alpha), 0.0);
    shape(w, h, fade(color, alpha), 0.0);
}

/// A value that matters: ink text on a small gold pill. Gold text never sits
/// on paper. `y` is the text baseline.
fn gold_pill(font: &Font, s: &str, x: f32, y: f32, size: u16, align: Align, alpha: f32) {
    let d = measure_text(s, Some(font), size, 1.0);
    let cap = size as f32 * 0.72;
    let (pw, ph) = (d.width + 16.0, cap + 10.0);
    let px = match align {
        Align::Left => x - 8.0,
        Align::Center => x - pw / 2.0,
        Align::Right => x - pw + 8.0,
    };
    let py = y - cap - 5.0;
    ui::rrect(px - 2.0, py - 1.0, pw + 4.0, ph + 4.0, ph / 2.0 + 2.0, fade(GOLD_DEEP, alpha));
    ui::rrect(px, py, pw, ph, ph / 2.0, fade(GOLD, alpha));
    ui::rrect(px + 4.0, py + 2.0, pw - 8.0, ph * 0.35, ph * 0.18, fade(WHITE, 0.45 * alpha));
    text(font, s, x, y, size, fade(INK, alpha), align);
}

/// Per-pilot portrait colours: (light, deep) for the medallion disc.
fn pilot_colors(i: usize) -> (Color, Color) {
    match i {
        0 => (Color::new(1.0, 0.55, 0.50, 1.0), Color::new(0.78, 0.17, 0.22, 1.0)),
        1 => (Color::new(0.72, 0.88, 1.0, 1.0), Color::new(0.30, 0.52, 0.78, 1.0)),
        _ => (Color::new(0.60, 0.90, 0.62, 1.0), Color::new(0.14, 0.52, 0.30, 1.0)),
    }
}

/// A round gold-rimmed portrait with the pilot standing out of the frame.
fn medallion(app: &App, pilot: usize, c: Vec2, r: f32, dog_w: f32, dim: bool) {
    let (light, deep) = pilot_colors(pilot);
    draw_circle(c.x, c.y + 4.0, r + 6.0, Color::new(0.07, 0.09, 0.2, 0.3));
    draw_circle(c.x, c.y, r + 6.0, GOLD_DEEP);
    draw_circle(c.x, c.y, r + 4.0, GOLD);
    draw_circle(c.x, c.y - 1.5, r + 2.5, Color::new(1.0, 0.94, 0.62, 1.0));
    draw_circle(c.x, c.y + 0.5, r + 1.5, GOLD);
    draw_circle(c.x, c.y, r, deep);
    draw_circle(c.x - r * 0.1, c.y - r * 0.14, r * 0.82, lerp(deep, light, 0.6));
    draw_circle(c.x - r * 0.18, c.y - r * 0.26, r * 0.55, light);
    let tint = if dim { Color::new(0.82, 0.82, 0.86, 1.0) } else { WHITE };
    // The dog sprite's art sits right of centre: nudge it back.
    ui::spr(&app.sprites.dogs[pilot.min(2)], c + vec2(-dog_w * 0.05, -dog_w * 0.02), dog_w, 0.0, tint);
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

const CARD_W: f32 = 256.0;
const CARD_H: f32 = 112.0;

fn picker_cards() -> [Rect; 3] {
    let gap = 12.0;
    let x = screen_width() - CARD_W - 18.0;
    [0, 1, 2].map(|i| Rect::new(x, 74.0 + i as f32 * (CARD_H + gap), CARD_W, CARD_H))
}

pub fn picker_hit(p: Vec2) -> Option<usize> {
    picker_cards().iter().position(|r| r.contains(p))
}

/// The results postcard, centred.
fn results_rect() -> Rect {
    let (sw, sh) = (screen_width(), screen_height());
    let (w, h) = (784.0, 468.0);
    Rect::new((sw - w) / 2.0, (sh - h) / 2.0 + 14.0, w, h)
}

/// Where the Restart button sits on the results card (for hit-testing).
pub fn restart_button_rect() -> Rect {
    let r = results_rect();
    let (w, h) = (264.0, 64.0);
    let cx = r.x + 404.0 + 176.0;
    Rect::new(cx - w / 2.0, r.y + r.h - h - 42.0, w, h)
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

pub fn render(app: &App) {
    let g = &app.game;
    let sw = screen_width();
    let sh = screen_height();
    let now = get_time();
    let tag = match g.phase {
        Phase::AimPower { .. } => 0,
        Phase::Sliding { .. } => 1,
        Phase::Flying => 2,
        Phase::Landed => 3,
    };
    let combo = app.combo_now();
    let (since, pilot_since) = ui::with_state(|s| {
        if s.phase != tag {
            s.phase = tag;
            s.phase_at = now;
        }
        if s.pilot != app.pilot_idx {
            s.pilot = app.pilot_idx;
            s.pilot_at = now;
        }
        if combo >= 2 && combo != s.combo {
            s.combo = combo;
            s.combo_at = now;
        } else if combo < 2 && s.combo >= 2 {
            if s.combo_gone_at < s.combo_at {
                s.combo_gone_at = now;
            } else if now - s.combo_gone_at > 0.4 {
                s.combo = 0;
            }
        }
        if g.presents != s.presents {
            s.presents = g.presents;
            s.presents_at = now;
        }
        if tag != 2 {
            s.slots.clear();
        }
        ((now - s.phase_at) as f32, (now - s.pilot_at) as f32)
    });
    let aiming = matches!(g.phase, Phase::AimPower { .. } | Phase::Sliding { .. });
    // The flight HUD stays up through the quiet hold after landing.
    let holding = g.phase == Phase::Landed && (app.landed_for) < result_hold(g.crashed);
    if !aiming && (g.phase != Phase::Landed || holding) {
        flight_hud(app);
    }
    match g.phase {
        Phase::AimPower { meter } => {
            title_logo(app, ui::ease_out_back((since - 0.05) / 0.5), 0.0);
            picker(app, since, pilot_since, 0.0);
            let rise = 1.0 - ui::ease_out_cubic((since - 0.1) / 0.45);
            let dy = 170.0 * rise;
            power_dial(app, sw / 2.0, sh - 52.0 + dy, Game::meter_value(meter), true);
            target_plank(app, sw / 2.0 - 152.0, sh - 86.0 + dy * 1.2);
            today_plank(app, sw / 2.0 + 152.0, sh - 86.0 + dy * 1.2);
            if !app.taught(crate::TEACH_PUSH) {
                let hint = "space or click to lock the push";
                let w = ui::text_w(&app.font, hint, 14, 1.0) + 28.0;
                ui::chip(&app.sprites.ui.chip_dark, Rect::new(sw / 2.0 - w / 2.0, sh - 27.0 + dy, w, 22.0), 0.45, WHITE);
                text(&app.font, hint, sw / 2.0, sh - 11.0 + dy, 14, PAPER, Align::Center);
            }
        }
        Phase::Sliding { power, t, jump } => {
            // The title and the picker leave as the sled goes.
            let out = ui::ease_in_cubic(since / 0.4);
            if out < 1.0 {
                title_logo(app, 1.0, out);
                picker(app, 10.0, 10.0, out);
            }
            power_dial(app, sw / 2.0, sh - 52.0, power, false);
            if jump.is_none() {
                jump_timer(app, t);
                jump_prompt(app, t);
            }
        }
        Phase::Flying => boost_bars(app),
        Phase::Landed => results(app),
    }
    teach_prompts(app);
}

/// The game logo, top centre over the sky: pops in with the title, bobs
/// gently, and flies off upwards (`out` 0..1) when the sled goes.
fn title_logo(app: &App, enter: f32, out: f32) {
    if enter <= 0.0 {
        return;
    }
    let kit = &app.sprites.ui;
    let sw = screen_width();
    let t = get_time() as f32;
    let right = picker_cards()[0].x - 16.0;
    let w = (right - 250.0).clamp(300.0, 470.0);
    let cx = (sw / 2.0).min(right - w / 2.0 + 10.0);
    let h = w * kit.logo.height() / kit.logo.width();
    let bob = (t * 1.6).sin() * 3.0;
    let cy = 8.0 + h / 2.0 + bob - out * (h + 60.0);
    let scale = enter.max(0.0);
    let a = 1.0 - out;
    ui::spr(&kit.logo, vec2(cx, cy), w * scale, (t * 1.1).sin() * 0.012, ui::tint(a));
    // A glint that sweeps across the letters every few seconds.
    let cyc = (t * 0.35).fract();
    if cyc < 0.25 && scale >= 1.0 {
        let k = cyc / 0.25;
        let gx = cx - w * 0.42 + w * 0.84 * k;
        let gy = cy + h * 0.2;
        let r = 10.0 + 6.0 * (k * std::f32::consts::PI).sin();
        draw_circle(gx, gy, r * 1.8, Color::new(1.0, 1.0, 0.9, 0.18 * a));
        draw_line(gx - r * 1.6, gy, gx + r * 1.6, gy, 2.5, Color::new(1.0, 1.0, 1.0, 0.85 * a));
        draw_line(gx, gy - r * 1.6, gx, gy + r * 1.6, 2.5, Color::new(1.0, 1.0, 1.0, 0.85 * a));
    }
}

/// One rule at a time, at the moment it matters, never again once used:
/// the second press at the lip, the flap, and (after a few flights) the stomp.
fn teach_prompts(app: &App) {
    let g = &app.game;
    if app.preview > 0.0 {
        return;
    }
    let big = &app.display;
    let t = get_time() as f32;
    let pulse = 1.0 + 0.04 * (1.0 + (t * 6.0).sin());
    if let Phase::Flying = g.phase {
        let hp = app.cam.to_screen(g.pos.x, g.pos.y);
        let r = PILOT_RADIUS * app.cam.scale();
        let y = (hp.y + r * 4.3).min(screen_height() - 40.0);
        let x = hp.x.clamp(140.0, screen_width() - 140.0);
        if g.botched {
            // The fall after a missed press: the banner says MISSED THE
            // JUMP; this line under it says exactly what was missing.
            text_shout(big, "PRESS SPACE OR CLICK AT THE LIP", screen_width() / 2.0, 236.0, 20, pulse, PAPER, Align::Center);
        } else if g.flaps == 0 && g.flight_time > 0.6 && g.flight_time < 4.0 && !app.taught(crate::TEACH_FLAP) {
            key_prompt(app, x, y, &format!("TO {}", app.pack.labels.flap), pulse);
        } else if app.stomp_teachable() && g.vel.y < 0.0 && g.pos.y < PILOT_RADIUS + STOMP_REACH + 4.0 && g.pos.y > PILOT_RADIUS + 0.5 {
            key_prompt(app, x, y, "TO STOMP", pulse);
        }
    }
}

/// A paper plate reading [SPACE][mouse] WORD, centred on (x, y), with a
/// small tail pointing up at the dog.
fn key_prompt(app: &App, x: f32, y: f32, word: &str, k: f32) {
    let (font, big) = (&app.font, &app.display);
    let caps_w = 62.0 + 8.0 + 18.0;
    let tw = ui::text_w(big, word, 22, 1.0);
    let w = (caps_w + 14.0 + tw + 36.0) * k;
    let h = 46.0 * k;
    let (px, py) = (x - w / 2.0, y - h / 2.0);
    ui::panel(&app.sprites.ui.panel, Rect::new(px, py, w, h), 0.42, WHITE);
    draw_triangle(vec2(x - 9.0, py + 1.0), vec2(x + 9.0, py + 1.0), vec2(x, py - 9.0), PAPER_DEEP);
    draw_triangle(vec2(x - 6.0, py + 3.0), vec2(x + 6.0, py + 3.0), vec2(x, py - 5.0), PAPER);
    key_caps(font, px + 18.0 * k + caps_w * k / 2.0, py + h / 2.0, k);
    text_scaled(big, word, px + (18.0 + caps_w + 14.0) * k, py + h / 2.0 + 8.0 * k, 22, k, INK, Align::Left);
}

/// Distance tag, presents seal, combo burst and the height ruler.
fn flight_hud(app: &App) {
    let g = &app.game;
    let big = &app.display;
    let kit = &app.sprites.ui;
    let sw = screen_width();
    let now = get_time();
    let flying = g.phase == Phase::Flying;
    // Entry: the panel drops in over 300 ms after the lip with an overshoot.
    let enter = if flying { ease_out_back(g.flight_time / 0.3) } else { 1.0 };
    let (x, w, h) = (14.0, 300.0, if flying { 86.0 } else { 58.0 });
    let y = 14.0 - 110.0 * (1.0 - enter);
    ui::panel(&kit.panel, Rect::new(x, y, w, h), 0.5, WHITE);
    ui::spr(&kit.ic_flag, vec2(x + 30.0, y + 30.0), 40.0, 0.0, WHITE);
    let dp = 1.0 + 0.15 * (app.dist_pulse * std::f32::consts::PI).sin().max(0.0);
    number_unit(big, &format!("{:.1}", g.distance()), "m", x + 56.0, y + 45.0, 40, dp, INK, Align::Left);
    rank_line(app, x + w - 14.0, y + 26.0, 1.0);
    if flying {
        // FLAP: a painted segmented tube, eased so a flap drains visibly.
        let target = g.stamina.max(0.0);
        let shown = ui::with_state(|s| {
            s.stamina += (target - s.stamina) * 0.25;
            s.stamina
        });
        let fy = y + 66.0;
        ui::spr(&kit.ic_ear, vec2(x + 28.0, fy), 28.0, (now as f32 * 9.0).sin() * 0.08 * (app.flap_anim.min(1.0)), WHITE);
        text(big, &app.pack.labels.flap, x + 46.0, fy + 7.0, 18, INK, Align::Left);
        let left = ((target / hamsters_core::FLAP_COST).floor() as i32).clamp(0, 12);
        let blink = (now as f32 * 6.0 * std::f32::consts::TAU).sin() > 0.0;
        let col = if left < 3 {
            if blink { BERRY } else { Color::new(1.0, 0.6, 0.6, 1.0) }
        } else if left < 6 {
            AMBER
        } else {
            PINE
        };
        let (bx, bw, bh) = (x + 98.0, w - 116.0, 16.0);
        ui::rrect(bx - 3.0, fy - bh / 2.0 - 3.0, bw + 6.0, bh + 6.0, bh / 2.0 + 3.0, BROWN);
        ui::rrect(bx, fy - bh / 2.0, bw, bh, bh / 2.0, Color::new(0.42, 0.33, 0.30, 1.0));
        let frac = (shown / (12.0 * hamsters_core::FLAP_COST)).clamp(0.0, 1.0);
        let seg = bw / 12.0;
        for i in 0..12 {
            let k = ((frac * 12.0) - i as f32).clamp(0.0, 1.0);
            if k <= 0.0 {
                continue;
            }
            let sx = bx + i as f32 * seg + 1.0;
            let sw_ = (seg - 2.0) * k;
            ui::rrect(sx, fy - bh / 2.0 + 1.0, sw_, bh - 2.0, 4.0, ui::shade(col, 0.75));
            ui::rrect(sx, fy - bh / 2.0 + 1.0, sw_, bh * 0.62, 4.0, col);
            ui::rrect(sx + 2.0, fy - bh / 2.0 + 3.0, (sw_ - 4.0).max(0.0), 3.0, 1.5, Color::new(1.0, 1.0, 1.0, 0.55));
        }
        streak_line(app, x + 46.0, y + h + 22.0);
    }
    // Presents: a green wax seal with the gift, the count on a paper chip.
    let (pulse_t, combo, combo_at, gone_at) = ui::with_state(|s| ((now - s.presents_at) as f32, s.combo, s.combo_at, s.combo_gone_at));
    let pulse = 1.0 + 0.3 * (1.0 - (pulse_t / 0.25).clamp(0.0, 1.0)) * (pulse_t / 0.25 * std::f32::consts::PI).sin().abs();
    let (cx, cy) = (sw / 2.0 - 30.0, 42.0 - 110.0 * (1.0 - enter));
    ui::chip(&kit.chip, Rect::new(cx + 6.0, cy - 21.0, 88.0, 42.0), 0.5, WHITE);
    ui::spr(&kit.seal_green, vec2(cx, cy), 64.0 * pulse, 0.0, WHITE);
    blit(&app.sprites.presents[0], vec2(cx, cy - 1.0), 32.0 * pulse, 0.1, false, WHITE);
    text_scaled(big, &g.presents.to_string(), cx + 58.0, cy + 11.0, 30, pulse, INK, Align::Center);
    // Combo: a gold burst that punches on every link and fades after the chain.
    if combo >= 2 {
        let age = (now - combo_at) as f32;
        let fade_k = if gone_at > combo_at { 1.0 - ((now - gone_at) as f32 / 0.35).clamp(0.0, 1.0) } else { 1.0 };
        let pop = ui::ease_out_elastic(age / 0.45) * (0.85 + 0.15 * fade_k);
        let heat = ((combo as f32 - 2.0) / 6.0).clamp(0.0, 1.0);
        let bx = cx + 138.0;
        let size = (64.0 + 10.0 * heat) * pop;
        let spin = now as f32 * 1.2;
        ui::spr(&kit.burst, vec2(bx, cy), size, spin, ui::tint(fade_k));
        ui::text_painted(big, &format!("x{}", combo), bx, cy + 10.0, 30, pop.max(0.01), fade(PAPER, fade_k), fade(Color::new(0.6, 0.22, 0.04, 1.0), fade_k), 3.0, -0.08);
    }
    if flying {
        height_ruler(app);
    }
}

/// A painted rail on the right edge with the milestone heights on paper
/// chips and a paper arrow tag sliding along it. Hidden below 60 m.
fn height_ruler(app: &App) {
    let alt = app.game.altitude();
    let show = ((alt - 50.0) / 10.0).clamp(0.0, 1.0);
    if show <= 0.0 {
        return;
    }
    let kit = &app.sprites.ui;
    let big = &app.display;
    let sw = screen_width();
    let sh = screen_height();
    let x = sw - 26.0 + 40.0 * (1.0 - ui::ease_out_cubic(show));
    let (top, bottom) = (150.0, sh - 96.0);
    let k = |h: f32| (h / 720.0).clamp(0.0, 1.0).sqrt();
    let y_at = |h: f32| bottom - k(h) * (bottom - top);
    ui::rrect(x - 6.0, top - 6.0, 12.0, bottom - top + 12.0, 6.0, BROWN);
    ui::rrect(x - 3.5, top - 3.5, 7.0, bottom - top + 7.0, 3.5, ICE);
    ui::rrect(x - 3.5, y_at(alt), 7.0, bottom - y_at(alt) + 3.5, 3.5, PINE);
    ui::spr(&kit.ic_alt, vec2(x - 4.0, top - 26.0), 34.0, 0.0, WHITE);
    let arrow_y = y_at(alt);
    for (h, _) in hamsters_core::MILESTONES.iter() {
        let y = y_at(*h);
        draw_line(x - 10.0, y, x + 6.0, y, 3.0, BROWN);
        if (y - arrow_y).abs() < 26.0 {
            continue;
        }
        let (cw, ch) = (38.0, 18.0);
        ui::chip(&kit.chip, Rect::new(x - 14.0 - cw, y - ch / 2.0, cw, ch), 0.3, WHITE);
        text(big, &format!("{:.0}", h), x - 14.0 - cw / 2.0, y + 5.0, 13, INK, Align::Center);
    }
    let y = arrow_y;
    let (w, h) = (70.0, 30.0);
    let tx = x - 24.0 - w;
    draw_triangle(vec2(tx + w - 2.0, y - 11.0), vec2(tx + w - 2.0, y + 11.0), vec2(tx + w + 14.0, y), GOLD_DEEP);
    draw_triangle(vec2(tx + w - 2.0, y - 7.0), vec2(tx + w - 2.0, y + 7.0), vec2(tx + w + 8.0, y), GOLD);
    ui::chip(&kit.chip, Rect::new(tx, y - h / 2.0, w, h), 0.4, Color::new(1.0, 0.93, 0.7, 1.0));
    number_unit(big, &format!("{:.0}", alt), "m", tx + w / 2.0 - 2.0, y + 8.0, 21, 1.0, INK, Align::Center);
}

/// Active boosts, bottom-left: an icon on a paper chip with a draining bar.
fn boost_bars(app: &App) {
    let g = &app.game;
    let sh = screen_height();
    let kit = &app.sprites.ui;
    let mut by = sh - 42.0;
    for (tex, frac, col) in [(&app.sprites.wings, g.glide / (3.5 * g.pilot.glide_bonus), ICE), (&app.sprites.firework, g.rocket / 1.2, AMBER)] {
        if frac <= 0.0 {
            continue;
        }
        let frac = frac.clamp(0.0, 1.0);
        ui::chip(&kit.chip, Rect::new(16.0, by - 19.0, 184.0, 38.0), 0.5, WHITE);
        blit(tex, vec2(42.0, by), 38.0, 0.0, false, WHITE);
        ui::bar(66.0, by - 6.0, 118.0, 12.0, frac, col, Color::new(0.42, 0.33, 0.30, 1.0), BROWN);
        by -= 48.0;
    }
}

/// Event text: banners on a painted ribbon in whichever slot is clear of the
/// dog, the rank stamp on its pennant. Combo counts live in the HUD burst.
/// Returns true when the effect was drawn here (main.rs then skips its own).
pub fn draw_fx(app: &App, f: &Fx, hp: Vec2, r: f32) -> bool {
    let k = f.age / f.life;
    let big = &app.display;
    let kit = &app.sprites.ui;
    let (sw, sh) = (screen_width(), screen_height());
    let zap = !f.banner && f.text == app.pack.labels.stormcloud;
    if f.banner || zap {
        if f.text.is_empty() {
            return true;
        }
        // Two slots: under the top HUD, or low over the street. Pick the one
        // farther from the dog when the banner first shows, and keep it.
        let key = (f.text.clone(), f.life.to_bits());
        let top_slot = 168.0;
        let low_slot = sh - 118.0;
        // The clear slot is re-checked every frame (the camera keeps moving
        // after a teleport or a dive); the ribbon glides to it if it changes.
        let dog_top = hp.y - r * 2.6;
        let dog_bot = hp.y + r * 2.6;
        let clear = |y: f32| (y + 44.0) < dog_top || (y - 44.0) > dog_bot;
        let target = if clear(top_slot) {
            top_slot
        } else if clear(low_slot) {
            low_slot
        } else if (hp.y - top_slot).abs() > (hp.y - low_slot).abs() {
            top_slot
        } else {
            low_slot
        };
        let cy = ui::with_state(|s| {
            if let Some(e) = s.slots.iter_mut().find(|e| e.0 == key.0 && e.1 == key.1) {
                e.2 += (target - e.2) * 0.18;
                return e.2;
            }
            s.slots.push((key.0.clone(), key.1, target));
            if s.slots.len() > 8 {
                s.slots.remove(0);
            }
            target
        });
        let slide = ui::ease_out_back(f.age / 0.22);
        let out = ((k - 0.72) / 0.28).clamp(0.0, 1.0);
        let a = 1.0 - out * out;
        let size = if zap { 52 } else { 46 };
        let tw = ui::text_w(big, &f.text, size, 1.0);
        let (bw, bh) = (tw + 70.0, 62.0);
        // In from the left with an overshoot, out to the right, so a banner
        // that follows never sits on top of this one.
        let cx = -bw + (sw / 2.0 + bw) * slide + (sw / 2.0 + bw) * ui::ease_in_cubic(out);
        let cy = cy - 10.0 * out;
        let tilt = -2.5f32.to_radians();
        let (rib, edge) = if zap { (Rib::Red, BERRY_DEEP) } else { ribbon_kind(f.color) };
        let punch = 1.0 + 0.12 * (1.0 - (f.age / 0.3).clamp(0.0, 1.0));
        ui::ribbon(kit.ribbon(rib), vec2(cx, cy), bw * punch, bh * punch, tilt, ui::tint(a));
        let (fill, edge) = if rib == Rib::Ice { (INK, PAPER) } else if rib == Rib::Gold { (PAPER, edge) } else { (PAPER, edge) };
        ui::text_painted(big, &f.text, cx, cy + size as f32 * 0.36 * punch, size, punch, fade(fill, a), fade(edge, a), 3.5, tilt);
        // Sparkles at the ribbon ends for the gold ones.
        if rib == Rib::Gold && a > 0.5 {
            let t = get_time() as f32;
            for i in 0..4 {
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let ph = (t * 3.0 + i as f32 * 1.3).sin();
                let px = cx + side * (bw / 2.0 + 30.0 + 10.0 * i as f32);
                let py = cy - 24.0 + 18.0 * i as f32 * 0.6 + ph * 4.0;
                let rr = 5.0 + 3.0 * ph.abs();
                draw_line(px - rr, py, px + rr, py, 2.0, fade(WHITE, a));
                draw_line(px, py - rr, px, py + rr, 2.0, fade(WHITE, a));
            }
        }
        return true;
    }
    if f.text.starts_with('x') {
        // The combo count is the burst beside the presents counter.
        return true;
    }
    if f.text.starts_with('#') {
        // The rank stamp snaps down onto the pennant that marks this run.
        let anchor = app.cam.to_screen(f.pos.x, f.pos.y);
        let drift = -90.0 * (1.0 - (1.0 - (k / 0.25).min(1.0)).powi(3)) + 90.0;
        let x = anchor.x.clamp(130.0, sw - 130.0);
        let y = (anchor.y - drift).clamp(150.0, sh - 60.0);
        let a = if k < 0.7 { 1.0 } else { (1.0 - k) / 0.3 };
        let pop = ui::ease_out_elastic(k / 0.3);
        ui::spr(&kit.seal_gold, vec2(x, y - 14.0), 78.0 * pop, -0.15, ui::tint(a));
        ui::text_painted(big, &f.text, x, y - 2.0, 32, pop.max(0.01), fade(PAPER, a), fade(GOLD_DEEP, a), 3.0, -0.15);
        return true;
    }
    false
}

/// The results postcard: ribbon header, portrait with medal, the distance
/// counting up, a best comparison, stat chips, the scoreboard and a big
/// FLY AGAIN button.
fn results(app: &App) {
    let g = &app.game;
    let big = &app.display;
    let kit = &app.sprites.ui;
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;
    // Nothing until the hold is over: the player gets the dog and the snow first.
    let since = app.landed_for - result_hold(g.crashed);
    if since < 0.0 {
        return;
    }
    let enter = ui::ease_out_back(since / 0.4);
    draw_rectangle(0.0, 0.0, sw, sh, fade(Color::new(0.06, 0.08, 0.18, 1.0), 0.55 * (since / 0.3).min(1.0)));
    let r = results_rect();
    let (x, y, w, h) = (r.x, r.y + (sh - r.y) * (1.0 - enter), r.w, r.h);
    let new_best = g.distance() >= g.best_distance - 0.01 && g.distance() > 0.0;
    let my_rank = app.scores.iter().position(|sc| (sc.distance - g.distance()).abs() < 0.01 && sc.pilot == app.pilot_idx);
    if new_best {
        // Slow golden rays behind the card.
        for i in 0..18 {
            let a = i as f32 * std::f32::consts::TAU / 18.0 + t * 0.25;
            let c = vec2(sw / 2.0, y + h * 0.4);
            draw_triangle(c, c + vec2(a.cos(), a.sin()) * 640.0, c + vec2((a + 0.12).cos(), (a + 0.12).sin()) * 640.0, fade(GOLD, 0.16));
        }
    }
    ui::panel(&kit.panel, Rect::new(x, y, w, h), 0.9, WHITE);
    ui::nine(&kit.snow_wide, Rect::new(x - 8.0, y - 14.0, w + 16.0, 52.0), 40.0, 26.0, WHITE);
    ui::spr(&kit.holly, vec2(x + 40.0, y + 22.0), 92.0, -0.2, WHITE);
    ui::spr(&kit.holly, vec2(x + w - 40.0, y + h - 18.0), 84.0, 2.9, WHITE);
    // Header ribbon straddling the top edge.
    let (head, rib, edge) = if new_best {
        ("NEW BEST!".to_owned(), Rib::Gold, GOLD_DEEP)
    } else if g.crashed {
        (app.pack.labels.crash.clone(), Rib::Red, BERRY_DEEP)
    } else {
        (app.pack.labels.flight_over.clone(), Rib::Green, PINE_DEEP)
    };
    let hk = ui::ease_out_back((since - 0.15) / 0.35);
    if hk > 0.0 {
        let hw = ui::text_w(big, &head, 44, 1.0) + 120.0;
        ui::ribbon(kit.ribbon(rib), vec2(x + w / 2.0, y + 4.0), hw * hk, 64.0 * hk, 0.0, WHITE);
        ui::text_painted(big, &head, x + w / 2.0, y + 20.0, 44, hk, PAPER, edge, 3.5, 0.0);
        if new_best || g.crashed {
            let sub = if new_best && g.crashed { app.pack.labels.crash.as_str() } else { "" };
            if !sub.is_empty() {
                text(big, sub, x + w / 2.0, y + 54.0, 14, BERRY, Align::Center);
            }
        }
    }

    // Left: portrait, medal, the distance counting up, the comparison.
    let lx = x + 200.0;
    medallion(app, app.pilot_idx, vec2(lx, y + 128.0), 60.0, 150.0, false);
    let pname = app.pack.pilots.get(app.pilot_idx).map(|p| p.name.as_str()).unwrap_or("?");
    let tick = (since / 1.2).clamp(0.0, 1.0);
    let ease = ui::ease_out_cubic(tick);
    let landed_pop = 1.0 + 0.14 * (1.0 - ((since - 1.2) / 0.25).clamp(0.0, 1.0)) * if since > 1.2 { 1.0 } else { 0.0 };
    number_painted(big, &format!("{:.1}", g.distance() * ease), "m", lx, y + 266.0, 72, landed_pop, INK, PAPER);
    text(big, &format!("{} flew", pname).to_uppercase(), lx, y + 208.0, 15, SLATE, Align::Center);
    // Medal / rosette slams on after the count.
    let mk = (since - 1.25) / 0.45;
    if mk > 0.0 {
        let s = 2.4 - 1.4 * ui::ease_out_back(mk);
        let a = (mk * 4.0).clamp(0.0, 1.0);
        let at = vec2(lx + 70.0, y + 152.0);
        if new_best {
            ui::spr(&kit.rosette, at + vec2(4.0, 6.0), 92.0 * s, 0.18, ui::tint(a));
            ui::text_painted(big, "BEST", at.x + 4.0, at.y + 4.0, 20, s, fade(BROWN, a), fade(Color::new(1.0, 0.95, 0.7, 1.0), a), 2.0, 0.18);
        } else if let Some(rank) = my_rank.filter(|r| *r < 3) {
            ui::spr(&kit.medals[rank], at, 70.0 * s, 0.15, ui::tint(a));
            ui::text_painted(big, &format!("{}", rank + 1), at.x + 3.0, at.y + 22.0 * s, 22, s, fade(PAPER, a), fade(GOLD_DEEP, a), 2.0, 0.15);
        } else if g.crashed {
            ui::spr(&kit.stamp_red, at + vec2(0.0, 10.0), 120.0 * s, 0.2, ui::tint(a));
            ui::text_painted(big, &app.pack.labels.crash, at.x, at.y + 19.0, 24, s * 0.9, fade(PAPER, a), fade(BERRY_DEEP, a), 2.5, 0.2);
        }
    }
    // Best comparison bar.
    let by = y + 296.0;
    // The record this flight is measured against (the dev keys skip the
    // launch that stores best_before, so fall back to the board).
    let prev_best = if new_best { app.best_before } else { app.best_before.max(g.best_distance) };
    let best = prev_best.max(g.distance()).max(1.0);
    let bw = 300.0;
    let bx = lx - bw / 2.0;
    let fk = ui::ease_out_cubic((since - 0.3) / 1.0);
    text(big, "THIS FLIGHT", bx, by, 14, SLATE, Align::Left);
    if prev_best > 0.0 {
        text(big, &format!("BEST {:.1} m", prev_best), bx + bw, by, 14, GOLD_DEEP, Align::Right);
    }
    let fill = if new_best { GOLD } else if g.crashed { BERRY } else { PINE };
    ui::bar(bx, by + 8.0, bw, 16.0, g.distance() / best * fk, fill, Color::new(0.88, 0.82, 0.72, 1.0), PAPER_DEEP);
    if prev_best > 0.0 && prev_best < best {
        let mx = bx + bw * (prev_best / best);
        draw_line(mx, by + 3.0, mx, by + 29.0, 3.0, GOLD_DEEP);
        draw_triangle(vec2(mx - 6.0, by + 1.0), vec2(mx + 6.0, by + 1.0), vec2(mx, by + 8.0), GOLD_DEEP);
    } else if prev_best > 0.0 {
        let mx = bx + bw * (prev_best / best);
        draw_line(mx - 1.5, by + 3.0, mx - 1.5, by + 29.0, 3.0, GOLD_DEEP);
    }
    if tick >= 1.0 {
        let line = if new_best {
            if prev_best > 0.0 && prev_best < g.distance() - 0.01 {
                format!("+{:.1} m over your old best!", g.distance() - prev_best)
            } else {
                "your first record!".to_owned()
            }
        } else {
            format!("{:.1} m short of your best", (prev_best - g.distance()).max(0.0))
        };
        text(big, &line, lx, by + 52.0, 19, if new_best { GOLD_DEEP } else { INK }, Align::Center);
        // Where in town the dog came down.
        if let Some(d) = app.city.as_ref().and_then(|c| c.district_index(g.pos.x).map(|i| &c.districts[i])) {
            let place = if d.landed.is_empty() { format!("landed in {}, {}", title_case(&d.name), d.street) } else { format!("landed {}", d.landed) };
            text(big, &place, lx, by + 74.0, 14, BERRY, Align::Center);
        }
        if !new_best {
            let tip = coach_tip(app);
            let tw = ui::text_w(big, tip, 14, 1.0) + 44.0;
            ui::chip(&kit.chip, Rect::new(lx - tw / 2.0, by + 88.0, tw, 30.0), 0.4, Color::new(1.0, 0.97, 0.86, 1.0));
            ui::spr(&kit.ic_star, vec2(lx - tw / 2.0 + 17.0, by + 103.0), 22.0, 0.0, WHITE);
            text(big, tip, lx + 10.0, by + 108.0, 14, INK, Align::Center);
        }
    }

    // Right: stat chips, then the scoreboard.
    let sx = x + 404.0;
    let swd = w - 434.0;
    stat_chips(app, sx, y + 62.0, swd, since);
    let sy = y + 222.0;
    ui::spr(&kit.ic_trophy, vec2(sx + 10.0, sy - 5.0), 22.0, 0.0, WHITE);
    text(big, &app.pack.labels.scoreboard, sx + 26.0, sy, 16, INK, Align::Left);
    const ROWS: usize = 4;
    let first = my_rank.map_or(0, |r| r.saturating_sub(ROWS - 1).min(app.scores.len().saturating_sub(ROWS)));
    let mut ry = sy + 8.0;
    for (n, (i, sc)) in app.scores.iter().enumerate().skip(first).take(ROWS).enumerate() {
        let rk = ui::ease_out_cubic((since - 0.5 - n as f32 * 0.08) / 0.35);
        if rk <= 0.0 {
            break;
        }
        let rx = sx + 40.0 * (1.0 - rk);
        let mine = my_rank == Some(i);
        let cy = ry + 15.0;
        if mine {
            ui::chip(&kit.chip, Rect::new(rx - 6.0, ry + 1.0, swd + 12.0, 28.0), 0.35, Color::new(1.0, 0.9, 0.55, 1.0));
        }
        if i < 3 {
            ui::spr(&kit.medals[i], vec2(rx + 12.0, cy - 2.0), 22.0, 0.0, WHITE);
        } else {
            text(big, &(i + 1).to_string(), rx + 12.0, cy + 6.0, 16, SLATE, Align::Center);
        }
        blit(&app.sprites.dogs[sc.pilot.min(2)], vec2(rx + 42.0, cy - 2.0), 28.0, 0.0, false, WHITE);
        let pname = app.pack.pilots.get(sc.pilot).map(|p| p.name.as_str()).unwrap_or("?");
        text(big, pname, rx + 62.0, cy + 6.0, 17, INK, Align::Left);
        if sc.crashed {
            text(big, "splat", rx + swd - 74.0, cy + 5.0, 13, BERRY, Align::Right);
        }
        number_unit(big, &format!("{:.1}", sc.distance), "m", rx + swd, cy + 6.0, 18, 1.0, INK, Align::Right);
        if !mine {
            draw_line(rx, ry + 30.0, rx + swd, ry + 30.0, 1.0, fade(PAPER_DEEP, 0.35));
        }
        ry += 31.0;
    }
    if app.scores.is_empty() {
        text(big, "no flights yet", sx + swd / 2.0, sy + 40.0, 14, SLATE, Align::Center);
    }
    // The button: grey while the numbers settle, then green and breathing.
    let mut b = restart_button_rect();
    b.y += y - r.y;
    let armed = since > 0.5;
    let hover = armed && b.contains(mouse_position().into());
    let pressed = hover && is_mouse_button_down(MouseButton::Left);
    let bk = ui::ease_out_back((since - 0.35) / 0.35);
    if bk > 0.0 {
        let breathe = if armed && !pressed { 1.0 + 0.025 * (t * 4.0).sin() } else { 1.0 };
        let s = bk * breathe;
        let (bw, bh) = (b.w * s, b.h * s);
        let c = vec2(b.x + b.w / 2.0, b.y + b.h / 2.0 + if pressed { 4.0 } else { 0.0 });
        // ui_button: 300x100 svg, body x 8..292, y 6..78.
        let k = bw / 284.0;
        let tex = if armed { &kit.button } else { &kit.button_off };
        let tintc = if hover && !pressed { Color::new(1.08, 1.08, 1.08, 1.0) } else { WHITE };
        draw_texture_ex(tex, c.x - bw / 2.0 - 8.0 * k, c.y - bh / 2.0 - 6.0 * k, tintc, DrawTextureParams { dest_size: Some(vec2(300.0 * k, 100.0 * k)), ..Default::default() });
        ui::spr(&kit.ic_restart, vec2(c.x - bw * 0.30, c.y - 1.0), 34.0 * s, 0.0, WHITE);
        let label = app.pack.labels.restart.as_str();
        ui::text_painted(big, label, c.x + 16.0 * s, c.y + 11.0 * s, 32, s, PAPER, if armed { PINE_DEEP } else { Color::new(0.36, 0.40, 0.47, 1.0) }, 3.0, 0.0);
        text(big, "or press R", c.x, b.y + b.h + 22.0, 14, SLATE, Align::Center);
    }
}

/// The run's numbers as painted chips: icon, label, value. Values worth
/// celebrating sit on a gold pill.
fn stat_chips(app: &App, x: f32, y: f32, w: f32, since: f32) {
    let g = &app.game;
    let big = &app.display;
    let kit = &app.sprites.ui;
    let slide = hamsters_core::SLIDE_TIME * (1.25 - 0.25 * g.push);
    let jump_text = if g.jump_q >= hamsters_core::JUMP_PERFECT_FROM {
        "perfect".to_owned()
    } else if g.jump_q <= 0.0 {
        "missed".to_owned()
    } else if g.jump_t > 1.0 {
        format!("late {:.2}s", (g.jump_t - 1.0) * slide)
    } else {
        format!("early {:.2}s", (1.0 - g.jump_t) * slide)
    };
    let rows: [(&Texture2D, &str, String, f32); 6] = [
        (&kit.ic_push, "PUSH", format!("{:.0}%", g.push * 100.0), g.push),
        (&kit.ic_jump, "JUMP", jump_text, g.jump_q),
        (&kit.ic_ear, "FLAPS", format!("{}", g.flaps), (g.flaps as f32 / 30.0).clamp(0.0, 1.0)),
        (&kit.ic_gift, "PRESENTS", format!("{}", g.presents), (g.presents as f32 / 8.0).clamp(0.0, 1.0)),
        (&kit.ic_alt, "HEIGHT", format!("{:.0} m", g.max_height), (g.max_height / 160.0).clamp(0.0, 1.0)),
        (&kit.ic_flag, "DISTANCE", format!("{:.0} m", g.distance()), if g.distance() >= g.best_distance - 0.01 { 1.0 } else { 0.5 }),
    ];
    let (cw, ch, gap) = ((w - 10.0) / 2.0, 42.0, 8.0);
    for (i, (icon, lab, val, k)) in rows.iter().enumerate() {
        let pk = ui::ease_out_back((since - 0.3 - i as f32 * 0.07) / 0.3);
        if pk <= 0.0 {
            continue;
        }
        let cx = x + (i % 2) as f32 * (cw + 10.0);
        let cy = y + (i / 2) as f32 * (ch + gap);
        let (pw, ph) = (cw * pk, ch * pk);
        let r = Rect::new(cx + (cw - pw) / 2.0, cy + (ch - ph) / 2.0, pw, ph);
        ui::chip(&kit.chip, r, 0.45, WHITE);
        if pk < 0.9 {
            continue;
        }
        ui::spr(icon, vec2(cx + 22.0, cy + ch / 2.0), 30.0, 0.0, WHITE);
        text(big, lab, cx + 42.0, cy + 17.0, 12, SLATE, Align::Left);
        if *k >= 0.82 {
            gold_pill(big, val, cx + 50.0, cy + 36.0, 16, Align::Left, 1.0);
        } else {
            text(big, val, cx + 42.0, cy + 36.0, 18, if *k < 0.3 { BERRY } else { INK }, Align::Left);
        }
    }
}

/// The coach line for the weakest of the three skills.
fn coach_tip(app: &App) -> &str {
    let g = &app.game;
    let l = &app.pack.labels;
    let air_k = ((g.flaps as f32 / 40.0) + (g.presents as f32 / 10.0)).clamp(0.0, 1.0);
    let ks = [g.push, g.jump_q, air_k];
    let weakest = ks.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0);
    match weakest {
        0 => &l.tip_push,
        1 => &l.tip_jump,
        _ => &l.tip_air,
    }
}

/// The power gauge: the painted dial with a candy-cane needle. `live` while
/// the meter still swings.
fn power_dial(app: &App, cx: f32, cy: f32, value: f32, live: bool) {
    let big = &app.display;
    let kit = &app.sprites.ui;
    let t = get_time() as f32;
    let sweet = value > 0.82;
    let pulse = if sweet { 1.0 + 0.06 * (t * 10.0).sin() } else { 1.0 };
    let k = 0.82;
    if sweet {
        // A gold halo pulsing round the rim.
        let a = 0.35 + 0.25 * (t * 10.0).sin();
        for i in 0..3 {
            draw_arc(cx, cy, 48, 122.0 + i as f32 * 5.0, 180.0, 6.0, 180.0, fade(GOLD, a / (1.0 + i as f32)));
        }
    }
    draw_texture_ex(&kit.dial, cx - 160.0 * k, cy - 152.0 * k, WHITE, DrawTextureParams { dest_size: Some(vec2(320.0 * k, 176.0 * k)), ..Default::default() });
    let wob = if live { (t * 40.0).sin() * 0.006 } else { 0.0 };
    let v = (value + wob).clamp(0.0, 1.0);
    let rot = (v - 0.5) * std::f32::consts::PI;
    // ui_needle: 28x124 svg, pivot (14,110).
    draw_texture_ex(&kit.needle, cx - 14.0 * k, cy - 110.0 * k, WHITE, DrawTextureParams { dest_size: Some(vec2(28.0 * k, 124.0 * k)), rotation: rot, pivot: Some(vec2(cx, cy)), ..Default::default() });
    // Readout: where this push lands is the hero; the percent is the footnote.
    match app.aim_distance() {
        Some(d) => number_painted(big, &format!("~{:.0}", d), "m", cx, cy - 40.0, 28, pulse, INK, PAPER),
        None => number_painted(big, &format!("{:.0}", value * 100.0), "", cx, cy - 40.0, 28, pulse, INK, PAPER),
    }
    // The percent is a footnote left of the hub, where the needle seldom is.
    text(big, &format!("{:.0}%", value * 100.0), cx - 15.0, cy + 5.0, 14, SLATE, Align::Right);
    if sweet {
        // A gold ribbon pops over the dial.
        let w = ui::text_w(big, "SWEET SPOT!", 22, 1.0) + 30.0;
        ui::ribbon(kit.ribbon(Rib::Gold), vec2(cx, cy - 146.0), w * pulse, 36.0 * pulse, -0.03, WHITE);
        ui::text_painted(big, "SWEET SPOT!", cx, cy - 138.0, 22, pulse, PAPER, GOLD_DEEP, 2.5, -0.03);
    }
}

/// A key cap reading SPACE and a mouse with its button lit, side by side,
/// centred on `cx`; returns the total width. The input itself is drawn so
/// no one has to guess what "press" means.
fn key_caps(font: &Font, cx: f32, cy: f32, k: f32) -> f32 {
    let (kw, kh) = (62.0 * k, 24.0 * k);
    let gap = 8.0 * k;
    let mw = 18.0 * k;
    let total = kw + gap + mw;
    let x0 = cx - total / 2.0;
    // Key cap: a deep lip, paper cap, label.
    ui::rrect(x0, cy - kh / 2.0 + 3.0 * k, kw, kh, 6.0 * k, BROWN);
    ui::rrect(x0, cy - kh / 2.0, kw, kh, 6.0 * k, BROWN);
    ui::rrect(x0 + 2.0 * k, cy - kh / 2.0 + 2.0 * k, kw - 4.0 * k, kh - 4.0 * k, 4.5 * k, SNOW);
    ui::rrect(x0 + 4.0 * k, cy - kh / 2.0 + 3.0 * k, kw - 8.0 * k, kh * 0.3, 3.0 * k, WHITE);
    text_scaled(font, "SPACE", x0 + kw / 2.0, cy + 5.0 * k, 14, k, INK, Align::Center);
    // Mouse: a pill with the left button in berry.
    let mx = x0 + kw + gap;
    let mh = 26.0 * k;
    rounded(mx, cy - mh / 2.0 + 3.0 * k, mw, mh, 9.0 * k, BROWN);
    rounded(mx, cy - mh / 2.0, mw, mh, 9.0 * k, BROWN);
    rounded(mx + 2.0 * k, cy - mh / 2.0 + 2.0 * k, mw - 4.0 * k, mh - 4.0 * k, 7.0 * k, SNOW);
    rounded(mx + 2.5 * k, cy - mh / 2.0 + 2.5 * k, (mw - 5.0 * k) / 2.0, mh * 0.42, 5.0 * k, BERRY);
    draw_line(mx + mw / 2.0, cy - mh / 2.0 + 2.0 * k, mx + mw / 2.0, cy - 1.0 * k, 1.5 * k, BROWN);
    total
}

/// The jump cue over the sled on every ride: the key cap and the mouse with
/// TO JUMP beside them, so the second press is never a mystery; over the
/// last stretch it becomes a gold JUMP! shout that pulses with the glowing
/// deck.
fn jump_prompt(app: &App, t: f32) {
    let g = &app.game;
    let (font, big) = (&app.font, &app.display);
    let time = get_time() as f32;
    let hp = app.cam.to_screen(g.pos.x, g.pos.y);
    let r = PILOT_RADIUS * app.cam.scale();
    let sw = screen_width();
    let x = hp.x.clamp(170.0, sw - 170.0);
    let y = (hp.y - r * 4.0).max(80.0);
    // The plate is gone before the shout pops, so the two never overlap.
    let a = 1.0 - ((t - 0.70) / 0.07).clamp(0.0, 1.0);
    let window = ((t - 0.78) / 0.12).clamp(0.0, 1.0);
    if a > 0.0 {
        let label = "TO JUMP";
        let tw = ui::text_w(big, label, 24, 1.0);
        let caps_w = 62.0 + 8.0 + 18.0;
        let pop = ui::ease_out_back(t / 0.12) * a;
        let w = (caps_w + 14.0 + tw + 36.0) * pop;
        let h = 50.0 * pop;
        let bob = (time * 5.0).sin() * 3.0;
        let (px, py) = (x - w / 2.0, y - h / 2.0 + bob);
        if pop > 0.05 {
            ui::panel(&app.sprites.ui.panel, Rect::new(px, py, w, h), 0.42 * pop, WHITE);
            draw_triangle(vec2(x - 10.0, py + h + 3.0), vec2(x + 10.0, py + h + 3.0), vec2(x, py + h + 14.0), PAPER_DEEP);
            draw_triangle(vec2(x - 7.0, py + h + 1.0), vec2(x + 7.0, py + h + 1.0), vec2(x, py + h + 9.0), PAPER);
        }
        if pop > 0.6 {
            key_caps(font, px + 18.0 * pop + caps_w * pop / 2.0, py + h / 2.0, pop);
            text_scaled(big, label, px + (18.0 + caps_w + 14.0) * pop, py + h / 2.0 + 9.0 * pop, 24, pop, INK, Align::Left);
        }
    }
    if window > 0.0 {
        // JUMP! in gold, growing in, pulsing fast; the key cap stays under it.
        let pop = ease_out_back(window);
        let pulse = 1.0 + 0.08 * (time * 18.0).sin().abs();
        ui::text_painted(big, "JUMP!", x, y - 4.0, 52, pop * pulse, GOLD, GOLD_DEEP, 4.0, -0.05);
        key_caps(font, x, y + 22.0, 0.8 * pop);
    }
}

/// The ramp ride as a timing challenge: a glowing stretch of the real ramp
/// that turns gold right at the lip and berry on the coast beyond it.
fn jump_timer(app: &App, t: f32) {
    let cam = &app.cam;
    let time = get_time() as f32;
    let s = cam.scale();
    let n = 30;
    let end = 1.0 + hamsters_core::LIP_COAST;
    for i in 0..=n {
        let e = hamsters_core::JUMP_WINDOW_START + (end - hamsters_core::JUMP_WINDOW_START) * i as f32 / n as f32;
        let w = Game::slide_point(e);
        let p = cam.to_screen(w.x, w.y + 0.25);
        let k = i as f32 / n as f32;
        let gold = (0.97..=1.03).contains(&e);
        let late = e > 1.03;
        let pulse = 0.55 + 0.45 * (time * (if gold { 14.0 } else { 6.0 }) - k * 8.0).sin();
        let col = if gold { fade(GOLD, 0.95 * pulse) } else if late { fade(BERRY, 0.75 * pulse) } else { fade(WHITE, 0.4 * pulse) };
        let rad = (0.12 + 0.12 * k.min(0.85)) * s;
        draw_circle(p.x, p.y, rad + 1.5, fade(INK, col.a * 0.6));
        draw_circle(p.x, p.y, rad, col);
    }
    let _ = t;
}

/// Three painted pilot cards on the right: a big portrait medallion, the
/// name, the trait, three stat bars and the pilot's best. The chosen one is
/// gold-framed, lifted and glowing; each has a wax seal with its key.
/// `since` drives the slide-in, `out` (0..1) the exit.
fn picker(app: &App, since: f32, pilot_since: f32, out: f32) {
    let big = &app.display;
    let kit = &app.sprites.ui;
    let cards = picker_cards();
    let t = get_time() as f32;
    // Header ribbon.
    let hk = ui::ease_out_back((since - 0.05) / 0.35);
    let hx = cards[0].x + cards[0].w / 2.0 + out * 340.0;
    if hk > 0.0 {
        let title = "PICK YOUR PILOT";
        let tw = ui::text_w(big, title, 22, 1.0);
        ui::ribbon(kit.ribbon(Rib::Green), vec2(hx, 40.0), (tw + 40.0) * hk, 38.0 * hk, 0.0, WHITE);
        ui::text_painted(big, title, hx, 48.0, 22, hk, PAPER, PINE_DEEP, 2.5, 0.0);
    }
    for (i, h) in PILOTS.iter().enumerate() {
        let selected = i == app.pilot_idx;
        let ek = ui::ease_out_back((since - 0.12 - i as f32 * 0.08) / 0.4);
        if ek <= 0.0 {
            continue;
        }
        let mut r = cards[i];
        r.x += (1.0 - ek) * 320.0 + out * (340.0 + i as f32 * 40.0);
        let bump = if selected { (1.0 - (pilot_since / 0.35).clamp(0.0, 1.0)) * (pilot_since / 0.35 * std::f32::consts::PI).sin().max(0.0) } else { 0.0 };
        if selected {
            r.x -= 10.0;
            r.y -= 2.0;
            // Soft gold glow behind the chosen card.
            let glow = 0.5 + 0.5 * (t * 3.0).sin();
            for g in 0..4 {
                let e = 4.0 + g as f32 * 4.0;
                ui::rrect(r.x - e, r.y - e, r.w + 2.0 * e, r.h + 2.0 * e, 16.0 + e, fade(GOLD, (0.16 + 0.06 * glow) / (1.0 + g as f32)));
            }
        }
        let tex = if selected { &kit.panel_gold } else { &kit.panel };
        let dim = if selected { WHITE } else { Color::new(0.93, 0.93, 0.95, 1.0) };
        ui::panel(tex, r, 0.55, dim);
        if selected {
            ui::nine(&kit.snow_small, Rect::new(r.x + 18.0, r.y - 8.0, r.w - 30.0, 18.0), 30.0, 12.0, WHITE);
        }
        let m = vec2(r.x + 54.0, r.y + r.h / 2.0 + 2.0);
        medallion(app, i, m, 38.0 * (1.0 + 0.08 * bump), 116.0 * (1.0 + 0.12 * bump), !selected);
        // Key seal.
        ui::spr(&kit.seal_red, vec2(r.x + 12.0, r.y + 12.0), 38.0, -0.2, WHITE);
        ui::text_painted(big, &(i + 1).to_string(), r.x + 12.0, r.y + 19.0, 19, 1.0, PAPER, BERRY_DEEP, 1.8, -0.2);
        let (pname, word) = app.pack.pilots.get(i).map(|t| (t.name.as_str(), t.word.as_str())).unwrap_or((h.name, "far"));
        let nx = r.x + 104.0;
        text(big, pname, nx, r.y + 31.0, 26, if selected { INK } else { SLATE }, Align::Left);
        // Trait on a small coloured tag.
        let (light, deep) = pilot_colors(i);
        let ww = ui::text_w(big, word, 13, 1.0) + 14.0;
        let wx = r.x + r.w - 14.0 - ww;
        ui::rrect(wx - 1.5, r.y + 13.5, ww + 3.0, 21.0, 10.5, ui::shade(deep, 0.7));
        ui::rrect(wx, r.y + 15.0, ww, 18.0, 9.0, deep);
        ui::rrect(wx + 3.0, r.y + 16.5, ww - 6.0, 5.0, 2.5, fade(light, 0.6));
        text(big, word, wx + ww / 2.0, r.y + 29.0, 13, PAPER, Align::Center);
        // Stat bars grow in.
        let stats = [
            (&kit.ic_speed, ((h.launch_speed - 45.0) / 40.0).clamp(0.1, 1.0)),
            (&kit.ic_ear, (h.flap_power / 1.5).clamp(0.1, 1.0)),
            (&kit.ic_bounce, (h.bounce / 0.8).clamp(0.1, 1.0)),
        ];
        let gk = ui::ease_out_cubic((since - 0.35 - i as f32 * 0.08) / 0.6);
        for (j, (ic, v)) in stats.iter().enumerate() {
            let sy = r.y + 46.0 + j as f32 * 15.0;
            ui::spr(ic, vec2(nx + 6.0, sy), 16.0, 0.0, WHITE);
            ui::bar(nx + 20.0, sy - 4.0, 112.0, 8.0, v * gk, if selected { lerp(deep, light, 0.3) } else { ui::shade(lerp(deep, light, 0.3), 0.85) }, Color::new(0.87, 0.81, 0.70, 1.0), PAPER_DEEP);
        }
        let ly = r.y + r.h - 9.0;
        match app.pilot_best(i) {
            Some(b) => {
                ui::spr(&kit.ic_trophy, vec2(nx + 6.0, ly - 5.0), 17.0, 0.0, WHITE);
                text(big, &format!("best {:.0} m", b), nx + 18.0, ly, 14, INK, Align::Left);
            }
            None => text(big, "no record yet", nx, ly, 14, SLATE, Align::Left),
        }
        // Today's course: a gold star while it is still to fly.
        match app.today_flown(i) {
            Some(d) => text(big, &format!("today {:.0} m", d), r.x + r.w - 14.0, ly, 14, SLATE, Align::Right),
            None => {
                let tw = ui::text_w(big, "today", 14, 1.0);
                ui::spr(&kit.ic_star, vec2(r.x + r.w - 23.0 - tw, ly - 5.0), 17.0, 0.0, WHITE);
                text(big, "today", r.x + r.w - 14.0, ly, 14, GOLD_DEEP, Align::Right);
            }
        }
    }
}

/// Under the distance: how far to the next rung, or NEW BEST once passed,
/// plus the live scoreboard rank for half a second after it changes.
fn rank_line(app: &App, right_x: f32, y: f32, alpha: f32) {
    let g = &app.game;
    let t = get_time() as f32;
    if app.best_beaten {
        gold_pill(&app.display, "NEW BEST", right_x, y + 2.0, 18, Align::Right, alpha);
    } else if let Some((rank, d)) = app.target() {
        let near = app.ghost_flag_x().map_or(false, |fx| g.phase == Phase::Flying && (fx - g.pos.x).abs() < 15.0);
        let col = if near { lerp(SLATE, INK, 0.5 + 0.5 * (t * 8.0).sin()) } else { SLATE };
        text(&app.display, &format!("{:.0} m to #{}", (d - g.distance()).max(0.0), rank), right_x, y, 14, fade(col, alpha), Align::Right);
    }
    if app.rank_pulse > 0.0 && g.phase == Phase::Flying {
        let k = app.rank_pulse / 0.5;
        text_shout(&app.display, &format!("#{}", app.live_rank()), right_x + 14.0, y + 4.0, 22, 1.0 + 0.4 * k, GOLD, Align::Left);
    }
}

/// The wiggle streak only speaks up when it is close: "5 to WIGGLE POWER".
fn streak_line(app: &App, x: f32, y: f32) {
    let streak = app.game.flaps % hamsters_core::WIGGLE_STREAK;
    let left = hamsters_core::WIGGLE_STREAK - streak;
    if streak >= hamsters_core::WIGGLE_STREAK - 5 {
        text_shout(&app.font, &format!("{} to {}", left, app.pack.labels.wiggle_power.trim_end_matches('!')), x, y, 13, 1.0, GOLD, Align::Left);
    }
}

/// What the target plank says when there is no board yet.
const FIRST_FLIGHT: &str = "first flight!";

/// Before the push: the rung to beat this run, or a first-flight greeting.
fn target_plank(app: &App, right_x: f32, y: f32) {
    let kit = &app.sprites.ui;
    let (w, h) = (164.0, 54.0);
    let x = right_x - w;
    ui::panel(&kit.panel, Rect::new(x, y, w, h), 0.45, WHITE);
    ui::spr(&kit.ic_flag, vec2(x + 24.0, y + h / 2.0), 34.0, 0.0, WHITE);
    text(&app.display, "TARGET", x + 46.0, y + 21.0, 14, SLATE, Align::Left);
    match app.target() {
        Some((rank, d)) => gold_pill(&app.display, &format!("#{} · {:.0} m", rank, d), x + 50.0, y + 43.0, 17, Align::Left, 1.0),
        None => text(&app.display, FIRST_FLIGHT, x + 46.0, y + 42.0, 17, BERRY, Align::Left),
    }
}

/// Today's course: the three pilots with a dash or today's distance under
/// each, so the day's three flights read as one small session.
fn today_plank(app: &App, x: f32, y: f32) {
    let kit = &app.sprites.ui;
    let (w, h) = (164.0, 54.0);
    ui::panel(&kit.panel, Rect::new(x, y, w, h), 0.45, WHITE);
    ui::spr(&kit.ic_star, vec2(x + 20.0, y + 20.0), 22.0, 0.0, WHITE);
    text(&app.display, "TODAY", x + 8.0, y + 47.0, 13, SLATE, Align::Left);
    for i in 0..3 {
        let cx = x + 72.0 + i as f32 * 34.0;
        blit(&app.sprites.dogs[i], vec2(cx, y + 18.0), 30.0, 0.0, false, WHITE);
        match app.today_flown(i) {
            Some(d) => gold_pill(&app.display, &format!("{:.0}", d), cx, y + 46.0, 12, Align::Center, 1.0),
            None => ui::rrect(cx - 6.0, y + 40.0, 12.0, 3.0, 1.5, SLATE),
        }
    }
}

/// "THE RED MILE" -> "The Red Mile".
fn title_case(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c.flat_map(|ch| ch.to_lowercase())).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
