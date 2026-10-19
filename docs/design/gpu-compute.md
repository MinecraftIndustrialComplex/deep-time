# The GPU compute path: design

Status: **infrastructure and the first kernel built and measured** (2026-10-18 to 2026-10-19;
coordinator's task after Ben's decision of 2026-10-18, decisions log: "I am not worried about
determinism, performance is far more important, if people want to share a world they can just
share a zip of the finished render or some shit."). Branch `worktree-agent-a1265f3344b19b0c1`
from master 584b1241, masters 5cf2f9bb, e2166684 and c767da2c merged in. The aim is the P1-19
exit item 1 gate (full 16k, 8 threads, 1000 Myr, 900 s), which the 2026-10-18 profile put at
1,242.8 s (`docs/design/p1-17-benchmark-gate.md`, "where the full 16k run's time goes";
`review/pages/sim-time/data.js`). Phase 2 (2026-10-19, branch `gpu-phase2` from master
b337fa73): the plates' L1 relink and the climate snapshot's cells ported (sections 13 and 14),
the whole-history comparison and the laptop's benches in section 10.

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
  fallback and the reference. On the laptop's Radeon the relink takes `write_l1` from 14.9 to
  9.2 s per 100 Myr. A GPU world differs from the CPU's about as much as a CPU world with one
  weight an ulp off (section 10). WHOLE_RUN_SUMMARY
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
| compute mode | `crates/deeptime-core/src/compute.rs` | `ComputeMode` (auto, cpu, gpu), `DEEPTIME_COMPUTE`, `device(mode, log)` |
| session switch | `Session::set_compute`, `StageContext::compute` | the mode a stage reads |
| dispatch point | `surface/amplify/kernels.rs`: `PassDevice`, `run_passes_on` | the CPU path is `middle_cpu`; a device implements `middle` |
| tile loop | `surface/amplify/tiles.rs` | reserves the device's host memory, runs a tile on the device, reruns it on the CPU when the device fails |
| the kernels | `surface/amplify/gpu.rs` | WGSL next to the CPU code it mirrors; `AmplifyGpu` |
| ambient context | `compute.rs`: `Ambient`, `enter`, `ambient`, `ambient_device`, `Ambient::kept` | the session's mode and log on every worker of its pool, for kernels deep inside a stage that get no `StageContext` (the plates' L1 sampling); `kept` holds a kernel's pipeline and buffers until the session ends |
| dispatch point (L1) | `plates/l1.rs`: `sample` (the device from `ambient_device`), `on_device`, `sample_with` | the device links the covered cells, the CPU every other cell and all the layers (section 13) |
| the relink kernel | `plates/l1/gpu.rs`: `relink`, `Resident` | WGSL beside the CPU code; tolerance test in `plates/l1/gpu/tests.rs` |
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
b337fa73, the climate sub-agent's branch merged in).

*Correctness on lavapipe*: sections 13 and 14.

*Whole history, CPU against GPU.* The carbon cycle carries a last-bit difference into CO2, so the
comparison runs a whole history from scratch (section 12): `deeptime stage volcanoes --preset
earthlike_full_16k --start scattered_continents --length-myr 100 --threads 8 --set seed=1
--compute cpu|gpu` on the VM (lavapipe held to 8 threads), then `tools/review/p1-17-hyps.py`
and `p1-17-cmp.py`. To tell a GPU's difference from the model's own sensitivity, the same planet
also ran on the CPU with one f32 ulp flipped in the first weight of every covered cell (a
temporary switch, not committed). The laptop's rows are the A/B benches below (their land
figures from the bench's `surface_final` line, which can differ from the stage's hypsometry in
the last digit).

| world | CO2 ppm | T °C | glaciated land | sea level m | land | land z50 m | land z90 m | land over 2 km |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| CPU (VM; the merged branch's CPU path gives the same to the digit) | 735 | 15.5 | 0.062 | 36.9 | 0.239 | 463 | 1,785 | 0.088 |
| CPU, one weight an ulp off (VM) | 653 | 14.5 | 0.083 | 31.4 | 0.263 | 466 | 1,866 | 0.093 |
| GPU relink, lavapipe (VM) | 619 | 14.8 | 0.058 | 37.5 | 0.257 | 457 | 1,487 | 0.070 |
| GPU relink and cells, lavapipe (VM) | 597 | 14.7 | 0.054 | 38.3 | 0.259 | 453 | 1,438 | 0.068 |
| GPU relink, Radeon 8050S (laptop, three runs alike) | 641 | 14.5 | 0.082 | 31.2 | 0.264 | 465 | 1,855 | 0.096 |
RADEON_ALL_ROW

- A one-ulp change on the CPU moves this planet as far as either GPU does: CO2 by 82 ppm, the
  mean temperature by 1.0 °C, the glaciated share by a third, the sea level by 5.5 m. So the
  differences are the model's sensitivity to its last bit, not an error of the kernels, and the
  GPU worlds sit inside the spread of the CPU's own. The land's 90th percentile is lower in
  both lavapipe worlds and higher in the Radeon's and the jittered CPU's: no sign of a bias.
- `p1-17-cmp.py`: the keyframes differ from the first (kf-0), as expected once the first
  sampling's weights differ in the last bits.
- Each world is reproducible on its own device: the three Radeon runs of a set ended on the same
  figures.

*Speed: the laptop's Radeon 8050S*, as in Phase 1 (binary built on the VM, `systemd-run --user
--scope -p CPUQuota=800% nice -n 19 deeptime bench --preset earthlike_full_16k --start
scattered_continents --length-myr 100 --threads 8 --compute auto|cpu`, three pairs alternating
which goes first). Seconds per 100 Myr, mean of three; `auto` also runs the amplifier on the
GPU (section 10 above).

| set | arm | wall | plates stage | write_l1 | relink (CPU part) | device | tree (beside the device) | layers | amplify |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| relink | CPU | 326.6 | 34.0 | 14.9 | 8.8 | | 1.4 | 3.8 | 52.7 |
| | GPU | 303.9 | 28.6 | 9.2 | 2.4 | 1.0 | 1.7 | 3.8 | 28.1 |
CELLS_ROWS

- The relink: 8.8 s on the CPU becomes 1.0 s on the device beside the k-d tree's build and
  2.4 s of CPU work for the 5.3 % of cells the device leaves (3,566,358 of 67,174,400 over the
  run); `write_l1` 14.9 to 9.2 s, 1.6 times faster, 5.7 s per 100 Myr.
- The wall times are of different worlds once the GPU is on (the table above), so they mix the
  kernels' gain with the world's own cost; the kernels' laps are the cleaner figure.

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

**Where the branches stand** (coordinator, 2026-10-19; master c767da2c, merged into this
branch): the climate (snapshot cells, ice, the 2-D model in larger pieces, the clock), plates
parts 1 and 2 (transition, collapse, collision, record, `dijkstra_sparse`; feedback, belts,
fronts, `write_l1`, detect, `stats_margins`) and the after-group work (bake, horizons, keyframes,
deposits, volcanoes, the allocator) are merged, so rows 3, 4 and 7 are free. Still being edited:
the surface (flood, diffusion and slope-failure memory, advection, fields and events, glacial,
flexure), the surface solvers (multigrid, slope failure, the flood) and the plate cadence (which
may touch `plates/mod.rs` and the step constants, so a `write_l1` port keeps to one dispatch
point inside `plates/l1.rs`).

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
2. **`surface:slope_failure`** (row 1), after the world-changing surface branch (its early stop
   changes the iteration count): a full-grid Jacobi gather per iteration instead of the sparse
   worklist. 70 to 78 s.
3. **`surface:diffusion_solve`** (row 2), after the multigrid branch: if multigrid lands, a GPU
   V-cycle (restriction, smoothing and prolongation are stencils), else the PCG with defect
   correction against an f64 residual. 70 to 90 s.
4. **`plates:write_l1`** (row 3): **the relink is ported** (2026-10-19, section 13); the layers
   stay on the CPU, since the split measured first gave the relink 5.0 s and the layer pass
   1.8 s of the CPU's 8.3 s per 100 Myr, and moving the layers would download 160 MB a step.
5. **`surface:advect` and `climate:topography`** (rows 5 and 9) together: one semi-Lagrangian
   gather kernel for the continuous fields; the strata log stays on the CPU. 23 to 36 s.
6. **`climate snapshot:cells`** (row 4): **ported** (2026-10-19, section 14): the per-cell months
   on the device, the row march and the sums that feed the carbon cycle on the CPU in f64.
   `snapshot:ebm2d` (row 11) stays on the CPU: 864 steps of about seven dependent phases a
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
- **What stays on the CPU.** The k-d tree (for the CPU's cells), the cells the device leaves
  (5.3 % on the full 16k: boundary strips, gaps, cells another group covers), the six- and
  32-nearest searches, the gap crust and all 30 layers. Porting the second-group search would
  take part of the 2.4 s (per 100 Myr on the laptop) the CPU's cells still cost.

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
- **Sizes (read, full 16k).** About 32.5 MiB on the device (56.5 MiB on the last snapshot, which
  keeps the months); 113 MB up and 54 MB down a call (129 MB more on the last). 103 calls per
  1000 Myr. The buffers are optional like the relink's (section 4).
- **Precision and error budget.** `climate::tests::the_gpu_cells_stay_within_their_error_budget_of_the_cpu`
  on the terrain of a 20 Myr scattered-continents run (L1 256 x 82, 5,990 land cells) under
  today's climate and a cold one: budgets 0.01 K, 1e-4 relative on rain, aridity and the
  equilibrium line, 1e-4 on sea ice, classes equal but at thresholds. Measured on lavapipe:
  temperatures within 7.6e-6 K, rain 1.2e-6 relative, aridity 7.1e-7, the equilibrium line
  1.5 mm, sea ice 2.4e-7, no Köppen class or glacier flag different (cells within rounding of a
  model threshold are left out of the comparison: 5 for the rain regimes, 110 to 390 for
  Thornthwaite's steps).
