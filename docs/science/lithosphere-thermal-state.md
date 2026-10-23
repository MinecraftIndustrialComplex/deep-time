# The lithosphere's thermal state

Status of this page: `stage 2 (constants checked against sources on 2026-10-23, where a source
could be read; the rest marked from-memory)`. Written by the gap-4 agent, 2026-10-23, before the
code (the brief's "design first"); the measured sections were added after the pool runs.

Ledger ids on this page: `PLT-460` to `PLT-470`, `ISO-110` to `ISO-112` (see
[claims.md](claims.md) and [ledger/PLT.md](ledger/PLT.md), [ledger/ISO.md](ledger/ISO.md)).
Every law here sits behind a `--set` switch that is off by default (`tectonics.thermal_state`,
`tectonics.thermal_isostasy_m`, `tectonics.thermal_strength`, `tectonics.thermal_backstop`,
`tectonics.heat_flow_refresh`; `tectonics.thermal_keel_km` sets the start's keel).

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
  as a new arc.

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
2. **Stage 2, rifted margins:** the transition's linear taper (PLT-261 to PLT-264) replaced by a
   stretching factor β on both sides of a new rift: crust T_n / β, lithosphere L / β, then McKenzie's
   thermal subsidence from the state (already there: a stretched margin's L recovers over τ_c and
   z_th falls by the same law). Coastal plains and shelves follow from the subsidence.
3. **Stage 3, root removal:** the founder half of the 70 km cap (PLT-258) replaced by a
   Rayleigh-Taylor removal of the thickened mantle lithosphere (Houseman & Molnar 1997: half to
   three quarters of it in about 10 Myr), which heats L under a collision belt that has no keel.

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

Filled in after the pool runs (below, "Measured").

## Simplifications and limits

- One thermal depth per sample and a relaxation towards a target, not a conductive geotherm: the
  first mode of McKenzie's plate solution.
- Heating is by flag (arc, back-arc, hotspot, rift), at one rate. Collisional thickening of the
  lithosphere and its removal (stage 3) are not modelled yet: a collision belt keeps the state it had.
  The crust's own heating when thickened (its radiogenic heat) is not in the state either, so with
  `thermal_strength` a collision between two keeled continents stays as stiff as their keels; on
  Earth a thickened orogen warms and weakens (its thickened crust's own heat, the removal of its
  root: Houseman & Molnar 1997), which stage 3 is for.
- The backstop takes the cordillera's crust from weak ground only. Earth's foreland craton gives the
  Andes crust by underthrusting at its edge, moving as a rigid block; the model's samples cannot
  move under a range, so a cold source gives (almost) nothing instead.
- The keel is static: it forms only at the start and thins only in a rift. No new keel grows.
- Heat flow has one crust part everywhere (26 mW/m²): the crust's own heat production does not
  follow its rock.

## Game choices

- Every start continent is a keeled craton (PLT-463). Earth's continents are a mix of cratons and
  younger mobile belts; the planet's start stands for an old continental world.
- The frame L_ref (ISO-111) is a calibration of the model's existing Airy line, not a physical
  constant.

## Open questions

- The frame (ISO-111): the pool's answer is under "Measured".
- Whether the heat-flow pass (PLT-469) should go on by default on its own, so the release pace
  (ISO-099) can go on without the rest.

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
