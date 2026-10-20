# The seamless world: Petroff's equal-area map in the Guyou layout

Status of this page: `stage 2 (written with the code, 2026-10-04; Petroff's paper and code read in
full, the map's properties tested; the layout and its sizes are game choices)`.

Ledger ids on this page: `SCL-049`, `SCL-083`, `SCL-084`, `SCL-086` to `SCL-091`, `POL-022` to
`POL-024` (see [claims.md](claims.md)).

## What you see in the world

A Deep Time world (format 0.5, the default) is a rectangle of 13,312 × 6,656 blocks for a 16k planet
and 26,112 × 13,056 for a 32k one: two squares, each holding one hemisphere, side by side. The equator
runs straight across the middle (z = 0); north is −z along it. Every block covers the same area of
the planet, as on the Lambert world before it, but shapes are stretched in places: half the planet is
drawn with one direction up to 1.78 times longer than the other, and at most 2.67 times (Petroff's
own figures, reproduced by the projection survey). Walking east or west wraps round the planet as
before (Toroidal World). The north and south edges are each glued onto themselves by a half-turn
about their midpoint: walking north over the north pole comes out heading south on the far side, as
on a real globe. Until Deep Time's wrap engine joins those two edges, a temporary border stops
players one block inside them; the terrain beyond the border is already the glued planet, so the view
over it is right.

Near the north and south edges the action bar says the edge is closed for now (24 blocks before the
stop line), and at the line a player, the vehicle they ride or a Sable ship is put back inside;
spectators pass. There is no polar storm and no pole cap on this world: the poles are not stretched,
so nothing needs closing off there (POL-022, POL-023). New players never spawn within 64 blocks of
those edges (POL-024). The F3 line and Deep Time's climate API (version 4: `latitudeDeg(level, x,
z)`, `longitudeDeg`, `projectionKind`) give each block's true latitude and longitude, which on this
world depend on both x and z. The Lambert world stays available: a preset's `scale.projection`
`lambert` (the planet editor's "World map") bakes it, played on the level types
`deeptime:planet_lambert` and `deeptime:planet_lambert_32k`.

The poles are ordinary places of the planet. They lie on the north and south edges, a quarter of the
way in (x = ∓b/2), so for now they stand at the border. Four other places on the edges, the cone
points, are where a full turn of planet is drawn in half a turn of world: walking round one takes half
the usual distance, and the map there is stretched without bound. The bake turns the planet about
its axis so these four points lie as far from land as the planet allows.

## The model in plain words

1. The simulation runs as it always has, on its own Lambert grid; only the world is projected, at
   the end of the bake (Ben, decisions log 2026-10-03, "Bake only").
2. **The map.** Petroff (2021) draws the whole sphere in a square with the north pole at its centre,
   equal-area and in closed form (ledger SCL-083). Its northern half is a diamond; tilted 45° it is a
   square showing one hemisphere with the equator round its edge. Two of these squares, the second
   holding the southern hemisphere mirrored, make a 2 : 1 rectangle whose edges are all the same
   circle of the sphere drawn from both sides, so every edge glues exactly (SCL-086).
3. **Guyou's aspect.** The sphere is turned so that the first square's centre is a point of the
   equator at longitude λc and its top-edge midpoint the north pole. The equator then runs straight
   through both squares (SCL-086).
4. **Size.** The rectangle has the planet's area at today's scale: 2b² = 4πR², b = C/√(2π), rounded
   to a multiple of 256 blocks (SCL-084).
5. **λc.** The bake tries every quarter degree and keeps the longitude that puts the four cone points
   farthest from land (SCL-087).
6. **Resampling.** Every world cell finds its point on the sphere and reads the simulation's layers
   there, averaged over a patch that is round on the sphere (SCL-090). Near the poles a Lambert row is
   many blocks tall but its columns a fraction of a block wide; read at a point, that draws radial
   streaks round the poles. The round patch averages across the narrow columns until both directions
   are resolved alike. That alone is not enough: the simulation's solvers work in Lambert blocks,
   so its polar terrain itself has the grid's shape, each feature long north–south and thin
   east–west on the planet, and the round patch only shortens the streaks (to a thirteenth of the
   slope energy the point sample had, still thirteen times the radial slope). Poleward of about 59°
   the patch is therefore also stretched along the parallel, by (0.52 / cos φ)⁴ and at most 8
   times (from about 72°), which leaves only what the simulation resolved in both directions. Categories and rock
   stacks take the nearest cell; the climate's band coordinates, set per Köppen class, are read at
   the point so they keep to the class (a wide polar patch would mix neighbouring classes' values).
7. **Things with a position** (rivers, folds and faults, ore bodies, sites, volcanoes, craters, spawn
   points) move through the projection; directions and lengths through its local stretch. A river
   that crosses a turning edge is cut there.

## The forward on the diagonals and at the cone points

The forward (latitude and longitude to block) is Petroff's construction step by step. It finds the
point's spherical triangle (the 67.5° vertex V, a corner C and a third vertex W) and then places the
point P′ in the matching flat triangle: U′ on the side V′W′ at the distance up from V′ (the area
share), P′ on C′U′ at the fraction xpxpyp from C′, and P′'s angle at V′ (alphap) from the three
sides of the triangle V′U′P′ by the law of cosines (SCL-083).

**What was wrong (found 2026-10-20 by the developer map).** When P′ lies on one of that triangle's
sides the triangle V′U′P′ is flat, and the law of cosines gives rounding noise. That happens on
exact inputs on the square's diagonals, which in the world are the equator and the meridians λc and
λc + 180°: from a tile's centre to its 67.5° vertex (the equator at λc ± 22.5°, latitude ±22.5° on
the centre meridian) up is rounding noise, and from that vertex to the edge P′ lies on V′U′ (yp ≈ 0).
At the cone points P is the corner C itself, so up = 0, and the code took the fallback angle
ξ′ − π/6, which is the whole angle of one of the three flat triangles but not of the two the cone
points belong to. Petroff's own `new_projection.py` (Zenodo 4661351, re-read 2026-10-20 at its
`alphaprime` step) has the same fallback ("Avoid divide by zero": `alphaprime = (xiprime - np.pi /
6) * idx` where `uprime * rprime <= 0`). On the play planet (b 6,656, λc 146°) 180 of 4,341 exact
inputs on these lines came back more than 1e-5° from where they were asked: up to 1.6° on the
equator, 1.1° to 1.4° on the meridians λc and λc + 180°, and 10.5° at the cone points, where
(45° N, 236°) landed 616 blocks along the edge. Random inputs never land exactly on a line, which is
why the round-trip tests had not seen it.

**How the forward does it now.** Within 1e-6 local units of either side (up or yp at most 1e-6,
about 0.002 blocks on a 6,656-block hemisphere) it places P′ by coordinates instead: V′ at the
origin, U′ = (up, 0), C′ = cp (cos gp, sin gp), P′ = C′ + xpxpyp (U′ − C′), and P′'s distance and
angle are read from its coordinates. That is the same point the law of cosines describes, without
the cancellation. Everywhere else the code is unchanged and gives the same bits: none of 200,000
random inputs in each of four worlds changes, nor any existing row of the parity fixture. The switch
is numerical, not physical: just above it the law of cosines is still good to about 1.5e-7 blocks
(measured against a 50-digit mpmath reference of the construction, inputs 10⁻² to 10⁻¹⁵ degrees off
the lines). Rust, Java and the pages make the same change and agree bit for bit on the fixture's
exact rows.

Measured after the fix: every exact input on the equator (every 0.1° of longitude), on the meridians
λc, λc ± 90° and λc + 180° (every 0.1° of latitude), at the cone points, the 67.5° vertices, the
tile centres and the poles comes back within 1e-5° in five worlds (`exact_inputs_on_the_diagonals_round_trip`,
`ProjectionParityTest.exactInputsOnTheDiagonalsRoundTrip`); the worst is 4.2e-7°, which is the
inverse's own rounding on the diagonals (the forward is within 1.4e-7 blocks of the reference
there). The inverse is unchanged.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-083 | Petroff's map | φ0 = 3π/8; ψ0 = asin(1/√(2 − cos²φ0)); h′ = (12/π)(ψ0 + asin(2 sin φ0/√(3 − cos 2φ0)) − π/2); forward and inverse closed forms | radians | Petroff 2021, CC0 code | `verified` |
| SCL-084 | hemisphere side | b = 256·round(C/(256√(2π))): 6,656 (16k), 13,056 (32k) | blocks | — | `game-choice` |
| SCL-086 | the layout and its properties | X = 2x/b + 1, Y = −2z/b; equal area, seams close to 10⁻¹² rad | blocks | Petroff 2021 | `verified` (tested) |
| SCL-087 | λc search | 0.25° steps, distances to land to 20° | degrees | — | `game-choice` |
| SCL-088 | temporary border | 1 block inside the north and south edges | blocks | — | `game-choice` |
| SCL-089 | detail noise at the edges | blend over 32 blocks to the half-turn-symmetric mean | blocks | — | `game-choice` |
| SCL-090 | footprint | D = max(stretch·r, r / cos φ), stretch ≤ 8; along the parallel × min(8, max(1, (0.52 / cos φ)⁴)) | blocks | — | `game-choice` |
| SCL-091 | a body's semi-axes through the projection | each axis x the mapping's local scale along it; where that scale is not finite or not > 0 (a body on or past the pole row) the source axis is kept | blocks | the format refuses a zero axis | `game-choice` |
| POL-022 | the game's edge border; no pole cap | stop 1 block inside the north and south edges; spectators pass; off when the wrap engine joins the edges | blocks | — | `game-choice` |
| POL-023 | the border's warning | 24 blocks before the stop line, every 40 ticks | blocks, ticks | — | `game-choice` |
| POL-024 | spawn clear of the edges | candidates within 64 blocks of an edge are passed over | blocks | — | `game-choice` |

The world block's true size is √(4πR²/2b²) metres (294.6 m at 16k, 300.4 m at 32k), written as
`scale.m_per_block_h`.

## Earth against the mod

Not a model of Earth: these are properties of a map. Measured on the bake (STATUS build log, SW-1;
earthlike_quick_16k, scattered_continents, 500 Myr, seed 1, `tools/review/seamless-evidence.sh`):

| Quantity | On the planet | On the Petroff–Guyou world |
|---|---|---|
| land share | 0.13168 | 0.13174 |
| ice cover share | 0.0072 | 0.0093 (the ice layer is coarse, L2, and its footprint wide near the poles) |
| tangential over radial slope, 80–88° | — | 1.84 (point sample 77.5; mid-latitudes 1.36) |
| tangential over radial slope, 70–80° | — | 4.04 (point sample 26.5) |
| cone points' distance to land | — | 7.0° at the least (λc 42.5°; 39 % of candidates put all four at sea) |

On the earlier bake of the same preset the isotropic footprint alone gave 13.4 at 80–88° and 10.3 at
70–80° (point sample 47.4 and 14.1); the widening brings them to 1.39 and 3.2.

## Simplifications and limits

- Shapes are not true: up to 2.67 times longer one way than the other, worst near the four cone
  points. Ben accepted this (decisions log 2026-10-04) and may swap to the cube if it feels too bad in
  play.
- The simulation still runs on Lambert's grid, whose polar cells are slivers; the world cannot show
  detail the simulation did not resolve near the poles, so the polar regions are smoother than the
  rest (they are no longer streaked).
- Footprint averaging softens the terrain a little everywhere (a box of a fraction of a cell at the
  equator), and the polar caps much more along the parallels (up to 8 times the row spacing east–west), so
  above about 75° the terrain is smooth on the scale of a few hundred blocks; the game's detail
  noise, drawn in the world's own blocks, is what gives it texture there.
- The widening removes the fine streaks, not the broad ones: features the simulation drew many
  rows long (its solvers work in Lambert blocks, so a valley or ridge a few cells across at 70° is
  several times longer north–south than east–west on the planet) still fan out from the poles at
  65–80° latitude, softened (the tangential-over-radial slope ratio at 70–80° falls from 14–26 for a
  point sample to 3–4; at 80–88° from 47–78 to 1.4–1.8, the mid-latitudes' 1.3–1.4). The same striation is in
  the Lambert world, where it runs north–south. Removing it needs solvers that work in the planet's
  metric (the sphere-native grids research, decisions log 2026-10-04).
- River chains crossing the north or south edge are cut there; a river does not yet continue through
  the seam.
- Until the wrap engine joins the north and south edges, the poles are reachable only up to the
  border.
- The terrain past an edge (seen from the border) is the folded inside for everything read from the
  rasters (height, rock, water levels, climate, biomes); point features placed per chunk (ore bodies,
  rivers' channels, structures) are not copied across, since nobody can reach that side yet.

## Game choices

- The projection and layout: Ben, decisions log 2026-10-04 ("We'll proceed w/ Petroff–Guyou").
- Projecting only at the bake: Ben, 2026-10-03 ("Bake only").
- The world size at today's blocks per km² (not the prototype's 16,384 × 8,192): agent, 2026-10-04,
  in STATUS "Pending questions".
- λc search, border inset, edge blend, footprint: agent, 2026-10-04 (SCL-087 to SCL-090).
- The legal-axis guard for bodies at the pole row (found as a dead bake on the Earth-proportioned test planet of the Phase 1 exit, 1,221 m per block): agent, 2026-10-05 (SCL-091).
- The game side of the border (warning band, spectators, Sable hulls, the spawn margin) and turning
  the pole cap off on this world: agent, 2026-10-04 (POL-022 to POL-024).
- The same bits in Rust, Java and the pages: every projection call runs on the portable trigonometry
  (`deeptime_format::portable_math`, Java `PortableMath`, JS `DTMath`), so the bake, the game and the
  workbench agree on every block's latitude bit for bit (`ProjectionParityTest`,
  `tools/viewer/projection-check.js`).

## Open questions

- The world size (above).
- Whether the cone points should also avoid the poles' neighbourhood (they cannot: both lie on the
  same edges by construction).

## Sources

- `petroff2021` (Petroff, M. A. 2021, ACM TSAS 7(4):18; code Zenodo 4661351): read in full (paper by a
  sub-agent, 2026-10-03; code by the implementing agent, 2026-10-04).
- The projection survey, `docs/research/notes/seamless-projection-survey.md`, for Guyou's aspect,
  the shape statistics and the layout.
