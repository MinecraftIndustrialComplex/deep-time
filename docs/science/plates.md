# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207, PLT-229, PLT-251 to PLT-252, PLT-285 to PLT-286, PLT-289, PLT-312, PLT-320 (49; PLT-320 the cordillera at Earth's typical arc, with no convergence factor, the crust budget against Stern & Scholl, 2026-10-20; PLT-312 the arcs on the convergence normal to the front, an experiment kept off, the land-shape task, 2026-10-20; PLT-289 ocean floor as thick as a continent turns continental, the resampling lumps, 2026-10-18; PLT-285 and PLT-286 the continental arcs and the cones as rock, plate task 8, 2026-10-17) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081, PLT-222 to PLT-227, PLT-241 to PLT-250, PLT-253, PLT-290 to PLT-292, PLT-400 (35; PLT-400 the wedge's crust crosses whole, a switch kept off, the crust flux into collision fronts, 2026-10-21; PLT-290 to PLT-292 the crust a collision took counted once, the resampling lumps, 2026-10-18; PLT-069 to PLT-075 and PLT-227 retired 2026-10-02) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212, PLT-228, PLT-293, PLT-350 to PLT-355 (26; PLT-293 ocean crust keeps its own rock at resamplings and in the L1 sampling, 2026-10-19; PLT-350 to PLT-353 the floor's GDH1 rate with the sea from the ocean's volume, 2026-10-21, PLT-354 the gap's floor held in place and PLT-355 GDH1's curve between other ridge and abyssal depths, on [sea-level.md](sea-level.md)) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208, PLT-254 to PLT-259, PLT-261 to PLT-264, PLT-277 to PLT-284, PLT-287 to PLT-288, PLT-294 (44; PLT-294 the switch for the transition's ocean side, ocean crust or the continent's rock, 2026-10-19, on in the shipped presets since 2026-10-20; PLT-287 and PLT-288 the fastest thickening and rise, 2026-10-18; PLT-283 the gap budget and PLT-284 the accretion's height, 2026-10-17; PLT-254 to PLT-259 the orogenic collapse and the thickest crust, 2026-10-03; PLT-261 to PLT-264 the ocean–continent transition, 2026-10-04; PLT-277 spreading before foundering, 2026-10-05; PLT-278 to PLT-282 the eclogite root and whole-column isostasy, 2026-10-06) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173, PLT-213 to PLT-221, PLT-265, PLT-310 to PLT-311 (43; PLT-310 and PLT-311 the boundary's normal and Bird's transform test, the land-shape task, 2026-10-20; PLT-265 Earth's passive margins, the transition's comparison) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-180, PLT-191 to PLT-202 (19) |
| [PLT](ledger/PLT.md) | the force balance's settings, coded and recorded, not applied (page [plate-forces.md](plate-forces.md); constants in [FRC](ledger/FRC.md)) | PLT-181 to PLT-190 (10) |
| [PLT](ledger/PLT.md) | the continents census's Earth references and measuring choices (`tools/review/continents_census.py`, section "White patches, drowned continents and coastal high ground", 2026-10-21) | PLT-360 to PLT-363 (4) |
| [PLT](ledger/PLT.md) | the crust flux into collision fronts: the crust budget's measures and Earth's references (section "The crust flux into collision fronts", 2026-10-21) | PLT-401 (1) |

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
   column (mafic root first) and gives no area.
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
   200 km measured **on the planet**, PLT-262) gets a signed distance d from it (inland positive)
   and a target crystalline thickness (the crust less its sediment and sedimentary rock)
   T*(d) = T_o + (T_n − T_o) · clamp(1/2 + d/W, 0, 1), with T_n = 33.0 km (the normal continental
   column's) and T_o = 7 km (the ocean's). The profile is linear in thickness: β = T_n / T* runs
   from 1 at the hinge to T_n / T_o ≈ 4.7 at the toe, and centred on the model's boundary, so
   turning the one-sample step into the ramp keeps the crust's volume.
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

**Earth against the model.** Measured 2026-10-04 on `earthlike_quick_16k`
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
- The profile is linear in thickness and the same on every margin; Earth's margins have a necking
  zone where the crust thins fastest, and volcanic and magma-poor margins differ in width by a
  factor of two to three. One width (200 km, inside the sources' 50–300 km) stands for all.
- The width is in metres on the planet, so a small planet's margins are as wide as Earth's in
  metres and take a larger share of its continents; on the quick 16k planet the zone is 23 samples
  across.
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
general flexure already bends the plate under every load (decisions log 2026-10-02).

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
   (7.0 times the uplift, not 5.5).

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
(W/2 = 100 km = 21 cells), and switching the transition off (`transition_width_km` = 0) takes
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
7–8") and is not changed here.

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
| margin taper 24.5 km instead of 200 (`transition_width_km`) | 0.323 / 0.315 | 0.346 / 0.329 | 155 / 149 | 0.39 / 0.33 | 726 / 917 |

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
island arc is about 26,700 lattice km.

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
times from the rate, and the front's length is the orientation above. On the plate-check table the
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
share's, which PLT-330 could not move either (32.1 to 34.4 % there).

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
   again.
3. *Where subduction starts* (Crameri): needs a law for trenches to propagate and slabs to roll
   back before initiation can move near trenches; a plate reorganisation is the open law (Ben's,
   as in the sections above).
4. *The plate count*: the lattice's floor (0.0377 sr at 100,000 samples); a finer lattice or a
   small-plate law, not a constant.
5. *The island-arc share of the land* (0.43): "a few per cent" (the section above, from memory)
   is not a figure Earth's sources give. By Stern & Scholl 1.5 of the 2.7 AU of crust grown at
   convergent margins is intra-oceanic arc, and the census tags any land with 1 km or more of
   island-arc crust as island arc. A share of the crust's volume against Stern & Scholl's
   Table 1 would be the measure with a source behind it.
6. *The sea off the datum* (the sea level's): a sea standing away from 0, above or below, loses
   about 0.026 more of the planet's continental crust over 1,000 Myr than a sea at 0 and shortens
   the assembly period (the section above). Find the code that still takes the datum for the
   sea; `forces::column_energy` (FRC-014: "h above sea level", computed from 0) is the first to
   test, by an arm that feeds it the sea.
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
  their fronts (the agent's estimate from the budget); 0.95 of what crosses stays and builds the
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
   window can go on.
2. *The active belts* (the plates'): 0.95 of the crust that crosses a collision front stays in
   the crust (PLT-242, `plates/belts.rs`, on Stern & Scholl 2010), where the India-Asia balance has
   about half of the colliding crust gone into the mantle (Ingalls et al. 2016, abstract; disputed by van Hinsbergen et al.
   2017, and Yakovlev & Clark 2014 conserve it if the crust before the collision was 23 to 29 km).
   A lower share alone would lose still more crust (the losses are already over Earth's because
   too much crosses); less crossing, from fewer or shorter collisions, is the plates' work on
   `plates/dynamics.rs`, and the share is worth a second look after it.
3. *The cordillera's height* (PLT-050): a sourced value from Profeta et al. 2015 in place of the
   calibrated 3000 m; measured above, it does not move the high ground by itself.
4. *Plateau interiors*: thick crust with no room within reach does not collapse. A lower-crustal
   flow that carries crust farther than the nearest room (PLT-300's flow does not reach) would
   be a flow solve, not a constant.

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
crust budget on (`DEEPTIME_CRUST_BUDGET=1`): the flux over the whole run, the criteria on the final
frame (`tools/review/continents_census.py fields`) and the plate checks of each run. Each value is
the median of the 12 runs; p is a two-sided Mann-Whitney U test on the runs against the base. The
base is master 49492ce3 as it runs (this branch with both switches off). The build log entry of
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
| base | master's behaviour |
| rs | PLT-400 on |
| rsrev | PLT-400 and PLT-371 on |
| rs50 | PLT-400 on, `collision_stacking_share` 0.5 (Ingalls) |
| rsw10 | PLT-400 on, `weld_speed_mm_yr` 10 (PLT-246, a game choice) |

Medians of 12 runs (p against the base):

@@ARMS_TABLE@@

### PLT-371's reversal on top

@@REV_TEXT@@

### What is left, and for whom

@@LEFT_TEXT@@
