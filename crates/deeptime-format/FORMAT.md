<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->
# Deep Time world-data format 0.0

The contract between the Rust writer (`crates/deeptime-format`) and the Java reader
(`neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime/worlddata`). Plan §5.3 and §5.7
give the reasons; this file gives the bytes. All integers are **little-endian**. Offsets are
bytes from the start of the file. Before 1.0 a reader accepts exactly its own `0.<minor>` and
refuses anything else with `world data <found>, this Deep Time reads <own>; before 1.0 worlds are
not migrated`.

The version lives in one place: `[workspace.metadata.deeptime] format_version` in the workspace
`Cargo.toml`. `deeptime_format::FORMAT_VERSION` and Java's `FormatVersion.CURRENT` mirror it, and a
test on each side fails when they drift.

## Folder layout

```text
<world-data>/
  manifest.json                 written last (tmp + fsync + rename); present ⇒ folder complete
  rasters/elevation.dtr         L3, i16, 1/16 block
  rasters/water_level.dtr       L3, i16, 1/16 block, nodata −32768
  rasters/climate/<name>.dtr    L2, i16, vanilla parameter × 10,000
                                (<name> ∈ temperature, vegetation, continents, erosion, ridges)
  strata/horizons.dtr           L1, horizon_stack, K = 3 in the synthetic worlds (8 with --strata every-class)
  strata/classes.json           class index → rock class name
  spawn/candidates.json         land cells for spawn selection
  preset.json                   the exact preset of a run (canonical JSON; `deeptime run`, the game)
  events/index.json             the event log's writers and segments (EVENTS.md)
  events/<writer>-<n>.dtev      event segments (EVENTS.md); geology is writer 1
  sites/sites.dtsr              the site registry's records
  sites/index.dtsi              512-block grid → site ids
  keyframes/index.json          the timelapse frames
  keyframes/kf-<n>.dtk          one frame: RGBA8 per layer, LZ4
  checkpoints/<i>-<stage>.dtcp  optional stage checkpoints (dev and CLI only; never needed to play)
```

Every file is written as `<name>.tmp`, fsynced, renamed and its directory fsynced. A writer
deletes an existing `manifest.json` before it starts, so an interrupted write leaves a folder with
no manifest, which readers treat as absent. A run (`deeptime run`, or the game through JNI) writes
`preset.json`, `events/`, `sites/` and `keyframes/index.json` in every folder, empty when the preset
produced nothing; keyframe files are written while the run goes, the rest by `finalizeWrite`, the
manifest last. The synthetic worlds have none of these.

## `manifest.json`

```json
{
  "format_version": "0.0",
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
`grids` and `layers` empty until the bake (P1-08) writes rasters, and every other file in `files`:
`preset.json`, `events/index.json` and each segment, `sites/sites.dtsr`, `sites/index.dtsi`,
`keyframes/index.json` and each frame. `notes.run` records per-stage steps and milliseconds, the
final `t_years` and the event, site and keyframe counts. (P0-07's session wrote an ad-hoc manifest
with `writer`, `preset` and `created_unix_ms`; from P1-01 the session writes this one.)

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
8             2     u16 format_minor (0 for "0.0")
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

## `strata/classes.json`

`{"classes": ["sandstone", "red_bed", "granite"]}` — index → rock class id (plan §5.4.1,
`deeptime:rock/<class>` tags).

## `spawn/candidates.json`

```json
{"rule": "…", "candidates": [{"x": 12, "z": -40, "surface_y": 71.5, "cell": [262, 108]}]}
```

Block coordinates of L3 sample points that are dry land, gently sloping, spread apart, ordered by
distance from (0, 0).

## Sectioned files (`DTEV`, `DTSR`, `DTSI`, `DTKF`, `DTCP`)

Every binary file other than a tile pack shares one container (`container.rs`): the 16-byte header
of plan §5.7.2, fixed fields of the file type, a section table and the sections.

```text
offset   size  field
0        4     magic
4        4     u32 header_len = 16 + F + 16·K (F fixed bytes, K sections)
8        2     u16 format_minor (0 for "0.0")
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
12   4  u32 body id (u32::MAX = none; bodies/ arrives with P1-14)
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

## Keyframes (`keyframes/index.json`, `keyframes/kf-<n>.dtk`)

Timelapse frames (plan §6.1), written while a run goes; frame `n` counts from 0.

```json
{"format_version": "0.0",
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

`DTCP` fixed fields: `u32 schema, u32 reserved`. Section 0 is a JSON summary (`core_version`,
`schema`, `stage_names`, `next_stage`, `next_stage_name`, `mid_stage`, `t_years`, `events`, `sites`,
`fields`); section 1 is the postcard encoding of `deeptime_core::checkpoint::Checkpoint` (the
canonical preset, the content bundle, the pipeline, the next stage, that stage's saved state when
taken mid-stage, and the whole `WorldState`). Postcard is not self-describing: a checkpoint is read
only by a build with the same schema number and core version, and is never part of a world's
contract with Java.

## Synthetic worlds (`deeptime synth`)

- `wrap-test` — the closed form of `AnalyticTestTerrain` (plan §7.1), sampled at each cell's
  block: `elevation = round(16·H)`, `water_level` all nodata; climate layers are
  `round(10000·p)` of `AnalyticTestTerrain.raster`'s formulas (temperature by latitude,
  continents from H, erosion 0.2, ridges 0.35, vegetation 0).
- `lake-test` — a plain at y 100 with a round basin and a closed channel; its geometry is in
  `manifest.json` `notes.lake`. Floor y 110, rim crest y 125, `water_level` 120 inside the rim.
- Both: a three-horizon strata placeholder on L1 (`notes.strata`); with `--strata every-class`
  (P1-12) eight horizons 10 blocks apart holding all 59 rock classes, horizon h of L1 cell column
  i having class (6·i + h) mod 59, so any ten neighbouring cell columns show every class at
  h ≥ 2 (`notes.strata.class_rule`).
