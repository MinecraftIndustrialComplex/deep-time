# The P1-17 benchmark gate (V1): measurements and the per-stage report

Status: **the gate is missed on both presets** (2026-10-05). This page is the per-stage report the
plan asks for when a preset misses (§9.1: "Missing it here is reported with the per-stage breakdown
and the measured scaling across thread counts (1, 2, 4, 8), and the decision is Ben's"). Every
speed-up on the branch is bit-identical: the worlds before and after are the same files byte for
byte. Ben's answer to P1-24 (2026-10-04, "Optimization only") makes optimisation the only route to
a faster Create World, so the second target, a player's 6-core PC, is measured here too.

Statements are marked **measured** (a run made for this page), **read** (from the code) or
**inferred**.

## Summary

- **The 8-thread gate (plan P1-17: quick ≤ 300 s, full ≤ 900 s, 1000 Myr)** — measured,
  test VM, 8 threads, before = master 449a3ae2, after = this branch, run side by side:
  - quick 16k: scattered continents ‹QSC›, supercontinent 945.8 → 753.5 s (1.26×). **Missed by
    about 2.3×.**
  - full 16k: scattered continents 2,497 → 1,955 s (1.28×), supercontinent 2,704 → 2,176 s (1.24×).
    **Missed by about 2.2×.**
- **The player's PC (P1-24's mid6 profile: 6 cores / 12 CPUs, 11 threads, 6 GiB)** — measured:
  ‹PP›
- **Bit-identical.** Every world file of every after-run equals the before-run's (manifest
  compared without its wall times and the memory accounting's notes). A new test runs the coupled
  pipeline on 1 and 3 threads and compares every field and the whole world body bit for bit.
- **Why the gate cannot be met by these passes alone** (measured, inferred): one thread does the
  quick preset in 1,969 s of wall time and 1,906 s of CPU, so 300 s at 8 threads needs a parallel
  efficiency of about 80 %; today's is 35 % (2.84× at 8 threads, 1.10× from 4 to 8). The full
  preset uses 7,140 s of CPU at 2 threads, so even perfect scaling would give about 890 s at 8.
  About 40 % of the wall time at 8 threads is code that runs on one thread (the driver thread's
  samples exceed every worker's by that much); the largest pieces are listed below.
- **Memory** (measured): the run's own gauge peaks at 3.15–3.16 GiB (quick) and 4.76–4.78 GiB
  (full), under the editor's estimates of 4.00 and 5.91 GiB, but the process's resident memory
  peaks at 4.7–4.9 GiB (quick) and 9.5–11.1 GiB (full): the estimate under-counts by 0.7–1 GiB
  (quick) and 4–5 GiB (full). That is the memory-accounting fix another agent is making; it was
  left alone here.

## Method

- **Machine.** The test VM (Azure D32as_v7, AMD EPYC 9V45, 32 vCPU, 125 GiB), shared with the other
  agents: load 20–45 throughout (the admission queue held 60–110 jobs). Wall times are therefore
  pessimistic against a quiet machine; the before and after binaries of each pair ran side by side
  (16 of 32 CPUs) so both saw the same load, and the ratio is the reliable number.
- **8-thread gate.** `deeptime bench --preset <p> --start <s> --length-myr 1000 --threads 8`
  (`RAYON_NUM_THREADS=8`, `/usr/bin/time -v`), each admitted as `sim:<p>:8t`. The presets name no
  timeline; 1,000 Myr is the length the earlier P1-17 runs and the decisions log's reference used.
- **Player profile.** `tools/review/p1-24-profile.sh mid6` (a cgroup scope of 12 CPUs and 6 GiB, 11
  Rayon threads, the client's 4 GiB cap; full at 16 GiB since it does not fit 6), before pinned to
  CPUs 0–11 and after to 12–23 at the same time.
- **Scaling.** The after binary at 1, 2 and 4 threads (quick) and 2 and 4 (full), scattered
  continents, 1000 Myr, beside the 8-thread pair.
- **Equivalence.** `cmp.py`: every file of the two world folders by SHA-256; `manifest.json` as
  JSON without wall times, the history's reserved-bytes accounting and its spill timings.
- Scripts and logs: `/Volumes/Graphide/ban/deeptime-runs/p1-17-bench/` on the VM (`run3.sh`,
  `v1.sh`, `pp.sh`, `prof24.sh`, `tablev1.py`, `scalingtab.py`, `cmp.py`, `serial.sh`; `out/`).

## The 8-thread gate, per stage (seconds)

‹TABLES›

The surface stage's `diffusion` now includes the slope failure, which master timed inside it;
the branch times `slope_failure` (and `diffusion_solve`) apart.

## Scaling across thread counts (after, scattered continents, 1000 Myr)

‹SCALING›

The 8-thread column is the side-by-side run; the others ran at the same time as each other, under
the same shared load. From 4 to 8 threads the quick preset gains 10 %: the climate (`ebm2d`,
`clock`), the structure stage's deformation and the surface's glacial and flood passes do not
scale at all.

## Where the remaining time goes (quick, 8 threads, after)

- **Coupled stages: 88 % of the wall time** (plates 137, climate 153, surface 250, volcanoes 12,
  structure 61 of 694 s); the fixed costs (deposits, young landscape, amplification, bake) 80 s.
- **Serial work** (a `perf` profile with frame pointers, quick 200 Myr at 8 threads: the driver
  thread's samples less a worker's; ×5 for 1000 Myr where the cost is per step, inferred):
  the zonal climate model's time steps (`ebm::run`, `thomas`: about 8 s per 200 Myr; sequential in
  time), the craton-edge distance transform (`dist_to_edge`, 3.6 s; called twice per structure
  step and per surface frame), the box blur's copy-back (3.2 s), the amplifier's priority flood
  (`flood_repair` and its heap, 6.6 s, once per run), the slope failure's apply loop (2.6 s), the
  2-D climate's per-cell loops (2.2 s), the plate mesh rebuild at resampling (`hull`, 3.8 s), the ice
  margin distance's copy-back (1.7 s), hydrology (1.7 s), the priority flood of the surface
  (`pflood`, about 1 s; serial by design).
- **The biggest parallel items** (they scale, but they are large): the slope failure (58 s quick,
  164 s full at 8 threads; about 40 sweeps over every cell per step), the diffusion solve (24 s;
  282 s full at 4 threads), flexure (62 s), advection (24 s), the 2-D climate (55 s), the plates
  (137 s quick, 400 s full; no sub-timings yet).

## What was optimised (all bit-identical, each with a test)

Branch `worktree-agent-af385527348745fb2`; the tests named are in the files changed.

| Where | Change | Test |
|---|---|---|
| surface | `Dims::neighbours` finds the cell's column and row once | neighbour order test |
| surface / marine | the submarine slope failure recomputes only the neighbourhood of the last iteration's movers (the full sweep when they are many), marks it on the raster and scans rows in parallel; applies the movers rows in parallel and lists them only for the sparse path | active set = full sweep bit for bit, 1/2/4/8 threads, grids 1–160 wide |
| surface | the step's per-cell loops and maps run in parallel; the deposit fields are written into their columns; the strata log is carried without zero-filling | coupled pipeline 1 vs 3 threads, whole world |
| surface / flexure | each preconditioner level in two passes instead of six; the cell-by-cell passes in parallel | preconditioner = old passes |
| surface / glacial | the ice solve walks the drainage forest in parallel | same bits on any thread count |
| surface / grid | the box blur copies its columns back rows in parallel | = serial copy-back |
| surface / fields | the craton-edge distance finds its edges in parallel and sweeps the interior by index steps | = reference on random masks |
| structure | the horizon smoothing walks each neighbour's stack once; the frame's host fields in parallel; the two distance transforms side by side | smoothing = old walk; pipeline test |
| volcanoes | edifices measured and the front found in parallel | = serial |
| climate | the per-cell energy balance factors its implicit solves once per run; the ocean's transport gathers each cell's faces rows in parallel; the present-climate search runs its bracket's six trials side by side; the step's margins, masks and ice fields mapped in parallel; the ice-margin distance copies back in parallel | factored = old loop; ocean = old; pipeline test |
| climate / ebm | the zonal model's two tridiagonal systems factored once per run | = `thomas` bit for bit |
| plates | the L1 sampling reads its rows back field by field in parallel | pipeline test |
| deposits | the favourability layers made in parallel | pipeline test |
| tests | the coupled pipeline on 1 and 3 threads: every field and the whole world body, bit for bit | — |
| bench | the editor's estimate printed beside the measurement; slope-failure sweep counters; climate and surface sub-timers | — |

Measured per stage (quick, scattered, 8 threads, master → branch): volcanoes 38.7 → 12.4 s, surface
diffusion with the slope failure 155.5 → 82.2 s, advection 43.8 → 24.2 s, the 2-D climate 81.0 →
55.0 s, the structure outputs 15.8 → 10.0 s, the bake 20.4 → 15.4 s.

## What it would take (inferred)

- **Quick 16k at 8 threads, 300 s**: 2.3× from here. Removing the serial items above (a few more
  bit-identical passes) is worth perhaps 10–15 %; the rest needs the coupled stages to scale past 4
  threads (today 1.1× from 4 to 8): coarser-grained parallel work in the climate step (the 2-D
  model's per-step loops are short and synchronise 70 times a simulated year), a parallel distance
  transform (exact, with a test against the chamfer sweeps), and the plates stage measured by
  sub-step first (it has no timers).
- **Full 16k at 8 threads, 900 s**: needs both better scaling and less work per step; the CPU time
  alone (7,140 s at 2 threads) is about 8 × 890 s.
- **Not bit-identical, so not done here** (calibration is out of scope): fewer slope-failure
  iterations or a convergence stop, fewer climate snapshots or years, the plate time step. P1-24
  measured some of these (4-Myr steps 1.6×).

## LZ4 compression per layer (quick, scattered, 1000 Myr; measured from the manifest)

Simple-dtype layers 155.4 MiB raw → 50.3 MiB stored (3.09×); all layers 62.3 MiB stored. L3
elevation (i16) 1.50×, detail amplitude (u8) 2.60×, water level 206×, playa 225×; L1/L2 climate
temperatures 1.2–1.4×, monthly precipitation shares 2.3–3.2×, Köppen 8.5×, vegetation 6.1×, ice
cover 85×; plate id 23×, crust age 2.9×, heat flow 3.3×, province 20×. Strata horizons
(a stack type) 12.1 MiB.
