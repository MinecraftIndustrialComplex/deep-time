// SPDX-License-Identifier: LGPL-3.0-or-later
use deeptime_format::rivers::{Chain, Mouth, RiverVertex, write_rivers};

use super::*;
use crate::hydro::HydroParams;

const W: usize = 128;
const H: usize = 64;
/// Blocks across the planet: a 16-block cell grid of W × H.
const C: u32 = (W * 16) as u32;

/// A planet 128 × 64 cells wide: land sloping south to a sea (rows 54 and on) and in towards a
/// valley down column 64, a closed bowl five blocks deep in the north-west, a warm, moderately wet
/// climate (700 mm, 15 °C: 160 mm of runoff; the bowl gets more than it evaporates, so it fills).
fn world() -> WorldState {
    world_in(700.0, 15.0)
}

/// [`world`] with `rain` mm a year at a mean of `t` °C.
fn world_in(rain: f32, t: f32) -> WorldState {
    let grid = GridDesc::for_planet("L1", C, 16).unwrap();
    assert_eq!((grid.w as usize, grid.h as usize), (W, H));
    let mut world = WorldState::new(1, C, 0.0);
    let elevation: Vec<f32> = (0..W * H)
        .map(|k| {
            let (i, j) = ((k % W) as f32, (k / W) as f32);
            if j >= 54.0 {
                return 40.0;
            }
            let base = 64.0 + 0.6 * (53.0 - j) + 0.35 * (i - 64.0).abs();
            let bowl = (20..=26).contains(&(k % W)) && (10..=16).contains(&(k / W));
            // A bowl with a rim: the cells inside it are sunk 5 blocks (100 m).
            if bowl && (21..=25).contains(&(k % W)) && (11..=15).contains(&(k / W)) {
                base - 5.0
            } else {
                base
            }
        })
        .collect();
    let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
        world.fields.insert(
            name.into(),
            Field {
                grid: grid.clone(),
                data,
            },
        );
    };
    put(&mut world, "elevation", elevation);
    put(&mut world, T_FIELD, vec![t; W * H]);
    put(&mut world, P_FIELD, vec![rain; W * H]);
    world
}

const SIZE: (u32, u32) = (W as u32, H as u32);

fn at(v: &WaterView, i: usize, j: usize) -> usize {
    assert!(i < v.w && j < v.h);
    j * v.w + i
}

fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("dt-water-view-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn a_valley_carries_a_river_that_grows_to_the_sea_and_a_bowl_holds_a_lake() {
    let (v, rivers) = derive_with_rivers(&world(), &ViewScale::default(), SIZE).unwrap();
    assert_eq!(v.info.source, Source::Derived);
    assert!(!rivers.chains.is_empty());
    // The river runs down column 64: nothing on the dry slopes, a river in the valley, bigger
    // downstream.
    let far = at(&v, 5, 30);
    assert_eq!((v.river[far], v.q[far]), (0.0, 0.0));
    let (upper, lower) = (at(&v, 64, 28), at(&v, 64, 50));
    assert!(v.river[upper] >= 0.5 && v.river[lower] >= 0.5);
    assert!(
        v.q[lower] > 2.0 * v.q[upper],
        "{} {}",
        v.q[upper],
        v.q[lower]
    );
    assert!(
        v.q[upper] >= 2.0,
        "a river carries at least the stream threshold"
    );
    // Not on the sea (rows 54 and on are 23 blocks under it).
    for j in 56..H {
        assert_eq!(v.river[at(&v, 64, j)], 0.0, "row {j}");
    }
    // The bowl is a lake, and only the bowl.
    assert_eq!(v.lake[at(&v, 23, 13)], 1.0);
    assert_eq!(v.lake[at(&v, 5, 30)], 0.0);
    assert!(
        v.info.lake_px >= 9.0 && v.info.lake_px < 60.0,
        "{}",
        v.info.lake_px
    );
}

#[test]
fn thresholds_follow_the_presets_hydrology() {
    let w = world();
    // A stream needs 100 m³/s: the valley's river is gone, or at least far shorter.
    let mut view = ViewScale::default();
    view.hydro.river_min_discharge_m3s = 100.0;
    let strict = derive(&w, &view, SIZE).unwrap();
    let normal = derive(&w, &ViewScale::default(), SIZE).unwrap();
    let px = |v: &WaterView| v.river.iter().filter(|&&c| c >= 0.5).count();
    assert!(
        px(&strict) < px(&normal) / 2,
        "{} {}",
        px(&strict),
        px(&normal)
    );
    // Hydrology off: no water at all, and no layer to draw.
    view.hydro.enabled = false;
    assert!(derive(&w, &view, SIZE).is_none());
    let mut no_elevation = world();
    no_elevation.fields.remove("elevation");
    assert!(derive(&no_elevation, &ViewScale::default(), SIZE).is_none());
}

#[test]
fn rivers_are_wider_where_the_flow_is_bigger() {
    let (v, _) = derive_with_rivers(&world(), &ViewScale::default(), SIZE).unwrap();
    // The river's cross-section, in coverage, near the head and near the mouth.
    let width = |j: usize| (0..W).map(|i| v.river[at(&v, i, j)]).sum::<f32>();
    assert!(width(50) > width(28), "{} {}", width(50), width(28));
}

#[test]
fn water_is_drawn_over_the_satellite_after_its_shading_and_leaves_dry_land_alone() {
    let ramps = Ramps::builtin();
    let (v, _) = derive_with_rivers(&world(), &ViewScale::default(), SIZE).unwrap();
    let grey = [120u8, 110, 90, 255];
    let mut rgba: Vec<u8> = (0..W * H).flat_map(|_| grey).collect();
    // A pixel without data is left alone.
    rgba[at(&v, 64, 40) * 4 + 3] = 0;
    let before = rgba.clone();
    v.overlay(&mut rgba, &ramps);
    let px = |k: usize| {
        [
            rgba[k * 4],
            rgba[k * 4 + 1],
            rgba[k * 4 + 2],
            rgba[k * 4 + 3],
        ]
    };
    // Dry land: unchanged.
    assert_eq!(px(at(&v, 5, 30)), grey);
    // A river: bluer than the ground, and the larger the river the darker blue it is.
    let river = px(at(&v, 64, 50));
    assert!(river[2] > river[0] + 40 && river[3] == 255, "{river:?}");
    assert_eq!(
        px(at(&v, 64, 40)),
        [120, 110, 90, 0],
        "no data stays no data"
    );
    assert_eq!(&before[at(&v, 64, 40) * 4..][..4], &[120, 110, 90, 0]);
    // A lake: nearly the lake colour (95 %).
    let lake = px(at(&v, 23, 13));
    for c in 0..3 {
        let want = f32::from(grey[c]) * 0.05 + f32::from(LAKE[c]) * 0.95;
        assert!((f32::from(lake[c]) - want).abs() <= 1.0, "{lake:?}");
    }
}

#[test]
fn land_ice_hides_the_water_on_the_satellite_view_but_not_in_the_layer() {
    let ramps = Ramps::builtin();
    let view = ViewScale::default();
    let mut w = world();
    let grid = w.fields["elevation"].grid.clone();
    // Ice 100 m thick over the valley's west half and the bowl; none east of it.
    let ice = (0..W * H)
        .map(|k| if k % W < 64 { 100.0 } else { 0.0 })
        .collect();
    w.fields
        .insert(satellite::input::ICE.into(), Field { grid, data: ice });
    let (v, _) = derive_with_rivers(&w, &view, SIZE).unwrap();
    let grey = [120u8, 110, 90, 255];
    let mut rgba: Vec<u8> = (0..W * H).flat_map(|_| grey).collect();
    v.overlay(&mut rgba, &ramps);
    let px = |k: usize| {
        [
            rgba[k * 4],
            rgba[k * 4 + 1],
            rgba[k * 4 + 2],
            rgba[k * 4 + 3],
        ]
    };
    // The lake (x 23) lies under the ice: not drawn. The river (column 64) is just off it: drawn.
    assert_eq!(px(at(&v, 23, 13)), grey);
    assert!(
        v.lake[at(&v, 23, 13)] == 1.0,
        "the lake is there, only hidden"
    );
    assert_ne!(px(at(&v, 64, 50)), grey);
    // The layer shows them all.
    let layer = v.render(&ramps, &view).unwrap();
    assert_eq!(&layer[at(&v, 23, 13) * 4..][..3], &LAKE);
}

#[test]
fn the_layer_has_land_sea_lake_and_a_river_on_the_ramp() {
    let ramps = Ramps::builtin();
    let view = ViewScale::default();
    let (v, _) = derive_with_rivers(&world(), &view, SIZE).unwrap();
    let rgba = v.render(&ramps, &view).unwrap();
    let px = |k: usize| {
        [
            rgba[k * 4],
            rgba[k * 4 + 1],
            rgba[k * 4 + 2],
            rgba[k * 4 + 3],
        ]
    };
    assert_eq!(&px(at(&v, 23, 13)), &[LAKE[0], LAKE[1], LAKE[2], 255]);
    assert_eq!(&px(at(&v, 100, 60)), &[SEA[0], SEA[1], SEA[2], 255]);
    // A river pixel is exactly the ramp's colour at its discharge, so the workbench can read the
    // discharge back from the colour.
    let k = at(&v, 64, 50);
    let want = ramps.ramps[RAMP].colour(f64::from(v.q[k]).log10());
    assert_eq!(&px(k)[..3], &want);
    // Land is the layer's pale neutral, softly shaded: bright, and not blue.
    let land = px(at(&v, 5, 30));
    assert!(land[0] >= 180 && land[0] >= land[2], "{land:?}");
    assert_eq!(land[3], 255);
    // Every pixel is opaque (all of this planet has terrain).
    assert!(rgba.chunks(4).all(|p| p[3] == 255));
}

#[test]
fn the_ramps_json_keys_are_the_colours_drawn() {
    let v: serde_json::Value = serde_json::from_str(deeptime_format::ramps::RAMPS_JSON).unwrap();
    let keys = &v["ramps"][RAMP]["keys"];
    let hex = |c: [u8; 3]| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
    assert_eq!(keys["land"], hex(LAND));
    assert_eq!(keys["sea"], hex(SEA));
    assert_eq!(keys["lake"], hex(LAKE));
    assert_eq!(keys["frozen_lake"], hex(FROZEN));
    assert_eq!(keys["salt_flat"], hex(PLAYA));
    // The satellite view's pointer reads the three by the same colours.
    let sat = &v["ramps"][satellite::LAYER]["keys"];
    assert_eq!(sat["lake"], hex(LAKE));
    assert_eq!(sat["frozen_lake"], hex(FROZEN));
    assert_eq!(sat["salt_flat"], hex(PLAYA_SATELLITE));
    let salt = satellite::CLASSES.iter().position(|&c| c == "Salt flat").unwrap();
    assert_eq!(
        Ramps::builtin().ramps[satellite::LAYER].colour(salt as f64 + 0.5),
        PLAYA_SATELLITE
    );
    assert_eq!(v["layers"][LAYER], RAMP);
    // The river ramp runs over log10 of the discharge from the stream threshold (2 m³/s).
    let first = v["ramps"][RAMP]["stops"][0][0].as_f64().unwrap();
    assert!((first - 2.0f64.log10()).abs() < 0.05, "{first}");
}

#[test]
fn a_cold_lake_is_drawn_as_ice_on_both_views() {
    let ramps = Ramps::builtin();
    let view = ViewScale::default();
    // Mean −5 °C, a +10 °C summer: the bowl fills with meltwater and is frozen most of the year.
    let mut w = world_in(700.0, -5.0);
    let grid = w.fields["elevation"].grid.clone();
    for (name, t) in [
        (crate::climate::field::T_WARM, 10.0),
        (crate::climate::field::T_COLD, -20.0),
    ] {
        w.fields
            .insert(name.into(), Field::filled(grid.clone(), t));
    }
    let v = derive(&w, &view, SIZE).unwrap();
    let k = at(&v, 23, 13);
    assert_eq!((v.frozen[k], v.lake[k]), (1.0, 0.0));
    assert!(v.info.frozen_px >= 9.0 && v.info.frozen_px == v.info.lake_px);
    let layer = v.render(&ramps, &view).unwrap();
    assert_eq!(&layer[k * 4..][..3], &FROZEN);
    let grey = [120u8, 110, 90, 255];
    let mut rgba: Vec<u8> = (0..W * H).flat_map(|_| grey).collect();
    v.overlay(&mut rgba, &ramps);
    for c in 0..3 {
        let want = f32::from(grey[c]) * 0.05 + f32::from(FROZEN[c]) * 0.95;
        assert!((f32::from(rgba[k * 4 + c]) - want).abs() <= 1.0);
    }
    // The freezing point is the preset's.
    let mut hardy = ViewScale::default();
    hardy.hydro.lake_freeze_mean_c = -10.0;
    let open = derive(&w, &hardy, SIZE).unwrap();
    assert_eq!((open.frozen[k], open.lake[k]), (0.0, 1.0));
    // Where the summer never thaws, there is no water at all.
    w.fields.insert(
        crate::climate::field::T_WARM.into(),
        Field::filled(grid, -3.0),
    );
    let never = derive(&w, &view, SIZE).unwrap();
    assert_eq!(never.info.lake_px, 0.0);
    assert!(never.river.iter().all(|&c| c == 0.0));
}

#[test]
fn a_desert_bowl_is_a_salt_flat_or_dry_and_the_old_rule_fills_it() {
    let ramps = Ramps::builtin();
    let view = ViewScale::default();
    // Hot (30 °C: 1,800 mm a year off open water). The wetter it is, the more lake the bowl holds;
    // somewhere between none and a lake its catchment sends it only enough to wet its floor.
    let runs: Vec<(f32, WaterView)> = (4..=40)
        .map(|k| {
            let rain = 25.0 * k as f32;
            (rain, derive(&world_in(rain, 30.0), &view, SIZE).unwrap())
        })
        .collect();
    assert!(
        runs.windows(2)
            .all(|p| p[1].1.info.lake_px >= p[0].1.info.lake_px),
        "more rain never holds less lake"
    );
    assert_eq!(runs[0].1.info.lake_px, 0.0);
    assert!(runs.last().unwrap().1.info.lake_px >= 9.0);
    let (rain, v) = runs
        .iter()
        .find(|(_, v)| v.info.lake_px == 0.0 && v.info.playa_px >= 1.0)
        .expect("a rainfall that leaves a playa and no lake");
    let w = world_in(*rain, 30.0);
    let k = (0..W * H)
        .max_by(|&a, &b| v.playa[a].total_cmp(&v.playa[b]))
        .unwrap();
    assert!((21..=25).contains(&(k % W)) && (11..=15).contains(&(k / W)));
    let layer = v.render(&ramps, &view).unwrap();
    assert_eq!(&layer[k * 4..][..3], &PLAYA);
    // Drier still: nothing reaches the floor.
    let dry = derive(&world_in(60.0, 30.0), &view, SIZE).unwrap();
    assert_eq!((dry.info.lake_px, dry.info.playa_px), (0.0, 0.0));
    // The rule before the water balance: full to the rim whatever the climate.
    let mut old = ViewScale::default();
    old.hydro.lake_balance = false;
    let full = derive(&w, &old, SIZE).unwrap();
    assert_eq!(full.lake[at(&full, 23, 13)], 1.0);
}

#[test]
fn the_bakes_file_draws_the_same_rivers_as_the_derivation_wrote() {
    let w = world();
    let view = ViewScale::default();
    let (derived, rivers) = derive_with_rivers(&w, &view, SIZE).unwrap();
    let d = dir("same");
    write_rivers(&d, &rivers).unwrap();
    // No manifest layer: rivers only.
    let baked = from_bake(&w, &view, &d, None, SIZE).unwrap().unwrap();
    assert_eq!(baked.info.source, Source::Bake);
    assert_eq!(baked.river, derived.river);
    assert_eq!(baked.q, derived.q);
    assert!(baked.lake.iter().all(|&c| c == 0.0));
    assert_eq!(baked.info.chains, derived.info.chains);
    // A folder with no rivers file (a bake with `hydro.enabled` false) has no bake water.
    let empty = dir("none");
    assert!(from_bake(&w, &view, &empty, None, SIZE).unwrap().is_none());
    // A named water_level layer that is not there is an error, not a silent dry map.
    assert!(from_bake(&w, &view, &d, Some("rasters/missing.dtl"), SIZE).is_err());
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(empty);
}

#[test]
fn a_river_crosses_the_seam() {
    let w = world();
    let view = ViewScale::default();
    // A chain across the seam at row 20 (block z = -C/4 + 20·16), 8 blocks each side.
    let z = -(C as f32) / 4.0 + 20.0 * 16.0;
    let half = C as f32 / 2.0;
    let mut rivers = Rivers::empty(C);
    for (k, x) in [half - 8.0, half, half + 8.0].into_iter().enumerate() {
        rivers.vertices.push(RiverVertex {
            x,
            z,
            w: 100.0 - k as f32,
            hw: 1.0,
            depth: 1.0,
            q: 50.0,
        });
    }
    rivers.chains.push(Chain {
        first: 0,
        count: 3,
        mouth: Mouth::Sea,
        into: u32::MAX,
    });
    rivers.build_index();
    let d = dir("seam");
    write_rivers(&d, &rivers).unwrap();
    let v = from_bake(&w, &view, &d, None, SIZE).unwrap().unwrap();
    // The river sits at x = 127.5 → 128.5 px: pixels 127 and 0 hold it, the middle of the map not.
    assert!(v.river[at(&v, 127, 20)] >= 0.5);
    assert!(v.river[at(&v, 0, 20)] >= 0.5);
    assert_eq!(v.river[at(&v, 64, 20)], 0.0);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn lake_cover_averages_the_cells_under_a_pixel() {
    // 8 × 4 cells under a 4 × 2 picture: pixel (px, py) is centred on cell (2px, 2py) and covers
    // the 2 × 2 cells (2px − 1 ..= 2px) × (2py − 1 ..= 2py). Pixel (1, 1): cells (1, 1), (2, 1),
    // (1, 2) and (2, 2) = 9, 10, 17, 18 of which three are wet.
    let wet = |c: usize| matches!(c, 9 | 10 | 17);
    let cover = lake_cover(8, 4, &wet, (4, 2));
    assert_eq!(cover[4 + 1], 0.75);
    assert_eq!(cover.iter().filter(|&&c| c > 0.0).count(), 1);
    // The same grid at the picture's own size is the grid itself.
    let same = lake_cover(4, 2, &|c| c == 5, (4, 2));
    assert_eq!(same, [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
}

#[test]
fn a_fine_grid_is_averaged_down_to_the_working_size() {
    // L2 of a 16k planet: 2048 × 1024 = 2.1 M cells, over MAX_CELLS.
    let grid = GridDesc::for_planet("L2", 16384, 8).unwrap();
    assert!(grid.cells() > MAX_CELLS);
    let field = Field::filled(grid, 70.0);
    let (g, elev) = work_terrain(&field, 63 * 16);
    assert_eq!((g.w, g.h, g.r), (1024, 512, 16));
    assert_eq!(elev.len(), g.cells());
    assert!(elev.iter().all(|&e| e == 70 * 16));
}

#[test]
fn the_water_view_is_the_same_on_one_thread_and_on_many() {
    let w = world();
    let view = ViewScale::default();
    let run = |threads: usize| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| derive(&w, &view, SIZE).unwrap())
    };
    let (a, b) = (run(1), run(3));
    assert_eq!(
        (a.river.clone(), a.q.clone(), a.lake.clone()),
        (b.river, b.q, b.lake)
    );
}

#[test]
fn hydro_params_default_is_the_bakes() {
    let h = HydroParams::default();
    assert_eq!(
        (h.river_min_discharge_m3s, h.river_min_area_km2),
        (2.0, 50.0)
    );
}
