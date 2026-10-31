# How small terranes dock: design

Status: **built; on by default since 2026-10-31** (the coordinator's call, decisions log 2026-10-31: its pool arm correct, the guard clean)
(2026-10-30 and 31, the docking lane, a sub-agent of the coordinator's session; "What the arm showed") on the branch `small-terrane-docking`: `tectonics.terrane_docking_cover` (`shortening::advance`
classes the front, `belts::stack` strips it); ledger PLT-770, PLT-771; plates.md "How small terranes
dock". Its 8-run pool arm runs on the collision orogen's physics (origin/collision-orogen merged in,
PLT-730, PLT-731 and PLT-740 set on) against the cached arm orog-cum ("Measuring it"). For the
coast-with-high-ground lane (Ben, 2026-10-30: "Freeboard, then orogen
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

### On base34 (the freeboard merge)

base34: master f80d75a8 (PR #49, PLT-660 and PLT-661 on), pool key 122e4803cded, `cd665cef`, 24 runs;
the arm's base, `base34-s4`, is its seeds 1 to 4. The same tools (`records-base34.json`,
`maps-base34.jsonl`; `dock_maps.py --summary` prints both bases side by side). base33's in brackets.

- **Records.** Hand-offs per 100 Myr: under 50 samples 20.4 (29.2), 50 to 200 11.8 (13.3), 200 to
  1,000 8.1 (8.6), 1,000 or more 5.3 (3.9). Under 50: duration 0 / 2 / 4 Myr, shortening 122 / 152 /
  202 km, front 229 / 311 / 479 km, crust crossed 31,400 / 44,600 / 69,000 km², kept 0.72 / 1.03 / 1.59 M
  km³ (the mean crossing thickness 24.4 km (32.8), kept share 0.949), the front's uplift 2.8 / 4.4 /
  6.9 km, the terrane at the hand-off 51,000 / 97,000 / 158,000 km², 0.35 of what reached the front
  crossing it (pooled 0.59). Extent across the front 336 / 462 / 660 km (p10 256, p90 915); the share
  within 220 / 270 / 300 / 350 / 400 km 0.052 / 0.128 / 0.179 / 0.277 / 0.377 (base33 0.054 / 0.127 /
  0.180 / 0.289 / 0.392); 50 to 200 samples 0.002 / 0.006 / 0.012 / 0.021 / 0.030.
- **Makeup.** 60 small bodies a snapshot (127), and thinner: own crust p10 / p25 / p50 / p75 / p90
  2.4 / 6.4 / 10.2 / 15.0 / 23.6 km, 0.75 of the samples under 15 km (0.40); island arcs 0.68 of them
  (own crust 7.1 / 10.0 / 13.7 km), the rest 4.0 / 10.5 / 21.4 km; their surface a median 3.3 km under
  the sea, 0.057 dry (0.18).
- **The coast with high ground** (the map's M5 0.089 median, 0.098 mean (0.142)). The nearest
  qualifying high ground: hand-offs under 50 / 50 to 200 / 200 to 1,000 / 1,000 or more samples
  0.0029 / 0.0041 / 0.0011 / 0.0017; partings 0.0017; open 0.0443; Andean 0.0114; other 0.0035. Without
  the high ground of the hand-offs under 200 samples M5 is 0.0870 (0.0021 lower, the mean 0.0044);
  without the open collisions' 0.056. The land the hand-offs under 50 samples stamped: 0.026 of the
  land, 117 / 331 / 635 / 838 m over the sea, 0.014 of it above 2 km; the open collisions' 0.044 of the
  land at 551 / 1,056 / 2,046 / 3,118 m, 0.26 above 2 km.

The freeboard merge left fewer, thinner small bodies, most of them under Cloos's 15 km, and their
share of the coast with high ground fell to about 0.002 to 0.004. The thinner crust lengthens the reach
(S* = 308 km under 17 km of crust, an arc of 10 km on its 7 km basement; 391 km under 10.5 km; on a
full slab), so more of them are within it than on base33, and they matter less to M5.

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

## The rule (PLT-770, PLT-771)

1. **Which bodies go down.** Each step, at each collision front whose contact still has a slab (the
   pair's slab track, as the jam reads it, PLT-516): the slab's reach against a body's buoyancy,
   S* = L / (1 + R T / T_c) (`shortening::dragged_length_km`), with L the slab's length up to
   `slab_depth_km` (PLT-184), T the body's mean crust thickness at the front (as FRC-030 reads it),
   R = 2.25 and T_c = 30 km (Cloos 1993's densities, FRC-028 and FRC-029). Each body at the front
   (the incoming plate's continental crust connected to some of the front's crust) is judged on its
   own: its extent across the front is all its area, the part already under the overriding plate
   (hidden) and the rest, over its share of the front's length (the front's length times its share
   of the front's crust samples). When every body is within its S*, the front subducts them
   (`Collision::subducting`); one body beyond its reach jams the slab for the whole front
   (`shortening::docking`).
2. **Judged as it arrives.** Each body judged going down is remembered by its samples and its share
   of the front's length when judged (`Collision::docking`, refreshed each step). A body now holding
   some of a judged body's samples is that body (or their union, where judged bodies have joined):
   while it holds no more samples than it did a step before (its crust only goes under, and a
   resampling or a sample turned oceanic only takes some away), it goes on down; when it holds more
   (crust has joined it), it alone is judged again, by all its samples over the share of the front
   it was judged with, so a few samples joining move its extent by a few samples' width. A body
   holding none of the judged samples has just arrived and is judged over its share of today's
   front. A resampling numbers the samples afresh: the bodies then share the judged bodies' length
   by their front crust and go on down while they hold no more samples than the judged did (or are
   judged again over those shares). A plate split keeps the numbering; samples gone to the other
   piece no longer count.
3. **What stays of them.** At a subducting front only the crossing crust's cover stays (PLT-771,
   `pools::COVER`: its own sediment, sedimentary rock and volcanic rock; the basement already goes,
   PLT-331), by PLT-242's share, as the cover's rock (its sediments metamorphosed, PLT-248), with the
   crust this collision stacked on it (PLT-400), pool by pool (`belts::kept_pools`: the cover's pools
   in full and the stacked share of each pool under it). All of it goes to the overriding plate: a
   terrane going down has no pro-wedge (PLT-730) or wedge (PLT-244) on it, so the kept crust takes
   the retro side, the belt and the overriding edge (PLT-700). The plutonic and metamorphic crust
   under it goes down with the slab (`Stacked::stripped_km2_m`, the global
   `plates/collision_stripped_km3`, the records' `stripped_km2_m`).
4. **No transfer, a live slab.** A subducting terrane does not transfer at PLT-068's threshold: it
   goes down whole, and its collision state ends when no continental crust stands at the front (a
   parting). A stalled one still welds or transfers (PLT-246). Its front feeds the slab as ocean floor
   does and does not block it (FRC-019's block time is for crust that jams the trench), so the slab
   neither stops lengthening nor breaks off under a body that takes longer than `slab_breakoff_myr`
   to go down.
5. **The rest as before.** A front with a body beyond its reach jams its slab and collides: stacked
   at PLT-242's share, transferred at the threshold, or by transference or the reversal once its slab
   has gone.

No new constant: R and T_c are FRC-028 and FRC-029, L PLT-184's, the share PLT-242's, the cover the
model's own pools.

## What is the project's reading

- The balance in one dimension along the slab: the dense slab's pull over the length left,
  F_sp (L − S)/D, against FRC-030's buoyancy of the dragged length, R F_sp (T / T_c) S / D. Cloos's
  own analysis is of lithosphere columns by local isostasy, his result a thickness (about 15 km); the
  extent condition is the model's reading for a body of finite size along the slab. It is not
  bracketed by Yan et al. 2021's 2-D models: S* takes in their small class (150 km or less subducting
  steeply) at any thickness, but it is 215 km under 30 km of crust and passes their 300 km under crust
  thinner than about 18 km (308 to 391 km for the base34 small bodies' 17 and 10.5 km), where they
  find a moderate plateau breaks the slab off and subducts flat or makes the subduction jump, which
  the model does not have; Yan et al. 2022 (abstract) find plateaus up to 20 km thick subduct
  steeply. Nor is the reading conservative either way: it leaves out the help of the slab beside a
  body short along the trench (Tetreault & Buiter's "small crustal units") and takes the slab's
  whole pull with no resistance of the mantle.
- The pull itself is not applied at the terrane's own front samples (the model's front law:
  continental front samples resist only, FRC-018 and FRC-030; the slab pulls through the contact's
  ocean-floor front and its hanging length). The terrane goes down at the convergence the plates'
  other forces keep, which does not slow under the small ones (54.8 mm/yr at the onset and 60.1 at
  the end on base33). Applying the pull through a docking front would be a new force term, left out.
- A body judged on its own share of the front, and one that jams holding the whole front, are the
  project's readings; so is judging a body as it arrives (the extent is the body's, not a number that
  changes as it crosses; re-judged when crust joins it).
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
bodies per 100 Myr, each taken at about 60,000 km² and 20 km of crust, less their cover). On base34,
the arm's base, about 0.18 to 0.38 of the hand-offs under 50 samples are within S* (300 to 400 km at
the crust's thinner thickness, on a full slab; a shorter slab reaches less), 3.7 to 7.7 bodies per 100
Myr: the coast with high ground should move by less than their 0.002 to 0.004, and the crust should
lose about 0.02 to 0.07 km³ a year (the project's estimate: each body at most its extent times its
front, 60,000 to 90,000 km², at 8 to 10 km of own crust less its cover, at the share). The arm is for the law's
correctness on the planet (the guard, no divergence) and its count (`collision_stripped_km3`, the
records' `stripped_km2_m`, partings by reason); Ben's rule puts physics on once it is built and
correct, whatever V1 does. On a finer lattice (32k presets) the smaller bodies are resolved and the law
reaches more of them.

## What the arm showed (2026-10-31)

Arm small-terrane (key dcf0d7480ef3, argkey 35ce6e09, 8 runs, seeds 1 to 4 of both starts; this branch
with origin/collision-orogen merged in, the docking law and both orogen laws set on) against orog-cum
(key f2d6412d801f, argkey d3261b0a, the same physics without the docking law). Medians, two-sided
Mann-Whitney p in brackets (`pool.sh table`; collcoast's p from its `--json` values).

- **Correct on the planet.** The guard clean: the worst 10,154 m from the sea against 9,945 (p 0.51), no
  step beyond 2 Earth reliefs, no plate cells 5 km over Airy, the highest plate elevation 4,700 m
  against 4,162 (p 0.13). All 8 runs ran.
- **What it did.** Hand-offs under 50 samples 12.3 per 100 Myr against 17.6 (p 0.014); 50 to 200 9.9
  against 9.8, 200 to 1,000 7.1 against 7.8 (p 0.50), 1,000 or more 4.6 and 4.6. Partings 29.2 against
  23.3 (p 0.23): of the parted (14.1 against 10.9), 5.8 per 100 Myr stripped crust, the terranes that
  went down whole. Crust stripped a median 35 million km³ a run (p25 28.5, p75 42.9 million), 0.035 km³
  a year (0.029 to 0.043), inside the estimate's 0.02 to 0.07: about 1 % of Stern & Scholl's 3.2 km³ a
  year of continental additions. Hand-offs that stripped at some step: 2.5 per 100 Myr under 50 samples,
  1.2 at 50 to 200, 0.56 at 200 to 1,000, 0.11 at 1,000 or more (the project's reading, not traced:
  a small body at the front went down first and the larger crust behind it on the same plate then
  jammed the slab). The hand-offs left under 50 samples are the larger ones: 424 / 568 / 749 km across
  the front against 362 / 492 / 674, 0.079 within 300 km against 0.127. The crust crossing a front is
  as thick as before (30.4 against 30.3 km), the kept share 0.950 against 0.954.
- **V1 and the coast** (none beyond the seeds' spread). Land 0.222 against 0.205 (p 0.19), largest
  landmass 0.083 against 0.074 (p 0.33), continental crust under the sea 0.567 against 0.605 (p 0.16),
  land above 2 km 0.018 against 0.013 (p 0.96), the coast with high ground 0.091 against 0.067
  (p 0.38), collisions per 100 Myr 4.45 against 4.60 (p 0.92), the continental crust's net change
  +0.083 against +0.094 (p 0.44), criteria met 6 and 6. collcoast: collision high coasts on landmasses
  under 0.02 of the land 0.359 against 0.668 (p 0.44), the collision regime 0.031 of the land against
  0.013 (p 0.105), collision high coasts whose belt is not elongated 0.170 against 0.075 (p 0.038, one
  column of twelve). coastset's coast with high ground 0.096 against 0.092 (p 0.72). On the maps
  (`dock_maps.py`): the coast whose nearest high ground the hand-offs under 50 samples stamped 0.0005
  against 0.0018, their land 0.012 of the land against 0.018; the map's M5 0.102 against 0.079, the rise
  in the open collisions' share (0.063 against 0.033), not the small terranes'.

So the law does what the sources say on the planet (small bodies go down with their slab, leaving their
cover; the larger ones collide as before), costs about 1 % of the crust's additions, and moves no V1
number beyond the seeds' spread. By Ben's rule (physics on once it is built and correct) it goes on
by default; the coordinator's call, with the orogen's flip.

## Measuring it

On the collision orogen's physics (the coordinator, 2026-10-31: docking interacts with where the
collision crust goes, and the two wedges and the retro plateau go on by default next): this branch
with origin/collision-orogen merged in, `pool.sh run small-terrane --seeds 1,2,3,4 --set
tectonics.terrane_docking_cover=true --set tectonics.collision_two_wedges=true --set
tectonics.collision_retro_plateau=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`
against the cached arm orog-cum (key f2d6412d801f, argkey d3261b0a, 8 runs, the same physics without
the docking law; or `base35-s4` once base35 exists), with the default, `guard`, `collcoast` and
`coastset` measures, and the two tools above over both arms.

Unit tests: `a_slab_drags_down_buoyant_crust_as_far_as_its_pull_outweighs_it` (shortening.rs: S*'s
values and the balance at it), `a_terrane_its_slab_drags_down_subducts_stripped_of_its_cover` (belts.rs:
an arc one row wide subducts stripped and does not transfer, its cover all on the overriding plate,
with `collision_restack_whole` on (the default) and off; five rows wide, the switch off or no slab, a
collision as before with its pro-wedge; five rows with the first already hidden under the overriding
plate still jam the slab), `the_reach_holds_a_terrane_just_inside_it_and_not_one_just_beyond`
(belts.rs: a slab 1 % past and short of the extent's S*, and a judgment kept or made again by the
samples judged), `each_body_at_a_front_is_judged_on_its_own_share_of_it` (belts.rs: a long arc and a
short deep fragment at one front, within the reach pooled, jam),
`a_terrane_going_down_keeps_its_cover_and_its_stacked_crust_pool_by_pool` (belts.rs: `kept_pools`),
`a_terrane_going_down_over_many_steps_leaves_its_cover_on_the_overriding_plate` (belts.rs: eight steps
with the plates moving, the overlaps resolved: no transfer, nothing on the incoming plate, the cover
kept on the overriding plate, no plutonic rock, the arc all under),
`a_terrane_going_down_keeps_the_crust_its_collision_stacked_on_it` (belts.rs: restacked crust, r > 0,
through `stack_front`), `a_front_going_down_feeds_its_slab_and_does_not_break_it_off` (belts.rs:
through `forces::step`), `the_cover_is_the_layered_rock_on_the_basement` (pools.rs).

## The review (PR #51, 2026-10-31) and what it changed

An Opus reviewer (Graphide was out of reviews) read PR #51 (the first build, renumbered PLT-770 and
PLT-771, on by default) and asked for fixes before landing. Each finding and its fix:

1. **The kept cover landed on the terrane going down** (confirmed). With the two wedges on (PLT-730),
   `fill_two_wedges` puts the kept rise on the pro flank first, which at a docking front is the
   terrane itself: a submarine arc against an emergent overriding edge took all of it, carried it
   across the front again a step later (its sediment metamorphosed, so not in the cover) and lost it,
   counted it in `collision_stripped_km3`, and thickened the front, shrinking S* on the way down. Fixed
   by the sources' placement, the cover "added onto the accretionary prism" (Tetreault & Buiter 2014,
   p. 1243) and "accreted to the growing foldand-thrust belt" (Enkelmann et al. 2008, p. 916): a
   stripped front has no pro flank and no wedge, so the overriding plate takes all of it (the retro
   side, the belt, the edge by PLT-700). Test: eight moving steps.
2. **The kept rock's mix** (confirmed): the kept thickness held the restacked crust while the mix kept
   only the cover's pools, repainting it; and the cover counted restacked volcanic rock twice. Fixed
   pool by pool (`kept_pools`): the cover's pools in full, the stacked share of each pool under it.
3. **Bodies pooled at one front** (confirmed): fixed, each body judged on its own share of the front;
   one that jams holds the whole front. Test: two bodies.
4. **The slab broke off under a docking body** (plausible): a body holding half the front blocked the
   slab (FRC-019), so after 10 Myr the slab was gone and the rest transferred at once (a 300 km body at
   30 mm/yr needs 10 Myr). Fixed: a front going down feeds its slab as ocean floor does.
5. **Docs**: the bracketing and "conservative" wording withdrawn (S* passes Yan et al.'s 300 km under
   crust thinner than about 18 km), the module docs, the switch's doc, the ledger's count line, the
   build log and the bibliography fixed.
6. **Tests**: the default restack path, the reach's boundary, many moving steps and two bodies added.

Writing the moving test found a seventh: judged each step by the shortening so far plus the visible
rest, a terrane going down turned into a jam halfway (at 153 km of shortening a one-row arc read 343
km against its 329: a sample stays visible until the overriding plate covers it, so the shortening
runs ahead of the hiding by up to a spacing), and counting its hidden part instead turned it into a
jam at the end (the front shrinks to its last samples while the hidden part lies along the old front).
So a body is judged as it arrives, by all its samples, and keeps the judgment while it goes down.
It likely lies behind the first arm's hand-offs that stripped at some step and then transferred (2.5
per 100 Myr under 50 samples, 1.2 at 50 to 200; the project's reading, not traced).

## The second arm (after the review, 2026-10-31)

Arm small-terrane-2: branch `small-terrane-on` at 705f4500, key a7bb6d240ebe, argkey 35ce6e09 (the
first arm's settings: the docking law and both orogen laws set on, which master now has by default),
8 runs, against orog-cum (medians, two-sided Mann-Whitney p in brackets; the first arm's medians
after the slash). The base is not clean: master's source key since orog-cum (fe4c1b633535 against
f2d6412d801f) also holds the continental heat flow of crust made continental late in a step (PLT-469,
b1b7a79e) and PR #50's review; `base35-s4` is the clean base once base35 exists.

- **The guard.** No step beyond 2 Earth reliefs, no plate cells 5 km over Airy, the highest plate
  elevation 4,194 m against 4,162 (p 0.80). The farthest any cell stood from the sea (a trench's
  floor) 10,243 m against 9,945 (p 0.001; the first arm 10,154, p 0.51): about 300 m deeper, half the
  guard's 19.8 km; whether the slabs fed under docking fronts or master's heat flow made it is not
  told apart by this base.
- **What the law did.** Hand-offs under 50 samples 13.3 per 100 Myr against 17.6 (p 0.010; / 12.3),
  50 to 200 10.6, 200 to 1,000 7.7, 1,000 or more 4.7 (all as orog-cum's); partings 30.4 against 23.3
  (p 0.005), the parted share of them 0.538 against 0.484 (p 0.003), 5.3 per 100 Myr of the parted
  stripped (/ 5.8). Crust stripped 0.033 km³ a year (p25 to p75 0.031 to 0.038; / 0.035). Hand-offs
  that stripped at some step and then transferred: 1.33 per 100 Myr under 50 samples (/ 2.5), 0.61 at
  50 to 200 (/ 1.17), 0.50 at 200 to 1,000 (/ 0.56): the judgment kept on the way down halved the
  small ones; the rest are fronts crust joined (judged again) or stalls.
- **V1** (none beyond the seeds' spread): land 0.210 against 0.205 (p 0.80), largest landmass 0.068
  against 0.074 (p 0.72), continental crust under the sea 0.582 against 0.605 (p 1.0), land above
  2 km 0.011 against 0.013 (p 0.80), the coast with high ground 0.092 against 0.067 (p 0.33),
  collisions per 100 Myr 4.75 against 4.60 (p 0.96), criteria met 5 and 5.50 against 6 and 5.75
  (p 0.40). coastset's coast with high ground 0.096 against 0.092; the collision belt's share of
  the high coasts 0.742 against 0.535 (/ 0.606), the arc's 0.131 against 0.240.
- **Small landmasses.** Collision high coasts on landmasses under 0.02 of the land 0.346 against 0.668
  (p 0.038; / 0.359, p 0.44): the island belts the audit read as small terranes turned collision
  belts are half as many.
- **The maps** (`dock_maps.py`): the coast whose nearest high ground a hand-off under 50 samples
  stamped 0.0001 against 0.0018 (p 0.61); the map's M5 0.114 against 0.079 (p 0.51), its rise the open
  collisions' share (0.068 against 0.033, p 0.083), not the small terranes'.

So the corrected law does on the planet what the first build did, more cleanly: fewer small
hand-offs and more partings, both now significant, half the small landmasses' collision high coasts,
the crust stripped at about 1 % of the continents' additions, and no V1 number beyond the spread. The
deeper trench floor is the one significant change to read again against base35-s4.

## The re-review (ab427b39, 2026-10-31) and its fixes

The re-review confirmed fixes 1 to 6 and found no bug in the deeper trench floor (a slab grows only
with convergence; the 243 m is the surface stage's, from the fed slab, the cover on the overriding
edge or master's heat flow: `base35-s4` will tell). Its two findings, fixed on `small-terrane-on`:

1. **Bodies recognised only by their count** (`docking_samples`). Too lax: once hidden crust is
   consumed (a resampling, a sample turned oceanic), new crust could join unjudged up to the
   consumed count. Too harsh: on growth every body was judged again by all its samples over today's
   front, the reading found wrong halfway down, so one sample joining near the end made a jam that
   transferred at once (likely the second arm's remaining 1.33 per 100 Myr). Fixed by remembering
   each judged body by its samples and its judged share of the front (`Collision::docking`, the
   rule's item 2): any growth is seen (the samples are refreshed each step), and only the body that
   gained crust is judged again, over the share it was judged with.
2. **A forces test**: `a_front_going_down_feeds_its_slab_and_does_not_break_it_off` (the arc along
   the whole converging boundary through `forces::step` for 12 Myr: going down, the slab lengthens
   to `slab_depth_km` and stays; with the switch off it jams, lengthens only by the ocean floor at
   the arc's ends, and breaks off). And `a_terrane_going_down_keeps_the_crust_its_collision_stacked_on_it`
   (a stripping with restacked crust, r = 5 / 40.1, through `stack_front`), the reach test's kept and
   renewed judgments (`docking` with judged bodies, one sample short, a doubled share, a stranger,
   and a resampling).

Of its lower findings one is fixed: the stripped rise's rock mix weighed the cover at 1 where the
rise holds it at s (1 − r) + r with `collision_restack_whole` (at most 5 % too much cover rock); it
now weighs it so. Two are limits, kept:

- **The cover on ocean floor.** With no pro flank, a front whose overriding plate has no continental
  crust within the belt's reach puts the kept cover on the overriding edge by PLT-700's rule, on
  ocean floor (a few km per event), as PLT-700 already does for any collision against an oceanic
  overriding plate. The sources' accretionary prism stands at that edge whatever its crust; PLT-290's
  concern (ocean floor taking a continent's stack and coming back as one lump at a resampling) is
  about tens of km, not a cover's few.
- **A jammed body after a resampling.** A front that jams keeps no bodies; after a resampling has
  consumed the part of a jammed body already under, it is judged afresh by what is left and may go
  down for its remainder. A jam that holds half its front breaks its slab off in 10 Myr (FRC-019),
  and without a slab nothing goes down, so this needs a jam on a front mostly ocean floor that
  outlasts a resampling.
