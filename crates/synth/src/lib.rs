//! Procedural sound effects for the launch game. Pure DSP: no platform
//! code, so it compiles and tests anywhere; the game turns the samples
//! into WAVs for its audio backend.
//!
//! The toolkit is small but real: enveloped oscillators with soft attacks so
//! nothing clicks, RBJ biquad filters so noise can be shaped into whooshes,
//! thumps and crackle, brown noise for wind, FM bells for chimes, a short
//! feedback delay for air, and a soft clipper on every mix. Each sound is
//! peak-normalized to its own level and faded at both ends.

pub const RATE: u32 = 44100;
const SR: f32 = RATE as f32;
const TAU: f32 = std::f32::consts::TAU;

/// Sound names a pack may override, in `packs/<pack>/sounds/<name>.ogg|wav`.
pub const SOUND_NAMES: [&str; 24] = [
    "flap", "bounce", "present", "milestone", "land", "launch", "crash", "zap", "firework", "pop", "freeze", "jump",
    "trampoline", "reindeer", "updraft", "sleigh", "ufo", "wings", "gust", "wind", "wind_mid", "wind_high",
    "slide", "music",
];

/// Names that are seamless loops rather than one-shots.
pub const LOOP_NAMES: [&str; 4] = ["wind", "wind_mid", "wind_high", "slide"];

/// Length of every wind loop, seconds. Modulations complete whole cycles
/// over it so the cross-faded seam carries no phase jump.
const LOOP_SECS: f32 = 6.0;

/// Number of rungs on the present chime's ladder.
pub const PRESENT_STEPS: usize = 9;

// ---------------------------------------------------------------------------
// Toolkit
// ---------------------------------------------------------------------------

/// Tiny xorshift so the noise is deterministic and needs no crate.
struct Noise(u32);

impl Noise {
    fn new(seed: u32) -> Noise {
        Noise(seed | 1)
    }
    /// White noise in -1..1.
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x & 0xFFFF) as f32 / 32768.0 - 1.0
    }
    /// Uniform 0..1.
    fn unit(&mut self) -> f32 {
        self.next() * 0.5 + 0.5
    }
}

/// RBJ cookbook biquad, recomputed per block for sweeps.
#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn set(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
    fn lowpass(&mut self, f: f32, q: f32) {
        let w = TAU * f.clamp(20.0, SR * 0.45) / SR;
        let (s, c) = (w.sin(), w.cos());
        let alpha = s / (2.0 * q);
        self.set((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, 1.0 + alpha, -2.0 * c, 1.0 - alpha);
    }
    fn bandpass(&mut self, f: f32, q: f32) {
        let w = TAU * f.clamp(20.0, SR * 0.45) / SR;
        let (s, c) = (w.sin(), w.cos());
        let alpha = s / (2.0 * q);
        // Constant peak gain form.
        self.set(alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * c, 1.0 - alpha);
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

fn samples(dur: f32) -> usize {
    (dur * SR) as usize
}

/// Attack / decay envelope: a linear rise over `attack` seconds, then a
/// power-curve decay to zero at the end (`curve` 1 = linear, 3 = snappy).
fn env(i: usize, n: usize, attack: f32, curve: f32) -> f32 {
    let t = i as f32 / SR;
    let dur = n as f32 / SR;
    let a = if attack > 0.0 { (t / attack).min(1.0) } else { 1.0 };
    let d = (1.0 - t / dur).max(0.0).powf(curve);
    a * d
}

/// Exponential glide from `f0` to `f1` across 0..1.
fn glide(f0: f32, f1: f32, k: f32) -> f32 {
    f0 * (f1 / f0).powf(k.clamp(0.0, 1.0))
}

/// A sine (with optional harmonics) gliding from `f0` to `f1`.
fn tone(f0: f32, f1: f32, dur: f32, attack: f32, curve: f32, harmonics: &[f32]) -> Vec<f32> {
    let n = samples(dur);
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            phase += glide(f0, f1, k) / SR;
            let mut s = (phase * TAU).sin();
            for (h, amp) in harmonics.iter().enumerate() {
                s += (phase * TAU * (h as f32 + 2.0)).sin() * amp;
            }
            s * env(i, n, attack, curve)
        })
        .collect()
}

/// An FM bell: carrier `f` with a modulator at `ratio` whose index decays,
/// so the strike is bright and the ring is pure.
fn bell(f: f32, ratio: f32, index: f32, dur: f32) -> Vec<f32> {
    let n = samples(dur);
    (0..n)
        .map(|i| {
            let t = i as f32 / SR;
            let e = env(i, n, 0.004, 2.5);
            let idx = index * (1.0 - t / dur).powi(3);
            let m = (TAU * f * ratio * t).sin() * idx;
            (TAU * f * t + m).sin() * e
        })
        .collect()
}

/// Band-passed white noise whose centre glides `f0` -> `f1`.
fn whoosh(f0: f32, f1: f32, q: f32, dur: f32, attack: f32, curve: f32, seed: u32) -> Vec<f32> {
    let n = samples(dur);
    let mut rng = Noise::new(seed);
    let mut bp = Biquad::default();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        if i % 32 == 0 {
            bp.bandpass(glide(f0, f1, i as f32 / n as f32), q);
        }
        out.push(bp.run(rng.next()) * env(i, n, attack, curve));
    }
    out
}

/// Low-passed noise burst: a thump or a muffled pop.
fn thump(cutoff: f32, dur: f32, curve: f32, seed: u32) -> Vec<f32> {
    let n = samples(dur);
    let mut rng = Noise::new(seed);
    let mut lp = Biquad::default();
    lp.lowpass(cutoff, 0.8);
    (0..n).map(|i| lp.run(rng.next()) * env(i, n, 0.003, curve)).collect()
}

/// Sparse impulses through a filter: snow crunch or firework crackle.
fn crackle(density: f32, cutoff: f32, bright: bool, dur: f32, curve: f32, seed: u32) -> Vec<f32> {
    let n = samples(dur);
    let mut rng = Noise::new(seed);
    let mut f = Biquad::default();
    if bright {
        // A band-pass rather than a true high-pass: an impulse through a
        // high-pass is a full-height step, which reads as a click.
        f.bandpass(cutoff, 0.8);
    } else {
        f.lowpass(cutoff, 0.7);
    }
    (0..n)
        .map(|i| {
            let e = env(i, n, 0.003, curve);
            let hit = if rng.unit() < density * e.max(0.15) { rng.next() } else { 0.0 };
            f.run(hit) * e
        })
        .collect()
}

/// Brown noise: integrated white with a leak, then low-passed.
fn brown(cutoff: f32, dur: f32, seed: u32) -> Vec<f32> {
    let n = samples(dur);
    let mut rng = Noise::new(seed);
    let mut lp = Biquad::default();
    lp.lowpass(cutoff, 0.7);
    let mut y = 0.0f32;
    (0..n)
        .map(|_| {
            y = (y + rng.next() * 0.03) * 0.995;
            lp.run(y) * 4.0
        })
        .collect()
}

/// Mix `src` into `dst` starting `at` seconds in, scaled by `gain`.
fn mix(dst: &mut Vec<f32>, src: &[f32], at: f32, gain: f32) {
    let off = samples(at);
    if dst.len() < off + src.len() {
        dst.resize(off + src.len(), 0.0);
    }
    for (i, s) in src.iter().enumerate() {
        dst[off + i] += s * gain;
    }
}

/// A short feedback delay for a little air on chimes.
fn air(buf: &mut Vec<f32>, delay: f32, feedback: f32, wet: f32) {
    let d = samples(delay);
    let tail = samples(delay * 6.0);
    buf.resize(buf.len() + tail, 0.0);
    for i in d..buf.len() {
        let echo = buf[i - d] * feedback;
        buf[i] += echo * wet;
    }
}

/// Soft clip, gentle low-pass, peak-normalize to `peak`, fade the edges.
fn finish(mut buf: Vec<f32>, peak: f32) -> Vec<f32> {
    let mut lp = Biquad::default();
    lp.lowpass(7000.0, 0.6);
    for s in buf.iter_mut() {
        let x = *s;
        // Cubic soft clip, transparent below about 0.5.
        let c = if x.abs() < 1.5 { x - x * x * x / 6.75 } else { x.signum() };
        *s = lp.run(c);
    }
    let max = buf.iter().fold(0.0f32, |m, s| m.max(s.abs())).max(1e-6);
    let g = peak / max;
    let fade = samples(0.006).max(1);
    let n = buf.len();
    for (i, s) in buf.iter_mut().enumerate() {
        let k_in = (i as f32 / fade as f32).min(1.0);
        let k_out = ((n - 1 - i) as f32 / fade as f32).min(1.0);
        *s *= g * k_in * k_out;
    }
    if let Some(last) = buf.last_mut() {
        *last = 0.0;
    }
    buf
}

/// A 16-bit mono WAV with a 44-byte header.
pub fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------------------
// Ogg Vorbis duration
// ---------------------------------------------------------------------------

/// Length in seconds of an Ogg Vorbis file, from the identification header's
/// sample rate and the granule position of the last page. `None` if the
/// bytes are not Ogg Vorbis. Enough to schedule a playlist; no decoding.
pub fn ogg_duration(bytes: &[u8]) -> Option<f32> {
    let page = |at: usize| bytes.get(at..at + 4) == Some(b"OggS");
    if !page(0) || bytes.len() < 27 {
        return None;
    }
    // First page: header (27 bytes) + segment table, then the first packet,
    // which for Vorbis is 0x01 "vorbis" version(4) channels(1) rate(4).
    let segs = bytes[26] as usize;
    let packet = 27 + segs;
    if bytes.get(packet..packet + 7) != Some(b"\x01vorbis") {
        return None;
    }
    let r = packet + 12;
    let rate = u32::from_le_bytes(bytes.get(r..r + 4)?.try_into().ok()?);
    if rate == 0 {
        return None;
    }
    // Last page: scan back for the capture pattern; granule is bytes 6..14.
    let mut at = bytes.len().checked_sub(27)?;
    loop {
        if page(at) {
            let g = u64::from_le_bytes(bytes[at + 6..at + 14].try_into().ok()?);
            if g != u64::MAX {
                return Some(g as f32 / rate as f32);
            }
        }
        if at == 0 {
            return None;
        }
        at -= 1;
    }
}

// ---------------------------------------------------------------------------
// The sounds
// ---------------------------------------------------------------------------

/// Semitone steps of a major pentatonic ladder the present chime climbs.
const LADDER: [f32; PRESENT_STEPS] = [0.0, 2.0, 4.0, 7.0, 9.0, 12.0, 14.0, 16.0, 19.0];

fn semis(f: f32, n: f32) -> f32 {
    f * 2f32.powf(n / 12.0)
}

/// A soft "fwup": a band-passed puff of air with a quiet thump under it.
fn synth_flap() -> Vec<f32> {
    let mut b = whoosh(900.0, 300.0, 1.2, 0.11, 0.006, 1.6, 11);
    mix(&mut b, &tone(120.0, 70.0, 0.06, 0.003, 2.0, &[]), 0.0, 0.35);
    finish(b, 0.35)
}

/// Landing on snow: a low pitch-dropping body and a muffled thud.
fn synth_bounce() -> Vec<f32> {
    let mut b = tone(95.0, 40.0, 0.22, 0.003, 2.2, &[0.25]);
    mix(&mut b, &thump(420.0, 0.07, 2.0, 21), 0.0, 0.8);
    finish(b, 0.5)
}

/// One rung of the present ladder: an FM bell with a little air.
fn synth_present(step: usize) -> Vec<f32> {
    let f = semis(1046.5, LADDER[step.min(LADDER.len() - 1)]);
    let mut b = bell(f, 2.0, 1.1, 0.3);
    mix(&mut b, &bell(f * 0.5, 2.0, 0.6, 0.18), 0.0, 0.3);
    air(&mut b, 0.06, 0.35, 0.5);
    finish(b, 0.35)
}

/// Three rising bells for an altitude milestone.
fn synth_milestone() -> Vec<f32> {
    let mut b = Vec::new();
    for (i, f) in [523.25, 659.25, 783.99].iter().enumerate() {
        mix(&mut b, &bell(*f, 2.0, 1.2, 0.6), i as f32 * 0.14, 1.0);
        mix(&mut b, &bell(*f * 2.0, 2.0, 0.8, 0.3), i as f32 * 0.14, 0.25);
    }
    air(&mut b, 0.09, 0.4, 0.5);
    finish(b, 0.4)
}

/// Leaving the lip: a long rising whoosh over a low boom.
fn synth_launch() -> Vec<f32> {
    let mut b = whoosh(150.0, 2500.0, 0.9, 0.55, 0.15, 1.4, 31);
    mix(&mut b, &tone(70.0, 35.0, 0.3, 0.004, 2.0, &[0.3]), 0.0, 0.9);
    mix(&mut b, &thump(300.0, 0.08, 2.0, 32), 0.0, 0.7);
    finish(b, 0.65)
}

/// Coming to rest: a soft thud and a snow crunch.
fn synth_land() -> Vec<f32> {
    let mut b = tone(80.0, 40.0, 0.15, 0.003, 2.0, &[0.2]);
    mix(&mut b, &crackle(0.35, 1500.0, false, 0.25, 1.4, 41), 0.02, 0.9);
    finish(b, 0.45)
}

/// Head-first into the snow: a big thud, a long crunch and a slide.
fn synth_crash() -> Vec<f32> {
    let mut b = tone(60.0, 28.0, 0.3, 0.003, 1.8, &[0.35, 0.15]);
    mix(&mut b, &thump(250.0, 0.12, 1.8, 51), 0.0, 1.0);
    mix(&mut b, &crackle(0.5, 1800.0, false, 0.4, 1.3, 52), 0.03, 0.9);
    mix(&mut b, &whoosh(800.0, 200.0, 1.5, 0.2, 0.02, 1.5, 53), 0.1, 0.5);
    finish(b, 0.75)
}

/// A storm cloud strike: a buzzing, band-passed, tremolo'd zap.
fn synth_zap() -> Vec<f32> {
    let n = samples(0.35);
    let mut rng = Noise::new(61);
    let mut bp = Biquad::default();
    bp.bandpass(900.0, 0.7);
    let b: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / SR;
            let sq = ((TAU * 60.0 * t).sin() * 5.0).tanh();
            let trem = 0.6 + 0.4 * (TAU * 30.0 * t).sin();
            bp.run(sq + rng.next() * 0.5) * trem * env(i, n, 0.004, 1.5)
        })
        .collect();
    finish(b, 0.45)
}

/// Grabbing a firework: an ignition hiss and a rising whistle with vibrato.
fn synth_firework() -> Vec<f32> {
    let mut b = whoosh(3000.0, 2200.0, 1.0, 0.12, 0.005, 1.5, 71);
    let n = samples(0.4);
    let mut phase = 0.0f32;
    let whistle: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let t = i as f32 / SR;
            phase += (glide(400.0, 1800.0, k) + 18.0 * (TAU * 9.0 * t).sin()) / SR;
            (phase * TAU).sin() * env(i, n, 0.03, 1.2)
        })
        .collect();
    mix(&mut b, &whistle, 0.05, 0.9);
    finish(b, 0.4)
}

/// A firework bursting: a muffled pop and a bright crackle tail.
fn synth_pop() -> Vec<f32> {
    let mut b = thump(300.0, 0.05, 1.5, 81);
    mix(&mut b, &crackle(0.25, 2500.0, true, 0.5, 1.6, 82), 0.02, 0.35);
    finish(b, 0.5)
}

/// Freezing solid: a glassy descending FM tone with shimmer.
fn synth_freeze() -> Vec<f32> {
    let n = samples(0.6);
    let mut phase = 0.0f32;
    let b: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let t = i as f32 / SR;
            let f = glide(1400.0, 500.0, k) * (1.0 + 0.01 * (TAU * 6.0 * t).sin());
            phase += f / SR;
            let m = (phase * TAU * 2.0).sin() * 1.5 * (1.0 - k);
            (phase * TAU + m).sin() * env(i, n, 0.01, 1.5)
        })
        .collect();
    let mut out = b;
    mix(&mut out, &bell(2800.0, 3.0, 0.8, 0.25), 0.0, 0.25);
    finish(out, 0.4)
}

/// A perfect press at the lip: a bright ding and a short whoosh.
fn synth_jump() -> Vec<f32> {
    let mut b = bell(1568.0, 2.0, 1.2, 0.4);
    mix(&mut b, &bell(2093.0, 2.0, 0.8, 0.3), 0.04, 0.5);
    mix(&mut b, &whoosh(600.0, 1800.0, 1.0, 0.18, 0.02, 1.5, 91), 0.0, 0.45);
    air(&mut b, 0.07, 0.35, 0.4);
    finish(b, 0.4)
}

/// Trampoline boing: a bent tone with harmonics.
fn synth_trampoline() -> Vec<f32> {
    let n = samples(0.25);
    let mut phase = 0.0f32;
    let b: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let f = if k < 0.35 { glide(180.0, 420.0, k / 0.35) } else { glide(420.0, 300.0, (k - 0.35) / 0.65) };
            phase += f / SR;
            let s = (phase * TAU).sin() + (phase * TAU * 2.0).sin() * 0.4 + (phase * TAU * 3.0).sin() * 0.2;
            s * env(i, n, 0.005, 1.6)
        })
        .collect();
    finish(b, 0.45)
}

/// Reindeer: a handful of sleigh bells scattered over a third of a second.
fn synth_reindeer() -> Vec<f32> {
    let mut rng = Noise::new(101);
    let mut b = Vec::new();
    for _ in 0..6 {
        let f = 2200.0 + rng.unit() * 1000.0;
        mix(&mut b, &bell(f, 1.5, 0.7, 0.12), rng.unit() * 0.28, 0.6 + rng.unit() * 0.4);
    }
    finish(b, 0.3)
}

/// Updraft: a soft, wide whoosh.
fn synth_updraft() -> Vec<f32> {
    finish(whoosh(300.0, 1200.0, 0.8, 0.4, 0.12, 1.3, 111), 0.3)
}

/// Santa's sleigh: a warm bell chord under a jingle.
fn synth_sleigh() -> Vec<f32> {
    let mut b = Vec::new();
    for (i, f) in [261.63, 329.63, 392.0, 523.25].iter().enumerate() {
        mix(&mut b, &bell(*f, 2.0, 1.0, 0.8), i as f32 * 0.03, 0.8);
    }
    let mut rng = Noise::new(121);
    for _ in 0..5 {
        mix(&mut b, &bell(2400.0 + rng.unit() * 800.0, 1.5, 0.7, 0.12), 0.1 + rng.unit() * 0.4, 0.25);
    }
    air(&mut b, 0.1, 0.4, 0.5);
    finish(b, 0.45)
}

/// UFO: a descending sci-fi sweep with vibrato.
fn synth_ufo() -> Vec<f32> {
    let n = samples(0.5);
    let mut phase = 0.0f32;
    let b: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let t = i as f32 / SR;
            phase += (glide(1200.0, 300.0, k) + 30.0 * (TAU * 8.0 * t).sin()) / SR;
            let s = (phase * TAU).sin() + (phase * TAU * 2.0).sin() * 0.3 + (phase * TAU * 3.0).sin() * 0.15;
            s * env(i, n, 0.01, 1.3)
        })
        .collect();
    finish(b, 0.4)
}

/// Angel wings: a quick harp-like run of three bells.
fn synth_wings() -> Vec<f32> {
    let mut b = Vec::new();
    for (i, f) in [1046.5, 1318.5, 1568.0].iter().enumerate() {
        mix(&mut b, &bell(*f, 2.0, 0.9, 0.4), i as f32 * 0.06, 1.0);
    }
    air(&mut b, 0.08, 0.35, 0.5);
    finish(b, 0.35)
}

/// Cross-fade the tail of a loop into its head and normalize; loops get no
/// edge fades, so the seam is the only place a click could hide.
fn seal_loop(mut b: Vec<f32>, peak: f32) -> Vec<f32> {
    let n = b.len();
    let fade = samples(0.25);
    for i in 0..fade {
        let k = i as f32 / fade as f32;
        let tail = b[n - fade + i];
        b[n - fade + i] = tail * (1.0 - k) + b[i] * k;
    }
    let max = b.iter().fold(0.0f32, |m, s| m.max(s.abs())).max(1e-6);
    for s in b.iter_mut() {
        *s *= peak / max;
    }
    b
}

/// Cycles-per-loop to Hz, so a modulation lands where it started.
fn per_loop(cycles: f32) -> f32 {
    cycles / LOOP_SECS
}

/// The low wind bed: brown rumble under 300 Hz with a slow swell.
fn synth_wind_low() -> Vec<f32> {
    let mut b = brown(300.0, LOOP_SECS, 131);
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *s *= 0.7 + 0.2 * (TAU * per_loop(1.0) * t).sin() + 0.1 * (TAU * per_loop(2.0) * t + 1.0).sin();
    }
    seal_loop(b, 0.9)
}

/// The mid wind: a broad band that breathes between 500 and 900 Hz with
/// gusty amplitude wobble.
/// Sled runners on packed snow: a hissing band of noise with a grainy
/// crunch riding on it, looped; the runtime raises it as the sled speeds up.
fn synth_slide() -> Vec<f32> {
    let n = samples(LOOP_SECS);
    let mut rng = Noise::new(171);
    let mut hiss = Biquad::default();
    let mut crunch = Biquad::default();
    hiss.bandpass(600.0, 0.7);
    crunch.lowpass(1800.0, 0.8);
    let mut b = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR;
        if i % 64 == 0 {
            hiss.bandpass(520.0 + 160.0 * (TAU * per_loop(2.0) * t).sin(), 0.7);
        }
        let h = hiss.run(rng.next()) * 0.7;
        // Sparse grains: a click every few ms, low-passed into a crunch.
        let grain = if rng.next() > 0.985 { rng.next() * 2.5 } else { 0.0 };
        let c = crunch.run(grain) * 0.8;
        let wobble = 0.85 + 0.15 * (TAU * per_loop(5.0) * t).sin();
        b.push((h + c) * wobble);
    }
    seal_loop(b, 0.9)
}

fn synth_wind_mid() -> Vec<f32> {
    let n = samples(LOOP_SECS);
    let mut rng = Noise::new(141);
    let mut bp = Biquad::default();
    let mut b = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR;
        if i % 32 == 0 {
            let centre = 700.0 + 200.0 * (TAU * per_loop(1.0) * t).sin();
            bp.bandpass(centre, 0.8);
        }
        let gust = 0.6 + 0.25 * (TAU * per_loop(3.0) * t).sin() + 0.15 * (TAU * per_loop(4.0) * t + 2.0).sin();
        b.push(bp.run(rng.next()) * gust);
    }
    seal_loop(b, 0.9)
}

/// The high wind: a quiet whistle, a resonant band gliding between 1.2 and
/// 2.4 kHz with a little vibrato.
fn synth_wind_high() -> Vec<f32> {
    let n = samples(LOOP_SECS);
    let mut rng = Noise::new(151);
    let mut bp = Biquad::default();
    let mut b = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR;
        if i % 16 == 0 {
            let glide_k = 0.5 + 0.5 * (TAU * per_loop(1.0) * t).sin();
            let centre = glide(1200.0, 2400.0, glide_k) * (1.0 + 0.02 * (TAU * 5.0 * t).sin());
            bp.bandpass(centre, 6.0);
        }
        let swell = 0.7 + 0.3 * (TAU * per_loop(2.0) * t + 0.7).sin();
        b.push(bp.run(rng.next()) * swell);
    }
    seal_loop(b, 0.9)
}

/// A gust: one short band-passed whoosh for a flap or a boost.
fn synth_gust() -> Vec<f32> {
    finish(whoosh(400.0, 1400.0, 1.1, 0.3, 0.05, 1.5, 161), 0.3)
}

/// Synthesize one sound by name; the present chime takes its ladder step.
pub fn synth(name: &str, step: usize) -> Vec<f32> {
    match name {
        "flap" => synth_flap(),
        "bounce" => synth_bounce(),
        "present" => synth_present(step),
        "milestone" => synth_milestone(),
        "land" => synth_land(),
        "launch" => synth_launch(),
        "crash" => synth_crash(),
        "zap" => synth_zap(),
        "firework" => synth_firework(),
        "pop" => synth_pop(),
        "freeze" => synth_freeze(),
        "jump" => synth_jump(),
        "trampoline" => synth_trampoline(),
        "reindeer" => synth_reindeer(),
        "updraft" => synth_updraft(),
        "sleigh" => synth_sleigh(),
        "ufo" => synth_ufo(),
        "wings" => synth_wings(),
        "gust" => synth_gust(),
        "wind" => synth_wind_low(),
        "wind_mid" => synth_wind_mid(),
        "wind_high" => synth_wind_high(),
        "slide" => synth_slide(),
        // Music is never synthesized: packs ship it as `music_N.ogg`.
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every synthesized sound: no hard peaks, no clicks, silent edges.
    /// Also writes each one as a WAV into `SFX_OUT` (if set) for listening.
    #[test]
    fn sounds_are_clean() {
        let out = std::env::var("SFX_OUT").ok();
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        for name in SOUND_NAMES {
            if name == "music" {
                continue;
            }
            let steps = if name == "present" { LADDER.len() } else { 1 };
            for step in 0..steps {
                let s = synth(name, step);
                assert!(s.len() > samples(0.04), "{name} is too short");
                let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
                assert!(peak <= 0.95, "{name} peaks at {peak}");
                let jump = s.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
                assert!(jump <= 0.35, "{name} clicks: sample jump {jump}");
                if !LOOP_NAMES.contains(&name) {
                    // Silent edges: the first millisecond is the fade-in, the
                    // last five are the fade-out.
                    let head = s[..samples(0.001)].iter().fold(0.0f32, |m, v| m.max(v.abs()));
                    let tail = s[s.len() - samples(0.005)..].iter().fold(0.0f32, |m, v| m.max(v.abs()));
                    assert!(head < 0.12 && tail < 0.05, "{name} edges: head {head} tail {tail}");
                }
                if let Some(dir) = &out {
                    let file = if steps > 1 { format!("{dir}/{name}_{step}.wav") } else { format!("{dir}/{name}.wav") };
                    std::fs::write(file, wav(&s)).unwrap();
                }
            }
        }
    }

    /// Loops: the seam (last sample to first) is as smooth as the body.
    #[test]
    fn loops_are_seamless() {
        for name in LOOP_NAMES {
            let s = synth(name, 0);
            assert_eq!(s.len(), samples(LOOP_SECS), "{name} is not one loop long");
            let seam = (s[0] - s[s.len() - 1]).abs();
            assert!(seam <= 0.35, "{name} seam jumps {seam}");
        }
    }

    /// A hand-built two-page Ogg Vorbis skeleton reports its length.
    #[test]
    fn ogg_duration_from_last_granule() {
        let mut f = Vec::new();
        // Page 1: header, one segment of 30 bytes, then the id packet.
        f.extend_from_slice(b"OggS");
        f.extend_from_slice(&[0u8; 22]);
        f.push(1);
        f.push(30);
        let mut id = b"\x01vorbis".to_vec();
        id.extend_from_slice(&0u32.to_le_bytes());
        id.push(2);
        id.extend_from_slice(&44100u32.to_le_bytes());
        id.resize(30, 0);
        f.extend_from_slice(&id);
        // Page 2: granule 88200 samples = 2 s.
        f.extend_from_slice(b"OggS");
        f.extend_from_slice(&[0u8; 2]);
        f.extend_from_slice(&88200u64.to_le_bytes());
        f.extend_from_slice(&[0u8; 13]);
        assert_eq!(ogg_duration(&f), Some(2.0));
        assert_eq!(ogg_duration(b"RIFF...."), None);
    }

    #[test]
    fn present_ladder_rises() {
        // Higher rungs have shorter zero-crossing intervals on the attack.
        let period = |s: &[f32]| {
            let a = &s[samples(0.02)..samples(0.06)];
            a.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count()
        };
        let lo = period(&synth("present", 0));
        let hi = period(&synth("present", 5));
        assert!(hi > lo, "rung 5 ({hi} crossings) should ring higher than rung 0 ({lo})");
    }
}
