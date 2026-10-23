# The simulation's cube grid: what S1 and S2 built, and what S3 to S6 and the bake need

Written 2026-10-22 by the agent of the cube world plan's phase 4, items S1 and S2
(`docs/plans/cube-world-plan.md` §3; branch `worktree-agent-a66447186bac217ed`). The science and the
measured checks are on [the science page](../science/cube-simulation-grid.md); the ledger rows are
SCL-140 to SCL-152. This note is the interface: what exists, and what each later item needs from it.
Everything here can be changed until a later item builds on it; the recommendations are the
agent's, not decisions.

## 1. What exists

### The grid (`crates/deeptime-core/src/sphere/cube.rs`)

- `CubeTopology::new(n)`: every cell's eight neighbours in the D8 order of
  `surface::grid::OFFSETS` (E, NE, N, NW, W, SW, S, SE in the cell's face frame), `NO_CELL` on the
  24 corner cells' missing diagonal. `index(face, i, j)`, `coords(c)`, `slots(c)`, `slot_of(c, m)`.
  Cell (face, i, j) is `face · n² + j · n + i`: the format's cube-raster order.
- `CubeGrid::new(n, radius_m)` on `CubeMap::simulation()`, or `CubeGrid::with_map(map, n, radius_m)`
  on any map of the slot. Per cell: `unit(c)`, `lat_deg(c)`, `lon_deg(c)`, `bearing(c)` (the face's
  up, radians clockwise from north), `areas()`, `dists(c)` (the eight great-circle distances, m) and
  `lap(c)` (the eight Laplacian couplings, dimensionless). Totals: `total_area()`, `mean_area()`.
  Points: `map()`, `cell_of(p)` (the cell holding a cube-frame vector) and `point_in(c, s, t)` (a
  point of the cell's own square on the map, s and t in [-1/2, 1/2]).
- `face_sum(n, v)`: a sum in the grid's fixed order (per face row, rows, then faces F0 to S).
- `GhostTable::new(&topo, g)`: for every slot of a face padded by g rings, the cell that fills it.
  `source(face, i, j)` for i, j in [-g, n + g); `fill(field, out)` gathers into padded faces, face
  by face, each row by row. Corner squares take FV3's x-direction rule.
- `hilbert_key(n, face, i, j)` and `hilbert_order(n)`: the face in the top bits, the Hilbert index
  below. Storage stays row-major.

### The solvers on any grid (`surface/grid.rs`'s `SurfaceGrid`)

`cells`, `neighbours(c)` (with distances), `area(c)`, `ref_area()`, `uniform_area()`, `strips(rows)`,
`strip_border(s, out)` and `neighbours_in(c, lo, hi, out)`. `Dims` and `CubeGrid` implement it.
Running on it: `pflood` (serial and parallel), `routing` (receivers, stack, accumulate), `spl`
(`solve`, `solve_cut`, `families`), `sea_level::fill_level_on` and `marine::fill`. On `Dims` each is
the raster's arithmetic bit for bit.

### The cube's own solvers (`surface/cube/`)

- `diffusion::solve(ws, g, h, kd, dt, solver, tol_m)` and `diffusion::step_in` (Jacobi, the cells
  with a coupling only). The operator is `Op` (CSR-like: cells, eight columns and couplings per row,
  mu = a/a_mean, the diagonal); it is `pub(super)` today.
- `multigrid` (private): 2 x 2 blocks of a face per level, the raster's constants, f32.
- `spread::step_in`, `failure::slope_failure`.
- `footprint`: `distance_m`, `cell_m`, `offset` (east, south in the azimuthal equidistant frame at a
  centre), `for_cells`, `cells_within`, `cell_weight`, `render`, `volume_m3`, `sample`,
  `centre_weight`, `ring_base`.
- `ordered_sum(len, f)`, `volume(g, v)`, `CHUNK` (4,096).

### Memory

From the struct sizes, per cell: `CubeGrid` 216 bytes (unit vector 24, latitude, longitude,
bearing and area 32, distances 64, couplings 64, neighbours 32). At n 240 (quick 16k's L1, 345,600
cells) that is 75 MB; at n 480, 299 MB. The map table is shared (0.66 MB). The diffusion workspace
holds the operator (about 124 bytes a cell) and nine f64 vectors (72 bytes a cell), plus the
multigrid's f32 levels. S8's `memory_estimate` should count these per 6n² cells.

### Speed

`crates/deeptime-core/examples/cube_surface_bench.rs` runs one surface step of every ported solver
on a smooth random planet: the cube at n 240 (345,600 cells, quick 16k's L1) against a 1,024 x 338
raster (346,112 cells, the same mean cell area) on the raster's own code, 8 threads, 12 steps of
2 Myr with the first 2 not timed, the fastest of three repeats. On the VM (r4, the final head;
`cargo run --release -p deeptime-core --example cube_surface_bench`), milliseconds per step:

| Phase | Cube | Raster | Cube / raster | Plan |
|---|---|---|---|---|
| Flood (open sea, lakes) | 5.25 | 7.51 | 0.70 | |
| Routing (receivers, stack, drainage area) | 4.10 | 5.14 | 0.80 | 1.0 |
| Stream power | 23.42 | 26.01 | 0.90 | |
| Mouth fill | 9.20 | 363.81 | 0.03 | |
| Diffusion, Jacobi | 14.90 | 6.52 | 2.29 | 1.0 to 1.3 |
| Diffusion, multigrid | 19.29 | 8.90 | 2.17 | 1.0 to 1.3 |
| Slope failure | 13.40 | 9.57 | 1.40 | |
| Spreading | 3.38 | 3.14 | 1.08 | |
| Sea level | 14.33 | 15.04 | 0.95 | |

The ratios move with the VM's load: an earlier run (r3) gave routing 0.96, diffusion 2.65 and 2.53,
slope failure 1.71. The iteration counts match (multigrid 24 on both; Jacobi 84 against 80 over the
12 steps).

- **Diffusion costs about 2.2 to 2.6 times the raster's.** The raster's operator is a 5-point
  stencil, stored as an east and a south coupling per cell, its matrix product fused into strips. The
  cube's P1 operator has eight couplings and eight column indices per cell, about four times the
  memory per matrix product, and the same iteration counts. The plan's 1.0 to 1.3 assumed a 5-point
  stencil. Ways to close part of the gap: store each pair's coupling once (four a cell, by
  symmetry), index a face's inside cells by offset instead of a column table, and hold the couplings
  in f32 (the multigrid already does).
- **The slope failure** skips the cells away from the last iteration's movers once they are a
  quarter of the cells or fewer, as the raster does (it was 10.6 times the raster's before the
  skip).
- **The mouth fill cannot be compared yet.** Both grids run the same `marine::fill`. In the last
  step the cube's 3,875 fills reached 19,288 cells in all (5 a mouth), and the raster's 4,579 fills
  reached 1,819,325 cells (397 a mouth). 1,819,325 is close to 9 x 200,000, the bench's
  `max_cells`, so a few raster fills may run to the cap. The bench's raster is not a sphere: it is a
  rectangle with closed north and south edges, sampled at equal steps of sin(latitude), so its seas
  are not the cube's. This is not explained yet. A diagnostic run (commit 0b925182, reverted after;
  it counts the fills stopped at `max_cells` and the volume delivered) is queued on the VM as
  `cube-bench-r5`, with its output in `/data/ban/deeptime-runs/cube-s1s2/bench-n240-r5.{log,csv}`.
  The totals of the two grids (107 against 446 ms) are therefore not a fair comparison. A step
  with one diffusion solver (multigrid) and without the mouth fill takes 83 ms on the cube against
  75 ms on the raster: 1.10 times.

## 2. What each later item needs

### S3, climate

- **Latitude and the wind's frame.** `lat_deg(c)` replaces `lat_of_row`, `row_weights` and
  `band_weight`. A wind given east and north turns into the face frame by `bearing(c)`; the local
  east and north vectors are `deeptime_format::projection::local_frame(unit(c))`.
- **Zonal sums** (belts, the energy balance's bands): bin cells by `lat_deg` with weight `area`,
  summed with `face_sum` or `ordered_sum`, never a rayon float sum.
- **The coarse grid.** A coarse cube is `CubeGrid::new(n / k, R)` on the same map. Fine to coarse
  is the multigrid's 2 x 2 aggregation applied log2 k times (area-weighted means); coarse to fine is
  the bilinear sampler below. For n 240 and k 8 the coarse side is 30.
- **The upwind budget's `Mesh`.** `climate/upwind.rs` already takes cells and faces. A face needs
  the edge length between two side neighbours, the centre distance (`dists`) and the unit normal in
  each cell's local (east, north) basis. **Missing:** the edge length. It is the arc between the two
  corners the cells share; `CubeGrid::with_map` computes the corners and drops them. Add
  `CubeGrid::edge_m(c, k)` for the four side slots, kept by the lower-indexed cell like the
  distances (32 bytes a cell). The normal is the tangent of the arc from c to m at c, in
  `local_frame(unit(c))`, and at m in `local_frame(unit(m))`.
- **The orographic rain.** Today an FFT in x. On the cube: a convolution per face over a window
  padded by `GhostTable` with g the kernel's half-width in cells (g up to n is allowed). The corner
  squares are FV3 copies, not true geometry, so a kernel that reaches a corner square sees a copied
  cell there; for a smooth kernel that is the usual cubed-sphere compromise.

### S4, flexure

- The biharmonic is the Laplacian twice: (L (D L w)), with L from `lap` and the cell areas. That
  needs `Op` as `pub(crate)` and an apply of two passes (L, scale by D, L) in the same fixed chunks.
  A 25-point stencil on `GhostTable` g 2 is the alternative, but the two-pass form reuses the
  couplings that already pass the harmonic tests.
- The multigrid's 2 x 2 aggregation is the coarse hierarchy; a biharmonic needs a smoother suited
  to it (two Jacobi sweeps per level are too weak for a fourth-order operator). Test against
  Hetényi's plate at a face centre, a seam and a corner: the Laplacian's error is largest at the 24
  corner cells (SCL-141), and a fourth-order operator squares it.

### S5, plates L1, the ice sheet, hydro and deposits

- **A bilinear sampler** (also the bake's step 5, §3 below). For a point p: (face, u, v) =
  `map().inverse(p)`; x = (u + 1) n / 2 - 1/2 and y likewise; i0 = floor(x), j0 = floor(y); the four
  cells are `GhostTable` g 1's `source(face, i0 .. i0 + 1, j0 .. j0 + 1)`, weighted bilinearly in the
  fractions. About 30 lines; not written yet. At a corner square the source is the FV3 copy.
  (`footprint::sample` today reads the holding cell only, on purpose: the volcano shapes measure
  relief at the cell they weight.)
- **Plates L1's advection.** A departure point is the cell's `unit` turned back about the plate's
  Euler pole; read with the sampler (semi-Lagrangian) or `cell_of` (nearest). Both are pure
  functions of the field, so the result does not depend on the thread count.
- **Hydro.** `hydro/mod.rs`'s `neighbour8(w, h, c, k)` becomes `topology().slots(c)[k]` (`NO_CELL`
  for none). The order is the same.
- **The ice sheet.** The shallow-ice diffusivity per cell goes to `diffusion::solve` as `kd`; a
  Picard loop around it for the nonlinearity. The solve keeps Σ a·h to round-off.
- **Deposits forecasts.** Per-cell vectors in the grid's order; latitude from `lat_deg`.

### S6, GPU kernels

- **Layout.** Padded faces, 6 (n + 2g)² values, face by face, each padded face row by row: what
  `GhostTable::fill` writes. One gather kernel per step reads the source table (one u32 per padded
  slot). **Missing:** a `GhostTable::sources()` accessor for the table's bytes.
- **Tiles.** Hilbert keys within a face; a workgroup takes a tile of one face, so its neighbours are
  in the padded face and no kernel needs the adjacency.
- **Reductions.** Per-workgroup partials in tile order, summed per face, then the faces in order
  F0, F1, F2, F3, N, S (on the CPU or in a final pass of one workgroup). The CPU solvers' sums are
  over 4,096-cell chunks; a GPU kernel that must match the CPU bit for bit has to use the same
  chunks.
- **Corners.** A stencil kernel at a corner cell reads the FV3 corner ghost, so its result there
  differs from the CPU's neighbour-table solvers, which skip the missing diagonal. Either the CPU
  path keeps its tables (and GPU parity is to a tolerance at the 24 corner cells), or the kernel
  takes the eight-slot table as a second input.

### S7, determinism

Every cube solver sums in a fixed order. The test
`surface::cube::tests::the_closed_planet_is_the_same_on_any_thread_count` runs 30 steps of the
closed planet (flood, routing, stream power, mouth fill, multigrid diffusion, slope failure,
spreading) on 1, 3 and 8 threads and compares the bits.

## 3. The bake's simulation-grid trait

Plan §2: the bake resamples the simulation to the world, cube to cube, through "the simulation grid
as a trait". What the bake's steps read from the simulation grid, and where the cube already has
it:

| The bake needs | Cube | Lambert today |
|---|---|---|
| the field at a planet point (B1 step 5) | the bilinear sampler (§2 S5, to write) | the bilinear over rows and columns |
| the cell holding a point (B4 categories) | `CubeGrid::cell_of` | the projection's inverse and a floor |
| a cell's size and axes in metres (B2's footprint, J_sim) | `point_in(c, ±1/2, 0)` and `point_in(c, 0, ±1/2)`, times R | `true_cell_m` and the row's latitude |
| cell areas (area-weighted averages) | `areas()` | the row weights |
| the map's frame | `map()` (the polar-axis Moser cube) | the Lambert projection |

A recommended shape, for the bake lane to take or change:

```rust
pub trait SimGrid: Sync {
    fn cells(&self) -> usize;
    /// The cell's centre, a unit vector in the simulation's frame.
    fn unit(&self, c: usize) -> [f64; 3];
    fn area_m2(&self, c: usize) -> f64;
    /// The cell holding p (any length > 0).
    fn cell_of(&self, p: [f64; 3]) -> usize;
    /// A field in the grid's order, read bilinearly at p.
    fn sample(&self, field: &[f32], p: [f64; 3]) -> f32;
    /// The cell's two axes on the sphere, metres per cell (east-ish, south-ish), for J_sim.
    fn axes_m(&self, c: usize) -> ([f64; 3], [f64; 3]);
}
```

The simulation's fields are held in the format's cube-raster order, but the world's cube is turned
by λ0 and drawn on the conformal map, so a field is never written to the world as it is held: the
resample stays (plan §2).

## 4. Open points for the coordinator

- **D8 on the cube.** D8's drainage statistics follow the grid's directions; on the cube the grid
  kinks at a seam, and a cone across a seam drains to a 20 % higher p99 than on a face. The plan's
  2 % cannot hold for D8 (a ridge turned 45° on a plain raster moves the p99 by 40 %). A
  multiple-flow-direction router would be the remedy; it would change Lambert too, so it is a
  separate task.
- **The plan's S2 list** names the young landscape, glacial, thermal, advect, amplify, impacts,
  erosion bands and carbonate. They are not ported: each is written on rows and columns and runs
  later in the surface stage; none is needed by S3 to S5's interfaces above.
- **The volcano stage itself** waits for the plates on the cube (catalogue positions as unit
  vectors); its footprint code has the cube's metric already (`footprint`).
- **`underground.rs`** has no grid metric to port; its `active.json` coordinates are blocks, the
  format and game lane's.
