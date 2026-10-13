//! The painted UI kit: textures drawn by `tools/gen_ui.py` (logo, paper
//! panels, ribbons, wax seals, medals, stamps, buttons, the power dial and
//! the HUD icons) plus the slicing, text and animation helpers the HUD is
//! built from. The art carries its own painted depth; the code only lays it
//! out and animates it.

use macroquad::prelude::*;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Polls a batch of futures side by side until all are ready, keeping order.
struct JoinAll<'a, T> {
    futs: Vec<Pin<Box<dyn Future<Output = T> + 'a>>>,
    outs: Vec<Option<T>>,
}

impl<T: Unpin> Future for JoinAll<'_, T> {
    type Output = Vec<T>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Vec<T>> {
        let this = self.get_mut();
        if this.outs.is_empty() {
            this.outs = (0..this.futs.len()).map(|_| None).collect();
        }
        let mut done = true;
        for (f, o) in this.futs.iter_mut().zip(this.outs.iter_mut()) {
            if o.is_none() {
                match f.as_mut().poll(cx) {
                    Poll::Ready(v) => *o = Some(v),
                    Poll::Pending => done = false,
                }
            }
        }
        if done {
            Poll::Ready(this.outs.iter_mut().map(|o| o.take().unwrap()).collect())
        } else {
            Poll::Pending
        }
    }
}

async fn load_ui(base: &str, name: &str) -> Texture2D {
    let path = format!("{base}/{name}.png");
    let t = load_texture(&path).await.unwrap_or_else(|e| panic!("missing sprite {path}: {e:?}"));
    t.set_filter(FilterMode::Linear);
    t
}

/// Ribbon colours, in the order of [`UiKit::ribbons`].
#[derive(Clone, Copy, PartialEq)]
pub enum Rib {
    Red,
    Green,
    Gold,
    Ice,
}

pub struct UiKit {
    pub logo: Texture2D,
    pub panel: Texture2D,
    pub panel_gold: Texture2D,
    pub chip: Texture2D,
    pub chip_dark: Texture2D,
    pub ribbons: [Texture2D; 4],
    pub seal_red: Texture2D,
    pub seal_green: Texture2D,
    pub seal_gold: Texture2D,
    /// Gold, silver, bronze.
    pub medals: [Texture2D; 3],
    pub rosette: Texture2D,
    pub stamp_red: Texture2D,
    pub stamp_gold: Texture2D,
    pub stamp_green: Texture2D,
    pub button: Texture2D,
    pub button_off: Texture2D,
    pub dial: Texture2D,
    pub needle: Texture2D,
    pub holly: Texture2D,
    pub snow_wide: Texture2D,
    pub snow_small: Texture2D,
    pub burst: Texture2D,
    pub ic_flag: Texture2D,
    pub ic_ear: Texture2D,
    pub ic_alt: Texture2D,
    pub ic_star: Texture2D,
    pub ic_restart: Texture2D,
    pub ic_trophy: Texture2D,
    pub ic_speed: Texture2D,
    pub ic_bounce: Texture2D,
    pub ic_jump: Texture2D,
    pub ic_push: Texture2D,
    pub ic_gift: Texture2D,
}

impl UiKit {
    /// Loads every UI texture at once (the requests run in parallel, so the
    /// kit adds almost nothing to the start-up wait).
    pub async fn load(base: &str) -> Self {
        const NAMES: [&str; 38] = ["ui_logo", "ui_panel", "ui_panel_gold", "ui_chip", "ui_chip_dark", "ui_ribbon_red", "ui_ribbon_green", "ui_ribbon_gold", "ui_ribbon_ice", "ui_seal_red", "ui_seal_green", "ui_seal_gold", "ui_medal_gold", "ui_medal_silver", "ui_medal_bronze", "ui_rosette", "ui_stamp_red", "ui_stamp_gold", "ui_stamp_green", "ui_button", "ui_button_off", "ui_dial", "ui_needle", "ui_holly", "ui_snow_wide", "ui_snow_small", "ui_burst", "ui_ic_flag", "ui_ic_ear", "ui_ic_alt", "ui_ic_star", "ui_ic_restart", "ui_ic_trophy", "ui_ic_speed", "ui_ic_bounce", "ui_ic_jump", "ui_ic_push", "ui_ic_gift"];
        let futs = NAMES.iter().map(|n| Box::pin(load_ui(base, n)) as Pin<Box<dyn Future<Output = Texture2D> + '_>>).collect();
        let mut t = JoinAll { futs, outs: Vec::new() }.await.into_iter();
        let mut n = || t.next().expect("ui texture");
        UiKit {
            logo: n(),
            panel: n(),
            panel_gold: n(),
            chip: n(),
            chip_dark: n(),
            ribbons: [n(), n(), n(), n()],
            seal_red: n(),
            seal_green: n(),
            seal_gold: n(),
            medals: [n(), n(), n()],
            rosette: n(),
            stamp_red: n(),
            stamp_gold: n(),
            stamp_green: n(),
            button: n(),
            button_off: n(),
            dial: n(),
            needle: n(),
            holly: n(),
            snow_wide: n(),
            snow_small: n(),
            burst: n(),
            ic_flag: n(),
            ic_ear: n(),
            ic_alt: n(),
            ic_star: n(),
            ic_restart: n(),
            ic_trophy: n(),
            ic_speed: n(),
            ic_bounce: n(),
            ic_jump: n(),
            ic_push: n(),
            ic_gift: n(),
        }
    }

    pub fn ribbon(&self, r: Rib) -> &Texture2D {
        &self.ribbons[r as usize]
    }
}

// ---------------------------------------------------------------------------
// Animation state the HUD keeps for itself (wasm is single-threaded).
// ---------------------------------------------------------------------------

pub struct UiState {
    /// Phase tag (0 aim, 1 slide, 2 fly, 3 landed) and when it began.
    pub phase: u8,
    pub phase_at: f64,
    pub pilot: usize,
    pub pilot_at: f64,
    /// Displayed stamina, eased towards the real value.
    pub stamina: f32,
    /// Combo badge: the count shown, when it last changed, when it ended.
    pub combo: u32,
    pub combo_at: f64,
    pub combo_gone_at: f64,
    /// Banner slot memo: (text, life bits) -> screen y.
    pub slots: Vec<(String, u32, f32)>,
    pub presents: u32,
    pub presents_at: f64,
}

thread_local! {
    pub static STATE: RefCell<UiState> = RefCell::new(UiState {
        phase: 255,
        phase_at: 0.0,
        pilot: 0,
        pilot_at: -10.0,
        stamina: 1.0,
        combo: 0,
        combo_at: -10.0,
        combo_gone_at: -10.0,
        slots: Vec::new(),
        presents: 0,
        presents_at: -10.0,
    });
}

pub fn with_state<R>(f: impl FnOnce(&mut UiState) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// ---------------------------------------------------------------------------
// Easing
// ---------------------------------------------------------------------------

pub fn ease_out_cubic(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    1.0 - (1.0 - k).powi(3)
}

pub fn ease_in_cubic(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    k * k * k
}

pub fn ease_out_back(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.70158 * k * k * k + 1.70158 * k * k
}

/// Ease-out-elastic for stamps that slam and wobble into place.
pub fn ease_out_elastic(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    if k <= 0.0 || k >= 1.0 {
        return k;
    }
    2f32.powf(-10.0 * k) * ((k * 10.0 - 0.75) * std::f32::consts::TAU / 3.0).sin() + 1.0
}

// ---------------------------------------------------------------------------
// Sprite placement
// ---------------------------------------------------------------------------

/// Draw centred on `c`, `w` wide (aspect kept), rotated clockwise by `rot`.
pub fn spr(tex: &Texture2D, c: Vec2, w: f32, rot: f32, tint: Color) {
    let h = w * tex.height() / tex.width();
    draw_texture_ex(tex, c.x - w / 2.0, c.y - h / 2.0, tint, DrawTextureParams { dest_size: Some(vec2(w, h)), rotation: rot, ..Default::default() });
}

/// Draw the texture region `src` (in texture pixels) into `dst`.
fn part(tex: &Texture2D, src: Rect, dst: Rect, tint: Color) {
    if dst.w <= 0.0 || dst.h <= 0.0 {
        return;
    }
    draw_texture_ex(tex, dst.x, dst.y, tint, DrawTextureParams { dest_size: Some(vec2(dst.w, dst.h)), source: Some(src), ..Default::default() });
}

/// Nine-slice: `b` is the border in texture pixels, `d` on screen.
pub fn nine(tex: &Texture2D, r: Rect, b: f32, d: f32, tint: Color) {
    let (tw, th) = (tex.width(), tex.height());
    let xs = [0.0, b, tw - b, tw];
    let ys = [0.0, b, th - b, th];
    let d = d.min(r.w / 2.0).min(r.h / 2.0);
    let dx = [r.x, r.x + d, r.x + r.w - d, r.x + r.w];
    let dy = [r.y, r.y + d, r.y + r.h - d, r.y + r.h];
    for j in 0..3 {
        for i in 0..3 {
            part(tex, Rect::new(xs[i], ys[j], xs[i + 1] - xs[i], ys[j + 1] - ys[j]), Rect::new(dx[i], dy[j], dx[i + 1] - dx[i], dy[j + 1] - dy[j]), tint);
        }
    }
}

/// A paper panel whose visible body fills `r`. `k` scales the painted
/// border (1 = the art's own 40 px corner pieces).
pub fn panel(tex: &Texture2D, r: Rect, k: f32, tint: Color) {
    // ui_panel*: 120x120 svg, body inset 6 left/right, 4 top, 10 bottom.
    let s = k;
    nine(tex, Rect::new(r.x - 6.0 * s, r.y - 4.0 * s, r.w + 12.0 * s, r.h + 14.0 * s), 80.0, 40.0 * s, tint);
}

/// A small paper chip whose body fills `r` (border scaled by `k`).
pub fn chip(tex: &Texture2D, r: Rect, k: f32, tint: Color) {
    // ui_chip: 72x72 svg, body inset 4 left/right, 3 top, 8 bottom.
    let s = k;
    nine(tex, Rect::new(r.x - 4.0 * s, r.y - 3.0 * s, r.w + 8.0 * s, r.h + 13.0 * s), 48.0, 24.0 * s, tint);
}

/// A ribbon with folded swallow tails whose flat body is `bw` x `bh`,
/// centred on `c` and tilted clockwise by `rot`.
pub fn ribbon(tex: &Texture2D, c: Vec2, bw: f32, bh: f32, rot: f32, tint: Color) {
    // ui_ribbon_*: 360x100 svg, caps 96, body x 48..312, y 20..76.
    let s = bh / 56.0;
    let total_w = bw + 96.0 * s;
    let h = 100.0 * s;
    let cap = 96.0 * s;
    let x0 = c.x - total_w / 2.0;
    let y0 = c.y - 48.0 * s;
    let (tw, th) = (tex.width(), tex.height());
    let srcs = [Rect::new(0.0, 0.0, 192.0, th), Rect::new(192.0, 0.0, tw - 384.0, th), Rect::new(tw - 192.0, 0.0, 192.0, th)];
    let mid = (total_w - 2.0 * cap).max(0.0);
    let dsts = [Rect::new(x0, y0, cap, h), Rect::new(x0 + cap, y0, mid, h), Rect::new(x0 + cap + mid, y0, cap, h)];
    for (src, dst) in srcs.iter().zip(dsts.iter()) {
        if dst.w <= 0.0 {
            continue;
        }
        draw_texture_ex(tex, dst.x, dst.y, tint, DrawTextureParams { dest_size: Some(vec2(dst.w, dst.h)), source: Some(*src), rotation: rot, pivot: Some(c), ..Default::default() });
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

pub fn text_w(font: &Font, s: &str, size: u16, scale: f32) -> f32 {
    measure_text(s, Some(font), size, scale).width
}

/// Text with a round outline of `edge` (`t` px) and a soft drop below,
/// rotated clockwise by `rot` about its own centre. `x` is the centre.
#[allow(clippy::too_many_arguments)]
pub fn text_painted(font: &Font, s: &str, x: f32, y: f32, size: u16, scale: f32, fill: Color, edge: Color, t: f32, rot: f32) {
    let d = measure_text(s, Some(font), size, scale);
    let (sn, cs) = (rot.sin(), rot.cos());
    // `y` is the baseline; the text turns about its baseline centre.
    let o = vec2(x - d.width / 2.0 * cs, y - d.width / 2.0 * sn);
    let p = |dx: f32, dy: f32, col: Color| {
        draw_text_ex(s, o.x + dx, o.y + dy, TextParams { font: Some(font), font_size: size, font_scale: scale, rotation: rot, color: col, ..Default::default() });
    };
    let shadow = Color::new(0.07, 0.09, 0.2, 0.35 * fill.a);
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        p(a.cos() * t, a.sin() * t + t * 0.9, shadow);
    }
    for i in 0..16 {
        let a = i as f32 * std::f32::consts::TAU / 16.0;
        p(a.cos() * t, a.sin() * t, edge);
    }
    p(0.0, 0.0, fill);
}

// ---------------------------------------------------------------------------
// Bars
// ---------------------------------------------------------------------------

/// A rounded rectangle (triangle fan).
pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, r, h - 2.0 * r, color);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, color);
    for (cx, cy) in [(x + r, y + r), (x + w - r, y + r), (x + r, y + h - r), (x + w - r, y + h - r)] {
        draw_circle(cx, cy, r, color);
    }
}

/// A painted bar: dark recessed track, a top-lit fill and a gloss line.
pub fn bar(x: f32, y: f32, w: f32, h: f32, frac: f32, fill: Color, track: Color, edge: Color) {
    rrect(x - 2.0, y - 2.0, w + 4.0, h + 4.0, h / 2.0 + 2.0, edge);
    rrect(x, y, w, h, h / 2.0, track);
    rrect(x, y, w, h * 0.35, h * 0.3, Color::new(0.0, 0.0, 0.0, 0.12));
    let fw = (w * frac.clamp(0.0, 1.0)).max(if frac > 0.0 { h } else { 0.0 });
    if fw > 0.0 {
        rrect(x, y, fw, h, h / 2.0, shade(fill, 0.78));
        rrect(x, y, fw, h * 0.72, h * 0.36, fill);
        rrect(x + h * 0.35, y + h * 0.16, (fw - h * 0.7).max(0.0), h * 0.22, h * 0.11, Color::new(1.0, 1.0, 1.0, 0.55 * fill.a));
    }
}

pub fn shade(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

pub fn tint(a: f32) -> Color {
    Color::new(1.0, 1.0, 1.0, a)
}
