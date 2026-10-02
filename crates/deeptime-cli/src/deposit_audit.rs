// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime deposit-audit --world-data <dir>`: how the deposits of a baked planet come out in
//! blocks, per deposit type, beside the background scatter of the same material and vanilla's
//! ore density (Ben, 2026-10-01: "Audit all types, then tune"). It needs no Minecraft server: it
//! evaluates the bake as the mod's features do.
//!
//! **Deposit ore** is `DepositFeature.materialAt`'s draw, ported bit for bit ([`unit`]): a block
//! inside a body becomes ore of the first zone whose accumulated `density` exceeds the hash of
//! (canonical x, y, z, body id). The host check is modelled from the column evaluator
//! (`ColumnWorld::sample`), because only the game knows the blocks: a block hosts ore when it is
//! at or below the column's top solid block (the bed under water), and either the body's own class
//! is loose (a placer's gravel hosts in anything solid) or the block is bedrock — its rock class
//! is not of the `unconsolidated` family and it lies under the `--soil` top blocks (4 by default,
//! the surface rules' soil). Caves, lava and aquifers are not modelled, so counts run a few
//! per cent high (measured against a real box: STATUS.md, build log "deposit audit").
//!
//! **Scatter** is `BackgroundScatterFeature.plan`, ported the same way, over `--chunks` random
//! land chunks (and every chunk of `--box`), with the same host model and the rule's `host_bias`
//! of the class at the block.
//!
//! The output is a table per deposit type (sites, body volume, ore blocks per site, ore per chunk
//! in the body, what the same material's scatter gives per chunk, vanilla's figure) and, with
//! `--json`, every site's row.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use deeptime_core::deposits::types::{self, DepositType};
use deeptime_core::surface::lithology;
use deeptime_core::synth::SplitMix;
use deeptime_format::sites::read_sites;
use deeptime_format::{Body, ColumnWorld, Site};
use serde_json::{Value, json};

use crate::run::CliResult;

/// The scatter rules the jar ships (`data/deeptime/deeptime/scatter/*.json`).
const SCATTER: &[&str] = &[
    include_str!("../../../neoforge/src/main/resources/data/deeptime/deeptime/scatter/coal.json"),
    include_str!("../../../neoforge/src/main/resources/data/deeptime/deeptime/scatter/copper.json"),
    include_str!("../../../neoforge/src/main/resources/data/deeptime/deeptime/scatter/gold.json"),
    include_str!("../../../neoforge/src/main/resources/data/deeptime/deeptime/scatter/iron.json"),
    include_str!("../../../neoforge/src/main/resources/data/deeptime/deeptime/scatter/zinc.json"),
];

/// Ore blocks per chunk in a vanilla 1.21.1 overworld (all heights, every biome of the sample),
/// measured with `tools/review/deposit-audit.sh vanilla` (STATUS.md, build log "deposit audit"):
/// material → blocks per chunk. One sample: 1,024 chunks around 0,0 of seed 20261001 on the
/// standalone set with level-type `minecraft:normal` (2026-10-01), so zinc is Create's own ore
/// feature, and copper is high for a world average (the sample's copper is 92 % above y 0, which
/// points at `ore_copper_large` under dripstone caves). Lead, silver and molybdenum have no ore
/// feature in that set.
pub const VANILLA_PER_CHUNK: &[(&str, f64)] = &[
    ("c:coal", 101.07),
    ("c:copper", 293.19),
    ("c:gold", 25.58),
    ("c:iron", 77.40),
    ("c:zinc", 76.18),
];

/// A stack of ore, and the audit's yardstick: ten of them.
pub const TEN_STACKS: f64 = 640.0;

/// What to audit.
pub struct Options {
    pub world_data: PathBuf,
    /// Extra or overriding scatter rules: a folder of `*.json` (a pack's `deeptime/scatter`).
    pub scatter_dir: Option<PathBuf>,
    /// Random land chunks the scatter is measured on.
    pub chunks: usize,
    pub seed: u64,
    /// Top solid blocks of a land column that are soil, not bedrock.
    pub soil: i32,
    /// Also count the deposit ore and scatter of this block box (x0, z0, x1, z1, inclusive), to
    /// compare with an `export-area` of the same box.
    pub area: Option<[i64; 4]>,
    pub json: Option<PathBuf>,
    /// Only the sites of these deposit types (ids, with or without `deeptime:`); empty = all.
    pub only: Vec<String>,
}

/// A uniform draw in [0, 1) from a block position and a salt: `DepositFeature.unit`.
pub fn unit(x: i32, y: i32, z: i32, salt: i32) -> f64 {
    fn mix(mut z: u64) -> u64 {
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    let l = |v: i32| i64::from(v) as u64;
    let mut h =
        mix(0x9E37_79B9_7F4A_7C15u64.wrapping_mul(l(x).wrapping_add(0x632B_E59B_D9B4_E019)));
    h = mix(h ^ 0xC2B2_AE3D_27D4_EB4Fu64.wrapping_mul(l(y).wrapping_add(0x2545_F491_4F6C_DD1D)));
    h = mix(h ^ 0x1656_67B1_9E37_79F9u64.wrapping_mul(l(z).wrapping_add(0x1B87_3593)));
    h = mix(h ^ u64::from(salt as u32));
    (h >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Java's `String.hashCode`.
pub fn java_hash(s: &str) -> i32 {
    s.encode_utf16()
        .fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(i32::from(c)))
}

/// One scatter rule (`OreData.ScatterRule`).
#[derive(Clone, Debug)]
pub struct ScatterRule {
    pub material: String,
    pub clusters_per_chunk: f64,
    pub size: i32,
    pub y_window: [f64; 2],
    pub host_bias: BTreeMap<String, f64>,
}

impl ScatterRule {
    fn parse(text: &str) -> CliResult<ScatterRule> {
        let v: Value = serde_json::from_str(text)?;
        let w = v["y_window"].as_array();
        Ok(ScatterRule {
            material: v["material"]
                .as_str()
                .ok_or("a scatter rule has no material")?
                .to_owned(),
            clusters_per_chunk: v["clusters_per_chunk"].as_f64().unwrap_or(0.0),
            size: v["size"].as_i64().unwrap_or(1) as i32,
            y_window: [
                w.and_then(|w| w[0].as_f64()).unwrap_or(0.0),
                w.and_then(|w| w[1].as_f64()).unwrap_or(1.0),
            ],
            host_bias: v["host_bias"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, x)| Some((k.clone(), x.as_f64()?)))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    fn bias(&self, class: Option<&str>) -> f64 {
        let star = self.host_bias.get("*").copied().unwrap_or(1.0);
        self.host_bias
            .get(class.unwrap_or("unclassified_crust"))
            .copied()
            .unwrap_or(star)
    }
}

/// The rules: the jar's, then a folder's on top (by material).
pub fn scatter_rules(dir: Option<&Path>) -> CliResult<Vec<ScatterRule>> {
    let mut by: BTreeMap<String, ScatterRule> = BTreeMap::new();
    for text in SCATTER {
        let r = ScatterRule::parse(text)?;
        by.insert(r.material.clone(), r);
    }
    if let Some(dir) = dir {
        for e in std::fs::read_dir(dir)? {
            let p = e?.path();
            if p.extension().is_some_and(|x| x == "json") {
                let r = ScatterRule::parse(&std::fs::read_to_string(&p)?)?;
                by.insert(r.material.clone(), r);
            }
        }
    }
    Ok(by.into_values().collect())
}

/// What the host model knows of one column.
struct Col {
    /// The top solid block (the bed under water).
    solid: i32,
    wet: bool,
    /// The terrain height `wd.height` floors for the scatter.
    surface: i32,
    /// (top, bottom, class) runs, top-down.
    rock: Vec<(i32, i32, Option<String>)>,
}

impl Col {
    fn class(&self, y: i32) -> Option<&str> {
        self.rock
            .iter()
            .find(|r| y <= r.0 && y >= r.1)
            .and_then(|r| r.2.as_deref())
    }
}

struct Model<'a> {
    w: &'a ColumnWorld,
    loose: HashSet<&'static str>,
    soil: i32,
    cols: HashMap<(i64, i64), Col>,
}

/// What a block is as a host.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Host {
    None,
    Loose,
    Rock,
}

impl<'a> Model<'a> {
    fn new(w: &'a ColumnWorld, soil: i32) -> Model<'a> {
        let loose = lithology::table()
            .iter()
            .filter(|c| c.family == "unconsolidated")
            .map(|c| c.name)
            .collect();
        Model {
            w,
            loose,
            soil,
            cols: HashMap::new(),
        }
    }

    fn col(&mut self, wx: i64, z: i64) -> &Col {
        let w = self.w;
        self.cols.entry((wx, z)).or_insert_with(|| {
            let c = w.sample(wx, z);
            let wet = c.hydro.kind.is_water();
            let solid = match (wet, c.hydro.bed) {
                (true, Some(b)) => b,
                _ => c.hydro.ground,
            };
            Col {
                solid,
                wet,
                surface: w.height(wx, z).floor() as i32,
                rock: c.rock,
            }
        })
    }

    /// The host kind of block (wx, y, z) and whether `rule` (a scatter rule) accepts `draw` there.
    fn host(&mut self, wx: i64, y: i32, z: i64) -> Host {
        let soil = self.soil;
        let w = self.w;
        let c = self.cols.entry((wx, z)).or_insert_with(|| {
            let c = w.sample(wx, z);
            let wet = c.hydro.kind.is_water();
            let solid = match (wet, c.hydro.bed) {
                (true, Some(b)) => b,
                _ => c.hydro.ground,
            };
            Col {
                solid,
                wet,
                surface: w.height(wx, z).floor() as i32,
                rock: c.rock,
            }
        });
        if y > c.solid {
            return Host::None;
        }
        let is_soil = !c.wet && y > c.solid - soil;
        if is_soil || c.class(y).is_some_and(|k| self.loose.contains(k)) {
            Host::Loose
        } else {
            Host::Rock
        }
    }

    fn class(&mut self, wx: i64, y: i32, z: i64) -> Option<&str> {
        self.col(wx, z).class(y)
    }

    fn forget(&mut self) {
        self.cols.clear();
    }
}

/// What one body places.
#[derive(Default, Clone)]
struct BodyCount {
    /// Blocks inside the body (below its clip).
    volume: u64,
    /// Of those, blocks that can host its ore.
    hosted: u64,
    /// Of the hosted, blocks inside a zone that places ore.
    in_zone: u64,
    ore: BTreeMap<String, u64>,
    chunks: HashSet<(i64, i64)>,
}

/// Counts body `b`. `seen` holds the blocks earlier bodies of the same site covered (a placer's
/// bars overlap): a block in it is not counted again as volume, and takes a second draw only if
/// the first left it rock, as in the game.
fn count_body(
    m: &mut Model<'_>,
    b: &Body,
    area: Option<[i64; 4]>,
    seen: &mut HashMap<(i64, i32, i64), bool>,
    track: bool,
) -> BodyCount {
    let c = m.w.c;
    let (min_y, max_y) = (m.w.min_y, m.w.max_y);
    let body_loose = b.rock_class.as_deref().is_some_and(|k| m.loose.contains(k));
    let mut out = BodyCount::default();
    let [lo, hi] = b.aabb;
    for x in lo[0]..=hi[0] {
        let wx = m.w.wrap_x(i64::from(x));
        for z in lo[2]..=hi[2] {
            let z = i64::from(z);
            if let Some([x0, z0, x1, z1]) = area
                && !((wx - x0).rem_euclid(c) <= x1 - x0 && (z0..=z1).contains(&z))
            {
                continue;
            }
            for y in lo[1].max(min_y)..=hi[1].min(max_y) {
                let Some(at) = b.locate(c as u32, wx as i32, y, z as i32) else {
                    continue;
                };
                let before = if track {
                    seen.get(&(wx, y, z)).copied()
                } else {
                    None
                };
                if before.is_none() {
                    out.volume += 1;
                    if track {
                        seen.insert((wx, y, z), false);
                    }
                }
                out.chunks.insert((wx.div_euclid(16), z.div_euclid(16)));
                if before == Some(true) {
                    continue;
                }
                let kind = m.host(wx, y, z);
                if kind == Host::None || (kind == Host::Loose && !body_loose) {
                    continue;
                }
                if before.is_none() {
                    out.hosted += 1;
                }
                let u = unit(wx as i32, y, z as i32, b.id as i32);
                let mut acc = 0.0;
                let mut any = false;
                for zone in b.zones_at(at) {
                    let Some(mat) = &zone.material else { continue };
                    if zone.density <= 0.0 {
                        continue;
                    }
                    any = true;
                    acc += f64::from(zone.density);
                    if u < acc {
                        *out.ore.entry(mat.clone()).or_default() += 1;
                        if track {
                            seen.insert((wx, y, z), true);
                        }
                        break;
                    }
                }
                out.in_zone += u64::from(any && before.is_none());
            }
        }
    }
    out
}

/// The ore blocks rule `r` places in chunk (cx, cz): `BackgroundScatterFeature.plan` and the host
/// check.
fn scatter_chunk(m: &mut Model<'_>, r: &ScatterRule, cx: i64, cz: i64) -> u64 {
    let c = m.w.c;
    let min_y = m.w.min_y;
    let salt = java_hash(&r.material);
    let kx = cx.rem_euclid(c >> 4) as i32;
    let czi = cz as i32;
    let mut n = r.clusters_per_chunk.floor() as i32;
    if unit(kx, czi, -1, salt) < r.clusters_per_chunk - f64::from(n) {
        n += 1;
    }
    let (x0, z0) = (cx * 16, cz * 16);
    let mut seen: HashSet<(i32, i32, i32)> = HashSet::new();
    let mut placed = 0;
    for k in 0..n {
        let ks = salt ^ k.wrapping_mul(0x9E37_79B9u32 as i32);
        let mut lx = (unit(kx, czi, 1, ks) * 16.0) as i32;
        let mut lz = (unit(kx, czi, 2, ks) * 16.0) as i32;
        let wx0 = m.w.wrap_x(x0 + i64::from(lx));
        let surface = m.col(wx0, z0 + i64::from(lz)).surface;
        let f = r.y_window[0] + unit(kx, czi, 3, ks) * (r.y_window[1] - r.y_window[0]);
        let mut y = surface - (f * f64::from(surface - min_y) + 0.5).floor() as i32;
        for step in 0..r.size {
            if y >= min_y && y <= surface && seen.insert((lx, lz, y)) {
                let wx = m.w.wrap_x(x0 + i64::from(lx));
                let z = z0 + i64::from(lz);
                let draw = unit(wx as i32, y, z as i32, ks);
                if m.host(wx, y, z) == Host::Rock && draw < r.bias(m.class(wx, y, z)) {
                    placed += 1;
                }
            }
            match (unit(kx, czi, 16 + step, ks) * 6.0) as i32 {
                0 => lx = (lx + 1).min(15),
                1 => lx = (lx - 1).max(0),
                2 => lz = (lz + 1).min(15),
                3 => lz = (lz - 1).max(0),
                4 => y += 1,
                _ => y -= 1,
            }
        }
    }
    placed
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = (q * (sorted.len() - 1) as f64).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

fn summary(mut v: Vec<f64>) -> Value {
    v.sort_by(f64::total_cmp);
    let mean = if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    };
    json!({
        "p10": num(quantile(&v, 0.1)), "p50": num(quantile(&v, 0.5)), "p90": num(quantile(&v, 0.9)),
        "mean": num(mean), "min": num(v.first().copied().unwrap_or(f64::NAN)),
        "max": num(v.last().copied().unwrap_or(f64::NAN)),
    })
}

fn num(v: f64) -> Value {
    if v.is_finite() {
        json!((v * 1000.0).round() / 1000.0)
    } else {
        Value::Null
    }
}

fn vanilla(material: &str) -> Option<f64> {
    VANILLA_PER_CHUNK
        .iter()
        .find(|(m, _)| *m == material)
        .map(|(_, v)| *v)
        .filter(|v| *v > 0.0)
}

/// Blocks from pole to pole on a planet `c` blocks round (the projection's north–south extent).
fn pole_to_pole_blocks(c: i64) -> i64 {
    deeptime_format::Projection::of(c as f64).extent().1 as i64
}

/// Runs the audit; returns the JSON report (also written to `--json`).
pub fn run(o: &Options) -> CliResult<Value> {
    let w = ColumnWorld::open(&o.world_data)?;
    let (sites, _) = read_sites(&o.world_data)?.ok_or("the world data has no sites")?;
    let types: Vec<DepositType> = types::builtin()?;
    let rules = scatter_rules(o.scatter_dir.as_deref())?;
    let mut m = Model::new(&w, o.soil);
    let c = w.c;

    // Scatter on random land chunks.
    let mut rng = SplitMix(o.seed);
    let sea = w.manifest.scale.sea_level_y;
    let mut land = Vec::new();
    let mut tries = 0;
    while land.len() < o.chunks && tries < o.chunks * 200 {
        tries += 1;
        let cx = rng.below((c / 16) as u64) as i64 - c / 32;
        // Between the poles, away from their last 1/16.
        let span = (pole_to_pole_blocks(c) / 16) * 7 / 8;
        let cz = rng.below(span as u64) as i64 - span / 2;
        if w.height(cx * 16 + 8, cz * 16 + 8) >= f64::from(sea) {
            land.push((cx, cz));
        }
    }
    let mut scatter: BTreeMap<String, f64> = BTreeMap::new();
    let mut scatter_json = serde_json::Map::new();
    for r in &rules {
        let total: u64 = land
            .iter()
            .map(|&(cx, cz)| scatter_chunk(&mut m, r, cx, cz))
            .sum();
        let per = total as f64 / land.len().max(1) as f64;
        scatter.insert(r.material.clone(), per);
        scatter_json.insert(
            r.material.clone(),
            json!({
                "per_chunk": num(per),
                // A chunk is 16 × 16 blocks: 3,906.25 chunks to the square kilometre of blocks.
                "per_km2_of_blocks": num(per * 3906.25),
                "rule": {"clusters_per_chunk": r.clusters_per_chunk, "size": r.size, "y_window": r.y_window},
                "rule_max_per_chunk": num(r.clusters_per_chunk * f64::from(r.size)),
                "vanilla_per_chunk": vanilla(&r.material),
                "chunks_for_ten_stacks": num(TEN_STACKS / per),
            }),
        );
    }
    m.forget();

    // The box, for a comparison with a real export.
    let mut area_json = Value::Null;
    if let Some(a) = o.area {
        let mut ore: BTreeMap<String, u64> = BTreeMap::new();
        let mut per_body = Vec::new();
        for b in &w.bodies {
            let n = count_body(&mut m, b, Some(a), &mut HashMap::new(), false);
            if n.volume > 0 {
                per_body.push(json!({"body": b.id, "site": b.site, "role": b.role,
                    "voxels": n.volume, "ore": n.ore}));
                for (k, v) in n.ore {
                    *ore.entry(k).or_default() += v;
                }
            }
        }
        let mut sc: BTreeMap<String, u64> = BTreeMap::new();
        let (cx0, cx1) = (a[0].div_euclid(16), a[2].div_euclid(16));
        let (cz0, cz1) = (a[1].div_euclid(16), a[3].div_euclid(16));
        for r in &rules {
            let mut t = 0;
            for cx in cx0..=cx1 {
                for cz in cz0..=cz1 {
                    t += scatter_chunk(&mut m, r, cx, cz);
                }
            }
            sc.insert(r.material.clone(), t);
        }
        area_json = json!({
            "box": a, "chunks": (cx1 - cx0 + 1) * (cz1 - cz0 + 1),
            "note": "scatter is counted on whole chunks touching the box",
            "deposit_ore": ore, "bodies": per_body, "scatter_ore": sc,
        });
        m.forget();
    }

    // Every site.
    let mut by_body: HashMap<u32, Vec<&Body>> = HashMap::new();
    for b in &w.bodies {
        if let Some(s) = b.site {
            by_body.entry(s).or_default().push(b);
        }
    }
    let mut rows: Vec<Value> = Vec::new();
    struct Row {
        ty: String,
        volume: f64,
        ore_total: f64,
        ore_primary: f64,
        per_chunk: f64,
        in_zone_share: f64,
        hosted_share: f64,
        tonnage: f64,
        chunks: f64,
    }
    let mut stats: Vec<Row> = Vec::new();
    for s in &sites {
        if !o.only.is_empty()
            && !o
                .only
                .iter()
                .any(|t| s.deposit_type == *t || s.deposit_type == format!("deeptime:{t}"))
        {
            continue;
        }
        let bodies = by_body.get(&s.id).cloned().unwrap_or_default();
        let mut tot = BodyCount::default();
        let mut ore_volume = 0u64;
        // Only a site whose bodies' boxes overlap (a placer's bars) needs the per-block record.
        let track = bodies.iter().enumerate().any(|(i, a)| {
            bodies[..i].iter().any(|o| {
                (0..3).all(|k| a.aabb[0][k] <= o.aabb[1][k] && o.aabb[0][k] <= a.aabb[1][k])
            })
        });
        let mut seen = HashMap::new();
        for b in &bodies {
            let n = count_body(&mut m, b, None, &mut seen, track);
            if b.role == "ore" || b.role == "reservoir" {
                ore_volume += n.volume;
            }
            tot.volume += n.volume;
            tot.hosted += n.hosted;
            tot.in_zone += n.in_zone;
            for (k, v) in n.ore {
                *tot.ore.entry(k).or_default() += v;
            }
            tot.chunks.extend(n.chunks);
        }
        m.forget();
        let primary = primary_of(s);
        let ore_total: u64 = tot.ore.values().sum();
        let ore_primary = primary
            .as_deref()
            .and_then(|p| tot.ore.get(p))
            .copied()
            .unwrap_or(0);
        let tonnage = s.products.first().map_or(0.0, |p| p.tonnage_t);
        let chunks = tot.chunks.len() as f64;
        let here = w.hydro(i64::from(s.center[0]), i64::from(s.center[2]));
        rows.push(json!({
            "site": s.id, "type": s.deposit_type, "center": s.center, "bodies": bodies.len(),
            // The ground and the water at the site's centre column.
            "ground_y": here.ground, "water": here.kind.name(),
            "tonnage_t": tonnage,
            "grade": s.products.first().map(|p| p.grade),
            "primary": primary,
            "volume_blocks": tot.volume, "ore_body_blocks": ore_volume,
            "hostable_blocks": tot.hosted, "in_ore_zone_blocks": tot.in_zone,
            "ore_blocks": tot.ore, "ore_total": ore_total, "chunks": chunks,
            "coe_vein": s.extraction.coe_vein, "cdg_charge_mb": s.extraction.cdg_charge_mb,
        }));
        stats.push(Row {
            ty: s.deposit_type.clone(),
            volume: tot.volume as f64,
            ore_total: ore_total as f64,
            ore_primary: ore_primary as f64,
            per_chunk: if chunks > 0.0 {
                ore_total as f64 / chunks
            } else {
                0.0
            },
            in_zone_share: if tot.hosted > 0 {
                ore_total as f64 / tot.hosted as f64
            } else {
                0.0
            },
            hosted_share: if tot.volume > 0 {
                tot.hosted as f64 / tot.volume as f64
            } else {
                0.0
            },
            tonnage,
            chunks,
        });
    }

    let mut families = Vec::new();
    let mut ids: Vec<String> = types.iter().map(|t| t.id.clone()).collect();
    for r in &stats {
        if !ids.contains(&r.ty) {
            ids.push(r.ty.clone());
        }
    }
    for id in &ids {
        let t = types.iter().find(|t| &t.id == id);
        let rs: Vec<&Row> = stats.iter().filter(|r| &r.ty == id).collect();
        let primary = t
            .and_then(|t| t.products.first())
            .map(|p| p.material.clone());
        let sc = primary.as_ref().and_then(|p| scatter.get(p)).copied();
        let ore: Vec<f64> = rs.iter().map(|r| r.ore_total).collect();
        let mut sorted = ore.clone();
        sorted.sort_by(f64::total_cmp);
        let med = quantile(&sorted, 0.5);
        let dens: Vec<f64> = rs
            .iter()
            .filter(|r| r.ore_total > 0.0)
            .map(|r| r.in_zone_share)
            .collect();
        let mut dsorted = dens.clone();
        dsorted.sort_by(f64::total_cmp);
        let dmed = quantile(&dsorted, 0.5);
        families.push(json!({
            "type": id,
            "family": t.map(|t| t.family.clone()),
            "primary": primary,
            "sites": rs.len(),
            "sites_with_ore": rs.iter().filter(|r| r.ore_total > 0.0).count(),
            "tonnage_t": summary(rs.iter().map(|r| r.tonnage).collect()),
            "volume_blocks": summary(rs.iter().map(|r| r.volume).collect()),
            "chunks": summary(rs.iter().map(|r| r.chunks).collect()),
            "ore_blocks": summary(ore),
            "ore_primary_blocks": summary(rs.iter().map(|r| r.ore_primary).collect()),
            "ore_per_chunk_in_body": summary(rs.iter().filter(|r| r.chunks > 0.0).map(|r| r.per_chunk).collect()),
            "hostable_share": summary(rs.iter().map(|r| r.hosted_share).collect()),
            "ore_share_of_hostable": summary(dens),
            "scatter_per_chunk": sc.map(num),
            "vanilla_per_chunk": primary.as_deref().and_then(vanilla),
            // How many chunks of the same material's scatter the median deposit is worth.
            "median_deposit_in_scatter_chunks": sc.filter(|s| *s > 0.0).map(|s| num(med / s)),
            "median_deposit_in_vanilla_chunks": primary.as_deref().and_then(vanilla).map(|v| num(med / v)),
            // Rock to dig inside the body for ten stacks, at the median site's ore share.
            "blocks_dug_for_ten_stacks_in_body": if dmed > 0.0 { num(TEN_STACKS / dmed) } else { Value::Null },
            "scatter_chunks_for_ten_stacks": sc.filter(|s| *s > 0.0).map(|s| num(TEN_STACKS / s)),
            "zones": t.map(|t| t.geometry.zoning.iter().map(|z| json!({
                "zone": z.zone, "material": z.material, "density": z.density, "d": z.d, "v": z.v,
            })).collect::<Vec<_>>()),
            "balance": t.map(|t| json!({
                "size_mult_h": t.balance.size_mult_h, "size_mult_v": t.balance.size_mult_v,
                "richness_mult": t.balance.richness_mult, "count_mult": t.balance.count_mult,
            })),
            "coe_vein": t.and_then(|t| t.minecraft.coe_vein.clone()),
        }));
    }

    // The planet's ore by where it comes from: every deposit's blocks against the scatter of the
    // land between the caps (the sampled share of land × the chunks there).
    let band_chunks = (c / 16) as f64 * ((pole_to_pole_blocks(c) / 16) * 7 / 8) as f64;
    let land_chunks = band_chunks * land.len() as f64 / tries.max(1) as f64;
    let mut in_deposits: BTreeMap<String, u64> = BTreeMap::new();
    for r in &rows {
        for (k, v) in r["ore_blocks"].as_object().into_iter().flatten() {
            *in_deposits.entry(k.clone()).or_default() += v.as_u64().unwrap_or(0);
        }
    }
    let mut totals = serde_json::Map::new();
    let mut materials: Vec<&String> = in_deposits.keys().chain(scatter.keys()).collect();
    materials.sort();
    materials.dedup();
    for k in materials {
        let d = in_deposits.get(k).copied().unwrap_or(0) as f64;
        let s = scatter.get(k).copied().unwrap_or(0.0) * land_chunks;
        totals.insert(
            k.clone(),
            json!({
                "in_deposits": d, "in_scatter": s.round(),
                "deposit_share": if d + s > 0.0 { num(d / (d + s)) } else { Value::Null },
            }),
        );
    }

    let report = json!({
        "planet": {"land_chunks_estimate": land_chunks.round(), "ore_by_source": totals},
        "world_data": o.world_data.display().to_string(),
        "circumference": c,
        "preset": w.manifest.notes.get("run").and_then(|r| r.get("preset")).cloned(),
        "host_model": {"soil_blocks": o.soil, "note": "column evaluator; caves, lava and aquifers not modelled"},
        "scatter": {"land_chunks": land.len(), "by_material": scatter_json},
        "types": families,
        "area": area_json,
        "sites": rows,
    });
    if let Some(p) = &o.json {
        std::fs::write(p, serde_json::to_string_pretty(&report)?)?;
    }
    Ok(report)
}

/// The site's primary product (the first, as the types list them).
fn primary_of(s: &Site) -> Option<String> {
    s.products
        .iter()
        .find(|p| p.role == "primary")
        .or(s.products.first())
        .map(|p| p.material.clone())
}

/// The report as a table (Markdown).
pub fn table(report: &Value) -> String {
    let f = |v: &Value| match v.as_f64() {
        Some(x) if x.abs() >= 100.0 => format!("{x:.0}"),
        Some(x) if x.abs() >= 10.0 => format!("{x:.1}"),
        Some(x) => format!("{x:.2}"),
        None => "–".into(),
    };
    let mut out = String::new();
    out.push_str(&format!(
        "Deposit audit of {} (C = {})\n\n",
        report["world_data"].as_str().unwrap_or("?"),
        report["circumference"]
    ));
    out.push_str("| type | sites (with ore) | body blocks p50 | chunks p50 | ore/site p10 | p50 | p90 | ore/chunk in body p50 | scatter/chunk | vanilla/chunk | p50 deposit = scatter chunks | dig for 640 in body | scatter chunks for 640 |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for t in report["types"].as_array().into_iter().flatten() {
        out.push_str(&format!(
            "| {} | {} ({}) | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            t["type"]
                .as_str()
                .unwrap_or("?")
                .trim_start_matches("deeptime:"),
            t["sites"],
            t["sites_with_ore"],
            f(&t["volume_blocks"]["p50"]),
            f(&t["chunks"]["p50"]),
            f(&t["ore_blocks"]["p10"]),
            f(&t["ore_blocks"]["p50"]),
            f(&t["ore_blocks"]["p90"]),
            f(&t["ore_per_chunk_in_body"]["p50"]),
            f(&t["scatter_per_chunk"]),
            f(&t["vanilla_per_chunk"]),
            f(&t["median_deposit_in_scatter_chunks"]),
            f(&t["blocks_dug_for_ten_stacks_in_body"]),
            f(&t["scatter_chunks_for_ten_stacks"]),
        ));
    }
    out.push_str("\n| scatter material | per chunk | rule max | per km² of blocks | vanilla per chunk | chunks for 640 |\n|---|---|---|---|---|---|\n");
    for (k, v) in report["scatter"]["by_material"]
        .as_object()
        .into_iter()
        .flatten()
    {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            k,
            f(&v["per_chunk"]),
            f(&v["rule_max_per_chunk"]),
            f(&v["per_km2_of_blocks"]),
            f(&v["vanilla_per_chunk"]),
            f(&v["chunks_for_ten_stacks"]),
        ));
    }
    out.push_str(&format!(
        "\n| material | ore blocks in deposits | in scatter (≈{} land chunks) | deposits' share |\n|---|---|---|---|\n",
        report["planet"]["land_chunks_estimate"]
    ));
    for (k, v) in report["planet"]["ore_by_source"]
        .as_object()
        .into_iter()
        .flatten()
    {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            k,
            f(&v["in_deposits"]),
            f(&v["in_scatter"]),
            f(&v["deposit_share"]),
        ));
    }
    if !report["area"].is_null() {
        out.push_str(&format!(
            "\nBox {}: deposit ore {}, scatter ore {} ({} chunks)\n",
            report["area"]["box"],
            report["area"]["deposit_ore"],
            report["area"]["scatter_ore"],
            report["area"]["chunks"]
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DepositFeature.unit` and `String.hashCode`, against values the Java gives
    /// (`DepositFeatureTest`'s fixture draws).
    #[test]
    fn the_draw_is_in_range_and_stable() {
        let u = unit(433, 119, -34, 700);
        assert!((0.0..1.0).contains(&u));
        assert_eq!(u, unit(433, 119, -34, 700));
        assert_ne!(u, unit(433, 119, -34, 701));
        assert_eq!(java_hash("c:gold"), {
            let mut h = 0i32;
            for b in "c:gold".bytes() {
                h = h.wrapping_mul(31).wrapping_add(i32::from(b));
            }
            h
        });
        assert_eq!(java_hash("a"), 97);
        assert_eq!(java_hash("ab"), 97 * 31 + 98);
    }

    #[test]
    fn the_shipped_scatter_rules_parse() {
        let r = scatter_rules(None).unwrap();
        assert_eq!(r.len(), 5);
        let gold = r.iter().find(|r| r.material == "c:gold").unwrap();
        assert_eq!(gold.size, 4);
        assert_eq!(gold.bias(Some("shale")), 1.0);
        let coal = r.iter().find(|r| r.material == "c:coal").unwrap();
        assert_eq!(coal.bias(Some("granite")), 0.3);
        assert_eq!(coal.bias(None), 0.6);
    }
}
