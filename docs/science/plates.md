# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207, PLT-229, PLT-251 to PLT-252, PLT-285 to PLT-286, PLT-289, PLT-312, PLT-320 (49; PLT-320 the cordillera at Earth's typical arc, with no convergence factor, the crust budget against Stern & Scholl, 2026-10-20; PLT-312 the arcs on the convergence normal to the front, an experiment kept off, the land-shape task, 2026-10-20; PLT-289 ocean floor as thick as a continent turns continental, the resampling lumps, 2026-10-18; PLT-285 and PLT-286 the continental arcs and the cones as rock, plate task 8, 2026-10-17) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081, PLT-222 to PLT-227, PLT-241 to PLT-250, PLT-253, PLT-290 to PLT-292, PLT-400, PLT-546 (36; PLT-546 the incoming plate under the belt, V1 gap 6, 2026-10-24; PLT-400 the wedge's crust crosses whole, a switch kept off, the crust flux into collision fronts, 2026-10-21; PLT-290 to PLT-292 the crust a collision took counted once, the resampling lumps, 2026-10-18; PLT-069 to PLT-075 and PLT-227 retired 2026-10-02) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212, PLT-228, PLT-293, PLT-350 to PLT-355 (26; PLT-293 ocean crust keeps its own rock at resamplings and in the L1 sampling, 2026-10-19; PLT-350 to PLT-353 the floor's GDH1 rate with the sea from the ocean's volume, 2026-10-21, PLT-354 the gap's floor held in place and PLT-355 GDH1's curve between other ridge and abyssal depths, on [sea-level.md](sea-level.md)) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208, PLT-254 to PLT-259, PLT-261 to PLT-264, PLT-277 to PLT-284, PLT-287 to PLT-288, PLT-294 (44; PLT-294 the switch for the transition's ocean side, ocean crust or the continent's rock, 2026-10-19, on in the shipped presets since 2026-10-20; PLT-287 and PLT-288 the fastest thickening and rise, 2026-10-18; PLT-283 the gap budget and PLT-284 the accretion's height, 2026-10-17; PLT-254 to PLT-259 the orogenic collapse and the thickest crust, 2026-10-03; PLT-261 to PLT-264 the ocean–continent transition, 2026-10-04; PLT-277 spreading before foundering, 2026-10-05; PLT-278 to PLT-282 the eclogite root and whole-column isostasy, 2026-10-06) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173, PLT-213 to PLT-221, PLT-265, PLT-310 to PLT-311 (43; PLT-310 and PLT-311 the boundary's normal and Bird's transform test, the land-shape task, 2026-10-20; PLT-265 Earth's passive margins, the transition's comparison) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-180, PLT-191 to PLT-202 (19) |
| [PLT](ledger/PLT.md) | the force balance's settings, coded and recorded, not applied (page [plate-forces.md](plate-forces.md); constants in [FRC](ledger/FRC.md)) | PLT-181 to PLT-190, PLT-585 (11; PLT-585 Forsyth & Uyeda's balance, a switch kept off, section "Plate speed, phase 2", 2026-10-25) |
| [PLT](ledger/PLT.md) | the continents census's Earth references and measuring choices (`tools/review/continents_census.py`, section "White patches, drowned continents and coastal high ground", 2026-10-21) | PLT-360 to PLT-363 (4) |
| [PLT](ledger/PLT.md) | the crust flux into collision fronts: the crust budget's measures and Earth's references (section "The crust flux into collision fronts", 2026-10-21) | PLT-401 (1) |
| [PLT](ledger/PLT.md) | the trench budget: sediment subduction, subduction erosion and where it acts, the trench's release, the measures and Earth's references (section "The trench budget", 2026-10-22; switches, off by default) | PLT-440 to PLT-444 (5) |

Rows by status (after the ocean–continent transition, 2026-10-04): `verified` 17, `cited-unverified` 46, `from-memory` 4, `abstract-only` 19, `calibrated` 14, `game-choice` 79, `uncited` 42; `pending-rewrite` 151; `not-yet-coded` 8; `merged` (not counted) 33.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/PLT.md](extraction-notes/PLT.md).
- Designs: `docs/design/plate-force-balance.md`, `docs/design/plate-collisions-and-welding.md`, `docs/design/plate-check-baseline.md` (the measured state of today's model).
- Accuracy review of one run: `docs/research/notes/run-review-1a0f969d358.md`.
- Work package in the verification plan: WP-13 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Since 2026-10-02 (plate-motion task 2) a contact keeps a collision state with its real shortening: the convergence across the collision front, integrated (ledger PLT-222 to PLT-226). Since plate tasks 3 and 4 (the same day) that state builds the collision belt step by step, resists the convergence and welds the plates when it has stalled (section "Collision belts, the wedge and the weld" below); the older count of relative speed times time and its switch (PLT-227) are gone.

- The accuracy review calls today's model "a convincing cartoon of plate tectonics, not yet an accurate simulation": no Wilson cycle (rifted pieces weld back after a median 18 Myr), one plate over half the planet in about half of all frames, collisions that raise round blobs in one step.
- The force balance is computed and recorded every step since 2026-10-02 (task 5), not yet applied: [plate-forces.md](plate-forces.md), constants in ledger FRC; PLT-181 to PLT-190 are coded.
- The force-balance design could not confirm three figures: the gross size of slab pull, a factor of 8 for continental drag attributed to Forsyth & Uyeda, and the wedge-taper reference. WP-13a settled all three; see the next section.

## Collision belts, the wedge and the weld (plate tasks 3 and 4)

Built 2026-10-02 (ledger PLT-241 to PLT-252; rows changed: PLT-029, PLT-030, PLT-038, PLT-053, PLT-068, PLT-078 to
PLT-080, PLT-192, PLT-202; retired: PLT-069 to PLT-075, PLT-227). This was the first of two stages: it ran on
the old speed rules for plates. Since 2026-10-04 a balance of forces moves the plates and the belt's
resistance is a term of it (`docs/design/plate-force-balance.md` task 7; [plate-forces.md](plate-forces.md)).

**What happens on Earth.** When two continents meet at a subduction zone, the continent arriving on
the sinking plate is too buoyant to go far down. Its crust is scraped off and stacked: on the
overriding plate behind the suture it thickens into a belt that, if the collision goes on, spreads
into a plateau (Tibet); on the arriving plate a wedge of thrust sheets builds up in front of the
suture (the Himalaya itself is Indian crust). A wedge that has reached its critical shape keeps it
while it grows, "continuing to grow at constant taper as additional material is encountered at the
toe" (Davis, Suppe & Dahlen 1983, abstract); the frontal Himalayan wedge's surface slopes at 1–1.6°
(DeCelles & Carrapa 2021, full text). The weight of the belt and wedge bends the arriving plate
down in front of them into a foreland basin, typically 100–300 km wide and 2–8 km deep in sediment
(DeCelles & Giles 1996, full text). The thickened crust's weight pushes back against the
convergence: India's convergence with Asia fell from about 15 to about 4 cm/yr between 50 and
35 Ma, and "crustal thickening in the Tibetan Plateau was the dominant cause" (Copley, Avouac &
Royer 2010, abstract); the force involved is about 7 × 10¹² N per metre of front (Molnar &
Lyon-Caen 1988, as cited by Schmalholz & Mancktelow 2016, full text), against about 10¹³ N/m for
slab pull. A collision lasts tens of Myr; two plates that have stopped converging move as one.

**What the model does.** Each 2 Myr step, for every contact in collision (continental crust of the
sinking plate standing at the overriding plate's edge and converging, PLT-224):

1. The crust that crossed the front this step (the convergence across it times the step, times
   the front's length) times its own thickness is the volume to place. `collision_stacking_share`
   (0.5, PLT-242; no source gives a measured share) of it stays in the crust; the rest goes down
   with the slab. Thickened crust raises the ground by one 5.5th of its thickness (Airy, PLT-121).
2. The wedge takes its part first (PLT-244): continental ground on the arriving plate is raised to
   the profile that rises from the foreland towards the front at `wedge_slope_deg` (1.5°, PLT-192)
   up to the overriding plate's height at the front.
3. The belt takes the rest (PLT-241, PLT-243): spread over `collision_belt_km` (400 km) behind the
   front, highest at the front. A belt sample rises no higher than the crust's strength height
   (about 5.2 km at 140 MPa and 1 g, PLT-253): what the front cannot take goes to the belt behind
   it, so the belt widens into a plateau, and what no sample can take goes down with the slab.
   Above that height the surface stage's spreading moves crust sideways (the same height), and
   erosion wears the belt down. Before the limit (2026-10-03) a collision that never stalled
   piled crust 200-290 km thick on a sliver of a plate and the surface stood 14 km high: the
   spreading runs once a step on the surface, while a fast front stacks kilometres a step.
4. Belt and wedge are stamped as a collision orogeny (the `collision` regime, PLT-079, PLT-247), and
   the stacked rock is the consumed crust's own rock, its sediments metamorphosed (PLT-248).
5. The surface stage's flexure (`flexure.md`) carries the uplift as a load: the stiff arriving
   plate bends down in front of the weak young wedge into the foreland basin, which the surface
   stage fills.
6. The belt resists through the force balance (FRC-018; until 2026-10-04 a cut inside the speed
   rules, PLT-245, retired): per metre of front a force equal to the belt's gravitational potential
   energy over the arriving crust's, against the convergence, whatever its speed. A plate pulled by
   a long slab elsewhere keeps converging and widens its belt; a plate whose slab broke off when
   the continent filled its trench stops. Continental crust at a front does not pull (only oceanic
   slabs do, PLT-251), and a continent at a front is not dragged down to trench depth (PLT-252).
7. When the convergence has stayed below `weld_speed_mm_yr` (5 mm/yr) for `weld_after_myr`
   (10 Myr), a continent welds its plate to the other and the larger plate keeps its id (PLT-246,
   PLT-250). A terrane smaller than `collision_suture_frac` moves alone to the overriding plate once
   the collision has shortened by `collision_threshold_km` (PLT-068) or stalled.

Until these tasks every collision lifted everything within up to 2,000 km of the contact by the
same 1,400 m in the step the count fired, and welded the plates at once (the review's three faults:
land popping up in round blobs, rifted oceans re-welding at once, a mega-plate).

**Earth against the model** (`earthlike_quick_16k`, supercontinent start, 1000 Myr, seeds 1 to 4,
2026-10-02, `tools/review/plate-check.sh`; all tables in `docs/design/plate-check-baseline.md`,
section "Tasks 3 and 4").

| Quantity | Earth | Model before | Model now |
|---|---|---|---|
| Collision belt: width, length over width | Himalaya 150–350 km wide and 2,400 km long; young belts 5 to 15 times longer than wide (Wikipedia, via the collisions design) | 2,470 km, 2.1 | 378 km, 6.2 |
| How long a collision lasts | India–Asia 50 Myr and going on; Alleghanian 65 Myr (Wikipedia; no statistic exists) | one event, 6 Myr of state | median 28 Myr, 90th percentile 51 Myr |
| Largest plate's share of the planet | 0.205 (the Pacific, Bird 2003 Table 1) | 0.47 | 0.28 |
| Convergence 20 Myr after onset over at onset | India: about 15 to about 4 cm/yr in 15 Myr, 0.27 (Copley et al. 2010, abstract) | not defined (welded) | 0.06 |
| Wedge on the arriving plate | the Himalaya, 151–350 km (Wikipedia) | none | median 0; the widest of a run 150–215 km |
| Foreland basin | 100–300 km wide, 2–8 km of fill (DeCelles & Giles 1996, full text) | none | median none; under belts above 3 km at most 71–357 km wide, 170–310 m deep |
| Belt height | Tibet about 5 km over 1,000 km | every collision 1,400 m | median peak 1.6–2.3 km (mean within 200 km of the front); the highest belts 4.7–6.5 km |

**Limits and game choices.**

- The table above was measured on the old speed rules (a resistance of 200 mm/yr tuned so that
  collisions stopped before they consumed their continents). Since the switch to the force balance
  see [plate-forces.md](plate-forces.md), "Earth against the mod", for the same rows.
- Crust that crosses a collision front is lost as continental area; the share that stays (0.8 in
  the pipeline presets, calibrated against the land's height distribution; 0.5 by design; no
  measured value was found) thickens the belt. Per 1000 Myr 0.25 to 0.40 of the sphere's area in
  continental crust crosses collision fronts; island arcs that mature into continental crust make
  up for it, as on master.
- On the quick preset a sample of the plate model is 71 km across. A wedge at 1.5° under a belt
  2 km high is 76 km wide, so most wedges are one row of samples or none, and the foreland basin,
  which needs the wedge's weight on the arriving plate, does not form: the plate boundary is a
  break in the flexure until the plates weld. The mechanism is tested with a resolved wedge.
- The belt's profile across its width, the weld speed and wait, the 3-step grace of a collision
  and the terrane threshold are choices of the implementation (ledger `game-choice`).
- The folds and faults of the structure stage follow the belt and wedge through the `collision`
  regime; they do not yet read the real shortening or the flexural basin (task 6 of the design).

### The belt from critical taper (2026-10-25)

Ledger PLT-600 (new, `verified`) and PLT-601 (new, `game-choice`); rows changed: PLT-192,
PLT-243, PLT-244, PLT-401. On by default since 2026-10-25 (`tectonics.collision_belt_taper`). It
replaces step 3's profile above, `1 - (d/W)^2` highest at the front (PLT-243, an agent's choice
with no source), and leaves how much crust stays untouched.

**What happens on Earth.** In a collision the plate boundary reaches the surface at the
deformation front (the Himalaya's Main Frontal Thrust), and the orogenic wedge sits on its hanging
wall: the Lesser Himalaya is Indian crust above the décollement. Its surface rises from the toe at
a nearly constant slope, 1 to 1.6° in the Nepal Himalaya (DeCelles & Carrapa 2021, section 2.4.1;
PLT-192). A critical wedge grows self-similarly, its width as the square root of the material it
has taken in, W = [2hVt / tan(α + β)]^½ (Dahlen 1990, eq. 3, p. 60). It cannot rise without limit:
"it is thus the existence of a brittle-plastic transition at 10-15 km depth that ultimately limits
the height of very wide fold-and-thrust belts and accretionary wedges. This provides a natural
explanation for the break in topographic slope of the Higher Himalaya at the edge of the Tibetan
Plateau" (Dahlen 1990, p. 96). DeCelles & Carrapa limit their taper analysis to the southernmost
~80 km, since farther north the décollement lies deeper than 18 km and brittle behaviour is
unlikely. So the high ground of a collision belt is not at the front: the front is the toe, the
surface climbs from it, and behind the slope stands a plateau.

**What the model does.** The front is the toe. The belt's surface rises from the overriding
edge's mean height z_f at `wedge_slope_deg` (1.5°, PLT-192) up to the crust's strength height
h_s (about 5.2 km at 1 g, PLT-253), and is flat behind: S(d) = min(h_s, z_f + d tan α). The step's
kept crust raises the ground under S nearest the front first, samples at the same distance alike,
until it runs out, so the belt widens behind its toe as it takes crust in. When the whole belt
within `collision_belt_km` (400 km, PLT-241) is under S and crust is left, the surface rises in
place (S = min(h_s, H + d tan α), H by bisection): the toe advances (PLT-601). What does not fit
with every sample at h_s goes down with the slab as before. How much the belt keeps is the old
fill's placed total to the metre, PLT-243's room share and its repeated passes included; only
where the thickening lands moves. At z_f = 300 m the ramp reaches h_s 187 km behind the front.

The first two arms broke that rule. They kept PLT-243's room share k alone, while the old fill
repeats its pass and keeps 1 - (1 - k)^6 behind a narrow overriding continent, so they sent 6.8 %
of the crossed crust down as left over behind a sliver against base12's 4.4 %. The fixture test
had a wide overriding plate (k = 1) and missed it. The fix (1e4bc9dd) places exactly the old
fill's total, with a test behind a continent one row deep
(`the_taper_keeps_what_the_old_fill_keeps_behind_a_sliver`); ct-taper3 is the clean arm.

**The arms** (`earthlike_quick_16k`, 1000 Myr, seeds 1 to 4, both starts, `--threads 2`,
`+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`, measures `v1census` version 2
and `guard`; base12 is master's 24-run base, key 4f5b17199a21 = 5cf16ecd76cd, argkey cd665cef):

| arm | what | key, argkey |
|---|---|---|
| ct-base12-s1to4 | base12's own runs of seeds 1 to 4, cached, no sim | 5cf16ecd76cd, cd665cef |
| ct-taper3 | the taper at 1.5°, conserving (the clean arm) | 2242d7ae7b13, e4750c17 |
| ct-taper2 | the taper at 1.5°, before the fix | 43da3799ceea, e4750c17 |
| ct-taper1 | the taper at 1.6°, before the fix | f04289f05a38, 1d8485b3 |

Medians, the two-sided Mann-Whitney p against base12's 24 runs in brackets (`pool.sh table --base
base12 --measure v1census --measure guard`); the crust budget's shares of the crossed crust from
each run's last `cb_globals` line (`tools/review/m5diag/cbfinal.py`):

| measure | base12 (24) | ct-base12-s1to4 | ct-taper3 | ct-taper2 | ct-taper1 |
|---|---|---|---|---|---|
| land | 0.312 | 0.309 (0.68) | 0.316 (0.72) | 0.301 (0.14) | 0.327 (0.086) |
| largest landmass | 0.189 | 0.204 (0.91) | 0.203 (0.40) | 0.183 (0.33) | 0.221 (0.69) |
| continental crust under the sea | 0.190 | 0.198 (0.74) | 0.181 (0.88) | 0.242 (0.094) | 0.160 (0.43) |
| land above 2 km (fires above 0.07) | 0.128 | 0.128 (0.81) | 0.104 (0.78) | 0.091 (0.12) | 0.091 (0.12) |
| coast with high ground (fires above 0.11) | 0.229 | 0.230 (0.68) | 0.234 (0.88) | 0.209 (0.11) | 0.202 (0.25) |
| land under ice | 0.107 | 0.100 (0.45) | 0.082 (0.20) | 0.105 (0.69) | 0.091 (0.27) |
| collisions per 100 Myr | 1.60 | 1.95 (0.48) | 1.95 (0.18) | 1.45 (0.19) | 1.70 (0.98) |
| continental crust's net change over the run | −0.069 | −0.065 (0.78) | −0.053 (0.064) | −0.050 (0.018) | −0.061 (0.21) |
| criteria met per run, median and mean of 7 | 5, 5.04 | 5, 5.00 | 5, 5.00 | 5, 5.25 | 5, 5.12 |
| guard: steps beyond 2 Earth reliefs | 0 | 0 | 0 | 0 | 0 |
| guard's worst, m from the sea | 10,118 | 10,105 (0.45) | 10,025 (0.48) | 10,138 (0.85) | 10,054 (0.45) |
| largest rise / fall of a cell in a step, m | 7,237 / 7,338 | 7,252 / 7,277 | 6,902 / 7,101 | 7,259 / 7,326 | 7,052 / 7,086 |
| plate cells' largest excess over Airy, m | 2,819 | 2,782 (0.91) | 2,248 (0.007) | 2,343 (0.010) | 2,333 (0.005) |
| highest plate elevation, m | 6,341 | 6,272 (0.50) | 5,985 (0.033) | 5,996 (0.002) | 6,114 (0.013) |
| crust crossing collision fronts, km³ over the run | 1.32e10 | 1.35e10 | 1.54e10 | 1.42e10 | 1.45e10 |
| kept | 0.885 | 0.890 | 0.898 | 0.872 | 0.870 |
| in the wedge (PLT-244) | 0.173 | 0.176 | 0.173 | 0.166 | 0.153 |
| left over behind a sliver | 0.044 | 0.041 | 0.038 | 0.068 | 0.058 |
| down at the strength height | 0.024 | 0.024 | 0.017 | 0.019 | 0.015 |
| placed after the belt was full (`taper_toe`, PLT-601) | | | 0.035 | 0.032 | not counted |

Against base12's own runs of the same seeds (`--base ct-base12-s1to4`) ct-taper3's p are 0.57 to
1.0 for the realism checks (0.19 for the crust's net change), 0.028 for the excess over Airy and
0.13 for the highest elevation.

Where the coast's high ground comes from: the class of its nearest qualifying high ground, in
absolute shares of the land. These come from the M5 diagnosis's fields
(`tools/review/m5diag/m5arm.sh`, then `m5compare.py`), so every M5 and M4 in this table is the
**version 1 rule** (the census grid, closed depressions as sea, the reach straight across), not
the census's version 2 above. base12 here is its runs of seeds 1 to 4, and the p are against them:

| version 1 rule | base12 s1-4 | ct-taper3 | ct-taper2 | ct-taper1 |
|---|---|---|---|---|
| M5, coast with high ground | 0.231 | 0.264 (0.72) | 0.211 (0.44) | 0.233 (0.65) |
| M4, land above 2 km | 0.128 | 0.104 (1.0) | 0.091 (0.28) | 0.091 (0.28) |
| M5 by young collision belts | 0.111 | 0.090 (0.08) | 0.075 (0.005) | 0.059 (0.01) |
| M5 by young Andean margins | 0.026 | 0.061 (0.08) | 0.040 (0.38) | 0.047 (0.08) |
| M5 by old collision belts | 0.035 | 0.044 (0.19) | 0.038 (0.80) | 0.046 (0.33) |
| M5 without the young collision belts | 0.157 | 0.241 (0.03) | 0.187 (0.44) | 0.214 (0.13) |
| M4 by young collision belts | 0.061 | 0.033 (0.01) | 0.030 (0.01) | 0.032 (0.005) |
| M4 by young Andean margins | 0.008 | 0.026 (0.005) | 0.015 (0.10) | 0.014 (0.13) |

What the arms say:

- The physics holds on the pool: no run failed, no guard step, and the taper lowers what stood
  too high. Plate cells' largest excess over Airy falls by a fifth (2,819 to 2,248 m) and the
  highest plate elevation by 360 m, since the belt is no longer piled at the front.
- The crust budget agrees with the rule. ct-taper3 keeps 0.898 of what crossed against 0.890 on
  the same seeds, and its sliver and strength-height losses are as small or smaller. The planets
  part after the first collisions, so the totals differ by run, not by rule.
- The coast's high ground does not move (0.234 against 0.229, p 0.88). Land above 2 km falls from
  0.128 to 0.104 (p 0.78). On the version 1 rule the young collision belts' share of land above
  2 km halves (p 0.01) and their part of the high coast falls a fifth, but the young Andean
  margins' part rises in step, so M5 stays where it was. The taper moves the collision belts'
  high ground off the coast; it does not remove the coast's high ground.
- The slope's own effect, ct-taper1 (1.6°) against ct-taper2 (1.5°), is small: coast with high
  ground 0.202 against 0.209, land above 2 km 0.091 in both. Both carry the conservation flaw, so
  the comparison holds between them, not against base12. 1.5° stays (PLT-192). 1.6° is the
  average of DeCelles & Carrapa's nine profiles outside the Chitwan region, inside the 1 to 1.6°
  already ledgered, and changing it would move the wedge (PLT-244) too.
- PLT-241's width binds now and then: in the median run 3.5 % of the crossed crust (2.0 to 3.6 %)
  was placed after the belt was full to 400 km. That bound is the belt's existing reach, a game
  choice (PLT-601), not a value from the sources.

**Limits.**

- On the quick preset a sample is 71 km across, so the 187 km ramp is under three samples: the
  surface's slope is resolved coarsely and a belt is a ramp of two or three rows and a plateau.
- The Greater Himalaya's steeper front (7.4° on average outside the Chitwan region, DeCelles &
  Carrapa 2021) is not modelled: it lies beyond the brittle wedge the slope belongs to, and one
  sample could not resolve it.
- The fill order (nearest the front first, ties alike), the front as the toe and the strength
  height as the wedge's height limit are this lane's reading of Dahlen 1990 and DeCelles & Carrapa
  2021, not values from them (PLT-600's status).
- In the head-on fixture, where the overriding edge sits right at the front, the wedge on the
  arriving plate (PLT-244) waits for the toe to advance; on the pool its share of the crossed crust
  is unchanged (0.173 against 0.176). The two tests that read the old profile's wedge from the
  first step pin `collision_belt_taper = false`.

## Orogenic collapse and the thickest crust (2026-10-03)

Built 2026-10-03 (ledger PLT-254 to PLT-259; `plates/collapse.rs`). Ben's answer of that day
("Merge, then give area back (Recommended)"): plate tasks 3 and 4 cost continental crust, because
crust that crosses a collision front becomes belt thickness and its area never comes back.

**What happens on Earth.** Thickened crust stands high, and its weight pushes outwards. While the
plates converge they hold it up; when convergence stops, the hot, weak orogen spreads under its
own weight and thins back towards a normal thickness, mostly by extension rather than erosion
(Dewey 1988, abstract: collapse explains "the return of orogenic large crustal thicknesses to
normal without very much erosional denudation"). The Variscan crust of Europe was 55-70 km thick
by 315 Ma and thinned from 315 to 290 Ma (Hillenbrand & Williams 2022, abstract); the Caledonides
began to collapse about 405 Ma, with detachments active from 400 to 365 Ma that moved rock
40-100 km (Peron-Pinvidic & Osmundsen 2020, full text). Where extension goes is set by crustal
thickness (Wernicke et al. 1987, full text). Crust thicker than about 50 km does not stay: "Isostatic
forces uplift crust thicker than about 50 km, leading to erosion and crustal thinning" (Mooney,
Prodehl & Pavlenkova 2002, full text); an orogen that never extended, the Urals, keeps a root of
50-56 km (Carbonell et al. 2000, abstract). The weak lower crust flows only so far: "Over 15 m.y.,
the length scale of channel-flow extrusion is 150 km rather than 1500 km" (Rey, Teyssier &
Whitney 2010, full text). And no crust is much thicker than Tibet's and the Andes' 70-80 km
(Mooney et al. 2002: "The thickest crust (70+ km)"; Wang et al. 2021: Moho at 80 ± 2 km under
Lhasa): beyond that, continental crust carried down in a collision stays in the mantle (Stern &
Scholl 2010, full text: at ultrahigh-pressure terranes "the bulk of the approximately 40 km-thick
crust that must have originally existed is missing"), and a mafic root turned to eclogite founders.

**What the model had.** At 500 Myr on master (6f72f0dd) a fifth of the visible continental samples
had crust thicker than 70 km, holding over 40 % of the continental crust's volume, under ground at
a median 200-500 m: in the coupled pipeline the surface stage owns the elevation, the belt's cap
(PLT-253) reads that elevation while the stacked thickness stays, and an overlap at a resampling
adds a whole column to the one on top (PLT-228). None of that crust gave area back.

**What the model does now.** Each 2 Myr step, after the collision belts and subduction uplift:

1. *The thickest crust* (PLT-258, PLT-277): a visible continental column thicker than
   `crust_max_km` (70 km) spreads the excess at once to the nearest room on its plate, by the
   collapse's transport below but without waiting for the orogeny to end (since 2026-10-05: the
   lower crust of a belt that a long collision thickens flows out while the convergence goes
   on). It fills continental crust only up to `collapse_thickness_km` and ocean floor up to the
   normal column. What finds no room within reach goes to the mantle from the bottom of the
   column (mafic root first) and gives no area; since 2026-10-23 at the pace of a root's
   drop-off, 1 − e^(−δt / 10 Myr) of it a step, not all at once (PLT-477, stage 3 of
   [the lithosphere's thermal state](lithosphere-thermal-state.md)).
2. *Collapse* (PLT-254): a visible continental sample that no orogeny has raised for
   `collapse_quiet_myr` (10 Myr, PLT-259: collapse follows the end of the thickening) and whose
   crust is thicker than `collapse_thickness_km` (50 km since 2026-10-05, was 55; PLT-256) releases
   (1 − e^(−δt/τ)) × (T − 50 km) of its thickness, τ = `collapse_myr` (15 Myr, PLT-255). The
   crust released (its own rock, thinned in proportion) fills the nearest samples of the same
   plate that are thinner than the model's normal column (35.1 km), within `collapse_reach_km`
   (200 km, PLT-257), nearest first: thinned continental crust, then the plate's own ocean floor
   beside the continent, which turns continental once filled. A sample with no room within reach
   keeps its crust until room appears.
3. Heights follow Airy (5.5 m of crust per m of height) within the column's Airy height: a source
   sinks but not below the height of the crust it keeps (a hidden root spreads without dragging
   its surface down), a receiver rises but not above the height of its new column (ocean floor
   filled to the normal column stands at a continent's mean height, about 525 m).

**Earth against the model.** See the build log entry "Orogenic collapse" (2026-10-03) for the
measured tables (`plate-check` at 1000 Myr and the 500 Myr hypsometry ensemble).

**Limits and game choices.**
- The transport is to the nearest room, not a solved lower-crustal flow; it is instantaneous within
  the reach. The reach (200 km) is Rey et al.'s 150 km in 15 Myr rounded to about three samples of
  the quick preset.
- Collapse waits 10 Myr after the last orogeny that raised the sample, the model's stall time
  before a weld (PLT-246). The sources say when collapse starts (as thickening ends, Variscan; with
  a change of the plate boundary, Nevada) but not how long after; on Earth thinning can lag
  thickening by tens of Myr (the Basin and Range: Lundstern & Miller 2021, abstract).
- The threshold was 55 km at first, chosen among the sources' values by the 1000 Myr crust share
  on the old motion law (with 50 km the crust ended at 0.52 of the planet). Since 2026-10-05 it
  is 50 km, the figure Mooney et al. state, under Ben's validation policy of that day: physics
  from sources, and the land and crust shares reported against Earth rather than tuned. The quiet
  time is still the stall time before a weld, not a sourced value.
- Since 2026-10-05 crust beyond `crust_max_km` first spreads (PLT-277): it fills continental
  crust up to `collapse_thickness_km` and ocean floor up to the normal column within
  `collapse_reach_km`, and only what finds no room founders. Filling to the limit itself (70 km)
  was tried and dropped: every long collision then widened into a 70 km plateau. Earth loses
  about 0.7 km³ of continental crust a year at all collision zones, about 0.3 km³/yr of it deep
  continental crust (Stern & Scholl 2010, after Scholl & von Huene 2009); the model founders about
  6 km³ a year under the force balance (14 before this rule; build log 2026-10-05), so the reach
  and the room are still too small for the crust a long collision stacks.
- Thickness beyond `crust_max_km` is removed whether the crust is felsic or mafic. Felsic crust
  does not founder by itself (Wang et al. 2021); in the model the limit stands for continental
  subduction at the front as much as for foundering.
- The driver is the excess thickness, not the height: the model's thickness is the better record
  of the crust's weight because the surface stage's flexure and spreading change the height
  without changing the thickness. That decoupling is the model's, and the reason the thickest
  crust needs a limit at all.

## The ocean–continent transition of rifted margins (2026-10-04)

Built 2026-10-04 (ledger PLT-261 to PLT-265; `plates/transition.rs`; the ocean side's cooling in
`plates/pools.rs`, PLT-129's note). Ben's answer of 2026-10-03 ("Merge both, then fix the cause"):
after submarine slope failure the continental slopes were still about twice Earth's, and the cause
was the plate model's ocean–continent step.

**What happens on Earth.** Most of today's continental edges are passive margins: 105,000 km of
them against 53,000 km of convergent plate boundary (Bradley 2008, full text). A passive margin is
a rift that broke up. Before the ocean opened, the continent was stretched, by a factor β that
grows towards the future ocean (McKenzie 1978, full text: "a unit length of continental lithosphere
is suddenly extended to a length β"), so its crust thins from a normal thickness at the landward
limit of thinning to a few kilometres where it meets the first ocean crust. In the Coral Sea "a
thinning of the crust from 22 km down to a 7–8 km thick" continental crust lies "in direct contact
with the steady oceanic crust of similar thickness" (Sapin, Ringenbach & Clerc 2021, full text). How
wide the thinning is depends on the margin: at volcanic margins, "the majority of passive
continental margins worldwide", "Crustal thinning typically occurs over a comparably short distance
in the order of only 50 to 100 km" (Franke 2013, accepted manuscript), while magma-poor margins
have wide extended domains: the necking domain from about 50 to more than 110 km, the exhumation
domain more than 150 km (Sapin et al. 2021), hyper-extended crust under 10 km thick over 70 km off
Iberia and 200 km off Angola (Brune et al. 2014, full text). Thinned crust stands lower, and once
the stretched lithosphere cools the margin subsides further with the ocean floor beside it
(McKenzie 1978). The surface over the taper is the shelf, the slope and the rise: passive-margin
shelves average 88 km wide and their slopes 46 km (Harris et al. 2014, author copy); the slope
between the 500 and 2,000 m isobaths has a median of 2.1° in ETOPO 2022 (the margins study, build
log 2026-10-03).

**What the model had.** The plate model's crust steps from a continental column (35.1 km, a few
hundred metres up) to ocean floor (7 km, 2.5–5.6 km down) between two neighbouring samples. On the
quick 16k planet a sample is 71 km apart on the Earth-sized model sphere but 8.7 km on the planet,
and heights are not scaled: the step is drawn 8.1 times narrower than Earth's margins, and the
surface stage inherited a wall that nothing under the sea relaxes (marine diffusion vanishes below
wave base; slope failure moves only sediment).

**What the model does now.** Each 2 Myr step, after the collision belts, subduction uplift and the
orogenic collapse and before the surface processes:

1. *Who takes part.* For every plate, the edges between one of its visible continental samples and
   one of its visible oceanic samples are its ocean–continent boundary. Samples at a trench, in a
   subduction zone or on an arc, and continental samples an orogeny raised within the quiet time
   (PLT-259), are left out: an active margin or a mountain front is not a rifted margin. So are
   oceanic samples whose crystalline crust is thicker than a continent's: the plate model keeps
   crust stacked at resamplings and accreted from slabs under the oceanic label (on master, in the
   L1 map of one 1000 Myr run, supercontinent seed 2, 13 % of the ocean floor is thicker than
   35 km and the thickest is 240 km), and that is not
   the ocean floor beside a rifted margin.
2. *The taper* (PLT-261). Every sample within W/2 of the boundary (W = `transition_width_km`,
   200 km measured **on the model sphere**, PLT-262; on the planet until 2026-10-22, see "The
   transition's width and the drowned continents" at the end of this page) gets a signed distance
   d from it (inland positive)
   and a target crystalline thickness (the crust less its sediment and sedimentary rock)
   T*(d) = T_o + (T_n − T_o) · S(1/2 + d/W), with T_n = 33.0 km (the normal continental
   column's) and T_o = 7 km (the ocean's). Since 2026-10-23 S is a smoothstep, 3s² − 2s³ on
   [0, 1] (PLT-475, stage 2 of [the lithosphere's thermal state](lithosphere-thermal-state.md)):
   β = T_n / T* stays near 1 over the zone's inner quarter, rises fastest at the boundary and is
   3 or more over its outer quarter, the proximal, necking and hyperextended domains; before, S
   was the linear clamp(s, 0, 1). β runs from 1 at the hinge to T_n / T_o ≈ 4.7 at the toe, and
   the profile is centred on the model's boundary and symmetric about it, so turning the
   one-sample step into it keeps the crust's volume. A continental source thinned by β has its
   lithosphere and keel thinned by β too (PLT-476).
3. *Moving the crust.* Crust moves from samples thicker than T* to samples thinner than T* within a
   stretch of margin (boundary samples grouped round seeds at least W/2 apart; every sample of the
   zone belongs to the seed of its nearest boundary sample, through its own crust and plate), the
   share 1 − e^(−δt/τ) per step of what would bring the stretch to the profile (τ =
   `transition_myr`, 20 Myr, PLT-263), the sources thinned in proportion (pure shear), as their
   own rock. A switch added on 2026-10-19 (`transition_ocean_crust`, PLT-294, "Ocean crust keeps
   its own rock" below), on in every shipped preset since 2026-10-20 (decisions log 2026-10-20),
   gives an oceanic sink the same volume as ocean crust instead, lavas and gabbro in the ocean
   column's proportion. Volume is conserved, and crust types do not change: the thickened
   ocean-side samples stay oceanic, so the continental area and the crust share are not redefined
   by this rule.
4. *Heights* (PLT-264). A metre of crust moved changes the height by 1/f, f = (T_n − T_o) / (z_0 −
   z_f) = 4.24 m/m with the presets' continent (z_0 = 525 m) and old ocean floor (z_f = −5,600 m),
   on both sides, so the two sides meet. An oceanic sample of the zone then cools with its age (the
   dampening, PLT-129) towards the floor its crust holds up, z_f + (min(T, T*) − T_o)/f, instead
   of z_f (never above what the taper's own crust holds up, about ridge depth beside the
   boundary): a young margin stands near ridge depth, and the taper deepens as its ocean ages, the
   age–depth law standing for McKenzie's thermal subsidence.

The plate model's height change is its uplift for the surface stage, which carries it under its
own elevation and flexure; the shelf comes from the sediment the surface brings onto the thinned
crust, and its slope failure keeps that sediment at most 2.5° steep.

**Earth against the model.** Measured 2026-10-04, with W then measured on the planet (200 km
there was 1,632 km on the model sphere of the quick 16k planet, so every figure in this table and
the paragraph after it is for that width; the figures for 200 km on the model sphere are in "The
transition's width and the drowned continents" at the end of this page), on `earthlike_quick_16k`
(coupled pipeline to the volcano stage, `structure.deformation.density=0`; master = the same build with
`tectonics.transition_width_km=0`, which returns before touching anything; `tools/review/margin-runs.sh`,
transects by `tools/review/margins.py` with the margins study's definitions, so the ETOPO 2022 figures
compare). Passive margins are the transects with no trench, arc or collision within 6 L1 cells of the
coast. Master → the transition (W = 200 km), mean or median over the runs:

| | Earth | master | 200 km | 100 km |
|---|---|---|---|---|
| passive slope 500–2,000 m, median (500 Myr, 12 runs) | 2.1° | 4.5° | **2.6°** | 3.0° |
| passive slope, share under 3° | (median 2.1°) | 0.24 | **0.60** | 0.50 |
| all slopes 500–2,000 m, median (active ones 11.3° in all three) | 2.1° | 6.3° | 4.8° | 5.0° |
| passive shelf (shore to the break), median / mean | 88 km mean (Harris) | 1.0 / 16.9 km | **8.0 / 31.8 km** | 2.0 / 20.6 km |
| passive shore to the 200 m isobath, median / mean | 25 / 51 km (ETOPO) | 7.5 / 18.1 km | **12.5 / 29.0 km** | 9.0 / 20.9 km |
| land at 500 Myr, seeds 1–3 × both starts (1–6) | 0.29 | 0.235 (0.225) | **0.184 (0.178)** | 0.182 (0.202) |
| land 50 Myr in, 12 runs | | 0.374 | 0.339 | 0.357 |
| median sea depth at 500 Myr | 4,062 m | 3,889 m | 3,232 m | 3,643 m |
| continental crust at 500 Myr (1–6) | 0.41 | 0.390 | 0.387 | 0.360 |
| land at 1000 Myr, supercontinent seeds 1–4 | | 0.160 | 0.137 | |
| continental crust at 1000 Myr (lowest step) | | 0.424 (0.36) | 0.395 (0.38) | |
| plate-check targets met, 1000 Myr | | 27 of 45 | 31 of 45 | |

The slope of passive margins comes to Earth's within half a degree and their shelves widen, but the
land falls by about 0.04–0.05, early and for good: 50 Myr in, the drop is already 0.035 at 200 km and
0.017 at 100 km, in proportion to the width. That is geometry, not a defect of the rule. Sea level
crosses the taper where the crust has lost (z_0 − 0) · f ≈ 2.2 km, 83 km inland of the model's
boundary at W = 200 km: every rifted margin drowns a band of continent as wide as an Earth shelf. On
Earth that band (passive shelves, about 105,000 km × 88 km) is about 4 % of the continental crust; on
the quick 16k planet, whose radius is 782 km (1/8.1 of Earth's), the same band in metres is about
eight times that share, because a continent's coast shrinks as its size and its area as the square.
An Earth-width margin on a small planet therefore costs land unless the planet has more continental
crust. The ocean beside the margin is shallower too (the held floor over a band that is a large
share of a small ocean). The plate-check gain (27 → 31: the lowest crust step, the largest land gain
in a step, older ocean floor and the rift families) is partly chance: the rule changes every run's
course.

**Limits and game choices.**
- The profile is the same on every margin (a smoothstep since 2026-10-23, linear in thickness
  before; Earth's margins have a necking zone where the crust thins fastest, which the smoothstep
  draws), and volcanic and magma-poor margins differ in width by a factor of two to three. One width (200 km, inside the sources' 50–300 km) stands for all.
- The width was in metres on the planet until 2026-10-22, so a small planet's margins were as wide
  as Earth's in metres and took eight times the share of its continents (23 samples across on the
  quick 16k planet). It is now in kilometres on the model sphere (Ben, decisions log 2026-10-22,
  D6): the plates see Earth's width in their own Earth-sized frame, and the planet draws the margin
  R/R_model of that (24.6 km on the quick 16k planet, five L1 cells), with the heights unscaled.
  The ramp is then coarse on the quick planet: its 100,000 L0 samples are 71 km apart on the model
  sphere, so W is 2.8 samples across (the full planet's 250,000, 45 km apart, 4.4).
- τ (20 Myr) is a game choice: the sources give the duration of rifting (tens of Myr) and the
  thermal subsidence's time constant (62.8 Myr, carried by the ocean's age–depth law), not a time
  for the crust to take the taper's shape.
- The heights follow a straight line through two of the model's own columns rather than Airy
  isostasy with and without water: Airy gives 5.5 (dry) and 3.78 (water-loaded) metres of crust per
  metre of height, and 4.24 lies between; the line is what makes the plate model's continent and
  ocean heights meet.
- **Land against width.** On a planet much smaller than Earth an Earth-width margin takes a much
  larger share of the continents (above): the land at 500 Myr falls from 0.235 to 0.184 at 200 km
  (0.202 at 100 km over 12 runs). Getting Earth's land back needs more continental crust or a
  narrower margin; that choice is open (STATUS, pending questions).
- The rule moves crust only within a plate and only between visible samples; the thickened ocean
  side keeps its oceanic type and is subducted with its ocean when the margin later turns active.

## The planned force balance: its constants and targets checked (WP-13a)

Checked 2026-10-02, before the force balance is built, so that it is built on numbers that were
read. Nothing here is in the game yet; the rows are PLT-174 to PLT-202 (constants) and the targets
PLT-143 to PLT-162 and CHK-001 to CHK-057. No default of the design changed. Two plate-check ranges
did.

**What moves plates on Earth.** Forsyth and Uyeda (1975) solved for the forces that balance the
torques on twelve plates. The pull of sinking slabs and the resistance to their sinking came out an
order of magnitude larger than anything else. Plates attached to slabs move at 6–9 cm/yr, close to
a terminal velocity at which the slab's weight and the mantle's resistance balance; plates without
slabs move at 0–4 cm/yr; and drag under the continents is about eight times the drag under the
oceans. Later work splits the slabs' effect into a direct pull from upper-mantle slabs and a
"suction" from flow driven by lower-mantle slabs, and finds slabs supply over 90 % of the driving
force and ridge push under 10 % (Lithgow-Bertelloni and Richards 1998). Continental roots raise the
coupling to the mantle two to five times (Conrad and Lithgow-Bertelloni 2006).

**The constants.**

| Ledger id | Quantity | Design value | Source and how it was read | Status |
|---|---|---|---|---|
| PLT-182 | slab pull law | ∝ slab length × √(age of floor) | Faccenna et al. 2012, eq. 1 (after Turcotte & Schubert 1982); full text | `verified` (the 10 Myr cut-off is not sourced) |
| PLT-183 | size of slab pull | of order 10¹³ N/m | 1.4 × 10¹³ N/m for 80 Myr floor, from Faccenna et al.'s eq. 1 (computed) | `verified` |
| PLT-184 | slab length, break-off time | 700 km; 10 Myr | 700 km as Faccenna et al.; break-off 10 (young slab) to 20–25 Myr (old slab), van Hunen et al. 2011 abstract | `abstract-only` |
| PLT-185 | slab resistance | 3 | terminal velocity: Forsyth & Uyeda p. 178; the 3 chosen by the open-loop test | `calibrated` |
| PLT-186 | slab suction share | 0.3 of pull, overriding plate only | Conrad & Lithgow-Bertelloni 2002, 2004 (abstracts): suction 30–40 % of the forces, on both plates; their suction ÷ pull is 0.43–0.67 | `abstract-only` (not the same quantity; question for Ben) |
| PLT-187 | edge energy (ridge push, belt weight) | 0.1 of slab pull | ridge push 0.06 of slab pull per unit length (Forsyth & Uyeda Table 6); under 10 % of the driving force (Lithgow-Bertelloni & Richards 1998) | `verified` |
| PLT-188 | drag under continents | 3 times | 2–5 (Conrad & Lithgow-Bertelloni 2006, full text); about 8 (Forsyth & Uyeda, p. 189) | `verified` |
| PLT-189 | boundary resistance | 3 | Forsyth & Uyeda's transform resistance; the 3 chosen by the open-loop test | `calibrated` |
| PLT-192 | wedge surface slope | 1.5° | Lesser Himalayan wedge 1–1.6° (DeCelles & Carrapa 2021, full text); a critical wedge keeps its taper (Davis, Suppe & Dahlen 1983, abstract) | `verified` |

The design's 1.5° "taper" is a surface slope. In the theory of critical wedges the taper is the
surface slope plus the dip of the fault beneath; the Himalayan front has a taper of 6–7.5° made
mostly of that dip. The foreland line-load profile of the design (PLT-193) will not be built: the
general flexure already bends the plate under every load (decisions log 2026-10-02). Its setting,
the belt on the end of the stiff incoming plate, is built since 2026-10-24 (PLT-546, ISO-131; "The
incoming plate under the belt", below).

**Earth's figures for the targets.**

| Quantity | Earth | Source and how it was read |
|---|---|---|
| Area-weighted RMS plate speed | about 47 mm/yr today; 50–100 mm/yr over the last 200 Myr | Forsyth & Uyeda 1975 Table 1 (computed); Zahirovic et al. 2015 §3.2 (full text) |
| Plate speeds, 10th / 50th / 90th percentile | 11 / 33 / 80 mm/yr (today's 12 plates); median of 3,952 plate-samples over 200 Myr 42 | Forsyth & Uyeda Table 1 (computed); Zahirovic et al. §3.5 |
| Oceanic ÷ continental plate speed | 2–3 | Zahirovic et al., abstract and §3.5 |
| Slab-attached ÷ other plate speed | 3.0 (200 Myr); about 4 (today) | Zahirovic et al. §3.5; Conrad & Lithgow-Bertelloni 2004 (abstract); 3.8 from Forsyth & Uyeda Table 1 |
| Rank correlation of speed with slab share / with area | 0.71 / −0.34 (12 plates) | computed from Forsyth & Uyeda Table 1 |
| Sea-floor age: median, 90th percentile, share older than 180 Myr | 55 Myr, 133 Myr, 0.6 % (oldest 339 Myr) | Seton et al. 2020 age grid, area-weighted (computed) |
| Supercontinent tenure; cycle | 100–300 Myr; 400–800 Myr, about 600 dominant | Mitchell et al. 2021 (accepted manuscript, full text); Nance 2022 gives about 500 |
| Largest plate; plates over 5 % of the sphere | 20.5 %; seven | Bird 2003, Table 1 (full text) |
| India–Asia convergence after collision | about 15 to about 4 cm/yr between 50 and 35 Ma | Copley et al. 2010 (abstract) |

Two plate-check ranges were wrong and were changed: the area-weighted RMS speed (35–55 → 40–100
mm/yr; the old top left out every Earth value of the last 200 Myr) and the supercontinent period
(300–600 → 400–800 Myr; 300–500 was a Wikipedia figure the cited papers do not give). A test in
`plates/check.rs` (`earth_figures_pass_their_targets`) holds every Earth figure above inside the
range it anchors. Three targets are lenient against Earth and are questions for Ben in STATUS.md:
the sea floor's 90th percentile ceiling (230 Myr against Earth's 133), the collision length (15–50
Myr against Earth's examples of 50–140) and, for the design, the speed scale and the suction share.

## Plate-check: boundary mix and arc heights

Added 2026-10-02 (PLT-213 to PLT-221; measured values in `docs/design/plate-check-baseline.md`).
`deeptime plate-check` now records and judges two things the plate rules were not judged on before.
They exist because a workbench run 28 Myr after a supercontinent start showed nearly every boundary
between ocean plates as a subduction zone with a continuous volcanic arc high enough to carry snow.

**What happens on Earth.** Of the 260,487 km of present plate boundary in Bird's PB2002 model
(Bird 2003, Table 3), 19.7 % is subduction zone (51,310 km), 36.4 % is spreading (ocean ridges
25.8 %, continental rifts 10.5 %), 28.4 % is transform, 8.8 % is continental collision and 6.7 % is
oceanic convergence without a Benioff zone or arc. Subduction is concentrated round the Pacific;
most of the length inside an ocean is ridge and transform. Pangaea is commonly described as ringed by subduction (not sourced here), so a
supercontinent planet may sit above 19.7 %, which is why the target (15-25 %) is a range and the
mix over the first 50 Myr after a supercontinent start is reported beside it, without a judgment.

Arcs differ by what they stand on. Intra-oceanic arcs (the Marianas, Tonga-Kermadec, Izu-Bonin)
are chains of volcanoes mostly below the sea: along the Mariana arc 20 of the 26 active volcanic
centres are submarine (Baker et al. 2008), the Kermadec arc's vents lie 250-1,660 m below the sea
(de Ronde et al. 2003), and the highest island-arc cones are about 1 km. Continental arcs stand on thick
crust and are high: the Cascades' basement under the chain is 1.2-2 km and a dozen cones exceed
3 km (Hildreth 2007); the Central Andes' arc sits on a 4.5 km plateau (Gregory-Wodzicki 2000).

**What the model measures.** Every step, the plates stage counts the boundary by kind (ocean-ocean
and ocean-continent subduction, collision, ridge, rift, other) and the elevation of its arc cells
(median, 90th percentile, share above the sea datum) split into oceanic and continental arcs. The
definitions are PLT-220 and PLT-221. Nothing a plate rule reads depends on them.

**The model today** (baseline of 2026-10-02, mean of four 1000 Myr supercontinent runs): 56 % of its
boundary is subducting (191,000 km a step against Earth's 51,310 km; its spreading share, 39.5 %, is
close to Earth's 36.4 %, and the model has almost no transform boundary, 1.3 % against 28 %). Its
oceanic arcs are mostly submarine (14 % of arc cells above the sea, crest 312 m) and its continental
arcs stand 1.7 km median, 2.9 km crest, all inside the Earth ranges; the subducting share is the one
measure outside its range.

**Limits.** Lengths are counts of lattice samples times the sample spacing (about 70-80 km), so the
subducting share is an estimate in the same unit as the slab front length, not a surveyed length.
Bird's classes are not the model's: his subduction zones were marked by hand where a Benioff zone
or an arc is, and the model's "other" also holds converging boundaries that have no contact yet.
Since 2026-10-20 the model's transform is Bird's (PLT-311): motion within 20 degrees of the
boundary's own direction, the direction from the boundary's normal over two rings of samples
(PLT-310). Until then each sample judged the motion along its own lattice edges, which zigzag
about 30 degrees either side of a boundary, and read strike-slip as convergence on one edge and
divergence on the next (section "Land shape and the high ground" below has the before and after). The arc
comparison is by cells (area) against Earth's count of volcanoes. The oceanic arc crest limit
(1 km) and the Andean crest limit (6 km) are from memory (PLT-217, PLT-219). The target ranges
fail today's model by design: they are what the force-balance and belt tasks are to move.

## Arcs on a lattice coarser than the arc band (2026-10-19)

**What was wrong.** The GameTest planet (`tests/presets/planet_world_4k.json`, 5,000 samples) ended
every run without an active arc volcano, on master and on every earlier commit measured. The arc
band is where the arc weight w_arc = exp(-((d - 200 km) / 60 km)^2) passes 0.3 (PLT-042, PLT-043):
d from 134 to 266 km behind the slab front, a band 132 km wide. A front's distance field runs over
the lattice from its front samples (d = 0), and 5,000 samples on the plate model's sphere (Earth's
radius) stand about 320 km apart, so the first ring behind the front lies near 320 km and no sample
fell in the band. No sample took the ARC flag, the volcano stage saw no arc (VOL-002, VOL-003) and
no stratovolcano was ever born. On Earth 19.7 % of the plate boundary is subduction zone, which
Bird's model sets apart from the 6.7 % of oceanic convergence without a Benioff zone or arc (section
"Plate-check" above): where a slab sinks there is an arc, so a mobile-lid planet with subduction and
no arc is a fault of the lattice, not physics. The game presets are not affected: their 100,000 and
250,000 samples stand 71 and 45 km apart. The first ring falls beyond the band once the spacing
passes 266 km, below about 7,000 samples; the preset editor allows down to 2,000 (500 km).

**The change.** Each front sample's strip (the samples its distance field reached) now also flags
its sample with arc magma nearest the arc axis, when that lies within one lattice spacing of the
axis (PLT-043). On a lattice finer than the band that sample is inside the band already, so the
rule adds nothing there. The flag still marks only samples that took arc magma. The magma is not
changed: on a coarse lattice the arcs still add far less crust than on a fine one (the weight at
320 km is 0.018 of the axis's), which is left as a limit of coarse lattices.

The volcano stage's blind step changed with it (VOL-011, [volcanoes-and-magma.md](volcanoes-and-magma.md)):
the stage is blind exactly at the plate model's resampling steps, which clear the flags, and no
longer at any step with a fifth of the last valid step's arc cells or fewer. The last valid step's
count only moved on valid steps, so a real fall in the arcs blinded the stage for many steps: on
seed 16, 24 of 75 steps were blind, 9 of them resamplings.

**Measured.** `deeptime run --threads 8` on the test VM (scratch scripts), active volcanoes in
`volcanoes/active.json` at the end; planet_world_4k is 150 Myr from scattered continents, seed 11
unless named. Runs at a fixed thread count were identical.

| Build | Runs | Active volcanoes (arc stratovolcanoes / hotspot shields) | Arc-flagged cells, most in a step |
|---|---|---|---|
| master 025676c8 | 6 | 0 (0 / 0) in every run | 0 |
| fb38204f (the commit before the solvers merge) | 6 | 4 (0 / 4) in every run | 0 |
| master 26ce664e with the old solvers (`surface.numerics.diffusion_solver=jacobi`, `flood=serial`) | 1 | 4 (0 / 4) | 0 |
| master at 20,000 samples (`resolution.l0_samples=20000`) | 1 | 21 (19 / 2) | 888 |
| the arc rule alone, seeds 11 to 16 | 6 | 6, 12, 8, 11, 9, 7 (3 to 10 / 0 to 3) | 561 to 1,891 |
| the arc rule and the exact blind step (this branch), seeds 11 to 16 | 6 | 6 (3 / 3), 12 (9 / 3), 5 (3 / 2), 11 (10 / 1), 9 (9 / 0), 6 (5 / 1) | 524 to 1,891 |
| earthlike_quick_16k, scattered continents, 150 Myr, seed 1, master 26ce664e | 1 | 49 (48 / 1) | 15,924 |
| the same, this branch | 1 | 49 (48 / 1), every volcano count the same (3,037 born, 9 blind steps, the resamplings): on 100,000 samples neither change fires | 15,924 |

Master's 4 to 0 is not an arc change: neither planet had an arc. The solvers merge changed the
surface, and with it which hotspot volcanoes stood at the end (a hotspot's next volcano waits until
its earlier ones, dead ones included, have drifted 70 km away, VOL-007, and erosion decides when a
dead one leaves the catalogue, VOL-049); master with the old solvers has the 4 shields again.

**Also found, not changed: the hotspot population.** Hotspots are born at 3 per 100 Myr and live
40 to 150 Myr (PLT-109, PLT-110), so a planet holds about 3 live plumes (3 × 95 Myr / 100 Myr) once
the first 8 (PLT-108) have died, which they all have by 150 Myr. Earth's catalogues hold 26 to 69
(Davies 26, Sleep 37, Steinberger 44, Courtillot et al. 49, Anderson more than 60, Ito and van
Keken 69, as listed by King & Adam 2014, read in full; spot check PLT-108/109/110), with "a few
dozen" the usual view (Wikipedia "Hotspot (geology)", secondary). Sleep's 37 at a mean life of
95 Myr would take births near 39 per 100 Myr and 37 at the start. Plume heads (LIPs) come with half
of the births (PLT-117), 1.5 per 100 Myr or one per 67 Myr (the volcano page measured one per
59 Myr), against Earth's one per 22 Myr (Courtillot & Renne 2003); at 39 births a head share of
about 0.12 would give Earth's rate (the share of primary hotspots, seven on Wikipedia after
Courtillot et al. 2003 of the 49 they considered, 0.14, gives one per 18 Myr). This changes every planet, its LIP CO2 and the share codes,
and touches the carbon cycle under investigation for the hothouse 16k planets, so it is left for
its own task (STATUS known issues). The rows PLT-108 to PLT-110 and PLT-117 stay `uncited`.

## The continental crust budget: resampling footprints and island-arc maturity

Added 2026-10-02 (ledger PLT-045, PLT-047, PLT-228, PLT-229; build log "Continental crust fix").
Ben's answer of that day ("Investigate, then fix arc emergence") followed a bisect. With general
flexure on, the continental crust ended 1,000 Myr runs at 0.25-0.27 of the planet instead of
about 0.40.

**What happens on Earth.** Continental crust is made at subduction zones and destroyed there.
Stern & Scholl (2010, read in full) put creation and destruction in balance at about 3.2 km³ a year
or have destruction ahead. Of the creation, about 2.7 km³ a year is at convergent margins. The
Izu-Bonin-Mariana and Aleutian arcs, active for 45-50 Myr, grew 90-180 km³ per km of arc per Myr.
Growth is fastest in an arc's first few Myr. Intra-oceanic arc crust thickens by "several hundred
metres per million years". Its Moho lies at 25-35 km, thinner than continental crust (typically
about 40 km) and more mafic. Because crustal thickness shows in elevation, these arcs "are
generally mostly below sea level" (Stern 2010, read in full). What makes arc crust behave like a
continent is buoyancy, not height. Cloos (1993, abstract) finds that only crust thicker than about
15 km makes the lithosphere buoyant enough to jam a subduction zone. Island arcs built on ocean
crust "typically must be active for more than ~20 m.y." to reach that thickness. Tetreault & Buiter
(2014, read in full) give island arcs an average of 26 km at 2.79 g/cm³. They also note that whole
island-arc sections are rare in the geological record, which goes against Cloos's threshold.

**What the model did.** The plate model's continental area came out of two churns, each roughly as
large as the whole crust over 1,000 Myr:

- *The resampling lost crust.* Every few steps the crust is resampled onto a new lattice, about 110
  times in 1,000 Myr (PLT-228). A new point used to count as inside a plate only when a whole
  lattice triangle of that plate held it. A point on a plate's boundary strip copied the nearest
  visible sample, but only when no other plate covered it. At a subduction zone the subducting
  plate's hidden crust covers the overriding plate's strip. So the overriding margin, often a
  continent's edge, lost its outer half-spacing at every resampling. Measured on seed 1 of
  `earthlike_quick_16k` (supercontinent start), the continental area fell by a mean 0.77 % per
  resampling: 344 samples, 29 resamplings over 300 Myr. That was a conservation bug.
- *Arcs above the sea turned continental.* A rising island arc became continental once it had
  80 m of the model's arc magma (`arc_continental_m`), but only while it stood above sea level that
  step. The 80 m was tuned to keep the crust near 0.40 against the resampling's loss (P1-27). It
  converted about 50,000 samples in a 1,000 Myr run, half the planet's samples. When flexure began
  to bend the plate under the arcs' load, they stood about 260 m lower, fewer crossed the sea, and
  the budget fell.

**What the model does now.**

- *Resampling by footprints* (PLT-228). A plate's extent is its footprint: the old lattice's cells
  its samples owned, moved rigidly with it. Each lattice triangle is split between its corners by
  the largest barycentric weight. Footprints tile the sphere at a resampling, so a plate keeps its
  area until it really overlaps another or leaves a gap. Where footprints overlap, the crust on top
  is chosen in this order: visible, the overriding plate of a subduction contact, continental,
  higher. A test resamples a static two-plate planet and one with an ocean running under a
  continent, on six lattices each: continental area is conserved within 0.2 %. The old rule lost
  0.35 % at the converging margin; the new one loses 0.03 %. On real runs (seed 1, the same
  states), the old rule would lose 0.92 % per resampling and the new one loses 0.33 %. Over a
  1,000 Myr run the new rule's loss (0.38 % per resampling) is real overlap: 82 % is continental
  crust under other continental crust, where colliding continents overlap, and 18 % is continental
  crust under an overriding oceanic plate. Neither is a lattice artefact. A subduction initiation
  no longer splits off a plate that is at least half continental (PLT-229). With the fix, one test
  history's initiation had split off a plate that was 0.89 continental.
- *The new lattice's triangulation* (P1-19, numerics only). Every lattice is the same Fibonacci
  sphere under a random rotation, and a rotation keeps a Delaunay triangulation Delaunay. A
  resampling therefore turns the first lattice's triangles, neighbours and edge lengths with the
  points instead of triangulating them again. Only where four points are cocircular to rounding
  can a diagonal differ from a fresh build's, and either diagonal is Delaunay.
- *Island arcs mature by the crust they built* (PLT-047). An oceanic sample becomes continental
  crust once its accumulated island-arc magma reaches `arc_magma_m_per_myr` × `arc_maturity_myr`,
  whatever its height. Hidden crust does not convert. `arc_maturity_myr` is 20 Myr in every preset,
  Cloos's lower bound. The model's arc magma (20 m per Myr at the arc axis at full convergence) is
  25-90 times smaller than Earth's arc growth (PLT-045) and is not a thickness, so the rule counts
  Cloos's time in it, at the model's full arc rate. An arc at a trench converging at half of v_0, or
  off the arc axis, takes longer. No value was tuned against the crust share. (Since plate task 8,
  2026-10-17, the arc rate is Earth's and a physical thickness, and the rule counts Cloos's 15 km
  instead: `arc_mature_crust_km`; see "The crust budget on sourced rates" below.)

**Earth against the mod.** `deeptime plate-check`, `earthlike_quick_16k`, supercontinent start,
1,000 Myr, flexure on, mean of seeds 1-4 (per-seed values in the build log):

| Build | Continental crust at the end | Lowest step | Land, median | Largest continental piece | Arc samples turned continental |
|---|---|---|---|---|---|
| before (master) | 0.34 | 0.33 | 0.31 | 0.59 | 51,836 |
| resampling by footprints only | 0.55 | 0.43 | 0.37 | 0.83 | 45,070 |
| arc maturity only | 0.15 | 0.15 | 0.25 | 0.37 | 8,532 |
| both (the model now) | 0.31 (seeds 0.27-0.36) | 0.31 | 0.31 | 0.42 | 6,535 |

The 500 Myr hypsometry ensemble (seeds 1-3, both starts) moves from land 0.300 and crust 0.418 to
land 0.291 and crust 0.387, with a narrower spread between runs.

**What it means.** The crust budget no longer depends on the surface stage's height.
Seeds 1 and 2 with `surface.flexure.strength` 0 end at 0.28 and 0.37, against 0.33 and 0.30
with flexure on. Before the fix flexure cut them from 0.39 and 0.40 to 0.27 and 0.25.
Island arcs now add about 6,500 continental samples per 1,000 Myr instead of about 50,000. That is
about 6.5 % of the planet's area per Gyr. As a rough check, Stern & Scholl's 2.7 km³ a year,
spread at 40 km thick, is about 13 % of Earth's surface per Gyr (that conversion is this page's
arithmetic, not theirs). The losses now set the share: continental crust that goes under at
collisions and overlaps in them. Counted over 1,000 Myr on seeds 1 and 2 (100,000 samples, 42,254 continental at the start),
continental crust hidden under other plates and not welded back cost about 5,000 samples. Crust
consumed where continents overlap at a resampling cost about 13,000. Arcs added 6,000-7,900. The model has about 11 collisions per 100 Myr against a
target of 2-6, so it destroys continental area too fast. That is the collision and force-balance
work (plate tasks 3-5), not a calibration of the arc rule.

**Limits.**
- Cloos 1993 was read as an abstract only.
- His 20 Myr is a lower bound ("more than ~20 m.y."), and he states it as a thickness reached, not
  as a time.
- Counting it at the model's full arc rate is this fix's choice.
- A sample turns continental all at once. Its height does not change, so a mature arc may be
  continental crust 2 km below the sea, like a submerged continental fragment. Its crust is not
  thickened to a continent's.
- The resampling's overlap order (visible, overriding, continental, higher) is a rule, not physics.
- Where two visible continents of plates with no subduction contact between them overlap, the
  losing crust's rock pools are not kept: volume goes with the area. This was the old behaviour
  and is unchanged.

## The continental crust budget under the force balance (plate tasks 7 and 8, 2026-10-05)

Ledger PLT-242 and PLT-256 (values changed), PLT-276 and PLT-277 (new). Ben's answers of
2026-10-05 ("Keep 200 km, raise crust in 7–8"; and the validation policy of the same day: physics
from sources, the land share reported against Earth's history, not tuned towards a target), and
the coordinator's choices of the same night (the stacking share from Stern & Scholl 2010; the
plateau rule checked for physics).

**What the switch did to the budget.** Under the force balance a collision keeps converging after
its onset (the convergence 20 Myr in is 0.3–0.5 of the onset's, inside the 0.2–0.6 that India
gives), so about twice as much continental crust crosses collision fronts as under the old speed
law (`plate-check`, 1000 Myr, eight runs: 17 + 19 million km² per 100 Myr in collisions that end
without and with a transfer or weld, against 8 + 7 on master). Four things destroyed that crust
much faster than Earth does:

1. *Counted twice* (fixed, PLT-276). The belt stacks `collision_stacking_share` of the crust that
   crosses the front; the crust itself, hidden under the overriding plate, was also accreted to the
   column on top at the next resampling. Up to 1.8 times the crossed volume reached the overriding
   plate, and the thickest-crust limit removed the excess.
2. *A fifth sent down at every front* (PLT-242, 0.8 → 0.95). Stern & Scholl (2010, after Scholl &
   von Huene 2009) put the recycling at suture zones at about 115 km³ per km of suture per Myr, of
   which 50 is deeply subducted lower-plate crust; about 1,600 km³ per km per Myr crosses a model
   front at 40 mm/yr, so a retained share of 0.93–0.97.
3. *Foundered with nowhere to go* (changed, PLT-277). Crust beyond `crust_max_km` (70 km) went to
   the mantle at once; the collapse could only move crust that no orogeny was raising. Now the
   excess first spreads: it fills continental crust up to `collapse_thickness_km` and ocean floor
   up to the normal column within `collapse_reach_km`, and only what finds no room founders.
   Filling neighbours up to the 70 km limit itself was tried and dropped: every long collision
   then widened into a 70 km plateau (0.35 of the land above 2 km at 500 Myr, 0.41 at 1000 Myr).
4. *Collapse stopping at 55 km* (PLT-256 now 50 km, the figure the sources state).

| `earthlike_quick_16k`, 500 Myr (km³ a year, Earth-normalized) | Branch as found (4 runs) | Now (6 runs) | Earth today |
|---|---|---|---|
| Continental crust crossing collision fronts | 13.4 | 15.0 | n/a |
| Lost at the front (not stacked) | 3.0 | 2.3 | deep lower-plate crust at sutures about 0.3 |
| Foundered beyond the thickest column | 14.4 | 5.7 | (included above) |
| Accreted at resamplings (of it, a collision's crust a second time) | 36.8 (about 9) | 23.8 (0) | n/a |
| All crustal losses | n/a | n/a | more than about 3.2 (subduction erosion 1.5, sediment subduction 1.0, collision zones 0.7) |
| All crustal additions | n/a | n/a | about 3.2 (oceanic arcs 1.5, continental arcs 1.0, rifts and hotspots 0.5, collision zones 0.2) |

Earth's column: Stern & Scholl 2010 (full text, Figure 3 and pp. 18–19). The model still loses
continental crust at collision zones many times faster than Earth: about 8 km³ a year in all,
mostly what founders because a long collision stacks crust faster than 200 km of reach can
take it.

**What it gives** (reported against Earth, not tuned). 500 Myr, seeds 1–3, both starts:
continental crust 0.30 of the planet (master 0.38), 0.57 of it dry (master 0.46), land 0.18
(master 0.18), median land 900 m (master 410), land above 2 km 0.25 of the land (master 0.08).
1000 Myr, seeds 1–4: crust 0.31 (master 0.40), land 0.19 (master 0.14), land above 2 km 0.27.
Earth's land share over the Phanerozoic was about 18–30 % (Ben's reference range, 2026-10-05);
today's is 29 % with continental crust (shelves included) about 41 %, and about a tenth of the
land stands above 2 km (from memory).

**Why the high ground (a model issue; the isostasy part fixed 2026-10-06, see the next section).**
Collapse thins quiet crust only to 50 km, and a 50 km column stood at its Airy height with the
upper crust's density, about 3.2 km (the middle of `continent_elevation_m` plus 15 km / 5.5).
Earth's crust of that thickness under an old orogen stands far lower (the Urals keep 50–58 km,
Carbonell et al. 1996 and 2000, at a mean altitude of 0.5–0.8 km, Artyushkov et al. 2000, snippet
only). Erosion is not what holds it up: the erosion audit of 2026-10-05 finds land above 2 km
eroding at 116–144 m/Myr (mean), the rate of mountain ranges and 5–10 times faster than a real
plateau interior (southern Tibet under 0.01 km/Myr since about 11 Ma, Tremblay et al. 2015,
ERO-174). The high ground is the crust supply (the 50 km collapse floor and the belts held at up
to 70 km) at the height isostasy gives it.

**Limits.**
- Which of spreading and foundering comes first, and the reach, are the model's.

## The eclogite root and whole-column isostasy (plate task 7, 2026-10-06 to 2026-10-16)

Ben's choice of 2026-10-06 (decisions log, "Fix isostasy first, then land (Recommended)"): thick
continental crust gets a dense root before task 7 lands, so that a 50 km crust stands near the
Urals' height rather than 3.2 km. Ledger PLT-278 to PLT-282, ISO-043 and ISO-046; sources in the
bibliography, "The eclogite root and whole-column isostasy". Read by a research sub-agent on
2026-10-05; the rows say which were read in full.

**What happens on Earth.**
- *Height against thickness.* Hyndman & Currie (2011, full text) measure 131 ± 12 m of elevation
  per km of crust in the Cordillera and 141 ± 27 m/km in stable areas. Airy isostasy gives
  (ρ_m − ρ_c) / ρ_m of height per unit of crust: 142 m/km with the continental crust's mean density,
  2,830 kg/m³ (Christensen & Mooney 1995, abstract), and 182 m/km with the upper crust's 2,700,
  which the model used for every change of a column's thickness until 2026-10-06.
- *Eclogite.* Mafic lower crust carried past about 1.5–2.0 GPa (about 60 km under southern
  Tibet, Wittlinger et al. 2009) can turn to eclogite, 3,450 kg/m³, denser than the mantle. Whether it does depends on temperature, fluids and
  time, not the pressure alone: the Indian lower crust under southern Tibet, below about 60 km, is
  about 30 % eclogitized and gravitationally stable (Shi F. et al. 2018 and Shi D. et al. 2020,
  full texts). Other readings of the same layer differ (Wittlinger et al. 2009: fully eclogitic;
  Krystopowicz & Currie 2013: 20–90 % for the density contrasts observed).
- *The Urals.* An old orogen keeps 50–58 km of crust (Carbonell et al. 1996 and 2000, abstracts)
  at a mean altitude of 0.5–0.8 km (Artyushkov et al. 2000, snippet only). The sources put the low
  stand down to dense mafic rock at the base of the crust that is not eclogite (Stadtlander,
  Mechie & Schulze 1999, abstract; Leech 2001, snippet only) and to the cold lithosphere of old
  orogens: at equal crustal thickness, the thermal state moves elevation by about 1,600 m
  (Hyndman & Currie 2011, full text).

**What the model does now.**
1. *Whole-column isostasy* (PLT-281). When crust is stacked onto a column, spread from it or
   founders (belts, collapse, the thickest-crust limit), and under a cordillera's root (PLT-051),
   its height changes by the thickness change over ρ_m / (ρ_m − ρ̄_c) = 7.0, not 5.5. Erosion
   from the top of a column keeps the upper crust's 5.5 (ISO-008): the rock it removes is upper
   crust.
2. *The eclogite root* (PLT-278 to PLT-280, the rule PLT-282). The column's mafic pools (mafic
   plutonic and mafic volcanic, which the model keeps at the bottom) lying deeper than 1.5 GPa
   (54 km at Earth's gravity) are eclogite at a share of 0.3; a root of E metres lowers the surface
   by E × (3,450 − 2,830) / 3,300. The root stays crust: at 30 % it is lighter than the mantle.
3. *The surface flexure* (ISO-046) infers the crust thickened behind a plate uplift with the same
   mean density, so the thickening it passes to the flexed plate is the one the plate model made
   (7.0 times the uplift, not 5.5). Since 2026-10-22 the uplift it reads for this is the change of
   the column's own crust's Airy height (`plates/crust_uplift_mm_yr`, ISO-091 and ISO-092), not
   the column's whole rise: the settling's and the height rules' moves are no load.

**What the root can and cannot do.** The model's 35 km continental column stands at about 525 m
(the middle of `continent_elevation_m`). A 50 km column now stands at about 2.66 km (3.2 km
before): it has no root, the eclogite depth being 54 km. A 70 km column made by doubling the
standard one (9 km of mafic pools) has a 2.7 km root that pulls it down by about 0.5 km, to about
5.0 km. So the root is small at the thicknesses the model holds; the whole-column density does
most of the work. The Urals' height at 50 km needs the dense non-eclogite base and the cold
lithosphere, and the model carries neither (one mean crustal density for every column, no
lithosphere temperature). The 1 km target is not reached, and nothing was tuned to reach it.

**Measured** (`earthlike_quick_16k`; reported against Earth, not tuned; mean [lowest, highest]
of the runs). Hypsometry at 500 Myr, seeds 1–3, both starts (`tools/review/force-balance-variants.sh
… final.txt 500 1,2,3 … hyps`, summarized by `tools/review/p1-17-hyps.py`). Earth's line is the
project's reading of a relief grid (HYP-001 to HYP-021; its median not confirmed, claims.md):

| | Land | Median land height, m | Land height, 90th pct, m | Land above 2 km (of the land) | Continental crust (of the planet) | Of it dry |
|---|---|---|---|---|---|---|
| Earth | 0.289 (0.18–0.30 over the Phanerozoic) | 427 | 2,022 | 0.102 | about 0.41 (shelves included) | about 0.70 |
| Master, 2026-10-05 | 0.184 | 414 | n/a | 0.083 | 0.382 | 0.46 |
| Branch before the root, 2026-10-05 | 0.183 | 900 | n/a | 0.25 | 0.299 | 0.57 |
| With the root and the whole column (2026-10-16) | 0.195 [0.124, 0.231] | 842 [659, 1,016] | 3,427 [2,746, 4,011] | 0.243 [0.190, 0.284] | 0.292 [0.234, 0.314] | 0.62 |
| Final: also the surface flexure at 2,830 | 0.191 [0.145, 0.214] | 745 [509, 1,058] | 2,907 [2,035, 4,196] | 0.187 [0.103, 0.274] | 0.307 [0.262, 0.335] | 0.58 |

The surface flexure feeds back into the plates (`surface/feedback_applied`), so the last two rows
are different planets, not one planet with one change; the spread between runs (land above 2 km
0.10 to 0.27 in the final row) is as large as the change between them.

**Plate-check at 1000 Myr** (the final code; seeds 1–4, both starts;
`tools/review/force-balance-variants.sh … final.txt 1000 1,2,3,4 … ck`): 31 of 45 targets met (31
with the root before the flexure change, 30 on 2026-10-05). The 14 that fail are the same with and
without the flexure change: collisions per 100 Myr 0.89 (target 2–6); continental crust at its
lowest step 0.25 (at least 0.37); active share of the continental margin 0.46 (0.25–0.40); slab ÷
free plate speed 1.82 (at least 2.5) and the rank correlation 0.45 (at least 0.65); rotation change
per step, 90th percentile, 25.2 % (at most 20); ocean floor older than 250 Myr 3.55 % (at most 3);
supercontinent tenure 53 Myr (100–300) and period 380 Myr (400–800); subducting share of the plate
boundary 52.6 % (15–25); oceanic arc crest 1,107 m (at most 1,000); the wedge 9 km wide (100–350)
and the foredeep 41 km wide and 113 m deep (150–450 km, 2–7 km). Pooling the supercontinent
episodes of the eight runs (`force-balance-series.py`): 2.5 assemblies a run, tenure 48 Myr, period
267 Myr (45 and 230 with the root alone, 46 and 213 on 2026-10-05). At 1000 Myr the same runs have
land 0.192 [0.170, 0.221], median land height 1,004 m [797, 1,274], 90th percentile 3,471 m, land
above 2 km 0.265 [0.182, 0.346] of the land, sea depth median 3,264 m (Earth 4,062), continental
crust 0.317 of the planet, 0.54 of it dry. The crust budget at 1000 Myr, over the whole run:
continental arcs 11.2 km³ a year, island arcs 0.18, foundered 12.4, collapse 7.9; 8.4 billion km³
of continental crust at the end.

**The crust budget against Earth** (km³ a year, Earth-normalized, mean of the six final 500 Myr
runs; new run-summary globals `plates/arc_continental_km3`, `plates/arc_oceanic_km3` and
`plates/continental_volume_km3`; Earth from Stern & Scholl 2010, full text):

| | Model, 500 Myr | Earth today |
|---|---|---|
| Added at continental arcs (arc magma and the cordillera's root) | 8.5 | about 1.0 |
| Added at oceanic (island) arcs | 0.14 | about 1.5 |
| Continental crust crossing collision fronts; kept in the belt | 14.5; 13.3 | n/a |
| Foundered beyond the thickest column | 7.0 | collision zones lose about 0.7, deeply subducted crust about 0.3 of it |
| Moved by orogenic collapse | 4.7 | n/a |
| Continental crust at the end, km³ | 7.3 billion | n/a |

So the model's continental crust is the balance of two rates neither of which is Earth's: the
cordillera roots create crust about 8 times faster than Earth's continental arcs (the root is 7.0
times an uplift calibrated to the arcs' height, PLT-048 and PLT-051, not crust shortened or a
sourced magma rate), and collisions
destroy it about 10 times faster than Earth's collision zones. Island arcs add a tenth of Earth's
rate (PLT-045). The model ends with about 7.3 billion km³ over 0.31 of the planet, a mean of about
47 km; Earth holds a similar volume over about 0.41 at about 40 km (both Earth figures from
memory). A fifth of the model's continental samples are 55–70 km thick (`crust_bins` in the run
summary); on Earth crust that thick is confined to the active orogens (from memory). There is
not too little crust.

**The land share: height has fallen behind thickness.** The run summaries' `crust_bins` give each
thickness band's share of the continental samples and its median elevation in the plate model. At
500 Myr (the six final runs, lowest to highest run):

| Crust thickness | Share of the continental samples | Median elevation, m | The model's own Airy height at the band's middle, m |
|---|---|---|---|
| 30–40 km | 0.22–0.32 | −72 to +51 | about 510 |
| 40–55 km | 0.30–0.39 | −71 to +763 | about 2,300 |
| 55–70 km | 0.17–0.28 | 779 to 2,281 | about 4,200 (the root included) |

The Airy height is the one the collapse already uses (`collapse::iso_height`: 525 m at the normal
35.1 km column, plus the extra thickness over 7.0, less the eclogite root's pull). So most of the
continental crust stands half a kilometre to three kilometres below the height its own thickness
holds it at, near sea level and half flooded (0.58 of the continental crust dry). At 1000 Myr
(eight runs) it is further behind: 30–40 km at −859 to −71 m, 40–55 km (0.35–0.45 of the samples)
at −191 to +61 m, 55–70 km (0.25–0.38) at 394 to 1,469 m. At its Airy height almost every
continental column would be dry and the land would be near the crust's 0.31 of the planet. So the
land share is set by where the surface has drifted, not by how much crust there is. Which processes
take the height and leave the crust is measured below (the gap budget). The high ground is partly
the same effect seen from the other side: land above 2 km is 0.19 of the land but about 0.036 of
the planet against Earth's 0.029 (0.10 of 0.289), so most of its excess over Earth's share is the
small land beneath it.

**Which process holds the gap (the gap budget, 2026-10-17).** The gap budget
(`plates/gap_budget.rs`, ledger PLT-283; on with `DEEPTIME_GAP_BUDGET=1`, tables by
`tools/review/gap-budget.py`) keeps each sample's gap, its elevation less the Airy height of its
own column, split by the process that changed it. Every process of a plate step books its change;
the surface's feedback is split into the rock eroded or laid down, its local (Airy) rebound, the
local answer to the change of sea water and ice, the flexure's regional answer to those loads, the
flexure's bending of the roots the plate model's uplift brings, its answer to the moment a
weakened plate releases, the rest of the flexure, the spreading, what the volcano stage and the
impacts add to the surface after the surface stage, and the carry (advection on L1, sampling); a
resampling books what its blend and its accretion change. Each part also keeps what of it was made while the sample was still ocean
floor. The parts add up to the gap in every run (worst residual 0.0 m) and the budget changes
nothing the model does (test `the_gap_budget_changes_nothing_and_adds_up`). The 40–55 km crust, a
third of the continental samples, at 500 Myr: mean of six runs (seeds 1 to 3, both starts) on the
code with the rain at Beck's 773 mm, beside four 1000 Myr runs and the four 500 Myr runs from
before the rain (seeds 1 and 2):

| Part | Stock at 500 Myr, m | of it made on the ocean floor | Rate, last 50 Myr, m/Myr | Stock at 1000 Myr | Stock at 500 Myr before the rain |
|---|---|---|---|---|---|
| **The gap** | **−2,326** | **−388** | **−12.8** | **−2,709** | **−2,333** |
| A resampling's accretion: crust under an overlap stacked onto the column on top, with no height | −4,046 | −395 | −5.7 | −6,809 | −4,039 |
| The plate model's ocean-floor step (`pools::surface_step`) | −2,142 | −2,142 | 0 | −3,700 | −2,156 |
| Flexure, and the local answer to the sea water and ice (split below) | −2,086 | −478 | −8.1 | −3,215 | −2,020 |
| Transition | −420 | +39 | −2.3 | −560 | −455 |
| Erosion and deposition, net of their local rebound | −401 | +300 | −3.3 | −571 | −360 |
| Arcs and Andean margins (`subduction::fronts_and_uplift`) | +3,071 | +1,471 | +7.0 | +4,924 | +3,118 |
| The thickest crust foundering (`collapse::founder`) | +1,786 | −174 | −1.2 | +3,994 | +1,748 |
| Carry: the volcano stage's cones and the impacts' craters (split below), advection, sampling | +1,383 | +981 | −0.2 | +2,070 | +1,243 |
| Orogenic collapse | +441 | −164 | +1.3 | +911 | +528 |
| Birth, spreading, hotspots, a resampling's blend | +87 | +171 | −0.5 | +244 | +61 |

In the other bands at 500 Myr the gap is −930 m (30–40 km) and −3,254 m (55–70 km), with the
same parts: accretion −842 and −8,420, the ocean-floor step −1,381 and −3,513, the flexure with
the water's local answer −1,095 and −3,232, erosion net of its rebound +179 and −816, the
transition −315 and −310; arcs +1,650 and +4,443, founder +220 and +5,739. What it says:

- *The gap is made on the continent.* Of the 40–55 km crust's −2,326 m, −1,937 m was made after
  the column became continental and −388 m on the ocean floor (30–40 km: −886 and −44; 55–70 km:
  −2,107 and −1,147). The ocean-floor step damps every oceanic sample towards the floor at
  −5,600 m whatever its thickness (the transition's held floor apart, PLT-264), and so takes
  2.1 km from crust that later turns continental (arcs turn continental at any height,
  `finish_samples`); the arcs' uplift, which raises the surface with no crust (Cortial's slab
  uplift, PLT-048), and the carry give most of it back.
- *The largest part is a resampling's accretion.* Where plates overlap at a resampling, the crust
  under the top cover is stacked onto it (`ridges.rs`, `pools::accrete`) and the column's height
  stays: −3,652 m of the 40–55 km crust's gap made on the continent, −7,518 m of the 55–70 km
  crust's. The collapse documents it ("thickens a column without raising it") and is built around
  it: founder and collapse lower a surface only down to its Airy height, so the crust they remove
  from a column already below it closes its gap (+1,960 and +605 m on the 40–55 km continent).
- *Erosion is not the maker.* Erosion lowered the 40–55 km continent by 14.0 km and its local
  rebound gave 13.3 km back: −701 m on the continent, −401 m all told, 3 % of the depth eroded.
  The rest is the rebound's share: the surface gives back 0.8 of the depth eroded (`isostasy`,
  ISO-001, rock at 2,640 kg/m³), while the column's Airy height falls by 1/7.0 of it, as if 0.858
  came back (the column's mean 2,830 kg/m³, PLT-281); 0.058 of 14.0 km is 810 m. Removing light
  rock from the top leaves a denser column, so this part is the measure's as much as the
  surface's. The rain at 1.42 times raised erosion's part of the band's gap rate from −47.0 to
  −61.4 m/Myr and the rebound with it; the gap did not move (−2,333 to −2,326 m). At 1000 Myr the rain did lower the land (0.189 to 0.174,
  four runs each), through the thickest crust: the share of continental samples at 55–70 km fell
  from 0.33 to 0.27 and at 40–55 km rose from 0.37 to 0.46, and 40–55 km crust stands near sea
  level whatever erodes it.
- *The rates name the same processes.* Over the last 50 Myr the 40–55 km crust's surface fell by
  13.1 m/Myr to erosion and its rebound against its Airy height's 9.8, by 5.8 to the transition
  (heights on a line of 4.24 m of crust per metre, PLT-261/264) against 3.4, and by 8.1 to the
  flexure and the water's local answer with no change of thickness; accretion raised the Airy height by 5.7 m/Myr and the
  surface by nothing. The arcs raised the surface by 16.5 m/Myr against the Airy height's 9.5.
- *Sea level is not it.* It stands at −16 to −25 m at 500 Myr and −58 to −62 m at 1000 Myr.

**The flexure, the sea water and the later stages, split.** Four 500 Myr runs (seeds 1 and 2,
both starts; the same runs as four of the six above, with the same land, the split being only
bookkeeping) with the flexure's answer split into three extra solves and the volcano stage's and
the impacts' changes booked apart:

| Part, stock at 500 Myr, m | 30–40 km | 40–55 km | 55–70 km |
|---|---|---|---|
| Flexure: regional answer to the loads (erosion, deposition, sea water and ice) | −223 | −420 | −670 |
| Flexure: bending of the plate model's roots | +67 | +44 | +99 |
| Flexure: answer to the moment a weakened plate releases | −47 | −104 | −414 |
| Flexure: the rest (its answer to the loads the later stages laid, the solves' unfinished parts) | −543 | −1,216 | −1,816 |
| Sea water and ice, local answer | −372 | −350 | −273 |
| The volcano stage's cones, lava and calderas, the impacts' craters and ejecta | +552 | +1,270 | +1,888 |
| Carry (advection on L1, sampling, clamps) | +156 | +60 | −60 |
| Together | −410 | −716 | −1,246 |

- *The flexure holds little of its own.* The plate model's roots bend nothing (+44 m); the
  regional answer to erosion and the water spreads the thick crust's rebound onto its neighbours
  (−420 m), the released moment takes −104 m. The large rest is the answer to the cones and
  craters: they add 1,270 m of height to the 40–55 km crust over its history and the flexure takes
  1,216 m of it back (+9 m and +72 m net in the other bands). Net of them, the flexure and the
  water hold about −830 m of the 40–55 km crust's gap, a third.
- *The water is the flooding's own answer.* −350 m: a column that has fallen below the sea
  carries the water, which presses it further down, and the Airy height the gap is measured
  against is a dry column's. It follows the gap, it does not make it.
- *The cones are height with no crust under it.* The volcano stage hands its cones, lava and
  calderas to the plate model as elevation only (`surface/feedback_m`); the rock pools change only
  by the surface stage's net erosion and deposition (`surface/material_m`). So a cone's rock never
  joins the column, yet when the surface stage erodes the cone the plate model strips that depth
  from the column (`pools::strip`) and lays it down as sediment downstream. The +1,270 m counts
  every rebuild of an active cone as it wore down, so the crust under the cones lost up to about
  that depth of rock it never gained (inferred: what still stands at the end is not split out),
  moved from arcs and hotspots to basins and the sea. On the gap the two nearly cancel; on the
  crust it is a sink of up to about 1.3 km of the band mean over 500 Myr. Not fixed: where the
  cones' rock belongs is the crust budget's question (the plate model's arc crust is 7.0 times an
  uplift, not erupted magma, PLT-048 and PLT-051), so it stays a recommendation for plate task 8.
- *A fix to the split, not to the model.* Under the sea the roots' crust pushes the water aside,
  and the split first booked the roots' answer to their whole crust (−705 m on the 40–55 km crust)
  and the water's answer to the water pushed aside (+416 m) apart; their sum is the same
  (−289 m before, −306 m after). The water pushed aside now goes with the roots (test
  `under_the_sea_a_root_with_nothing_to_bend_answers_nothing`), so the roots' part is their
  bending only.

**What-if: accreted crust given its height** (an experiment, not in the code). A resampling's
accretion onto a visible continental column raised it by the Airy change of the crust added, up
to the column's Airy height, as the collapse raises its receivers, and handed the rise to the
surface as uplift. Four 500 Myr runs against the same four without it:

| | Without | With |
|---|---|---|
| Gap of 30–40 / 40–55 / 55–70 km crust, m | −975 / −2,382 / −3,196 | −650 / −1,256 / −1,465 |
| Elevation of the 40–55 km crust, m | −73 | +988 |
| Land share | 0.175 | 0.201 |
| Continental crust dry | 0.53 | 0.67 |
| Continental crust, share of the planet; volume, 10⁹ km³ | 0.31; 7.40 | 0.27; 6.15 |
| Eroded over the run, 10⁸ km³ | 1.33 | 1.75 |
| Median land, m (Earth 427) | 760 | 1,244 |
| Land above 2 km, share of the land (Earth 0.10) | 0.20 | 0.34 |

So the accretion's missing height is half the thick crust's gap, but giving it back moves the land
share little: the crust it raises is the thick crust, which becomes high ground and erodes faster,
and the eroded rock leaves the continents (their crust's volume falls by a sixth). Once heights
follow thickness, the erosion law sets the land.

**What is still off.**
- *The land share.* 0.18 at 500 Myr and 0.17 at 1000 Myr with the rain at Beck's 773 mm, inside
  Earth's Phanerozoic range at its bottom and a third below today's 0.29, with the continental
  crust at 0.31 of the planet against about 0.41. Most continental crust stands 0.5–3 km below its
  own Airy height, near sea level (above). The gap budget puts most of it on a resampling's
  accretion (crust with no height), then the ocean-floor step (which the arcs' uplift gives back)
  and the flexure with the sea's weight on flooded columns (about a third of the gap, net of the
  cones it answers); erosion's own share is a few per cent of what it removes. Giving accreted crust its height halves the thick crust's gap but moves the land
  share only from 0.175 to 0.201 and doubles the high ground (the what-if above).
- *The high ground.* Land above 2 km is 0.19 of the land at 500 Myr and 0.27 at 1000 (Earth 0.10),
  and the median land 745 m and 1,004 m (Earth 427 m). As a share of the planet it is about 0.036
  at 500 Myr against Earth's 0.029: the land beneath it is too small. At 1000 Myr it is 0.05, and
  the share of continental samples at 55–70 km has grown from 0.17–0.28 to 0.25–0.38.
- *The Urals.* By the model's isostasy a 50 km column stands about 2.7 km, not 0.5–0.8 km.
  Reaching it from sources needs a lithosphere temperature (old orogens cold and low, young ones
  hot and high; Hyndman & Currie 2011's 1,600 m) or a lower-crust density per column. Neither
  exists in the model.
- *The crust budget.* Creation at cordilleras and destruction at collisions are both unsourced and
  both many times Earth's; island arcs a tenth of Earth's. Putting all three on sourced rates is
  one change (each alone moves the land the wrong way). The volcano stage's cones belong in it:
  their rock never joins the column but is stripped from it when they erode (the split above).

**Limits.**
- One eclogite share for every column, at once: the reaction's dependence on temperature, fluids
  and time is not modelled (PLT-280).
- The mafic pools stand for the lower crust; the model has no lower-crust density of its own.
- The root's pull applies only where a column's thickness changes as a whole (belts, collapse,
  foundering); a column thickened by arc magma or the cordillera's root carries none.

## The crust budget on sourced rates (plate task 8, 2026-10-17)

The coordinator's calls of 2026-10-17 (decisions log, "Plate task 7: the crust budget on sourced
rates" and "The land's gap budget", recommendations 1, 3 and 6). Ledger PLT-044 to PLT-051,
PLT-264, PLT-284 to PLT-286, HYP-017 and HYP-018, PRS-015 to PRS-017, PRS-038 and PRS-039,
HNT-092 and HNT-125 to HNT-127; sources in the bibliography, "The crust budget on sourced rates",
and its Stern & Scholl 2010 entry. The erosion law was not recalibrated: Ben chose one erosion
recalibration after plate tasks 7 and 8, which a later task does on top of this one. So the land
below is this change alone, and it stands too high.

**What happens on Earth.**
- *Island arcs.* Stern & Scholl (2010, read in full) take intra-oceanic arcs at "approximately
  100 km3/km-million years" (500 a Myr for the first 10 Myr, 30 a Myr for 60 Myr after), so
  1.5 km³ a year over today's 14,500 km of them (p. 11 and Table 1). Izu-Bonin-Mariana and the
  Aleutians grew 90 to 180 per km and Myr. Gravity gives the Western Pacific island arcs 28 to 60
  (Parcutela et al. 2023, full text). Clift et al. (2009, full text) average 125 for oceanic arcs
  and 81 for continental ones, their whole magmatic output, recycled crust included.
- *Continental arcs.* Stern & Scholl adopt 35 km³ per km and Myr (Reymer & Schubert's rate, via
  Haschke & Günther 2003, abstract only) over 27,500 km: about 1.0 km³ a year. Francis &
  Hawkesworth (1994, abstract only) give the Central Andes about 13.
- *Intrusive to extrusive.* The continental arcs' rate "implies a reasonable intrusive : extrusive
  ratio of approximately 7:1" (Stern & Scholl, p. 12). White et al. (2006, full text) find 5:1
  "common to most magmatic systems"; the Aleutians 1:1 to 3:1.
- *Shortening.* Bolivia, the most shortened part of the Andes, took "~300 km, 40%" since 50 Ma,
  about 6 mm a year, to crust up to 75 km thick, by pure shear at constant volume (Eichelberger et
  al. 2015, full text). Shortening explains "only 70-80% of the observed crustal thickening" of the
  Altiplano (Allmendinger et al. 1997, abstract); the arcs' magma is the rest.
- *Losses at collisions.* About 0.7 km³ a year in Stern & Scholl's Table 1, of which deeply
  subducted crust is about 0.3.
- *Maturity.* Only arc crust thicker than about 15 km jams a subduction zone (Cloos 1993, abstract).

**What the model did** (plate task 7, measured 2026-10-16). Island arcs took 20 m of magma a Myr at
the arc axis (25 to 90 times below Earth's) and stood up by a height-only uplift (HYP-017); they
matured by time (20 Myr). Continental arcs took arc magma plus a root of 7.0 times a calibrated
uplift (`cordillera_uplift_mm_yr`, PLT-051): 8.5 km³ a year against Earth's 1.0. The volcano
stage's cones were height only, and eroding them stripped rock they never added. A resampling's
accretion added crust and no height. The ocean-floor step damped every oceanic sample to the bare
floor whatever its thickness.

**What the model does now.**
1. *Island arcs* (PLT-045): 100 km³ per km of arc and Myr, times g = min(v / v0, 1) with
   v0 = 100 mm/yr, spread over the arc band (a Gaussian of half-width 60 km whose integral across
   the arc is √π × 60 = 106 km; 940 m a Myr at the axis at g = 1). The arc stands up by the crust
   it adds on the ocean's height line (4.24 m of crust per metre, PLT-264), towards the floor its
   crust holds up. Only an oceanic slab makes an arc.
2. *Continental arcs* (PLT-285): 35 km³ per km and Myr (329 m a Myr at the axis), standing up by
   the crust's Airy change towards the column's Airy height (`collapse::iso_height`).
3. *Into the pools* (PLT-046): continental arcs 1 in 8 felsic volcanic (7:1), island arcs 1 in 6
   volcanic (5:1; two thirds of it mafic), the rest felsic plutonic.
4. *Maturity by thickness* (PLT-047): an island arc turns continental once its arc crust reaches
   Cloos's 15 km less the ocean's 7: 8.5 Myr at the axis at full convergence.
5. *The cordillera by shortening* (PLT-048 to PLT-050): a range within 440 km of the front on a
   continental overriding plate shortens at 6 mm a year (Eichelberger et al.) by pure shear, its
   strain spread as a Gaussian of half-width 120 km, slowing as the column's Airy height nears
   `cordillera_height_m` (3,000 m in the earthlike presets) and stopping there. The volume comes
   from the same plate's continental crust 440 to 800 km behind the front, pool by pool, so it is
   moved, not made. The root from an uplift (PLT-051) and both arc uplifts (HYP-017, PLT-044) are
   gone.
6. *Cones are rock* (PLT-286): the volcano stage's stratovolcanoes, silicic centres and ignimbrites
   add felsic volcanic rock to the pools, shields and flood basalts mafic volcanic, and they count
   inside the arc and hotspot rates (an arc's cones are its extrusive share), so arc magma is
   counted once. Erosion strips the mafic volcanic pool too.
7. *Accretion stands up* (PLT-284): a visible continental column that receives crust at a
   resampling rises by the Airy change of the crust added, towards and never past its Airy height,
   handed to the surface as uplift.
8. *A held floor for thick oceanic crust* (PLT-264): every oceanic sample cools towards the floor
   its own crust holds up on the ocean's line, capped at 20 km of crystalline crust (about
   −2,540 m), so arcs raised by their thickness are not sunk again.

The run summary's plates line gains `cordillera_shortened_km3`, `resample_accreted_visible_km3`,
`resample_accreted_rise_km3`, `hotspot_magma_km3`, `cone_felsic_km3`, `cone_mafic_km3` and the
arc fronts' length × time by side (`arc_front_continental_km_myr`, `arc_front_oceanic_km_myr`);
`tools/review/crust-budget.py` prints them as rates.

**Measured** (`earthlike_quick_16k`, `DEEPTIME_GAP_BUDGET=1`, `tools/review/force-balance-ensemble.sh`;
500 Myr: seeds 1 to 3, both starts; 1000 Myr: seeds 1 to 4, both starts, the plate-check runs;
master 7827e9f0 against the branch; mean [lowest, highest] of the runs; reported, not tuned).
The crust budget in km³ a year over the whole run, Earth-normalized (`crust-budget.py`):

| | Master, 500 Myr | Now, 500 Myr | Master, 1000 Myr | Now, 1000 Myr | Earth today (Stern & Scholl 2010) |
|---|---|---|---|---|---|
| Continental arcs (master: arc magma and the cordillera's root; now: arc magma less the cones counted in it) | 8.77 | 1.63 | 11.6 | 1.91 | about 1.0 |
| Island arcs (the same) | 0.15 | 3.96 | 0.18 | 4.41 | about 1.5 |
| Per km of arc and Myr: continental; island (cones not included) | n/a | 19.5; 31.7 | n/a | 18.7; 31.4 | 35; about 100 |
| The volcano stage's cones into the pools: felsic; mafic | none (height only) | 1.04; 0.71 | none | 1.15; 0.60 | n/a |
| The hotspots' own basalt | n/a | 0.048 | n/a | 0.041 | rifts and hotspots 0.5 |
| Crust shortened into cordilleras (moved within a plate) | none | 3.22 | none | 3.48 | n/a |
| Foundered beyond the thickest column | 7.38 | 4.70 | 12.6 | 7.75 | collision zones lose about 0.7 |
| A resampling's accretion; of it onto visible continental crust | 23.3; 6.6 | 22.8; 8.0 | 32.1; 9.4 | 31.0; 10.7 | n/a |
| Surface the accretion raised over the run, 10⁶ km³ | none | 979 | none | 2,727 | n/a |
| Continental crust at the end, 10⁹ km³ | 7.28 | 7.46 | 8.41 | 8.33 | n/a |

The rates per km are below Earth's because the model's trenches converge slower than v0 on the
whole and the cones an arc builds are counted in its rate but not in these columns. The rates a
year are above Earth's because the model has far more trench: `plate-check` measures 245,000 to
259,000 km of subducting boundary per step (after the first 100 Myr) against Earth's 51,310 km
(Bird 2003), and the arc fronts the run summary counts come to about 84,000 km under continental
crust and 125,000 km under oceanic crust at 500 Myr against Earth's 27,500 and 14,500 (inferred:
the fronts' km × Myr over the run's length). That is the plate model's boundary mix (the subducting
share of the boundary fails its 15 to 25 % target at 50 %), not the arc rates.

The surface (`tools/review/p1-17-hyps.py`; Earth from the relief grid, HYP-001 to HYP-021):

| | Land | Median land height, m | 90th pct, m | Land above 2 km (of the land) | Continental crust (of the planet) | Of it dry | Sea depth, median, m | Surface temperature, °C | CO₂, ppm (geometric mean) |
|---|---|---|---|---|---|---|---|---|---|
| Earth | 0.289 | 427 | 2,022 | 0.102 | about 0.41 | about 0.70 | 4,062 | about 15 | 280 (pre-industrial) |
| Master, 500 Myr | 0.178 [0.164, 0.202] | 736 | 2,744 | 0.179 [0.136, 0.268] | 0.305 | 0.54 | 3,307 | 15.9 | 730 |
| Now, 500 Myr | 0.192 [0.177, 0.224] | 836 | 4,287 | 0.274 [0.173, 0.364] | 0.380 | 0.46 | 3,202 | 21.7 | 6,100 |
| Master, 1000 Myr | 0.180 [0.167, 0.205] | 851 | 3,092 | 0.220 [0.071, 0.317] | 0.324 | 0.50 | 3,122 | 16.6 | 1,110 |
| Now, 1000 Myr | 0.195 [0.175, 0.212] | 1,142 | 5,210 | 0.343 [0.263, 0.395] | 0.391 | 0.45 | 2,903 | 25.8 | 18,900 |

The gap budget at 500 Myr (`tools/review/gap-budget.py --t 0`, six runs each; the gap is the
mean elevation less the column's own Airy height):

| Crust thickness | Share of the continental samples, master; now | Mean elevation, m | The gap, m | Of it made on the ocean floor, m |
|---|---|---|---|---|
| 30–40 km | 0.250; 0.360 | −462; −341 | −930; −795 | −44; −556 |
| 40–55 km | 0.336; 0.255 | −21; +528 | −2,326; −1,623 | −388; −1,370 |
| 55–70 km | 0.235; 0.101 | 1,072; 2,895 | −3,254; −1,379 | −1,147; −2,268 |

The median elevation of the 40–55 km crust (`crust_bins`) rose from 87 to 396 m at 500 Myr and from
−105 to +97 m at 1000 Myr. Its parts at 500 Myr, master against now: a resampling's accretion −4,046
against −902 m; the ocean-floor step −2,142 against −2,097; the arcs +3,071 against +1,612;
founder +1,786 against +904; erosion net of its rebound −401 against −344; the cones and craters
+1,324 against +1,458; collapse +441 against +229.

**What it means.**
- *The rates are on sources, and the budget is no longer two churns.* Continental arcs went from
  nearly nine times Earth's rate to 1.6 to 1.9 times it, island arcs from a tenth to 2.6 to 2.9
  times, both from the length of the model's trenches, not from the rate per km. Foundering fell by
  more than a third (less is stacked beyond 70 km now that the cordillera's crust is moved, not
  made; inferred) and is still 7 to 11 times Earth's collision losses. The crust's volume is about
  the same; it is spread wider (0.38 to 0.39 of the planet against 0.31 to 0.32) and thinner (the
  55–70 km share fell from 0.24 to 0.10).
- *Accretion's height closed most of its gap.* Its part of the 40–55 km crust's gap fell from
  −4,046 to −902 m and the gap from −2,326 to −1,623 m.
- *What is left is made on the ocean floor.* −1,370 of the 40–55 km crust's −1,623 m. The
  ocean-floor step still takes its 2.1 km (ocean floor stands below the dry Airy height of its own
  thin column: the water's load and the cooled lithosphere), and the arcs no longer give it back
  with a height-only uplift (+224 m on the ocean floor, against +1,471). So an island arc turns
  continental at its height on the ocean's line, which lies below the continent's Airy line for any
  crust thinner than the normal 33 km (1.4 km below at the 15 km of maturity; this page's
  arithmetic from PLT-264 and `iso_height`). Under the sea the dry Airy height overstates where a
  column would stand: with the water's load a 15 km column of continental density stands about
  −3.4 km (Airy under water, 470 / 2,270 m of height per m of crust below sea level), the ocean's
  line −3.7 km (this page's arithmetic). So much of this part is the measure's.
- *The land rose a little and the high ground a lot.* Land 0.19 to 0.20 of the planet against
  0.18; land above 2 km 0.27 to 0.34 of the land against 0.18 to 0.22 (Earth 0.10), the 90th
  percentile 4.3 to 5.2 km (Earth 2.0). The thick crust now stands near its height, and erosion,
  not recalibrated, does not wear it down: that is the next task's.
  Taken up on 2026-10-18 (erosion-and-sediment.md, "The erosion law against measured rates"): with
  the erosion at Montgomery & Brandon's rates land above 2 km is 0.25 to 0.28 of the land and the
  plate checks meet 25 of 45.

**What is still off.**
- *The high ground,* above: the erosion recalibration's job (Ben's choice), not tuned here.
- *The climate follows the high ground.* The branch's runs are warmer (21.7 °C at 500 Myr, 25.8 at
  1000; CO₂ 6,100 and 18,900 ppm, geometric means). Over all 28 runs, master and branch, CO₂ and
  temperature rise with the share of land above 2 km (rank correlations 0.93 and 0.91; 0.91 and
  0.87 over the branch's 14 alone). Degassing reads ridge production, the ocean floor consumed and
  the large igneous provinces (`carbon::degassing`, `new_provinces`), none of which this task
  raised by its own code, and the plates are slower; so the warmth is weathering's (inferred), and
  which part of it the high ground cuts was not traced.
- *Trenches too long.* The arcs' rates a year are above Earth's because the subducting boundary is
  about five times Earth's (above).
- *Foundering,* 4.7 to 7.8 km³ a year against Earth's collision losses of about 0.7.
- *Cones away from arcs and hotspots* (rift and intraplate volcanoes) add rock beyond the sourced
  rates; how much of the cones' 1.75 km³ a year that is was not split out.
- *Continental arcs stand lower.* The median continental arc cell stands 1,095 m against master's
  2,055 m (`plate-check`, target 1,200 to 4,500 m) and the ground under the volcano stage's arc
  cones at −783 m at 500 Myr against +782 m: arcs no longer have a height-only uplift, and their
  crust stands at its Airy height less what erosion takes.
- *The structure stage's arc strain* (DEF-026) is scaled by the plate uplift over 0.1 mm/yr, and
  the arcs' uplift changed with this task. Re-checked by the erosion recalibration (2026-10-18,
  natural climate, 500 Myr, seeds 1 and 2, both starts): the arcs' mean lift factor fell from 0.365
  to 0.257 and their strain from 0.145 to 0.105, but the Andean margin and its foreland together
  made 5 % fewer structures (1,441 → 1,370 a run); the scale stays at 0.1 mm/yr (DEF-026).

**The plate checks** (`plate-check`, 1000 Myr, seeds 1 to 4, both starts): 26 of 45 targets met
against master's 29 (master 7827e9f0; 31 on 2026-10-16, the difference being collision length
and the speed ratio by crust, both near their limits). Moved to pass: collision length, 90th
percentile (128 → 107 Myr, at most 120); speed of oceanic ÷ continental plates (1.95 → 2.52, at
least 2); oceanic arc crest (1,077 → 332 m, at most 1,000). Moved to fail: rift-born plates welded
back within 50 Myr (8.8 → 12.0 %, at most 10); continental crust, highest step (0.44 → 0.47, at most
0.45); plate speed RMS (41.6 → 36.5 mm/yr, 40 to 100); steps on which the speed guard binds
(0 → 0.12, one step of one run); ocean floor older than 180 Myr (9.6 → 11.5 %, at most 10);
continental arc median height (2,055 → 1,095 m, 1,200 to 4,500). Still failing and moved: collisions
per 100 Myr 1.05 → 1.61 (2 to 6); continental crust, lowest step 0.26 → 0.34 (at least 0.37); slab ÷
free plates 1.84 → 2.17 (at least 2.5); the wedge 4 → 59 km (100 to 350); ocean floor older than
250 Myr 3.5 → 4.6 % (at most 3); supercontinent tenure 46 → 21 Myr and period 285 → 82 Myr (100 to
300, 400 to 800; assemblies 3.3 → 7.1 a run: with more continental crust, 70 % of it lies in one
piece more often and for shorter stays, inferred). Unchanged in effect: active margin share 0.48,
speed against slab share 0.46 → 0.49, rotation change 90th percentile 25 → 26 %, subducting share
of the boundary 53 → 50 %, the foredeep. Slower plates go with more continental crust: mostly
continental plates move at 23.6 mm/yr against 27.9, mostly oceanic ones at 44.7 against 47.7.

**Limits.**
- One long-term rate for every arc, young or old: a young arc builds slower and an old one faster
  than Earth's front-loaded arcs (PLT-045).
- The Andes' largest shortening rate at every continental arc above an oceanic slab, from one
  source zone behind it (PLT-048).
- The 2:1 mafic to felsic split of island-arc lavas and the felsic plutonic pool for every arc
  intrusive are the model's (PLT-046).
- The held floor's 20 km cap and the accretion's cap at the column's Airy height are rules
  (PLT-264, PLT-284).
- The gap is measured against a dry Airy height, which overstates where a flooded column stands.

## The fastest a column thickens and rises (the plate spikes, 2026-10-18)

The coordinator's task after the surface divergence (ISO-072): find which plate process lifts a
column by up to 33 km in one step against the 30 km ceiling (ISO-025), decide whether it is
physical, and bound it by a sourced rate. Ledger PLT-287 and PLT-288 (and the changes to PLT-258,
PLT-277, PLT-283, PLT-284 and ISO-025; the guard's near counts ISO-073); sources in the
bibliography, "The fastest a column thickens and rises", and its Sundell et al. 2021 entry. Code:
`crates/deeptime-core/src/plates/rate.rs`.

**The processes.** The gap budget's spike record (PLT-283: every process of a plate step
bracketed, the largest rise and thickening one visible column took from it) on two 500 Myr runs of
`earthlike_quick_16k` on master (scattered start seed 2, supercontinent seed 1):

| process | largest rise of a column in one 2 Myr step | largest thickening in one step | column-steps rising more than 10 km |
|---|---|---|---|
| a resampling's accretion (PLT-284) | 35.8 km, 33.2 km | 526 km, 566 km | 2,697, 1,272 |
| the collision belts and the wedge | 13.1 km, 9.9 km | 94 km, 77 km | 5, 0 |
| the founder (its receivers) | 5.8 km, 6.0 km | 42 km, 42 km | 0 |
| the collapse (its receivers) | 4.3 km, 4.3 km | 30 km, 30 km | 0 |
| hotspots, arcs, transition, the rest | 1.2 km or less | 3.7 km or less | 0 |

Every step whose largest rise passed 20 km (21 and 16 of 250) was a resampling, and so was the run's
final peak, 29,181 and 26,984 m: the run ends with a forced resampling, and two steps before it the
highest cell stood at 5.6 km. The surface's flexure turned these single-step lifts into the 50 to
100 km the guard met (ISO-072).

**What happens on Earth.** Crust thickens where it shortens, at dH/dt = u·H/W across a region of
width W shortening at u (Garzione et al. 2006, eq. 2, full text): for the Altiplano, 10 mm/yr over
300 km of 50 km crust, about 1.7 mm/yr, and the surface rises by about a seventh of it, 0.3 mm/yr.
Southern Tibet thickened by about 40 km at about 1.3 mm/yr between 60 and 30 Ma (Sundell et al.
2021, full text). The fastest surface uplift Garzione et al. found, 2.5 to 3.5 km in 1 to 4 Myr
(0.6 to 3.5 mm/yr), needed "the removal of dense eclogitic lower crust and mantle lithosphere".

**Physical?** None of the large spikes is. The accretion's 36 km in 2 Myr is 18 mm/yr of uplift,
five times the fastest Altiplano rate, and its 566 km of crust in a step is seven times the
thickest crust on Earth (about 80 km, PLT-258): it hands one column the crust of every footprint
under it at once. The wedge's 94 km in a step is 47 mm/yr of thickening, about 35 times Tibet's; the
belt's 13 km is 6.5 mm/yr of uplift. The founder's and the collapse's rises (2.9 and 2.2 mm/yr)
are inside Garzione's range, but their receivers thickened by up to 21 mm/yr.

**What the model does now** (rate.rs).
1. When a plate step begins every sample keeps its thickness T₀ and surface z₀. Its ceiling is
   T₀ + 5 mm/yr × δt (`thickening_max_mm_yr`, PLT-287: about three times the regional rates, for
   one lattice column over one step), its top z₀ + 3.5 mm/yr × δt (`uplift_max_mm_yr`, PLT-288,
   Garzione's fastest rate).
2. After the step's thickening processes (belts, arcs, founder, collapse, transition, collisions,
   hotspots) and again after a resampling's accretion (the limits carried across with the
   column's own resampling weights), a visible column above its ceiling moves the excess to the
   nearest room on its plate within `collapse_reach_km`, as the collapse moves crust (continental
   crust up to `crust_max_km`, ocean floor up to the normal column, each under its own ceiling);
   what finds no room goes down from the bottom of the column, as beyond `crust_max_km`.
3. A column that releases crust sinks by its Airy change, never below where it stood when the
   step began; no visible column ends higher than its top. A column lifted faster is left below
   its Airy height, and later steps let it rise the rest.
4. The collapse's and the founder's receivers fill only up to their ceilings.

**Measured** (`earthlike_quick_16k`, seeds 1 to 3, both starts; master 10c92fe4 with this
branch's reports only (the spike record and the guard's near counts, which change nothing) against
the fix; mean over six runs, [min, max]; build log 2026-10-18 "Plate spikes"):

| 500 Myr | master | the rate limits |
|---|---|---|
| highest cell at the end, L1 (m) | 23,544 [14,259, 29,181] | 7,288 [5,860, 9,958] |
| after the young stage (m) | 8,203 [7,057, 9,893] | 6,071 [5,613, 6,338] |
| after the amplifier, highest block (y) | 313 [279, 360] | 246 [234, 253] |
| largest rise of a column in one plate step (two traced runs, m) | 35,842, 32,126 | 7,000, 7,000 |
| land share | 0.196 [0.190, 0.202] | 0.199 [0.160, 0.222] |
| land above 2 km | 0.227 [0.129, 0.320] | 0.200 [0.150, 0.271] |
| continental crust, share of the planet at the end | 0.406 [0.352, 0.458] | 0.393 [0.366, 0.415] |
| CO2 (ppm) | 3,774 [443, 13,543] | 907 [403, 1,518] |
| present-day nudge's CO2 factor | 5.31 [1.00, 19.59] | 1.24 [1.00, 2.20] |
| mean surface temperature (°C) | 18.2 [14.4, 25.0] | 15.7 [13.3, 17.3] |
| the guard's worst (m; the bound is 237,286) | 66,669 [43,693, 101,282] | 30,815 [19,599, 59,631] |
| steps beyond 1, 2, 3, 6 Earth reliefs (sum of six runs) | 193, 27, 5, 0 | 25, 8, 1, 0 |
| foundered (km³ a year, Earth-normalized) | 4.1 | 1.8 |
| beyond the ceilings, moved or lost (km³ a year) | - | 11.6 |
| of which to the mantle (km³ a year) | - | 4.3 |
| plate checks met (of 45) | 27 | 29 |

| 1000 Myr | master | the rate limits |
|---|---|---|
| highest cell at the end, L1 (m) | 25,248 [21,041, 29,518] | 8,089 [5,917, 13,752] |
| after the young stage (m) | 8,115 [7,247, 8,848] | 6,196 [5,919, 6,617] |
| after the amplifier, highest block (y) | 310 [290, 336] | 249 [241, 261] |
| land share | 0.200 [0.174, 0.225] | 0.180 [0.164, 0.200] |
| land above 2 km | 0.223 [0.143, 0.265] | 0.231 [0.106, 0.318] |
| median and 90th percentile land height (m) | 774, 3,277 | 843, 3,323 |
| continental crust, share of the planet at the end | 0.433 [0.392, 0.467] | 0.405 [0.361, 0.485] |
| CO2 (ppm) | 2,046 [405, 6,331] | 2,192 [179, 4,469] |
| present-day nudge's CO2 factor | 2.88 [1.00, 8.17] | 3.31 [1.00, 6.86] |
| mean surface temperature (°C) | 16.7 [13.2, 21.9] | 17.3 [12.4, 20.2] |
| the guard's worst (m) | 82,335 [70,106, 98,837] | 32,119 [19,636, 49,599] |
| steps beyond 1, 2, 3, 6 Earth reliefs (sum of six runs) | 427, 72, 18, 0 | 60, 2, 0, 0 |
| foundered (km³ a year) | 6.6 | 2.3 |
| beyond the ceilings, moved or lost (km³ a year) | - | 16.4 |
| of which to the mantle (km³ a year) | - | 6.3 |
| plate checks met (of 45) | 26 | 25 |

The 13,752 m at 1000 Myr is the scattered start's seed 2 (not traced; the other five fixed runs end
between 5.9 and 7.4 km). The plate checks
change little. At 500 Myr the fix passes four that master fails (rift-born plates welded back
within 50 Myr, 10.8 to 6.6 %; the collisions' median length, 57 to 45 Myr; speed against plate area;
oceanic arc cells above sea level) and fails two that master passes (the collisions' 90th
percentile length, 91 to 132 Myr; supercontinent tenure, 103 to 98 Myr). At 1000 Myr the only
change of verdict is the most land gained in one step, 1.12 to 1.64 % of the planet against at
most 1.5 (not traced; likely the crust a resampling used to pile on one column, now spread over
its ring, lifting more cells above the sea at once). The collision wedge's width is 0 km in every
fixed run (master: 13 and 54 km, both far below the 100 to 350 km target), probably because the
wedge's thickening, up to 94 km a step on master, is now held to 10 km.

**Limits.**
- One rate for every column, not a strain-rate law: the excess is moved, not deformed, whatever
  the column's thickness or heat (PLT-287).
- Where no room is within reach (about one ring of samples at 16k), the excess goes to the mantle:
  4.3 km³ a year over 500 Myr and 6.3 over 1000, against the founder's loss falling by 2.3 and
  4.4. The crust lost to the mantle in all rose from 4.1 to 6.1 and from 6.6 to 8.5 km³ a year,
  against about 0.7 km³ a year at Earth's collision zones (PLT-258, Scholl & von Huene 2009 as
  cited there), and the planet ends with 0.013 and 0.028 less of its area continental. A wider
  reach for the release, or an accretion that does not stack every footprint on one column, would
  keep more of it (a follow-up, taken up in "Resampling lumps" below: the lumps were crust counted
  twice, not many footprints on one column).
- Falls are not limited: crust that founders or spreads away lowers its surface at once (up to
  about 6.6 km in a step, 3.3 mm/yr, under the uplift cap of PLT-288; no source on the fastest
  subsidence was read).
- The surface stage's own excursions are not touched: a polar row of L1 rose 20 to 40 km over a
  few steps with plate uplift of 2.2 km or less, on master and with the fix. The fixed run with the
  largest worst (supercontinent seed 2, 500 Myr, 59,631 m) met it as a hole: two neighbouring cells
  of a row near the pole (rows 8 to 15 of L1) went from 17.1 km below the sea at −290 Myr, beside a
  cell at +17.3 km, to 59.7 km below at −274 and back to 18.7 km below by −264, moving along the
  row, while the plates raised them by 0.3 to 2.0 km a step and no plate step lifted any cell more
  than 5.8 km. It was traced to the surface stage's flexure, outside the plate model: where a
  plate boundary weakened a cell, the plate let go of its bending moment, and a moment that steps
  from cell to cell released several times the load the plate held. Since 2026-10-18 a weakening
  plate lets go of its departure from local isostasy instead (ISO-056 rewritten, ISO-074 to
  ISO-077; [flexure.md](flexure.md), "The polar holes"; build log 2026-10-18, the polar holes).

## Resampling lumps: crust a collision took is counted once (2026-10-18)

The coordinator's follow-up to the plate spikes: a resampling's accretion stacked crust on one
column, and the rate limit (PLT-287) then sent the excess to the mantle, 4.3 km³ a year at 500 Myr
on top of the founder's 1.8 (4.1 and 1.4 since the polar holes' fix). Fix the cause so the limit
rarely binds and the loss falls towards Earth's rate, and keep the limit as a guard. Ledger PLT-289
to PLT-292 (and the changes to PLT-047, PLT-243, PLT-258, PLT-276, PLT-284 and PLT-287); the source
in the bibliography, "Resampling lumps". Code: `plates/{subduction.rs, belts.rs, collision.rs,
ridges.rs, state.rs, stats.rs}`.

**Where the lumps came from.** Not from many footprints on one column. A temporary trace (not
committed) sorted every resampling's accretion by what lay under the column and by whether the
column took more than 10 km at once, on the plate spikes' two traced runs (500 Myr, scattered
start seed 2 and supercontinent seed 1, master 9091c163):

| accreted over the run, km³ (of it in lumps of more than 10 km on one column) | scattered 2 | supercontinent 1 |
|---|---|---|
| crust of hidden ocean floor | 8.56e9 (3.40e9) | 5.71e9 (1.98e9) |
| visible continental crust of a plate going under | 2.06e9 (1.49e9) | 1.75e9 (1.26e9) |
| visible ocean floor of a plate going under | 1.84e9 (0.81e9) | 1.19e9 (0.44e9) |
| hidden continental crust (not a collision's) | 0.58e9 (0.43e9) | 0.43e9 (0.30e9) |
| the column's own plate | 0.64e9 (0.44e9) | 0.44e9 (0.27e9) |
| visible crust of another plate marked `stacked` | 0.10e9 (0.07e9) | 0.003e9 (0.002e9) |
| all | 13.78e9 (6.66e9) | 9.52e9 (4.25e9) |

A single covered sample carried the lump: at −372 Myr on the scattered run, one hidden sample
labelled ocean floor held 128 km of rock (32 km metamorphic, 85 km felsic plutonic) and raised the
continental column above it from 600 m to 17.9 km. Four ways put a continent's worth of crust where
the accretion counted it again:

1. *The belt stacked on ocean floor* (PLT-290). The belt's receivers were every sample of the
   overriding plate behind the front. Ocean floor among them (beside a narrow continent, or an
   island arc) stayed ocean floor under any thickness; the dampening pulled its surface back to the
   ocean floor, so it kept its room and took more every step, up to 60 km of metamorphic rock, then
   subducted as ocean floor and came back at a resampling.
2. *The colliding plate's crust still visible* (PLT-292). Continental crust of the plate going
   under, inside the overriding plate's footprint, had crossed the front, and the belt had counted
   it when it did; the resampling added it to the column on top as well.
3. *Crossed crust brought back up* (PLT-291). `stacked` (PLT-276) was cleared every step, and a
   terrane transfer or a weld unhid the terrane's crust and moved it, so crust that had gone under
   came back as the receiving plate's crust beneath its own, and the next resampling accreted it.
4. *Ocean floor as thick as a continent* (PLT-289). Crust stacked under the oceanic label by
   earlier accretions subducted as ocean floor at a contact and came back in one lump.

**The fix.** Each volume is counted once, and crust stays what it is:
1. The belt stacks only on continental crust (PLT-290); behind a narrow continent the area floor of
   PLT-243 sends more of the stack down with the slab.
2. At a resampling, visible continental crust of a plate colliding under the column's plate is
   consumed, as the hidden crust of PLT-276 is (PLT-292).
3. `stacked` stays while the crust stays hidden; a transfer moves, and a weld unhides, only the
   terrane's crust that has not gone under (PLT-291).
4. Ocean floor at a contact whose crystalline crust is at least a normal continental column's (T_n,
   33 km) turns continental (PLT-289), and then collides or moves as a terrane. Cloos (1993) puts
   the buoyancy that jams a subduction zone at more than about 15 km of crust; the threshold is T_n
   so that the transition's ocean side (up to about 20 km of thinned margin, ocean floor by design,
   PLT-261) keeps its type. At 15 km two traced runs turned about ten times as many samples, the
   taper formed again behind each converted ring, and continental crust ended at 0.478 and 0.464
   of the planet (master 0.394 and 0.415).

**Ablation** (the two traced runs, 500 Myr, before the polar holes' fix was merged; "all four" is what landed):

| | master | all four | without 4 | 4 at 15 km, without 1 |
|---|---|---|---|---|
| accreted in lumps over 10 km (10⁹ km³) | 6.66, 4.25 | 2.82, 1.55 | 3.54, 1.71 | 1.83, 1.30 |
| sent to the mantle by the rate limit (km³ a year) | 5.63, 3.23 | 0.80, 0.34 | 1.44, 0.61 | 0.68, 0.30 |
| founder plus rate limit (km³ a year) | 7.89, 4.92 | 3.15, 1.54 | 4.56, 1.70 | 3.71, 1.48 |
| continental crust at the end | 0.394, 0.415 | 0.413, 0.457 | 0.347, 0.433 | 0.438, 0.391 |
| highest cell at the end (m) | 6,540, 9,958 | 6,595, 6,032 | 11,797, 6,987 | 5,865, 7,146 |
| guard's worst (m) | 40,648, 20,326 | 22,039, 27,305 | 25,951, 29,099 | 122,559, 30,229 |

Without the fourth rule the scattered run ended with an 11.8 km peak and less continental crust;
without the first, one run met the guard at 123 km (not traced).

**Measured** (`earthlike_quick_16k`, seeds 1 to 3, both starts, master against the fix, both with
the polar holes' fix of 991354a4; mean [min, max] over six runs; `tools/review/force-balance-ensemble.sh`
in `ck` mode, then `tools/review/p1-17-hyps.py`). The loss to the mantle is the founder's and the
rate limit's (`foundered_km3` and `rate_lost_km3` over the run's length); "with the slab" adds the
belts' crust that went down with the slab (`collision_crossed_km3` less `collision_kept_km3`).

| 500 Myr | Master | Fix |
|---|---|---|
| Highest cell at the end, L1 | 7,031 m [6,555, 7,648] | 6,689 m [5,655, 7,499] |
| Land | 0.207 [0.155, 0.237] | 0.191 [0.152, 0.219] |
| Land above 2 km (share of the land) | 0.207 [0.161, 0.255] | 0.206 [0.154, 0.239] |
| Continental crust at the end | 0.405 [0.360, 0.454] | 0.396 [0.368, 0.418] |
| Loss to the mantle, founder and rate limit (km³/yr) | 5.49 [2.44, 7.59] | 2.13 [1.79, 2.80] |
| of it the founder | 1.41 [0.71, 1.99] | 1.60 [1.15, 2.23] |
| of it the rate limit | 4.07 [1.72, 5.60] | 0.53 [0.37, 0.64] |
| With the slab's share (km³/yr) | 6.48 [2.85, 8.87] | 3.81 [2.95, 5.09] |
| Past the step ceilings (km³/yr) | 11.9 [7.7, 15.2] | 6.0 [5.0, 7.3] |
| A resampling accreted; consumed (km³/yr) | 22.1; 11.5 | 13.8; 15.6 |
| Crust that crossed a collision front (km³/yr) | 15.5 [8.2, 19.8] | 16.6 [12.8, 22.2] |
| Ocean floor turned continental (samples a run) | - | 4,511 [3,483, 5,466] |
| Guard's worst (bound 237,286 m) | 15,471 m [13,187, 18,101] | 15,230 m [13,602, 16,808] |
| Steps beyond 1, 2, 3, 6 reliefs (six runs) | 0, 0, 0, 0 | 0, 0, 0, 0 |
| CO₂ | 1,058 ppm [532, 1,846] | 1,043 ppm [477, 2,862] |
| Present-day nudge's CO₂ factor | 2.09 [1.00, 4.83] | 1.72 [1.00, 3.71] |
| Mean surface temperature | 15.9 °C [13.9, 17.9] | 15.7 °C [13.2, 19.3] |
| Collision wedge, median width per run | 0 km in all six | 0 km in five, 94 km in one |
| Plate checks met (of 45) | 30 | 29 |

| 1000 Myr | Master | Fix |
|---|---|---|
| Highest cell at the end, L1 | 7,177 m [6,739, 7,551] | 6,463 m [4,878, 7,682] |
| Land | 0.179 [0.164, 0.203] | 0.168 [0.118, 0.211] |
| Land above 2 km (share of the land) | 0.277 [0.146, 0.384] | 0.215 [0.050, 0.347] |
| Continental crust at the end | 0.420 [0.338, 0.525] | 0.431 [0.346, 0.526] |
| Loss to the mantle, founder and rate limit (km³/yr) | 8.26 [5.39, 10.76] | 2.96 [1.70, 4.14] |
| of it the founder | 2.21 [1.59, 2.73] | 2.21 [1.22, 3.06] |
| of it the rate limit | 6.05 [3.79, 8.17] | 0.75 [0.48, 1.08] |
| With the slab's share (km³/yr) | 9.56 [6.53, 12.11] | 5.03 [2.87, 7.15] |
| Past the step ceilings (km³/yr) | 16.5 [14.3, 18.4] | 9.2 [7.4, 11.0] |
| A resampling accreted; consumed (km³/yr) | 28.8; 13.8 | 19.0; 21.2 |
| Crust that crossed a collision front (km³/yr) | 18.9 [15.5, 21.6] | 21.9 [13.4, 29.7] |
| Ocean floor turned continental (samples a run) | - | 17,092 [12,675, 21,837] |
| Guard's worst (bound 237,286 m) | 16,680 m [15,028, 17,737] | 16,098 m [14,779, 17,379] |
| Steps beyond 1, 2, 3, 6 reliefs (six runs) | 0, 0, 0, 0 | 0, 0, 0, 0 |
| CO₂ | 8,097 ppm [519, 22,870] | 1,787 ppm [425, 4,668] |
| Present-day nudge's CO₂ factor | 10.06 [1.09, 39.45] | 2.22 [1.00, 4.05] |
| Mean surface temperature | 20.2 °C [14.4, 28.6] | 17.2 °C [15.2, 21.4] |
| Collision wedge, median width per run | 0 km in all six | 0 km in all six |
| Plate checks met (of 45) | 25 | 27 |

The loss to the mantle falls from 5.5 to 2.1 km³ a year at 500 Myr and from 8.3 to 3.0 at
1000 Myr, the part the rate limit sends down by a factor of about eight (4.1 to 0.5, 6.1 to 0.8);
the six runs' ranges no longer overlap. The founder's part is unchanged within the spread. Crust
still passes the step ceilings, about half of master's volume (6.0 and 9.2 km³ a year against 11.9
and 16.5), but most of it now finds room within reach. With the slab's share the fix loses 3.8 and
5.0 km³ a year (master 6.5 and 9.6). Earth's figure for collision zones is 0.7 km³ a year at an
assumed past length of collisions, about 1.4 at today's, and 3.2 at all subduction zones together,
most of it subduction erosion at ocean margins, which the model does not have (Scholl and von Huene
2009, below). The fix's loss through the founder and the rate limit is three to four times the
0.7 (one and a half to two times the 1.4); with the slab's share it is about what Earth loses at
all its margins.

Each crossed volume is now counted once: the crust a resampling consumes as a collision's (PLT-276
and PLT-292) is 15.6 and 21.2 km³ a year against 16.6 and 21.9 that crossed a front, where master
consumed 11.5 and 13.8 of 15.5 and 18.9 and accreted the rest again. The continental crust at the end
is the same within the spread (0.396 against 0.405, 0.431 against 0.420), and so are the guard,
the land and the highest cell. One fixed run, supercontinent seed 1 at 1000 Myr, ended with the
most continental crust (0.526, as master's scattered seed 1 did) and the least land (0.118, 5 % of
it above 2 km); it was not traced.

**Plate checks.** 30 met on master and 29 with the fix at 500 Myr, 25 and 27 at 1000 Myr. At
500 Myr the continental crust's lowest step passes (0.37 to 0.38), and two fail: the collision's
median length (38.7 to 54.5 Myr; its runs span 27 to 85 Myr, and at 1000 Myr it passes, 36.2 to
40.7) and the oceanic arc cells above sea (10.2 to 8.6 %; 12.9 to 10.6 %, a pass, at 1000 Myr).
At 1000 Myr the most land gained in one step passes (1.53 to 1.35 %) and so do the collisions per
100 Myr (1.73 to 2.17): with the ocean-floor rule, thick island arcs and stacked ocean floor
collide instead of going under, and more plates die by welding (5.3 to 7.0 a run at 500 Myr, 17.3
to 21.7 at 1000). The continental arcs' median stands lower (941 to 709 m, 1,014 to 817 m; a fail
in both); not traced, though the ocean-floor rule moving the thickest island arcs into the
continental count would lower both arc figures.

**The collision wedge's width.** Its 0 km comes with the rate limits, not with this change: master
with the limits gives 0 km in every run at 500 and 1000 Myr, and master before them gave 13 and
54 km ("The fastest a column thickens and rises", above). The fix gives 0 km in eleven of its twelve runs and 94 km in one
(supercontinent seed 1 at 500 Myr), against 100 to 350 km on Earth (PLT-244). The width
is the farthest incoming continental sample still below the 1.5° profile from the front
(`belts.rs` `stack_front`), so it comes in steps of one sample, about 70 km on the model sphere,
and a wedge one sample wide needs the front to stand about 1.9 km above the incoming crust (PLT-192:
191 km for 5 km). With the uplift held to 3.5 mm a year the front seldom gets there in the steps
that are measured. The rule is the wedge's and is left for a follow-up.

**The guard's outlier, before the polar holes' fix.** Measured on this branch before 991354a4 was
merged, one fixed run, supercontinent seed 3 at 500 Myr, met the guard at 129,576 m below the sea
and spent 67 steps beyond one Earth relief. A traced rerun (a temporary print of the farthest cell,
its neighbours and the plates' uplift, not committed) put every one of those steps on rows 323 to
326 of the 328-row surface grid, the last rows before the pole: a hole of two neighbouring cells
deepened to 129.6 km and moved along the row while the plates' uplift at the cell stayed between
−5.9 and +3.5 mm a year. It was the surface stage's polar-row excursion, which the polar holes'
fix (flexure.md, "The polar holes") has since removed: after the merge the same run's worst is
13,602 m, and no run on either side passes one relief.

**The source behind PLT-258.** The row compared the loss with "about 0.7 km³ a year at
collision zones (Scholl and von Huene 2009, as cited)" by Stern and Scholl (2010). The paper is
paywalled; its abstract, read through Crossref, gives at ocean-margin subduction zones c. 60 km³
per Myr per km of margin, c. 2.5 km³ a year in all, most of it subduction erosion, and at
crust-suturing (collision) zones c. 115 km³ per Myr per km, "assessed far less securely", times a
global length of 6,000 km, which "presumes that in the past the global length of crustal
collision zones averaged c. 6000 km, which is one-half that under way since the early Tertiary":
c. 0.7 km³ a year. The total loss is c. 3.2 km³ a year against 2.7 added. So 0.7 is the paper's
figure for collision zones and was cited correctly; at today's length it would be about 1.4. The
row now quotes the abstract; its status stays `cited-unverified`, the paper `abstract-only`.

**What is left.**
- Lumps remain, at 42 and 36 % of master's in the two traced runs (2.82 and 1.55 × 10⁹ km³ in
  lumps over 10 km, traced before the merge). Most are the felsic crust of hidden ocean floor (1.62
  and 0.81): sediment, arcs and the transition's thinned margins under 33 km of crust, which the
  accretion keeps (it conserves the first five pools) and puts on the column above in one go.
  Spreading it over the column's ring, or sending part of it down with the slab, is a follow-up.
- The loss to the mantle is still above Earth's collision-zone figure (above).
- The wedge's width (above).
- The ocean-floor rule turns more samples as the run goes on (4,511 a run by 500 Myr, 17,092 by
  1000 Myr); what they are and whether they should all turn is not traced.
- The CO₂ of master's 1000 Myr runs reached 22,870 and 18,612 ppm in two runs (28.6 and 25.8 °C);
  the fix's highest was 4,668 ppm (21.4 °C). Why was not traced.

## The coupled group's cadence (P1-19 #27, 2026-10-18)

The coordinator's task for the 900 s gate (P1-19 exit item 1, profile item #27): the full 16k gate
run (`earthlike_full_16k`, scattered start, 1000 Myr, 8 threads) took 1,242.8 s, and the plates,
climate, surface, volcanoes and structure, which run together once per plate step, take most of it.
Find the cadence that saves the most time for the least change to the planet. Code: `params.rs`,
`plates/{rifting,pools,ridges,shortening,subduction,check}.rs`, `structure/deform/feature.rs`,
`surface/{params,mod}.rs`, `climate/params.rs`. Ledger: PLT-053, PLT-058, PLT-059, PLT-063,
PLT-082, PLT-083, PLT-098, PLT-225, CRE-025, DEF-035, ERO-065, the per-step check targets CHK-008,
CHK-009, CHK-030, CHK-031, PLT-150 and PLT-151, and SCL-038 (the climate's snapshot interval; its
section is climate.md, "The climate snapshot every 20 Myr").

**How the group is clocked.** The plate step is the length divided by round(length ÷
`time_step_myr`) (SCL-034); the climate, the surface, the volcanoes, the impacts and the structure
run once per plate step; the surface takes `substeps` landscape steps inside it (ERO-018); the
climate takes a full snapshot every round(`snapshot_every_myr` ÷ step) steps (SCL-038); and with a
surface or climate stage L1 is written every step (`l1_every_steps` must be 1). Writing L1 less
often would need the session to run the surface and the climate on a coarser clock than the
plates, a new coupling that was not tried.

**Rules that counted steps now count time.** At the 2-Myr step every one gives the number it gave
before: a 40 Myr `earthlike_quick_16k` run (seed 1, 4 threads) prints the same summary as master's
in every line but the timings.

| rule | before, per step | now | at a 4-Myr step |
|---|---|---|---|
| rifting's λ₀ (PLT-082, PLT-083) | `rift_lambda0` 0.012 | `rift_per_myr` 0.006 × the step | 0.024 |
| spontaneous subduction initiation (PLT-063) | `subduction_initiation_rate` 0.3 | `subduction_initiation_per_myr` 0.15 × the step | 0.6 |
| a trench sample's pull to the trench depth (PLT-053) | `trench_relax` 0.5 | 1 − 0.5^(step ÷ `trench_half_life_myr` 2.0) | 0.75 |
| global resampling, at least and at most (PLT-098) | `resample_min_steps` 10, `resample_max_steps` 60 | `resample_min_myr` 20, `resample_max_myr` 120 | 5 and 30 steps |
| a contact unseen is closed (PLT-059) | `contact_timeout_steps` 12 | `contact_timeout_myr` 24 | 6 steps |
| a contact is confirmed (PLT-058, CRE-025) | `CONFIRM_STEPS` 3 | `CONFIRM_MYR` 6, the fewest steps that span it | 2 steps (8 Myr) |
| a collision without a front parts (PLT-225) | `GRACE_STEPS` 3 | `GRACE_MYR` 6, as many steps as fit in it | 1 step |
| a structure off continental crust is lost (DEF-035) | `LOST_AFTER_STEPS` 10 | `LOST_AFTER_MYR` 20 | 5 steps |
| mud chokes carbonate (ERO-065) | `carbonate_clastic_m` 20 m | `carbonate_clastic_m_myr` 10 m × the time since the step began | 40 m at the step's end |

The plate checks that measure one step (the most land a step gains, the most land one collision
raises in a step, the change of a plate's rotation from one step to the next; CHK-008, CHK-009,
CHK-030, CHK-031, PLT-150, PLT-151) had their ranges set at the 2-Myr step, so a run at another
step reports them per 2 Myr of step (`TARGET_STEP_MYR`, a game-choice: exact for a steady rate).
The rotation's change does not grow with the step (the balance is solved anew each step), so at
4 Myr its scaled median falls from 3.7 % to 0.7 and 1.1 % (1000 Myr); the range still holds, but
the figure does not compare across steps.

**What stays per step, and why.** The landscape steps per plate step and the solver's iterations
per landscape step (`substeps`, `spl_max_iter`) are numerical settings. A stratovolcano's one
forgiven step without arc flux (VOL-010) forgives the arc band's flicker between plate samples, a
sampling rule. The rest were in time already: the collapse (1 − e^(−step ÷ `collapse_myr`)), the
rate limits (PLT-287, PLT-288: mm/yr × the step), the arcs' and the cordillera's growth and the
subduction uplift (× the step), the orogeny stamp (m/Myr × the step), the founder (a thickness),
the plateau spreading (an implicit diffusion over the step).

**Measured.** `earthlike_quick_16k`, seeds 1 to 3, both starts, 500 and 1000 Myr, 2 threads,
`tools/review/force-balance-ensemble.sh` and `tools/review/p1-17-hyps.py` (build log 2026-10-18,
"The coupled group's cadence"); means over six runs, [min, max]; CO₂ the geometric mean; "checks
moved" counts the 45 plate checks whose mean left master's seed range. Master steps 2 Myr with one
landscape step and a snapshot every 10 Myr. **P4**: a 4-Myr step with one landscape step of 4 Myr;
**P4S2**: a 4-Myr step with two of 2 Myr; **P3**: a 3-Myr step with one of 3 Myr; **C20**:
master's steps with a snapshot every 20 Myr. Stage time is the sum of the plates, climate,
surface, volcanoes and structure stages on a shared machine, a guide only (the timing pairs are
below).

| 500 Myr | master | P4 | P4S2 | P3 | C20 |
|---|---|---|---|---|---|
| land | 0.191 [0.152, 0.219] | 0.199 | 0.198 | 0.165 | 0.195 |
| land above 2 km (share of land) | 0.206 [0.154, 0.239] | 0.290 [0.222, 0.351] | 0.231 [0.193, 0.263] | 0.342 [0.230, 0.457] | 0.238 [0.141, 0.348] |
| continental crust | 0.396 [0.368, 0.418] | 0.397 | 0.401 | 0.361 | 0.404 |
| highest cell, m | 6,689 [5,655, 7,499] | 8,916 [8,208, 9,752] | 8,913 [8,342, 9,666] | 9,144 [7,885, 11,401] | 7,117 [6,818, 7,511] |
| loss to the mantle, km³/yr (with the slab's) | 2.13 (3.81) | 1.87 (4.11) | 1.76 (4.72) | 2.06 (5.18) | 2.02 (3.89) |
| plateau wear, m/Myr | 144.6 | 124.0 | 173.3 | 141.5 | 160.2 |
| collision wedge, km (runs above 0) | 94 in one run | 34 to 116 in four | 75 and 84 | none | none |
| T, °C | 15.7 [13.2, 19.3] | 20.2 [16.6, 22.3] | 15.9 [14.4, 18.8] | 23.1 [16.1, 29.9] | 17.7 [15.8, 21.6] |
| CO₂, ppm | 837 [477, 2,862] | 4,532 [1,474, 9,351] | 1,072 [508, 2,730] | 7,463 [1,273, 34,573] | 1,810 [929, 8,208] |
| carbonate, km³ | 811,217 | 429,567 | 684,400 | 472,667 | 802,650 |
| stratovolcanoes born (standing) | 13,252 (489) | 10,228 (366) | 10,791 (367) | 12,525 (489) | 13,432 (576) |
| checks met; checks moved | 29 | 28; 9 | 30; 6 | 29; 4 | 29; 1 |
| stage time, s | 751 | 366 | 402 | 476 | 731 |

| 1000 Myr | master | P4 | P4S2 | P3 | C20 |
|---|---|---|---|---|---|
| land | 0.168 [0.118, 0.211] | 0.189 | 0.188 | 0.178 | 0.171 |
| land above 2 km | 0.215 [0.050, 0.347] | 0.329 [0.237, 0.417] | 0.347 [0.317, 0.361] | 0.269 [0.176, 0.347] | 0.205 [0.079, 0.358] |
| continental crust | 0.431 [0.346, 0.526] | 0.404 | 0.392 | 0.437 | 0.447 |
| highest cell, m | 6,463 [4,878, 7,682] | 8,880 [8,010, 9,712] | 8,708 [8,286, 9,545] | 7,981 [7,339, 9,153] | 6,524 [5,226, 7,415] |
| loss to the mantle, km³/yr (with the slab's) | 2.96 (5.03) | 2.50 (5.54) | 2.72 (6.00) | 2.33 (4.63) | 3.02 (5.29) |
| plateau wear, m/Myr | 121.7 | 106.3 | 150.8 | 119.9 | 109.9 |
| collision wedge, km (runs above 0) | none | 68 in one | 35 and 77 | 81 in one | none |
| T, °C | 17.2 [15.2, 21.4] | 22.1 [16.6, 30.7] | 21.9 [18.1, 25.7] | 19.2 [15.0, 24.3] | 19.0 [13.8, 27.6] |
| CO₂, ppm | 1,209 [425, 4,668] | 6,709 [1,803, 43,973] | 7,709 [2,816, 20,138] | 2,456 [482, 12,396] | 1,704 [331, 23,563] |
| carbonate, km³ | 1,487,283 | 703,633 | 1,171,217 | 933,950 | 1,481,350 |
| stratovolcanoes born (standing) | 31,898 (505) | 24,375 (408) | 24,717 (422) | 26,166 (396) | 32,420 (526) |
| collisions per run | 21.7 | 18.5 | 17.7 | 20.3 | 19.3 |
| checks met; checks moved | 27 | 27; 12 | 26; 13 | 26; 5 | 25; 1 |
| stage time, s | 1,266 | 699 | 731 | 845 | 967 |

P3's five moved checks at 1000 Myr: the median collision lasts 58.8 Myr against 40.7 (a pass
becomes a fail), the most land one collision raises in a step falls from 0.089 to 0.055 % of the
planet, the lowest continental share rises from 0.332 to 0.369, the ocean floor older than 250 Myr
grows from 3.9 to 4.9 % of the sea floor, and one run grows a wedge (81 km). P4 and P4S2 move 9
and 6 checks at 500 Myr and 12 and 13 at 1000. Besides the arc checks (below) and the per-step
checks (above), those are the collision length (P4's 90th percentile at 500 Myr, 118.7 to 161.3
Myr, and P4S2's median at 1000, 40.7 to 57.7 Myr: passes become fails), the median plate's life,
the lowest continental share, the subducting share of the boundary, the fastest plates' speed, the
foredeep and the wedge (these on the same side of their targets as master's). The full lists are in
the build log.

- *Every coarser plate step raises the high ground.* The highest cell stands 1.5 to 2.5 km higher
  at a 3- or 4-Myr step whether the landscape step is 2, 3 or 4 Myr, and the land above 2 km
  grows (P4S2's mean at 1000 Myr is master's highest run). The landscape step is not the cause:
  P4S2 erodes in master's 2-Myr steps and its peaks match P4's. On master's high ground the
  critical slope does most of the work (supercontinent seed 2 at 1000 Myr, `erosion_bands`: 389 of
  the 430 m/Myr eroded above 4 km is the slope's cut), so the height there is set by the slope and
  the width of the uplifted ground more than by the river law's rate; P4S2's ground above 4 km
  erodes at 335 m/Myr over 1.6 times the area. Not traced to one rule. A candidate: the surface
  carries its elevation with the plates by bilinear semi-Lagrangian advection once per plate step,
  and the blur that leaves (`surface/advect.rs`: it "blurs detail that erosion re-cuts every
  step") halves when the steps halve.
- *The higher ground makes the present hotter.* The temperate present's nudge raises CO₂ until the
  land under ice sheets is at most 0.15 (CLM-150; climate.md, "The present-day nudge on high
  ground"), and high ground holds ice: P4S2's six 1000 Myr runs all end with 0.32 to 0.36 of their
  land above 2 km and need 3.9 to 19.6 times their natural CO₂ (master: 1 to 4 times). A
  Mann–Whitney test on the six runs each gives p ≈ 0.015 for P4S2's CO₂ at 1000 Myr against
  master's.
- *A landscape step longer than 2 Myr cuts the carbonate* (P4 and P3, 37 to 53 % less; P4S2,
  with 2-Myr landscape steps, 16 to 21 % less), and P4's and P3's CO₂ is high already at 500 Myr.
- *The arc checks that pass at P4, P4S2 and P3 are a sampling effect.* The arc elevation measures
  read the arcs after the plates' uplift and before the surface erodes, and an island arc's axis
  rises by about 330 m per Myr of growth (PLT-045, at v₀): 2 more Myr of step read about 660 m
  higher (measured: the continental arcs' median 709 m at master, 1,348 at P4S2, 1,257 at P3).
- *Fewer stratovolcanoes at 4 Myr* (19 to 24 % fewer born, 17 to 25 % fewer standing; at 3 Myr
  18 % fewer born over 1000 Myr, 5 % over 500). A cone is
  born where the arc flux is enough and none stands within `arc_spacing_km`, and lives 2 to 8 Myr
  (VOL-038), one or two steps at 4 Myr. Not traced.
- *C20 leaves the plates alone:* one check moved at each length (the 90th percentile collision's
  length at 500 Myr, 118.7 to 79.0 Myr, a pass both; the subducting share of the boundary at 1000
  Myr, 47.6 to 48.9 %, a fail both). At 1000 Myr 25 checks pass in the all-runs table against
  master's 27: four checks near a bound flipped (three to a fail, one to a pass) with their means
  inside master's seed range. Its CO₂'s geometric mean is higher (1,810 against 837 ppm at
  500 Myr, 1,704 against 1,209 at 1000) but within the nudge's spread: a Mann–Whitney test gives
  p ≈ 0.1 and 0.8, the natural CO₂ before the nudge is 830 against 550 ppm and 640 against 630,
  and the high means come from one run at 500 Myr and two at 1000 whose natural end had 0.28 to
  0.31 of the land under ice sheets (master's most: 0.24).

**Time** (`tools/review/bench-quiet.py` pairs, `earthlike_full_16k`, scattered start, 8 threads,
master and the setting side by side on CPU sets A and B; build log 2026-10-18). The test machine
ran the other speed-up agents' jobs throughout (in the 1000 Myr pair the other jobs kept 18 of the
32 CPUs busy on average, load up to 111), so a pair compares two runs under the same load and the
absolute times are not the quiet gate's 1,242.8 s.

| pair | set A | set B | setting ÷ master |
|---|---|---|---|
| P4, 200 Myr, round 1 | master 459.4 s | P4 338.1 s | 0.736 |
| P4, 200 Myr, round 2 (swapped) | P4 549.6 s | master 683.0 s | 0.805 |
| C20, 1000 Myr, one round | C20 2,219.0 s (climate stage 247.0 s) | master 2,308.5 s (climate stage 332.0 s) | 0.961 (climate stage 0.744) |

P4S2 and P3 were not timed in pairs: both change the planet too much, and their summed stage time
in the ensembles is 0.54 and 0.63 of master's at 500 Myr (P4's 0.49, which the pairs put at 0.74
to 0.81 of the whole run, since the stages after the group and the bake do not shrink). On the
quiet gate run the work that scales with the snapshots is about 80 s (the climate snapshot's cells
46.1 s; the 2-D energy balance about 35 s, five times the 7.0 s measured over 200 Myr;
docs/design/p1-17-benchmark-gate.md), so C20 saves about 40 s of the 1,242.8 s, 3 to 4 %, as the
1000 Myr pair's 0.961 says. On its own it does not reach 900 s.

**What was chosen.** C20 in the pipeline and earthlike presets (SCL-038). The plate step stays at
2 Myr and the landscape step at one per plate step: every coarser plate step tried gives a planet
with more high ground and a hotter, CO₂-richer present, and P4 and P3 also cut the carbonate by 37 to 53 %.
The rules above count time now, so a coarser step is one preset key once the high ground's
dependence on the step is found and removed.

**Again on master of 2026-10-19** (fde7e021: plates parts 1 and 2 and the climate, surface and
after-group speed-ups; build log 2026-10-18, "The coupled group's cadence"). The same ensembles
with the shipped x86-64-v3 builds (`tools/natives/gate-bins.sh`): master; **C20**, this branch as
it ships (the per-Myr rules and the 20-Myr snapshot); **P4C20**, the branch with 4-Myr steps. At
2 Myr with a 10-Myr snapshot the branch still prints master's 40 Myr summary (the words that differ
are those that differ between two runs of master: memory and timings).

| | master 500 | C20 500 | P4C20 500 | master 1000 | C20 1000 | P4C20 1000 |
|---|---|---|---|---|---|---|
| land | 0.184 | 0.192 | 0.191 | 0.186 | 0.184 | 0.194 |
| land above 2 km | 0.241 [0.184, 0.282] | 0.237 | 0.308 | 0.275 [0.161, 0.322] | 0.261 | 0.351 |
| continental crust | 0.389 | 0.380 | 0.387 | 0.418 | 0.415 | 0.397 |
| highest cell, m | 6,903 [6,464, 7,368] | 7,114 | 9,834 | 7,042 [5,778, 8,680] | 6,615 | 9,247 |
| loss to the mantle, km³/yr (with the slab's) | 2.50 (4.31) | 2.64 (4.96) | 1.89 (4.68) | 3.30 (5.46) | 3.10 (5.08) | 2.79 (5.89) |
| plateau wear, m/Myr | 158.5 | 144.3 | 119.0 | 116.6 | 121.9 | 98.1 |
| collision wedge, km (runs above 0) | 36 in one | 68 to 75 in three | 39 to 85 in three | 78 in one | none | none |
| T, °C | 18.5 [15.5, 24.3] | 16.9 | 20.9 | 17.2 [14.8, 20.3] | 17.5 | 20.1 |
| CO₂, ppm (geometric mean) | 2,237 | 1,325 | 5,553 | 1,664 | 1,712 | 5,126 |
| carbonate, km³ | 779,183 | 693,083 | 401,550 | 1,280,800 | 1,292,750 | 708,100 |
| stratovolcanoes born (standing) | 13,997 (607) | 14,165 (616) | 10,105 (388) | 32,010 (589) | 31,174 (497) | 24,516 (379) |
| checks met; checks moved | 29 | 30; 3 | 29; 8 | 25 | 26; 4 | 27; 12 |
| climate stage, s | 91.1 | 68.6 | 59.5 | 157.5 | 118.7 | 95.4 |
| stage time, s | 450 | 428 | 265 | 867 | 820 | 503 |

- *C20 still leaves the planet alone.* Its moved checks: at 500 Myr the rank correlation of speed
  against plate area (−0.416 to −0.376, a fail becomes a pass), the supercontinent's tenure (35 to
  135 Myr, a fail becomes a pass) and the continental arcs' median (659 to 830 m, a fail both); at
  1000 Myr the largest plate's median share (0.175 to 0.208), the rift-born plates alive after
  50 Myr (78.7 to 72.6 %) and the fast plates' 90th percentile speed (74.7 to 73.7 mm/yr), passes
  all three, and the supercontinent period (172.5 to 121.5 Myr, a fail both). The present's CO₂ is now
  below master's at 500 Myr and level with it at 1000, so the higher means of the first
  measurement were the nudge's spread, as the Mann–Whitney tests said.
- *A 4-Myr step still makes the planet worse:* the highest cell 2.2 to 2.9 km higher, more land
  above 2 km, the present 2.4 to 3.0 °C warmer with 2.5 to 3.1 times the CO₂, 45 to 49 % less
  carbonate, a quarter fewer stratovolcanoes, and 8 and 12 checks moved. The plates' rewrite did
  not change the verdict.

**Time on that master** (`tools/review/p1-19-pair.sh`, the full 16k gate run, scattered start,
1000 Myr, 8 threads, the two shipped builds side by side on CPU sets A and B, then swapped):

| order | set A | set B | branch ÷ master |
|---|---|---|---|
| ab | master 1,374.1 s (climate stage 176.3 s) | branch 1,880.5 s (192.7 s) | 1.369 (climate 1.093) |
| ba | branch 1,598.7 s (155.6 s) | master 2,155.2 s (262.4 s) | 0.742 (climate 0.593) |
| P4C20, 200 Myr | master 280.8 s | P4C20 210.6 s | 0.750 |
| P4C20, 200 Myr, swapped | P4C20 215.6 s | master 290.8 s | 0.741 |

The geometric mean of the two orders is 1.008 for the whole run and 0.805 for the climate stage.
Under the other agents' load set B ran each build about 35 % slower than set A (its runs had 2.5
and 3.6 CPUs on average, set A's 3.3 and 5.2), so a change of a few % in the whole run is below
what these pairs resolve; the stages C20 does not touch came out at 1.007 (plates) and 1.044
(surface), which is that noise. The climate stage's cut is the measured saving: 0.81 of master's
in the pairs and 0.75 in the ensembles at both lengths. On the quiet gate run, whose climate stage
after its speed-ups is about 200 s per 1000 Myr (build log 2026-10-19, the climate stage's
entry: 27.6 s per 100 Myr with the 7.7 s present search once), that is roughly 40 to 50 s.

P4C20 takes 0.746 of master's time over 200 Myr (both orders, geometric mean; the coupled group's
stages 0.63 and 0.68, the rest of the run fixed), and its ensemble stage sums are 0.59 and 0.58 of
master's at 500 and 1000 Myr, so at the gate's length it would save about a third of the run. It
stays out for the planet's sake (above).

## The 4-Myr plate step (P1-19 #27 unlock, 2026-10-19)

The task after C20 landed: find every place where a longer plate step changes the physics rather
than only the cost, make it per Myr or exact in the step, and measure a 4-Myr step against C20 as
ensembles with the chaos separated from the bias. Ben (decisions log 2026-10-19): "I'm a little
worried about the impact on quality of worlds by increasing the plate step, make sure to do
thorough testing on it". So the 4-Myr step is a preset switch and the 2-Myr step stays the
default; whether it becomes the default is Ben's call after the comparison. Code:
`surface/{advect,mod,params}.rs`, `climate/mod.rs`, `surface/volcanoes/mod.rs`. Ledger: ERO-185,
ERO-186, VOL-040.

**What changed in the code.** None of the three changes a run at a 2-Myr step.

- *The carry's blur* (ERO-185; erosion-and-sediment.md, "The carry's blur and the spreading at
  long plate steps"). The surface rides the plates by one bilinear carry a plate step, and each
  carry blurs by the same amount whatever the displacement, so a 4-Myr step blurred half as much
  per Myr. A step longer than 2 Myr now adds the blur the 2-Myr carries would have left, over
  cells of the same crust; the regolith and the climate's ice and ELA fields take the same.
- *The crust's spreading* (ERO-186). Its diffusivity is the cube of the excess height, and one
  solve over a step holds it at the step's start; a long step is now solved in sub-steps of at
  most 2 Myr.
- *A dead cone's slot* (VOL-040). A stratovolcano that outlived its drawn life frees its place on
  the arc from the end of that life, not from the step that found it dead (at 4-Myr steps the
  slot stayed shut up to a step too long).

**What still depends on the step** (the per-step audit of the coupled stages; the rules that
counted steps were made per Myr in the cadence task above).

| what | at a 2-Myr step | at a 4-Myr step |
|---|---|---|
| the landscape step (`surface.substeps`, ERO-018) | one of 2 Myr | one of 4 Myr, or two of 2 with `substeps` 2; inside it the river solver's iterations (`spl_max_iter`), the slope failure's (ERO-169), the glacial sub-steps' caps (4 a landscape step, each taking at most 100 m of till and lowering the bed by at most half the ice's thickness) and the frozen flow routing are per landscape step |
| the isostatic answer to a step's erosion | lands after that step's erosion | the same, over twice the erosion; the diagnosis branch's `rebound_ref_myr` (screened as p4fix, not in this branch) holds back the share beyond 2 Myr |
| a stratovolcano without arc flux (VOL-010) | one 2-Myr step forgiven | one 4-Myr step forgiven |
| a cone's life (VOL-038) | ends at the first step end after its drawn death, 1 Myr late on average | 2 Myr late on average |
| hotspot births | at most one a hotspot a step | the same: above 17.5 mm/yr over the plume (35 at 2 Myr) a chain's spacing is the plate's motion in a step |
| cadences of round(X ÷ step) steps | every 10 Myr (margins, ages, arc bases VOL-071) | every 12 Myr; the 20-Myr climate snapshot stays 20 |
| times compared at step ends (orogeny minimum, slowdown PLT-226, welds PLT-246) | rounded up to 2 Myr | rounded up to 4 Myr |
| the event record (`surface/events.rs`) | events on 2-Myr step ends; pulses and basins found from the step's mean rates | on 4-Myr step ends, from 4-Myr means |
| the strata log | consecutive entries of a land column 2 Myr apart at the median | 4 Myr apart (with `substeps` 2 as well) |
| contacts and collisions | found from 2 Myr of motion (200 km at 100 mm/yr) | from 4 Myr (400 km), not sub-stepped |
| the ocean floor's subsidence (pools.rs) | one explicit step, 0.8 % of the way to the −10 km floor at 0.04 mm/yr | 1.6 %; the exact exponential differs by about 0.01 % |
| `ice_relax_kyr` (GLA-074) | per plate step | the same; inert, as the ice dynamics are off in every preset |

The plate checks that measure one step are reported per 2 Myr of step (`TARGET_STEP_MYR`, above),
but the rotation's change, the land one step gains and the partings' lengths still read the
step's sampling.

**The screen** (quick 16k, 2026-10-20). `earthlike_quick_16k`, 1000 Myr, 2 threads, seeds 1 to 3
on both starts, six runs a setting, all on this branch's build (0c368e27) except p4fix and
p4fs2, which ran the diagnosis branch's (0c368e27 with 29bfe825 and afaec853, `rebound_ref_myr`
2). **C20** is the default; **c20tw** is C20 with the serial flood fill
(`surface.numerics.flood=serial`), which changes only rounding, so C20 against c20tw is what
chaos alone does (the A/A row); **p4** `tectonics.time_step_myr=4`; **p4s2** p4 with
`surface.substeps=2`; **p4fix** p4 with the rebound hold-back; **p4fs2** p4fix with
`surface.substeps=2`. Each run's 820 numbers (`build/p4/metrics.py`: relief, belts, plates and
the plate checks, ocean floor, continents and coasts, rivers, crust, strata, volcanoes, climate,
the event record) were compared with C20's by `build/p4/compare.py`: a number is flagged when
the Mann-Whitney p ≤ 0.1, or |d| ≥ 0.8, or all of five or more pairs (the same seed and start)
move the same way, or the pairs' Wilcoxon signed-rank p ≤ 0.1, or (against the twin) the pair
differences differ from the twin's at p ≤ 0.1. The rule that picked the setting for the
full runs was written before the tables were computed: the flagged numbers among 66 headline
ones (land share, high ground and the plate checks met counted twice) plus the flagged
plate-check numbers (141); lowest wins.

| | c20tw (A/A) | p4 | p4s2 | p4fix | p4fs2 |
|---|---|---|---|---|---|
| headline numbers flagged (weighted) | 8 (9) | 37 (43) | 30 (36) | 33 (36) | 28 (31) |
| plate-check numbers flagged, of 141 | 23 | 65 | 48 | 72 | 65 |
| all numbers flagged, of 820 | 68 | 391 | 377 | 436 | 372 |
| score | 32 | 108 | 84 | 108 | 96 |

The written rule described the flag without its Wilcoxon part; scored that way the five are
26, 107, 80, 106 and 94, the same order. p4s2 against p4fs2 directly: 160 of 820 flagged. Means
over the six runs (C20's range in
brackets; * flagged, † outside C20's range):

| | C20 | c20tw | p4 | p4s2 | p4fix | p4fs2 |
|---|---|---|---|---|---|---|
| land | 0.166 [0.126, 0.193] | 0.179 | 0.191* | 0.205*† | 0.216*† | 0.205*† |
| land above 2 km (share of land) | 0.248 [0.124, 0.383] | 0.254 | 0.324* | 0.334* | 0.299 | 0.290 |
| highest cell, m | 7,038 [5,415, 7,563] | 6,866 | 8,989*† | 9,255*† | 7,790*† | 7,985*† |
| land median, m | 809 [466, 1,312] | 884 | 1,094* | 1,181* | 1,008* | 1,010 |
| mountain belts | 19.2 [17, 22] | 20.0 | 15.8*† | 17.0 | 16.7*† | 16.8*† |
| plates (run mean) | 21.6 [17.6, 23.8] | 22.9* | 22.3 | 22.7 | 22.1 | 22.6 |
| size ranks' L1 to Bird 2003 | 0.282 [0.251, 0.308] | 0.288 | 0.274 | 0.269 | 0.261* | 0.255* |
| plate speed p50, mm/yr | 21.3 [17.8, 25.1] | 23.4 | 26.1*† | 22.7 | 23.8* | 25.0* |
| plate checks met, of 45 | 25.8 [22, 28] | 28.3*† | 28.0* | 27.3* | 25.8 | 27.5* |
| ocean floor age mean, Myr | 89.3 [71.1, 120] | 85.5 | 88.0 | 79.1 | 79.5 | 82.7 |
| collisions per 100 Myr | 1.92 [1.70, 2.20] | 2.00 | 1.58*† | 1.70 | 1.67*† | 1.68*† |
| Hack's exponent | 0.450 [0.348, 0.500] | 0.482* | 0.465 | 0.428 | 0.473 | 0.494 |
| carbonate, km³ | 1,452,583 | 1,404,667 | 670,017*† | 1,086,433* | 606,417*† | 1,088,833* |
| eroded, km³ | 296.5 M | 308.2 M | 241.2 M*† | 296.7 M | 272.8 M* | 321.2 M |
| stratovolcanoes born | 30,898 | 30,720 | 24,388* | 25,283* | 25,331* | 26,480* |
| T, °C | 19.1 [16.0, 26.9] | 17.5 | 21.2* | 20.4 | 22.5* | 21.7 |
| natural CO₂, ppm | 907 [532, 2,434] | 818 | 1,010 | 1,241* | 1,571* | 1,161 |
| distinct event times per 100 Myr | 46.9 [46.1, 48.3] | 47.2 | 24.0*† | 24.0*† | 24.0*† | 24.1*† |
| strata layer span p50, Myr | 2 | 2 | 4*† | 4*† | 4*† | 4*† |
| strata log layers per land column | 5.37 [4.41, 5.97] | 6.42*† | 4.22*† | 4.11*† | 4.75* | 4.15*† |

p4s2 scored lowest and went to the full runs (C20 and p4s2 only; the coordinator's call).
Shared by every 4-Myr setting: the event record and the strata log on 4-Myr ends, the per-step
plate numbers (the rotation's change, the land one step gains, partings of 8 against 5.3 Myr and
24 against 35 per 100 Myr) and the arcs read higher by the sampling effect above (continental
arcs' median 1.19 to 1.41 km against 0.66). The screen's full tables are on the review page
below.

**The full runs** (full 16k, 2026-10-20). `earthlike_full_16k`, 1000 Myr, 8 threads, C20 and
p4s2 on scattered seeds 1 to 6 and supercontinent seeds 1 to 3 (nine runs each, the same seeds),
c20tw on scattered seeds 1 and 2, all on 0c368e27's build; each run baked (deposits, the young
landscape, amplify) and measured twice, at the end of the coupled stages and in the baked world
(`metrics.py`, 1,146 numbers a run), then compared by the same rule. The A/A row sets C20's nine
runs on both starts against c20tw's two scattered ones, so its sides differ in their starts too;
the last two rows set both settings against C20 on c20tw's two runs only, where only |d| ≥ 0.8 can
flag a number.

| | c20tw (A/A) | p4s2 |
|---|---|---|
| runs: C20, the setting, pairs | 9, 2, 2 | 9, 9, 9 |
| headline numbers flagged (weighted) | 18 (18) | 29 (34) |
| plate-check numbers flagged | 41 of 138 | 43 of 141 |
| all numbers flagged | 293 of 1,146 | 461 of 1,157 |
| on c20tw's two runs (2 v 2): headline (weighted), plate checks, all | 28 (28), 71 of 138, 483 of 1,146 | 42 (45), 95 of 136, 685 of 1,146 |

Means over the nine runs (C20's range in brackets; * flagged, † outside C20's range; "baked" is
the same number in the baked world):

| | C20 | p4s2 |
|---|---|---|
| land | 0.208 [0.170, 0.268] | 0.215 |
| land, baked | 0.194 [0.150, 0.259] | 0.200 |
| land above 2 km (share of land) | 0.274 [0.134, 0.384] | 0.353* |
| land above 2 km (share of land), baked | 0.227 [0.115, 0.320] | 0.284* |
| highest cell, m | 6,754 [5,639, 7,484] | 9,071*† |
| highest cell, m, baked | 6,411 [5,518, 7,075] | 7,556*† |
| land median, m | 993 [644, 1,458] | 1,278* |
| land median, m, baked | 857 [585, 1,165] | 1,048* |
| mountain belts | 25.0 [15.0, 39.0] | 20.7 |
| plates (run mean) | 20.6 [17.2, 23.6] | 21.1 |
| size ranks' L1 to Bird 2003 | 0.284 [0.215, 0.430] | 0.279 |
| plate speed p50, mm/yr | 19.8 [14.3, 25.5] | 19.4 |
| plate checks met, of 45 | 26.8 [25.0, 30.0] | 25.7* |
| ocean floor age mean, Myr | 95.9 [73.9, 117.1] | 90.9 |
| collisions per 100 Myr | 2.50 [1.50, 3.90] | 2.07 |
| Hack's exponent | 0.478 [0.403, 0.574] | 0.465 |
| carbonate, km³ | 1,458,222 [1,129,000, 1,757,000] | 1,358,578 |
| eroded, km³ | 310.5 M [239.6 M, 378.3 M] | 287.3 M* |
| stratovolcanoes born | 34,551 [26,388, 41,028] | 26,846* |
| T, °C | 19.7 [14.5, 26.1] | 20.6 |
| natural CO₂, ppm | 673 [311, 1,460] | 1,167* |
| distinct event times per 100 Myr | 47.8 [47.3, 48.8] | 24.3*† |
| strata layer span p50, Myr | 2 [2, 2] | 4*† |
| strata log layers per land column | 6.53 [5.22, 7.89] | 4.43*† |

**Time** (the VM, paired, 2026-10-20: `build/p4/pair.sh`, `deeptime bench --preset
earthlike_full_16k --start scattered_continents --length-myr 1000` with 8 threads, C20 and p4s2
side by side on two pinned sets of 8 CPUs, then swapped, through `tools/review/bench-quiet.py`;
other jobs kept 18 to 20 of the VM's CPUs busy on average). Wall, C20 against p4s2: 1,183
against 868 s, then 1,269 against 953 s (p4s2 at 0.73 and 0.75 of C20). Stage means of the two
rounds: plates 306 against 167 s, the surface 588 against 481, climate 134 against 104,
volcanoes 28.6 against 12.6, structure 43 against 36, the bake (deposits to finalize) 117 against
101. Largest resident set 4.44 and 4.48 GiB against 4.34 and 4.32 GiB.

**The 2-Myr step, unchanged** (identity check, 2026-10-20, `build/p4/idcheck.sh`). The master
side of the merge (3e7edd49: the same crates without this change, built as the shipped x86-64-v3
binary) and the branch's build (0c368e27) ran the defaults: quick 16k, scattered seed 1, 100 Myr
to the volcanoes stage, 2 threads. The 101 fields read back from the two checkpoints are
byte-identical, the dynamics records are identical, and so is the plate-check report but for the
checkpoint's own path in it. The stage lines differ in wall times and in the logged preset,
which on the branch names the two new settings at 2 Myr. The checkpoint files differ in length by
72 bytes: their JSON summaries are identical, and the body, which holds the preset, is longer on
the branch.

All of it, with the per-seed tables (every headline number, each run's value under both settings
and c20tw's where it ran), the curves, the keyframes and bakes side by side and two BlueMap sites
a world (the highest mountain box and the most coastline box, by a rule fixed before any picture
was taken), is on the review page,
[deeptime-p4-c20-review](https://spacesheep.dev/@benmac/deeptime-p4-c20-review) (private).

**Measured before the continents' fix.** Every run above is on 0c368e27, before the
lower-crustal flow and the settling ("Mountains eat the world: the continents drown", below; land
0.16 to 0.22 before them, 0.31 to 0.32 after, on its runs), so none of these numbers says how a
4-Myr step behaves with them. Their rules, read for the step and not run:

- *The settling* (ISO-078) moves a column 1 − e^(−δt/τ) of the way to its height, with τ = 4
  kyr: the whole way at 2 and at 4 Myr. The surface's flexure it adds is the surface stage's
  last one, a plate step old.
- *The flow's heights* (PLT-281) follow from each column's crust before and after the step, as
  the collapse's do, and the settling then takes them to equilibrium.
- *The flow's substeps* (PLT-300): explicit, as many as keep it stable. The P4 review estimated
  that the old bound of 64 a step would bind at full 16k above about 74 km of crust; but the
  crust is held to `crust_max_km` 70, where the channel is 20 km and K = 2.4 K₀, not the 8 K₀
  the estimate took. Measured on 2026-10-20 (land-shape task; the step's substeps and the plate
  time they left uncovered are now world globals, `plates/channel_flow_substeps` and
  `plates/channel_flow_short_myr`, printed by the crust budget): a full 16k run (seed 1, both
  starts, 1000 Myr) with the bound and one without are the same to the bit, the steps took 30
  substeps on average (12 at quick 16k), and no plate time went uncovered. The bound is now only a
  guard (4096), and a step longer than 2 Myr runs in 2-Myr pieces, each finding its channels and
  rings again, so a 4-Myr step moves crust as two 2-Myr steps do (test
  `a_long_step_is_two_short_ones`).
- *The flow's reach.* Crust moves only among the channel columns and the two rings of their
  plate's crust around them, found once a piece, so a channel's front can move at most two rings
  (about 150 km at quick 16k, 100 km at full 16k) per 2 Myr. K₀'s length over 15 Myr is Rey,
  Teyssier & Whitney's 150 km; √(K t) over a 2-Myr piece at the thickest channel (2.4 K₀) is 84
  km. Measured with the substeps (`plates/channel_flow_edge`, the outermost-ring columns left
  with a channel): about 9 a step at full 16k and 1.5 at quick 16k, against roughly 13,000
  columns thicker than 50 km at full 16k (about 12 % of the continental crust in the crust
  budget's bands, 0.43 of 250,000 samples), so the reach's dependence on the lattice moves almost
  nothing and was left as it is.

## L1 sampling on a GPU (2026-10-19)

**What changes.** With a usable GPU (`compute` auto or gpu; `docs/design/gpu-compute.md`, section
12) the relink that `plates/l1.rs` runs before every L1 sampling settles the covered cells on the
device: for each cell, the nearest visible sample (from a latitude-band bucket grid, ties to the
lower index), the walk to the lattice triangle holding the cell in
that sample's rigid group, the test that all three corners are that group's and none is hidden, and
the barycentric weights. Every other cell (boundary strips, gaps, a walk the device could not
finish) goes to the CPU's code unchanged, and the layers (the barycentric fields, the nearest
sample's categories, the gap crust) are written on the CPU from the links. No law, constant or
threshold changes; no ledger row changes and none is added.

**What that does to the crust.** The device works in f32 on unit vectors, about 1e-7 rad (under a
metre on Earth's radius), where the CPU works in f64. A link can therefore differ only at a tie:
two samples equidistant from the cell to that precision, or a cell on a triangle's edge. The
weights differ by the rounding of f32 directions. The tolerance test
`plates::l1::gpu::tests::the_gpu_relink_matches_the_cpu_but_at_ties` steps a 6,000-sample planet
through 60 Myr from a supercontinent and compares six samplings on lavapipe (2026-10-19): of
125,952 cells the device settled 88.4 % and left 11.6 % to the CPU; no link differed; the weights
differed by at most 1.6e-5; and every layer value equalled the CPU's to 1e-4 of its layer's
largest. A world made with a GPU is not byte-identical to one made on the CPU (Ben, 2026-10-18:
worlds are shared as finished folders); the whole-history comparison of a full 16k planet is in
the design note, section 10.

**On a whole planet (2026-10-19).** A temporary switch (not committed) ran both relinks from the
same state on every one of the 50 samplings of a full 16k planet's 100 Myr history (lavapipe,
67.2 million cells in all): 0.015 % of the links fell on another triangle and the weights agreed
to about 8e-4 where the triangle did; a few cells per sampling sat on an edge that the CPU's f64
test put outside and the device's f32 test inside. Every layer's signed difference was as often
up as down (elevation 27.29 million cells up and 27.28 million down, mean -4e-5 m; the land count
off by 2 cells in 19.6 million; continental crust 29 cells up and 24 down), so the relink does
not lean the crust any way. The world still differs, because the climate's carbon cycle carries
a last-bit difference forward (`docs/science/per-cell-climate.md`, "Down to L1 on a GPU"): on
five planets the GPU's land share ended 0 to 0.013 above the CPU's (a mean of +0.006, higher in
four, equal in one), where one f32 ulp on one weight moved the CPU's own land share on the first
planet by up to 0.004 and, before the cadence change, by 0.024. The same-state comparison above
finds no lean, so the four in five is chance or an effect too indirect to show in a single
sampling; a larger paired ensemble would tell.

## Ocean crust keeps its own rock (2026-10-19)

**What was wrong.** The external review of 2026-10-19 (defect D1) found granite in 73 to 86 % and
metamorphic rock in 67 to 78 % of the sea-floor columns of run 1a156f22b66, and 11 to 14 km from
the sea floor to the peridotite where Earth has about 7.5 km (GlobSed's 0.4 km of deep-ocean
sediment over White et al. 1992's 7.1 km of igneous crust). Part of that was drawing: every crust
kind's pools were drawn with the continental classes (the rock page, RCK-095 and RCK-096). The
rest is rock the plate model puts there.

**Where the rock came from.** A diagnosis build (master d54c7f0f with counters on every
mechanism that adds felsic plutonic or metamorphic rock to an ocean sample, except the
transition; `earthlike_quick_16k`, supercontinent start, seed 1, 1000 Myr) counted:

| Felsic plutonic rock gained by ocean samples | Share |
|---|---|
| from continental corners, when a resampling blended all three corners' pools | 51 % |
| island arcs (PLT-043 to PLT-046) | 32 % |
| crust accreted at a resampling (the footprints under the new sample) | 17 % |

| Metamorphic rock gained by ocean samples | Share |
|---|---|
| diagenesis of sediment piles thicker than about 11 km | 52 % |
| from continental corners at resamplings | 34 % |
| accretion at resamplings | 13 % |

Blending within the ocean then moved 3.2 times as much felsic plutonic rock between ocean samples
as all the sources added, a little further at every resampling. The first change (PLT-293) stops
both: a resampling and the L1 sampling blend each pool over corners of the sample's own kind, and
on ocean crust a resampling keeps the felsic and metamorphic pools of the nearest corner. On its
own, on master d54c7f0f, it barely moved where the rock lay. The final checkpoints, read with
`deeptime dump-fields` (`tools/review/geo_template_fields.py`), show why: the share of ocean L1
cells holding more than 50 m of felsic plutonic rock falls with the distance (L1 cells, 4.8 km
each on the planet) from the nearest continental cell over about the transition's half-width
(W/2 = 100 km on the planet = 21 cells, W then on the planet), and switching the transition off (`transition_width_km` = 0) takes
most of it away (master d54c7f0f, 2026-10-19):

| cells to land | ocean cells (master) | master | PLT-293 | PLT-293, no transition | PLT-293 and PLT-294 on |
|---|---|---|---|---|---|
| 1 | 36,171 | 0.96 | 0.91 | 0.72 | 0.73 |
| 2 | 23,360 | 0.93 | 0.91 | 0.69 | 0.70 |
| 3 | 17,839 | 0.90 | 0.89 | 0.60 | 0.61 |
| 4 to 5 | 27,380 | 0.85 | 0.84 | 0.48 | 0.48 |
| 6 to 8 | 29,531 | 0.78 | 0.76 | 0.34 | 0.37 |
| 9 to 13 | 31,611 | 0.67 | 0.66 | 0.23 | 0.31 |
| 14 to 21 | 23,576 | 0.52 | 0.50 | 0.15 | 0.21 |
| 22 to 34 | 12,340 | 0.37 | 0.29 | 0.10 | 0.12 |
| farther | 5,054 | 0.12 | 0.20 | 0.25 | 0.01 |

(Supercontinent start, seed 1, each column its own run, so the planets differ by chaos.) The
transition (PLT-261) moves the continent's own rock to the oceanic samples of its ocean side and
keeps them oceanic; on this planet nearly all the ocean lies within W/2 of a margin, so it laid
granite and gneiss between the lavas and the gabbro of nearly every ocean column.

**On today's master** (bc2e5472, the continents fix, PLT-300 to PLT-302; the sea-floor fix
49f5f543 does not touch the plate model), the same planet with checkpoints, 2026-10-20. "Before" is
master, "after" the branch's default, "switch on" the branch with PLT-294 on; each its own run:

| cells to land | ocean cells (before) | before | after | after, switch on |
|---|---|---|---|---|
| 1 | 37,906 | 0.96 | 0.89 | 0.75 |
| 2 | 23,108 | 0.94 | 0.88 | 0.71 |
| 3 | 16,875 | 0.93 | 0.84 | 0.62 |
| 4 to 5 | 23,839 | 0.90 | 0.77 | 0.48 |
| 6 to 8 | 24,484 | 0.83 | 0.67 | 0.34 |
| 9 to 13 | 26,710 | 0.72 | 0.55 | 0.23 |
| 14 to 21 | 24,364 | 0.56 | 0.41 | 0.12 |
| 22 to 34 | 16,442 | 0.32 | 0.26 | 0.06 |
| farther | 5,899 | 0.04 | 0.05 | 0.00 |

| ocean L1 cells | before | after | after, switch on |
|---|---|---|---|
| crust 0 to 20 Myr old: felsic plutonic over 50 m | 0.27 | 0.35 | 0.16 |
| crust 0 to 20 Myr old: crystalline crust, sediment cover (km) | 8.2, 0.5 | 8.0, 0.8 | 7.5, 0.4 |
| crust 20 to 65 Myr: felsic plutonic over 50 m | 0.67 | 0.73 | 0.47 |
| crust 65 to 150 Myr: felsic plutonic over 50 m | 0.89 | 0.84 | 0.63 |
| crust over 150 Myr: felsic plutonic over 50 m; sediment cover (km) | 0.99; 7.8 | 0.98; 9.6 | 0.79; 9.9 |
| regime ocean basin: felsic plutonic, metamorphic over 50 m | 0.81, 0.72 | 0.73, 0.65 | 0.46, 0.38 |
| regime ridge: felsic plutonic over 50 m | 0.48 | 0.36 | 0.21 |
| regime arc; back-arc: felsic plutonic over 50 m | 0.99; 0.87 | 1.00; 0.79 | 1.00; 0.73 |

On the new master PLT-293 moves more than it did on d54c7f0f (0.72 to 0.55 at 9 to 13 cells), and
the switch takes most of the ocean side's felsic rock away (0.23 there). The column draws the felsic
plutonic rock left on ocean crust as diorite or granodiorite and its metamorphic rock as schist
(RCK-096), never granite or gneiss; the switch decides how much of the ocean column is arc-like rock
between the lavas and the gabbro. The review's census of the sea floor (strata.md, "The ocean crust
and the mountains, measured") still finds granite in 4 % and gneiss in 21 % of the ocean columns
(62 % and 77 % before), and both come from the continent beside them: the column rule blends 4 x 4
horizon cells by key whatever their crust kind, so near a coast the continent's granite and its
gneiss (its metamorphic pool and its strata log's layers buried to 650 degC, which only continental
crust metamorphoses) reach into the ocean columns and pinch out. Of 80 such columns on the
supercontinent planet (seed 1), 58 lie within 5 km of continental crust or land and 69 within 15 km;
before the change, 6 and 12, with 33 more than 75 km from any.

**The transition's ocean side: a switch for Ben** (PLT-294). With `transition_ocean_crust` on, an
oceanic sink takes the same volume as ocean crust (lavas and gabbro in the ocean column's
proportion), as the thick igneous crust of a volcanic margin would be. That takes most of the
transition's granite off the sea floor (the "switch on" columns above), but the continental rock that
the sources give up is then lost, and land falls (`earthlike_quick_16k`, 1000 Myr, the end of the
run, seeds 1 to 3; "before" is master 49f5f543, which holds the continents fix bc2e5472, "after" the
branch's default; 2026-10-20):

| start | land, before | after | switch on | continental crust (10^9 km3), before | after | switch on |
|---|---|---|---|---|---|---|
| supercontinent | 0.264, 0.326, 0.321 | 0.309, 0.307, 0.344 | 0.277, 0.286, 0.252 | 7.5, 8.8, 8.1 | 8.0, 8.4, 9.1 | 7.9, 7.8, 7.0 |
| scattered continents | 0.289, 0.342, 0.282 | 0.356, 0.323, 0.294 | 0.221, 0.289, 0.293 | 7.5, 8.6, 7.7 | 8.5, 8.7, 7.8 | 7.2, 7.7, 8.3 |

The means: before 0.30 on both starts, after 0.32, switch on 0.27 on both, against Earth's 0.29.
"After" against "before" is within the seeds' spread (0.26 to 0.34). On master d54c7f0f, before the continents fix, the switch cost more: supercontinent 0.140,
0.131 and 0.139 against 0.165, 0.189 and 0.177.

Ben weighed the transition's land cost on 2026-10-05 ("Keep 200 km, raise crust in 7-8"), so the
switch was off at first and the choice his; since 2026-10-20 it is on in every shipped preset, the
coordinator's call under Ben's rule of 2026-10-16 once the cordillera at 1.7 mm/yr held the land at
Earth's with it on ("The ocean crust switch, PLT-294" below; decisions log 2026-10-20). The physical fix is a continent that widens as it
stretches, which the model's fixed boundary cannot do without the ratchet PLT-289 records.

**What is left under the sea** (switch on, seed 1 supercontinent on master bc2e5472, the table
above): all the arc cells and 0.73 of the back-arc cells hold island-arc rock (drawn as andesite over
diorite or granodiorite, RCK-096; the model's oceanic fronts were about 8.6 times Earth's length,
125,000 km against 14,500 at 500 Myr in plate task 8, 2026-10-17, "The crust budget on sourced
rates" above, inferred from the fronts' km × Myr and not re-measured since, PLT-045), and 0.46 of the ocean basin, 0.21 of the ridges and 0.16 of the crust younger than 20 Myr
hold some; on d54c7f0f these were 0.61, 0.45, 0.16 and 0.07. On crust older than 150 Myr the
sediment pile averages 9.6 to 9.9 km (6.4 km on d54c7f0f) and its base turns to schist (the
diagenesis above): the pile, not the ocean crust, makes old sea floor deep to its mantle.

## Mountains eat the world: the continents drown (2026-10-19)

Ben, watching a planet form (earthlike_quick_16k, supercontinent, 1000 Myr): "mountains will
'eat' the world and delete most of the continents and leave things as a stringy mess of
'islands.'" One supercontinent at 900 Myr ago, narrow snow-capped chains and island strings in a
mostly empty ocean by 650 Myr ago. Ledger PLT-300 to PLT-302, ISO-078 and HYP-035; the sources'
bibliography entries name those ids. Code: `plates/{flow.rs, settle.rs, crust_budget.rs,
collapse.rs, mod.rs}`; tables and maps by `tools/review/continent-budget.py`.

**The budget (measurement only, PLT-302).** `DEEPTIME_CRUST_BUDGET=1` keeps the continental
crust's stocks every 50 Myr (area, volume visible and hidden, land at the climate's sea level, the
dry share of the continental crust, the land above 2 km, the median distance from land to the
coast, the thickness bands with their elevation and Airy height), books each stock's change to the
process of the plate step that made it (the gap budget's brackets, PLT-283) and counts single
samples changing state (ocean floor made continental, hidden, drowned, emerged).
`DEEPTIME_CRUST_BUDGET_MAPS=<dir>` also writes the visible samples at every snapshot. On master
(c88c2eb5 plus the budget, which changes nothing: test `the_crust_budget_changes_nothing`),
quick 16k, 1000 Myr, seeds 1 to 3 (scattered: seeds 1 and 2):

| | supercontinent, start | supercontinent, end (3 seeds) | scattered, end (2 seeds) | Earth |
|---|---|---|---|---|
| continental crust, share of the planet | 0.422 | 0.417 (0.387 to 0.473) | 0.463 | 0.41 (Cogley 1984, HYP-012) |
| its volume, 10^9 km³ | 7.56 | 8.69 | 9.62 | |
| land at sea level | 0.300 | 0.178 | 0.160 | 0.289 (HYP-001) |
| dry share of the continental crust | 0.71 | 0.39 | 0.30 | 0.71 (HYP-012) |
| land above 2 km, share of land | 0 | 0.25 | 0.23 | 0.102 (HYP-004) |
| median distance from land to the coast, km | 2,220 | 169 | 155 | |
| mean continental thickness, km | 35.1 | 41.0 | 40.8 | 41.1 (Christensen & Mooney 1995) |

The continents do not lose their area: it holds near the start's 0.42 and the scattered start
gains. They lose their land. Half of the continental crust that was dry is under the sea at the end,
the land is a sixth of the planet, and the median coast is 150 to 190 km from any land, an
archipelago. The thickness spreads both ways at once (supercontinent seed 2 at the end): 30 % of
the continental crust is thinner than 35 km, 41 % thicker than 45 km, 15 % 55 to 70 km, and every
band stands 1.1 to 1.9 km below its Airy height, so even crust of a normal thickness is drowned:

| band, km | share of the continental area | median elevation, m | Airy height, m | dry |
|---|---|---|---|---|
| 30 to 35 | 0.127 | -1,079 | 178 | 0.18 |
| 35 to 40 | 0.166 | -349 | 868 | 0.33 |
| 40 to 45 | 0.170 | -132 | 1,581 | 0.42 |
| 45 to 55 | 0.230 | 244 | 2,520 | 0.64 |
| 55 to 70 | 0.123 | 1,814 | 4,271 | 0.90 |

Where the land goes, over the run (mean of the three supercontinent runs, as shares of the
planet; a stock's parts add up): the belts raise 0.53 of land and the arcs 0.38, the collapse
0.13; subduction drowns 0.52 (the plate going under), the ocean-floor step 0.39 (arcs that are
still ocean floor), the transition 0.38 and the surface's erosion 0.20. Of the dry continental
crust, the transition takes 0.46, the surface 0.47 and subduction 0.49. The area turns over: the
plates going under hide 0.85 of the planet's worth of continental crust in 1000 Myr and the
resamplings take 0.84 of it back, while arcs that mature turn 0.58 of ocean floor continental
(`finish_samples`) at a mean of -3,219 m and 19.6 km of crust, 1.5 km under that crust's own Airy
height and 0.8 km under its water-loaded one. The collisions turn area into thickness at
17.5 km³ a year (the crust the belts keep), the collapse moves 4.1 km³ a year of it back out.

**What Earth does.** Continental crust is 41 % of the surface and 71 % of it is dry (Cogley 1984,
HYP-012). Earth's freeboard holds because a column out of isostatic balance does not stay so: the
mantle under Fennoscandia gives back an ice load's depression with relaxation times "close to
3-5 kyr" (Wieczerkowski, Mitrovica & Wolf 1999, abstract), and what a cooling lithosphere adds is
spent on its thermal time, 62.8 Myr (McKenzie 1978, full text, Table 1). Thickened crust does not
stay thick either: its hot lower crust flows out from under the high ground, "Over 15 m.y., the
length scale of channel-flow extrusion is 150 km rather than 1500 km" (Rey, Teyssier & Whitney
2010, full text), the eastern margin of Tibet was built by lower-crustal flow (Clark & Royden
2000), and the Basin and Range "cannot maintain short-wavelength (100 km) Moho relief of more
than 1 km for times greater than 10-20 m.y. at most" (Bird 1991, abstract). Crust of every
thickness ends near its isostatic height and spreads when it is thick and hot.

**The cause.** Two things in the plate model, neither of them the mountains' height:

1. *Nothing returns a column to its isostatic height.* Every process moves a column's surface by
   the change it makes (`collapse::toward`, collapse.rs:205 on master; the belts', arcs' and
   transition's own height lines), and none moves it towards the height its crust holds up. A gap
   once made stays. The largest maker is the arcs: island-arc crust becomes continental "whatever
   its height" (`finish_samples`, mod.rs:1076 on master) and keeps the ocean floor's height as it
   thickens, so the continents a run grows stand on the ocean's line. Accretion, the ocean-floor
   step, the transition's height line (f = 4.24, steeper than Airy's 7.02) and the surface's
   flexure add theirs (the gap budget, "Which process holds the gap" above).
2. *Thickened crust inside a continent keeps its root.* The collapse releases only quiet crust
   (collapse.rs:241 to 252, PLT-259) and only into room thinner than the normal column within
   200 km (collapse.rs:328 to 351, PLT-257): a belt ringed by crust of normal thickness has no
   room, and of the about 33 km³ a year its rule would release, 4.1 moves.

The transition's width (PLT-262) is Ben's (decisions log 2026-10-05, "Keep 200 km, raise crust in
7–8") and is not changed here (he moved it to the model sphere on 2026-10-22, decisions log D6;
"The transition's width and the drowned continents" at the end of this page).

**The fix.** Two rules on the plate model's samples, each with its own parameter and its own
ledger row, run each plate step after the collapse and before the rate limits:

1. *The lower-crustal flow* (`plates/flow.rs`, PLT-300, PLT-301): continental crust thicker than
   `collapse_thickness_km` (50 km, PLT-256) spreads into the crust around it on its plate,
   orogeny or not, by diffusion of thickness, ∂T/∂t = ∇·(K∇T) with K = K₀ (h/15 km)³ and h the
   channel's thickness above 50 km (at most 30 km). It is the thin-channel flux in the isostatic
   limit (Bird 1991; Clark & Royden 2000). K₀ = `crust_channel_km2_myr` = 1,500 km²/Myr, the
   diffusivity whose length over 15 Myr, √(K t), is Rey, Teyssier & Whitney's 150 km. It moves
   crust, not area: the test `a_thick_belt_spreads_into_the_crust_around_it_and_keeps_its_volume`
   holds the volume to a millionth.
2. *The settling* (`plates/settle.rs`, ISO-078): every visible continental column moves to the
   height its crust holds up, z* = z_A above the sea and s + (z_A − s) · 1.454 below it (its
   water on it), plus the surface's flexure. Its time, `isostasy_relax_myr`, is the mantle's:
   4 kyr (Wieczerkowski, Mitrovica & Wolf 1999, abstract), so a 2 Myr step settles the whole way.
   The model's thermal heights (the ocean floor's age-depth law, the margins' transition on
   McKenzie's 62.8 Myr) have their own rules; the gaps the plate rules leave are mechanical.

The flow alone does not bring the land back, and the settling on the thermal time does not either
(`earthlike_quick_16k`, supercontinent seed 2, 1000 Myr, the end):

| run | land | dry share of the continental crust | land above 2 km, of land | coast distance p50, km | continental crust |
|---|---|---|---|---|---|
| master (c88c2eb5 with the budget) | 0.192 | 0.45 | 0.336 | 166 | 0.392 |
| flow only | 0.200 | 0.44 | 0.233 | 189 | 0.411 |
| settling on 62.8 Myr only | 0.244 | 0.56 | 0.206 | 151 | 0.422 |
| flow, settling on 62.8 Myr | 0.250 | 0.61 | 0.294 | 175 | 0.399 |
| flow, settling on 10 Myr | 0.274 | 0.71 | 0.448 | 174 | 0.347 |
| flow, settling on 0.5 Myr | 0.338 | 0.68 | 0.301 | 152 | 0.433 |
| Earth | 0.289 | 0.71 | 0.102 | 433 | 0.41 |

One seed each, so the differences between the fast settlings are within a run's chaos; every
settling of 10 Myr or less brings every thickness band to its Airy height. The flow takes the
thickest crust down (the 55-70 km band's share) and the land above 2 km with it when the settling
is off; with the settling on, the land above 2 km is set by the thickness distribution under the
model's Airy line (z_0 = 525 m at 35.1 km, 1 km per 7.02 km of crust): 45 km of crust stands at
1.9 km and 50 km at 2.6 km. At the end of a run with the fix, 0.20 (supercontinent seed 1) and
0.28 (scattered seed 1) of the continental crust is thicker than 45 km, 0.41 on master
(supercontinent seed 2, the table above), where Earth's mean is 41.1 km and its orogens 46.3 km
(Christensen & Mooney 1995).

**The result.** Master (c88c2eb5 with the budget) against the fix, `earthlike_quick_16k`,
1000 Myr, seeds 1 to 3, the end of the run, means of the three seeds; the full row is
`earthlike_16k`, scattered seed 1, one run (`tools/review/force-balance-ensemble.sh` with
`DEEPTIME_CRUST_BUDGET=1`, tables and figures by `tools/review/continent-budget.py`):

| start | run | land | continental crust under the sea | land above 2 km, of land | coast p50, km | continental crust | mean thickness, km |
|---|---|---|---|---|---|---|---|
| supercontinent | master | 0.178 | 0.61 | 0.251 | 169 | 0.417 | 41.0 |
| supercontinent | fix | 0.310 | 0.34 | 0.285 | 166 | 0.432 | 37.5 |
| scattered | master | 0.155 | 0.68 | 0.282 | 162 | 0.426 | 41.0 |
| scattered | fix | 0.305 | 0.33 | 0.354 | 148 | 0.408 | 39.0 |
| scattered, full 16k | master | 0.221 | 0.46 | 0.400 | 163 | 0.386 | 42.8 |
| scattered, full 16k | fix | 0.317 | 0.35 | 0.350 | 129 | 0.442 | 39.5 |
| Earth | | 0.289 | 0.29 | 0.102 | 433 (409 at 250,000 points) | 0.41 | 41.1 |

Halfway (500 Myr ago) the fix holds 0.28 and 0.31 of the planet as land where master has 0.19 and
0.16, and the dry share of the continental crust stays at 0.59 to 0.67 through the whole run
(master falls to 0.32 to 0.42). The continental area holds on both (0.39 to 0.48 at the end). The
budget's parts (mean of the six quick runs): the settling raises 2.09 of the planet's worth of
land over the run and the surface's flexure and erosion take back what is above the line (the
plate surface -1.24, erosion -0.92 where master's are -0.40 and -0.25); the flow moves
10.2 km³ a year, the collapse's own release falls from 4.3 to 2.0 km³ a year (likely because the
flow has taken the thick crust first), and the foundering halves (2.1 to 1.1 km³ a year).

*The climate.* More land weathers more: the end's global temperature is lower on six of the seven
pairs, by 2.2 to 4.3 °C (supercontinent seed 3 is 1.5 °C warmer), and the natural CO2 is lower on
five (full 16k: 1,461 to 810 ppm, 19.3 to 15.0 °C). The cold land grows: the Köppen E share of
the land rises from 0.23 to 0.33 (supercontinent) and from 0.28 to 0.40 (scattered), and with
nearly twice the land the area of E climate is 2.5 to 2.8 times master's on the quick runs; on the full
run it falls from 0.41 to 0.33.

*The sea floor.* The two rules touch only visible continental crust, never the ocean floor, but the
sea floor changes through it: the drowned continental crust stands at its water-loaded Airy height,
so its median depth goes from 1.2 to 1.9 km to 0.4 to 0.9 km (500 Myr ago and at the end; the
shelves' share of the sea goes
from 0.05 to 0.09 to 0.07 to 0.16), and the shallowest tenth of the ocean floor proper is
shallower (its p90 -0.5 to -0.9 km against -1.3 to -1.9 km on master, the end of seed 1 of each
start and the full run), likely through the resampling's blend at the margins (not traced). The
median sea depth falls by about 0.3 km. The relief between neighbouring sea samples (the p90 of
the height difference within 1.5 spacings) is 721, 1,084 and 722 m with the fix against 877, 706
and 852 m on master: no consistent change, and the drowned continental crust is smoother (p90 444
to 546 m against 554 to 717 m).

**What it does not fix: the shape.** The continents keep their land and their freeboard, but not
their shape. The median distance from land to the coast stays at 130 to 170 km on both, against
Earth's 433 km (HYP-035), and the land at the end is still the belts and their forelands with
island strings between them (the maps the budget writes, `maps-*.png`). The pieces of land holding 90 % of it at the end:
6, 6 and 10 with the fix, 14, 13 and 1 on master (seed 1 of each start, the full run). Three leads,
not yet measured one by one: the collisions turn area into thickness (the belts keep about
18 km³ a year, so a continent shrinks where it meets another), the arcs build continental crust
along thin strings (4.1 km³ a year of island-arc crust, where Earth's 14,500 km of
intra-oceanic arcs grow about 1.5 km³ a year, Stern & Scholl 2010, PLT-045), and the resampling fills the gaps inside a continent with ocean
floor (it turns 15.9 of the planet's worth of continental samples to ocean floor and 14.7 back
over a run). The land above 2 km is also still too much (0.29 to 0.35 of land, Earth 0.10): the
thickness distribution is wider than Earth's, and the model's Airy line puts 41 km of crust at
1.4 km.

The figures are not kept in the repo: `tools/review/continent-budget.py --out <dir> --group
master=<summaries> --group fix=<summaries> --maps <name>=<maps dir>` makes them from the runs'
summaries and maps (`series.png` the stocks over time per seed, `bands-*.png` the thickness bands,
`maps-*.png` six snapshots of elevation and of the continental crust); the build log entry of
2026-10-20 has the commands.

## Land shape and the high ground (2026-10-20)

The coordinator's task after "Mountains eat the world" above: with the land right (0.29 to 0.31 of
the planet), land above 2 km is 0.29 to 0.35 of the land (Earth 0.10, HYP-004) and the median
distance from land to the coast 130 to 170 km (Earth 433, HYP-035). Each lead's share by paired
ablation, the causes fixed at the source with sourced rates where that needs no choice of Ben's,
and the Köppen E share and natural CO₂ beside, the climate untouched. Ledger PLT-310 to PLT-312,
HYP-040 and HYP-041, with PLT-221, PLT-300 and PLT-301 changed. Code: `plates/{shortening.rs,
dynamics.rs, subduction.rs, flow.rs, crust_budget.rs}`. Tables: `tools/review/land-shape.py` (each
run's crust budget and climate), `tools/review/land-maps.py` (the budget's maps: land pieces, the
drowned crust, the high ground by its last orogeny) and `tools/review/earth-crust.py` (Earth's
crust from CRUST1.0); the build log entry of 2026-10-20 "Land shape" has the commands.

**What Earth looks like** (CRUST1.0, Laske et al. 2013, on a 1-degree grid with its own surface
heights; HYP-040 and HYP-041, the model file read, the paper not). Earth's land stands on crust of
32.0 / 38.6 / 45.8 km (p10 / p50 / p90); crust 45 km or thicker carries 0.12 of the land (45 to
55 km 0.097, 55 to 70 km 0.019, 70 km and more 0.006), and 35 to 45 km 0.64 of it. Land above 2 km
is 0.104 of the land by its surface, which counts the ice sheets' tops (HYP-004's 0.102), 0.047 by
the rock under the ice and 0.052 of the ice-free land. Pooled over the ice-free land, rock height
rises only 81 m per km of crust (−2,559 m + 81 m/km × T); land on 45 to 55 km of crust stands at
940 m (median), where the model's Airy line (143 m/km, 525 m at 35.1 km) puts 45.5 km at 2 km.
The pooled slope is not Earth's isostasy: inside one thermal regime Hyndman & Currie (2011, full
text) measure 131 ± 12 m/km (the Cordillera) and 141 ± 27 m/km (stable areas), the model's slope,
with sea-level crust about 27 km thick in hot back-arcs and about 38 km under cold cratons. Earth's
thick crust is mostly old, cold shield standing low; the pooled line mixes the two regimes.

**What the model makes** (master 5e0615b9 with the budget, `earthlike_quick_16k`, 1000 Myr, seeds
1 to 3; supercontinent / scattered, mean of the seeds; ranges are over the six runs):

| | supercontinent | scattered | Earth |
|---|---|---|---|
| land, share of the planet | 0.310 | 0.305 | 0.289 |
| land above 2 km, share of the land | 0.285 | 0.354 | 0.104 by surface, 0.047 by rock |
| median distance from land to the coast, km | 166 | 148 | 433 |
| continental crust under the sea, share of it | 0.34 | 0.33 | 0.19 (crust 25 km and thicker) |
| the land's crust, p50 / p75 / p90, km | 37.0 to 41.7 / 43.5 to 52.8 / 52.2 to 63.3 | 38.7 to 41.9 / 50.5 to 52.4 / 62.9 to 64.9 | 38.6 / 42.0 / 45.8 |
| Köppen E, share of the land | 0.33 | 0.40 | |
| natural CO₂ (before the present's nudge), ppm | 703 | 864 | |

*The high ground is young orogen.* By the column's last orogeny, 0.54 to 0.72 of the land above
2 km is Himalayan (a collision) and 0.25 to 0.43 Andean; under 0.05 has none. The last orogeny is
0 to 14 Myr old at the median and under a fifth of it is older than 100 Myr; the crust under it is
50 to 54 km (Andean) and 55 to 60 km (Himalayan) at the median. So a lithosphere temperature per
column, which would sink old orogens by up to Hyndman & Currie's 1,600 m and is what the Urals
need (section "The eclogite root" above), would lower at most that old fifth: it is not what makes
the high ground. Earth's young orogens are as high as the model's; the model has too many of them.

*The coast is a drowned margin.* A third of the continental crust is under the sea (Earth 0.19).
It is margin: its median sample is 71 to 85 km from the ocean floor and all of it lies within
800 km, on 28 to 30 km of crust, below the Airy line's sea-level crossing (31.4 km). The
continental crust itself holds together (1 to 4 pieces hold 90 % of it, the largest 0.56 to 0.92
of it); the land on it is in 1 to 8 pieces holding 90 %. The land's median crust is Earth's, its
upper quartile and tail are not: Earth's land is 0.64 on 35 to 45 km of crust, the model's middle
is thin and its tail is orogen.

**The leads, one at a time** (the same seeds with one process off; supercontinent / scattered):

| run | land | above 2 km | coast p50, km | Köppen E | natural CO₂, ppm |
|---|---|---|---|---|---|
| base | 0.310 / 0.305 | 0.285 / 0.354 | 166 / 148 | 0.33 / 0.40 | 703 / 864 |
| no island arcs (`island_arc_km3_per_km_myr` 0) | 0.154 / 0.129 | 0.398 / 0.366 | 187 / 155 | 0.22 / 0.28 | 1,398 / 1,710 |
| no collision stacking (`collision_stacking_share` 0) | 0.161 / 0.134 | 0.155 / 0.200 | 101 / 97 | 0.23 / 0.26 | 615 / 718 |
| no cordillera, no continental arcs | 0.289 / 0.260 | 0.306 / 0.326 | 190 / 146 | 0.33 / 0.35 | 616 / 995 |
| margin taper 24.5 km instead of 200 (`transition_width_km`; W then on the planet, so 24.5 km is 200 km on the model sphere, the shipped width since 2026-10-22) | 0.323 / 0.315 | 0.346 / 0.329 | 155 / 149 | 0.39 / 0.33 | 726 / 917 |

By the origin of the crust under it (the budget's tag), the land above 2 km is 0.49 to 0.51
island-arc crust, 0.27 to 0.33 collision crust, 0.16 to 0.21 Andean and 0.02 other; the land is
0.53 to 0.55 island-arc crust. Every process builds land with a quarter to a half of it above
2 km, so taking one away takes land with it and leaves the share of high ground about where it
was. The one exception is the collisions: without stacking the land above 2 km falls to 0.16 to
0.20 and the crust 55 km and thicker from 0.105 to 0.016 of the continental crust (supercontinent), but the land
halves (the crust a collision does not stack is lost) and the coast comes nearer (100 km). The
margin taper and the cordillera move the land little. The lower-crustal flow plays no part: its
bound never bound and its flux at plate edges is negligible (P4 notes in "The 4-Myr plate step").
The resampling does not break the continents apart (above); it was not ablated.

**The subducting boundary** (the known issue, five times Earth's). Half of it was the
measurement. Each boundary sample judged its motion towards the other plate along its own lattice
edges, which on the Fibonacci lattice lie about 30 degrees either side of the boundary, so
strike-slip read as convergence on one edge and divergence on the next: 3.1 % of the boundary was
transform or converging without contact, against Bird's 28.4 %. The classes now judge the motion
across the boundary's normal over two rings of samples (PLT-310) with Bird's own 20 degrees
(PLT-311, read in the paper):

| share of the boundary, mean over the steps after the first 100 Myr | before (lattice edges) | after (normal, 20 degrees) | Earth (Bird 2003) |
|---|---|---|---|
| subducting (ocean-ocean and ocean-continent) | 46.4 % | 31.1 % | 19.7 % (target 15 to 25) |
| spreading (ridge and rift) | 40.0 % | 35.6 % | 36.4 % |
| collision | 10.5 % | 8.4 % | 8.8 % |
| transform, or converging with no contact | 3.1 % | 24.9 % | 28.4 % + 6.7 % |
| subducting in the first 50 Myr | 32.1 % | 25.3 % | |
| subducting boundary, km a step | 243,926 | 144,340 | 51,310 |
| total boundary, km a step | 522,534 | 462,627 | 260,487 |
| plate checks met (pooled) | 27 of 45 | 27 of 45 | |
| full 16k, seed 1, subducting (supercontinent / scattered) | 48.3 % / 49.8 % | 32.1 % / 32.8 % | |
| full 16k, plate checks met | 25 / 27 of 45 | 25 / 27 of 45 | |

(Quick 16k, seeds 1 to 3, both starts pooled; the full 16k is `earthlike_full_16k` at 8 threads.)
The subducting share still fails its target, at 1.6 times Earth's share and 2.8 times its length.

The classes are measurement only (nothing a plate rule reads), so the planets are the same bit for
bit: in all six quick runs and both full runs the crust budget's last line (every crust volume and
area to the km³) and every number of the land-shape table equal the runs before the change. The total boundary is about twice Earth's in km (463,000 to
523,000 against 260,487), a lattice boundary being rougher than a surveyed one, so lengths compare
as shares.

The other half is physical. The arcs, the cordillera and the subduction uplift took the full
relative speed as their convergence on every front, oblique or strike-slip; along the boundary's
normal (PLT-312, the switch `DEEPTIME_ARC_NORMAL=1`) the fronts' convergence factor is 0.54 of it.
Measured on the same seeds (supercontinent / scattered):

| | base | arcs on the normal convergence | Earth |
|---|---|---|---|
| island arcs, km³ a year | 3.95 / 4.17 | 2.47 / 2.48 | 1.5 (Stern & Scholl 2010) |
| continental arcs, km³ a year | 1.43 / 1.56 | 0.87 / 0.95 | 1.0 |
| cordillera shortened, km³ a year | 3.27 / 3.45 | 1.99 / 2.20 | |
| continental crust, share of the planet | 0.43 / 0.41 | 0.29 / 0.31 | 0.41 (HYP-012) |
| land | 0.310 / 0.305 | 0.233 / 0.245 | 0.289 |
| land above 2 km | 0.285 / 0.354 | 0.384 / 0.454 | 0.104 / 0.047 |
| coast p50, km | 166 / 148 | 167 / 188 | 433 |
| Köppen E | 0.33 / 0.40 | 0.36 / 0.36 | |
| natural CO₂, ppm | 703 / 864 | 1,282 / 1,493 | |
| plate checks met (pooled) | 27 of 45 | 30 of 45 | |

The arcs come near Earth's rates and the plate checks improve (the plates speed up from 31.8 to
39.5 mm/yr RMS, the old ocean floor passes; collisions fall below 2 per 100 Myr and the lowest
continental share below 0.37), but the continents lose a third of their area and the low
island-arc land goes first, so the high ground's share rises. The model's continents are held up by
arcs at 2.7 times Earth's rate, which balance losses above Earth's. So PLT-312 stays off until the
continental crust's losses are on Earth's numbers too (a loss budget against Stern & Scholl's
Table 1, which the crust budget's bookings can give); landed alone it moves the land away from its
target.

The full 16k pair (`earthlike_full_16k`, seed 1, 8 threads; the switch off / on) says the same:

| | supercontinent | scattered |
|---|---|---|
| land | 0.332 / 0.280 | 0.317 / 0.202 |
| land above 2 km | 0.304 / 0.407 | 0.350 / 0.484 |
| coast p50, km | 137 / 193 | 129 / 142 |
| continental crust | 0.457 / 0.349 | 0.442 / 0.261 |
| island arcs, km³ a year | 4.46 / 1.64 | 4.75 / 2.13 |
| Köppen E | 0.34 / 0.30 | 0.33 / 0.34 |
| natural CO₂, ppm | 375 / 1,112 | 810 / 2,075 |
| plate checks met | 25 / 29 of 45 | 27 / 30 of 45 |

**On the ocean crust branch's planets** (its rock pools kept within one crust kind, PLT-293;
measured on this branch with that branch's head 45de0c9a merged, the same seeds; supercontinent /
scattered): land 0.322 / 0.326, land above 2 km 0.335 / 0.306, coast p50 173 / 161 km, Köppen E
0.34 / 0.32, natural CO₂ 764 / 731 ppm. With its transition switch on (PLT-294): land 0.274 /
0.270, above 2 km 0.328 / 0.356, coast 149 / 168 km. With PLT-312 on: land 0.252 / 0.238, the
continental crust 0.317 / 0.296 of the planet, above 2 km 0.343 / 0.352, island arcs 2.07 / 2.20
km³ a year, plate checks 33 of 45 against 26. The high ground and the coast are where they were.

**What is left, and for whom.**
1. *The extent of young orogen* is the high ground. Its sources: subduction on about half the
   boundary (Earth's 20 %), the active share of the continental margin (0.47 against 0.25 to 0.40)
   and collisions that last too long (90th percentile 146 Myr against at most 120), each a plate
   check that fails. The cordillera shortens every active continental margin at Bolivia's 6 mm a
   year (PLT-048, the Andes' most shortened part) up to 3,000 m; most of Earth's continental arcs
   shorten far less. Bringing these to Earth's is plate work on the force balance and the
   cordillera's source, with PLT-312 and the loss budget.
2. *The height of old thick crust* (the Urals, cold cratons): a lithosphere temperature per column.
   It is the right physics and Earth's pooled 81 m/km shows it, but it moves only the old fifth of
   the high ground; it is an architecture change the coordinator deferred on 2026-10-17, and the
   numbers here say it can keep waiting behind item 1.
3. *What the target counts.* The model's land above 2 km is the rock's height; HYP-004's 0.10
   counts the ice sheets' tops. By rock Earth's figure is 0.047 (HYP-041). The model's present-day
   comparisons of high ground should use 0.05 by rock (or 0.10 counting the model's own ice on
   top).

## The crust budget against Stern & Scholl (2026-10-20)

The coordinator's task after "Land shape and the high ground" above: book the continental crust's
gains and losses against Stern & Scholl 2010's Table 1 per Myr, find the one off Earth and fix it
at its law; land PLT-312 only if land, high ground, coast and the plate checks all move toward
Earth or hold; source the cordillera for a typical arc instead of Bolivia; shorten the collisions
(p50 15 to 50 Myr, p90 at most 120) at their law. Ledger PLT-048 and PLT-320, HNT-127, HYP-018,
PRS-039; sources in the bibliography, "The cordillera at Earth's typical continental arc". Code:
`plates/subduction.rs` (the cordillera). Tables: `tools/review/loss-budget.py` (Stern & Scholl's
rows from each run's crust budget, `DEEPTIME_CRUST_BUDGET=1`), `tools/review/collision-ends.py`
(how the check's collisions end) and `tools/review/land-shape.py`; the build log entry of
2026-10-20 "Crust budget against Stern & Scholl" has the commands.

**Earth's budget** (Stern & Scholl 2010, Int. Geol. Rev. 52:1-31, read in full; Table 1, p. 12;
km³ a year). Additions: intra-oceanic arcs 1.5 (100 km³ per km and Myr over 14,500 km),
continental arcs 1.0 (35 over 27,500 km), crust-suturing zones 0.2, rifts, hotspots and volcanic
rifted margins 0.5; 3.2 in all. Losses: at ocean margins sediment subduction 1.1 and subduction
erosion 1.4 (about 60 km³ per km and Myr over 43,000 km of margin), at suturing zones 0.7
(sediment 0.1, erosion 0.3, deep subduction of continental crust 0.3), foundering not known; at
least 3.2 in all, so the losses at least match the additions.

**The model's budget** (`earthlike_quick_16k`, 1000 Myr, seeds 1 to 3, supercontinent and
scattered continents, the mean of the six runs over the whole run, km³ a year; master c24ff793
against this branch, both with the transition's ocean crust switch PLT-294 off):

| Stern & Scholl's row | master | branch | Earth |
|---|---|---|---|
| intra-oceanic (island) arcs | 4.12 | 4.20 | 1.5 |
| continental arcs | 1.62 | 1.56 | 1.0 |
| crust-suturing zones (additions) | no rule | no rule | 0.2 |
| rifts, hotspots, rifted margins | not booked | 0.04 | 0.5 |
| model only: the ocean floor under a matured island arc made continental | 4.62 | 4.67 | |
| model only: pelagic sediment, counted felsic | not booked | 1.40 | |
| ocean margins: sediment subduction and subduction erosion | no rule | no rule | 2.5 |
| crust-suturing zones (losses): crust crossing a collision front and not stacked | 2.57 | 2.65 | 0.7 |
| foundering (crust beyond `crust_max_km`) | 1.24 | 1.23 | not known |
| model only: crust over the thickening ceiling sent down | 0.65 | 0.92 | |
| model only: crust a resampling drops | 0.76 | 0.89 | |
| net change of the continental volume | +0.98 | -0.12 | about 0 |
| island-arc front, lattice km | 134,790 | 142,395 | 14,500 |
| continental-arc front, lattice km | 96,339 | 99,288 | 27,500 |
| moved, not made: shortened into cordilleras | 3.59 | 1.66 | |

("Not booked": master's binary does not write those two lines; this branch's budget stocks, commit
3dd2e2e0, add them.) The rows do not sum to the net: the surface stage moves 18.3 km³ a year of
sediment off the continents and the resamplings accrete 21.4 of ocean-floor felsic rock back, flows
that the budget books as moved.

**What is off Earth, and at which law.**
1. *Island arcs, 2.8 times Earth's.* Per lattice km of front they add 29.5 km³ a Myr, below
   Stern & Scholl's 100 (the model's rate is 100 times g, and the mean g of its oceanic fronts is
   0.59); the front is 9.8 times Earth's in lattice km, about 5 times as a share of the boundary (a
   lattice boundary is about twice as long as a surveyed one: "Land shape and the high ground").
   The gain is the length of ocean-ocean subduction: the subducting share of the boundary is
   31.6 % (target 15 to 25), 54 % of it ocean-ocean. Its law is the plate geometry, fewer and
   longer-lived subduction zones from the force balance: plate work beyond this task. PLT-312 cuts
   the rate per km instead (below).
2. *The arc's ocean floor made continental, 4.67 with no row in Table 1.* When an island arc
   matures (PLT-047) its whole column turns continental, the ocean crust under the arc included;
   Stern & Scholl's 1.5 counts the arc's own magma. It doubles what the island arcs add. A column
   holds one crust kind, so counting only the arc's magma as continental is a change to the crust's
   data model: left for Ben.
3. *Collisions lose 2.65, 3.8 times Earth's 0.7.* The crust crossing collision fronts is 22.3 km³ a
   year and 0.12 of it is not stacked. The collision boundary is Earth's share (8.5 % against
   Bird's 8.8 %); the collisions last long (p90 125 Myr) and shorten far (275 km at the median).
   The slab tear below tried the collisions' end.
4. *Ocean margins lose nothing by rule* (Earth 2.5). The model's sinks (foundering 1.23, the rate
   cap 0.92, the resampling's drops 0.89, together 3.0) stand in for them at about Earth's total,
   but inside the continents, not at their margins.
5. *Hotspots and rifts add 0.04, twelve times below Earth's 0.5*: flood basalts and volcanic rifted
   margins have no rule of their own.
6. *Pelagic sediment counted felsic, 1.40*: the plate step's pelagic rain (`pelagic_m_per_myr`)
   books as felsic rock on the ocean floor and reaches the continents at resamplings.

Table 1's rows add about 12 km³ a year on the branch against Earth's 3.2 and take 5.7 against at
least 3.2, but the continental volume moves far more by the budget's own parts: the surface stage
takes 18.3 km³ a year off as sediment, the resampling's blend 14.8 and the transition 2.7, while
the collision belts add 19.7 (crust stacked onto the continents, the sediment and arc crust on the
ocean floor with it), the subduction step 6.5 and the step's finish 6.2. The net holds at -0.12
(-1.43 to +1.09 over the six runs). Bringing one of Table 1's rows to Earth's alone moves that
balance: PLT-312 below brings the arcs to Earth's and the crust shrinks.

**The cordillera at Earth's typical arc** (PLT-048, PLT-320). The cordillera shortened every
continental arc at Bolivia's 6 mm a year (Eichelberger et al. 2015), the Andes' most shortened
part, scaled by g = min(v / v0, 1). Heuret & Lallemand (2005, full text) find that the upper
plate's own motion over the mantle, not the convergence, controls back-arc deformation ("upper
plate absolute motion appears to exert a dominant control on back-arc deformation"; Vup "the best
single predictor of strain regime", after Jarrard 1986). Their Table 1 gives 88 two-degree segments
of continental upper plates; 21 of them are class C3, back-arc thrusting, at a mean measured rate of
7.1 mm/yr, the rest shorten little or extend: 1.7 mm/yr on average (21 x 7.1 / 88). The cordillera
now shortens at 1.7 mm/yr at every continental arc above an oceanic slab, with no convergence
factor; the per-front law (about 0.09 of max(Vup, 0), the mean of which over the 88 segments is
18.1 mm/yr) waits for the force balance's plate motions to reach the arcs. Measured (supercontinent
/ scattered; master c24ff793 against the branch):

| | master | branch | Earth |
|---|---|---|---|
| shortened into the ranges, km³ a year | 3.40 / 3.77 | 1.62 / 1.71 | |
| land | 0.322 / 0.326 | 0.284 / 0.256 | 0.29 |
| land above 2 km | 0.335 / 0.306 | 0.320 / 0.359 | 0.05 by rock, 0.10 by surface |
| coast p50, km | 173 / 161 | 149 / 158 | 433 |
| continental crust, share of the planet | 0.434 / 0.429 | 0.418 / 0.349 | 0.41 |
| collision length p50 / p90, Myr (pooled check) | 51 / 134 | 43 / 125 | 15 to 50 / at most 120 |
| collisions per 100 Myr | 1.90 | 2.17 | 2 to 6 |
| plate checks met (pooled) | 26 of 45 | 27 of 45 | |

The collisions shorten (the median passes, and so does their number); the plate checks gain those
two and lose one (the speed of slab-edged plates over slab-free ones, 2.51 to 2.38 against at least
2.5). The land falls to Earth's on the supercontinent and below it on the scattered start, the coast
comes nearer, the high ground holds. On master 9d6758bb, before the ocean crust merge, the same
change moved land 0.310 / 0.305 to 0.319 / 0.298, land above 2 km 0.285 / 0.354 to 0.385 / 0.347,
the coast 166 / 148 to 173 / 159 km and the plate checks 27 to 29: within the seeds' spread either
way.

**PLT-312, the arcs on the normal convergence, again** (`DEEPTIME_ARC_NORMAL=1`; the branch off
against on, supercontinent / scattered):

| | off | on | Earth |
|---|---|---|---|
| island arcs, km³ a year | 4.20 | 2.19 | 1.5 |
| continental arcs, km³ a year | 1.56 | 0.80 | 1.0 |
| collisions lose, km³ a year | 2.65 | 2.98 | 0.7 |
| net change of the continental volume, km³ a year | -0.12 | -1.52 | about 0 |
| land | 0.284 / 0.256 | 0.231 / 0.233 | 0.29 |
| land above 2 km | 0.320 / 0.359 | 0.418 / 0.427 | 0.05 by rock |
| coast p50, km | 149 / 158 | 189 / 183 | 433 |
| continental crust | 0.418 / 0.349 | 0.282 / 0.288 | 0.41 |
| plate checks met | 27 of 45 | 30 of 45 | |

The arcs come near Earth's and the plate checks and coast improve, but with the collisions still
losing four times Earth's the crust shrinks, the land falls below Earth's and the high ground's
share rises. Two of the four measures move away from Earth, so PLT-312 stays off (decisions log
2026-10-20). It is half of a pair: it can land with a collision loss at Earth's.

**The ocean crust switch, PLT-294** (`tectonics.transition_ocean_crust`; the ocean-basin felsic
share is the share of `ocean_basin` cells (the regime field) on oceanic crust holding more than
50 m of felsic plutonic rock, `tools/review/geo_template_fields.py` on the final checkpoint, the
six runs' range):

| | master off | master on | branch off | branch on | Earth |
|---|---|---|---|---|---|
| land | 0.322 / 0.326 | 0.274 / 0.270 | 0.284 / 0.256 | 0.271 / 0.272 | 0.29 |
| land above 2 km | 0.335 / 0.306 | 0.328 / 0.356 | 0.320 / 0.359 | 0.348 / 0.297 | 0.05 by rock |
| coast p50, km | 173 / 161 | 149 / 168 | 149 / 158 | 149 / 145 | 433 |
| ocean-basin felsic share | 0.75 (0.72 to 0.78) | 0.45 (0.40 to 0.48) | 0.76 (0.61 to 0.91) | 0.46 (0.34 to 0.57) | near 0 |
| collision length p50 / p90, Myr | 51 / 134 | 43 / 154 | 43 / 125 | 38 / 121 | 15 to 50 / at most 120 |
| collisions lose, km³ a year | 2.57 | 1.96 | 2.65 | 1.85 | 0.7 |
| plate checks met | 26 of 45 | 25 of 45 | 27 of 45 | 28 of 45 | |

On the branch the switch takes the felsic rock off 0.30 of the ocean basins and moves the land to
0.27, the high ground and coast within the seeds' spread, and the plate checks hold (27 to 28). It
was the coordinator's call (decisions log 2026-10-20, "off until the high-ground fix lands"), and
on these numbers it went on in every shipped preset (`earthlike_*` and `pipeline_*`; decisions
log 2026-10-20, the second PLT-294 entry; the plate pipeline's `plates_*` keep the code's default): the land holds near Earth's 0.29 on both starts
and the ocean basins' felsic share falls from 0.76 to 0.46. The high ground does not move one way
(better on the quick ensemble, worse on the single full run, seed 1: land above 2 km 0.299 / 0.314
off, 0.380 / 0.455 on) and belongs to the high-ground work.

**The collisions' end: a slab tear, tried and reverted.** Every collision the check counts ends by
stalling into a weld (130 of 130 on the branch: convergence under 5 mm/yr for 10 Myr; at the end
1.2 mm/yr at the median), none by a terrane transfer or a plate's death. The tail's fourteen
(124 to 320 Myr) take up 800 to 7,100 km of shortening, so they keep converging at tens of mm a
year long after the continents meet. On Earth a colliding continent's slab breaks off 10 to 25 Myr after the continent
arrives and the tear runs along the trench at 100 to 300 mm/yr (van Hunen & Allen 2011, full text:
"20-25 Myr" for old slabs, about 10 for young; tear 100 to 150 mm/yr for old slabs, up to 800 for
young, about 300 in their asymmetric model). The tear was built (PLT-324): a contact's slab breaks
off after it has held continental crust for `breakoff_myr`, then the tear runs from that crust along
the trench at 300 mm/yr, and the torn front stops pulling. On master 9d6758bb with the cordillera at
1.7 it lengthened the tail (p90 124 to 157 Myr, p50 40 to 32), took the plate checks from 29 to 21
of 45, the land from 0.319 / 0.298 to 0.241 / 0.239, the coast from 173 / 159 to 122 / 110 km
(the land above 2 km 0.385 / 0.347 to 0.227 / 0.215). The tail does not run on its own slab: its
convergence comes from the other plates' forces, so dropping the slab's pull takes speed from the
plates (and the arcs they feed) without ending the collision. The tear was reverted and has no
ledger row (decisions log 2026-10-20). The other law the task named, a plate reorganisation that
ends a collision by breaking or re-forcing the plate behind it, is a change to how plates split:
left for Ben as its own task.

**The full 16k planet** (`earthlike_full_16k`, seed 1, 8 threads, 1000 Myr; supercontinent /
scattered; one seed each, so within the spread the quick ensemble shows):

| | master, PLT-294 off | branch, off | branch, on | Earth |
|---|---|---|---|---|
| land | 0.333 / 0.334 | 0.365 / 0.330 | 0.312 / 0.268 | 0.29 |
| land above 2 km | 0.255 / 0.305 | 0.299 / 0.314 | 0.380 / 0.455 | 0.05 by rock |
| coast p50, km | 158 / 123 | 185 / 146 | 151 / 210 | 433 |
| continental crust | 0.435 / 0.440 | 0.465 / 0.409 | 0.421 / 0.383 | 0.41 |
| ocean-basin felsic share | | 0.82 / 0.79 | 0.45 / 0.45 | near 0 |
| shortened into the ranges, km³ a year | 2.81 / 3.76 | 1.71 / 1.79 | 1.46 / 1.71 | |
| collision length p50 / p90, Myr (pooled check) | 41 / 117 | 45 / 135 | 38 / 125 | 15 to 50 / at most 120 |
| collisions lose, km³ a year | 2.26 | 1.67 | 2.01 | 0.7 |
| plate checks met (pooled) | 26 of 45 | 25 of 45 | 29 of 45 | |

**Stage times** (`tools/review/p1-19-pair.sh`, `earthlike_full_16k`, scattered continents,
1000 Myr, 8 threads, master c24ff793 and the branch side by side on the VM's two halves, then
swapped; the VM was loaded, 18 to 22 other cores busy, so the totals are above a quiet run's):

| pair | plates, master / branch, s | total, master / branch, s |
|---|---|---|
| ab | 409.2 / 402.0 | 1,634.9 / 1,675.5 |
| ba | 347.6 / 341.5 | 1,422.0 / 1,430.8 |
| mean ratio | 0.982 | 1.016 |

The plates take 1.8 % less (the shorter cordillera moves less rock); the total takes 1.6 % more,
all of it in the surface stage (747 to 780 s), which works on a different planet. The branch
adds no work per step, so the 900 s gate stands where master left it.

**What is left, and for whom.**
1. *The island-arc front* (gain, 2.8 times Earth's): fewer and longer-lived ocean-ocean subduction
   zones from the force balance. Plate work; with it PLT-312 can be measured again.
2. *The collision loss* (3.8 times Earth's) and the collision tail (p90 121 to 135 Myr against at
   most 120): the slab tear is not the tail's law; a plate reorganisation is the other candidate,
   a change to how plates split (Ben's).
3. *The arc's ocean floor made continental* (4.67 km³ a year, no row in Table 1): a column holding
   its arc's magma apart from its ocean crust is a crust data-model change (Ben's). Done
   2026-10-20 by PLT-331 without one: the column keeps the ocean crust under its arc as a basement
   tag on its mafic pools, out of the continental books ("The arc's basement and the arcs' fronts"
   below).
4. *The cordillera per front*: 0.09 of the upper plate's motion over the mantle (Heuret &
   Lallemand) once the force balance hands each arc its upper plate's motion.
5. *Hotspots and rifted margins* (0.04 against 0.5): flood basalts and volcanic margins have no
   rule.


## The arc's basement and the arcs' fronts (2026-10-20)

The coordinator's task after "The crust budget against Stern & Scholl" above, on Ben's complaint
that mountains eat the world and leave a stringy mess of islands: (1) the arc's ocean floor made
continental (4.67 km³ a year with no row in Stern & Scholl's Table 1, item 3 of that section's
"What is left"): keep the arc's own magma apart from the ocean crust under it, so only the arc's
crust counts as continental, with arcs still accreting to continents in collisions; (2) the
island-arc front (142,000 lattice km a step against Earth's 14,500; 31.6 % of the boundary
subducting against 15 to 25): find why and fix it at its law, or report it if the cause is how
plates split and reorganise. Ledger PLT-330 and PLT-331 (amended: PLT-047, PLT-163, PLT-221,
PLT-247, PLT-248, PLT-277, PLT-289, PLT-302, PLT-312); sources in the bibliography, "The arc's
basement and the arcs' fronts". Code: `plates/state.rs` (`Sample::basement_m`,
`make_continental`, `basement`, `own_thickness_m`, `own_pools`), `plates/subduction.rs`
(`fronts_begin`, the contact's resolve), `plates/mod.rs` (`finish_samples`), `plates/belts.rs`
(`stack_front`), `plates/collapse.rs`, `plates/transition.rs`, `plates/ridges.rs`,
`plates/crust_budget.rs`; `tools/review/loss-budget.py` books the basement. The build log entry
of 2026-10-20 "The arc's basement and the arcs' fronts" has the commands.

**What Stern & Scholl count** (Int. Geol. Rev. 52:1-31, read in full; read again for this task).
Their intra-oceanic arc rate is the arc's crust. On p. 7: "The largest sources of error are as
follows: (1) uncertainty about whether to include an original 6–7 km thickness of ‘trapped’
oceanic crust"; "early concepts assumed that new arcs grew on a substrate of pre-existing oceanic
crust that was trapped when the new subduction zone formed (Dickinson and Sealy 1979). More recent
considerations indicate that the substrate for intra-oceanic arcs is produced when subduction
begins (Stern and Bloomer 1992; Hall et al. 2003)". The caption of their Figure 6 (after Tatsumi et
al. 2008): "Incipient arc magmatism replaces the pre-existing oceanic crust to create the initial
mafic arc crust". So Table 1's 1.5 km³ a year holds the ocean crust under an arc at most once, as
the arc's own first crust; the model's arc rate already adds that, and the ocean column the arc
stands on is not new continental crust.

**The basement** (PLT-331). A sample that turns continental from ocean floor records
`basement_m` = min(the ocean column's 7 km, its mafic rock). That covers the three ways ocean floor
turns: an island arc matured (PLT-047, the step's finish), ocean floor as thick as a continent at a
contact (PLT-289) and ocean floor filled to the normal column by the collapse's spread (PLT-277).
On master the arc's maturity was about 47 % of the old row (supercontinent seed 1: the finish 0.48
of the turned area, the subduction step 0.20, the rate limit 0.21, the collapse 0.09, the founder
0.03). The basement stays in the column's mafic pools: the column's rock, its kind and its height
are as before. It is only a tag on how much of the mafic rock is the ocean's. Two readers leave it
out:
- the crust budget, so the continental volume counts the crust on top (PLT-302; `o2c_b` books the
  basement apart, the loss budget's "ocean floor kept under it as basement" row);
- a collision, which stacks the crust on top only (the belts' thickness and pools less the
  basement, PLT-247 and PLT-248), so the basement goes down with the slab. Cloos 1993 (abstract):
  "Oceanic island arc complexes built upon ocean crust" and buoyant crust over about 15 km jam a
  trench; from memory, not read: Boutelier, Chemenda & Burg 2003 (EPSL 212:31-45), an arc at a
  trench loses its lower part to the slab while its upper crust accretes.

The basement thins with its column (the transition's and the collapse's sources), is never more
than the column's mafic rock (what eroded, foundered or flowed took it with it), blends between
same-kind corners at a resampling and is 0 on ocean floor and on the continents' own crust. A
matured arc still turns continental, emerges and accretes to a continent at a collision (test
`arcs_turn_continental_by_their_crust_not_their_height` keeps 7 km of basement under a converted
arc; `a_basement_goes_down_with_the_slab` stacks only the crust on top). Every other reader of
kind and thickness (isostasy and `settle.rs`, the kind's own readers: the regime, the L1 crust
fields, strata and the plate checks' continental share) reads the whole column as before. The
field lives in the plate state and the checkpoint, not in the world data: no format the Java side
reads changes. Checkpoints written before it do not load (no compatibility before a release).

**Why the height keeps the whole column.** Leaving the basement out of the arc's height too
would be a mistake: water-loaded isostasy (`settle.rs`, ISO-078) already puts 7 km of ocean crust
at old ocean floor's depth, 525 + (7,000 − 35,100) / 7 = −3,489 m dry, times 3,300 / 2,270 =
−5.07 km under water, so an arc on old floor stands by its own crust's buoyancy above that.
Dropping the basement's 7 km from the height would sink every arc on old floor by about
1.5 km (1,000 m dry x 1.4537, the agent's arithmetic) below where its crust floats.

**The fronts** (PLT-330). An arc's front was every lattice edge of an active contact along which
the plates converged (`fronts_begin`). A contact belongs to a pair of plates, not to a stretch of
their boundary, and one lattice edge's direction is no azimuth: plates sliding past each other
converge along half the edges of a zigzag boundary. The boundary mix judges the motion along the
boundary's normal (PLT-310) and calls it a transform within Bird's 20 degrees (PLT-311,
`dynamics::classify_boundary`); the fronts did not. So the contacts' strike-slip and parting
stretches built arcs (PLT-045, PLT-285), raised the subduction uplift, shortened cordilleras
(PLT-048) and counted in each plate's front length (the dynamics record's `front_km`, which the
slab-attached test PLT-163 reads). On master with PLT-294 on the island-arc front was 143,997
lattice km a step against the mix's 84,800 km of ocean-ocean subduction (480,580 km of boundary,
32.1 % subducting, 55.0 % of that ocean-ocean). A front is now where the plates converge across
their boundary and the motion lies more than 20 degrees from it, the mix's own test (Bird 2003,
paragraphs [126] and [127]). The island-arc front falls to 85,213 km, at the mix's 93,800 km of
ocean-ocean subduction (476,671 km, 34.4 %, 57.2 %); the fronts' normal convergence is 0.80 of
their full speed, against 0.47 before. Test `a_contact_whose_plates_slide_past_has_no_arc_front`:
a contact converging head-on has more than 20 fronts, the same contact sliding past has none. The
force balance's slab pull (`forces.rs`) finds its own fronts and still takes single lattice edges
(left below).

**Why the subducting boundary is long: the plates.** The rest of the excess is not short-lived,
slow or shallow subduction that a law for where an arc forms could remove. Master's subducting
boundary (the dynamics record's `front_bands`, commit 485858eb, on master's rules; ocean-ocean / ocean-continent):

| | supercontinent | scattered |
|---|---|---|
| slab at least 600 km long | 67 % / 64 % | 72 % / 72 % |
| normal convergence at least 20 mm/yr | 79 % / 80 % | 82 % / 83 % |
| contact at least 24 Myr old | 74 % / 74 % | 77 % / 79 % |

The subduction is long-lived, fast and deep. Its length is the plates' geometry: 24 plates alive at
the median (p10 17, p90 29) against Earth's 52 (14 large; Bird 2003 Table 3), 476,671 lattice km of
boundary a step, about 238,000 km surveyed with the lattice's factor of about 2 (Earth 260,487).
The boundary is about Earth's; 34.4 % of it subducts against Bird's 19.7 % (51,310 km), and 57 % of
that is ocean-ocean against Earth's 35 % (14,500 of Stern & Scholl's 42,000 km of arcs). The two
shares together, 1.75 x 1.63, are the island-arc front's remaining excess (85,213 lattice km,
about 42,600 surveyed, 2.9 times 14,500). How much of the boundary converges is the force balance's
plate motions and how plates split and reorganise: Ben's scope, as the task said, and not changed.

**Measured** (`tools/review/force-balance-ensemble.sh`, `earthlike_quick_16k`, 1000 Myr, seeds 1 to
3, supercontinent / scattered, `DEEPTIME_CRUST_BUDGET=1`; `tools/review/land-shape.py`,
`loss-budget.py`, `collision-ends.py`; km³ a year over the whole run). Master moved during the
task: PLT-294 went on in the shipped presets (71103327); the sim is otherwise the branch point
d9bd2b65's, so both sides ran with `--set tectonics.transition_ocean_crust=true` on that base.

With PLT-294 on (supercontinent / scattered; "PLT-331 only" is the branch with PLT-330's test
taken out, so the two fixes' parts show apart):

| | master | PLT-331 only | the branch (PLT-330 and PLT-331) | the branch + PLT-312 | Earth |
|---|---|---|---|---|---|
| land | 0.271 / 0.272 | 0.267 / 0.273 | 0.234 / 0.242 | 0.210 / 0.203 | 0.29 |
| land above 2 km | 0.348 / 0.297 | 0.315 / 0.313 | 0.312 / 0.324 | 0.335 / 0.362 | 0.05 by rock |
| coast p50, km | 149 / 145 | 168 / 164 | 159 / 159 | 165 / 160 | 433 |
| continental crust, share of the planet | 0.411 / 0.405 | 0.385 / 0.392 | 0.348 / 0.351 | 0.303 / 0.297 | 0.41 |
| island-arc land, share of the planet | 0.139 / 0.156 | 0.136 / 0.139 | 0.103 / 0.107 | 0.089 / 0.097 | small |
| other land (collision, Andean, the rest) | 0.132 / 0.116 | 0.131 / 0.134 | 0.131 / 0.135 | 0.121 / 0.106 | |
| plate checks met (pooled) | 28 of 45 | 29 of 45 | 28 of 45 | 27 of 45 | |
| island arcs | 4.25 | 4.31 | 3.24 | 2.81 | 1.5 |
| continental arcs | 1.58 | 1.64 | 1.62 | 1.15 | 1.0 |
| ocean floor made continental | 4.33 | 0.12 | 0.14 | 0.12 | none |
| kept under it as basement (not counted) | | 4.09 | 3.71 | 3.06 | |
| collisions lose | 1.85 | 1.83 | 1.83 | 2.13 | 0.7 |
| foundering | 1.06 | 0.82 | 0.76 | 0.76 | not known |
| net change of the continental volume | +0.21 | -0.91 | -1.57 | -2.25 | about 0 |
| island-arc front, lattice km a step | 143,997 | 143,992 | 85,213 | 89,262 | 14,500 |
| continental-arc front, lattice km a step | 99,201 | 98,738 | 59,909 | 55,814 | 27,500 |
| boundary subducting (the mix) | 32.1 % | 32.5 % | 34.4 % | 35.4 % | 15 to 25 % |
| collisions per 100 Myr | 2.15 | 2.12 | 1.58 | 1.55 | 2 to 6 |
| collision length p50 / p90, Myr (pooled check) | 38 / 121 | 36 / 115 | 48 / 121 | 53 / 136 | 15 to 50 / at most 120 |

(The island-arc land is the land times the share of land whose crust an island arc made, the
crust budget's `cb_origin`; "other land" is the rest of the land.)

**What each fix does.** PLT-331 alone holds the land (0.267 / 0.273), brings the coast nearer
Earth's (168 / 164 km), moves the high ground within the seeds' spread (0.315 / 0.313), gains a
plate check (29 of 45) and brings the collisions' p90 under 120 Myr (115): the basement that goes
down with the slab no longer stacks into the belts. The ocean floor row falls from 4.33 to 0.12 km³
a year (Table 1: none; the rest is ocean floor thinner than 7 km when it turned), and 4.09 stays
under the arcs as basement. The continental crust's share falls a little (0.411 / 0.405 to 0.385 /
0.392) and the budget's net to -0.91: the books no longer count the basement.

PLT-330 on top takes the island arcs from 4.31 to 3.24 km³ a year, and the land it costs (0.267 /
0.273 to 0.234 / 0.242) is island-arc land only: the island arcs' land falls from 0.136 / 0.139 of
the planet to 0.103 / 0.107, the rest of the land holds at 0.131 / 0.134 to 0.131 / 0.135. The arcs
the contacts' transform and parting stretches built were islands, the stringy mess Ben named, and
island arcs still make 0.44 of the land (on Earth the intra-oceanic arcs hold a few per cent of the land:
the agent's estimate from memory, not a measured row). The coast moves within the spread (159 / 159), a plate check
goes (28 of 45) and the collisions' number falls under the target (2.12 to 1.58 per 100 Myr): fewer
arcs reach continents. PLT-330 stays on (decisions log 2026-10-20, the arc-basement task): the
front test is Bird's, and keeping the transform stretches' arcs would keep a source with no
subduction under it to stand in for the losses the model has too much of (the collisions lose 1.83
against 0.7).

With PLT-294 off (the branch point's presets), for the record:

| | master | branch | branch + PLT-312 | Earth |
|---|---|---|---|---|
| land | 0.284 / 0.256 | 0.278 / 0.251 | 0.228 / 0.249 | 0.29 |
| land above 2 km | 0.320 / 0.359 | 0.414 / 0.374 | 0.445 / 0.386 | 0.05 by rock |
| coast p50, km | 149 / 158 | 165 / 150 | 185 / 159 | 433 |
| continental crust, share of the planet | 0.418 / 0.349 | 0.354 / 0.348 | 0.292 / 0.322 | 0.41 |
| plate checks met (pooled) | 27 of 45 | 29 of 45 | 30 of 45 | |
| island arcs | 4.20 | 2.99 | 2.90 | 1.5 |
| continental arcs | 1.56 | 1.51 | 1.21 | 1.0 |
| ocean floor made continental | 4.67 | 0.24 | | none |
| kept under it as basement | | 3.74 | | |
| collisions lose | 2.65 | 2.07 | 2.56 | 0.7 |
| foundering | 1.23 | 0.89 | | not known |
| net change of the continental volume | -0.12 | -1.00 | -1.67 | about 0 |
| island-arc front, lattice km | 142,395 | 78,778 | | 14,500 |
| continental-arc front, lattice km | 99,288 | 58,242 | | 27,500 |
| boundary subducting | 31.6 % | 33.3 % | | 15 to 25 % |
| collisions per 100 Myr | 2.17 | 1.98 | | 2 to 6 |

**The ocean-basin felsic share** (the share of `ocean_basin` cells on oceanic crust holding more
than 50 m of felsic plutonic rock, `tools/review/geo_template_fields.py` on the final checkpoint;
the six runs' mean and range, or the one full run):

| | master | PLT-331 only | the branch | Earth |
|---|---|---|---|---|
| quick, PLT-294 on | 0.46 (0.34 to 0.57) | 0.44 (0.39 to 0.52) | 0.26 (0.22 to 0.28) | near 0 |
| quick, PLT-294 off | 0.76 (0.61 to 0.91) | | 0.64 (0.51 to 0.76) | near 0 |
| full 16k, on / off | 0.45 / 0.79 | | 0.26 / 0.61 | near 0 |

The fronts do it: the arcs the transform stretches built dropped their felsic rock onto the ocean
floor next to them (the arcs' felsic rock onto the ocean floor falls from 4.02 to 3.09 km³ a year
with PLT-294 on).

**PLT-312 again** (`DEEPTIME_ARC_NORMAL=1`, the arcs on the normal convergence). With the fronts on
the converging boundary the switch moves the arcs less than before (island arcs 3.24 to 2.81 km³ a
year, continental 1.62 to 1.15, against 4.20 to 2.19 and 1.56 to 0.80 in the last section), and
land falls further below Earth's, the land above 2 km rises and the plate checks lose one (28 to
27 with PLT-294 on; with it off they gain one, 29 to 30, while land and high ground move the same
way). Two or more of the four measures move away from Earth on both bases, so it stays off
(decisions log 2026-10-20, the arc-basement task).

**The full 16k planet** (`earthlike_full_16k`, scattered continents, seed 1, 8 threads, 1000 Myr;
one seed, so within the spread the quick ensemble shows; master's run with PLT-294 on repeats the
last section's "branch, on" column exactly):

| | master, PLT-294 on | branch, on | master, off | branch, off | Earth |
|---|---|---|---|---|---|
| land | 0.268 | 0.236 | 0.330 | 0.264 | 0.29 |
| island-arc land / other land, share of the planet | 0.146 / 0.122 | 0.111 / 0.125 | 0.176 / 0.154 | 0.129 / 0.135 | |
| land above 2 km | 0.455 | 0.270 | 0.314 | 0.394 | 0.05 by rock |
| coast p50, km | 210 | 159 | 146 | 141 | 433 |
| continental crust | 0.383 | 0.359 | 0.409 | 0.355 | 0.41 |
| plate checks met | 27 of 45 | 32 of 45 | 26 of 45 | 29 of 45 | |
| island arcs | 5.17 | 3.89 | 4.47 | 3.51 | 1.5 |
| continental arcs | 1.86 | 1.95 | 1.59 | 1.77 | 1.0 |
| ocean floor made continental | 5.18 | 0.12 | 4.78 | 0.19 | none |
| kept under it as basement | | 4.13 | | 4.01 | |
| collisions lose | 2.41 | 2.42 | 2.28 | 2.79 | 0.7 |
| foundering | 2.96 | 1.50 | | | not known |
| net change of the continental volume | +0.45 | -1.38 | +0.96 | -0.85 | about 0 |
| island-arc front, lattice km a step | 180,850 | 106,810 | 167,460 | 96,630 | 14,500 |
| boundary subducting | 33.6 % | 35.2 % | 32.4 % | 35.8 % | 15 to 25 % |
| collision length p50 / p90, Myr | 32 / 124 | 36 / 78 | 48 / 158 | 52 / 130 | 15 to 50 / at most 120 |
| collisions per 100 Myr | 3.10 | 2.30 | 3.70 | 2.50 | 2 to 6 |

On the full planet with PLT-294 on the branch gains five plate checks and its high ground falls
from 0.455 to 0.270 of the land; the land falls with the island arcs as on the quick ensemble (the
other land holds, 0.122 to 0.125) and the coast's 210 km falls to 159.

**Stage times** (`tools/review/p1-19-pair.sh`, master d9bd2b65 against the branch side by side,
`earthlike_full_16k`, scattered continents, 1000 Myr, 8 threads, the presets' PLT-294 off on both,
pairs ab and ba; the VM loaded by other agents, 16 to 19 other CPUs busy on average): the plates
stage 305.0 against 309.2 s and 348.3 against 328.8 s (the branch's over master's, 1.014 and
0.944), the whole run 1,263.9 against 1,234.8 s and 1,421.4 against 1,350.6 s (0.977 and 0.950).
The step does no new work: the fronts' test reads the normals `fronts_begin` already had, and the
basement is one more float a sample. The rest of the difference is in the surface stage, which
works on a different planet.

**What is left, and for whom.**
1. *The boundary's subducting share* (34 % against Bird's 19.7 %, 57 % of it ocean-ocean against
   35 %): the plates' geometry and motions, how plates split and reorganise. Ben's.
2. *The force balance's slab pull* takes its fronts from single lattice edges (`forces.rs`), as the
   arcs did before PLT-330: a contact's transform stretches still pull. Giving it PLT-330's test is
   a change to the plate forces, measured on its own (the plate speeds and the slab-attached checks
   move with it).
3. *Land and continental crust fall* below Earth's with the island arcs (land 0.234 / 0.242 against 0.29;
   continental crust 0.348 / 0.351 against 0.41, and the check on its lowest step fails at 0.30
   against at least 0.37). The budget's net is -1.57 km³ a year, against +0.21 on master, which
   counted 4.33 of ocean floor as new crust. The continents' own land holds; what is short is island
   arcs, and the arcs that remain still add twice Earth's (3.24 against 1.5) on a front 2.9 times
   Earth's while the collisions lose 2.6 times Earth's. The levers are the plates' geometry (item 1)
   and the losses (item 4), not the arcs' rule.
4. *The collisions*: their number now fails (1.58 per 100 Myr on the quick ensemble with PLT-294
   on, target 2 to 6), the p90 holds at 121 Myr (at most 120) and the loss at 1.83 km³ a year
   (Earth 0.7); a plate reorganisation is the open law (Ben's, as in the last section).
5. *The high ground* (0.31 to 0.39 of land above 2 km against 0.05 by rock) did not move toward
   Earth with either fix and stays the high-ground work's.

## White patches, drowned continents and coastal high ground (2026-10-21)

Ben ran build 0a7431e1 (`earthlike_quick_16k`, seed 1, supercontinent start, 1000 Myr, keyframes
every 50 Myr) and asked three things: what the "absolutely massive patches that seem to have
absurd elevation" 840 to 736 Myr ago are, "where did my continents go?", and why the mountains are
"very prone to forming directly bordering oceans". Measured on his staging copy on the VM (the
original under his Prism instance untouched): his ten epoch checkpoints (L1 fields through
`deeptime dump-fields`, 332,800 cells of 1,533 km²) and his keyframes; master 30c4d664's quick
ensemble (seeds 1 to 3, both starts, with the crust budget's maps, PLT-302) and one more
supercontinent seed-1 run; and Earth on the same measures (CRUST1.0, Laske et al. 2013; SRTM15+
15', Tozer et al. 2019, on 332,800 Fibonacci points, the L1 grid's count). Tool:
`tools/review/continents_census.py` (`fields`, `keyframes`, `maps`, `earth`). Ledger PLT-360 to
PLT-363 and CLM-220. No simulation code changed. This extends "Land shape and the high ground"
above, whose findings all hold.

### The white patches are ice on high plateaus

They are both: glaciers and perennial snow on Himalayan-type plateaus 4 to 5 km high on 60 to 70 km
of crust. The blocky edges are the L1 cells (about 39 km), the satellite view's resolution, not an
artefact. The largest patches on Ben's run:

| time | patch, Mkm² | latitude | ground above sea, p50 / p90 / max, m | ice p50, m | ELA p50, m | crust p50, km |
|---|---|---|---|---|---|---|
| -820 Myr | 0.95 | 14°S | 4,508 / 5,142 / 5,463 | 788 | 3,296 | 69.9 |
| -730 Myr | 7.86 | 42°N | 4,069 / 5,025 / 5,785 | 897 | 3,050 | 65.7 |
| -730 Myr | 3.88, 1.98, 1.65 | 34°N, 22.5°S, 46°S | | | | |

They grow with the plateaus. The crust 55 km and thicker covers 0 (-910 Myr), 3.2 (-820) and 17.2
Mkm² (-730), then 13 to 22 Mkm²; the land under ice is 0 (-950), 0.013 (-850), 0.05 (-800), 0.16
(-750) and 0.19 to 0.30 of the land after. The high ground is young: its last orogeny is 0 / 6 /
22 Myr old (p10 / p50 / p90) at -730 Myr and 2 to 18 Myr at the median later.

**Against Earth.** The heights are Earth's; the extent is not:

| | Ben's run, -910 / -820 / -730 Myr, then to -100 | 30c4d664, end (7 runs) | Earth (SRTM15+) |
|---|---|---|---|
| land above 2 km, share of land | 0.004 / 0.037 / 0.229, then 0.24 to 0.33 | 0.24 to 0.39 | 0.106 by surface, 0.055 without the two ice sheets, 0.047 by rock (HYP-041) |
| land above 4 km, share of land | 0.0001 / 0.009 / 0.082, then 0.03 to 0.09 | 0.06 to 0.15 | 0.017 (0.019 without the ice sheets) |
| largest ground above 3 km, Mkm² | 0.06 / 0.90 / 5.27, then 2.8 to 7.8 | 3.6 to 11.9 | 2.95 (Tibet; median 4,580 m) |
| highest mean of a 1,000 km box, m | 1,056 / 3,014 / 4,517, then 4,180 to 4,923 | | 4,837 (Tibet, 32.6°N 88.1°E) |
| white share of land | 0.001 / 0.03 / 0.22, then 0.20 to 0.35 | land under ice 0.20 to 0.29 | about 0.10 (the two ice sheets and the glaciers, from memory) |

So the plateaus stand at Tibet's height, but the largest is up to 2.7 times Tibet's area on Ben's
run and 4 times on 30c4d664, and the land above 2 km is four to seven times Earth's 0.055 without
its ice sheets (two to three and a half times the 0.106 that counts them).

**Why they carry ice: the plateaus' summers are cold.** The glaciers' rule (Ohmura et al. 1992,
`glaciers.rs`) puts ice where the warmest month at the ground is below T_ELA, which is -2.35 °C
with no snow and 1.2 °C with 1,000 mm a year. The plateaus' summers are far below either (land 3.5
to 5 km high, Ben's checkpoints from -730 Myr on):

| latitude | warmest month, p50 at the ground | ground p50 | glacier share |
|---|---|---|---|
| 10 to 25° | -6.8 to +0.9 °C (+5.1 °C at -100 Myr, the one ice-free frame) | 3.6 to 4.3 km | 0.00 to 0.90 |
| 25 to 45° | -10.4 to -4.2 °C | 3.8 to 4.4 km | 0.48 to 0.99 |
| 45 to 60° | -16.4 to -11.6 °C | 3.7 to 4.3 km | 0.99 to 1.00 |

On Earth the Tibetan Plateau at 4.5 km has a July mean near 10 °C (Nagqu, 4,507 m, 31.5°N; 1991 to
2020 normals, read in a secondary summary) and the Altiplano at 3.8 km near 10 °C (Puno, from
memory), so neither glaciates below about 5.5 km. The model's surface temperature is the sea-level
temperature less 6.5 K/km (`lapse.rs`) everywhere. Earth's large plateaus are warmer than the free
air at the same height (the mass-elevation effect): 2 to 7 °C warmer than their surroundings at
equal altitude, 3.6 to 6.6 °C at 4,500 m against the Sichuan Basin's air (Yao & Zhang 2015, read
in search snippets only; Wiley refused the page), and Nagqu's July is about 11 °C above what a
6.5 K/km lapse from a 28 °C lowland July at 31°N would give (from memory). The model has no such
term (CLM-220). Adding 5 to 11 °C would bring the median plateau summer at 25 to 45° to about -5
to +7 °C, around T_ELA: it would cut the plateau ice, not remove it, while the plateaus' extent
stays three to five times Earth's. Once a plateau is white its albedo holds it.

**Not an ice age.** Ben's run has no snowball: CO₂ 1,500 to 3,000 ppm from -850 to -730 Myr, the
global mean 10.6 to 13.1 °C, the sun at 0.93 to 0.94 of today's. The land's mean temperature falls
from 10.5 to 5.6 °C as the land above 2 km grows from 0.024 to 0.23 of the land: the ice follows the
high ground, starting near -850 Myr, about 130 Myr before the Cryogenian's first glaciation (the
Sturtian, about 717 to 660 Ma; the Marinoan about 650 to 635 Ma; from memory, Rooney et al. 2015 and
Hoffman et al. 2017). The simulation has no clock tied to Earth's dates, so the overlap is chance.
The ice is sound for its inputs (the snow line and its temperature rule); the inputs are the
plateaus' extent and their missing warmth.

### Where the continents went

Ben's keyframes (selected; the continental crust from the `crust_age` layer's grey, the land from
`elevation` at the manifest's sea level):

| t, Myr | land | continental crust | its share under the sea (shelf < 200 m, deeper) | sea level, m | freeboard41, m | landmasses over 1 Mkm² |
|---|---|---|---|---|---|---|
| -998 | 0.426 | 0.424 | 0.000 | 30 | 497 | 1 |
| -900 | 0.322 | 0.447 | 0.285 (0.097, 0.188) | 98 | 378 | 3 |
| -800 | 0.272 | 0.449 | 0.404 (0.112, 0.292) | -61 | 229 | 5 |
| -750 | 0.213 | 0.409 | 0.498 (0.139, 0.358) | 1 | 24 | 6 |
| -600 | 0.228 | 0.346 | 0.383 (0.086, 0.296) | -122 | 109 | 8 |
| -500 | 0.203 | 0.343 | 0.442 (0.106, 0.337) | -5 | -9 | 7 |
| -300 | 0.277 | 0.363 | 0.299 (0.071, 0.228) | -152 | 420 | 7 |
| -100 | 0.272 | 0.374 | 0.315 (0.077, 0.239) | -134 | 258 | 5 |
| 0 | 0.237 | 0.362 | 0.395 (0.119, 0.276) | -38 | 162 | 7 |
| Earth | 0.29 | 0.41 with shelves (CRUST1.0 ≥ 20 km 0.384, ≥ 25 km 0.357) | 0.248 (0.110, 0.138) for ≥ 20 km, 0.195 (0.115, 0.080) for ≥ 25 km | 0 to 250 above today (Phanerozoic) | 220 | |

freeboard41 is the median height above the sea of the planet's highest 41 % (Earth's share of
continental crust with its shelves), the crust's stand against the sea. The shelf share is
Earth's (0.06 to 0.14 from -900 Myr on, against 0.11); the excess is the crust deeper than
200 m, 0.19 to 0.36 of the continental crust against Earth's 0.08 to 0.14.

**A mix, about 60 % drowned and 40 % lost.** From the first plate frame (-950 Myr) to the last the
land falls by 0.158 of the planet: the continental crust under the sea grows from 0.043 to 0.143
of the planet (it is still there, under the sea), and the crust's area falls from 0.435 (peak
0.459 at -850) to 0.362, which takes about 0.066 of the planet's land with it. The crust does not
thin on land: the land's crust is CRUST1.0's at p10, at p50 it is 4 km thin at -730 Myr on Ben's
run and Earth's at 30c4d664's end, and the isostatic settle gives back more than the erosion takes
(below). The sea-level curve stays within -172 and +98 m,
inside Earth's range; the drowning is the crust standing low, not the sea standing high:

| | land, p10 / p50 / p90, km | under the sea, p10 / p50 / p90, km |
|---|---|---|
| Ben's run, -730 Myr | 31.9 / 34.4 / 63.6 | 20.9 / 28.6 / 31.8 |
| 30c4d664, end (7 runs) | 31.7 to 32.8 / 37.8 to 42.5 / 56.5 to 66.0 | 19.7 to 22.0 / 27.7 to 29.4 / 31.3 to 32.2 |
| CRUST1.0, crust ≥ 20 km | 32.0 / 38.6 / 45.8 | 21.6 / 28.8 / 35.8 |
| CRUST1.0, crust ≥ 25 km | 32.0 / 38.7 / 45.8 | 26.4 / 29.9 / 37.0 |

The drowned crust is Earth's drowned crust by thickness. What differs is how much of the crust
sits in the 25 to 35 km band and where that band stands: on 30c4d664 (supercontinent seed 1, end)
the 25 to 30 km band is 0.14 of the continental area at a median -654 m (0.09 dry) and the 30 to
35 km band 0.34 at +19 m (0.57 dry), around the Airy line's sea-level crossing at 31.4 km. Earth's
25 to 30 km crust is about 80 % under the sea as well.

**The budget** (dry continental crust gained or lost by each process of the plate step, summed
over the 1000 Myr, in shares of the planet; 30c4d664, supercontinent seeds 1 to 3 / scattered seeds
1 to 3 / the extra supercontinent seed 1):

| process | dry crust |
|---|---|
| erosion | -1.59 / -1.55 / -1.52 |
| isostatic settle (ISO-078) | +1.94 / +1.88 / +1.87 |
| margin transition (PLT-261 to PLT-265, PLT-294) | -0.76 / -0.73 / -0.74 |
| subduction | -0.47 / -0.58 / -0.61 |
| resampling | -0.06 / -0.05 / -0.12 |
| lower-crustal flow (PLT-300, PLT-301) | +0.01 |
| collision belts / arcs / collapse / rate | +0.35 / +0.27 / +0.08 / +0.13 (supercontinent) |
| net | -0.04 / -0.07 / -0.09 |

The margin transition is the largest net drowner; erosion without rebound is not the cause (the
settle more than restores it), and the flow and the resampling are small. The 2026-10-20 ablation
of the margin taper (24.5 km instead of 200) moved the land only from 0.310 to 0.323, so the
transition is the channel, not the cause by itself.

**For the sea-level task** (decisions log 2026-10-21; not built here). The ocean's water under the
curve on Ben's keyframes runs 1.06 to 1.34 × 10⁹ km³ (Earth-normalised; the start holds 1.085,
Earth 1.332 plus its ice). The sea level that would hold the start's water (ice returned at 0.917)
is 37 to 840 m below the curve's after the start, with land 0.29 to 0.43; with Earth's water
(PLT-362) it is 1 m below to 643 m above, with land 0.12 to 0.22 from -850 Myr on (0.28 and 0.39 in
the first two plate frames). The first frame's water is 80 %
of Earth's, so the choice of the planet's water decides whether a volume-balanced sea level
restores the land or drowns more of it.

### The coastal high ground

Measured on the grid's cells: the share of the coast with land above 2 km within 200 km inland on
the same landmass, and the share of the land above 2 km within 200 km of the sea, with the share
of all land within 200 km of the sea beside it (narrow landmasses put more of everything near the
coast).

| | coast with high ground within 200 km | high ground within 200 km of the sea (all land) |
|---|---|---|
| Ben's run, -910 / -820 / -730 Myr | 0.065 / 0.198 / 0.423 | 0.559 (0.161) / 0.431 (0.296) / 0.538 (0.538) |
| Ben's run, -640 to -100 Myr | 0.26 to 0.37 | 0.18 to 0.37 (0.45 to 0.56) |
| Earth | 0.104 (0.076 without Antarctica and Greenland) | 0.070 (0.294); 0.110 (0.293) without them |

Three to five times Earth's share of coast carries high ground (2.5 to 4 times the 0.104 that
counts the ice sheets, 3.5 to 5.5 times the 0.076 without them). Two things make it: the land above 2 km is
four to seven times Earth's share, and the model's landmasses are narrower (0.45 to 0.56 of the
land within 200 km of the sea against 0.29). Even allowing for that, the high ground sits nearer
the coast than Earth's: its coastal share is 0.4 to 1.0 of the land's, against 0.24 (0.38 without
the ice sheets) on Earth.

**By the crust's origin.** On Ben's run (the L1 `plates/orogeny_type`) the coast of Himalayan
(collision) crust carries high ground on 0.35 to 0.64 of its length, Andean 0.17 to 0.32, none 0.20
to 0.37. On 30c4d664's maps at the end (the budget's origin tag: island arc if its arc crust is
1 km or more and mostly oceanic, then collision, Andean, other; the maps' samples are coarser
than the L1 grid, so the shares sit higher, 0.45 to 0.63 overall):

| origin | share of the land | its land above 2 km | share of the coast | its coast with high ground |
|---|---|---|---|---|
| island arc | 0.36 to 0.49 | 0.35 to 0.54 | 0.47 to 0.58 | 0.47 to 0.66 |
| collision | 0.17 to 0.25 | 0.30 to 0.36 | 0.14 to 0.22 | 0.53 to 0.78 |
| Andean | 0.24 to 0.37 | 0.14 to 0.31 | 0.19 to 0.27 | 0.23 to 0.45 |
| other | 0.04 to 0.11 | 0.01 to 0.02 | 0.07 to 0.11 | 0.29 to 0.61 |

**What the plate geometry explains.** The plates subduct on 34 % of the boundary (Bird 19.7 %), 57 %
of it ocean-ocean (35 %), so ocean-ocean subduction is 0.196 of the boundary against Earth's 0.069
(2.8 times; the island-arc front is 2.9 times Earth's 14,500 km). Cutting the island-arc coast by
2.9 with its high ground lowers the coast with high ground by only 0.01 to 0.03 on the maps (0.45
to 0.63 to 0.42 to 0.62), because the collision and Andean coasts carry high ground as often. So the geometry
explains why a third to a half of the land is island arc (Earth's island arcs are a few per cent
of its land, from memory), but little of the coastal mountains. Those come from too much young
orogen on narrow landmasses. The 24 plates against Earth's 52 and the collisions at 1.58 per 100
Myr (target 2 to 6) bear on the landmasses' size and on how often continents meet, which this
measure cannot separate.

### What is left, and for whom

1. *The plate geometry* (Ben, decisions log 2026-10-21: its own task, next): the island-arc
   share of the land and the collisions. It will not by itself move the coastal high ground.
2. *The high ground's extent* (four to seven times Earth's ice-free land above 2 km, plateaus up to
   four times Tibet's area on 60 to 70 km crust): the high-ground work, as in the sections above. This is
   what puts both the ice and the coastal mountains on the map. Done as far as it goes without
   the sea at the datum: "The high ground's extent: post-orogenic collapse (2026-10-21)" below.
3. *The plateaus' warmth* (CLM-220): a mass-elevation term in the surface temperature is a climate
   law for every planet (it moves the ice, the albedo and the CO₂ the weathering draws down).
   Recommended as its own climate task.
4. *The sea level from the ocean's volume*: the numbers above are the sea-level task's test case;
   the planet's water amount is the choice that decides its sign.
5. *The margin crust* (28 to 30 km, below the 31.4 km crossing): the transition's share of the
   drowning is the lead if the land is still short once 1 and 4 land.

## The plate geometry: arcs, reversals and where subduction starts (2026-10-21)

Ben (2026-10-21): "mountains seem very prone to forming directly bordering oceans, I don't believe
this is realistic. (I usually expect mountains to form at continent - continent intersetcions
though I might be wrong.)" and "I've noticed a lack of proper continents across most of my recent
runs in the past ~2 days". The section above handed this task the plates' part: the boundary mix
(34 % subducting against Bird's 19.7 %, 57 % of it ocean-ocean against Stern & Scholl's 35 %), 24
plates against 52, 1.6 collisions per 100 Myr (target 2 to 6) and a third to a half of the land
built as island arc. Ledger PLT-370 to PLT-372 (notes on PLT-042, PLT-056, PLT-062, PLT-064,
PLT-221, PLT-330); sources in the bibliography, "The plate geometry". Code: `plates/dynamics.rs`
(the boundary mix's classes, the `FrontBands` diagnostics), `plates/subduction.rs` (the arc-front
counters, `continent_within`), `plates/check.rs` (a label); `tools/review/plate-geometry.py`.
The build log entry of 2026-10-21 "The plate geometry" has the commands.

Measured on master c790d1f3 (`earthlike_quick_16k`, 1000 Myr, seeds 1 to 3, both starts, the
crust budget's maps on; `tools/review/plate-geometry.py` on the dynamics records, means over the
steps after the first 100 Myr):

| measure | model | Earth |
|---|---|---|
| boundary: subducting (ocean-ocean + ocean-continent) | 0.347 (0.197 + 0.150) | 0.197 (Bird 2003 Table 3, SUB) |
| boundary: collision | 0.068 | 0.088 (CCB) |
| boundary: spreading (ridge + rift) | 0.353 | 0.363 (OSR + CRB) |
| boundary: transform / converging without a contact | 0.210 / 0.022 | 0.283 (OTF + CTF) / 0.067 (OCB) |
| ocean-ocean share of the subducting boundary | 0.567 | 0.35 (14,500 of 42,000 km of arc, Stern & Scholl 2010) |
| plates alive | 23.7 | 52 |
| plates over 1 / 0.3 / 0.1 / 0.0377 / 0.01 sr | 3.4 / 12.4 / 19.4 / 22.7 / 23.5 | 7 / 10 / 14 / 19 / 31 (Bird Table 1) |
| collisions (terrane at least 0.1 A0) per 100 Myr | 1.63 | 2 to 6 (CHK-015) |
| polarity reversals per 100 Myr | 17.0 | 4 since 104 Ma (Crameri et al. 2020, Table 1) |
| subduction initiations per 100 Myr | 7.0 | 13 since 104 Ma, all within 1,500 km of an active subduction zone (Crameri) |

The model's boundary is 480,054 lattice km a step against Bird's 260,487 km: a boundary on the
lattice zigzags, so its lattice length is 1.84 times the geodesic one, and Earth's 14,500 km of
island arc is about 26,700 lattice km. (Corrected 2026-10-24, the front lane, PLT-530: the 1.84
was this ratio itself, which assumes the model's boundary is as long as Earth's. One spacing per
boundary sample counts a straight boundary 1.04 times, so the model's boundary is about 460,300
km, 1.77 times Bird's, and Earth's island arc is about 15,100 lattice km.)

### What makes the gap

**The ocean-ocean excess was mostly a classification.** The mix named a front by the crust of the
front sample itself. The overriding plate's oceanic samples at its edge are often a thin fringe in
front of its continent: on seed 1 (both starts), 48 % of the ocean-ocean front had the overriding
plate's continental crust within 150 km of it through that plate, 70 % within 300 km, and only 12 %
lay 1,200 km or more from it. The arc stands 200 km behind the front (`arc_distance_km`, PLT-042),
so most of these arcs stand on continental crust: continental arcs in Stern & Scholl's sense, who
split the world's arcs by the crust the arc is built on ("the present 14,500 km of intra-oceanic
arcs", "The combined length of continental margin arcs is approximately 27,500 km"). The arc
builder already did this right (it books the crust by the kind of the sample the arc lands on); the
mix and the arc-front counters did not.

**The subducting share is the boundaries' orientation, not where subduction starts.** The model's
boundary is 43.7 % converging, 35.3 % parting and 21.0 % transform; boundaries at random to the
motion, with Bird's 20 degree transform window (PLT-311), would be 39 / 39 / 22; Earth's are 35.2 /
36.3 / 28.3. Switching off subduction initiation (seeds 1 and 2, both starts) leaves the
subducting share at 0.340 (0.347 with it), and switching off the oceanic share of rifting
(`rift_ocean_weight` 0) at 0.341: neither makes it. Earth's spreading boundaries are segmented
into ridges square to the spreading and transforms along it, and its trenches face their
convergence; the model's plates are cut by warped Voronoi splits and keep those boundaries
whatever the motion becomes. That is a property of how plates are made and moved, not a constant
to tune.

**Collisions are few because polarity reversal ends most of them.** Of the collision states the
base runs began, 483 ended by reversal against 98 counted collisions (and 1,312 small terranes
parted). The reversed ones lasted 6 Myr at the median, after 145 km of shortening (p90 868 km);
82 % had 0.1 A0 or more of continental crust on the subducting plate (median 0.32 A0), while the
overriding plate held a median 0.06 A0 of continental crust on 0.49 A0 of area. Reversals ran at
17 per 100 Myr against Crameri's 4 polarity-reversal initiations since 104 Ma. The reversal
(PLT-056) judges the overriding side by the crust of its samples in the overlap, the same fringe:
a continent arriving at an overriding plate whose own continent stands 100 km behind its oceanic
edge reads as arriving at an intra-oceanic arc and flips the trench.

Stern (2004) separates the two cases. Polarity reversal "is triggered by buoyant crust entering the
trench" where "the new subduction zone nucleates in the originally overriding plate", whose back
arc is "relatively thin and weak" (the Solomon arc after the Ontong Java Plateau arrived;
"subduction on the south side of the Solomons began almost immediately"). Where the overriding
plate is a continent, the jam goes on as a collision ("It takes less work to deform large parts of
continental Asia than it does to start a new subduction zone in the Indian Plate"), and any new
zone forms outboard (transference). So the test the reversal needs is the crust the arc stands
on, not the crust at the plate's edge.

**The plate count is the lattice's floor.** At 100,000 samples the smallest plate the model keeps
(PLT-091, 300 samples) is 0.0377 sr. Above that the model has 22.7 plates against Bird's 19; it is
short of large plates (3.4 over 1 sr against 7) and has too many middle-sized ones (19.4 over 0.1
sr against 14). Bird's 52 count 33 plates under 0.0377 sr (21 under 0.01 sr), below what this
resolution holds; Morra et al. (2013) and Mallard et al. (2016) treat Earth's small plates as a
second population, set by "a hierarchical fragmentation process" and broken off by "the stresses
caused by the bending of trenches", for which the model has no law. The count is not a target
this task can move without a finer lattice.

**Subduction starts in the wrong place.** The model starts subduction at floor older than 150 Myr
farther than 1,000 km from every trench (PLT-062, PLT-064, neither sourced). Crameri et al. (2020)
compiled the 13 initiation events since 104 Ma: "All SZI events start less than 1500 km away
from other active subduction zones and, in most cases, less than 500 km"; "At least nine of all
thirteen events ... occur at or near a pre-existing convergent plate boundary"; the subducting
plates' mean ages at initiation are 32 to 117 Myr (Supplementary Data 1); "collision events along
pre-existing subduction trenches are often precursors of SZI events"; and "subduction breeds
subduction". Gurnis et al. (2004, Table 1) list eleven Cenozoic initiations, each at a fracture
zone, a transform, a spreading centre, an extinct subduction zone or by polarity reversal; none in
old floor far from a boundary.

### What changed: an arc on continental crust is a continental arc (PLT-370)

The boundary mix and the arc-front counters (`plates/arc_front_continental_km_myr`,
`arc_front_oceanic_km_myr`, which `tools/review/crust-budget.py` divides the arcs' crust by) now
name an ocean-ocean front by the crust its arc stands on: a front whose overriding plate has
visible continental crust within `arc_distance_km` of it, through that plate's own visible
samples, counts as ocean-continent. The test is the one the arc builder's booking implies (the arc
lands 200 km behind the front). It is measurement only: the branch's runs repeat master's
exactly (land, crust, plates, collisions and every scored plate check are identical; the census
at the end too).

| | master c790d1f3 | branch | Earth |
|---|---|---|---|
| ocean-ocean share of the subducting boundary | 0.567 | 0.295 | 0.35 |
| ocean-ocean share of the boundary | 0.197 | 0.103 | 0.069 (35 % of 19.7 %) |
| ocean-continent share of the boundary | 0.150 | 0.245 | 0.128 |
| subducting share of the boundary | 0.347 | 0.347 | 0.197 |
| island-arc front (the counters), lattice km a step | 85,095 | 45,275 | 14,500 km, about 26,700 lattice km |
| continental-arc front, lattice km a step | 61,088 | 100,913 | 27,500 km, about 50,700 lattice km |
| island arcs' crust per km of island-arc front per Myr | 70 km³ | 131 km³ | 100 km³ (90 to 180 at Izu-Bonin-Mariana and the Aleutians) |
| continental arcs' crust per km of continental-arc front per Myr | 49 km³ | 30 km³ | 35 km³ |

(The rates per km take the counters' lattice km over the boundary's 1.84. The arcs added 3.22 km³
a year to ocean floor and 1.62 to continents on both, against Earth's 1.5 and 1.0.)

By the crust the arc stands on, the model's island-arc front is 1.7 times Earth's, not 2.9 or 3.2
times, and its continental-arc front 2.0 times: both near the subducting share's 1.76 times
(0.347 against 0.197). The rates per km of each kind of arc now sit near Stern & Scholl's. So the
island arcs' crust (3.22 km³ a year against 1.5) is 1.7 times from the front's length and 1.3
times from the rate, and the front's length is the orientation above.

(Corrected 2026-10-24, the front lane, PLT-530: the lattice counts a straight front 1.04 times,
not 1.84. Earth's arcs are about 15,100 and 28,700 lattice km; the branch's island-arc front,
45,275 lattice km, is about 43,400 km, 3.0 times Earth's 14,500, and its continental-arc front
about 96,800 km, 3.5 times Earth's 27,500 (master's 5.6 and 2.1 times); the rates per km become
about 74 and 17 km³/km/Myr on the branch (master 40 and 28), below Stern & Scholl's 100 and 35.
So the island arcs' crust is about 3.0 times from the front's length and 0.74 times from the
rate, and the length is the whole boundary's 1.77 times Bird's times the subducting share's
1.76: "The front's length: why the plates are ragged (2026-10-24)" below.) On the plate-check table the
reported share of ocean-ocean subduction moves from 56.7 % to 29.5 % (after the first 100 Myr) and
from 65.6 % to 50.2 % (the first 50 Myr).

### What was tried and not kept

Each arm is the branch plus one change, the same ensemble (`earthlike_quick_16k`, 1000 Myr, seeds
1 to 3, both starts); land, drowned share, land above 2 km, the largest landmass and the island-arc
share of the land by `tools/review/continent-bisect.py` at the end (Earth's: 0.29, about 0.19, the
largest 0.167); the coast with high ground within 200 km by `tools/review/continents_census.py
maps` (Earth 0.104); the rest by `plate-geometry.py` and `deeptime plate-check`.

- *r1*: a contact with a collision on it no longer reverses by its crust (PLT-056's crust test only
  before the collision; the test on locked crust kept).
- *r3*: the reversal judges the overriding side by the crust its arc stands on: its overlap
  samples with continental crust within `arc_distance_km` through their plate, under a quarter
  (Stern 2004; PLT-371).
- *i1*: subduction initiation within 1,500 km of a trench instead of beyond 1,000 km (Crameri et
  al. 2020; PLT-062, PLT-064).
- *r3i*: r3 and i1.

| | master = branch | r1 | r3 | i1 | r3i | Earth |
|---|---|---|---|---|---|---|
| subducting share of the boundary | 0.347 | 0.343 | 0.339 | 0.344 | 0.345 | 0.197 |
| plates alive / over 0.0377 sr | 23.7 / 22.7 | 25.2 / 24.3 | 23.5 / 22.7 | 26.8 / 25.3 | 28.8 / 27.2 | 52 / 19 |
| collisions per 100 Myr | 1.63 | 2.83 | 2.22 | 1.73 | 2.05 | 2 to 6 |
| polarity reversals per 100 Myr | 17.0 | 12.4 | 11.6 | 19.1 | 16.3 | 4 since 104 Ma |
| initiations per 100 Myr | 7.0 | 7.3 | 7.1 | 12.7 | 13.7 | 13 since 104 Ma |
| median plate life, Myr | 129 | 121 | 121 | 81 | 75 | |
| land | 0.264 | 0.218 | 0.208 | 0.235 | 0.230 | 0.29 |
| drowned share of the continental crust | 0.368 | 0.376 | 0.370 | 0.371 | 0.373 | about 0.19 |
| largest landmass | 0.127 | 0.082 | 0.060 | 0.119 | 0.084 | 0.167 |
| land above 2 km, share of land | 0.273 | 0.305 | 0.335 | 0.324 | 0.314 | |
| island-arc share of the land | 0.429 | 0.509 | 0.491 | 0.465 | 0.522 | |
| coast with high ground within 200 km | 0.478 | 0.518 | 0.560 | 0.491 | 0.530 | 0.104 |
| continental volume, 10⁹ km³ | 6.60 | 5.50 | 5.26 | | 5.74 | |
| collisions lose, km³ a year | 1.72 | 2.35 | 2.00 | | 2.35 | 0.7 |
| plate checks met, of 45 | 31 | 32 | 30 | 26 | 26 | |

**The reversals (r1, r3).** Both raise the collisions into Earth's range (2.83 and 2.22 per 100
Myr) and cut the reversals by a third, and both lose land: 0.264 to 0.218 and 0.208, the largest
landmass 0.127 to 0.082 and 0.060, the continental volume 6.60 to 5.50 and 5.26 × 10⁹ km³. Per run
over 1000 Myr, r3 against master (the crust budget's channels, 10⁶ km³): erosion takes 923 more
(the belts stand higher: land above 2 km 0.273 to 0.335), resampling gives 658 less, and
subduction (2,087 more lost) and the belts (2,075 more stacked) cancel. The collision fronts lose
2.00 km³ a year (r1 2.35) against master's 1.72 and Earth's 0.7 (Scholl & von Huene 2009 as
Stern & Scholl 2010 give it): every collision the model adds costs land at 2.5 to 3.4 times
Earth's rate and builds high ground that erodes. The coastal high ground rises too (0.478 to 0.518
and 0.560): more collisions put more young orogen on more coasts. Raising the collision rate
toward Earth's is right; doing it before the collisions' losses and the high ground are on Earth's
rates loses the largest landmass, which is the gap Ben sees. Not kept: the reversal stays
PLT-056's, and PLT-371 records r3 as the law to switch on once those land.

**Initiation near trenches (i1).** The initiation rate reaches Crameri's (12.7 per 100 Myr against
13 since 104 Ma), but the new trenches split off slivers that the next resampling absorbs (plates
absorbed 7.7 to 12.4 per 100 Myr, median life 129 to 81 Myr, 16.3 % dead within 20 Myr against at
most 15), the old interior floor that the far rule consumed stays (floor older than 250 Myr 3.2 to
4.4 %, against at most 3), the supercontinent tenure falls from 126 to 39 Myr and the assembly
period from 431 to 185 Myr, and the plate checks fall from 31 to 26. The model has no law for a
trench to propagate along a margin or for a slab to roll back into old floor; with the
far-from-trench start removed, nothing recycles old floor away from the trenches. Not kept.

### PLT-330 and the arcs' fronts

PLT-330's convergence test (a front only where the plates converge across the boundary) is
neither redundant nor wrong. PLT-370 changes how a front is named, not which samples are fronts:
the counters' total (about 146,200 lattice km a step) and the subducting share (0.347) are the
same on master and the branch. What the arc-front task measured as an island-arc front 2.9 times
Earth's was 85,095 lattice km named by the front sample's crust; by the arc's crust it is 45,275,
1.7 times Earth's 14,500 km on the lattice's 1.84. The remaining excess is the subducting
share's, which PLT-330 could not move either (32.1 to 34.4 % there). (On the lattice's 1.04,
PLT-530, 2026-10-24: 3.0 times, the subducting share's 1.76 times the whole boundary's 1.77.)

### Against c24ff793 and master

The coordinator's figures from the continent bisect (c24ff793, seeds 1 and 2, the land's peak)
against master c790d1f3 and the branch (seeds 1 to 3, both starts, the same reader):

| | c24ff793 | master c790d1f3 | branch | Earth |
|---|---|---|---|---|
| land | 0.323 | 0.264 | 0.264 | 0.29 |
| drowned share of the continental crust | 0.31 | 0.368 | 0.368 | about 0.19 |
| largest landmass | 0.166 | 0.127 | 0.127 | 0.167 |
| island-arc share of the land | 0.55 | 0.429 | 0.429 | |

The branch moves no land: its change is a measurement. None of the plate-geometry changes
measured here brings the land or the largest landmass back toward c24ff793's: each one that
raises the collisions or the initiations lowers both (the table above). The fall since c24ff793
came with the crust budget's Earth rates and the arcs' basement and fronts (PLT-330, PLT-331;
the bisect, `tools/review/out-free/continent-bisect.md`), and the island-arc share fell with it
toward Earth's.

### The continental area's swing

The sea-level task found the sea swinging about a kilometre because the continental crust's area
swings within a run (sea-level.md). On the same ensemble, after the first 100 Myr (the dynamics
records' visible continental samples; `build/pg/swing.py` in the build log):

| | master = branch | r1 | r3 | i1 | r3i |
|---|---|---|---|---|---|
| continental crust, lowest and highest step (mean of the runs) | 0.312, 0.489 | 0.271, 0.471 | 0.272, 0.486 | 0.303, 0.495 | 0.277, 0.487 |
| the same, over all six runs | 0.276 to 0.523 | 0.256 to 0.520 | 0.225 to 0.525 | 0.268 to 0.533 | 0.241 to 0.528 |
| at 100 Myr and at the end | 0.457, 0.381 | 0.447, 0.317 | 0.458, 0.298 | 0.459, 0.342 | 0.458, 0.328 |
| trend, share of the planet per 100 Myr | −0.001 | −0.009 | −0.013 | −0.010 | −0.009 |
| range about the trend | 0.176 | 0.164 | 0.139 | 0.149 | 0.169 |
| the largest change within 50 Myr | 0.096 | 0.098 | 0.090 | 0.090 | 0.092 |
| the sea floor's median age, lowest and highest step of any run, Myr | 40 to 108 | 34 to 105 | 41 to 101 | 42 to 104 | 40 to 101 |

The continental crust's area moves by up to a tenth of the planet within 50 Myr in every arm. The
branch does not change that (its runs are master's); the reversal arms add a downward trend, their
collisions' losses, and keep the swing about it. The swing is not the plate geometry's to fix.

### The plates under the sea from the ocean's volume (after the sea-level merge)

The sea-level task (sea-level.md, open questions) reported that the plates answer its moving sea:
the crust budget's net loss −0.049 → −0.074 of the planet over 1,000 Myr, more collisions, the
supercontinent's tenure 231 → 21 Myr and the plate checks 29 → 27 and 31 → 26 of 45. Traced here on
the merged code (this branch with the sea-level branch at 975dd57e; master d53cc6c4's simulation is
the same code), the same ensemble as above, with scratch environment switches (never committed)
that pin the sea at one height for the whole run: for every reader (the climate's global), only for
the plate stage's reading (its settling and GDH1's dry share, PLT-353) or only for the surface's
(erosion, deposition and the carry, which come back to the plates' samples); and the old dampening
floor (`tectonics.ocean_depth=dampening`) under the moving sea and under a sea pinned at 0. Base is
master c790d1f3 (the old sea curve within about 150 m of the datum, the dampening floor), run on
seeds 4 to 6 as well to see its own spread.

| arm | the sea the plates read after 100 Myr, mean (first → last), m | crust budget's net, share of the planet | continental crust into contacts (lost there), 10⁹ km³ | collisions per 100 Myr | tenure: check (mean of runs) / median run, Myr | assembly period (check), Myr | plate checks of 45 |
|---|---|---|---|---|---|---|---|
| base, seeds 1 to 3 | −35 (74 → −34) | −0.042 | 16.2 (1.72) | 1.63 | 126 / 50 | 431 | 31 |
| base, seeds 4 to 6 | −36 (61 → −27) | −0.082 | 18.7 (2.11) | 1.50 | 46 / 37 | 255 | 28 |
| merged, seeds 1 to 3 (volume sea, GDH1) | +259 (97 → 394) | −0.087 | 18.6 (1.97) | 1.72 | 25 / 18 | 144 | 28 |
| merged, seeds 4 to 6 | +273 (144 → 462) | −0.100 | 19.1 (2.37) | 1.67 | 19 / 10 | 289 | 29 |
| volume sea over the old floor | +468 (612 → 548) | −0.095 | 18.2 (2.13) | 1.83 | 16 / 18 | 125 | 28 |
| both changes off: the old floor, the sea pinned at 0 | 0 | −0.061 | 17.0 (2.12) | 1.75 | 41 / 23 | 309 | 30 |
| sea pinned at −250 m (every reader) | −250 | −0.101 | 17.8 (1.94) | 1.65 | 84 / 45 | 341 | 30 |
| sea pinned at 0 | 0 | −0.069 | 18.5 (2.02) | 1.62 | 37 / 12 | 258 | 29 |
| sea pinned at +300 m | +300 | −0.089 | 19.5 (2.30) | 1.45 | 20 / 19 | 216 | 30 |
| only the plates' reading at −250 m | −250 | −0.090 | 19.6 (2.31) | 1.75 | 20 / 16 | 103 | 28 |
| only the surface's reading at −250 m | +409 (201 → 837) | −0.085 | 18.7 (2.02) | 1.82 | 57 / 63 | 340 | 28 |

What it says:

1. *Most of the response is base's own spread.* Base on seeds 4 to 6 loses as much crust (−0.082),
   moves as much of it into contacts (18.7 against 16.2), holds its supercontinents as briefly
   (46 Myr; 37 at the median run) and meets as few checks (28) as the merged code on seeds 1 to 3.
   Seeds 1 to 3 were a kind draw for base: one run (supercontinent, seed 1) held its first
   supercontinent 558 Myr and carries the check's mean. The tenure is a mean of one figure per
   run, each from a handful of assemblies, between 4 and 558 Myr.
2. *What is left after twelve runs each.* The crust's net loss: −0.062 (base) against −0.093
   (merged), 10 of the 12 seed-and-start pairs losing more. The tenure's fall (a median run of
   50 Myr against 13, one-sided Mann-Whitney p about 0.04) does not hold up: the twelve runs
   with the sea at the datum (below) give 17.5, the 35 with the sea off it 24, and neither differs
   from base's twelve beyond z 1.3. The collisions (1.6 → 1.7 per 100 Myr) do not move beyond the
   runs' spread.
3. *Not the sea's motion and not GDH1, but the sea standing off the datum.* The merged code with
   both of its changes off (the old floor, the sea pinned at 0) loses −0.061 and meets 30 checks;
   with GDH1 on and the sea still at 0 it loses −0.069 and meets 29. Those twelve runs (−0.065)
   are base's twelve (−0.062; Mann-Whitney z +0.12). Every arm whose sea stands away from 0,
   below it (−250 m) or above it (+300 m, or the volume sea's +259 to +468 m), moving or pinned,
   under either floor, and whichever reader is moved off 0 (the plates' settling and dry share,
   or the surface's base level), loses more: −0.085 to −0.101, −0.091 over its 36 runs (against
   the twelve at the datum, one-sided z +2.65, p about 0.004; in four of the six seed-and-start
   pairs five or six of the six arms lose more than the datum arms' mean). Its assembly period is
   shorter too (median run 145 Myr against 236 at the datum, z +2.31). The volume sea over the
   old floor (−0.095) is no better than with GDH1 (−0.087), so the sea-level task's reading ("the
   moving sea, not GDH1") holds for GDH1 and not for the motion. That the sign of the offset does
   not matter says the loss is not more or less water over the margins but a mismatch: code that
   still takes the datum for the sea while the rest reads the sea. One candidate, not tested
   here: the force balance's column energy (`forces::column_energy`, FRC-006 and FRC-014), which
   loads a continental column with water below 0, not below the sea.
4. *Where the crust goes.* In every arm with the sea-level code more continental crust crosses
   into contacts (17.0 to 19.6 against 16.2 × 10⁹ km³ over the run) and more of it is lost there
   (1.94 to 2.37 against 1.72); the crust budget's subduction part takes 0.69 to 0.80 of the
   planet against 0.62. Base on seeds 4 to 6 does the same (18.7, 2.11, 0.72), and the arms at
   the datum sit inside that range (17.0 and 18.5; 0.69 and 0.75), so these totals do not show
   the split the net loss shows.

The continental crust's swing under the merged code, after the first 100 Myr (the crust budget's `cb_t` lines, one per step, in the run's report):
lowest and highest step 0.290 and 0.475 of the planet (base 0.312 and 0.489), the largest change in
50 Myr 0.089 (0.096), a trend of −0.008 per 100 Myr (−0.001); pinned at 0 0.292 and 0.491 (0.089
in 50 Myr), at +300 m 0.296 and 0.487 (0.096), at −250 m 0.303 and 0.480 (0.087); both changes off 0.293 and 0.475 (0.085, −0.005 per 100 Myr);
the volume sea over the old floor 0.299 and 0.471 (0.095, −0.009); on seeds 4 to 6 base 0.302 and
0.484 (0.093, −0.0075) and the merged code 0.283 and 0.483 (0.098, −0.006). The swing is the plate
model's, whatever the sea does, and the downward trend is the crust's net loss, as steep in base on
seeds 4 to 6 as under the merged code.

The continent table under the merged code (seeds 1 to 3, both starts, the bisect's reader, as
"Against c24ff793 and master" above):

| | c24ff793 | master c790d1f3 = this branch before the merge | merged (master d53cc6c4's simulation) | Earth |
|---|---|---|---|---|
| land | 0.323 | 0.264 | 0.269 | 0.29 |
| drowned share of the continental crust | 0.31 | 0.368 | 0.357 | about 0.19 |
| largest landmass | 0.166 | 0.127 | 0.159 | 0.167 |
| island-arc share of the land | 0.55 | 0.429 | 0.423 | |
| continental crust, share of the planet | | 0.381 | 0.336 | 0.37 to 0.45 (the plate check) |

The merge adds land and the largest landmass with less continental crust: the sea from the
ocean's volume stands where the crust leaves room, as sea-level.md says. The sea-level task's own
table (30c4d664 → fad59a5e on seeds 1 and 2: land 0.257 → 0.268, largest 0.112 → 0.163, drowned
0.374 → 0.373) agrees.

### What is left, and for whom

1. *The boundaries' orientation* (Ben's: how plates are made and moved, big scope): the
   subducting share (0.347 against 0.197) and with it the island-arc front (1.7 times Earth's)
   and the island arcs' crust (3.22 against 1.5 km³ a year). It needs spreading boundaries that
   segment into ridges and transforms and trenches that face their convergence.
2. *The collision rate* (1.63 per 100 Myr, target 2 to 6): PLT-371's reversal is sourced and
   measured (2.22 per 100 Myr) and waits on the collisions' losses (1.72 km³ a year against 0.7)
   and the high ground (the high-ground work). Switch it on after those, and measure the land
   again. Measured again in "The crust flux into collision fronts (2026-10-21)" below: the
   collisions' losses per km of front are Earth's (their excess is the length of front in
   collision, the plates'), PLT-371 is in the code as a switch, and on top of PLT-400 it keeps the
   land but does not reliably reach 2: 2.00 per 100 Myr at the median of 12 runs (p = 0.09) on
   master 49492ce3, 1.85 (p = 0.32) on master df40ce80, by longer fronts in collision and at a
   cost in crust, so it stays off. Welding collisions sooner (the weld at 10 mm/yr) reaches 3.00
   but drowns the continents (land 0.190). The rate waits on the length of front in collision
   (item 4) and on what the check counts as a collision (that section's list, item 3).
3. *Where subduction starts* (Crameri): needs a law for trenches to propagate and slabs to roll
   back before initiation can move near trenches; a plate reorganisation is the open law (Ben's,
   as in the sections above).
4. *The plate count*: the lattice's floor (0.0377 sr at 100,000 samples); a finer lattice or a
   small-plate law, not a constant. It is also behind the collision fronts' length ("The crust
   flux into collision fronts" below): with 24 plates against Bird's 52, continents meet along
   whole margins, 28,800 km of front in collision against Earth's 6,000 to 12,000 km of suture.
5. *The island-arc share of the land* (0.43): "a few per cent" (the section above, from memory)
   is not a figure Earth's sources give. By Stern & Scholl 1.5 of the 2.7 AU of crust grown at
   convergent margins is intra-oceanic arc, and the census tags any land with 1 km or more of
   island-arc crust as island arc. A share of the crust's volume against Stern & Scholl's
   Table 1 would be the measure with a source behind it.
6. *The sea off the datum* (the sea level's): a sea standing away from 0, above or below, loses
   about 0.026 more of the planet's continental crust over 1,000 Myr than a sea at 0 and shortens
   the assembly period (the section above). Find the code that still takes the datum for the
   sea; `forces::column_energy` (FRC-014: "h above sea level", computed from 0) is the first to
   test, by an arm that feeds it the sea. Done ("The sea off the datum" below): the column
   energy, the wedge floor (PLT-380) and the Andean mark now take the sea, and the measurement
   counts from it. On seeds 1 to 6 none of it moves the crust (net −0.085 against master's −0.095
   over 1,000 Myr, p 0.60), and a sea held at the datum loses as much (−0.099): the 0.026 was
   seeds 1 to 3's draw. The drowned crust is the sea's height (0.235 with the sea at the datum).
7. *Plate checks that are a mean of runs* (the supercontinent's tenure above all): one long run
   carries the mean (base: 558 Myr in one run of six; 126 Myr on seeds 1 to 3 against 46 on seeds
   4 to 6). Compare arms on twelve runs or more, and by the runs' median, before reading a change
   in them as the code's.

## The high ground's extent: post-orogenic collapse (2026-10-21)

The coordinator's task after item 2 of "White patches, drowned continents and coastal high ground"
above, on Ben's two complaints: "massive patches that seem to have absurd elevaiton appear to be
forming which don't feel like normal mountain ranges" and "mountains seem very prone to forming
directly bordering oceans". Targets on `earthlike_quick_16k` (1000 Myr, seeds 1 to 3, both starts,
the final frame): land above 2 km at most 0.12 of the land (Earth 0.055 ice-free, about 0.10 with
the ice sheets' tops, HYP-004 and HYP-041), coast with land above 2 km within 200 km at most 0.15
(Earth 0.104, PLT-360 ff.), land under ice outside a glaciation at most 0.15; and not to break the
land (0.26 to 0.32 of the planet), the largest landmass (at least 0.12), the continental crust
under the sea (at most 0.25, failing before) and the collisions (2 to 6 per 100 Myr, the plates').
Ledger PLT-390 and PLT-391. Code: `plates/collapse.rs` and `params.rs`; nothing in the plates'
dynamics, subduction or checks. Both laws are measured and **not on** (below).

### What the high ground is

On the base (master d53cc6c4; final frame, mean of the six runs) the land above 2 km is 0.256 of
the land and the coast with high ground within 200 km 0.387. Taken apart by the column's state on
the budget's maps:

- *Collapsed crust that stays high.* The collapse thins a quiet column only to T_c = 50 km
  (PLT-256), and on the Airy line (525 m at 35.1 km, 142.9 m per km, PLT-278 ff.) 50 km stands at
  2.65 km: a column the collapse has finished with is still high ground, and stays so for as long as
  it is not eroded. Quiet crust thicker than 51 km is 0.27 to 0.44 of the base's land above 2 km.
- *Active belts.* Crust raised by an orogeny within the last 10 Myr is most of the rest (0.50 to
  0.74 of the high ground once the first is dealt with), on crust of 60 to 68 km at the median and
  about 200 km from the sea at the median. The collisions lose crust at 2.5 to 3.8 times Earth's
  0.7 km³ a year (the crust budget's sections above) because 3 to 4 times Earth's crust crosses
  their fronts (the agent's estimate from the budget; measured in "The crust flux into collision
  fronts" below, the crossing per km of front is Earth's, and what is 2.4 to 4.8 times Earth's is
  the length of front in collision); 0.95 of what crosses stays and builds the
  belts (PLT-242). How much crosses is the plates' motion (`plates/dynamics.rs`), outside this task.
- *Plateau interiors with no room.* Quiet thick crust in a plateau's middle finds no thin crust
  within the collapse's 200 km reach (PLT-257): its nearest room is 400 to 440 km away at the
  median, so it does not collapse at all.

Earth's collapsed orogens end far thinner than 50 km. The Variscan root of 55 to 70 km thinned from
about 315 to 290 Ma, as its thickening ended (Hillenbrand & Williams 2022, abstract), "flattened by
lateral crustal flow and gravitational collapse" to "a relatively flat Moho at ca 30-32 km depth"
where it flattened, and the Bohemian Massif is "still fairly thick (ca. 35 km)" (Puziewicz et al.
2025, abstract). The Basin and Range extended 46 % ± 8 % from 54 ± 6 km (Long 2019, abstract):
about 37 km. The Cyclades are on 26 km (Jolivet & Brun 2010). What stays thick is old and cold: the
Urals keep 50 to 56 km some 300 Myr after their collision (Carbonell et al. 2000, PLT-256's
source). And it is the heat that decides: the locus of extension is set by the crust's thickness,
"while its timing is governed by the thermal state of the lithosphere at the time of thickening"
(Wernicke et al. 1987). The Caledonides went from collapse to rifting in 30 to 40 Myr
(Peron-Pinvidic & Osmundsen 2020).

### The laws

**PLT-390, the hot window** (`collapse_hot_myr`). A quiet column whose last orogeny is younger than
W thins towards the normal column T_n (35.1 km) instead of T_c, at the collapse's own rate
(PLT-254, PLT-255); older crust, and crust no orogeny raised, keeps T_c. W = 40 Myr: the 10 Myr
quiet time (PLT-259) and the Variscan's 25 Myr of thinning, rounded up within the Caledonides'
30 to 40 Myr.

**PLT-391, continental room first** (`collapse_continent_first`). The room the spread fills
(the collapse's and the thickest crust's, PLT-277) is ordered continental before ocean floor,
nearest first within each, so a plate's thinned continental crust is filled before its ocean floor
turns continental. No source gives the order; it is the agent's (the lower crust flows within hot
continental crust, a channel that "travels at most 150 km into the foreland", Rey et al. 2010).

### What they do

Final frame, mean of the six runs (pooled plate checks of 45):

| | land | above 2 km | coast with high ground | under ice | continental crust under the sea | largest landmass | collisions per 100 Myr | plate checks |
|---|---|---|---|---|---|---|---|---|
| target | 0.26 to 0.32 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | ≤ 0.25 | ≥ 0.12 | 2 to 6 | |
| base | 0.269 | 0.256 | 0.387 | 0.183 | 0.358 | 0.159 | 1.72 | 28 |
| PLT-390 (W 40) | 0.220 | 0.194 | 0.309 | 0.142 | 0.484 | 0.104 | 1.72 | 27 |
| PLT-390 and PLT-391 | 0.256 | 0.185 | 0.303 | 0.166 | 0.415 | 0.100 | 1.65 | 27 |
| the same, reach 600 km | 0.226 | 0.145 | 0.281 | 0.153 | 0.494 | 0.089 | 1.70 | 29 |

One frame per run is noisy (the largest landmass runs from 0.04 to 0.26 between runs of one arm),
so the arms were also averaged over the budget's maps of the last 500 Myr (every saved frame) and
the crust budget's lines over the last 600 Myr:

| | land | above 2 km | largest landmass | under the sea | continental crust, share of the planet | sea over the datum, m | mean continental crust, km | its volume, 10⁹ km³ |
|---|---|---|---|---|---|---|---|---|
| base | 0.299 | 0.288 | 0.165 | 0.321 | 0.346 | 281 | 36.3 | 6.04 |
| PLT-390 | 0.263 | 0.181 | 0.126 | 0.421 | 0.403 | 497 | 34.2 | 6.68 |
| PLT-390 and PLT-391 | 0.276 | 0.167 | 0.134 | 0.402 | 0.399 | 557 | 34.9 | 6.71 |

The high ground falls by 0.06 to 0.07 of the land on the final frame and 0.11 to 0.12 over the
frames, the coast with high ground by 0.08, the ice to 0.14 to 0.17, and the continents get their
area back (0.35 to 0.40 of the planet; Earth 0.41,
HYP-012) and keep more of their volume (less of it founders above 70 km, PLT-258). But the land
falls by 0.01 to 0.05 (0.02 to 0.04 over the frames), the largest landmass by 0.03 to 0.06 and the drowned share rises by 0.06 to
0.13. The cause is the sea: crust spread over ocean floor displaces the planet's water (the sea is
the planet's water filling its hypsometry, ISO-085), which stands 200 to 280 m higher, and crust
thinned to T_n stands at z_0 = 525 m over the datum, so with the sea 500 to 560 m over it the
collapsed belts end at the shore. The model's continents are thin for their area (34 to 36 km on
average against Earth's 41.1 km, Christensen & Mooney 1995 as in PLT-256), and the sea stands off
the datum ("The plate geometry", "What is left, and for whom", item 6). Neither is this task's.

PLT-391 on top of PLT-390 keeps 0.013 more land over the frames (under the 0.03 noise) and 0.014
less high ground; a longer reach (400 or 600 km, PLT-257 unchanged, measured only) lowers the high
ground further but floods more (final land 0.227 and 0.226).

### What was tried and not kept

| arm (final frame, mean of six) | land | above 2 km | coast with high ground | under the sea | largest | note |
|---|---|---|---|---|---|---|
| W 50 Myr | 0.217 | 0.159 | 0.272 | 0.520 | 0.119 | more of the same |
| W 50, reach 600, the thickest crust filling only to T_n | 0.224 | 0.188 | 0.299 | 0.482 | 0.107 | the founder's fill (PLT-277) to T_n took land |
| the cordillera's ceiling 1800 m (PLT-050, from 3000) | 0.241 | 0.231 | 0.334 | 0.422 | 0.094 | the high ground 0.03 lower, the land too |
| W 40 and the ceiling 1800 m | 0.222 | 0.164 | 0.286 | 0.504 | 0.092 | |

The reach at 600 km alone, over the last 600 Myr: land 0.298, above 2 km 0.256 (base 0.296 and
0.286). The cordillera's 3000 m (PLT-050, `calibrated`, uncited) is a lead, not a fix: Profeta et
al. 2015's eight continental arcs stand on 34.6 to 65 km of crust, 44.2 km on average and 40 km at
the median (about 1,800 m and 1,200 m on the Airy line).

### Why it is not on

Both laws are in the code with W = 0 and the order off in the default and every built-in preset,
so the planets are the same bit for bit. On, they trade Ben's high ground for his drowned continents
(the drowned share was already over its target). They are ready for the sea at the datum: once the
sea stands at 0 (or the crust's budget is Earth's), run the PLT-390 and PLT-391 arm again on the
same seeds and switch it on if the land holds. Ben's call when.

### What is left, and for whom

1. *The sea off the datum* (the sea level's; a peer task is on it): it decides whether the hot
   window can go on. Measured ("The sea off the datum" below): with the plate model reading the
   sea, W 40 still drowns the continents (0.510 under the sea, land 0.209), so it stays off until
   the sea stands lower against the continents (more continental crust, or Earth's crust budget).
2. *The active belts* (the plates'): 0.95 of the crust that crosses a collision front stays in
   the crust (PLT-242, `plates/belts.rs`, on Stern & Scholl 2010), where the India-Asia balance has
   about half of the colliding crust gone into the mantle (Ingalls et al. 2016, abstract; disputed by van Hinsbergen et al.
   2017, and Yakovlev & Clark 2014 conserve it if the crust before the collision was 23 to 29 km).
   A lower share alone would lose still more crust (the losses are already over Earth's because
   too much crosses); less crossing, from fewer or shorter collisions, is the plates' work on
   `plates/dynamics.rs`, and the share is worth a second look after it. Measured in "The crust
   flux into collision fronts (2026-10-21)" below: per km of front the crossing and the losses are
   Earth's, and 0.95 is in line with Stern & Scholl at the model's crossing (PLT-242 now
   `verified`); the excess is the length of front in collision (28,800 km against 6,000 to 12,000).
   Ingalls' half as the share (arm rs50, with PLT-400) takes land above 2 km from 0.277 to 0.142
   but the land from 0.257 to 0.204, the largest landmass from 0.133 to 0.052 and the drowned
   continental crust from 0.348 to 0.514: rejected. The belts' height is left to the front's
   length and the collisions' life.
3. *The cordillera's height* (PLT-050): a sourced value from Profeta et al. 2015 in place of the
   calibrated 3000 m; measured above, it does not move the high ground by itself.
4. *Plateau interiors*: thick crust with no room within reach does not collapse. A lower-crustal
   flow that carries crust farther than the nearest room (PLT-300's flow does not reach) would
   be a flow solve, not a constant.

## The sea off the datum (2026-10-21)

Item 6 of the plate geometry's "What is left, and for whom" above: a sea standing off elevation 0, above or below it,
lost about 0.026 more of the planet's continental crust over 1,000 Myr than a sea at 0 and
shortened the assembly period, which pointed at code that still took the datum for the sea. This
section lists every such place in `deeptime-core`, feeds the step's sea to the ones that model
something physical, and measures the change against master 44032ad9 on twelve runs an arm.

### The two frames

The plate model's heights stand in a datum frame: the ridge crest, GDH1's floor (PLT-351), the
continents' Airy heights and the isostatic heights of the settling and the collapse are measured
from elevation 0 on a planet without water. The sea S is a height in the same frame: the ocean's
volume fills the ground's hypsometry once a step (sea-level.md), and the plates read the last
step's (`plates/mod.rs` `sea_m`: the climate's `climate/sea_level_m`, else the surface's, else 0).
The surface, the climate, the rivers, the bake, the strata and the deposits already took their
heights from the sea (a sweep of those modules found no reader of 0 as the sea); the plate
model's own readers did not.

### Where the code took 0 for the sea

| site | what it does | reached by the moving sea in the shipped Earth-like presets | change | ledger |
|---|---|---|---|---|
| `plates/forces.rs` `column_energy` (all its callers go through `Coeffs::energy` in `forces::step`: the edge energies and the collision's energy change) | the land's potential energy against the ridge: the ridge push and the collision's resistance | yes, every step, at every edge between land and ocean | h = z − S, the water switch at S, H = 35 km + 6 S (below) | FRC-013, FRC-014 |
| `plates/belts.rs` `WEDGE_FLOOR_M` (`Reach::new`) | the lowest ground the orogenic wedge's distance field is followed down to, −500 m | yes, every collision | 500 m below the sea | PLT-380 (new) |
| `plates/subduction.rs` `fronts_finish` (`andean`) | an oceanic sample raised at a front takes the Andean orogeny mark when it stands above 0 | yes, every subduction front | above the sea | PLT-060 |
| `plates/belts.rs` `stack_front`, `plates/subduction.rs` `fronts_finish` (the `ceiling` of uplift) | the uplift's isostatic ceiling, 1 − z/z_c | no: `uplift_ceiling_m` is 0 (off) in the shipped presets | z − S | PLT-041 |
| `plates/pools.rs` `surface_step` | Cortial's continental erosion, ε_c z/z_c | no: `continental_erosion_mm_yr` is 0 in the shipped presets (the surface stage erodes) | the height above the sea | PLT-127 |
| `plates/dynamics.rs` (`arc_stats`, `arc_heights`, the step record's land), `plates/belts.rs` (`emerged`, the belt's `height_m`), `plates/check.rs` (the land gain) | measurement: the land share, the arcs' heights and emergence, the belts' heights, the plate check's land gain | yes, measurement only | from the sea; the land gain with the sea held (below) | PLT-151, PLT-215 to PLT-219, CHK-009 |
| `provinces.rs` `cells()` | the province map's heights | output only | less the surface's sea, else the climate's, else 0 | PRV-012 |
| `plates/crust_budget.rs` snapshot (`cb_t`'s `dry`), `tools/review/continent-budget.py` | the continental crust's dry share, which the tool drew against Earth's 0.71 above the sea (HYP-012) | measurement only, with `DEEPTIME_CRUST_BUDGET` | `dry_sl` at the sea beside `dry` at the datum; the tool draws `dry_sl` against Earth | PLT-302 |

Left as they are:

- `plates/crust_budget.rs` (`bits`, `stocks`): the dry bit and the dry stock at 0, which the
  parts and the moves (`drown`, `emerge`) book. The budget prints the land at the datum (`land0`)
  beside the land above the sea (`land`), and now the dry share at both; the bookings stay in the
  datum frame, a census of the solid column's moves, not a law. The thickness bands' `dry` was
  already at the sea.
- `params.rs` (a preset's ridge must stand below 0 and its highest ground above): a validation of
  the preset, in the datum frame.
- The height cap `max_elevation_m`, the settling (`settle.rs`, which reads the sea for the water
  it loads, ISO-078), the ocean–continent transition (`transition.rs`), the L1 resampling
  (`l1.rs`) and the orogenic collapse (`collapse.rs`): heights of the solid column, in the datum
  frame on purpose.
- The hotspot uplift's taper (`hotspots.rs`, PLT-114: the swell fades to nothing at 3 km on
  oceanic crust and 1.5 km on continents, the room counted from 0): the moving sea reaches it, but
  it is a cap on the swell and its edifice, a height of the solid column (a volcano's height is
  set by its edifice and the plate under it, not by the shore), and it is uncited; left in the
  datum frame.
- `surface/amplify.rs` `Config.sea`: already the step's sea; only its doc did not say the frame.

### The column energy taken from the sea

The force balance's column energy (design §3.2, FRC-014) is the vertical-stress integral of an
Airy-compensated column against a reference, Turcotte & Schubert's lithostatic balance; the
reference is a column whose top stands at the sea. Until now the code put that shore at 0: under
a sea at +300 m a shelf at +100 m, under 200 m of water, was weighed as dry land 100 m high, and
under a sea at −250 m a coastal plain at −100 m, 150 m above the water, was weighed as a shelf
under water.

Taken from the sea S: h = z − S, the water switch at h = 0, and the reference crust H thick enough
to hold a dry column at S, H = 35 km + S ρ_m / (ρ_m − ρ_c) = 35 km + 6 S (ρ_m 3,300, ρ_c 2,750
kg/m³). Then for a dry column U(z, S) = U(z, 0) − U(S, 0): the difference between two dry columns
does not change with the sea (the rock is the same whatever water lies elsewhere), the reference
shore moves with the sea, and at S = 0 the formula is the datum's, bit for bit. The ridge crest
stays where the datum frame puts it (FRC-013). Against the ridge a sea at +300 m lowers every
continent's energy by U(300 m, 0), about 2.9 × 10¹¹ N/m; one at −250 m raises it by about
2.3 × 10¹¹ N/m (a belt 5 km high: 6.7 × 10¹² N/m). Not taken: h = z − S on a fixed 35 km crust
(a high sea would weaken the push of every dry belt by ρ_c g S ρ_m / (ρ_m − ρ_c) per metre of
relief though the columns are the same), and the water switch alone (the reference shore stays at
0, the fault itself). Tested in `forces.rs` `the_land_s_column_energy_is_taken_from_the_sea`.

### The land gain with the sea held

Counting land above the sea made the plate check's land gain (CHK-009, PLT-151: the most land
gained in one step after the first 10 Myr, target at most 1.5 % of the planet a 2 Myr step) count
a falling sea's new shore as land made: on the first branch arm its median run rose from 0.84 to
2.00 % and failed. The check is about land the ground's motion makes, so the step record now
carries `land_held`, the step's land above the sea of the step before, and the gain is
`land_held` less the step before's land (`check.rs`, test `land_gain_holds_the_sea`).
With the sea held the branch's median run gains 1.08 % (master 0.84; two-sided Mann-Whitney p
0.04), and two of its twelve runs are over the target (1.71 and 1.80; master none). The rest of
the rise is where the line is drawn: with the sea pinned at the datum (the pin0 arm below), where
master's count and the shore are the same line, the median run gains 1.02 % (against master 0.84,
p 0.05). The surface planes land down to its base level, so more ground stands near the shore
than near a datum 200 to 450 m below it, and the same motion carries more of it across. Counted
at the shore is what the check means.

### Against master on twelve runs

**Measured** (VM, admitted): `earthlike_quick_16k`, 1000 Myr, seeds 1 to 6, the supercontinent and
scattered_continents starts, 2 threads, the crust budget on (`DEEPTIME_CRUST_BUDGET=1`), by
`tools/review/force-balance-ensemble.sh`; per run `deeptime plate-check` and
`tools/review/continents_census.py fields` on the final frame's dumped fields (a scratch driver,
not committed). Twelve runs an arm, compared by the runs' median and a two-sided Mann-Whitney test
over the runs. The arms:

- *master*: 44032ad9. Master 49492ce3 (the high ground's laws, off) gives the same crust budget
  on supercontinent seed 1, line for line.
- *branch*: this branch. After the merge of 49492ce3 it gives the same budget on supercontinent
  seed 1, line for line.
- *pin0*: the branch with the climate's sea pinned at 0 for every reader (a scratch switch, not
  committed). The Earth-like presets' steering still nudges it: 0 to 93 m, median 0 after the
  first 100 Myr.
- *rest*: the branch with the column energy back on the datum (a scratch switch): the wedge
  floor, the Andean mark and the measurement at the sea.
- *wd*: the branch with the wedge floor back on the datum (a scratch switch): the column energy,
  the Andean mark and the measurement at the sea.
- *master + W 40* and *branch + W 40*: `--set tectonics.collapse_hot_myr=40` (PLT-390, the high
  ground's hot window), the coordinator's follow-up.

V1 item 1's seven measures on the final frame (the census; collisions from the plate check),
median of twelve runs, the p against master in brackets:

| arm | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | continental crust's net change over 1,000 Myr | criteria met per run: median, mean | sea over the datum, median after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|
| target | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 | | of 7 | |
| master | 0.257 | 0.133 | 0.348 | 0.277 | 0.381 | 0.172 | 1.60 | −0.095 | 1, 1.58 | 292 / 455 |
| branch | 0.247 (0.19) | 0.104 (0.18) | 0.414 (0.053) | 0.248 (0.58) | 0.369 (0.53) | 0.170 (0.58) | 1.65 (1.0) | −0.085 (0.60) | 1, 1.25 (0.38) | 210 / 474 |
| pin0 | 0.353 (< 0.001) | 0.233 (0.007) | 0.235 (< 0.001) | 0.259 (0.80) | 0.387 (0.69) | 0.180 (0.84) | 1.80 (0.60) | −0.099 (0.69) | 2, 2.42 (0.016) | 0 / 8 |
| rest | 0.248 (0.31) | 0.124 (0.54) | 0.419 (0.13) | 0.234 (0.061) | 0.339 (0.028) | 0.156 (0.13) | 1.55 (0.60) | −0.081 (0.13) | 2, 1.33 (0.76) | 325 / 608 |
| wd | 0.234 (0.10) | 0.095 (0.17) | 0.466 (0.004) | 0.185 (0.007) | 0.314 (0.026) | 0.143 (0.18) | 1.60 (0.93) | −0.066 (0.026) | 1, 1.50 (0.90) | 320 / 763 |
| master + W 40 | 0.192 (0.002) | 0.065 (0.035) | 0.589 (0.001) | 0.170 (0.001) | 0.285 (0.001) | 0.143 (0.012) | 1.70 (0.43) | −0.011 (< 0.001) | 2, 1.42 (0.95) | 460 / 734 |
| branch + W 40 | 0.209 (0.017) | 0.099 (0.11) | 0.510 (0.007) | 0.163 (< 0.001) | 0.285 (0.002) | 0.128 (0.006) | 1.55 (0.77) | −0.032 (0.001) | 2, 1.75 (0.67) | 361 / 584 |

Per run, master / branch (supercontinent seeds 1 to 6, then scattered_continents seeds 1 to 6):

| run | land | largest | under the sea | above 2 km | coast with high ground | under ice | collisions | crust, net |
|---|---|---|---|---|---|---|---|---|
| super-1 | 0.223 / 0.260 | 0.136 / 0.091 | 0.508 / 0.387 | 0.281 / 0.235 | 0.358 / 0.399 | 0.272 / 0.103 | 1.1 / 1.8 | −0.042 / −0.082 |
| super-2 | 0.257 / 0.244 | 0.080 / 0.120 | 0.403 / 0.445 | 0.225 / 0.135 | 0.386 / 0.353 | 0.140 / 0.152 | 1.5 / 2.4 | −0.070 / −0.076 |
| super-3 | 0.295 / 0.269 | 0.212 / 0.109 | 0.306 / 0.324 | 0.212 / 0.295 | 0.374 / 0.380 | 0.140 / 0.217 | 2.4 / 1.4 | −0.092 / −0.101 |
| super-4 | 0.227 / 0.263 | 0.058 / 0.152 | 0.437 / 0.418 | 0.178 / 0.159 | 0.296 / 0.284 | 0.120 / 0.093 | 1.6 / 2.0 | −0.074 / −0.061 |
| super-5 | 0.278 / 0.260 | 0.130 / 0.164 | 0.353 / 0.322 | 0.312 / 0.351 | 0.278 / 0.465 | 0.204 / 0.182 | 1.8 / 1.5 | −0.058 / −0.104 |
| super-6 | 0.253 / 0.250 | 0.093 / 0.055 | 0.396 / 0.412 | 0.248 / 0.184 | 0.384 / 0.292 | 0.163 / 0.129 | 1.5 / 2.1 | −0.077 / −0.079 |
| scatt-1 | 0.287 / 0.249 | 0.257 / 0.127 | 0.306 / 0.428 | 0.312 / 0.261 | 0.432 / 0.359 | 0.194 / 0.174 | 1.6 / 1.4 | −0.101 / −0.069 |
| scatt-2 | 0.301 / 0.222 | 0.175 / 0.061 | 0.282 / 0.417 | 0.194 / 0.372 | 0.374 / 0.417 | 0.172 / 0.242 | 1.2 / 1.4 | −0.097 / −0.107 |
| scatt-3 | 0.249 / 0.225 | 0.091 / 0.040 | 0.343 / 0.527 | 0.313 / 0.065 | 0.399 / 0.209 | 0.177 / 0.167 | 2.5 / 1.9 | −0.119 / +0.007 |
| scatt-4 | 0.258 / 0.240 | 0.186 / 0.144 | 0.283 / 0.394 | 0.363 / 0.308 | 0.476 / 0.308 | 0.206 / 0.190 | 1.3 / 1.4 | −0.152 / −0.106 |
| scatt-5 | 0.280 / 0.243 | 0.148 / 0.099 | 0.287 / 0.420 | 0.287 / 0.208 | 0.402 / 0.381 | 0.171 / 0.106 | 2.1 / 1.1 | −0.124 / −0.088 |
| scatt-6 | 0.233 / 0.246 | 0.097 / 0.090 | 0.375 / 0.361 | 0.273 / 0.335 | 0.377 / 0.409 | 0.132 / 0.174 | 1.7 / 1.9 | −0.115 / −0.106 |

The plate checks: master 29 of 45 at the median, the branch 29 (pin0 29, rest 27, p 0.029; the two
W 40 arms 29 and 28). The supercontinent's tenure 13 and 24 Myr (p 0.40), the assembly period 196
and 142 Myr (p 0.53; two branch runs make no second assembly), pin0 22 and 264 Myr (p 0.46 and
0.27).

**What moved.** Only what measures from the sea: the plate check's land (its median 0.330 of the
planet above 0 m, 0.270 above the sea, p < 0.001), its land gain (0.84 to 1.08 % a step, p 0.04,
above), and the arcs' heights (the island arcs' 90th percentile −2,498 to −2,289 m, p 0.035; the
continental arcs' median 2,048 to 1,675 m and the island arcs' emergence 23.7 to 22.0 %, p < 0.001). None of
the seven measures, the crust or the plates' timing moved at p < 0.05.

**What did not.** The crust: master loses 0.095 of the planet's continental crust over the 1,000
Myr at the median run, the branch 0.085 (p 0.60), and a sea held at the datum 0.099 (pin0, p
0.69). The 0.026 the section above found was a draw of seeds 1 to 3: there, mean of six runs,
master loses 0.087 and pin0 0.070; on seeds 4 to 6 master 0.100 and pin0 0.099. Runs of one arm
spread by 0.03 to 0.04, so twelve runs against twelve resolve about 0.04, and 0.026 would need
about thirty an arm. The column energy is not measurable either: the branch against rest differs
in nothing at p < 0.19 but the plate checks met (29 against 27, p 0.048).

**The drowned crust is the sea's height.** Under the sea at the datum (pin0) the continental crust
under the sea is 0.235 (target ≤ 0.25, met; master 0.348), the largest landmass 0.233 and the land
0.353 (over its 0.32 ceiling), with the crust lost the same. The plate model's own count at the sea
agrees (the crust budget's thickness bands, Σ a × dry, the branch's `dry_sl`): master 0.313 on the
final frame, pin0 0.231. The sea stands 200 to 450 m over the datum because the planet's water
fills a hypsometry whose continents are thin for their area (sea-level.md, the high ground's
section above), not because code reads 0 as the sea. The branch's final-frame rise (0.348 to
0.414, p 0.053) is shared by rest (0.419), so it is not the column energy; over the frames it is
small (the plate model's mean of the 19 frames after the first 100 Myr 0.335 to 0.362, p 0.44;
the last 500 Myr 0.324 to 0.348, p 0.47). Nor is it the wedge floor: with the floor back on the
datum (wd) the share is 0.466. The Andean mark, the one site left, was not tried alone; with the
hot window on, the branch drowns less than master does (0.510 against 0.589, p 0.54), so the
branch's own effect on the share is not established either way. Many measures and arms were
compared here, so single p near 0.03 to 0.05 are expected by chance.

**Drowned with the crust kept.** Run by run the drowned share follows the continental crust a run
keeps: over the 72 runs of the six arms whose sea moves (pin0 left out), Spearman's ρ of the
census's share under the sea against the continental crust's area at the end is +0.84, and against
the sea's height at the end +0.75; within each arm alone ρ is +0.66 to +0.84. A run that keeps
more of its continents keeps them thin for their area, the planet's water stands higher on them,
and more of them are under it: the hot window keeps the most (0.39 to 0.41 of the planet against
master's 0.33) and drowns the most. With these continents the crust's loss target and the drowned
target pull against each other; thicker continents for their area would serve both.

**The hot window (PLT-390) on both.** W 40 keeps the crust (net −0.011 and −0.032 against −0.095
and −0.085) and lowers the high ground (above 2 km 0.170 and 0.163, the coast 0.285, the ice 0.143
and 0.128), as the high ground's section found; and it drowns the continents on both: under the sea
0.589 and 0.510, the land 0.192 and 0.209 under its floor, the largest landmass 0.065 and 0.099
under its floor, the sea 460 and 361 m over the datum at the median. Branch + W 40 against master +
W 40 differs in none of the seven measures (p ≥ 0.25) nor the crust (p 0.37). It meets the most
criteria per run on the mean (1.75 against 1.58, 1.42 and 1.25: the ice, and a collision rate or a
land share in some runs; p 0.67 against master and 0.21 against the branch), but with the
continents drowned, so PLT-390 stays off.

### What is left, and for whom

1. *The drowned continents* (V1 item 1; the plates' and the sea level's): the share under the sea
   follows the sea's height over the continents, not a datum in the code. With the sea at the
   datum it meets its target and the land overshoots, so the gap is the continents' volume for
   their area (34 to 36 km on average against Earth's 41.1 km): the crust budget's losses
   (collisions at 2.5 to 3.8 times Earth's, the high ground's section) and the boundaries'
   orientation (item 1 of the plate geometry's list) are where to work.
2. *The land gain* (CHK-009, PLT-151): counted at the sea it runs 1.08 % at the median run, two
   runs of twelve over the 1.5 % target. If a run fails it, look at the steps where the surface
   planes ground to its base level near the shore, not at the sea.
3. *The hotspot taper* (PLT-114, uncited) and the other solid-column heights in the datum frame
   ("Left as they are" above): leave them unless a source puts them at the shore.

## The crust flux into collision fronts (2026-10-21)

The coordinator's task CF-1, V1 item 1 (decisions log 2026-10-21): the crust flux into collision
fronts, the collisions' losses, then the collision rate. "The high ground's extent" above had the
collisions losing 2.5 to 3.8 times Earth's 0.7 km³ a year because "3 to 4 times Earth's crust
crosses their fronts", an estimate from the budget. This section measures the flux term by term
against Earth's sources. Ledger PLT-400 and PLT-401, with notes on PLT-242, PLT-246 and PLT-371;
sources in the bibliography, "The crust flux into collision fronts". Code: `plates/belts.rs` (the
crossing's parts as crust-budget globals, PLT-400's switch), `plates/mod.rs`,
`plates/crust_budget.rs`, `plates/subduction.rs` (PLT-371's switch) and `params.rs`. Both switches
are off in the default and every built-in preset, so the planets are the same bit for bit.

Measured on `earthlike_quick_16k`, 1000 Myr, seeds 1 to 6, both starts (12 runs per arm), with the
crust budget on (`DEEPTIME_CRUST_BUDGET=1`): the flux over the whole run (`tools/review/loss-budget.py`), the criteria on the final
frame (`tools/review/continents_census.py fields`) and the plate checks of each run. Each value is
the median of the 12 runs; p is a two-sided Mann-Whitney U test on the runs against the base. The
base is master 49492ce3 (with ee5ef027, which moves no planet) as it runs: this branch with both
switches off. Master df40ce80 (the sea off the datum) came in after and changes every planet; the
base and PLT-371's arm ran again on it ("Again on master df40ce80" below). The build log entry of
2026-10-21 "The crust flux into collision fronts" has the commands.

### The flux and Earth's

| term | the model (base) | Earth | source |
|---|---|---|---|
| front in collision at any time (each step's front, averaged over the run) | 28,800 km | 6,000 km of active suture on average, 12,000 km today, 1,000 to 20,000 km over geologic time; 23,003 km of continental convergent boundary (CCB) | Stern & Scholl 2010, p. 18-19 (after Hildebrand & Bowring 1999, as cited); Bird 2003, Table 3 |
| convergence across it | 17.1 mm/yr | 18.7 mm/yr across the CCB (its area flux over its length; 26.2 mm/yr mean relative speed) | Bird 2003, Table 3 |
| crust crossing (the incoming crust's own thickness) | 38.3 km | 35.1 km the normal column, 41.1 km the continents' mean; India's margin 23 to 29 km if its crust was conserved | PLT-278; Christensen & Mooney 1995 (PLT-256); Yakovlev & Clark 2014 |
| area crossing | 0.49 km² a year | 0.43 km² a year (0.013616 m²/s, 12.6 % of all boundaries' area flux) | Bird 2003, Table 3 |
| volume crossing | 18.9 km³ a year; 655 km³ per km of front and Myr | about 15 to 18 km³ a year (0.43 km² a year on 35 to 41 km) | the agent's arithmetic |
| kept: PLT-242's share, then room in the wedge and the belt | 0.881 of it | | |
| lost: the share's 5 % | 0.94 km³ a year (32 per km and Myr) | | |
| lost: no room, left over after the belt's six passes with room somewhere in it | 1.15 | | |
| lost: no room, the belt full at the strength height (PLT-253) | 0.25 | | |
| lost at the fronts in all | 2.33 km³ a year; 75 per km and Myr | 0.7 km³ a year at suture zones, about 115 per km and Myr (15 sediment, 50 eroded upper plate, 50 deeply subducted lower plate); Table 1: 0.3 deep subduction, 0.1 sediment, 0.3 erosion; India 0.24 km³ a year at a 10 % minimum; 0.43 (up to 0.65) km³ a year from 16,000 km of rifted margin 18 km thick since 65 Ma | Stern & Scholl 2010, p. 12 and 18-19; Clift et al. 2009, §5.1 |
| a collision's front at its longest, weighted by the area it crossed: median (p25 to p75) | 6,200 km (2,900 to 10,300); the counted collisions 9,800 | the Himalaya about 2,400 km long | the collisions design, from Wikipedia (this page's first table) |
| a collision's life, the same weighting | 82 Myr (30 to 140); counted 158 | India against Asia about 50 Myr so far | Stern & Scholl 2010, p. 19 |
| a collision's shortening, the same weighting | 1,750 km (570 to 2,700); counted 2,160 | India against Asia: more than 4,000 km of convergence; 1,000 to 1,200 km of it Tibet's shortening, 600 to 900 km the Himalaya's, up to 1,000 km of continental subduction | van Hinsbergen 2022 |

(The front's length is its straight extent with the lattice's jags taken out, `Front::length_km`.
The weighted figures are over each run's collision records, `deeptime plate-check --record`,
counted collisions and the smaller terranes' transfers together, pooled over the 12 runs.)

### Which term is 3 to 4 times Earth's

Per km of front the model is Earth's: it converges at 17.1 mm/yr against 18.7, on 38 km of crust,
and loses 75 km³ per km and Myr against Stern & Scholl's 115 (65 without the upper plate's
erosion, which the model does not take at a front). The area crossing is Bird's too: 0.49 against
0.43 km² a year. **What is 3 to 4 times Earth's is the length of front that loses crust as a
suture**: 28,800 km at any time against Stern & Scholl's 6,000 km on average (4.8 times) and
12,000 km today (2.4 times); 2.33 km³ a year against 0.7 is 3.3 times.

The two references agree once read together. Bird's CCB counts every convergent step that is not
oceanic, transitional ones included (paragraph 127), 23,003 km of it; Stern & Scholl's suture
losses come from 6,000 to 12,000 km of active suturing. The rest of Earth's continental
convergence shortens and thickens the plates, Tibet's 1,000 to 1,200 km, with no suture's losses
booked. The model's fronts suture along all their length for all their life. A collision's front
is 2.6 times the Himalaya's at the weighted median and its life is long (82 Myr; a quarter of the
crossing is in collisions longer than 140 Myr; the plate check's collision length p90 is 139 Myr
against at most 120), while its shortening is India's (1,750 km). About 4.6 such fronts run at once
(28,800 over 6,200).

Neither is the collisions' code. The front's length is how the plates meet: 24 plates against
Bird's 52 (the lattice's floor, "The plate geometry", item 4 of its list), so continents meet along
whole margins. The collision's life is how it ends: the force balance (`plates/forces.rs`) does not
stall it, and the weld (PLT-246) needs the convergence under 5 mm/yr for 10 Myr. The boundaries'
orientation (Ben's scope) is not needed for either.

### PLT-242 against its sources

The share (0.95) came from Stern & Scholl's 50 km³ per km and Myr of deep loss over an assumed
crossing of 1,600 km³ per km and Myr (40 km of crust at 40 mm/yr). The model crosses 655 (17.1
mm/yr on 38.3 km), so Stern & Scholl's 50 is 7.6 % of it and 65, with the sediment, 9.9 %; the
model loses 5 % by the share and 6.6 % for want of room, 11.9 % in all. In line: the share stays
at 0.95, and the row is now `verified` (Stern & Scholl read again in the library, p. 18-19).

Ingalls et al. 2016 (the abstract; not in the library) find about half of India's crust missing
from the India-Asia mass balance; van Hinsbergen et al. 2017 dispute it, Yakovlev & Clark 2014
conserve the crust if the margin was 23 to 29 km thick, and Stern & Scholl take 10 % for India as a
minimum. Half of every collision's crust lost is 6.5 times Stern & Scholl's deep loss per km at
the model's crossing (327 against 50 km³ per km and Myr); as a global share it is not supported. It was measured anyway (arm rs50 below).

### What changed: the wedge's crust crosses whole (PLT-400)

Crust a collision stacked on the incoming plate (its wedge, PLT-244) is carried by that plate to
the front and crosses it: 0.14 of the crossing at the base's median. Since every crossing took the
share, that crust paid the 5 % twice. With `collision_restack_whole` (PLT-400) the part of a
crossing sample's crust above the normal column that this collision stacked crosses whole. It is
bookkeeping, and small: the share's loss falls from 0.94 to 0.79 km³ a year (p = 0.002), the
belt's no-room takes most of it back (1.35 to 1.42), and the fronts' loss goes from 2.33 to 2.17
(p = 0.80).

The rest is measurement (PLT-401): the crust budget writes the crossing's area, its front length
times the step, the crust above the normal column, the part of it raised by a collision within 30
Myr, the part this collision stacked, the wedge's share of what stays, and the no-room loss split
three ways (left over with room in the belt, the belt full at the strength height, clipped by the
ceilings); `tools/review/loss-budget.py` reads them.

### The arms

| arm | settings |
|---|---|
| base | master 49492ce3's behaviour (both switches off) |
| rs | PLT-400 on |
| rsrev | PLT-400 and PLT-371 on |
| rs50 | PLT-400 on, `collision_stacking_share` 0.5 (Ingalls) |
| rsw10 | PLT-400 on, `weld_speed_mm_yr` 10 (PLT-246, a game choice) |

Medians of 12 runs (p against the base):

| arm | land | largest landmass | continental crust under the sea | above 2 km | coast with high ground | under ice | collisions per 100 Myr | plate checks (per run) |
|---|---|---|---|---|---|---|---|---|
| base | 0.257 | 0.133 | 0.348 | 0.277 | 0.381 | 0.172 | 1.60 | 29 |
| rs | 0.250 (0.45) | 0.102 (0.40) | 0.423 (0.13) | 0.207 (0.17) | 0.329 (0.12) | 0.161 (0.58) | 1.50 (0.66) | 29 (0.81) |
| rsrev | 0.270 (0.93) | 0.132 (1.00) | 0.375 (0.73) | 0.273 (0.40) | 0.370 (0.93) | 0.176 (0.75) | 2.00 (0.09) | 29 (0.57) |
| rs50 | 0.204 (<0.01) | 0.052 (<0.01) | 0.514 (<0.01) | 0.142 (<0.01) | 0.272 (<0.01) | 0.143 (0.15) | 1.60 (0.73) | 29 (0.82) |
| rsw10 | 0.190 (<0.01) | 0.059 (<0.01) | 0.585 (<0.01) | 0.100 (<0.01) | 0.234 (<0.01) | 0.121 (<0.01) | 3.00 (<0.01) | 27 (<0.01) |

rs50 is of 11 runs: its supercontinent run of seed 2 stopped at -400 Myr on the surface guard
(item 5 below). The plate checks pooled over each arm's runs (`deeptime plate-check` on all of
them at once) pass 29 (base), 30 (rs), 27 (rsrev), 30 (rs50) and 25 (rsw10) of 45. The flux:

| arm | crust lost to the mantle, 10⁹ km³ per 1000 Myr | net change, 10⁹ km³ per 1000 Myr | fronts' loss, km³ a year | of it the share's | of it no room | front in collision, km | collision length p90, Myr | collision states a reversal ended, per 100 Myr | supercontinent tenure, Myr | continental crust at the end |
|---|---|---|---|---|---|---|---|---|---|---|
| base | 4.10 | -1.26 | 2.33 | 0.94 | 1.35 | 28,845 | 139 | 8.4 | 13 | 0.328 |
| rs | 4.08 (0.98) | -1.00 (0.07) | 2.17 (0.80) | 0.79 (<0.01) | 1.42 (0.17) | 26,402 (0.13) | 116 (0.20) | 8.7 (0.58) | 26 (0.11) | 0.349 (0.07) |
| rsrev | 4.21 (0.93) | -1.60 (0.62) | 2.31 (0.80) | 0.87 (0.01) | 1.46 (0.62) | 31,778 (0.08) | 119 (0.37) | 5.2 (<0.01) | 31 (0.66) | 0.316 (0.54) |
| rs50 | 6.80 (<0.01) | -2.37 (<0.01) | 6.09 (<0.01) | 5.95 (<0.01) | 0.19 (<0.01) | 22,551 (<0.01) | 102 (0.02) | 9.0 (0.06) | 30 (0.14) | 0.333 (0.83) |
| rsw10 | 2.81 (<0.01) | -0.70 (0.05) | 1.25 (<0.01) | 0.51 (<0.01) | 0.80 (0.02) | 16,253 (<0.01) | 63 (<0.01) | 6.3 (<0.01) | 32 (0.09) | 0.384 (<0.01) |

The continents under the sea (the census's SERIES line of the final frame; the sea from the
ocean's volume, sea-level.md):

| arm | sea level, m | continental crust's volume, 10⁹ km³ | its mean thickness, km |
|---|---|---|---|
| base | 522 | 6.80 | 40.4 |
| rs | 798 (0.06) | 7.12 (0.09) | 39.9 (0.62) |
| rsrev | 426 (0.67) | 6.47 (0.54) | 40.1 (0.91) |
| rs50 | 269 (<0.01) | 5.87 (<0.01) | 34.2 (<0.01) |
| rsw10 | 821 (<0.01) | 7.38 (0.13) | 36.2 (<0.01) |

The p values are two-sided Mann-Whitney U over the runs. The runs:

<details><summary>Per run (sca: scattered_continents, sup: supercontinent; seeds 1 to 6)</summary>

| measure | arm | sca-1 | sca-2 | sca-3 | sca-4 | sca-5 | sca-6 | sup-1 | sup-2 | sup-3 | sup-4 | sup-5 | sup-6 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| land | base | 0.287 | 0.301 | 0.249 | 0.258 | 0.280 | 0.233 | 0.223 | 0.257 | 0.295 | 0.227 | 0.278 | 0.253 |
| land | rs | 0.266 | 0.258 | 0.302 | 0.189 | 0.273 | 0.237 | 0.264 | 0.291 | 0.242 | 0.214 | 0.224 | 0.229 |
| land | rsrev | 0.281 | 0.237 | 0.278 | 0.282 | 0.213 | 0.284 | 0.258 | 0.263 | 0.231 | 0.278 | 0.235 | 0.284 |
| land | rs50 | 0.229 | 0.232 | 0.193 | 0.221 | 0.174 | 0.198 | 0.203 | - | 0.194 | 0.227 | 0.214 | 0.204 |
| land | rsw10 | 0.149 | 0.217 | 0.222 | 0.115 | 0.177 | 0.170 | 0.126 | 0.202 | 0.142 | 0.216 | 0.222 | 0.245 |
| largest landmass | base | 0.257 | 0.175 | 0.091 | 0.186 | 0.148 | 0.097 | 0.136 | 0.080 | 0.212 | 0.058 | 0.130 | 0.093 |
| largest landmass | rs | 0.094 | 0.128 | 0.222 | 0.051 | 0.152 | 0.072 | 0.105 | 0.197 | 0.109 | 0.099 | 0.075 | 0.084 |
| largest landmass | rsrev | 0.144 | 0.048 | 0.101 | 0.104 | 0.100 | 0.127 | 0.184 | 0.183 | 0.145 | 0.153 | 0.112 | 0.136 |
| largest landmass | rs50 | 0.088 | 0.069 | 0.050 | 0.049 | 0.035 | 0.063 | 0.085 | - | 0.050 | 0.090 | 0.046 | 0.052 |
| largest landmass | rsw10 | 0.039 | 0.084 | 0.079 | 0.015 | 0.043 | 0.026 | 0.023 | 0.072 | 0.059 | 0.059 | 0.104 | 0.117 |
| under the sea | base | 0.306 | 0.282 | 0.343 | 0.283 | 0.287 | 0.375 | 0.508 | 0.403 | 0.306 | 0.437 | 0.353 | 0.396 |
| under the sea | rs | 0.406 | 0.343 | 0.323 | 0.580 | 0.338 | 0.484 | 0.370 | 0.239 | 0.462 | 0.482 | 0.503 | 0.440 |
| under the sea | rsrev | 0.293 | 0.443 | 0.230 | 0.326 | 0.467 | 0.306 | 0.381 | 0.381 | 0.447 | 0.369 | 0.410 | 0.270 |
| under the sea | rs50 | 0.335 | 0.416 | 0.538 | 0.492 | 0.603 | 0.541 | 0.532 | - | 0.550 | 0.411 | 0.477 | 0.514 |
| under the sea | rsw10 | 0.716 | 0.514 | 0.537 | 0.779 | 0.629 | 0.631 | 0.741 | 0.540 | 0.671 | 0.495 | 0.502 | 0.410 |
| above 2 km | base | 0.312 | 0.194 | 0.313 | 0.363 | 0.287 | 0.273 | 0.281 | 0.225 | 0.212 | 0.178 | 0.312 | 0.248 |
| above 2 km | rs | 0.180 | 0.368 | 0.234 | 0.210 | 0.298 | 0.168 | 0.203 | 0.353 | 0.204 | 0.188 | 0.148 | 0.271 |
| above 2 km | rsrev | 0.375 | 0.183 | 0.362 | 0.282 | 0.316 | 0.251 | 0.236 | 0.325 | 0.236 | 0.251 | 0.264 | 0.330 |
| above 2 km | rs50 | 0.240 | 0.205 | 0.134 | 0.132 | 0.123 | 0.142 | 0.142 | - | 0.166 | 0.161 | 0.171 | 0.137 |
| above 2 km | rsw10 | 0.092 | 0.148 | 0.096 | 0.040 | 0.062 | 0.095 | 0.055 | 0.170 | 0.103 | 0.178 | 0.117 | 0.146 |
| coast with high ground | base | 0.432 | 0.374 | 0.399 | 0.476 | 0.402 | 0.377 | 0.358 | 0.386 | 0.374 | 0.296 | 0.278 | 0.384 |
| coast with high ground | rs | 0.301 | 0.327 | 0.436 | 0.359 | 0.303 | 0.276 | 0.331 | 0.567 | 0.396 | 0.261 | 0.276 | 0.358 |
| coast with high ground | rsrev | 0.469 | 0.319 | 0.511 | 0.488 | 0.472 | 0.365 | 0.361 | 0.348 | 0.365 | 0.298 | 0.397 | 0.376 |
| coast with high ground | rs50 | 0.350 | 0.365 | 0.187 | 0.240 | 0.218 | 0.267 | 0.230 | - | 0.353 | 0.272 | 0.355 | 0.288 |
| coast with high ground | rsw10 | 0.193 | 0.316 | 0.201 | 0.175 | 0.240 | 0.333 | 0.198 | 0.229 | 0.265 | 0.250 | 0.192 | 0.317 |
| under ice | base | 0.194 | 0.172 | 0.177 | 0.206 | 0.171 | 0.132 | 0.272 | 0.140 | 0.140 | 0.120 | 0.204 | 0.163 |
| under ice | rs | 0.162 | 0.204 | 0.141 | 0.157 | 0.190 | 0.159 | 0.138 | 0.174 | 0.166 | 0.106 | 0.157 | 0.216 |
| under ice | rsrev | 0.224 | 0.158 | 0.170 | 0.200 | 0.178 | 0.175 | 0.176 | 0.176 | 0.090 | 0.181 | 0.155 | 0.183 |
| under ice | rs50 | 0.212 | 0.157 | 0.128 | 0.091 | 0.179 | 0.139 | 0.115 | - | 0.188 | 0.106 | 0.148 | 0.143 |
| under ice | rsw10 | 0.097 | 0.136 | 0.110 | 0.082 | 0.082 | 0.151 | 0.052 | 0.114 | 0.132 | 0.129 | 0.160 | 0.159 |
| collisions per 100 Myr | base | 1.6 | 1.2 | 2.5 | 1.3 | 2.1 | 1.7 | 1.1 | 1.5 | 2.4 | 1.6 | 1.8 | 1.5 |
| collisions per 100 Myr | rs | 0.9 | 1.2 | 1.4 | 1.5 | 1.5 | 1.6 | 2.4 | 1.2 | 1.0 | 2.7 | 2.5 | 2.3 |
| collisions per 100 Myr | rsrev | 1.9 | 2.2 | 2.3 | 1.3 | 2.8 | 1.5 | 2.0 | 1.5 | 3.2 | 2.5 | 2.0 | 1.9 |
| collisions per 100 Myr | rs50 | 1.0 | 1.2 | 1.6 | 1.4 | 1.7 | 2.1 | 1.3 | - | 1.4 | 2.1 | 1.8 | 1.9 |
| collisions per 100 Myr | rsw10 | 3.3 | 3.8 | 3.0 | 2.9 | 2.5 | 2.7 | 2.1 | 3.1 | 3.1 | 2.7 | 3.0 | 4.0 |
| plate checks | base | 29 | 30 | 29 | 29 | 32 | 28 | 28 | 28 | 29 | 24 | 31 | 29 |
| plate checks | rs | 29 | 32 | 28 | 27 | 29 | 29 | 31 | 31 | 28 | 28 | 29 | 30 |
| plate checks | rsrev | 30 | 29 | 28 | 28 | 29 | 25 | 29 | 31 | 29 | 28 | 29 | 26 |
| plate checks | rs50 | 31 | 29 | 30 | 29 | 27 | 31 | 29 | - | 26 | 27 | 29 | 28 |
| plate checks | rsw10 | 27 | 29 | 27 | 24 | 27 | 26 | 25 | 26 | 28 | 27 | 27 | 25 |
| crust lost, 10⁹ km³ per 1000 Myr | base | 4.86 | 4.05 | 3.26 | 4.66 | 4.77 | 4.14 | 3.83 | 3.83 | 3.27 | 2.63 | 4.48 | 4.70 |
| crust lost, 10⁹ km³ per 1000 Myr | rs | 4.51 | 4.53 | 3.65 | 4.28 | 4.81 | 4.08 | 2.82 | 5.26 | 3.53 | 3.06 | 4.09 | 3.89 |
| crust lost, 10⁹ km³ per 1000 Myr | rsrev | 4.81 | 4.15 | 3.87 | 3.99 | 4.28 | 4.45 | 2.92 | 4.36 | 2.87 | 2.74 | 4.49 | 4.73 |
| crust lost, 10⁹ km³ per 1000 Myr | rs50 | 8.51 | 7.67 | 5.62 | 7.79 | 6.80 | 6.68 | 8.11 | - | 6.94 | 6.53 | 6.74 | 6.03 |
| crust lost, 10⁹ km³ per 1000 Myr | rsw10 | 2.18 | 3.22 | 2.74 | 2.17 | 2.39 | 2.91 | 1.88 | 2.80 | 2.89 | 3.21 | 2.83 | 3.05 |

</details>

### The weld at 10 mm/yr (arm rsw10)

The length of front in collision is the term out of line, and the weld (PLT-246, a game choice)
is the one constant that sets a collision's life inside the collisions' code. At 10 mm/yr, with
PLT-400:

- the front in collision falls from 28,800 to 16,300 km, toward Earth's 6,000 to 12,000; the
  collision length p90 from 139 to 63 Myr (the target at most 120); the fronts' loss from 2.33 to
  1.25 km³ a year (Earth 0.7), still 83 km³ per km and Myr; the crust lost to the mantle from 4.10
  to 2.81 × 10⁹ km³ per 1000 Myr;
- the collisions rise from 1.60 to 3.00 per 100 Myr, the land above 2 km falls to 0.100, the
  coast with high ground to 0.234 and the land under ice to 0.121 (all p < 0.01);
- but the land falls from 0.257 to 0.190, the largest landmass from 0.133 to 0.059, and the
  continental crust under the sea rises from 0.348 to 0.585 (all p < 0.01). The continents keep
  their volume (7.38 against 6.80 × 10⁹ km³, p = 0.13) but spread it over 0.384 of the planet
  against 0.328, 36.2 km thick against 40.4, under a sea 300 m higher (822 against 522 m, since
  the ocean basins are smaller). Less crust crosses (0.32 against 0.49 km² a year), so less of the
  continents' area is turned into thickness;
- the plate checks pooled fall from 29 to 25 of 45: the collision length, the collision rate and
  the slab's speed over the free plates' now pass, while the largest plate over half the planet,
  plates dead within 20 Myr, rifts alive and welded back at 50 Myr, the continental share's
  maximum, the speed's rank against area and the ocean older than 180 Myr now fail.

So shortening the collisions alone drowns the continents: in the model their long life is what
thickens the crust enough to stand above the sea. The weld stays at 5 mm/yr. Bringing the front's
length to Earth's needs the crust thickened, or the continents' area held, by something other
than collisions that last too long (item 1 below).

### PLT-371's reversal on top

PLT-371 (the plate-geometry task's reversal judged by the crust the arc stands on, now the switch
`reversal_by_arc_crust`) on top of PLT-400, arm rsrev. Against the base:

- the collisions go from 1.60 to 2.00 per 100 Myr at the median (p = 0.09); 7 of 12 runs reach 2,
  against 3 of 12. Pooled over the runs the plate check passes (2.00 on the scattered start, 2.18
  on the supercontinent), where the base fails at 1.69;
- the collision states a reversal ended (the plate check's `partings_reversed_per_100myr`) fall from
  8.4 to 5.2 per 100 Myr (p < 0.001);
- the land holds: 0.257 to 0.270 (p = 0.93), the largest landmass 0.133 to 0.132, continental crust
  under the sea 0.348 to 0.375 (p = 0.73), above 2 km 0.277 to 0.273, coast with high ground 0.381
  to 0.370, under ice 0.172 to 0.176;
- the fronts' loss is the same (2.33 to 2.31 km³ a year), on a longer front in collision (28,800 to
  31,800 km, p = 0.08).

Against rs (PLT-400 alone) it puts back what PLT-400 moved: land 0.250 to 0.270 (p = 0.39), the
largest landmass 0.102 to 0.132 (p = 0.29), above 2 km 0.207 to 0.273 (p = 0.06), coast with high
ground 0.329 to 0.370 (p = 0.06); the collisions last longer (the plate check's median length 40 to
49 Myr, p = 0.05). The plate-geometry task's loss of land (0.264 to 0.208 on seeds 1 to 3) was
PLT-371 alone on master c790d1f3; on seeds 1 to 3 here, with PLT-400, the land goes 0.269 to 0.258
(mean of six), inside the runs' spread.

### Again on master df40ce80 (the sea off the datum)

The arms above ran on this branch before master's sea off the datum (df40ce80) was merged in; that
merge changes every planet. The base and rsrev ran again on the merged branch (arms m2base and
m2rsrev, 12 runs each, the same seeds and starts). Medians, p against m2base:

| arm | land | largest landmass | continental crust under the sea | above 2 km | coast with high ground | under ice | collisions per 100 Myr | plate checks (per run) | plate checks pooled |
|---|---|---|---|---|---|---|---|---|---|
| m2base | 0.247 | 0.104 | 0.414 | 0.248 | 0.369 | 0.170 | 1.65 | 29 | 30 of 45 |
| m2rsrev | 0.244 (0.80) | 0.097 (0.58) | 0.365 (0.44) | 0.282 (0.37) | 0.389 (0.40) | 0.179 (0.44) | 1.85 (0.32) | 28 (0.43) | 26 of 45 |

| arm | crust lost to the mantle, 10⁹ km³ per 1000 Myr | net change, 10⁹ km³ per 1000 Myr | fronts' loss, km³ a year | front in collision, km | collision length p90, Myr | collision states a reversal ended, per 100 Myr | continental crust at the end |
|---|---|---|---|---|---|---|---|
| m2base | 4.25 | -1.27 | 2.43 | 27,554 | 112 | 9.5 | 0.338 |
| m2rsrev | 4.24 (0.80) | -1.61 (0.01) | 2.30 (1.00) | 33,741 (<0.01) | 135 (0.26) | 5.1 (<0.01) | 0.317 (0.04) |

Per run, seeds 1 to 6 on the scattered start, then on the supercontinent:

| measure | arm | sca-1 | sca-2 | sca-3 | sca-4 | sca-5 | sca-6 | sup-1 | sup-2 | sup-3 | sup-4 | sup-5 | sup-6 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| land | m2base | 0.249 | 0.222 | 0.225 | 0.240 | 0.243 | 0.246 | 0.260 | 0.244 | 0.269 | 0.263 | 0.260 | 0.250 |
| land | m2rsrev | 0.224 | 0.256 | 0.238 | 0.206 | 0.251 | 0.223 | 0.223 | 0.270 | 0.250 | 0.274 | 0.300 | 0.228 |
| largest landmass | m2base | 0.127 | 0.061 | 0.040 | 0.144 | 0.099 | 0.090 | 0.091 | 0.120 | 0.109 | 0.152 | 0.164 | 0.055 |
| largest landmass | m2rsrev | 0.073 | 0.097 | 0.097 | 0.057 | 0.075 | 0.078 | 0.055 | 0.188 | 0.164 | 0.216 | 0.215 | 0.174 |
| under the sea | m2base | 0.428 | 0.417 | 0.527 | 0.394 | 0.420 | 0.361 | 0.387 | 0.445 | 0.325 | 0.418 | 0.322 | 0.412 |
| under the sea | m2rsrev | 0.461 | 0.356 | 0.391 | 0.487 | 0.331 | 0.432 | 0.352 | 0.339 | 0.374 | 0.283 | 0.221 | 0.475 |
| above 2 km | m2base | 0.261 | 0.372 | 0.065 | 0.308 | 0.208 | 0.335 | 0.235 | 0.135 | 0.295 | 0.159 | 0.351 | 0.184 |
| above 2 km | m2rsrev | 0.269 | 0.294 | 0.227 | 0.299 | 0.328 | 0.279 | 0.274 | 0.281 | 0.283 | 0.370 | 0.353 | 0.151 |
| coast with high ground | m2base | 0.359 | 0.417 | 0.209 | 0.308 | 0.381 | 0.409 | 0.399 | 0.353 | 0.380 | 0.284 | 0.465 | 0.292 |
| coast with high ground | m2rsrev | 0.349 | 0.269 | 0.354 | 0.426 | 0.338 | 0.384 | 0.448 | 0.513 | 0.435 | 0.393 | 0.398 | 0.334 |
| under ice | m2base | 0.174 | 0.242 | 0.167 | 0.190 | 0.106 | 0.174 | 0.103 | 0.152 | 0.217 | 0.093 | 0.182 | 0.129 |
| under ice | m2rsrev | 0.191 | 0.189 | 0.179 | 0.174 | 0.219 | 0.189 | 0.179 | 0.172 | 0.072 | 0.182 | 0.175 | 0.096 |
| collisions per 100 Myr | m2base | 1.4 | 1.4 | 1.9 | 1.4 | 1.1 | 1.9 | 1.8 | 2.4 | 1.4 | 2.0 | 1.5 | 2.1 |
| collisions per 100 Myr | m2rsrev | 1.9 | 1.9 | 1.7 | 2.0 | 1.8 | 1.6 | 3.0 | 1.4 | 1.6 | 1.1 | 2.5 | 2.8 |
| plate checks | m2base | 28 | 27 | 27 | 27 | 29 | 29 | 29 | 31 | 28 | 30 | 30 | 29 |
| plate checks | m2rsrev | 27 | 25 | 26 | 30 | 30 | 28 | 27 | 31 | 29 | 27 | 29 | 28 |
| crust lost, 10⁹ km³ per 1000 Myr | m2base | 4.34 | 4.70 | 3.53 | 4.72 | 5.13 | 4.29 | 2.97 | 4.72 | 3.75 | 3.57 | 4.21 | 3.74 |
| crust lost, 10⁹ km³ per 1000 Myr | m2rsrev | 4.22 | 4.18 | 3.67 | 5.25 | 4.93 | 4.19 | 3.34 | 4.67 | 4.16 | 4.34 | 4.26 | 4.46 |

On the merged master the collisions rise less: 1.65 to 1.85 at the median (p = 0.32), 4 of 12
runs at 2 or more against 3, and pooled over the runs 1.69 to 1.94, so the plate check still
fails. The land holds again (0.247 to 0.244). What it costs shows on both bases: the continental
volume's net change goes from -1.27 to -1.61 × 10⁹ km³ per 1000 Myr (p = 0.01; -1.26 to -1.60
before the merge), the front in collision lengthens by 10 to 22 % (28,800 to 31,800 km, p = 0.08;
27,600 to 33,700 km, p < 0.01), and the pooled plate checks fall by 2 to 4 (29 to 27; 30 to 26:
the collision length's p50 and p90, the speed guard on one step, the ocean older than 180 Myr).

So PLT-371 does not reliably lift the collisions into 2 to 6: it reaches the floor on one base
(2.00, p = 0.09) and not on the other (1.85, p = 0.32). It does so by more and longer collisions,
which lengthens the term that is already 3 to 4 times Earth's, and it costs continental crust. It
stays off. PLT-400 is bookkeeping with no criterion moved past p = 0.1; it can go on whenever the
planets are changed anyway (the coordinator's call). Both switches are off on this branch.

### What is left, and for whom

1. *The length of front in collision* (the plates'): 28,800 km against Earth's 6,000 to 12,000
   km of suture, the term that is 3 to 4 times Earth's. Two causes, neither in the collisions'
   code. The plate count: 24 plates against Bird's 52 ("The plate geometry", item 4), so
   continents meet along whole margins. The collisions' life: the plate check's collision length
   p90 is 139 Myr against at most 120, since the force balance (`plates/forces.rs`, outside this
   task) does not stall a collision and the weld (PLT-246) waits for the convergence to stay under
   5 mm/yr for 10 Myr. Ending collisions sooner by the weld alone (arm rsw10, 10 mm/yr) brings
   the front to 16,300 km and the collisions to 3.00 per 100 Myr but drowns the continents (land
   0.190, continental crust under the sea 0.585): the long collisions are what thickens the
   crust, so a shorter life needs the crust thickened, or the continents' area held, some other
   way. A collision stalled by its own thickened crust is a force law in `plates/forces.rs`; it
   was not touched here.
2. *The no-room loss* (the belts'): 1.15 km³ a year, half the fronts' loss, is crust left over
   after the belt's passes with room still somewhere in the belt (`fill_belt`), and 0.25 more
   finds the belt full at the strength height (PLT-253). With it the fronts lose 75 km³ per km and
   Myr, under Stern & Scholl's 115, so it is not an excess per km; but it is a loss for want of a
   place, not a process. Placing it in the belt would raise the belts, against the high-ground
   criteria; a belt that widens as it fills (its foreland, PLT-244's wedge reaching farther) would
   put it on the land without raising it.
3. *What a collision is* (the plate check's): PLT-157's target of 2 to 6 per 100 Myr has no source
   (P1-27's figure), and the check counts a collision only when the terrane is a continent
   (PLT-078). Counting also the smaller transfers behind which continental crust of at least 0.1 of
   a continent's area went under gives 4.05 per 100 Myr at the base's median and 4.65 with
   PLT-400 and PLT-371 (p = 0.03; on master df40ce80 4.05 and 4.70, p = 0.01), an upper bound
   since a record's `cont_in` counts the
   continental crust that went under before the fronts met. Which count Earth's figure means
   (sutures that close an ocean, or every accretion of continental crust) decides how far the
   model is below it.
4. *The switches* (the coordinator's): PLT-371 stays off (it lifts the collisions on one base and
   not the other, by longer fronts in collision, at a cost in crust and plate checks); PLT-400 is
   bookkeeping that moves no criterion and can go on with the next planet change. Both are off on
   this branch, so the planets are the same bit for bit and no expected value changed; switching
   either on changes every planet, and the seed-specific tests' expected values then need the new
   planets. (PLT-400 went on 2026-10-22, in the change that put the transition's width on the
   model sphere: the last section of this page.)
5. *Ingalls' half and the surface guard*: the share at 0.5 (arm rs50) loses the land and is
   rejected (PLT-242). One of its 12 runs (supercontinent, seed 2) stopped at -400 Myr on the
   surface guard (a cell at -238,024 m against the bound of 237,286 m; flexure.md "The surface
   guard"), so its medians are of 11. No run of the other arms stopped. It happens only with half
   of the crossing crust going down, a setting no preset uses, and was not followed further.

## The transition's width and the drowned continents (2026-10-22)

The full measurement of the cause is in sea-level.md, "The continents under the sea and the sea's
height at the end". For the plate model: the ocean-continent transition's width W (PLT-262) was
200 km on the planet, which on the quick 16k planet is 1,632 km on the Earth-sized model sphere
the plate model runs on, eight times Earth's margins. The taper's continental half (target
thickness 20 to 33 km, `plates/transition.rs`) lies under the 35.1 km at which the median surface
crosses the sea, so it is under the sea wherever nothing thickened it, and 97 to 100 % of the
continental crust lay within 100 km (on the planet) of oceanic crust. Twelve runs an arm
(`earthlike_quick_16k`, seeds 1 to 6, both starts, master df40ce80, `--set
tectonics.transition_width_km=...` on the code where W was on the planet), medians:

| Median of 12 | W 200 km (master) | W 70 km | W 24.5 km (200 on the model sphere) |
|---|---|---|---|
| Continental crust under the sea, final frame (≤ 0.25) | 0.414 | 0.384 (p 0.40) | 0.279 (p 0.002) |
| The same, the plate model's mean over the frames after the first 100 Myr | 0.362 | 0.281 (p 0.001) | 0.243 (p < 0.001) |
| Land, final frame (0.26 to 0.32) | 0.247 | 0.253 (p 0.51) | 0.274 (p 0.002) |
| Largest landmass (≥ 0.12) | 0.104 | 0.080 (p 0.84) | 0.125 (p 0.073) |
| Continental crust's mean thickness, km | 39.5 | 41.3 (p 0.002) | 42.9 (p < 0.001) |
| Passive margins: slope between 500 and 2,000 m, median | 2.5° | 2.8° | 3.2° |

W at 70 km fell short on the final frame because its sea climbs through the run (to 741 m over
the datum over the last 100 Myr): the higher continents shed more sediment, and none subducts.

**The change (Ben, decisions log 2026-10-22, D6: "200 km on sphere").** `transition_width_km` is
now kilometres on the model sphere, like `collision_belt_km` and the plate model's other lengths
(`Norm::rad`, PLT-003): `width_rad` is `transition_width_km / model_radius_km`, so the taper is
200/6,370 rad wide on every planet (24.6 km on the quick 16k planet, 2.8 L0 samples across; 4.4
at the full presets' 250,000 samples) and the planet's radius no longer enters. The key keeps its
name and every preset its 200: only what the number means moved. PLT-262 changed with it (unit,
description, status). The same change turns PLT-400 on (`collision_restack_whole`, in the default
and every built-in preset: the coordinator's call, since it moves no criterion; it is
bookkeeping).

**Measured on the shipped change.** Twelve runs an arm, `earthlike_quick_16k`, 1,000 Myr, seeds 1
to 6, both starts, 2 threads, the crust budget on, on the test VM: the base is master 10bafdae
(W on the planet, PLT-400 off), the arm this branch (W on the model sphere, PLT-400 on), each
built from an archive of its commit and run side by side; *W only* is this branch with
`--set tectonics.collision_restack_whole=false`, to see what each change does. Medians, two-sided
Mann-Whitney U over the runs against the base:

| Median of 12 | base | arm | p | W only | p |
|---|---|---|---|---|---|
| Land (0.26 to 0.32) | 0.247 | 0.300 | 0.001 | 0.300 | 0.001 |
| Largest landmass (≥ 0.12) | 0.104 | 0.124 | 0.34 | 0.158 | 0.069 |
| Continental crust under the sea (≤ 0.25) | 0.414 | 0.278 | < 0.001 | 0.263 | 0.001 |
| Land above 2 km, of the land (≤ 0.12) | 0.248 | 0.271 | 0.80 | 0.267 | 0.40 |
| Coast with high ground (≤ 0.15) | 0.369 | 0.411 | 0.033 | 0.404 | 0.078 |
| Land under ice (≤ 0.15) | 0.170 | 0.170 | 0.93 | 0.174 | 0.47 |
| Collisions per 100 Myr (2 to 6) | 1.65 | 1.80 | 0.84 | 1.60 | 1.0 |
| Criteria of the seven met, per run | 1 | 2 | 0.018 | 2 | 0.010 |
| Continental crust's net change over 1,000 Myr, share of the planet | −0.085 | −0.065 | 0.069 | −0.061 | 0.18 |
| Plate checks met, of 45 | 29 | 28 | 0.38 | 28 | 0.43 |
| Under the sea, the plate model's mean over the last 500 Myr | 0.348 | 0.215 | < 0.001 | 0.247 | < 0.001 |
| Crust at least 25 km thick above the sea (Earth 0.81) | 0.572 | 0.677 | 0.001 | 0.693 | 0.002 |
| Continental crust's mean thickness, km | 39.5 | 43.4 | < 0.001 | 44.4 | < 0.001 |
| Passive margins, slope between 500 and 2,000 m, median (pooled transects) | 2.5° | 3.1° | | 3.0° | |

The drowned share lands where the scratch arm put it (0.278 against 0.279; all 12 runs fall, by
0.02 to 0.21), the land and the largest landmass reach their bands, and the passive slopes steepen
by 0.6° (the steepest by 0.7°), not the 1.9° the decision of 2026-10-05 weighed. Five of the twelve
runs are under 0.25; the median is not. What did not move is the high ground (above 2 km 0.271;
the coast 0.411, slightly worse, p 0.033: the thicker continents stand higher), the ice and the
collisions; the width does not touch those. The land's median, 0.300, is 0.026 above the scratch
arm's 0.274 at the same W to 0.3 % (24.5 km on a planet of 782.28 km is a taper 0.26 % narrower
than 200/6,370 rad), and the W-only arm gives 0.300 as well: any change reshuffles the runs'
plates, and medians of twelve carry about 0.02 on the land and the drowned share. PLT-400 adds
nothing at this size: against W only, no row of the table differs with p under 0.17 (the largest
landmass 0.158 to 0.124, the drowned share 0.263 to 0.278).

The sea stands higher with the thicker continents (820 m over the datum at the end against 538;
sea-level.md), which keeps the final share 0.03 over the target.

**Tests.** `plates::belts::tests::stacking_moves_crust_and_creates_none` tests PLT-242's share
alone (what is added is the share times what crossed) and failed with PLT-400 on by default
(share 0.5: 1.06 × 10¹⁰ added for 1.96 × 10¹⁰ crossed); it now sets the switch off, and
`a_collisions_own_wedge_crosses_whole_with_the_switch` covers the switch itself. The taper's own
tests (`plates::transition`) had Earth's 6,370 km model sphere and the quick planet's 782 km, so
the 200 km taper on the planet spanned 9 of the fixture's 16,000 samples; with W on the model
sphere the same 200 km would be one sample wide (178 km spacing), too coarse to test the taper's
shape, so the fixture's model sphere is now 782 km and the taper spans the same 9 samples. A new
assertion fixes `width_rad` at 200/6,370 on two planet sizes. No other seed-specific test moved
(`cargo test --workspace`; 841 tests in the core crate).

Decided: W is on the model sphere and PLT-400 is on. Open: the high ground, the ice and the
collisions, which the width does not touch, are the next planet change's.

## The high ground after D6: the collapse measured from the sea (2026-10-22)

The coordinator's task on V1 item 1's high ground, on master f9005138 (after D6, the previous
section): move land above 2 km (at most 0.12 of the land), the coast with land above 2 km within
200 km (at most 0.15) and land under ice (at most 0.15) towards their targets without losing the
land (0.26 to 0.32 of the planet), the largest landmass (at least 0.12) or the drowned share (at
most 0.25); collisions 2 to 6 per 100 Myr. Ledger PLT-412 to PLT-415 (new), PLT-390 and PLT-391
(measured again). Code: `plates/collapse.rs`, `plates/mod.rs`, `plates/rate.rs` (one call),
`params.rs` and the table driver `tools/review/v1-item1-table.sh`; nothing in `surface/`. The new
laws were in the code as switches and **not on** when this section was written (below, "What is
on"); the sheet's four keys are on by default since 2026-10-23, with the trench budget ("On by
default (2026-10-23)", at the end of this page). The diagnosis, including
mid-run frames for Ben's question about upland continents, is in "What the high ground is" and
"Where the high ground stands mid-run": plateau interiors are not the main term.

### What the high ground is on master f9005138

Twelve runs (`earthlike_quick_16k`, 1,000 Myr, seeds 1 to 6, both starts, the crust budget on),
the final frame, medians of the runs (a scratch `hg.py` on `deeptime dump-fields`; the age is the
last orogeny stamp, the thickness the plate model's):

- Land above 2 km is 0.271 of the land and the sea stands 820 m over the datum.
- *Active belts.* 0.53 of the high ground was raised by an orogeny within the last 10 Myr (0.13
  from 10 to 20 Myr, 0.11 from 20 to 40, 0.11 from 40 to 100, 0.05 older, 0.03 none); 0.67 of it
  is Himalayan (collision), 0.31 Andean. 0.72 of it is still rising. Only 0.04 of it stands on
  crust thinner than 50 km.
- *Quiet thick crust that cannot collapse.* 0.43 of the high ground is quiet (its last orogeny
  10 Myr or more ago) and thicker than T_c = 50 km; only 0.30 of that has room on its plate within
  the collapse's 200 km (continental crust thinner than T_n = 35.1 km, or ocean floor).
- *The lowland it would collapse into is not room.* The plates' old dry lowland (no orogeny in
  100 Myr, area-weighted over the plates) is 41.3 km of crust standing 528 m over the sea: crust
  between 35.1 km and the lowland's 41 km is no room for the collapse though it stands 2 km
  under the orogen beside it. And the hot window's target T_n stands at z_0 = 525 m over the
  datum, so 300 m under a sea 820 m over it: thinned to T_n, an orogen ends under the sea, which
  is why PLT-390 drowns the continents.
- *Volume.* The high ground's crust over 35.1 km is 1.04 × 10⁹ km³ and the drowned continental
  crust's room under 35.1 km 0.23 × 10⁹ km³; up to 42 km the two are 0.76 and 0.53 × 10⁹ km³.
- *The coast.* The high ground within 200 km of the sea is about four fifths raised within the
  last 10 Myr and three quarters Himalayan: belts at the continents' edges, not collapsed crust.

So even a collapse that took every quiet column to the lowland would leave the active belts,
0.53 × 0.271 = 0.14 of the land, above the 0.12 target. The rest of the way is the collisions'
(the front's length in collision, "The crust flux into collision fronts" above), not the collapse's.

### The law: the column from the sea (PLT-412) and its cap (PLT-413)

Gravitational collapse "corresponds to the decay of lateral contrast in gravitational potential
energy"; with the surroundings fixed it is "a transfer of gravitational potential energy from the
elevated regions towards the low lands ... accommodated by a combination of gravitational sliding
of the brittle crust and horizontal spread of the lower crust", and "fixed-boundary divergent
collapse implies the lateral growth of the orogenic domain at the expense of the surrounding
lithosphere" (Rey, Vanderhaeghe & Teyssier 2001, abstract). The level a collapse works towards is
the surrounding lowland's, not a column fixed at the datum. Earth's stable continents are about
the global mean: "Shields and platforms have an average crustal thickness nearly equal to the
global average", 41.1 km (extended crust 30.5 km, orogens 46.3 km; Christensen & Mooney 1995,
abstract), and they stand "a few hundred meters" over the sea, the freeboard; the continents'
thickening "exerts a first‐order control on the global sea level" (Whitehead & Clift 2009,
abstract).

**PLT-412** (`collapse_sea_column`). With the switch on, the collapse's normal column is the one
whose Airy height stands as far over the step's sea S as T_n's stands over the datum:
T_col = T_n + 7.02 S (`pools::COLUMN_ISOSTATIC_FACTOR`), 41.8 km under a sea 956 m over the
datum, which is the model's own old lowland (41.3 km at 528 m over the sea on the base) and
Christensen & Mooney's mean. T_col is what a hot orogen thins towards (PLT-390), the level the
collapse fills its room up to (continental crust thinner than T_col, and ocean floor), the column
at which filled ocean floor turns continental, and the thickest crust's ocean fill (PLT-277, no
higher than its own fill). It is kept within [T_n / 2, T_c] (guards, not reached by the presets'
seas). Heights still follow Airy in the datum frame; only the column the collapse aims at moves.
The thickening's own spread (`rate.rs`, PLT-287) keeps T_n: it is not the collapse.

**PLT-413** (`collapse_sea_column_max_km`). The same column, never thicker than Christensen &
Mooney's 41.1 km: a sea more than 855 m over the datum no longer thickens it. Added in the second
wave after the first showed the sea rising with the arms (below).

### The arms

Twelve runs an arm, `earthlike_quick_16k`, 1,000 Myr, seeds 1 to 6, both starts, 2 threads,
`DEEPTIME_CRUST_BUDGET=1`, on the test VM; the base is master f9005138 and the arms this branch
(commits 7a823bf8 and 6a498363, which run the base's planets bit for bit with the switches off),
each built from an archive of its commit; the first wave ran side by side with the base, the
second a few hours later (the runs are deterministic: the cap's binary with the cap unset
reproduces the first wave's PLT-391 arm's census byte for byte). The census of the final frame
(`tools/review/continents_census.py`), the plate checks, and a two-sided Mann-Whitney U over the
runs against the base. Master moved to f87de661 during the task with documents only. *W* is
PLT-390's window (`collapse_hot_myr`), *cf* PLT-391 (`collapse_continent_first`), *sea* PLT-412,
*cap* PLT-412 with PLT-413 at 41.1 km.

| Median of 12 (p against the base) | base | W 40 | W 20 | W 40, cf | sea, W 0 | sea, W 40 | sea, W 40, cf |
|---|---|---|---|---|---|---|---|
| Land (0.26 to 0.32) | 0.300 | 0.262 (0.29) | 0.278 (0.19) | 0.249 (0.069) | 0.298 (0.98) | 0.304 (0.91) | 0.353 (0.001) |
| Largest landmass (≥ 0.12) | 0.124 | 0.086 (0.75) | 0.112 (0.89) | 0.091 (0.44) | 0.097 (0.53) | 0.152 (0.51) | 0.193 (0.019) |
| Continental crust under the sea (≤ 0.25) | 0.278 | 0.399 (0.078) | 0.358 (0.17) | 0.440 (0.057) | 0.266 (1.0) | 0.253 (0.84) | 0.171 (0.017) |
| Land above 2 km, of the land (≤ 0.12) | 0.271 | 0.214 (0.035) | 0.212 (0.16) | 0.194 (0.026) | 0.201 (0.54) | 0.214 (0.035) | 0.167 (0.002) |
| Coast with high ground (≤ 0.15) | 0.411 | 0.332 (0.004) | 0.322 (0.002) | 0.331 (0.003) | 0.374 (0.36) | 0.352 (0.009) | 0.395 (0.44) |
| Land under ice (≤ 0.15) | 0.170 | 0.162 (0.71) | 0.178 (0.19) | 0.161 (0.98) | 0.169 (0.98) | 0.157 (0.34) | 0.148 (0.19) |
| Collisions per 100 Myr (2 to 6) | 1.80 | 1.65 (0.75) | 1.55 (0.30) | 1.65 (0.69) | 1.85 (0.24) | 1.75 (1.0) | 2.10 (0.064) |
| Criteria of the seven met, per run | 2 | 2 (0.20) | 2 (0.19) | 2 (0.20) | 2 (0.61) | 2 (0.68) | 3 (0.021) |
| Continental crust's net change over 1,000 Myr | −0.065 | −0.012 (< 0.001) | −0.041 (0.083) | −0.021 (< 0.001) | −0.046 (0.26) | −0.040 (0.023) | −0.043 (0.061) |
| Plate checks met, of 45 | 28 | 28 | 28 | 28 | 28 | 28 | 28 |
| Above 2 km, the plate model's mean over the frames after 100 Myr | 0.257 | 0.187 (0.001) | 0.195 (0.006) | 0.175 (< 0.001) | 0.247 (0.44) | 0.191 (0.001) | 0.164 (< 0.001) |
| Sea over the datum at the end, m | 820 | 726 | 664 | 955 | 915 | 746 | 1,236 (0.005) |
| Continental crust's mean thickness, km | 43.4 | 40.3 | 41.4 | 41.1 | 43.5 | 41.2 | 45.2 |

Runs meeting each criterion, of 12:

| | base | W 40 | W 40, cf | sea, W 40 | sea, W 40, cf | cap, W 40, cf | sea, W 40, cf, reach 150 | cap, W 40, cf, reach 150 |
|---|---|---|---|---|---|---|---|---|
| Land | 8 | 3 | 3 | 6 | 3 | 3 | 6 | 7 |
| Largest landmass | 7 | 5 | 4 | 8 | 11 | 8 | 9 | 8 |
| Under the sea | 5 | 3 | 2 | 6 | 9 | 8 | 5 | 8 |
| Above 2 km | 0 | 1 | 1 | 1 | 2 | 4 | 3 | 2 |
| Coast with high ground | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Under ice | 4 | 4 | 4 | 4 | 6 | 5 | 6 | 6 |
| Collisions | 3 | 3 | 5 | 4 | 8 | 7 | 6 | 4 |

The hot window alone (W 40, as PLT-390 measured on master d53cc6c4) still takes the high ground
down by a fifth and the coast by 0.08 and still drowns the continents (land 0.262, under the sea
0.399); W 20 does about as much to the high ground and drowns less. Measured from the sea, the
same window keeps the land (0.304), the largest landmass (0.152) and the drowned share (0.253)
where the base has them while the high ground falls as far (0.214, p 0.035) and the coast with it
(0.352, p 0.009): the collapsed crust now ends at the lowland's height instead of under the sea,
the continents keep more of their crust (net −0.040 against −0.065) and the sea does not rise.
The column from the sea alone (W 0) moves nothing at p under 0.24: what it changes is where the
hot window's crust goes. But criteria met per run do not rise: 2 against 2 (p 0.68).

With continental room first as well, the drowned margins fill before the ocean floor: the
largest landmass (0.193), the drowned share (0.171) and the collisions (2.10) reach their bands in
most runs, and criteria per run rise to 3 (p 0.021). But the land overshoots, 0.353 (in band in
3 runs against the base's 8), as the sea climbs to 1,236 m over the datum and the continental
slope empties (crust 200 to 2,000 m under the sea: 0.064 of the continental crust against 0.115;
the shelf 0.060 against 0.087).

### Why continental room first lifts the land and the sea

Filling the thin continental crust within reach to T_col (standing about 500 m over the sea)
before any ocean floor lifts the drowned margins out of the sea; the water they held goes to the
ocean, the sea rises, and with it T_col. The cap (PLT-413) was the test of that loop: with the
column held at 41.1 km the sea still ends 1,208 m over the datum and the land at 0.327 (p 0.040),
so it is the order, not the column's feedback, that lifts the sea (PLT-391 with W 40 alone also
lifts it, to 955 m, though not at a meaningful p: 0.44). A shorter reach, 150 km (the lower crust's channel "travels at most 150 km
into the foreland", Rey, Teyssier & Whitney 2010, as in PLT-391; PLT-257 unchanged, measured
only), fills fewer margins:

| Median of 12 (p against the base) | base | cap, W 40, cf | sea, W 40, cf, reach 150 | cap, W 40, cf, reach 150 |
|---|---|---|---|---|
| Land (0.26 to 0.32) | 0.300 | 0.327 (0.040) | 0.322 (0.046) | 0.318 (0.035) |
| Largest landmass (≥ 0.12) | 0.124 | 0.161 (0.37) | 0.159 (0.21) | 0.196 (0.17) |
| Continental crust under the sea (≤ 0.25) | 0.278 | 0.227 (0.20) | 0.266 (0.25) | 0.188 (0.038) |
| Land above 2 km, of the land (≤ 0.12) | 0.271 | 0.172 (0.002) | 0.169 (0.005) | 0.191 (0.019) |
| Coast with high ground (≤ 0.15) | 0.411 | 0.350 (0.017) | 0.345 (0.19) | 0.381 (0.37) |
| Land under ice (≤ 0.15) | 0.170 | 0.156 (0.34) | 0.150 (0.56) | 0.152 (0.36) |
| Collisions per 100 Myr (2 to 6) | 1.80 | 2.10 (0.097) | 2.00 (0.40) | 1.75 (0.88) |
| Criteria of the seven met, per run | 2 | 3 (0.19) | 2 (0.22) | 2 (0.22) |
| Continental crust's net change over 1,000 Myr | −0.065 | −0.035 (0.004) | −0.056 (0.51) | −0.078 (0.67) |
| Plate checks met, of 45 | 28 | 28 | 28 | 28 |
| Sea over the datum at the end, m | 820 | 1,208 (0.001) | 933 (0.47) | 811 (0.89) |

The last arm is the only one whose medians meet one criterion more than the base's (land,
largest landmass and the drowned share) and break none; per run it is 2 against 2 (p 0.22).

### Twenty-four runs of the last arm

Twelve more runs a side (seeds 7 to 12, both starts), the base and the last arm side by side from
the same archives, make 24 runs an arm:

| Median of 24 (p against the base) | base | cap, W 40, cf, reach 150 |
|---|---|---|
| Land (0.26 to 0.32) | 0.292 | 0.318 (0.001) |
| Largest landmass (≥ 0.12) | 0.118 | 0.183 (0.029) |
| Continental crust under the sea (≤ 0.25) | 0.286 | 0.186 (0.001) |
| Land above 2 km, of the land (≤ 0.12) | 0.259 | 0.212 (0.018) |
| Coast with high ground (≤ 0.15) | 0.401 | 0.421 (0.58) |
| Land under ice (≤ 0.15) | 0.171 | 0.159 (0.27) |
| Collisions per 100 Myr (2 to 6) | 1.95 | 1.65 (0.12) |
| Criteria of the seven met, per run | 2 | 3 (0.20) |
| Continental crust's net change over 1,000 Myr | −0.061 | −0.082 (0.24) |
| Sea over the datum at the end, m | 778 | 754 |

Runs meeting each criterion, of 24: land 19 and 13, the largest landmass 12 and 18, under the sea
9 and 17, above 2 km 1 and 4, the coast 0 and 0, ice 7 and 11, collisions 12 and 6.

The medians hold: the largest landmass, the drowned share and the high ground move towards their
targets at p under 0.03. But the land leaves its band in six more runs than the base's (its median
0.318 sits under the ceiling of 0.32), the collisions in six more, and the coast does not move: per
run the arm meets 3 criteria against 2, p 0.20. By the ship rule it stays off.

### Where the high ground stands mid-run

Ben asked whether a whole continent standing as upland 606 Myr ago is realistic. Six runs (seeds
1 to 3, both starts) of the base and of the last arm ran again with `DEEPTIME_CRUST_BUDGET_MAPS`,
which writes the plate model's samples every 50 Myr; a scratch `midrun.py` reads them with the sea
of each run's `cb_t` lines. Heights are over the step's sea; the age is the last orogenic uplift's.
"Near a belt" is within 250 km of crust an orogeny raised in the last 10 Myr; the "interior" is
high ground farther than 250 km from any such crust and farther than 300 km from the sea. Medians
of the six runs (the run ends at 0, so −600 Myr is 400 Myr in):

| Base, median of 6 | −800 | −700 | −600 | −500 | −400 |
|---|---|---|---|---|---|
| Land, of the planet | 0.326 | 0.292 | 0.303 | 0.319 | 0.292 |
| Land above 1 km, of the land | 0.330 | 0.426 | 0.448 | 0.558 | 0.567 |
| Land above 2 km, of the land | 0.190 | 0.273 | 0.285 | 0.363 | 0.362 |
| High ground an orogeny raised in the last 10 Myr | 0.56 | 0.55 | 0.53 | 0.55 | 0.51 |
| High ground near a belt | 0.77 | 0.82 | 0.78 | 0.81 | 0.78 |
| High ground in an interior | 0.13 | 0.14 | 0.18 | 0.16 | 0.19 |
| Quiet high ground on crust over T_c = 50 km | 0.24 | 0.33 | 0.34 | 0.29 | 0.39 |
| of it, with room within 200 km | 0.28 | 0.21 | 0.22 | 0.24 | 0.23 |
| The high ground's crust, median, km | 56.5 | 56.9 | 57.6 | 57.4 | 57.5 |
| Largest connected ground above 2 km, Mkm² | 12.7 | 13.9 | 18.5 | 18.9 | 15.5 |
| Land in landmasses of 5 Mkm² or more standing over half above 1 km | 0.17 | 0.18 | 0.47 | 0.84 | 0.69 |
| The continental land's median height over the sea, m | 544 | 829 | 1,041 | 1,608 | 1,547 |
| Its median crust, km | 35.1 | 37.3 | 39.9 | 42.4 | 43.6 |
| Continental crust of 41 ± 1.5 km, median height over the sea, m | 1,325 | 1,291 | 1,177 | 1,395 | 1,012 |
| Sea over the datum, m | 61 | 85 | 198 | −6 | 374 |

How long ground stays high: of the continental crust an orogeny last raised 0 to 10 Myr before,
0.39 to 0.49 stands above 2 km; 10 to 20 Myr, 0.29 to 0.45; 20 to 40 Myr, 0.22 to 0.42; 40 to
100 Myr, 0.10 to 0.31 (its median crust 34 to 40 km); over 100 Myr, 0.06 to 0.21. With the last
arm the high ground is 0.17 to 0.23 of the land at these times, the largest connected region 7.7
to 11.8 Mkm², the upland landmasses 0.09 to 0.27 of the land and the continental land's median
0.50 to 0.95 km over the sea.

What this says:

- **Plateau interiors are not the main term.** They hold 0.13 to 0.19 of the high ground. About
  four fifths of it lies within 250 km of an active belt and half was raised in the last 10 Myr.
  The quiet thick crust (0.24 to 0.39 of the high ground) lies mostly in and beside the belts, and
  a quarter of it has room within the collapse's reach. A flow that levels interiors (Bird 1991:
  an isolated high becomes "pancake shaped, with a flat central plateau and steeper flanks
  spreading outward") would act on about a sixth of the high ground.
- **The belts are too many and too wide.** The largest connected ground above 2 km is 13 to 19
  Mkm², 4 to 6 times Earth's largest ground above 3 km (2.95 Mkm², Tibet; "Against Earth" above;
  Earth's region above 2 km was not measured, so the ratio is an upper bound). That is the audit's
  gap 2: collision fronts 2.4 to 4.8 times Earth's length (docs/science/physics-gap-audit.md).
- **The upland continent of 400 to 500 Myr in is freeboard over a low sea.** At −500 Myr 0.84 of
  the land lies in large landmasses standing more than half above 1 km. Their continental land
  stands 1.0 to 1.6 km over the sea on a median 40 to 44 km of crust, because the sea is then
  −6 to 374 m over the datum (820 m at the end) and the model's Airy frame puts 41 km of crust
  1.0 to 1.4 km over such a sea. Earth's shields, on about the same crust (41.1 km, Christensen &
  Mooney 1995), stand "a few hundred meters" over the sea (Whitehead & Clift 2009, both as read
  for PLT-412). The crust thickens through the run (the continental land's median 35 to 44 km)
  while the sea rises only later. So the mid-run upland is mostly normal crust standing high over
  a sea that has not yet risen: the sea level and the crust budget (the audit's gap 3), not the
  collapse.

So the answer to Ben's question is no, not as it stands, and the main terms are the belts' extent
and the mid-run freeboard. Collapse that never reaches plateau interiors is real (the reach and
the same-plate filter) but carries the smaller part.

### As a thin viscous sheet (PLT-414, PLT-415)

The coordinator asked for continents that spread sideways under their own weight, if the
diagnosis made interiors the main term. It did not, but the audit's gap 1 ranks continent-scale
spreading first, so the law is built as a switch and measured on all seven measures.

*The sources.* England & McKenzie (1982, abstract) treat the lithosphere as "a thin sheet of power
law material" that "flows in response to forces applied to its boundaries ... and to forces in its
interior arising from gradients in crustal thickness"; a weak sheet (Argand number over 10) "is
unable to sustain appreciable crustal elevation contrasts", and the forces from thickness contrasts
"are great enough to produce net extension in the region of thick crust". Houseman & England
(1986, abstract): with n over 3 and Ar of 1 to 10 a plateau forms. The potential energy that turned
Tibet to east-west extension is 5 to 10 × 10¹² N/m (England & Houseman 1989, abstract), and
buoyancy forces are "most responsible for driving extension where it is observed in the Basin and
Range" (Sonder & Jones 1999, abstract). The depth-averaged viscosity is about 10²² Pa s for the
Tibetan lithosphere (England & Molnar 1997, abstract) and the Basin and Range (Flesch et al. 2000,
abstract; California's shear zone 10²¹).

*The law.* With `collapse_sheet_viscosity_pa_s` η_0 above 0, a quiet column an orogeny raised also
thins in pure shear, in plane strain, at ε̇ = ΔU / (4 η L) (the depth-integrated stress difference
L (τ_xx − τ_zz) = 4 η L ε̇, with ε̇_zz = −ε̇_xx, balances ΔU), releasing q = T (1 − e^(−ε̇ δt)) and at most
T − T_col. ΔU is the column's potential energy at its Airy height over the step's sea
(`forces::column_energy`, FRC-014) less that of the collapse's normal column T_col (PLT-412, capped
by PLT-413), L the lithosphere's 100 km (FRC-010), and η = η_0 e^(a / τ_s) with a the time since
the column's last orogenic uplift and τ_s = 62.8 Myr, McKenzie (1978)'s thermal time (PLT-415: the
exponential form is the model's; η is 100 times η_0 at 290 Myr). A column releases the larger of
this and the threshold law's q. Crust no orogeny raised (cratons, platforms) does not flow: the
audit's minimal strength field (since 2026-10-25 the sheet reads the lithosphere's thermal state in
the tag's place: "The collapse reads the thermal state", PLT-581). At η_0 = 10²² Pa s a 4.2 km column under a sea at the datum
(ΔU about 5 × 10¹² N/m, England & Houseman's figure) thins 3.9 % per Myr as its orogeny ends
(an e-fold of 26 Myr), a 2 km one 1.3 % (75 Myr); a 255 m eclogite root pull (PLT-281) brings the
test column to 3.56 % per Myr.

*Against a channel's diffusivity.* The sheet relaxes a column at its own rate r instead of
diffusing it; over a plateau of half-width ℓ the same decay is a diffusivity K = r ℓ² / π², about
990 km²/Myr for the 4.2 km column and 330 km²/Myr for the 2 km one at ℓ = 500 km. The thin-channel
law K = ρ g h³ / (12 μ) with Clark & Royden (2000, abstract)'s 15 km channel gives 2.4 × 10⁵
km²/Myr at their 10¹⁸ Pa s under the low-gradient margins and 240 km²/Myr at 10²¹ Pa s under the
steep ones (the arithmetic is the project's). The sheet sits at the stiff end of that range, beside
the plates' own channel flow (K₀ = 1,500 km²/Myr, PLT-300). Bird (1991, abstract) expects such
extrusion to be "marginal under shields and platforms", important under elevated plains and
"dominant beneath high plateaus and/or hot, delaminated regions".

*What it is not.* It is the local limit of the thin sheet: each column extends at the rate its
own contrast with the lowland sets; nothing solves the sheet's velocity field, and n = 1, not the
papers' n = 3. The crust it releases goes where the threshold law's goes, to the nearest room on
the same plate within `collapse_reach_km`. Of the audit's reach limits ("Scope failures") it
removes the 50 km floor (a column thins to T_col) and, in the arms, the 200 km reach (set to
5,000 km, the plate's size, so a continent's edge moves out over its own ocean floor); PLT-412
raises the room from T_n = 35.1 km to T_col. It keeps the same-plate filter (`plates/collapse.rs`,
`plates/flow.rs`, `plates/rate.rs`), the quiet time (PLT-259: an orogeny's push holds a belt up),
the belts' strength height of 5.1 km and its lost overflow (PLT-253) and the 70 km cap (PLT-258).
The full solve (a velocity on the plate lattice with n = 3, the audit's "What it takes") is still
gap 1's.

#### The sheet's arms

The sheet's binary is this branch at 46887626 (an archive of the commit; with the switches off it
runs the base's planets bit for bit). Every arm below sets PLT-412 and PLT-413 at 41.1 km and no
hot window. *Plate-wide* is `collapse_reach_km` 5,000; *cf* PLT-391.

Exploration first (the coordinator's rule: the fewest seeds that answer the question), seeds 1
and 2, both starts, 4 runs an arm against the base's same 4 (with 4 runs a side the smallest
two-sided p is 0.029):

| Median of 4 (p against the base) | base | plate-wide, cf, no sheet | sheet 10²², plate-wide, cf | sheet 10²², plate-wide | sheet 10²², reach 200 |
|---|---|---|---|---|---|
| Land (0.26 to 0.32) | 0.269 | 0.360 (0.029) | 0.385 (0.029) | 0.331 (0.34) | 0.321 (0.057) |
| Largest landmass (≥ 0.12) | 0.086 | 0.170 (0.057) | 0.246 (0.029) | 0.162 (0.11) | 0.126 (0.11) |
| Continental crust under the sea (≤ 0.25) | 0.314 | 0.143 (0.029) | 0.133 (0.029) | 0.287 (1.0) | 0.277 (0.49) |
| Land above 2 km, of the land (≤ 0.12) | 0.231 | 0.235 (1.0) | 0.142 (0.11) | 0.135 (0.029) | 0.140 (0.057) |
| Coast with high ground (≤ 0.15) | 0.407 | 0.522 (0.029) | 0.404 (0.89) | 0.307 (0.20) | 0.290 (0.20) |
| Land under ice (≤ 0.15) | 0.152 | 0.173 (0.15) | 0.169 (0.38) | 0.095 (0.11) | 0.165 (0.89) |
| Collisions per 100 Myr (2 to 6) | 1.80 | 1.80 (1.0) | 1.85 (1.0) | 2.10 (0.38) | 1.90 (0.55) |
| Criteria of the seven met, per run | 2 | 2 (0.44) | 3 (0.16) | 3 (0.13) | 2 (0.35) |
| Continental crust at the end, of the planet | 0.355 | 0.341 (0.49) | 0.387 (0.11) | 0.452 (0.029) | 0.403 (0.057) |
| Sea over the datum at the end, m | 820 | 1,151 (0.057) | 1,375 (0.029) | 1,331 (0.029) | 877 (0.89) |

The plate-wide reach with continental room first and no sheet moves crust out of no high ground
(0.235): it is the sheet that takes the high ground down, by two fifths (0.142, 0.135 and 0.140),
and the threshold-law arms never did that. Without cf the coast falls with it (0.307) and the ice
(0.095, in band in all 4 runs). What the high ground loses the continents' area gains: the crust
the sheet releases fills the nearest room on the plate, which is mostly ocean floor without cf,
and filled ocean floor turns continental at T_col, so the continental crust grows from 0.355 to
0.452 of the planet and the sea climbs about 500 m. With cf the drowned margins fill first: the
drowned share falls to 0.13 and the land to 0.385, far over its ceiling. At the threshold law's
200 km reach the sheet takes the high ground down as far (0.140) and fills less (continental crust
0.403, the sea 877 m), but the coast and the ice move less and criteria per run stay at 2. The
plate-wide arm without cf went on to 12 runs.

#### Twelve runs of the sheet

Seeds 1 to 6, both starts, against the base's 12 (the base's binary and runs of "The arms"; the
runs are deterministic). One run, scattered continents seed 5, stopped at −102 Myr on the surface
guard (below), so the arm has 11:

| Median (p against the base) | base, 12 | sheet 10²², plate-wide, 11 |
|---|---|---|
| Land (0.26 to 0.32) | 0.300 | 0.323 (0.013) |
| Largest landmass (≥ 0.12) | 0.124 | 0.169 (0.049) |
| Continental crust under the sea (≤ 0.25) | 0.278 | 0.280 (0.90) |
| Land above 2 km, of the land (≤ 0.12) | 0.271 | 0.145 (0.001) |
| Coast with high ground (≤ 0.15) | 0.411 | 0.316 (0.014) |
| Land under ice (≤ 0.15) | 0.170 | 0.111 (0.007) |
| Collisions per 100 Myr (2 to 6) | 1.80 | 2.10 (0.13) |
| Criteria of the seven met, per run | 2 | 3 (0.063) |
| Continental crust's net change over 1,000 Myr | −0.065 | +0.003 (< 0.001) |
| Plate checks met, of 45 | 28 | 29 (0.14) |
| Above 2 km, the plate model's mean over the frames after 100 Myr | 0.257 | 0.172 (< 0.001) |
| Continental crust at the end, of the planet | 0.358 | 0.425 (< 0.001) |
| Sea over the datum at the end, m (the census's) | 820 | 1,139 (0.003) |
| Continental crust's mean thickness, km | 43.4 | 43.7 (0.74) |
| The land's mean height over the sea, m | 1,374 | 983 |

Runs meeting each criterion, base (of 12) and sheet (of 11): land 8 and 2, the largest landmass
7 and 10, under the sea 5 and 4, above 2 km 0 and 3, the coast 0 and 0, ice 4 and 8, collisions
3 and 7.

What it does:

- **The sheet takes the high ground down by almost half** (0.271 to 0.145, in band in 3 runs
  against none), and the coast (0.411 to 0.316) and the ice (0.170 to 0.111, in band in 8 runs
  against 4) with it. Over the run the plate model's high ground falls from 0.257 to 0.172. No
  collapse law measured on this lane did as much: the threshold law's best was 0.167 with cf.
- **The land goes over its ceiling.** 0.323, in band in 2 runs against the base's 8. The
  continents gain area at the same mean thickness (43.4 and 43.7 km): the crust budget's parts
  over the run give the collapse 0.43 of the planet in continental area made (gross, cumulative)
  against the base's 0.16, and the founder 0.13 against 0.08 (the thickest crust's own fill of
  ocean room to T_col, PLT-277: its volume to the mantle goes from −72 to +806 in the budget's
  units, so less founders), less 0.10 more taken by subduction. The continental crust's net
  change goes from −0.065 to +0.003 and its area from 0.358 to 0.425 of the planet. The filled
  ocean floor's water goes to the sea, which rises 320 m, and the land still climbs.
- **The drowned share does not get worse** (0.278 to 0.280), against the audit's expectation from
  PLT-390: the sheet's crust ends at T_col over the sea, not at T_n under it. It does not get
  better either: it is out of band in both.

*The surface guard.* The stopped run's surface reached 242 km under the sea at L1 cell (325, 112),
past the guard's 237 km. A rerun from a checkpoint (scratch `div.sh`, `div2.sh`, `onset.py`) shows
why. From at least −400 Myr, 50 to 250 cells stand with the plate model's elevation over 20 km,
up to its 30 km cap (`max_elevation_m`), over a median 31 to 38 km of crust (11 to 24 km over its
Airy height) with no orogeny, while the surface's flexure under them is a median 10 to 34 km (up
to 290 km). The patch rides its plate from the north pole's rows at −368 Myr to row 110 at −112
Myr, and the surface under it falls 37 km, 93 km and 242 km under the sea in the last three
steps. The plates' own laws
(the collapse, the sheet, the spread's receivers) never raise a column past its Airy height
(`collapse::toward`): the height comes from the surface's feedback (the whole offset of the
surface over the plate model, `PlatesStage::apply_surface_feedback`, clamped at 30 km), so the
fault is in the coupling of the surface's flexure to the plates, outside this lane. The arms
trigger it more often than the base (each run's `guard_worst_m`). The base's 24 runs never take
the surface more than two Earth reliefs (39.6 km) from the sea (worst 37.8 km) and never end with
a peak over 20 km. The arms that move much crust by the collapse do in 1 to 3 runs of 12: the
worst is 110 km with W 40, 89 km with W 20, 57 km with W 40 and cf, 41 km with the sea column, W 40
and cf, 218 km with the cap, W 40 and cf (19 km short of the stop), 86 km with the same at reach
150 (of 24), and 46 km in a finished run of the sheet. Patches at the 30 km cap end 1 run of 12 of
the sea column with W 40 and cf, 3 of the sheet's 11, and 1 of 4 of each exploration arm with the
plate-wide reach (none of the 4 at reach 200).

*By the ship rule it stays off*: criteria per run rise from 2 to 3 at p 0.063, not a meaningful p,
and it breaks the land criterion that the base meets in 8 of 12 runs; one run stops on the guard.
Its `--set` keys for the combined arm with the trench-budget lane are
`--set tectonics.collapse_sea_column=true --set tectonics.collapse_sea_column_max_km=41.1
--set tectonics.collapse_sheet_viscosity_pa_s=1e22 --set tectonics.collapse_reach_km=5000`
(and, to try the threshold law's reach, without the last).

*Did the arms need the trench side?* Yes, for the land. Every arm that moves the high ground's
crust into room (PLT-391, the sheet) turns ocean floor continental and nothing takes the crust
back: the continental crust's net change rises to zero or above and the land passes 0.32. The
drowned share is kept, not mended, by the sheet; with cf it is mended but the land goes further
over. A sink for continental crust at the trenches (sediment subduction and subduction erosion,
with PLT-312) is what would let the sheet's lower high ground stand with the land in band.

### What is on

**Since 2026-10-23 the sheet is on by default**, plate-wide and with the cap (the four keys above),
together with the trench budget: "On by default (2026-10-23)", at the end of this page. What follows
is this section's state when it was written.

Nothing new. PLT-412 to PLT-415 are in the code as switches, off in the defaults and every
built-in preset, and the default runs are the base's bit for bit (the switches off). The arm
closest to shipping by the medians (the sheet, plate-wide) fails the ship rule on the land and
on its p; the arm closest by the ship rule (cap, W 40, cf, reach 150) is 3 criteria against 2 at
p 0.20 over 24 runs. The table
driver behind this section's arms is `tools/review/v1-item1-table.sh` (with
`tools/review/v1-item1-table.py`): `run` makes an arm's runs and their census, `table` the seven
measures, the criteria per run and the Mann-Whitney p against a base; the base's runs are listed
for other lanes in `/data/ban/deeptime-runs/pool/BASES.md` on the test VM. These runs predate
the shared run pool (`tools/review/pool/`, master 2db74acd); new arms go through it.

### What is left, and for whom

1. *The extent of the belts* (Ben's: how plates are made and moved, the audit's gap 2): about four
   fifths of the high ground lies within 250 km of an active belt in every frame, and the largest
   connected high ground is 4 to 6 times Tibet's. Fronts 2.4 to 4.8 times Earth's length are the
   main term, and no collapse law reaches them while the orogeny that raises them is active.
2. *The trench budget and the sea* (the trench-budget lane, the audit's gap 3): moving the high
   ground's crust into drowned and ocean room lifts the land past its 0.32 ceiling in the arms
   that move most (PLT-391, the sheet arms above), because nothing takes crust back. The combined
   arm with that lane (this branch's switches as `--set` keys against the shared base) is the next
   measurement; check each alone and both together on 3 or 4 seeds first. Measured 2026-10-23
   ("The trench budget", "Twelve runs of the trench budget with the sheet"): with the trench
   budget the land is back in the band (0.300) and the continental budget balances, at 3.08
   criteria a run against the sheet's 3.50 and the base's 2.17 (p 0.073).
3. *The mid-run freeboard* (the sea level's and the crust budget's): the continents stand 1.0 to
   1.6 km over a low sea 400 to 500 Myr in. Whether the sea's late rise or the crust's growth is
   wrong is the sea-level task's question, measured on these frames.
4. *The lithosphere's strength and heat* (the audit's gap 4, Ben's: an architecture change): the
   sheet's strength is a time since uplift standing in for a thermal state, and the founder and the
   rise cap stand in for root removal.
5. *The full sheet* (gap 1): a velocity solve with n = 3, across plate boundaries where the
   continents are welded, replacing the reach, the room, the same-plate filter and the flow
   (PLT-254 to PLT-259, PLT-300). Worth building after items 1 and 2, when the belts it acts on
   are Earth's size and the crust it moves has a sink.
6. *The ice* follows the high ground (the audit's M6): 0.159 to 0.171 of the land at 24 runs, it
   moves only with land above 2 km; with the sheet it falls to 0.111.
7. *The surface guard under the collapse arms* (the surface flexure's owner; `surface/` was not
   touched here): plate columns held 20 km or more over their Airy height by the surface's
   feedback, with the flexure under them at tens of km, appear in the arms that move the most
   crust and stopped one run of the sheet ("The sheet's arms", the surface guard). Settled
   2026-10-22 (ISO-091, [flexure.md](flexure.md), "The settling's feedback"): the coupling step
   was the surface flexure booking every rise of a building column as crust thickened, 7 times
   the rise, including the settling's move of a column to its Airy height plus the flexure's
   offset (ISO-078). A stiff plate held that root up, the offset grew, the settling moved the
   column again, and the loop ran to the 30 km cap. The flexure now books only the column's own
   crust's Airy change (`plates/crust_uplift_mm_yr`, ISO-092), either sign. The reproduction was
   scattered continents seed 5 with the sheet's `--set` keys, which reached the stop at step 449.

## The trench budget (2026-10-22)

The coordinator's task on V1 item 1, gap 3, explored on master 883a8ef6 (key 54658b9718e7, twelve
runs, named `trench-base883` in the pool) and measured last on master 18ace5d7 with the surface
flexure's load on gained crust (key a7d34642f456, the pool's `base`, twelve runs): sediment subduction (PLT-440) and subduction erosion (PLT-441,
PLT-442) on Earth's numbers, PLT-312 if the measures support it once the losses are in, every loss
booked against Stern & Scholl 2010 Table 1, and Ben's D5 trenches and deep floor: which law digs
them, Earth's share of floor deeper than 6 km from SRTM15+, and the measures "sea floor deeper
than 6 km, share" and "deepest point". Ledger PLT-440 to PLT-444 (new), HYP-050 and HYP-051 (new);
sources in the bibliography, "The trench budget". Code: `plates/ridges.rs` (the resampling's
accrete branch), `plates/subduction.rs` (`Forearcs`, `erode_forearcs`), `plates/pools.rs` (the
trench's release, `gdh1_floor_m`), `plates/mod.rs`, `plates/gap_budget.rs` (the part `forearc`),
`plates/crust_budget.rs`, `params.rs`; nothing in `surface/`. Tools: the pool measure `trench`
(`tools/review/pool/measure_trench.py`, registry `measures_trench.json`), the loss table
`tools/review/loss-budget.py` (its sediment subduction and subduction erosion rows),
`tools/review/earth-deep-floor.py` (Earth on SRTM15+) and `tools/review/trench-depth.py` (a bake's
trenches). Every rule is a `--set` switch. PLT-440, PLT-441 and PLT-443 were off by default when
this section was written and are on since 2026-10-23 at the values under "What is on" (the next
section, "On by default (2026-10-23)"); PLT-312 stays off.

### Ben's D5 trenches against Earth's deep floor

On Ben's D5 single-player bake (Lite New World, seed 1, supercontinent; `trench-depth.py`) 0.21 of
the sea floor is deeper than 6 km below the sea and 0.032 deeper than 8 km, and the deepest cell
is 12,803 m; its deep troughs run 78 to 175 km long and 25 to 90 km wide. The base's twelve runs
(medians, the final frame's L1 grid of equal-area cells): 0.160 deeper than 6 km, 0.030 deeper
than 8 km, the deepest 13,003 m, the 99th percentile 8,957 m.

Earth (`earth-deep-floor.py` on GMT's `earth_relief_15m_p`, SRTM15+ at 15', Tozer et al. 2019,
area-weighted cells below 0 m; HYP-050, HYP-051): deeper than 5 km 0.208 of the sea floor, 6 km
0.0073 (0.0076 on 100,000 Fibonacci points), 7 km 0.0010, 8 km 0.0003, 10 km none; median depth
4,098 m, p90 5,356, p99 5,953; the deepest 15' cell 9,242 m. Harris et al. 2014 (Table 5, read in
full): the hadal floor deeper than 6,000 m is 3,437,930 km², 0.95 % of the mapped sea floor, and
the 56 trenches 1,967,350 km² (0.544 %). Stern 2021 (Table 1): Challenger Deep 10,920 m, four
trenches deeper than 10 km, the 24 listed maxima's median 8,246 m, and "all trenches deeper than
8000 m are nonaccretionary". The model's deep floor is 20 to 30 times Earth's.

### Which law digs them

Split of the base's sea floor deeper than 6 km (twelve runs, medians; a scratch `deepdiag.py`
on the pool's dumps: the depth below the step's sea of the final elevation, and the plate model's
own elevation z_p over the datum):

| what the plate model holds there | share of the sea floor |
|---|---|
| ocean floor in GDH1's range (z_p at or above -5,652 m: old floor under a sea 820 m over the datum) | 0.049 |
| ocean floor pulled below GDH1's floor and not at z_t (-9,900 < z_p < -5,652) | 0.088 |
| ocean floor at the trench law's z_t = -10,000 m | 0.0004 |
| continental crust | 0.0005 |
| all deeper than 6 km | 0.160 |

Of the floor deeper than 8 km (0.030), 0.0297 is pulled floor. The pulled floor deeper than 6 km
is a trench this step in only 11.8 % of its cells; 0.18 of it lies within one L1 cell of a
current trench, 0.23 within two, 0.31 within four, 0.45 within eight; by regime it is 0.61 open
basin, 0.13 ridge, 0.12 trench and 0.13 back-arc. Current trench cells are 0.018 of the sea floor.

So three laws dig the deep floor:

1. *Fossil trench depth* (0.088 of the sea floor, all of the floor deeper than 8 km). The trench
   law pulls a TRENCH-flagged ocean sample towards z_t with a 2 Myr half-life (PLT-052, PLT-053),
   and the flag moves with the front. Under GDH1 (PLT-352) the floor sinks by its age's increment
   each step and nothing brings it back to its age's depth, so the depth a trench gave a sample
   stays with it after the front has moved on, and the floor carries it across the basin. This is
   the plate model's, and the trench's release (PLT-443, below) is its fix.
2. *Old floor under a high sea* (0.049). GDH1 puts floor older than about 55 Myr within a few
   hundred metres of 5,651 m below the datum; the sea stands 820 m over the datum on the base (the
   ocean's thick oceanic-labelled crust holds it up: sea-level.md), so that floor is 6.2 to 6.5 km
   under the sea. It falls as the sea falls, and the sea falls when the ocean's surplus sediment
   goes down at the trenches (PLT-440).
3. *The surface stage's own deepening.* The final elevation is the plate model's plus what the
   surface stage does (its flexure and sediment): under floor deeper than 10 km it adds a median
   468 m, and at the deepest point about 2.2 km (the plate model's deepest is 10,820 m, the final
   13,003 m, medians). That is `surface/`, outside this lane; the lead for the surface lane is
   the flexure's water load ("the sea over the cell against the sea at the reference") over a
   trench, not verified here.

What stays after those: the trench rows themselves. The plate model's trench is one row of 71 km
samples along every oceanic front, held at z_t = -10,000 m over the datum, so about 10.8 km under
a sea 800 m over it whatever the slab's age. Earth's trench maxima run from 3.1 km (Cascadia) to
10.9 km and grow with the slab's age (Jarrard 1986, Table 11: Ad = -0.81 + 0.0185 As + 0.0816 Dip
km below the regional floor, 60 % of the variance; Stern 2021, Fig. 7, read by eye: 2 to 5 km of
relief from young to 140 Myr slabs). The model's oceanic fronts are 3.4 to 3.7 times Earth's
43,000 km (146,000 km on the base), which the plate geometry sets.

*The surface guard's worst cell* (the coordinator's question, 2026-10-23: the guard's worst stands
14.8 to 21.9 km from the sea in every run on master 18ace5d7, [flexure.md](flexure.md)). A scratch
build that prints the worst cell of every step past 11 km (never committed), on master 18ace5d7
with this lane's switches off, supercontinent seed 1, 1,000 Myr (the run's worst 15,818 m, at
−806 Myr): the worst cell is ocean floor below the sea in every step (never more than 2 cells
stand 11 km above it), and in 483 of the 485 steps it is trench-pulled floor: the plate model
holds it at z_t in 366 and pulled below GDH1's floor for its age in 117, and at a current trench
in only 37. The plate model holds those cells 10.3 km under the sea (median) and the surface stage
carries them 2.8 km deeper (4.1 km at the 17 steps past 14 km). With this lane's rules on (the
release, s = 0.12, E = 90; the same seed) the fossil floor is gone (cells deeper than 11 km: a
median of 145 a step to 3), the steps past 11 km fall from 485 to 322, and the worst to 14,778 m.
It is then a current trench row (294 of the 322 steps) that the plate model holds 6.9 km under the
sea (median), with the surface stage 4.8 km below that, and 7.8 km at the 6 steps past 14 km,
5 of them poleward of 60°. So the guard's worst is this section's pulled floor plus the surface
stage's own deepening; with the release, only the surface stage's part at trench rows is left past
Earth's 10.9 km.

### The laws

*Sediment subduction (PLT-440).* Until now a resampling accreted every overridden column's
conserved pools onto the column on top (PLT-276), so no sediment went down at a trench (the
gap sea-level.md found: the ocean holds five to six times GlobSed's sediment). With
`tectonics.sediment_subduction_share` s, of ocean floor that went under (hidden, or of a plate the
top overrides) the share s of its sediment and sedimentary rock goes down with the slab and the
rest accretes. Earth's share: "Globally, the median proportion of the rock delivered as sediment
to the trench that is accreted from the subducting plate is only 17%" (Clift & Vannucchi 2004,
[39]), so s = 0.83; von Huene & Scholl 1991 put it at all of it at type 2 margins, about 80 % at
small and medium prisms and about 70 % at large prisms. Earth's flux: 1.1 km³ a year at ocean
margins and 0.1 at sutures (Stern & Scholl 2010, Table 1).

*Subduction erosion (PLT-441, PLT-442).* A front erodes its forearc where the convergence normal
to it is at least 60 mm/yr and the incoming plate carries less than 1 km of sediment there
("Tectonic erosion is favored in regions where convergence rates exceed 6 ± 0.1 cm yr-1 and where
the sedimentary cover is <1 km", Clift & Vannucchi 2004). Such a front takes E km³ per km and
Myr from the overriding plate's crust beyond a normal ocean floor's 7 km within 200 km of the
front, the nearest samples first; continental columns sink by Airy and other crust by the crust-per-metre line (PLT-264). E = 90 is Clift & Vannucchi's
"global average rate of crustal loss" per km of erosive margin (von Huene & Scholl 1991: 25 to
50, "conservative" 30; Stern & Scholl 2010: 35 averaged over all 43,000 km of margin; Table 1:
1.4 km³ a year at ocean margins, 0.3 at sutures).

*The trench's release (PLT-443).* With `tectonics.trench_release_half_life_myr` > 0, visible
ocean floor that is not at a trench this step and stands below the bare floor of its age
(`pools::gdh1_floor_m`) rises back towards that floor at the half-life; floor above its age's
depth (seamounts, plateaus, sediment) is left alone. The half-life is the trench law's own 2 Myr
(PLT-053). A trench's depth is then the slab's to hold while the front is there, and floor away
from a trench stands at its age's depth ("Maximum trench depth is strongly correlated with
regional seafloor depth away from the trench", Jarrard 1986). It is the model's consistency, not a
measured rate: `game-choice`.

All three are `--set` keys, 0 by default, and the branch with them off is bit-identical to master
(`pool.sh alias` on supercontinent seed 1, 100 Myr: world state and dynamics record identical; key
69953ab39a57 resolves to master 883a8ef6's 54658b9718e7, and after the merge of master 18ace5d7
key eb4ef634e607 to adf36da1f017, which master's a7d34642f456 also resolves to).

### The arms on master 883a8ef6 (exploration, seeds 1 and 2)

Through the pool (`tools/review/pool/pool.sh`): earthlike_quick_16k, 1,000 Myr, both starts,
2 threads, `+DEEPTIME_CRUST_BUDGET=1`; the base is master 883a8ef6's twelve runs (key 54658b9718e7,
named `trench-base883` here since `base` has moved on), the arms 4 runs (seeds 1 and 2; `sed83` and
`ero90` 2 runs), and the branch's keys 2ca399c21604 and 69953ab39a57 are aliased bit-identical to
master's with the switches off. Keys: `rel` the release (half-life 2 Myr), `s83` / `s12` the
sediment subduction share 0.83 / 0.12, `e` the erosion at E = 90, `sheet` the high ground's sheet
(`collapse_sea_column`, 41.1 km, 10²² Pa s, reach 5,000 km: "As a thin viscous sheet" above),
`arcn` PLT-312. Medians, the Mann-Whitney p against the base in brackets.

| arm | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | criteria per run (of 7): median, mean | sea over the datum, 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|
| target | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 | | |
| base (883a8ef6, 12 runs) | 0.300 | 0.124 | 0.277 | 0.271 | 0.411 | 0.170 | 1.80 | 2, 2.25 | 362 / 745 |
| s83 (2 runs) | 0.189 (0.022) | 0.080 | 0.471 (0.022) | 0.147 | 0.217 (0.022) | 0.117 | 1.45 | 1.5, 1.50 | −213 / −354 |
| e (2 runs) | 0.328 | 0.205 | 0.177 (0.044) | 0.336 | 0.485 | 0.176 | 1.35 | 2, 2.00 | 134 / 266 |
| s83, e | 0.191 (0.001) | 0.070 | 0.466 (0.001) | 0.144 (0.002) | 0.304 (0.008) | 0.076 (0.008) | 1.00 (0.006) | 1, 1.25 | −659 / −927 |
| rel | 0.285 | 0.146 | 0.334 (0.058) | 0.180 (0.042) | 0.291 (0.013) | 0.154 | 2.30 | 3.5, 3.25 (0.13) | 884 / 1,250 |
| rel, e | 0.256 (0.078) | 0.110 | 0.348 | 0.228 | 0.331 (0.058) | 0.152 | 1.65 | 1.5, 1.50 | 702 / 1,107 |
| rel, s12, e | 0.240 (0.020) | 0.103 | 0.328 | 0.232 | 0.359 | 0.146 | 1.50 | 1.5, 1.75 | 571 / 657 |
| rel, s83, e | 0.156 (0.001) | 0.048 (0.004) | 0.558 (0.001) | 0.143 (0.004) | 0.255 (0.002) | 0.144 | 1.15 (0.038) | 1, 1.25 | −215 / −258 |
| sheet (11 runs) | 0.323 (0.013) | 0.169 (0.045) | 0.280 | 0.145 (< 0.001) | 0.316 (0.015) | 0.111 (0.007) | 2.10 | 3, 3.09 (0.063) | 706 / 1,078 |
| rel, sheet | 0.293 | 0.117 | 0.367 | 0.139 (0.001) | 0.256 (0.004) | 0.116 (0.030) | 1.95 | 2.5, 2.50 | 728 / 1,414 |
| rel, e, sheet | 0.308 | 0.136 | 0.283 | 0.117 (0.008) | 0.260 (0.078) | 0.133 (0.058) | 1.80 | 3.5, 3.25 (0.13) | 769 / 1,192 |
| rel, e, sheet, arcn | 0.315 | 0.138 | 0.155 (0.13) | 0.220 | 0.430 | 0.165 | 1.45 | 2.5, 2.50 | 692 / 808 |
| rel, s83, e, sheet | 0.186 (0.001) | 0.043 (0.008) | 0.486 (0.001) | 0.092 (0.001) | 0.236 (0.004) | 0.077 (0.008) | 1.45 | 2.5, 2.50 | 126 / −108 |

The measure `trench` on the same runs (registry `tools/review/pool/measures_trench.json`; Earth
from HYP-050, HYP-051 and Table 1):

| arm | deeper than 6 km, share (Earth 0.0073) | deepest, m (Earth 9,242 at 15', 10,920) | deeper than 8 km (Earth 0.0003) | p99, m (Earth 5,953) | deepest: plates / surface, m | sediment subducted, km³/yr (1.1 + 0.1) | forearc eroded, km³/yr (1.4 + 0.3) | erosive / oceanic fronts, km (22,925 / 43,000) |
|---|---|---|---|---|---|---|---|---|
| base | 0.160 | 13,003 | 0.0301 | 8,957 | 10,820 / 13,003 | 0 | 0 | 0 / 146,155 |
| s83 | 0.066 (0.022) | 11,594 | 0.0132 | 8,172 | 9,670 / 11,594 | 8.88 | 0 | 0 / 159,947 |
| e | 0.139 | 12,408 | 0.0352 | 8,874 | 10,361 / 12,408 | 0 | 0.91 | 17,985 / 152,627 |
| rel | 0.189 | 11,839 | 0.0042 (0.001) | 7,281 (0.001) | 9,988 / 11,839 | 0 | 0 | 0 / 137,796 |
| rel, e | 0.112 (0.058) | 10,843 (0.004) | 0.0029 (0.001) | 6,892 (0.001) | 9,763 / 10,843 | 0 | 0.84 | 16,644 / 147,015 |
| rel, s12, e | 0.052 (0.001) | 10,785 (0.008) | 0.0017 (0.001) | 6,714 (0.001) | 9,330 / 10,785 | 1.36 | 0.83 | 16,711 / 141,345 |
| rel, s83, e | 0.0105 (0.001) | 10,798 (0.001) | 0.0003 (0.001) | 6,030 (0.001) | 8,617 / 10,798 | 8.83 | 1.19 | 25,014 / 166,164 |
| sheet | 0.207 (0.003) | 13,168 | 0.0377 | 9,208 | 11,060 / 13,168 | 0 | 0 | 0 / 126,232 |
| rel, e, sheet | 0.176 | 12,050 | 0.0052 (0.001) | 7,426 (0.001) | 10,162 / 12,050 | 0 | 0.69 | 13,597 / 123,493 |
| rel, s83, e, sheet | 0.0141 (0.001) | 10,112 (0.001) | 0.0005 (0.001) | 6,257 (0.001) | 8,588 / 10,112 | 7.66 | 1.17 | 24,461 / 151,505 |

What they say:

1. *The release takes the fossil trench depth away and nothing else.* The floor deeper than 8 km
   falls from 0.030 to 0.004 and the 99th percentile from 8,957 to 7,281 m; of what is left deeper
   than 6 km by the plate model's own pull (0.015 of the sea floor) 52 % is a current trench and
   95 % lies within 8 cells of one. But the floor deeper than 6 km rises (0.160 to 0.189): the sea
   rises with it (884 m over the datum after 100 Myr, 1,250 at the end, against 362 and 745), since
   floor that stands at its age's depth holds less water, and 0.178 of the sea floor is then old
   floor in GDH1's range under that sea. With the sea at the datum the plate model's floor deeper
   than 6 km would be 0.011. The release is also the arm with most criteria met (3.5 per run against
   2, p 0.13), from the high ground and the coast.
2. *Sediment subduction at Earth's share fixes the deep floor and drowns the land.* With s = 0.83
   the sea falls to 200 to 900 m below the datum, the floor deeper than 6 km to 0.0105 (Earth
   0.0073), deeper than 8 km to Earth's 0.0003 and the 99th percentile to Earth's 6,000 m; but the
   land falls to 0.16 to 0.19 and half the continental crust is under the sea (the next section).
   Note (the M5 lane, 2026-10-25): these arms ran before the motion fix (PLT-191), on plates a
   deferred collision held still, so the drowning is not a reading of the share; the section
   "Sediment by margin type and the margin's sediment" reads Clift & Vannucchi's share again on
   moving plates (PLT-660), together with the margin rule (PLT-661), whose ratchet drowns the land
   there for its own reason; the share alone is not read again.
3. *Erosion alone* (2 runs) lowers the sea and keeps the land: continental crust under the sea
   0.177 (p 0.044), land 0.328.
4. *PLT-312* (arcn) on the release, the erosion and the sheet: the arcs' crust falls from 4.25 to
   3.70 km³ a year (Earth 2.5) and the continental crust under the sea from 0.283 to 0.155, but the
   land above 2 km rises from 0.117 to 0.220 and the coast with high ground from 0.260 to 0.430, the
   collisions fall to 1.45, and the criteria from 3.5 to 2.5 per run; one of its 4 runs reached
   82 km from the sea (the surface guard's worst, on master 883a8ef6's flexure). The measures do not
   support it: it stays off.
5. *The deepest point is the surface stage's.* Every arm's plate model is within Earth's band or
   just over it (8,588 to 10,162 m with the release); the surface stage adds 1.1 to 2.2 km to it.

### The crust budget against Stern & Scholl 2010, Table 1

`tools/review/loss-budget.py` over the crust budget's globals (`+DEEPTIME_CRUST_BUDGET=1`), seeds 1 and
2 of both starts (4 runs; means over the 1,000 Myr, km³ a year; Earth's from Table 1, read in full,
quotes in the bibliography). The arms: `base` (master), `trench-rel` (the release alone),
`trench-rs12e` (the release, s = 0.12 and E = 90), `trench-rse` (the release, s = 0.83 and E = 90).

| Table 1's row, or the model's | Earth | base | trench-rel | trench-rs12e | trench-rse |
|---|---|---|---|---|---|
| additions: intra-oceanic (island) arcs | 1.5 | 3.13 | 3.04 | 3.31 | 3.89 |
| additions: continental arcs | 1.0 | 1.68 | 1.51 | 1.66 | 1.97 |
| additions: crust-suturing zones | 0.2 | no rule | no rule | no rule | no rule |
| additions: rifts, hotspots, rifted margins | 0.5 | 0.04 | 0.04 | 0.04 | 0.04 |
| losses, ocean margins: sediment subduction (PLT-440) | 1.1 | 0 | 0 | 1.40 | 8.61 |
| losses, ocean margins: subduction erosion (PLT-441) | 1.4 | 0 | 0 | 0.87 | 1.19 |
| of it continental crust | | | | 0.30 | 0.47 |
| losses, crust-suturing zones (sediment 0.1, erosion 0.3, continental crust 0.3) | 0.7 | 2.67 | 2.33 | 2.24 | 1.21 |
| model only: foundering | | 0.95 | 0.76 | 0.77 | 0.34 |
| model only: the rate cap and the resampling drops | 0 | 1.05 | 1.05 | 0.74 | 0.60 |
| flow: ocean-floor felsic rock accreted at resamplings | | 20.82 | 18.33 | 16.51 | 6.73 |
| flow: off the continents by the surface stage | | 16.02 | 12.71 | 12.80 | 9.62 |
| erosive fronts, km (Clift & Vannucchi: 22,925 of 40,225) | | 0 | 0 | 17,467 | 25,152 |
| island-arc and continental-arc fronts, km (Earth 14,500 and 27,500) | | 44,015 and 101,289 | 42,222 and 93,178 | 47,052 and 99,804 | 51,493 and 111,848 |
| net change of the continental volume | 0 | −0.47 | +0.45 | −1.31 | −3.91 |

What the table says:

1. *Both new losses come out at Earth's size per km of front.* Erosion at E = 90 over the fronts
   that meet PLT-442's test takes 0.87 to 1.19 km³ a year, against Table 1's 1.4 at ocean margins;
   the erosive fronts are 17,000 to 25,000 km, Clift & Vannucchi's 22,925. The shortfall is the
   200 km band running out of crust beyond 7 km on island arcs' forearcs (a third of the eroded
   crust is continental).
2. *Sediment subduction at Earth's share takes eight times Earth's flux.* At s = 0.83 the slabs take
   7.5 to 9.9 km³ a year; Table 1 has 1.1. Earth's sediment reaching the trenches is about 1.3 to
   1.5 km³ a year (von Huene & Scholl 1991: 1.5 km³ at 43,500 km of margin), and the model's ocean
   floor carries 12 to 16 km³ a year off the continents by the surface stage, to be scraped back at
   resamplings (16 to 21 km³ a year of ocean-floor felsic rock accreted). The share is Earth's; what
   it acts on is not. The share that gives Table 1's flux is s = 0.12 (1.40 km³ a year), `calibrated`.
3. *The model's continents live on that return flow.* Its additions are 4.8 to 5.9 km³ a year
   (Earth 3.2: the arcs are twice Earth's per year, on fronts 3.4 times Earth's length), and its
   losses before this lane were 3.6 to 3.7 km³ a year (sutures 2.3 to 2.7, four times Earth's
   0.7; foundering 0.8 to 1.0; the rate cap and resampling drops 1.0), a net loss already on the
   base. Earth's two subduction losses on top (2.3 to 2.5 km³ a year at s = 0.12 and E = 90) leave
   the continents losing 1.3 km³ a year (a quarter of their volume over the run); at s = 0.83 they
   lose 3.9.
4. *The release adds nothing and takes nothing* (no crust moves; the floor rises to its age's depth),
   but the sutures' loss (2.33 against 2.67) and the foundering fall a little with it and the net
   turns slightly positive (+0.45 against −0.47 km³ a year; why was not traced).

### The interaction check on master 18ace5d7 (seeds 1 to 4)

This is the coordinator's check, 2026-10-23. It compares the base, the trench budget alone, the
sheet alone, and both together.

- **Branch.** It runs from this branch with master 18ace5d7 merged in (0e7e2410, key
  eb4ef634e607).
- **Bit-identity with the switches off.** `pool.sh alias` on supercontinent seed 1 found the
  branch bit-identical to adf36da1f017 at 100 and at 1,000 Myr. This holds for the base's
  argument set and for the sheet's. So `trench-m-base4` and `trench-m-sheet4` are master's `base`
  and `fg-sheet` runs for seeds 1 to 4, read from this branch.
- **Arms.**
  - `rs12e` is the release, s = 0.12 and E = 90.
  - `re` is the release and E = 90.
  - `sheet` is the high ground's four keys.
- **Runs.** Each arm has 8 runs (seeds 1 to 4 of both starts), with `+DEEPTIME_CRUST_BUDGET=1`.
- **Statistics.** Each cell is the median, with the Mann-Whitney p against `trench-m-base4` in
  brackets.

| arm | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | criteria per run: median, mean | sea over the datum, 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|
| target | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 | of 7 | |
| base | 0.292 | 0.191 | 0.253 | 0.261 | 0.378 | 0.169 | 1.80 | 3, 2.50 | 357 / 601 |
| sheet | 0.342 (0.003) | 0.210 | 0.234 | 0.162 (0.001) | 0.354 | 0.115 (0.010) | 2.15 | 3.5, 3.50 (0.012) | 564 / 988 |
| rs12e | 0.265 (0.021) | 0.109 (0.10) | 0.294 | 0.278 | 0.409 | 0.170 | 1.55 | 2, 2.00 (0.28) | 467 / 559 |
| re | 0.248 (0.021) | 0.072 (0.010) | 0.347 (0.021) | 0.225 | 0.301 (0.050) | 0.170 | 1.95 | 1, 1.38 (0.037) | 578 / 1,131 |
| rs12e, sheet | 0.298 | 0.183 | 0.279 | 0.163 (< 0.001) | 0.337 | 0.098 (0.010) | 1.85 | 3, 2.88 (0.49) | 604 / 853 |
| re, sheet | 0.317 (0.10) | 0.162 | 0.261 | 0.166 (0.028) | 0.382 | 0.115 (0.015) | 1.50 | 3, 2.62 | 707 / 1,093 |

Runs that meet each criterion (of 8; `tools/review/pool` v1census, the targets above):

| arm | land | largest | under the sea | above 2 km | coast | ice | collisions | per run |
|---|---|---|---|---|---|---|---|---|
| base | 5 | 7 | 4 | 0 | 0 | 2 | 2 | 2.50 |
| sheet | 2 | 7 | 5 | 2 | 0 | 7 | 5 | 3.50 |
| rs12e | 5 | 4 | 3 | 1 | 0 | 1 | 2 | 2.00 |
| re | 3 | 2 | 0 | 0 | 0 | 2 | 4 | 1.38 |
| rs12e, sheet | 5 | 6 | 3 | 1 | 0 | 6 | 2 | 2.88 |
| re, sheet | 4 | 5 | 4 | 2 | 0 | 4 | 2 | 2.62 |

The trench measure:

| arm | deeper than 6 km (Earth 0.0073) | deepest, m (Earth 10,920) | deeper than 8 km (Earth 0.0003) | p99, m (Earth 5,953) | deepest: plates / surface, m | sediment subducted, km³/yr (1.1 + 0.1) | forearc eroded, km³/yr (1.4 + 0.3) | erosive / oceanic fronts, km |
|---|---|---|---|---|---|---|---|---|
| base | 0.145 | 12,943 | 0.032 | 8,967 | 10,672 / 12,943 | 0 | 0 | 0 / 142,792 |
| sheet | 0.196 (0.010) | 13,062 | 0.029 | 9,203 | 11,043 / 13,062 | 0 | 0 | 0 / 127,330 |
| rs12e | 0.054 (0.002) | 11,154 (< 0.001) | 0.0022 (< 0.001) | 6,951 (< 0.001) | 9,387 / 11,154 | 1.46 | 0.92 | 18,129 / 148,234 |
| re | 0.111 (0.028) | 11,554 (0.010) | 0.0025 (< 0.001) | 6,911 (< 0.001) | 9,549 / 11,554 | 0 | 0.84 | 17,010 / 146,832 |
| rs12e, sheet | 0.099 (0.007) | 10,953 (< 0.001) | 0.0030 (< 0.001) | 7,115 (< 0.001) | 9,561 / 10,953 | 1.29 | 0.85 | 17,256 / 135,788 |
| re, sheet | 0.137 | 11,558 (0.005) | 0.0033 (< 0.001) | 7,179 (< 0.001) | 10,043 / 11,558 | 0 | 0.84 | 16,842 / 129,661 |

The crust budget (`tools/review/loss-budget.py`, means over 8 runs, km³ a year; Earth from
Table 1):

| row | Earth | base | sheet | rs12e | re | rs12e, sheet | re, sheet |
|---|---|---|---|---|---|---|---|
| island and continental arcs | 2.5 | 4.95 | 4.18 | 5.17 | 5.04 | 4.70 | 4.45 |
| sediment subduction (PLT-440) | 1.1 | 0 | 0 | 1.47 | 0 | 1.28 | 0 |
| subduction erosion (PLT-441) | 1.4 | 0 | 0 | 0.90 | 0.85 | 0.87 | 0.80 |
| crust-suturing zones | 0.7 | 3.00 | 2.20 | 2.39 | 2.55 | 1.88 | 2.16 |
| foundering and the rate cap | | 1.48 | 0.06 | 1.15 | 1.27 | 0.06 | 0.03 |
| net change of the continental volume | 0 | −0.62 | +0.72 | −0.97 | −0.20 | +0.07 | +0.80 |
| continental crust at the end, share of the planet | | 0.346 | 0.414 | 0.332 | 0.363 | 0.392 | 0.410 |

What they say:

1. *The trench budget alone takes crust from a budget that was already losing it.* With
   rs12e the continental volume falls 0.97 km³ a year, against the base's 0.62. Land drops to
   0.265 (p 0.021) and the largest landmass to 0.109. The arm meets 2 criteria a run, against
   the base's 2.5. Without sediment subduction (`re`) the sea stands 1,131 m over the datum at
   the end, and half the continental crust that is left drowns (0.347).
2. *The sheet alone adds crust.* The founder and the rate cap almost stop: 1.48 km³ a year fall
   to 0.06, because the sheet spreads the high ground's crust rather than founder it. The
   continental volume then grows by 0.72 km³ a year and land reaches 0.342.
3. *Together the budget balances, and land is back in the band.* The trench budget's 2.15 km³ a
   year of new losses offset the sheet's gain. The net is +0.07 km³ a year (Earth's is 0) and
   land is 0.298. On these seeds the pair meets 2.88 criteria a run, against the sheet's 3.50
   and the base's 2.50. It wins back the land in 3 more runs than the sheet. It loses the
   collision rate in 3 runs, the ice in 1, the high ground in 1, the largest landmass in 1 and
   the drowned crust in 2. The collisions fall with the trench budget in every pairing (1.80 to
   1.55 alone, 2.15 to 1.85 with the sheet); this was not traced.
4. *`re` (no sediment subduction) is worse than `rs12e` everywhere.* Its land and sea are both
   off, so it is not the final arm.

### Twelve runs of the trench budget with the sheet

`trench-m-rs12e-sheet` is the release, s = 0.12, E = 90 and the sheet's four keys, on seeds
1 to 6 of both starts. It is compared with master's `base` and `fg-sheet` (12 runs each). The p
in brackets is against `base`; the last row also gives the p against the sheet.

| arm | land | largest landmass | under the sea | above 2 km | coast | ice | collisions | criteria per run: median, mean | sea, 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|
| base | 0.292 | 0.144 | 0.253 | 0.245 | 0.369 | 0.168 | 1.75 | 2.5, 2.17 | 385 / 690 |
| sheet | 0.342 (0.003) | 0.230 (0.061) | 0.234 | 0.157 (0.001) | 0.354 | 0.106 (0.001) | 2.10 | 3.5, 3.50 (< 0.001) | 505 / 988 |
| trench and sheet | 0.300 (0.19; 0.046 against the sheet) | 0.188 | 0.249 | 0.163 (< 0.001) | 0.337 | 0.121 (0.014) | 1.90 | 3, 3.08 (0.073; 0.20 against the sheet) | 604 / 850 |

The runs that meet each criterion, of 12:

| arm | land | largest | under the sea | above 2 km | coast | ice | collisions |
|---|---|---|---|---|---|---|---|
| base | 6 | 8 | 6 | 0 | 0 | 4 | 2 |
| sheet | 2 | 10 | 7 | 5 | 0 | 11 | 7 |
| trench and sheet | 7 | 9 | 6 | 2 | 0 | 8 | 5 |

The trench measure on the same runs:

- **Deep floor.** The floor deeper than 6 km is 0.099 of the sea floor (base 0.145, sheet
  0.200). The floor deeper than 8 km is 0.0029 (base 0.032), and the 99th percentile 7,066 m
  (base 8,967).
- **Deepest point.** It is 10,864 m (base 13,093), in Earth's band in 7 of 12 runs. No run
  meets the deep-floor criterion.
- **Fluxes.** Sediment subducted is 1.29 km³ a year (Table 1: 1.1 at ocean margins) and forearc
  eroded 0.88 (Table 1: 1.4).
- **Erosive fronts.** They total 17,467 km of 133,794 km of oceanic front. Clift & Vannucchi
  have 22,925 km of 40,225.

The crust budget over the 12 runs:

- **New losses.** Sediment subduction takes 1.29 km³ a year and erosion 0.87 (0.31 of it
  continental crust).
- **Other losses.** The sutures take 1.95 (base 2.97, Earth 0.7). The founder and the rate cap
  together take 0.06 (base 1.53).
- **Net change of the continental volume.** It is +0.03 km³ a year (base −0.38, sheet +0.82).
  Continental crust ends at 0.390 of the planet (base 0.355, sheet 0.421).

The surface guard's worst is 16,020 m (base 15,898, sheet 16,136; p 0.71).

*Does the pair keep the sheet's criteria gain and bring land back into the band?* Land, yes:
0.300 is in the band in 7 runs of 12, against the sheet's 2 and the base's 6 (p 0.046 against
the sheet). The gain, partly: 3.08 criteria a run, against the sheet's 3.50 and the base's
2.17. That is two thirds of the sheet's gain, at p 0.073 against the base. No criterion the base
meets is lost: every column has at least as many runs as the base's. Against the sheet it gives
up the high ground in 3 runs, the ice in 3, the collisions in 2, the largest landmass in 1 and
the drowned crust in 1. By the ship rule (a meaningful p, and no criterion of the base broken)
it stays off: p 0.073 is not meaningful.

### Which law digs the deep floor, on master 18ace5d7

The same split as above (`deepdiag.py` on the pool's dumps, medians), on the merged key.

- **Rows.** "Pulled" is ocean floor the plate model holds below GDH1's floor for its age.
  "GDH1" is ocean floor within GDH1's range.
- **"Plates, from the datum".** This row is the plate model's floor deeper than 6 km below the
  datum: the deep floor if the sea stood at the datum.

| share of the sea floor | base | sheet | rs12e | re | rs12e, sheet |
|---|---|---|---|---|---|
| sea over the datum at the end, m | 747 | 1,033 | 635 | 1,167 | 914 |
| deeper than 6 km | 0.145 | 0.200 | 0.054 | 0.111 | 0.099 |
| of it pulled floor | 0.086 | 0.084 | 0.020 | 0.0095 | 0.017 |
| of it GDH1's old floor | 0.044 | 0.124 | 0.029 | 0.101 | 0.079 |
| pulled floor at a current trench, share | 0.11 | 0.09 | 0.55 | 0.67 | 0.49 |
| plates, from the datum: deeper than 6 km | 0.073 | 0.067 | 0.017 | 0.0066 | 0.0125 |
| deepest, m (plates over the datum / surface over the datum) | 13,093 (−9,775 / −12,256) | 13,418 | 11,154 (−7,021 / −10,299) | 11,554 | 10,864 (−6,586 / −9,927) |

So on master 18ace5d7:

- **Fossil trench depth.** The release takes it away. Pulled floor falls from 0.086 to 0.010 to
  0.020 of the sea floor. Half or more of what is left is a trench this step, and 93 % of it
  lies within 8 cells of one.
- **What the plate model holds against the datum.** Its floor deeper than 6 km is then 0.007
  to 0.017. Earth's is 0.0073.
- **What is left is the sea's.** GDH1's old floor lies 5.65 km below the datum. With the sea
  600 to 1,170 m over the datum, that floor stands deeper than 6 km under the sea. It falls as
  the sea falls: 0.029 at 635 m and 0.101 at 1,167 m. The sea stands high because the ocean
  carries several times GlobSed's sediment (sea-level.md). Sediment subduction at Earth's share
  would fix that, but on this model's ocean it drowns the land. The routing that puts 12 to
  15 km³ a year of the continents' rock on the ocean floor is the surface stage's.
- **The deepest point.** The surface stage carries the deepest cell 3.3 km below the plate
  model's (the trace above: current trench rows, mostly poleward of 60°). This is the surface
  lane's.

### What is on

**Since 2026-10-23 the pair below is the default** (the trench budget at the recommended values
with the sheet): "On by default (2026-10-23)", the next section. What follows is this section's
state when it was written: nothing new was on by default.

- **The switches.** PLT-440 to PLT-443 are in the code as `--set` switches, 0 in the defaults
  and in every built-in preset until 2026-10-23. With them off, the runs are master's bit for bit.
- **PLT-312.** It stays off (the exploration above).
- **Recommended values for an arm.** The release at 2 Myr. Erosion at E = 90 with
  PLT-442's 60 mm/yr and 1 km test. Sediment subduction at s = 0.12: Table 1's flux on this
  model's ocean (`calibrated`). Earth's share s = 0.83 takes eight times Table 1's flux here and
  drowns the land.
- **Why nothing ships.**
  - Alone, the trench budget loses criteria: 2 a run against 2.5 on matched seeds. It takes land
    to the band's floor, because the model's continents were already losing crust and Earth's
    two subduction losses come on top.
  - With the sheet it brings land back into the band. It balances the continental budget (+0.03
    km³ a year over 12 runs). It keeps two thirds of the sheet's gain (3.08 criteria a run
    against 2.17, p 0.073).
  - It takes the deep floor deeper than 8 km to a tenth, and the deepest point to Earth's band.
  - By the ship rule it stays off until a larger ensemble gives a meaningful p. The 24-run
    ensemble gave it: 3.42 criteria a run against 2.33, p 0.004 (the next section).
- **The command line.** The pair's keys:
  `--set tectonics.trench_release_half_life_myr=2 --set tectonics.sediment_subduction_share=0.12
  --set tectonics.subduction_erosion_km3_per_km_myr=90 --set tectonics.collapse_sea_column=true
  --set tectonics.collapse_sea_column_max_km=41.1 --set tectonics.collapse_sheet_viscosity_pa_s=1e22
  --set tectonics.collapse_reach_km=5000`.
- **The pool.** The arm `trench-m-rs12e-sheet` is in the pool: key eb4ef634e607, argkey
  86d11d8b, 12 runs.

### What is left, and for whom

1. *The surface stage's deepening at trench rows* (the surface lane's, `surface/`). With the
   release, the deepest cell stands 6.6 to 7.0 km below the datum in the plate model. The
   surface stage carries it 3.3 km deeper.
   - The surface guard's worst (15.6 to 16.9 km from the sea in every arm) is that deepening at
     current trench rows. Past 14 km it is mostly poleward of 60° (the trace above).
   - The lead, not verified here: the flexure's water load over a trench.
2. *The sediment's routing* (the surface stage's and the sea level's).
   - The surface stage moves 11.5 to 15.5 km³ a year of the continents' rock onto ocean floor. The
     resamplings accrete 14.7 to 20.7 km³ a year of it back. Earth's sediment reaching the
     trenches is about 1.5.
   - While that loop runs, the ocean holds several times GlobSed's sediment. The sea stands 600
     to 1,170 m over the datum, and GDH1's old floor stands deeper than 6 km under it. That is
     the deep floor that is left: 0.03 to 0.10 of the sea floor, against the plate model's own
     0.007 to 0.017 from the datum.
   - Sediment subduction at Earth's share (0.83) is the physical setting. It becomes usable once
     the ocean's sediment is Earth's size.
3. *The fronts' length* (the plate geometry's, Ben's gap 2).
   - The oceanic fronts are 3.1 to 3.5 times Earth's 43,000 km.
   - The arcs add 4.2 to 5.2 km³ a year against Earth's 2.5.
   - The sutures take 1.9 to 3.0 against Earth's 0.7.
   - Per km of front, both new losses are Earth's size. Their totals follow the model's front
     length.
4. *The trench's depth against the slab's age* (this lane, not done). The trench law holds every
   oceanic front at z_t = −10 km over the datum, whatever the slab's age. Earth's trenches run
   from 3 to 11 km and deepen with the slab's age (Jarrard 1986, Table 11). A z_t from the
   slab's age is the next step for the trench rows themselves.
5. *The collision rate under the trench budget* (not traced). The collisions fall from 1.75 to
   1.55 a 100 Myr with the trench budget alone, and from 2.10 to 1.90 with the sheet.
6. *A larger ensemble for the pair* (the coordinator's). Twelve more seeds of `base`, `fg-sheet`
   and the pair would settle whether p 0.073 is a real gain. Done on 2026-10-23: seeds 1 to 12,
   p 0.004, and the pair is the default (the next section).

## On by default (2026-10-23)

The coordinator's call under Ben's ship rule for V1 item 1 (a Mann-Whitney p under 0.05 on 24 runs an
arm, and no criterion the base meets in fewer runs; decisions log 2026-10-23): the high ground's sheet
("As a thin viscous sheet", above) and the trench budget (the previous section) are the default and
every built-in preset's since 2026-10-23. Seven keys changed:

- **The sheet.** `tectonics.collapse_sea_column` true (PLT-412), `collapse_sea_column_max_km` 41.1
  (PLT-413), `collapse_sheet_viscosity_pa_s` 1e22 (PLT-414) and `collapse_reach_km` 5,000, a plate's
  size (PLT-257; 200 before). PLT-415's strength growth (62.8 Myr) is read with PLT-414 and keeps its
  value.
- **The trench budget.** `tectonics.sediment_subduction_share` 0.12 (PLT-440),
  `subduction_erosion_km3_per_km_myr` 90 (PLT-441) and `trench_release_half_life_myr` 2 (PLT-443).
  PLT-442's test (60 mm/yr, 1 km) is read with PLT-441 and keeps its value. PLT-312 stays off.

The code is in `params.rs` (`TectonicParams::default`) and the twelve built-in presets
(`presets/earthlike_*`, `presets/pipeline/pipeline_*`, `presets/plates/plates_*`); the Java side
packs those files into the jar, so the Create World screen and the workbench read the same values. The
test preset `tests/presets/planet_world_4k.json` lists none of the six other keys and takes them from
the default; it keeps its own `collapse_reach_km` 200 and has the collapse off (`collapse_myr` 0).

### Twenty-four runs

`pool.sh table --base base --arm fg-sheet --arm trench-m-rs12e-sheet`: `earthlike_quick_16k`,
1,000 Myr, both starts, seeds 1 to 12, the crust budget on (`+DEEPTIME_CRUST_BUDGET=1`), 2 threads.
`base` is master 3fc91491 (key a7d34642f456), `fg-sheet` the sheet's four keys (key adf36da1f017) and
`trench-m-rs12e-sheet` the seven keys (key eb4ef634e607, argkey 86d11d8b). Medians of the runs, the
p against `base` in brackets. The default since 2026-10-23 is the last arm's runs bit for bit (below).

| arm | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | continental crust's net change over the run | criteria met per run: median, mean | sea over the datum, median after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|
| target | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 | | of 7 | |
| base | 0.292 | 0.139 | 0.264 | 0.261 | 0.389 | 0.170 | 1.75 | −0.064 | 3, 2.33 | 393 / 701 |
| fg-sheet | 0.347 (< 0.001) | 0.236 (0.001) | 0.241 (0.10) | 0.156 (< 0.001) | 0.347 (0.031) | 0.129 (< 0.001) | 2.00 (0.10) | −0.001 (< 0.001) | 3, 3.33 (0.002) | 551 / 1,029 |
| trench and sheet | 0.306 (0.065) | 0.189 (0.068) | 0.238 (0.28) | 0.161 (< 0.001) | 0.344 (0.038) | 0.123 (< 0.001) | 1.95 (0.16) | −0.035 (0.004) | 3, 3.42 (0.004) | 597 / 843 |
| **default since 2026-10-23** | 0.306 | 0.189 | 0.238 | 0.161 | 0.344 | 0.123 | 1.95 | −0.035 | 3, 3.42 | 597 / 843 |

The runs that meet each criterion, of 24 (`pool.sh table --json`, counted against the targets):

| arm | land | largest | under the sea | above 2 km | coast | ice | collisions | criteria, all runs |
|---|---|---|---|---|---|---|---|---|
| base | 15 | 15 | 11 | 0 | 0 | 8 | 7 | 56 |
| fg-sheet | 4 | 22 | 14 | 9 | 0 | 17 | 14 | 80 |
| trench and sheet, the default | 15 | 20 | 14 | 6 | 0 | 15 | 12 | 82 |

*Does the pair meet the ship rule?* Yes. It meets 3.42 criteria a run against the base's 2.33 at
p 0.004, and every column has at least as many runs as the base's: the land is in the band in 15 runs,
as in the base, where the sheet alone keeps it in 4. Against the twelve runs of seeds 1 to 6 (3.08 at
p 0.073, the previous section) seeds 7 to 12 moved the mean up and the p under 0.05. No criterion is
met for the coast with high ground in any arm.

*What changes in a default run.* The high ground falls from 0.26 of the land to 0.16 and the ice from
0.17 to 0.12; the largest landmass grows from 0.14 to 0.19 of the planet. The continental crust's net
loss falls from 0.064 of the planet to 0.035. The sea stands higher over the datum, 597 m after
100 Myr and 843 m at the end against 393 and 701: the sheet's spreading crust displaces the ocean
("Why continental room first lifts the land and the sea", above).

*The default is the measured arm.* The new default on this branch was checked against the arm's key:

- **The pool's alias check** (`pool.sh alias HEAD eb4ef634e607 --check-seed 1 --length 1000` with the
  seven keys as `--set` and `+DEEPTIME_CRUST_BUDGET=1`): the seven keys on the new defaults give
  supercontinent seed 1's state and record bit for bit (`state_sha256`, `rec_sha256`).
- **The default with no `--set`** (the pool's arm `ship-default`, seeds 1 and 2 of both starts): each
  run's `state_sha256` in `meta.json` equals the `trench-m-rs12e-sheet` run of the same start and
  seed.

### What is left

The trench budget's "What is left" (above) holds for the default as it stands: the surface stage's
deepening at trench rows, the sediment's routing (the sea 600 to 1,170 m over the datum), the fronts'
length, the trench's depth against the slab's age and the collisions under the trench budget. The
high ground's "What is left" holds too, the belts' extent first. The coast with high ground (0.34 of
the coast against at most 0.15) is the criterion no arm moves.

## The polar peaks (2026-10-23)

The coordinator's lane after the trench rows ([flexure.md](flexure.md), "The trench rows", "What is
left"): in two of three traced runs the plate model's columns stood 15.5 to 29.4 km above the sea
on the polar rows, held at the rise guard's ceiling (PLT-288, 3.5 mm/yr) step after step and
reaching for the preset's 30 km cap (`max_elevation_m`). On the pool's base (master bf8477b2, key
7f9d8f115049, 24 runs) the guard's worst is 11.9 to 30.0 km, median 13.7 km, and over 24 km in four
runs (scattered 4 and 7, supercontinent 5 and 7). Ledger ISO-120 (new). Code: `surface/flexure.rs`
and `surface/mod.rs` (the law), `plates/peaks.rs` with hooks in `plates/mod.rs` and
`plates/gap_budget.rs` (the trace, measurement only).

### The trace

`DEEPTIME_PEAK_TRACE=<m>` (`plates/peaks.rs`, off by default, no rule reads it) prints, every plate
step, the visible samples that end the step higher than `<m>` (at most six a step, the highest
first): latitude, crust kind, regime and flags, the crust's thickness, the height before the
surface's feedback, after it and at the step's end, the Airy height, the surface's flexure `F` under
the sample and the settling's target, and with the gap budget on (`DEEPTIME_GAP_BUDGET`) each
process's change of the height and the thickness over the step and the largest parts of the gap.
The arm `pp-trace` (key 81404d9815b5, the base's code with the trace) ran seeds 1 to 4, both
starts, 1000 Myr, with `+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_GAP_BUDGET=1 +DEEPTIME_PEAK_TRACE=10000`.
Its V1 census and guard equal the base's runs to the digit (three runs compared; the state hashes
differ, since the budget keeps its parts in the state). Scratch readers `peaks.py`, `peaks_agg.py`
and `peaks_polar.py` (in the lane's `build/polar-peaks/`, not kept).

Over the eight runs, 314 lines (a sample above 10 km at a step): 252 continental, 62 oceanic.

- **What holds them up is the settling.** 236 of the 252 continental lines were last raised by the
  settling (ISO-078), and 188 of the 209 whose `F` is known stand at its target, the Airy height
  plus `F`, within 50 m. Quartiles: `F` 10.8, 14.3 and 20.1 km; the height over the Airy height
  10.7, 13.6 and 19.3 km; the crust 25.0, 33.7 and 39.2 km thick. Summed over the continental
  lines, the settling raised them 1,499 km in the step and the subduction stage's heights
  (`subduction::fronts_finish`) took 717 km back; each step the one undid the other.
- **What feeds them is the collapse.** The crust that reached them in the step came from the
  collapse (799 km summed over the lines; 78 lines took more than 1 km in their step) and the
  thickest crust's spread (PLT-277, 274 km, nearly all in scattered 4), against 104 km of
  resampling's accretion and 22 km from the arcs. They are back-arc (67 lines), rift (56),
  collision (48) and passive-margin (23) samples, with no collision under way at the step
  (`coll_mm_yr` 0 in every quartile).
- **Why they are room.** The collapse and the spread fill continental crust thinner than their
  column (PLT-412's T_col, at most 41.1 km; PLT-277's 50 km) and ocean floor. These columns are 25
  to 40 km thick, so they are room, whatever their height: a column held 15 km over its Airy height
  takes crust as if it were lowland.
- **`F` grows, and nothing bounds it.** `F` is the load the surface's plate holds away from local
  isostasy, over the restoring density. Under a column the collapse fed step after step it grew by
  1.5 to 3.7 km a step. The worst run's worst column, scattered 4, sample 65563, a rift column at
  10 to 15°S: from −438 to −432 Myr the collapse gave it 10.0, 10.0, 10.0 and 8.6 km of crust a step
  (the thickening's rate limit, PLT-287), its crust stayed 37.6 to 39.3 km thick (the surface
  eroded about as much: the gap budget's erosion part fell 16 to 20 km a step), and `F` under it went 17.9, 21.6, 22.8 and 24.4 km. At −432 Myr it began the
  step at 19.8 km, the subduction stage took it to 10.0 km, and the settling put it back at
  25.5 km, its Airy height 1.1 km plus `F`. The largest `F` traced was 46.9 km (scattered 4) and
  28.5 km on the polar rows (scattered 3, sample 22820 at 75°S, a back-arc column of 13.7 km
  standing at 22.4 km). The plate is elastic: no rule limits what it holds.
- **What does not bring them down.** The rise guard (PLT-288) caps a step's rise from the column's
  height at the step's start; these columns start the step already high, so the guard lets them
  stay there, and they rise towards the cap by up to 7 km a step when `F` grows. The 70 km cap
  (PLT-258) does not apply to crust 25 to 40 km thick. The sheet's collapse takes crust from columns
  thicker than its column, so it adds to these columns rather than taking from them. The arcs' and
  fronts' heights pull them down within the step, and the settling restores them.

**Are the poles special?** Not in kind. The polar rows (60° and over, 13.4 % of the planet) hold
99 of the 314 lines (61 continental), 64 of them from one run (scattered 3, whose columns stand
there for 31 steps); without it they hold 35 of 248 (14 %), their share of the area. The worst run,
scattered 4, peaks at 7 to 16°S, and all its 135 lines lie between 60°S and 60°N (46 of them within
30° of the equator). Nothing in the polar lines looks degenerate: their convergence (38, 47 and
68 mm/yr at the quartiles) is like the low latitudes' (33, 36 and 59), their regimes are the same
kinds (collision 21, back-arc 15, rift 10, arc 6, craton 6 of the 61), and the same loop (collapse
in, `F` up, settling) raises them. What is particular to the poles, and was not
measured here, is the surface flexure's metric: the solve treats every cell as a square of the
equator's width, so on the polar rows it bends about 1 / cos⁴ φ too stiffly north to south
([flexure.md](flexure.md), "Simplifications and limits"), which lets a pole-side plate hold more of a
column's root than it should.

A second, smaller kind is oceanic and polar: 38 of the polar lines are ocean floor (22 ridge, 16
basin samples) 10.1 to 16.2 km high, their Airy height median −1.9 km and `F` −0.5 to 1.2 km. The
surface's feedback raised them (0.4, 1.6 and 2.5 km in the step at the quartiles, up to 6.7 km),
and the largest part of their gap is the carry's (the gap budget's part for the surface's
semi-Lagrangian advection, the L0/L1 sampling and the clamps: 9.6, 12.8 and 16.0 km at the
quartiles, up to 29.5 km), not the flexure's. The highest, supercontinent 2 at 82°N, a ridge sample
at 16.2 km. Not traced further here, and the yield (below) does not reach them.

### The law

The trace's loop has one link that Earth does not have: a plate holding 10 to 47 km of rock away
from local isostasy. No plate holds more than about 2 to 4 km (ISO-093: the largest free-air
anomalies, −380 mGal at the Puerto Rico Trench and up to +300 mGal over Hawaii), because rock
yields before it carries such stresses. The fix bounds that link, not the collapse's room or the
rise guard: the plate's yield, ISO-120 ([flexure.md](flexure.md), "The plate's yield"). After each
solve of the surface's flexure a cell's plate holds at most σ_max = Δg_max / 2πG of load away from
local isostasy, 2.75 km of mantle at Δg_max = 380 mGal; the excess sinks under the cell alone. With
it, the settling's target over a column the collapse keeps feeding is its Airy height plus at most
2.75 km, so the crust the collapse gives it raises it only by its Airy share. Off by default when
it landed (`--set surface.flexure.yield_anomaly_mgal=380`); on by default since the flexure-faults
lane, 2026-10-23 ([flexure.md](flexure.md), "The plate's yield"). Code: `surface/flexure.rs` (commit c8134072, pool
key 08c9d7d679c0; the later commits on the branch change only comments and formatting).

The collapse's room (thickness, not height) is left as it is. A column 15 km over its Airy height
taking crust as lowland is wrong only because the column stands there; with the yield it no longer
does, and a room rule that also read the height would have needed a law of its own (open question
below).

### The arms

Pool arm `pp-yield380` (key 08c9d7d679c0, argkey e042f3ab): `--set
surface.flexure.yield_anomaly_mgal=380`, `+DEEPTIME_CRUST_BUDGET=1`, 1000 Myr, both starts, against
the pool's `base` (master bf8477b2, key 7f9d8f115049, argkey 2a53566b, the same arguments without
the switch). Exploration on seeds 1 to 4 (8 runs against the base's 24): the guard's worst fell from
13.7 to 12.9 km at the median (p 0.051) and its largest from 26.0 to 15.5 km, with every V1 p 0.35
or more. Final on seeds 1 to 12 (24 runs a side), medians and the two-sided Mann-Whitney p:

| arm | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | crust's net change | criteria met per run: median, mean | sea over the datum after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|
| target | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 |  | of 7 |  |
| base | 0.300 | 0.151 | 0.274 | 0.151 | 0.333 | 0.116 | 1.75 | −0.035 | 3, 3.33 | 538 / 861 |
| pp-yield380 | 0.313 (0.35) | 0.161 (0.24) | 0.253 (0.37) | 0.160 (0.50) | 0.351 (0.37) | 0.126 (0.73) | 1.80 (0.56) | −0.033 (1.0) | 3, 3.04 (0.31) | 495 (0.32) / 759 (0.18) |

| arm | the guard's worst, m | runs over 15 / 20 km | largest rise / fall of a cell in a step, m | plate cells 5 km over Airy (final frame), share | largest excess over Airy (final frame), m | highest plate elevation (final frame), m |
|---|---|---|---|---|---|---|
| base | 13716 | 9 / 4 | 6992 / 9450 | 0.0000 | 3510 | 6509 |
| pp-yield380 | 13112 (0.015) | 1 / 0 | 6524 (0.012) / 7422 (< 0.001) | 0.0000 (0.041) | 3391 (0.32) | 6495 (0.60) |

Per run, base / yield (`sc` scattered continents, `sp` supercontinent; the runs part once the yield
first acts, so a run's two columns are two different planets, not a before and after):

| run | land | largest | under the sea | above 2 km | high coast | under ice | collisions | criteria met | the guard's worst, km |
|---|---|---|---|---|---|---|---|---|---|
| sc1 | 0.294 / 0.321 | 0.166 / 0.213 | 0.272 / 0.249 | 0.168 / 0.158 | 0.346 / 0.346 | 0.096 / 0.145 | 1.50 / 1.80 | 3 / 3 | 12.4 / 15.5 |
| sc2 | 0.309 / 0.284 | 0.277 / 0.261 | 0.294 / 0.272 | 0.157 / 0.172 | 0.340 / 0.354 | 0.094 / 0.066 | 1.90 / 2.10 | 3 / 4 | 13.1 / 11.9 |
| sc3 | 0.266 / 0.238 | 0.115 / 0.151 | 0.279 / 0.466 | 0.234 / 0.088 | 0.327 / 0.255 | 0.079 / 0.077 | 2.10 / 1.40 | 3 / 3 | 18.8 / 12.2 |
| sc4 | 0.329 / 0.322 | 0.176 / 0.276 | 0.276 / 0.163 | 0.090 / 0.244 | 0.272 / 0.455 | 0.094 / 0.161 | 2.20 / 2.10 | 4 / 3 | 26.0 / 13.0 |
| sc5 | 0.299 / 0.331 | 0.124 / 0.277 | 0.295 / 0.256 | 0.161 / 0.162 | 0.365 / 0.383 | 0.143 / 0.134 | 1.30 / 1.60 | 3 / 2 | 15.0 / 13.2 |
| sc6 | 0.314 / 0.343 | 0.100 / 0.163 | 0.165 / 0.125 | 0.196 / 0.217 | 0.431 / 0.442 | 0.108 / 0.207 | 2.10 / 2.30 | 4 / 3 | 15.6 / 13.5 |
| sc7 | 0.317 / 0.283 | 0.138 / 0.108 | 0.229 / 0.309 | 0.164 / 0.155 | 0.382 / 0.327 | 0.043 / 0.144 | 2.30 / 2.20 | 5 / 3 | 30.0 / 12.6 |
| sc8 | 0.292 / 0.329 | 0.073 / 0.158 | 0.264 / 0.223 | 0.137 / 0.180 | 0.304 / 0.330 | 0.185 / 0.087 | 2.70 / 1.20 | 2 / 3 | 13.5 / 12.1 |
| sc9 | 0.260 / 0.325 | 0.110 / 0.206 | 0.384 / 0.176 | 0.118 / 0.246 | 0.277 / 0.425 | 0.118 / 0.212 | 1.60 / 1.80 | 3 / 2 | 13.7 / 12.3 |
| sc10 | 0.335 / 0.318 | 0.167 / 0.244 | 0.173 / 0.153 | 0.238 / 0.288 | 0.435 / 0.452 | 0.119 / 0.189 | 1.10 / 2.20 | 3 / 4 | 13.3 / 13.3 |
| sc11 | 0.301 / 0.313 | 0.126 / 0.206 | 0.319 / 0.176 | 0.082 / 0.291 | 0.282 / 0.434 | 0.081 / 0.153 | 1.60 / 2.00 | 4 / 4 | 13.8 / 14.1 |
| sc12 | 0.308 / 0.313 | 0.114 / 0.159 | 0.278 / 0.202 | 0.099 / 0.136 | 0.287 / 0.296 | 0.108 / 0.105 | 1.40 / 1.70 | 3 / 4 | 12.3 / 11.9 |
| sp1 | 0.290 / 0.308 | 0.148 / 0.125 | 0.306 / 0.257 | 0.099 / 0.145 | 0.301 / 0.271 | 0.081 / 0.213 | 1.10 / 1.20 | 4 / 2 | 17.5 / 13.5 |
| sp2 | 0.284 / 0.256 | 0.192 / 0.060 | 0.237 / 0.419 | 0.360 / 0.040 | 0.407 / 0.179 | 0.210 / 0.093 | 1.30 / 1.50 | 3 / 2 | 12.4 / 13.3 |
| sp3 | 0.230 / 0.314 | 0.055 / 0.240 | 0.511 / 0.217 | 0.071 / 0.167 | 0.196 / 0.348 | 0.115 / 0.108 | 1.20 / 2.00 | 2 / 5 | 14.6 / 12.9 |
| sp4 | 0.309 / 0.280 | 0.081 / 0.102 | 0.241 / 0.306 | 0.145 / 0.149 | 0.343 / 0.389 | 0.197 / 0.075 | 1.70 / 1.90 | 2 / 2 | 13.0 / 12.7 |
| sp5 | 0.289 / 0.200 | 0.205 / 0.041 | 0.325 / 0.571 | 0.106 / 0.061 | 0.286 / 0.189 | 0.071 / 0.067 | 2.80 / 2.50 | 5 / 3 | 29.1 / 12.7 |
| sp6 | 0.297 / 0.325 | 0.154 / 0.098 | 0.272 / 0.216 | 0.155 / 0.158 | 0.335 / 0.422 | 0.160 / 0.120 | 2.00 / 1.30 | 3 / 2 | 13.6 / 12.4 |
| sp7 | 0.300 / 0.306 | 0.179 / 0.184 | 0.327 / 0.274 | 0.147 / 0.130 | 0.274 / 0.331 | 0.169 / 0.120 | 1.20 / 2.20 | 2 / 4 | 24.7 / 13.1 |
| sp8 | 0.284 / 0.357 | 0.203 / 0.200 | 0.335 / 0.168 | 0.105 / 0.173 | 0.192 / 0.392 | 0.144 / 0.052 | 1.00 / 1.50 | 4 / 3 | 18.4 / 13.7 |
| sp9 | 0.300 / 0.285 | 0.215 / 0.155 | 0.244 / 0.337 | 0.209 / 0.118 | 0.357 / 0.234 | 0.166 / 0.132 | 1.80 / 1.50 | 3 / 4 | 13.3 / 13.1 |
| sp10 | 0.328 / 0.268 | 0.160 / 0.102 | 0.190 / 0.395 | 0.217 / 0.084 | 0.331 / 0.198 | 0.168 / 0.113 | 2.50 / 2.20 | 3 / 4 | 11.9 / 13.7 |
| sp11 | 0.317 / 0.296 | 0.159 / 0.132 | 0.165 / 0.302 | 0.268 / 0.181 | 0.441 / 0.358 | 0.135 / 0.157 | 1.80 / 1.50 | 4 / 2 | 12.1 / 13.6 |
| sp12 | 0.309 / 0.332 | 0.121 / 0.274 | 0.251 / 0.222 | 0.108 / 0.214 | 0.334 / 0.373 | 0.082 / 0.172 | 2.00 / 1.70 | 5 / 2 | 17.1 / 14.8 |

**The ship rule.** Not met. The criteria met per run fell from 3.33 to 3.04 (p 0.31), not up at
p < 0.05. No criterion's median left its band (land, the largest landmass and the ice stay in it;
the crust under the sea, land above 2 km, the high coast and the collisions were out of it on the
base and stay out), and none moved at p under 0.24. The yield takes the guard's tail away (the
largest excursion 30.0 to 15.5 km, the runs over 20 km 4 to 0, the largest one-step fall 9.45 to
7.42 km) without moving the planet's V1 shape: the peaks were a few cells, and the land above 2 km
and the high coast, which are plateaus and ranges, did not move measurably (p 0.50 and 0.37). So the
switch stayed off by default; whether a correction of what the plate
holds should go on without a V1 gain was the coordinator's call (below). It went on with the
flexure-faults lane on 2026-10-23, under the decisions log's rule of that day (physics goes on
once built and correct; the V1 criteria are no gate).

### The trace with the yield

Two runs ran again with the yield and the trace, scattered 4 (the base's worst) and scattered 1
(the yield arm's worst) (arm `pp-yield380-trace`, argkey a2244c59, `+DEEPTIME_GAP_BUDGET=1
+DEEPTIME_PEAK_TRACE=10000`; their V1 census and guard equal `pp-yield380`'s same runs). Lines above
10 km, base / yield:

| run | continental lines | highest continental z, km | largest `F`, km | oceanic lines (polar) | highest oceanic z, km |
|---|---|---|---|---|---|
| scattered 1 | 21 / 0 | 15.8 / none | 15.3 / none | 3 (1) / 17 (9) | 10.9 / 14.7 |
| scattered 4 | 133 / 2 | 30.0 / 11.0 | 46.9 / 0.04 | 2 (0) / 2 (0) | 12.3 / 11.3 |

The loop the trace found is gone: no continental column stands on a held `F`. The two continental
lines left are collision columns (63 and 68 km of crust, Airy heights 4.2 and 4.7 km) at 11.0 and
10.5 km, 5.8 to 6.8 km over the settling's target; one was resampled in the step, and the other's
`F` is 39 m, its gap's largest parts the carry's (6.5 km) and the erosion's (5.7 km). What is left
above 10 km is the second, oceanic kind (ridge samples, Airy height −1.8 to −3.7 km, `F` −0.7 to
0.4 km), whose gap is the carry's part (9.3 to 23.5 km). In scattered 1 it reached 14.7 km at 78°S
in the last step, and it is the likeliest source of that run's 15.5 km guard's worst, the arm's
largest (the trace reads plate samples and the guard L1 cells, so this is not shown).

### What is left, and for whom

- **The oceanic carry peaks** (above, and "Are the poles special?"): ridge and basin samples raised
  10 to 16 km by the surface's carry, more often on the polar rows. They are now the guard's
  largest excursions with the yield on. The surface lane's (the carry in `surface/advect.rs` and the
  gap budget's carry part); not traced to a rule here. *Done 2026-10-23, the flexure-faults lane*:
  an integrator between the offset the surface holds over a gap and the plate model's feedback
  (ISO-078), fixed by the held offsets (ISO-125, on by default; [flexure.md](flexure.md), "The held
  offsets"): the guard's worst 13.8 to 10.2 km at the median over 8 runs (p < 0.001).
- **The surface flexure's block metric** ([flexure.md](flexure.md), "Simplifications and limits"):
  the solve treats every row as the equator's, about 1 / cos⁴ φ too stiff north to south on the
  polar rows. A numerical fault; correcting it moves master's output and needs its own arm. The
  flexure lane's. *2026-10-23*: a solve on the sphere's metric exists (ISO-127) and is right for
  a single solve, but its step diverges at the poles on the pool, so it is off: its preconditioner does not
  converge where the rigidity changes along a row near a pole ([flexure.md](flexure.md), "Why it
  diverges"; the recommended cure is a line solve along those rows).
- **The collapse's and the founder's room** judge room by thickness, not height. With the yield on,
  the columns no longer stand high, so it was left; a height-aware room would need a law and a
  source of its own. The plates' lane, only if a later trace shows columns standing over their Airy
  height again.
- **The yield's default.** Off: it does not meet the ship rule. It is the correction of a fault (a
  plate holding up to 47 km of rock away from local isostasy) and moves no V1 criterion, so the
  coordinator may take it on as such. *On by default since 2026-10-23.*
- **The bound's value.** Only 380 mGal (the largest anomaly on Earth) was run; 300 mGal (Hawaii's
  largest) or a bound that depends on the plate (`Te`, age) were not.

## Plate boundary organisation (2026-10-23, V1 gap 2)

The fourth of the V1 gaps Ben put inside V1 (decisions log 2026-10-22, answer 1: "All four inside
V1 (Recommended)"; [physics-gap-audit.md](physics-gap-audit.md), "Gap 2"). The gap: spreading
boundaries that do not break into ridges and transforms, trenches that may not face their
convergence, rifts and subduction starts that ignore the stress the force balance computes
(FRC-022), and no law for reorganisation. Ben's view of it (decisions log 2026-10-21): "mountains
seem very prone to forming directly bordering oceans, I don't believe this is realistic." Ledger
PLT-500 to PLT-507 (new) and PLT-270 (now `verified`). Code: `plates/ridges.rs` (the ridge segments), `plates/rifting.rs` (where
rifts and subduction start), `plates/forces.rs` (the stress handed on), `params.rs` (the
switches); the pool's measure `boundary` (`tools/review/pool/measure_boundary.py`, registry
`tools/review/pool/measures_boundary.json`).

### The boundary before (pool arm base, 24 runs)

The measure `boundary` reads a run's plate check (`plate.json`), its dynamics record
(`rec.json`), its globals and its summary: no new sim. On the pool's base (master bf8477b2, key
7f9d8f115049, argkey 2a53566b: `earthlike_quick_16k`, 1000 Myr, both starts, seeds 1 to 12,
`+DEEPTIME_CRUST_BUDGET=1`), medians of the 24 runs with their quartiles; the shares are of the
boundary's length over the steps after the first 100 Myr (PLT-500):

| measure | base: median (quartiles) | Earth |
|---|---|---|
| subducting | 0.342 (0.337 to 0.347) | SUB 0.197 (Bird 2003, Table 3) |
| spreading (ridge and rift) | 0.360 (0.357 to 0.364) | OSR + CRB 0.363 |
| transform | 0.212 (0.207 to 0.215) | OTF + CTF 0.283 |
| in collision | 0.066 (0.064 to 0.071) | CCB 0.088 |
| converging with no slab | 0.019 (0.017 to 0.024) | OCB 0.067 |
| ocean-ocean share of the subducting boundary | 0.302 (0.296 to 0.312) | about 0.35 (PLT-370) |
| subducting / all boundary, lattice km | 151,400 / 443,700 | 51,310 / 260,487 geodesic |
| trench convergence across over full, capped (PLT-312's measurement): all / continental | 0.798 / 0.805 | SUB 0.854 |
| mean convergence across the oceanic front | 53.1 mm/yr (51.9 to 55.3) | SUB 53.2 mm/yr |
| oceanic front under 10 / at 80 or more mm/yr across | 0.042 / 0.190 | |
| active share of the continental margin | 0.518 (0.505 to 0.541) | 0.25 to 0.40 (CHK) |
| coast at a subduction front / in a collision belt | 0.312 / 0.208 | |
| coast at a front where the continent overrides / at a front an initiation made | 0.227 / 0.173 | |
| passive coast | 0.460 | |
| front in collision | 25,800 km (25,100 to 29,200) | 6,000 on average, 12,000 today (PLT-401) |
| collision length p50 / p90 | 41 / 106 Myr | 15 to 50 / at most 120 (CHK) |
| collisions | 1.75 per 100 Myr | 2 to 6 (CHK) |
| plates alive | 23 | 52, 19 over 0.0377 sr (PLT-372) |
| rifts / initiations / polarity reversals | 2.4 / 6.5 / 16.9 per 100 Myr | 13 initiations in 104 Myr, 4 of them reversals (Crameri et al. 2020, Table 1) |

The lattice's boundary is about 1.84 times a geodesic one (PLT-213), so in Earth's terms the
model has about 82,300 km of subducting boundary against Bird's 51,310, 86,800 km of spreading
against 94,810 and 51,100 km of transform against 73,915; the total, about 241,000 km, is close
to Bird's 260,487.

(Corrected 2026-10-24, the front lane, PLT-530: the 1.84 was the model's lattice boundary over
Bird's, so it assumed the totals equal. One spacing per boundary sample counts a straight
boundary 1.04 times (`shortening::crossing_sum` over great and small circles, the test
`the_straight_shares_add_up_to_the_boundarys_length`). In Earth's terms the model has about
145,200 km of subducting boundary against Bird's 51,310 (2.8 times), 153,100 km of spreading
against 94,810 (1.6 times) and 90,200 km of transform against 73,915 (1.2 times); the total,
about 425,400 km, is 1.63 times Bird's with 23 plates against 52. The shares below stand; the
lengths are all long, and the subducting boundary most. Why: "The front's length: why the plates
are ragged (2026-10-24)" below.) What the table says:

- **The trenches already face their convergence.** The convergence across the oceanic front is
  Bird's to the millimetre a year (53.1 against 53.2), and the capped ratio is 0.80 (the cap at
  v_0 lifts it, so the plain ratio is lower still) against Bird's 0.854. The audit's "orient
  trenches" is answered by this measurement: no law is needed for it (PLT-501).
- **The excess of subduction is not obliquity but a shortage of transform**: the model has 1.6
  times Earth's trench and 0.7 times its transform, and its spreading boundary is Earth's. (By
  share, 1.74 times Earth's subducting share and 0.75 times its transform share; by length, on
  the corrected 1.04, 2.8 and 1.2 times.)
- **More than half of the coast at a subduction front is at a front that a subduction
  initiation made** (0.173 of the coast, of 0.312), and most of that has the continent
  overriding. `rifting::initiate` splits the plate's old floor off along a Voronoi line, keeping
  continental crust on the parent (PLT-067): where the old floor lies against a continent, which
  on an Atlantic-type ocean it does, the line runs along the continent's edge and the old floor
  goes down under it. That is a passive margin turned into an active one, which Earth does not
  do: "There are no Cenozoic examples of a passive continental margin transforming into a
  convergent margin, in spite of the fact that seafloor adjacent to SE Africa, NW Africa and
  eastern N. America is about 170 Ma old" (Stern 2004, §4.1); of Crameri et al.'s 13 initiations
  since 104 Ma none lies along a passive margin and two near one, and "heterogeneous oceanic
  plate interiors are more favourable for SZI than ocean-continent boundaries, or more
  specifically, passive margins" (Crameri et al. 2020, conclusions). This is Ben's "mountains ...
  directly bordering oceans": the active margin is 0.52 of the coast against Earth's 0.25 to
  0.40.
- **The front in collision is 2 to 4 times Earth's**, as before (PLT-401), while the collisions'
  lengths are within their targets.

### Earth's ridges and transforms, from PB2002's steps

Bird's `PB2002_steps.dat` (fetched with the paper: 5,819 digitisation steps; read with a lane
script, `build/gap2/tools/bird_segments.py`, not kept), as runs of consecutive steps of one class
along one boundary:

| | runs | median | mean | p90 | length-weighted median |
|---|---|---|---|---|---|
| OSR (spreading ridge) | 596 | 75 km | 113 km | 250 km | 165 km |
| OTF (oceanic transform) | 601 | 36 km | 80 km | 200 km | 165 km |
| OSR between two OTF | 462 | 70 km | 105 km | 228 km | 152 km |

The ridge's obliquity (the angle between the relative velocity and the step's normal) is 5° at
the length-weighted median and 11.5° on average, with 82 % of the ridge within 20° of
orthogonal; the subduction zones' is 21° at the median, 49 % within 20°. Transform length is
0.71 of the ridge's. Gerya 2012 (Tectonophysics 522-523, §1): "The perpendicularity of the
ridges and the transform faults is so common that it is considered to be an 'intrinsic property'
of the oceanic spreading process (Oldenburg and Brune, 1972). Tens to hundreds of kilometer long
active transform fault sections connecting two spreading centers are typically extended from
both sides by even longer (up to thousands of kilometers) inactive fracture zones of similar
orientation." The quick lattice's spacing is about 71 km and the boundary's normal is judged
over two rings (PLT-310), so Earth's 75 to 165 km segments are under its resolution; their
geometry is not. A ridge segment normal to the motion and a transform along it share a fixed
ratio for a given envelope obliquity α, transform over ridge = tan α, whatever the segments'
length: Earth's 0.71 is an envelope about 35° from orthogonal.

### The design

Each stage is its own switch. They were built off by default, and off was the base bit for bit:
the pool's alias check at 839dc4d2 (key 21aa3ea4d466 against base's 7f9d8f115049, seed 1,
supercontinent start, 1000 Myr: the world state's 106 files and the dynamics record identical;
alias recorded), and again on the new base at 0fe14663 (key 15660e3c5636 against base2's
4eeee4b84a8c: the world state's 108 files and the dynamics record identical). Under Ben's rule
of 2026-10-23 (CLAUDE.md "How many runs") a stage goes on by default once it is built and
correct, its unit tests against its sources passing and its pool arm showing no divergence or
guard failure, and the rule it replaces goes off with it; the V1 numbers are realism checks, not
a gate. Every stage passed and is on by default since 2026-10-23 ("The arms", below). So each
stage below names the rule it replaces, and none of its values is chosen to move a V1 number or
Bird's shares: each comes from a source or from the lattice's resolution.

**A. Ridge segments normal to the motion, joined by transforms** (`tectonics.ridge_segment_km`,
PLT-502). The ridge is made where the crust is: a resampling turns each gap point (no footprint
covers it) into new floor of the nearer plate, so the ridge is the gap's midline and keeps the
shape of the rift that opened it. With the switch on, the gap points between two plates a and b
are put in the frame of the pair's relative rotation, ê = (ω_a − ω_b) / |ω_a − ω_b|:
colatitude θ about ê and longitude φ around it. A transform follows a small circle about the
relative pole (constant θ), a ridge normal to the motion a meridian (constant φ). The gap is cut
into bands of θ `ridge_segment_km` wide (shifted by a per-pair offset, so pairs do not share band
edges); in each band the ridge stands at the circular mean of the gap's φ there, and a gap point
on a's side of it (the +φ side: a moves that way from b) goes to a, the rest to b. Its age and
height follow its distance from that segment's ridge, as Cortial's α follows the distance from
the midline. Each band's ridge is a meridian piece, and the steps between bands are small-circle
pieces: transforms by construction. A band whose two sides lie the other way round (a on the −φ
side, where the rotation makes the pair converge) keeps the midline. A gap is the spreading since
the last resampling (up to `resample_drift_km`, 1,500 km), so a band straightens in one
resampling where its obliquity's offset across the band (L tan α) is under the gap's width, and
over several where it is wider; the ridge's older pieces stay in the plates as fracture zones.
The length to run is Earth's first-order segment, 600 km ("First-order segments extend for 600 ±
300 km and persist for >5 × 10^6 years. Second-order segments extend for 140 ± 90 km", Aghaei et
al. 2014 after Macdonald 1998): about eight spacings of the quick lattice, where the second-order
segments and PB2002's runs (above) are under the two rings over which the boundary's normal is
judged. The ratio of transform to ridge does not depend on the length. *Replaces*, in the bands
it draws: which plate a gap point goes to (the nearer one) and its α from the midline (PLT-100,
PLT-101, Cortial's). The edges' blend z̄ stays (it is the position's interpolation between the
two plates' edges, the same whichever plate takes the point), and a band that converges or is
too thin to tell keeps the midline, as does a point whose band's meridian lies beyond the other
plate's edge: where a gap is narrower than its obliquity's offset across a band, the band's
segment ends inside the band and does not cross the gap at its ends, so those rows keep the
midline rather than going whole to one plate with no ridge in them. *Test against the source*:
`ridge_segments_stand_normal_to_the_motion_with_transforms_along_it` (`plates/ridges.rs`): on a
gap 35° oblique, each band's ridge is one meridian of the relative
pole, the steps run along the motion, and transform over ridge is tan 35° within 0.05; and
`a_segment_ends_where_its_meridian_leaves_the_gap`: on a gap 2° wide in 4° bands (an offset of
2.8° across each), no point is handed to a side with its band's meridian beyond the other plate's
edge, and the rows that take the segment hold both plates.

**B. Trenches that face their convergence**: measured above, and not built. The model's trenches
take Earth's convergence across them (PLT-501).

**C1. Subduction starts off the passive margin** (`tectonics.subduction_initiation_offshore`,
PLT-503). An initiation's fracture is kept off the parent's continental crust by the reach of the
arc the new trench builds on the parent, `arc_distance_km` + `arc_width_km` (200 + 60 km), and
one lattice spacing for where on the lattice the front falls (a great-circle distance, measured
over the parent's visible samples): the old floor within it stays on the parent whatever the
Voronoi split says, and the seed old sample is drawn from the old floor beyond it (no initiation
when there is none). The old floor then goes down under the parent's oceanic margin, at an
intra-oceanic trench, not under the continent; its arc stands on ocean floor (an island arc),
and the floor between the trench and the coast stays passive until the ocean closes and the arc
meets the continent (an arc-continent collision, as on Earth). The rate (PLT-063) and the
qualifying floor (PLT-062, PLT-064) are unchanged, so C1 moves where subduction starts, not how
often. Sources: Stern 2004 and Crameri et al. 2020, above. There is no distance of its own to
choose: it is the arc's (2026-10-23: a `subduction_initiation_margin_km` knob before, 500 km
proposed; replaced by the derived reach so that no knob is left). *Replaces*: PLT-067's split
along the continent's edge, which keeps the continent on the parent but sends the old floor
beside it down under the coast, the passive margin turned active that PLT-067's own text says
it prevents; and the seed's draw over all the old floor. *Test against the source*:
`subduction_starts_off_the_passive_margin` (`plates/rifting.rs`): a 60° continent in old floor;
off, some of eight draws send the floor beside the coast down; on, none of the sinking plate
lies within the reach, and an arc reaching across the ocean starts nothing.

**C2. Rifts and subduction starts from the stress and from weakness** (PLT-504 to PLT-506).
FRC-022's mean in-plane stress σ (MPa, tension positive) is each plate's answer at every step; it
is handed on from the force balance to the rifting rules (no state is kept, so off changes
nothing).

- *Rifts under tension* (`tectonics.rift_stress_mpa`, σ_s, 0 = off): a plate's rift hazard
  (PLT-083) is weighted by w = e^(σ/σ_s) / ⟨e^(σ/σ_s)⟩, the mean over the plates that may rift
  weighted by the hazard each has unweighted (its λ: area, continental fraction and plume
  boost), so the expected number of rifts is the base's (to first order in λ) and only where
  they happen moves: to the plates the edge forces pull apart. On base's records the plates' σ is
  18 MPa at the median, its quartiles about 0 and 43 MPa, continental plates −3 to 60 MPa
  between p10 and p90; σ_s = 20 MPa is about half the quartiles' spread (game-choice: set from
  the model's own stresses, not fit to any outcome). *Replaces*: the hazard's share between
  plates by their area, continental fraction and plume heads alone (PLT-083, PLT-084, PLT-090,
  which stay as the hazard's size). *Test*: `stress_weights_keep_the_mean_and_favour_the_stress_asked_for`.
- *Subduction starts under compression* (`tectonics.initiation_stress_mpa`, σ_s, 0 = off): the
  same weight with −σ on the initiation hazard (PLT-063). "Horizontally forced SZI is clearly
  more common than vertically forced SZI ... in the last 100 Ma" (Crameri et al. 2020), and
  collisions, which load their plates in compression: "collision events along pre-existing
  subduction trenches are often precursors of SZI events" (ibid.). *Replaces*: the share of
  PLT-063's hazard between plates by their old floor alone.
- *Rifts along weak lithosphere* (`tectonics.rift_weakness`, an exponent m, 0 = off; it reads
  `tectonics.thermal_state`, on by default since gap 4's merge of 2026-10-23, and with the state
  off the split is the great circle's whatever m is): the rift's fragments are grown from their centres through the
  plate (Dijkstra over the lattice) with each edge's cost its length times (L_ref / L)^m, L the
  lithosphere's depth (gap 4's state, `Sample::lithosphere_km`; unset counts as L_ref, clamped
  to 20 to 225 km) and L_ref the old craton's 225 km (PLT-462). Thin, hot lithosphere is slow
  to cross, so the fragments meet in it: the fracture runs through arcs, back-arcs, plume heads
  and young floor and around cratons. The exponent to run is m = 2: a brittle strength growing
  linearly with depth (Byerlee's friction, from memory) integrates to a plate strength ∝ L², and
  an edge costs its strength's inverse. The fractal warp stays on top (PLT-087). A sample no
  fragment reaches through the plate (a part cut off on the lattice) goes by the great circle to
  every centre. *Replaces*: the
  fragments' great-circle Voronoi split (Cortial's, PLT-087's distance; its warp stays). *Test
  against the source*: `rifts_break_where_the_lithosphere_is_thin` (`plates/rifting.rs`): a
  225 km plate with a 30 km belt off the centres' bisector; at m = 0 under 0.3 of the fracture
  lies in the belt, at m = 2 over 0.8.

**D. Reorganisation** (PLT-507). A plate reorganisation is a change of the plates' motion and of
the boundaries that follow it. The model's motion already relaxes towards the force balance's
answer over 6 Myr (PLT-191), and the balance answers every boundary change (a slab that breaks
off, a collision that stalls, a new trench). The design adds the boundaries' half: with A on,
the ridge's segments are drawn again at every resampling in the frame of the pair's present
relative rotation, so a change of spreading direction re-segments the ridge and its old segments
stay behind as fracture zones. Gerya 2012 (§4): transform faults' development "occurs on the
timescale of plate separation and thus should react nearly instantaneously to changes in
spreading direction (Menard and Atwater, 1968; Merkur'ev et al., 2009)". With C2 on, a change of
the boundary forces moves where the next rifts and subduction starts come: a collision that
stalls puts its plates in compression and raises their initiation weight, and a slab's pull puts
its plate in tension and raises its rift weight. That is the law: no separate trigger; the
plates' forces and their relative motion redraw the boundaries.

### Collision belts on the coast (2026-10-23)

Gap 4's measure `coastset` (`tools/review/pool/measure_coastset.py`) on base's 24 runs puts most
of the coast with high ground in collision belts: 0.333 of the coast has high ground (v1census's
`coast_hi`, which fires above 0.11, docs/science/realism-checks.md), and of it 0.690 is within
150 km of a collision belt, 0.190 at an arc and 0.085 on a rifted margin. The collision belts
touch 0.360 of the coast, and 0.642 of that has high ground (arcs 0.246, rifted margins 0.094).
Without the collision belts' high coasts `coast_hi` would be 0.094. The coordinator's question
(2026-10-23): why does a collision front end at an open sea instead of inside a continent? The
measure `collcoast` (PLT-508, registry `measures_boundary.json`) answers it from the same cached
runs (medians of 24; the hand-offs and partings over the run after its first 100 Myr):

| measure | base |
|---|---|
| collision regime (a Himalayan orogeny under 30 Myr old): share of the land / of it within 150 km of the sea | 0.170 / 0.427 |
| its land's median distance to the sea | 175 km |
| hand-offs per 100 Myr, terranes under 50 / 50 to 200 / 200 to 1000 / 1000 or more samples (about 5,100 km2 each) | 65.1 / 17.4 / 9.3 / 1.8 |
| the plate check's collisions (a terrane of 0.1 of a starting plate or more, RLC-070) | 1.8 per 100 Myr |
| share of the crust crossing collision fronts, in those four sizes | 0.111 / 0.104 / 0.304 / 0.061 |
| partings (states that end with no hand-off) per 100 Myr / their share of the crust crossed | 37.6 / 0.43 |
| partings: no continental crust left at the front (`parted`) / polarity reversed / contact closed | 0.56 / 0.27 / 0.13 |
| collision high coasts: the belt along the coast / ending at the coast / not elongated | 0.315 / 0.216 / 0.465 |
| collision high coasts on landmasses under 0.02 / of 0.10 or more of the land | 0.240 / 0.618 |

What it says:

- **Few of the coast's belts are continent-continent fronts that run out to sea.** A belt that
  ends at the coast, the case the question names, is 0.22 of the collision high coasts. Almost
  half are belts not elongated at 400 km, the footprint of a small terrane's belt, and a third run
  along the coast.
- **They are the dockings at active margins.** The model hands off about 93 collision states per
  100 Myr, 65 of them terranes under 50 samples (about 255,000 km2), and ends another 38 with no
  hand-off, most when the terrane's continental crust at the front is used up (stacked into the
  belt). Earth's arc-continent sutures with ophiolites are 15 to 46 per 100 Myr (Macdonald et al.
  2019; RLC-073, whose caveat holds: they match the model's terrane accretions, not its counted
  collisions). The model's hand-offs of 50 samples or more, 28 per 100 Myr, sit inside that
  range; with the smaller ones and the partings the model has about 130 docking episodes per
  100 Myr, three to nine times Earth's sutures (a loose comparison: one suture can hold several of
  the model's episodes). `belts.rs` stacks every collision state's shortened crust into a belt on
  the overriding plate, whatever the terrane's size, and at an active margin the overriding plate's
  edge is the coast: the terrane docks, the belt rises at the coast, and the ocean behind the
  terrane stays open. That is how Earth's accreted coasts look too (the North American
  Cordillera's), so the kind is right; the amount comes from how many terranes and arcs reach the
  trenches.
- **Where that number comes from**, and which of this lane's stages touch it:
  - the subducting boundary, 1.6 times Earth's (above), with every trench a place for a docking.
    Stage A adds transform at the ridges but does not shorten the trenches: indirect.
  - the subduction initiations at passive margins (0.173 of the coast). Stage C1 moves them
    offshore, which makes intra-oceanic arcs that later dock against the continent: Earth's way
    for an arc to end, and possibly more collision coast, not less. Its arm will say which.
  - the polarity reversals, 16.9 per 100 Myr against Crameri et al.'s 4 in 104 Myr (PLT-371):
    each leaves a belt beside a new trench. The subduction lane's, not this one's.
  - the arcs' crust (PLT-045), the stuff of the small terranes: not this lane's.
  - stage C2 moves where rifts and initiations come, not how many: indirect.
- **The belts' spread** (how far a belt's thickened crust reaches inland, which gap 4 left to this
  lane) is not a stage here. The belt's width and its plateau would need a law from the belt's
  strength against its driving force (the thin viscous sheet's Argand number, England and
  McKenzie 1982, from memory), which none of the stages builds; recommended as its own lane after
  the arms, not a knob on `belts.rs`'s widths. `collision_orogeny_myr` is only the regime's label
  window (it decides which belts the measures call collision) and is not a lever.

### The arms (2026-10-23, on the new base)

Master 2fde68f7 merged in (the flexure and gap 4 lanes with their physics on; its base is the
pool's base2, key 4eeee4b84a8c, argkey 2a53566b, 24 runs) and the alias check repeated on it
(above). One cumulative 8-run arm per stage, `earthlike_quick_16k`, 1000 Myr, seeds 1 to 4, both
starts, `+DEEPTIME_CRUST_BUDGET=1`, key 15660e3c5636, read against g2-base2s4 (base2's own runs of
the same seeds and starts, cached, no sim):

| arm | adds | argkey |
|---|---|---|
| g2-A | `ridge_segment_km=600` | af86bbd6 |
| g2-AC1 | + `subduction_initiation_offshore=true` | a14b0ebd |
| g2-AC1C2 | + `rift_stress_mpa=20`, `initiation_stress_mpa=20`, `rift_weakness=2` | ac2787f9 |
| g2-AC1C2R | + `reversal_by_arc_crust=true` (PLT-371, the coordinator's request) | 1ae5c62e |

Medians of 8 runs, the two-sided Mann-Whitney p against the control in brackets (`pool.sh table
--base g2-base2s4 --registry tools/review/pool/measures_boundary.json --registry
tools/review/pool/measures_gap4.json`, measures `v1census`, `boundary`, `collcoast`, `coastset`
and `guard`); the docking episodes are the sum of the four hand-off medians and the partings':

| measure | control | g2-A | g2-AC1 | g2-AC1C2 | g2-AC1C2R |
|---|---|---|---|---|---|
| runs that failed | 0 | 0 | 0 | 0 | 0 |
| guard: steps beyond 2 Earth reliefs | 0 | 0 | 0 | 0 | 0 |
| guard's worst, m from the sea | 12,172 | 13,144 (0.046) | 12,341 (0.13) | 11,810 (0.72) | 12,061 (1.0) |
| plate cells' largest excess over Airy, m | 2,971 | 3,407 (0.083) | 3,256 (0.44) | 3,398 (0.015) | 3,481 (0.44) |
| land | 0.315 | 0.333 (0.44) | 0.311 (1.0) | 0.331 (0.23) | 0.319 (0.72) |
| largest landmass | 0.173 | 0.147 (0.80) | 0.220 (0.16) | 0.243 (0.13) | 0.235 (0.16) |
| land above 2 km (fires above 0.07) | 0.126 | 0.110 (0.80) | 0.116 (0.80) | 0.110 (0.57) | 0.146 (0.57) |
| coast with high ground (fires above 0.11) | 0.310 | 0.254 (0.57) | 0.311 (1.0) | 0.292 (0.65) | 0.330 (0.57) |
| collisions per 100 Myr | 1.65 | 1.95 (0.29) | 1.70 (0.75) | 1.60 (0.96) | 1.95 (0.057) |
| continental crust's net change over the run | −0.020 | +0.019 (0.10) | −0.006 (0.38) | +0.025 (0.021) | −0.005 (0.28) |
| v1census criteria met per run, mean of 7 | 5.00 | 5.12 | 5.12 | 4.75 | 5.00 |
| subducting / transform / spreading (Bird 0.197 / 0.283 / 0.363) | 0.340 / 0.213 / 0.358 | 0.334 / 0.212 / 0.364 | 0.340 / 0.211 / 0.358 | 0.339 / 0.211 / 0.359 | 0.338 / 0.215 / 0.356 |
| ocean-ocean share of subducting (about 0.35) | 0.300 | 0.319 (0.007) | 0.339 (< 0.001) | 0.345 (< 0.001) | 0.347 (< 0.001) |
| active share of the continental margin (0.25 to 0.40) | 0.532 | 0.540 (0.72) | 0.527 (0.44) | 0.516 (0.28) | 0.527 (0.88) |
| coast at a front: the continent overriding / an initiation's | 0.226 / 0.167 | 0.220 / 0.149 | 0.202 / 0.156 | 0.203 / 0.131 | 0.208 / 0.167 |
| front in collision, km (6,000 to 12,000) | 28,616 | 30,688 (0.33) | 32,886 (0.038) | 29,658 (0.38) | 33,751 (0.010) |
| collision length p50 / p90, Myr | 32 / 102 | 46 / 138 | 38 / 136 | 46 / 135 | 52 / 125 |
| rifts / initiations per 100 Myr | 2.45 / 6.50 | 2.85 / 6.70 | 2.10 / 8.00 | 2.45 / 7.90 | 2.55 / 7.20 |
| polarity reversals per 100 Myr (Crameri: 4 in 104 Myr) | 16.3 | 19.3 (0.040) | 17.6 (0.13) | 15.8 (0.80) | 12.1 (0.005) |
| hand-offs per 100 Myr, under 50 / 50 to 200 / 200 to 1000 / 1000 or more samples | 63.8 / 15.8 / 8.1 / 1.6 | 70.3 / 18.6 / 9.2 / 2.0 | 64.5 / 20.3 / 10.0 / 1.8 | 68.0 / 19.0 / 9.2 / 1.6 | 69.7 / 20.7 / 10.9 / 1.9 |
| partings per 100 Myr | 34.4 | 38.9 | 35.9 | 33.0 | 32.8 |
| docking episodes per 100 Myr (Macdonald et al. 2019: 15 to 46 sutures) | about 124 | about 139 | about 133 | about 131 | about 136 |
| collision belts' share of the high coasts (`coastset`) | 0.767 | 0.786 | 0.823 | 0.812 | 0.803 |

Read against base2's 24 runs instead (`--base base2`), the realism checks are the same: land
above 2 km 0.134 on base2, p 0.29 to 0.56 for the arms; coast with high ground 0.312, p 0.22 to
0.62. What the tables say:

- **Every stage is correct by Ben's rule, so every stage is on.** All 32 runs ran their 1000 Myr,
  none had a step beyond two Earth reliefs, and the guard's worst stays within 11,810 to 13,144 m
  of the control's 12,172 (A's p 0.046, the others 0.13 or more). Since 2026-10-23 the defaults are
  `ridge_segment_km` 600, `subduction_initiation_offshore` true, `rift_stress_mpa` and
  `initiation_stress_mpa` 20, `rift_weakness` 2 and `reversal_by_arc_crust` true: the default
  planet is g2-AC1C2R's. The rules they replace run only with them off: PLT-100/101's nearer plate
  and midline α in the ridge's bands, PLT-067's split along the continent's edge, the hazards'
  share between plates by area, continental fraction, plume heads and old floor alone (PLT-083,
  PLT-084, PLT-090, PLT-063, which stay as the hazards' sizes), PLT-087's great-circle distance
  (its warp stays) and PLT-056's judgement of the overriding side by its crust in the overlap.
  PLT-064's clearance stays.
- **The two realism checks that fire on the base fire on every arm, and no stage moves them**
  (p 0.57 or more on both): land above 2 km 0.110 to 0.146 (fires above 0.07), coast with high
  ground 0.254 to 0.330 (fires above 0.11). Nothing was tuned to them.
- **The docking count stays about 130 per 100 Myr** (124 on the control, 131 to 139 on the arms),
  three to nine times Macdonald et al.'s sutures, and the collision belts keep 0.79 to 0.82 of
  the high coasts. None of these stages was built to change the number of terranes and arcs
  reaching the trenches, and none does.
- **A does not show in the boundary's classes.** Transform stays 0.211 to 0.215 (Bird 0.283) and
  spreading 0.356 to 0.364. The law is built and its test holds (one meridian per band, transform
  over ridge tan α), but the boundary's classes are read on the lattice over two rings (PLT-310)
  and its transform window (PLT-311) at every step, and either a band's transform step (L tan α)
  is too short to read there or the plates' edges between resamplings do not keep it; which of the
  two is not measured (a measure of the spreading boundary's obliquity from the record would say).
  A raised the reversals to 19.3 (p 0.040) and the ocean-ocean share to 0.319 (p 0.007).
- **C1 makes the intra-oceanic trenches Earth has**: the ocean-ocean share of the subducting
  boundary goes to 0.339 and on to 0.347 with C2 and R (p < 0.001; Stern & Scholl's about 0.35),
  and the coast where the continent overrides falls from 0.220 to 0.202. The active margin barely
  moves (0.540 to 0.527): a coast counts as at a front within 400 km of one
  (`stats::MARGIN_ACTIVE_KM`), and C1's trench starts at the arc's reach, about 330 km off the
  continent, inside that window.
- **C2 keeps the counts and moves the places**: rifts 2.10 to 2.45 and initiations 8.00 to 7.90 per
  100 Myr (the weights keep the expected count). The continental crust's net change over the run
  goes from −0.020 on the control to +0.025 (p 0.021), and the largest excess of a plate cell over
  Airy from 2,971 to 3,398 m (p 0.015), still far from the guard's 2 reliefs.
- **R does what PLT-371's row measured on the older bases**: the reversals fall from 15.8 to 12.1
  per 100 Myr (p 0.005 against the control, Crameri et al.'s 4 in 104 Myr), the share of
  partings ended by a reversal from 0.261 to 0.175, and the jams go on as collisions instead: the
  front in collision 29,700 to 33,800 km (p 0.010 against the control, already 2 to 4 times
  Earth's), collisions 1.60 to 1.95 per 100 Myr (p 0.057), land above 2 km 0.110 to 0.146 and coast
  with high ground 0.292 to 0.330 (neither significant).

The seed-pinned weld tests were swept with every stage on (`plates/tests.rs`, the seed history in
each test's comment): three tests moved to new seeds, and no one-law fixture needed the old rules
pinned off. In none of the 32 arm runs did the plate speed guard bind (`plates/guard_bound_plate_steps`
absent); on the small test planet it bound once, at scattered seed 18.

### What was decided, and what comes next

How plates are made and moved is Ben's scope; he approved doing the gap, not one design. The
stages above stay on the lattice and change no world-data format. Two questions were raised and
settled by the coordinator (2026-10-23), not put to Ben: the plate count and a finer lattice
(PLT-372) stay as they are, and the lattice's boundaries are not replaced by drawn ones, since
each stage's physics works on the lattice as it is. Induced initiation near trenches comes later,
as its own stage after C2's arm: "All SZI events start less than 1500 km away from other active
subduction zones and, in most cases, less than 500 km" (Crameri et al. 2020), while PLT-064
keeps every initiation 1,000 km from all trenches (`pending-rewrite` since the plate geometry
task's arm i1, 2026-10-21); C2's weight is per plate and does not place an initiation, so PLT-064
is not replaced here. A stage that starts subduction at a plate's compressive boundary would
replace PLT-064's clearance. The base's polarity reversals, 16.9 per 100 Myr against Crameri et
al.'s 4 in 104 Myr, are a measurement for the subduction work; stage R (PLT-371, its arm above)
is the reversal judged by the arc's crust, the subduction lane's law, run here on top of C2 at
the coordinator's request, and on by default with the others.

## Docking: what stands at the collision fronts (2026-10-23)

V1 item 1 (the docking lane, `build/coord/briefs/docking-rate.md`): on base3 (key 88c144b6c2cb,
24 runs, `earthlike_quick_16k`, 1000 Myr) the land above 2 km is 0.153 (Earth 0.047 to 0.055) and
the coast with high ground 0.354 (Earth 0.076), and collision belts hold 0.832 of that high coast
(`coastwalk`, PLT-513). Gap 2's reading by count ("Collision belts on the coast" above: most
hand-offs are small terranes at active margins) is right about the count but not about what
stands at the fronts. Read by front-time, length of front times how long it stands, the high coast
is held by a few long continent-continent collisions that never end.

### What was measured

All from cached runs, no new simulation: the measures `sutures` (PLT-512) and `coastwalk`
(PLT-513) in `tools/review/pool/measures_docking.json`, the dock tally (PLT-515) and the confirmed
reversals (PLT-514) in the run summary (both measurement only; HEAD a9fd061e564f, its seed-1 run
bit-identical to base3's, `pool.sh alias`), and four scratch reads of `rec.json` (the
`partings` and `collisions` records and the last step's plates and force balance).

**The count is not the fault.** Base3 hands off 101.3 collision states per 100 Myr, but a state
hands off every 100 km of shortening (PLT-068) and a new one begins on the same contact, so they
make 55.7 sutures per 100 Myr (27.0 of terranes of 50 samples or more) against Macdonald et al.
2019's 19 per 100 Myr over the Phanerozoic and 39 in the last 100 Myr (their table of 104
ophiolite-bearing arc-continent sutures); suture length 39,715 km per 100 Myr against their 23,235
over the Phanerozoic and 50,919 over 0 to 100 Ma. The model's sutures are shorter (median 271 km,
Earth's 1,019) and 1.4 to 2.9 times as many: inside Earth's range by length. (Corrected
2026-10-24, the front lane: the measure divided the collision fronts, straight lengths already
(PLT-222), by 1.84. Version 3 does not: 73,076 km per 100 Myr, 1.4 to 3.1 times Macdonald's, and
the median suture 499 km, half his. So by length too the sutures are more than Earth's, and
shorter; PLT-512.)

**What stands at the fronts in the last 30 Myr** (the collision regime's window, so what makes
the collision high coast; medians of 24, share of the front-time):

| collision states | share |
|---|---|
| handed off (transfers and welds) | 0.224 |
| still open at the end, begun under 10 Myr before it | 0.009 |
| still open, 10 to 30 Myr old | 0.067 |
| still open, 30 to 60 Myr old | 0.148 |
| still open, 60 Myr old or more | 0.482 |
| parted, reversed, closed | 0.006, 0.012, 0.001 |
| open, the incoming plate with 3,000 or more continental samples / 500 to 3,000 | 0.569 / 0.110 |

At the end of a run 16 collision states are open (27,554 km of front, geodesic; 50,700 km by
version 3, the fronts not divided by 1.84), 0.81 of that
front on contacts over 60 Myr old. The 134 open states 30 Myr or older (5.6 a run) have gone on
70 Myr (median), shortened 1,089 km across 3,186 km of front, crossed 3.6 million km2 of crust and
peaked at 2.8 km and 31 mm/yr. Gap 2's boundary measure says the same at the plate scale: 18,950 km
of front in collision against Stern & Scholl 2010's 12,000 km of active suture, and collisions
lasting 48 / 143 Myr (p50 / p90) against the plate check's 15 to 50 and at most 120.

**What the long ones are** (the dock tally, seed 1, after the first 100 Myr; km·Myr of front by
the state's own age, with the means of the front's sides; "near" is continental crust within
300 km of the front on the subducting plate's side):

| state age | share of the front-time | incoming side continental / near | dry incoming crust / dry overriding edge | convergence, mm/yr |
|---|---|---|---|---|
| under 10 Myr | 0.13 | 0.425 / 0.504 | 0.282 / 0.277 | 18.9 |
| 10 to 30 Myr | 0.16 | 0.635 / 0.700 | 0.546 / 0.448 | 13.5 |
| 30 to 60 Myr | 0.23 | 0.722 / 0.788 | 0.688 / 0.645 | 12.8 |
| 60 Myr or more | 0.49 | 0.749 / 0.818 | 0.847 / 0.770 | 12.5 |

Of the 60-Myr-old front-time, the overriding plate's reach (400 km, the belt's width, PLT-241) is
under a quarter ocean for 0.38 and a quarter to a half ocean for 0.43: the belt is built on an
overriding continent about as wide as the belt, so its far side is a coast. The land across a
collision high coast is 512 km (Earth's coasts 600 km, `tools/review/earth-across.py`), and 0.771
of those coasts face a trench or an arc within 300 km.

**What keeps them going** (the force balance of the last step, `steps[-1].forces`, torque in
10^24 N m; medians over 24 runs):

| plates | n | visible samples | continental | own slab front, lattice km | slab pull | collision resistance | its share of the driving torque | speed, mm/yr |
|---|---|---|---|---|---|---|---|---|
| incoming, in a collision 60 Myr or older | 57 | 12,193 | 0.49 | 9,283 | 878 | 642 | 0.425 | 15.7 |
| overriding them | 65 | 4,316 | 0.48 | 1,143 | 137 | 364 | 0.573 | 15.2 |
| all others | 410 | 1,728 | 0.05 | 4,606 | 629 | 13 | 0.012 | 38.4 |

Every incoming plate of a long collision has a slab of its own elsewhere, and they are the
planet's largest plates (0.12 of its surface each, the size of the Indo-Australian plate). The
collision's own slab broke off long ago (its front is three quarters continental, PLT-184's
breakoff after 10 Myr), and the plate's other slabs pull it on: the rigid plate carries their pull
to the jammed continent without limit, the collision resists at its plastic force (Cloos 1993's
buoyancy, FRC-028, and the belt's weight) and converges at 12 to 16 mm/yr, never under the weld
speed (5 mm/yr for 10 Myr, PLT-246), and the state ends only when the crust leaves the front. A
continent of `collision_suture_frac` or more never moves alone (PLT-078): it waits for a stall
that a plate pulled elsewhere never gives.

### What Earth does instead

Stern 2004 (read in full, 2026-10-21): induced subduction initiation follows a collision in two
ways. In **transference** "a buoyant crustal block enters the subduction zone and causes it to
fail, and in the process is sutured to the original hanging wall ... the oceanic lithosphere
outboard of the collision zone ultimately ruptures to become a new subduction zone"; polarity
reversal nucleates the new zone in the overriding plate instead (the model's PLT-056 and PLT-371).
Gurnis et al. 2004 (read in full) give the work: the force resisting a new subduction zone grows
as the plate bends, "reaching a maximum value dependent on plate thickness, but typically ranging
from 2 to 3 × 10^12 N/m for cases that become self-sustaining", "Cumulative convergence, not the
rate of convergence, is the dominant control", and the critical convergence before the zone
sustains itself "only varies between 100 and 130 km" (their Table 3; "100–150 km" in the
abstract; Hall et al. 2003: "approximately 100 km of convergence"). The model's collision pushes
back with more than that per metre of front once it has shortened some 30 km
(`collision_buoyancy_n_m`: 2.25 × 3 × 10^13 N/m × 30/700 for 30 km of crust), so a plate that
keeps converging on a jammed continent carries a force across the continent's own passive margin
that would break the lithosphere there, and the continent would stay with the overriding plate.

India is the counter-example Stern names: 50 Myr after the collision "a new subduction zone
outboard of the colliding blocks has not formed but appears to be in the making", because "It
takes less work to deform large parts of continental Asia than it does to start a new subduction
zone in the Indian Plate". That is the overriding plate deforming across thousands of km, which
the model's 400 km belt (PLT-241) does not do (gap 2's note on the belts' spread, above). In the
model the jammed front takes the whole force, so the cheaper failure is the incoming plate's.

### The stage this lane builds

**T. Transference** (`tectonics.transference`, ledger PLT-516 to PLT-518): a continent in a
collision moves alone to the overriding plate, as a small terrane does, once (1) its contact's
slab has broken off (the subduction has failed: no slab of the pair is left, PLT-184), (2) its
plate has gone on converging on it for Gurnis et al.'s critical convergence of 100 km since then
(the collision's shortening since the jam), and (3) the collision's force on it, over the length
of the continent's margin against its own plate's ocean floor, is at least Gurnis et al.'s
3 × 10^12 N/m (gravity-scaled as the slab pull). The continent's ocean floor stays on its plate;
the force balance then decides what the new boundary does (a trench where the floor converges on
the continent, a rift where it parts), as with every other transfer. A continent whose plate has
no ocean floor at its margin, or whose collision stalls first, still welds its plate whole on the
stall (PLT-246). It replaces the indefinite deferral of PLT-078 for a pushed continent; it does
not replace PLT-064's clearance, which keeps spontaneous initiation on old floor away from
trenches (T's new trench is the collision's own, 0 km from it, the kind of near-trench initiation
Crameri et al. 2020 count).

### Arm dock-T: what transference does (2026-10-23)

Built behind the switch and checked: off, the planet is bit-identical to base3 (key
d06e5e058b69 against 88c144b6c2cb, seed 1, 1000 Myr, `pool.sh alias`: the state's 107 files and
the dynamics record identical). On: arm `dock-T`, 8 runs (seeds 1 to 4, both starts,
`earthlike_quick_16k`, 1000 Myr, 2 threads, the crust budget on), against base3's 24, medians and
the two-sided Mann-Whitney p.

**The continents' collisions end.** 259 transferences in the 8 runs (30 a run): the terrane
5,528 samples at the median (the largest 39,238), after 26 Myr of collision, 457 km of shortening
and at 17.2 mm/yr. 97 continents still weld (12 a run, against base3's 21), on the stall as
before (34 Myr, 1.1 mm/yr).

| measure | base3 | dock-T | p | Earth |
|---|---|---|---|---|
| collisions open at the end: front, geodesic km (version 3, not over 1.84: 50,700 / about 17,300) | 27,554 | 9,399 | | |
| collisions open at the end: how long they have gone on, Myr | 90 | 20 | | |
| last 30 Myr's front-time on partings (open collisions included) | 0.776 | 0.487 | | |
| collision length p50 / p90, Myr | 48 / 143 | 33 / 77 | | 15 to 50 / at most 120 |
| front in collision, km | 34,862 | 16,455 | < 0.001 | 6,000 to 12,000 |
| in collision, share of the boundary | 0.076 | 0.041 | < 0.001 | 0.05 to 0.13 (Bird CCB 0.088) |
| collisions per 100 Myr | 2.10 | 4.25 | < 0.001 | at most 6 |
| sutures per 100 Myr | 55.7 | 54.5 | | 19 to 39 |
| the collision belt's rate of high coast on its coast | 0.604 | 0.254 | | |
| coast with high ground | 0.354 | 0.114 | < 0.001 | 0.076 (at most 0.11) |
| land above 2 km | 0.153 | 0.053 | < 0.001 | 0.047 to 0.055 (at most 0.07) |

The plate check's collision lengths pass in all 8 runs (base3: p50 in 13 of 24, p90 in 7), and
so do its collision count (16 of 24) and the lowest continental share (9 of 24). The guard is
unchanged (its worst 12,669 m from the sea against 12,282, p 0.19; no step beyond 2 Earth
reliefs in either). Against base3's own seeds 1 to 4 (arm `dock-c3`, the same cached runs) every
reading holds: coast with high ground 0.330 to 0.114 and land above 2 km 0.146 to 0.053 (both
p < 0.001), the front in collision 33,751 to 16,455 km, the guard's worst 12,061 to 12,669 m
(p 0.050) with still no step beyond 2 Earth reliefs. The lane's question is answered: the high
coast was the long collisions'.

**The continents grow and drown.** The same runs:

| measure | base3 | dock-T | p | Earth |
|---|---|---|---|---|
| continental crust at the end, share of the planet | 0.404 | 0.571 | | 0.41 (Cogley 1984) |
| the highest continental share of any step | 0.465 | 0.580 | | at most 0.45 (PLT-159) |
| continental crust volume at the end, 10^9 km3 (7.56 at the start) | 7.61 | 11.44 | | |
| sea over the datum, after 100 Myr / at the end, m | 587 / 830 | 1,239 / 1,729 | | |
| continental crust under the sea | 0.259 | 0.780 | < 0.001 | at most 0.56 |
| land | 0.319 | 0.129 | < 0.001 | 0.13 to 0.36 |
| largest landmass | 0.232 | 0.021 | < 0.001 | at least 0.06 |
| V1 criteria met per run: median, mean | 5, 5.04 | 4, 3.75 | < 0.001 | of 7 |
| largest plate, median share / the most | 0.220 / 0.441 | 0.351 / 0.744 | | 0.15 to 0.35 |
| steps with one plate above half the planet, % | 0 | 16.5 | | at most 10 |
| the largest continent's share of the crust, median / time in a supercontinent, % | 0.42 / 36 | 0.69 / 70 | | |

The crust budget (`DEEPTIME_CRUST_BUDGET`, `cb_parts` at the end, medians) says where the area
comes from. Of the continental area (share of the planet over the run), base3's subduction hides
0.983 under overriding plates and the resampling consumes another 0.306; arcs maturing into
continental crust (`finish`, PLT-047) make 0.587, collapse 0.415, founder 0.141, the rate limit
0.107: a closed book (−0.019). Under transference the makers fall to 0.515, 0.146, 0.033 and
0.057, but subduction hides only 0.395 and the resampling 0.192: +0.149. The long collisions
were base3's sink of continental area: a continent held at a front for 60 Myr and more goes
under its overriding plate sample by sample, and its volume goes into the belt (`belts`, +20,030
against `subduction`'s −20,300, 10^6 km3). With them gone, the area made at arcs has nothing
consuming it, the continents cover more of the planet, the ocean basins shrink and the same water
stands about 900 m higher over the datum, and 0.78 of the continental crust is under the sea. The
volume follows: drowned continents erode less (the surface's book −6,423 against −11,500) while
the arcs add as before (+1,829 against +1,718), so the volume grows by half and the mean crust
thickens from 37.0 to 39.6 km. The largest plate grows too: an overriding plate keeps the
continents that dock on it, so one plate holds the assembled continents.

So transference is Stern's mechanism with Gurnis et al.'s thresholds, and it removes the high
coast the long collisions made, but the model's continental area was held at Earth's 0.41 by
those same collisions. What holds it on Earth, against the area the arcs make, is missing: an
area sink of continental crust (the forearc's tectonic erosion, PLT-441, only thins the edge in
the model, never below a normal ocean floor's 7 km: `forearc` −328 of volume and no area;
continental crust never turns oceanic, `c2o` 0 in every process). That is the crust budget's physics (build log
2026-10-19 "mountains eat the world"), not a cap on this one.

**Held off** (the coordinator's call, 2026-10-23). Ben's rule turns physics on once it is built
and correct on the pool. T's own physics is (its unit tests against Gurnis et al. and Stern,
no divergence, no guard failure), but the planet under it is not: the plate check fails in 8
of 8 runs (PLT-159's highest continental share 0.45, the arcs' continental heights, the largest
land gain) and one plate holds over half the planet in 16.5 % of the steps, and a drowned
master would move every GameTest planet and every deployed world for the other lanes. So
`tectonics.transference` stays off by default (off is bit-identical to base3) until it can go
on together with the sink of continental area it exposes; that sink is this lane's next stage.

### What transference exposes: the crust's fluxes against Earth's (2026-10-23)

The coordinator's next question for this lane: under transference the continental crust grows
from 0.42 of the planet to 0.57 and the sea stands 1,729 m over the datum at the end (base3
830 m); which of the model's fluxes are off Earth's, in area as well as in volume, and why does
one plate grow past half the planet? The numbers are the crust budget's
(`DEEPTIME_CRUST_BUDGET`, `crust_budget.rs`) over arm `dock-T`'s 8 runs and `dock-c3`'s (base3's
own seeds 1 to 4, both starts): means of the runs for a flux, medians for a share. Earth's are
the sources' (the bibliography has what was read).

**The course.** The area does not run away; it moves to a new level early and holds there. The
medians of dock-T's runs: continental share 0.422 at the start, 0.503 after 300 Myr, 0.548 after
500, then 0.565 to 0.589 to the end (base3 0.418 to 0.453 throughout). The volume grows steadily,
7.56 to 9.41 to 11.44 (10^9 km3) at the same times, the mean crust from 35.1 to 39.6 km; the sea
over the datum 0, 1,190, 1,730 m. At the end 0.644 of the continental area is 40 to 45 km thick,
its median 1,536 m over the datum and 0.18 of it dry: the continents are thick enough, the sea
stands over them because the ocean basins are 0.43 of the planet instead of 0.58.

**Volume** (km3 a year over the 1000 Myr; Earth: Stern & Scholl 2010 Table 1):

| flux | dock-c3 | dock-T | Earth |
|---|---|---|---|
| added: island arcs | 3.82 | 3.38 | 1.5 |
| added: continental arcs | 1.66 | 1.71 | 1.0 |
| added: at sutures | 0 | 0 | 0.2 |
| added: rifts, hotspots, rifted margins | 0.04 | 0.04 | 0.5 |
| lost: sediment subduction (PLT-440) | 1.19 | 1.00 | 1.1 |
| lost: subduction erosion (PLT-441) | 0.89 | 0.83 | 1.4 |
| of it continental crust | 0.30 | 0.32 | |
| lost: at sutures, all three (PLT-242, the belts' room) | 2.18 | 0.86 | 0.7 |
| lost: foundering | 0.00 | 0.00 | unknown |
| the stock's change, every process | +0.27 | +3.99 | 0, or a loss |

The model's own rows: the ocean floor's crust made continental 0.16 and 0.12, pelagic sediment
1.35 and 1.11, the resampling's drops 0.55 and 0.55, and the surface stage's move off the
continents 11.56 and 6.74. What is off:

- Sediment subduction is Earth's in both arms, and so is the loss at sutures under T (0.86
  against 0.7). Base3's 2.18 was three times Earth's: the long collisions closed its book.
- The arcs add 1.7 to 2.5 times Earth's, and not by their rate: per km of front (the lattice
  front, one spacing a front sample, over 1.12; see "The front's length on the lattice" below)
  island arcs add about 79 km3/km/Myr against Stern & Scholl's 100 at full convergence (Clift et
  al. 2009: 125), continental arcs about 18 against 35; the slow plates' convergence factor g is
  0.57 on the oceanic fronts. The fronts are long: 47,823 and 107,334 lattice km under T, about
  42,700 and 95,800 km, against Earth's 14,500 and 27,500. The plate check's subducting share of
  the boundary is 36.6 % (dock-c3 33.8 %) against Bird 2003's 19.7 % and fails in every run of
  both arms, and the whole boundary is about 1.65 times Bird's: the arcs' surplus is the
  subduction zones' length, gap 2's boundary organisation (`physics-gap-audit.md`), with or
  without T. (Corrected 2026-10-24, PLT-530: over 1.04, not 1.12, the rates are about 74 and 17
  km3/km/Myr, the fronts about 45,900 and 102,900 km, and the boundary 1.63 to 1.77 times Bird's.)
  Base3 hid it under the excess loss at sutures; under T nothing offsets it, so the
  volume grows by about 4 km3 a year and the crust thickens.
- Subduction erosion takes 0.6 of Earth's, for two reasons. Its erosive fronts are 16,888 lattice
  km (about 15,100 km) against Clift & Vannucchi's 22,925, because few slabs come in at the
  60 mm/yr PLT-442 asks for (the plates' area-weighted RMS speed is about 29 mm/yr, the plate
  check's band 40 to 100, failing in both arms). And the band runs out: PLT-441 takes its volume
  from the samples within 200 km of the front, nearest first, each down to an ocean floor's
  7 km, and the front stays where it is in the overriding plate's frame, so a margin that erodes
  for long has nothing left to give (`short`): it takes about 55 km3/km/Myr of its 90.
- Foundering is 0, and delaminating a lower crust takes it from below, so it would make or
  consume no area on Earth either; it is not in the area book.
- The volume grows faster under T because the drowned continents erode less (6.74 against
  11.56 moved off them) while the arcs add as before: it follows the area.

**Area** (continental crust, share of the planet over the run, medians; times 0.51 for km2 a
year: a share over 1000 Myr of the planet's 510 million km2):

| part (`cb_parts`) | base3 | dock-T | dock-T, km2 a year |
|---|---|---|---|
| made: arcs maturing (`finish`, PLT-047) | 0.587 | 0.515 | 0.26 |
| made: collapse | 0.415 | 0.146 | 0.074 |
| made: founder | 0.141 | 0.033 | 0.017 |
| made: rate | 0.107 | 0.057 | 0.029 |
| consumed: hidden at the fronts (`subduction`) | 0.983 | 0.395 | 0.20 |
| consumed: resampling | 0.306 | 0.192 | 0.098 |
| net, every part | -0.019 | +0.149 | +0.076 |

Earth's continental area fluxes, from the sources:

- consumed by shortening at continental convergent boundaries: Bird 2003 Table 3's CCB, an
  area production of 0.013616 m2/s, 0.43 km2 a year, on 23,003 km at 18.7 mm/yr across. That
  is today, in the Tethys' closing; Stern & Scholl put today's suturing at 12,000 km against a
  long-term 6,000.
- made by extension at continental rift boundaries: Table 3's CRB, +0.011502 m2/s, +0.36 km2 a
  year.
- consumed by subduction erosion: the trench migrates inboard at about 2.5 km/Myr (Scholl & von
  Huene 2010, 1 to 5), a median 3.0 on Clift & Vannucchi's erosive margins (Table 2, 0.9 to
  4.7), so about 0.06 km2 a year on their 22,925 km.
- consumed by the subduction of rifted margins in collisions: Clift et al. 2009's 0.43 km3 a
  year at 18 km, about 0.024 km2 a year.

So the model's book under T is 0.38 made against 0.30 consumed, Earth's (as far as these
sources go) 0.36 by extension plus the arcs' against about 0.51. What is off:

- Subduction erosion consumes no area. PLT-441 thins the forearc and never removes it
  (`forearc` takes volume only; continental crust never turns oceanic, `c2o` 0 in every
  process): about 0.06 km2 a year on Earth, none here. This is missing physics, the trench's
  retreat.
- The collisions consume 0.30 km2 a year under T (the book's two sinks, all of them at
  collision fronts, since nothing else consumes continental area; the area crossing the fronts
  is 0.251, PLT-222's Voronoi crossing, which is exact on a jagged front; dock-c3 0.585), against
  Bird's 0.43 for today and about half of it for a long-term suturing length. Base3's was above
  Earth's; T's is between the two.
- The arcs' maturing makes 0.26 km2 a year on island-arc fronts about three times Earth's
  length, and extension (collapse and the rate part, 0.10) makes less than Bird's CRB. The first
  is gap 2's again.

**Why one plate grows past half the planet.** Under T one plate holds over half the planet in a
median 16.5 % of the steps (8.6 to 26.8 by run; dock-c3 0 in 8 of 8; the plate check's band at
most 10). At those steps its own crust is 0.679 continental (median) and it holds 0.730 of all
the continental crust: it is the plate of an assembled supercontinent about 0.4 of the planet
across. Base3's rare ones were the same (0.542 and 0.676). The rift hazard (PLT-083, gap 2's
stress weights on) splits it as it should: its λ per 2-Myr step is a median 0.052, a rift in
about 39 Myr, far from the λe^-λ roll-off; the 47 episodes above half last a median 14 Myr (p90
70); the plate check counts 3 to 12 rifts of a plate above 40 % per run, after 20 to 63 Myr
above it (medians by run), and 20 to 100 % of those rifts are still apart 50 Myr later. What
regrows it is mostly the same lineage, by its ridges and the small plates absorbed into it, and
in some episodes a docked continent of 20,000 to 40,000 samples. With continents 0.57 of the
planet a broken supercontinent's pieces are big and few (`continent_pieces_p50` 3 to 4 against
base3's 5 to 10), so they meet again soon. The megaplate follows from the area; breakup is not
what is missing, and no rift law changes.

**The plate check, dock-T against dock-c3** (runs passing of 8). Failing more under T, and
following from the area or the sea: the arcs' continental heights (p50 7 to 0, p90 8 to 0;
medians 1,494 to 608 m and 3,299 to 1,897 m: drowned), one plate above half (8 to 1), the
highest continental share (1 to 0; 0.467 to 0.580), the largest land gain (4 to 0), the largest
plate's median share (8 to 4), the plates above 5 % (8 to 5). Failing more and T's own, to be
read again when the area is right: the collisions' convergence ratio after 20 Myr (7 to 3) and
rifts welded back within 50 Myr (7 to 4). Passing more under T: the collision count and
lengths, the lowest continental share, the slab-pulled plates' speeds. Failing in both: the
subducting share, the active margins, the RMS speed, the supercontinent period, the wedge and
the foredeep. This lane reads "the plate check passes" as: no judged measure passes in fewer
runs than in dock-c3 (one run of noise allowed), with no guard step.

**The stages.** Each behind a switch with unit tests against its sources, one cumulative 8-run
arm with T on against dock-c3 per stage:

1. **S1, the trench's retreat under subduction erosion.** An erosive margin truncates the
   overriding plate: the front sample's whole column goes down and the sample leaves its plate,
   so the front, the forearc and the arc move inboard. On Earth's numbers it is small for the
   area: 2.5 km/Myr on the model's 15,100 km of erosive front is about 0.038 km2 a year, a
   quarter of it under continental crust, against the +0.076 to close. It is the missing
   physics all the same, and the arm says what it does to the arcs' maturing (an eroded
   island-arc forearc no longer matures).
2. What comes after S1 is decided on its arm: the collisional consumption (T ends a collision by
   moving the subducting plate's whole connected continent onto the overriding plate, a median
   5,528 samples, after which nothing shortens it, where India has gone on converging with Asia
   for 50 Myr) or the arcs' maturing threshold against Earth's continental area per km3 of arc
   crust. (Read below: neither; the lane stops after S1.)

Not this lane's: the subduction zones' length and the plates' slowness (gap 2), the belts
spreading inland (a later lane, the coordinator's call).

**Arm dock-TS1: T and S1** (`pool.sh run dock-TS1 --length 1000 --seeds 1,2,3,4 --set
tectonics.transference=true --set tectonics.erosion_retreat=true +DEEPTIME_CRUST_BUDGET=1`, key
f2c8da8b0dd4, argkey 295dc57c, 8 runs, both starts; with S1 off the key is bit-identical to
base3, `pool.sh alias`, seed 1, 1000 Myr). Medians, dock-T to dock-TS1:

- The retreat does what it was built to do: 2.44 to 2.60 km/Myr per lattice km of continental
  erosive front in every run (2.7 to 2.9 per km of front, by the lattice's 1.12 below; 2.55 to
  2.71 by the corrected 1.04, PLT-530), taking
  0.011 km2 of continental area a year (0.022 of the planet over the run; a quarter of the erosive
  front is under continental crust). Subduction erosion's volume falls from 0.83 to 0.61 km3 a
  year: the retreat replaces the band's thinning, and an island-arc forearc's column is thinner
  than a continent's.
- The continental area does not fall: 0.571 to 0.621 at the end (higher in 6 of 8 seeds;
  Mann-Whitney U 7 of 64, p about 0.01), and the sea stands 2,024 m over the datum (1,730). By the area book's means over
  the runs: the retreat -0.022, the fronts' hiding -0.412 to -0.380, the collapse +0.144 to
  +0.171, the arcs' maturing +0.506 to +0.518, the sum +0.153 to +0.201. The fronts' change is
  the supercontinent start's (-0.390 to -0.330; the scattered start's -0.434 and -0.430), and one
  part moves more from run to run within an arm (the fronts -0.34 to -0.48 in dock-T) than S1's
  own -0.022. The rise is not traced further: whichever way it is read, S1 is a sink of 0.011
  km2 a year against a surplus of 0.076.
- The plate check against dock-c3 (runs passing of 8), failing more as under T: the arcs'
  continental heights (p50 7 to 0, p90 8 to 0; medians 567 and 1,727 m), one plate above half
  (8 to 3, 13.5 % of steps), rifts welded back within 50 Myr (7 to 2), the largest land gain
  (4 to 0), the speed-area rank (8 to 4), the largest plate's median share (8 to 6), the big
  plates' median speed (8 to 6). Against dock-T it passes more on one plate above half (1 to 3),
  the largest plate's share (4 to 6), the plates above 5 % (5 to 7) and the collisions'
  convergence ratio (3 to 6), and fewer on the speed-area rank (7 to 4) and the collisions per
  100 Myr (8 to 6). The guard: no step beyond 2 Earth reliefs and `guard_bind_steps` 0 in 8 of
  8; its worst 15,042 m from the sea (dock-c3 12,061, the sea higher). The V1 census: land
  0.065 (dock-c3 0.319), continental crust under the sea 0.897, criteria met 4 (5).
- So S1 does not make T and the sink correct on the pool.

**The front's length on the lattice.** The rates per km of front (the arcs, PLT-045; subduction
erosion, PLT-441; the retreat, PLT-522) book one lattice spacing (76.7 km on the quick lattice) per
front sample. Counted that way a straight boundary is 1.12 times its length (1.08 to 1.21 over 40
random great circles on a 100,000-point Fibonacci lattice, `build/dock/zigzag.py`, the agent's
script; a hexagonal lattice gives about 1.10). The factor 1.84 used earlier on this page and in
`measure_boundary.py` is the model's whole boundary in lattice km (480,054) over Bird's 260,487
km, which assumes the model's boundary is as long as Earth's; it is not the lattice's zigzag. The
model's boundary is about 428,600 km, 1.65 times Earth's with 24 plates against 52 (gap 2). The
collision fronts' lengths and crossings (PLT-222) are straight lengths already. So each per-km
rate is about 12 % over its source per km of the model's own front: the retreat's 2.5 is 2.8,
inside Scholl & von Huene's 1 to 5 and Clift & Vannucchi's 0.9 to 4.7. One change would serve all
three: each front sample's share of the straight front, its crossing sum along the boundary's
normal (`shortening::crossing_sum`, `boundary_normal`). It is not made here, because through the
arcs it changes the default planet.

(Corrected 2026-10-24, the front lane, PLT-530. The spacing the counters book is sqrt(4 pi / N)
R, 71.4 km on the quick lattice; `zigzag.py` took the hexagonal neighbour distance, 76.7 km, so
its 1.12 is 1.04 on the counters' own spacing, and the crossing sums agree: on great and small
circles each side's straight shares add up to the circle's length within 1.5 % and 4 %, and the
lattice counts it 1.043 times (the test `the_straight_shares_add_up_to_the_boundarys_length`).
So each per-km rate is about 4 % over its source, not 12 %: the retreat's 2.44 to 2.60 per
lattice km is 2.55 to 2.71 per km. Under T the fronts are about 45,900 and 102,900 km, not 42,700
and 95,800, against Earth's 14,500 and 27,500; on master's base the boundary is about 425,000 to
460,000 km, 1.63 to 1.77 times Bird's (the gap 2 section's base 1.63, the plate geometry's
c790d1f3 1.77). The collision fronts are straight lengths, so `measure_sutures.py` no longer
divides them by 1.84 (version 3, PLT-512). The change named here is built: "The front's length:
why the plates are ragged (2026-10-24)".)

**What comes after S1: nothing in this lane.** With the lengths read right, no flux left in this
lane is off Earth's numbers in the direction that would close the area:

- the collisions consume 0.25 to 0.30 km2 a year, against Earth's long-term of about 0.2 and
  today's 0.43, so making them consume more would take them off the long-term;
- extension (collapse and the rate part) makes 0.10, below Bird's CRB of 0.36, so righting it
  adds area;
- the retreat is in (S1), at Earth's rate;
- the arcs mature at Cloos's 15 km (PLT-047), a sourced threshold.

What is off is the island-arc front's length, about three times Earth's, and the plates'
slowness (the erosive front is 0.66 of Earth's): gap 2's boundary organisation. (Corrected
2026-10-24: this said Ben deferred it until after V1 item 1. He did not: gap 2 was his scope and
it is built; holding the plate count and the lattice was the coordinator's call.) On Earth's 14,500 km of island arc the maturing would make about 0.09 km2 a
year, and the book would stand at about 0.21 made against 0.31 consumed (an inference from the
book's parts, not a run). The lane stops here: the coordinator's step 3 stops when a stage makes
T and the sink correct, and never tunes to a V1 number. T and S1 stay off by default, both built
and correct.

## The front's length: why the plates are ragged (2026-10-24)

The coordinator's brief for the front lane: the island-arc front is about three times Earth's
(45,900 km under T against Stern & Scholl's 14,500) and the whole boundary is about 1.65 times
Bird's with 24 plates against 52. Fewer plates with a longer boundary points at the boundary's
shape, not the count. Four steps: read the fronts' rates per km of straight front (step 1);
diagnose the length from cached runs, with measures in a registry and no new simulations (step 2);
build the physics the diagnosis names, behind switches with unit tests against their sources
(step 3); T and S1 again on top, as one cumulative arm (step 4).

### Step 1: the fronts' rates per km of straight front (PLT-530)

`tectonics.front_share` gives each front sample its share of the straight front: its lattice
edges' crossings on the boundary's normal (`shortening::crossing_sum`, PLT-310), which along a
front add up to the front's length with the lattice's zigzag taken out, as a collision front's
length is measured (PLT-222). Subduction erosion (PLT-441), the trench's retreat (PLT-522, PLT-523)
and the fronts' km x Myr globals take it. The arcs' magma does not: it is an areal field over the
distance from the fronts (PLT-045's band, `w_arc_of`), so per km of the band, and the switch leaves
it alone. Counted one spacing a sample (the rule it replaces), a straight boundary is 1.043 times
its length on the lattice (the test `the_straight_shares_add_up_to_the_boundarys_length`), not 1.12
or 1.84 as this page said before ("The front's length on the lattice" above, corrected).

Arm fl-share against dock-c3 (base3's seeds 1 to 4, `earthlike_quick_16k`, 1000 Myr, both starts,
8 runs each; key 73dd051b08e8): on the model's own fronts one spacing a sample counts 1.32 times
the straight front (the extra 0.27 over the lattice's 1.04 is the fronts' raggedness at the
lattice's scale, which the share does not count); subduction erosion is 30 to 45 % lower; the arcs'
crust is almost unchanged; land 0.313 (p 0.80); the V1 census's 5 criteria met in both; no step
beyond 2 reliefs, the worst 12,588 m against 12,061 (p 0.13); the plate check passes 210 against
207, `guard_bind_steps` 0. It is correct, so it is on by default and one spacing a sample is off.

### Step 2: the boundary against Bird's

Two pool measures over fl-share's 8 runs (registry `tools/review/pool/measures_front.json`, ledger
PLT-531): `front` from the dynamics record (each plate's perimeter against a spherical cap of its
area, split by size, age, origin, speed, crust and the speed guard), and `boundgeom` from the crust
budget's plate maps every 50 Myr (the boundary as Bird drew PB2002's: Voronoi chains between the
plates, triple junctions, pieces from junction to junction, Bird's divider walks at a 77 km and a
380 km chord, each piece's tortuosity over the great circle between its ends). PB2002 is measured
the same way from Bird's own files (`tools/review/pool/bird/bird.py`) and rasterised on the 16k
lattice (`bird_raster.py`), so what the lattice alone adds to Earth's numbers can be read off.
Medians over the runs, snapshots after the first 100 Myr:

| | fl-share | PB2002 on the lattice | PB2002 |
|---|---|---|---|
| boundary at a 77 km chord, km | 525,500 | 273,631 | 246,541 |
| at a 380 km chord, km | 347,200 | 236,679 | 231,009 |
| perimeters over caps' perimeters, 77 km | 3.43 | 1.58 | 1.42 |
| at 380 km | 2.27 | 1.37 | 1.27 |
| each plate's at 380 km, area-weighted | 2.34 | 1.40 | 1.32 |
| triple junctions | 292 | 98 | 100 |
| pieces between junctions | 438 | 147 | 150 |
| closed loops | 250 | 1 | 0 |
| tortuosity (sum over sum), 77 km | 1.86 | 1.32 | 1.22 |
| at 380 km | 1.38 | 1.15 | 1.14 |
| the subduction pieces', 77 km | 2.21 | 1.20 | 1.15 |
| at 380 km | 1.52 | 1.10 | 1.11 |
| subduction front at 77 km, km | 216,500 | 52,564 | 51,310 |
| at 380 km, km | 141,200 | 48,406 | |

What it says:

- **The plates are not too few for the boundary; their outlines are too long.** 22.7 plates
  against 52 have less cap perimeter between them than Earth's, yet the boundary is 1.9 times
  PB2002's on the lattice at 77 km and 1.5 times at 380 km, and each plate's perimeter is 2.3 times
  a cap's at 380 km against Earth's 1.3. The lattice itself adds 11 % at 77 km and 2 % at 380 km
  to Earth's map.
- **The map is not a plate map.** Plates meeting three at a time with no islands have
  2 (P - 2) triple junctions and 3 (P - 2) boundaries (Bird 2003 [110]): 41 and 62 for 22.7 plates.
  The model has 292 and 438, and 250 closed loops, where every one of PB2002's 150 boundaries runs
  from a junction to a junction ([112]: "all are of type 1"). A plate here is on average 18
  connected pieces (`comps_per_plate`), and 7 % of the visible samples lie outside their plate's
  largest piece (`frag_samples`).
- **It grows with age.** The perimeters over the caps' at 380 km are 2.19 in the first snapshot
  after 100 Myr and 2.31 in the last; by the plates' age (`front`, at the lattice's scale) 2.37 for
  plates under 50 Myr old and 3.35 for those 200 Myr or older.
- **What it is not.** The speed guard holds no plate (its share of the perimeters 0); slow plates
  (under 20 mm/yr, 2.93) are no more ragged than fast ones (3.06); continental plates (3.10) and
  oceanic ones (2.93) alike; plates born of a rift (2.92) less so than the start's (3.18). By size:
  under 0.0377 sr 2.17, up to 0.5 sr 2.81, larger 3.17 (PB2002 at 77 km 1.38, 1.35, 1.47).
- **The front by plate size.** The plates under 0.0377 sr subduct 3.6 % of the front (PB2002 11.2
  %), those over 0.5 sr 41 % (55 %); per subducting plate the front is 0.45 of its cap's perimeter
  (area-weighted; PB2002 0.19).

### The fragments and the plates' main bodies

A plate in the model is the set of samples that carry its id, and nothing in the step keeps them
together. `boundgeom` version 2 finds each plate's connected pieces (through Delaunay edges between
its own samples) and measures the boundary twice: as it is, and with every piece outside a plate's
largest given to the plate that most of its neighbours in a plate's largest piece belong to (`clean`:
the boundary as it would be with each plate one body). Medians over fl-share's 8 runs (ledger
PLT-533):

| | fl-share | each plate one body | PB2002 on the lattice |
|---|---|---|---|
| boundary at a 77 km chord, km | 525,500 | 332,700 | 273,631 |
| at a 380 km chord, km | 347,200 | 256,100 | 236,679 |
| perimeters over caps' perimeters, 77 km | 3.43 | 2.14 | 1.58 |
| at 380 km | 2.27 | 1.65 | 1.37 |
| each plate's at 380 km, area-weighted | 2.34 | 1.76 | 1.40 |
| triple junctions | 292 | 40 | 98 |
| pieces between junctions | 438 | 60 | 147 |
| closed loops | 250 | 0.6 | 1 |
| tortuosity (sum over sum), 77 km | 1.86 | 1.86 | 1.32 |
| at 380 km | 1.38 | 1.44 | 1.15 |
| the subduction pieces', 77 km | 2.21 | 2.34 | 1.20 |
| at 380 km | 1.52 | 1.66 | 1.10 |
| subduction front at 77 km, km | 216,500 | 116,200 | 52,564 |
| at 380 km, km | 141,200 | 80,500 | 48,406 |

- **The fragments are 7 % of the crust and 37 % of the boundary.** 7.1 % of the visible samples
  lie outside their plate's largest piece (quartiles over the runs 5.5 to 7.7 %): 5.3 % in pieces of
  over 100 samples (510,000 km² or more; PLT-091's floor for a plate is 1.53 million km²), 1.1 % in
  pieces of 11 to 100 and 0.7 % in pieces of 1 to 10. Around them runs 37 % of the boundary at 77 km
  and 46 % of the subduction front (the island-arc front, where the overriding sample is oceanic:
  120,600 km at 77 km, 65,700 with each plate one body; a chain of the map, not the front samples'
  count over time that gives the 45,900 km above).
  With them given to the plates around them the map becomes a plate map: 40 triple junctions where
  plates meeting three at a time have 2 (P - 2) = 41, 60 pieces for 3 (P - 2) = 62, and under one
  closed loop.
- **They are whole regions of a plate, far from it.** A one-off look at one run (fl-share's
  supercontinent start, seed 1, every 200 Myr): the fragments hold 7.0 % of the samples at 200 Myr
  (530 pieces), 13.6 % at 400, 9.3 % at 600, 4.2 % at 800 and 4.0 % at 1,000; three fifths or more
  of their samples are in pieces of over 100 samples, and two thirds or more lie more than 12
  Delaunay steps (about 850 km) from their plate's largest piece; about half carry the
  subduction-zone flag, a quarter to two fifths the divergent one, and an eighth to a third are
  continental. They are pieces of active boundary that move with the rotation the force balance
  sets for their plate as a whole, a body they no longer touch.
- **The groups cannot say where they came from.** Every fragment's samples belong to rigid groups
  that also hold samples of its plate's largest piece (`frag_cut` 7.09 %, `frag_own` 0.04 %), which
  read alone says the pieces were cut off their plate rather than handed to it. But a resampling
  (about every 20 Myr) gives each plate one unrotated group, so a terrane handed to a plate before the
  last resampling reads the same. The cut trace below tells them apart.
- **The main bodies are still too ragged, and most at the trench.** With each plate one body the
  boundary is 1.22 times PB2002's on the lattice at 77 km and 1.08 times at 380 km, with 23 plates
  against 52; each plate's perimeter is 1.76 times its cap's (area-weighted, 380 km) against Earth's
  1.40. On Earth the subduction pieces are straighter than the boundary as a whole (tortuosity 1.20
  against 1.32 at 77 km, 1.10 against 1.15 at 380 km); in the model they are the most tortuous
  (2.34 against 1.86, 1.66 against 1.44). The two maps' pieces differ in length (about 4,200 km
  between junctions at 380 km against 1,600 on Earth), and a longer piece of the same curve is more
  tortuous, so the tortuosities are read within each map, not across them. The front is 35 % of the cleaned
  boundary at 77 km against PB2002's 19 % (the plate check's subducting share, 36.6 % against Bird's
  19.7 %, fails in every run): after the fragments, the front's excess is its raggedness and its share
  of the boundary.

### What cuts the plates

`plates/cut_trace.rs` (measurement, off unless a test turns it on; the ignored test
`where_the_plates_are_cut` runs the quick preset at 40,000 samples for 300 Myr from each start)
counts each plate's samples outside its largest connected piece (over lattice edges between visible
samples, `geometry::cut_samples`) before and after every part of the step and books what each part
cut and joined; a collision's outcome is split by the plate it changes (the subducting plate and
the overriding one). Between resamplings the lattice edges between two groups are the last
resampling's, so a part's cut can be transient; the resampling's own count is what the true
positions join or cut again. Samples cut, net over 300 Myr:

| part of the step | supercontinent, seed 1 | scattered continents, seed 2 |
|---|---|---|
| collision, terrane transfer: the overriding plate | +2,440 | +3,641 |
| the same: the subducting plate | +686 | +764 |
| overlaps resolved (`subduction::resolve`), net (cut, joined) | +1,448 (7,761, 6,313) | -594 (9,029, 9,623) |
| subduction initiation | +301 | +151 |
| rifts | -58 | -103 |
| welds | none | 0 (the subducting plate +76, the overriding -76) |
| the resampling (new crust in the gaps, every sample to its nearest lattice point) | -2,478 (705, 3,183) | -153 (2,871, 3,024) |
| every other part (motion, shortening, belts, fronts, forearcs, collapse, flow, forces, heal, burial) | 0 | 0 |
| the share cut off at 300 Myr | 5.8 % | 9.3 % |

- **The collision's terrane transfer cuts the plates.** It moves the subducting plate's continent
  connected to the contact into the overriding plate as groups of their own and leaves the stacked
  strip hidden with the subducting plate (`plates/collision.rs`). The terrane joins the overriding
  plate across the subducting plate's hidden strip, so the overriding plate gains a piece that
  touches it nowhere, and the subducting plate keeps ocean cut off behind the terrane.
- **The overlaps churn.** Hiding the overridden samples cuts and joins almost as much
  (7,761 against 6,313, 9,029 against 9,623): an overlap hides a neck of a plate and the next step's
  motion opens it again.
- **Nothing makes a plate whole.** The resampling joins more than it cuts, but only where the true
  positions touch again; `connect_plates` runs only at the start and in rifts
  (`geometry.rs`). A piece once cut stays a part of its plate, with its plate's rotation, until an
  overlap or collision happens to rejoin it or it is subducted.
- **What it is not.** Rifts join more than they cut. The parts that hand no sample to another
  plate and hide none (the motion, the shortening, the belts, the fronts, the force balance and
  its speed guard) cannot cut on the lattice; what the motion does to the true positions shows at
  the resampling, which joins more than it cuts.

### What Earth does

- Gordon 1998 (full text): "The central assumption of plate tectonics is that the plates are
  rigid." A plate moves as one body because it is one: its pieces carry its stresses to each other.
- Bird 2003 (full text): PB2002's plates file "contains 52 segments", each "a closed curve
  outlining that plate" ([123]), so every plate is one region; all 150 boundaries run from a
  junction to a junction ([112]), and none of its plates is an island: "The Shetland plate comes
  closest to being an 'island' plate (type 2) because it is almost surrounded by the Antarctica
  plate, but it also has a short boundary with the Scotia plate" ([112]). "The term 'microplate'
  has not been precisely defined, but in practice is used most often for a plate of 100–500 km
  dimensions which appears spontaneously in the center of an ocean spreading system" ([79]).
- Stock & Lee 1994 (full text): the Farallon plate's last pieces off Baja California and
  California broke into microplates down to about 100 km across as the Pacific-Farallon ridge
  neared the trench. "If a small fragment breaks off a large plate, then it may lose its slab-pull
  force and become driven entirely by different conditions." The Monterey, Arguello, Guadalupe and
  Magdalena microplates "became attached to the Pacific plate when their ridge boundaries with the Pacific plate stopped spreading", and "None of
  these microplates became attached to the North America plate"; in general "a very small
  microplate with insufficient slab-pull force may stagnate or attach itself mainly to the
  overriding plate".
- Wang et al. 2013 (full text), of the same margin: "Some fragments were incorporated into the
  Pacific plate as subduction and spreading ceased."
- Nicholson et al. 1994 (abstract only): the Monterey microplate captured by the Pacific plate at
  about 20 Ma.
- Lonsdale 2005 (abstract only): the Farallon plate split into the Cocos and Nazca plates at about
  23 Ma, driven by the "divergence of slab-pull stresses".
- Mallard et al. 2016 (full text, read 2026-10-21): "the stresses caused by the bending of trenches
  break plates into smaller fragments".

No source we read states that a plate must be one connected region, but none describes one that
is not: a piece cut off a plate either moves as a plate of its own (fission) or joins a plate it
adjoins (capture).

### The diagnosis

The boundary is too long for two reasons, in this order:

1. **The plates are not one body each.** 7 % of the crust lies in pieces cut off their plates,
   chiefly by the collision's terrane transfer and the overlaps' hiding, and nothing makes a plate
   whole again. These pieces carry 37 % of the boundary and 46 % of the subduction front and turn a
   map of 23 plates into one of 292 junctions and 250 loops. Earth has no such pieces: a cut-off
   piece becomes a plate or joins its neighbour (above). This is step 3's law.
2. **The main bodies are ragged, most at their trenches, and more so with age.** Without the
   pieces the boundary is still 1.2 times Earth's on the lattice at 77 km with fewer than half the
   plates, the front is 35 % of it against 19 %, and the subduction pieces are the model's most
   tortuous where they are Earth's straightest. Not the speed guard, not slow plates, not rifts
   (above and "Step 2"). Which part of the step roughens a trench is left to the arm with step 3 on:
   with every plate one body, the trace and the measures can be read on a plate map.

### Step 3: one plate, one body (PLT-532)

`tectonics.plate_fission`: right after each resampling (`ridges::bury`, when every group shares
the lattice and is unrotated), each plate keeps its largest connected piece of visible crust. Every
other piece of PLT-091's floor for a plate (`min_plate_area_frac`, 0.003 of the planet) or more
becomes a plate of its own with the parent's rotation, its birth recorded with origin "fission"
(the force balance sets its motion from the next step); every smaller piece joins the plate most of
its lattice neighbours belong to, as a plate below the floor is absorbed. No new constant: the size
that separates fission from capture is the model's own floor for a plate. The unit test
`a_plate_cut_in_two_splits_and_a_cut_off_patch_is_captured` (`ridges.rs`): a plate cut in two by a
band of another, with a patch of it inside the band; on, its larger piece stays, the other becomes a
plate with its rotation and its id as parent, the patch joins the band's plate, every plate is then
one piece and no sample moves; off, the plate keeps its three pieces.

The cut trace with it on: the share cut off at every 50 Myr is at most 1.0 % (off up to 11.7 %, and
5.8 and 9.3 % at 300 Myr);
15 and 23 plates are born of fission in 300 Myr, taking 4,298 and 6,886 samples, and 5,584 and 5,629
samples are captured; 23 and 26 plates at 300 Myr against 17 and 22 off (the scattered start peaks at
34). The step still cuts between resamplings, more than off (the terrane transfer +5,532 and +4,054,
the overlaps +4,394 and +7,264), and each resampling makes the plates whole again (it joins 10,456
and 11,637).

**Arm fl-fission** (fission on top of fl-share, base3's seeds 1 to 4, `earthlike_quick_16k`, 1000
Myr, both starts, 8 runs; key c94090963380, argkey 686beee0). Against fl-share, medians:

- the guard's worst is 10,348 m against 12,588 (p < 0.001), with no step beyond 2 reliefs, and the
  V1 census meets its 5 criteria in both arms (land 0.327 against 0.313, p 0.80);
- 27.2 plates against 22.7 (`boundgeom`), with 0.9 % of the visible samples outside their plate's
  largest piece at the 50-Myr snapshots against 7.1 % (the pieces left are cut between
  resamplings);
- the boundary at a 77 / 380 km chord is 382,434 / 284,518 km against 525,489 / 347,237, with 113
  triple junctions against 292 and 91 closed loops against 250;
- the subduction front at 77 km is 147,193 km against 216,472;
- the plate check passes 201 measures against 210.

But 162 collision states parted within two steps of a fission of one of their plates, which is
the defect Step 3b mends.

**The plate count against Earth's.** The model's floor for a plate (PLT-091, 0.0377 sr) is the
count to read the model's plates against, not PB2002's 52: 19 of PB2002's plates are at or above it
in Bird's table (20 on the lattice, where the Caroline plate, 0.03765 sr against the floor's 0.03770,
rasterises to 0.0382; `bird_raster.py`'s `plates_at_floor`), and they hold 97.9 % of the planet.

**A known limit: no microplates.** PB2002's 33 plates under the floor (2.1 % of the planet; Bird's
microplates are "100–500 km" across, [79]) have no counterpart: a piece under the floor is captured,
and nothing in the model makes a plate that small. Fission splits off only pieces of the floor's
size or more, so the law cannot make them either.

### Step 3b: a boundary belongs to the lithosphere on its two sides (PLT-534)

Arm fl-fission (fission on top of fl-share, 8 runs) found the defect that fission exposed. The
state of a boundary is kept per pair of plates: a contact (polarity, onset, onset event), a slab
(its length down the dip) and a collision state. When a plate splits, the parent keeps its id and
every such state stays keyed to the parent, wherever its lithosphere went. A piece that holds a
trench starts a new contact with no slab and no onset, and a collision it carried parts once its
front is gone: 162 collision states parted within two steps of a fission of one of their plates.
Rifts and subduction initiations split plates in the same way, so the defect is older than
fission.

What Earth does. Plate identity is bookkeeping, and the boundary goes with the lithosphere on its
two sides.

- **A subducting plate that splits keeps subducting on both pieces.** Stock & Lee 1994: "The
  fragmentation of a subducting plate creates a new triple junction, two limbs of which are along
  the trench". Of the Cocos and Rivera plates, the triple junction "has not moved much with respect
  to the North America plate since its inception because subduction of both plates has been nearly
  orthogonal to the trench".
- **The slabs stay with their pieces.** Wang et al. 2013: "Some fragments of the Farallon plate
  still exist as separate plates subducting beneath North America, including the Cocos and Juan de
  Fuca plates, which subsequently further fragmented generating the actively subducting Rivera,
  Gorda, and Explorer microplates", and high-velocity anomalies "which we interpret as the fossil
  slabs still attached to the remnant oceanic microplates".
- **An overriding plate that splits leaves the trench with the piece beside it.** In a back-arc
  basin, the arc keeps the trench and the remnant arc is left behind. Ishizuka et al. 2010: "The
  West Mariana Ridge (WMR) is a remnant arc (Karig, 1972) that separated from the magmatic arc by
  opening of the Mariana Trough". Stern, Fouch & Klemperer 2003: the trough "is bounded to the east
  by the active Mariana arc, to the west by the remnant arc of the West Mariana Ridge [Karig, 1972],
  and to the south by the Challenger Deep part of the Mariana Trench".
- **The rift line can fall on either side of the arc.** Taylor & Karner 1983 (abstract): rifting
  "may occur on either side of the line of active volcanoes (±50 km)". So the piece that keeps the
  trench must be found from the geometry, not from which side of the rift it is on.
- **A continent.** As South America separated from Africa, the Nazca subduction system kept rolling
  back at its margin "during the opening of the Southern Atlantic for almost ~100 Myr" (Faccenna et
  al. 2017). That this is because only South America borders the Nazca plate is our reading, from
  memory.

None of these sources states the rule as a rule, and none contradicts it.

The law, `tectonics.split_inheritance` (`plates/inherit.rs`). It runs after every split: a fission
(`ridges::split_cut_plates`), a rift (`rifting::rift`), and a subduction initiation's new plate
(`rifting::initiate`). Each contact, slab and collision state that the parent had with another plate
goes to the pieces that border that plate.

- **Bordering.** A piece borders a plate when at least 3 of its samples (`MIN_CONTACT`), hidden
  ones included, have a sample of that plate within 1.5 lattice spacings of their true positions.
  1.5 spacings takes a lattice neighbour, about 1.07 spacings away, and not the ring behind it, at
  about 1.86. True positions are used because mid-step the lattice's edges between plates are
  stale.
- **Contacts and slabs.** Every bordering pair keeps the contact and the slab, the subducting side
  staying the subducting side. The slab is one sheet along the trench, so a copy of a slab that
  was at a front is latent (`SlabTrack::latent`): it pulls only at its own piece's fronts and does
  not hang, and its first front makes it a slab like any other; a copy that reaches no front is
  gone after `slab_breakoff_myr` (PLT-184). A copy of a slab that hung hangs on, along its own
  piece's stretch. (Added after the guard chase below.)
- **The collision.** It goes with its front, to the pair whose border lies nearest the collision's
  front centroid. The other pairs get the contact without the collision.
- **What stays.** A boundary that no piece borders stays with the parent and closes as before. A
  pair inside one split plate is not touched, such as the slab an initiation plants under its
  fracture (PLT-270).

There are no new physical constants: 1.5 spacings and 3 samples are how the lattice is read. Seven
unit tests cover the law and each of its paths:

- in `inherit.rs`:
  - a subducting plate split in two: both halves keep the trench and its slab, the collision stays
    with the half at its front (or moves to the other half when the front is there), and a trench
    that only one half borders moves to it whole;
  - an overriding plate split behind its arc: the strip beside the trench takes the trench, slab
    and collision, and the polar part keeps nothing;
  - a boundary that no piece borders is untouched;
  - `a_split_plates_pieces_pull_with_the_parents_slab_force`: the slab pull and suction summed over
    the pieces equal the parent's (the guard chase below);
- `a_fission_hands_the_cut_off_piece_the_trench_it_borders` (`ridges.rs`);
- `the_fragments_of_a_rifted_plate_keep_the_trench_they_border` (`rifting.rs`);
- `an_initiation_hands_the_sinking_plate_the_trench_it_borders` (`rifting.rs`).

**Arm fl-fission2** (fission and the inheritance on top of fl-share, 8 runs, `earthlike_quick_16k`,
1000 Myr, key 0fc579, argkey a8641acb). Against fl-share, medians with the Mann-Whitney p:

- **The defect.** The collision states parted within two steps of a fission fall from 162 to 58.
  Per run, the splits moved 238 to 309 contacts and copied 276 to 353; they moved 37 to 53
  collisions, and moved 205 to 266 slabs and copied 262 to 355.
- **The guard.** The guard's worst is 10,490 m against 12,588 (p < 0.001) and dock-c3's 12,061
  (p 0.015). No step goes beyond 2 reliefs, and `guard_bind_steps` is 0.
- **The V1 census.** It meets its 5 criteria in both arms (land 0.322, p 0.72). Its collisions
  per 100 Myr fall from 2.0 to 0.9 (p 0.002).
- **The boundary.** It is 352,268 lattice km against 478,669 (Bird 260,487, or 271,700 on the
  lattice). Its subducting part is 109,741 km against 159,426 (Bird 51,310), and the subducting
  share is 0.314 against 0.331 (Bird 0.197). The boundary criteria meet 3 per run against 2
  (p 0.017).
- **Its geometry (`boundgeom`).**
  - 27.6 plates against 22.7.
  - At a 77 / 380 km chord the boundary is 387,493 / 288,070 km, and the perimeters over the
    caps' at 380 km are 1.646 against 2.266 (PB2002 on the lattice 1.365).
  - There are 115 triple junctions against 292, and 96 closed loops against 250. With each plate
    one body, 50 junctions (2 (P - 2) = 51) and 0.5 loops.
  - With each plate one body, the front at 77 km is 115,544 km against PB2002's 52,564 on the
    lattice, and its tortuosity is 2.03 against 1.20.
- **The sutures.** 42.4 per 100 Myr against 51.8 (Macdonald 19 to 39), and 41,589 km per 100 Myr
  against 68,397 (Macdonald 23,235 to 50,919). The front in collision is 24,217 km against 36,100
  (Stern & Scholl 6,000 to 12,000).
- **Fission's churn.** 56 to 90 plates are born of fission per run (median 64.5). Of the 531 born
  in the 8 runs, 478 died: 425 absorbed under the floor, 44 consumed and 9 sutured. Plates born of
  a subduction initiation, 78 to 124 per run (fl-share 69 to 103), die absorbed as well: 548 of 639
  deaths. The resampling's own roughening (below) is the likely feed, since every resampling cuts
  a ragged plate's thin necks again.

The plate check passes 187 measures against 210. The coordinator's ruling (2026-10-24): its
targets are realism checks, not a gate, and correctness is judged on the unit tests, divergence
and the guard. Its moves against fl-share (passes of 8 and the median), each with its target's
ledger status:

| measure | fl-share | fl-fission2 | the target's source |
|---|---|---|---|
| `speed_oceanic_over_continental` | 5/8, 2.18 | 1/8, 1.72 | CHK-024, verified (Zahirovic et al.); fl-fission still 5/8, so the drop comes with the inheritance |
| `supercontinent_tenure_myr` | 1/8, 57.5 | 0/8, 23.5 | CHK-035, verified |
| `largest_share_p50` | 8/8, 0.234 | 7/8, 0.169 | CHK-001, verified |
| `ocean_old180_pct` | 0/8 | 1/8 | CHK-033, verified (up) |
| `plate_life_p50_myr` | 7/8, 107 | 0/8, 68.5 | PLT-146, abstract only |
| `rift_alive_50_pct` | 7/8 | 5/8 | CHK-006, abstract only |
| `collision_length_p50_myr` | 2/8, 56 | 1/8, 73.5 | CHK-012, abstract only |
| `collision_convergence_ratio_20myr` | 5/8, 0.40 | 2/8, 0.62 | CHK-037, abstract only |
| `continental_min` | 2/8 | 0/8 | CHK-016, cited, not verified |
| `collisions_per_100myr` | 4/8, 2.0 | 0/8, 0.9 | CHK-015, uncited; it counts welds of terranes of 0.1 of a plate or more |
| `dead_within_20_pct` | 1/8, 20.8 | 0/8, 29.1 | uncited |
| `rift_welded_back_50_pct` | 7/8, 3.9 | 5/8, 9.5 | uncited |
| `land_gain_max_pct` | 6/8 | 8/8 | up |
| `continental_max` | 3/8 | 5/8 | up |
| `speed_big_p90_mm_yr` | 6/8 | 8/8 | up |
| `arc_continental_p50_m` | 7/8 | 8/8 | up |

Fission and the inheritance are correct by the lane's rule: their unit tests pass against their
sources, the runs do not diverge and the guard does not fail. So `plate_fission` and
`split_inheritance` are on by default from 2026-10-24 (the coordinator's call, the same day).

**The speed guard (the guard chase, 2026-10-24).** On the 8k test planet (8,000 samples,
scattered continents, 300 Myr) the speed guard (250 mm/yr at a plate's farthest point, PLT-022)
bound on 5 of seeds 1 to 24 with fission and the inheritance (seeds 2, 3, 9, 14 and 15, for 1 to 6
plate-steps each), on 2 with fission alone (5 and 15) and on none with neither. The inheritance
alone bound on all 5 of the seeds it was run on (3, 11, 14, 18 and 23, 1 to 5 plate-steps). On the
16k planet of the pool, fl-fission2's 8 runs never bind. The coordinator held the merge on it, his
suspect being the slab counted twice: both pieces keep the whole slab where both border the other
plate.

- **The pull is not counted twice at the fronts.** A slab pulls per front edge (`forces.rs`): the
  pull on a front sample is the slab's weight times its length down the dip over 700 km, times one
  spacing of trench. The length is not a share of the trench, so each piece keeps all of it and
  pulls only along its own stretch, and the pieces' front edges are the parent's. A trace of every
  slab force (a test-only patch, 24 seeds) shows it at the binds: on seed 2, the plate the guard
  held was pulled by a copied slab along 21 front edges and a moved one along 11, which were
  the parent's 32.
- **A smaller over-count was there.** A piece that borders the other plate only across a ridge or
  a transform also took a copy, and with no front it hung along the piece's whole border (a slab
  with no front hangs, PLT-271) for up to `slab_breakoff_myr`. Of the 27 to 71 slabs copied per
  run, 6 to 25 never reached a front; the copies' hanging pull was 1.3 to 3.2 % of a run's slab
  force (median 2.1 %, though part of it is the copies of slabs that hung before the split, which
  rightly hang on). No bind was at one. The fix: a copy of a slab that was at a front is latent
  (`SlabTrack::latent`) and pulls only at its own piece's fronts until its first one; a copy of a
  slab that hung hangs on along its own piece's stretch. The unit test
  `a_split_plates_pieces_pull_with_the_parents_slab_force` (`inherit.rs`) checks that the pull's
  and the suction's torques summed over a split plate's pieces equal the parent's to 1e-9, for a
  split across the trench and along it, on either plate, and for a hanging slab; without the
  latent copy the splits along the trench fail it.
- **The sweep again, with the fix.** With fission and the inheritance the guard binds on 2 of
  seeds 1 to 24 (15 for 2 plate-steps, 16 for 1), against 5 before; the inheritance alone on 2 of
  its 5 (11 for 3, 14 for 2), against 5. The planet is chaotic, so part of the drop is a reshuffle
  of the runs, not the fix: no bind before it was at a hanging copy. The binds that remain are of
  the same kind (traced as above): on seed 15 a plate of 37 samples born of plate 14 36 Myr
  before, which subducts under it along 10 front edges; on seed 11 (the inheritance alone) one of
  the starting plates, down to 28 to 35 samples, with a plate subducting under it along 10 to 13
  edges; on seed 14 (the inheritance alone) a piece of 3 samples being consumed, the same bind as
  before the fix. Seed 16's one plate-step is not in the step records, which list the plates alive
  at the step's end, so it was a plate removed in the step it bound.
- **With the level set on as well** (Step 3c; the sweep on seeds 1 to 24 with all three on) the
  guard binds on 3 of 24: seed 2 for 7 plate-steps, 17 for 1 and 24 for 7. They are of the same
  kind: on seed 2 a plate of 47 to 50 samples born of a subduction initiation 46 Myr before, with
  a plate subducting under it along 14 front edges; on seed 24 a plate of 32 to 38 samples born of
  a split 56 Myr before, with its parent's slab (moved to it at the split) under it along 12 to 16
  edges, then another along 8; on seed 17 a plate of 23 samples between a trench on either side
  (5 and 6 edges). The 16k planet of the pool never binds with it (arms fl-ls and fl-ls2, Step 3c).
- **What the guard holds: small plates ringed by trenches.** Every bind, before the fix and after,
  is a plate of at most 131 visible samples (1.6 % of the sphere; the floor is 24, PLT-091) with
  other plates subducting beneath it along 30 % to all of its boundary (10 to 33 front edges).
  Most are plates born of a split whose parent subducts under them, so with the inheritance they
  take the parent's other trenches at once; two are tiny plates being consumed (3 and 21 samples).
  The overriding plate's slab suction is a fixed 0.3 of the slab pull per metre of trench
  (PLT-186), against the basal drag of its area, so its speed grows as its trench over its area,
  about 1/√area for a round plate.
- **Earth's fast small plates are of this kind.** Bevis et al. 1995 (the abstract, as a search
  summary gave it; not read): convergence across the Tonga trench rises northwards to about
  240 mm/yr, and the Lau basin behind it opens at up to about 160 mm/yr, the northern Tonga ridge
  moving as a rigid microplate. So a small plate on a trench reaching the guard's 250 mm/yr is not
  unphysical in itself; that the model's small plates reach it with a fixed suction is the open
  question.
- **What it would take.** The suction is a force that does not depend on how fast the trench
  retreats: a small overriding plate is drawn on whatever the slab's rollback. A suction that
  saturates with the trench's retreat (the overriding plate drawn no faster than the hinge rolls
  back) would bound these plates by the physics, not by the guard. It is not built: the
  coordinator's call.
- **The weld tests' seeds.** With the fix, 11 of seeds 1 to 24 weld (1, 2, 8, 11 to 14, 16, 20,
  22 and 23). The collisions test is on its original seed 12 again. The recording test stayed on
  seed 20: its original seed 19 still did not weld, and its seed is the next that welds, chosen by
  the weld, not by the guard (on welding seed 16 the guard binds once, which that test asserts
  against). With the level set on by default (Step 3c) 12 of 24 weld (1, 2, 4, 5, 7, 9, 12, 15,
  17 to 19 and 21): seed 19 welds again and 20 no more, so the recording test is back on its
  original seed 19, and the collisions test stays on 12; the guard binds on neither.

**Known limit: merges.** When two plates merge (a healed rift, a weld, a plate absorbed under the
floor), the dead plate's contacts with third plates still close. That is the same law in reverse,
and this step does not build it.

### What still roughens the plates: the resampling's random walk

With every plate one body, the main bodies are still too ragged, and most at their trenches (the
front's tortuosity with each plate one body is 2.03 at 77 km against PB2002's 1.20 on the
lattice). The resampling is one cause, and it shows on a planet where nothing else happens.

**How the resampling decides membership.** It turns the lattice at random each time (Cortial et al.
2019, §4.3). Each new point takes the plate of the old footprint cell that covers it: the old
lattice's triangles are split between their corners by the largest barycentric weight, which is a
nearest-cell rule. So every boundary is rounded to the new lattice at each resampling. The
rounding errors of successive resamplings are independent, and nothing pulls the boundary back,
so they add up as a random walk. At a contact the overriding plate wins the cover (visible crust
first, then the overriding plate), so the trench is the overriding plate's footprint edge, rounded
in the same way. A ridge is drawn again by the gap fill at each resampling (Cortial's ridge
profile, the segments of PLT-502), so it does not walk.

**The measurement.** The test `how_resamplings_roughen_a_boundary` (`plates/tests.rs`; the table
below is its run with the level set of Step 3c off) takes the two-plate planet (40,000 samples, the boundary on the equator) and resamples it 40 times
with nothing else in the step. In one case the planet is static. In the other, the ocean is turned
under the continent by 0.02 rad between resamplings, which makes a trench where y > 0 and a ridge
where y < 0, with the overlaps resolved as in the step. After each resampling it reads, per stretch
of boundary:

- the offset of plate 0's boundary samples from the equator;
- their spread about that offset, in lattice spacings;
- the boundary's lattice edges per spacing of the stretch's length.

| resamplings | static: spread, spacings | static: edges per spacing | the trench: spread | the trench: edges | the ridge: edges |
|---|---|---|---|---|---|
| 1 | 0.44 | 2.25 | 0.43 | 2.14 | 2.23 |
| 2 | 0.50 | 2.39 | 0.49 | 2.34 | 2.12 |
| 5 | 0.68 | 2.78 | 0.67 | 2.69 | 2.36 |
| 10 | 0.92 | 3.24 | 0.86 | 2.87 | 2.46 |
| 20 | 1.21 | 3.85 | 1.05 | 3.09 | 2.37 |
| 30 | 1.46 | 4.16 | 1.15 | 3.25 | 2.51 |
| 40 | 1.73 | 4.65 | 1.35 | 3.39 | 2.48 |

- **The static boundary walks.** Its spread's variance grows by about 0.07 spacing² per
  resampling, and its length on the lattice doubles in 40 resamplings. A 1000-Myr run resamples
  about 50 times (PLT-098), which is enough to account for a spread of one to two spacings and a
  boundary twice its straight length at the lattice's scale.
- **The trench walks as well, a little slower.** The ridge does not: its edges per spacing stay at
  2.1 to 2.5. Its spread grows only because the ridge migrates and tilts (its offset reaches -16.7
  spacings by the 40th resampling).
- **What it is not.** This is the scheme, not the physics: the same boundary, not moving, comes out
  of each resampling longer than it went in.

The fix is a sub-lattice boundary carried through the resampling (Step 3c).

### Step 3c: the outline between the samples (PLT-536)

`tectonics.level_set_membership` (`plates/level_set.rs`, the footprint in `ridges::resample`)
carries each plate's outline below the lattice spacing, so that a resampling moves it only where
the physics moves it.

**The source.** Osher & Sethian 1988 follow a front as the zero level set of a function carried
with the flow: "the front can also be passively advected by an underlying flow", and "By viewing
the surface as a level set, topological complexities and changes in the moving front are handled
naturally". They initialise the function from the distance to the front (§III.D: "ψ(x , 0) = 1 ±
d 2, where d is the distance from the point x to the initial surface, and the plus (minus) sign is
chosen if x is inside (outside) the initial surface"). Sussman, Smereka & Osher 1994 (abstract
read) describe an interface as "the zero level set of a smooth function"; their reinitialisation,
which keeps that function a signed distance by setting it again from its zero level set, is the
method as we remember it, not read.

**The law.**

- **What a sample carries.** Each sample keeps ψ, the sine of its angular distance to its group's
  outline, capped at 2 spacings. Only a sample with a lattice neighbour of another group is ever
  read, and it lies within one edge of the outline, so the cap is numerical, not physical.
- **Who holds a new lattice point.** In each group's old triangle (in the group's own frame, as
  before), every group present weighs its own corners' ψ by the point's barycentric weights,
  A = Σ w ψ. The group holds the point where its A is the largest, so the outline is where two
  groups weigh the same: the zero line of the interpolated signed distance. Before, the corner
  with the largest weight decided, which puts the outline halfway between two samples whatever
  their distances.
- **Why it is exact for a great circle.** The weights are gnomonic (the point where the ray meets
  the triangle's plane), and a great circle's signed distance n·p is linear in the position, so it
  interpolates exactly: a great-circle outline lands where it was on every turned lattice.
- **The new samples' distances.** A covered point takes its distance inside the top group's
  outline, or its distance outside the outline of a group of another plate that would be on top of
  it, where that is nearer (in front of a trench, the overriding plate's outline is the
  boundary). A point in a ridge's gap takes its distance from the midline, half the difference of
  its distances from the two plates' edges, or from the ridge segment (PLT-502). An orphan takes
  half a spacing.
- **Reinitialisation.** After the resampling and `ridges::bury` (fission, capture, the
  absorptions under the floor), every sample's ψ is set again from the outline: the zero crossing
  on each lattice edge between two groups (t = ψ_u / (ψ_u + ψ_v)), joined inside each triangle into
  great-circle arcs. A triangle with three groups gets three arcs that meet where the three weigh
  the same. Each sample takes its distance to the nearest arc of its group among its own and its
  neighbours' triangles.
- **New boundaries.** A boundary drawn since the last resampling (a rift, a fission, a suture, a
  twin group) has no outline to keep. A sample whose neighbours' membership changed since the last
  reinitialisation gets half a spacing on every corner, which splits its triangles as the old rule
  does. A run's first resampling is the same.

Membership is by group, not by plate, everywhere, as the footprint is.

**A trench's distances.** In front of a trench the ocean's own outline lies under the continent,
since the slab goes on below it. In the first build a new ocean sample whose triangle in the
continent's frame had no continental corner took its distance inside the ocean's outline, which
is too far from the trench, and the reinitialisation's crossings then moved the trench landward,
by 2.1 spacings in 40 resamplings. A group with no corner in a triangle now lies outside it by
the other groups' interpolated distance from their own outline (`level_set::margin`), so such a
sample takes its distance outside the continent's outline, the trench. The test
`a_triangle_without_the_group_lies_its_distance_outside` checks it on a great circle, to 1e-12.

**The test, with and without the level set** (`how_resamplings_roughen_a_boundary`, as in the
table above; the offset in spacings, positive landward at the trench):

| resamplings | static, off: spread, edges | static, on: spread, edges | trench, off: offset, spread, edges | trench, on: offset, spread, edges |
|---|---|---|---|---|
| 1 | 0.44, 2.25 | 0.41, 2.17 | +0.51, 0.43, 2.14 | +0.53, 0.42, 2.11 |
| 2 | 0.50, 2.39 | 0.39, 2.15 | +0.58, 0.49, 2.34 | +0.58, 0.41, 2.24 |
| 5 | 0.68, 2.78 | 0.35, 2.20 | +0.62, 0.67, 2.69 | +0.59, 0.38, 2.37 |
| 10 | 0.92, 3.24 | 0.35, 2.04 | +0.63, 0.86, 2.87 | +0.54, 0.36, 2.02 |
| 20 | 1.21, 3.85 | 0.34, 2.03 | +0.29, 1.05, 3.09 | +0.64, 0.36, 2.15 |
| 30 | 1.46, 4.16 | 0.35, 2.24 | +0.10, 1.15, 3.25 | +0.61, 0.33, 1.99 |
| 40 | 1.73, 4.65 | 0.32, 2.13 | -0.37, 1.35, 3.39 | +0.71, 0.33, 2.19 |

- **The boundary stays where it was drawn.** With the level set the spread does not grow (0.32
  to 0.42 spacings at every count) and the edges per spacing stay within 4 % of the first
  resampling's. The trench moves 0.2 spacings in 40 resamplings, against 0.9 without it.
- **The ridge is unchanged.** It is drawn again by the gap fill at each resampling either way:
  its edges per spacing stay at 2.0 to 2.4 with the level set (2.1 to 2.5 without), and its
  offset reaches -16.8 spacings (-16.7) as it migrates.
- **The test asserts it.** With the level set, after 10 to 40 resamplings, the spread stays
  within 0.15 spacings of the first, the edges per spacing within 15 % and the offset within 0.5
  spacings; without it, the static spread must more than double and its edges grow by half, so
  the test fails if either side stops showing what it measures.

**The cost: a numerical curvature flow.** The interpolated distance is exact only for a great
circle. A curved outline's distance is not linear in the position, and its linear interpolation
(and the reinitialisation's chords) moves the outline towards its centre of curvature at each
resampling, by about c h² κ, where h = √(4π/n) is the lattice spacing (a sample's area is h²)
and κ the outline's geodesic curvature. Its area loss is that drift integrated along the outline.
For a closed convex outline the curvature integrates to 2π (Gauss-Bonnet, less the enclosed area
on the unit sphere, which is small here), so a small round plate loses the same number of
samples per resampling whatever its radius, about 2πc. The test
`a_small_round_plate_through_resamplings` measures it on a static planet, resampled 40 times,
the loss read over the first 20 (the plate of radius 3 is gone by the 40th):

| radius, spacings | samples at the start | off: after 1, 10, 20, 40 | off: lost per resampling | on: after 1, 10, 20, 40 | on: lost per resampling |
|---|---|---|---|---|---|
| 3 | 28 | 28, 32, 33, 45 | -0.25 | 28, 21, 12, 0 | 0.80 |
| 6 | 113 | 116, 107, 106, 87 | 0.35 | 114, 109, 98, 81 | 0.75 |
| 12 | 452 | 446, 445, 440, 438 | 0.60 | 447, 442, 435, 422 | 0.85 |

- **The loss does not depend on the radius.** 0.75 to 0.85 samples per resampling at radii of 3
  to 12 spacings, so c is about 0.13. Without the level set the count wanders instead, by +5, -7
  and -12 samples after 20 resamplings.
- **What it costs a plate.** The floor of plates (PLT-091) is 0.003 of the samples, 300 on the
  100,000-sample planet, a radius of about 9.8 spacings. A plate at the floor, round and alone,
  would lose about 40 samples in the 50 resamplings of a 1000-Myr run (13 %); a larger plate
  loses the same 40, a smaller fraction. A microplate of 28 samples is gone in 35 resamplings,
  but it is under the floor and absorbed long before.
- **In a mosaic.** A plate's outline is not closed and convex: its arcs meet its neighbours' at
  triple junctions, and the drift moves each arc towards its own centre of curvature, so one
  plate's loss is its neighbour's gain. For a curvature flow in a plane foam this is von
  Neumann's law (from memory, not read: a cell's area changes in proportion to its number of
  sides less six), so plates with few neighbours shrink and plates with many grow, at a rate of
  the order of a sample per resampling.
- **The test asserts it.** The loss with the level set must lie between 0.4 and 1.4 samples per
  resampling at each radius, and the largest within twice the smallest. If it grows, a
  second-order interpolation (each sample's distance gradient stored and the distance
  extrapolated from each corner, s ≈ Σ w_k [s_k + ½ ∇s_k · (q - p_k)]) would cut the drift to
  the order of h³; it is not built.

**Arm fl-ls** (the level set on top of fl-fission2, 8 runs, `earthlike_quick_16k`, 1000 Myr, key
85a1e1cdbf45, argkey 029ca01a; with the level set off the step takes fl-fission2's code path, so
fl-fission2 is its control). Against fl-fission2, medians with the Mann-Whitney p:

- **The guard.** Its worst is 10,530 m against 10,490 (p 0.51). No step goes beyond 2 reliefs,
  and `guard_bind_steps` is 0 in every run.
- **The V1 census.** It meets its 5 criteria in both arms (land 0.320, p 0.72).
- **The boundary.** It is 323,059 lattice km against 352,268, and 326,556 straight km per step
  against 353,140 (p 0.002; Bird 260,487). E, the perimeters over their caps', is 1.929 against
  2.013 (p < 0.001; Bird 1.510). The subduction front is 91,750 straight km per step against
  98,984 (p 0.005; Bird 51,310), and the front in collision 21,038 km against 24,217 (p 0.005).
  With each plate one body (`boundgeom`, 77 km), the front's tortuosity is 1.925 against 2.026
  (PB2002 1.20 on the lattice).
- **The plates.** 27.1 alive per step against 27.8 (p 0.38). Over the run they go from 23.2 in
  the first window to 31.4 in the last, against 24.9 to 27.6, so the boundary grows too (290,272
  to 371,939 km against 309,152 to 352,067).
- **Fission's churn.** Fissions per run 65 against 64.5. Fission-born plates live 20 / 36 / 72.5
  Myr (p25 / p50 / p75) against 20 / 40 / 80; the other plates 22 / 76 / 144 against
  20 / 74 / 152.
- **The sutures.** 40.9 per 100 Myr against 42.4, and 37,228 km per 100 Myr against 41,589
  (p 0.038; Macdonald 23,235 to 50,919).
- **The plate check.** It passes 191 measures against 187.

So the level set shortens the boundary and the front by 7 to 8 % and the front's tortuosity by
5 %. That is far less than the test's static boundary gains without it (twice its length in 40
resamplings). In the runs the ridges are drawn again by the gap fill at each resampling either
way, and the trenches are moved every step by the overlaps' hiding and the collisions (below),
which the level set does not touch. The front is still 1.8 times Bird's, and its tortuosity 1.6
times PB2002's.

**What still cuts the plates, with the level set.** `where_the_plates_are_cut` (40,000 samples,
300 Myr, fission on; samples cut off their plate's largest piece, summed over the run):

| part of the step | supercontinent 1, off | supercontinent 1, on | scattered 2, off | scattered 2, on |
|---|---|---|---|---|
| overlaps resolved (`subduction::resolve`): cut / joined | 5,304 / 1,771 | 8,800 / 2,751 | 7,379 / 2,772 | 10,207 / 2,328 |
| the collision's terrane transfer: cut (the overriding plate, the subducting one) | 5,038 (3,016, 2,022) | 2,734 (1,817, 917) | 3,660 (2,045, 1,615) | 2,556 (1,748, 808) |
| subduction initiation: cut | 3 | 256 | 74 | 39 |
| the resampling: cut / joined | 0 / 8,299 | 0 / 8,866 | 0 / 8,209 | 0 / 9,215 |
| fissions | 14 | 18 | 16 | 18 |

- **The resampling cuts nothing either way.** With fission each cut piece becomes a plate or is
  captured at the resampling, which books it as joined.
- **The overlaps' hiding is the main cut, and more of it with the level set.** Hiding the
  overridden samples cuts a plate's neck that the next step's motion opens again. The terrane
  transfers cut less with the level set. Each case is one run of a chaotic planet, so these
  moves are readings, not a law.

**Arm fl-ls2** (the level set with the latent slab copies of Step 3b, on top of fl-fission2, 8
runs; key dce14248713c, argkey 029ca01a, `--set tectonics.level_set_membership=true`). Against
fl-fission2, medians with the Mann-Whitney p:

- **The guard.** Its worst is 10,240 m against 10,490 (p 0.16). No step goes beyond 2 reliefs,
  and `guard_bind_steps` is 0 in every run.
- **The V1 census.** It meets its 5 criteria (land 0.304).
- **The boundary.** It is 326,139 lattice km against 352,268, and 327,874 straight km per step
  (p 0.002). E is 1.908 against 2.013 (p < 0.001). The front is 89,331 straight km per step
  (p 0.002), and the front in collision 19,668 km against 24,217 (p 0.002). With each plate one
  body (77 km) the front's tortuosity is 1.924 against 2.026; its length there is the same
  (115,979 km against 115,544).
- **The plates.** 27.5 alive per step against 27.8 (p 0.72), from 21.9 in the first window to
  25.9 in the last (against 24.9 to 27.6), 25.5 at the end against 27.
- **Fission's churn.** Fissions per run 77 against 64.5 (p about 0.23). Fission-born plates live
  20 / 40 / 72.5 Myr (p25 / p50 / p75) against 20 / 40 / 80; the other plates 18 / 72 / 154
  against 20 / 74 / 152. Per run the splits moved 215 to 278 slabs and copied 284 to 342, moved
  245 to 316 contacts and copied 287 to 344, and moved 33 to 57 collisions.
- **The sutures.** 40.6 per 100 Myr against 42.4, and 37,382 km per 100 Myr against 41,589
  (p 0.050).
- **The plate check.** It passes 189 measures against 187.

Against fl-ls (the level set without the latent copies) no measure differs at p < 0.05, so the
latent copies of Step 3b change the planet little, as their 1.3 to 3.2 % of the slab force
foretold.

**On by default.** `tectonics.level_set_membership` is on (2026-10-24), with fission and the
split inheritance. Its unit tests pass against their source (the great circle exact, the
reinitialisation keeping it, the boundary no longer roughened by the resampling), both arms on
the pool show no divergence and no guard binding, the census keeps its 5 criteria, and the plate
check passes 189 to 191 measures against 187. On the 8k test planet the guard binds on 3 of 24
seeds with it (Step 3b, "The speed guard"), small plates held by a fixed suction, the same kind
as without it; that is the suction law's known limit, not the level set's. What it does not do:
the front stays 1.7 times Bird's (89,331 against 51,310 straight km), since the trenches are
moved every step by the overlaps' hiding and the collisions, which it does not touch.

### The overlaps' hiding: what it hides and why it cuts (2026-10-24)

The coordinator's item, before anything more is built: what the overlaps' hiding in
`subduction::resolve` hides, why it cuts the plates, and what the physics should do instead, from
fl-ls2's 8 cached runs only (no simulation). The measure is `hiding` in
`tools/review/pool/measures_front.json` (`measure_hiding.py`): the crust budget's maps every 50
Myr after the first 100, without those taken at a resampling (13 to 16 a run), and the runs'
records. Ranges below are over the 8 runs, each a mean over its snapshots.

**What it hides.** Each step `resolve` clears every sample's flags and hides each sample of a
contact's subducting plate that lies inside one of the overriding plate's triangles
(`geometry::cover`, a group's own triangles), if the contact is seen that step (a new pair needs
`MIN_CONTACT`, 3 such overlaps; a known pair needs one). Nothing carries over from the step
before: a sample is hidden exactly while it lies under the overriding plate's cover. The
resampling consumes what is hidden at that moment (its pools go to the crust on top: PLT-276,
PLT-292, PLT-440). In the records the step is 2 Myr and a resampling comes every 16 to 17 Myr (58
to 63 a run: the schedule's 20 Myr, PLT-098, and one after each weld). Between resamplings 5.1 to
6.4 % of the samples are hidden on average, and the planet's visible count never rises from one
step to the next (0 steps in any run).

**What it cuts.**

- 0.74 to 1.72 % of the visible samples lie in pieces cut off their plate's largest, 84 to 144
  pieces a snapshot, the median piece one sample.
- 69 to 80 % of the cut samples are in pieces beside their own hidden samples (the TRENCH flag),
  and those pieces' trench edges face the overriding plate's subduction zone (within 800 km of
  its front) as often as the main bodies' trenches do: 0.87 to 0.90 of the edges for the small
  pieces and 0.73 to 0.98 for those of the fission floor or more (PLT-091, 300 samples), against
  0.95. The share whose other side moves away (DIVERGENT) is the main bodies' too (0.42 to 0.51
  against 0.43 to 0.47). The hiding cuts at ordinary trenches.
- By shape, shares of the cut samples: a thin hidden band, at most 3 spacings from the plate's
  largest piece, 0.28 to 0.44 (necks); farther, with one plate between, 0.17 to 0.32; farther,
  across two plates or more, 0.07 to 0.14; enclosed by one plate, 1 to 3 samples 0.08 to 0.21
  (holes), more 0.11 to 0.21 (islands).
- Pieces of the fission floor or more: 0.33 to 1.08 a snapshot, 14 to 36 % of the cut samples, 75
  to 100 % of them beside their own slab, all necks or with one plate between (two in a run's
  snapshots across two plates).
- Seams: 7.5 to 17 % of the enclosed pieces touch a seam between two groups of the plate around
  them, about ten times chance (0.17 to 0.30 % of the visible samples are seam samples, about 1 %
  of a piece's six neighbours by chance). A seam's triangles belong to no group, so they hide
  nothing. This is 3 to 7 % of the cut samples.

**What the pieces cost.** `boundgeom` over the same runs (Step 3c, at 77 km): the subduction front
is 139,771 km with the pieces and 115,979 km with each piece given to the plate around it, the
boundary 353,285 and 325,644 km. The pieces carry 17 % of the front and 8 % of the boundary. On
the lattice the front is 2.66 times PB2002's (52,564 km) with them and 2.21 times without, so the
main bodies' outline at their trenches holds most of the excess.

**What becomes of them.** Fission makes 55 to 86 plates a run of them (median 77). They live a
median 26 to 40 Myr, about two resamplings, and of those absorbed only 2 to 9 % go back into their
parent; the rest join a neighbour.

**Why it cuts.** Three ways, from the code:

1. *A neck under the overriding plate.* Where the overriding plate's cover reaches across a narrow
   part of the subducting plate, from one of its boundaries to another, the visible crust is in
   two. The plates' outlines are ragged at 1 to 3 spacings (the front's tortuosity 1.92 at 77 km
   with each plate one body, PB2002's 1.20 on the lattice), so such necks are common and a band
   that thin hides them. This is the measured majority: the necks and most pieces with one plate
   between, all at ordinary trenches.
2. *Crust that went under shows again.* The flag is set again every step, so a sample shows again
   whenever it leaves the overriding plate's triangles before the next resampling: behind a
   corner of the overriding plate that has swept past it, beyond an overriding plate narrower
   than the slab's travel since the resampling (at 50 mm/yr, 830 km or 12 spacings in 16.7 Myr),
   in a step the contact is not seen, and when the contact reverses (`reverse` swaps the plates:
   the old slab shows at once and the old overriding plate's crust on it is hidden). The cut
   trace books this among the overlaps' joins (1,771 to 2,772 samples per 300 Myr at 40,000
   samples, against 5,304 to 10,207 cut, Step 3c's table). The records cannot size it: their
   counts are net, and what a plate hides at one edge hides what it shows at another. What shows
   again is old crust that went down a trench, at a place where the ground has opened.
3. *Seam holes* (above), 3 to 7 % of the cut samples.

**What Earth does.** Subduction is one way: the lithosphere that goes down a trench is the slab
whose weight is the slab pull the force balance applies (`forces.rs`, Forsyth & Uyeda 1975), and
none of it comes back to the surface. Where an overriding plate moves away from the trench, or
past a stretch of it, the ground between opens and takes new crust (back-arc spreading; from
memory, no source read for this page). A subducting plate cut through by a trench is torn, and
its pieces move on their own or join a neighbour (Stock & Lee 1994, "What Earth does" above), as
fission and capture already do.

**What the physics should do.**

- *One-way subduction* (for way 2). A sample once hidden under a contact stays hidden until the
  resampling consumes it, whatever the overriding plate does next, and holds no ground at the
  resampling: the ground it leaves is a gap and gets new crust from the gap fill (PLT-502). It
  is not built. It would not shorten the front: it keeps consumed necks consumed instead of
  letting them show again, so it may cut a little more. What it corrects is the crust behind
  narrow overriding plates and after reversals, which would be new instead of recycled. To size
  it first: a counter of the samples shown again per step, by cause (left the cover, contact not
  seen, reversal), on one arm.
- *Necks* (way 1) are physical: a neck consumed whole is a tear, and fission or capture follows.
  Their number follows the plates' raggedness at their trenches, the 2.21 times above, which is
  the next thing to diagnose: what roughens the overriding plates' outlines between resamplings
  (the arcs' growth, the terranes, the collisions' fronts).
- *Seams* (way 3) are geometry, not physics: a triangle whose corners are all one plate's, in
  two of its groups, should cover for that plate. Minor.

So the hiding is not more urgent than the saturating suction (Step 3b, "The speed guard"): its
pieces carry 17 % of the front, one-way subduction would not shorten it, and the guard's binds on
the 8k planet are the open fault in the physics. Next: step 4 (T and S1 on base4), then the
saturating suction, then one-way subduction with its counter.

### Step 4: T and S1 on the new planet (arm fl-ts1, 2026-10-24)

The front lane's step 4: T (transference) and S1 (the trench's retreat, PLT-522, PLT-523) together
as one cumulative arm on master b75d0b7f's default, which has fission, the latent copies and the
level set on (`pool.sh run fl-ts1 --length 1000 --seeds 1,2,3,4 --threads 2 --set
tectonics.transference=true --set tectonics.erosion_retreat=true +DEEPTIME_CRUST_BUDGET=1
+DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`, key e5cf2bf81f88, argkey 482ecb22, 8 runs, both starts).
It is read against base4, master's 24-run base (argkey 2a53566b), and against base4-14, an arm
file holding base4's own seeds 1 to 4 (the same runs, no new simulation). Medians, base4-14 to
fl-ts1, with Mann-Whitney p:

- **The V1 census.** Land 0.304 to 0.151 (p 0.003), the largest landmass 0.180 to 0.026,
  continental crust under the sea 0.220 to 0.733, land above 2 km 0.146 to 0.038, coast with high
  ground 0.278 to 0.102, ice 0.093 to 0.107, collisions per 100 Myr 1.30 to 4.85, criteria met 5
  to 4. The continental crust's net change over the run goes from -0.029 to +0.136 of the planet
  (0.4225 at the start, 0.393 to 0.558 at the end), and the sea stands 1,659 m over the datum at
  the end (723).
- **The area book** (`build/dock/areabook.py`'s parts, shares of the planet over the run): the
  fronts' hiding (`subduction`) -0.935 to -0.408, the collapse +0.423 to +0.151, the arcs'
  maturing (`finish`) +0.390 to +0.361, `founder` +0.148 to +0.059, the resampling's blend
  -0.169 to -0.094, the retreat -0.013. T hands the subducting plate's continents to the
  overriding plate instead of letting the front consume them, so the fronts take less than half
  as much continental crust, while the arcs' maturing barely moves.
- **The guard.** No step beyond 2 Earth reliefs in any run. The worst is 12,807 m from the sea
  against 10,240 (the sea 936 m higher). `guard_bind_steps` is 4 in one run (supercontinent-3);
  base4 has 4 in one of its 24 (supercontinent-8).
- **The plate check** passes 189 measures against 189. In fewer runs of 8: the arcs' continental
  heights (p50 8 to 0, 1,476 to 632 m: drowned; p90 8 to 3), the highest continental share (6 to
  0, 0.441 to 0.558), the largest land gain (6 to 0), one plate above half (8 to 3, 12 % of the
  steps), the plates above 5 % (8 to 4), the largest plate's share (8 to 6), the rotation change
  (8 to 4), the speed-area rank (7 to 3), the collisions' convergence ratio (2 to 0). In more:
  the collisions per 100 Myr (1 to 6), their lengths (p50 1 to 8, p90 0 to 8), the lowest
  continental share (2 to 8), the oceanic over continental speed (2 to 8), the slab-pulled over
  free speed (1 to 6), rifts alive and not welded back after 50 Myr (3 to 7, 4 to 7), the
  supercontinent's tenure (0 to 2).
- **The front.** It does not get shorter. The straight front per step is 89,331 to 99,333 km (p
  0.002; Bird's SUB 51,310), E 1.91 to 2.17. In `boundgeom` at 77 km against fl-ls2 (master's
  maps): 139,771 to 131,265 km with the cut pieces, 115,979 to 100,151 with each plate one body;
  under oceanic overriding crust 66,822 to 60,592 km one body, against Stern & Scholl's 14,500 km
  of island arc.
- **Sutures** 40.6 to 47.2 per 100 Myr (Macdonald's 19 to 39), 37,382 to 44,960 km per 100 Myr
  (inside Macdonald's 23,235 to 50,919).

**The verdict: it does not pass.** By this lane's reading (no judged measure passes in fewer runs,
one run of noise allowed, and no guard step), ten measures fall by more than one run and one run
binds the guard. The coordinator's ruling makes the plate check a realism check, not a gate, but
the census shows the planet drowned: the continental crust grows to 0.558 of the planet and the
sea rises over it. This is dock-TS1's failure on base3 again ("So S1 does not make T and the sink
correct on the pool", above). What is still off is the same: under T the fronts no longer consume
continental crust at a collision, and nothing else consumes it in their place. The arcs' maturing
stays on an island-arc front four times Stern & Scholl's, and step 3 left that front's length
where it was. T and S1 stay off by default, both built.

What could close the book, from what has been measured, without a knob:

1. The island-arc front's length (the main bodies' outline at their trenches, 2.21 times PB2002 at
   77 km: "The overlaps' hiding" above). On Earth's length the maturing would make about a third
   of what it makes now (0.09 km2 a year against 0.26, "What comes after S1" above).
2. T's own consumption: after T moves the subducting plate's continent onto the overriding plate,
   nothing shortens it, where India has gone on converging with Asia for 50 Myr (Stage 2's first
   candidate, above).

Neither is built in this lane. The saturating suction comes next.

### Step 5: the saturating suction (PLT-540, 2026-10-24)

The speed guard's binds on the 8k test planet ("The speed guard" in Step 3b) are all small plates
with other plates subducting under them along much of their edge, drawn by a suction that was a
fixed 0.3 of the slab pull (PLT-186) however fast the plate already moved. This step makes the
suction saturate (`tectonics.suction_saturates`, `forces::Balance::add_suction`).

**The law.** The suction is the traction on the overriding plate of the mantle flow its sinking
slab drives (Conrad & Lithgow-Bertelloni 2002, PLT-186). A viscous traction grows with the
difference between the flow's speed and the plate's, and the flow's speed at the trench is the
slab's sinking speed, so the overriding plate is drawn at most as fast as the slab sinks (the
premise is from memory; no source was read for it). At each front sample the suction becomes
`share × pull × (1 − u / v_t)`: `u` the overriding plate's speed along the slab's direction across
the trench, `v_t = pull / (slab_res ℓ)` that slab's terminal velocity, the speed at which its pull
and its slab resistance balance (PLT-185; Forsyth & Uyeda 1975, p. 178: plates attached to slabs
move at "effectively the terminal velocity of the dense slab falling in a viscous medium"). At
rest it is the old share; at the slab's terminal velocity it is nothing; faster, it resists, the
plate then pushing the slab back through the mantle. In the balance this is the old force plus a
link from the plate to the mantle along that direction with `k = share × slab_res × ℓ`, so the law
adds no constant: its share is PLT-186's and its speed PLT-185's. With Earth's settings `v_t` is
105 mm/yr for an old slab at full length (45 × (1 + 3 + 3) / 3), less for a young or short one.
For a straight trench and a plate in translation, a plate the fixed share draws at `u_f` now moves
at `u_f / (1 + u_f / v_t)`: a slow plate keeps most of its speed and a fast one is held below
`v_t` whatever its size. A hanging slab (PLT-271) sucks the same way.

How to read the speed-dependent part. In a linear viscous mantle the traction on a plate is the
traction of the slab's flow with the plate held still (the fixed share) plus the drag of the
plate's own motion, and that drag is already the basal drag. What the link adds is something
else: an overriding plate that advances on its trench, with no back-arc opening behind it, makes
the trench retreat as fast, so the slab's hinge must move back through the mantle, and a slab
cannot roll back faster than it sinks. The link is that resistance, sized so that it cancels the
suction at the slab's terminal velocity (from memory; no source on rollback was read for it).

The speed that saturates it is the slab's free sinking speed, not the speed at which the trench
actually consumes the subducting plate. Coupling the two plates through the trench (the
subduction rate as the subducting plate's advance plus the overriding plate's) would make the
suction depend on the subducting plate's speed as well, and would load the subducting plate with
the overriding plate's resistance, which Earth's fast subducting plates do not show (PLT-186's
four times). It is not built.

**The unit test** (`the_suction_draws_no_overriding_plate_faster_than_its_slab_sinks`,
`forces.rs`). A cap drawn only by a straight trench's suction: at rest the suction's torque is the
share's to 1e-9, and the plate moves at `u_f / (1 + u_f / v_t)` within the test's tolerance (the
trench is a few rim points, so the plate turns a little as well as translates). A cap of 3°
(about 70 samples of the 100k lattice, the size the guard held) with a trench along a third of its
edge and nothing else at its edges: 2,060 mm/yr with the fixed share, 99.8 saturating (expected
99.9; `v_t` 105). A cap of 31° with boundary resistance all round and a 3,000 km trench: 27.8 and
22.1 (expected 22.0).

**The guard sweep** (the 8k test planet, scattered continents, 300 Myr, seeds 1 to 24, master's
default with the fixed suction and with the saturating one; the ignored test of Step 3b, appended
on the VM only):

| | fixed | saturating |
|---|---|---|
| seeds on which the guard binds | 3 (2 for 7 plate-steps, 17 for 1, 24 for 7) | 0 |
| top plate speed, range over the seeds | 135 to 210 mm/yr | 127 to 161 mm/yr |
| plate speed p50, median over the seeds | 48.3 mm/yr | 37.3 mm/yr |
| plate speed p90, median over the seeds | 97.5 mm/yr | 89.3 mm/yr |
| seeds with a plate over 200 mm/yr | 2 | 0 |
| seeds that weld | 12 | 11 |

No small plate reaches the guard any more, and the plates as a whole move about a fifth slower
(the speeds are each plate's on each step, all plates alike). The weld tests' seeds still weld:
the collisions test's 12 and the recording test's 19 weld with both laws (with the saturating
suction 1, 2, 4, 8, 10, 12, 18, 19 and 22 to 24 weld).

**The arm** (`pool.sh run fl-sat --length 1000 --seeds 1,2,3,4 --threads 2 --set
tectonics.suction_saturates=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`,
key 4e350ff878b3, argkey e4cd5c3b, 8 runs, both starts; master 43d96c7c's simulation with the
switch set). Read against base4-14 (base4's own seeds 1 to 4) and base4's 24; medians, base4-14
to fl-sat, Mann-Whitney p in brackets:

- **The V1 census.** Land 0.304 to 0.315 (p 0.96), the largest landmass 0.180 to 0.220,
  continental crust under the sea 0.220 to 0.263, land above 2 km 0.146 to 0.113, coast with high
  ground 0.278 to 0.224, ice 0.093 to 0.104, collisions per 100 Myr 1.30 to 1.40, criteria met 5
  to 5. The continental crust's net change over the run is -0.029 to -0.017 of the planet, and
  the sea ends 961 m over the datum (723). Against base4's 24 nothing moves at p < 0.05 (the
  collisions 1.15 to 1.40, p 0.052).
- **The guard.** No step beyond 2 Earth reliefs; the worst is 10,590 m from the sea against
  10,240 (p 0.10). `guard_bind_steps` is 0 in all 8 runs (base4: 4 in one run of 24).
- **The area book** (shares of the planet over the run): the fronts' hiding -0.935 to -0.732, the
  collapse +0.423 to +0.319, the arcs' maturing +0.390 to +0.315, `founder` +0.148 to +0.122, the
  resampling's blend -0.169 to -0.156. Slower plates turn the crust over less, in step.
- **The plate check** passes 175 measures against 189. In fewer runs of 8: the large plates'
  median speed (8 to 0, 35.9 to 25.3 mm/yr against the band's 30 to 60) and their p90 (8 to 4,
  73.4 to 70.1), the ocean's p90 age (8 to 2, 201.5 to 247.1 Myr against at most 230) and its
  median (8 to 7, 66.6 to 76.2), the ocean over 180 and over 250 Myr (1 to 0 each), the largest
  land gain (6 to 3), the highest continental share (6 to 4), the speed-area rank (7 to 5), the
  rifts welded back (4 to 1), the arcs' continental p50 (8 to 6), the collisions' convergence
  ratio (2 to 1). In more: the slab-pulled over free speed (1 to 6, 2.05 to 3.01; PLT-186's
  sources say subducting plates move about four times faster), the oceanic over continental speed
  (2 to 6, 1.83 to 2.30), the lowest continental share (2 to 5), the active continental margin (0
  to 3, 0.449 to 0.413), the supercontinent's tenure (0 to 3), the collisions per 100 Myr (1 to
  2), their p90 length (0 to 2).
- **The boundary.** The subducting share 0.309 to 0.292 (p 0.015; Bird 0.197), the mean
  convergence across the oceanic front 52.1 to 44.3 mm/yr (p < 0.001; Bird 53.2), the boundary
  criteria met per run 2.62 to 3.25 on average (p 0.034).
- **The front.** 89,331 to 86,745 straight km per step (p 0.28; against base4's 24, 91,083, p
  0.046), E 1.908 to 1.876; the initiations' plates' E 1.902 to 1.811 (p 0.005). In `boundgeom`
  against fl-ls2, the front under oceanic overriding crust with each plate one body stays at
  66,822 to 67,813 km.
- **Sutures** 40.6 to 36.5 per 100 Myr (inside Macdonald's 19 to 39 now), 37,382 to 32,666 km.
  Fissions per run 77 to 58.5; plates per step 27.5 to 28.4.

**On by default** (2026-10-24). The law passes its unit test against its source, the pool shows
no divergence and no guard binding, and the 8k planet's binds are gone (3 seeds of 24 to none).
The suction of a fixed share goes off with it. What it costs is in the plate check's speeds: the
plates the fixed share drew towards their trenches, mostly the large overriding ones, now move
more slowly, the large plates' median at 25 mm/yr under the band's 30, and with slower spreading
the ocean grows older (its p90 age 247 Myr against at most 230). Subducting plates now outrun
the others three times over, nearer the four times PLT-186's sources give. These are realism
checks, not a gate (decisions log 2026-10-23), and nothing is tuned for them. One thing is open:
PLT-186's share is the open-loop test's 0.3, set against speed targets with the fixed law, where
Conrad & Lithgow-Bertelloni's suction over pull is 0.43 to 0.67 (their suction acting on both
plates, so not the same quantity). That is a question about PLT-186, not this law. The weld
tests' seeds still weld (12 and 19). One test's planet is held on the fixed suction: the epochs
nudge test (`the_nudges_reach_their_targets_gently`) is about the climate's nudges, and with the
saturating suction its steered planet warms past the band's top in its last step and the
outgassing turns back once.

### Step 6: the crust shown again and one-way subduction (PLT-542, 2026-10-24)

"The overlaps' hiding" (above) asked for a count before the law: how much of the crust that goes
down a trench shows again, and why.

**The counter** (`shown.rs`, measurement only, kept beside the dynamics record; the pool measure
`shown` in `tools/review/pool/measures_front.json`). Each step it counts the samples that went
under (visible as `resolve` began, hidden after it) and those shown again (hidden before,
visible after, or brought up by a collision), by the first cause that holds: a rift moved the
sample to another plate (`plate`), the contact reversed this step (`reversal`), its plate now
overrides the plate it lies under (`on_top`), no contact was seen (`unseen`), it lies under no
plate's cover (`left`), a collision brought it up (`collision`). At each resampling it counts the
hidden samples consumed and the samples that went under since the last resampling and stand
visible (escaped), by the cause of their last showing, with the ages of the ocean floor among
them. On arm fl-shown (master 25941ca5's simulation with the counter, key 601e007feb64, argkey
2a53566b, `earthlike_quick_16k`, 1000 Myr, both starts, seeds 1 to 4, 8 runs; its supercontinent
runs' state hashes equal base5's, so the counter changes nothing), medians of the 8 runs:

- 3.34 km² a year went under (Earth-normalised) and the resamplings consumed 2.55.
- 0.230 of what went under showed again: 0.199 because the overriding plate moved off it
  (`left`), 0.025 under a plate its own now overrides (`on_top`), 0.004 at reversals, 0.002 by
  collisions, 0.001 by rifts, nothing for want of a contact.
- 0.167 of what went under since the last resampling stood visible at the next one (escaped):
  0.144 `left`, 0.016 `on_top`, the rest under 0.004 each. That is 0.017 of the planet a
  resampling, 16 % of it continental.
- The escaped ocean floor was 99.5 Myr old on average, the consumed ocean floor 104.7, the
  visible ocean floor 115.3 at the resamplings (112.6 with the escaped counted as new).

**Why it matters.** Crust that goes down a trench is the slab, and its weight is the slab pull
the force balance applies (`forces.rs`, Forsyth & Uyeda 1975); none of it comes back to the
surface. Here a sixth of it came back as old ocean floor where the overriding plate had moved
off. In a steady state the ocean floor's mean age is its area over the rate new floor is made,
and that rate is the rate floor is consumed; consuming the escaped ocean floor too (2.55 km² a
year over 1 less 0.167, less its continental share) would raise it to about 2.98 to 3.06 km² a
year and bring the mean age from 115 to about 96 to 99 Myr. The plate check's ocean p90 age was
247 Myr against at most 230 on fl-sat (Step 5). So the counter says the law matters.

**The law** (`tectonics.one_way_subduction`; `subduction::resolve`, `ridges::resample`; ledger
PLT-542). Ocean floor hidden under an overriding plate's forearc is held: it stays hidden each
step, whatever that plate does next, until the next resampling. The forearc is the overriding
plate's visible crust within the arc's distance of a subduction front (`arc_distance_km`,
PLT-042, 200 km) plus one lattice spacing for where the front falls, as PLT-503's reach, from
the fronts the last step left (`fronts_finish`); a sample counts as hidden under it when the
cover's major corner is there. Gating by the forearc keeps the law to trenches: a contact forms
for any overlap, with no test of convergence, and the subduction zone flag reaches 1,800 km, too
far. Continental crust is never held (it is too buoyant to stay down, Cloos 1993, PLT-047: it
collides, or shows again where the overriding plate leaves it, as before), held crust that a
collision brings up or a rule turns continental is let go, and the held set is cleared at each
resampling (so the first step after one holds nothing new). At the resampling held crust is
never on top of the new lattice, and a point that only held crust covers is a gap and gets new
sea floor from the gap fill (PLT-502): the ground the overriding plate left opens as new crust,
as a back-arc opens. The held crust's conserved pools at such points go down with the slab
(`plates/resample_slab_km3`, its sediment counted in `plates/sediment_subducted_km3` too).

**The unit test** (`one_way_subduction_keeps_the_slab_down`): an ocean plate running under a
continent on a 20,000-sample planet, with 200 m of sediment on its floor. After the fronts are
found, a step hides the ocean's leading edge under the continent (249 samples); then the ocean
plate is turned back to where it was before it converged, out from under the continent, and the
planet is resampled. Off, all of it shows again and the resampling makes no new floor. On, the
207 samples under the forearc stay hidden (the 42 beyond its reach show again), the resampling
makes 205 samples of young sea floor where they were, and the slab global is their sediment, 205
× 200 m × the sample area, within 5 %. With no fronts found first, or with continental crust in
the overlap, nothing is held.

**The arm** (`pool.sh run fl-oneway --length 1000 --seeds 1,2,3,4 --threads 2 --set
tectonics.one_way_subduction=true +DEEPTIME_CRUST_BUDGET=1`, key d2580cce8210, argkey f2aaae39,
8 runs, both starts; master 25941ca5's simulation with the switch set). Read against fl-shown
(the same simulation off, the same seeds and starts), medians, Mann-Whitney p in brackets:

- **The counter.** 3.06 km² a year went under (fewer: less crust goes down twice) and the
  resamplings consumed 2.92, near the estimate. 0.050 of what went under showed again (0.230),
  0.038 of it `left` (crust hidden beyond the forearc's reach, and continental crust); 0.033
  escaped the resamplings (0.167), now 56 % continental. The visible ocean floor's mean age at
  the resamplings is 105.7 Myr (115.3).
- **The plate check** passes 175 measures, as fl-shown. The ocean's p90 age is 214.2 Myr
  against 247.1 (7 runs of 8 at most 230, against 2; base5's 24: 219.4) and its median 73.2
  against 76.2 (8 against 7). In fewer runs: the lowest continental share (1 against 5, 0.345
  against 0.377), the oceanic over continental speed (4 against 6, 2.00 against 2.30), the
  large plates' p90 speed (1 against 4), the supercontinent's tenure (1 against 3), the
  collisions per 100 Myr (0 against 2, 1.50 against 1.40) and their lengths. In more: the largest
  land gain (5 against 3), the highest continental share (5 against 4), the collisions'
  convergence ratio (3 against 1).
- **The V1 census.** Land 0.315 to 0.287 (p 0.38), the largest landmass 0.220 to 0.131 (p
  0.028), continental crust under the sea 0.263 to 0.301 (p 0.80), land above 2 km 0.113 to
  0.125, coast with high ground 0.224 to 0.242, ice 0.104 to 0.111, collisions per 100 Myr 1.40
  to 1.50, criteria met 5 to 5. The continental crust's net change over the run is -0.017 to
  -0.049 of the planet (p 0.038; against base5's 24, -0.016, p 0.004), and the sea ends 693 m
  over the datum (961).
- **The guard.** No step beyond 2 Earth reliefs; the worst 10,157 m from the sea against 10,590
  (p 0.23). `guard_bind_steps` is 0 in all 8 runs.
- **The area book** (shares of the planet over the run). The continental crust ends at 0.374
  against 0.405, its volume 6.88 against 7.90 (Earth-normalised units of the book). The parts
  that moved: the oceanic arcs' maturing (`finish`) +0.315 to +0.294, the thickness limits'
  spreading (`rate`) +0.088 to +0.069, the fronts' hiding -0.732 to -0.751, the resampling's
  blend -0.156 to -0.132.
- **The boundary and the front.** The ocean-ocean front 37,624 to 34,707 straight km a step (p
  0.007), the subduction front 86,745 to 82,390 (p 0.19), the boundary 325,739 to 305,653 (p
  0.065); plates alive per step 28.4 to 26.8 (p 0.040), E 1.876 to 1.796 (p 0.038). Polarity
  reversals 11.6 to 9.1 per 100 Myr (p 0.051), initiations 11.8 to 10.8. Sutures 36.5 to 31.8
  per 100 Myr, 32,666 to 28,352 km (p 0.038). The oceanic arcs' p90 height 752 to 606 m.

What the arm says. The law does what it is for: the crust shown again falls to a fifth, the
resamplings consume what went under, and the ocean floor is younger, by about the steady-state
estimate. The plates are fewer and their outlines straighter. What it costs is continental
crust: about 3 % of the planet less over 1000 Myr, the largest landmass smaller. The area book
puts most of it in the crust the fronts make: the island arcs mature less (the ocean-ocean
front is 8 % shorter, the oceanic arcs lower) and less thick crust spreads onto ocean floor. A
reading, not a measured cause: before, about a quarter of the crust that went down a trench
came back, and some of it went down again (what went under fell by 8 % with the law), which
kept ocean-ocean fronts and their arcs working on recycled floor; now the ground behind an
overriding plate that moves off opens as new floor instead. The largest landmass is a chaotic number on 8 runs (base5's own seeds 1 to 4
give 0.281 against its 24 runs' 0.213).

One thing is left as it was: held crust still forms its plate's triangles in `detect`, so where a
contact has the slab's plate overriding another, crust of the other that comes over the slab
goes under it until the resampling, as hidden crust's triangles did before. It is not measured.

### Step 7: the subduction zone through a resampling (PLT-550) and the front measured like for like (2026-10-24)

After step 6 the pool measure `boundgeom` still put the subduction front at 77 km at about 2.2
times PB2002's. Before any law, the diagnosis read the cached maps of fl-ls2 and fl-sat (8 runs
each) and base6's records (seeds 1 to 4, both starts); no simulation ran for it. It found four
parts, and none of them is an outline rough at the plates' scale.

**1. The measure.** `boundgeom` averaged in the snapshots taken at a resampling, where nothing is
hidden, no sample has the TRENCH flag and so there is no front. Between resamplings the raw front
at 77 km is 2.76 times PB2002's (fl-ls2) and 2.53 (fl-sat).

**2. Two lattices at the front.** Put on one fixed lattice (each snapshot re-rastered onto a
100,000-point Fibonacci lattice, a point taking the commonest plate of its 7 nearest visible
samples, `k7`; PB2002's raster through the same step), the front is as rough as PB2002's at 300 km
and longer chords: its length at 300 km over its length at 1,200 km is 1.029 (fl-ls2) and 1.028
(fl-sat) against PB2002's 1.031. At 77 km it is 1.19 against 1.10, the lattice's staircase. The
ladder for fl-ls2, at 77 / 1,200 km against PB2002's SUB raster: raw 2.76 / 2.19, one lattice at
the same resolution (k1) 2.23 / 1.85, k7 1.98 / 1.83. Two plates' lattices meeting at the front
cost 1.24 times at 77 km and 1.19 at 1,200 km, single stray samples 1.13 at 77 km. The mechanism:
`geometry::cover` hides a subducting sample only inside a whole lattice triangle of the overriding
group, so the subducting plate's edge row reaches into the overriding plate's before it goes
under; 43 % of the front's TRENCH samples have another plate's sample within half a spacing.

**3. What counts as front.** The TRENCH flag marks a collision's front as well as a subduction
zone's (collisions are 18 % of the record's subduction plus collision length), while PB2002's
raster flagged SUB only. Bird 2003 classes a continental convergent boundary (CCB) apart from a
subduction zone (SUB): SUB steps are convergent steps "with a Benioff zone of intermediate to deep
earthquakes and/or a parallel volcanic arc with Quaternary activity" ([126]), classed first, and
a continental step is any that is not oceanic ([127]), so a CCB step is a convergent step on
continental or transitional lithosphere with no Benioff zone or arc; Table 3 gives SUB 51,310 km
and CCB 23,003. The model's boundary mix (`dynamics::classify_boundary`) counts a front wherever
an active contact converges, whatever the crust, and calls it a collision when the subducting
side is continental. So the like-for-like reference is SUB plus CCB, 74,313 km, not SUB alone.
This fixes the measure; it does not move a target. For context only, Mallard et al. 2016 give
48,000 km of trenches on Earth today and 66,000 on average in their model 2, whose plate sizes
they find Earth-like.

**4. The resampling's dip.** A resampling rebuilds every plate's lattice and leaves no overlap
anywhere, and a contact counted as a working subduction zone (fronts, arcs, uplift, slab pull, the
boundary mix) only at a step that saw its overlap. The overlap takes a few steps to form again,
so every resampling switched the subduction zones off. Base6, seeds 1 to 4, both starts, the
record's subducting length and the converging boundary with no contact
(`front_bands.other_converging`), km, by Myr since the last resampling:

| Myr since the resampling | subducting | converging, no contact |
|---|---|---|
| 0 (the resampling step) | 98,912 | 6,462 |
| 1 to 2 | 63,341 | 37,459 |
| 3 to 4 | 85,184 | 12,985 |
| 5 to 6 | 91,421 | 6,867 |
| 7 to 8 | 94,398 | 5,334 |
| 9 to 12 | 96,438 | 4,794 |
| 13 to 20 | 99,562 | 4,522 |

With a resampling every 16 Myr on average (median 20) the time-mean subducting length sits about
8 % under its steady state, and the measured front grows through each cycle (on k7 at 1,200 km,
fl-ls2 74,000 to 91,000 km, fl-sat 60,000 to 95,000, while the hidden share goes from 1.5 to 2 %
to 7.6 to 9.3 %). This one is a dynamic artefact, not a measuring one.

**The law** (`tectonics.contact_carry`, on by default; `Contact::carried` and `Contact::active` in
`state.rs`, `subduction::carry`, `resolve`, `fronts_begin`, `fronts_finish`; ledger PLT-550). A
lattice rebuild is a numerical step, not a geological event, and a subduction zone that worked
before it works after it. A contact active at the resampling step is carried: it counts as active
for its fronts, arcs and uplift, its slab pull (`forces.rs`), the boundary mix, rifting's touching
test and the record's live contacts, until its overlap is seen again (then it is active by that,
as before), or a step finds no front on its boundary (its plates no longer converge across it;
`fronts_finish` ends the carry), or `contact_timeout_myr` (PLT-059, 24 Myr since the overlap was
last seen) closes it. The fronts are still found each step by convergence across the boundary, so
a carried contact makes fronts only where its plates converge. Collisions are carried as
subduction zones are: the rebuild is no more an event for a collision, and the collision's own
clock (its onset, the belts) is untouched. What acts on the overlap itself still reads the step it
was last seen: the hiding, a collision's onset, the crust-shown counter (`shown.rs`) and the
timeout.

**The unit test** (`a_subduction_zone_goes_on_through_a_resampling`): the ocean plate of the
two-plate planet (20,000 samples) converging on the continent at 50 mm/yr has 136 front samples.
The planet is resampled and, before the plates move, the contact sees no overlap. Carried, it has
144 front samples; off, none. Turned to slide past each other, the plates have no front and the
carry ends; turned to converge further, the overlap is seen again and the contact is active by
it, carried no more.

**The measure, version 3** (`tools/review/pool/measure_boundgeom.py`, `measures_front.json`; the
maps' flags in `crust_budget.rs`). The crust maps now carry the boundary mix's class on the
overriding side of each front: MAP_FRONT_SUBDUCTION (256) where the subducting sample is oceanic,
MAP_FRONT_COLLISION (512) where it is continental, none at a resampling snapshot. `boundgeom`
skips the snapshots taken at a resampling, reads the front from these flags (TRENCH on older
maps), reports `front` (both, against SUB plus CCB) and `front_sub` (against SUB), and the same
numbers on one lattice (`_k7`). PB2002 through the same code (`bird_raster.py`, the front flags
on the overriding side of SUB steps and both sides of the unpolarised CCB steps): on the lattice
SUB plus CCB is 79,608 km at 77 km and 72,965 at 380, SUB alone 52,743 and 48,619; through the
one-lattice step 77,844 / 72,060 and 51,614 / 47,982, the boundary 259,814 / 232,613; SUB flagged
TRENCH on its subducting side, as maps without the flags read, 51,812 / 48,094 on one lattice. On
one of fl-ls2's runs (scattered start, seed 1; old maps, so the TRENCH front) the one-lattice front
is 98,048 km at 77 km, 1.26 times SUB plus CCB.

**The arm** (`pool.sh run fl-carry --length 1000 --seeds 1,2,3,4 --threads 2
+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`, key 7836eb64cc67, argkey
cd665cef, 8 runs, both starts, the carry on by default). Read against base6's own seeds 1 to 4
(arm `base6-s1to4`, the same runs as base6's, so matched starts and seeds), and against base6's
24 where said; medians, Mann-Whitney p in brackets:

- **The dip.** The record's mix by Myr since the resampling, as in the table above: the
  subducting length is 83,020 km at 1 to 2 Myr (63,341 before), 89,173 at 3 to 4 (85,184), 97,992
  at 13 to 20 (99,562), and the converging boundary with no contact 5,557 km at 1 to 2 Myr (37,459)
  against 3,600 to 4,300 otherwise. Over the cycle the subducting length's time mean is 0.954 of its
  level at 13 to 20 Myr, against 0.921. Ocean-ocean fronts and collisions no longer dip (38,347 and
  21,153 km at 1 to 2 Myr against 38,080 and 19,933 at 13 to 20); the oceanic arc cells neither
  (2,509 against 1,858 before). What is left is the ocean-continent front alone: 44,673 km at 1 to
  2 Myr, climbing to 59,912 at 13 to 20, as before the carry (base6 37,342 to 60,320) and before
  one-way subduction (fl-shown, master 25941ca5, 39,174 to 62,812). It is not a contact switched
  off: the boundary that is neither a front, a ridge, a rift nor converging is 16,000 to 17,500 km
  longer at 1 to 2 Myr than at 13 to 20 in all three arms, so after a rebuild about that much of the
  ocean-continent boundary is judged not converging and comes back over the cycle. The continental
  arc cells follow it (435 at 1 to 2 Myr to 652; base6 357 to 648). The cause is not found.
- **The plate check** passes 183 measures, as base6-s1to4. In fewer runs: the oceanic over
  continental speed (5 against 8, 2.08 against 2.50), the slab-pulled over free speed (6 against
  8), the rifts alive at 50 Myr (3 against 5), the lowest continental share (0 against 2). In more:
  the speed and area's rank correlation (5 against 2), the collisions' p90 length (2 against 0,
  143 against 186 Myr), the active margin, the supercontinent's tenure and period, the large
  plates' p90 speed, the collisions' convergence ratio, the continental arcs' p50 height.
- **The V1 census.** Land 0.318 to 0.298 (p 0.23), the largest landmass 0.198 to 0.178 (p 0.57),
  continental crust under the sea 0.202 to 0.234 (p 0.51), **land above 2 km 0.128 to 0.174 (p
  0.038; against base6's 24, p 0.015)**, **coast with high ground 0.244 to 0.282 (p 0.021; against
  the 24, p < 0.001)**, ice 0.114 to 0.105, collisions per 100 Myr 1.45 either way, the continental
  crust's net change -0.054 to -0.057 (p 0.51), criteria met 5 either way (mean 5.12 to 4.88).
  Land above 2 km is higher in all 8 start-and-seed pairs, by 0.002 to 0.138 (chaotic planets, so
  a sign count, not a paired test).
- **The guard.** No step beyond 2 Earth reliefs; the worst 10,236 m from the sea against 10,418
  (p 0.80); `guard_bind_steps` 0 in all 8 runs.
- **The area book.** The continental crust ends at 0.365 of the planet against 0.369, its volume
  6.82 against 6.92. The parts that moved: the fronts' hiding -0.679 to -0.722, the oceanic arcs'
  maturing (`finish`) +0.289 to +0.319; in volume, the arcs 917 to 990, the belts 14,910 to
  15,290, erosion -8,839 to -9,287.
- **The arcs' heights** by Myr since the resampling (`arc_continental` in the record): the
  continental arcs' p90 stands 100 to 280 m higher in every phase of the cycle (3,580 against
  3,304 m at 1 to 2 Myr, 3,096 against 2,975 at 13 to 20).
- **The boundary and the front** (record): the subducting share of the boundary 0.289 to 0.301 (p
  0.021; Bird SUB 0.197), converging with no slab 0.031 to 0.013 (p < 0.001), the subduction front
  82,604 to 85,882 straight km a step (p 0.16), the boundary 309,725 to 308,120, E 1.813 to 1.801.
  The crust that went under 2.99 to 3.15 km² a year, the resamplings consumed 2.86 to 2.99.
  Sutures 29,682 to 27,220 km per 100 Myr (p 0.083); the partings' share of front-time 0.340 to
  0.508 (p 0.015).
- **The measure, version 3, on the arm** (8 runs, every snapshot's front from the flags, 3 a run
  skipped at a resampling). The front, subduction and collision: 126,931 / 96,237 km at 77 / 380
  km, 1.59 / 1.32 times PB2002's SUB plus CCB on the lattice; **on one lattice 87,550 / 75,693,
  1.12 / 1.05 times PB2002's through the same step** (1.18 / 1.02 times Bird's 74,313). Subduction
  alone: 104,979 / 79,430, on one lattice 71,270 / 61,756, 1.38 / 1.29 times SUB. The boundary on
  one lattice 299,021 / 262,736 (1.15 / 1.13 times PB2002's), perimeters over caps 1.708 / 1.499
  (1.503 / 1.345), the mostly-front pieces' tortuosity 1.519 / 1.318 (1.247 / 1.138). The front
  as the TRENCH flag gives it, on one lattice: 98,604 / 85,193, between fl-ls2's 102,658 / 87,707
  and fl-sat's 94,863 / 81,007 (older simulations).

What the arm says. The carry does what it is for: the converging boundary with no contact after a
rebuild falls from 37,500 km to 5,600, the island arcs and collisions go on through it, and the
subducting length's time mean is 5 % under its steady level, not 8 %. The rest of the dip is the
ocean-continent boundary's own geometry after a rebuild, which a contact cannot reach. Measured
like for like, on one lattice and against SUB plus CCB, the model's front is now 1.12 times
PB2002's at 77 km and 1.05 at 380 km; most of the 2.2 times step 6 left was the measure (the
snapshots at a resampling, SUB alone against a front that holds the collisions) and the two
lattices. Subduction alone is still 1.38 times SUB, and the collision front shorter than CCB, so
the model calls more of its convergent boundary subduction than Earth does.

What it costs is high ground: land above 2 km and coast with high ground rise, both realism checks
that already fired. A reading, not a measured cause: before, every rebuild took a third or more of
the island arcs' and the collisions' fronts away for 1 to 2 Myr and a tenth for the next 2 (base6:
ocean-ocean 25,999 km at 1 to 2 Myr against 39,242 at 13 to 20, collisions 12,331 against
21,007), every 16 to 20 Myr, while erosion went on, so arcs and belts stood lower than their own
laws give; now the uplift runs all the time, the continental arcs
stand 100 to 280 m higher through the whole cycle, and a share over a threshold moves more than the
volumes (the arcs' 8 %, the belts' 2.5 %). The census was flattered by the artefact; no knob is
added to take it back (the foreland basins of gap 6 are the lane expected to lower both).

On 24 runs (base7, master 1b56529d, the carry on, against base6's 24) the rise is smaller and no
longer significant: land above 2 km 0.128 to 0.141 (p 0.23), coast with high ground 0.228 to
0.266 (p 0.059), the largest landmass 0.156 to 0.210; the guard's worst 10,140 m, no step beyond
2 reliefs. fl-carry's 0.174 and 0.282 were seeds 1 to 4's draw.

**The two lattices after the carry.** The front on the model's own samples is 1.45 times its
length on one lattice at 77 km and 1.27 at 380 km, where PB2002's raster is 1.02 and 1.01: the
part the carry cannot touch, as large as before it (fl-ls2's raw over one-lattice, 1.39 at 77 km).
The record's own subducting length (the boundary mix, 93,743 lattice km) is 1.32 times the
one-lattice subduction front, so the boundary criteria (the subducting share 0.301 against Bird's
0.197) read a front 1.32 times its straight length. Corrected in step 8: that 1.32 is not the
edge row reaching in. The boundary mix counts one spacing per overriding sample on its own
plate's lattice, which is PLT-530's count on the model's ragged fronts (1.3175 times their
straight length, arm fl-share), on lattice adjacency, which the two lattices' interleaving does
not reach; the edge row reaching in is in the raw maps' front over the one-lattice front (part 2
above). Whether the laws that count front samples see either is measured in step 8.

### Step 8: a sample goes under at the outline (PLT-553), and what the slab pull and the arcs count (PLT-554) (2026-10-24)

Step 7's part 2 left the front on the model's own samples 1.45 times its length on one lattice at
77 km, where PB2002's raster is 1.02: the subducting plate's edge row reaches into the overriding
plate's before it goes under, because `geometry::cover` hides a subducting sample only once a
whole lattice triangle of the overriding group holds it. The resampling footprint already
counts the crust finer than that: since step 3c a new lattice point belongs to the group that
weighs the most in the old triangle by the interpolated signed distance (PLT-536), so the
outline lies between the samples, about half a spacing in from the hull of the group's
triangles where two groups meet.

**The law** (`tectonics.outline_cover`, on by default; `level_set::cover`,
`level_set::distances_now`, `subduction::overlaps`; ledger PLT-553). A sample lies on another
group's surface where the resampling would give that group the point: in a triangle of the
group's own lattice, in its own frame, where that group weighs the most by `level_set::margin`
(weights renormalised to the group's corners). The distances are the level set's where the
neighbours' groups are as at the last resampling, half a spacing where not, which is the old
midpoint rule. So a sample goes under the step it crosses the overriding plate's outline. Off,
the hull rule as before. The overlaps feed `resolve` unchanged: an existing contact hides what
overlaps it each step, a new contact needs `MIN_CONTACT` overlaps and convergence. No constant of
its own.

**The unit tests.** `a_sample_goes_under_where_it_crosses_the_outline`: the two-plate planet
(20,000 samples, the outline the equator, held exactly by the level set), the ocean turned under
the continent by a third of a spacing on one half. 63 samples lie past the outline; the outline
rule finds all 63 and none short of it, the hull rule none (19 lie within 0.1 spacings of it, 30
within 0.1 to 0.2, 14 within 0.2 to 0.35). By the hull rule a sample a third of a spacing past
the outline (24 km on the quick lattice, a quarter of a 2-Myr step at 50 mm/yr) is still on top.
`what_the_outline_cover_finds_at_a_transform` is the risk: the same plates sliding past each other,
which no sample crosses. On the exact outline neither rule finds an overlap. On a lattice's outline
(the midpoints of the edges between the plates, as a cut leaves it until a resampling) the outline
rule finds up to 0.22 overlaps per spacing of transform once the plates have slid a spacing (0.27
after one resampling, 0.16 after five), up to 0.48 spacings from the boundary, where the hull rule
finds up to 0.04: one plate's zigzag outline no longer nests in the other's. An overlap at a
transform hides a sample only where a contact between the two plates already stands, and starts
none without convergence, so the arm reads hiding and consumption, not only the fronts.

**What the slab pull and the arcs count** (measurement only; PLT-554). The slab pull
(`forces::balance_step`) acts at each overriding edge sample with a front, one spacing of trench
each (`pull_n_m` is per metre, FRC-002); so do the slab resistance (FRC-017), the boundary
resistance and the edge forces. The arcs' magma is an areal field over the distance from the
fronts (`w_arc_of`, PLT-045) and does not count samples. The record now books, in km × Myr: the
slab pull's spacings (`plates/slab_front_lattice_km_myr`), their part along the boundary's normal
(`plates/slab_front_normal_km_myr`), the same samples' shares of the straight front
(`plates/slab_front_straight_km_myr`, PLT-530's crossings), and the arcs' band weight over the
band's width (`plates/arc_band_km_myr`), against the arcs' straight front
(`plates/arc_front_straight_km_myr`). The test `the_fronts_by_the_slab_pull_and_the_arcs` prints
them on a 100-Myr run.

**The ocean-continent dip, from the records** (base7's 24 runs, every step after the first 100 Myr,
by Myr since the last resampling; no simulation). The boundary mix is taken before a step's
resampling, so the resampling step's row is the end of a cycle and "1 to 2" is the first step on a
new lattice. km:

| Myr since the resampling | 0 | 1 to 2 | 3 to 4 | 7 to 8 | 13 to 20 |
|---|---|---|---|---|---|
| ocean-continent front | 61,071 | 44,564 | 49,114 | 52,661 | 60,099 |
| of it, normal convergence 40 mm/yr or more | 37,888 | 23,656 | 25,167 | 28,285 | 37,012 |
| of it, slab 600 km or longer | 50,149 | 32,174 | 35,487 | 39,596 | 49,044 |
| of it, contact 50 Myr or older | 42,297 | 27,457 | 30,466 | 33,296 | 41,244 |
| ocean-ocean front | 37,013 | 38,166 | 39,457 | 39,836 | 37,794 |
| collision front | 20,536 | 20,932 | 20,308 | 20,224 | 20,097 |
| judged a transform (other, less converging with no contact) | 67,862 | 82,352 | 79,301 | 72,261 | 65,417 |
| rift | 17,458 | 7,779 | 10,316 | 12,802 | 17,056 |
| ridge | 100,517 | 108,089 | 108,450 | 107,654 | 101,020 |

The dip is the mature, fast margins': 14,200 of the 16,500 km lost lie at 40 mm/yr or more of
normal convergence, 18,000 km on slabs of 600 km or longer, 14,800 km on contacts 50 Myr or older,
while the slower ocean-continent fronts, the ocean-ocean fronts and the collisions hold. The plates
do not slow (the visible-weighted speed 23.0 mm/yr at the end of a cycle, 22.4 at 1 to 2 Myr, 22.6
at 13 to 20), and the length does not shift to slower bands: it goes to the boundary judged a
transform (Bird's 20 degrees, `shortening::is_transform`, on the crossing sums' normal), which
gains 14,500 km at 1 to 2 Myr, and comes back step by step through the whole cycle, not within 2
to 4 Myr as the contacts' dip did before the carry. Between resamplings the lattice's adjacency is
fixed and the boundary normal is read in each sample's own plate's frame, so a sample's class can
change only through its or a neighbour's plate, its hiding, its contact's state or the plates'
relative motion; the records do not say which one turns these fronts back on through the cycle.
The rifts show the same saw the other way: a rift opened between resamplings keeps its
continental samples as lattice neighbours across the new ocean, and is called a rift until the
next resampling fills it with ocean floor (rift 17,500 km at the end of a cycle, 7,800 after it,
ridge the mirror). The probe that would name the ocean-continent cause is a unit test: the
two-plate planet with a fast ocean-continent margin through a resampling, each front sample's
normal, transform test and contact before and after. Not run here (the coordinator's call: records
and maps first, no simulation).

**The arm** (fl-outline: master 1b56529d with the outline cover on, both starts × seeds 1 to 4, 1000
Myr, `+DEEPTIME_CRUST_BUDGET=1` with its maps; key e6bdf186ab78, argkey cd665cef; `pool.sh
measure` v1census, guard, front, boundgeom, shown, hiding, boundary, sutures; `build/front`'s
records-only scripts for the mix by phase, the plate check, the area book and the PLT-554 globals).
Read against base7's same eight runs (fl-carry's: same key and argkey) and base7's 24; two-sided
Mann-Whitney p against the eight in brackets unless said.

- **What it is for.** The front on the raw maps 109,350 km at 77 km against 126,931, 95,030 at 380
  km against 96,237; over the one-lattice map 1.16 and 1.14 times its length, from 1.45 and 1.27
  (PB2002's raster 1.02 and 1.01). Cut-off pieces of the plates 20.3 a snapshot against 63.2, and
  0.85 of the cut samples lie beside their own hidden samples against 0.71 (p < 0.001): what is
  cut off is now mostly an edge that has just gone under. The resamplings stop eating the
  continents: in the area book the resampling's blend takes 0.015 of the planet's area of
  continental crust over the run against 0.137, the fronts' hiding 0.892 against 0.722 (the loss
  moves from a numerical step to the law that books it); the continental crust ends at 0.378 of
  the planet against 0.365, its volume 7.08 against 6.82 (units of the book), the land 0.333
  against 0.313. Converging boundary with no contact in the first 2 Myr after a rebuild: 137 km
  against 5,557 (the new lattice's overlaps are found at once).
- **The risk at transforms did not show.** The boundary judged a transform 0.233 against 0.236 (p
  0.96); converging with no slab 0.005 against 0.013 (p < 0.001); subduction initiations 10.05
  per 100 Myr against 11.80, polarity reversals 9.1 against 9.5 (p 0.53). The crust that went
  under 3.34 km² a year against 3.15, the resamplings consumed 3.17 against 2.99; what showed
  again because the cover moved off it 4.2 % of what went under against 3.8 %, what escaped the
  resamplings 3.9 % against 3.4 %.
- **The record's fronts did not lengthen.** The subduction front 84,364 straight km a step against
  85,882 (p 0.44), the boundary mix's subducting length 91,822 lattice km against 93,743. The
  one-lattice map's front did: 94,515 km against 87,550 at 77 km, 83,037 against 75,693 at 380
  km (1.21 and 1.15 times PB2002's through the same step, from 1.12 and 1.05), and its TRENCH
  flags' front 125,350 against 98,604. A reading, not traced: the one-lattice point takes its
  flags from its plate's nearest sample, and the subducting plate's flagged edge row, reaching
  into the overriding plate's samples, lost the 7-sample vote, so the map's boundary ran beside
  the subducting plate's unflagged second row; with the overlap gone the flagged row is the
  visible edge. If so, step 7's 1.12 and 1.05 were low by the overlap, and 1.21 and 1.15 is the
  like-for-like figure (subduction alone 1.49 and 1.41 times SUB).
- **Fewer plates.** 24.5 alive a step against 27.9 (p 0.014; PB2002 52), the boundary 288,044
  straight km against 308,120 (p 0.015), E 1.818 against 1.801 (p 0.16); rifts 1.85 per 100 Myr
  against 2.45, fission births 44 a run against 47.5, a plate's median life 65.5 Myr against
  59.5. The subducting share of the boundary 0.312 against 0.301 (p 0.028; Bird 0.197), from the
  shorter boundary, not a longer front. The mean convergence across the oceanic front 41.4 mm/yr
  against 43.4 (p 0.015), the visible-weighted plate speed 2 to 4 % lower through the cycle (22.5
  against 23.0 mm/yr at a rebuild, 21.8 against 22.6 at 13 to 20 Myr). Not traced to a cause.
- **The V1 census**, against the eight and base7's 24 (p against the 24 second): land 0.323
  against 0.298 and 0.302 (0.38, 0.25); the largest landmass 0.208 against 0.178 and 0.210;
  land above 2 km 0.118 against 0.174 and 0.141 (0.083, 0.51); coast with high ground 0.254
  against 0.282 and 0.266 (0.19, 0.78); collisions 1.75 per 100 Myr against 1.45 and 1.65; the
  criteria met 5.12 a run against 4.88 and 5.04. Nothing significant.
- **The guard.** The worst 10,272 m from the sea against 10,236 (p 0.79) and base7's 10,140 (p
  0.094); no step beyond 2 Earth reliefs, `guard_bind_steps` 0 in all 8 runs. All 8 ran (1,150 s
  each at 2 threads, as before).
- **The plate check** (the V1 dynamics criteria): 182 passes against 183. Moved: a supercontinent's
  tenure 83 Myr against 35 and its period 264 against 199; the collisions' p90 length 201 Myr
  against 143; the speed-area rank correlation -0.43 against -0.38 (2 of 8 in the band against 5).
- **The ocean-continent dip is not this one's.** The ocean-continent front still falls by a quarter
  in the first 2 Myr after a rebuild (58,012 to 43,739 km, against 61,635 to 44,673), into the
  boundary judged a transform.

**The slab pull and the arcs on the arm** (PLT-554; the 8 runs' medians, each run's time mean).
The slab pull counts 131,600 km of trench a step, one spacing per pulling sample: 1.108 times the
same samples' straight front (1.093 to 1.114 over the runs; the test's 8,000 samples 1.106 with
the outline cover and 1.138 without, one run each), 0.708 of it along the boundary's normal. Its
fronts are not the arcs': the arcs' straight front is 0.648 of the pull samples' (0.868 and 0.869
on the test planet, with and without). The force balance's front (`forces.rs`, `Edge::front`) is
still every lattice edge converging on an active contact, the rule PLT-330 took from the arcs on
2026-10-20 because a zigzag boundary converges along half its edges where the plates slide past
each other; so the pull also acts along the contacts' strike-slip stretches. In all it counts 1.71
times the arcs' straight front in spacings, and 1.21 times it along the normal. So the slab pull
does see an inflation, larger than the one the two lattices make and of another kind: the count
(1.11) and the old front (1.54). The forces scale with both. The speeds see less of it, because
the slab resistance (FRC-017) counts the same samples and the drag (FRC-016) was calibrated on a
straight reference trench; how much less depends on each plate's resistances and was not
measured. The arcs make their crust over their band, not their samples (`plan.magma_m` is the
axis rate × `w_arc_of` × g at each sample the band reaches), and the band is 1.316 times the
straight front they book (1.119 and 1.151 on the test planet, where 250-km spacings barely
resolve a 60-km band). A reading, not measured: the band 200 km behind a front wraps round both
its ends, so a front of length L gets about L + π · 200 km of band, 1.3 times L for pieces near
2,000 km; Earth's arcs stop near the slab's edge. Neither is changed here.

**Recommendation: the outline cover on by default** (it is). It is built from the resampling's own
rule, its unit tests hold, all eight runs ran, the guard did not bind and the risk it was read for
(hiding and spurious contacts at transforms) did not show. What it changes in the realism checks
(fewer plates, a little slower convergence, the census unmoved within its noise) is the physics'
answer, not a reason to keep the hull rule. The two inflations left are the slab pull's (its old
front and its count, a step of its own: give the pull the arcs' fronts and their straight shares,
along the normal) and the arcs' band ends (a unit test on a straight front of known length first).

### Step 9: the slab pull at the arcs' fronts (PLT-560), and the drag's calibration on its count (2026-10-24)

Step 8 measured what the slab pull counts: one lattice spacing at each overriding sample with a
front, along the fastest-converging lattice edge, where the front is every lattice edge converging
on an active contact, so 1.71 times the arcs' straight front in spacings and 1.21 times it along
the normal on the default planet (PLT-554). The coordinator's calls for this step: the pull takes
the arcs' fronts (PLT-330's filter) and their straight shares, along the normal, as one cumulative
8-run arm; FRC-016's drag is calibrated again on the same count against its own definition (the
reference plate and its torque balance, not the V1 checks), and if the speeds then fall outside
their sourced band that is reported, not tuned; a unit test of the arcs' band ends before any law
change; the ocean-continent dip probe as a two-plate unit test through a resampling.

**The arcs' band, first** (`the_arcs_band_runs_round_a_fronts_ends`, 40,000 samples, no law
changed). A straight front of known length (`a_front_of_known_length`: a box of plate 1 under plate
0, its edge a great-circle arc), on the equator and on a great circle tilted 60 degrees; the band's
weight over its width (`plates/arc_band_km_myr`, PLT-554) split at the meridians of the front's
ends into the part along the front and the part past its ends:

| Front | Straight front over its length | Band over the straight front | Along the front, over the straight front | Past the ends, over π × 200 km |
|---|---|---|---|---|
| equator, 10 degrees (1,112 km) | 0.971 | 1.371 | 0.839 | 0.91 |
| equator, 20 degrees (2,224 km) | 0.991 | 1.085 | 0.828 | 0.90 |
| equator, 40 degrees (4,447 km) | 0.992 | 0.989 | 0.883 | 0.74 |
| equator, 60 degrees (6,671 km) | 1.002 | 0.994 | 0.911 | 0.89 |
| tilted, 10 degrees | 0.989 | 1.236 | 0.806 | 0.75 |
| tilted, 20 degrees | 0.985 | 1.041 | 0.886 | 0.54 |
| tilted, 40 degrees | 1.001 | 1.008 | 0.896 | 0.79 |
| tilted, 60 degrees | 0.983 | 0.967 | 0.906 | 0.64 |

Step 8's reading holds in its first half: the band runs round both ends of a front, 0.54 to 0.91
of the two quarter circles (340 to 574 km) whatever the front's length (the split's meridians lie
within a spacing of the lattice front's ends, which moves up to 0.15 of the wrap from one part to
the other at each end). Its second half does not: a front of length L does not get L + π × 200 km
of band, because the part along the front falls 9 to 19 % short of L. The band is made on the
lattice distance from the nearest front sample (`fronts_reach`), which runs longer than the
straight line across the band (up to 2/√3 on a hexagonal lattice, by the edges' direction), so the
band is narrower than `arc_width_km` says. The two partly cancel: 1.24 to 1.37 times the straight
front at 1,100 km, 1.04 to 1.09 at 2,200 km, 0.97 to 1.01 from 4,400 km. The planet's 1.316
(step 8) is about what fronts near 1,000 km get. Not changed here: the ends are a law of their own
(the coordinator's question for step 10), and the shortfall along the front is the lattice's metric
under every distance field, not the arcs'.

**The law** (`tectonics.front_pull`, on by default; `forces::balance_step`, the slab feed and the
hanging slab; ledger PLT-560). A front sample is an overriding edge sample whose plates converge
across their boundary beyond Bird's 20 degrees of a transform (`shortening::is_transform`, PLT-311,
the rule PLT-330 gave the arcs' fronts). The slab pull acts there along the boundary's normal
(`shortening::boundary_normal`, PLT-310) over the sample's share of the straight front
(`shortening::front_share_rad`, PLT-530), and the slab grows by the convergence along that normal.
The slab resistance (FRC-017), the slab suction (PLT-540) and the collision resistance (FRC-018)
count the same metres along the same normal. A slab no front feeds (it hangs for up to `slab_breakoff_myr`) pulls along
the normal over the shares of the whole shared boundary. Off, the old law: every lattice edge
converging on an active contact is a front, one spacing per overriding sample along that edge, the
slab fed by the convergence along it. No constant of its own.

**The unit tests.**
- `the_slab_pull_takes_the_arcs_fronts_along_their_normal`: the front of 40 degrees, plate 1
  converging at 50 mm/yr at θ from the front's normal. On, the pull's torque is the reference
  pull's over the front's straight length along its normal: 0.988 and 0.996 of it (equator,
  tilted) at 0 and 45 degrees, 0.956 and 0.964 at 60, within a degree of the normal; the slab fed
  at 0.97 to 0.99 of the convergence across the front's middle. At 80 and 90 degrees the box's
  edge has no front: none on the equator; on the tilted box a corner front or two, where the
  boundary turns, pull 0.068 and 0.020 of the front's (asserted below 0.1). Off: 1.095 and 1.052
  at 0 degrees, 18 to 24 degrees off the normal at 45 and 60, the slab fed up to 1.30 times the
  convergence at 60, and at 90 degrees (a transform) 0.46 and 0.66 of the front's pull with a slab
  fed at 36 to 44 km/Myr.
- `the_reference_plate_on_the_lattice_moves_at_the_speed_scale`: the drag's definition on the
  lattice (below). On, the pull is exactly the lattice trench's shares along the normal.
- `a_split_plates_pieces_pull_with_the_parents_slab_force` (inherit.rs, PLT-534): a plate split
  across its trench keeps its pull and suction. Off, exactly, as before. On, to about a spacing of
  pull at each junction where the cut meets the trench, because each piece's front share is taken
  on its own boundary: 0.58 spacings at the weight's cap in case 0, 0.10 in case 1, 0.66 over case
  4's two junctions, cases 2 and 3 exact; the tolerance is a spacing per junction.

**The drag's calibration on the pull's count** (FRC-016; the full account in
[plate-forces.md](plate-forces.md), "The drag's calibration on the pull's count"). The definition
puts a reference oceanic plate (the average starting plate's area, a straight 3,000 km trench of a
full-length 80-Myr slab, its neighbours at rest) at the speed scale, and c_b follows from it in
closed form (`Coeffs::new`). The definition counts the trench straight and along its normal, which
the old pull did not: on the 40,000-sample lattice its pull came to 1.235 times the rim trench's,
11 degrees off the normal, and the plate to 55.4 mm/yr. On the pull's own count it is 1.060 times
the rim's, 0.5 degrees off, exactly the lattice trench's shares (3,193 km: the lattice's trench ends
where the plate's samples end), and the plate 53.2 mm/yr, 50.2 per the definition's 3,000 km,
against 49.5 for the analytic ring's plate (the flat definition on a spherical cap). So the lattice
adds nothing but its trench's length, and c_b does not move: 2.5 × 10¹⁴ Pa s/m, 0.35 MPa at 45
mm/yr (Forsyth & Uyeda's "roughly 5 bars" at 8 cm/yr is a coefficient 0.8 times that). What moved
is the pull on the planet: the old count's surplus is gone, so the plates move more slowly for the
same c_b.

**The ocean-continent dip probe** (`the_ocean_continent_front_through_a_resampling`; step 8's
records: 16,500 km of ocean-continent front lost within 2 Myr of a resampling, to the boundary
judged a transform, on the mature, fast margins). The two-plate planet of `converging_plates`
(20,000 samples, an ocean moving north under a continent, head-on, at up to 50 mm/yr) goes through
a resampling with its contact carried (PLT-550), then a step on, the boundary mix and the
continent's margin read sample by sample at each stage:

| | Before | After the resampling | A step on |
|---|---|---|---|
| ocean-continent front (the mix, km) | 21,715 | 22,993 | 22,993 |
| other (km) | 80 | 160 | 160 |
| ocean-continent margin samples | 117 | 131 | 131 |
| of them, the normal over 70 degrees off the convergence | 0 | 0 | 0 |
| the normal's mean angle off it (degrees) | 3.3 | 6.2 | 6.2 |
| of them, beside hidden crust | 117 | 0 | 80 |
| the contact | active | active, carried | active by its overlap, not carried |

No dip: the head-on margin keeps its class, its normals turn by a few degrees and the contact holds.
So the records' dip is not the resampling of a head-on margin as such. What this fixture lacks is
where to look next: obliquity (a margin near 70 degrees of its convergence, which a few degrees of
the new lattice's normal would tip across Bird's 20), plates of several groups, a real margin's
shape and other plates nearby. The probe that would decide is one on a real run's state: a base
run's planet at a resampling, the margin samples that turn transform followed through it (class,
normal, the transform test's angle, contact). The test asserts what it found (the front within 0.9
of before, no normal over 70 degrees off at any stage, the contact active).

**The arm** (fl-pull: front-pull at 7121c383, master 4dd85fe2 with `front_pull` on; both starts ×
seeds 1 to 4, 1000 Myr, `+DEEPTIME_CRUST_BUDGET=1` with its maps; key 822ccd2a2d81, argkey
cd665cef; `pool.sh measure` v1census, guard, front, boundgeom, shown, hiding, boundary, sutures;
`build/front`'s records-only scripts for the plate check, the speeds, the area book, the mix by
phase and the PLT-554 globals). Read against base8 (master 4dd85fe2's 24 runs, key 6b02802a0f21,
an alias of fl-outline's e6bdf186ab78) and fl-outline's same eight, which base8 holds; two-sided
Mann-Whitney p in brackets, against base8 unless said.

- **What it is for.** The pull counts 73,500 km of straight front a step along the normal, against
  the old count's 131,553 km of spacings, 93,140 of them along the normal (0.79). The pulling
  fronts are now 0.90 of the arcs' straight front, against 1.21 along the normal before; the arcs
  also count fronts whose slab does not pull (continental crust going under, a slab under 10 Myr
  old or of no length yet), so the pull's fronts are a part of theirs, as they should be. The
  pulling samples' spacings are 82,767 km a step, 1.126 times their straight front and 0.845
  along the normal (PLT-554, which now books the pulling fronts' own metres).
- **The speeds** (the plate check's records; the sourced bands in the ledger, CHK-020 to CHK-023;
  reported, not tuned): the area-weighted RMS 25.95 mm/yr (23.8 to 28.6 over the runs) against
  26.79 (0.007) and fl-outline's 27.12 (0.024), the band 40 to 100 (Forsyth & Uyeda's 47 today,
  Zahirovic's 50 to 100 over 200 Myr); the median 23.05 against 26.70 (< 0.001), the band 30 to
  60; the 90th percentile 66.35 against 65.15 (0.12), the band 70 to 110; the 10th 6.64 against
  8.05 (0.011), at most 20. Slab-attached over slab-free plates 3.07 against 2.50 (0.008; Earth
  about 3 to 4), oceanic over continental 2.62 against 2.20 (0.094; 2 to 3). The visible-weighted
  speed 18.6 to 19.4 mm/yr through a resampling cycle against 21.3 to 22.5. The RMS and the median
  were outside their bands on master already; the pull's own count takes them 3 % and 14 % further
  below, and leaves the fast plates as they were: the plates that slow are the ones the old count
  pulled along their contacts' strike-slip stretches. The drag's calibration is not the cause: the
  reference plate on the lattice meets its definition on this count (above). Full table in
  [plate-forces.md](plate-forces.md), "On the pull's own count".
- **The boundary.** Judged a transform 0.209 of the boundary against fl-outline's 0.233 (< 0.001;
  Bird 0.283, the band 0.22 to 0.34), spreading 0.398 against 0.377 (< 0.001), subducting 0.323
  against 0.312 (0.065), the boundary criteria met 2.75 a run against 3.75 (0.003): the transform
  share falls under its band. The mean convergence across the oceanic front 37.6 mm/yr against
  41.4 (< 0.001; Bird 53.2). A reading, not traced: the old pull at an oblique front ran 18 to 24
  degrees off the normal (the unit test's 45 and 60 degrees), along the lattice edge, so it drove
  part of the plates' motion along their boundaries; pulled along the normal they slide past each
  other less.
- **The fronts.** The subduction front 89,751 straight km a step against fl-outline's 84,364 (p
  0.038; Bird SUB 51,310), the ocean-ocean front 42,299 against 37,741 (0.001), the one-lattice
  map's front 100,789 km against 94,515 at 77 km (PB2002's 77,844 through the same step). Plates
  24.7 alive a step against 24.5 (0.56), the boundary 298,052 straight km against 288,044 (0.33).
- **The V1 census.** Land 0.288 against 0.321 (0.009) and fl-outline's 0.323 (0.038); the largest
  landmass 0.121 against 0.219 (0.004); continental crust under the sea 0.318 against 0.236
  (0.070); land above 2 km 0.086 against 0.117 (0.046); coast with high ground 0.198 against 0.253
  (0.012); land under ice 0.083 against 0.105 (0.086); collisions 1.95 per 100 Myr against 1.80
  (0.21); the criteria met 5 a run (mean 5.12) against 5 (5.08). In the area book the continental
  crust ends at 0.388 of the planet as on base8 (0.388), but thinner (a mean 35.6 km against 36.1)
  and lower: the land 0.300 against 0.331, the belts' volume 13,340 units against 14,900, the arcs'
  793 against 925 (per km of front and Myr, continental arcs 15.8 to 17.9 against 18.5 to 21.4,
  oceanic 44.6 to 53.0 against 58.5 to 65.3: their rate follows the convergence), what went under
  14,960 against 16,880. The same crust, slower plates, smaller belts, more of it under the sea; a
  supercontinent's tenure 38 Myr against fl-outline's 83 (one run of eight in the band each).
- **The guard.** The worst 10,102 m from the sea against base8's 10,272 (p 0.33; fl-outline's
  eight the same median, p 0.32), the largest rise and fall of a cell in a step 7,160 and 7,354 m
  against 7,418 and 7,296; no step beyond 2 Earth reliefs, `guard_bind_steps` 0 in all 8 runs. All
  8 ran (about 890 s each at 2 threads).
- **The plate check:** 181 passes against fl-outline's 182 (base8's alias folder holds only one
  run's `plate.json`). Moved: the median speed 0 of 8 runs in its band against 2; slab over free 7
  against 4; collisions per 100 Myr 4 against 3; the collisions' p90 length 176 Myr against 201.
- **The ocean-continent dip is not this one's either.** The ocean-continent front falls from 58,279
  km at a rebuild to 45,244 in the first 2 Myr after it (fl-outline 58,012 to 43,739).

**The weld seeds.** Three unit tests need a weld or a suture within 300 Myr on the 8,000-sample
planet, and their seeds no longer had one: `collisions_weld_on_stall_and_transfer_on_shortening`
(scattered 12), `recording_does_not_change_the_world_and_matches_the_run` (scattered 20) and
`sutures_and_initiations_keep_continents_whole` (supercontinent 5, scattered 12). The sweep
(`weld_seed_sweep`, ignored, now over both starts): scattered continents weld on 13 of seeds 1 to
24 (1 to 6, 10, 14, 15, 18, 21, 23 and 24; 7, 8, 11, 12 and 20 no more), the supercontinent on 8
(4, 7, 11, 14, 15, 21, 23 and 24; 5 no more). The collisions and recording tests pass on all 13
welding seeds, the sutures test on each of 8 pairs tried; the tests take scattered 1, scattered 2
and the pair supercontinent 4 with scattered 1. The seed is chosen by its weld, not by a result.

**Recommendation: the slab pull at the arcs' fronts on by default** (it is). It counts each front
the way the per-metre forces' definitions do (a straight trench, the pull normal to it), on the
arcs' fronts by Bird's transform rule; its unit tests hold against the reference pull and the
reference plate, all eight runs ran and the guard did not bind. What it changes in the realism
checks (slower plates, the RMS and median speeds further under their sourced bands, the transform
share just under Bird's band, less land and a smaller largest landmass) is the physics' answer, not
a reason to keep the old count, and none of it is tuned. The speeds' shortfall is older than this
step and is not the drag's calibration; it is a diagnosis of its own (the planet's plates against
the reference plate). Step 10's question is the arcs' band ends (above): stop the band at the
slab's ends, Earth's arcs stopping near the slab's edge, with a unit test first on the straight
front of known length. The along-front shortfall is the lattice's metric under every distance
field and is not the arcs' law.

### Step 10: the arcs' band stops at the slab's ends (PLT-561, 2026-10-24)

Step 9's band test found the band 200 km behind a front running round both its ends, 0.54 to 0.91 of
π × 200 km whatever the front's length: 1.24 to 1.37 times the straight front at 1,100 km, and on
the planet 1.316 (fl-outline) and 1.237 (fl-pull) times the arcs' straight front. The coordinator's
calls for this step: stop the band at the slab's ends, with the unit test first on the same
straight-front fixture; leave the shortfall along the front, which is the lattice's metric and not
the arcs' law; one cumulative 8-run arm, read against base9 (master's 24 runs on 06c5e327fcb0).

**The physics.** An arc stands over its slab: "It is generally accepted that the volcanic front at
subduction zones lies above the place where the slab has reached a critical depth" (England, Engdahl
& Thatcher 2004, introduction; the depth is 65 to 130 km, "The mean depth is 101 km, with a standard
deviation of 16 km", section 2.3); Syracuse & Abers (2006, abstract) put it at 72 to 173 km over
500-km segments, 105 km on average. Where the slab stops there is no arc beyond it: at a slab window
"'normal' arc volcanism is interrupted" (Thorkelson & Breitsprecher 2005, p. 27); from southeastern
Costa Rica through Panama "The absence of a Wadati-Benioff zone ... corresponds to the position of
the slab window" (Johnston & Thorkelson 1997, abstract); "no relict slab underlies the extinct
northern Kamchatka volcanic arc" (Levin et al. 2002, abstract). Slab edges do have volcanism of
other kinds, slab-edge melts (Yogodzinski et al. 2001, abstract) and volcanoes at the tear faults at
a retreating slab's lateral margins (Cocchi et al. 2017); the model has none of them. No source read
states the arc's end at the slab's edge as a general rule; it follows from the arc standing over the
slab, and the slab-window and Kamchatka cases show it.

**The law** (`tectonics.arc_slab_ends`, `subduction::over_the_slab`, on by default). A sample the
arcs' distance field reaches makes arc magma only where it stands over its front's slab: along the
line of its nearest front sample (its label in `fronts_reach`), between the ends of the front that
sample stands on. The front is the label and the front samples joined to it one by one within two
lattice edges in the same group (a zigzag boundary's second row, a corner), out to the band's reach
(`arc_distance_km` + 2 `arc_width_km`, past which its weight is under e^-4, and two lattice
spacings, because a sample's label need not be its nearest front sample on the sphere: the lattice's
distance runs up to 2/√3 longer than the straight line, by the edges' direction). The line runs
through the label square to the front's normals (PLT-310), each weighted by its share of the
straight front (PLT-530), and each front sample covers its share along it, centred on it. The arc
flag, the axis-gap rule (PLT-043), the arcs' budgets (PLT-286) and the band measure (PLT-554) follow
the magma. The cordillera (PLT-048, PLT-320), the subduction uplift and the forearc (PLT-542) still
run on the whole distance field (until step 11, below). No constant of its own.

**The unit test first** (`the_arcs_band_stops_at_the_slabs_ends`, which takes over step 9's
`the_arcs_band_runs_round_a_fronts_ends` as its off half; written and committed before the law,
eba05586). The same fixture, off and on; the band is now the samples given arc magma, by their
weight, split at the meridians of the front's ends; the farthest arc is the farthest sample given
arc magma past them, in spacings of 113 km (40,000 samples):

| Front | Band over the straight front, off / on | Along the front over the straight front (both) | Past the ends over π × 200 km, off / on | Farthest arc past the meridians, spacings, off / on |
|---|---|---|---|---|
| equator, 10 degrees (1,112 km) | 1.371 / 0.878 | 0.839 | 0.91 / 0.07 | 6.06 / 0.24 |
| equator, 20 degrees (2,224 km) | 1.085 / 0.848 | 0.828 | 0.90 / 0.07 | 6.04 / 0.29 |
| equator, 40 degrees (4,447 km) | 0.989 / 0.892 | 0.883 | 0.74 / 0.07 | 7.10 / 0.41 |
| equator, 60 degrees (6,671 km) | 0.994 / 0.933 | 0.911 | 0.89 / 0.24 | 7.07 / 1.02 |
| tilted, 10 degrees | 1.236 / 0.947 | 0.806 | 0.75 / 0.25 | 5.80 / 0.38 |
| tilted, 20 degrees | 1.041 / 0.906 | 0.886 | 0.54 / 0.07 | 5.65 / 0.34 |
| tilted, 40 degrees | 1.008 / 0.930 | 0.896 | 0.79 / 0.24 | 7.30 / 1.01 |
| tilted, 60 degrees | 0.967 / 0.912 | 0.906 | 0.64 / 0.07 | 5.75 / 0.34 |

On, the band past the ends is 42 to 156 km (0.37 to 1.38 spacings: what the lattice's front reaches
past the meridians, the end sample within about half a spacing of its meridian and its half share
beyond it), against 340 to 574 km off; no sample further than 1.02 spacings past the meridians makes
arc magma, against 5.7 to 7.3. The part along the front is the same to the km, and every sample a
spacing or more inside the meridians makes the magma it made off (the test compares them sample by
sample). The band is now 0.85 to 0.95 times the straight front at any length, the lattice's
shortfall along the front left as it is. The test bounds: past the ends under two spacings, no arc
past 1.25, along the front no more than off and within a tenth of the wrap of it, the samples inside
unchanged.

A first cut read the extent along the label's own line (its position and its own normal). It left
the band 0.07 to 0.28 of the wrap past the ends but up to 1.64 spacings out: a front's end sample's
normal turns at the lattice's corner, and the band 200 km behind it slipped past the end by that
distance times the turn's tangent. The front's line (its share-weighted normals over the reach) does
not turn at the end, which brought the farthest arc to 1.02 spacings.

**The arm** (fl-ends, cumulative on step 9: the law on, everything else master's at 41b11137; 8
runs, seeds 1 to 4, both starts, 16k samples, 1000 Myr, key 37c894cc0de0; read against base9,
master's 24 runs on 06c5e327fcb0, and against fl-pull, its matched 8; medians, Mann-Whitney p in
brackets):

| Measure | base9 (24) | fl-pull (8) | fl-ends (8), p against base9 / fl-pull |
|---|---|---|---|
| Land | 0.296 | 0.288 | 0.294 (0.48 / 0.80) |
| Largest landmass | 0.115 | 0.121 | 0.132 (0.31 / 0.44) |
| Continental crust under the sea | 0.249 | 0.318 | 0.205 (0.24 / 0.083) |
| Land above 2 km | 0.114 | 0.086 | 0.112 (0.88 / 0.028) |
| Coast with high ground | 0.249 | 0.198 | 0.233 (0.72 / 0.050) |
| Land under ice | 0.113 | 0.083 | 0.095 (0.72 / 0.38) |
| Collisions per 100 Myr | 1.90 | 1.95 | 1.65 (0.19 / 0.14) |
| Continental crust's net change | -0.045 | -0.035 | -0.077 (< 0.001 / 0.001) |
| Criteria met per run, median and mean | 5, 5.08 | 5, 5.12 | 5, 5.00 (0.44 / 0.38) |
| Sea over the datum after 100 Myr / at the end, m | 448 / 711 | 470 / 796 | 264 / 403 |
| The guard's worst, m from the sea | 10,198 | 10,102 | 10,169 (0.54 / 0.88) |
| Steps beyond 2 Earth reliefs | 0 | 0 | 0 |

The law does on the planet what it does in the test: the arcs' band is 0.891 times the arcs'
straight front (0.876 to 0.898 by run), against 1.237 on fl-pull (1.221 to 1.245), inside the 0.85
to 0.95 the test gives a straight front of any length; the band is cut at the ends and nowhere along
the front. The other front counts stay: the slab pull's spacings 1.124 times their straight front,
0.845 along the normal, the arcs' straight front 1.112 times the pulling fronts'.

The arcs make less crust. Per km of front and Myr, the continental arcs (35 km³ per km and Myr at
the reference convergence, PLT-285) went from 17.1 to 14.4 km³ (medians over the runs, the
convergence factor included), down 16 %; the island arcs (100, PLT-045) from 48.8 to 32.4, down 34
%. The island arcs lose more than the band (28 %) because the wrap ran round a continental front's
ends onto the ocean beside it, where each sample makes crust at the island-arc rate, 2.9 times the
continental. In the area book (base9's 24 runs against fl-ends' 8, medians, shares of the planet's
area in continental crust) what the step's finish adds, where mature island arcs turn continental
(`finish_samples`), falls from 0.242 to 0.195 and the net change from -0.045 to -0.077; the
continental volume at the end from 6.98 to 6.15 (×10⁹ km³), which lowers the sea over the datum (403
m at the end against 711) and leaves less continental crust drowned (0.205 against 0.249). Before
this step the arcs made 1.24 times their sourced rate per km of straight front; now 0.89, the
lattice's shortfall along the front, which the coordinator left as it is.

Read against base9, only the crust's net change moves (p < 0.001); land, the largest landmass, the
high ground above 2 km and the coast with high ground do not (p 0.31 to 0.88). Against fl-pull the
high ground above 2 km (0.086 to 0.112, p 0.028) and the coast with high ground (0.198 to 0.233, p
0.050) rise, but base9's 24 runs (0.114 and 0.249) show that fl-pull's seeds 1 to 4 sat low: the
step moves neither item 1 check. The plates against fl-pull (matched 8, not read against base9):
27.3 alive against 24.7 (p 0.021), the sum of perimeters over the cap perimeters 1.758 against 1.806
(p 0.010; Bird 1.510), spreading 0.408 of the boundary against 0.398 (p 0.007), the ocean-ocean
share of the subducting front 0.483 against 0.453 (p 0.002; Stern & Scholl 0.35), the mean
convergence across the oceanic front 40.4 mm/yr against 37.6 (p 0.010; Bird 53.2). The plate check
passes 195 of its 264 run-checks (33 checks on 8 runs) against fl-pull's 181; `continental_min`
passes on none of the 8 runs (0.328 against 0.357; band 0.37 or more), the crust loss again. The
guard: worst 10,169 m, no step beyond 2 reliefs, the plate cells' largest excess over Airy 2,318 m
against fl-pull's 2,923 and base9's 2,753 (p 0.010 against each).

**Recommendation.** On by default (done in this branch): the law is the arc standing over its slab,
the test bounds hold, the band on the planet is what the straight-front test gives, and nothing
diverges or binds. The crust loss it brings is the arcs' wrap no longer making crust past the slab's
ends, and it is not to be made up with a knob. What is left between the band and the arcs' rate is
the lattice's metric along the front (the band 0.89 of the straight front, 11 % short), which the
coordinator ruled out of this step.


### Step 11: the margin stops at the slab's ends (PLT-562, 2026-10-24)

Step 10 stopped the arcs' magma at the slab's ends and left the cordillera, the subduction uplift
and the forearc on the whole distance field, so they still ran round each end of a front as the
arcs had. The coordinator's call: they stop at the slab's ends too, by step 10's test; the unit test
first on the same straight-front fixture, then the law, on by default if the test holds; one
cumulative 8-run arm, read against base10 (master's 24 runs on 37c894cc0de0: fl-ends' 8 and 16
more) and against fl-ends, its matched 8.

**The physics.** Step 10's sources show that past a slab's edge there is no slab (Thorkelson &
Breitsprecher 2005, p. 27; Johnston & Thorkelson 1997, abstract; Levin et al. 2002, abstract), and
each process this step stops is driven by the slab in the model's own laws: the subduction uplift is
Cortial's u = u_0 f(d) g(v) h(z̃), a function of the distance to the front over the slab (PLT-036);
the cordillera thickens a continental arc above an oceanic slab by crust shortened into it (PLT-048,
PLT-320); subduction erosion takes the forearc's crust at the plate interface, where the subducting
plate abrades it (PLT-441, Clift & Vannucchi 2004); one-way subduction holds the slab under the
forearc (PLT-542). With no slab there is no forearc, no arc crust thickening and no slab-driven
shortening. No source read states the margin's end at the slab's edge as a rule; it follows from
each of these standing over the slab, as the arc does (step 10).

**The law** (`tectonics.margin_slab_ends`, on by default; `subduction::fronts_finish`,
`subduction::erode_forearcs`). A sample the fronts' distance field reaches takes the subduction
uplift and the folds' increment only where it stands over its front's slab by step 10's test
(`subduction::over_the_slab`, now computed when either switch is on). The cordillera's range gets
no demand past the ends, and the crust behind a range past the ends gives none to it, so the
contact's budget (PLT-048) is met from the sources over the slab alone. The one-way subduction's
forearc (PLT-542) holds only what lies over the slab, and subduction erosion's band (PLT-441) takes
only from samples over it. The trench's retreat (PLT-522) is unchanged: it works on the erosive
front samples themselves. The subduction-zone flag, the convergence, the obliquity and the slab's
age `fronts_finish` writes on each sample it reaches still run on the whole distance field.
No constant of its own.

**The test's geometry, and the law's two tiers.** Step 10's test read a front's line over the arc
band's reach (`arc_distance_km` and two `arc_width_km`, 440 km by default). The uplift and the range
reach `subduction_distance_km` (1,800 km in the test's preset), and there a sample's label, its
nearest front sample by the lattice's distance, can lie along the front by some hundreds of km,
since the lattice's distance runs up to 2/√3 longer than the straight line by the edges' direction.
Two changes came out of the test. First, a sample's foot: from its label, the law steps front
sample by front sample (within two lattice edges) to the front sample nearest it on the sphere, and
reads the front's line there. Second, the line's reach: the lattice gives a front's direction at
its end only to a few degrees over the band's 440 km, where the normals turn, and that angle moves
the end's meridian by the sample's distance times it. Read over the band's reach, the uplift and the
range ran on up to 2.1 spacings past a 40-degree tilted front's end. So a sample within the band's
reach of the front reads the line over that reach, and one further out reads it over
`subduction_distance_km`, each with two lattice spacings more. A curved front's far field reads the
chord of its last `subduction_distance_km`.

**The unit test first** (`the_margin_stops_at_the_slabs_ends`; the switch, off and without its law,
and the test committed first, 0e01e4bb; the law 23477400). Step 10's fixture (40,000 samples, 113 km
apart; `a_front_of_known_length`, plate 1 converging at 80 mm/yr under no sediment, an erosive
margin), fronts of 20 and 40 degrees on the equator and tilted 60 degrees, `arc_slab_ends` on in
both halves. Two runs per case. The first sets no cordillera, the arc 400 km behind the front (twice
the default: at this spacing a forearc band to 200 km reaches past the meridians no further than
the 1.25 spacings the front itself reaches, 0.50 to 1.46 measured, and the test could not tell on
from off) and an erosion of a hundred columns' spare crust per spacing of front, so that every
forearc's band goes down to the ocean's column; it measures the uplift, the folds, the forearc's
loss and the one-way forearc. The second sets the cordillera on and no erosion and measures the
range (each sample's change of thickness less its arc crust). The farthest change past the
meridians of the front's ends, in spacings:

| Front | The uplift and the folds, off / on | The forearc's loss, off / on | The one-way forearc, off / on | The range, off / on |
|---|---|---|---|---|
| equator, 20 degrees | 13.93 / 0.29 | 2.58 / 0.29 | 3.36 / 0.29 | 13.93 / 0.25 |
| equator, 40 degrees | 15.80 / 0.48 | 3.08 / 0.22 | 4.07 / 0.41 | 15.80 / 0.48 |
| tilted, 20 degrees | 13.81 / 0.05 | 2.53 / 0.03 | 3.38 / 0.03 | 13.81 / 0.05 |
| tilted, 40 degrees | 15.13 / 0.79 | 3.71 / 0.72 | 4.36 / 0.72 | 15.13 / 0.79 |

Off, 266 to 314 samples beyond 1.25 spacings past the meridians take the uplift and the range, and
7 to 17 the forearc's loss and the one-way hold; on, none. A spacing or more inside the meridians
the uplift, the folds, the forearc's loss and the one-way forearc are the same sample by sample. The
range there takes in what it took off (the gain 1.0000 of off's, the worst sample 0.0000 from it),
and the crust behind it thins 1.52 to 1.95 times as much (equator 20 degrees 1.868, 40 degrees
1.527; tilted 1.948 and 1.522): the contact's budget no longer draws on the crust round the ends.
The test bounds: off beyond 1.25 spacings, on none beyond them, the four measures inside unchanged,
the range's gain inside within 0.05 of off's at its worst sample, and the crust behind it thinned
no less than off.

The foot and the far tier changed step 10's arcs too. The farthest arc past the meridians in
`the_arcs_band_stops_at_the_slabs_ends`, step 10's value / now: equator 10 degrees 0.24 / 0.08, 20
degrees 0.29 / 0.13, 40 degrees 0.41 / 0.41, 60 degrees 1.02 / 0.92; tilted 10 degrees 0.38 / 0.16,
20 degrees 0.34 / 0.03, 40 degrees 1.01 / 0.72, 60 degrees 0.34 / 0.03. The band is the same to
three places but for the equator's 20 degrees (0.848 / 0.847 of the straight front). Two collision
tests' worlds move with the law: the weld test's scattered continents weld on 10 of the first 24
seeds and the supercontinent on 6 (the test keeps seed 1; the recording test keeps seed 2).

**The arm** (fl-margin, cumulative on step 10: the law on, everything else master's at 2dd50093;
8 runs, seeds 1 to 4, both starts, 16k samples, 1000 Myr, the same arguments as fl-ends, key
f42d7b12531b; read against base10, master's 24 runs on 37c894cc0de0, and against fl-ends, its
matched 8; medians, Mann-Whitney p in brackets):

| Measure | base10 (24) | fl-ends (8) | fl-margin (8), p against base10 / fl-ends |
|---|---|---|---|
| Land | 0.294 | 0.294 | 0.285 (0.59 / 0.88) |
| Largest landmass | 0.150 | 0.132 | 0.166 (1.0 / 0.80) |
| Continental crust under the sea | 0.246 | 0.205 | 0.252 (0.40 / 0.44) |
| Land above 2 km | 0.112 | 0.112 | 0.133 (0.070 / 0.23) |
| Coast with high ground | 0.226 | 0.233 | 0.233 (0.78 / 0.88) |
| Land under ice | 0.095 | 0.095 | 0.121 (0.59 / 0.80) |
| Collisions per 100 Myr | 1.80 | 1.65 | 1.85 (0.95 / 0.43) |
| Continental crust's net change | -0.068 | -0.077 | -0.073 (0.20 / 0.96) |
| Criteria met per run, median and mean | 5, 5.00 | 5, 5.00 | 5, 5.00 (1.0 / 1.0) |
| Sea over the datum after 100 Myr / at the end, m | 320 / 472 | 264 / 403 | 243 / 434 |
| The guard's worst, m from the sea | 10,082 | 10,169 | 9,978 (1.0 / 0.88) |
| Steps beyond 2 Earth reliefs | 0 | 0 | 0 |
| Plate cells' largest excess over Airy, m | 2,641 | 2,318 | 2,869 (0.59 / 0.083) |

No census measure moves against base10's 24 runs or against fl-ends (p 0.070 to 1.0); the land above
2 km comes nearest (0.133 against 0.112, p 0.070 against base10). The guard binds on no step of any
run and no step goes beyond 2 reliefs.

What the law takes away is the margin round the ends. The crust shortened into the cordilleras over
a run falls from 0.84 to 0.61 ×10⁹ km³ (medians, fl-ends against fl-margin), down 27 %: the
cordillera's distance field reached 800 km round each end of a front, a quarter disc beside each
end, and much of the range stood there. The forearcs' erosion barely moves (continental crust 0.148
to 0.142 ×10⁹ km³ over a run, other crust 0.245 to 0.243; the area book's `forearc` part -0.142 ×10⁹
km³ against base10's -0.144), since the band to 200 km reaches little past an end. In the area book (base10's 24
runs against fl-margin's 8) the collapse's part of the continental area falls from 0.310 to 0.276 and
the belts' volume over a run from 12.7 to 12.1 ×10⁹ km³. The arcs keep step 10's band (0.888 times the
arcs' straight front, fl-ends 0.891) and rate (14.4 to 13.9 km³ per km and Myr on continental crust,
32.4 to 31.3 on oceanic, the convergence factor included). One-way subduction now holds only over the
slab, so a little more of the crust that went under shows again (0.063 of it against 0.046 on
fl-ends, mostly where the cover left it: 0.050 against 0.033), and the escaped ocean floor is older
(74.5 Myr against 62.7). The plates against fl-ends (matched 8): 26.4 alive against 27.3 (p 0.34),
the sum of perimeters over the cap perimeters 1.763 against 1.758 (p 0.57; Bird 1.510), the
ocean-ocean share of the subducting front 0.494 against 0.483 (p 0.38), the mean convergence across
the oceanic front 39.3 mm/yr against 40.4 (p 0.28). The plate check passes 197 of its 264
run-checks against fl-ends' 195 (base10 567 of 792).

**Recommendation.** On by default (done in this branch): the law is the margin standing over its
slab, as the arc does; the test bounds hold, nothing diverges or binds on the pool, and no V1 measure
moves. The cordillera's lost crust is the range that stood round the slab's ends, not to be made up
with a knob.

**Open item, for a later step.** The subduction-zone flag and the convergence, obliquity and slab
age written on each reached sample still run on the whole distance field, so their readers (the
volcanoes and the deposits) see a subduction zone past a front's ends. They make no relief or crust
of their own; the coordinator's call (2026-10-25): leave them until those readers are checked, then
stop them at the slab's ends in a step of their own.

### The ocean-continent dip on base10's maps (2026-10-25, no simulation)

Step 8 left open what turns the mature ocean-continent fronts transform at a resampling, and step 9's
two-plate probe found no dip on a head-on margin. This reads base10's 24 runs (key 37c894cc0de0,
`+DEEPTIME_CRUST_BUDGET=1`) without a new run: the records over their 1,140 resamplings after the
first 100 Myr, and the 50-Myr maps, whose flags carry the boundary mix's fronts (256 on the
overriding side of a subduction front) and whose current geometry (the Delaunay triangulation of
the visible samples) is what a resampling rebuilds the lattice from.

**The records** (the step before a resampling against the first step on the new lattice, means):
the ocean-continent front 56,100 km, -12,900; ocean-ocean +3,600; judged a transform +12,400; rift
-9,100; ridge +8,800; the lost ocean-continent length is the slab's 600 km or longer (-14,800) and
the contacts' 50 Myr or older, as on base7. The plates' slab pull falls by 32 % and comes back over
the cycle; the visible-weighted speed does not move (about 21 mm/yr). The plate count's change does
not explain it.

**The maps**, binned by Myr since the last resampling (a 2-Myr map is the first step on the new
lattice), samples a map; a front is on the current boundary when it has a Delaunay neighbour of
another plate; ocean-continent as the mix splits it (continental crust of its plate within two
steps):

| Myr since the resampling | 2 | 4 to 6 | 8 to 12 | 14 on |
|---|---|---|---|---|
| maps | 47 | 98 | 100 | 115 |
| front on the current boundary | 1,204 | 1,243 | 1,212 | 1,185 |
| of it ocean-continent / ocean-ocean | 572 / 631 | 610 / 633 | 617 / 595 | 652 / 533 |
| front off the current boundary | 77 | 100 | 151 | 230 |
| of it ocean-continent / ocean-ocean | 43 / 34 | 56 / 44 | 95 / 56 | 171 / 59 |
| off fronts' distance to the nearest sample of another plate, mean km | 134 | 168 | 228 | 304 |
| of the off fronts, a hole (no sample within 70 km) on the way there | 0.006 | 0.069 | 0.181 | 0.293 |
| current boundary samples | 7,864 | 7,450 | 6,832 | 6,538 |
| holes (no visible sample within 70 km), 10⁶ km² | 1.5 | 8.6 | 22.7 | 41.0 |
| of them within 190 km of both a trench and a front sample | 0.12 | 0.38 | 0.66 | 0.92 |

The maps show the records' dip: the ocean-continent front 823 a map late in a cycle against 615 on
the new lattice (-25 %; the records -23 %), ocean-ocean 592 against 665.

- **The fronts on the current boundary do not dip** (1,185 against 1,204). Within them 80 a map
  move from ocean-continent to ocean-ocean (652 to 572, 533 to 631): fronts the new lattice puts
  farther than two steps from their plate's continent. The records' ocean-ocean fronts move off the
  coast the same way (the 150 km band -3,100 km, the 300, 600 and 1,200 km bands +2,500, +2,200
  and +1,600).
- **The rest, three fifths of the dip (128 of 208 a map), is fronts off the current boundary**:
  overriding samples that `subduction::fronts_begin` and `dynamics::boundary_mix` still class a
  front through a lattice neighbour of the other plate (both take such a neighbour whether it is
  hidden or not), with no sample of another plate beside them now. They build up through the cycle
  (6 % of the front two Myr after a resampling, 16 % late; 7 % to 21 % of the ocean-continent
  front), lie deeper behind the boundary as it goes (134 to 304 km), and the new lattice drops
  them. They are a band behind a real front, not a second front: 99 % of those with only their
  own plate on the way have an on-boundary front sample between them and the other plate, 90 % are
  of their plate's biggest group. Late in a cycle 29 % face a hole, crust of the subducting plate
  that went under and that no overriding sample covers (0.9 × 10⁶ km² of hole beside both a trench
  and a front, against 0.12 on a new lattice), which a resampling fills with new ocean floor.
- **The gain in the boundary judged a transform is not on the maps** (they carry no class but the
  fronts). It comes with the current boundary's return at a resampling (6,538 boundary samples a
  map late in a cycle, 7,864 on the new lattice), when the holes, 41 × 10⁶ km² late in a cycle and
  59 % of them beside divergent samples, are filled with ocean floor and ridge segments. So step
  8's "to the boundary judged a transform" is a match of totals, not shown margin by margin.

So the late-cycle ocean-continent front and the slab pull read on it carry a band the frozen
lattice keeps behind the margin, up to a fifth of that front. A likely source, not shown: one-way
subduction (PLT-542) keeps the subducting plate's crust hidden once it went under, so where the
convergence is oblique to an irregular margin its trench follows the envelope of the overriding
outline swept along the motion, and the overriding samples in the lee keep only hidden neighbours
of it. The maps hold visible samples only, so whether the band's neighbours are hidden or moved off
is not in them; that, and the margins followed through a resampling, needs a trace of the lattice
at a few resamplings of one run (measurement only), which is the coordinator's to call. The
scripts are `build/front/recdip.py`, `recdip2.py`, `mapdip.py` to `mapdip4.py` (not kept).

### The front trace (2026-10-25, measurement only, PLT-580)

Two questions the maps cannot answer, asked of one run behind one variable,
`DEEPTIME_FRONT_TRACE=<folder>` (`plates/front_trace.rs`); no rule reads either, and the world is
the same with it on or off (the test `the_front_trace_changes_nothing`; on the pool, `pool.sh alias`
of the commit against master's key).

- **The dip trace**: are the band's front neighbours hidden? At the step a resampling ends, the step
  after it and halfway through the next cycle, each visible sample with a lattice neighbour of
  another plate gets a row: its class from the boundary mix, the front `subduction::fronts_begin`
  found for it (that neighbour hidden or not, its distance now, the convergence along the
  boundary's normal, the contact, its slab's length and age), the neighbour the mix's front reads,
  and the distance to the nearest visible sample of another plate.
- **The thickening attribution**: which process last took a continental column past 45 km (the
  lower edge of the crust budget's 45-55 km bin; a measurement threshold, game-choice), and what
  has thickened it since. The step's processes are bracketed as the gap budget brackets them; a
  resampling hands each new sample the record of its blend's major corner and books the blend and
  the crust accreted under it (the stack) as two parts of their own. Every 50 Myr a file holds the
  visible continental samples with their record, orogeny tag and age, lithosphere and keel.

#### What the trace found (supercontinent seed 1, 300 Myr, 2026-10-25)

One run (`pool.sh run fl-trace`, key 188b5e2f2852, aliased to master's base12 runs), read with
`build/front/trdip.py`, `trdip2.py` and `tratt.py` (not kept). The lattice spacing is about 60 km.

*The dip.* The band is real and it is the frozen lattice. Taking "on the boundary" as a visible
sample of another plate within 1.5 spacings now, 91 % of the off-boundary fronts read a hidden front
neighbour, and so do 90 % of the on-boundary fronts by a cycle's end; the frozen front neighbour
stands 1,015 km away now (median) off the boundary and 704 km on it at a cycle's end, against 60 to
70 km on a new lattice. The ocean-continent fronts per step, and what a rule on the current geometry
would keep:

| rule | end of a cycle | new lattice | mid-cycle | change at a resampling |
|---|---|---|---|---|
| today's (a lattice neighbour of another plate) | 708 | 498 | 590 | −29.7 % |
| a visible lattice neighbour of another plate | 94 | 97 | 77 | |
| a visible sample of another plate within 2 spacings now | 485 | 469 | 510 | −3.4 % |
| within 3 spacings | 551 | 494 | 558 | −10.3 % |

On a new lattice 97 % of the ocean-continent fronts have a visible sample of another plate within
2.25 spacings. Two spacings is the rule the coordinator took on 2026-10-25 (below, "The front on the
current geometry").

*The thickening.* At 300 Myr the land above 2 km is 0.125 of the land: 4,113 samples, of them 2,813
Himalayan, 546 Andean and 754 untagged. 377 untagged columns stand at 49 to 51 km, T_c (50 km in
the run), every one above 2 km: 93.6 % crossed 45 km in the founder (`collapse::founder`, the
excess over `crust_max_km` spread to continental room thinner than T_c, filled up to T_c), and 87 %
of what they gained since came from it; their peak was 50.0 to 50.3 km. Their lithosphere is 61,
78 and 137 km (quartiles), keel 0: hot crust, not cratons (the untagged 35 to 45 km columns are at
225 km). The founder's receivers keep no orogeny, so PLT-415's tag gate leaves them where it put
them. The coast with high ground (land within 1.6 spacings of the sea with land above 2 km within
200 km; v1census's landmass test left out) is 0.231: its nearest high ground is Himalayan in 66 %
(stamped 0, 0 and 14 Myr ago, quartiles), untagged in 22 % (parked at T_c in 5 %) and Andean in 12 %,
68, 80 and 134 km thick. The coast cells themselves carry no front flag; 20 % carry the foreland
flag. Coastal high ground is mostly young collision belts running out to the sea, not the parked
columns.

### The collapse reads the thermal state (2026-10-25, PLT-581)

*Is the tag gate physics?* No: a stand-in. PLT-415 says why crust no orogeny raised does not flow:
"the audit's minimal strength field: cratons and old belts stay". The tag stood for the crust's
strength. Since ISO-112 the sheet's viscosity already reads the lithosphere's thermal age in place
of the orogeny's, so the gate is the last place where the tag stands in for the thermal state, and
it is wrong where the two part: crust thickened by something other than an orogeny (the founder's
receivers, the lower-crustal flow of PLT-300, which carries no tag either) over a hot lithosphere.
Reading the thermal state directly is the cleaner fix than handing the tag on (the coordinator's
call of 2026-10-25: the thermal reading, not the inheritance).

*The source.* Clark & Royden (2000, full text read 2026-10-25) fit the topography of eastern Tibet
with flow in a lower-crustal channel and draw where it goes by the crust's strength, not by its
history. Fig. 5's caption (p. 706): "Areas shaded in dark gray represent regions of cold, strong,
continental material; light gray area represents intermediate strength; and white areas represent
weak crustal regions. Thus, lower crust escapes from beneath thickened, elevated central plateau
through regions where crust is weak." The Sichuan Basin is the strong case (p. 706): "the strength
of the crust beneath the Sichuan Basin inhibits flow of lower crustal material from central Tibet".
Their best fit had the channel "uniformly weak at the initiation of flux into the channel,
suggesting that the low-gradient margins of Tibet were already weak before the onset of crustal
thickening" (p. 705): the margin's crust flowed because it was weak, before any orogeny thickened
it. Beaumont et al. (2001, author manuscript read 2026-10-25) tie the weakness to heat: the channel
is 10¹⁹ Pa s where it is at 750 °C or more (p. 1), and "Isotherms are advected pro-ward by channel
flow" (p. 2), so hot crust that flowed out of a belt is still hot.

*The law.* With `tectonics.collapse_sheet_thermal` (on by default) and the thermal state and
strength on, the sheet's gate on the orogeny goes: crust no orogeny raised thins at ε̇ = ΔU / (4 η L)
too, with η = η_0 e^(t_eq / τ_s) at its lithosphere's thermal age (PLT-414, PLT-415, ISO-112). A
craton at L_cold has an infinite thermal age, so an infinite η: it stays rigid, as the gate kept it,
now because it is cold. A sample the state has not reached (L = 0) keeps the gate. Off, or without
`thermal_strength`, the sheet is PLT-415's as before.

*The test.* `the_founders_receivers_flow_where_the_lithosphere_is_hot` (`plates/collapse.rs`): a
belt 95 km thick inside a continent on a 78 km lithosphere (the parked columns' median), a craton at
L_cold beside it and a thinned margin as room. The founder fills its receivers to exactly T_c with
no orogeny; gated (the switch off), every receiver is still at T_c after 100 Myr while the belt
thins past it; with the switch, the hot receivers thin below T_c − 1 km and the craton's stay at
T_c. The rate itself: a 15 km excess flows on a 78 km lithosphere, not on L_cold, not without a
state, not with the switch off.

### The front on the current geometry (2026-10-25, PLT-582)

*The rule.* With `tectonics.front_current_geometry` (on by default), `subduction::fronts_begin` and
`dynamics::boundary_mix` read an overriding sample's lattice neighbour of a subducting plate (a
plate with an active contact under the sample's plate) as a front only while a visible sample of
that plate stands within `subduction::trench_row_rad` of it now. The radius is two lattice spacings,
computed from the run's spacing: one spacing for the step to the neighbour across the boundary, and
one for the trench's single hidden row. The overlaps hide the subducting plate's edge sample once it
crosses the overriding outline (PLT-553), so the sample that has just gone under is hidden and the
next one in is still visible. The radius is not tuned to a length. On a new lattice, where every
front's neighbour is beside it, 97 % of the ocean-continent fronts meet 2.25 spacings (the trace
above). The change at a resampling falls from −29.7 % to −3.4 % at two spacings; three spacings
keep more of the band (−10.3 %).

The query runs on the step's tree of every sample, the one the overlaps search. It is asked only of
samples that have such a neighbour, and once per plate met. The mix drops a converging neighbour by
the same test, so a sample with nothing else across it is no boundary. It asks nothing of an
opening or a transform between the same two plates: the gap an opening leaves between resamplings
is the opening's, which the next resampling fills, and the boundary is there (the arm below found
the mix asking every neighbour, and `an_opening_boundary_of_a_subducting_plate_stays_in_the_mix`
holds it to this). The full adjacency rebuild (the lattice's edges from the current geometry) stays
a follow-up, for a residue the trace can see; the arm found none.

*The plate asked.* The trace's rule asked for a visible sample of any other plate; the code asks for
the subducting plate itself. A third plate's crust beside an overriding sample is no slab of the
plate the front reads, and counting it would keep a front where that plate has gone. The difference
was not measured on the trace (the front lane's call, which can be undone).

*The tests.* `an_oblique_promontory_through_a_resampling` was written first. It drives an ocean at
50 mm/yr, 30 degrees oblique, onto a continent with a promontory 4 spacings deep and 8 wide, at
80,000 samples, so that the forearc's reach covers the promontory. The ocean moves 12 spacings and
the lattice is then resampled. The test was meant to show the band, and found none:

- 137 fronts at every step before the resampling and 131 after it, the same with the rule off and
  on;
- the mix's ocean-continent boundary 10,938 km before and 10,459 km after.

The forearc holds the ocean that goes down the trench along the margin (PLT-542). The promontory's
corner, though, has no front and so no forearc. The ocean passing under it is not held, and the lee
fills again from the ocean coming in behind, so every front keeps a visible ocean sample beside it.
At 45 degrees the band was one front of 193 at three steps of eight. So the test holds the rule to
what it must not do: on a live margin, oblique and irregular, it keeps every front. It does not show
which of the planet's margins carry the trace's band.

`a_consumed_slab_leaves_no_front` shows the band. It drives an ocean plate 3 spacings deep and 30
degrees long (`a_front_of_known_length`) north under a continent at 50 mm/yr, at 20,000 samples,
until all of the plate is under. The contact stays active, because the slab's hidden samples still
overlap the continent. The fronts per step, off and on:

| step | 1 to 3 | 4 | 5 | 6 to 9 |
|---|---|---|---|---|
| off | 24 | 24 | 24 | 24 |
| on | 24 | 22 | 15 | 0 |

- **Off**, the margin keeps all 24 fronts (3,832 km of ocean-continent boundary in the mix) to the
  end, 8 Myr after the last of the slab went under.
- **On**, the fronts fall as the slab's trailing edge reaches the trench, and are gone once it is
  under; the mix has 0 km.
- The live margin of the first steps keeps all 24 fronts either way.

`an_opening_boundary_of_a_subducting_plate_stays_in_the_mix` holds the mix to the fronts' reading.
On the plates of `converging_plates`, the ocean has turned 4 spacings about the x axis, so it runs
under the continent on one half and opens a gap on the other. The contact is active. The mix's rift
is 21,556 km with the rule off. With the rule asking every neighbour of the subducting plate, as
the arm ran it, the rift was 12,774 km. Asking only a converging neighbour, it is 21,556 km, and the
ocean-continent front is 21,715 km either way.

### The stage's arm: the collapse and the front (2026-10-25, PLT-581 and PLT-582)

The arm `fl-n2th` (source key d9242330d508) ran both switches on, as one stage: both starts, seeds 1
to 4, 1,000 Myr, with the crust-budget env. It is read against base12's matched 8
(`ct-base12-s1to4`). The table gives medians, with the Mann-Whitney p in brackets:

| measure | base12, matched 8 | arm |
|---|---|---|
| land | 0.309 | 0.311 (0.96) |
| largest landmass | 0.204 | 0.199 (0.96) |
| continental crust under the sea | 0.198 | 0.186 (0.51) |
| land above 2 km | 0.128 | 0.114 (0.80) |
| coast with high ground | 0.230 | 0.218 (0.72) |
| land under ice | 0.100 | 0.126 (0.38) |
| collisions per 100 Myr | 1.95 | 1.60 (0.46) |
| crust's net change | −0.065 | −0.072 (0.80) |
| the guard's worst, m | 10,105 | 10,632 (0.28) |
| plate excess over Airy, m | 2,782 | 2,641 (0.65) |

- **Against base12's 24 runs**, land above 2 km goes from 0.128 to 0.114 (p 0.56), and coast with
  high ground from 0.229 to 0.218 (p 0.48).
- Both arms meet 5 criteria per run. Neither has a step beyond two Earth reliefs.
- The sea over the datum is 250 m after 100 Myr and 352 m at the end, against 297 m and 442 m.

Nothing moved significantly, so the switches were not split into separate arms.

*The collapse.* Continental columns at T_c (49 to 51 km of crust) that no orogeny raised, on the
t = 0 map, are the founder's parked receivers. base12's runs have 262 such samples (the mean; the
median is 274), 0.84 % of the land. The arm has 149 (median 110), 0.47 % of the land. Most of them
stand above 2 km in both arms: 244 of base12's and 145 of the arm's.

*The front at a resampling.* This is the mix's ocean-continent boundary at the step of a resampling
and the steps after it, over 378 resamplings (base12) and 379 (the arm).

- **base12.** 55,829 km at the resampling, then −21.9 % one step later and −14.0 % three steps
  later. Through a cycle it grows from 43,868 km (1 to 2 Myr after a resampling) to 56,073 km at the
  next (+28 %). The loss at a resampling is in fast-converging fronts (40 mm/yr or more across the
  boundary: −11,051 km), the band reading a neighbour that went under and on.
- **The arm.** 38,806 km at the resampling, then +5.8 % and +12.5 %. Through a cycle it runs from
  41,311 km to 38,997 km (−6 %). The fast-converging change is +164 km.

What the arm gains at a resampling is new contacts, with no slab yet: +3,078 km, where base12 gains
+3,187 km. So the band is gone, and nothing is left that calls for the full adjacency rebuild.

*Trench share of the coast.* M5's attribution reads the share of the coast within 200 km of a
trench on the t = 0 maps. It is 0.238 for base12's matched 8 and 0.256 for the arm (medians,
p 0.33). Earth's is 0.122, so the rule leaves the excess where it was.

*The mix's records.* Through each cycle, the arm's records lost a fifth of the ridge length: from
103,703 km at 1 to 2 Myr after a resampling to 84,222 km at the next, where base12 held 108,000 to
116,000 km. The mix had asked the rule of every neighbour of a subducting plate, including one that
opens. It now asks only a converging neighbour. The mix is measurement only, so the arm's planet
stands as measured, but its records' ridge, rift and other lengths are the old reading's.

### Step 12: incipient subduction (PLT-610 to PLT-614) and the Benioff census (PLT-615) (2026-10-25)

The model made every front a working subduction zone from its first step: a contact found by the
overlaps pulled with its slab, built an arc and counted as subducting boundary at once. On Earth a
new subduction zone is forced before it sinks by itself. Gurnis, Hall & Lavier 2004 (G-cubed
5:Q07001, read in full) model forced convergence across a fracture zone: the resisting force
grows "as the plate begins bending, reaching a maximum value dependent on plate thickness, but
typically ranging from 2 to 3 × 10^12 N/m for cases that become self-sustaining", and
"Cumulative convergence, not the rate of convergence, is the dominant control", the critical
convergence "only varies between 100 and 130 km (Table 3)". Hall et al. 2003 (abstract): "a
fracture zone could be converted into a self-sustaining subduction zone after approximately 100 km
of convergence". Crameri et al. 2020: "The progression from no subduction to subduction, marked by
a slab reaching around 100 km depth". And Bird 2003 [126] defines his SUB steps by "a Benioff zone
of intermediate to deep earthquakes and/or a parallel volcanic arc with Quaternary activity"; his
other convergent oceanic steps are OCB (17,449 km, 6.7 % of the boundary).

**The law** (`tectonics.incipient_subduction`, PLT-610). A front whose contact's slab, before the
step, is shorter than 100 km (PLT-611, the bottom of Gurnis's table, as transference's PLT-518)
is incipient:

- The force balance adds the bending's resistance against its convergence, 2.5 × 10^12 N/m of
  front (PLT-612, the middle of Gurnis's range), or 5 × 10^11 N/m where the floor going under is
  younger than 10 Myr (PLT-613: "The entire initial hump in resisting force is eliminated when
  lithosphere with a typical thermal model of a spreading center is subject to compression", the
  ridge's topography adding "an additional 5 × 10^11 N/m"), both scaled with gravity as the slab
  pull. The force is plastic (it does not grow with speed), so it enters the linear solve as a
  collision's resistance does: a coefficient of the force over max(convergence, 1 mm/yr) (FRC-011).
  It is booked in the record's collision resistance.
- It builds no arc (`subduction::fronts_finish`: no arc magma and no arc band weight).
- The boundary mix counts it as class other, converging, in the record's new
  `front_bands.incipient` (not in `other_converging`, nor in the slab, normal and age bands), and
  the margins count no subduction margin at it (PLT-614).

The slab grows by the convergence as before (FRC-020), so the stage ends at the first step whose
slab starts at 100 km or more. A subduction initiation's planted slab (PLT-270, 150 km) starts past
it, so the model's spontaneous initiations are not held back: they stand for a zone that has
already passed its forced stage. The slabs are kept by plate pair (`PlateState::slabs`), so a
resampling's re-found contact keeps its stage.

*Speed-dependent terms (for the force balance lane).* The incipient link is a plastic force
linearised at the step's convergence, floored at 1 mm/yr: on Forsyth & Uyeda's balance it would be
`add_friction`, a force whatever the speed.

**The census column (B)** (PLT-615, measurement only). Beside `sub`, which counts every front past
its incipient stage, the record now counts the subducting fronts whose slab reaches 100 km under
their arc (`front_bands.benioff`): a slab at least hypot(`arc_distance_km`, 100) km long, 223.6
km at the presets' 200, a straight slab from the trench to that depth under the arc's axis.
`measure_boundary.py` version 3 adds `incipient`, `ocb` (no contact plus incipient, Bird's OCB)
and `sub_benioff`; `transform` no longer counts the incipient fronts. The depth is read (Crameri
2020; Syracuse & Abers 2006, abstract: "H ranges from 72 to 173 km with a global average of 105
km"); the arc's distance is PLT-042's 200 km, which is `uncited`, so the column inherits its flag.

**The tests** (`plates/tests.rs`, 20,000 samples):

- `an_incipient_front_holds_its_plate_and_is_no_subduction_zone`: the reference plate with a
  40-km slab (its pull about 1.7 × 10^12 N/m, under the resistance). On, the trench moves at 0.59
  mm/yr in the balance (under the 1 mm/yr floor: a plastic force larger than the drive stalls the
  plate), the mix has 3,832 km of incipient front and no ocean-continent front, and no coastal
  sample of 127 is a subduction margin. Off, 3.03 mm/yr, 3,832 km of ocean-continent front, 31
  subduction margins.
- `cumulative_convergence_ends_the_incipient_stage`: the reference plate pushed at 10 and at 25
  mm/yr from no slab (Gurnis's forced models), 2-Myr steps. The resistance is the same at both
  rates (8.145 × 10^18 N over the front). It acts while the slab starts under 100 km and stops at
  the first step that starts past it (114.6 km at 10 mm/yr, 143.3 km at 25). The balance's speed
  then jumps from 6.28 to 10.60 mm/yr (10 mm/yr) and from 9.25 to 14.64 (25), more than the slab's
  growth alone gives. On a straight front, the arcs over a 90-km slab: 0 m of magma on, 6,486 m
  off; over a 110-km slab the arcs are the same on and off, bit for bit.
- `a_ridge_under_compression_resists_less`: floor 5 Myr old resists a fifth of the 80-Myr floor's.
- `an_initiations_planted_slab_is_past_the_incipient_stage`: every built-in preset plants at least
  100 km; with the planted 150 km the reference plate has no incipient front.
- `a_slabs_length_survives_a_resampling`: the head-on plates through carry, resampling and the
  overlaps found again. With a 700-km slab the ocean-continent front holds (21,715 km before,
  22,993 km after) with no incipient front; with none, the whole front is incipient on both sides.
- `incipient_subduction_changes_nothing_past_its_stage`: with a full slab the balance, the slab,
  the mix and the arcs are the same on and off, bit for bit; off, a 40-km slab has no incipient
  front.
- `an_incipient_front_takes_no_forsyth_uyeda_friction` (the guard with PLT-585, below): with both
  switches on, plate 1 under plate 0 on a 3,258 km front, the incipient front takes no friction
  between the pair (0 N either way) and the incipient resistance alone (8.1446 × 10^18 N, the same
  with PLT-585 off); once mature, the front takes F_CR × its length (2.432 × 10^18 N, F_CR 7.465 ×
  10^11 N/m) and no incipient resistance. The other pairs' friction is the same in both cases.

**With Forsyth & Uyeda's balance (PLT-585).** That lane landed first, so the guard is this lane's
(plate-forces.md "With incipient subduction", FRC-035): while a contact is incipient its oceanic
front samples take the incipient resistance alone, and the loop over the other boundaries skips an
incipient partner from either side. A continental sample on an incipient front keeps its friction:
it is a collision's drag, as at a mature contact (the agent's choice, 2026-10-25). With the switch
off the set of incipient pairs is empty, so the guard changes nothing.

**On by default** (2026-10-25, the rule that physics goes on once it is correct). The flip moved
which seeds weld on stall: the release sweep (`weld_seed_sweep`, 8,000 samples, 300 Myr, 24 seeds
per start) welds on 9 scattered seeds and 8 supercontinent seeds with incipient subduction on
(11 and 5 before), so `collisions_weld_on_stall_and_transfer_on_shortening` now uses scattered 2
and `sutures_and_initiations_keep_continents_whole` supercontinent 5; the five fixture tests that
build a front with no slab pin the switch off, as their geometry is a working subduction zone.

The pool arm `fl-incip` (8 runs, seeds 1 to 4 on both starts, `earthlike_quick_16k`, 1000 Myr,
`+DEEPTIME_CRUST_BUDGET=1`, key a963c9ac56ee: this branch on master 17ab80b7, PLT-585 off and
PLT-540 on) against base13 (24 runs, key 89e867370c1f), medians and the two-sided Mann-Whitney p
(`pool.sh table --base base13 --arm fl-incip`, measures `v1census`, `guard` and `boundary` version
3). Every run ran its 1000 Myr; no step beyond 2 Earth reliefs on any run.

| Measure | base13 | fl-incip | p |
|---|---|---|---|
| land | 0.313 | 0.310 | 0.95 |
| largest landmass | 0.166 | 0.177 | 1.0 |
| continental crust under the sea | 0.214 | 0.215 | 0.98 |
| land above 2 km (realism ≤ 0.07) | 0.097 | 0.077 | 0.40 |
| coast with high ground (≤ 0.11) | 0.196 | 0.187 | 0.51 |
| land under ice | 0.083 | 0.098 | 0.85 |
| collisions per 100 Myr | 1.70 | 1.55 | 0.60 |
| V1 criteria met per run (median, mean) | 5, 5.38 | 5.5, 5.50 | 0.42 |
| the guard's worst, m | 10,047 | 10,020 | 0.65 |
| plate cells' largest excess over Airy, m | 2,411 | 2,511 | 0.27 |
| subducting, share of boundary (Bird SUB 0.197) | 0.291 | 0.250 | < 0.001 |
| incipient fronts | | 0.035 | |
| converging with no subduction zone (Bird OCB 0.067) | 0.006 | 0.041 | |
| slab 100 km under the arc, column (B) (Bird SUB 0.197) | | 0.224 | |
| oceanic front under 10 / at 80 or more mm/yr | 0.130 / 0.073 | 0.087 / 0.082 | |
| convergence across the oceanic front, mm/yr (Bird 53.2) | 38.3 | 41.1 | < 0.001 |
| active share of continental margin (0.25 to 0.40) | 0.379 | 0.337 | < 0.001 |
| boundary criteria met per run (median, mean) | 2, 1.88 | 2.5, 2.75 | 0.004 |

Base13's `incipient` and `sub_benioff` are blank: its record has no such bands. The M4 lane's split
(`tools/review/m5diag/m4split.sh`, the land above 2 km by class, means over the runs; Earth's from
its table on base13) reads where the high ground went:

| Class, share of the land | base13 | fl-incip | Earth |
|---|---|---|---|
| all above 2 km | 0.0978 | 0.0833 | |
| collision belts under 10 Myr | 0.0368 | 0.0405 | 0.026 |
| Andean margins under 10 Myr | 0.0137 | 0.0090 | 0.0085 |
| older Andean margins and untagged arcs | 0.0159 | 0.0116 | about 0.0055 |

The young Andean high ground falls to Earth's: a new front builds no arc until its slab reaches
100 km. The collision belts do not move (8 runs against 24), and the older
Andean margins and arcs stay about twice Earth's.

Off, the planet is base13's bit for bit: one pool run of the merged head (b935cea7, key
ac619d59fdab, `--set tectonics.incipient_subduction=false`, supercontinent 1, base13's other
arguments) has the state_sha256 (adf20d4c...) and fields_sha256 of base13's run. Its rec_sha256
differs, as it should: the record gains the `incipient` and `benioff` front bands.


## Plate speed (2026-10-25)

The coordinator's brief for the plate speed lane: CHK-020's `speed_rms_mm_yr` (the plate check's
area-weighted RMS plate speed, averaged over steps) reads about 26 mm/yr on master against its
sourced band of 40 to 100 (Earth about 47 today from Forsyth & Uyeda 1975's Table 1, 50 to 100 over
the last 200 Myr from Zahirovic et al. 2015). It read 41.4 on 2026-10-05, 28.1 on base7 and about 26
on base9 and base10. Find why, from cached runs and unit tests (no new simulations), and fix it
only if the fix is sourced physics or a bug fix: no knob, cap, gain or forcing term (Ben: "Results
that don't align with earth but are still perfectly playable and physics backed > Results that are
forced to align with earth by introducing extremely weird forcing functions").

**The answer.** The level is not a bug and not one merge. It is the force balance's open-loop
calibration of 2026-10-02 (the speed scale 45 mm/yr, PLT-181, which its own row says was "to be
calibrated in task 8 against CHK-020" and never was; the slab and boundary resistances 3 and 3,
PLT-185 and PLT-189, chosen against the open-loop speed targets), with each merge since adding a
resistance or taking a drive away and nothing calibrated again. The largest single fall is the
saturating suction (PLT-540), whose premise no source read supports. Nothing is changed here; the
recommendation at the end is the coordinator's and Ben's to take.

### The level as the design sets it

FRC-016 defines the basal drag coefficient c_b by a reference plate: an oceanic plate of the
average starting plate's area A (a fourteenth of the sphere), pulled by a straight 3,000 km trench
of a full-length 80-Myr slab, its neighbours at rest, moves at the speed scale v_s:
(1 + s + b) c_b A v_s = F_sp L, with s = 3 (slab resistance, PLT-185), b = 3 (boundary resistance,
PLT-189) and v_s = 45 mm/yr. The resistances per metre follow (FRC-017): slab resistance s c_b A / L
per metre of trench against the speed towards it, boundary resistance b c_b A / (2 √(π A)) per metre
of boundary against the relative velocity of the two plates. The slab's terminal velocity is then
v_s (1 + s + b) / s = 105 mm/yr. Every speed in the balance scales with v_s (the drag goes as its
inverse), so the level is linear in it: the diagnosis below gives ×1.49 for ×1.5.

### The bisection (cached pool runs)

Each run's CHK-020 `speed_rms_mm_yr` from the pool's plate check (`earthlike_quick_16k`, 1000 Myr,
both starts), medians over the matched runs, two-sided Mann-Whitney p (build/speed/speedlevel.py,
`pairs`). `v_slab` and `v_free` are the check's speeds of the plates with and without a slab.

| Step (merge) | Runs | RMS, mm/yr | p | What moved |
|---|---|---|---|---|
| trench-base883 to base | 12 | 37.0 to 37.4 | 1.0 | |
| base to base2 | 24 | 37.1 to 36.0 | 0.23 | |
| base2 to base3 (V1 gap 2) | 24 | 36.0 to 34.4 | 0.038 | v_slab 53.5 to 50 (p < 0.001) |
| base3 to base4 (front share, fission, level set) | 24 | 34.4 to 35.2 | 0.39 | |
| **base4 to base5 (PLT-540, the saturating suction)** | 12 | **33.8 to 28.9** | **< 0.001** | **v_free 30.6 to 23.7 (p < 0.001); v_slab unchanged** |
| base5 to base6 (V1 gap 5, one-way subduction) | 12 | 28.9 to 27.1 | 0.019 | fl-oneway against base5: no change (p 0.89) |
| base6 to base7 | 24 | 27.3 to 28.1 | 0.08 | |
| base7 to base8 (PLT-553) | 24 | 28.1 to 26.8 | 0.023 | |
| base8 to base9 (PLT-560, the pull along the normal) | 24 | 26.8 to 25.9 | 0.005 | v_free 23.3 to 20.0 |
| base9 to base10 (PLT-561, the arcs' band at the slab's ends) | 24 | 25.9 to 27.0 | 0.019 | |
| trench-base883 to base10 (the whole span) | 12 | 37.0 to 27.4 | < 0.001 | |

The arms that split the steps: base4 to fl-sat (PLT-540 alone, 8 runs) 34.1 to 27.5 (p 0.001);
base3 to fl-share 36.8 to 34.1 (p 0.018); fl-fission to fl-fission2 34.1 to 36.2 (p 0.052); dock-c3
to dock-T 36.8 to 28.9 (p 0.001); base4 to fl-ts1 34.1 to 30.9 (p 0.001). Before the pool the level
was 41.4 (2026-10-05) and the crust budget took it from 41.6 to 36.5 (2026-10-17).

Within a run (the records' RMS per 100 Myr window, build/speed/timeline.py) base10 falls from 33.0
in the first 100 Myr to 26.5 by 300 Myr and stays there (25.2 to 26.6); trench-base883 from 40.8 to
about 35. The boundary per step is 226,000 to 330,000 km on base10 (260,000 to 550,000 on
trench-base883); late in base10's runs the resisting torques split basal drag 0.12, slab resistance
0.23, boundary resistance 0.39, collision resistance 0.26.

### The reference plate term by term (the lattice)

`the_reference_plate_on_the_lattice_term_by_term` (plates/tests.rs, 40,000 samples) puts FRC-016's
reference plate on the lattice with `front_pull` on and compares each term's torque in the record
with the continuous cap's (FRC-016's definition on the sphere: basal drag over the cap, slab
resistance along the trench's normal, boundary resistance against the full relative velocity all
round the rim, by quadrature). The lattice moves the plate at 53.2 mm/yr, the continuous cap at
48.7 (the scale is 45; the flat definition on a spherical cap). At the lattice's answer the record
over the cap is: basal drag 0.977 (the plate is 0.976 of A), slab resistance 1.055 and the pull
1.060 (the trench's shares, PLT-560), boundary resistance 0.875 (its metres 0.886 of the rim's
20,615 km: the lattice under-counts the rim). The resisting shares on the lattice are 0.150, 0.487
and 0.362, on the cap 0.149, 0.449 and 0.402 (the flat plate's 1/7, 3/7, 3/7). **The lattice moves
the reference plate faster than the definition, not slower**: no counting of a term on the lattice
slows the planet's plates.

### The planet term by term

`what_sets_the_planets_plate_speed` (plates/tests.rs, ignored; build/speed/diag3.sh on the VM): the
pool's preset `earthlike_quick_16k` at its 100,000 samples, the plate stage alone (no erosion, the
datum's sea) for 600 Myr on two planets, and every 50 Myr after the first 100 the force balance
solved again on the step's state with one term changed, c_b kept unless the row says so. One step's
balance only: the slabs, fronts and collisions a faster motion would make are not in it. The
balance's RMS as CHK-020 weighs it (visible samples, the plates' centroid speeds), then the plates
with a slab pull and the rest; means over the 11 looks. The plate stage alone gives the pool's level
(the motion's RMS 24.9 and 26.4 mm/yr), in about two and a half minutes of test time.

| One change | Supercontinent, seed 1: RMS / pulled / rest | Scattered continents, seed 2 |
|---|---|---|
| the defaults | 27.3 / 29.0 / 15.5 | 29.3 / 30.6 / 15.5 |
| the suction not saturating (PLT-540 off) | 29.4 / 30.5 / 22.4 | 31.2 / 32.3 / 20.0 |
| no suction | 31.5 / 32.9 / 22.0 | 34.2 / 35.4 / 21.1 |
| no edge energy (ridge push, belt weight) | 24.9 / 26.5 / 13.7 | 26.8 / 27.9 / 13.3 |
| no keels (keel drag 1) | 28.8 / 30.4 / 16.9 | 31.0 / 32.2 / 16.9 |
| no collision buoyancy | 28.7 / 30.4 / 16.6 | 30.8 / 32.1 / 17.7 |
| no boundary resistance | 45.8 / 46.2 / 44.5 | 48.7 / 49.0 / 53.1 |
| no slab resistance | 90.2 / 94.7 / 59.3 | 101.9 / 105.5 / 67.2 |
| boundary resistance 1, c_b by the definition | 25.6 / 26.7 / 18.4 | 27.3 / 28.2 / 19.4 |
| the speed scale ×1.5 (c_b / 1.5) | 40.8 / 43.2 / 23.3 | 43.7 / 45.6 / 23.1 |
| Forsyth & Uyeda's Table 6 ratios and terminal velocity | 51.3 / 43.8 / 80.8 | 60.0 / 46.8 / 151.7 |
| Forsyth & Uyeda's ratios, the speed scale kept | 35.8 / 30.5 / 57.0 | 41.8 / 32.5 / 105.9 |

The area with a slab pull is 0.851 and 0.901 of the planet. At 40,000 samples the pattern is the
same (the motion 27.7 and 32.7). `plates_quick_16k` is no guide to the pool's planet: its ridge and
floor (-1,000 and -10,000 m against `earthlike_quick_16k`'s -2,600 and -5,651) give a ridge push
several times Earth's (the motion 39.2 and 40.8 at 100,000 samples, 29.7 and 32.8 with no edge
energy).

What the rows say:

- **The contrast is Earth's, the level is not.** The plates with a slab move about twice as fast as
  the rest (29 to 31 against 15.5), as on Earth (Zahirovic et al. 2015's medians, any subducting
  7.05 cm/yr against 2.6 with no subduction; Forsyth & Uyeda's Table 1, the slab plates 6.1 to 8.6
  cm/yr against 0.7 to 4.2 for the others), but both are short: the pulled plates 0.42 of Zahirovic's
  70.5 mm/yr and the rest 0.60 of 26.
- **The two resistances of the open-loop calibration hold the planet.** At fixed c_b, no boundary
  resistance raises the RMS 1.66 to 1.68 times (the reference plate's 7/4) and no slab resistance
  3.3 to 3.5 times (more than the reference plate's 7/4, since it also takes the suction's link and
  the slabs' terminal velocity with it). The keels, the collision buoyancy and the edge energy each
  move the level by under 10 %.
- **PLT-540 acts mostly as a drag on the plates without a slab.** Not saturating, the suction gives
  the rest 22.4 and 20.0 mm/yr; saturating, 15.5 and 15.5; with no suction at all, 22.0 and 21.1. So
  the link that saturates the suction (to the mantle along the slab's direction, k = share × slab
  resistance × ℓ) slows the overriding plates by more than the whole suction speeds them. That is
  the pool's base4 to base5 step, where only `v_free` fell.
- **Forsyth & Uyeda's force strengths alone raise the level into the band but are not a working
  balance.** Their Table 6 (model X_s, read; PLT-575) put on FRC-016's reference plate gives slab
  resistance 8.94 times the oceanic drag (against the model's 3, with 3 more for the boundary), the
  continental drag 7.9 times the oceanic (the model's keel 3, inside Conrad & Lithgow-Bertelloni
  2006's 2 to 5), the suction 0.078 of the pull and independent of speed (the model's 0.3, saturating),
  a terminal velocity of 72 mm/yr (6.43 / 0.89 cm/yr) and the reference plate at 65 mm/yr against
  the model's 45. Their transform resistance (0.36 per km) and colliding resistance (0.16 per km)
  do not depend on speed and act on transforms and convergent boundaries only; they have no ridge
  resistance and no viscous resistance between plates, so that variant has no boundary resistance.
  It moves the planet at 51 to 60 mm/yr, but the plates without a slab outrun the pulled ones (81 and
  152 against 44 and 47), against Forsyth & Uyeda's own Table 1 and Zahirovic et al.: the
  speed-independent resistances at transforms and collisions, which the viscous all-round boundary
  resistance stood in for, are what holds those plates, and the variant leaves them out.

### What it is and what it is not

- Not a lattice or counting bug: the lattice moves the reference plate faster than its definition.
- Not one merge: the fall from 37 to 27 on the pool is spread over six steps at p < 0.05, five
  down and one up: the largest PLT-540 (-4.9 mm/yr, p < 0.001), then V1 gap 5 (-1.8), V1 gap 2
  (-1.6), PLT-553 (-1.3) and PLT-560 (-0.9), with PLT-561 giving 1.1 back. The fall from 41 on
  2026-10-05 to 37 is the crust budget's.
- The level is the open-loop calibration's, never redone: v_s 45 was not calibrated against CHK-020
  (PLT-181), and s and b were chosen against open-loop targets (CHK-021 to CHK-023, CHK-028) on a
  planet that had none of the fronts, collisions and suction laws since.
- Three places where the model differs from the sources it cites, each read in this lane:
  1. PLT-189 cites Forsyth & Uyeda's transform resistance, but theirs is speed-independent, acts on
     transforms only and has no ridge part; the coded term is viscous and acts on every boundary,
     ridges and trenches included.
  2. The resistance ratios are not Forsyth & Uyeda's (above, and PLT-575).
  3. PLT-540's premise (a slab cannot roll back faster than it sinks, so the mantle resists the
     trench's migration from rest) is `from-memory`, and the trench-migration studies read do not
     support it. In them trench motion follows the upper plate's (V_t about V_up): "spontaneous
     trench rollback related to slab pull is negligible with respect to upper plate motion", and
     92 % of trenches move within ±50 mm/yr (Heuret & Lallemand 2005). The anchor force "is
     inefficient to prevent trench migrations up to ±50 mm.y⁻¹" (Lallemand, Heuret & Boutelier 2005,
     after Heuret & Lallemand). V_t = 0.5 V_sub − 2.3 cm/yr in HS3 (Lallemand et al. 2008), and the
     mean retreat is 1.33 to 1.51 cm/yr (Schellart, Stegman & Freeman 2008). None states a limit of
     rollback by the slab's sinking speed, and the first two say the resistance to migration is
     weak below 50 mm/yr, where PLT-540's resistance has already halved the suction.

### Recommendation (the coordinator's and Ben's; nothing applied)

1. **Physics, recommended: take the force balance's resistances from one source.** As its own lane,
   take Forsyth & Uyeda's Table 6 as a whole: slab resistance 8.94, the continental drag 7.9 (or
   Conrad & Lithgow-Bertelloni's range), the suction speed-independent at 0.078 of the pull (PLT-540
   off). Replace the viscous all-round boundary resistance with their speed-independent transform
   and colliding resistances on those boundaries only, as friction links of the collision link's
   form (k = F / max(|Δv|, floor), `forces.rs`) on the boundary types V1 gap 2 classifies. The
   level then follows from their terminal velocity, and v_s stops being a free number (65 mm/yr on
   the reference plate). The diagnosis shows the first half alone lifts the level into the band and
   lets the free plates run, so the second half is what the lane would build and test: a unit test
   against Forsyth & Uyeda's Table 1 (slab plates 6 to 9 cm/yr, the others 0 to 4) on the lattice,
   then one cumulative arm.
2. **Calibration, Ben's decision: PLT-181's own calibration against CHK-020.** The level is linear in
   v_s: about 68 mm/yr for the band's floor and 80 for Earth's 47. That makes c_b 1.65 to 1.4 ×
   10¹⁴ Pa s/m, an asthenosphere of 1.4 to 1.7 × 10¹⁹ Pa s over 100 km, inside Lithgow-Bertelloni &
   Richards 1998's "of the order of 10^19 Pa s". It is a gain on the level, so under Ben's rule it is
   his to choose, not the lane's.
3. **PLT-540 either way.** Its premise is unsourced and reads against the trench-migration studies,
   it is the largest single fall, and it acts as a drag on the plates without a slab. Option 1 takes
   it off. Off on its own it gives about +2 mm/yr in one step's balance, and its row records the
   speed guard binding on 3 of 24 seeds of the 8k test planet with the fixed share.

## Plate speed, phase 2: Forsyth & Uyeda's balance (PLT-585, 2026-10-25)

The coordinator approved recommendation 1 above and held recommendation 2 (the speed scale's
calibration) for Ben, so `plate_speed_scale_mm_yr` is untouched. The brief: take the drives and
resistances of Forsyth & Uyeda 1975's Table 6 together; build the transform and colliding
resistances as speed-independent friction links of the collision link's form, on transforms and
colliding boundaries only; replace PLT-189's viscous boundary resistance on every boundary, since
its own citation does not support it; turn PLT-540 off in the same change; and let the level follow
from their terminal velocity with no gain or knob added. One switch, unit tests first, on by default
once correct.

The outcome: built, and correct against the paper in its unit tests, but **off by default**. On the
pool (arm `sp-fu`, below) the speed guard (PLT-022) binds in all 8 runs, against none of base12's
24, and CLAUDE.md puts physics on by default only with "no divergence or guard failure on the pool".

### What was built

One switch, `tectonics.forsyth_uyeda_balance` (PLT-585). With it, `forces::Coeffs::new` reads none
of the old law's sizes (`plate_speed_scale_mm_yr`, `slab_resistance`, `boundary_resistance`,
`keel_drag`, `slab_suction_share`, `suction_saturates`): every size is Table 6's (FRC-033) against
the model's reference pull F_sp (3 × 10¹³ N/m at Earth's gravity, FRC-002), which stands for their
6.43 (FRC-034).

| Term | Table 6 (per km; the drags per km² per cm/yr) | In the model |
|---|---|---|
| Slab pull F_SP | 6.43 | F_sp per metre of oceanic front, as before (PLT-182) |
| Slab resistance F_SR | 0.89 per cm/yr | F_sp / v_t per metre of trench, v_t = 6.43 / 0.89 cm/yr = 72.2 mm/yr; on the subducting plate only, against its speed towards the trench (p. 183) |
| Mantle drag F_DF | 0.82 × 10⁻⁵ per cm/yr | the slab resistance per km of trench times 0.82 × 10⁻⁵ / 0.89 per km²: on FRC-016's reference plate the slab resistance is 8.9 times the basal drag (the old law's 3) |
| Continental drag F_CD | 5.65 × 10⁻⁵ per cm/yr | beyond the mantle drag: the keel factor (0.82 + 5.65) / 0.82 = 7.89 (the old law's 3) |
| Suction F_SU | 0.50 | 0.50 / 6.43 = 0.078 of the pull, independent of speed (pp. 170-171, 195; the old law's 0.3; PLT-540 not read) |
| Transform resistance F_TF | 0.36, against the relative motion | a friction link of 0.36 / 6.43 F_sp per metre on transforms |
| Colliding resistance F_CR | 0.16, against the relative motion | a friction link of 0.16 / 6.43 F_sp per metre on colliding boundaries, trenches and collision fronts alike ("no distinction between the resisting force at trenches and the resisting force at mountain belts", p. 170) |
| Ridge push F_RP | 0.36 | not taken: the model's column energy stays (PLT-187), about 0.10 of the pull at the pool preset's ridge and old floor against Table 6's 0.056 |
| (viscous boundary resistance) | not in their model | none (PLT-189 is not read) |

The collision resistance (the belt's weight and the dragged-down crust's buoyancy, FRC-018, FRC-030)
and the edge energy are kept: they are the model's physics for what Table 6 lumps into F_CR and F_RP,
and the brief named only the transform and colliding resistances. The speed guard (PLT-022,
250 mm/yr) is unchanged.

**The friction links** (FRC-035). Forsyth & Uyeda's F_TF and F_CR are forces per unit length "opp.
rel. motion" whatever the speed, acting "with equal strength on both plates" (p. 169). Each is a
link between the two plates at the boundary sample along their relative velocity, of coefficient
k = F / max(|Δv|, floor) with FRC-011's floor of 1 mm/yr, `Balance::add_friction`, the collision
link's form: linearised about the step's motion, so the fixed point the motion relaxes to is the
plastic answer, and a drive smaller than the friction leaves the boundary slipping under the floor.

**Which boundary is which.** Forsyth & Uyeda's own rule, the one their strengths were fitted with
(p. 183): "We have arbitrarily chosen φ = 60° ... as the dividing point between classifying a
boundary segment as a convergent boundary, I_CR, or as a transform fault, I_TF", φ = tan⁻¹(V_PAR /
V_PER) (`forces::fu_transform`). So a segment within 30 degrees of strike-slip is a transform; one
whose plates converge across it otherwise is colliding; one where they part has no friction (they
have no ridge resistance). V1 gap 2's boundary types (Bird's 20 degrees, PLT-311) are not used here.

**Where it acts.** A trench's friction is counted once, at the front the overriding sample has, over
the front's metres (PLT-560's share of the straight front), as its pull and suction are, against the
plate going under. Every other stretch of boundary with a visible plate beyond it counts from both
sides, half of its share each (PLT-530), by its own normal (PLT-310); the side going under in an
active contact skips it, since its front counts it. The first build counted every boundary by the
visible-neighbour rule alone and missed most trenches: at a mature trench the subducting plate's
samples beside the trench have gone under and see no other plate, and the overriding plate's front
sample was skipped as a front, so a small plate over a trench had no friction at all. The force
record books the friction in its `boundary_resistance` column.

**PLT-540 and the pinned fixtures.** PLT-540 (`suction_saturates`) is not read under PLT-585
(`Coeffs::new`), so one switch chooses the whole law: Forsyth & Uyeda's suction is independent of
speed, and PLT-540's premise is `from-memory` and unsupported by the trench-migration studies read
(above). The brief turned PLT-540 off in the same change; with PLT-585 off by default, turning it
off by itself would change the default planet with nothing in its place, so it stays on with the
old law. The lattice reference tests (`the_reference_plate_on_the_lattice_*`, `old_law()`) test the
speed scale's own law, and the 8k test planet's collision, suture and recording fixtures pin the old
law (`forsyth_uyeda_balance` false, `suction_saturates` true) whatever the defaults are: under
PLT-585 the guard binds on that planet (below).

### Unit tests against the paper (`forces.rs`)

- `the_resistances_are_forsyth_and_uyedas_table_6`: the slab's terminal velocity 72.2 mm/yr against
  the paper's "7.0 cm/yr" (p. 189; their "(= 5.65/0.82)" is the ratio of their two drags, 6.43 /
  0.89 is 7.2), within 5 %; the continental drag 7.89 times the oceanic ("approximately eight
  times", p. 189); the suction and the two frictions their shares of the pull; the slab resistance
  8.9 times the drag on FRC-016's reference plate; none of the old law's settings read, PLT-540
  included.
- `the_reference_plate_moves_near_its_slabs_terminal_velocity`: FRC-016's reference plate (the
  starting plates' mean area, 3,000 km of trench of a full-length 80-Myr slab) moves at 66.3 mm/yr
  without friction (65.1 by v_t s / (1 + s), s = 9.15, on a flat plate) and at 56.1 mm/yr with
  friction all round its rim, after 13 solves; Forsyth & Uyeda's plates attached to a slab move at
  "6-9 cm/year" (p. 178), their terminal velocity "roughly coincident with the average velocity of
  oceanic plates attached to a slab" (p. 189). On the cap the 60 degree rule makes 84 of the rim's
  187 samples transforms and 103 colliding; the friction as fixed forces against the motion gives
  the same answer.
- `friction_resists_with_its_force_whatever_the_speed`: a drive larger than the friction moves the
  plate as a fixed force would; a smaller one leaves it under the floor speed.
- `friction_holds_back_a_plate_with_no_slab`: the runaway the diagnosis found. A plate of the
  reference plate's size with no slab, pushed by a ridge along the back 140 degrees of its rim at the
  pool preset's ridge and floor: the old law 19.2 mm/yr; Table 6 without friction 146.2 (the
  runaway); with friction 37.6; at Table 6's own ridge push (0.36 per km) 0.81, held at the floor
  ("continental drag is the primary regulator of the velocity of plates not attached to downgoing
  slabs", p. 190). Continental: the old law 12.5, with friction 4.8.

The terminal-velocity test builds its own trench on a cap and reads no lattice fronts, so it does
not depend on the front set (the front lane's current-geometry fronts change only the planet's).

### The planet (one step's balance, iterated to its fixed point)

`what_sets_the_planets_plate_speed` again (build/speed/diag7.sh), the same two planets and looks as
above (`earthlike_quick_16k`, 100,000 samples, the plate stage alone, 11 looks from 100 to 600 Myr),
the friction iterated to its fixed point at each look:

| Variant | Supercontinent, seed 1: RMS / pulled / rest | Scattered continents, seed 2 |
|---|---|---|
| the old law (PLT-585 off, PLT-540 on) | 26.7 / 27.7 / 19.5 | 28.5 / 29.6 / 15.5 |
| Table 6 without the friction (phase 1's row) | 62.7 / 42.3 / 134.6 | 54.1 / 45.6 / 94.7 |
| **Forsyth & Uyeda's balance (PLT-585)** | **50.2 / 36.4 / 97.0** | **44.2 / 38.3 / 68.7** |
| PLT-585 at Table 6's ridge push (the edge energy scaled to 0.056 of the pull) | 41.2 / 32.4 / 69.6 | 33.1 / 34.2 / 21.9 |

The level is in CHK-020's band with no knob, and the pulled plates move at 36 to 38 mm/yr (the old
law's 28 to 30). The friction takes the plates without a slab from 135 and 95 to 97 and 69 mm/yr,
but not down to the pulled plates' speed: the contrast Forsyth & Uyeda's Table 1 and Zahirovic et al.
2015 show (slab plates two to three times the others) is reversed on the planet.

**Why: the overriding oceanic plates.** The rest's RMS² is mostly oceanic plates of 0.1 to 0.5
starting plates (0.65 and 0.51 of it). They lie over trenches. Table 6's suction draws an overriding
plate with 0.50 per km of trench, its colliding resistance at the same trench holds it with 0.16, the
slab resistance acts only on the plate going under ("the component of motion of the plate relative
to the mantle which is directed perpendicular to the strike of the trench", p. 183, for the
subducting plate), and nothing else in their model resists the overriding plate's advance but the
friction on its other boundaries. So 0.34 per km of trench, plus its ridge push, less that friction,
against the basal drag alone, 0.82 × 10⁻⁵ per km² per cm/yr: its speed goes as its trench length
over its area, as 1 / R. Forsyth & Uyeda's Earth had one such plate, the Caribbean (Table 1: 3.8 ×
10⁶ km², no continent, 2.4 cm/yr, 3,700 km of trench it overrides, 4,400 km of transform and 4,400 km
of convergent boundary), and on Table 6 its friction holds it: 0.36 × 4,400 + 0.16 × 4,400 = 2,288
against a suction of 0.50 × 3,700 = 1,850, 1.24 times (computed). Their other plates without a slab
are continental (Eurasia, North America, Africa, Antarctica), held by the continental drag. Their
own words on the term: "In view of the uncertainty of the nature of F_SU, in the present paper we
modelled it to be velocity independent and perpendicular to the trench, although F_SU caused by the
secondary flow may be proportional to the descent rate of the slab" (pp. 170-171); models with the
suction proportional to the slab's descent or to the convergence gave "similar" results (p. 195);
"The least understood part of our model is the nature of this suction force" (p. 193). Nothing here
changes it: the brief was Table 6 as a whole.

### The speed guard

On the 8k test planet (`plates_quick_16k`, whose ridge push is several times Earth's;
`weld_seed_sweep`, 300 Myr, seeds 1 to 24) the guard binds on all 24 scattered seeds and 22 of 24
supercontinent seeds, 3 to 146 plate-steps a run (under the old law: 0 of the 24 scattered seeds at
PLT-540's arm). On `earthlike_quick_16k` at 8,000 samples (`where_the_speed_guard_binds`, 300 Myr)
it binds 0, 21 and 5 plate-steps on scattered 2, supercontinent 4 and scattered 1. The plates it
holds are the overriding oceanic plates above: no slab pull, the suction about equal to the basal
drag, and on the smallest (a few samples) a balance answer above 1,000 mm/yr. The collision, suture
and recording tests on the 8k test planet keep the old law: only supercontinent 11 of the 48 runs
both welds and stays clear of the guard, and the recording test asserts the guard never binds.

### The arm (`sp-fu` against base12)

One 8-run arm with the switch on (the default at the arm's commit, pool key b301f884308e, master
05a2850a merged): `earthlike_quick_16k`, both starts, seeds 1 to 4, 1,000 Myr, 2 threads,
`+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`; read against base12 (24 runs) with
`pool.sh table --base base12 --arm sp-fu` (v1census version 2) and build/speed/speedlevel.py (the
plate check's speed measures, medians, Mann-Whitney p):

| Measure | base12 (24) | sp-fu (8) | p |
|---|---|---|---|
| RMS plate speed, mm/yr | 27.2 | 31.5 | 0.001 |
| plates with a slab, mm/yr | 49.0 | 50.5 | |
| plates without one, mm/yr | 21.2 | 24.4 | |
| collisions per 100 Myr | 1.60 | 1.55 | 0.71 |
| collision duration p90, Myr | 177.6 | 204 | |
| land above 2 km (census) | 0.128 | 0.126 | 0.98 |
| speed guard, steps that bind | 0 in all 24 | 3 to 15 in all 8 (5, 15, 6, 5; 8, 3, 9, 9) | |

The census moves at no measure with p < 0.05: land 0.312 to 0.329 (p 0.11), the largest landmass
0.189 to 0.245 (p 0.22), continental crust under the sea 0.190 to 0.175 (p 0.24), coast with high
ground 0.229 to 0.236 (p 0.98), land under ice 0.107 to 0.085 (p 0.25), the continental crust's net
change −0.069 to −0.059 (p 0.19), criteria met 5 of 7 in both (means 5.04 and 5.12).

Over a whole run the plates move at 31.5 mm/yr, 0.67 of Earth's 47 (PLT-181), not the one-step
balance's 44 to 50, and the plates with a slab move at twice the speed of the others (50.5 against
24.4), so the one-step balance's reversed contrast does not last a run. The one-step looks put the
new law on planets grown under the old one; why a planet grown under the new law is slower was not
measured. The guard binds too seldom to explain it (3 to 15 of a run's 500 steps), but on a few
small plates in every run, which is enough to keep the switch off by CLAUDE.md's rule.

**Again on the front on the current geometry** (`sp-fu2`, the same arm with `--set
tectonics.forsyth_uyeda_balance=true` on the merge of master 05a60997, whose PLT-582 judges fronts on
the current geometry and drops the stale fronts read through hidden neighbours; key faf1a3, argkey
f7a070a5). Its control is the front lane's arm `fl-n2th` (PLT-581 and PLT-582 on, the switch off),
since base13 does not exist yet:

| Measure | base12 (24) | fl-n2th (8) | sp-fu2 (8) | p, sp-fu2 against fl-n2th |
|---|---|---|---|---|
| RMS plate speed, mm/yr | 27.2 | 27.5 | 31.9 | 0.001 |
| plates with a slab, mm/yr | 49.0 | 49.7 | 50.8 | 0.10 |
| plates without one, mm/yr | 21.2 | 22.9 | 27.9 | 0.002 |
| collisions per 100 Myr | 1.60 | 1.60 | 1.30 | (0.042 against base12) |
| collision duration p90, Myr | 177.6 | 196.2 | 158.9 | |
| land above 2 km (census) | 0.128 | 0.114 | 0.120 | |
| speed guard, steps that bind | 0 in all 24 | 0 in all 8 | 0 to 16, in 7 of 8 (9, 13, 16, 5; 10, 0, 4, 6) | |

The census against base12: no measure at p < 0.05 but the collisions per 100 Myr, 1.60 to 1.30 (p
0.042); coast with high ground 0.229 to 0.195 (p 0.24), continental crust under the sea 0.190 to
0.160 (p 0.31). The current-geometry fronts do not clear the guard: it binds in 7 of the 8 runs, so
the switch stays off on the merged head too.

**Against base13** (`sp-fu3`, the same arm on the merge of master c47f3056 and 683166ea: the
critical-taper belt, PLT-600 and PLT-601, with PLT-581 and PLT-582; key 26ea8ec0342f, argkey
f7a070a5), read against base13 (master c47f3056, 24 runs) with `pool.py table --base base13`
(v1census version 2; `--measure guard`, the relief guard) and build/speed/speedlevel.py. The plate
check's `speed_rms_mm_yr` of every arm against base13:

| Arm (8 runs each) | RMS plate speed, mm/yr | p against base13's 24 (26.9) | p against its matched 8 (27.3) |
|---|---|---|---|
| sp-fu (the branch before the merges) | 31.5 | < 0.001 | 0.014 |
| sp-fu2 (master 05a60997 merged) | 31.9 | < 0.001 | 0.001 |
| sp-fu3 (master c47f3056 merged) | 32.8 | < 0.001 | 0.001 |

| Measure | base13 (24) | sp-fu3 (8) | p |
|---|---|---|---|
| plates with a slab, mm/yr | 50.5 | 51.3 | 0.23 |
| plates without one, mm/yr | 22.6 | 27.3 | < 0.001 |
| collisions per 100 Myr | 1.70 | 1.20 | 0.097 |
| collision duration p90, Myr | 191.8 | 163.2 | |
| land above 2 km (census) | 0.097 | 0.105 | 0.51 |
| relief guard's worst, m from the sea | 10,047 | 10,188 | 0.57 |
| speed guard, steps that bind | 0 in all 24 | 0 to 12, in 7 of 8 (4, 5, 12, 2; 0, 6, 2, 8) | |

The census against base13: continental crust under the sea 0.214 to 0.160 (p 0.041), land 0.313 to
0.331 (p 0.086), the largest landmass 0.166 to 0.221 (p 0.24), coast with high ground 0.196 to 0.218
(p 0.45), land under ice 0.083 to 0.128 (p 0.24), the crust's net change −0.042 to −0.049 (p 0.43),
criteria met 5 of 7 in both (means 5.38 and 5.12); the sea over the datum after 100 Myr and at the
end 284 / 467 m to 189 / 206 m (not explained). No step passes 2 Earth reliefs in either. The
collisions per 100 Myr fall in all three arms (1.60 to 1.55 and 1.30 against base12, 1.70 to 1.20
against base13), none at p < 0.05 against the base it was read on but sp-fu2's 0.042. The speed
guard binds in 7 of 8 runs again, so the switch stays off on this head. With it off, the head is
master's planet bit for bit: one run (`sp-off3`, supercontinent 1, base13's arguments) has the
state_sha256 and rec_sha256 of base13's.

**The boundaries by the 60 degree rule** (a VM-only probe at PLT-585's fixed point on the same two
planets as above, not committed). Of the planet's boundary, 0.28 to 0.29 is transform, 0.56 to 0.58
converging and 0.13 to 0.15 diverging; Earth's by Table 1 is 30 % transform, 34 % convergent and 35 %
ridge (ridge 1,183, transform 1,016, convergent 1,136, × 100 km; computed). So the planet has
Earth's share of transforms; it has less ridge and more converging boundary. The free oceanic plates
(0.1 to 0.5 starting plates) hold, all together, 1.11 to 1.23 times their suction in friction, near
the Caribbean's 1.24; the fast ones hold 0.43 to 0.67 times: long trench and few transforms. Their
speed is about (0.50 − 0.16) × trench over the basal drag times their area. The probe may count too
much trench where the overriding plate meets crust it cannot see (a hidden neighbour), so the fast
plates' ratio is a lower bound.

**What would hold them, with no knob.** On Earth an overriding plate whose trench retreats opens a
marginal sea behind its arc, and then "the oceanward migration of trench-arc system becomes an
inefficient means of generating suction on the continental plate behind the marginal sea" (p. 170):
the arc's sliver moves with the trench and the plate behind it does not. The model's plates are
rigid and do not split along an arc, so the whole small plate follows the suction. Two physical
routes, neither built: back-arc opening (an overriding plate splits at its arc when its suction
outruns its own drag, so the sliver that moves is small and bounded by the new ridge's spreading),
or a suction law with a source (pp. 170-171: "may be proportional to the descent rate of the slab";
PLT-540's saturating law is a version of this, but its premise is `from-memory`). Until one of them
is built, PLT-585 stays a switch that is off.

**With incipient subduction (the front lane's switch).** The front lane is building incipient
subduction (Gurnis et al. 2004; decisions log 2026-10-25) behind its own switch: a contact stays
incipient until its slab reaches 100 km, and until then its front resists with a plastic 2.5 × 10¹²
N/m (5 × 10¹¹ at a ridge), linearised as the collision term is, and builds no arc. That is a
friction-like resistance on a converging front, and PLT-585's colliding resistance F_CR acts on the
same front (a trench's friction is counted at the overriding sample's front, above). With both
switches on, the rule is: **an incipient contact's front takes the incipient resistance alone, and
PLT-585's friction (F_CR, or F_TF where the 60 degree rule calls the front a transform) only once the
contact is mature.** Forsyth & Uyeda fitted F_CR on Earth's mature trenches and mountain belts
("no distinction between the resisting force at trenches and the resisting force at mountain belts",
p. 170), and the incipient resistance is the whole resistance of a front whose slab has not yet
formed, so counting both on one front counts that front's resistance twice. In size, F_CR is 0.16 /
6.43 × 3 × 10¹³ = 7.5 × 10¹¹ N/m (computed), so the incipient front resists 3.4 times as hard as a
mature one, and 0.67 times at a ridge. The code is two conditions in the friction pass of
`forces.rs` (`if friction`): the front's link is skipped while its contact is incipient, and the
loop over the other boundaries skips a plate that holds an incipient contact with the sample's plate
from either side, so neither side adds a half share in its place. The front lane landed second and
added them (2026-10-25), with the unit test `an_incipient_front_takes_no_forsyth_uyeda_friction`
(both switches on: an incipient front carries the incipient resistance alone, the same as with
PLT-585 off, and no friction between its two plates; a mature one F_CR alone, from the overriding
side, and no incipient resistance; the other boundaries' friction the same in both). The incipient
contacts are the ones the arcs and the boundary mix read, by the slab before the step's growth. A
continental sample at an incipient contact's front takes no incipient resistance (it is a
collision's, PLT-610), so it keeps the front's friction there as at a mature contact.

## The overriding plate's share (PLT-652, 2026-10-25)

The collision-front diagnosis (build log 2026-10-25, "Front lane: what sets the crust added per km of
collision front") found base14's fronts carrying Earth's crust per km (1.16 times Bird's CCB over the
run) but keeping 0.89 of it within 400 km of the front, where the planet stacks 1.81 times Earth's
excess crust per km and 2.8 times its crust over 55 km. Earth's overriding plates take much of a
collision's convergence away from the front: of Arabia's 440 km since 22 Ma, 135 km went into the
Zagros and 230 km into Central Iran and the Alborz behind it (52 %, Mouthereau 2011), and Asia took
600 to 1,050 km of India's convergence beside the Himalaya's 450 to 900 (an Asia share of 0.40 to 0.70,
van Hinsbergen et al. 2011, Tectonics). The model's overriding plate had no way to do that.

### The law (`tectonics.collision_upper_plate_shortening`)

The continental lithosphere deforms as a thin viscous sheet (England & McKenzie 1982; Houseman &
England 1986), so where it shortens is set by its strength, and the model already has that strength:
PLT-414's sheet, ε̇ = ΔU / (4 η L), with PLT-415's η = η_0 e^(t_eq / τ_s) at the lithosphere's thermal
age (ISO-112). PLT-652 puts the belt's push where ΔU stands:

- **The push.** A mountain range pushes on the lowland beside it with its gravitational potential
  energy over the lowland's (Molnar & Lyon-Caen 1988: Tibet's 6 to 7 × 10¹² N/m). That is the
  balance's edge term (FRC-018) read across the belt (`forces::belt_push_n_m`): the mean column
  energy of the belt's continental samples (within the belt's 400 km of the front) over the mean of
  the interior's strip (below), the lowland it pushes on. A 5 km belt pushes with 6.7 × 10¹² N/m, a belt at the 40 MPa strength height (1.46 km)
  with about 1.5 × 10¹².
- **The interior** behind the belt carries that push as a thin sheet. Along the strip behind the
  front (the overriding plate's visible continental samples beyond the belt's 400 km, between the
  front's ends along it and not ahead of it), each column shortens at ε̇ = F / (4 η L), so the
  interior shortens at v = F / (4 η_0 L) ∫ w dx, with w = η_0 / η = e^(−t_eq / τ_s) (PLT-470's
  form). At w = 1 that is 158 mm/yr per 1,000 km of strip for 2 × 10¹³ N/m, 53 mm/yr for a 5 km
  belt's push.
- **The share.** The interior takes v / v_n of the convergence, at most all of it. That share of the
  crust the step keeps goes to the strip's columns in proportion to T w (each column thickens by its
  own strain), the rest to the wedge and the belt by their own rules (PLT-242's share, PLT-600 and
  PLT-601's taper, PLT-253's strength height unchanged).

There is no share constant and no new value: η_0 (10²² Pa s), τ_s (62.8 Myr) and L (100 km) are
PLT-414's and PLT-415's, the column energy and the edge scale the balance's. The belt sets its own
height. A fresh front has no belt and no push, so the front takes the first step; the belt then
rises until its push drives the interior at the rest of the convergence, which is how a thin sheet
grows a plateau in front of an indenter (England & Houseman 1986). The weakness sets where that
balance falls. A start continent's keel (200 km, `thermal_keel_km`) has t_eq 384 Myr and w 0.0022;
a lithosphere of 150 km has w 0.078, of 120 km 0.23, and the hot limit (60 km) 1. So a keeled
craton behind a front takes almost nothing even under a belt at its strength height, and a hot,
thinned plate takes most or all under a low belt. Crust is conserved: the interior's crust is the
crossing's mix, counted in `kept` and in its own column of the crust budget (`upper`).

In the unit fixture (a head-on collision at 30 mm/yr, the overriding continent's strip about 2,050 km
deep behind the belt), the twelfth step's share is 0.008 at 200 km of lithosphere under a push of
6.6 × 10¹² N/m, 0.055 at 180, 0.14 at 165, 0.25 at 150, 0.55 at 120 and 0.82 at 60 under 1.4 × 10¹²
N/m: behind the hot plate the belt stays low. Over the twelve steps the 60 km interior took 0.67 of the
crust kept, the 200 km one 0.006.

The push is not the collision's resistance in the balance. The balance resists a collision with the
edge term and the buoyancy of the continental lithosphere dragged down (FRC-030), a plastic
resistance whose capacity grows with the shortening to about 9 × 10¹³ N/m at the slab's full depth.
That buoyancy is the mantle's hold on the slab, not a horizontal force carried by the overriding
plate, and it is an order of magnitude over what a range pushes with. The first build read it as
the force on the interior: in the unit fixture the force grew by 6.5 × 10¹² N/m a step with the
shortening, and a plate of 150 km took all of the convergence from the fourth step. That reading
was dropped before any arm ran.

Two choices are this lane's, neither a fitted value. The strip is one-dimensional: it carries the
front's force undiminished to the plate's far side, where a two-dimensional sheet spreads a
boundary's stress over a length scale of the order of the boundary's width (England, Houseman &
Sonder 1985, from memory), so the share is the interior's largest. And the law is kinematic: the
balance's resistance and the plates' motion are as before; only where the crust goes changes. The interior's columns
are stamped as a collision orogeny (fold along the front), so the collapse, the root removal and
the erosion treat them as such, but they stay out of the belt's record (its height, width and
footprint are the front's).

### On the pool (arm fl-upper-unfreeze, 2026-10-25): on by default

The arm ran on the M4 lane's motion fix and chemical denudation (PLT-191, ERO-205 to ERO-207; merged
as master 16db742d): `pool.sh combine` of this branch and 4d5cf4dd, key 4040677e7800, 8 runs
(supercontinent and scattered_continents, seeds 1 to 4, earthlike_quick_16k, 1,000 Myr, 2 threads),
`--set tectonics.collision_upper_plate_shortening=true` and the crust budget's two envs. The base for
the census and the guard is m5-unfreeze (67edbe8a8914, the fix alone, no envs). The base for the
front's budget and maps is its twin fl-off-env8: the same combination with the switch off and the
envs on. All 8 of its runs match m5-unfreeze's final world state and dynamics record bit for bit
(state_sha256 and rec_sha256, 109 state files). So the switch off changes nothing, and the envs only
measure.

| v1census, medians of 8 (Mann-Whitney p) | target | m5-unfreeze | fl-upper-unfreeze |
|---|---|---|---|
| land | 0.13 to 0.36 | 0.193 | 0.191 (0.96) |
| largest landmass | ≥ 0.06 | 0.0549 | 0.0597 (0.96) |
| continental crust under the sea | ≤ 0.56 | 0.566 | 0.552 (0.65) |
| land above 2 km (M4; Earth 0.047) | ≤ 0.07 | 0.016 | 0.026 (0.050) |
| coast with high ground | ≤ 0.11 | 0.091 | 0.129 (0.44) |
| land under ice | ≤ 0.32 | 0.059 | 0.073 (0.44) |
| collisions per 100 Myr | ≤ 6 | 5.6 | 5.0 (0.49) |
| continental share of the planet | | 0.402 | 0.384 (0.010) |
| continental crust's net change (area) | | −0.021 | −0.039 (0.015) |
| sea over the datum, after 100 Myr / end, m | | 469 / 687 | 486 / 710 (0.57 / 0.28) |
| criteria met per run, median and mean | of 7 | 6, 5.38 | 5.5, 5.38 (0.78) |

The guard stays clean. Its worst is 10,103 m against 10,251 (p 0.65). No step goes beyond two Earth
reliefs, and no continental plate cell stands 5 km over its Airy height. The largest excess over Airy
is 2,322 m against 2,403, the highest plate elevation 5,097 m against 5,321 (p 0.23).

The interior shortens, so the continents lose area to thickness. In the final fields the median
continental area falls 4.5 % (0.402 to 0.384 of the planet), the mean thickness rises from 35.7 to
36.1 km, and the volume, as km of crust spread over the planet, falls 2.2 % (14.21 to 13.90). The
switch's own step creates and removes nothing: `kept` includes the interior's `upper`, and no-room
losses fall. The volume goes afterwards. More high ground erodes, and more of that sediment goes
down the trenches: sediment subducted rises 8.0 × 10⁷ km³ over the run (median, 9.70 × 10⁸ to
1.05 × 10⁹), the slab's crust 6.5 × 10⁷ (its sediment counted in both), and the mafic pools dropped
at accretion 2.7 × 10⁷.

At the front (collfront, twin against arm; medians of runs over the whole run, per km of front over
the last 50 Myr):

| | twin (switch off) | arm | Earth | base14 |
|---|---|---|---|---|
| kept / crossed | 0.757 | 0.792 | | 0.89 |
| kept at the front | 0.757 | 0.664 | | |
| the interiors (`upper` / crossed; by crossed area) | 0 | 0.131; 0.126 | | |
| crossed area at fronts whose interior took half or more | 0 | 0.076 | | |
| lost to no room at a capped front / crossed | 0.136 | 0.111 | | |
| excess over the planet's continental median within 800 km, km³/km | 7,497 | 9,707 | 7,970 | 14,400 |
| crust 45 km or thicker within 800 km, km²/km | 241 | 405 | 289 | 599 |
| crust 55 km or thicker within 800 km, km²/km | 62 | 115 | 123 | 344 |
| land above 2 km within 800 km, km²/km | 98 | 193 | 185 | 512 |
| collision front, last 50 Myr, km | 7,281 | 9,540 | | |

Behind the belt, the overriding side's mean thickness rises from 41.1 to 43.1 km at 200 to 400 km,
from 36.8 to 39.1 at 400 to 800 km, and from 35.9 to 36.5 at 800 to 1,500 km. The belt also gives
less crust back to its lowland: over the run the collapse moved 1.7 × 10⁸ km³ less and the channel
flow 2.5 × 10⁸ less, and the collapse held 5.4 × 10⁸ more. The thickened interior lowers the
contrast those two laws run on (the budget's reading, not traced column by column). The front's
55 km tail doubles to Earth's, and so does its high ground. The collisions' timing barely moves
(m4decay section A, m5-unfreeze against arm):

- trains: 2,945 against 2,985;
- duration p50 10 and 10 Myr, p90 50 and 52, max 144 and 146;
- by the longest front p50 32 and 32 Myr, p90 68 and 72;
- front-time in trains over 110 Myr 0.02 and 0.04, over 200 Myr 0 and 0.

| land in each crust-thickness band (share of the land; m4split, mean of 8) | Earth (HYP-040) | m5-unfreeze | fl-upper-unfreeze |
|---|---|---|---|
| under 30 km | | 0.046 | 0.055 |
| 30 to 35 km | 0.197 | 0.044 | 0.056 |
| 35 to 40 km | 0.367 | 0.686 | 0.671 |
| 40 to 45 km | 0.274 | 0.171 | 0.154 |
| 45 to 55 km | 0.097 | 0.041 | 0.047 |
| 55 to 70 km | 0.019 | 0.0063 | 0.0150 |
| over 70 km | 0.006 | 0.0000 | 0.0002 |

M4 rises from 0.016 to 0.026 (p50; Earth 0.047). Most of the gain is young collision belts (0.0053 to
0.0131 of the land, mean) and belts 10 to 50 Myr old (0.0049 to 0.0083). The share of the high
ground under ice rises from 0.20 to 0.47. The interior's stamped columns enter the collision class:
its land share rises from 0.011 to 0.074 for the young belts and from 0.21 to 0.35 for the older
ones. The pooled collision-belt samples triple (194,000 to 583,000), at a median T of 39 km at the
stamp against 57.

Validation readouts, not fitted (no value was chosen to reach them): the interiors take 0.13 of the
crust crossed (0.126 of the convergence by crossed area), and fronts whose interior took half or more
carry 0.076 of the crossed area. Earth's Arabia put 52 % behind the Zagros (Mouthereau 2011), and
Asia took 0.40 to 0.70 of India's shortening (van Hinsbergen et al. 2011, Tectonics). So the model's
interiors take a quarter to a third of Earth's two cases, and the 1-D strip already gives the
interior's largest share. The law has no free value to close that gap, and none was looked for. On
by default since this arm: the law is built and tested against its sources, it conserves crust, and
the guard is clean (the rule of 2026-10-23: the V1 numbers are realism checks, not a gate).

## Sediment by margin type and the margin's sediment (PLT-660 to PLT-662, 2026-10-25)

The drowning diagnosis (build log 2026-10-25, "M5 lane: why the land drowns on m5-unfreeze") found the
sea standing about 700 m over the datum because oceanic-labelled crust of 15 km and more takes the
basins' volume. That crust is built mostly from the continents' sediment, on ocean floor next to the
continents that never recycles. On base15's seeds 1 to 4 it is 0.269 of the oceanic samples at the
end (m5thickoc), and refilling the oceanic crust of 10 km and more as a normal ocean column lowers the
counterfactual sea by about 1,000 m (m5sea). Earth keeps that rock out of the ocean basins two ways:
the margin's own sediment lies on thinned continental crust, and a trench sends most of what reaches it
down. Two stages were asked for, each behind its own switch, with tests written first:

- (a) margin material within Brune et al. 2014's 70 to 200 km stays continental, the width taken from
  the source, and the spread path labelled by its rock;
- (b) PLT-440's flat 0.12 replaced by Clift & Vannucchi 2004's share by the margin's type.

### The rules

- **(b) Sediment subduction by the margin's type** (PLT-660, `tectonics.sediment_subduction_sourced`).
  At a resampling the slab takes 0.83 of the sediment and sedimentary pools of the floor that went
  under where the pile is thicker than 1 km and the convergence is slower than 76 mm/yr, and all of
  it otherwise. Clift & Vannucchi 2004: accretion works "when convergent rates are <76 mm yr-1 and
  where the sedimentary cover is >1 km thick", and "Even at the accretionary margins, typically 83% of
  the incoming sedimentary pile is subducted". von Huene & Scholl 1991: at erosive margins
  "effectively all incoming sediment is subducted". The rule holds whatever the top's crust, so an
  intra-oceanic margin's thin pile (the Mariana, Tonga) goes down whole, where PLT-440 accreted 88 %
  of it to the top.
- **(a) The margin's sediment is continental** (PLT-661, `tectonics.margin_sediment_continental`).
  The rifted-margin taper's ocean side (PLT-261, −W/2 < d < 0, the crust thinning from 20 to 7 km) is
  where Earth's margins have their necking and hyper-extended domains: continental crust thinned to
  under 10 km, "slivers that vary in widths between 70 km offshore Iberia and 200 km off Angola"
  (Brune et al. 2014). A member there that is ocean floor and holds rock from the continents
  (sediment and sedimentary rock beyond the pelagic rain for its age, PLT-103) turns continental at the
  end of the step, over its own ocean crust as its basement (PLT-331). Members with the rain alone,
  at a trench, in a subduction zone or on an arc stay oceanic.
- **The side rule.** So that a converted member is never the next member's margin, the taper takes a
  continental column with under 10 km of its own crystalline crust (`hyper_extended`) as its ocean
  side, not its continental side, and ocean crust the taper adds to it joins its basement. A matured
  island arc (8 km of its own over its basement) falls under the same test, so with the switch on it
  is the ocean side too. The zone is found as the step begins, so a step's conversions do not move it
  within the step.
- **The spread's rock** (PLT-662, the same switch). The collapse, the founder and a rate release fill
  ocean floor up to the normal column before it turns continental (PLT-277), so up to 32 km of
  continental rock stood under the oceanic label. With the switch on the receiver turns continental at
  once when the rock is continental (always for the collapse and the founder, and for a rate release
  when continental columns gave more than half of it). This is a bookkeeping rule (`game-choice`).

No value was chosen: 1 km, 76 mm/yr, 0.83 and 10 km are read in the sources, and W is PLT-262's 200
km. On the quick planet's 68 to 71 km spacing, W/2 reaches ring 1 only. Unit tests (written first):
`sediment_goes_down_by_the_margins_type` and `a_resampling_subducts_sediment_by_the_margins_type` for
(b); `the_margins_sediment_turns_its_distal_margin_continental` (every member with 2,500 m of pile in
the zone turns, none with the rain alone, none on the subducting plate, and over 30 steps the crust
and heights equal the switch-off run's) and `spread_continental_rock_is_continental_at_once_with_the_margin_switch`
for (a). deeptime-core's tests pass (1,079).

One deviation from the source, on the first arm: PLT-660 read the top sample's `convergence_mm_yr`,
which subduction.rs stamps as the plates' whole relative speed at the front (the normal component
only with `DEEPTIME_ARC_NORMAL=1`). Clift & Vannucchi's 76 mm/yr is an orthogonal rate, so an oblique
margin read faster and its pile went down whole where the normal component alone would accrete 17 %
of it. Fixed for the second arm ("The blend closed (PLT-663)" below): the front stamps the normal
component as its own field and PLT-660 reads it.

### Where the stock lies (the ring split, before the arm)

The diagnostic run (base15's supercontinent-1 with the pools maps, key fb136aadfe68, argkey cd665cef;
its world matches base15's run) split the oceanic-labelled stock beyond the 7 km ocean column with
m5pools.py. The stock is 3,216 m planet-mean, of it 1,494 m of the continents' sediment. By
distance to the plate's own continent, in sample spacings, it lies:

| ring 1 | ring 2 | ring 3 | ring 4 and beyond | on plates with no continent |
|---|---|---|---|---|
| 0.46 | 0.19 | 0.11 | 0.18 | 0.06 |

So (a) at Brune's taper can reach under half of it. The rest is sediment on ocean floor farther out,
Earth's continental rise and abyssal fans, which lie on oceanic crust on Earth too and stay oceanic.
On that run, m5margins.py's converted-margin reading (continental crust over a basement with under
10 km of its own, no arc magma) finds 0.0001 of the planet with no relabel: that is the reading's
floor, and the share with and without the converted margins is the same there (0.401, 0.401).

### The arm (m5-margins, 2026-10-25)

Key cba2ba6afb66, argkey 5818ecce: 8 runs (supercontinent and scattered_continents, seeds 1 to 4,
earthlike_quick_16k, 1,000 Myr, 2 threads), `--set tectonics.sediment_subduction_sourced=true
tectonics.margin_sediment_continental=true` and the crust budget's two envs. The base is base15's seeds
1 to 4 (e5e78994377c, cd665cef, `@4`, the same envs). Both stages run together, as one cumulative arm.

| v1census, medians of 8 (Mann-Whitney p) | target | base15 seeds 1 to 4 | m5-margins |
|---|---|---|---|
| land | 0.13 to 0.36 | 0.191 | 0.114 (<0.001), out |
| largest landmass | ≥ 0.06 | 0.060 | 0.027 (0.065), out |
| continental crust under the sea (M3) | ≤ 0.56 | 0.552 | 0.816 (<0.001), out |
| land above 2 km (M4; Earth 0.047) | ≤ 0.07 | 0.026 | 0.003 (<0.001) |
| coast with high ground | ≤ 0.11 | 0.129 | 0.040 (0.003), now in |
| land under ice | ≤ 0.32 | 0.073 | 0.008 (<0.001) |
| collisions per 100 Myr | ≤ 6 | 5.00 | 5.35 (0.92) |
| continental crust's net change (area) | | −0.039 | +0.211 (<0.001) |
| sea over the datum, after 100 Myr / end, m | | 486 / 710 | 174 / 244 |
| criteria met per run, median and mean | of 7 | 5.5, 5.38 | 4, 4.12 (0.012) |

The guard stays clean. Its worst is 9,831 m against 10,251 (p 0.10). No step goes beyond two Earth
reliefs, and no continental plate cell stands 5 km over its Airy height. The largest excess over Airy
is 1,938 m against 2,403 (p <0.001), the highest plate elevation 3,340 m against 5,321 (p <0.001).
M4 (m4table, p50) falls from 0.0261 to 0.0026: young belts 0.0131 to 0.0015 of the land, arcs and
Andean belts 0.0058 to 0.0010.

What the stages were meant to fix moved as intended. The thick oceanic-labelled crust (m5thickoc,
share of the oceanic samples) falls from 0.175 to 0.047 at 15 to 25 km and from 0.094 to 0.001 at 25
km and more, so the crust of 15 km and more falls from about 0.166 of the planet to 0.018. Its hold on
the sea (m5sea, the counterfactual sea with the oceanic crust of 10 km and more refilled as a normal
column, against the sea refilled on the run's own cells) falls from about 1,000 m to 117 m. The sea
ends 244 m over the datum against 710.

### The continental share with and without the converted margins (t0000, plate samples)

| | Earth (Cogley 1984) | switch-off run | m5-margins with | m5-margins without |
|---|---|---|---|---|
| continental share | 0.41 | 0.401 | 0.634 | 0.567 |
| under the sea | 0.306 | 0.462 | 0.808 | 0.785 |

The converted margins are 0.0656 of the planet, all under the sea, at a median depth of 3,405 m, crust
16.5 km and sediment 4.1 km: Earth's distal margins, and the right label for them. Columns with under 10
km of their own and 1 km or more of island-arc magma (matured arcs, or converted margins that took arc
magma; the maps do not tell them apart) are 0.0475 against the switch-off run's 0.0037, and the
"without" reading keeps them. The census's M3 (0.816) is on its grid, these on the plate samples.

The converted margins explain little of the rise. The "without" share is still 0.567 against 0.401, and
0.785 of it is under the sea against 0.462. The rest of the new continental crust is the next section's.

### The ratchet: the continents spread, they do not grow

m5cont.py splits the continental plate samples by their own crystalline crust (metamorphic, felsic and
mafic pools less the basement). As shares of the planet at the end (the arm's 8 runs; the switch-off
run alone):

| own crystalline crust | switch-off run | m5-margins | m5-margins, depth below the sea (p50) |
|---|---|---|---|
| 0 to 10 km (the converted margins and the arcs above) | 0.004 | 0.113 | 3,472 m |
| 10 to 15 km | 0.006 | 0.073 | 2,535 m |
| 15 to 20 km | 0.007 | 0.076 | 1,705 m |
| 20 to 25 km | 0.015 | 0.085 | 940 m |
| 25 to 30 km | 0.061 | 0.118 | 190 m |
| 30 km and more (the cores) | 0.309 | 0.159 | −16 m |

Both runs start with 0.423 of cores at 33 km of their own (felsic plutonic 21, metamorphic 6, mafic
plutonic 4.5). In the arm the 10 to 30 km bands lie on a mixing line between a converted margin (own 7
km, basement 6.0, felsic plutonic 4.6) and a core (own 32 km, felsic plutonic 21.4): across the bands the
basement falls from 5.1 to 1.8 km and the felsic plutonic rock rises from 8.6 to 19.5 km. The own
crystalline rock is about 13.3 km spread over the planet against 12.5 on the switch-off run, so the same
rock covers 1.5 times the area. The mean continental thickness falls from 36.1 to 26.9 km (m5sea); the
cores thin from 35 to 40 km to 30 to 35 km of crust and still sit at the sea.

The growth comes right after a resampling (m5resample.py, from the record: the change in the continental
count by steps since the last resampling, about 70 resamplings a run):

| steps since the resampling | base15, per 100 steps | m5-margins |
|---|---|---|
| 0 (the resampling's own step) | +0.027 | +0.016 |
| 1 | −0.035 | +0.296 |
| 2 | −0.016 | +0.010 |
| 3 and later | about −0.008 | about −0.008 |
| net over the run | −0.038 | +0.204 |
| of it in the first 3 steps after a resampling | −0.019 | +0.220 |

The reading that fits all of it: the resampling's blend takes a continental sample's conserved pools
and basement from its continental corners only (ridges.rs), so a new sample that falls between a
converted margin and the core it borders gets a mix of their crust, 10 km or more of its own. It is then
no longer hyper-extended, so it is the taper's continental side, and in the next step the ring beyond
it, which holds the continents' sediment (the ring split above), is in the zone and converts. Each
resampling moves the boundary out a ring. The crust budget's parts agree (supercontinent-1, the arm's
run against base15's): over the run the transition added 0.321 of the planet to the continents' area,
against no entry on base15, while subduction took 0.292 against 0.310. The collapse added 0.006 against
0.061, and moved 1.9 × 10⁸ km³ against 9.5 × 10⁸, so PLT-662's path is not the route. The channel flow
fell from 9.2 × 10⁸ to 1.9 × 10⁸ km³, and the ocean floor's felsic rock accreted at resamplings from
5.6 × 10⁹ to 7.9 × 10⁸.

This is numerical diffusion through the blend, not a physical law and not Brune's width: Earth's
hyper-extended crust does not take the craton's crust each time the model renumbers its samples. It is
the leading reading, not yet isolated by an arm or a unit test of a resampling.

### Why the measures went out

Land, the largest landmass and M3 go out for one reason. The continents' crystalline rock is spread
over 1.5 times the area, so the crust is thinner and stands lower: 0.35 of the planet is continental
crust with 10 to 30 km of its own, 190 to 2,535 m under the sea. The sea fell 466 m, but the
continents fell further. M4 and the ice fall with the relief: the crust that crosses a collision front
is 18.9 km against 30.6, the collision's loss per km halves (62 against 128 km³ per km and Myr), and
the share of crossing crust above the normal column falls from 0.022 to 0.003, so the belts stand
lower. The coast with high ground comes in for the same reason. The route is the ratchet, so these are
findings about the blend, not readings of either stage's physics. No knob was added to hold a margin,
and the taper was not widened to reach rings 2 and beyond.

### The crust cycle against Stern & Scholl and Clift

loss-budget.py, km³ a year over the run (means of 8):

| | base15 | m5-margins | Earth |
|---|---|---|---|
| island arcs | 1.20 | 0.86 | 1.5 (Stern & Scholl 2010) |
| continental arcs | 0.34 | 0.28 | 1.0 |
| arcs together | 1.54 | 1.14 | 2.5; 3.8 to keep the volume (Clift et al. 2009) |
| rifts and hotspots | 0.04 | 0.04 | 0.5 |
| ocean floor's own crust made continental | 0.04 | 0.04 | |
| sediment subduction | 1.05 | 2.03 | 1.1; 1.65 (Clift et al. 2009) |
| subduction erosion | 0.33 | 0.27 | 1.4; 1.33 (Clift et al. 2009) |
| suturing (all three) | 1.21 | 0.62 | 0.7 |
| resampling drops | 0.12 | 0.11 | |
| net change of the continental volume | −0.80 | +0.09 | 0 |

(Clift et al. 2009's sediment subduction and erosion as Stern & Scholl 2010's Table 1 quotes them.)
The losses together are 3.03 against Earth's 3.2, the additions 1.22 against 3.2. Sediment subduction
is now above both Earth figures: the share is Earth's, and the pile it acts on is thicker than Earth's.
The suturing loss halves to Earth's, with the thinner crust at the fronts. The continental volume holds
(+0.09) through relabel flows, not made crust: crust made continental 1.92, the ocean floor kept under
it as basement and not counted 1.98, and the transition taking 0.81 onto the continents.

So the arcs are now the binding gap: 1.14 km³ a year against 2.5 to 3.8, worse than base15's 1.54. They
fell with their fronts, the island-arc front from 44,237 to 37,541 km and the continental-arc front
from 25,736 to 21,301, and the island arcs per km from 27.1 to 22.8 km³ per km and Myr (Clift et al.
2009: 125 for oceanic arcs; continental arcs 13.1 against 81). Reported, not fixed.

### The sea's sediment

m5cont.py's count of the sediment and sedimentary rock under the sea, both kinds of crust (GlobSed's
grid covers the margins too): 2,099 m planet-mean, 1.07 × 10⁹ km³, in the arm against 2,136 m,
1.09 × 10⁹ km³, on the switch-off run, about 3.2 times GlobSed's 3.37 × 10⁸ km³ (Straume et al. 2019).
On continental crust: 1,530 m against 404. The stock was relabelled, not reduced. (b) cannot reach it,
since it acts on oceanic floor only, and the margins (a) turned continental now hold it.

### What it is not

- Not on by default. Both switches stay off: the arm's measures move through the ratchet, a numerical
  route, so they read the blend, not the stages.
- Not a reading of (b) alone. No arm ran it alone, and the land PLT-440's note said it drowned (on
  frozen plates) is not read again by this arm.
- Not a tuned width. (a) stays at Brune's taper, ring 1 on the quick planet.
- The next step proposed to the coordinator: close the blend, so that a resampling blends a continental
  sample's pools within its class (hyper-extended or not) and a converted margin stays one, tested on a
  resampling first; then one (a)+(b) arm against base15.

### The blend closed (PLT-663) and the orthogonal rate (2026-10-25)

The coordinator approved both: close the blend, with or without the switches, and read PLT-660 on the
orthogonal convergence; then one cumulative arm (the blend fix, the orthogonal rate, (a) and (b))
and a switch-off arm, with no arm of (b) alone, both on master c5203e38 (Law 1, PLT-655) and read
against base16's seeds 1 to 4.

**The class blend** (PLT-663, `ridges::interpolate`). A resampling blends a continental sample's rock
pools and its basement over the corners of its own class only: hyper-extended (under 10 km of its own
crystalline crust, PLT-661's line from Brune et al. 2014) or not, the class of its major corner, as
the kind already is (PLT-293). A column's own crystalline crust is its crystalline pools less its
basement, both blended with the same weights, so a blend over one class is a convex mean of columns
on one side of the 10 km line and stays there. The sediment pool still blends over all three corners,
the heat flow over the kind. There is no switch: with the switches off it keeps an old arc's thin
continental crust apart from the thick crust beside it too.

The unit test `a_resampling_keeps_a_converted_margin_apart_from_its_core` (written first, failing on
the old blend) puts a continent (z > 0) and its own ocean floor out to z = −0.3 under 2,500 m of its
sediment on one plate. The first transition step turns 271 columns of the zone and a second turns
none. Then:

| | old blend | class blend |
|---|---|---|
| hyper-extended columns after one resampling (6 lattices; 271 before) | 213 to 224 | 265 to 280 |
| columns mixing the classes (a basement under 10 km or more of own crust) | 177 to 194 | 0 |
| the core after six resamplings in a row, each followed by a step (9,998 before) | 10,117 (+1.2 %) | 10,002 (+0.04 %) |
| mixed columns after the six | 975 | 0 |
| columns turned at the step after each of the six | 69, 32, 43, 25, 10, 20 | 40, 29, 27, 31, 14, 29 |

| hyper-extended columns at the step after each of the six | | 314, 343, 364, 393, 411, 441 |
| the zone's lowest z after each of the six (−0.031 before) | | −0.043, −0.054, −0.062, −0.071, −0.074, −0.074 |

The class blend closes the mixing and keeps the core, but not the zone: it still grows, by the
columns the step after each resampling turns, and reaches about a ring past where the rule puts it
(the rule's zone is the ring beside the core, as on the quick planet). This is a residual ratchet, and the first reading of the last row (the zone's edge
refilled, the zone in place) was wrong. A new sample takes the class of its major corner, so the
lattice's noise both drops oceanic samples into the ring beside the core, which the next step turns,
and carries hyper-extended ones past that ring, which nothing turns back: the conversion is one-way.
The resamplings themselves keep the count (271 to 274, 314, 343 to 337, 364 to 362, 393 to 397, 411
to 412); every step adds. The test prints the count and the reach and does not assert on them.

**The orthogonal rate** (PLT-660). Clift & Vannucchi's 76 mm/yr is an orthogonal rate, so the
resampling's share now reads the convergence normal to the boundary (PLT-312's `across`):
`subduction::orthogonal_convergence_mm_yr`, the stamped speed times the cosine of the stamped
obliquity. For that the front now measures the obliquity from the boundary's normal whatever
`DEEPTIME_ARC_NORMAL` sets; until now, with the switch off, it was measured from the lattice edge to
the converging neighbour, which zigzags about the normal. A field of its own was tried first and
dropped: 4 B more on every plate sample took full 16k's memory estimate to 5,370,649,280 B, 1.85 MiB
past its 5 GiB cap (`the_estimate_covers_the_measured_hold_closely`). The unit test
`a_front_stamps_its_orthogonal_convergence`: an ocean coming in at 50 mm/yr on a straight 40-degree
front gives a median normal over whole rate of 0.998 head-on and 0.557 turned 60 degrees towards the
front (cos 60° = 0.5; 0.48 at the front's ends).

The arcs, the uplift and the losses do not read the obliquity, so the plates' state is unchanged by
it; what changes is the obliquity the samples carry out of the plate stage: the L1 field
`plates/obliquity_deg`, which the structure stage's strike-slip shear (DEF-029, above 30°) and the
porphyry deposits' obliquity curve (ORE-033, peak near 15°) read. Both now get the angle CRE-018 and
the deposit field's description (`deeptime:obliquity_deg`, "convergence obliquity to the trench
normal") already said they got.

#### The arms m5-blend and m5-blendoff against base16 (2026-10-25)

Two arms on the branch merged with master c5203e38: m5-blend (key 7f14a0031e94, argkey 5818ecce:
`tectonics.sediment_subduction_sourced` and `tectonics.margin_sediment_continental` on) and
m5-blendoff (the same key, argkey cd665cef: both off), each both starts by seeds 1 to 4,
earthlike_quick_16k, 1,000 Myr, with the crust-budget maps. Both are read against base16's seeds 1
to 4 (fdb71fad7135, cd665cef): medians of 8, Mann-Whitney p against base16 in brackets.

| | target | base16 | m5-blendoff | m5-blend |
|---|---|---|---|---|
| land | 0.13 to 0.36 | 0.226 | 0.244 (0.80) | 0.105 (< 0.001) |
| largest landmass | ≥ 0.06 | 0.095 | 0.065 (0.57) | 0.027 (0.015) |
| continental crust under the sea (M3) | ≤ 0.56 | 0.448 | 0.405 (0.65) | 0.836 (< 0.001) |
| land above 2 km | ≤ 0.07 | 0.031 | 0.030 (0.80) | 0.004 (< 0.001) |
| coast with high ground | ≤ 0.11 | 0.099 | 0.134 (0.065) | 0.034 (0.002) |
| land under ice | ≤ 0.32 | 0.079 | 0.053 (0.96) | 0.013 (0.001) |
| collisions per 100 Myr | ≤ 6 | 4.25 | 5.00 (0.082) | 5.55 (0.009) |
| continental crust's net change | | −0.043 | −0.062 (0.083) | +0.214 (< 0.001) |
| criteria met: median, mean | of 7 | 6, 6.12 | 6, 5.50 (0.11) | 4, 4.00 (< 0.001) |
| sea over the datum after 100 Myr / end, m | | 428 / 564 | 425 / 537 | 142 / 190 |
| guard's worst, m | | 10,014 | 9,992 (0.87) | 9,640 (0.040) |
| highest plate elevation, m | | 5,064 | 5,354 (0.51) | 2,857 (< 0.001) |
| M4 p50 (m4table) | | 0.0312 | 0.0305 | 0.0036 |

**The switch-off planet.** Its world state and its dynamics record differ from base16's in every run
(106 or 107 of the 109 state files): the class blend acts with the switches off, on the thin
continental crust of matured arcs, and the plates take another path from there. No census measure
moves at p < 0.05; the coast with high ground goes past its 0.11 (p 0.065). Its loss budget is
base16's within the runs' spread (island arcs 1.29 against 1.31 km³ a year, sediment subduction 1.10
against 1.12, collisions 1.80 against 1.66, net −1.25 against −0.99), and so is the share of the
continental change in the steps after a resampling (m5resample: −0.040 against −0.038 of the planet
per 100 steps at the first step). It is the arm the stages are measured from: it has the pools maps
base16 lacks.

**The cumulative arm** comes out as m5-margins did on the old blend (land 0.114, M3 0.816): closing
the blend did not change what the stages do to the planet. By part:

- *The relabel.* Continental share 0.361 (switch-off) to 0.637 with the converted margins, 0.508
  without them (m5margins, plate samples, t0000; Cogley 1984 0.41). M3 0.350 to 0.816 with, 0.765
  without (Cogley 0.306). The converted margins are 0.126 of the planet, all wet, 2,983 m deep, crust
  17.5 km, sediment 7.8 km. Continental crust under 10 km of its own crust with 1 km or more of
  island-arc magma is 0.079 against 0.005 off: the maturity rule (15 km, PLT-331) is the same on both
  arms, so most of it is likely arc-flank ocean floor (b) turned for its arc-derived sediment. Less
  both, the share is about 0.43 (by difference of medians).
- *The cores thin.* Share of the planet by own crystalline crust (m5cont):

| map | arm | 0-10 km | 10-15 | 15-20 | 20-25 | 25-30 | ≥ 30 (the core) |
|---|---|---|---|---|---|---|---|
| t-1000 | both | 0 | 0 | 0 | 0 | 0 | 0.423 |
| t-500 | off | 0.006 | 0.005 | 0.007 | 0.017 | 0.057 | 0.318 |
| t-500 | m5-blend | 0.124 | 0.011 | 0.021 | 0.042 | 0.104 | 0.261 |
| t0 | off | 0.005 | 0.004 | 0.006 | 0.021 | 0.073 | 0.254 |
| t0 | m5-blend | 0.202 | 0.013 | 0.037 | 0.087 | 0.147 | 0.152 |

  The arm's 0-10 km band at t0 lies 3,051 m deep under 7.3 km of sediment on a 7.6 km basement. The
  sea's sediment and sedimentary rock is 2,432 m planet-mean (1.24e9 km³), 1,881 m of it on
  continental crust; off, 1,996 m (1.02e9 km³), 290 m of it on continental crust. Both are about three
  to four times GlobSed's 3.37e8 km³: the stages move the excess into the continental count, they do
  not make it.
- *The thick oceanic crust goes* (m5thickoc, share of the ocean, base16 against m5-blend): 0-8 km
  0.494 to 0.550, 8-10 0.120 to 0.204, 10-15 0.149 to 0.188, 15-25 0.159 to 0.041, 25 and more 0.082
  to 0.001. Its hold on the sea (m5sea, the sea with the oceanic crust of 10 km and more at the bare
  floor's height) falls from 844 m (base16) to 89 m, so the sea falls to 190 m.
- *The growth comes after the resamplings* (m5resample, the continental change at the first step
  after one, share of the planet per 100 steps): base16 −0.038, off −0.040, m5-blend +0.347
  (m5-margins on the old blend +0.296); the resampling's own step +0.002. Of the arm's +0.216 net over
  the run, +0.254 comes in the first 3 steps after a resampling. That is the residual ratchet's
  pattern in the unit test above: the resampling keeps the count and the step after it adds.
- *The zone's reach* (`tools/review/m5diag/m5hxreach.py`, hyper-extended continental crust by its
  distance from 10 km of its continent's own crust, km past the boundary half a spacing out, the
  quick planet's spacing 68 km): at t0 0.202 of the planet, 0.079 of it arcs; of the rest 0.090
  within 100 km (the rule's W/2), 0.028 at 100 to 200 km, 0.007 at 200 to 500 km. Beyond W/2 0.002 at
  t-900, 0.016 at t-500, 0.036 at t0, 3,331 m deep. The maps cannot split this between the ratchet
  and a core edge thinned under 10 km. Off, 0.005, nearly all arcs.

**The crust cycle** (loss-budget.py, km³ a year; base16, m5-blend; Stern & Scholl 2010 Table 1 and
Clift et al. 2009):

| | base16 | m5-blend | Stern & Scholl | Clift et al. |
|---|---|---|---|---|
| island arcs | 1.31 | 0.91 | 1.5 | |
| continental arcs | 0.37 | 0.29 | 1.0 | |
| arcs together | 1.68 | 1.20 | 2.5 | 3.8 needed |
| rifts, hotspots, rifted margins | 0.04 | 0.05 | 0.5 | |
| sediment subduction | 1.12 | 2.01 | 1.1 | 1.65 |
| subduction erosion | 0.34 | 0.28 | 1.4 | 1.33 |
| collisions | 1.66 | 0.78 | 0.7 | |
| ocean-floor felsic rock accreted at resamplings (a flow) | 6.36 | 0.77 | | |
| off the continents by the surface stage (a flow) | 4.05 | 1.66 | | |
| net change of the continental volume | −0.99 | −0.26 | 0 | |

The island-arc front shrinks from 42,258 to 33,094 km (Earth 14,500), the continental share having
grown, and collisions lose 67 km³/km/Myr against 187 (Earth about 115) over a front of 11,554 km
against 8,860. Sediment subduction doubles to 2.01, past Clift's 1.65: PLT-660 sends 0.83 to all of
a pile down where PLT-440 sent 0.12, and the piles are the three-to-four-fold sediment stock above.

**The physical reading.** The stages close what they were built for: the oceanic crust of 15 km and
more (0.24 of the ocean to 0.04) and its hold on the sea. They also cut base16's return path. In
base16 the surface stage carries 4.05 km³ a year off the continents and trench accretion at the
resamplings (PLT-440's 0.88 kept) brings 6.36 back to the continents' edges. With (a) the slab takes
0.83 or all of a pile and with (b) the margins' sediment is continental crust where it lies, so the
return is 0.77. Eroded crust stays on the converted margins, under 10 km of its own crust at 3 km
depth, or goes down the slab. The cores thin (the 30 km band 0.423 to 0.152, against 0.254 off), and
their land drowns although the sea falls 374 m: land over continental crust 0.165 against 0.68 off.
The arm moves the measures out through a real loss of thick crust, not a bookkeeping one, with the
residual ratchet adding to the converted area at each resampling. No knob was added.

#### What sets the arc rate (research, 2026-10-25; nothing built)

**In the code** (`subduction.rs`, the fronts' plan loop). A sample in an oceanic slab's distance
field gets magma R / (√π × 60 km) × w_arc(d) × g a Myr: R 100 km³/km/Myr over ocean crust (PLT-045),
35 over continental crust (PLT-285), a Gaussian band 60 km wide at 200 km from the trench, g =
min(conv / 100 mm/yr, 1) (PLT-037) with conv the whole relative speed (the normal component only
with `DEEPTIME_ARC_NORMAL`, PLT-312). Nothing past the slab's ends (PLT-561), over an incipient slab
(PLT-610) or at a collision front; less the felsic cones (PLT-286). The global addition is the sum of
front length × R × g. Nothing reads the arc's age, the slab's age or dip, the sediment coming in, the
water flux, the overriding lid's thickness or back-arc extension. On base16 the island arcs add 30.9
km³/km/Myr at a mean g of 0.467 (0.383 on the normal component), the continental arcs 13.8 at 0.299.
Stern & Scholl's 100 and 35 are averages over Earth's arcs at Earth's convergences, so multiplying
them by g counts the convergence a second time unless Earth's mean convergence were 100 mm/yr.

**In the sources** (read by a research sub-agent, 2026-10-25; bibliography: SternScholl2010,
Clift2009, JichaJagoutz2015, Parcutela2023, Bai2019):

- *Arc age*, the main argument of Stern & Scholl 2010 (pp. 8, 11-12, 25): growth is front-loaded,
  "perhaps several hundred to as much as 500-600 km3/km-million years" for a few Myr after
  initiation, then "a few tens or less" for mature arcs; the conclusions give 300-500 then 30-35.
  Their long-term intra-oceanic 100 is 500 for 10 Myr then 30 for 60 Myr. Clift et al. 2009 (p. 97)
  name "pulses of enhanced mafic accretion ... associated with subduction initiation and
  arc-extension events", with the Mariana arc's back-arc splits as their case, and no numbers.
- *Convergence.* Stern & Scholl (p. 7) cite a model (Cagnioncle 2007) in which melting "increases
  with increasing convergence rate", with slab dip, mantle temperature and fluid flux as other
  controls, and test none. Clift et al. take it as a premise (p. 84, "based on the premise that
  production rates are linked to the supply of aqueous fluids ... and thus to the rates of
  convergence"): their Table 1's productivity is about 1.3 to 1.4 times the convergence row by row,
  so it encodes the premise rather than testing it. On a melting-column premise the oceanic arcs'
  share is 37 % against 40 %.
- *The overriding lid.* Clift et al. (p. 98): a thin lid "allows significant upwelling under the arc
  and thus more melting compared to a mature continental arc". Stern & Scholl put the oceanic
  against continental difference down to lifespan (p. 13), not the crust.
- *Slab age, dip and sediment.* Not in Stern & Scholl or Clift et al. as controls on the rate.
  Parcutela et al. 2023 attribute the Western Pacific arcs' higher rates (28-60 against 12-25) to
  "the variation in the ages of the subducting plates". Bai et al. 2019 (a discussion preprint) find
  the Aleutian rate (37 to 89, mean 61) follows the slab's dip, "The correlation coefficient between
  the rates and dips is 0.83", where the convergence and the plate's age run the other way along the
  arc.
- *Size.* Clift et al.'s 3.8 km³ a year (94 km³/km/Myr over 40,225 km) is a mass balance against
  their losses, "on the high side of what is tenable", 2.0 at the low end; their seismic arcs give
  Aleutians 55-82 and Izu 66, 106 with Izu's lost forearc. Jicha & Jagoutz 2015 (abstract only)
  find intra-oceanic arcs' rates "consistent with those calculated for mid-ocean ridge segments that
  have slow to moderate spreading rates"; Bai et al. cite them for the Aleutians' 89.

So the sources agree on arc age (a front-loaded or pulsed rate) and on a per-km rate times the
arcs' length, put convergence in by a model or a premise, and leave slab age, dip and back-arc
extension to single studies. The code has the length and a convergence factor on top of averages
that already hold Earth's convergences, and no arc age.

## The sea floor's age and the initiation's polarity (2026-10-25)

The floor-age diagnosis (build log 2026-10-25, "Front lane: why floor older than 180 Myr does not go
under on base15") found 0.24 of base15's ocean older than 180 Myr late in the runs (Earth 0.006), from
two causes of about equal weight: slow turnover, and consumption blind to age. Where the old floor sat
could not be read from the cache, and in about three initiations of four the trench took the opposite
polarity to the slab planted under the fracture. The coordinator's order (2026-10-25): an age map
first, then the polarity, then the sourced constants, then a design for propagation.

### The age map (measurement only)

With `DEEPTIME_CRUST_BUDGET_MAPS` set, every crust-budget snapshot also writes `maps/age/t<Myr>.bin`:
one little-endian f32 per record of the snapshot's map (`maps/t<Myr>.bin`, the visible samples in
order), the floor's age at the snapshot (`Sample::age_myr`) on oceanic crust and NaN on continental
crust (`crust_budget.rs`, `write_ages` and `age_records`; the test
`the_age_records_are_the_ocean_ages`: the oceanic records, sorted, are `dynamics::ocean_ages`, which
the dynamics record's age quantiles read). Nothing reads it in the model.
`tools/review/floorage/floorage_age.py` reads it with the map and `maps/plate/`.

#### Where the old floor sits (2026-10-25)

Four pool runs carry the map (arm fl-agemap, key b95402b27d42: seeds 1 and 2, both starts, base15's
environment, 2 threads). Each run's state and record hashes equal base15's: the map measures and does
not act. On master 1b210fac (arm fl-agemap-1b21, key 153f5de6d52c) the records are base15's too and
the world differs in one field of 109, the trap structure, which DEF-081 (structures drawn past a
pole are not started) changes on purpose. `floorage_age.py` reads the 33 snapshots from -500 Myr on (none at a resampling, where the
map hides nothing and flags no front) and checks each against the record: the map's oceanic count is
the record's, and its count older than 180 Myr is the record's `old180`. Pooled over the four runs:

- **Floor older than 180 Myr is 0.234 of the ocean.** Of it, 0.809 lies in the margin zone, within
  331 km of its own plate's continent (PLT-503's reach plus one spacing: the floor an initiation keeps
  on the parent); 0.546 lies farther than 1,000 km from every trench (PLT-064's clearance); 0.325 sits
  on plates over a fifth of the planet; 0.067 on plates with no trench; 0.007 on plates with no
  convergent front at all.
- **Exclusively**, margin 0.809, beyond the margin and the clearance 0.074, open floor within 1,000 km
  of a trench 0.116. The margin's share rises with age: 0.165 of the floor at 0 to 10 Myr, 0.356 at
  60 to 100, 0.549 at 150 to 180, 0.668 at 180 to 250, and 0.900 at 250 Myr and over (0.143 of the
  ocean).
- **Its own plate goes down far from it.** The old margin floor's distance to the nearest trench of
  its own plate: 0.147 within 500 km, 0.153 at 500 to 1,000 km, 0.269 at 1,000 to 2,000 km, 0.388
  beyond, 0.044 on plates with none. The floor its plate feeds that trench is of any age: 0.215
  younger than 30 Myr, 0.369 older than 180.
- **Initiation's view.** Floor older than 150 Myr (PLT-062's gate): within 1,000 km of a trench 0.482,
  kept by the margin (PLT-503) 0.417, exposed to initiation 0.093, too few samples or no seed 0.009.
  With Crameri et al. 2020's oldest initiation age, 117.4 Myr, as the gate, the exposed floor goes
  from 0.027 to 0.045 of the ocean. This status reads the map's end-of-step flags, without the hidden
  trench samples the clearance also sees, so it is close, not exact.
- **By start.** The supercontinent start keeps 1.13 plates over a fifth of the planet per snapshot
  against the scattered start's 0.67, and 0.42 of its old floor on them against 0.24; the margin's
  share is the same in both (0.81).

So the old floor is passive-margin floor that no trench reaches along the margin, not interior floor
beyond the clearance, nor floor on plates that never go down, and the age gate holds back little.
This points at how a trench comes to a margin (propagation along strike) more than at the gate's age
(PLT-062) or incipient subduction's strength (PLT-612), which acts only at a front between two plates,
where the margin's floor is not.

### Law 1: the older side of the fracture goes under (PLT-655)

An initiation (PLT-062 to PLT-067, PLT-503) splits the old floor off its plate and plants a slab under
the fracture (PLT-270), always on the split. The trench that formed a few steps later took its
polarity from the rule of a new contact, which reads the overlap's crust and ages, and on base15 it
put the parent under in 1,494 of the 1,998 such contacts the records hold, at a median 6 Myr after the
split: the planted slab and the trench's slab then pulled against each other. Gurnis, Hall & Lavier
2004 settle which side goes down at a fracture zone in compression: "the older plate is thrust beneath
the younger" ([60]), and with an asymmetry in age "the weak zone overlies the thicker plate or where
the negative buoyancy is localized" ([78]).

With `tectonics.initiation_polarity_by_age` (on by default):

- **The fracture's sides.** The visible samples of the parent with a lattice neighbour of the same
  rigid group, visible, on the other side of the split: one spacing each side (about 180 km on the
  quick lattice), the nearest floor the lattice resolves.
- **Which goes under.** A side whose fracture samples are half or more continental crust does not
  (the rule of a new contact's reading of an overlap); otherwise the side whose fracture floor is
  older on the mean, the split on a tie, with both sides continental, or with no fracture found.
- **The slab and the trench agree.** The planted slab hangs from that side, the split's or the
  parent's. The contact the two plates form later takes the planted slab's polarity
  (`subduction::planted_polarity`: a new contact between a plate and its parent with a slab one way
  between them), not the overlap's.

No value of its own. Tests: `the_older_side_of_the_fracture_goes_under` (`rifting.rs`: a plate of old
floor split at x = 0, the slab hangs from the parent where its side is older, from the split where the
split's is, on a tie, or where the parent's side of the fracture is continental; off, from the split)
and `an_initiations_contact_takes_its_planted_slab` (`plates/tests.rs`: two oceanic plates, the
older turned under the younger; with the younger's slab planted under the older and the older split
from it, the contact puts the younger under; off, with no split, or with no slab, the older).

**On the pool (arm fl-law1, key fdb71fad7135, argkey cd665cef: seeds 1 to 4, both starts, base15's
settings and the age maps, 2 threads; against base15's 24 runs, medians, Mann-Whitney p in brackets).**

| | base15 | fl-law1 |
|---|---|---|
| land (0.13 to 0.36) | 0.213 | 0.226 (0.54) |
| largest landmass (≥ 0.06) | 0.069 | 0.095 (0.48) |
| continental crust under the sea (≤ 0.56) | 0.510 | 0.448 (0.22) |
| land above 2 km (≤ 0.07) | 0.025 | 0.031 (0.72) |
| coast with high ground (≤ 0.11) | 0.109 | 0.099 (0.65) |
| land under ice (≤ 0.32) | 0.067 | 0.079 (0.92) |
| collisions per 100 Myr (≤ 6) | 5.10 | 4.25 (0.020) |
| criteria met per run, median and mean | 6, 5.79 | 6, 6.12 (0.53) |
| sea over the datum, after 100 Myr / end, m | 500 / 697 | 428 / 564 |
| the guard's worst, m from the sea | 10,106 | 10,014 (0.18); no step beyond 2 Earth reliefs |

No V1 measure breaches its target; only the collisions move beyond chance. In the records (polarity.py),
the contacts between an initiated plate and its parent still put the parent under in 0.75 of them
(710 of 947), at a median 6 Myr after the split, as on base15 (0.75): the parent's side of the
fracture is the older because the floor the initiation leaves on it is the margin's kept floor
(PLT-503). What the law changes is that the planted slab now hangs on the same side, so the trench
forms on a slab that pulls with it. The initiated plates die younger less often (dead within 20 Myr
0.13 of those born after -900 Myr, against 0.22), reach a front within 20 Myr more often (0.70
against 0.65) and keep more of the trench: the subduction front is 79,042 km against 68,762, its
flux 3.04 km²/yr against 2.64, the area-weighted plate speed 22.5 mm/yr against 20.8. The floor is
younger as a result, though no rule reads its age to that end: late mean age (from -500 Myr, the
records) 112 Myr against 126 (p < 0.001), older than 180 Myr 0.197 of the ocean against 0.244
(p < 0.001); the hazard of going down stays flat to 220 Myr instead of falling after 200, and the
floor each resampling takes is 0.81 of the standing mean age against 0.75. The age maps (67
snapshots from -500 Myr, against fl-agemap's 33, base15's seeds 1 and 2): older than 180 Myr 0.202
of the ocean against 0.234, 0.791 of it in the margin zone against 0.809; the margin zone holds
0.419 of all the floor against 0.440.
