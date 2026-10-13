//! Image pipeline: how the world's pixels reach the screen.
//!
//! The world is drawn into an off-screen, supersampled scene buffer instead
//! of straight to the canvas. A short chain of small passes then builds a
//! bloom pyramid and sun shafts, and one composite pass resolves the
//! supersampling, adds the glow, grades the colour (warm highlights, cool
//! shadows, a little vibrance and contrast, warmer with the afternoon),
//! darkens the corners and dithers with a whisper of film grain. The HUD is
//! drawn afterwards, straight to the canvas, so its text stays crisp and
//! ungraded.
//!
//! Also here: sprite loading with alpha bleeding and mipmaps, so painted
//! sprites shrink without shimmering or dark fringes.
//!
//! Everything is GLSL 100 (WebGL1). If any shader fails to compile the
//! effect it belongs to is simply skipped and the frame is drawn as before.
//!
//! Hook (main.rs `render`): `post::begin()` before the world, `post::end(cam)`
//! after it and before the HUD. `draw::sky` reports the sun with `set_sun`.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, MipmapFilterMode, PipelineParams, UniformType};
use macroquad::prelude::*;
use std::cell::RefCell;

/// Post-processing on at startup. Dev builds toggle with O (on, off, split A/B).
pub const POST_DEFAULT: bool = true;
/// Supersampling limits for the scene buffer: at most 2x per axis, and no
/// more than this many pixels in total (fill cost on big low-dpi screens).
const SS_MAX: f32 = 2.0;
const SS_PIXEL_BUDGET: f32 = 3.2e6;
/// Above this device pixel ratio the screen's own pixels are small enough.
const SS_DPI_LIMIT: f32 = 1.75;
/// Largest render target edge we ask for (safe on every WebGL device).
const MAX_EDGE: f32 = 4096.0;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    On,
    Off,
    /// Left half raw, right half processed: for A/B stills.
    Split,
}

struct Targets {
    /// Physical canvas size and supersampling these were built for.
    key: (u32, u32, u32),
    ss: f32,
    scene: RenderTarget,
    /// Plain half-resolution copy of the scene (sun shafts read it).
    half: RenderTarget,
    /// Bloom pyramid, ping-pong pairs at 1/2, 1/4 and 1/8 logical size.
    b0: [RenderTarget; 2],
    b1: [RenderTarget; 2],
    b2: [RenderTarget; 2],
    rays: RenderTarget,
}

struct Mats {
    down: Material,
    blur: Material,
    rays: Option<Material>,
    composite: Material,
}

struct Post {
    mode: Mode,
    mats: Option<Mats>,
    /// The world pass's sprite material: same as macroquad's default but
    /// with full-precision texture coordinates (no wobble on big strips on
    /// mobile GPUs, where `lowp` really is low).
    sprite: Option<Material>,
    targets: Option<Targets>,
    capturing: bool,
    /// Sun this frame: screen centre, radius (px), tint, visibility.
    sun: Option<(Vec2, f32, Color, f32)>,
    tried: bool,
    /// Supersampling ceiling; lowered step by step if frames run long.
    ss_cap: f32,
    /// Accumulated seconds of over-budget frames (net of good ones).
    #[cfg_attr(feature = "devkeys", allow(dead_code))]
    slow: f32,
}

thread_local! {
    static POST: RefCell<Post> = RefCell::new(Post {
        mode: if POST_DEFAULT { Mode::On } else { Mode::Off },
        mats: None,
        sprite: None,
        targets: None,
        capturing: false,
        sun: None,
        tried: false,
        ss_cap: SS_MAX,
        slow: 0.0,
    });
}

/// Report the sun for this frame (screen position and radius in logical
/// pixels). Drives the light shafts and a warmer grade around it.
pub fn set_sun(center: Vec2, radius: f32, tint: Color, visible: f32) {
    POST.with(|p| p.borrow_mut().sun = Some((center, radius, tint, visible)));
}

/// Aerial perspective: valley mist over the feet of the far layers
/// (mountains and the downtown skyline), in the colour of the horizon sky,
/// so they sit back behind the hills. Drawn between the far and near layers
/// and only when post is on, so the A/B split stays honest.
pub fn haze(cam: &crate::Cam) {
    if !POST.with(|p| p.borrow().capturing) {
        return;
    }
    let dusk = crate::draw::dusk_factor(cam);
    let space = crate::draw::space_factor(cam);
    let strength = 0.55 * (1.0 - space);
    if strength <= 0.01 {
        return;
    }
    let (sw, sh) = (screen_width(), screen_height());
    let zz = cam.zoom.powf(0.4);
    let horizon = cam.to_screen_parallax(0.0, 0.0, crate::draw::para_y(0.18)).y;
    let top = horizon - 230.0 * zz;
    if top > sh {
        return;
    }
    let ice = Color::new(0.84, 0.91, 1.0, 1.0);
    let peach = Color::new(1.0, 0.80, 0.64, 1.0);
    let violet = Color::new(0.62, 0.52, 0.78, 1.0);
    let c = crate::draw::lerp(crate::draw::lerp(ice, peach, smooth(0.15, 0.6, dusk)), violet, smooth(0.6, 1.0, dusk));
    // (y, alpha) stops: clear sky above the peaks, thickest at the foot.
    let stops = [(top, 0.0), (top + (horizon - top) * 0.45, 0.22), (top + (horizon - top) * 0.8, 0.65), (horizon, 1.0), (horizon + 90.0 * zz, 0.7), (horizon + 220.0 * zz, 0.0)];
    let mut vertices = Vec::with_capacity(stops.len() * 2);
    let mut indices: Vec<u16> = Vec::new();
    for (i, (y, a)) in stops.iter().enumerate() {
        let col: [u8; 4] = Color::new(c.r, c.g, c.b, a * strength).into();
        for x in [0.0, sw] {
            vertices.push(macroquad::models::Vertex { position: vec3(x, *y, 0.0), uv: vec2(0.0, 0.0), color: col, normal: vec4(0.0, 0.0, 1.0, 0.0) });
        }
        if i > 0 {
            let k = (i as u16 - 1) * 2;
            indices.extend_from_slice(&[k, k + 1, k + 2, k + 1, k + 3, k + 2]);
        }
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let k = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// Start the frame's world pass. Returns true when the world goes through
/// post-processing (draw into the scene buffer, then call [`end`]).
pub fn begin() -> bool {
    POST.with(|p| {
        let mut p = p.borrow_mut();
        p.sun = None;
        p.capturing = false;
        #[cfg(feature = "devkeys")]
        if is_key_pressed(KeyCode::O) {
            p.mode = match p.mode {
                Mode::On => Mode::Off,
                Mode::Off => Mode::Split,
                Mode::Split => Mode::On,
            };
        }
        if !p.tried {
            p.tried = true;
            p.mats = build_materials();
            p.sprite = build_sprite_material();
        }
        if p.mode == Mode::Off || p.mats.is_none() {
            return false;
        }
        // Frame-time guard: if frames keep missing ~45 fps, supersample
        // less (dev builds keep full quality for reproducible stills).
        #[cfg(not(feature = "devkeys"))]
        {
            let dt = get_frame_time();
            p.slow = if dt > 1.0 / 45.0 { p.slow + dt } else { (p.slow - dt).max(0.0) };
            if p.slow > 3.0 && p.ss_cap > 1.0 {
                p.ss_cap = (p.ss_cap - 0.5).max(1.0);
                p.slow = 0.0;
            }
        }
        let dpi = screen_dpi_scale().max(0.5);
        let (pw, ph) = ((screen_width() * dpi).round().max(1.0), (screen_height() * dpi).round().max(1.0));
        let ss = if dpi >= SS_DPI_LIMIT { 1.0 } else { (SS_PIXEL_BUDGET / (pw * ph)).sqrt().clamp(1.0, p.ss_cap) };
        let ss = ss.min(MAX_EDGE / pw).min(MAX_EDGE / ph).max(0.25);
        let key = (pw as u32, ph as u32, (ss * 100.0) as u32);
        if p.targets.as_ref().map(|t| t.key) != Some(key) {
            p.targets = Some(build_targets(key, ss));
        }
        let t = p.targets.as_ref().unwrap();
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
        cam.render_target = Some(t.scene.clone());
        set_camera(&cam);
        clear_background(Color::new(0.5, 0.7, 0.9, 1.0));
        if let Some(m) = &p.sprite {
            gl_use_material(m);
        }
        p.capturing = true;
        true
    })
}

/// Finish the world pass: run the post chain and composite onto the canvas.
/// Everything drawn after this (the HUD) lands on the canvas untouched.
pub fn end(cam: &crate::Cam) {
    POST.with(|p| {
        let mut p = p.borrow_mut();
        if !p.capturing {
            return;
        }
        p.capturing = false;
        gl_use_default_material();
        let p = &*p;
        let (Some(m), Some(t)) = (&p.mats, &p.targets) else {
            set_default_camera();
            return;
        };
        let dusk = crate::draw::dusk_factor(cam);
        let space = crate::draw::space_factor(cam);
        let (sw, sh) = (screen_width(), screen_height());

        // Bloom: plain half-res copy, bright pass, then three blurred levels.
        let scene_tex = t.scene.texture.clone();
        pass(&t.half, &m.down, &scene_tex, |mat| {
            mat.set_uniform("Texel", texel(&scene_tex));
            mat.set_uniform("Thresh", [0.0f32, 0.1, 0.0, 0.0]);
        });
        // Daylight snow is near white: keep the threshold high so only the
        // sun, lights, sparkles and fireworks glow; dusk lowers it.
        let thresh = 0.84 - 0.12 * dusk - 0.2 * space;
        let half_tex = t.half.texture.clone();
        pass(&t.b0[0], &m.down, &half_tex, |mat| {
            mat.set_uniform("Texel", texel(&half_tex));
            mat.set_uniform("Thresh", [thresh, 0.12, 1.0, 0.0]);
        });
        blur(&m.blur, &t.b0);
        down(&m.down, &t.b0[0], &t.b1[0]);
        blur(&m.blur, &t.b1);
        down(&m.down, &t.b1[0], &t.b2[0]);
        blur(&m.blur, &t.b2);

        // Sun shafts: march from each pixel towards the sun through the
        // half-res scene; only light near the sun counts, so buildings,
        // clouds and the dog cut dark wedges into the glow.
        let mut rays_k = 0.0;
        let mut sun_uv = [0.5f32, 2.0];
        let mut sun_tint = [1.0f32, 0.9, 0.7];
        let mut sun_r = 90.0;
        if let (Some(rm), Some((c, r, tint, vis))) = (&m.rays, p.sun) {
            // Fade out when the sun leaves the frame.
            let off = ((c.x / sw - 0.5).abs() - 0.5).max(0.0) + ((c.y / sh - 0.5).abs() - 0.5).max(0.0);
            rays_k = vis * (1.0 - off * 3.0).clamp(0.0, 1.0);
            sun_uv = [c.x / sw, 1.0 - c.y / sh];
            sun_tint = [tint.r, tint.g * 0.95, tint.b * 0.85];
            sun_r = r;
            if rays_k > 0.0 {
                let half_tex = t.half.texture.clone();
                pass(&t.rays, rm, &half_tex, |mat| {
                    mat.set_uniform("Sun", [sun_uv[0], sun_uv[1], r / sh, sw / sh]);
                    mat.set_uniform("RayP", [1.0f32, 0.955, 0.85, 0.0]);
                });
            }
        }

        // Composite onto the canvas.
        set_default_camera();
        let split = if p.mode == Mode::Split { 0.5 } else { -1.0 };
        let res = if t.ss > 1.01 { 0.25 / (sw * screen_dpi_scale()) } else { 0.0 };
        m.composite.set_texture("Bloom0", t.b0[0].texture.clone());
        m.composite.set_texture("Bloom1", t.b1[0].texture.clone());
        m.composite.set_texture("Bloom2", t.b2[0].texture.clone());
        m.composite.set_texture("Rays", t.rays.texture.clone());
        m.composite.set_uniform("Resolve", [res, res * sw / sh]);
        // Bloom weights per level, overall strength (up at dusk and in space).
        m.composite.set_uniform("BloomK", [0.55f32, 0.45, 0.30, 0.75 + 0.35 * dusk + 0.4 * space]);
        let rk = 0.75 * rays_k;
        m.composite.set_uniform("RaysTint", [sun_tint[0] * rk, sun_tint[1] * rk, sun_tint[2] * rk, 0.0]);
        // contrast, vibrance, split-tone, dusk warmth
        m.composite.set_uniform("Grade", [0.45f32 + 0.1 * dusk, 0.35, 1.0, dusk * (1.0 - space)]);
        // vignette strength, inner radius, aspect, grain
        m.composite.set_uniform("Vig", [0.30f32, 0.55, sw / sh, 0.035]);
        m.composite.set_uniform("Misc", [(get_time() * 60.0 % 1000.0) as f32, split, space, 0.0]);
        m.composite.set_uniform("SunUv", [sun_uv[0], sun_uv[1], rays_k, sun_r / sh]);
        gl_use_material(&m.composite);
        draw_texture_ex(&t.scene.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(sw, sh)), ..Default::default() });
        gl_use_default_material();
    })
}

fn texel(t: &Texture2D) -> [f32; 2] {
    [1.0 / t.width(), 1.0 / t.height()]
}

/// One full-target pass: `src` through `mat` into `dst`. Uniforms are set
/// after the camera switch, which flushes the previous pass.
fn pass(dst: &RenderTarget, mat: &Material, src: &Texture2D, uniforms: impl FnOnce(&Material)) {
    let (w, h) = (dst.texture.width(), dst.texture.height());
    set_camera(&Camera2D { zoom: vec2(2.0 / w, 2.0 / h), target: vec2(w / 2.0, h / 2.0), render_target: Some(dst.clone()), ..Default::default() });
    uniforms(mat);
    gl_use_material(mat);
    draw_texture_ex(src, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
    gl_use_default_material();
}

fn down(mat: &Material, src: &RenderTarget, dst: &RenderTarget) {
    let tex = src.texture.clone();
    pass(dst, mat, &tex, |m| {
        m.set_uniform("Texel", texel(&tex));
        m.set_uniform("Thresh", [0.0f32, 0.1, 0.0, 0.0]);
    });
}

/// Separable gaussian, horizontal into [1] then vertical back into [0].
fn blur(mat: &Material, rt: &[RenderTarget; 2]) {
    let a = rt[0].texture.clone();
    pass(&rt[1], mat, &a, |m| m.set_uniform("Dir", [1.0 / a.width(), 0.0f32]));
    let b = rt[1].texture.clone();
    pass(&rt[0], mat, &b, |m| m.set_uniform("Dir", [0.0f32, 1.0 / b.height()]));
}

fn target(w: f32, h: f32) -> RenderTarget {
    let rt = render_target(w.round().max(1.0) as u32, h.round().max(1.0) as u32);
    rt.texture.set_filter(FilterMode::Linear);
    rt
}

fn build_targets(key: (u32, u32, u32), ss: f32) -> Targets {
    let (pw, ph) = (key.0 as f32, key.1 as f32);
    // Bloom sizes follow the logical screen: glow does not need retina detail.
    let (lw, lh) = (screen_width().max(16.0), screen_height().max(16.0));
    let pair = |d: f32| [target(lw / d, lh / d), target(lw / d, lh / d)];
    Targets {
        key,
        ss,
        scene: target(pw * ss, ph * ss),
        half: target(lw / 2.0, lh / 2.0),
        b0: pair(2.0),
        b1: pair(4.0),
        b2: pair(8.0),
        rays: target(lw / 4.0, lh / 4.0),
    }
}

// ---------------------------------------------------------------------------
// Shaders (GLSL 100 / WebGL1)
// ---------------------------------------------------------------------------

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying highp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const HEADER: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
varying vec2 uv;
uniform sampler2D Texture;
"#;

/// Mip LOD bias -0.4: trilinear filtering errs soft; this keeps painted
/// outlines crisp while still smoothing shrunken sprites.
const SPRITE_FS: &str = r#"
varying lowp vec4 color;
void main() {
    gl_FragColor = color * texture2D(Texture, uv, -0.4);
}
"#;

/// 4-tap box downsample; with Thresh.z = 1 also a soft-knee bright pass
/// that favours coloured light (sun, lamps, fireworks) over plain white snow.
const DOWN_FS: &str = r#"
uniform vec2 Texel;
uniform vec4 Thresh;
void main() {
    vec3 c = texture2D(Texture, uv + Texel * vec2(-1.0, -1.0)).rgb;
    c += texture2D(Texture, uv + Texel * vec2(1.0, -1.0)).rgb;
    c += texture2D(Texture, uv + Texel * vec2(-1.0, 1.0)).rgb;
    c += texture2D(Texture, uv + Texel * vec2(1.0, 1.0)).rgb;
    c *= 0.25;
    if (Thresh.z > 0.5) {
        float l = max(c.r, max(c.g, c.b));
        float k = Thresh.y;
        float soft = clamp(l - Thresh.x + k, 0.0, 2.0 * k);
        soft = soft * soft / (4.0 * k + 0.0001);
        float w = max(soft, l - Thresh.x) / max(l, 0.0001);
        float sat = l - min(c.r, min(c.g, c.b));
        w *= mix(0.45, 1.0, clamp(sat * 3.0, 0.0, 1.0));
        c *= w;
    }
    gl_FragColor = vec4(c, 1.0);
}
"#;

/// 9-tap gaussian in 5 bilinear fetches.
const BLUR_FS: &str = r#"
uniform vec2 Dir;
void main() {
    vec3 c = texture2D(Texture, uv).rgb * 0.2270270270;
    c += texture2D(Texture, uv + Dir * 1.3846153846).rgb * 0.3162162162;
    c += texture2D(Texture, uv - Dir * 1.3846153846).rgb * 0.3162162162;
    c += texture2D(Texture, uv + Dir * 3.2307692308).rgb * 0.0702702703;
    c += texture2D(Texture, uv - Dir * 3.2307692308).rgb * 0.0702702703;
    gl_FragColor = vec4(c, 1.0);
}
"#;

/// Radial march towards the sun. Sun.xy: sun in uv, Sun.z: radius in uv-y,
/// Sun.w: aspect. RayP.x: gain, y: decay per step, z: march length.
const RAYS_FS: &str = r#"
uniform vec4 Sun;
uniform vec4 RayP;
void main() {
    vec2 stepv = (Sun.xy - uv) * RayP.z / 28.0;
    vec2 p = uv;
    float wgt = 1.0;
    vec3 acc = vec3(0.0);
    for (int i = 0; i < 28; i++) {
        vec3 s = texture2D(Texture, p).rgb;
        vec2 q = (p - Sun.xy) * vec2(Sun.w, 1.0);
        float near = 1.0 - smoothstep(Sun.z * 0.6, Sun.z * 3.0, length(q));
        float lum = dot(s, vec3(0.3, 0.55, 0.15));
        float lit = smoothstep(0.80, 1.0, lum);
        acc += s * (lit * near * wgt);
        wgt *= RayP.y;
        p += stepv;
    }
    gl_FragColor = vec4(acc * (RayP.x / 12.0), 1.0);
}
"#;

const COMPOSITE_FS: &str = r#"
uniform sampler2D Bloom0;
uniform sampler2D Bloom1;
uniform sampler2D Bloom2;
uniform sampler2D Rays;
uniform vec2 Resolve;
uniform vec4 BloomK;
uniform vec4 RaysTint;
uniform vec4 Grade;
uniform vec4 Vig;
uniform vec4 Misc;
uniform vec4 SunUv;

float luma(vec3 c) { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }

void main() {
    vec2 suv = vec2(uv.x, 1.0 - uv.y);
    // Resolve the supersampled scene: rotated 4-tap box.
    vec3 c = texture2D(Texture, suv + vec2(-Resolve.x, -Resolve.y)).rgb;
    c += texture2D(Texture, suv + vec2(Resolve.x, -Resolve.y)).rgb;
    c += texture2D(Texture, suv + vec2(-Resolve.x, Resolve.y)).rgb;
    c += texture2D(Texture, suv + vec2(Resolve.x, Resolve.y)).rgb;
    c *= 0.25;
    if (uv.x < Misc.y) {
        gl_FragColor = vec4(c, 1.0);
        return;
    }

    // Grade first (on the painted image), light on top.
    // Vibrance: lift muted colours more than the already saturated ones.
    float l = luma(c);
    float l0 = l;
    float sat = max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
    c = mix(vec3(l), c, 1.0 + Grade.y * (1.0 - sat));
    c = clamp(c, 0.0, 1.0);
    // Gentle S-curve.
    vec3 s = c * c * (3.0 - 2.0 * c);
    c = mix(c, s, Grade.x);
    // Split tone: cool blue-violet shadows, warm sunlit highlights.
    l = luma(c);
    float sh = 1.0 - smoothstep(0.0, 0.5, l);
    float hi = smoothstep(0.45, 1.0, l);
    c += Grade.z * (sh * vec3(-0.035, -0.008, 0.045) + hi * vec3(0.035, 0.012, -0.040));
    // The afternoon: a peach wash that deepens with distance.
    c = mix(c, c * vec3(1.08, 0.96, 0.84), Grade.w * 0.6);
    // Sunlight across the frame: warmer on the sun's side, cooler away.
    vec2 ds = (uv - vec2(SunUv.x, 1.0 - SunUv.y)) * vec2(Vig.z, 1.0);
    float d = length(ds);
    float toward = SunUv.z * (1.0 - smoothstep(0.0, 1.6, d));
    c *= mix(vec3(0.97, 0.985, 1.03), vec3(1.07, 1.03, 0.93), toward);

    // Light: bloom pyramid and sun shafts, screen-blended so nothing clips.
    vec3 b = texture2D(Bloom0, suv).rgb * BloomK.x;
    b += texture2D(Bloom1, suv).rgb * BloomK.y;
    b += texture2D(Bloom2, suv).rgb * BloomK.z;
    b *= BloomK.w;
    b += texture2D(Rays, suv).rgb * RaysTint.rgb;
    // The sun's halo: a tight glare and a wide warm bloom (SunUv.w: radius).
    float r = max(SunUv.w, 0.001);
    // Objects in front of the sun keep their colour: the glare veils the
    // bright sky far more than dark or saturated sprites.
    float skyish = mix(0.3, 1.0, smoothstep(0.55, 0.9, l0) * (1.0 - smoothstep(0.25, 0.5, sat)));
    b += SunUv.z * skyish * (0.5 * exp(-d * d / (r * r * 2.2)) + 0.22 * exp(-d / (r * 2.6))) * vec3(1.0, 0.86, 0.6);
    c = 1.0 - (1.0 - c) * (1.0 - clamp(b, 0.0, 1.0));

    // Soft vignette, a little cooler in the corners.
    vec2 q = (uv - 0.5) * vec2(2.0, 1.7);
    float v = smoothstep(Vig.y, Vig.y + 0.9, length(q));
    c *= 1.0 - Vig.x * v * v;
    c = mix(c, c * vec3(0.92, 0.95, 1.06), v * 0.6);

    // Fine animated grain, strongest in the mid-tones; also dithers the sky.
    vec2 g = gl_FragCoord.xy + vec2(Misc.x * 5.588238, Misc.x * 3.8571);
    float n = fract(52.9829189 * fract(dot(g, vec2(0.06711056, 0.00583715)))) - 0.5;
    l = luma(c);
    c += n * Vig.w * (0.35 + l * (1.0 - l) * 2.6);

    gl_FragColor = vec4(c, 1.0);
}
"#;

fn fs(body: &str) -> String {
    format!("{HEADER}{body}")
}

fn material(body: &str, uniforms: &[(&str, UniformType)], textures: &[&str], blend: bool) -> Option<Material> {
    let pipeline_params = PipelineParams {
        color_blend: if blend {
            Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha)))
        } else {
            None
        },
        ..Default::default()
    };
    let frag = fs(body);
    match load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: &frag },
        MaterialParams {
            pipeline_params,
            uniforms: uniforms.iter().map(|(n, t)| UniformDesc::new(n, *t)).collect(),
            textures: textures.iter().map(|s| s.to_string()).collect(),
        },
    ) {
        Ok(m) => Some(m),
        Err(e) => {
            warn!("post: shader failed, effect disabled: {:?}", e);
            None
        }
    }
}

fn build_materials() -> Option<Mats> {
    use UniformType::{Float2, Float4};
    let down = material(DOWN_FS, &[("Texel", Float2), ("Thresh", Float4)], &[], false)?;
    let blur = material(BLUR_FS, &[("Dir", Float2)], &[], false)?;
    let composite = material(
        COMPOSITE_FS,
        &[
            ("Resolve", Float2),
            ("BloomK", Float4),
            ("RaysTint", Float4),
            ("Grade", Float4),
            ("Vig", Float4),
            ("Misc", Float4),
            ("SunUv", Float4),
        ],
        &["Bloom0", "Bloom1", "Bloom2", "Rays"],
        false,
    )?;
    let rays = material(RAYS_FS, &[("Sun", Float4), ("RayP", Float4)], &[], false);
    Some(Mats { down, blur, rays, composite })
}

fn build_sprite_material() -> Option<Material> {
    material(SPRITE_FS, &[], &[], true)
}

// ---------------------------------------------------------------------------
// Sprite loading: alpha bleed + mipmaps
// ---------------------------------------------------------------------------

/// Load a sprite for drawing at any scale: transparent pixels take the
/// colour of the nearest painted ones (no dark fringes when filtered), and
/// the texture gets trilinear mipmaps where the GL allows it (WebGL1 only
/// for power-of-two sizes), so shrunken sprites stay smooth instead of
/// sparkling.
pub async fn load_sprite(path: &str) -> Result<Texture2D, macroquad::Error> {
    let bytes = load_file(path).await?;
    let mut img = Image::from_file_with_format(&bytes, None)?;
    bleed(&mut img);
    let t = Texture2D::from_image(&img);
    t.set_filter(FilterMode::Linear);
    mipmap(&t);
    Ok(t)
}

/// Trilinear mipmaps for `t` when the GL supports them for its size.
pub fn mipmap(t: &Texture2D) {
    let (w, h) = (t.width() as u32, t.height() as u32);
    let pot = w.is_power_of_two() && h.is_power_of_two();
    let mut gl = unsafe { get_internal_gl() };
    let info = gl.quad_context.info();
    let webgl1 = cfg!(target_arch = "wasm32") && !info.glsl_support.v300es;
    if webgl1 && !pot {
        return;
    }
    gl.flush();
    let id = t.raw_miniquad_id();
    gl.quad_context.texture_generate_mipmaps(id);
    gl.quad_context.texture_set_min_filter(id, FilterMode::Linear, MipmapFilterMode::Linear);
}

/// Push-pull alpha bleed: fully transparent pixels get the alpha-weighted
/// colour of the nearest coverage, found through a coverage pyramid.
pub fn bleed(img: &mut Image) {
    let (w, h) = (img.width as usize, img.height as usize);
    if w == 0 || h == 0 || !img.bytes.chunks_exact(4).any(|p| p[3] == 0) {
        return;
    }
    // levels[k]: (w, h, rgba) with straight colour and coverage in a.
    let mut levels: Vec<(usize, usize, Vec<[u8; 4]>)> = Vec::new();
    {
        let (pw, ph) = (w.div_ceil(2), h.div_ceil(2));
        let mut out = vec![[0u8; 4]; pw * ph];
        for y in 0..ph {
            for x in 0..pw {
                let mut acc = [0u32; 4];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = (2 * x + dx, 2 * y + dy);
                    if sx < w && sy < h {
                        let i = (sy * w + sx) * 4;
                        let a = img.bytes[i + 3] as u32;
                        acc[0] += img.bytes[i] as u32 * a;
                        acc[1] += img.bytes[i + 1] as u32 * a;
                        acc[2] += img.bytes[i + 2] as u32 * a;
                        acc[3] += a;
                    }
                }
                out[y * pw + x] = pack(acc);
            }
        }
        levels.push((pw, ph, out));
    }
    while {
        let (lw, lh, _) = levels.last().unwrap();
        *lw > 1 || *lh > 1
    } {
        let (lw, lh, src) = levels.last().unwrap();
        let (lw, lh) = (*lw, *lh);
        let (pw, ph) = (lw.div_ceil(2), lh.div_ceil(2));
        let mut out = vec![[0u8; 4]; pw * ph];
        for y in 0..ph {
            for x in 0..pw {
                let mut acc = [0u32; 4];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = (2 * x + dx, 2 * y + dy);
                    if sx < lw && sy < lh {
                        let p = src[sy * lw + sx];
                        let a = p[3] as u32;
                        acc[0] += p[0] as u32 * a;
                        acc[1] += p[1] as u32 * a;
                        acc[2] += p[2] as u32 * a;
                        acc[3] += a;
                    }
                }
                out[y * pw + x] = pack(acc);
            }
        }
        levels.push((pw, ph, out));
    }
    // Pull: uncovered texels inherit from their parent, coarse to fine.
    for k in (0..levels.len() - 1).rev() {
        let (upper, lower) = levels.split_at_mut(k + 1);
        let (lw, lh, dst) = &mut upper[k];
        let (pw, _, src) = &lower[0];
        for y in 0..*lh {
            for x in 0..*lw {
                let d = &mut dst[y * *lw + x];
                if d[3] == 0 {
                    let s = src[(y / 2) * *pw + x / 2];
                    *d = [s[0], s[1], s[2], 0];
                }
            }
        }
    }
    let (l0w, _, l0) = &levels[0];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            if img.bytes[i + 3] == 0 {
                let s = l0[(y / 2) * l0w + x / 2];
                img.bytes[i] = s[0];
                img.bytes[i + 1] = s[1];
                img.bytes[i + 2] = s[2];
            }
        }
    }
}

/// Alpha-weighted colour sums into a texel: colour = sum / coverage, and
/// any coverage at all keeps a non-zero alpha so it is not overwritten.
fn pack(acc: [u32; 4]) -> [u8; 4] {
    if acc[3] == 0 {
        return [0, 0, 0, 0];
    }
    [(acc[0] / acc[3]) as u8, (acc[1] / acc[3]) as u8, (acc[2] / acc[3]) as u8, acc[3].div_ceil(4).clamp(1, 255) as u8]
}

/// Window platform settings: ask for WebGL2 (mipmaps for any sprite size);
/// `web/mq_js_bundle.js` falls back to WebGL1 when the browser has no WebGL2.
pub fn platform() -> macroquad::miniquad::conf::Platform {
    macroquad::miniquad::conf::Platform { webgl_version: macroquad::miniquad::conf::WebGLVersion::WebGL2, ..Default::default() }
}
