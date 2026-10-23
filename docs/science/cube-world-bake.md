# The bake on a cube world

Status of this page: `stage 1 (written with the code, cube plan phase 1, 2026-10-22; measured on
real bakes, nothing checked against a source)`.

Ledger ids on this page: `SCL-120` to `SCL-129` (see [claims.md](claims.md)); it relies on the
cube's geometry and maps, `SCL-078` to `SCL-081` and `SCL-100` to `SCL-108`, described on
[seamless-world.md](seamless-world.md) ("The cube world").

## What you see in the world

A run that asks for it (`--set scale.projection=cube`) is written as a cube world, format 0.7: the
planet drawn on the six faces of a cube, laid out as a cross in the world (four faces in a band that
wraps east to west, one above and one below the second). The simulation is the same as for any
other world; only the last step, which turns the simulation's grid into the world the game plays,
differs. The game does not play a cube world yet (the plan's phase 2); the default is still the
Petroff-Guyou world, unchanged to the byte.

The map is conformal, so shapes are kept: a round lake is round on every face, and a mountain range
keeps its proportions. It gives up area instead: a block in the middle of a face covers about 1.3
times the planet's mean area per block, and a block near one of the cube's eight corners much less,
so land there is drawn larger. The bake turns the planet inside the cube so that the eight corners
lie far out at sea, where nobody walks. MEASURE-WHAT

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
   read alone. This takes the place of the Petroff-Guyou pass's polar widening (SCL-090): near the
   Lambert grid's poles the footprint follows the grid's stretch exactly instead of by a rule of
   thumb. Categories (rock, biome classes) take the nearest cell.
3. **The topology** (SCL-122). The passes that look at neighbours (the closed basins, the open sea,
   the biome layers' fills and distances, the relief's slopes) take a cell's neighbours on the
   planet, across every seam, and measure distances as great circles. A basin that lies across a
   turning seam is one basin with one level.
4. **The relief** (SCL-123, SCL-125). The block-scale relief (gullies on cones, the jagged
   steep-ground texture, the cliff bands) is drawn on each face in world blocks. Its constants are
   block sizes the game's 4-block noise corners need (gullies at least 8 blocks apart, cliff bands
   at most 24 blocks), so they keep their sizes in blocks; only what a block means in metres
   varies with the map's local scale σ. The noises they read are written in net coordinates, which
   turn at a turning seam; within one wavelength of such a seam the bake mixes each noise with its
   value on the far side, half and half on the seam itself, so both sides read the same value.
5. **Rivers, bodies and points** (SCL-126 to SCL-128). Positions go through the inverse map,
   sizes and directions through the Jacobian. A river that crosses a turning seam is cut on it into
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
| SCL-125 | noise seam blend | reach one wavelength (32 and 512 blocks), smoothstep, ½ on the seam | blocks | none | `game-choice` |
| SCL-126 | rivers | cut on turning seams (mouth 3); width / σ; jump 256 blocks | blocks | none | `game-choice` |
| SCL-127 | spawn margins | 256 from a corner, 64 from a turning edge | blocks | none | `game-choice` |
| SCL-128 | structures on seams | kept and counted | count | none | `game-choice` |
| SCL-129 | deposit forecast | the turn search at 100,000 points, 20,000 turns, 8 refined over 60 rounds of 24 | degrees | none | `game-choice` |

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

MEASURE-TABLES

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
  shape, its centre's; the plan's per-face images are phase 2's (the game side reads them).

## Game choices

Every row on this page is a game choice made by the agent of the cube plan's phase 1 on
2026-10-22, within Ben's answers of the same day (decisions log 2026-10-22: format 0.7 face by face
with the map named by its sha256, Q1; the conformal cube turned freely, Q4). The ledger rows give
the reason for each value.

## Open questions

- MEASURE-OPEN

## Sources

- Shoemake1992 (from memory): the uniform random turns of the search.
- The cube's maps rest on Lee1976, Rancic1996 and Moser1965 (see [seamless-world.md](seamless-world.md)).
- docs/research/notes/mountains-and-the-projection.md §5.4 (the projection study): the turn's
  score, the search and the two targets (land at most twice its area, corners 5° at sea).
