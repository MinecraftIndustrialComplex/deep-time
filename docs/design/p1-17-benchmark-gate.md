# The P1-17 benchmark gate (V1): measurements and the per-stage report

Status: **the gate is missed on both presets** (2026-10-05; re-measured on the rebuilt test VM
2026-10-16, below). Ben's answer (decisions log 2026-10-06): accept it for V1 and keep optimising. This page is
the per-stage report the plan asks for when a preset misses (§9.1: "Missing it here is reported with
the per-stage breakdown and the measured scaling across thread counts (1, 2, 4, 8), and the decision
is Ben's"). Every speed-up on the branch is bit-identical: the worlds before and after are the same
files byte for byte. Ben's answer to P1-24 (2026-10-04, "Optimization only") makes optimisation the
only route to a faster Create World, so the second target, a player's 6-core PC, is measured here
too.

Statements are marked **measured** (a run made for this page), **read** (from the code) or
**inferred**.

## Summary

- **Re-measured on the rebuilt VM (2026-10-16; below)**: measured, 8 threads, 1000 Myr, before =
  master a55f70f6, worlds identical (83 of 83 files). Side by side, scattered: quick 774.5 →
  **577.2 s (1.34×)**, full 2,815.7 → **2,054.5 s (1.37×)**. One run at a time, with this
  session's passes (the two runs of a pair saw different loads, so their CPU time is the fairer
  ratio): quick scattered 895.8 → **559.7 s** (CPU time 1.33×), quick supercontinent 763.4 →
  **488.7 s** (1.33×), full scattered 2,053.5 → **1,251.7 s** (1.39×). Still over the gate: quick
  1.6 to 1.9×, full 1.4×. Ben accepted the miss for V1 (decisions log 2026-10-06).
- **The memory test passes again** (2026-10-17, below): two parts of a run worked on Rayon's
  global pool, every core of the machine, instead of the session's threads. Both now run in the
  session's pool, and a new test checks it. The resident peak fell from 124 to 133 MiB to 114 MiB,
  under the 119 MiB bound. The bake's estimate row is explained there.
- **The 8-thread gate (plan P1-17: quick ≤ 300 s, full ≤ 900 s; 1000 Myr)**: measured, test VM,
  8 threads, before = master 6489d4a5, after = this branch, run side by side, worlds identical:

  | preset | start | before | after | × | gate | result |
  |---|---|---:|---:|---:|---:|---|
  | quick 16k | scattered continents | 718 s | **541 s** | 1.33 | 300 s | missed, 1.8× over |
  | quick 16k | supercontinent | 771 s | **552 s** | 1.40 | 300 s | missed, 1.8× over |
  | full 16k | scattered continents | 2,813 s | **2,001 s** | 1.41 | 900 s | missed, 2.2× over |
  | full 16k | supercontinent | 2,901 s | **2,119 s** | 1.37 | 900 s | missed, 2.4× over |

  Each 8-thread run got 4.1–4.9 CPUs on average on the shared VM (`cpu %` below), so on eight
  free cores the times would be lower; the after-runs' CPU time (2,301–2,365 s quick, 8,365–8,368 s
  full) over eight cores at the efficiency measured from 1 to 4 threads (65 %) gives about 450 s
  quick and 1,600 s full (inferred): still over the gate.
- **Memory** (measured): the run's count peaks at 3.17–3.20 GiB (quick) and 4.08–4.09 GiB (full)
  against the editor's estimates of 4.00 and 4.82 GiB; resident memory 3.35–3.38 and 4.28–4.32 GiB
  (master's memory accounting, merged here). **Peak memory is inside the estimate** on every run.
- **The player's PC (P1-24's mid6 profile: 6 cores / 12 CPUs, 11 threads)**: measured, 500 Myr,
  scattered continents, worlds identical: **quick 342.6 → 287.0 s (1.19×)**, **full 1,404 → 1,067 s
  (1.32×)** (P1-24's baseline 349 s on master of 2026-10-04; full needs 16 GiB in the scope as
  before: 4.0 GiB counted, 6.2 GiB with the page cache of what it writes).
- **Bit-identical.** Every world file of every after-run equals the before-run's (83 files at
  1000 Myr, 73 on the player runs; the manifest compared without timings and spill accounting). A
  test runs the coupled pipeline on 1 and 3 threads and compares every field and the world body.
- **The young stage's estimate** (the coordinator's item): full 16k's was 6,049 MiB against the
  4,143 MiB the stage held; it is now 4,485 MiB (quick 1,931 → 1,240 MiB against 1,105 MiB held),
  so full fits a 5 GiB cap (below).
- **Why the gate cannot be met by these passes alone** (measured, inferred): one thread does the
  quick preset in about 1,900 s of CPU, so 300 s at 8 threads needs a parallel efficiency of about
  80 %; from 1 to 4 threads it is 65 % here, and a third of the 8-thread wall time is the driver
  thread working alone (214 of 623 s in the profile). The full preset's CPU time (7,140 s at
  2 threads) is itself about 8 × 890 s. Meeting 300/900 s needs less work per step (fewer
  slope-failure sweeps, fewer climate steps, coarser plate steps: not bit-identical, so Ben's
  call; P1-24 measured some) or a different parallel structure (the stages of one step overlapping).

## Re-measured on the rebuilt VM (2026-10-16)

The test VM was rebuilt (the same class: D32as_v7, 32 vCPU), so the 2026-10-05 times above are
not comparable with these. Before = master a55f70f6 (`ma55`, the master merged into the branch),
8 threads, 1000 Myr. All measured.

**Side by side, the first runs after the rebuild** (cold page cache; both builds in one admitted
job, as on 2026-10-05; after = `v6a`, the branch at d7e9eee2, which adds the L1 sampling's single
pass and the ocean's caches to the `v4a` of the tables above). Each run got 4.1 to 4.7 CPUs.

| stage | quick before | after | × | full before | after | × |
|---|---:|---:|---:|---:|---:|---:|
| plates | 126.0 | 118.6 | 1.06 | 505.1 | 490.2 | 1.03 |
| climate | 187.3 | 133.7 | 1.40 | 403.8 | 293.9 | 1.37 |
| surface | 278.4 | 188.1 | 1.48 | 1,215.9 | 857.4 | 1.42 |
| volcanoes | 34.3 | 10.4 | 3.29 | 128.7 | 38.7 | 3.32 |
| impacts | 1.6 | 1.6 | 0.96 | 4.6 | 4.6 | 1.00 |
| structure | 62.7 | 45.6 | 1.37 | 226.3 | 158.5 | 1.43 |
| deposits | 26.8 | 25.6 | 1.04 | 169.5 | 112.8 | 1.50 |
| young | 4.6 | 3.1 | 1.45 | 25.5 | 12.9 | 1.98 |
| amplify | 23.2 | 22.9 | 1.01 | 31.8 | 28.3 | 1.12 |
| finalize | 29.5 | 27.3 | 1.08 | 98.5 | 55.4 | 1.78 |
| **total** | **774.5** | **577.2** | **1.34** | **2,815.7** | **2,054.5** | **1.37** |
|   plates.write_l1 | – | 43.0 | | – | 193.2 | |
|   plates.resample | – | 20.8 | | – | 83.8 | |
|   climate.ebm2d | 91.4 | 56.2 | 1.63 | 92.9 | 58.5 | 1.59 |
|   surface.diffusion (with the slope failure) | 117.8 | 53.5 | 2.20 | 626.7 | 380.5 | 1.65 |
| user s | 3,052 | 2,427 | | 11,286 | 9,099 | |
| cpu % | 417 | 459 | | 414 | 465 | |
| peak GiB (gauge) | 3.17 | 3.16 | | 4.10 | 4.09 | |
| max RSS GiB | 3.35 | 3.35 | | 4.27 | 4.35 | |
| world files equal | 83 of 83 | | | 83 of 83 | | |

**One run at a time** (the coordinator's rule of 2026-10-16 for the shared VM: before, then
after, each its own admitted 8-thread job, `tools/review/p1-17-bench.sh seq`; after = `v7a`, which
adds this session's climate and smoothing passes to `v6a`). The two runs of a pair no longer see
the same load, so the wall-time ratio carries the VM's load as well as the code (`cpu %`: how many
CPUs each run got); the side-by-side ratio above stays the reliable one.

| stage | quick scattered before | after | × | full scattered before | after | × |
|---|---:|---:|---:|---:|---:|---:|
| plates | 140.7 | 125.2 | 1.12 | 360.2 | 305.3 | 1.18 |
| climate | 220.5 | 130.6 | 1.69 | 314.7 | 186.4 | 1.69 |
| surface | 328.0 | 187.1 | 1.75 | 870.2 | 489.9 | 1.78 |
| volcanoes | 37.0 | 10.6 | 3.50 | 110.3 | 24.5 | 4.49 |
| impacts | 1.8 | 1.7 | 1.08 | 3.3 | 3.1 | 1.07 |
| structure | 73.4 | 41.3 | 1.78 | 187.7 | 100.6 | 1.87 |
| deposits | 33.4 | 18.9 | 1.76 | 104.3 | 74.5 | 1.40 |
| young | 5.4 | 2.9 | 1.88 | 16.3 | 11.3 | 1.44 |
| amplify | 23.6 | 21.0 | 1.12 | 21.3 | 21.2 | 1.01 |
| finalize | 32.0 | 20.3 | 1.57 | 60.5 | 33.8 | 1.79 |
| **total** | **895.8** | **559.7** | **1.60** | **2,053.5** | **1,251.7** | **1.64** |
|   climate.ebm2d | 106.3 | 49.4 | 2.15 | 73.8 | 33.2 | 2.22 |
|   climate.clock | 32.8 | 24.4 | 1.34 | 37.8 | 26.8 | 1.41 |
|   structure.outputs (the horizon smoothing) | 20.3 | 5.8 | 3.50 | 43.5 | 16.4 | 2.65 |
|   plates.write_l1 | – | 44.4 | | – | 136.2 | |
|   plates.resample | – | 22.3 | | – | 45.2 | |
| user s | 3,228 | 2,420 | 1.33 | 8,689 | 6,240 | 1.39 |
| cpu % | 384 | 471 | | 438 | 524 | |
| peak GiB (gauge) | 3.17 | 3.16 | | 4.10 | 4.09 | |
| max RSS GiB | 3.34 | 3.36 | | 4.25 | 4.26 | |
| world files equal | 83 of 83 | | | 83 of 83 | | |

**Quick, supercontinent** (the same pair, between the two above): 763.4 → **488.7 s** wall (1.56×),
user 2,964 → 2,236 s (**1.33×**), cpu % 412 → 497, max RSS 3.31 → 3.32 GiB; the 83 world files
both runs wrote are equal. Both runs simulated to the end and then failed in the write, so there
is no `bench.json` and no stage table: `body 2034: semi-axes [0.0, 7.10946, 0.0] must be finite
and > 0`. That is master's bug, not this branch's: `bake/project.rs`'s `project_body` scales a
body's horizontal axes by the projection's Jacobian, and at or past the simulation's pole row the
Jacobian vanishes (read). The P1-19 branch (`worktree-agent-a5f716254d9f81516`, not merged on
2026-10-17) already keeps the source axes there and adds a test, so this branch leaves that file
alone.

Against the side-by-side `v6a`, the new passes show in their own timers. On quick the 2-D climate
went 56.2 → 49.4 s and the structure stage's outputs 10.1 → 5.8 s; these are different runs and
loads, so the change is inferred, not a measured ratio. Each before-run got fewer CPUs than its
after-run (3.8 against 4.7 on quick, 4.4 against 5.2 on full), so much of the 1.6× in wall time is
the load. The ratio of CPU time (user s: 1.33× quick, both starts, and 1.39× full) is the figure to
compare with the side-by-side 1.34 and 1.37. Full's one-at-a-time after-run, 1,251.7 s, is the
best full time on this VM so far. It is still 1.39× the 900 s gate, and quick's 488.7 to 559.7 s
are 1.6 to 1.9× the 300 s gate.

These worlds were made by `v7a`. The branch's head adds two things to it. One is the metamorphism
note's cell-order sum (below, "What was optimised"), which can move only the last bits of
`notes.structure.metamorphosed_km3` in the manifest; master's own value of that note changes from
run to run. The other is a test-only clippy fix. The master merged at the end (85c9cfe2) changes
in the core only the steering switch, which presets with steering on do not reach, and some tests.
So the head's worlds are `v7a`'s (read).

## Method

- **Machine.** The test VM (Azure D32as_v7, AMD EPYC 9V45, 32 vCPU, 125 GiB), shared with about
  ten other agents: load 25–70 on its 32 CPUs throughout (the admission queue held 20–110 jobs),
  so an 8-thread run got 3.6–4.5 CPUs on average (`/usr/bin/time`'s "Percent of CPU", 361–474 %).
  The absolute wall times are therefore those of about four cores, not eight, and pessimistic
  against a quiet machine; the before and after binaries of each pair ran side by side inside
  one admitted job, so both saw the same load, and **the ratio is the reliable number**.
- **8-thread gate.** `tools/review/p1-17-bench.sh pair` (`deeptime bench --preset <p> --start <s>
  --length-myr 1000 --threads 8`, `RAYON_NUM_THREADS=8`, `/usr/bin/time -v`). The presets name no
  timeline; 1,000 Myr is the length the earlier P1-17 runs and the decisions log's reference used.
  Before = master 6489d4a5, after = this branch (the same master merged in).
- **Player profile.** `tools/review/p1-17-player.sh` runs `tools/review/p1-24-profile.sh mid6` (a
  cgroup scope of 12 CPUs and 6 GiB, 11 Rayon threads, the client's 4 GiB cap; full at 16 GiB, since
  it does not fit 6) for both builds at once, before pinned to CPUs 0–11 and after to 12–23.
- **Scaling.** The branch at 1, 2 and 4 threads (quick) and 2 and 4 (full), scattered continents,
  1000 Myr, run before the last master merges (binary `v1a`, 2026-10-04), with its own 8-thread run.
- **Equivalence.** `tools/review/p1-17-cmp.py`: every file of the two world folders by SHA-256;
  `manifest.json` as JSON without wall times and the history's spill and reservation figures (when
  frames go to disk follows the allocator's count, which a different thread schedule changes; the
  frames are the same).
- **Where the time goes.** `tools/review/p1-17-profile.sh` (a frame-pointer build under `perf`,
  quick 1000 Myr at 8 threads; the driver thread's samples less a worker's), and the plate step's
  new sub-timers (`plates[...]` in `deeptime bench`'s timings line).
- Tables: `python3 tools/review/p1-17-table.py pair <out> <before> <after> 1000 8` and `... scale`.

## The 8-thread gate, per stage (seconds)

#### earthlike_quick_16k, 1000 Myr, 8 threads (seconds; before m648, after v4a)

| stage | scattered before | after | × | supercontinent before | after | × |
|---|---:|---:|---:|---:|---:|---:|
| plates | 110.6 | 109.9 | 1.01 | 125.6 | 115.9 | 1.08 |
| climate | 171.2 | 128.0 | 1.34 | 177.7 | 125.1 | 1.42 |
| surface | 255.2 | 165.0 | 1.55 | 263.4 | 164.5 | 1.60 |
| volcanoes | 33.3 | 9.9 | 3.38 | 31.9 | 9.0 | 3.53 |
| impacts | 1.5 | 1.6 | 0.96 | 1.9 | 1.9 | 1.01 |
| structure | 62.1 | 46.3 | 1.34 | 66.5 | 44.7 | 1.49 |
| deposits | 25.9 | 22.7 | 1.14 | 29.0 | 24.5 | 1.18 |
| young | 4.7 | 3.0 | 1.54 | 6.0 | 3.8 | 1.58 |
| amplify | 17.1 | 19.0 | 0.90 | 25.1 | 25.3 | 0.99 |
| finalize | 36.2 | 35.2 | 1.03 | 43.8 | 37.2 | 1.18 |
| total | 718.1 | 540.9 | 1.33 | 771.1 | 552.4 | 1.40 |
|   surface.diffusion | 103.3 | 44.1 | 2.34 | 105.1 | 43.4 | 2.42 |
|   surface.diffusion_solve | – | 13.6 |  | – | 14.8 |  |
|   surface.slope_failure | – | 30.0 |  | – | 28.1 |  |
|   surface.flexure | 47.5 | 37.3 | 1.27 | 49.6 | 37.5 | 1.32 |
|   surface.advect | 19.9 | 12.5 | 1.59 | 20.8 | 12.5 | 1.66 |
|   surface.glacial | 16.4 | 15.1 | 1.09 | 16.3 | 15.4 | 1.06 |
|   surface.fields_events | 17.5 | 12.9 | 1.36 | 19.3 | 12.9 | 1.50 |
|   surface.spl | 11.3 | 10.9 | 1.04 | 11.9 | 11.3 | 1.05 |
|   surface.pflood | 9.4 | 8.8 | 1.07 | 9.8 | 9.0 | 1.09 |
|   climate.ebm2d | 82.7 | 55.6 | 1.49 | 83.5 | 51.1 | 1.63 |
|   climate.clock | 24.5 | 21.2 | 1.16 | 28.3 | 22.0 | 1.29 |
|   climate.cells | 19.1 | 14.5 | 1.32 | 18.4 | 14.0 | 1.31 |
|   climate.ice | 14.4 | 10.1 | 1.43 | 14.7 | 10.3 | 1.43 |
|   climate.present | 7.4 | 6.0 | 1.23 | 9.9 | 6.7 | 1.48 |
|   structure.outputs | 12.6 | 9.5 | 1.33 | 15.0 | 8.8 | 1.70 |
|   structure.deformation | 24.0 | 13.9 | 1.73 | 24.1 | 13.7 | 1.76 |
|   structure.frame_fields | 11.3 | 8.2 | 1.38 | 12.0 | 7.8 | 1.54 |
|   volcanoes.births | 15.1 | 2.6 | 5.81 | 15.1 | 2.4 | 6.29 |
|   volcanoes.measure | 13.4 | 2.4 | 5.58 | 12.3 | 2.2 | 5.59 |
|   plates.feedback | – | 4.1 |  | – | 4.4 |  |
|   plates.flexure_sample | – | 1.8 |  | – | 1.9 |  |
|   plates.motion | – | 3.5 |  | – | 3.5 |  |
|   plates.detect | – | 7.9 |  | – | 8.1 |  |
|   plates.resolve | – | 0.8 |  | – | 0.8 |  |
|   plates.shortening | – | 1.1 |  | – | 1.1 |  |
|   plates.belts | – | 1.9 |  | – | 1.7 |  |
|   plates.fronts | – | 5.2 |  | – | 5.5 |  |
|   plates.collapse | – | 1.9 |  | – | 1.8 |  |
|   plates.transition | – | 5.4 |  | – | 6.7 |  |
|   plates.boundary_mix | – | 0.4 |  | – | 0.5 |  |
|   plates.forces | – | 1.4 |  | – | 1.3 |  |
|   plates.collision | – | 0.9 |  | – | 1.2 |  |
|   plates.hotspots | – | 0.0 |  | – | 0.0 |  |
|   plates.surface_finish | – | 0.8 |  | – | 0.8 |  |
|   plates.motion_rifting | – | 1.6 |  | – | 1.7 |  |
|   plates.close_bury | – | 0.2 |  | – | 0.2 |  |
|   plates.record | – | 3.8 |  | – | 4.3 |  |
|   plates.stats_margins | – | 6.1 |  | – | 5.8 |  |
|   plates.write_l1 | – | 41.3 |  | – | 42.6 |  |
|   plates.resample | – | 18.9 |  | – | 20.7 |  |
| user s | 2918.5 | 2364.5 |  | 2952.8 | 2300.7 |  |
| cpu % | 436 | 488 |  | 410 | 461 |  |
| peak GiB (gauge) | 3.20 | 3.20 |  | 3.18 | 3.17 |  |
| max RSS GiB | 3.38 | 3.38 |  | 3.35 | 3.36 |  |
| estimate GiB | – | 4.00 |  | – | 4.00 |  |
| estimate s | – | 811 |  | – | 811 |  |

#### earthlike_full_16k, 1000 Myr, 8 threads (seconds; before m648, after v4a)

| stage | scattered before | after | × | supercontinent before | after | × |
|---|---:|---:|---:|---:|---:|---:|
| plates | 482.6 | 454.9 | 1.06 | 486.3 | 480.4 | 1.01 |
| climate | 357.0 | 268.0 | 1.33 | 367.6 | 289.0 | 1.27 |
| surface | 1321.5 | 838.0 | 1.58 | 1329.3 | 907.0 | 1.47 |
| volcanoes | 125.8 | 36.6 | 3.44 | 118.6 | 36.1 | 3.28 |
| impacts | 4.6 | 4.6 | 1.01 | 4.9 | 5.0 | 0.97 |
| structure | 228.8 | 158.3 | 1.45 | 230.7 | 165.3 | 1.40 |
| deposits | 131.9 | 113.9 | 1.16 | 171.5 | 114.8 | 1.49 |
| young | 22.3 | 19.4 | 1.15 | 30.2 | 16.6 | 1.82 |
| amplify | 24.3 | 27.4 | 0.89 | 28.8 | 27.5 | 1.05 |
| finalize | 95.7 | 76.6 | 1.25 | 113.5 | 73.7 | 1.54 |
| total | 2813.1 | 2000.8 | 1.41 | 2900.9 | 2118.5 | 1.37 |
|   surface.diffusion | 578.8 | 321.0 | 1.80 | 577.6 | 348.5 | 1.66 |
|   surface.diffusion_solve | – | 181.0 |  | – | 195.6 |  |
|   surface.slope_failure | – | 137.6 |  | – | 150.1 |  |
|   surface.flexure | 93.2 | 71.1 | 1.31 | 89.8 | 75.1 | 1.20 |
|   surface.advect | 149.4 | 79.8 | 1.87 | 149.3 | 85.0 | 1.76 |
|   surface.glacial | 88.6 | 67.9 | 1.30 | 94.8 | 75.4 | 1.26 |
|   surface.fields_events | 116.6 | 63.0 | 1.85 | 117.4 | 65.8 | 1.78 |
|   surface.spl | 56.7 | 47.0 | 1.21 | 63.9 | 57.7 | 1.11 |
|   surface.pflood | 47.0 | 44.5 | 1.06 | 50.0 | 48.2 | 1.04 |
|   climate.ebm2d | 81.9 | 51.8 | 1.58 | 84.0 | 57.9 | 1.45 |
|   climate.clock | 43.3 | 37.3 | 1.16 | 49.4 | 40.4 | 1.22 |
|   climate.cells | 75.3 | 57.2 | 1.32 | 75.5 | 59.9 | 1.26 |
|   climate.ice | 58.9 | 38.6 | 1.53 | 60.3 | 42.1 | 1.43 |
|   climate.present | 15.6 | 11.5 | 1.36 | 19.8 | 15.3 | 1.29 |
|   structure.outputs | 52.4 | 37.0 | 1.42 | 52.2 | 39.6 | 1.32 |
|   structure.deformation | 94.0 | 51.5 | 1.83 | 94.3 | 53.3 | 1.77 |
|   structure.frame_fields | 48.4 | 36.1 | 1.34 | 49.1 | 37.0 | 1.33 |
|   volcanoes.births | 59.6 | 9.7 | 6.14 | 60.5 | 10.7 | 5.65 |
|   volcanoes.measure | 48.3 | 9.3 | 5.19 | 41.4 | 8.1 | 5.11 |
|   plates.feedback | – | 20.9 |  | – | 21.9 |  |
|   plates.flexure_sample | – | 8.9 |  | – | 9.4 |  |
|   plates.motion | – | 12.1 |  | – | 12.3 |  |
|   plates.detect | – | 24.4 |  | – | 25.9 |  |
|   plates.resolve | – | 2.8 |  | – | 2.9 |  |
|   plates.shortening | – | 3.9 |  | – | 4.2 |  |
|   plates.belts | – | 7.2 |  | – | 6.8 |  |
|   plates.fronts | – | 21.7 |  | – | 22.0 |  |
|   plates.collapse | – | 7.5 |  | – | 8.4 |  |
|   plates.transition | – | 28.2 |  | – | 31.1 |  |
|   plates.boundary_mix | – | 1.7 |  | – | 1.8 |  |
|   plates.forces | – | 5.3 |  | – | 5.5 |  |
|   plates.collision | – | 5.9 |  | – | 6.8 |  |
|   plates.hotspots | – | 0.0 |  | – | 0.0 |  |
|   plates.surface_finish | – | 3.0 |  | – | 3.0 |  |
|   plates.motion_rifting | – | 6.7 |  | – | 7.3 |  |
|   plates.close_bury | – | 0.6 |  | – | 0.7 |  |
|   plates.record | – | 17.4 |  | – | 18.4 |  |
|   plates.stats_margins | – | 13.4 |  | – | 14.8 |  |
|   plates.write_l1 | – | 193.2 |  | – | 206.5 |  |
|   plates.resample | – | 68.5 |  | – | 69.1 |  |
| user s | 10159.6 | 8368.0 |  | 10389.2 | 8364.7 |  |
| cpu % | 389 | 458 |  | 384 | 432 |  |
| peak GiB (gauge) | 4.08 | 4.08 |  | 4.09 | 4.09 |  |
| max RSS GiB | 4.28 | 4.29 |  | 4.29 | 4.32 |  |
| estimate GiB | – | 4.82 |  | – | 4.82 |  |
| estimate s | – | 3038 |  | – | 3038 |  |

`surface.diffusion` includes the slope failure, which master times inside it; the branch also
times `slope_failure` and `diffusion_solve` apart.

## The player's PC (mid6)

| mid6, 500 Myr, scattered | quick before | quick after | × | full before | full after | × |
|---|---:|---:|---:|---:|---:|---:|
| wall (s) | 342.6 | 287.0 | 1.19 | 1,404.1 | 1,067.0 | 1.32 |
| plates | 45.9 | 48.5 | 0.95 | 195.2 | 209.0 | 0.93 |
| climate | 73.9 | 69.3 | 1.07 | 181.0 | 154.4 | 1.17 |
| surface | 108.2 | 74.8 | 1.45 | 610.6 | 410.8 | 1.49 |
| volcanoes | 14.9 | 4.2 | 3.55 | 62.1 | 17.7 | 3.51 |
| structure | 31.3 | 22.6 | 1.38 | 129.2 | 88.6 | 1.46 |
| deposits | 14.4 | 16.4 | 0.88 | 76.3 | 60.3 | 1.27 |
| young | 4.2 | 4.5 | 0.93 | 26.3 | 19.0 | 1.38 |
| amplify | 16.5 | 17.8 | 0.93 | 26.8 | 28.8 | 0.93 |
| bake and write | 32.4 | 27.9 | 1.16 | 93.7 | 74.9 | 1.25 |
| user CPU (s) | 1,778 | 1,617 | | 7,025 | 5,846 | |
| counted peak (MiB) | 2,979 | 2,977 | | 4,143 | 4,141 | |
| scope peak (GiB, with page cache) | 3.08 | 3.09 | | 6.21 | 6.22 | |
| CPU wait in the scope (s) | 8.5 | 8.5 | | 65.7 | 67.8 | |

Both builds ran at the same time on their own 12 CPUs (load 15–25 on the VM). The plates stage,
which this branch barely touched, is now the second largest after the surface; the bake includes
the Petroff–Guyou projection pass (about 23 s on full, master's).

An earlier pair (master 2e07d954 against this branch, before the memory-accounting merge) showed
the branch holding 0.8–0.9 GiB more resident memory on mid6 than master at the same counted peak:
glibc's per-thread arenas keeping freed blocks the parallel loops had allocated (with
`MALLOC_ARENA_MAX=2` the same run held 3.72 instead of 4.86 GiB). Master's counting allocator, which
maps every block of 1 MiB or more on its own, removed it: the two builds now hold the same.

## Scaling across thread counts (branch `v1a`, scattered continents, 1000 Myr)

#### earthlike_quick_16k, scattered continents, 1000 Myr: seconds by thread count

| stage | 1 t | 2 t | 4 t | 8 t | 1 → 8 t |
|---|---:|---:|---:|---:|---:|
| plates | 424 | 239 | 165 | 137 | 3.10× |
| climate | 314 | 201 | 156 | 153 | 2.06× |
| surface | 713 | 406 | 276 | 250 | 2.85× |
| volcanoes | 30 | 19 | 14 | 12 | 2.40× |
| structure | 145 | 87 | 64 | 61 | 2.39× |
| deposits | 121 | 60 | 30 | 31 | 3.95× |
| amplify | 144 | 69 | 35 | 28 | 5.05× |
| total | 1969 | 1131 | 764 | 694 | 2.84× |
| surface.diffusion | 257 | 136 | 88 | 82 | 3.12× |
| surface.slope_failure | 168 | 93 | 62 | 58 | 2.90× |
| surface.flexure | 161 | 90 | 64 | 62 | 2.58× |
| surface.glacial | 30 | 22 | 18 | 18 | 1.60× |
| surface.pflood | 11 | 10 | 10 | 10 | 1.07× |
| climate.ebm2d | 76 | 55 | 48 | 55 | 1.39× |
| climate.clock | 40 | 32 | 30 | 33 | 1.21× |
| climate.cells | 91 | 48 | 29 | 19 | 4.72× |
| structure.deformation | 26 | 25 | 25 | 26 | 0.98× |
| user s | 1906 | 1861 | 1982 | 2389 | |
| cpu % | 99 | 170 | 274 | 378 | |

#### earthlike_full_16k, scattered continents, 1000 Myr

| stage | 2 t | 4 t | 8 t | 2 → 8 t |
|---|---:|---:|---:|---:|
| plates | 970 | 614 | 400 | 2.43× |
| climate | 604 | 413 | 301 | 2.01× |
| surface | 1852 | 1186 | 824 | 2.25× |
| structure | 356 | 256 | 188 | 1.89× |
| total | 4362 | 2891 | 1955 | 2.23× |
| structure.deformation | 98 | 96 | 90 | 1.09× |
| user s | 7142 | 7701 | 7840 | |
| cpu % | 169 | 277 | 425 | |

(The full table, with every sub-timer, is `p1-17-table.py scale` over the runs' logs.)

These runs ran beside each other and other agents' jobs: `cpu %` says how many CPUs each got
(378 % at 8 threads). From 4 to 8 threads the quick preset gains 10 % here, but most of that is
the box (the 8-thread run got under four CPUs), not the code: the user CPU grows from 1,906 s at
one thread to 2,389 s at eight (spinning workers), the wall time does not fall as far.

## Where the remaining time goes (quick, 8 threads, branch)

- **Coupled stages: about 88 % of the wall time**; the fixed costs (deposits, young landscape,
  amplification, bake) the rest.
- **The plate step** (new sub-timers, quick 1000 Myr, 8 threads, 130 s in all): the L1 sampling
  (`write_l1`) 44.5 s (a 6-nearest k-d-tree query and a containing-triangle search for each of
  336k cells every step, then 30 fields), resampling 22.1 s (the serial convex hull, `hull::orient`
  and `build_seeded`: 14 s of it on one thread), the ocean–continent transition 8.6 s, overlap
  detection 8.0 s, subduction fronts and uplift 7.6 s, statistics and margins 6.7 s, the dynamics
  record 6.1 s, the surface feedback 5.0 s.
- **Serial work** (`perf`, the driver thread's samples less a worker's, 214 s of a 623 s run):
  the zonal climate model (`ebm::run`, `Tri::solve`: 17 s; serial in time), the 2-D climate's
  per-cell loops (`ebm2d::run`: 9 s) and the ocean's set-up and sub-step count (`Ocean::build`,
  `conveyor`, `substeps`: 9 s, recomputed every snapshot or step although fixed), zero-filling new
  vectors (`memset` 8 s), the plate mesh rebuild (14 s), the surface's priority flood (`pflood`
  10 s, serial by design), subduction fronts (4 s), the k-d tree build's median split (4 s), the
  transition's Dijkstra (3 s), the glaciers' step (3 s), the dynamics record (3 s).
- **The largest parallel items**: the slope failure (43 s quick, 159 s full; about 40 sweeps over
  every cell per step), the diffusion solve (21 s; 213 s full), flexure (50 s), advection (24 s),
  the 2-D climate (62 s), the L1 sampling (45 s).

### The profile of 2026-10-16, and what is left to other files' owners

A `perf` profile of `v6a` on the rebuilt VM (`tools/review/p1-17-profile.sh`, quick 1000 Myr,
8 threads, 594 s): the driver thread 536 s of samples, a worker 325 s, so about 211 s of serial
work (measured). The serial items, largest first: the zonal climate model (`EbmParams::run`
12.7 s and `Tri::solve` 6.2 s), the 2-D climate's own loop (`ebm2d::run` 8.8 s), zero-filling new
vectors (`memset` 7.7 s), the plate mesh rebuild (`hull::orient` 7.2 s, `build_seeded` 5.2 s),
copies (`memmove` 5.4 s), the plate step's own body (5.3 s), the conveyor's operator (4.2 s), the
k-d tree build's median split (3.8 s), the priority flood (`open_sea`, `flood` and the heap:
9.6 s), the glaciers' step (3.3 s), the subduction fronts' Dijkstra (3.2 s), the craton-edge
distance's serial part (3.2 s), the ocean's build (3.0 s). The two zonal solves now run
interleaved, the 2-D loop's serial passes are folded into its parallel ones, and the conveyor's
operator runs by rows without a division (this session's rows in the table below).

The parallel item with the most CPU is the L1 sampling's nearest-sample query
(`SampleTree::nearest_n`: 7.7 % of all samples with its callees, 4.5 % its own; 59 % of the
counting allocator's 2 % is the `Vec` each query returns). These are in files other agents own,
so they are recorded here and not edited (the coordinator's rule, 2026-10-16):

| file (owner) | item | what would help | bit-identical? |
|---|---|---|---|
| `plates/l1.rs` (plates) | the relink asks the 6 nearest visible samples for every cell (`vis.nearest_n(q, 6)`), though the nearest sample's group covers the cell almost always | ask the nearest one (no allocation) and run the 6-query only when its group's triangle does not cover the cell | only up to exact distance ties between two samples (which sample the two queries return first); needs the world comparison on both presets |
| `sphere/kdtree.rs`, kiddo 6.3 | `nearest_n` returns a new `Vec` per query | a query into a caller's buffer (kiddo's API has none; a wrapper keeping a per-thread buffer) | yes |
| `plates/ridges.rs`, `sphere/hull.rs` (plates) | the mesh rebuild after resampling: the hull's orientation tests and the seeded build, serial, 12 s on quick | in `ridges::resample`, the new lattice's hull (`Geo::build`) and the `found` search, which reads only the new points and the old mesh, side by side (read); a parallel hull | yes for the overlap; a parallel hull if it stays exact (it is checked exactly) |
| `surface/fields.rs`, `surface/pflood.rs`, `surface/glacial.rs` (erosion) | the craton-edge distance's serial sweep (3 s), the priority flood (10 s, serial by design), the glaciers' step (3 s) | the sweep by rows; the flood stays serial | yes for the sweep |
| `alloc.rs` (memory accounting) | the counting allocator's global atomics (2 % of the CPU, most of it from the per-query `Vec` above) | per-thread counts folded at a stage's end, keeping the same totals | yes |
| `bake/project.rs` (the seamless world) | `resample_grid`, the bake's projection pass | not profiled in detail here | |
| `bake/project.rs` (the seamless world; P1-19 branch) | not speed: `project_body` gives a body zero horizontal axes where the projection's Jacobian vanishes (at or past the simulation's pole row), and the write then fails; quick supercontinent 1000 Myr on master and on this branch | keep the source axes there: done on the P1-19 branch (`worktree-agent-a5f716254d9f81516`) with a test, not merged on 2026-10-17 | changes only worlds that failed to write |
| `alloc.rs`, `session.rs` (memory accounting) | the session's own threads' glibc arenas: about 8 MiB of the memory test's resident peak above the count after this branch's pool fix | `malloc_trim` when a stage ends, or a cap on arenas | yes |
| `climate/clock.rs` | the zonal climate clock, one EBM run per carbon step | sequential in time by nature | |

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
| plates / L1 (2026-10-05) | the L1 sampling interpolates its 14 fields in one pass over each cell's three samples and maps the nearest samples in place | = `Coupling::interpolate` bit for bit |
| climate / ocean (2026-10-05) | the sub-step count cached per build; the conveyor's basins bucketed in one pass | cached = computed for any dt; buckets = per-basin scans |
| climate / ebm (2026-10-16) | with both diffusions on, the atmosphere's and the slab's tridiagonal solves run interleaved in one pass over the bands (`Tri::solve_pair`) | pair = each system's own solve, bit for bit |
| climate / ebm2d (2026-10-16) | the step's serial per-cell passes folded into its parallel ones: T̄ made in the energy balance, the tiles' change of T̄ with the zonal shift, freezing with the month's means, by rows; the three global sums in one loop, each in cell order | = the old loop, bit for bit |
| climate / ocean (2026-10-16) | the gyres' sweep and the conveyor's operator find wrapped neighbours without a division; the operator by rows | neighbours = the modulo forms |
| structure (2026-10-16) | the horizon smoothing finds each neighbour's spanning class for every interval at once (a binary search over each class's key span) instead of a first-match search per interval | table = first-match search, 2,000 random stacks; smoothing = reference, 1/2/8 threads |
| deposits | the favourability layers made in parallel | pipeline test |
| tests | the coupled pipeline on 1 and 3 threads: every field and the whole world body, bit for bit | itself |
| bench | the editor's estimate printed beside the measurement; slope-failure sweep counters; climate, surface and (new) plate sub-timers | none |

**Two fixes that are not optimisations.** The session's threads (2026-10-17): see "The memory
test and the session's threads" below. The metamorphism note (2026-10-16) is the same on every
run. `structure/mod.rs` summed each column's metamorphosed thickness with a parallel float
`sum`. Rayon splits that sum by how the threads steal work, so the terms grouped differently
from run to run and the last bits of `notes.structure.metamorphosed_km3` changed. The
`history_cap` test caught it: on the rebuilt VM two runs of one planet differed in that note.
The columns are now worked in parallel and their thicknesses summed serially in cell order, the
codebase's rule for float sums. The worlds are unchanged, since the note is all it fed; the
note's own value is now fixed, where master's moved between runs. `history_cap` is the test.

## The young stage's memory estimate (tightened here)

The young stage's estimate was the surface stage's whole estimate (its step's working arrays, the
strata log's advected copy and the glaciers' passes, all freed by the time the young stage runs)
plus 352 B for each cell of a grid with half as many rows as columns (the Lambert L2 grid has
1/π as many): 6,049 MiB on full 16k against the 4,143 MiB the stage held, so the editor refused
full at a 5 GiB cap it fits under. It is now the world the coupled stages leave
(`surface::retained_bytes`: the plates' estimate, the surface's retained fields and log, the
climate's 24 monthly maps), the history, and 460 B for each cell of the real L2 grid.

| measured (500 Myr, scattered, kept-block cache off) | stage held | own arrays per L2 cell | estimate before | estimate now |
|---|---:|---:|---:|---:|
| quick 16k | 1,105 MiB | 413 B | 1,931 MiB | 1,240 MiB |
| full 16k | 4,143 MiB | 416 B | 6,049 MiB | 4,485 MiB |

With it, full 16k's whole estimate fits a 5 GiB cap (test `the_estimate_covers_the_measured_hold_closely`).

## The memory test and the session's threads (2026-10-17)

The coordinator's item. `tests/memory_estimate.rs` runs a small planet through every stage and
the bake on a 2-thread session, under the game's counting allocator. It asserts that no stage
holds more than its estimate, that the run's peak is inside the estimate's peak, and that the
resident peak is within the counted peak plus 16 MiB plus a tenth (line 175). On the rebuilt VM
that last check failed on master and on this branch alike. All measured, `cargo test -p
deeptime-core --test memory_estimate -- --nocapture`, MiB:

| build | resident peaks | counted peak | bound | result |
|---|---|---:|---:|---|
| master a55f70f6 | 133.0, 139.8, 123.1 | 94.1 | 119.5 | fails 3 of 3 |
| master a55f70f6, `MALLOC_ARENA_MAX=1` | 103.9, 104.4 | 94.1 | 119.5 | passes |
| master ba8fb5cb (coordinator's gate batch) | 116.3 | 94.1 | 119.5 | passed on a rerun |
| branch before the fix (070c1548) | 133.3 (gate), 125.6, 128.0 | 94.0 | 119.4 | fails |
| the same, `MALLOC_ARENA_MAX=1` | 106.7, 105.6 | 94.0 | 119.4 | passes |
| the same, `RAYON_NUM_THREADS=2` | 114.6, 114.5 | 93.8 | 119.2 | passes |
| **branch with the fix (15febcde)** | **114.5, 113.7, 113.5** (and the gate) | **93.8** | 119.2 | **passes** |

**The count was right; the threads were not.** The allocator counts every block, and maps the
ones of 1 MiB or more itself. Smaller blocks go through glibc, which gives each thread an arena
and keeps freed blocks in it, so the resident set grows with the number of threads that
allocate. One arena (`MALLOC_ARENA_MAX=1`) brings the resident peak to about 12 MiB over the
count. `RAYON_NUM_THREADS` sizes only Rayon's global pool, not the session's (the session builds
its own), yet setting it to 2 took 11 to 13 MiB off the resident peak and even moved the
counted peak, so some of the run was working on the global pool. gdb on the test binary, with a
breakpoint where a thread outside every pool asks for the global registry, found two callers.
Both run on the test's own thread, outside the session's `install`:

- `Session::bake_water_into_last_frame` built the water view (`water_view::from_bake`, the
  hydrology of `hydro::network`) outside the pool. Only the redraw after it was inside.
- The deposits stage's finish built its coal seams' cover (`structure::Cover::new`, the horizon
  bake and smoothing) outside the pool. The stage runs its other parallel work in `install`
  calls of its own.

On the VM those parts ran on 32 threads beside a 2-thread session. In the game they would use
every core whatever the player's thread setting. The bench sets `RAYON_NUM_THREADS` to its own
thread count, so its timings and worlds are unaffected. Both calls now run in the session's pool,
and `tests/session_pool.rs` keeps it that way. That test gives the global pool one thread and
keeps that thread busy for the whole run, so a run that sends the global pool any work never
finishes. Without the fix the test was still waiting after 240 s; with it, it passes in 32 s. The
1-and-3-thread pipeline test still passes, so those two parts do not depend on the thread count
(they now run on the session's 1 or 3 threads where before both runs used the global pool).

The resident check now passes with about 5 MiB to spare. The rest of the gap over the count is
the session's own threads' arenas (one arena: 106 MiB). Trimming them (`malloc_trim` when a
stage ends, or a cap on arenas) is for whoever owns memory accounting; it is recommended, not
done here.

**The bake's row** (held 93.8 MiB against its estimate of 75.3, of which the projection pass is
18.0) is printed but not asserted, by design. `estimate_for` (`session.rs`) counts in
`bake_bytes` only the finished world and the projection pass's working set. It says that "the
Lambert bake itself has stayed inside the stages' peak so far; `tests/memory_estimate.rs` checks
the run's peak". The run's estimate, 96.8 MiB, does cover the bake's 93.8 (3 MiB, 3 %, to
spare), and the test asserts that. This branch's change in the bake's path, the horizon
smoothing's per-thread rows, leaves the bake's count where master has it (93.8 to 94.1 on both).
At 16k the estimate's `bake_bytes` are 1.95 GiB (quick) and 4.24 GiB (full), against run peaks
of 4.00 and 4.82 GiB estimated and 3.16 and 4.09 GiB counted. The bench records no peak for
the bake alone, so the estimate is left as it is: raising it without a measured 16k bake peak
would move the editor's figures on a guess. The margin is thin on the test planet, and a bake
change that adds a few MiB will fail the run-peak check. That check is the guard.

## What it would take (inferred)

- **Quick 16k at 8 threads, 300 s**: 1.8× from the shared-VM figure, about 1.5× from a quiet
  machine's (inferred). The serial items above are about a third of the 8-thread wall time; the
  bit-identical ones left (the ocean's per-snapshot set-up and sub-step count cached, the 2-D
  climate's per-cell loops in parallel, the zeroed allocations reused, a parallel hull) are worth
  perhaps 5–10 %. The zonal climate clock (17 s) and the surface's priority flood (10 s) are serial
  by nature.
- **Full 16k at 8 threads, 900 s**: its CPU time alone is about 8 × 1,050 s; it needs less work
  per step, not only better scaling.
- **Not bit-identical, so not done here** (calibration is out of scope; Ben decides): fewer
  slope-failure iterations or a convergence stop (43 s quick, 159 s full), fewer climate snapshot
  years, a coarser L1 sampling cadence (`l1_every_steps`, 45 s quick), the plate time step (P1-24:
  4-Myr steps 1.6×).
- **Structural** (inferred, large): the stages of one plate step run one after another on all
  threads, each with short parallel loops; letting the climate's snapshot or the structure stage
  of step k overlap the plates of step k + 1 would use the idle cores, but the stages share one
  world and that is a redesign.

## LZ4 compression per layer (quick, scattered, 1000 Myr; measured from the manifest)

Simple-dtype layers 155.4 MiB raw → 50.3 MiB stored (3.09×); all layers 62.3 MiB stored. L3
elevation (i16) 1.50×, detail amplitude (u8) 2.60×, water level 206×, playa 225×; L1/L2 climate
temperatures 1.2–1.4×, monthly precipitation shares 2.3–3.2×, Köppen 8.5×, vegetation 6.1×, ice
cover 85×; plate id 23×, crust age 2.9×, heat flow 3.3×, province 20×. Strata horizons
(a stack type) 12.1 MiB.
