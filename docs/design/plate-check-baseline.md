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
| Slab-attached / other plate speed (mean speeds) | 1.41 | 1.44 1.38 1.50 1.33 | Earth: 3 to 4 (Forsyth and Uyeda) |
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
