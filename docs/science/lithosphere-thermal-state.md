# The lithosphere's thermal state

Status of this page: `stage 2 (constants checked against sources on 2026-10-23, where a source
could be read; the rest marked from-memory)`. Written by the gap-4 agent, 2026-10-23, before the
code (the brief's "design first"); the measured sections were added after the pool runs.

Ledger ids on this page: `PLT-460` to `PLT-470`, `PLT-475` to `PLT-478`, `ISO-110` to `ISO-112`
(see [claims.md](claims.md) and [ledger/PLT.md](ledger/PLT.md), [ledger/ISO.md](ledger/ISO.md)).
Every law here sits behind a `--set` switch that is off by default (`tectonics.thermal_state`,
`tectonics.thermal_isostasy_m`, `tectonics.thermal_strength`, `tectonics.thermal_backstop`,
`tectonics.heat_flow_refresh`, `tectonics.transition_beta` (stage 2), `tectonics.root_removal`
(stage 3); `tectonics.thermal_keel_km` sets the start's keel). Stages 2 and 3 were added by the
gap-4 stages 2-3 agent on 2026-10-23, after a diagnosis of where the high coasts are.

## What you see in the world

The lithosphere is the cold, strong outer shell of the planet: the crust and the mantle under it
down to the depth where the rock is hot enough (about 1,333 °C) to flow. Under a hot region it is
thin, 50 to 60 km; under an old craton it is 200 to 250 km. A hot lithosphere stands higher at the
same crust, because warm rock is lighter, and it is weak; a cold one stands lower and is stiff.

With the switches on, ground behind an active subduction zone, over a hotspot and in a continental
rift warms over about ten million years and rises; it cools and sinks slowly once the heating stops,
over hundreds of millions of years. The old continents the planet starts with carry a depleted,
buoyant keel that back-arc heating cannot remove, so they stay cold and stiff when an arc reaches
them. The heat flow the game reads (geothermal fields, metamorphism, deposits) comes from the same
state: about 41 mW/m² over an old craton, about 53 over a cooled younger continent, about 82 behind
an arc.

## The model in plain words

### What each sample carries and what resets it

Each plate sample (`plates/state.rs`, `Sample`) carries two numbers.

- `lithosphere_km`, L: the depth of the thermal lithosphere's base, km. McKenzie's (1978) plate
  model puts it at the 1,333 °C isotherm; the model only keeps its depth.
- `keel_km`, K: the thickness of depleted, chemically buoyant mantle under the crust, km: Jordan's
  (1978) tectosphere, "stabilised against convective disruption by the depletion of the continental
  upper mantle". Only the continents the planet starts with have one.

The thermal step (`plates/thermal.rs`) runs once per plate step, after the hotspots and before the
settling, when this step's flags are all set. For each visible continental sample:

1. **Heating.** A sample flagged as an arc (`ARC`), behind a trench (`SUBDUCTION_ZONE`, the
   overriding plate within `subduction_distance_km` of a trench: a back-arc), over a hotspot
   (`HOTSPOT`) or in a continental rift (`DIVERGENT`) is heated: L relaxes towards a target L* over
   τ_h = 10 Myr. The back-arc's and the plume's heating is convective (sub-lithospheric flow erodes
   the lithosphere from below), and a keel resists it: L* = max(L_hot, K). A rift stretches the
   lithosphere and its keel together: L* = L_hot, and K follows L down (K ≤ L always).
2. **Cooling.** Any other continental sample cools: L relaxes towards L_cold over McKenzie's thermal
   time for that thickness, τ_c = a²/(π²κ) with a = L_cold, which is 62.8 Myr × (225/125)² =
   203 Myr.
3. **Ocean floor** carries the plate model's own lithosphere, L = min(2.32 √(κ t), 125 km) at age
   t, set each step; it matters only when the floor becomes continental (an arc matures, crust spreads
   onto it), and the new continental sample starts from it with no keel.
4. **The start.** The continents the planet starts with begin at L = L_ref (the frame below) with a
   keel K = min(K_0, L) (K_0 = `thermal_keel_km`, 200 km; 0 gives no keels). A continental sample that appears later with no state (none is set) takes
   L_ref and no keel.

A resampling copies both numbers from the nearest corner of the sample's kind (`ridges.rs`
`interpolate` keeps the major corner's values).

### Where the state goes

- **Heat flow** (`tectonics.thermal_state`): continental heat flow is a crust part and a mantle part,
  q = q_c + B / L, with B / L the heat conducted up through the lithosphere's base, plus the old
  proxy's magmatic terms for an arc and a hotspot (+40 each, PLT-137). The proxy's +30 for a rift is
  dropped: a rift's thinning is now in L. The ocean keeps its age law.
- **The heat-flow fault** (`tectonics.heat_flow_refresh`, and on with the state): after the step's
  resampling, the rate guard can spread crust onto old sea floor and turn it continental
  (`collapse::spread`, PLT-331). That happens after the step set every sample's heat flow, so the
  converted sample kept the sea floor's value (33.7 mW/m² on 229 Myr floor) into the bake's
  continental heat-flow layer. A final pass after the resampling gives every continental sample at
  least its law's conductive value.
- **Thermal isostasy** (`tectonics.thermal_isostasy_m` > 0): the Airy height a continental column
  settles to (`settle.rs`) and the height the thin sheet takes its potential energy from
  (`collapse.rs`, `Sheet`) gain z_th = E (L_ref − L) / (L_cold − L_hot). E is the height between a
  hot back-arc and a cold craton at equal crust (1,600 m, Hyndman & Currie 2011). L_ref is the
  lithosphere at which the model's Airy line already stands (z_th = 0), so it says what the current
  calibration means.
- **Strength** (`tectonics.thermal_strength`): the state gives a thermal age, the time the cooling
  law takes from L_hot to L, t_eq = τ_c ln((L_cold − L_hot) / (L_cold − L)), infinite at L_cold.
  It replaces the age of the orogeny in the thin sheet's viscosity (PLT-415, η = η_0 e^(t / 62.8))
  and in the flexure's continental Te law (ISO-050, ISO-051), which reads it from the L1 field
  `plates/thermal_age_myr`. A hot orogen spreads as a fresh one does today; a cold, keeled craton
  reached by an arc's orogeny keeps its strength, where today the orogeny's new date makes it as weak
  as a new arc. It is not a strength envelope: it bounds no stress, so a stiffer craton holds more
  load. The bound is the plate's yield (ISO-120, `surface.flexure.yield_anomaly_mgal`), a separate
  switch ("Measured" has the two together).

- **The backstop** (`tectonics.thermal_backstop`): an arc's cordillera grows by crust shortened
  into it from the same plate's crust behind the range (PLT-048), which loses the same share of
  every pool. With the backstop each source's share is weighted by its weakness, the sheet's
  η_0 / η at its thermal age, w = e^(−t_eq / τ_s): a hot back-arc gives its crust, a cold craton
  hardly any, and the range gets what the sources give (volume kept). Hyndman, Currie & Mazzotti
  (2005): back-arc lithospheres are "weaker than cratons by more than a factor of 10", so the
  back-arc, not the craton, takes the shortening.

This reuses the release pace's state rather than adding a clock: McKenzie's τ is the same law, and
the flexure's held Te (ISO-099) keeps working on top of the new target.

### The two faults folded in

**Heat flow with the release pace on.** At release commit 1bdacb76, with
`surface.flexure.te_weakening_myr=62.8`, `bake::tests::a_planet_run_bakes_every_chunk_time_layer`
(seed 5, 100 Myr) baked a continental cell at (159, 11) with 33.7 mW/m², under the old proxy's
55 floor: that is the heat flow of 229 Myr sea floor. On master bf8477b2 that planet no longer
shows it, so a scan of the same test planet with the release pace on (seeds 1 to 12 at 60 and at
100 Myr, no final pass) looked for it again: seed 1 at 60 Myr bakes four continental cells under
the floor, the lowest 33.5 at (150, 60); the 23 other planets are clean. The cause is the order of
two steps. The plate step sets every sample's heat flow, then resamples, and only then does the rate
guard spread crust onto old floor next to it (`collapse::spread`, PLT-331), making those samples
continental with the sea floor's value still on them; the release pace changes which steps spread,
so it shows the fault, it does not make it. The fix is the final pass (PLT-469,
`tectonics.heat_flow_refresh`, and always on with the state): after the resampling and the spread,
every continental sample gets at least its law's conductive value (the proxy's
55 + 35 e^(−t/60), at least 55 mW/m²; with the state, q_c + B / L). The same planet with the pass bakes no cell under 55 (the scan's
rerun), and `the_release_pace_bakes_continental_heat_flow_with_the_final_pass` bakes it with the
release pace on and holds every continental cell to at least 54.9;
`the_thermal_state_bakes_its_own_heat_flow` bakes it with the state on and holds them to the
state's floor, an old craton's 41.

**The savanna pit's craton** (build log 2026-10-23, savanna pit part B: Ben's seed, L1 cell
(396, 231)). An Andean arc's subduction zone reached a craton at step 498: its regime went from
craton to back-arc, the arc's orogeny restarted its continental clock, the flexure's Te went from
80 to 5 km in one step and let go of its held load, and its crust went from 38.6 to 27.6 km in
40 Myr. What the state does to each part:

- *Te and the sheet's strength* (`thermal_strength`). A keeled craton behind a trench heats only
  to its keel: after 40 Myr its lithosphere is 200.5 km deep, a thermal age of 388 Myr. The flexure
  reads Te = 80 (1 − e^(−388/300)) = 58 km in place of 5, and the thin sheet's viscosity stays
  about 480 times a fresh orogen's. The step that dug the pit does not happen (test
  `a_keeled_craton_behind_an_arc_stays_cold_and_stiff`). Without a keel (`thermal_keel_km=0`) the
  same craton warms to a back-arc's 60 km over a few tens of Myr, so its Te falls at that pace, not
  in one step.
- *The crust.* The 11 km is not thermal. Continental crust 440 to 800 km behind an arc is the
  cordillera's source (PLT-048): it gives the range its share of the shortening, at most 1.7 mm a
  year over the band's 360 km, 19 % of its crust in 40 Myr, about 7 of the 11 km (the trace did not
  record the cell's distance to the front or the rest of its losses). The state reaches it through
  the backstop (`thermal_backstop`, PLT-470): a source gives in proportion to η_0 / η at its thermal
  age, so the keeled craton gives about 1/480 of a back-arc's share, about 14 m in the 40 Myr.

### The order of the stages

1. **Stage 1 (this task):** the state, heat flow from it with the fault's final pass, thermal
   isostasy and strength, each its own switch.
2. **Stage 2, rifted margins** (built 2026-10-23, `tectonics.transition_beta`, below): the
   transition's linear taper (PLT-261 to PLT-264) replaced by a stretching factor β on both sides of
   a new rift: crust T_n / β, lithosphere L / β, then McKenzie's thermal subsidence from the state
   (already there: a stretched margin's L recovers over τ_c and z_th falls by the same law). Coastal
   plains and shelves follow from the subsidence.
3. **Stage 3, root removal** (built 2026-10-23, `tectonics.root_removal`, below): the founder half
   of the 70 km cap (PLT-258) replaced by a Rayleigh-Taylor removal of the thickened mantle
   lithosphere (Houseman & Molnar 1997: half to three quarters of it in about 10 Myr), which heats L
   under a collision belt that has no keel.

### Where the high coasts are (the diagnosis before stages 2 and 3)

The brief asked which plate setting the coast with high ground sits in before building either
stage, since stage 2 can only reach rifted margins and stage 3 only collision belts. The `coastset`
measure (`tools/review/pool/measure_coastset.py`, registry `tools/review/pool/measures_gap4.json`,
run by `pool.sh measure` over the base's cached runs, no new simulation) recomputes v1census's coast
and high ground (`continents_census.coast_metrics`: coast cells within 1.6 cell spacings of the
sea, high ground land above 2 km of the same landmass within 200 km) and gives each coast cell a
setting from the plate regimes within 150 km of it, the first that holds of: collision (a collision
belt, regime 5), arc (trench, arc or back-arc), rifted (a rift, a ridge or a passive margin),
hotspot, other (only craton and ocean basin: an inland sea, a transform). The high ground's origin
is the orogeny of its nearest cell above 2 km.

Base, 24 runs (medians; the two starts are alike):

| setting | share of the high coasts | share of the coast | high-coast rate in the setting | high coasts facing flooded continental crust |
|---|---|---|---|---|
| collision belt | 0.690 | 0.360 | 0.642 | 0.265 |
| arc | 0.190 | 0.279 | 0.246 | 0.438 |
| rifted margin | 0.085 | 0.285 | 0.094 | 0.501 |
| hotspot | 0.000 | 0.000 | | |
| other | 0.007 | 0.063 | 0.040 | about 1 |
| all | 1 (coast_hi 0.333) | 1 | 0.333 | 0.323 (all coast: 0.552) |

- **The high coasts are collision belts.** Seven in ten sit within 150 km of a collision belt, where
  two in three coast cells have high ground. Their high ground is Himalayan (0.837 of it), active
  (its orogeny's median age 0 Myr), 56 km from the coast, on 56 km of crust. Without them coast_hi
  would be 0.094, inside its band (≤ 0.15).
- **Rifted margins are not the problem.** They hold 0.285 of the coast but 0.085 of the high
  coasts, a rate of 0.094 that already meets the band. Removing every rifted high coast would take
  coast_hi from 0.333 only to 0.306. Their high ground is mostly no orogeny's (0.621; 0.265 Andean,
  0.090 Himalayan), 56 Myr old at the median, 88 km inland, on 46.7 km of crust.
- **Arcs** are the second share (0.190), Andean high ground 106 km inland.
- Over the whole planet, high ground by origin: Himalayan 0.612, Andean 0.228, none 0.140.

So stage 2 can move coast_hi by at most 0.03; the lever for that criterion is the collision belts,
which stage 3 (with the strength) reaches. A collision belt at an ocean coast is itself the oddity:
on Earth a continent-continent collision closes the ocean between the two, and its belt lies inland
(the Himalaya, the Alps, the Urals), with the sea at its ends only.

### Stage 2 as built (`tectonics.transition_beta`, PLT-475, PLT-476)

- **The profile.** The transition keeps its zone (W = 200 km on the model sphere, PLT-262), its
  catchments, its pace (τ = 20 Myr, PLT-263) and its height line (PLT-264), and its target becomes
  a stretching factor's: T*(d) = T_o + (T_n − T_o) S(1/2 + d / W), S(s) = 3s² − 2s³. β = T_n / T*
  stays under about 1.15 over the inner quarter of the zone (Manatschal & Chenin 2023's proximal
  domain, crust "30+-5 km", "only a little thinned"), rises fastest at the boundary (the necking
  domain, "from 30+-5 km to ca. 10 km") and is 3 or more over the outer quarter (the hyperextended
  domain, under 10 km) up to T_n / T_o = 4.7 at the toe. On Earth the necking domain averages 55 km
  and the hyperextended 50 km (Chenin et al. 2017, as Manatschal & Chenin cite it); the model's
  zone is 200 km, so these lie inside it in that order. S is symmetric about the boundary like the
  linear ramp, so the step's crust is held. Each conjugate margin is its own plate's boundary, so
  both sides of a new ocean get the profile. On the quick 16k planet a sample is 71 km across on the
  model sphere, so one or two samples a side fall in the zone: the boundary samples at d = ±35 km
  hold 26.6 km (land side) and 13.4 km (ocean side) where the ramp gave 24.6 and 15.4.
- **The lithosphere.** A continental source the transition thins to `keep` of its crystalline crust
  in a step (β = 1 / keep for that step) has its lithosphere and keel thinned by the same factor,
  L / β and K / β: McKenzie's (1978) uniform stretching, in which "a is the thickness of the
  lithosphere and t_c the initial thickness of the continental crust" both stretch by β. Lang, ten
  Brink & Makovsky (2026) find the mantle stretched more than the crust at the Baltimore Canyon
  Trough (a mantle to crust "beta ratio = 2.8"); uniform stretching is the simpler choice and is
  what McKenzie's subsidence table assumes.
- **The subsidence.** Nothing new: the state cools the thinned lithosphere back towards L_cold over
  τ_c (PLT-465) and, with `thermal_isostasy_m` on, z_th falls by the same law. McKenzie's Table 2
  gives the total thermal subsidence of a stretched margin: 1,300 m at β = 1.5, 1,950 m at 2,
  2,930 m at 4, at most 3,890 m; Montadert et al. (1979) measured about 4,000 m at the Bay of
  Biscay's continent-ocean boundary falling to about 1 km at the shelf edge over 200 km.
- **Without the state** there is no lithosphere to thin and only the crust's profile changes. With
  the state but no thermal isostasy the stretched margin's heat flow and strength change (a hot,
  weak margin) and its height is the crust's alone.

### Stage 3 as built (`tectonics.root_removal`, PLT-477, PLT-478)

- **Which columns.** A continental column of a collision orogeny (Himalayan) whose crust is thicker
  than `collapse_thickness_km` (50 km, PLT-256): crust thickened that far stands for a lithosphere
  thickened with it. No source read gives a threshold (England & Houseman 1989: the instability
  "would only occur after there had already been substantial thickening of the lithosphere"; Kay &
  Kay 1993: "a critical amount of shortening is required"), so the collapse's own threshold is
  reused. An Andean range's lithosphere is the arc's (stage 1 already heats it).
- **What happens.** Its lithosphere relaxes towards max(L_hot, K, its crust) with an e-folding time
  of 10 Myr: the thickened mantle lithosphere drops off to the Moho, 63 % of the way in the first
  10 Myr (Houseman & Molnar 1997: "between 50 (for rheological parameters representative of dry
  olivine) and 75 per cent (for wet olivine) of the lithospheric mantle would be involved in an
  instability whose duration is of the order of 10 Myr"; Conrad 2000: "The hottest 60 per cent of
  the mantle lithosphere could be removed"). A keel holds it: a belt on a keeled craton stops at the
  keel, as stage 1's back-arc heating does (Jordan 1978's tectosphere). A rift's or an arc's heating
  comes first.
- **The founder.** The 70 km cap's founder (PLT-258), which sank all of a column's excess that found
  no room in one step, now sinks 1 − e^(−δt / 10 Myr) of it a step, the same drop-off's pace. It
  barely fires in the base (1.15 × 10⁷ km³ foundered over 1,000 Myr on supercontinent seed 2,
  against 8.55 × 10⁹ km³ the collapse moved), so this half is bookkeeping.
- **The eclogite stays.** The eclogite pool (PLT-279, PLT-280) is not removed: a root only about
  30 % eclogitized is "gravitationally stable" (Shi D. et al. 2020), and removing the eclogite would
  raise the surface by only about 0.045 of its thickness at the model's densities.
- **Without the state** only the founder's pace changes. With the state, the heated belt's heat
  flow rises; with `thermal_strength` its thermal age falls to a fresh orogen's, so it spreads and
  its flexure's Te drops; with `thermal_isostasy_m` it rises by up to E = 1,600 m (Molnar, England &
  Martinod 1993: "approximately 1000 m to 2500 m in a few million years" over Tibet).

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| PLT-460 | the state and its update | L ← L* + (L − L*) e^(−δt/τ) per step; heated: τ_h, L* = max(L_hot, K) (convective) or L_hot (rift, then K ← min(K, L)); else τ_c, L* = L_cold | km | McKenzie 1978 (the plate), Jordan 1978 (the keel); the relaxation form is the agent's | `game-choice` (the form; its constants are the rows below) |
| PLT-461 | L_hot, a hot back-arc's lithosphere | 60 | km | Hyndman, Currie & Mazzotti 2005 ("50-60 km"), Currie & Hyndman 2006 ("1200°C at 60 km") | `verified` |
| PLT-462 | L_cold, an old craton's lithosphere | 225 | km | Sleep 2003 (full text, abstract: "relatively uniform, 225 km"); Jaupart et al. 1998 200-250 km as Rolandone et al. 2002 cite it | `verified` |
| PLT-463 | K_0, the start's keel | 200 | km | Jaupart & Mareschal 1999 as Rolandone et al. 2002 cite it ("unlikely to be smaller than 200 km") | `abstract-only` |
| PLT-464 | τ_h, the heating time | 10 | Myr | Houseman & Molnar 1997 (p. 131: "may require less than 10 Myr if the viscosity is less than about 10^21 Pa s"); Currie, Huismans & Beaumont 2008 (5-10 Ma) | `verified` |
| PLT-465 | τ_c, the cooling time | 62.8 × (L_cold / 125)² = 203.5 | Myr | McKenzie 1978 (eq. 6, τ = a²/(π²κ); Table 1, a = 125 km, τ = 62.8 My) | `verified` (the scaling with a² is the equation's) |
| PLT-466 | the ocean floor's lithosphere | min(2.32 √(κ t), 125), κ = 0.80 × 10⁻⁶ m²/s from McKenzie's τ and a | km | McKenzie 1978 (a, τ); the 2.32 (the 90 % isotherm of a half-space) is from memory | `from-memory` |
| PLT-467 | continental heat flow | q = q_c + B / L, q_c = 26 mW/m², B = 3,375 mW/m² × km | mW/m² | Archean mean 41 (Nyblade & Pollack 1993 as two open papers cite it); mantle heat flow 11 to 17 (Rolandone et al. 2002; Sleep 2003 citing Jaupart & Mareschal 1999), 15 taken at L_cold | `abstract-only` |
| PLT-468 | the magmatic terms | +40 arc, +40 hotspot (PLT-137's); the rift's +30 dropped | mW/m² | PLT-137 (uncited) | `uncited` |
| PLT-469 | the final heat-flow pass | after the resampling, continental q ← max(q, the law's conductive value) | mW/m² | the bake test's fault (2026-10-23) | `game-choice` (a bookkeeping fix) |
| PLT-470 | the backstop | a source's share of the cordillera's strain w = e^(−t_eq / τ_s), τ_s = 62.8 (PLT-415) | none | Hyndman, Currie & Mazzotti 2005 (back-arcs weaker than cratons "by more than a factor of 10"); the weight is PLT-415's law | `game-choice` |
| PLT-475 | stage 2's margin profile | T* = T_o + (T_n − T_o) S(1/2 + d / W), S(s) = 3s² − 2s³ (PLT-261's S(s) = s when off) | km | Manatschal & Chenin 2023 (the proximal, necking and hyperextended domains; full text); Montadert et al. 1979 (the Moho 25 km at the shelf break, 12 km on the lower margin; full text) | `game-choice` (the form) |
| PLT-476 | stage 2's lithosphere | a source thinned to `keep`: L ← L · keep, K ← min(K · keep, L) | km | McKenzie 1978 (uniform stretching of the lithosphere and its crust by β; eq. 1 and Table 2 read); Lang, ten Brink & Makovsky 2026 (the mantle stretches more; full text) | `verified` |
| PLT-477 | stage 3's drop-off | heated towards max(L_hot, K, crust) at e-folding 10 Myr; the founder sinks 1 − e^(−δt/10) of its no-room excess a step | Myr | Houseman & Molnar 1997 ("between 50 ... and 75 per cent ... of the lithospheric mantle", "of the order of 10 Myr"; full text); Conrad 2000 (60 %; abstract); Houseman, McKenzie & Molnar 1981 (under 30-50 m.y.; abstract) | `verified` |
| PLT-478 | stage 3's trigger | a Himalayan orogeny's continental column with crust > `collapse_thickness_km` (50) | km | England & Houseman 1989, Kay & Kay 1993 (a threshold exists, no number; abstracts) | `game-choice` (the threshold is PLT-256's) |
| ISO-110 | thermal isostasy | z_th = E (L_ref − L) / (L_cold − L_hot), E = 1,600 | m | Hyndman & Currie 2011 ("about 1,600 m at equal thickness") | `verified` |
| ISO-111 | L_ref, the frame | 225 (craton frame) or 187 (Hyndman & Currie's frame) | km | the model's Airy line against HC2011's two sea-level crusts (below) | `game-choice` (a calibration) |
| ISO-112 | the thermal age | t_eq = τ_c ln((L_cold − L_hot) / (L_cold − L)) into PLT-415 and ISO-051 | Myr | the inverse of PLT-460's cooling; the coupling is the agent's | `game-choice` |

### Why these heat-flow constants

The mantle part B / L is the heat conducted through the lithosphere's base. Over the Canadian shield
and the Kaapvaal the mantle heat flow is 11 to 17 mW/m² (Rolandone et al. 2002; Sleep 2003 citing
Jaupart & Mareschal 1999), under a lithosphere of 200 to 250 km. Taking 15 mW/m² at 225 km gives
B = 3,375 mW/m² × km (an effective conductivity of B / 1,333 °C = 2.5 W/m/K). The Archean mean
surface heat flow is 41 mW/m² (Nyblade & Pollack 1993, as two open papers cite it), so the crust part
is q_c = 26. The law then gives 53 at 125 km (the old proxy's floor was 55; Proterozoic terrains
far from cratons average 55 ± 17, Nyblade 1996's abstract) and 82 at 60 km (a back-arc).

### Why these frames

The model's Airy line is z = 525 m + (T − 35.1 km) / 7.02 (PLT-281, `continent_elevation_m`'s
middle at the normal column). Its sea stands near 600 m. Hyndman & Currie (2011) put a hot back-arc
at sea level on 27 km of crust and a craton on 38 km. On the model's line 27 km stands 1,229 m
below its sea and 38 km stands 338 m above it: the model's line lies 338 m above the craton line and
1,229 m below the back-arc line, 1,567 m between them (HC2011's 1,600 m). On a line linear in L from
60 to 225 km, the model's line is the lithosphere at 60 + (1,229 / 1,600) × 165 = 187 km: that is
Hyndman & Currie's frame, in which the start's continents begin at 187 km and cool by up to 368 m.
The craton frame takes the model's line as the craton's (L_ref = L_cold = 225): nothing that stays
cold moves and only heated ground rises, by up to 1,600 m. The pool chooses between them.

## Earth against the mod

The state's own numbers stand where the sources put them by construction: an old craton's heat flow
41 mW/m² (Earth's Archean mean 41), a cooled younger continent's 53 (Proterozoic terrains away from
cratons 55 ± 17), a back-arc's 82 (Hyndman, Currie & Mazzotti 2005 give back-arcs "high heat flow");
a keeled craton reached by an arc keeps a Te of 58 km after 40 Myr. What follows is what the switches
do to the V1 measures.

### Measured (2026-10-23)

The pool (`pool.sh table --base base --arm <arm>`): earthlike_quick_16k, 1,000 Myr, both starts,
`+DEEPTIME_CRUST_BUDGET=1`, 2 threads, against master bf8477b2's 24 runs (the base, key
7f9d8f115049). Every arm ran at key c2acdd83055b (the branch before master fc6ab402 was merged in;
that merge brought only the plate's yield, off unless set). Medians, the two-sided Mann-Whitney p
against the base in brackets.

Exploration, seeds 1 to 4 (8 runs an arm):

| arm | switches (`--set tectonics.`) | land | largest landmass | crust under the sea | above 2 km | high coast | under ice | collisions | crust's net change | criteria a run, mean |
|---|---|---|---|---|---|---|---|---|---|---|
| target | | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 | | of 7 |
| base (24) | none | 0.300 | 0.151 | 0.274 | 0.151 | 0.333 | 0.116 | 1.75 | −0.035 | 3.33 (seeds 1 to 4: 3.00) |
| th-iso | state, isostasy 1600 (craton frame) | 0.302 (0.85) | 0.131 (0.78) | 0.304 (0.36) | 0.174 (0.48) | 0.345 (0.69) | 0.158 (0.25) | 1.95 (0.32) | −0.048 (0.16) | 2.62 (0.24) |
| th-iso187 | state, isostasy 1600, frame 187 | 0.296 (0.45) | 0.170 (0.59) | 0.285 (0.98) | 0.153 (0.85) | 0.345 (0.48) | 0.119 (0.75) | 1.30 (0.023) | −0.059 (0.026) | 2.88 (0.25) |
| th-str | state, strength | 0.319 (0.013) | 0.204 (0.018) | 0.229 (0.070) | 0.147 (0.92) | 0.340 (0.92) | 0.114 (0.85) | 1.70 (0.59) | −0.033 (0.98) | 3.38 (0.78) |
| th-bs | state, backstop | 0.323 (0.009) | 0.189 (0.19) | 0.272 (0.36) | 0.160 (0.78) | 0.358 (0.36) | 0.133 (0.95) | 1.15 (0.10) | −0.029 (0.27) | 2.88 (0.22) |
| th-full | state, strength, isostasy 1600, frame 187 | 0.286 (0.38) | 0.135 (0.95) | 0.280 (0.75) | 0.138 (0.62) | 0.334 (0.78) | 0.132 (0.45) | 1.55 (0.63) | −0.074 (0.007) | 3.00 (0.31) |
| th-str-k0 | state, strength, keel 0 | 0.326 (0.020) | 0.192 (0.14) | 0.225 (0.15) | 0.138 (0.95) | 0.318 (0.51) | 0.146 (0.33) | 1.65 (0.60) | −0.026 (0.36) | 2.62 (0.100) |

- **Thermal isostasy in the craton frame** lifts the sea, not the land: every continent that is
  not at L_cold stands higher (ground made from ocean floor starts at its 125 km or less and stands
  up to 970 m higher), so the drowned crust displaces water, and the sea over the datum went from
  538 to 959 m after 100 Myr and from 861 to 1,403 m at the end. Land did not move; more crust sank
  under the sea and more land is under ice.
- **Hyndman & Currie's frame** keeps the sea nearer (738 / 1,114 m) but the start's continents sink
  by up to 368 m as they cool: fewer collisions (1.75 to 1.30) and twice the crust lost. Why the
  collisions fall was not traced.
- **The strength** is the one that moves V1 towards Earth: cold, keeled crust no longer spreads as
  a fresh orogen does, so less of it thins under the sea and the largest landmass grows. Its cost is
  land over the band's top in some runs.
- **The backstop** keeps more crust on land, too much: land over the band at the median, and the
  fewest collisions of any arm.
- **No keels** (the start's continents heat like any other behind an arc) loses the strength's gain:
  land further over the band, criteria 2.62. The keels are what the strength needs.

Final, the strength (`thermal_state=true`, `thermal_strength=true`), seeds 1 to 12 (24 runs):

| arm | land | largest landmass | crust under the sea | above 2 km | high coast | under ice | collisions | crust's net change | criteria a run: median, mean | sea over the datum after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|
| base | 0.300 | 0.151 | 0.274 | 0.151 | 0.333 | 0.116 | 1.75 | −0.035 | 3, 3.33 | 538 / 861 |
| th-str | 0.314 (0.049) | 0.182 (0.010) | 0.251 (0.18) | 0.146 (0.86) | 0.295 (0.32) | 0.113 (0.50) | 1.80 (1.0) | −0.032 (0.78) | 3, 3.42 (0.57) | 543 / 817 |

Runs meeting each criterion, base against th-str: land 20 to 13, largest landmass 17 to 22, crust
under the sea 8 to 12, above 2 km 9 to 6, high coast 0 to 1, under ice 17 to 19, collisions 9 to 9.
Seed for seed the criteria rose in 10 runs and fell in 7. The ship rule (criteria mean up at
p < 0.05, no median pushed out of its band) is not met: no median left its band, but the mean rose by
0.08 at p 0.57. The strength stays off. It does what Earth's cratons do (the largest landmass and
the drowned crust both move towards their targets at p 0.010 and 0.18) and the land share's top is
what it runs into: seven runs end over 0.32.

**The strength with the plate's yield** (ISO-120, `surface.flexure.yield_anomaly_mgal=380`, master
fc6ab402; the coordinator's combined arm, key 37afd1c8fbf1, seeds 1 to 12, 24 runs), beside the
polar-peaks lane's yield alone (`pp-yield380`):

| arm | land | largest landmass | crust under the sea | above 2 km | high coast | under ice | collisions | criteria a run: median, mean | sea after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|
| base | 0.300 | 0.151 | 0.274 | 0.151 | 0.333 | 0.116 | 1.75 | 3, 3.33 | 538 / 861 |
| pp-yield380 | 0.313 (0.35) | 0.161 (0.24) | 0.253 (0.37) | 0.160 (0.50) | 0.351 (0.37) | 0.126 (0.73) | 1.80 (0.56) | 3, 3.04 (0.31) | 495 / 759 |
| th-str | 0.314 (0.049) | 0.182 (0.010) | 0.251 (0.18) | 0.146 (0.86) | 0.295 (0.32) | 0.113 (0.50) | 1.80 (1.0) | 3, 3.42 (0.57) | 543 / 817 |
| th-str-y380 | 0.309 (0.34) | 0.165 (0.26) | 0.269 (0.86) | 0.128 (0.28) | 0.278 (0.074) | 0.108 (0.48) | 1.80 (0.49) | 3, 3.17 (0.98) | 564 / 941 |

The `guard` measure on the same arms (medians): the guard's worst 13,716 m (base), 13,112
(yield, p 0.015), 14,746 (strength, p 0.60), 13,044 (both, p 0.033); the largest one-step fall
9,450, 7,422, 10,620 and 7,492 m. The two do different things. The strength is a clock: it sets the
thin sheet's viscosity and the flexure's Te from the thermal age, so a cold keeled craton keeps a Te
of up to 80 km and holds more load, and nothing in it bounds what a plate holds. The yield is that
bound. With the strength on, the guard's worst and the largest fall grow a little; the yield takes
them back down to its own values. On V1 the yield takes back most of the strength's gain in the
largest landmass (0.182 to 0.165) and the drowned crust (0.251 to 0.269). Neither arm meets the ship
rule; the yield is not made redundant by the strength, and the strength does not replace it.

## Simplifications and limits

- One thermal depth per sample and a relaxation towards a target, not a conductive geotherm: the
  first mode of McKenzie's plate solution.
- Heating is by flag (arc, back-arc, hotspot, rift), at one rate. Collisional thickening of the
  lithosphere is not modelled: a collision belt keeps the state it had until stage 3
  (`root_removal`) takes its mantle lithosphere off to the Moho, at once in the sense that nothing
  thickens it first (Earth's belt thickens its mantle lithosphere, which then drops off). The
  crust's own heating when thickened (its radiogenic heat) is not in the state either, so with
  `thermal_strength` and stage 3 off a collision between two keeled continents stays as stiff as
  their keels; with stage 3 on it stays as stiff as its keel and no stiffer.
- Stage 2 stretches the lithosphere only where the transition thins crust, which on the quick
  planet is one or two samples a side of a margin (71 km apart on the model sphere against a 200 km
  zone). A rift that has not broken up (regime 6) heats as stage 1 says (L ← L_hot) and its crust is
  not thinned by β. One β for crust and mantle, though the mantle stretches more on real margins
  (Lang, ten Brink & Makovsky 2026).
- Stage 3 removes no crust: the eclogite pool stays (Shi D. et al. 2020: a root about 30 %
  eclogitized is stable), and the founder's cap still sinks crust above 70 km that finds no room,
  only at the drop-off's pace.
- The backstop takes the cordillera's crust from weak ground only. Earth's foreland craton gives the
  Andes crust by underthrusting at its edge, moving as a rigid block; the model's samples cannot
  move under a range, so a cold source gives (almost) nothing instead.
- The keel is static: it forms only at the start and thins only in a rift. No new keel grows.
- Heat flow has one crust part everywhere (26 mW/m²): the crust's own heat production does not
  follow its rock.

## Game choices

- Every start continent is a keeled craton (PLT-463). Earth's continents are a mix of cratons and
  younger mobile belts; the planet's start stands for an old continental world. The pool says the
  keels matter: without them the strength's gain is lost (th-str-k0, "Measured").
- The frame L_ref (ISO-111) is a calibration of the model's existing Airy line, not a physical
  constant.

## Open questions

- The frame (ISO-111). Neither frame helped V1 ("Measured"): the craton frame lifts the sea, Hyndman
  & Currie's sinks the start's continents and loses crust. Hyndman & Currie's is the physical one
  (the start's continents are not all cratons at the model's line); the isostasy should wait until
  the sea's 840 m over the datum (gap 1) is gone, and be tried again in that frame.
- The heat-flow pass (PLT-469) changes only the heat flow of the cells the rate guard turned
  continental (the bake's `meta/heat_flow_mw_m2`, the deposits' and the underground's reading of
  it), not the plates' dynamics. It should go on with the release pace (ISO-099), or on its own by
  default: the fault is a bookkeeping one either way.
- The land share's top. The strength moves the largest landmass and the drowned crust towards
  Earth and pushes seven of 24 runs over 0.32 land: the crust it keeps from spreading under the sea
  stands on land instead. Whether it can go on beside a change that lowers the land share, or
  with a weaker keel than 200 km on some of the start's continents (no keel at all lost the gain),
  is open; neither was tried.
- The yield's bound (ISO-120) is one figure for every plate. A hot lithosphere yields at less than
  a cold one (the gap audit cites Rey, Teyssier & Whitney 2010 for the Moho temperature's part; not
  read for this page), so the state could set it; that would be a new row above ISO-120 and was not
  tried.
- Stages 2 and 3 (above, "The order of the stages") are designed, not built.

## Sources

- McKenzie1978: full text (scan), eq. 6 and Table 1 read (a, T_l, τ).
- HyndmanCurrie2011: full text (2026-10-05): the 1,600 m and the two sea-level crusts.
- HyndmanCurrieMazzotti2005, CurrieHyndman2006, CurrieHuismansBeaumont2008: full text (2026-10-23).
- Jordan1978: abstract only. Jordan1988: abstract only. Lee2011: full text, section 1 (the isopycnic
  hypothesis as Lee states it).
- NybladePollack1993: abstract only; the 41 mW/m² read in two open papers that cite it.
- Rolandone2002: full text (mantle heat flow, the 200 to 250 km, Jaupart & Mareschal 1999 as cited).
- Sleep2003: full text (the 225 km and the mantle heat flows of Jaupart & Mareschal 1999 as cited).
- JaupartMareschal1999: not read (paywalled); only as Rolandone et al. and Sleep cite it.
- HousemanMolnar1997, ConradMolnar1997: full text.
- HasterokChapman2007: abstracts of both parts (about 3 km of relief between hot and cold
  provinces).
- Michaut2007: abstract (Kaapvaal lithosphere 200 to 270 km, basal heat flow 12 to 16 mW/m²).
- Stages 2 and 3 (read 2026-10-23 by a sub-agent of the stages 2-3 agent; the bibliography's section
  "The lithosphere's stages 2 and 3"): McKenzie1978 again (eq. 1 on the page image, Table 2);
  ManatschalChenin2023, Montadert1979, Lang2026: full text; StecklerWatts1978, LePichonSibuet1981,
  HousemanMcKenzieMolnar1981, Conrad2000, MolnarEnglandMartinod1993, KayKay1993: abstracts only;
  EnglandHouseman1989 and ShiD2020 as read before. Chenin et al. 2017, Mohn et al. 2012, Royden &
  Keen 1980 and Davis & Kusznir 2004 could not be read (paywalled or no abstract); nothing is taken
  from them except as Manatschal & Chenin cite Chenin et al.
