//! Procedural sound effects for the launch game. Pure DSP: no platform
//! code, so it compiles and tests anywhere; the game turns the samples
//! into WAVs for its audio backend.
//!
//! `dsp` is the toolkit (noise colours, band-limited and FM oscillators, a
//! state-variable filter for sweeps, ADSR, saturation, echo, a Freeverb
//! room, Karplus-Strong, sleigh bells, synth brass, the mastering chain);
//! `sounds` builds every effect from layers of it. One-shots come in
//! variants (reseeded noise, nudged pitch and timing) so the runtime can
//! rotate them and repeats never sound identical.
//!
//! `cargo run -p hamsters-synth --release --bin sfx_render -- <dir>` writes
//! every sound as a WAV plus a stats table (peak, RMS, length, edges).

pub mod dsp;
mod sounds;

pub use sounds::LADDER;

pub const RATE: u32 = 44100;

/// Sound names a pack may override, in `packs/<pack>/sounds/<name>.ogg|wav`.
pub const SOUND_NAMES: [&str; 38] = [
    "flap", "bounce", "present", "milestone", "land", "launch", "crash", "zap", "firework", "pop", "freeze", "jump",
    "trampoline", "reindeer", "updraft", "sleigh", "ufo", "wings", "gust", "wind", "wind_mid", "wind_high",
    "slide", "slide_fast", "rocket", "click", "select", "lock", "botch", "kaboom", "thaw", "meteor", "power",
    "record", "results", "results_best", "results_crash", "music",
];

/// Names that are seamless loops rather than one-shots.
pub const LOOP_NAMES: [&str; 6] = ["wind", "wind_mid", "wind_high", "slide", "slide_fast", "rocket"];

/// Number of rungs on the present chime's ladder.
pub const PRESENT_STEPS: usize = 9;

/// How many variants of a sound the runtime keeps to rotate through.
/// Busy sounds get more so a burst of flaps or shells never repeats.
pub fn variants(name: &str) -> usize {
    match name {
        "flap" | "pop" => 4,
        "gust" | "bounce" | "click" => 3,
        "present" | "crash" | "zap" | "launch" | "land" | "trampoline" | "reindeer" => 2,
        _ => 1,
    }
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

/// Synthesize variant 0 of a sound by name; the present chime takes its
/// ladder step. Empty for unknown names and for "music".
pub fn synth(name: &str, step: usize) -> Vec<f32> {
    synth_variant(name, step, 0)
}

/// Synthesize one variant of a sound by name.
pub fn synth_variant(name: &str, step: usize, v: usize) -> Vec<f32> {
    use sounds::*;
    let v = v as u32;
    match name {
        "flap" => flap(v),
        "bounce" => bounce(v),
        "present" => present(step, v),
        "milestone" => milestone(v),
        "land" => land(v),
        "launch" => launch(v),
        "crash" => crash(v),
        "kaboom" => kaboom(v),
        "zap" => zap(v),
        "firework" => firework(v),
        "pop" => pop(v),
        "freeze" => freeze(v),
        "thaw" => thaw(v),
        "jump" => perfect(v),
        "botch" => botch(v),
        "trampoline" => trampoline(v),
        "reindeer" => reindeer(v),
        "updraft" => updraft(v),
        "sleigh" => sleigh(v),
        "ufo" => ufo(v),
        "wings" => wings(v),
        "gust" => gust(v),
        "meteor" => meteor(v),
        "power" => power(v),
        "click" => click(v),
        "select" => select(v),
        "lock" => lock(v),
        "record" => record(v),
        "results" => results(v),
        "results_best" => results_best(v),
        "results_crash" => results_crash(v),
        "wind" => wind_loop(0),
        "wind_mid" => wind_loop(1),
        "wind_high" => wind_loop(2),
        "slide" => slide_loop(false),
        "slide_fast" => slide_loop(true),
        "rocket" => rocket_loop(),
        // Music is never synthesized: packs ship it as `music_N.ogg`.
        _ => Vec::new(),
    }
}

/// Numbers for one rendered sound, for the offline report.
#[derive(Clone, Debug)]
pub struct Stats {
    pub secs: f32,
    pub peak_db: f32,
    /// Loudest 50 ms RMS, 100 Hz high-passed (the loudness the mix uses).
    pub max_rms_db: f32,
    /// Loudest 50 ms RMS, unweighted.
    pub max_rms_raw_db: f32,
    /// RMS over the whole sound.
    pub rms_db: f32,
    pub crest_db: f32,
    /// Largest sample-to-sample step anywhere (bright sounds step a lot).
    pub max_step: f32,
    /// Largest step in the first 0.25 ms and the last 5 ms, where an
    /// onset or cut-off click would live.
    pub edge_step: f32,
    /// |first| and |last| sample.
    pub head: f32,
    pub tail: f32,
    /// Seam step for a loop (last sample to first).
    pub seam: f32,
    /// RMS frequency in Hz (brightness: where the energy sits).
    pub centroid: f32,
    pub dc: f32,
}

pub fn stats(s: &[f32]) -> Stats {
    let n = s.len().max(1) as f32;
    let peak = dsp::peak(s);
    let rms = (s.iter().map(|v| v * v).sum::<f32>() / n).sqrt();
    let max_step = s.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
    let step = |w: &[f32]| w.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
    let edge_step = step(&s[..dsp::samples(0.00025).min(s.len())]).max(step(&s[s.len() - dsp::samples(0.005).min(s.len())..]));
    // RMS frequency: the energy of the first difference over the energy of
    // the signal is (2 sin(pi f / SR))^2 for a sine at f.
    let d: f32 = s.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
    let e: f32 = s.iter().map(|v| v * v).sum::<f32>().max(1e-12);
    let centroid = ((d / e).sqrt() * 0.5).min(1.0).asin() / std::f32::consts::PI * dsp::SR;
    Stats {
        secs: s.len() as f32 / dsp::SR,
        peak_db: dsp::db(peak),
        max_rms_db: dsp::db(dsp::max_rms(s)),
        max_rms_raw_db: dsp::db(dsp::max_rms_raw(s)),
        rms_db: dsp::db(rms),
        crest_db: dsp::db(peak) - dsp::db(rms),
        max_step,
        edge_step,
        head: s.first().map_or(0.0, |v| v.abs()),
        tail: s.last().map_or(0.0, |v| v.abs()),
        seam: if s.len() > 1 { (s[0] - s[s.len() - 1]).abs() } else { 0.0 },
        centroid,
        dc: s.iter().sum::<f32>() / n,
    }
}

#[cfg(test)]
mod tests {
    use super::dsp::samples;
    use super::*;

    /// Every synthesized one-shot variant: no clipping, no clicks, exact
    /// zero at both ends, no DC. Writes WAVs into `SFX_OUT` if set.
    #[test]
    fn sounds_are_clean() {
        let out = std::env::var("SFX_OUT").ok();
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        for name in SOUND_NAMES {
            if name == "music" || LOOP_NAMES.contains(&name) {
                continue;
            }
            let steps = if name == "present" { PRESENT_STEPS } else { 1 };
            for step in 0..steps {
                for v in 0..variants(name) {
                    let s = synth_variant(name, step, v);
                    let st = stats(&s);
                    assert!(s.len() > samples(0.04), "{name} is too short");
                    assert!(st.peak_db <= -0.9, "{name} peaks at {} dB", st.peak_db);
                    assert!(st.edge_step <= 0.03, "{name} clicks at an edge: step {}", st.edge_step);
                    assert!(st.max_step <= 0.9, "{name} jumps {} in one sample", st.max_step);
                    assert!(st.head == 0.0 && st.tail == 0.0, "{name} edges: head {} tail {}", st.head, st.tail);
                    assert!(st.dc.abs() < 0.01, "{name} has DC {}", st.dc);
                    assert!(s.iter().all(|v| v.is_finite()), "{name} has NaN");
                    if let Some(dir) = &out {
                        std::fs::write(format!("{dir}/{name}_{step}_{v}.wav"), wav(&s)).unwrap();
                    }
                }
            }
        }
    }

    /// Variants differ (that is their point).
    #[test]
    fn variants_differ() {
        for name in ["flap", "pop", "bounce"] {
            let a = synth_variant(name, 0, 0);
            let b = synth_variant(name, 0, 1);
            let n = a.len().min(b.len());
            let diff: f32 = a[..n].iter().zip(&b[..n]).map(|(x, y)| (x - y).abs()).sum::<f32>() / n as f32;
            assert!(diff > 0.001, "{name} variants are identical");
        }
    }

    /// Loops: the seam (last sample to first) is as smooth as the body.
    #[test]
    fn loops_are_seamless() {
        for name in LOOP_NAMES {
            let s = synth(name, 0);
            let st = stats(&s);
            assert!(st.secs >= 3.9, "{name} is too short a loop");
            let body = st.max_step;
            assert!(st.seam <= body.max(0.05), "{name} seam jumps {} (body max {})", st.seam, body);
            assert!(st.peak_db <= -2.9, "{name} peaks at {}", st.peak_db);
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
        // Higher rungs have more zero crossings on the attack.
        let crossings = |s: &[f32]| {
            let a = &s[samples(0.02)..samples(0.06)];
            a.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count()
        };
        let lo = crossings(&synth("present", 0));
        let hi = crossings(&synth("present", 5));
        assert!(hi > lo, "rung 5 ({hi} crossings) should ring higher than rung 0 ({lo})");
    }
}
