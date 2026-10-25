# The bake on a cube world

Status of this page: `stage 1 (written with the code, cube plan phase 1, 2026-10-22; measured on
real bakes, nothing checked against a source)`.

Ledger ids on this page: `SCL-120` to `SCL-130` and `SCL-215` (see [claims.md](claims.md)); it relies on the
cube's geometry and maps, `SCL-078` to `SCL-081` and `SCL-100` to `SCL-108`, described on
[seamless-world.md](seamless-world.md) ("The cube world").

## What you see in the world

Every planet is written as a cube world, format 0.7: the planet drawn on the six faces of a cube,
laid out as a cross in the world (four faces in a band that wraps east to west, one above and one
below the second). The simulation is the same as for any other world; only the last step, which
turns the simulation's grid into the world the game plays, differs. (When this page was written,
in phase 1, a run asked for it with `--set scale.projection=cube` and the default was the
Petroff-Guyou world. Since cube plan §7.4, 2026-10-24, Ben's "Cube only (Recommended)" in the
decisions log, the cube is the only world a planet bakes to and the game plays; the Petroff-Guyou
world the measurements below compare against is deleted.)

The map is conformal, so shapes are kept: a round lake is round on every face, and a mountain range
keeps its proportions. It gives up area instead: a block in the middle of a face covers about 1.3
times the planet's mean area per block, and a block near one of the cube's eight corners much less,
so land there is drawn larger. The bake turns the planet inside the cube so that the eight corners
lie far out at sea, where nobody walks. On the 13 Earth-like 16k planets it was tried on, every
corner fell at sea, 2.8° to 8.7° from the nearest land; no land is drawn at more than 2.5 times
its planet area, and 99.9 % of it at no more than 2.2 times.

Across the four seams that are straight in the net, and across the band's wrap, the world runs on
as one surface. The other six seams turn: N's east edge is F1's top seen after a quarter turn, and
so on. Until the game's wrap engine joins them, those seams are borders; the bake already writes
both sides so that they match: heights, rivers, lakes, biomes, cliffs and gullies run on across
every seam on the planet.

## The model in plain words

The pass runs after the Lambert bake, on the finished world folder, and rewrites every layer.

1. **The turn** (SCL-120, SCL-121). A turn is a rotation from the planet's frame to the cube's.
   Its score is the least of the eight corners' signed distances to the coast (positive at sea),
   plus a thousandth of their sum, each clamped to ±20°. A seeded generator draws 60,000 uniformly
   random turns; the 24 best, and the best turn about the polar axis, are refined by small random
   turns that shrink from 4° to 0.03°. The cube's 24 symmetries give the same corners; the bake
   takes the one with the north pole in face N and the most land on F0. A preset that pins
   `scale.lambda_c_deg` gets the polar turn instead, so A/B bakes of one planet share a frame. The
   land is the Lambert bake's elevation at or above sea level.
2. **The rasters** (SCL-124). Every world cell is a point sample at its centre. The map's
   Jacobian, the turn and the Lambert grid's metric say how many simulation cells the world cell
   spans along each of its two principal axes. Where it spans more than one, a Gaussian of the
   remaining width averages the simulation along that axis; where it spans less, the bilinear
   read alone. This takes the place of the Petroff-Guyou pass's polar widening (SCL-090, retired with that world): near the
   Lambert grid's poles the footprint follows the grid's stretch exactly instead of by a rule of
   thumb. Categories (rock, biome classes) take the nearest cell.
3. **The topology** (SCL-122). The passes that look at neighbours (the closed basins, the open sea,
   the biome layers' fills and distances, the relief's slopes) take a cell's neighbours on the
   planet, across every seam, and measure distances as great circles. A basin that lies across a
   turning seam is one basin with one level.
4. **The relief** (SCL-123, SCL-215). The block-scale relief (gullies on cones, the jagged
   steep-ground texture, the cliff bands) is drawn on each face in world blocks. Its constants are
   block sizes the game's 4-block noise corners need (gullies at least 8 blocks apart, cliff bands
   at most 24 blocks), so they keep their sizes in blocks; only what a block means in metres
   varies with the map's local scale σ. The noises they read (the jag, the cliff bands' levels)
   are the cube's surface noise, the same noise the game's detail term reads: value noise on a
   lattice laid on the cube's surface, whose lines meet the cube's edges, so it runs on through
   every seam and corner and both sides of a seam read one value with nothing blended. (Until cube
   plan G3 the noises were written in net coordinates and blended within one wavelength of a
   turning seam, SCL-125, now retired.)
5. **Rivers, bodies and points** (SCL-126 to SCL-128, SCL-130). Positions go through the inverse
   map, sizes and directions through the Jacobian. A deposit body's ellipse is the exact image of
   its planet ellipse under the Jacobian at its centre (its semi-axes the map's singular values),
   so it keeps its planet area where the map shears; the Petroff-Guyou pass stretched each axis
   alone (SCL-091), which grew a sheared body. A river that crosses a turning seam is cut on it into
   two chains: the upstream one ends on the seam with mouth code 3 (`seam`) into the downstream
   one, which starts at the same planet point on the partner edge. River widths are divided by the
   local scale, so a river is as many metres wide everywhere; depths are kept. Structure windows
   that cross a turning seam are kept and counted. Spawn candidates are kept 256 blocks from a
   corner and 64 from a turning edge.
6. **The guaranteed deposits** (SCL-129). The deposit stage runs before the bake and needs to know
   which regions of the world each essential deposit will fall in. On a cube world it forecasts the
   bake's turn with a smaller search on its own land, lays its regions on the net's window (the
   void outside the six faces holds no land), and the top-up after the bake fills any region the
   forecast missed.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-120 | turn score and search | min_k d_k + 0.001 Σ_k d_k, d clamped to ±20°; 400,000 points, 60,000 random turns, the 24 best refined in 120 rounds of 32 trials from 4° to 0.03°, the polar turn in 0.25° steps | degrees | Shoemake 1992 (from memory) for the random turns | `game-choice` |
| SCL-121 | corner zone | 5 | degrees | none | `game-choice` |
| SCL-122 | topology | neighbours on the planet across all 12 seams; great-circle distances | none | geometry | `game-choice` |
| SCL-123 | relief in world blocks | gullies 8 to 12 blocks apart, jag 32 blocks, cliff bands 24 blocks, σ recorded | blocks | none | `game-choice` |
| SCL-124 | raster footprint | D_k = r_w √λ_k / r_s; width r_s √(D_k² − 1), dropped under 0.25 r_s, at most C/12 | blocks | none | `game-choice` |
| SCL-125 | noise seam blend (retired with G3) | reach one wavelength (32 and 512 blocks), smoothstep, ½ on the seam | blocks | none | `merged` into SCL-215 |
| SCL-215 | surface noise | value noise on the cube's surface lattice, m = max(1, round(s/λ)) cells per cube edge, fade f²(3 − 2f) | blocks | none | `game-choice` |
| SCL-126 | rivers | cut on turning seams (mouth 3); width / σ; jump 256 blocks | blocks | none | `game-choice` |
| SCL-127 | spawn margins | 256 from a corner, 64 from a turning edge | blocks | none | `game-choice` |
| SCL-128 | structures on seams | kept and counted | count | none | `game-choice` |
| SCL-129 | deposit forecast | the turn search at 100,000 points, 20,000 turns, 8 refined over 60 rounds of 24 | degrees | none | `game-choice` |
| SCL-130 | body ellipse | semi-axes = singular values of J R(yaw) diag(a, c); the record's area over 12 x 4 points | blocks | none | `game-choice` |

The footprint. The map's Jacobian, turned onto the planet, gives the ground's displacement east
and north, in simulation blocks, per block of +x and of +z. The simulation's Lambert grid is
cylindrical and equal-area (x along the longitude, z along the sine of the latitude φ), so a step
east of e blocks is e / cos φ Lambert blocks in x and a step north of n blocks is −n cos φ in z;
that gives L, Lambert blocks per world block. The eigenvalues λ1 ≥ λ2 of L Lᵀ are the squared
stretches along the cell's principal axes, so

    D_k = r_w √λ_k / r_s          (simulation cells per world cell along axis k)
    width_k = r_s √(D_k² − 1)     (0 if under 0.25 r_s; the long axis at most C/12)

and the cell's value is the Gaussian-weighted mean of bilinear reads along the two axes (at most
2,049 and 65 of them). r_w and r_s are the world's and the simulation's cell sizes in blocks, C the
simulation's circumference in blocks. The record's checkerboard leak for a band is

    leak = 0.657 · ½ (R(f_u + f_v) + R(f_u − f_v)),   R(f) = exp(−2π² width² f²)

where 0.657 = (8/π²)² is the bilinear read's own response to a checkerboard at the simulation's
resolution and f the checkerboard's frequency along each axis.

## Measured on real bakes

The runs (2026-10-22, on the test VM, `build/cube1/cube_bakes.sh b2` with the branch at e00f9f88;
each world then through `deeptime world-map` and the kept ones through `deeptime deposit-audit`;
the numbers gathered by `build/cube1/numbers.py`):

- `planet_world_4k` seed 11 (face 1,024 blocks) as a cube and as Petroff-Guyou, with
  `deposits_world_4k`'s three essentials at 2,048 blocks.
- `earthlike_quick_16k` seed 1 (face 3,840 blocks), from a supercontinent over 1,000 Myr, as a cube
  and as Petroff-Guyou, with the game's six essentials.
- `earthlike_quick_16k` seeds 2 to 13, the same story, as cubes, for the turn.

The four kept worlds are in the shared pool (`/data/ban/deeptime-runs/cube1/b2/`, listed in
`/data/ban/deeptime-runs/pool/BASES.md`). The B1 control columns and the bodies' areas come from
a later bake of the two cubes on the merged branch (`build/cube1/cube_bakes.sh b5 build,p4kc,q16c`).

**Seams (B1).** The 99th percentile of the elevation step between neighbouring L3 cells, in
blocks: inside a face, on the first step inside a face edge, across each kind of seam, and the
first step beside each kind of seam on both of its faces (the same ground's control).

| world | inside a face | inside an edge | direct seams (beside) | the slide (beside) | turning seams (beside) |
|---|---|---|---|---|---|
| 16k seed 1 | 4.56 | 3.94 | 3.0 (3.0) | 5.56 (5.63) | 3.25 (3.25) |
| 4k seed 11 | 3.94 | 3.38 | 3.5 (3.44) | 2.56 (2.44) | 3.94 (3.5) |
| 4k test planet, G3 | 3.94 | 2.69 | 3.38 (3.44) | 3.38 (3.38) | 2.31 (2.25) |

Every kind of seam steps no more than the inside of a face. At 16k each also steps no more than
the ground beside it: the slide's 5.56 is the band's own steep ground there (5.63 beside it), not
a seam. At 4k the turning seams' 3.94 is a little over the 3.5 beside them and equal to the inside
of a face; the 4k planet has 3,072 turning-seam pairs, a quarter of the 16k's sample.

The largest steps: 40.6 blocks inside a face against 6.2, 7.4 and 5.6 across the direct seams, the
slide and the turning seams (16k). The detail layer's amplitude steps are 0.375 inside a face and
0.25 to 0.625 across seams (p99, 16k).

The first two rows were measured with the phase-1 seam blend (SCL-125, retired). The third is the
4k test planet (`planet_world_4k` baked as a cube, s = 1,024) after G3, with the detail noise on
the cube's surface lattice and no blend (SCL-215; `build/cube2/g3-checks.sh` on the VM,
2026-10-23): every kind of seam steps no more than the ground beside it, give or take one sixteenth
of a block, and less than the inside of a face. The largest steps are 12.6 blocks inside a face
against 4.06, 3.56 and 4.25 across the direct seams, the slide and the turning seams. The detail
layer's amplitude steps (p99) are 0.375 inside a face and 0.25, 0.1875 and 0.3125 across the
direct seams, the slide and the turning seams, each the same as beside them.

**The footprint (B2).** The share of L3 cells and the record's checkerboard leak by D along the
cell's long axis (simulation cells per world cell; L1 and L2 within 0.1 % of the same shares):

| D (long axis) | under 1 | 1 to 1.1 | 1.1 to 1.2 | 1.2 to 1.5 | 1.5 to 2 | 2 to 4 | 4 and over |
|---|---|---|---|---|---|---|---|
| 16k: share | 16.4 % | 23.9 % | 17.9 % | 18.9 % | 10.5 % | 9.2 % | 3.0 % |
| 16k: leak | 0.657 | 0.358 | 0.132 | 0.027 | 2.6e-4 | 1.5e-8 | 0 |
| 4k: share | 28.0 % | 22.3 % | 14.0 % | 15.4 % | 9.5 % | 8.2 % | 0.7 % |
| 4k: leak | 0.657 | 0.439 | 0.156 | 0.025 | 2.6e-4 | 1.6e-8 | 0 |

Under D = 1 the world cell is finer than the simulation's and shows the checkerboard as it is: no
alias. The plan's 1 % holds from D = 1.5 on the long axis (and, tested, where both axes span 1.2
cells or more). Between 1 and 1.5 it does not: see "Open questions".

**The turn (B3, SCL-120, SCL-121).** For each planet: the least distance from a corner to land, whether
all eight are 5° or more from land, the land's blocks per planet area (99.9th percentile and
largest) and the share of the 60,000 random turns that put all eight corners at sea.

| planet | least distance | all 5° | land p99.9 | land largest | random turns all at sea |
|---|---|---|---|---|---|
| 16k seed 1 | 4.0° | no | 1.85 | 2.13 | 4.7 % |
| 16k seed 2 | 6.25° | yes | 1.59 | 1.72 | 10.9 % |
| 16k seed 3 | 5.55° | yes | 1.69 | 1.79 | 10.8 % |
| 16k seed 4 | 8.2° | yes | 1.43 | 1.51 | 5.7 % |
| 16k seed 5 | 3.4° | no | 2.07 | 2.30 | 0.85 % |
| 16k seed 6 | 2.8° | no | 2.20 | 2.50 | 0.93 % |
| 16k seed 7 | 8.7° | yes | 1.43 | 1.49 | 12.3 % |
| 16k seed 8 | 4.5° | no | 1.80 | 1.95 | 9.5 % |
| 16k seed 9 | 5.5° | yes | 1.67 | 1.84 | 9.0 % |
| 16k seed 10 | 5.8° | yes | 1.66 | 1.78 | 4.4 % |
| 16k seed 11 | 5.45° | yes | 1.71 | 1.83 | 9.3 % |
| 16k seed 12 | 5.35° | yes | 1.60 | 1.79 | 5.1 % |
| 16k seed 13 | 3.5° | no | 1.81 | 1.97 | 8.4 % |
| 4k seed 11 | 15.65° | yes | 1.11 | 1.12 | 48.3 % |

All eight corners are at sea on every planet. Both of the study's targets (corners 5° from land,
land at most twice its area) hold on 8 of the 13 16k planets; the land within 5° of a corner is at
most 0.34 % of the land (seed 6). The best turn about the polar axis (the plan's first B3) reached
only −2.93° to 2.17° on the same planets: on most of them it leaves a corner on land.

**Bodies, structures, spawn (B4, SCL-130).** A body's planet area as drawn, against its source's:
the map's local scale squared averaged over 12 directions on 4 rings of the ellipse, times the
ellipse's area. The medians, under SCL-130 and (beside) under SCL-091 for the same bodies:

| world | mid-face | within 64 blocks of a face edge | within 5° of a corner | largest change |
|---|---|---|---|---|
| 16k seed 1 (7,045, 393 and 10 bodies) | 0.99999 (1.0007) | 0.99999 (1.0041) | 1.0007 (1.058) | 0.0043 (16.2) |
| 4k seed 11 (241, 43 and 0 bodies) | 0.99997 (1.00002) | 0.99996 (0.99999) | none | 0.098 (99.3) |

The largest change is that of the one body furthest from 1: where the map shears, SCL-091 draws
one 16k body at 17 times its planet area and one 4k body at 100 times. Two more 4k bodies hold the image
of a corner (an aureole 61.8 by 101.5 blocks and a pluton 57.2 by 88.1, both centred 38 blocks
from corner s0): they read 1.38 and 1.31 (SCL-091 1.39 and 1.32). Round a corner the world has
three quarters of a turn, so an ellipse over it covers a full turn of the net and the quarter in
the void folds back onto ground it already covers: about 4/3 for any one ellipse. The record lists
such bodies apart (`over_a_corner`) and leaves them out of the medians; phase 2's per-face images
are theirs (0.16 of each lies in the void). Away from the corners, 52 bodies at 16k and 11 at 4k
reach past a turning seam into the void (at most 0.49 and 0.41 of one ellipse); the game has to
fold that part onto the partner face. The first record (b2) read the scale at the body's centre alone, which
misses that fold: it gave the two 1.007.

Structure windows that cross a turning seam: 303 of 18,450 (16k) and 107 of 532 (4k), all kept.
Eight spawn candidates on each.

**Rivers (B5, SCL-126).** 16k: 4,015 chains; 12 cut on turning seams, each with its partner; none
cut on the direct seams or the slide, which 62 chains cross whole; the planet length changes by
2.1e-8. 4k: 57 chains, none on a seam (5.7e-7).

**Lakes (B6, SCL-122).** 16k: the 21 cell pairs across turning seams with a lake on both sides have
the same water level, as do the 19,095 such pairs inside faces.

**Biomes (B7).** The equal-area census of `deeptime world-map` (2,097,152 points on the planet),
in % of the planet; the Köppen classes are `climate/koppen_l2`, the classes the biome inputs were
placed by.

| class | 16k cube | 16k Petroff-Guyou | 4k cube | 4k Petroff-Guyou |
|---|---|---|---|---|
| land | 27.78 | 27.77 | 7.785 | 7.786 |
| ice | 4.517 | 4.531 | 0 | 0 |
| wetland | 1.842 | 1.892 | 1.071 | 1.007 |
| Af | 2.851 | 2.912 | 1.157 | 1.163 |
| Aw | 1.480 | 1.475 | 0.619 | 0.737 |
| BSh | 1.175 | 1.189 | 1.314 | 1.254 |
| BSk | 1.763 | 1.764 | 0.855 | 0.844 |
| Cfa | 2.458 | 2.464 | 0.549 | 0.506 |
| Cfb | 3.240 | 3.235 | 0.832 | 0.787 |
| Dfc | 1.508 | 1.510 | 0.045 | 0.049 |
| EF | 4.546 | 4.531 | 0 | 0 |
| ET | 1.596 | 1.584 | 0.148 | 0.140 |

At 16k every class over 0.5 % of the planet is within 2.7 % of itself (wetland; Af 2.1 %), land
and ice within 0.3 %. At 4k the land is the same to 0.02 %, but the classes move more: Aw 16 %,
the Köppen land as a whole 2 % less on the cube.

**Essentials (B8, SCL-129).** `deeptime deposit-audit`, regions of the shipped sizes: on the 16k cube
every top-up land region of each of the six essentials holds a source with ore on dry ground
(coal and iron 8 of 8, copper, gold, lead and zinc 3 of 3; Petroff-Guyou 8 of 8 and 2 of 2), with
14 guaranteed sites, each with ore at its centre, and no region short. On the 4k cube coal, copper
and iron hold 3 of 3 (Petroff-Guyou 2 of 2), with 7 guaranteed sites.

**Relief and scale (SCL-123).** σ over the faces: 77 to 331 m per block at 16k (median 305, nominal
294.8), 72 to 311 at 4k (median 286, nominal 276.4). The relief, 16k cube against Petroff-Guyou:
51 and 51 active cones gullied with 8 gullies each (median), gully span 16.4 and 15.7 blocks, the
jag on 529,535 and 505,100 L3 cells, cliff bands on 145,204 and 128,578, mean change 0.95 and
1.01 blocks. 4k: 3 and 3 cones gullied, 8 gullies each.

**Coexistence (phase 1; gone since cube plan §7.4 deleted the Petroff-Guyou world, 2026-10-24).**
With `scale.projection` unset the bake writes the Petroff-Guyou world of format
0.6 exactly as master does: `tools/review/projection-identity.sh` on master (8485aaf6) and on the
branch gives the same hashes for all 385 files of every bake and staged world but two of the
viewer's: `report.js` differs between two runs of one binary, and `bundle.js` holds its build time
and the preset's folder in its title. No field was added to a Petroff-Guyou world's notes. The game
(Java) still refuses a cube world.

## Simplifications and limits

- **Corners.** A corner of the cube is a cone point: three faces meet there, and a full turn round
  it is three quarters of a turn of world. The turn puts all eight at sea; the land nearest to a
  corner is drawn larger than its planet area (SCL-121 records how much).
- **Turning seams are borders for now.** The bake writes both sides to match, but until the wrap
  engine (phase 2) joins them, a player meets a wall there. River chains are cut on them; the
  game's river carving has to treat a `seam` mouth as continuing into its partner.
- **The simulation is still Lambert.** The footprint follows the Lambert grid's stretch; its
  poles are the simulation's, not the world's. Where the world cell is finer than the simulation
  (near the simulation's equator on a 16k world the long axis spans less than one cell), the bake
  shows the simulation's resolution, as on every other world. Phase 4 moves the simulation onto
  Moser's equal-area cube.
- **Per-face body images.** A deposit or intrusion body that crosses a turning seam keeps one
  shape, its centre's; the part past the seam lies in the net's void and the game would fold it
  onto the partner face. A body over a corner draws about 4/3 of its planet area (see B4). The
  plan's per-face images are phase 2's (the game side reads them).
- **The simulation's own artefacts show through.** Near the Lambert simulation's poles its fans of
  stretched cells and its stretched impact ellipses appear on the cube as on Petroff-Guyou: the
  bake shows what the simulation made. Phase 4 moves the simulation off that grid.

## Game choices

Every row on this page is a game choice made by the agent of the cube plan's phase 1 on
2026-10-22, within Ben's answers of the same day (decisions log 2026-10-22: format 0.7 face by face
with the map named by its sha256, Q1; the conformal cube turned freely, Q4). The ledger rows give
the reason for each value.

## Open questions

- **The footprint just above D = 1.** Between D = 1 and 1.5 on the long axis (60 % of a 16k
  world's cells) a checkerboard at the simulation's resolution leaks 3 % to 36 %, as a slow beat.
  A width that grows from 0 at D = 1 cannot stop it; a filter that could (0.92 simulation cells,
  everywhere or from D = 1 on) halves every feature 4 cells across. The bake's rasters carry little
  at their own resolution, so the bake keeps the sharper world; a pre-blur is the fix if beats show.
- **Small islands and the turn.** The search's 400,000 points lie 0.32° apart, so it can miss the
  smallest islands; the record's distance then sits under the search's (seed 13: 3.5° against
  4.6°). More points cost little.
- **Biome shares on a small 4k planet.** The Köppen classes of the 4k cube differ from the
  Petroff-Guyou world's by up to 16 % of a class (Aw, 0.6 % of the planet) and its Köppen land is
  2 % smaller, where the 16k world agrees to 2.7 %. Not traced.

## Sources

- Shoemake1992 (from memory): the uniform random turns of the search.
- The cube's maps rest on Lee1976, Rancic1996 and Moser1965 (see [seamless-world.md](seamless-world.md)).
- docs/research/notes/mountains-and-the-projection.md §5.4 (the projection study): the turn's
  score, the search and the two targets (land at most twice its area, corners 5° at sea).
