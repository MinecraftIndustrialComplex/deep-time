# Grid-artifact audit (P1-23)

Does the finished world show the grids it was computed on? Audited 2026-10-01 on one planet:
`earthlike_quick_16k`, scattered continents, 500 Myr, seed 1, master e8668ca. Circumference 16,384
blocks; L0 100,000 mesh samples (about 30 blocks apart), L1 16 blocks a cell, L2 8, L3 2.

Three kinds of statement are kept apart: **measured** (a number from the bake), **seen** (in a
picture made from the bake, at one block a pixel or coarser), **inferred** (my reading of the code
and the two). Nothing was looked at in the game: there is no client here.

## Summary

**Verdict.** The mesh and the rasters themselves do not show: no seam, no tile edge, no mesh-cell
polygons, slopes that face every direction equally. What shows is the *routing* (water runs along
eight directions) and *per-cell decisions* (a class or a rock chosen cell by cell). Worst first:

| # | What a player sees | Number (isotropic or control in brackets) | Cause | State |
|---|---|---|---|---|
| 1 | Ruler-straight dams 3–8 blocks high across lake plains, lakes with straight shores, 150 to over 1,000 blocks long, along the axes and diagonals | lake shores within 2° of an axis 7.3 % (4.4 %); straight runs of 16+ in the lake box 7–11 % (5 %) | the deep-time surface drops a lake's sediment along the one path the flow takes through it | open, design |
| 2 | Rivers in dead-straight reaches along eight directions, parallel streams | 10.2 % of river length within 0.5° of an eight-direction heading (2.2 %); 6.5 % in straight reaches of 32+ blocks, longest 274; median sinuosity 1.035 | D8 routing in the amplifier and the river network; smoothing only rounds the corners | open, design |
| 3 | Biome borders in 16-block stair steps and long east–west lines | temperature input: 24 % of its change faces exactly along an axis (4.4 %); vegetation 31 % | temperature and humidity are set per L1 cell from its Köppen class; and the known latitude bands (P1-26) | open, with P1-26 |
| 4 | Square pits, benches and cliff bands the size of an L1 cell where the rock changes | erodibility stepped 7 × more at L1 cell edges than inside (now 1.02 ×) | the erosion read the rock through a class copied into each fine cell | **fixed** |
| 5 | Coastline pieces 30–60 blocks long lying along the axes about twice as often as chance | 4.3–4.8 % within 2° of each axis below 30° latitude (2.2 %) | grows through the surface, young and amplify stages (D8 and cell-wide steps at the coast) | open, mild |
| 6 | Rock boundaries: a faint 64-block lattice in bed thickness; steps in the base of the top unit along the axes and diagonals; in flat country boundaries on L1 cell lines, saw-tooth edges | thickness changes 15 % more at the lattice's phase (controls 2–7 %); steep steps within 2° of an axis 20.6 % (4.4 %) | the smoothing's sample lattice (**fixed**); the rest not found | part fixed |
| 7 | Inland biome bands with octagonal outlines | inland a8 −0.39 (0) | a 3 × 3 chamfer distance to the sea | **fixed** |
| 8 | Faint creases every 8 blocks in terrain; fine valleys favour eight directions | curvature 19–24 % higher on L2 lines (controls under 1.3 %); spectrum a8 +0.12 to +0.14 ± 0.03 at 4–32 blocks | the amplifier's ×2 upsampling pins the coarse cells; D8 | open, low |
| 9 | A 32-block quilt in the smallest bumps | the detail noise changes 3.2 × less across lattice lines; but it is 4.6 % of the block-scale relief | value noise whose three octaves share one lattice | open, low |
| 10 | Land stretched east–west towards the poles | east–west slopes hold 0.54 of the north–south energy at 60–75° | the equirectangular world (a design fact) | noted |
| – | Ice edge and climate along latitude rows | 23 % of the ice edge in its 8 busiest rows; latitude explains 73 % of the temperature input | zonal climate | known, P1-26 |

**Clean** (measured): the antimeridian seam; tile edges (none at 16k: the amplifier ran in one
piece); slope directions overall; volcano cones; deposit sites and bodies; the Köppen, province and
surface-class maps against a random map.

**Fixed on this branch** (each with a property test that fails on the old code): rock erodibility
interpolated between L1 cells; the strata smoothing's weights tapered to zero at the window's edge;
the distance to the sea made Euclidean.

**For Ben** (options below, recommendation first): lake sediment (1), river routing (2), where the
Köppen class is decided (3), the detail noise (9).

## Method

`tools/review/grid-audit.sh` (bake, dump, analyse) and `tools/review/grid_audit.py`; the numbers are
in `review/out/grid-audit/<world>/results.json`, the pictures beside it (not committed).
`deeptime heightmap --columns` (new) gives the rock and the water the column evaluator decides.

What is tested is what the game reads: `elevation` and the detail term (bilinear, L3), the five
biome inputs and `ice_cover` (bilinear, L2), `water_level` (nearest, L3), the river polylines, the
horizon stacks through the column rule (cubic B-spline, L1), volcanoes, deposit sites and bodies.

The tests, each against what a field with no grid in it gives:

- **Orientation.** The direction θ of a field's gradient, or of a boundary's normal, folded to
  0–180°. `a4` = mean cos 4θ: 0 for an isotropic field, positive when slopes face along the block
  axes (lines run north–south and east–west), negative for the diagonals. `a8` = mean cos 8θ:
  positive when axes and diagonals are both favoured, the mark of eight-direction (D8) routing.
  "Within 2° of an axis" is 4.4 % for an isotropic field, 2.2 % for each axis alone. Fields
  stored in steps are counted only where the difference is large (two blocks for the coast),
  because a difference of one step has only three directions.
- **Phase.** The mean |first difference| (d1) and |second difference| (d2) of a field by the
  sample's place inside a coarser cell (index mod p). No imprint: every place has the same mean;
  the figure is max/min − 1. Periods that are no grid (3, 5, 7) are the control.
- **Spectrum.** `a4` and `a8` of the power spectrum by wavelength band, over 1,024-block land
  tiles, less what the band's own modes give with equal power. Only bands up to 128 blocks have
  enough modes to mean anything.
- **Outliers.** Every column's and row's mean curvature against its neighbours' median, as a
  robust z-score: a seam or a tile edge is a column that stands out.
- **Straight runs** (class maps). The share of boundary edges in straight axis-aligned runs of
  8 and 16 edges, beside a random isotropic map of the same boundary density (the control) and
  the same map copied 4 × 4 (what a nearest lookup gives: 72 % in runs of 8).
- **Stages.** The same tests on the terrain after the plates (L1), the deep-time surface (L1),
  the young landscape (L2) and the amplifier (L3), from the run's checkpoints.

Limits of the method: one planet, one seed; the orientation estimator for class maps is itself
biased by the raster (its control reads a4 +0.23 to +0.30), so class maps are judged against the
control, not against 0; pictures are hill-shaded arrays, not the game.

## Findings

### 1. Dams and straight lake shores on plains (open)

- **Seen.** `boxes/lake-terrain.png`, `boxes/lake-water.png` (box x 4608…5120, z −512…0): a
  diagonal ridge over 350 blocks long, east–west ridges 150–180 blocks long, a triangular lake
  whose shore is that diagonal, lakes that are east–west strips. `overview/terrain.png`: the same
  diagonal runs over 1,000 blocks across the eastern continent's plain.
- **Measured.** The lines stand 3–8 blocks over the plain (terrain less its 66-block median: 95th
  percentile 3.3, 99th 7.6 blocks). Lake shores within 2° of an axis: 7.3 % (4.4 %); a8 +0.10.
  Lake and river columns of the box in straight runs of 16+: 7.1 % (control 4.6 %); 11.2 % in the
  biome box. No shore is cut by the `water_level` mask (0 of 45,924 shore pairs): the shores are
  the terrain's.
- **Stage** (seen, `probe/lake-*.png`): the lines are already in the deep-time surface (L1) and
  are lines of zero depth across the young landscape's lake.
- **Cause** (inferred from `surface/spl.rs`, the flux pass): a lake cell takes sediment only from
  the flow routed *through it*, up to the lake level. The flow crosses a lake along the flood's
  own tree, which runs straight along D8 lines, so the trunk path fills to the lake level first
  and becomes a causeway, and the rest of the lake stays deep.
- **Reach.** The young landscape has 34,988 lake cells, 6 % of the land.

### 2. Straight rivers (open)

- **Measured** (the river file, 163,552 blocks of river in 2,369 chains).
  - Heading within 0.5° of an eight-direction heading: 10.2 % of the length per segment (2.2 %),
    5.5 % over 16 segments (about 35 blocks), 3.2 % over 64 segments (about 140 blocks).
    a8 +0.19, +0.15, +0.07.
  - In straight reaches (heading within 3°) of 16+ blocks: 19.8 %; 32+: 6.5 %; 64+: 1.4 %; 128+:
    0.4 %. Longest 274 blocks.
  - Sinuosity over 128 blocks of path: 10th percentile 1.010, median 1.035, 90th 1.112; 27 %
    below 1.02. Real rivers are commonly 1.1–1.5 and over (from memory, not sourced).
- **Seen.** `boxes/river-water.png`: a wide river straight at 45° for 270 blocks across a coastal
  plain, two tributaries parallel beside it.
- **Cause** (inferred): the amplifier and `hydro::network` route by D8 on the L3 grid; the
  network's Gaussian smoothing (8 blocks) rounds the 45° corners and keeps a straight run
  straight. STATUS already left "straight parallel streams on even slopes" for this task.

### 3. Biome borders follow L1 cells (open)

- **Measured** (the five L2 inputs, land below 75°).

| input | within 2° of an axis (4.4 %) | N–S lines (2.2 %) | E–W lines (2.2 %) | a8 | latitude R² |
|---|---|---|---|---|---|
| temperature | 24.3 % | 9.3 % | 15.0 % | +0.42 | 0.73 |
| vegetation | 31.5 % | 11.5 % | 20.0 % | +0.41 | 0.36 |
| continents (before the fix) | 15.9 % | 5.6 % | 10.3 % | −0.09 | 0.21 |
| erosion | 7.3 % | 3.2 % | 4.1 % | +0.09 | 0.12 |
| ridges | 12.3 % | 4.5 % | 7.8 % | +0.24 | 0.12 |

- **Seen.** `boxes/biome-temperature.png`: plateaus of one value with borders in 16-block steps,
  some straight for 150 blocks.
- **Cause** (read in `bake/biomes.rs`): the temperature and humidity band coordinates are computed
  per L1 cell from that cell's Köppen class, each class with its own ramp, then interpolated to
  L2. The class map is a staircase of L1 cells, so the inputs step where it steps; bilinear
  interpolation blends a step over 16 blocks and keeps its shape. The east–west excess over
  north–south is the latitude banding of the climate (P1-26). The N–S figure (9–12 % against
  2.2 %) is the staircase alone. The `ridges` sign rules are per class, nearest (12 % more of its
  change falls on L1 cell edges).
- The erosion input's square window (a 9 × 9 cell range, then a box mean) shows little: 7.3 %.

### 4. Cell-shaped pits and benches where the rock changes (fixed)

- **Seen.** `boxes/seam-terrain.png` (box x −8448…−7936, z 1154…1666): rectangles and L shapes
  16–40 blocks across, a few blocks high (terrain less its median: 1st percentile −4.3 blocks,
  99th +3.0), exactly where `probe/seam-3-young-class.png` shows the surface rock changing from
  L1 cell to L1 cell.
- **Cause** (read in `surface/young.rs`): `young/class` is L1's class copied into each L2 cell,
  and the rivers' K, the hillslopes' κ and `young/erodibility` (which the amplifier's incision and
  the mountain character's cliff bands read) were looked up through it: constant over 16 × 16
  blocks, stepping at the cells' edges.
- **Fix.** `young::rock_property`: the L1 cells' erodibility and diffusivity interpolated linearly
  to L2, as every other L1 field the stage takes is; till laid by the stage's own glaciers keeps
  its own value. The class is unchanged.
- **Measured, before → after.** `young/erodibility` on land: its steps were 7.0 × larger across
  an L1 cell's edge than inside the cell (d1 excess 603 %); after, 1.8 % (control periods 1–6 %).
- **Seen, before → after** (`world/boxes/seam-terrain.png`, `world-after/boxes/seam-terrain.png`):
  the crisp rectangles become faint, rounded benches. They do not vanish: the rock still changes
  from one L1 cell to the next, now over 16 blocks instead of at a line. The box's relief
  figures are unchanged (1st percentile −4.3, 99th +2.9 blocks), so I have no terrain number for
  this fix, only the picture.

### 5. Coast (open, mild)

- **Measured.** The coast's normal over 32-block chords, within 2° of each axis (2.2 %): 4.3 %
  N–S and 4.8 % E–W below 30° latitude; 3.4 % and 6.3 % at 30–60°; 2.2 % and 9.0 % at 60–75°
  (there the projection's stretch adds east–west coast). a4 +0.13, +0.14, +0.23. Over 4- and
  8-block chords a4 is 0.00–0.01; over 16 blocks +0.10.
- The land/sea map has **no** cell staircase: 12.6 % of its edges in runs of 8+ (control 14.8 %,
  blocky control 72 %).
- **Stage.** Coast a4 and a8: plates +0.04/+0.04, surface +0.06/+0.07, young +0.11/+0.13.
- Not visible as steps in `boxes/coast-terrain.png` or `boxes/river-terrain.png` (seen).

### 6. Rock (part fixed)

- **Planet-wide, every 8th block** (measured): the rock at the surface and 8, 32 and 96 blocks
  down has fewer straight runs than a random map: 0.8 %, 1.1 %, 2.8 %, 2.4 % of boundary edges in
  runs of 8+ (controls 9–13 %). No polygons, no cell staircase.
- **The smoothing lattice** (measured, fixed). The bake smooths the stacks over a lattice of
  every 4th L1 cell (64 blocks) with a Gaussian cut off at 2σ, where it still holds 0.135 of its
  peak. As a cell moved by one, a row of samples dropped out at full weight: the samples' mean
  position swung ±0.23 cells round the cell (±0.68 at σ 20). The top unit's thickness changed
  15 % more at one phase of the 64-block period (d2 16 %; controls 2–7 %). **Fix:** weights that
  reach zero at the window's edge (`bake_horizons::smooth_axis_weight`). After (with all three fixes): 7 % (d2 10 %;
  controls 2–6 %). At twice the period 21 % → 18 %, so something else of that size remains.
- **Steps along the axes** (measured, open). Where the base of the top unit drops 30 blocks or
  more within 16, its slope faces within 2° of an axis in 20.6 % of cases (4.4 %), a8 +0.52. Not
  changed by the fix. Cause not found.
- **Flat country** (measured in one box, seen in `boxes/plain-rock_top.png`, open): rock changes
  fall on one place of the 16-block L1 cell 4–8 times as often as the mean; a unit's edge runs
  east–west for 350 blocks; another edge is a saw of 16-block teeth. In the other ten boxes the
  ratio is 1.2–2.5. Not changed by the fix. Cause not found; candidates are the column rule's
  sliver step and the per-cell winner of each key interval in the smoothing.
- **Seen.** `boxes/biome-rock_32.png`: comb edges and a unit with a square notch.

### 7. Continentalness (fixed)

- **Cause.** `bake::biomes::distance_cells` was a two-pass 3 × 3 chamfer: lines of equal distance
  are octagons, 8 % out at 22.5°.
- **Fix.** The exact Euclidean distance (two separable passes).
- **Measured**, beyond 96 blocks from the sea: a8 −0.39 → 0.00 (0 is isotropic; −1 is a slope
  always 22.5° off the axes).
- Near the coast the input still follows the L2 land mask's cells (16.9 % within 2° of an axis):
  the distance is counted from cell centres, not from the shoreline inside a cell. Left.

### 8. Terrain at the fine scale (open, low)

- **Slopes overall** (measured, land below 75°): a4 +0.017, within 2° of an axis 4.8 % (4.4 %).
  By latitude: a4 +0.001 at 0–30°, +0.023 at 30–60°.
- **Creases.** Curvature (d2) is higher on every 4th L3 cell, the L2 points: +18.6 % (x), +15.7 %
  (z); +21 % on every 8th (L1); steps (d1) +2.6 % to +4.5 %. Controls 0.4–1.2 %. In the young
  landscape the same test reads +6 % to +8 %; in the surface stage +1 % to +2 %.
- **Cause** (inferred from `amplify/resample.rs`): each ×2 level keeps the coarse cells' values
  and puts clamped Catmull–Rom values between them; the clamp flattens extremes at the coarse
  points.
- **D8.** Spectrum a8 +0.13, +0.12, +0.14 (± 0.02–0.04) at wavelengths 4–8, 8–16, 16–32 blocks;
  0.02 at 32–128. a4 +0.06 to +0.11 (± 0.03–0.05) from 4 to 128 blocks.
- **Seen.** Nothing grid-like in `boxes/mountain-terrain.png` beyond a faint block pattern in one
  corner.

### 9. The detail noise (open, low)

- **Measured.** The noise N changes 3.2 × less (d1 +218 %) across the lines of its 32-block
  lattice than between them; +84 % at 16 blocks, +32 % at 8. Its slopes: a4 +0.10.
- **Cause** (read in `deeptime-format/src/detail.rs`): value noise with a smoothstep fade has no
  slope across its lattice lines, and the three octaves (32, 16, 8 blocks) share one origin, so
  the lines of all three coincide every 32 blocks.
- **Size.** A·N on land: median 0.1 block, 90th percentile 0.6, 99th 1.7. Where A is a block or
  more (35 % of the land) the step over 2 blocks is 0.09 block from the detail against 2.03 from
  the elevation: 4.6 %.

### 10. High latitudes (noted)

- **Measured.** East–west slope energy over north–south: 1.07 at 0–30°, 1.03 at 30–60°, 0.54 at
  60–75°, 0.24 beyond 75°. Landforms are stretched east–west towards the pole cap.
- **Cause.** The plates live on a sphere and the world is its equirectangular map: at 67° a
  round feature is 2.6 × wider than tall in blocks. The surface stages run in block metric and
  take part of it out. This is the projection, not a grid.
- Rows near the pole stand out in the outlier scan (z up to 23 on L1 rows), all beyond 79°,
  outside the 75° cap and the hard stop at 80°.

### Known: climate bands (P1-26)

- Ice cover: 23 % of its edge cells lie in the 8 busiest rows; 23 % of its edge in straight runs
  of 8+ (control 15 %).
- Latitude alone explains 86 % of the mean temperature and 62 % of the precipitation on land.

### Clean

| Looked for | Result (measured) |
|---|---|
| Seam at the antimeridian | elevation step across it 0.95 block, interior columns 1.07; z-score of its two columns 2.3 and −0.4; the picture `boxes/seam-terrain.png` shows nothing at x = −8192 |
| Tile edges | none to find: at 16k the amplifier runs in one piece (`mode=in_memory tiles=1`). **Not tested** for the streamed 32k presets |
| Mesh-cell polygons | none in rock, Köppen, provinces or surface class against the random control |
| Volcano cones | summits are not on any grid; on a ring at half the cone's radius harmonics 4 and 8 hold 2.0 % and 0.3 % of the variance round the cone (even share 8.3 %), 82 cones |
| Deposits | site centres on the 2-, 8-, 16-block grids: 32 %, 3.2 %, 0.9 % (chance 25 %, 1.6 %, 0.4 %; 218 sites); body centres not on any (1,035 bodies) |
| Lake shores against the mask | 0 of 45,924 shore pairs cut by the `water_level` cells |

## What was fixed

Branch `worktree-agent-a851cf286456a3901`. Each fix has a property test that fails on the old code.

| Fix | File | Test |
|---|---|---|
| Rock erodibility and diffusivity interpolated between L1 cells | `surface/young.rs` (`rock_property`) | `rock_properties_are_interpolated_between_l1_cells` |
| Strata smoothing weights reach zero at the window's edge | `structure/bake_horizons.rs` (`smooth_axis_weight`) | `the_smoothing_lattice_leaves_no_period`, `a_tilted_bed_stays_a_plane_under_smoothing` |
| Distance to the sea is Euclidean | `bake/biomes.rs` (`distance_cells`) | `distance_to_the_sea_is_euclidean` |

Side effects: the strata smoothing's effective width is 0.85 σ (it was 0.89 σ); terrain changes
wherever the rock changes between L1 cells. The planet's totals hardly move (measured, the same
run before → after): land 27.7 % → 27.7 %, median land height 715 → 715 m, highest point
6,609 → 6,614 m, lakes 997 → 1,025, river chains 2,369 → 2,345. The fixes do not change findings
1, 2, 3, 5, 8, 9: rivers within 0.5° of an eight-direction heading 10.2 % → 10.2 %, elevation
curvature on L2 lines +18.6 % → +17.5 %.

## What is left, with options

### A. Lake sediment (finding 1)

1. **Fill each lake level, deepest first (recommended).** Pool what a lake's cells trap in a
   step and lay it from the deepest cell up to one common level; the surplus leaves at the
   outlet as now. Mass stays conserved. Touches `spl.rs`'s flux pass (which is parallel and
   bit-identical to its serial loop), the strata log's lake share, and both stages that use the
   solver. About two to three days with the tests.
2. **Deposit as a delta.** Spread the inflow from each inlet over the lake cells within a reach
   of it. More faithful, more work, and the reach is a new setting.
3. **Hide it.** Smooth lake floors in the amplifier. Cheap; the dams stay in the strata and in
   the L1 history.

### B. Rivers and valleys along eight directions (findings 2, 8)

1. **Route with a direction that is not bound to eight (recommended).** In the amplifier and in
   `hydro::network`, choose each cell's receiver between the two neighbours that bracket the
   true downslope direction, by a hash of the cell in proportion to the angle (Fairfield and
   Leymarie's ρ8), so a path's mean heading is the slope's. Deterministic. Changes every valley
   at L3 and the river network; the area prior and the flood stay. About three days; needs a
   look at the result before it lands.
2. **Meander the river lines only.** Displace the smoothed polylines sideways by a noise of a
   few channel widths. A day. Rivers stop being ruled lines; the valleys they sit in stay
   straight, so a river can leave its valley floor.
3. **Leave it** for the Phase 2 river detail (meanders, floodplains).

### C. Where the Köppen class is decided (finding 3)

1. **Decide the class on L2, in P1-26 (recommended).** Interpolate the continuous climate
   (monthly temperature and precipitation) to L2 and classify there, so a border is a smooth
   line; P1-26 is rewriting this climate now, and doing it first would be done twice.
2. **Jitter now.** When the L2 cell reads its L1 band coordinates, move the sampling point by a
   smooth noise of about one L1 cell. Half a day; breaks the stair steps and the straight lines
   into wobbles; the class's hard rules (`ridges` sign, erosion limits) need the same.
3. **Leave it.** Vanilla's own biome blur (4-block cells) softens a 16-block step a little.

### D. The detail noise (finding 9)

1. **Give each octave its own lattice offset (recommended, when the format next changes).** Shift
   octaves 2 and 3 by a fraction of their cell, so the flat lines no longer coincide. A few lines
   in `detail.rs` and its Java twin, new pinned values on both sides.
2. **Gradient noise.** No flat lines at all; a larger change to both sides.
3. **Leave it.** It is 5 % of the block-scale relief.

### E. Smaller

- Coast distance from the shoreline inside a cell, not from cell centres (finding 7).
- The steps in rock boundaries along the axes and the L1-line boundaries in flat country
  (finding 6): find the cause first.
- The creases on L2 lines (finding 8): a monotone cubic in the amplifier's upsampling would
  remove the kink at the coarse points; it changes all terrain a little.
- Two other chamfer distances (`surface/fields.rs`, craton distance; `surface/volcanoes`, the arc
  front): not measured, not read by the game directly.

## Not verified

- Nothing was seen in the game. Vanilla's biome blur, surface rules, trees and the 4-block noise
  cells may hide or sharpen what the arrays show.
- One planet, one seed, one preset. The streamed amplifier (32k) and its tile edges were not run.
- Sea ice, MIC's biome mapping and Distant Horizons' far terrain were not looked at.
- The cause of finding 1 is read from the code and the stage pictures, not proven by a run with
  the lake rule changed. The causes in findings 5 and 8 are inferred.
- Earth figures for river sinuosity are from memory.

## Reproduce

```text
tools/remote/mac.sh -- tools/review/grid-audit.sh bake                    # about 6 min
tools/remote/mac.sh -- tools/review/grid-audit.sh dump
tools/remote/mac.sh -- tools/review/grid-audit.sh analyse                  # the planet-wide tests
tools/remote/mac.sh --no-sync -- tools/review/grid-audit.sh analyse pick   # overview, boxes.json
tools/remote/mac.sh --no-sync -- tools/review/grid-audit.sh boxes
tools/remote/mac.sh --no-sync --pull review/out/grid-audit -- tools/review/grid-audit.sh analyse boxes
tools/remote/mac.sh --no-sync -- tools/review/grid-audit.sh clean
```

`GRID_AUDIT_WORLD=<name>` measures a second bake beside the first; `GRID_PROBE=name:x0,z0,x1,z1`
with `analyse probe` draws one box through the stages.
