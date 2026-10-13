//! Headless play-analysis: flies many seeded runs with scripted players and
//! prints distances, run lengths, pickup rates and "dead air" so course and
//! pacing changes are measured instead of guessed (numbers in REPORT.md).
//!
//!     cargo run --release -p hamsters-core --example playtest            per-pilot table
//!     cargo run --release -p hamsters-core --example playtest -- pilots  pilot balance
//!     cargo run --release -p hamsters-core --example playtest -- ab      launch quality A/B
//!     cargo run --release -p hamsters-core --example playtest -- launch  push x jump grid
//!     cargo run --release -p hamsters-core --example playtest -- campaign  orders and ranks
//!     cargo run --release -p hamsters-core --example playtest -- trace <pilot> <seed> [mash|chain]
//!
//! Players:
//!   none    never flaps (launch only)
//!   mash    presses every frame
//!   rhythm  flaps whenever falling with stamina to spare, never steers
//!   skilled steers like a person: wiggles only when its line would miss the
//!           next pickup within a second, hops over storms on its line, never
//!           climbs into one, brakes and stomps for the landing
//!
//! A "peak" is any pickup, hazard, close shave, hard bounce, milestone or
//! order; "dead air" is flight time spent in gaps of more than 3 s between
//! peaks.

use hamsters_core::*;

const DT: f32 = 1.0 / 120.0;

#[derive(Clone, Copy, PartialEq)]
enum Player {
    None,
    Mash,
    Rhythm,
    Skilled,
}

#[derive(Default, Clone)]
struct Run {
    dist: f32,
    time: f32,
    presents: u32,
    boosts: u32,
    storms: u32,
    crashed: bool,
    max_h: f32,
    /// Longest stretch of flight with no highlight event, seconds.
    longest_gap: f32,
    /// Fraction of flight time spent in gaps longer than 3 s.
    dead_frac: f32,
    /// Highlights per 10 s of flight.
    peaks_per_10s: f32,
    missions: u32,
    /// Boosters caught before the course's own boosters start (the relay).
    relay: u32,
    /// Speed when the dog reached the end of the relay stretch.
    entry_speed: f32,
}

fn proj(g: &Game, x: f32) -> f32 {
    let t = ((x - g.pos.x) / g.vel.x.max(4.0)).max(0.0);
    let grav = if g.glide > 0.0 { GRAVITY * GLIDE_GRAVITY_SCALE } else { GRAVITY };
    g.pos.y + g.vel.y * t - 0.5 * grav * t * t
}

/// Steer like a person: pick the next pickup ahead that is reachable and
/// flap while the line runs under it; never climb into a storm on the way.
fn skilled_wants_flap(g: &Game) -> bool {
    let vx = g.vel.x.max(6.0);
    for e in g.entities.iter().filter(|e| e.alive && matches!(e.kind, EntityKind::Stormcloud | EntityKind::ColdWind)) {
        let dx = e.pos.x - g.pos.x;
        if dx > 0.0 && dx < vx * 0.9 {
            let dy = e.pos.y - proj(g, e.pos.x);
            if dy.abs() < 5.0 {
                return dy < 0.0; // over it if it sits on or under the line
            }
        }
    }
    let target = g
        .entities
        .iter()
        .filter(|e| e.alive && !matches!(e.kind, EntityKind::Stormcloud | EntityKind::ColdWind | EntityKind::Trampoline))
        .filter(|e| {
            let dx = e.pos.x - g.pos.x;
            let dy = e.pos.y - proj(g, e.pos.x);
            dx > 1.0 && dx < vx * 1.0 && dy > -4.0 && dy < 16.0
        })
        .min_by(|a, b| a.pos.x.partial_cmp(&b.pos.x).unwrap());
    if let Some(t) = target {
        // Only correct when the line would miss it: a wiggle is a big kick.
        let reach = t.kind.radius() * if t.kind == EntityKind::Present { PRESENT_REACH } else { PICKUP_REACH } + PILOT_RADIUS;
        return proj(g, t.pos.x) < t.pos.y - reach * 0.7 && g.vel.y < 14.0;
    }
    g.vel.y < -6.0 && g.pos.y < 30.0 && g.stamina > 0.3
}

fn wants(g: &Game, player: Player) -> bool {
    match player {
                Player::None => false,
                Player::Mash => true,
                Player::Rhythm => g.vel.y < 0.0 && g.stamina > 0.25,
                Player::Skilled => {
                    let near = g.vel.y < 0.0 && g.pos.y < PILOT_RADIUS + STOMP_REACH && g.pos.y > PILOT_RADIUS + 0.5;
                    let brake = g.vel.y < -18.0 && g.pos.y < PILOT_RADIUS + STOMP_REACH + 9.0 && !near;
                    if brake {
                        true
                    } else if near {
                        g.pos.y < PILOT_RADIUS + 2.5
                    } else {
                        skilled_wants_flap(g)
                    }
                }
            }
}

fn fly(pilot: usize, seed: u64, power: f32, jump_t: f32, player: Player) -> Run {
    fly_orders(pilot, seed, power, jump_t, player, [0, 1, 2]).0
}

fn fly_orders(pilot: usize, seed: u64, power: f32, jump_t: f32, player: Player, orders: [u32; 3]) -> (Run, [bool; 3]) {
    let mut g = Game::new(PILOTS[pilot], seed);
    g.set_orders(orders.map(|i| Some(missions::goal(i))));
    g.phase = Phase::AimPower { meter: power };
    g.action();
    let mut jumped = false;
    let mut run = Run::default();
    let mut gap = 0.0f32;
    let mut dead = 0.0f32;
    let mut peaks = 0u32;
    let mut steps = 0;
    while g.phase != Phase::Landed && steps < 120 * 240 {
        steps += 1;
        if let Phase::Sliding { t, .. } = g.phase {
            if !jumped && t >= jump_t {
                g.action();
                jumped = true;
            }
        } else if g.phase == Phase::Flying {
            let press = wants(&g, player);
            if press {
                g.action();
            }
        }
        let before = g.pos.x < g.course.boost_from;
        g.step(DT);
        if before && g.pos.x >= g.course.boost_from {
            run.relay = g.stats.rockets + g.stats.reindeer;
            run.entry_speed = g.vel.length();
        }
        if g.phase == Phase::Flying || g.phase == Phase::Landed {
            gap += DT;
            for e in g.events.drain(..) {
                let peak = match e {
                    Event::Hit(EntityKind::Stormcloud) => {
                        run.storms += 1;
                        true
                    }
                    Event::Hit(EntityKind::Present) => {
                        run.presents += 1;
                        true
                    }
                    Event::Hit(EntityKind::ColdWind) => true,
                    Event::Hit(_) => {
                        run.boosts += 1;
                        true
                    }
                    Event::GroundBounce { speed, .. } => speed > 8.0,
                    Event::Milestone(_) | Event::WigglePower | Event::Meteor => true,
                    Event::CloseShave => true,
                    Event::MissionComplete(_) => {
                        run.missions += 1;
                        true
                    }
                    _ => false,
                };
                if peak {
                    if gap > 3.0 {
                        dead += gap;
                    }
                    run.longest_gap = run.longest_gap.max(gap);
                    gap = 0.0;
                    peaks += 1;
                }
            }
        } else {
            g.events.clear();
        }
    }
    if gap > 3.0 {
        dead += gap;
    }
    run.longest_gap = run.longest_gap.max(gap);
    run.dist = g.distance();
    run.time = g.flight_time;
    run.crashed = g.crashed;
    run.max_h = g.max_height;
    run.dead_frac = if g.flight_time > 0.0 { dead / g.flight_time } else { 0.0 };
    run.peaks_per_10s = if g.flight_time > 0.0 { peaks as f32 * 10.0 / g.flight_time } else { 0.0 };
    (run, g.orders_done)
}

/// Play the order book from scratch: how many flights does it take to
/// finish the first N orders (and reach each rank)?
fn campaign(pilot: usize, player: Player, flights: usize, seed0: u64) -> (u32, Vec<usize>) {
    let mut active = [0u32, 1, 2];
    let mut next = 3u32;
    let mut done = 0u32;
    let mut rank_at = vec![0usize; missions::RANKS.len()];
    for f in 0..flights {
        let (_, ok) = fly_orders(pilot, seed0 + f as u64 * 31, 0.9, 0.985, player, active);
        for i in 0..3 {
            if ok[i] {
                done += 1;
                active[i] = next;
                next += 1;
            }
        }
        let r = missions::rank_for(done);
        for k in 1..=r {
            if rank_at[k] == 0 {
                rank_at[k] = f + 1;
            }
        }
    }
    (done, rank_at)
}

fn pct(v: &mut Vec<f32>, p: f32) -> f32 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f32 * p).round() as usize]
}

fn report(label: &str, runs: &[Run]) {
    let n = runs.len() as f32;
    let mut d: Vec<f32> = runs.iter().map(|r| r.dist).collect();
    let mean = |f: &dyn Fn(&Run) -> f32| runs.iter().map(f).sum::<f32>() / n;
    println!(
        "{:<28} dist p10 {:>6.0} p50 {:>6.0} p90 {:>6.0} | time {:>5.1}s | pres {:>5.1} boost {:>4.1} storm {:>4.1} | crash {:>3.0}% | gap max {:>4.1}s dead {:>3.0}% peaks/10s {:>4.1} | missions {:>3.1}",
        label,
        pct(&mut d.clone(), 0.1),
        pct(&mut d.clone(), 0.5),
        pct(&mut d, 0.9),
        mean(&|r| r.time),
        mean(&|r| r.presents as f32),
        mean(&|r| r.boosts as f32),
        mean(&|r| r.storms as f32),
        100.0 * mean(&|r| r.crashed as u8 as f32),
        mean(&|r| r.longest_gap),
        100.0 * mean(&|r| r.dead_frac),
        mean(&|r| r.peaks_per_10s),
        mean(&|r| r.missions as f32),
    );
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("campaign") {
        for (pn, p) in [("rhythm", Player::Rhythm), ("skilled", Player::Skilled)] {
            for pilot in 0..3 {
                let mut tot = 0;
                let mut ranks = vec![0usize; missions::RANKS.len()];
                let n = 20;
                for s in 0..n {
                    let (d, r) = campaign(pilot, p, 40, 1000 + s * 977);
                    tot += d;
                    for k in 0..r.len() {
                        ranks[k] += if r[k] == 0 { 99 } else { r[k] };
                    }
                }
                println!("{pn:<8} {:<8} orders done in 40 flights: {:>5.1}  flights to rank: {:?}", PILOTS[pilot].name, tot as f32 / n as f32, ranks.iter().skip(1).map(|v| *v as f32 / n as f32).collect::<Vec<_>>());
            }
        }
        return;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("trace") {
        let pilot = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let seed = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(7932);
        if args.get(4).map(|s| s.as_str()) == Some("chain") {
            let g = Game::new(PILOTS[pilot], seed);
            for e in &g.entities {
                println!("{:?} {:.1} {:.1}", e.kind, e.pos.x, e.pos.y);
            }
            println!("landing {:.0} boost_from {:.0}", g.landing_x, g.course.boost_from);
            return;
        }
        trace(pilot, seed, args.get(4).map(|s| s.as_str()) == Some("mash"));
        return;
    }
    if args.get(1).map(|s| s.as_str()) == Some("arcs") {
        for pi in 0..3 {
            for (pw, q) in [(0.5f32, 1.0f32), (0.7, 1.0), (0.85, 1.0), (1.0, 1.0), (0.85, 0.3), (1.0, 0.5)] {
                let a = predict_arc(PILOTS[pi], pw, q);
                let apex = a.iter().fold(0.0f32, |m, p| m.max(p.y));
                print!(" {}@{pw:.2}/q{q:.1}: land {:.0} apex {:.0} |", PILOTS[pi].name, a.last().unwrap().x, apex);
            }
            println!();
        }
        return;
    }
    if args.get(1).map(|s| s.as_str()) == Some("pilots") {
        for pi in 0..3 {
            let mut line = format!("{:<8}", PILOTS[pi].name);
            for (pn, p) in [("none", Player::None), ("mash", Player::Mash), ("rhythm", Player::Rhythm), ("skilled", Player::Skilled)] {
                let rs: Vec<Run> = (1..=150u64).map(|s| fly(pi, s * 7919 + 13, 0.9, 0.985, p)).collect();
                let mut d: Vec<f32> = rs.iter().map(|r| r.dist).collect();
                let t = rs.iter().map(|r| r.time).sum::<f32>() / rs.len() as f32;
                line += &format!(" | {pn} p50 {:>5.0} p90 {:>5.0} {t:>3.0}s", pct(&mut d.clone(), 0.5), pct(&mut d, 0.9));
            }
            println!("{line}");
        }
        return;
    }
    if args.get(1).map(|s| s.as_str()) == Some("ab") {
        for (label, power, jt) in [("weak push, early jump", 0.5f32, 0.6f32), ("weak push, lip jump", 0.5, 0.985), ("sweet push, early jump", 0.85, 0.7), ("sweet push, lip jump", 0.85, 0.985), ("full push, late jump", 1.0, 1.05)] {
            for (pn, p) in [("none", Player::None), ("mash", Player::Mash), ("rhythm", Player::Rhythm), ("skilled", Player::Skilled)] {
                let rs: Vec<Run> = (0..3).flat_map(|pi| (1..=200u64).map(move |s| (pi, s * 7919 + 13))).map(|(pi, s)| fly(pi, s, power, jt, p)).collect();
                let n = rs.len() as f32;
                let mut d: Vec<f32> = rs.iter().map(|r| r.dist).collect();
                println!("{label:<24} {pn:<8} mean {:>5.0}  p50 {:>5.0}  p90 {:>5.0}  relay {:.1}  time {:.0}s", d.iter().sum::<f32>() / n, pct(&mut d.clone(), 0.5), pct(&mut d, 0.9), rs.iter().map(|r| r.relay as f32).sum::<f32>() / n, rs.iter().map(|r| r.time).sum::<f32>() / n);
            }
        }
        return;
    }
    if args.get(1).map(|s| s.as_str()) == Some("launch") {
        for (pn, p) in [("none", Player::None), ("skilled", Player::Skilled)] {
            println!("{pn}: p50 distance by push (rows) and jump press (cols 0.6 0.8 0.95 0.985 1.03 1.08)");
            for power in [0.5f32, 0.7, 0.85, 0.95, 1.0] {
                let mut row = format!("  push {power:.2}:");
                for jt in [0.6f32, 0.8, 0.95, 0.985, 1.03, 1.08] {
                    let mut d: Vec<f32> = (0..3).flat_map(|pi| (1..=60u64).map(move |s| (pi, s * 7919 + 13))).map(|(pi, s)| fly(pi, s, power, jt, p).dist).collect();
                    let rs: Vec<Run> = (0..3).flat_map(|pi| (1..=40u64).map(move |s| (pi, s * 7919 + 13))).map(|(pi, s)| fly(pi, s, power, jt, p)).collect();
                    let relay = rs.iter().map(|r| r.relay as f32).sum::<f32>() / rs.len() as f32;
                    let entry = rs.iter().map(|r| r.entry_speed).sum::<f32>() / rs.len() as f32;
                    row += &format!(" {:>5.0}/{relay:.1}/{entry:.0}", pct(&mut d, 0.5));
                }
                println!("{row}");
            }
        }
        return;
    }
    if args.get(1).map(|s| s.as_str()) == Some("find") {
        let pilot: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
        for s in (1..=150u64).map(|s| s * 7919 + 13) {
            let r = fly(pilot, s, 0.9, 0.985, Player::Skilled);
            if r.time > 100.0 {
                println!("seed {s} dist {:.0} time {:.0}", r.dist, r.time);
            }
        }
        return;
    }
    let seeds: Vec<u64> = (1..=150).map(|s| s * 7919 + 13).collect();
    let launches = [("sweet push, lip jump", 0.9, 0.985), ("full push, early jump", 1.0, 0.75), ("weak push, lip jump", 0.55, 0.985), ("sweet push, late jump", 0.9, 1.04)];
    let players = [("none", Player::None), ("mash", Player::Mash), ("rhythm", Player::Rhythm), ("skilled", Player::Skilled)];
    for pilot in 0..3 {
        println!("== {} ==", PILOTS[pilot].name);
        for (ln, power, jt) in launches {
            for (pn, p) in players {
                if ln != launches[0].0 && p != Player::Skilled && p != Player::None {
                    continue;
                }
                let runs: Vec<Run> = seeds.iter().map(|&s| fly(pilot, s, power, jt, p)).collect();
                report(&format!("{ln} / {pn}"), &runs);
            }
        }
    }
}

#[allow(dead_code)]
pub fn trace(pilot: usize, seed: u64, mash: bool) {
    let mut g = Game::new(PILOTS[pilot], seed);
    g.phase = Phase::AimPower { meter: 0.85 };
    g.action();
    let mut jumped = false;
    let mut t = 0.0f32;
    let none = std::env::var("NONE").is_ok();
    let mut next = 0.0f32;
    let mut log = String::new();
    while g.phase != Phase::Landed && t < 200.0 {
        if let Phase::Sliding { t: rt, .. } = g.phase {
            if !jumped && rt >= 0.985 {
                g.action();
                jumped = true;
            }
        } else if !none && wants(&g, if mash { Player::Mash } else { Player::Skilled }) {
            g.action();
        }
        g.step(DT);
        for e in g.events.drain(..) {
            match e {
                Event::Hit(k) => log.push_str(&format!(" {:?}", k)),
                Event::WigglePower => log.push_str(" WP"),
                Event::CloseShave => log.push_str(" SHAVE"),
                Event::GroundBounce { speed, stomped } => log.push_str(&format!(" bounce{:.0}{}", speed, if stomped { "S" } else { "" })),
                _ => {}
            }
        }
        if g.phase == Phase::Flying {
            t += DT;
        }
        if t >= next {
            next += 0.5;
            let ahead = g.entities.iter().filter(|e| e.alive && e.pos.x > g.pos.x && e.pos.x < g.pos.x + 60.0).count();
            let near = g.entities.iter().filter(|e| e.alive && e.pos.x > g.pos.x && e.pos.x < g.pos.x + 60.0 && (e.pos.y - g.pos.y).abs() < 25.0).count();
            println!("t {:>5.1} x {:>6.0} y {:>5.0} vx {:>5.1} vy {:>6.1} st {:.2} ahead {} near {} |{}", t, g.pos.x, g.pos.y, g.vel.x, g.vel.y, g.stamina, ahead, near, log);
            log.clear();
        }
    }
}
