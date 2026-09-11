//! Sound at runtime: every effect comes from `hamsters-synth` (or a pack's
//! own `.ogg`/`.wav`), is wrapped as a WAV and handed to macroquad's audio.
//! Which sound plays for which event lives in `App::on_event`.

use hamsters_synth::{ogg_duration, synth, wav, PRESENT_STEPS};
use macroquad::audio::{load_sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::load_file;
use std::sync::{Arc, Mutex};

/// Music level while playing, and while the results card or crash hold is up.
const MUSIC_VOLUME: f32 = 0.28;
const MUSIC_DUCKED: f32 = 0.12;
/// Silence between playlist tracks, seconds.
const TRACK_GAP: f32 = 1.5;

/// One playlist track: the sound and its length, so the next can follow.
struct Track {
    sound: Sound,
    duration: f32,
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

/// Every sound the runtime can play.
pub struct Sfx {
    flap: Sound,
    bounce: Sound,
    /// One chime per combo step up the pentatonic ladder.
    present: Vec<Sound>,
    milestone: Sound,
    land: Sound,
    launch: Sound,
    crash: Sound,
    zap: Sound,
    firework: Sound,
    pop: Sound,
    freeze: Sound,
    jump: Sound,
    trampoline: Sound,
    reindeer: Sound,
    updraft: Sound,
    sleigh: Sound,
    ufo: Sound,
    wings: Sound,
    gust: Sound,
    /// Three looped wind layers, cross-faded by airspeed: rumble, gusty
    /// mid band, and a whistle for real speed.
    wind: [Sound; 3],
    wind_volume: [f32; 3],
    /// Sled runners on the ramp, looped; rises with the slide.
    slide: Sound,
    slide_volume: f32,
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

impl Sfx {
    /// Loads every sound: the pack's file when it has one, else synthesized.
    /// `sound_dir` is `packs/<pack>/sounds`.
    pub async fn load(sound_dir: &str) -> Sfx {
        async fn get(dir: &str, name: &str, step: usize) -> Sound {
            if step == 0 {
                if let Some(s) = pack_sound(dir, name).await {
                    return s;
                }
            }
            load_sound_from_bytes(&wav(&synth(name, step))).await.expect("synthesized sound")
        }
        let mut present = Vec::new();
        for step in 0..PRESENT_STEPS {
            present.push(get(sound_dir, "present", step).await);
        }
        let wind = [get(sound_dir, "wind", 0).await, get(sound_dir, "wind_mid", 0).await, get(sound_dir, "wind_high", 0).await];
        for w in &wind {
            play_sound(w, PlaySoundParams { looped: true, volume: 0.0 });
        }
        let slide = get(sound_dir, "slide", 0).await;
        play_sound(&slide, PlaySoundParams { looped: true, volume: 0.0 });
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
            flap: get(sound_dir, "flap", 0).await,
            bounce: get(sound_dir, "bounce", 0).await,
            present,
            milestone: get(sound_dir, "milestone", 0).await,
            land: get(sound_dir, "land", 0).await,
            launch: get(sound_dir, "launch", 0).await,
            crash: get(sound_dir, "crash", 0).await,
            zap: get(sound_dir, "zap", 0).await,
            firework: get(sound_dir, "firework", 0).await,
            pop: get(sound_dir, "pop", 0).await,
            freeze: get(sound_dir, "freeze", 0).await,
            jump: get(sound_dir, "jump", 0).await,
            trampoline: get(sound_dir, "trampoline", 0).await,
            reindeer: get(sound_dir, "reindeer", 0).await,
            updraft: get(sound_dir, "updraft", 0).await,
            sleigh: get(sound_dir, "sleigh", 0).await,
            ufo: get(sound_dir, "ufo", 0).await,
            wings: get(sound_dir, "wings", 0).await,
            gust: get(sound_dir, "gust", 0).await,
            wind,
            wind_volume: [0.0; 3],
            slide,
            slide_volume: 0.0,
            music,
            music_wanted: false,
            music_idx: 0,
            music_t: 0.0,
            music_on: true,
            music_playing: false,
            music_volume: MUSIC_VOLUME,
        }
    }

    fn once(s: &Sound, volume: f32) {
        play_sound(s, PlaySoundParams { looped: false, volume: volume.clamp(0.0, 1.0) });
    }

    pub fn flap(&self) {
        Self::once(&self.flap, 1.0);
    }

    /// Louder with impact speed; a stomped landing is always full.
    pub fn bounce(&self, speed: f32, stomped: bool) {
        Self::once(&self.bounce, if stomped { 1.0 } else { (speed / 25.0).clamp(0.3, 1.0) });
    }

    /// `combo` is the chain length so far, 1 for the first present.
    pub fn present(&self, combo: u32) {
        let i = (combo.max(1) as usize - 1).min(self.present.len() - 1);
        Self::once(&self.present[i], 1.0);
    }

    pub fn milestone(&self) {
        Self::once(&self.milestone, 1.0);
    }

    pub fn land(&self) {
        Self::once(&self.land, 1.0);
    }

    pub fn crash(&self) {
        Self::once(&self.crash, 1.0);
    }

    pub fn launch(&self) {
        Self::once(&self.launch, 1.0);
    }

    pub fn zap(&self) {
        Self::once(&self.zap, 1.0);
    }

    pub fn firework(&self) {
        Self::once(&self.firework, 1.0);
    }

    /// A firework shell bursting in the sky.
    pub fn fireworks_pop(&self) {
        Self::once(&self.pop, 0.8);
    }

    pub fn freeze(&self) {
        Self::once(&self.freeze, 1.0);
    }

    /// A perfect press at the lip.
    pub fn jump(&self) {
        Self::once(&self.jump, 1.0);
    }

    pub fn trampoline(&self) {
        Self::once(&self.trampoline, 1.0);
    }

    pub fn reindeer(&self) {
        Self::once(&self.reindeer, 1.0);
    }

    pub fn updraft(&self) {
        Self::once(&self.updraft, 1.0);
    }

    pub fn sleigh(&self) {
        Self::once(&self.sleigh, 1.0);
    }

    pub fn ufo(&self) {
        Self::once(&self.ufo, 1.0);
    }

    pub fn wings(&self) {
        Self::once(&self.wings, 1.0);
    }

    /// A short whoosh of air, for a flap.
    pub fn gust(&self) {
        Self::once(&self.gust, 0.8);
    }

    /// The sled hiss: `level` 0..1 follows the ride down the ramp.
    pub fn set_slide(&mut self, level: f32, dt: f32) {
        let target = level.clamp(0.0, 1.0) * 0.55;
        let k = 1.0 - (-dt * 12.0).exp();
        self.slide_volume += (target - self.slide_volume) * k;
        set_sound_volume(&self.slide, self.slide_volume);
    }

    /// The wind layers follow airspeed: rumble from 10 m/s, the gusty mid
    /// band from 25, the whistle only past 45 or on a rocket. Each layer is
    /// smoothed so nothing steps; together they stay under about 0.4.
    pub fn set_wind(&mut self, speed: f32, rocket: bool, dt: f32) {
        let low = ((speed - 10.0) / 40.0).clamp(0.0, 1.0) * 0.22;
        let mid = ((speed - 25.0) / 40.0).clamp(0.0, 1.0) * 0.14;
        let high = (((speed - 45.0) / 30.0).clamp(0.0, 1.0) * 0.08).max(if rocket { 0.07 } else { 0.0 });
        let k = 1.0 - (-dt * 5.0).exp();
        for (i, target) in [low, mid, high].into_iter().enumerate() {
            self.wind_volume[i] += (target - self.wind_volume[i]) * k;
            set_sound_volume(&self.wind[i], self.wind_volume[i]);
        }
    }

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
    /// `ducked` lowers the level (results card, crash hold).
    pub fn update_music(&mut self, dt: f32, ducked: bool) {
        let target = if ducked { MUSIC_DUCKED } else { MUSIC_VOLUME };
        self.music_volume += (target - self.music_volume) * (1.0 - (-dt * 4.0).exp());
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

