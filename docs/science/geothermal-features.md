# Geothermal features

Status of this page: `stage 2 (written 2026-10-02, WP-16)`. One of the 51 rows is `verified` (the
sinter and travertine rule, GEO-022); 47 are `game-choice` and 3 are `uncited`. Nothing in the
feature's numbers comes from a source: the sources checked below support the kinds of feature and
the order in which they occur, not a size, a count or a threshold.

Ledger ids on this page: `GEO-001` to `GEO-051` ([ledger/GEO.md](ledger/GEO.md)). The editor hints
that describe these settings are rows GEO-047 to GEO-051. The geothermal gradient (the rise of
temperature with depth that drives metamorphism and the oil window) is a different thing and is
ledgered under rock and metamorphism and under deposits.

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and 50
blocks up.

## What you see in the world

Around the volcanoes that are still active at the end of the history, the ground changes colour in
patches a few blocks to a few tens of blocks wide. There are two kinds of place.

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
95 patches of altered ground.

There are no geysers. The water does not steam, boil or erupt, and no temperature is stored. A
separate warm-ground signal lets another mod treat the ground in and beside a site as a few kelvin
warmer than the climate's temperature (at most 12 K at the heart of a vent).

## The model in plain words

The feature runs in the game, not in the simulation. The Rust bake writes nothing for it except four
settings in the `underground` section; the game builds the sites once per level from the list of
active volcanoes (`volcanoes/active.json`), the baked rain and mean temperature, the ice cover, the
strata horizons and the terrain height (Java `worldgen/Geothermal`). The block pass
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
   them. (GEO-033, GEO-035, GEO-037 to GEO-040, GEO-043)
7. **Warm-ground signal.** `DeepTimeClimate.geothermalWarmthC(x, y, z)` returns the warming in
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
| GEO-020 | Coldest mean temperature with pools | 3.0 | degC | none (a vanilla freezing rule) | `game-choice` |
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

## Simplifications and limits

- **Heat only at the active volcanoes** (GEO-012). Earth's high-temperature fields also lie along
  ridges, rifts and extensional provinces. The plate model's heat flow and its rifts are not exported
  by the bake, so nothing is placed on them. A proposal to export a heat-flow layer is a Pending
  question (GEO-013).
- **No geysers, no steam, no boiling, no temperature** (GEO-026). Yellowstone alone has more than 500
  geysers. Water in the pools is plain still water; vanilla Minecraft has no hot water.
- **Sizes are not true scale.** A block cannot be smaller than 300 m across, so a pool of even four
  blocks is over a kilometre wide (GEO-027). Real springs are tens to a hundred metres or so; this was
  not sourced here.
- **Local rain stands for groundwater** (GEO-007, GEO-019). A real spring is fed by water that fell
  anywhere upstream in the aquifer, so a dry place can have springs; the baked annual rain at the
  field is only a stand-in.
- **No ice-covered fields and no cold pools** (GEO-020). A mean temperature under 3 degC gives
  fumaroles only. This is vanilla's freezing rule, not geology: Yellowstone and Iceland have hot
  springs in cold climates (not sourced here).
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
number. The default density and the use of the warm-ground signal are open questions in
`STATUS.md` (Pending). The block stand-ins (calcite for sinter, yellow concrete for sulfur, red
terracotta for cinnabar) are vanilla choices; Deep Time adds no block.

## Open questions

- Should fields also come from rifts, ridges and a heat-flow layer (GEO-013; Pending question,
  recommended: yes, via a heat-flow layer in the bake)?
- Is 400 mm (GEO-007) a sensible default, and should springs depend on the year's rain at all?
- Should a cold field (mean under 3 degC) have hot pools? The game-side reason is vanilla's freezing
  (GEO-020).
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
- `Janik_Lassen_WRI904201`: Paulson, K.M. and Ingebritsen, S.E. 1991. Sodium and chloride data from
  selected streams in the Lassen area, north-central California, and their relation to thermal-fluid
  discharge from the Lassen hydrothermal system. USGS Water-Resources Investigations Report 90-4201
  (full text; abstract and introduction read): the vertical zoning of steam-heated and chloride
  features.
- `WEC2013_Geothermal`: World Energy Council, 2013, World Energy Resources, chapter 9 Geothermal
  (full text; the passage on plate boundaries and hotspots read).

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
