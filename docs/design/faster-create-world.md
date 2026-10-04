# Faster Create World (P1-24): measurements and options

Status: **options for Ben** (2026-10-04). Nothing here changes the shipped game. Ben chooses which
option to build (plan §10, P1-24: "Measure player-side generation on a mid-range PC profile, then
choose (Ben) among a faster quick mode, shipped pre-made planets and background generation, and
build it"). The decisions log of 2026-10-01 still holds: features land first, optimisation passes
(CPU first) come after, and there is no speed gate yet.

Statements are marked **measured** (a run made for this note), **read** (from the code or a
document) or **inferred**.

{{SUMMARY}}

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
- **The box is shared.** The VM (AMD EPYC, D32as_v7) ran other agents' jobs beside these, at a load of
  24 to 39 on 32 threads. The pinned CPUs keep that out of the run mostly: the threads waited
  9 to 30 s for a CPU in a ~350 s run. Memory bandwidth and caches are still shared, so these
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
{{FULL_ROWS}}

Peak memory is the run's peak resident set. Sampled every 2 s, the quick 500-Myr run peaks at
2.4 GiB in the coupled stage, 2.5 GiB in deposits and **4.2 GiB in young + amplify + bake** (the
amplifier holds the whole 8192 × 2624 terrain level); the full run at 7.6 GiB in the coupled stage
and 8.8 GiB at the end.

### 1.3 Findings

1. **A quick planet takes about 6 minutes for 500 Myr and 10 minutes for 1,000 Myr** on a 6-core PC
   (the coupled stage costs ~0.55 s per Myr; deposits, young, amplify and bake add a fixed ~65 s).
   On a fast current desktop, divide by 1.2–1.4 (inferred): about 4–5 and 7–8 minutes. The two starts
   cost the same.
2. **The full preset does not fit a 16 GB PC.** It needs 8.9 GiB resident at 500 Myr and was killed
   twice on the 6 GiB profile. On a real PC without a cgroup the OS would swap or kill the game. It
   takes 20 minutes on 6 cores when it fits.
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
5. **The box was shared.** The runs' threads waited 9–30 s for a CPU (cgroup `cpu.pressure`) and the
   two runs of the same planet differ by 7 % (349 and 372 s).

## 2. Option (a): a faster quick mode

{{OPTION_A}}

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
- A full-preset planet is larger: {{FULL_SIZE}}.
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

- On 6 cores the run is CPU-bound, not waiting: it used {{CPU_UTIL}} of the 12 hardware threads on
  average. More threads will not help a 6-core PC; less work per step will.
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

{{OPTION_D}}

## 7. Questions for Ben

{{QUESTIONS}}

## 8. What was built for this note

- `tools/review/p1-24-profile.sh`: one run on a player-PC profile (a cgroup scope), with stamped
  log lines, a memory sampler and `result.json`.
- `tools/review/p1-24-matrix.sh`: the runs of this note (`timing`, `quality`, `mem`, `bg`).
- `tools/review/p1-24-views.py` / `.sh`: the quality pictures (the Create World screen's last
  frame, the whole planet hill-shaded, the most rugged and the most coastal 1024-block windows).
- Nothing in the game or the simulation changed: every option needs Ben's choice first.
