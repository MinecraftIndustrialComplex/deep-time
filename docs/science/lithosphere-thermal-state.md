# The lithosphere's thermal state

Status of this page: `stage 2 (constants checked against sources on 2026-10-23, where a source
could be read; the rest marked from-memory)`. Written by the gap-4 agent, 2026-10-23, before the
code (the brief's "design first"); the measured sections were added after the pool runs.

Ledger ids on this page: `PLT-460` to `PLT-470`, `PLT-475` to `PLT-478`, `ISO-110` to `ISO-112`
(see [claims.md](claims.md) and [ledger/PLT.md](ledger/PLT.md), [ledger/ISO.md](ledger/ISO.md)).
**On by default since 2026-10-23** (Ben's rule of that day, "Always, fix V1 elsewhere": physics goes
on once it is built and correct, and the rule it replaces goes off with it; "On by default" below):
the state with its heat flow (`tectonics.thermal_state`), the strength from it
(`tectonics.thermal_strength`), stage 2's stretched rifted margins (in place of the linear taper)
and stage 3's root removal (in place of the founder's sinking at once). Stages 2 and 3 have no
switch: the rules they replace are gone. **Off by default:** thermal isostasy
(`tectonics.thermal_isostasy_m`, an open fault: no frame for it is right while the model's sea is
not) and the backstop (`tectonics.thermal_backstop`, a stand-in, not physics); the heat-flow pass
(`tectonics.heat_flow_refresh`, PLT-469) is on by default too, and the state turns it on for
itself. `thermal_state=false`
remains for tests that isolate one law. `tectonics.thermal_keel_km` sets the start's keel. Stages 2
and 3 were added by the gap-4 stages 2-3 agent on 2026-10-23, after a diagnosis of where the high
coasts are.

## What you see in the world

The lithosphere is the cold, strong outer shell of the planet: the crust and the mantle under it
down to the depth where the rock is hot enough (about 1,333 °C) to flow. Under a hot region it is
thin, 50 to 60 km; under an old craton it is 200 to 250 km. A hot lithosphere stands higher at the
same crust, because warm rock is lighter, and it is weak; a cold one stands lower and is stiff.

Ground behind an active subduction zone, over a hotspot and in a continental rift warms over about
ten million years and grows weak; it cools and stiffens slowly once the heating stops, over hundreds
of millions of years. The old continents the planet starts with carry a depleted, buoyant keel that
back-arc heating cannot remove, so they stay cold and stiff when an arc reaches them. A mountain
belt whose crust a collision thickened loses the thickened mantle under it in about ten million
years, heats and weakens, and spreads. A rifted margin thins towards the new ocean the way stretched
continents do: barely at first, fastest at the old boundary, very thin at its outer edge. The heat
flow the game reads (geothermal fields, metamorphism, deposits) comes from the same state: about
41 mW/m² over an old craton, about 53 over a cooled younger continent, about 82 behind an arc. With
thermal isostasy (off by default) warm ground would also stand higher, by up to 1,600 m.

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
   its floor's lithosphere at the floor's age, at least its lid's floor, and no keel (PLT-821,
   since 2026-10-31; L_ref until then).

A resampling copies both numbers from the nearest corner (`ridges.rs` `interpolate`, PLT-664), and
the final pass raises L to the new column's lid floor (PLT-820).

### Where the state goes

- **Heat flow** (`tectonics.thermal_state`): continental heat flow is a crust part and a mantle part,
  q = q_c + B / L, with B / L the heat conducted up through the lithosphere's base, plus the old
  proxy's magmatic terms for an arc and a hotspot (+40 each, PLT-137). The proxy's +30 for a rift is
  dropped: a rift's thinning is now in L. The ocean keeps its age law.
- **The heat-flow fault** (`tectonics.heat_flow_refresh`, on by default, and on with the state): after the step's
  resampling, the rate guard can spread crust onto old sea floor and turn it continental
  (`collapse::spread`, PLT-331). That happens after the step set every sample's heat flow, so the
  converted sample kept the sea floor's value (33.7 mW/m² on 229 Myr floor) into the bake's
  continental heat-flow layer. A final pass after the resampling gives every continental sample at
  least its law's conductive value. Young floor's value is above the law (up to 300 mW/m² under
  about 3 Myr), which "at least" kept, so since 2026-10-31 the floor's value goes with the floor's
  kind (`Sample::make_continental` leaves none) and the pass gives such a sample its law's value, as
  the step would have ("The two faults folded in", young floor).
- **Thermal isostasy** (`tectonics.thermal_isostasy_m` > 0; off by default, "On by default" says
  why): the Airy height a continental column
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
- **The sheet's gate** (`tectonics.collapse_sheet_thermal`, PLT-581, on by default since
  2026-10-25; needs the strength): the thin sheet no longer asks whether an orogeny raised the
  crust, only how hot its lithosphere is. Crust the founder or the lower-crustal flow thickened
  over a hot lithosphere thins on as an orogen's does; a craton at L_cold stays rigid because its
  thermal age is infinite (plates.md "The collapse reads the thermal state").

- **The backstop** (`tectonics.thermal_backstop`; off by default, a stand-in: "On by default"): an
  arc's cordillera grows by crust shortened
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
`tectonics.heat_flow_refresh`, on by default since 2026-10-23, and always on with the state): after the resampling and the spread,
every continental sample gets at least its law's conductive value (the proxy's
55 + 35 e^(−t/60), at least 55 mW/m²; with the state, q_c + B / L). The same planet with the pass bakes no cell under 55 (the scan's
rerun), and `the_release_pace_bakes_continental_heat_flow_with_the_final_pass` bakes it with the
release pace on and holds every continental cell to at least 54.9;
`the_thermal_state_bakes_its_own_heat_flow` bakes it with the state on and holds them to the
state's floor, an old craton's 41.

**Young floor (2026-10-31).** With the collision orogen on by default (branch collision-orogen-on,
PLT-730, PLT-731, PLT-740) the release-pace test's planet baked 300 mW/m² at (45, 140), over the
test's 200.1 (the proxy's most is 55 + 35 + 40 + 40 + 30 = 200). A per-stage print on its last step
found the path: the rate guard after the resampling (`rate::limit`, PLT-287, spreading by PLT-277)
put 9.3 km of rock from continental sources onto 2.4 Myr floor (a 16.3 km column) and some onto
6.6 Myr floor, and both turned continental at once (PLT-662, the margin's sediment continental, on
by default since 2026-10-31). The plateau (PLT-740) placed nothing there: it grows only through
continental samples. The step had set every heat flow before that, so the two kept the floor's,
510 / √2.4 capped at 300 (ISO-016) and 198.8, and the pass's "at least" left a value above the
law alone: the fault of 2026-10-23 seen from the other side, since old floor's value is under the
law and young floor's over it. The label stays as PLT-662 gives it (the spread rock is continental
crust, its floor's 7 km are its basement, PLT-331); the heat flow is what was wrong. The fix keeps
the rule the resampling already follows, heat flow belongs to the crust's kind: a kind change
leaves no heat flow (`Sample::make_continental` sets 0), and the pass gives a continental sample
with none its law's full value, as `finish_samples` would (`continental_heat_flow`, the one law
both call: the proxy with its terms; the state over the sample's own lithosphere, the floor's
2.32 √(κt) where `thermal::step` set it, and for floor the resampling made that step, which has no
state yet, the frame's depth until 2026-10-31, since then its floor's at least its lid's floor,
PLT-821). Test: `young_floor_made_continental_after_the_step_takes_the_continental_law`.

Master has the same fault without the orogen. A scan of the plate stage alone on the test's preset
(`pipeline_quick_16k` at 3,000 samples, the release pace on, 60 Myr, scattered_continents and
supercontinent, seeds 1 to 12; a temporary test, not kept), before the fix: crust that kept a
floor's heat flow above its law in 22 of the 24 planets of master's switches with the proxy, and
continental heat flow over 200.1 mW/m² at the final frame in 6 of 24 (13 at some step); with the
state, the shipped default, 1 of 24 at the final frame (7 at some step, up to 300); with the orogen
on, 11 of 24 with the proxy and 3 of 24 with the state. After it no crust in the 96 runs keeps a
floor's value, and none passes 200.1 with the proxy. With the state and the orogen, 3 of 24 runs
reach 203.4 to 204.5 at one step (none at the final frame); two of them baked the same value
before the fix, and no sample there kept a floor's. That is the state's own law, which has no
ceiling at 200 (26 + 3,375 / L reaches 204 with an arc's 40 at L = 24 km, or with an arc's and a
hotspot's at L = 34 km: arithmetic, the samples were not traced), so the 200.1 of the state's bake
test holds for its planet, not for the law.

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

**The lid's floor (2026-10-31; PLT-820 to PLT-823).** The open question of PR #50's review: the
heat-flow law, 26 + 3,375 / L (PLT-467), has no ceiling as L falls. Where the continental laws take L
from, by the code:

- *Floor made continental* keeps the floor's lithosphere (`Sample::make_continental`), the
  2.32 √(κt) `thermal::step` set at the floor's age (PLT-466). Floor the last resampling made is at
  least one step old there (2 Myr on every shipped preset: 16.5 km, 231 mW/m²); 2.4 Myr floor is
  18 km and 213 mW/m².
- *The state's update* heats a column towards L_hot's 60 km under an arc, a subduction zone, a
  hotspot or a rift, whatever its crust (PLT-460): a 70 km belt under an arc had its lithosphere's
  base 10 km above its Moho. Only stage 3's drop-off stopped at the crust (PLT-477).
- *Stage 2's stretch* thins L with the crust, L × keep, down to 1 km (PLT-476).
- *Crust with no state* (made continental on floor the resampling made in the same step: the rate
  guard's spread onto a gap point, a volcanic margin) took the frame's 225 km (ISO-111): an old
  craton's 41 mW/m² over floor days old, the fault the other way.

The 1 km clamp itself (3,401 mW/m²) is the law's reach, not a path the shipped presets take by the
code: the floor's L is at least a step's 16.5 km, and a stretch keeps a source its target crust
(PLT-261, at least the ocean's 7 km) and thins L and crust by the same factor, so L reaches 1 km only
under a lithosphere already about seven times thinner than its crust, or with a step under 0.0074
Myr (no run traced it). What the law did reach is the cause: nothing held the lithosphere's base
below the crust.

What bounds continental heat flow on Earth (read by one research sub-agent, 2026-10-31;
bibliography.md "The sources of the continental lid's floor"). The thermal lithosphere is the
conductive lid over the convecting mantle (Jaupart & Mareschal 2007, p. 220: "The shallowest
boundary, h1, corresponds to the lower boundary of the conductive upper part and of what we shall
call the thermal lithosphere"); the crust is in it. Where extension has taken the mantle
lithosphere away, the crust lies on the asthenosphere and its base stands at the basalt solidus, not
at the mantle's temperature: in the Salton Trough "The base of the crust is at or near asthenosphere
(basalt-solidus) temperatures" (Lachenbruch, Sass & Galanis 1985, p. 6723), "the lithosphere has
been thinned to the crust" (p. 6731), 1080 °C at its 23.5 km Moho (Fig. 18's caption), and faster
extension does not heat it further, "because of the narrow melting interval for basalt" (p. 6728).
The static conductive flux of that crust, K × 1080 / 23.5, is their "q* ≈ 125 mW/m²", "the same as
the mean reduced heat flow observed in the Imperial Valley" (p. 6725), whose modal surface heat flow
is "about 140 mW m-2" (p. 6712). Earth's highest regional continental heat flows sit there: "can
reach 120 mW/m² in magmatic provinces and active rift zones" (Hasterok & Chapman 2011, p. 66); the
Battle Mountain High "100 to 150 mWm-2" (Blackwell 1983, p. 84); above 120 "would suggest extensive
melting which is not common in the crust" (Davies 2013); "Steady-state thermal models are only valid
if heat flux is less than about 90 mW m–2. Higher values imply melting in the crust or weak
lithospheric mantle" (Jaupart & Mareschal 2007, p. 238). Above those the values are geothermal
fields (Lachenbruch et al. 1985, p. 6712: "Higher values (> 200 mW m-2) all correspond to known
geothermal fields"; Poort & Klerkx 2004: the Baikal rift's 300 to 35,000 mW/m² anomalies "not larger
than 2 to 4 km in diameter").

The rule (PLT-820). A continental column's thermal lithosphere reaches no shallower than where the
plate model's own steady geotherm (ISO-138, `thermal::geotherm_share`: the crust's radiogenic heat
decaying as e^(−z/h_r) over the mantle's B / L, T_m = 1,350 °C at L) puts its Moho at the crust's
basal solidus T_s = 1,080 °C (PLT-822; one place's value, the Salton Trough's at its 23.5 km Moho, applied to
every Moho depth and crust: the agent's extrapolation, and the caption's symbol was lost to the scan's OCR):
T(z_Moho) = T_s, solved for L by Newton's method
(`thermal::lid_floor_km`), with z_Moho the column's rock: 1.254 z_Moho over the ocean's 7 km column,
1.268 over 23.5 km, 1.274 over 70 km. (The mantle part alone, linear through the lid, would put T_s
at 1.25 z_Moho, the first build's floor; the radiogenic part heats the Moho beyond that.) A
lithosphere thinned to that floor has its crust on the asthenosphere, as Lachenbruch et al.'s Salton
Trough has it, and the law there is 26 + B / L_floor before PLT-468's magmatic terms: 139.3 mW/m²
over the Salton Trough's 23.5 km (PLT-823), 101.8 over a 35 km crust, 410.4 over the ocean column's
7 km, a spreading centre's. Over 23.5 km that is 113.3 from the mantle and 26 from the crust,
against Lachenbruch et al.'s ~125 reduced (their static conductive q*, which implies K ≈ 2.72 W/m/K
by the agent's arithmetic, against the law's 2.69) and "about 15 mW m-2" from crustal radioactivity:
the totals agree (139 against ~140), the parts do not, so the law has their form and not their
value (the comparison is the agent's). It is applied where L is set (a floor made continental; every
step's heating and cooling, `thermal::step`, hidden samples included, its targets raised to the
floor; stage 2's stretch, in place of its 1 km; the final pass after the resampling, which stores it)
and where the laws read it (`thermal::sample_l_km`: `continental_heat_flow` and the final pass,
PLT-469). Crust with no state takes its floor's 2.32 √(κt) at the floor's age, at least the floor
(PLT-821), in place of the frame's 225 km. Not a cap on the law: the lid cannot be thinner than its
crust allows, and the bound follows from it; a resampled column's blend is held to its own law's
ceiling (PLT-824, below).

What it changes, by the code (the arms below): a column of crust thicker than 47.1 km under an
arc's, a hotspot's, a subduction zone's or a rift's heating e-folds towards its lid's floor, not
L_hot's 60 km (a 70 km belt towards 89.2 km: 103.8 mW/m² with the arc's 40 against 122; its thermal
age 39.6 Myr, not 0, so its thin sheet stiffer, η_0 / η 0.53 at τ_s 62.8, ISO-112); stage 3's
drop-off e-folds towards the floor, 1.27 times the crust, not to the Moho; floor made continental
under more than 14.3 km of rock keeps its column's floor (a 16.3 km column on 2.4 Myr floor:
20.6 km, 190 mW/m², not 213); crust on brand-new floor is hot and young (8.8 km over the ocean's
7 km column to about 25 km, not 225), its thin sheet weak; a stretch binds only where L was already
under the stretched crust's floor. A column of normal crust under no heating (L 225, 35 km) is
unchanged. Under thermal isostasy (ISO-110, off by default; on in
`the_thermal_state_bakes_its_own_heat_flow`) PLT-821 lifts crust with no state from a thermal height
of 0 (L 225) to E (225 − L) / 165, +1.9 to +2.1 km at E = 1,600 m, and the floor lowers a heated
70 km belt's from 1,600 m to 1,317 m. Tests: `a_continental_lithosphere_reaches_down_to_at_least_its_moho`
and `the_final_frame_holds_every_continental_column_to_its_lid_floor` (plates/tests.rs),
`the_lid_floor_puts_the_moho_at_the_crusts_basal_solidus` (thermal.rs). The magmatic terms (PLT-468, uncited) and the crust's radiogenic share, 26
mW/m² over any crust (McKenzie 1978: Fc / β after stretching; Jaupart & Mareschal 2007, p. 239: the
thinned crust's heat flux "will reflect the smaller amount of radioactive elements"), stay as they
were.

Traced after the gate (temporary prints, not kept). On the plate stage alone of the bake tests'
planet (`pipeline_quick_16k` at 3,000 samples, 60 Myr, the state with thermal isostasy and strength,
seeds 1 to 6 from both starts), continental heat flow passed 200.1 mW/m² at some step in 6 of the 12
runs and at the final frame in 1, by two paths: crust with no state on floor 1.5 to 1.6 Myr old
(PLT-821: 264.3 over an 8.3 km column, 212.1 over 14.5 km), under its floor's own 2.32 √(κt),
deeper than its lid's floor, where it read 41 before; and an arc's 40 over floor made continental
2 to 4 Myr before, a 7.0 to 7.9 km column under 24.5 km of lithosphere (203.4 to 203.8), the 203.4
to 204.5 of the young-floor scan above, its samples now traced. In the full pipeline,
`the_thermal_state_bakes_its_own_heat_flow` bakes 259.8 at (21, 279), the run's one sample over
200.1: crust with no state at the final frame, a 7.3 km column on 1.54 Myr floor (14.4 km of
lithosphere). Both are the law at the floor's age, and the floor bounds them: the most the state
gives is its law over the thinnest crust the model makes continental, the ocean's 7 km column (floor
made continental starts from it; a stretch leaves a source at least its target crust, PLT-261) at
its floor, 8.78 km on the plate model's geotherm (8.75 on the first build's linear one), 410 mW/m²
before the magmatic terms and 490 with both (412 and 492 on the first build). The bake tests now hold the
state to that ceiling (`state_heat_flow_ceiling` in bake/tests.rs) in place of the proxy's 200.1,
which held for their planets, not for the law; the release-pace test, with the proxy, keeps 200.1.
Those columns are not Earth's continental crust of PLT-823's regional maxima (100 to 150): a 7 to 15
km column on floor a few Myr old is a margin's sediment or an arc's crust on a spreading centre's
lithosphere, and its heat flow is that lithosphere's.

**The arm (2026-10-31).** `pool.sh run hc-on`, the branch's default (key f2d26e94aec9;
`earthlike_quick_16k`, 1,000 Myr, seeds 1 to 4 from both starts, the crust budget's record on), against
base35-s4, medians of 8 with Mann-Whitney p: no guard failure (the worst 10,241 m from the sea against
10,304, no step beyond 2 Earth reliefs, no cell 5 km over Airy), and no census measure moved at p
under 0.3. Land 0.211 against 0.216; largest landmass 0.060 against 0.048; coast with high ground
0.111 against 0.071 (p 0.51; over its 0.11 in 4 runs against 1); criteria met 5, mean 5.00 against
5.38 (p 0.32). The high coasts shift towards collision belts (coastset: their share 0.750 against
0.517, the arcs' 0.172 against 0.355; read below, "What the review's fixes found"). The heat flow at the final frame
(the pool's `hfcells`, version 2, against base35-hf: base35's key run again with the heat-flow fields
kept, its 8 worlds bit-identical to base35-s4's by the state's sha256): median 48.8 mW/m² against
47.4 (p 0.19), mean 53.7 against 52.6 (0.10); maximum 196.5 against 162.3 (0.013), 406.7 in one run;
the share of continental cells over 150 mW/m² 0.015 % against 0.004 % (0.050); over 200, 0.0005 to
0.0023 % in 4 runs and none in any base run (0.032). The tail rises, as the trace above has it:
crust with no state takes its young floor's lithosphere (PLT-821) where it took 225 km. A cell at
406.7 is, by the law's arithmetic (the cells were not traced: the runs keep no checkpoint), a column
of at most 7.1 km at its lid's floor or on floor at most 0.58 Myr old with no magmatic term, at most
7.9 km or 0.72 Myr with the arc's 40, at most 8.9 km or 0.93 Myr with both; or a column of any
thickness that the resampling blended with a hotter corner of its kind, which the final pass let
stand (PR #58's review, below; the class blend and the column's ceiling since).

**PR #58's review (2026-10-31; PLT-824).** The review found four gaps in the first build, fixed on
the branch. (1) The resampling blended heat flow over the corners of the sample's kind, so a thick
column took a share of thin or young-floor crust's heat flow (a 50 km column with a 0.4 weight of
231 mW/m² baked about 125, over its own bound of 80 before the magmatic terms), and the final pass
only raised values. The blend now runs over the corners of the sample's class, as the pools do
(`ridges.rs`, the kind and hyper-extension, PLT-293 and PLT-663), and the final pass holds every
continental sample's heat flow to its column's ceiling, its law at its lid's floor with an arc's and
a hotspot's terms (`thermal::column_ceiling_mw_m2`, PLT-824): a blend of a hotter corner is not the
column's. (2) The floor clamped the heating's and stage 3's approaches partway (a 70 km belt reached
87.5 km after 21.8 Myr and stopped there); every target is now raised to the floor, so the approach
e-folds to it (PLT-460, PLT-477): the floor is where the lid can be. (3) The laws that read the
stored L (the flexure's thermal age, the strength laws) saw a resampled column under its floor until
the next step; the final pass now stores the floored L, and rifting's weakness reads crust with no
state through PLT-821 (PLT-506). (4) The floor was the mantle part's linear geotherm, not ISO-138's,
which the rule cited; it is now solved on ISO-138's geotherm (the rule above), the Salton Trough's
bound 139.3 with its parts the agent's comparison (PLT-823), and T_s is labelled one place's value
(PLT-822). The final-frame test runs the plate stage and fails with either PLT-820 or PLT-821
reverted (checked: with the floor at 0, a 57.9 km column at L = 62.0 km has its 1080 °C isotherm at
48.7 km; with crust with no state at 225 km, its heat flow falls under its floor's law).

What the first arm's shifts come from (the coordinator's question; hc-on, the first build, against
base35-s4, from the cached runs). The high coasts' share in collision belts rose from 0.517 to 0.750
(Mann-Whitney p 0.021) while the rate of high ground on a collision belt's coast did not move (0.445
against 0.470, p 0.96): the arcs lost theirs (their share of the coast 0.182
against 0.228, p 0.01; their rate of high coast 0.071 against 0.131, p 0.083; their high ground's
crust 49.0 against 47.7 km, p 0.13). By the code the floor reaches the plates only through the laws
that read L (the collapse's and the interior's thin sheets, PLT-415 and PLT-733 by PLT-652's
weakness; the rifts' weakness, PLT-506; the flexure's Te through the thermal age), and the columns it
binds are heated crust over 47 km, arcs and collision belts. How much stiffer depends on the crust: a
70 km belt reaches a thermal age of about 40 Myr where it was at 0 (η_0 / η 0.53 against 1), but at
the arcs' high-ground crust of 49 km (the median above) the floor is 62.4 km, a thermal age of 3.0 Myr
and η_0 / η 0.95, at 55 km 12.8 Myr and 0.82: the stiffening is in the arcs' thick tail, so stiffer
arc belts are a weak fit for the median arc, not shown by a single-cause arm. The weld seeds: the sweep's welding seeds did not fall (master 16 scattered and 12
supercontinent of 24; the first build 17 and 14; with the review's fixes 18 and 17), so the seeds
that stopped welding are the planets' divergence from a changed L, not fewer welds; the weld tests'
seeds still weld.

### The order of the stages

1. **Stage 1:** the state, heat flow from it with the fault's final pass, thermal isostasy and
   strength, each its own switch. The state and the strength on by default since 2026-10-23.
2. **Stage 2, rifted margins** (built 2026-10-23, on by default, below): the transition's linear
   taper (PLT-261) replaced by a stretching factor β on both sides of a new rift: crust T_n / β,
   lithosphere L / β, then McKenzie's thermal subsidence from the state (a stretched margin's L
   recovers over τ_c; its height follows only once thermal isostasy is on, z_th falling by the same
   law). Coastal plains and shelves follow from the subsidence.
3. **Stage 3, root removal** (built 2026-10-23, on by default, below): the founder half of the
   70 km cap (PLT-258) replaced by a Rayleigh-Taylor removal of the thickened mantle lithosphere
   (Houseman & Molnar 1997: half to three quarters of it in about 10 Myr), which heats L under a
   collision belt that has no keel.

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

### Stage 2 as built (PLT-475, PLT-476)

Measured below as the switch `tectonics.transition_beta`, removed when it went on by default.


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
- **By default** (the state on, thermal isostasy off) the stretched margin's heat flow and strength
  change (a hot, weak margin) and its height is the crust's alone: the thermal subsidence waits for
  thermal isostasy. With the state off (tests only) there is no lithosphere to thin and only the
  crust's profile is drawn.

### Stage 3 as built (PLT-477, PLT-478)

Measured below as the switch `tectonics.root_removal`, removed when it went on by default.


- **Which columns.** A continental column of a collision orogeny (Himalayan) whose crust is thicker
  than `collapse_thickness_km` (50 km, PLT-256): crust thickened that far stands for a lithosphere
  thickened with it. No source read gives a threshold (England & Houseman 1989: the instability
  "would only occur after there had already been substantial thickening of the lithosphere"; Kay &
  Kay 1993: "a critical amount of shortening is required"), so the collapse's own threshold is
  reused. An Andean range's lithosphere is the arc's (stage 1 already heats it).
- **What happens.** Its lithosphere relaxes towards max(L_hot, K, its crust, its lid's floor) with an
  e-folding time of 10 Myr: the thickened mantle lithosphere drops off to the lid's floor (where its
  Moho stands at the crust's basal solidus, 1.27 times the crust, PLT-820; to the Moho until
  2026-10-31), 63 % of the way in the first
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
- **By default** (the state and the strength on) the heated belt's heat flow rises and its thermal
  age falls to a fresh orogen's, so it spreads and its flexure's Te drops. With `thermal_isostasy_m`
  it would also rise by up to E = 1,600 m (Molnar, England & Martinod 1993: "approximately 1000 m to
  2500 m in a few million years" over Tibet). With the state off (tests only) only the founder's
  pace is there.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| PLT-460 | the state and its update | L ← L* + (L − L*) e^(−δt/τ) per step; heated: τ_h, L* = max(L_hot, K) (convective) or L_hot (rift, then K ← min(K, L)); else τ_c, L* = L_cold; every L* raised to the column's lid floor (PLT-820, since 2026-10-31) | km | McKenzie 1978 (the plate), Jordan 1978 (the keel); the relaxation form is the agent's | `game-choice` (the form; its constants are the rows below) |
| PLT-461 | L_hot, a hot back-arc's lithosphere | 60 | km | Hyndman, Currie & Mazzotti 2005 ("50-60 km"), Currie & Hyndman 2006 ("1200°C at 60 km") | `verified` |
| PLT-462 | L_cold, an old craton's lithosphere | 225 | km | Sleep 2003 (full text, abstract: "relatively uniform, 225 km"); Jaupart et al. 1998 200-250 km as Rolandone et al. 2002 cite it | `verified` |
| PLT-463 | K_0, the start's keel | 200 | km | Jaupart & Mareschal 1999 as Rolandone et al. 2002 cite it ("unlikely to be smaller than 200 km") | `abstract-only` |
| PLT-464 | τ_h, the heating time | 10 | Myr | Houseman & Molnar 1997 (p. 131: "may require less than 10 Myr if the viscosity is less than about 10^21 Pa s"); Currie, Huismans & Beaumont 2008 (5-10 Ma) | `verified` |
| PLT-465 | τ_c, the cooling time | 62.8 × (L_cold / 125)² = 203.5 | Myr | McKenzie 1978 (eq. 6, τ = a²/(π²κ); Table 1, a = 125 km, τ = 62.8 My) | `verified` (the scaling with a² is the equation's) |
| PLT-466 | the ocean floor's lithosphere | min(2.32 √(κ t), 125), κ = 0.80 × 10⁻⁶ m²/s from McKenzie's τ and a | km | McKenzie 1978 (a, τ); the 2.32 (the 90 % isotherm of a half-space) is from memory | `from-memory` |
| PLT-467 | continental heat flow | q = q_c + B / L, q_c = 26 mW/m², B = 3,375 mW/m² × km | mW/m² | Archean mean 41 (Nyblade & Pollack 1993 as two open papers cite it); mantle heat flow 11 to 17 (Rolandone et al. 2002; Sleep 2003 citing Jaupart & Mareschal 1999), 15 taken at L_cold | `abstract-only` |
| PLT-468 | the magmatic terms | +40 arc, +40 hotspot (PLT-137's); the rift's +30 dropped | mW/m² | PLT-137 (uncited) | `uncited` |
| PLT-469 | the final heat-flow pass | after the resampling, crust made continental after the step (no heat flow since 2026-10-31) q ← its law's value; other continental q ← max(q, the law's conductive value) | mW/m² | the bake test's faults (2026-10-23; young floor, 2026-10-31) | `game-choice` (a bookkeeping fix, on by default since 2026-10-23) |
| PLT-470 | the backstop | a source's share of the cordillera's strain w = e^(−t_eq / τ_s), τ_s = 62.8 (PLT-415) | none | Hyndman, Currie & Mazzotti 2005 (back-arcs weaker than cratons "by more than a factor of 10"); the weight is PLT-415's law | `game-choice` |
| PLT-475 | stage 2's margin profile | T* = T_o + (T_n − T_o) S(1/2 + d / W), S(s) = 3s² − 2s³ (PLT-261's S(s) = s until 2026-10-23) | km | Manatschal & Chenin 2023 (the proximal, necking and hyperextended domains; full text); Montadert et al. 1979 (the Moho 25 km at the shelf break, 12 km on the lower margin; full text) | `game-choice` (the form) |
| PLT-476 | stage 2's lithosphere | a source thinned to `keep`: L ← max(L · keep, its lid's floor, PLT-820; until 2026-10-31 1 km), K ← min(K · keep, L) | km | McKenzie 1978 (uniform stretching of the lithosphere and its crust by β; eq. 1 and Table 2 read); Lang, ten Brink & Makovsky 2026 (the mantle stretches more; full text) | `verified` |
| PLT-477 | stage 3's drop-off | heated towards max(L_hot, K, crust, the lid's floor) at e-folding 10 Myr; the founder sinks 1 − e^(−δt/10) of its no-room excess a step | Myr | Houseman & Molnar 1997 ("between 50 ... and 75 per cent ... of the lithospheric mantle", "of the order of 10 Myr"; full text); Conrad 2000 (60 %; abstract); Houseman, McKenzie & Molnar 1981 (under 30-50 m.y.; abstract) | `verified` |
| PLT-478 | stage 3's trigger | a Himalayan orogeny's continental column with crust > `collapse_thickness_km` (50) | km | England & Houseman 1989, Kay & Kay 1993 (a threshold exists, no number; abstracts) | `game-choice` (the threshold is PLT-256's) |
| PLT-820 | the lid's floor | a continental L ≥ L_floor, where ISO-138's geotherm puts the Moho at T_s (bisection; 1.254 to 1.274 z_Moho), where L is set and read | km | Lachenbruch, Sass & Galanis 1985 (the crust on the asthenosphere at the basalt solidus; full text); Jaupart & Mareschal 2007 (the thermal lithosphere, the conductive lid; full text) | `verified` (the crust on the asthenosphere at the solidus; the floor's form on the model's own geotherm, the agent's) |
| PLT-821 | crust with no state | L ← max(2.32 √(κt) at the floor's age, the lid's floor), no keel | km | PLT-466's law | `game-choice` |
| PLT-822 | T_s, the crust's base on the asthenosphere | 1,080 (one place's value, a basalt crust at 23.5 km; for every Moho depth and crust, the agent's extrapolation) | °C | Lachenbruch et al. 1985, Fig. 18's caption (the Salton Trough's 23.5 km crust; the symbol lost to the copy's OCR) | `verified` (the value; its use everywhere the agent's) |
| PLT-823 | the bound against Earth | 26 + 3,375 / 29.8 = 139.3 (113.3 + 26) against the Imperial Valley's ~140 (~125 reduced + about 15 from crustal radioactivity); regional maxima 100 to 150 | mW/m² | Lachenbruch et al. 1985; Hasterok & Chapman 2011; Blackwell 1983; Davies 2013; Jaupart & Mareschal 2007 (full texts) | `verified` (a check: the values read; the comparison, the totals alike and the parts apart, the agent's) |
| PLT-824 | a resampled column's heat flow | blended over the corners of its class (kind and hyper-extension), at most its column's ceiling, 26 + B / L_floor + 80 | mW/m² | PLT-820's law; the class is PLT-293's and PLT-663's | `game-choice` |
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
cold moves and only heated ground rises, by up to 1,600 m. Neither is right yet, as the 187 km
depends on the model's sea, itself gap 1's fault ("On by default"), so thermal isostasy is off.

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
0.08 at p 0.57, so the strength stayed off until Ben's rule of the same day put it on ("On by
default"). It does what Earth's cratons do (the largest landmass and
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

### Measured, stages 2 and 3 (2026-10-23)

The pool as above, against the same base, with the measures `v1census` and `coastset` (the
registry `tools/review/pool/measures_gap4.json`). Every arm ran at key c9ec7e5e87f6 (this branch at
1263dda0); the branch's last code commit, 8d35c39f (key 8b420c4c0fd9), changed only a test's bound.

**Off is the base.** `pool.sh alias HEAD 7f9d8f115049 --check-seed 1 --length 1000
+DEEPTIME_CRUST_BUDGET=1` at c9ec7e5e87f6: the world state's sha256 (106 files: every field and the
globals) is 7ed7d589e8ae17bf on both and the dynamics record is identical; the alias is recorded for
the base's argument set only.

Exploration, seeds 1 to 4 (8 runs an arm; base's own seeds 1 to 4 meet 3.00 criteria a run):

| arm | switches (`--set tectonics.`) | land | largest landmass | crust under the sea | above 2 km | high coast | under ice | collisions | crust's net change | criteria a run: median, mean | sea over the datum after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|---|
| target | | 0.26 to 0.32 | ≥ 0.12 | ≤ 0.25 | ≤ 0.12 | ≤ 0.15 | ≤ 0.15 | 2 to 6 | | of 7 | |
| base (24) | none | 0.300 | 0.151 | 0.274 | 0.151 | 0.333 | 0.116 | 1.75 | −0.035 | 3, 3.33 | 538 / 861 |
| g4-beta | transition_beta | 0.320 (0.086) | 0.177 (0.12) | 0.250 (0.45) | 0.145 (0.72) | 0.337 (0.75) | 0.141 (0.92) | 1.70 (0.90) | −0.015 (0.22) | 3, 3.12 (0.68) | 541 / 795 |
| g4-root | root_removal | 0.310 (0.31) | 0.149 (0.78) | 0.263 (0.75) | 0.165 (0.48) | 0.365 (0.057) | 0.116 (0.72) | 1.80 (0.71) | −0.032 (0.92) | 3, 3.00 (0.35) | 442 / 800 |
| g4-str | thermal_state, thermal_strength (stage 1's th-str at this key: every median and p the same) | 0.319 (0.013) | 0.204 (0.018) | 0.229 (0.070) | 0.147 (0.92) | 0.340 (0.92) | 0.114 (0.85) | 1.70 (0.59) | −0.033 (0.98) | 3.5, 3.38 (0.78) | 571 / 853 |
| g4-beta-str | transition_beta, thermal_state, thermal_strength | 0.320 (0.20) | 0.194 (0.005) | 0.246 (0.56) | 0.144 (0.78) | 0.303 (0.65) | 0.113 (0.48) | 1.35 (0.21) | −0.034 (0.85) | 3.5, 3.50 (0.53) | 546 / 815 |
| g4-root-str | root_removal, thermal_state, thermal_strength | 0.294 (0.95) | 0.121 (0.75) | 0.282 (0.54) | 0.131 (0.43) | 0.289 (0.24) | 0.126 (0.78) | 1.75 (0.59) | −0.027 (0.29) | 3.5, 3.25 (0.98) | 598 / 917 |

Where the high coasts are on the same runs (`coastset`, medians; settings as in "Where the high
coasts are"):

| arm | high coast | share of the high coasts: collision belt / arc / rifted margin | high-coast rate: collision belt / rifted margin | high coast without the collision belts' | high coasts facing flooded continental crust: collision belt / rifted margin | rifted margins' high ground: Himalayan share, km inland |
|---|---|---|---|---|---|---|
| base (24) | 0.333 | 0.690 / 0.190 / 0.085 | 0.642 / 0.094 | 0.094 | 0.265 / 0.501 | 0.090, 88 |
| g4-beta | 0.337 | 0.688 / 0.185 / 0.071 | 0.618 / 0.106 | 0.109 | 0.298 / 0.412 | 0.092, 99 |
| g4-root | 0.365 | 0.717 / 0.193 / 0.065 | 0.667 / 0.104 | 0.111 | 0.244 / 0.464 | 0.147, 89 |
| g4-str | 0.340 | 0.758 / 0.146 / 0.087 | 0.616 / 0.093 | 0.075 | 0.247 / 0.450 | 0.104, 99 |
| g4-beta-str | 0.303 | 0.780 / 0.153 / 0.050 | 0.640 / 0.049 | 0.060 | 0.257 / 0.416 | 0.286, 109 |
| g4-root-str | 0.289 | 0.691 / 0.216 / 0.047 | 0.590 / 0.043 | 0.076 | 0.244 / 0.318 | 0.146, 118 |

- **Stage 2 alone** (no state, so only the crust's profile changes) leans towards more land: the
  crust's net change −0.015 against −0.035 (p 0.22) and land at the band's top (0.320, p 0.086),
  neither past the seeds' spread. The smoothstep stands more of a new margin's crust on its land
  side (26.6 km at the boundary sample where the ramp held 24.6) and less on its ocean side, with
  the same crust in the zone. The rifted margins' share of the high coasts falls from 0.085 to 0.071 and the flooded
  continental crust they face from 0.501 to 0.412; coast_hi does not move, as the diagnosis said.
- **Stage 3 alone** (no state) only paces the founder, which barely fires, and still draws a
  different history from the first step that differs (the runs are chaotic). Nothing moved past
  the seeds' spread: coast_hi 0.365 at p 0.057 is the largest move, one of 8 measures in 5 arms on
  8 runs, and the sea's 442 m after 100 Myr lies inside base's 154 to 734 m.
- **Stage 2 with the strength** (g4-beta-str) meets the most criteria of any arm, 3.50 a run
  against the strength alone's 3.38 on the same seeds: the largest landmass 0.194 (p 0.005), the
  drowned crust 0.246 (inside its band at the median), coast_hi 0.303. The rifted margins' high
  coasts nearly go: their rate falls from 0.093 (g4-str) to 0.049 and their share from 0.087 to
  0.050, and what high ground they keep is more often Himalayan (0.286 against 0.104): an old suture
  rifted again. Collisions fall to 1.35 (p 0.21), further under the band's floor of 2.
- **Stage 3 with the strength** (g4-root-str) has the lowest coast_hi (0.289) and lowers the
  collision belts' rate (0.616 to 0.590 against g4-str), but a belt that loses its root is hot and,
  with the strength, as weak as a fresh orogen: it spreads, and crust thins under the sea. The
  largest landmass falls to 0.121 (the band's floor is 0.12), the drowned crust rises to 0.282 and
  the sea stands highest of the arms (598 / 917 m). Its mean, 3.25, is under the strength alone's.

**No final ran.** Under the coordinator's first rule of 2026-10-23 (a 24-run final only for an
8-run arm whose criteria mean is at least 0.5 above the base's) g4-beta-str's final (3.50 against
3.33, p 0.53) was started and stopped before any of its 16 new runs began. Ben's rule later that
day ("On by default", next) makes the V1 numbers realism checks, not a gate, so no final is run to
decide a default; the 8 runs above stay in the pool.

**Off was the base bit for bit** before the defaults changed: the alias check above, and again at
the branch's last commit with the switches (2e396409, key b40869d8b891): IDENTICAL, so the arms
measured the switches and nothing else.

### On by default (2026-10-23)

Ben's rule (decisions log 2026-10-23, "Always, fix V1 elsewhere"): physics goes on by default once
it is built and correct (unit tests against its sources, no divergence or guard failure on the
pool), and the rule or forcing term it replaces goes off with it; the V1 numbers are realism checks.
And: "Results that don't align with earth but are still perfectly playable and physics backed >
Results that are forced to align with earth by introducing extremely weird forcing functions".
Under it, on the coordinator's instruction of the same day:

- **On** in `TectonicParams::default`, and so in every built-in preset (none sets these keys): the
  state with its heat flow (PLT-460 to PLT-468; the final pass PLT-469 runs with the state), the
  strength from the thermal age (ISO-112), stage 2 (PLT-475, PLT-476) and stage 3 (PLT-477,
  PLT-478).
- **Gone:** the linear taper's shape (PLT-261's S(s) = s), the founder's sinking at once (PLT-258's
  founder half), and the switches `transition_beta` and `root_removal` (a preset that sets either
  is now refused as an unknown key). The old heat-flow proxy (PLT-137) and the orogeny's age as the
  strength's clock (ISO-051, PLT-415) remain only under `thermal_state=false`, which tests that
  isolate one law set (the collapse's fixture, the heat-flow pass's bake test with the release
  pace, and its plates test, `the_final_heat_flow_pass_lifts_only_low_continental_samples`).
- **Correct by its sources, the unit tests:** the cooling and heating laws and the keel
  (`thermal.rs`: McKenzie's τ_c and the inverse thermal age, Jordan's keel against back-arc heating,
  Houseman & Molnar's drop-off held by a keel, stretching by β), the heat flow's floor at an old
  craton's 41 mW/m² in a bake, the profile's crust kept and its domains (Manatschal & Chenin's
  proximal, necking and hyperextended order), a stretched margin's L / L₀ = T / T₀ (McKenzie's
  uniform stretching), and the founder's excess going at 1 − e^(−δt / 10 Myr) a step, mafic root
  first, to the cap after many steps.
- **No divergence or guard failure on the pool.** The `guard` measure over every cached run of the
  arms (8 runs an arm; th-str 24): every run finished, and in no run of any arm did a step leave an
  L1 cell more than two Earth reliefs (39.6 km) from the sea. The guard's worst over a run is
  12.4 to 32.8 km on g4-beta-str and 12.5 to 35.2 km on g4-root-str against the base's 11.9 to
  30.0 km, a heavy tail in the base too; no median moved at p < 0.05.

| arm | the guard's worst, m from the sea | steps beyond 2 Earth reliefs | largest rise / fall of a cell in a step, m | continental cells 5 km over Airy, share | plate cells' largest excess over Airy, m | highest plate elevation, m |
|---|---|---|---|---|---|---|
| base (24) | 13,716 | 0 | 6,992 / 9,450 | 0.0000 | 3,510 | 6,509 |
| th-str (24) | 14,746 (0.60) | 0 | 7,810 / 10,620 | 0.0000 (0.96) | 3,536 (0.70) | 6,585 (0.27) |
| g4-str | 14,594 (0.98) | 0 | 7,834 / 11,367 | 0.0000 (0.24) | 3,529 (0.48) | 6,617 (0.19) |
| g4-beta | 13,634 (0.81) | 0 | 7,720 / 10,037 | 0.0000 (0.76) | 3,531 (0.72) | 6,297 (0.10) |
| g4-root | 13,088 (0.15) | 0 | 6,804 / 9,393 | 0.0000 (0.92) | 3,858 (0.29) | 6,575 (0.69) |
| g4-beta-str | 24,208 (0.078) | 0 | 7,866 / 10,750 | 0.0000 (0.76) | 3,338 (0.56) | 6,498 (0.98) |
| g4-root-str | 13,596 (0.81) | 0 | 7,550 / 11,172 | 0.0000 (0.59) | 3,237 (0.40) | 6,580 (0.27) |

(`pool.py table --base base --measure guard`, medians, p against the base in brackets.) No arm ran
all four together (stages 2 and 3 with the strength): the coordinator's 24-run base on master after
this merge is the first, and its guard measure is that combination's check.

- **Thermal isostasy (ISO-110) stays off, an open fault.** A frame is right by its sources only if
  it puts Hyndman & Currie's two columns at the sea: a hot back-arc on 27 km of crust and a craton on
  38 km. The craton frame (L_ref = L_cold) takes the model's line for the craton's, which holds only
  if the sea stands 938 m over the datum (the model's 38 km column's height); at the model's 600 m
  it is 338 m off, and every column warmer than a craton then rises and lifts the sea (th-iso: 959 /
  1,403 m). Hyndman & Currie's frame is the right kind, since it reads L_ref from both columns, but
  it moves with the sea: L_ref = 60 + (S + 629 m) × 165 / 1,600 km for a sea S over the datum, so
  125 km at the datum and 180 to 214 km over the base's run (538 to 861 m). The sea over the datum
  is gap 1's fault: an Earth-like world's sea stands at its datum. A fixed L_ref is therefore not
  right today, and one that followed the model's sea would feed the sea's error into every column's
  height. ISO-110 goes on when gap 1 holds the sea at its datum (then Hyndman & Currie's 125 km), or
  when the continental column's height comes from the whole lithosphere's buoyancy instead of a
  frame (open questions).
- **The backstop (PLT-470) stays off, a stand-in.** It reweights a kinematic rule (PLT-048: a
  cordillera takes a share of the crust 440 to 800 km behind it) by the sheet's η_0 / η. On Earth a
  cold foreland gives the Andes its crust by underthrusting as a rigid block (Simplifications); the
  weight makes a cold craton give almost nothing instead. It is a forcing of where the crust comes
  from, not a force balance, so it stays off; the physics that would replace PLT-048 is a range fed
  by the plates' own convergence and the sheet's strength.
- **The V1 numbers, now realism checks:** on the exploration arms above, stage 2 with the strength
  met the most criteria (3.50 a run against the base's 3.33), stage 3 with the strength put the
  largest landmass on its band's floor (0.121) and the collisions stayed under theirs (1.35 to
  1.80 against 2). None is a reason to keep physics off under Ben's rule; a check that fires on
  the coordinator's base is a pointer to missing physics (open questions), not to a knob.
### Measured: the heat-flow pass on by default (2026-10-23)

Two pool arms of one source key (06b62e1202b8, master 84b8ee03 with the flip): earthlike_quick_16k,
1,000 Myr, both starts, seed 1, `+DEEPTIME_CRUST_BUDGET=1`, 2 threads. The pass on (argkey
2a53566b) and off (`--set tectonics.heat_flow_refresh=false`, argkey fa64281e), each against the
base run (master 84b8ee03, key 7f9d8f115049, argkey 2a53566b, pass off).

- **Off is the base.** The world state's per-file sha256, the per-step records (`rec.json`) and
  `plate-table.txt` of the off arm equal the base's on both starts. With the switch off the code is
  inert.
- **On against off, the L1 heat-flow layers.** The fault is fixed where it was seen. The continental
  cells under 54.9 mW/m² in the final L1 snapshot (`hfcells`, `tools/review/pool/measure_hfcells.py`;
  the lowest value in brackets): supercontinent 76 (32.3) off, 0 (55.0) on; scattered_continents
  268 (34.3) off, 0 (55.0) on. These two pairs are not a clean before and after: the worlds part
  (next point), so they differ in more than the pass.
- **On against off, the dynamics: not byte for byte.** `summary.txt` differs in 133 of its 164 lines
  (supercontinent) and 134 (scattered_continents): plates at the end 32 against 21 and 21 against 22,
  the events' count 7,844 against 8,215 and 7,958 against 8,106, the land above 2 km on the
  supercontinent start 0.099 against 0.198, the ocean's median age 55.5 against 68.2 Myr and 68.1
  against 50.8. The per-step records show where they part. Supercontinent: records 0 to 98
  identical, then `land` 28,818 against 28,822 cells and `sea_m` 475.243311 against 475.243312 m
  (6.8e-7 m) at record 99 (t = -800 Myr), the plates' other recorded numbers equal at that record,
  `land` differing in 400 of 500 records (a sea that moves by 7e-7 m moves the land count by four
  cells only if some heights already differ by more than that, so the ground differs a step
  or two before the first visible record). Scattered_continents: records 0 to 78 identical,
  then at record 79 (t = -840 Myr) `land` 33,083 against 33,082 cells, `sea_m` 103.661188 against 103.661183 m, the
  forces table and the oceanic arc count; by step 99 `land` is 32,704 against 32,672 and by the last
  step 32,663 against 32,916, with `land` differing in 420 of 500 steps and `collisions` in 28. The
  same pass on the 200 Myr planet (a different history, since the run's length sets its start
  time) changed only the two heat-flow layers and `structure/log_grade`, and a repeat of the on arm
  there was byte for byte the first.
- **What it changes in the plates, by the code.** The pass writes `Sample::heat_flow_mw_m2` of
  continental samples that sit under their law, and nothing else (`thermal::refresh_heat_flow`,
  called once at the end of the plate step). The field's readers are the next step's resampling
  (`ridges.rs`, which blends it into a heat flow and nothing else), `finish_samples` (which sets it
  afresh each step), the L1 layers `plates/heat_flow_mw_m2` and `plates/continental_heat_flow_mw_m2`,
  the surface stage's step fields (the burial temperature and the heat-flow field the deposit
  engine accumulates, `surface/fields.rs`), and after the run the structure stage's metamorphic
  grade, the bake's `meta/heat_flow_mw_m2`, the underground and the deposits. No plate, climate or
  surface law reads it back: elevation, thickness, forces, the Airy height, the flexure's Te and
  the sea level take nothing from it.
- **The two disagree and the carrier is not found.** The code says the plates' dynamics should be
  byte for byte, the pool says they part at record 79 or 99, first in the sea level the plates
  stood under (the climate stage's `sea_from_water` over the plates' elevation, which reads only
  `plates/plate_id`, `elevation_m`, `group_id`, `uplift_mm_yr` and `flags`) and in the land count.
  Two readings fit: a reader that the search missed carries heat flow into that stage or the
  plates' step, or the extra pass moved a float sum or a solver's last digit, which the plate
  dynamics then amplify over the next 400 steps. The second is the less likely: the climate's sums
  add their rows in row order on purpose (`volume_km3`), and the off arm repeats the base byte for
  byte. A repeat of the on arm at another argkey (an A/A control) separates them: a repeat that
  differs from the first on arm is noise and the pass is innocent. The next measurement is that
  pair on the merged master, and a `dump-fields` of the two arms cut at record 80
  (`stage --length-myr 1000 --max-steps 80`; the recipe is written and was not run, since this
  lane ran nothing outside the pool after the coordinator's call), to see which field differs
  first.

## Simplifications and limits

- One thermal depth per sample and a relaxation towards a target, not a conductive geotherm: the
  first mode of McKenzie's plate solution.
- Heating is by flag (arc, back-arc, hotspot, rift), at one rate. Collisional thickening of the
  lithosphere is not modelled: a collision belt keeps the state it had until stage 3 takes its
  mantle lithosphere off to the Moho, at once in the sense that nothing thickens it first (Earth's
  belt thickens its mantle lithosphere, which then drops off). The crust's own heating when
  thickened (its radiogenic heat) is not in the state either, so a collision between two keeled
  continents stays as stiff as its keel and no stiffer.
- Thermal isostasy is off by default, so no height follows the state: a stretched margin's thermal
  subsidence (stage 2) and a belt's rise when its root drops off (stage 3) wait for it ("On by
  default").
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

- **Thermal isostasy's frame (ISO-110, ISO-111), an open fault.** No fixed frame is right while the
  model's sea stands off its datum ("On by default"). The recommendation: the physics, not a frame:
  give a continental column the height of its whole lithosphere's buoyancy (the crust's density,
  the mantle lithosphere's thermal contraction and the keel's depletion) against the ridge's column,
  as Lachenbruch & Morgan (1990) do (from-memory, not read), in place of PLT-281's line, so the
  sea follows the columns' volume and no L_ref is needed; it belongs with gap 1's sea. Until then,
  once gap 1 holds the sea at its datum, Hyndman & Currie's frame at 125 km is the one their two
  columns give.
- The heat-flow pass (PLT-469) is on by default since 2026-10-23 (the coordinator's call: physics
  that is built and correct goes on without a V1 gate, and the pass fixes a real fault). By the code
  it changes only the heat flow of the cells the rate guard turned continental (the bake's
  `meta/heat_flow_mw_m2`, the deposits' and the underground's reading of it). By the pool the 1,000
  Myr runs part from record 79 or 99 ("Measured: the heat-flow pass on by default"). The path, found
  in the code by the coordinator on 2026-10-23 and not yet confirmed by a field dump: the structure
  stage reads `plates/heat_flow_mw_m2` as the metamorphism's geotherm (`structure/mod.rs`, the
  `Thermal` of each column), `metamorphism::metamorphose` rewrites the classes of the surface's
  strata log (RCK-037), the surface erodes the log by class (`surface/lithology.rs`, erodibility),
  and erosion and deposition feed the next plate step. Hotter crust metamorphoses more and erodes
  more slowly, which is physics, not a fault. It is not noise: the runs are bit-identical (off
  equals the base byte for byte), so an A/A control would only repeat the on arm. With the
  lithosphere's thermal state on (the default since 2026-10-23) the pass runs whatever its switch
  says, and the state's own heat flow (PLT-467) reaches the plates by the same path.
- **The continental law over a young floor's lithosphere had no ceiling** (a follow-up from PR #50's review, 2026-10-31). Answered the same day ("The lid's floor (2026-10-31)", PLT-820 to PLT-824): the bound belongs in L. A continental column's thermal lithosphere reaches no shallower than where the plate model's own geotherm (ISO-138) puts its Moho at the crust's basal solidus, 1,080 °C (Lachenbruch, Sass & Galanis 1985, the Salton Trough's crust on the asthenosphere), 1.254 to 1.274 times its crust (PR #58's review; 1.25 on the first build's linear geotherm); the law is then at most 26 + B / L_floor before the magmatic terms (139.3 mW/m² over 23.5 km against the Imperial Valley's ~140), the 1 km clamp is out of reach, and crust with no state takes its floor's lithosphere, not the frame's 225 km. Left open: the crust's radiogenic share over thin crust (26 mW/m² over any column) and the magmatic terms (PLT-468, uncited).
- The land share's top. The strength moves the largest landmass and the drowned crust towards
  Earth and pushes seven of 24 runs over 0.32 land: the crust it keeps from spreading under the sea
  stands on land instead. Under Ben's rule this is a realism check, not a reason to turn it off: if
  the coordinator's base flags the land, the recommendation is the missing physics that drowns old
  continents on Earth (thermal subsidence once ISO-110 goes on; the keel's own buoyancy), not a
  weaker keel chosen to hit the band.
- The yield's bound (ISO-120) is one figure for every plate. A hot lithosphere yields at less than
  a cold one (the gap audit cites Rey, Teyssier & Whitney 2010 for the Moho temperature's part; not
  read for this page), so the state could set it; that would be a new row above ISO-120 and was not
  tried.
- **The high coasts are collision belts at ocean coasts.** Neither stage reaches them: without the
  collision belts' high coasts, coast_hi would be 0.06 to 0.11 in every arm, inside its band, and
  with them it is 0.29 to 0.37. On Earth a continent-continent collision closes the ocean between
  the two and its belt lies inland. The recommendation: a lane on why a model belt forms or stays on
  an open coast (a continent against an arc or a sliver, a suture whose ocean stays open along
  strike), with `coastset`'s collision rate as its measure; that is plate boundaries' ground (gap 2)
  more than the lithosphere's.
- **The cooling time of a stretched margin.** τ_c is a 225 km plate's, 203.5 Myr (PLT-465), for
  every continent. Steckler & Watts (1978) fit the New York margin's subsidence with a thermal
  lithosphere of 113 to 139 km, about 51 to 78 Myr on McKenzie's scaling (the abstract's numbers,
  the arithmetic ours), and Montadert et al. (1979) find the time constant "increases with depth"
  across the margin. With thermal isostasy off (the default) the time sets only the margin's heat
  flow and strength. The recommendation: leave it until thermal isostasy goes on; then cool a
  sample with no keel towards a 125 km plate on McKenzie's 62.8 Myr and a keeled one towards L_cold
  on τ_c, a new row beside PLT-465.
- **Stage 3's uplift and coast_hi.** With thermal isostasy on, a belt that loses its root would
  rise by up to 1,600 m (Molnar, England & Martinod 1993 give Tibet 1,000 to 2,500 m), and seven in
  ten high coasts are collision belts. The rise is physics; the oddity is the belt at an open coast.
  The recommendation: fix where belts form (the high coasts' question above), not the drop-off.
- **Stage 3 and the strength.** With the strength on, the belt that loses its root is as weak as a
  fresh orogen and spreads (g4-root-str: the largest landmass on its floor, more crust under the
  sea). Whether the model's belt spreads further than a real one was not measured. The
  recommendation: if the coordinator's base flags it, measure the belts' spreading against Tibet's
  (the thin sheet's own test) beside the plate's yield (ISO-120), which bounds what a column holds,
  before touching the drop-off.
- **One β for crust and mantle.** Lang, ten Brink & Makovsky (2026) fit a mantle β 2.8 times the
  crust's at the Baltimore Canyon Trough. The recommendation: keep one β (PLT-476); a mantle β
  would change only a margin's heat flow and strength on this model, and nothing measures those
  against Earth yet.
- **Collisions under the band.** β with the strength had fewer collisions (1.35 at seeds 1 to 4,
  p 0.21), and the base is already under the band's floor (1.75 against 2). Not traced. The
  recommendation: trace it in the plate-boundary lane, which owns the collision count.

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
- The lid's floor (read 2026-10-31 by the continental heat-ceiling lane's research sub-agent; the
  bibliography's section "The sources of the continental lid's floor"): LachenbruchSassGalanis1985,
  JaupartMareschal2007, HyndmanCurrieMazzotti2005 again, HasterokChapman2011, Davies2013,
  DaviesDavies2010, Blackwell1983, Sass1994, PoortKlerkx2004, McKenzie1978 again: full text;
  LachenbruchSassMorgan1994, Morgan1983, Pollack1993: abstracts only.
- Stages 2 and 3 (read 2026-10-23 by a sub-agent of the stages 2-3 agent; the bibliography's section
  "The lithosphere's stages 2 and 3"): McKenzie1978 again (eq. 1 on the page image, Table 2);
  ManatschalChenin2023, Montadert1979, Lang2026: full text; StecklerWatts1978, LePichonSibuet1981,
  HousemanMcKenzieMolnar1981, Conrad2000, MolnarEnglandMartinod1993, KayKay1993: abstracts only;
  EnglandHouseman1989 and ShiD2020 as read before. Chenin et al. 2017, Mohn et al. 2012, Royden &
  Keen 1980 and Davis & Kusznir 2004 could not be read (paywalled or no abstract); nothing is taken
  from them except as Manatschal & Chenin cite Chenin et al.
