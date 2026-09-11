//! Screen-space UI: a paper Christmas card. Opaque cream plates with a thick
//! ink outline and a hard offset shadow, Lilita One for every number and
//! shout, Fredoka for labels. Every UI colour is defined in the palette below
//! and nowhere else.

use hamsters_core::{Game, Phase, PILOTS, PILOT_RADIUS, STOMP_REACH};
use macroquad::prelude::*;

use crate::draw::lerp;
use crate::sprites::blit;
use crate::App;

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
pub const SILVER: Color = Color::new(0.788, 0.812, 0.855, 1.0);
pub const AMBER: Color = Color::new(0.941, 0.651, 0.227, 1.0);
/// Secondary text on paper: solid slate, never ink at alpha (that goes muddy).
pub const SLATE: Color = Color::new(0.353, 0.380, 0.471, 1.0);
const PINE_DARK: Color = Color::new(0.11, 0.42, 0.24, 1.0);
const PINE_LIGHT: Color = Color::new(0.36, 0.76, 0.48, 1.0);
const SHADOW_ALPHA: f32 = 0.25;
/// Seconds the dog sits in the snow before the results card arrives.
pub const RESULT_HOLD: f32 = 1.6;
/// Shorter after a crash: the SPLAT banner and the sinking fill the beat.
pub const RESULT_HOLD_CRASH: f32 = 3.0;

pub fn result_hold(crashed: bool) -> f32 {
    if crashed { RESULT_HOLD_CRASH } else { RESULT_HOLD }
}
const OUTLINE_PX: f32 = 3.0;

pub fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

/// Ease-out-back: overshoots to about 1.1 then settles at 1.
pub fn ease_out_back(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.70158 * k * k * k + 1.70158 * k * k
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

/// Hard offset shadow under a plate.
fn shadow(x: f32, y: f32, w: f32, h: f32, r: f32, cut: f32, alpha: f32) {
    fan(&tag_pts(x, y + 4.0, w, h, r, cut), vec2(x + w / 2.0, y + h / 2.0 + 4.0), fade(INK, SHADOW_ALPHA * alpha));
}

/// A paper plate: shadow, ink outline, fill.
pub fn plate(x: f32, y: f32, w: f32, h: f32, r: f32, fill: Color, outline: Color, alpha: f32) {
    shadow(x, y, w, h, r, 0.0, alpha);
    rounded(x, y, w, h, r, fade(outline, alpha));
    let o = OUTLINE_PX;
    rounded(x + o, y + o, w - 2.0 * o, h - 2.0 * o, (r - o).max(1.0), fade(fill, alpha));
}

/// A gift tag: paper with the top-left corner cut and a punched hole.
/// Returns the hole centre.
pub fn gift_tag(x: f32, y: f32, w: f32, h: f32, alpha: f32) -> Vec2 {
    let (r, cut, o) = (10.0, 20.0, OUTLINE_PX);
    let c = vec2(x + w / 2.0, y + h / 2.0);
    shadow(x, y, w, h, r, cut, alpha);
    fan(&tag_pts(x, y, w, h, r, cut), c, fade(INK, alpha));
    fan(&tag_pts(x + o, y + o, w - 2.0 * o, h - 2.0 * o, r - o, cut - o), c, fade(PAPER, alpha));
    let hole = vec2(x + cut + 6.0, y + cut + 6.0);
    draw_circle(hole.x, hole.y, 7.5, fade(INK, alpha));
    draw_circle(hole.x, hole.y, 4.5, fade(SNOW, alpha));
    hole
}

/// One stamina pip: filled, or hollow snow when spent.
fn pip(x: f32, y: f32, w: f32, h: f32, fill: Option<Color>, alpha: f32) {
    rounded(x, y, w, h, 4.0, fade(INK, if fill.is_none() { 1.0 } else { alpha }));
    rounded(x + 2.0, y + 2.0, w - 4.0, h - 4.0, 2.5, fade(fill.unwrap_or(SNOW), alpha));
}

/// A wax seal: a scalloped disc with an ink edge.
fn seal(cx: f32, cy: f32, r: f32, color: Color, alpha: f32) {
    draw_circle(cx, cy + 4.0, r + 1.0, fade(INK, SHADOW_ALPHA * alpha));
    for pass in 0..2 {
        let (rad, col) = if pass == 0 { (r + 2.5, fade(INK, alpha)) } else { (r, fade(color, alpha)) };
        for i in 0..12 {
            let a = i as f32 * std::f32::consts::TAU / 12.0;
            draw_circle(cx + a.cos() * rad * 0.82, cy + a.sin() * rad * 0.82, rad * 0.3, col);
        }
        draw_circle(cx, cy, rad * 0.9, col);
    }
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

/// A pill button: pine with a light band on top and a dark lip below;
/// pressed drops four pixels and loses the lip.
fn button(r: Rect, label: &str, font: &Font, armed: bool, pressed: bool) {
    let drop = if pressed { 4.0 } else { 0.0 };
    let (x, y, w, h) = (r.x, r.y + drop, r.w, r.h);
    let rad = h / 2.0;
    let body = if armed { PINE } else { SILVER };
    if !pressed {
        rounded(x, y + 4.0, w, h + 2.0, rad, fade(INK, SHADOW_ALPHA));
        rounded(x, y + 6.0, w, h, rad, INK);
        rounded(x + OUTLINE_PX, y + 6.0, w - 2.0 * OUTLINE_PX, h - OUTLINE_PX, rad - 3.0, if armed { PINE_DARK } else { fade(INK, 0.5) });
    }
    rounded(x, y, w, h, rad, INK);
    rounded(x + OUTLINE_PX, y + OUTLINE_PX, w - 2.0 * OUTLINE_PX, h - 2.0 * OUTLINE_PX, rad - 3.0, body);
    rounded(x + 6.0, y + 5.0, w - 12.0, h * 0.4, rad - 6.0, if armed { PINE_LIGHT } else { fade(PAPER, 0.4) });
    text_shout(font, label, x + w / 2.0, y + h / 2.0 + 9.0, 26, 1.0, PAPER, Align::Center);
}

/// A postage stamp: a coloured rectangle with a perforated (paper-dotted)
/// edge and a label, rotated a few degrees.
fn stamp(cx: f32, cy: f32, w: f32, h: f32, rot: f32, color: Color, label: &str, font: &Font) {
    let (s, c) = (rot.sin(), rot.cos());
    let at = |p: Vec2| vec2(cx + p.x * c - p.y * s, cy + p.x * s + p.y * c);
    draw_rectangle_ex(cx, cy + 4.0, w + 8.0, h + 8.0, DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: rot, color: fade(INK, SHADOW_ALPHA) });
    draw_rectangle_ex(cx, cy, w + 8.0, h + 8.0, DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: rot, color: INK });
    draw_rectangle_ex(cx, cy, w + 2.0, h + 2.0, DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: rot, color: PAPER });
    draw_rectangle_ex(cx, cy, w - 4.0, h - 4.0, DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: rot, color });
    let step = 9.0;
    let nx = (w / step) as i32;
    let ny = (h / step) as i32;
    for i in 0..=nx {
        let px = -w / 2.0 + i as f32 * w / nx as f32;
        for py in [-h / 2.0 - 1.0, h / 2.0 + 1.0] {
            let p = at(vec2(px, py));
            draw_circle(p.x, p.y, 3.0, PAPER);
        }
    }
    for j in 1..ny {
        let py = -h / 2.0 + j as f32 * h / ny as f32;
        for px in [-w / 2.0 - 1.0, w / 2.0 + 1.0] {
            let p = at(vec2(px, py));
            draw_circle(p.x, p.y, 3.0, PAPER);
        }
    }
    let d = measure_text(label, Some(font), 22, 1.0);
    let o = at(vec2(-d.width / 2.0, d.height / 2.0));
    for (dx, dy) in [(2.0, 0.0), (-2.0, 0.0), (0.0, 2.0), (0.0, -2.0)] {
        draw_text_ex(label, o.x + dx, o.y + dy, TextParams { font: Some(font), font_size: 22, rotation: rot, color: INK, ..Default::default() });
    }
    draw_text_ex(label, o.x, o.y, TextParams { font: Some(font), font_size: 22, rotation: rot, color: PAPER, ..Default::default() });
}

/// Rank badge: gold, silver and ice for the podium, snow below it.
fn rank_circle(cx: f32, cy: f32, rank: usize, font: &Font) {
    let fill = match rank {
        1 => GOLD,
        2 => SILVER,
        3 => ICE,
        _ => SNOW,
    };
    draw_circle(cx, cy, 12.0, INK);
    draw_circle(cx, cy, 9.5, fill);
    text(font, &rank.to_string(), cx, cy + 5.0, 14, INK, Align::Center);
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
    rounded(px - 2.0, py - 2.0, pw + 4.0, ph + 4.0, ph / 2.0 + 2.0, fade(INK, alpha));
    rounded(px, py, pw, ph, ph / 2.0, fade(GOLD, alpha));
    text(font, s, x, y, size, fade(INK, alpha), align);
}

/// A small pennant glyph: pole and a triangular flag, `h` tall.
fn pennant_glyph(x: f32, y: f32, h: f32, color: Color) {
    draw_line(x, y, x, y - h, 3.0, INK);
    draw_triangle(vec2(x + 1.0, y - h), vec2(x + h * 0.7 + 1.0, y - h * 0.72), vec2(x + 1.0, y - h * 0.44), INK);
    draw_triangle(vec2(x + 2.0, y - h + 3.0), vec2(x + h * 0.7 - 4.0, y - h * 0.72), vec2(x + 2.0, y - h * 0.44 - 3.0), color);
}

/// Upper half-disc with a flat base strip, for the power dial.
fn half_disc(cx: f32, cy: f32, rad: f32, base: f32, col: Color) {
    let n = 40;
    for i in 0..n {
        let a0 = std::f32::consts::PI * i as f32 / n as f32;
        let a1 = std::f32::consts::PI * (i + 1) as f32 / n as f32;
        draw_triangle(vec2(cx, cy), vec2(cx - a0.cos() * rad, cy - a0.sin() * rad), vec2(cx - a1.cos() * rad, cy - a1.sin() * rad), col);
    }
    draw_rectangle(cx - rad, cy, 2.0 * rad, base, col);
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

fn picker_cards() -> [Rect; 3] {
    let (w, h, gap) = (208.0, 74.0, 12.0);
    let x = screen_width() - w - 16.0;
    [0, 1, 2].map(|i| Rect::new(x, 104.0 + i as f32 * (h + gap), w, h))
}

pub fn picker_hit(p: Vec2) -> Option<usize> {
    picker_cards().iter().position(|r| r.contains(p))
}

/// The results postcard, centred.
fn results_rect() -> Rect {
    let (sw, sh) = (screen_width(), screen_height());
    let (w, h) = (660.0, 484.0);
    Rect::new((sw - w) / 2.0, (sh - h) / 2.0, w, h)
}

/// Where the Restart button sits on the results card (for hit-testing).
pub fn restart_button_rect() -> Rect {
    let r = results_rect();
    let (w, h) = (220.0, 50.0);
    let cx = r.x + 330.0 + 150.0;
    Rect::new(cx - w / 2.0, r.y + r.h - h - 34.0, w, h)
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

pub fn render(app: &App) {
    let g = &app.game;
    let sw = screen_width();
    let sh = screen_height();
    let aiming = matches!(g.phase, Phase::AimPower { .. } | Phase::Sliding { .. });
    // The flight HUD stays up through the quiet hold after landing.
    let holding = g.phase == Phase::Landed && (app.landed_for) < result_hold(g.crashed);
    if !aiming && (g.phase != Phase::Landed || holding) {
        flight_hud(app);
    }
    match g.phase {
        Phase::AimPower { meter } => {
            picker(app);
            power_dial(app, sw / 2.0, sh - 52.0, Game::meter_value(meter), true);
            target_plank(app, sw / 2.0 - 148.0, sh - 84.0);
            today_plank(app, sw / 2.0 + 148.0, sh - 84.0);
            if !app.taught(crate::TEACH_PUSH) {
                let hint = "space or click to lock the push";
                let d = measure_text(hint, Some(&app.font), 13, 1.0);
                plate(sw / 2.0 - d.width / 2.0 - 12.0, sh - 28.0, d.width + 24.0, 24.0, 8.0, PAPER, INK, 1.0);
                text(&app.font, hint, sw / 2.0, sh - 11.0, 13, INK, Align::Center);
            }
        }
        Phase::Sliding { power, t, jump } => {
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
    match g.phase {
        Phase::Flying => {
            let hp = app.cam.to_screen(g.pos.x, g.pos.y);
            let r = PILOT_RADIUS * app.cam.scale();
            let y = (hp.y + r * 3.6).min(screen_height() - 40.0);
            let x = hp.x.clamp(140.0, screen_width() - 140.0);
            if g.botched {
                // The fall after a missed press: the banner says MISSED THE
                // JUMP; this line under it says exactly what was missing.
                text_shout(big, "PRESS SPACE OR CLICK AT THE LIP", screen_width() / 2.0, 236.0, 20, pulse, PAPER, Align::Center);
            } else if g.flaps == 0 && g.flight_time > 0.6 && g.flight_time < 4.0 && !app.taught(crate::TEACH_FLAP) {
                text_shout(big, "PRESS TO FLAP", x, y, 26, pulse, PAPER, Align::Center);
            } else if app.stomp_teachable() && g.vel.y < 0.0 && g.pos.y < PILOT_RADIUS + STOMP_REACH + 4.0 && g.pos.y > PILOT_RADIUS + 0.5 {
                text_shout(big, "PRESS TO STOMP", x, y, 26, pulse, PAPER, Align::Center);
            }
        }
        _ => {}
    }
}

/// Distance gift tag, presents seal and the height ruler.
fn flight_hud(app: &App) {
    let g = &app.game;
    let (font, big) = (&app.font, &app.display);
    let sw = screen_width();
    let flying = g.phase == Phase::Flying;
    // Entry: the tag drops in over 140 ms after the lip. Paper is opaque:
    // plates only ease to 90% while flying.
    let enter = if flying { ease_out_back(g.flight_time / 0.14) } else { 1.0 };
    let alpha = if flying { 0.9 + 0.1 * (1.0 - (g.flight_time / 0.5).min(1.0)) } else { 1.0 };
    let (x, w, h) = (14.0, 300.0, if flying { 84.0 } else { 60.0 });
    let y = 14.0 - 30.0 * (1.0 - enter);
    gift_tag(x, y, w, h, alpha);
    let dp = 1.0 + 0.15 * (app.dist_pulse * std::f32::consts::PI).sin().max(0.0);
    number_unit(big, &format!("{:.1}", g.distance()), "m", x + 46.0, y + 44.0, 38, dp, fade(INK, alpha), Align::Left);
    rank_line(app, x + w - 14.0, y + 26.0, alpha);
    if flying {
        let left = ((g.stamina / hamsters_core::FLAP_COST).floor() as i32).clamp(0, 12);
        let blink = (get_time() as f32 * 6.0 * std::f32::consts::TAU).sin() > 0.0;
        for i in 0..12 {
            let fill = if i < left {
                Some(if left < 3 {
                    if blink { BERRY } else { PAPER }
                } else if left < 6 {
                    AMBER
                } else {
                    PINE
                })
            } else {
                None
            };
            pip(x + 46.0 + i as f32 * 14.0, y + 56.0, 11.0, 17.0, fill, alpha);
        }
        streak_line(app, x + 46.0, y + h + 18.0);
    }
    // Presents: wax seal with the count on a small tag beside it.
    let pulse = 1.0 + 0.35 * (app.counter_pulse * std::f32::consts::PI).sin().max(0.0);
    let (cx, cy) = (sw / 2.0 - 44.0, 38.0 - 30.0 * (1.0 - enter));
    plate(cx + 14.0, cy - 20.0, 78.0, 40.0, 8.0, PAPER, INK, alpha);
    seal(cx, cy, 22.0 * pulse, PINE, alpha);
    blit(&app.sprites.presents[0], vec2(cx, cy), 26.0 * pulse, 0.1, false, fade(WHITE, alpha));
    text_scaled(big, &g.presents.to_string(), cx + 52.0, cy + 10.0, 28, pulse, fade(INK, alpha), Align::Center);
    let _ = font;
    if flying {
        height_ruler(app, alpha);
    }
}

/// A vertical ruler on the right edge with the milestone heights on paper
/// chips, and a paper arrow sliding along it. Hidden below 60 m.
fn height_ruler(app: &App, alpha: f32) {
    let alt = app.game.altitude();
    let show = ((alt - 50.0) / 10.0).clamp(0.0, 1.0);
    if show <= 0.0 {
        return;
    }
    let a = alpha * show;
    let (font, big) = (&app.font, &app.display);
    let sw = screen_width();
    let sh = screen_height();
    let x = sw - 26.0 + 30.0 * (1.0 - show);
    let (top, bottom) = (140.0, sh - 96.0);
    let k = |h: f32| (h / 720.0).clamp(0.0, 1.0).sqrt();
    let y_at = |h: f32| bottom - k(h) * (bottom - top);
    draw_line(x, top, x, bottom, 4.0, fade(INK, a));
    let arrow_y = y_at(alt);
    for (h, _) in hamsters_core::MILESTONES.iter() {
        let y = y_at(*h);
        draw_line(x - 10.0, y, x + 6.0, y, 3.0, fade(INK, a));
        // Milestone heights on small paper chips so they read on any sky;
        // a chip steps aside while the altitude arrow covers it.
        if (y - arrow_y).abs() < 26.0 {
            continue;
        }
        let (cw, ch) = (36.0, 18.0);
        rounded(x - 12.0 - cw, y - ch / 2.0, cw, ch, 4.0, fade(INK, a));
        rounded(x - 10.0 - cw, y - ch / 2.0 + 2.0, cw - 4.0, ch - 4.0, 2.5, fade(PAPER, a));
        text(font, &format!("{:.0}", h), x - 12.0 - cw / 2.0, y + 5.0, 13, fade(INK, a), Align::Center);
    }
    let y = arrow_y;
    let (w, h) = (66.0, 28.0);
    let tx = x - 22.0 - w;
    plate(tx, y - h / 2.0, w, h, 8.0, PAPER, INK, a);
    draw_triangle(vec2(tx + w - 2.0, y - 9.0), vec2(tx + w - 2.0, y + 9.0), vec2(tx + w + 11.0, y), fade(INK, a));
    draw_triangle(vec2(tx + w - 2.0, y - 4.0), vec2(tx + w - 2.0, y + 4.0), vec2(tx + w + 5.0, y), fade(PAPER, a));
    number_unit(big, &format!("{:.0}", alt), "m", tx + w / 2.0 - 2.0, y + 7.0, 19, 1.0, fade(INK, a), Align::Center);
}

/// Active boosts, bottom-left: an icon on a small tag with a draining bar.
fn boost_bars(app: &App) {
    let g = &app.game;
    let sh = screen_height();
    let mut by = sh - 40.0;
    for (tex, frac, col) in [(&app.sprites.wings, g.glide / (3.5 * g.pilot.glide_bonus), ICE), (&app.sprites.firework, g.rocket / 1.2, AMBER)] {
        if frac <= 0.0 {
            continue;
        }
        let frac = frac.clamp(0.0, 1.0);
        plate(16.0, by - 18.0, 176.0, 36.0, 8.0, PAPER, INK, 0.85);
        blit(tex, vec2(40.0, by), 34.0, 0.0, false, WHITE);
        rounded(62.0, by - 7.0, 116.0, 14.0, 5.0, INK);
        rounded(64.0, by - 5.0, (112.0 * frac).max(6.0), 10.0, 3.5, col);
        by -= 44.0;
    }
}

/// The results postcard: stamp header, portrait, distance, report card,
/// ruled scoreboard and the restart button.
fn results(app: &App) {
    let g = &app.game;
    let (font, big) = (&app.font, &app.display);
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;
    // Nothing until the hold is over: the player gets the dog and the snow first.
    let since = app.landed_for - result_hold(g.crashed);
    if since < 0.0 {
        return;
    }
    let enter = ease_out_back(since / 0.14);
    draw_rectangle(0.0, 0.0, sw, sh, fade(INK, 0.45 * since.min(0.2) / 0.2));
    let r = results_rect();
    let (x, y, w, h) = (r.x, r.y + 40.0 * (1.0 - enter), r.w, r.h);
    let new_best = g.distance() >= g.best_distance - 0.01 && g.distance() > 0.0;
    if new_best {
        for i in 0..16 {
            let a = i as f32 * std::f32::consts::TAU / 16.0 + t * 0.3;
            let c = vec2(sw / 2.0, y + h / 2.0);
            draw_triangle(c, c + vec2(a.cos(), a.sin()) * 520.0, c + vec2((a + 0.09).cos(), (a + 0.09).sin()) * 520.0, fade(GOLD, 0.10));
        }
    }
    plate(x, y, w, h, 14.0, PAPER, INK, 1.0);
    // A record gets the gold stamp; the outcome becomes a sub-line.
    let outcome = if g.crashed { &app.pack.labels.crash } else { &app.pack.labels.flight_over };
    if new_best {
        stamp(x + 92.0, y + 10.0, 150.0, 42.0, -6.0f32.to_radians(), GOLD, "NEW BEST!", big);
        text(font, outcome, x + 92.0, y + 52.0, 13, if g.crashed { BERRY } else { SLATE }, Align::Center);
    } else {
        stamp(x + 92.0, y + 10.0, 150.0, 42.0, -6.0f32.to_radians(), BERRY, outcome, big);
    }
    // Left: the pilot as a round stamp and this run's distance, big.
    let lx = x + 160.0;
    draw_circle(lx, y + 118.0, 54.0, INK);
    draw_circle(lx, y + 118.0, 50.0, if new_best { GOLD } else { SNOW });
    let portrait = if new_best { &app.sprites.dogs_heads[app.pilot_idx][6] } else { &app.sprites.dogs[app.pilot_idx] };
    blit(portrait, vec2(lx, y + 120.0), 90.0, 0.04, false, WHITE);
    let tick = (since / 1.1).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - tick).powi(3);
    number_unit(big, &format!("{:.1}", g.distance() * ease), "m", lx, y + 240.0, 68, 1.0, INK, Align::Center);
    if tick >= 1.0 {
        if new_best {
            if app.best_before > 0.0 && app.best_before < g.distance() - 0.01 {
                text(font, &format!("beats your {:.1} m", app.best_before), lx, y + 268.0, 14, SLATE, Align::Center);
            }
        } else {
            let short = (app.best_before - g.distance()).max(0.0);
            text(font, &format!("{:.1} m short of your best", short), lx, y + 268.0, 14, SLATE, Align::Center);
        }
        report_card(app, x + 44.0, y + 302.0, 232.0, !new_best);
    }
    // Right: the scoreboard on ruled lines.
    let sx = x + 330.0;
    let sw_ = w - 360.0;
    text(font, &app.pack.labels.scoreboard, sx, y + 42.0, 13, SLATE, Align::Left);
    // Six rows; the window slides down so this run's row is always in it.
    const ROWS: usize = 6;
    let my_rank = app.scores.iter().position(|sc| (sc.distance - g.distance()).abs() < 0.01 && sc.pilot == app.pilot_idx);
    let first = my_rank.map_or(0, |r| r.saturating_sub(ROWS - 1).min(app.scores.len().saturating_sub(ROWS)));
    let mut ry = y + 56.0;
    for (i, sc) in app.scores.iter().enumerate().skip(first).take(ROWS) {
        let mine = my_rank == Some(i);
        if mine {
            rounded(sx - 8.0, ry, sw_ + 16.0, 32.0, 8.0, fade(GOLD, 0.22));
        }
        let cy = ry + 16.0;
        rank_circle(sx + 14.0, cy, i + 1, big);
        blit(&app.sprites.dogs[sc.pilot.min(2)], vec2(sx + 46.0, cy - 2.0), 24.0, 0.0, false, WHITE);
        let pname = app.pack.pilots.get(sc.pilot).map(|p| p.name.as_str()).unwrap_or("?");
        text(font, pname, sx + 66.0, cy + 5.0, 14, INK, Align::Left);
        if sc.crashed {
            text(font, "splat", sx + sw_ - 78.0, cy + 5.0, 13, BERRY, Align::Right);
        }
        number_unit(big, &format!("{:.1}", sc.distance), "m", sx + sw_, cy + 6.0, 17, 1.0, INK, Align::Right);
        draw_line(sx - 4.0, ry + 32.0, sx + sw_ + 4.0, ry + 32.0, 1.0, fade(INK, 0.15));
        ry += 34.0;
    }
    if app.scores.is_empty() {
        text(font, "no flights yet", sx + sw_ / 2.0, y + 100.0, 14, SLATE, Align::Center);
    }
    // A short board leaves room: the ghost of the best run fills it.
    if app.scores.len() < 4 && !app.best_path.path.is_empty() {
        if let Some(last) = app.best_path.path.last() {
            let gy = ry + 12.0;
            blit(&app.sprites.dogs_heads[app.best_path.pilot.min(2)][0], vec2(sx + 46.0, gy + 12.0), 26.0, 0.0, false, WHITE);
            text(font, "ghost", sx + 66.0, gy + 18.0, 13, SLATE, Align::Left);
            number_unit(big, &format!("{:.1}", last.0), "m", sx + sw_, gy + 19.0, 15, 1.0, SLATE, Align::Right);
        }
    }
    trend_strip(app, sx, y + 274.0, sw_);
    // Restart button, armed once the numbers have settled.
    let mut b = restart_button_rect();
    b.y += 40.0 * (1.0 - enter);
    let armed = since > 0.5;
    let pressed = armed && is_mouse_button_down(MouseButton::Left) && b.contains(mouse_position().into());
    button(b, &app.pack.labels.restart, big, armed, pressed);
    text(font, "or press R", b.x + b.w / 2.0, b.y + b.h + 20.0, 13, SLATE, Align::Center);
}

/// The power gauge: a paper half-dial with flat wedges, a candy-striped
/// sweet spot and a candy-cane needle. `live` while the meter still swings.
fn power_dial(app: &App, cx: f32, cy: f32, value: f32, live: bool) {
    let (font, big) = (&app.font, &app.display);
    let r = 96.0;
    let base = 24.0;
    let t = get_time() as f32;
    let sweet = value > 0.82;
    let pulse = if sweet { 1.0 + 0.06 * (t * 10.0).sin() } else { 1.0 };
    half_disc(cx, cy + 4.0, r + 24.0, base, fade(INK, SHADOW_ALPHA));
    half_disc(cx, cy, r + 24.0, base, if sweet { lerp(INK, GOLD, 0.5 + 0.5 * (t * 10.0).sin()) } else { INK });
    half_disc(cx, cy, r + 21.0, base - 3.0, PAPER);
    // Wedges from left (0) to right (1) along the upper half.
    let wedge = |a0: f32, a1: f32, col: Color, rad: f32, wd: f32| {
        let n = 16;
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
    wedge(0.0, 1.0, INK, r + 2.0, 30.0);
    wedge(0.0, 0.5, PINE, r - 1.0, 24.0);
    wedge(0.5, 0.82, AMBER, r - 1.0, 24.0);
    for i in 0..12 {
        let (a0, a1) = (0.82 + 0.18 * i as f32 / 12.0, 0.82 + 0.18 * (i + 1) as f32 / 12.0);
        wedge(a0, a1, if i % 2 == 0 { GOLD } else { PAPER }, r - 1.0, 24.0);
    }
    for k in [0.5, 0.82] {
        let a = std::f32::consts::PI * (1.0 - k);
        draw_line(cx + a.cos() * (r + 2.0), cy - a.sin() * (r + 2.0), cx + a.cos() * (r - 28.0), cy - a.sin() * (r - 28.0), 3.0, INK);
    }
    // Candy-cane needle: ink edge, paper body, berry stripes.
    let wob = if live { (t * 40.0).sin() * 0.006 } else { 0.0 };
    let a = std::f32::consts::PI * (1.0 - (value + wob).clamp(0.0, 1.0));
    let dir = vec2(a.cos(), -a.sin());
    let tip = vec2(cx, cy) + dir * (r - 6.0);
    draw_line(cx, cy, tip.x, tip.y, 12.0, INK);
    draw_circle(tip.x, tip.y, 6.0, INK);
    draw_line(cx, cy, tip.x, tip.y, 7.0, PAPER);
    draw_circle(tip.x, tip.y, 3.5, PAPER);
    let len = r - 6.0;
    let mut d = 6.0;
    while d + 8.0 < len {
        let p0 = vec2(cx, cy) + dir * d;
        let p1 = vec2(cx, cy) + dir * (d + 8.0);
        draw_line(p0.x, p0.y, p1.x, p1.y, 7.0, BERRY);
        d += 16.0;
    }
    draw_circle(cx, cy, 13.0, INK);
    draw_circle(cx, cy, 9.0, PAPER);
    // Readout: where this push lands is the hero; the percent is the footnote.
    match app.aim_distance() {
        Some(d) => number_unit(big, &format!("~{:.0}", d), "m", cx, cy - 44.0, 34, pulse, INK, Align::Center),
        None => text_scaled(big, &format!("{:.0}", value * 100.0), cx, cy - 44.0, 34, pulse, INK, Align::Center),
    }
    if sweet {
        text_shout(font, "SWEET SPOT", cx, cy + 18.0, 15, 1.0, GOLD, Align::Center);
    } else {
        text(font, &format!("{:.0}% POWER", value * 100.0), cx, cy + 18.0, 15, SLATE, Align::Center);
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
    // Key cap: ink shadow plate, paper cap, label.
    rounded(x0, cy - kh / 2.0 + 3.0 * k, kw, kh, 6.0 * k, INK);
    rounded(x0, cy - kh / 2.0, kw, kh, 6.0 * k, INK);
    rounded(x0 + 2.0 * k, cy - kh / 2.0 + 2.0 * k, kw - 4.0 * k, kh - 4.0 * k, 4.5 * k, PAPER);
    text_scaled(font, "SPACE", x0 + kw / 2.0, cy + 5.0 * k, 14, k, INK, Align::Center);
    // Mouse: a pill with the left button in berry.
    let mx = x0 + kw + gap;
    let mh = 26.0 * k;
    rounded(mx, cy - mh / 2.0 + 3.0 * k, mw, mh, 9.0 * k, INK);
    rounded(mx, cy - mh / 2.0, mw, mh, 9.0 * k, INK);
    rounded(mx + 2.0 * k, cy - mh / 2.0 + 2.0 * k, mw - 4.0 * k, mh - 4.0 * k, 7.0 * k, PAPER);
    rounded(mx + 2.5 * k, cy - mh / 2.0 + 2.5 * k, (mw - 5.0 * k) / 2.0, mh * 0.42, 5.0 * k, BERRY);
    draw_line(mx + mw / 2.0, cy - mh / 2.0 + 2.0 * k, mx + mw / 2.0, cy - 1.0 * k, 1.5 * k, INK);
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
        // Paper plate: [SPACE] [mouse]  TO JUMP
        let label = "TO JUMP";
        let d = measure_text(label, Some(big), 22, 1.0);
        let caps_w = 62.0 + 8.0 + 18.0;
        let w = caps_w + 14.0 + d.width + 36.0;
        let h = 46.0;
        let bob = (time * 5.0).sin() * 3.0;
        let (px, py) = (x - w / 2.0, y - h / 2.0 + bob);
        plate(px, py, w, h, 10.0, PAPER, INK, a);
        if a > 0.5 {
            key_caps(font, px + 18.0 + caps_w / 2.0, py + h / 2.0, 1.0);
            text(big, label, px + 18.0 + caps_w + 14.0, py + h / 2.0 + 8.0, 22, INK, Align::Left);
            // A little tail pointing at the dog.
            draw_triangle(vec2(x - 8.0, py + h), vec2(x + 8.0, py + h), vec2(x, py + h + 10.0), INK);
        }
    }
    if window > 0.0 {
        // JUMP! in gold, growing in, pulsing fast; the key cap stays under it.
        let pop = ease_out_back(window);
        let pulse = 1.0 + 0.08 * (time * 18.0).sin().abs();
        text_shout(big, "JUMP!", x, y - 6.0, 44, pop * pulse, GOLD, Align::Center);
        key_caps(font, x, y + 18.0, 0.8 * pop);
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

/// Three paper cards on the right: portrait, name, trait, best. The chosen
/// one lifts and gets a gold outline; each has a berry tab with its key.
fn picker(app: &App) {
    let (font, big) = (&app.font, &app.display);
    let cards = picker_cards();
    // Title on an ink chip so it reads over the sun.
    let title = "PICK YOUR PILOT   1 / 2 / 3";
    let d = measure_text(title, Some(font), 14, 1.0);
    let (tx, ty) = (cards[0].x + cards[0].w, cards[0].y - 22.0);
    rounded(tx - d.width - 20.0, ty - 16.0, d.width + 20.0, 24.0, 7.0, INK);
    text(font, title, tx - 10.0, ty + 1.0, 14, PAPER, Align::Right);
    for (i, h) in PILOTS.iter().enumerate() {
        let selected = i == app.pilot_idx;
        let mut r = cards[i];
        let alpha = if selected { 1.0 } else { 0.85 };
        if selected {
            r.y -= 8.0;
            rounded(r.x - 4.0, r.y - 4.0, r.w + 8.0, r.h + 8.0, 16.0, GOLD);
        }
        plate(r.x, r.y, r.w, r.h, 12.0, PAPER, INK, alpha);
        // Key tab.
        rounded(r.x + 10.0, r.y - 8.0, 24.0, 28.0, 5.0, INK);
        rounded(r.x + 13.0, r.y - 5.0, 18.0, 22.0, 3.0, BERRY);
        text(big, &(i + 1).to_string(), r.x + 22.0, r.y + 12.0, 15, PAPER, Align::Center);
        blit(&app.sprites.dogs[i], vec2(r.x + 60.0, r.y + r.h / 2.0), 52.0, 0.0, false, WHITE);
        let (pname, word) = app.pack.pilots.get(i).map(|t| (t.name.as_str(), t.word.as_str())).unwrap_or((h.name, "far"));
        text(big, pname, r.x + 92.0, r.y + 27.0, 22, if selected { INK } else { SLATE }, Align::Left);
        text(font, word, r.x + 92.0, r.y + 46.0, 14, SLATE, Align::Left);
        if let Some(b) = app.pilot_best(i) {
            text(font, &format!("best {:.0} m", b), r.x + r.w - 12.0, r.y + 46.0, 13, SLATE, Align::Right);
        }
        // Today's course on its own line: a gold dot while it is still to fly.
        match app.today_flown(i) {
            Some(d) => text(font, &format!("today {:.0} m", d), r.x + 92.0, r.y + 64.0, 13, INK, Align::Left),
            None => {
                draw_circle(r.x + 97.0, r.y + 59.5, 4.5, INK);
                draw_circle(r.x + 97.0, r.y + 59.5, 3.0, GOLD);
                text(font, "today's course", r.x + 106.0, r.y + 64.0, 13, SLATE, Align::Left);
            }
        }
    }
}

/// Under the distance: how far to the next rung, or NEW BEST once passed,
/// plus the live scoreboard rank for half a second after it changes.
fn rank_line(app: &App, right_x: f32, y: f32, alpha: f32) {
    let g = &app.game;
    let font = &app.font;
    let t = get_time() as f32;
    if app.best_beaten {
        gold_pill(&app.display, "NEW BEST", right_x, y + 2.0, 18, Align::Right, alpha);
    } else if let Some((rank, d)) = app.target() {
        let near = app.ghost_flag_x().map_or(false, |fx| g.phase == Phase::Flying && (fx - g.pos.x).abs() < 15.0);
        let col = if near { lerp(SLATE, INK, 0.5 + 0.5 * (t * 8.0).sin()) } else { SLATE };
        text(font, &format!("{:.0} m to #{}", (d - g.distance()).max(0.0), rank), right_x, y, 14, fade(col, alpha), Align::Right);
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
    let (w, h) = (156.0, 48.0);
    let x = right_x - w;
    plate(x, y, w, h, 10.0, PAPER, INK, 1.0);
    text(&app.font, "TARGET", x + 14.0, y + 19.0, 13, SLATE, Align::Left);
    match app.target() {
        Some((rank, d)) => gold_pill(&app.display, &format!("#{} · {:.0} m", rank, d), x + 18.0, y + 40.0, 17, Align::Left, 1.0),
        None => {
            pennant_glyph(x + 16.0, y + 42.0, 20.0, BERRY);
            text(&app.font, FIRST_FLIGHT, x + 34.0, y + 39.0, 14, INK, Align::Left);
        }
    }
}

/// Today's course: the three pilots with a dash or today's distance under
/// each, so the day's three flights read as one small session.
fn today_plank(app: &App, x: f32, y: f32) {
    let (w, h) = (156.0, 48.0);
    plate(x, y, w, h, 10.0, PAPER, INK, 1.0);
    text(&app.font, "TODAY", x + 14.0, y + 19.0, 13, SLATE, Align::Left);
    for i in 0..3 {
        let cx = x + 70.0 + i as f32 * 32.0;
        blit(&app.sprites.dogs[i], vec2(cx, y + 15.0), 24.0, 0.0, false, WHITE);
        match app.today_flown(i) {
            Some(d) => gold_pill(&app.display, &format!("{:.0}", d), cx, y + 41.0, 12, Align::Center, 1.0),
            None => rounded(cx - 5.0, y + 35.0, 10.0, 2.0, 1.0, SLATE),
        }
    }
}

/// The last few flights as bars, newest right: silver, this run berry, a
/// gold dashed line at the previous best, a berry cap on every splat.
fn trend_strip(app: &App, x: f32, y: f32, w: f32) {
    let g = &app.game;
    let font = &app.font;
    if app.recent.is_empty() {
        return;
    }
    let (bw, pitch, max_h) = (22.0, 30.0, 70.0);
    let n = crate::RECENT_RUNS;
    let strip_w = pitch * n as f32 - (pitch - bw);
    let x0 = x + (w - strip_w) / 2.0;
    let base = y + max_h;
    let scale_to = app.recent.iter().map(|s| s.distance).fold(app.best_before, f32::max).max(1.0);
    for (i, sc) in app.recent.iter().enumerate() {
        let slot = n - app.recent.len() + i;
        let bx = x0 + slot as f32 * pitch;
        let bh = (max_h * sc.distance / scale_to).max(4.0);
        let mine = i + 1 == app.recent.len() && (sc.distance - g.distance()).abs() < 0.01;
        rounded(bx - 2.0, base - bh - 2.0, bw + 4.0, bh + 2.0, 4.0, INK);
        rounded(bx, base - bh, bw, bh, 3.0, if mine { BERRY } else { SILVER });
        if sc.crashed {
            rounded(bx, base - bh, bw, 3.0, 1.5, BERRY);
        }
    }
    draw_line(x0 - 4.0, base + 1.0, x0 + strip_w + 4.0, base + 1.0, 2.0, INK);
    if app.best_before > 0.0 {
        let by = base - max_h * (app.best_before / scale_to).min(1.0);
        let mut dx = x0 - 4.0;
        while dx < x0 + strip_w + 4.0 {
            draw_line(dx, by, (dx + 6.0).min(x0 + strip_w + 4.0), by, 2.0, GOLD);
            dx += 10.0;
        }
    }
    if let Some(avg) = app.recent_avg() {
        text(font, &format!("last {} · avg {:.0} m", app.recent.len(), avg), x + w / 2.0, base + 16.0, 12, SLATE, Align::Center);
    }
}

/// Grade colour: gold for a great value, ink for fine, berry for poor.
fn grade(k: f32) -> Color {
    if k >= 0.82 {
        GOLD
    } else if k >= 0.4 {
        INK
    } else {
        BERRY
    }
}

/// The report card: the three skills of a run, each graded, and one coach
/// line for the weakest of them.
fn report_card(app: &App, x: f32, y: f32, w: f32, with_tip: bool) {
    let g = &app.game;
    let (font, big) = (&app.font, &app.display);
    let l = &app.pack.labels;
    let slide = hamsters_core::SLIDE_TIME * (1.25 - 0.25 * g.push);
    let jump_text = if g.jump_q >= hamsters_core::JUMP_PERFECT_FROM {
        "perfect".to_owned()
    } else if g.jump_q <= 0.0 {
        "missed".to_owned()
    } else if g.jump_t > 1.0 {
        format!("late by {:.2} s", (g.jump_t - 1.0) * slide)
    } else {
        format!("early by {:.2} s", (1.0 - g.jump_t) * slide)
    };
    // Air grade: presents and flaps both count; forty flaps or ten presents is a full mark.
    let air_k = ((g.flaps as f32 / 40.0) + (g.presents as f32 / 10.0)).clamp(0.0, 1.0);
    let rows = [
        ("PUSH", format!("{:.0}%", g.push * 100.0), g.push),
        ("JUMP", jump_text, g.jump_q),
        ("AIR", format!("{} flaps · {} presents", g.flaps, g.presents), air_k),
        ("HEIGHT", format!("{:.0} m up", g.max_height), (g.max_height / 160.0).clamp(0.0, 1.0)),
    ];
    let mut ry = y;
    for (label, value, k) in &rows {
        text(font, label, x, ry, 13, SLATE, Align::Left);
        let col = grade(*k);
        if col.r == GOLD.r && col.g == GOLD.g {
            gold_pill(big, value, x + w, ry + 1.0, 13, Align::Right, 1.0);
        } else {
            text(font, value, x + w, ry, 13, col, Align::Right);
        }
        draw_line(x, ry + 6.0, x + w, ry + 6.0, 1.0, fade(INK, 0.15));
        ry += 20.0;
    }
    if !with_tip {
        return;
    }
    // The tip names the weakest of the three skills; height is not a skill.
    let weakest = rows.iter().take(3).enumerate().min_by(|a, b| a.1 .2.partial_cmp(&b.1 .2).unwrap()).map(|(i, _)| i).unwrap_or(0);
    let tip = match weakest {
        0 => &l.tip_push,
        1 => &l.tip_jump,
        _ => &l.tip_air,
    };
    text(font, tip, x + w / 2.0, ry + 4.0, 13, BERRY, Align::Center);
}
