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
production code, on the same four runs: `earthlike_quick_16k`, supercontinent, 1000 Myr, seeds 1-4,
`stage volcanoes`, seed 1 with `structure.deformation.density=0` as before; master 9e84fcf plus
this task. Every row of the tables above is reproduced to the digit by these runs (the boundary-mix
rows, the arcs and the rows of "Other rows moved" alike): the record changes nothing a run computes
(the test `recording_does_not_change_the_world_and_matches_the_run` checks it bit for bit). Reproduce:

    tools/remote/mac.sh -- tools/remote/detach.sh start fb1 env R=<scratch> J=1 tools/review/plate-check.sh 1000 1 supercontinent base:structure.deformation.density=0
    tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start fb2 env R=<scratch> J=1 tools/review/plate-check.sh 1000 2 supercontinent base
    (then seeds 3 and 4 the same way; deeptime plate-check --run <ck> x 4 prints the mean table)

All rows are reported, not judged: the design's section 7 targets judge the motion after the switch
(task 7). The targets are quoted beside the balance's rows so the switch has them in view.

### The balance against today's motion

Values are the mean of the four runs; runs are seeds 1, 2, 3, 4. Plates of 30 samples or more, as
the speed rows above; "today" is the same runs' own motion (the tables above).

| Measure | Balance | Runs | Today | Design target (section 7) | Design's open-loop test (section 4, last column) |
|---|---|---|---|---|---|
| Plate speed, area-weighted RMS (mean over steps), mm/yr | 38.0 | 37.4 36.1 36.8 41.6 | 59.6 | 35-55 | scaled to 45 |
| Plate speed, 10th / 50th / 90th percentile of plate-steps, mm/yr | 10.6 / 48.1 / 94.1 | 10.1 10.2 11.1 11.0 / 47.6 43.6 51.7 49.6 / 94.9 92.4 93.7 95.6 | 41.5 / 72.3 / 85.6 | at most 20 / 30-60 / 70-110 | |
| The same, each step scaled to an RMS of 45 mm/yr (the design's normalisation), mm/yr | 14.4 / 57.6 / 107 | 13.7 14.3 15.6 14.1 / 58.5 54.7 62.9 54.1 / 114 110 108 95.0 | | | 27-31 / 62-68 / 95-109 |
| Plate-steps above 150 mm/yr; above 250 (where the switch's guard would bind) | 0.02 %; 0 | 0 0 0 0.08; 0 0 0 0 | 0 (capped at 100) | at most 0.5 %; 0 | 1.3-3.1 % |
| Median speed, plates under 20 % continental / over 50 % | 3.93 | 3.79 3.80 4.38 3.73 | 1.59 | at least 2 (Earth about 3) | 66-72 / 27-33 |
| Median speed, plates with 20 % or more of the edge on slab / under 5 % | 3.16 | 3.04 2.78 3.54 3.29 | 1.54 | at least 2.5 (Earth 3-4) | |
| Speed against slab share of the edge, rank correlation | 0.44 | 0.42 0.45 0.46 0.45 | 0.44 | at least 0.65 | 0.69-0.77 |
| Speed against plate area, rank correlation | -0.36 | -0.38 -0.33 -0.38 -0.35 | -0.28 | -0.4 to 0.2 | |
| Change of a plate's rotation per step, median / 90th percentile | 25.6 % / 84.9 % | 25.7 27.7 24.7 24.1 / 86.4 88.0 82.0 83.0 | 4.9 % / 9.1 % | at most 8 % / 20 % | 16-18 % / 62-70 % |
| Balance speed against today's speed: rank correlation; median of their ratio | 0.40; 0.70 | 0.47 0.36 0.37 0.37; 0.71 0.64 0.74 0.72 | | | |

### The forces and the slabs

| Measure | Value | Runs |
|---|---|---|
| Share of a plate's driving torque that is slab pull (by size, of pull, suction, edge energy, mantle flow), median | 0.85 | 0.83 0.83 0.89 0.87 |
| Mean in-plane stress of a plate above 40 % of the planet, median (tension positive) | -0.47 MPa | -0.44 -1.33 0.22 -0.32 |
| Steps in which a plate above 40 % is in tension | 46.3 % | 42.2 39.7 54.9 48.2 |
| Slab length per subducting contact, median over contact-steps | 615 km | 612 496 700 652 |
| Slab break-offs per 100 Myr | 0.90 | 0.50 0.60 1.70 0.80 |

### What the numbers say

- **The balance gives Earth's speeds and contrasts that today's rules do not.** Its RMS (38 mm/yr),
  its percentiles (11 / 48 / 94) and its two contrasts (oceanic over continental 3.9, slab over
  free 3.2) are inside every section 7 target they are measured against, at the default settings
  and with nothing tuned; today's motion meets none of them (RMS 60, 42 / 72 / 86, 1.6 and 1.5).
  Earth (Zahirovic et al. 2015, read in the library 2026-10-02): median RMS about 40 mm/yr, plates
  under 25 % continental 81 against 28 mm/yr over 50 %, more than 25 % of the perimeter subducting
  79 against 26 without subduction. Nothing runs away: 0.02 % of plate-steps are above 150 mm/yr
  (one plate-step in seed 4), none above 250.
- **It is noisy from step to step.** A plate's balance rotation changes by a median 26 % a step
  (90th percentile 85 %), more than in the design's test (16-18 %, 62-70 %) and far above the
  target (8 %, 20 %). Boundaries are found per sample and torques jump as samples cross; the
  slab-length tracking does not smooth it enough. The design's answer is the relaxation at the
  switch (`motion_relax_myr` 6 Myr); a relaxation of 6 Myr over 2-Myr steps damps a step's jump to
  about 28 % of itself, so the median would land near 7 %, if the jumps are uncorrelated (not
  measured).
- **Its speed follows the slab share less than the design's test found** (rank correlation 0.44,
  the same as today's motion, against 0.69-0.77), although the slab-over-free ratio (3.2) passes.
  See the variants below for what moves it.
- **It agrees with today's motion only loosely** (rank correlation of the two speeds 0.40) and is
  slower (median ratio 0.70): the switch will change every plate's speed.
- **The mega-plate is near neutral.** A plate above 40 % of the planet has a median mean stress of
  -0.5 MPa and is in tension in 46 % of its steps. The design's section 8 asked whether a
  mega-plate sits in tension waiting for a rift: in today's geometry it does not, or not strongly
  (supercontinent models put 20-50 MPa of extension in the interior, Huang et al. 2019, abstract).
  The model's mega-plates have ridges as well as trenches on their rims, and their push nearly
  cancels the suction.
- **Slabs.** The median slab is 615 km long (700 is full length): most contacts live long enough
  to pull in full. Break-offs are rare (0.9 per 100 Myr) because today's collisions move the
  continent to the other plate within a step or two; with the collision state of task 2 the block
  will last and break-offs will become common.
