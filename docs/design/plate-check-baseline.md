# plate-check: baseline on master (2026-10-02)

`deeptime plate-check` on the mean of seeds 1-4 of `earthlike_quick_16k`, supercontinent start,
1000 Myr, 2 Myr steps, `deeptime stage volcanoes` (the coupled pipeline), on master 38ca6b0 plus
the recording of task 1 (`docs/design/plate-collisions-and-welding.md`, section 7) and its additions
of task 1b (`docs/design/plate-force-balance.md`, section 7, "More for `plate-check` to report").
The check **fails** on this baseline: 10 of its 36 ranged targets are met (the 19 targets of the
collisions design: 4; the 17 added by the force-balance design that today's model can be measured
on: 6), and 8 more rows wait for physics that is not built (they say "no data" and do not fail).
That is the point of a baseline: it is what tasks 2-5 must move. The recording changes nothing a
run computes (a test shows it), and the plates are exactly those of the first baseline (fa09e8b):
every figure of the first table below that did not change its definition is the same to the digit.

**Update 2026-10-02 (tasks 3 and 4):** the last section, "Tasks 3 and 4", has the tables before and after the collision
belts, the resistance and the weld on stall: 28 of 45 targets met.

**Update 2026-10-02:** the section "Added 2026-10-02: the boundary mix and the arc heights" below
adds five judged and many reported rows, and says how the rows above moved on master 1eac09f (general
flexure on). The tables above are the baseline as first measured (master 38ca6b0).

Reproduce (about 20 minutes a run at 2 threads on the test VM, two runs at a time):

    tools/remote/mac.sh -- tools/remote/detach.sh start pc tools/review/plate-check.sh 1000 2,3,4 supercontinent base
    tools/remote/mac.sh -- tools/remote/detach.sh start pc1 tools/review/plate-check.sh 1000 1 supercontinent base:structure.deformation.density=0
    tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait pc

**Seed 1 needs `structure.deformation.density=0`.** On master the structure stage (P2-01 stage B)
panics on seed 1 of this run ("a history holds at most 128 events, the base stratigraphy
included", `deeptime-noddy/src/history.rs`, on a rayon thread of the structure stage). The
structure stage reads the plates and writes nothing they read, so switching its folds and faults
off cannot change the plate record; this was checked on seed 2, whose measures are all
identical with and without it (`diff` of the two `plate-check --json` files differs only in the
checkpoint's path). Seeds 2-4 ran with the structure stage on. The panic is not part of this task and
is reported to the coordinator.

The script prints the per-run tables and the mean table below, and `deeptime plate-check --run <ck>
[--run <ck> ...] [--json out] [--targets file] [--param name=value ...]` reads any checkpoint,
world-data folder or manifest. Target ranges live in `MEASURES` in
`crates/deeptime-core/src/plates/check.rs`; a file of `{"key": [lo, hi]}` moves them (null: open
end; `"key": null`: report only; a `"params"` object sets the definitions below). The definitions
the table was made with are printed under its header:

- a supercontinent is **assembled** when the largest connected piece of continental crust holds 70 %
  or more of the continental crust, and stays so until it holds under 50 % (`assembled_share`,
  `dispersed_share`; hysteresis, so a flicker around 70 % is not a break-up);
- a **mega-plate** is a plate above 40 % of the visible planet (`mega_plate_share`);
- speeds, rotation changes and the pole tests count plates of 30 or more visible samples
  (`min_plate_samples`, as the design's trace did); the RMS and the guard count every plate;
- sea-floor measures start after the first 100 Myr (`warmup_myr`), the continental crust's lowest and
  highest step after the first 10 Myr (`start_up_myr`), and "still separate" is 50 Myr after the
  rift (`rift_window_myr`).

## Against the targets

Value is the mean of the four runs; "runs" are the runs' own values (seeds 1, 2, 3, 4). The design's
"Now" column is the number the designs measured with the throwaway trace on the same four runs (a
bracket is its range over the seeds), or the review's figure where the force-balance design quotes
that. "Agrees" below means the value is inside the design's figure or its range.

### From the collisions design (section 6)

| Measure | Value | Runs | Design "Now" | Target | Result |
|---|---|---|---|---|---|
| Largest plate, median share of the planet | 0.58 | 0.56 0.66 0.61 0.50 | 0.58 [0.50-0.66] | 0.15-0.35 | FAIL |
| Steps with one plate above 50 % | 58.7 % | 58.8 68.8 57.8 49.6 | 59 % [50-69] | at most 10 | FAIL |
| Plates above 5 % of the planet, median | 3.75 | 4 3 4 4 | 3.75 | 5-9 | FAIL |
| Median plate lifetime (plates that died) | 45 Myr | 46 40 56 38 | 45 [38-56] | at least 80 | FAIL |
| Plates dead within 20 Myr | 33.4 % | 27.3 38.2 33.8 34.4 | 33 % | at most 15 | FAIL |
| Rift-born plates alive after 50 Myr | 18.7 % | 14.0 20.0 21.6 19.2 | 19 % [14-22] | at least 60 | FAIL |
| Rift-born plates welded back to their own rift family within 50 Myr | 41.9 % | 38.0 40.0 47.1 42.5 | about 40 % | at most 10 | FAIL |
| Land raised by one collision in one step, the most | 4.47 % | 5.19 4.88 2.86 4.96 | 4.5 % [2.9-5.2] | at most 0.5 | FAIL |
| Most land gained in any step (after the first 10 Myr) | 3.75 % | 4.61 4.20 2.59 3.60 | 3.8 % [2.6-4.6] | at most 1.5 | FAIL |
| Collision belt width, median (ground raised 100 m or more) | 2,783 km | 2,630 3,102 2,913 2,488 | 2,800 km | at most 800 | FAIL |
| Collision belt elongation, median | 2.19 | 2.37 2.18 1.97 2.24 | 2.2 | at least 4 | FAIL |
| Collision length, first to last step, median | 2 Myr (one step) | 2 2 2 2 | one step | 15-50 | FAIL |
| Collision length, 90th percentile | 2 Myr | 2 2 2 2 | | at most 120 | pass |
| Peak uplift per collision, 90th / 10th percentile | 1.00 | 1 1 1 1 | 1,400 m, all | at least 2 | FAIL |
| Collisions per 100 Myr | 10.0 | 9.4 9.9 9.2 11.6 | 10 [9-12] | 2-6 | FAIL |
| Continental crust share, lowest step (after the first 10 Myr) | 0.40 | 0.39 0.40 0.43 0.37 | | at least 0.37 | pass |
| Continental crust share, highest step (after the first 10 Myr) | 0.48 | 0.45 0.49 0.48 0.48 | | at most 0.45 | FAIL |
| Active share of the continental margin (after the first 100 Myr) | 0.30 | 0.31 0.28 0.32 0.29 | 0.30 [0.28-0.32] | 0.25-0.40 | pass |
| Ocean floor, 90th percentile age, mean over the run after the first 100 Myr | 200 Myr | 189 222 193 197 | 208-269 (end of run) | at most 230 | pass |

### Added by task 1b, from the force-balance design (section 7)

| Measure | Value | Runs | Design "Now" | Target | Result |
|---|---|---|---|---|---|
| Plate speed, area-weighted RMS (mean over steps) | 57.0 mm/yr | 58.4 55.9 53.5 60.0 | about 65 (an estimate) | 35-55 | FAIL |
| Plate speed, 10th percentile of plate-steps | 40.1 mm/yr | 39.1 41.6 39.3 40.4 | 41 | at most 20 | FAIL |
| Plate speed, median of plate-steps | 72.6 mm/yr | 72.7 73.5 72.3 71.8 | 71 | 30-60 | FAIL |
| Plate speed, 90th percentile of plate-steps | 85.1 mm/yr | 84.4 85.7 85.0 85.4 | 83 | 70-110 | pass |
| Median speed, plates under 20 % continental / plates over 50 % | 1.64 | 1.76 1.65 1.69 1.48 | 1.6 | at least 2 | FAIL |
| Median speed, plates with 20 % or more of the edge on slab / plates under 5 % | 1.61 | 1.67 1.59 1.75 1.45 | 1.6 | at least 2.5 | FAIL |
| Speed against slab share of the edge, rank correlation | 0.43 | 0.43 0.37 0.49 0.43 | 0.47-0.59 | at least 0.65 | FAIL |
| Speed against plate area, rank correlation | -0.30 | -0.20 -0.34 -0.43 -0.21 | -0.36 to -0.46 | -0.4 to 0.2 | pass |
| Plate-steps above 150 mm/yr | 0 % | 0 0 0 0 | 0 (capped) | at most 0.5 | pass |
| Steps on which the speed guard binds (of 500) | 369 | 353 375 385 362 | not counted | 0 | FAIL |
| Change of a plate's rotation per step, median | 4.87 % | 4.86 4.88 4.91 4.85 | 5 % | at most 8 | pass |
| Change of a plate's rotation per step, 90th percentile | 8.95 % | 9.02 8.93 8.71 9.15 | 9 % | at most 20 | pass |
| Ocean floor, median age (mean over the run after the first 100 Myr) | 72.3 Myr | 67.7 78.9 70.7 71.8 | 56-100 | 50-80 | pass |
| Ocean floor older than 180 Myr | 13.2 % | 11.3 16.2 12.6 12.8 | 12 % (the review) | at most 10 | FAIL |
| Ocean floor older than 250 Myr | 5.11 % | 3.83 6.73 4.73 5.14 | 5-12 % | at most 3 | FAIL |
| Supercontinent tenure, median | 44.8 Myr | 49 36 53 41 | about 360 (the review's run) | 100-300 | FAIL |
| Assembly-to-assembly period, median | 76.0 Myr | 91 54 75 84 | about 650 (the review's run) | 300-600 | FAIL |

### Waiting for physics that is not built

These rows report "no data" and do not fail the check. The record that feeds them exists and is
empty (`StepRec::forces`, `StepRec::slabs_km`, `Dynamics::belts` in `plates/dynamics.rs`); the task
that builds the physics fills it, and from then on the row is judged (a test shows the rows lighting
up). Target ranges are the design's.

| Measure | Target | Waits for |
|---|---|---|
| Convergence 20 Myr after a collision's onset / at onset, median | 0.2-0.6 | collision state and belts (design tasks 2-3) |
| Wedge on the incoming side, width, median | 100-350 km | belts (task 3) |
| Foredeep width, median | 150-450 km | belts (task 3) |
| Foredeep depth of fill, median | 2,000-7,000 m | belts (task 3) |
| Share of a plate's driving torque that is slab pull, median | reported | the force budget (task 5) |
| Mean in-plane stress of a plate above 40 %, median (tension positive) | reported | the force budget (task 5) |
| Steps in which a plate above 40 % is in tension | reported | the force budget (task 5) |
| Slab length per contact, median | reported | slab tracking (task 5) |

Reported, not judged (the designs' section 2.2 figures and what a force-balance model will be
judged on):

| Measure | Value | Runs | Design |
|---|---|---|---|
| Steps with one plate above 30 % | 83.0 % | 81.4 85.8 81.2 83.6 | 83 % |
| Largest plate, worst step | 0.90 | 0.93 0.93 0.88 0.84 | 0.90 [0.84-0.93] |
| Plates alive, median | 12.5 | 14 12 12 12 | |
| After a rift splits a plate above half the planet, until one is above half again, median | 17 Myr | 16 14 22 16 | 17 [14-22] |
| Rift-born plates that died: median lifetime | 18.5 Myr | 20 16 16 22 | 16-22 |
| Plate deaths by welding, per run | 100 | 94 99 92 116 | 100 |
| Plate deaths by absorption of a tiny plate, per run | 48.5 | 44 56 51 43 | 49 |
| Plate deaths subducted away, per run | 2.25 | 1 2 2 4 | 2 |
| Welds in which the dying plate is the larger | 49.9 % | 53.2 49.5 43.5 53.4 | 43-53 % |
| Ground one collision lifts by more than 10 m, median | 3.15 % | 2.91 3.70 3.37 2.64 | 3.2 % |
| Ground one collision lifts by more than 10 m, the most | 22.6 % | 27.6 25.7 14.8 22.2 | 23 % [15-28] |
| Collision belt length, median | 7,062 km | 6,705 8,075 7,044 6,423 | 7,100 km |
| Collisions whose belt is at least 3 times longer than wide | 24.5 % | 25.5 23.2 21.7 27.6 | 25 % |
| Contact age at the collision, median | 7.5 Myr | 8 8 6 8 | 8 |
| Contact age at the collision, 90th percentile | 25.1 Myr | 27.4 20 30 23 | 25 |
| What the collision counter read, median | 350 km | 344 353 351 353 | 350 km |
| Real shortening at the collision, median | 78.3 km | 87.4 76.1 82.2 67.7 | 87 (seed 1) |
| Real shortening at the collision, 90th percentile | 240 km | 269 216 209 267 | 269 (seed 1) |
| Continental crust at the end of the run | 0.42 | 0.44 0.42 0.45 0.38 | 0.38-0.45 |
| Continental crust share, lowest / highest step, all steps | 0.40 / 0.48 | 0.39 0.40 0.42 0.37 / 0.45 0.49 0.48 0.48 | |
| Land, median share over the run | 0.31 | 0.30 0.29 0.34 0.29 | |
| Ocean floor, 90th percentile age, last step | 235 Myr | 228 269 233 208 | 208-269 |
| Plate speed, median over all plates and steps | 72.6 mm/yr | 72.7 73.5 72.3 71.8 | |
| Mean speed of plates with a slab on 20 % or more of their boundary / on less | 72.2 / 51.3 mm/yr | 71.9 72.7 72.9 71.4 / 49.9 52.8 48.7 53.8 | |
| Slab-attached / other plate speed (mean speeds) | 1.41 | 1.44 1.38 1.50 1.33 | Earth: 3 to 4 (3.8 by means from Forsyth & Uyeda 1975 Table 1; "~4 times" Conrad & Lithgow-Bertelloni 2004; 3.0 Zahirovic et al. 2015; WP-13a) |
| Correlation (Pearson) of plate speed with slab share of the boundary | 0.48 | 0.48 0.44 0.55 0.43 | |
| Slab share of the boundary, mean over plates and steps | 0.47 | 0.46 0.51 0.48 0.45 | |
| Mean speed of mostly oceanic / mostly continental plates | 70.6 / 47.1 mm/yr | 70.5 71.6 70.9 69.5 / 44.7 47.9 45.3 50.4 | |
| Plate-steps turning about a pole within 30 degrees of their own centroid | 6.27 % | 6.14 5.97 4.42 8.55 | 0.5-2 % ("near themselves") |
| Plate-steps turning about a pole on the plate itself | 10.9 % | 10.1 10.4 9.61 13.3 | |
| Plate-steps on which the speed guard binds | 728 (1.5 a step) | 769 698 637 809 | |
| Connected pieces of continental crust (1 % or more of it), median over steps | 3.5 | 3 4 3 4 | |
| Largest continental piece, share of the continental crust, median / lowest step | 0.78 / 0.25 | 0.83 0.74 0.89 0.66 / 0.27 0.22 0.29 0.24 | |
| Supercontinent assemblies in the run (one at the start counts) | 10.2 | 9 12 9 11 | |
| Steps with a supercontinent assembled | 73.7 % | 73.6 74.6 82.8 63.8 | |
| Rifts of a plate above 40 % of the planet, per run | 18.8 | 18 18 16 23 | |
| A plate above 40 %: time above it until a rift splits it, median | 17.8 Myr | 21 19 17 14 | |
| Of those rifts: fragments all still separate 50 Myr later (strict) | 6.2 % | 6.3 11.8 6.7 0 | |
| Of those rifts: two plates or more still hold the crust 50 Myr later (not healed whole) | 9.4 % | 0 17.6 20 0 | |

## Added 2026-10-02: the boundary mix and the arc heights

Two measures Ben asked for after a workbench run showed, 28 Myr after a supercontinent start,
nearly every boundary between ocean plates as a subduction zone with a continuous high volcanic arc
("Measure now, fix in the plate track", decisions log 2026-10-02). They are measurement only
(`plates/dynamics.rs`, record version 3: `StepRec::mix_km`, `arc_oceanic`, `arc_continental`); the
plate work (force-balance motion, the belts) is to be judged on them. Definitions: ledger PLT-220
and PLT-221, science page `docs/science/plates.md`. Run on master 1eac09f plus this task, the same
setup as above (`earthlike_quick_16k`, supercontinent, 1000 Myr, seeds 1-4, `stage volcanoes`;
seed 1 again with `structure.deformation.density=0` for the structure-stage panic, which cannot
change the plate record). Values are the mean of the four runs; "runs" are seeds 1, 2, 3, 4.
Reproduce:

    tools/remote/mac.sh -- tools/remote/detach.sh start m1 env R=<scratch> tools/review/plate-check.sh 1000 1 supercontinent base:structure.deformation.density=0
    tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start m2 env R=<scratch> tools/review/plate-check.sh 1000 2 supercontinent base
    (then seeds 3 and 4 the same way, two runs at a time; a last call with seeds 1,2,3,4 prints the mean table)

**What is counted.** Boundary length is one sample spacing (about 70-80 km on the Earth-normalized
sphere) per boundary sample: a subduction front counts a whole spacing, once (the overriding side,
as `PlateStep::front_km` does), a spreading or other boundary half a spacing from each side. Kinds:
subducting (an oceanic slab under oceanic crust, "ocean-ocean", or under continental crust,
"ocean-continent"), collision (continental crust on the subducting side), spreading (ridge, and rift
when a side is continental), other (transform, or converging with no contact yet). Arc cells are the
visible samples that received arc magma in the step (the code's own `ARC` flag); an oceanic arc is
one whose arc crust was oceanic when the magma came, a continental arc the rest. Elevations are the
plate model's, m above its sea datum, before the later stages (the volcano stage's cones and the
surface stage's amplification are not in them).

### Judged against Earth

| Measure | Value | Runs | Earth and source | Target | Result |
|---|---|---|---|---|---|
| Subducting share of the boundary length, mean over the run after the first 100 Myr | 55.9 % | 55.5 54.9 56.9 56.4 | 19.7 %: subduction zones 51,310 km of 260,487 km (Bird 2003, Table 3; PLT-213) | 15-25 % (PLT-214) | FAIL |
| Oceanic arc cells above sea level, share of the arc cells | 14.4 % | 13.6 12.4 17.5 14.3 | 23 % of the active Mariana arc centres (6 of 26; Baker et al. 2008, abstract), at least 30 % of 696 arc volcanoes submarine (de Ronde et al. 2003, abstract); PLT-215 | 10-35 % | pass |
| Oceanic arc crest (90th percentile elevation) | 312 m | 248 153 523 324 | submarine arc summits and vents 250-1,660 m down (de Ronde et al. 2003; PLT-216), highest island-arc cones about 1 km (from memory; PLT-217) | -1,700 to 1,000 m | pass |
| Continental arc cells, median elevation | 1,670 m | 1,650 1,574 1,760 1,696 | Cascades basement 1.2-2 km (Hildreth 2007); Andean arc on a 4.5 km plateau (Gregory-Wodzicki 2000); PLT-218 | 1,200-4,500 m | pass |
| Continental arc crest (90th percentile elevation) | 2,874 m | 2,881 2,770 2,958 2,889 | a dozen Cascade cones above 3 km, two above 4 km (Hildreth 2007); Andean stratovolcanoes above 6 km (from memory); PLT-219 | 2,000-6,000 m | pass |

The check now meets 13 of its 41 ranged targets (above: 10 of 36). Of the 5 new ones, 4 pass; the
one that fails is the boundary mix. The ranges are the agent's choice around the Earth anchors
(ledger rows say which part of each is read, abstract-only or from memory); the Earth anchors for
the oceanic arc are counts of volcanoes and vent depths, the model's are cells and elevations, so
the comparison is loose.

### Reported, not judged

| Measure | Value | Runs | Earth (Bird 2003, Table 3) |
|---|---|---|---|
| Plate boundary, total length per step | 340,885 km | 344,218 339,321 342,808 337,195 | 260,487 km |
| Subducting length per step | 191,174 km | 191,474 185,715 195,389 192,117 | 51,310 km |
| Spreading (ridge and rift), share of the length | 39.5 % | 39.6 40.2 38.6 39.6 | 36.4 % (OSR 25.8 + CRB 10.5) |
| Collision (continental crust on the subducting side), share | 3.25 % | 3.53 3.38 3.46 2.62 | 8.8 % (continental convergent) |
| Transform or converging with no contact yet, share | 1.32 % | 1.35 1.56 1.05 1.33 | 28.4 % transform (OTF + CTF), 6.7 % oceanic convergent without arc |
| Ocean-ocean share of the subducting length | 50.1 % | 47.6 51.3 49.1 52.3 | not split by Bird |
| Oceanic arc cells, median elevation | -2,146 m | -2,298 -2,301 -1,822 -2,162 | (the band includes flanks, so not judged) |
| Oceanic and continental arc cells per step, mean | 3,000 and 2,571 | 2,925 3,078 2,927 3,072 and 2,651 2,407 2,754 2,473 | |

**The first 50 Myr after the start** (the window in which the run Ben looked at showed the
problem), mean of the four runs; runs in brackets:

| Measure | First 50 Myr | After the first 100 Myr |
|---|---|---|
| Subducting share of the boundary | 49.9 % (48.7 52.3 47.2 51.4) | 55.9 % |
| Ocean-ocean share of the subducting length | 63.4 % (60.9 61.8 62.7 68.1) | 50.1 % |
| Spreading share | 48.0 % (48.8 46.3 50.3 46.5) | 39.5 % |
| Collision share | 0.73 % (0.84 0.63 0.69 0.76) | 3.25 % |
| Other share | 1.39 % (1.64 0.70 1.85 1.39) | 1.32 % |
| Oceanic arc cells above sea level | 23.6 % (22.2 28.2 22.8 21.2) | 14.4 % |
| Oceanic arc crest | 743 m (722 875 764 609) | 312 m |
| Continental arc crest | 2,719 m (2,694 2,765 2,794 2,624) | 2,874 m |

**What the numbers say.**

- The model's boundary is dominated by subduction: 56 % of its length against Earth's 20 %, and in
  kilometres 191,000 against 51,000 (3.7 times Earth) on a boundary that is only 1.3 times Earth's
  total. Its spreading share is close to Earth's (39.5 against 36.4 %), so the surplus of subduction
  stands where Earth has transform and oceanic-convergent boundaries (35 %); the model has almost no
  "other" (1.3 %). One cause is in the classification: the model's tests are signs only (a sample is
  divergent or converging), with no azimuth tolerance like Bird's +-20 degrees for strike-slip, so
  strike-slip and oblique boundaries become fronts or spreading. How much of the surplus is that and
  how much is real over-subduction was not separated.
- In the first 50 Myr the subducting share is already 50 %, 63 % of it ocean-ocean, so the ocean-ocean
  subduction Ben saw is in the record.
- The arc heights do not show snow-capped arcs between ocean plates in the plate model's own
  elevation: oceanic arcs are mostly submarine (14 % of cells emerge, 24 % in the first 50 Myr, a
  crest of 312 m, 743 m early) and continental arcs stand at 1.7 km median, 2.9 km crest. They are
  inside the Earth ranges above. What Ben saw may therefore come from stages that add to the plate
  elevation (the volcano stage's cones, the surface stage's amplification). That was not checked
  here.

### Other rows moved: master now has general flexure on

The baseline above (master 38ca6b0 plus tasks 1 and 1b) is not reproduced by master 1eac09f: the
flexure feature (on in the built-in presets) and other merges changed the plate runs themselves
(elevation feeds back into the plates). Not this task's doing. The rows that moved most, mean of
seeds 1-4, before then after: largest plate, median share 0.58 then 0.45; steps with one plate above
50 % 58.7 then 40.8 %; plates above 5 % 3.75 then 4.25; median plate lifetime 45 then 41.5 Myr;
rift-born plates alive after 50 Myr 18.7 then 24.1 %; welded back within 50 Myr 41.9 then 31.8 %;
land raised by one collision in one step 4.47 then 2.37 %; most land gained in a step 3.75 then
2.34 %; collisions per 100 Myr 10.0 then 11.3; continental crust, lowest step 0.40 then 0.29 (the
target of at least 0.37 now fails), highest step 0.48 then 0.45; land, median share 0.31 then 0.29;
supercontinent tenure 44.8 then 33.5 Myr; assembly period 76 then 69 Myr; plate speed RMS 57.0 then
59.6 mm/yr; sea floor older than 180 Myr 13.2 then 12.5 %, older than 250 Myr 5.1 then 4.2 %;
steps on which the speed guard binds 369 then 375. The old rows' targets met: 9 of 36 (10 before).
The earlier tables' wording and figures above are left as they were measured.

### Cost

The same 300 Myr plates run with and without the whole record (`DEEPTIME_NO_PLATE_DYNAMICS=1`),
both at once on the test VM, plates-stage seconds for 150 steps: seed 1 80.1 against 78.8 s (+1.6 %),
seed 2 91.5 against 89.2 s (+2.6 %). Task 1b's recording alone measured within 0.5 %, so the two
new measures cost about 1-2 % of the plates stage (10-15 ms a step at 16k samples; about 0.5 % of
a whole run, the plates stage being a quarter of it). The record grows by 16 numbers a step (`mix_km`,
two arrays of 5), about 50 KB of JSON per 1000 Myr run (seed 1: 1.05 MB in all).

## Does it agree with the designs?

Every figure the designs measured is reproduced inside its stated range, including the ones that
come from the recording rather than from per-step plate counts (collision shapes, contact age,
counter, real shortening, rift-born fates, mega-plate regrowth). Where the collisions design gives
one seed (real shortening 87 and 269 km, seed 1) seed 1 gives 87.4 and 269 km.

The force-balance design's "Now" column, which it measured with a trace on the same four runs:

- **Agrees:** speed percentiles (40 / 73 / 85 against 41 / 71 / 83), oceanic over continental speed
  (1.64 against 1.6), slab over free speed (1.61 against 1.6), rotation change per step (4.9 % and
  9.0 % against 5 % and 9 %), sea-floor age median (72 against 56-100), sea floor older than 180 Myr
  (13 % against 12 %) and 250 Myr (5.1 % against 5-12 %).
- **Close but outside the range:** rank correlation of speed with slab share, 0.43 (seeds 0.37-0.49)
  against 0.47-0.59, and with area, -0.30 (-0.20 to -0.43) against -0.36 to -0.46. Both are over
  plate-steps of 30 samples or more, pooled over the run; the trace may have pooled differently.
  The design's own caution applies: read changes in them, not their level.
- **Different quantity, as far as can be told:** the RMS speed "about 65" was an estimate from the
  percentiles ("plate-check will give it"); the number is 57. The 0.5-2 % of plate-steps "turning
  about a pole near themselves" is not defined in the design; the pole within 30 degrees of the
  plate's own centroid (a plate that mostly spins) is 6.3 %, and the pole anywhere on the plate is
  10.9 %. The design's open-loop columns (10-17 %) read as the second.
- **Different definition, and it matters:** the supercontinent tenure and period. The review
  quotes 360 and 650 Myr from one run, without a definition. By the definition above (70 % of the
  continental crust in one connected piece, until under 50 %) the baseline has about ten
  assemblies per run, each lasting 45 Myr and coming back after 76 Myr. That is the mega-plate
  fault measured directly: the supercontinent is above 40 % of the planet for a median 18 Myr
  before a rift splits it (19 such rifts per run). In 94 % of those rifts at least one fragment
  rejoins another within 50 Myr (only 6 % keep every fragment apart), and in 91 % the crust is
  back on one plate (9 % leave two or more plates holding it).
  The calibration task should read the 100-300 and 300-600 targets against this definition, or
  move the thresholds with `--param`.

## The three readings task 1 left open, settled

1. **Continental crust "every step".** Judged on the lowest and highest step *after the first
   10 Myr* (the start-up the land gain already leaves out), and the extremes over every step are
   reported beside them. The first baseline took the literal reading (every step including the
   first); the number barely changes (highest step 0.48 either way), so the start-up is not the
   cause: the crust really does reach 0.45-0.49 in the middle of the runs (end values 0.38-0.45
   agree with the design). The upper end of 0.45 fails on master and stays failing; the lower end
   passes (0.37 in seed 4, exactly at the limit). This reading catches mid-run excursions, which an
   end-of-run value would hide, and ignores the start-up, which is the land-jump task's.
2. **Ocean-floor age.** Judged as the mean over the run, after the first 100 Myr, of the
   per-step 90th percentile (`ocean_age_p90_mean_myr`, at most 230); the last step's value
   (`ocean_age_p90_myr`) is reported beside it. The end-of-run value is one noisy snapshot (208 to
   269 across the four seeds, so a ranged measure on it fails or passes on the seed), while the run
   mean moves smoothly with the knobs a calibration turns and is the same number the design calls
   "mean after the first 100 Myr" (now 200, inside the limit). The mean is also taken from the
   per-step record, not from the 10 Myr histograms.
3. **Collision length.** From the collision's first step to its last, inclusive
   (`end_myr - start_myr + dt`). Counting from the contact's onset would mix the long approach
   (contact age at the collision: median 8 Myr) into the collision. `CollisionRec::start_myr` and
   `end_myr` are documented in `plates/dynamics.rs` as the first and last step of the collision
   (equal today, so the length is one step, 2 Myr), for the task that makes a collision a
   multi-step state.

## What else the baseline found

- **The speed guard binds in 74 % of the steps** (369 of 500; 1.5 plates a step). It is a cap of
  100 mm/yr on the *fastest point* of a plate, which is faster than the centroid speed the table
  uses, and the slab pull law drives many plates to it. The design raises the guard to 250 and
  wants it never to bind; the check fails the baseline on it, and will fail any run on which any
  plate is held by the cap (Ben's rule: the guard is only a reported guard).
- **The record and its cost.** Per 1000 Myr run the recording is 0.95-1.09 MB of JSON (task 1: 0.9),
  so it stays in the game's `manifest.json`. Recording costs nothing measurable: the same 300 Myr
  run with and without it (`DEEPTIME_NO_PLATE_DYNAMICS=1`), both at once on the test VM so they
  share its load, took 341.9 and 343.0 s of wall time and 539.3 and 542.2 s of CPU on seed 1, 310.4
  and 309.3 s and 488.3 and 488.8 s on seed 2 (recording first): within 0.5 %, inside the noise.
  Runs of different seeds or at different times on the shared machine differ by 10-20 %, which is
  why the pairs were run together.

## What the force-balance measures say today

Slab-attached plates move 1.4 times faster than the rest in mean speed (Earth: 3 to 4), the
rank correlation of speed with slab share is 0.43, and mostly continental plates move 47 mm/yr
against 71 for oceanic ones. A plate's rotation changes by a median 4.9 % a step and by 9 % at the
90th percentile, so the present rules are smooth; the design's open-loop balance changed it by 16-18
% and 62-70 %. Slab share is an estimate: the slab front length of a plate
(`subduction::Pull::front_km`) over its boundary length counted in sample spacings
(`PlateStep::boundary_km`); both are counts of boundary samples, not true lengths, so read changes
in it, not its level.

## Task 2: real shortening and the collision state (2026-10-02)

`earthlike_quick_16k`, supercontinent, 1000 Myr, `stage volcanoes`, 2 threads, on the test VM. "Before" is master
1eac09f (flexure on). "After" is the task 2 branch on 1eac09f with the built-in presets, where
`tectonics.collision_on_shortening` is off. "Switch on" is the same branch with it on (two seeds only). The before
column is read by master's own `plate-check`, because the branch cannot read master's checkpoints (the plate state
gained fields). Each cell is the mean, with the runs in brackets. "same" means equal to the last digit in every seed.

With the switch off the planets are master's. Only the measures that depend on when a collision starts move: a
collision now starts when continental crust first converges at the front, not when the old count fires. The rows
below the `n/a` are new.

| Measure | Unit | Before: master 1eac09f (seeds 1 2 3 4) | After: this branch, switch off | Switch on (seeds 1 2) | Target |
|---|---|---|---|---|---|
| Largest plate, median share of the planet | share | 0.45 (0.48 0.47 0.43 0.41) | same | 0.46 (0.45 0.47) | 0.15–0.35 |
| Steps with one plate above 50 % | % | 40.8 (42.4 47.4 39.6 33.6) | same | 44.7 (42.6 46.8) | ≤ 10.0 |
| Plates above 5 % of the planet, median | plates | 4.25 (4.00 4.00 4.00 5.00) | same | 3.50 (3.00 4.00) | 5.00–9.00 |
| Median plate lifetime (plates that died) | Myr | 41.5 (40.0 40.0 36.0 50.0) | same | 63.0 (69.0 57.0) | ≥ 80.0 |
| Plates dead within 20 Myr | % | 34.6 (36.4 32.9 37.4 31.5) | same | 21.2 (20.1 22.2) | ≤ 15.0 |
| Rift-born plates alive after 50 Myr | % | 24.1 (26.4 27.7 18.8 23.7) | same | 23.6 (23.2 24.0) | ≥ 60.0 |
| Rift-born plates welded back to their own rift family within 50 Myr | % | 31.8 (35.8 23.1 36.2 32.2) | same | 29.6 (39.3 20.0) | ≤ 10.0 |
| Land raised by one collision in one step, the most | % of planet | 2.37 (3.00 2.10 2.96 1.40) | same | 1.29 (1.33 1.26) | ≤ 0.50 |
| Most land gained in any step (after the first 10 Myr) | % of planet | 2.34 (2.05 2.77 3.17 1.37) | same | 1.52 (2.30 0.74) | ≤ 1.50 |
| Collision belt width, median (ground raised 100 m or more) | km | 2604 (2958 2597 2554 2306) | same | 2519 (2562 2475) | ≤ 800 |
| Collision belt elongation, median | ratio | 2.05 (1.88 2.03 2.24 2.05) | same | 2.30 (2.23 2.36) | ≥ 4.00 |
| Collision length, first to last step of the collision, median | Myr | 2.00 (2.00 2.00 2.00 2.00) | 6.00 (6.00 6.00 6.00 6.00) | 10.0 (10.0 10.0) | 15.0–50.0 |
| Collision length, first to last step, 90th percentile | Myr | 2.00 (2.00 2.00 2.00 2.00) | 10.9 (10.2 11.4 12.0 10.0) | 20.8 (19.6 22.0) | ≤ 120 |
| Peak uplift per collision: 90th ÷ 10th percentile | ratio | 1.00 (1.00 1.00 1.00 1.00) | same | 1.00 (1.00 1.00) | ≥ 2.00 |
| Collisions per 100 Myr | per 100 Myr | 11.3 (11.0 12.4 11.4 10.3) | same | 7.40 (7.70 7.10) | 2.00–6.00 |
| Continental crust, share of the planet, lowest step (after the first 10 Myr) | share | 0.28 (0.27 0.25 0.32 0.30) | same | 0.35 (0.37 0.33) | ≥ 0.37 |
| Continental crust, share of the planet, highest step (after the first 10 Myr) | share | 0.45 (0.44 0.45 0.45 0.45) | same | 0.46 (0.45 0.46) | ≤ 0.45 |
| Active share of the continental margin (after the first 100 Myr) | share | 0.31 (0.32 0.29 0.31 0.31) | same | 0.34 (0.33 0.35) | 0.25–0.40 |
| Ocean floor, 90th percentile age, mean over the run after the first 100 Myr | Myr | 194 (188 206 188 195) | same | 182 (181 184) | ≤ 230 |
| Plate speed, area-weighted RMS (mean over steps) | mm/yr | 59.6 (58.9 60.6 57.4 61.3) | same | 57.6 (57.7 57.4) | 35.0–55.0 |
| Plate speed, 10th percentile of plate-steps (plates of 30 samples or more) | mm/yr | 41.5 (41.3 43.8 38.5 42.4) | same | 40.5 (40.8 40.2) | ≤ 20.0 |
| Plate speed, median of plate-steps (plates of 30 samples or more) | mm/yr | 72.3 (71.4 71.8 72.3 73.8) | same | 72.8 (71.3 74.4) | 30.0–60.0 |
| Plate speed, 90th percentile of plate-steps (plates of 30 samples or more) | mm/yr | 85.5 (84.2 85.9 85.8 86.3) | same | 85.0 (84.9 85.1) | 70.0–110 |
| Median speed of plates under 20 % continental ÷ of plates over 50 % | ratio | 1.59 (1.60 1.60 1.54 1.61) | same | 1.65 (1.54 1.76) | ≥ 2.00 |
| Median speed of plates with 20 % or more of their edge on slab ÷ of plates under 5 % | ratio | 1.54 (1.54 1.43 1.64 1.53) | same | 1.59 (1.51 1.67) | ≥ 2.50 |
| Speed against slab share of the edge, rank correlation | rho | 0.44 (0.51 0.40 0.40 0.43) | same | 0.37 (0.34 0.40) | ≥ 0.65 |
| Speed against plate area, rank correlation | rho | -0.28 (-0.34 -0.22 -0.29 -0.27) | same | -0.28 (-0.25 -0.31) | -0.40–0.20 |
| Plate-steps above 150 mm/yr | % | 0.00 (0.00 0.00 0.00 0.00) | same | 0.00 (0.00 0.00) | ≤ 0.50 |
| Steps on which the speed guard binds | steps | 375 (352 372 421 355) | same | 424 (465 383) | ≤ 0.00 |
| Change of a plate's rotation per step, median | % | 4.92 (5.02 4.99 4.81 4.87) | same | 4.79 (4.76 4.83) | ≤ 8.00 |
| Change of a plate's rotation per step, 90th percentile | % | 9.05 (9.44 9.26 8.67 8.82) | same | 8.44 (8.35 8.53) | ≤ 20.0 |
| Ocean floor, median age (mean over the run after the first 100 Myr) | Myr | 72.5 (75.4 77.1 64.4 72.9) | same | 66.8 (65.6 68.0) | 50.0–80.0 |
| Ocean floor older than 180 Myr (mean over the run after the first 100 Myr) | % of sea floor | 12.5 (11.8 14.6 11.0 12.6) | same | 10.8 (10.7 10.9) | ≤ 10.0 |
| Ocean floor older than 250 Myr (mean over the run after the first 100 Myr) | % of sea floor | 4.24 (3.05 5.23 4.13 4.56) | same | 4.05 (3.93 4.18) | ≤ 3.00 |
| Supercontinent tenure (largest continent from assembled to dispersed), median | Myr | 33.5 (37.0 43.0 26.0 28.0) | same | 33.5 (28.0 39.0) | 100–300 |
| Assembly-to-assembly period, median | Myr | 69.2 (156 46.0 34.0 41.0) | same | 74.5 (90.0 59.0) | 300–600 |
| Convergence 20 Myr after a collision's onset ÷ at onset, median (needs the collision state) | ratio | n/a (n/a n/a n/a n/a) | 1.30 (n/a 2.77 -0.35 1.47) | 0.69 (0.85 0.54) | 0.20–0.60 |
| Wedge on the incoming side, width, median (needs the belts) | km | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | 100–350 |
| Foredeep width, median (needs the belts) | km | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | 150–450 |
| Foredeep depth of fill, median (needs the belts) | m | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | 2000–7000 |
| Steps with one plate above 30 % | % | 78.3 (79.0 77.2 81.4 75.6) | same | 79.9 (76.4 83.4) | – |
| Largest plate, worst step | share | 0.93 (0.91 0.88 0.99 0.93) | same | 0.96 (0.91 1.00) | – |
| Plates alive, median | plates | 13.0 (12.0 13.0 14.0 13.0) | same | 15.5 (16.0 15.0) | – |
| After a rift splits a plate above half the planet: until one is above half again, median | Myr | 24.0 (20.0 22.0 24.0 30.0) | same | 45.5 (46.0 45.0) | – |
| Rift-born plates that died: median lifetime | Myr | 21.0 (20.0 24.0 18.0 22.0) | same | 26.0 (26.0 26.0) | – |
| Plate deaths by welding | per run | 113 (110 124 114 103) | same | 74.0 (77.0 71.0) | – |
| Plate deaths by absorption of a tiny plate | per run | 46.8 (37.0 44.0 64.0 42.0) | same | 57.5 (64.0 51.0) | – |
| Plate deaths subducted away | per run | 2.00 (4.00 2.00 1.00 1.00) | same | 3.50 (3.00 4.00) | – |
| Welds in which the dying plate is the larger one | % | 53.8 (54.5 62.1 42.1 56.3) | same | 54.9 (50.6 59.2) | – |
| Land, median share of the planet | share | 0.29 (0.29 0.29 0.32 0.27) | same | 0.32 (0.33 0.32) | – |
| Continental crust at the end, share of the planet | share | 0.29 (0.27 0.25 0.32 0.31) | same | 0.36 (0.38 0.33) | – |
| Ground one collision lifts by more than 10 m: median | % of planet | 2.37 (2.41 2.19 2.63 2.26) | same | 2.87 (3.06 2.67) | – |
| Ground one collision lifts by more than 10 m: the most | % of planet | 22.1 (28.4 20.6 23.9 15.6) | same | 15.0 (12.8 17.3) | – |
| Collision belt length, median (ground raised 100 m or more) | km | 6115 (6688 5689 6463 5621) | same | 5887 (5705 6068) | – |
| Collisions whose belt is at least 3 times longer than wide | % | 25.1 (26.4 22.6 26.3 25.2) | same | 27.1 (24.7 29.6) | – |
| Contact age when the collision began, median | Myr | 7.00 (8.00 8.00 6.00 6.00) | 3.25 (4.00 3.00 2.00 4.00) | 2.00 (2.00 2.00) | – |
| Contact age when the collision began, 90th percentile | Myr | 32.3 (34.2 31.4 24.0 39.6) | 27.2 (30.4 25.4 18.0 34.8) | 25.4 (14.8 36.0) | – |
| What the pre-task-2 count read at the transfer or weld (relative speed × time), median | km | 351 (356 354 346 349) | same | 568 (571 565) | – |
| Continental crust gone under at the collision, over the front's length, median | km | 88.0 (84.2 90.9 93.2 83.9) | same | 143 (149 138) | – |
| Continental crust gone under at the collision, 90th percentile | km | 243 (237 229 253 254) | same | 326 (336 317) | – |
| Ocean floor, 90th percentile age, last step | Myr | 205 (221 237 166 196) | same | 216 (216 216) | – |
| Continental crust, share of the planet, lowest step (all steps) | share | 0.28 (0.27 0.25 0.32 0.30) | same | 0.35 (0.37 0.33) | – |
| Continental crust, share of the planet, highest step (all steps) | share | 0.45 (0.44 0.45 0.45 0.45) | same | 0.46 (0.45 0.46) | – |
| Plate speed, median over plates and steps | mm/yr | 72.3 (71.4 71.8 72.4 73.8) | same | 72.8 (71.3 74.4) | – |
| Speed of plates with a slab on 20 % of their boundary or more | mm/yr | 72.5 (72.3 72.6 71.3 73.6) | same | 72.1 (70.6 73.6) | – |
| Speed of plates with a slab on less than 20 % of their boundary | mm/yr | 53.7 (54.0 55.9 50.9 54.1) | same | 53.1 (53.8 52.3) | – |
| Slab-attached ÷ other plate speed (Earth: 3 to 4) | ratio | 1.35 (1.34 1.30 1.40 1.36) | same | 1.36 (1.31 1.41) | – |
| Correlation of plate speed with slab share of the boundary | r | 0.45 (0.50 0.42 0.42 0.44) | same | 0.41 (0.35 0.47) | – |
| Slab share of the boundary, mean over plates and steps | share | 0.44 (0.42 0.42 0.48 0.45) | same | 0.46 (0.45 0.47) | – |
| Speed of mostly oceanic plates (under half continental) | mm/yr | 70.4 (70.0 70.4 69.2 71.8) | same | 70.4 (69.1 71.7) | – |
| Speed of mostly continental plates | mm/yr | 48.1 (46.9 47.6 48.8 49.2) | same | 47.4 (48.9 46.0) | – |
| Plate-steps above 150 mm/yr, count | plate-steps | 0.00 (0.00 0.00 0.00 0.00) | same | 0.00 (0.00 0.00) | – |
| Plate-steps on which the speed guard binds | plate-steps | 729 (556 749 989 623) | same | 934 (1177 692) | – |
| Plate-steps turning about a pole on the plate itself | % | 10.2 (11.8 10.7 8.70 9.63) | same | 9.53 (9.50 9.56) | – |
| Plate-steps turning about a pole within 30° of their own centroid (design: near themselves) | % | 6.30 (4.69 5.31 8.95 6.26) | same | 7.27 (8.72 5.83) | – |
| Connected pieces of continental crust (of 1 % or more of it), median over steps | pieces | 5.50 (7.00 6.00 4.00 5.00) | same | 3.00 (3.00 3.00) | – |
| Largest continental piece, share of the continental crust, median over steps | share | 0.58 (0.46 0.60 0.78 0.49) | same | 0.72 (0.60 0.84) | – |
| Largest continental piece, share of the continental crust, lowest step | share | 0.22 (0.20 0.18 0.28 0.20) | same | 0.27 (0.25 0.29) | – |
| Supercontinent assemblies in the run (one at the start counts) | episodes | 8.00 (6.00 6.00 11.0 9.00) | same | 11.0 (11.0 11.0) | – |
| Steps with a supercontinent assembled | % | 49.3 (38.0 46.0 75.4 37.8) | same | 61.5 (44.4 78.6) | – |
| Rifts of a plate above 40 % of the planet, per run | rifts | 17.0 (13.0 14.0 24.0 17.0) | same | 13.5 (16.0 11.0) | – |
| A plate above 40 % of the planet: time above it until a rift splits it, median | Myr | 15.2 (22.0 14.0 13.0 12.0) | same | 20.5 (15.0 26.0) | – |
| Rifts of such a plate whose fragments are all still separate 50 Myr later (strict) | % | 13.6 (15.4 14.3 13.0 11.8) | same | 29.0 (12.5 45.5) | – |
| Rifts of such a plate not healed whole: two plates or more still hold its crust 50 Myr later | % | 29.7 (53.8 28.6 13.0 23.5) | same | 29.3 (31.2 27.3) | – |
| Share of a plate's driving torque that is slab pull, median (needs the force budget) | share | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | – |
| Mean in-plane stress of a plate above 40 %, median; tension positive (needs the force budget) | MPa | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | – |
| Steps in which a plate above 40 % is in tension (needs the force budget) | % | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | – |
| Slab length per contact, median (needs slab tracking) | km | n/a (n/a n/a n/a n/a) | same | n/a (n/a n/a) | – |
| Shortening across the front at the transfer or weld (convergence integrated), median | km | – | 167 (166 172 156 171) | 334 (334 335) | – |
| Shortening across the front at the transfer or weld, 90th percentile | km | – | 320 (309 328 296 347) | 386 (382 391) | – |
| Collisions that fired with no collision state (no converging continental front) | % | – | 13.3 (13.6 14.5 11.4 13.6) | 0.00 (0.00 0.00) | – |
| Collision states that ended with no transfer or weld | per 100 Myr | – | 26.4 (20.5 31.9 28.0 25.0) | 37.6 (42.1 33.1) | – |
| Of them: the continental crust left the front | per 100 Myr | – | 7.33 (5.90 8.70 7.70 7.00) | 17.5 (19.4 15.6) | – |
| Of them: the contact reversed | per 100 Myr | – | 2.65 (2.10 3.60 2.70 2.20) | 3.20 (3.50 2.90) | – |
| Of them: the contact closed or a plate died | per 100 Myr | – | 15.9 (11.8 19.1 17.1 15.4) | 15.9 (18.5 13.4) | – |
| Their length, median | Myr | – | 3.75 (4.00 4.00 3.00 4.00) | 5.00 (4.00 6.00) | – |
| Their shortening, median | km | – | 93.5 (91.7 85.8 98.5 97.9) | 135 (126 144) | – |
| Their shortening, 90th percentile | km | – | 213 (220 207 217 206) | 258 (252 265) | – |
| Continental crust that crossed their fronts | million km² per 100 Myr | – | 2.29 (2.02 2.43 2.64 2.07) | 4.28 (4.39 4.16) | – |
| Continental crust that crossed the fronts of transfers and welds | million km² per 100 Myr | – | 8.35 (7.75 7.91 9.43 8.33) | 11.2 (11.1 11.3) | – |

Hypsometry at 500 Myr (`tools/review/p1-17-hyps.sh`, seeds 1–3 × both starts, mean [min, max]). The "after" run of
seed 1, supercontinent, on the branch merged with master 412d517 printed the same hypsometry, climate and plate lines
as master 1eac09f's run. Its only extra was the volcano height percentiles that 412d517 added to the volcano summary.

| Measure | Before: master 1eac09f | After: switch off (one run of six re-run, identical) | Switch on | Target |
|---|---|---|---|---|
| Land share | 0.295 [0.252, 0.327] | same | 0.315 [0.286, 0.365] | 0.27–0.30 |
| Median land height | 687 m [569, 832] | same | 732 m [603, 933] | 430–640 m |
| Land height, 90th percentile | 2,181 m [1,925, 2,392] | same | 2,130 m [1,838, 2,479] | – |
| Land above 2 km | 0.124 [0.092, 0.149] | same | 0.119 [0.079, 0.178] | 0.10–0.13 |
| Median sea depth | 3,903 m [3,654, 4,175] | same | 3,814 m [3,415, 4,192] | – |
| Continental crust | 0.407 [0.391, 0.426] | same | 0.418 [0.371, 0.469] | – |
| Mean surface temperature | 12.0 °C | same | 13.0 °C | – |
| Glaciated land | 0.137 | same | 0.159 | – |
| Active share of the continental margin | 0.33 | same | 0.35 | 0.25–0.40 |
| Arc base height, median | 604 m | same | 533 m | – |
## Added 2026-10-02: the force balance (task 5 of the force-balance design)

Task 5 of `docs/design/plate-force-balance.md`: every step the plates stage sums each plate's
driving and resisting forces, solves the torque balance for every plate's rotation, and records it
without applying it (`plates/forces.rs`; dynamics record version 4: `StepRec::forces`, `slabs_km`,
`breakoffs`; science page `docs/science/plate-forces.md`, ledger FRC and PLT-181 to PLT-191). The
plates still move by today's rules, so this is the design's open-loop test (its section 4) made in
production code: `earthlike_quick_16k`, supercontinent, 1000 Myr, seeds 1-4, `stage volcanoes`, seed 1
with `structure.deformation.density=0` as before, on master 2c60ca3f (task 2 merged, its switch off)
plus this task. The record changes nothing a run computes (the test
`recording_does_not_change_the_world_and_matches_the_run` checks it bit for bit, and a first set of
the same four runs on master 9e84fcf reproduced every row of the earlier tables to the digit).
Reproduce:

    tools/remote/mac.sh -- tools/remote/detach.sh start fb1 env R=<scratch> J=1 tools/review/plate-check.sh 1000 1 supercontinent base:structure.deformation.density=0
    tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start fb2 env R=<scratch> J=1 tools/review/plate-check.sh 1000 2 supercontinent base
    (then seeds 3 and 4 the same way; deeptime plate-check --run <ck> x 4 prints the mean table)

All rows are reported, not judged: the design's section 7 targets (as WP-13a corrected them) judge
the motion after the switch (task 7). They are quoted beside the balance's rows so the switch has
them in view.

### The balance against today's motion

Values are the mean of the four runs; runs are seeds 1, 2, 3, 4. Plates of 30 samples or more, as
the speed rows above; "today" is the same runs' own motion.

| Measure | Balance | Runs | Today | Target (section 7, CHK) | Design's open-loop test (section 4, last column) |
|---|---|---|---|---|---|
| Plate speed, area-weighted RMS (mean over steps), mm/yr | 37.1 | 36.7 37.9 38.2 35.5 | 59.2 | 40-100 (CHK-020) | scaled to 45 |
| Plate speed, 10th / 50th / 90th percentile of plate-steps, mm/yr | 11.2 / 48.9 / 92.9 | 10.6 11.5 11.4 11.3 / 48.1 46.1 55.2 46.1 / 92.7 90.6 97.6 90.8 | 40.6 / 72.1 / 85.7 | at most 20 / 30-60 / 70-110 | |
| The same, each step scaled to an RMS of 45 mm/yr (the design's normalisation), mm/yr | 15.7 / 58.4 / 110 | 15.6 15.4 15.6 16.5 / 56.2 53.9 63.2 60.2 / 108 109 109 115 | | | 27-31 / 62-68 / 95-109 |
| Plate-steps above 150 mm/yr; above 250 (where the switch's guard would bind) | 0.04 %; 0 | 0.06 0.03 0.02 0.04; 0 0 0 0 | 0 (capped at 100) | at most 0.5 %; 0 | 1.3-3.1 % |
| Median speed, plates under 20 % continental / over 50 % | 3.66 | 3.42 3.32 4.55 3.36 | 1.59 | at least 2 (Earth 2-3) | 66-72 / 27-33 |
| Median speed, plates with 20 % or more of the edge on slab / under 5 % | 3.11 | 2.96 2.70 3.88 2.89 | 1.52 | at least 2.5 (Earth 3-4) | |
| Speed against slab share of the edge, rank correlation | 0.43 | 0.44 0.43 0.46 0.38 | 0.41 | at least 0.65 | 0.69-0.77 |
| Speed against plate area, rank correlation | -0.34 | -0.35 -0.35 -0.35 -0.33 | -0.28 | -0.4 to 0.2 | |
| Change of a plate's rotation per step, median / 90th percentile | 24.6 % / 83.6 % | 25.1 26.2 21.0 26.1 / 86.9 85.6 76.1 85.9 | 4.95 % / 8.99 % | at most 8 % / 20 % | 16-18 % / 62-70 % |
| Balance speed against today's speed: rank correlation; median of their ratio | 0.38; 0.72 | 0.34 0.38 0.48 0.32; 0.71 0.70 0.79 0.69 | | | |

### The forces and the slabs

| Measure | Value | Runs |
|---|---|---|
| Share of a plate's driving torque that is slab pull (by size, of pull, suction, edge energy, mantle flow), median | 0.86 | 0.86 0.84 0.89 0.83 |
| Mean in-plane stress of a plate above 40 % of the planet, median (tension positive) | -0.30 MPa | 0.01 -0.43 -0.08 -0.71 |
| Steps in which a plate above 40 % is in tension | 46.2 % | 50.2 47.2 49.0 38.3 |
| Slab length per subducting contact, median over contact-steps | 636 km | 618 574 700 652 |
| Slab break-offs per 100 Myr | 0.55 | 0.20 0.50 1.10 0.40 |

### What drives the slab correlation and the noise: four variants

The force settings change nothing a run computes, so variants of them run on the same geometry and
differ only in the balance. `earthlike_quick_16k`, supercontinent, 300 Myr, seed 2, on master 9e84fcf
plus this task (before the merge; the comparison is within the set):

| Variant | RMS | 10th / 50th / 90th pct | above 150 / 250 mm/yr | oceanic / continental | slab / free | slab rank corr. | rotation change p50 / p90 |
|---|---|---|---|---|---|---|---|
| defaults | 40.1 | 8.5 / 45.2 / 88.2 | 0 / 0 % | 3.02 | 2.59 | 0.37 | 22.9 / 80.7 % |
| no boundary resistance (`boundary_resistance` 0) | 54.0 | 8.8 / 43.0 / 83.6 | 3.2 / 0.9 % | 1.84 | 0.95 | -0.04 | 21.8 / 92.3 % |
| slabs pull in full at once (`slab_depth_km` 1) | 50.5 | 18.8 / 62.2 / 101 | 0 / 0 % | 2.72 | 2.57 | 0.57 | 11.9 / 63.0 % |
| no mantle flow (`mantle_flow_mm_yr` 0) | 40.1 | 8.8 / 45.2 / 88.3 | 0 / 0 % | 2.99 | 2.59 | 0.37 | 22.7 / 81.2 % |
| all three | 67.4 | 22.5 / 57.2 / 96.6 | 4.2 / 1.1 % | 1.65 | 0.85 | 0.00 | 10.5 / 73.6 % |

- Boundary resistance against the real neighbour is what makes the answer usable: without it small
  plates spin and run (3 % of plate-steps above 150 mm/yr, 1 % above 250) and the slab contrast
  disappears. This is the design's section 4 finding, now with real neighbours.
- **Slab-length tracking is the largest source of the noise, not a cure for it.** With slabs pulling
  in full from the start, the rotation change halves (median 12 % against 23 %) and the slab
  correlation rises to 0.57. A contact's slab starts again from zero whenever the contact is new,
  reversed, or closed and reopened after `contact_timeout_steps`, and today's contacts flicker; the
  design expected the tracking to smooth the pull (section 4, point 4). Task 7 should key slabs to
  the trench more persistently, or start a reopened contact with the slab it had.
- The mantle flow at 5 mm/yr changes nothing measurable.

### What the numbers say

- **The balance gives Earth's spread of speeds and its contrasts, which today's rules do not.**
  Its percentiles (11 / 49 / 93 mm/yr) and both contrasts (oceanic over continental 3.7, slab over
  free 3.1) are inside the section 7 targets at the default settings, with nothing tuned; today's
  motion meets the 90th percentile only (41 / 72 / 86; 1.6 and 1.5). Its RMS, 37 mm/yr, is just
  under the corrected target of 40-100 (Earth's area-weighted RMS is about 47 mm/yr today and 50-100
  over the last 200 Myr, WP-13a): the speed scale is to be calibrated at task 8. Nothing runs away:
  0.04 % of plate-steps are above 150 mm/yr, none above 250.
- **It is noisy from step to step.** A plate's balance rotation changes by a median 25 % a step
  (90th percentile 84 %), above the design's test (16-18 %, 62-70 %) and far above the target
  (8 %, 20 %); the variants show the slab tracking doubles it. The relaxation at the switch
  (`motion_relax_myr` 6 Myr over 2-Myr steps) damps uncorrelated jumps to roughly a quarter, which
  would put the median near 6 % (not measured).
- **Its speed follows the slab share less than the design's test found** (rank correlation 0.43,
  the same as today's motion, against 0.69-0.77; Earth 0.71 from Forsyth & Uyeda's twelve plates,
  WP-13a), although the slab-over-free ratio passes. With slabs at full pull it is 0.57.
- **It agrees with today's motion only loosely** (rank correlation of the two speeds 0.38) and is
  slower (median ratio 0.72): the switch will change every plate's speed.
- **The mega-plate is near neutral.** A plate above 40 % of the planet has a median mean stress of
  -0.3 MPa and is in tension in 46 % of its steps. The design's section 8 asked whether a mega-plate
  sits in tension waiting for a rift: in today's geometry it does not, or not strongly
  (supercontinent models put 20-50 MPa of extension in the interior, Huang et al. 2019, abstract).
  The model's mega-plates have ridges as well as trenches on their rims, and the ridges' push nearly
  cancels the suction.
- **Slabs.** The median slab is 636 km long (700 is full length). Break-offs are rare (0.55 per
  100 Myr) because today's collisions move the continent to the other plate within a step or two;
  with task 2's collision state switched on, the block will last and break-offs will become common.

### Cost

The same 300 Myr run with the balance and without it (`DEEPTIME_NO_PLATE_FORCES=1`, which leaves the
rest of the record on), both at once on the test VM, `stage volcanoes`, supercontinent, before the
merge: plates stage 78.3 against 78.3 s (seed 1) and 85.0 against 77.6 s (seed 2, every stage of
that run was 5-8 % slower: the shared machine); repeated with `/usr/bin/time`: plates 79.8 against
79.4 s, user CPU 623.2 against 624.1 s, wall 387.1 against 387.7 s (seed 1); plates 90.2 against
88.2 s, user CPU 757.2 against 756.5 s, wall 466.9 against 471.7 s (seed 2). The balance costs 0-2 %
of the plates stage (at most about 10 ms of a 0.5 s step) and nothing measurable of a run. The
record grows from 236 to 416 KB per 300 Myr and from about 1.05 to 1.7-2.0 MB per 1000 Myr (1.94 MB
for seed 1 on the merged tree): the per-plate force budget is 14 numbers a plate-step.

## Tasks 3 and 4: belts, the wedge, resistance and the weld on stall (2026-10-02)

`earthlike_quick_16k`, supercontinent, 1000 Myr, `stage volcanoes`, 2 threads, seeds 1 2 3 4, on the
test VM, every run with `structure.deformation.density=0` (the structure stage writes nothing the
plates read; it only saves time). Each cell is the mean, with the runs in brackets.

- **Before: master 750fc2cd** is the master this branch started from.
- **Before: master 2c60ca3f** is the master the branch was measured on (merged in during the work;
  it differs from 750fc2cd by the impact and deposit science packages, which change the planets a
  little through the coupled stages). This is the column to compare with.
- **After** is the branch on 2c60ca3f, with the built-in preset (the light calibration below
  included). Master 5c4d2ed3 (task 5, the forces recorded and not applied) was merged afterwards;
  it changes nothing a run computes: the four seeds re-run on the branch merged with master
  00ef214c (b14023a3) give every measure of the table below unchanged in every run; only task 5's
  force rows gain values (next section). Master 60bbbea8 (deposits), merged after, is inert for
  the plates too (150 Myr, seed 1: the same table and hypsometry).

**The force balance on these planets** (task 5's rows, recorded and not applied; the merged
head, the same four runs, mean and runs):

| Measure | After (merged head) | Target |
|---|---|---|
| Share of a plate's driving torque that is slab pull, median | 0.86 (0.84 0.87 0.88 0.86) | – |
| Mean in-plane stress of a plate above 40 %, median (tension positive) | 1.83 MPa (−0.80 4.11 1.95 2.05) | – |
| Steps in which a plate above 40 % is in tension | 69 % (36 80 81 80) | – |
| Slab length per subducting contact, median | 700 km (all four) | – |
| Slab break-offs per 100 Myr | 5.1 (7.0 5.1 4.0 4.4) | – |
| Balance: speed RMS | 46.4 mm/yr | 40–100 |
| Balance: speed 10th / 50th / 90th percentile | 13.9 / 54.8 / 97.8 mm/yr | ≤ 20 / 30–60 / 70–110 |
| Balance: oceanic ÷ continental plates, median speed | 2.76 | ≥ 2 |
| Balance: slab-attached ÷ slab-free plates, median speed | 2.85 | ≥ 2.5 |
| Balance: speed against slab share, rank correlation | 0.57 | ≥ 0.65 |
| Balance: speed against area, rank correlation | −0.41 | −0.4 to 0.2 |
| Balance: plate-steps above 150 / 250 mm/yr | 0.017 % / 0 | ≤ 0.5 % / 0 |
| Balance: change of rotation per step, median / 90th percentile | 17.8 % / 69.8 % | ≤ 8 / ≤ 20 |

On the belts' planets the open-loop balance meets its speed-distribution and speed-ratio targets,
not the slab correlation (0.57), and its rotation changes from step to step are large (the
balance is not applied, so it answers a plate geometry it does not shape; task 7's relaxation is
for this).

The before columns are read by each master's own `plate-check` (the branch cannot read their
checkpoints: the plate state changed). Three measures changed their definition with the belts, so
their before and after values are not the same quantity: the belt's width, length and elongation
(before: second moments of the ground the one-step surge raised by 100 m or more; after: the spread
of the belt and wedge's uplift across the front, and the front's length), and the "lifted" and
"land raised" measures (before: the surge's one step, which was the whole collision; after: the
largest single step of a collision that lasts many).

Reproduce (about 25 minutes a run on the loaded VM, two at a time):

    tools/remote/mac.sh -- tools/remote/detach.sh start pc env R=<scratch> tools/review/plate-check.sh 1000 1,2,3,4 supercontinent base:structure.deformation.density=0
    deeptime plate-check --run <checkpoint> --record rec.json && tools/review/belts-record.py rec.json

### Judged

The check meets **28 of its 45 ranged targets** (master 750fc2cd: 15 of 42; master 2c60ca3f: 14 of
42; three more rows are judged now that belts exist).

| Measure | Unit | Before: master 750fc2cd | Before: master 2c60ca3f | After: this branch | Target | After |
|---|---|---|---|---|---|---|
| Largest plate, median share of the planet | share | 0.484 (0.613 0.437 0.42 0.466) | 0.471 (0.442 0.371 0.544 0.526) | 0.282 (0.327 0.273 0.274 0.255) | 0.15–0.35 | pass |
| Steps with one plate above 50 % | % | 46.4 (63.4 39.8 39.0 43.4) | 43.5 (37.4 30.0 54.4 52.2) | 7.50 (16.0 0.8 7.40 5.80) | ≤ 10.0 | pass |
| Plates above 5 % of the planet, median | plates | 4.00 (3.00 4.00 4.00 5.00) | 3.75 (4.00 4.00 4.00 3.00) | 6.00 (6.00 6.00 6.00 6.00) | 5.00–9.00 | pass |
| Median plate lifetime (plates that died) | Myr | 48.8 (66.0 46.0 47.0 36.0) | 48.8 (54.0 42.0 51.0 48.0) | 95.5 (91.0 87.0 94.0 110) | ≥ 80.0 | pass |
| Plates dead within 20 Myr | % | 32.2 (30.8 30.3 34.4 33.1) | 29.9 (30.6 32.4 30.1 26.6) | 6.81 (12.3 4.62 6.30 4.07) | ≤ 15.0 | pass |
| Rift-born plates alive after 50 Myr | % | 25.2 (23.5 33.3 24.2 19.7) | 23.6 (29.8 14.3 26.3 24.1) | 70.0 (64.4 75.0 64.9 75.5) | ≥ 60.0 | pass |
| Rift-born plates welded back to their own rift family within 50 Myr | % | 37.1 (41.2 30.0 39.4 37.9) | 29.8 (21.3 22.9 47.4 27.8) | 6.10 (4.44 0 10.5 9.43) | ≤ 10.0 | pass |
| Land raised by one collision in one step, the most | % of planet | 2.57 (3.13 1.65 2.84 2.67) | 2.98 (1.12 3.37 3.88 3.56) | 0.096 (0.094 0.109 0.119 0.063) | ≤ 0.5 | pass |
| Most land gained in any step (after the first 10 Myr) | % of planet | 2.01 (2.21 2.17 1.91 1.76) | 2.19 (0.984 2.51 2.69 2.59) | 1.06 (1.11 0.988 1.15 0.993) | ≤ 1.50 | pass |
| Collision belt width, median (belt and wedge, weighted by their uplift) | km | 2,520 (2,644 2,498 2,471 2,468) | 2,468 (2,486 2,172 2,491 2,724) | 378 (376 380 378 377) | ≤ 800 | pass |
| Collision belt elongation, median | ratio | 2.28 (2.47 2.07 2.30 2.30) | 2.06 (2.03 2.08 2.34 1.78) | 6.15 (5.54 6.50 5.39 7.18) | ≥ 4.00 | pass |
| Collision length, first to last step of the collision, median | Myr | 6.00 (6.00 6.00 6.00 6.00) | 6.00 (6.00 6.00 6.00 6.00) | 28.2 (26.0 30.0 28.0 29.0) | 15.0–50.0 | pass |
| Collision length, first to last step, 90th percentile | Myr | 11.5 (10.0 10.0 14.0 12.0) | 12.3 (11.2 10.0 14.0 14.0) | 50.8 (49.0 58.0 42.4 53.6) | ≤ 120 | pass |
| Peak uplift per collision: 90th ÷ 10th percentile | ratio | 1.00 (1.00 1.00 1.00 1.00) | 1.00 (1.00 1.00 1.00 1.00) | 14.6 (10.1 20.9 9.11 18.4) | ≥ 2.00 | pass |
| Collisions per 100 Myr | per 100 Myr | 10.4 (8.50 11.8 10.4 10.9) | 10.9 (11.5 12.7 8.50 11.0) | 4.88 (4.00 5.10 5.00 5.40) | 2.00–6.00 | pass |
| Continental crust, share of the planet, lowest step (after the first 10 Myr) | share | 0.341 (0.379 0.323 0.322 0.342) | 0.332 (0.309 0.282 0.399 0.335) | 0.35 (0.357 0.336 0.388 0.319) | ≥ 0.37 | FAIL |
| Continental crust, share of the planet, highest step (after the first 10 Myr) | share | 0.459 (0.452 0.444 0.476 0.465) | 0.454 (0.453 0.456 0.459 0.447) | 0.451 (0.452 0.45 0.452 0.448) | ≤ 0.45 | FAIL |
| Active share of the continental margin (after the first 90 Myr of the record) | share | 0.316 (0.316 0.306 0.332 0.309) | 0.307 (0.32 0.287 0.31 0.313) | 0.4 (0.423 0.386 0.381 0.41) | 0.25–0.4 | pass |
| Ocean floor, 90th percentile age, mean over the run after the first 100 Myr | Myr | 193 (197 206 185 186) | 195 (193 211 190 187) | 193 (185 194 200 193) | ≤ 230 | pass |
| Plate speed, area-weighted RMS (mean over steps) | mm/yr | 58.4 (56.8 60.3 57.1 59.3) | 59.2 (59.8 59.7 56.9 60.3) | 58.1 (57.8 59.1 58.0 57.7) | 40.0–100 | pass |
| Plate speed, 10th percentile of plate-steps (plates of 30 samples or more) | mm/yr | 40.0 (41.7 40.6 39.9 37.8) | 40.6 (41.5 40.6 37.8 42.6) | 34.6 (33.1 32.2 37.5 35.7) | ≤ 20.0 | FAIL |
| Plate speed, median of plate-steps (plates of 30 samples or more) | mm/yr | 72.4 (72.6 71.7 71.9 73.3) | 72.1 (70.6 72.5 73.5 71.6) | 67.4 (64.6 68.0 70.4 66.7) | 30.0–60.0 | FAIL |
| Plate speed, 90th percentile of plate-steps (plates of 30 samples or more) | mm/yr | 85.3 (84.6 85.3 85.1 86.1) | 85.7 (85.0 86.1 86.4 85.3) | 84.0 (82.2 84.8 84.6 84.5) | 70.0–110 | pass |
| Median speed of plates under 20 % continental ÷ of plates over 50 % | ratio | 1.56 (1.61 1.48 1.60 1.53) | 1.59 (1.56 1.57 1.73 1.48) | 1.62 (1.70 1.65 1.54 1.59) | ≥ 2.00 | FAIL |
| Median speed of plates with 20 % or more of their edge on slab ÷ of plates under 5 % | ratio | 1.54 (1.54 1.49 1.55 1.59) | 1.52 (1.44 1.47 1.74 1.43) | 1.68 (1.73 1.75 1.64 1.62) | ≥ 2.50 | FAIL |
| Speed against slab share of the edge, rank correlation | rho | 0.419 (0.348 0.449 0.422 0.457) | 0.411 (0.407 0.426 0.416 0.397) | 0.495 (0.524 0.492 0.503 0.462) | ≥ 0.65 | FAIL |
| Speed against plate area, rank correlation | rho | -0.272 (-0.268 -0.293 -0.317 -0.211) | -0.279 (-0.261 -0.305 -0.312 -0.239) | -0.322 (-0.223 -0.248 -0.429 -0.386) | -0.4–0.2 | pass |
| Plate-steps above 150 mm/yr | % | 0 (0 0 0 0) | 0 (0 0 0 0) | 0 (0 0 0 0) | ≤ 0.5 | pass |
| Steps on which the speed guard binds | steps | 403 (420 403 411 378) | 383 (392 378 343 418) | 440 (423 438 417 481) | ≤ 0 | FAIL |
| Change of a plate's rotation per step, median | % | 4.89 (4.87 4.90 4.84 4.97) | 4.95 (4.96 4.90 4.99 4.96) | 5.03 (5.03 5.06 5.00 5.03) | ≤ 8.00 | pass |
| Change of a plate's rotation per step, 90th percentile | % | 8.89 (8.61 9.02 8.73 9.20) | 8.99 (9.12 8.95 8.92 8.95) | 9.43 (9.72 9.81 8.84 9.34) | ≤ 20.0 | pass |
| Ocean floor, median age (mean over the run after the first 100 Myr) | Myr | 69.7 (72.8 73.2 65.9 66.7) | 72.6 (74.3 74.3 69.3 72.4) | 68.3 (70.9 66.1 71.3 65.1) | 50.0–80.0 | pass |
| Ocean floor older than 180 Myr (mean over the run after the first 100 Myr) | % of sea floor | 12.1 (12.3 13.9 10.8 11.2) | 12.7 (12.6 14.9 11.6 11.7) | 11.5 (10.5 11.0 13.0 11.5) | ≤ 10.0 | FAIL |
| Ocean floor older than 250 Myr (mean over the run after the first 100 Myr) | % of sea floor | 4.69 (4.73 5.74 3.79 4.51) | 4.81 (4.87 5.85 4.66 3.85) | 4.71 (4.19 5.35 4.63 4.68) | ≤ 3.00 | FAIL |
| Supercontinent tenure (largest continent from assembled to dispersed), median | Myr | 38.0 (62.0 25.0 24.0 41.0) | 29.2 (8.00 25.0 24.0 60.0) | 9.50 (12.0 10.0 8.00 8.00) | 100–300 | FAIL |
| Assembly-to-assembly period, median | Myr | 90.0 (93.0 106 45.0 116) | 65.0 (31.0 46.0 54.0 129) | 41.5 (26.0 34.0 86.0 20.0) | 400–800 | FAIL |
| Plate boundary that is subducting (ocean-ocean and ocean-continent), share of the total length, mean over the run after the first 100 Myr | % of boundary | 56.2 (56.5 55.2 56.2 57.0) | 55.8 (54.6 55.2 57.4 56.0) | 55.8 (55.3 56.2 56.1 55.6) | 15.0–25.0 | FAIL |
| Oceanic (island) arc cells above sea level, share of the arc cells, over the run after the first 100 Myr | % of arc cells | 15.4 (14.3 13.9 15.7 17.7) | 14.0 (13.1 12.5 16.7 13.5) | 18.5 (18.8 18.9 16.9 19.3) | 10.0–35.0 | pass |
| Oceanic arc cells, 90th percentile elevation (the crest; mean over the run after the first 100 Myr) | m | 386 (313 308 393 528) | 269 (185 156 451 284) | 647 (662 676 541 708) | -1,700–1,000 | pass |
| Continental arc cells, median elevation (mean over the run after the first 100 Myr) | m | 1,592 (1,463 1,504 1,627 1,775) | 1,574 (1,604 1,533 1,659 1,500) | 1,771 (1,896 1,848 1,674 1,665) | 1,200–4,500 | pass |
| Continental arc cells, 90th percentile elevation (the crest; mean over the run after the first 100 Myr) | m | 2,698 (2,555 2,646 2,730 2,858) | 2,682 (2,659 2,671 2,766 2,633) | 3,091 (3,291 3,243 2,845 2,983) | 2,000–6,000 | pass |
| Convergence 20 Myr after a collision's onset ÷ at onset, median | ratio | 0.565 (1.33 0.113 -0.438 1.25) | 0.759 (1.56 n/a -0.046 n/a) | 0.063 (0.012 0.088 0.065 0.086) | 0.2–0.6 | FAIL |
| Wedge on the incoming side, width when the belt stood highest, median | km | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | 0 (0 0 0 0) | 100–350 | FAIL |
| Foredeep width in front of a belt above 3 km, median | km | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | 0 (0 0 0 0) | 150–450 | FAIL |
| Foredeep depth below local isostasy in front of a belt above 3 km, median | m | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | 9.49 (1.05 23.7 8.28 4.92) | 2,000–7,000 | FAIL |

### What moved, and what did not

**The review's three faults.**

1. *Land pops up in round areas.* Fixed. The most land one collision raises in a step is 0.10 % of
   the planet (3.0 % on master 2c60ca3f), the most land gained in any step 1.06 % (2.2 %, target
   1.5), a belt is a median 378 km wide and 6.2 times longer than wide (2,470 km and 2.1), and a
   collision takes a median 28 Myr (6). Peak uplift now differs between collisions (90th over 10th percentile 15,
   was 1).
2. *No Wilson cycle.* Fixed on these measures. Rift-born plates alive after 50 Myr: 70 % (24 %);
   welded back to their own rift family within 50 Myr: 6 % (30 %); plates dead within 20 Myr: 7 %
   (30 %); median plate lifetime 96 Myr (49).
3. *The mega-plate.* Fixed on these measures. One plate above half the planet in 7.5 % of steps
   (44 %; seed 1 alone 16 %), the largest plate a median 0.28 of the planet (0.47), six plates above
   5 % (3.75). After a rift splits a plate above half the planet it takes a median 343 Myr until
   one is above half again (28 Myr). Collisions per 100 Myr: 4.9 (10.9).

**Not met.**

- *Convergence 20 Myr after onset over convergence at onset*: 0.06 (target 0.2 to 0.6). Collisions
  stop too quickly: with `collision_resistance_mm_yr` 200 a belt about 1 km high already cancels
  40 mm/yr on a front as long as the plate's own slab, so most collisions have stalled by 20 Myr.
  At 100 the collisions did not stop before consuming their continents (below). The present speed
  law gives no middle: it drives a plate with no slab at 45 mm/yr for no reason, and the resistance
  has to overcome that. This is the measure the force balance (task 7) is for.
- *Wedge width* 0 km and *foredeep* 0 km wide, 9 m deep (targets 100 to 350 km, 150 to 450 km, 2 to
  7 km). The wedge is built (a test shows it), but at 1.5 degrees it is as wide as the belt's edge
  stands above the foreland over 26 m per km, and belts peak at a median 1.6 to 2.3 km (mean height
  within half the belt's width): under one sample spacing (71 km) for most. The widest wedges are
  150 to 215 km. With no wedge standing on the incoming plate there is no load on it: the belt is
  on the other plate, and the boundary between them is a break in the flexure until they weld. So
  the basin does not form on these planets, although the mechanism works (a flexure test with a
  resolved wedge gives a basin of over 100 m in one step). Of the belts above 3 km (10 to 19 a
  run) the widest foredeep of each run was 71 to 357 km and the deepest 170 to 310 m.
- *Active share of the continental margin*: 0.40 (0.38 to 0.42), at the top of its range (0.31
  on master). Fewer welds leave more continents on plates of their own, facing oceans that subduct.
- *Continental crust, lowest step*: 0.35 (0.36 0.34 0.39 0.32), against master's 0.33 (0.31 0.28
  0.40 0.34) and the target of 0.37. Not compensated: the loss on master comes from the flexure
  feedback on arcs and from resampling (the crust bisect, 2026-10-02), which another task repairs.
  What collisions themselves do to the crust is reported below.
- The speed measures, the boundary mix (56 % subducting), the supercontinent tenure and period and
  the old sea floor are the speed law's and are not this task's; they are within noise of master.

**The crust that collisions consume.** Continental crust that crosses collision fronts is lost as
area (the stacked share stays as thickness). Per 1000 Myr run 0.25 to 0.40 of the sphere's area
crossed fronts (0.26 0.31 0.25 0.40; master: about 0.21, from 8.4 + 2.1 million km2 per 100 Myr),
and a collision that ends in a weld has shortened by a median 280 to 500 km. The
design's defaults (resistance 100 against a 3,000 km front, threshold 300 km) gave 0.53 and a
lowest crust share of 0.18 to 0.25 on one seed; the four changes that brought it down are in the
design's "As built: tasks 3 and 4".

### Reported, not judged

| Measure | Unit | Before: master 750fc2cd | Before: master 2c60ca3f | After: this branch | Target | After |
|---|---|---|---|---|---|---|
| Steps with one plate above 30 % | % | 74.8 (86.8 65.6 72.2 74.8) | 78.7 (83.4 66.8 84.4 80.2) | 41.2 (52.2 37.8 40.0 34.8) | – | – |
| Largest plate, worst step | share | 0.933 (0.973 0.917 0.924 0.919) | 0.873 (0.874 0.893 0.855 0.871) | 0.665 (0.745 0.555 0.769 0.59) | – | – |
| Plates alive, median | plates | 12.5 (14.0 12.0 13.0 11.0) | 13.5 (13.0 15.0 12.0 14.0) | 16.2 (13.0 16.0 17.0 19.0) | – | – |
| After a rift splits a plate above half the planet: until one is above half again, median | Myr | 19.5 (16.0 21.0 27.0 14.0) | 27.8 (25.0 52.0 20.0 14.0) | 343 (532 n/a 198 298) | – | – |
| Rift-born plates that died: median lifetime | Myr | 23.0 (20.0 33.0 19.0 20.0) | 22.0 (24.0 21.0 23.0 20.0) | 85.8 (72.0 86.0 91.0 94.0) | – | – |
| Plate deaths by welding | per run | 104 (85.0 118 104 109) | 109 (115 127 85.0 110) | 48.8 (40.0 51.0 50.0 54.0) | – | – |
| Plate deaths by absorption of a tiny plate | per run | 49.2 (47.0 46.0 53.0 51.0) | 46.2 (38.0 54.0 48.0 45.0) | 69.2 (62.0 74.0 75.0 66.0) | – | – |
| Plate deaths subducted away | per run | 1.25 (1.00 1.00 3.00 0) | 2.75 (4.00 1.00 3.00 3.00) | 3.50 (4.00 5.00 2.00 3.00) | – | – |
| Welds in which the dying plate is the larger one | % | 53.7 (48.2 56.8 49.0 60.6) | 56.2 (58.3 62.2 42.4 61.8) | 0 (0 0 0 0) | – | – |
| Land, median share of the planet | share | 0.322 (0.323 0.3 0.36 0.303) | 0.305 (0.299 0.279 0.327 0.315) | 0.309 (0.359 0.285 0.308 0.286) | – | – |
| Continental crust at the end, share of the planet | share | 0.355 (0.405 0.332 0.322 0.363) | 0.343 (0.309 0.31 0.409 0.342) | 0.352 (0.364 0.336 0.391 0.319) | – | – |
| Ground one collision lifts by more than 10 m in one step: median | % of planet | 2.44 (2.25 2.22 2.70 2.61) | 2.39 (1.91 1.80 3.10 2.76) | 0.283 (0.286 0.288 0.269 0.29) | – | – |
| Ground one collision lifts by more than 10 m in one step: the most | % of planet | 23.0 (20.4 24.2 27.8 19.7) | 25.5 (15.5 25.0 30.3 31.2) | 1.05 (0.883 1.33 0.913 1.06) | – | – |
| Collision belt length, median (belt and wedge, weighted by their uplift) | km | 6,316 (6,040 6,304 7,232 5,688) | 6,073 (6,249 5,506 6,948 5,589) | 2,250 (2,116 2,488 2,052 2,343) | – | – |
| Collisions whose belt is at least 3 times longer than wide | % | 29.5 (40.0 26.3 28.8 22.9) | 28.3 (28.7 29.9 36.5 18.2) | 84.3 (90.0 82.4 76.0 88.9) | – | – |
| Contact age when the collision began, median | Myr | 3.75 (4.00 4.00 3.00 4.00) | 3.50 (4.00 4.00 4.00 2.00) | 5.25 (5.00 4.00 6.00 6.00) | – | – |
| Contact age when the collision began, 90th percentile | Myr | 25.7 (23.2 24.6 26.8 28.0) | 24.5 (26.0 18.0 20.0 34.0) | 36.5 (34.0 36.0 52.4 23.4) | – | – |
| Continental crust gone under at the collision, over the front's length, median | km | 88.6 (90.5 85.6 80.8 97.7) | 70.9 (61.8 68.9 63.1 89.9) | 373 (280 499 388 323) | – | – |
| Continental crust gone under at the collision, 90th percentile | km | 243 (254 234 251 234) | 239 (271 202 240 242) | 892 (663 1,304 761 839) | – | – |
| Shortening across the front at the transfer or weld (convergence integrated), median | km | 184 (202 173 159 204) | 168 (165 187 150 170) | 373 (280 499 388 323) | – | – |
| Shortening across the front at the transfer or weld, 90th percentile | km | 328 (298 344 334 336) | 321 (316 328 334 307) | 892 (663 1,304 761 839) | – | – |
| Collisions that fired with no collision state (no converging continental front) | % | 11.4 (8.24 12.7 14.4 10.1) | 10.6 (13.0 12.6 10.6 6.36) | 0 (0 0 0 0) | – | – |
| Collision states that ended with no transfer or weld | per 100 Myr | 22.7 (25.1 23.2 23.1 19.4) | 27.2 (26.8 32.7 18.1 31.2) | 25.7 (20.3 27.4 27.1 27.8) | – | – |
| Of them: the continental crust left the front | per 100 Myr | 6.23 (7.40 6.40 5.40 5.70) | 7.28 (6.10 8.80 6.10 8.10) | 11.1 (9.70 11.2 12.4 11.1) | – | – |
| Of them: the contact reversed | per 100 Myr | 2.62 (3.80 2.40 2.60 1.70) | 3.00 (3.00 4.10 1.90 3.00) | 6.62 (5.00 8.00 5.50 8.00) | – | – |
| Of them: the contact closed or a plate died | per 100 Myr | 13.6 (13.4 14.1 14.7 12.0) | 16.5 (17.2 19.0 9.70 20.0) | 7.20 (5.00 7.30 8.50 8.00) | – | – |
| Their length, median | Myr | 3.50 (2.00 4.00 4.00 4.00) | 4.00 (4.00 4.00 4.00 4.00) | 5.50 (8.00 4.00 4.00 6.00) | – | – |
| Their shortening, median | km | 95.1 (91.1 90.4 85.3 113) | 92.7 (91.0 90.5 89.5 99.8) | 94.9 (107 94.7 87.7 90.5) | – | – |
| Their shortening, 90th percentile | km | 215 (199 231 194 235) | 206 (210 205 204 204) | 579 (655 550 435 675) | – | – |
| Continental crust that crossed their fronts | million km² per 100 Myr | 2.08 (1.92 2.22 2.48 1.72) | 2.48 (2.46 2.99 1.40 3.07) | 7.05 (7.63 6.81 4.77 8.99) | – | – |
| Continental crust that crossed the fronts of transfers and welds | million km² per 100 Myr | 8.38 (7.61 7.93 8.48 9.50) | 7.75 (6.81 8.15 7.70 8.33) | 8.44 (5.64 8.84 8.04 11.2) | – | – |
| Ocean floor, 90th percentile age, last step | Myr | 205 (219 220 213 167) | 202 (204 171 235 197) | 206 (209 207 196 211) | – | – |
| Continental crust, share of the planet, lowest step (all steps) | share | 0.341 (0.379 0.323 0.322 0.342) | 0.332 (0.309 0.282 0.399 0.335) | 0.35 (0.357 0.336 0.388 0.319) | – | – |
| Continental crust, share of the planet, highest step (all steps) | share | 0.459 (0.452 0.444 0.476 0.465) | 0.454 (0.453 0.456 0.459 0.447) | 0.451 (0.452 0.45 0.452 0.448) | – | – |
| Plate speed, median over plates and steps | mm/yr | 72.4 (72.6 71.7 71.9 73.3) | 72.1 (70.6 72.6 73.5 71.6) | 67.5 (64.7 68.0 70.4 66.8) | – | – |
| Speed of plates with a slab on 20 % of their boundary or more | mm/yr | 72.0 (72.0 71.9 71.5 72.5) | 72.2 (70.8 72.9 73.5 71.6) | 69.2 (68.4 69.6 70.9 68.1) | – | – |
| Speed of plates with a slab on less than 20 % of their boundary | mm/yr | 52.9 (52.5 54.2 53.1 51.9) | 53.8 (54.6 54.9 49.5 56.2) | 49.7 (47.8 47.4 52.4 51.1) | – | – |
| Slab-attached ÷ other plate speed (Earth: 3 to 4) | ratio | 1.36 (1.37 1.33 1.35 1.40) | 1.35 (1.30 1.33 1.49 1.27) | 1.40 (1.43 1.47 1.35 1.33) | – | – |
| Correlation of plate speed with slab share of the boundary | r | 0.444 (0.406 0.453 0.446 0.47) | 0.427 (0.412 0.428 0.486 0.382) | 0.462 (0.5 0.459 0.478 0.412) | – | – |
| Slab share of the boundary, mean over plates and steps | share | 0.454 (0.484 0.428 0.465 0.439) | 0.446 (0.436 0.424 0.485 0.44) | 0.411 (0.393 0.413 0.431 0.409) | – | – |
| Speed of mostly oceanic plates (under half continental) | mm/yr | 70.1 (70.4 69.5 69.9 70.6) | 70.2 (69.2 70.4 72.0 69.2) | 67.3 (66.4 67.6 69.2 66.2) | – | – |
| Speed of mostly continental plates | mm/yr | 49.5 (48.0 51.7 49.2 49.1) | 48.7 (48.2 49.4 45.6 51.8) | 46.6 (44.3 45.7 49.5 47.1) | – | – |
| Plate-steps above 150 mm/yr, count | plate-steps | 0 (0 0 0 0) | 0 (0 0 0 0) | 0 (0 0 0 0) | – | – |
| Plate-steps on which the speed guard binds | plate-steps | 836 (876 841 930 695) | 755 (914 777 529 799) | 1,180 (1,060 1,075 1,041 1,545) | – | – |
| Plate-steps turning about a pole on the plate itself | % | 10.6 (11.3 10.8 9.14 11.2) | 10.2 (12.2 9.70 9.97 9.08) | 10.7 (10.6 12.3 9.99 9.90) | – | – |
| Plate-steps turning about a pole within 30° of their own centroid (design: near themselves) | % | 7.58 (7.25 6.05 7.34 9.68) | 6.22 (5.84 6.22 5.51 7.31) | 10.6 (11.4 11.2 8.74 10.9) | – | – |
| Connected pieces of continental crust (of 1 % or more of it), median over steps | pieces | 4.25 (3.00 7.00 4.00 3.00) | 5.50 (6.00 6.00 4.00 6.00) | 5.25 (3.00 5.00 6.00 7.00) | – | – |
| Largest continental piece, share of the continental crust, median over steps | share | 0.717 (0.839 0.492 0.735 0.803) | 0.595 (0.453 0.507 0.8 0.619) | 0.535 (0.816 0.549 0.373 0.4) | – | – |
| Largest continental piece, share of the continental crust, lowest step | share | 0.24 (0.276 0.17 0.259 0.255) | 0.207 (0.183 0.159 0.249 0.239) | 0.156 (0.158 0.171 0.129 0.166) | – | – |
| Supercontinent assemblies in the run (one at the start counts) | episodes | 8.75 (7.00 6.00 13.0 9.00) | 9.00 (9.00 10.0 10.0 7.00) | 14.2 (15.0 21.0 7.00 14.0) | – | – |
| Steps with a supercontinent assembled | % | 56.2 (75.4 21.4 60.0 68.2) | 48.5 (25.8 34.8 75.6 57.8) | 39.6 (69.4 48.2 10.2 30.6) | – | – |
| Rifts of a plate above 40 % of the planet, per run | rifts | 17.0 (12.0 12.0 23.0 21.0) | 15.5 (13.0 14.0 21.0 14.0) | 5.75 (5.00 6.00 7.00 5.00) | – | – |
| A plate above 40 % of the planet: time above it until a rift splits it, median | Myr | 16.5 (25.0 15.0 10.0 16.0) | 13.2 (14.0 17.0 12.0 10.0) | 13.8 (18.0 7.00 20.0 10.0) | – | – |
| Rifts of such a plate whose fragments are all still separate 50 Myr later (strict) | % | 9.55 (0 9.09 9.09 20.0) | 12.9 (8.33 23.1 5.00 15.4) | 52.9 (75.0 80.0 16.7 40.0) | – | – |
| Rifts of such a plate not healed whole: two plates or more still hold its crust 50 Myr later | % | 33.2 (27.3 63.6 31.8 10.0) | 40.7 (58.3 69.2 20.0 15.4) | 85.8 (100 80.0 83.3 80.0) | – | – |
| Plate boundary, total length per step, mean (after the first 100 Myr; Earth: 260,487 km, Bird 2003) | km | 340,623 (365,798 361,781 330,456 304,459) | 352,274 (343,597 345,340 346,028 374,130) | 354,802 (303,138 339,717 366,473 409,880) | – | – |
| Subducting plate boundary, length per step, mean (after the first 100 Myr; Earth: 51,310 km, Bird 2003) | km | 191,474 (207,289 199,369 185,365 173,873) | 198,247 (190,056 191,579 200,690 210,665) | 198,442 (167,019 191,447 206,089 229,213) | – | – |
| Oceanic arc cells, median elevation (mean over the run after the first 100 Myr; band cells include the flanks, not only summits) | m | -2,036 (-2,070 -2,241 -1,938 -1,894) | -2,206 (-2,354 -2,322 -1,919 -2,227) | -1,893 (-1,936 -1,850 -1,964 -1,824) | – | – |
| Subducting boundary that is ocean-ocean, share of the subducting length (after the first 100 Myr) | % of subduction | 51.1 (53.6 50.6 50.3 50.1) | 53.0 (52.8 55.0 49.3 54.8) | 54.1 (53.0 53.4 54.1 55.8) | – | – |
| Plate boundary that is spreading (ridge and rift), share of the total length (after the first 100 Myr) | % of boundary | 39.4 (39.2 40.2 39.4 38.8) | 39.7 (40.5 40.3 38.7 39.3) | 39.4 (39.6 39.1 39.6 39.2) | – | – |
| Plate boundary at a collision (continental crust at the subducting side), share of the total length (after the first 100 Myr) | % of boundary | 3.09 (2.95 3.12 3.12 3.15) | 3.00 (3.09 2.87 2.77 3.26) | 3.60 (3.70 3.63 3.13 3.95) | – | – |
| Plate boundary that is transform or converging with no contact yet, share of the total length (after the first 100 Myr) | % of boundary | 1.30 (1.40 1.48 1.35 0.988) | 1.48 (1.75 1.64 1.13 1.39) | 1.25 (1.41 1.12 1.21 1.26) | – | – |
| Plate boundary that is subducting, share of the total length, first 50 Myr | % of boundary | 50.0 (48.7 51.9 47.3 52.1) | 50.0 (48.7 52.0 47.3 51.9) | 49.0 (46.9 51.4 46.3 51.5) | – | – |
| Subducting boundary that is ocean-ocean, share of the subducting length, first 50 Myr | % of subduction | 63.5 (61.0 61.4 62.5 69.2) | 63.4 (61.1 61.2 62.5 68.8) | 62.7 (62.8 58.4 62.7 67.0) | – | – |
| Plate boundary that is spreading, share of the total length, first 50 Myr | % of boundary | 47.9 (48.8 46.8 50.2 45.9) | 47.9 (48.8 46.6 50.2 46.0) | 48.6 (50.2 47.0 50.4 46.7) | – | – |
| Plate boundary at a collision, share of the total length, first 50 Myr | % of boundary | 0.74 (0.844 0.552 0.643 0.921) | 0.732 (0.853 0.476 0.643 0.957) | 0.924 (1.00 0.595 1.26 0.841) | – | – |
| Plate boundary that is transform or converging with no contact yet, share of the total length, first 50 Myr | % of boundary | 1.35 (1.64 0.817 1.83 1.10) | 1.37 (1.64 0.862 1.83 1.15) | 1.45 (1.83 0.95 2.09 0.937) | – | – |
| Oceanic arc cells per step, mean (after the first 100 Myr) | cells | 2,977 (3,379 3,162 2,736 2,630) | 3,232 (3,117 3,266 2,891 3,654) | 2,994 (2,430 2,783 3,251 3,512) | – | – |
| Continental arc cells per step, mean (after the first 100 Myr) | cells | 2,682 (2,905 2,674 2,733 2,416) | 2,604 (2,554 2,359 2,844 2,658) | 2,611 (2,400 2,583 2,672 2,791) | – | – |
| Oceanic arc cells above sea level, share, first 50 Myr | % of arc cells | 22.3 (21.4 25.5 22.4 20.1) | 22.8 (21.3 27.4 22.4 20.1) | 25.2 (21.5 31.5 22.9 24.8) | – | – |
| Oceanic arc cells, 90th percentile elevation, first 50 Myr | m | 673 (686 733 721 552) | 696 (678 831 727 549) | 824 (727 953 804 814) | – | – |
| Continental arc cells, 90th percentile elevation, first 50 Myr | m | 2,561 (2,505 2,609 2,663 2,470) | 2,561 (2,510 2,610 2,661 2,465) | 2,652 (2,554 2,696 2,794 2,565) | – | – |
| Share of a plate's driving torque that is slab pull, median (needs the force budget) | share | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | – | – |
| Mean in-plane stress of a plate above 40 %, median; tension positive (needs the force budget) | MPa | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | – | – |
| Steps in which a plate above 40 % is in tension (needs the force budget) | % | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | – | – |
| Slab length per contact, median (needs slab tracking) | km | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | n/a (n/a n/a n/a n/a) | – | – |

### Hypsometry at 500 Myr

`tools/review/p1-17-hyps.sh 500 "1 2 3" "supercontinent scattered_continents"`, mean of six
[min, max]. The highest cell is the top of each run's hypsometric curve (L1, the plate-and-surface
raster), the 99th percentile the surface stage's `land_z_p99_m`.

| Measure | Before: master 750fc2cd | Before: master 2c60ca3f | Design defaults (share 0.5, uplift 0.42; 2 runs) | After (share 0.8, uplift 0.55) | Target |
|---|---|---|---|---|---|
| Land share | 0.304 [0.291, 0.312] | 0.300 [0.252, 0.364] | 0.231, 0.236 | **0.284** [0.252, 0.307] | 0.27–0.30 |
| Median land height | 746 m [686, 824] | 605 m [435, 799] | 462, 379 m | **515 m** [398, 571] | 430–640 m |
| Land height, 90th percentile | 2,126 m [1,854, 2,382] | 1,890 m [1,517, 2,132] | | 2,169 m [1,941, 2,397] | – |
| Land above 2 km | 0.118 [0.076, 0.152] | 0.088 [0.052, 0.119] | 0.066, 0.075 | **0.118** [0.094, 0.144] | 0.10–0.13 |
| Highest L1 cell | 4.8 km [4.4, 5.3] | 4.6 km [3.7, 5.6] | 5.4, 5.7 km | **6.6 km** [5.4, 9.8] | 5.5–7 km |
| 99th percentile of land | 3.2 km [2.7, 3.6] | 3.2 km [2.7, 4.2] | | 4.1 km [3.4, 4.5] | 4.2–5.5 km |
| Sample-steps at the plate model's elevation guard | 0 | 0 | | 0 | 0 |
| Median sea depth | 3,833 m | 3,935 m | | 3,706 m | – |
| Continental crust | 0.409 [0.394, 0.425] | 0.418 [0.369, 0.459] | 0.360, 0.371 | 0.413 [0.388, 0.435] | – |
| Active share of the continental margin | 0.33 | 0.32 | | 0.42 [0.39, 0.46] | 0.25–0.40 |
| Glaciated land | 0.134 | 0.123 | | 0.137 | – |
| Crust spread by the plateau flow, per run | | 35 to 10,300 km3 | | 190,000 to 680,000 km3 | – |

Land, the median, the share above 2 km and the mean highest cell are inside their targets; master
met only the land share (and, on 2c60ca3f, the median). One run of six has a cell at 9.8 km (above
the 7 km of the range), three have their highest cell at 5.4 km (just under 5.5), and the 99th
percentile is 0.1 km under its range. The active margin share is above its range here as in the
1000 Myr runs. Four of the six "after" runs were made with the two values given as `--set`
overrides before they went into the preset; the last two with the preset. All six re-run on the
branch merged with master 00ef214c, with the preset alone: every number of this table identical
in every run.

**The light calibration** (the owner chose light at this landing). Tuned: `collision_resistance_mm_yr`
100 to 200 and `collision_threshold_km` 300 to 100 (so collisions stop and small terranes accrete
before they are consumed: plate measures), `collision_stacking_share` 0.5 to 0.8 and
`subduction_uplift_mm_yr` 0.42 to 0.55 on the quick presets (0.34 to 0.45 on the full presets, by
the same factor, not re-measured) for the hypsometry, which lost the surge's broad uplift. Not
tuned: the belt width, the wedge slope, the weld speed and wait, the cordillera, `arc_continental_m`
(one seed with 40 m instead of 80 m gave a lower crust share, 0.29 against 0.32: noise, and the
coordinator asked that the crust not be compensated).
