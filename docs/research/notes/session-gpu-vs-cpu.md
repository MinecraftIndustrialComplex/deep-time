# GPU against CPU optimisation, and a target of one second per million years

Date: 2026-09-28 (first look at GPU options) and 2026-10-01 (the comparison and the speed target);
both from the transcript's UTC timestamps.
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

Does the simulation use the GPU, and would a large redesign for the GPU save more time than a large
round of CPU optimisation? Separately: could the simulation run at about one second of wall-clock
time per million simulated years ("1 s per Myr") on a good gaming PC, for the final version with all
planned features?

## Assessment

GPU at design time, 2026-09-28. The coordinator listed a GPU path (route B in the engine
comparison) and advised against making it authoritative. The reasons were: GPU compute shaders need
OpenGL 4.3 and macOS stops at 4.1; hosted servers have no GPU; a compute shader in the Veil
rendering library already crashes on Intel UHD 620 graphics; and GPU results are not bit-identical
to CPU results. It named two published GPU codes that could be a base: an MIT-licensed
stream-power shader of 214 lines (Schott et al. 2023) and FastFlow, whose code is research-only.
Its advice was that the GPU could at most power previews.

GPU in practice, 2026-10-01. The coordinator said nothing uses the GPU: the simulation runs on the
CPU across several threads. It did not think a GPU was worth adding yet, for three reasons. Most of
the time goes to step-by-step work on irregular data (plate motion, river routing and lake filling,
rock structure, event bookkeeping), which would have to be redesigned and not just ported. The
grids are small, about half a million cells per step, so moving data to and from the GPU eats much
of the gain. And dedicated servers usually have no GPU while a player's GPU is busy drawing the
game, so a CPU version must stay and both paths must be kept in step.

Where the time goes, from the timing output of a 750-million-year run (about 31 minutes on 8
threads on a heavily loaded Apple-silicon desktop):

| Stage | Time | Kind of work |
|---|---|---|
| Surface erosion | 767 s | grid maths (diffusion solve 245 s, rivers 108 s), plus routing and bookkeeping |
| Plates | 304 s | irregular, about 100,000 moving samples |
| Rock structure | 287 s | per-column bookkeeping and output |
| Climate | 251 s | small grids, many short steps |
| Final detail pass | 81 s | one 33-million-cell grid |
| Deposits, volcanoes, young terrain | 130 s | mostly irregular |

Estimates for the two options, which the coordinator labelled estimates from stage timings and not
measurements:

- Full GPU redesign: about half the run is regular grid arithmetic that could speed up 10 to 30
  times, so overall about 3 to 6 times on a good dedicated graphics card, 1 to 2 times on laptop
  integrated graphics, and none on servers. It estimated months of work, two code paths and a GPU
  test machine.
- CPU optimisation wave: a multigrid solver (a faster iterative method) could take the 245 s
  diffusion solve to about 30 s; fewer river passes; computing rock-structure output only for kept
  frames; incremental event bookkeeping; more parallel stages, especially plates; longer time steps
  in quiet periods. It estimated 3 to 5 times from the first two groups and 4 to 8 times with
  adaptive steps, on every machine, in weeks, with one code path.
- Most of the GPU's theoretical gain comes from the same algorithm changes the CPU wave makes. After
  those, a GPU might add 2 to 3 times more, only on grid-heavy stages and only with a real card.
  The coordinator recommended the CPU wave first and named the final detail pass as the best GPU
  candidate afterwards.

Speed target. Measured on the quick preset (Earth-like, 16k-block planet) on an Apple-silicon
desktop: 1000 Myr in 376 s (0.38 s per Myr) and 500 Myr in 239 s (0.48 s per Myr), both under light
load; the 750 Myr run above, under heavy load, took 1,868 s (2.5 s per Myr). Those totals include a
fixed finish (deposits, detail pass, bake) of about 2 to 3 minutes. The coordinator said the quick
preset already meets 1 s per Myr when the machine is quiet. Its estimates for the final version:

- Quick preset: 0.6 to 0.9 s per Myr without optimisation, and 0.15 to 0.3 s after the CPU wave.
  The per-cell climate adds 0.03 to 0.12 s per Myr (the design document's own estimate).
- Full preset: no clean timing existed. If its grids hold about four times the cells, it would run
  at about 2 to 3.5 s per Myr unoptimised and about 0.4 to 0.9 s after a 4 to 8 times wave.
- It suggested the target be stated as a simulation rate, with the fixed finish given its own
  budget, and made a gate in the benchmark task with a named reference PC.

## What was sourced and what was not

- The OpenGL 4.3 and macOS limit, the Veil crash and the two GPU codes: sub-agent research saved in
  `docs/research/notes/runtime_packaging_options.md` and
  `docs/research/geologic-history-worldgen-report.md`. FastFlow's timing and licence are in that
  research and were not re-checked here.
- Stage timings and the three run times: measurements in this repository's own run logs (the
  750 Myr run's timing line, and the build report for the 1000 and 500 Myr runs). They were taken on
  one machine under varying load.
- The GPU and CPU speed-up ranges (3 to 6, 4 to 8, 10 to 30, 2 to 3 times), the multigrid figure of
  30 s, and the final-version rates: the coordinator's estimates. No source was consulted; this is
  the model's own knowledge and has not been checked. Nothing was prototyped.
- No x86 desktop was measured on a quiet machine, and no recent full-preset timing existed.

## What was decided

On 2026-10-01 Ben decided: no GPU work for now, no speed gate yet, features land first and
optimisation passes (CPU first, under the faster-Create-World task P1-24) come afterwards. This is
the 2026-10-01 row "GPU use, and a speed gate of ~1 s per Myr" in
`docs/deeptime-design-decisions.md`, which also records the quick-preset reference of about 0.4 to
0.5 s per Myr unloaded. The same day's row on reproducibility says performance is preferred over
identical runs across machines.

## Open

- The speed-up estimates for both options are untested. A profile of a full run with a ranked list
  of the top five costs was offered and not requested.
- No x86 timing on an unloaded machine; no recent full-preset timing; no definition of a "good
  gaming PC" was fixed.
- Whether 1 s per Myr holds once all planned features exist is unknown.
- The GPU claims about drivers and API limits come from research notes and were not rechecked.
