# How small terranes dock: design

Status: **built behind a switch, off** (2026-10-30, the docking lane, a sub-agent of the coordinator's
session) on the branch `small-terrane-docking`: `tectonics.terrane_docking_cover` (`shortening::advance`
classes the front, `belts::stack` strips it); ledger PLT-760, PLT-761; plates.md "How small terranes
dock". Its 8-run pool arm against `base34-s4` is the coordinator's to schedule (two other lanes hold the
simulation slots). For the coast-with-high-ground lane (Ben, 2026-10-30: "Freeboard, then orogen
(Recommended)", the advisor's third item; docs/research/notes/coast-high-ground-audit-2026-10-30.md).

## Why

The audit found the model's small terranes turned into collision belts: terranes under 50 samples are
0.54 of the collision states, they dock in a median 0 to 2 Myr, and about 0.95 of the crust that
crosses the front stays as thickness on the overriding plate (PLT-242's share, read for large
sutures). Its expectation: "on Earth such fragments accrete as area or subduct", about +0.01 of the
coast with high ground's +0.049 excess.

## What the cached runs say (base33, measurement only)

base33: master 2abbc567, pool key 31c7c70bd7c7, `earthlike_quick_16k-1000/cd665cef`, 24 runs, the
crust-budget maps on. Tools on this branch: `tools/review/terranes/dock_records.py` (the dynamics
record and the crust budget), `tools/review/terranes/dock_maps.py` (the crust-budget maps); outputs
`/data/ban/deeptime-runs/terranes/` on the VM. A sample is about 5,100 km² (Earth-normalized), 71 km
across.

**The collision states by terrane size** (after each run's first 100 Myr; counts are medians over the
runs, the rest pooled quartiles p25 / p50 / p75; the terrane's samples at the hand-off):

| | under 50 | 50 to 200 | 200 to 1,000 | 1,000 or more |
|---|---|---|---|---|
| hand-offs per 100 Myr | 29.2 | 13.3 | 8.6 | 3.9 |
| share of the crust crossing collision fronts (by area) | 0.086 | 0.071 | 0.147 | 0.314 |
| duration, Myr | 0 / 0 / 4 | 0 / 2 / 4 | 0 / 2 / 10 | 24 / 42 / 60 |
| shortening, km | 123 / 151 / 200 | 122 / 154 / 204 | 126 / 171 / 322 | 485 / 1,030 / 1,650 |
| front, km | 217 / 291 / 433 | 294 / 453 / 745 | 373 / 678 / 1,600 | 2,290 / 3,430 / 5,060 |
| crust crossed, km² | 30,200 / 42,500 / 62,400 | 39,900 / 61,900 / 106,000 | 49,100 / 91,300 / 325,000 | 649,000 / 1.89 M / 4.28 M |
| crust kept, km³ (crossed × the run's mean crossing thickness, 32.8 km, × its kept share, 0.946) | 0.94 / 1.31 / 1.93 M | 1.24 / 1.92 / 3.28 M | 1.52 / 2.82 / 9.98 M | 20 / 58 / 132 M |
| terrane at the hand-off, km² | 51,000 / 87,000 / 148,000 | 337,000 / 444,000 / 638,000 | 1.42 / 2.08 / 2.95 M | 11 / 22 / 41 M |
| share of what reached the front that crossed it (pooled) | 0.349 median (0.521) | 0.123 (0.301) | 0.047 (0.224) | 0.072 (0.091) |
| the front's uplift, summed over the steps, km | 3.5 / 5.1 / 7.6 | 4.5 / 6.7 / 9.8 | 6.0 / 9.2 / 26 | 41 / 79 / 108 |
| convergence at the end, mm/yr | 36.5 / 60.1 / 82.7 | 32.4 / 55.1 / 78.3 | 22.9 / 46.0 / 70.9 | 0.8 / 7.9 / 19.6 |
| how it ended | all on shortening (PLT-068) | all on shortening | 0.007 stalled, 0.035 transference | 0.24 welded, 0.76 transference |
| extent across the front, km (the shortening plus the rest's area over the front) | 333 / 455 / 642 (p10 254, p90 887) | 831 / 1,236 / 1,874 | 2,137 / 3,200 / 5,215 | 4,787 / 7,800 / 13,667 |
| share with that extent at most 220 / 270 km | 0.054 / 0.127 | 0.0007 / 0.0024 | 0 / 0 | 0 / 0 |

Partings (states with no hand-off): parted 15.0, reversed 4.3, closed 4.9 and open at the end 1.2 per
100 Myr. Under 50 samples the convergence does not slow (54.8 mm/yr at the onset, 60.1 at the end):
the terrane does not stop its plate; half dock in the step they begin, one step's convergence
(about 120 to 150 km) crossing the front.

**What the small bodies are** (at every map from -900 to -50 Myr: the bodies of one plate's continental
crust under 50 samples that are not their plate's largest; 127 a snapshot): own crust (the thickness
less a converted arc's ocean basement, PLT-331) p10 / p25 / p50 / p75 / p90 8.9 / 11.4 / 19.0 / 28.3 /
34.2 km; 0.40 of the samples under 15 km, 0.25 at 15 to 25, 0.26 at 25 to 35, 0.09 at 35 or more;
0.73 of them island arcs by their magma (own crust 10.1 / 14.4 / 23.4 km), the rest 24.1 / 28.6 /
32.8 km; their surface a median 2.2 km under the sea, 0.18 of them dry.

**Which collisions make the coast with high ground** (the final map, t0000: the plate model's ground
less the last step's sea on 100,000 Fibonacci points, M5 0.142 against the census grid's 0.1325; every
land point with a collision stamp matched to the collision state that stamped it by plate and time,
0.66 of them with more than one candidate, the one ending nearest the stamp taken; medians of 24):

| the nearest qualifying high ground | share of the coast |
|---|---|
| hand-offs under 50 / 50 to 200 / 200 to 1,000 / 1,000 or more samples | 0.0099 / 0.0037 / 0.0057 / 0.0004 |
| partings / collisions open at the end | 0.0035 / 0.0547 |
| Andean / other | 0.0284 / 0.0102 |

Without the high ground of the hand-offs under 200 samples M5 is 0.1288 (0.0055 lower, the mean over
runs 0.0082); without the open collisions' 0.095. The land the small hand-offs stamped stands low:
under 50 samples 0.046 of the land, p25 / p50 / p75 / p90 219 / 498 / 702 / 1,244 m over the sea, 0.056 of
it above 2 km; the open collisions' 0.050 of the land at 945 / 1,755 / 2,991 / 3,639 m, 0.46 above 2 km,
a median 215 km from the sea.

So the small terranes hold a small part of the excess, about 0.006 to 0.008 of M5 at most (the
advisor's +0.01, about), and the belts they leave stand at Earth's examples' heights (below). The
bulk is the open, large collisions near the sea: the orogen lane's.

## What Earth does (a Sonnet research sub-agent, 2026-10-30; its prints, copied)

Read in full or by pages as marked; the agent could not open the full texts of Cloos 1993, Tetreault
& Buiter 2012, Vogt & Gerya 2014 and others (publishers' blocks), and used no shadow library.

- **Cloos 1993** (GSA Bulletin 105:715-737; the abstract only): "Buoyancy analysis indicates that
  only bodies of continental and oceanic island are crust that are > ∼15 km thick make the
  lithosphere buoyant enough to jam a subduction zone" (the copy's typo); "Basaltic plateaus must
  have crustal thicknesses >∼30 km to typically cause collisional orogenesis during subduction";
  "Short subducting seamounts (<1-2 km tall) typically cause only temporary dents ... The direct
  tectonic effect resulting from the subduction of most bathymetric highs is only a temporary
  isostatic uplift of the fore-arc region of as much as several kilometers, followed by subsidence
  to original elevations."
- **Tetreault & Buiter 2014** (Solid Earth 5:1243-1275; pp. 1243-1257, 1259, 1262-1265 read): "most
  accreted terranes are thin crustal units sheared off of FATs and added onto the accretionary prism,
  with thicknesses on the order of hundreds of meters to a few kilometers. However, many island arcs,
  oceanic plateaus, and submarine ridges were sheared off in the subduction interface and underplated
  onto the overlying continent. Other times we find evidence of terrane–continent collision leaving
  behind accreted terranes 25–40 km thick." (p. 1243); "The tectonic addition of crustal material to
  continents at accretionary zones usually occurs by adding slivers of thrusted crustal units to the
  accretionary prism region (Coney et al., 1980; Snyder, 2002; Cawood et al., 2009), rather than
  collision and addition of the entire crustal thickness to the continent." (p. 1244); "Underplating
  of FAT crustal material onto the overriding plate during or after subduction, also called
  subcretion, is perhaps the most common type of terrane accretion." (p. 1245); "Intact accretion of
  FAT crusts by “docking” is often a collisional process (when subduction ceases) rather than an
  accretionary process (subduction continues after accretion) ... Many of the larger FATs such as
  oceanic plateaus and continental fragments are accreted by collision." (p. 1246); "in most cases
  only the upper 2–5 km of arc crust are accreted onto continents through thin-skinned thrusting and
  preserved. This most likely occurs when island arcs are on the subducting plate and arc material is
  underplated and accreted onto the overriding plate." (p. 1251); "Finally, the force of the
  subducting slab drives subduction and can most likely overcome the buoyancy of small crustal units
  (Molnar and Gray, 1979; Cloos, 1993)." and "Seamounts and submarine volcanics subduct easily if they
  are not incorporated into the accretionary prism." (p. 1265). Island arcs average "26 ± 6 km"
  (p. 1247), plateaus and ridges "21 ± 4 km" (p. 1253), continental fragments "∼ 24.8 ± 5.7 km"
  (p. 1259); arcs about "100 km in width" (p. 1246).
- **Tetreault & Buiter 2012** (JGR Solid Earth 117, B8; the abstract only): "(1) accretion of crustal
  units occurs when there is a weak detachment layer within the FAT, (2) the depth of detachment
  controls the amount of crust accreted onto the overriding plate, and (3) lithospheric buoyancy does
  not prevent FAT subduction during constant convergence."
- **Yan et al. 2021** (JGR Solid Earth 126(7), e2021JB021855; the abstract only): "steep subduction
  without subduction zone reorganization, favored by a small‐sized (≤150 km) or lowly depleted oceanic
  plateau"; "breakoff of subducting slab followed by flat‐slab subduction, which is conditional on a
  moderate‐sized (≥300 km) oceanic plateau"; "initiation of a new subduction zone behind the plateau
  (subduction zone jump)". **Yan et al. 2022** (GJI 231(2):1349-1362; the abstract only): "a
  relatively thin oceanic plateau (≤20 km thick) undergoes steep subduction, a moderate-scale plateau
  (25–30 km thick) favours flat-slab subduction and large-scale plateaus (≥35 km thick) are more
  susceptible to collide and accrete to the overriding upper plate."
- **Clift, Vannucchi & Phipps Morgan 2009** (Earth-Science Reviews 97:80-104; the library's copy,
  pp. 84-88, 95, 98-99): "ca. 90% of the mass of the Luzon Arc is transferred to the edge of
  continental Asia" (p. 85; the arc there is on the overriding plate); in arc–arc collision "the
  oceanic arc is plastered to the edge of the continental mass, effectively as a coherent block with
  no opportunity for major crustal loss by subduction" (p. 86); "Even in accretionary plate margins
  volcanic seamounts are generally subducted, with only thin slices of basalts (<1 km thick) being
  offscraped into the wedge of trench sediment" (p. 95).
- **Stern & Scholl 2010** (International Geology Review 52:1-31; the library's copy, p. 22): "Accretion
  accompanies collision, when a subduction zone is choked with buoyant crust and lithosphere (Cloos
  1993), destroying the original subduction zone and perhaps forcing a new one to form elsewhere
  (Stern 2004)"; terrane accretion episodes "typically last 8–10 million years (Clift et al. 2009)".
- **The upper plate over a docking body.** Cocos Ridge (Morell et al. 2012, JGR 117 B04409, pp. 1-4,
  6, 14, 16, 18): "rock uplift during the growth of the Cordillera de Talamanca is no greater than ∼2
  km" (abstract), "the greatest shortening (40 km since the Pliocene) ... directly inboard of the Cocos
  Ridge axis" (p. 6), the range "reaching elevations of 3800 m" (p. 4), the ridge subducting at
  85 mm/yr (p. 3). Yakutat (Enkelmann et al. 2008, Geology 36:915-918, pp. 915, 916, 918): "Ongoing
  oblique collision and flat-slab subduction of the Yakutat terrane has produced Earth’s highest
  coastal mountain range"; "Most of the Tertiary sedimentary cover to the Yakutat terrane has been
  scraped off basement rocks and accreted to the growing foldand-thrust belt" (p. 916). Taiwan's
  Coastal Range, the accreting Luzon arc itself, 0 to 1.68 km (Lai et al. 2022, PLT-241's note).

The sources disagree on arcs: Tetreault & Buiter find most arc crust lost (preserved, the agent's
ratios of their p. 1248 numbers: Talkeetna 0.51 to 0.60, Bonanza 0.63, Canyon Mountain 0.28; "in most
cases only the upper 2–5 km"), Clift et al. find the Luzon arc (on the overriding side) 88 to 90 %
transferred and arc–arc collision plastering arcs on whole (Talkeetna and Kohistan with "the full
crustal section").

## Reading the model against them

- **The model's small terranes are not small in the sources' sense.** At the quick planet's 71-km
  lattice, the hand-offs under 50 samples reach 333 / 455 / 642 km across the front (p25 / p50 / p75):
  the size Yan et al. 2021 find can reorganize the subduction zone ("moderate‐sized (≥300 km)", with a
  depleted root: breakoff and a flat slab, or a jump), not the "small‐sized (≤150 km)" one that
  subducts steeply. Only the p10 (254 km) lies between the two. They are island arcs a median 14.4 km thick on 7 km of ocean crust, about
  Cloos's 15 km (arcs turn continental at that, PLT-047), and continental fragments of 28.6 km
  (Tetreault & Buiter's mean 24.8).
- **What happens to them is mostly Earth's in kind.** A body of that size docks: the model hands the
  rest of the terrane to the overriding plate as area (0.65 of what reached the front at the median)
  after one step's crossing, and the crossing crust thickens the overriding plate's edge (Tetreault &
  Buiter's underplating, "perhaps the most common type"): the front's largest uplift is about 5 km
  (summed over the steps, mostly one), and at the run's end the land those hand-offs stamped stands a
  median 0.5 km over the sea (p90 1.2 km); Talamanca's uplift is at most 2 km over a subducting ridge,
  Taiwan's accreting arc 0 to 1.68 km, St Elias Earth's highest coast.
- **What is missing is the small branch.** A body its slab can pull down goes down with it,
  scraped of its cover ("the force of the subducting slab ... can most likely overcome the buoyancy of
  small crustal units"; seamounts with "only thin slices of basalts (<1 km thick) being offscraped";
  "only the upper 2–5 km of arc crust"). The model docks every continental body, however small, as a
  collision: stacked at 0.95 and transferred at 100 km of shortening (PLT-068, a calibrated value).

## The rule (PLT-760, PLT-761)

1. **Which bodies go down.** Each step, at each collision front whose contact still has a slab (the
   pair's slab track, as the jam reads it, PLT-516): the slab's reach against the terrane's buoyancy,
   S* = L / (1 + R T / T_c) (`shortening::dragged_length_km`), with L the slab's length up to
   `slab_depth_km` (PLT-184), T the incoming crust's mean thickness at the front (as FRC-030 reads
   it), R = 2.25 and T_c = 30 km (Cloos 1993's densities, FRC-028 and FRC-029). The terrane's extent
   across the front is the collision's shortening so far plus the visible area of the incoming plate's
   continental crust connected to the front's crust over the front's length. Within S*, the front
   subducts the terrane (`Collision::subducting`).
2. **What stays of them.** At a subducting front only the crossing crust's cover stays (PLT-761,
   `pools::COVER`: its own sediment, sedimentary rock and volcanic rock; the basement already goes,
   PLT-331), by PLT-242's share, as the cover's rock (its sediments metamorphosed, PLT-248), with the
   crust this collision stacked on it (PLT-400). The plutonic and metamorphic crust under it goes down
   with the slab (`Stacked::stripped_km2_m`, the global `plates/collision_stripped_km3`, the records'
   `stripped_km2_m`).
3. **No transfer.** A subducting terrane does not transfer at PLT-068's threshold: it goes down
   whole, and its collision state ends when no continental crust stands at the front (a parting). A
   stalled one still welds or transfers (PLT-246).
4. **The rest as before.** A terrane beyond S* jams its slab and collides: stacked at PLT-242's share,
   transferred at the threshold, or by transference or the reversal once its slab has gone.

No new constant: R and T_c are FRC-028 and FRC-029, L PLT-184's, the share PLT-242's, the cover the
model's own pools.

## What is the project's reading

- The balance in one dimension along the slab: the dense slab's pull over the length left,
  F_sp (L − S)/D, against FRC-030's buoyancy of the dragged length, R F_sp (T / T_c) S / D. Cloos's
  own analysis is of lithosphere columns by local isostasy, his result a thickness (about 15 km); the
  extent condition is the model's reading for a body of finite size along the slab, which the 2-D
  models of Yan et al. 2021 bracket (150 km subducting, 300 km reorganizing; S* is 215 to 330 km for
  30 to 15 km of crust on a full slab). In three dimensions the slab beside a body short along the
  trench helps pull it down (Tetreault & Buiter's "small crustal units"), which this reading leaves
  out: it is the conservative one.
- The cover's base as the detachment (Tetreault & Buiter 2014: "rheologically weak crustal layers or
  shear zones that were formed when the FATs were produced can be activated as detachments", p. 1243;
  Tetreault & Buiter 2012: "the depth of detachment controls the amount of crust accreted"). For the
  model's arcs it keeps a sixth of the arc's crust (its lavas, PLT-046) and its sediment: 2.5 km of a
  15-km arc on its basement, inside "the upper 2–5 km of arc crust" (a check, not a calibration).
- "Subduction continues" read as a live slab of the pair, the reading transference's jam uses.

## Considered and not taken

- **Every terrane under `collision_suture_frac` stripped** (the first build of this lane, replaced
  before any arm): it would strip the crossing of terranes up to 4.2 million km² (833 samples), which
  the sources dock by collision ("Continental fragments and composite terranes typically lead to
  collision", Tetreault & Buiter 2014, p. 1246); the size split there is a game-choice (PLT-078), not
  physics. By the crossed shares above it would have sent about 2 km³ a year more down (the project's
  estimate on supercontinent-1's crust budget: 0.30 of the 3.5 × 10⁸ km² crossing collision fronts in
  1,000 Myr, taken at 25 km of crust less 3 km of cover, at the share), against Stern & Scholl's
  3.2 km³ a year of all continental additions.
- **Moving the transfer to the jam's first step**: half the small hand-offs already end in the step
  they begin; the step's crossing (one step's convergence) would not change.
- **A kept share by rock density** (the mafic crust as eclogite going down): the model's arc crust is
  felsic plutonic under its lavas, the basement apart, so it would strip nothing new from arcs, and
  for continents it would contradict PLT-242's verified share.

## What the arm should show (the project's estimate before it runs)

Small: about 0.05 to 0.13 of the hand-offs under 50 samples and almost none above are within S* on
base33, so the coast with high ground should move by less than its small terranes' 0.006 to 0.008, and
the continental crust should lose a few hundredths of a km³ a year (the project's estimate: 1.6 to 3.7
bodies per 100 Myr, each taken at about 60,000 km² and 20 km of crust, less their cover). The arm is for the law's
correctness on the planet (the guard, no divergence) and its count (`collision_stripped_km3`, the
records' `stripped_km2_m`, partings by reason); Ben's rule puts physics on once it is built and
correct, whatever V1 does. On a finer lattice (32k presets) the smaller bodies are resolved and the law
reaches more of them.

## Measuring it

`pool.sh run small-terrane --key <this branch's key> --seeds 1,2,3,4 --set tectonics.terrane_docking_cover=true
+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps` against `base34-s4`, with the default,
`guard`, `collcoast` and `coastset` measures, and the two tools above over both arms.

Unit tests: `a_slab_drags_down_buoyant_crust_as_far_as_its_pull_outweighs_it` (shortening.rs: S*'s
values and the balance at it), `a_terrane_its_slab_drags_down_subducts_stripped_of_its_cover` (belts.rs:
an arc one row wide subducts stripped and does not transfer; five rows wide, the switch off or no slab,
a collision as before), `the_cover_is_the_layered_rock_on_the_basement` (pools.rs).
