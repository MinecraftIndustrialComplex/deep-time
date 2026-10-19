# The P1-17 benchmark gate (V1): measurements and the per-stage report

Status: **the gate is missed on both presets** (2026-10-05; re-measured on the rebuilt test VM
2026-10-16, and full 16k on eight pinned cores for P1-19 on 2026-10-18, below). Ben's answer
(decisions log 2026-10-06): accept it for V1 and keep optimising. This page is
the per-stage report the plan asks for when a preset misses (§9.1: "Missing it here is reported with
the per-stage breakdown and the measured scaling across thread counts (1, 2, 4, 8), and the decision
is Ben's"). Every speed-up on the branch is bit-identical: the worlds before and after are the same
files byte for byte. Ben's answer to P1-24 (2026-10-04, "Optimization only") makes optimisation the
only route to a faster Create World, so the second target, a player's 6-core PC, is measured here
too.

Statements are marked **measured** (a run made for this page), **read** (from the code) or
**inferred**.

## Summary

- **The shipped builds (2026-10-18; below).** Measured: full 16k, 100 Myr, 8 threads, pairs run both
  ways round. Against plain `release`, the `ship` profile's x86-64-v3 build is **1.08×** faster
  in wall time (1.10× in CPU time). Its baseline build matches plain in wall time (1.00×) and is
  1.03× faster in CPU time. All eight pairs built the same worlds (66 of 66 files). The gate now
  times both ship builds (`tools/natives/gate-bins.sh`) and reports both numbers.
- **P1-19 exit item 1, the stages of one step side by side (2026-10-18; below)**: measured, the
  same run on master 991354a4. The structure step now runs beside the next plate step, the only
  pair of a step's stages with no data between them. Side by side: 1,485.8 → **1,430.5 s
  (1.039×)** at others' load 4.8, 1,883.5 → 1,864.7 s (1.010×) under load; worlds identical (84
  of 84 files). Quiet on set A: **1,290.1 s**, 1.43× the gate. The other overlaps (deposits
  beside young and amplify, keyframes beside the next step, the deposit stage beside the group)
  would take at most about 110 s more (estimated), so the gate still needs less work per step.
- **P1-19 exit item 1, full 16k on eight pinned cores (2026-10-18; below)**: measured, 8 threads,
  1000 Myr, scattered continents, held to one CPU of each of 8 physical cores, worlds identical
  (84 of 84 files). Side by side with master e34dbe54: 2,255.8 → **1,990.4 s** and 2,316.0 →
  **2,045.6 s**, 1.13× both ways round (CPU time 1.13 to 1.15×, surface 1.28 to 1.34×, system time
  halved). Quiet, one run at a time on the same 8 cores: the branch **1,382.3 s** (others' load
  2.4), master 1,828.6 s (others' load 7.1). Missed: **1.54× the 900 s gate quiet**, 2.2× under
  the shared VM's usual load. Quiet, the work (6,657 s of CPU) would fit 8 fully busy cores in
  832 s, but about 630 s of the run is on one core (the plate mesh's hull, the priority flood and
  the zonal climate clock among it), so the gate needs less work per step or overlapping stages,
  which is Ben's call.
- **Re-measured on the rebuilt VM (2026-10-16; below)**: measured, 8 threads, 1000 Myr, before =
  master a55f70f6, worlds identical (83 of 83 files). Side by side, scattered: quick 774.5 →
  **577.2 s (1.34×)**, full 2,815.7 → **2,054.5 s (1.37×)**. One run at a time, with this
  session's passes (the two runs of a pair saw different loads, so their CPU time is the fairer
  ratio): quick scattered 895.8 → **559.7 s** (CPU time 1.33×), quick supercontinent 763.4 →
  **488.7 s** (1.33×), full scattered 2,053.5 → **1,251.7 s** (1.39×). Still over the gate: quick
  1.6 to 1.9×, full 1.4×. Ben accepted the miss for V1 (decisions log 2026-10-06).
- **Re-checked after the second merge of master (5bc63aa5, 2026-10-17; below)**: quick scattered,
  one run at a time, 761.3 → **574.0 s** (1.33×; CPU time 1.27×); every file the same but one
  manifest note's last two bits, which is the metamorphism fix.
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

## Running the gate: the shipped builds (2026-10-18)

The gate times what players run (decisions log 2026-10-18, "Both, picked at runtime"): the `ship`
profile (`release` plus fat LTO in one codegen unit, `Cargo.toml`), built twice, once for the
x86-64 baseline and once for x86-64-v3, the two builds an x86-64 player's machine can load (plan
§6.4). **Report both numbers**: v3 is what the test VM's EPYC and every x86-64 CPU since about
2013 (Intel) or 2015 (AMD) loads; the baseline is the floor for the rest. Commands (test VM):

```sh
# both binaries (and, with --plain, the everyday release build for comparison), admitted as cargo:build
tools/remote/mac.sh --kind cargo:build -- tools/natives/gate-bins.sh /data/ban/deeptime-runs/<task>/bins
# one run each, side by side: v3 on set A, the baseline on set B (then the other way round)
B=/data/ban/deeptime-runs/<task>
tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start gate-v3 --kind sim:earthlike_full_16k:8t \
    tools/review/bench-quiet.py --cpus 0,16,2,18,4,20,6,22 --out $B/v3 $B/bins/deeptime-x86-64-v3 \
    earthlike_full_16k scattered_continents 1000 8
tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start gate-baseline --kind sim:earthlike_full_16k:8t \
    tools/review/bench-quiet.py --cpus 8,24,10,26,12,28,14,30 --out $B/baseline $B/bins/deeptime-baseline \
    earthlike_full_16k scattered_continents 1000 8
```

`tools/review/p1-17-bench.sh seq <bins>/deeptime-x86-64-v3 v3 <bins>/deeptime-baseline baseline
...` takes the same two binaries. `tools/review/p1-17-profile.sh` profiles the ship build
(`P117_CPU=x86-64-v3` for v3). The worlds of the two builds need not match (Ben, 2026-10-18: "I
am not worried about determinism, performance is far more important").

### Plain release against the two shipped builds (measured, 2026-10-18)

`earthlike_full_16k`, scattered continents, **100 Myr**, 8 threads, `bench-quiet.py` on the test
VM. Each pair ran side by side, the first build on set A (0,16,2,18,4,20,6,22) and the second on
set B (8,24,10,26,12,28,14,30), and then the other way round. "Plain" is the everyday `release`
profile (thin LTO, 16 codegen units) built from the natives branch, whose `deeptime-core` is
master f221b3d3's. "Ship baseline" and "ship v3" are `gate-bins.sh`'s two binaries.

The VM was shared with 8 to 20 other jobs throughout: others' busy CPUs were 14 to 25 of 32, and
the runs' cores' SMT siblings were 33 to 88 % busy. So wall time moves with the load from pair to
pair. Within a pair both runs see the same load, and the two orders cancel the sets' difference.
CPU time (user + system) moved least. The speed-up column is the slower build's time over the
faster's.

| A | B | A wall | B wall | speed-up | A CPU | B CPU | speed-up | others' busy CPUs | siblings busy |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| plain | ship baseline | 480.7 s | 459.3 s | 1.047 | 1,311.0 s | 1,282.8 s | 1.022 | 24.8, 25.0 | 81, 81 % |
| ship baseline | plain | 250.5 s | 312.0 s | 1.246 | 1,089.5 s | 1,129.0 s | 1.036 | 13.9, 14.4 | 33, 36 % |
| plain | ship v3 | 321.4 s | 296.3 s | 1.085 | 1,286.5 s | 1,166.7 s | 1.103 | 20.7, 21.0 | 71, 73 % |
| ship v3 | plain | 360.0 s | 388.5 s | 1.079 | 1,238.7 s | 1,351.2 s | 1.091 | 24.9, 23.6 | 88, 85 % |
| ship baseline | ship v3 | 266.1 s | 231.7 s | 1.148 | 1,190.8 s | 1,123.0 s | 1.060 | 16.0, 15.2 | 50, 55 % |
| ship v3 | ship baseline | 272.8 s | 285.9 s | 1.048 | 1,087.8 s | 1,153.3 s | 1.060 | 20.1, 19.2 | 67, 65 % |
| plain (second round) | ship baseline | 316.6 s | 321.5 s | 0.985 | 1,072.2 s | 1,044.1 s | 1.027 | 14.2, 14.2 | 23, 23 % |
| ship baseline (second round) | plain | 309.1 s | 315.0 s | 1.019 | 1,025.2 s | 1,053.3 s | 1.027 | 12.7, 12.3 | 14, 14 % |

The plain and ship baseline pair ran a second time because the first round's two orders
disagreed (1.047 and 1.246). In the 1.246 run, the plain build on set B had 3.57 CPUs busy on
average against its partner's 4.26: something else held set B back. The second round ran with the
siblings 14 to 23 % busy.

The speed-ups are the geometric mean of the two orders:

| build | over | wall | CPU time |
|---|---|---:|---:|
| ship v3 | plain | **1.08×** (1.085, 1.079) | 1.10× (1.103, 1.091) |
| ship baseline | plain | **1.00×** in the second round (0.985, 1.019); 1.07× over all four pairs | 1.03× (1.022 to 1.036, all four pairs) |
| ship v3 | ship baseline | 1.10× (1.148, 1.048) | 1.06× (1.060, 1.060) |

Fat LTO in one codegen unit saves about 3 % of the CPU time, which is inside the wall-time
noise here. x86-64-v3 saves about 6 % more, and it showed in wall time too (1.05 to 1.15). By
CPU time the two compound: 1.027 × 1.060 = 1.089, against the 1.097 measured directly. Each
shipped build's speed-up over plain `release`:

- **v3: 1.08× in wall time**, 1.10× in CPU time.
- **Baseline: 1.00× in wall time** (the quieter round), 1.03× in CPU time.

All of these are at 100 Myr. A 1000 Myr gate run of both builds has not been made yet.

**The worlds.** `p1-17-cmp.py` compared every pair: 66 of 66 files were equal in all eight.
At 100 Myr, plain, ship baseline and ship v3 build the same world byte for byte. Nothing requires
it.

**The build cost** was measured on the test VM with `CARGO_INCREMENTAL=0`, each profile in its own
fresh target directory, while other agents held the load at 22 to 77. The `deeptime_jni` builds
followed the CLI's in the same directory, so the dependencies were already built.

| build | release | ship | ship v3 |
|---|---:|---:|---:|
| `deeptime` CLI, cold | 81.2 s | 273.8 s (3.4×) | 289.1 s (3.6×) |
| `deeptime_jni`, dependencies built | 75.5 s | 193.6 s (2.6×) | 259.1 s (3.4×) |
| CLI after touching `deeptime-core/src/lib.rs` | 71.7 s | 323.3 s (4.5×) | 305.4 s (4.3×) |
| `deeptime_jni` after the same touch | 64.9 s | 189.5 s (2.9×) | 244.6 s (3.8×) |

`gate-bins.sh` itself took 320.3 s and 305.8 s for its two builds. Fat LTO in one codegen unit
optimises the whole program again after every change, in one thread. That is why everyday builds,
Gradle's host native and the tests stay on `release`.

The whole cross-build, `tools/natives/build-natives.sh` with its 16 files, took 2,594 s on the
loaded VM: 2.4 to 4.1 min for each of the 12 libraries. The last CI run, of 2026-09-28, built 8
release-profile libraries of a much smaller core in 25 min. So `natives.yml`'s build job timeout
went from 30 to 180 min; the estimate on the 2-core private-repo runner is about 100 min
(inferred, not run).

**The sizes** (bytes; linux-x86_64-gnu):

| file | release | ship baseline | ship v3 |
|---|---:|---:|---:|
| `libdeeptime_jni.so` | 19,692,808 | 15,252,816 (−22.5 %) | 15,215,144 |
| the same, deflated (`gzip -6`, as in the jar) | 5,101,915 | 4,666,210 (−8.5 %) | 4,715,010 |
| `deeptime` CLI | 24,337,712 | 18,534,552 (−23.8 %) | 18,545,280 |

**The release jar** (`./gradlew jar -Prelease` after `build-natives.sh`, measured) is
**57,690,314 B**. Its 16 native files deflate to 55.8 MB:

- the eight baseline libraries, 35.5 MB;
- the four x86-64-v3 libraries, 19.4 MB;
- the four probes, 0.9 MB.

There is no comparable jar from master: the last CI artifacts date from 2026-09-28. The jar with
master's eight release-profile libraries is estimated at about 40.7 MB (inferred: the jar's 1.9 MB
of non-native content, plus the eight baseline libraries scaled by the release-to-ship ratio of
the deflated linux-gnu library, 1.093). So the jar grows by about 17 MB (1.42×). The v3 builds
add 19.4 MB, and the ship profile saves about 3.3 MB.

## P1-19 exit item 1: full 16k on eight pinned cores (2026-10-18)

Plan §10 P1-19's first exit item is this page's full 16k gate: 900 s at 8 threads, scattered
continents, 1000 Myr. The P1-19 exit suite's run took 1,785 s on the loaded VM. The coordinator's
call (decisions log 2026-10-17) kept the gate: re-measure on a quiet machine with the run held to
8 cores, then speed up the stages that dominate with the results unchanged, and take the gate to
Ben only if that cannot reach 900 s. Branch `worktree-agent-a21dc1b3e8bbf4f7e` from master
8f2b7e79, with master e34dbe54 (plate task 8, the erosion recalibration, the gossan, the
overturning sinks) merged before the last measurements, as the coordinator asked (2026-10-18).
**It cannot be reached with the results unchanged: quiet, the branch takes 1,382.3 s, 1.54× the
gate (2.2× under the VM's usual load), 1.13× faster than master side by side.** All measured
unless marked.

**Method.**

- `tools/review/bench-quiet.py` (new) runs one `deeptime bench --preset earthlike_full_16k --start
  scattered_continents --length-myr 1000 --threads 8` with `RAYON_NUM_THREADS=8` under
  `/usr/bin/time -v`. Its CPU affinity holds it to one logical CPU of each of 8 physical cores,
  set A: CPUs 0, 16, 2, 18, 4, 20, 6, 22, taken in turn from the VM's two L3 caches. Every 5 s it
  samples `/proc/stat`, so everyone else's busy CPUs are known for the whole run ("others' load"),
  and so is the busy share of set A's SMT siblings, which share the run's cores.
- A run starts once the 1-minute load has stayed under 6 for three minutes (or after a long wait),
  and is repeated while others' mean load over it was 8 or more. The VM is shared by several
  agents, whose sweeps come in waves of about fifteen 2-thread jobs, so the load seldom stays low
  for the half hour a run takes. Two runs met the mark (below): the branch at others' load 2.4,
  master at 7.1.
- This branch's builds and tests ran during the benches on the other 8 physical cores and their
  siblings (`taskset -c 8-15,24-31`), so they did not touch set A.
- Each world is compared with master's (`tools/review/p1-17-cmp.py`, as in the method below).
- Each pass's own gain comes from a 100 Myr pair: the binaries before and after it side by side
  in one job, one on set A and one on set B, so both saw the same load.

**Side by side at 1000 Myr** (seconds; both binaries at once, one on each CPU set, then the
other way round, so each pair saw the same load; master = e34dbe54, branch = 13f49fd5, its merge
of that master; others' load 15 to 16 throughout):

| stage | master (A) | branch (B) | × | master (B) | branch (A) | × |
|---|---:|---:|---:|---:|---:|---:|
| plates | 535.6 | 481.9 | 1.11 | 585.9 | 515.2 | 1.14 |
| climate | 277.7 | 284.5 | 0.98 | 282.7 | 286.7 | 0.99 |
| surface | 1,035.5 | 808.6 | **1.28** | 1,065.0 | 792.4 | **1.34** |
| volcanoes | 37.4 | 38.7 | 0.97 | 38.6 | 38.3 | 1.01 |
| impacts | 4.4 | 4.7 | | 4.7 | 4.9 | |
| structure | 151.0 | 155.8 | 0.97 | 158.2 | 164.1 | 0.96 |
| deposits | 100.0 | 93.3 | 1.07 | 94.4 | 133.4 | 0.71 |
| young | 20.8 | 18.3 | | 14.9 | 18.6 | |
| amplify | 32.6 | 47.2 | | 24.0 | 33.7 | |
| finalize | 51.0 | 52.6 | | 38.7 | 52.3 | |
| **wall** | **2,255.8** | **1,990.4** | **1.13** | **2,316.0** | **2,045.6** | **1.13** |
| CPU time, user + system | 8,371 + 867 | 7,645 + 423 | 1.15 | 9,025 + 920 | 8,388 + 446 | 1.13 |
| the run's CPUs, mean | 4.10 | 4.05 | | 4.29 | 4.32 | |
| others' busy CPUs, mean | 15.7 | 16.3 | | 15.1 | 16.2 | |
| its cores' SMT siblings busy | 50 % | 54 % | | 56 % | 59 % | |
| peak, MiB (count) | 4,239 | 4,200 | | 4,239 | 4,200 | |
| world files equal | 84 of 84 | | | 84 of 84 | | |

The surface's own phases (the same runs, master → branch):

| surface phase | master (A) | branch (B) | master (B) | branch (A) |
|---|---:|---:|---:|---:|
| diffusion, with its solve and the slope failure | 332.3 | 320.9 | 340.9 | 308.9 |
|   the diffusion solve | 183.4 | 173.9 | 191.8 | 172.9 |
|   the slope failure | 144.7 | 144.6 | 144.7 | 133.9 |
| spread | 120.8 | 57.9 | 120.5 | 56.6 |
| advect | 94.8 | 59.4 | 96.8 | 57.2 |
| glacial | 88.9 | 58.1 | 92.5 | 55.6 |
| flexure | 79.2 | 59.5 | 79.3 | 58.3 |
| fields and events | 66.0 | 49.1 | 67.6 | 49.2 |
| SPL | 54.3 | 48.2 | 55.8 | 47.2 |
| priority flood | 45.3 | 45.1 | 50.3 | 48.3 |
| marine fill | 40.8 | 35.5 | 42.6 | 35.0 |

The spread, advection and flexure phases have no pass of their own and gained most. They were,
by inference, mapping in again, page by page, the large blocks the session had given back before
each step (7fa875f1 keeps them; system time halved, 867 → 423 s and 920 → 446 s), and copying
grids on the driver (a85d5298). The plate stage's code is master's on both sides; its 1.11 to
1.14× is, by inference, the allocator (each thread's parked small blocks take kiddo's per-query
`Vec` off malloc and the shared count, edc3eae4; the counters on their own cache lines, 85cff4e2)
and the relink collecting in place. The climate, structure and volcano stages are the same code
on both sides; their 0.96 to 1.01 and the one-off stages' spread (deposits 0.71 to 1.07, amplify,
finalize) are the load. The two sets give the same ratio to within a per cent.

**The same pair on master 8f2b7e79**, before the merge (branch at a385e758): 2,005.0 → 1,784.5 s
(1.12×), CPU time 7,922 + 869 → 7,327 + 410 s, surface 926.1 → 733.0 s, 84 of 84 files equal.
Single runs on set A of that master: 1,889.1 and 1,874.2 s (others' load 11.1, siblings 35 and
38 % busy); of the branch, 1,849.7 s (others' load 16.5, siblings 64 % busy). Master's CPU time
moves with the load as much as with the code: 8,437 to 8,791 s on 8f2b7e79 at others' load 11 to
13, and on e34dbe54 8,635 s at 7.1 and 9,238 to 9,945 s at 15 to 16.

**Single runs on set A, quiet** (after the pairs, each started in a lull; seconds; master =
e34dbe54, branch = 13f49fd5; both worlds equal master's from the pairs, 84 of 84 files):

| stage | master `bq-1` | branch `fq-1` |
|---|---:|---:|
| plates | 424.4 | 325.7 |
| climate | 220.6 | 208.5 |
| surface | 782.1 | 529.8 |
| volcanoes | 29.0 | 27.3 |
| impacts | 3.7 | 3.8 |
| structure | 128.1 | 123.3 |
| deposits | 116.3 | 82.1 |
| young | 21.1 | 12.3 |
| amplify | 35.5 | 23.5 |
| finalize | 61.2 | 37.9 |
| **wall** | **1,828.6** | **1,382.3** |
| CPU time, user + system | 7,816 + 818 | 6,294 + 363 |
| the run's CPUs, mean | 4.72 | 4.82 |
| others' busy CPUs, mean | 7.1 (p90 16.4) | 2.4 (p90 10.1) |
| its cores' SMT siblings busy | 29 % | 10 % |
| peak, MiB (count) | 4,239 | 4,200 |

Both are under the "others' load under about 8" mark, but the branch's run was the quieter (its
cores' siblings 10 % busy against 29 %), so their 1.32× carries the load as well as the code;
the side-by-side 1.13× is the code's. A master run as quiet as the branch's would take about
1,560 s (1,382.3 × 1.13, inferred). A quiet pair side by side waited an hour for a third lull
and was not run.

**The passes** (each bit-identical; the gains are 100 Myr pairs on master 8f2b7e79's code, before
→ after, others' load 14 to 18; all of them together at 1000 Myr are the side-by-side table
above):

| commit | where | change | gain (100 Myr pair) | test |
|---|---|---|---|---|
| 85cff4e2, 585fce47 | surface / diffusion, marine; sphere / coupling; alloc | the diffusion solve skips rows with nothing to diffuse, and its conjugate gradient forms z only where it is used and defers x; the slope failure marks its neighbourhoods by rows in parallel; the allocator's counters each on their own cache line; the L1 relink collects in place | surface 82.3 → 78.5 s; wall 290.7 → 280.7 s; CPU 1,391 → 1,329 s | parity tests against the textbook solve; the slope failure's active-set tests |
| edc3eae4 | alloc, session, stage | where the session needs room it gives back only the kept large blocks it needs, oldest first, not all of them; each thread parks its last four freed small blocks (up to 4 KiB) for its next block of that size, so the k-d tree's per-query `Vec` no longer goes through malloc and the shared count | none to measure: wall 280.7 → 280.7 s, CPU 1,290 → 1,274 s | `history_cap`, `memory_estimate` |
| 7fa875f1 | alloc, session, stage | kept blocks give way by themselves, oldest first, when the count would pass the ceiling; the before-step check, the history's trim and a stage's reserve count live bytes only; 2 GiB of kept blocks in 2,048 slots (was 1 GiB). At 1 GiB full 16k pushed out 2,730 kept blocks (33 GB) in 30 Myr, to be mapped and faulted in again by the next step; at 2 GiB, 184 | surface 68.4 → 57.1 s; system time 95 → 52 s; wall 265.7 → 255.7 s | `history_cap`, `memory_estimate` |
| 8350ade2 | surface / glacial | the glacial step walks only the cells with ice, in stack order; its debris pass visits only the stack positions below an eroding cell, its last pass and clearing only the cells written; its working arrays stay between steps; its fills in parallel; the river mouths and the glacial sea loads found in parallel | not isolated (its only 100 Myr run saw others' load 26.8); in the final runs | the step with and without the forest (the old whole walks) at 1, 3 and 8 threads, bit for bit |
| a85d5298 | surface | the step's grid copies and fills on the pool's threads (`alloc::copied`, `alloc::filled`) instead of `clone`, `to_vec` and `vec!` on the driver: the height snapshots, the output fields, the kind and coal rasters, the empty ice and rate fields, the diffusion's right-hand side; the glacial rate map in parallel | in the final runs | the coupled pipeline, 1 against 3 threads |
| 6fd0d036, a385e758 | alloc, surface | memory, not speed (below) | | `memory_estimate` |

Tried and left out: transparent huge pages on the large blocks (`madvise(MADV_HUGEPAGE)`, a
scratch build): 361.4 s without, 356.5 s with, inside the two sets' difference, so not adopted.

**Memory.** The glacial step's working arrays (six grids) stay between steps, so the young stage,
which makes its own glacial output each step, carried them on: `memory_estimate`'s young row held
77.2 MiB against its 74.3 MiB estimate (master 71.8 to 73.0). The young stage now gives them back
after its glacial step, and the surface stage after its last step. The allocator's count also takes
its peak after kept blocks give way for a new block, not before, so kept blocks never lift the
counted peak over the ceiling. On the VM, after both: plates 50.7, climate 60.5, surface 56.8,
volcanoes 46.3, impacts 46.0, structure 55.1, deposits 59.3, young 72.3 (estimate 74.3), amplify
74.5, run 96.2 (estimate 97.5; master 96.2) MiB held; resident peak 99.7 to 101.1 MiB against the
121.8 MiB bound (a385e758, before the merge). `history_cap` and `session_pool` pass; after the
merge, `memory_estimate` passes again in the workspace tests.

**Why 900 s is out of reach without changing results** (measured, and inferred where marked):

- **The work fits; the serial part does not.** Quiet, the branch's run is 6,657 s of CPU (user
  + system). On 8 cores busy for the whole run that would be 832 s, under the gate. The run keeps
  4.8 of them busy on average (4.1 to 4.6 under load, where the busy SMT siblings also slow each
  core: the same run's CPU time is 8,068 to 8,834 s side by side). With the run split into a part
  on one core (S) and a part spread over all 8, the quiet 1,382 s gives S ≈ 630 s (inferred), and
  900 s needs S under about 80 s with the work unchanged.
- **Where the serial part is.** A `perf` profile at 200 Myr (7fa875f1, before the merge, under
  load) splits the driver thread's samples into time alone and time beside the workers: alone,
  surface 60.2 s, plates 45.6 s, climate 18.9 s, structure 6.9 s, bake 4.5 s, deposits 3.1 s, so
  about 650 s per 1000 Myr (inferred, agreeing with the fit above). The largest serial items: the
  plate mesh's hull (`SphereMesh::insert` and `orient`, 8.2 s per 200 Myr; `plates.resample` is
  39.2 s in the quiet run), `apply_surface_feedback` and `sample_flexure`'s scatter-adds and the
  `Group` index (7.1 s per 200 Myr; `plates.feedback` 16.3 s and `plates.flexure_sample` 7.3 s
  quiet), the priority flood (`surface.pflood` 33.7 s quiet, serial by design), the
  amplification's `flood_repair`, and the zonal climate clock (`climate.clock` 29.4 s quiet,
  serial in time). The flood and the clock alone are about 63 s of the 80 s the gate leaves. This
  branch took the surface's serial copies and fills (`memset` 5.5 s and `memmove` 2.4 s per
  200 Myr in that profile) and the glacial step's whole-grid walks.
- **Left to the files' owners** (as in the table below): the plate step's serial items are in
  `plates/` and `sphere/hull.rs`. `plates/` was plate task 8's while this branch ran (it has been
  merged since), so the task's rule kept this branch's edits there to none; the L1 relink's
  nearest-first query and kiddo's per-query `Vec` are left likewise.
- **What it would take** (inferred). The bit-identical passes left (the serial plate items
  above, the craton-edge sweep by rows, the relink's nearest-first query) could take perhaps 100
  to 150 s of the serial part, which would bring the quiet run to about 1,250 to 1,300 s, still
  1.4× the gate. Getting under 900 s needs either less work per step, at 4.8 busy cores about
  4,300 s of CPU (a third less), which changes the worlds and is Ben's call (fewer slope-failure
  iterations or a convergence stop, a diffusion solve with a stronger preconditioner than its
  Jacobi one or multigrid, a coarser L1 cadence (`l1_every_steps`), a longer plate time step:
  P1-24 measured 4-Myr steps at 1.6×), or the stages of one step overlapping so the flood, the
  clock and the hull run beside parallel work (the climate or the structure of step k beside the
  plates of step k + 1), which is a redesign.

## P1-19 exit item 1: the stages of one step side by side (design, 2026-10-18)

Ben's decision of 2026-10-18: overlap the stages of one step. The aim is the same 900 s gate
(full 16k, scattered continents, 1000 Myr, 8 threads), with the worlds unchanged byte for byte.
Branch `worktree-agent-a429a6a924e26ae78` from master 0729e267. This section was written before
the implementation. The measured results come at its end. Statements are marked as on the rest of
this page; **estimated** marks a gain worked out before it was measured.

**The work to hide** (measured; the quiet branch run `fq-1` above and a 200 Myr run of the same
preset with each stage's process CPU time counted, `scratch1`, under load):

| stage | wall, quiet 1000 Myr (s) | wall, 200 Myr (s) | CPU, 200 Myr (s) | busy cores |
|---|---:|---:|---:|---:|
| plates | 325.7 | 64.9 | 249.6 | 3.84 |
| climate | 208.5 | 58.4 | 276.4 | 4.74 |
| surface | 529.8 | 98.7 | 560.4 | 5.68 |
| volcanoes | 27.3 | 4.5 | 14.3 | 3.14 |
| impacts | 3.8 | 0.6 | 1.1 | 1.75 |
| structure | 123.3 | 25.9 | 119.7 | 4.62 |
| deposits | 82.1 | 17.5 | 98.6 | 5.63 |
| young | 12.3 | 11.4 | 56.8 | 4.98 |
| amplify | 23.5 | 23.2 | 128.3 | 5.52 |
| keyframes and the history's trim (after each step) | | 4.2 | 4.2 | 1.00 |
| finalize (the bake) | 37.9 | | | |

### The dependency map (read)

The session runs the coupled group of a plate step in this order: plates (the lead), climate,
surface, volcanoes, impacts, structure. Then come the stages after the group (deposits, young,
amplify) and the bake. What each step of the group reads and writes:

| step | reads | writes |
|---|---|---|
| plates k + 1 | its plate model; the globals; the surface's feedback fields, which it takes out of the world (`surface/volcanic_felsic_m` and `_mafic_m`, `surface/feedback_m`, the surface's material `surface/material_m`, the gap budget's `surface/gap_*` parts); the flexure `surface/flexure_m` | the plate model; `world.t_years`; every L1 field anew each step (`regime`, `plates/*`, the pool fields); `surface/feedback_applied_m`; events (appended); notes |
| climate k + 1 | the L1 fields; `crust_age`; the surface's elevation (`world.surface`); the crust's plate and group the surface stage left; the new events; its own last fields | `climate/*`; the sea level and cap carbonate globals |
| surface k + 1 | the L1 fields; `climate/*`; the whole surface state (elevation, the strata log, ice) | the surface state and its strata log; `surface/*` fields, the feedback the next plate step takes among them; the recorded history's frames (every 10 Myr); events |
| volcanoes, impacts k + 1 | the surface state; the L1 fields; events | the surface and its strata log (edifices, craters); `surface/volcanic_*` (which the next plate step takes); the trap field; events |
| structure k | the strata log; 14 L1 fields; `climate/t_mean_c`; the surface's deposition, erosion, material and flexure; the impacts' trap field; the pool fields; the plate model's time, rotations and parents; the events (its fold and fault causes) | the strata log (metamorphism, plutons, folds and faults); its fields in the history's new frames; on keyframe steps its keyframe layers; on its last step the bodies, its events and its note |

The edges that order a step after another:

- plates k + 1 after surface, volcanoes and impacts k: it takes their feedback fields and samples
  their flexure.
- climate k + 1 after plates k + 1 (the L1 fields), surface k + 1 after climate k + 1 (its
  fields), volcanoes, impacts and structure k + 1 after surface k + 1 (the strata log), structure
  after volcanoes and impacts (their edifices and craters are in the log it reads).
- surface k + 1 after structure k: both change the strata log.
- **Nothing in step k + 1 before the surface reads what structure k writes.** The plate step
  reads neither the surface state nor the history, and it writes nothing structure k reads
  except fields it writes anew (the L1 fields) and two it takes or reads (the material, the
  flexure). Structure k's events, bodies and note come only on its last step; events are
  append-only, so structure k's causes are the events up to the end of step k.
- The deposits stage reads the whole recorded history (setup reads the frame count; each frame
  is folded in in order); young and amplify read the last surface; the bake reads everything.

### What can overlap

- **A. Structure k beside plates k + 1.** The only edge that leaves a step's sink free. Detaching
  the structure step's inputs is enough: the L1 fields and the fields the plate step does not
  touch move out of the world, the material and the flexure are copied (the plate step takes or
  reads them), the strata log moves out with the surface state (the plate step never reads it),
  and the events seen are fixed before the plate step appends to them. The structure step then
  runs in a `rayon::join` beside the plate step, in the session's pool, and its fields go into the
  history's frames after the join, as they would have before the plate step. Keyframe steps
  (their layers are read by the keyframe) and the last step (bodies, events, the note) run on the
  world as before. Anything that reads the world between `advance` calls (a keyframe, an epoch
  checkpoint, a checkpoint, `with_world`) runs a put-off step on the world first.
- **Not climate k + 1 beside structure k** (only a little more): climate k + 1 needs the surface's
  elevation (structure k would have to leave it in the world) and rewrites `climate/t_mean_c`,
  which structure k reads (a copy). It is possible, but the structure step is already shorter
  than the plate step it hides behind (123 against 326 s), so carrying it on into the climate
  step gains little.
- **Not the chain plates, climate, surface, volcanoes, impacts:** each needs the step before it
  (above). Splitting one stage's step to start part of it early (the climate clock, the priority
  flood) is work inside `climate/` and `surface/`, not between stages.
- **B. Deposits beside young and amplify** (not done). Neither reads the other's output, so the
  deposit stage's 82 s and the 36 s of young and amplify could run together. They are separate
  stages outside a group, so the session needs a second way to run stages side by side, and
  their peaks add: deposits' accumulators beside the amplification's tiles, the run's peak today.
  Both are mostly parallel (5.6 and 5.0 to 5.5 busy cores), so the gain is the one-core parts:
  **10 to 20 s (estimated)**.
- **C. A keyframe drawn beside the next plate step** (not done). The 20 keyframes at full 16k
  (every 50 Myr) and the history's trim take about 4 s of one core per 200 Myr, so about 20 s per
  1000 Myr, half of it the keyframes. Drawing a keyframe beside the next step needs a frozen copy
  of the layers it draws (a few hundred MiB at full 16k). **About 10 s (estimated).**
- **D. The deposit stage folding in each frame beside the coupled group** (not done; not one step
  but one stage beside another, so outside this task). Its ingest reads frame k once the
  structure stage has published into it, in order, so it could keep pace with the group on the
  cores the group leaves idle (about 3 of 8). It needs the setup split (the frame count is known
  only at the end), the ingest's writes to the world held until its stage, and its accumulators
  (each type's `CellAcc` per cell) held for the whole group instead of only at its end, at full
  16k several hundred MiB on top of the group's peak. **Up to most of the 82 s (estimated, the
  ingest's share not measured).**
- **The bake** reads everything: it can overlap nothing of the run.

### The expected gain of A (estimated)

Split each stage's wall into a one-core part S and a part spread over all 8 threads, from its wall
and CPU time (S + P/8 = wall, S + P = CPU): over 1000 Myr the structure stage is about 60 s on
one core and 510 core-seconds spread; the plate step about 190 s on one core, when 7 of the 8
threads are idle. Beside the plate step, the structure step's one-core part is hidden as long as
the plate step lasts, and its spread part runs on the threads the plate step leaves idle or
shares the threads with the plate step's own spread part. The bounds are 60 s (only the one-core
part hidden) and 123 s (all of it). Rayon's stealing lets a plate thread that waits in a join pick
up structure work, which lengthens the plate step's own one-core path, so the lower end is the
likelier: **45 to 65 s per 1000 Myr (estimated)**, the quiet run from 1,382 s to about 1,315 to
1,340 s. The 20 keyframe steps (every 50 Myr, the last step among them) run on the world as
before, so 480 of the 500 steps run beside the plates.

Its cost: the session (the put-off step, the join, the `Stage` methods for detaching) and the
structure stage (its step split from its outputs; its fold and fault causes cached instead of
read from the world in the step; no change to what it computes). B, C and D gain less per effort
or are outside one step, so A is the one implemented here.

### The memory cost of A (read; inferred where marked)

- **The working sets add.** The structure step's working set (its reservations: the metamorphism
  and pluton scratch, and on the 1 in 5 steps that publish, the frame fields, about 172 MiB at
  full 16k) now coincides with the plate step's. The estimate counts it: a coupled group's
  stages each hold the group's most, and the most now also takes the lead's set plus the
  follower's own (`stage_sets`). The memory test keys its phase peaks by stage name and the join
  runs under the plate step's phase, so the plate row is the one that holds both.
- **Copies:** the material and the flexure, 2 × 4 bytes per L1 cell (about 11 MiB at full 16k).
- **Held longer:** the frame fields stay with the detached step until the join ends (one step's
  worth, at most about 180 MiB at full 16k); the old L1 fields stay with it until the plate step
  has written the new ones, which is when the plate step would have dropped them anyway.
- **Frames in flight:** none more. The history gets the structure fields of the frames the
  surface recorded before the plate step, after it, and its trim runs after the join.
- **A refused reservation:** the structure step reserves first, so a refusal beside the plate
  step changes nothing, and the session runs it again after the join, when the plate step's
  working set is gone.
- At full 16k the history is mostly on disk (14.8 MiB resident at the end), so the extra working
  set comes out of the run's headroom under its cap, not out of the history's frames (inferred).

### Measured (A, 2026-10-18)

Method as in the section above: `tools/review/bench-quiet.py`, full 16k, scattered continents,
1000 Myr, 8 threads; set A = CPUs 0, 16, 2, 18, 4, 20, 6, 22 and set B = 8, 24, 10, 26, 12, 28,
14, 30. Each pair was one admitted job (`sim:earthlike_full_16k:16t`), the quiet run one
`sim:earthlike_full_16k:8t` job started once the 1-minute load had stayed under 6 for three
minutes. Master = 991354a4 (the polar-holes flexure fix), branch = cb270657, its merge of that
master. Every branch world equals master's, **84 of 84 files** in both pairs and in the quiet run
(`tools/review/p1-17-cmp.py`; the quiet run against the second pair's master world).

| stage (s) | master (A) | branch (B) | master (B) | branch (A) | branch (A), quiet |
|---|---:|---:|---:|---:|---:|
| plates (the structure steps beside it included on the branch) | 487.7 | 532.2 | 354.6 | 373.6 | 321.9 |
| climate | 270.5 | 277.1 | 216.0 | 212.9 | 197.5 |
| surface | 747.4 | 768.1 | 572.6 | 575.7 | 515.7 |
| volcanoes | 40.0 | 41.1 | 30.3 | 30.2 | 28.4 |
| impacts | 4.3 | 4.3 | 3.4 | 3.4 | 3.3 |
| structure (on the world) | 150.1 | 56.6 | 129.0 | 50.1 | 47.6 |
| structure beside the plates | | 101 | | 79 | 74 |
| deposits | 101.6 | 101.5 | 100.7 | 102.9 | 97.0 |
| young | 13.2 | 13.7 | 12.9 | 12.7 | 12.8 |
| amplify | 25.4 | 26.0 | 24.5 | 24.5 | 24.2 |
| finalize | 40.6 | 41.3 | 39.6 | 42.3 | 39.4 |
| **total** | **1,883.5** | **1,864.7** | **1,485.8** | **1,430.5** | **1,290.1** |
| × master | | **1.010** | | **1.039** | |
| plates + structure | 637.8 | 588.8 | 483.6 | 423.7 | 369.5 |
| CPU time, user + system | 8,154 + 441 | 8,385 + 448 | 6,725 + 376 | 6,750 + 367 | 6,111 + 337 |
| the run's CPUs, mean | 4.55 | 4.72 | 4.76 | 4.96 | 4.98 |
| others' busy CPUs, mean | 14.6 | 14.6 | 4.8 | 4.8 | 0.0 |
| its cores' SMT siblings busy | 51 % | 59 % | 0 % | 0 % | 0 % |
| peak, MiB (count) | 4,198 | 4,198 | 4,198 | 4,198 | 4,198 |
| peak resident (`time -v`), MiB | 4,245 | 4,529 | 4,239 | 4,521 | 4,495 |

- **The gain is the estimate's** (measured): the plate and structure stages together take 49 s
  (under load) and 60 s (others' load 4.8) less, against the estimated 45 to 65 s. Of master's
  129 to 150 s structure stage, the 480 steps beside the plates took 74 to 101 s; the 20 steps
  still taken on the world (the keyframe steps with their layers, and the last with its folds,
  faults and bodies) take 48 to 57 s.
- **The run:** 1.039× at others' load 4.8 (1,485.8 → 1,430.5 s) and 1.010× under load (1,883.5
  → 1,864.7 s, the branch on the set whose siblings were the busier). Under load the cores the
  plate step leaves idle are fewer, and the stages other than plates and structure moved by up
  to 21 s between the sets, more than the gain. The pair at the lower load is the better
  measure of the code.
- **Quiet: 1,290.1 s** (process wall 1,292.3 s; others' load 0.0, its siblings 0 % busy; CPU
  6,448 s, 4.98 cores busy). The quiet run of the section above (1,382.3 s) was on older code
  and a little less quiet, so the 92 s between them is not all this change; a master run as quiet
  would take about 1,340 s (1,290.1 × 1.039, inferred).
- **Memory** (measured): the run's estimate is unchanged (peak 4.86 GiB; the coupled group's
  4.17 GiB, master's too, is one member's own set, above the plate step's plus the
  structure's), and so is the counted peak (4,198 MiB). The peak resident set rose by about 280 MiB (4,239 to 4,245 → 4,495
  to 4,529 MiB, 6.6 %), still under the estimate; by inference the allocator's arenas, since the
  two stages now allocate on the pool's threads at the same time. On the memory test's small
  planet the resident peak was 101.0 MiB against its 121.1 MiB bound, and every stage held less
  than its estimate (plates 51.2 against 72.8 MiB); `memory_estimate`, `history_cap` and
  `session_pool` pass.
- **What is left to 900 s:** 390 s quiet (1.43× the gate). B, C and D together would take, at
  their estimates, at most about 110 s, so the gate still needs less work per step (fewer
  slope-failure iterations, a stronger diffusion preconditioner, a coarser L1 cadence or a longer
  plate step, all of which change the worlds and are Ben's call) or the one-core items left to
  `plates/`, `surface/` and `climate/` (the hull, the feedback scatter, the priority flood, the
  climate clock; about 100 to 150 s, inferred above).

## P1-19 exit item 1: where the full 16k run's time goes (profile, 2026-10-18)

Ben asked for every system that takes part in the gate run and what each one costs ("I am almost
positive you are missing out on some optimizations, please make a space sheep showing me all
systems which are interacting with this system and contributing time"). The page is
`review/pages/sim-time/` (title "Full 16k Run Profile"; the coordinator publishes it). Branch
`worktree-agent-a6302ca9dc7ca54f4` from master 25ae6723; no simulation code changed. All measured
unless marked.

**Method.**

- `tools/review/sim-time-run.py` (new) runs `deeptime bench --preset earthlike_full_16k --start
  scattered_continents --length-myr 1000 --threads 8` under `taskset` on set A (CPUs 0, 16, 2, 18,
  4, 20, 6, 22), after everyone else's busy CPUs have stayed under 1.5 for 30 to 120 s. It samples
  `/proc/stat` and every thread's `/proc/<pid>/task/*/stat` every 0.5 s. Others' busy CPUs
  averaged 0.04 to 0.16 over every run.
- Two full runs of master 25ae6723: the plain release build (the gate's) for the wall time, and a
  build with frame pointers and span timers (`tools/review/sim-time-trace.patch`, an experiment
  file, not applied to master) under `perf record -F 199 -g -k CLOCK_MONOTONIC` (cpu-clock; the
  VM has no hardware counters), decoded as root with `perf script -f --no-inline`.
  `tools/review/sim-time-analyse.py` (new) charges every sample to the deepest span running on
  its thread, sorts it into work, kernel and rayon's wait loop, and writes the page's `data.js`.
- Thread scaling: the same preset for 200 Myr on the traced build at 1, 2, 4, 8 and 16 threads,
  fitted with T(n) = S + P / n over 1 to 8; the 16-thread run (sets A and B) checks the fit.
- Build settings: plain, fat LTO with one codegen unit, and `-C target-cpu=x86-64-v3`, 100 Myr
  each, twice, alternating; worlds compared with `tools/review/p1-17-cmp.py`.
- Four Opus reviewers read the plates and structure, surface and volcanoes, climate, and the
  stages after the group with the session's machinery, given the measured times, for changes that
  keep the worlds byte-identical. No change was made.

| stage (s) | plain | profiled |
|---|---:|---:|
| plates (the structure steps beside it included) | 333.6 | 338.8 |
| climate | 199.2 | 204.7 |
| surface | 476.1 | 480.8 |
| volcanoes | 24.9 | 24.8 |
| impacts | 3.4 | 3.5 |
| structure (on the world) | 50.1 | 51.2 |
| deposits | 80.4 | 82.1 |
| young | 12.3 | 12.8 |
| amplify | 23.8 | 24.5 |
| finalize | 36.8 | 38.2 |
| **total** | **1,242.8** | **1,263.7** |
| CPU time, user + system | 5,955 + 326 | 6,048 + 338 |
| the run's CPUs, mean | 5.05 | 5.03 |
| minor page faults | 68.5 million | 68.4 million |
| peak resident (`time -v`), MiB | 4,531 | 4,525 |

The profiled world equals the plain one, 84 of 84 files. The quiet 1,290.1 s above was cb270657;
today's master is 47.3 s faster.

**What the profile shows.**

- **Half of the 8-thread wall is serial.** The 200 Myr runs fit T(n) = 156 s + 1,215 s / n within
  1 % (1,371.9, 759.8, 460.0, 308.7 s at 1, 2, 4, 8 threads); 16 threads take 250.6 s, 19 s more
  than the fit. The run keeps 5.05 of 8 cores busy.
- **243 s of the critical path runs on about one core:** the priority flood (32.7 s), the plate
  transition (26.2 s), fronts (21.8 s, 1.5 cores), marine fill (16.8 s), the keyframe render
  (16.1 s), belts (15.4 s), record (12.6 s), climate's untimed code (11.7 s), collision (11.6 s),
  collapse (9.2 s), the volcano build (8.9 s), the plates' flexure sample (7.1 s) and smaller spans.
- **The heavy kernels already scale:** the diffusion solve keeps 7.6 cores busy (109.3 s), the L1
  write 7.4 (95.8 s), slope failure 7.1 (83.8 s), the climate snapshot's cells 7.3 (46.1 s). Only
  less work per call helps them.
- **The plate step is the critical arm.** 316.1 s over 479 steps; the structure step beside it
  (77.0 s) always finishes first (0.02 s of waiting in all). Fifteen of its 22 laps keep two cores
  or fewer busy: 178.5 s.
- **Kernel time is 313 s of CPU:** futex wake-ups 150 s (66 s of it in the 2-D energy balance),
  page faults 121 s (31 s in surface advect, which collects a fresh 515 MB strata log each step),
  munmap 23 s (11 s on one core), file writes 1.8 s. On the critical path: about 25 s of faults,
  16 s of munmap and 24 s of wake-ups.
- **The 2-D energy balance is slower on 8 threads than on 4** (7.0 s against 6.0 s at 200 Myr;
  10.5 s on 1): 864 short steps per snapshot, 105 s of work against 66 s of wake-ups and 36 s of
  spinning.
- **The same horizons are baked three times** (the structure stage's last step, deposits, the
  bake's strata pass), about 100 s of CPU each in `Horizons::smoothed`.
- **Build settings, measured** (100 Myr, two runs each): plain 181.0 and 180.6 s, fat LTO 179.2
  and 176.6 s (−1.6 %), x86-64-v3 173.1 and 172.1 s (−4.5 %); every world equals plain's, 66 of 66
  files. Rust does not contract a multiply and an add into an FMA by itself, so v3 changes no
  value. Not measured together.

**Ranked opportunities** (gains in s on the 1,242.8 s plain run; estimated, from the profile and
the code read, except the two build rows; "yes" = keeps the worlds byte-identical by construction,
still to be proven with `p1-17-cmp.py`; "check" = probably, to be confirmed first; "no" = changes
the worlds, Ben's call):

| # | change | systems | gain (s) | identical | effort |
|---:|---|---|---:|---|---|
| 1 | build for x86-64-v3 (measured) | all | 45 to 60 | yes | small; the CPUs it needs are Ben's call |
| 2 | plate transition, record, collision, collapse in parallel | plates | 27 to 37 | yes | large |
| 3 | plate feedback, flexure sample, belts, fronts | plates | 25 to 33 | yes | medium |
| 4 | priority flood beside advect and setup (`rayon::join`) | surface | 20 to 27 | yes | medium |
| 5 | cheaper k-d tree queries in the L1 write, detect, margins | plates | 15 to 30 | check | medium |
| 6 | fat LTO, one codegen unit (measured) | all | 15 to 25 | yes | trivial |
| 7 | fewer page faults and unmaps (persistent strata log, block size classes) | surface, structure, memory, climate | 10 to 25 | yes | medium |
| 8 | deposits: serial parts in parallel, frames decoded once | deposits | 10 to 20 | yes | medium |
| 9 | diffusion solve and slope failure: less memory traffic | surface | 10 to 20 | yes | medium |
| 10 | 2-D energy balance in larger pieces | climate | 10 to 18 | yes | small |
| 11 | bake the horizons once, not three times | structure, deposits, finalize | 12 to 28 | check | medium |
| 12 | keyframe render in parallel | keyframes | 11 to 14 | yes | small |
| 13 | plate resample: the hull beside the rest | plates | 10 to 15 | yes | medium |
| 14 | climate snapshot: tables per row, serial pieces overlapped | climate | 8 to 15 | yes | medium |
| 15 | advection sources: one rotation per plate | surface, climate | 8 to 14 | yes | small |
| 16 | climate ice: weathering loops in parallel | climate | 8 to 13 | yes | small |
| 17 | fields and events: zeroed grids, tag lookups | surface | 6 to 12 | yes | small |
| 18 | climate clock and untimed climate code | climate | 6 to 12 | yes | small |
| 19 | the structure step beside the plates on its own threads | plates, structure | 0 to 15 | yes | small |
| 20 | glacial and surface flexure in larger pieces | surface | 4 to 10 | yes | medium |
| 21 | finalize: parallel tile packs and layer writes | finalize | 4 to 8 | yes | small |
| 22 | volcanoes: render the cones in parallel first | volcanoes | 5 to 7 | check | small |
| 23 | history spill: stop reading frames back | memory, keyframes | 3 to 6 | yes | medium |
| 24 | profile-guided optimisation (not built) | all | 35 to 100 | check | medium |
| 25 | multigrid or stronger preconditioner for diffusion | surface, young | 40 to 80 | no | large |
| 26 | stop slope failure early | surface | 20 to 50 | no | small |
| 27 | coarser L1 cadence or longer plate step | plates, structure | 50 to 150 | no | medium |
| 28 | order-independent parallel flood | surface, young | 25 to 30 | no | large |

The "yes" rows add up to **240 to 399 s** (estimated) against the 342.8 s between the run and the
gate; the "check" rows add 67 to 165 s more (PGO included). The sum overstates: the build
percentages apply to a shorter run once the code changes land (4.5 % of 900 s is 41 s), and a
one-core span parallelised helps less where the structure step beside the plates already fills
the idle cores. **So 900 s with the worlds unchanged looks reachable (inferred), but only if most
of the "yes" rows land.**

**Not measured:** every gain except the two build rows; hardware counters (none on the VM); the
plate laps' own time apart from structure work their threads steal (needs a run with the overlap
off); PGO, and fat LTO with v3 together; an allocator swap (the counting allocator owns the memory
cap); time inside closures inlined into rayon's plumbing (442 s of CPU stay unattributed); sub-steps
of deposits and amplify (no timers; perf only); scaling at full length (200 Myr runs; the
after-group stages are 27.5 % of the 200 Myr run against 12.5 % of the full run); the player's PC.

**Scratch.** The perf data, the run folders and the trace binaries were deleted on the VM; the
trace patch stays as `tools/review/sim-time-trace.patch` to repeat the profile (`git apply`, then
build with `RUSTFLAGS="-C force-frame-pointers=yes"`).

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
So the head's worlds at that merge were `v7a`'s (read).

The second merge of master (5bc63aa5, 2026-10-17) brings plate task 7, the erosion-by-process
notes and new preset values, which change the worlds on both sides alike. Its two conflicts in
code kept master's work and the branch's: the plate step's phase timers around master's new
crust-budget sums, and the surface step's parallel per-cell loops with master's new arrays of
erosion by process. So quick scattered was run once more, one run at a time, against that
master (measured, 8 threads, 1000 Myr, `out9`, labels `m5bc` and `v8h`):

| | master 5bc63aa5 | branch f4f9a7be | × |
|---|---:|---:|---:|
| total, s | 761.3 | **574.0** | 1.33 |
| climate | 168.8 | 117.8 | 1.43 |
| surface | 263.9 | 192.0 | 1.37 |
| structure | 64.2 | 40.4 | 1.59 |
| plates | 123.7 | 137.7 | 0.90 |
| user s | 3,214.8 | 2,532.0 | 1.27 |
| cpu % | 446 | 479 | |

84 files, 83 the same byte for byte. The one that differs is the manifest, in one note:
`notes.structure.metamorphosed_km3` is 59,837,472.25688099 on master and 59,837,472.256881006 on
the branch, two units in the last place. That is the cell-order sum (master adds the columns in
parallel, in no fixed order), not a different world. The plate stage is the same code on both
sides apart from the branch's timers, a lock and a clock read per part per step; its 0.90 is the
shared VM's load on a mostly serial stage (it was 1.12 the other way in `out7`). Full was not
run again.

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
done here. (Done later on 2026-10-17, branch worktree-agent-a6726a524e4c6fb75:
`alloc::release_cache` now also calls `malloc_trim(0)`. On master 1aeb7e4e the test failed 3 of
5 on a busy VM, with resident peaks of 111.2 to 124.9 MiB against a bound of 116.7. With the trim
it passed 5 of 5, at 97.1 to 99.7 MiB. See STATUS.md.)

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
