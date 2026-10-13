//! Tests for the ear-flap and booster rules rebalanced for replay value:
//! skill beats mashing, a stall sinks, space comes back down.

use crate::math::Vec2;
use crate::*;

fn flying(alt: f32, vx: f32, vy: f32) -> Game {
    let mut g = Game::new(PILOTS[0], 1);
    g.entities.clear();
    g.phase = Phase::Flying;
    g.spawned_to = 1.0e9;
    g.pos = Vec2::new(300.0, alt + PILOT_RADIUS);
    g.vel = Vec2::new(vx, vy);
    g
}

#[test]
fn a_wiggle_lifts_a_stall_but_never_pushes_it_along() {
    let mut stall = flying(40.0, 8.0, 0.0);
    stall.flap();
    assert_eq!(stall.vel.x, 8.0, "no forward kick near a hover");
    assert!(stall.vel.y > 0.0 && stall.vel.y < FLAP_IMPULSE * 0.3, "a stall barely flutters: {}", stall.vel.y);
    let mut cruise = flying(40.0, 30.0, 0.0);
    cruise.flap();
    assert!(cruise.vel.x > 30.5, "a cruising wiggle still pushes");
    let mut fast = flying(40.0, 80.0, 0.0);
    fast.flap();
    assert_eq!(fast.vel.x, 80.0, "at top speed the ears only lift");
    assert!(fast.vel.y > cruise.vel.y, "and lift more");
}

#[test]
fn ears_need_air() {
    let mut low = flying(50.0, 40.0, 0.0);
    let mut high = flying(FLAP_THIN_TOP + 50.0, 40.0, 0.0);
    low.flap();
    high.flap();
    assert!(high.vel.y < low.vel.y * 0.15, "high {} low {}", high.vel.y, low.vel.y);
}

#[test]
fn rockets_have_a_top_speed() {
    let mut slow = flying(40.0, 30.0, 0.0);
    let mut fast = flying(40.0, ROCKET_TOP_SPEED, 0.0);
    slow.apply_hit(EntityKind::Firework);
    fast.apply_hit(EntityKind::Firework);
    assert!(slow.vel.x > 55.0);
    assert_eq!(fast.vel.x, ROCKET_TOP_SPEED, "no kick past the top speed");
    for _ in 0..120 {
        fast.step(1.0 / 120.0);
    }
    assert!(fast.vel.x < ROCKET_TOP_SPEED, "no thrust past it either; drag wins");
}

#[test]
fn the_wiggle_streak_is_a_top_up_not_a_refill() {
    let mut g = flying(40.0, 30.0, 0.0);
    g.flaps = WIGGLE_STREAK - 1;
    g.stamina = 0.3;
    g.flap();
    assert!(g.events.contains(&Event::WigglePower));
    let expect = 0.3 - FLAP_COST + WIGGLE_POWER_REFILL;
    assert!((g.stamina - expect).abs() < 1e-4, "stamina {}", g.stamina);
}

#[test]
fn mashing_cannot_keep_a_dog_up_for_ever() {
    // An empty sky, pressing every frame from a good height and speed.
    let mut g = flying(60.0, 40.0, 0.0);
    let mut t = 0.0;
    while g.phase == Phase::Flying && t < 120.0 {
        g.action();
        g.step(1.0 / 120.0);
        t += 1.0 / 120.0;
    }
    assert_eq!(g.phase, Phase::Landed, "still up after {t} s of mashing");
    assert!(t < 30.0, "mashing kept the dog up for {t} s");
}
