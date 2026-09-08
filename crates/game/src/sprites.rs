//! All textures, loaded at startup from the active asset pack
//! (`packs/<pack>/sprites/<name>.png`), so a whole look can be swapped
//! without rebuilding.

use macroquad::prelude::*;

async fn load_one(base: &str, name: &str) -> Texture2D {
    let path = format!("{base}/{name}.png");
    let t = load_texture(&path).await.unwrap_or_else(|e| panic!("missing sprite {path}: {e:?}"));
    t.set_filter(FilterMode::Linear);
    t
}

macro_rules! tex {
    ($base:expr, $name:literal) => {
        load_one($base, $name).await
    };
}

pub struct Sprites {
    pub dogs: [Texture2D; 3],
    /// Per dog: scream, panic, squint, dizzy, grit, whee.
    pub dogs_heads: [[Texture2D; 7]; 3],
    pub dogs_bodies: [[Texture2D; 7]; 3],
    /// One ear per pilot; drawn twice, mirrored, and flapped by the game.
    pub dogs_ear: [Texture2D; 3],
    pub santa: Texture2D,
    pub elf_a: Texture2D,
    pub elf_b: Texture2D,
    pub sled: Texture2D,
    pub tower: Texture2D,
    pub presents: [Texture2D; 3],
    pub trampoline: Texture2D,
    pub reindeer: Texture2D,
    pub wings: Texture2D,
    pub firework: Texture2D,
    pub sleigh: Texture2D,
    pub ufo: Texture2D,
    pub updraft: Texture2D,
    pub coldwind: Texture2D,
    pub icecube: Texture2D,
    pub moon: Texture2D,
    pub sun: Texture2D,
    pub clouds: [Texture2D; 3],
    pub snowflake: Texture2D,
    pub star: Texture2D,
    pub mountains_far: Texture2D,
    pub mountains_near: Texture2D,
    pub hills_far: Texture2D,
    pub hills_near: Texture2D,
    pub treeline: Texture2D,
    pub meadow: Texture2D,
    pub ground: Texture2D,
    pub xmas_tree: Texture2D,
    pub pine_snow: Texture2D,
    pub snowman: Texture2D,
    pub lamppost: Texture2D,
    pub candycane: Texture2D,
    pub fence: Texture2D,
    pub signpost: Texture2D,
    pub gift_pile: Texture2D,
    pub tuft: Texture2D,
    pub puff: Texture2D,
    pub sparkle: Texture2D,
    pub feather: Texture2D,
    pub ring: Texture2D,
    pub confetti: Texture2D,
    pub vignette: Texture2D,
    pub stormcloud: Texture2D,
}

impl Sprites {
    pub async fn load(base: &str) -> Self {
        Sprites {
            dogs: [tex!(base, "dog_biscuit"), tex!(base, "dog_nanook"), tex!(base, "dog_pixie")],
            dogs_heads: [
                [tex!(base, "dog_biscuit_head_idle"), tex!(base, "dog_biscuit_head_scream"), tex!(base, "dog_biscuit_head_panic"), tex!(base, "dog_biscuit_head_squint"), tex!(base, "dog_biscuit_head_dizzy"), tex!(base, "dog_biscuit_head_grit"), tex!(base, "dog_biscuit_head_whee")],
                [tex!(base, "dog_nanook_head_idle"), tex!(base, "dog_nanook_head_scream"), tex!(base, "dog_nanook_head_panic"), tex!(base, "dog_nanook_head_squint"), tex!(base, "dog_nanook_head_dizzy"), tex!(base, "dog_nanook_head_grit"), tex!(base, "dog_nanook_head_whee")],
                [tex!(base, "dog_pixie_head_idle"), tex!(base, "dog_pixie_head_scream"), tex!(base, "dog_pixie_head_panic"), tex!(base, "dog_pixie_head_squint"), tex!(base, "dog_pixie_head_dizzy"), tex!(base, "dog_pixie_head_grit"), tex!(base, "dog_pixie_head_whee")],
            ],
            dogs_ear: [tex!(base, "dog_biscuit_ear"), tex!(base, "dog_nanook_ear"), tex!(base, "dog_pixie_ear")],
            dogs_bodies: [
                [tex!(base, "dog_biscuit_body_seated"), tex!(base, "dog_biscuit_body_dive"), tex!(base, "dog_biscuit_body_spread"), tex!(base, "dog_biscuit_body_flap_a"), tex!(base, "dog_biscuit_body_flap_b"), tex!(base, "dog_biscuit_body_tumble"), tex!(base, "dog_biscuit_body_land")],
                [tex!(base, "dog_nanook_body_seated"), tex!(base, "dog_nanook_body_dive"), tex!(base, "dog_nanook_body_spread"), tex!(base, "dog_nanook_body_flap_a"), tex!(base, "dog_nanook_body_flap_b"), tex!(base, "dog_nanook_body_tumble"), tex!(base, "dog_nanook_body_land")],
                [tex!(base, "dog_pixie_body_seated"), tex!(base, "dog_pixie_body_dive"), tex!(base, "dog_pixie_body_spread"), tex!(base, "dog_pixie_body_flap_a"), tex!(base, "dog_pixie_body_flap_b"), tex!(base, "dog_pixie_body_tumble"), tex!(base, "dog_pixie_body_land")],
            ],
            santa: tex!(base, "santa"),
            elf_a: tex!(base, "elf_a"),
            elf_b: tex!(base, "elf_b"),
            sled: tex!(base, "sled"),
            tower: tex!(base, "tower"),
            presents: [tex!(base, "present_red"), tex!(base, "present_green"), tex!(base, "present_blue")],
            trampoline: tex!(base, "trampoline"),
            reindeer: tex!(base, "reindeer"),
            wings: tex!(base, "wings"),
            firework: tex!(base, "firework"),
            sleigh: tex!(base, "sleigh"),
            ufo: tex!(base, "ufo"),
            updraft: tex!(base, "updraft"),
            coldwind: tex!(base, "coldwind"),
            icecube: tex!(base, "icecube"),
            moon: tex!(base, "moon"),
            sun: tex!(base, "sun"),
            clouds: [tex!(base, "cloud_a"), tex!(base, "cloud_b"), tex!(base, "cloud_c")],
            snowflake: tex!(base, "snowflake"),
            star: tex!(base, "star"),
            mountains_far: tex!(base, "mountains_far"),
            mountains_near: tex!(base, "mountains_near"),
            hills_far: tex!(base, "hills_far"),
            hills_near: tex!(base, "hills_near"),
            treeline: tex!(base, "treeline"),
            meadow: tex!(base, "meadow"),
            ground: tex!(base, "ground"),
            xmas_tree: tex!(base, "xmas_tree"),
            pine_snow: tex!(base, "pine_snow"),
            snowman: tex!(base, "snowman"),
            lamppost: tex!(base, "lamppost"),
            candycane: tex!(base, "candycane"),
            fence: tex!(base, "fence"),
            signpost: tex!(base, "signpost"),
            gift_pile: tex!(base, "gift_pile"),
            tuft: tex!(base, "tuft"),
            puff: tex!(base, "puff"),
            sparkle: tex!(base, "sparkle"),
            feather: tex!(base, "feather"),
            ring: tex!(base, "ring"),
            confetti: tex!(base, "confetti"),
            vignette: tex!(base, "vignette"),
            stormcloud: tex!(base, "stormcloud"),
        }
    }
}

/// Draw a texture centred on `center`, `w` pixels wide, keeping aspect.
/// `rot` is counter-clockwise radians in world sense.
pub fn blit(tex: &Texture2D, center: Vec2, w: f32, rot: f32, flip_x: bool, tint: Color) {
    let h = w * tex.height() / tex.width();
    draw_texture_ex(
        tex,
        center.x - w / 2.0,
        center.y - h / 2.0,
        tint,
        DrawTextureParams { dest_size: Some(vec2(w, h)), rotation: -rot, flip_x, ..Default::default() },
    );
}

/// Like [`blit`] but with independent width and height (squash and stretch).
pub fn blit_wh(tex: &Texture2D, center: Vec2, w: f32, h: f32, rot: f32, flip_x: bool, tint: Color) {
    draw_texture_ex(
        tex,
        center.x - w / 2.0,
        center.y - h / 2.0,
        tint,
        DrawTextureParams { dest_size: Some(vec2(w, h)), rotation: -rot, flip_x, ..Default::default() },
    );
}


/// Draw with the sprite's bottom-centre at `foot` (things standing on the ground).
pub fn blit_standing(tex: &Texture2D, foot: Vec2, w: f32, flip_x: bool, tint: Color) {
    let h = w * tex.height() / tex.width();
    blit(tex, vec2(foot.x, foot.y - h / 2.0), w, 0.0, flip_x, tint);
}

/// Draw rotated about a pivot given in texture uv (0..1), placing that pivot at `at`.
pub fn blit_pivot(tex: &Texture2D, at: Vec2, pivot_uv: Vec2, w: f32, rot: f32, flip_x: bool, tint: Color) {
    let h = w * tex.height() / tex.width();
    let px = if flip_x { 1.0 - pivot_uv.x } else { pivot_uv.x };
    let x = at.x - px * w;
    let y = at.y - pivot_uv.y * h;
    draw_texture_ex(tex, x, y, tint, DrawTextureParams { dest_size: Some(vec2(w, h)), rotation: -rot, flip_x, pivot: Some(at), ..Default::default() });
}
