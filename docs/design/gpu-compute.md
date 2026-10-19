# The GPU compute path: design

Status: **infrastructure and the first kernel built and measured** (2026-10-18 to 2026-10-19;
coordinator's task after Ben's decision of 2026-10-18, decisions log: "I am not worried about
determinism, performance is far more important, if people want to share a world they can just
share a zip of the finished render or some shit."). Branch `worktree-agent-a1265f3344b19b0c1` from master 584b1241. The aim is the
P1-19 exit item 1 gate (full 16k, 8 threads, 1000 Myr, 900 s), which the 2026-10-18 profile put at
1,242.8 s (`docs/design/p1-17-benchmark-gate.md`, "where the full 16k run's time goes";
`review/pages/sim-time/data.js`).

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
- **Headline estimate (section 9).** About 513 s of the run's 1,242.8 s wall is grid work on the
  critical path that a compute shader can do. On a mid-range GPU (an NVIDIA T4 or the Radeon 8060S
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
  host copies). On an integrated GPU (the 8060S, Apple silicon, lavapipe) the device's buffers are
  system memory, so the tile reserves `56 + 8` bytes a cell more against the session's native
  memory cap, through the same `reserve` that spills history frames; a discrete GPU reserves the
  8. The stream decision counts that share (section 3), so a tile's reserve fails only when the
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
| 3 | `plates:write_l1` | 95.8 / 711.3 | per L1 cell: k-d tree 6-nearest over visible samples, a triangle walk, barycentric interpolation of about 14 fields | samples (250 k x about 30 fields) up; 14 L1 fields down | f32 angles with per-plate offsets; budget: the CPU's nearest-sample choice except at ties | 50 to 75 | a spatial hash replaces kiddo and the mesh walk (different tie-breaks); plates branches in flight |
| 4 | `climate snapshot:cells` | 46.1 / 334.7 | per row an air march (a recurrence with floors), then per cell 12 months of T and P, Köppen, glaciers | about 15 fields up; f32 outputs down; 103 calls | per-cell months in f32 (temperatures to 0.01 K); the march stays a row scan | 20 to 30 | the march is serial along a row (656 rows); climate branch in flight |
| 5 | `surface:advect` | 40.2 / 253.1 | semi-Lagrangian gather, per-plate rotation, bilinear fields, nearest for the 516 MB strata log | fields resident; the log only if its readers move too | rotation in f32 with a per-plate offset; budget one hundredth of a cell in position | 15 to 25 | the log's readers (fields, structure, bake) are on the CPU, so only the continuous fields move |
| 6 | `surface:flexure` | 38.2 / 275.3 | PCG on the thin plate, FFT-spectral preconditioner, to 2 m or 60 iterations | loads, D, C up; deflection resident | f32 plausible: the stop is 2 m | 20 to 28 | an FFT in WGSL (no vendor FFT in wgpu), sizes not powers of two |
| 7 | `deposits` (stage) | 82.0 / 319.0 | per frame: LZ4 decode, octagon window sums, a rule interpreter, TTI; a serial tail | accumulators resident; frames up (160 MB each) | i64 fixed point window sums kept exact in u32 pairs; rules in f32 | 15 to 30 | the rule interpreter has to become WGSL; decode and tail stay CPU |
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
- **Changed:** `tests/memory_estimate.rs` runs on the CPU (`Session::set_compute`), because the
  driver's memory sits outside the count it checks (section 4).
- **Worlds.** A 100 Myr planet on the CPU and with the GPU forced on lavapipe, compared with
  `tools/review/p1-17-cmp.py` and `p1-17-hyps.py`: section 10.
- **Speed** only on real GPUs (the laptop's 8060S, Ben's rule of 2026-10-18), GPU against CPU in
  A/B pairs on the same machine and binary.

## 9. Amdahl on a mid-range GPU (estimated)

- **Bandwidth.** The heavy kernels already keep 7 to 7.6 of 8 cores busy and are memory-bound
  stencils. Eight VM cores stream perhaps 60 to 100 GB/s; a T4 has 320 GB/s (and PCIe 3 at about
  12 GB/s for transfers), the 8060S 256 GB/s shared with the CPU (no PCIe). So a resident stencil
  runs 3 to 6 times faster; one whose fields cross every step, less. **Measured** check (section
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
(mean change 37.6 m, eroded 2.458e4 km³, glacial carved 1.203e4 km³). Cell by cell (a scratch
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

## 11. Natives build per platform

(pending: the cross-build checks)

## 12. The next kernels, in order, once the CPU branches land

Each waits for the CPU branch now editing its code (the CPU result is the reference, so the
kernel ports the merged algorithm, not today's), and each gets one dispatch point and a tolerance
test as in section 2.

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
4. **`plates:write_l1`** (row 3), after the plates branches: a spatial hash on the sphere for the
   6 nearest samples, then the barycentric gather. Tie-breaks differ from the k-d tree's, which
   Ben's decision allows. 50 to 75 s.
5. **`surface:advect` and `climate:topography`** (rows 5 and 9) together: one semi-Lagrangian
   gather kernel for the continuous fields; the strata log stays on the CPU. 23 to 36 s.
6. **`climate snapshot:cells`** (row 4), after the climate branch: the per-cell months on the
   device, the row march on the CPU. 20 to 30 s.
7. **`surface:flexure`** (row 6): needs an FFT in WGSL for sizes that are not powers of two
   (Bluestein, or padding to the next power of two with the preconditioner adjusted). 20 to 28 s.
8. **The amplifier's mountain character** (7.6 s, no branch in the way): a per-cell noise sum on
   the window already on the device. Small, but it is the rest of the amplify stage's grid work
   and the window is already resident. 5 to 6 s.
