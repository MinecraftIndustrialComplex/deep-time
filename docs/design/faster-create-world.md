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
  {{WAIT}} s for a CPU in a ~350 s run. Memory bandwidth and caches are still shared, so these
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

{{TIMING}}

## 2. Option (a): a faster quick mode

{{OPTION_A}}

## 3. Option (b): shipped pre-made planets

{{OPTION_B}}

## 4. Option (c): background generation

{{OPTION_C}}

## 5. Option (e): faster code, same planets

{{OPTION_E}}

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
