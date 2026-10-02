# Extraction notes: volcanoes and magma

Written by the stage-1 extraction agent for this subsystem (2026-10-01/02) while it built the ledger. It holds the measured Earth-against-mod numbers found in the build log, the simplifications and game choices the code or documents state, and flags: admissions of unchecked values, contradictions between comments, code and documents, and things that looked wrong. **Nothing here is verified.** A line marked "suspicion" or "my arithmetic" is the extraction agent's own reading, to be checked in stage 2. See [claims.md](../claims.md).
Scope: `crates/deeptime-core/src/surface/volcanoes/{mod,params,shape}.rs`, `crates/deeptime-core/src/surface/amplify/volcano.rs`, `crates/deeptime-core/src/underground.rs` (the bake's export and the lava settings), the Java `worlddata/Volcanoes.java` and `worldgen/{Magma,MagmaPass,MagmaRules,LavaFeatureRule,LavaFeatures,LavaFluids,LavaRules}.java`. Ledgers: `ledger/VOL.md` (73 rows), `ledger/MAG.md` (42 rows). World scale in all conversions: 300 m per block across, 20 m per block up (one block of height is 20 m; 1 km across is 3.3 blocks, 1 km up is 50 blocks). Block counts below that the build log did not state are my own division and are marked "derived".

`crates/deeptime-format/src/sites.rs` holds no volcano or lava constants (it is the site-record file format: magic numbers, record sizes); `crates/deeptime-cli/src/volcano_view.rs` holds no physical constants (it prints numbers and uses 'window' sizes for the view). Neither produced rows.

## What a player sees

All sizes use the defaults (`flank_steepness` 0.5, `size` 1, `density` 1). Where a number is the build log's it says so; the rest are the code's formulas worked out (derived).

**Arc volcanoes (stratovolcanoes).** They stand in curved lines along the volcanic front of an arc, at least about 60 km (200 blocks) apart (spacing times a random 0.8 to 1.2), on the overriding side of a trench, both on continents (about 61 % of arcs in one 1,000-Myr run, run-review-1a0f969d358.md) and as island chains in the sea (39 %). A cone is 1.2 to 3 km tall (60 to 150 blocks) and, at the default gentle flank setting, 13 to 28 km in basal radius (45 to 93 blocks; the build log says a 3-km cone is "26-28 km in radius"). The flanks are concave, steepest at the summit. A summit crater is at least 1 km in radius (2 km across, about 7 blocks) and about 6 % of the cone's height deep (a 3-km cone: about 180 m, 9 blocks). The mountain pass leaves young cones smooth, so a young cone reads as a clean cone with radial gullies; a dead one slowly regains rugged ground and loses its crater with a 5-Myr time constant, its cone shape three times slower. Cones sit on ground that the plate model has raised (a continental arc cone stands on a median 812 m of ground after P1-27; the cone itself is 1.2-3 km above that), and no summit passes 6.8 km above the sea (340 blocks).

**Shield volcanoes and island chains.** Over a mantle hotspot on ocean crust a broad, convex basalt shield rises 3.5 to 6.5 km above the sea floor (175 to 325 blocks) with a basal radius of 49 to 80 km (163 to 267 blocks; derived from the code, the build log writes "up to 71 km"). A shield's summit may not pass 4.4 km above the sea (221 blocks), so most hotspot volcanoes are seamounts or small islands: the build log's examples are an island 418 m high on an island-arc cone and a 756-m island (38 blocks, "a 38-block island in a 70-km body") on the youngest shield of the chain with most islands. A new shield forms about every 70 km (233 blocks) of drift, so a chain of volcanoes is strung down-track, the newest over the plume, older ones eroded and sinking at 25 m/Myr (up to 1.5 km). In the run-review run only 3 islands and 8 seamounts were recorded standing, so Hawaii-like chains are rare.

**Continental hotspot centres and calderas.** On continents a hotspot makes a very low, broad silicic centre: 0.9 km tall (45 blocks) over a basal radius of 70 km (233 blocks, derived). Silicic cones and centres collapse into calderas: a flat-floored depression with steep walls and a slightly wavy rim, 6 to 18 km across (20 to 60 blocks; the radius is drawn between 3 and 9 km and held under 0.6 of the cone's radius), about 400 to 850 m deep (20 to 42 blocks; the cap is 400 m plus 5 % of the radius, and half the cone's height when the cone is short). The build log's example is a caldera 14.6 km across (about 49 blocks) with a flat floor 764 m (38 blocks) deep. Around it an ignimbrite (tuff) sheet, up to 250 m (12 blocks) thick at the rim, thins to nothing at four rim radii (so a 10-km-wide caldera has a skirt of tuff out to about 20 km from its centre). A big shield (over 4 km) opens a smaller summit caldera of 8 % of its radius.

**Flood basalts.** Each large igneous province becomes a flat basalt plateau. At 16k the province radius is the plate model's 900 km scaled to the planet (about 110 km, 370 blocks, derived), the pile is 1,500 m thick (75 blocks) at the centre, built of 8 basalt flows each capped by a thin tuff bed. Where the pile crops out within 150 m of the surface the terrain is cut into steps: flat treads and steep risers, 80 m (4 blocks) per step, treads taking 70 % of each step. A run of the run-review (1,000 Myr) has 17 such provinces.

**Rock.** Arc cones are andesite over tuff (on thick continental crust, rhyolite over tuff); shields are basalt; calderas are surrounded and filled by tuff; flood-basalt piles are basalt and tuff. The strata log buries the old soil under new lava as a palaeosol. (The build log notes no lava or obsidian blocks, and no volcanic ore, in the P1-21 version; lava blocks came with the magma work below.)

**Magma, conduits and crater lakes (underground, only at volcanoes active at the present).** Under each active volcano there is a hidden chamber of lava, an ellipsoid whose top is 3 km (150 blocks) below the crater floor, with radius a tenth of the volcano's base radius (1.2 to 4.5 km). Because the world is 15 times taller than wide the chamber is a tall lens: 19 blocks across and 57 high (9,500 lava blocks) under a 28-km arc cone, 31 across and 91 high (42,400 blocks) under a shield (both from the build log). A plus-shaped column of lava, a few blocks wide, rises from the chamber's centre to six blocks under the crater floor; above it a plug of magma blocks reaches the surface in the crater on land (buried 3 blocks under the bed under water). At the most active quarter of the volcanoes standing on dry land there is a crater lava lake instead of the plug: radius 2 to 10 blocks (half the crater's radius), up to 10 blocks deep, its surface one block under the lowest ground around it, with the ground above it opened to the sky so the player can see it. A lake sits in a pit cut beside the real peak, because baked summits are rarely bowls. On the build log's planet, 100 of 123 active volcanoes stand on land and 16 hold a crater lake (about 62 could). Everything lava touches is walled in by smooth basalt within a block or two, so lava never meets the sea, a lake, a river, an aquifer or a cave. A volcano under the sea or at a coast keeps its chamber and conduit (at least 8 blocks under the ground) but no lake. Dead volcanoes have nothing.

**Lava elsewhere.** The biomes' random lava lakes and springs are removed in Deep Time levels (vanilla's four and Terralith's four, plus Terralith's lava remover). The only other lava is the lava floor at the bottom of the world: open space below y -342 (floor y -352 plus 10 blocks) is lava, and deep cave lakes below about y -288 may be lava. Under a sea bed that reaches the floor the lava stays 8 blocks under the bed. Spreading ridges and rifts, where real lava is common, have none. From a random land point on the build log's planet the nearest active volcano on land (with the plug in its crater) is a median 658 blocks away (p10 169, p90 1,309), the nearest open crater lava lake a median 1,178 blocks (p10 380, p90 2,251), and the lava floor 438 blocks straight down (p10 411, p90 510). The build log says vanilla has lava within about 100 blocks of anywhere.

## The model in plain words

1. The volcano stage `VolcanoStage::advance` runs once per plate step after the surface stage; it keeps a catalogue of discrete volcanoes that ride their plates.
2. Each step the stage first turns every volcano with its plate's rotation (the `turn` closure) and removes oceanic volcanoes whose crust has been carried under another plate (`Gone::Subducted`).
3. The arc flux of each cell comes from `arc_flux`: zero unless the plate model flagged the cell as arc, otherwise convergence speed over 80 mm/yr (capped at 1), times 0.7 on oceanic crust.
4. `volcanic_front` reduces the arc flux band to its centre line (and drops crust more than 4 km below the sea), because real arcs line their cones along one front.
5. `place_arcs` then takes the cells on that front with flux of at least 0.25, highest first, and gives each a new stratovolcano if no live, just-dead or still-standing cone lies within 60 km times a random 0.8 to 1.2.
6. A new cone's target height is a straight-line function of the flux between 1.2 and 3 km, its radius follows height to the power 0.8 (14 km at 3 km) and is divided by the flank steepness, and it lives a random 2 to 8 Myr.
7. If the crust is continental and at least 42 km thick the cone is silicic (rhyolite-bearing, caldera-prone); every new cone starts with a small crater, 6 % of its height deep.
8. Hotspot volcanoes are born at each live plume once the chain's last volcano has drifted 70 km away: a shield on ocean crust with a height set by the plume's strength, a silicic centre on continental crust.
9. A hotspot volcano stays active while it lies within 70 km of its plume and builds more the closer it is (target times the square root of one minus the squared distance fraction).
10. Growth is `build`: each step the volcano gains (target minus height) times (1 minus e to the minus step over 1 Myr), but only while every cell it stands on is below the ceiling of 6.8 km above sea (0.65 of that for shields), scaled with the planet's crust strength height.
11. The new relief is added to the surface's elevation and to the plate feedback, and its rock goes to the strata log as tuff and andesite (arc), basalt (shield), or tuff and rhyolite (silicic), burying the old soil.
12. Each following step the surface stage erodes the cone like any land; the stage fits that change over the cone's footprint (`fit_change`) and lowers the cone's kept height by it, so the volcano is only as tall as the surface shows.
13. A stratovolcano goes dormant when its planned life ends, or when the arc flux near it has been below 0.25 for two steps; one step without flux is forgiven, and steps in which the plate model cleared its arc flags (the `flux_valid` test) are skipped.
14. A dead cone is retired from the catalogue when it has been dead over 5 Myr and worn below a tenth of its peak, or after 250 Myr in any case (the 250 Myr rule is not in the module header).
15. Dead oceanic volcanoes also subside at 25 m/Myr up to 1.5 km, a smooth kernel 1.6 radii wide, standing for flexure and cooling of the sea floor.
16. A silicic cone erupts a caldera at 0.04 per Myr (a continental hotspot centre at 0.12, a shield over 4 km at 0.04 as a small summit one), with a radius drawn between 3 and 9 km and depth capped at 400 m plus 5 % of the radius.
17. The caldera removes the summit and lays half of the erupted volume as an ignimbrite sheet out to four rim radii and half as a tuff fill; the volcano then dies.
18. Each new `large_igneous_province` event of the plate model becomes a flood-basalt province: 8 flows, each 88 % basalt capped by 12 % tuff, with the plate model's dome flattened to a plateau with a steep edge.
19. At the end of the run `finish` writes the flood-basalt outcrop layer and the events; the bake's `active_volcanoes` in `underground.rs` then lists only the volcanoes still active, with kind, radius, height, activity (the last arc flux, or the mean nearness to the plume) and crater.
20. The amplifier's `Shaper::from_world` replaces the blurred bump the young landscape holds with the true profile on L2 and adds the sharp summit, crater and caldera floor on the finest grid after the last flood, fading them for dead volcanoes.
21. In Java, `Magma.build` reads that list, the terrain height and the settings and gives each active volcano a chamber whose top is 3 km under the crater floor, radius a tenth of its basal radius (clamped 1.2 to 4.5 km) and half-height a fifth of that.
22. The chamber is raised to stay 8 blocks over the lava floor and squeezed to stay 24 blocks under the crater floor; if under 3 blocks of half-height remain, the volcano gets no magma.
23. The conduit is a plus-shaped column from the chamber's centre to six blocks under the crater floor, capped by a plug of magma blocks (3 blocks under the bed if the volcano is at sea or by water).
24. `Magma.build` then ranks the volcanoes that can hold a lake (dry land, crater ground not too sharp, away from the pole cap) by activity, and the top quarter (rounded up) get a crater lava lake whose radius is half the crater's, between 2 and 10 blocks.
25. `MagmaPass.apply` places the blocks in the surface step, never within 8 blocks of the ground for the chamber, turns open space around every body into smooth basalt, and masks the body from vanilla's carvers.
26. Lava elsewhere is controlled by `LavaRules` and `LavaFluids`: vanilla's two lava heights, measured from the world's floor instead of from y 0, and `LavaFeatures.removed` refuses any placed feature whose configuration names lava.

## Earth against the mod

Earth figures here are only those the repo writes down. Where the build log gives none, the table says "no source given". All numbers below are quoted from the three requested build-log entries unless the row says otherwise.

### P1-21 volcanoes as landforms (STATUS.md, 2026-09-30)

Run: `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --checkpoints`, the test machine (load 60-110), 2026-09-30; seed not stated (preset default). Branch an agent branch.

| Quantity | Earth figure (stated source) | Mod's measured figure | Run |
|---|---|---|---|
| Wall time, peak memory | no source given | 418 s, peak 3.6 GiB (unchanged); volcanoes 14.1 s (births 6.0, erosion accounting 6.0); amplify's volcano sink 0.6 s | first run |
| Volcanoes in catalogue at end | no source given | `volcanoes_final=250` | first run |
| Stratovolcanoes born | no source given | `strato_born=24017` (about 24,000 per run; eroded ones are counted, not written as events) | first run |
| Shields born | no source given | `shield_born=181` | first run |
| Caldera complexes born | no source given | `caldera_complex_born=91` | first run |
| Standing (not eroded away) | no source given | `standing_strato=1275`, `standing_shield=8`, `standing_caldera_complex=5` | first run |
| Active at the end | no source given | `active_strato=203`, `active_shield=2` | first run |
| Calderas | no source given | `calderas=295`, `summit_calderas=27` | first run |
| Flood basalts | no source given | `flood_basalts=9` | first run |
| Tallest volcano | no source given | `tallest_m=4643` | first run |
| Flood-basalt outcrop cells | no source given | `flood_basalt_outcrop_cells=389` | first run |
| Volcano events | no source given | `events=1468` of 3,498 events | first run |
| Share of the planet the plate model flags as arc | no source given | about 12 % of the planet at 100k samples (bands a few samples wide plus broad hatched patches where intra-oceanic arcs overlap) | first run |
| Relief the young landscape keeps of an edifice | no source given | 50 to 70 % of the nominal relief at L1, less after the young stage | first run |
| Highest live continental-arc cone | no source given | relief 2.1 km (young L2), 3.1 km (L3), summit 3.6 km a.s.l. | `deeptime volcano-view`, first run |
| An island-arc cone | no source given | 2.5 km then 3.2 km relief, an island 418 m high | volcano-view, first run |
| Youngest shield of the chain with most islands | no source given | 4.8 km then 5.0 km relief (a 756 m island), with two older islands and islets down-chain | volcano-view, first run |
| A 2-Myr caldera | no source given | 14.6 km across, flat floor 764 m deep | volcano-view, first run |
| Fuji profile | code comment: "a third of its relief at 0.55 of its radius" (no source) | (1 - 0.55)^1.5 = 0.30 by the code's own formula (derived) | shape.rs comment |
| A pit on an Andean plateau | no source given | about 5 km deep pit in the young landscape (run q16c), not at a volcano, not investigated | run q16c |
| Cave lava under deep ocean | no source given | 4.5 M lava blocks in a 256 x 256 box of sea floor, below y -54 (vanilla's lava level, not the volcanoes') | 3D previews on the first run's bake |

**Follow-up 1 (flank steepness 0.5; decisions log 2026-09-30).** Preset `earthlike_quick_16k`, 500 Myr, run q16g.

| Quantity | Earth figure | Mod's figure | Run |
|---|---|---|---|
| A 3-km arc cone | no source given | radius 26-28 km (the build log: "56 blocks of relief per 90 blocks of run instead of per 45") | q16g |
| Shields | no source given | "up to 71 km" in radius | q16g |
| Volcano events | no source given | 611 of 2,931 events (before grouping 1,468 of 3,498): `volcano_arc` 341, `volcano_hotspot_chain` 20, `volcano_stratovolcano` 73, `volcano_shield` 4, `volcano_caldera_complex` 3, `volcano_caldera` 162, `volcano_flood_basalt` 8 | q16g |
| Run time | no source given | 420 s wall on the loaded Mac, volcanoes 22.7 s (erosion fit 13.5 s), amplify's volcano sink 1.1 s | q16g |
| Live and standing | no source given | 282 live arc cones, 1 live shield, 137 calderas, 8 flood basalts | q16g |

**Follow-up 2 (small segments counted; smooth young cones).** Same command, `(a scratch folder on the test machine)`, Mac at load about 80, 2026-09-30.

| Quantity | Earth figure | Mod's figure | Run |
|---|---|---|---|
| Wall time, peak memory, volcano stage | no source given | 1,269 s, peak 4.0 GiB, volcanoes 31.0 s | p1-21 after |
| Volcano events | no source given | 585 of 2,499: `volcano_arc` 296, `volcano_caldera` 164, `volcano_stratovolcano` 87, `volcano_hotspot_chain` 21, `volcano_shield` 8, `volcano_caldera_complex` 2, `volcano_flood_basalt` 7 | p1-21 after |
| Small arc segments (fewer than 3 cones) | no source given | 15 segments, 20 cones counted on their onsets; arc events fall by 5 % | p1-21 after |
| Active continental-arc cone 19773 (2,976 m, x 371 z -2765) | no source given | summit y 452 to 442 with the calm mask (the sharpened spike gone); mean abs Laplacian on its land 0.656 to 0.613 blocks; mean abs change 0.16 blocks | A/B (`deeptime stage amplify --in 6-young.dtcp --bake`, mask line removed = before) |
| Active shield 19366 (4,673 m, x -3668 z 658; a 38-block island in a 70-km body) | no source given | mean abs Laplacian 0.919 to 0.826; highest ground in its body (110 blocks off the vent) y 415 to 380; mean abs change 0.74 blocks | same A/B |
| Whole planet's character change | no source given | 1.86 to 1.78 blocks mean | same A/B |
| Tallest young shield or caldera complex above the sea | no source given | a 38-block island (of 8 standing young shields and caldera complexes) | same run |

### Magma under active volcanoes (STATUS.md, 2026-10-01)

Run: a fresh `earthlike_quick_16k` planet, supercontinent start, 500 Myr, baked by branch an agent branch; previews `tools/workbench/preview.py run`, seed 20261001, three 256 x 256 x 992 boxes per profile; probe `/deeptime probe magma` with 4,000 land points drawn evenly inside the pole cap. Earth figure for lava: no source given.

| Quantity | Mod's measured figure |
|---|---|
| Active volcanoes | 123 (119 arc stratovolcanoes, 3 caldera complexes, 1 shield); 100 on land; 16 hold a crater lake (about 62 of the 100 can hold one) |
| Chamber under a 28-km arc cone | 19 blocks across, 57 high, 9,500 blocks |
| Chamber under the shield | 31 across, 91 high, 42,400 blocks |
| Box: arc stratovolcano 12529 (crater lake, summit y 190) | MIC 10,300 and standalone 10,300 lava blocks; one body of 10,239 from y -16 to 182; 29 faces open to the sky; 61 blocks at the lava floor |
| Box: shield 12523 (summit 81 blocks under the sea) | MIC 43,188; standalone 45,677; one body of 43,051 from y -258 to -24; 0 faces on water |
| Box: oceanic arc volcano 12565 (summit y -91, under frostfire caves in MIC) | MIC 5,709; standalone 6,747; one body of 5,181 from y -285 to -97; 0 faces on water |
| Lava above y -288 not a volcano's | 0 blocks in all six boxes; lava touching water: 0 faces |
| Distance from a random land point to the nearest active volcano on land | p10 169, median 658, p90 1,309 blocks |
| ... to the nearest open crater lava lake | p10 380, median 1,178, p90 2,251 blocks |
| ... straight down to the lava floor (y -342) | p10 411, median 438, p90 510 blocks |
| Vanilla comparison | "Vanilla has lava within about 100 blocks of anywhere" (no source or measurement given) |
| Terralith lava remover | ate 4,226 of 4,521 blocks of a submarine chamber in MIC (standalone kept all); 300 blobs of radius 10 to 12 per chunk below y 30 in frostfire caves |
| GameTest `magmaUnderActiveVolcanoes` | three shields of the test planet read: 43,075 / 42,881 / 43,051 molten blocks, `stray=0 wetFaces=0 airFaces=0`; with MIC `lavaFeaturesStayOut` refused 14 of 932 placed features; lake volcano: 29 lava faces on air (the lake surface) after the carving-mask change, 41 more before |
| Also seen | at lake volcanoes the ground 3 to 5 blocks from the listed summit lies 1 to 11 blocks below it; 13 of the first 20 lakes stood within 160 blocks of the pole edge and 2 beyond the cap |

### Lava floods (STATUS.md, 2026-10-01)

Ben's report, from a workbench 3D preview of run `1a0f6e918c9` (`earthlike_quick_16k`, MIC profile) around x -6901, z -128 (the build log's run id; see Flags). Earth figure: none (a bug measurement against vanilla's hard-coded levels, no source given beyond the Minecraft class names).

| Quantity | Before | After |
|---|---|---|
| Coast box (256 x 256) lava blocks | 2,152,868 (51,040 of 65,536 columns; 0 cave air below y -54; 337 lava blocks above it; 4,020 sea columns standing on lava down to y -267; 5,810 lava-water faces) | 8,267 (8,163 below y -300, the lava floor); 0 sea columns on lava; 0 lava-water faces (another seed) |
| Mountain box (x -7801..-7546, z -209..46) | 311,110 lava blocks, 100 % of the open space below y -54 | 568 lava blocks (60 at the floor, the rest feature lakes) |
| Same seed 20261001, old levels (`lava_floor_blocks` 298, `lava_lake_blocks` 342) | coast 732,008; mountain 930,637 | default MIC: coast 395, mountain 1,421; standalone: coast 3,719, mountain 3,249 |
| Lava by depth after the fix | - | nothing between y -250 and y -54 except single-block springs; lava-lake aquifers at y -324..-307 (standalone: 3,586 and 1,017 blocks); floor at y <= -343; above y -54 only lava-lake features of 80 to 140 blocks each: 3 in the coast box (y -24, 25, 35, under the sea bed), 7 to 9 in the mountain box (y 51 to 156) |
| Vanilla heights | lava below y -54 and lake lava at y -10 or lower for a floor at y -64 (floor + 10 and floor + 54) | Deep Time: floor y -352 gives y -342 and about y -298 |
| Flooded fraction claim | "the lowest 298 blocks of every cave, and every sea deeper than 117 blocks" | - |
| GameTest `lavaStaysAtTheFloor` (level `deeptime:test_planet_caves`) | with vanilla's picker put back: FAIL, `lava=256029 topLavaY=-55 lavaAboveLakes=238767 seaOnLava=418` | with the fix: `lava=0 lavaAboveLakes=0 seaOnLava=0 openDeep=42275` (4,096 columns, 2,048 over a sea bed below y -60); MIC `openDeep=78373` |
| Also seen | test planet deepest sea bed at y -399 (y -389 after merge), below the world's floor y -352; old worlds had a few hundred thousand blocks of plain deepslate and blackstone per box (crust 296,087 to 12,819 in the coast box), not investigated | - |

### Other volcano numbers in the same build log and research notes (not in the three requested entries)

| Quantity | Earth figure (stated source) | Mod's measured figure | Run |
|---|---|---|---|
| Arcs and cone heights | no source given | 713 arcs, 61 % continental and 39 % island arcs; cone heights 1.2-3.0 km | run-review-1a0f969d358.md section 6: `earthlike_quick_16k`, supercontinent, 1000 Myr, 2-Myr steps, seed 1, 2026-10-01 |
| Hotspot tracks | Hawaii-Emperor "about 6,000 km over about 80 Myr (from memory)" | 35 tracks, median 4,750 km (Earth-equivalent) over 78 Myr (63 mm/yr); 124 of about 370 built volcanoes subducted; only 3 islands and 8 seamounts standing | same run |
| Flood basalts | "Earth averages one per 20-30 Myr, so 33-50" (no source given) | 17 in 1000 Myr, each 1,500 m thick, 2.0-2.7 million km2 (Earth-equivalent), "the size of the Deccan or Siberian Traps" | same run |
| Volcano density effect on land (P1-17 hypsometry, 2026-10-01) | Earth (GMT earth_relief 30', Tozer et al. 2019): land 0.289, median 427 m, above 2 km 0.102 | master: land 0.381, median 1,211 m, above 2 km 0.307; `volcanoes.density` 0: 0.316, 766 m, 0.170; density 0.5 (with other changes): 0.233, 582 m, 0.070 | VM, `earthlike_quick_16k`, 500 Myr, 4 runs, `tools/review/p1-17-hyps.sh` |
| Cones' share of eroded rock | no source given | about 2x10^7 km3 of cones built per run, a fifth of everything eroded | same entry |
| Ground under continental arc cones (P1-27) | no source given | 307 to 812 m median; cones on ground below 500 m 62 % to 41 % | Mac, 6 pipeline runs x 400 Myr |
| Summit ceiling with gravity (height limit stage 2) | no source given | 6.8 km at 1 g and 140 MPa; 17.9 km at 0.38 g; 3.4 km at 2 g; at 2 g the highest ground is the volcanoes at their 3.4 km ceiling; at 0.38 g summits of 9.5-11 km are volcanoes under their higher ceiling | 500 Myr runs on `earthlike_quick_16k` |
| Test planet maxima after the cone-ratchet fix | no source given | max 20.1, 7.5, 16.2, 2.5, 3.3, 4.6 km before to 4.4, 4.2, 4.5, 4.4, 3.0, 3.7 km after (the 4.4 km are shields at their ceiling) | `planet_world_4k`, 150 Myr, seeds 11-16 |

## Simplifications and game choices

From the code and docs, with the reason they give.

- **Three volcano kinds only.** No cinder cones, no spreading-ridge or rift volcanoes, no submarine fissure eruptions: "Spreading ridges and rifts are not exported: they are plate-boundary geometry, not cheap, and nothing draws them yet" (build log 2026-10-01). Scope was Ben's choice (decisions log 2026-09-29).
- **Stratovolcano lifetimes of 2 to 8 Myr** against a real cone's 0.1 to 1 Myr: "a centre here stands for a volcanic complex that rebuilds itself in one place over a 2-Myr step" (params.rs; build log P1-21 Deviations).
- **Stratovolcanoes that erode away are counted, not written as events** (about 24,000 per run) (Deviations).
- **Flank steepness 0.5**: radius doubled so cones read as volcanoes in a world 15 times taller than wide; Ben's "Setting, gentler default" (decisions log 2026-09-30). At 1 the profile is physical.
- **Crater is exaggerated**: radius at least 1 km "a few blocks across at 300 m per block" (code comment) and 6 % of the height, to be visible at block scale.
- **No guyot planing** beyond the surface's wave-base diffusion (Deviations; P2-11).
- **Lengths are block-metric kilometres, not Earth-normalised** (params.rs header), so slopes do not shrink with the planet; flood-basalt provinces, by contrast, take the plate model's Earth-normalised radius scaled to the planet.
- **Mantle activity as a number**: an arc volcano's "activity" is its last arc magma flux, a hotspot volcano's mean nearness to its plume; only a stand-in for eruption rate. Eruption rate, erupted volume and magma flux are not modelled; the only volume figures are the cone's relief built (`built_km3`) and the caldera's removed volume.
- **Magma chambers are a fixed shape**: an ellipsoid with radius a tenth of the cone's, half-height a fifth of that, 3 km deep, regardless of the volcano's age or activity; no sills, dykes or storage depth variation (the module code has no such rules).
- **Crater lava lakes are 25 % of the able volcanoes** though real persistent lava lakes are rare ("about eight of some 1,400"): "a player should be able to find one" (underground.rs).
- **Lava is sealed**: every body is walled by smooth basalt and masked from carvers so lava never touches water (a design rule after Ben's 2026-10-01 bug report).
- **The lava floor at the bottom of the world** is "a game convention" (STATUS pending question and decisions log 2026-10-01: "Whether the lava floor at the bottom of the world stays is not decided yet (it stays, as a setting, meanwhile)").
- **All random lava is removed** (`remove_lava_features` on), "geologically arbitrary" in the build log's words; the open question is how far a player must travel to lava.
- **No fluid tick on placed lava**: it rests until a player opens it.

## Flags

Quotations are from the files named. "Suspicion" means my own reading of the code, not a verified fact.

1. **Flank slope in the comment does not match the formula (suspicion, arithmetic).** `shape.rs` module comment: "stratovolcano: (1 - rho)^1.5 ... upper flanks ~20 degrees at 2.5 km over R = 14 km". For that profile the steepest slope is at the summit, 1.5 x H / R = 1.5 x 2500 / 14000 = 0.27, about 15 degrees, not 20. (Ledger VOL-016.)
2. **Shield slope claim holds only at the default flank setting (suspicion).** `shape.rs`: "shield: (1 - rho^2)^1.5, broad and convex with flanks under ~10 degrees". The steepest slope of that profile is 1.5 x H / R; for the tallest shield (6.5 km, 40 km) that is about 14 degrees, under 10 only after the radius is doubled by `flank_steepness` 0.5 (about 7 degrees). (VOL-017.)
3. **Island subsidence rate vs its comment.** `params.rs`: "Hawaiian islands sink ~2 km over ~30 Myr beyond the swell the plate model makes" but the coded rate is 25 m/Myr (0.75 km over 30 Myr), cap 1,500 m. The "beyond the swell" wording may explain the gap, but nothing in the code says how the two add up. (VOL-048.)
4. **Caldera comment gives widths, the field is a radius.** `params.rs`: "Caldera radius, km [min, max] (Crater Lake 4-5, Long Valley 8-16, Yellowstone 30-45 across)". Default [3, 9] km radius means 6 to 18 km across: Crater Lake (4-5 km across) is below the minimum and Yellowstone (30-45 across) well above the maximum; the radius is also capped at 0.6 of the cone's radius. (VOL-029.)
5. **Crater size contradicts its comment.** `volcanoes/mod.rs` (arc births): "A crater a few blocks across at 300 m per block (real ones are 0.3-1 km)" but the code gives radius `max(1000 x size, 0.07 R)`: at least 1 km radius (2 km across, about 7 blocks), and 0.07 x 28 km = 2 km radius (4 km across) for a 3-km cone, both at or above the upper end of the quoted real range. (VOL-026.)
6. **Retirement rule not in the header.** `volcanoes/mod.rs` header step 8: "A dead volcano worn to a tenth of its peak for 5 Myr leaves the catalogue"; the code also retires any volcano dead more than 250 Myr (`dead_for > 250.0`), and the 5 Myr is a literal, not tied to `fade_myr`. The build log repeats the header's version. (VOL-049.)
7. **Ceiling documentation omits the shield factor and the gravity scaling.** `params.rs` `max_elevation_km`: "No summit rises above this elevation (above the sea of the time), km (Ojos del Salado 6.9 km; Mauna Kea 4.2)". The code multiplies by 0.65 for shields (4.42 km) and by `strength_scale` (the planet's crust strength height over Earth's). The module header says only "capped so no summit passes `max_elevation_km` above the sea". (VOL-032.)
8. **Build log "up to 71 km" vs code.** STATUS 2026-09-30 follow-up: "shields up to 71 km". At the defaults the code gives 40 km / 0.5 = 80 km for the tallest shield (6.5 km), 49 km for the shortest. 71 km may be the largest actually born; the entry does not say. (VOL-022.)
9. **Build log "56 blocks of relief per 90 blocks of run" is unclear.** A 3-km cone is 150 blocks of relief over 26-28 km (87-93 blocks) of radius; 56 over 90 does not match any quantity named. The mean slope of a mid-flank section might be meant; the entry does not say. (VOL-024.)
10. **Java comment: "7 across and 40 high for a 2-km chamber".** `Magma.java` class comment. For a 2-km radius, rh = 2000 / 300 = 7 blocks is the radius (14 to 15 across); rv = 0.2 x 2000 / 20 = 20 is the half-height (40 high). "7 across" is the radius. The build log's "19 across and 57 high under a 28-km cone" uses the full width. (MAG-003, MAG-004.)
11. **Run id mismatch.** The lava-floods entry names Ben's run `1a0f6e918c9`; the research note is `run-review-1a0f969d358.md`, which describes `earthlike_quick_16k` supercontinent 2026-10-01. They may be one run with a mistyped id in one place; nothing says. The lava entry's own caveat: "A preview's world seed is random, so the cave layout differs between those two."
12. **Admission, from memory.** `docs/research/notes/run-review-1a0f969d358.md` section 6: "Hawaii-Emperor is about 6,000 km over about 80 Myr (from memory)". Used as the reference for the mod's 35 tracks (median 4,750 km over 78 Myr). Not a ledger row.
13. **Vanilla-lava claim has no measurement.** STATUS 2026-10-01 (Magma) and the pending-questions row: "Vanilla has lava within about 100 blocks of anywhere" with no source or measurement given. (Earth file only.)
14. **Real-world numbers without references.** The chamber depth 3 km ("Real shallow chambers sit 2-6 km ... Kilauea's about 2-4 km, Mount St Helens' from 5 km, Vesuvius' shallow one about 4-5 km") and "about eight of some 1,400 active volcanoes" with persistent lava lakes have no citation anywhere (MAG-002, MAG-020). Likewise every named-volcano size in `params.rs` (VOL-019 to VOL-023, VOL-032): no references; only the 60 km arc spacing cites a paper (de Bremond d'Ars, Jaupart & Sparks 1995), and nothing says whether it was read. (Note: the Mauna Kea 4.2 km in the `max_elevation_km` comment is given as an elevation, while the shield ceiling it is compared with, 4.4 km, is above sea level.)
15. **Hotspot-strength coupling is hard-wired (suspicion).** `volcanoes/mod.rs`: `strength = clamp((uplift / hotspot_uplift_mm_yr - 0.6) / 0.8, 0, 1)`. The 0.6 to 1.4 range matches the multiplier `state.rng.range(0.6, 1.4)` in `plates/hotspots.rs`; the two constants live in different files with no link, so a change in one silently skews the other. (VOL-008.)
16. **Caldera depth is set by the cap in the build-log example.** The build log's 2-Myr caldera is "14.6 km across, flat floor 764 m deep": 400 + 0.05 x 7,300 m = 765 m, i.e. the cap formula, not a physical depth law (VOL-030).
17. **Activity saturates, so crater lakes go to the lowest ids.** Build log (Magma, "Also seen"): "Activity saturates at 1.0 for most arc volcanoes, so the lakes go to the lowest ids among them." The ranking therefore does not reflect real activity. (MAG-019.)
18. **Baked summit is not the listed summit.** Build log (Magma, "Also seen"): "At every lake volcano the ground three to five blocks from the listed summit lies 1 to 11 blocks below it ... the amplifier's crater and the catalogue's position should be compared at L3." The magma pass reads terrain height at the catalogue position; the lake therefore sits beside the actual peak, and about 38 of 100 land volcanoes have a summit too sharp to hold one.
19. **Terralith remover radius.** `LavaFeatures.java` comment: "turns every lava source within 12 blocks of 300 points per chunk into deepslate"; build log: "300 blobs of radius 10 to 12 per chunk below y 30". Slightly different radii; nothing says which is right. (MAG-036.)
20. **Possible unit mixing in the flood-basalt radius (suspicion).** All other volcano lengths are block-metric km; flood basalts use `lip_radius_km / model_radius_km x planet radius` (Earth-normalised, 900 km scaled to the planet; about 110 km at 16k, derived). The run-review quotes province areas as "Earth-equivalent" 2.0-2.7 million km2, which holds only in the normalised frame. Consistent within each file, but not stated as a choice.
21. **Hypsometry drift tied to volcanoes.** STATUS 2026-10-01 (P1-17): "The drift came with the volcano and glacial merges" (land 0.283 to 0.383 against Earth 0.289) and "`volcanoes.density` 0: land 0.316 ... median 766 m"; Ben kept density 1 (decisions log 2026-10-01) with volcano volume left as "decisions for Ben (not asked)". Cones were "2x10^7 km3 built per run, a fifth of everything eroded". So the default volcano volume is a known contributor to land that is too high; it is kept as a game choice, not calibrated to a target.
