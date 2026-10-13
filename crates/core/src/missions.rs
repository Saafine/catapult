//! Santa's flight orders: per-run challenges that give every launch a goal
//! beyond "farther than last time".
//!
//! Three orders are active at a time. An order is judged within one flight
//! ("collect 10 presents", "reach the Bow", "pass 3 storm clouds"); finishing
//! it fires [`crate::Event::MissionComplete`] the moment it happens, and the
//! runtime swaps in the next order from [`CAMPAIGN`] for the next run. After
//! the hand-made campaign, [`goal`] keeps generating harder ones forever.
//! Completed orders raise the pilot's flight-school rank ([`RANKS`]), which
//! is what unlocks capes in the runtime.

use crate::course::district_name_at;
use crate::{Game, Phase, JUMP_PERFECT_FROM};

/// What one order asks for. Distances and heights are meters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Goal {
    /// Catch this many presents in one flight.
    Presents(u32),
    /// Fly this far. Named after the Calgary district that starts there.
    Distance(f32),
    /// Climb this high above the snow.
    Height(f32),
    /// A present chain this long (each within `CHAIN_WINDOW` of the last).
    Chain(u32),
    /// Ride this many firework rockets.
    Rockets(u32),
    /// Get kicked by this many reindeer.
    Reindeer(u32),
    /// Ride this many Chinooks (updrafts).
    Chinooks(u32),
    /// Bounce off this many elf trampolines.
    Trampolines(u32),
    /// Fly past this many storm clouds without touching them.
    StormsPassed(u32),
    /// This many close shaves: storm clouds passed within a whisker.
    CloseShaves(u32),
    /// This many stomped landings that bounce back up.
    Stomps(u32),
    /// Reach this distance without touching a single storm cloud.
    Clean(f32),
    /// A perfect jump off the lip, then reach this distance.
    PerfectJump(f32),
    /// Reach this distance with at most this many wiggles.
    FewFlaps(u32, f32),
    /// End the flight past this distance without crashing into a hole.
    LandSafe(f32),
    /// Fall long enough to become a meteor.
    Meteor,
}

/// The hand-made order book, in the order a new pilot meets it. Early
/// orders teach the systems one at a time (presents, height, rockets,
/// storms, the stomp); later ones send the dog across town and ask for
/// clean or frugal flights. Three are active at once.
pub const CAMPAIGN: [Goal; 30] = [
    Goal::Presents(10),
    Goal::Distance(190.0),
    Goal::Height(60.0),
    Goal::Rockets(1),
    Goal::StormsPassed(3),
    Goal::Chain(5),
    Goal::Presents(20),
    Goal::Stomps(1),
    Goal::Reindeer(2),
    Goal::PerfectJump(200.0),
    Goal::Distance(400.0),
    Goal::CloseShaves(1),
    Goal::Chinooks(2),
    Goal::Clean(300.0),
    Goal::Height(160.0),
    Goal::Trampolines(2),
    Goal::Presents(35),
    Goal::Distance(480.0),
    Goal::LandSafe(200.0),
    Goal::FewFlaps(10, 300.0),
    Goal::Chain(10),
    Goal::StormsPassed(8),
    Goal::Distance(650.0),
    Goal::Meteor,
    Goal::CloseShaves(3),
    Goal::Height(400.0),
    Goal::Distance(850.0),
    Goal::Presents(60),
    Goal::Clean(700.0),
    Goal::Distance(1100.0),
];

/// Order number `i`: the campaign first, then an endless rotation that
/// keeps raising the bar.
pub fn goal(i: u32) -> Goal {
    if (i as usize) < CAMPAIGN.len() {
        return CAMPAIGN[i as usize];
    }
    let k = i - CAMPAIGN.len() as u32;
    let lap = (k / 8) as f32 + 1.0;
    match k % 8 {
        0 => Goal::Distance(1100.0 + 150.0 * lap),
        1 => Goal::Presents(60 + 10 * lap as u32),
        2 => Goal::Chain(10 + 2 * lap as u32),
        3 => Goal::StormsPassed(8 + 3 * lap as u32),
        4 => Goal::Clean(700.0 + 120.0 * lap),
        5 => Goal::CloseShaves(3 + lap as u32),
        6 => Goal::FewFlaps(12, 400.0 + 100.0 * lap),
        _ => Goal::Stomps(2 + lap as u32),
    }
}

/// Flight-school ranks by orders completed. The runtime unlocks one cape
/// per rank after the first.
pub const RANKS: [(u32, &str); 6] = [
    (0, "CADET"),
    (3, "ELF FLYER"),
    (7, "REINDEER WING"),
    (12, "STORM CHASER"),
    (18, "CHINOOK RIDER"),
    (26, "SANTA'S ACE"),
];

/// Rank index for a number of completed orders.
pub fn rank_for(done: u32) -> usize {
    RANKS.iter().rposition(|(n, _)| done >= *n).unwrap_or(0)
}

/// Orders still needed to reach the next rank, if there is one.
pub fn to_next_rank(done: u32) -> Option<u32> {
    RANKS.iter().find(|(n, _)| *n > done).map(|(n, _)| n - done)
}

/// What the order says, in the voice of the flight school.
pub fn describe(g: Goal) -> String {
    match g {
        Goal::Presents(n) => format!("Catch {n} presents"),
        Goal::Distance(d) => match district_name_at(d) {
            Some(name) => format!("Reach {name}"),
            None => format!("Fly {d:.0} m"),
        },
        Goal::Height(h) => format!("Climb {h:.0} m high"),
        Goal::Chain(n) => format!("Chain {n} presents"),
        Goal::Rockets(1) => "Ride a rocket".into(),
        Goal::Rockets(n) => format!("Ride {n} rockets"),
        Goal::Reindeer(n) => format!("Bump {n} reindeer"),
        Goal::Chinooks(n) => format!("Ride {n} Chinooks"),
        Goal::Trampolines(n) => format!("Bounce on {n} trampolines"),
        Goal::StormsPassed(n) => format!("Fly past {n} storm clouds"),
        Goal::CloseShaves(1) => "Skim a storm cloud".into(),
        Goal::CloseShaves(n) => format!("Skim {n} storm clouds"),
        Goal::Stomps(1) => "Stomp a landing".into(),
        Goal::Stomps(n) => format!("Stomp {n} landings"),
        Goal::Clean(d) => format!("{d:.0} m without a storm hit"),
        Goal::PerfectJump(d) => format!("Perfect jump, then {d:.0} m"),
        Goal::FewFlaps(n, d) => format!("{d:.0} m with {n} flaps or less"),
        Goal::LandSafe(d) => format!("Land unhurt past {d:.0} m"),
        Goal::Meteor => "Fall as a meteor".into(),
    }
}

/// Live state of an order during a flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    /// How far along, in the order's own unit (presents, meters...).
    pub now: f32,
    pub target: f32,
    pub done: bool,
    /// The order can no longer be finished this flight.
    pub failed: bool,
}

impl Progress {
    /// 0..1 for a progress bar.
    pub fn frac(&self) -> f32 {
        if self.done {
            1.0
        } else if self.target <= 0.0 {
            0.0
        } else {
            (self.now / self.target).clamp(0.0, 1.0)
        }
    }
}

/// Judge an order against the flight so far.
pub fn progress(goal: Goal, g: &Game) -> Progress {
    let s = &g.stats;
    let launched = matches!(g.phase, Phase::Flying | Phase::Landed);
    let d = if launched { g.distance() } else { 0.0 };
    let count = |n: u32, target: u32| Progress { now: n as f32, target: target as f32, done: n >= target, failed: false };
    let reach = |now: f32, target: f32, failed: bool| Progress { now, target, done: !failed && now >= target, failed };
    match goal {
        Goal::Presents(n) => count(s.presents, n),
        Goal::Distance(t) => reach(d, t, false),
        Goal::Height(h) => reach(g.max_height, h, false),
        Goal::Chain(n) => count(s.best_chain, n),
        Goal::Rockets(n) => count(s.rockets, n),
        Goal::Reindeer(n) => count(s.reindeer, n),
        Goal::Chinooks(n) => count(s.chinooks, n),
        Goal::Trampolines(n) => count(s.trampolines, n),
        Goal::StormsPassed(n) => count(s.storms_passed, n),
        Goal::CloseShaves(n) => count(s.close_shaves, n),
        Goal::Stomps(n) => count(s.stomps, n),
        Goal::Clean(t) => reach(d, t, s.storms_hit > 0),
        Goal::PerfectJump(t) => reach(d, t, launched && g.jump_q < JUMP_PERFECT_FROM),
        Goal::FewFlaps(n, t) => reach(d, t, g.flaps > n),
        Goal::LandSafe(t) => {
            let over = g.phase == Phase::Landed;
            Progress { now: d, target: t, done: over && !g.crashed && d >= t, failed: over && (g.crashed || d < t) }
        }
        Goal::Meteor => Progress { now: s.meteor as u8 as f32, target: 1.0, done: s.meteor, failed: false },
    }
}

/// Short live readout for an order, like "7/10" or "212/400 m".
pub fn readout(goal: Goal, p: &Progress) -> String {
    if p.done {
        return "done".into();
    }
    if p.failed {
        return "missed".into();
    }
    match goal {
        Goal::Distance(_) | Goal::Height(_) | Goal::Clean(_) | Goal::PerfectJump(_) | Goal::FewFlaps(..) | Goal::LandSafe(_) => {
            format!("{:.0}/{:.0} m", p.now.min(p.target), p.target)
        }
        Goal::Meteor => String::new(),
        _ => format!("{:.0}/{:.0}", p.now.min(p.target), p.target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use crate::*;

    fn flying(x: f32, y: f32, vx: f32, vy: f32) -> Game {
        let mut g = Game::new(PILOTS[0], 1);
        g.entities.clear();
        g.phase = Phase::Flying;
        g.spawned_to = 1.0e9;
        g.pos = Vec2::new(x, y);
        g.vel = Vec2::new(vx, vy);
        g
    }

    fn put(g: &mut Game, kind: EntityKind, x: f32, y: f32) {
        g.entities.push(Entity { kind, pos: Vec2::new(x, y), alive: true, dead_for: 0.0 });
    }

    #[test]
    fn an_order_completes_once_the_moment_it_is_met() {
        let mut g = flying(100.0, 30.0, 20.0, 0.0);
        g.set_orders([Some(Goal::Presents(3)), Some(Goal::Distance(190.0)), None]);
        for k in 0..3 {
            put(&mut g, EntityKind::Present, 101.0 + k as f32 * 2.0, 30.0);
        }
        let mut done = Vec::new();
        for _ in 0..60 {
            g.glide = 10.0;
            g.step(1.0 / 60.0);
            for e in g.events.drain(..) {
                if let Event::MissionComplete(i) = e {
                    done.push(i);
                }
            }
        }
        assert_eq!(done, vec![0], "only the presents order, and only once");
        assert_eq!(g.orders_done, [true, false, false]);
        assert_eq!(g.stats.best_chain, 3, "three presents in a row are a chain");
        let p = progress(Goal::Distance(190.0), &g);
        assert!(!p.done && p.now > 100.0 && p.frac() > 0.5);
        assert_eq!(readout(Goal::Presents(3), &progress(Goal::Presents(3), &g)), "done");
    }

    #[test]
    fn clean_and_frugal_orders_fail_on_the_first_slip() {
        let mut g = flying(100.0, 30.0, 20.0, 0.0);
        g.set_orders([Some(Goal::Clean(300.0)), Some(Goal::FewFlaps(2, 300.0)), None]);
        for _ in 0..3 {
            g.flap_cooldown = 0.0;
            g.flap();
        }
        assert!(progress(Goal::FewFlaps(2, 300.0), &g).failed);
        assert!(!progress(Goal::Clean(300.0), &g).failed);
        put(&mut g, EntityKind::Stormcloud, 100.5, 30.0);
        g.step(1.0 / 60.0);
        assert_eq!(g.stats.storms_hit, 1);
        let p = progress(Goal::Clean(300.0), &g);
        assert!(p.failed && !p.done);
        assert_eq!(readout(Goal::Clean(300.0), &p), "missed");
    }

    #[test]
    fn landing_unhurt_is_judged_when_the_flight_ends() {
        // A gentle roll-out past the mark.
        let mut g = flying(240.0, 1.5, 8.0, -1.0);
        g.set_orders([Some(Goal::LandSafe(200.0)), None, None]);
        while g.phase != Phase::Landed {
            g.step(1.0 / 120.0);
        }
        assert!(!g.crashed && g.orders_done[0]);
        assert!(g.events.contains(&Event::MissionComplete(0)));
        // Head first into a hole: no.
        let mut g = flying(240.0, 30.0, 8.0, -30.0);
        g.set_orders([Some(Goal::LandSafe(200.0)), None, None]);
        while g.phase != Phase::Landed {
            g.step(1.0 / 120.0);
        }
        assert!(g.crashed && !g.orders_done[0]);
        assert!(progress(Goal::LandSafe(200.0), &g).failed);
    }

    #[test]
    fn storms_flown_past_and_close_shaves_are_counted() {
        let mut g = flying(100.0, 30.0, 30.0, 0.0);
        g.stamina = 0.5;
        put(&mut g, EntityKind::Stormcloud, 106.0, 30.0 + 5.5); // a whisker over the dog
        put(&mut g, EntityKind::Stormcloud, 112.0, 30.0 - 15.0); // well clear below
        put(&mut g, EntityKind::Stormcloud, 118.0, 30.0 + 40.0); // nowhere near
        let mut shaves = 0;
        for _ in 0..40 {
            g.glide = 10.0;
            g.pos.y = 30.0;
            g.vel.y = 0.0;
            g.step(1.0 / 60.0);
            shaves += g.events.drain(..).filter(|e| *e == Event::CloseShave).count();
        }
        assert_eq!(g.stats.storms_hit, 0);
        assert_eq!(g.stats.storms_passed, 2);
        assert_eq!(g.stats.close_shaves, 1);
        assert_eq!(shaves, 1);
        assert!(g.stamina > 0.5 + NEAR_MISS_BONUS - 0.01, "a close shave pays stamina");
    }

    #[test]
    fn the_order_book_never_runs_out_and_ranks_climb() {
        for i in 0..200 {
            assert!(!describe(goal(i)).is_empty());
        }
        assert_eq!(describe(Goal::Distance(400.0)), "Reach the Bow");
        assert_eq!(describe(Goal::Distance(190.0)), "Reach Stephen Avenue");
        assert_eq!(describe(Goal::Distance(1234.0)), "Fly 1234 m");
        // The endless book keeps raising the bar.
        let lap = |i: u32| match goal(i) {
            Goal::Distance(d) => d,
            other => panic!("{other:?}"),
        };
        let first = CAMPAIGN.len() as u32;
        assert!(lap(first + 8) > lap(first));
        assert_eq!(rank_for(0), 0);
        assert_eq!(rank_for(3), 1);
        assert_eq!(rank_for(1000), RANKS.len() - 1);
        assert_eq!(to_next_rank(5), Some(2));
        assert_eq!(to_next_rank(1000), None);
    }

    #[test]
    fn a_stomp_saves_an_ordinary_descent_but_not_a_meteor_or_a_crawl() {
        let land = |vx: f32, vy: f32, stomp: bool| {
            // Before the frozen lakes and drifts start: plain snow.
            let mut g = flying(100.0, 1.6, vx, vy);
            if stomp {
                g.stomp = STOMP_WINDOW;
            }
            for _ in 0..30 {
                g.step(1.0 / 120.0);
                if g.phase == Phase::Landed || g.vel.y >= 0.0 {
                    break;
                }
            }
            g
        };
        // Terminal fall of a plain descent: about 25 m/s.
        assert!(land(20.0, -25.0, false).crashed);
        let saved = land(20.0, -25.0, true);
        assert!(!saved.crashed && saved.vel.y > 15.0 && saved.stats.stomps == 1);
        assert!(land(20.0, -70.0, true).crashed, "no stomp survives a meteor");
        // Too slow to skip: the tuck still cushions, but there's no spring.
        let crawl = land(5.0, -12.0, true);
        assert!(!crawl.crashed && crawl.stats.stomps == 0);
        assert!(crawl.vel.y < saved.vel.y * 0.6);
    }
}
