# The simulation's cube grid and its surface solvers

Status of this page: `stage 1 (written from the ledger, nothing verified)`. Written 2026-10-22 for
the cube world plan's phase 4, items S1 and S2 (`docs/plans/cube-world-plan.md` §3).

Ledger ids on this page: `SCL-140` to `SCL-152` (see [claims.md](claims.md), ledger
[SCL](ledger/SCL.md)). The rules the solvers keep are ledgered with the raster's solvers:
`ERO-169` (slope failure), `ERO-183` (multigrid), `ERO-184` (the parallel flood).

## What you see in the world

Nothing yet. The cube grid is opt-in: no stage, preset or bake uses it, and every world is still
simulated on the Lambert raster. The grid and the solvers on it are the ground work for moving the
simulation onto the cube (the plan's S3 to S9), after which a world's rivers, coasts, sediment and
volcanoes are computed on the same six faces the world is drawn on.

What a player would see once it is used: no seam and no pole. On the raster the simulation's
cells shrink towards the poles in one direction and stretch in the other; on the cube every cell is
within 9 % of the mean area (0.909 to 1.007 of it at 240 cells a side), so a river or a volcano
has the same resolution everywhere.

## The model in plain words

**The grid (S1, `crates/deeptime-core/src/sphere/cube.rs`).** Six faces, F0, F1, F2, F3, N and S,
each n × n cells, stored face by face in the format's order and each face row by row (cell
(face, i, j) is index face·n² + j·n + i). The simulation's map is Moser's equal-area cube
(SCL-081, SCL-102) on the polar axis: +Z is the north pole, +X the point (0°, 0°). Every quantity
comes from points of the map:

- a cell's centre is the map's image of its local centre;
- its corners are the images of the grid's corner points, written with integer numerators so a
  face's symmetries hold bit for bit (SCL-140);
- its area is R² times the solid angle of the spherical quadrilateral of its corners, as two
  triangles (Van Oosterom and Strackee 1983, SCL-140). The six faces tile the sphere, so the areas
  sum to 4πR² to round-off;
- the distance to each of its eight neighbours is the great-circle arc between the centres;
- its latitude, longitude and the bearing of the face's "up" are stored per cell (SCL-140);
- the Laplacian's couplings come from linear finite elements on the quads of four cell centres
  around each grid vertex, both diagonal cuts averaged, so each cell couples to its four sides and
  its four diagonals (the cotangent weights; Meyer, Desbrun, Schröder and Barr 2003; SCL-141).

Each cell's neighbours are listed in the raster's D8 order (east, north-east, north, north-west,
west, south-west, south, south-east, in the cell's own face frame). Across a face edge the
neighbour lies on the next face. The 24 cells at the cube's eight corners have seven neighbours:
the diagonal step across a corner lands on no cell. Each pair's distance and coupling is computed
once, by the lower-indexed cell, so both ends hold the same bits.

A ghost table pads a face with g rings of copies of its neighbours' cells. A ghost beyond one
edge is the neighbour face's cell. A ghost beyond two edges (the corner squares, where three faces
meet) takes the x-direction neighbour's cell turned about the corner, FV3's rule (Putman and Lin
2007; Harris et al. 2021; SCL-142). Hilbert keys order each face's cells along a Hilbert curve for
tiling (SCL-143); storage stays row-major.

**The solvers (S2).** The surface's graph solvers take any grid with a neighbour function, cell
areas and strips (the `SurfaceGrid` trait, `surface/grid.rs`). The raster (`Dims`) and the cube
(`CubeGrid`) both implement it, and on the raster every method is the raster's own arithmetic, so
the raster's results are bit for bit what they were. Running as they are on the cube:

- the depression flood, serial and parallel (`pflood.rs`; ERO-184's method);
- the D8 routing: receivers, the stack, the drainage area (`routing.rs`);
- the stream-power solve (`spl.rs`): the cube's cells carry their own areas, and a lake's room is
  counted in metres over cells of the mean area (SCL-149);
- the sea level from the ocean's volume (`sea_level.rs`): true cell areas, summed in cell order
  (SCL-150);
- the mouth fill (`marine::fill`): true areas, the last ring's layer spread over the area the fill
  visited (SCL-151).

New for the cube, because their raster versions were written on rows and columns
(`surface/cube/`):

- the implicit diffusion (backward Euler with the coupling Laplacian, SCL-144), its first guess
  (rows then columns of each face, SCL-145), conjugate gradients with the diagonal or a multigrid
  V-cycle as the preconditioner (SCL-146), and the residual's volume put back so Σ a·h is kept to
  round-off (SCL-145);
- the crust's spreading (`spread.rs`'s law on the cube's solve);
- the submarine slope failure (ERO-169's rule, pair volumes on true areas, SCL-148);
- round shapes on the sphere for the volcanoes (SCL-152): a cell's distance from a volcano is the
  great-circle arc, its offset is in the azimuthal equidistant frame at the volcano, and a cell's
  weight is the profile's mean over 4 × 4 points of the cell's own square on the map. On the
  raster the same code measures distances in block metric (block offsets times metres per block).

Every sum is taken in a fixed order (SCL-147): the solvers' sums over fixed chunks of 4,096 cells
in ascending order, the chunk sums added in order; the grid's totals per face row, then the rows,
then the faces. No result depends on the thread count. Cells are stored face by face, so the
order is per face first and then F0, F1, F2, F3, N, S.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-140 | cell area | R² (Ω(q00, q10, q11) + Ω(q00, q11, q01)), tan(Ω/2) = \|a·(b×c)\| / (1 + a·b + b·c + c·a) | m² | Van Oosterom and Strackee 1983 | `from-memory`, checked numerically |
| SCL-141 | Laplacian couplings | w_side, w_diag from ½ cot of the opposite angles, both cuts of each quad averaged; at a corner the triangle of three centres whole | dimensionless | Meyer et al. 2003 (the P1 cotangent stiffness) | `from-memory`, checked numerically |
| SCL-142 | corner ghosts | the x-direction neighbour's cell turned about the corner | - | Putman and Lin 2007; Harris et al. 2021 (FV3 `fill_corners`) | `from-memory` |
| SCL-143 | tiling order | a Hilbert curve within each face, the face in the top bits | - | Hilbert 1891 | `game-choice` |
| SCL-144 | diffusion system | (μ_c + Σ K_cm) x_c − Σ K_cm x_m = μ_c b_c, μ_c = a_c/ā, K_cm = Δt D_cm w_cm / ā; side D the mean of the two cells, diagonal D the least of the quad's four side means | - | the raster's scheme (ERO-183) | `game-choice` |
| SCL-145 | first guess and put-back | rows then columns per face, couplings across edges at the old values; Δx_c = k s_c / μ_c with s_c = Σ_m K_cm and k = lost / Σ s | - | the raster's scheme | `game-choice` |
| SCL-146 | the cube's multigrid | 2 × 2 blocks per face (side ⌈n/2⌉), coarse couplings SCALE × the summed fine ones, damped Jacobi, the raster's constants | - | Braess 1995; ERO-183 | `game-choice` |
| SCL-147 | ordered sums | chunks of 4,096 cells, the chunk sums added in order | cells | - | `game-choice` |
| SCL-148 | slope failure pairs | volume rate · excess · ½(a_c + a_m), each end over its own area | m³ | ERO-169's rule | `game-choice` |
| SCL-149 | lake room | metres × cells of the mean area (`SurfaceGrid::ref_area`) | m | - | `game-choice` |
| SCL-150 | sea level | the raster's fill (ISO-086) with true areas, summed in cell order | m | ISO-086 | `game-choice` |
| SCL-151 | mouth fill | true areas; the last ring's layer spread over the visited area | m | the raster's fill | `game-choice` |
| SCL-152 | volcano footprint | great-circle metres, azimuthal equidistant offsets, 4 × 4 points per cell on the map; the walk looks 2.5 mean cell sides past the reach | m | - | `game-choice` |

Where:
- a, b, c are unit vectors, Ω a triangle's solid angle, q the cell's corners;
- a_c is cell c's area and ā the mean, x the new heights and b the old;
- D_cm is the pair's diffusivity (m²/yr), w_cm the coupling, Δt the step (yr).

## Earth against the mod

There is no Earth figure to compare against: the grid and its solvers are numerics. These are the
measured properties of the code, from `cargo test -p deeptime-core --release --lib -- --nocapture
cube` on the test VM, 2026-10-22, branch `worktree-agent-a66447186bac217ed`.

**The grid (S1).**

| Check | Result |
|---|---|
| Areas sum to 4πR² (relative error) | 2.45e-16 at n 16, 1.23e-16 at n 64, 2.45e-16 at n 240 |
| Cell area against the mean, n 240 | 0.909 to 1.0065 |
| Side couplings (n 64) | all positive, least 0.706; the largest diagonal 0.142 |
| Neighbour table against the format's `cell_on` | equal, every cell, n 2, 3, 5, 8, 16, 48 |
| Ghost fill against a direct fold | equal on every ghost |
| Laplacian of Y₁ and Y₂ (relative RMS error, n 16, 32, 64) | Y₁ 1.40e-2, 7.13e-3, 3.62e-3; Y₂ 1.84e-2, 9.38e-3, 4.76e-3 |

The Laplacian's error halves when n doubles (first order), and its largest error, 5.7 % for Y₁ and
7.4 % for Y₂, sits at the 24 corner cells; away from them it is about 2 %. The corner cells carry
the RMS, so the RMS converges at first order, not the second order a smooth grid gives.

**The solvers (S2), n 96 (cells about 104 km across).** The same feature placed at a face's
centre, on the middle of the F0 | F1 seam and on the F0, F1, N corner:

| Check | Face, seam, corner | Largest deviation |
|---|---|---|
| Flood: a bowl's lake volume | 15,430,506, 15,434,392, 15,429,405 km³ | 0.025 % |
| Sea level over a basin 4 km deep | −1,340.2, −1,340.3, −1,340.0 m | 0.1 m |
| Multigrid diffusion, a cone after three steps: peak | 2,315.32, 2,315.21, 2,313.37 m | 0.084 % (spread 0.013 %) |
| The same with the diagonal preconditioner | identical | identical |
| Iterations, multigrid against Jacobi | 9, 13, 15 against 129, 150, 159 | |
| Diffusion on a spherical harmonic (step error, n 16, 32, 64) | 8.06e-4, 2.08e-4, 5.27e-5 | second order |
| Spreading: volume moved | 16,388, 16,385, 16,202 km³ | 1.13 % (peak 0.026 %) |
| Mouth fill: area covered | 765,611, 765,677, 772,703 km² | 0.93 % (thickest 0.24 %) |
| Slope failure, a pile 3 km high and 15° across: top | 2,662.6, 2,637.0, 2,723.9 m | 2.30 % |
| Slope failure: volume moved on the way | 246,760, 259,387, 170,941 km³ | 30.7 % |
| Stream power, a cone 3 km high: eroded, G 0 | 1,238,830, 1,257,349, 1,324,567 km³ | 6.92 % |
| Stream power, G 1 | 746,068, 760,887, 799,326 km³ | 7.14 % |
| D8 routing, cone: drainage area p99 | 230,607, 276,766, 276,817 km² | 20.0 % |
| D8 routing, ridge: p99 | 92,243, 73,803, 119,967 km² | 30.1 % |
| D8 routing, ridge turned 90°: p99 | 92,243, 101,475, 119,954 km² | 30.0 % |
| A shield of 600 km basal radius: rendered volume against the exact cap integral | 0.002 % error at the face | 0.45 % |

The raster control (`raster_control` in `surface/cube/tests.rs`): the same features on a uniform
raster of cells of the cube's mean area. A cone with its peak on a cell centre or on a cell corner
gives the same p99 (230,606 km² both). A ridge struck at 0°, 30° and 45° to the raster gives
92,243, 129,140 and 129,140 km²: **turning a ridge on a raster changes D8's drainage p99 by 40 %**.

Conservation on a closed planet (n 32, 1,000 steps of the open sea, the flood, the routing, the
stream-power solve with G 1, the mouth fill, the multigrid diffusion, the slope failure and the
spreading): Σ a·h drifted by 3.57e-15 of Σ a·|h|, the worst single step by 1.9e-15; the water the
roots collect was the planet's area to 6.6e-15 in every step (4,301,227 lake cell-steps, no sinks).

## Simplifications and limits

- **D8 follows the grid's directions.** Single-direction routing sends each cell's water to one of
  eight neighbours, so drainage patterns follow the grid's lines. On the cube the grid lines kink at
  a seam and meet three to a corner, so flow lines merge along a seam: a cone straddling a seam has a
  20 % higher drainage p99 than on a face, and the stream-power erosion that follows the drainage
  area 7 % more. That is within D8's own anisotropy on any raster (40 % for a ridge turned 45°), but
  it is a pattern tied to the cube: seams attract rivers. The plan's 2 % target for the drainage p99
  cannot hold for D8; a multiple-flow-direction router would be the remedy (Open questions).
- **The slope failure is a pairwise eight-neighbour rule.** Near a corner the cells are skewed
  (120° corners; one diagonal is about as short as a side), so the same rule moves a different
  volume on the way (31 % less at the corner); the pile ends at the same top within 2.3 %.
- **First order at the corners.** The Laplacian's error at the 24 corner cells does not fall
  faster than the cell size; everywhere else it does (second order). The diffusion step on a
  smooth field converges at second order in the RMS, because the corner cells are few.
- **Cells are not exactly equal.** The simulation's map is close to equal-area, not exactly: its
  cells range 0.909 to 1.007 of the mean at n 240. Every solver uses each cell's own area, so this
  costs no conservation.
- **Memory.** A `CubeGrid` holds about 216 bytes per cell (unit vector, latitude, longitude,
  bearing, area, eight distances, eight couplings, eight neighbours): about 75 MB at n 240, quick
  16k's L1. The map table it shares is 0.66 MB.
- **What is not on the cube yet.** The young landscape, the glacial and thermal passes, the
  advection and the amplifier (window and tiles), the impacts, the erosion bands, the carbonate and
  the volcano stage itself; only the volcanoes' footprint code has a cube-metric version.

## Game choices

- The diagonal pair's diffusivity (the least of the quad's side means, SCL-144) keeps the matrix
  symmetric positive definite where diffusivities jump; the side pair keeps the raster's mean.
  Chosen by the S2 agent, build log 2026-10-22.
- The first guess runs only when every cell takes part (SCL-145); a solve on a few cells (the crust's
  spreading) goes straight to the conjugate gradients. Build log 2026-10-22.
- The slope failure visits every cell each iteration; the raster's neighbourhood skipping is a
  speed-up of the same result and is not ported (SCL-148). Build log 2026-10-22.
- Hilbert keys with row-major storage (SCL-143): the keys give tiles, the storage keeps the format's
  order.

## Open questions

- D8 against multiple-flow-direction routing on the cube (a coordinator's call): D8's seam bias is
  measured above. MFD or D∞ would spread a cell's water over its downhill neighbours in proportion
  to the slopes and take most of the grid's direction out of the drainage.
- The plan's 2 % rotation target holds for the flood, the sea level, the diffusion, the spreading
  and the mouth fill, and not for D8's drainage statistics, the stream power that follows them, or
  the slope failure's volume moved (the table above).
- None of SCL-140 to SCL-142's sources has been read for this work; the formulas were written from
  memory and checked numerically (the areas' sum, the harmonics, the neighbour and ghost tests).

## Sources

- [VanOosterom1983]: from memory, not fetched. The formula is checked by the areas' sum.
- [Meyer2003]: from memory, not fetched. The weights are checked by the harmonics.
- [PutmanLin2007], [Harris2021FV3]: read for the sphere-native grids note (2026-10-04, by a
  sub-agent), not re-read; the corner fill follows FV3's `fill_corners` as that note describes it.
- [Hilbert1891]: from memory, not fetched (the curve itself; the code is the standard iterative
  index-to-curve map).
- [Braess1995]: as for ERO-183.
