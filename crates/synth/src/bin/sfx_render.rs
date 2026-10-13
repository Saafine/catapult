//! Offline render of every sound for inspection:
//! `cargo run -p hamsters-synth --release --bin sfx_render -- <out_dir>`
//! writes `<name>[_<step>]_v<variant>.wav`, `stats.md` (peak, RMS,
//! length, edges, seam, brightness, render time) and, for the sheet, a
//! log-frequency spectrogram (`<name>.spec`, 8-bit rows) and a waveform
//! envelope (`<name>.env`, min/max pairs) of variant 0 of each sound.
//!
//! `sfx_render --analyze <in.wav> <name> <out_dir>` writes the same
//! `.spec`/`.env` pair for any 16-bit mono WAV (e.g. a capture of the
//! in-game mix), with the frequency axis scaled to its sample rate.

use hamsters_synth::{stats, synth_variant, variants, wav, LOOP_NAMES, PRESENT_STEPS, RATE, SOUND_NAMES};
use std::fmt::Write as _;
use std::time::Instant;

const SPEC_W: usize = 360;
const SPEC_H: usize = 120;
const FFT: usize = 2048;

fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -std::f32::consts::TAU / len as f32;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (ang * k as f32).sin_cos();
                let a = start + k;
                let b = a + len / 2;
                let tr = re[b] * c - im[b] * s;
                let ti = re[b] * s + im[b] * c;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        len <<= 1;
    }
}

/// Columns over time, rows on a log axis 40 Hz .. 16 kHz (top = high),
/// magnitude in dB mapped -90..0 to 0..255.
fn spectrogram(s: &[f32]) -> Vec<u8> {
    spectrogram_at(s, RATE as f32, SPEC_W)
}

fn spectrogram_at(s: &[f32], rate: f32, width: usize) -> Vec<u8> {
    let spec_w = width;
    let mut out = vec![0u8; spec_w * SPEC_H];
    let hop = (s.len() as f32 / spec_w as f32).max(1.0);
    let win: Vec<f32> = (0..FFT).map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / FFT as f32).cos()).collect();
    for col in 0..spec_w {
        let centre = (col as f32 * hop) as isize;
        let mut re = vec![0.0f32; FFT];
        let mut im = vec![0.0f32; FFT];
        for i in 0..FFT {
            let at = centre + i as isize - FFT as isize / 2;
            if at >= 0 && (at as usize) < s.len() {
                re[i] = s[at as usize] * win[i];
            }
        }
        fft(&mut re, &mut im);
        for row in 0..SPEC_H {
            let k = 1.0 - row as f32 / (SPEC_H - 1) as f32;
            let f_lo = 40.0 * (16000.0f32 / 40.0).powf(k - 0.5 / SPEC_H as f32);
            let f_hi = 40.0 * (16000.0f32 / 40.0).powf(k + 0.5 / SPEC_H as f32);
            let b_lo = ((f_lo / rate * FFT as f32) as usize).max(1).min(FFT / 2 - 1);
            let b_hi = ((f_hi / rate * FFT as f32) as usize).max(b_lo + 1).min(FFT / 2);
            let mut m = 0.0f32;
            for b in b_lo..b_hi {
                m = m.max((re[b] * re[b] + im[b] * im[b]).sqrt());
            }
            let db = 20.0 * (m / (FFT as f32 * 0.25)).max(1e-9).log10();
            out[row * spec_w + col] = (((db + 90.0) / 90.0).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

fn envelope(s: &[f32]) -> String {
    envelope_at(s, SPEC_W)
}

fn envelope_at(s: &[f32], width: usize) -> String {
    let mut out = String::new();
    let per = (s.len() / width).max(1);
    for c in s.chunks(per).take(width) {
        let lo = c.iter().cloned().fold(0.0f32, f32::min);
        let hi = c.iter().cloned().fold(0.0f32, f32::max);
        let _ = writeln!(out, "{lo:.4} {hi:.4}");
    }
    out
}

/// Reads a 16-bit PCM mono WAV (44-byte header) as samples and its rate.
fn read_wav(path: &str) -> (Vec<f32>, f32) {
    let b = std::fs::read(path).expect("wav");
    let rate = u32::from_le_bytes(b[24..28].try_into().unwrap()) as f32;
    let s = b[44..].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect();
    (s, rate)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--analyze") {
        let (s, rate) = read_wav(&args[2]);
        let (name, dir) = (&args[3], &args[4]);
        // As wide as `cells` sheet cells (default 3, for a session capture).
        let cells: usize = args.get(5).and_then(|c| c.parse().ok()).unwrap_or(3);
        let width = SPEC_W * cells + 20 * (cells - 1);
        let st = stats(&s);
        let line = format!("{:.2}|{:.1}|{:.1}|{:.0}\n", st.secs * RATE as f32 / rate, st.peak_db, st.max_rms_db, st.centroid * rate / RATE as f32);
        std::fs::write(format!("{dir}/{name}.stat"), line).unwrap();
        std::fs::write(format!("{dir}/{name}.spec"), spectrogram_at(&s, rate, width)).unwrap();
        std::fs::write(format!("{dir}/{name}.env"), envelope_at(&s, width)).unwrap();
        println!("{name}: {:.1} s at {rate} Hz, width {width}", s.len() as f32 / rate);
        return;
    }
    let dir = std::env::args().nth(1).unwrap_or_else(|| "sfx_out".into());
    std::fs::create_dir_all(&dir).unwrap();
    let mut table = String::from(
        "| sound | var | len s | peak dBFS | loud (50ms RMS, HP100) dBFS | raw 50ms RMS dBFS | avg RMS dBFS | crest dB | max step | edge step | head | tail | seam | bright Hz | DC | ms |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    let mut total_ms = 0.0;
    let mut total_secs = 0.0;
    for name in SOUND_NAMES {
        if name == "music" {
            continue;
        }
        let steps = if name == "present" { PRESENT_STEPS } else { 1 };
        for step in 0..steps {
            for v in 0..variants(name) {
                let t0 = Instant::now();
                let s = synth_variant(name, step, v);
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                total_ms += ms;
                let st = stats(&s);
                total_secs += st.secs;
                let label = if steps > 1 { format!("{name}_{step}") } else { name.to_string() };
                let seam = if LOOP_NAMES.contains(&name) { format!("{:.4}", st.seam) } else { "-".into() };
                let _ = writeln!(
                    table,
                    "| {label} | {v} | {:.2} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.3} | {:.4} | {:.4} | {:.4} | {seam} | {:.0} | {:.5} | {ms:.1} |",
                    st.secs, st.peak_db, st.max_rms_db, st.max_rms_raw_db, st.rms_db, st.crest_db, st.max_step, st.edge_step, st.head, st.tail, st.centroid, st.dc
                );
                std::fs::write(format!("{dir}/{label}_v{v}.wav"), wav(&s)).unwrap();
                if v == 0 && (steps == 1 || step == 0 || step == steps - 1) {
                    std::fs::write(format!("{dir}/{label}.spec"), spectrogram(&s)).unwrap();
                    std::fs::write(format!("{dir}/{label}.env"), envelope(&s)).unwrap();
                }
            }
        }
    }
    let _ = writeln!(table, "\nTotal: {total_secs:.1} s of audio rendered in {total_ms:.0} ms (native release).");
    std::fs::write(format!("{dir}/stats.md"), &table).unwrap();
    println!("{table}");
}
