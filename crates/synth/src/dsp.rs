//! The DSP toolkit every sound is built from: noise colours, band-limited
//! oscillators, a zero-delay state-variable filter that stays stable under
//! per-sample sweeps, ADSR and exponential envelopes, saturation, a
//! Freeverb-style room, a damped echo, Karplus-Strong plucks, and the
//! finishing chain (DC block, soft limit, loudness match, click-free edges).

pub const SR: f32 = crate::RATE as f32;
pub const TAU: f32 = std::f32::consts::TAU;
const PI: f32 = std::f32::consts::PI;

pub fn samples(dur: f32) -> usize {
    (dur.max(0.0) * SR) as usize
}

pub fn semis(f: f32, n: f32) -> f32 {
    f * 2f32.powf(n / 12.0)
}

/// Exponential glide from `a` to `b` across 0..1.
pub fn glide(a: f32, b: f32, k: f32) -> f32 {
    a * (b / a).powf(k.clamp(0.0, 1.0))
}

pub fn lerp(a: f32, b: f32, k: f32) -> f32 {
    a + (b - a) * k
}

// ---------------------------------------------------------------------------
// Randomness and noise
// ---------------------------------------------------------------------------

/// Xorshift32: deterministic, so every render of a variant is identical.
#[derive(Clone)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Rng {
        let mut r = Rng(seed.wrapping_mul(2654435761).wrapping_add(0x9E37_79B9) | 1);
        for _ in 0..4 {
            r.next();
        }
        r
    }
    /// White noise in -1..1.
    pub fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / 8_388_608.0 - 1.0
    }
    /// Uniform 0..1.
    pub fn unit(&mut self) -> f32 {
        self.next() * 0.5 + 0.5
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }
}

/// Pink noise (Paul Kellet's economy filter): -3 dB/octave, the colour of
/// wind and surf.
pub struct Pink {
    rng: Rng,
    b: [f32; 3],
}

impl Pink {
    pub fn new(seed: u32) -> Pink {
        Pink { rng: Rng::new(seed), b: [0.0; 3] }
    }
    pub fn next(&mut self) -> f32 {
        let w = self.rng.next();
        self.b[0] = 0.99765 * self.b[0] + w * 0.099_046;
        self.b[1] = 0.963 * self.b[1] + w * 0.296_516_4;
        self.b[2] = 0.57 * self.b[2] + w * 1.052_691_3;
        (self.b[0] + self.b[1] + self.b[2] + w * 0.1848) * 0.22
    }
}

/// Brown noise: leaky-integrated white, -6 dB/octave rumble.
pub struct Brown {
    rng: Rng,
    y: f32,
}

impl Brown {
    pub fn new(seed: u32) -> Brown {
        Brown { rng: Rng::new(seed), y: 0.0 }
    }
    pub fn next(&mut self) -> f32 {
        self.y = (self.y + self.rng.next() * 0.04) * 0.996;
        self.y * 3.5
    }
}

/// A smooth random curve: value noise with cosine interpolation, `rate`
/// new targets per second. Output -1..1. For drifting filters and gusts.
pub struct Drift {
    rng: Rng,
    a: f32,
    b: f32,
    k: f32,
    step: f32,
}

impl Drift {
    pub fn new(seed: u32, rate: f32) -> Drift {
        let mut rng = Rng::new(seed);
        let a = rng.next();
        let b = rng.next();
        Drift { rng, a, b, k: 0.0, step: rate / SR }
    }
    pub fn next(&mut self) -> f32 {
        self.k += self.step;
        if self.k >= 1.0 {
            self.k -= 1.0;
            self.a = self.b;
            self.b = self.rng.next();
        }
        let s = 0.5 - 0.5 * (self.k * PI).cos();
        lerp(self.a, self.b, s)
    }
}

// ---------------------------------------------------------------------------
// Oscillators
// ---------------------------------------------------------------------------

fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

/// A phase accumulator with band-limited (polyBLEP) saw and square, so
/// the bright waves don't alias into a cheap digital buzz.
#[derive(Clone, Copy, Default)]
pub struct Osc {
    pub phase: f32,
}

impl Osc {
    pub fn with_phase(p: f32) -> Osc {
        Osc { phase: p.rem_euclid(1.0) }
    }
    fn adv(&mut self, f: f32) -> (f32, f32) {
        let dt = (f / SR).clamp(0.0, 0.5);
        let t = self.phase;
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        (t, dt)
    }
    pub fn sine(&mut self, f: f32) -> f32 {
        let (t, _) = self.adv(f);
        (t * TAU).sin()
    }
    pub fn saw(&mut self, f: f32) -> f32 {
        let (t, dt) = self.adv(f);
        2.0 * t - 1.0 - poly_blep(t, dt)
    }
    pub fn square(&mut self, f: f32) -> f32 {
        let (t, dt) = self.adv(f);
        let s = if t < 0.5 { 1.0 } else { -1.0 };
        s + poly_blep(t, dt) - poly_blep((t + 0.5) % 1.0, dt)
    }
    pub fn tri(&mut self, f: f32) -> f32 {
        let (t, _) = self.adv(f);
        4.0 * (t - 0.5).abs() - 1.0
    }
}

/// A two-operator FM voice: sine carrier at `f`, sine modulator at
/// `f * ratio`, modulation depth `index` (radians) supplied per sample.
#[derive(Clone, Copy, Default)]
pub struct Fm {
    c: Osc,
    m: Osc,
}

impl Fm {
    pub fn new() -> Fm {
        Fm::default()
    }
    pub fn next(&mut self, f: f32, ratio: f32, index: f32) -> f32 {
        let m = self.m.sine(f * ratio) * index;
        let (t, _) = self.c.adv(f);
        (t * TAU + m).sin()
    }
}

// ---------------------------------------------------------------------------
// Filters
// ---------------------------------------------------------------------------

/// Topology-preserving state-variable filter (Zavalishin). Low, band and
/// high outputs at once; safe to retune every sample for sweeps.
#[derive(Clone, Copy, Default)]
pub struct Svf {
    ic1: f32,
    ic2: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
    f: f32,
    q: f32,
}

pub struct SvfOut {
    pub lp: f32,
    pub bp: f32,
    pub hp: f32,
}

impl Svf {
    pub fn new(f: f32, q: f32) -> Svf {
        let mut s = Svf::default();
        s.tune(f, q);
        s
    }
    pub fn tune(&mut self, f: f32, q: f32) {
        if (f - self.f).abs() < 0.05 && q == self.q {
            return;
        }
        self.f = f;
        self.q = q;
        let g = (PI * f.clamp(15.0, SR * 0.46) / SR).tan();
        self.k = 1.0 / q.max(0.05);
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }
    pub fn run(&mut self, x: f32) -> SvfOut {
        let v3 = x - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        SvfOut { lp: v2, bp: v1, hp: x - self.k * v1 - v2 }
    }
    pub fn lp(&mut self, x: f32) -> f32 {
        self.run(x).lp
    }
    /// Band-pass normalized to unity gain at the centre.
    pub fn bp(&mut self, x: f32) -> f32 {
        let k = self.k;
        self.run(x).bp * k
    }
    pub fn hp(&mut self, x: f32) -> f32 {
        self.run(x).hp
    }
}

/// One-pole low-pass, for smoothing and gentle tone.
#[derive(Clone, Copy, Default)]
pub struct OnePole {
    y: f32,
    a: f32,
}

impl OnePole {
    pub fn new(f: f32) -> OnePole {
        OnePole { y: 0.0, a: 1.0 - (-TAU * f / SR).exp() }
    }
    pub fn run(&mut self, x: f32) -> f32 {
        self.y += (x - self.y) * self.a;
        self.y
    }
}

// ---------------------------------------------------------------------------
// Envelopes and shaping
// ---------------------------------------------------------------------------

/// ADSR: smooth (raised-sine) attack, exponential-shaped decay to
/// `sustain`, held until `gate`, then an exponential-shaped release. All
/// times in seconds; zero at t = 0 and after `gate + release`.
pub fn adsr(t: f32, a: f32, d: f32, s: f32, r: f32, gate: f32) -> f32 {
    let level = |t: f32| -> f32 {
        if t < a {
            let k = t / a.max(1e-5);
            0.5 - 0.5 * (k * PI).cos()
        } else {
            let k = ((t - a) / d.max(1e-5)).min(8.0);
            s + (1.0 - s) * (-k * 4.0).exp()
        }
    };
    if t < 0.0 {
        0.0
    } else if t < gate {
        level(t)
    } else {
        let k = (t - gate) / r.max(1e-5);
        if k >= 1.0 {
            0.0
        } else {
            // Exponential feel, but reaching true zero at the end.
            level(gate) * ((-k * 5.0).exp() - (-5.0f32).exp()) / (1.0 - (-5.0f32).exp())
        }
    }
}

/// A percussive envelope: smooth attack, exponential decay with time
/// constant `tau`, forced to zero by `len`.
pub fn perc(t: f32, attack: f32, tau: f32, len: f32) -> f32 {
    if t < 0.0 || t >= len {
        return 0.0;
    }
    let a = if t < attack { 0.5 - 0.5 * (t / attack * PI).cos() } else { 1.0 };
    let tail = 1.0 - smooth(((t - len * 0.85) / (len * 0.15)).clamp(0.0, 1.0));
    a * (-(t - attack).max(0.0) / tau).exp() * tail
}

/// Smoothstep 0..1.
pub fn smooth(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// Tanh saturation normalized so full scale stays full scale.
pub fn drive(x: f32, amount: f32) -> f32 {
    (x * amount).tanh() / amount.tanh()
}

// ---------------------------------------------------------------------------
// Buffers
// ---------------------------------------------------------------------------

/// Render `dur` seconds by calling `f(i, t)` per sample.
pub fn render(dur: f32, mut f: impl FnMut(usize, f32) -> f32) -> Vec<f32> {
    (0..samples(dur)).map(|i| f(i, i as f32 / SR)).collect()
}

/// Mix `src` into `dst` starting `at` seconds in, scaled by `gain`.
pub fn mix(dst: &mut Vec<f32>, src: &[f32], at: f32, gain: f32) {
    let off = samples(at);
    if dst.len() < off + src.len() {
        dst.resize(off + src.len(), 0.0);
    }
    for (i, s) in src.iter().enumerate() {
        dst[off + i] += s * gain;
    }
}

pub fn peak(b: &[f32]) -> f32 {
    b.iter().fold(0.0f32, |m, s| m.max(s.abs()))
}

/// Loudest RMS over any 50 ms window (hop 10 ms), measured through a
/// 100 Hz high-pass: sub-bass that small speakers can't play doesn't
/// count toward loudness, so a boomy sound isn't matched too quiet.
pub fn max_rms(b: &[f32]) -> f32 {
    let mut hp = Svf::new(100.0, 0.7);
    let w: Vec<f32> = b.iter().map(|s| hp.hp(*s)).collect();
    max_rms_raw(&w)
}

/// Loudest unweighted RMS over any 50 ms window (hop 10 ms).
pub fn max_rms_raw(b: &[f32]) -> f32 {
    let w = samples(0.05).max(1);
    let hop = samples(0.01).max(1);
    if b.len() <= w {
        return (b.iter().map(|s| s * s).sum::<f32>() / b.len().max(1) as f32).sqrt();
    }
    let mut best = 0.0f32;
    let mut i = 0;
    while i + w <= b.len() {
        let e = b[i..i + w].iter().map(|s| s * s).sum::<f32>() / w as f32;
        best = best.max(e);
        i += hop;
    }
    best.sqrt()
}

pub fn db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

pub fn undb(d: f32) -> f32 {
    10f32.powf(d / 20.0)
}

// ---------------------------------------------------------------------------
// Space: echo and reverb
// ---------------------------------------------------------------------------

/// A damped feedback echo (the repeats lose top end like real air).
pub fn echo(buf: &mut Vec<f32>, time: f32, feedback: f32, wet: f32, tone: f32) {
    let d = samples(time).max(1);
    let fb = feedback.clamp(0.0, 0.95);
    // Enough tail for the echo to fall 60 dB.
    let repeats = if fb > 0.0 { (-3.0 / fb.log10()).ceil() as usize } else { 1 };
    buf.resize(buf.len() + d * repeats.min(12), 0.0);
    let mut line = vec![0.0f32; d];
    let mut lp = OnePole::new(tone);
    let mut idx = 0;
    for s in buf.iter_mut() {
        let delayed = line[idx];
        line[idx] = lp.run(*s + delayed * fb);
        *s += delayed * wet;
        idx = (idx + 1) % d;
    }
}

struct Comb {
    buf: Vec<f32>,
    idx: usize,
    store: f32,
}

struct Allpass {
    buf: Vec<f32>,
    idx: usize,
}

/// Freeverb-style mono reverb: eight damped combs into four allpasses.
/// `size` 0..1 is the room (decay), `damp` 0..1 darkens the tail, `wet` is
/// the reverb level against the dry. `pre` delays the reverb (seconds) so
/// a transient stays crisp. The buffer grows by `tail` seconds.
pub fn reverb(buf: &mut Vec<f32>, size: f32, damp: f32, wet: f32, pre: f32, tail: f32) {
    const COMBS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
    const ALLS: [usize; 4] = [556, 441, 341, 225];
    let fb = 0.7 + 0.28 * size.clamp(0.0, 1.0);
    let damp = damp.clamp(0.0, 1.0) * 0.4;
    let mut combs: Vec<Comb> = COMBS.iter().map(|&n| Comb { buf: vec![0.0; n], idx: 0, store: 0.0 }).collect();
    let mut alls: Vec<Allpass> = ALLS.iter().map(|&n| Allpass { buf: vec![0.0; n], idx: 0 }).collect();
    let pre_n = samples(pre).max(1);
    let mut pre_line = vec![0.0f32; pre_n];
    let mut pre_idx = 0;
    // Keep mud out of the tail: high-pass the send at 180 Hz.
    let mut send_hp = Svf::new(180.0, 0.7);
    buf.resize(buf.len() + samples(tail), 0.0);
    for s in buf.iter_mut() {
        let x = send_hp.hp(pre_line[pre_idx]) * 0.03;
        pre_line[pre_idx] = *s;
        pre_idx = (pre_idx + 1) % pre_n;
        let mut out = 0.0;
        for c in combs.iter_mut() {
            let y = c.buf[c.idx];
            c.store = y * (1.0 - damp) + c.store * damp;
            c.buf[c.idx] = x + c.store * fb;
            c.idx = (c.idx + 1) % c.buf.len();
            out += y;
        }
        for a in alls.iter_mut() {
            let b = a.buf[a.idx];
            a.buf[a.idx] = out + b * 0.5;
            out = b - out;
            a.idx = (a.idx + 1) % a.buf.len();
        }
        *s += out * wet;
    }
}

// ---------------------------------------------------------------------------
// Instruments
// ---------------------------------------------------------------------------

/// An FM bell / glockenspiel note: bright strike that mellows into a
/// pure ring, plus an inharmonic partial for metal. `ratio` 3.5 is a
/// glassy tine, 1.4 a church bell, 2 a warm chime.
pub fn bell(f: f32, ratio: f32, index: f32, dur: f32) -> Vec<f32> {
    let mut fm = Fm::new();
    let mut part = Osc::default();
    let tau = dur * 0.32;
    render(dur, |_, t| {
        let e = perc(t, 0.0015, tau, dur);
        let idx = index * (-t / (dur * 0.12)).exp();
        let body = fm.next(f, ratio, idx);
        let metal = part.sine(f * 2.756) * (-t / (dur * 0.07)).exp() * 0.35;
        (body + metal) * e
    })
}

/// A marimba-ish mallet note: fundamental, a fast-dying 4th partial and a
/// tiny mallet knock.
pub fn mallet(f: f32, dur: f32, seed: u32) -> Vec<f32> {
    let mut o1 = Osc::default();
    let mut o4 = Osc::default();
    let mut rng = Rng::new(seed);
    let mut knock = Svf::new(f * 6.0, 1.5);
    render(dur, |_, t| {
        let e = perc(t, 0.001, dur * 0.25, dur);
        let n = knock.bp(rng.next()) * (-t / 0.004).exp() * 0.5;
        (o1.sine(f) + o4.sine(f * 3.93) * 0.4 * (-t / 0.05).exp()) * e + n
    })
}

/// A Karplus-Strong pluck: a noise burst ringing in a tuned, damped
/// delay line. Harps, plucked strings, bright twangs.
pub fn pluck(f: f32, dur: f32, bright: f32, seed: u32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let period = (SR / f).max(2.0);
    let n = period as usize;
    let frac = period - n as f32;
    let mut line: Vec<f32> = (0..n + 2).map(|_| rng.next()).collect();
    // Shape the excitation: softer pluck = darker burst.
    let mut lp = OnePole::new(1000.0 + 9000.0 * bright);
    for s in line.iter_mut() {
        *s = lp.run(*s);
    }
    let mut idx = 0;
    let len = line.len();
    let mut prev = 0.0;
    let decay = 0.996 + 0.0035 * bright.clamp(0.0, 1.0);
    let total = samples(dur);
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        let a = line[idx];
        let b = line[(idx + 1) % len];
        let y = a + (b - a) * frac;
        let avg = 0.5 * (y + prev) * decay;
        prev = y;
        line[idx] = avg;
        idx = (idx + 1) % len;
        let t = i as f32 / SR;
        out.push(y * perc(t, 0.0005, dur * 0.5, dur));
    }
    out
}

/// A synth-brass chord: per note, five detuned band-limited saws through
/// a low-pass whose cutoff opens with the attack ("blat") and settles,
/// a little delayed vibrato, and a gentle drive. `notes` in Hz.
pub fn brass(notes: &[f32], dur: f32, gate: f32, bright: f32, seed: u32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let detune = [-0.11, -0.05, 0.0, 0.05, 0.11];
    let mut voices: Vec<(f32, Osc)> = Vec::new();
    for &f in notes {
        for d in detune {
            voices.push((semis(f, d), Osc::with_phase(rng.unit())));
        }
    }
    let mut filt = Svf::new(800.0, 0.9);
    let mut filt2 = Svf::new(800.0, 0.6);
    let norm = 1.0 / (voices.len() as f32).sqrt();
    let base = notes.iter().cloned().fold(f32::MAX, f32::min);
    render(dur, |_, t| {
        let env = adsr(t, 0.018, 0.25, 0.75, 0.22, gate);
        // Filter envelope: bright blat at the attack, then settle.
        let fe = adsr(t, 0.03, 0.18, 0.55, 0.2, gate);
        let cut = base * (1.5 + (5.0 + 10.0 * bright) * fe);
        filt.tune(cut, 1.1);
        filt2.tune(cut * 1.3, 0.6);
        let vib = 1.0 + 0.004 * (TAU * 5.5 * t).sin() * smooth((t - 0.25) / 0.3);
        let mut s = 0.0;
        for (f, o) in voices.iter_mut() {
            s += o.saw(*f * vib);
        }
        let y = filt2.lp(filt.lp(s * norm));
        drive(y * 1.3, 1.6) * env
    })
}

/// A sleigh-bell shake: a cluster of tiny inharmonic bells and a metallic
/// noise rattle, `dur` long.
pub fn jingle(dur: f32, count: usize, seed: u32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut b = Vec::new();
    for _ in 0..count {
        let f = rng.range(3200.0, 6200.0);
        let at = rng.unit() * dur * 0.35;
        mix(&mut b, &bell(f, rng.range(1.31, 1.47), rng.range(1.0, 2.2), rng.range(0.07, 0.16)), at, rng.range(0.4, 1.0));
    }
    let mut n = Rng::new(seed ^ 0x55);
    let mut bp = Svf::new(7500.0, 2.0);
    let rattle = render(dur * 0.6, |_, t| bp.bp(n.next()) * perc(t, 0.002, dur * 0.12, dur * 0.6) * 0.35);
    mix(&mut b, &rattle, 0.0, 1.0);
    b
}

/// Band-passed noise whose centre glides `f0` -> `f1` (exponentially),
/// shaped by an attack and an exponential decay.
pub fn whoosh(f0: f32, f1: f32, q: f32, dur: f32, attack: f32, seed: u32) -> Vec<f32> {
    let mut pink = Pink::new(seed);
    let mut f = Svf::new(f0, q);
    render(dur, |i, t| {
        if i % 4 == 0 {
            f.tune(glide(f0, f1, t / dur), q);
        }
        let e = adsr(t, attack, dur * 0.4, 0.0, 0.01, dur);
        f.bp(pink.next() * 3.0) * e
    })
}

/// Sparse bright crackle: random impulses, each a tiny decaying burst of
/// band-passed noise. `density` per second follows `env(t)`.
pub fn crackle(dur: f32, density: f32, centre: f32, seed: u32, env: impl Fn(f32) -> f32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut bp = Svf::new(centre, 1.2);
    let mut level = 0.0f32;
    let mut amp = 0.0f32;
    let decay = (-1.0 / (0.0025 * SR)).exp();
    render(dur, |_, t| {
        let e = env(t);
        if rng.unit() < density * e / SR {
            amp = rng.range(0.3, 1.0);
            level = 1.0;
        }
        level *= decay;
        bp.bp(rng.next() * level * amp) * 2.0
    })
}

/// Lumps of snow or debris landing: soft low-passed thumps scattered over
/// `dur` with thinning density.
pub fn pitter(dur: f32, count: usize, cutoff: f32, seed: u32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut b = Vec::new();
    for _ in 0..count {
        let at = rng.unit().powf(1.6) * dur;
        let len = rng.range(0.03, 0.08);
        let mut lp = Svf::new(cutoff * rng.range(0.6, 1.4), 0.8);
        let mut n = Rng::new(rng.next().to_bits());
        let thump = render(len, |_, t| lp.lp(n.next()) * perc(t, 0.001, len * 0.3, len));
        let gain = rng.range(0.25, 1.0) * (1.0 - at / dur * 0.7);
        mix(&mut b, &thump, at, gain);
    }
    b
}

// ---------------------------------------------------------------------------
// Finishing
// ---------------------------------------------------------------------------

/// The mastering chain for a one-shot: DC block, soft limit, trim the
/// silent tail, loudness match (loudest 50 ms RMS to `rms_db`, but never
/// above `peak_db`), then raised-cosine fades so it starts and ends on
/// exact zero.
pub fn finish(mut buf: Vec<f32>, rms_db: f32, peak_db: f32) -> Vec<f32> {
    dc_block(&mut buf);
    let p = peak(&buf).max(1e-6);
    // Soft limit the top 6 dB of the raw mix so transients don't dictate
    // the level of everything else.
    for s in buf.iter_mut() {
        let x = *s / p;
        *s = if x.abs() < 0.5 { x } else { x.signum() * (0.5 + 0.5 * ((x.abs() - 0.5) * 2.0).tanh()) };
    }
    // Trim trailing near-silence (-70 dB under the peak), keep 30 ms.
    let floor = peak(&buf) * undb(-70.0);
    let mut end = buf.len();
    while end > 1 && buf[end - 1].abs() < floor {
        end -= 1;
    }
    buf.truncate((end + samples(0.03)).min(buf.len()));
    let g = (undb(rms_db) / max_rms(&buf).max(1e-6)).min(undb(peak_db) / peak(&buf).max(1e-6));
    let fin = samples(0.0015).max(1);
    let fout = samples(0.02).max(1).min(buf.len() / 3);
    let n = buf.len();
    for (i, s) in buf.iter_mut().enumerate() {
        let a = if i < fin { 0.5 - 0.5 * (i as f32 / fin as f32 * PI).cos() } else { 1.0 };
        let r = n - 1 - i;
        let b = if r < fout { 0.5 - 0.5 * (r as f32 / fout as f32 * PI).cos() } else { 1.0 };
        *s *= g * a * b;
    }
    if let Some(first) = buf.first_mut() {
        *first = 0.0;
    }
    if let Some(last) = buf.last_mut() {
        *last = 0.0;
    }
    buf
}

/// One-pole DC blocker (about 20 Hz).
pub fn dc_block(buf: &mut [f32]) {
    let r = 1.0 - TAU * 20.0 / SR;
    let (mut x1, mut y1) = (0.0f32, 0.0f32);
    for s in buf.iter_mut() {
        let y = *s - x1 + r * y1;
        x1 = *s;
        y1 = y;
        *s = y;
    }
}

/// Make a seamless loop of `len` seconds: render `len + xf` seconds with
/// `f`, then equal-power cross-fade the overhang into the head. The last
/// sample flows into the first exactly as it did in the render.
pub fn make_loop(len: f32, xf: f32, rms_db: f32, mut f: impl FnMut(usize, f32) -> f32) -> Vec<f32> {
    let n = samples(len);
    let x = samples(xf);
    let mut raw: Vec<f32> = (0..n + x).map(|i| f(i, i as f32 / SR)).collect();
    dc_block(&mut raw);
    let mut out = raw[..n].to_vec();
    for i in 0..x {
        let k = i as f32 / x as f32 * PI * 0.5;
        out[i] = raw[i] * k.sin() + raw[n + i] * k.cos();
    }
    let mut hp = Svf::new(100.0, 0.7);
    let rms = (out.iter().map(|s| hp.hp(*s).powi(2)).sum::<f32>() / n.max(1) as f32).sqrt();
    let g = (undb(rms_db) / rms.max(1e-6)).min(undb(-3.0) / peak(&out).max(1e-6));
    for s in out.iter_mut() {
        *s *= g;
    }
    out
}
