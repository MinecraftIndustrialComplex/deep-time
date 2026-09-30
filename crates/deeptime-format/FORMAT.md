<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->
# Deep Time world-data format 0.2

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

**0.2 (P1-25)** over 0.1: every header's minor is 2; a planet run with a climate section bakes the
climate stage's twelve months on L1 (`climate/month/t_sl_c_01..12`, `climate/month/p_share_01..12`,
`notes.bake.climate.monthly`), and the climate at a block that Deep Time's public climate API
answers is specified ("The climate at a block"). Nothing a 0.1 world held changed its bytes, only
the minor.

## Folder layout

```text
<world-data>/
  manifest.json                 written last (tmp + fsync + rename); present ⇒ folder complete
  rasters/elevation.dtr         L3, i16, 1/16 block
  rasters/water_level.dtr       L3, i16, 1/16 block, nodata −32768
  rasters/detail/amplitude.dtr  L3, u8, 1/16 block: amplitude of the sub-cell detail term (P1-06)
  rasters/climate/<name>.dtr    L2, i16, vanilla parameter × 10,000
                                (<name> ∈ temperature, vegetation, continents, erosion, ridges)
  rasters/climate/t_mean_c.dtr  L1, i16, 0.01 °C            ┐
  rasters/climate/p_mm.dtr      L1, u16, mm/yr              │ a baked planet run only
  rasters/climate/koppen.dtr    L1, u8, Köppen code index   │ (P1-08, "The bake" below)
  rasters/climate/month/*.dtr   L1: t_sl_c_NN i16 0.01 °C,  │ the twelve months (0.2, P1-25)
                                p_share_NN u8 1/40          │
  rasters/surface/*.dtr         L2, u8                      │
  rasters/meta/*.dtr            L1, u16                     ┘
  strata/horizons.dtr           L1, horizon_stack, K = 3 in the synthetic worlds (8 with --strata every-class,
                                32 in a baked run)
  strata/classes.json           class index → rock class name
  spawn/candidates.json         land cells for spawn selection
  climate_mapping.json          written by the game at the world's first start (P1-09, plan §7.2; not listed
                                in the manifest): the tables from the baked biome inputs to the pack's
                                (see "The climate mapping")
  parity/columns.json           optional: the column evaluator on 500 columns (`deeptime parity`; not listed
                                in the manifest, never read by the game)
  preset.json                   the exact preset of a run (canonical JSON; `deeptime run`, the game)
  events/index.json             the event log's writers and segments (EVENTS.md)
  events/<writer>-<n>.dtev      event segments (EVENTS.md); geology is writer 1
  sites/sites.dtsr              the site registry's records
  sites/index.dtsi              512-block grid → site ids
  bodies/bodies.dtb             deposit bodies (P1-14): parametric primitives with zones
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
            "min_y": -352, "max_y": 639, "sea_level_y": 63},
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

## The bake (a planet run, P1-08)

`Session::finalize_write` bakes every planet preset (`deeptime_core::bake`; plan §5.3, §8.8) after
the amplify stage's L3 packs are listed and before the manifest. `manifest.scale` is the preset's
own scale. The layers (grids from `resolution`: L1 = C / `l1_r` wide, L2 = C / `l2_r`, L3 = C /
`l3_r`; all LZ4 tiles):

| Layer | Grid | dtype | scale, units | nodata | What |
|---|---|---|---|---|---|
| `elevation` | L3 | i16 | 1/16, block | — | terrain y: the amplifier's; without the amplify stage the young landscape (L2) or the deep-time surface (L1) upsampled bilinearly |
| `detail/amplitude` | L3 | u8 | 1/16, block | — | the detail term's A (P1-06; absent without the amplify stage) |
| `water_level` | L3 | i16 | 1/16, block | −32768 | every cell nodata: lakes and rivers are Phase 2 |
| `strata/horizons` | L1 | horizon_stack | 1/16 | — | P1-07's stacks, the first top raised ("The P1-08 bake") |
| `climate/t_mean_c` | L1 | i16 | 0.01, degC | — | mean annual temperature |
| `climate/p_mm` | L1 | u16 | 1, mm/yr | — | annual precipitation |
| `climate/koppen` | L1 | u8 | 1, code index | 255 | `notes.bake.climate.koppen_codes[i]` (31 = sea) |
| `climate/month/t_sl_c_{01..12}` | L1 | i16 | 0.01, degC | — | each month's mean temperature at sea level (the lapse rate taken out: `+ lapse_c_per_km · max(0, h − sea)`), from the climate stage's last snapshot (0.2) |
| `climate/month/p_share_{01..12}` | L1 | u8 | 1/40, ratio | — | each month's precipitation rate over the twelve's mean (0–6.375; they sum to 12 within the quantum) (0.2) |
| `climate/{temperature,vegetation,continents,erosion,ridges}` | L2 | i16 | 1e-4, vanilla_param | — | the router's five biome inputs, placed against the reference bands and vanilla's land distributions (P1-09, "The climate mapping") |
| `surface/regolith_blocks` | L2 | u8 | 1/16, block | — | regolith thickness |
| `surface/class_top` | L2 | u8 | 1, class index | 255 | the rock class at the surface (`strata/classes.json`) |
| `meta/plate_id` | L1 | u16 | 1, plate id | 65535 | the plate |
| `meta/crust_age_myr` | L1 | u16 | 1, Myr | 65535 | oceanic crust age; nodata on continental crust |
| `meta/province` | L1 | u16 | 1, province id | — | contiguous geologic provinces ≥ 50,000 km² (`notes.bake.meta.provinces`) |

`notes.bake`: `version` (1), `baked`, per part (`terrain`, `strata`, `climate`, `surface`, `meta`)
what it did, `millis` per part, `bytes` and `layers` written, and **`detail`** — the game's detail
term: `{"layer": "detail/amplitude", "seed": "<u64, decimal>", "salt": 1729, "octaves": 3,
"wavelength_blocks": 32.0}` (present exactly when `detail/amplitude` is). A preset without the
deep-time surface (plate-only) bakes nothing (`baked: false`).

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
  "glacier_cap": 0.35,                   land above its equilibrium line (climate/glacier) is held at or below it
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
    "BSk": { "temperature": { "when": { "of": "t_cold", "above": 0.0 },   the ramp "then" where the input is above
                              "then": "temperate", "else": "boreal" },     the threshold, "else" elsewhere
             "humidity": "steppe" }
  }
}
```

The default table reproduces what P1-09 first compiled in, operation for operation (a test holds
the core's default to the old formulas bit for bit). A ramp is a `base`/`span` pair rather than two
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
the Create World editor's runs and the workbench do (they pass no bundle, as they pass no
essentials). The table the bake used is recorded whole in the world data: `notes.bake.climate.placement`
is `{"source": "default" | "content bundle", "table": {…}}`. The game does not read the table
again: the world's biome inputs are already baked with it.

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

## Grids

Equirectangular in block space (plan §5.2). A grid has `w` × `h` cells with `w = C / r`,
`h = w / 2`, `r` blocks per cell, origin `(−C/2, −C/4)`.

- Cell `(i, j)` is the **point sample** at block `x = origin_block_x + i·r`,
  `z = origin_block_z + j·r` (not the centre of an r × r square). Column 0 lies on the seam.
- Row-major: row `j` (z) outer, column `i` (x) inner. Row 0 is the north edge (z = −C/4).
- Indices wrap in x: `i mod w` (Euclidean). They clamp in z: `clamp(j, 0, h − 1)`.
- Bilinear sampling at block `(bx, bz)`:
  `u = (bx − ox) / r`, `v = (bz − oz) / r`, `i0 = floor(u)`, `j0 = floor(v)`, `fu = u − i0`,
  `fv = v − j0`, then `a = v00 + (v10 − v00)·fu`, `b = v01 + (v11 − v01)·fu`,
  result `a + (b − a)·fv` on physical values, in exactly this order (both sides agree bit for bit).
  Any nodata corner makes the result NaN.
- Nearest sampling: the cell `(floor(u + 0.5), floor(v + 0.5))`.

## Tile pack (`.dtr`)

```text
offset        size  field
0             4     magic "DTR1" (0x44 0x54 0x52 0x31)
4             4     u32 header_len = 16 + json_len (bytes from file start to the end of the JSON)
8             2     u16 format_minor (2 for "0.2")
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

- `dtype`: `u8`, `i8`, `u16`, `i16`, `u32`, `f32` (IEEE 754 binary32), or `horizon_stack`.
- physical value = raw · `scale` + `offset`. `nodata` (optional) is a **raw** value.
- `stack_depth` (only for `horizon_stack`): K.
- Unknown fields must be ignored by readers.

### `horizon_stack` cells (plan §5.7.4)

`cell_bytes = 3K`: K × i16 horizon top elevation (1/16 block), then K × u8 class index (into
`strata/classes.json`). Horizons are ordered top-down; horizon k's rock fills from its top down to
horizon k+1's top, the last one down to `min_y`. Unused horizons have top −32768 and class 255.

A column at block (x, z) is read as the Java strata pass reads it (`StrataPass.column`;
`LoadedLayer::stack_at` in Rust): x wrapped into [−C/2, C/2), u = (x − origin_x)/r, v = (z −
origin_z)/r; each horizon's class from the nearest cell (⌊u + ½⌋, ⌊v + ½⌋), its top bilinear
between the four cells around (u, v) (the nearest cell's top where a corner lacks the horizon),
floored to a block and never above the horizon over it; the walk stops at the first unused horizon
of the nearest cell.

**The P1-07 bake** (`deeptime_core::structure::bake_horizons`) makes stacks with these properties:
the first horizon's top is the cell's surface (y = sea_level_y + (h − sea level)/m_per_block_v,
clamped to [min_y + 1, max_y]); every horizon is at least one block (16 raw) thick, the last one
down to `min_y` too; at most K are used; neighbours differ in class. A stack walks the column of
`structure::columns` from the top: regolith, the strata log (the young, shallow part of a clastic
deposit as its `loose_equivalent`, buried loose material and palaeosols as their `lithifies_to`),
the plate model's rock pools less the log, and peridotite below the crust. Plutons are bodies
(role `pluton`, below), not horizons.

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

## The detail layer (`rasters/detail/amplitude.dtr`, P1-06)

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

## Columns (the column evaluator, format 0.1)

What Deep Time decides for block column (x, z) — Rust `deeptime_format::column::ColumnWorld::sample`
(behind `deeptime inspect-column`, `deeptime parity` and the JNI `sampleColumn`), Java
`worlddata/Columns.sample` (which the strata pass's column rule is part of). Both must give equal
answers for every column (`ColumnParityTest`):

1. `wx` = x folded into [−C/2, C/2). `elevation` = E(wx, z), `detail` = A·N at (wx, z) (0 without a
   detail term), `surface` = H = E + A·N (E alone without a detail term); `top_y` = ⌈H⌉ − 1 clamped
   to [min_y − 1, max_y] (min_y − 1 when H is not finite).
2. `water` = the `water_level` layer's nearest cell at (wx, z), null where nodata or absent.
3. `horizons` = the stack at (x, z) by the column rule of "`horizon_stack` cells": `[top y, class
   index]`, top-down.
4. `bodies`: every body whose AABB footprint holds (wx, z) (z inside [min z, max z]; wx inside the x
   span with wrap), ascending id; for each, y from min(aabb max y, max_y) down to max(aabb min y,
   min_y) located at block (wx, y, z) ("Bodies", step 1–3); each maximal run of inside blocks is
   `[body id, top y, bottom y]`.
5. `rock`: for y from `top_y` down to min_y, the class name of the horizon holding y (none above the
   first top, or for an index outside `strata/classes.json`), then for each body of step 4 in order
   that holds block (wx, y, z), its `rock_class`, a zone's where one applies (the last zone that
   applies and names one); a body naming none leaves the class as it is. Runs of equal class are
   `[top y, bottom y, class or null]`.

`parity/columns.json` (`deeptime parity --world-data <dir>`): `{"format_version", "world" (the
manifest's source), "circumference", "min_y", "max_y", "seed" (decimal), "classes", "columns": [ … ]}`,
each column the object above plus its `x` and `z`. Numbers are serde_json's shortest round-trip
form: read them with a correctly rounded parser (Java's `Double.parseDouble`) and compare exactly.
The 500 columns: 10 % within 40 blocks of the seam, 10 % near and past the poles, 20 % inside
random bodies' boxes (none when the world has no bodies), the rest anywhere with x over two laps.

## `strata/classes.json`

`{"classes": ["sandstone", "red_bed", "granite"]}` — index → rock class id (plan §5.4.1,
`deeptime:rock/<class>` tags).

## `spawn/candidates.json`

```json
{"rule": "…", "candidates": [{"x": 12, "z": -40, "surface_y": 71.5, "cell": [262, 108]}]}
```

Block coordinates of L3 sample points that are dry land, gently sloping, spread apart, ordered by
distance from (0, 0).

A world whose essentials set `max_blocks_from_spawn` (P1-15, plan §7.8; a synthetic deposit world
today) ranks its candidates instead (`deeptime_core::deposits::spawn`): points on a 16-block lattice
that are dry, at least sea level + 2 and within one block of their L3 neighbours, ordered by the
number of such essentials whose nearest site is out of reach (`unmet`), then
`score` = Σ min(d / max_blocks_from_spawn, 4), then distance from (0, 0); at most 10, 64 blocks apart.
The file then also has `ranked_by` (`[{material, max_blocks_from_spawn, sites}]`) and each candidate
`unmet`, `score` and `essentials` (`[{material, site, distance, max_blocks_from_spawn}]`). The game
takes the dry candidate with the fewest `unmet`, file order breaking ties (`SpawnHandler`).

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
ceil(C/2/512), i32 origin_x (−C/2), i32 origin_z (−C/4), u32 entries, u32 reserved`. Section 0 is
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
40   12 f32 × 3 semi-axes a, b, c (blocks)       36  4  reserved, 0
52   4  f32 yaw (radians about +y, +x → +z)
56   4  f32 dip (radians about the local x axis)
60   4  i32 clip_top_y (i32::MAX = none)
64   24 i32 × 6 AABB min x, y, z, max x, y, z (inclusive; x may leave the canonical span)
88   4  u32 first zone
92   4  u32 zone count
96   4  u32 rock class of the whole body (string, u32::MAX = none)
100  4  reserved, 0
104  8  reserved, 0 (voxel blobs, later)
```

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
share the block's probability, capped at 1).

**Plutons** (P1-07, `structure::plutons`): ellipsoid bodies with no site, role `pluton`, the
body's `rock_class` its intrusion (granodiorite, granite or diorite), and, where the roof's host
was a carbonate or another non-crystalline rock, one zone `aureole` (no material, density 0) whose
`rock_class` (skarn or hornfels) holds the outer shell d ∈ [0.85, 1] (`structure.aureole_shell`).
Their ids come before any deposit body the deposit stage adds later, so where they overlap the
pluton paints first.

The AABB is conservative: the world extent along axis i is `Σ_j |R_ji|·h_j` around the centre
(R the rotation above, h the semi-axes), clipped at `clip_top_y`. The viewer's `report.js` carries
each body as JSON: `id, site, shape, metric, role, center, axes, yaw, dip, clip_top_y (null = none),
aabb, rock_class, zones [{name, material, rock_class, d, v, grade, density}]`.

`manifest.json` `notes.deposits` (runs with the deposit stage) is the stage's census: per type
`id, sites, target (the survivors the density asks for: count_mult · ρ · Σ permissive area, plan
§8.6), preservation (the formation-weighted share the planet preserves), formation_mult, capped,
expected_formed, expected (the survivors expected; = target unless capped), drawn (deposits
formed), tracts, permissive_km2, mean_f, buried, truncated, eroded, too_deep, children, guaranteed,
dropped_separation`, plus `province`, `provinces`, `count_mult`, `max_formed_per_survivor`,
`brine_flow_km`, `sites`, `bodies`. A forming event's `p_survive` is its cell's survival chance.

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
Pixel (px, py) is centred on longitude 360°·px/w (the seam is 0°, plan §5.2) and latitude
90° − 180°·py/h, the nearest cell of the field it was drawn from, coloured with `crates/deeptime-viewer/assets/ramps.json`
(the same ramps the viewer and the game use); alpha 0 is nodata. Frames are taken when the clock
passes the next multiple of the preset's keyframe interval (counted from the run's start) and at
the end of each stage.

## Checkpoints (`*.dtcp`, dev and CLI only)

`DTCP` fixed fields: `u32 schema, u32 history_frames` (F). Section 0 is a JSON summary (`core_version`,
`schema`, `stage_names`, `next_stage`, `next_stage_name`, `mid_stage`, `t_years`, `events`, `sites`,
`fields`, `history_frames`, `history_bytes`); section 1 is the postcard body: the canonical preset, the
content bundle, the pipeline, the next stage, that stage's saved state when taken mid-stage, the
`WorldState` without its recorded history, and the history's head (`deeptime_core::deposits::recorded::
HistoryHead`: grid, drainage receivers, and per frame its time, span, clocks and each field's name, range,
bits and blob length). Sections 2 … F + 1 are the history's frames, oldest first, one frame's record
each: its fields' LZ4 blobs back to back in the head's order (the same record the session's spill file
holds). The next stage is always a group's lead: inside a group of coupled stages (plates + surface,
P1-04) the saved state is the postcard `Vec<Vec<u8>>` of every member's state, in pipeline order.
Schema 4 (P1-04) added the surface state and the recorded field history to `WorldState`; schema 6
(P1-17) moved the history's frames out of the body into sections of their own, so a checkpoint is
written streamed (`<path>.tmp`, fsync, rename; frames copied from memory or from the spill file, never
held whole) and read without loading the frames (every section's CRC is checked when it is read; the
frames are read from the file on demand). Postcard is not self-describing: a checkpoint is read only
by a build with the same schema number and core version (any other is refused naming both), and is
never part of a world's contract with Java.

**The history spill file** (P1-17, not a world-data file). A session whose recorded history does not fit
its memory cap writes the oldest frames' records to `history-<pid>-<n>.spill` in its output folder and
reads them back from there. The file is anonymous (unlinked as soon as it is created on Unix,
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
- `deposit-history` (P1-15) — not a `synth --kind`: a `deposits_synthetic` preset with
  `"terrain": {"strata": "placeholder" | "every-class"}` bakes it at `finalize_write` next to the run's
  sites, bodies and events. `elevation` = round(16·(63 + h/20)) of the synthetic history's present
  elevation h (m) sampled bilinearly on L1 at each L3 point — the same surface the deposit stage
  placed and clipped the bodies against (`deposits::bilinear_cells`, `surface_y`); `water_level` all
  nodata; climate as wrap-test's formulas on this terrain; spawn candidates ranked by the preset's
  (or the content bundle's) essentials. `crates/deeptime-core/tests/presets/deposits_world_{4k,16k}.json`
  are the GameTest and smoke worlds.
- All: a three-horizon strata placeholder on L1 (`notes.strata`); with `--strata every-class`
  (P1-12) eight horizons 10 blocks apart holding all 59 rock classes, horizon h of L1 cell column
  i having class (6·i + h) mod 59, so any ten neighbouring cell columns show every class at
  h ≥ 2 (`notes.strata.class_rule`).
