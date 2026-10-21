# The continent bisect (2026-10-21)

Ben: "I've noticed a lack of proper continents across most of my recent runs in the past ~2 days"
(his run: earthlike_quick_16k, seed 1, supercontinent, 1000 Myr, master 0a7431e1, land 0.215, the
right half an archipelago). The same runs on ten first-parent commits of master, each commit's own
release `deeptime` and its own shipped `earthlike_quick_16k`, no overrides: seeds 1 and 2, starts
supercontinent and scattered_continents, 1000 Myr, read at the end and at 500 Myr by one script.
4b615b24 (land shape and the high ground) is bit-identical to bc2e5472 on supercontinent seed 1
(the three dumped fields compare equal) and is left out. 80 runs, 2 threads each, on the Azure VM.

| commit | date (2026) | what it is |
|---|---|---|
| 25ae6723 | 10-18 | resampling lumps (PLT-289 to PLT-292) |
| fb38204f | 10-19 | plate cadence, per-Myr plate rules |
| 3e7edd49 | 10-19 | GPU phase 2 |
| bc2e5472 | 10-20 02:48 | continents keep their land, lower-crustal flow |
| c24ff793 | 10-20 11:37 | ocean crust drawn as ocean crust |
| 3bf532d1 | 10-20 14:10 | crust budget, cordillera 6.0 to 1.7 mm/yr (PLT-320) |
| 9c21cee3 | 10-20 15:16 | PLT-294 `transition_ocean_crust` on in the shipped presets |
| 9c9c2012 | 10-20 18:19 | arc fronts (PLT-330), arc basement (PLT-331) |
| 3768c05b | 10-20 23:25 | lake sediment (HYD-110) |
| 30c4d664 | 10-21 02:37 | now (the biome bake's land call: its simulation is byte-identical to 3768c05b) |

## What each column is

All on the stage's L1 grid (Lambert equal-area, 1024 by 328 cells, true-area weighted). Land is
`surface/elevation_m` above the climate stage's sea level, read after plates, climate, surface,
volcanoes, impacts and structure (`deeptime stage volcanoes`; the checkpoint's next stage is deposits).

- `land` land share of the planet (Earth 0.29); `cont` continental crust's share of the planet
  (`plates/crust_type` >= 0.5); `drown` the share of that crust under the sea (Earth about 0.19);
  `hi2` land above 2 km as a share of the land.
- `L1`, `L2` the largest and second-largest landmass as shares of the planet (Earth: Afro-Eurasia
  0.167, the Americas 0.083, Antarctica 0.027, Australia 0.015); `n>=.01` landmasses of at least
  0.01 of the planet; `in>=.01` the share of all land in those (Earth about 0.95); `pieces` all
  landmasses.
- A landmass is land cells joined by an edge or a corner, wrapped across the seam.
- `arc` the island-arc share of the land (the crust budget's `cb_origin`, L0 samples at the run's
  own final time); it exists only from c24ff793 on (4b615b24 added the line).
- `check` the largest difference to the stage's own printed `hypsometry` line (land, continental
  share, drowned share, land above 2 km): at most 0.0005 in every run, so the script reads what
  the stage reports.
- `sd` the run-to-run standard deviation over the four runs of a commit: a land mean of four runs
  carries about +-0.01 to +-0.02, so a step under 0.03 is not a finding.

## Choices (mine, undoable)

- **500 Myr** is `--set timeline.stop_myr=500` on the run planned for 1000 Myr: the world at that
  time of the longer plan, with its own tail (docs/design/playability-steering.md). The stage prints
  `steps=250 t_years=-500000000`. It is not the 1000 Myr run's frame at 500 Myr (the climate and
  surface tail differ), and not a separate 500 Myr run (the plan's rates differ).
- **CPU compute** (`DEEPTIME_COMPUTE=cpu`) on every run: the VM has no GPU, and a GPU world differs from
  a CPU world (STATUS: land 0.215 CPU against 0.235 GPU at one commit). Ben's 0.215 on 0a7431e1
  is therefore not directly comparable to the table; the nearest commit here, 9c9c2012 (the plate
  and surface simulation of 0a7431e1; its later commits are deposits, ledger, biome and map changes), reads
  0.227 on supercontinent seed 1.
- **Final frame** is the end of the coupled pipeline (the `stage volcanoes` checkpoint), not the
  bake's frame: the young landscape and amplify stages come after and are not read.
- `DEEPTIME_CRUST_BUDGET=1` on every run (it prints the `arc` lines). It is measurement only: HEAD's
  supercontinent seed 1, 1000 Myr, run without it, has byte-identical `surface/elevation_m`,
  `plates/crust_type` and `elevation` (`continent-bisect.sh control`).
- Older commits' `dump-fields` has no `globals.json`: the sea level is the stage's printed
  `sea_level_m` (one decimal) there, and `check` stays under 0.0005.

## Commands

```
tools/review/continent-bisect.sh ship <commits...>                     # laptop: git archive of each commit onto the VM
tools/remote/mac.sh -- tools/remote/detach.sh start cb-<c> \
    tools/review/continent-bisect.sh all <c> 1 supercontinent,scattered_continents 1000,500
tools/remote/mac.sh -- tools/remote/detach.sh start cb2-<c> \
    tools/review/continent-bisect.sh runs <c> 2 supercontinent,scattered_continents 1000,500
tools/remote/mac.sh -- tools/remote/detach.sh start cb-ctl tools/review/continent-bisect.sh control 30c4d664 1 supercontinent 1000
tools/remote/mac.sh -- python3 -I tools/review/continent-bisect.py /data/ban/deeptime-runs/continent-bisect \
    25ae6723,fb38204f,3e7edd49,bc2e5472,c24ff793,3bf532d1,9c21cee3,9c9c2012,3768c05b,30c4d664 \
    1,2 supercontinent,scattered_continents 1000,500 --csv continent-bisect.csv --md continent-bisect.md --runs
```

The per-run rows are in continent-bisect.csv (`--runs` text in continent-bisect.txt).

## The tables (mean over seeds 1 and 2; the last two are the mean of all four runs)

### supercontinent, 1000 Myr (mean over seeds 1,2)

| commit | runs | land | cont | drown | hi2 | L1 | L2 | n>=.01 | in>=.01 | arc | pieces | check |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25ae6723 | 2 | 0.146 | 0.468 | 0.688 | 0.180 | 0.032 | 0.025 | 4.5 | 0.642 | - | 150.5 | 0.0003 |
| fb38204f | 2 | 0.184 | 0.458 | 0.606 | 0.202 | 0.109 | 0.034 | 3.0 | 0.827 | - | 173.0 | 0.0005 |
| 3e7edd49 | 2 | 0.158 | 0.424 | 0.635 | 0.196 | 0.051 | 0.036 | 4.0 | 0.701 | - | 114.0 | 0.0003 |
| bc2e5472 | 2 | 0.295 | 0.446 | 0.405 | 0.222 | 0.142 | 0.064 | 4.5 | 0.872 | - | 275.0 | 0.0004 |
| c24ff793 | 2 | 0.308 | 0.419 | 0.327 | 0.301 | 0.130 | 0.078 | 5.0 | 0.852 | 0.539 | 286.0 | 0.0003 |
| 3bf532d1 | 2 | 0.292 | 0.411 | 0.349 | 0.330 | 0.129 | 0.045 | 5.0 | 0.819 | 0.587 | 313.0 | 0.0004 |
| 9c21cee3 | 2 | 0.276 | 0.423 | 0.398 | 0.303 | 0.089 | 0.055 | 5.0 | 0.843 | 0.468 | 305.0 | 0.0005 |
| 9c9c2012 | 2 | 0.236 | 0.344 | 0.371 | 0.305 | 0.090 | 0.055 | 3.5 | 0.858 | 0.456 | 274.0 | 0.0003 |
| 3768c05b | 2 | 0.265 | 0.394 | 0.381 | 0.224 | 0.130 | 0.043 | 4.5 | 0.881 | 0.376 | 296.5 | 0.0005 |
| 30c4d664 | 2 | 0.265 | 0.394 | 0.381 | 0.224 | 0.130 | 0.043 | 4.5 | 0.881 | 0.376 | 296.5 | 0.0005 |

### scattered_continents, 1000 Myr (mean over seeds 1,2)

| commit | runs | land | cont | drown | hi2 | L1 | L2 | n>=.01 | in>=.01 | arc | pieces | check |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25ae6723 | 2 | 0.181 | 0.383 | 0.548 | 0.313 | 0.063 | 0.046 | 3.5 | 0.827 | - | 132.5 | 0.0004 |
| fb38204f | 2 | 0.183 | 0.416 | 0.577 | 0.280 | 0.099 | 0.035 | 3.0 | 0.829 | - | 126.0 | 0.0004 |
| 3e7edd49 | 2 | 0.177 | 0.445 | 0.612 | 0.211 | 0.055 | 0.029 | 4.5 | 0.722 | - | 156.5 | 0.0004 |
| bc2e5472 | 2 | 0.316 | 0.415 | 0.323 | 0.316 | 0.208 | 0.048 | 2.0 | 0.844 | - | 281.0 | 0.0004 |
| c24ff793 | 2 | 0.337 | 0.439 | 0.293 | 0.290 | 0.201 | 0.035 | 4.0 | 0.897 | 0.564 | 283.5 | 0.0004 |
| 3bf532d1 | 2 | 0.277 | 0.366 | 0.317 | 0.320 | 0.140 | 0.067 | 4.0 | 0.875 | 0.591 | 234.5 | 0.0003 |
| 9c21cee3 | 2 | 0.261 | 0.400 | 0.418 | 0.234 | 0.092 | 0.035 | 5.5 | 0.756 | 0.568 | 323.5 | 0.0004 |
| 9c9c2012 | 2 | 0.251 | 0.353 | 0.354 | 0.276 | 0.086 | 0.074 | 4.5 | 0.817 | 0.431 | 276.5 | 0.0004 |
| 3768c05b | 2 | 0.248 | 0.354 | 0.367 | 0.316 | 0.094 | 0.048 | 5.5 | 0.896 | 0.449 | 255.0 | 0.0004 |
| 30c4d664 | 2 | 0.248 | 0.354 | 0.367 | 0.316 | 0.094 | 0.048 | 5.5 | 0.896 | 0.449 | 255.0 | 0.0004 |

### supercontinent, 500 Myr (mean over seeds 1,2)

| commit | runs | land | cont | drown | hi2 | L1 | L2 | n>=.01 | in>=.01 | arc | pieces | check |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25ae6723 | 2 | 0.179 | 0.385 | 0.548 | 0.231 | 0.078 | 0.032 | 3.5 | 0.827 | - | 116.0 | 0.0004 |
| fb38204f | 2 | 0.187 | 0.415 | 0.561 | 0.239 | 0.072 | 0.033 | 4.0 | 0.761 | - | 129.5 | 0.0004 |
| 3e7edd49 | 2 | 0.171 | 0.404 | 0.587 | 0.192 | 0.051 | 0.046 | 3.5 | 0.698 | - | 145.5 | 0.0004 |
| bc2e5472 | 2 | 0.277 | 0.412 | 0.380 | 0.303 | 0.105 | 0.064 | 4.0 | 0.824 | - | 211.0 | 0.0003 |
| c24ff793 | 2 | 0.269 | 0.386 | 0.346 | 0.368 | 0.138 | 0.060 | 3.5 | 0.873 | 0.389 | 231.0 | 0.0004 |
| 3bf532d1 | 2 | 0.277 | 0.395 | 0.341 | 0.333 | 0.157 | 0.050 | 3.5 | 0.903 | 0.350 | 230.5 | 0.0004 |
| 9c21cee3 | 2 | 0.261 | 0.360 | 0.322 | 0.376 | 0.170 | 0.039 | 3.0 | 0.893 | 0.310 | 241.0 | 0.0004 |
| 9c9c2012 | 2 | 0.225 | 0.310 | 0.329 | 0.370 | 0.075 | 0.066 | 6.0 | 0.932 | 0.298 | 196.5 | 0.0004 |
| 3768c05b | 2 | 0.257 | 0.399 | 0.387 | 0.160 | 0.136 | 0.060 | 3.0 | 0.869 | 0.228 | 215.0 | 0.0004 |
| 30c4d664 | 2 | 0.257 | 0.399 | 0.387 | 0.160 | 0.136 | 0.060 | 3.0 | 0.869 | 0.228 | 215.0 | 0.0004 |

### scattered_continents, 500 Myr (mean over seeds 1,2)

| commit | runs | land | cont | drown | hi2 | L1 | L2 | n>=.01 | in>=.01 | arc | pieces | check |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25ae6723 | 2 | 0.187 | 0.370 | 0.509 | 0.303 | 0.097 | 0.044 | 2.5 | 0.846 | - | 108.0 | 0.0005 |
| fb38204f | 2 | 0.168 | 0.372 | 0.564 | 0.308 | 0.077 | 0.037 | 3.5 | 0.830 | - | 124.0 | 0.0004 |
| 3e7edd49 | 2 | 0.163 | 0.363 | 0.571 | 0.293 | 0.088 | 0.023 | 3.0 | 0.785 | - | 141.5 | 0.0004 |
| bc2e5472 | 2 | 0.325 | 0.453 | 0.335 | 0.216 | 0.152 | 0.074 | 4.0 | 0.891 | - | 246.5 | 0.0004 |
| c24ff793 | 2 | 0.263 | 0.347 | 0.302 | 0.380 | 0.095 | 0.076 | 4.0 | 0.848 | 0.459 | 274.0 | 0.0004 |
| 3bf532d1 | 2 | 0.249 | 0.367 | 0.375 | 0.365 | 0.083 | 0.049 | 5.5 | 0.841 | 0.440 | 260.5 | 0.0004 |
| 9c21cee3 | 2 | 0.298 | 0.405 | 0.324 | 0.271 | 0.218 | 0.034 | 2.5 | 0.875 | 0.411 | 258.0 | 0.0004 |
| 9c9c2012 | 2 | 0.235 | 0.360 | 0.394 | 0.299 | 0.085 | 0.050 | 4.5 | 0.801 | 0.324 | 267.5 | 0.0005 |
| 3768c05b | 2 | 0.211 | 0.297 | 0.350 | 0.381 | 0.094 | 0.031 | 4.0 | 0.833 | 0.347 | 218.5 | 0.0004 |
| 30c4d664 | 2 | 0.211 | 0.297 | 0.350 | 0.381 | 0.094 | 0.031 | 4.0 | 0.833 | 0.347 | 218.5 | 0.0004 |

### both starts, t = 1000 Myr, seeds 1,2 (mean over every run present)

| commit | runs | land | cont | drown | hi2 | L1 | L2 | n>=.01 | in>=.01 | arc | pieces | check | sd land | sd L1 | sd n>=.01 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25ae6723 | 4 | 0.163 | 0.426 | 0.618 | 0.246 | 0.047 | 0.036 | 4.0 | 0.734 | - | 141.5 | 0.0004 | 0.039 | 0.032 | 0.8 |
| fb38204f | 4 | 0.184 | 0.437 | 0.591 | 0.241 | 0.104 | 0.035 | 3.0 | 0.828 | - | 149.5 | 0.0004 | 0.038 | 0.055 | 0.8 |
| 3e7edd49 | 4 | 0.168 | 0.435 | 0.624 | 0.204 | 0.053 | 0.033 | 4.2 | 0.712 | - | 135.2 | 0.0004 | 0.032 | 0.024 | 1.0 |
| bc2e5472 | 4 | 0.305 | 0.430 | 0.364 | 0.269 | 0.175 | 0.056 | 3.2 | 0.858 | - | 278.0 | 0.0004 | 0.034 | 0.064 | 1.7 |
| c24ff793 | 4 | 0.323 | 0.429 | 0.310 | 0.295 | 0.166 | 0.056 | 4.5 | 0.874 | 0.552 | 284.8 | 0.0003 | 0.021 | 0.098 | 1.7 |
| 3bf532d1 | 4 | 0.284 | 0.388 | 0.333 | 0.325 | 0.134 | 0.056 | 4.5 | 0.847 | 0.589 | 273.8 | 0.0003 | 0.030 | 0.049 | 1.3 |
| 9c21cee3 | 4 | 0.269 | 0.411 | 0.408 | 0.268 | 0.090 | 0.045 | 5.2 | 0.800 | 0.518 | 314.2 | 0.0004 | 0.013 | 0.035 | 1.7 |
| 9c9c2012 | 4 | 0.244 | 0.349 | 0.363 | 0.291 | 0.088 | 0.065 | 4.0 | 0.838 | 0.444 | 275.2 | 0.0003 | 0.012 | 0.018 | 1.4 |
| 3768c05b | 4 | 0.257 | 0.374 | 0.374 | 0.270 | 0.112 | 0.045 | 5.0 | 0.889 | 0.412 | 275.8 | 0.0004 | 0.021 | 0.033 | 1.8 |
| 30c4d664 | 4 | 0.257 | 0.374 | 0.374 | 0.270 | 0.112 | 0.045 | 5.0 | 0.889 | 0.412 | 275.8 | 0.0004 | 0.021 | 0.033 | 1.8 |

### both starts, t = 500 Myr, seeds 1,2 (mean over every run present)

| commit | runs | land | cont | drown | hi2 | L1 | L2 | n>=.01 | in>=.01 | arc | pieces | check | sd land | sd L1 | sd n>=.01 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25ae6723 | 4 | 0.183 | 0.377 | 0.528 | 0.267 | 0.087 | 0.038 | 3.0 | 0.836 | - | 112.0 | 0.0004 | 0.006 | 0.027 | 0.8 |
| fb38204f | 4 | 0.178 | 0.394 | 0.562 | 0.273 | 0.074 | 0.035 | 3.8 | 0.795 | - | 126.8 | 0.0004 | 0.019 | 0.025 | 0.5 |
| 3e7edd49 | 4 | 0.167 | 0.383 | 0.579 | 0.242 | 0.070 | 0.035 | 3.2 | 0.741 | - | 143.5 | 0.0004 | 0.016 | 0.036 | 1.5 |
| bc2e5472 | 4 | 0.301 | 0.432 | 0.357 | 0.260 | 0.129 | 0.069 | 4.0 | 0.858 | - | 228.8 | 0.0004 | 0.042 | 0.041 | 1.2 |
| c24ff793 | 4 | 0.266 | 0.367 | 0.324 | 0.374 | 0.117 | 0.068 | 3.8 | 0.860 | 0.424 | 252.5 | 0.0004 | 0.021 | 0.027 | 0.5 |
| 3bf532d1 | 4 | 0.263 | 0.381 | 0.358 | 0.349 | 0.120 | 0.050 | 4.5 | 0.872 | 0.395 | 245.5 | 0.0004 | 0.017 | 0.043 | 1.7 |
| 9c21cee3 | 4 | 0.279 | 0.382 | 0.323 | 0.323 | 0.194 | 0.036 | 2.8 | 0.884 | 0.361 | 249.5 | 0.0004 | 0.038 | 0.031 | 1.0 |
| 9c9c2012 | 4 | 0.230 | 0.335 | 0.361 | 0.335 | 0.080 | 0.058 | 5.2 | 0.866 | 0.311 | 232.0 | 0.0005 | 0.018 | 0.017 | 1.5 |
| 3768c05b | 4 | 0.234 | 0.348 | 0.369 | 0.271 | 0.115 | 0.046 | 3.5 | 0.851 | 0.288 | 216.8 | 0.0004 | 0.031 | 0.041 | 0.6 |
| 30c4d664 | 4 | 0.234 | 0.348 | 0.369 | 0.271 | 0.115 | 0.046 | 3.5 | 0.851 | 0.288 | 216.8 | 0.0004 | 0.031 | 0.041 | 0.6 |

## What the tables say (final frame, all four runs per commit)

- **The big step up is bc2e5472** (continents keep their land): land 0.168 to 0.305 (+0.137),
  drowned continental crust 0.62 to 0.36, largest landmass 0.053 to 0.175 of the planet (Afro-Eurasia
  0.167). Nothing else comes close. Before it (25ae6723, fb38204f, 3e7edd49) the model sat at land
  0.16 to 0.18 with 0.59 to 0.62 of its continental crust under the sea: that is the "no proper
  continents" Ben describes, and it was the state of master through 10-19 until 10-20 02:48.
- **c24ff793 is the best state measured**: land 0.323, drowned 0.31, largest landmass 0.166, 87 % of the
  land in landmasses of at least 0.01.
- **Then a slide, c24ff793 to 9c9c2012 (10-20 11:37 to 18:19)**: land 0.323, 0.284, 0.269, 0.244;
  largest landmass 0.166, 0.134, 0.090, 0.088; continental crust 0.429, 0.388, 0.411, 0.349. By
  commit: 3bf532d1 (crust budget, cordillera) -0.039 land; 9c21cee3 (PLT-294 on) -0.015 land and the
  largest landmass 0.134 to 0.090; 9c9c2012 (arc fronts and basement) -0.025 land and -0.062 continental
  crust, the lowest continental share of any commit. Single steps under 0.03 are inside the noise
  (a four-run mean carries +-0.01 to +-0.02); the whole c24ff793 to 9c9c2012 fall (-0.079) is not.
- **3768c05b (lake sediment) gives back +0.013 land and +0.025 continental crust**, inside the noise;
  30c4d664's simulation is byte-identical to 3768c05b's, as expected of a bake change.
- At 500 Myr the same shape with a lower peak: bc2e5472 0.301, c24ff793 0.266, 9c21cee3 0.279,
  9c9c2012 0.230, 3768c05b 0.234.
- **Against Earth at the end**: land 0.257 (0.29), drowned 0.374 (about 0.19), largest landmass 0.112
  (0.167), 5.0 landmasses of at least 0.01 (4), 89 % of the land in them (95 %). Landmass count is
  not the shortfall; the area of the largest ones and the drowned share are. The island-arc share
  of the land falls over the same commits (0.552 at c24ff793, 0.589 at 3bf532d1, 0.412 now).
- **Regression or throughout?** Both. Master was at its worst before bc2e5472 (and 25ae6723, the
  oldest commit measured, is already there, so how long the model sat in that trough is not
  measured), jumped to Earth-like land and a largest landmass at Earth's Afro-Eurasia for about half a day
  (10-20 02:48 to 14:10), and has since fallen about 0.065 land and 0.054 of the largest landmass
  back, to a state that is still far better than the one before bc2e5472 but short of Earth's
  drowned share and largest landmass. Ben's 0.215 on 0a7431e1 sits in the slide (9c9c2012 reads
  0.227 for that seed and start on the CPU).
- Limits: two seeds, four runs a commit; the CPU world (the GPU world differs); the end of the
  coupled pipeline, not the bake; no commit before 25ae6723 measured.
