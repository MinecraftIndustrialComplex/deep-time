# Seamless wrapping: a quincuncial world joined by Immersive Portals (prototype)

**Status (2026-10-03).** A prototype on branch `worktree-agent-afec00aa80f7be0a6`, asked for by Ben
(decisions log 2026-10-03: "Send out an agent to begin prototyping implementations for a pierce
quincuil projection w/ 100% seamless wrappings. For now prototype it w/ immersive portals"), widened
the same day to compare the Peirce and Guyou aspects. Lambert's cylindrical equal-area projection is
still the shipped one; nothing here changes the mod, the core, the CLI or the MIC pack.

Reading before this: [`world-projection-options.md`](../research/notes/world-projection-options.md)
(the projections), [`rotated-seam-worlds-prior-art.md`](../research/notes/rotated-seam-worlds-prior-art.md)
(nobody has built this), [`immersive-portals-pillowcase-prototype.md`](../research/notes/immersive-portals-pillowcase-prototype.md)
(the first portal pillowcase, on a flat test world and with Lambert terrain), and
[`equal-area-projection.md`](equal-area-projection.md) (the projection-neutral layer).

**Tags.** [R] run and seen (a log line, an RCON readout, a screenshot); [C] computed by the
prototype's tool or tests; [I] inference; [U] not verified.

SUMMARY_PLACEHOLDER

---

## 1. What was built

- **The maths, once, with the aspect as a parameter** (`crates/deeptime-quincuncial`, a dev-only
  crate that nothing else depends on; it folds into `deeptime_format::projection` if Ben picks it):
  - `elliptic`: Jacobi sn, cn, dn (real by the AGM, complex by the addition formulas), K(m), Carlson's
    R_F for complex arguments, and arccn. No dependency.
  - `peirce`: the conformal square. Block (x, z) of a square of side a maps to w = K + (2K/a)(x − i z);
    the planet point's stereographic coordinate is ζ = cn(w | ½). Forward, inverse, linear scale,
    singular points, the fold into the square.
  - `world`: the two aspects as Minecraft worlds. **Peirce** keeps the projection's pole at the north
    pole. **Guyou** turns the sphere so the projection's pole is a point of the equator (the western
    hemisphere's centre) and turns the block plane 45°: the same function, a different rotation of
    the sphere and of the plane. Wikipedia's Guyou page (read 2026-10-03) agrees: "an oblique aspect
    of the Peirce quincuncial projection", "conformal except for the four corners of each
    hemisphere's square", "The Equator is represented by a horizontal line", 2:1.
  - `glue`: the gluing table and each seam's Immersive Portals portal (NBT and `/summon` line).
  - `resample`: a baked world-data folder resampled onto either layout, readable by today's Java
    readers unchanged (§3).
  - `stats`: the measurements of §2. CLI: `deeptime-quincuncial stats|portals|place|resample`.
- **22 tests** (property tests over seeded random points, no new crate): every sphere point maps into
  the domain and back (to 1e-9 rad, 30–40k points per aspect, plus both poles, the singular points and
  the edge meridians); every block maps to the sphere and back; each glued pair of edge points is one
  planet point (to 1e-10 rad), and so is every point within 100 blocks off an edge and its glued image
  (the map continues smoothly through the seam, not only continuously); a mirror seam would *not*
  match (the turn is a turn); conformal (singular values equal to 1e-5) and not mirrored; the scale
  matches the analytic one; exactly four zeros of the scale, at the predicted places, with scale ∝
  distance there; the area distortion integrates to the mean; Guyou's equator is the straight line
  z = 0, north is −z, east +x, the poles sit at the long edges' quarter points, the singular points at
  ±45°; a resampled test world matches itself across every seam.
- **The prototype run** (`tools/proto/quincuncial/`): `proto.py` (setup, boot, portals, terrain,
  entities, physics, client), `notw_jar.py` (a TW-free copy of the Deep Time jar for the dev run only),
  `render.py` (maps). It runs on the test VM under `tools/remote/mac.sh --server`, with the copies of
  Immersive Portals – CE 6.0.10 and Cloth Config that the 2026-10-02 prototype downloaded. Nothing is
  redistributed or committed.

**Licences.** Immersive Portals – CE is Apache-2.0 (its `LICENSE`, read on the VM 2026-10-03), so not
GPL, and it is used only at run time in the dev run; no code was copied from it. Downloading it from its
GitHub release or Modrinth needs no acceptance. Distant Horizons is not used by the run.

## 2. Peirce or Guyou: measured

Both are the same conformal map (the same p2 tiling of the plane), cut differently and turned
differently on the sphere. The measurements are for the planet of the 16k presets drawn at about the
same scale: Guyou with hemisphere squares of b = 8,192 (a 16,384 × 8,192 world, today's 16k
footprint), Peirce with a square of side a = b√2 ≈ 11,585 (the same planet, the same block count).
`deeptime-quincuncial stats`, 1,600-cell grids [C].

| | Peirce (pole in the centre) | Guyou (equator straight) |
|---|---|---|
| World shape | square, 11,584 × 11,584 | 2:1 rectangle, 16,384 × 8,192 |
| Poles | north: the centre; south: the four corners (one point) | on the long edges, a quarter of the way along (each pole is on a seam) |
| Equator | the diamond through the four edge midpoints | the straight line z = 0 |
| Singular points | 4, on the equator, 90° of longitude apart: the edge midpoints | 4, at ±45° latitude on the two meridians 90° from the hemisphere centres: the long edges' midpoints and the rectangle's corners |
| Blocks per degree, smallest | 38.6 at both poles | 38.6 at the two hemisphere centres (equator) |
| Blocks per degree at the poles | 38.6 | 54.5 |
| Blocks per degree on the equator | 54.5 (between singular points) rising without bound at them; median 64.8 | 38.6 to 54.5, median 44.5; no singular point |
| Around a singular point | 1° of planet away: 292 blocks per degree, 584 blocks from the point; 5° away: 131, 1,306 blocks away | same |
| Area scale, over the sphere | blocks per planet area / mean: min 0.457, 10 % 0.51, 50 % 0.79, 90 % 1.50, 99 % 4.6, 99.9 % 14.5; ∞ at the singular points; 91 % of the planet within 2× of the median | identical (same map) |
| … by latitude (planet-area-weighted mean, bands 0–15°, 15–30°, …, 75–90°) | 1.77, 0.98, 0.72, 0.57, 0.50, 0.47: the tropics drawn largest, the poles smallest | 0.66, 0.79, 1.31, 1.48, 1.02, 0.93: the mid-latitudes drawn largest, the tropics smallest |
| Seams | 4, every one a half-turn about its midpoint | 2 half-turns (the long edges) and 2 slides (the short edges, a plain translation like today's x wrap) |
| Seam length needing a portal | 46,336 blocks, all of it turning | 49,152 blocks with portals; only 32,768 of it turns, and the 16,384 of slide could be done by Toroidal World's ordinary cylinder if TW and Immersive Portals could share a world |
| An east–west walk along the equator | 4 legs of 8,191 blocks (32,765 in all, twice Guyou's), a 90° turn at each singular point, which also sits on a seam: no straight walk follows it | one straight line of 16,384 blocks, crossing one slide seam at a regular point; scale between 38.6 and 54.5 blocks per degree |
| A north–south walk over a pole | the north pole is an ordinary interior point; reaching the south pole means walking into a corner, where two turning seams meet | a straight walk north along a hemisphere's centre meridian reaches the pole on the long edge, crosses the half-turn seam and comes out heading south on the other side, exactly as over a real pole |
| Singular points on the test planet (`place`, 3° discs, 1° steps) | all four at sea for 188 of 360 longitudes, all four 3° discs dry-free for 52 | 216 of 360, and 42 |
| Fits Deep Time's grids | no: a square that must fold in both axes, with turns | yes: the 2:1 grid, x wrap and every reader stay; only the two long edges fold, with a turn |
| Distant Horizons | four turning edges to fold in `FarTerrain` | x wraps as today; two edges to fold |
| Workbench map | a square picture; north radial | the familiar 2:1 picture, equator across the middle, north up along the equator |

What lies at the singular points on a typical planet: nothing in particular, because Deep Time's
planets are random. The test planet (earthlike_quick_16k, scattered continents, 500 Myr, seed 1, land
0.174 of the planet) lets either aspect put all four points in open sea, with dry-free discs of 3°
(several hundred blocks of world) round each [C]. On an Earth-like planet the difference would favour
Peirce: its points are equatorial, where Earth has less land than at 45° [I]. The bake can search the
longitude, as `place` does.

Neither map is equal-area. Ben chose equal area for today's world ("Use an equal area projects FFS",
2026-10-02). The tiling, the seams and the portals are the same for the equal-area relatives with this
layout (Gringorten, the Collignon quincuncial, Petroff 2021; world-projection-options.md §2.2): if Ben
wants equal area and seamless, only `peirce.rs` changes, not the gluing.

**Recommendation: Guyou.** Same map, same singularities, same spread of area, but:
1. it is Deep Time's own 2:1 grid with today's x wrap; only the two long edges are new, and they turn;
2. the equator is a straight east–west walk and north is up along it, as now;
3. walking over a pole works like the real thing (a half-turn at the long edge);
4. its singular points sit at 45° latitude, away from the seams' corners where two seams meet on
   Peirce's south pole;
5. Distant Horizons, the workbench map and the climate's "rows are parallels" assumption are less
   disturbed (rows are parallels only near the equator, but the picture stays 2:1).
Against it: the poles are on seams (a portal at the pole), and its singular points are at
mid-latitudes, where Earth-like land is commoner. The runnable world was built in Guyou.

## 3. How the world is sampled

- **Blocks per degree** (b = 8,192): 38.6 at the hemisphere centres, 44.5 on the equator's median
  point, 54.5 at the poles and where the equator crosses the hemisphere boundary, 49 at 60° on a
  centre meridian; around each singular point it grows without bound, as 1/r: 131 at 5° from it (1,306
  blocks away), 292 at 1° (584 blocks away). For comparison the 16k Lambert world draws 45.5 blocks per
  degree east–west at the equator. The same numbers hold for Peirce, at different places (§2).
- **At the four singular points** the map doubles angles: a small circle round the point in the world
  (a cone of total angle 180°: the half-disc on one side of the seam, closed by the portal's turn)
  covers a full circle of planet. The planet there is magnified without bound: a source cell is drawn
  over hundreds of blocks, so terrain there is smooth and stretched, and a coastline through the point
  bends. Placing the four points in open sea hides this.
- **Resampling from the simulation's grid** (`resample.rs`): every output raster keeps its name, dtype
  and cell size r (L1 16, L2 8, L3 2 blocks). Each output cell is a point sample at its block; the block
  goes to the planet through the forward map and to the source's block through the source's own
  projection module (read-only), and is sampled there: bilinearly for continuous fields; the weirdness
  input as an odd field, like Java's `bilinearOdd`; the nearest source cell for categories, rock stacks,
  lake surfaces and ids. The elevation takes the source's detail term A·N at the source point, so
  the sub-cell texture is the planet's own and agrees across seams. Where the output is coarser than the
  source (the 0.46 end of the area scale) this point sampling aliases; a production bake would filter
  by each cell's footprint.
- **The strip.** The output is a strip of circumference C′ = 4b = 32,768 (the 32k preset's): x wraps
  with C′ (two copies of the slide period), and z runs over ±C′/4 = ±b, twice the domain's ±b/2. The
  forward map is defined on the whole plane, so everything outside the domain is the glued neighbour's
  terrain, the "folded outside". Today's latitude-from-z code (the pole cap, `BeyondPoleDensity`) reads
  at most 45° inside the domain and never fires there.
- **What the resampler drops**: rivers, folds and faults, volcanoes, craters, deposit sites, bodies
  and events, keyframes (block-space vector data the prototype does not transform). The spawn
  candidates are carried through latitude and longitude. A 60 s resample of the test planet on 8
  threads; 316 MB of world data (165 MB at the source) [R].

## 4. The gluing table for a 16k world

Every seam is a one-way global portal on the edge, facing into the world, 1,200 blocks tall centred on
y = 64 (it covers −536 to 664). A half-turn portal's destination is its own centre, with Immersive
Portals' rotation NBT `rotationA:0f, rotationB:0f, rotationC:1f, rotationD:0f` (a half-turn about +y);
a slide portal's destination is its centre moved by the slide. For blocks (cells, not points) the
half-turn about the integer point (cx, cz) maps block (bx, bz) to (2cx − bx − 1, 2cz − bz − 1).

**Guyou, b = 8,192** (the prototype): the world is x ∈ [−8192, 8192], z ∈ [−4096, 4096]; 4 portals.

| Seam | Edge | Portal centre (x, y, z) | axisW | Width | Destination | Transform |
|---|---|---|---|---|---|---|
| east | x = 8192 | (8192, 64, 0) | (0, 0, 1) | 8,192 | (−8192, 64, 0) | (x, z) → (x − 16384, z) |
| west | x = −8192 | (−8192, 64, 0) | (0, 0, −1) | 8,192 | (8192, 64, 0) | (x, z) → (x + 16384, z) |
| south | z = 4096 | (0, 64, 4096) | (−1, 0, 0) | 16,384 | (0, 64, 4096), turned 180° | (x, z) → (−x, 8192 − z), yaw + 180° |
| north | z = −4096 | (0, 64, −4096) | (1, 0, 0) | 16,384 | (0, 64, −4096), turned 180° | (x, z) → (−x, −8192 − z), yaw + 180° |

**Peirce, a = 11,584** (the same planet; 724 chunks a side): x, z ∈ [−5792, 5792]; 4 portals, all turning.

| Seam | Edge | Portal centre | axisW | Width | Transform (destination = centre, turned 180°) |
|---|---|---|---|---|---|
| east | x = 5792 | (5792, 64, 0) | (0, 0, 1) | 11,584 | (x, z) → (11584 − x, −z) |
| south | z = 5792 | (0, 64, 5792) | (−1, 0, 0) | 11,584 | (x, z) → (−x, 11584 − z) |
| west | x = −5792 | (−5792, 64, 0) | (0, 0, −1) | 11,584 | (x, z) → (−11584 − x, −z) |
| north | z = −5792 | (0, 64, −5792) | (1, 0, 0) | 11,584 | (x, z) → (−x, −11584 − z) |

`deeptime-quincuncial portals --layout guyou|peirce --size N --lon L` prints the table and the
`/summon` lines for any size.

## 5. The prototype run

**Set-up** [R]. NeoForge 21.1.249 dedicated server on the test VM with three mods: the Deep Time jar
built from this branch and patched by `notw_jar.py` (Toroidal World optional, `WorldDimensionsShapeMixin`
removed; TW and Immersive Portals cannot share a world, 2026-10-02 note §5), Immersive Portals – CE
6.0.10 and Cloth Config. Level type `deeptime:planet_32k` (C = 32,768, the strip), world data
`guyou-world`: the earthlike_quick_16k preset, scattered continents, 500 Myr, seed 1, baked by this
branch's `deeptime run` (land 0.174 of the planet), resampled onto Guyou b = 8,192, λc = 31°. Deep Time
bound it with no change ("Bound Deep Time world data … (format 0.3, C = 32768)"), built its climate
mapping, chose a spawn from the carried-over candidates and generated terrain. The four portals were
summoned over RCON from `deeptime-quincuncial portals`, converted to global portals, and the server was
restarted (a fresh global portal moves entities only near its centre until then). The client is a real
NeoForge client under Xvfb with Mesa llvmpipe (portablemc, offline profile), driven with xdotool;
positions come over RCON every 0.25 s.

**Maps** (`review/out/quincuncial/`, not committed): `src-equirect.png` (the bake),
`guyou-domain.png` (the world: the rectangle, equator across the middle, poles and singular points
marked), `guyou-strip.png` (the whole strip: the domain outlined, its copies beyond every edge turned
or slid as the seams say).

### 5.1 Terrain is continuous across every seam [R]

`proto.py terrain`: 48 points along each seam. At each, the terrain height the chunk generator uses
(`/deeptime probe column`, `terrainH`) at the block one step inside (A), the block one step outside (O,
hidden by the portal), and O's glued image G (inside, on the far side).

| Seam | Kind | H(O) − H(G): outside vs its glued image | per-block step A→G across the seam: median / p90 / max | per-block step between ordinary neighbours: median / p90 / max |
|---|---|---|---|---|
| east | slide | 0 at all 48 | 0.06 / 0.35 / 1.06 | 0.06 / 0.38 / 1.06 |
| west | slide | 0 at all 48 | 0.06 / 0.35 / 1.06 | 0.06 / 0.36 / 1.05 |
| south | half-turn | 0 at all 48 | 0.06 / 0.59 / 1.03 | 0.06 / 0.50 / 1.16 |
| north | half-turn | 0 at all 48 | 0.05 / 0.30 / 1.00 | 0.05 / 0.28 / 1.31 |

The folded outside is exactly the glued inside, so what a portal shows is what lies beyond it, and the
step across a seam is drawn from the same distribution as the step between any two neighbouring
blocks: there is no seam in the terrain. (Block for block, the generated column next to a half-turn
seam is the terrain one block further on, because a half-turn about a block corner maps a block's
sample point to its neighbour's corner; that is a one-block offset, within the slope of the step.)
Caves, ores and decoration are seeded by block position and chunk, not by planet point, so they do
not continue across a seam [I].

### 5.2 Entities cross every seam and land at the matching point [R]

`proto.py entities`: an item, a minecart and an arrow (no gravity, 0.8 blocks a tick outward) sent at
each seam from 3 blocks inside.

| Seam | Item | Minecart | Arrow |
|---|---|---|---|
| east (slide) | crossed: −8159.9, 701.0 | crossed: −8184.0, 701.0 | crossed: −8159.9, 701.0 |
| west (slide) | crossed: 8160.0, −700.0 | crossed: 8184.0, −700.0 | crossed: 8159.8, −700.0 |
| south (half-turn) | crossed: 1200.0, 4064.0 | crossed: 1200.0, 4088.1 | crossed: 1200.0, 4064.0 |
| north (half-turn) | crossed: −1201.0, −4064.0 | crossed: −1201.0, −4088.1 | crossed: −1201.0, −4063.9 |

All twelve crossed, and every one came out on the glued image of its track: 0.00 blocks off it along
the seam (x = −1200 ↔ 1200 at the south seam, z = 701 kept at the east seam), moving away from the seam
on the far side, as the half-turn requires. (A first try with NoAI pigs and no-gravity armour stands
did not move them at all, so those two kinds are untested; arrows from that try stopped half a block
short of the plane, in water, inconclusively.)

WALK_PLACEHOLDER

### 5.4 Fluids, redstone, chunk loading, load [R]

- **Water** placed one block inside the north seam spread along the inside, but neither into the hidden
  outside block nor into the glued block on the far side: fluids stop at the portal plane.
- **Redstone**: a powered wire ending at the north seam does not light a lamp in the glued
  neighbouring block. There is no block adjacency across a portal (as on 2026-10-02).
- **Chunk loading**: with the player 8 blocks inside the north seam, the chunks on the far side of the
  seam (the glued side, 4,000 blocks away in raw coordinates) were loaded by Immersive Portals, and so
  were the hidden outside chunks (vanilla's view distance, which knows nothing of portals).
- **Server load**: 1.7 ms/tick idle with the four global portals and no player; 6.4 ms/tick with one
  player standing at a seam (chunks on both sides). Without a matched run without portals this is no
  measure of the portals' cost; the 2026-10-02 prototype found none that stood out of the noise.

RUN_REST_PLACEHOLDER
