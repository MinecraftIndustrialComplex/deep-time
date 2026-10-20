// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's deposits overlay (Ben, 2026-10-01, "Deposits layer"): what the page needs to
//! draw a run's deposit sites over the map, and nothing more.
//!
//! It is a view of data a run already keeps. The deposit stage writes `world.sites` and
//! `world.bodies` when it places them; the page reads them as
//!
//! - **live**, from the world in memory, once, by the thread that drives the run, between two
//!   steps, when the first frame that names the sites layer ([`SITES_LAYER`]) is published
//!   (`Workbench::publish`): a clone of a few hundred records, no simulation step involved; and
//! - **bake**, from `sites/sites.dtsr` and `bodies/bodies.dtb` of the finished run's world data
//!   (the same files the game reads), for a run that is done: the baked deposits at the end, and
//!   what a run kept across a restart shows.
//!
//! Nothing is added to the simulation, the keyframes or the bake. The answer
//! (`GET /api/runs/<id>/deposits`) is one JSON object, [`view`]; the events the page already
//! streams carry each site's age and its cause chain, so they are not repeated here.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use deeptime_core::preset::{Preset, PresetBody};
use deeptime_format::bodies::{Body, read_bodies};
use deeptime_format::sites::{Site, read_sites};
use serde_json::{Value, json};

/// The keyframe layer the deposit stage adds when it has placed its sites: the page fetches the
/// overlay when a frame names it.
pub use deeptime_core::deposits::SITES_LAYER;

/// Sites read from the world in memory, between two steps of the run.
pub const LIVE: &str = "live";
/// Sites read from the finished run's world data.
pub const BAKE: &str = "bake";

/// The default horizontal scale of a planet preset (`scale.m_per_block_h`), for a preset that
/// names none (a synthetic deposit world).
const DEFAULT_M_PER_BLOCK_H: f64 = 300.0;

fn round(v: f64, digits: i32) -> f64 {
    let k = 10f64.powi(digits);
    (v * k).round() / k
}

/// Whether a frame message (`frame_message`) names the sites layer.
pub fn names_sites(frame: &Value) -> bool {
    frame["layers"]
        .as_array()
        .is_some_and(|l| l.iter().any(|n| n == SITES_LAYER))
}

/// A site's footprint on the map: the horizontal ellipse of its widest body (semi-axes `a`, `c` in
/// blocks, `yaw` radians from +x towards +z), centred at block (`x`, `z`). A site without bodies
/// gets the ellipse inscribed in its box.
struct Footprint {
    x: f64,
    z: f64,
    a: f64,
    c: f64,
    yaw: f64,
}

fn footprint(site: &Site, bodies: &[&Body]) -> Footprint {
    let widest = bodies.iter().max_by(|p, q| {
        let area = |b: &Body| f64::from(b.axes[0]) * f64::from(b.axes[2]);
        area(p).total_cmp(&area(q))
    });
    if let Some(b) = widest {
        return Footprint {
            x: b.center[0],
            z: b.center[2],
            a: f64::from(b.axes[0]),
            c: f64::from(b.axes[2]),
            yaw: f64::from(b.yaw),
        };
    }
    let [lo, hi] = site.aabb;
    let mid = |k: usize| (f64::from(lo[k]) + f64::from(hi[k])) / 2.0;
    let half = |k: usize| (f64::from(hi[k]) - f64::from(lo[k])) / 2.0 + 0.5;
    Footprint {
        x: mid(0),
        z: mid(2),
        a: half(0),
        c: half(2),
        yaw: 0.0,
    }
}

/// x wrapped into the canonical span [−C/2, C/2).
fn wrap_x(x: f64, circumference: f64) -> f64 {
    x - circumference * ((x + circumference / 2.0) / circumference).floor()
}

/// The overlay's data: every site with where it is, its footprint, what it yields, and the event
/// that formed it. `source` is [`LIVE`] or [`BAKE`].
pub fn view(
    sites: &[Site],
    bodies: &[Body],
    circumference: u32,
    m_per_block_h: f64,
    source: &str,
) -> Value {
    view_in_world(sites, bodies, circumference, m_per_block_h, source, &HashMap::new())
}

/// [`view`] for sites whose `x` and `z` are not the world's blocks: `world` maps a site's id to
/// its block (x, z) in the world the game plays (a Petroff–Guyou bake's, [`from_bake`]). Each site's
/// `block` is that, or its `x` and `z` when it has none.
fn view_in_world(
    sites: &[Site],
    bodies: &[Body],
    circumference: u32,
    m_per_block_h: f64,
    source: &str,
    world: &HashMap<u32, [i32; 2]>,
) -> Value {
    let c = f64::from(circumference);
    let km = m_per_block_h / 1000.0;
    let mut by_site: HashMap<u32, Vec<&Body>> = HashMap::new();
    for b in bodies {
        if let Some(s) = b.site {
            by_site.entry(s).or_default().push(b);
        }
    }
    let mut list: Vec<&Site> = sites.iter().collect();
    list.sort_by_key(|s| s.id);
    let out: Vec<Value> = list
        .into_iter()
        .map(|s| {
            let own = by_site.get(&s.id).map_or(&[][..], Vec::as_slice);
            let f = footprint(s, own);
            let block = world
                .get(&s.id)
                .copied()
                .unwrap_or([s.center[0], s.center[2]]);
            json!({
                "id": s.id,
                "kind": s.kind.name(),
                "type": s.deposit_type,
                "lat": round(f64::from(s.lat_lon[0]), 3),
                "lon": round(f64::from(s.lat_lon[1]), 3),
                "x": s.center[0],
                "y": s.center[1],
                "z": s.center[2],
                "block": block,
                "y_range": [s.aabb[0][1], s.aabb[1][1]],
                "fp": {
                    "x": round(wrap_x(f.x, c), 1),
                    "z": round(f.z, 1),
                    "a": round(f.a, 1),
                    "c": round(f.c, 1),
                    "yaw": round(f.yaw, 3),
                    "km2": round(std::f64::consts::PI * f.a * f.c * km * km, 1),
                },
                "bodies": own.len(),
                "products": s.products.iter().map(|p| json!({
                    "material": p.material,
                    "grade": round(f64::from(p.grade), 6),
                    "tonnage_t": p.tonnage_t,
                    "role": p.role,
                })).collect::<Vec<_>>(),
                "forming_event": s.forming_event.0,
                "parent": s.parent,
            })
        })
        .collect();
    json!({
        "available": true,
        "source": source,
        "circumference": circumference,
        "m_per_block_h": m_per_block_h,
        "sites": out,
    })
}

/// What the preset says one block is, horizontally, in metres.
pub fn m_per_block_h(preset_json: &str) -> f64 {
    match Preset::from_json(preset_json).map(|p| p.body) {
        Ok(PresetBody::Planet(p)) => p.scale.m_per_block_h,
        _ => DEFAULT_M_PER_BLOCK_H,
    }
}

/// The view of a finished run's world data (`sites/` and `bodies/`), or `None` when the run has no
/// sites file (a run that ended before the deposit stage, or was cut short).
///
/// The overlay is drawn over the simulation's Lambert pictures (circumference `circumference`), so a
/// Petroff–Guyou bake's sites and bodies (format 0.5) are brought back to the simulation's frame: each
/// position through the world's projection to latitude and longitude, then to the Lambert block.
/// Each site's `block` keeps its baked centre, the world block the inspector and the game use
/// (the site card's "Block x, z"); its `x` and `z` are the simulation's.
pub fn from_bake(world: &Path, circumference: u32, m_per_block_h: f64) -> Option<Value> {
    let (mut sites, _) = read_sites(world).ok()??;
    let mut bodies = read_bodies(world).ok()?.unwrap_or_default();
    let mut blocks = HashMap::new();
    if let Ok(m) = deeptime_format::Manifest::read(world) {
        let wp = m.world_projection();
        if let deeptime_format::projection::WorldProjection::PetroffGuyou(_) = wp {
            let sim = deeptime_format::Projection::new(circumference);
            let back = |x: f64, z: f64| {
                let (lat, lon) = wp.lat_lon(x, z);
                sim.block_of(lat, lon)
            };
            for s in &mut sites {
                blocks.insert(s.id, [s.center[0], s.center[2]]);
                let (x, z) = sim.block_of(f64::from(s.lat_lon[0]), f64::from(s.lat_lon[1]));
                let (dx, dz) = (
                    x.round() as i32 - s.center[0],
                    z.round() as i32 - s.center[2],
                );
                s.center[0] += dx;
                s.center[2] += dz;
                for k in 0..2 {
                    s.aabb[k][0] += dx;
                    s.aabb[k][2] += dz;
                }
            }
            for b in &mut bodies {
                let (x, z) = back(b.center[0], b.center[2]);
                b.center[0] = x;
                b.center[2] = z;
                b.aabb = b.computed_aabb();
            }
        }
    }
    Some(view_in_world(
        &sites,
        &bodies,
        circumference,
        m_per_block_h,
        BAKE,
        &blocks,
    ))
}

/// The source and the JSON text of an answer.
pub type Answered = (&'static str, Arc<Vec<u8>>);

/// A run's deposits as last answered.
#[derive(Default)]
pub struct Cache(Mutex<Option<Answered>>);

impl Cache {
    pub fn set(&self, source: &'static str, v: &Value) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) =
            Some((source, Arc::new(v.to_string().into_bytes())));
    }

    pub fn get(&self) -> Option<Answered> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

/// The answer to `GET /api/runs/<id>/deposits`. A finished run's world data wins over what was
/// captured while it ran (the baked deposits at the end); a run that is not done answers with
/// what it captured, or `{"available": false, "reason"}`.
pub fn answer(
    cache: &Cache,
    done: bool,
    world: &Path,
    circumference: u32,
    preset_json: &str,
) -> Arc<Vec<u8>> {
    if done && cache.get().is_none_or(|(source, _)| source != BAKE) {
        let baked = from_bake(world, circumference, m_per_block_h(preset_json));
        if let Some(v) = baked {
            cache.set(BAKE, &v);
        }
    }
    cache.get().map(|(_, text)| text).unwrap_or_else(|| {
        let reason = if done {
            "this run wrote no sites (it has no deposit stage)"
        } else {
            "the deposit stage has not placed its sites yet"
        };
        Arc::new(
            json!({ "available": false, "reason": reason })
                .to_string()
                .into_bytes(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::bodies::{NO_CLIP, Shape};
    use deeptime_format::events::EventId;
    use deeptime_format::sites::{Extraction, Product, SiteKind};

    fn site(id: u32, ty: &str, ll: [f32; 2], center: [i32; 3]) -> Site {
        Site {
            id,
            kind: SiteKind::Deposit,
            deposit_type: ty.into(),
            center,
            lat_lon: ll,
            aabb: [
                [center[0] - 10, center[1] - 4, center[2] - 6],
                [center[0] + 10, center[1] + 4, center[2] + 6],
            ],
            body: None,
            products: vec![Product {
                material: "c:coal".into(),
                grade: 1.0,
                tonnage_t: 2.5e9,
                role: "primary".into(),
            }],
            forming_event: EventId::new(3, 7),
            parent: None,
            extraction: Extraction::default(),
        }
    }

    #[test]
    fn a_site_is_its_place_footprint_yield_and_forming_event() {
        let mut coal = site(4, "deeptime:coal_seam", [12.34567, 201.5], [-300, 30, -140]);
        coal.parent = Some(2);
        let porphyry = site(2, "deeptime:porphyry_cu_mo_au", [-5.0, 10.0], [100, 20, 50]);
        // The widest of a site's bodies is its footprint (a thin seam over a narrow one).
        let wide = Body::new(
            9,
            Some(4),
            Shape::Seam,
            "ore",
            [-300.0, 30.0, -140.0],
            [40.0, 0.1, 60.0],
            0.5,
            0.0,
            NO_CLIP,
        );
        let narrow = Body::new(
            8,
            Some(4),
            Shape::Seam,
            "ore",
            [-300.0, 25.0, -140.0],
            [10.0, 0.1, 10.0],
            0.0,
            0.0,
            NO_CLIP,
        );
        let v = view(&[coal, porphyry], &[narrow, wide], 16384, 300.0, LIVE);
        assert_eq!(v["available"], true);
        assert_eq!(v["source"], "live");
        let sites = v["sites"].as_array().unwrap();
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0]["id"], 2, "sorted by id");
        let c = &sites[1];
        assert_eq!(c["type"], "deeptime:coal_seam");
        assert_eq!(c["lat"], 12.346);
        assert_eq!(c["bodies"], 2);
        assert_eq!(c["fp"]["a"], 40.0);
        assert_eq!(c["fp"]["c"], 60.0);
        assert_eq!(c["fp"]["yaw"], 0.5);
        // pi · 40 · 60 blocks² at 0.3 km a block.
        assert_eq!(c["fp"]["km2"], 678.6);
        assert_eq!(c["parent"], 2);
        assert_eq!(c["forming_event"], EventId::new(3, 7).0);
        assert_eq!(c["products"][0]["tonnage_t"], 2.5e9);
        assert_eq!(c["y_range"], json!([26, 34]));
        // A site without bodies falls back to the ellipse in its box.
        let p = &sites[0];
        assert_eq!(p["bodies"], 0);
        assert_eq!(p["fp"]["a"], 10.5);
        assert_eq!(p["fp"]["c"], 6.5);
        assert_eq!(p["fp"]["x"], 100.0);
    }

    #[test]
    fn a_footprint_across_the_seam_is_wrapped_into_the_span() {
        assert_eq!(wrap_x(8200.0, 16384.0), 8200.0 - 16384.0);
        assert_eq!(wrap_x(-8200.0, 16384.0), 16384.0 - 8200.0);
        assert_eq!(wrap_x(5.0, 16384.0), 5.0);
    }

    #[test]
    fn frames_that_name_the_sites_layer_are_found() {
        assert!(names_sites(
            &json!({"layers": ["deposits/favourability/coal_seam", "deposits/sites"]})
        ));
        assert!(!names_sites(&json!({"layers": ["satellite", "elevation"]})));
        assert!(!names_sites(&json!({})));
    }

    #[test]
    fn the_answer_says_why_there_is_none_and_prefers_the_bake() {
        let dir = std::env::temp_dir().join(format!("deeptime-deposits-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cache = Cache::default();
        let a: Value = serde_json::from_slice(&answer(&cache, false, &dir, 1024, "{}")).unwrap();
        assert_eq!(a["available"], false);
        assert!(a["reason"].as_str().unwrap().contains("not placed"));
        let a: Value = serde_json::from_slice(&answer(&cache, true, &dir, 1024, "{}")).unwrap();
        assert_eq!(a["available"], false);
        // A live capture answers while the run goes on.
        let live = view(
            &[site(1, "deeptime:bif_fe", [0.0, 0.0], [0, 0, 0])],
            &[],
            1024,
            300.0,
            LIVE,
        );
        cache.set(LIVE, &live);
        let a: Value = serde_json::from_slice(&answer(&cache, false, &dir, 1024, "{}")).unwrap();
        assert_eq!(a["source"], "live");
        // A done run whose world data has no sites file keeps the capture.
        let a: Value = serde_json::from_slice(&answer(&cache, true, &dir, 1024, "{}")).unwrap();
        assert_eq!(a["source"], "live");
        // With the files, the bake wins.
        std::fs::create_dir_all(&dir).unwrap();
        let sites = [site(1, "deeptime:bif_fe", [0.0, 0.0], [0, 0, 0])];
        deeptime_format::sites::write_sites(&dir, 1024, &sites).unwrap();
        let a: Value = serde_json::from_slice(&answer(&cache, true, &dir, 1024, "{}")).unwrap();
        assert_eq!(a["source"], "bake");
        assert_eq!(a["sites"][0]["type"], "deeptime:bif_fe");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_petroff_bake_keeps_each_sites_world_block_beside_the_simulations() {
        // Site 15 of the REV-2 inspector's bake (b 6,656, λc 176.25, a 16k simulation): its
        // record's latitude and longitude, and its baked centre, the block holding its bodies.
        use deeptime_format::projection::PetroffGuyou;
        use deeptime_format::{Manifest, Scale, Writer};
        let dir = std::env::temp_dir().join(format!("deeptime-deposits-pg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let g = PetroffGuyou::new(6_656, 176.25);
        let mut m = Manifest::new("test", Writer::this("unit"), Scale::default_for(13_312));
        m.projection = Some(g.spec(None));
        m.write(&dir).unwrap();
        let ll = [40.2804_f32, 335.0288];
        let (wx, wz) = g.block_of(f64::from(ll[0]), f64::from(ll[1]));
        let world = [wx.round() as i32, wz.round() as i32];
        assert_eq!(world, [2391, -1829]);
        let sites = [site(15, "deeptime:coal_seam", ll, [world[0], 40, world[1]])];
        deeptime_format::sites::write_sites(&dir, 13_312, &sites).unwrap();
        let v = from_bake(&dir, 16_384, 300.0).unwrap();
        let s = &v["sites"][0];
        // `block` is the world's: the inspector reads the site's own latitude and longitude there.
        assert_eq!(s["block"], json!(world));
        let (lat, lon) = g.lat_lon(f64::from(world[0]), f64::from(world[1]));
        assert!((lat - f64::from(ll[0])).abs() < 0.02 && (lon - f64::from(ll[1])).abs() < 0.02);
        // `x` and `z` are the simulation's Lambert block the overlay is drawn at, not a world block:
        // read as one, they are another place (the inspector's 23.875° N, 101.11° E).
        let sim = deeptime_format::Projection::new(16_384);
        let (sx, sz) = sim.block_of(f64::from(ll[0]), f64::from(ll[1]));
        assert_eq!(
            (s["x"].as_i64().unwrap(), s["z"].as_i64().unwrap()),
            (sx.round() as i64, sz.round() as i64)
        );
        assert_eq!((s["x"].as_i64(), s["z"].as_i64()), (Some(7056), Some(-1686)));
        let (elat, elon) = g.lat_lon(7056.0, -1686.0);
        assert!((elat - 23.875).abs() < 0.01 && (elon - 101.11).abs() < 0.01);
        // A Lambert world's block is its x and z.
        let live = view(&sites, &[], 13_312, 300.0, LIVE);
        assert_eq!(live["sites"][0]["block"], json!(world));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_scale_comes_from_the_planet_preset() {
        // Not a preset, or not a planet: the default.
        assert_eq!(m_per_block_h("{}"), 300.0);
        let json = crate::run::load_preset_with(
            "earthlike_quick_16k",
            Some("scattered_continents"),
            Some(10.0),
        )
        .unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        let named = v["scale"]["m_per_block_h"].as_f64().unwrap_or(300.0);
        assert_eq!(m_per_block_h(&json), named);
    }
}
