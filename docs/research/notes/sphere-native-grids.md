# Sphere maths without losing speed: sphere-native grids for the climate and surface solvers

Researched 2026-10-04 for Deep Time, after Ben's request in the decisions log (2026-10-04): "I feel like someone has
probably done research on fixing this exact problem, go send an agent at it, see if there is a way to be doing this
kind of maths on non-euclidian sphereical geometry without performance loss, also look for any similar research that
might have solved the problem." It changes no shipped code. The benchmark behind its numbers is
`tools/research/sphere-grid-bench/`.

**Tags.** **[C]** measured or computed by me with the benchmark on the test VM (§7 and §12 say how); **[V]** read by
me in the source; **[SA]** read in the source by a Sonnet sub-agent this session, its reading recorded in §13;
**[I]** my inference, reasoning given; **[M]** from memory, not checked.

---

## 0. The answer in plain words

- **Yes, this is a solved problem, and the solution is not an unstructured mesh.** Weather and climate models met
  exactly this "pole problem" of latitude–longitude grids decades ago. The models that care most about speed store
  the sphere as **a few patches, each an ordinary rectangular array**: six cube faces (GFDL's FV3), ten icosahedral
  rhombi (NICAM, and GME before ICON), or twelve HEALPix base pixels. Inside a patch every stencil is the same
  fixed-offset raster loop as today's; only a one-cell ring of "ghost" cells round each patch is copied from the
  neighbouring patch before each step. [SA, V]
- **Measured on Deep Time's own L1 cell count (333,824 cells), the patch layouts cost what the Lambert raster
  costs** [C]:
  - one diffusion pass (the matrix product of one conjugate-gradient iteration, ghost copy included): cube
    0.89 ns per cell against Lambert's 0.88 on one thread (1.0×), 0.38 against 0.29 on eight (1.3×, 0.03 ms);
    HEALPix the same; the icosahedral rhombi 2.0× on one thread because a hexagon has six neighbours, not four;
  - a whole flow-routing pass (receivers, Braun–Willett stack, drainage accumulation): 14.1 ns per cell on the cube
    against 14.4 on Lambert (1.0×);
  - at the L2 and L3 cell counts (1.3 and 5.3 million) the cube stays within 0.9–1.2× of Lambert.
- **The coordinator's guess (1.5–3× slower) is right only for an unstructured mesh, and only on many threads.** On
  the Fibonacci sphere's Delaunay mesh (Deep Time's own `SphereMesh`) a diffusion pass costs 7.5× the raster in
  Fibonacci order and 4.9× in a Hilbert order on one thread, 2.7–3.1× on eight; flow routing costs 1.0× (Hilbert)
  to 1.3× (Fibonacci order), and a random order, the worst case, 2.4×. [C]
- **For the solvers that use true distances, the sphere grid is faster, not slower.** Lambert's polar cells are up to
  120 times longer than wide (37 inside 80°). An implicit diffusion solve in the true metric needs 785
  conjugate-gradient iterations on the Lambert raster against 138 on the cube at L1 (6×), and 9–14× as many at L2
  and L3; an explicit scheme's stable time step on the raster is 1/80 of a uniform grid's. On the cube the same solve
  takes 249 ms against Lambert's 1,273 ms. [C]
- **Prior art that solved the same problem:** GFDL FV3 (cubed sphere, regular arrays per face); NICAM (icosahedral
  rhombi as 2-D arrays with direct offsets); fastscapelib's `HealpixGrid` (Braun–Willett stream power and single
  flow-direction routing on an equal-area HEALPix sphere, 2025); goSPL (global landscape evolution on icosahedral
  meshes of 6–24 million nodes, at HPC cost). [SA, V]
- **Recommendation: a cubed sphere of six regular faces with one ghost ring as Deep Time's sphere-native simulation
  grid.** The climate (P1-26, not built yet), orographic rain and ice sheets move to it first, because they use true
  distances. The surface solvers (erosion, routing, diffusion, marine) follow the world: they stay on the Lambert
  raster in block metric while the world is Lambert, and move to the cube faces if the world becomes the
  equal-area cube of the projection survey, where the simulation's faces and the world's faces are then the same
  grid (at the quick 16k preset both have 236 L1 cells along a face edge). §9 and §10.
- **Questions for Ben** (§11): adopt the cube for the climate now (recommended); let the surface grid follow the world
  decision (recommended); per-face FFT for orographic rain (recommended) or a non-spectral model.

---

## 1. Deep Time's two metrics, and where the poles actually hurt

Deep Time runs below the plates on rasters in the world's projection, Lambert's cylindrical equal-area projection
since 2026-10-02 (`crates/deeptime-format/src/projection.rs`; `docs/design/equal-area-projection.md`). Two metrics
live on those rasters (design note §2, "Two metrics") [V]:

- **Block metric** (square cells of r blocks, what a player walks): erosion, routing, hillslope and marine diffusion,
  hydrology, amplification (`surface/grid.rs`: "every L1 cell is r × 300 m wide in x and z whatever its latitude").
  These solvers see no polar anisotropy at all: in block metric the Lambert raster is a uniform square grid. What
  the poles cost them is elsewhere: planet-scale inputs (uplift, plates) are stretched east–west on the map, by
  sec φ, and rows near the pole resolve the polar caps badly (the grid-artifact audit's "radial streaks round both
  poles", `grid-artifact-audit.md`).
- **True metric** (lengths on the sphere): the climate's geometry (`climate/mod.rs` `sx`, `sy`, per row), the ice
  distance transform, the climate's row marches, the orographic FFT along rows, the ITCZ. On Lambert a cell is
  cos φ as wide and 1/cos φ as tall as at the equator, so its aspect ratio is 1/cos²φ: 3 at 55°, 8.5 at 70°, 33 at
  80°, and about 120 in the L1 raster's first row (one cell from the pole). [C, I]

So "the flat raster distorts the maths near the poles" is true for the true-metric solvers, and for the
planet-scale fields the block-metric solvers are fed; it is not true of the block-metric stencils themselves. That
distinction drives the recommendation (§9).

**Cell counts.** The quick 16k preset has L1 = 1,024 × 326 = 333,824 cells (`resolution.l1_r` = 16; keyframes
`[1024, 326]` in `presets/earthlike_quick_16k.json`) and L2 = 2,048 × 652 = 1,335,296. [V]

## 2. How weather and climate models solved it

<!-- CLIMATE-SECTION -->

## 3. Data layouts that recover raster speed on a sphere

### 3.1 Patches of regular arrays (the layout that works)

- **NICAM stores its icosahedral grid as 2-D arrays and reaches neighbours by integer offsets.** "Data structure of
  NICAM: (ij,k,l) = horizontal grid, vertical grid, region"; the seven-point stencil uses `ip1j = n+1`,
  `ijp1 = n+ADM_gall_1d`, `im1jm1 = n-1-ADM_gall_1d`, with no index lists (Yashiro, RIKEN, ECMWF 2014 slides).
  Tomita & Satoh (2004): the icosahedral grid allows "the treatment of gridpoint data as two-dimensional structured
  data in the program code". [SA]
- **FV3 keeps six tiles of 2-D arrays**, with special stencils at corners and edges; "FV3's corner and edge handling
  has to be computed on every rank" (Dahm et al. 2023, the Pace port). [SA, summary only]
- **HEALPix's nested numbering is a Morton (Z-order) curve inside each of 12 base pixels**: children of pixel u are
  4u … 4u + 3, and "near-neighbour searches are conducted optimally in the data storage medium or computer RAM"
  (Górski et al. 2005). DKRZ's documentation for HEALPix model output: nested order "ensures that geographical
  proximity of data is mapped to proximity in the array". [SA]
- **S2** links six per-face Hilbert curves into one loop of 64-bit cell ids (3 face bits, 2 bits per level), with a
  quadratic map that evens out cell sizes; close ids are close cells (s2geometry.io). The tool's Hilbert order for
  the Fibonacci mesh is this construction. [SA]

### 3.2 Reordering an unstructured mesh (helps, does not close the gap)

- Hilbert reordering of a 2-D unstructured quad mesh of 250k–1M elements: 15–25 % faster on a 32-thread Xeon;
  last-level cache misses roughly halved (Araújo, Gross & Xavier-de-Souza 2021). Cuthill–McKee-like orders were
  worse than the original. [SA]
- Laplacian smoothing on meshes of 298k–392k vertices (our size): random order 10.3 s, generator order 7.6 s,
  breadth-first order 6.6 s (Aupy, Park & Raghavan 2016). [SA]
- SFC orders and reverse Cuthill–McKee give "about the same data cache hit-rate" (Aftosmis, Berger & Murman 2004).
  [SA]
- Reordering data and computation together is what pays (Mellor-Crummey, Whalley & Kennedy 2001). [SA]
- ICON, the operational unstructured model, says it plainly: "Accesses to cell, edge, and vertex neighbors go
  through indexing arrays, i.e., indirect addressing", and "horizontal neighbour relations cannot be exploited in
  the memory layout" (Giorgetta et al. 2022). ICON amortises each index lookup over the vertical levels stored
  innermost (nproma blocking); Firedrake measured that 10–20 contiguous layers per lookup bring the cost of
  indirection under the hardware limit's 70–90 % (Bercea et al. 2016). A 2-D landscape solver has no vertical
  levels to hide behind, so it is the worst case for an unstructured mesh. [SA, I]
- Both stencils and sparse matrix products are memory-bandwidth bound (Williams, Waterman & Patterson 2009: SpMV
  0.17–0.25 flop per byte, a 7-point stencil 0.33–0.50); SIMD barely helps a memory-bound kernel "unless the working
  set is small enough to fit into a cache" (Kreutzer et al. 2014). Deep Time's L1 fields (2.7 MB each) do fit the
  VM's 64 MB of L3, which is why the raster's SIMD-friendly loops show (§7.3). [SA, C]

No source the sub-agent found measures the same stencil on an index-list mesh and on direct offsets on one sphere
and one CPU. §7 does.

## 4. Landscape evolution on the sphere

- **fastscapelib `HealpixGrid`** (v0.3.0, June 2025): "2-dimensional, HEALPix grid on the sphere, static", for
  "Planetary scale simulations" [V]. Its flow routers and sink resolvers are grid-agnostic; its planetary example
  runs single-direction routing, the minimum-spanning-tree sink resolver and the Braun–Willett implicit stream-power
  eroder at nside 100 (120,000 cells) and omits hillslope diffusion, which the documentation lists for "raster grids
  only". No timings published. This is the closest prior art: Deep Time's own L1 algorithms on an equal-area
  sphere grid. [SA]
- **goSPL** (Salles et al. 2020, 2023): global landscape evolution on unstructured icosahedral meshes (10 km =
  5.9 million nodes; the 2023 Nature run "more than 10 million nodes"), multiple-flow-direction routing solved as a
  sparse linear system with PETSc, implicit finite-volume hillslope and marine diffusion on the Voronoi dual,
  parallel priority-flood, flexure by spherical harmonics. Its 10 km mesh cannot start below 16 MPI ranks (521 s
  for an unstated number of steps; 44 s on 288 ranks). Proof that every one of Deep Time's surface processes has
  a published sphere-native form, and a warning of what the unstructured route costs. goSPL also implemented a
  shallow-ice glacier solve and removed it: the free ice margin made the implicit solve diverge at its long time
  steps. [SA]
- **Braun & Willett's O(n) stack is graph-generic**: Badlands (TIN), Cordonnier et al. 2016 (Delaunay graph) and
  fastscapelib (TriMesh, HEALPix) use it on irregular meshes; Barnes, Lehman & Mulla 2014's priority-flood "can work
  on irregular meshes as well as 4-, 6-, 8-, and n-connected grids"; Cordonnier, Bovy & Braun 2019's depression
  routing takes any planar graph. The paper itself is paywalled and was not read. [SA]
- Performance figures in the literature are not comparable (different codes, hardware, decades): Cordonnier 2016,
  160k irregular nodes, 1.27 s per step unoptimised; Barnes 2018's raster B&W about 240 ns per cell-step (the
  sub-agent's arithmetic). Nothing published compares raster and sphere grid like for like. [SA]

## 5. FFT and spectral steps on the sphere

Deep Time has two FFT solvers on rows: Smith & Barstad orographic rain (`climate/orographic.rs`) and lithospheric
flexure (`surface/flexure.rs`).

<!-- SPECTRAL-SECTION -->

- **Hergarten & Robl 2022 (LFPM)** is a non-spectral orographic precipitation model: two moisture components,
  first-order upwind advection, transversal dispersion, one tridiagonal system of 2 × 2 blocks per slice; linear in
  cell count, about 2.2–2.4× the cost of a simple stream-power step. It assumes a uniform wind along a lattice axis
  in its cheap form. [SA]
- **goSPL's mesh solver** drops the Smith–Barstad mountain-wave term and solves an upwind finite-volume form on the
  unstructured mesh; its documentation contradicts itself on whether this works on global meshes. No published
  spherical-harmonic or per-face Smith–Barstad was found. [SA]
- **On a cube** [I]: each face is a regular n × n array, so Smith & Barstad runs per face exactly as today, with
  (a) the wind turned into the face's local frame (the face basis is known per cell), (b) the face padded with a
  band of its neighbours (as the raster already zero-pads rows) and a taper, and (c) the overlap blended. The
  transfer function is local in practice (a few hundred kilometres, the rain-shadow length), far smaller than a
  face (about 10,000 km on Earth), so the faces' seams matter only within the pad. Flexure is the same (its
  flexural parameter is tens to hundreds of kilometres); goSPL uses spherical harmonics for it. [SA, I]

## 6. Graphics and procedural planets

- Red Blob Games' planet generator (Patel 2018): "The sphere map is a graph and the river code didn't require any
  changes to work on spheres", with a half-edge dual mesh in integer arrays; "It may be slow when you increase the
  number of regions" (no timings). [SA]
- World Orogen: Fibonacci sphere, Delaunay, Voronoi cells (5k–2.56M), stream-power erosion with priority-flood, in
  JavaScript, no timings. [SA]
- Cordonnier et al. 2016 chose a Poisson-disk Delaunay graph over a regular grid because "the edges of the graph
  better represent possible directions for local streams" (planar). [SA]
- Cortial et al. 2019 (the plate model Deep Time's L0 follows): 60k–500k samples on a spherical triangulation;
  erosion is a constant, stream power left as future work. [SA]
- Planet renderers (Outerra, SpaceEngine and the like) use cube-face quadtrees for terrain; the projection survey
  covers their maps (`seamless-projection-survey.md` §2.2). [SA, snippets only]

Games that hydrologically simulate a sphere do it on the graph (Fibonacci or Goldberg cells) and accept the speed;
none reported timings. Planetary renderers use cube faces. Deep Time's L0 already follows the games; the
recommendation follows the renderers and FV3 for everything finer.

## 7. The benchmark

### 7.1 What it measures

`tools/research/sphere-grid-bench/` (its header and `run.sh` give the details) builds eight layouts of about the
same cell count, puts the same smooth terrain on each (32 random waves of the unit vector), and times four kernels
with the shipped build profile (release, thin LTO, generic x86-64; a second build with `target-cpu=native`):

- `diff`: one application of the implicit diffusion operator (I + Δt·L)·h with a coefficient per cell and neighbour;
  the patch layouts first copy their ghost ring. This is the matrix product of one conjugate-gradient iteration of
  `surface/diffusion.rs`, the hot loop of hillslope and marine diffusion and of a 2-D energy balance.
- `rcv`: D8-style receivers (steepest descent over the 8 square or 6 hexagonal neighbours).
- `stack`, `acc`: donor lists, Braun–Willett stack and drainage accumulation (serial, as shipped).
- A whole implicit diffusion solve in each layout's true metric (Jacobi-preconditioned CG to 10⁻⁸), at
  D·Δt/Δx² = 1 and 30 (30 is the shelf's value in `surface/diffusion.rs`), one thread.

Layouts: `lambert` (today, W/H = π, wrapping in x, closed pole rows); `cube` (six equiangular gnomonic faces of
236 × 236, one ghost ring); `healpix` (twelve base pixels of 167 × 167); `icos` (ten icosahedral rhombi of 183 × 183
triangle-lattice points, six neighbours; the two pole points left out); and the Fibonacci sphere's Delaunay graph
(`SphereMesh`, max degree 7) as a neighbour list in Fibonacci order (`fib-plain`), S2-style Hilbert order
(`fib-hilbert`), random order (`fib-random`), and Hilbert order padded to fixed width (`fib-hilbert-ell`, ELLPACK).

Ghost cells: each patch's ghost ring points at the owned cell nearest the reflection of the edge cell's inward
neighbour. The median ghost source lies 0.96–1.00 inward spacings away (cube 0.963, HEALPix 1.000, icosahedron
0.996); the maxima (1.38 HEALPix, 1.89 icosahedron) are the corners where three or five patches meet. The diffusion
operator keeps a link only if both ends name each other, so it stays symmetric at those corners. [C]

### 7.2 Results (median ns per cell; in brackets, the ratio to Lambert)

Test VM (AMD EPYC 9V45, 16 cores / 32 threads, 64 MB L3), generic build, 4 runs at L1 and L2 and 2 at 5.3M; the VM
was shared (load average 13–25 on 32 threads during the runs), so single numbers move by about ±20 %. [C]

**L1, 333,824 cells:**

| layout | diff, 1 thread | diff, 8 threads | rcv, 1 | rcv, 8 | stack, 1 | acc, 1 | routing pass, 1 (rcv + stack + acc) |
|---|---|---|---|---|---|---|---|
| lambert | 0.88 | 0.29 | 5.17 | 0.86 | 7.49 | 1.75 | 14.4 (1.0×) |
| cube | 0.89 (1.0×) | 0.38 (1.3×) | 5.14 (1.0×) | 0.86 (1.0×) | 7.38 | 1.62 | 14.1 (1.0×) |
| healpix | 0.95 (1.1×) | 0.37 (1.3×) | 5.21 (1.0×) | 0.81 (0.9×) | 8.70 | 2.19 | 16.1 (1.1×) |
| icos | 1.77 (2.0×) | 0.40 (1.4×) | 3.99 (0.8×) | 0.65 (0.8×) | 7.37 | 1.69 | 13.1 (0.9×) |
| fib-plain | 6.59 (7.5×) | 0.77 (2.7×) | 6.68 (1.3×) | 0.73 (0.8×) | 10.29 | 1.92 | 18.9 (1.3×) |
| fib-hilbert | 4.29 (4.9×) | 0.87 (3.1×) | 5.20 (1.0×) | 0.77 (0.9×) | 7.79 | 1.63 | 14.6 (1.0×) |
| fib-random | 8.34 (9.5×) | 1.31 (4.6×) | 7.89 (1.5×) | 1.26 (1.5×) | 24.77 | 2.24 | 34.9 (2.4×) |
| fib-hilbert-ell | 4.32 (4.9×) | 0.86 (3.0×) | 5.22 (1.0×) | 0.74 (0.9×) | 7.62 | 1.62 | 14.5 (1.0×) |

In absolute terms an L1 diffusion pass is 0.29 ms on Lambert and on the cube on one thread, 1.4 ms on the
Hilbert-ordered mesh; a routing pass 4.8 ms on Lambert, 4.7 ms on the cube.

**Diffusion pass at larger sizes** (ns per cell):

| layout | L2 (1.34 M), 1 thread | L2, 8 threads | 5.34 M, 1 thread | 5.34 M, 8 threads |
|---|---|---|---|---|
| lambert | 1.96 | 0.43 | 2.21 | 0.72 |
| cube | 2.15 (1.1×) | 0.47 (1.1×) | 2.30 (1.0×) | 0.65 (0.9×) |
| healpix | 2.00 (1.0×) | 0.46 (1.1×) | 2.33 (1.1×) | 0.65 (0.9×) |
| icos | 2.41 (1.2×) | 0.78 (1.8×) | 3.12 (1.4×) | 1.20 (1.7×) |
| fib-plain | 8.61 (4.4×) | 1.30 (3.0×) | 8.89 (4.0×) | 1.38 (1.9×) |
| fib-hilbert | 7.03 (3.6×) | 1.40 (3.3×) | 9.86 (4.5×) | 1.75 (2.4×) |
| fib-random | 10.41 (5.3×) | 1.61 (3.7×) | 54.78 (24.7×) | 6.55 (9.2×) |

At 5.3M cells the random order falls out of cache (a field is 43 MB): 25× the raster. The Fibonacci order stays
usable because the Fibonacci spiral numbers points by latitude band, much as a raster does; its neighbours sit at a
few fixed index distances (Fibonacci numbers). The routing pass (rcv + stack + acc, one thread) stays within
0.9–1.2× of Lambert at L2 and 5.3M for the patch layouts and the Hilbert-ordered mesh; the Fibonacci order costs
1.4× at L2 and 2.0× at 5.3M (its stack walk), the random order 2.8× and 11×. [C, I]

**Implicit diffusion solve, true metric, one thread** (iterations, then milliseconds):

| layout | L1, D·Δt/Δx² = 1 | L1, 30 | L2, 30 | 5.34 M, 30 |
|---|---|---|---|---|
| lambert, block metric (as shipped, isotropic) | 16, 30 ms | 129, 208 ms | 124, 965 ms | 118, 4.3 s |
| lambert, true metric | 129, 194 ms | 785, 1,273 ms | 1,235, 10.1 s | 1,676, 57.8 s |
| cube | 20, 34 ms | 138, 249 ms | 132, 1,049 ms | 122, 5.0 s |
| healpix | 19, 33 ms | 132, 227 ms | 124, 952 ms | 114, 4.5 s |
| icos | 19, 37 ms | 117, 238 ms | 115, 994 ms | 108, 4.6 s |
| fib-hilbert | 16, 46 ms | 110, 326 ms | 103, 1,404 ms | 95, 7.2 s |

**Geometry** (L1): longest over shortest neighbour step per cell, maximum (inside 80°), and the explicit diffusion
step limit against an ideal uniform square grid of the same cell count:

| layout | anisotropy max | inside 80° | explicit Δt / ideal |
|---|---|---|---|
| lambert (true metric) | 119.6 | 37.0 | 0.012 |
| cube (equiangular) | 1.41 | 1.41 | 0.79 |
| healpix | 1.86 | 1.85 | 0.94 |
| icos | 1.89 (pole points omitted; 1.16 at p99) | 1.18 | 0.71 |
| fibonacci mesh | 1.60 | 1.60 | 1.06 |

**Resampling onto the Lambert raster** (a 4-point weighted gather per raster cell, L1, one run): 0.6 ms on one
thread and 0.1–0.2 ms on eight from the cube, the icosahedron or the mesh. A sphere grid pays this once per field
each time the game raster is written (keyframes, the bake), not per solver iteration. [C]

**SIMD.** The `target-cpu=native` build (AVX-512 on this CPU) speeds the raster's diffusion pass by about 30 % on one
thread at L1 (0.62 against 0.88 ns per cell) and the cube's by about 25 % (0.68); it does nothing for the mesh's
gathers (4.85 against 4.29). Deep Time ships the generic build, so the table above uses it. [C, one run]

### 7.3 What the numbers say

1. **Patches of regular arrays are as fast as the raster.** The ghost copy (about 4 n cells per face) is noise; the
   inner loops are identical. Two dense faces of 236 × 236 are as good for the prefetcher as one 1,024-wide raster.
2. **The hexagonal rhombi pay for their six neighbours** in the diffusion pass (more coefficient streams), and win
   back a little in receivers (six candidates instead of eight). Their CG converges in the fewest iterations of the
   patch layouts.
3. **The index-list mesh pays 3–5× in the stencil pass**, which is the gather and the per-edge index and weight
   streams. A Hilbert order does not close it at L1 (4.9×; it is already near the Fibonacci order's locality there)
   and helps against random order by 2× to 6×. The ELLPACK padding brings nothing on the CPU. Flow routing barely
   notices the layout, because its hot loops (the stack's depth-first walk and the accumulation) are graph walks on
   every layout.
4. **The anisotropic raster costs more in iterations than any layout costs per iteration.** In true metric the
   Lambert raster's 120:1 polar cells make the implicit operator ill-conditioned: 6× the iterations of the cube at
   L1, 14× at 5.3M, more than every layout's per-pass overhead. Today's block-metric solvers avoid this only
   because block metric treats the raster as flat.

## 8. Comparison

| layout | distortion (area max/min; shape) | poles and singular points | stencil speed vs Lambert (L1: diffusion 1 / 8 threads; routing) | FFT or spectral | flow routing | cost in Deep Time |
|---|---|---|---|---|---|---|
| Lambert raster (today) | equal-area; true-metric shape up to 120:1 (33 at 80°) | two pole rows, a seam (wraps) | 1.0 / 1.0; 1.0 | FFT along rows (today); poles break the periodic grid's assumption | D8 (today); fine in block metric | none |
| cubed sphere, equiangular (FV3) | area 1.41 [I: analytic, edge midpoint against centre]; shape ≤ 1.41 | 8 corners (3 faces meet) | 1.0 / 1.3; 1.0 | FFT per face with a padded halo; spherical harmonics for global fields | D8 per face, ghost ring at edges (fastscapelib-style canonical ids) | grid module + coupling ~2–3 days; per solver below |
| equal-area cube (survey's Moser cube or QSC) | area 1.0; shape ≤ 1.42 (Moser) / 1.55 (QSC) (`seamless-projection-survey.md`) | 8 corners | as the equiangular cube [I: same layout; only the per-cell coefficients differ] | as the cube | as the cube | as the cube, plus the survey's face table |
| HEALPix (12 base pixels) | area 1.0; shape ≤ 1.86 (measured neighbour steps) | 8 points where 3 pixels meet; faces are rhombi on the sphere | 1.1 / 1.3; 1.1 | ring scheme suits spherical-harmonic transforms (iso-latitude rings); no per-face rectangular FFT | fastscapelib `HealpixGrid` (prior art) | as the cube; less natural against a cube world |
| icosahedral rhombi (NICAM, GME) | near-uniform [I]; shape ≤ 1.18 inside 80° | 12 pentagons | 2.0 / 1.4; 0.9 | no rectangular FFT on a triangle lattice (needs hexagonal FFT or spherical harmonics) | 6-neighbour routing: new D6 code, B&W stack unchanged | higher: hexagonal stencils everywhere, new D6 |
| Fibonacci / Voronoi mesh, Hilbert order | near-equal areas [I]; shape ≤ 1.60 | none (but irregular everywhere) | 4.9 / 3.1; 1.0 | none on the mesh: spherical harmonics or resample | graph routing (goSPL, Red Blob, Cordonnier) | highest: every stencil becomes a gather; L0 already has the mesh |
| Fibonacci mesh, plain / random order | as above | as above | 7.5 / 2.7; 1.3 (plain); 9.5 / 4.6; 2.4 (random) | as above | as above | as above |
| Yin-Yang (two lat-lon patches) | see §2 | none (overlap seams) | not measured [I: two rasters plus overlap interpolation] | FFT along each patch's rows | D8 per patch, overlap interpolation | interpolation at the overlap each step |
| reduced Gaussian / spectral | see §2 | poles thinned | not measured | spherical harmonics (the global FFT analogue) | not a routing grid | not for routing; a candidate for global transforms only |

## 9. Recommendation and migration sketch

**Recommended: a cubed sphere of six n × n faces with one ghost ring, as Deep Time's sphere-native grid for every
solver that uses true distances, and for the surface solvers once the world is a cube.**

Why the cube rather than HEALPix or the icosahedron:

- the same speed as the raster (§7), with square cells, so the D8 code, the 5-point CG, `box_blur` and the FFT
  solvers carry over per face with no new stencil;
- the projection survey's recommended seamless world is an equal-area cube in a cross net; a cube simulation grid is
  then the world's own grid (§10);
- the climate models' most-used high-performance layout (FV3), with a documented treatment of edges and corners;
- HEALPix is the close second (equal-area, fastscapelib precedent), but its faces are rhombi on the sphere with
  steeper shape (1.86) and no face-aligned FFT; the icosahedron needs six-neighbour stencils everywhere.

Which cube map: the survey's equal-area cube (Moser-smoothed conformal cube, or QSC), so the simulation's cells
are equal-area like today's (mass bookkeeping per cell stays a multiply) and, on a cube world, coincide with the
world's blocks. An equiangular cube is simpler (closed form) and was what the benchmark used; the per-face loops are
identical, only the per-cell coefficients change. [I]

**Migration, in order:**

1. **`core::sphere::cube`** (2–3 days): faces of n × n with ghost rings, face frames and per-cell metric
   (local east/north basis, cell area, neighbour distances), the ghost tables (as in the benchmark, but built
   exactly from the face adjacency), a cell → L0 coupling (the existing `Coupling` generalised from a `GridDesc` to
   any list of unit vectors) and a cube → raster resampler (the same structure the coupling already has). Cell
   order: face-major, row-major inside a face (the benchmark's), no curve needed.
2. **The per-cell climate (P1-26) on the cube from the start** (not built yet, so the cheapest moment; +1–2 days
   over its lat-lon plan): the 2-D energy balance's heat transport and the moisture budget's upwind fluxes are
   finite-volume operators on the faces; winds are stored in each cell's east/north basis and turned at the
   ghost copy. A 128 × 64 lat-lon climate grid (P1-26's default) becomes six faces of 37 × 37 (8,214 cells,
   about 250 km on Earth).
3. **Orographic rain and flexure per face** (2–3 days): FFT over each face padded by a neighbour band, the wind
   turned into the face's frame, overlap blended; or Hergarten & Robl's LFPM, which needs no FFT (§11 Q3).
4. **Ice sheets** (`climate/sheet.rs`, 1–2 days): the shallow-ice flux on the faces; the distance transform
   becomes a fast-marching pass with the cube's metric.
5. **Surface solvers, only if the world becomes the cube** (2–3 weeks for the whole surface stage): routing,
   priority-flood and the stack take a neighbour function instead of `Dims` (the B&W stack and the forest are
   already graph code); diffusion's CG runs per face with the ghost copy before each matrix product; marine,
   glacial and the amplifier follow the same pattern. While the world stays Lambert they stay where they are: in
   block metric on the raster they have no polar anisotropy, and moving them to the sphere would draw high-latitude
   valleys squashed north–south in the game (aspect 1/cos²φ: 8.5 at 70°, the cap line).
6. **Bake and keyframes** keep the world's projection: the cube fields are resampled to it once per keyframe and at
   the bake (0.1–0.6 ms per field at L1, §7.2).

Expected speed: the climate solves converge in 1/6–1/14 of today's true-metric iterations at the same per-iteration
cost; the surface stage, if moved, runs at the raster's speed ±10 % (the measured range).

## 10. How the grid fits each game-world option

| world (`seamless-projection-survey.md`, decisions log 2026-10-02/03) | climate grid | surface grid | resample at bake |
|---|---|---|---|
| **Lambert cylindrical equal-area (shipped)** | cube | Lambert raster, block metric (unchanged) | cube → Lambert for climate fields; L0 → raster as today |
| **Equal-area cube in a cross net (survey's recommendation)** | cube | the same cube faces; at quick 16k, 236 L1 cells (16 blocks each) per 3,776-block face edge, so simulation cells and world blocks coincide | none: the world is the grid (L3 is the same faces at a finer r) |
| **Petroff or Guyou 2:1 with portal seams (runner-up)** | cube | cube (true shape; the 2:1 map distorts up to 2.67×, so block metric and true metric part ways again) | cube → 2:1 map at the bake (Ben chose "Bake only" for the seamless production world, 2026-10-03) |

The cube is the one choice that is right under every world: it is the climate's grid in all three, and in the
survey's preferred world it is also the surface's.

## 11. Open questions for Ben (recommendation first)

1. **Grid for the climate (P1-26) and the other true-metric solvers.** (a) The cubed sphere, built into P1-26 from
   the start (recommended); (b) HEALPix; (c) keep lat-lon/Lambert and accept the polar cost.
2. **Surface solvers.** (a) Follow the world decision: stay on Lambert in block metric while the world is Lambert,
   move to the cube faces with a cube world (recommended); (b) move to the cube now, accepting landforms squashed
   north–south near the poles in a Lambert world; (c) stay on the raster whatever the world.
3. **Orographic rain on the cube.** (a) Smith & Barstad per face with padded halos (recommended: today's model,
   same results away from the seams); (b) Hergarten & Robl's LFPM, which carries moisture farther and needs no FFT
   but is a different model to calibrate.

## 12. Method

- **Tool** (LGPL-3.0-or-later, not shipped, its own one-crate workspace): `tools/research/sphere-grid-bench/`
  (`src/main.rs`, `run.sh`, `summarize.py`). It depends on `deeptime-core` for the Fibonacci sphere, `SphereMesh`
  and `SampleTree`, so the mesh is Deep Time's own. HEALPix centres from the nested-scheme face formulas (Górski et
  al. 2005, as in healpix_base's pix2loc, written from the formulas); the rest from the definitions in its header.
- **Runs:** test VM, 2026-10-04, through the admission tool:
  `tools/remote/mac.sh -- tools/research/sphere-grid-bench/run.sh build`, then
  `tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start sgbench --kind sim:quick:8t tools/research/sphere-grid-bench/run.sh run`
  and `... start sgrepeat ... run.sh repeat`; `python3 tools/research/sphere-grid-bench/summarize.py
  review/out/sphere-grid-bench generic` for the medians. Results in `review/out/sphere-grid-bench/` (not committed).
  Each L1 run takes about 15 s; the 5.3M runs about 7 minutes (mostly the Lambert true-metric solve).
- **Not done:**
  - no Deep Time solver was ported; the kernels are representative loops, not the shipped code (the shipped CG adds
    vector operations, which cost the same on every layout);
  - D8 on the patch layouts uses one step length per direction (block-metric style); a true-metric D8 would add a
    per-cell length table on every layout;
  - the cube's ghost ring copies the adjacent cell (the finite-volume neighbour); FV3's higher-order halo
    interpolation (needed for high-order advection, not for these kernels) was not timed;
  - the icosahedron's two pole points were left out (2 of 334,890 cells);
  - the VM was shared and loaded; medians over 2–4 runs.

## 13. Sources and how they were read

FT = full text, AB = abstract or snippet only, DOC = documentation page, SUM = a web summariser's reading of the
page, not checked against it. [SA] = read by a Sonnet sub-agent this session, which wrote transcriptions to the
session scratchpad (not committed); [V] = read by me.

<!-- SOURCES -->
