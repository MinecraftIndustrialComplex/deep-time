// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tests on a real region file: `tests/fixtures/world/region/r.0.0.mca` holds chunks 4..=9 ×
//! 0..=3 of the standalone smoke world (C = 16384, synthetic `wrap-test` world data), copied with
//! `deeptime-inspect fixture`. Chunks 4..=8 are `full` (warm ocean floor near y 15 with coral, an
//! amethyst geode at x 79, z 18); chunk column 9 is the proto-chunk ring (`initialize_light`).

use std::path::{Path, PathBuf};

use deeptime_format::{
    Codec, DType, GridDesc, LayerDesc, LayerEntry, Manifest, Raster, Scale, Writer, write_tile_pack,
};
use deeptime_inspect::blocks;
use deeptime_inspect::nbt;
use deeptime_inspect::region::{self, RegionFile};
use deeptime_inspect::render::{self, Axis, Out, SectionArgs};
use deeptime_inspect::surface::ExcludeOpts;
use deeptime_inspect::world::World;
use deeptime_inspect::worlddata::WorldData;

const C: u32 = 16384;

fn fixture_world() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world")
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("dt-inspect-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The wrap-test terrain's closed form (plan §7.1; `AnalyticTestTerrain.height`).
fn analytic_height(x: f64, z: f64) -> f64 {
    let c = f64::from(C);
    let theta = x * 2.0 * std::f64::consts::PI / c;
    let phi = (std::f64::consts::PI * z / (c / 2.0))
        .clamp(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    let mut h = 63.0
        + 60.0 * (theta - std::f64::consts::PI).cos() * phi.cos()
        + 25.0 * (3.0 * theta + 0.5).sin() * (2.0 * phi).cos();
    let mut dx = x - c / 8.0;
    dx -= c * (dx / c).round();
    let dz = z - c / 32.0;
    h += 450.0 * (-(dx * dx) / (2.0 * 420.0 * 420.0) - (dz * dz) / (2.0 * 260.0 * 260.0)).exp();
    h
}

/// A small world-data folder for C = 16384: only `elevation`, on a coarse grid (r = 16).
fn write_world_data(dir: &Path) {
    let grid = GridDesc::for_planet("L3", C, 16).unwrap();
    let ras = Raster::<i16>::from_fn(grid.clone(), |i, j| {
        let x = grid.block_x(i64::from(i)) as f64;
        let z = grid.block_z(i64::from(j)) as f64;
        (16.0 * analytic_height(x, z)).round() as i16
    });
    let mut desc = LayerDesc::scalar("elevation", DType::I16, grid.clone(), Codec::Lz4Block);
    desc.scale = 1.0 / 16.0;
    desc.units = "block".into();
    let rel = "rasters/elevation.dtr";
    let w = write_tile_pack(&dir.join(rel), &desc, &ras.to_le_bytes()).unwrap();
    let mut m = Manifest::new(
        "test:wrap-test-coarse",
        Writer::this("deeptime-inspect-test"),
        Scale::default_for(C),
    );
    m.grids.insert("L3".into(), grid);
    m.layers.push(LayerEntry {
        name: "elevation".into(),
        path: rel.into(),
        grid: "L3".into(),
        dtype: DType::I16,
        codec: Codec::Lz4Block,
        bytes: w.bytes,
        crc32: w.crc32,
    });
    m.write(dir).unwrap();
}

#[test]
fn fixture_is_small_and_complete() {
    let path = fixture_world().join("region/r.0.0.mca");
    assert!(std::fs::metadata(&path).unwrap().len() <= 2 * 1024 * 1024);
    let w = World::load_area(&fixture_world(), None).unwrap();
    assert!(w.errors.is_empty(), "{:?}", w.errors);
    assert_eq!(w.chunks.len(), 24);
    assert_eq!(w.full_chunks().len(), 20);
    assert_eq!(w.chunks[&(9, 1)].status, "initialize_light");
    let c = &w.chunks[&(4, 1)];
    assert_eq!(
        (c.min_y(), c.max_y()),
        (-352, 640),
        "Deep Time's build range"
    );
}

#[test]
fn known_blocks_decode() {
    let w = World::load_area(&fixture_world(), None).unwrap();
    let name = |x, y, z| w.reg.name(w.block(x, y, z)).to_owned();
    assert_eq!(name(79, 18, 18), "minecraft:budding_amethyst");
    assert_eq!(name(79, 22, 18), "minecraft:smooth_basalt");
    assert_eq!(name(79, 23, 18), "minecraft:water");
    assert_eq!(name(150, 19, 20), "minecraft:horn_coral_block");
    assert_eq!(name(150, 18, 20), "minecraft:sand");
    assert_eq!(w.terrain_top(79, 18), Some(22));
    assert_eq!(
        w.terrain_top(150, 20),
        Some(18),
        "coral and sea pickles are features"
    );
    assert_eq!(w.water_top(150, 20), Some(62));
}

#[test]
fn heightmaps_agree_with_blocks() {
    let w = World::load_area(&fixture_world(), None).unwrap();
    let mut checked = 0;
    let mut floor_equal = 0;
    for &(cx, cz) in &w.full_chunks() {
        let c = &w.chunks[&(cx, cz)];
        for lz in 0..16 {
            for lx in 0..16 {
                let (x, z) = (cx * 16 + lx as i32, cz * 16 + lz as i32);
                let scanned = c.scan_down(&w.reg, lx, lz, c.max_y() - 1, blocks::AIR);
                assert_eq!(
                    c.heightmap_top("WORLD_SURFACE", lx, lz),
                    scanned,
                    "WORLD_SURFACE at {x},{z}"
                );
                // OCEAN_FLOOR also counts solid features (coral blocks, sea pickles): the terrain
                // surface is it or lies under features only.
                let floor = c.heightmap_top("OCEAN_FLOOR", lx, lz).unwrap();
                let terrain = w.terrain_top(x, z).unwrap();
                assert!(
                    terrain <= floor,
                    "terrain {terrain} above OCEAN_FLOOR {floor} at {x},{z}"
                );
                for y in terrain + 1..=floor {
                    assert!(
                        w.flags(x, y, z) & blocks::NOT_TERRAIN != 0,
                        "{} at {x},{y},{z} between the terrain and OCEAN_FLOOR",
                        w.reg.name(w.block(x, y, z))
                    );
                }
                floor_equal += usize::from(terrain == floor);
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 20 * 256);
    // A warm-ocean floor: coral and sea pickles cover about a third of it.
    assert!(
        floor_equal * 2 >= checked,
        "{floor_equal} of {checked} equal OCEAN_FLOOR"
    );
}

#[test]
fn a_real_chunk_reads_back_in_every_compression() {
    let r = RegionFile::open(&fixture_world().join("region/r.0.0.mca")).unwrap();
    let (lx, lz) = (5, 2);
    let raw = r.raw_chunk(lx, lz).unwrap().unwrap();
    assert_eq!(raw.compression, 2, "the game writes zlib by default");
    let bytes = region::decompress(raw.compression, &raw.payload).unwrap();
    let tag = r.chunk_nbt(lx, lz).unwrap().unwrap();
    for kind in [1u8, 3, 4] {
        let payload = match kind {
            1 => {
                use std::io::Write;
                let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
                e.write_all(&bytes).unwrap();
                e.finish().unwrap()
            }
            3 => bytes.clone(),
            _ => region::encode_lz4_block_stream(&bytes, 1 << 16),
        };
        let back = region::decompress(kind, &payload).unwrap();
        assert_eq!(nbt::parse(&back).unwrap().1, tag, "compression {kind}");
    }
}

#[test]
fn compare_against_world_data() {
    let wd_dir = scratch("wd");
    write_world_data(&wd_dir);
    let wd = WorldData::open(&wd_dir).unwrap();
    let out = scratch("compare");
    let v = render::compare(
        &fixture_world(),
        &wd,
        None,
        ExcludeOpts::default(),
        &Out {
            dir: out.clone(),
            scale: 1,
        },
    )
    .unwrap();
    assert_eq!(v["columns_total"], 20 * 256);
    let compared = v["columns_compared"].as_u64().unwrap();
    assert!(compared >= 4000, "compared {compared}");
    // This patch of ocean floor is the bumpiest of the smoke world (aquifer barrier stone,
    // a geode: 4 % of its columns lie 2-6 blocks high), so p99 here is not the world's; the
    // bulk must still match exactly.
    let exact = v["histogram"]["0"].as_u64().unwrap();
    assert!(
        exact * 100 >= compared * 85,
        "{exact} of {compared} exact: {v:#}"
    );
    assert_eq!(v["p50_abs_diff_blocks"], 0);
    assert!(
        v["excluded"]["carved"].as_u64().unwrap() > 0,
        "pits are left out"
    );
    assert!(out.join("compare.json").is_file());
    assert!(out.join("compare-map.png").is_file());
    assert!(out.join("compare-hist.png").is_file());
    // The geode is a bump, not a pit: it stays in the comparison as measured > expected.
    let geode = v["worst"].as_array().unwrap().iter().any(|p| {
        (74..=84).contains(&p["x"].as_i64().unwrap())
            && (13..=23).contains(&p["z"].as_i64().unwrap())
            && p["measured"].as_i64() > p["expected"].as_i64()
    });
    assert!(geode, "{}", v["worst"]);
}

#[test]
fn section_prints_the_water_surface() {
    let out = scratch("section");
    let args = SectionArgs {
        axis: Axis::X,
        at: 100,
        from: 0,
        to: 63,
        y_from: None,
        y_to: None,
    };
    let mut lines = Vec::new();
    let v = render::section(
        &fixture_world(),
        None,
        &args,
        &Default::default(),
        &Out { dir: out, scale: 2 },
        &mut lines,
    )
    .unwrap();
    assert!(
        lines.contains(&"water_top_y=63 water_block_y=62 columns=64".to_owned()),
        "{lines:?}"
    );
    assert!(Path::new(v["png"].as_str().unwrap()).is_file());
}

#[test]
fn renders_and_regions() {
    let out = scratch("renders");
    let o = Out {
        dir: out.clone(),
        scale: 1,
    };
    render::topdown(&fixture_world(), None, &Default::default(), &o).unwrap();
    render::ore(&fixture_world(), None, &o).unwrap();
    let pal =
        deeptime_inspect::palette::Palette::from_tags(&[
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../neoforge/src/main/resources")
        ])
        .unwrap();
    let rock = render::rock(&fixture_world(), None, &pal, &Default::default(), &o).unwrap();
    assert!(
        rock["classes"]["sandstone"].as_u64().unwrap() > 1000,
        "{}",
        rock["classes"]
    );
    for f in [
        "topdown.png",
        "ore.png",
        "rock.png",
        "topdown.json",
        "ore.json",
        "rock.json",
    ] {
        assert!(out.join(f).is_file(), "{f}");
    }

    // Region span: one populated file outside, one empty one outside.
    let world = scratch("regions");
    std::fs::create_dir_all(world.join("region")).unwrap();
    let src = fixture_world().join("region/r.0.0.mca");
    std::fs::copy(&src, world.join("region/r.0.0.mca")).unwrap();
    std::fs::copy(&src, world.join("region/r.-17.3.mca")).unwrap();
    std::fs::write(world.join("region/r.16.0.mca"), b"").unwrap();
    let mut lines = Vec::new();
    render::regions(&world, i64::from(C), &mut lines).unwrap();
    assert_eq!(lines.last().unwrap(), "outside=1", "{lines:?}");
    assert!(
        lines.iter().any(|l| l.contains("outside_empty=1")),
        "{lines:?}"
    );
}
