# Köppen classes and biomes

Status of this page: `stage 2 (checked against sources on 2026-10-02, work package WP-11)`. The
Köppen rules were read in full in Peel, Finlayson and McMahon 2007 and two departures from it were
fixed. The band edges of Minecraft's biome builder were read in its source. The mapping from class to
biome is a game choice throughout and is labelled so. The climate that feeds the classifier (monthly
temperature and rain) is being rewritten and is not checked here.

Ledger ids on this page: `BIO-001` to `BIO-137` (see [ledger/BIO.md](ledger/BIO.md)).

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
| BIO-026 to BIO-070, BIO-075 to BIO-081, BIO-100 to BIO-103, BIO-133 to BIO-136 | the placement table, knobs, cold wetland and ice cover | see ledger | band coordinate, blocks | none: game choices | `game-choice` |
| BIO-137 | northern peatland share of the world's peatland | 3.21 of 4.23 million km2, about 76 % | million km2 | Xu et al. 2018 (PEATMAP), through a 2025 study that quotes the 3.21 | `cited-unverified` |

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

## Cold wetland

Vanilla puts swamp in its cool and temperate rows at the erosion band E6, whatever the temperature, and
the wetland rule (BIO-066) sends wet, flat, low, thawed land there. On one test planet that made 66 % of
the Dfc land swamp: a boreal plain of wet ground drawn as a Louisiana bayou. Most of the Earth's peatland
is boreal: PEATMAP (Xu et al. 2018) has the northern peatlands at 3.21 million km2 of 4.23 million km2
worldwide, about 76 %, formed "under high precipitation-low temperature climatic regimes" (BIO-137; the
paper itself could not be fetched, see the ledger row for what was read). Ben's answer (decisions log
2026-10-03): "Taiga instead, peatland if present".

What the mapping does (BIO-133 to BIO-136, all game choices):

- Wetland of the six subarctic classes (Dfc, Dfd, Dsc, Dsd, Dwc, Dwd: `classes.<code>.cold_wetland` in
  `placement.json`, the classes of Ben's answer and the ones the table lifts to vanilla's taiga humidity) is
  **cold wetland**. Dfa, Dfb and the C classes keep the swamp even though the table puts all of D on the same
  boreal temperature ramp. It is not put in E6. The
  cell keeps its class's temperature and humidity and its erosion is held under E5, so without a mod vanilla
  draws what it draws for the class: taiga (or its old growth and snowy variants). The bake flags it in
  `climate/wetland` (2; 1 for temperate and warm wetland, which stays swamp, mangrove where hot).
- Where the pack has a peatland biome, the flagged ground goes there. A peatland biome is a biome in the
  shared tag `#deeptime:peatland` (every entry optional, shared with the biome rules, BIO-129: Expanded Ecosphere's
  fen, berry bog, snowy bog and snowy fen, BOP's bog and muskeg, Regions Unexplored's fen, CliffTree's bog; the
  rules place a member directly, and this probe is what makes a pack's own biome source draw one where no
  rule does), or, for a pack nobody listed, one the NeoForge tags call a swamp (`c:is_swamp`) and cold
  (`c:is_cold/overworld` or `c:is_snowy`): BOP's bog and muskeg are both, vanilla's and Terralith's swamps and
  BOP's wetland are neither. A pack's support adds its biome to the tag; no biome name is in the code, and a
  tagged biome only counts where the pack's biome source puts it at the swamp row (below).
  At a world's first flagged cell the game asks the pack's biome source, in each TerraBlender region of the
  province table, what it puts at the swamp row of the cold band (450 climates: five temperatures of the band,
  five wet humidities, three inland continentalness values, six weirdness values). A region that answers
  with a peatland biome at half of them or more has its flagged ground raised into E6 (mid-band), so the pack's
  biome source draws its own bog. The verdict is stored in `climate_mapping.json`, so installing or removing a
  biome mod later does not move explored ground.
- `climateConsistent` counts it: the census leaves E6 land out of the Köppen families as before (that is where
  a peatland region's flagged ground is), and counts the flagged columns apart (`cold_wetland`: on a peatland
  biome, on taiga, on another swamp). Cold wetland on a swamp that is not a peatland fails the test; on the
  old rule it was all of it.

MIC's set (Terralith, Terralith ReStoned) has no peatland biome at the cold band's swamp row: its swamps there
are vanilla's `swamp` and `orchid_swamp`, neither cold. It does have `terralith:ice_marsh`, a frozen-band E6
biome, which thawed wetland never reaches. So with MIC cold wetland is taiga. With Biomes O' Plenty it is
`bog` in the provinces of BOP's secondary region and taiga in its primary region, where the swamp row holds
`wetland`, which is not cold (read in the class files of `BOPOverworldBiomeBuilder` and
`BOPSecondaryOverworldBiomeBuilder`, 2026-10-03; BOP's `muskeg` sits in the frozen band of its middle biomes).

Measured on one `earthlike_quick_16k` planet (scattered continents, 500 Myr, the preset's seed, test machine,
2026-10-04, after master's plate, 35 m and Expanded Ecosphere merges): the same simulation baked with the old rule
(swamp wherever wet, flat, low land is thawed: the placement table without `cold_wetland`) and the new one,
`/deeptime probe biomes 16` through each profile's biome source (`tools/review/peat-runs.sh`, `peat-biomes.sh`).
The six subarctic classes are 6,979 of the sampled land columns (Dfc 6,335, Dfb 5,157); the bake flags 97
columns of the probe grid as cold wetland (0.1 % of the 94,659 land columns; the wet, flat, low land of this
planet's subarctic belt is small, and 1,135 columns of wetland become 1,037 in E6, the rest of it flagged).
Columns of the six classes on a wetland biome (swamp, marsh, bog, fen, wetland), old rule to new rule:

| Profile | Old rule | New rule |
|---|---|---|
| standalone | swamp 30 | none |
| MIC (Terralith) | swamp 65 (ice marsh 561, both rules: a frozen-band biome the wetland rule never reaches) | none (ice marsh 561) |
| Biomes O' Plenty | BOP wetland 25, swamp 5 | none |
| Expanded Ecosphere | fen 40 | fen 68 |

The class shares move by the same amounts: Dfc taiga standalone 50.4 to 50.7 %, MIC 36.2 to 36.6 %, EE 61.6 to
61.3 %; Dfc swamp standalone 0.5 to 0 %. Dfb and the C classes are unchanged (Dfb swamp: 448 columns standalone and
BOP, 316 MIC, EE's birch swamp 138, old growth taiga swamp 163, fen 49). The 97 flagged columns end on (new rule):
standalone, taiga 36, old growth spruce taiga 25, beach 36; MIC, old growth spruce taiga 45, taiga 42, beach 10;
BOP, field 46, gravel beach 20, beach 16, old growth spruce taiga 13, taiga 2; EE, fen 95, cold ocean 2. None is on a
swamp that is not a peatland. A third of them are shore (beach, gravel beach: wet coastal plain is where the wetland
rule puts them, and the continentalness band beside the sea is beach in vanilla's table, before and after). In
the BOP world the verdict is `biomesoplenty:overworld_secondary` only (bog; 6 of the 25 provinces): cold wetland is
bog in those provinces and taiga, or whatever BOP draws on the ground below E5 (field), in the others, because BOP's
primary region puts its `wetland`, which is not cold, at the swamp row; none of this planet's 97 columns lay in a
secondary province (an earlier planet had 9 bog columns). Standalone and MIC have no peatland region
(`regions: []`). Expanded Ecosphere's verdict is also empty (its biome source puts no tagged biome at the cold
swamp row through the probe), and its fen comes from the biome rule (`deeptime:peatland`, BIO-129), which reads the
same flag: 95 of 97 columns.

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
tolerances.

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
- Ohmura, Kasser and Funk 1992 (`Ohmura1992`): the equation read; its use belongs to the glacier
  page.
- Holdridge 1967 (`Holdridge1967`): the sentences on the 30 °C cut-off and the factor 58.93 read.
- Wikipedia, "Köppen climate classification", "Seawater", "Ecliptic coordinate system": secondary
  sources, read.
- Not read: Beck et al. 2018 (observed Köppen map), the UNEP atlas (Middleton and Thomas 1997, a
  book), Thornthwaite 1948 (paywalled; belongs to the climate package), Fick and Hijmans 2017.
