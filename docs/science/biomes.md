# Köppen classes and biomes

Status of this page: `stage 2 (checked against sources on 2026-10-02, work package WP-11)`. The
Köppen rules were read in full in Peel, Finlayson and McMahon 2007 and two departures from it were
fixed. The band edges of Minecraft's biome builder were read in its source. The mapping from class to
biome is a game choice throughout and is labelled so. The climate that feeds the classifier (monthly
temperature and rain) is being rewritten and is not checked here.

Ledger ids on this page: `BIO-001` to `BIO-114` (see [ledger/BIO.md](ledger/BIO.md)).

## What you see in the world

Every land cell of the planet gets a Köppen class: a code such as Af (tropical rainforest), BWh (hot
desert), Cfb (temperate, rain all year, warm summer), Dfc (cold forest with a short summer) or ET
(tundra). The code is read off twelve monthly temperatures and twelve monthly rain totals. A cell
that is dry in the Köppen sense is a desert (BW) or a steppe (BS) even when it is cold: a high cold
plateau with little rain is cold desert, not tundra.

The class then decides which Minecraft biomes the cell may have. The class is placed on vanilla's
five temperature bands (frozen, cold, temperate, warm, hot) and five humidity bands, and the biome
source of the pack (vanilla, Terralith, Biomes O' Plenty) picks the biome inside that spot. A
desert class gets the driest humidity band, a rainforest class the wettest, tundra the frozen band
and so on. Wet, flat, low land that is not frozen becomes swamp (or mangrove where it is hot)
whatever its class, as in vanilla (BIO-066); "wet" is the cell's own rainfall against evaporation
(the aridity index), not the humidity band its class is lifted to, since 2026-10-03 (before, every
subarctic Dfc cell counted as wet because its class sits in vanilla's taiga humidity). Land under ice gets a snow and ice cover whatever the biome, frozen lakes follow the
mean annual temperature, and other mods can read the monthly climate of any block through
`DeepTimeClimate`.

## The model in plain words

1. **Classify.** After each climate snapshot, `climate/koppen.rs` takes the twelve monthly mean
   temperatures and twelve monthly rain totals of each land cell and follows Peel's Table 1. It
   first works out the dryness threshold from the mean annual temperature and the season in which
   the rain falls, and tests for a dry climate (B) first. Only a cell that is not dry is polar (E,
   warmest month under 10 °C), tropical (A, coldest month at least 18 °C), temperate (C, coldest
   month above 0 °C) or cold (D). The second letter says whether there is a dry season, the third
   how warm the summer is. The "summer half" of the year is the warmer of April to September and
   October to March, which Peel also uses.
2. **Place on the bands.** At bake time `bake/biomes.rs` and the table `placement.json` turn the
   class, the warmest and coldest month and the aridity index (rain over potential evaporation) into
   a temperature band coordinate and a humidity band coordinate between 0 and 5. The ranges are game
   choices, tuned so that each main group lands where vanilla draws its matching biomes. The
   aridity index goes through a log scale whose first three anchors are the UNEP class limits.
3. **Add the rest of the vanilla inputs.** Continentalness (from distance to the sea), erosion (from
   local relief) and weirdness (from ridge and valley shape) are filled so that vanilla's variants
   appear where the ground suggests them. All are game choices.
4. **Ice and snow.** Ground above its equilibrium line gets a cover of snow, packed ice and blue ice,
   up to 12 blocks; lakes with a mean annual temperature below 0 °C freeze.

## Equations and constants

The Köppen test, in the order the code runs it (`MAT` mean annual temperature in °C, `MAP` annual rain
in mm, `Thot`, `Tcold` warmest and coldest month in °C, `Pdry` driest month in mm, `ps`, `pw` rain of
the summer and winter half, `Psdry`, `Pswet`, `Pwdry`, `Pwwet` driest and wettest month of each half):

```
P_th = 2 MAT            if pw >= 0.7 MAP
     = 2 MAT + 28       if ps >= 0.7 MAP
     = 2 MAT + 14       otherwise
B  if MAP < 10 P_th (BW if MAP < 5 P_th, else BS; h if MAT >= 18, else k)
E  if Thot < 10 (ET if Thot > 0, else EF)
A  if Tcold >= 18 (f if Pdry >= 60; m if Pdry >= 100 - MAP/25; else w, or s if the driest month is in summer)
C  if Tcold > 0, else D
second letter: s if Psdry < 40 and Psdry < Pwwet/3; w if Pwdry < Pswet/10; f otherwise;
               if both s and w hold, w if ps > pw, else s
third letter:  a if Thot >= 22; else b if at least four months are above 10; else d if D and Tcold < -38; else c
```

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| BIO-002 | B tested before E | B first | | Peel et al. 2007, text under Table 1 | `verified`, fixed in WP-11 |
| BIO-003 | ET / EF split | Thot > 0 | °C | Peel, Table 1 | `verified` |
| BIO-004, BIO-005 | dryness threshold and B, BW, BS | 2 MAT + {0, 14, 28}; 10 and 5 times it | mm | Peel, Table 1 and note | `verified` |
| BIO-006, BIO-007 | h / k and A | MAT 18, Tcold 18 | °C | Peel, Table 1 | `verified` |
| BIO-008, BIO-009 | Af, Am, Aw | 60 mm; 100 - MAP/25 | mm | Peel, Table 1 | `verified` |
| BIO-010 | As | driest month in summer | | Wikipedia (Peel has no As) | `abstract-only` |
| BIO-011 | C / D split | Tcold > 0 | °C | Peel, Table 1 (after Russell 1931) | `verified` |
| BIO-012, BIO-013 | s and w, and their tie rule | 40 mm and 1/3; 1/10; wetter half wins | mm | Peel, Table 1 and p. 1639 | `verified`, tie rule fixed in WP-11 |
| BIO-014 to BIO-016 | a, b, c, d | 22 °C; 4 months above 10 °C; -38 °C | | Peel, Table 1 | `verified`, b fixed to "above" |
| BIO-017 | summer half | warmer of AMJJAS and ONDJFM | | Peel, Table 1 note | `verified` |
| BIO-021 to BIO-024 | vanilla band edges | temperature, humidity, continentalness, erosion, weirdness | | `OverworldBiomeBuilder`, Minecraft 1.21.1 | `verified` |
| BIO-038 | aridity anchors 0.03, 0.2, 0.5 | UNEP limits | | Zomer et al. 2022, quoting UNEP 1997 | `verified` |
| BIO-095 | precipitation seasonality | coefficient of variation | % | O'Donnell and Ignizio 2012 (adds 1 mm to the mean) | `verified` |
| BIO-026 to BIO-070, BIO-075 to BIO-081, BIO-100 to BIO-103 | the placement table, knobs and ice cover | see ledger | band coordinate, blocks | none: game choices | `game-choice` |

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Land share of the five groups | A 19.0 %, B 30.2 %, C 13.4 %, D 24.6 %, E 12.8 % | Peel et al. 2007, section 4, read in full text | A 18 %, B 21 %, C 21 %, D 12 %, E 28 % (area-weighted, last climate snapshot) | `earthlike_quick_16k`, scattered continents, 500 Myr, default seed, 2026-10-02 |
| Class changes from the test-order fix on that planet | | | 2,703 of 132,955 land cells (2.0 %) changed class: ET 2,602 and EF 101 became BWk (1,425) and BSk (1,278); B rose from 12.7 % to 14.7 % of land cells, E fell from 49.7 % to 47.7 % | the same run before and after the fix, same climate (identical snapshots) |
| Cs / Cw tie rule on that planet | | | 19 cells moved from s to w (Csb -14 and Cwb +14; Dsb -2 and Dsc -3, Dwb +2 and Dwc +3) | the same pair of runs (the "after" binary predates the change of Tmon10 to "above 10", which can only matter for a month at exactly 10.0 °C) |
| Agreement with observed Köppen classes | 100 % by definition | Beck et al. 2018 (not in the library) | exact class 34.6 % and main group 64.9 % by land area, on Earth's relief with today's climate model | design document `p1-26-per-cell-climate.md`, section 1.2; this number tests the climate physics, not the classifier |

The mod's planet is not Earth. The group shares above are those of a random 500 Myr planet whose
climate is colder than Earth's (mean land temperature 7.6 °C at the end of the run); they show what
the classifier does with it, not whether the climate is right. The classifier itself is checked
against published values: Peel's worked example, Herberton Post Office in Australia (Table 2), is
Cwa and its dryness threshold is 678 mm (unit tests `herberton_is_dry_winter_not_dry_summer` and
`the_b_threshold_is_the_published_678_mm`).

### What the fixes changed in the game

After the fixes, `./gametest.sh` and `./gametest.sh -PwithMic` each pass all 68 required tests,
and every Köppen family is above its `climateConsistent` floor, so no floor was changed. The shares
below are the share of a family's columns whose biome carries a matching tag, on the 4k test planet
(`test_planet_real`, default seed, 2026-10-02):

| Family | Columns (standalone / MIC) | Share standalone | Share MIC | Floor |
|---|---|---|---|---|
| A | 368 / 350 | 0.707 | 0.789 | 0.45 |
| BW (dry tags) | 189 / 175 | 0.820 | 0.903 | 0.57 |
| BS | 255 / 286 | 0.686 | 0.647 | 0.42 |
| BW desert or badlands | 189 / 175 | 0.651 | 0.760 | 0.37 |
| E | 2,105 / 2,199 | 0.998 | 0.999 | 0.82 |

No GameTest was run on master before the change, so there is no paired "before" for these shares.
The paired before-and-after measurement is the bake-level one in the table above (the 16k planet,
same climate, only the classifier differs): 2.0 % of land cells changed class, all by the dry-before-polar
rule apart from 19 by the Cs/Cw rule. On the test planet BW has 189 columns (standalone), of which
the share of BW columns with a desert or badlands biome is 0.651, against 0.475 to 0.827 measured
over ten planets before (floor comment in `BiomeGameTests`), so the cold deserts did not pull it
outside the earlier range.

## Cold deserts

Köppen tests dry land before polar (Peel et al. 2007, Table 1; BIO-002), so cold dry land is BWk or BSk and not ET or EF.
Vanilla Minecraft has no cold desert: its desert is sand and its badlands are red, both in the hot band. Ben's answer
(decisions log 2026-10-02): "This feels like a gap in Vanilla biomes, classify them properly and if a mod a player
brings has a better biome then swap to it. Otherwise just go with our best bet in terms of other cold bimoes."

What the mapping does (`placement.json`, `classes.BWk`; ledger BIO-101 to BIO-103, all game choices):

- BWk keeps its own class through the whole bake, and splits on the mean annual temperature. Above 10 °C it is the hot
  desert as before (ramp `desert`). From 0 to 10 °C it is the cold band at the driest humidity (ramp `cold_desert`,
  1.05 to 1.95, humidity at most 0.6). At or below 0 °C it is barren frozen ground like EF's (ramp `polar_ice`,
  humidity rule `polar_desert`).
- BSk was never sand: it takes the C or D ramp by its coldest month (BIO-033) and lands on plains, as before.
- A biome mod is not special-cased. Its biomes enter through the same band coordinates, so a mod's cold desert or cold
  steppe is used wherever the mod put it on vanilla's temperature and humidity bands. Terralith (shipped by MIC) puts
  cold shrubland, steppe, gravel desert and temperate highlands on the cold and frozen bands at low humidity, and
  Biomes O' Plenty puts `cold_desert` there (TerraBlender regions, which follow the geologic provinces; P1-10).
- With no mod, the best fit in vanilla is the cold band's driest open biome, plains (vanilla's cold-band, humidity-0
  entry, a treeless grass plain), and in the frozen band snowy plains. They are the treeless, low, dry-looking cold
  biomes vanilla has: not taiga (a forest), not a mountain biome (those follow height, not climate), and not desert
  (sand is the one thing a cold desert is not). The 10 °C and 0 °C switches are the agent's; no source gives where a
  desert stops being drawn as sand (BIO-101).

Measured on one `earthlike_quick_16k` planet (scattered continents, 500 Myr, seed 1, test machine, 2026-10-03): the
same world baked twice, once with the table before this change (given to the bake as a `climate_placement` content
bundle) and once with the new one; the world data differ only in the temperature and vegetation layers (byte compare of
the other rasters, strata and events). `/deeptime probe biomes 16` through each profile's biome source,
land columns of class BWk (10,541 columns; BSk 16,642 columns, which hardly moves):

| BWk columns on | Before | After |
|---|---|---|
| standalone: desert / plains / snowy plains / badlands / eroded badlands | 68.2 / 3.3 / 0.0 / 7.8 / 4.7 % | 40.5 / 36.9 / 3.0 / 1.9 / 0.7 % |
| MIC: vanilla desert / Terralith cold shrubland / temperate highlands / snowy slopes / snowy plains / plains | 26.8 / 0.0 / 0.0 / 0.2 / 0.0 / 2.6 % | 9.2 / 14.2 / 9.8 / 2.1 / 1.5 / 4.9 % |
| MIC: vanilla badlands / eroded badlands | 14.9 / 7.1 % | 5.4 / 1.8 % |
| Biomes O' Plenty: vanilla desert / BOP cold_desert / vanilla plains | 24.1 / 0.3 / 0.9 % | 12.5 / 3.5 / 16.1 % |

Over the land of that planet (138,101 land columns): BWk is 7.6 % of it, and the new table redraws 3.4 % of the land (4,750 columns, 45 % of the BWk) in standalone, 3.5 % (4,909 columns) with MIC; the rest of the BWk is above 10 °C and stays desert.

(The desert that is left is the BWk above 10 °C: the Gobi at a mean of 3 to 6 °C is not in it, the Atacama coast at 16 to 18 is; those means are from memory, see BIO-101.
BOP's shares are not asserted anywhere: its regions fill whole provinces, so they follow province draws.)
The GameTest `climateConsistent` has a check for this, `BWk:cold_not_sand` (BiomeCensus): of the BWk columns colder
than the cold band's top, none may land on a sand desert or a badlands; it reads 0 of 65 columns standalone and with MIC, and 0 of 1
column with TerraBlender (Biomes O' Plenty's columns are left out of that census). The BW family shares (`BW`, `BW:desert`) now leave those cold
columns out (`BiomeCensus.Check.notColderThan`), because they are not meant to be desert any more; the floors did not
change. On the 4k test planet (201 columns left in BW): BW 0.692 standalone, 0.766 MIC (floor 0.57), BW:desert 0.502 and
0.592 (floor 0.37). Without the exclusion the same planet read BW 0.457 and BW:desert 0.297 standalone and BW 0.543 MIC,
which failed the floors: the shares fell because 65 of 266 columns became plains by design, not because the mapping broke.

## Volcanic biomes

Biome mods bring volcanic biomes: Terralith (shipped by MIC) has yellowstone (hot springs and geysers),
volcanic peaks, volcanic crater, caldera, ashen savanna, basalt cliffs and the cave biome thermal caves;
Biomes O' Plenty has volcano, volcanic plains and hot springs. A biome source places each of them where its
climate parameters fall, so before this change they came wherever the climate point happened to match,
unrelated to the planet's volcanoes, and the planet's real volcanoes, calderas and hot springs stood in
whatever biome the climate gave. Ben's answer (decisions log 2026-10-03): "Tie them to real volcanoes
(Recommended)": where the pack has them they go only at real volcanoes (hot-spring fields, active cones,
calderas, fresh ash) and are kept out everywhere else; without them (vanilla) the climate's biome stays.
This follows the cold-desert answer: a mod's better biome when the player has it, otherwise the best vanilla
pick.

What the game does (Java `worldgen/VolcanicZones`, `climate/VolcanicBiomes`; ledger BIO-104 to BIO-114, all
game choices but the vanilla depth band of BIO-112):

- **Kept out.** Every biome in the tag `#deeptime:volcanic` is taken out of the climate search of a Deep Time
  level. The search still runs on the pack's own search tree (the region's, with TerraBlender), with every
  volcanic leaf set infinitely far, so the place gets the next-nearest biome of the same tree: the pack's own
  second choice for that climate point. Nothing is filled with a fixed biome. One hole did open: Biomes O'
  Plenty puts its volcanic plains in vanilla's mushroom-fields slot (land surrounded by deep sea), and the
  next-nearest biome of its region there is the lukewarm ocean; with TerraBlender such a slot takes the
  vanilla region's biome instead (measured below).
- **Put in.** Zones are discs around the active volcanoes of the baked list, their craters and calderas,
  their hot-spring pools and their magma (FORMAT.md, "Volcanic biomes"). In priority order:

| Role | Shipped tag order (first the pack has) | Where |
|---|---|---|
| hot-spring field | terralith:yellowstone, biomesoplenty:hot_springs | around every hot-spring pool (from a volcano or from hot ground), out to the pool's terraces + 4 blocks |
| caldera floor | terralith:caldera | 0.9 of the caldera's rim radius |
| summit crater | terralith:volcanic_crater, terralith:volcanic_peaks, biomesoplenty:volcano | the crater's rim + 4 blocks |
| cone (summit, upper flanks) | terralith:volcanic_peaks, biomesoplenty:volcano | 0.45 of the basal radius, stratovolcanoes and shields |
| fresh ash | terralith:ashen_savanna, biomesoplenty:volcanic_plains | out to the basal radius, stratovolcanoes and caldera complexes of activity 0.75 or more, where the climate allows |
| young lava | terralith:basalt_cliffs, biomesoplenty:volcanic_plains | out to the basal radius, shields, where the climate allows |
| thermal caves (underground) | terralith:cave/thermal_caves | 1.5 chamber radii round the magma, from the biome's own depth down |

  A surface role holds only on land and above the cave band (biome depth under 0.2, vanilla's cave biomes
  start there: BIO-112), and never on a river's channel. "Where the climate allows" means the climate point
  lies inside that biome's own temperature and humidity ranges in the pack's parameter list (Terralith puts
  ashen savanna on the warm band at dry to middling humidity), so the rule follows whatever biome a pack
  lists. A role without a biome in the pack, or not allowed at the spot, gives way to the next role whose
  disc holds the place and in the end to the climate search. Every disc is at least 8 blocks in radius, two
  biome cells, and a field ring narrower than 8 blocks is dropped, so the edges blend into areas, not specks.
  The setting `volcanic_biome_reach` (1, 0 to 3) scales the cone and the fields; 0 places none.
- **Datapacks.** Both lists are tags: a pack adds a biome to a role tag (and it is kept out elsewhere too) or
  to `#deeptime:volcanic` alone (kept out, never placed), or reorders a role tag with `"replace": true`.
- Without such a biome in the pack nothing changes: the level's biome queries skip the whole layer (the
  GameTest `volcanicBiomesChangeNothingWithoutThem` checks every sampled column on the vanilla profile).

Measured on one `earthlike_quick_16k` planet (scattered continents, 500 Myr, test machine, 2026-10-03; 110 active
stratovolcanoes, 45 of them oceanic, no shield and no caldera; 50 hot-spring pools), `/deeptime probe biomes 16`, whose
`volcanic` section samples the same 51,574 land columns as the plain climate search places them (the placement before this
change) and as the game now does:

| | MIC (Terralith) before | after | Biomes O' Plenty before | after |
|---|---|---|---|---|
| land columns in a volcanic biome | 490 (0.95 %) | 591 (1.15 %) | 358 (0.69 %) | 557 (1.08 %) |
| of them more than 256 blocks from every active volcano and hot-spring pool | 285 (58 %) | 0 | 356 (99 %) | 0 |
| of them more than 8 blocks outside every zone | 459 (94 %) | 0 | 358 (100 %) | 0 |
| hot-spring pools in the hot-spring biome | 2 of 50 | 50 of 50 | 0 of 50 | 50 of 50 |
| volcanic surface cells with no neighbour of their biome | 10 of 525 | 26 of 9,797 | 0 of 5 | 37 of 9,538 |

Before, Terralith's yellowstone (378 of the 490 columns) lay wherever its cold-band, mid-humidity box fell, and BOP's volcanic
plains (all 358) almost all far from any volcano. After, MIC shows volcanic peaks (341 columns), yellowstone (154), ashen
savanna (53) and volcanic crater (43); BOP its volcano (384), hot springs (154) and volcanic plains (19). The places the kept-out
biomes left went, with MIC, to taiga (173), Terralith's white cliffs (110), jagged peaks (37), savanna (36), forest (35); with
BOP to its rainforest (176), the vanilla region's beach (109: the slot fix above) and floodplain (51). The vanilla profile
changed no column. The zones on that planet: 50 hot-spring discs (mean 81 km², the pools' terraces plus margin, at least 8
blocks), 110 crater discs (20 km²), 109 cone discs (187 km²), 64 fresh-ash discs (1,180 km², of which the climate gate let 53
sampled columns become ashen savanna) and 110 thermal-cave discs (38 km²). These sizes are the game's choices; no real
landscape was measured for them.

## Expanded Ecosphere and the biome rules

Expanded Ecosphere (EE, 3.4.7 for NeoForge 1.21.1, by William Wythers and Cristelknight; CC BY-NC-ND 4.0) has
189 realistic biomes, all of vanilla blocks, and fills most of the gaps Terralith leaves (the biome-mods survey,
docs/research/notes/biome-mods-survey.md). Ben (decisions log 2026-10-04): "Build out support for both so if I
install EE it works or if I go w/ terralith it also works". Deep Time never ships, edits or redistributes EE; it
ships a data file of its own that names EE's biome ids.

**What EE did to a Deep Time world.** EE's built-in datapack `ee_default` (registered through Cristel Lib, on in
its default mode, the mode without TerraBlender) defines `minecraft:overworld` as a dimension: vanilla's noise
generator with an inline list of 4,589 biome entries, and its own `minecraft:overworld` noise settings (its surface
rule). Vanilla's world loader takes a dimension from the datapacks first and from the world preset only where no
datapack defines it, so a world created with Deep Time's preset and EE installed was an ordinary EE world: on the
16k test planet the server started, Toroidal World still looped it every 16,384 blocks, and `/deeptime probe
biomes` answered "minecraft:overworld is not a Deep Time level on world data". Now (Java `worldgen/OverworldOverride`):
in a Deep Time world the datapack's overworld is set aside, Deep Time's generator stays, and it takes the datapack
overworld's biome list in place of vanilla's (the same for every other Deep Time level a datapack defines, such as
the GameTest planets); EE's surface rule paints the terrain as Terralith's does in MIC. A world on a vanilla preset
keeps EE's override, as EE intends. One part of EE's surface rule is left out of Deep Time levels: a layer of smooth
basalt below y −55, drawn for vanilla's 384-block world, which in a Deep Time world (floor at −512) replaced every
rock class below −55 (BIO-131; a pack's top-level layers between two fixed heights, other than vanilla's deepslate
and the bedrock floor, are left out; the strata are the rock there).

**Biome rules.** EE's own list puts biomes where Deep Time's climate coordinates never reach: its ice cap below
vanilla's lowest temperature (−1.2 to −1.0), its salt lakes, mud pools and Tibesti mountains above the warm end
(0.9 to 1.2), its volcanoes at a continentalness under −1.05 (deep-sea islands). And it puts others where Deep
Time's classes are not: chaparral in the driest hot cells, so 40 % of the planet's hot desert came out chaparral;
temperate rainforest in the wettest cold cells, so 28 % of the humid boreal forest (Dfc) came out temperate
rainforest; snowy taiga and ice spikes in the tundra. A rule file (`data/<ns>/deeptime/biome_rules/*.json`,
FORMAT.md "Biome rules"; Java `climate/BiomeRuleSet`, `climate/BiomeRules`) places a pack's biomes from Deep Time's
own coordinates: the Köppen class, the climate at the column in °C and mm, the wetland flag, the ice cover, the
playas, the volcanic zones and the rock at the surface. The first rule that holds runs the pack's own climate
search over that rule's biomes alone, so the pack still picks among them by its ranges (a hot-spring field takes
the snowy or the plain thermal taiga by temperature). A rule's biomes may be exclusive: kept out of the climate
search elsewhere, as the volcanic biomes are; a file may also keep biomes out without placing them anywhere: EE's
ten island biomes and its mushroom fields sit at a continentalness Deep Time gives the deep sea, where on the
GameTest planet they took 17 % of the sea floor (BIO-132). The same machinery serves every pack: the volcanic roles' tags list
EE's volcanoes after Terralith's and Biomes O' Plenty's biomes, and the peatland rule's tag lists every pack's bogs.

| Rule (ledger) | Where (Deep Time's coordinates) | EE biomes | Basis |
|---|---|---|---|
| icy / tropical volcano (BIO-117) | an active cone in E or Dxc/Dxd; in A | icy_volcano; tropical_volcano (elsewhere the role tag's volcano) | game choice |
| thermal taiga / mud pools (BIO-118) | a hot-spring field in D or E; elsewhere | thermal_taiga, snowy_thermal_taiga, thermal_taiga_crags; mud_pools | game choice |
| glacial cliffs (BIO-119) | a coast under the ice cover | glacial_cliffs (exclusive) | game choice |
| ice cap (BIO-119) | land under the ice cover, but its peaks | ice_cap (exclusive) | game choice |
| salt lakes (BIO-120) | a playa (the hydrology's salt flat) | salt_lakes_white, _pink, _turquoise (exclusive) | game choice |
| tsingy (BIO-121) | carbonate at the surface in A, Cwa, Cfa | tsingy_forest (exclusive) | game choice (tropical tower karst) |
| black beach (BIO-122) | a beach in a volcano's zone or on basaltic rock | black_beach (exclusive) | game choice |
| Tibesti, hot desert (BIO-123) | BWh above 1,500 m; BW with a mean above 10 °C | tibesti_mountains (exclusive); EE's deserts and badlands | game choice |
| tundra (BIO-124) | ET with a warmest month of 5 °C or more; colder ET | tundra; snowy_tundra | CAVM subzone C edge (Walker et al. 2005, from memory) |
| polar desert (BIO-125) | EF off the ice, BWk at or below 0 °C | snowy_tundra | game choice |
| dry tropical forest (BIO-126) | Aw, As with 1,000 mm or more | dry_tropical_forest | Holdridge's dry forest band (from memory) |
| temperate rainforest (BIO-127) | Cfb, Cfc with 1,400 mm or more and a mean of 4 to 12 °C | temperate_rainforest, its crags, its flooded form (exclusive) | Alaback 1991 (from memory) |
| Mediterranean scrub, cold steppe (BIO-128) | Csa, Csb with 800 mm or less; BSk | chaparral; cold_plains, highland_plains, subtropical_grassland | game choice |
| boreal peatland (BIO-129; every pack) | cold wetland | `#deeptime:peatland`: fen, berry_bog, snowy_bog, snowy_fen | game choice (Ben, 2026-10-03, "peatland if present") |

**Measured** on one `earthlike_quick_16k` planet (scattered continents, 500 Myr, seed 1, test machine, 2026-10-04;
`tools/review/ee-biomes.sh`, `tools/review/profile_shares.py`), the same world data through each profile's biome
source, `/deeptime probe biomes 16`, 93,727 land columns. Before (master's jar with EE): no Deep Time overworld at
all. After, with EE, the rules decided 36,891 of 94,657 land columns (39 %; the census's own count, poles
included), the volcanic roles 107, the climate search the rest; no column a rule decided lay outside the rule's
classes or held a biome it does not name, no exclusive biome came from the climate search, and 94 of 97
cold-wetland columns were on a peatland biome. The commonest biomes per class (share of the class's columns):

| Class (columns) | EE | MIC (Terralith) | Vanilla |
|---|---|---|---|
| Af (3,452) | tropical_rainforest 65, jungle 20 | sparse_jungle 39, tropical_jungle 19 | jungle 69, bamboo_jungle 24 |
| Am (1,507) | tropical_rainforest 71, sparse_jungle 8 | shrubland 29, jungle 27, brushland 21 | jungle 74, sparse_jungle 10 |
| Aw (7,192) | dry_tropical_forest 60, savanna 14, wooded_savanna 12 | arid_highlands 43, savanna 23 | savanna 76, forest 13 |
| BWh (1,561) | eroded_badlands 32, cactus_desert 30, desert 14 | desert 38, badlands 27 | desert 62, stony_shore 12 |
| BSk (14,077) | cold_plains 50, subtropical_grassland 26, highland_plains 10 | plains 57, highlands 14 | plains 82, windswept_hills 6 |
| Csa (1,469) | chaparral 87 | plains 46, highlands 25 | plains 70, windswept_hills 13 |
| Csb (2,921) | chaparral 69, forest 10 | plains 60, birch_taiga 10 | plains 63, forest 23 |
| Cfb (7,501) | forest 14, autumnal_birch_forest 13, birch_forest 11 | old_growth_birch_forest 19, forest 16 | old_growth_birch_forest 30, forest 29 |
| Dfc (6,337) | taiga 61, pine_barrens 11, snowy_taiga 8 | taiga 36, forested_highlands 14 | taiga 50, old_growth_spruce_taiga 19 |
| ET (9,952) | tundra 58, ice_cap 16, snowy_tundra 8 | cold_shrubland 24, snowy_slopes 17, snowy_plains 15 | snowy_plains 67, snowy_slopes 10 |
| EF (3,362) | ice_cap 88, glacial_cliffs 6 | frozen_peaks 34, snowy_slopes 33, gravel_desert 11 | snowy_slopes 51, ice_spikes 20 |
| all land: biomes, share of the mod's own | 141, 0.69 | 88, 0.42 | 40, 0 |

The gaps each pack closes on this planet: **EE** closes peatland (fen and bog on 97 % of the cold wetland), summer
tundra, the ice sheet as a biome, tropical dry forest, Mediterranean scrub, cold steppe, salt lakes on the playas
(41 columns), tropical karst (tsingy, 128), temperate rainforest (68), black-sand beaches (56) and climate variants
of volcanoes and hot springs (icy volcano 121 columns, mud pools 125, thermal taiga 19). **Terralith** keeps its
known gaps (monsoon forest as dry shrubland and brushland, 50 % of Am; plains for Mediterranean and cold steppe; no
peatland, no dry forest, no salt or karst biome) and is still the better set for volcanoes (its caldera and ash
biomes, which EE lacks) and geology-named landforms. **Vanilla** closes none of them. EE's distinct biomes on land
are 141 against Terralith's 88. The candidates a rule names that this planet's climate never reached: berry_bog,
snowy_fen, salt_lakes_pink.

## Terralith's placement table and the geology gate

The biome-mods survey (research note `biome-mods-survey.md`, sections 7 to 9) measured Terralith, MIC's biome mod, on
a real 16k planet: monsoon forest drawn as dry brushland, Mediterranean land as plains, cold steppe never on
Terralith's own steppe, tundra too snowy, Terralith's cloud forest never reached, and its rock-named biomes (Shield,
Yosemite, the White, Granite and Basalt Cliffs, the red-bed canyons, Stony Spires) wherever their parameters fell, on
any rock. Ben's answers (decisions log 2026-10-04): "Deep Time-side fixes (Recommended)", no Deep Time biomes this
round; keep the fantasy biomes. Terralith's licence forbids editing or overriding its files, so every fix lives in
Deep Time's own data.

**A placement table per pack.** The climate placement table (above) is written for vanilla's slots. A biome mod
whose biomes sit elsewhere gets a table of its own: a JSON merge patch on `placement.json`
(`data/climate_mapping/packs/<id>.json`), used only when every biome it names in `requires_biomes` is registered.
A dedicated server's first start chooses the tables from its biome registry, the Create World screen from the
data packs it has loaded; the bake merges them and records their ids (FORMAT.md, "Pack placement tables"). A
vanilla world, or one with Biomes O' Plenty, bakes exactly as before: the default table is untouched. A table may
also hold *class rules*: overrides for a class that hold only where conditions on the cell (temperatures, P/PET,
latitude, height) do.

Terralith's table (ledger BIO-140 to BIO-146; every value a game choice, tuned against Terralith 2.6.2's parameter
list with an offline copy of the game's climate search that agrees with the in-game census on 98.8 % of land
columns, `tools/review/terralith_emulate.py`):

| Class | Change | Why (Terralith's slot) |
|---|---|---|
| Am (monsoon forest) | humidity band 4.05 and up | Terralith's jungles sit in the wettest band; band 3 is its brushland and shrubland |
| Csa, Csb, Csc (Mediterranean) | humidity squeezed into 0.1 … 0.6 | Terralith's shrubland, a dry scrub, is the driest part of the dry band (humidity below −0.585); plains and its alpine highlands are above |
| BSk (cold steppe) | the same dry humidity, erosion held to 0.12 … 0.4 | Terralith's steppe needs erosion 0.05 to 0.40 in the pack's own units; MIC's climate mapping (Tectonic's router) turns 0.12 … 0.4 into about 0.12 … 0.33 |
| ET (tundra) | the positive branch of the variant noise | in the frozen band's humidity band 1 the negative branch is vanilla's snowy plains, the positive Terralith's gravel desert (and frozen peaks, not jagged ones). Holding ET's humidity at 1.5, the survey's proposal, was measured and changed nothing: most ET is in band 1 already |
| Cfb, Cwb, Cfc, Cwc below 25° latitude, above 1,000 m, P/PET above 1 | rule `cloud_forest`: the cold band, humidity 3.05 and up, erosion 0.62 and up, far inland | Terralith's cloud forest has one slot: cold band, humid, the swamp band's erosion, far inland |

The cloud forest rule follows the tropical montane cloud forest: lowland forest gives way to lower montane forest
where the mean minimum temperature drops below 18 °C, "on large equatorial inland mountains ... at an altitude of
1200–1500 m.a.s.l.", and regional lower limits of cloud forest run from 1,000 to 2,000 m (Scatena et al. 2010, read;
BIO-145). The latitude limit (about 23° N to 25° S) is from a secondary snippet, not read (BIO-144, `from-memory`).
The Köppen classes C at tropical latitudes are mountain climates by construction (a coldest month under 18 °C at low
latitude needs height), so the rule asks the classes rather than a temperature.

**The geology gate.** A rock-named biome stands only on the ground it is named for (BIO-147 to BIO-153): Shield on
Precambrian shield provinces, Yosemite on exposed granite, the White, Granite and Basalt Cliffs on coasts of
carbonate, granite and basalt, Bryce Canyon, the Painted Mountains, White Mesa, Sandstone Valley and Desert Canyon on
sedimentary rock in a dry climate, Stony Spires on carbonate in a humid one. The rock is the column's bedrock: most
land wears a weathering mantle (saprolite and laterite are 68 % of the measured planet's land surface), so where the
surface class is loose cover the first rock of the column's stack within 8 blocks is used. Where the climate search
answers such a biome off its ground, the biome is kept out the way the volcanic biomes are (the next-nearest biome
of the pack's own tree answers). On its ground it answers where the climate puts it: the gate takes biomes away and
never adds them, so Yosemite (whose slot needs a cold, dry valley floor) stays rare. Terralith's Basalt Cliffs are
volcanic too: they go at real volcanoes and on basalt coasts. The gate is a biome rule file like Expanded
Ecosphere's (section above), `deeptime:rock_named`: per ground a rule whose candidates are the tag
`#deeptime:geology/<ground>`, kept out of the climate search (`exclusive`) and given back its own slot (`in_slot`)
where the column's ground has the role (`ground`), so another pack's rock-named biomes can join by tag and a
datapack can replace the file (FORMAT.md, "Biome rules" and "The geology gate").

Measured on one `earthlike_quick_16k` planet (scattered continents, 500 Myr, seed 1; test machine, 2026-10-04),
baked twice from the same simulation, without and with Terralith's table (all other rasters byte-identical; without
the table all 43 rasters are byte-identical to master's), `/deeptime probe biomes 16` with MIC:

| Class (sampled land columns) | Before (master): top biomes, % | After: top biomes, % |
|---|---|---|
| Am (1,507) | shrubland 29, jungle 27, brushland 21, sparse jungle 6, hot shrubland 3, tropical jungle 2 | sparse jungle 44, tropical jungle 21, jungle 14, rocky jungle 4, brushland 4, bamboo jungle 3 |
| Csa (1,469) | plains 46, highlands 25, blooming plateau 14, forest 6 | shrubland 52, steppe 26, fractured savanna 8, windswept savanna 7 |
| Csb (2,921) | plains 61, birch taiga 10, old-growth birch forest 8, forest 8 | shrubland 80, steppe 3, snowy slopes 3, forest 2 |
| BSk (14,077) | plains 57, highlands 14, blooming plateau 7, shield 5 | steppe 44, shrubland 42, plains 2, beach 2 |
| ET (9,952) | cold shrubland 24, snowy slopes 17, snowy plains 15, gravel desert 14, frozen peaks 4, snowy badlands 4, scarlet mountains 4 | gravel desert 25, cold shrubland 23, snowy slopes 16, frozen peaks 8, snowy plains 5, snowy badlands 4 |
| Cfb (7,501), Cwb (970) | cloud forest 0 | cloud forest 9.7 and 9.8; of the 1,351 columns of the rule's ground (class, latitude, height; P/PET not checked), 49 % |

Terralith's names; "plains", "forest", "jungle" and the like are vanilla's. The census's checks for the table (BIO-154):
Am on jungles 0.858, Cs on shrubland and steppe 0.794, BSk on steppe and dry shrublands 0.860 and on the steppe
itself 0.436, ET on snowy plains 0.048. The geology gate on the same world: without it, 2,888 sampled land columns
carried a rock-named biome and 1,654 of them stood off their ground (Shield on orogens and platforms, Granite and
White Cliffs on andesite and mudstone coasts, Stony Spires off carbonate); with it none does, and 1,321
remain (White Mesa 941, Shield 271, all on shield provinces, Painted Mountains 108). The ground on that planet: 8,891
shield columns, 7 granite, 57 carbonate coast, 6 granite coast, 853 basalt coast, 27,885 dry sedimentary, 280 humid
karst. Vanilla on the same world: no pack table chosen, the gate inactive, 0 columns changed.

## Simplifications and limits

- Peel classifies station records of many years. The mod classifies the monthly means of one
  climate snapshot, lapsed to each cell's height.
- Peel's table leaves a warmest month of exactly 10.0 °C in no class; the code puts it in C or D.
- The mod uses the warmer half-year as "summer" (Peel's definition), so a planet with a different
  tilt gets the right halves.
- As (tropical savanna with a dry summer) is not in Peel's table; the code takes it from the common
  extension (Wikipedia).
- The aridity index in the biome mapping is rain over a Thornthwaite-type evaporation. UNEP's index
  uses a Penman-Monteith evaporation, so the UNEP limits are borrowed, not equivalent. That is a
  property of the climate physics, which is being rewritten.
- The Köppen class of a block in the climate interface is the class of the ground's cell, not
  recomputed at the block's height, while its temperatures are lapsed.
- Vanilla has no cold desert. A BWk below a mean of 10 °C is placed on the frozen or cold band by its
  mean temperature (see "Cold deserts" below), so vanilla draws snowy plains or plains there; the
  10 °C and 0 °C switches are game choices (BIO-101), not a classification of any source.
- The volcanic biomes follow the baked list of active volcanoes, their craters and calderas, the hot-spring
  pools and the magma. Dead volcanoes, flood-basalt plateaus and young lava outside a shield are not exported
  by the bake, so they get no volcanic biome; nor do fumarole fields (only hot-spring pools mark a field). The
  zones are discs, not ash plumes downwind; their sizes are game choices (BIO-104 to BIO-111), not measured
  areas of real volcanic landscapes.
- A volcanic role's biome ignores TerraBlender's regions: a volcano in a province whose region is vanilla's
  still gets Biomes O' Plenty's volcanic biome when the pack has it.
- A pack placement table is chosen when the world is baked. A singleplayer world takes the tables in Deep Time's
  jar (a datapack's own pack table reaches only a dedicated server); the workbench, and its 3D previews with MIC,
  use the default table, so they draw MIC's biomes as a world baked without Terralith's table.
- Terralith's table is tuned to Terralith 2.6.2 and to MIC's climate mapping (Tectonic's router stretches erosion);
  a Terralith update that moves its slots needs the table measured again (`tools/review/terralith_emulate.py`).
- The geology gate only takes biomes away: a rock-named biome whose slot the climate never reaches on its ground
  (Yosemite) stays rare or absent. The ground tests read the baked rock and provinces, not the blocks a pack's
  surface rule paints. Off land (below sea level) and below the surface layer the gate's biomes are always kept
  out, as every exclusive rule candidate is.
- The biome rules' climate thresholds (5 °C for low-arctic tundra, 1,000 mm for tropical dry forest, 1,400 mm and
  4 to 12 °C for temperate rainforest, 800 mm for Mediterranean scrub) are cut from classifications read from
  memory, not checked in their sources; a single class-wide threshold replaces what is in reality a gradient
  with fire, soil and grazing in it (Holdridge's own text notes that most savannas lie in his tropical dry
  forest zone). The rock condition reads the first horizon within 6 blocks of the surface, so a beach whose
  sand horizon covers basalt is not "on basalt".
- Expanded Ecosphere is tested in its default mode only. Its COMPATIBLE mode (with TerraBlender) registers a
  TerraBlender region instead of the dimension override; Deep Time would deal that region to provinces like
  any other, untested. EE with Terralith is not supported by EE itself (both define the overworld's biomes).
- A world keeps the biome list it was created with in level.dat; removing EE afterwards leaves it naming
  biomes that no longer exist.

## Game choices

Everything in `placement.json`, the preset's `biomes` section, the ice cover and the climate
interface's conventions is a choice, not a measurement: the temperature ramps (BIO-026 to BIO-034),
the glacier cap (BIO-035), the ocean and latitude-only temperature anchors (BIO-036, BIO-037), the
humidity rules (BIO-039 to BIO-048, BIO-053, BIO-054, BIO-100), the knobs (BIO-049 to BIO-062), the
continentalness, erosion and weirdness derivations (BIO-063 to BIO-070), the frozen-ground placement
(BIO-074 to BIO-078: Ben, decisions log 2026-09-29 and 2026-09-30) and the ice cover (BIO-079 to
BIO-084: Ben, 2026-09-30). The GameTest floors (BIO-072) were set 0.1 under the lowest share
measured over ten test planets and catch a broken mapping, not a wrong climate. The volcanic biomes (BIO-104 to BIO-114) are Ben's in kind
(decisions log 2026-10-03, "Tie them to real volcanoes") and the agent's in every number: zone sizes, the
activity needed for fresh ash, the role order and the tags' order. The biome rules (BIO-115 to BIO-132) are Ben's
in kind (decisions log 2026-10-04, "Build out support for both") and the agent's in every rule: which biome
goes with which class or setting, every threshold, which biomes are exclusive, the rock window and the GameTest
tolerances. Terralith's placement table and the geology
gate (BIO-140 to BIO-154) are Ben's in kind (decisions log 2026-10-04, "Deep Time-side fixes (Recommended)") and the
agent's in every number, but the cloud forest's height (BIO-145, read in Scatena et al. 2010).

## Open questions

- UNEP's humid class starts at 0.65, which is not an anchor of the mapping (BIO-100); the
  anchors 1, 2 and 5 are the mapping's own. Whether to add 0.65 is a design choice.
- The Terralith facts under the frozen-ground placement (BIO-074) rest on an emulation that is not in
  the repository and on one pack version.
- The vanilla reference distributions (BIO-025) were measured by the project's tool and not
  re-measured.
- The Holdridge biotemperature upper cut-off at 30 °C (HYD-030) is missing from the hydrology code.
  Confirmed in Holdridge's text (see the sources) and left to the rivers and lakes package, which
  owns that row.

## Sources

- Peel, Finlayson and McMahon 2007 (`Peel2007`): read in full text. Table 1, the notes under it,
  the Cs/Cw rule, the worked example and Table 2, and the world shares were read.
- Zomer et al. 2022 (`Zomer2022`): read in full text. The aridity table, as a general reference to
  UNEP.
- O'Donnell and Ignizio 2012 (`ODonnell2012`): read in full text. The definition of Bio 15.
- Minecraft 1.21.1 `OverworldBiomeBuilder` (`MinecraftSrc1211`): read in the NeoForge 21.1.249
  sources.
- Scatena, Bruijnzeel, Bubb and Das 2010 (`Scatena2010`), chapter 1 of *Tropical Montane Cloud Forests*: the
  publisher's excerpt read (the 18 °C threshold, 1200–1500 m on large equatorial mountains, the regional table).
- Terralith 2.6.2's parameter list (the jar in MIC's server set): read on the test machine, 2026-10-04.
- Ohmura, Kasser and Funk 1992 (`Ohmura1992`): the equation read; its use belongs to the glacier
  page.
- Holdridge 1967 (`Holdridge1967`): the sentences on the 30 °C cut-off and the factor 58.93 read.
- Wikipedia, "Köppen climate classification", "Seawater", "Ecliptic coordinate system": secondary
  sources, read.
- Not read: Beck et al. 2018 (observed Köppen map), the UNEP atlas (Middleton and Thomas 1997, a
  book), Thornthwaite 1948 (paywalled; belongs to the climate package), Fick and Hijmans 2017.
