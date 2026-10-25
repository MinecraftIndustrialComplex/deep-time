<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->
# Deep Time world-data format 0.6

The contract between the Rust writer (`crates/deeptime-format`) and the Java reader
(`neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime/worlddata`). Plan §5.3 and §5.7
give the reasons; this file gives the bytes. All integers are **little-endian**. Offsets are
bytes from the start of the file. Before 1.0 a reader accepts exactly its own `0.<minor>` and
refuses anything else with `world data <found>, this Deep Time reads <own>; before 1.0 worlds are
not migrated`.

The version lives in one place: `[workspace.metadata.deeptime] format_version` in the workspace
`Cargo.toml`. `deeptime_format::FORMAT_VERSION` and Java's `FormatVersion.CURRENT` mirror it, and a
test on each side fails when they drift.

**0.1 (P1-08)** over 0.0: every header's minor is 1; a planet run bakes every chunk-time layer of
plan §5.3 into its folder (the table below, `notes.bake`); the terrain height includes the detail
term a baked world declares ("The terrain height"); the first horizon of a baked stack reaches over
the amplified terrain ("The P1-08 bake"); and the column evaluator both sides implement is specified
("Columns"). Nothing a 0.0 world held changed its bytes, only the minor.

**0.2** over 0.1 (three changes, released together): every header's minor is 2.
- *Monthly climate (P1-25):* a planet run with a climate section bakes the climate stage's twelve
  months on L1 (`climate/month/t_sl_c_01..12`, `climate/month/p_share_01..12`,
  `notes.bake.climate.monthly`), and the climate at a block that Deep Time's public climate API
  answers is specified ("The climate at a block").
- *Continuous strata:* a `horizon_stack` cell carries each horizon's stratigraphic key span (7K bytes
  instead of 3K); the column rule correlates the stacks of the 4 × 4 cells around a column by those
  keys, with B-spline weights, and intertongues facies changes, so beds run on across cells instead
  of stepping at cell edges; the bake smooths the stacks laterally ("The 0.2 bake"); a column's
  horizons carry a unit key (`[top y, class, unit]` in "Columns").
- *Rivers and lakes (P1-22):* `water_level` holds the lakes a run's bake finds (it was all nodata),
  the new `rivers/rivers.dtrv` (`DTRV`) holds the river network ("Rivers"), the preset gains a
  `hydro` section, and the column evaluator decides each column's water ("Water columns"; `hydro`
  in "Columns" and `parity/columns.json`). A reader treats a folder without `rivers/rivers.dtrv` as
  having no rivers (the synthetic worlds have none).
- *Lakes sized by the climate, lake ice and playas (2026-10-01):* a bake writes a lake's level from
  its water balance, so `water_level` may hold a level below the basin's rim and fewer, smaller
  lakes in dry and frozen climates ("Rivers", how the bake builds it). The game ices a lake by the
  climate's rule, read from `climate/t_mean_c` and `notes.bake.water.params.lake_freeze_mean_c`
  ("Lake ice"), and the new layer `playa` holds the salt of basins too dry for a lake ("Playas").
Every other file's bytes are as in 0.1.

**0.3** over 0.2 (P2-01, folds and faults): every header's minor is 3. A baked planet may hold
`structure/events.dtst` (`DTST`), the folds, faults and small intrusions its tectonic history left, and the horizon
stack a column shows is then the 0.2 column rule's stack read through those events ("Structural
events", the deformed column rule; `horizons` and `rock` in "Columns" are the deformed ones). A
folder without the file is read as before. The `elevation` of such a world already carries the
young structures' relief (fault scarps, rift valleys, fold ridges: the amplifier adds it); no
reader needs to know.

**0.4** over 0.3 (the equal-area projection; Ben, decisions log 2026-10-02, "Lambert for now"):
every header's minor is 4. The world and every map use **Lambert's cylindrical equal-area
projection** ("Grids"): latitude φ = asin(−z / R) with R = C / 2π, so the poles are at z = ∓R
(∓2,607.6 blocks at C = 16,384) instead of ∓C/4; every grid's rows start at z = −Zo (Zo the smallest
multiple of 64 at or past the pole: 2,624 at 16k, 5,248 at 32k) and H = 2·Zo / r; keyframes are
Lambert maps, π : 1 ("Keyframes"); the rivers', structure and site indexes are framed from z = −Zo
over 2·Zo. Every raster, every latitude in events, sites and the climate API, and every block
position derived from a latitude changed. 0.3 worlds are refused.

**0.5** over 0.4 (the seamless world; Ben, decisions log 2026-10-04: "We'll proceed w/
Petroff–Guyou, if I think distortion is too bad in gameplay, I'll swap later."): every header's
minor is 5. A world is laid out in the projection its preset chose (`projection.kind`, default
`petroff_guyou`; `lambert` keeps 0.4's world). The manifest gains **`projection`**, the world's
projection and the gluing of its edges ("The world's layout"), the data the wrap engine reads. A
Petroff–Guyou world is 2b × b blocks (13,312 × 6,656 at 16k), `scale.circumference_blocks` is its
x period 2b and `scale.m_per_block_h` the true size of its blocks; its grids span the domain edge to
edge and fold through the north and south edges' half-turns (`z_edges`, "Grids"). The bake samples
the simulation on the sphere for every world cell ("The bake's projection pass"); events and
keyframes stay in the simulation's frame (latitude and longitude; Lambert pictures). 0.4 worlds are
refused.

**0.6** over 0.5 (closed ground below sea level by its water balance, HYD-100; the lakes follow-up,
decisions log 2026-10-20): every header's minor is 6. Ground below sea level that the open sea does
not reach is no longer flooded to sea level: the bake gives such a closed basin the water balance's
lake (whose surface may stand below sea level), a playa or nothing, and the new layer
**`closed_basin`** (u8 flag on the `water_level` grid) marks it with an 8-block halo. The column
rule's sea step ("Water columns", step 3) holds only where the layer is 0 or absent; the game's
fluid picker, its water pass, the cave shield and the coast tests read the same layer. Water
`method` is "hyd-100" and `stats.below_sea` is new ("Rivers"). 0.5 worlds are refused.

**0.7, the cube world (Ben, decisions log 2026-10-22: "Fuck it, remove equal are restriction and
move to a cube"):** written by the Rust side for cube worlds only, while the Lambert-frame layout stays 0.6
("Lambert-frame and cube worlds side by side"). A cube world's manifest has `projection.kind` `cube` with
`layout_version` 2 ("The cube world's layout"), names its map table ("The cube map table") and
stores its grids face by face ("Cube grids"). `deeptime synth --kind cube` writes one, and every planet bakes to one.

**Cube only (cube plan §7.4, 2026-10-24; Ben, decisions log 2026-10-24: "Cube only
(Recommended)"):** the Petroff–Guyou layout and the playable Lambert world are gone, and no world is
migrated. `projection.kind` is `cube` or `lambert`, and `lambert` is now only the simulation's own
frame laid out as it is, for the synthetic test worlds and the analytic test terrains (still 0.6),
until the simulation itself moves to the cube (plan phase 4). A `petroff_guyou` manifest is refused
as an unknown kind; its fields `hemisphere_blocks` and `lambda_c_deg` and the grids'
`"z_edges": "half_turn"` are gone. Presets lose `scale.pole_cap_deg` (the pole cap) and
`scale.projection` defaults to `cube`; the generator's `world_data` loses `pole_cap_deg`,
`pole_stop_deg` and the projection `petroff_guyou`; the level types `deeptime:planet`,
`deeptime:planet_32k`, `deeptime:planet_lambert` and `deeptime:planet_lambert_32k` are gone.

**The river file's layout 2 (2026-10-23, the deltas; Ben, decisions log 2026-10-22: "Start now,
format change OK (Recommended)"):** `rivers/rivers.dtrv` carries its own layout number, the last word
of its fixed block, and both readers refuse any other with `river file layout 1, this reader takes
layout 2 (bake the world's water again: deeptime hydro)`. Layout 2 adds each chain's `from` (a
distributary leaves the chain it splits from at a delta; "Rivers"). The world's `format_version` and
every header's minor are unchanged: 0.7 is the cube world's, so the river file numbers its own
layouts, as the cube's `layout_version` does. A layout-1 file (no number; 48 fixed bytes, 16-byte
chains) is recognised by its header length, 128. No file is migrated.

### Lambert-frame and cube worlds side by side (until cube plan phase 4)

Every planet is a cube world since cube plan §7.4, but the synthetic test worlds (`deeptime synth`
without `--kind cube`, the deposit and river fixtures) and the analytic test terrains are still laid
out in the simulation's Lambert frame, so two format versions live together until the simulation
moves to the cube (plan phase 4):

- A **cube world's `manifest.json` says `"format_version": "0.7"`**, and only a cube world's does.
  A Lambert-frame world keeps 0.6. Every file header of either kind keeps
  `format_minor` 6 (tile packs, the sectioned files, keyframes, events): none of those files
  changed, and the manifest is what a reader checks first.
- The Rust readers (`Manifest::read`) accept 0.6 and 0.7 and refuse the mismatch either way: a 0.7
  manifest must have a `cube` layout and a cube layout must say 0.7 (`format_version 0.6 on a cube
  world, which is 0.7`). They then refuse a cube world whose map table they do not ship by name and
  sha256, or whose rotation is not one, before anything else is read. Their version refusal names
  both: `world data 0.9, this Deep Time reads 0.6 (Lambert-frame worlds) and 0.7 (cube worlds);
  before 1.0 worlds are not migrated`.
- The game (Java, `worlddata/FormatVersion`) reads both with the same pairing rule and the same
  refusal message: a cube world through the level types `deeptime:planet_cube` and
  `deeptime:planet_cube_32k` (the only Deep Time world types), a Lambert-frame world through the dev
  level types `deeptime:test_planet_lambert` and `deeptime:test_planet_lambert_32k`, which only the
  GameTest resources and the repo datapack `tools/smoke/datapacks/deeptime-lambert-frame` define.
- Phase 4 moves the simulation, and with it the synthetic worlds, to the cube; then everything is
  0.7 (the workspace's `format_version`, every header's minor) and 0.6 worlds are refused.

## Folder layout

```text
<world-data>/
  manifest.json                 written last (tmp + fsync + rename); present ⇒ folder complete
  rasters/elevation.dtr         L3, i16, 1/16 block
  rasters/water_level.dtr       L3, i16, 1/16 block, nodata −32768 (lake surfaces; grid L3h on 32k full)
  rivers/rivers.dtrv            optional: the river network (P1-22, "Rivers")
  rasters/detail/amplitude.dtr  L3, u8, 1/16 block: amplitude of the sub-cell detail term (P1-06)
  rasters/delta.dtr             L3, i16, 1/16 block: what the delta plains added to elevation ("Rivers", deltas)
  rasters/floodplain.dtr        L3, i16, 1/16 block, nodata −32768: what the floodplains moved elevation by ("Rivers", floodplains)
  rasters/fan.dtr               L3, i16, 1/16 block: what the alluvial fans added to elevation ("Rivers", fans)
  rasters/climate/<name>.dtr    L2, i16, vanilla parameter × 10,000
                                (<name> ∈ temperature, vegetation, continents, erosion, ridges)
  rasters/climate/t_mean_c.dtr  L1, i16, 0.01 °C            ┐
  rasters/climate/p_mm.dtr      L1, u16, mm/yr              │ a baked planet run only
  rasters/climate/koppen.dtr    L1, u8, Köppen code index   │ (P1-08, "The bake" below)
  rasters/climate/ice_cover.dtr L2, u8, blocks of snow/ice  │ ("The ice cover")
  rasters/climate/koppen_l2.dtr L2, u8, the biome inputs' Köppen │
  rasters/climate/wetland.dtr   L2, u8, 0 none 1 swamp 2 cold │ ("The wetland flag")
  rasters/climate/month/*.dtr   L1: t_sl_c_NN i16 0.01 °C,  │ the twelve months (0.2, P1-25)
                                p_share_NN u8 1/40          │
  rasters/surface/*.dtr         L2, u8                      │
  rasters/meta/*.dtr            L1, u16 (meta/heat_flow_mw_m2: u16, 0.1 mW/m², "Geothermal features") ┘
  strata/horizons.dtr           L1, horizon_stack (7K bytes a cell), K = 3 in the synthetic worlds (8 with
                                --strata every-class, 32 in a baked run)
  strata/lift.dtr               L1, i16 1/16 block: how far each cell's stack rose after the history
                                ("Structural events", "The lift"; a baked run with structural events)
  strata/classes.json           class index → rock class name
  spawn/candidates.json         land cells for spawn selection
  volcanoes/active.json         the volcanoes active at the present ("Lava")
  volcanoes/standing.json       the volcanoes standing at the present, alive or dead ("Standing
                                volcanoes"; read by the bake's projection pass, not by the game)
  impacts/craters.json          the impact craters at the present ("Impact craters")
  climate_mapping.json          written by the game at the world's first start (P1-09, plan §7.2; not listed
                                in the manifest): the tables from the baked biome inputs to the pack's
                                (see "The climate mapping"), and, with TerraBlender, each province's biome
                                region ("The geologic provinces and TerraBlender's regions")
  parity/columns.json           optional: the column evaluator on 500 columns (`deeptime parity`; not listed
                                in the manifest, never read by the game)
  preset.json                   the exact preset of a run (canonical JSON; `deeptime run`, the game)
  events/index.json             the event log's writers and segments (EVENTS.md)
  events/<writer>-<n>.dtev      event segments (EVENTS.md); geology is writer 1
  sites/sites.dtsr              the site registry's records
  sites/index.dtsi              512-block grid → site ids
  bodies/bodies.dtb             deposit bodies (P1-14): parametric primitives with zones
  structure/events.dtst         optional: folds and faults (P2-01, "Structural events")
  keyframes/index.json          the timelapse frames
  keyframes/kf-<n>.dtk          one frame: RGBA8 per layer, LZ4
  checkpoints/<i>-<stage>.dtcp  optional stage checkpoints (dev and CLI only; never needed to play)
```

Every file is written as `<name>.tmp`, fsynced, renamed and its directory fsynced. A writer
deletes an existing `manifest.json` before it starts, so an interrupted write leaves a folder with
no manifest, which readers treat as absent. A run (`deeptime run`, or the game through JNI) writes
`preset.json`, `events/`, `sites/`, `bodies/` and `keyframes/index.json` in every folder, empty when
the preset produced nothing; keyframe files are written while the run goes, the rest by `finalizeWrite`, the
manifest last. The synthetic worlds have none of these.

## `manifest.json`

```json
{
  "format_version": "0.2",
  "writers": [{"name": "deeptime-cli", "version": "0.0.1", "format_crate": "0.0.1"}],
  "source": "synth:wrap-test",
  "preset_hash": null,
  "planet": {"radius_m": 48892.96},
  "scale": {"circumference_blocks": 1024, "m_per_block_h": 300.0, "m_per_block_v": 20.0,
            "min_y": -512, "max_y": 639, "sea_level_y": 63},
  "grids": {"L3": {"name": "L3", "w": 512, "h": 256, "r": 2, "wraps_x": true,
                   "origin_block_x": -512, "origin_block_z": -256}, "...": {}},
  "layers": [{"name": "elevation", "path": "rasters/elevation.dtr", "grid": "L3",
              "dtype": "i16", "codec": "lz4_block", "bytes": 12345, "crc32": 305419896}],
  "files": [{"path": "spawn/candidates.json", "bytes": 812, "crc32": 1234}],
  "created_utc": "2026-09-28T12:00:00Z",
  "notes": {"...": "source-specific; the synthetic worlds describe their geometry here"}
}
```

A reader checks `format_version` **before** parsing anything else. `bytes` and `crc32` (CRC-32/ISO-HDLC,
the zlib CRC, as an unsigned JSON number) cover each whole file; readers may verify them lazily.

A run's manifest (P1-01) has `source` `run:<preset name>` (`run:dummy` until P1-03), one writer
`deeptime-core`, `preset_hash` `crc32:<8 hex>` of the canonical preset JSON, `planet.radius_m`,
`grids` and `layers` (empty for a run that bakes nothing; a planet run's bake lists every layer, see
"The bake"), and every other file in `files`:
`preset.json`, `events/index.json` and each segment, `sites/sites.dtsr`, `sites/index.dtsi`,
`keyframes/index.json` and each frame. `notes.run` records per-stage steps and milliseconds, the
final `t_years` and the event, site and keyframe counts. (P0-07's session wrote an ad-hoc manifest
with `writer`, `preset` and `created_unix_ms`; from P1-01 the session writes this one.)

`notes.plates` (runs with the plates stage) holds two series the game never reads, written every
10 Myr and at the end: `margins`, one record per 10 Myr of the continental margins by tectonic
setting (P1-27, `plates::stats::Margins`), and `dynamics`, the plate dynamics record that
`deeptime plate-check` judges a run by (`plates::dynamics`, whose module documentation is the
format): `version`, `t0_myr`, `dt_myr`, `samples`, then `steps` (per step: `t`, `visible`, `land`,
`continental` in L0 samples, `collisions`, `transfers`, and `plates`, one array
`[id, visible, continental, speed_mm_yr, front_km, boundary_km]` per plate alive), `plates` (each
plate's `parent`, `origin` `initial`/`rift`/`subduction_initiation`, `born_myr`, `died_myr`, `fate`
`sutured`/`absorbed`/`consumed`, `into`, `share_at_death`), `collisions` (one record per terrane
transfer: times, plates, `shortening_km`, `peak_uplift_m`, the `lifted10`, `lifted100` and `emerged`
footprints as `{samples, length_km, width_km}`) and `ages` (the sea-floor age histogram every
10 Myr, 41 bins of 10 Myr). Lengths are Earth-normalized km, speeds mm/yr, times Myr relative to
world creation. A 1000 Myr run on the quick 16k planet (500 steps) adds about 0.9 MB to the manifest.

## The bake (a planet run, P1-08)

`Session::finalize_write` bakes every planet preset (`deeptime_core::bake`; plan §5.3, §8.8) after
the amplify stage's L3 packs are listed and before the manifest. `manifest.scale` is the preset's
own scale. The layers (grids from `resolution`: L1 = C / `l1_r` wide, L2 = C / `l2_r`, L3 = C /
`l3_r`; all LZ4 tiles):

| Layer | Grid | dtype | scale, units | nodata | What |
|---|---|---|---|---|---|
| `elevation` | L3 | i16 | 1/16, block | — | terrain y: the amplifier's, mountain character included ("The mountain character"); without the amplify stage the young landscape (L2) or the deep-time surface (L1) upsampled bilinearly |
| `detail/amplitude` | L3 | u8 | 1/16, block | — | the detail term's A (P1-06; absent without the amplify stage) |
| `water_level` | L3 (L3h: L3 averaged 2 × 2 when L3 has more than 40 M cells, 32k full) | i16 | 1/16, block | −32768 | each lake's cells hold its surface, nodata elsewhere (P1-22, "Rivers"): its basin's rim when the lake is full, the level of its water balance, below the rim, when it is not (2026-10-01; a reader does not tell them apart); all nodata when `hydro.enabled` is false |
| `playa` | as `water_level` | u8 | 1, block | — | blocks of salt on a playa's floor, 0 elsewhere ("Playas"); absent when `hydro.enabled` is false |
| `closed_basin` | as `water_level` | u8 | 1, flag | - | 1 where no sea stands on ground below sea level: a closed basin the open sea does not reach, with an 8-block halo ("Rivers", "Water columns" step 3; HYD-100, HYD-102), 0 elsewhere; absent when `hydro.enabled` is false (the sea then stands on all ground below sea level) |
| `floodplain` | L3 (the `elevation` grid) | i16 | 1/16, block | −32768 | what laying the floodplains moved `elevation` by on each plain cell, nodata off the plains ("Rivers", floodplains; HYD-150 to HYD-152; added 2026-10-24); the game reads it as a category, on a plain or not (the levee, HYD-153), and the projection pass takes the nearest cell; absent without `hydro.floodplains` |
| `fan` | L3 (the `elevation` grid) | i16 | 1/16, block | — | what the alluvial fans added to `elevation` on each fan cell, 0 off the fans ("Rivers", fans; HYD-165 to HYD-170; added 2026-10-24); a re-run of the water bake takes it off first; the game does not read it, and the projection pass resamples it as a number (bilinear); absent without `hydro.fans` or with no fan |
| `strata/horizons` | L1 | horizon_stack | 1/16 | — | P1-07's stacks with key spans, smoothed ("The 0.2 bake"), the first top raised ("The P1-08 bake") |
| `strata/lift` | L1 (the grid of `strata/horizons`) | i16 | 1/16, blocks | none | how far every horizon of the cell's stack rose after the history (the bake's whole-stack lift, ledger STR-052; ≥ 0), which the structure file's heights do not include: the deformed column rule adds it ("Structural events", "The lift"; added 2026-10-20). A companion of the stacks: the projection pass takes the nearest cell, as it does theirs |
| `climate/t_mean_c` | L1 | i16 | 0.01, degC | — | mean annual temperature |
| `climate/p_mm` | L1 | u16 | 1, mm/yr | — | annual precipitation |
| `climate/koppen` | L1 | u8 | 1, code index | 255 | `notes.bake.climate.koppen_codes[i]` (31 = sea) |
| `climate/ice_cover` | L2 | u8 | 1, blocks | — | blocks of snow and ice the surface pass lays over the climate's land ice, read by the cover rule ("The ice cover"); absent without the climate stage or its ice fields |
| `climate/koppen_l2` | L2 | u8 | 1, code index | 255 | the Köppen class the biome inputs were placed by (each L2 cell's temperatures lapsed to its own height, the world's ground: the `elevation` layer's mean over the cell, `notes.bake.climate.ground`, BIO-187); absent when the class was decided on L1. Checks of biomes against classes read this, not the L1 `climate/koppen` |
| `climate/wetland` | L2 | u8 | 1, flag | — | 0 not wetland, 1 temperate or warm wetland (vanilla's swamp band E6), 2 cold wetland: boreal peatland ("The wetland flag") |
| `climate/month/t_sl_c_{01..12}` | L1 | i16 | 0.01, degC | — | each month's mean temperature at sea level (the lapse rate taken out: `+ lapse_c_per_km · max(0, h − sea)`), from the climate stage's last snapshot (0.2). On land it includes the cell's plateau warmth (the mass-elevation effect, MOI-085: the lowland's sea-level temperature plus a monthly term times the cell's base height), so a reader that lapses it to a block's height gives every block of the cell that warmth |
| `climate/month/p_share_{01..12}` | L1 | u8 | 1/40, ratio | — | each month's precipitation rate over the twelve's mean (0–6.375; they sum to 12 within the quantum) (0.2) |
| `climate/{temperature,vegetation,continents,erosion,ridges}` | L2 | i16 | 1e-4, vanilla_param | — | the router's five biome inputs, placed against the reference bands and vanilla's land distributions (P1-09, "The climate mapping") on the world's ground (the `elevation` layer's mean over each L2 cell, the height the game's terrain stands on, BIO-187) |
| `surface/regolith_blocks` | L2 | u8 | 1/16, block | — | regolith thickness |
| `surface/class_top` | L2 | u8 | 1, class index | 255 | the rock class at the surface (`strata/classes.json`) |
| `meta/plate_id` | L1 | u16 | 1, plate id | 65535 | the plate |
| `meta/crust_type` | L1 | u8 | 1, crust type code | 255 | the plate model's crust type at the present (L1 field `plates/crust_type`, its nearest sample): **0 oceanic, 1 continental** (the codes of `plates::state`); nodata where the model has nothing. `meta/crust_age_myr` has data exactly where this is 0, `meta/heat_flow_mw_m2` exactly where it is 1. A category: the projection pass takes the nearest cell. The column inspectors read it ("Inspecting a column"; added 2026-10-19) |
| `meta/crust_age_myr` | L1 | u16 | 1, Myr | 65535 | oceanic crust age; nodata on continental crust |
| `meta/heat_flow_mw_m2` | L1 | u16 | 0.1, mW/m² | 65535 | the plate model's surface heat flow at the present, on **continental crust only** (nodata on oceanic crust, whose heat flow is the sea floor's age law: a coastal land column must not read the young ridge crust beside it through the bilinear sample; the plate mesh's heat flow is interpolated over its continental corners only, L1 field `plates/continental_heat_flow_mw_m2`); with the lithosphere's thermal state (`tectonics.thermal_state`, on by default since 2026-10-23; PLT-467, PLT-468) the law is 26 + 3,375 / L for the depth L in km of the sample's thermal lithosphere, 41 over an old craton (L 225 km, where the start's continents begin), 53 at 125 km, 82 at 60 km behind an arc, plus 40 on an arc and 40 on a hotspot, so most of a young planet's continental ground is under 50; with the state off the old proxy, from 55 (old, quiet crust) up to 200: 55 + 35 e^(−t/60) for the time t in Myr since the last orogeny, plus 40 on an arc, 40 on a hotspot, 30 at a divergent boundary (Java: geothermal fields, "Geothermal features (P2-14) and their settings"; added 2026-10-02) |
| `meta/province` | L1 | u16 | 1, province id | — | geologic provinces (P1-10, "The geologic provinces"): contiguous areas of one USGS-style class, each at least the preset's `provinces.min_area_km2` (50,000 km² by default); `notes.bake.meta.provinces` lists them. With TerraBlender installed the game gives every province one biome region |

`notes.bake`: `version` (1), `baked`, per part (`terrain`, `strata`, `climate`, `water`, `surface`, `meta`)
what it did, `millis` per part, `bytes` and `layers` written, and **`detail`** — the game's detail
term: `{"layer": "detail/amplitude", "seed": "<u64, decimal>", "salt": 1729, "octaves": 3,
"wavelength_blocks": 32.0}` (present exactly when `detail/amplitude` is). A preset without the
deep-time surface (plate-only) bakes nothing (`baked: false`).

### The bake's projection pass

The simulation runs on its own Lambert grid (`crates/deeptime-core`; Ben, 2026-10-03: "Bake only").
`Session::finalize_write` bakes the world in Lambert as above, settles the crater lenses and placers,
and then **rewrites the whole folder as the cube world** (`deeptime_core::bake::project_cube`, "The
bake on a cube world" below). Only a synthetic preset that says `scale.projection` `lambert` keeps
its folder as baked, with a Lambert-frame layout block. The rules of this section were written for
the Petroff–Guyou pass the cube's replaced (format 0.5 to 0.6, deleted with cube plan §7.4,
2026-10-24) and hold on the cube except where the next section says otherwise.

- **The world**: the cube's turn ("The bake on a cube world"). The deposit stage forecasts it on its
  present L1 land at a coarser resolution (`notes.deposits.world_frame`, DEP-220) to count the
  essentials' regions in; the projection does not read the forecast, and the anchor's top-up
  (below) adds what the world's own regions lack.
- **Rasters**: every layer is rewritten on the world grid of its own cell size (`L1`, `L2`, `L3`,
  `L3h` keep their names and r; "Grids"). A world cell is the point sample at its block; the block's
  sphere point is found through the projection and the layer is read at the matching Lambert block.
  Continuous layers are averaged over a **footprint isotropic on the sphere**, as wide as the world
  cell's own footprint and as the Lambert rows' spacing there (near the poles a Lambert row spans many
  blocks of planet north–south and its columns a fraction of a block east–west; read at a point, that
  anisotropy draws radial streaks round the poles). Categories (`climate/koppen`,
  `climate/koppen_l2`, `climate/wetland`, `surface/class_top`, `meta/plate_id`, `meta/crust_type`,
  `meta/province`, `delta`, `floodplain`, and any layer whose `units` is `flag`), horizon stacks and any footprint touching no-data take the
  nearest cell (`climate/koppen_l2` only since 2026-10-18: averaged as a number, a class border named
  a class neither side had; `delta` since 2026-10-23, HYD-146; `floodplain` since 2026-10-24, HYD-152). A world cell whose nearest cell is
  on a delta plain (`delta` above 0) also takes that cell's `elevation` and `detail/amplitude`: the
  plain stands at most a block over the sea, and a mean or a bilinear read blended its edge with the
  sea beside it and took it under the sea (HYD-146). The
  `climate/ice_cover` takes the **cover rule** (BIO-159, "The ice cover"): the world cell is covered
  where its nearest Lambert cell is, the cell the categories beside it read (so a covered world cell
  is EF in `climate/koppen_l2`), at the mean depth of the footprint's samples that are covered, each
  the Lambert layer read by the rule at its block (a footprint mean of the blocks covered every world
  cell that one covered sample reached, so the cover's area grew with the footprint, most at the
  poles). The
  climate's band coordinates (`units` `vanilla_param`: `climate/temperature`, `vegetation`,
  `continents`, `erosion`, `ridges`), which the bake sets per Köppen class, are read bilinear at the
  point, as the game read them on the Lambert world, so they stay with the class beside them. The
  `closed_basin` flag (nearest cell) is then drawn again on the world's ground (`hydro::
  closed_basin_on_world`, HYD-105): the Lambert ground's narrow sills that held a pocket of sea floor
  apart from the ocean can be averaged away in the world's, which would leave the pocket a dry hole
  beside the sea. A world cell at or under sea level keeps the flag only where the world's open sea
  does not reach it, the open sea being each 8-connected body (x wraps) of world cells a block or
  more under sea level that holds at least 5,000 km² of cells the nearest Lambert cell left 0; the
  8-block halo is drawn again on the world grid (`notes.bake.projection.closed_basin`:
  `cleared_low_cells`, `flagged_cells_nearest`, `flagged_cells`, `min_open_cells`, `radius_cells`). The
  world's `elevation` then gets its **block-scale relief** (`bake::relief`, drawn in world blocks on
  the world's own grid from the drawn terrain, so it does not depend on the frame the simulation ran
  in): the cone gullies of the standing stratovolcanoes ("Standing volcanoes"), then the
  **steep-ground texture** (D3, ledger AMP-080 to AMP-084): a ridged jag of a few blocks on steep
  hard rock and cliff bands (flat benches, steep risers) on steep lava, the rock read from
  `surface/class_top`. The gullies only lower cells; the texture raises and lowers them. Neither
  takes a cell below one block over sea level nor touches a lake, a river or the sea, so the sea and
  land and the closed basins stay as they are (`notes.bake.projection.relief`, below). A
  Lambert-frame world (no projection pass) gets none. No
  new layer: both write `elevation`, which the game, the far terrain and the column evaluator read
  as before. The five
  band coordinates and the two categories of the L2 biome inputs (`climate/koppen_l2`,
  `climate/wetland`) are held back until the world's `elevation` and `closed_basin` exist and then
  **settled to the world's ground** (BIO-170, BIO-171, BAK-057): at each L2 node's block the world's
  sea rule (`water::sea_covers_height`, the column evaluator's own, HYD-106: the top block under sea
  level − 1 and the node's `closed_basin` not 1) says sea or land; a node whose continentalness lies on
  the other side of the ocean | coast edge (−0.19) takes the other medium's record (the sea's:
  continentalness from the depth, Köppen class 31, no wetland; the land's: continentalness from the
  distance to the sea and the nearest agreeing land node's other fields), and every other node is
  left as the resampling wrote it (`notes.bake.projection.biome_land`: `land_biome_under_sea` and
  `ocean_biome_on_dry_ground`, each `nodes` and `km2`). The ice cover (`climate/ice_cover`) is held with
  them (after the relief has read it as resampled) and settled by its own extent rule, a land cell
  ("The ice cover", step 1), on the world's ground (BIO-177): a node the world's sea covers is set to 0,
  and a node still covered is of the Köppen class EF (BIO-158) whatever the settle gave it
  (`biome_land`: `ice_cover_off_the_sea` and `ice_cover_class_ef`, each `nodes` and `km2`). After the
  settle no wetland stands on or beside the world's playas ("The wetland flag", BIO-178): a node holds a
  playa when a cell of the world's `playa` layer lies in its square, and every node within 20 blocks plus
  two L2 cells plus one `playa` cell of such a node (the settle's own distance in cells) is dried
  (`biome_land.playa_dried`: `playa_nodes`, `reach_blocks`, `nodes`, `km2`). Between the settle and
  that rule no ocean biome is left on a delta plain's land (BIO-185): a node within one L2 cell less one `delta` cell of a
  world cell that is on a plain and dry by the sea rule, in x and in z, is the plain's, and the settle
  runs again with those nodes counted dry (`biome_land.delta_plain`: `plain_nodes`, `to_land`,
  `to_land_km2`). The
  first horizon of every stack cell is then raised to
  the bake's top rule over the world's own terrain (`notes.bake.projection.top_raise`): the stacks
  come from the nearest cell and the terrain from the footprint mean.
- **Vector data**: rivers (vertices through the projection, x continuous along a chain; a chain
  crossing a turning edge is split there, the parts before the edge ending as at the sea, and
  confluences re-pointed; the index rebuilt in the world's frame), structural events (the window's
  centre through the projection, its axes, sizes, a fold's wavelength and a fault's slip and plane
  through the local Jacobian; an intrusion's window the exact image of its ellipse, ledger SCL-236),
  bodies (centre, and the exact image of the horizontal ellipse under the Jacobian, SCL-130 and SCL-236, and
  moved up or down by the change of the drawn terrain height between their old and new column, so a
  crater's lens stays under the drawn floor), sites (centre; box from their bodies; the index in the
  world's frame), `volcanoes/active.json`, `volcanoes/standing.json` and `impacts/craters.json` (`x`, `z`), and
  `spawn/candidates.json` (ranked candidates moved to the nearest world L3 point,
  `surface_y` from the world's elevation, those past the simulation's pole or within 64 blocks of a
  turning edge dropped; unranked ones, dry land nearest (0, 0), picked again on the world's own
  grids by the same rule, `picked_in_the_world` true).
- **Unchanged**: events (latitude and longitude; their L1 masks and paths are in the simulation's
  frame), keyframes (the simulation's Lambert pictures, π : 1), `preset.json`, `strata/classes.json`.
- **The manifest** (the cube's own fields are in the next section): `projection` (the layout
  block, with the border the preset's `scale.edge_border` asks for), and `notes.bake.projection`'s
  `relief` (the block-scale relief:
  `cone_gullies` with `cones`, `not_strato` (standing cones of other kinds, not gullied),
  `gullied`, `cells_cut`, and `active` and `dead`, one group each for the cones active and dead at
  the present, with `cones`, `gullied`, `skipped_sea`, `skipped_low`, `skipped_no_flank`,
  `skipped_other`, `capped`, the medians of the cones' `target_share`, `removed_share`, `gullies`,
  `span_blocks` and `deepest_blocks`, `deepest_blocks_max` and `first`, six cones as [id, gullies,
  target, removed, span blocks, deepest blocks]; `steep_texture` with `cells_moved`, `cells_jagged`,
  `cells_banded`, `cells_raised`, `cells_lowered`, `mean_abs_blocks`, `max_raise_blocks`,
  `max_cut_blocks` and `millis`; and `millis`), `vector` and `millis`.

**The guaranteed deposits under the world's ground** (DEP-221, ORE-280; 2026-10-20). Last, on the
finished folder (`deeptime_core::deposits::anchor::anchor_folder`), the sites whose forming event is a `deeptime:guaranteed_occurrence` (the event
log; no flag in the sites) are read against the world as the game reads it (`ColumnWorld`): each
one's bodies move together by whole blocks (centre, `clip_top_y` and box; the site's centre y and
box follow) so the top of its ore (the highest block of the site's centre column where a zone with
a material applies, not the padded box's top) lies at the **top rock block** over the site's centre (the
`WaterColumn` ground less the 4 blocks of soil on dry land, the bed under water) for an exposed
essential, and no higher than it for the others. In-stratum sites (MVT) stay in their host horizon
while their ore tops out in rock in the centre column; otherwise (the projection's clip under the
drawn ground empties a body whose horizon lies in the soil band) the body is read unclipped, the
ore's top goes down to the highest block of one of the type's host classes at or under the top
rock block within 64 blocks of it (to the block itself when there is none), and its `clip_top_y`
becomes the top rock block. The host run is the highest one as thick as the ore's span in the
centre column, else the thickest within reach (ORE-280). An exposed essential's in-stratum body
stays only when its ore tops out at the top rock block; otherwise (buried deeper, or above it)
its ore's top goes to a host run's top within 8 blocks under the top rock block
(`EXPOSED_BLOCKS`), or to the top rock block when none lies there.
Placers stay where `placer_channels` settled them; natural sites are not touched.

**The top-up in the world's own regions** (DEP-220; 2026-10-21). Then, for each essential of the
preset, the world's land regions (the world's index frame, squares of `region_blocks`; a region is
land when a point of a 64-block lattice in it, x from −C/2 and z from the world's top edge, stands
at or above sea level) are counted again, now with the sources that hold ore in rock in their
centre column as the game reads it (within 8 blocks of the top rock block for an exposed
essential) and whose centre column is dry land (2026-10-21, Ben's rule of 2026-10-16: the ground
at the centre stands at or above sea level, the test that picks the lattice's land, with no water
over its top rock block, the test that picks a copy's point; a source whose centre lies under the
sea counts for no region, since reaching its ore means diving). A land region with fewer than
`min_sites_per_region` gets a copy of a template source: the essential's first guaranteed site
holding ore, else its natural source holding ore with the smallest footprint (never a placer or a
child site), a source whose centre is under the sea skipped while another exists. The copy goes to the region's dry lattice
point nearest the world's spawn (`spawn/candidates.json`, the first candidate) whose lattice
neighbours within 128 blocks are dry too (any dry point when none is), clear of every guaranteed
site's footprint, and is anchored as above (an in-stratum copy at a point whose host lies within
reach, tried at up to 48 points). Its site and bodies take the next free ids, and its forming event
is a new `deeptime:guaranteed_occurrence` (writer `deeptime:geology`, appended as a new segment of
the event log) at the template's forming time, caused by the template's event, with the params of
the stage's guarantee events (`region` the world's region `"col,row"`) plus `copy_of` (the
template site's id). The top-up does nothing on a preset without essentials (the 4k GameTest
planet), and a second run adds nothing.

The event log's new segment and `events/index.json`, then `bodies/bodies.dtb`, `sites/sites.dtsr` and `sites/index.dtsi` are rewritten when any site moved or was added; the caller writes the manifest last. `notes.bake.guaranteed`:
`guaranteed`, `moved`, `moved_up`, `moved_down`, `largest_move_blocks`, `in_stratum_kept`,
`in_stratum_to_host` and `in_stratum_out_of_host` (in-stratum sites moved down into a host class,
or into other rock when none lay within reach), `placers_kept`, `centre_holds_ore` (the sites whose centre column holds ore of theirs at or under
the top rock block), `under_water`, `millis` (the anchor), `topped_up` (copies added),
`top_up` (per essential: `land_regions`, `with_source` (land regions holding enough sources with
ore and a dry centre before the top-up), `counted` (the sources with ore and a dry centre), `sub_sea`
(the sources with ore left out because their centre lies under the sea), `short_by_sea` (land
regions short only for that: they would hold enough with the sea sources counted), `added`,
`added_inland`, `short` (land regions still short: no template or no dry point)), `top_up_millis`.

### The bake on a cube world (format 0.7, cube plan phase 1)

Every planet preset (`scale.projection` `cube`, the default) runs the simulation and the Lambert
bake, then rewrites the folder as a cube world (`deeptime_core::bake::project_cube`; science page
`docs/science/cube-world-bake.md`). It was opt-in beside the Petroff–Guyou pass until cube plan
§7.4 (2026-10-24) made it the only one. The differences from the rules above:

- **The turn** (SCL-120, SCL-121): the cube `conformal-n513` of side s = `face_for(C)` is turned
  freely on the planet so its eight corners lie as far from land as a seeded search finds
  (`bake::cube_turn`: random rotations, the best kept and refined, and the polar turns as a
  candidate; seeded by the preset's seed, so a planet always gets one turn). A pinned
  `scale.lambda_c_deg` gives the polar turn at that longitude instead. The land is the Lambert
  elevation at or above sea level. The manifest's `projection` carries the rotation and the
  corners' `land_distance_deg` ("The cube world's layout").
- **Rasters** (SCL-124): every world cell is a point sample at its centre (`cube_point`). Its
  footprint on the Lambert grid comes from the composed Jacobian (world cell to planet to Lambert
  block): along each of the footprint's two principal axes, where the cell spans more than one
  simulation cell, a Gaussian of the remaining width averages it; else the bilinear read alone.
  There is no polar widening (the Petroff–Guyou pass's (0.52 / cos φ)⁴, SCL-090, retired): the
  cube has no poles of its own. Categories, stacks and cover take the nearest cell as there; the ice cover keeps its
  cover rule.
- **Topology** (SCL-122): the closed-basin redraw, the biome layers' fills and distances, and the
  relief's neighbours run on the cube's own cells across every seam (the open sea is an
  8-connected body over the six faces; distances are great-circle). Nothing is glued.
- **Relief** (SCL-123, SCL-215): the cone gullies and the steep-ground texture are drawn on each
  face, in world blocks, across the seams; a cone's angles are taken on the planet in the vent's
  tangent frame. The noise they read is the cube's surface noise ("The detail layer"), which runs on
  through the turning seams (it replaced SCL-125's blend there).
- **Vector data**: positions through the inverse map, sizes and directions through the analytic
  Jacobian. A river chain that crosses a turning seam is cut there into two chains whose ends
  meet under the seam's fold, the upstream part ending with `Mouth::Seam` (SCL-126; "Rivers");
  widths scale by the local metres per block, depths are kept. Structure windows crossing a
  turning seam are kept (SCL-128). Bodies, sites, `volcanoes/active.json`,
  `volcanoes/standing.json`, `impacts/craters.json` move with their centres. Ranked spawn
  candidates move to the world and are dropped within 256 blocks of a cube corner or 64 of a
  turning edge (SCL-127).
- **The guaranteed deposits**: the deposit stage forecasts the turn on its L1 land
  (`notes.deposits.world_frame` with `projection` `cube` and its own `turn`, SCL-129); the anchor
  and the top-up then run on the cube world's own index frame, with lattice points off the six
  faces left out.
- **The manifest**: `format_version` 0.7, `projection` the cube layout, `scale.circumference_blocks`
  4s, `scale.m_per_block_h` the nominal √(4πR²/6s²); `notes.bake.stats` the land under the cube
  (`corners` with each corner's latitude, longitude and `land_distance_deg`,
  `corner_min_land_distance_deg`, `land_share`, `land_blocks_per_planet_area` quantiles,
  `land_within_5deg_of_corner`, `land_p999_within_2x`, `corners_5deg_from_land`); and
  `notes.bake.projection` with `kind` `cube`, `face_blocks`, `world_blocks`, `map` {name,
  sha256}, `turn` (the search's record), `stats` (as above), `m_per_block_h`,
  `sigma_m_per_block` (quantiles of the local metres per block), `bake_blocks_per_world_block`,
  `layers`, `relief`, `shares` (`land_map`, `land_world_planet` (cells weighted by their planet
  area), `land_true`, and the same for the ice cover), `footprint` (per grid: the cells by the
  footprint's longer stretch in simulation cells, each band's mean samples and its
  checkerboard leak), `seam_steps` (per layer: the 50th, 90th, 99th percentile and largest
  height step between neighbouring cells within a face, near an edge, across direct seams,
  across the slide and across turning seams), `top_raise`, `closed_basin`, `biome_land`,
  `vector` (`rivers` with `seam_cuts`, `seam_partner_mismatch`, `cuts_on_direct_seams`,
  `length_change` and the crossings by seam; `structure_events` with
  `crossing_turning_seams_kept`; `bodies` with `planet_area_ratio_median` mid-face, by a seam
  and by a corner; `sites`; `spawn_candidates`) and `millis`.

### The climate at a block (0.2, P1-25)

`notes.bake.climate.monthly` is `null` for a run without a climate section, else
`{"t_sl": [the 12 layer names], "p_share": [...], "lapse_c_per_km": Γ, "months": "..."}`. Month 01
starts at solar longitude 280° (about 1 January on Earth); the twelve months are equal 30° steps of
solar longitude (a circular orbit), so year fraction f corresponds to λ = 280° + 360°·f and the
northern spring equinox is at f = 80/360. What Deep Time's public climate API
(`com.minecraftindustrialcomplex.deeptime.api.climate`) and Rust's `deeptime_format::ClimateWorld`
answer at block (x, y, z), in this order (both sides bit for bit; `deeptime parity` writes
`parity/climate.json`, 500 blocks, beside `parity/columns.json`):

1. x folded into [−C/2, C/2); every raster sampled at (x, z) in block space (the `RasterRead`
   bilinear and nearest of "Tile pack").
2. `e = max(0, y − sea_level_y) · m_per_block_v` metres; `L = Γ · e / 1000`.
3. for m = 0..11: `T[m] = bilinear(t_sl_c_{m+1}) − L`; `s[m] = bilinear(p_share_{m+1})`; `S = Σ s[m]`.
4. `P = max(0, bilinear(climate/p_mm))`; for m = 0..11: `P[m] = P · (S > 0 ? 12 · s[m] / S : 1) / 12`.
5. the mean `Σ T[m] / 12` (summed m ascending); the warmest and coldest month the first maximum
   and minimum of T.
6. the Köppen class `nearest(climate/koppen)` (NaN → 31), named by `notes.bake.climate.koppen_codes`
   (31 → `sea`).

Precipitation is not lapsed (the simulation's orographic rain is in `p_mm` already). Below sea level
the temperature is the sea-level value. The values are the climate (long-term monthly means), not
weather.

### The climate mapping (P1-09)

The bake (`deeptime_core::bake::biomes`, plan §7.2) writes the five biome inputs in vanilla units
against a **reference**: the band edges of `OverworldBiomeBuilder` (1.21.1) and the land
distribution of vanilla's own erosion and peaks-and-valleys (quantiles at u = i/64, measured from
the `minecraft:overworld` router), the file `crates/deeptime-core/data/climate_mapping/default.json`
(shipped in the jar as `data/deeptime/deeptime/climate_mapping/default.json`). `notes.bake.climate`
records `mapping: "p1-09"`, the preset's `params` (its `biomes` section, defaults filled in),
`reference` (`bands`, and `reference` with `erosion`, `peaks_valleys`, `land_min_continents`), the
mean band coordinates per Köppen class (`koppen_bands`) and the land share of each band
(`land_shares`), and the class placement table the bake used (`placement`, below).

**The preset's `biomes` section** (every key optional; the built-in presets carry them all):
`temperature_shift`, `humidity_shift` (bands, ±2), `beach_blocks` (0–150), `terrain_window_blocks`
(8–1,024), `variant_blocks` (64–16,384), `wetlands` (0–1), and, added with the placement table,
the distances from the sea, in blocks, where the continentalness bands begin —
`inland_mid_blocks` (mid-inland, default 200, 160–800), `inland_far_blocks` (far-inland, 1,000,
900–3,000), `inland_interior_blocks` (the top of the scale, 5,000, 3,500–20,000) — and the
wetland thresholds: `wetland_humidity` (the humidity band coordinate from which land counts as
wet, 2.5, 1–4.5; full weight 1.5 bands above), `wetland_flat_share` (the share of land, flattest
first, swamps may take, 0.35, 0.05–0.9) and `wetland_low_m` (the height above the sea at which
lowland stops being low, 150 m, 20–600). The ranges keep the edges in order (the beach is below
`inland_mid_blocks` below `inland_far_blocks` below `inland_interior_blocks`).

**The class placement table** says where each Köppen class goes on the temperature and humidity
bands. It is a datapack resource, `data/deeptime/deeptime/climate_mapping/placement.json`
(source: `crates/deeptime-core/data/climate_mapping/placement.json`, beside `default.json`; the
jar ships it and the core compiles the same file in as the default). Band coordinates: temperature
0 … 5 over frozen | cold | temperate | warm | hot, humidity 0 … 5 over the five humidity bands.

```text
{
  "schema": 1,
  "_comment": "…",                       optional, ignored
  "glacier_cap": 0.35,                   glaciated land ("Glaciated land", below) is held at or below it
                                         (a class with a "glaciated" block, below, is not)
  "temperature_ramps": {                 named ramps
    "<name>": {
      "base": 2.0, "span": 0.95,         b = base + span · Σ weightᵢ · clamp01((xᵢ − fromᵢ) / (toᵢ − fromᵢ))
      "inputs": [                        base and base + span stay within [0, 5]; the weights add up to 1
        { "of": "t_cold",                t_mean, t_warm (warmest month) or t_cold (coldest month), °C
          "from": 0.0, "to": 18.0,       the temperatures for 0 and 1 (they differ; from may exceed to)
          "weight": 1.0 }                optional, default 1
      ]
    }
  },
  "humidity_rules": {                    named rules; h starts as the aridity index P/PET's band coordinate
    "<name>": {                          (0.03, 0.2, 0.5, 1, 2, 5 → 0 … 5 on a log scale), then, in this order:
      "squeeze": [1.05, 2.55],           band 1 … 4 mapped linearly onto [lo, hi] (0 … 5)
      "min": 2.05, "max": 3.0            raised to min, lowered to max; at least one of the three keys
    }
  },
  "classes": {                           every land class Af … EF (31 of them; not "sea"), no others
    "Af":  { "temperature": "tropical", "humidity": "rainforest" },
    "EF":  { "temperature": "polar_ice", "humidity": "polar_desert",       optional "glaciated": what the class does
             "erosion": { "max": 0.3 },                                    where the glacier flag is set: its own
             "weirdness": { "positive_below_erosion": -0.27 },             "temperature", "humidity" and/or "erosion"
             "glaciated": { "humidity": "ice_cap",                         (at least one; each falls back to the
                            "erosion": { "max": 1.0 } } },                 class's own); optional "erosion": the
                                                                           class's land erosion is clamped into
                                                                           {min, max} (−1 … 1, at least one key);
                                                                           optional "weirdness": "positive" or
                                                                           {"positive_below_erosion": e,
                                                                            "flats": {positive_noise_above, max_magnitude}}
    "BSk": { "temperature": { "when": { "of": "t_cold", "above": 0.0 },   the ramp "then" where the input is above
                              "then": "temperate", "else": "boreal" },     the threshold, "else" elsewhere; "then" and
             "humidity": "steppe" },                                       "else" may themselves be switches (4 deep)
    "BWk": { "temperature": { "when": { "of": "t_mean", "above": 10.0 }, "then": "desert",
                              "else": { "when": { "of": "t_mean", "above": 0.0 },
                                        "then": "cold_desert", "else": "polar_ice" } },
             "humidity": { "when": { "of": "t_mean", "above": 10.0 }, "then": "desert",     the humidity may switch on
                           "else": { "when": { "of": "t_mean", "above": 0.0 },               a temperature the same way
                                     "then": "cold_desert", "else": "polar_desert" } } }
  },
  "ice_cover": {                         optional, every key optional (defaults as written here): "The ice cover"
    "step_m": 160.0, "max_blocks": 12, "snow_blocks": 3, "blue_ice_from_blocks": 9, "blue_ice_blocks": 2,
    "snow": "minecraft:snow_block", "ice": "minecraft:packed_ice", "deep_ice": "minecraft:blue_ice"
  }
}
```

**Glaciated land.** Without a `glaciated` block a class is held at or below `glacier_cap` wherever
the land is glaciated: on L2, where the bake lays an ice cover, the covered cells, which are of class
EF whatever their ground's temperature (BIO-158, "The ice cover"); without a cover, where the glacier
flag (`climate/glacier`) is set. With one, the block
decides instead: its temperature ramp (else the class's own, uncapped), its humidity rule (else
the class's own) and its erosion limits (else the class's own). The default table gives EF one (Ben, 2026-09-29, "Ice only where there's ice"):
EF above its equilibrium line is **ice cap** (humidity `ice_cap`, the fixed band coordinate 0.5,
vanilla's first humidity band: ice spikes, frozen and jagged peaks, snowy slopes and plains, no
forest) and the rest of EF is **barren frozen ground** (`polar_desert`, fixed 1.5: Terralith's gravel
desert and cold shrubland, thin snow in vanilla, which has no dry frozen biome). Both sit at the
temperature ramp `polar_ice` (0.6 … 0.8 by the warmest month), the upper part of the frozen band:
Terralith splits the band at temperature −0.727 (coordinate 0.496) and puts its red, terracotta
snowy badlands below it and its dry cold biomes above. Humidity for EF is fixed because P/PET says
nothing in a freezing climate (PET is 0 below 0 °C, and the stage pins the index at 10). A rule
with `min` = `max` is a fixed coordinate. ET is as it was (`polar`, `tundra`, capped where glaciated).
**Erosion and weirdness (Ben, 2026-09-30, "Fix both").** A class may carry `"erosion": {min, max}`
(vanilla's erosion units, −1 … 1; at least one key): its land's erosion is clamped into it, after the
relief's rank and the wetlands have set it; a `glaciated` block may give its own. And it may carry
`"weirdness"`: `"positive"` (its land takes the positive branch of the variant noise everywhere) or
`{"positive_below_erosion": e, "flats": {"positive_noise_above": n, "max_magnitude": m}}` (the positive
branch where the land's erosion is below `e`, the negative one above, except that the flats, the land
above `e`, are positive where the variant noise (-1 .. 1) is above `n` (default 1: never; 0: the
noise's own half) and their |weirdness| is at most `m` (0.07 .. 1, default none); `flats` is
optional). The default table uses all three for EF:
- barren EF is held below erosion 0.3, which keeps it off Terralith's `glacial_chasm` (packed ice; it
  takes erosion from 0.4, or from 0.565 at any humidity); ice cap (`glaciated`, `max` 1) keeps the
  chasm, the one ice biome for low-relief land;
- its peaks (erosion below −0.27) are on the positive branch, vanilla's frozen peaks, and never on the
  negative branch (jagged peaks in vanilla, where Terralith puts its scarlet mountains): −0.27 is a
  little above vanilla's −0.375 edge because MIC's Tectonic router stretches erosion, and the flats
  (erosion above it) are on the negative branch, where vanilla's and Terralith's cherry groves, which
  are positive-branch variants, do not grow, except where the variant noise is above 0.2 (about a
  fifth of them): those are positive, so that vanilla's ice spikes, a positive-branch variant, come
  back on the ice cap (Ben, 2026-09-30, "Bring ice spikes back"), and their |weirdness| is held to 0.25,
  below the magnitudes the cherry groves need (0.4 and up at mid erosion; MIC's router puts every
  reference value from 0.3 to 0.57 there). All of it is data a pack overrides; the limits are in
  vanilla's units, which the game then maps onto the pack's, so a pack with another router wants
  its own numbers (MIC's tuned ones are the defaults').

**Cold deserts (Ben, 2026-10-02).** Vanilla has no cold desert, and P1-09's BWk was the hot
desert's ramp from a mean of 5 °C up, so a BWk in the tundra's climate was drawn as sand. The
default table splits BWk by its mean annual temperature (a switch on `t_mean`): above 10 °C it is
the hot desert as before (`desert` ramp and rule: sand, badlands on plateaus); from 0 to 10 °C it
is `cold_desert` (temperature 1.05 … 1.95 by the mean, the cold band; humidity at most 0.6, the
driest band: vanilla's plains and meadow, Terralith's steppe, cold shrubland and temperate
highlands, and any biome mod's own cold-desert or cold-steppe biome that its parameter list puts
there); at or below 0 °C it is barren frozen ground like EF's (`polar_ice`, `polar_desert`: snowy
plains in vanilla, gravel desert and cold shrubland in Terralith). A pack with its own biomes
overrides the table where its cold-desert biome sits on other coordinates. BSk is unchanged: it
was already cold (its D ramp below a coldest month of 0 °C, its C ramp above), semi-arid
(humidity 1.05 … 1.95), so vanilla draws plains or snowy plains and Terralith its steppe and
cold shrubland.

The default table reproduces what P1-09 first compiled in, operation for operation for every class
but EF and BWk (a test holds the core's default to the old formulas bit for bit). A ramp is a `base`/`span` pair rather than two
end points so that the coordinates the default writes are exactly the ones P1-09 computed.
The sea's own temperature curve, the aridity → humidity anchors and the depth anchors of the sea
stay in the core.

*How a pack's table reaches the bake.* The bake is Rust and runs before the game (or the mod)
can read a datapack, so the table travels in the **content bundle** (plan §5.4), the JSON the
game passes with the preset: a dedicated server's first-start generation (`FirstStartGenerator`)
reads the table from the server's resource manager (the highest-priority pack's copy; the jar's
own when no pack overrides it) and adds it as the bundle's key `climate_placement`;
`deeptime run --content <file>` takes the same key. `Preset::with_content` parses and checks it
(an unusable table stops the run with the path of the problem, `$.climate_placement.classes.Af.temperature`)
and the bake places the climate by it. Without the key the core's default is used, which is what
the Create World editor's runs and the workbench do (they pass no table, as they pass no
essentials). The table the bake used is recorded whole in the world data: `notes.bake.climate.placement`
is `{"source": "default" | "content bundle", "table": {…}, "patches": [ids], "class_rule_cells": {name: L2 cells}}`
(`patches`: the pack placement tables merged in, below). The game does not read the table
again: the world's biome inputs are already baked with it.

*Pack placement tables* (decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"). A biome mod
whose biomes sit in other parameter slots than vanilla's gets a table of its own:
`data/<namespace>/deeptime/climate_mapping/packs/<id>.json` (the jar ships the core's
`crates/deeptime-core/data/climate_mapping/packs/*.json`; Terralith's is `terralith.json`):
`{"schema": 1, "id": "<id>", "requires_biomes": ["ns:biome", …], "priority": 0, "patch": {…}}` with an
optional `_comment`. `patch` is a JSON merge patch (RFC 7386) on the placement table: objects merge key
by key (a class entry keeps the keys the patch does not name), `null` removes a key, anything else
replaces. The game keeps a pack table when **every** biome in `requires_biomes` is in its biome registry
(one table per `id`; a datapack's file of the same id replaces the jar's) and orders them by `priority`,
then `id`: a dedicated server's first start reads them from its resources, the Create World editor from
Deep Time's jar against the screen's data-pack registries. They travel in the content bundle as the list
`climate_placement_patches` (whole files); `deeptime run --placement-patch <file>` (repeatable) adds to
it. `Preset::with_content` merges them in order over the base table (the bundle's `climate_placement`,
else the default), checks the merged table whole (an error names the packs) and the bake records their
ids. Without a pack table the bake is exactly the default table's. A pack table and a biome rule file
("Biome rules", under the climate interface's game side) are the two halves of one pack's support: the table
moves the climate coordinates a world is baked with, so the pack's own search lands in the right slots
(Terralith's jungles for Am); a rule file chooses at the game's biome query, from Deep Time's own facts, where
no coordinate reaches the right biome or where a biome must be kept to some ground (Expanded Ecosphere's ice cap,
Terralith's rock-named biomes, `deeptime:rock_named`).

*Class rules.* A table may hold `class_rules`: a list of `{"name", "classes": [codes], "when": [{"of",
"above" | "below"}, …], "temperature"?, "humidity"?, "erosion"?, "continentalness"?}` (and `_comment`).
On unglaciated land of one of its classes, the first rule whose conditions all hold (strictly) replaces
what it names of the class's own placement. A condition, like a `when` switch, may read `t_mean`,
`t_warm`, `t_cold` (°C), `aridity` (P/PET), `abs_lat_deg` and `height_m` (the cell's ground above the
sea); an unknown input (the L1 path has no ground height) fails it. A ramp still reads temperatures
only. `continentalness: {min, max}` (vanilla units, −1.2 … 1), for a class or a rule, holds the land
inland of the beach strip (continentalness band 4 and up) in its range; a coast keeps its beach.

Terralith's table (ledger BIO-140 to BIO-146): Am takes humidity 4.05 and up (`monsoon_jungle`), Csa,
Csb and Csc are squeezed into 0.1 … 0.6 (`dry_scrub`: Terralith's shrubland), BSk takes `dry_scrub` and
erosion 0.12 … 0.4, ET takes the positive branch of the variant noise, and the class rule
`cloud_forest` puts Cfb, Cwb, Cfc and Cwc land below 25° of latitude, above 1,000 m and with P/PET above
1 on the cold band (`montane`: 1.2 + 0.75 · unit((t_mean − 6)/12)), humidity 3.05 and up, erosion 0.62
and up and continentalness 0.35 and up (Terralith's cloud forest slot).

At the world's first start the game builds `climate_mapping.json` next to the manifest and reads
it on every later start (so a pack update never moves biomes in unexplored chunks): `schema` 1,
`tables` — per layer `climate/<name>` a monotone table `{"from": [...], "to": [...]}`
(piecewise linear, clamped with slope 1 outside; `odd: true` for `climate/ridges`, applied to |w|
with the sign kept) from the baked value to the pack's — plus `mapping`, `computed_utc`, `why`,
`pack` (`bands` from the pack's copy of the resource, `router`: the pack router's land quantiles,
the points sampled) and `reference` (copied from the notes). Temperature, vegetation and
continentalness map band edge to band edge; erosion maps the reference's land quantiles below E6
onto the pack's below its E6, then E6 edge to edge; weirdness maps the peaks-and-valleys
quantiles above the valley slice on the rising branch (|w| = (PV + 1)/3). A folder without a
P1-09 record (the synthetic worlds, older bakes) gets no file: its values are used as written.

### The geologic provinces and TerraBlender's regions (P1-10)

`meta/province` (L1, u16) is the planet cut into **provinces**: the USGS-style class of every cell
(the `provinces` view layer's rules: shield, platform, orogen, basin, large igneous province,
extended crust, oceanic crust of 0–20, 20–65 and over 65 Myr; `Unclassified` where the plate model
has nothing) is split into connected regions of one class (4-neighbours, x wraps), and regions
smaller than `min_area_km2` are absorbed by the nearest larger one, so provinces are contiguous and
at least that large. Ids count from 0 in the order of each province's first cell (row-major). The
preset's `provinces` section sets `min_area_km2` (default 50,000), `orogen_max_age_myr` (500: a
collision belt stays an orogen this long), `andean_max_age_myr` (50) and `andean_min_elevation_m`
(1,500); all four are recorded in `notes.bake.meta.provinces.params`. The notes also hold
`classes` (the class names), `rule`, `count`, `province` (per id: `class`, `cells`, `km2`) and
`shares`: `classified` and `provinces`, each with `planet` and `continental` (each class's share by
true area as the rules classed the cells, and as the provinces hold them after the small regions
were absorbed), and `absorbed_cells` (how many cells changed class by it). The classified share is
the one to set against Earth's (orogens and arcs about 0.17 of the continents); absorption thins
belts (orogens, the young sea-floor bands) and fattens the coastal extended-crust ring.

The game gives every province one TerraBlender region (plan §7.3) when TerraBlender is installed.
The first start with TerraBlender deals the provinces out to its registered overworld regions in
proportion to their weights and stores the result in `climate_mapping.json` under `regions`:
`{"names": ["minecraft:overworld", "biomesoplenty:overworld_primary", …], "by_province": [i₀, i₁, …]}`
(`by_province[p]` indexes `names`). The deal is stratified, not independent: a pseudo-random rank
per province (a hash of its id and the preset's seed), and the ranks dealt out over the regions by
cumulative weight, so a light region still gets its share of a small planet's provinces. Regions
are stored by name, so installing or removing a biome mod later does not move the provinces that
already have one; a name that is no longer registered falls back to the default overworld region.
A world whose mapping has no `regions` (TerraBlender was absent at its first start) gets them at
the first start that has TerraBlender.

### The ice cover (Ben, 2026-09-30, "Force snow/ice cover")

Terralith has no flat inland ice biome, so an ice cap's flats would read as gravel desert or cold
shrubland whatever biome tag the pack's source picked. Deep Time's own surface pass (`worldgen/IcePass`,
after the pack's surface rules, the strata pass and the water pass) therefore lays snow and ice over
glaciated ground regardless of biome. The contract between the climate and that pass is the layer
`climate/ice_cover` (L2, u8, blocks; 0 = none) and the record `notes.bake.climate.ice_cover`:

- **The layer** (`deeptime_core::bake::ice::downscale`) is the climate's land ice
  (`climate/ice_thickness_m`, L1, m: the ice sheets and glaciers of the deep-time climate, "The
  climate's ice fields") laid on the L2 cells, in three steps (ledger BIO-082, BIO-157):
  1. **The extent**: an L2 land cell (ground at or above the sea) is a candidate when its nearest L1
     cell carries at least 1 m of ice (as for `climate/glacier`), so the candidates are the climate's
     own ice cells and no more (the cube world applies the land test again on its own ground in
     the projection pass, BIO-177). Its thickness `t` is the bilinear blend of the L1 thickness over the
     iced corners only (renormalised over them), so the ice keeps its depth up to its edge instead of
     being blended with the bare cells beside it.
  2. **Its balance**: each candidate has the climate stage's surface mass balance
     (`climate::ice::surface_balance`, the law of `climate/ice_thickness_m`: accumulation
     `min(β_a·Δ, acc_max·P)` above the equilibrium line, ablation `β_m·Δ` below; GLA-069, GLA-070) at
     its own ice surface, `Δ` = ground above the sea + `t` − ELA, with the published equilibrium
     line `climate/ela_m` and the precipitation `climate/precip_mm` blended over their known corners
     (0.5 m/yr where none is), the ground the one the biome inputs were placed on (the world's
     `elevation` averaged onto L2, BIO-187).
  3. **The body closes**: the candidates fall into bodies (4-connected, x wrapping). A body's L2
     balance must not fall below what the climate gave the same ice at the mean height of its L1
     cells' candidates (`min(0, ΣT)`: a body the climate holds in surplus must be fed, one it holds in
     deficit, lagged ice the climate has not melted yet, keeps that deficit and no more); while it does,
     the cell with the most negative balance (ties by index) is given up. So a valley floor far below
     the line that the climate's coarse cell happened to cover loses its ice, and an outlet glacier fed
     by its accumulation area (Franz Josef, below 300 m in rain forest) keeps it.
  Each kept cell is `1 + ⌊t / step_m⌋` blocks, at most `max_blocks`; every other cell is 0. Absent
  when the world has no climate stage or the run has no ice fields (there is no fallback). The pass
  does not read the climate at all: a climate that changes how it carries ice only has to keep this
  layer's meaning. The bake's L2 biome inputs read the same cells: a covered cell is glaciated and of
  class EF (BIO-158), an uncovered one keeps its ground's class.
- **The record** is the placement table's `ice_cover` section, defaults filled in (`step_m`,
  `max_blocks`, `snow_blocks`, `blue_ice_from_blocks`, `blue_ice_blocks`, and the block ids `snow`,
  `ice`, `deep_ice`), plus `source` (the thickness field), `rule`, `cells` (covered L2 cells),
  `climate_cells` (the candidates of step 1), `trimmed_cells` (those step 3 gave up; `cells` +
  `trimmed_cells` = `climate_cells`), `bodies`, `trimmed_bodies` and `thickest_blocks`.
- **The cover rule** (`deeptime_format::raster::cover_of`, `RasterRead::cover`; Java
  `RasterSampler.cover`, `coverOf`; ledger BIO-159): every reader of the layer (the ice pass, the
  far terrain, the census, the bake's projection, the workbench's map) reads it at a block the same
  way: the block is covered where its nearest cell is (as `RasterRead::nearest`, the read the Köppen
  class takes, so every covered block is of the class EF), and its depth is then the weighted mean,
  with the bilinear blend's weights, of the blocks of those of the blend's four cells that are
  covered (the nearest cell's own where none has weight); else 0. So a cover's edge lies half-way
  between a covered cell and a bare one, where the class changes too, and its depth thins over the
  covered cells without spreading past them (a plain bilinear read, rounded, covered every block
  where the blend passed half a block: most of a cell past a 12-block margin).
- **The pass** reads the layer by the cover rule and rounds, giving `n` blocks for the column. The cover **replaces** the top `n` solid blocks of the
  ground, so the terrain's height is unchanged: the top `snow_blocks` (at most `n`) are `snow`, the
  last `blue_ice_blocks` of a cover of at least `blue_ice_from_blocks` are `deep_ice`, the blocks
  between `ice`. A column whose surface is fluid (a lake, the sea) or is the ice of a frozen lake
  (the water pass's `minecraft:ice` top block: the lake's surface, not ground), and air, bedrock and
  fluid blocks, are left alone. Vanilla's `freeze_top_layer` feature then lays its snow layer on the snow block.
  The layer is not part of the column evaluator: Java alone reads it, and Rust's `inspect-column`
  prints it with the other layers.

### The wetland flag (Ben, 2026-10-03, "Taiga instead, peatland if present")

Wet, flat, low, thawed land (BIO-066) is vanilla's swamp (erosion E6) in temperate and warm
climates; in the subarctic classes the placement table marks `cold_wetland` (Dfc, Dfd, Dsc, Dsd, Dwc,
Dwd; the table's `classes.<code>.cold_wetland`, default false) it is boreal peatland, which vanilla does
not have. `climate/wetland` (L2, u8) says which: 0 none, 1 the swamp
band, 2 cold wetland. A cold wetland cell keeps its class's temperature and humidity and its erosion is
held under E5, so vanilla draws the class's ordinary biome there (taiga); the bake's note records
`cold_wetland_cells` and `cold_wetland_cells_by_class` beside `wetland_cells`. Java alone reads the layer: where the pack's own biome source answers
with a peatland biome (tagged `c:is_swamp` and `c:is_cold/overworld` or `c:is_snowy`: Biomes O' Plenty's
bog and muskeg) at the swamp row of the cold band, `WorldDataTerrain` returns an erosion inside E6 for
flagged cells of that region, and the pack draws its biome there. The verdict
(`Peatland`, `PeatlandProbe`) is decided once per world, per TerraBlender region of its province table
(`""` when it has none), at the first biome query of a flagged cell, and stored in
`climate_mapping.json` as `peatland: {erosion, regions, biomes}` so a pack update does not move
explored biomes; `regions: []` means the pack has none and cold wetland is taiga everywhere.

No wetland stands on or beside a playa (BIO-178). A cell within the reach of a playa whose flag is not 0,
or whose `climate/erosion` is in vanilla's swamp band E6 with the flag 0, gets the flag 0 and its erosion
held under E5 as a cold wetland's is (a lower one is kept); any other cell is left alone. The water bake
does it after it writes `playa`, on every L2 cell within `PLAYA_WETLAND_MARGIN_BLOCKS` (20) blocks of
ground of a playa cell's block, plus two L2 cells and one water cell (the grids' reach); on the Lambert
grid the box is 1 / cos φ times as wide in x (a block of ground spans that many grid blocks east-west)
and wraps across the seam. The projection pass does it again on the world's own L2 cells after its land
call ("The bake's projection pass"), since the resampling and the land call can carry a swamp's record onto
the world's playa. A playa is a basin whose water dries off its floor, so a swamp there came from the
climate's wetness or a neighbour's record, and its trees (a mangrove's roots reach 9 blocks) stood over the
salt. `notes.bake.water.playa.wetland_cleared` is `{playa_cells, reach_blocks, lambert, nodes}` (`nodes`:
the cells it changed).

## Grids

Rasters in block space on the **Lambert cylindrical equal-area projection** (plan §5.2 as amended
by `docs/design/equal-area-projection.md`). The projection's formulas (block ↔ latitude and
longitude, the grids' extent and neighbour rule, the local metric and cell areas, the frame of the
spatial indexes below, map pictures) are in one place per side: Rust `deeptime_format::projection`,
Java `worlddata/Projection` (and the pages' `projection.js`). A planet of circumference C has radius
R = C / 2π blocks; longitude λ = 360°·(x + C/2)/C (the seam x = −C/2 is 0°), latitude
φ = asin(−z / R) (north is −z; clamped to ±90° past the poles). Every block covers the same area
of the planet; east–west a block is cos φ blocks of planet, north–south 1 / cos φ.

A grid has `w` × `h` cells with `w = C / r`, `h = 2·Zo / r`, `r` blocks per cell, origin
`(−C/2, −Zo)`. Zo is the smallest multiple of the planet's quantum q at or past the pole R, q the
largest power of two ≤ 64 dividing C/2 and ≤ R/2 (64 on every real planet), raised to `r` when `r`
is coarser: Zo = 2,624 at C = 16,384 and 5,248 at C = 32,768, the same for every grid of a planet,
so the rows of L1, L2 and L3 nest. Rows are uniform in z, hence in sin φ: every cell is equal in
area, (r·m)² for m metres per block, less only on the rows that reach past a pole.

- Cell `(i, j)` is the **point sample** at block `x = origin_block_x + i·r`,
  `z = origin_block_z + j·r` (not the centre of an r × r square). Column 0 lies on the seam.
- Row-major: row `j` (z) outer, column `i` (x) inner. Row 0 is at z = −Zo, at or just past the
  north pole; rows past a pole hold no planet (they sample the pole).
- Indices wrap in x: `i mod w` (Euclidean). They clamp in z: `clamp(j, 0, h − 1)`.
- Bilinear sampling at block `(bx, bz)`:
  `u = (bx − ox) / r`, `v = (bz − oz) / r`, `i0 = floor(u)`, `j0 = floor(v)`, `fu = u − i0`,
  `fv = v − j0`, then `a = v00 + (v10 − v00)·fu`, `b = v01 + (v11 − v01)·fu`,
  result `a + (b − a)·fv` on physical values, in exactly this order (both sides agree bit for bit).
  Any nodata corner makes the result NaN.
- Nearest sampling: the cell `(floor(u + 0.5), floor(v + 0.5))`.
- **`z_edges`**: `"clamp"` (the simulation's grids and Lambert-frame worlds; writers always write
  the key, since the checkpoints encode grids with postcard, which cannot leave a field out) or
  `"cube"` ("Cube grids"). The Petroff–Guyou worlds' `"half_turn"` (rows folding through the north
  and south edges' half-turns) went with them (cube plan §7.4, 2026-10-24).

## The world's layout (`manifest.json` `projection`, format 0.5)

The world's projection and how its edges are glued. **This block is the contract the wrap engine
reads to install its topology** (`docs/design/wrap-engine.md` §9, Phase 2); it is stable from 0.5,
and a change to it bumps `layout_version`. Rust: `deeptime_format::projection::ProjectionSpec`
(`lambert_spec`, `WorldProjection::spec`); Java: `worlddata/WorldLayout`. Layout version 1 was the
Petroff–Guyou world's (0.5 to 0.6, gone with cube plan §7.4, 2026-10-24: two hemisphere squares of
side b, glued north and south by half-turns, with four cone points of 180°) and is now only the
Lambert frame's; the cube's is layout version 2 (next section).

- **`kind`**: `cube` or `lambert`. **`layout_version`**: 2 or 1.
- **`domain`**: the fundamental domain, x in [x_min, x_max), z in [z_min, z_max]. Every stored
  block lies in it; what lies beyond an edge is a copy of something inside, by that edge's motion.
- **`edges`**: one entry per edge of the domain, in the wrap engine's terms (`WrapEdge`):
  - `axis` `x` (an east or west edge, the line x = `coord`) or `z` (north or south, z = `coord`);
    `side` `min` (beyond is v < coord) or `max` (beyond is v ≥ coord); `coord` is a block boundary;
    `span` [first, one past the last] block along the edge;
  - `iso` = `[k, tx, tz]` maps a block beyond the edge to the block inside that it is, as the wrap
    engine's `CellIsometry(quarterTurns = k, tx, tz)`: k clockwise quarter-turns seen from above,
    then the translation. On block **cells** (the block at integer (x, z) spans [x, x+1) × [z, z+1)):
    k = 0: (x + tx, z + tz); k = 2: (−x − 1 + tx, −z − 1 + tz). On **points** (entities) k = 2 is
    (−x + tx, −z + tz);
  - `kind` `slide` (k = 0), `half_turn` (k = 2, about `pivot`, a block corner) or `quarter_turn`
    (the cube's, k = 1 or 3);
  - `owner`: who joins it in the game, `toroidal_world` (the east–west slide, as before; Ben,
    2026-10-03: "TW slides, engine turns") or `wrap_engine`.
- **`border`**: what the game does at edges nothing joins yet ("The cube world's layout").

A **Lambert** world's block is `{"kind": "lambert", "layout_version": 1, "domain": {x ±C/2, z ±R},
"edges": [west, east slides with iso [0, ±C, 0], span [−Zo, Zo]], "cone_points": [], "poles":
[{"name": "north", "images": []}, …]}` and no border: a synthetic test world, which ends at its
first and last rows (the pole cap that stopped players there went with the playable Lambert world).
A manifest without `projection` is read as Lambert.

## The cube world's layout (format 0.7, `layout_version` 2)

Written by the Rust side since cube plan phase 1 (`deeptime synth --kind cube`, the bake's cube
option; docs/plans/cube-world-plan.md B1 to B10) and read by its tools; the game reads it from
phase 2 ("Lambert-frame and cube worlds side by side"). It follows the plan's question Q1, answer (a)
(Ben, 2026-10-22, decisions log): rasters face by face with no void cells, the map table shipped in
the code, and the manifest naming the table and its sha256. The map is the conformal cube turned
freely (Q4, Ben, 2026-10-22). The geometry is coded and tested in all three languages: Rust `deeptime_format::cube` (the net) and `deeptime_format::cube_map` (the
map, the turn, `CubeWorld`), Java `worlddata/CubeLayout` and `worlddata/CubeMap`, JS `DTCube`,
`DTCubeMap` and `DTCubeWorld` in `projection.js`, all checked against the parity fixture's `cube`
section (`tests/data/projection_parity.json`).

A cube world is six square faces of side s blocks laid out as a cross: four in a band (F3, F0, F1,
F2 from west to east, x wrapping with period 4s through Toroidal World), N above F0 and S below it.
Each face is drawn on the planet through the **map** (a table, "The cube map table") and the whole
cube is placed on the planet by a free **rotation**, so the poles lie wherever the rotation puts
them (usually inside N and S, not at their centres): latitude does not run along the net and −z is
not north. Here the 16k world (s = 3,840) with a turn taken from the parity fixture's free-turn
world:

```json
"projection": {
  "kind": "cube",
  "layout_version": 2,
  "face_blocks": 3840,
  "map": {"name": "conformal-n513",
          "sha256": "c9dd9fd493ac905237678150c332fd580ed05bf0967af9519a85b889e279b9e1"},
  "rotation": [[ 0.465740957, -0.875614377, 0.128003218],
               [ 0.664987828,  0.441743093, 0.602207795],
               [-0.583846341, -0.195352253, 0.788010754]],
  "domain": {"x_min": -5760.0, "x_max": 9600.0, "z_min": -5760.0, "z_max": 5760.0, "period_x": 15360},
  "faces": [
    {"id": "F0", "index": 0, "origin": [-1920, -1920], "frame": ["+X", "+Y", "-Z"], "centre_lat_deg": 7.3542,   "centre_lon_deg": 298.0086},
    {"id": "F1", "index": 1, "origin": [1920, -1920],  "frame": ["+Y", "-X", "-Z"], "centre_lat_deg": 37.0282,  "centre_lon_deg": 33.5956},
    {"id": "F2", "index": 2, "origin": [5760, -1920],  "frame": ["-X", "-Y", "-Z"], "centre_lat_deg": -7.3542,  "centre_lon_deg": 118.0086},
    {"id": "F3", "index": 3, "origin": [-5760, -1920], "frame": ["-Y", "+X", "-Z"], "centre_lat_deg": -37.0282, "centre_lon_deg": 213.5956},
    {"id": "N",  "index": 4, "origin": [-1920, -5760], "frame": ["+Z", "+Y", "+X"], "centre_lat_deg": 52.0,     "centre_lon_deg": 198.5},
    {"id": "S",  "index": 5, "origin": [-1920, 1920],  "frame": ["-Z", "+Y", "-X"], "centre_lat_deg": -52.0,    "centre_lon_deg": 18.5}
  ],
  "edges": [
    {"name": "F3-west",   "seam": "east_west", "kind": "slide",        "axis": "x", "side": "min", "coord": -5760, "span": [-1920, 1920], "iso": [0, 15360, 0],      "partner": "F2-east",   "owner": "toroidal_world"},
    {"name": "F2-east",   "seam": "east_west", "kind": "slide",        "axis": "x", "side": "max", "coord": 9600,  "span": [-1920, 1920], "iso": [0, -15360, 0],     "partner": "F3-west",   "owner": "toroidal_world"},
    {"name": "N-east",    "seam": "ne", "kind": "quarter_turn", "axis": "x", "side": "max", "coord": 1920,  "span": [-5760, -1920], "iso": [1, 0, -3840],    "pivot": [1920, -1920],  "partner": "F1-top",    "owner": "wrap_engine"},
    {"name": "F1-top",    "seam": "ne", "kind": "quarter_turn", "axis": "z", "side": "min", "coord": -1920, "span": [1920, 5760],   "iso": [3, 3840, 0],     "pivot": [1920, -1920],  "partner": "N-east",    "owner": "wrap_engine"},
    {"name": "N-west",    "seam": "nw", "kind": "quarter_turn", "axis": "x", "side": "min", "coord": -1920, "span": [-5760, -1920], "iso": [3, 0, -3840],    "pivot": [-1920, -1920], "partner": "F3-top",    "owner": "wrap_engine"},
    {"name": "F3-top",    "seam": "nw", "kind": "quarter_turn", "axis": "z", "side": "min", "coord": -1920, "span": [-5760, -1920], "iso": [1, -3840, 0],    "pivot": [-1920, -1920], "partner": "N-west",    "owner": "wrap_engine"},
    {"name": "N-top",     "seam": "nn", "kind": "half_turn",    "axis": "z", "side": "min", "coord": -5760, "span": [-1920, 1920],  "iso": [2, 7680, -7680], "pivot": [3840, -3840],  "partner": "F2-top",    "owner": "wrap_engine"},
    {"name": "F2-top",    "seam": "nn", "kind": "half_turn",    "axis": "z", "side": "min", "coord": -1920, "span": [5760, 9600],   "iso": [2, 7680, -7680], "pivot": [3840, -3840],  "partner": "N-top",     "owner": "wrap_engine"},
    {"name": "S-east",    "seam": "se", "kind": "quarter_turn", "axis": "x", "side": "max", "coord": 1920,  "span": [1920, 5760],   "iso": [3, 0, 3840],     "pivot": [1920, 1920],   "partner": "F1-bottom", "owner": "wrap_engine"},
    {"name": "F1-bottom", "seam": "se", "kind": "quarter_turn", "axis": "z", "side": "max", "coord": 1920,  "span": [1920, 5760],   "iso": [1, 3840, 0],     "pivot": [1920, 1920],   "partner": "S-east",    "owner": "wrap_engine"},
    {"name": "S-west",    "seam": "sw", "kind": "quarter_turn", "axis": "x", "side": "min", "coord": -1920, "span": [1920, 5760],   "iso": [1, 0, 3840],     "pivot": [-1920, 1920],  "partner": "F3-bottom", "owner": "wrap_engine"},
    {"name": "F3-bottom", "seam": "sw", "kind": "quarter_turn", "axis": "z", "side": "max", "coord": 1920,  "span": [-5760, -1920], "iso": [3, -3840, 0],    "pivot": [-1920, 1920],  "partner": "S-west",    "owner": "wrap_engine"},
    {"name": "S-bottom",  "seam": "ss", "kind": "half_turn",    "axis": "z", "side": "max", "coord": 5760,  "span": [-1920, 1920],  "iso": [2, 7680, 7680],  "pivot": [3840, 3840],   "partner": "F2-bottom", "owner": "wrap_engine"},
    {"name": "F2-bottom", "seam": "ss", "kind": "half_turn",    "axis": "z", "side": "max", "coord": 1920,  "span": [5760, 9600],   "iso": [2, 7680, 7680],  "pivot": [3840, 3840],   "partner": "S-bottom",  "owner": "wrap_engine"}
  ],
  "cone_points": [
    {"name": "n0", "cone_deg": 270.0, "faces": ["F0", "F1", "N"], "cube": [1, 1, 1],    "lat_deg": 61.2284,  "lon_deg": 310.9952, "land_distance_deg": 7.1,  "reachable": true,  "images": [[1920, -1920]]},
    {"name": "n1", "cone_deg": 270.0, "faces": ["F1", "F2", "N"], "cube": [-1, 1, 1],   "lat_deg": 46.7809,  "lon_deg": 108.9206, "land_distance_deg": 9.4,  "reachable": false, "images": [[1920, -5760], [5760, -1920]]},
    {"name": "n2", "cone_deg": 270.0, "faces": ["F2", "F3", "N"], "cube": [-1, -1, 1],  "lat_deg": 1.9124,   "lon_deg": 172.0803, "land_distance_deg": 6.4,  "reachable": false, "images": [[-1920, -5760], [-5760, -1920], [9600, -1920]]},
    {"name": "n3", "cone_deg": 270.0, "faces": ["F3", "F0", "N"], "cube": [1, -1, 1],   "lat_deg": 10.4383,  "lon_deg": 242.6305, "land_distance_deg": 12.0, "reachable": true,  "images": [[-1920, -1920]]},
    {"name": "s0", "cone_deg": 270.0, "faces": ["F0", "F1", "S"], "cube": [1, 1, -1],   "lat_deg": -1.9124,  "lon_deg": 352.0803, "land_distance_deg": 8.2,  "reachable": true,  "images": [[1920, 1920]]},
    {"name": "s1", "cone_deg": 270.0, "faces": ["F1", "F2", "S"], "cube": [-1, 1, -1],  "lat_deg": -10.4383, "lon_deg": 62.6305,  "land_distance_deg": 6.9,  "reachable": false, "images": [[1920, 5760], [5760, 1920]]},
    {"name": "s2", "cone_deg": 270.0, "faces": ["F2", "F3", "S"], "cube": [-1, -1, -1], "lat_deg": -61.2284, "lon_deg": 130.9952, "land_distance_deg": 15.3, "reachable": false, "images": [[-1920, 5760], [-5760, 1920], [9600, 1920]]},
    {"name": "s3", "cone_deg": 270.0, "faces": ["F3", "F0", "S"], "cube": [1, -1, -1],  "lat_deg": -46.7809, "lon_deg": 288.9206, "land_distance_deg": 7.7,  "reachable": true,  "images": [[-1920, 1920]]}
  ],
  "poles": [
    {"name": "north", "face": "N", "u": 0.823877, "v": 0.169330,  "images": [[1581.8, -3514.9]]},
    {"name": "south", "face": "S", "u": -0.823877, "v": 0.169330, "images": [[-1581.8, 4165.1]]}
  ],
  "border": {"kind": "edge_stop", "inset_blocks": 1.0, "temporary": true,
             "segments": ["N-east", "F1-top", "N-west", "F3-top", "N-top", "F2-top",
                          "S-east", "F1-bottom", "S-west", "F3-bottom", "S-bottom", "F2-bottom"]}
}
```

The rotation, the corners' and the face centres' latitudes and longitudes and the poles are the
fixture's turn (Rz(−71.5°)·Rx(38°)·Rz(12°)) computed by the code; the corners' `land_distance_deg`
in the example are made up, since no planet stands behind it. A real manifest writes every number at
full f64 precision.

- **`kind`** `cube`. **`layout_version`** 2 (the 0.5 layouts are version 1).
- **`face_blocks`** s, the face side in blocks: the planet's equal-area face C/√(6π) for the
  simulation's circumference C, rounded to the nearest multiple of 256 (`face_for`, ledger SCL-078
  and SCL-100): **1,024 at 4k, 3,840 at 16k, 7,424 at 32k**. Every grid's r divides s, every pivot
  is a chunk corner, and the period 4s is whole region files.
- **`map`**: the table that draws a face on the sphere, by `name` and the `sha256` of its file
  ("The cube map table"). A reader that does not ship a table of that name with that hash refuses
  the world (`world data needs cube map <name> (sha256 <hash>); this Deep Time has <names>`); it
  never falls back to another table. The world's map is `conformal-n513` (the conformal cube, τ = 0,
  ledger SCL-101). The simulation runs on its own cube (`moser-t1-n513`, Moser's equal-area cube,
  τ = 1, ledger SCL-102, on the identity turn), which the layout does not name: the simulation's
  files carry latitude and longitude, never cube coordinates.
- **`rotation`**: the turn that places the cube on the planet, a 3 × 3 rotation written as its
  three rows (row-major), **planet frame to cube frame**: a planet unit vector p (the format's frame,
  +x the point (0°, 0°), +y (0°, 90° E), +z the north pole) is the cube-frame vector Q·p, and a cube
  vector c is the planet's Qᵀ·c. It replaces the polar layout's `lambda0_deg` (a turn about the axis
  only; `CubeTurn::polar(λ0)` is the special case). The bake chooses it per planet by the corner
  search (docs/research/notes/mountains-and-the-projection.md §5.4: the corners as far out at sea as
  possible, then the cube's 24 symmetries fix it with the north pole in N and the most land on F0).
  A reader checks it is a rotation (rows orthonormal to 10⁻¹², determinant positive;
  `CubeTurn::from_rows`) and refuses the world otherwise. In the three languages the products are
  summed in one order, `(m0·p0 + m1·p1) + m2·p2`.
- **`domain`**: the net window, x in [x_min, x_max) = [−3s/2, 5s/2), z in [z_min, z_max] =
  [−3s/2, 3s/2], and **`period_x`** 4s, Toroidal World's loop. Half the window is void (the six
  s × s squares above and below F1, F2 and F3); no block of the void is stored.
- **`faces`**: the six faces in the format's order (F0, F1, F2, F3, N, S; `index` is the order
  rasters store them in). `origin` is the net block of the face's local cell (0, 0), its north-west
  corner; the face covers [origin, origin + s) in x and z. Every face is drawn in the net without a
  turn, so its local axes are the net's: u east (+x), v south (+z). `frame` gives the cube-frame
  directions of the face's centre, of local +u and of local +v (each frame has e_u × e_v = −centre,
  so the turns between faces are rotations). `centre_lat_deg`, `centre_lon_deg`: the planet point at
  the face's centre through the rotation (longitude in [0°, 360°)), for tools and pages.
- **`edges`**: the 14 net edges that face the void or the window's ends, in the order of
  `Cube::edges` and the wrap engine's `WrapLayouts.cubeCross`. Each has the fields of "The world's
  layout" (`axis`, `side`, `coord`, `span`, `iso`, `pivot`, `owner`), and adds `seam`
  (`east_west`, `ne`, `nw`, `se`, `sw`, `nn`, `ss`) and `partner`, the other net edge of the same
  cube edge. `kind` is `slide` (k = 0), `quarter_turn` (k = 1 or 3, about `pivot`, one of F0's
  corners) or `half_turn` (k = 2, about `pivot`, the centre of the void square above or below F1).
  The `iso` on
  cells: k = 1 is (−z − 1 + tx, x + tz), k = 3 is (z + tx, −x − 1 + tz), k = 2 as before; on points
  (entities) k = 1 is (−z + tx, x + tz) and k = 3 (z + tx, −x + tz). So N-east's `[1, 0, −s]`
  takes the point (s/2 + d, z) beyond N's east edge to (−z, s/2 + d − s) on F1, a heading turned a
  quarter clockwise. A block beyond two bands at once (the void squares by F0's corners) takes the
  nearer edge's motion, ties to the earlier edge in the list, so each such square splits on its
  diagonal through F0's corner (plan §1.4); a block in no band, or more than s out, is past the net's
  corners and the game draws deep ocean there (`CubeFold::Ocean`).
- **`cone_points`**: the cube's 8 corners, where three faces meet and the world covers 270° round
  the point (`cone_deg`). `n0` to `n3` are face N's corners counter-clockwise from cube (+1, +1, +1),
  `s0` to `s3` face S's; `faces` the three faces, `cube` the corner's cube-frame direction
  (±1, ±1, ±1). `lat_deg`, `lon_deg`: the planet point there through the rotation.
  **`land_distance_deg`**: the signed great-circle distance from the corner to the nearest land of
  the final keyframe (positive at sea, minus the distance to the sea on land, clamped to ±20°, the
  corner search's score; study §5.4 items 2 and 7). `reachable`: whether a player can stand on it
  (F0's four corners, inside the world, each with one image) or not (the other four lie on bordered
  edges, with two images, and a third at x = 5s/2 where the window's closure repeats F3's west
  corner). `images`: every block corner of the window's closure that is the corner.
- **`poles`**: where the rotation puts the two poles: the `face` and its local (u, v) ∈ [−1, 1]²
  (`CubeWorld::north_pole`, the map's inverse of the planet vector), and `images`, the net points
  (f64, not block corners any more) that are the pole: one inside a face, two when the pole lies on a
  net edge. On the identity turn (the simulation's cube) they are the centres of N and S.
- **`border`**: what the game does at the edges nothing joins yet, on **segments**: `edge_stop`
  (the preset's `scale.edge_border` = `stop`, the default) names the 12 turning net edges (all but
  the slide), and players, the vehicles they ride and Sable ships are stopped `inset_blocks`
  (SCL-088) inside each edge line. It is `temporary`: it goes when the wrap engine joins the edges.
  Absent (`edge_border` = `none`): no border; the terrain past an edge is the folded copy ("Cube
  grids"), but nothing joins the edge. A position beyond two segments (inside a void square by F0's corner) is pushed
  back to the nearest point of the planet, not segment by segment (plan W3). There is no pole cap.

**Scale.** `scale.circumference_blocks` is the loop period 4s (15,360 at 16k), as Toroidal World's
loop reads it. `scale.m_per_block_h` is the nominal size, √(4πR²/6s²) for the planet's radius R in
metres (`planet.radius_m`): 294.82 m at 16k. The true size varies with the map: every
metres-to-blocks conversion uses the local scale σ (planet metres per block) from the map's
Jacobian (`CubeWorld::jacobian`, which also gives the 2 × 2 matrix and the local angle).

### Cube grids

A grid of cell size r is stored face by face: six planes of n × n cells, n = s / r, in the faces'
order, each in its own net orientation (cell (i, j) is the r × r square of net blocks from
`origin` + (i·r, j·r), row j outer and column i inner), so a reader never turns a cell. There are no
void cells. Rust `GridDesc::for_cube`, `face_cell`, `cube_spot` and the cube branches of
`RasterRead`.

- **Descriptor.** `{"name": "L3", "kind": "cube", "n": 1920, "r": 2}`, in the manifest's `grids`
  and in every tile pack's LayerDesc. A reader takes w = n and h = 6n (the six planes stacked F0 to
  S down the rows), no x wrap, and origin F0's north-west corner (−s/2, −s/2), s = n·r. Postcard
  copies (checkpoints) keep the polar grids' fields with `z_edges` `cube`.
- **Tiles.** `tile` is the largest divisor of n that divides 256, so no tile holds two faces: 256 at
  4k (n 512, 256), 64 for L1 at 4k (n 64); 128, 64 and 16 at 16k (n 1,920, 960, 240).
- **Where a value sits.** Cell (i, j)'s value is the **centre** of its square, the net point
  `origin + ((i + ½)·r, (j + ½)·r)` (not a point sample on a block corner as on the polar grids):
  the faces tile the planet with no shared samples, and every cell has the same footing on all six
  faces. The bake and the synthetic world sample the planet there.
- **A reader's block.** Every reader takes block (bx, bz) as on the polar grids, and on a cube grid
  reads at the block's centre, the net point (bx + ½, bz + ½). That point is first folded onto the
  planet through the net edges (`Cube::fold_point`: the slide, then the nearest band's motion), which
  gives its face and its local point; past the net's corners there is no planet and a read is NaN
  (cover 0). Its fractional cell is `a = (x − ox)/r − ½`, `b = (z − oz)/r − ½`, (x, z) the folded
  point and (ox, oz) its face's origin.
- **Nearest:** the face's cell `(floor(a + ½), floor(b + ½))`. **Bilinear:** `i0 = floor(a)`,
  `j0 = floor(b)`, the four cells (i0, j0), (i0 + 1, j0), (i0, j0 + 1), (i0 + 1, j0 + 1) of that face
  with the polar grids' arithmetic and order; a cell off the face is read through the adjacency
  below. The **column spline** (`strata/lift`, the horizon tops) and the **cover** rule read their
  4 × 4 and 2 × 2 cells the same way.
- **Off the face.** A face's cell (i, j) with one index out of [0, n) (up to n out) is the
  neighbour's cell across that side, with the quarter turns between the two frames
  (`cube::ADJACENCY`, `cube::cell_on`). Beyond two sides at once (the corner squares, where three
  faces meet at a 270° corner) it is the neighbour across the side it is further out of, ties going
  east or west, with the other index clamped onto the face: the corner square repeats the strip of
  that neighbour along its edge (the diagonal ghost of the FV3 rule). More than a face out clamps to
  a face out. A stacked index (i, j) reads face ⌊j / n⌋ (clamped to the six) at (i, j mod n) by the
  same rule (`GridDesc::cell`); a reader that steps across a face's edge steps in face coordinates,
  never down the stacked rows. So bilinear reads are continuous across every seam.

The spatial indexes (sites, rivers, structure buckets, the site index) of a cube world use the
window as their frame: origin (−3s/2, −3s/2), span (4s, 3s) (`WorldProjection::index_frame`). The
bake's notes (`notes.bake.stats`: the land's largest and 99.9th-percentile blocks per planet area
and the land share within 5° of a corner, study §5.4 item 7) are "The bake on a cube world".

## The cube map table (`crates/deeptime-format/data/cube-map-<name>.bin`)

The table a cube world's `projection.map` names, shipped in the code, never in a world: Rust
embeds it (`CubeMap::builtin`, `CubeMap::builtin_bytes`), the jar carries it as the resource
`deeptime/cube-map/cube-map-<name>.bin` (Java `CubeMap.builtin`), and each viewer bundle carries the
world's table as `cube-map-<name>.bin` beside `projection.js` (`DTCubeMap.load` on its bytes). Each
file has `<file>.sha256` beside it (sha256sum format); a reader checks the hash before use.
`tools/cube-map/cube_map.py` writes them (plan M1); nobody has to regenerate one bit for bit, since
the hash pins the world.

| name | τ | m | q | patch K, F | flow steps | bytes | sha256 |
|---|---|---|---|---|---|---|---|
| `conformal-n513` (the world's, SCL-101) | 0 | 256 | 128 | 4, 16 | 0 | 701,168 | `c9dd9fd493ac905237678150c332fd580ed05bf0967af9519a85b889e279b9e1` |
| `moser-t1-n513` (the simulation's, SCL-102) | 1 | 256 | 128 | 0, 0 | 400 | 664,720 | `000e6532b89d84de287fbf9694fa080a3845261483e195b254d3b5a42a34f9d9` |

What it holds is one eighth of a face, the triangle 0 ≤ v ≤ u ≤ 1 of the face's local coordinates
(u, v); the cube's symmetry gives the rest by sign flips and a swap. Each forward node stores the
planet point in **gnomonic coordinates of the face frame**, (a, b) = (y/x, z/x) with x along the
face's centre and y, z along e_u and e_v, so a node is the point (1, a, b)/√(1 + a² + b²) of the
face frame. Nodes on the edge u = 1 hold a = 1 exactly, on the axis b = 0, on the diagonal a = b,
and the corner (1, 1).

```text
offset  size  field (little-endian)
     0     4  magic "DTCM"
     4     4  u32 layout version (1)
     8     4  u32 m: forward cells along the triangle's side (the face has n = 2m + 1 nodes a side)
    12     4  u32 q: inverse cells along the gnomonic triangle's side
    16     8  f64 τ
    24     4  u32 flow steps (Moser's flow, RK4, t from 0 to 1; 0 for τ = 0, no flow)
    28     4  u32 K: the corner patch's extent, coarse cells from the corner (0: no patch)
    32     4  u32 F: the corner patch's fine cells to a coarse cell (0 when K is 0)
    36     4  u32 reserved (0)
    40    32  name, ASCII, NUL padded
    72    40  generator commit, ASCII hex, NUL padded
   112        forward nodes: for i in 0..=m, for j in 0..=i: a, b (f64); node (i, j) at u = i/m, v = j/m
              inverse nodes: for k in 0..=q, for l in 0..=k: u, v (f64); node (k, l) at a = k/q, b = l/q
              patch nodes, n = K·F: for i in −1..=n+1, for j in −1..=i: a, b (f64); node (i, j) at
              u = 1 − K/m + i/(m·F), v likewise; i = n + 1 and (n + 1, n) are ghosts
```

The file's length is exactly what the header implies; a reader refuses any other length, magic or
layout version.

**Evaluation** (Rust `deeptime_format::cube_map`, Java `worlddata/CubeMap`, JS `DTCubeMap`; the
same bits in all three, pinned by the parity fixture; only +, −, ×, IEEE division and square root
in a fixed order, no fused multiply-add):

- **Forward** (face, u, v) → unit vector: fold to the triangle (|u|, |v|, swapped when |v| > |u|),
  evaluate (a, b) there, undo the swap and the signs, then n = (1, a, b)/√((1 + a²) + b²) in the
  face frame, turned to the cube frame by the face's `frame`.
- **The bicubic.** A Catmull-Rom bicubic over the forward nodes, rows along u first, then v, with
  the cell index found by a clamped floor (truncation, then minus one where it overshoots). The
  nodes cover the quadrant [0, 1]² (the triangle and its swap, (a, b) → (b, a)) and one ring of
  ghosts: beyond u = 1 the mirror of node (m − 1, j) across the edge's plane, (a, b) → (1/a, b/a);
  beyond v = 1 its swap; past the corner a linear extrapolation along the edge, its swap, and along
  the diagonal; beyond u = 0 and v = 0 the sign flips.
- **The corner patch** (`conformal-n513` only; ledger SCL-104). The conformal map behaves like
  (z − z_c)^(4/3) at a cube corner, so a finer table of the same kind covers the last K coarse cells
  at the corner, F fine cells to a coarse cell, with its ghosts stored. Within K − 1 coarse cells of
  the corner the fine bicubic is used, beyond K the coarse one, and between them
  coarse + w²(3 − 2w)·(fine − coarse), w the Chebyshev distance inward from K cells in coarse cells:
  continuous, with a continuous Jacobian.
- **Jacobian**: the derivatives of the same bicubic, d n = (d g − n·k)/√((1 + a²) + b²) with
  k = (a d a + b d b)/√((1 + a²) + b²), per face axis. `CubeWorld::jacobian` turns it into planet
  metres per block east and north (the 2 × 2 matrix), σ = √|det| and the angle.
- **Inverse** unit vector → (face, u, v): the face is the vector's largest component (ties to x,
  then y, then z; a zero component counts as positive), (a, b) from the face frame, folded to the
  gnomonic triangle; a bilinear start from the inverse nodes (over the square [0, 1]², the triangle
  and its swap), then 4 Newton steps on the forward bicubic (ledger SCL-103), then, in this order:
  u above 1 becomes 1, u below 0 becomes 0, v above u becomes u, v below 0 becomes 0, and a = 1 sets
  u = 1, b = 0 sets v = 0, a = b sets v = u.

Measured (plan M1 and M2, on the test VM): the conformal table against one of twice the
resolution 7.0·10⁻⁷ rad (0.0036 blocks at 32k), the Moser one 5.5·10⁻⁵ rad (0.28 blocks); the
bicubic's determinant d(a, b)/d(u, v) positive everywhere, also within one block of every corner at
32k (conformal smallest 0.0072, Moser 0.52); round trips on 10⁶ random points within 6.7·10⁻¹⁰ (conformal) and
4.5·10⁻¹² blocks (Moser) at 32k; the two faces' values on an edge within 2.8·10⁻¹⁶; about 30 ns a
forward. The science page is `docs/science/seamless-world.md`.

## Tile pack (`.dtr`)

```text
offset        size  field
0             4     magic "DTR1" (0x44 0x54 0x52 0x31)
4             4     u32 header_len = 16 + json_len (bytes from file start to the end of the JSON)
8             2     u16 format_minor (6 for "0.6")
10            2     u16 flags, reserved, 0
12            4     u32 CRC32 of bytes 0..12 followed by the JSON bytes
16            json  LayerDesc, UTF-8 JSON, no BOM
header_len    pad   zero bytes up to the next multiple of 8 = index_offset
index_offset  16·N  tile index, N = tiles_x · tiles_z entries, tile t = tz·tiles_x + tx:
                      u64 offset (absolute), u32 stored length, u32 CRC32 of the stored bytes
…             pad   zero bytes to a multiple of 8
tiles               each tile starts on a multiple of 8 and is followed by zero padding to one
```

`tiles_x = ceil(w / tile)`, `tiles_z = ceil(h / tile)`. Tile `(tx, tz)` covers columns
`[tx·tile, tx·tile + tw)` and rows `[tz·tile, tz·tile + th)` with `tw = min(tile, w − tx·tile)`,
`th = min(tile, h − tz·tile)`: **edge tiles are cropped**, not padded. Its cells are row-major
within the tile, `cell_bytes` each, so its decoded size is `tw · th · cell_bytes`.

- `codec: "raw"` — the stored bytes are the decoded cells.
- `codec: "lz4_block"` — the stored bytes are one LZ4 **raw block** (no frame, no size prefix,
  no checksum), as written by `lz4_flex::block::compress` and read by lz4-java's
  `LZ4SafeDecompressor` given the decoded size above. Tiles are independent.

### LayerDesc JSON

```json
{"name": "elevation", "dtype": "i16", "scale": 0.0625, "offset": 0.0, "units": "block",
 "grid": {"name": "L3", "w": 512, "h": 256, "r": 2, "wraps_x": true,
          "origin_block_x": -512, "origin_block_z": -256},
 "tile": 256, "codec": "lz4_block", "nodata": -32768.0}
```

- `dtype`: `u8`, `i8`, `u16`, `i16`, `u32`, `f32` (IEEE 754 binary32), or `horizon_stack` (7K bytes a cell).
- physical value = raw · `scale` + `offset`. `nodata` (optional) is a **raw** value.
- `stack_depth` (only for `horizon_stack`): K.
- `tongue_band_keys`, `tongue_minor_share` (optional, only for `horizon_stack`): the column rule's
  band B and minor share m (below); absent: B = 64, m = 0.15. A baked run writes the preset's
  (`structure.strata_tongue_scale` × 64, rounded; `structure.strata_minor_share`).
- `min_blocks` (optional, only for `horizon_stack`): per class index, the least thickness in blocks
  of a horizon of that class inside a column (the column rule's step 5); absent, or 0 or 1 for a
  class: none. A baked run writes 8 for the regional-metamorphic classes (slate, phyllite, schist,
  gneiss, quartzite, marble, greenstone, amphibolite), 16 for the plutonic ones (granite,
  granodiorite, diorite, gabbro, anorthosite) and 1 for the rest; the synthetic worlds
  write none.
- Unknown fields must be ignored by readers.

### `horizon_stack` cells (plan §5.7.4)

`cell_bytes = 7K`: K × i16 horizon top elevation (1/16 block), K × u8 class index (into
`strata/classes.json`), K × u16 **key top**, K × u16 **key base**. Horizons are ordered top-down;
horizon k's rock fills from its top down to horizon k+1's top, the last one down to `min_y`.
Unused horizons (only after the used ones) have top −32768, class 255 and keys 65535.

**Stratigraphic keys.** A key orders units the same way in every cell (it grows downwards, older
and deeper); a horizon's span [key top, key base] (top < base) covers the units it stands for, and
the spans of one stack grow downwards without overlapping (key top of k+1 ≥ key base of k). A gap
between two spans is a hiatus: that cell has none of those units. The key space is the baker's
choice (the column rule below reads any keys that grow downwards). The synthetic worlds give
horizon h the span 4096·h … 4096·h + 4095 in every cell.

**The bake's key space** (`deeptime_core::structure::columns::key`) is ordered by metamorphic
grade first, so that any blend of neighbouring stacks by key has the grade rising downwards (no
schist on shale; Phase 1 has no thrusts that could put it there):

| keys | what |
|---|---|
| 0–4095 | regolith |
| 4096–32767 | band 0 (unmetamorphosed), the strata log: 4096 the present, 32767 the start of the run, linear in age |
| 32768–34559 | band 0, the basement pools |
| 34560 + 4480·(b − 1) … + 2687 | band b = 1…6, the strata log, linear in age |
| … + 2688 … + 4479 | band b, the basement pools |
| 61440–65535 | the mantle |

Bands are the formation temperatures of the regional metamorphic classes: 1 from 250 °C (slate),
2 from 300 (quartzite, marble, greenstone), 3 from 350 (phyllite), 4 from 500 (amphibolite), 5 from
525 (schist, the garnet zone), 6 from 650 (gneiss). A unit's band is that of the hottest formation
temperature at or above it in its own column (so a column's bands never decrease downwards);
plutonic pool units (granite, granodiorite, gabbro) are always in band 6, with the gneiss: the
crystalline basement. In a band's strata log a layer spans the ages from the base of the layer
above it to its own base, the top layer to the present where its cell still accumulates (net
deposition over the cell and its eight neighbours at the last step) and only its base age where
erosion has taken an unknown part of it, and the young loose part of a layer the upper half of
its span. In a band's pools, pool q of `plates::l1::TOP_ORDER` has the 256 keys from q·256, each
of its metamorphic steps 64 of them (one tongue band at the default B). The parts — the regolith,
each band's log, each band's pools, the mantle — are the **key domains**: a baked horizon's span
lies inside one of them.

**The column rule** (format 0.2; Rust `deeptime_format::strata::stack_at`, Java
`worlddata/Columns.stack`; both must give equal answers). A cell's stack is read as a height
function of the key: at key s ≤ the first key top, the first top; inside a span [kt, kb] of
horizon h, t − (t − b)·((s − kt)/(kb − kt)) with t its top and b the next top (min_y after the
last); across a gap or past the last span, the next top (min_y). For block (x, z):

1. x wrapped into [−C/2, C/2), u = (x − origin_x)/r, v = (z − origin_z)/r, i0 = ⌊u⌋, j0 = ⌊v⌋. The
   16 cells (i0 − 1 + a, j0 − 1 + b), a, b = 0…3 (x wraps, z clamps), in the order n = 4b + a,
   have weights wₙ = Bₐ(u − i0)·B_b(v − j0) with the uniform cubic B-spline: for g = 1 − f,
   B₀ = g·g·g/6, B₁ = (3f³ − 6f² + 4)/6, B₂ = (−3f³ + 3f² + 3f + 1)/6, B₃ = f³/6 (f² = f·f,
   f³ = f²·f; evaluated in exactly this order).
2. The keys: every key top and base of the 16 stacks' used horizons, sorted, duplicates removed:
   s₀ < s₁ < … < s_{m−1}. The height at key q is Y_q = y_{5,q} + Σₙ wₙ·(y_{n,q} − y_{5,q}) (the sum in
   order n = 0…15, y_{n,q} cell n's height at s_q; cell 5 is (i0, j0)). No key: no horizons. One key:
   one horizon, the first used horizon of the first cell that has one, top Y₀.
3. For each interval q (s_q, s_{q+1}), top Y_q, bottom Y_{q+1}: each cell n with wₙ > 0 whose
   horizon h spans the interval (key top ≤ s_q and s_{q+1} ≤ key base) and whose thickness
   t = wₙ·(y_{n,q} − y_{n,q+1}) is positive gives its class: per class, the order (least key top +
   key base of its horizons), the unit (least key top) and the summed t, classes in the order cells
   first give them. One class: emit (Y_q, class, unit). Several: sort by (order, class) and
   **intertongue** with the layer's band B and minor share m (LayerDesc): T = Σ t; shares
   pᵢ = max(tᵢ/T − m, 0) (if all are 0, pᵢ = tᵢ/T), P = Σ pᵢ; for each band n from ⌊s_q/B⌋ to
   ⌊(s_{q+1} − 1)/B⌋ (integer division) with base B·n, walking the classes in order with a running
   sum S (from 0): lo = base + B·(S/P), S += pᵢ, hi = base + B·(S/P),
   clipped to [s_q, s_{q+1}]; if lo < hi, emit (Y_q − (lo − s_q)/(s_{q+1} − s_q)·(Y_q − Y_{q+1}),
   class, unit).
4. Emitting (top, class, unit): t = ⌊top⌋, at most the last emitted top; if it equals the last
   emitted top, that horizon (left with no block) is removed; then if the last emitted horizon has
   the same class, nothing is added, else (t, class, unit) is appended.
5. Slivers, where the layer has `min_blocks` (N_c for class c; 1 for a class past its end): with
   the emitted horizons h = 0 … n − 1 (tops T_h), the first is kept and a carried top C starts
   unset; then for h = 1 … n − 1 in order, with t = C if it is set, else T_h: if h < n − 1 and
   N_class(h) > 1 and t − T_{h+1} < N_class(h), horizon h is removed and C = t (its blocks go to
   the horizon under it); else C is unset and, if its class is the class of the last horizon
   kept, it is removed (it continues that one), else it is kept with top t.

The result is continuous across cell edges (a cell whose weight is zero changes nothing), a bed
present in every cell has smoothly interpolated top and base, a bed some cells lack thins out
towards them, and a facies change (cells disagreeing about the rock of the same keys) becomes
tongues of the two classes, each thinning towards the cells that lack it, on a band grid of the
keys that every column shares (so the tongues follow the bedding). With `min_blocks`, a unit of a
named class that ends sideways ends at a step of that many blocks instead of thinning to a sheet
one block thick; a class the layer does not name still thins out continuously.

**The P1-07 bake** (`deeptime_core::structure::bake_horizons`) makes stacks with these properties:
the first horizon's top is the cell's surface (y = sea_level_y + (h − sea level)/m_per_block_v,
clamped to [min_y + 1, max_y]); every horizon is at least one block (16 raw) thick, the last one
down to `min_y` too; at most K are used; neighbours differ in class or in key domain; every
horizon carries the key span of the units it stands for — a merge keeps the union where both
members lie in one key domain, and the span of the member whose class it keeps where they do not
(the other's keys become a hiatus), so no horizon's height is spread over keys that other cells
give to other rocks. A horizon of regional-metamorphic rock under the first horizon and over the
last is at least 8 blocks thick and one of plutonic rock at least 16 (a thinner one gives its
blocks to the horizon under it where that is of its key domain, else to the one over it where
that is, else to the one under it; that horizon keeps its class and keys): such rock is a
regional body, not a sheet between other rocks. A stack walks the column of
`structure::columns` from the top: regolith, the strata log (the young, shallow part of a clastic
deposit as its `loose_equivalent`, buried loose material and palaeosols as their `lithifies_to`:
glacial till, the deep-time glaciers', the ice ages' and a snowball Earth's alike, becomes
`diamictite`, and young, shallow diamictite shows as `till`),
the plate model's rock pools less the log, and peridotite below the crust. No unit is cooler
than the rock above it: a sedimentary or volcanic pool under a metamorphosed log layer steps
along its metamorphic path at least as far as that layer's formation temperature (the pools
themselves know only today's geotherm), and a plutonic pool stays what it is (it may be younger
than the metamorphism of its roof) but for granite turning to gneiss at the 650 °C isotherm.
Plutons and contact aureoles are bodies (roles `pluton` and `aureole`, below), not horizons.

**The 0.2 bake** smooths the stacks laterally before writing them, because the strata log is
carried cell by cell and neighbouring cells' stacks differ more than the rocks would: each cell's
new stack is the weighted mean (σ = `structure.strata_smoothing_cells` L1 cells, default
10; x wraps, z clamps; per axis a Gaussian of 1.5 σ lowered by its value at the window's edge, so a
sample at the edge weighs nothing and the sample lattice leaves no period, P1-23; effective width
0.85 σ) of its neighbours' height functions, cut at every key top and base any of
them has; each interval takes the class holding the largest weighted thickness there, equal
neighbours of one key domain join; the cell's own surface stays its first top: the regolith and
the loose sediment on top (the unconsolidated classes) lie on it with their mean thickness,
neighbours' rock units above their base pinch out there, and where the cell stands above the mean
the thickest rock unit that begins within 16 blocks of the mean rock surface fills the
difference, the thinner beds over it keeping their thickness; then the stack is settled again (at least a block thick, metamorphic and plutonic
rock inside it at least 8 and 16 blocks, at most K). The neighbours are the cells of a lattice of
step ⌈σ / 2.5⌉ anchored at cell (0, 0) within 5 steps (11 × 11 samples whatever σ, and every cell
samples the same lattice); σ 0 bakes each cell's own stack. `notes.bake.strata.realism` records the share of the stored
stacks a geologist would object to (a grade inversion, a metamorphic sliver, a plutonic sheet,
plutonic rock over other rock; `deeptime strata-check` measures the same on the columns of a
world-data folder). `notes.bake.strata.smoothing` records
σ, the step, the window and the tongue parameters.

**The P1-08 bake** writes them with one change: the first horizon's top of every cell is raised,
never lowered, to ⌈max E + max A⌉ + 1 blocks (at most `max_y`), E and A the `elevation` and
`detail/amplitude` of the L3 cells in the 4 × 4 L1 cells i − 2 … i + 1, j − 2 … j + 1 (x wraps, z
clamps) — every L3 cell any block interpolating from this cell reads. The column rule interpolates
tops between the four cells around a block, so the first horizon then reaches over the terrain
everywhere and the rock of the top unit fills amplified ridges; deeper tops are the history's.

## The terrain height (format 0.1)

The height H(x, z) a Deep Time generator builds (`deeptime:terrain`, `deeptime:depth`, spawn) is,
with x folded into [−C/2, C/2) and both terms evaluated there as doubles:

  H = E(x, z) + A(x, z) · N(x, z)

E the `elevation` layer bilinear, A the layer `notes.bake.detail.layer` bilinear, N =
`Noise::fbm(x, z, wavelength_blocks, octaves, salt)` of `Noise::new(seed, C)` below. A world without
`notes.bake.detail` (every synthetic world) has H = E. Java: `WorldData.height`; Rust:
`deeptime_format::ColumnWorld::height`.

**What the game draws.** Vanilla evaluates the density at noise-cell corners four blocks apart and
interpolates between them, so a corner column's top is ⌈H⌉ − 1 exactly and the columns between
follow the bilinear H of the four corners. For that to hold on steep ground, `deeptime:terrain`'s
density (H − y)/scale doubles its scale from k (8) until it reaches 0.8 × the largest height difference to the
eight neighbouring corners (Java `worldgen/TerrainScale`, 2026-09-29): with the fixed k the router's
cave band (the terrain value only within 1.5625·scale blocks of the surface) broke faces steeper
than about three blocks per block into three-block benches and 20-block steps every four blocks
(the "striped snow" of the P1-08 renders). The scale changes nothing the world data says: H, the
corner columns and `ColumnParityTest` are as before; `-Ddeeptime.steepScale=0` restores the fixed
scale.

## The mountain character (in `elevation`, 2026-09-29)

The amplifier's last level ends with the mountain character (`deeptime_core::surface::amplify::
character`, the decisions log's "block-scale mountain character in Phase 1"): drainage-aligned
gully octaves (stripe noise whose crests follow the slope of the smoothed terrain, depth ∝ slope ×
wavelength, capped by the local relief, the wavelength and the height above the local base level,
fading out on rivers and at the coast), ridged fold stripes along the plate model's fold trend where
an orogen is recorded, unsharp masking of young terrain (orogeny age, uplift) and smoothing of old
terrain, and cliff bands (a smooth terrace) where the surface rock is hard and steep; then, where it
closed a pit deeper than 1.5 blocks, it is softened back, and the flood makes the surface drain again
(no new sinks). `amplify.ruggedness` (0..2, default 1; the editor's "Ruggedness") scales all of it;
0 leaves the passes' terrain bit for bit. Everything is baked into the `elevation` layer, so the
game and the column evaluator read it like any other terrain: the contract (H = E + A·N, the
layers, the column rule) is unchanged, and it added no format change. `notes.amplify.character` records what
it did (cells changed, mean and largest change, the flood after it, milliseconds); the keyframes
carry the youth and fold strength it read (`amplify/youth`, `amplify/fold`, L1).

## The detail layer (`rasters/detail/amplitude.dtr`, P1-06)

**Lambert-frame worlds**: N as below. (The Petroff–Guyou worlds' fold and blend through their
half-turn edges went with them, cube plan §7.4.)

**Cube worlds (0.7).** N is read on the cube's surface (`DetailSpec::cube_noise`, Java
`DetailNoise.cubeAt`; SCL-215), not in net coordinates, whose axes turn at the turning seams. N at
block (x, z) is taken at the block's centre p = (x + ½, z + ½): the block (⌊p⌋) is folded
(`Cube::fold`, Java `CubeLayout.fold`, "The world's layout"), and on a face the point's local coordinates are
(u, v) = (i + (pₓ − ⌊pₓ⌋), j + (p_z − ⌊p_z⌋)) with (i, j) the block's cell; past a net edge the point
is moved by the fold's motion (`iso`, k quarter turns then the translation), giving r, and
(u, v) = (i + (rₓ − qₓ), j + (r_z − q_z)) with q the block the fold shows and (i, j) its cell; past
the net's corners N = 0. With half = s/2 and the face's frame [c, e_u, e_v] (`Face::frame`), the
surface point is P = half·c + (u − half)·e_u + (v − half)·e_v (each component summed in that order),
on the face whose axis a is the non-zero component of c. Value noise of wavelength λ there:
m = max(1, round(s/λ)) cells along every cube edge; on the two free axes a₁ < a₂ the index is
l = (P + half)·m/s (in that order), the corners are (⌊l₁⌋, ⌊l₂⌋) … (⌊l₁⌋ + 1, ⌊l₂⌋ + 1) and the fixed
axis's index is m when c's component is +1 and 0 otherwise; the lattice value is
h₃(salt, i_x, i_y, i_z) = ((mix(mix(mix(mix(seed ⊕ salt·0x9E3779B97F4A7C15) ⊕ i_x·0xD6E8FEB86659FD93) ⊕
i_y·0xA0761D6478BD642F) ⊕ i_z·0xE7037ED1A0B428DB) >> 11) · 2⁻⁵² − 1, blended with the fade along a₁
first (a = h₀₀ + (h₁₀ − h₀₀)·t₁, b = h₀₁ + (h₁₁ − h₀₁)·t₁, a + (b − a)·t₂). The octaves sum as below.
Two faces share the lattice points of their common edge, so N is one function on the cube's
surface, continuous through every seam and corner, and a block off the faces reads exactly its
fold partner's N.


`deeptime:terrain` (plan §7.1) adds **A(x, z) · N(x, z)** to the L3 height, where A is this layer
(u8, physical = raw / 16 blocks, L3 grid, sampled bilinearly like `elevation`) and N the noise
below. The amplifier writes `elevation` and this layer tile by tile, in any tile order (the tile
pack's index is patched when the last tile is in, `deeptime_format::TilePackWriter`; the file is
byte for byte what `write_tile_pack` writes for the same cells).

A = min(drama · f · relief, m · drama) · (1 − smoothstep(a₀, a₁, area_km²)), where relief is the
range of the L3 heights over the 5 × 5 cells around the cell, in blocks, `area` the cell's effective
drainage area, smoothstep(t) = 3t² − 2t³ on the clamped t = (area − a₀)/(a₁ − a₀), and drama the
preset's `surface.drama` (0 to 2, default 1). Defaults: f = 0.12, m = 6 blocks, a₀ = 25 km²,
a₁ = 250 km² (`amplify.detail`).

**N(x, z)** (block coordinates, IEEE-754 double arithmetic with `+ − × /`, `floor` and `round` only;
`deeptime_format::detail::Noise` is the reference (moved there from `surface/detail.rs` in P1-08),
`worlddata/DetailNoise` its Java twin; both tests pin the same values):
value noise of wavelength λ has n = max(1, round(C/λ)) lattice cells around the planet, spacing
s = C/n, u = (x + C/2)/s, v = z/s; the corners are (⌊u⌋ mod n, ⌊v⌋), (⌊u⌋ + 1 mod n, ⌊v⌋),
(⌊u⌋ mod n, ⌊v⌋ + 1), (⌊u⌋ + 1 mod n, ⌊v⌋ + 1) with lattice value
h(salt, i, j) = ((mix(mix(mix(seed ⊕ salt·0x9E3779B97F4A7C15) ⊕ i·0xD6E8FEB86659FD93) ⊕
j·0xA0761D6478BD642F) >> 11) · 2⁻⁵² − 1 (u64 wrapping arithmetic, `mix` splitmix64's finaliser
z = (z ⊕ z>>30)·0xBF58476D1CE4E5B9; z = (z ⊕ z>>27)·0x94D049BB133111EB; z ⊕ z>>31), blended
bilinearly with the fade f²(3 − 2f). N is `octaves` (default 3) of them at λ, λ/2, λ/4 (salt + octave)
with weights 1, ½, ¼ divided by their sum, λ = `amplify.detail.wavelength_blocks` (32). It is
periodic in x with period C (the lattice is indexed mod n), which is what keeps the wrap seamless.

## Columns (the column evaluator, format 0.1; its stack rule 0.2)

What Deep Time decides for block column (x, z) — Rust `deeptime_format::column::ColumnWorld::sample`
(behind `deeptime inspect-column`, `deeptime parity` and the JNI `sampleColumn`), Java
`worlddata/Columns.sample` (which the strata pass's column rule is part of). Both must give equal
answers for every column (`ColumnParityTest`):

1. `wx` = x folded into [−C/2, C/2). `elevation` = E(wx, z), `detail` = A·N at (wx, z) (0 without a
   detail term), `surface` = H = E + A·N (E alone without a detail term); `top_y` = ⌈H⌉ − 1 clamped
   to [min_y − 1, max_y] (min_y − 1 when H is not finite).
2. `water` = the `water_level` layer's nearest cell at (wx, z), null where nodata or absent.
3. `horizons` = the stack at (x, z) by the column rule of "`horizon_stack` cells", read through the
   world's folds and faults where it has any ("Structural events", the deformed column rule):
   `[top y, class index, unit key]`, top-down.
4. `bodies`: every body whose AABB footprint holds (wx, z) (z inside [min z, max z]; wx inside the x
   span with wrap), ascending id; for each, y from min(aabb max y, max_y) down to max(aabb min y,
   min_y) located at block (wx, y, z) ("Bodies", step 1–3); each maximal run of inside blocks is
   `[body id, top y, bottom y]`.
5. `rock`: for y from `top_y` down to min_y, the class name of the horizon holding y (none above the
   first top, or for an index outside `strata/classes.json`), then the first body of step 4 (by
   ascending id) that holds block (wx, y, z) and names a class there: its `rock_class`, a zone's
   where one applies (the last zone that applies and names one); a body naming none leaves the
   class to the next. Where bodies overlap the lowest id keeps the block (the strata pass paints
   in id order and never paints a block a body has taken). Runs of equal class are
   `[top y, bottom y, class or null]`.
6. `hydro` (P1-22): the column's water as "Water columns" decides it: `{"kind", "surface", "ground",
   "level", "top", "bed", "min_ground", "max_ground", "seal_from", "river_biome", "river"}`, `river`
   `{"seg", "t", "d", "w", "hw", "depth"}` or null, absent values null.

`parity/columns.json` (`deeptime parity --world-data <dir>`): `{"format_version", "world" (the
manifest's source), "circumference", "min_y", "max_y", "seed" (decimal), "classes", "columns": [ … ]}`,
each column the object above plus its `x` and `z`. Numbers are serde_json's shortest round-trip
form: read them with a correctly rounded parser (Java's `Double.parseDouble`) and compare exactly.
The 500 columns: 10 % within 40 blocks of the seam, 10 % near and past the poles, 20 % inside
random bodies' boxes (none when the world has no bodies), with a river network 10 % on and beside
random river vertices and 5 % around the mouths of rivers that end in lakes, with lakes 5 % within 8
blocks of random shore cells of `water_level` (wet beside dry), the rest anywhere with x over two laps.

### Inspecting a column (added 2026-10-19)

`deeptime inspect-column --world-data <dir> x z` (without `--parity`) prints the column object
above plus what a reader wants to know about it; the workbench's Inspect panel shows the same JSON,
and its sections (`/api/runs/<id>/rock/section`) carry the same fields per sampled column. None of
it is part of the parity contract (`--parity` prints the column alone), the game reads none of it,
and it adds no physical value: it reads the world data, the key layout of
`deeptime_core::structure::columns::key` and the folder's `preset.json`.

- `lat`, `lon`, `world`, `layers` (every other layer's nearest value, and its bilinear one for a
  continuous layer), `sites` (the sites whose box holds the column; `--why` adds each one's forming
  event and chain).
- `strata`: the horizons `{top, bottom, class, unit}`, and per horizon `unit_info`, its unit key
  decoded (`deeptime_core::column_info`): `part` (`regolith`, `log`, `pool`, `mantle`, or
  `intrusion` for a horizon an intrusion event made: its key and class are the event's), `band`,
  `peak_c` (`[lower, upper]` °C of the band, upper null for the last; null for the regolith and the
  mantle), `grade` (words), `age_ma` for a log unit (Myr before the end of the history at the unit's
  top: `(k − first) / (last − first)` of the log's span `(now − t_start).max(1)` Myr, the inverse of
  `key::of_age`, so to within one key) and for an intrusion (its event's age), `key_myr` (one key's
  span), `pool` and `pool_step` for a basement unit, `parent` (`kind` `pool`, `inferred`, `itself`
  or `none`, `rocks`, `text`) and `text`. A basement unit's parent is its pool; for a unit some steps
  along its class's path also the pool's classes that reach its class at that step, worked out from
  the columns' own pool rule. A metamorphic log unit's parent is **inferred**: the strata log keeps
  only what the deepest burial made of a layer, so `rocks` lists the sedimentary and volcanic
  classes whose metamorphic path reaches its class. Plutonic basement is keyed in the last band
  whatever its temperature, which `grade` says.
- `body_details`: per body run of step 4, `{id, top, bottom, site, role, shape, rock_class, age_ma,
  deposit_type, zones}`, `age_ma` the body's own (null: no age), `deposit_type` its site's, `zones`
  `{name, rock_class, material}`; and `drawn_scale` `{size_mult_h, size_mult_v, ledger, text}` when
  the body's deposit type is drawn larger than the deposit model (its `balance.size_mult_*` as the
  run's preset selected the types, `feature_scale.deposits` included): "coal seam drawn 35 x its
  modelled thickness (game choice, ledger DEP-127)". Other types cite ORE-055 (DEP-195 for the
  porphyries).
- `crust`: `{type, type_name, age_myr}` from `meta/crust_type` and `meta/crust_age_myr` at the
  column (nearest cell); `age_myr` is null on continental crust, which the plate model gives no
  formation age. Null when the folder has neither layer.
- `structure_events`, youngest first: the column rule's candidates ("Structural events") whose
  window weight at the column is above 0, and any other candidate that changed the column's stack.
  `{id, kind, t_years, age_ma, key, weight, changed}` plus, for a fold, `wavelength`, `amplitude`
  and `uplift` (blocks the fold lifted the column's rock here); for a fault, `plane_y` (the plane's
  height at the column), `throw` and `heave` (weight x slip); for an intrusion, `class`, `inside`
  and `top_y`, `bottom_y`. `changed` compares the deformed stack with every candidate against the
  stack with all but that one (the column rule reads only the column's own candidates, so this is
  exact); it is null past 64 candidates. `structure_candidates` counts the candidates.
- `units_note`: how to read `unit_info`.

## Rivers (`rivers/rivers.dtrv`, P1-22)

Code `crates/deeptime-format/src/rivers.rs` (Java `worlddata/Rivers`); written by the bake's water
part (`deeptime_core::hydro`, `bake::water`; plan §8.4). `DTRV` fixed fields (52 bytes, layout 2):
`u32 C, u32 tile (32), u32 tiles_x (C / tile), u32 tiles_z (2·Zo / tile), u32 V vertices, u32 K
chains, u32 E index entries, f32 biome_min_width, f32 bank_zone, f32 bank_ratio, i32 origin_x, i32
origin_z, u32 layout (2)`. Both readers refuse a layout other than 2 by name (a layout-1 file, whose
header is 128 bytes long, as layout 1). Sections: 0 V vertices of 24 bytes, 1 K chains of 20 bytes,
2 `tiles_x · tiles_z + 1` u32 offsets, 3 E u32 entries.

```text
vertex                                         chain
0   4  f32 x (block; continuous along a chain)  0   4  u32 first vertex
4   4  f32 z (block)                            4   4  u32 vertex count (≥ 2)
8   4  f32 w: water surface (fills y < w)       8   1  u8 mouth: 0 sea (or the polar edge row), 1 lake, 2 river, 3 seam
12  4  f32 hw: half-width, blocks (> 0)         9   3  reserved, 0
16  4  f32 depth at the centre, blocks (≥ 1)    12  4  u32 into: the lake (mouth 1) or chain (2, 3), else u32::MAX
20  4  f32 q: discharge, m³/s                   16  4  u32 from: the chain this one leaves (a distributary), else u32::MAX
```

Chains are consecutive in the vertex array; **segment s** is vertices (s, s + 1) when both belong to
one chain. A chain runs downstream from a spring or a lake's outlet (whose first vertex is the lake
cell feeding it) to its mouth: its last vertex lies in the sea, in lake `into`, or on a vertex of
chain `into` (a confluence, where the tributary keeps its own width). w never rises along a chain.
**A distributary** (layout 2, HYD-144) has `from` = the chain it splits from at a delta: its first
vertex is (x and z bit for bit) a vertex of chain `from` (both readers check it), its last lies in
the sea, and it carries its own share of the discharge with its own width and depth; the chain it
leaves runs on with the rest. Readers sample a distributary like any chain: the column rule needs
nothing new. A projection pass that splits chains carries `from` to the part holding that vertex and
starts the distributary's first part on that part's point (a distributary whose first part does not
survive loses its `from`).
**On a cube world** (format 0.7, mouth 3, SCL-126) a chain that crosses a turning seam of the net
("The cube world's layout") is cut there into two chains: the first ends with mouth 3 on the seam,
`into` the second, whose first vertex is the same planet point on the partner edge (the image of the
crossing under the edge's isometry), with the same w, hw, depth and q. A chain crossing a sliding
seam (F3|F0, F0|F1, F1|F2, N|F0, S|F0, and F2|F3 across the period) is never cut: its x and z stay
continuous across it. `tiles_z` is then `3s / tile` from z = −3s/2 and the x fold is the period 4s.
**The index**: tile (tx, tz) covers x ∈ [−C/2 + tx·tile, …), z ∈ [−Zo + tz·tile, …) ("Grids"); entries
`offsets[t] … offsets[t+1]` of tile t = tz·tiles_x + tx are the segments, ascending, whose box grown
by `max(hw_a, hw_b, 0.71) + bank_zone + bank_ratio · max(hw_a, hw_b) + 1` touches it (x folded).

**Sampling column (x, z)** — both readers, in this order, doubles from the f32 values:
wx = x folded; over the entries of wx's tile, for segment (a, b): `k = ⌊(ax − wx)/C + 0.5⌋`,
`px = wx + k·C`; `e = b − a`; `t = ((px − ax)·ex + (z − az)·ez) / (ex² + ez²)` (0 when the length is
0), clamped to [0, 1] (`t < 0 → 0`, `t > 1 → 1`); `d = √(dx² + dz²)` with `dx = px − (ax + t·ex)`,
`dz = z − (az + t·ez)`; `hw = hw_a + (hw_b − hw_a)·t`, `hw' = max(hw, 0.71)`; likewise w, depth, q.
If d ≤ hw' the segment **covers** the column: the hit is the covering segment with the lowest w
(ties: the lowest segment id), the **deep hit** the covering segment whose channel floor
`w − max(1, depth · (1 − (d/hw')²))` is lowest (ties: the lowest segment id; HYD-120), and
`river_biome` is set when 2·hw ≥ biome_min_width. A wide river's segments overlap a column many
times over (a vertex every few blocks, a half-width of ten or twenty), and the hit is mostly a
neighbour downstream whose end cap only grazes it; its floor there is a block under w, so the bed
follows the deep hit and the water surface the hit. Otherwise, if
d ≤ hw' + bank_zone + bank_ratio·hw, it offers the **bank cap** ⌈w⌉ + ⌊d − hw'⌋; the lowest offered
is the column's `max_ground`.

**How the bake builds it** (`deeptime_core::hydro`): on the `water_level` grid, the open sea (cells at
most one block below sea level: their top block is under the sea's; connected, ≥ 5,000 km²); Priority-Flood with the pit queue from its shores (spill level F
and the cell each land cell was reached from); receivers the steepest strictly lower neighbour on F,
else that cell; depressions (F > height) of at least `hydro.lake_min_area_km2` and
`lake_min_depth_blocks` are basins. The liquid runoff of the L1 climate is accumulated as discharge
(precipitation, the mean temperature and the warmest and coldest month from `climate/month/t_sl_c_*`,
their departures from the year's mean added to the cell's mean, ± 8 °C without them; a sinusoidal
year between the two months gives the share of the year above freezing and its mean temperature
above freezing T⁺; liquid = min(P, P · share + `snow_melt_mm_per_degree_day` · 365.25 · T⁺); runoff
= liquid − ET, ET by Fu's Budyko curve on the liquid water with PET = `pet_mm_per_c` (60) · T⁺,
Holdridge's biotemperature rule, which is the surface stage's 60 mm/°C · T where there is no frost;
ground whose warmest month is below freezing gives none). **A basin's lake level is its water balance**
(`lake_balance`, on by default): a cell under water loses its runoff as land and its evaporation
(`lake_evaporation_factor` · (`lake_evaporation_base_mm` + `pet_mm_per_c` · T⁺); 0 when
`lake_evaporation` is false) less the liquid precipitation on it. A basin that, wet to its rim, still gains (inflow ≥ those losses) is a lake at
its spill level and spills the surplus. One that does not is terminal: its lowest cell becomes a
sink, a Priority-Flood of the basin alone from that cell drains its floor to it (receivers and
spill levels of the exposed floor; the basins inside it are solved the same way first, up to 8
deep), and the lake stands at the highest 1/16-block step of the basin's hypsometry (cells counted
by their spill level towards the sink) whose losses the inflow at the sink still covers, at most
one step below the rim; a lake inside the basin that this level covers becomes part of it. A
balance lake under the lake thresholds is a playa (the `playa` layer), and a basin no water reaches is
dry. **Below sea level** (HYD-100, format 0.6): the **open sea** is every cell a block or more under
sea level in an 8-connected body (x wraps) of at least 5,000 km², and the ground it **reaches** is
every cell at or under sea level connected to it through such cells. A basin whose lowest cell the
sea reaches stands at sea level at least where its floor lies a block or more under sea level, and
is never dry there, as before (the column rule floods it with the sea's water). A basin the sea does
not reach takes its balance whatever its height: a lake whose surface may stand below sea level, a
playa or dry ground. **`closed_basin`** is 1 on every cell at or under sea level that the sea does
not reach and on the cells within 8 blocks (`hydro::CLOSED_REACH_BLOCKS`, HYD-102) of one, each
taking the side of the nearest such low cell, ties to the open sea; 0 elsewhere (`hydro::
closed_basin_layer`). On the cube world the projection pass draws the layer again on the
world's own ground ("The bake's projection pass", HYD-105). The biome bake, which runs before the water bake, finds the same closed ground
on its own grid (`hydro::closed_below_sea`) and maps it as land. Basins are solved in drainage order, so a lake that does not spill passes nothing downstream.
With `lake_balance` false every basin is a full lake, as before 2026-10-01. River cells carry ≥
`river_min_discharge_m3s` from ≥ `river_min_area_km2`, a lake cell less than a block deep counting
as ground a river crosses; chains follow the largest donor; w = max(F − `freeboard_blocks`, the
level of the sea or lake downstream); width `width_coef · Q^0.5 · scale.feature_scale.rivers` and
depth `depth_coef · Q^0.4` (at least 1, at most `max_width_blocks`, `max_depth_blocks`); vertex
positions smoothed by a Gaussian of half-width `smoothing_blocks` (ends fixed; a tributary's mouth
on the smoothed vertex it joins). **Deltas** (since "hyd-130", 2026-10-23; docs/science/
rivers-and-lakes.md "Deltas and distributaries", HYD-130 to HYD-144): between two runs of the
network, each river reaching the sea builds a delta plain from its BQART sediment load: over the sea
and beach cells near its mouth, taken in a lobe ahead of the channel, until the volume
(Nienhuis's area for the load times `delta_area_mult`, times the channel depth) is spent; the plain's height lies in
(sea level, sea level + 1] (top block = sea level, falling 1/16 block a step from the mouth to the
front), its `detail/amplitude` is 0, and the raster **`delta`** (i16, 1/16 block, on the
`elevation` grid; 0 off the plains) holds what the plain added, which a re-run of the water bake
takes off first. The second run drains the raised ground, and its main stem across each plain
splits at the apex (where the main stem comes onto the plain, HYD-137) into distributaries (`from`).
**Floodplains** (since "hyd-150", 2026-10-24; rivers-and-lakes.md "Floodplains and levees", HYD-150 to
HYD-153): the second run also finds GFPLAIN's floodplains, the land cells (no lake, sea, delta plain or
dry basin floor) whose filled height lies at most h = 0.01 A^0.30 m (A in m², the area draining to the
cell's drain, its first river cell downstream) over their drain's, and the bake lays each at its
drain's bank level, the drain's filled height but at least one block over its water; `detail/amplitude`
is 0 there, and the raster **`floodplain`** (i16, 1/16 block, on the `elevation` grid; nodata −32768
off the plains) holds what the plain moved the ground by, which a re-run takes off first. The game
reads it as a category (on a plain or not): a river's bank column on a plain gets sand for its soil
top (the levee, HYD-153).
**Meanders** (since "hyd-155", 2026-10-24; rivers-and-lakes.md "Meanders", HYD-155 to HYD-159): after
the floodplains, each reach of a chain that lies on a plain, is gentler than Leopold & Wolman's (1957)
braided line and has room is redrawn as a sine-generated curve about its course (wavelength 10.9 drawn
widths, sinuosity up to 2, measured on the planet through the Lambert metric). Nothing in the layout
changes: the chains keep their order and get more vertices (spaced a 40th of the wavelength, 1 to 6
blocks), the water surface along a redrawn reach is the old course's at the same distance down the
valley, a tributary whose parent moved ends on a vertex of the parent's new channel, and
distributaries are left as they were.
**Fans** (since "hyd-165", 2026-10-24; rivers-and-lakes.md "Fans and braids", HYD-165 to HYD-170,
HYD-177):
the first network's chains are searched for apexes, cells where the river's surface falls at least
Bull's (1964) fan slope S_F = 0.022 A_D^−0.32 over the fan's free length upstream and less than it
downstream; at each, a cone of that slope (measured on the planet) is laid where it stands over
the ground and the land is no lake, playa, delta plain or larger river's channel (and nowhere
thicker than over the apex), high enough to
cover Bull's area A_F = 1.3 A_D^0.88 (square miles) and at most as high as a free fan (a quarter
circle of that area); a cell whose spill level (from the open sea and the grid's end rows) the
fans raised over its ground, on a fan or behind one, is filled to it unless the first network had a
lake or playa there (HYD-177), and on L3 a filled cell's cells stand at the spill. `detail/amplitude` is 0 there, and the raster **`fan`** (i16, 1/16 block, on
the `elevation` grid; 0 off the fans) holds what the cones and the fill added, which a re-run takes
off first.
The second network run (which then runs when there are fans and no delta plain too) drains down
the cones; no floodplain is laid on a fan.
**Braids** (since "hyd-165", 2026-10-24; rivers-and-lakes.md "Fans and braids", HYD-171 to
HYD-175): after the meanders, a reach on a fan, or on a plain and steeper than Leopold & Wolman's
braided line, divides around islands 5 channel widths long, on alternate sides. Nothing in the
layout changes: the course stays one chain, narrowed (0.9 of its width) and shallowed (0.75 of its
depth) around each island and carrying half the flow there; each island's other thread is a chain
of its own appended after the old chains, with `from` and `into` both the course's chain and mouth
`River`, its first and last vertices copies of two of the course's vertices, and its water surface
the course's at the same distance down it, lowered on a fan by as much as the cone under the thread
lies below the cone under the course, and never rising.
`notes.bake.water.params` holds the parameters (`deltas`, `delta_area_mult`, `delta_shape_k`,
`delta_max_mouths`, `delta_mouth_spacing`, `floodplains`, `meanders`, `fans`, `braids`), `stats.deltas` the counts, areas, the largest deltas
and Earth's figures beside them, `stats.floodplains` the plain's cells, km² and share of the land
(within 60° too), the same by the area its river drains, how far it moved the ground and each cell's
top block, the heights above the drains and the cells left out, and `stats.meanders` the network's
length before and after (km of planet), the course on a plain, too steep, too narrow or too short,
the reaches redrawn, their axis and path length and sinuosity (with a histogram), the mean amplitude
and largest offset, the same by discharge class, the vertex counts and the joins cut, carried or moved,
`stats.fans` the fans, the candidates and those too thin or hemmed in, their cells, km², share of the
land and volume, their area over Bull's, thickness, area, drainage area and slope (percentiles), the
fans past Bull's range of drainage areas, and `laid` (the L3 cells raised, the largest raise, the
amplitude cells zeroed, the horizon stacks' cells raised and their largest raise), and `stats.braids` the threads added, islands and islands with no room, the
network's km, the km on fans, steep on a plain, too narrow, braided and braided on fans, the same
by discharge class, and the milliseconds. `notes.bake.water`: `enabled`, `method`, `params`,
`width_scale`, `grid`, `halved`, `climate`, `seasons`, `rivers_file`, `rivers_bytes`, `stats`
(counts, lengths, largest discharge and width, milliseconds per step; since "p1-22b" also
`lake_share_of_land`, `basins` (candidates, full, below_rim, playa, dry, merged, nested_too_deep),
`playa_km2`, the lakes `below_rim`, `frozen` (mean annual temperature over the lake below
`lake_freeze_mean_c`), `arid` (P/PET < 0.2), `polar` (warmest month < 10 °C) as counts, km² and
shares of the lake area, the five `largest` lakes, and `frozen_in_frozen_band` /
`open_in_frozen_band`: the share of the frozen and of the open lake cells whose biome temperature
input `climate/temperature` lies in the frozen band, below −0.45, where the game ices the water;
since "hyd-100" also `below_sea`: `open_km2` (ground at or under sea level the sea reaches),
`closed_km2`, `closed_share_of_land`, the closed ground's `lake_km2`, `playa_km2`, `dry_km2` and
`dry_share_of_land`, `lakes_below_sea_level` and their km², and `closed_basin_radius_cells`).
`method` is "p1-22" in a folder baked before the water balance, "p1-22b" since, "hyd-100" since
format 0.6, "hyd-130" since the deltas, "hyd-150" since the floodplains, "hyd-155" since the meanders and "hyd-165" since the fans and braids; the game does not read it. `notes.bake.water.closed_basin` is `{layer, reach_blocks}`.

## Water columns (P1-22)

Code `crates/deeptime-format/src/water.rs` (Java `worlddata/Water`); the game's
`worldgen/WaterBodies` pass makes every generated column match it (after the strata pass, before
carvers and features). A function of (x, z) and the world data only, with the generator's noise cell
width `cell` (4 in `deeptime:planet`; the parity fixture assumes 4) as the shore rule's `reach`:

1. **surface** = H (the terrain height) bilinear between the four corners of the noise cell holding
   the column (corners at multiples of `cell`, fractions (wx − x0)/cell, (z − z0)/cell, the corner H
   values h00, h10, h01, h11 as `a = h00 + (h10 − h00)·fx`, `b = h01 + (h11 − h01)·fx`,
   `a + (b − a)·fz`): the generated surface. **ground** = ⌈surface⌉ − 1 clamped to [min_y − 1,
   max_y] (min_y − 1 when not finite).
   **channel** = ⌊f⌋ of the deep hit's floor f = w − max(1, depth · (1 − (d/hw')²)) ("Rivers"),
   when a segment covers the column.
2. **level** = the `water_level` cell nearest the column, else the value of the nearest wet cell
   within `reach` blocks (cells visited by column then row, first at the smallest squared distance;
   Java `WaterLevels.levelAt`). **Lake** when level is finite and ground < top = ⌈level⌉ − 1; bed =
   ground, or min(ground, channel) where a channel covers the column and level − surface < 1 (a river
   crossing lake water less than a block deep keeps its bed; `water::SHALLOW_LAKE`, HYD-122).
3. Else **sea** when ground < sea level − 1 and the column is not in a closed basin (the
   `closed_basin` layer's nearest cell is not 1; absent layer: never; Rust `WaterSource::closed`,
   Java `WorldData.closedBasin`; HYD-100, format 0.6): top = sea level − 1, bed = ground, or
   min(ground, channel) where a channel covers the column (a river run on across the shallows into
   the sea keeps its bed; HYD-121). The test is
   one function in each language, `water::sea_covers` (Java `Water.seaCovers`; HYD-106), which the
   bake's biome layers read too (`sea_covers_height`, BIO-170): a column whose height H lies in
   (sea level − 1, sea level] has top block sea level − 1 and is dry. The
   aquifer's water, which the pass tops up where the aquifer left such a column dry (it decides from
   its own noise near the surface, and leaves coastal flats a block below sea level dry). In a
   closed basin a column below sea level goes on to steps 4 and 5: dry ground unless a lake, a river
   or a bank holds it.
4. Else **river** when a segment covers the column: with t = ⌈w⌉ − 1 of the hit, the channel
   **incises** (HYD-123): drop = min(1, t − (sea level − 1)) (`water::INCISION`, 1) when t > sea
   level − 1 and the **bank ground** ≤ t, else 0; top = t − drop, bed = min(top − 1, channel − drop).
   The bank ground is the lower of the ground (steps 1's ⌈surface⌉ − 1, clamped) at the two points
   hw' + 1 blocks out on either side of the deep hit's segment, on its normal through the foot at the
   deep hit's t (points floored to blocks, x folded; a segment of no length: the column's own
   ground): where the channel's banks stand before the pass touches them. A channel whose banks do
   not stand over its water's top block is cut a block lower, so its banks hold it without being
   raised; never under the sea's top block.
5. Else, visiting the neighbours (x+1, z), (x−1, z), (x, z+1), (x, z−1) evaluated by 1–4: for each
   that holds water (river, lake, sea), `min_ground` = max(held for a river, top otherwise), where a
   river's **held** is the highest top among it and its own water neighbours (the step the pass puts
   on it where a neighbour's water stands higher; HYD-124; so the rule reads a ring of two), and
   `seal_from` = min(bed + 1, or for the sea max(bed + 1, top − 3): only its waterline); `max_ground` = the column's bank cap. The column is a **bank** when
   either `min_ground` or `max_ground` is set, else **none**. `river_biome` and `river` (the hit)
   are kept whatever the kind.

**The sea's status in a closed basin (game side).** The fluid picker of `deeptime:planet` and
`deeptime:planet_nocaves` (`worldgen/LavaFluids`) answers, for a block under sea level and at or
above the lava floor in a column whose `closed_basin` is 1, water at the lava floor's level instead
of the sea: nothing floods to sea level there, while the aquifer's own water tables under the basin
stay water (vanilla's local aquifers take the global status's fluid). Vanilla's aquifer also reads
the picker at 12 columns up to three chunks away and can bring a nearby sea's status into a closed
basin by the coast, so the pass drains a closed column: from the sea's top block down to its
ground, the open water above the column's own (a lake's or a river's top; none for other kinds) is
removed. Caves and aquifers under a closed basin are kept as elsewhere; only the sea is kept out.

The pass: a sea column the aquifer left dry is filled from the sea's top down to the ground (the
pass runs in every chunk with elevation cells within 17 blocks of sea level for this); a river column is cut or filled to `bed` (gravel on the bed of a channel under 6 blocks
wide, sand on a wider one), water from bed + 1 to top, air above; a sea column whose bed is under its
ground (a channel's, HYD-121) is cut as a river column is, with the sea's top; a lake column whose
bed a channel lowers under its ground (HYD-122) gets the channel's gravel or sand on its bed; a lake column is cut to its `bed` as a river's
channel is: generated ground over `top` is removed, ground between `bed` and `top` becomes water
(the generated ground between noise-cell corners can stand a block or two above `ground`, because the
terrain density's scale differs from corner to corner; before the cut such a column in a lake one
block deep held no water and was dry shore, flush with the lake: 22 % of one planet's lake columns),
and a cave under `bed` is plugged; a bank is lowered to
`max_ground` and raised to `min_ground` keeping its top block, and air from `seal_from` to
`min_ground` is filled; where a water column's neighbour holds water higher up, the block at the
higher top gets flowing water (level 1, resting on the water under it)
and the blocks between falling water (a waterfall). **River biomes**: the game's weirdness input is
0 (the river valley slice) at columns whose `river_biome` is set (`WorldDataTerrain.raster`).

**Lake ice (game side, not part of the column rule).** A lake column is frozen when the layer
`climate/t_mean_c`, bilinear at block (x, z), is below `notes.bake.water.params.lake_freeze_mean_c`
(preset key `hydro.lake_freeze_mean_c`, 0 °C: with a yearly cycle, ice for more than half the year;
NaN is not frozen). It is the rule of the bake's statistics and of the workbench's maps, which
apply it at cell centres (`HydroParams::frozen`; Java `worlddata/LakeIce`). The pass sets the top
water block of a frozen lake column to `minecraft:ice`, whatever the biome. A lake column that is
not frozen stays open in a cold biome: `Biome.shouldFreeze` answers false for the block at the
lake's top (⌈level⌉ − 1 by the shore rule), at generation (`freeze_top_layer`) and afterwards
(snowfall). Rivers and the sea are left to the biome. A world without the threshold or the layer
(a synthetic world, `hydro.enabled` false) has no rule.

**Playas (game side).** `playa` (u8, on the `water_level` grid) holds, for the cells of a playa
(the floor of a terminal basin whose balance lake is under the lake thresholds: "Rivers"), the
blocks of salt to lay there (`hydro.playa_blocks`, 2; 0 elsewhere). `notes.bake.water.playa` is
`{layer, rock_class ("halite"), blocks, wetland_cleared}` (the wetland flag cleared beside the
playas, "The wetland flag", BIO-178). After the water pass, in every column whose nearest cell
holds n > 0, `worldgen/PlayaPass` replaces the top n blocks of the ground (not air, fluid or
bedrock; not under water) with the block the rock palette resolves for `rock_class`, as a stratum
of that class would be: the pack's rock tags decide it.

**Carvers (game side, not part of the column rule).** Vanilla's carvers run after the pass, and the
`#minecraft:overworld_carver_replaceables` tag lists water. When `notes.bake.water.params.
protect_water_from_carvers` is true or absent (preset key `hydro.protect_water_from_carvers`, on by
default), the pass puts every water block it places or keeps in a river, lake or topped-up sea
column in the chunk's carving masks (vanilla skips masked positions), and only those: ground under
and beside the water stays carvable, so caves run under rivers and lakes and may open into a bed.
After the air carvers the lowest water block of each column that now has air under it is marked for
post-processing, so vanilla's fluid tick lets it run into the cave. Set to false, the carvers cut
through the water like any other block.

## Lava (game side, `notes.bake.underground`)

`notes.bake.underground.params` is the preset's `underground` section (its defaults when the preset
has none): `{"lava_floor_blocks": 10, "lava_lake_blocks": 54, "magma_chamber_depth_km": 3.0,
"magma_chamber_size": 1.0, "crater_lake_share": 0.25, "remove_lava_features": true, "magma_lens_count": 24,
"magma_lens_spacing_blocks": 28.0, "magma_lens_width": 1.5, "magma_mush_width": 1.0, "sea_cliff_skin_blocks": 6}`. No layer holds
lava; the game reads the first two numbers when it generates chunks (Java `worldgen/LavaRules`, `LavaFluids`,
`density/LavaLakesDensity`), and a world without the note uses the defaults. With `min_y` the floor
of the noise settings (−512) and cover = 8:

- **The lava floor.** Open space with y < `min_y + lava_floor_blocks` (never above sea level) is lava.
  In a column whose terrain height H is that low, the limit is `min(that, ⌊H⌋ − cover)`: lava stays
  8 blocks below a sea bed, and everything above the limit is the sea. 0: no lava floor.
- **Lava lakes.** A cave lake (an aquifer with its own level) may be lava only when its 40-block
  level cell `⌊y/40⌋` is centred at or below `min_y + lava_lake_blocks`; lake surfaces lie within 12
  blocks of the cell's centre. 0: no lava lakes.

Both are vanilla's rules (lava below y −54, lava lakes at y −10 and lower, in a world whose floor is
y −64) measured from the floor instead of from y 0. Vanilla's own constants flood the lowest 458
blocks of every cave, and of every sea deeper than 117 blocks, of a world whose floor is y −512.
Carver lava in the bottom eight blocks is vanilla's and not covered by these rules.

### Impact craters (`impacts/craters.json`)

The bake lists the craters the impact stage (P2-08, `surface/impacts`) leaves in its catalogue at the
present: those that still show, the buried ones and the eroded roots (craters worn through their
breccia, subducted or overprinted are not listed; a world without the impact stage has no file):

```json
{"version": 1, "t_myr": 0.0, "craters": [
  {"id": 73, "x": 2410.5, "z": -1033.0, "diameter_m": 6300, "morphology": "complex", "age_myr": 12.4,
   "depth_m": 255, "rim_m": 91, "relief_m": 180, "fate": "degraded", "breccia_m": 255, "melt_m": 0,
   "cover_m": 12, "in_surface": false}]}
```

`x`, `z` are the centre's block position (x in [−C/2, C/2)); `diameter_m` is rim to rim; `morphology`
is `simple`, `complex` or `peak_ring`; `depth_m` (rim crest to floor) and `rim_m` (rim above the
ground it struck) are the fresh crater's in game metres (the physical ones × `impacts.depth_scale`),
and `relief_m` is what erosion, burial and soil creep have left of `depth_m`; `fate` is `fresh`,
`degraded`, `buried` or `eroded_to_roots`; `breccia_m` and `melt_m` are the thickness of the breccia
lens and the melt sheet; `cover_m` the sediment over the floor; `in_surface` says the crater was large
enough (two L1 cells) to be cut into the deep-time surface and laid in the strata log, rather than
drawn by the amplifier and given a body. `notes.bake.impacts` records `craters` (the count) and
`craters_file`. The game does not read the file to build anything: the craters are in `elevation`,
their rock in the horizon stacks and in bodies of role `impact_breccia` (rock class `impact_breccia`,
with a zone `melt_sheet` of rock class `impact_melt` on the floor of a crater that has one; the
body is an ellipsoid centred on the crater's original floor, which the bake seats at the height of
`elevation` at the crater's centre less `cover_m`, so the lens lies under the floor the game draws). It is
there for tools, and Java's `worlddata/Craters` reads it for the GameTest that checks a crater in the placed world.

### Standing volcanoes (`volcanoes/standing.json`, 2026-10-22)

The bake lists the volcanoes still standing at the present, alive or dead, of every kind, at least
30 m high (VOL-064), in id order. The projection pass reads the file to cut the cone gullies into
the stratovolcanoes and to keep the steep-ground texture off every volcano's body ("The bake's
projection pass"; VOL-068, VOL-100 to VOL-105); the game does not read it.

```json
{"version": 1, "t_myr": 500.0, "fade_myr": 5.0, "volcanoes": [
  {"id": 3, "kind": "stratovolcano", "x": 2410.5, "z": -1033.0, "radius_m": 14000,
   "height_m": 1700, "crater_rho": 0.07, "born_myr": 498.0, "extinct_myr": null}]}
```

`kind` is `stratovolcano`, `shield` or `caldera_complex`, as in `volcanoes/active.json`; `x`, `z` are the
vent's block position (moved to the world's frame by the projection, as
`volcanoes/active.json`'s); `radius_m` the cone's drawn basal radius and `height_m` its height,
both in game metres; `crater_rho` the summit crater's or caldera's rim as a share of the radius
(`null`: none); `born_myr` and `extinct_myr` (`null` while active) are history times, so the
time since its first and since its last activity are `t_myr − born_myr` and `t_myr − extinct_myr`;
`fade_myr` is the volcano stage's fade time (VOL-065).

### Active volcanoes (`volcanoes/active.json`) and their magma

The bake lists the volcanoes the volcano stage leaves **active at the present** (dead ones and those
eroded or subducted are not listed; a world without the volcano stage has no file):

```json
{"version": 1, "t_myr": 0.0, "volcanoes": [
  {"id": 412, "kind": "stratovolcano", "x": -6120.5, "z": 233.0, "radius_m": 21000, "height_m": 2300,
   "activity": 0.62, "oceanic": false, "silicic": false, "age_myr": 3.0,
   "crater": {"radius_m": 1700, "depth_m": 180, "caldera": false}}]}
```

`kind` is `stratovolcano` (arc), `shield` (oceanic hotspot) or `caldera_complex` (continental
hotspot). `x`, `z` are the summit's block position (x in [−C/2, C/2)); `radius_m` the basal radius;
`height_m` the edifice's relief; `activity` 0–1 (an arc volcano's magma flux at its last step, a
hotspot volcano's mean nearness to its plume); `crater` its summit crater or caldera, or null.
`notes.bake.underground` records `active_volcanoes` (the count) and `volcanoes_file`. Spreading
ridges and rifts are not exported.

The game builds each volcano's magma from the list, the terrain height H and four more settings of
`notes.bake.underground.params` (Java `worldgen/Magma`, placed by `worldgen/MagmaPass` in the surface
step after the water pass). With `mH`, `mV` the metres per block, (cx, cz) the summit column rounded,
`floor` = ⌈H(cx, cz)⌉ − 1 and `lava_floor` the lava floor's y:

- **Chamber**: an ellipsoid of lava. Horizontal radius `rh` = clamp(0.1 · `radius_m`, 1200, 4500) ·
  `magma_chamber_size` / mH blocks (at least 2); half-height `rv` = 0.2 · that radius in metres / mV
  (at least 4). Its top is `magma_chamber_depth_km` under `floor`. It is raised to stay 8 blocks
  above `lava_floor`, squeezed to stay 24 blocks under `floor`, and dropped (the volcano gets no magma)
  when less than 3 blocks of half-height are left. No chamber block becomes lava within 8 blocks of
  its column's ground (the sea bed under the sea). `magma_chamber_size` 0: no magma anywhere.
- **Conduit**: the centre column and its four neighbours, lava from the chamber's centre to
  `floor − 6`, then magma blocks: up to the surface on land, to `floor − 3` under water.
- **Crust-deep column** (Ben, decisions log 2026-10-02, "Crust-deep magma system"; Java `Magma.Deep`):
  from the chamber's bottom down to the lava floor. Lenses are flattened ellipsoids of lava, `n` of
  them at most `magma_lens_count` (24, 0 to 64; **0: no column**), the first 0.8 spacings under the
  chamber's bottom, then evenly (each up to 0.2 spacings off) down to a basal lens whose centre is 2
  blocks over `lava_floor` and at least 6 over `min_y` (read from the level, not a number); the
  spacing is `magma_lens_spacing_blocks` (28, 8 to 200), widened where the count would stop the column
  short. Radius at the top = `rh` · `magma_lens_width` (1.5, 0 to 4; **0: no column**) · a kind share
  (shield 1.4, silicic 0.9, otherwise 1.0) · (0.7 + 0.3 · activity), at least 5 and at most 90 blocks;
  the lowest lens is 2.2 times wider (linear in between, each drawn 0.9 to 1.05 of that); half-height
  0.2 · radius, 2 to 5 blocks; centres up to 0.15 of the radius off the axis. A volcano's chamber is
  1.6 times larger when silicic. Neighbouring lenses are joined by the conduit's plus-shaped column
  (lava), walled like the conduit. Lens lava obeys the chamber's rule (nothing within 8 blocks of its
  column's ground) and the seal. Around the column lies **altered hot rock** (the mush): radius
  `magma_mush_width` (1.0, 0 to 4; 0: none) · (1.25 · the column's radius at that depth + 3, bulging
  by 0.25 at a lens, 1.1 times broader under a shield), narrowing to nothing at the chamber's equator;
  it replaces the solid ground and shell with `minecraft:basalt` (`#deeptime:magma/mush`, fallback
  `#deeptime:magma_fallback/mush`), one block in 25 `minecraft:magma_block` (`/hot`), never fluid,
  bedrock or lava.
- **Crater lava lake**: of the volcanoes that can hold one, the `crater_lake_share` most active
  (ties by id; ⌈share · n⌉ of n). Radius r = clamp(half the crater's radius in blocks, 2, 10); its
  surface is one block under the lowest ground of the ring r < d ≤ r + 2; lava from there down to two
  blocks under the lower of that and `floor` (at most 10 deep), the ground above the surface removed,
  the conduit opening into its floor. A volcano cannot hold one when its summit is at or under sea
  level + 2, when any column of the lake or its ring holds or borders water (the sea, a lake, a river,
  a bank), or when the surface would lie more than 12 blocks under `floor` (the pole cap's line,
  past which no volcano held one on the Lambert world, went with that world, cube plan §7.4).
- **Seal**: every open block (air, water) within 1.5 blocks of a chamber, two columns of a conduit or
  two columns of a lake becomes smooth basalt, so lava never touches water or hangs over a cave.

`sea_cliff_skin_blocks` (6, 0 to 32; 0: off; Java `worldgen/SeaCliffGuard`, `SeaCliffPass`): under the
sea level, a block inside the ground is kept rock (no cave of any kind: it goes in the chunk's carving
masks and open blocks in it are filled, before the strata pass) where the ground within that many
columns to the side lies lower than the block and the ground drops by at least that many blocks within
that distance (a cliff of 45 degrees or more).

`remove_lava_features` (on by default): placed features whose configuration places lava (a block or
fluid state named `minecraft:lava`: vanilla's `lake_lava_surface`, `lake_lava_underground`,
`spring_lava`, `spring_lava_frozen`, Terralith's `volcano/lava_lakes`, `volcano/lava_flow`,
`volcano/spring_lava`, `cave/mantle/lava_drip`, any other mod's; also Terralith's
`cave/frostfire/lava_remover`, which turns lava sources into deepslate and would eat a chamber) are
not placed in a Deep Time level
(Java `worldgen/LavaFeatures`); other dimensions keep them.

### Geothermal features (P2-14) and their settings

Hot springs, sinter and travertine terraces, fumaroles with native sulfur and bare altered ground
sit around the active volcanoes and on hot continental ground. No file holds them: the game builds the sites from
`volcanoes/active.json`, the layer `meta/heat_flow_mw_m2`, the terrain height, the water columns and six more keys of
`notes.bake.underground.params` (defaults in brackets; Java `worldgen/Geothermal`, placed by
`worldgen/GeothermalPass` in the surface step after the magma and before the ice):
`geothermal_density` (1.0, 0 to 4; **0: no geothermal features**), `geothermal_reach` (1.0, 0.25 to 3),
`geothermal_size` (1.0, 0.5 to 3), `geothermal_spring_min_precip_mm` (400, 0 to 4000),
`geothermal_heat_flow_min_mw_m2` (100, 50 to 400) and `geothermal_heat_density` (1.0, 0 to 4; 0: no
fields from heat flow, the volcanoes' only).

**There are two heat sources: the active volcanoes, and continental ground whose heat flow is high**
(Ben, decisions log 2026-10-02, "Export heat flow"). The bake exports the plate model's heat flow as the
L1 layer `meta/heat_flow_mw_m2` (see the table above; nodata on oceanic crust and in a world without
the layer, which has no heat-flow fields). Spreading ridges are under the sea, so only the land
that the model heats (rifts, arcs, hotspot swells, young belts) gets fields.

**Fields from heat flow.** The world (C blocks round, 2R = C/π from pole to pole, z from −R: "Grids") is cut into a
grid of cells of 10,000 km² / `geothermal_heat_density` each (100 km across at density 1; the cell side
is rounded so a whole number of cells goes round the world). Each cell, from a seed made of its own
index, draws up to 6 random points and puts one field at the first where the bilinear
`meta/heat_flow_mw_m2` is at least `geothermal_heat_flow_min_mw_m2`, the ground is dry land at least 5
blocks above the sea with no water column and no ice cover. So fields follow the hot ground: at most one for each cell, about one for each 10,000 km² of a
broad hot province and a field in most of the cells a thin belt crosses; none is placed where the
layer is nodata or cooler. The field
is then built by the volcanoes' machinery below, as a field "away from the summit" (a share of 1: hot
springs where the rain and the temperature allow, else fumaroles), at a non-silicic source (no
cinnabar); its sites are numbered with the volcanoes' and carry the source id −1. Every other rule
below holds for them. The geothermal features are all off at `geothermal_density` 0.

With R the volcano's basal radius in blocks (`radius_m` / `m_per_block_h`, at least 6), a the activity
(0.05 to 1), a volcano gets ⌊d · (0.7 + 2.3 a) · clamp(√(R / 45), 0.6, 2.2) + u⌋ **fields** (d =
`geothermal_density`, u a uniform draw from the volcano's own seed, a hash of its id and position).
The first field lies 0.08 to 0.35 R · `geothermal_reach` from the summit, the others 0.25 to 1.0 R ·
`geothermal_reach`, in any direction, on the first of twelve draws whose centre is dry land at least 5
blocks above the sea (the 3 of `MIN_ABOVE_SEA` plus 2) with no water column and no ice cover. That
gives from none (a small, quiet volcano: expected 0.49 fields) to seven fields (a large, active one:
expected 6.6) at the default density; there is no "one to four". Every site's own columns must stand at
least 3 blocks above the sea. A field is **hot springs** when it is at least 0.35 R from the summit,
the baked `climate/p_mm` there reaches `geothermal_spring_min_precip_mm`, and `climate/t_mean_c` is at
least 3 °C (a world without those layers counts as warm and wet); else it is a **fumarole field**.

- A hot-spring field has a main pool (mean radius round((2.4 + 2.2 u) · `geothermal_size`), 2 to 9
  blocks), one to three smaller ones (1.6 to 3 times the size, 2 to 6), in a chain down the slope half
  the time (a terrace), and now and then a vent or a patch of altered ground. A pool's outline wobbles
  (`1 + 0.16 sin 3θ + 0.09 sin 5θ`). Its **level** is the mean ground top (⌈H⌉ − 1) of its pool and rim
  columns; it is refused where any column's ground is more than 6 blocks above the level (cut away) or
  more than 6 below it (built up as a dam: rim and bed fill down to the ground). Water fills the level
  and, in the middle 75 % of the radius, the block under it; the bed below is sinter (the site's terrace rock) in the heart (45 % of the radius), yellow
  mat, then orange mat towards the edge; the ground above the level is cut to air. The **rim** (every
  column that touches the pool, else out to 1.5 blocks) is solid at the level and one block under it. The
  **apron** around it, out to the site's `apron` radius, wears the terrace rock on its top block (a
  fifth the other rock), 7 % of it stained yellow, and half the time the block under it too.
- A fumarole field has two to four **vents** (radius 1 or 2: a core of magma blocks two blocks deep, a
  crust of native sulfur around it, bleached and yellow-stained ground out to 3 to 10 blocks) and one or
  two **patches** of altered ground (3 to 11 blocks). About 30 % of altered ground is stained yellow and
  the rest bleached; at silicic volcanoes 6 % is cinnabar (red stain) and 24 % yellow.
- A terrace is **sinter** (white, siliceous) unless carbonate rock (limestone, dolostone, chalk, marble
  or travertine) lies within 150 blocks under the site in the strata horizons: then **travertine**.
- Refused: a site on ground under 3 blocks above the sea; with a water column (the sea, a lake, a
  river or a bank: `Water.column`, as for the magma), ice cover or anything of another site's hard part
  (pool and rim, vent and crust) in its footprint; within 3 blocks + its hard radius of an open crater lake or the plug of a
  magma body. Two sites that claim a column: the higher priority wins (pool, rim, vent, sulfur,
  terrace, altered ground), so a pool's walls are never overwritten.
- **Blocks** are roles resolved like rock classes: the blocks of `#deeptime:geothermal/<role>`, else the first entry of
  `#deeptime:geothermal_fallback/<role>`, else the vanilla block: `sinter` (calcite), `altered`
  (white terracotta), `stain` (yellow terracotta), `mat` (orange terracotta), `sulfur` (yellow
  concrete), `cinnabar` (red terracotta), `vent` (magma block); travertine is the rock class of
  that name (`#deeptime:rock/travertine`, dripstone block in vanilla). Deep Time adds no block.
- **Water safety.** Bed, water and rim are in the chunk's carving masks. A vent's magma block never has
  water over it and no site touches lava. Columns with a fluid on top are left alone.

`DeepTimeClimate.geothermalWarmthC(x, y, z)` (API 3) is the warm-ground signal: kelvin over the
climate's temperature, at most 12 (vent) / 10 (pool) / 6 (patch) at a site's heart, falling linearly to
0 a few blocks beyond its altered ground (1.1 times its altered-ground radius plus 4 blocks), within 10
blocks under (12 for a pool) and 6 over its ground.

**What is not modelled.** There are no geysers: pool water is plain water, with no steam, eruption,
boiling point or temperature. The colours stand for the temperature zones of a real spring (a hot,
sterile heart, then yellow and orange microbial mats) but no temperature is coded. The science page
`docs/science/geothermal-features.md` lists the rules that are game choices.

### Volcanic biomes (decisions log 2026-10-03) and their setting

The pack's volcanic biomes go only at real volcanoes (Ben, "Tie them to real volcanoes (Recommended)").
No file holds them: the game builds the zones from `volcanoes/active.json`, the geothermal sites and the
magma bodies above and one key of `notes.bake.underground.params`, **`volcanic_biome_reach`** (1.0, 0 to
3; **0: no volcanic biome anywhere**), and decides the biome in the biome source (Java
`worldgen/VolcanicZones`, `climate/VolcanicBiomes`, the mixins `VolcanicBiomeSourceMixin` and, with
TerraBlender, `compat.terrablender.RegionBiomeSourceMixin`).

- **Kept out.** A biome in the tag `#deeptime:volcanic` is never the answer of a Deep Time level's climate
  search: the search runs on the biome source's own R-tree (the region's, with TerraBlender) with a
  distance metric that sets every such leaf infinitely far, so the next-nearest biome of the same tree
  answers. With TerraBlender, where that answer is an ocean or river biome and the kept-out one was not,
  the vanilla region's tree (region 0) answers instead, so a mod that put its volcanic land biome in a
  slot surrounded by sea (Biomes O' Plenty: mushroom fields) leaves land, not ocean. Other levels are not
  touched.
- **Roles**, in priority order, each a disc in blocks around its centre (R the basal radius
  `radius_m` / `m_per_block_h`, at least 6; r = `volcanic_biome_reach`); no disc is under 8 blocks in radius.
  On a projected world a volcano's crater, caldera, cone and field zones are instead the ellipse the
  projection makes of that circle on the planet at the vent (since 2026-10-23, SCL-230; the simulation's
  Lambert frame before), semi-axes from the projection's Jacobian there, each at least 8 blocks (ledger
  BIO-182): discs on the conformal cube, wide east-west on a Lambert-frame world; hot-spring fields
  and
  thermal caves stay discs:
  `hot_spring_field` (every hot-spring pool, from a volcano or hot ground: its reach + 4),
  `caldera` (a crater with `caldera: true`: 0.9 of its rim radius), `crater` (any other crater: its rim
  radius + 4), `cone` (stratovolcanoes and shields: 0.45 R · r), `ash_field` (stratovolcanoes and caldera
  complexes of activity 0.75 or more: R · r), `lava_field` (shields: R · r), `thermal_caves` (a magma body
  with a chamber: 1.5 chamber radii). An ash or lava field narrower than 8 blocks beyond the zone inside it
  is left out.
- **The biome of a role** is the first biome of `#deeptime:volcanic/<role>` (the tag's order) that the
  level's biome source can place; with none, the role gives way to the next role whose zone holds the
  column, and in the end to the climate search. Surface roles answer where the biome depth parameter is
  under 0.2 (vanilla's cave biomes start at 0.2), on land (terrain height at the cell's centre at or
  above the sea level) and not on a river channel (variant input exactly 0); `ash_field` and
  `lava_field` only where the climate point lies in one of that biome's own temperature and humidity
  ranges in the source's parameter list. `thermal_caves` answers underground, from that biome's own
  lowest depth parameter down.
- **Tags shipped** (all entries optional, so a pack without the mod loads them): `caldera`
  terralith:caldera; `crater` terralith:volcanic_crater, terralith:volcanic_peaks, biomesoplenty:volcano;
  `hot_spring_field` terralith:yellowstone, biomesoplenty:hot_springs; `cone` terralith:volcanic_peaks,
  biomesoplenty:volcano; `ash_field` terralith:ashen_savanna, biomesoplenty:volcanic_plains; `lava_field`
  terralith:basalt_cliffs, biomesoplenty:volcanic_plains; `thermal_caves` terralith:cave/thermal_caves;
  `#deeptime:volcanic` holds the seven role tags. A datapack adds a biome to a role tag (it is then kept
  out too) or to `#deeptime:volcanic` alone (kept out, never placed).
- Without any such biome in the pack (vanilla) every query is answered as before. `/deeptime probe
  biomes` reports the placement against the volcanoes (`census.volcanic`).
- **Tags added 2026-10-04** (Expanded Ecosphere, after the entries above): `cone` wythers:volcano,
  wythers:tropical_volcano, wythers:icy_volcano; `crater` wythers:volcanic_crater; `hot_spring_field`
  wythers:thermal_taiga, wythers:snowy_thermal_taiga, wythers:thermal_taiga_crags, wythers:mud_pools;
  `lava_field` wythers:savanna_basaltic_incursions; `thermal_caves` wythers:volcanic_chamber. Which of a
  role's biomes answers is chosen by the biome rules' role rules (next section) before the tag's order.

### Biome rules (decisions log 2026-10-04) and a datapack's overworld

Ben: "Build out support for both so if I install EE it works or if I go w/ terralith it also works". Two
game-side pieces, no world-data layer of their own:

- **A datapack's overworld** (`data/minecraft/dimension/overworld.json`, as Expanded Ecosphere's built-in
  datapack `ee_default` ships through Cristel Lib) is the first source vanilla's `WorldDimensions.bake`
  takes a dimension from, ahead of the world preset and level.dat. Java `worldgen/OverworldOverride` (from
  `mixin/WorldDimensionsShapeMixin`, before the planet shape) sets it aside when the world's own overworld
  is a `deeptime:geologic` generator, and gives every `deeptime:geologic` generator whose biome source is
  `{"type": "minecraft:multi_noise", "preset": "minecraft:overworld"}` that datapack overworld's biome
  source (its inline biome list) instead; a world whose overworld is not Deep Time's keeps the override.
  The adopted list is saved with the generator in level.dat.
- **Rule files**, `data/<ns>/deeptime/biome_rules/<name>.json` (Java `climate/BiomeRuleSet` the format,
  `climate/BiomeRules` the game side), read from the server's datapacks at a level's first biome query
  and again after a tag reload. A broken file is logged and skipped. Shape:

  ```json
  {"requires": ["wythers:ice_cap"],
   "rules": [{"id": "ice_cap", "ice": true, "keeps": ["#c:is_mountain/peak"],
              "biomes": ["wythers:ice_cap"], "exclusive": true}]}
  ```

  `requires` (ids or `#tags`; empty: always): the file is active only when the level's biome source can
  place one of them. `keep_out` (ids and `#tags`): biomes kept out of the climate search and placed by no rule
  (a pack's biomes whose slots Deep Time's coordinates reach in the wrong place). Keys starting with `_` are
  comments, any other unknown key refuses the file. `rules`, tried in order, files in the order of their ids. A rule's keys, all optional
  but `biomes`: `id`; `biomes` (ids and `#tags`: the candidates; those the source cannot place are
  dropped, and a rule left with none is dropped); `role` (a volcanic role id: the rule is tried only inside
  that role's zones, before the role tag's biome; cave roles take no rules); `koppen` (codes or prefixes:
  `"Cs"` is Csa/Csb/Csc, `"E"` is ET/EF; read from `climate/koppen_l2`, the class the biome inputs were placed
  by, where the bake wrote it, else from `climate/koppen`; nearest cell); `t_mean_c`, `t_warm_c`,
  `t_cold_c` and `p_mm` (`{"min", "max"}`, either end optional: the climate at the column's surface,
  lapsed, from the monthly layers; a world without a climate never matches); `elevation_m` (surface above
  sea level); `wetland` (`any`, `warm`, `cold`, `none`: `climate/wetland` 1 or 2 where the world has the
  layer, else wet land in the swamp erosion band E6, cold in Dfc, Dfd, Dsc, Dsd, Dwc and Dwd); `ice` (the
  ice cover lays ice); `playa` (`playa` above 0, nearest cell); `volcanic` (any volcanic zone holds the
  column); `rock` (class names: the first horizon of the column's strata within 6 blocks of the surface); `ground`
  (geology roles, "The geology gate" below: the column's ground has one of them); `in_slot` (true: the source's
  own climate search with nothing kept out gives one of the rule's candidates, so the rule keeps the pack's own
  slot; with `exclusive` a gate: the candidates appear in their slots only where the other conditions hold);
  `replaces` and `keeps` (ids and `#tags`: the rule holds only where the biome the climate search would
  give is, respectively is not, among them); `exclusive` (true: the candidates are kept out of the climate
  search, like `#deeptime:volcanic`, and appear only where a rule places them). Ordinary rules apply at
  the surface (biome depth under 0.2), on land, off river channels. `replaces` and `keeps` are tested before
  the climate conditions (their search is the level's own answer when no rule holds, so it is made anyway),
  so a rule that replaces a few biomes samples the climate only where the search gives one of them. The first rule whose conditions hold
  and whose search finds a candidate decides: the source's own climate search runs over the rule's
  candidates alone (every other leaf infinitely far, as for the volcanic keep-out), so the pack's
  parameter ranges choose among them, out-of-range biomes included.
- **Shipped files**: `deeptime:peatland` (requires `#deeptime:peatland`; cold wetland takes a member of the
  tag `#deeptime:peatland`: wythers:fen, wythers:berry_bog, wythers:snowy_bog, wythers:snowy_fen,
  biomesoplenty:bog, biomesoplenty:muskeg, regions_unexplored:fen, clifftree:bog, all optional) and
  `deeptime:expanded_ecosphere` (requires wythers:ice_cap or wythers:fen; 19 rules, docs/science/biomes.md,
  "Expanded Ecosphere and the biome rules"), `deeptime:rock_named` (requires `#deeptime:geology`; the geology
  gate below) and `deeptime:snow_line` (always active; two rules: where the climate search gives a biome of
  `#deeptime:snow_line/snowfield` (frozen and jagged peaks, snowy slopes) on a column whose warmest month at its
  own height is 10 °C or more, or `minecraft:grove` where the coldest month is 0 °C or more, and the column is
  not under ice, a member of `#deeptime:snow_line/snowless` (vanilla's stony peaks, meadow and windswept hills,
  Terralith's rocky mountains and highlands, optional) answers; docs/science/biomes.md, "The snow line"). A
  datapack adds a file, or replaces one by its id.
- `/deeptime probe biomes` reports what decided each sampled land column (`census.rules`: per rule, per
  role and the climate search; candidates seen and absent; the checks the GameTests read).

### The geology gate (decisions log 2026-10-04)

A pack's rock-named biomes stand only on the ground they are named for: the biome rule file
`deeptime:rock_named` ("Biome rules" above) with the rule condition `ground`, whose roles Java
`worldgen/GeologyGround` computes per column. No world-data layer holds them: the game reads `meta/province` (with `notes.bake.meta.provinces.province[i].class`), `surface/class_top`, the
column's `strata/horizons` stack, `climate/koppen` and the terrain height. A column's **bedrock** is its
surface rock class, or, where that is loose cover (the rock-class group `deeptime:rock/unconsolidated`:
saprolite, laterite, till, sand …), the first other class of its stack within 8 blocks of the surface.
Its **roles** (`#deeptime:geology/<role>`, all entries optional):

| role | ground | tag shipped |
|---|---|---|
| `shield` | province class Shield | terralith:shield, snowy_shield, shield_clearing |
| `granite` | bedrock in `deeptime:rock/intrusive` and `deeptime:rock/felsic` | terralith:yosemite_cliffs, yosemite_lowlands |
| `carbonate_coast` | a coast (the sea within 32 blocks: the column, or 16 or 32 blocks away in eight directions, below the sea level) whose bedrock (the column's, else the first within 48 blocks) is in `deeptime:rock/carbonate` | terralith:white_cliffs |
| `granite_coast` | as above, intrusive and felsic | terralith:granite_cliffs |
| `basalt_coast` | as above, `deeptime:rock/volcanic` and `deeptime:rock/mafic` | terralith:basalt_cliffs |
| `dry_sedimentary` | bedrock in `deeptime:rock/sedimentary`, Köppen B | terralith:bryce_canyon, painted_mountains, white_mesa, sandstone_valley, desert_canyon |
| `humid_karst` | bedrock in `deeptime:rock/carbonate`, Köppen A, C or D | terralith:stony_spires |

`#deeptime:geology` holds the seven role tags. The file has one rule per role:
`{"id": "<role>", "biomes": ["#deeptime:geology/<role>"], "ground": "<role>", "in_slot": true, "exclusive": true}`.
So a gated biome is kept out of the climate search, and on land at the surface, off river channels, a column
whose own slot (the pack's search, nothing kept out) is one of them gets it back where its ground has the role;
elsewhere the pack's next-nearest biome answers (on land never an ocean or river: the search is repeated with
water leaves set aside where the kept-out answer would turn land into water). On its ground the biome answers
where the climate search puts it: the gate takes biomes away and never adds them. A gated biome that is volcanic
too (terralith:basalt_cliffs) goes at real volcanoes by its role and on its own ground by the rule. The file sorts
after `deeptime:peatland`, so a rule that places another biome is tried first. Without such a biome in the
pack the file is inactive and every query is answered as before. `/deeptime probe biomes` reports the
rock-named biomes against their ground (`census.geology`).

## `strata/classes.json`

`{"classes": ["sandstone", "red_bed", "granite"]}` — index → rock class id (plan §5.4.1,
`deeptime:rock/<class>` tags). The vocabulary is the rock-tag README's 60 classes (`diamictite`,
lithified till, added 2026-09-30) plus `impact_breccia` and `impact_melt` (the rocks of meteorite
impacts, added for P2-08 on 2026-10-01), 62 in all; a reader maps indices through this file, never through a fixed
list, so a class added to the vocabulary changes no bytes a reader depends on.

## `spawn/candidates.json`

```json
{"rule": "…", "candidates": [{"x": 12, "z": -40, "surface_y": 71.5, "cell": [262, 108]}]}
```

Block coordinates of L3 sample points that are dry land, gently sloping, spread apart, ordered by
distance from (0, 0).

Every world's candidates follow this one rule (`synth::spawn_candidates`: an 8-block lattice, dry,
at least sea level + 2, within one block of the L3 neighbours, 64 blocks apart, at most 8). They are
not ranked by the essentials (Ben, decisions log 2026-10-05: spawn follows the normal rules); the
`ranked_by`, `unmet`, `score` and `essentials` keys of P1-15 no longer exist. The game takes the first
candidate that is dry land once generated (`SpawnHandler`).

## Sectioned files (`DTEV`, `DTSR`, `DTSI`, `DTKF`, `DTCP`)

Every binary file other than a tile pack shares one container (`container.rs`): the 16-byte header
of plan §5.7.2, fixed fields of the file type, a section table and the sections.

```text
offset   size  field
0        4     magic
4        4     u32 header_len = 16 + F + 16·K (F fixed bytes, K sections)
8        2     u16 format_minor (2 for "0.2")
10       2     u16 flags, reserved, 0
12       4     u32 CRC32 of bytes 0..12 followed by bytes 16..header_len
16       F     fixed fields (per type, below)
16 + F   16·K  section table: u64 offset (absolute), u32 length, u32 CRC32 of the section's bytes
…              sections, each starting on a multiple of 8, zero padding between and after
```

A reader checks the magic, then the minor (refusing another one with the version message), then
`header_len` against its type's F and K, the header CRC, and each section's bounds and CRC. String
tables are `u32 byte length + UTF-8 bytes`, back to back, with the count in the fixed fields.

| Magic | File | F | K | Spec |
|---|---|---|---|---|
| `DTEV` | `events/<w>-<n>.dtev` | 32 | 5 | `EVENTS.md` |
| `DTSR` | `sites/sites.dtsr` | 16 | 3 | below |
| `DTSI` | `sites/index.dtsi` | 32 | 2 | below |
| `DTBD` | `bodies/bodies.dtb` | 16 | 3 | below |
| `DTRV` | `rivers/rivers.dtrv` | 48 | 4 | "Rivers" |
| `DTKF` | `keyframes/kf-<n>.dtk` | 24 | 1 + layers | below |
| `DTCP` | `checkpoints/*.dtcp` | 8 | 2 | below (dev only) |

## `preset.json`

The run's preset as canonical JSON (keys sorted), pretty-printed. The dummy
preset's keys are documented in `crates/deeptime-core/src/dummy.rs`; the real preset schema
arrives with P1-03 (`crates/deeptime-format/schema/preset.schema.json`, plan §5.1).

## Sites (`sites/sites.dtsr`, `sites/index.dtsi`)

Plan §5.6. `DTSR` fixed fields: `u32 N sites, u32 S strings, u32 P products, u32 reserved`.
Sections: 0 strings (deposit types, materials, roles, COE vein ids), 1 N records of 104 bytes
sorted by id, 2 P products of 24 bytes.

```text
site record                               product entry
0    4  u32 id (u32::MAX reserved)        0   4  u32 material (string index)
4    1  u8 kind: 0 deposit, 1 reservoir,  4   4  f32 grade (mass fraction for ores)
        2 placer, 3 supergene, 4 fantasy  8   8  f64 tonnage_t
5    1  u8 flags: bit 0 finite            16  4  u32 role (string index: primary, byproduct, …)
6    2  reserved, 0                       20  4  reserved, 0
8    4  u32 deposit type (string index)
12   4  u32 body id (u32::MAX = none; the site's first body in bodies/)
16   12 i32 × 3 centre (block x, y, z; x in the canonical span)
28   8  f32 × 2 latitude, longitude (degrees)
36   24 i32 × 6 AABB min x, y, z, max x, y, z (inclusive; x may leave the canonical span)
60   4  u32 parent site (u32::MAX = none)
64   8  u64 forming event id (EVENTS.md)
72   4  u32 first product
76   4  u32 product count
80   8  f64 amount_base
88   4  u32 COE vein (string index, u32::MAX = none)
92   4  reserved, 0
96   8  u64 CDG charge in mB (u64::MAX = none)
```

`DTSI` fixed fields: `u32 C, u32 cell_blocks (512), u32 cols = ceil(C/512), u32 rows =
ceil(2·Zo/512), i32 origin_x (−C/2), i32 origin_z (−Zo, "Grids"), u32 entries, u32 reserved`. Section 0 is
`cols·rows + 1` u32 offsets, section 1 `entries` u32 site ids: cell `c = row·cols + col` holds
`ids[offsets[c] .. offsets[c+1]]`, ascending — every site whose AABB footprint (x, z) touches the
cell, with columns wrapping in x. Block (x, z) is in column `floor((x − origin_x)/512) mod cols`
and row `clamp(floor((z − origin_z)/512), 0, rows − 1)`.

The viewer's `sites.json` holds one object per site: `id, kind, deposit_type, center, lat_lon, aabb,
body, products [{material, grade, tonnage_t, role}], forming_event, parent, extraction {finite,
amount_base, coe_vein, cdg_charge_mb}`.

Deposit sites (P1-14):

- `material` is `c:<name>` (`c:copper`, `c:crude_oil`), resolved through `#c:ores/<name>` (plan
  §7.6). `tonnage_t` is the **ore** tonnage the grade applies to (the same for every product of a
  site; contained metal = `tonnage_t · grade`), after truncation by erosion and leaching into a
  supergene child. `grade` is a mass fraction (`_pct` / 100, `_gpt` · 10⁻⁶); a product that is the
  whole ore (coal, crude oil) has grade 1. A site with no ore left (a porphyry whose preserved roots
  were all leached into its blanket) has no products.
- `body` is the site's first body; its bodies are the records of `bodies/bodies.dtb` whose `site`
  is the site's id, consecutive ids from `body`: the ore body first, then a lithocap or a gossan.
  A **stacked** deposit (coal seam fields: the type's `geometry.stack`) has **several ore bodies,
  one per seam**, all with role `ore`, in consecutive ids from the top seam down: the same centre
  x and z, footprint (a, c), yaw and dip, each with its own vertical centre and thickness, apart by
  the gap between seams (no two touch), each carrying the type's zones. The site's `tonnage_t` is
  the whole field's: a seam holds its share of it, in proportion to its thickness. Erosion removes
  seams from the top (a seam above the ground is dropped, one crossing it is clipped at the surface
  by its own `clip_top_y`), so a truncated stack holds fewer seams and `tonnage_t` counts what is
  left. Readers treat the `ore` bodies of a site as disjoint and add them (Java's
  `SiteChunks.footprint` sums them; the deposit feature and the strata pass already loop over every
  body). Nothing in the file layout changed, so the format version stays: a reader that handled a
  site's several bodies before handles a stack.
  A **placer** of a planet run (a world with a river network, `rivers/rivers.dtrv`) also has
  **several ore bodies**: after the bake it is laid where the finished world has its gravel
  (`deeptime_core::deposits::placer_channels`, `notes.bake.placers`). With a river channel within
  its type's `placer.river_reach_km` it becomes gravel bars along the channel: one `channel` body
  per straight piece of the centre line (at most 24 blocks; downstream through confluences), its
  local x along the piece and 1.2 times its length, so neighbouring bars overlap a little, its
  half-width the channel's plus 2 blocks, and its blocks from the water's top block down to three
  blocks (or the ribbon's thickness, if more) under the bed at the channel's centre. Without one it
  becomes a bench: the ribbon in pieces of at most 24 blocks along its direction, each with its top
  block at the terrain's top solid block (the bed under water) at the piece's centre. Either way
  the bodies have consecutive ids from `body`, the type's rock class (gravel) and zones (the pay
  streak in the lower half), and the site's `center` is the channel vertex it settled at (y the
  bed) or its own place (y the ground). A placer's bars overlap, so a block may lie in two of the
  site's bodies: the deposit feature draws once per body, lowest id first, and a block already
  turned to ore hosts nothing more; `SiteChunks.footprint` counts the overlap twice (a share of a
  chunk, never the ore). A synthetic world's placer (no river network) stays one ribbon.
- `amount_base` = `tonnage_t`; `coe_vein` is the deposit type's `minecraft.coe_vein`
  (`createoreexcavation:ore_vein_type/copper`, …); `cdg_charge_mb` (reservoirs) is the oil tonnage
  / 0.85 t/m³ · 1000 mB/m³. The records keep these physical values; the game scales them at run time
  (P1-16, plan §7.14): COE's vein size follows the site's tonnage quantile in its type's lognormal,
  and CDG sees `cdg_charge_mb` × the type's `minecraft.cdg_scale`, split over the chunks of the ore
  body by volume.
- `parent`: a supergene blanket's porphyry. A placer's parent was eroded away, so it has none; its
  forming event (`placer_formation`) is caused by the parent's `deposit_exhumation`.

## Bodies (`bodies/bodies.dtb`)

Plan §5.7.4; code `crates/deeptime-format/src/bodies.rs`. `DTBD` fixed fields: `u32 N bodies,
u32 S strings, u32 Z zones, u32 reserved`. Sections: 0 strings (roles, materials, rock classes,
zone names), 1 N body records of 112 bytes sorted by id, 2 Z zone records of 40 bytes.

```text
body record                                      zone record
0    4  u32 id (u32::MAX reserved)               0   4  u32 name (string index)
4    4  u32 site (u32::MAX = none)               4   4  u32 material (string, u32::MAX = none)
8    1  u8 shape (below)                         8   4  u32 rock class (string, u32::MAX = none)
9    1  u8 flags: bit 0 clipped by erosion       12  8  f32 × 2 d range [d0, d1)
10   2  reserved, 0                              20  8  f32 × 2 v range [v0, v1)
12   4  u32 role (string: ore, lithocap, …)      28  4  f32 grade (mass fraction of material)
16   24 f64 × 3 centre (block x, y, z)           32  4  f32 density (P(host block → ore), 0–1)
40   12 f32 × 3 semi-axes a, b, c (blocks)       36  4  f32 raw (share of its ore → raw block)
52   4  f32 yaw (radians about +y, +x → +z)
56   4  f32 dip (radians about the local x axis)
60   4  i32 clip_top_y (i32::MAX = none)
64   24 i32 × 6 AABB min x, y, z, max x, y, z (inclusive; x may leave the canonical span)
88   4  u32 first zone
92   4  u32 zone count
96   4  u32 rock class of the whole body (string, u32::MAX = none)
100  4  f32 age: when the body formed, Myr before the end of the world's history (NaN = none)
104  8  reserved, 0 (voxel blobs, later)
```

The age (2026-10-19, decisions log "Add the fields"; within 0.5, no minor of its own, since nothing
is released): plutons, their aureoles, stocks' aureoles and stock roots carry their emplacement
(the aureole takes its intrusion's), every other body NaN for now. A file written before it holds 0
there.

Shapes: 0 ellipsoid, 1 stockwork_shell, 2 lens, 3 pod, 10 dome use the **ellipsoid metric**;
4 sheet, 5 dyke, 6 pipe, 7 blanket, 8 seam, 9 channel use the **cylinder metric**. Evaluation of
block (x, y, z) on a planet of circumference C — Java must do exactly this:

1. The sample point is the block centre: `dx = x + 0.5 − cx` wrapped into [−C/2, C/2)
   (`dx −= floor(dx/C + 0.5)·C`), `dy = y + 0.5 − cy`, `dz = z + 0.5 − cz`. Outside if
   `y > clip_top_y`.
2. Local coordinates, with `(sy, cy) = sin/cos yaw`, `(sd, cd) = sin/cos dip`:
   `lx = cy·dx + sy·dz`, `ly = sd·sy·dx + cd·dy − sd·cy·dz`, `lz = −cd·sy·dx + sd·dy + cd·cy·dz`.
3. `d = √((lx/a)² + (ly/b)² + (lz/c)²)` (ellipsoid) or `d = max(√((lx/a)² + (lz/c)²), |ly/b|)`
   (cylinder); outside if `d > 1`. `v = clamp((b − ly)/(2b), 0, 1)` (0 at the local top).
4. A zone applies when `d0 ≤ d < d1` and `v0 ≤ v < v1`, an upper bound ≥ 1 being inclusive. Zones
   may overlap (several products in one place); a zone with no material only changes the rock
   (`rock_class`), and a zone density of 0 places no ore (fluids, alteration halos).

In the game (P1-15; `neoforge/…/worlddata/Body.java` is this evaluation, checked against Rust by
`DepositDataParityTest` over `deeptime dump-deposits`): the strata pass paints a body's
`rock_class` (a zone's, where one applies, winning; of several zones the last) over the crust or
horizon rock inside it, and the deposit feature turns a host block into ore where one uniform draw
per block, a hash of the canonical block position and the body id, falls below the running sum of
the densities of the zones with a material that apply there, in zone order (so overlapping zones
share the block's probability, capped at 1). An ore block of a zone with `raw` > 0 is the
material's raw-ore block instead (`#c:storage_blocks/raw_<material>`, else vanilla's
`raw_iron_block`, `raw_copper_block` or `raw_gold_block`; none: it stays ore) where a second draw,
the same hash with the body id XOR `0x52415721`, falls below `raw`: vanilla's ore veins carry raw
blocks among their ore the same way (Ben, 2026-10-02).

**Plutons** (P1-07, `structure::plutons`): ellipsoid bodies with no site, role `pluton`, no zones,
the body's `rock_class` its intrusion (granodiorite, granite or diorite).

**Contact aureoles** (REV-2, 2026-10-19): ellipsoid bodies with no site, role `aureole`, no
`rock_class`, one per pluton and per stock whose host can be baked. The body is its intrusion grown
by the aureole's width on every axis (`structure.aureole_share` of the intrusion's smallest
dimension, `structure.aureole_carbonate_factor` times that in a carbonate host). Its zones (named
`aureole`, no material, density 0) hold the baked rock band by band down the column: each zone's
`rock_class` is skarn or hornfels and its v range the depths of a host horizon older than the
intrusion, read on the column at the intrusion's centre as the strata pass paints it (the column rule over `strata/horizons`) and by the structural
events' test (a horizon whose top key is at least the intrusion's); a pluton's zones span d ∈ [0, 1], and the pluton body, whose id is lower, keeps its own
rock inside. A stock's aureole is hollow: the stock is a structural event, which bodies paint
over, so its zones are cut into eight horizontal slabs whose d range starts at the stock's wall
in that slab. The centre's column also holds the other intrusion events that reach the inner half
of the intrusion's plan (any of the centre and the points half a semi-axis out along and across:
an older stock it was emplaced in or beside, a younger one through it), applied there as the
column rule applies them, as thick as at the point they hold deepest: an older one's rock is
crystalline and a younger one's is younger than the aureole, so no zone paints over either (the
aureole goes unbaked at those depths all round). **Stock roots**: a stock rooted on a pluton (`structure::deform::intrude::
root_on_plutons`) also gets an ellipsoid body of role `stock`, its own ellipsoid, with one zone
`root` (its rock class, no material) from the top of the pluton's aureole down, so the stock's
rock reads through the aureole it rises across. Ids (since 2026-10-24): the plutons first (the
later-emplaced first), then the stock roots, then the aureoles (of the stocks, then of the
plutons, each list backwards), then any deposit body the deposit stage adds later; where bodies
overlap, the one with the lower id keeps the block ("Columns", step 5), so each
intrusion keeps its own rock through its aureole and of two overlapping plutons the younger keeps
the overlap. Until then the order was the reverse and was read as "a higher id paints over a lower
one", which no evaluator did. `notes.structure.aureoles`
counts them: `plutons`, `stocks`, `zones`, `median_width_m` [pluton, stock] and `stock_roots`;
`notes.structure.deformation.stocks_rooted` the stocks rooted on a pluton and `stocks_under_cover`
the stocks whose top was taken down to the base of younger cover.

The AABB is conservative: the world extent along axis i is `Σ_j |R_ji|·h_j` around the centre
(R the rotation above, h the semi-axes), clipped at `clip_top_y`. The viewer's `report.js` carries
each body as JSON: `id, site, shape, metric, role, center, axes, yaw, dip, clip_top_y (null = none),
aabb, rock_class, age_ma (null = none), zones [{name, material, rock_class, d, v, grade, density, raw}]`.

`manifest.json` `notes.deposits` (runs with the deposit stage) is the stage's census: per type
`id, sites, target (the survivors the density asks for: count_mult · ρ · Σ permissive area, plan
§8.6), preservation (the formation-weighted share the planet preserves), formation_mult, capped,
expected_formed, expected (the survivors expected; = target unless capped), drawn (deposits
formed), tracts, permissive_km2, mean_f, buried, truncated, eroded, too_deep, children, guaranteed,
dropped_separation`, plus `province`, `provinces`, `count_mult`, `max_formed_per_survivor`,
`brine_flow_km`, `sites`, `bodies`, `richness_mult`, `scatter_mult` (the game multiplies every background-scatter rule by it), `inherited_bif_share`, `bif_basin_km2`, and `bif_basins`, `bif_basin_share` (the inherited iron-formation basins the surface stage dealt). A forming event's `p_survive` is its cell's survival chance.
`world_frame` (DEP-220) is the world the playability pass counted the essentials' regions in:
`{"projection": "lambert"}` (with `"why": "no essentials"` when the preset has none, and on a
Lambert-frame preset), or `{"projection": "cube", ...}` with its own `turn`: the stage's forecast of
the bake's turn on its present L1 land (SCL-129; "The bake on a cube world"). The projection chooses
its own turn on the baked land; the anchor's top-up fills the world's regions the forecast missed.
(The Petroff–Guyou forecast of λc went with that world, cube plan §7.4.)

## Structural events (`structure/events.dtst`, P2-01, format 0.3)

The folds and faults of a baked planet, and the rule that reads a column's horizon stack through
them. Code: `crates/deeptime-format/src/structure.rs` (`Structure`, `StructIndex`,
`Structure::deformed_stack`); Java `worlddata/Structure.java`. `DTST` fixed fields (16 bytes):
`u32 N events, u32 P profiles, f64 margin (blocks)`. Sections: 0 the N event records of 136 bytes,
**youngest first** (keys never decrease); 1 the P fold profiles, 629 × f32 each.

```text
event record
0    4   u32 id
4    1   u8 kind: 0 fold, 1 fault, 2 intrusion
5    1   u8 profile (folds: index into section 1; faults: 0; intrusions: their rock class index)
6    2   u16 key: the event's age as a stratigraphic key of `strata/horizons`
8    8   f64 t, years relative to world creation (for people; the rule reads the key)
16   16  f64 × 2 window centre, block (x, z)
32   16  f64 × 2 along: unit vector along the structure (a fold's axis, a fault's strike), (x, z)
48   16  f64 × 2 across: unit vector across it (a fault: towards its hanging wall), (x, z)
64   8   f64 half_along, blocks
72   8   f64 half_across, blocks
80   56  f64 × 7: a fold: wavelength, amplitude, offset (blocks), 0, 0, 0, 0
                  a fault: y of the plane at the centre, pole (x, y, z; y > 0), slip (x, y, z)
                  an intrusion: y of its centre, half_height (blocks, > 0), 0, 0, 0, 0, 0
```

All arithmetic below is IEEE-754 double `+ − × /`, `√` (correctly rounded), comparisons, `floor`
and `ceil`, in the order written; both sides must give equal answers (`ColumnParityTest`). TAU is the constant 6.2831853
(Noddy's seven-decimal 2π, part of the contract).

**Window.** `bump(t) = (1 − t²)²` for −1 < t < 1, else 0. For a block position (x, z):
`dx = x − centre x; dx −= floor(dx/C + 0.5)·C; dz = z − centre z`; `a = dx·along x + dz·along z`,
`b = dx·across x + dz·across z`; the weight is `wa = bump(a / half_along)`, 0 if that is 0, else
`wa · bump(b / half_across)`.

**Profile.** `profile(table, p)`, p ≥ 0: `d = (p / TAU) · 628`, `f = floor(d)`, `i1 = f mod 628`,
`i2 = ceil(d) mod 628`, both clamped to [0, 627]; `v1 = table[i1] / 100`, `v2 = table[i2] / 100`;
the value is `v1 + (d − f)·(v2 − v1)`. For p < 0 (the bake keeps `offset ≤ −half_across`, so it
does not occur inside a window): the same with `d` from −p and `i = 628 − (… mod 628)`. This is
Noddy's `calcProfile` (Loop3D/noddyverse, MIT); the one profile a bake writes is its sine table,
`table[i] = (f32)(100·sin(2π·i/628))`.

**Candidates.** Event e is a candidate for the column at block (x, z) when |dx| and |dz| (dx
wrapped as above) are both at most `√(half_along² + half_across²) + margin`. The candidates are
undone in file order (youngest first). The margin is at least the furthest any chain of faults
moves rock sideways in this world (the bake measures it), so every piece below meets every event
that could touch it; it is part of the rule, not only a shortcut.

**The lift** (added 2026-10-20). The bake raises every horizon of a strata cell by the cell's
whole-stack lift after the history (the terrain's relief over the history's surface, ledger
STR-052) and writes it as `strata/lift`; the events keep the history's heights. L(x′, z′), the
lift of block column (x′, z′), is that layer's physical values through the column rule's
B-spline: x′ folded as the column rule folds it, `u = (x′ − origin x) / r`, `v = (z′ − origin z) / r`,
`i0 = ⌊u⌋`, `j0 = ⌊v⌋`, the weights `bspline(u − i0)` and `bspline(v − j0)` ("`horizon_stack`
cells"), the 16 cells (i0 − 1 + a, j0 − 1 + b) row by row (b outer, a inner), and
`L = V(i0, j0) + Σ (w_a · w_b) · (V(i0 − 1 + a, j0 − 1 + b) − V(i0, j0))`, so that a uniform lift is
exact (Rust `RasterRead::column_spline`, Java `RasterSampler.columnSpline`). Far terrain's cell
stacks (each the nearest strata cell's stored stack) take the nearest cell's value. A world
without the layer has L = 0. A height y of the history at column (x′, z′) is y + L(x′, z′) in the
column's stacks, so the rule below reads each source stack through its own lift and every column
rises by its own: a fault's plane or an intrusion's body moves with the rock at each column it
cuts, not with the rock at the event's centre.

**The deformed column rule** for block column (x, z), with S(x′, z′) the 0.2 column rule's stack
at a block column (integer tops T_h, classes, unit keys, top-down) and L(x′, z′) its lift:

1. S₀ = S(x, z). No candidates, or S₀ empty: the answer is S₀.
2. One *piece* covers every height: `lo = −∞, hi = +∞`, source position (x, z) as doubles,
   `dy = 0 − L(x, z)`, `key = 0`, not finished, source stack S₀ and its lift L_s = L(x, z). (A
   piece's dy is the history's height at its source less the column's height, before the
   source's lift.) For each candidate e in order, for each
   piece in order (bottom-up; pieces e itself creates are not revisited by e) that is not finished
   and at whose source position e's weight w is not 0:
   - **Cover.** h = the first horizon of the piece's source stack with unit key ≥ e's key. None:
     the piece is finished. If h > 0: `cut = (T_h + 1) − L_s − dy`; if `cut ≤ lo` the piece is finished;
     else if `cut < hi` the piece is split at `cut` and the upper part, [cut, hi), is finished.
     (Rock laid down after e is not moved by e, nor by anything older.)
   - **Fold:** `dy ← dy − w·(amplitude · profile(table, (b − offset)·TAU / wavelength))`,
     `key ← e's key`.
   - **Fault:** with `dx′ = a·along x + b·across x`, `dz′ = a·along z + b·across z`:
     `plane = y − (dx′·pole x + dz′·pole z) / pole y`, `cut = plane − dy`. If `cut < hi`, the part
     [max(cut, lo), hi) (the whole piece when `cut ≤ lo`, else split off above the rest) becomes
     the hanging wall: source position `(sx − w·slip x, sz − w·slip z)`, `dy ← dy − w·slip y`,
     `key ← e's key`, not finished, source stack S(⌊sx⌋, ⌊sz⌋) of the new position and its lift
     L_s = L(⌊sx⌋, ⌊sz⌋).
   - **Intrusion:** `r² = (a / half_along)² + (b / half_across)²`. If `r² < 1`:
     `hh = half_height · √(1 − r²)`, `l = max((y − hh) − dy, lo)`, `u = min((y + hh) − dy, hi)`;
     if `l < u` the part [l, u) of the piece is split off as the intrusion's rock (its class, unit
     key = e's key) and is finished; what is left of the piece below and above goes on unchanged.
     (An intrusion moves nothing. It is read at the piece's source position, so the faults and
     folds younger than it have moved it with the rock, and it cuts the ones older than it.)
3. The pieces top-down. A piece holds blocks `y_min … y_max`: `y_max = min(T₀ of S₀, ⌈hi − ½⌉ − 1)`
   (T₀ of S₀ for hi = +∞), `y_min = max(min_y, ⌈lo − ½⌉)` (min_y for lo = −∞); skipped when
   `y_max < y_min`. With `d = ⌊dy + L_s + ½⌋` (`(dy + L_s) + ½`) and the piece's source stack (an intrusion's piece is instead the one horizon
   `(y_max, its class, its key)`, appended unless the last appended has the same class): its horizon h covers
   `top = y_max` for h = 0, else `min(y_max, T_h − d)`, down to `bottom = max(y_min, T_{h+1} + 1 − d)`
   (y_min for the last); skipped when `top < bottom`. Its class and unit key are those of horizon
   g: g = h, and if g's unit key is below the piece's key (the shift reached up into rock younger
   than the event that moved the piece), g is advanced while the *next* horizon's key is also
   below the piece's key and there is a next one: the lowest bed of that cover stands in, as the
   bed laid on the deformed surface fills its lows.
   `(top, class, unit)` is appended unless the last appended has the same
   class.

So a column no event touches reads as in 0.2; under a fold the beds older than it move up and down
beneath a cover whose own beds are flat and whose lowest bed fills the troughs (an angular
unconformity); above a fault's plane the column is the hanging
wall's, read from where it came from, so a normal fault drops beds and a thrust repeats them.
Bodies are painted over the result where they are, undeformed.

**What a bake writes** (`deeptime_core::structure::deform`): fold trains (kind 0: an upright sine
fold under a window a few wavelengths across and several long) and fault segments (kind 1: thrusts,
normal and strike-slip faults whose slip dies out along the trace and away from it) and intrusions
(kind 2: stocks in orogens, dykes along rifts, plugs under extinct volcanoes), each dated to
the plate step that made it; the key is `structure::columns::key::of_age(0, age, span)`, the
unmetamorphosed strata log's key of that age, so basement and metamorphic units (larger keys) are
deformed by every event; every height is the history's (the readers add the lift, "The lift"); a
fault's `y` is the history's ground at its centre; an intrusion's `y` is its
ellipsoid's centre: for dykes and plugs the world's floor (they are rooted there and narrow
upwards to their top), for a stock the middle of its own thickness (Cruden & McCaffrey's law
on its outline) or of its reach down into the roof of the pluton it rises from. `notes.bake.structure` records
the counts and the margin, `notes.structure.deformation` what the run made.

## Keyframes (`keyframes/index.json`, `keyframes/kf-<n>.dtk`)

Timelapse frames (plan §6.1), written while a run goes; frame `n` counts from 0.

```json
{"format_version": "0.2",
 "frames": [{"n": 0, "t": -390000000.0, "stage": "dummy_drift", "w": 256, "h": 128,
             "layers": ["elevation"], "path": "keyframes/kf-0.dtk", "bytes": 5321, "crc32": 1}]}
```

`DTKF` fixed fields: `u32 n, u32 w, u32 h, u32 L layers, f64 t (years)`. Section 0 is a string
table of `1 + L` strings: the stage, then the layer names in section order. Sections `1 … L` are
one LZ4 raw block each (as tile packs), decoding to `w·h·4` bytes of RGBA8, row-major, row 0 north.
Pixel (px, py) is centred on longitude 360°·px/w (the seam is 0°) and on sin φ = 1 − 2·py/h, i.e.
block z = −R + 2R·py/h (a Lambert equal-area map, "Grids": rows uniform in sin φ from the north pole
to the south pole; aspect π : 1, 1024 × 326 by default), the nearest cell of the field it was drawn
from, coloured with `crates/deeptime-viewer/assets/ramps.json`
(the same ramps the viewer and the game use); alpha 0 is nodata. Frames are taken when the clock
passes the next multiple of the preset's keyframe interval (counted from the run's start) and at
the end of each stage.

Three layers are derived when a frame is drawn rather than read from one field, and are never stored
in the world: `provinces` (geologic provinces, coloured with its ramp like any layer), `rivers`
(rivers and lakes, below) and
`satellite`, a true-colour picture composited from the elevation, the climate's fields (land cover
from `climate/t_mean_c`, `t_warm_c`, `t_cold_c`, `aridity`, `precip_mm` and `koppen`; snow from the
temperatures and `climate/ela_m`; `climate/ice_thickness_m` or `glacier` as glacier ice;
`climate/sea_ice`), the surface rock (`surface/class`) and a hill shade (`deeptime_core::satellite`
has the rules). It needs no layer kind of its own: it is RGBA8 like every layer, so any reader of
these files (the viewer, the workbench, the game's timelapse) shows it unchanged. Its ramp in
`ramps.json` (`satellite`: categorical, with `labels` and `groups`) is not applied to it: its classes
are the compositor's anchor colours and the legend's key. It is the **first** layer of every frame of
the plate model's group and of the young landscape's and amplification's frames, so a reader that
opens on a frame's first layer opens on it. A frame without climate fields draws bare ground by
geology (no plants, snow or ice).

`rivers` is the water of the frame's terrain: the **last** layer of every frame that has the
satellite view, and the same water is drawn over the satellite view (rivers blended over the land,
lakes flat). It is RGBA8 too: a quiet base (land, sea), the lakes in flat colours (open water, lake
ice where the frame's mean annual temperature at the cell is below `hydro.lake_freeze_mean_c`, and
the playas of basins too dry for a lake as salt flats) and the rivers
in the colours of the `rivers` ramp of `ramps.json`, whose stops run over log₁₀ of the discharge in
m³/s, so a reader can read the discharge back from a pixel's colour. A river is drawn at its width
in the game (since 2026-10-20): one narrower than a pixel is a thread blended over the base at its
width in pixels (at least a tenth), so only a pixel at least half covered is hard-edged on the
ramp and readable; a fainter one is a blend no reader should decode. The ramp's `keys` hold the land, sea, lake, frozen-lake and salt-flat colours, and the
`satellite` ramp's `keys` the three the satellite view draws them in (`deeptime_core::water_view` has the
rules). The water is **derived, not stored**: the bake's own hydrology (`hydro::network`, P1-22: the
open sea, a Priority-Flood of the frame's terrain, lakes from `hydro.lake_min_area_km2` and a block
deep at the level of their water balance, the climate's liquid runoff accumulated down the steepest descent, rivers from
`hydro.river_min_discharge_m3s` and `river_min_area_km2`) is run on the frame's elevation at render
time (a grid over 600,000 cells is averaged down first), so it adds nothing to the simulation or the
bake; with `hydro.enabled` false there is no layer and the satellite view is dry. **The run's last
frame that has these layers is drawn again after the bake**, from `rivers/rivers.dtrv` and the
`water_level` layer (frozen where the frame's climate says so; the playas are the frame's own), so the final map shows the exact rivers and lakes of the world data (the
frame's file and its index entry are rewritten before the manifest lists them); a run without a
bake keeps the derived water.

## The climate's ice fields (pipeline fields and history frames; not world data yet)

The climate stage (`crates/deeptime-core/src/climate/`, plan §8.3; deep-time climate 2026-09-29)
writes these L1 fields into the run's `WorldState` every plate step, *before* the surface stage's
step of the same length, so the surface (glacial erosion) reads the ice of the ground it is about to
erode. Thickness is in metres like the plates' elevations (ice sheets sized in Earth-normalized
lengths, the planet's `R/R⊕` undone). Sea cells hold 0 in the land-ice fields. Since 2026-09-30 the
ice is a flowing ice sheet (`climate/sheet.rs`: shallow-ice flow with the elevation feedback,
isostasy and calving on a coarse grid of ~`climate.ice_grid_cells` columns, bilinearly refined)
where it is thicker than the equilibrium glaciers of the fine ground, so the LGM fields carry
continental sheets that flowed below the snowline.

| Field | Units | What |
|---|---|---|
| `climate/ice_thickness_m` | m | land ice (glaciers and ice sheets) at the end of the step |
| `climate/ice_max_thickness_m` | m | the thickest the ice stood during the step's fine climate clock; inside the recent ice-age window (`climate.orbital_window_myr`, the last 3 Myr by default) the maximum since the window began, so at the present it is the **last-glacial-maximum** ice (moved with the plates) |
| `climate/ice_max_extent` | 0/1 | 1 where `climate/ice_max_thickness_m` > 0 (the LGM extent at the present) |
| `climate/ice_time_share` | 0–1 | share of the step's time the cell was under ice |
| `climate/sea_ice` | 0–1 | annual-mean sea-ice cover of the sea (0 on land) |
| `climate/sea_ice_max` | 0–1 | (P1-26) the winter maximum of the twelve monthly sea-ice covers of the per-cell climate (0 on land); pipeline field, **not baked** |
| `climate/sea_ice_min` | 0–1 | (P1-26) the summer minimum of the same; pipeline field, **not baked** |
| `climate/ice_balance_m_yr` | m ice/yr | surface mass balance of the land ice at the end of the step: positive where snow builds it (above the equilibrium line `climate/ela_m`, at most `ice_accumulation_max_m_yr` × the precipitation in m/yr), negative where it melts (`ice_ablation_gradient` × the ice surface's depth below the line); 0 off the ice. For the hydrology (decisions log 2026-10-01, "Yes, with the ice-sheet branch"): no runoff and no lake where `climate/ice_thickness_m` ≥ 1 m, and meltwater −balance × 0.917 m of water per year where it is negative (the ice edge) |
| `climate/glacier` | 0/1 | (P1-05) 1 where land ice stands at the end of the step (= `climate/ice_thickness_m` > 0) |

**The per-cell climate (P1-26, 2026-10-03) changes no baked layer.** The climate layers a bake reads
and writes (`climate/t_mean_c`, `p_mm`, `koppen`, `month/*`, `ice_cover`, and the ice fields above)
keep their names, units, grids and meaning; only the values come from the 2-D model and, for
`climate/koppen`, from the L2 classification (the bake's `bake/biomes.rs`). `climate/sea_ice` is the
annual mean as before (per cell now, the band's value under `climate.model = banded`).
`climate/sea_ice_max` and `climate/sea_ice_min` are **pipeline fields only**: they live in the run's
`WorldState`, are carried between snapshots like `climate/sea_ice`, and are written to
checkpoints and shown by the viewer, but no bake or world folder stores them and the climate
API (mic-climate) does not see them (design `docs/design/p1-26-per-cell-climate.md`, Q8: seasonal
sea ice is not baked in Phase 1). The per-cell climate's state (the 2-D model's temperatures, slab
and ice, the last sinking region and the flux correction) is part of the climate stage's saved
state in checkpoints, not of any world layer.

The surface stage's field history (the deposit engine's frames, `surface::fields::STEP_FIELDS`)
records two of them as frame means:

| Frame field | Units | What |
|---|---|---|
| `deeptime:ice_thickness_m` | m | time-mean land-ice thickness over the frame |
| `deeptime:ice_cover` | 0–1 | share of the frame's time the cell was under ice (from `climate/ice_time_share`) |

## Checkpoints (`*.dtcp`, dev and CLI only)

`DTCP` fixed fields: `u32 schema, u32 history_frames` (F). Section 0 is a JSON summary (`core_version`,
`schema`, `stage_names`, `next_stage`, `next_stage_name`, `mid_stage`, `t_years`, `events`, `sites`,
`fields`, `history_frames`, `history_bytes`); section 1 is the postcard body: the canonical preset, the
content bundle, the pipeline, the next stage, that stage's saved state when taken mid-stage, the
`WorldState` without its recorded history, and the history's head (`deeptime_core::deposits::recorded::
HistoryHead`: grid, drainage receivers, and per frame its time, span, clocks and each field's name, range,
bits and blob length). Sections 2 … F + 1 are the history's frames, oldest first, one frame's record
each: its fields' LZ4 blobs back to back in the head's order (the record the session's spill file
holds, put together in that order when a stage added fields to the frame after it was spilled). The next stage is always a group's lead: inside a group of coupled stages (plates + surface,
P1-04) the saved state is the postcard `Vec<Vec<u8>>` of every member's state, in pipeline order.
Schema 4 (P1-04) added the surface state and the recorded field history to `WorldState`; schema 6
(P1-17) moved the history's frames out of the body into sections of their own (schema 12, P1-26 T6, added the zonal model's correction to the 2-D climate and the triggered-snapshot counters; schema 11, P1-26, added
the per-cell climate's state; schema 10, 2026-09-30,
added the climate's ice sheet, weathering supply and snowball list; schema 9, P2-08, the world's crater
catalogue; schema 8, P1-21, the world's
volcano catalogue; schema 7, the deep-time
climate, added `WorldState::globals` and the climate stage's carbon cycle, fine clock and ice), so a checkpoint is
written streamed (`<path>.tmp`, fsync, rename; frames copied from memory or from the spill file, never
held whole) and read without loading the frames (every section's CRC is checked when it is read; the
frames are read from the file on demand). Postcard is not self-describing: a checkpoint is read only
by a build with the same schema number and core version (any other is refused naming both), and is
never part of a world's contract with Java.

**The history spill file** (P1-17, not a world-data file). A session whose recorded history does not fit
its memory cap writes the oldest frames' records to `history-<pid>-<n>.spill` in its output folder and
reads them back from there. Fields a later stage adds to a frame already there are appended as a
record of their own (P1-19), so a frame may span several records of the file; its head, in memory,
says where each field is. The file is anonymous (unlinked as soon as it is created on Unix,
delete-on-close on Windows): it uses disk only while the session lives and never appears in, or
survives in, the finished folder. The manifest's `notes.run.history` reports what it held
(`spilled_bytes`, `spilled_frames`, `spill_read_bytes`, …).

## Synthetic worlds (`deeptime synth`)

- `wrap-test` — the closed form of `AnalyticTestTerrain` (plan §7.1), sampled at each cell's
  block: `elevation = round(16·H)`, `water_level` all nodata; climate layers are
  `round(10000·p)` of `AnalyticTestTerrain.raster`'s formulas (temperature by latitude,
  continents from H, erosion 0.2, ridges 0.35, vegetation 0).
- `lake-test` — a plain at y 100 with a round basin and a closed channel; its geometry is in
  `manifest.json` `notes.lake`. Floor y 110, rim crest y 125, `water_level` 120 inside the rim.
  West of it (format 0.6) two closed bowls below sea level (`notes.lake.closed_bowls`): a dry one
  with its floor at y 40 round (−200, −70) and one with its floor at y 30 round (−200, 70) holding
  `water_level` 50, both 1 in a `closed_basin` layer (HYD-100).
- `cube` (format 0.7, cube plan B10) — a cube world ("The cube world's layout") of a planet `--size`
  blocks round: face side `face_for(size)` (1,024 at 4,096), the world map `conformal-n513`, the
  turn azimuth −71.5°, tilt 38°, spin 12° (the parity fixture's free turn). Its terrain is a closed
  form on the sphere sampled at each cell's centre through the map and the turn, so every net edge
  is seamless: `elevation = round(16·H(p))`, H = 63 − 60·p_x + 25·sin(4·p_y + 0.5)·cos(3·p_z) (p the
  planet unit vector), plus a 450-block ridge across the `ne` seam (a quarter turn), a 300-block cone
  on corner `n0` (reachable) and a plateau at y 116 round a lake across the `nn` seam (a half turn):
  floor y 100, `water_level` 108 within 0.06 R of its centre, a 6-block wall. Climate is the polar
  worlds' formulas with the planet's latitude asin(p_z). `manifest.json` `notes.synthetic` holds the
  formula and the features' net blocks. The viewer bundle (`--viewer`) is refused: it draws polar
  worlds; `deeptime world-map` draws the net.
- `deposit-history` (P1-15) — not a `synth --kind`: a `deposits_synthetic` preset with
  `"terrain": {"strata": "placeholder" | "every-class"}` bakes it at `finalize_write` next to the run's
  sites, bodies and events. `elevation` = round(16·(63 + h/20)) of the synthetic history's present
  elevation h (m) sampled bilinearly on L1 at each L3 point — the same surface the deposit stage
  placed and clipped the bodies against (`deposits::bilinear_cells`, `surface_y`); `water_level` all
  nodata; climate as wrap-test's formulas on this terrain; the standard spawn candidates. `crates/deeptime-core/tests/presets/deposits_world_{4k,16k}.json`
  are the GameTest and smoke worlds.
- All: a three-horizon strata placeholder on L1 (`notes.strata`); with `--strata every-class`
  (P1-12) eight horizons 10 blocks apart holding all 62 rock classes, horizon h of L1 cell column
  i having class (6·i + h) mod 62, so any eleven neighbouring cell columns show every class at
  h ≥ 2 (six consecutive classes per column; 11 × 6 = 66 ≥ 62) (`notes.strata.class_rule`). Horizon h spans the keys 4096·h … 4096·h + 4095 in every
  cell, so the column rule correlates the synthetic stacks by index (and mixes the neighbouring
  cell columns' classes in as thin tongues). `--strata basin`: a readable sedimentary basin, the
  same in every cell — sandstone, shale, sandstone, mudstone, limestone, shale, sandstone and shale
  from the surface, 0, 8, 22, 36, 55, 75, 100 and 130 blocks down, granite from 165 blocks to the
  floor (classes `sandstone, shale, mudstone, limestone, granite`); `deposits_world_16k` uses it.
