# Faster Create World (P1-24): measurements and options

Status: **options for Ben** (2026-10-04). Nothing here changes the shipped game. Ben chooses which
option to build (plan §10, P1-24: "Measure player-side generation on a mid-range PC profile, then
choose (Ben) among a faster quick mode, shipped pre-made planets and background generation, and
build it"). The decisions log of 2026-10-01 still holds: features land first, optimisation passes
(CPU first) come after, and there is no speed gate yet.

Statements are marked **measured** (a run made for this note), **read** (from the code or a
document) or **inferred**.

## Summary

- **Today, on a mid-range PC** (6 cores, 16 GB; section 1): a quick 16k planet takes **about 6
  minutes for 500 Myr and 10 minutes for 1,000 Myr**, peak 4.2–5.5 GiB. Both starts cost the same.
  The coupled stage (plates, climate, surface, volcanoes, structure) is 80–90 % of it.
- **The full preset does not fit a 16 GB PC**: 7.7–8.9 GiB at 500 Myr, 9.5 GiB at 1,000 Myr; it was
  killed for lack of memory twice on the 6 GiB profile. Given the memory, it takes 21–22 minutes
  (500 Myr) and 42 minutes (1,000 Myr).
- **The memory cap under-counts**: the run's own accounting stays under its 4 GiB cap while the
  process holds 1.5 GiB (quick) to 5 GiB (full) more, so the editor's estimate, refusal and
  low-memory warning cannot be trusted. This needs fixing whatever Ben chooses.
- **Options:**
  - (a) an "express" preset family: 3.0× faster (115 s), with rounder coasts and fewer islands;
    4-Myr steps alone give 1.6× and look like today's planets;
  - (b) shipped ready planets: instant, ~45 MB each, but fixed and re-baked every release;
  - (c) a background planet pool: instant from the second world on, 13.5 minutes of 2 low-priority
    threads to refill;
  - (e) optimisation passes: no quality cost, an uncertain gain.
- **Recommendation: (a) + (c2)**, that is express now (2–3 days) and the planet pool next (4–6
  days), plus the memory-accounting fix. Pre-made planets are not recommended: they give up the
  player's own planet and add upkeep each release. Optimisation stays the planned later pass.

## 1. What a player waits for today

### 1.1 Method

The player's PC is a cgroup scope on the test VM (`tools/review/p1-24-profile.sh`):

| Profile | CPUs | Rayon threads | Memory | Stands for |
|---|---|---|---|---|
| `mid6` | 6 cores / 12 threads | 11 (the game's `MachineInfo.clientThreads()`: processors − 1) | 6 GiB | a mid-range gaming PC with 16 GB: the OS and a Minecraft client (MIC-sized heap) take the other ~10 GB |
| `low4` | 4 cores / 8 threads | 7 | 6 GiB | an older desktop or a gaming laptop |
| `bg2` | 2 cores / 4 threads | 2, at nice 19 | 6 GiB | a background run beside the game on a `mid6` PC (option c) |

- `AllowedCPUs` pins the run to whole cores (both SMT siblings), `MemoryMax` with no swap is the
  memory, and the run gets the game's default native cap (`--mem-cap` 4 GiB, `EditorModel.DEFAULT_CAP_GIB`).
- Each run records wall time, CPU time, per-stage seconds (the run's own `timings` line), its
  resident memory every 2 s (`mem.log`), the scope's peak, how long its threads waited for a CPU
  (`cpu.pressure`) and the VM's load.
- **The box is shared.** The VM (AMD EPYC, D32as_v7) ran other agents' jobs beside these, at a mean load of
  21 to 41 on 32 threads. The pinned CPUs keep that out of the run mostly: the threads waited
  9 to 30 s for a CPU in a ~350 s quick run, 80 to 91 s in a ~1,300 s full run. Memory bandwidth and caches are still shared, so these
  times are a little pessimistic.
- **A desktop is faster per core.** A current desktop core (e.g. a Ryzen 5 7600 or Core i5-13400 at
  4.6 to 5.1 GHz) is roughly 1.2 to 1.4 times the VM's per-core speed (inferred, from the clock
  ratio; not measured). Divide the times below by that for a fast desktop; an older 6-core PC is
  about the VM.

Commands (from the repo root, through `tools/remote/mac.sh`):

```
tools/remote/detach.sh start p124-matrix --kind sim:earthlike_quick_16k:11t \
    tools/review/p1-24-matrix.sh /Volumes/Graphide/ban/deeptime-runs/p1-24 all   # timing + quality
tools/remote/detach.sh start p124-mem --kind sim:earthlike_full_16k:11t \
    tools/review/p1-24-matrix.sh /Volumes/Graphide/ban/deeptime-runs/p1-24 mem   # per-phase memory, full on 16 GiB
tools/remote/detach.sh start p124-bg --kind sim:earthlike_quick_16k:2t \
    tools/review/p1-24-matrix.sh /Volumes/Graphide/ban/deeptime-runs/p1-24 bg    # background profile
tools/review/p1-24-views.sh <runs> <out> base=t-mid6-quick-scattered_continents-500 l1r32=a-l1r32 ...
```

### 1.2 Results

**Measured** (`python3 tools/review/p1-24-summary.py /Volumes/Graphide/ban/deeptime-runs/p1-24`;
seconds are wall time on the pinned CPUs; "coupled" is the one stage in which plates, climate,
surface, volcanoes, impacts and structure step together, split by the run's own timers; the bake
includes writing the folder):

| Preset, start, length | Profile | Total | Coupled (plates / climate / surface / volcanoes / structure) | Deposits | Young + amplify | Bake | Peak memory | Folder |
|---|---|---|---|---|---|---|---|---|
| quick, scattered, 500 Myr | mid6 | **349 s** (5.8 min); again 372 s | 285 s (41 / 80 / 119 / 15 / 29) | 18 s | 6 + 23 s | 16 s | 4.2–4.4 GiB | 130 MB |
| quick, supercontinent, 500 Myr | mid6 | **356 s** | 298 s (47 / 84 / 122 / 15 / 29) | 16 s | 5 + 21 s | 15 s | 4.6 GiB | 134 MB |
| quick, scattered, 1000 Myr | mid6 | **608 s** (10.1 min) | 546 s (91 / 132 / 240 / 32 / 50) | 23 s | 5 + 18 s | 15 s | 5.5 GiB | 196 MB |
| quick, supercontinent, 1000 Myr | mid6 | **609 s** | 545 s (89 / 135 / 239 / 31 / 50) | 24 s | 5 + 19 s | 16 s | 5.0 GiB | 190 MB |
| quick, scattered, 500 Myr | low4 | **418 s** (7.0 min) | 331 s (50 / 88 / 140 / 15 / 38) | 25 s | 7 + 32 s | 23 s | 3.9 GiB | 130 MB |
| quick, scattered, 500 Myr | bg2 | **812 s** (13.5 min) | 630 s (108 / 141 / 296 / 17 / 68) | 45 s | 11 + 64 s | 63 s | 3.7 GiB | 130 MB |
| full, scattered, 500 Myr | mid6 (6 GiB) | **killed** after 494 s: out of memory in the coupled stage | | | | | > 6 GiB | |
| full, supercontinent, 500 Myr | mid6 (6 GiB) | **killed** after 924 s: out of memory in the coupled stage | | | | | > 6 GiB | |
| full, scattered, 500 Myr | mid6 on 16 GiB | **1,236 s** (20.6 min) | 1,063 s (179 / 174 / 545 / 60 / 104) | 70 s | 25 + 31 s | 45 s | **8.9 GiB** | 204 MB |
| full, supercontinent, 500 Myr | mid6 on 16 GiB | **1,325 s** (22.1 min) | 1,119 s (196 / 175 / 566 / 63 / 116) | 81 s | 30 + 34 s | 59 s | 7.7 GiB | 222 MB |
| full, scattered, 1000 Myr | mid6 on 16 GiB | **2,548 s** (42.5 min) | 2,286 s (423 / 342 / 1,190 / 127 / 199) | 148 s | 27 + 28 s | 57 s | **9.5 GiB** (1.4 GiB of history spilled to disk as well) | 285 MB |

Peak memory is the run's peak resident set. Sampled every 2 s, the quick 500-Myr run peaks at
2.4 GiB in the coupled stage, 2.5 GiB in deposits and **4.2 GiB in young + amplify + bake** (the
amplifier holds the whole 8192 × 2624 terrain level); the full run at 7.6 GiB in the coupled stage
and 8.8 GiB at the end; the full 1,000-Myr run at 8.6 GiB in the coupled stage and 9.3 GiB at the
end, while its accounting said 3.4 GiB and pushed 1.4 GiB of history to disk.

### 1.3 Findings

1. **A quick planet takes about 6 minutes for 500 Myr and 10 minutes for 1,000 Myr** on a 6-core PC
   (the coupled stage costs ~0.5 s per Myr; deposits, young, amplify and bake add a fixed ~65 s).
   On a fast current desktop, divide by 1.2–1.4 (inferred): about 4–5 and 7–8 minutes. The two starts
   cost the same.
2. **The full preset does not fit a 16 GB PC.** It needs 8.9 GiB resident at 500 Myr and was killed
   twice on the 6 GiB profile; at 1,000 Myr it needs 9.5 GiB. On a real PC without a cgroup the OS
   would swap or kill the game. It takes 21–22 minutes for 500 Myr and 42 minutes for 1,000 Myr on 6
   cores when it fits.
3. **The memory cap does not hold the run.** Both presets run with the client's 4 GiB cap and the
   run's own accounting keeps under it: quick reports "982 MiB in use of the 4096 MiB cap" after the
   coupled stage (reserved peak 2.9 GiB); full reports 2,645 MiB after the coupled stage and 3,586
   MiB at its fullest, and even spilled history frames to disk to stay under the cap. The process
   meanwhile held 4.2 GiB (quick) and 7.6 GiB in the coupled stage, 8.9 GiB at the end (full). The
   accounting misses about 1.5 GiB on quick and about 5 GiB on full, so the editor's estimate, its
   "over the cap" refusal and its low-memory warning all under-count. This is a defect whichever
   option is chosen (question 3).
4. **Cores help less than expected.** 2 → 4 → 6 cores: 812 → 418 → 349 s. Past 4 cores the run gains
   1.2× for 1.5× the cores; about 5.4 of its 11 threads are busy on average (535 % CPU). A
   12-core PC would gain little over 6 (inferred).
5. **The box was shared.** The runs' threads waited 9–30 s (quick) and 80–91 s (full) for a CPU (cgroup `cpu.pressure`) and the
   two runs of the same planet differ by 7 % (349 and 372 s).

## 2. Option (a): a faster quick mode

**What it is.** A built-in preset family beside quick and full (`earthlike_express_16k`, and a 32k
one), made of settings the presets already have: a coarser grid, longer steps, fewer climate
snapshots. No new simulation code; the player picks it on the setup page like quick or full.

**Measured.** Each candidate is the quick preset, scattered continents, 500 Myr, on `mid6`, with one
change (the `quality` section of the matrix; baseline 349 s):

| Candidate | Change (`--set`) | Total | Speed-up | Coupled (plates / climate / surface / volcanoes / structure) | Peak memory | Folder |
|---|---|---|---|---|---|---|
| base | today's quick preset | 349 s | 1.0× | 285 s (41 / 80 / 119 / 15 / 29) | 4.2 GiB | 130 MB |
| l1r32 | `resolution.l1_r` 16 → 32: the L1 grid 512 × 164 instead of 1024 × 328 (4× fewer cells) | 160 s | **2.2×** | 125 s (32 / 52 / 25 / 5 / 10) | 3.4 GiB | 89 MB |
| l0-50k | `resolution.l0_samples` 100,000 → 50,000 (the plates' points) | 329 s | 1.06× | 271 s (29 / 73 / 124 / 16 / 28) | 4.1 GiB | 131 MB |
| step4 | `tectonics.time_step_myr` 2 → 4, `climate.snapshot_every_myr` 10 → 20 | 215 s | **1.6×** | 162 s (23 / 51 / 60 / 6 / 22) | 4.1 GiB | 135 MB |
| kf100 | `keyframes.every_myr` 50 → 100 (half the timelapse frames) | 343 s | 1.0× | 280 s | 4.2 GiB | 105 MB |
| len250 | 250 Myr instead of 500 (already the player's choice) | 204 s | 1.7× | 151 s (23 / 44 / 60 / 8 / 16) | 3.8 GiB | 93 MB |
| **express** | l1r32 + l0-50k + step4 + kf100 together | **115 s** | **3.0×** | 77 s (10 / 43 / 15 / 2 / 8) | 3.1 GiB | 74 MB |

- The plates' points and the keyframes cost almost nothing; the L1 grid and the step are what cost.
- In the express run the climate (43 s) and the fixed tail (young 6 s, amplify 22 s, bake 6 s:
  the 8192 × 2624 terrain level does not shrink with L1) are two thirds of the time. Climate
  snapshots every 50 Myr might take it to about 90 s (inferred, not run).
- On a 1,000-Myr history the coupled stage doubles and the tail does not, so express would gain
  more there (inferred: ~600 s → ~180 s).

**Quality.** The pictures are on the review page (private: https://spacesheep.dev/@benmac/deeptime-p1-24): for each candidate, the Create World
screen's last frame, the whole planet hill-shaded, its most rugged and its most coastal
1024-block squares. A coarser grid or a longer step changes the whole history, so each candidate is
a different planet: the comparison is of character, not places.

**What the pictures show** (Claude's reading of the review page; Ben judges):

- **Ground level hardly changes.** In the most rugged and the most coastal 1024-block squares,
  every candidate has the amplifier's ridges, gullies and valley lines. The block-scale character
  comes from the L3 amplifier, which none of the candidates touches.
- **The coarser L1 grid (l1r32, express) shows at map scale.** Continents are rounder with simpler
  coastlines, there are fewer small islands and island arcs, plains are broader and flatter, and
  the stair-steps on the sea floor (the L1 lattice the grid audit noted) are twice as large, 32
  blocks instead of 16. Lake shores in express show the same steps.
- **4-Myr steps (step4) look like today's planets**: the same coast detail, arcs and shelves.
- **Land share differs run to run.** Base 0.29, l1r32 0.16, l0-50k 0.17, step4 0.22, len250 0.22,
  express 0.21. Every candidate is a different planet, and P1-26's ensemble put the quick preset's
  land at 0.19–0.20 on average, so one run each cannot separate a bias from chance. The l1r32 run
  is the lowest; a calibration check would settle it.
- **The keyframe interval changes nothing but the timelapse**: kf100 gives the same planet as base
  (identical land share, windows and relief). That is a check that the comparison is sound.

**Build cost** (inferred): the preset files and the editor's family choice (½ day), a calibration
check of the coarse preset against Earth's hypsometry, Köppen shares and deposit counts (P1-17's
ensemble tools; 1–2 days, likely with a few settings retuned for the coarser grid, since erosion
and uplift were calibrated at L1 r 16), lang keys and docs (½ day). About 2 to 3 days.

## 3. Option (b): shipped pre-made planets

**What it is.** A set of planets baked ahead of time (by us, on the test VM, for each release) that
the editor offers as "Ready planets": pick one, see its timelapse and review page, Accept. Nothing is
simulated on the player's PC, so the wait is the copy and decompression: a few seconds (inferred:
42 MB of zstd at a few hundred MB/s).

**Size** (measured on the quick 500-Myr planet, `t-mid6-quick-scattered_continents-500`):

| What is shipped | Raw | zstd -19 | xz -9 |
|---|---|---|---|
| The whole world folder | 130 MB | 87 MB | — |
| Without the keyframes (playable, review page works, no timelapse) | 69 MB | 42 MB | 38 MB |
| The keyframes alone (26 layers × 14 frames) | 61 MB | 45 MB | — |

- The keyframes are only the timelapse. Keeping only the satellite layer for a replay would add
  about 2–3 MB (inferred: 1 of 26 layers), so about **45 MB per quick planet** with a short replay.
- A full-preset planet is larger: 204–222 MB raw at 500 Myr and 285 MB at 1,000 Myr (measured), so about 65–90 MB compressed without the keyframes (inferred from the quick planet's ratio).
- The climate mapping and TerraBlender regions are computed by the game at the review page from the
  pack's biome source (read: plan §7.12), so one shipped planet serves standalone, MIC and other
  packs alike.

**Hosting.** Six planets (both starts × three seeds) are about 270 MB.

| Where | Limit | For | Against |
|---|---|---|---|
| Inside the mod jar | Modrinth 500 MiB per file ([Modrinth issue 7260](https://github.com/modrinth/code/issues/7260)), CurseForge 2 GB (forum report, not checked) | works offline, no code for downloads | every server and every update downloads them; the jar goes from a few MB to ~300 MB |
| A separate optional "Deep Time planets" file or project | the same per file | the mod stays small; packs (MIC) can include it | a second thing to publish and keep in step with the format |
| Downloaded on demand from GitHub Releases | 2 GiB per file, no bandwidth limit ([GitHub docs](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)) | only what a player picks is fetched | needs a public repository or another host; network code, checksums and an offline fallback in the game |

**Upkeep.** Until a public release, formats change freely (decisions log 2026-10-01), so the planets
are re-baked for every release that touches the simulation or the format: a VM job of about 6 × 6
minutes and an upload. After a public release, old worlds keep their own data, so only new planets
need the current format.

**What it costs the player.** No choice of settings: a ready planet is fixed (its preset, its
start, its length). The editor's parameters stay greyed out for it. A planet everyone has seen is
less "your planet's own history", which is Deep Time's point.

**Build cost** (inferred): the bake job and manifest (1 day), a "Ready planets" list on the setup
page with thumbnails from the satellite frame and the replay from the shipped keyframes (2 days),
copy into staging (the existing `StagingMover` path; ½ day), and the hosting route (½ day in the jar,
2–3 days for downloads). About 4 to 6 days.

## 4. Option (c): background generation

Three variants; the first two can be combined.

**(c1) Start while the player edits.** When the setup page opens with a timeline already chosen
(the last one used), start the run at once at low priority; if the player presses Run with the same
settings, the page attaches to it; any change discards it. It saves the time spent on the setup page
(tens of seconds, inferred) and wastes the CPU when the settings change. Small (1–2 days), but a small
gain.

**(c2) A planet pool.** The game keeps one ready random planet in `<gamedir>/deeptime/pool/` and
bakes the next in the background whenever the game is open (title screen, other worlds), with 2
threads at the lowest priority. Create World offers it as "Use the planet the game prepared
(instant)"; any custom settings still run live.

- **Measured** (`bg2`: 2 cores, 2 threads at nice 19): the quick 500-Myr planet takes **812 s
  (13.5 min)**, peak 3.7 GiB. The timelapse comes with it (the keyframes are in the folder).
- The first world after installing still waits, unless combined with (b).
- It needs: a background session manager (start, pause while a world is being created, stop at
  quit), checkpoints so a half-done planet survives quitting the game (the core writes `*.dtcp`
  checkpoints today, dev and CLI only), a free-memory check (3.7 GiB beside a running client: skip
  on PCs with too little free memory; `MachineInfo.freeBytes` exists), a setting to turn it off, and
  disk: 130 MB per pooled planet.
- FPS beside the background run was not measured (Ben cannot launch the game, and the real-client
  route on the VM is for screenshots); 2 low-priority threads on a 6-core PC should leave the
  client's own threads alone (inferred).
- Build cost (inferred): about 4 to 6 days.

**(c3) Play while it generates.** Not possible as the simulation stands: every block depends on the
whole history (the terrain at the end, the strata of every age, the deposits). A player could only
wait in a holding area. Rejected.

## 5. Option (e): faster code, same planets

**What it is.** Optimisation passes that keep the planets as they are (the decisions log of
2026-10-01 already plans them "after features land").

**What the measurements say** (section 1):

- On 6 cores about 5.4 of the run's 11 threads are busy on average (535 % CPU). With 6 physical
  cores that is close to one busy thread per core, so the PC is nearly full; more threads would gain
  only what SMT gives. Less work per step, or less serial work, is what helps.
- Scaling is poor past 4 cores: 4 cores took 418 s, 6 cores 349 s (1.2× for 1.5× the cores), and
  2 cores 812 s. Part of the run is serial or bound by memory bandwidth.
- Where the time goes (quick 500 Myr, mid6): surface ~119 s, climate ~80 s, plates ~41–47 s,
  structure ~29 s, deposits ~17 s, amplify ~22 s, bake ~15 s. P1-17 found diffusion the largest
  surface cost and the depression flood serial on purpose.

**Cost and gain** (inferred): a profiling pass (1–2 days) and targeted work on the surface and
climate stages could plausibly give 1.3–2× on the coupled stage, over 1–3 weeks, with no quality
cost and no choice for the player. The gain is uncertain until profiled, and every later feature
can eat it again.

## 6. Option (d): combinations, and the recommendation

| Option | Wait for a quick 500-Myr planet on a 6-core PC | Player's own settings | Planet quality | Build (inferred) | Upkeep |
|---|---|---|---|---|---|
| Today | 5.8 min (10 min at 1,000 Myr) | yes | — | — | — |
| (a) express preset | **1.9 min** (about 3 min at 1,000 Myr, inferred) | yes | rounder coasts, fewer islands, coarser sea-floor steps; ground level the same | 2–3 days | a third preset family to keep calibrated |
| (a′) 4-Myr steps only | 3.6 min | yes | looks like today | 1–2 days (calibration check) | none extra if it becomes quick itself |
| (b) ready planets | seconds | no: a fixed list | today's quick | 4–6 days | re-bake and upload every release; hosting |
| (c2) planet pool | **instant** from the second world on (the first still waits) | the pooled planet is random; custom settings run live | today's quick | 4–6 days | none; 2 background threads and 130 MB of disk on the player's PC |
| (e) optimisation | perhaps 3–4.5 min (1.3–2×, uncertain) | yes | the same | 1–3 weeks | later features can eat the gain |
| **(d1) = (a) + (c2)** (recommended) | instant (pool) or 1.9 min (express, live) | yes | the player chooses | 6–9 days | as (a) |
| (d2) = (b) + (c2) | always instant | the first worlds are fixed planets | today's quick | 8–12 days | as (b) |

**Recommendation: (d1).**

- **Express first.** It is cheap, needs no new code, and cuts the live wait 3× for anyone who
  changes settings. Ben chooses between the full express (3.0×, coarser at map scale) and 4-Myr
  steps alone (1.6×, no visible cost; it could even become the quick preset itself).
- **The planet pool next.** Most players take a random planet. The pool gives them today's quality
  at no wait from the second world on, without fixing the planet in advance or shipping data.
- **The memory-accounting fix either way** (finding 3). Without it the editor can let a 16 GB PC
  start a full-preset run that the OS will kill.
- **Ready planets are not recommended.** They give up what makes Deep Time different (your planet's
  own history), they must be re-baked for every release while formats are free, and the
  download-on-demand route needs a public host. If wanted later, the pool's folder format is the
  same thing, so a modpack such as MIC could pre-fill the pool with a planet or two.

## 7. Questions for Ben

1. **Which option to build?** (a, recommended) **(d1): express now, the planet pool next**; (b) the
   express preset only; (c) the planet pool only; (d) shipped ready planets plus the pool; (e)
   optimisation passes only, no new mode.
2. **What is "express"?** (a, recommended) **all four changes** (coarser L1 grid, half the plate
   points, 4-Myr steps, a keyframe every 100 Myr): 3.0×, coarser coasts at map scale, offered beside
   quick and full; (b) 4-Myr steps only, 1.6× and no visible cost, as a separate choice; (c) 4-Myr
   steps made the quick preset itself, so every quick planet gets 1.6× after a calibration check.
3. **The memory accounting** (finding 3). (a, recommended) **Fix it now as its own task**: count the
   working sets the run misses, check the estimate against the measured peaks of this note, and have
   the editor warn that full needs about 9 GiB free; (b) only add the warning for full now and fix
   the accounting in the optimisation pass.
4. **When does the pool refill?** (a, recommended) **Whenever the game is open**, 2 threads at the
   lowest priority, skipped while a world is being created or when less than 6 GiB is free, with a
   setting to turn it off; (b) only on the title screen and other menus (no load during play, but it
   rarely gets the 13 minutes it needs).

## 8. What was built for this note

- `tools/review/p1-24-profile.sh`: one run on a player-PC profile (a cgroup scope), with stamped
  log lines, a memory sampler and `result.json`.
- `tools/review/p1-24-matrix.sh`: the runs of this note (`timing`, `quality`, `mem`, `bg`).
- `tools/review/p1-24-views.py` / `.sh`: the quality pictures (the Create World screen's last
  frame, the whole planet hill-shaded, the most rugged and the most coastal 1024-block windows).
- Nothing in the game or the simulation changed: every option needs Ben's choice first.
