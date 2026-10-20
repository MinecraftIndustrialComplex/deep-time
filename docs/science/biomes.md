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
subarctic Dfc cell counted as wet because its class sits in vanilla's taiga humidity). Land under the climate's ice gets a snow and ice cover whatever the biome and is of class EF (BIO-158), the
cover lying only as far as each ice body's balance can feed it (BIO-157); frozen lakes follow the
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
   up to 12 blocks; lakes with a mean annual temperature below 0 °C freeze. The game lays the cover
   last, over whatever the pack's surface was, and leaves only water and a frozen lake's own ice
   alone (BIO-084). Until 2026-10-18 it took any ice at the top for a lake's, so the ice a pack
   paints on its own glacier biomes (Expanded Ecosphere's ice cap, vanilla's frozen peaks) went
   without the cover; it now asks the lake ice rule whether that block is a frozen lake's top.

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
| BIO-026 to BIO-070, BIO-075 to BIO-082, BIO-084, BIO-100 to BIO-103, BIO-133 to BIO-136, BIO-158 to BIO-160 | the placement table, knobs, cold wetland, the ice cover, the class under it and its read rule, the census's tag gaps | see ledger | band coordinate, blocks | none: game choices | `game-choice` |
| BIO-157 | an ice body's surface balance on its own ground is at least its climate's, min(0, sum T) | the climate's balance law (GLA-069, GLA-070) | m ice/yr | Meier and Post 1962, Porter 1975, Osmaston 2005, Cuffey and Paterson 2010 (none read) | `from-memory` |
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

### The floors against the census's noise (2026-10-19)

The MIC gate on master 025676c8 failed `climateConsistent` with "Köppen families below their floor:
C 0.621 < 0.630 (132 columns)". Neither the mapping nor the test planet's climate had moved:

- On `planet_world_4k` (seeds 11 to 14, on 26ce664e, before master's arc change to that planet,
  d54c7f0f) the present-day nudge does nothing. Its factor is 1.00 on
  all four, and 0.000 to 0.010 of the land is under ice sheets. The same holds with and without the
  nudge's new cap (climate.md, "The present-day nudge's cap"). The climate fix therefore cannot
  change this share, and it was not a climate fault.
- The census samples columns, so a family's share is a binomial proportion. At 0.63 on 132 columns
  its standard error is about 0.042, so a family whose true share is one standard error above its
  floor falls under it in about one census in six. The bare share most likely failed on where
  the census's points fell.

`climateConsistent` now fails a family only when the share's one-sided 99 % upper bound is under
the floor (BIO-072; `ShareBound.wilsonUpper`, z = 2.326). The bound is Wilson's score interval,
which stays inside 0 and 1 and is sound at small counts and at shares near 0 or 1, where the
normal approximation is not.

| Case | Share | Columns | 99 % upper bound | Floor | Result |
|---|---|---|---|---|---|
| The MIC failure | 0.621 | 132 | 0.713 | 0.63 | passes |
| A broken mapping, small family | 0.30 | 40 | 0.484 | 0.63 | fails |
| A broken mapping, C's size | 0.30 | 132 | 0.402 | 0.63 | fails |

- *Not vacuous.* A mapping that sends a family to the wrong tags leaves it only the columns that
  match by chance, far under floors set 0.1 under the lowest of ten planets. A share of 0.3 fails
  at every count from the census's 40-column minimum up (`ShareBoundTest` holds both cases).
- *Not one planet.* The floors and the bound depend only on the share and the count, so a planet
  with fewer columns in a family gets a wider allowance and a large family a tight one.
- *99 % rather than 95 %:* a gate batch makes about 65 such comparisons (13 families, five test
  sets). If every share sat at its floor, a 95 % bound would fail about three of them by chance
  in every batch on a correct mapping, and a 99 % bound fewer than one.
- *What it does not cover:* a mapping that is a little wrong, say 0.05 under its floor on a small
  family, now passes; on C's 132 columns the allowance is about 0.09, and a share of 0.50 (bound
  0.599) still fails. The floors were never meant to catch a small error. They catch a broken
  mapping, not a wrong climate (Game choices, below).

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
and the bedrock floor, are left out; the strata are the rock there). So is a trim at the sea (BIO-156): EE turns
all ground above y 63 into air in its beaches, rivers, salt lakes and several oceans, which in vanilla's terrain lie
at the sea anyway. In a Deep Time world a beach or a river may stand above the sea where the world data puts it,
and the trim cut it down to y 62, a step at the biome's edge that far terrain does not draw: on the 2026-10-17 test
planet a black beach east of the seam, its ground at y 65 to 67, was cut to 62. A pack's top-level rule that turns
everything above a fixed height into air and paints nothing else is left out; EE's band at y 62, which paints the
top block (water, grass, packed mud, gravel, salt) and removes no ground, stays, as does its band from y 58 to 62
(dead brain coral on warm ocean floors, sand on lukewarm ones); far terrain draws both, since it runs the leading
part of the level's surface rule that reads no chunk (BAK-048, world-scale-and-bake.md). One of EE's biomes is kept out for the same
reason (BIO-155): its calcite caverns, a cave biome its list places by depth (0.35 to 0.9) under much of the world,
the deep sea included, carry a surface rule that paints diorite on every stone block of the biome at every depth.
The strata pass leaves a surface rule's blocks, so in a Deep Time world a cavern region was diorite from the cave
level down, whatever the strata held: on the 2026-10-17 test planet 22 of the parity test's 146 rock samples, all
under the deep sea, were diorite where the strata are gneiss or basalt. Kept out, the next biome of EE's list at
that depth answers and the strata stay the rock. The same rule paints EE's thermal taiga biomes, which stay: they
stand only in real volcano zones (Ben, decisions log 2026-10-05, "Leave it, check in game").

**Biome rules.** EE's own list puts biomes where Deep Time's climate coordinates never reach: its ice cap below
vanilla's lowest temperature (−1.2 to −1.0), its salt lakes, mud pools and Tibesti mountains above the warm end
(0.9 to 1.2), its volcanoes at a continentalness under −1.05 (deep-sea islands). And it puts others where Deep
Time's classes are not: chaparral in the driest hot cells, so 40 % of the planet's hot desert came out chaparral;
temperate rainforest in the wettest cold cells, so 28 % of the humid boreal forest (Dfc) came out temperate
rainforest; snowy taiga and ice spikes in the tundra. A rule file (`data/<ns>/deeptime/biome_rules/*.json`,
FORMAT.md "Biome rules"; Java `climate/BiomeRuleSet`, `climate/BiomeRules`) places a pack's biomes from Deep Time's
own coordinates: the Köppen class, the climate at the column in °C and mm, the wetland flag, the ice cover, the
playas, the volcanic zones and the rock at the surface. The class is the one the bake placed the biome inputs by
(`climate/koppen_l2`: each L2 cell's months lapsed to its own height) where the bake wrote it. Until 2026-10-17 the
rules read the climate stage's L1 class, which gives the low ground of an L1 cell the class of the high ground it
shares the cell with: on that day's test planet 757 of the 3,027 columns a rule decided lay outside the rule's
classes on the L2 class (tundra on a Csb coast below a plateau, a snow-capped volcano in BSk). Read on the L2 class,
the rules decided 2,743 columns of the same planet and none lay outside its rule's classes; the icy volcano fell from
218 columns to 37 (the other cones take the role tag's plain volcano, 269 → 450), low-arctic tundra from 161 to none,
and Mediterranean scrub rose from 299 to 423. The first rule that holds runs the pack's own climate
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

The census's cloud forest ground (BIO-154) is the rule's own: the four classes, the latitude, the height and P/PET
above 1, read from the level's monthly climate with the bake's Thornthwaite arithmetic (MOI-065, MOI-066). Columns in
a volcano zone are left out, as from every pack check, because a volcano zone's biome comes before the table. Until
2026-10-17 the census did not check P/PET. That held while it read the L1 class, and stopped holding when it moved
to the L2 class with plate task 7's merge: on the L2 class every column of the tropics that is high enough is a C
class, dry ones too. On that day's 4k test planet with MIC the old ground held 261 sampled columns, 69 of them on the
cloud forest (0.264, under the bound of 0.3). 72 of the 261 are dry (P/PET at most 1, where the rule never fires) and
9 lie in volcano zones; the rule's ground is the other 180, 67 of them on the cloud forest (0.372). The rest of that
ground is on Terralith's highlands (28), temperate highlands (18), lavender valley (11), alpine highlands (10), lush
valley (10) and plains (7).

Why those columns missed the slot showed on the next planet. The P1-26 rain merge (master 68d95f44, the same
day) changed the test planet: its census holds 11,574 land columns instead of 7,228 and 1,252 volcano cone columns
instead of 450. The census's ground fell to 34 columns, 8 on the cloud forest (0.235), with 90 left out as volcanic and
84 as dry. The bake decides the rule per L2 cell (8 blocks on the 4k planet) on the young landscape's height and that
height's P/PET. The level does not carry that height, so the census's block heights only estimate it, and amplified
relief puts single blocks above 1,000 m in cells below it and the reverse. The bake marks every cell where the rule
fired: it clamps the cell's erosion and continentalness to the rule's floors (0.62 and 0.35, continentalness inland
of the beach strip). Of the 34 columns, only 5 lay in such a cell, and 4 of those 5 were on the cloud forest. Of the
other 29, 4 were. So since 2026-10-17 the census's ground also asks for the bake's mark, with the floors read from the
table the bake recorded (`notes.bake.climate.placement`). On P1-26's planet that leaves 5 columns. The check counts
only from 30 columns (BIO-154), so on this planet it is reported and not counted: here the rule's ground lies mostly
under volcano zones, whose biomes come first.

The same planet showed a second gap in the pack checks. They counted the coast strip: columns on a beach, a stony
shore or a sea biome, whose biome comes from the coast band whatever the class. The family checks have set these
apart since 2026-10-05, and the pack check's own description already said the coast strip is not the table's to
place. Of Cs's 373 counted columns, 68 were coast and none matched, so Cs on shrubland and steppe read 0.499, under
its 0.5. Without the coast it reads 186 of 305 (0.610). The other checks change the same way: Am 346 of 385 (0.899),
BSk 531 of 619 (0.858), ET on snowy plains 10 of 233 (0.043, at most 0.1). Since 2026-10-17 every pack check leaves
the coast strip out.

The class itself was misread on the projected world until 2026-10-18. The bake's projection pass rewrites every
layer on the Petroff–Guyou grid and takes the nearest cell for categories, but `climate/koppen_l2` came a day after
the pass's list of categories and was never put on it, so it was averaged over each cell's footprint as a number
(SCL-090). Where classes meet, the mean of their code indices names a class neither side has: cold steppe (BSk, 7)
beside humid subtropical (Cfa, 14) reads Mediterranean (8 to 10). The census, the families and the biome rules'
Köppen conditions all read this layer. On the CLM-203 planet (master 8b2cfb83, warmer and with more high ground)
Cs on shrubland and steppe read 142 of 383 (0.371): 118 of those columns were Csc, a class the bake had barely
placed, and the shrubland misses had the humidity of the class beside them, not the Mediterranean squeeze. Read
nearest, the same planet has 137 Cs columns, none Csc, and 76 on the table's biomes (0.555); Am 280 of 381 (0.735),
BSk 341 of 523 (0.652), ET on snowy plains 0 of 112. A third of the 137 lie under the ice cover (45, 2 on shrubland),
where the bake holds the ground in the frozen band whatever its class (BIO-035) and the ice pass lays snow and ice
on it; the pack checks leave that ground out since the same day, as the forbidden families already split it off,
and Cs reads 74 of 92 (0.804), Am 277 of 349 (0.794; 32 under the cover), BSk 325 of 475 (0.684; 48), ET on snowy
plains 0 of 40 (72). In 26 of those 45 the bake's glacier flag froze the band: an L2 cell lapsed to its
own ground at 490 to 1,630 m, warm enough for Csa or Csb, under an L1 cell whose ice keeps it cold (its mean annual
temperature −18 to +3 °C, median −11 °C); the other 19 lie at the ice cover's fringe, where the cover's bilinear
thickness reaches past the flag's nearest cell (L1 mean annual −9 to +14 °C), a cover on ground the climate
does not freeze that is the bake's to revisit.

Revisited the same day (BIO-157 to BIO-159). The cover now lies on the climate's own extent (the L2 land whose
nearest L1 cell holds at least 1 m of ice), each ice body closed on its surface balance on its L2 ground, and a
covered cell is of class EF; every reader takes the cover's extent from the nearest cell, as the class is read, so
the fringe past the flag's cell is gone. On the world grid, before and after (master 991354a4; the 4k test planet
with the Terralith patch; earthlike_quick_16k, scattered continents, 500 Myr): the cover went from 190 to 131 world
cells (3.4 to 2.3 % of the land) and from 42,047 to 37,808 (15.3 to 13.7 %), its true area by 29 % and 8 %. The
covered columns not of class EF went from 189 of 190 to none and from 21,962 to none. The share of the cover over
ground whose L1 cell is warmer than 0 °C (mean annual) went from 0.221 to 0.092 and from 0.060 to 0.040; on
earthlike_quick_16k what is left there is 1,298 world cells at 0 to 5 °C, 191 at 5 to 10 °C and 13 at 10 °C or
more (before: 2,045, 402 and 84), the parts of bodies their higher cells feed (BIO-157), the way an outlet
glacier's tongue reaches below the snowline on its accumulation area. Over the ground's own temperature (the L2 bed
lapsed to its own height, without the ice) 60 % and 16 % of the cover lie above 0 °C: under a sheet the bed sits
far below the ice surface the L1 climate is of, so that measure is no target.

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

Measured on one `earthlike_quick_16k` planet (scattered continents, 500 Myr, seed 1; test machine, 2026-10-05, on
master 2e07d954 with this work merged: the plate transition, the 140 km foreland and cold wetland included), baked
twice from the same simulation, without and with Terralith's table, `/deeptime probe biomes 16` with MIC (43,834
sampled land columns). The world without the table is the "before": without a pack table the bake is the default
table's, byte for byte (checked against master's CLI on the earlier planet: all 43 rasters identical; with the
table only the five biome inputs differ).

| Class (sampled land columns) | Before: top biomes, % | After (Terralith's table): top biomes, % |
|---|---|---|
| Am (719) | brushland 32.5, jungle 28.5, beach 8.9, sparse jungle 7.5, tropical jungle 7.1, mangrove swamp 4.7 | tropical jungle 33.1, jungle 27.3, bamboo jungle 10.3, brushland 9.5, beach 7.2, mangrove swamp 4.7 |
| Csa (367) | plains 71.4, forest 7.9, gravel beach 6.3, beach 5.4, highlands 2.7 | shrubland 77.4, gravel beach 6.3, beach 5.4, steppe 4.6, savanna 2.5 |
| Csb (1,764) | plains 54.0, forest 9.1, old-growth birch forest 6.4, birch taiga 4.8, brushland 3.9 | shrubland 68.5, snowy slopes 3.7, brushland 3.6, forest 3.3, frozen peaks 3.3 |
| BSk (4,650) | plains 50.8, snowy slopes 7.6, frozen peaks 5.2, rocky mountains 4.6, forest 3.8, highlands 3.7 | shrubland 55.8, steppe 12.3, plains 7.1, beach 3.3, frozen peaks 2.5 |
| ET (5,588) | cold shrubland 21.7, gravel desert 15.6, snowy slopes 14.0, snowy plains 9.8, taiga 5.9, frozen peaks 3.0 | gravel desert 23.4, cold shrubland 21.6, snowy slopes 13.7, taiga 5.9, frozen peaks 5.7, snowy plains 2.0 |
| Cfb (5,322), Cwb (499) | cloud forest 0 | cloud forest 6.4 and 8.8 |

Terralith's names; "plains", "forest", "jungle" and the like are vanilla's. The census's checks for the table
(BIO-154; mountains and volcanic biomes left out, near-coast land left out of the steppe check), before → after:
Am on jungles 0.475 → 0.773 (at least 0.6); Cs on shrubland and steppe 0.001 → 0.775 (0.5); BSk on steppe and dry
shrublands 0.017 → 0.782 (0.5); BSk mid and far inland on the steppe itself 0.003 → 0.848 (0.1); ET on snowy plains
0.100 → 0.022 (at most 0.1). Cold wetland (the subarctic classes' wet lowland) is the same in both: 40 columns, 30 on
taiga, none on swamp (Terralith has no peatland biome).

The geology gate (the rule file `deeptime:rock_named`), the census's plain search (no gate, no volcanic layer)
against the level's. Without the table 1,564 sampled land columns carried a rock-named biome and 844 of them stood
off their ground: Shield 456 of 456 (this planet has no shield province), Granite Cliffs 173 of 173 (no granite
coast), Painted Mountains 118 of 215, White Cliffs 53 of 53, White Mesa 27 of 650, Snowy Shield 12, Bryce Canyon 4,
Basalt Cliffs 1. With the gate none does, and 720 remain, every one placed by its rule: White Mesa 623, Painted
Mountains 97. With the table: 1,079 and 359 off their ground → the same 720, 0 off. No land column turned to water
(holes 0); the rule census's not-candidate, wrong-class and leaked-exclusive counts are 0. What took the kept-out
biomes' places (without the table): badlands 162, snowy slopes 153, plains 118, eroded badlands 114, forest 113,
Terralith's rocky mountains 49. The ground on that planet: no shield, no granite and no granite coast; 218 carbonate
coast, 367 basalt coast, 13,010 dry sedimentary, 280 humid karst columns.

On the first planet measured (the same preset and seed on master 449a3ae2, before the plate transition; two Shield
provinces, 8,891 shield columns) the shares moved the same way: Am shrubland 29 / jungle 27 / brushland 21 → sparse
jungle 44 / tropical jungle 21 / jungle 14; Csa plains 46 → shrubland 52, steppe 26; BSk plains 57 → steppe 44,
shrubland 42; ET snowy plains 15 → 5 %, gravel desert 14 → 25 %; cloud forest 9.7 % of Cfb. The gate there: 2,888
rock-named, 1,654 off their ground → 1,234, 0 off (White Mesa 941, Shield 189, Painted Mountains 103); with the
table Shield's own slot covers only 3 columns, all off shield ground, so it has none.

Without Terralith the table and the rule file are not used (GameTests on the 4k test planet, 4,322 sampled land
columns, 2026-10-05): vanilla records no pack table and no rule, and no column differs from the plain search;
Biomes O' Plenty under TerraBlender and Expanded Ecosphere record no pack table and sample no rock-named biome (the
211 and 618 columns that differ from the plain search are the volcanic layer's and, with Expanded Ecosphere, its own
rule file's). With MIC the same test planet bakes with Terralith's table; the plain search there puts a rock-named
biome on 141 columns, 120 of them off their ground, and the level 21, none off their ground.

The retry that keeps land from turning into water (a kept-out biome whose next-nearest leaf is ocean or river is
searched again without the water leaves) runs only on land. At sea the water is the right answer: Expanded
Ecosphere's island biomes, kept out of Deep Time's deep-sea slots (BIO-132), would otherwise come back as land biomes
on the sea floor (the test planet's sea fell to 0.685 ocean biomes with Expanded Ecosphere before this was fixed; 1.0
after).

## Ice spikes and the rare variants (2026-10-20)

Ben, on his play world (bake be17a7cf, workbench run 1a1578fa5d2: `earthlike_quick_16k`, supercontinent, 1,000 Myr,
seed 1, with MIC): "I've also noticed like 20x more ice spike biomes than I'd usually see, might be indicative of
biome rarities being ignored."

**How a column's biome is chosen.** No step weighs a biome by rarity. In a normal world a biome is rare because its
slots in the pack's parameter list cover a small part of the noise space; Deep Time replaces the noise with the baked
climate, so a biome's share is how often the planet's baked inputs fall in its slots.

1. The bake, once per world. `bake/biomes.rs` `map` (line 1147) gives every L2 cell the five inputs in vanilla units.
   Temperature and humidity come from the Köppen class through `placement.json` (`Placement::place`,
   `bake/placement.rs` line 1259). EF under the ice cover takes the glaciated block: temperature ramp `polar_ice`
   (band 0.6 to 0.8, vanilla -0.67 to -0.56) and humidity `ice_cap` (band 0.5, vanilla -0.675), the frozen and
   driest cell (T0, H0) where vanilla and Terralith draw their ice spikes (BIO-078). Continentalness is the distance
   to the sea (the block after `// Continentalness.` in `map`: the beach strip, 16 blocks, is band 4, 200 blocks
   band 5, 1,000 band 6, 5,000 band 7), clamped inland of the beach strip by a class's `continentalness` limits.
   Erosion follows the rank of the local relief, weirdness the crest index and the class's weirdness rule (EF's flats
   rule, BIO-077). They are the layers `climate/temperature`, `vegetation`, `continents`, `erosion` and `ridges`.
2. The router, every column. `worldgen/noise_settings/planet.json` (lines 57 to 85) reads those layers through
   `deeptime:raster` density functions (`RasterDensity.compute`, line 36, then `WorldDataTerrain.raster`, line 310:
   bilinear, the ridges as an odd field) and maps each from vanilla units onto the pack's router with the world's
   climate mapping (`ClimateMapping.apply`, line 234, each table's `Table.apply`, line 56): with MIC, onto
   Tectonic's stretched erosion and continentalness.
3. The biome source. `worldgen/world_preset/planet.json` (lines 8 and 9) keeps `minecraft:multi_noise` with the
   preset `minecraft:overworld`, so the parameter list is the pack's: vanilla's `OverworldBiomeBuilder`, or with MIC
   Terralith 2.6.2's own list (MIC ships Terralith with Lithostitched and Tectonic, which reshapes the router, and
   no TerraBlender). The game takes the nearest slot (`Climate.ParameterList`).
4. Deep Time's hook. `VolcanicBiomeSourceMixin` (its `@Inject` at line 34; Deep Time levels only, line 37) keeps
   the volcanic biomes out of that search and places them at the volcanoes and applies the biome rules
   (`BiomeRules.search`, line 158); with TerraBlender, whose regions follow the provinces (P1-10),
   `mixin.compat.terrablender.RegionBiomeSourceMixin` does the same in the region's tree. No rule file names ice
   spikes, so with vanilla and MIC they come from the parameter list alone.
5. The ice cover (`IcePass`) then lays snow and ice over the ice cap's columns, whatever their biome.

**What was measured.** `tools/review/ice-spikes.sh` (run on the test machine through `tools/remote/mac.sh`) dumps a
bake's biome inputs every 16 blocks on the game census's grid (`dump`), boots the profile's server on a copy of the
world data and runs `/deeptime probe biomes 16` and `/deeptime probe reference_biomes` (`census`: the pack's own router
and parameter list over 4 seeds, 400,000 blocks square every 512 blocks, so 1.17 million land samples for MIC and
1.55 million for vanilla), and `tools/review/ice_spikes.py` (`shares`, `emulate`) emulates the game's pipeline on the
dump (the climate mapping's tables, then the nearest slot of the pack's parameter list). The emulation agreed with the
game's own census on 98.5 % (MIC) and 99.3 % (vanilla) of the play world's land columns. `--ef-c-min 0.1` emulates the
fix on a bake made without it; the worlds of 2026-10-20's master (after the continents fix, which roughly doubled
cold land) were baked with `planet` and `testplanet` and `--placement-patch` of `no-ef-floor.json` (the fix taken out)
for "before". "The corner" is the frozen, driest cell (temperature band T0 and humidity band H0) that both packs
draw ice spikes from; "flats" are its columns at erosion -0.375 and up (E2 and up), where both packs draw them in
their own worlds (all of MIC's, 87 % of vanilla's corner).

| world, pack | corner, % of land | ice spikes, % of land (x the pack's own) | of the corner | of the corner's flats | of the ice cap |
|---|---|---|---|---|---|
| the pack's own worlds, MIC | 1.28 | 0.329 | 25.6 % | 34.5 % | (none) |
| the pack's own worlds, vanilla | 0.83 | 0.356 | 42.7 % | 46.8 % | (none) |
| play world (be17a7cf), MIC, before | 10.7 | 2.05 (6.2x) | 19.1 % | 66.9 % | 18.3 % |
| same, after | 10.8 | 0.59 (1.8x) | 5.5 % | 19.3 % | 4.8 % |
| play world, vanilla, before | 10.7 | 2.12 (6.0x) | 19.7 % | 16.8 % | 19.3 % |
| same, after | 10.8 | 0.61 (1.7x) | 5.7 % | 16.8 % | 5.3 % |
| 2026-10-20 master, supercontinent, MIC, before | 23.5 | 6.03 (18.3x) | 25.6 % | 65.9 % | 25.4 % |
| same, after | 23.6 | 1.73 (5.3x) | 7.3 % | 19.0 % | 7.1 % |
| same, vanilla, before | 23.5 | 3.66 (10.3x) | 15.6 % | 6.4 % | 15.3 % |
| same, after | 23.6 | 0.84 (2.4x) | 3.5 % | 6.4 % | 3.4 % |
ICE_SPIKES_MORE_WORLDS

The 2026-10-20 master's supercontinent is the play world's own preset (`earthlike_quick_16k`, supercontinent,
1,000 Myr, seed 1) baked again by that master, with the default placement table as the workbench bakes it.

**Why there are so many.** Two things multiply.

1. *The corner is 8 to 28 times the pack's.* No rarity is lost on the way to it: the planet's frozen land is
   16.9 % of land on the play world and 29.5 % on the 2026-10-20 supercontinent, against 10.8 % (MIC) and 10.9 %
   (vanilla) in the packs' own worlds; and the ice cap, 10.8 % and 23.6 % of land, is placed in the driest humidity
   band on purpose (BIO-078, Ben's "Ice only where there's ice": the ice cap takes no taiga), so 63 % and 80 % of the
   frozen land is in the corner against the packs' 7.6 to 12 %.
2. *Inside the corner, near-inland ground got ice spikes past the flats rule.* BIO-077 sends EF's flats to the
   negative branch of the variant noise but for about a fifth of them, so that ice spikes, a positive-weirdness
   variant in vanilla, come back on a share of the ice cap's flats. Both packs had a way past it, both near-inland,
   and 72 to 78 % of these planets' land is near-inland (continentalness -0.11 to 0.03) against 22 to 27 % in the
   packs' own worlds:
   - Terralith 2.6.2 draws `minecraft:ice_spikes` on every near-inland flat of the corner (continentalness -0.2 or
     -0.11 to 0.03, erosion -0.375 or -0.223 to 0.05) on both signs of weirdness, so the rule's negative branch is
     ice spikes too: 66 to 67 % of the corner's flats against MIC's own 34.5 %. Mid- and far-inland its ice spikes
     sit at erosion 0.05 to 0.33 only, again on both signs.
   - Vanilla's `OverworldBiomeBuilder` gives near-inland ground in its low weirdness slice at erosion below -0.375
     the middle biome, which in the corner is ice spikes on positive weirdness, and EF's own rule gives steep ground
     positive weirdness (`positive_below_erosion`), so 20 to 30 % of the ice cap's steep near-inland ground was ice
     spikes, 74 to 81 % of all of them. Mid-inland that slot is snowy slopes.

**What "right" means here** (the coordinator's call, 2026-10-20). The target is a share of the parent, not an
area: when the climate makes more frozen, dry land, more ice spikes are right. The parent is the corner's flats,
not the whole corner, because ice spikes are a flats biome in both packs' own worlds and the planet's ice cap is
steep (61 to 72 % of it at erosion E0 and E1, against 26 % of MIC's corner and 13 % of vanilla's): measured on the
whole corner, the planet's share follows how steep its ice cap happens to be (MIC before the fix: 19.1 % on the
play world, 25.6 % on the 2026-10-20 supercontinent, whose ice cap is flatter), while on the flats it held at
66 to 67 % before and 19 % after on both. The share is BIO-077's, about a fifth of the flats, which Ben's
"Bring ice spikes back" decision put in place; it is below both packs' own shares (34.5 % for MIC, 46.8 % for
vanilla), so the ice cap's flats are thinner in ice spikes than the pack's frozen, dry flats, never thicker. Taken
on the whole corner instead, MIC's share before the fix matched its own on the 2026-10-20 supercontinent (25.6 %,
an excess per flat offset by the steep ice cap) and the fix leaves 7.3 %: that reading would keep the bypass.

**How much too cold** (not retuned here). Köppen E is 16.8 % of the play world's land and 29.2 % of the 2026-10-20
supercontinent's, against 12.8 % of Earth's (Peel et al. 2007): 1.3 and 2.3 times. The ice cap is 10.8 % and 23.6 %
of their land, against about 10 % of Earth's under Antarctica's and Greenland's ice (from memory, not checked).
The frozen band (T0) holds 16.9 % and 29.5 % of their land against 10.8 to 10.9 % in the packs' own worlds. With
the fix, the remaining ice spikes excess on these worlds (1.7 to 5.3 times the pack's area) is that cold, dry
land's.

**The fix** (BIO-161, the coordinator's call). EF's land inland of the beach strip, ice cap and barren alike, is
held mid-inland: `placement.json` `classes.EF.continentalness` `{"min": 0.1}`, the class clamp `map` already applies
at band 4 and up (FORMAT.md, "for a class or a rule"; no format change). 0.1 rather than the band edge 0.03 keeps
the game's bilinear read next to the unclamped beach strip inside the band. Terralith's near-inland ice spikes slot
and vanilla's near-inland low-slice middle biome are then out of reach. What sets the share after that differs by
pack:

- With vanilla, the flats rule: its positive branch on the flats, plus a few valley columns (absolute weirdness
  under 0.05, which vanilla's builder gives the variant mid- and far-inland at erosion E0 and E1: up to 7 % of
  those columns). The flats' share is what the noise lets through, 16.8 % on the play world and 6.4 % on the
  2026-10-20 supercontinent: BIO-077's threshold is not a fifth on every world (see "Open questions").
- With MIC, Terralith's own mid-inland slot: erosion 0.05 to 0.33 (after the climate mapping) on both signs of
  weirdness. 94 to 95 % of the ice cap's mid-inland E4 ground is ice spikes, and its other flats none, so the flats'
  share is how much of them falls in that window: 19.3 % on the play world and 19.0 % on the 2026-10-20
  supercontinent, near BIO-077's fifth by the erosion and not by the noise. The beach strip, which is not clamped,
  keeps Terralith's coastal ice spikes (half to four fifths of its E3 columns).

Both stay under the pack's own share of the flats. What else moves on the ice cap: with MIC, snowy slopes 40 to 41 %, frozen peaks 14 to 28 %, snowy plains 8 to 18 %, gravel
desert 16 to 5 % (play world); with vanilla, frozen peaks 7 to 28 % (the high slice's steepest mid-inland ground is
frozen peaks, near-inland snowy slopes). With MIC, EF's barren ground (0.1 to 0.2 % of land) trades its ice
spikes (11 % of it) for Terralith's cold shrubland. `bake::biomes::tests::ef_land_is_held_mid_inland_off_terraliths_near_inland_ice_spikes`
checks the clamp against the same table without the key. The GameTest census bounds the ice cap's ice spikes
(`EF:few_ice_spikes`, BIO-162): it fails when the share's one-sided 99 % lower bound is above 0.10, between the
shares after the fix (0.034 to 0.071) and before it (0.153 to 0.254). On the GameTests' 4k planet the ice cap is 3
census columns (0.2 % of its land), under the 40 a family needs, so there the bound is logged and not asserted; it
bites on a planet with an ice cap.

**The rare variants.** The same emulation gives the other variants against the packs' own worlds (the play world,
before the fix; the fix changes only frozen biomes):

| biome | play world, MIC | MIC's own | play world, vanilla | vanilla's own |
|---|---|---|---|---|
| snowy slopes | 6.38 % | 0.91 % | 7.59 % | 0.63 % |
| frozen peaks | 2.19 | 1.27 | 0.87 | 0.27 |
| jagged peaks | 0.50 | 0.94 | 0.12 | 0.28 |
| grove | 0.13 | 0.71 | 0.37 | 1.19 |
| snowy taiga | 0.13 | 1.55 | 0.24 | 3.50 |
| mushroom fields | 0 | 0.175 | 0 | 0.148 |
| cherry grove | 0 | 0.94 | 0 | 0.45 |
| eroded badlands | 1.01 | 0.42 | 0.77 | 0.44 |
| wooded badlands | 0 | 0.55 | 0 | 0.88 |
| sunflower plains | 0.010 | 0.44 | 0.079 | 1.12 |
| flower forest | 0.051 | 0.40 | 0.17 | 1.09 |
| bamboo jungle | 2.65 | 0.555 | 4.61 | 1.07 |
| old growth birch forest | 6.71 | 1.82 | 5.13 | 3.44 |
| windswept gravelly hills | 0.014 | 0.24 | 0.002 | 0.16 |
| meadow | 0.006 | 0.91 | 0.055 | 1.85 |

They follow the same mechanism as the ice spikes: a biome's share is how often the baked inputs fall in its
slots. Snowy slopes and frozen peaks follow the steep ice cap. Mushroom fields need continentalness -1.2 to -1.05,
which no land cell gets. Vanilla's plateau biomes (meadow, and cherry grove as their positive-weirdness variant)
are drawn mid- and far-inland, and these planets' land is mostly near-inland. Which weirdness branch a class takes
is its rule's, not a rarity: sunflower plains (the positive variant of flower forest's cell) are thin, while old
growth birch forest and bamboo jungle, positive variants too, are 2 to 4 times the packs'. None of these is
changed here (see "Open questions").

**Not checked.** The fix's numbers are the emulation's, checked against a bake with the fix on the GameTests' 4k
test planet (ICE_SPIKES_VALIDATION); the game's own census was run on the play world before the fix only. The
emulation reads the bake every 16 blocks, as the census does; Terralith's surface rules and features (the ice
spikes themselves) were not looked at. The packs' own shares are one MIC and one vanilla server set (MIC's pins of
2026-10-20), 4 seeds each.

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
- The ice cover's balance closure (BIO-157) scores an ice body with the climate's law on its L2 ground and gives
  up its most negative cells first. It moves no ice: there is no flow model on L2, so an outlet tongue the climate's
  coarse cell did not reach is not drawn, and the trim takes a body's lowest cells whether or not they are its
  terminus. Covered cells are EF by choice (BIO-158): Köppen classes a climate, and the ground beside a glacier's
  tongue keeps its own class.
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
BIO-084: Ben, 2026-09-30; its balance closure, the class under it and its read rule, BIO-157 to BIO-159, and the
census's tag gaps, BIO-160, the agent's, 2026-10-18). The GameTest floors (BIO-072) were set 0.1 under the lowest share
measured over ten test planets and catch a broken mapping, not a wrong climate; since 2026-10-19 a family is
tested by its share's 99 % upper bound, not the bare share ("The floors against the census's noise", the coordinator's call). The volcanic biomes (BIO-104 to BIO-114) are Ben's in kind
(decisions log 2026-10-03, "Tie them to real volcanoes") and the agent's in every number: zone sizes, the
activity needed for fresh ash, the role order and the tags' order. The biome rules (BIO-115 to BIO-132) are Ben's
in kind (decisions log 2026-10-04, "Build out support for both") and the agent's in every rule: which biome
goes with which class or setting, every threshold, which biomes are exclusive, the rock window and the GameTest
tolerances. Terralith's placement table and the geology
gate (BIO-140 to BIO-154) are Ben's in kind (decisions log 2026-10-04, "Deep Time-side fixes (Recommended)") and the
agent's in every number, but the cloud forest's height (BIO-145, read in Scatena et al. 2010). EF's continentalness
floor (BIO-161) and the census's bound on the ice cap's ice spikes (BIO-162) are the coordinator's calls (2026-10-20,
on Ben's ice spikes report), and so is the target they serve: BIO-077's share of the frozen, driest flats, not the
pack's own share and not an area ("Ice spikes and the rare variants").

## Open questions

- UNEP's humid class starts at 0.65, which is not an anchor of the mapping (BIO-100); the
  anchors 1, 2 and 5 are the mapping's own. Whether to add 0.65 is a design choice.
- The Terralith facts under the frozen-ground placement (BIO-074) rest on an emulation that is not in
  the repository and on one pack version.
- The vanilla reference distributions (BIO-025) were measured by the project's tool and not
  re-measured.
- After the fix the ice cap's flats carry ice spikes below the packs' own shares (MIC 34.5 %, vanilla 46.8 % of
  their frozen, driest flats). With MIC the share is set by Terralith's mid-inland erosion window (19 % on both
  measured worlds); with vanilla by what BIO-077's noise lets through, which gave the positive branch to 20 % of the
  play world's ice cap at erosion E3 and to 4 % of the 2026-10-20 supercontinent's (16.8 % and 6.4 % of the flats).
  Reaching a pack's own share would need a share of the ice cap kept near-inland by noise, or a weirdness threshold
  set per world, which the placement format cannot say (a new key: Ben's to decide).
- The rare variants ("Ice spikes and the rare variants"): mushroom fields never appear (no land cell is below
  continentalness -1.05), plateau biomes are thin on mostly near-inland land, and which variants appear is each
  class's weirdness rule's. Whether the planet should draw them at the pack's rarity is a design choice, not taken.
- 72 to 78 % of the measured planets' land is near-inland without EF's floor (continentalness from the distance to the
  sea), against 22 to 27 % in the packs' own worlds; it shapes every biome that differs between near- and
  mid-inland.
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
- Terralith 2.6.2's parameter list (the jar in MIC's server set): read on the test machine, 2026-10-04; its nine
  `minecraft:ice_spikes` entries read again 2026-10-20 (BIO-161).
- Minecraft 1.21.1 `OverworldBiomeBuilder`'s high, mid and low slices and its biome tables: read 2026-10-20 in the
  NeoForge 21.1.249 sources (BIO-161).
- Ohmura, Kasser and Funk 1992 (`Ohmura1992`): the equation read; its use belongs to the glacier
  page.
- Holdridge 1967 (`Holdridge1967`): the sentences on the 30 °C cut-off and the factor 58.93 read.
- Wikipedia, "Köppen climate classification", "Seawater", "Ecliptic coordinate system": secondary
  sources, read.
- Meier and Post 1962 (`Meier1962`), Porter 1975 (`Porter1975`), Osmaston 2005 (`Osmaston2005`), Cuffey and
  Paterson 2010 (`Cuffey2010`, section 4.6): the steady-state balance of a glacier and the AAR and AABR methods,
  from memory, not read (BIO-157).
- Not read: Beck et al. 2018 (observed Köppen map), the UNEP atlas (Middleton and Thomas 1997, a
  book), Thornthwaite 1948 (paywalled; belongs to the climate package), Fick and Hijmans 2017.
