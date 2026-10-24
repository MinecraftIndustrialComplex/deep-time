# Geothermal features

Status of this page: `stage 2 (written 2026-10-02, WP-16), extended the same day with the heat-flow
fields, and on 2026-10-20 with the rule that a pool's water never freezes (GEO-062)`. Of the 62 rows, `verified` ones are the sinter and travertine rule (GEO-022), the Earth
heat-flow figures (GEO-059), the model-against-Earth arithmetic (GEO-060) and the threshold hint
(GEO-057); the rest are `game-choice` or `uncited`. Almost nothing in the feature's numbers comes from
a source: the sources checked below support the kinds of feature and the order in which they occur,
and (for the heat-flow threshold) the level of young extended crust, not a size or a count.

Ledger ids on this page: `GEO-001` to `GEO-062` ([ledger/GEO.md](ledger/GEO.md)). The editor hints
that describe these settings are rows GEO-047 to GEO-051. The geothermal gradient (the rise of
temperature with depth that drives metamorphism and the oil window) is a different thing and is
ledgered under rock and metamorphism and under deposits.

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and 50
blocks up.

## What you see in the world

Around the volcanoes that are still active at the end of the history, and on continental ground that
the plate model heats (rifts, volcanic arcs, hotspot swells), the ground changes colour in patches a
few blocks to a few tens of blocks wide. There are two kinds of place.

**Hot-spring fields.** A main pool of water with a radius of 2 to 5 blocks (4 to 10 blocks across,
which is 1.2 to 3 km; at the default size setting), and one to three smaller pools of radius 2 to 3
blocks. The pools may be set in a chain down a slope, like the steps of a terrace, and each is ringed
by a rim and a wider apron of white terrace rock. The terrace rock is sinter (a white siliceous
deposit; calcite blocks in the game) or, where carbonate rock lies within 150 blocks (3 km) below the
site, travertine (a pale calcium carbonate deposit; dripstone blocks). The water is plain water. The
bed under it, and the water's depth, change from the heart of the pool outwards: sinter or travertine
under two blocks of water in the middle 45 % of the radius, yellow terracotta under two blocks out to
75 %, orange terracotta under one block to the edge. These stand for the colour bands of microbial
mats in a real spring, which follow its temperature. A hot-spring field now and then has a vent or a
patch of altered ground a short way off.

**Fumarole fields.** Two to four vents (a core of magma blocks two blocks deep, with a crust of
yellow concrete standing for native sulfur around it) and one or two patches of altered ground, which
is bleached white terracotta with about 30 % of its cells stained yellow. At silicic volcanoes (those
that erupt silica-rich magma) a few cells are red terracotta, standing for cinnabar, a mercury ore.

Fields near the summit are always fumaroles. Farther out, a field is hot springs where the year's
rain is at least 400 mm and the annual mean temperature is at least 3 degC; otherwise it is a
fumarole field. Nothing is placed in water, under glacial ice, on ground less than 3 blocks above the
sea, or within a few blocks of the lava of a crater lake or plug. A volcano gets from none to seven
fields: few at small, quiet ones and most at large, active ones. On the 16k quick planet of the P2-14
build log there were 260 sites around 40 of the 67 volcanoes, of which 23 were pools, 142 vents and
95 patches of altered ground. On the quick 16k planet of 2026-10-02 (63 active volcanoes), the
volcanoes alone give 170 sites around 24 volcanoes (11 pools, 97 vents, 62 patches) and the hot
ground adds 97 (11 pools, 51 vents, 35 patches): 267 together.

Where the pack has a hot-spring biome (Terralith's yellowstone, Biomes O' Plenty's hot springs), every
hot-spring field lies in it, and the pack's other volcanic biomes sit on the active volcanoes, their craters
and calderas instead of wherever their climate falls (decisions log 2026-10-03; see
[biomes.md](biomes.md), "Volcanic biomes"). A pool within 20 blocks of a playa is the exception: it keeps its
rim and terraces but stands in the flat's biome, so that the pack's hot-spring features stay off the salt
(BIO-165).

A pool's water never freezes, whatever the temperature of the biome it stands in: a pack's hot-spring biome
can be much colder than the field's own climate (Biomes O' Plenty's hot springs have a temperature of 0.17,
cold enough at height for vanilla to ice a lake), and a real hot spring stays open through the winter because
its water is hot. The surface block of every pool column is exempt from vanilla's top-layer freezing when the
chunk is generated and from freezing in the weather afterwards (GEO-062); snow may still lie on the dry
apron and rim round it.

There are no geysers. The water does not steam, boil or erupt, and no temperature is stored. A
separate warm-ground signal lets another mod treat the ground in and beside a site as a few kelvin
warmer than the climate's temperature (at most 12 K at the heart of a vent).

## The model in plain words

The feature runs in the game, not in the simulation. The Rust bake writes for it six settings in the
`underground` section and one layer, `meta/heat_flow_mw_m2` (the plate model's heat flow on
continental crust); the game builds the sites once per level from the list of active volcanoes
(`volcanoes/active.json`), that layer, the baked rain and mean temperature, the ice cover, the strata
horizons and the terrain height (Java `worldgen/Geothermal`). The block pass
(`GeothermalPass`) runs in the surface step after the magma and before the ice, carvers and
features.

1. **Fields per volcano.** A volcano with basal radius R (in blocks, at least 6) and activity a
   (clamped to 0.05 to 1) gets n = floor(d x (0.7 + 2.3 a) x clamp(sqrt(R / 45), 0.6, 2.2) + u)
   fields, where d is the density setting and u is a uniform number from the volcano's own seed.
   That is 0 or 1 for the smallest, least active volcano (expected 0.49) and 6 or 7 for the largest,
   most active (expected 6.6). (GEO-014)
2. **Where.** The first field is 0.08 to 0.35 of R from the summit, the others 0.25 to 1.0 of R, all
   multiplied by the reach setting, in a random direction. A field is drawn again, up to twelve
   times, until its centre is dry ground at least 5 blocks above the sea with no water column and no
   ice. (GEO-015 to GEO-017)
3. **What kind.** A field is hot springs when it is at least 0.35 of R from the summit, the rain at
   the centre is at least the spring-rain setting, and the mean temperature is at least 3 degC. A
   world without climate layers counts as warm and wet. Otherwise it is a fumarole field.
   (GEO-018 to GEO-021)
4. **Sites.** Each field is made of sites: pools, vents and patches, each with a radius, an outline
   that wobbles by up to a quarter, and a seed for its stains. A site is refused where a column of it
   is wet, iced or less than 3 blocks above the sea, where it would come within 3 blocks plus its own
   hard radius of a magma body's open lake or plug, where it touches another site's pool, rim or
   vent, or (for a pool) where the ground is more than 6 blocks above or below the pool's level, the
   mean ground height of its footprint. (GEO-027 to GEO-032, GEO-034, GEO-036, GEO-042)
5. **Rock of the terrace.** Travertine if carbonate rock (limestone, dolostone, chalk, marble or
   travertine) lies within 150 blocks below the site in the strata horizons, else sinter.
   (GEO-022 to GEO-024)
6. **Blocks.** A pool is cut to its level, filled with water over a bed, and walled by a rim that is
   solid at the water's level and one block below it. Terraces, vents and altered ground repaint the
   top one or two ground blocks. Where two sites claim a column the one with the higher part priority
   wins. All blocks are vanilla ones behind tags (`deeptime:geothermal/<role>`), so a pack can swap
   them. (GEO-033, GEO-035, GEO-037 to GEO-040, GEO-043) The water's top block, at the pool's level,
   is kept open: the lake rule that stops vanilla's freezing on a lake the climate keeps open
   (`LakeFreeze`, through `BiomeFreezeMixin` on `Biome.shouldFreeze`, which both the generation
   feature `freeze_top_layer` and the weather call) also answers "does not freeze" for the column
   `Geothermal.poolSurface` names, whatever the biome. (GEO-062) Only that block: the lake rule
   itself answers only in a lake's own columns, so a lake beside a pool whose top block lies in the
   pool's water does not keep that block open (it did until 2026-10-23; HYD-072).
7. **Fields on hot ground** (2026-10-02; GEO-052 to GEO-056). Beyond the volcanoes, the world is cut
   into cells of 10,000 km2 (100 km across at the default density); each cell tries up to six random
   points and puts one field at the first where the baked heat flow is at least 100 mW/m2 (the setting)
   and the place is dry land at least 5 blocks above the sea, with no water column and no ice. The
   field is then built exactly as a volcano's is, as one "away from the summit": hot springs where
   the rain and the temperature allow, else fumaroles; no cinnabar. At most one field for each
   10,000 km2 cell, so the fields follow the hot ground: a broad hot province takes about one for
   each 10,000 km2, a thin belt a field in most of the cells it crosses.
8. **Warm-ground signal.** `DeepTimeClimate.geothermalWarmthC(x, y, z)` returns the warming in
   kelvin: a peak of 12 (vent), 10 (pool) or 6 (patch) that falls linearly to zero between the edge
   of the site's core and 1.1 times its altered-ground radius plus 4 blocks, within 10 blocks under
   (12 for a pool) and 6 over the ground. No mod reads it yet. (GEO-044 to GEO-046)

## Equations and constants

Every figure below is a game choice unless the status says otherwise. Sources are in the last
section.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| GEO-001, GEO-002 | Density setting | default 1.0, range 0 to 4 | multiple | none | `game-choice` |
| GEO-003, GEO-004 | Reach setting | default 1.0, range 0.25 to 3 | basal radii | none | `game-choice` |
| GEO-005, GEO-006 | Size setting | default 1.0, range 0.5 to 3 | multiple | none | `game-choice` |
| GEO-007, GEO-008 | Rain needed for hot springs | default 400, range 0 to 4000 | mm per year | none for the number; the rule is qualitative (NPS) | `game-choice` |
| GEO-014 | Fields per volcano | floor(d x (0.7 + 2.3 a) x clamp(sqrt(R / 45), 0.6, 2.2) + u) | count | none | `game-choice` |
| GEO-015, GEO-016 | Distance of a field from the summit | 0.08 + 0.27 u (first), 0.25 + 0.75 sqrt(u) (others), times R and reach | share of R | none | `game-choice` |
| GEO-017 | Height above the sea | field centre at least 5 blocks, site column at least 3 | blocks | none | `game-choice` |
| GEO-018 | Summit zone (always fumaroles) | 0.35 | share of R | pattern: Paulson and Ingebritsen 1991; number none | `game-choice` |
| GEO-019 | Hot-spring rule | share >= 0.35, rain >= setting, mean >= 3 degC | - | qualitative: NPS | `game-choice` |
| GEO-020 | Coldest mean temperature with pools | 3.0 | degC | none (a game rule; began as a guard against vanilla's freezing, which GEO-062 now does itself) | `game-choice` |
| GEO-062 | Pool water never frozen | the surface block of a pool column (y = the site's level) is exempt from `Biome.shouldFreeze`, at generation and in weather, in any biome | - | none read (a hot spring's water is hot) | `game-choice` |
| GEO-022 | Terrace rock | travertine over carbonate rock, else sinter | - | NPS; USGS | `verified` (the rule, qualitatively) |
| GEO-023, GEO-024 | Carbonate search | 150 blocks below the ground; five rock classes | blocks | none | `game-choice` |
| GEO-027, GEO-028 | Pool radius | main round((2.4 + 2.2 u) x size) clamped 2 to 9; smaller round((1.6 + 1.4 u) x size) clamped 2 to 6 | blocks | none | `game-choice` |
| GEO-029 | Apron beyond the pool | main r + 3 to r + 5; smaller r + 3 to r + 4 | blocks | none | `game-choice` |
| GEO-032 | Outline | radius x (1 + 0.16 sin(3 theta + p1) + 0.09 sin(5 theta + p2)) | - | none | `game-choice` |
| GEO-033 | Pool zones | core to 0.45 of the radius, yellow mat to 0.75, orange mat to the edge; water 2, 2, 1 blocks deep | share, blocks | NPS (colour follows temperature, qualitatively) | `game-choice` |
| GEO-034 | Cut and dam limits | 6 and 6 | blocks | none | `game-choice` |
| GEO-036 | Fumarole field | 2 to 4 vents of radius 1 (60 %) or 2 (40 %), 1 to 2 patches | count | none | `game-choice` |
| GEO-037 | Vent | magma-block core, 80 % native-sulfur crust out to 1.3 blocks beyond the radius | share, blocks | USGS HVO (sulfur forms at fumaroles) | `game-choice` |
| GEO-038, GEO-039 | Altered ground | disc of 3 to 11 blocks; about 30 % stained yellow, rest bleached | blocks, share | USGS (alteration colours) | `game-choice` |
| GEO-040, GEO-041 | Cinnabar | 6 % of altered ground at silicic volcanoes | share | none | `game-choice`, `uncited` |
| GEO-042 | Keep-clear distance from lava | 3 blocks + the site's hard radius | blocks | none | `game-choice` |
| GEO-052 | Heat flow for fields beyond the volcanoes | default 100, range 50 to 400 | mW/m2 | Davies and Davies 2010 (Cenozoic extensional 102.4; continents 70.9) | `game-choice` |
| GEO-053 | Hot-ground cell | 10,000 km2 / heat density; 6 draws a cell | km2, count | none | `game-choice` |
| GEO-054 | Heat density setting | default 1.0, range 0 to 4 | multiple | none | `game-choice` |
| GEO-055 | The exported layer | `meta/heat_flow_mw_m2`, u16, 0.1 mW/m2, nodata on oceanic crust | mW/m2 | the plate model (PLT-137) | `uncited` (a format choice) |
| GEO-044 to GEO-046 | Warm-ground signal | peaks 12, 10, 6; fade to 1.1 x altered radius + 4; band 10 (12 pool) under to 6 over | K, blocks | none | `game-choice` |

## Earth against the mod

Only numbers that were read in a source or measured. The mod's counts are from the P2-14 build log
(one planet, `earthlike_quick_16k`, the run named there); they were not remeasured in WP-16, because
no geothermal behaviour was changed.

| Quantity | Earth | Source of the Earth figure (how it was read) | The mod | Run |
|---|---|---|---|---|
| Hydrothermal features in Yellowstone | more than 10,000, of which more than 500 are geysers | NPS, Yellowstone hydrothermal features page (saved page, read) | geysers: none; sites on a 16k planet: 260 around 40 of 67 volcanoes | P2-14 build log, 16k quick planet |
| Temperature of a fumarole | as high as 138 degC (280 F) at Yellowstone | same NPS page | no temperature; warm-ground signal at most 12 K over the climate | - |
| Where high-temperature fields lie | on plate boundaries (arcs, the Mid-Atlantic Ridge in Iceland, the East African Rift) and on hotspots (Hawaii, Yellowstone) | WEC 2013, Geothermal, chapter 9 (saved text, read) | only around the active volcanoes of the baked list | - |
| Steam vents against springs on a volcano | fumaroles and steam-heated acid-sulfate springs at 1,800 to 2,500 m, chloride springs at about 1,500 m, Lassen, California | Paulson and Ingebritsen 1991, USGS WRI 90-4201 (saved text, abstract and introduction read) | fumaroles within 0.35 of the basal radius of the summit, springs beyond if wet and warm | - |
| Terrace rock | travertine where water rises through limestone (Mammoth); silica sinter in the geyser basins | NPS page above (read); USGS Yellowstone Volcano Observatory page (read through a page-fetch tool) | travertine over carbonate rock, else sinter | P2-14 build log: 1 of 23 pools was travertine |
| Native sulfur | forms at fumaroles; solid only below 115 degC | USGS Hawaiian Volcano Observatory, Volcano Watch (read through a page-fetch tool) | a yellow-concrete crust around every vent | - |
| Size of a spring | not read in a source | - | main pool 4 to 10 blocks across, 1.2 to 3 km | - |
| Mean heat flow of the continents | 70.9 mW/m2 (area 2.073e14 m2, 14.7 TW) | Davies and Davies 2010, Table 7 (full text, read) | the model's continental values run 55 to 200; 72 % of the quick 16k planet's dry land is at 70 or more | measured 2026-10-02, one baked planet (below) |
| Heat flow of young extended crust | Cenozoic extensional geology 102.4 mW/m2 (standard deviation 29.0; 16 observations); Quaternary 82.0; Tertiary 77.3; arc margins 73.8 | same, Table 3 | threshold 100: reached by an arc, a hotspot or a young rift in the model, not by a young orogen alone (90 at its start) | same |
| Heat flow against age | q = 102 - 2.25 sqrt(t), t in Myr, continents excluding the Precambrian | same, Eq. 2 (an empirical fit to category means, R squared 0.75) | 55 + 35 e^(-t/60) (+40 arc, +40 hotspot, +30 divergent): 90 at t = 0 against 102, 62 at 100 Myr against 79.5 | arithmetic, GEO-060 |
| Share of dry land hot enough | not read | - | at or over 70 / 80 / 90 / 100 / 120 / 150 mW/m2: 0.715 / 0.628 / 0.092 / 0.047 / 0.015 / 0.0002 | quick 16k planet, seed 1, 500 Myr, scattered continents (`tools/review/impacts-census.sh`, `GeothermalCensusTest`); another planet of the same preset gave 0.486 / 0.301 / 0.092 / 0.058 / 0.024 / 0 before the resampling fix below |
| Sites by heat source | - | - | volcanoes only: 170 (11 pools, 97 vents, 62 patches; 24 of 63 volcanoes); with hot ground: 267, the 97 new ones 11 pools, 51 vents, 35 patches | same planet, same code with `geothermal_heat_density` 0 and 1 |

## Simplifications and limits

- **Heat from volcanoes and continental heat flow only** (GEO-012, GEO-013; changed 2026-10-02).
  Earth's high-temperature fields lie on arcs, rifts, ridges and hotspots (WEC 2013); the mod now
  puts them at the active volcanoes and on continental ground whose plate-model heat flow reaches a
  threshold. Ridges are under the sea and get none. The plate model's continental heat flow is a
  simple sum (GEO-060) that is cooler than Davies and Davies's fit for young crust, and a young
  orogen alone reaches only 90 mW/m2, under the default threshold of 100, so collision belts get
  fields only where an arc, a rift or a hotspot also heats them.
- **No geysers, no steam, no boiling, no temperature** (GEO-026). Yellowstone alone has more than 500
  geysers. Water in the pools is plain still water; vanilla Minecraft has no hot water.
- **Sizes are not true scale.** A block cannot be smaller than 300 m across, so a pool of even four
  blocks is over a kilometre wide (GEO-027). Real springs are tens to a hundred metres or so; this was
  not sourced here.
- **Local rain stands for groundwater** (GEO-007, GEO-019). A real spring is fed by water that fell
  anywhere upstream in the aquifer, so a dry place can have springs; the baked annual rain at the
  field is only a stand-in.
- **No ice-covered fields and no cold pools** (GEO-020). A mean temperature under 3 degC gives
  fumaroles only. This is not geology: Yellowstone and Iceland have hot springs in cold climates
  (not sourced here). The rule began as a guard against vanilla's freezing, which would put ice on a
  hot pool; since 2026-10-20 a pool's water stays open whatever the biome (GEO-062), so the guard
  is no longer needed and the rule stands only as the game choice Ben confirmed on 2026-10-02.
  Dropping it would let hot springs stand in cold climates; that is his call.
- **Open water is a game rule, not a model** (GEO-062). The pool's water is not hot in the game
  (there is no temperature, GEO-026); it is only exempt from freezing. The warm-ground signal
  (GEO-044 to GEO-046) is separate, and the dry ground round a pool takes snow as the biome says.
- **Chemistry is reduced to two terrace rocks** (GEO-022). Real springs also deposit other
  minerals; acid-sulfate ground (opal, kaolinite, alunite) is shown only as white and yellow
  terracotta.
- **Cinnabar is a coloured stain** (GEO-041), tied to silicic volcanoes for no sourced reason, not to
  an ore deposit.
- **Distant Horizons far terrain** draws none of it: the features are a few blocks wide.

## Game choices

All numbers in the table above whose status is `game-choice`, decided by the P2-14 sub-agent (build
log P2-14, 2026-10-01) unless stated. Ben accepted the feature (decisions log 2026-10-01, "Geothermal
features"; and "Feel free to start now", on its own branch, GEO-011) but was not asked about any
number at the time. On 2026-10-02 he confirmed three of the rules as game choices (decisions log,
"Keep all three (Recommended)"): the 400 mm rain test for springs (GEO-007, GEO-019), no pools
under a 3 degC mean (GEO-020) and pools 4 to 10 blocks across (GEO-027). On 2026-10-20 the coordinator
took the rule that a pool's water is never frozen by its biome (GEO-062) under Ben's rule of
2026-10-16 (it can be undone by removing one line from `LakeFreeze.keepsOpen`). The default density and
the use of the warm-ground signal are open questions in `STATUS.md` (Pending). The block stand-ins (calcite for sinter, yellow concrete for sulfur, red
terracotta for cinnabar) are vanilla choices; Deep Time adds no block.

## Open questions

- Settled by Ben on 2026-10-02 ("Export heat flow (Recommended)"): the bake exports the plate model's
  heat flow and fields also stand on continental ground above a threshold (GEO-013, GEO-052 to
  GEO-056). Left to Ben: whether young orogens should qualify on their own (a threshold near 85
  would admit belts younger than about 20 Myr, and much of the continental interior would then
  be hot too: 63 % of dry land is at or over 80 on the planet measured, 30 % on another, because
  the share of young crust depends on the history), and whether 10,000 km2 a field is the right
  spacing.
- Settled by Ben on 2026-10-02 (kept as game choices): 400 mm of rain for springs (GEO-007), no
  pools under a 3 degC mean, whose reason is vanilla's freezing (GEO-020), and pools 4 to 10 blocks
  across (GEO-027).
- Should the cinnabar stain stay (GEO-041), and be tied to a silicic volcano?
- Should geysers be added (periodic eruptions of a pool)? None of the sources read says how many a
  planet of this size would have; Yellowstone has more than 500 (NPS).
- A geologist should check the zoning of springs on a volcano (GEO-018), the 150-block carbonate window
  (GEO-023) and the list of carbonate rock classes (GEO-024).

## Sources

Read in full or in part from the project's source library (not distributed):

- `NPS_YellowstoneHydrothermal`: National Park Service, "Hydrothermal Features", Yellowstone
  National Park (saved page, read): more than 10,000 features of which more than 500 geysers; hot
  spring water begins as rain; a fumarole has "so little water in its system that the water boils
  away before reaching the surface", steam vents reach 138 degC; travertine terraces form where
  thermal water rises through limestone; thermophiles live at specific temperatures and make
  gradations of colour.
- `PaulsonIngebritsen1991`: Paulson, K.M. and Ingebritsen, S.E. 1991. Sodium and chloride data from
  selected streams in the Lassen area, north-central California, and their relation to thermal-fluid
  discharge from the Lassen hydrothermal system. USGS Water-Resources Investigations Report 90-4201
  (full text; abstract and introduction read): the vertical zoning of steam-heated and chloride
  features.
- `WEC2013_Geothermal`: World Energy Council, 2013, World Energy Resources, chapter 9 Geothermal
  (full text; the passage on plate boundaries and hotspots read; the world average surface heat flow
  of 82 mW/m2 and the list of causes of shallow geothermal resources, among them a high surface heat
  flow from a thin crust and a high gradient, and the statement that warm springs are more numerous
  in areas of high regional conductive heat flow and of young tectonic activity).
- `Davies2010`: Davies, J.H. and Davies, D.R. 2010. Earth's surface heat flux. Solid Earth 1:5-24
  (full text; Tables 3 and 7 and Eq. 2 read, 2026-10-02): continents 70.9 mW/m2, Cenozoic
  extensional geology 102.4, the empirical fit q = 102 - 2.25 sqrt(t) (GEO-059).

Read through a page-fetch tool that returned quoted sentences; no copy saved (the agency site
returns only a script stub to a plain download):

- USGS Yellowstone Volcano Observatory, "Diverse chemistry of Yellowstone's hydrothermal features":
  silica sinter in the geyser basins, travertine at Mammoth, acid-sulfate waters as meteoric water
  with magmatic gases.
- USGS, "Alterations to go! Hydrothermal alteration in Yellowstone": acid-steam alteration gives
  opal, kaolinite, alunite and pyrite; yellow, orange and red rocks from alteration of rhyolite.
- USGS Hawaiian Volcano Observatory, "Volcano Watch: Many forms of sulfur are found on Kilauea
  Volcano" (2018): native sulfur forms at fumaroles (SO2 + 2H2S = 3S + 2H2O) and is stable as a solid
  below 115 degC.

Seen only as search summaries (not read; they support no `verified` status): White 1967 and the
Steamboat Springs and Sulphur Bank mercury deposits, for cinnabar in hot-spring systems (GEO-041);
Hurwitz and Lowenstern 2014, "Dynamics of the Yellowstone hydrothermal system", Reviews of
Geophysics 52:375-411 (abstract only: more than 10,000 thermal features; waters of mantle, crustal
and meteoric origin). Full text of the second could not be downloaded (publisher blocks scripted
access).
