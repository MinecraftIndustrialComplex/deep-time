# P1-17 hypsometry recalibration: run plan (2026-10-01)

Working plan for the recalibration Ben asked for (decisions log 2026-10-01). It replaces the "Next"
line of the WIP build-log entry in STATUS.md. Delete this file when the task lands; the results go
into STATUS.md.

## Targets

Earth (GMT earth_relief 30′, SRTM15+, cos φ-weighted; `tools/review/p1-17-hyps-plot.py` recomputes it):

| figure | Earth | master today (6 runs, 500 Myr) | accept |
|---|---|---|---|
| land share | 0.289 | 0.450 [0.40–0.52] | 0.27–0.33 mean |
| land median | 427 m | 1,323 m | 400–700 m mean |
| land p90 | 2,022 m | 4,045 m | 1,700–2,600 m |
| land above 2 km | 0.102 | 0.343 | 0.07–0.15 |
| sea median depth | 4,062 m | 2,738 m | ≥ 3,500 m |
| sea below −6 km (share of planet) | 0.003 | 0.048 | ≤ 0.01 |
| mean surface temperature | 14–15 °C | 6.5 °C | 12–16 °C |
| glaciated land | 0.10 | 0.251 | 0.05–0.15 |
| Köppen E / B | 0.13 / 0.30 (model on Earth's relief 0.23 / 0.26) | 0.42 / 0.20 | E 0.15–0.28, B 0.20–0.30 |
| active margin share | ≈ 0.3 | 0.31 | 0.25–0.35 (must not move) |
| ground under arc cones | 1.2–4 km | 834 m | ≥ 1,200 m if the land balance allows |

The climate is checked, not fitted (the ice branch and P1-26 change it afterwards).

## What is known (STATUS.md, 2026-10-01 WIP entry)

- The land is too high first; cold, ice and the extra land follow from it. Fixed CO₂ made it colder.
- Ice protection (K = 0 under ice) is worth ~280 m of the median. `surface.glacial.strength` stays 1
  (Ben, 2026-09-30), so it is compensated elsewhere, not switched off.
- Cold planets lose 30–400 m of sea level to ice and expose their shelves: land 0.47–0.52 there.
- Margin uplift and progradation dry most of the initial shelf within 10 Myr.
- Collision surges are the largest suspected uplift source (33 per run, most at the 4 km cap, mean
  lift ⅓ of the peak over up to 2,000 km) and with isostasy 0.8 each km of uplift takes 5 km of
  erosion to remove. Not yet measured.
- Cortial's ocean law is 0.9 km too shallow at 10 Myr and 1.6 km too deep at 200 Myr; the new
  `tectonics.oceanic_floor_elevation_m` (−5,600 m, damping 0.215 mm/yr) follows GDH1.

## Machine budget

32 vCPUs shared with about five other agents: my share is 6–8 vCPUs. Default `J=4 T=2` (8 threads);
`J=6 T=2` only while the 1-minute load is under 16; never more than 12 threads. A run took 588 s at
2 threads on the quiet M6; assume ~12 min here, so about 20 runs per hour. Runs are niced
(`nice -n 10`). Scratch under `<scratch>/deeptime-runs/p1-17-hyps/`, deleted at the end.

All runs: `earthlike_quick_16k`, 500 Myr, `tools/review/p1-17-hyps.sh`. Screening uses 4 runs per
variant (seeds 1, 2 × supercontinent, scattered); anything that goes into a preset is confirmed on
6 (seeds 1–3) and the final set on 10 (seeds 1–5).

## Stages

**0. Setup (≈ 40 min, 6 runs).** Build master's CLI and keep it as `deeptime-base`; build this
branch as `deeptime-new` (the floor parameter defaults to Cortial's, so unset it is master's physics:
verify with one 100-Myr run that the hypsometry line is identical). Re-run the baseline (6 runs; the
curves were lost with the Mac folder). Build a3c63fe, 6070e9e and bd1cc0f into one shared target dir.

**1. Drivers (≈ 3 h, 52 runs).** One change each against the baseline, 4 runs:

| label | change | question |
|---|---|---|
| nocord | `tectonics.cordillera_uplift_mm_yr=0` | what the arc ranges cost |
| col2 | `tectonics.collision_uplift_max_m=2000` | collision plateaus |
| sub3 | `tectonics.subduction_uplift_mm_yr=0.3` | Andean band |
| novolc | `volcanoes.density=0` | cones |
| k4 | `surface.k_spl=4e-9` | erosion efficiency |
| ce | `tectonics.continent_elevation_m=[50,600]` | memory of the start |
| fill40 | `surface.marine_fill_depth_m=40` | shelf fill to sea level |
| eus0 | `climate.glacio_eustasy=0` | ice-exposed shelves |
| sl50 | `surface.sea_level.base_m=50` | datum |
| gdh | floor −5,600, damping 0.215 | ocean depth; side effects on land |
| at-a3c63fe, at-6070e9e, at-bd1cc0f | old builds (`STAGE=surface` for the first two) | which merge moved it |

Order: col2, k4, nocord, gdh first (they decide stage 2), then the rest, then the bisection.
Output: the driver table for STATUS.md (noglac and nocc are already measured).

**2. Calibration (≈ 3 h, ~50 runs).** Two rounds of 4-run variants, then one 6-run confirmation.
Fixed from the start: the GDH1 floor if `gdh` leaves the land figures within noise. Round 1 is a
small grid on the two strongest levers of stage 1, expected to be K and the collision cap:
K ∈ {3e-9, 4e-9, 6e-9} × cap ∈ {4,000, 2,500, 1,500} minus the corners already run (7 variants).
Round 2 takes the best cell and adds, one at a time: cordillera 0.3 and 0.4 (with
`cordillera_height_m` 3,000), and whichever of fill40 / sl50 / shelf share 0.35 stage 1 showed is
needed if the land share is still above 0.33 (≤ 5 variants). Preference order when two sets fit
equally: fewer changed parameters, values with a source, K before uplift constants, no change to
`isostasy` (Airy) or `glacial.strength`.

**3. Final checks (≈ 3 h).** The chosen set written into the four `pipeline_*` presets and synced:
- 10 runs (seeds 1–5 × both starts), 500 Myr: the before/after table and the plots;
- one 1,000-Myr run per start (supercontinent seed 1, scattered seed 2) for drift after 500 Myr;
- `earthlike_full_16k` and `earthlike_quick_32k`, one 500-Myr run each, before and after
  (`J=1 T=6` for the full one, ~1 h);
- run time: base and final side by side, same seed, `T=2`, twice; stage timings from the `timings` line.

**4. Landing on the branch.** Editor field `abyssal_depth` and its lang key with the preset key;
`knob_extremes.py --only abyssal_depth`; hints that quote defaults (`erosion_strength`); the
test planet `planet_world_4k.json` is left as it is unless the coordinator wants it synced (it would
move the GameTest floors); `sync_builtin_presets.py`; cargo test, clippy, fmt, deny; `./gradlew build
test`; plots pulled to review/out/p1-17-hyps/; the STATUS.md entry with every changed key listed for
the merge with the ice branch.

## Decisions that will go to Ben (not asked yet)

1. Cordillera strength: keep 0.2 or raise, with the measured cost in land share.
2. The GDH1 ocean floor as the Earth-like default (new setting) or left at Cortial's law.
3. If only a sea-level datum or a larger flooded-shelf share brings the land to 0.29: accept that
   knob or accept ~0.33 land.
