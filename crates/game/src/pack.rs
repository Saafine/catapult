//! The active asset pack: which folder to load sprites from, and the strings
//! that change with the theme (title, milestone names, town signposts).

use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct PilotText {
    pub name: String,
    pub word: String,
}

#[derive(Deserialize, Clone)]
pub struct Labels {
    pub present: String,
    pub trampoline: String,
    pub reindeer: String,
    pub wings: String,
    pub firework: String,
    pub sleigh: String,
    pub ufo: String,
    pub updraft: String,
    pub coldwind: String,
    pub stormcloud: String,
    pub wiggle_power: String,
    pub flight_over: String,
    pub fly_again: String,
    pub flap: String,
    pub crash: String,
    pub jump_perfect: String,
    pub jump_good: String,
    pub jump_early: String,
    pub jump_missed: String,
    /// A press after the lip, in the coast. Older packs fall back to a default.
    #[serde(default = "default_jump_late")]
    pub jump_late: String,
    pub restart: String,
    pub scoreboard: String,
    /// Coach lines on the results card, one per skill; the weakest one shows.
    #[serde(default = "default_tip_push")]
    pub tip_push: String,
    #[serde(default = "default_tip_jump")]
    pub tip_jump: String,
    #[serde(default = "default_tip_air")]
    pub tip_air: String,
    /// The fall became a fireball; and the crash it makes.
    #[serde(default = "default_meteor")]
    pub meteor: String,
    #[serde(default = "default_crash_meteor")]
    pub crash_meteor: String,
}

fn default_meteor() -> String {
    "METEOR!".to_owned()
}

fn default_crash_meteor() -> String {
    "KABOOM!".to_owned()
}

fn default_tip_push() -> String {
    "The gold zone starts at 82".to_owned()
}

fn default_tip_jump() -> String {
    "Press when the ramp glows gold".to_owned()
}

fn default_tip_air() -> String {
    "Flap when you're falling, not rising".to_owned()
}

fn default_jump_late() -> String {
    "TOO LATE!".to_owned()
}

#[derive(Deserialize, Clone)]
pub struct Pack {
    pub name: String,
    pub title: String,
    pub tower_sign: String,
    pub milestones: Vec<String>,
    pub towns: Vec<String>,
    pub pilots: Vec<PilotText>,
    pub labels: Labels,
    /// Attribution lines for bundled music and art; shown nowhere yet.
    #[serde(default)]
    pub credits: Vec<String>,
}

#[derive(Deserialize)]
struct Active {
    pack: String,
}

impl Pack {
    /// Folder the sprites live in, relative to the page (wasm) or the
    /// working directory (native).
    pub fn sprite_dir(&self) -> String {
        format!("packs/{}/sprites", self.name)
    }

    /// The pack folder itself (pack.json, the optional city.json).
    pub fn pack_dir(&self) -> String {
        format!("packs/{}", self.name)
    }

    /// Folder for optional sound overrides (`<name>.ogg` or `.wav`).
    pub fn sound_dir(&self) -> String {
        format!("packs/{}/sounds", self.name)
    }

    /// Reads `packs/active.json` (or `HAMSTERS_PACK` on native) and then the
    /// pack's own `pack.json`.
    pub async fn load_active() -> Pack {
        let mut name = None;
        #[cfg(not(target_arch = "wasm32"))]
        {
            name = std::env::var("HAMSTERS_PACK").ok();
        }
        if name.is_none() {
            let raw = load_string("packs/active.json").await.expect("packs/active.json");
            let a: Active = serde_json::from_str(&raw).expect("packs/active.json is not valid json");
            name = Some(a.pack);
        }
        let name = name.unwrap();
        let raw = load_string(&format!("packs/{name}/pack.json")).await.unwrap_or_else(|e| panic!("packs/{name}/pack.json: {e:?}"));
        let mut pack: Pack = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("packs/{name}/pack.json: {e}"));
        pack.name = name;
        pack
    }

    pub fn milestone(&self, i: usize) -> &str {
        self.milestones.get(i).map(|s| s.as_str()).unwrap_or("MILESTONE!")
    }
}
