# The equal-area projection (Lambert cylindrical)

Ben, decisions log 2026-10-02: "Use an equal area projects FFS", then "Game world and maps
(Recommended)" and "The equator (Lambert) (Recommended)". This note is the plan for the change,
written from the code before any of it was changed. It is for the coordinator; the build follows
it unless it says otherwise.

## 1. The projection

A planet of circumference C blocks has radius R = C / 2π blocks (2,607.6 at C = 16,384, 5,215.2 at
32,768).

| | equirectangular (until now) | Lambert cylindrical equal-area (now) |
|---|---|---|
| longitude λ of block x | 360°·(x + C/2)/C | unchanged |
| latitude φ of block z | −360°·z/C | **asin(−z/R)** |
| z of latitude φ | −φ/360°·C | **−R·sin φ** |
| poles at | z = ∓C/4 (∓4,096 at 16k) | z = ∓R (∓2,607.6 at 16k) |
| pole to pole | C/2 = 8,192 blocks | 2R = C/π = 5,215 blocks |
| east–west scale (blocks per true block) | sec φ | sec φ |
| north–south scale | 1 | **cos φ** |
| area of one block | (300 m)²·sec φ of planet | **(300 m)² everywhere** |

So x stays linear in longitude (the world still wraps in x with Toroidal World, the seam is still
x = ±C/2), and every block covers the same area of the planet. True scale and shape hold only on
the equator. At 60° land is drawn twice as wide and half as tall as it is (2.0, 0.5); at 75°
3.86 and 0.26; at 80° 5.76 and 0.17.

The simulation itself is untouched where it runs on the Fibonacci sphere (plates, L0): only the
mapping between block space and the sphere changes, and everything that turns rows into
latitudes or distances.

## 2. One projection module on each side

Today the block ↔ sphere mapping is written out in about 60 places (Rust) and 30 (Java, JS),
most as `c / 4` or `90 − 180·j/h`. The change puts it in one place per language, and every other
place calls it:

- **Rust: `deeptime_format::projection`** (new; the format crate, because `GridDesc`, the event
  and site readers, the rivers and structure indexes and the CLI all need it). `Projection::new(C)`
  with `radius`, `wrap_x`, `lon_deg`, `x_of_lon`, `sin_lat`, `lat_deg`, `z_of_lat`, `block_of`,
  `scale_ew`, `scale_ns`, `half_extent(r)` (the grids' z extent, §3), `band_share` (the area
  share of a z band, §4), `picture_z`/`picture_row` and `picture_height` (map pictures, §6) and
  `great_circle_blocks`. Core's `sphere::coupling::BlockFrame` keeps its API (lat/lon ↔ block ↔
  unit vector) and delegates to it; `deposits::fields::{lat_of_row, cell_area_km2}`,
  `deposits::sites::{block_of, lat_lon_of}` and `WorldState::block_of` become thin calls.
  `GridDesc` gains `row_of_z`, `lat_of_row`, `area_share(j)` and `true_cell_m(j, m)`.
- **Java: `worlddata/Projection.java`** (new): the same formulas in the same order (the parity
  tests compare them). `PoleCapCurve.latitudeDeg/zOfLatitude`, `DeepTimeClimate.latitudeDeg`,
  `GlobeProjection.latLon/block/mapPixel`, `BeyondPoleDensity` and the rest call it.
- **JS (workbench, viewer)**: one `DT.proj` object in `workbench.js` (and the same few lines in
  the standalone `viewer.js`): `latOfZ`, `zOfLat`, `yOfLat`/`latOfY` for map pictures. The
  workbench reads R from the run's planet info (`z_pole`), so nothing in JS recomputes C/2π its
  own way.

## 3. The grids: rows uniform in sin φ

**Choice.** Rasters stay square lattices in block space: cell (i, j) is still the point sample at
block (origin_x + i·r, origin_z + j·r), x wraps, z clamps, bilinear in block space. Only the z
extent changes. Because z is linear in sin φ, **rows are uniform in sin φ**, and every cell of
every raster covers the same area of the planet, (r · 300 m)².

**Why this layout.** It is the only one that keeps every reader unchanged: the Java
`RasterSampler`, the column rule (Rust `strata::stack_at` and its Java twin), the amplifier's
ladder (L1 → L2 → L3 by doubling, index-aligned), the tile packs, the bake's L3/L1 squares
(`bake/strata.rs` `raise_tops`) and the chunk generator all work in block space or in index ratios
between grids, and none of them needs to know the projection. Rows uniform in latitude (a
"latitude raster" sampled through the projection per block) would need a non-affine z → row
mapping in every sampler and in the Java↔Rust parity, would make cells unequal in area again
(the opposite of the decision), and would break the amplifier's index-aligned ladder. Rows
uniform in sin φ are also what an equal-area grid has to be: the area between two rows is
2πR²·Δ(sin φ), so equal row spacing in sin φ is equal area per cell exactly.

**Extent.** The rows must cover [−R, R] and every grid of a planet must share its origin so the
ladder stays aligned (L3 row 8j is L1 row j). So all grids of a planet use one half-extent
Zo = the smallest multiple of the quantum q ≥ R, with q = 64 for every real preset (in general the
largest power of two ≤ 64 dividing C/2 and ≤ R/2, raised to the grid's r when that is coarser);
origin z = −Zo, H = 2·Zo / r.

| | C = 16,384 | C = 32,768 |
|---|---|---|
| R (pole) | 2,607.6 | 5,215.2 |
| Zo (grid edge) | 2,624 (16 blocks past the pole) | 5,248 (33 past) |
| L1 r 16 / r 8 | 1024 × 328 / 2048 × 656 (was × 512 / × 1024) | r 32: 1024 × 328; r 16: 2048 × 656 |
| L3 r 2 | 8192 × 2624 (was 8192 × 4096) | r 4: 8192 × 2624; r 2: 16384 × 5248 |

Rows past the pole sample the pole (latitude clamps to ±90°), as row 0 did before. H stays a
multiple of 8, so the flexure solve's coarsening by 2 (`flexure::coarsening`) and the water
view's halving keep working. Every raster has 36 % fewer cells (328/512): the simulation's L1
stages, the bake and the world-data files shrink by about a third.

## 4. Areas: what was weighted by cos φ

Every true area computed from the grid becomes constant: `cell_area_km2(grid, j, m)` =
(r·m/1000)² × `area_share(j)`, where `area_share` is 1 inside the sphere and the share of the
row's band [z − r/2, z + r/2] inside [−R, R] at the edges (the first and last row's bands reach
to infinity, since they take everything clamped to them; the shares sum to 2R/r exactly). Every
caller of `cell_area_km2` (deposit densities and permissive areas, the climate's ocean area and
ice volume, the surface events' areas, the volcano areas, pluton totals) is right without edits.
Code that computes the cos φ weight itself is rewritten to call it: `surface/volcanoes/mod.rs`
(the area per row, "cells shrink towards the poles (the grid is equirectangular)"),
`climate/clock.rs` (band weights by cos φ: those are latitude bands, not rows, and stay),
`climate/belts.rs` (a cos φ-weighted mean over latitude rows: becomes unweighted per row).
`bake/biomes.rs`, `provinces.rs`, `bake/meta.rs` and `deposits/count.rs` compute a row's latitude
by hand; they call `GridDesc::lat_of_row`.

A side effect worth stating: the surface processes run in block metric (§5), and until now a
block-metric cell at 60° stood for half the area it was counted as (rain × area → discharge,
sediment volumes, flexure loads all twice too large per true km² there). With Lambert the block
metric's cell area *is* the true area, so every mass and water budget becomes right at every
latitude without changing a kernel.

## 5. Distances and slopes on the raster

There are two metrics in the code, and the change affects them differently.

**Block metric (the game's metric): unchanged.** Erosion and deposition (`surface/spl.rs`,
`surface/diffusion`, `surface/young.rs`), flow routing and the river network (`surface/routing`,
`hydro/network.rs`: D8 steps of r and r√2 blocks), hillslope diffusion, the amplifier
(`surface/amplify/*`: incision, glacial, character, detail, cones and craters drawn in blocks),
the box blurs (`surface/grid.rs::box_blur`), the grid-artifact fixes of P1-23 (`bake/biomes.rs`
exact Euclidean distance to the sea, the strata smoothing weights, the erodibility interpolation),
the deposits' regional windows and the flexure solve (`surface/flexure.rs`: the 5-point Laplacian
on square cells of the Earth-normalized L1 cell size, with a spectral preconditioner) all use
square cells of r blocks, whatever the latitude. The plan chose that on purpose (§5.2: "landforms
are isotropic in the game world, which is what a player walking through them sees"), and nothing
in these kernels assumed equal spacing *in latitude*: they assume equal spacing in blocks, which
still holds. They stay as they are. What changes for them is only their input: plate-scale relief
sampled from the sphere is now squashed north–south by cos φ as well as stretched east–west by
sec φ, so a range at 60° that runs east–west has twice its true north–south slope in blocks
before erosion works on it. The flexure solve stays isotropic in blocks: the alternative, a
true-metric operator with sec²φ and cos²φ coefficients, would make basins true on the sphere and
squashed in the game, and would break the separable spectral preconditioner; the block metric is
the plan's choice and costs nothing.

**True metric (on the sphere): corrected.** These convert rows to true distances and assumed the
north–south spacing was r blocks:

| where | what | correction |
|---|---|---|
| `climate/mod.rs` `Geometry { sx, sy }` | true cell size per row: sx = dx·cos φ, sy = dx | sx unchanged; **sy per row** = R_m·(φ(z − r/2) − φ(z + r/2)) (`GridDesc::true_cell_m`), so sy is a `Vec` |
| `climate/ice.rs` `distance_to_margin`, `thickness` | separable exact distance transform, columns with one spacing `sy` | the column pass takes the rows' cumulative true positions (the same parabola envelope with positions instead of `s·q`); the margin half-cell uses the row's own sy |
| `climate/snapshot.rs` `true_km`, marches `ds = dx cos φ` | east–west lengths along rows | correct as they are (east–west true size is dx·cos φ in both projections); the latitude comes from the projection |
| `satellite.rs` `hillshade` | true metric: dy = πR/h, dx = 2πR cos φ/w | dy per picture row from the projection (the picture's rows are uniform in sin φ, so dy = 2R/h / cos φ, capped as dx is); latitude from the picture row |
| `surface/events.rs` | great-circle distances from lat/lon | unchanged (it is on the sphere); latitudes come from the projection |
| `deposits/fields.rs` `cell_area_km2` | cos φ area | §4 |

The climate's blurs and smoothing lengths are in cells (`cells(km)` = km / dx): they were
already true only north–south and are now true only on the equator (east–west the true length is
dx·cos φ and north–south dx/cos φ). They stay in cells; the error is the projection's distortion of
smoothing widths, which no one reads as a physical length.

## 6. Maps and pictures

Every map picture of the whole planet (keyframes, the satellite view, the rivers & lakes view,
province view, the CLI's review pictures) becomes a **Lambert map**: x linear in longitude as
before (column 0 at the seam), y linear in sin φ from the north pole (y = 0) to the south pole
(y = h): `sin φ = 1 − 2y/h`. Pictures sample the grid at block z = −R + 2R·y/h (not at grid row
y·H/h, because the grid reaches 16–33 blocks past each pole), through `Projection::picture_z`. The
aspect is π : 1, so the default keyframe size becomes 1024 × 326 (a preset field: every built-in,
pipeline and test preset). `FORMAT.md` "Keyframes" says so.

Consumers of those pictures change their y ↔ latitude rule to the same formula: the Java
`MapWidget` (markers, province hint boxes, the hover readout), `GlobeProjection.render` (texel
lookup) and `mapPixel`; the workbench (`llToCanvas`, `mapLatLon`, `mapXY`, the pixel lookups, the
latitude labels and lines, the sections and deposits overlays, the globe's texture lookup) and the
standalone viewer.

The workbench's zoomable world map (`watch/worldmap.rs` tiles, `worldmap.js`) already works in
block space: its tile pyramid spans z ∈ [−C/4, C/4). It changes to span [−Zo, Zo) (so tiles are
not two-thirds empty), tiles past the pole draw as background, and the latitude lines and readout
use the projection. Sections (`watch/sections`) pick their columns by block (x, z) from lat/lon
through `blockXZ`, which becomes the projection's; inspect clicks go through the same function.

## 7. The world in the game

**Extent.** The playable world is z ∈ [−R, R]: 5,215 blocks pole to pole at 16k (was 8,192),
10,430 at 32k (was 16,384). `BeyondPoleDensity` (the ice-sheet ocean past the pole) starts at |z|
= R instead of C/4; the far-terrain generator for Distant Horizons reads the same rasters by block
and needs no other change.

**The pole cap.** The cap stays defined in latitude (`pole_cap_deg` 75°, `pole_stop_deg` 80°),
so it closes off the same share of the planet as before: poleward of 75° is 3.4 % of the
sphere (it was 16.7 % of the old map), poleward of 80° 1.5 % (was 11.1 %). In blocks the band
becomes narrow:

| | cap line (75°) | stop line (80°) | pole | storm band |
|---|---|---|---|---|
| 16k, equirectangular | 3,413 | 3,641 | 4,096 | 228 blocks |
| 16k, Lambert | 2,519 | 2,568 | 2,608 | **49 blocks** |
| 32k, Lambert | 5,038 | 5,136 | 5,215 | 98 blocks |

The pushback curve is defined over the band's depth s (0 at the cap, 1 at the stop), so a 49-block
band is crossed four to five times sooner: at walking speed (4.3 blocks/s, before the push) the band
takes about 11 s instead of 53 s, and a 40 blocks/s Sable jet entering at 75° is no
longer turned round before the stop (it reaches it in about 1.5 s and is stopped there). The
stop holds either way. This is a gameplay question, not a projection one: whether to widen the
band in blocks (say a cap at 70°, z 2,450 at 16k: a 118-block band and 6 % of the planet behind
it) goes to Ben (STATUS "Pending questions"); the build keeps 75°/80° meanwhile.

**Generation cost and world size.** Chunks between the poles: 36 % fewer (5,215 against 8,192
rows of blocks); inside the stop line 29 % fewer (2 × 2,568 against 2 × 3,641). The bake's L3
rasters and every tile pack shrink by the same 36 %, the simulation's L1 by 36 % per step.

**Latitude-placed things.** Volcanoes, craters, deposit sites, events and plutons are placed on
the sphere and mapped to blocks through the frame (`BlockFrame::block`, `sites::block_of`,
`events.rs`): they follow the projection without edits once the frame does. Their density per
block is now uniform (it was lower by cos φ at high latitude, since a block there covered less
planet). Climate lookups by latitude (`lat_of_row`, `row_lat`, the EBM's bands, `DeepTimeClimate`)
follow from the projection. Biome placement by latitude moves with the rows (TerraBlender's
regions follow provinces, which follow the rows). The spawn search works on L3 cells in block
space and ranks by distance from (0, 0): unchanged. The playability regions (`deposits/
playability.rs`) and the river and structure indexes (`rivers.rs` tiles, `structure.rs` buckets)
index block space from z = −C/4; those are spatial indexes, not geography, and C/4 > Zo, so they
stay as they are (empty rows of tiles past the pole cost a few index entries).

**Features drawn in blocks.** Cones, craters, calderas, dykes, folds and faults are drawn in
blocks at their true size (block metric), so they keep their shape in the game at every latitude
while the land around them is distorted: a crater at 70° is round in the game, though on a
faithful Lambert map of the sphere it would be an ellipse 2.9 times wider than tall. This is the
plan's block-metric choice and is unchanged. The vertical scale (20 m per block, the folds'
`vertical_scale`) does not depend on the projection.

## 8. What the change does not do

- No change to the plate model, L0, its Earth normalization, or anything measured on the sphere.
- No change to the block-metric kernels (§5) beyond their inputs.
- No compatibility: the world-data format goes from 0.3 to 0.4, old bakes, checkpoints and worlds
  are refused (Ben, decisions log 2026-10-01).

## 9. Tasks

1. `deeptime_format::projection` and `GridDesc::for_planet` (origin −Zo, H = 2Zo/r), the new
   `GridDesc` helpers, FORMAT.md "Grids" and "Keyframes", format 0.4 (Rust and Java).
2. Core: `BlockFrame`, `WorldState::block_of`, `deposits::{fields, sites}` delegate; every hand-made
   row latitude (`provinces`, `bake/meta`, `bake/biomes`, `surface/volcanoes`, `deposits/count`,
   `deposits/synthetic`, `structure/deform`, `dummy`, `synth`) through the grid or the projection;
   the `c / 4` extents (`surface/amplify`, `structure/deform/feature`, `water_view`, `satellite`,
   `keyframes`) through the projection.
3. Climate's true metric (§5): `Geometry.sy` per row, the variable-spacing distance transform.
4. Pictures (§6): keyframes, satellite (and its hillshade), water view, provinces, CLI pictures;
   keyframe size 1024 × 326 in every preset.
5. Java: `Projection.java`; `PoleCapCurve`, `DeepTimeClimate`, `BeyondPoleDensity`,
   `WorldDataTerrain`'s crater-lake line, the probes, `PlanetDebugLine`, `GlobeProjection`,
   `MapWidget`, `ReviewModel`, the censuses and the GameTests that place things by C/4.
6. Workbench and viewer JS (§6), `watch/worldmap.rs` tiles and `watch/sections`.
7. Tests (each fails on the old projection): round trips block ↔ lat/lon ↔ raster; Σ cell areas
   = 4πR² and any two interior cells equal; great-circle distance against the raster's metric;
   the seam continuous; Java and Rust agree (a projection parity test, `ColumnParityTest` on the
   re-baked fixture).
8. Evidence: one quick 16k run before and after (hypsometry, land share, the satellite map, the
   polar-ice share of the map); the workbench showing the new map (tiles, sections, inspect).
9. Ledger (SCL-049 rewritten, new SCL/BAK/POL rows), `docs/science/world-scale-and-bake.md`,
   the plan's §5.1–5.2 tables marked as superseded by this note, STATUS build log.
