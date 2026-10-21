# Rivers, lakes and runoff

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Rain that does not evaporate runs downhill, gathers into rivers and fills hollows as lakes. The mod routes water over the finished terrain, sizes each lake from a water balance of its basin (inflow against evaporation: a lake may be full to its outlet, stand below its rim, shrink to a salt flat, or be dry), freezes lakes in cold climates, and sizes river channels from their discharge with hydraulic-geometry laws (width and depth grow as powers of discharge). Rivers and lakes are far wider in blocks than a true scale would give, so that they exist at 300 m per block; that enlargement is a game choice.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [HYD](ledger/HYD.md) | rivers | HYD-001 to HYD-027 (27) |
| [HYD](ledger/HYD.md) | runoff and evaporation | HYD-028 to HYD-046 (19) |
| [HYD](ledger/HYD.md) | lakes | HYD-047 to HYD-065 (19) |
| [HYD](ledger/HYD.md) | water in the game | HYD-066 to HYD-079 (14) |
| [HYD](ledger/HYD.md) | lakes and rivers against Earth (below, "Lakes and rivers against Earth", 2026-10-20) | HYD-090 to HYD-092 (3) |

Rows by status (81 active, 1 `merged`): `verified` 2, `cited-unverified` 12, `from-memory` 3, `abstract-only` 2, `calibrated` 1, `game-choice` 40, `uncited` 21.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/HYD.md](extraction-notes/HYD.md).
- Audit of grid artifacts in rivers: `docs/research/notes/grid-artifact-audit.md`.
- Work package in the verification plan: WP-5 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- River sinuosity (channel length over straight-line length) is 1.035 at the median in the audited world; the audit's Earth comparison ("commonly 1.1 to 1.5 and over") was written from memory.
- Leopold & Maddock 1953 was read in stage 1: the exponents 0.5 (width) and 0.4 (depth) are in it, as "representative" downstream values for a discharge of given frequency. The coefficients are the code's own.
- Two runoff laws coexist: hydrology uses the corrected cold-climate law, the young landscape and the erosion stage still use the old one (extraction notes HYD, flag 5).
- Holdridge defines biotemperature with temperatures above 30 degC counted as zero as well as those below 0 degC; the code applies only the lower limit (spot check HYD-030, MISMATCH).
- The shores in the game are kept free of cave mouths (HYD-078): the noise router's near-surface cave entrances opened dry pits beside rivers, lakes and shallow coasts, and the water pass built the banks and beds up from the pits' floors, which drew walls of dirt 10 to 27 blocks tall around the water (Ben's report, 2026-10-20). The router now closes the entrances within 8 blocks of the water, fading out by 16, and what the water pass fills is 3 blocks of soil over the column's own rock (HYD-079). Every other water GameTest runs without noise caves; `WaterShoreGameTests.shoresHaveNoDirtWalls` runs with them.
- A lake in the game is a game-side placement of the hydrology's level: every lake column is cut to its bed like a river channel, because the generated ground between noise-cell corners can stand a block or two above the rule's bilinear ground and left one-block-deep lake margins dry (HYD-077). A frozen lake's ice is its surface, not ground, so the snow and ice cover leaves it alone (BIO-084).

## Lakes and rivers against Earth (2026-10-20)

Ben's report of 2026-10-20: "the majority of the world surface (that isn't mountains) appears to be getting dominated by rivers & lakes". Measured with `tools/review/water_census.py` (every keyframe of a run, and its bake) on the quick 16k planet, 1000 Myr, 8 threads, seeds 1 to 3, both starts, on master d9bd2b65 unless a line says otherwise. A map pixel is 13 blocks (3.8 km) across.

**Rivers: the map drew them far wider than the game.** Before 2026-10-20 the keyframes' `rivers` layer and the satellite overlay drew every river at least a pixel wide plus half a pixel per tenfold of discharge, so on a 1,024-pixel map rivers 2 to 6 blocks wide in the game took a pixel or more: 11 to 14 % of the land was river on the map (p50 over the plate frames, 11.1 to 13.7 % by run; 12.6 % on Ben's world 1a15b0f76c4), while the bake's channels cover 1.8 to 2.6 % of the land. The map now draws each river at its width in the game (HYD-091): one narrower than a pixel is a faint thread whose coverage is its width in pixels, so the ink the map lays down is the bake's channel area (2.21 % of the land drawn against 2.21 % in the bake of the seed-1 supercontinent run, 2.39 % against 2.39 % for the scattered start). The game's channels are wider than true scale by design (HYD-003, HYD-005, HYD-008), so even the bake's 2.2 to 2.6 % is four times Earth's 0.58 % of the nonglaciated land (Allen & Pavelsky 2018, HYD-092); that is a game choice, not something this pass changed. View only: the bake, the hydrology and the statistics are unchanged.

**Lakes: about twice Earth's share.** Lakes cover 4.65 % of the land on the map (p50 over the plate frames, 3.8 to 5.9 % by run; 5.85 % of the ice-free land) and 4.35 % in the bake (3.6 to 5.2 %); the largest lake is 8,000 to 31,000 km2 in the bakes, 0.15 % of the planet on the map. Earth's natural lakes of 10 ha and more cover 1.8 % of the land, of 1 ha and more 2.2 % (Messager et al. 2016, read; HYD-090), and every lake over 0.002 km2 about 3.7 % of the nonglaciated land (Verpoorter et al. 2014, abstract only; HYD-062). The game's lakes are 1 km2 and more (HYD-047), so Earth's share for the same lakes is under 1.8 %. A quarter to a third of the bake's lake area (1.2 % of the land on average, 1.2 to 1.6 % in the runs split) is closed ground a block or more below sea level, which the game floods by rule (HYD-052; Ben deferred dry land below sea level): on Earth such basins are often dry (the Qattara Depression and Death Valley are; from memory).

What the lakes are, measured at a lake-heavy time of world 1a15b0f76c4 (`tools/review/lake_basins.py` on the plate fields): at step 315, 37 % of the lake cells lie on oceanic crust and 36 % on ground below sea level, and 11 steps later (lakes 9.57 % of the land) 46 % and 45 %: pockets of sea floor under the 5,000 km2 an open sea needs, enclosed by land. By regime the lake cells at step 315 are craton 43 %, ocean basin 16 %, back-arc 15 %, passive margin 9.5 %, ridge 8 %. The lakes are not set by the climate: the largest are full to their outlets with inflow above their evaporation (the seed-1 supercontinent bake's largest, 30,879 km2: 2,326 m3/s in, 1,793 m3/s evaporated), so the closed depressions set their area, and the water balance (HYD-049, HYD-050) only trims the dry ones.

Where the depressions come from (counterfactuals: 11 plate steps from step 315 with one law turned off, `tools/review/lake_basins.py` on the result; lakes as a share of the land against 9.57 % unchanged):

| Change | Lakes | Land |
|---|---|---|
| none | 9.57 % | 0.320 |
| no settling (ISO-078) | 6.37 % | 0.267 |
| no lower-crustal flow | 7.87 % | 0.295 |
| no flexure | 5.34 % | 0.350 |
| the cover's pore water in the column (tried, reverted) | 8.54 % | |
| deposits loading as saturated porous sediment, 0.532 of the mantle's density (ISO-047 is 0.8) | 9.92 % | 0.337 |
| deposits with no load at all | 9.19 % | 0.383 |

- The lake share doubled when the continents began to settle onto their crust and the lower crust began to flow (bc2e5472): Ben's world 4 (be17a7cf, before it) had 2.75 % on the map and 2.17 % in the bake; world 5 (1a15b0f76c4, master c24ff793) 4.69 % and 3.62 %.
- Sediment does not fill the basins: at step 315 the lake cells get a median 32 m of deposition per Myr (12 % under 1 m), while their floors sink a median 81 m per 2-Myr plate step.
- Two laws were tried and measured. The porous cover in the plate model's column (isostasy-and-height-limit.md) fills a basin with half the sediment, but over six seed pairs it changed no lake share (map 4.65 to 4.53 %, bake 4.35 to 4.42 %) and lifted the continents, so it was reverted. Lighter deposits in the surface's flexure barely move the lakes (the table). The flexure's own moats (the mountain and sea loads) and the oceanic pockets remain the open causes.

In the game (checked by the bake numbers, no client): the largest lake on the raster of the seed-1 scattered bake is at play (-350, -996): `deeptime inspect-column` gives a lake at level y 100.31, its bed and ground at y 85 (15 blocks deep), as the raster has it (6,532 km2; the bake statistics' third-largest at 6,537 km2). The bake statistics' block coordinates of the largest lakes (`notes.bake.water.stats`) are in the hydrology grid's frame, not play coordinates.

On master 9c9c2012 (PLT-294 on in the presets, the arcs' PLT-330 and PLT-331), seed 1, both starts, with the river ink rule: lakes cover 4.31 % (supercontinent) and 4.12 % (scattered) of the land on the map (5.18 % and 4.97 % of the ice-free land), 4.71 % and 3.76 % in the bake, against 3.81 % and 4.83 % for the seed-1 supercontinent on master d9bd2b65; the largest lake is 15,903 and 18,646 km2. The map's rivers sum to 2.33 % and 2.28 % of the land, the bake's channel area to the digit. In the game the largest lake on the seed-1 supercontinent raster, at play (-3994, 3118), is a lake at level y 91.94 over ground at y 61 (the raster's floor 62.06 is that block's top), 30 blocks deep.
