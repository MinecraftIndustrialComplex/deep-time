# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207, PLT-229, PLT-251 to PLT-252 (44) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081, PLT-222 to PLT-227, PLT-241 to PLT-250, PLT-253 (31; PLT-069 to PLT-075 and PLT-227 retired 2026-10-02) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212, PLT-228 (19) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208, PLT-254 to PLT-258 (28; PLT-254 to PLT-258 the orogenic collapse and the thickest crust, 2026-10-03) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173, PLT-213 to PLT-221 (40) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-180, PLT-191 to PLT-202 (19) |
| [PLT](ledger/PLT.md) | the force balance's settings, coded and recorded, not applied (page [plate-forces.md](plate-forces.md); constants in [FRC](ledger/FRC.md)) | PLT-181 to PLT-190 (10) |

Rows by status (after the orogenic collapse, 2026-10-03): `verified` 17, `cited-unverified` 44, `from-memory` 4, `abstract-only` 19, `calibrated` 13, `game-choice` 75, `uncited` 42; `pending-rewrite` 151; `not-yet-coded` 8; `merged` (not counted) 33.

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
PLT-080, PLT-192, PLT-202; retired: PLT-069 to PLT-075, PLT-227). This is the first of two stages: it runs on
today's speed rules for plates, which a balance of forces is to replace (`docs/design/plate-force-balance.md`).

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
6. The belt resists (PLT-245): the convergence across the front loses, over the speed rules' own
   relaxation time, up to `collision_resistance_mm_yr` (200 mm/yr, calibrated) × (the belt's height ÷ the strength height)
   × (the front's length ÷ the slab front that pulls the arriving plate, at most 1). A plate pulled
   by a long slab elsewhere keeps converging and widens its belt; a plate with no slab elsewhere
   stops once its belt stands about 2 km high. Continental crust at a front does not pull (only
   oceanic slabs do, PLT-251), and a continent at a front is not dragged down to trench depth (PLT-252).
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

- The motion law is still the speed rules, not forces. The resistance is a speed (mm/yr), tuned
  (200) so that collisions stop before they consume their continents; they then stop sooner than
  India did. A plate with no slab is driven at 45 mm/yr for no physical reason, and the resistance
  has to overcome that.
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

Built 2026-10-03 (ledger PLT-254 to PLT-258; `plates/collapse.rs`). Ben's answer of that day
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

1. *The thickest crust* (PLT-258): a visible continental column thicker than `crust_max_km`
   (70 km) loses the excess from the bottom of the column (mafic root first) to the mantle. It
   gives no area.
2. *Collapse* (PLT-254): a visible continental sample that no orogeny has raised for
   `collapse_quiet_myr` (10 Myr, PLT-259: collapse follows the end of the thickening) and whose
   crust is thicker than `collapse_thickness_km` (55 km, PLT-256) releases
   (1 − e^(−δt/τ)) × (T − 55 km) of its thickness, τ = `collapse_myr` (15 Myr, PLT-255). The
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
- The threshold (55 km) and the quiet time were chosen among values the sources allow by the
  1000 Myr crust share (build log): with 50 km the crust ended at 0.52 of the planet, over the
  check's ceiling of 0.45. That is a calibration inside the sources' range, not a measured value.
- Thickness beyond `crust_max_km` is removed whether the crust is felsic or mafic. Felsic crust
  does not founder by itself (Wang et al. 2021); in the model the limit stands for continental
  subduction at the front as much as for foundering.
- The driver is the excess thickness, not the height: the model's thickness is the better record
  of the crust's weight because the surface stage's flexure and spreading change the height
  without changing the thickness. That decoupling is the model's, and the reason the thickest
  crust needs a limit at all.

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
- *Island arcs mature by the crust they built* (PLT-047). An oceanic sample becomes continental
  crust once its accumulated island-arc magma reaches `arc_magma_m_per_myr` × `arc_maturity_myr`,
  whatever its height. Hidden crust does not convert. `arc_maturity_myr` is 20 Myr in every preset,
  Cloos's lower bound. The model's arc magma (20 m per Myr at the arc axis at full convergence) is
  25-90 times smaller than Earth's arc growth (PLT-045) and is not a thickness, so the rule counts
  Cloos's time in it, at the model's full arc rate. An arc at a trench converging at half of v_0, or
  off the arc axis, takes longer. No value was tuned against the crust share.

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
