# The lithosphere's thermal state

Status of this page: `stage 2 (constants checked against sources on 2026-10-23, where a source
could be read; the rest marked from-memory)`. Written by the gap-4 agent, 2026-10-23, before the
code (the brief's "design first"); the measured sections were added after the pool runs.

Ledger ids on this page: `PLT-460` to `PLT-469`, `ISO-110` to `ISO-112` (see
[claims.md](claims.md) and [ledger/PLT.md](ledger/PLT.md), [ledger/ISO.md](ledger/ISO.md)).
Every law here sits behind a `--set` switch that is off by default (`tectonics.thermal_state`,
`tectonics.thermal_isostasy_m`, `tectonics.thermal_strength`, `tectonics.heat_flow_refresh`).

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
   keel K = min(K_0, L). A continental sample that appears later with no state (none is set) takes
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

This reuses the release pace's state rather than adding a clock: McKenzie's τ is the same law, and
the flexure's held Te (ISO-099) keeps working on top of the new target.

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
| PLT-460 | the state and its update | L ← L* + (L − L*) e^(−δt/τ) per step; heated: τ_h, L* = max(L_hot, K) (convective) or L_hot (rift, then K ← min(K, L)); else τ_c, L* = L_cold | km | McKenzie 1978 (the plate), Jordan 1978 (the keel); the relaxation form is the agent's | `cited-unverified` |
| PLT-461 | L_hot, a hot back-arc's lithosphere | 60 | km | Hyndman, Currie & Mazzotti 2005 ("50-60 km"), Currie & Hyndman 2006 ("1200°C at 60 km") | `verified` |
| PLT-462 | L_cold, an old craton's lithosphere | 225 | km | Sleep 2003 (abstract: "relatively uniform, 225 km"); Jaupart et al. 1998 200-250 km as Rolandone et al. 2002 cite it | `verified` (abstract, and a secondary source) |
| PLT-463 | K_0, the start's keel | 200 | km | Jaupart & Mareschal 1999 as Rolandone et al. 2002 cite it ("unlikely to be smaller than 200 km") | `secondary` |
| PLT-464 | τ_h, the heating time | 10 | Myr | Houseman & Molnar 1997 (p. 131: "may require less than 10 Myr if the viscosity is less than about 10^21 Pa s"); Currie, Huismans & Beaumont 2008 (5-10 Ma) | `verified` |
| PLT-465 | τ_c, the cooling time | 62.8 × (L_cold / 125)² = 203.5 | Myr | McKenzie 1978 (eq. 6, τ = a²/(π²κ); Table 1, a = 125 km, τ = 62.8 My) | `verified` (the scaling with a² is the equation's) |
| PLT-466 | the ocean floor's lithosphere | min(2.32 √(κ t), 125), κ = 0.80 × 10⁻⁶ m²/s from McKenzie's τ and a | km | McKenzie 1978 (a, τ); the 2.32 (the 90 % isotherm of a half-space) is from memory | `from-memory` |
| PLT-467 | continental heat flow | q = q_c + B / L, q_c = 26 mW/m², B = 3,375 mW/m² × km | mW/m² | Archean mean 41 (Nyblade & Pollack 1993 as two open papers cite it); mantle heat flow 11 to 17 (Rolandone et al. 2002; Sleep 2003 citing Jaupart & Mareschal 1999), 15 taken at L_cold | `secondary` |
| PLT-468 | the magmatic terms | +40 arc, +40 hotspot (PLT-137's); the rift's +30 dropped | mW/m² | PLT-137 (uncited) | `uncited` |
| PLT-469 | the final heat-flow pass | after the resampling, continental q ← max(q, the law's conductive value) | mW/m² | the bake test's fault (2026-10-23) | `game-choice` (a bookkeeping fix) |
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
