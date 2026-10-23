# The seamless world: Petroff's equal-area map in the Guyou layout

Status of this page: `stage 2 (written with the code, 2026-10-04; Petroff's paper and code read in
full, the map's properties tested; the layout and its sizes are game choices; the forward's
degenerate lines fixed 2026-10-20)`. The section "The cube world" at the end: `stage 1 (written
with phase 0's code, 2026-10-22; the map's sources not read, its properties measured)`.

Ledger ids on this page: `SCL-049`, `SCL-083`, `SCL-084`, `SCL-086` to `SCL-091`, `POL-022` to
`POL-024`; the cube world: `SCL-078` to `SCL-081`, `SCL-100` to `SCL-108` (see
[claims.md](claims.md)).

Ben moved the world to a cube on 2026-10-22 (decisions log 2026-10-22; the plan is
`docs/plans/cube-world-plan.md`). Everything above the section "The cube world" describes the world
the game plays today, format 0.5 and 0.6. That section describes what phase 0 of the move has coded:
the cube's geometry and its two maps. No bake writes a cube world yet; this page is rewritten for the
cube when the bake and the game move (the plan's D1).

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
tile centres and the poles (10,812 inputs a world) comes back within 1e-5° in five worlds
(`exact_inputs_on_the_diagonals_round_trip`; four in Java's
`ProjectionParityTest.exactInputsOnTheDiagonalsRoundTrip`). The worst is 4.3e-7° (0.02 blocks),
which is the inverse's own rounding on the diagonals: the forward is within 1.4e-7 blocks of the
reference there. The inverse is unchanged.

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

## The cube world (format 0.7, coming; phase 0 coded 2026-10-22)

### What it will look like

The planet is drawn on the six faces of a cube, each a square of s blocks: s = 1,024 for a 4k planet,
3,840 for a 16k one and 7,424 for a 32k one (SCL-078, SCL-100). Six faces of side s carry the
planet's area, so a block is nominally 276.4 m (4k), 294.82 m (16k) or 305.0 m (32k) on a side
(SCL-107), against 294.6 m and 300.4 m on today's world. The faces are laid out as a cross: four in a
band from west to east (F3, F0, F1, F2, with F0 centred on x = 0), N above F0 and S below it. The
band wraps east to west with a period of 4s (15,360 blocks at 16k), as Toroidal World wraps today;
the whole net fits in a window 4s wide and 3s tall (SCL-079).

Inside a face, and across the four seams that are straight in the net (F3 to F0 to F1 to F2, N and S
to F0), the world is seamless. The other edges of N and S meet faces the net draws elsewhere: N's
east edge is F1's top turned a quarter turn about F0's corner, N's far edge is F2's top turned a
half turn, and likewise for S. Those seams are glued by a motion of the world (a quarter turn about
one of F0's corners or a half turn about the centre of the empty square above or below F1). Until
the wrap engine joins them, a player meets a border there; the ground seen past it is the planet
the seam leads to, drawn up to one face width out, and anything further, or in no band, is deep
ocean (SCL-108).

The world's map is conformal: at every block the planet is drawn with the same scale in every
direction, so a round lake is round and a mountain range keeps its proportions (the shape ratio is
at most 1.0011 in the table). Area is not kept: a block at a face's centre covers about 1.27 times
the mean planet area and a block very near one of the cube's eight corners much less (in the corner
cell of the table the planet is drawn 25.6 times larger in area, 5 times in length). The corners are
cone points: three faces meet there, so a full turn round the planet is drawn in three quarters of a
turn of world. Four corners (F0's) lie inside the net and can be walked to; the other four lie on
the border. Where the poles lie depends on a turn of the planet inside the cube (a free rotation,
SCL-079) that the bake will choose so the corners sit far out at sea; that search is not coded yet,
so nothing here says where north is. Latitude and longitude come only from the map and that turn,
never from x and z alone.

### The model in plain words

1. **The face side.** s = C / √(6π) for a circumference of C blocks, so that 6s² = 4πR² at today's
   blocks per km², rounded to the nearest multiple of 256 blocks: every grid cell up to 256 blocks
   divides it, the period 4s is a whole number of region files, the Nether's 1:8 loop stays whole
   and every turn pivot sits on a chunk corner (SCL-078, SCL-100).
2. **The cube's frames.** Each face has a centre direction and two local axes in the cube frame:
   F0 (+X, +Y, −Z), F1 (+Y, −X, −Z), F2 (−X, −Y, −Z), F3 (−Y, +X, −Z), N (+Z, +Y, +X),
   S (−Z, +Y, −X). A direction belongs to the face of its largest component; a world block's face
   and local (u, v) follow from the net (SCL-079).
3. **The two maps.** A map takes a face's local square to the planet. The world's map is the
   conformal cube (SCL-080, SCL-101): one 48th of a face, the triangle from the face centre to an
   edge midpoint to a corner, goes conformally to the matching spherical triangle by Schwarz
   triangle functions. The simulation's map, for when the simulation moves onto the cube (the plan's
   phase 4), is Moser's equal-area cube (SCL-081, SCL-102): the conformal map pushed by a flow that
   evens out the area per block. The two maps are the two ends of one family, τ = 0 (conformal) and
   τ = 1 (equal area); the study chose τ = 0 for the world and τ = 1 for the simulation.
4. **The tables.** Neither map has a closed form fast enough for every block, so each is computed
   once by `tools/cube-map/cube_map.py` and stored as a table over one eighth of a face (the cube's
   symmetry gives the rest exactly): the planet point, in the face frame's gnomonic coordinates, at
   the nodes of a triangle 257 nodes a side (513 nodes across a face), and the reverse table on a
   triangle 129 nodes a side. The
   conformal map behaves like a 4/3 power at a corner, so its table adds a finer sub-table over the
   last 4 cells by the corner, 16 times finer (SCL-104). Both files ship in the code (Rust, the jar,
   the viewer's bundles); a world's manifest will name its map and the file's sha256.
5. **Reading a table.** The forward (face and local point to planet) is a Catmull-Rom bicubic over
   the nodes, padded with a ring of ghost nodes that the cube's symmetry gives exactly, so two faces
   compute the same bits on their shared edge (SCL-105); near a corner it blends into the fine
   sub-table over one cell (SCL-106). The inverse (planet to face) starts from the reverse table and
   takes four Newton steps on the forward (SCL-103). The Jacobian is the bicubic's own derivative:
   a 2 × 2 matrix of metres east and north per block of +x and +z, from which come the local size
   σ = √|det| (metres per block at that block) and the angle of +x from east (SCL-107).
6. **The turn.** A 3 × 3 rotation takes the planet's frame to the cube's (the manifest's
   `projection.rotation`, replacing Petroff-Guyou's single longitude). With the identity the poles
   are at the centres of N and S and the corners at ±35.26° latitude; the world's own turn is free.
7. **The same bits everywhere.** Rust (`deeptime_format::cube`, `cube_map`), Java
   (`worlddata/CubeLayout`, `CubeTurn`, `CubeMap`, `CubeWorld`) and the pages (`projection.js`:
   `DTCube`, `DTCubeTurn`, `DTCubeMap`, `DTCubeWorld`) read the same table in the same order with
   the portable trigonometry, and a shared fixture checks that they agree bit for bit; a Python twin
   of the reader is what the generator measures with.

### Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-078 | face side | s = 256·round(C / (256 √(6π))): 1,024 (4k), 3,840 (16k), 7,424 (32k) | blocks | none | `game-choice` |
| SCL-079 | net and turn | the cross, the frames above, 7 seams (1 translation, 4 quarter turns, 2 half turns), a free 3 × 3 turn | dimensionless | geometry | `game-choice`; the turn's search `not-yet-coded` |
| SCL-080 | conformal cube | Schwarz triangle functions, angles π(1/4, 1/2, 1/4) flat and π(1/4, 1/2, 1/3) spherical | dimensionless | Lee 1976; Rančić et al. 1996 (neither read) | `from-memory`, checked numerically |
| SCL-081 | Moser's flow | Δu = ρ − 1, dx/dt = −∇u / ((1 − t) + tρ), RK4, 400 steps, 512² grid | dimensionless | Moser 1965 (from memory) | `game-choice` |
| SCL-100 | face quantum | 256 | blocks | none | `game-choice` |
| SCL-101 | world table | conformal, m = 256 (513 nodes a face), q = 128, corner patch K = 4, F = 16; 701,168 bytes, sha256 c9dd9fd4… | dimensionless | none | `game-choice` |
| SCL-102 | simulation table | Moser τ = 1, m = 256, q = 128, 400 flow steps, no patch; 664,720 bytes, sha256 000e6532… | dimensionless | none | `game-choice` |
| SCL-103 | inverse | bilinear start, 4 Newton steps, clamps into the triangle | steps | none | `game-choice` |
| SCL-104 | corner patch | the last K = 4 cells, F = 16 fine cells a cell | cells | none | `game-choice` |
| SCL-105 | interpolation | Catmull-Rom bicubic, one ghost ring: beyond u = 1 the mirror (a, b) → (1/a, b/a) | dimensionless | none | `game-choice` |
| SCL-106 | patch blend | s = w²(3 − 2w) over one coarse cell | dimensionless | none | `game-choice` |
| SCL-107 | nominal size | m_per_block_h = √(4πR² / 6s²); R_m = m_per_block_h · s √(3 / 2π); local σ = √(abs(det J)) | m per block | none | `game-choice` |
| SCL-108 | outside the planet | the fold: one face width deep, nearer edge first, else deep ocean | blocks | none | `game-choice` |

The forward on a face, with (u, v) the block's local position in [−1, 1]² and (a, b) the table's
gnomonic coordinates: the planet direction in the face frame is (1, a, b) / √(1 + a² + b²), turned
into the cube frame by the face's frame and into the planet's by the transpose of the turn.

### The map against the planet

Not a model of Earth: these are properties of the two maps, measured on the tables by
`tools/cube-map/check.py` (on the test VM, 2026-10-22; the reports are
`review/out/cube-map/check-conformal-n513.json` and `check-moser-t1-n513.json`) and by the Rust
tests (`cargo test --release -p deeptime-format cube`, on the test VM, 2026-10-22).

| Quantity | Conformal (world) | Moser (simulation) |
|---|---|---|
| table against one twice as fine (n 1,025) | 7.0e-7 rad (0.0036 blocks at 32k); order 1.56 | 5.5e-5 rad (0.28 blocks at 32k); order 2.49 |
| shape ratio (long over short axis) | at most 1.0011; 2ω at most 0.063° | median 1.264, at most 1.417 |
| blocks per planet area, over the mean | 0.79 to 2.17 (0th to 99th percentile), 25.6 at the corner | coefficient of variation 0.00052; 0.928 to 1.0057 at the corner |
| determinant of the table's map, least | 0.0072 (at the corner; 0.0072 over 80,200 points within one block of it at 32k) | 0.52 |
| derivative jump across a face edge | at most 0.0034 | 1.9e-4 beyond 4 cells of a corner, 0.056 beyond 1, 0.77 in the corner cell |
| round trip, 10⁶ random points (Rust) | 6.7e-10 blocks at 32k | 4.5e-12 blocks at 32k |
| the two faces on a shared edge | 2.8e-16 rad apart | 2.2e-16 rad apart |
| Jacobian against finite differences | 2.6e-8 | 2.7e-8 |
| one forward (Rust, test VM) | 30.0 ns random, 26.1 ns in raster order | 29.9 ns, 26.5 ns |

An inverse takes about 206 ns and a Jacobian about 50 ns. The round trips need four Newton steps:
on the conformal table two steps leave 0.0049 blocks at 32k on the triangle and 0.029 within 100
blocks of a corner, three 6.3e-7 and 7.9e-5, four 3.3e-12 and 7.3e-10 (SCL-103). The conformal table
integrates to 1.00000004 of a sixth of the sphere per face. The plan's D1 also
asks for the land share on the cube against the planet, the corners' distances to land and the area
and shape statistics on land; those need a bake that writes the cube and the turn's search (the
plan's B1 to B3), so they are not measured yet. The study measured them on its own prototype
(`docs/research/notes/mountains-and-the-projection.md` §5: land drawn up to 1.92 times its true area
on the conformal cube, the corners 5.3° or more out at sea).

### The seams joined (the wrap engine; dev-only, E1 and E2 coded 2026-10-22)

The wrap engine (`docs/design/wrap-engine.md` §12) joins the twelve turning net edges so that a
player walks across them as across any other seam. It is behind `-Ddeeptime.wrapengine=true` and
installs a cube only with `-Ddeeptime.wrapengine.cube=<s>`; nothing in the shipped game reads it yet.

- **A seam is a motion of blocks.** Each turning seam glues two edges by a grid isometry (a quarter
  or half turn and a translation), so a block across the seam is an ordinary neighbour of the same
  size, and the conformal map's two faces agree on the shared edge with a matching derivative (the
  measurements above). Nothing about the metric jumps (SCL-184).
- **Toroidal World keeps the east-west slide**; the engine's folds land in its window, with the slide
  composed in where a seam crosses the window's end (SCL-180).
- **Bands overlap at F0's corners.** There a block takes the edge it lies least deep beyond, the same
  rule as the generator's fold, so each corner square splits on its diagonal (SCL-181).
- **The corners stay cone points.** Three faces meet at each, so a loop round one turns a heading by
  a quarter turn and no gluing makes it flat; seam links within 2 blocks of a corner are refused
  (SCL-182, SCL-183).
- **The band is D = 32 blocks deep** (two chunks; SCL-185).
- **Block entities show across the seam.** A chest, a sign's text, an end gateway's exit or a
  suspicious block's brushing side shows in the band where it is drawn, its positions moved and its
  directions turned by the seam's motion. Reach and the side of a sign are measured from the copy
  nearest the player (SCL-186).

| Ledger id | Quantity | As coded | Status |
|---|---|---|---|
| SCL-180 | seams and motions in TW's frame | 12 edges; F2-top/bottom cut at x = 2s, N-top/S-bottom at x = 0 | `game-choice` |
| SCL-181 | fold on overlapping bands | least deep edge, ties to the earlier; split chunks; convex corners extended by D | `game-choice` |
| SCL-182 | corners | 8 cone points of 270°, holonomy a quarter turn | derived |
| SCL-183 | corner keep-out | K = 2 blocks | `game-choice` |
| SCL-184 | metric across a seam | grid isometry; faces 2.8e-16 rad apart, derivative jump at most 0.0034 | derived |
| SCL-185 | band depth | D = 32 (16 in the GameTests) | `game-choice` |
| SCL-186 | block-entity data in the band | positions move, directions turn; reach from the nearest image | `game-choice` |

### Simplifications and limits

- **Area near the corners.** The conformal map keeps shapes by giving up area: around each of the
  eight corners the planet is drawn ever larger (25.6 times in area in the table's corner cell, about
  5 times in length), so a fixed number of metres is ever more blocks there. Every conversion from
  metres to blocks has to use the local σ (SCL-107); the bake's use of it is phase 1, not coded.
- **The corners themselves.** A full turn round a corner is three quarters of a turn of world;
  walking round one is shorter than round anywhere else, and the map's derivative goes to zero at
  the point. The turn's search is to keep all eight at sea.
- **The simulation's table at the corners.** Moser's table has no corner patch: its derivative
  jumps by 0.77 across an edge in the one table cell (15 blocks at 16k) touching a corner, 0.056 a
  cell further and 1.9e-4 beyond four cells. Nothing reads it until the simulation moves (phase 4),
  which may want a patch like the world's.
- **Finite tables.** Both maps are tables, good to 0.0036 blocks (world) and 0.28 blocks
  (simulation) at 32k against tables twice as fine; the world's limit is the corner's 4/3 power,
  which the patch tames but does not remove.
- **Borders.** Until the wrap engine joins the turning seams, the net's twelve outer edges are
  borders (N's and S's three outer edges, the tops and bottoms of F1, F2 and F3), and four corners
  lie on them; the border's detours round the empty squares are game work (the plan's G-items), not
  coded. The wrap engine joins all twelve when it is on (above; dev-only), and the border, when
  phase 2 builds it, leaves the joined edges out (`WrapEngine.joinedEdges`).
- **Unread sources.** The conformal map's formulas were written from memory and checked numerically
  (SCL-080); neither Lee 1976 nor Rančić, Purser and Mesinger 1996 was read.

### Game choices

- Moving the world to a cube: Ben, decisions log 2026-10-22.
- The face side, its 256-block quantum, the net, the fold's bands: the coordinator's call
  2026-10-22 under Ben's rule of 2026-10-16, following the plan §1.1 to §1.4 (SCL-078, SCL-100,
  SCL-108).
- The conformal map for the world, Moser's for the simulation, and the free turn: the study's pick,
  taken as the coordinator's call 2026-10-22 until Ben answers the plan's question Q4 (SCL-079,
  SCL-101, SCL-102).
- The table's resolution, the corner patch, the interpolation and its ghosts, the blend and four
  Newton steps where the plan said two: the phase 0 agent, 2026-10-22, from the measurements above
  (SCL-103 to SCL-106).
- The format 0.7 layout (rasters face by face, the table shipped in the code and named by sha256 in
  the manifest): written as schema only in `FORMAT.md`, following question Q1's recommended answer,
  which Ben has not given yet.

### Open questions

- Q1 (the format) and Q4 (the map and the turn) wait for Ben (plan §8).
- Whether a raster cell sits at a block's centre or its corner on the cube (left to the bake's B1).
- Whether the simulation's table needs a corner patch (above).
- The land, corner and area numbers on a real bake (above).

## Sources

- `petroff2021` (Petroff, M. A. 2021, ACM TSAS 7(4):18; code Zenodo 4661351): read in full (paper by a
  sub-agent, 2026-10-03; code by the implementing agent, 2026-10-04). `new_projection.py` re-read
  2026-10-20 at the forward's last step (the `alphaprime` lines and their divide-by-zero fallback);
  the paper was not re-read for that fix, and the coordinate form is geometry of the code's own
  quantities, checked numerically (above), not a formula from the paper.
- The projection survey, `docs/research/notes/seamless-projection-survey.md`, for Guyou's aspect,
  the shape statistics and the layout.

For the cube world:

- `Lee1976` (conformal projections from elliptic functions, the conformal cube): not fetched;
  catalogue entry only.
- `Rancic1996` (the conformal cubed sphere): not read (paywalled; a sub-agent tried, 2026-10-03).
- `Moser1965` and `DacorognaMoser1990` (the flow that makes a map equal-area): from memory.
- `PutmanLin2007` (finite volume on cubed spheres, the corner cells): read as extracted text by a
  sub-agent, 2026-10-04, for the sphere-native grids note; not re-read for this page.
- `Ronchi1996` (the gnomonic cubed sphere): abstract only. `Sadourny1972` (the first cubed sphere):
  not fetched.
- `RoscaPlonka2011` (QSC, the equal-area cube kept as the fallback, SCL-082): the preprint read in
  full by a sub-agent, 2026-10-03.
- The mountains-and-the-projection study, `docs/research/notes/mountains-and-the-projection.md` §5,
  for the choice of maps, the turn and the face size; and the plan, `docs/plans/cube-world-plan.md`.
