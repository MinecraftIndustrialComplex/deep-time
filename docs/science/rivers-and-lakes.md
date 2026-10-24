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
| [HYD](ledger/HYD.md) | below sea level (below, "Below sea level", 2026-10-20) | HYD-100 to HYD-105 (6) |
| [HYD](ledger/HYD.md) | a lake's sediment (below, "Basins that outlast their sediment", 2026-10-21) | HYD-110 (1) |
| [HYD](ledger/HYD.md) | the sea's one rule (below, "The sea's one rule", 2026-10-21) | HYD-106 to HYD-109 (4) |
| [HYD](ledger/HYD.md) | river channels in the game (below, "River channels cut to the bake's depth", 2026-10-22) | HYD-120 to HYD-124 (5) |
| [HYD](ledger/HYD.md) | deltas and distributaries (below, "Deltas and distributaries", 2026-10-23) | HYD-130 to HYD-144, HYD-146, HYD-147 (17) |
| [HYD](ledger/HYD.md) | floodplains and levees (below, "Floodplains and levees", 2026-10-24) | HYD-150 to HYD-153 (4) |
| [HYD](ledger/HYD.md) | meanders (below, "Meanders", 2026-10-24) | HYD-155 to HYD-159 (5) |
| [HYD](ledger/HYD.md) | alluvial fans and braided reaches (below, "Fans and braids", 2026-10-24) | HYD-165 to HYD-175 (11) |
| [BIO](ledger/BIO.md) | land on a delta plain (below, "Stage 1b on the D5 world"; biomes.md, "Land on a delta plain") | BIO-185 (1) |

Rows by status (134 active, 1 `merged`): `verified` 18, `cited-unverified` 12, `from-memory` 4, `abstract-only` 3, `calibrated` 1, `game-choice` 73, `uncited` 23.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/HYD.md](extraction-notes/HYD.md).
- Audit of grid artifacts in rivers: `docs/research/notes/grid-artifact-audit.md`.
- Work package in the verification plan: WP-5 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- River sinuosity (channel length over straight-line length) is 1.035 at the median in the audited world; the audit's Earth comparison ("commonly 1.1 to 1.5 and over") was written from memory.
- Leopold & Maddock 1953 was read in stage 1: the exponents 0.5 (width) and 0.4 (depth) are in it, as "representative" downstream values for a discharge of given frequency. The coefficients are the code's own.
- Two runoff laws coexist: hydrology uses the corrected cold-climate law, the young landscape and the erosion stage still use the old one (extraction notes HYD, flag 5).
- Holdridge defines biotemperature with temperatures above 30 degC counted as zero as well as those below 0 degC; the code applies only the lower limit (spot check HYD-030, MISMATCH).
- The shores in the game are kept free of cave mouths (HYD-078): the noise router's near-surface cave entrances opened dry pits beside rivers, lakes and shallow coasts, and the water pass built the banks and beds up from the pits' floors, which drew walls of dirt 10 to 27 blocks tall around the water (Ben's report, 2026-10-20). The router now closes the entrances within 8 blocks of the water, fading out by 16, and since 2026-10-21 vanilla's noodle caves there too within the band 12.5 blocks under the ground, after the shore test found noodle tubes 1 to 4 blocks under shallow sea floors and an 8-block void under a river bank with the entrances shut (173 of 11,086 shore columns in a pit; none after); what the water pass fills is 3 blocks of soil over the column's own rock (HYD-079). Every other water GameTest runs without noise caves; `WaterShoreGameTests.shoresHaveNoDirtWalls` runs with them.
- A lake in the game is a game-side placement of the hydrology's level: every lake column is cut to its bed like a river channel, because the generated ground between noise-cell corners can stand a block or two above the rule's bilinear ground and left one-block-deep lake margins dry (HYD-077). A frozen lake's ice is its surface, not ground, so the snow and ice cover leaves it alone (BIO-084). The climate's ice rule (iced or kept open in a cold biome, HYD-072) answers only in the columns that hold the lake by the water pass's own test (the ground's top block under the lake's top block): the shore rule (HYD-067) gives the lake's level to the dry columns within its reach too, and until 2026-10-23 the rule answered there as well, which kept open five blocks of a hot-spring pool's water one block under the pool's surface on master 4ab29084's test planet.

## Lakes and rivers against Earth (2026-10-20)

Ben's report of 2026-10-20: "the majority of the world surface (that isn't mountains) appears to be getting dominated by rivers & lakes". Measured with `tools/review/water_census.py` (every keyframe of a run, and its bake) on the quick 16k planet, 1000 Myr, 8 threads, seeds 1 to 3, both starts, on master d9bd2b65 unless a line says otherwise. A map pixel is 13 blocks (3.8 km) across.

**Rivers: the map drew them far wider than the game.** Before 2026-10-20 the keyframes' `rivers` layer and the satellite overlay drew every river at least a pixel wide plus half a pixel per tenfold of discharge, so on a 1,024-pixel map rivers 2 to 6 blocks wide in the game took a pixel or more: 11 to 14 % of the land was river on the map (p50 over the plate frames, 11.1 to 13.7 % by run; 12.6 % on Ben's world 1a15b0f76c4), while the bake's channels cover 1.8 to 2.6 % of the land. The map now draws each river at its width in the game (HYD-091): one narrower than a pixel is a faint thread whose coverage is its width in pixels, so the ink the map lays down is the bake's channel area (2.21 % of the land drawn against 2.21 % in the bake of the seed-1 supercontinent run, 2.39 % against 2.39 % for the scattered start). The game's channels are wider than true scale by design (HYD-003, HYD-005, HYD-008), so even the bake's 2.2 to 2.6 % is four times Earth's 0.58 % of the nonglaciated land (Allen & Pavelsky 2018, HYD-092); that is a game choice, not something this pass changed. View only: the bake, the hydrology and the statistics are unchanged.

**Lakes: about twice Earth's share.** Lakes cover 4.65 % of the land on the map (p50 over the plate frames, 3.8 to 5.9 % by run; 5.85 % of the ice-free land) and 4.35 % in the bake (3.6 to 5.2 %); the largest lake is 8,000 to 31,000 km2 in the bakes, 0.15 % of the planet on the map. Earth's natural lakes of 10 ha and more cover 1.8 % of the land, of 1 ha and more 2.2 % (Messager et al. 2016, read; HYD-090), and every lake over 0.002 km2 about 3.7 % of the nonglaciated land (Verpoorter et al. 2014, abstract only; HYD-062). The game's lakes are 1 km2 and more (HYD-047), so Earth's share for the same lakes is under 1.8 %. A quarter to a third of the bake's lake area (1.2 % of the land on average, 1.2 to 1.6 % in the runs split) lay on ground at or below sea level; that count took the pockets the sea reaches as well, and the closed ground a block or more below sea level, which the game flooded by rule until 2026-10-20 (HYD-052), is 0.45 to 1.14 % of the land on master 72280b3b and 0.05 to 0.52 % with HYD-110 ("Below sea level" below, where it now takes its water balance): on Earth such basins are often dry (the Qattara Depression and Death Valley are; from memory, HYD-101).

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

## Basins that outlast their sediment (2026-10-21)

The question: why closed depressions outlast the sediment that reaches them, so that lakes cover about twice Earth's share of the land (above). Measured on the quick 16k planet, 1000 Myr, 8 threads, seeds 1 to 3, supercontinent and scattered starts, master 72280b3b against the same with the change, `tools/review/water_census.py` (map: p50 over the plate frames; bake) and `tools/review/lake_basins.py` (closed basins on the plate fields of the epoch checkpoints).

**The law that was wrong: a lake's sediment built a causeway.** The surface stage's lake rule (ERO-020) laid the sediment that reached a lake only on the cells along the flow path across it, each up to the lake level, and passed the rest on. Once those cells were full, the path crossed the lake at its surface and every later load went straight over the spill: the rest of the lake kept its depth whatever its supply. In 11 plate steps from master's seed-1 supercontinent at step 360, the continental lake cells got a median 32 m of deposition per 2 Myr but a quarter of them 6 m or less and a tenth none, while their floors sank a median 7 m per 2 Myr, the slow end of Earth's intracratonic basins (a few to tens of metres per Myr; from memory). Removing the 11 steps' local isostatic response (the gap budget's `rebound` part) or the lake water's part changed the lake share by 0.1 point or less, the flexure's parts together by 1 point the other way. The basins were not outrunning their sediment; the sediment was not reaching them.

**The law now (HYD-110).** A lake keeps the sediment that reaches it, up to its room (the volume under its level), and passes the rest on where its water leaves it (its spill, the sea, a sink or a lower lake). What it keeps settles over the whole lake, every cell raised by the same share of its depth, so the deep parts take the most. A reservoir whose capacity is large against its inflow traps nearly all of its sediment (Brune 1953, from memory), and lake sediment focuses into the deep basin (Lehman 1975, from memory); the equal share of depth is this project's simplification. The parallel routing keeps its bit-identical serial reference (`surface/routing/parity.rs`).

| Run | Map lakes, % of land | Bake lakes, % of land | Map land, share | Bake land, share | Land above 2 km, % of land (bake) | Coast p50, km (map) |
|---|---|---|---|---|---|---|
| supercontinent 1 | 3.69 to 3.21 | 5.65 to 0.88 | 0.242 to 0.273 | 0.231 to 0.257 | 25.5 to 14.0 | 266 to 325 |
| supercontinent 2 | 4.97 to 4.00 | 4.35 to 1.17 | 0.250 to 0.287 | 0.256 to 0.280 | 22.3 to 14.8 | 263 to 420 |
| supercontinent 3 | 3.29 to 4.60 | 2.91 to 2.05 | 0.250 to 0.290 | 0.202 to 0.318 | 26.5 to 18.5 | 256 to 277 |
| scattered 1 | 4.11 to 3.30 | 5.46 to 1.48 | 0.216 to 0.246 | 0.231 to 0.261 | 17.8 to 22.7 | 226 to 291 |
| scattered 2 | 4.20 to 5.07 | 3.41 to 3.01 | 0.229 to 0.252 | 0.203 to 0.236 | 23.5 to 19.3 | 247 to 250 |
| scattered 3 | 4.71 to 3.96 | 4.25 to 1.56 | 0.263 to 0.254 | 0.272 to 0.244 | 21.7 to 18.6 | 297 to 241 |
| mean | 4.16 to 4.02 | 4.34 to 1.69 | 0.242 to 0.267 | 0.233 to 0.266 | 22.9 to 18.0 | 259 to 301 |

- The bake's lakes fall in all six runs, to under Earth's 1.8 % for lakes of 10 ha and more (HYD-090), and the largest lake from a mean 17,465 to 6,372 km2. The cut happens in the young landscape: the last plate frame holds 4.96 % (4.93 % on master), the young stage's 2 Myr on the finer grid leave 0.98 % (3.07 % on master), the amplifier 1.58 % (4.35 %).
- The map's plate frames do not change (4.16 to 4.02 %; 11 steps from the lake-heavy epochs, 10.1 to 10.6 % on the plate fields): a lake filled as a whole keeps its area until it is full, and in the plate era the plates remake the basins faster than any supply fills them (below).
- Land rises on the mean and in five runs (scattered 3 loses 0.009 on the map, 0.028 in the bake); the share of the land above 2 km falls in five, and its share of the planet from 5.3 to 4.8 %. The plate checks pass 29 and 31 of 45 targets (27 and 28 on master; `deeptime plate-check`, per start over the three seeds). The rise of the land is measured, not traced.
- The young stage has no subsidence of its own, so a basin the plates keep sinking on Earth (a Baikal, a Tanganyika) can fill in its 2 Myr as readily as a basin left behind; the bake does not tell the two apart.

**What keeps the plate era's lakes: sea-floor pockets (not changed, for Ben).** On master's epoch checkpoints (4 epochs of each of the six runs, `lake_basins.py`), the lakes are 7.6 % of the land on the plate fields and 37 % of their area lies on oceanic crust: 126 closed basins on sea floor per snapshot, 3.3 million km2 Earth-equivalent (the planet's areas times 66.3, Earth's radius over the planet's squared), against 2.67 million km2 for all of Earth's natural lakes (HYD-090). There are 3.1 basins at or over the Caspian's 371,000 km2 (Earth-equivalent) per snapshot, 1.0 of them on sea floor; Earth has one, the Caspian, whose southern basin lies on thin, ocean-like crust, and the Black Sea is open to the ocean (both from memory). A single remnant ocean caught between continents is physical; a hundred of them, a third of the lake area, are not. The law that keeps them is the oceanic dampening (PLT-129): it relaxes an oceanic sample towards its floor depth with the age-depth curve's own 47 Myr time constant whatever loads it, so on old floor enclosed by land it sinks the sediment the surface lays. In the 11 steps from master's seed-1 step 360 the oceanic lake cells (crust p50 201 Myr old) took a median 234 m of deposition per 2 Myr and sank a median 96 m; the age-depth curve subsides about 2 m per Myr at 200 Myr (Parsons & Sclater 1977's old-floor curve, from memory), the model about 48. A held floor raised by the fill's isostatic rise (the sediment-loading correction of Steckler & Watts 1978, from memory; 0.44 m per metre under the sea, 0.2 on land, from the flexure's rock shares, ISO-047) was built and measured (commit 640e1797, then taken back out): the oceanic lake cells' floors went from sinking 96 m to rising 17 m per 2 Myr and the lake cells below sea level from 30 to 17 %, but the pockets stayed closed above sea level and the lakes did not fall (map 4.16 to 4.66 %, bake 4.34 to 3.98 %; with HYD-110, 3.93 % and 1.73 %), while the land rose from 0.242 to 0.297 on the map (0.308 with HYD-110; Earth 0.29) and the land above 2 km from 5.3 to 6.1 % of the planet (6.4 %). Because the sea level is a prescribed curve, not the ocean's water over its basins' volume, every metre of sediment the ocean floor keeps raises the land: the dampening is the ocean's sediment sink, as the withdrawn age-rate trial found (erosion-and-sediment.md). Fixing the floor law therefore means a sea level that answers to the ocean's volume, a plate-model change left to Ben. Ben took it as its own task (decisions log 2026-10-21), and since 2026-10-21 the sea is the ocean's water over its basins and the floor sinks at GDH1's rate whatever lies on it (sea-level.md, PLT-350 to PLT-355). It did not cut the pockets: on the six-run ensemble (master f1632cf3 against the change, the same tools) the closed basins on sea floor rose from 124 to 180 per snapshot and from 34 to 40 % of the plate era's lake area, the basins of the Caspian's size or more from 3.7 to 5.2 per snapshot (0.9 to 1.6 on sea floor), and the lakes from 3.85 to 4.16 % of the land on the map and from 1.69 to 1.99 % in the bake. The pockets stand above the sea (their lake level p50 552 m over the datum) on island arcs and the stacked crust the plate model keeps under the oceanic label, and the new sea stands 300 to 700 m lower for most of a run, laying more of that ground dry; they go with the arcs' rate (the plate-geometry task), not with the sea.

Also ruled out: counting a lake's water as a load in the surface flexure (only the sea's water loads it now) deepens a lake while it fills and leaves the same load when it is full, so it would keep lakes longer, not shorter; lighter lake deposits were measured in the lakes task (above) and moved nothing.

## Below sea level (2026-10-20, HYD-100)

Before 2026-10-20 every terminal basin whose floor lay a block or more below sea level stood at sea level however dry its climate (HYD-052), and the game stood the sea on any ground below sea level. Since format 0.6:

- **The rule** (HYD-100). The open sea is every cell a block or more under sea level in an 8-connected body of at least 5,000 km2 (HYD-012); a cell at or under sea level that joins it through such cells is reached. A basin whose lowest cell is reached keeps HYD-052 and the sea (Ben's row of 2026-09-30: coastal flats below sea level are sea). A closed basin takes its water balance as a basin above sea level does (HYD-050): a lake whose surface may stand below sea level, a playa, or dry ground. Earth has all three (HYD-101, from memory: the Dead Sea and the Caspian; Badwater; the Qattara Depression).
- **The layer** (BAK-055, HYD-102). `closed_basin` marks the closed ground and an 8-block halo, and the column rule's sea step holds only where it is 0. On a Petroff–Guyou world the projection pass draws it again on the world's own ground (HYD-105): taken from the nearest Lambert cell, it had flagged pockets of sea floor whose Lambert sill the world's averaged ground no longer has, and the game would have left them dry holes beside the sea (dry flagged ground the world's sea reaches, on master 72280b3b's planets: 47, 40, 244, 23, 1,519 and 248 km2 in the six bakes of the second table below, in its order; with the redraw, none in any of the six bakes on master 3768c05b's planets, and none in the seed-2 scattered bake of 72280b3b, where it cleared 5,088 low cells, 1,766 km2, and left the census unchanged).
- **In the game** (HYD-103, HYD-104). Under sea level in a closed column the fluid picker answers water at the lava floor's level, not the sea; the water pass drains the open water an aquifer carries in from a coast above the column's own lake or river; the cave shield (HYD-078) counts no closed cell as sea; salt lies where a playa's water dried off the floor (HYD-054). Far terrain, the dev map, the keyframes' water view, the workbench's map and the biome bake treat the closed ground as land, with its lake where it has one (the biome layers' call is decided again on the world's ground in the projection pass, below, HYD-108).
- **Caves and aquifers under such basins are kept** (coordinator's call): carvers and noise caves run as elsewhere and vanilla's local aquifers under the basin stay water; only the sea's level is kept out. A dry basin floor may have cave mouths like any dry land; beside its lake or river the shield closes them as anywhere.
- **GameTests** (`WaterGameTests.closedBowlsHoldNoSea` and `closedBowlsHoldNoSeaWithTheAquifer`, on the fixture of two closed bowls below sea level, BAK-056): the dry bowl (floor y 40) holds no water in any of its 5,025 columns and no sea between y 49 and 62; the wet bowl (floor y 30) holds its lake at its own level, top block y 49 against the sea's 62, with no water over it, without and with noise caves and the aquifer.

Measured: earthlike_quick_16k, 1000 Myr, 8 threads, seeds 1 to 3, both starts; master 3768c05b (with HYD-110) against the branch with that master merged in; `tools/review/water_census.py` over every keyframe and the bake. The planets are identical before and after (the same land in the bake, the same largest lake). Map lakes are the p50 over the plate frames; the other shares are of the bake's land, on the world rasters. "Closed" is ground a block or more below sea level that the world's open sea does not reach (the rule above, on the world rasters); "in the game" is that ground where `closed_basin` is 1, and its lake, dry and playa parts are after the change; "sea on closed ground" is closed ground on which the game stands the sea: all of it before, after only where the Lambert hydrology found it open (below).

| run | map lakes before, after | bake lakes before, after | closed | in the game | its lake | its dry | its playa | sea on closed ground before, after | lakes with their surface below sea level (bake) | land (bake) |
|---|---|---|---|---|---|---|---|---|---|---|
| supercontinent 1 | 3.21, 3.07 % | 0.88, 0.88 % | 0.09 % | 0.09 % | 0.08 % | 0.00 % | 0 km2 | 1,771, 54 km2 | 385, 5,704 km2 | 0.2613 |
| supercontinent 2 | 4.00, 3.86 % | 1.17, 1.16 % | 0.09 % | 0.09 % | 0.09 % | 0.00 % | 0 km2 | 2,015, 25 km2 | 301, 14,709 km2 | 0.2854 |
| supercontinent 3 | 4.60, 4.47 % | 2.05, 2.05 % | 0.52 % | 0.29 % | 0.29 % | 0.00 % | 1 km2 | 12,957, 5,776 km2 | 433, 15,016 km2 | 0.3244 |
| scattered 1 | 3.30, 3.15 % | 1.48, 1.47 % | 0.05 % | 0.05 % | 0.05 % | 0.00 % | 0 km2 | 1,095, 47 km2 | 438, 11,434 km2 | 0.2628 |
| scattered 2 | 5.07, 4.79 % | 3.01, 3.00 % | 0.11 % | 0.10 % | 0.09 % | 0.01 % | 0 km2 | 1,925, 4 km2 | 235, 8,445 km2 | 0.2383 |
| scattered 3 | 3.96, 3.73 % | 1.56, 1.54 % | 0.14 % | 0.13 % | 0.11 % | 0.02 % | 0 km2 | 2,613, 27 km2 | 436, 13,857 km2 | 0.2514 |

- With HYD-110 the lakes fill with their sediment, and closed ground a block or more below sea level is 0.05 to 0.52 % of the land (0.45 to 1.14 % on 72280b3b, the second table). Nearly all of it holds its own lake: 235 to 438 lakes a bake stand with their surface below sea level, and the game leaves 0.00 to 0.02 % of the land dry below sea level. Playas are 0.01 to 0.17 % of the land, the same before and after, and 0 to 1 km2 of them lie on closed ground below sea level.
- The bake's lakes fall 0 to 0.02 points of the land (mean 1.69 % both, median 1.52 to 1.51 %), the map's 0.13 to 0.28 points (mean 4.02 to 3.85 %, median 3.98 to 3.79 %): over the plate era's frames more closed basins are dry than in the bake. The map's land share rises by up to 0.0018 (dry closed ground drawn as land); the bake's does not change.
- No low ground the layer flags is reached by the world's sea in any bake (HYD-105).
- The other way round: closed ground on the world rasters that the Lambert hydrology found open keeps the sea, 4 to 54 km2 in five bakes and 5,776 km2 on supercontinent 3 (not traced). That is master's behaviour on that ground (the sea at sea level behind a sill); giving it a water balance would need the hydrology on the world's grid.
- The hydrology's own statistics (`notes.bake.water.stats.below_sea`, on the Lambert grid) put the dry part at 0.00 to 0.23 % of the land; the larger figure (scattered 3) counts pockets whose Lambert sill the world's ground has lost, which the world's sea covers (HYD-105 cleared 5,080 low cells there, 1,764 km2).

First measured on master 72280b3b (before HYD-110) against the branch at 51d309c8, the same ensemble and census (closed ground's parts after; dry from the census's rule, not the layer):

| run | map lakes before, after | bake lakes before, after | closed | its lake | its dry | its playa | lakes with their surface below sea level (bake) | land (bake) |
|---|---|---|---|---|---|---|---|---|
| supercontinent 1 | 3.69, 3.53 % | 5.65, 5.65 % | 0.66 % | 0.66 % | 0.00 % | 0 | 403, 12,506 km2 | 0.2375 |
| supercontinent 2 | 4.97, 4.66 % | 4.35, 4.27 % | 0.45 % | 0.45 % | 0.01 % | 0 | 426, 20,317 km2 | 0.2668 |
| supercontinent 3 | 3.29, 3.01 % | 2.91, 2.85 % | 0.60 % | 0.56 % | 0.04 % | 0 | 358, 5,379 km2 | 0.2071 |
| scattered 1 | 4.11, 3.93 % | 5.46, 5.19 % | 1.10 % | 1.02 % | 0.08 % | 0 | 362, 40,620 km2 | 0.2371 |
| scattered 2 | 4.20, 3.97 % | 3.42, 3.41 % | 0.74 % | 0.73 % | 0.00 % | 0 | 410, 5,598 km2 | 0.2069 |
| scattered 3 | 4.71, 4.45 % | 4.25, 4.04 % | 1.14 % | 0.97 % | 0.16 % | 0 | 305, 11,544 km2 | 0.2794 |

- There the bake's lakes fell 0 to 0.27 points of the land (median 4.30 to 4.16 %), the map's 0.16 to 0.31 points (median 4.16 to 3.95 %); playas were 0.01 to 0.02 % of the land.
- `tools/review/lake_basins.py` was not re-run: it reads a checkpoint's plate fields, and the simulation is unchanged.

## River channels cut to the bake's depth, not curbed (2026-10-22, HYD-120 to HYD-124)

Ben's largest river on a fresh earthlike_quick_16k single-player world (x 6131, z -3022, 1,000 to 1,727 m3/s) was about a block deep where the bake's channel is 7.9 to 9.9 blocks deep and 16 to 21 half-wide, and every river, most visibly the one-block streams, ran between one-block lips along both banks. Both came from the game's column rule (`water::core_with_surface` and `finish`, Java `Water.core` and `finish`), not from the bake: the bake's rivers file holds the right channel.

**Why the channels were shallow.**

- *The lowest-surface hit* (the main cause). A river column took its water and its bed from the one covering segment with the lowest water surface w, ties to the lowest segment id (`Rivers::sample_among`, Java `Rivers.sampleRange`). A wide river's vertices are 3 to 4 blocks apart against a half-width of 16 to 21, so a dozen segment capsules cover each column, and where w is flat (at sea level, or along a filled reach) or falls downstream, the lowest-w one is a neighbour whose capsule only grazes the column: d/hw 0.88 to 0.99 on the largest river's centre line, where its parabolic bed is a block deep. The game cut that neighbour's bed.
- *Lake and sea before the river* (minor). The column rule tested the lake level and then the sea before the river, with the bed at the ground, so a channel crossing lake water or shallows lost its depth there.
- *The curb.* Every bank column beside a river was raised to the river's top + 1 (HYD-069). A stream sits about a block under its filled surface (HYD-018), and the ground beside it rounds to the same block as its water, so the rule built a one-block wall on both sides.

**The rules now.**

- **The deep hit** (HYD-120). The bed comes from the covering segment whose floor under the column, w - max(1, depth (1 - (d/hw)²)), is lowest; the water's top still comes from the lowest-surface hit, so the water stays level across a confluence or a flat.
- **Through the sea** (HYD-121). A sea column a channel covers takes the lower of its ground and the channel's floor as its bed, and the water pass cuts it as a river column with the sea's top: the channel runs on across sea-level flats and through the shallows at the mouth, as an estuary's does.
- **Across a shallow lake** (HYD-122). A lake column a channel covers takes the channel's bed only where the lake water is less than a block deep over the generated surface (a river running over a lake's flat edge); deeper lake water keeps its own floor, so a river entering a lake does not trench it.
- **Incision instead of a curb** (HYD-123). Where the lower of the ground a block beyond the channel's two edges is at or under the river's top block, the water and the bed go a block down (never under the sea's top block), so the banks are the ground itself. Where the banks stand above the water, as in a valley, nothing changes.
- **Banks hold the water there is** (HYD-124). A bank beside a river is raised only to the water the river holds there: its own top, or a higher neighbour's top where the river steps down. A raise is left only where a real dip in the ground crosses the river's path.

Java, the far terrain and the Rust mirror agree: `FarColumn` reads `Water.core` and counts a sea or lake column cut by a channel as channel for its sand, gravel and soil; `deeptime-format`'s water module and `deeptime-inspect`'s river map are the Rust side; the column parity fixtures (`ColumnParityTest`, `WaterRegionTest`) pin them. FORMAT.md's "Water columns" gives the steps.

**Measured on a copy of Ben's bake** (C 13,312, 2,850 chains; `deeptime inspect-column` before and after over the river's vertex columns and cross-sections, scripts in the branch's build/river-depth/). Bed against the bake's bed w - depth, + is shallower:

| Columns | Before | After |
|---|---|---|
| Largest river, last 700 vertex columns with depth 3 or more | mean +5.87 blocks, 4 % within a block | +0.01, 100 % |
| Every channel's vertex centre column with hw and depth 2 or more, river (13,911) | +1.59, 59 % | -0.03, 99 % |
| the same, lake water (235) | +2.22, 20 % | +0.13, 98 % |
| the same, sea (30) | +1.63, 53 % | +0.17, 100 % |
| Bank columns beside streams (hw under 3) raised | 3,594 of 18,865 (19.1 %), 3,580 by one block | 9 (0.05 %) |
| Bank columns beside rivers raised | 549 of 3,123 (17.6 %) | 0 |

Near the lake mouths (columns within hw + 2 of the last vertex of chains ending in a lake) no lake column is cut (879 keep their floor); a cut without HYD-122's test trenched 500 of them a mean 1.94 blocks. On the river's edge columns the lowest neighbouring bank now stands level with the water's top block in 2,992 of 18,166 stream and 415 of 3,129 river columns (flush), where before it was never under top + 1.

**The same river, generated.** On the river's last 700 vertex columns the game held a mean 2.36 blocks of water against the bake's 9.00 in Ben's region files (372 columns generated, 5 % within a block; 1.57 against 9.47 where w is the sea level). This build, generating the same columns from a copy of his bake, holds 8.58 against 9.00 on those 372 (98 % within a block), 7.68 against 8.08 on all 700 (99 %), and 9.00 against 9.47 on the 138 at sea level (100 %).

**The GameTest** (`RiverDepthGameTests` on the river-test world, `deeptime synth --kind river-test`: a river of half-width 10 and depth 8 sloping down onto sea-level flats and through a block of sea into the sea, and a stream on a plain whose surface lies in the plain's ground block). Before (the base's column rule): 19 of 1,255 columns of the river's lower reach within a block of the bake's depth (1.5 %), 12 of 745 at w = sea level, a mean 2.08 blocks of water against 6.56; all 225 stream columns held their water in the plain's ground block behind a raised bank, 150 of 750 side columns raised a block. After: 1,253 of 1,255 (99.8 %, the two others where the surface steps down a block), 745 of 745 at sea level, 6.30 against 6.56; no stream column perched and no side column raised.

**Game choices.** All five rows are block-world choices (agent, 2026-10-22, for the coordinator's river depth task and Ben's curb report). The depth the game cuts is the bake's own (HYD-022). The one-block incision is the freeboard the bake already leaves under its filled surface (HYD-018). No levee height is cited: the walls were the rule's, not levees; natural levees build from overbank deposition on lowland rivers (from memory), which the bake does not model.

## The sea's one rule (2026-10-21, HYD-106 to HYD-109)

The game puts the sea on a column by one test, step 3 of the column evaluator: the top block of the ground, ceil(H) - 1, is under sea level - 1 and the `closed_basin` flag is not 1. It is now one function in each language (`water::sea_covers`, Java `Water.seaCovers`; HYD-106), which the column evaluator, the terrain's water columns, the far terrain (through `Water.core`) and the biome bake's projection pass all call.

- **The 35 m band** (HYD-107). A column whose height H lies in (sea level - 1, sea level] has top block sea level - 1, which is not under the line, so it is dry: one block of height, 35 m, of beach flat at the sea's level. The biome bake called all of it ocean, and 5 to 10 % of the land carried an ocean biome on dry ground (92 % of those nodes in the band).
- **Two grids** (HYD-108). The biome bake decided on the simulation's Lambert L2 grid, the world is a footprint mean on another grid with its own closed flag (HYD-105), so 0.6 to 1.2 % of the land carried a land biome under the sea (a pocket HYD-105 opened to the world's sea, or ground the world averages under the line). The projection pass now re-decides every L2 node with `water::sea_covers_height` on the world's `elevation` and `closed_basin` and settles the nodes that disagree (BIO-170 to BIO-172, biomes.md "The land call on the world's ground"); 0 L2 nodes disagree after, on four earthlike_quick_16k bakes. What is left at the 8-block columns is the sub-node coast (BIO-174, 0.25 to 0.38 % and 0.79 to 1.09 % of the land, a median 2 blocks from the line).
- **Not on this rule** (HYD-109). `VolcanicBiomes.land`, `GeologyGround.sea`, the fluid picker in `LavaFluids` and the biome census's `land` keep their own thresholds; they answer other questions or measure, and were not changed.

## Deltas and distributaries (2026-10-23, HYD-130 to HYD-144, HYD-146, HYD-147)

V1 gap 5 (physics-gap-audit.md, "Rivers and deltas"), stage 1 of four: delta plains above the sea at river mouths and the distributary channels across them. Ben's answer of 2026-10-22: "Start now, format change OK (Recommended)". Floodplains and levees, meanders, fans and braids are stages 2 to 4 and are not in this section. The stream-power solve keeps its single receiver: the splits live in the bake's network and the game's carving only.

### Design

**Where it runs.** In the bake's water part (`bake/water.rs`), on the hydrology grid (Lambert L3, 2 blocks a cell, or L3h), between two runs of `hydro::network::run`:

1. *Pass 1* drains the terrain as today and, with the network's `mouths` switch on (only the water part sets it; the amplifier's balance and the water view do not pay for it), also returns, for every river that reaches the sea, its mouth cell, its discharge Q, its catchment area A, the catchment's highest ground (relief R over the sea), its area-weighted mean annual temperature T, the share of its area that drains through a lake, and the direction the channel leaves the land in.
2. *Supply.* Each such river's sediment load Qs comes from BQART (Syvitski & Milliman 2007, read: Qs = ω B Q^0.31 A^0.5 R T for T ≥ 2 °C and 2 ω B Q^0.31 A^0.5 R below; ω = 0.02 for kg/s with Q in km³/yr, A in km², R in km, T in °C; HYD-130). B = I L (1 − TE) Eh: glacial erosion I, lithology L and human influence Eh are 1 (the paper's own B = 1 case explains 68 % of the variance); the trapping TE is the share of the catchment that drains through a lake, since a lake on a main stem keeps the river's sediment (HYD-110, HYD-131).
3. *How much plain.* The delta plain Earth builds for that load: geomorphic area A* from Qs, the least-squares fit to the 1,741 deltas of Nienhuis et al. (2026, data read): log10 A*[m²] = 5.463 + 0.932 log10 Qs[kg/s], scatter 1.09 dex (HYD-132). Their plain thickness is the channel depth D = 0.08 Q^0.65 m, at most 69 m (`get_retention.m`, read; HYD-133). The volume the river may place is V = m A* D, with m = 32 (HYD-141, game-choice): the rivers are drawn 40 to 70 times their physical width (HYD-001, HYD-003), so a plain of Earth's area for D5's largest river (about 300 km²) would be barely wider than its own 52-block channel and hold no second one. 32 keeps the plains in scale with the rivers as drawn. It is not tuned to Earth's share of delta land: D5's plains come to 0.39 % of its land, which is written beside Earth's 0.57 % (Edmonds et al. 2020, HYD-143) and not aimed at (decisions log 2026-10-23: results need not match Earth, and nothing is tuned to an Earth number). 48 was not run.
4. *Accommodation.* The plain grows out from the mouth over sea and beach-band cells (height at most the sea's surface, not lake, not a closed basin, not another delta), each cell costing its area times max(D, its depth below the sea): over ground shallower than D a river gets Earth's area for its load, over deeper water less, over a steep margin almost none (HYD-134). The river's course comes first: its stem's crossing of the beach band (from the last land cell to the first open-sea cell), in order from the coast, each cell with its banks (the beach-band cells beside it, and beside the land cell for the first), until the next cell and its banks would take the course past half the volume. Without the banks, the second pass's river left its plain sideways for the lower beach band beside it: on the D5 bake 121 of 154 plains had no split for that reason. Without the half share, a beach band hundreds of cells wide (the D5 world's are) took the whole volume as a strip three cells wide with no front. This is the supply-against-accommodation test; waves and tides, which Caldwell et al. (2019) show also decide whether a delta forms, are not modelled in stage 1 and are only compared against (HYD-143).
5. *Shape.* Then the lobe: the cells beside what the plain holds are taken in order of their distance from the nearest cell of the stem's crossing (claimed or not, so the lobe follows the river's way across the band) weighted by direction, d (1 + k (1 − cos θ)) with θ the angle from the channel's outflow direction and k = 1, so the plain is a lobe built forward and narrowing behind the old coast, the river-dominated form (HYD-135). Sea pockets the plain closes off from the open sea (low cells, shallow ones included, that the open sea reached across the low ground before the plain and no longer does) are filled with it up to 200 cells (72 km²); larger ones stay as lagoons, and a plain of fewer than 3 cells is no delta (HYD-142).
6. *Height.* The plain stands one block above the sea's top water block: its height H lies in (sea level, sea level + 1], top block = sea level, graded in 1/16 block steps as a ramp along the outflow direction from the plain's entry, from the river's last land cell's height over the sea (at most 15/16) at the entry down to 1/16 at the plain's farthest cell ahead, so the drainage runs outward. Where the lobe closes the course's end off, the course goes on by the shortest way through the plain (keeping off its edge, an edge cell costing 4 times more) to the plain's tip, its cell beside the open sea's water farthest along the outflow. The course itself keeps the beach band's height, in (sea level - 1, sea level]: each cell at the highest ground the course has from it on, so it never rises downstream and nothing is lowered, and at least 15/16 under the sea's surface (or the beach band beside its end, so the river spills onto it). That column is dry ground one block under the plain, where the game carves the channel anyway, and every other plain cell stands over it, so the second pass keeps the river on its course to the front (HYD-136). A first version raised the course with the plain; the coastal land and the beach band beside it lie a few sixteenths from the sea too, and on the D5 bake 67 of 190 plains had no river across them. A delta plain is a few metres above the sea, under one 35 m block, so a block is the smallest step that reads as land; the column rule then makes the plain dry land and a channel at the sea's surface one block below its banks. The detail amplitude is set to 0 on the plain so the noise term does not break it.
7. *Pass 2* drains the raised terrain (the lakes, the closed basins and every channel follow the new ground).
8. *Apex.* On the main stem across each plain, the apex (the delta node, the upstream-most bifurcation) is where the main stem comes onto the plain, the valley's mouth (HYD-137). On a delta the avulsion node sits about the backwater length L_b = h / S upstream of the shoreline (Jerolmack & Swenson 2007, as read in the review of Edmonds et al. 2021); on a fan, where the river leaves its valley at an abrupt fall in bed slope (Ganti et al. 2014, abstract). With h = D and S the main stem's water-surface slope over the 30 km above the old mouth, L_b is longer than the stem's run on the plain for the low rivers, which puts the node at the plain's landward end, and shorter for steep ones, whose deltas are fan deltas with the node at the valley's mouth: the same cell. L_b is reported beside each split. On the D5 bake 143 of 190 plains have the shorter L_b; a first version put their apex L_b up from the front, where almost no front lay ahead of it, and 4 plains split.
9. *Distributaries.* The plain's front (plain cells beside water the open sea reaches: the open sea, or the beach band's cells at or under the sea's surface where the plain stops inside a band wider than it grows; only those within a quarter turn of the outflow ahead of the apex, so the flanks behind it take no mouth) is ordered by bearing from the apex. The number of mouths N is the largest count, at most 8, whose channels, each carrying Q / N, fit the front two channel widths apart (a mouth bar forms within about two widths of a mouth and splits it, Canestrelli et al. 2014 in Edmonds et al. 2021; HYD-138). The front is halved recursively: at each bifurcation one branch keeps the parent chain and the other is a new chain that leaves a vertex of it, heading for the middle of its half of the front (or, where no way leads there, the half's front cell nearest the middle that a path reaches across the plain and that has a free way down to the open sea); each branch splits again halfway to the front, and a main stem that touches the plain at one cell (the plain spread along the coast beside a mouth that stays open) splits there, at the shore (channel lengths, widths and depths fall at every bifurcation, Edmonds et al. 2021; HYD-140). A branch carries the parent's discharge in proportion to the mouths it serves (HYD-139) and takes the width and depth laws' channel for it (HYD-001 to HYD-004). The paths run inside the plain (the shortest 8-connected path over plain cells, a cell another channel already takes costing 4 times more, then from the front cell the shortest 8-connected way over the band's sea-reached cells to the nearest open-sea cell, never onto a cell another channel takes, then the network's own smoothing, HYD-144) and end one cell into the open sea, as every sea mouth does. The water surface is the sea's on the plain.
10. *What is written.* `elevation` with the plain raised, `detail/amplitude` 0 on the plain, a new layer `delta` (i16, 1/16 block: what the plain added, 0 elsewhere), the network with its distributaries, and `notes.bake.water.stats.deltas`. A re-run (`deeptime hydro`) takes `delta` off the elevation first, so it starts from the same ground. A Lambert world's first horizon tops are raised again over the plain (a Petroff-Guyou or cube world's projection pass raises them anyway). On a Petroff-Guyou or cube world the projection pass gives a world cell whose nearest bake cell is on a plain that cell's `elevation` and `detail/amplitude`, and resamples `delta` by the nearest cell (HYD-146); after the land call it gives the world's nodes on a dry plain the land's biome record (BIO-185; "Stage 1b on the D5 world" below).

**The river file** (`rivers/rivers.dtrv`, FORMAT.md "Rivers"). A chain gains `from`: the chain it leaves at its first vertex (a distributary), or `u32::MAX` for a head. A chain is 20 bytes (first, count, mouth, 3 reserved, into, from), and the fixed block gains a u32 `layout` (2), 52 bytes. The world's `format_version` stays 0.6: 0.7 is the cube world's, so a polar 0.7 would collide, and the river file's own layout number refuses old files on both sides with a message naming both layouts (decisions log 2026-10-01: no compatibility before release). The validator checks that `from` names a chain and that the first vertex is a vertex of that chain. Java's reader (`worlddata/Rivers`) and the Rust one change together; the projection pass's chain splits (`project_rivers`, the cube's) carry `from` to the part that holds the vertex.

**The game.** Nothing new in the column rule: a distributary is a chain like any other, and the existing rule (the deep hit, the channel through the sea, HYD-120 to HYD-124) cuts it at the bake's width and depth with its water at the sea's surface, one block below the plain. `DeltaGameTests` on a synthetic delta world (`deeptime synth --kind delta-test`) checks the carving.

### Stage 1 on the D5 bake (measured 2026-10-23)

**How.** The D5 world's Lambert bake (the play world's, master d74c74aa, `rivers-d5/lam` on the test VM), copied without its epochs, had its water part run again: `deeptime hydro --world-data <copy>`, with `--set hydro.deltas=false` for before and `--set hydro.delta_area_mult=N --set hydro.delta_mouth_spacing=K` for the variants. Before and after were then projected to Petroff-Guyou (`deeptime project`) as the play world was. `tools/review/delta_census.py` counts each world's river file and layers. This is not a simulation: the water part reads the cached bake's ground, and the pool keeps no baked worlds, so it ran outside the pool (CLAUDE.md, "No simulation outside the pool"). The review page with the largest deltas, before and after, is the private space "deeptime-deltas" (`tools/review/spaces.json`).

The arms: mN is area multiplier N (HYD-141), sK is mouths K channel widths apart (HYD-138). m8s3 was the first default. m32s2 is the defaults before a split that missed the middle of its half tried the half's other front cells. m32s2r has that retry, but not yet the tail's free way to the sea or the split at the shore (HYD-140, HYD-144). After is the branch head's defaults: m32s2 with both.

| Measure | before | m8s3 | m16s2 | m32s2 | m32s2r | after | before, PG | after, PG | Earth |
|---|---|---|---|---|---|---|---|---|---|
| Plains (the bake's count) | 0 | 190 | 275 | 359 | 359 | 359 | | | |
| ... of loaded sea mouths | | | | | | 34 % (359 of 1,054) | | | 40 % of rivers at least 50 m wide (Caldwell et al. 2019) |
| Distributaries | 0 | 117 | 245 | 389 | 378 | 632 | 0 | 632 | |
| Rivers that split | 0 | 46 | 90 | 132 | 133 | 233 | 0 | 233 | |
| Q at least 50 m³/s: split of all | 0 of 58 | 25 of 60 | 31 of 62 | 38 of 62 | 38 of 61 | 39 of 60 | 0 of 58 | 39 of 60 | |
| Q at least 100: split of all | 0 of 33 | 15 of 35 | 20 of 34 | 22 of 34 | 23 of 35 | 22 of 34 | 0 of 33 | 22 of 34 | |
| Q at least 300: split of all | 0 of 10 | 3 of 9 | 4 of 9 | 5 of 10 | 5 of 10 | 5 of 10 | 0 of 10 | 5 of 10 | |
| 25 largest rivers: with a plain | 0 | 19 | 21 | 23 | 22 | 23 | 0 | 23 | near all (Caldwell et al. 2019: likelihood near 1 for the largest) |
| 25 largest rivers: split | 0 | 11 | 14 | 15 | 16 | 15 | 0 | 15 | |
| Plain raised, km² (the census's grid) | 0 | 2,571 | 4,691 | 8,453 | 8,453 | 8,453 | 0 | 10,390 | 847,936 (Edmonds et al. 2020) |
| Share of the land | 0 | 0.12 % | 0.22 % | 0.39 % | 0.39 % | 0.39 % | 0 | 0.49 % | 0.57 % (Edmonds et al. 2020) |
| Plain cells above the sea | | 96 % | 97 % | 97 % | 97 % | 97 % | | 67 % | |
| Largest raise, blocks | | 5.7 | 7.8 | 10.3 | 10.3 | 10.3 | | 9.5 | |

- **Mouths.** After, the 233 rivers that split end in 2 mouths (72 rivers), 3 (82), 4 (22), 5 (9), 6 (12), 7 (18) or 8, the cap (18). Of the 359 plains, 109 have a front that fits one mouth; of the 250 that fit two or more, 240 split. The 10 that do not: in 8 every front cell of the half the split heads for lies on the main stem, and 2 have no free way to the open sea (HYD-147; stage 1 counted the 8 as no path, see "Stage 1b on the D5 world").
- **The largest rivers.** Rank 1 (Q 2,745 m³/s) ends in 6 mouths and rank 2 in 8. Of the 25 largest, 8 have a plain but do not split. Among the bake's 10 largest plains, 3 hold one mouth because their front fits only one (Q 690, 478 and 475 m³/s). For example, the Q 690 river has a 37-cell front and a channel drawn 26 blocks wide. A fourth (Q 304) found no path from its stem. The rivers are drawn 40 to 70 times their physical width (HYD-141), so a big river's plain fills with its own channels.
- **How it got here.** The first version put the apex the backwater length up from the front: 4 of 190 plains split (14 distributaries). With the apex at the plain's entry, 51 plains split (117). Mouths 2 widths apart gave 71 (175). Area multiplier 32 gave 139 plains (389). The retry over the half's other cells split 1 more plain. The tail's shortest free way to the sea and the split at the shore gave 240 (632). Before the last step, 110 of the 250 plains that fit two mouths held one: 64 had a stem that touches the plain at one cell, 38 had no strictly descending way to the open sea past another channel's tail, and 8 had no path.
- **Petroff-Guyou.** 67 % of the projected plain's cells stood above the sea, against 97 % on the Lambert bake, and the projected plain counted more cells (29,927 against 23,481, a share of 0.49 %). Stage 1b traced it cell by cell to the projection's footprint mean and fixed it (HYD-146): 97 % now, over 24,313 cells ("Stage 1b on the D5 world" below).
- **Stacks.** The Lambert world's horizon tops are raised to the plain's surface on 989 cells. The raise is measured from each stack's old top, which lay up to 58 blocks below the new surface there; the plain itself raises the ground by at most 10.25 blocks. Strata are stretched to the new top, not moved.
- **Cost.** The water part with the deltas ran in 8.45 s on the VM (the second network run included).

### Stage 1b on the D5 world (measured 2026-10-23)

What a player would see before Ben looks at a world: (1) the Petroff-Guyou world's plains stood above the sea on 67 % of their cells against the bake's 97 %, (2) land on a plain could carry an ocean biome, (3) 8 plains that fit two mouths held one, counted as having no path. The area multiplier stays 32 (HYD-141); the wave and tide gate is left for later.

**How.** As for stage 1: the D5 bake's water part run again at the branch head (`deeptime hydro`), projected to Petroff-Guyou (`deeptime project`), and counted by `tools/review/delta_census.py`. "Stage 1" below is master 84fb7d7c as merged. The trace of (1) ran an example over the stage 1 world and its Lambert bake (not committed). The census now reads the biome inputs as the game does (the continentalness bilinear from L2 at each L3 column; ocean below −0.19), and it finds a river's plain anywhere along its main stem, not only near its mouth: rank 10's plain lies far up its stem inside a wide beach band, so 24 of the 25 largest rivers have a plain on both arms (23 in stage 1's count).

**(1) The plain under the sea on the world (HYD-146).** Of the 24,313 world cells whose nearest Lambert cell is on a plain, that cell stands above the sea for 23,585; the other 728 are the river's course, a block under the plain by design (HYD-136). The world kept 19,151 of them above the sea (78.8 %). The projection's footprint mean took 3,534 under it: by 0.10 blocks at the median and 0.37 at p90, 3,191 of them beside the open sea (whose deepest neighbour lay 0.62 blocks under the sea at the median), and 3,493 from 2 to 4 samples, so it is the plain's seaward edge blending with the beach band and the sea beside it, not a coarse footprint. 1,048 more had a mean above the sea and ended at or under it in the written layer. The mean of `delta` also spread the layer onto 5,747 cells beside the plain (939 of them above the sea), where it raised nothing, so the census counted 29,927 plain cells. The fix: a world cell whose nearest bake cell is on a plain takes that cell's `elevation` and `detail/amplitude`, and `delta` is resampled by the nearest cell everywhere (`bake/project.rs`, `bake/project_cube.rs`: `PLAIN_NEAREST`, `plain_rule`). The world's plain is now 24,313 cells, 97.0 % above the sea (the bake's 97.1 %), 8,440.6 km² against the bake's 8,453.2.

**(2) Ocean biomes on the plain (BIO-185; biomes.md, "Land on a delta plain").** The climate stage and the biome bake run before the water part lays the plains, so the bake's L2 nodes over a plain hold the sea's record: 90.6 % of the Lambert bake's plain columns read an ocean continentalness. On the world, the land call (BIO-170) gives land nodes the land's record but reads the ground at a node's block only, and the game reads the biome inputs bilinear between four nodes, so the plain's columns beside the sea still read a sea node: 752 of the stage 1 world's 29,473 dry plain columns (2.6 %) had an ocean biome. The fix runs the land call a second time with every node within one L2 cell of a dry plain cell counted dry (4,417 nodes; 271 of them, 1,505 km², took the land's record), so 0 of the 24,313 dry plain columns read an ocean biome. A Lambert world has no land call, so its plains keep the bake's ocean record (left as it is; the play worlds are Petroff-Guyou).

**(3) The 8 plains with no path (HYD-147).** None lacked a path: in each, every front cell of the half the split heads for lies on the main stem. The 10th largest river (Q 304 m³/s) runs over all 55 of its plain's front cells and no other plain cell: the second network takes it onto the plain off another land cell than the one the plain's course was built off, and it runs along the plain's edge beside the sea. The other 7 are Q 4 to 14 m³/s with fronts of 3 to 8 cells, of which the stem takes all but at most one. The fix is not local: for the Q 304 river it lies in how the plain's course is chosen (HYD-134, HYD-136), and the 7 small plains are strips a few cells long. The split is not changed; the record now counts the causes apart (`split_fails`, `fit_unsplit_plains`).

| Measure | Lambert bake (both) | Stage 1, PG | Stage 1b, PG |
|---|---|---|---|
| Plain cells | 23,481 | 29,927 | 24,313 |
| ... above the sea | 22,794 (97.1 %) | 20,008 (66.9 %) | 23,585 (97.0 %) |
| Plain, km² | 8,453.2 | 10,389.6 | 8,440.6 |
| Share of the land | 0.39 % | 0.49 % | 0.39 % |
| Largest raise, blocks | 10.25 | 9.5 | 10.25 |
| Dry plain columns with an ocean biome | 21,268 of 23,481 (90.6 %) | 752 of 29,473 (2.6 %) | 0 of 24,313 |
| Sea cells within an L2 cell of the dry plain with a coast or land continentalness | 126 of 4,705 | 386 of 5,027 | 2,947 of 4,587 |
| The whole world: dry columns with an ocean biome | 423,512 | 56,533 | 55,459 |
| The whole world: sea columns with a land biome | 47,257 | 21,110 | 24,037 |
| Dry plain columns in the swamp band (erosion E6) | 14,778 (62.9 %) | 18,727 (63.5 %) | 15,904 (65.4 %) |
| Distributaries; rivers that split | 632; 233 | 632; 233 | 632; 233 |
| 25 largest rivers: with a plain; split | 24; 15 | 24; 15 | 24; 15 |
| Plains that fit 2 mouths and hold 1 | 10 | 10 | 10 |

- **The cost of (2).** The sea's columns beside the plain read the plain's land record bilinear: 2,947 of the 4,587 sea cells (L3) within an L2 cell of its dry cells read a coast or land continentalness (386 in stage 1), about 1,020 km², 0.044 % of the land. That is the water's edge of a wetland; the sea there is the sea's water whatever biome it carries.
- **Wetland.** 65 % of the plain's dry columns lie in the swamp band, where the valley slice is swamp or mangrove swamp: the plains are wetlands, as deltas are. Left as it is.
- **Cost.** The projection ran in 6.77 s at the branch head against 8.07 s for stage 1 (noise: no measurable cost).

## Floodplains and levees (2026-10-24, HYD-150 to HYD-153)

V1 gap 5, stage 2 of four: the flat valley floors beside rivers that floods build, and the levees along their banks. Meanders (stage 3) and fans and braids (stage 4) are not in this section.

### Design

**Which ground is a floodplain (HYD-150, HYD-152).** GFPLAIN's rule (Nardi et al. 2019, read; the values from its code, `03-GFPLAIN.py`, read, and only read: the code is GPL and nothing of it is ported): a cell is floodplain where its ground lies at most a flood depth h over the channel cell it drains to, with h = a A^b metres, a = 0.01, b = 0.30 and A the area draining to that channel cell in m² (h is about 2 m at 50 km², 5 m at 1,000 km², 10 m at 10,000 km² and 20 m at 100,000 km²). The bake runs it in the last network run of the water part, on the hydrology's grid (Lambert L3, 2 blocks a cell): each land cell's drain is the first river cell down its receivers, none where the way meets the sea, a lake a block deep or a sink first; its height above that drain (HAND) is the difference of the two cells' filled heights. A cell is plain when HAND is at most h. Not plain: lake and sea cells, the delta plains (laid already, stage 1), and the floor of a basin that holds no lake (ground a block or more under its filled surface: laying a plain there would fill the basin the lake balance left dry; 138 cells on D5). A river cell beside the plain is part of it, so the channel is cut into it.

**Where it is laid (HYD-151).** h is under one 35 m block for every river on the planet, so the plain's cells already stand within a block of their river. What the plain changes is that it is flat and stands where a bank stands: each plain cell takes its drain's bank level, the drain's filled height but at least one block over its water (the plain's top block one above the water's top block, as the column rule's banks stand, HYD-124). A cell lower than that level (a hollow a fraction of a block deep beside the channel) is filled to it, as overbank deposition fills a floodplain; a cell higher (by at most h) is lowered to it. The detail amplitude is set to 0 on the plain (a floodplain is flat), and the layer `floodplain` keeps what the plain moved the ground by (i16, 1/16 block, NODATA off the plain), so a re-run of the water part takes it off first and ends where it ended (checked on D5: the re-run's `elevation`, `floodplain`, `delta`, `detail/amplitude`, `water_level` and horizon stacks hash the same). On a Lambert world the horizon stacks' first tops are raised over the cells the plain raised, as for the deltas; a projected world resamples `floodplain` by the nearest cell (a categorical layer, as `delta`, HYD-146).

**Levees (HYD-153).** Natural levees are low wedge-shaped ridges of channel-borne sediment, "typically highest and coarsest at the channel margins", and their widths "vary from a fraction to over 10 times the channel width" (Smith et al. 2009, read); levee and proximal floodbasin sediment ranges from very fine sand to medium silt (Filgueira-Rivera et al. 2007, read). In the game a river's bank columns (the column rule's bank zone, HYD-024: within the channel's half-width plus one block plus half the half-width, or beside its water) that lie on the floodplain get sand for a soil top (grass, podzol, mycelium, dirt, coarse or rooted dirt; never mud, snow, sand, gravel or clay). The band is the bank zone the rule already draws (game-choice: no new width): 1 + hw/2 blocks beyond a channel of half-width hw, so 0.27 to 0.96 of the drawn channel's width (from the widest channel, 64 blocks, to the smallest river's 1.4, HYD-001, HYD-009), inside the 0.15 to 1.07 levee widths per channel width of the Upper Columbia and under the Lower Saskatchewan's 0.41 to 25.96 (Adams et al. 2004, read, Table 1). No levee ridge is raised: the plain is one block over the water, and a ridge a block higher than the plain would be 35 m. Far terrain (Distant Horizons) does not draw the sand (left for later). The first build laid no sand at all: it asked for the column's covering channel, which a bank column never has (a column inside a channel is a river column, not a bank), and the GameTest gate on master 1e27a44f caught it (404 of 404 bank columns on the plain kept their soil); since 2026-10-24 the sand keys on the bank zone's cap, the mark of a bank column.

**What is written.** The layer `floodplain` (FORMAT.md), `notes.bake.water.params.floodplains` (preset key `hydro.floodplains`, on by default) and `notes.bake.water.stats.floodplains`. Java reads the layer through `WorldData.onFloodplain`; `WaterBodies` lays the levee's sand. `DeltaGameTests.leveesAreSandOnTheFloodplainOnly` checks it on the delta-test world, which now writes a floodplain over its river's upstream reach.

### On the D5 world (measured 2026-10-24)

**How.** As for the deltas: the D5 world's Lambert bake (`rivers-d5/lam` on the test VM) had its water part run again at the branch head with `--set hydro.floodplains=false` (before) and with the defaults (after), and each was projected to Petroff-Guyou (`deeptime project`; both chose λc 120.25). This is not a simulation (the water part reads the cached bake's ground). `build/deltas/fp_census.py` (not committed) counts the projected worlds; the bake's own record is `notes.bake.water.stats.floodplains`. The review page is the private space "deeptime-floodplains".

| Measure | D5 | Earth |
|---|---|---|
| Floodplain, km² (the bake) | 167,243 | 13,394,139 (GFPLAIN, basins over 1,000 km², 60°N to 60°S; the figure as Lane et al. 2023 quote it, read: GFPLAIN's own table did not load) |
| Share of the land | 7.2 % (6.5 % within 60° of the equator) | about 11.5 % for GFPLAIN (its area over the land between 60°N and 60°S, the land area from memory); 19.7 % in Lane et al. 2023's own map (26.6 million km², read), and about 21 % for their map cut to GFPLAIN's latitudes with a 20 km² minimum drain (24.2 million km², read; the planet's rivers start at 50 km²) |
| Share of the land, drains of at least 1,000 km² | 4.97 % (plain 5.3 km wide per km of channel) | GFPLAIN's cut-off |
| ... at least 10,000 km² | 2.78 % (14.9 km) | |
| ... at least 100,000 km² | 0.28 % (13.5 km) | |
| River cells in the plain | 60,383 | |
| Delta plain cells left out | 26,197 | |
| Cells raised (overbank fill) / lowered | 302,210 (at most 2.31 blocks) / 90,586 (at most 0.75) | |
| Top block moved: up 1 / unmoved / down 1 / up 2 or more | 243,204 / 214,003 / 7,146 / 212 | |
| Plain cells whose noise was cleared | 99,906 | |
| The projected world: plain cells, km² | 481,777 cells, 167,256 km² (7.25 % of its land) | |
| ... detail amplitude on the plain, mean blocks (before, after) | 0.024, 0.006 (zero on 77.5 % and 94.2 %) | |
| ... where the plain lies: under a drawn channel / in its bank zone / beyond | 27.0 % / 18.3 % / 54.6 % (91,352 km²) | |
| ... plain cells within 8 blocks of a bank zone whose top block stands over the channel's water top block by at most 0 / 1 / 2 / 3 or more, before | 125,559 / 92,990 / 3,909 / 613 | |
| ... after | 12,334 / 208,004 / 2,121 / 612 | |

- **Against Earth.** The planet's floodplains are about half of GFPLAIN's share by the same rule, and a third of Lane et al.'s with a minimum drain near the planet's. The rule is GFPLAIN's own and nothing was tuned to an Earth figure (decisions log 2026-10-23); the planet's rivers drain smaller areas than Earth's (h depends on A only) and its valleys come from the stream-power solve, so a smaller share is what the physics gives.
- **The raise.** HAND is under a sixteenth of a block on 276,448 cells: flats and hollows at their drain's filled level, flush with the river's water before. Laying the plain one block over the water lifts them by up to a block (2.31 at most, the fill of a hollow), and 243,204 cells' top block rises by one. That is the change a player sees: before, 56 % of the plain beside a channel stood flush with the water or under it (the bank rule then raised a one-column bank); after, 93 % stands one block over it, as a bank does.
- **Stacks.** On the Lambert bake the horizon stacks' tops were raised on 2,924 cells, by up to 34 blocks measured from each stack's old top (the existing `raise_world_tops` behaviour also seen for the deltas, whose maximum is 58); the plain itself raises the ground by at most 2.31 blocks.
- **Cost.** The floodplain step took 1.24 s of the water part on the VM.

## Meanders (2026-10-24, HYD-155 to HYD-159)

V1 gap 5, stage 3 of four: a river on a floodplain winds across it. Fans and braids (stage 4) are not in this section.

### Design

**Why.** The network's chains follow the drainage grid's cells, smoothed: straight down every valley (sinuosity 1.035 at the median before this stage, HYD-027), where Leopold & Wolman's rivers have sinuosities "dominantly between 1.1 and 2.0" (Langbein & Leopold 1966, read) and a reach of 1.5 or more is what they call a meander (Leopold & Wolman 1957, read).

**Where (HYD-158, HYD-159).** The bake's water part redraws, after the floodplains, each stretch of a chain whose channel lies on the floodplain with its water no higher than the plain's top block (the plain is laid at the bank, HYD-151), whose course over a wavelength is gentler than Leopold & Wolman's (1957) line between meandering and braided channels, s = 0.06 Q^−0.44 with Q in ft³/s (0.0125 Q^−0.44 in m³/s: 0.0055 at 10 m³/s, 0.0020 at 100, 0.00071 at 1,000), and that keeps away from both ends of its chain and from a distributary's start. Q is the network's mean discharge, standing in for the bankfull discharge the line was drawn for, and the slope is the valley's, standing in for the channel's (both below, open questions). Across the course the plain's room is probed, 1 block of planet a step, to the plain's edge or half way to another river's bank zone (a direct tributary within its join radius does not count). The other river is looked for twice as far as the belt can reach (twice the cap's full amplitude plus the half-width), past the plain's edge too: each of two neighbours then takes at most half the gap between them, however far their belts are pushed towards each other (the first D5 census found 40 new crossings, on the planet, between neighbours whose belts were both pushed into a gap the probe had not looked across). The two bounds are eroded over a wavelength and averaged over half of one, and the belt is placed inside them as close to the old course as the room allows. A stretch at least a wavelength long with at least a block of amplitude is drawn, tapered back to the old course over half a wavelength at each end.

**Shape (HYD-156, HYD-157).** Langbein & Leopold's (1966) sine-generated curve: the channel's direction off the valley's is φ = ω sin(2π s / M) along its own length s, M = k λ for a downvalley wavelength λ; the sinuosity is k = 1 / J₀(ω) and the half amplitude across the valley a = λ H₀(ω) / (4 J₀(ω)). The code integrates the curve once into a table of its offset by phase and ω; the unit test checks it against the paper's table of ω and k (0.5, 1.0, 1.5 and 1.75 rad give 1.066, 1.307, 1.954 and 2.710 against the printed 1.06, 1.30, 1.96 and 2.67; the last is 1.5 % over the printed value, the others agree to the paper's rounding). The amplitude is what the room allows, up to sinuosity 2 (ω = 1.52 rad, a = 0.36 λ, a full amplitude of 7.8 drawn widths): Leopold & Wolman (1960) found the amplitude "determined more by erosion characteristics of stream banks and by other local factors than by simple hydraulic principles", with "a limit to the amplitude of meanders ... provided by the formation of cut-offs", so it is not tied to the width here.

**Wavelength (HYD-155).** λ = 10.9 w (Leopold & Wolman 1960, Table 1: L = 10.9 w^1.01; the text's "7 to 10 times the channel width"; Leopold & Wolman 1957, Fig. 45B: 7 for small streams to 15 for large rivers). w is the drawn width, 2 hw: the channels are drawn wider than a 300 m block would give, so a wavelength from the true width would be a few blocks; from the drawn width the bends keep their proportion to the channel a player sees (a game choice, below).

**The planet's metric.** Everything is measured on the planet: on a Lambert folder a block east is cos φ of the planet's and a block south 1 / cos φ (round-features.md, SCL-230), so a belt is as wide and its wavelength as long at 55° as at the equator (the unit test `the_belt_is_the_same_on_the_planet_at_the_equator_and_at_55_degrees`). A projected folder's own grid is the world's, and its metric is the identity. A baked Lambert folder carries a layout block of kind `lambert` (a fresh bake's manifest has none until the bake ends); the water part took any layout block for a projected world, so `deeptime hydro` over a baked folder measured the meanders (and the playas' dry margin, BIO-178) in the grid's blocks, and the first D5 census found the wavelength 1.4 times 10.9 widths on rivers running north-south at 55° to 70°. The water part now asks for the kind (`water::lambert_frame`; the bake test `a_lambert_preset_keeps_the_cylinder` drains a baked folder again and gets the bake's river file).

**What changes.** Only the river file: the chains keep their order and get more vertices (a 40th of a wavelength apart, 1 to 6 blocks); the water surface along a redrawn stretch is the old course's at the same downvalley distance, so it never rises downstream. No layer and no ground change: the plain is one block over the water wherever the new channel runs, and the old course is plain like the rest (the game cuts channels from the river file). A tributary whose parent moved under its mouth is cut where it first meets the parent's new channel, or carried on to it, and ends on a vertex of the new channel; distributaries stay as they were. `hydro.meanders` is on by default (with the floodplains); `notes.bake.water.stats.meanders` is the record (FORMAT.md). The meander-test synthetic world (`deeptime synth --kind meander-test`: a straight river down a wide flat plain) and `MeanderGameTests` check the game's side: the river winds, its channel holds its water at the plain's top block less one, and the old course is dry plain.

### On the D5 world (measured 2026-10-24)

**How.** As for the floodplains: the D5 world's Lambert bake (`rivers-d5/lam` on the test VM) had its water part run again at the branch head with `--set hydro.meanders=false` (before) and with the defaults (after), and each was projected to Petroff-Guyou (both chose λc 120.25). This is not a simulation. The re-run of the "after" folder's water part gives the same river file and the same `elevation`, `floodplain` and `water_level` (hashes compared). `build/deltas/mn_census.py` (not committed) measures the redrawn runs on the planet (a run is a stretch of a chain at least 1.5 blocks off its old course; lengths in planet blocks, with the Lambert metric) and counts the projected worlds; the bake's own record is `notes.bake.water.stats.meanders`. The review page is the private space "deeptime-deeptime-meanders".

| Measure | D5 | Earth |
|---|---|---|
| Plain course (candidates' length) | 39,298 km, of which 8,021 km steeper than the braided line, 23,528 km with less than a block of room, 740 km in stretches shorter than a wavelength | |
| Meandered | 6,762 km of valley (17.2 % of the plain course) on 196 chains, 200 reaches; 10,597 km of channel | |
| Reach sinuosity (the bake, length-weighted) | 1.567; reaches by sinuosity, edges 1.05, 1.1, 1.3, 1.5, 2: 10, 12, 41, 36, 100, 1 | "dominantly between 1.1 and 2.0" (Langbein & Leopold 1966, p. H5); 1.5 or more is meandering (Leopold & Wolman 1957) |
| Runs on the planet: sinuosity p10 / p50 / p90 | 1.037 / 1.598 / 2.024 (279 runs; 1.67 weighted by length) | as above |
| Wavelength over 10.9 drawn widths, p10 / p50 / p90 | 0.925 / 0.991 / 1.002 (95 runs long enough to measure) | 10.9 w^1.01 (Leopold & Wolman 1960, Table 1) |
| ... by latitude, p50: 0-30°, 30-55°, 55-70° | 0.964, 0.989, 0.997 (the first census, before the frame fix: 1.40 at 55-70°) | |
| Full amplitude over the drawn width, p50 / p90 | 8.15 / 13.6 (twice the largest offset off the old course, so the belt's shift is in it) | 2.7 w^1.1 (Leopold & Wolman 1960, Table 1, a poor fit by their own account) |
| Mean / largest amplitude | 7.5 / 50.6 blocks | |
| By discharge, < 10 / 10-100 / 100-1,000 / ≥ 1,000 m³/s: plain course, km | 21,670 / 14,030 / 2,953 / 645 | |
| ... meandered share of it | 23.0 % / 10.6 % / 10.3 % / 0 | |
| ... sinuosity of the meandered reaches | 1.625 / 1.451 / 1.187 / none | |
| Whole network | 92,293 km to 95,974 km (× 1.040) | |
| Tributary joins on a moved parent | 58 cut where they meet the new channel, 46 carried on to it (at most 40.3 blocks) | |
| The projected world: network length | 95,339 km to 100,888 km (× 1.058; the world's lengths are 1.033 times the planet's) | |
| ... vertices off the old course | 21,202: 99.42 % on the plain, 35 in the sea | |
| ... their ground's top block over the water's top block: < 0 / 0 / 1 / 2 / 3 or more | 38 / 19 / 21,112 / 14 / 19 | |
| ... crossings of two chains / of a chain with itself, before and after | 665 to 669 / 0 to 4 (the first census: 40 more between neighbours, before the neighbour fix) | |
| Cost | 384 ms of the water part | |

- **Against Earth.** The meandered reaches sit in Langbein & Leopold's 1.1 to 2.0, at a median of 1.6, and their wavelength is the 10.9 widths it was drawn with at every latitude (within 4 % at the median; the low tail is short runs whose ends taper). The share of the network that meanders is small (the whole network's sinuosity 1.04): most of the plain course is too narrow for a belt, and a fifth of it is steep enough to braid (stage 4).
- **The largest rivers do not meander.** None of the 645 km of plain course on rivers of 1,000 m³/s or more is redrawn, and none of it is steep. Their drawn half-widths are 19.7 blocks at the median, so a wavelength is 430 blocks (127 km) and the capped belt 310 blocks across. Probed across the course with `build/deltas/mn_big.py` (not committed): the plain alone, eroded over a wavelength as the bake does, leaves a block of amplitude on 247 of 635 km; on the rest another chain's bank stands inside the belt's reach (at 88 % of these vertices another chain's bank is within 2 blocks of the river's own, mostly tributaries near their mouths, and a direct tributary counts as an obstacle past its join radius). The split between the two was not measured. The cause is the drawn scale: the channels are drawn wider than a 300 m block gives (HYD-009), and the wavelength follows the drawn width, so the belt of a large river no longer fits between its plain's edges and its tributaries. An open question below.
- **Crossings left.** Four of a chain with itself (chain 954 at segments 163 and 165, chain 3623 at 178 and 180, and chain 1151 twice at 12°, segments 94 and 140, 100 and 134; why was not traced) and four between a tributary and its parent (a short tributary whose parent's new channel sweeps over its source, and three that meet the parent's belt at their own end). They are 8 points on a network of 100,000 km; a player would see a bend that touches the next one or a tributary entering at a different point.
- **Open questions.** (1) Mean discharge stands in for the bankfull discharge of Leopold & Wolman's line, and the valley slope for the channel slope; both move reaches across the braided line. Recommendation: keep both until stage 4 draws braids, then measure the steep share again. (2) The wavelength from the drawn width keeps the bends in proportion to the channel a player sees but stops the largest rivers meandering. Recommendation: keep it, and let a belt sweep over its direct tributaries' lower courses (cutting them where they meet the new channel, as the join rule does within the join radius) if the large rivers should wind; that is a design change, not a tuning. (3) The 8 residual crossings. Recommendation: leave them (a follow-up could stop a taper at the first sample where it would come within a width of its own earlier course).

## Fans and braids (2026-10-24, HYD-165 to HYD-175, HYD-177)

V1 gap 5, stage 4 of four: where a river leaves steep ground for gentler it builds an alluvial fan, and a river on a fan, or steep on its floodplain, divides around islands.

### Design

**Why.** Before this stage a river came out of a range in one channel and ran on across the plain below at the ground's own slope: the range front met the plain with no fan, and every chain was one channel from its source to its mouth (only a delta's distributaries split). Bull (1964, read) describes the fan as "a segment of a cone that radiates downslope from the point where the stream channel emerges from a mountainous area", with the channel dividing on it "into several braided distributary channels"; Leopold & Wolman (1957, read) call a river braided where it "flows in two or more anastomosing channels around alluvial islands", steeper, wider and shallower than an undivided reach carrying the same flow.

**Where a fan is (HYD-167).** The bake's water part looks along each chain of its first network run for an apex: a cell where the river's filled surface falls at least the fan's own slope S_F over the fan's free length upstream and less than S_F over the free length downstream (run on down the river's parents where the chain ends first). Upstream steeper than the fan is Bull's "steep fans form at the mouths of canyons having steep longitudinal profiles" (p. 100); downstream gentler is the slope break Ganti et al. (2014, read) put the Huanghe's fan node at ("avulsions occur at an abrupt decrease in bed slope", "where upland channel confinement ceases (e.g., at a canyon-fan transition)"), and it is what lets the cone stand on the ground below. Of the cells that qualify on a chain, the apexes are taken where the cone would stand thickest, at least a free length apart along the chain.

**Its size and shape (HYD-165, HYD-166, HYD-168).** Bull's laws for the fans of western Fresno County, for the drainage area A_D at the apex in square miles: the fan's area A_F = 1.3 A_D^0.88 and its overall slope S_F = 0.022 A_D^−0.32 (Fig. 54, the sandstone basins; the mudstone and shale basins' fans are about twice as large, 2.4 A_D^0.88, over basins of 0.2 to 29 mi² only, and the bake has no lithology for a basin). The fan is a cone of that slope about its apex, of one slope where Bull's are segmented and gently concave (the segments differ by a fraction of a block over a fan's length; a game choice). A free fan spreads over a quarter circle: the median sector of Stock et al.'s (2008, read) four Mojave fans, 2 A / L² from their areas and arc lengths, is 1.63 rad. Its free length L_free is the radius whose quarter circle covers A_F, and a free fan stands S_F L_free over its apex.

**Its extent (HYD-169, HYD-170).** The cone is laid on the cells joined to its apex where it stands over the ground, as far as land goes: no lake or playa, no ground below the sea's level outside a closed basin, no delta plain (laid first), and no channel of a river that drains more than the fan's own ("The radii of a fan are restricted by adjacent fans, and by stream deposits at the toe of the fan", Bull 1964 p. 94). A fan is also nowhere thicker than over its apex: a cell joins only where its ground lies no lower than a cone of the fan's slope through the apex's ground, so where the ground falls away from the apex faster than the fan (off a plateau's edge, into a deeper valley) the fan ends (the unit test `ground_falling_away_faster_than_the_fan_ends_it`). Without it the first census on the D5 world had cones standing up to 39 blocks (1.4 km) over coastal lowland beside the high ground their apexes were on, and a tenth of the fan cells raised by more than 27 blocks. Its height over the apex is found by bisection so that it covers Bull's area (on a Lambert frame the cells are of equal area, so the area is a cell count), but it is at most the free fan's height: a fan hemmed in by higher ground or a larger river is smaller than Bull's, never taller than a free fan. Where fans overlap the higher cone wins. A fan less than a block (35 m) high at its apex is not drawn (a game choice: thinner is under the world's resolution). The cone falls from its apex in every direction, upstream too, and with the straight distance, not along the ground's drainage, so it closes hollows: in the canyon above its apex (the cone falls upstream while the canyon floor rises), where a river bends back towards the apex, and where it buries the way out of lower ground beside it (a hole the cap or a larger river's channel leaves in it, a valley it dams). So the spill level of every cell is found before the fans and with them (a Priority-Flood from the open sea and the grid's end rows), and a cell whose spill the fans raised over its ground is filled to it, as the fan's deposit and the river's behind it would fill it, unless the first network had a lake or a playa there (that lake only rises; HYD-177, a game choice; the unit test `a_hollow_the_fans_close_is_filled_to_its_spill`). Without it the D5 world had 17,965 km² of new lakes, nearly all wholly new and touching a fan, mostly under a block deep, which the world cannot show (rivers ended in them on dry ground).

**In the bake.** The water part takes off a previous run's fans (the layer `fan`) before it starts, finds the fans on the first network run's chains after the deltas, raises the ground to the cones, and runs the network again (the run that already ran for the delta plains), so the rivers drain down the cones; no floodplain is laid on a fan. On the bake's L3 grid each cell takes the cone at its own position (so a fan is round at L3's spacing too), `detail/amplitude` is 0 there (the cone is the fan's surface), the raster `fan` keeps what the cones and the fill added (a re-run takes it off first and ends where it ended), and on a Lambert world the horizon stacks' first tops are raised over the fan, as for the deltas and floodplains. The game does not read `fan`; it reads the raised `elevation`.

**Where a river braids (HYD-171).** After the meanders, along each chain that is not itself a distributary or a thread: every reach on a fan, and a reach on the floodplain (its bank at or over its water) whose water-surface slope over an island's length is above Leopold & Wolman's line between meandering and braided channels, s = 0.0125 Q^−0.44 (Q in m³/s; HYD-158: the reaches the meanders leave as too steep). Nanson & Croke's (1992, read) braided floodplains are their class B, 10 to 300 W m⁻² of specific stream power at bankfull, with "moderately steep gradients and usually easily erodible banks"; the bake has no bankfull power, so the class is context, not a rule.

**Threads and islands (HYD-172 to HYD-175).** Around each island the channel is two threads, each 0.9 of the undivided channel's width and 0.75 of its depth at the island's middle (Leopold & Wolman 1957, p. 50: the divided channels' widths sum to 1.6 to 2.0 times the undivided one's in their natural rivers, and their depth is 0.6 to 0.9 of it), each carrying half the flow (a game choice: their one measured split, the Green River's 305 and 120 of 425 ft³/s, is no rule). An island is 5 undivided widths long (Horse Creek's central bar, "nearly 250 feet" in a channel of 50, p. 41) and one width across at its middle, a lens (a game choice: the sources read give no width); the islands follow one another down a braiding reach on alternate sides of the course, so the river weaves between them. A channel narrower than 3 blocks does not braid (its threads and island would merge in the world's blocks). A thread is drawn only where its bank lies on the floodplain at or over its water, or on a fan, and no other channel's bank zone lies within twice its reach across the course; otherwise the island goes to the other side, or is not drawn.

**What changes in the river file.** No format change. The course stays one chain, narrowed and shallowed around each island and carrying the other half of the flow there; each island's other thread is a chain of its own appended after the old ones, with `from` and `into` both the course's chain and mouth `River`, its first and last vertices exact copies of two of the course's vertices, and its water surface the course's at the same distance down it, never rising. On a fan the cone falls away to either side of a course down its axis (a thread 15 blocks off it, 20 blocks below the apex, runs on ground half a block lower on a cone falling 0.1 a block), so there the thread's water is lowered by as much as the cone under the thread lies below the cone under the course: it stands as far under its own ground as the course's under the course's (the unit test `a_thread_on_a_cone_keeps_its_water_under_the_ground`). The game carves each thread as it carves any chain; the island between is the plain or the fan left standing.

**The planet's metric.** Everything is measured on the planet: on a Lambert folder a block east is cos φ of the planet's and a block south 1 / cos φ (round-features.md, SCL-230), so a fan is round and an island as long and as far off its course at 55° as at the equator (the unit tests `a_fan_is_round_on_the_planet_at_55_degrees` and `the_islands_are_the_same_on_the_planet_at_55_degrees`).

**Switches and the record.** `hydro.fans` and `hydro.braids` are on by default (physics on once built and correct, the decisions log of 2026-10-23); `notes.bake.water.stats.fans` and `stats.braids` are the record and the water part's method is "hyd-165" (FORMAT.md). The fan-test synthetic world (`deeptime synth --kind fan-test`: a ridge, a cone fan at its foot round on the planet, and a straight river down the fan, onto a plain and into the sea, braided by the bake's own braid pass) and `FanGameTests` check the game's side: the threads leave and rejoin the course on the fan, hold their water around dry islands, and the fan's ground is the cone.

### On the D5 world (measured 2026-10-24)

**How.** As for the meanders: the D5 world's Lambert bake (`rivers-d5/lam` on the test VM) had its water part run again at the branch head with `--set hydro.fans=false --set hydro.braids=false` (before) and with the defaults (after), and each was projected to Petroff-Guyou (both chose λc 120.25). This is not a simulation. The re-run of the "after" folder's water part gives the same river file and the same `elevation`, `floodplain`, `fan` and `water_level` (hashes compared). `build/deltas/fb_census.py` (not committed) measures the fans on the planet (the `fan` layer's raised cells in 4-connected groups of at least 12), the threads against the design, the lakes before and after, and the projected world. The page: https://spacesheep.dev/@benmac/deeptime-deeptime-fans-braids (private).

| Measure | D5 | Earth |
|---|---|---|
| Fans (the bake's record) | 818 of 20,564 apexes looked at (22 too thin, 517 hemmed in before Bull's area); 206,323 km², 8.9 % of the land; 13,332 km³ | |
| Fan area, p10 / p50 / p90 / max | 56 / 170 / 899 / 14,772 km² | Bull's fans: 0.5 to 296 mi² of drainage (his fit's range); 193 of the 818 drain more |
| Fan area over Bull's, p10 / p50 / p90 | 0.35 / 0.82 / 1.00 (hemmed fans are smaller, none larger) | 1 |
| Drainage at the apex, p10 / p50 / p90 / max | 104 / 304 / 2,094 / 99,942 km² | |
| Slope, p10 / p50 / p90 | 0.003 / 0.005 / 0.007 | Bull's 0.0035 to 0.029 |
| Height over the apex, p50 / p90 / max | 2.18 / 2.78 / 4.62 blocks | |
| Hollows the fans closed, filled (HYD-177) | 88,207 cells, 31,755 km², at most 3.06 blocks, 475 km³ | |
| Ground raised in the bake (L3) | 581,290 cells, at most 4.56 blocks; `detail/amplitude` zeroed on 389,247 | |
| Shape on the planet, groups north-south over east-west, p50: 0-30°, 30-55°, 55-70° | 1.06, 1.53, 1.94 (on the grid 1.04, 0.68, 0.41 against cos² φ 0.93, 0.54, 0.21) | |
| Braids (the bake's record) | 247 islands and threads; 2,048 km braided (2,028 km of it on fans) of 21,599 km of course on fans and 5,285 km steep on the plain; 11,647 km too narrow (under 3 blocks), 858 islands without room | braided reaches on fans (Bull 1964) and above s = 0.0125 Q^−0.44 (Leopold & Wolman 1957) |
| ... by discharge, < 10 / 10-100 / 100-1,000 / ≥ 1,000 m³/s | 197 / 1,572 / 279 / 0 km braided of 12,363 / 11,533 / 2,728 / 260 km braiding | |
| Island length over 10 half-widths, p10 / p50 / p90 | 0.97 / 1.03 / 1.10 (the design 1) | Horse Creek's bar, 5 widths (Leopold & Wolman 1957) |
| Thread offset over 3.8 half-widths, p10 / p50 / p90 | 0.69 / 0.98 / 1.17 | |
| Lakes, before and after | 60,010 to 60,222 km²: 407 km² new (12 km² on fans), 194 km² lost; 14 wholly new lakes, 401 km² | |
| Rivers ending in lake water under a block deep (which the world cannot show) | 53 (1,914 m³/s) to 35 (1,786 m³/s) | |
| The projected world: ground raised | 678,270 cells (10.2 % of the land), p50 / p90 / max 1.0 / 2.44 / 6.44 blocks; 27,554 of them off the fan layer, 50,752 lowered (the second network's floodplain and channels where the rivers moved) | |
| ... network length, chains | 100,888 to 91,782 km; 4,599 to 4,526 | |
| ... thread vertices: ground's top block over the water's, < 0 / 0 / 1 / 2 / 3 or more | 0 / 12 / 5,708 / 840 / 145 (none in the sea) | |
| ... course vertices on the fans: the same | 57 / 551 / 32,761 / 2,979 / 332 (the courses there before: 47 / 1,006 / 40,899 / 1,341 / 258) | |
| ... crossings of two chains / of a chain with itself | 669 to 406 / 4 to 1 | |

- **Against Earth.** The fans are Bull's where they have room: the median fan is 0.82 of Bull's area for its drainage, the 301 not hemmed in are his, and none is larger; their slopes, 0.003 to 0.007, sit at the gentle end of his 0.0035 to 0.029, as his law gives for their drainage (a median 304 km², 117 mi²). A ninth of the land is fan, and on it the rivers braid: 2,028 of the 2,048 braided km are on fans, while the plain's steep reaches mostly lack the width (a channel under 3 blocks does not braid). The threads are drawn as designed on the planet at every latitude (island length and offset within 3 % at the median).
- **The fans' hollows.** The first census of this stage had 17,965 km² of new lakes, nearly all wholly new and touching a fan, mostly under a block deep: the cone falls from its apex upstream too and with the straight distance, so it closed hollows in the canyons above the apexes, where rivers bend back towards an apex, and where it buried the way out of lower ground beside it. A rule that kept the channels of rivers from beyond the fan open (commit f7551b29, reverted in 529c7467) made it worse (22,940 km²). Filling the hollows the fans close to their spill (HYD-177) leaves 407 km² of new lakes, 12 km² of them on fans, and fewer rivers end in water the world cannot show than before the fans (35 against 53).
- **Shape at high latitude.** The fan groups' planet ratio rises with latitude (1.53 at 30-55°, 1.94 at 55-70°), where a free fan is round on the planet at 55° in the unit test `a_fan_is_round_on_the_planet_at_55_degrees`. Most fans are hemmed in (517 of 818), so a group follows its valley and touching fans merge into one group; why the valleys lean north-south there was not traced.
- **Open questions.** (1) Bull's laws are for the sandstone basins; the mudstone and shale basins' fans are about twice as large (2.4 A_D^0.88) and the bake has no lithology for a basin. Recommendation: keep 1.3 until a basin's rock is in the record. (2) 193 fans drain more than Bull's fitted range (up to 99,942 km², a 14,772 km² fan), megafans his law was not fitted to. Recommendation: keep the extrapolation, capped by the free fan's height as now, and look for a megafan source (Leier et al. 2005 is the one to read) before changing it. (3) The cone has one slope where Bull's fans are segmented and gently concave. Recommendation: keep it (the segments differ by a fraction of a block over a fan's length). (4) A fan's surface is the ground's own blocks, not gravel. Recommendation: a surface block rule for fans with the strata work. (5) Each thread carries half the flow. Recommendation: keep it. (6) The fill makes flats at a hollow's spill (31,755 km², at most 3.06 blocks deep) where a graded deposit would slope. Recommendation: keep the flats; a graded fill would need the drainage distance the cone lacks. (7) 43 % of the D5 world's lake cells were under a block deep before this stage (a world the game cannot show; the rivers ending in them end on dry ground). Not this stage's; recommendation: a lake shallower than a block is shown as wet ground or deepened to one block by the game's lake carve, a decision for the lakes owner.

## Navigable rivers and deltas on the D5 world (measured 2026-10-22)

Ben's reports of 2026-10-22: "I am yet to see a navigable river / interior waterway within the modpack" and "Do we have river deltas, I'm yet to see one?". This section only measures. No code was changed, and the live play server was read, never written. The world is D5, the play world on master d74c74aa (tag `play-d5`, workbench run 1a16576c9de, format 0.6, Petroff-Guyou, 13,312 blocks round, 294.6 m per block, 35 m per block of height).

Words used below. A **main stem** runs from a sea mouth upstream along the donor with the larger discharge at every junction, through lakes. **Navigable** means the bake's channel is 8 or more blocks wide and 3 or more deep (the vertex's width 2 hw and depth, read from the bake; the game does not yet generate that depth, see "Depth in the game"). A **step** is a place where the water surface's top block, ceil(w) - 1, rises by one block going upstream: a one-block waterfall in the game. **Earth-equivalent** is a value times 66.33, Earth's area (510.07 million km2, from memory) over this planet's (7.690 million km2, radius 782.3 km); a length is scaled by the square root of that, 8.14.

### Short answers

1. **Sizes.** Seen against its own size the world is not short of big rivers. Seen in absolute terms it is, and all of that is the planet being 66 times smaller in area. The largest river carries 2,745 m3/s and is 52 blocks wide and 11.9 deep at the mouth (catchment 256,870 km2). Scaled by area that is 182,000 m3/s, 87 % of the Amazon's 209,000. Sea-reaching rivers of at least 50 / 100 / 300 / 1,000 / 3,000 m3/s number 57 / 33 / 10 / 2 / 0; Earth has 45 / 25 / 7 / 1 / 1 at the area-scaled thresholds (3,315 / 6,630 / 19,890 / 66,300 / 198,900 m3/s). Ben's single-player world has 48 / 30 / 5 / 1 / 0 at the same absolute thresholds, on a planet of the same size.
2. **Navigable reaches.** By the bake's own width and depth the 25 largest are navigable for 33 to 781 km (132 to 2,641 blocks; median 115 km). Only ranks 1 and 2 pass 500 km. The water surface steps down one block 459 times in the 17,640 blocks of main stem examined (one step per 38 blocks, 11 km). The first step lies 7 to 545 blocks from the mouth (median 129 blocks, 38 km); for 6 rivers (ranks 9, 10, 20, 21, 22, 23) it lies above the navigable reach. The steps are the real fall of the terrain: the surface falls 450 blocks over the lowest reaches against the terrain's 451 (correlation 0.996), so they are neither noise nor a placement rule. In the generated world the surface is still water (98 to 99 % source blocks), the top blocks match the bake's in 91 to 98 % of the columns and within one block in 99 to 100 %, and the steps are where the bake puts them or up to half a river width upstream. The depth is not: the generated centre of ranks 1, 2 and 3 is 2, 1 and 1 deep against the bake's 12, 8 and 5.
3. **Mouths and deltas.** Every river reaches the sea in one channel; the routing has no distributaries, and there are 0 to 4 small mouths within 10 km of the 25 (at most one of 50 m3/s or more). The land that mouth fill can add is capped at sea level, so at most it is the beach band, height in (62, 63] joined to the sea. Around the 25 mouths that band covers a median 997 km2 of the 60 km square, against 122 km2 around coasts with no mouth near (p90 398 km2); 17 of the 25 are above that p90. Seven of the 25 (ranks 3, 6, 11, 12, 16, 19, 24) rise 2 m or more per km in their first 50 km and have only 94 to 260 km2 of it. What the river itself built cannot be separated out: the bake does not record it. In the game the mouths (ranks 1 to 3) are a single channel, 1 to 2 deep at the centre.
4. **What else is in the way.** The game-side depth rule (above, another agent's fix); the steps as waterfalls (a boat very probably cannot climb one; not tested, no client); the discharge needed for 8 x 3, 88 m3/s, which only 35 sea-reaching rivers reach; and the lakes, which five of the 25 main stems cross. The width and depth caps (64 and 12) do not bind: they start at 4,096 and 2,822 m3/s and no river reaches either.

### How it was measured

The bake keeps neither the catchment area nor the discharge field nor the receivers; they exist only inside `hydro::network::run`. The hydrology runs on the Lambert equal-area L3 grid (8,192 x 2,624 cells of 600 m, 0.36 km2) whatever the world's projection, so the catchments were taken by running that stage again on the D5 run's own history. A temporary hook (env var `DT_RIVDUMP_DIR`, in `hydro/network.rs`) wrote the arrays, and a temporary pin of `scale.projection` in `epochs::key()` let a Lambert resume accept the Petroff-Guyou run's checkpoints. Both were reverted and never committed.

| Step | Command (on the test VM, through `tools/remote/mac.sh`) |
|---|---|
| Lambert bake of D5 | `deeptime resume --out <lam> --no-epochs --threads 8 --set scale.projection=lambert`, from the run's step-450 checkpoint (t = -100 Myr, the last 50 steps), with the hook on |
| Drainage arrays | `deeptime hydro --world-data <lam>` with `DT_RIVDUMP_DIR` set: area (km2), discharge, receiver, fill, state, 8,192 x 2,624 |
| Rivers, steps, reaches | Python scripts over `rivers.dtrv` of the D5 world (Petroff-Guyou frame, lengths in blocks) and of the Lambert bake (true surface km), in `build/rivers-d5/` (gitignored): `rv.py`, `catch.py`, `steps.py`, `mouths.py`, `navnet.py`, `distsea.py` |
| Generated world | `tools/workbench/preview.py submit --profile standalone --size 13312 --world-data <D5 world data> --keep-world --job rivers-d5-gen2 --box ...` (36 boxes of 128 x 128 columns on the lowest 1,000 blocks of ranks 1 to 3), then `gen_lib.py`, `steps_game.py`, `transect3.py` read the region files |

The Lambert bake's water statistics equal the D5 bake's in all 14 values compared (land cells, river cells, chains, vertices, river length, maximum discharge, widest river, land area, lakes, lake area, largest lake, basins, lake share, playa area). The counts at 50 / 100 / 300 / 1,000 m3/s are the same in both rivers files (57 / 33 / 10 / 2). The sea-mouth chains number 1,048 in the Petroff-Guyou file and 1,062 in the Lambert one (the projection pass's handling of chain ends at the coast; not traced). Kilometres below are the Lambert bake's true surface kilometres; blocks are the Petroff-Guyou file's (what a player walks). Earth's rivers are from Wikipedia, "List of rivers by discharge" (fetched 2026-10-22, rows 1 to 100, discharge and Outflow columns). A river counts as sea-reaching when the page's Outflow is a sea, ocean, gulf or bay (the Brahmaputra and Ganges, the Paraná and Uruguay, and the Tocantins are counted by their own outflow, the Volga's Caspian as a sea). Without the Volga Earth's first two counts are 44 and 24.

### 1. River sizes

Discharge at the sea mouths (1,048 mouths; the median is 4.2 m3/s, the 99th percentile 280 m3/s; the sum 20,958 m3/s, which is 1.39 million m3/s Earth-equivalent, against 1.22 to 1.40 million for the oceans on Earth: the page gives 38,500 to 44,200 km3 per year):

| At least (m3/s) | 2 | 5 | 10 | 20 | 50 | 100 | 200 | 300 | 500 | 1,000 | 2,000 | 3,000 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Sea-reaching rivers | 1,048 | 438 | 217 | 109 | 57 | 33 | 20 | 10 | 6 | 2 | 1 | 0 |

The 25 largest (catchment area from the drainage arrays; runoff is Q over the catchment; width and depth are the bake's at the mouth):

| Rank | Q (m3/s) | Q x 66.3 | Catchment (km2) | Runoff (mm/yr) | Main stem (km) | Width x depth (blocks) |
|---|---|---|---|---|---|---|
| 1 | 2,745 | 182,053 | 256,870 | 337 | 906 | 52.4 x 11.9 |
| 2 | 1,774 | 117,666 | 121,208 | 462 | 616 | 42.1 x 10.0 |
| 3 | 690 | 45,744 | 44,566 | 488 | 569 | 26.3 x 6.8 |
| 4 | 645 | 42,793 | 50,604 | 402 | 256 | 25.4 x 6.7 |
| 5 | 523 | 34,703 | 16,851 | 980 | 245 | 22.9 x 6.1 |
| 6 | 523 | 34,702 | 61,345 | 269 | 340 | 22.9 x 6.1 |
| 7 | 478 | 31,710 | 13,225 | 1,141 | 143 | 21.9 x 5.9 |
| 8 | 475 | 31,521 | 16,112 | 931 | 237 | 21.8 x 5.9 |
| 9 | 345 | 22,893 | 18,733 | 581 | 159 | 18.6 x 5.2 |
| 10 | 304 | 20,169 | 14,714 | 652 | 221 | 17.4 x 4.9 |
| 11 | 299 | 19,805 | 60,992 | 154 | 413 | 17.3 x 4.9 |
| 12 | 258 | 17,118 | 27,216 | 299 | 236 | 16.1 x 4.6 |
| 13 | 257 | 17,054 | 11,857 | 684 | 139 | 16.0 x 4.6 |
| 14 | 237 | 15,718 | 13,855 | 540 | 188 | 15.4 x 4.5 |
| 15 | 233 | 15,436 | 14,729 | 499 | 185 | 15.3 x 4.4 |
| 16 | 230 | 15,266 | 9,441 | 769 | 111 | 15.2 x 4.4 |
| 17 | 223 | 14,778 | 24,564 | 286 | 245 | 14.9 x 4.3 |
| 18 | 222 | 14,717 | 37,393 | 187 | 345 | 14.9 x 4.3 |
| 19 | 214 | 14,165 | 16,617 | 406 | 204 | 14.6 x 4.3 |
| 20 | 205 | 13,611 | 12,197 | 531 | 168 | 14.3 x 4.2 |
| 21 | 194 | 12,871 | 8,082 | 758 | 140 | 13.9 x 4.1 |
| 22 | 181 | 12,032 | 7,451 | 768 | 166 | 13.5 x 4.0 |
| 23 | 173 | 11,440 | 4,821 | 1,129 | 107 | 13.1 x 3.9 |
| 24 | 153 | 10,133 | 10,840 | 445 | 145 | 12.4 x 3.7 |
| 25 | 149 | 9,899 | 17,650 | 267 | 214 | 12.2 x 3.7 |

Against Earth's sea-reaching rivers, rank for rank (Earth: Wikipedia, mean discharge in m3/s):

| Rank | D5 Q x 66.3 | Earth river | Earth Q |
|---|---|---|---|
| 1 | 182,053 | Amazon | 209,000 |
| 2 | 117,666 | Congo | 41,400 |
| 3 | 45,744 | Orinoco | 39,000 |
| 4 | 42,793 | Yangtze | 31,900 |
| 5 | 34,703 | Brahmaputra | 27,000 |
| 6 | 34,702 | Mississippi | 21,300 |
| 7 | 31,710 | Yenisei | 20,200 |
| 8 | 31,521 | Parana | 19,706 |
| 9 | 22,893 | Lena | 18,300 |
| 10 | 20,169 | Mekong | 16,010 |
| 15 | 15,436 | Amur | 12,360 |
| 20 | 13,611 | Volga (Caspian) | 8,380 |
| 25 | 9,899 | Salween | 6,691 |

D5 is above Earth at every rank from 2 to 25 on this scaling, and below it only at the very top. Rank 1 has 2.8 times the Amazon's area when scaled (17.0 against 6.02 million km2) but 337 mm of runoff a year against the Amazon's 1,096 (209,000 m3/s over 6,020,438 km2, both from the page), so it is a big dry basin. The mean runoff of the sea catchments is 408 mm/yr.

**Planet size against drainage.** The counts at area-scaled thresholds:

| D5 threshold (m3/s) | Earth-equivalent (m3/s) | D5 | Ben's single-player world | Earth |
|---|---|---|---|---|
| 50 | 3,315 | 57 | 48 | 45 |
| 100 | 6,630 | 33 | 30 | 25 |
| 300 | 19,890 | 10 | 5 | 7 |
| 1,000 | 66,300 | 2 | 1 | 1 |
| 3,000 | 198,900 | 0 | 0 | 1 |

Ben's single-player world (Lite Prism, `rivers.dtrv`, C = 13,312; largest river 1,727 m3/s, 41.6 blocks wide and 9.9 deep) is the same size as D5, so planet size explains none of the difference between the two; the difference is the layout of the continents. D5 has 9 more rivers at 50 m3/s, 3 more at 100, 5 more at 300 and one more at 1,000, and its largest river is 59 % larger. The single-player catchments were not measured (the save is read only and the counts came from the coordinator). Against Earth, the whole shortfall in absolute discharge is the factor 66: a river of Earth's rank is 66 times smaller here. Per area both worlds carry about Earth's number of big rivers (the single-player world has 5 against Earth's 7 at 19,890 m3/s Earth-equivalent and none at the top); the spread between the two worlds is layout.

What limits the size of a river is the size of a continent. The largest landmass is 730,139 km2 (48 million km2 Earth-equivalent) but it is split: its biggest catchment is 8 % of it, 11 % (82,182 km2) drains inland, and its biggest rivers are ranks 3, 4, 6, 11, 13, 17 and 19. The two largest rivers sit on landmasses where one river takes over half (55 % of 464,332 km2 for rank 1, 52 % of 232,985 km2 for rank 2). Overall 4.06 % of the land (94,188 km2 of 2,318,897) drains away from the sea. The farthest land from the sea is 236 km (about 1,920 km Earth-equivalent; Earth's is about 2,600 km, from memory); 14.5 % of the land is farther than 100 km.

### 2. Navigable reaches and steps

Reach is the length of the main stem from the mouth to the first vertex narrower than 8 or shallower than 3, on bake values. A lake on the way counts as water. Steps are counted in the lowest 1,000 blocks of the main stem, or the whole main stem where it is shorter (20 of the 25 are, 370 to 870 blocks), and in the reach. The fall columns are the water surface's and the terrain's (median of the ground under the line over 50 blocks) over the same lowest reach.

| Rank | Reach (km) | Reach (blocks) | Lake in main stem (km) | First step (blocks) | Steps, lowest 1,000 | Steps in reach | Surface fall (blocks) | Terrain fall (blocks) |
|---|---|---|---|---|---|---|---|---|
| 1 | 781 | 2,641 | | 91 | 3 | 3 | 2.1 | 3.6 |
| 2 | 529 | 1,637 | | 545 | 4 | 14 | 3.5 | 5.5 |
| 3 | 231 | 800 | 182 (in reach) | 48 | 4 | 4 | 4.0 | 4.5 |
| 4 | 190 | 605 | | 55 | 10 | 2 | 9.3 | 9.3 |
| 5 | 182 | 567 | | 154 | 17 | 8 | 16.8 | 17.4 |
| 6 | 215 | 730 | 25 (in reach) | 34 | 15 | 8 | 14.8 | 16.6 |
| 7 | 115 | 509 | | 129 | 3 | 2 | 2.3 | 4.1 |
| 8 | 125 | 429 | 6 | 244 | 25 | 8 | 24.4 | 25.8 |
| 9 | 61 | 236 | 8 | 332 | 17 | 0 | 16.9 | 18.5 |
| 10 | 135 | 378 | | 418 | 17 | 0 | 16.9 | 16.6 |
| 11 | 320 | 1,092 | 207 (in reach) | 19 | 7 | 7 | 7.6 | 7.4 |
| 12 | 112 | 415 | | 33 | 38 | 5 | 37.3 | 37.5 |
| 13 | 59 | 248 | | 118 | 56 | 5 | 55.8 | 52.6 |
| 14 | 103 | 342 | | 215 | 26 | 3 | 25.8 | 22.8 |
| 15 | 96 | 327 | | 70 | 41 | 4 | 40.6 | 36.9 |
| 16 | 47 | 156 | | 20 | 10 | 3 | 9.5 | 8.9 |
| 17 | 159 | 541 | | 321 | 22 | 5 | 21.2 | 20.5 |
| 18 | 240 | 874 | | 168 | 15 | 11 | 14.5 | 16.3 |
| 19 | 80 | 265 | | 14 | 27 | 12 | 26.4 | 23.9 |
| 20 | 90 | 308 | | 336 | 3 | 0 | 2.9 | 4.3 |
| 21 | 46 | 161 | | 368 | 4 | 0 | 3.5 | 4.8 |
| 22 | 54 | 248 | | 388 | 10 | 0 | 9.4 | 10.8 |
| 23 | 33 | 132 | | 142 | 12 | 0 | 12.0 | 12.5 |
| 24 | 72 | 222 | | 7 | 52 | 16 | 51.9 | 47.9 |
| 25 | 136 | 407 | | 72 | 21 | 9 | 20.9 | 22.1 |

- **Reach.** Ten rivers have 500 blocks or more of navigable channel (ranks 1 to 7, 11, 17, 18). The biggest river is the only one with a long clean reach: 3 steps in 2,641 blocks, all in the first 663 blocks, then about 1,980 blocks (580 km) without one. Six rivers (9, 10, 20, 21, 22, 23) have no step inside their reach: 33 to 135 km of clean 8 x 3 water from the sea.
- **Network.** Across the whole bake there are 52 river systems with any 8 x 3 channel, 5,586 km in all (14 systems of 100 km or more, 2 of 500 km or more, 1 of 1,000 km: 1,301 km; the next is 732 km). At 4 x 2 there are 121 systems and 12,770 km. For a boat alone (2 wide, 1 deep) there are 813 systems and 48,109 km, 15 systems of 500 km or more and 5 of 1,000 km or more (the largest 8,332 km). All channel is 76,948 km in 4,003 chains.
- **Do the steps follow the real fall?** Yes. Over the lowest reaches of the 25 the water surface falls 450 blocks and the terrain under the line 451 (mean absolute difference 1.5 blocks per river, at most 4.1; correlation 0.996; surface fall = 1.03 x terrain fall). A step is one crossing of a block level, and the 459 steps are 1.02 per block of fall. The surface is held at the sea's base level (63) over a median 21 % of the lowest 1,000 blocks (more than half in ranks 2, 9, 10, 20, 21 and 22), so those rivers run level for a long way before their first step. So the steps are not noise and not the placement rule: they are the terrain falling 35 m a block. The fall is steep for rivers this big. It averages 3.0 m per km over the 5,197 km of lowest reach examined (450 blocks of 35 m over 17,640 blocks of 294.6 m); Earth's great rivers fall a few cm to about 10 cm per km in their lower thousand kilometres (from memory). The relief is Earth's but the horizontal scale is 1:8 (reasoned, not measured).
- **Generated world, ranks 1 to 3** (36 boxes, covering 295, 390 and 297 of the 407, 513 and 395 vertices of the lowest 1,000 blocks):

| Rank | Bake steps (blocks from the mouth) | Generated steps | Tops equal | Within 1 block | Source blocks |
|---|---|---|---|---|---|
| 1 | 3 at 93, 206, 665 | 2 at 116, 230 | 91 % | 100 % | 293 of 295 |
| 2 | 1 at 546 | 1 at 559 | 98 % | 100 % | 389 of 390 |
| 3 | 4 at 50, 56, 59, 191 | 4 at 62, 64, 67, 191 | 98 % | 99 % | 293 of 297 |

Both are counted on the covered vertices only, so the bake's step counts here (3, 1, 4) differ from the table above (3, 4, 4) where the boxes do not reach.

The generated top is the highest water block in the 3 x 3 columns round each vertex. No surface falls going upstream, the surface is still water, and the steps come out 0 to 24 blocks upstream of the bake's (rank 1's half width is 26). The game's top is on average lower than the bake's by 0.09, 0.02 and 0.03 blocks (probably the lowest-w rule below picking a downstream segment's lower surface; not traced), and rank 1's third bake step (665) has no match in the generated tops.

### Depth in the game (the measure the coordinator asked for)

The game-side depth bug has its own fix agent; nothing was fixed here. On the 25 largest rivers the bake asks for 3.7 to 11.9 blocks of water at the mouth. In the generated lower reaches of ranks 1 to 3:

Cross-sections cut every few vertices across the channel, columns every 2 blocks from one bank to the other (rule and nearest are the bake's rule evaluated on the same columns, see below):

| Rank | Cross-sections | Generated, centre third (median) | Bake intends (median) | The rule gives (median) | Smallest d / hw gives (median) | Sections with a 3-deep column | Same, in the centre third |
|---|---|---|---|---|---|---|---|
| 1 | 50 | 2 | 11 | 2 | 11 | 74 % | 62 % |
| 2 | 63 | 1 | 8 | 1 | 8 | 19 % | 13 % |
| 3 | 39 | 1 | 4.5 | 1 | 5 | 13 % | 8 % |

The centre line of the path, one column per vertex (295, 375 and 239 vertices of the lowest 1,000 blocks):

| Rank | Generated depth (median) | Bake depth (median) | Vertices 3 or more deep | Vertices 1 or less deep | First 3-deep vertex (blocks from the mouth) |
|---|---|---|---|---|---|
| 1 | 2 | 12 | 41 % | 39 % | 68 |
| 2 | 1 | 8 | 12 % | 83 % | 534 |
| 3 | 1 | 5 | 8 % | 84 % | 39 |

The width is right (the generated water columns x 2 against 2 hw: 50 against 50.8, 36 against 35.2, 20 against 19.7). The depth is not, along the whole lowest 1,000 blocks of all three rivers checked. The cause is the covering-segment rule (`Rivers.sampleRange`: of the segments that cover a column, the one with the lowest w, ties to the lowest id). Segments are capsules, so their round caps overlap at every vertex, and where the surface falls the downstream segment (lower w) wins the column, where it is level the lower id does, over the deep centre of the segment that should own it. Evaluating the documented rule over the same cross-sections gives 2, 1 and 1, the game's depths; choosing the covering segment with the smallest d / hw instead gives 11, 8 and 5, the bake's. The sea test is not the cause: the centre column is under the sea rule (ground at or below y 62, answered before the river rule) in 0, 9 and 0 % of ranks 1 to 3's lowest 1,000 vertices, and in 0 to 9 % (median 0.4 %) over the 25. Rank 1's first 68 blocks are under 3 deep at the centre without it.

### 3. Mouths and deltas

How a river meets the sea, for the 25 largest. Beach band is the land with height in (62, 63] joined to the sea (the 35 m flat of "The sea's one rule"); flat to +3 is the same up to height 65. Both are counted in the 60 km square round the mouth (3,600 km2). Rise is the water surface's rise over the first 50 km, in m per km. To +3 is the length of channel up to which the water surface is within 3 blocks of the sea's. Others is the number of other sea mouths within 10 km.

| Rank | Beach band (km2) | Flat to +3 (km2) | Rise (m/km) | To +3 (km) | Others |
|---|---|---|---|---|---|
| 1 | 322 | 943 | 0.7 | 803 | 0 |
| 2 | 2,383 | 2,390 | 0.0 | 329 | 2 |
| 3 | 260 | 472 | 2.9 | 28 | 0 |
| 4 | 754 | 1,226 | 0.5 | 233 | 1 |
| 5 | 1,578 | 2,127 | 0.0 | 121 | 3 |
| 6 | 258 | 509 | 4.6 | 18 | 0 |
| 7 | 1,411 | 2,043 | 0.9 | 143 | 4 |
| 8 | 1,828 | 2,003 | 0.0 | 103 | 0 |
| 9 | 1,939 | 2,201 | 0.0 | 110 | 0 |
| 10 | 1,930 | 2,253 | 0.0 | 176 | 1 |
| 11 | 153 | 269 | 3.7 | 6 | 0 |
| 12 | 148 | 444 | 3.2 | 24 | 0 |
| 13 | 923 | 1,615 | 1.8 | 53 | 4 |
| 14 | 1,251 | 1,696 | 0.0 | 104 | 1 |
| 15 | 997 | 1,514 | 1.3 | 68 | 0 |
| 16 | 175 | 432 | 2.1 | 62 | 0 |
| 17 | 1,066 | 1,767 | 0.0 | 142 | 0 |
| 18 | 784 | 1,364 | 1.2 | 205 | 0 |
| 19 | 94 | 189 | 6.6 | 13 | 0 |
| 20 | 1,548 | 2,106 | 0.0 | 168 | 1 |
| 21 | 1,084 | 1,576 | 0.0 | 137 | 0 |
| 22 | 1,893 | 1,905 | 0.0 | 110 | 3 |
| 23 | 1,396 | 2,176 | 1.0 | 63 | 2 |
| 24 | 108 | 171 | 9.6 | 20 | 0 |
| 25 | 712 | 1,026 | 1.7 | 56 | 1 |

- **Channels.** The routing gives each cell one receiver, so a river reaches the sea in one channel and does not split. There is no distributary in the bake. Of the 25, 14 have no other mouth within 10 km, the rest 1 to 4, and only ranks 7, 22 and 25 have one of 50 m3/s or more.
- **Land built by the river.** The surface stage's mouth fill (`surface/marine.rs`) raises the sea cells next to a mouth to a level `marine_fill_depth_m` below sea level (the preset's value is 0.0), falling with distance, so it cannot make land above sea level. The most it makes is the beach band. That band joined to the sea covers a median 997 km2 round the 25 mouths, 28 % of the square. Round 3,000 coast cells drawn from the 59,806 with no sea mouth of 5 m3/s or more within 36 km it covers a median 122 km2 (mean 185, p90 398), and round 3,000 drawn from all 99,321 coast cells a median 154 km2 (p90 528). 17 of the 25 are above the no-mouth p90, and the flat to +3 shows the same (median 1,576 km2 round the 25 against 207 km2). The bake does not record which cells a river built, and the mouth fill's land is the same height as flat coast the plates left, so this is an upper bound and a sign, not an attribution. Seven rivers sit on a rising coast (ranks 3, 6, 11, 12, 16, 19, 24: 2.1 to 9.6 m per km in the first 50 km) with 94 to 260 km2 of beach band. The land of two checkpoints (steps 439 and 450) could not be differenced: they are 22 Myr apart, the sea stood 723.9 m at one and 851.2 m at the other, and the plates moved.
- **Plain behind the mouth.** Most big rivers run on a very flat plain: the water surface stays within 3 blocks (105 m) of the sea's for a median 104 km of channel, 803 km on rank 1, 329 km on rank 2. The surface is held at the base level (63) over a median 21 % of the lowest 1,000 blocks.
- **What the game shows** (ranks 1 to 3, generated). One wide channel through low flat ground: rank 1's top blocks run from y 62 to 64 over its lowest 1,000 blocks, its first 68 blocks are under 3 deep at the centre, and nothing fans out. There is no delta in the sense of a river that splits and builds land out into the sea. The flat coastal plain is there, but it is not marked as anything, and the water in it is 1 to 2 deep at the centre (above).

### 4. Other things in the way

- **The depth rule in the game** (above): the lower reaches of the three rivers checked are 1 to 2 deep where the bake asks for 5 to 12.
- **Steps are waterfalls.** 459 of them in 17,640 blocks of main stem. A boat very probably cannot climb a one-block drop of flowing water; this was not tested (no client). The biggest river has only 3.
- **The entry ticket.** Width is Q^0.5 and depth 0.5 Q^0.4 (the bake's laws, HYD-001 to HYD-004; the D5 mouths fit them: 52.4 = 2,745^0.5); 8 x 3 needs 88 m3/s (depth is the binding one: 8 wide needs only 64). 35 sea-reaching rivers reach it (58 reach 50). Only 52 river systems carry any 8 x 3 channel, 5,586 km in all, because the planet is small.
- **Lakes.** Five of the 25 main stems pass through a lake (ranks 3, 6, 8, 9, 11: 182, 25, 6, 8 and 207 km). Lakes are open water and sailable; a lake that fills a basin on a main stem also keeps the river's sediment (HYD-110).
- **The caps** (HYD-006, HYD-007). Width is capped at 64 (Q = 4,096) and depth at 12 (Q = 2,822); rank 1 (2,745 m3/s, 11.9 deep) is just under the depth cap and no river passes either.

### What could not be measured

- Ben's single-player catchments, steps and mouths: the save is read only and holds the rivers file, not the drainage arrays; only the counts the coordinator gave are used.
- The generated world beyond ranks 1 to 3 and beyond the lowest 1,000 blocks: only 36 boxes were generated. The 22 other rivers' game depth is not measured; the rule is the same for all.
- Boats at a step: no client was run.
- Land built by the river: not recorded in the bake (above).
- Earth's navigable lengths, mouth widths and depths: not fetched, and not used.

### World generation runs (Ben's pool rule of 2026-10-22)

Measured on existing data first: the D5 `rivers.dtrv` (Petroff-Guyou) for every count, path and step; Ben's single-player numbers as given; the bakes in `/data/ban/deeptime-runs/pool/BASES.md` (the cube1 and high-ground lanes, none of them D5). Three kinds of run were needed:

| Run | What | Why an existing one would not do |
|---|---|---|
| `deeptime resume` of run 1a16576c9de from step 450, Lambert, `--no-epochs`, 8 threads (twice: the first run's dump came from a coarser call of the stage and was discarded; the second kept epochs for a look at the surface fields of steps 439 and 450, which was dropped) | the Lambert bake with the hydrology's drainage dumped | The bake keeps no catchment area, discharge field or receivers, and Ben's own save and the pool's bakes are other worlds. |
| `deeptime hydro --world-data <lam>` with the dump on | the full-resolution drainage arrays (8,192 x 2,624) | The first resume dumped a coarser call of the same stage. |
| Two `tools/workbench/preview.py submit` jobs, `rivers-d5-gen` (failed: `--size` was not 13312) and `rivers-d5-gen2` (36 boxes of 128 x 128 columns; it failed only in its export step, after the chunks were saved) | generated chunks on the lower reaches of ranks 1 to 3 | The live play world holds 4 region files, round the spawn (r.-1.-1 to r.0.0), none on these rivers, and the server is read only. |

Kept on the VM and listed in BASES.md: `/data/ban/deeptime-runs/rivers-d5/` `lam` (the Lambert bake; its epochs are hard links of the D5 run's), `dump2` (the drainage arrays), `world-d5`, `region-gen`, and the result files (`pg.json`, `lam.json`, `catch.json`, `steps.json`, `mouths.json`, `transect.json`, `transect3.json`, the paths in `pg_paths.npz` and `lam_paths.npz`).
