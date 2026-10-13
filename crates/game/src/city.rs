//! Calgary under the flight path: the districts the dog flies over, from
//! Inglewood next to the launch building out to the prairie. Everything is
//! data: `packs/<pack>/city.json` (written by tools/gen_city.py) lists the
//! districts, their facades, landmarks and street props with sizes in
//! metres. The street is laid out once at startup and only the visible
//! slice is drawn. A pack without a city.json keeps the open countryside.

use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::draw::{cheering, daylight, hash, height_haze, lerp, mul, para_y, HAZE, OUTLINE};
use crate::sprites::{blit_pivot, blit_standing};
use crate::{Cam, PPM};

/// Buildings stand a little behind the snowbank lip of the ground tile, props
/// on the sidewalk in front of them.
const BUILDING_LIFT: f32 = 0.6;
const PROP_LIFT: f32 = 0.45;
/// Street props are tried every PROP_SLOT metres, PROP_DENSITY of the slots get one.
const PROP_SLOT: f32 = 3.2;
const PROP_DENSITY: f32 = 0.68;
/// One car in each lane about every this many metres of street.
const CAR_EVERY: f32 = 38.0;
/// The street is laid out this far; nobody flies further.
const LAYOUT_END: f32 = 6000.0;
/// Widest sprite in the layout (the Saddledome), for the visible-slice search.
const MAX_HALF_W: f32 = 40.0;

#[derive(Deserialize)]
struct Facade {
    sprite: String,
    w: f32,
    /// Where chimney smoke leaves the sprite, as fractions of its width and height.
    #[serde(default)]
    smoke: Option<(f32, f32)>,
}

/// A car driving along the street.
#[derive(Deserialize)]
struct CarDef {
    sprite: String,
    w: f32,
}

#[derive(Deserialize)]
struct PropDef {
    sprite: String,
    w: f32,
    weight: f32,
}

#[derive(Deserialize)]
struct Mark {
    sprite: String,
    x: f32,
    w: f32,
    #[serde(default)]
    lift: f32,
    /// Repeat every this many metres (0: once).
    #[serde(default)]
    every: f32,
}

#[derive(Deserialize)]
struct TrainDef {
    sprite: String,
    w: f32,
    speed: f32,
}

#[derive(Deserialize)]
struct SkywalkDef {
    sprite: String,
    lift: f32,
}

#[derive(Deserialize)]
struct PumpjackDef {
    base: String,
    beam: String,
    w: f32,
    every: f32,
    from: f32,
}

#[derive(Deserialize)]
struct SkylineDef {
    sprite: String,
    /// World distance at which the skyline is centred on screen.
    x: f32,
    factor: f32,
    /// On-screen height in pixels at zoom 1.
    height: f32,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Ground {
    Street,
    River,
    Snow,
}

#[derive(Deserialize)]
pub struct District {
    pub name: String,
    pub street: String,
    pub from: f32,
    pub to: f32,
    pub ground: Ground,
    /// How the results card says where the dog came down ("on the Red Mile").
    #[serde(default)]
    pub landed: String,
    #[serde(default)]
    facades: Vec<Facade>,
    #[serde(default)]
    props: Vec<PropDef>,
    #[serde(default)]
    landmarks: Vec<Mark>,
    #[serde(default)]
    train: Option<TrainDef>,
    #[serde(default)]
    skywalk: Option<SkywalkDef>,
    #[serde(default)]
    pumpjacks: Option<PumpjackDef>,
    /// Space between neighbouring facades, min and max metres.
    #[serde(default = "default_gap")]
    gap: (f32, f32),
}

fn default_gap() -> (f32, f32) {
    (0.3, 1.7)
}

#[derive(Deserialize)]
struct CityDef {
    skyline: Option<SkylineDef>,
    /// The hazy second rank of towers behind the skyline.
    #[serde(default)]
    skyline_far: Option<SkylineDef>,
    /// Cars that drive the city streets.
    #[serde(default)]
    traffic: Vec<CarDef>,
    districts: Vec<District>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Building,
    Skywalk,
    Prop,
    /// Hops and waves when the dog goes by.
    Person,
    /// Startled into a hop when the dog goes by.
    Critter,
}

/// One sprite placed on the street; `x` is the centre, in metres.
struct Placed {
    tex: usize,
    x: f32,
    w: f32,
    lift: f32,
    flip: bool,
    kind: Kind,
    seed: i32,
    smoke: Option<(f32, f32)>,
}

struct Pumpjack {
    base: usize,
    beam: usize,
    x: f32,
    w: f32,
}

pub struct City {
    pub districts: Vec<District>,
    skyline: Option<(usize, SkylineDef)>,
    skyline_far: Option<(usize, SkylineDef)>,
    textures: Vec<Texture2D>,
    /// Facades and landmarks, sorted by x.
    back: Vec<Placed>,
    /// +15 skywalks across the downtown avenues.
    skywalks: Vec<Placed>,
    /// Sidewalk props, people and critters, sorted by x.
    front: Vec<Placed>,
    pumpjacks: Vec<Pumpjack>,
    /// Per district, the CTrain texture if it has a train.
    train_tex: Vec<Option<usize>>,
    /// (texture, width m) of the cars in the traffic.
    cars: Vec<(usize, f32)>,
    /// The soft puff the chimney smoke is painted with, if the pack has one.
    puff: Option<Texture2D>,
}

impl City {
    /// Reads `packs/<pack>/city.json` and the sprites it names. `None` when
    /// the pack has no city.
    pub async fn load(pack_dir: &str, sprite_dir: &str) -> Option<City> {
        let raw = load_string(&format!("{pack_dir}/city.json")).await.ok()?;
        let def: CityDef = match serde_json::from_str(&raw) {
            Ok(d) => d,
            Err(e) => {
                warn!("{pack_dir}/city.json: {e}");
                return None;
            }
        };
        let mut names: Vec<String> = Vec::new();
        let mut want = |n: &str| {
            if !names.iter().any(|m| m == n) {
                names.push(n.to_owned());
            }
        };
        for sk in [&def.skyline, &def.skyline_far].into_iter().flatten() {
            want(&sk.sprite);
        }
        def.traffic.iter().for_each(|c| want(&c.sprite));
        for d in &def.districts {
            d.facades.iter().for_each(|f| want(&f.sprite));
            d.props.iter().for_each(|p| want(&p.sprite));
            d.landmarks.iter().for_each(|m| want(&m.sprite));
            if let Some(t) = &d.train {
                want(&t.sprite);
            }
            if let Some(s) = &d.skywalk {
                want(&s.sprite);
            }
            if let Some(p) = &d.pumpjacks {
                want(&p.base);
                want(&p.beam);
            }
        }
        let mut textures = Vec::with_capacity(names.len());
        for n in &names {
            let path = format!("{sprite_dir}/{n}.png");
            let t = crate::post::load_sprite(&path).await.unwrap_or_else(|e| panic!("missing city sprite {path}: {e:?}"));
            textures.push(t);
        }
        let index: HashMap<String, usize> = names.into_iter().enumerate().map(|(i, n)| (n, i)).collect();
        let mut city = City::build(def, textures, &index);
        city.puff = load_texture(&format!("{sprite_dir}/puff.png")).await.ok();
        Some(city)
    }

    fn build(def: CityDef, textures: Vec<Texture2D>, index: &HashMap<String, usize>) -> City {
        let idx = |n: &str| index[n];
        let (mut back, mut skywalks, mut front, mut pumpjacks) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (di, d) in def.districts.iter().enumerate() {
            let salt = di as i32 * 7919;
            let end = d.to.min(LAYOUT_END);
            // Landmarks first; facades flow around them.
            let mut spans: Vec<(f32, f32)> = Vec::new();
            for m in &d.landmarks {
                let mut x = m.x;
                while x + m.w <= end {
                    back.push(Placed { tex: idx(&m.sprite), x: x + m.w / 2.0, w: m.w, lift: BUILDING_LIFT + m.lift, flip: false, kind: Kind::Building, seed: 0, smoke: None });
                    spans.push((x - 1.0, x + m.w + 1.0));
                    if m.every <= 0.0 {
                        break;
                    }
                    x += m.every;
                }
            }
            if let Some(p) = &d.pumpjacks {
                let mut x = p.from;
                while x + p.w <= end {
                    if !spans.iter().any(|(a, b)| x + p.w > *a && x < *b) {
                        pumpjacks.push(Pumpjack { base: idx(&p.base), beam: idx(&p.beam), x: x + p.w / 2.0, w: p.w });
                        spans.push((x - 1.0, x + p.w + 1.0));
                    }
                    x += p.every;
                }
            }
            if !d.facades.is_empty() {
                let n = d.facades.len();
                let mut x = d.from + 1.0;
                let mut k = 0;
                let mut prev = usize::MAX;
                let mut pending: Option<Placed> = None;
                while x < end - 3.0 {
                    let mut i = (hash(k + salt, 501) * n as f32) as usize % n;
                    if i == prev && n > 1 {
                        i = (i + 1) % n;
                    }
                    let f = &d.facades[i];
                    if x + f.w > end {
                        break;
                    }
                    if let Some(&(_, b)) = spans.iter().find(|(a, b)| x + f.w > *a && x < *b) {
                        // Step past the landmark; a skywalk never leads into one.
                        x = b + 0.5;
                        pending = None;
                        k += 1;
                        continue;
                    }
                    if let Some(sk) = pending.take() {
                        skywalks.push(sk);
                    }
                    back.push(Placed { tex: idx(&f.sprite), x: x + f.w / 2.0, w: f.w, lift: BUILDING_LIFT, flip: false, kind: Kind::Building, seed: k, smoke: f.smoke });
                    // Downtown, every other block ends at an avenue with a +15 across it.
                    let crossing = d.skywalk.is_some() && hash(k + salt, 503) < 0.5;
                    let gap = if crossing { 7.0 } else { d.gap.0 + hash(k + salt, 502) * (d.gap.1 - d.gap.0) };
                    if let (true, Some(sk)) = (crossing, &d.skywalk) {
                        pending = Some(Placed { tex: idx(&sk.sprite), x: x + f.w + gap / 2.0, w: gap + 1.2, lift: sk.lift, flip: false, kind: Kind::Skywalk, seed: k, smoke: None });
                    }
                    x += f.w + gap;
                    prev = i;
                    k += 1;
                }
            }
            // Sidewalk props, chosen by weight per slot.
            let total: f32 = d.props.iter().map(|p| p.weight).sum();
            if total > 0.0 {
                let mut k = (d.from / PROP_SLOT).ceil() as i32;
                loop {
                    let sx = k as f32 * PROP_SLOT;
                    if sx > end - 1.0 {
                        break;
                    }
                    if hash(k, 511) < PROP_DENSITY {
                        let mut pick = hash(k, 512) * total;
                        let p = d.props.iter().find(|p| {
                            pick -= p.weight;
                            pick <= 0.0
                        });
                        if let Some(p) = p {
                            let kind = if p.sprite.starts_with("ped_") {
                                Kind::Person
                            } else if ["magpie", "jackrabbit", "deer", "cow"].contains(&p.sprite.as_str()) {
                                Kind::Critter
                            } else {
                                Kind::Prop
                            };
                            front.push(Placed { tex: idx(&p.sprite), x: sx + hash(k, 513) * 1.6, w: p.w, lift: PROP_LIFT, flip: hash(k, 514) > 0.5, kind, seed: k, smoke: None });
                        }
                    }
                    k += 1;
                }
            }
        }
        back.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        front.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        let skyline = def.skyline.map(|s| (idx(&s.sprite), s));
        let skyline_far = def.skyline_far.map(|s| (idx(&s.sprite), s));
        let train_tex = def.districts.iter().map(|d| d.train.as_ref().map(|t| idx(&t.sprite))).collect();
        let cars = def.traffic.iter().map(|c| (idx(&c.sprite), c.w)).collect();
        City { districts: def.districts, skyline, skyline_far, textures, back, skywalks, front, pumpjacks, train_tex, cars, puff: None }
    }

    /// The district under world distance `x`, if any (the launch yard has none).
    pub fn district_index(&self, x: f32) -> Option<usize> {
        self.districts.iter().position(|d| x >= d.from && x < d.to)
    }

    pub fn ground_at(&self, x: f32) -> Option<Ground> {
        self.district_index(x).map(|i| self.districts[i].ground)
    }

    /// Facades and landmarks catch the evening light like the snow does.
    fn light(cam: &Cam) -> Color {
        lerp(WHITE, daylight(cam), 0.7)
    }

    /// Downtown on the horizon in two depths: the hazy far rank of towers,
    /// then the landmark rank, each one wide sprite on its own slow parallax
    /// layer, washed towards the sky with distance like the mountains.
    pub fn skyline(&self, cam: &Cam) {
        if let Some((tex, sk)) = &self.skyline_far {
            self.skyline_layer(cam, *tex, sk, Color::new(0.84, 0.89, 1.0, 1.0), 0.35);
        }
        if let Some((tex, sk)) = &self.skyline {
            self.skyline_layer(cam, *tex, sk, Color::new(0.93, 0.95, 1.0, 1.0), 0.0);
        }
    }

    fn skyline_layer(&self, cam: &Cam, tex: usize, sk: &SkylineDef, base: Color, haze: f32) {
        let tex = &self.textures[tex];
        let sw = screen_width();
        let sh = screen_height();
        let zz = cam.zoom.powf(0.4);
        let h = sk.height * zz;
        let w = h * tex.width() / tex.height();
        let cx = sw * 0.5 + (sk.x - cam.x) * sk.factor * PPM * zz;
        if cx + w * 0.5 < 0.0 || cx - w * 0.5 > sw {
            return;
        }
        let bottom = cam.to_screen_parallax(0.0, 0.0, para_y(sk.factor)).y + 6.0 * zz;
        if bottom - h > sh {
            return;
        }
        let light = daylight(cam);
        let near = lerp(mul(base, light), mul(HAZE, light), haze);
        let tint = lerp(near, mul(HAZE, light), height_haze(cam));
        draw_texture_ex(tex, cx - w * 0.5, bottom - h, tint, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
    }

    /// Indices into a sorted placement list whose sprites may be on screen.
    fn visible(list: &[Placed], left: f32, right: f32) -> std::ops::Range<usize> {
        let a = list.partition_point(|p| p.x < left - MAX_HALF_W);
        let b = list.partition_point(|p| p.x <= right + MAX_HALF_W);
        a..b.max(a)
    }

    /// The street behind the flight line: facades, landmarks, skywalks, the
    /// CTrain, pumpjacks, and the people and critters on the sidewalk. Drawn
    /// before the ground so the snowbank lip sits in front of their feet.
    pub fn street(&self, cam: &Cam, dog_x: f32, t: f32) {
        let s = cam.scale();
        let sw = screen_width();
        let gy = cam.to_screen(0.0, 0.0).y;
        if gy < -2.0 * s || gy - 70.0 * s > screen_height() {
            return;
        }
        let left = cam.x - sw * 0.5 / s;
        let right = cam.x + sw * 0.5 / s;
        let light = City::light(cam);
        let foot = |x: f32, lift: f32| {
            let p = cam.to_screen(x, lift);
            vec2(p.x, p.y)
        };
        for p in &self.back[City::visible(&self.back, left, right)] {
            blit_standing(&self.textures[p.tex], foot(p.x, p.lift), p.w * s, p.flip, light);
        }
        for p in &self.back[City::visible(&self.back, left, right)] {
            if let Some((fx, fy)) = p.smoke {
                let tex = &self.textures[p.tex];
                let f = foot(p.x, p.lift);
                let (w, h) = (p.w * s, p.w * s * tex.height() / tex.width());
                self.smoke(vec2(f.x - w * 0.5 + fx * w, f.y - h + fy * h), s, t + p.seed as f32 * 1.37, light);
            }
        }
        for p in &self.skywalks {
            if p.x + p.w < left - 2.0 || p.x - p.w > right + 2.0 {
                continue;
            }
            let tex = &self.textures[p.tex];
            let h = 3.6 * s;
            let at = foot(p.x, p.lift);
            draw_texture_ex(tex, at.x - p.w * s * 0.5, at.y - h, light, DrawTextureParams { dest_size: Some(vec2(p.w * s, h)), ..Default::default() });
        }
        for (d, tex) in self.districts.iter().zip(&self.train_tex) {
            if d.to < left - 30.0 || d.from > right + 30.0 {
                continue;
            }
            if let (Some(tr), Some(tex)) = (&d.train, tex) {
                self.train(cam, d, tr, *tex, t, light);
            }
        }
        for j in &self.pumpjacks {
            if j.x + j.w < left || j.x - j.w > right {
                continue;
            }
            let base = &self.textures[j.base];
            let beam = &self.textures[j.beam];
            let f = foot(j.x, PROP_LIFT);
            let (w, h) = (j.w * s, j.w * s * base.height() / base.width());
            blit_standing(base, f, w, false, light);
            // The bearing on the A-frame (176, 64 of 330 x 240) carries the
            // beam's pivot (176, 60 of 330 x 120); it nods like a drinking bird.
            let at = vec2(f.x - w * 0.5 + w * 176.0 / 330.0, f.y - h + h * 64.0 / 240.0);
            let nod = 0.2 * (t * 1.7 + j.x * 0.37).sin();
            blit_pivot(beam, at, vec2(176.0 / 330.0, 60.0 / 120.0), w, nod, false, light);
        }
        for p in &self.front[City::visible(&self.front, left, right)] {
            let near = ((9.0 - (dog_x - p.x).abs()) / 9.0).clamp(0.0, 1.0);
            // Nobody faces away from a flying dog for long.
            let flip = if near > 0.2 && p.kind == Kind::Person { dog_x < p.x } else { p.flip };
            let tex = &self.textures[p.tex];
            match p.kind {
                // People cheer the dog on; critters hop a little, nervously.
                Kind::Person => cheering(tex, foot(p.x, p.lift), p.w * s, flip, t, p.seed as f32 * 0.731, near, light),
                Kind::Critter => cheering(tex, foot(p.x, p.lift), p.w * s, flip, t, p.seed as f32 * 0.731, near * 0.5, light),
                _ => blit_standing(tex, foot(p.x, p.lift), p.w * s, flip, light),
            }
        }
    }

    /// Wood smoke curling out of a chimney and drifting off with the wind.
    fn smoke(&self, at: Vec2, s: f32, t: f32, light: Color) {
        const N: i32 = 7;
        for k in 0..N {
            let ph = (t * 0.22 + k as f32 / N as f32).fract();
            let rise = ph * 4.2 * s;
            let drift = (ph * ph * 3.4 + 0.25 * (t * 0.9 + k as f32 * 1.7).sin() * ph) * s;
            let r = (0.4 + ph * 1.2) * s;
            let a = (1.0 - ph).powf(1.2) * (ph * 8.0).min(1.0);
            let c = mul(Color::new(0.93, 0.94, 0.98, a), light);
            let (x, y) = (at.x + drift, at.y - rise);
            match &self.puff {
                Some(tex) => {
                    for (dx, dy, k2) in [(0.0, 0.0, 1.0), (-0.35, 0.2, 0.75), (0.3, 0.25, 0.7)] {
                        let rr = r * k2;
                        draw_texture_ex(tex, x + dx * r - rr * 1.3, y + dy * r - rr * 1.3, c, DrawTextureParams { dest_size: Some(vec2(rr * 2.6, rr * 2.6)), rotation: k as f32 * 1.9 + ph + dx, ..Default::default() });
                    }
                }
                None => {
                    draw_circle(x, y, r, c);
                    draw_circle(x - r * 0.3, y - r * 0.3, r * 0.6, Color::new(1.0, 1.0, 1.0, a * 0.5));
                }
            }
        }
    }

    /// Traffic on a street block: cars in both lanes, block heaters
    /// unplugged, the near lane heading west and the far lane east.
    fn traffic(&self, cam: &Cam, d: &District, top: f32, t: f32, light: Color) {
        if self.cars.is_empty() {
            return;
        }
        let s = cam.scale();
        let len = d.to.min(LAYOUT_END) - d.from;
        let n = (len / CAR_EVERY).floor().max(1.0) as i32;
        let salt = (d.from * 0.37) as i32;
        for lane in 0..2 {
            for k in 0..n {
                let (tex, w) = self.cars[(hash(k + salt, 531 + lane) * self.cars.len() as f32) as usize % self.cars.len()];
                let speed = 5.0 + hash(k + salt, 533 + lane) * 4.0;
                let dir = if lane == 0 { -1.0 } else { 1.0 };
                let span = len + w;
                let u = (k as f32 / n as f32 * span + hash(k + salt, 535) * 9.0 + dir * t * speed).rem_euclid(span);
                let x = d.from - w * 0.5 + u;
                // Cars only show inside the block; they turn off at the ends.
                if x - w * 0.5 < d.from + 1.0 || x + w * 0.5 > d.to - 1.0 {
                    continue;
                }
                let p = cam.to_screen(x, 0.0);
                let y = top + if lane == 0 { 1.25 } else { 2.45 } * s;
                let bob = (t * 9.0 + k as f32).sin() * 0.03 * s;
                draw_ellipse(p.x, y + 0.02 * s, w * 0.45 * s, 0.22 * s, 0.0, Color::new(0.12, 0.14, 0.3, 0.3));
                blit_standing(&self.textures[tex], vec2(p.x, y + bob), w * s, dir < 0.0, light);
            }
        }
    }

    /// The CTrain runs along 7 Av under its overhead wire, shuttling end to
    /// end through the district (both ends of the car are cabs).
    fn train(&self, cam: &Cam, d: &District, tr: &TrainDef, tex: usize, t: f32, light: Color) {
        let tex = &self.textures[tex];
        let s = cam.scale();
        let span = (d.to - d.from - tr.w - 4.0).max(1.0);
        let run = (t * tr.speed) % (2.0 * span);
        let (u, back) = if run < span { (run, false) } else { (2.0 * span - run, true) };
        let x = d.from + 2.0 + tr.w * 0.5 + u;
        // Catenary: poles every 30 m and a wire at 6.2 m.
        let wire = cam.to_screen(0.0, 6.2).y;
        let (a, b) = (cam.to_screen(d.from, 0.0).x, cam.to_screen(d.to, 0.0).x);
        draw_line(a, wire, b, wire, 0.06 * s, OUTLINE);
        let mut px = (d.from / 30.0).ceil() * 30.0;
        while px < d.to {
            let p = cam.to_screen(px, BUILDING_LIFT);
            draw_line(p.x, p.y, p.x, wire - 0.2 * s, 0.16 * s, Color::new(0.35, 0.38, 0.45, 1.0));
            draw_line(p.x, wire, p.x + 0.8 * s, wire, 0.08 * s, Color::new(0.35, 0.38, 0.45, 1.0));
            px += 30.0;
        }
        let p = cam.to_screen(x, BUILDING_LIFT - 0.1);
        blit_standing(tex, vec2(p.x, p.y), tr.w * s, back, light);
    }

    /// The street surface over the ground tiles: asphalt with a centre line
    /// on the city blocks, the frozen Bow under the Peace Bridge.
    pub fn road(&self, cam: &Cam) {
        let s = cam.scale();
        let sw = screen_width();
        let gy = cam.to_screen(0.0, 0.0).y;
        let left = cam.x - sw * 0.5 / s - 2.0;
        let right = cam.x + sw * 0.5 / s + 2.0;
        let light = City::light(cam);
        for d in &self.districts {
            if d.to < left || d.from > right {
                continue;
            }
            let x0 = cam.to_screen(d.from.max(left), 0.0).x;
            let x1 = cam.to_screen(d.to.min(right), 0.0).x;
            match d.ground {
                Ground::Street => {
                    let top = gy + 0.62 * s;
                    let h = 2.6 * s;
                    draw_rectangle(x0, top, x1 - x0, h, mul(Color::new(0.33, 0.35, 0.45, 1.0), light));
                    // Slush along the curb and a lighter wheel path.
                    draw_rectangle(x0, top, x1 - x0, 0.22 * s, mul(Color::new(0.80, 0.84, 0.94, 1.0), light));
                    draw_rectangle(x0, top + 0.7 * s, x1 - x0, 0.5 * s, mul(Color::new(0.38, 0.40, 0.50, 1.0), light));
                    // Dashed yellow centre line in world space so it scrolls.
                    let cy = top + 1.5 * s;
                    let first = (d.from.max(left) / 4.0).floor() as i32;
                    let last = (d.to.min(right) / 4.0).ceil() as i32;
                    for k in first..=last {
                        let a = (k as f32 * 4.0).max(d.from);
                        let b = (k as f32 * 4.0 + 2.2).min(d.to);
                        if b <= a {
                            continue;
                        }
                        let (pa, pb) = (cam.to_screen(a, 0.0).x, cam.to_screen(b, 0.0).x);
                        draw_rectangle(pa, cy, pb - pa, 0.16 * s, mul(Color::new(1.0, 0.82, 0.25, 1.0), light));
                    }
                    draw_rectangle(x0, top + h - 0.12 * s, x1 - x0, 0.12 * s, mul(Color::new(0.24, 0.25, 0.34, 1.0), light));
                    self.traffic(cam, d, top, get_time() as f32, light);
                    // Where the street meets open snow it ends under a
                    // ploughed snowbank instead of a hard cut.
                    for (bx, other) in [(d.from, d.from - 0.5), (d.to, d.to + 0.5)] {
                        if self.ground_at(other) == Some(Ground::Street) {
                            continue;
                        }
                        let p = cam.to_screen(bx, 0.0);
                        let dir = if bx == d.from { 1.0 } else { -1.0 };
                        for (k, (rx, ry, dy)) in [(3.6, 1.7, 1.5), (2.4, 1.2, 0.6), (2.0, 1.0, 2.3)].iter().enumerate() {
                            let cx = p.x + dir * (k as f32 * 0.9 - 0.4) * s;
                            let c = mul(Color::new(0.96, 0.97, 1.0, 1.0), light);
                            draw_ellipse(cx, top + dy * s, rx * s, ry * s, 0.0, mul(Color::new(0.62, 0.72, 0.88, 1.0), light));
                            draw_ellipse(cx - 0.12 * s, top + dy * s - 0.14 * s, rx * s * 0.96, ry * s * 0.86, 0.0, c);
                        }
                    }
                }
                Ground::River => {
                    // Snow-dusted ice from bank to bank, a skating oval swept
                    // clean in the middle, and snowy banks at both ends.
                    let top = gy - 0.25 * s;
                    let h = 3.4 * s;
                    draw_rectangle(x0, top, x1 - x0, h, mul(Color::new(0.70, 0.86, 0.97, 1.0), light));
                    draw_rectangle(x0, top, x1 - x0, 0.12 * s, OUTLINE);
                    draw_rectangle(x0, top + 0.12 * s, x1 - x0, 0.3 * s, mul(Color::new(0.93, 0.97, 1.0, 1.0), light));
                    let mid = cam.to_screen((d.from + d.to) * 0.5, 0.0).x;
                    draw_ellipse(mid, top + 1.4 * s, 18.0 * s, 0.8 * s, 0.0, mul(Color::new(0.82, 0.93, 1.0, 1.0), light));
                    draw_ellipse_lines(mid, top + 1.4 * s, 18.0 * s, 0.8 * s, 0.0, 0.05 * s, Color::new(0.45, 0.62, 0.85, 0.8));
                    for k in 0..6 {
                        let cx = d.from + 6.0 + hash(k, 521) * (d.to - d.from - 12.0);
                        let p = cam.to_screen(cx, 0.0);
                        let dx = (hash(k, 522) - 0.5) * 2.4 * s;
                        draw_line(p.x - dx, top + 0.5 * s, p.x, top + 1.1 * s, 0.05 * s, Color::new(0.35, 0.55, 0.8, 0.8));
                        draw_line(p.x, top + 1.1 * s, p.x + dx * 0.6, top + 1.7 * s, 0.05 * s, Color::new(0.35, 0.55, 0.8, 0.8));
                    }
                    for (bx, dir) in [(d.from, 1.0), (d.to, -1.0)] {
                        let p = cam.to_screen(bx, 0.0);
                        draw_ellipse(p.x + dir * 1.2 * s, gy - 0.1 * s, 3.0 * s, 0.9 * s, 0.0, mul(Color::new(0.97, 0.98, 1.0, 1.0), light));
                        draw_ellipse_lines(p.x + dir * 1.2 * s, gy - 0.1 * s, 3.0 * s, 0.9 * s, 0.0, 0.08 * s, OUTLINE);
                    }
                }
                Ground::Snow => {}
            }
        }
    }
}
