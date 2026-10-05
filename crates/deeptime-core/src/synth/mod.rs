// SPDX-License-Identifier: LGPL-3.0-or-later
//! Synthetic world data for Phase 0, the GameTests and the Phase 1 placeholder presets (plan §9.3,
//! P0-09). Moved here from `deeptime-cli` in P1-13 so a session can bake it too: `deeptime synth`
//! writes a folder of its own from it, and a dummy preset with `"synth": "<kind>"` bakes the same
//! layers into its run's world data (`Session::finalize_write`), which is what makes the built-in
//! placeholder presets playable before the real stages exist.

pub mod terrain;

use std::path::Path;

use deeptime_format::{
    Codec, DType, FileEntry, FormatError, GridDesc, LayerDesc, LayerEntry, LoadedLayer, Manifest,
    Raster, RasterRead, Scale, TILE, fsutil::write_atomic, write_tile_pack,
};
use serde_json::{Value, json};

use terrain::{LakeTest, Terrain, WrapTest};

/// Blocks per cell of each grid in the synthetic worlds (the "16k quick" ladder of plan §5.2 at
/// every size: L3 r 2, L2 r 4 — the climate layers' vanilla quart grid — and L1 r 16).
pub const R_L3: u32 = 2;
pub const R_L2: u32 = 4;
pub const R_L1: u32 = 16;

/// Raw nodata of i16 layers.
pub const NODATA_I16: i16 = i16::MIN;

/// The climate layers, in vanilla parameter × 10,000 (the router's field names, plan §7.1).
pub const CLIMATE: [&str; 5] = [
    "climate/temperature",
    "climate/vegetation",
    "climate/continents",
    "climate/erosion",
    "climate/ridges",
];

/// The strata placeholder's classes (vanilla members in the rock-tag vocabulary: sandstone,
/// red_sandstone, granite).
pub const STRATA_CLASSES: [&str; 3] = ["sandstone", "red_bed", "granite"];

/// Every rock class of the vocabulary, in the order of the rock-tag README's Classes table
/// (`/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md`). The
/// definitions are Deep Time's `rock_class` data (`neoforge/src/main/resources/data/deeptime/
/// deeptime/rock_class/`); a unit test keeps the two lists equal.
pub const EVERY_CLASS: [&str; 62] = [
    "granite",
    "granodiorite",
    "diorite",
    "gabbro",
    "anorthosite",
    "peridotite",
    "rhyolite",
    "andesite",
    "basalt",
    "tuff",
    "obsidian",
    "scoria",
    "pumice",
    "conglomerate",
    "diamictite",
    "sandstone",
    "red_bed",
    "mudstone",
    "shale",
    "black_shale",
    "limestone",
    "dolostone",
    "chalk",
    "chert",
    "travertine",
    "halite",
    "gypsum",
    "anhydrite",
    "coal",
    "ironstone",
    "banded_iron_formation",
    "phosphorite",
    "slate",
    "phyllite",
    "schist",
    "blueschist",
    "greenstone",
    "gneiss",
    "quartzite",
    "marble",
    "amphibolite",
    "eclogite",
    "hornfels",
    "skarn",
    "serpentinite",
    "kimberlite",
    "carbonatite",
    "pegmatite",
    "greisen",
    "impact_breccia",
    "impact_melt",
    "soil",
    "saprolite",
    "laterite",
    "bauxite",
    "gravel",
    "sand",
    "clay",
    "mud",
    "till",
    "loess",
    "volcanic_ash",
];

/// The basin strata's classes (`Strata::Basin`).
pub const BASIN_CLASSES: [&str; 5] = ["sandstone", "shale", "mudstone", "limestone", "granite"];

/// Which horizon stacks a synthetic world carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strata {
    /// Three horizons: sandstone from the surface, red beds from 6 blocks down, granite from 40
    /// blocks down to the bottom of the world (P0-09).
    Placeholder,
    /// Every rock class (P1-12): eight horizons 10 blocks apart from the surface down; horizon
    /// h of L1 cell column i has class (6·i + h) mod 62 of [`EVERY_CLASS`], so any eleven
    /// neighbouring cell columns hold every class at h ≥ 2 (20 blocks or more below the surface,
    /// under whatever the surface rules paint; each column holds six consecutive classes there,
    /// and 11 × 6 = 66 ≥ 62). The last horizon reaches down to min_y.
    EveryClass,
    /// A readable sedimentary basin, the same in every cell: sandstone, shale, sandstone,
    /// mudstone, limestone, shale, sandstone and shale beds from the surface down to 165 blocks,
    /// then granite to the floor (the synthetic deposit world's strata, so its coal seams and
    /// ore bodies sit in sections that read as geology).
    Basin,
}

impl Strata {
    pub fn name(self) -> &'static str {
        match self {
            Strata::Placeholder => "placeholder",
            Strata::EveryClass => "every-class",
            Strata::Basin => "basin",
        }
    }

    /// The strata called `name` (`placeholder`, `every-class`, `basin`).
    pub fn from_name(name: &str) -> Option<Strata> {
        [Strata::Placeholder, Strata::EveryClass, Strata::Basin]
            .into_iter()
            .find(|s| s.name() == name)
    }

    /// The class list written to `strata/classes.json`.
    pub fn classes(self) -> &'static [&'static str] {
        match self {
            Strata::Placeholder => &STRATA_CLASSES,
            Strata::EveryClass => &EVERY_CLASS,
            Strata::Basin => &BASIN_CLASSES,
        }
    }

    /// Horizon tops below the surface, in blocks.
    fn depths(self) -> &'static [i32] {
        match self {
            Strata::Placeholder => &[0, 6, 40],
            Strata::EveryClass => &[0, 10, 20, 30, 40, 50, 60, 70],
            Strata::Basin => &[0, 8, 22, 36, 55, 75, 100, 130, 165],
        }
    }

    /// The class index of horizon `h` in L1 cell column `i`.
    pub fn class_index(self, i: u32, h: usize) -> u8 {
        match self {
            Strata::Placeholder => h as u8,
            Strata::EveryClass => ((6 * u64::from(i) + h as u64) % EVERY_CLASS.len() as u64) as u8,
            Strata::Basin => [0, 1, 0, 2, 3, 1, 0, 1, 4][h.min(8)],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    WrapTest,
    LakeTest,
    /// A synthetic deposit world's terrain (P1-15): the deposit history's present elevation
    /// (`crate::deposits::synthetic::HistoryTerrain`). Needs the history, so it is baked through
    /// [`build_with`] only and is not in [`Kind::ALL`].
    DepositHistory,
}

impl Kind {
    /// The kinds [`build`] makes from a name alone.
    pub const ALL: [Kind; 2] = [Kind::WrapTest, Kind::LakeTest];

    pub fn name(self) -> &'static str {
        match self {
            Kind::WrapTest => "wrap-test",
            Kind::LakeTest => "lake-test",
            Kind::DepositHistory => "deposit-history",
        }
    }

    /// The kind called `name` (`wrap-test`, `lake-test`).
    pub fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// Checks that `c` can carry a synthetic world: a multiple of 64 and at least two tiles.
pub fn check_circumference(c: u32) -> Result<(), String> {
    if c < 2 * TILE || !c.is_multiple_of(64) {
        return Err(format!(
            "the circumference must be a multiple of 64 and at least {}",
            2 * TILE
        ));
    }
    Ok(())
}

/// A synthetic world's layers, sampled and encoded but not yet written.
pub struct SynthWorld {
    pub kind: Kind,
    /// Which horizon stacks `strata` holds.
    pub strata_kind: Strata,
    pub circumference: u32,
    /// L1, L2 and L3.
    pub grids: [GridDesc; 3],
    /// `elevation`, `water_level`, then the five climate layers.
    pub layers: Vec<LoadedLayer>,
    /// `strata/horizons` on L1.
    pub strata: LoadedLayer,
    /// `spawn/candidates.json`.
    pub spawn: Value,
}

/// Samples a synthetic world of `kind` with `strata` horizons at circumference `c`.
pub fn build(kind: Kind, strata: Strata, c: u32, codec: Codec) -> Result<SynthWorld, FormatError> {
    let terrain: Box<dyn Terrain> = match kind {
        Kind::WrapTest => Box::new(WrapTest::new(c)),
        Kind::LakeTest => Box::new(LakeTest { c }),
        Kind::DepositHistory => {
            return Err(FormatError::Invalid(
                "deposit-history needs its history: bake it with build_with".into(),
            ));
        }
    };
    build_with(terrain.as_ref(), kind, strata, c, codec)
}

/// Samples a synthetic world of `terrain` (named `kind`) with `strata` horizons at circumference
/// `c`.
pub fn build_with(
    terrain: &dyn Terrain,
    kind: Kind,
    strata: Strata,
    c: u32,
    codec: Codec,
) -> Result<SynthWorld, FormatError> {
    check_circumference(c).map_err(|e| FormatError::Invalid(format!("--size {c}: {e}")))?;
    if terrain.circumference() != c {
        return Err(FormatError::Invalid(format!(
            "the terrain's circumference {} is not {c}",
            terrain.circumference()
        )));
    }

    let l3 = GridDesc::for_planet("L3", c, R_L3)?;
    let l2 = GridDesc::for_planet("L2", c, R_L2)?;
    let l1 = GridDesc::for_planet("L1", c, R_L1)?;

    let mut layers: Vec<LoadedLayer> = Vec::new();

    let elev = sample_i16(&l3, |x, z| quantise(terrain.height(x, z) * 16.0));
    layers.push(layer_i16(
        "elevation",
        "block",
        1.0 / 16.0,
        None,
        &l3,
        codec,
        &elev,
    )?);

    let water = sample_i16(&l3, |x, z| match terrain.water_level(x, z) {
        Some(y) => quantise(y * 16.0),
        None => NODATA_I16,
    });
    layers.push(layer_i16(
        "water_level",
        "block",
        1.0 / 16.0,
        Some(NODATA_I16),
        &l3,
        codec,
        &water,
    )?);

    for name in CLIMATE {
        let ras = sample_i16(&l2, |x, z| quantise(terrain.raster(name, x, z) * 10_000.0));
        layers.push(layer_i16(
            name,
            "vanilla_param",
            1e-4,
            None,
            &l2,
            codec,
            &ras,
        )?);
    }

    let strata_kind = strata;
    let strata = strata_layer(&l1, &layers[0], strata_kind, codec)?;
    let spawn = spawn_candidates(&layers[0], &layers[1], Scale::default_for(c).sea_level_y);
    Ok(SynthWorld {
        kind,
        strata_kind,
        circumference: c,
        grids: [l1, l2, l3],
        layers,
        strata,
        spawn,
    })
}

impl SynthWorld {
    /// Writes the tile packs and the side files into `out` and records them in `manifest` (grids,
    /// layers, files, `planet.radius_m`, notes `synthetic`, `strata` and for the lake `lake`).
    /// Writing the manifest itself is the caller's job, and comes last. Returns the bytes written.
    pub fn write_into(&self, out: &Path, manifest: &mut Manifest) -> Result<u64, FormatError> {
        let c = self.circumference;
        let radius_m = f64::from(c) * manifest.scale.m_per_block_h / (2.0 * std::f64::consts::PI);
        manifest.planet.insert("radius_m".into(), json!(radius_m));
        for g in &self.grids {
            manifest.grids.insert(g.name.clone(), g.clone());
        }
        let mut total = 0u64;
        for l in self.layers.iter().chain(std::iter::once(&self.strata)) {
            let rel = if l.desc.name.starts_with("strata/") {
                format!("{}.dtr", l.desc.name)
            } else {
                format!("rasters/{}.dtr", l.desc.name)
            };
            let w = write_tile_pack(&out.join(&rel), &l.desc, &l.bytes)?;
            total += w.bytes;
            manifest.layers.push(LayerEntry {
                name: l.desc.name.clone(),
                path: rel,
                grid: l.desc.grid.name.clone(),
                dtype: l.desc.dtype,
                codec: l.desc.codec,
                bytes: w.bytes,
                crc32: w.crc32,
            });
        }

        let classes = json!({ "classes": self.strata_kind.classes() });
        for (rel, value) in [
            ("strata/classes.json", &classes),
            ("spawn/candidates.json", &self.spawn),
        ] {
            let mut bytes = serde_json::to_vec_pretty(value).expect("JSON");
            bytes.push(b'\n');
            let crc32 = write_atomic(&out.join(rel), &bytes)?;
            total += bytes.len() as u64;
            manifest.files.push(FileEntry {
                path: rel.to_owned(),
                bytes: bytes.len() as u64,
                crc32,
            });
        }

        manifest.notes.insert(
            "synthetic".into(),
            json!({
                "kind": self.kind.name(),
                "sampling": "cell (i, j) holds the value at block (origin_block_x + i·r, origin_block_z + j·r)",
                "elevation": "round(16·H(x, z)); H is AnalyticTestTerrain's closed form for wrap-test, and 63 + h/20 of the deposit history's present elevation h (m, bilinear on L1) for deposit-history",
                "climate": "round(10000·p) of AnalyticTestTerrain.raster's formulas, with H from this kind's terrain"
            }),
        );
        manifest
            .notes
            .insert("strata".into(), strata_notes(self.strata_kind));
        if self.kind == Kind::LakeTest {
            manifest.notes.insert("lake".into(), LakeTest::notes());
        }
        Ok(total)
    }
}

/// Rounds half away from zero and saturates to i16 (NaN → nodata).
pub fn quantise(v: f64) -> i16 {
    if v.is_nan() {
        return NODATA_I16;
    }
    // Keep −32768 for nodata.
    v.round()
        .clamp(f64::from(i16::MIN + 1), f64::from(i16::MAX)) as i16
}

/// Samples `f(block_x, block_z)` at every cell, rows split across threads.
fn sample_i16(grid: &GridDesc, f: impl Fn(i64, i64) -> i16 + Sync) -> Raster<i16> {
    let w = grid.w as usize;
    let mut data = vec![0i16; grid.cells()];
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let rows_per = (grid.h as usize).div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (chunk_no, chunk) in data.chunks_mut(rows_per * w).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (r, row) in chunk.chunks_mut(w).enumerate() {
                    let j = (chunk_no * rows_per + r) as i64;
                    let z = grid.block_z(j);
                    for (i, cell) in row.iter_mut().enumerate() {
                        *cell = f(grid.block_x(i as i64), z);
                    }
                }
            });
        }
    });
    Raster {
        grid: grid.clone(),
        data,
    }
}

fn layer_i16(
    name: &str,
    units: &str,
    scale: f64,
    nodata: Option<i16>,
    grid: &GridDesc,
    codec: Codec,
    ras: &Raster<i16>,
) -> Result<LoadedLayer, FormatError> {
    let mut d = LayerDesc::scalar(name, DType::I16, grid.clone(), codec);
    d.units = units.to_owned();
    d.scale = scale;
    d.nodata = nodata.map(f64::from);
    LoadedLayer::new(d, ras.to_le_bytes())
}

/// The key span of a synthetic horizon: horizon h spans `h·STEP ..= h·STEP + STEP − 1`.
pub const SYNTH_KEY_STEP: u16 = 4096;

/// The horizon stacks: per L1 cell the horizons of `kind`, each top a fixed depth below the
/// surface sampled from the L3 elevation at the same block.
fn strata_layer(
    l1: &GridDesc,
    elevation: &LoadedLayer,
    kind: Strata,
    codec: Codec,
) -> Result<LoadedLayer, FormatError> {
    let depths = kind.depths();
    let k = depths.len();
    let e = &elevation.desc.grid;
    let mut bytes = Vec::with_capacity(l1.cells() * 7 * k);
    for j in 0..l1.h {
        for i in 0..l1.w {
            // Same block, L3 index: (block − origin) / r.
            let bx = l1.block_x(i64::from(i));
            let bz = l1.block_z(i64::from(j));
            let ei = (bx - e.origin_block_x) / i64::from(e.r);
            let ej = (bz - e.origin_block_z) / i64::from(e.r);
            let surface = elevation.raw(ei, ej) as i32;
            for d in depths {
                let top = (surface - d * 16).max(i32::from(i16::MIN + 1)) as i16;
                bytes.extend_from_slice(&top.to_le_bytes());
            }
            for h in 0..k {
                bytes.push(kind.class_index(i, h));
            }
            // Horizon h's key span is the same in every cell (format 0.2): the column rule
            // correlates the synthetic stacks by index.
            for side in [0u16, SYNTH_KEY_STEP - 1] {
                for h in 0..k as u16 {
                    bytes.extend_from_slice(&(h * SYNTH_KEY_STEP + side).to_le_bytes());
                }
            }
        }
    }
    let mut d = LayerDesc::scalar("strata/horizons", DType::HorizonStack, l1.clone(), codec);
    d.units = "block/16 top, class index, key top, key base".into();
    d.scale = 1.0 / 16.0;
    d.stack_depth = Some(k as u32);
    LoadedLayer::new(d, bytes)
}

fn strata_notes(kind: Strata) -> Value {
    match kind {
        Strata::Placeholder => json!({
            "placeholder": true,
            "kind": kind.name(),
            "grid": "L1",
            "horizons": [
                {"class": STRATA_CLASSES[0], "top": "surface (the L3 elevation at the cell's block)"},
                {"class": STRATA_CLASSES[1], "top": "surface − 6 blocks"},
                {"class": STRATA_CLASSES[2], "top": "surface − 40 blocks, down to min_y"}
            ]
        }),
        Strata::EveryClass => json!({
            "placeholder": true,
            "kind": kind.name(),
            "grid": "L1",
            "horizons": "8 per cell; horizon h's top is the surface (the L3 elevation at the cell's block) − 10·h blocks; the last reaches down to min_y",
            "class_rule": "horizon h of L1 cell column i has class index (6·i + h) mod 62, so any eleven neighbouring cell columns hold every class at h ≥ 2"
        }),
        Strata::Basin => json!({
            "placeholder": true,
            "kind": kind.name(),
            "grid": "L1",
            "horizons": "9 per cell, the same in every cell: tops at the surface − 0, 8, 22, 36, 55, 75, 100, 130 and 165 blocks; the last reaches down to min_y",
            "classes": ["sandstone", "shale", "sandstone", "mudstone", "limestone", "shale", "sandstone", "shale", "granite"]
        }),
    }
}

/// Land cells for spawn: dry, above sea level + 2, locally flat (≤ 1 block to each L3 neighbour),
/// at least 64 blocks apart, nearest to (0, 0) first; at most 8. Every world's spawn candidates
/// (the planet bake, `bake::terrain`, and the synthetic worlds) come from it.
pub fn spawn_candidates(
    elevation: &impl RasterRead,
    water: &impl RasterRead,
    sea_level: i32,
) -> Value {
    let g = &elevation.desc().grid;
    let c = g.circumference() as i64;
    let stride = 4i64; // every 8 blocks at r 2
    let mut found: Vec<(i64, i64, i64, i64, f64)> = Vec::new(); // (dist², x, z, …)
    for j in (1..i64::from(g.h) - 1).step_by(stride as usize) {
        for i in (0..i64::from(g.w)).step_by(stride as usize) {
            let h = elevation.get(i, j);
            if h < f64::from(sea_level + 2) || !water.get(i, j).is_nan() {
                continue;
            }
            let flat = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .all(|&(di, dj)| (elevation.get(i + di, j + dj) - h).abs() <= 1.0);
            if !flat {
                continue;
            }
            let (x, z) = (g.block_x(i), g.block_z(j));
            found.push((x * x + z * z, x, z, i, h));
        }
    }
    found.sort_by_key(|f| (f.0, f.1, f.2));
    let mut picked: Vec<Value> = Vec::new();
    let mut taken: Vec<(i64, i64)> = Vec::new();
    for (_, x, z, i, h) in found {
        let far = taken.iter().all(|&(tx, tz)| {
            let mut dx = (x - tx).rem_euclid(c);
            dx = dx.min(c - dx);
            dx * dx + (z - tz) * (z - tz) >= 64 * 64
        });
        if !far {
            continue;
        }
        taken.push((x, z));
        let j = (z - g.origin_block_z) / i64::from(g.r);
        picked.push(json!({ "x": x, "z": z, "surface_y": h, "cell": [i, j] }));
        if picked.len() == 8 {
            break;
        }
    }
    json!({
        "rule": "L3 sample points on an 8-block lattice: elevation ≥ sea level + 2, no water, within 1 block of each L3 neighbour; nearest to (0, 0) first, at least 64 blocks apart, at most 8",
        "candidates": picked
    })
}

/// splitmix64, the deterministic generator behind the dumped random cells.
pub struct SplitMix(pub u64);

impl SplitMix {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantise_saturates_and_keeps_nodata_free() {
        assert_eq!(quantise(1.5), 2);
        assert_eq!(quantise(-1.5), -2);
        assert_eq!(quantise(1e9), i16::MAX);
        assert_eq!(quantise(-1e9), i16::MIN + 1);
        assert_eq!(quantise(f64::NAN), NODATA_I16);
    }

    #[test]
    fn kinds_round_trip_their_names() {
        for k in Kind::ALL {
            assert_eq!(Kind::from_name(k.name()), Some(k));
        }
        assert_eq!(Kind::from_name("flat"), None);
        assert!(check_circumference(16_384).is_ok());
        assert!(check_circumference(256).is_err());
        assert!(check_circumference(1000).is_err());
    }

    #[test]
    fn every_class_matches_the_rock_class_data() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../neoforge/src/main/resources/data/deeptime/deeptime/rock_class");
        let mut files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter_map(|n| n.strip_suffix(".json").map(str::to_owned))
            .collect();
        files.sort();
        let mut ours: Vec<String> = EVERY_CLASS.iter().map(|s| s.to_string()).collect();
        ours.sort();
        assert_eq!(files, ours);
    }

    #[test]
    fn any_eleven_columns_hold_every_class_below_the_soil() {
        let k = Strata::EveryClass.depths().len();
        // Horizons 2..k of a column are k − 2 = 6 consecutive classes; the documented eleven
        // columns are the least that cover the whole list (11 × 6 = 66 ≥ 62 > 10 × 6).
        assert_eq!(k - 2, 6);
        assert_eq!(EVERY_CLASS.len().div_ceil(k - 2), 11);
        for i0 in [0u32, 7, 58, 1000] {
            let mut seen = [false; EVERY_CLASS.len()];
            for i in i0..i0 + 11 {
                for h in 2..k {
                    seen[usize::from(Strata::EveryClass.class_index(i, h))] = true;
                }
            }
            assert!(seen.iter().all(|&s| s), "columns {i0}..{}", i0 + 11);
        }
    }

    #[test]
    fn every_class_strata_layer() {
        let w = build(Kind::WrapTest, Strata::EveryClass, 1024, Codec::Raw).unwrap();
        assert_eq!(w.strata.desc.stack_depth, Some(8));
        let (h0, h2) = (w.strata.horizon(3, 10, 0), w.strata.horizon(3, 10, 2));
        assert_eq!(i32::from(h0.top) - i32::from(h2.top), 20 * 16);
        assert_eq!((h0.class, h2.class), (18, 20));
        assert_eq!(h2.key, [2 * SYNTH_KEY_STEP, 3 * SYNTH_KEY_STEP - 1]);
        let p = build(Kind::WrapTest, Strata::Placeholder, 1024, Codec::Raw).unwrap();
        assert_eq!(p.strata.desc.stack_depth, Some(3));
        assert_eq!(p.strata.horizon(3, 10, 1).class, 1);
    }
}
