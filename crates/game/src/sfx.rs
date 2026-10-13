//! Sound at runtime: every effect comes from `hamsters-synth` (or a pack's
//! own `.ogg`/`.wav`), is wrapped as a WAV and handed to macroquad's audio.
//! Which sound plays for which event lives in `App::on_event`.
//!
//! The bank fills in the background (loops first, then the UI and launch
//! sounds, then the rest) so startup never waits on synthesis; a sound
//! asked for before it exists is simply skipped. One-shots come in several
//! variants and the runtime rotates them (never the same one twice in a
//! row) with a little level jitter, so repeats don't sound mechanical.
//!
//! The mix: one-shots are loudness-matched in the synth, the table in
//! `mix` sets their relative levels, big moments duck the music, and on
//! the web a master limiter (`web/quad_snd.js`) keeps the sum clean.

use hamsters_synth::{ogg_duration, synth_variant, variants, wav, PRESENT_STEPS};
use macroquad::audio::{load_sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::{get_time, load_file};
use std::cell::Cell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Poll;

/// Music level while playing, and while the results card or crash hold is up.
const MUSIC_VOLUME: f32 = 0.7;
const MUSIC_DUCKED: f32 = 0.42;
/// Silence between playlist tracks, seconds.
const TRACK_GAP: f32 = 1.5;

/// Relative one-shot levels (0..1 runtime volume). The synth already
/// matched loudness, so these are mix decisions, not corrections.
mod mix {
    pub const UI: f32 = 0.8;
    pub const FLAP: f32 = 0.75;
    pub const GUST: f32 = 0.55;
    pub const PRESENT: f32 = 0.85;
    pub const IMPACT: f32 = 1.0;
    pub const STING: f32 = 0.9;
    pub const PICKUP: f32 = 0.85;
    pub const SHELL: f32 = 0.75;
    pub const JINGLE: f32 = 0.85;
}

/// One playlist track: the sound and its length, so the next can follow.
struct Track {
    sound: Sound,
    duration: f32,
}

/// Everything the background loader has finished so far.
#[derive(Default)]
struct Bank {
    one: HashMap<&'static str, Vec<Sound>>,
    /// Present chimes, one list of variants per rung of the ladder.
    present: Vec<Vec<Sound>>,
    loops: HashMap<&'static str, Sound>,
}

/// Loops, in the order they cross-fade: wind low/mid/high, sled slow/fast,
/// rocket burn.
const LOOPS: [&str; 6] = ["wind", "wind_mid", "wind_high", "slide", "slide_fast", "rocket"];

/// One-shots in loading order: what the title and the first run need first.
const ONE_SHOTS: [&str; 30] = [
    "click", "select", "lock", "launch", "jump", "botch", "flap", "gust", "bounce", "land", "crash", "kaboom", "milestone",
    "firework", "pop", "zap", "freeze", "thaw", "meteor", "power", "trampoline", "reindeer", "updraft", "sleigh", "ufo",
    "wings", "record", "results", "results_best", "results_crash",
];

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

/// Every sound the runtime can play.
pub struct Sfx {
    bank: Arc<Mutex<Bank>>,
    /// Variant picker state: a xorshift and the last variant per sound.
    rng: Cell<u32>,
    last: Cell<[u8; 64]>,
    /// Three looped wind layers, cross-faded by airspeed: rumble, gusty
    /// mid band, and a howl for real speed.
    wind_volume: [f32; 3],
    /// Sled runners, slow and fast layers.
    slide_volume: [f32; 2],
    /// Rocket / meteor fire.
    burn_volume: f32,
    /// Music ducking under big moments: depth (multiplier) and time left.
    duck_depth: Cell<f32>,
    duck_left: Cell<f32>,
    /// A results jingle waiting for the card: (seconds left, name).
    results_due: Option<(f32, &'static str)>,
    /// Pack music, `music_1.ogg` onward; empty when the pack ships none.
    /// Filled by a background coroutine so startup never waits for it.
    music: Arc<Mutex<Vec<Track>>>,
    /// The playlist was asked to start (first gesture) but had no tracks yet.
    music_wanted: bool,
    music_idx: usize,
    music_t: f32,
    music_on: bool,
    music_playing: bool,
    music_volume: f32,
}

/// A pack file for `name`, if the pack ships one (`.ogg` first, then `.wav`).
async fn pack_sound(sound_dir: &str, name: &str) -> Option<Sound> {
    for ext in ["ogg", "wav"] {
        let path = format!("{sound_dir}/{name}.{ext}");
        #[cfg(not(target_arch = "wasm32"))]
        if !std::path::Path::new(&path).exists() {
            continue;
        }
        if let Ok(s) = load_sound(&path).await {
            return Some(s);
        }
    }
    None
}

/// Runs futures side by side and collects their outputs in order. No
/// executor needed: macroquad re-polls a coroutine every frame, so a dozen
/// decodes that each wait a frame finish together instead of in a queue.
async fn join_all<T, F: Future<Output = T>>(futs: Vec<F>) -> Vec<T> {
    let mut futs: Vec<Pin<Box<F>>> = futs.into_iter().map(Box::pin).collect();
    let mut out: Vec<Option<T>> = futs.iter().map(|_| None).collect();
    std::future::poll_fn(move |cx| {
        let mut pending = false;
        for (f, o) in futs.iter_mut().zip(out.iter_mut()) {
            if o.is_none() {
                match f.as_mut().poll(cx) {
                    Poll::Ready(v) => *o = Some(v),
                    Poll::Pending => pending = true,
                }
            }
        }
        if pending {
            Poll::Pending
        } else {
            Poll::Ready(out.iter_mut().map(|o| o.take().expect("joined output")).collect())
        }
    })
    .await
}

/// Fills the bank: probes the pack for overrides (all at once), then
/// synthesizes in batches of about 8 ms of work per frame and decodes each
/// batch in parallel. Loops start (silent) the moment they exist.
async fn fill_bank(dir: String, bank: Arc<Mutex<Bank>>) {
    let mut names: Vec<(&'static str, usize)> = LOOPS.iter().map(|n| (*n, 0)).collect();
    for (i, name) in ONE_SHOTS.iter().enumerate() {
        names.push((name, 0));
        // The present ladder goes in right after the flight basics.
        if i == 7 {
            names.extend((0..PRESENT_STEPS).map(|s| ("present", s)));
        }
    }
    let probes = join_all(
        names
            .iter()
            .map(|&(name, step)| {
                let dir = dir.clone();
                async move {
                    if step == 0 {
                        pack_sound(&dir, name).await
                    } else {
                        None
                    }
                }
            })
            .collect(),
    )
    .await;
    let mut i = 0;
    while i < names.len() {
        let t0 = get_time();
        let mut batch: Vec<(usize, Vec<Vec<u8>>)> = Vec::new();
        while i < names.len() && (batch.is_empty() || get_time() - t0 < 0.008) {
            let (name, step) = names[i];
            let wavs = if probes[i].is_some() { Vec::new() } else { (0..variants(name)).map(|v| wav(&synth_variant(name, step, v))).collect() };
            batch.push((i, wavs));
            i += 1;
        }
        let loaded = join_all(batch.iter().flat_map(|(_, w)| w.iter()).map(|bytes| load_sound_from_bytes(bytes)).collect()).await;
        let mut loaded = loaded.into_iter();
        for (idx, wavs) in &batch {
            let (name, _) = names[*idx];
            let list: Vec<Sound> = match &probes[*idx] {
                Some(s) => vec![s.clone()],
                None => loaded.by_ref().take(wavs.len()).filter_map(|r| r.ok()).collect(),
            };
            let mut b = bank.lock().unwrap();
            if LOOPS.contains(&name) {
                if let Some(s) = list.into_iter().next() {
                    play_sound(&s, PlaySoundParams { looped: true, volume: 0.0 });
                    b.loops.insert(name, s);
                }
            } else if name == "present" {
                b.present.push(list);
            } else {
                b.one.insert(name, list);
            }
        }
    }
}

/// Stable small index per sound name, for the no-repeat memory.
fn slot(name: &str) -> usize {
    name.bytes().fold(7usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize)) % 64
}

impl Sfx {
    /// Starts loading every sound (in the background) and the music.
    /// `sound_dir` is `packs/<pack>/sounds`.
    pub async fn load(sound_dir: &str) -> Sfx {
        let bank: Arc<Mutex<Bank>> = Arc::new(Mutex::new(Bank::default()));
        start_coroutine(fill_bank(sound_dir.to_owned(), bank.clone()));
        // Playlist: music_1.ogg, music_2.ogg ... until one is missing. The
        // tracks are megabytes, so they stream in behind the first frames.
        let music: Arc<Mutex<Vec<Track>>> = Arc::new(Mutex::new(Vec::new()));
        let shared = music.clone();
        let dir = sound_dir.to_owned();
        start_coroutine(async move {
            for n in 1.. {
                let path = format!("{dir}/music_{n}.ogg");
                #[cfg(not(target_arch = "wasm32"))]
                if !std::path::Path::new(&path).exists() {
                    break;
                }
                let Ok(bytes) = load_file(&path).await else { break };
                let Some(duration) = ogg_duration(&bytes) else { break };
                let Ok(sound) = load_sound_from_bytes(&bytes).await else { break };
                shared.lock().unwrap().push(Track { sound, duration });
            }
        });
        Sfx {
            bank,
            rng: Cell::new(0x2545_F491),
            last: Cell::new([255; 64]),
            wind_volume: [0.0; 3],
            slide_volume: [0.0; 2],
            burn_volume: 0.0,
            duck_depth: Cell::new(1.0),
            duck_left: Cell::new(0.0),
            results_due: None,
            music,
            music_wanted: false,
            music_idx: 0,
            music_t: 0.0,
            music_on: true,
            music_playing: false,
            music_volume: MUSIC_VOLUME,
        }
    }

    fn rand(&self) -> u32 {
        let mut x = self.rng.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng.set(x);
        x
    }

    /// Plays a variant of `name` (never the one played last) at `volume`
    /// with +-1 dB of jitter. Skipped if it hasn't loaded yet.
    fn play(&self, name: &str, volume: f32) {
        let bank = self.bank.lock().unwrap();
        let Some(list) = bank.one.get(name) else { return };
        self.pick(name, list, volume);
    }

    fn pick(&self, key: &str, list: &[Sound], volume: f32) {
        if list.is_empty() {
            return;
        }
        let mut last = self.last.get();
        let k = slot(key);
        let mut i = self.rand() as usize % list.len();
        if list.len() > 1 && i == last[k] as usize {
            i = (i + 1 + self.rand() as usize % (list.len() - 1)) % list.len();
        }
        last[k] = i as u8;
        self.last.set(last);
        let jitter = 0.89 + 0.22 * (self.rand() % 1000) as f32 / 1000.0;
        play_sound(&list[i], PlaySoundParams { looped: false, volume: (volume * jitter).clamp(0.0, 1.0) });
    }

    /// Dips the music to `depth` (0..1 of its level) for `secs`; the
    /// deepest, longest request wins while they overlap.
    pub fn duck(&self, depth: f32, secs: f32) {
        if self.duck_left.get() <= 0.0 {
            self.duck_depth.set(depth);
        } else {
            self.duck_depth.set(self.duck_depth.get().min(depth));
        }
        self.duck_left.set(self.duck_left.get().max(secs));
    }

    // -- UI -----------------------------------------------------------------

    /// A button or toggle.
    pub fn click(&self) {
        self.play("click", mix::UI);
    }

    /// Choosing a pilot.
    pub fn select(&self) {
        self.play("select", mix::UI);
    }

    /// The push power locked: the sled is off.
    pub fn lock(&self) {
        self.play("lock", mix::UI);
    }

    // -- Ride and flight ------------------------------------------------------

    pub fn flap(&self) {
        self.play("flap", mix::FLAP);
    }

    /// A short whoosh of air, for a flap.
    pub fn gust(&self) {
        self.play("gust", mix::GUST);
    }

    pub fn launch(&self) {
        self.play("launch", mix::IMPACT);
    }

    /// A perfect press at the lip.
    pub fn jump(&self) {
        self.play("jump", mix::STING);
        self.duck(0.45, 1.1);
    }

    /// A missed jump: the comic slide whistle down the building.
    pub fn botch(&self) {
        self.play("botch", mix::STING);
    }

    /// The wiggle streak refilled the stamina.
    pub fn power(&self) {
        self.play("power", mix::PICKUP);
    }

    /// The fall turned into a meteor.
    pub fn meteor(&self) {
        self.play("meteor", mix::IMPACT);
    }

    /// Overtook the next scoreboard flag mid-flight.
    pub fn record(&self) {
        self.play("record", mix::STING);
        self.duck(0.4, 2.2);
    }

    pub fn milestone(&self) {
        self.play("milestone", mix::PICKUP);
        self.duck(0.7, 1.0);
    }

    // -- Ground ---------------------------------------------------------------

    /// Louder with impact speed; a stomped landing is always full.
    pub fn bounce(&self, speed: f32, stomped: bool) {
        self.play("bounce", mix::IMPACT * if stomped { 1.0 } else { (speed / 25.0).clamp(0.3, 1.0) });
    }

    pub fn land(&self) {
        self.play("land", mix::IMPACT);
    }

    /// SPLAT (ordinary crash).
    pub fn crash(&self) {
        self.play("crash", mix::IMPACT);
        self.duck(0.75, 1.2);
    }

    /// The crash sound for an impact speed: SPLAT up to the meteor range,
    /// KABOOM beyond (same threshold as the banner in `App::on_event`).
    pub fn crash_impact(&self, speed: f32) {
        let big = ((speed - 35.0) / 35.0).clamp(0.0, 1.0);
        let big = big * big * (3.0 - 2.0 * big);
        if big > 0.5 {
            self.play("kaboom", mix::IMPACT);
            self.duck(0.3, 2.6);
        } else {
            self.crash();
        }
    }

    // -- Pickups --------------------------------------------------------------

    /// `combo` is the chain length so far, 1 for the first present.
    pub fn present(&self, combo: u32) {
        let bank = self.bank.lock().unwrap();
        if bank.present.is_empty() {
            return;
        }
        let i = (combo.max(1) as usize - 1).min(bank.present.len() - 1);
        self.pick("present", &bank.present[i], mix::PRESENT);
    }

    pub fn zap(&self) {
        self.play("zap", mix::IMPACT);
        self.duck(0.4, 1.4);
    }

    /// Grabbing a firework rocket (ignition, whistle, shells launching).
    pub fn firework(&self) {
        self.play("firework", mix::PICKUP);
    }

    /// A firework shell bursting in the sky.
    pub fn fireworks_pop(&self) {
        self.play("pop", mix::SHELL);
        self.duck(0.65, 0.8);
    }

    pub fn freeze(&self) {
        self.play("freeze", mix::PICKUP);
    }

    /// Breaking out of the ice.
    pub fn thaw(&self) {
        self.play("thaw", mix::PICKUP);
    }

    pub fn trampoline(&self) {
        self.play("trampoline", mix::PICKUP);
    }

    pub fn reindeer(&self) {
        self.play("reindeer", mix::PICKUP);
    }

    pub fn updraft(&self) {
        self.play("updraft", mix::PICKUP);
    }

    pub fn sleigh(&self) {
        self.play("sleigh", mix::PICKUP);
        self.duck(0.6, 1.4);
    }

    pub fn ufo(&self) {
        self.play("ufo", mix::PICKUP);
    }

    pub fn wings(&self) {
        self.play("wings", mix::PICKUP);
    }

    // -- Results --------------------------------------------------------------

    /// Schedules the results jingle for when the card comes up, `hold`
    /// seconds from now: a fanfare for a new best, a muted trombone after
    /// a crash, the cheerful tune otherwise. Cancelled by a restart.
    pub fn queue_results(&mut self, hold: f32, crashed: bool, new_best: bool) {
        let name = if new_best {
            "results_best"
        } else if crashed {
            "results_crash"
        } else {
            "results"
        };
        self.results_due = Some((hold, name));
    }

    // -- Loops ----------------------------------------------------------------

    fn set_loop(&self, name: &str, volume: f32) {
        if let Some(s) = self.bank.lock().unwrap().loops.get(name) {
            set_sound_volume(s, volume);
        }
    }

    /// The sled hiss: `level` 0..1 follows the ride down the ramp. Two
    /// layers cross-fade with it: a low, crunchy one at the start and a
    /// bright hiss near the lip, so the sound climbs in pitch with speed.
    pub fn set_slide(&mut self, level: f32, dt: f32) {
        let l = level.clamp(0.0, 1.0);
        let fast = ((l - 0.35) / 0.55).clamp(0.0, 1.0);
        let targets = [l * 0.55 * (1.0 - fast * 0.7), l * 0.6 * fast];
        let k = 1.0 - (-dt * 12.0).exp();
        for (i, name) in ["slide", "slide_fast"].into_iter().enumerate() {
            self.slide_volume[i] += (targets[i] - self.slide_volume[i]) * k;
            self.set_loop(name, self.slide_volume[i]);
        }
    }

    /// The wind layers follow airspeed: rumble from 8 m/s, the gusty mid
    /// band from 20, the howl past 40 or on a rocket. Each layer is
    /// smoothed so nothing steps, and they sit under the music (together
    /// about -29 dBFS at full speed against the music's -31 average).
    pub fn set_wind(&mut self, speed: f32, rocket: bool, dt: f32) {
        let low = ((speed - 8.0) / 35.0).clamp(0.0, 1.0) * 0.18;
        let mid = ((speed - 20.0) / 40.0).clamp(0.0, 1.0) * 0.13;
        let high = (((speed - 40.0) / 30.0).clamp(0.0, 1.0) * 0.1).max(if rocket && speed > 0.0 { 0.06 } else { 0.0 });
        let k = 1.0 - (-dt * 5.0).exp();
        for (i, target) in [low, mid, high].into_iter().enumerate() {
            self.wind_volume[i] += (target - self.wind_volume[i]) * k;
            self.set_loop(LOOPS[i], self.wind_volume[i]);
        }
    }

    /// Rocket thrust or meteor fire, `level` 0..1. Fast attack so the
    /// ignition and the roar land together, a slower fade when it burns out.
    pub fn set_burn(&mut self, level: f32, dt: f32) {
        let target = level.clamp(0.0, 1.0) * 0.5;
        let rate = if target > self.burn_volume { 18.0 } else { 3.0 };
        self.burn_volume += (target - self.burn_volume) * (1.0 - (-dt * rate).exp());
        self.set_loop("rocket", self.burn_volume);
    }

    // -- Music ----------------------------------------------------------------

    /// Starts the playlist; call on the first player gesture, since browsers
    /// block audio before one. Harmless to call again.
    pub fn start_music(&mut self) {
        self.music_wanted = true;
        let list = self.music.lock().unwrap();
        if self.music_playing || !self.music_on || list.is_empty() {
            return;
        }
        self.music_playing = true;
        self.music_t = 0.0;
        self.music_idx %= list.len();
        // A single track loops seamlessly; a playlist advances by the clock.
        play_sound(&list[self.music_idx].sound, PlaySoundParams { looped: list.len() == 1, volume: self.music_volume });
    }

    /// Advances the playlist clock and moves to the next track when one ends;
    /// `ducked` lowers the level (results card, crash hold). Also runs the
    /// ducking under big moments and the scheduled results jingle.
    pub fn update_music(&mut self, dt: f32, ducked: bool) {
        // The results jingle: only while the run is still on its card.
        if !ducked {
            self.results_due = None;
        }
        if let Some((left, name)) = self.results_due {
            let left = left - dt;
            if left <= 0.0 {
                self.results_due = None;
                self.play(name, mix::JINGLE);
                self.duck(0.35, if name == "results_best" { 3.5 } else { 2.5 });
            } else {
                self.results_due = Some((left, name));
            }
        }
        let left = self.duck_left.get() - dt;
        self.duck_left.set(left.max(0.0));
        let dip = if left > 0.0 { self.duck_depth.get() } else { 1.0 };
        let target = if ducked { MUSIC_DUCKED } else { MUSIC_VOLUME } * dip;
        // Duck fast (60 ms), recover slowly (about a second).
        let rate = if target < self.music_volume { 16.0 } else { 1.6 };
        self.music_volume += (target - self.music_volume) * (1.0 - (-dt * rate).exp());
        if !self.music_playing {
            // Tracks that arrived after the first gesture start now.
            if self.music_wanted && self.music_on && !self.music.lock().unwrap().is_empty() {
                self.start_music();
            }
            return;
        }
        let (duration, next) = {
            let list = self.music.lock().unwrap();
            set_sound_volume(&list[self.music_idx].sound, self.music_volume);
            (list[self.music_idx].duration, (self.music_idx + 1) % list.len())
        };
        self.music_t += dt;
        if self.music.lock().unwrap().len() > 1 && self.music_t >= duration + TRACK_GAP {
            self.music_idx = next;
            self.music_playing = false;
            self.start_music();
        }
    }

    /// Music on or off (the M key); returns the new state.
    pub fn toggle_music(&mut self) -> bool {
        self.set_music_enabled(!self.music_on);
        self.click();
        self.music_on
    }

    pub fn set_music_enabled(&mut self, on: bool) {
        self.music_on = on;
        if !on && self.music_playing {
            if let Some(track) = self.music.lock().unwrap().get(self.music_idx) {
                stop_sound(&track.sound);
            }
            self.music_playing = false;
        } else if on {
            self.start_music();
        }
    }
}
