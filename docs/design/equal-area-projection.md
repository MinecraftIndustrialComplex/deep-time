# The map projection: one coordinate layer, any projection

**Status (2026-10-02).** Ben first chose Lambert's cylindrical equal-area projection (decisions
log 2026-10-02: "Use an equal area projects FFS", then "Game world and maps" and "The equator
(Lambert)"). He has since reopened the choice. He wants the Peirce quincuncial projection and its
newer relatives looked at: square maps that tile the plane. A research agent is on that. So this
note covers any projection. It says:

- what the code assumes today;
- the single projection module, in Rust, Java and JS, that every coordinate conversion should go
  through;
- every place that converts between block coordinates, raster coordinates and latitude/longitude
  (§3);
- every place where a distance, an area or a slope depends on how the raster maps onto the sphere
  (§4);
- what each family of projection would change (§5), including a world that wraps in both x and z
  (a torus) instead of in x only with a pole cap.

It ends with Lambert's numbers (§6), the baseline already measured (§7) and the tasks (§8).

**Work already done.** A Lambert implementation was half built before the hold. It is parked on
branch `projection-lambert-wip` (6529b910, on top of this note's first version):

- `deeptime_format::projection`, the `GridDesc` helpers and `worlddata/Projection.java`;
- about 70 call sites in the Rust core, the CLI and the Java mod;
- world-data format 0.4.

It compiles. 35 core tests that pin the old layout still need new numbers, and the workbench JS is
not converted. It is the worked example behind §3, and it is the starting point if Ben comes back to
Lambert. This branch carries only this note.

## 1. What the code assumes today

- **L0**, the plates and crust, runs on a Fibonacci sphere. No projection is involved; it meets the
  rasters only through `sphere::coupling` (every raster cell's unit vector).
- **L1–L3** rasters are square lattices in block space. Cell (i, j) is the point sample at block
  (−C/2 + i·r, −C/4 + j·r), with W = C/r and H = W/2. Indices wrap in x and clamp in z. The grids
  of the ladder nest by index (L3 row 8j is L1 row j), and the amplifier and the bake rely on that.
- **The projection is equirectangular.** x is linear in longitude, z is linear in latitude
  (φ = −360°·z/C), and the poles are at z = ∓C/4.
- **That projection is written out by hand** about 60 times in Rust, about 30 in Java and about 25
  in the JS and Python tools. Most are a bare `c / 4` or `90 − 180·j/h` (§3). The only central
  piece is `sphere::coupling::BlockFrame` (Rust), and even there `lat_of_row`, `cell_area_km2`,
  `sites::block_of` and `WorldState::block_of` each repeat the formula.
- **Two metrics.**
  - The *block metric*: square cells of r blocks, what a player walks. Erosion, routing, diffusion,
    the amplifier, flexure and the grid-artifact fixes use it.
  - The *true metric*: lengths on the sphere. The climate's geometry, the ice-sheet distance
    transform, the hill shades and every area weight use it.
- **Topology.** The world wraps in x through Toroidal World's cylinder. Past |z| = C/4 there is no
  planet: a pole cap stops players at 80°, and `BeyondPoleDensity` fills the rest with an ice
  ocean.

## 2. The projection module

One module per language, with the same formulas in the same order:

- Rust: `deeptime_format::projection`, in the format crate. `GridDesc`, the event and site readers
  and every crate need it.
- Java: `worlddata/Projection`.
- JS: `window.DTProj` in `crates/deeptime-viewer/assets/projection.js`, loaded first by the
  workbench and by the viewer bundle (as built; the first plan was a `DT.proj` in `workbench.js`
  and a copy in `viewer.js`). It takes C from the run, so the pages never derive the layout
  themselves.

Core's `BlockFrame` keeps its `glam` API and delegates to the module. Every other site in §3 calls
one of the functions below.

**Interface.**

| function | meaning | today (equirect) | Lambert (WIP) | square or torus maps |
|---|---|---|---|---|
| `forward(x, z)` → (φ, λ), `unit(x, z)` | block to sphere; single-valued for every projection | φ = −360°z/C | φ = asin(−z/R), R = C/2π | elliptic functions (Peirce) or a polynomial approximation |
| `inverse(φ, λ)` → canonical (x, z) | sphere to block | z = −φC/360 | z = −R sin φ | as forward; on a torus a place has two blocks, and `copies(x, z)` lists them |
| `canonical(x, z)` → (x′, z′, turn) | folds any block into the canonical domain; every reader folds its query first | wrap x mod C | same | torus: the translation lattice; quincuncial: a 180° turn about an edge midpoint |
| `extent()`, `on_planet(x, z)` | the canonical domain in blocks; past it is not planet | C × C/2 | C × 2R | a × a square, or √2a × √2a |
| `metric(x, z)` → (g_xx, g_xz, g_zz) | blocks of planet per block, squared | diag(cos²φ, 1) | diag(cos²φ, 1/cos²φ) | conformal: s(x,z)²·I |
| `area_scale(x, z)` = √det g | area of planet per block | cos φ | 1 | s² for conformal, 1 for an equal-area relative |
| `north(x, z)` | unit vector of north in block space (winds, azimuths, light) | (0, −1) | (0, −1) | rotates across the map |
| `cell_area(grid, i, j)` | area of a raster cell (∫ area element) | (r·m)² cos φ, per row | (r·m)², edge rows clipped | per cell |
| `grid(C, r)` | origin, W, H; shared origin so the ladder nests; H a multiple of 8 for flexure's coarsening | (−C/2, −C/4) | (−C/2, −Zo), Zo the multiple of 64 past R | the canonical square |
| `neighbour(i, j, di, dj)` → (i′, j′, turn) | the raster's topology | wrap x, stop at the rows | same | wrap both, or fold with a turn |
| map picture | a map picture is the canonical domain drawn in block space, x and z linear | equirect 2:1 | Lambert π:1 (y linear in sin φ) | the square, or two of them |

**Parity.** `deeptime parity` writes the projection into the fixture that `ColumnParityTest`
reads: each picked column's (φ, λ) and the z of every 5° of latitude. Java must agree to 1e-9,
not bit for bit, because `asin` and the elliptic functions are not bitwise-portable. The grids
come to Java through each layer's JSON, so the raster sampling stays bitwise.

## 3. Inventory: every block ↔ raster ↔ latitude/longitude conversion

Each site is marked with what it does:

| class | what the site does |
|---|---|
| **F** | block → latitude/longitude |
| **I** | latitude/longitude → block (several blocks on a torus) |
| **R** | row → latitude (holds only for cylindrical projections) |
| **E** | the world's z extent, `C/4`, or the pole edge |
| **P** | map-picture pixel ↔ latitude or ↔ grid row, including the picture's aspect |
| **W** | an area weight per row (cos φ) |
| **S** | a spatial index framed from z = −C/4 (not geographic; it can keep any frame that covers the world) |
| **T** | topology: wrap in x, stop or clamp at the first and last rows |

### Rust: format crate

- `raster.rs` `GridDesc::for_planet` (E: origin −C/4, H = W/2) and its doc; `wrap_i`, `clamp_j` (T).
- `events.rs` `cell_of` (I), `CellIndex`.
- `column.rs` inspect JSON `lat`/`lon` (F); `wrap` (T).
- `sites.rs` site index origin −C/4 (S). `rivers.rs` tiles from −C/4 (S, T). `structure.rs`
  buckets from −C/4 (S, T). `strata.rs` and `lazy.rs` are generic (T through `GridDesc`).
- `FORMAT.md` "Grids", "Keyframes", crater-lake line. `EVENTS.md` place → block.

### Rust: core

- `sphere/coupling.rs` `BlockFrame` (F, I, unit vectors). Through it go the L0 coupling, `surface/
  advect.rs`, the plate stage and the placement of volcanoes, craters, plutons and structures.
- `world.rs` `WorldState::block_of` (I).
- `deposits/fields.rs` `lat_of_row` (R) and `cell_area_km2` (W). Their callers:
  - `climate/mod.rs` Geometry;
  - `climate/snapshot.rs` `ice_cover`;
  - `surface/events.rs`, `surface/fields.rs` `row_lat`, `surface/mod.rs` (the climate stand-in);
  - `structure/plutons.rs`;
  - the CLI's hypsometry lines.
- `deposits/sites.rs` `block_of`, `lat_lon_of` (F, I), called five times from `deposits/mod.rs`.
- `deposits/count.rs` `Provinces::new` (F, by hand). `deposits/synthetic.rs` (F, by hand).
- `deposits/cluster.rs`: "inside the planet" is |z| < C/4 (E). `deposits/playability.rs` regions
  from −C/4 (S). `deposits/regional.rs` windows (T: wrap x, clip at the rows).
- `provinces.rs` `coherence` (W, R: cos weights from `(j + ½)/h`). `bake/meta.rs` `shares` (W, R).
- `bake/biomes.rs` fallback temperature from `(j + ½)/h` (R).
- `surface/volcanoes/mod.rs` landmass area (W). `surface/impacts/mod.rs` `frame.lat_deg` (F).
- `structure/deform/mod.rs`: event place → block (I), cluster centre → lat/lon (F).
  `structure/deform/feature.rs` z clamp ±C/4 (E).
- `surface/amplify.rs`:
  - noise and character `origin_z = −C/4` (E);
  - the preview grid `for_planet(r·stride)`, which must keep the parent grid's origin (E);
  - `amplify/window.rs` halos (T).
- `climate/mod.rs` Geometry `sx`, `sy`, `area` (W, R, true metric); EBM band per row (R).
- `climate/snapshot.rs`:
  - `true_km` and the marches' `ds = dx·cos φ` (true metric, R);
  - the marches run along rows, so rows are taken to be parallels and the winds east–west (R);
  - orographic precipitation over bands of rows with a wind (u east, v south) (R, T).
- `climate/belts.rs` ITCZ: cos weights over rows, smoothing along rows (W, R).
- `climate/ice.rs` distance transform: `sx` per row, one `sy`, separable (true metric, R, T).
- `climate/ebm.rs`, `climate/clock.rs`: bands of latitude, independent of the map.
- `satellite.rs`:
  - `nearest` and `mean` take picture ↔ grid by index ratio (P);
  - `hillshade` takes latitude per picture row, dy = πR/h (P, true metric).
- `keyframes.rs` `render` (P); default size 1024 × 512 (P).
- `water_view.rs` `lake_cover`, `draw_rivers` (P). `dummy.rs` default keyframe size and latitude
  per row (P, R). `synth/terrain.rs` `latitude(z)` (F; twin of Java `AnalyticTestTerrain`).
- `params.rs` keyframe size default; every preset's `keyframes.size` and the description text
  "L1 1024x512…" (P).
- `surface/grid.rs` `Dims::neighbour`, `box_blur` (T). `surface/flexure.rs` FFT in x and DCT in z,
  coarsening by index (T). `climate/orographic.rs` FFT in x, zero-padded rows (T).
  `hydro/network.rs`: the first and last rows are outlets (T, E).

### Rust: CLI, inspect, viewer

- `run.rs` hypsometry lines (W through `cell_area_km2`).
- `satellite_view.rs` and `main.rs` picture height w/2 (P).
- `climate_dem.rs`: an equirectangular Earth DEM with latitude per row. Under another projection it
  must be resampled onto the planet's grid (R, P).
- `amplify_view.rs` `lonlat`, the centre latitude of a window (R).
- `volcano_view.rs` +C/4 offsets (E). `structure_view.rs` full sections z ±C/4 (E).
- `parity.rs`, `dump.rs`, `strata_check.rs`, `synth/mod.rs`: sample z over ±C/4 (E).
- `watch/worldmap.rs`: a 2:1 tile pyramid with z0 = −C/4 (E, P); hill shade with latitude per tile
  row (true metric).
- `watch/sections/slice.rs` `pixel_block` (P). `watch/sections/mod.rs` slice size w/2 (P).
- `deeptime-inspect` `census.rs` and `rivermap.rs` lattices over ±C/4 (E).
- `deeptime-viewer` `lib.rs` satellite pseudo-grid origin −C/4 (E, P).

### Java

- `wrap/PoleCapCurve` `latitudeDeg`, `zOfLatitude`, `capZ`, `stopZ` (F, I, E). Used by `PoleCap`,
  `compat/sable/SablePoleCap`, `client/PolarStorm`, `server/DeepTimeCommands` and
  `net/PlanetDebugLine` (the F3 line).
- `api/climate/DeepTimeClimate.latitudeDeg` (F). This is the public climate API (API 2), so its
  documented formula is part of the contract.
- `worldgen/density/BeyondPoleDensity` (E). `worldgen/WorldDataTerrain` crater-lake line
  C/4·cap/90 (I). `server/GeothermalProbe`, `server/MagmaProbe` zMax (I).
- `worldgen/AnalyticTestTerrain` `latitude` (F; twin).
- `climate/BiomeCensus`, `climate/RegionCensus` lattices over ±C/4 (E).
- `worlddata/GridDesc`, `RasterSampler`, `Columns` (the 4 × 4 column rule) and `FarColumn` (T).
  `Structure`, `Rivers` and `SiteData` frames (S).
- `screen/model/GlobeProjection` `render` texel, `mapPixel`, `latLon`, `block` (P, F, I).
  `screen/MapWidget` `fit` 2:1, province boxes, hover readout (P). `screen/model/ReviewModel` 512 ×
  256 map, latitude per row (P). `ProvinceHints` 30° boxes (drawn through `MapWidget`).
- GameTests that place things by C/4 or check latitudes (E, F):
  - `NetworkGameTests` (the F3 strings), `ClimateApiGameTests`, `EditorModelGameTests`;
  - `LavaGameTests`, `RealPlanetGameTests`, `BiomeGameTests`, `ImpactGameTests`;
  - `FarTerrainGameTests`, `TerraBlenderChecks`;
  - `PoleCapGameTests`, which works in latitude and moves with the curve.
- JUnit: `PoleCapCurveTest`, `ModelHelpersTest`, `FarColumnTest`, `DeepTimeClimateTest`, and
  `MagmaTest`/`GeothermalTest` (lake line C/4).

### JS and Python (not converted in the WIP)

- `workbench.js`:
  - `llToCanvas`, `mapLatLon`, `mapXY`, `blockXZ` (the inspect and section clicks);
  - the pixel lookups;
  - the latitude labels and graticule, box selection;
  - the globe's texture lookups and graticule (P, F, I).
- `worldmap.js`: the height of `maxBpp` (C/2), the pan limit ±C/4, tile rows from −C/4, latitude
  lines, readout, preview boxes (E, P, F).
- `sections.js`: latitude lines (P). `deposits.js`: footprints at C/360 blocks per degree on both
  axes, `clat` from z (F, true scale). `viewer.js`: lines, readout, pan limits, `toCanvas` (E, P, F).
- `tools/workbench/preview.py` box and inspect limits ±C/4 (E). `tools/review/grid_audit.py`,
  `p2-08-craters.py`, the `seams` and `wrap` review pages (E).

### Docs

- Plan §5.1 (the derived-geometry table) and §5.2.
- `FORMAT.md`, `EVENTS.md`.
- `docs/science/world-scale-and-bake.md` and `pole-cap.md`.
- Ledger SCL-049 (the frame) and POL-005, -006, -014 and -015.

### Tests that pin the layout

In the Lambert WIP, 35 core tests failed and still need new numbers:

- the water view (12);
- the amplifier and pipeline runs (4) and the bake (1);
- satellite pictures (3), keyframes (2) and hydrology (2);
- cluster, tracts, dummy, biomes, session, two `bake_horizons` tests, plate invariants, the volcano
  front and climate resume.

Also two CLI tests (the synth read-back and sections) and one format test (`lazy`).

Most build a small planet and index rows by hand. A projection-neutral refactor (§8, step 1) would
leave them as they are.

## 4. Distances, areas and slopes

**Block-metric kernels.** These are unchanged by any projection, as long as rasters stay square
lattices in block space:

- erosion, deposition and hillslope diffusion (`surface/spl.rs`, `surface/diffusion`,
  `surface/young.rs`);
- D8 routing and the river network (`surface/routing`, `hydro/network.rs`);
- the amplifier (incision, glacial, character, detail; cones, craters, folds and faults drawn in
  blocks);
- box blurs; the deposits' regional windows; the orographic precipitation's own lengths;
- the P1-23 grid-artifact fixes: the exact distance to the sea, the strata smoothing weights, the
  erodibility interpolation;
- the flexure solve: a square-cell Laplacian, Earth-normalized by one factor.

They assume square cells in *blocks*, never even spacing in latitude. They need two things from
the projection:

1. **Cell area.** Discharge (Σ rain × area), sediment volumes and flexure loads use the block area
   dx². That equals the planet's area only on an equal-area projection.
   - Equirectangular: at 60° a cell stands for half the planet it is counted as.
   - Lambert: right everywhere.
   - A conformal square map (Peirce): a cell stands for s(x,z)² of its block area, and s goes to 0
     at the four singular points.

   Either the projection is equal-area, or these kernels take a per-cell area factor.
2. **Flexure's Earth normalization.** It is one number for the whole map, which is right only
   where a block's true size does not vary across the map.
   - No cylindrical projection has that away from the equator.
   - Lambert at least keeps the area.
   - A conformal map whose scale varies would need a per-cell factor or a variable-coefficient
     operator.

**True-metric sites.** These need the projection's metric:

| where | today | correction |
|---|---|---|
| `climate/mod.rs` Geometry (`sx`, `sy`, `area` per row) | sx = dx·cos φ, sy = dx | from `metric`, per row (cylindrical) or per cell |
| `climate/ice.rs` `distance_to_margin` | separable; rows `sx[j]` apart across, `sy` along columns | cylindrical: column pass at the rows' cumulative true positions (done in the WIP); non-separable metric: a fast-marching or exact distance in block space scaled locally |
| `climate/snapshot.rs` marches and `true_km` | east–west along rows, ds = dx cos φ | cylindrical: unchanged; otherwise along the wind's direction from `north(x, z)` |
| `climate/orographic.rs` | FFT over row bands, wind (u east, v south) constant | cylindrical: unchanged; otherwise per patch, with the wind turned into the patch's block frame |
| `climate/belts.rs` ITCZ | cos-weighted mean over rows | area-weighted, by latitude from `forward` |
| `satellite.rs` `hillshade`, `watch/worldmap.rs` `shade_field` | dx = 2πR cos φ / w, dy = πR / h | `metric` at the pixel |
| `surface/events.rs` | great-circle distances from lat/lon | unchanged (on the sphere) |
| every W site of §3 | cos φ per row | `cell_area` |
| `deposits.js` footprints | C/360 blocks per degree on both axes | `metric` |

**Wrap and topology (T in §3).** With a pole cap, nothing changes for x-only wrap. A torus or a
folded square changes the neighbour rule in every kernel (§5.3).

## 5. What each family of projection changes

### 5.1 Another cylindrical projection (x ∝ λ, z = f(φ)): Lambert, or equirect kept

Rows stay parallels and columns meridians. What changes:

- f;
- the grid's z extent;
- the per-row metric and area;
- the map picture's aspect.

Every R site stays correct once it asks the module. This is the Lambert WIP; §6 has its numbers.
The pole cap stays a latitude, and its band narrows in blocks.

### 5.2 A square map with x-only wrap

There is no useful one: a square or tiling projection's edges are not meridians, so a translation
in x does not continue the map.

### 5.3 A square, tiling map on a torus (Peirce quincuncial, Guyou, Adams, equal-area relatives)

**Topology first.** A world that wraps in x and z by translation is a flat torus. Toroidal World
supports this natively (its torus preset; `wrapped_planet_worlds.md` §1.2). A torus cannot cover
the sphere once. By Riemann–Hurwitz (χ(torus) = 0 = 2d − Σ(e − 1)), a continuous map from a torus
onto the sphere is at least a 2-fold branched cover with at least 4 branch points. So **every place
appears at least twice in the world**.

The quincuncial family is exactly that minimum:

- Peirce's square holds the whole sphere once: north pole in the centre, south pole in the four
  corners, the equator the inscribed square.
- Its edges are glued to themselves by 180° turns about their midpoints. Those are four points on
  the equator, each a cone point of angle π.
- Two such turns compose to a translation. The translation lattice is spanned by (a, a) and
  (−a, a) for a square of side a.
- So the smallest torus is a square of side √2·a holding two copies, the second turned 180°.
  Equivalently, two Guyou 2:1 maps stacked.

A 180° turn keeps orientation, unlike the glide reflection that makes a seamless pole impossible
(POL-002). So a single copy with portal-glued edges is possible in principle, but not with Toroidal
World, which only translates.

What changes for the code:

1. **Latitude is no longer a function of the row.** Every R site becomes per cell:
   - the climate's Geometry and EBM band;
   - the ITCZ, which is by latitude, not by column;
   - the climate's row marches and the orographic bands: the winds' direction in blocks rotates
     with `north(x, z)`;
   - the per-row distance transform;
   - the zonal stand-ins (`surface/climate.rs`, `bake/biomes.rs`).

   This is the largest piece of work: the climate stage was built on rows being parallels.
2. **Metric.**
   - Peirce is conformal. Its scale s(x, z) varies across the square and goes to zero at the four
     singular points, so true-metric sites multiply by s, and the block-metric kernels' mass
     budgets need s² weights.
   - An equal-area relative keeps the budgets right but shears shapes near the singular points.
3. **One simulation, two copies.** The surface, hydrology, amplifier and bake must run on *one*
   copy (the canonical domain), with the fold as the neighbour rule. Otherwise the two copies of
   each place erode differently and the planet disagrees with itself. The world's second copy is
   then a replica:
   - `canonical(x, z)` folds every block query in the Java readers (`RasterSampler`, the column
     rule's 4 × 4 stencil, `FarColumn`, detail noise at canonical coordinates, structures,
     rivers, sites);
   - the same holds for Distant Horizons.

   Kernels with the fold as their neighbour rule:
   - `Dims::neighbour`, `box_blur`;
   - D8 routing and river polylines that cross a fold;
   - the deposits' regional windows, the amplifier's halos, `advect`;
   - flexure: solve on the double-cover torus with a 2-D FFT, whose 180°-symmetric solution is
     the folded one;
   - the orographic FFT.
4. **Places from the sphere.** Sites, events, volcanoes, craters and bodies are placed through
   `inverse`. They are stored once, in canonical coordinates, and every reader folds its query.
5. **The game.**
   - No pole cap and no beyond-pole ocean: both poles are ordinary points inside the map.
   - Toroidal World's torus preset replaces the cylinder.
   - A player walking straight crosses each place twice, the second time turned 180°.
   - The world holds 2 × 4πR² blocks of planet at true scale. At C = 16,384 (R = 2,607.6) that is
     a square torus about 13,070 blocks on a side, about twice the chunks of the Lambert cylinder
     (16,384 × 5,215) and 1.3 times today's equirectangular world (16,384 × 8,192).
6. **Maps.**
   - The workbench and the game's maps show the square (or the stacked pair).
   - Graticules are curves drawn through `forward`; the globe samples the picture through
     `inverse`.
   - The four singular points are where map features pinch.

## 6. Lambert in numbers (the cylindrical case as built in the WIP)

| | C = 16,384 | C = 32,768 |
|---|---|---|
| R, the pole | 2,607.6 | 5,215.2 |
| pole to pole | 5,215 (was 8,192) | 10,430 (was 16,384) |
| grid half-extent Zo (multiple of 64) | 2,624 | 5,248 |
| L1 r 16 / L3 r 2 | 1024 × 328 / 8192 × 2624 (were × 512 / × 4096) | r 32: 1024 × 328; r 2: 16384 × 5248 |
| cap line 75° / stop 80° | 2,519 / 2,568: a 49-block band (was 228) | 5,037 / 5,136: 98 blocks |

- **Scale.** East–west a block is sec φ of planet, north–south cos φ: 2.0 and 0.5 at 60°, 5.76 and
  0.17 at 80°. Every cell is (r · 300 m)².
- **Size and cost.** Rasters, bakes and chunks between the poles shrink by 36 %; chunks inside the
  stop line by 29 %.
- **Pole cap.** It keeps 3.4 % of the planet behind 75°, as before; on the old map that band took
  16.7 % of the area. But the storm band is crossed four to five times sooner: about 11 s at walking
  speed instead of 53 s, and a 40 blocks/s Sable jet reaches the stop line instead of turning
  inside the band. Whether to widen it, for example a cap at 70° (118 blocks), is a gameplay
  question for Ben.
- **Rows.** Rows are uniform in sin φ, so every cell is equal in area. This is the only layout that
  keeps the samplers, the ladder and the parity unchanged.

## 7. The baseline already measured (equirectangular, master 6d2e1e4c)

`deeptime stage volcanoes --preset earthlike_quick_16k --start scattered_continents --length-myr
500 --set seed=1 --threads 4` on the test VM took 7 m 44 s. The "ice line" was added to the CLI for
this; it is on the WIP branch.

- **Hypsometry, true-area weighted.**
  - land 0.268; land mean 753 m; p10 120, p50 581, p90 1,678 m;
  - above 1 km 0.269, above 2 km 0.055;
  - sea median depth 4,037 m; shelf 0.051 of the sea; continental crust 0.402, 65.9 % of it dry.
- **Cell-count land share** (the surface line): 0.255.
- **Ice.**

  | | share of the map's cells | true share of the planet |
  |---|---|---|
  | land ice | 0.084 | 0.039 |
  | sea ice | 0.249 | 0.097 |
  | either | **0.333** | **0.136** |
  | poleward of 60° | 0.334 | 0.134 (the sphere's 13.4 %) |

  On an equal-area map the map's share and the true share are the same number. That is the check
  for the after-run of whatever projection is chosen.
- **Pictures.** The satellite picture (1024 × 512) and its log are under
  `/Volumes/Graphide/ban/deeptime-runs/equal-area/before/` on the VM. The 0.75 GB checkpoint is
  deleted.

## 8. Tasks

1. **Projection-neutral refactor, equirect kept.** *Built 2026-10-02 (branch
   `worktree-agent-a3186d42ee9449d3d`): Rust `deeptime_format::projection` with the `GridDesc`
   helpers, Java `worlddata/Projection`, the pages' `projection.js` (`window.DTProj`).
   `tools/review/projection-identity.sh` shows the outputs bit-identical to master (355 files:
   synthetic worlds, the test planet's bake, keyframes, satellite picture, parity fixtures,
   deposit world).*

   *What the module holds, as built:* `lat_deg`, `lon_deg`, `z_of_lat`, `x_of_lon`, `block_of`,
   `lat_lon`, `unit` (forward and inverse); `pole_z`, `extent`, `on_planet`, `inside_poles`,
   `clamp_z`, `grid_origin_z`, `grid_rows` (the extent); `scale_ew`, `scale_ns`, `metric`,
   `area_scale`, `north`, `great_circle_blocks`, and `GridDesc::cell_area_km2`, `true_cell_m`,
   `band_weight`, `row_weights` (the metric and the areas); `wrap_col`, `clamp_row`,
   `neighbour` (the topology: `GridDesc::wrap_i`/`clamp_j`, the surface kernels'
   `Dims::neighbour` and Java's `GridDesc.wrapI`/`clampJ` call them); `index_origin`,
   `index_span` (the frame of the river tiles, the structure buckets, the site index and the
   playability regions, class S of §3); `picture_*` (map pictures, both axes).

   *Not routed yet, for whoever builds step 2 or 3:*
   - the kernels that wrap x or stop at the rows inline, about 35 Rust files (`box_blur`,
     flexure's FFT and DCT, the orographic FFT, `advect`, the amplifier's windows, the
     deposits' regional windows, the ice distance transform, the river network's outlet rows):
     the fold of §5.3 item 3 is a design pass, not a call-site change;
   - the x seam written as `c / 2` (block x wrapped into [−C/2, C/2)) at its many sites;
   - the climate's assumption that rows are parallels (§5.3 item 1): its geometry takes
     `true_cell_m` and the row weights from the module, the marches along rows stay;
   - the synthetic test terrain's analytic latitude (`synth/terrain.rs`, Java
     `AnalyticTestTerrain`: its own formula, twinned) and the CLI's Earth DEM reader
     (`climate_dem.rs`: the input file's own equirectangular layout);
   - preset data: every preset's `keyframes.size` (1024 × 512) and `keyframes::DEFAULT_SIZE`;
   - the workbench globe's texture lookup (u = longitude / 360 in its pixel loop), Java's
     `ProvinceHints` (a latitude/longitude lattice by definition);
   - review scripts: `tools/review/grid_audit.py`, `glacial_views.py`, `p2-01-report.py`, the
     `seams` and `wrap` pages;
   - the plan's §5.1–5.2 and the POL ledger rows (step 4, with the pick).

   Write the module of §2 with the equirectangular
   projection as its first implementation. Route every site of §3 through it, and add the parity
   fixture's projection block.
   - Output stays bit-identical: same bakes, same pictures, same tests.
   - It can land before Ben picks.
   - It turns every later choice into a module change plus the family-specific work below.
2. **If cylindrical (Lambert).** Finish the WIP:
   - the remaining tests;
   - the JS (`DT.proj`), the tools and the docs;
   - the keyframe size 1024 × 326 in every preset; format 0.4;
   - the climate's per-row sy (done in the WIP);
   - the evidence of §7 and the workbench screenshots;
   - the pole-cap band question to Ben.
3. **If square or torus.** Before any code, a second design pass on:
   - the per-cell climate (§5.3 item 1);
   - the fold neighbour rule in the kernels and the Java readers (item 3);
   - mass budgets on a conformal map (item 2);
   - the Toroidal World torus config;
   - the world size.

   Each is a multi-day item. The climate is the largest, and it overlaps P1-26 (per-cell climate),
   which is already planned.
4. **In every case.** The ledger:
   - SCL-049 rewritten: the projection is Ben's game choice with his reason;
   - a new SCL row for the equal-area property, or the conformal one, as a tested mathematical
     fact;
   - POL-005, -006, -014, -015;
   - BAK rows for the pictures;
   - `docs/science/world-scale-and-bake.md`, and the plan's §5.1–5.2 marked as superseded by this
     note.
