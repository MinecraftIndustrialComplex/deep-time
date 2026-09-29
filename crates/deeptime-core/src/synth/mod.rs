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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    WrapTest,
    LakeTest,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::WrapTest, Kind::LakeTest];

    pub fn name(self) -> &'static str {
        match self {
            Kind::WrapTest => "wrap-test",
            Kind::LakeTest => "lake-test",
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

/// Samples a synthetic world of `kind` at circumference `c`.
pub fn build(kind: Kind, c: u32, codec: Codec) -> Result<SynthWorld, FormatError> {
    check_circumference(c).map_err(|e| FormatError::Invalid(format!("--size {c}: {e}")))?;
    let terrain: Box<dyn Terrain> = match kind {
        Kind::WrapTest => Box::new(WrapTest::new(c)),
        Kind::LakeTest => Box::new(LakeTest { c }),
    };
    let terrain = terrain.as_ref();

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

    let strata = strata_layer(&l1, &layers[0], codec)?;
    let spawn = spawn_candidates(&layers[0], &layers[1], Scale::default_for(c).sea_level_y);
    Ok(SynthWorld {
        kind,
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

        let classes = json!({ "classes": STRATA_CLASSES });
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
                "elevation": "round(16·H(x, z)); H is AnalyticTestTerrain's closed form for wrap-test",
                "climate": "round(10000·p) of AnalyticTestTerrain.raster's formulas, with H from this kind's terrain"
            }),
        );
        manifest.notes.insert("strata".into(), strata_notes());
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

/// The strata placeholder: per L1 cell three horizons below the surface sampled from the L3
/// elevation at the same block — sandstone from the surface, red beds from 6 blocks down, granite
/// from 40 blocks down to the bottom of the world.
fn strata_layer(
    l1: &GridDesc,
    elevation: &LoadedLayer,
    codec: Codec,
) -> Result<LoadedLayer, FormatError> {
    const K: usize = 3;
    let depths = [0i32, 6 * 16, 40 * 16];
    let e = &elevation.desc.grid;
    let mut bytes = Vec::with_capacity(l1.cells() * 3 * K);
    for j in 0..l1.h {
        for i in 0..l1.w {
            // Same block, L3 index: (block − origin) / r.
            let bx = l1.block_x(i64::from(i));
            let bz = l1.block_z(i64::from(j));
            let ei = (bx - e.origin_block_x) / i64::from(e.r);
            let ej = (bz - e.origin_block_z) / i64::from(e.r);
            let surface = elevation.raw(ei, ej) as i32;
            for d in depths {
                let top = (surface - d).max(i32::from(i16::MIN + 1)) as i16;
                bytes.extend_from_slice(&top.to_le_bytes());
            }
            bytes.extend_from_slice(&[0u8, 1, 2]);
        }
    }
    let mut d = LayerDesc::scalar("strata/horizons", DType::HorizonStack, l1.clone(), codec);
    d.units = "block/16 top, class index".into();
    d.scale = 1.0 / 16.0;
    d.stack_depth = Some(K as u32);
    LoadedLayer::new(d, bytes)
}

fn strata_notes() -> Value {
    json!({
        "placeholder": true,
        "grid": "L1",
        "horizons": [
            {"class": STRATA_CLASSES[0], "top": "surface (the L3 elevation at the cell's block)"},
            {"class": STRATA_CLASSES[1], "top": "surface − 6 blocks"},
            {"class": STRATA_CLASSES[2], "top": "surface − 40 blocks, down to min_y"}
        ]
    })
}

/// Land cells for spawn: dry, above sea level + 2, locally flat (≤ 1 block to each L3 neighbour),
/// at least 64 blocks apart, nearest to (0, 0) first; at most 8.
fn spawn_candidates(elevation: &LoadedLayer, water: &LoadedLayer, sea_level: i32) -> Value {
    let g = &elevation.desc.grid;
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
}
