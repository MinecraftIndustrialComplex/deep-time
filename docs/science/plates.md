# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207, PLT-229, PLT-251 to PLT-252, PLT-285 to PLT-286, PLT-289 (47; PLT-289 ocean floor as thick as a continent turns continental, the resampling lumps, 2026-10-18; PLT-285 and PLT-286 the continental arcs and the cones as rock, plate task 8, 2026-10-17) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081, PLT-222 to PLT-227, PLT-241 to PLT-250, PLT-253, PLT-290 to PLT-292 (34; PLT-290 to PLT-292 the crust a collision took counted once, the resampling lumps, 2026-10-18; PLT-069 to PLT-075 and PLT-227 retired 2026-10-02) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212, PLT-228, PLT-293 (20; PLT-293 ocean crust keeps its own rock at resamplings and in the L1 sampling, 2026-10-19) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208, PLT-254 to PLT-259, PLT-261 to PLT-264, PLT-277 to PLT-284, PLT-287 to PLT-288, PLT-294 (44; PLT-294 the switch for the transition's ocean side, ocean crust or the continent's rock, 2026-10-19; PLT-287 and PLT-288 the fastest thickening and rise, 2026-10-18; PLT-283 the gap budget and PLT-284 the accretion's height, 2026-10-17; PLT-254 to PLT-259 the orogenic collapse and the thickest crust, 2026-10-03; PLT-261 to PLT-264 the ocean–continent transition, 2026-10-04; PLT-277 spreading before foundering, 2026-10-05; PLT-278 to PLT-282 the eclogite root and whole-column isostasy, 2026-10-06) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173, PLT-213 to PLT-221, PLT-265 (41; PLT-265 Earth's passive margins, the transition's comparison) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-180, PLT-191 to PLT-202 (19) |
| [PLT](ledger/PLT.md) | the force balance's settings, coded and recorded, not applied (page [plate-forces.md](plate-forces.md); constants in [FRC](ledger/FRC.md)) | PLT-181 to PLT-190 (10) |

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
   own rock. A switch added on 2026-10-19, off by default (`transition_ocean_crust`, PLT-294,
   "Ocean crust keeps its own rock" below), gives an oceanic sink the same volume as ocean crust
   instead, lavas and gabbro in the ocean column's proportion. Volume is conserved, and crust types do not change: the thickened
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
Bird's classes are not the model's: his transform class is by azimuth against the relative
velocity, the model's "other" also holds converging boundaries that have no contact yet. The arc
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
- *The flow's substeps* (PLT-300): explicit, as many as keep it stable, at most 64 a step, past
  which the step moves less than the law. At the thickest channel (K = 8 K₀ = 12,000 km²/Myr) on
  six edges and the preset's mean spacing taken as a hexagonal lattice's (about 77 km at quick
  16k's 100,000 samples, 48 km at full 16k's 250,000), the cap holds the step to 3.1 Myr of the
  law at quick 16k and 1.3 Myr at full 16k, whatever the step. So it binds at quick 16k only at 4
  Myr (crust above about 78 km), and at full 16k at both (above about 76 km at 2 Myr, 70 km at 4
  Myr). At the thickest channel a 4-Myr step then moves crust 0.79 as far per Myr as a 2-Myr
  one at quick 16k, and half as far at full 16k. These are estimates from the constants; the substeps a step takes (`Flow::substeps`) are not logged, and
  how much crust is that thick was not measured.
- *The flow's reach.* Crust moves only among the channel columns and the two rings of their
  plate's crust around them, found once a step, so a channel's front can move at most two rings
  (about 150 km at quick 16k, 100 km at full 16k) a step, half as far per Myr at 4 Myr. K₀'s
  length over 15 Myr is Rey, Teyssier & Whitney's 150 km, √(K₀ t) = 77 km over 4 Myr at K₀ and
  220 km at 8 K₀; not measured here.

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
proportion), as the thick igneous crust of a volcanic margin would be. That removes the
transition's granite from the sea floor (the last column above), but the continental rock that
the sources give up is then lost, and land falls (`earthlike_quick_16k`, 1000 Myr, the end of the
run, seeds 1 to 3; "before" is master 49f5f543, which holds the continents fix bc2e5472, "after" the
branch's default; 2026-10-20):

| start | land, before | after | switch on | continental crust (10^9 km3), before | after | switch on |
|---|---|---|---|---|---|---|
| supercontinent | 0.264, 0.326, 0.321 | 0.309, 0.307, 0.344 | 0.277, 0.286, 0.252 | 7.5, 8.8, 8.1 | 8.0, 8.4, 9.1 | 7.9, 7.8, 7.0 |
| scattered continents | 0.289, 0.342, 0.282 | 0.356, 0.323, 0.294 | 0.221, 0.289, SCAT3LAND | 7.5, 8.6, 7.7 | 8.5, 8.7, 7.8 | 7.2, 7.7, SCAT3VOL |

The means: before 0.30 on both starts, after 0.32, switch on 0.27 (supercontinent) and SCATMEAN
(scattered), against Earth's 0.29. "After" against "before" is within the seeds' spread (0.26 to
0.34). On master d54c7f0f, before the continents fix, the switch cost more: supercontinent 0.140,
0.131 and 0.139 against 0.165, 0.189 and 0.177.

Ben weighed the transition's land cost on 2026-10-05 ("Keep 200 km, raise crust in 7-8"), so the
switch is off by default and the choice is his. The physical fix is a continent that widens as it
stretches, which the model's fixed boundary cannot do without the ratchet PLT-289 records.

**What is left under the sea** (switch on, seed 1 supercontinent on master bc2e5472, the table
above): all the arc cells and 0.73 of the back-arc cells hold island-arc rock (drawn as andesite over
diorite or granodiorite, RCK-096; the model's oceanic fronts are about 8.6 times Earth's length,
PLT-045), and 0.46 of the ocean basin, 0.21 of the ridges and 0.16 of the crust younger than 20 Myr
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
