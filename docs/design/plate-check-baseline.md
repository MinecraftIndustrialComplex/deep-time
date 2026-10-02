# plate-check: baseline on master (2026-10-01)

`deeptime plate-check` on the mean of seeds 1-4 of `earthlike_quick_16k`, supercontinent start,
1000 Myr, 2 Myr steps, `deeptime stage volcanoes` (the coupled pipeline), on master fa09e8b plus the
recording of task 1 (`docs/design/plate-collisions-and-welding.md`, section 7). The check **fails**
on this baseline, 16 of its 19 ranged targets, which is the point of a baseline: it is what tasks 2-5
must move. The recording changes nothing a run computes (a test shows it).

Reproduce (about 15-20 minutes a run at 2 threads on the test VM, two runs at a time):

    tools/remote/mac.sh -- tools/remote/detach.sh start pc tools/review/plate-check.sh 1000 1,2,3,4 supercontinent base
    tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait pc

The script prints the per-run tables and the mean table below, and `deeptime plate-check --run <ck>
[--run <ck> ...] [--json out] [--targets file]` reads any checkpoint, world-data folder or manifest.
Target ranges live in `MEASURES` in `crates/deeptime-core/src/plates/check.rs`; a file of
`{"key": [lo, hi]}` moves them (null: open end; `"key": null`: report only).

## Against the targets

Value is the mean of the four runs; "runs" are the runs' own values (seeds 1, 2, 3, 4). The design's
"Now" column is the number the design measured with the throwaway trace on the same four runs (a
bracket is its range over the seeds). "Agrees" means the value is inside the design's figure or its
range.

| Measure | Value | Runs | Design "Now" | Target | Result |
|---|---|---|---|---|---|
| Largest plate, median share of the planet | 0.58 | 0.56 0.66 0.61 0.50 | 0.58 [0.50-0.66] | 0.15-0.35 | FAIL |
| Steps with one plate above 50 % | 58.7 % | 58.8 68.8 57.8 49.6 | 59 % [50-69] | at most 10 % | FAIL |
| Plates above 5 % of the planet, median | 3.75 | 4 3 4 4 | 3.75 | 5-9 | FAIL |
| Median plate lifetime (plates that died) | 45 Myr | 46 40 56 38 | 45 [38-56] | at least 80 | FAIL |
| Plates dead within 20 Myr | 33.4 % | 27.3 38.2 33.8 34.4 | 33 % | at most 15 % | FAIL |
| Rift-born plates alive after 50 Myr | 18.7 % | 14.0 20.0 21.6 19.2 | 19 % [14-22] | at least 60 % | FAIL |
| Rift-born plates welded back to their own rift family within 50 Myr | 41.9 % | 38.0 40.0 47.1 42.5 | about 40 % | at most 10 % | FAIL |
| Land raised by one collision in one step, the most | 4.47 % | 5.19 4.88 2.86 4.96 | 4.5 % [2.9-5.2] | at most 0.5 % | FAIL |
| Most land gained in any step (after the first 10 Myr) | 3.75 % | 4.61 4.20 2.59 3.60 | 3.8 % [2.6-4.6] | at most 1.5 % | FAIL |
| Collision belt width, median (ground raised 100 m or more) | 2,783 km | 2630 3102 2913 2488 | 2,800 km | at most 800 | FAIL |
| Collision belt elongation, median | 2.19 | 2.37 2.18 1.97 2.24 | 2.2 | at least 4 | FAIL |
| Collision length, onset to weld or parting, median | 2 Myr (one step) | 2 2 2 2 | one step | 15-50 Myr | FAIL |
| Collision length, 90th percentile | 2 Myr | 2 2 2 2 | | at most 120 | pass |
| Peak uplift per collision, 90th ÷ 10th percentile | 1.00 | 1 1 1 1 | 1,400 m, all | at least 2 | FAIL |
| Collisions per 100 Myr | 10.0 | 9.4 9.9 9.2 11.6 | 10 [9-12] | 2-6 | FAIL |
| Continental crust share, lowest step | 0.40 | 0.39 0.40 0.42 0.37 | | at least 0.37 | pass |
| Continental crust share, highest step | 0.48 | 0.45 0.49 0.48 0.48 | | at most 0.45 | FAIL |
| Active share of the continental margin (after the first 100 Myr) | 0.30 | 0.31 0.28 0.32 0.29 | 0.30 [0.28-0.32] | 0.25-0.40 | pass |
| Ocean floor, 90th percentile age, end of the run | 235 Myr | 228 269 233 208 | 208-269 | at most 230 | FAIL |

Reported, not judged (the design's section 2.2 figures and what a force-balance model will be
judged on):

| Measure | Value | Runs | Design |
|---|---|---|---|
| Steps with one plate above 30 % | 83.0 % | 81.4 85.8 81.2 83.6 | 83 % |
| Largest plate, worst step | 0.90 | 0.93 0.93 0.88 0.84 | 0.90 [0.84-0.93] |
| Plates alive, median | 12.5 | 14 12 12 12 | |
| After a rift splits a plate above half the planet, until one is above half again | 17 Myr | 16 14 22 16 | 17 [14-22] |
| Rift-born plates that died: median lifetime | 18.5 Myr | 20 16 16 22 | 16-22 |
| Plate deaths by welding / absorption of a tiny plate / subducted away, per run | 100 / 48.5 / 2.25 | | 100 / 49 / 2 |
| Welds in which the dying plate is the larger | 49.9 % | 53.2 49.5 43.5 53.4 | 40-62 of 92-116 (43-53 %) |
| Ground one collision lifts by more than 10 m, median / most | 3.15 % / 22.6 % | 2.91-3.70 / 14.8-27.6 | 3.2 % / 23 % [15-28] |
| Collision belt length, median | 7,062 km | 6705 8075 7044 6423 | 7,100 km |
| Collisions whose belt is at least 3 times longer than wide | 24.5 % | 25.5 23.2 21.7 27.6 | 25 % |
| Contact age at the collision, median / 90th percentile | 7.5 / 25.1 Myr | 8 8 6 8 / 27 20 30 23 | 8 / 25 |
| What the collision counter read, median | 350 km | 344 353 351 353 | 350 km |
| Real shortening at the collision, median / 90th percentile (seed 1) | 87.4 / 269 km | mean of seeds: 78.3 / 240 | 87 / 269 (seed 1) |
| Continental crust at the end of the run | 0.42 | 0.44 0.42 0.45 0.38 | 0.38-0.45 |
| Land, median share over the run | 0.31 | 0.30 0.29 0.34 0.29 | |
| Ocean floor, 90th percentile age, mean after the first 100 Myr | 200 Myr | 190 223 193 197 | |
| Plate speed, median over plates and steps | 72.6 mm/yr | 72.7 73.5 72.3 71.8 | |
| Speed of plates with a slab on 20 % or more of their boundary / on less | 72.2 / 51.3 mm/yr | | |
| Slab-attached ÷ other plate speed | 1.41 | 1.44 1.38 1.50 1.33 | Earth: 3 to 4 (Forsyth and Uyeda) |
| Correlation of plate speed with slab share of the boundary | 0.48 | 0.48 0.44 0.55 0.43 | |
| Slab share of the boundary, mean over plates and steps | 0.47 | 0.46 0.51 0.48 0.45 | |
| Speed of mostly oceanic / mostly continental plates | 70.6 / 47.1 mm/yr | | |

## Does it agree with the design?

Every figure the design measured is reproduced inside its stated range, including the ones that
come from the recording rather than from per-step plate counts (collision shapes, contact age,
counter, real shortening, rift-born fates, mega-plate regrowth). Where the design gives one seed
(real shortening 87 and 269 km, seed 1) seed 1 gives 87.4 and 269 km.

Three things the design's table does not settle, which the calibration task should decide:

1. **Continental crust "every step".** Section 6 asks for 0.37-0.45 at every step. The design's
   "Now" (0.38-0.45) is the value at the end of the run, which agrees here (0.38 0.42 0.44 0.45),
   but the highest step of a run is 0.45-0.49, so the check, which takes the target literally,
   fails its upper end on master. The lower end passes (0.37 in seed 4, exactly at the limit).
2. **Ocean floor age.** Read as the end-of-run 90th percentile, as the plate model's summary line
   prints it (`ocean_age_p90_myr`). The mean over the run after 100 Myr is 200 Myr, inside the
   limit. The mean of seeds, 235 Myr, is above 230 because of seed 2 (269).
3. **Collision length.** Today a collision is the step it fires in, so its length is one step
   (2 Myr). The new collision state of task 2 must keep `start_myr` and `end_myr` meaning first and
   last step of the collision (see `plates/dynamics.rs`).

## What the force-balance measures say today

Slab-attached plates move 1.4 times faster than the rest (Earth: 3 to 4), the correlation of speed
with slab share is 0.48, and mostly continental plates move 47 mm/yr against 71 for oceanic ones.
Slab share is an estimate: the slab front length of a plate (`subduction::Pull::front_km`) over its
boundary length counted in sample spacings (`PlateStep::boundary_km`); both are counts of boundary
samples, not true lengths, so read changes in it, not its level.
