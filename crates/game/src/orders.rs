//! Santa's orders in the runtime: which three are active, how many have
//! been finished (the flight-school rank), which cape the pilot wears, and
//! the small paper card that lists them. The rules live in
//! `hamsters_core::missions`; this file only keeps score and draws.
//!
//! Drawing uses the existing `hud` helpers (plate, text) and sits in its own
//! corner, so the HUD can be restyled without touching the rules.

use std::sync::atomic::{AtomicUsize, Ordering};

use hamsters_core::missions::{self, Goal};
use hamsters_core::Phase;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::hud::{self, Align, BERRY, GOLD, INK, PAPER, PINE, SLATE};
use crate::App;

pub const ORDERS_KEY: &str = "santas_flight_school_orders";

/// Capes unlocked by rank: index 0 is the pilot's own scarf colour, then one
/// per rank after CADET.
pub const CAPES: [(&str, Color); 6] = [
    ("scarf", Color::new(0.0, 0.0, 0.0, 0.0)),
    ("elf green", Color::new(0.16, 0.62, 0.34, 1.0)),
    ("velvet red", Color::new(0.70, 0.08, 0.20, 1.0)),
    ("storm violet", Color::new(0.48, 0.34, 0.82, 1.0)),
    ("chinook blue", Color::new(0.22, 0.72, 0.93, 1.0)),
    ("santa's gold", Color::new(0.97, 0.76, 0.22, 1.0)),
];

/// The cape the player wears, read by the dog drawing. 0 is the scarf.
static CAPE: AtomicUsize = AtomicUsize::new(0);

/// The cape colour to draw instead of the pilot's scarf colour, if any.
pub fn cape_fill() -> Option<Color> {
    match CAPE.load(Ordering::Relaxed) {
        0 => None,
        i => CAPES.get(i).map(|c| c.1),
    }
}

/// Saved between sessions under [`ORDERS_KEY`].
#[derive(Clone, Serialize, Deserialize)]
pub struct Orders {
    /// Indices into the order book (`missions::goal`).
    pub active: [u32; 3],
    /// The next order to hand out.
    pub next: u32,
    /// Orders completed in total: the rank.
    pub done: u32,
    /// Chosen cape, index into [`CAPES`].
    #[serde(default)]
    pub cape: usize,
    /// When each slot was ticked off this flight (game clock), for the flash.
    #[serde(skip)]
    pub ticked_at: [Option<f64>; 3],
    /// Slots handed a new order after the last flight.
    #[serde(skip)]
    pub fresh: [bool; 3],
}

impl Default for Orders {
    fn default() -> Self {
        Orders { active: [0, 1, 2], next: 3, done: 0, cape: 0, ticked_at: [None; 3], fresh: [false; 3] }
    }
}

impl Orders {
    pub fn goals(&self) -> [Option<Goal>; 3] {
        self.active.map(|i| Some(missions::goal(i)))
    }

    pub fn rank(&self) -> usize {
        missions::rank_for(self.done)
    }

    /// Capes this rank may wear: the scarf plus one per rank.
    pub fn capes_unlocked(&self) -> usize {
        (self.rank() + 1).min(CAPES.len())
    }

    /// Wear a cape (clamped to what is unlocked) and tell the dog drawing.
    pub fn wear(&mut self, cape: usize) {
        self.cape = cape.min(self.capes_unlocked() - 1);
        CAPE.store(self.cape, Ordering::Relaxed);
    }

    pub fn next_cape(&mut self) {
        let n = self.capes_unlocked();
        self.wear((self.cape + 1) % n);
    }

    /// Bank the orders a flight finished and hand out new ones. Returns the
    /// new rank if it went up; a new rank's cape goes straight on.
    pub fn bank(&mut self, done: [bool; 3]) -> Option<usize> {
        let before = self.rank();
        self.fresh = [false; 3];
        self.ticked_at = [None; 3];
        for i in 0..3 {
            if done[i] {
                self.done += 1;
                self.active[i] = self.next;
                self.next += 1;
                self.fresh[i] = true;
            }
        }
        let after = self.rank();
        if after > before {
            self.wear(after);
            Some(after)
        } else {
            None
        }
    }
}

/// A check box, ticked in pine when done, crossed in berry when missed.
fn check(x: f32, y: f32, s: f32, done: bool, failed: bool, alpha: f32) {
    hud::rounded(x, y, s, s, 3.0, hud::fade(INK, alpha));
    hud::rounded(x + 2.0, y + 2.0, s - 4.0, s - 4.0, 2.0, hud::fade(if done { PINE } else { PAPER }, alpha));
    if done {
        let w = PAPER;
        draw_line(x + s * 0.24, y + s * 0.52, x + s * 0.44, y + s * 0.74, 2.5, hud::fade(w, alpha));
        draw_line(x + s * 0.44, y + s * 0.74, x + s * 0.80, y + s * 0.28, 2.5, hud::fade(w, alpha));
    } else if failed {
        let c = hud::fade(BERRY, alpha);
        draw_line(x + s * 0.28, y + s * 0.28, x + s * 0.72, y + s * 0.72, 2.0, c);
        draw_line(x + s * 0.72, y + s * 0.28, x + s * 0.28, y + s * 0.72, 2.0, c);
    }
}

/// The list itself: one row per order with its live progress.
fn rows(app: &App, x: f32, y: f32, w: f32, row_h: f32, size: u16, alpha: f32) {
    let g = &app.game;
    let o = &app.orders;
    let now = get_time();
    for i in 0..3 {
        let goal = missions::goal(o.active[i]);
        let p = missions::progress(goal, g);
        let done = g.orders_done[i] || p.done;
        let ry = y + i as f32 * row_h;
        // A fresh tick flashes gold for a second.
        if let Some(t) = o.ticked_at[i] {
            let k = ((now - t) as f32).clamp(0.0, 1.2);
            if k < 1.2 {
                hud::rounded(x - 4.0, ry - 2.0, w + 8.0, row_h, 6.0, hud::fade(GOLD, alpha * (1.0 - k / 1.2) * 0.8));
            }
        }
        let box_s = row_h - 8.0;
        check(x, ry + 2.0, box_s, done, p.failed && !done, alpha);
        let label = missions::describe(goal);
        let col = if done { SLATE } else { INK };
        hud::text(&app.font, &label, x + box_s + 7.0, ry + row_h * 0.62, size, hud::fade(col, alpha), Align::Left);
        let launched = matches!(g.phase, Phase::Flying | Phase::Landed);
        let tag = if o.fresh[i] && !launched {
            "new".to_owned()
        } else if launched {
            missions::readout(goal, &p)
        } else {
            String::new()
        };
        if !tag.is_empty() && !done {
            let c = if tag == "new" { BERRY } else if p.failed { BERRY } else { SLATE };
            hud::text(&app.font, &tag, x + w, ry + row_h * 0.62, size.saturating_sub(1), hud::fade(c, alpha), Align::Right);
        }
    }
}

/// Before launch: the order card under the pilot picker, with the rank and
/// the cape. During flight: a compact tracker in the top-right corner.
/// After landing: a narrow column beside the results postcard.
pub fn draw(app: &App) {
    let g = &app.game;
    let (sw, sh) = (screen_width(), screen_height());
    match g.phase {
        Phase::AimPower { .. } => {
            if app.preview > 0.0 {
                return;
            }
            // Under the third picker card (cards at sw - 274, y 74 + i * 124, 112 tall; see `hud`).
            let (w, x) = (256.0, sw - 274.0);
            let y = 74.0 + 3.0 * 124.0 - 2.0;
            let h = (sh - y - 12.0).min(120.0);
            crate::ui::panel(&app.sprites.ui.panel, Rect::new(x, y, w, h), 0.42, Color::new(1.0, 1.0, 1.0, 1.0));
            hud::text(&app.font, "SANTA'S ORDERS", x + 12.0, y + 20.0, 13, SLATE, Align::Left);
            let rank = missions::RANKS[app.orders.rank()].1;
            hud::text(&app.display, rank, x + w - 12.0, y + 21.0, 15, GOLD_INK, Align::Right);
            rows(app, x + 12.0, y + 28.0, w - 24.0, 24.0, 13, 1.0);
            let foot = match missions::to_next_rank(app.orders.done) {
                Some(n) => format!("{n} more for a new cape"),
                None => "every cape is yours".to_owned(),
            };
            let cy = y + h - 12.0;
            if app.orders.capes_unlocked() > 1 {
                let (name, col) = CAPES[app.orders.cape];
                let col = if app.orders.cape == 0 { crate::draw::scarf_color(app.pilot_idx) } else { col };
                draw_circle(x + 18.0, cy - 4.0, 7.0, INK);
                draw_circle(x + 18.0, cy - 4.0, 5.5, col);
                hud::text(&app.font, &format!("cape: {name}  (K)"), x + 30.0, cy, 12, SLATE, Align::Left);
            } else {
                hud::text(&app.font, &foot, x + 12.0, cy, 12, SLATE, Align::Left);
            }
        }
        Phase::Flying => {
            if g.flight_time < 0.5 {
                return;
            }
            let (w, h) = (236.0, 82.0);
            let (x, y) = (sw - w - 14.0, 14.0);
            crate::ui::panel(&app.sprites.ui.panel, Rect::new(x, y, w, h), 0.42, Color::new(1.0, 1.0, 1.0, 0.92));
            rows(app, x + 10.0, y + 7.0, w - 20.0, 23.0, 12, 0.92);
        }
        Phase::Landed => {
            let hold = hud::result_hold(g.crashed);
            if app.landed_for < hold {
                let (w, h) = (236.0, 82.0);
                let (x, y) = (sw - w - 14.0, 14.0);
                crate::ui::panel(&app.sprites.ui.panel, Rect::new(x, y, w, h), 0.42, Color::new(1.0, 1.0, 1.0, 0.92));
                rows(app, x + 10.0, y + 7.0, w - 20.0, 23.0, 12, 0.92);
                return;
            }
            // Beside the results card (784 wide, centred; see `hud`).
            let card_right = (sw + 784.0) / 2.0;
            let w = (sw - card_right - 14.0).min(200.0);
            if w < 150.0 {
                return;
            }
            let x = sw - w - 8.0;
            let y = (sh - 484.0) / 2.0 + 20.0;
            let h = 132.0;
            crate::ui::panel(&app.sprites.ui.panel, Rect::new(x, y, w, h), 0.42, Color::new(1.0, 1.0, 1.0, 1.0));
            hud::text(&app.font, "ORDERS", x + 10.0, y + 18.0, 12, SLATE, Align::Left);
            let rank = missions::RANKS[app.orders.rank()].1;
            hud::text(&app.display, rank, x + w - 10.0, y + 19.0, 13, GOLD_INK, Align::Right);
            rows(app, x + 10.0, y + 26.0, w - 20.0, 23.0, 11, 1.0);
            let gained = g.orders_done.iter().filter(|d| **d).count();
            let line = if gained > 0 { format!("+{gained} this flight, {} in all", app.orders.done + gained as u32) } else { format!("{} done in all", app.orders.done) };
            hud::text(&app.font, &line, x + 10.0, y + h - 12.0, 11, SLATE, Align::Left);
        }
        Phase::Sliding { .. } => {}
    }
}

/// Gold that reads on paper: the rank title.
const GOLD_INK: Color = Color::new(0.78, 0.52, 0.06, 1.0);
