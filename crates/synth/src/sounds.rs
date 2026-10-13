//! Every sound in the game, built from layers in `dsp`. Each one-shot
//! takes a variant number: the variant reseeds its noise and nudges pitch,
//! filters and timing, so repeats never sound like the same sample.
//!
//! Levels: `finish(buf, rms_db, peak_db)` sets the loudest 50 ms RMS, so
//! the numbers below are the relative mix of the one-shots before the
//! runtime's per-event volumes.

use crate::dsp::*;

/// Semitone steps of the major-pentatonic ladder the present chime climbs.
pub const LADDER: [f32; crate::PRESENT_STEPS] = [0.0, 2.0, 4.0, 7.0, 9.0, 12.0, 14.0, 16.0, 19.0];

/// Per-variant randomness: pitch in cents, ranges, seeds.
struct Var(Rng);

impl Var {
    fn new(sound: u32, v: u32) -> Var {
        Var(Rng::new(sound.wrapping_mul(7919) ^ v.wrapping_mul(104_729).wrapping_add(17)))
    }
    /// A pitch multiplier within +-`cents`.
    fn p(&mut self, cents: f32) -> f32 {
        semis(1.0, self.0.range(-cents, cents) / 100.0)
    }
    fn a(&mut self, lo: f32, hi: f32) -> f32 {
        self.0.range(lo, hi)
    }
    fn seed(&mut self) -> u32 {
        (self.0.unit() * 4.0e9) as u32
    }
}

/// A pitch-dropping sine thump with drive: the body of every impact.
fn body(f0: f32, f1: f32, sweep: f32, tau: f32, len: f32, amount: f32) -> Vec<f32> {
    let mut o = Osc::default();
    render(len, |_, t| {
        let f = glide(f0, f1, (t / sweep).sqrt());
        drive(o.sine(f) * perc(t, 0.001, tau, len), amount)
    })
}

/// White noise through a low-pass that closes `f0` -> `f1` over `sweep`.
fn closing_noise(f0: f32, f1: f32, sweep: f32, tau: f32, len: f32, seed: u32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut lp = Svf::new(f0, 0.9);
    render(len, |i, t| {
        if i % 4 == 0 {
            lp.tune(glide(f0, f1, t / sweep), 0.9);
        }
        lp.lp(rng.next()) * perc(t, 0.0008, tau, len)
    })
}

/// A sharp transient: white noise a few ms long, band-limited between
/// `hp` and 9 kHz so it snaps without a digital fizz.
fn crack(hp: f32, tau: f32, seed: u32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut f = Svf::new(hp, 0.7);
    let mut lp = Svf::new(9000.0, 0.7);
    render(tau * 8.0, |_, t| lp.lp(f.hp(rng.next())) * perc(t, 0.0002, tau, tau * 8.0))
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

/// A crisp paper-and-wood tick for buttons and toggles.
pub fn click(v: u32) -> Vec<f32> {
    let mut x = Var::new(1, v);
    let p = x.p(60.0);
    let mut fm = Fm::new();
    let mut wood = Osc::default();
    let mut rng = Rng::new(x.seed());
    let mut bp = Svf::new(4200.0 * p, 1.4);
    let mut b = render(0.09, |_, t| {
        let tick = bp.bp(rng.next()) * perc(t, 0.0003, 0.0025, 0.02);
        let tone = fm.next(1750.0 * p, 1.5, 2.0 * (-t / 0.01).exp()) * perc(t, 0.0008, 0.012, 0.08);
        let w = wood.sine(620.0 * p) * perc(t, 0.001, 0.02, 0.09);
        tick * 0.8 + tone * 0.5 + w * 0.45
    });
    reverb(&mut b, 0.3, 0.5, 0.12, 0.005, 0.25);
    finish(b, -22.0, -4.0)
}

/// Choosing a pilot: two rising marimba notes with a glint on top.
pub fn select(v: u32) -> Vec<f32> {
    let mut x = Var::new(2, v);
    let p = x.p(15.0);
    let mut b = mallet(783.99 * p, 0.35, x.seed());
    mix(&mut b, &mallet(1174.66 * p, 0.5, x.seed()), 0.075, 0.9);
    mix(&mut b, &bell(2349.3 * p, 3.5, 1.4, 0.35), 0.075, 0.12);
    reverb(&mut b, 0.45, 0.4, 0.18, 0.01, 0.6);
    finish(b, -19.0, -3.0)
}

/// Locking the push power: a two-stage latch ("ka-chunk"), a little
/// pressure release and a confirming ting.
pub fn lock(v: u32) -> Vec<f32> {
    let mut x = Var::new(3, v);
    let latch = |f: f32, len: f32, seed: u32| {
        let mut rng = Rng::new(seed);
        let mut ring = Svf::new(f, 14.0);
        let mut bp = Svf::new(f * 1.6, 1.2);
        render(len, move |_, t| {
            let n = rng.next() * perc(t, 0.0002, 0.0015, 0.01);
            ring.bp(n) * 2.5 * perc(t, 0.0003, len * 0.25, len) + bp.bp(n)
        })
    };
    let mut b = latch(2600.0 * x.p(40.0), 0.05, x.seed());
    mix(&mut b, &latch(1250.0 * x.p(40.0), 0.07, x.seed()), 0.038, 1.0);
    mix(&mut b, &body(170.0, 70.0, 0.08, 0.035, 0.14, 2.0), 0.038, 0.9);
    mix(&mut b, &whoosh(500.0, 2400.0, 1.4, 0.16, 0.04, x.seed()), 0.0, 0.18);
    mix(&mut b, &bell(2093.0, 2.0, 1.6, 0.6), 0.06, 0.28);
    mix(&mut b, &bell(3136.0, 3.5, 1.2, 0.45), 0.1, 0.14);
    reverb(&mut b, 0.4, 0.4, 0.15, 0.01, 0.5);
    finish(b, -16.0, -2.0)
}

// ---------------------------------------------------------------------------
// The ride and the launch
// ---------------------------------------------------------------------------

/// Sled runners on packed roof snow, as a seamless loop. `fast` is the
/// bright hiss near the lip; the slow one is lower and crunchier, with
/// runner chatter. The runtime cross-fades them by speed, so the hiss
/// climbs in pitch and level as the sled picks up speed.
pub fn slide_loop(fast: bool) -> Vec<f32> {
    let seed = if fast { 0x51_1D } else { 0x51_1C };
    let mut pink = Pink::new(seed);
    let mut white = Rng::new(seed + 1);
    let mut brown = Brown::new(seed + 2);
    let mut grains = Rng::new(seed + 3);
    let mut drift = Drift::new(seed + 4, 2.5);
    let mut chatter = Drift::new(seed + 5, 16.0);
    let (centre, q, grain_f, grain_rate, rumble_f) =
        if fast { (2900.0, 0.8, 4200.0, 900.0, 230.0) } else { (950.0, 0.9, 1900.0, 420.0, 140.0) };
    let mut hiss = Svf::new(centre, q);
    let mut air = Svf::new(5500.0, 0.7);
    let mut crunch = Svf::new(grain_f, 1.3);
    let mut rumble = Svf::new(rumble_f, 0.8);
    let mut level = 0.0f32;
    let mut amp = 0.0f32;
    let decay = (-1.0 / (0.0018 * SR)).exp();
    make_loop(6.0, 0.3, -16.0, |i, _| {
        let d = drift.next();
        if i % 8 == 0 {
            hiss.tune(centre * (1.0 + 0.18 * d), q);
        }
        let h = hiss.bp(pink.next() * 3.0);
        if grains.unit() < grain_rate * (0.7 + 0.3 * d) / SR {
            level = 1.0;
            amp = grains.range(0.2, 1.0);
        }
        level *= decay;
        let c = crunch.bp(white.next() * level * amp) * 2.2;
        let ch = 0.7 + 0.3 * chatter.next();
        let r = rumble.lp(brown.next()) * ch;
        if fast {
            h * 0.9 + c * 0.45 + r * 0.55 + air.hp(white.next()) * 0.12
        } else {
            h * 0.6 + c * 0.85 + r * 0.7
        }
    })
}

/// Leaving the lip: a wooden thock off the kicker, a driven low boom, an
/// air whoosh that tears upward, a spray of snow, and a slap echo off the
/// factory wall.
pub fn launch(v: u32) -> Vec<f32> {
    let mut x = Var::new(10, v);
    let p = x.p(80.0);
    let mut b = body(190.0 * p, 110.0 * p, 0.05, 0.018, 0.08, 1.5);
    mix(&mut b, &crack(700.0, 0.004, x.seed()), 0.0, 0.5);
    mix(&mut b, &body(95.0 * p, 44.0, 0.35, 0.12, 0.5, 2.6), 0.005, 0.6);
    mix(&mut b, &whoosh(220.0 * p, 3400.0 * p, 1.5, 0.75, 0.1, x.seed()), 0.0, 1.5);
    mix(&mut b, &whoosh(900.0, 5200.0, 2.5, 0.5, 0.05, x.seed()), 0.05, 0.35);
    let spray = crackle(0.7, 1400.0, 3600.0 * p, x.seed(), |t| (1.0 - t / 0.7).max(0.0).powi(2));
    mix(&mut b, &spray, 0.02, 0.4);
    echo(&mut b, 0.11 * x.a(0.9, 1.1), 0.22, 0.22, 2600.0);
    reverb(&mut b, 0.5, 0.5, 0.1, 0.01, 0.6);
    finish(b, -15.0, -1.0)
}

/// The PERFECT sting: a bright synth-brass chord blat, a glockenspiel run
/// over it, a sparkling "shing" and a low punch, in a hall.
pub fn perfect(v: u32) -> Vec<f32> {
    let mut x = Var::new(11, v);
    let mut b = brass(&[523.25, 659.25, 783.99, 1046.5], 1.2, 0.34, 0.85, x.seed());
    mix(&mut b, &body(130.0, 55.0, 0.12, 0.06, 0.25, 2.0), 0.0, 0.7);
    for (i, f) in [2093.0, 2637.0, 3136.0, 4186.0].iter().enumerate() {
        mix(&mut b, &bell(*f, 3.5, 2.0, 0.7), 0.02 + i as f32 * 0.045, 0.3 - i as f32 * 0.03);
    }
    let mut rng = Rng::new(x.seed());
    let mut hp = Svf::new(7000.0, 0.7);
    let shing = render(0.9, |_, t| hp.hp(rng.next()) * perc(t, 0.003, 0.18, 0.9));
    mix(&mut b, &shing, 0.0, 0.16);
    reverb(&mut b, 0.78, 0.3, 0.28, 0.02, 1.2);
    finish(b, -14.0, -1.0)
}

/// A missed jump: a comic slide whistle down, breathy and wobbling.
pub fn botch(v: u32) -> Vec<f32> {
    let mut x = Var::new(12, v);
    let p = x.p(50.0);
    let mut o = Osc::default();
    let mut pink = Pink::new(x.seed());
    let mut breath = Svf::new(1000.0, 6.0);
    let mut b = render(1.0, |i, t| {
        let k = smooth(t / 0.85);
        let f = glide(1500.0 * p, 240.0 * p, k) * (1.0 + 0.018 * (TAU * 6.0 * t).sin());
        if i % 8 == 0 {
            breath.tune(f, 6.0);
        }
        let e = adsr(t, 0.03, 0.4, 0.8, 0.15, 0.82);
        (o.sine(f) * 0.8 + breath.bp(pink.next() * 3.0) * 0.3) * e
    });
    reverb(&mut b, 0.5, 0.4, 0.15, 0.01, 0.5);
    finish(b, -19.0, -3.0)
}

// ---------------------------------------------------------------------------
// Flight
// ---------------------------------------------------------------------------

/// The ear-flap swoosh: two quick strokes of the ears (fabric-like flutter
/// on a down-swept band of air) over a soft body whomp.
pub fn flap(v: u32) -> Vec<f32> {
    let mut x = Var::new(20, v);
    let p = x.p(150.0);
    let rate = x.a(26.0, 40.0);
    let stroke = |f0: f32, f1: f32, len: f32, seed: u32| {
        let mut pink = Pink::new(seed);
        let mut bp = Svf::new(f0, 1.8);
        render(len, move |i, t| {
            if i % 4 == 0 {
                bp.tune(glide(f0, f1, t / len), 1.8);
            }
            let flutter = 1.0 - 0.45 * (0.5 + 0.5 * (TAU * rate * t).sin()) * (1.0 - t / len);
            bp.bp(pink.next() * 3.0) * flutter * perc(t, 0.004, len * 0.3, len)
        })
    };
    let mut b = stroke(2300.0 * p, 600.0 * p, 0.14, x.seed());
    mix(&mut b, &stroke(1800.0 * p, 520.0 * p, 0.12, x.seed()), x.a(0.045, 0.065), 0.55);
    mix(&mut b, &body(165.0 * p, 85.0 * p, 0.05, 0.025, 0.08, 1.2), 0.0, 0.45);
    reverb(&mut b, 0.2, 0.6, 0.06, 0.0, 0.15);
    finish(b, -24.0, -6.0)
}

/// A gust of passing air: a band that swells up and falls back like a
/// doppler pass, with a darker body under it.
pub fn gust(v: u32) -> Vec<f32> {
    let mut x = Var::new(21, v);
    let p = x.p(120.0);
    let len = x.a(0.38, 0.48);
    let mut pink = Pink::new(x.seed());
    let mut brown = Brown::new(x.seed());
    let mut bp = Svf::new(400.0, 1.1);
    let mut lp = Svf::new(320.0, 0.7);
    let b = render(len, |i, t| {
        let k = t / len;
        if i % 4 == 0 {
            let f = if k < 0.4 { glide(350.0, 1500.0, k / 0.4) } else { glide(1500.0, 650.0, (k - 0.4) / 0.6) };
            bp.tune(f * p, 1.1);
        }
        let e = adsr(t, len * 0.2, len * 0.3, 0.45, len * 0.45, len * 0.5);
        (bp.bp(pink.next() * 3.0) + lp.lp(brown.next()) * 0.4) * e
    });
    finish(b, -26.0, -6.0)
}

/// Wind loops, cross-faded by airspeed. 0: a low brown rumble that gusts,
/// 1: a breathing mid band, 2: a howling two-whistle top for real speed.
pub fn wind_loop(layer: usize) -> Vec<f32> {
    let s = 0x3100 + layer as u32 * 16;
    let mut brown = Brown::new(s);
    let mut pink = Pink::new(s + 1);
    let mut white = Rng::new(s + 2);
    let mut d1 = Drift::new(s + 3, 0.7);
    let mut d2 = Drift::new(s + 4, 1.3);
    let mut d3 = Drift::new(s + 5, 0.45);
    let mut f1 = Svf::new(300.0, 0.7);
    let mut f2 = Svf::new(500.0, 0.7);
    let mut f3 = Svf::new(4000.0, 0.7);
    make_loop(6.0, 0.5, -16.0, |i, _| {
        let (a, b, c) = (d1.next(), d2.next(), d3.next());
        match layer {
            0 => {
                if i % 8 == 0 {
                    f1.tune(260.0 * (1.0 + 0.3 * a), 0.7);
                    f2.tune(700.0 * (1.0 + 0.25 * b), 0.6);
                }
                let gust = 0.6 + 0.4 * smooth(0.5 + 0.7 * c);
                (f1.lp(brown.next()) + f2.lp(pink.next()) * 0.9) * gust
            }
            1 => {
                if i % 8 == 0 {
                    f1.tune(650.0 * (1.0 + 0.35 * a), 1.3);
                    f2.tune(1250.0 * (1.0 + 0.3 * b), 1.8);
                }
                let n = pink.next() * 3.0;
                let gust = 0.45 + 0.55 * smooth(0.5 + 0.8 * c);
                (f1.bp(n) + f2.bp(n) * 0.6) * gust
            }
            _ => {
                if i % 4 == 0 {
                    f1.tune(1500.0 * (1.0 + 0.4 * a), 14.0);
                    f2.tune(2350.0 * (1.0 + 0.3 * b), 18.0);
                }
                let n = white.next();
                let howl = 0.5 + 0.5 * smooth(0.5 + 0.8 * c);
                (f1.bp(n) * 0.9 + f2.bp(n) * 0.6) * howl + f3.hp(n) * 0.08
            }
        }
    })
}

/// Rocket thrust (and the meteor's fire), looped: a driven roar with
/// combustion flutter, a low engine thrum, crackle and a hiss on top.
pub fn rocket_loop() -> Vec<f32> {
    let mut brown = Brown::new(0x40C);
    let mut pink = Pink::new(0x40D);
    let mut white = Rng::new(0x40E);
    let mut flutter = Drift::new(0x40F, 45.0);
    let mut sway = Drift::new(0x410, 1.5);
    let mut roar = Svf::new(700.0, 0.8);
    let mut hiss = Svf::new(4500.0, 0.7);
    let mut crk = Svf::new(2600.0, 1.2);
    let mut thrum_f = Svf::new(220.0, 0.9);
    let mut thrum = Osc::default();
    let mut level = 0.0f32;
    let decay = (-1.0 / (0.002 * SR)).exp();
    make_loop(4.0, 0.3, -15.0, |i, _| {
        let s = sway.next();
        if i % 8 == 0 {
            roar.tune(700.0 * (1.0 + 0.2 * s), 0.8);
        }
        let fl = 0.75 + 0.25 * flutter.next();
        let r = drive(roar.lp(brown.next() * 0.8 + pink.next() * 0.6) * 1.6, 2.5) * fl;
        if white.unit() < 70.0 / SR {
            level = white.range(0.4, 1.0);
        }
        level *= decay;
        let c = crk.bp(white.next() * level) * 2.0;
        let th = thrum_f.lp(thrum.saw(55.0 * (1.0 + 0.02 * s))) * 0.3;
        r + c * 0.5 + hiss.hp(white.next()) * 0.1 + th
    })
}

/// Grabbing a firework rocket: a fizzing fuse, the ignition "fwoomp" and
/// a whistle tearing upward, then two more whistles for the shells that
/// climb ahead of the dog (they burst on their own clock).
pub fn firework(v: u32) -> Vec<f32> {
    let mut x = Var::new(30, v);
    let mut white = Rng::new(x.seed());
    let mut fizz = Svf::new(5200.0, 1.0);
    let mut spit = Drift::new(x.seed(), 60.0);
    let mut b = render(0.26, |_, t| {
        let g = spit.next().max(0.0);
        fizz.bp(white.next()) * g * g * 2.0 * adsr(t, 0.01, 0.1, 0.8, 0.06, 0.2)
    });
    mix(&mut b, &whoosh(250.0, 2600.0, 0.9, 0.4, 0.02, x.seed()), 0.16, 1.0);
    mix(&mut b, &body(95.0, 45.0, 0.15, 0.06, 0.22, 2.0), 0.16, 0.8);
    let whistle = |f0: f32, f1: f32, len: f32, seed: u32| {
        let mut o = Osc::default();
        let mut pink = Pink::new(seed);
        let mut bp = Svf::new(f0, 8.0);
        render(len, move |i, t| {
            let f = glide(f0, f1, t / len) * (1.0 + 0.01 * (TAU * 11.0 * t).sin());
            if i % 8 == 0 {
                bp.tune(f, 8.0);
            }
            (o.sine(f) * 0.7 + bp.bp(pink.next() * 3.0) * 0.35) * adsr(t, 0.04, 0.2, 0.8, len * 0.25, len * 0.75)
        })
    };
    mix(&mut b, &whistle(700.0, 2600.0, 0.7, x.seed()), 0.2, 0.55);
    mix(&mut b, &whistle(1150.0, 3000.0, 0.3, x.seed()), 0.16, 0.25);
    mix(&mut b, &whistle(1250.0, 3300.0, 0.3, x.seed()), 0.44, 0.22);
    reverb(&mut b, 0.55, 0.4, 0.15, 0.02, 0.8);
    finish(b, -16.0, -1.0)
}

/// A firework shell bursting: a hard crack, a driven low boom, a long
/// tail of crackling stars, an echo off the city and a big sky.
pub fn pop(v: u32) -> Vec<f32> {
    let mut x = Var::new(31, v);
    let p = x.p(250.0);
    let mut b = crack(1500.0, 0.003, x.seed());
    let boom = closing_noise(900.0 * p, 140.0 * p, 0.25, 0.18, 1.0, x.seed());
    mix(&mut b, &boom, 0.0, 2.2);
    mix(&mut b, &body(80.0 * p, 36.0, 0.3, 0.12, 0.5, 2.0), 0.0, 0.9);
    let stars = x.a(1.1, 1.8);
    let dens = x.a(110.0, 220.0);
    let tail = crackle(stars, dens, 4500.0 * p, x.seed(), |t| (1.0 - t / stars).max(0.0).powf(1.5) * smooth(t / 0.1));
    mix(&mut b, &tail, x.a(0.1, 0.2), 0.55);
    echo(&mut b, 0.32 * x.a(0.8, 1.2), 0.3, 0.35, 1800.0);
    reverb(&mut b, 0.8, 0.5, 0.2, 0.03, 1.5);
    finish(b, -14.0, -1.0)
}

/// A storm cloud strike: a crack, an electric arc that sizzles and gates,
/// a buzzing hum, and rolling thunder in a big space.
pub fn zap(v: u32) -> Vec<f32> {
    let mut x = Var::new(40, v);
    let mut b = crack(2000.0, 0.006, x.seed());
    for s in b.iter_mut() {
        *s *= 1.3;
    }
    let mut white = Rng::new(x.seed());
    let mut gate_rng = Rng::new(x.seed());
    let mut arc_bp = Svf::new(3200.0, 0.9);
    let mut gate = 1.0f32;
    let arc = render(0.4, |_, t| {
        if gate_rng.unit() < 420.0 / SR {
            gate = if gate > 0.5 { gate_rng.range(0.0, 0.3) } else { 1.0 };
        }
        arc_bp.bp(white.next()) * gate * perc(t, 0.001, 0.12, 0.4)
    });
    mix(&mut b, &arc, 0.0, 0.7);
    let mut o1 = Osc::default();
    let mut o2 = Osc::default();
    let mut buzz_bp = Svf::new(1100.0, 1.5);
    let jit = x.a(56.0, 64.0);
    let buzz = render(0.55, |_, t| {
        let s = o1.saw(jit) + o2.saw(jit * 2.01) * 0.7;
        let trem = 0.5 + 0.5 * (TAU * 28.0 * t).sin();
        drive(buzz_bp.bp(s) * 1.5, 3.0) * trem * perc(t, 0.004, 0.16, 0.55)
    });
    mix(&mut b, &buzz, 0.0, 0.45);
    let mut brown = Brown::new(x.seed());
    let mut pink = Pink::new(x.seed());
    let mut lp1 = Svf::new(160.0, 0.8);
    let mut lp2 = Svf::new(420.0, 0.7);
    let mut bumps = Drift::new(x.seed(), 7.0);
    let thunder = render(2.4, |_, t| {
        let rumble = 0.5 + 0.5 * bumps.next().abs();
        (lp1.lp(brown.next()) * 1.4 + lp2.lp(pink.next()) * 0.6) * rumble * perc(t, 0.06, 0.7, 2.4)
    });
    mix(&mut b, &thunder, 0.05, 1.1);
    reverb(&mut b, 0.85, 0.6, 0.18, 0.02, 1.5);
    finish(b, -14.0, -1.0)
}

/// Freezing solid: an icy down-whoosh, ice crystallizing in a rush of
/// tiny crackles, a glassy shimmer cluster, a low "thoom" as it sets and a
/// final clink. Bright, long tail.
pub fn freeze(v: u32) -> Vec<f32> {
    let mut x = Var::new(50, v);
    let mut b = whoosh(6500.0, 1400.0, 2.5, 0.5, 0.01, x.seed());
    for s in b.iter_mut() {
        *s *= 0.5;
    }
    let ice = crackle(0.75, 900.0, 7000.0, x.seed(), |t| smooth(t / 0.45) * (1.0 - smooth((t - 0.55) / 0.15)));
    mix(&mut b, &ice, 0.05, 0.35);
    for (i, f) in [2637.0, 3322.4, 4186.0, 5274.0].iter().enumerate() {
        mix(&mut b, &bell(*f * x.p(10.0), if i % 2 == 0 { 3.5 } else { 1.41 }, 1.6, 0.9), 0.08 + i as f32 * 0.07, 0.22);
    }
    let mut fm = Fm::new();
    let thoom = render(0.5, |_, t| fm.next(glide(220.0, 110.0, t / 0.4), 1.41, 2.0 * (-t / 0.1).exp()) * perc(t, 0.005, 0.15, 0.5));
    mix(&mut b, &thoom, 0.02, 0.4);
    mix(&mut b, &bell(4400.0, 2.76, 2.5, 0.5), 0.6, 0.35);
    mix(&mut b, &bell(6600.0, 2.0, 1.5, 0.35), 0.62, 0.2);
    reverb(&mut b, 0.85, 0.15, 0.3, 0.02, 1.6);
    finish(b, -17.0, -2.0)
}

/// Breaking out of the ice: a crack and a spray of glassy shards.
pub fn thaw(v: u32) -> Vec<f32> {
    let mut x = Var::new(51, v);
    let mut b = crack(3000.0, 0.01, x.seed());
    let mut rng = Rng::new(x.seed());
    for _ in 0..26 {
        let at = rng.unit().powf(1.8) * 0.7;
        let len = rng.range(0.05, 0.2);
        let shard = bell(rng.range(2500.0, 9000.0), rng.range(2.2, 3.9), rng.range(2.0, 4.0), len);
        mix(&mut b, &shard, at, rng.range(0.15, 0.45) * (1.0 - at));
    }
    let tinkle = crackle(0.6, 400.0, 8000.0, x.seed(), |t| (1.0 - t / 0.6).max(0.0));
    mix(&mut b, &tinkle, 0.0, 0.25);
    reverb(&mut b, 0.6, 0.2, 0.22, 0.01, 1.0);
    finish(b, -18.0, -2.0)
}

/// The meteor igniting: a falling whistle through fire, a building roar
/// and spitting embers.
pub fn meteor(v: u32) -> Vec<f32> {
    let mut x = Var::new(52, v);
    let mut b = whoosh(2800.0, 500.0, 3.0, 1.2, 0.15, x.seed());
    for s in b.iter_mut() {
        *s *= 0.6;
    }
    let mut brown = Brown::new(x.seed());
    let mut lp = Svf::new(420.0, 0.8);
    let mut fl = Drift::new(x.seed(), 30.0);
    let roar = render(1.5, |_, t| drive(lp.lp(brown.next()) * 2.0, 2.5) * (0.75 + 0.25 * fl.next()) * adsr(t, 0.25, 0.5, 0.8, 0.4, 1.1));
    mix(&mut b, &roar, 0.0, 0.9);
    mix(&mut b, &crackle(1.2, 300.0, 3000.0, x.seed(), |t| smooth(t / 0.3)), 0.1, 0.3);
    reverb(&mut b, 0.5, 0.5, 0.12, 0.01, 0.6);
    finish(b, -16.0, -1.0)
}

// ---------------------------------------------------------------------------
// Ground contact
// ---------------------------------------------------------------------------

/// A bounce off the snow: a driven low body, a puff of snow and a crunch.
pub fn bounce(v: u32) -> Vec<f32> {
    let mut x = Var::new(60, v);
    let p = x.p(120.0);
    let mut b = body(110.0 * p, 45.0 * p, 0.12, 0.06, 0.25, 2.0);
    mix(&mut b, &closing_noise(2500.0 * p, 400.0, 0.15, 0.05, 0.2, x.seed()), 0.0, 0.9);
    mix(&mut b, &crackle(0.25, 1200.0, 2000.0 * p, x.seed(), |t| (1.0 - t / 0.25).max(0.0)), 0.005, 0.45);
    reverb(&mut b, 0.3, 0.6, 0.08, 0.01, 0.3);
    finish(b, -18.0, -2.0)
}

/// Coming to rest: a soft thud, then the sled scraping to a stop.
pub fn land(v: u32) -> Vec<f32> {
    let mut x = Var::new(61, v);
    let mut b = body(95.0, 42.0, 0.1, 0.05, 0.2, 1.6);
    mix(&mut b, &closing_noise(2000.0, 350.0, 0.12, 0.04, 0.15, x.seed()), 0.0, 0.7);
    let mut pink = Pink::new(x.seed());
    let mut bp = Svf::new(1200.0, 0.9);
    let scrape = render(0.75, |i, t| {
        if i % 8 == 0 {
            bp.tune(glide(1300.0, 450.0, t / 0.75), 0.9);
        }
        bp.bp(pink.next() * 3.0) * adsr(t, 0.03, 0.2, 0.6, 0.4, 0.35)
    });
    mix(&mut b, &scrape, 0.02, 0.45);
    mix(&mut b, &crackle(0.7, 900.0, 1800.0, x.seed(), |t| (1.0 - t / 0.7).max(0.0).powi(2)), 0.02, 0.35);
    reverb(&mut b, 0.3, 0.6, 0.08, 0.01, 0.3);
    finish(b, -20.0, -3.0)
}

/// The snow layers of a crash: a closing splat, a squelchy whump, the
/// crunch of packing snow, clods raining back down and a settling hiss.
fn splat_layers(x: &mut Var, p: f32) -> Vec<f32> {
    let mut b = closing_noise(3500.0 * p, 250.0, 0.25, 0.07, 0.3, x.seed());
    let mut pink = Pink::new(x.seed());
    let mut sq = Svf::new(900.0, 3.0);
    let squelch = render(0.2, |i, t| {
        if i % 4 == 0 {
            sq.tune(glide(950.0 * p, 280.0, t / 0.2), 3.0);
        }
        sq.bp(pink.next() * 3.0) * perc(t, 0.004, 0.06, 0.2)
    });
    mix(&mut b, &squelch, 0.0, 0.5);
    mix(&mut b, &crackle(0.5, 1500.0, 1800.0, x.seed(), |t| (1.0 - t / 0.5).max(0.0).powi(2)), 0.01, 0.5);
    mix(&mut b, &pitter(1.1, 14, 900.0, x.seed()), 0.12, 0.45);
    mix(&mut b, &whoosh(2500.0, 900.0, 0.8, 0.7, 0.05, x.seed()), 0.05, 0.25);
    b
}

/// SPLAT: head-first into deep snow.
pub fn crash(v: u32) -> Vec<f32> {
    let mut x = Var::new(70, v);
    let p = x.p(80.0);
    let mut b = body(130.0 * p, 45.0, 0.25, 0.09, 0.35, 3.0);
    for s in b.iter_mut() {
        *s *= 0.7;
    }
    let snow = splat_layers(&mut x, p);
    mix(&mut b, &snow, 0.0, 1.4);
    // Stuck head-first: the hind legs kick at the snow, muffled and
    // slowing down, so the crash hold isn't dead air.
    let mut at = x.a(0.45, 0.55);
    for k in 0..8 {
        let g = 0.4 * (1.0 - k as f32 / 9.0);
        let mut kick = closing_noise(1600.0 * x.p(200.0), 300.0, 0.05, 0.02, 0.07, x.seed());
        mix(&mut kick, &body(150.0 * x.p(150.0), 90.0, 0.03, 0.015, 0.05, 1.2), 0.0, 0.5);
        mix(&mut b, &kick, at, g);
        at += x.a(0.13, 0.2) * (1.0 + k as f32 * 0.08);
    }
    reverb(&mut b, 0.4, 0.6, 0.1, 0.01, 0.5);
    finish(b, -13.0, -1.0)
}

/// KABOOM: a meteor impact. A crack, a sub boom dropping to 26 Hz, a
/// driven blast closing from 5 kHz, rolling rumble, the snow splat, and
/// debris raining down for two seconds, with an echo off the city.
pub fn kaboom(v: u32) -> Vec<f32> {
    let mut x = Var::new(71, v);
    let mut b = crack(1500.0, 0.008, x.seed());
    mix(&mut b, &body(70.0, 26.0, 1.0, 0.45, 1.8, 2.5), 0.0, 1.3);
    let blast = closing_noise(5000.0, 150.0, 1.4, 0.35, 1.8, x.seed());
    let blast: Vec<f32> = blast.iter().map(|s| drive(*s * 2.0, 2.0)).collect();
    mix(&mut b, &blast, 0.0, 1.2);
    let mut brown = Brown::new(x.seed());
    let mut lp = Svf::new(110.0, 0.8);
    let rumble = render(3.2, |_, t| lp.lp(brown.next()) * 1.5 * perc(t, 0.15, 0.9, 3.2));
    mix(&mut b, &rumble, 0.0, 1.0);
    let snow = splat_layers(&mut x, 0.8);
    mix(&mut b, &snow, 0.0, 0.6);
    mix(&mut b, &pitter(2.2, 30, 1100.0, x.seed()), 0.25, 0.5);
    mix(&mut b, &crackle(1.6, 500.0, 2600.0, x.seed(), |t| (1.0 - t / 1.6).max(0.0)), 0.1, 0.35);
    echo(&mut b, 0.38, 0.3, 0.3, 1200.0);
    reverb(&mut b, 0.9, 0.55, 0.2, 0.03, 2.0);
    finish(b, -11.0, -1.0)
}

// ---------------------------------------------------------------------------
// Pickups
// ---------------------------------------------------------------------------

/// One rung of the present ladder: a glassy glockenspiel tine with an
/// octave chime and a fifth, a burst of high sparkles and a swish, in a
/// bright room. Higher rungs ring higher, sparkle more and add an octave.
pub fn present(step: usize, v: u32) -> Vec<f32> {
    let step = step.min(LADDER.len() - 1);
    let mut x = Var::new(80 + step as u32, v);
    let f = semis(1046.5, LADDER[step]) * x.p(4.0);
    let mut b = bell(f, 3.5, 2.2, 0.9);
    mix(&mut b, &bell(f * 2.0, 2.0, 1.2, 0.5), 0.0, 0.3);
    mix(&mut b, &bell(f * 1.5, 3.5, 1.5, 0.6), 0.035, 0.4);
    if step >= 4 {
        mix(&mut b, &bell(f * 4.0, 2.0, 1.0, 0.4), 0.06, 0.15);
    }
    let mut rng = Rng::new(x.seed());
    for _ in 0..(6 + step) {
        let sf = (f * rng.range(3.0, 6.0)).min(12000.0);
        let len = rng.range(0.04, 0.09);
        mix(&mut b, &bell(sf, 2.0, 1.0, len), 0.02 + rng.unit() * 0.28, rng.range(0.12, 0.25));
    }
    mix(&mut b, &whoosh(3000.0, 9000.0, 1.5, 0.12, 0.02, x.seed()), 0.0, 0.12);
    echo(&mut b, 0.09, 0.3, 0.18, 6000.0);
    reverb(&mut b, 0.6, 0.2, 0.2, 0.01, 0.9);
    finish(b, -19.0 + step as f32 * 0.25, -2.0)
}

/// A trampoline: a sproingy FM boing with a damped wobble, spring ring
/// and a thump.
pub fn trampoline(v: u32) -> Vec<f32> {
    let mut x = Var::new(90, v);
    let p = x.p(180.0);
    let mut fm = Fm::new();
    let mut tri = Osc::default();
    let mut b = render(0.7, |_, t| {
        let rise = glide(160.0, 440.0, t / 0.06);
        let settle = 300.0 * (1.0 + 0.25 * (-t / 0.15).exp() * (TAU * 14.0 * t).sin());
        let f = if t < 0.06 { rise } else { settle } * p;
        (fm.next(f, 1.0, 1.5 * (-t / 0.1).exp()) * 0.7 + tri.tri(f) * 0.3) * perc(t, 0.003, 0.18, 0.7)
    });
    mix(&mut b, &bell(2300.0, 1.9, 1.5, 0.4), 0.0, 0.12);
    mix(&mut b, &bell(3100.0, 1.9, 1.5, 0.35), 0.01, 0.1);
    mix(&mut b, &body(90.0, 50.0, 0.1, 0.04, 0.12, 1.5), 0.0, 0.6);
    reverb(&mut b, 0.4, 0.4, 0.12, 0.01, 0.4);
    finish(b, -16.0, -2.0)
}

/// A reindeer kick: two hoof clops, a jingle of sleigh bells and a whoosh.
pub fn reindeer(v: u32) -> Vec<f32> {
    let mut x = Var::new(91, v);
    let clop = |f: f32, seed: u32| {
        let mut rng = Rng::new(seed);
        let mut bp = Svf::new(f, 3.0);
        let mut o = Osc::default();
        render(0.08, move |_, t| (bp.bp(rng.next()) * 0.8 + o.sine(f * 0.5) * 0.5) * perc(t, 0.0005, 0.015, 0.08))
    };
    let mut b = clop(900.0, x.seed());
    mix(&mut b, &clop(1200.0, x.seed()), 0.075, 0.9);
    mix(&mut b, &jingle(0.5, 12, x.seed()), 0.02, 0.6);
    mix(&mut b, &whoosh(500.0, 2500.0, 1.0, 0.4, 0.05, x.seed()), 0.05, 0.4);
    reverb(&mut b, 0.5, 0.4, 0.15, 0.01, 0.6);
    finish(b, -17.0, -2.0)
}

/// An updraft: a wide, rising breath of air with a soft whistle and chime.
pub fn updraft(v: u32) -> Vec<f32> {
    let mut x = Var::new(92, v);
    let mut b = whoosh(200.0, 2000.0, 0.8, 0.9, 0.25, x.seed());
    let mut white = Rng::new(x.seed());
    let mut bp = Svf::new(600.0, 10.0);
    let whistle = render(0.8, |i, t| {
        if i % 8 == 0 {
            bp.tune(glide(600.0, 1800.0, t / 0.8), 10.0);
        }
        bp.bp(white.next()) * adsr(t, 0.2, 0.3, 0.6, 0.25, 0.55)
    });
    mix(&mut b, &whistle, 0.05, 0.3);
    mix(&mut b, &bell(1568.0, 2.0, 0.8, 0.6), 0.2, 0.12);
    reverb(&mut b, 0.6, 0.4, 0.18, 0.01, 0.8);
    finish(b, -20.0, -3.0)
}

/// Santa's sleigh: jingle bells on the beat over a warm brass chord and a
/// church-bell arpeggio.
pub fn sleigh(v: u32) -> Vec<f32> {
    let mut x = Var::new(93, v);
    let mut b = brass(&[261.63, 329.63, 392.0, 523.25], 1.6, 0.9, 0.3, x.seed());
    for s in b.iter_mut() {
        *s *= 0.35;
    }
    for i in 0..6 {
        mix(&mut b, &jingle(0.25, if i % 2 == 0 { 9 } else { 5 }, x.seed()), i as f32 * 0.15, if i % 2 == 0 { 0.5 } else { 0.3 });
    }
    for (i, f) in [1046.5, 1318.5, 1568.0, 2093.0].iter().enumerate() {
        mix(&mut b, &bell(*f, 1.41, 1.2, 0.9), i as f32 * 0.09, 0.2);
    }
    reverb(&mut b, 0.7, 0.4, 0.22, 0.02, 1.0);
    finish(b, -17.0, -2.0)
}

/// A UFO: a warbling theremin swooping down, with a wobbling ring-mod
/// shimmer.
pub fn ufo(v: u32) -> Vec<f32> {
    let mut x = Var::new(94, v);
    let p = x.p(40.0);
    let mut o = Osc::default();
    let mut ring = Osc::default();
    let mut b = render(1.1, |_, t| {
        let f = glide(1100.0 * p, 350.0 * p, t / 1.0) * (1.0 + 0.04 * (TAU * 6.5 * t).sin());
        let s = o.sine(f);
        let wub = 0.75 + 0.25 * (TAU * 8.0 * t).sin();
        (s * 0.8 + s * ring.sine(f * 1.51) * 0.3) * wub * adsr(t, 0.05, 0.3, 0.8, 0.25, 0.85)
    });
    reverb(&mut b, 0.7, 0.3, 0.25, 0.02, 1.0);
    finish(b, -18.0, -2.0)
}

/// Angel wings: a harp glissando (Karplus-Strong plucks) up two octaves.
pub fn wings(v: u32) -> Vec<f32> {
    let mut x = Var::new(95, v);
    let scale = [0.0, 2.0, 4.0, 7.0, 9.0, 12.0, 14.0, 16.0, 19.0, 21.0, 24.0];
    let mut b = Vec::new();
    for (i, s) in scale.iter().enumerate() {
        mix(&mut b, &pluck(semis(523.25, *s), 1.1, 0.7, x.seed()), i as f32 * 0.035, 0.6);
    }
    reverb(&mut b, 0.75, 0.3, 0.25, 0.02, 1.0);
    finish(b, -18.0, -2.0)
}

/// The wiggle streak paying off: a riser, a fast bell arpeggio and a
/// lift in the low end.
pub fn power(v: u32) -> Vec<f32> {
    let mut x = Var::new(96, v);
    let mut b = whoosh(400.0, 6000.0, 1.2, 0.45, 0.3, x.seed());
    for s in b.iter_mut() {
        *s *= 0.6;
    }
    for (i, f) in [1046.5, 1318.5, 1568.0, 2093.0, 2637.0].iter().enumerate() {
        mix(&mut b, &bell(*f, 2.0, 1.5, 0.5), 0.05 + i as f32 * 0.05, 0.4);
    }
    mix(&mut b, &body(80.0, 160.0, 0.3, 0.12, 0.35, 1.2), 0.0, 0.4);
    mix(&mut b, &crackle(0.5, 400.0, 7000.0, x.seed(), |t| smooth(t / 0.2)), 0.1, 0.2);
    reverb(&mut b, 0.7, 0.3, 0.25, 0.01, 1.0);
    finish(b, -17.0, -2.0)
}

/// An altitude milestone: a bell arpeggio over a soft brass pad.
pub fn milestone(v: u32) -> Vec<f32> {
    let mut x = Var::new(97, v);
    let mut b = brass(&[523.25, 659.25, 783.99], 1.6, 0.9, 0.2, x.seed());
    for s in b.iter_mut() {
        *s *= 0.4;
    }
    for (i, f) in [1046.5, 1318.5, 1568.0, 2093.0].iter().enumerate() {
        mix(&mut b, &bell(*f, 2.0, 1.4, 0.9), i as f32 * 0.09, 0.35);
    }
    mix(&mut b, &crackle(0.6, 300.0, 8000.0, x.seed(), |t| (1.0 - t / 0.6).max(0.0)), 0.25, 0.12);
    reverb(&mut b, 0.75, 0.3, 0.25, 0.02, 1.3);
    finish(b, -18.0, -2.0)
}

// ---------------------------------------------------------------------------
// Fanfares and jingles
// ---------------------------------------------------------------------------

/// A timpani-ish hit: a tuned drum body and a mallet thud.
fn timp(f: f32, len: f32, seed: u32) -> Vec<f32> {
    let mut b = body(f * 1.1, f, 0.05, len * 0.35, len, 1.5);
    mix(&mut b, &closing_noise(1200.0, 200.0, 0.05, 0.02, 0.1, seed), 0.0, 0.5);
    b
}

/// A new record mid-flight: "ta-ta-ta-TAAA" in brass, bells doubling the
/// top, timpani and a cymbal swell on the long chord.
pub fn record(v: u32) -> Vec<f32> {
    let mut x = Var::new(100, v);
    let g = [392.0, 493.88, 587.33];
    let mut b = Vec::new();
    for i in 0..3 {
        let at = i as f32 * 0.12;
        mix(&mut b, &brass(&g, 0.3, 0.07, 0.6, x.seed()), at, 0.8);
        mix(&mut b, &bell(1567.98, 3.5, 1.5, 0.3), at, 0.12);
        mix(&mut b, &timp(98.0, 0.25, x.seed()), at, 0.4);
    }
    mix(&mut b, &brass(&[523.25, 659.25, 783.99, 1046.5], 1.5, 0.8, 0.95, x.seed()), 0.36, 1.0);
    mix(&mut b, &bell(2093.0, 3.5, 2.0, 1.0), 0.36, 0.25);
    mix(&mut b, &bell(3136.0, 2.0, 1.2, 0.8), 0.4, 0.12);
    mix(&mut b, &timp(65.4, 0.8, x.seed()), 0.36, 0.8);
    let mut rng = Rng::new(x.seed());
    let mut hp = Svf::new(6000.0, 0.7);
    let cym = render(1.4, |_, t| hp.hp(rng.next()) * perc(t, 0.003, 0.5, 1.4));
    mix(&mut b, &cym, 0.36, 0.14);
    reverb(&mut b, 0.8, 0.35, 0.25, 0.02, 1.5);
    finish(b, -15.0, -1.0)
}

/// The results card after a good landing: a skipping marimba-and-bell
/// tune up to a held chord, sleigh bells on the beats.
pub fn results(v: u32) -> Vec<f32> {
    let mut x = Var::new(101, v);
    let tune = [(0.0, 783.99), (0.12, 1046.5), (0.24, 1318.5), (0.36, 1567.98), (0.52, 2093.0)];
    let mut b = Vec::new();
    for (at, f) in tune {
        mix(&mut b, &mallet(f, 0.5, x.seed()), at, 0.6);
        mix(&mut b, &bell(f * 2.0, 3.5, 1.2, 0.4), at, 0.12);
    }
    mix(&mut b, &bell(2093.0, 2.0, 1.4, 1.2), 0.52, 0.3);
    let mut pad = brass(&[523.25, 659.25, 783.99], 1.2, 0.6, 0.2, x.seed());
    for s in pad.iter_mut() {
        *s *= 0.3;
    }
    mix(&mut b, &pad, 0.52, 1.0);
    for at in [0.0, 0.24, 0.52] {
        mix(&mut b, &jingle(0.3, 8, x.seed()), at, 0.35);
    }
    reverb(&mut b, 0.7, 0.35, 0.22, 0.02, 1.2);
    finish(b, -17.0, -2.0)
}

/// A new personal best on the results card: a full brass fanfare
/// (C-C-C, F, G, big C) with timpani, bells, sleigh bells and cymbal.
pub fn results_best(v: u32) -> Vec<f32> {
    let mut x = Var::new(102, v);
    let c = [261.63, 329.63, 392.0, 523.25];
    let f = [349.23, 440.0, 523.25, 698.46];
    let g = [392.0, 493.88, 587.33, 783.99];
    let top = [523.25, 659.25, 783.99, 1046.5, 1318.5];
    let mut b = Vec::new();
    for i in 0..3 {
        let at = i as f32 * 0.11;
        mix(&mut b, &brass(&c, 0.28, 0.06, 0.6, x.seed()), at, 0.7);
        mix(&mut b, &timp(98.0, 0.22, x.seed()), at, 0.35);
    }
    mix(&mut b, &brass(&f, 0.5, 0.24, 0.7, x.seed()), 0.33, 0.8);
    mix(&mut b, &brass(&g, 0.45, 0.2, 0.75, x.seed()), 0.62, 0.8);
    mix(&mut b, &timp(87.3, 0.35, x.seed()), 0.62, 0.5);
    mix(&mut b, &brass(&top, 1.8, 1.0, 1.0, x.seed()), 0.9, 1.0);
    mix(&mut b, &timp(65.4, 0.9, x.seed()), 0.9, 0.8);
    for (i, fq) in [2093.0, 2637.0, 3136.0, 4186.0].iter().enumerate() {
        mix(&mut b, &bell(*fq, 3.5, 1.8, 0.9), 0.9 + i as f32 * 0.06, 0.2);
    }
    for i in 0..8 {
        mix(&mut b, &jingle(0.22, 7, x.seed()), i as f32 * 0.22, 0.25);
    }
    let mut rng = Rng::new(x.seed());
    let mut hp = Svf::new(6000.0, 0.7);
    let cym = render(1.6, |_, t| hp.hp(rng.next()) * perc(t, 0.003, 0.55, 1.6));
    mix(&mut b, &cym, 0.9, 0.16);
    reverb(&mut b, 0.8, 0.35, 0.25, 0.02, 1.6);
    finish(b, -15.0, -1.0)
}

/// The results card after a crash: a muted "wah-wah-wah-waaah" trombone,
/// the last note wobbling, so the crash stays funny.
pub fn results_crash(v: u32) -> Vec<f32> {
    let mut x = Var::new(103, v);
    let notes = [(0.0, 293.66, 0.3), (0.36, 277.18, 0.3), (0.72, 261.63, 0.3), (1.08, 246.94, 1.1)];
    let mut b = Vec::new();
    for (i, (at, f, gate)) in notes.iter().enumerate() {
        let len = gate + 0.25;
        let mut voices: Vec<Osc> = (0..3).map(|k| Osc::with_phase(k as f32 * 0.31)).collect();
        let mut wah = Svf::new(400.0, 3.0);
        let last = i == 3;
        let seed = x.seed();
        let mut breath = Rng::new(seed);
        let note = render(len, |_, t| {
            let vib = if last { 1.0 + 0.03 * (TAU * 5.0 * t).sin() * smooth((t - 0.15) / 0.2) } else { 1.0 };
            let bend = if last { glide(1.0, 0.94, (t - 0.5) / 0.6) } else { 1.0 };
            let ff = f * vib * bend;
            let s: f32 = voices.iter_mut().enumerate().map(|(k, o)| o.saw(ff * (1.0 + (k as f32 - 1.0) * 0.004))).sum();
            let open = adsr(t, 0.08, 0.15, 0.55, 0.15, *gate);
            wah.tune(250.0 + 1100.0 * open, 3.0);
            drive(wah.lp(s * 0.4 + breath.next() * 0.05), 1.5) * adsr(t, 0.03, 0.2, 0.85, 0.2, *gate)
        });
        mix(&mut b, &note, *at, 1.0);
    }
    reverb(&mut b, 0.55, 0.5, 0.15, 0.02, 0.8);
    finish(b, -19.0, -2.0)
}
