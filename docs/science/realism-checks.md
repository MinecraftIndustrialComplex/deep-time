# Realism checks: V1 item 1's seven measures against Earth's history

Status of this page: `stage 2 (Earth's ranges read in fetched sources and computed on fetched data on 2026-10-23; what was not read is marked from-memory, abstract-only or cited-unverified in the ledger)`.

Ledger ids on this page: `RLC-001` to `RLC-075` (see [claims.md](claims.md) and [ledger/RLC.md](ledger/RLC.md)); it leans on HYP-001, HYP-004, HYP-035, HYP-040 to HYP-046, PLT-157, PLT-360 to PLT-363 and CHK-015.

## What you see in the world

The seven measures are the planet's look from orbit and from a boat: how much of the globe is land, whether one continent is big enough to walk across for weeks, whether shallow seas lie over the continents, whether a quarter of the land is highland, whether the coasts are mountain walls, how much of the land is under ice, and how often continents fuse. A player sees the first, second, fourth, fifth and sixth at once on the globe and from a ship. The third shows as the shapes of the seas, and the last only in the timelapse. Each measure's section below ends with what a player sees when a planet is far outside Earth's range.

Scale, for every section: a block is about 300 m across and 35 m high, the sea stands at y 63, so 2 km above the sea is 57 blocks, y 120, and 200 km is about 670 blocks.

## The model in plain words

These are not model features. They are the seven numbers of V1 item 1 (decisions log 2026-10-21), read off a finished run by `tools/review/continents_census.py` through the pool's `v1census` measure. The log of 2026-10-23 made them realism checks: a check fires only when a planet lies outside the range Earth itself has had, and no knob, cap or forcing term is added to move one (Ben: "Results that don't align with earth but are still perfectly playable and physics backed > Results that are forced to align with earth by introducing extremely weird forcing functions"; RLC-001). This page re-sources each band against Earth's history, so that a check fires on a planet that Earth itself never was and on nothing else.

How the census counts (RLC-004; `tools/review/pool/measure_v1census.py`):

| Measure | What it counts | By rock or by surface |
|---|---|---|
| M1 land | cells above the sea datum (block 63), over all cells | the ice-free surface |
| M2 largest landmass | the largest 8-connected landmass, joined over the x seam, a share of the planet | the ice-free surface |
| M3 continental crust under the sea | cells the plate model flags continental (target thickness at the rifted edge 20 km, PLT-261) and not land, over the continental cells | the ice-free surface |
| M4 land above 2 km | land with the surface above 2,000 m, a share of the land | **by rock** |
| M5 coast with high ground | coast cells (land within 1.6 cell spacings of the sea) with land above 2 km on the same landmass within 200 km inland, a share of the coast | **by rock** |
| M6 land under ice | land with 1 m of glacier ice or more (`climate/ice_thickness_m`), a share of the land | ice, apart from the rock |
| M7 collisions | the plate check's `collisions_per_100myr`: a terrane of at least 0.1 of the average starting plate (510.07 / 14 = 36.4 million km2, so 3.64 million km2) that ends in a weld or a transfer, dated at its end | not a map measure |

The census's `elevation` field is the rock surface, with the ice in a separate field. So M4 and M5 are by rock, and Earth's like-for-like figures are by rock: 0.047 to 0.055 of the land above 2 km (HYP-041, PLT-360), not the 0.102 to 0.104 that counts the Antarctic and Greenland ice sheets' tops (RLC-004). The registry's 0.12 and 0.15 were set from the by-surface figures.

How the bands are set here (RLC-002, the writer's proposal): the check fires outside Earth's Phanerozoic range (about 540 Myr), rounded outward to 0.01. A "watch" line sits inside it where the nearest Earth state of interest lies: the range of the last 250 Myr where the source gives one (M1, M3), today's like-for-like value for the high-ground and ice measures (M4, M5, M6), the writer's own Phanerozoic maximum for M7. M7 is the one exception to the firing edge: its Phanerozoic count is the writer's and unverified, so it fires at the looser published average (6), not at the count's own edge (4). A registry band that Earth itself fails is wrong; one that Earth passes with room to spare is loose.

## Summary

Earth's range, by measure. "Last 250 Myr" and "Phanerozoic" are ranges over the maps or steps of the source named; they are not distributions (Earth is one planet, and neighbouring maps are not independent).

| Measure | Registry band | Earth today, like-for-like | Last 250 Myr | Phanerozoic | Source and status |
|---|---|---|---|---|---|
| M1 land, share of the planet | 0.26 to 0.32 | 0.292 | 0.185 to 0.292 (Marcilly); 0.22 to 0.365 (Scotese) | 0.131 to 0.292 (Marcilly); 0.143 to 0.365 (Scotese) | Marcilly et al. 2022 Table 1 read (`verified`); PaleoDEMs run by the project (`cited-unverified`) |
| M2 largest landmass, share of the planet | at least 0.12 | 0.156 | 0.070 to 0.363 | 0.070 to 0.363 | PaleoDEMs run by the project (`cited-unverified`) |
| M3 continental crust under the sea | at most 0.25 | 0.248 by the census's 20 km edge, 0.292 by the continental polygons | 0.258 to 0.528 by polygons, 0.21 to 0.48 after the 0.044 offset | up to 0.600 by polygons, 0.56 after the offset | Marcilly et al. 2022 Table 1 read (`verified`); the offset is the project's subtraction |
| M4 land above 2 km, share of the land | at most 0.12 | 0.047 to 0.055 by rock (0.104 by surface) | up to 0.062 | up to 0.062 | PaleoDEMs run by the project (`cited-unverified`); today's by HYP-041, PLT-360 |
| M5 coast with high ground | at most 0.15 | 0.076 by rock (0.104 by surface) | up to 0.092 on the 40,000-point grid, about 0.10 at the census's resolution | the same | PaleoDEMs run by the project (`cited-unverified`) |
| M6 land under ice, share of the land | at most 0.15 outside a glaciation | 0.10 to 0.107 | 0 (greenhouse) to 0.25 or 0.32 (Last Glacial Maximum) | the same; the Late Paleozoic ice age has no area figure | NSIDC and USGS (via Wikipedia) read (`verified`); greenhouse list tertiary (`cited-unverified`) |
| M7 collisions per 100 Myr | 2 to 6 | not defined for today; the three large ones are still going | about 0 to 2 by the writer's count | about 0 to 4 by the writer's count, one to 20 for a looser definition | the writer's count `from-memory`; Condie et al. 2015 abstract only; Macdonald et al. 2019 read (a different count) |

Proposed bands (RLC-014, 021, 032, 042, 052, 063, 075), against the registry's, with what the master default's median (the D6 arm of [physics-gap-audit.md](physics-gap-audit.md), not re-run here) does under each:

| Measure | Registry band | Does Earth itself pass it? | Proposed: fires | Proposed: watch | D6 median | D6 under the registry | D6 under the proposal |
|---|---|---|---|---|---|---|---|
| M1 | 0.26 to 0.32 | no: it fails at 15 of 26 Marcilly steps and 16 of 51 Scotese maps from 0 to 250 Ma | outside 0.13 to 0.36 | outside 0.18 to 0.30 | 0.300 | in | in |
| M2 | at least 0.12 | no: 13 of 51 maps from 0 to 250 Ma, 10 of 21 from 0 to 100 Ma | below 0.06 | below 0.07 | 0.124 | in | in |
| M3 | at most 0.25 | no: all 26 Marcilly steps from 0 to 250 Ma exceed it by polygons, 22 of 26 after the offset | above 0.56 | above 0.48 | 0.278 | out | in |
| M4 | at most 0.12 | yes, with room: none of 109 maps above it | above 0.07 | above 0.05 | 0.271 | out | out, at about 4 times Earth's maximum |
| M5 | at most 0.15 | yes, with room: none of 109 maps above it | above 0.11 | above 0.08 | 0.411 | out | out, at about 4 times Earth's maximum |
| M6 | at most 0.15 outside a glaciation | only in an interglacial: the Last Glacial Maximum was 0.25 to 0.32 | above 0.32 | above 0.11 if the planet is not in an ice age | 0.170 | out | in |
| M7 | 2 to 6 | the floor, no: Earth had very few from 0 to 150 Ma; the ceiling, yes | above 6 | above 4 | 1.80 | out (below the floor) | in |

Under the proposed bands only M4 and M5 fire on the master default, and they fire hard. The other five are inside what Earth has been. M1 to M3 and M7's floor were wrong, in the sense that Earth would fail them for much of its history; M4 and M5 were loose.

## Earth's range, measure by measure

The full history table behind M1 to M5 (RLC-011, 012, 020, 031, 041, 051), every 20 Myr. Marcilly: Marcilly, Torsvik and Conrad 2022, Table 1 (read; their exposed land and continental crust at 10 Myr steps, 0 to 520 Ma; drowned = (crust less exposed) / crust). Scotese: the PaleoDEMs of Scotese and Wright 2018 run through the census's own helpers at 40,000 points (spacing 113 km) by `tools/review/earth-paleodem.py`, every 5 Myr, 0 to 540 Ma. The command is `python3 -I tools/review/earth-paleodem.py paleodem <folder of Map*.nc> 40000`, one core, niced; the control, `... relief earth_relief_15m_p.grd 40000`, gives land 0.2913, largest 0.1561, above 2 km 0.1064 (the ice sheets' tops included) and coast 0.0949 on present-day SRTM15+ (RLC-003).

| Age (Ma) | Land, Marcilly | Drowned crust, Marcilly | Land, Scotese | Largest landmass | Above 2 km | Coast with high ground |
|---|---|---|---|---|---|---|
| 0 | 0.292 | 0.292 | 0.276 | 0.156 | 0.062 | 0.092 |
| 20 | 0.288 | 0.306 | 0.301 | 0.173 | 0.035 | 0.044 |
| 40 | 0.278 | 0.337 | 0.321 | 0.242 | 0.021 | 0.044 |
| 60 | 0.255 | 0.363 | 0.279 | 0.085 | 0.008 | 0.020 |
| 80 | 0.185 | 0.528 | 0.220 | 0.070 | 0.015 | 0.040 |
| 100 | 0.208 | 0.472 | 0.269 | 0.119 | 0.036 | 0.041 |
| 120 | 0.208 | 0.460 | 0.259 | 0.087 | 0.009 | 0.019 |
| 140 | 0.235 | 0.367 | 0.307 | 0.297 | 0.008 | 0.036 |
| 160 | 0.211 | 0.419 | 0.258 | 0.126 | 0.010 | 0.037 |
| 180 | 0.269 | 0.288 | 0.344 | 0.343 | 0.008 | 0.027 |
| 200 | 0.259 | 0.306 | 0.354 | 0.349 | 0.009 | 0.046 |
| 220 | 0.257 | 0.308 | 0.280 | 0.201 | 0.009 | 0.036 |
| 240 | 0.263 | 0.295 | 0.365 | 0.362 | 0.005 | 0.012 |
| 260 | 0.249 | 0.333 | 0.250 | 0.189 | 0.000 | 0.000 |
| 280 | 0.224 | 0.392 | 0.239 | 0.194 | 0.048 | 0.068 |
| 300 | 0.217 | 0.390 | 0.216 | 0.170 | 0.023 | 0.014 |
| 320 | 0.230 | 0.358 | 0.243 | 0.202 | 0.038 | 0.030 |
| 340 | 0.209 | 0.393 | 0.212 | 0.179 | 0.037 | 0.042 |
| 360 | 0.211 | 0.400 | 0.181 | 0.141 | 0.018 | 0.010 |
| 380 | 0.161 | 0.540 | 0.150 | 0.114 | 0.051 | 0.087 |
| 400 | 0.187 | 0.469 | 0.159 | 0.091 | 0.053 | 0.056 |
| 420 | 0.152 | 0.562 | 0.152 | 0.100 | 0.015 | 0.041 |
| 440 | 0.178 | 0.483 | 0.183 | 0.143 | 0.030 | 0.022 |
| 460 | 0.152 | 0.555 | 0.167 | 0.127 | 0.007 | 0.017 |
| 480 | 0.155 | 0.544 | 0.162 | 0.122 | 0.005 | 0.021 |
| 500 | 0.153 | 0.552 | 0.170 | 0.131 | 0.034 | 0.026 |
| 510 | 0.131 | 0.600 | 0.165 | 0.128 | 0.007 | 0.000 |
| 520 | 0.148 | 0.554 | 0.155 | 0.115 | 0.000 | 0.000 |
| 540 | not tabulated | not tabulated | 0.183 | 0.133 | 0.017 | 0.017 |

Two cautions on the Scotese columns apply to everything below. The PaleoDEMs are, in their authors' own words, a "first draft": the highest elevations are "likely overestimated" and the maps built in two phases are not homogeneous (the 275 to 280 Ma step, from 0.009 to 0.048 above 2 km, may be the join of the two phases or a real signal of the Variscan and Alleghanian belts; it is not settled here). And at 1 degree a land bridge one cell wide joins two landmasses, so the largest-landmass column jumps (140 Ma reads 0.297 between 0.148 and 0.150). The two sources disagree on the Pangaean land share by up to 0.09 (200 Ma: Scotese 0.354, Marcilly 0.259; RLC-012).

### M1 land, share of the planet

Definition: cells above the sea datum over all cells (RLC-010). A band of 0.26 to 0.32 was set around today's 0.289 to 0.292.

Earth today: exposed land 149.00 million km2 of 510.07, which is 0.292 (Marcilly et al. 2022, Table 1, read; `verified`); 0.289 (HYP-001), 0.2915 (PLT-360) and 0.2913 (the pipeline's control).

Earth's history: from Marcilly's Table 1, the exposed land was 0.185 at 80 Ma (the Late Cretaceous high stand: 94.19 million km2) and 0.292 today over the last 250 Myr, and 0.131 at 510 Ma (66.95 million km2) over the Phanerozoic. Today is the Phanerozoic maximum by this source. From the PaleoDEMs the range is 0.22 (80 Ma) to 0.365 (240 Ma, a map 0.07 above both neighbours) over 250 Myr, 0.143 (430 Ma) to 0.365 over the Phanerozoic. The Scotese 0 Ma map reads 0.276 where Earth is 0.291. At a glacial sea level, 125 m lower (Wikipedia, quoting the USGS), the hypsometric slope of 0.0057 km per million km2 (Marcilly et al.) puts about 21.9 million km2 more above the sea: 0.335 of the planet (RLC-013; the project's arithmetic, ice loading ignored). The registry's band fails at 15 of 26 Marcilly steps and at 16 of 51 Scotese maps from 0 to 250 Ma: Earth itself would fail it.

Proposed band: fires outside 0.13 to 0.36; watch outside 0.18 to 0.30. The upper edge is Scotese's Early Jurassic maps (0.344 to 0.354) rounded outward with the 240 Ma jump set aside; Marcilly's data alone would put it at 0.30.

What a player sees. Below 0.13 (for example 0.10) the globe is nine parts water: the Create World timelapse is blue with scattered islands and a player may sail for weeks between landmasses. Above 0.36 (0.45, say) the oceans are narrow inland seas, and every continent is a day's sail from the next. Both show at once on the globe. Between 0.18 and 0.30 a player on the ground cannot tell the planets apart: the difference between 0.26 and 0.32 is 31 million km2, a map-scale thing.

### M2 largest landmass, share of the planet

Definition: the largest 8-connected landmass (joined over the x seam), a share of the planet (RLC-020). At least 0.12 means at least 61 million km2: twice Africa's area and three quarters of Afro-Eurasia's (79.6 million km2 below).

Earth today: 0.156 (79.6 million km2: Afro-Eurasia on a 1 degree map and on the 15 arc-minute SRTM15+ control alike; the two agree).

Earth's history (PaleoDEMs; no second source): 0.070 (35.5 million km2) at 80 Ma and 0.074 and 0.075 at 85 and 90 Ma to 0.363 (184.9 million km2) at 240 Ma, median 0.1735, over the last 250 Myr; the Phanerozoic has the same extremes (0.070 to 0.363, median 0.141); the Paleozoic minimum is 0.091 at 400 Ma. The registry's 0.12 fails at 13 of 51 maps from 0 to 250 Ma, 25 of 109 from 0 to 540 Ma and 10 of 21 from 0 to 100 Ma: in this model Earth had no landmass of 61 million km2 at about a quarter of its maps. The column is noisy (land bridges at 1 degree), so single maps are not read, only the extremes and counts.

Proposed band: fires below 0.06 (0.0696 rounded outward); watch below 0.07; no ceiling (Earth reached 0.36 in the Triassic and Jurassic).

What a player sees. A planet whose largest landmass is under 0.06 (30 million km2, Africa's size) has no continent bigger than Africa and no deep interior: the dry continental interiors and the long overland journeys of a large continent are missing. On the ground this is nearly invisible: one lands on an island chain or a mid-size continent and walks for 5,000 km before the difference shows. On the globe it reads as an archipelago planet.

### M3 continental crust under the sea, share of the continental crust

Definition: cells the plate model flags continental and not land, over the continental cells (RLC-030). The flag's edge sits where a rifted margin's target thickness is 20 km, so Earth's like-for-like figure is the crust at least 20 km thick, not the continental polygons' outline.

Earth today: 0.248 for crust at least 20 km thick (CRUST1.0, HYP-042, PLT-361); 0.292 by Marcilly et al.'s continental polygons (210.49 million km2 of crust, 149.00 exposed; read); 0.306 by Cogley 1984 (HYP-042). The definitions differ by 0.044.

Earth's history (Marcilly et al., Table 1, read; `verified`): the drowned share is 0.258 at 210 Ma, 0.292 today and 0.528 at 80 Ma over the last 250 Myr (median 0.335), and 0.600 at 510 Ma over the Phanerozoic. Every one of the 26 steps from 0 to 250 Ma is above 0.25 by this definition; 22 of 26 stay above it after the 0.044 offset is taken off, which is measured today only (its constancy in the past is not checked).

Proposed band: fires above 0.56 (0.600 less the offset, rounded outward); watch above 0.48 (0.528 less 0.044). No floor is proposed: a share under Earth's lowest (0.21 after the offset) is outside Earth's range on the dry side, but nothing in the sources read says it looks wrong; if one is wanted, 0.20.

What a player sees. A drowned share above 0.56 puts more than half the continental crust under the sea: vast shallow seas over the continents, and with the continental crust at 0.41 of the planet the land falls to about 0.18 of it. Earth did this at 510 Ma (0.600) and, at 0.528, in the Late Cretaceous. A share much under 0.2 would show as cliffs at every coast with no shelf and no shallow sea; it is below Earth's range, but this page does not propose a floor for it.

### M4 land above 2 km, share of the land

Definition: land with the rock surface above 2,000 m (57 blocks above the sea, y 120), a share of the land (RLC-040).

Earth today: 0.047 (CRUST1.0), 0.055 (SRTM15+ without the two ice sheets) and 0.0617 (the PaleoDEM 0 Ma map) by rock; 0.104 by surface, with the ice sheets' tops (HYP-041, PLT-360). The census is by rock, so 0.05 is today's like-for-like value.

Earth's history (PaleoDEMs; RLC-041): the maximum over all 109 maps is the 0 Ma map, 0.0617. Then 0.0535 (5 Ma) and 0.0527 (10 Ma); before 15 Ma the Cenozoic is at most 0.039; the Mesozoic at most 0.036 (100 Ma); 250 to 270 Ma is under 0.001; the Paleozoic reaches 0.053 (400 Ma), 0.051 (380 Ma) and 0.048 (280 Ma). The median of all maps is 0.0097. Not one of 109 maps is above the registry's 0.12, and the high-ground share was a small part of Earth's land through almost all of the Mesozoic. The caps of about 3 km on many older maps do not change the area above 2 km, but the maps' smoothing may lower it.

Proposed band: fires above 0.07 (0.0617 rounded outward); watch above 0.05. The registry's 0.12 is not wrong: it is twice Earth's maximum. It is loose, and its derivation (the by-surface 0.104) counted ice.

What a player sees. At the master default's 0.27, more than a quarter of all land stands above y 120: plateaus and ranges across the continents, and the lowland plains correspondingly scarce. A player meets this at once, and it is the most visible unrealistic feature of the 0.27 planet: Earth's highest like-for-like share is about 0.06, one block of land in sixteen.

### M5 coast with high ground

Definition: coast cells (land within 1.6 cell spacings of the sea) with land above 2 km on the same landmass within 200 km inland, a share of the coast (RLC-050). 200 km is about 670 blocks.

Earth today: 0.104 by surface at the census's 332,800 points (PLT-360); 0.076 without the two ice sheets, the by-rock figure; the pipeline's 40,000-point control reads 0.0949, 0.91 of the census resolution's figure (RLC-003).

Earth's history (PaleoDEMs at 40,000 points; RLC-051): the maximum of 109 maps is the 0 Ma map, 0.0917, then 0.0867 (380 Ma), 0.0721 (10 Ma), 0.0700 (5 Ma) and 0.0683 (280 Ma); the median over 0 to 250 Ma is 0.0223 and the minimum 0.0027 (115 Ma). At the census's resolution the figures read 1.10 times higher (0.104 / 0.0949), so Earth's maximum is about 0.10. None of 109 maps is above the registry's 0.15. The value depends on resolution (the coast band is 181 km wide at 40,000 points where the census's is about 25 km), so it is compared with the control, not with the census's own numbers.

Proposed band: fires above 0.11 (0.10 rounded outward); watch above 0.08 (today's by-rock 0.076). The registry's 0.15 is loose, not wrong, as M4's.

What a player sees. At the master default's 0.41, four coastlines in ten have a 57-block rise within 670 blocks of the shore. From a boat a large part of the shore is a mountain wall, and the coastal plains that harbours and deltas need are scarce. Earth's value is 0.08 to 0.10, one coast in ten or twelve (the Andean and Alaskan coasts are the kind of shore it counts; named from memory).

### M6 land under ice, share of the land

Definition: land with glacier ice of at least 1 m (`ICE_MIN_M`, PLT-363), a share of the land (RLC-060). The registry's "outside a glaciation" has no definition in the census: it reads the final state only.

Earth today: 10 percent of the land (NSIDC, "Glacier Quick Facts", read), 10.7 percent in 2012 (USGS, quoted by Wikipedia, read); the Antarctic ice sheet 14 and Greenland 1.71 million km2 (Wikipedia "Ice sheet", read), together 0.105 of the 149.0 million km2 of land by the project's division. Today is an interglacial.

Earth's history (RLC-061, 062): at the Last Glacial Maximum, 26 to 20 ka with the sea about 125 m lower, ice covered 25 percent of the land (USGS via Wikipedia) or 32 percent (NSIDC). In a greenhouse state, "no continental glaciers exist anywhere on the planet" (Wikipedia "Greenhouse and icehouse Earth", tertiary): the Phanerozoic's greenhouse stretches are 420 to 360 Ma and 260 to 146 Ma, with icehouse states 450 to 420 Ma, 360 to 260 Ma (the Late Paleozoic ice age, peak near 300 Ma, no area figure found), 146 to 113 Ma and 33.9 Ma to today; the Antarctic ice sheet began about 34 Ma and the Arctic ones about 2 Ma. So Earth's range is 0 to 0.32, and 0.10 to 0.11 in an interglacial. The Snowball states of the Neoproterozoic are outside the Phanerozoic range and outside this page.

Proposed band: fires above 0.32 in any state; watch above 0.11 if the planet is not in an ice age. No floor: a greenhouse planet has no ice. The registry's conditional ("outside a glaciation") needs a definition of the glaciation in the model; this page proposes the unconditional ceiling and the watch line, and leaves the definition to the coordinator.

What a player sees. A planet at 0.17 (the master default's median) has one land cell in six under ice: more ice than Earth has today (0.105) and less than at the Last Glacial Maximum. A player sees the sheets as white land; it is within what Earth had 20,000 years ago. Above 0.32, more than a third of the land is buried: a snowball look, with most continents white. Below Earth's range nothing shows: a planet with no ice is Earth in the Cretaceous.

### M7 continental collisions per 100 Myr

Definition: the plate check's `collisions_per_100myr` (RLC-070): a terrane of at least `collision_min_frac` (0.1) of the average starting plate (initial_plates 14, so 510.07 / 14 = 36.4 million km2, and 3.64 million km2 for a terrane) that ends in a weld or a transfer, dated at its end. Smaller terranes are `terrane_accretion` events and are not counted; a collision still going on at the run's end is not counted either.

Earth: no source gives the rate for this definition. What exists:

- Condie, Pisarevsky, Korenaga and Gardoll 2015 (Precambrian Research 259:278-289), the abstract only (RLC-071): craton collisions peak at 1850 and 600 Ma and less at 1100 and 350 Ma, and are "very low" at 2200 to 2100, 1300 to 1100, 800 to 650 and 150 to 0 Ma; the frequency of craton collisions is not related to supercontinent assembly. The per-100-Myr figure quoted in PLT-157, "about one to 20 per 100 Myr, with an average of six" over 2.5 Gyr (RLC-072), came from a search-engine extract and is not in the abstract pages read: `cited-unverified`. It counts cratons of any size, a looser definition than the model's.
- Macdonald, Swanson-Hysell, Park, Lisiecki and Jagoutz 2019 (Science 364:181-184), read in full (RLC-073): six peaks of total suture length in 540 Myr for the major orogenies (about 1.1 per 100 Myr) and 15 to 46 ophiolite-bearing arc-continent sutures per 100 Myr (Paleozoic 44 in 290 Myr, Mesozoic 32 in 185, Cenozoic 30 in 65). The second is the model's `terrane_accretion`, not its collision.
- The writer's own list (RLC-074, `from-memory`): dates read in tertiary pages (the Caledonian Scandian phase 430 to 380 Ma, the Variscan meso-Variscan 380 to 340 Ma with stacking to 290 Ma, the Uralian from about 323 Ma to the Early Jurassic, the Cimmerian blocks in the Late Triassic, India against Asia from about 55 Ma and Arabia moving toward Eurasia since about 25 Ma, both still converging) and the writer's count of those that clear 3.64 million km2 and had ended: about 0 to 1 in 0 to 100 Ma (India, Arabia and Africa-Europe are still colliding, so none is dated yet), 1 to 3 in 100 to 250 Ma, and 2 to 4 per 100 Myr in 250 to 450 Ma. A geologist should rebuild it from the suture literature; the block areas are the writer's own.

Proposed band: fires above 6 per 100 Myr; watch above 4; no floor. The model's definition is stricter than Condie's, so its count for Earth is a subset of Condie's, whose average is 6 (the range one to 20 means single windows can exceed it). The writer's own count never exceeds 4, but it is unverified, so the margin to Condie's average stands in for the missing source: 6 is 1.5 times the writer's maximum. The watch line is that maximum. The registry's floor of 2 fails Earth: it had very few from 0 to 150 Ma. A count in one 100-Myr run is also noisy by itself: for a true rate of 2 the run shows fewer than 2 in 41 runs in 100 (Poisson), and for a rate of 3 it shows more than 6 in 3 runs in 100, so the pool's median over 8 or more runs, not a single run, is the number to read.

What a player sees. Far above 6 (the design's 2026-10 measurement had 10, range 9 to 12, `plate-collisions-and-welding.md` section 2.2) the continents weld and split every ten million years: the timelapse shows plates fusing and breaking again, and every mountain belt on the planet is young, which feeds M4 and M5 directly. Far below, at 0 to 1, nothing shows: a planet with no recent collisions has old, eroded ranges and no Himalaya, as Earth's last 100 Myr by the writer's count. A low count is invisible and realistic, a high one is neither.

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| M1 land | 0.292 | Marcilly et al. 2022 Table 1, read | 0.300 | D6 median, physics-gap-audit.md |
| M2 largest landmass | 0.156 | pipeline on SRTM15+ and the 0 Ma PaleoDEM | 0.124 | D6 median |
| M3 continental crust under the sea | 0.248 (20 km edge) | CRUST1.0 (HYP-042, PLT-361) | 0.278 | D6 median |
| M4 land above 2 km | 0.047 to 0.055 by rock; maximum of 109 maps 0.062 | HYP-041, PLT-360; PaleoDEMs | 0.271 | D6 median |
| M5 coast with high ground | 0.076 by rock; maximum of 109 maps about 0.10 at census resolution | PLT-360; PaleoDEMs | 0.411 | D6 median |
| M6 land under ice | 0.105 today; 0.25 to 0.32 at the Last Glacial Maximum | NSIDC, USGS via Wikipedia | 0.170 | D6 median |
| M7 collisions per 100 Myr | about 0 to 4 by the writer's count | RLC-074 (`from-memory`) | 1.80 | D6 median |

The mod's column is the median of the D6 arm as the audit printed it. This page ran no simulation and did not re-run the pool.

## Simplifications and limits

- **The PaleoDEMs are a model of the past, not data.** Their authors call them a first draft, say the highest elevations are likely overestimated, and the maps of the two phases are not homogeneous. The extremes quoted for M2, M4 and M5 are extremes of a model. The 0 Ma map differs from the real Earth (land 0.276 against 0.291).
- **Resolution.** The pipeline runs the census's helpers on a grid 111 km at the equator where the census's is about 16 km, and on 40,000 points where the census used 332,800. M5 reads 0.91 of the census's value on the control; maps are compared with each other and with the control, not with the census's own numbers.
- **Two sources disagree on the Pangaean land share** by up to 0.09, and the proposed M1 ceiling follows the higher.
- **The 0.044 offset for M3** is today's difference between two definitions of the continent's edge; whether it held in the past is not checked.
- **A range is not a distribution.** Earth's history is one path, its neighbouring maps are not independent, and a planet in a run is at an arbitrary stage. The bands say what has been, not what is likely.
- **Passing all seven is necessary, not sufficient.** The checks catch the planets that are clearly unlike any Earth has been; they do not say a passing planet is realistic.
- **M7 has no source for its own definition.** The count here is the writer's.
- **Present-day ice.** M6's Earth reference is two numbers from a web page and a quoted USGS figure; the Late Paleozoic ice age has no area.

## Game choices

- RLC-001: the rule that a V1 measure fires only outside Earth's range (Ben, decisions log 2026-10-23).
- RLC-002: how a band is set from Earth's range (fires at the Phanerozoic edge rounded outward, watch at the last 250 Myr). A proposal of this page's writer, not adopted.
- The proposed bands themselves, RLC-014, 021, 032, 042, 052, 063 and 075: proposals, not adopted. `tools/review/pool/measures.json` and `v1-item1-table.py` are untouched.
- The census's measuring choices (2 km, 200 km, 1 m of ice, 1.6 spacings) are PLT-363, the coordinator's of 2026-10-21.

## Open questions

Each with a recommendation; none blocks anything.

1. **Adopt the proposed bands?** Recommendation: yes for M1, M2, M3 and M7's floor, where the registry band is one Earth fails; optional for M4 and M5, where it is loose. Both fire on the master default either way.
2. **M6: unconditional or conditional on a glaciation?** The census has no glaciation flag, and the planet's final state is arbitrary. Recommendation: the unconditional 0.32 ceiling now, with 0.11 as a watch line; condition it only if the climate model exposes an ice-age state.
3. **M4 and M5 by rock.** The census's elevation is rock. Recommendation: keep it by rock and compare with Earth's by-rock figures (0.05 and 0.08), and let M6 carry the ice.
4. **M7's Earth rate for the model's own definition.** Recommendation: drop the floor of 2 now, keep a ceiling of 6, and have a geologist rebuild the count from a suture compilation (the Condie and Macdonald data sets are the candidates; the Macdonald code and data are public on GitHub, Swanson-Hysell-Group/Arc_Continent_Analysis, not downloaded here).
5. **Sources not read.** Condie et al. 2015 full text (abstract only), Kocsis and Scotese 2021 (ScienceDirect 403), Mitchell et al. 2021 and Nance 2022 (searched, not read for numbers), Zachos et al. 2001 (the abstract only). Recommendation: read Condie first.
6. **Which land-share history for M1's ceiling?** Marcilly's 0.29 or Scotese's 0.35. The proposal takes the higher; if the coordinator wants the narrower, use 0.30.

## Sources

The entries of [bibliography.md](bibliography.md) this page relies on, with how each has been read (RLC-010 to RLC-075):

- **Marcilly2022**: Table 1 (exposed land, continental crust, 10 Myr steps) and the paragraph on the hypsometric slope read in the PDF text, 2026-10-23.
- **ScoteseWright2018**: the report's caveats read; the PaleoDEM data used (1 degree, 5 Myr, 0 to 540 Ma), 2026-10-23.
- **Tozer2019**: the SRTM15+ grid used as the control, as in HYP-035; the paper not read.
- **Condie2015**: abstract only (repository pages), 2026-10-23.
- **Macdonald2019**: read in full.
- **NSIDC-GlacierFacts**: read, 2026-10-23.
- **WikipediaRealismChecks2026**: the Wikipedia pages read as tertiary sources, 2026-10-23.
- **Laske2013** and **Cogley1984**: through HYP-041, HYP-042 and PLT-361, not re-read.
