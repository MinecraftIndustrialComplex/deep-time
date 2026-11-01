# The GPU compute path: design

Status: **infrastructure and five kernels built and measured** (phases 1 to 3, 2026-10-18 to
2026-10-19; coordinator's task after Ben's decision of 2026-10-18, decisions log: "I am not
worried about determinism, performance is far more important, if people want to share a world
they can just share a zip of the finished render or some shit."). Branch `worktree-agent-a1265f3344b19b0c1`
from master 584b1241, masters 5cf2f9bb, e2166684 and c767da2c merged in. The aim is the P1-19
exit item 1 gate (full 16k, 8 threads, 1000 Myr, 900 s), which the 2026-10-18 profile put at
1,242.8 s (`docs/design/p1-17-benchmark-gate.md`, "where the full 16k run's time goes";
`review/pages/sim-time/data.js`). Phase 2 (2026-10-19, branch `gpu-phase2` from master
b337fa73, masters fb38204f, 7d60fc4a and 26ce664e merged in): the plates' L1 relink and the climate
snapshot's cells ported (sections 13 and 14), the whole-history comparisons and the laptop's
benches in section 10. Phase 3 (2026-10-19, branch `gpu-phase3` from master 3e7edd49): the
diffusion solve's V-cycle and the submarine slope failure ported (sections 15 and 16), the land
lean settled with ten more seeds and a kernel-by-kernel bisection, and the laptop's benches, in
section 10.

Statements are marked **measured** (a run made for this note, with its command), **read** (from
the code) or **estimated**.

## 1. Summary

- **What exists now.** A crate `deeptime-gpu` (wgpu 30: Vulkan, DX12, Metal) holds the device, the
  adapter choice with the CPU fallback, buffers, kernels and banded dispatches. `deeptime-core`
  takes it behind a feature `gpu` (on in the CLI and in the natives for Linux and Windows) and
  runs one ported kernel group through one dispatch point: the amplifier's middle passes
  (stream-power erosion with its drainage area, the glacial valleys, thermal erosion, deposition)
  on the L3 tiles. A run picks the device itself (`auto`), or is told `cpu` or `gpu` by the server
  config, the CLI or the environment.
- **What it is worth today.** The ported passes are 12.0 s of the 1,242.8 s run (the amplify
  stage's erode 3.6, glacial 5.3, thermal 1.8, deposit 1.3 s; the stage is 24.0 s): at most about
  1 % of the gate run. **Measured** on the laptop's Radeon 8050S (section 10): those passes run
  8 to 9 times faster (11.3 to 12.7 s on 8 CPU threads, 1.4 s on the GPU with the transfers),
  and the whole amplify stage of a full 16k planet goes from 23.5 to 17.2 s at the
  default 4 GiB cap (the GPU then runs it in 8 tiles) and from 25.4 to 14.4 s when the level fits
  whole: 6 to 11 s off a full run. The amplifier was picked because nobody else was editing it;
  the large kernels are on CPU branches in flight.
- **Phase 2.** The plates' L1 relink (section 13) and the climate snapshot's per-cell months
  (section 14) run on the device too, each behind one dispatch point, with the CPU path the
  fallback and the reference. On the laptop's Radeon, quiet, the relink takes `write_l1` from
  15.1 to 9.3 s per 100 Myr and the cells take 4.6 s to 1.1 s with their transfers (at a
  snapshot every 10 Myr; master fb38204f's 20 Myr halves both cells figures, so the cells save
  about 1.8 s per 100 Myr, some 18 s per 1000 Myr). Run against the CPU on the same plate state
  or the same snapshot, neither port leans any way (no layer, no climate field; section 10), and
  over whole histories the GPU's CO2 lands on either side of the CPU's on five planets (-117 to
  +70 ppm, mean temperatures equal). Over a 1000 Myr full 16k run on the laptop the ported
  kernels (with Phase 1's amplifier) take about 72 to 86 s off the CPU's 2,402 s (3 to 3.6 %);
  that pair's walls were set by the laptop's other load, not the GPU (section 10).
- **Phase 3.** The diffusion solve's multigrid V-cycle (section 15; the conjugate gradients and
  their f64 sums stay on the CPU) and the submarine slope failure (section 16) run on the device,
  in the surface stage and in the young stage. On the laptop's Radeon, three 100 Myr pairs: the
  slope failure 5.33 to 1.73 s, the solve 6.37 to 5.27 s, 7.7 s off the surface and young stages
  per 100 Myr; every ported kernel together takes a full 16k 100 Myr run from 151.4 to 123.9 s.
  The early stop of the slope failure was not taken (it stops no call early). On the same state
  the solve gives the CPU's heights to the bit on lavapipe and the slope failure leans no way.
  Over whole histories, ten more seeds settle Phase 2's open question: **land ends higher on the
  GPU** (13 of 15 seeds, +0.007 of the area, p = 0.002), from every kernel group and from none
  alone on the same state; CO2, temperature and the sea level do not lean (section 10).
  Three CPU controls with changes that do nothing physical lean up less (pooled +0.002, 19 seeds
  up, 9 down); the GPU's worlds sit +0.005 above them (p 0.003), about 0.5 % of the planet's
  area against a 2.4 % spread between seeds. Whether that rest is the model answering the GPU's
  larger perturbation or a small bias of the kernels that only a history shows, these seeds do
  not tell.
  Over 1000 Myr (two pairs, one each way): the slope failure 90.2 to 18.9 s, the solve even,
  about 74 s off from this phase and 113 s from every ported kernel; the walls 955.6 and 863.1
  s on average.
- **Headline estimate (section 9).** About 513 s of the run's 1,242.8 s wall is grid work on the
  critical path that a compute shader can do. On a mid-range GPU (an NVIDIA T4 or the Radeon 8050S
  class, 250 to 320 GB/s) the ten kernels of section 6 would take an estimated **290 to 400 s** off
  the run, to about **845 to 955 s**, a factor of 1.3 to 1.47; even an infinitely fast GPU leaves
  about 730 s, because the plate step, the floods and the serial bookkeeping stay on the CPU. The
  first five (slope failure, the diffusion solve, the L1 write, the climate cells, advection with
  the climate's topography carry) are about 230 to 300 s of it. The CPU branches now in flight
  claim some of the same seconds (diffusion, slope failure, advection's page faults), so the two
  estimates do not add.

## 2. Layout and the dispatch point

| piece | where | what |
|---|---|---|
| device, buffers, kernels, bands | `crates/deeptime-gpu/src/lib.rs` | no simulation, no Minecraft, no JNI; every failure is an `Err(String)`, never a panic |
| compute mode | `crates/deeptime-core/src/compute.rs` | `ComputeMode` (auto, cpu, gpu), `DEEPTIME_COMPUTE`, `device(mode, log, kernel)` |
| session switch | `Session::set_compute`, `StageContext::compute` | the mode a stage reads |
| dispatch point | `surface/amplify/kernels.rs`: `PassDevice`, `run_passes_on` | the CPU path is `middle_cpu`; a device implements `middle` |
| tile loop | `surface/amplify/tiles.rs` | reserves the device's host memory, runs a tile on the device, reruns it on the CPU when the device fails |
| the kernels | `surface/amplify/gpu.rs` | WGSL next to the CPU code it mirrors; `AmplifyGpu` |
| ambient context | `compute.rs`: `Ambient`, `enter`, `ambient`, `ambient_device`, `Ambient::kept` | the session's mode and log on every worker of its pool, for kernels deep inside a stage that get no `StageContext` (the plates' L1 sampling); `kept` holds a kernel's pipeline and buffers until the session ends |
| dispatch point (L1) | `plates/l1.rs`: `sample` (the device from `ambient_device`), `on_device`, `sample_with` | the device links the covered cells, the CPU every other cell and all the layers (section 13) |
| the relink kernel | `plates/l1/gpu.rs`: `relink`, `Resident` | WGSL beside the CPU code; tolerance test in `plates/l1/gpu/tests.rs` |
| dispatch point (diffusion) | `surface/diffusion/multigrid.rs`: `cg`, `cg_on`, `Pre` | the CG on the CPU, its V-cycle on the device or the CPU (section 15) |
| the cycle's kernels | `surface/diffusion/multigrid/gpu.rs`: `DeviceCycle`, `Resident` | WGSL beside the CPU cycle; tests in `multigrid/tests.rs` |
| dispatch point (slope failure) | `surface/marine.rs`: `slope_failure` | the whole call on the device or the CPU (section 16) |
| the slope failure's kernels | `surface/marine/gpu.rs` | WGSL beside the CPU code; test in `marine.rs` |
| one kernel group only | `compute.rs`: `DEEPTIME_GPU_KERNELS`, `kernel_allowed`; `device(mode, log, kernel)` | a comma list of kernel name prefixes (`plates:write_l1`, `climate:cells`, `amplify:middle`, `surface:diffusion`, `surface:slope_failure`; `surface:` both); the others run on the CPU. For bisecting a difference, not for players |
| read-back | `deeptime-gpu`: `Gpu::readback`, `Gpu::submit_and_read` | a mapped buffer kept with a kernel's buffers; copies, submits, waits and hands the bytes to a closure |
| compute gauge | `compute.rs`: `add`, `lap`, `timings` | the GPU parts and their CPU twins in milliseconds and counts; `deeptime bench` prints `compute[...]` |
| CLI | `deeptime run/stage/bench ... --compute auto\|cpu\|gpu` | the flag wins over `DEEPTIME_COMPUTE` |
| JNI | `createWith` flags 4 (cpu) and 8 (gpu) | neither = auto; both = `IllegalArgumentException` |
| Java | `SimSession.Compute`, `SimSession.start(..., Compute)`, `server.json` key `compute` | the dedicated server's first start; the Create World screen runs auto |

A further kernel adds one `PassDevice`-like trait at its own stage (one small dispatch point), a
WGSL file next to its CPU code and a tolerance test against the CPU path; nothing else changes.

## 3. Device selection and fallback

- **Modes.** `auto` (default): the best hardware adapter, else the CPU. `cpu`: never a device.
  `gpu`: as auto, but a software adapter (Mesa's lavapipe, WARP) is accepted, which is how the
  test VM runs the GPU tests; when no adapter works, a warning and the CPU. Sources, strongest
  first: the CLI's `--compute`, JNI flags (from `server.json`'s `compute`), `DEEPTIME_COMPUTE`.
- **Ranking (read, `deeptime-gpu::open_inner`).** Discrete, then integrated, then virtual, then
  other GPUs; within a type the platform's native backend first (Vulkan on Linux, DX12 on Windows,
  Metal on macOS). Software adapters only in `gpu` mode or by name. `DEEPTIME_GPU_ADAPTER=<text>`
  picks the first adapter whose name contains the text (case-insensitive), software included.
- **Required.** Compute shaders (downlevel flag `COMPUTE_SHADERS`), the adapter's own limits
  requested in full (`required_limits = adapter.limits()`), `MemoryHints::Performance`.
- **One device per process.** `deeptime_gpu::shared` caches the opened device (one slot for
  hardware only, one with software allowed) and a failure to open, so a machine without a GPU
  probes once. A lost device is reopened on the next request.
- **Whole or tiles.** With a device, `auto` streaming (`amplify::streams_on`) also runs a level in
  tiles when the whole level does not fit on the device, or when the whole path's peak plus the
  device's share of host memory (below) does not fit under the cap. Tiles give the terrain the
  whole raster gives, to the bit on the CPU (`the_levels_before_the_last_agree_tiled_and_whole`),
  so this keeps the device on the level instead of handing it to the CPU. (Measured 2026-10-18,
  before this rule: the full 16k last level at the default 4 GiB cap fitted whole for the CPU,
  3,918 MiB at its fullest, but not with lavapipe's 64 bytes a cell more, so with the GPU forced
  it ran whole on the CPU, `device_tiles` 0 of 1.)
- **Fallback, per tile.** The tile loop runs a tile on the device only when no earlier tile
  failed, the window fits the device (`fits`: the largest buffer under `max_buffer_size` and the
  window's 56 bytes a cell under the budget), and the host memory reserve succeeds. Any error
  (allocation refused under the out-of-memory scope, a validation error, a device lost, a wait over
  120 s, a panic inside wgpu caught at open) re-samples the window and runs it on the CPU; the
  error is kept, later tiles stay on the CPU, and the run's notes say so.
- **Reporting.** `notes.json` gains `"compute": {mode, device, device_tiles, tiles, error}` and
  each level its `device_tiles`; the stage's summary line ends in ` compute=cpu` or
  ` compute=<adapter>:<device tiles>/<tiles>` with `[fell_back: ...]` after a failure; the timings
  gain `transfer` (uploads and downloads). The sim-time scripts read the summary line by key, so
  they keep working (`tools/review/sim-time-analyse.py`'s pattern ends at `total=`).

## 4. Memory budget

- **Device side.** 14 arrays of 4 bytes a cell (56 bytes) live on the device for a window's whole
  middle pass; the arrays swap by index, never by copy. The full 16k's last level is 8,192 x 2,624
  = 21.5 million cells, 1.2 GB on the device when it runs whole (a discrete GPU at the default
  4 GiB cap, or any device under a larger cap); in tiles (an integrated GPU at the default cap) it
  is 8 windows of at most 2,876 x 2,462 cells (tiles of 2,048 plus the measured halo of 414,
  cut at the poles), at most 397 MB each; the halos make the windows 1.85 times the level's
  cells, all computed. The budget is `DEEPTIME_GPU_MEM_MIB` (default 3 GiB) and wgpu's
  `memory_budget_thresholds.for_resource_creation` of 90 % of what the driver reports, under which
  an allocation fails cleanly instead of paging.
- **Host side.** Downloads go through a 64 MiB staging buffer (8 bytes a cell reserved for the
  host copies). On an integrated GPU (the laptop's 8050S, Apple silicon, lavapipe) the device's
  buffers are system memory, so the tile reserves `56 + 8` bytes a cell more against the
  session's native memory cap, through the same `reserve` that spills history frames; a discrete
  GPU reserves the 8. The stream decision counts that share (section 3), so a tile's reserve fails only when the
  run's other memory grew in between; the tile then runs on the CPU with the CPU's own reserve.
- **The driver's own memory is not counted.** Its code and its heaps are allocated by the
  driver, not through the counting allocator, so the cap does not see them. **Measured** on the
  VM (`tests/memory_estimate.rs` with `DEEPTIME_COMPUTE` auto and cpu alternating, twice each):
  `auto`'s probe loads lavapipe and rejects it, and that alone raised the test's resident peak
  from 99.9 and 98.3 MiB to 137.5 and 138.8 MiB, about 39 MiB, with the counted peak 95.3 MiB in
  all four. That test now runs on the CPU, since it checks the allocator's count against the
  process. A hardware driver's share is not measured here; like the JVM's memory, it sits beside
  the cap, not under it.
- **The static estimate** (`amplify::estimate`, plan §5.7.5) is unchanged: the GPU path runs only
  inside the cap's headroom, so the cap still bounds the run, and an estimate that counted device
  memory would refuse runs that fit on the CPU path. A stage ported later whose device memory is
  not optional (a resident field the CPU never holds) has to add it there.
- **The L1 relink (section 13)** keeps on the device, for the session: the lattice mesh, 80 bytes
  a sample (points 16, triangles' corners and neighbours 64 for about two triangles a sample),
  20 MB on the full 16k's 250,000 samples; per call, the samples' groups and the buckets, about
  28 bytes a sample, and the output, 20 bytes an L1 cell (27 MB for 1.34 M cells). The host holds
  the downloaded 20 bytes a cell while the CPU links its cells. These are optional in the same
  way (a failed allocation sends the call to the CPU), not reserved against the cap, and not in
  the plate stage's estimate (`plates/mod.rs`, which another agent was editing); on an
  integrated GPU they are about 110 MB of system memory beside the cap (54 MB of buffers, the
  staging copy and the host's words), like the driver's own.
- **The climate cells (section 14)** keep nothing between snapshots: per call, the row, column
  and coarse tables and one chunk of 2^18 cells (13 input planes plus one per wind regime with a
  mountain field, 10 output planes and the monthly ones, 4 bytes each: since the runoff law's
  second and third re-reviews 12 at every snapshot, the months' temperatures, and 24 at the
  last, their rain too): about 44.5 MiB on the device (56.5 MiB on the last; 32.5 before the
  months were kept at every snapshot), with host copies of the same size for the
  uploads and downloads. The output fields are the ones the CPU path fills, so they are not
  extra. Freed when the call returns.
- **The diffusion cycle and the slope failure (sections 15 and 16, Phase 3)** each keep one grid's
  buffers in a session slot, rebuilt when a call comes on a grid of another size: the cycle 32
  bytes a cell and level (28 on the finest) and a 4-byte read-back, 49.5 + 5.1 MiB on the full
  16k's L1; the slope failure 28 bytes a cell and a 12-byte read-back, 51.4 MiB on L1. The young
  stage calls both on L2 (4,096 x 1,312, 5.4 M cells), where they take about 230 and 215 MB, so
  after a young step the slots hold L2's buffers (about 445 MB) until the next L1 call rebuilds
  them at L1's size (about 106 MiB). Like the relink's they are optional (a failed allocation
  sends the call to the CPU) and sit beside the cap on an integrated GPU. The host's copies are
  smaller than the CPU path's own arrays: the cycle's residual and correction in f32 (8 bytes a
  cell, where the CPU's cycle holds its hierarchy, 28 bytes a cell on the finest level alone), the
  slope failure's one 4-byte upload copy at a time (the read-back is read in wgpu's mapped buffer,
  not copied).
- **What the counted host peak sees (read, 2026-10-19).** The counting allocator
  (`deeptime-core/src/alloc.rs`) counts the process's Rust allocations, so the host copies of
  both ports are counted: the relink's words (27 MB on the full 16k) and its per-call tables
  (at most about 18 MB) while `write_l1` runs in the plate stage, the cells' chunk copies (32.5
  to 56.5 MiB) while a snapshot runs in the climate stage. Both are freed before their call
  returns, and neither port runs in the young stage, the deposits or the amplifier, so the run's
  counted peak (the young stage's on master 7d60fc4a, the deposits' since master 26ce664e) is
  not raised by them (only by the GPU's world being another world). The device buffers, the relink's session-kept ones included,
  are wgpu's and the driver's, outside the counted peak (on an integrated GPU they are system
  memory beside the cap, like the driver's own heaps above); nothing is mirrored into a counted
  host buffer for the session. Phase 3's two ports do run in the young stage, but their counted
  host copies there are smaller than what the CPU path allocates in the same place (the bullet
  above), so they do not raise the young stage's counted peak (both arms of section 10's Phase 3
  laptop pairs report the 4 GiB cap as their counted peak, where the history spills); the young
  stage's
  estimate (`young.rs`, which counts the CPU's hierarchy) is unchanged and stays an upper bound.

## 5. Watchdogs (TDR) and backends

- **Windows' TDR** resets a device whose work packet runs over about 2 s; Linux's amdgpu lockup
  timeout is 10 s for compute, macOS has its own. Every dispatch is split into bands of whole rows
  of at most 4 Mi cells (rows a multiple of 16), one bind of a dynamic uniform offset per band, so
  a dispatch is a few milliseconds even on lavapipe; one iteration's bands go in one submit; the
  host waits at the end of each phase (and at most 120 s, after which the device counts as hung).
- **Backends.** Vulkan (Linux, Windows, Android), DX12 (Windows), Metal (macOS). Not OpenGL
  compute (the client's GL context stays untouched; wgpu's GL backend is not built), not CUDA. The
  device is created without a surface, so it lives beside Minecraft's GL context in the client and
  works the same on a headless dedicated server.

## 6. Kernels ranked by profile seconds

Wall and CPU seconds of the 2026-10-18 profile (full 16k, 8 threads, 1000 Myr, 500 steps;
`review/pages/sim-time/data.js`); grids read from the code: L1 2,048 x 656 = 1.34 M cells; the
flexure solve on L1 halved (about 1,024 x 328); the climate's coarse grid 128 x 41; L0 250,000
samples; amplify's L3 8,192 x 2,624 = 21.5 M cells (whole on the CPU at the 4 GiB cap, 1 tile;
8 tiles on an integrated GPU, section 3). "Gain" is estimated for a
mid-range GPU and includes each step's uploads and downloads; "beside" spans run beside the plate
step and are off the critical path, so they gain no wall time.

| # | span | wall / CPU s | algorithm (read) | residency | precision, error budget | gain (s) | blockers |
|---:|---|---:|---|---|---|---:|---|
| 1 | `surface:slope_failure` | 83.8 / 600.8 | Jacobi gather over open-sea neighbour pairs, 40 iterations (later ones on the last movers only) | h, sediment, sea mask up; dh down; 40 ping-pong dispatches | f32 on differences from the cell: slope tests within 0.3 mm at 5 km; budget 1 mm per step | 70 to 78 | full sweep instead of the sparse worklist; a CPU branch edits `marine.rs` now |
| 2 | `surface:diffusion_solve` | 109.3 / 840.4 | backward Euler, 5-point flux Laplacian, ADI first guess, Jacobi-preconditioned CG, about 111 iterations, 1e-6 m | h, kd up; dh down; CG vectors resident | f32 cannot hold 1e-6 m on 5 km: solve for the increment against an f64 residual from the CPU (defect correction); budget 1e-6 m on dh | 70 to 90 | two global reductions an iteration; a multigrid branch in flight changes the solver |
| 3 | `plates:write_l1` (relink **ported**, section 13) | 95.8 / 711.3 | per L1 cell: k-d tree 6-nearest over visible samples, a triangle walk, barycentric interpolation of about 14 fields | samples (250 k x about 30 fields) up; 14 L1 fields down | f32 angles with per-plate offsets; budget: the CPU's nearest-sample choice except at ties | 50 to 75 (less now: plates part 2 cut it to one nearest query and one fused layer pass) | a spatial hash replaces kiddo (different tie-breaks); boundary and gap cells stay on the CPU (section 12; plates part 2 is merged: free) |
| 4 | `climate snapshot:cells` | 46.1 / 334.7 | per row an air march (a recurrence with floors), then per cell 12 months of T and P, Köppen, glaciers | about 15 fields up; f32 outputs down; 103 calls | per-cell months in f32 (temperatures to 0.01 K); the march stays a row scan; sums that feed the carbon cycle stay f64 (section 12) | 20 to 30 (less now: the merged climate branch cut the row loop about threefold) | the march is serial along a row (656 rows); the model is f64 closures over belts, regimes and two upwind passes |
| 5 | `surface:advect` | 40.2 / 253.1 | semi-Lagrangian gather, per-plate rotation, bilinear fields, nearest for the 516 MB strata log | fields resident; the log only if its readers move too | rotation in f32 with a per-plate offset; budget one hundredth of a cell in position | 15 to 25 | the log's readers (fields, structure, bake) are on the CPU, so only the continuous fields move |
| 6 | `surface:flexure` | 38.2 / 275.3 | PCG on the thin plate, FFT-spectral preconditioner, to 2 m or 60 iterations | loads, D, C up; deflection resident | f32 plausible: the stop is 2 m | 20 to 28 | an FFT in WGSL (no vendor FFT in wgpu), sizes not powers of two |
| 7 | `deposits` (stage) | 82.0 / 319.0 | per frame: LZ4 decode, octagon window sums, a rule interpreter, TTI; a serial tail | accumulators resident; frames up (160 MB each) | i64 fixed point window sums kept exact in u32 pairs; rules in f32 | 15 to 30 | the rule interpreter has to become WGSL; decode and tail stay CPU (the after-group branch is merged: free) |
| 8 | `surface:glacial` | 33.5 / 170.4 | ice mass balance, flux down the D8 tree, a per-cell bisection of the SIA law | routing up; ice fields resident | f32 bisection to 0.1 m | 10 to 20 | tree order: level-synchronous dispatches |
| 9 | `climate:topography` | 14.3 / 112.6 | the same gather as advection, for the ice and ELA fields | as 5 | as 5 | 8 to 11 | as 5 |
| 10 | amplify middle (**ported**) | 12.0 of 24.0 | stream-power erosion (Jacobi flow), glacial valleys, thermal, deposition on L3 tiles | a window stays on the device for the whole middle | section 7 | measured, section 10 | none left |
| 11 | `snapshot:ebm2d / pc_years` | 30.4 / 215.8 | 864 implicit steps a snapshot on 5,248 cells | state resident | f64 to f32 on a coupled 3 x 3 solve: needs checking | 5 to 15 | too few cells for a GPU; larger CPU pieces are better (profile row 10) |
| 12 | `surface:fields_events` | 34.4 / 197.9 | 26 per-cell fields reading the log, chamfer distances, region labelling, LZ4 | needs the log | f32 | 5 to 10 | the log; the rest is serial |
| 13 | `surface:spl` | 28.5 / 200.6 | implicit stream power along the D8 forest, Newton for n not 1 | routing up | f32 Newton to 1 mm | 5 to 15 | receiver-first sweeps along long rivers |
| 14 | `beside:structure` deformation, frame fields | 49.8, 20.9 (beside) | strain map, then serial Poisson placement; columns from the log | | | 0 (beside) | off the critical path |
| 15 | `surface:pflood` | 32.7 / 49.2 | Priority-Flood + epsilon (Barnes 2014), a heap | | f32 epsilon accumulates on flats | 0 | a priority queue; the order-independent flood branch in flight is the CPU answer |
| 16 | `plates:resample`, `transition`, `fronts`, `climate:clock`, `ice` | 38.7, 26.2, 21.8, 29.6, 29.6 | hull with exact predicates, Dijkstra, B-trees, 1-D EBM in time | | | 0 | not grid work |

## 7. The amplifier's middle on the GPU (what was ported)

- **Algorithm (read, `kernels.rs`, `glacial.rs`; WGSL in `gpu.rs`).** The CPU's Jacobi iterations
  written again one invocation per cell: the flow normaliser and the drainage-area hops, the
  stream-power cut against the incision cap, thermal erosion, the deposit routing; the glacial
  pass's ice depth, its separable maximum and mean filters (the mean Kahan-compensated on
  differences from the centre cell), the lateral share, the cut and free board, and the apply,
  whose dz goes into the erodibility array (unused after erosion: the host keeps its copy).
- **Residency.** One upload after the seed and the first flood, one download (z, area, eroded,
  sediment, the cap and the glacial dz) before the last flood; nothing crosses in between.
- **Precision and error budget.** f32, as on the CPU, and every sum over the eight neighbours in
  the CPU's order; the differences are rounding (fused multiply-adds the shader compiler may form,
  `pow` for exponents other than 1, 1.5 and 2) and the branches that rounding can flip (the
  steepest receiver, a positive slope). Budget, checked by
  `surface::amplify::tests::the_gpu_passes_stay_within_their_error_budget_of_the_cpu` on one whole
  glaciated window: z, eroded, sediment and the cap differ by more than 1 cm in under 0.1 % of
  cells, the mean difference is under 1 mm, the largest under 5 m (a flipped receiver at a
  saddle); the glacial statistics agree to 0.1 %; and through the tiled ladder under 1 % of L3
  cells differ by over 1 cm. Measured on lavapipe: section 8.
- **One documented difference.** The glacial lateral share is 1 where a neighbour's ice surface
  is not finite; the CPU gets 1 for NaN (what the code produces) and could get 0 for an infinity
  (which it does not produce).
- **What stays on the CPU.** The seed, the floods (a priority queue) and the mountain character
  (7.6 s; a per-cell noise sum, the next port in this stage).

## 8. Testing

- **The CPU path is the reference.** Every GPU kernel has a test that runs both and holds the GPU
  to its error budget; there is no golden GPU output, because worlds may differ by GPU, driver and
  run (Ben, 2026-10-18).
- **On the test VM** the flake's dev shell puts the Vulkan loader on `LD_LIBRARY_PATH` and adds
  Mesa's lavapipe through `VK_ADD_DRIVER_FILES` (which adds to the system's drivers, so a real GPU
  still wins elsewhere). The GPU tests open the device with software allowed; without any adapter
  they print why and pass (skip).
- **Tests:** `deeptime-gpu`: a kernel run in bands of a 131,072-cell-wide raster comes back right;
  a bad shader is an error, not a panic. `deeptime-core`: the tolerance test above;
  `amplify::tests::auto_tiles_a_level_the_device_could_not_keep_whole` (the stream decision with
  a stub device, no GPU needed); `compute::tests::modes_parse_and_print`. Java:
  `ServerConfigTest.computeIsAutoCpuOrGpu`; `NativeLinkTest.createWithTakesTheEpochFlags` now
  runs a session with each compute flag and refuses both together (16 is its unknown flag, since
  8 is the GPU's).
- **Phase 2 tests:** `plates::l1::gpu::tests::the_gpu_relink_matches_the_cpu_but_at_ties`
  (section 13) and `climate::tests::the_gpu_cells_stay_within_their_error_budget_of_the_cpu`
  (section 14), both against the CPU path and skipped without an adapter.
- **Phase 3 tests:** `surface::diffusion::multigrid::tests::the_gpu_cycle_matches_the_cpu_cycle`
  and `the_gpu_cycle_solve_meets_its_tolerance_and_keeps_the_mass` (section 15),
  `surface::marine::tests::the_gpu_slope_failure_matches_the_cpu` (section 16); against the CPU
  path, skipped without an adapter, run on lavapipe by `cargo test -p deeptime-core --features
  gpu --lib -- --nocapture the_gpu`, which prints each case's figures.
- **Changed:** `tests/memory_estimate.rs` runs on the CPU (`Session::set_compute`), because the
  driver's memory sits outside the count it checks (section 4).
- **Worlds.** A 100 Myr planet on the CPU and with the GPU forced on lavapipe, compared with
  `tools/review/p1-17-cmp.py` and `p1-17-hyps.py`: section 10.
- **Speed** only on real GPUs (the laptop's 8050S, Ben's rule of 2026-10-18), GPU against CPU in
  A/B pairs on the same machine and binary.

## 9. Amdahl on a mid-range GPU (estimated)

- **Bandwidth.** The heavy kernels already keep 7 to 7.6 of 8 cores busy and are memory-bound
  stencils. Eight VM cores stream perhaps 60 to 100 GB/s; a T4 has 320 GB/s (and PCIe 3 at about
  12 GB/s for transfers), the laptop's 8050S 256 GB/s shared with the CPU (no PCIe). So a
  resident stencil runs 3 to 6 times faster; one whose fields cross every step, less. **Measured** check (section
  10): the amplifier's passes, resident with one upload and one download, ran 8 to 9 times faster
  on the laptop's 8050S than on its own 8 CPU threads, above that range; the per-step kernels of
  rows 1 to 6 pay transfers every call unless the fields stay resident (section 12, item 1), so
  the estimate below is kept.
- **The addressable part.** Rows 1 to 10 of section 6 on the critical path: 109.3 + 83.8 + 95.8 +
  46.1 + 40.2 + 38.2 + about 40 of deposits + 33.5 + 14.3 + 12.0 = about 513 s of 1,242.8 s.
- **The estimate.** The rows' gains sum to 290 to 400 s: 1,242.8 s becomes about 845 to 955 s
  (1.3 to 1.47 times). With an infinitely fast GPU and free transfers: about 730 s. The serial
  243 s on one core (the flood, the plate transition, fronts, marine fill, the keyframe render,
  belts, record, collision, collapse) and the plate step's other laps bound it.
- **Against the gate.** 900 s needs most of rows 1 to 6 on the GPU, or the CPU branches' gains
  on top of a smaller set. A player's PC with a weaker CPU than the VM's gains more from the
  same GPU work.

## 10. Measurements

**Correctness on lavapipe** (the test VM, Mesa 26.2.3 llvmpipe, `cargo test -p deeptime-core
--features gpu gpu -- --nocapture` through `tools/remote/detach.sh`; 2026-10-19). Largest
difference, share of cells over 1 cm, mean difference, all in metres:

| test | z | eroded | sediment | incision cap |
|---|---|---|---|---|
| one glaciated window, 192 x 64 | 1.8e-4, 0, 2.9e-7 | 1.3e-5, 0, 5.5e-8 | 2.2e-5, 0, 6.5e-8 | 1.7e-4, 0, 2.5e-7 |
| the ladder in tiles, L3 384 x 128 | 9.8e-4, 0, 1.5e-6 | | | |

The glacial pass carved 3,033.548126 m-cells on the CPU and 3,033.548085 on the GPU (1.4e-8
apart), raised 68.85097 against 68.85096, over the same 1,276 glacier cells; the floods after it
were identical. So lavapipe sits three to four orders of magnitude inside the budget of section 7;
the budget stays as it is for hardware drivers that contract multiply-adds.

**A 100 Myr planet, CPU against the GPU forced on lavapipe** (the test VM; `earthlike_full_16k`,
scattered continents, `deeptime stage young --length-myr 100 --threads 8`, then `deeptime stage
amplify --in young100.ckpt --threads 8 --bake --compute cpu|gpu` from the same checkpoint). The
GPU ran every window: `"compute": {"mode": "gpu", "device": "llvmpipe ...", "device_tiles": 8,
"tiles": 8, "error": null}`, the level in 8 tiles with a halo of 414 (section 3; the CPU ran it
whole). `tools/review/p1-17-cmp.py`: 61 files, 7 differ (the elevation, the detail amplitude,
the rivers, the horizons, the keyframe and its index, the manifest), as expected of f32 rounding.
`tools/review/p1-17-hyps.py`: every figure the same to its printed precision (land 0.226, z50
455 m, z90 1,908 m, above 2 km 0.094, ice 0.075, T 14.9 °C, ...); the stage's own summary too
(mean change 37.6 m, eroded 2.458e4 km³, glacial carved 1.203e4 km³); CO2 551.208 ppm in both,
as it must be, since the climate ran before the checkpoint. Cell by cell (a scratch
script reading both worlds' tile packs): the baked elevation (6,656 x 3,329 cells in sixteenths
of a block) differs in 0.0006 % of cells, by at most 0.12 blocks, and the detail amplitude in
0.0001 %, by at most 0.06 blocks; no cell by a whole block. Wall time on the VM (not a speed
figure: lavapipe runs on all 32 cores) 63.8 s CPU, 66.5 s GPU; resident peak 4.1 and 4.2 GB.

**Speed: the laptop's Radeon 8050S** (AMD Strix Halo, RADV, Mesa 26.1.8, integrated, 2026-10-19).
Binary built on the VM (`cargo zigbuild --release -p deeptime-cli --target
x86_64-unknown-linux-gnu.2.35`), the VM's 100 Myr checkpoint above, `systemd-run --user --scope
-p CPUQuota=800% nice -n 19 deeptime stage amplify --in young100.ckpt --threads 8 --compute
gpu|cpu`, three pairs per set alternating which goes first, nothing else heavy on the laptop
(load 5 to 7 from desktop services). Seconds, mean of three (each run's own figures in the
build log):

| set | arm | wall | stage total | erode | glacial | thermal | deposit | transfer | flood | character |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| default 4 GiB cap | CPU, whole | 27.7 | 23.5 | 2.53 | 6.57 | 1.37 | 0.80 | 0 | 2.67 | 7.67 |
| | GPU, 8 tiles | 21.4 | 17.2 | 0.73 | 0.10 | 0.13 | 0.17 | 0.30 | 3.60 | 9.70 |
| 8 GiB cap | CPU, whole | 29.8 | 25.4 | 2.60 | 7.93 | 1.33 | 0.80 | 0 | 2.60 | 8.23 |
| | GPU, whole | 18.8 | 14.4 | 0.60 | 0.27 | 0.10 | 0.20 | 0.23 | 2.73 | 8.43 |

- The ported passes: 11.3 and 12.7 s on the CPU, 1.4 s on the GPU with the transfers, 8 to 9
  times faster, even in tiles that hold 1.85 times the cells.
- In tiles the CPU's own parts get dearer (the seed, the flood and the character over the halos:
  +0.2, +0.9 and +2.0 s), so the stage gains 6.3 s; whole it gains 11.0 s. A player's PC with
  more memory than the default cap allows gets the second.
- Every GPU run ended with the CPU's summary figures (mean change 37.6 m, eroded 2.458e4 km³,
  glacial carved 1.203e4 km³).
- No whole-run A/B: the GPU path only touches the amplify stage, which runs once at the end, so a
  1000 Myr pair would show the same 6 to 11 s inside the noise of a 20-minute run. The full run's
  figure is therefore 1,242.8 s less 6 to 11 s by arithmetic, not measured.

**Phase 2: the L1 relink and the climate cells** (2026-10-19; branch `gpu-phase2` from master
b337fa73, the climate sub-agent's branch and masters fb38204f, 7d60fc4a and 26ce664e merged in).

*Correctness on lavapipe*: sections 13 and 14.

*The same state on both (VM, lavapipe, master fb38204f).* Two temporary switches (not
committed) ran a full 16k planet's 100 Myr history (`deeptime stage volcanoes --preset
earthlike_full_16k --start scattered_continents --length-myr 100 --threads 8 --set seed=1
--compute gpu`) with every L1 sampling, and every snapshot's cells, done twice from the same
state, on the CPU and on the device, and printed the signed differences:

- **The relink**, 50 samplings, 67.2 M cells. 10,390 links on another triangle (0.015 %) and
  4,981 with another nearest corner, all but one of those on the same triangle (the larger of two
  near-equal weights). Where the triangle agrees the weights are within about 7.7e-4 in a
  sampling, coarser than the test planet's 1.6e-5 (section 13); in 14 samplings one cell or a few
  came out up to 0.38 apart, cells on an edge that the CPU's f64 test put outside (the nearest
  sample's one-hot weight) and the device's f32 test inside. Signed, over all 67.2 M: elevation
  27.29 M cells up and 27.28 M down, mean -3.8e-5 m; land 19,622,726 cells against the CPU's
  19,622,728; continental crust 29 cells up and 24 down; uplift, thickness, heat flow, crust age
  and the seven rock pools as many up as down; the plate and the group id 1 cell each. **No layer
  leans.**
- **The cells**, 7 snapshots of 355,000 to 435,000 land cells. Temperature means -1.3e-7 to
  +5.5e-9 K (at most 7.6e-6 K in a cell), rain -2e-8 to +1.9e-7 relative, aridity and the
  equilibrium line within 1e-7 relative; sea ice, the glacier flags, the Köppen classes, the
  glaciated area and each band's ice sheet identical. The weathering sums read these fields in f64
  on the CPU, so the carbon cycle sees differences of 1e-7. **No field leans.**

*Whole histories, CPU against GPU.* The carbon cycle carries a last-bit difference into CO2
(section 12), so these comparisons run whole histories from scratch: the command above with
`--compute cpu|gpu` on the VM (lavapipe held to 8 threads), then `tools/review/p1-17-hyps.py`
and `p1-17-cmp.py`. Five planets (seeds 1 to 5) on master 7d60fc4a, CPU / GPU:

| seed | CO2 ppm | T °C | glaciated land | sea level m | land | land z90 m |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 544 / 427 | 14.1 / 13.7 | 0.092 / 0.088 | 27.5 / 29.5 | 0.243 / 0.256 | 2,150 / 1,857 |
| 2 | 681 / 681 | 15.5 / 15.4 | 0.133 / 0.125 | 13.6 / 16.1 | 0.243 / 0.248 | 2,217 / 2,109 |
| 3 | 508 / 505 | 14.6 / 14.6 | 0.055 / 0.057 | 39.6 / 39.2 | 0.251 / 0.251 | 1,257 / 1,281 |
| 4 | 385 / 455 | 12.6 / 13.1 | 0.148 / 0.098 | 12.5 / 26.4 | 0.268 / 0.274 | 2,308 / 2,450 |
| 5 | 646 / 622 | 16.3 / 16.1 | 0.024 / 0.026 | 42.3 / 44.3 | 0.234 / 0.242 | 907 / 704 |
| mean | 553 / 538 | 14.6 / 14.6 | 0.090 / 0.079 | 27 / 31 | 0.248 / 0.254 | 1,768 / 1,680 |

To tell a GPU's difference from the model's own sensitivity, seed 1 also ran (master fb38204f)
on the CPU with one f32 ulp added to the first, second or third weight of every covered cell (a
temporary switch), with the GPU and the same additions, and with one port at a time:

| world (seed 1, master fb38204f) | CO2 ppm | T °C | glaciated land | sea level m | land | land z90 m |
|---|---:|---:|---:|---:|---:|---:|
| CPU | 552 | 14.0 | 0.106 | 25.1 | 0.245 | 2,243 |
| CPU, an ulp on the 1st, 2nd, 3rd weight | 546, 633, 598 | 14.1, 15.1, 15.0 | 0.091, 0.069, 0.060 | 27.4, 32.8, 35.4 | 0.244, 0.249, 0.247 | 2,131, 1,657, 1,568 |
| cells on the GPU (none, 2nd, 3rd) | 500, 607, 618 | 13.7, 15.0, 15.1 | 0.105, 0.062, 0.069 | 24.0, 35.8, 33.3 | 0.238, 0.249, 0.247 | 2,354, 1,580, 1,644 |
| relink on the GPU (none, 2nd, 3rd) | 514, 574, 425 | 13.7, 14.6, 13.6 | 0.107, 0.060, 0.082 | 24.4, 36.7, 31.3 | 0.242, 0.264, 0.257 | 2,294, 1,401, 1,928 |
| both on the GPU (none, 1st, 2nd, 3rd) | 451, 451, 553, 421 | 13.8, 13.8, 14.5, 13.6 | 0.092, 0.091, 0.062, 0.080 | 27.9, 28.4, 36.4, 31.9 | 0.255, 0.255, 0.266, 0.259 | 1,978, 1,964, 1,408, 1,870 |
| both, Radeon 8050S (laptop bench, three runs alike) | 457 | 13.8 | 0.090 | 28.6 | 0.254 | 1,916 |

And on master c767da2c with the climate branch, before the cadence change (one ulp on the first
weight only):

| world (seed 1, before fb38204f) | CO2 ppm | T °C | glaciated land | sea level m | land | land z50 m | land z90 m | land over 2 km |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| CPU (VM; the laptop's CPU path gives the same to the digit) | 735 | 15.5 | 0.062 | 36.9 | 0.239 | 463 | 1,785 | 0.088 |
| CPU, one weight an ulp off (VM) | 653 | 14.5 | 0.083 | 31.4 | 0.263 | 466 | 1,866 | 0.093 |
| GPU relink, lavapipe (VM) | 619 | 14.8 | 0.058 | 37.5 | 0.257 | 457 | 1,487 | 0.070 |
| GPU relink and cells, lavapipe (VM) | 597 | 14.7 | 0.054 | 38.3 | 0.259 | 453 | 1,438 | 0.068 |
| GPU relink, Radeon 8050S (laptop, three runs alike) | 641 | 14.5 | 0.082 | 31.2 | 0.264 | 465 | 1,855 | 0.096 |
| GPU relink and cells, Radeon 8050S (laptop, three runs alike) | 653 | 14.5 | 0.084 | 31.1 | 0.264 | 465 | 1,917 | 0.098 |

(The laptop's land figures come from the bench's `surface_final` line, which can differ from
the stage's hypsometry in the last digit.)

- **A last-bit change picks among a few histories.** Seed 1's worlds fall into families set by
  which weight got the ulp, whatever ran on the GPU: every run with the second weight's ulp ended
  warm, with little ice and a high sea (CO2 553 to 633 ppm, 14.5 to 15.1 °C, glaciated 0.060 to
  0.069, sea level 32.8 to 36.7 m). An ulp moves this planet as far as a GPU does.
- **Across planets the GPU leans no way that matters for CO2 or temperature.** Within seed 1,
  nine of ten GPU runs ended below the CPU run of their family in CO2 (by 26 to 177 ppm; the
  cells alone with the third weight's ulp ended 20 above), but these runs are not independent
  draws: the device's rounding is the same in each, only the ulp differs. Across the five seeds
  CO2 differs both ways (-117, 0, -3, +70 and -24 ppm), the mean temperature is the same, and the
  glaciated share and the sea level go both ways.
- **Land is the one figure that may lean.** It ended higher on the GPU in four seeds of five
  (equal in the fifth; +0.006 on average, 2 % of the land), and on seed 1 in six of the seven
  runs with the relink on the device (by 0.010 to 0.017). A sign test on four of four non-tied
  seeds gives p = 0.125 two-sided, so this is not shown, and the same-state comparison above
  finds no lean in any sampling (the land count off by 2 cells in 19.6 million over all 50), so
  if it is real it comes through the history's feedbacks, not the relink's output. A larger
  paired ensemble (ten more seeds, about 2.5 hours of the VM) would settle it; the ports do not
  wait for it.
- `p1-17-cmp.py`: the keyframes differ from the first (kf-0) on, as expected once the first
  sampling's weights differ in the last bits.
- Each world is reproducible on its own device: three Radeon runs of a set ended on the same
  figures, and the laptop's CPU worlds equal the VM's.

*Speed: the laptop's Radeon 8050S*, as in Phase 1 (binary built on the VM, `systemd-run --user
--scope -p CPUQuota=800% nice -n 19 deeptime bench --preset earthlike_full_16k --start
scattered_continents --length-myr 100|1000 --threads 8 --compute auto|cpu`, pairs alternating
which goes first). Seconds per run, means; `auto` also runs the amplifier on the GPU (section 10
above). The first two sets ran quiet (load 5 to 9); the third, after master fb38204f, with the
laptop in its power-saver profile and other sessions' work holding the load at 16 to 30, which
made every stage about 1.6 times slower and scattered the walls by 100 s, so only its kernels'
laps compare.

| set | arm | wall | plates stage | write_l1 | relink (CPU part) | device | tree (beside the device) | layers | amplify |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| relink, 100 Myr | CPU | 326.6 | 34.0 | 14.9 | 8.8 | | 1.4 | 3.8 | 52.7 |
| | GPU | 303.9 | 28.6 | 9.2 | 2.4 | 1.0 | 1.7 | 3.8 | 28.1 |
| relink and cells, 100 Myr | CPU | 325.7 | 34.3 | 15.1 | 8.9 | | 1.4 | 3.8 | 53.2 |
| | GPU | 298.8 | 29.3 | 9.3 | 2.5 | 1.0 | 1.7 | 3.8 | 28.2 |
| master fb38204f, 100 Myr, loaded | CPU | 538.0 | 56.9 | 22.6 | 12.9 | | 2.1 | 6.1 | 77.8 |
| | GPU | 529.5 | 52.3 | 15.2 | 4.1 | 2.2 | 2.8 | 6.3 | 39.1 |
| 1000 Myr, master 7d60fc4a, loaded (one pair, GPU first) | CPU | 2,402.1 | 524.4 | 200.9 | 127.4 | | 15.1 | 47.6 | 49.7 |
| | GPU | 2,769.6 | 545.6 | 161.4 | 61.7 | 15.8 | 21.7 | 62.5 | 30.7 |

| set | arm | climate stage | snapshot | cells | cells up | cells down | snapshot's 2-D model |
|---|---|---:|---:|---:|---:|---:|---:|
| relink and cells, 100 Myr (a snapshot every 10 Myr) | CPU | 57.3 | 16.0 | 4.60 | | | 6.8 |
| | GPU | 54.0 | 12.3 | 0.63 | 0.23 | 0.20 | 7.2 |
| master fb38204f, 100 Myr, loaded (every 20 Myr) | CPU | 81.2 | 14.8 | 4.00 | | | 6.7 |
| | GPU | 91.5 | 11.4 | 1.03 | 0.40 | 0.50 | 6.6 |
| 1000 Myr, master 7d60fc4a, loaded (every 20 Myr) | CPU | 278.2 | 81.9 | 23.30 | | | 35.9 |
| | GPU | 322.9 | 75.5 | 5.70 | 2.60 | 2.00 | 43.2 |

- **The relink**: 8.9 s on the CPU becomes 1.0 s on the device beside the k-d tree's build and
  2.5 s of CPU work for the 5.3 % of cells the device leaves (3,578,424 of 67,174,400 over the
  run); `write_l1` 15.1 to 9.3 s, 1.6 times faster, 5.8 s per 100 Myr.
- **The cells**: 4.6 s on the CPU become 0.63 s on the device and 0.43 s of transfers, 3.5 s per
  100 Myr at a snapshot every 10 Myr. Master fb38204f takes a snapshot every 20 Myr, which halves
  the cells' work on both paths, so the port is worth about 1.8 s per 100 Myr there (the loaded
  set's 4.0 s against 1.9 s is the same ratio, 2.1, at the loaded laptop's speed). The rest of
  the snapshot (the 2-D model, the march along the rows, the ice) stays on the CPU.
- The wall times are of different worlds once the GPU is on (the tables above), so they mix the
  kernels' gain with the world's own cost; the kernels' laps are the cleaner figure.
- **The 1000 Myr pair** (master 7d60fc4a, `--length-myr 1000`, the laptop loaded as in the third
  set). Walls 2,769.6 s with the GPU and 2,402.1 s on the CPU, but the GPU's arm ran in a busier
  window: the stages neither port touches (surface, volcanoes, impacts, structure, deposits,
  young, the finish) took 1,865 s against 1,474 s, 27 % longer, and the plate and climate work
  outside the ports 19 and 23 % longer. So the wall says nothing about the GPU; the kernels'
  laps do: `write_l1` 200.9 to 161.4 s (the relink with its tree 142.5 to 83.4 s; the layers,
  which do not move, 47.6 against 62.5 s), the cells 23.3 to 10.3 s with their transfers, the
  amplifier 49.7 to 30.7 s. That is 72 s less on the ported kernels even in the busier window,
  about 86 s with the layers at the CPU arm's 47.6 s.
- **The whole run on a player's machine** (from the laps, not the walls): the GPU takes about
  72 to 86 s, 3 to 3.6 %, off this laptop's 2,402 s for 1000 Myr of the full 16k. The time is
  elsewhere: the surface stage is half the run (1,203 s on the CPU arm; slope failure 273 s, the
  diffusion solve 173 s, flexure 116 s, advection 111 s, the flood 72 s), which is where the next
  ports go (section 12).
- Over 1000 Myr the device leaves 10.0 % of the relink's cells to the CPU (67.2 M of 671.7 M),
  twice the first 100 Myr's share, and they cost 61.7 s against the device's 15.8 s: the
  second-group search (section 13) is now the larger half of the relink.
- The two 1000 Myr worlds, CPU / GPU: CO2 10,011 / 7,981 ppm, 22.3 / 20.2 °C, glaciated land
  0.158 / 0.159, sea level -19.1 / 20.5 m, land 0.215 / 0.220, land z90 4,508 / 4,477 m. Both
  end far above today's CO2; that is master 7d60fc4a's model at 1000 Myr, not the GPU's.

**Phase 3: the diffusion cycle and the slope failure** (2026-10-19; branch `gpu-phase3` from
master 3e7edd49).

*Correctness on lavapipe*: sections 15 and 16.

*The same state on both (VM, lavapipe).* A temporary harness (not committed) ran seed 1's 100
Myr history (`deeptime stage volcanoes --preset earthlike_full_16k --start scattered_continents
--length-myr 100 --threads 8 --set seed=1 --compute gpu` with `DEEPTIME_GPU_KERNELS=surface:`)
with every surface step's diffusion solve and slope failure done twice from the same state, on
the CPU and on the device (the device's result carried on), and printed the signed differences:

- **The diffusion solve**, 50 solves on L1 (1,343,488 cells): the same heights to the bit in
  every cell of every solve, in the same number of iterations (17.64 a solve on average). On
  lavapipe the device's cycle rounds as the CPU's does (the kernels keep the CPU cycle's order of
  operations), so the conjugate gradients see the same preconditioner. A hardware driver that
  forms multiply-adds gives another z in the last bits, inside the tests' budget (section 15).
- **The slope failure**, 50 calls, about 254,800 cells moving a call. Metres moved 8.230063e8 on
  both (the device's total 1.5e-8 relative below the CPU's). Heights, device less CPU: the mean
  per call -1.0e-7 to +1.3e-8 m (43 calls below zero, 7 above; -2.3e-8 m over all), the RMS at
  most 2.5e-5 m, the largest 1.5e-3 m. No cell is land on one path and sea on the other. The
  small lean below zero is the f32 round-off of matter: the device's changes summed to -1.52
  m-cells over the 50 calls (at most 0.145 in a call), the CPU's to +0.028, that is 1e-6 m over
  the L1 grid in 100 Myr. **No field leans that matters.**

*Whole histories, kernel by kernel: the land lean.* Phase 2's five seeds left one figure open:
land ended higher on the GPU in four seeds of five (p = 0.125). The coordinator asked for ten
more seeds to settle it. Seeds 6 to 15 ran the command above on the VM, on the CPU and with the
GPU forced on lavapipe (8 threads): every ported kernel on the device (master 3e7edd49's binary,
so the relink and the cells; the amplifier runs after the volcanoes stage), then with this
branch's binary one group at a time through `DEEPTIME_GPU_KERNELS`: `surface:` (this phase's two
kernels), `plates:` (the relink), `climate:` (the cells). This branch's CPU path gives master's
CPU worlds to the digit (seed 6: land 0.267, CO2 459 ppm; seed 1: Phase 2's 0.243 and 544 ppm).
Three CPU controls ran the same seeds with changes that do nothing physical, to see how far the
CPU's own worlds move under a perturbation (`--compute cpu` with `--set`): the submarine critical
slope 1e-10 relative higher (`surface.submarine_slope_deg=2.50000000025`), the same lower
(`2.49999999975`), and a change of f32's rounding size on every cell every step: the land and
marine diffusivities, the stream-power coefficient and the climate's heat diffusivity each 1e-7
relative higher (`surface.kd_land_m2_yr=2.0000002`, `surface.kd_marine_m2_yr=100.00001`,
`surface.k_spl=1.50000015e-9`, `climate.diffusivity_w_m2_k=0.6490000649`). Land, the area share
above the sea:

| world | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| CPU | 0.267 | 0.271 | 0.266 | 0.273 | 0.213 | 0.234 | 0.275 | 0.262 | 0.222 | 0.276 |
| every kernel on the GPU | 0.277 | 0.287 | 0.277 | 0.275 | 0.218 | 0.236 | 0.277 | 0.251 | 0.251 | 0.279 |
| `surface:` only | 0.285 | 0.274 | 0.278 | 0.276 | 0.232 | 0.238 | 0.273 | 0.262 | 0.217 | 0.284 |
| `plates:` only | 0.265 | 0.282 | 0.282 | 0.272 | 0.226 | 0.233 | 0.278 | 0.254 | 0.242 | 0.278 |
| `climate:` only | 0.283 | 0.276 | 0.280 | 0.277 | 0.231 | 0.241 | 0.276 | 0.267 | 0.252 | 0.276 |
| control: the CPU, critical slope 1e-10 higher | 0.268 | 0.264 | 0.271 | 0.274 | 0.236 | 0.233 | 0.276 | 0.269 | 0.222 | 0.280 |
| control: the CPU, critical slope 1e-10 lower | 0.267 | 0.270 | 0.267 | 0.272 | 0.221 | 0.231 | 0.274 | 0.248 | 0.223 | 0.279 |
| control: the CPU, four rates 1e-7 higher | 0.283 | 0.275 | 0.271 | 0.272 | 0.221 | 0.235 | 0.276 | 0.249 | 0.226 | 0.282 |

Each world less the CPU's, over the ten seeds (up / down / equal; the mean; the two-sided sign
test):

| world | land | land mean | sign p | CO2 ppm up / down, mean | T °C mean | sea level m mean | glaciated land mean |
|---|---|---:|---:|---|---:|---:|---:|
| every kernel on the GPU | 9 / 1 / 0 | +0.0069 | 0.021 | 4 / 6, -3 | -0.13 | -1.1 | +0.008 |
| `surface:` only | 7 / 2 / 1 | +0.0060 | 0.18 | 6 / 4, -9 | -0.06 | +0.2 | +0.003 |
| `plates:` only | 6 / 4 / 0 | +0.0053 | 0.75 | 3 / 7, -24 | -0.16 | -1.0 | +0.002 |
| `climate:` only | 9 / 0 / 1 | +0.0100 | 0.004 | 5 / 5, -16 | -0.15 | +0.7 | +0.003 |
| control: the CPU, critical slope 1e-10 higher | 7 / 2 / 1 | +0.0034 | 0.18 | 5 / 5, -1 | -0.13 | -0.2 | +0.002 |
| control: the CPU, critical slope 1e-10 lower | 4 / 5 / 1 | -0.0007 | 1.0 | 8 / 2, +16 | +0.08 | +0.4 | -0.001 |
| control: the CPU, four rates 1e-7 higher | 8 / 2 / 0 | +0.0031 | 0.11 | 6 / 4, +20 | +0.14 | +0.7 | 0.000 |

- **Land leans up on the GPU; it is settled.** With Phase 2's seeds 1 to 5 (every kernel on the
  device, master 7d60fc4a: up in four, equal in one), the GPU's land is higher in 13 of 15
  seeds, equal in 1, lower in 1: p = 0.002 two-sided. The size is +0.007 on average, about 2.6 %
  of the land, a quarter of the spread between seeds (0.024 on the CPU, seeds 6 to 15). It
  holds against the controls too: over the four GPU sets and the three controls, the GPU's
  worlds sit +0.005 above the controls' (shuffling the seven labels within each seed gives that
  much in 0.14 % of 100,000 shuffles; the GPU sets' mean above the controls' mean in 9 seeds of
  10).
- **No one kernel carries it.** Every group alone leans up, the climate cells most (9 of 9
  non-tied seeds), the surface kernels and the relink less; they do not add (each alone is about
  as large as all together). In each seed the CPU's world is the lowest or the second lowest of
  the CPU's and the four GPU sets' worlds but once (mean rank 1.8 of 5; 3 if it were one draw
  among five).
- **Yet no kernel leans on the same state.** The relink (land off by 2 cells in 19.6 million over
  50 samplings), the cells (no field), the diffusion solve (identical) and the slope failure (no
  cell across the land line) each give the CPU's state back, so the lean comes from the history:
  a small difference anywhere, carried through the plate, climate and surface feedbacks for 100
  Myr, ends in a world with more land more often than not.
- **Part of it is the model's, not the GPU's.** The controls lean up as well, less: pooled, 19
  seeds up, 9 down, 2 equal, +0.0019 (sign p 0.09); the f32-sized one 8 up, 2 down, +0.0031; the
  slope one way 7 / 2, +0.0034, the other way 4 / 5, -0.0007. The worlds that move further from
  the CPU's lean further: the mean distance from the CPU's land is 0.007 to 0.010 for the GPU sets
  and 0.003 to 0.006 for the controls, and the lean is about 0.5 to 1.0 of that distance in each
  but the slope-lower control. The GPU's continuing f32 differences in four kernels move a world
  further than any control does, so two readings remain: the model answers any perturbation
  with a little more land, in proportion to how far the perturbation carries (the GPU's carries
  furthest), or the kernels carry a small bias that only a history shows. These ten seeds do not
  separate them: against the f32-sized control alone the GPU sets sit +0.004 higher (one-sided
  p 0.06). A CPU control that moves the worlds as far as the GPU does would.
- **What it means.** A full 16k world from the GPU has on average about 0.7 % of the planet's area
  more land than the CPU's world from the same seed (0.5 % more than the controls' worlds),
  against a spread of 2.4 % between seeds; no other field leans, and the GPU's worlds repeat on
  one device (below). A later port's whole-history check should compare against a CPU control
  as well as the CPU.
- **CO2, temperature and the sea level do not lean**: CO2 goes either way in every set (means
  -3 to -24 ppm on the GPU, -1 to +20 ppm in the controls, against a spread of 100 to 120 ppm
  between the pairs), the temperature by -0.16 to +0.14 °C, the sea level both ways.
  `present_co2_factor` is 1.000 in every run, so the natural CO2 the CO2 investigation uses
  equals the CO2 here.

*Speed: the laptop's Radeon 8050S* (binary built on the VM as before; `systemd-run --user --scope
-p CPUQuota=800% nice -n 19 deeptime bench --preset earthlike_full_16k --start
scattered_continents --length-myr 100 --threads 8 --compute auto|cpu`, three pairs, GPU first in
pairs 1 and 3, CPU first in pair 2). The laptop was on the **performance** power profile this time
(Ben's setting, left alone; Phase 2's loaded sets ran on power-saver). Load (1-minute average)
before each run: pair 1 GPU 4.65, CPU 11.75; pair 2 CPU 4.26, GPU 13.86; pair 3 GPU 12.71, CPU
12.62. The bench itself holds the load near 12 (its 8 threads and the desktop), so a figure near
12 before the second run of a pair is the first run's tail, and near 4 to 5 is a quiet start.
`auto` runs every ported kernel (the amplifier, the relink, the cells and this phase's two), so
the walls hold every phase's gain; this phase's are the surface and young stages' lines.

| arm | wall | plates | climate | surface | its diffusion | diffusion solve | cycle (lap) | slope failure | young | its diffusion | amplify |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| CPU, pair 1 / 2 / 3 | 146.7 / 153.1 / 154.5 | 17.8 / 18.4 / 18.2 | 25.0 / 25.8 / 27.2 | 35.5 / 36.4 / 35.9 | 11.7 / 12.0 / 11.9 | 6.3 / 6.4 / 6.4 | 4.5 / 4.7 / 4.7 | 5.3 / 5.4 / 5.3 | 7.0 / 7.2 / 7.5 | 3.6 / 3.7 / 3.9 | 25.1 / 26.2 / 26.8 |
| GPU, pair 1 / 2 / 3 | 123.2 / 125.3 / 123.2 | 14.5 / 15.9 / 14.9 | 22.9 / 23.9 / 23.3 | 29.8 / 30.9 / 30.1 | 7.3 / 7.2 / 7.0 | 5.4 / 5.3 / 5.1 | 3.7 / 3.6 / 3.5 | 1.8 / 1.7 / 1.7 | 5.3 / 5.2 / 5.3 | 2.0 / 1.9 / 1.9 | 13.7 / 13.5 / 13.5 |
| CPU, mean | 151.4 | 18.1 | 26.0 | 35.9 | 11.9 | 6.37 | 4.63 | 5.33 | 7.24 | 3.73 | 26.0 |
| GPU, mean | 123.9 | 15.1 | 23.4 | 30.3 | 7.17 | 5.27 | 3.60 | 1.73 | 5.26 | 1.93 | 13.6 |

("its diffusion" is the stage's diffusion timer: the solve, the slope failure and their
bookkeeping; the cycle's lap counts the young stage's solves too. The GPU's compute laps:
`mg_setup_device` 0.1 s, `mg_cycle_device` 3.5 to 3.7 s, `slope_failure_device` 2.1 to 2.2 s
with the young stage's calls; CG iterations a step 17.3 on the GPU, 17.6 on the CPU, different
worlds.)

- **The slope failure** 5.33 to 1.73 s per 100 Myr in the surface stage, 3.1 times faster,
  with every pass over every cell and the transfers; the CPU's sparse active set does less work
  but at a CPU's bandwidth.
- **The diffusion solve** 6.37 to 5.27 s (the cycle's lap 4.63 to 3.60 s with the young
  stage's): 1.2 times faster. The cycle is a memory-bound stencil, and on this APU the GPU and
  the CPU share one memory; the CG's f64 work stays on the CPU.
- **Per 100 Myr** the phase takes 5.7 s off the surface stage and 2.0 s off the young stage
  (whose five L2 steps call both), 7.7 s of the 27.5 s between the arms' walls (the rest is the
  amplifier's 12.4 s, the relink's and the cells' 5.6 s, and other stages' differences in
  different worlds).

*Speed over 1000 Myr* (the same command with `--length-myr 1000`, two pairs). Pair K ran the CPU
first (load 4.64 before it, 12.14 before the GPU, the CPU's tail), pair L the GPU first (load
3.50 before it, 11.10 before the CPU). The laptop suspended for 28 minutes in the middle of L's GPU
arm; the bench's monotonic clock does not count the sleep (825.9 s, against 2,523 s by the clock
less the 1,701 s asleep), and the device came back without a fault. The two runs of each arm gave
the same world to the digit, so the GPU path repeats itself on one device.

| arm | wall | plates | `write_l1` | climate | cells (with transfers) | surface | slope failure | diffusion solve | cycle (lap) | marine fill | young | amplify |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| CPU, K / L | 973.2 / 937.9 | 220.0 / 218.3 | 77.5 / 76.6 | 112.8 / 110.7 | 8.8 / 8.7 | 453.7 / 451.2 | 90.3 / 90.1 | 62.5 / 62.2 | 39.4 / 39.1 | 37.2 / 36.9 | 11.0 / 8.5 | 29.3 / 25.0 |
| GPU, K / L | 900.3 / 825.9 | 216.9 / 201.2 | 59.5 / 54.1 | 118.2 / 109.3 | 3.4 / 3.1 | 408.7 / 377.6 | 19.9 / 17.8 | 69.2 / 57.7 | 39.8 / 33.8 | 18.0 / 18.0 | 7.1 / 5.9 | 14.7 / 12.7 |
| CPU, mean | 955.6 | 219.2 | 77.1 | 111.8 | 8.75 | 452.5 | 90.2 | 62.4 | 39.3 | 37.1 | 9.75 | 27.2 |
| GPU, mean | 863.1 | 209.1 | 56.8 | 113.8 | 3.25 | 393.2 | 18.9 | 63.5 | 36.8 | 18.0 | 6.5 | 13.7 |

- **The slope failure** 90.2 to 18.9 s per 1000 Myr, 4.8 times faster: the CPU's worklist grows
  as more of the shelf moves over a history (5.3 s in the first 100 Myr, 90 s over 1000), while a
  device pass costs the same whatever moves.
- **The diffusion solve breaks even** over a whole history (62.4 against 63.5 s; the cycle's lap
  39.3 against 36.8 s): on this APU the residual up and the correction down each CG iteration eat
  the cycle's gain. Its 100 Myr gain (1.1 s) does not grow with the history.
- **This phase takes about 74 s off a 1000 Myr run** (the slope failure 71.4 s, the young stage's
  calls 3.3 s, the solve 1.1 s back); every ported kernel together about 113 s (with the relink's
  20.3 s, the cells' 5.5 s and the amplifier's 13.5 s). The walls fell 72.9 s (K) and 112.0 s (L),
  92.5 s on average, 9.7 %: the rest is in timers neither path runs on the device, in different
  worlds and windows. One of them is a world's, not the device's: the river mouths' fill
  (`marine_fill`) takes 18.0 s in the GPU's world and 37.1 s in the CPU's in both pairs (1.0
  against 1.1 s at 100 Myr).
- **Against the profile.** The coordinator's 273 s (slope failure) and 173 s (solve) per 1000 Myr
  came from Phase 2's loaded pair on the power-saver profile (CPU wall 2,402 s); on the
  performance profile with a quieter laptop the CPU's run is 938 to 973 s and those timers 90 and
  62 s. The ratios carry over, not the seconds. With every ported kernel, L's GPU arm ran a full
  16k 1000 Myr history in 825.9 s on the laptop, under the gate's 900 s, on a machine that is not
  the gate's.
- **Worlds at 1000 Myr** (default seed): CPU CO2 10,011 ppm, `present_co2_factor` 6.855, 22.3 °C,
  glaciated land 0.158, sea level -19.1 m, land 0.215, identical to Phase 2's CPU world on master
  7d60fc4a; GPU 12,600 ppm, 9.728, 22.1 °C, 0.165, -11.3 m, 0.235. Both are master's hothouse
  (Phase 2's GPU world on 7d60fc4a had 7,981 ppm); the CO2 investigation has the figures.

## 11. Natives build per platform

**Measured** on the VM (2026-10-19) with `tools/natives/build-natives.sh` as it stood at the branch
point (release profile; cargo-zigbuild for Linux and macOS, cargo-xwin for Windows): all eight
libraries built and passed the script's own checks, unchanged. No build tool, SDK or licence was
added: wgpu's backends are Rust crates that load their drivers at run time.

| platform | GPU path | backends | linked at build time | loaded at run time |
|---|---|---|---|---|
| linux x86_64, aarch64 (gnu) | on | Vulkan | libc, libm, libpthread, libdl, as before; highest GLIBC symbol still 2.17 | `libvulkan.so.1`, the system's Vulkan loader (wgpu-hal through ash's `Entry::load`) |
| linux x86_64, aarch64 (musl) | on | Vulkan | `libc.so` only, as before | the same |
| windows x86_64, aarch64 | on | DX12, Vulkan | adds `dxgi.dll`, `setupapi.dll`, `oleaut32.dll` and `user32.dll` (all part of Windows) to the imports it had (ntdll, kernel32, bcryptprimitives, `VCRUNTIME140.dll`, the CRT's api sets) | `d3d12.dll`; `dxcompiler.dll` when it is on the PATH, else `d3dcompiler_47.dll` (FXC, part of Windows 10 and 11); `vulkan-1.dll` |
| macos x86_64, aarch64 | **off** | none built | libSystem, libiconv, libcharset, as before; minimum macOS 11.7.1 | nothing |

- **Run time.** A machine without the loader or a driver opens no adapter, and `auto` runs on the
  CPU (section 3); nothing is shipped for it. DX12 and FXC are part of Windows; Vulkan's loader
  comes with the GPU vendors' drivers, and on Linux with the distribution's Mesa or vendor driver.
- **Size** (measured: the same release build of the branch with the feature off in
  `deeptime-jni`, `cargo tree` then listing no wgpu crate): linux-x86_64-gnu 13,778,008 to
  18,456,896 bytes (+4.7 MB, +34 %), windows-x86_64 16,842,240 to 22,758,912 bytes (+5.9 MB,
  +35 %); the other Linux and Windows libraries alike. The jar carries one library per platform
  and CPU level, so it grows by about 5 MB a library before compression.
- **macOS: off, and why.** wgpu's Metal backend (objc2-metal, objc2-quartz-core,
  objc2-foundation) links Apple's frameworks Metal, QuartzCore and Foundation at build time. The
  natives are cross-built from Linux with zig's libSystem stubs and no Apple SDK (the build warns
  that `xcrun` is missing), so the frameworks are not there to link against. **Measured:** the
  same zig build of macos-aarch64 with the feature forced on fails at the link, "unable to find
  dynamic system library 'objc'" (objc2's runtime, the first of the Apple libraries it needs).
  `deeptime-jni`'s `Cargo.toml` turns the feature on for every target except macOS, so the dylibs
  are as before. Turning it on needs one of: building the two macOS libraries on a Mac with Xcode
  (CI's macOS runners have it), or an Apple SDK on the build host, whose licence allows use only on
  Apple hardware. Either is a change to how the natives are built (plan §6.4), so it waits for
  Ben. Apple silicon has a capable integrated GPU, so this is the largest gap the path has.
- **The CLI** (`deeptime`, a developer tool) turns the feature on for every host it builds on.
- **After master's natives change** (`ship` profile with fat LTO, a baseline and an x86-64-v3
  library per x86-64 platform; merged into this branch at 5cf2f9bb): **measured** with the
  merged script (`build-natives.sh --only linux-x86_64-gnu,windows-x86_64`), all four libraries
  built and passed its checks. Sizes with the GPU path: linux-x86_64-gnu 16,445,360 (baseline)
  and 16,443,200 bytes (x86-64-v3), windows-x86_64 21,533,696 and 21,553,664 bytes, smaller than
  the release build's above; the ship profile without the GPU path was not built, so wgpu's share
  of these is not measured. The Linux libraries still need GLIBC 2.17 at most; the Windows
  ones still import `dxgi.dll`, `setupapi.dll`, `user32.dll` and `oleaut32.dll` beside the old
  set. The CPU probe crate (`deeptime-cpuprobe`) does not depend on the GPU feature.

## 12. The next kernels, in order, once the CPU branches land

Each waits for the CPU branch now editing its code (the CPU result is the reference, so the
kernel ports the merged algorithm, not today's), and each gets one dispatch point and a tolerance
test as in section 2.

**Where the branches stand** (coordinator, 2026-10-19; master 7d60fc4a, merged into this
branch): the climate, plates parts 1 and 2, the after-group work, the plate cadence (master
fb38204f: the shipped presets take a climate snapshot every 20 Myr instead of 10) and the surface
solvers (master 7d60fc4a: diffusion is a conjugate gradient preconditioned by an f32 multigrid
V-cycle, about 10 iterations a step against 63; the flood is an order-independent parallel
priority flood after Barnes 2016) are merged, so rows 2 to 4, 7 and the flood are free; so is
the young stage's memory work (master 26ce664e, also merged into this branch). The surface
kernels other than diffusion and the flood (slope failure, advection, flexure, residency) wait
for the coordinator. (Phase 3: the coordinator gave this branch the slope failure and its early
stop; both it and the diffusion cycle are ported, items 2 and 3.)

**The carbon cycle amplifies the last bit.** The climate branch found that changing only the
order of two row-subtotal sums (`land_terms` and `snapshot::ice_cover`) moved a 100 Myr world's
CO2 from 551 to 570 ppm, its mean temperature from 14.9 to 15.0 °C and its sea level from 32 to
30 m (build log 2026-10-19). So any climate kernel keeps the sums that feed weathering and the ice
cover in f64 on the CPU, or compensated (Kahan) in a fixed order on the device, and its CPU against
GPU comparison runs the whole history, not from a checkpoint after the climate, and reports CO2,
temperature and sea level beside the hypsometry. (The amplifier's comparison in section 10 starts
after the climate, so its CO2 is the checkpoint's in both arms: 551.208 ppm.)

1. **Resident surface fields first (infrastructure).** A per-session handle that keeps the L1
   height, sediment, erodibility and sea mask on the device across the surface substeps of one
   plate step, with one upload after advection and one download before the bake's readers. Without
   it, rows 1, 2 and 5 each pay an upload and a download per call, which on PCIe is most of their
   gain. The memory estimator then has to count these buffers (section 4's last point).
2. **`surface:slope_failure`** (row 1): **ported** (Phase 3, section 16): a full-grid Jacobi
   gather per pass instead of the sparse worklist, every pass. The early stop the profile
   proposed (20 to 50 s on the CPU, approved) was not taken: P1-19 found no tolerance stops a
   call early, and on the device it would cost a read-back a pass. Measured on the laptop
   (section 10): 5.3 s to 1.7 s per 100 Myr in the surface stage, 3.1 times faster, and the
   young stage's calls too; over 1000 Myr 90.2 to 18.9 s, 4.8 times (two pairs).
3. **`surface:diffusion_solve`** (row 2): **the V-cycle is ported** (Phase 3, section 15); the
   CG's f64 vectors and dot products stay on the CPU with the residual up and the correction
   down each iteration (5.4 MB each way on the full 16k's L1). Measured on the laptop (section
   10): the solve 6.4 to 5.3 s per 100 Myr (the cycle's own lap 4.6 to 3.6 s), 1.2 times faster:
   the cycle is a memory-bound stencil and the integrated GPU shares the CPU's memory. Over 1000
   Myr it breaks even (62.4 against 63.5 s, two pairs): the per-iteration transfers eat the
   cycle's gain on this APU. A discrete
   GPU would do more for the cycle and pay PCIe for the 5.4 MB each way; moving the CG too (with
   compensated sums in a fixed order) would remove the transfers but not the bandwidth limit.
   **The flood** (free since the same merge; 72 s per 1000 Myr on the CPU arm, its first call
   each step beside the advection's carry, so the wall gains less): a priority flood is serial by
   nature, and a device flood would be another algorithm (iterated minimum propagation) to prove
   equal; it ranks below the relink's second-group search (61.7 s of CPU cells per 1000 Myr,
   section 13) and below advection (row 5).
4. **`plates:write_l1`** (row 3): **the relink is ported** (2026-10-19, section 13); the layers
   stay on the CPU, since the split measured first gave the relink 5.0 s and the layer pass
   1.8 s of the CPU's 8.3 s per 100 Myr, and moving the layers would download 160 MB a step.
5. **`surface:advect` and `climate:topography`** (rows 5 and 9) together: one semi-Lagrangian
   gather kernel for the continuous fields; the strata log stays on the CPU. 23 to 36 s.
6. **`climate snapshot:cells`** (row 4): **ported** (2026-10-19, section 14): the per-cell months
   on the device, the row march and the sums that feed the carbon cycle on the CPU in f64.
   Measured on the laptop (section 10): 4.6 s of cells per 100 Myr became 1.1 s with the
   transfers at a snapshot every 10 Myr; at master fb38204f's 20 Myr both halve, so the port saves
   about 1.8 s per 100 Myr, some 18 s per 1000 Myr. `snapshot:ebm2d` (row 11) stays on the CPU: 864 steps of about seven dependent phases a
   snapshot are about 6,000 global barriers, its line solves have 41 or 128 lines each, and f32
   would lose its exact f64 conservation while f64 runs at a sixteenth to a thirty-second of the
   rate on consumer GPUs; at best 15 to 25 s per 1000 Myr for a rewrite of about 1,900 lines.
   The deposits stage (row 7) is free too and larger, but needs its rule interpreter in WGSL.
7. **`surface:flexure`** (row 6): needs an FFT in WGSL for sizes that are not powers of two
   (Bluestein, or padding to the next power of two with the preconditioner adjusted). 20 to 28 s.
8. **The amplifier's mountain character** (7.6 s, no branch in the way): a per-cell noise sum on
   the window already on the device. Small, but it is the rest of the amplify stage's grid work
   and the window is already resident. 5 to 6 s.

## 13. The plates' L1 relink on the GPU (what was ported)

- **Where the time was (measured first).** The compute gauge's laps on a full 16k, 8-thread,
  100 Myr CPU bench on the VM (`deeptime bench --preset earthlike_full_16k --start
  scattered_continents --length-myr 100 --threads 8 --compute cpu`, 2026-10-19, loaded VM):
  `write_l1` 8.3 s, of it the relink 5.0 s, the 30-layer pass 1.8 s and the visible samples'
  k-d tree 0.9 s. So the relink moved and the layers did not: they are a fifth of the kernel, and
  on the device they would cost a 160 MB download a step (30 layers of 1.34 M cells) plus about
  30 per-sample fields up, against 36 ms a step of CPU work on the VM.
- **Dispatch point (read, `plates/l1.rs`).** `sample` asks `compute::ambient_device` for the
  session's device (the plates get no `StageContext`; the session's pool carries the mode on each
  worker, section 2), and `sample_with` runs the device's relink beside the k-d tree's build
  (`rayon::join`). In `Coupling::relink`'s pass a cell the device settled takes its link and is
  COVERED; every other cell runs today's CPU code. `plates/mod.rs` is untouched.
- **Algorithm (read, `plates/l1/gpu.rs`).** Per call, on the CPU: the visible samples
  counting-sorted into latitude bands one spacing tall, each band cut into as many longitude
  buckets as its circumference holds spacings; the groups' inverse rotations; each sample's
  triangle and group (with a hidden bit). Per resampling: the lattice's points, triangles and
  their neighbours. One invocation per L1 cell: its unit vector from the coupling's own row and
  column sines and cosines; the nearest visible sample within 1.2 spacings over the buckets the
  cap touches (ties to the lower index); the cell rotated into that sample's group frame; a walk
  of at most 48 steps from the sample's triangle, each edge's orientation computed from the lower
  vertex so two neighbours never disagree; the test that the three corners are that group's and
  visible; the barycentric weights, and the nearest corner as the CPU picks it (the first largest
  weight). A cell that finds no sample within the cap, fails the test or does not arrive comes
  back marked for the CPU. One dispatch over the grid in bands of rows (section 5).
- **Residency.** The pipeline and the buffers stay in a session-scoped slot
  (`Ambient::kept`), so a server holds no device memory for a world after its session; the mesh
  is uploaded again only when the lattice changes. Sizes: section 4.
- **Precision and error budget.** f32 on unit vectors resolves about 1e-7 rad, so a link differs
  from the CPU's only at a tie (two samples equally near, or a cell on an edge); the weights
  differ by f32 rounding. Budget, checked by
  `plates::l1::gpu::tests::the_gpu_relink_matches_the_cpu_but_at_ties` on a 6,000-sample planet
  stepped 60 Myr from a supercontinent, six samplings: fewer than a fifth of the cells left to the
  CPU, fewer than one link in 1,000 different, weights within 1e-4, fewer than one layer value
  in 1,000 off by more than 1e-4 of its layer's largest (NaN equal to NaN). Measured on lavapipe:
  125,952 cells, 11.6 % left to the CPU, no link different, weights within 1.6e-5, no value off.
  On the full 16k (a temporary switch running both relinks on every sampling of a 100 Myr
  history, lavapipe; section 10): 0.015 % of links on another triangle, weights within about
  7.7e-4 where the triangle agrees, and every layer's signed difference as often up as down.
- **What stays on the CPU.** The k-d tree (for the CPU's cells), the cells the device leaves
  (5.3 % on the full 16k's first 100 Myr, 10.0 % over 1000 Myr: boundary strips, gaps, cells
  another group covers), the six- and
  32-nearest searches, the gap crust and all 30 layers. Porting the second-group search would
  take part of what the CPU's cells still cost: 2.5 s in the first 100 Myr on the quiet laptop,
  61.7 s over 1000 Myr on the loaded one, where the share left to the CPU grows to 10 %.

## 14. The climate snapshot's cells on the GPU (what was ported)

Ported by a sub-agent (branch `worktree-agent-a0a9401013662f19b`, merged into this branch); the
science page is `docs/science/per-cell-climate.md`, "Down to L1 on a GPU".

- **Dispatch point (read, `climate/snapshot.rs`).** `snapshot::CellsDevice` (`name`,
  `host_bytes`, `cells`), called from `compute_on(.., dev)`; `compute` keeps its signature and
  passes none. The stage finds the device once, at its first snapshot, through
  `compute::device(ctx.compute, ctx.log)`, and drops it after an error (a warning; the rest of
  the run's snapshots run on the CPU). The counts are kept in the stage's checkpoint state, so a
  resumed run's note counts every snapshot.
- **On the device (`climate/gpu.rs`).** Step 5 of a snapshot, one invocation per L1 cell with
  its twelve months: the belt rain and the mountain rain, the masked bilinear of the coarse
  tiles, the coast blend, the winter inversion, sea ice and its sharpened edge, aridity, the
  equilibrium line, Köppen and the glacier flag. In chunks of 2^18 cells (128 rows; six on the
  full 16k's L1), equal bit for bit to one dispatch.
- **On the CPU.** Everything the cells read (the zonal and 2-D models, the coarse tiles, the
  distances to the sea and the basins, the sea's departure from its row, the air's march along
  each row, the mountain rain's regimes, each row's and column's factors in f64, uploaded) and
  everything that reads them: the anomalies, `ice_cover` and `land_terms`, whose sums feed the
  carbon cycle and stay f64 in their present order (section 12). The banded climate, the
  moisture budget and the upwind rain stay on the CPU entirely.
- **Sizes (read, full 16k).** About 32.5 MiB on the device before the months (44.5 MiB with the
  twelve temperatures every snapshot keeps since the runoff law's re-reviews, 56.5 MiB on the last
  snapshot, which keeps their rain too); 113 MB up and 54 MB down a call (about 64.5 MB more at
  every snapshot for the temperatures, 129 MB more on the last). 103 calls per
  1000 Myr at a snapshot every 10 Myr, about 53 at the shipped presets' 20 Myr (master
  fb38204f). The buffers are optional like the relink's (section 4).
- **Precision and error budget.** `climate::tests::the_gpu_cells_stay_within_their_error_budget_of_the_cpu`
  on the terrain of a 20 Myr scattered-continents run (L1 256 x 82, 5,990 land cells) under
  today's climate and a cold one: budgets 0.01 K, 1e-4 relative on rain, aridity and the
  equilibrium line, 1e-4 on sea ice, classes equal but at thresholds. Measured on lavapipe:
  temperatures within 7.6e-6 K, rain 1.2e-6 relative, aridity 7.1e-7, the equilibrium line
  1.5 mm, sea ice 2.4e-7, no Köppen class or glacier flag different (cells within rounding of a
  model threshold are left out of the comparison: 5 for the rain regimes, 110 to 390 for
  Thornthwaite's steps).
  On the full 16k (a temporary switch running both on every snapshot of a 100 Myr history,
  lavapipe; section 10): temperature means within 1.3e-7 K, rain within 1.9e-7 relative, the
  classes, flags, sea ice and ice sheets identical, no field leaning either way.

## 15. The diffusion solve's V-cycle on the GPU (what was ported)

Phase 3 (2026-10-19, branch `gpu-phase3` from master 3e7edd49).

- **Dispatch point (read, `surface/diffusion/multigrid.rs`).** `cg` asks the session's ambient
  context for a device (`session_device`: `compute::ambient_device("surface:diffusion")`, and in
  `auto` only for a grid of at least 2^17 cells, `DEVICE_MIN_CELLS`; a forced `gpu` takes any
  size) and runs `cg_on(.., device)`. The preconditioner is an enum `Pre`: the CPU's levels
  (`Hierarchy`) or the device's (`gpu::DeviceCycle`). A device that fails while it is built sends
  the solve to the CPU's cycle; one that fails in an iteration hands the rest of the solve to a
  CPU cycle built then from the same operator and residual, so the solve ends either way. A
  warning once a session, `mg_fallback_n` in the compute gauge.
- **On the device (`multigrid/gpu.rs`).** Per solve: the finest operator (east, south and
  diagonal couplings, converted from the CPU's f64) uploaded once, the coarse levels built from
  it on the device (`COARSEN`: half the summed fine couplings of each 2 x 2 aggregate and its
  mass; `CDIAG`: the coarse diagonal and ω/diagonal), all in one submission. Per conjugate-gradient
  iteration: the residual up (f32, 4 bytes a cell), one V-cycle recorded as the CPU's `vcycle`
  runs it, sweep for sweep (`FIRST` from z = 0, `SWEEP` with or without the coarse correction,
  `RESTRICT` of the fine residual), in one submission, and the finest correction down (4 bytes a
  cell). The two z buffers of a level alternate between sweeps, so nothing is copied.
- **On the CPU.** The conjugate gradients as they were: the f64 vectors, the operator's products,
  the step lengths, the stopping rule and the mass put-back. The dot product the iteration needs
  from the cycle (Σ r·z) is taken on the CPU from the downloaded z, per strip of 8 rows in f64 and
  added in order, as the CPU's cycle takes it, so the solve stays the same at any thread count.
- **Residency.** The kernels, every level's buffers and their bind groups stay in a
  session-scoped slot (`Ambient::kept`) while the grid keeps its size: 32 bytes a cell and level
  (28 on the finest), 49.5 MiB on the full 16k's L1 with its eight coarse levels, and a 5.1 MiB
  read-back buffer. Each solve uploads its operator (16 MB on the full 16k), each iteration
  5.4 MB each way.
- **Precision and error budget.** The cycle was f32 on the CPU already, and each kernel does a
  cell's arithmetic in the CPU cycle's order; the device differs where it rounds otherwise (a
  fused multiply-add, WGSL's 2.5-ulp division). A preconditioner need not be exact: the conjugate
  gradients stop on the same f64 residual and put the same mass back either way. Checked by
  `surface::diffusion::multigrid::tests::the_gpu_cycle_matches_the_cpu_cycle` (seven grid shapes,
  wrapping and not, down to one row; three cycles each: z within 1e-4 of its largest value, Σ r·z
  within 1e-4 relative) and `the_gpu_cycle_solve_meets_its_tolerance_and_keeps_the_mass` (five
  shapes, three settings of Δt and tolerance: the error within the tolerance's bound, the mass to
  1e-12, within twice the bound of the CPU-cycle solve, at most two iterations more).
  **Measured** on lavapipe (the gates' run, 2026-10-19): the device's z and Σ r·z equal the CPU
  cycle's to the bit in all 21 cycles (seven shapes, three rounds), and all 30 solves take the
  CPU-cycle solve's iterations and end on its heights to the bit, their error at most 0.055 of the
  tolerance's bound. Lavapipe rounds as the CPU does here; a hardware driver may not, which the
  budgets allow for.

## 16. The submarine slope failure on the GPU (what was ported)

Phase 3 (2026-10-19). The science is ERO-168 to ERO-170 (`docs/science/erosion-and-sediment.md`);
neither the law, the rate (5 % of a pair's excess a pass) nor the 40 passes a step changed.

- **The early stop was not taken.** The coordinator's profile item "stop slope failure early"
  (20 to 50 s, world-changing, approved) was given to this phase so the CPU and GPU paths would
  not fight over the same code, to be done first only if it simplified the port. It does not:
  P1-19's trace of every call (`docs/science/erosion-and-sediment.md`, "the slope failure's
  stopping") found that no tolerance from 0.005 to 0.5 critical drops stops any call before its
  40 passes (about 200,000 cells moved in every pass), and the per-pair form that did cut the
  movers ran no faster and moved the planet outside master's ensemble. On the device the
  question goes away: a pass is one dispatch over the grid whatever moves, and a stop would need
  a read-back and a sum every pass.
- **Dispatch point (read, `surface/marine.rs`).** `slope_failure` asks the session's ambient
  context for a device (`gpu::session_device`: `compute::ambient_device("surface:slope_failure")`,
  and in `auto` only for at least 2^17 cells, `DEVICE_MIN_CELLS`); on success the call returns
  the device's metres moved, on an error the CPU path runs the call from the same inputs (the
  device writes nothing back until it has read everything). A warning once a session,
  `slope_failure_fallback_n` in the compute gauge.
- **On the device (`marine/gpu.rs`).** Per call: each cell's height above the sea level
  (`base`, f32), the sea's bits and the sediment up; the change since the call began (`acc`) and
  the metres moved (`mv`) cleared. Per pass, two dispatches: `SHARE` (for each cell under the sea
  with sediment, its total give over its lower neighbours under the sea beyond the critical drop,
  side or diagonal, and the share of it its sediment allows, min(avail / give, 1)) and `CHANGE` (each cell's net change, what its
  higher neighbours give it at their share less what it gives at its own, the new change and
  sediment, |Δh| / 2 to the moved metres). Heights are `base + acc`, so a slope of a few metres
  on a floor 4 km deep is not rounded to the floor's f32 step (0.24 mm at 2 to 4 km) every pass.
  The 40 passes go in one submission, ping-ponging `acc` and `avail` between two buffers; one
  read-back of the change, the sediment and the moved metres at the end. The host adds the change
  to its f64 heights and sums the moved metres in f64 in fixed blocks of 65,536 cells, in order.
- **Every pass runs.** The CPU path keeps an active set (the movers and their neighbours) and
  stops when a pass moves nothing; the device runs every pass over every cell. The results are
  the same: a cell outside the active set computes no change, and a pass that moves nothing
  leaves every input as it was.
- **Residency.** Seven f32 buffers and the sea's bits, 28 bytes a cell, and a 12-byte read-back:
  51.4 MiB on the full 16k's L1, kept for the session while the grid keeps its size. Per call
  about 10.9 MB up and 16.1 MB down. The young stage calls the same function on its own grid
  (4,096 x 1,312 on the full 16k), so its five calls per 100 Myr run on the device too and the
  buffers are rebuilt at its size and back (section 4).
- **Precision and error budget.** Each cell's share and change are the CPU's expressions in the
  CPU's order, in f32 instead of f64. A pair's two ends compute its flux from the same f32 values
  but in different sums, so matter is kept to the f32 round-off of the changes, not the CPU's f64
  round-off. Checked by `surface::marine::tests::the_gpu_slope_failure_matches_the_cpu` on the
  bit-for-bit test's worlds (dense sediment, a few mounds, a grid that does not wrap, one and two
  cells wide, one row; seeds 1 to 3; 40 and 400 passes): heights and sediment within 1 cm of the
  CPU's on floors 3 km deep, the moved metres within 1e-4 relative, matter within 1e-5 of the
  moved metres.
  **Measured** on lavapipe (the gates' run, 21 worlds at each pass count): at 40 passes heights
  within 4.2e-4 m, sediment within 1.5e-3 m, the moved metres within 4.1e-8 relative, matter
  within 4.4e-3 m-cells; at 400 passes 3.6e-3 m, 9.3e-3 m, 3.0e-7 and 6.1e-2 m-cells. The grids
  one and two cells wide move nothing on either path.
