# Extraction notes: meteor impacts

Written by the stage-1 extraction agent for this subsystem (2026-10-01/02) while it built the ledger. It holds the measured Earth-against-mod numbers found in the build log, the simplifications and game choices the code or documents state, and flags: admissions of unchecked values, contradictions between comments, code and documents, and things that looked wrong. **Nothing here is verified.** A line marked "suspicion" or "my arithmetic" is the extraction agent's own reading, to be checked in stage 2. See [claims.md](../claims.md).
Everything below is from the repo at W (the worktree) as read on 2026-10-01. Row ids refer to ledger/IMP.md. Where I computed something myself from the coded formulas it is marked "my arithmetic"; those are not verified against any paper.

## Earth against the mod

The build-log entry is STATUS.md "2026-10-01 — Meteor impacts (P2-08): craters that form, age and are buried; breccia, melt and impact deposits". It contains one measured run and no comparison of the modelled crater population with Earth's confirmed crater list (no source for such a list is given anywhere in the repo's impact material). The Earth numbers in the entry are inputs to the model (the rate, the slope, the transition, the largest known crater), not independent checks.

The run behind every "Measured" number below: test VM , as the build log says "test VM"), command `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --threads 4`, 506 s wall, peak 3.7 GiB, the impact stage 1.3 s of it; driver `tools/review/p2-08-impacts.sh`; pictures in `review/out/p2-08/q16/` (laptop, not committed); date 2026-10-01; branch an agent branch, master 38ca6b0 merged in. Seed: the build log states none and the command passes no `--seed`; the preset file earthlike_quick_16k.json has `"seed": 1`. The build log adds: "The 16k numbers above were measured before the merge with P2-01." The run's folder (199 MB) was deleted afterwards.

### Rate and arrivals

| Quantity | Earth figure and stated source | Mod's figure | Run |
|---|---|---|---|
| Craters of D ≥ 20 km per km² per year | 5.6e-15 (the code comment says (5.6 ± 2.8) × 10⁻¹⁵), Grieve & Shoemaker 1994; "checked against the literature by a search" | used as input: 5.6e-15 × flux 1 | build log, Scaling laws |
| Size slope of N(≥D) | 1.8 (Grieve & Shoemaker 1994) | used as input: 1.8 | same |
| Planet area | Earth about 510 million km² (not stated in the build log; the unit test comment says "1.5 % of Earth's surface") | 7.69 million km² at 16,384 blocks of 300 m | build log; tests.rs comment |
| Expected craters ≥ 2 km | not stated for Earth | 2.7 per Myr on the planet | derived, build log |
| Expected craters ≥ 20 km | not stated for Earth | one per 23 Myr on the planet | derived, build log |
| Impactors drawn in 500 Myr | expected 1,359 | 1,302 drawn (a Poisson draw) | q16 run |
| Chance of a crater of 200 to 300 km (700 to 1,000 blocks) in 500 Myr | not stated | "about a 1 in 6 chance" (my arithmetic from the rate: 0.34 expected at 200 km and above, 0.17 at 300 km and above, so about 0.18 between, consistent) | STATUS.md pending question |

### Craters formed, by final diameter (the build log's table, as written)

| | under 5 km | 5–10 | 10–20 | 20–50 | 50–100 | over 100 km | all |
|---|---|---|---|---|---|---|---|
| craters formed | 448 | 81 | 28 | 5 | 4 | 1 | 567 |
| of them in the sea floor | 134 | 22 | 16 | 0 | 3 | 1 | 176 |
| impactors the sea stopped | 637 | 89 | 9 | 0 | 0 | 0 | 735 |
| eroded through | 354 | 62 | 16 | 5 | 1 | 1 | 439 |
| subducted | 38 | 9 | 8 | 0 | 2 | 0 | 57 |
| at the present: fresh | 6 | 1 | 2 | 0 | 0 | 0 | 9 |
| degraded | 7 | 2 | 0 | 0 | 1 | 0 | 10 |
| buried | 28 | 5 | 0 | 0 | 0 | 0 | 33 |
| eroded to their roots | 15 | 2 | 2 | 0 | 0 | 0 | 19 |

Row sums check (my arithmetic): 567 = 439 + 57 + 71, so no crater was overprinted in this run (overprinted is not in the table); 567 + 735 = 1,302 impactors. The "stopped by sea" row is classed by the crater the body would have made on Earth's land, the "formed" rows by the crater actually formed.

### Other measured figures from the run

- Catalogue at the end: 71 craters remain; 19 show (relief of 40 m or more; that is the 9 fresh plus 10 degraded); 17 of those on continental crust; 7 with a dry floor above the sea.
- Oldest crater that shows: 288 Myr old. Largest crater formed: 112 km (eroded). Earth's largest known, as the code states it: Vredefort, 300 km, 2.02 Gyr (no source given); 300 km is the `max_crater_km` cap, so the run's largest is well under it.
- 40 craters were cut into the L1 grid, 61 left bodies, 132 events were written.
- Sea: 735 of 1,302 impactors (56 %, my arithmetic) were stopped by water; 176 of 567 craters (31 %, my arithmetic) formed in the sea floor, on a planet whose preset `ocean_fraction` is 0.7. Earth figure the rule was fitted to: Davison & Collins 2007, "the sea removes two thirds of craters under 1 km, a third at 30 km, hardly any over 100 km" (abstract only). The build log does not compare the modelled shares with those three numbers.
- Deposits from the run: 10 impact-hydrothermal sites survive (72 formed); 3 crater-basin uranium sites survive (81 formed); 3 impact-diamond deposits formed and all eroded; no Sudbury-type (no melt sheet reached 500 m on land). Earth: Sudbury is the only known example of its type (impact_ni_cu_pge.json note).
- `inspect-column --why` at a hydrothermal site: its chain is the 73-km crater and the column holds `impact_breccia` under later shale and mud.
- Views picked by `tools/review/p2-08-craters.py` (young, old, largest, young complex, buried, roots): a 2.1-km simple crater 5 Myr old (7 blocks across, 6 deep in the terrain, fresh); a 4.9-km crater 40 Myr old (degraded: 3.6 blocks deep, a small lake on its floor); an 11.9-km complex crater with its central peak; a 73-km peak-ring crater 5.5 Myr old in an active arc, "a third of its relief left and hard to read among the ranges and cones (a flaw of looks, not of the model ...)". The script compares nothing with Earth; it reports the run's own terrain.

### GameTest planet (a second measurement)

`planet_world_4k.json` (circumference 4,096 blocks, gravity 9.81) runs impacts at flux 40 and `max_crater_km` 40. `./gametest.sh` and `./gametest.sh -PwithMic`, test `aCraterIsABowlOverBreccia`: 194 craters on the test planet, 80 showing; crater 444 (2.1 km, 3 Myr) is 5.5 blocks deep in the terrain with breccia in 5 of 5 columns under its floor. Date 2026-10-01, on the branch with master 38ca6b0 merged in.

### Unit-test checks against known bodies (test comments, not in the build-log numbers)

These are checks the code was written to pass, not independent measurements, and the tests carry no citation except where noted: a 1-km stony asteroid at 20 km/s makes about 15 km ("~16-km" in the scaling.rs header, accepted range 12 to 20 km); Barringer, "a ~40-m iron (7,800 kg/m³) at ~13 km/s made a 1.2-km bowl" (accepted 0.8 to 1.8 km, simple); simple craters 0.17 to 0.25 as deep as wide; rim 2 to 5 % of the diameter ("Pike 1977: 0.036 D on the Moon"); Moon transition 15 to 22 km and Mars 6 to 10 km ("Pike 1980"); a "Chicxulub-sized" 180-km crater needs an impactor of 8 to 20 km, has a peak ring and a melt sheet of 300 to 3,000 m; a 20-km crater has a central peak; a 2-km body in 3.8 km of water leaves factor 0.70.

## Simplifications and game choices

What the code or docs say is simplified or deliberately unphysical, with the reason given (or "no reason given"):

- **One impactor population.** Speed 20 km/s, density 3,000 kg/m³ and angle 45 degrees are the same for every impactor (params.rs, scaling.rs ANGLE_RAD "the most probable one"). Iron (7,800) and comet (about 1,000) densities are quoted as guidance only; there is no mixture and no spread. No reason given beyond a setting.
- **One target.** Crust density 2,700 kg/m³ and the 3.2 km transition apply to every rock; the 2 km (sedimentary) and 4 km (crystalline) split Grieve 1987 gives is quoted and not used.
- **Rate constant over the run**, Earth's per-area rate applied to the planet's own area (7.69 million km²), no gravitational focusing, no early heavy bombardment. Reason: "constant within a factor of two for the last ~3 Gyr".
- **Slope extended below 20 km** because "erosion has removed the record" there.
- **Size window.** Nothing under 2 km on Earth's land (seven blocks, three cells of the finest grid: "a smaller bowl cannot be drawn", and the atmosphere breaks up stony bodies) and nothing over 300 km (larger "would melt and resurface much of so small a planet and is not simulated"; the planet-wide effects are listed as a possible later task in the pending question).
- **Depth scale 0.5.** Every vertical measure (floor depth, rim, central peak, breccia lens, melt sheet) is its physical value times 0.5, because blocks are 20 m tall and 300 m wide: "a bowl of real proportions (1) is a shaft three times deeper than wide in blocks". Decided by the agent following Ben's 2026-09-30 volcano answer "Setting, gentler default"; the pending question asks Ben to confirm. The deposit field `impact_melt_m` uses the physical melt thickness, the strata melt layer the scaled one.
- **Peak heights** are "a design value" (0.5 of the floor depth, 0.35 for a ring), not a measured law.
- **Sea rule** is a straight-line fit to three summary numbers, "not their curves"; the water depth is read from the 4.8-km L1 grid.
- **Ageing is a blur.** Hillslope diffusion is stood in for by a tent filter of half-width √(12 κ t); erosion and deposition are the surface stage's net change averaged inside the rim; a crater is "fresh" while it keeps 75 % of its depth and "shows" at 40 m of relief (two blocks). The thresholds are the agent's.
- **Craters are not deformed by later folding**, and "an impact does nothing to the climate" (build log, Not verified). Shocked-target deposits are not modelled. No secondary craters, no ejecta rays, no asymmetry from the oblique angle (the profile is round and the ejecta fall as r⁻³ out to 3 radii, half of them laid as breccia).
- **Craters round in block metres** at every latitude, like the volcanoes.
- **Two sizes of crater** are handled differently: at least two L1 cells across (9.6 km at 16k) is cut into the deep-time surface and the strata log; smaller is drawn by the amplifier and given a lens body at the end. Impact deposits form only where breccia is in the strata log, "so not in craters under two L1 cells".
- **Impact deposit numbers are placeholders.** "No USGS model covers impact deposits: tonnages, grades and densities are design values" (the file impact_structures.json and the four deposit-type notes); the curves' thresholds are "design". The types carry `size_mult_h` 12 (6 for diamond) and `size_mult_v` 4 (no reason written; presumably body enlargement for play).
- **Impact deposits do not claim their materials** (`claims_scatter: false`): "too rare to be a world's only source of their products"; pending question to Ben.
- **Vanilla stand-ins.** `impact_breccia` is cobblestone and `impact_melt` smooth basalt until the MIC rock-tag rows are applied; pending question to Ben.
- **Oil.** Crater-basin oil is closure (0.7 inside complex craters) plus breccia as a reservoir rock, not a deposit type.
- **Default rate, depth and cap are open decisions** (STATUS.md pending questions 2026-10-01): flux 1 versus about 5; `depth_scale` 0.5 versus 1 or 0.35; `max_crater_km` 300 versus 150 or none. Not asked yet.
- **Not run:** the 32k presets and other gravities ("The 32k presets and other gravities were not run"); no real client, no BlueMap, no browser look at the workbench.

## Flags

Quotes are exact. "Suspicion" means my own reading, not a fact.

1. **Constants written from memory (the main admission).** STATUS.md build log 2026-10-01 Meteor impacts, "Not verified": "The constants of Collins et al. 2005 and the other papers were written from memory and checked for consistency by the unit tests, not against the papers (the paper could not be fetched); the one I am least sure of is the complex depth law (0.294 D^0.301 km; 0.4 D^0.3 is the other form in the literature)." The same is in crates/deeptime-core/src/surface/impacts/scaling.rs, module comment: "The constants were written from the papers as remembered and checked for self-consistency by the tests below ... STATUS.md lists what was not checked against the papers." Ledger rows marked from-memory: IMP-005, 014, 020, 023, 024, 029 to 032, 034 to 042, 045, 046, 048, 050, 052 to 055, 057, 058, 062, 063, 072, 074, 083, 128.
2. **The complex-crater depth law** (IMP-042) is coded as `294.0 × km^0.301` metres (0.294 D^0.301 km) against the "other form in the literature", 0.4 D^0.3. My arithmetic: the two differ by a constant factor of about 0.735 (0.294 / 0.4), so the coded law makes every complex crater about a quarter shallower than the alternative would (at 20 km: 0.72 km against 0.98 km). The agent's own doubt, not resolved in the repo.
3. **The ocean rule was fitted to an abstract.** Build log: "the ocean rule against the paper's abstract only"; scaling.rs: "[`sea_floor_factor`] is a linear fit to those three numbers in the ratio of water depth to impactor diameter (their own curves are not reproduced)"; build log: "a linear fit to Davison & Collins 2007's three numbers (the sea removes two thirds of craters under 1 km, a third at 30 km, hardly any over 100 km), not their curves". Rows IMP-065 to 067 are abstract-only. The fit also rests on two uncited numbers, "a 3.8-km ocean over 71 % of the planet" (IMP-068).
4. **Contradiction between the cited summary and the fit.** The module comment says Davison & Collins found the sea removes "hardly any [craters] over 100 km"; the `sea_floor_factor` doc comment says the fit gives "a tenth at 100 km".
5. **The rate: two statements that disagree.** The build log says "The rate was checked against the literature by a search"; the scaling.rs module comment says all the constants "were written from the papers as remembered". Rows IMP-001 to 003 are cited-unverified on the build log's word. The ± 2.8 in "(5.6 ± 2.8) × 10⁻¹⁵" is not used anywhere: the rate is a single number.
6. **Comment against code: depth at the transition.** scaling.rs comment on the complex depth: "at the transition it is continued from the simple crater's D / 5". The code takes `min(0.294 D^0.301 km, 0.2 D)`, which is the complex law almost everywhere, not a continuation. My arithmetic at the transition (3.2 km): simple D/5 = 640 m, complex law = 294 × 3.2^0.301 = about 417 m, so depth drops by about a third across the transition (at depth_scale 0.5, about 16 blocks to about 10 blocks). Suspicion that the drop is unintended.
7. **Small disagreements between comment, build log and tests.** A 1-km stony asteroid makes "a ~16-km crater" (scaling.rs header) against "~15 km" (build log; accepted range 12 to 20 km in the test). A simple crater is "about D / 5" deep (scaling.rs) against "D / 4.7" (build log).
8. **Wrong by a factor of a million (suspicion, arithmetic).** crates/deeptime-viewer/assets/editor_fields.json, `impact_flux` hint: "one crater of 20 km or more per 180 million km² per year". 1 / 5.6e-15 per km² per yr is about 1.8e14 km² (180 trillion), not 180 million. On all of Earth (about 510 million km²) the rate is about one per 350,000 years.
9. **Ni-Cu-PGE note against the coded rate and thresholds (suspicion, my arithmetic).** impact_ni_cu_pge.json notes: "a crater large enough (about 170 km and more) forms once in a few hundred Myr and must survive". With the coded rate, craters of 170 km and more number about 0.46 per 500 Myr on the 16k planet, about one per 1.1 Gyr. Using the coded melt formulas on Earth's land, my arithmetic gives a melt sheet of about 1.07 km at 170 km and about 0.5 km at about 107 km (full ore, 1,500 m, near 209 km), so the "170 km" in the note does not match the 500 m threshold in the same file (IMP-105, 110, 112). The coded 500 m threshold would be reached by about 107 km craters, expected about once per 500 Myr; the run found its only over-100-km crater in the sea floor.
10. **Deposit-size claims against the stage's own limits.** impact_hydrothermal_cu_zn_au.json says "craters from about 8 km", but the build log says "Impact deposits form only where the crater's breccia is in the strata log, so not in craters under two L1 cells", which is 9.6 km at 16k. Craters of 8 to 9.6 km make no hydrothermal ore on this preset.
11. **Peak-ring onset (suspicion, low confidence, from general recall only).** The coded onset is 40 km × 8.87 / g = 36 km on Earth, scaled from Venus. Terrestrial peak-ring craters are, as I recall, usually quoted from a larger size. Not checked.
12. **Speed (suspicion, low confidence).** The code cites Collins, Melosh & Marcus 2005 for "about 20 km/s on average". I recall the Earth Impact Effects Program's own asteroid default being nearer 17 km/s. Not checked; the citation is from memory in any case.
13. **Wording.** STATUS.md pending question: "Blocks are 15 times taller than wide". A block is one cube; the scale is 300 m of planet per block horizontally and 20 m vertically, so the world is stretched 15 times vertically. params.rs says "The world is 15 times taller than wide (20 m against 300 m per block)". The arithmetic in both is consistent (a real bowl is three times deeper than wide in blocks); only the phrase is loose.
14. **Dead code that looks like physics.** `scaling::ejecta_m` (McGetchin, Collins eq. 32) and `Crater.excavation_m` (a third of the transient depth, Melosh 1989) are not used by the stage; the drawn ejecta come from `Profile::at_rim_radii` (`h x⁻³`), which agrees with `ejecta_m` at the rim (the test checks 5 %). Both rows (IMP-053, 054) are from-memory.
15. **Complex craters sized by the simple factor over a range.** `final_m` takes the larger of 1.25 D_tc and the complex law, so for transient diameters from about 0.8 to about 1.66 times the transition the final size uses the simple factor while the morphology test (`d < dc` on the final diameter) already says complex. The comment calls it deliberate ("a crater just past the transition is still bowl-sized"). Observation only.
16. **Two melt thicknesses.** The strata melt layer is the physical sheet times `depth_scale`; the deposit field `impact_melt_m` is the unscaled sheet (its range is 0 to 3,000 m). The Ni-Cu-PGE thresholds (500 to 1,500 m) therefore read the physical value, while the rock in the game is half as thick at the default. Not wrong, but not stated in the type's note.
17. **Possible double counting of wear for craters cut into L1 (suspicion).** The surface stage erodes the carved L1 dimple, and the same erosion is also added to `worn_m` and shrinks `kept`, which scales the amplifier's residual. The build log says only that each crater "is worn by what the surface stage just did inside its rim". Not tested separately in the entry.
18. **The 2.1-km crater is 6 blocks deep in the terrain, not 10.** The build log's own arithmetic gives 11 blocks at `depth_scale` 0.5 for 2 km; the 5-Myr-old crater in the run is 6 deep. Fading by diffusion over 5 Myr (tent half-width about 775 m against a radius of 1,050 m, my arithmetic) is a plausible cause; the entry does not say.
19. **Rock-class values are judgments.** impact_breccia.json and impact_melt.json label density, erodibility and diffusivity "judgment" and name Sklar & Dietrich 2001, Stock & Montgomery 1999, Roering et al. 1999 and Heimsath et al. 1997 only as a framework; I marked them uncited rather than cited-unverified.
20. **Preset against Rust Default.** No difference: all eleven keys of earthlike_quick_16k.json `impacts` equal `ImpactParams::default()`; the build log says the section is in all eight presets (I checked only earthlike_quick_16k). The GameTest preset differs on purpose (flux 40, max_crater_km 40).
21. **Review tools.** `tools/review/p2-08-craters.py` and `p2-08-impacts.sh` compare against nothing external; there is no check of the modelled size distribution against the Earth Impact Database or any other crater list in the repo.
22. **File name.** crates/deeptime-core/data/usgs/impact_structures.json sits among the USGS grade-tonnage files but states "not a USGS source" and "design values"; its tonnage block is labelled "Not used".

## What a player sees

All figures from the build log and the code; sizes use the world scale of 300 m per block horizontally and 20 m per block vertically (design-decisions scale table), so 1 km is 3.3 blocks across and 100 m of height is 5 blocks. Block counts for sizes other than the build log's own are my arithmetic from the coded formulas at `depth_scale` 0.5 and are approximate.

**Where and how many.** On the quick 16k planet (16,384 blocks round, 8,192 by 16,384 blocks) at Earth's rate, a 500-Myr history made 567 craters, but only 71 remain and 19 show; 17 of those 19 are on continental crust and 7 hold a dry floor above the sea. A player may walk a long way without meeting one (the build log's own pending question says so). The Heavy slider (flux 10) or a higher flux gives more.

**Size.** The smallest crater is 2 km on Earth's land (6.7 blocks across; the sea can shrink one to about 1 km, 3.3 blocks). Craters up to 3.2 km across (10.7 blocks) are simple bowls; larger ones have a flat floor, terraced walls and a central peak 0.22 of the diameter across, and from about 36 km (120 blocks) a ring of peaks at half the rim's radius instead of the peak. The largest a run may make is 300 km (1,000 blocks); the run measured here made 112 km (373 blocks).

| Crater | Across in blocks | Depth in blocks at the default (my arithmetic) |
|---|---|---|
| 2 km, simple | 6.7 | about 10 (the build log says 11) |
| 3 km, simple | 10 | about 15 |
| 5 km, complex | 17 | about 12 |
| 10 km | 33 | about 15 |
| 20 km | 67 | about 18 |
| 36 km, peak ring begins | 120 | about 22 |
| 100 km | 333 | about 29 |
| 300 km | 1,000 | about 41 |

The rim stands about 3 to 4 percent of the diameter above the old ground at full depth scale and half that at the default (a 2-km crater: about 1 to 2 blocks). The ejecta blanket falls as the cube of distance out to three radii (a 2-km crater: 10 blocks from the centre), and is visible only near the rim.

**Rock.** A crater cut into the coarse terrain (at least two L1 cells, 9.6 km at 16k) has its target rock removed down to the base of the breccia lens; the lens is laid as `impact_breccia` (cobblestone until the MIC tags exist), a melt sheet of at least 10 m (real) is laid on the floor inside 0.55 of the radius as `impact_melt` (smooth basalt), and half the rise outside the rim is breccia ejecta. A smaller crater gets a breccia lens as a 3-D body (an ellipsoid under its floor, with a `melt_sheet` zone when it has one). The lens is about a tenth of a simple crater's diameter thick at full scale; a complex crater's lens is as thick as the crater is deep. In the GameTest planet crater 444 (2.1 km, 3 Myr) had breccia under its floor in 5 of 5 columns.

**Ore.** Four impact deposit types sit in or under craters: Zn-Cu-Au veins in breccia and melt rock (craters from about 8 km, but see flag 10), basin uranium in the fill of craters of 5 km and more (only after the oxygenated-oceans clock), diamonds in the melt rock of craters from about 30 km over gneiss, schist, black shale or coal, and Sudbury-type Ni-Cu-PGE under melt sheets of 500 m and more on continental crust. In the run that is 10 hydrothermal sites and 3 uranium sites that survive, 3 diamond deposits that all eroded, and no nickel; the pack's own diamond, nickel, platinum and uranium ores keep generating (`claims_scatter: false`). The four types carry `balance` size multipliers of 12 horizontally (6 for diamond) and 4 vertically; the files do not say what they are for (my reading: to enlarge bodies for play, as the build log's "design values" suggest).

**Fresh against old.** A fresh crater has a sharp rim, a deep bowl (a flat floor and central peak or ring for complex ones) and a thin blanket of breccia around it. Years of soil creep round and lower the rim and make the bowl shallower and wider; a 2-km bowl fades in about 10 Myr, a 20-km crater in about a billion years. Erosion strips the rim; deposition lays sediment over the floor, so a crater can hold a lake if the bake's water decides it does (the build log's 4.9-km, 40-Myr-old crater is 3.6 blocks deep with a small lake). A crater with 75 percent or more of its depth is "fresh", with at least 40 m (two blocks) "degraded"; below that it is "buried" under sediment or "eroded to its roots", with no bowl at all but its breccia still in the rock column (the `inspect-column --why` example: `impact_breccia` under later shale and mud). The oldest crater that still showed in the run was 288 Myr old. A crater worn through its depth and lens is gone; one on oceanic crust that subducts goes with the plate; a larger later crater destroys smaller ones within 0.8 of its radius. Large, young craters in mountain belts can be hard to read (the 73-km ring crater: "a third of its relief left and hard to read among the ranges and cones").

**What the game does not build from the list.** The Java reader (`Craters.java`) only lists craters from `impacts/craters.json`; the craters themselves are in the terrain heights, the strata and the bodies.

## The model in plain words

1. The impact stage (`ImpactStage`, built by `ImpactStage::new` from the preset's `impacts` section or `ImpactParams::default`) runs once per plate step, coupled to the plates and placed after the volcanoes (`coupled_to`).
2. How many impactors arrive: `impactors_per_myr` takes Earth's rate for craters of at least `min_crater_km` from `earth_rate_per_km2_yr` (5.6e-15 per km² per year at 20 km, times (D / 20 km) to the minus 1.8), multiplies it by `flux`, by the planet's own area from `planet_area_km2`, and by a million.
3. In `ImpactStage::advance` the number in a step is a Poisson draw (`rng.poisson`) with that mean times the step's length in Myr, from a random stream seeded by the preset seed and the step number.
4. Each impactor is given a size by `draw_diameter_m`: a draw from a power law between `min_crater_km` and `max_crater_km`, read as the crater the body would make on Earth's land.
5. It is given a position uniform over the sphere (latitude from the arcsine of a uniform number, longitude uniform) and a uniform moment within the step.
6. `impactor_of_earth_crater_m` turns the Earth-land crater into the impactor's diameter by inverting `transient_of_final_m` and `Setting::impactor_m` at Earth's gravity (9.81 m/s²), with speed 20 km/s, density 3,000 kg/m³ and a 45-degree angle.
7. Water depth at the point is the sea level minus the elevation (plus this step's craters); `crater` passes it to `sea_floor_factor`, which keeps 1 − (depth / impactor diameter) / 6.3 of the crater and gives no crater at all once water is deeper than 6.3 impactor widths; a stopped body is counted, and one that would have made 20 km on land is kept for an `impact_ocean` event.
8. Otherwise `Setting::transient_m` gives the transient diameter at this planet's gravity, times the sea factor, and `Setting::final_m` takes the larger of the simple (1.25 times) and complex diameters.
9. The crater is simple if its final diameter is under `transition_m` (3.2 km times 9.81 over gravity), a complex crater with a central peak above it, and a peak-ring crater above `peak_ring_m` (40 km times 8.87 over gravity).
10. `crater` computes the rim height (0.07 D_tc⁴ / D³), the breccia lens, and the depth: for a simple crater the transient depth plus the rim less the lens, for a complex one 0.294 D^0.301 km capped at a fifth of D.
11. It also computes the melt volume from the impact energy (`melt_m3`) and spreads it over the transient crater as a sheet that counts only at 10 m or more.
12. A crater narrower than half the minimum size is dropped; `build` then multiplies depth, rim, peak, breccia and melt by `depth_scale` to give the `Profile` in game metres, and the crater is added to the catalogue (`ImpactState::craters`).
13. Any smaller crater whose centre lies within 0.8 of the new crater's radius is removed as overprinted.
14. A crater at least two L1 cells across is "imprinted": its profile is added to the deep-time surface's elevation and offset (`cell_weight`), the target rock is removed in the strata log down to the base of the lens, the breccia lens is laid (thinning toward the rim), melt goes on the floor inside 0.55 of the radius, and half the rise outside the rim is laid as breccia ejecta.
15. At the start of each later step the craters turn with their plate's rotation; a crater whose plate changes while it is on oceanic crust and the new cell is not continental is subducted and leaves the catalogue.
16. Degradation reads the surface stage's net erosion or deposition averaged inside the rim (`mean_inside`): deposition adds to the cover over the floor, erosion strips the cover first and then adds to the crater's wear, and a diffusion total Σ κ t grows by 0.01 m²/yr times the surface rock's diffusivity times the step, wherever the ground is above 60 m under the sea.
17. `Crater::state` works out the relief left (the share kept, 1 − (cover + worn) / depth, times the relief of the profile blurred by a tent of half-width √(12 κ t) from `Profile::aged`) and names the fate: fresh at 75 percent of depth or more, degraded at 40 m or more, buried if sediment covers it, otherwise eroded to its roots; a crater worn through its depth and lens is removed as eroded.
18. Every step the stage writes the map layer `impacts/craters` (`map`) and the closure layer `impacts/trap` (`trap_field`, 0.7 inside the rim of surviving complex craters) that the structure stage folds into the oil traps.
19. Whenever the surface closes a frame of the recorded history, `frame_fields` adds the four deposit fields (melt thickness, hydrothermal strength (D − 4 km) / 36, shock D / 100 km, and the basin flag) for the craters formed since the last frame, which the four impact deposit types read.
20. On the last step `finish` writes the events (`crater_event`, for craters of 5 km or more and smaller ones that still show, and for the stopped ocean impacts) in time order, and `to_body` turns the lens and melt of each crater too small for L1 into a body in the world.
21. The bake writes the standing craters to `impacts/craters.json` (`present_craters`), which Java's `Craters.read` loads only as a list.
22. In the amplifier, `Shaper::from_world` takes each crater that still has relief (`kept` above zero) and builds a table with `residual`: the aged profile times the share kept, less, for an imprinted crater, the same profile blurred by 1.5 L1 cells so that only what L1 cannot hold is added; tables whose largest change is under 2 m are dropped.
23. `Sink` adds those tables to every tile of the last grid, after the last flood, so whether a bowl holds a lake is decided by the bake's own water.
24. At the end `note` and `summary_line` record the counts by size class, what became of the craters, how many show and the largest and oldest that do.
