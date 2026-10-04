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

## 0. The answer in plain words

- **A quincuncial Deep Time world with seamless wrapping runs.** The planet (a real bake of the 16k
  quick preset) is laid out in Guyou's aspect of the Peirce quincuncial projection, 16,384 × 8,192
  blocks, and its four edges are joined by four Immersive Portals – CE global portals. In game:
  - terrain is continuous across every seam: the step from a block to its glued neighbour is drawn
    from the same distribution as any step between neighbours, and the world beyond each portal is
    exactly the terrain the portal shows (§5.1);
  - a player sprinting across each of the four seams lands at the glued point to 0.00 blocks along the
    seam, turned 180° on the two turning seams; walking north over the north pole comes out heading
    south on its far side, as on a real globe (§5.3);
  - items, minecarts and arrows cross all four seams and land on the glued track (§5.2).
- **Peirce or Guyou: Guyou** (§2). They are the same conformal map with the same four singular points
  and the same spread of area; Guyou is the oblique aspect with the equator straight. It keeps Deep
  Time's 2:1 grid and x wrap (only the two long edges turn), the equator is one straight east–west walk,
  and the poles are crossed like real poles. Peirce makes a square with four turning edges, the
  singular points on the equator, and an equator walk of four legs through them.
- **What breaks** (§6): collisions near a seam come from the hidden world behind the portal, which a
  player cannot see (found when a walk over the pole stopped at an invisible ice wall); fluids and
  redstone stop at seams; caves and decoration do not continue; Toroidal World must go; two pack mods
  crash with Immersive Portals and Sable ships do not cross (2026-10-02 note). Neither projection is
  equal-area; an equal-area relative (Petroff 2021) uses the same seams.
- **Production: about 5–8 weeks** of agent time with Immersive Portals (§8); a portal-free wrap engine
  of Deep Time's own would add 6–12 weeks.
- **Questions for Ben** (§9): Guyou (recommended) or Peirce; an equal-area relative (recommended) or
  conformal; Immersive Portals (recommended for a first version) or an own engine; project only the
  world at the bake (recommended).

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

### 5.3 A player walking across each seam lands at the matching point [R]

`proto.py client`: a real client; a 3-wide glass bridge 4 blocks above the higher of the two grounds,
30 blocks on the inside and, on the far side, the glued image of the 40 blocks beyond the edge. The
player starts 24 blocks inside and sprints straight at the seam (W + Ctrl held 9 s); RCON reads the
position and rotation every 0.25 s (`*.tsv`).

| Walk | Seam | Last sample before → first after | Off the glued point, along the seam | Across (one sample's run) | Turn of the player |
|---|---|---|---|---|---|
| walk-north | north, half-turn, x = −2000 | (−1999.5, −4095.3) → (1999.5, −4095.0) | 0.00 | 1.68 | 180° |
| walk-south | south, half-turn, x = −3000 | (−2999.5, 4095.3) → (2999.5, 4095.0) | 0.00 | 1.68 | 180° |
| walk-east | east, slide, z = 1500 | (8191.3, 1500.5) → (−8191.0, 1500.5) | 0.00 | 1.68 | 0° |
| walk-west | west, slide, z = 1200 | (−8190.7, 1200.5) → (8191.6, 1200.5) | 0.00 | 1.68 | 0° |
| walk-equator-east | east, slide, on the equator | (8191.9, 0.5) → (−8191.0, 0.5) | 0.00 | 1.12 | 0° |
| walk-north-pole | north, half-turn, at the north pole (x = −4096) | blocked at z = −4095.7, never crossed (see below) | | | |
| walk-north-pole-cleared | the same, hidden outside cleared | (−4095.5, −4095.3) → (4095.5, −4095.2) | 0.00 | 1.51 | 180° |

Every crossing lands exactly on the glued point (0.00 blocks along the seam), the half-turn seams turn
the player 180° (yaw −180° → 0°) and the slides do not, and the player keeps walking on the far bridge
(each walk ends 25 blocks into the far side). Walking north at x = −4096 takes the player over the
north pole: the walk comes out at x = +4096 heading south, which is what crossing a real pole does.

**The pole walk found a real breakage.** The first pole walk stopped half a block before the plane and
never crossed. At the pole the far side is an ice sheet higher than the bridge: the far bridge had a
tunnel cleared through it, but the *hidden outside* (the raw world beyond the plane, which the portal
covers) still held the uncleared ice. Immersive Portals collides the part of the player's box beyond the
plane with those raw blocks. With the same 3 × 3 tunnel also cleared in the hidden outside, the same
walk crossed at once. So collisions at a seam come from blocks the player cannot see. Generated terrain
is the same on both (the folded outside), so on untouched ground it matches what is drawn (to the
one-block offset of §5.1); anything a player builds or digs within a block or two of a seam is not.
A production version has to keep the outside in step with the far side, or stop Immersive Portals from
colliding with it.

Screenshots (`review/out/quincuncial/run3/`, not committed): `walk-north-look.png` (the bridge running
through the north seam over open sea: no seam visible), `walk-north-after.png` (on the far bridge,
heading south), the same for the other walks, `walk-north-pole-after.png` (the ice wall of the first
pole walk), and `run4/walk-north-pole-cleared-*.png`.

**Over land, without a bridge** (`proto.py land`, `land2`). `land_sites` scans every seam each 32
blocks for a point where both sides are land (terrain above y 68): one crossing on the east–west slide
(z = −1696, a wooded hill whose ridge runs along the seam) and one on the south half-turn (x = −5632,
on the south polar ice sheet, y about 120–140); on the test planet no land crosses the north seam.
The seam rows were marked: red concrete on the inside row, blue on the glued row on the far side.

- `run5/seam-view-east-near.png`: from 45 blocks inside, 28 up (spectator mode), looking east at the
  seam: the hill, its trees, the beach and the sea run on past the red row with no step, no gap and no
  change of light.
- `run5/seam-view-east-far.png`: the same hill seen from the other side of the seam (45 blocks inside the
  west edge, looking west), with the blue row on the ridge: the slope continues past it into the east
  side's terrain.
- `run5/seam-view-south-near.png`: the polar ice sheet across the south seam (the markers are hidden
  under snow).
- Ground walks with W, Ctrl and jump held (which in creative mode turned into a low flight 5–7 blocks
  above the ground): east crossed from (8189.6, −1694.2) to (−8189.0, −1694.2), 0.00 off the glued point
  along the seam; south crossed from (−5631.5, 4095.3) to (5631.5, 4091.2), 0.00 off, turned 180°. A
  plain walk on foot stopped at the first one-block step (no auto-jump) and proves nothing either way.
- Light: in every screenshot the daylight is the same on both sides of a seam; block light (torches)
  across a seam was not tested.

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

## 6. What breaks

Seen in this run [R] or carried from the 2026-10-02 prototype, which tested them in game ([02]).

| What | State | Evidence |
|---|---|---|
| Terrain, biomes and climate inputs across a seam | **Continuous** | §5.1, the screenshots |
| Players, items, minecarts, arrows across every seam | **Cross and land on the glued point** | §5.2, §5.3 |
| Collisions near a seam | **Against the hidden outside's blocks**, which nobody sees; matches the far side only while both are untouched generated terrain | §5.3, pole walk |
| Water and lava | Stop at the plane; neither the hidden outside nor the far side gets any | §5.4 |
| Redstone, Create machines, pistons across a seam | No adjacency across a portal: a wire does not power the glued neighbour | §5.4; [02] for pistons and Create |
| Caves, ores, trees, decoration | Placed by block position, so they do not continue across a seam (a cave can end at it) | [I], from how features are seeded |
| Rivers, folds and faults, volcanoes, craters, deposits | Not in the prototype's world data (the resampler drops vector data) | §3 |
| Chunk loading | The far side of a seam is loaded by Immersive Portals when a player is near; the hidden outside is loaded too (vanilla's view distance) and ticks | §5.4 |
| Entities that cross into a chunk that is not ticking | Freeze there until a player comes ([02]); not re-tested | [02] |
| Mobs seeing or pathing across | No ([02]) | [02] |
| Sable ships | Do not cross a same-dimension portal; Sable + Immersive Portals crashed the pack server four times ([02]). IPSable, the compat mod Ben named, was not tried | [02] |
| Toroidal World | Cannot run with Immersive Portals (every join kicked); the dev run patches Deep Time to boot without it | [02], `notw_jar.py` |
| Petrolpark/Destroy, CBC Modern Warfare | Crash at mod construction with Immersive Portals ([02]) | [02] |
| Distant Horizons | Not run. DH draws the raw world in raw coordinates and Immersive Portals does not draw DH terrain through portals ([02]). Here the raw world beyond every seam **is** the folded planet (§5.1), so DH's LODs beyond a seam would show the right terrain from beyond vanilla's render distance on; only the band inside it is the portal's | [I], [02] |
| Maps, compasses, waypoints | Read raw coordinates: a map at a seam draws the hidden outside (the right terrain, the wrong copy for player edits); "north" is up only along the equator | [I] |
| The four singular points | Smooth, stretched terrain round each, a cone of 180° (walking round one takes half the usual distance); placed at sea here | §3 |
| Corners where two seams meet (Guyou: the rectangle's corners, where a slide meets a half-turn; they are singular points) | A walk aimed exactly at a corner escaped the world in [02]; not re-tested. Here the escape lands in the folded outside, which is the right terrain | [02], §3 |
| Server load | 1.7 ms/tick idle with four global portals; 6.4 ms/tick with one player at a seam; no matched run without portals | §5.4 |
| Client cost | [02]: 10–15 % of frame rate per seam in view, 30–40 % at a corner; not re-measured (llvmpipe) | [02] |

[02] = `docs/research/notes/immersive-portals-pillowcase-prototype.md`.

## 7. What a production version needs

1. **The projection in the shipped layer.** Fold `peirce.rs` and `world.rs` into
   `deeptime_format::projection`, with Java and JS twins and the parity fixture (the module's
   interface in `equal-area-projection.md` §2 already has `canonical` and `copies`). A Guyou world
   keeps today's 2:1 grid and x wrap; the new parts are the half-turn fold at the long edges and a
   `north(x, z)` that rotates.
2. **Bake-time resampling, not a re-gridded simulation.** The simulation keeps its own grid; the
   bake writes the world on the Guyou grid, as this prototype's resampler does, with footprint
   filtering. The vector data must be carried over: rivers (polylines split at seams), structures,
   volcanoes, craters, deposit sites and bodies, events, spawn. The simulation's climate stays
   per-latitude internally; only the world is projected.
3. **The fold everywhere the game reads world data:** `FarTerrain`/Distant Horizons, the column rule,
   the samplers. The strip trick (C′ = 4b) hides the problem for the rasters, but anything that wraps x
   at C or clamps z must fold at the long edges.
4. **The seams.** Either Immersive Portals as a required dependency, with the seams installed at world
   creation (four global portals, chunks loaded, a restart or a fix to its `addPortal` cache), or Deep
   Time's own half-turn wrap engine (an extension of Toroidal World's, which has the algebra but refuses
   turning shapes). With Immersive Portals: TW goes (or is fixed to coexist), the pack drops or patches
   Petrolpark/Destroy and CBC Modern Warfare, Sable needs IPSable or a fix for the crash, and the hidden
   outside must be kept in step with the far side (mirror player edits within a few blocks of a seam, or
   patch the collision to use the destination side).
5. **Distant Horizons.** Fold the long edges in `FarTerrain` and `DhLodGenerator` (a few dozen lines;
   the 2026-10-02 note §7) and test what DH and the portals draw where both apply.
6. **Gameplay pieces.** The pole cap goes (both poles are walkable); the F3 line, the climate API's
   latitude, compasses and maps work per position; the workbench draws the Guyou picture.
7. **GameTests.** A seam test dimension with the four portals: terrain continuity (the probe of §5.1),
   entity crossings (§5.2), the pole crossing, fluids and redstone stopping, the fold in `FarTerrain`.
   Real-client walks stay a review script (this one), as today's screenshot driver.

## 8. Effort

Agent working days, for the Guyou world with Immersive Portals, the simulation unchanged:

| Piece | Days |
|---|---|
| Projection module (Rust, Java, JS), parity, docs | 2–3 |
| Bake-time resampling with filtering, rasters | 2 |
| Vector data through the projection (rivers, structures, volcanoes, craters, deposits, events, spawn) | 5–8 |
| Readers' fold at the long edges, `FarTerrain`, Distant Horizons | 3–4 |
| Seam installation at world creation; the hidden-outside sync or collision patch | 4–6 |
| TW removal or coexistence; pack fixes (Petrolpark, CBC Modern Warfare, Sable/IPSable) | 5–10, uncertain: two crashes are in other mods |
| Pole cap removal, F3, climate API, maps, workbench picture | 2–3 |
| GameTests and review scripts | 3–4 |
| **Total** | **about 26–40 days**, roughly 5–8 weeks |

Replacing Immersive Portals with Deep Time's own half-turn wrap engine (no portals; chunk aliasing with
a turn, contraptions and fluids across the seam) adds 6–12 weeks and has no precedent anywhere
(rotated-seam-worlds-prior-art.md). An equal-area relative (Petroff 2021) instead of Peirce's conformal
map adds about 2 days (its forward and inverse in `peirce.rs`; the gluing does not change).

## 9. Open questions for Ben (recommendation first)

1. **Aspect: Guyou (recommended), or Peirce?** Guyou keeps the 2:1 world, the x wrap, a straight
   equator and north up along it, and walks over the poles like the real thing; Peirce puts the north
   pole in the middle of a square and four turning seams round it.
2. **Projection: an equal-area relative in the same layout (recommended: Petroff 2021), or Peirce's
   conformal map?** Conformal draws land near the four singular points without bound larger and the
   poles (Peirce) or the equator (Guyou) at 0.46 of the mean; you chose equal area for today's world.
   The seams and portals are the same either way.
3. **Seams: Immersive Portals as a dependency (recommended for a first version), or Deep Time's own
   wrap engine?** Portals work today for walking and entities, at the price of dropping Toroidal World,
   two pack mods' compatibility, Sable crossings, and fluids, redstone and contraptions at the seams;
   the engine would carry all of that but is months of work with no precedent.
4. **Keep the simulation on its own grid and project only the world at the bake (recommended)?** It
   keeps the climate, flexure and erosion as they are; a simulation run on the quincuncial grid itself
   would be weeks more and buys nothing a player sees.

## 10. Method and files

- Code: `crates/deeptime-quincuncial/` (dev-only crate), `tools/proto/quincuncial/` (`proto.py`,
  `notw_jar.py`, `render.py`). Ledger rows SCL-070 to SCL-077; science page
  [`../science/quincuncial-projection.md`](../science/quincuncial-projection.md).
- Planet: `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500
  --threads 4` (this branch, 2026-10-03), then `deeptime-quincuncial resample --layout guyou --size 8192
  --lon 31`; measurements `deeptime-quincuncial stats|place`. Scratch on the VM under
  `/data/ban/deeptime-runs/quincuncial/` (to be deleted when this lands).
- Game: `tools/remote/mac.sh --server -- 'python3 tools/proto/quincuncial/proto.py setup boot portals
  terrain entities physics client land land2'` (phases can run alone). Results in
  `/data/ban/deeptime-runs/quincuncial/out/report.json`; screenshots pulled to
  `review/out/quincuncial/` (not committed).
- Gates on this branch: cargo fmt, clippy `-D warnings` (workspace), `cargo deny check licenses bans`,
  `cargo test --workspace` (784 passed, 0 failed; the new crate 22), `./gradlew build test` (251 tests,
  0 failures). No GameTest was run: nothing the game reads changed.
