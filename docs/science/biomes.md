# Köppen classes and biomes

Status of this page: `stage 2 (checked against sources on 2026-10-02, work package WP-11)`. The
Köppen rules were read in full in Peel, Finlayson and McMahon 2007 and two departures from it were
fixed. The band edges of Minecraft's biome builder were read in its source. The mapping from class to
biome is a game choice throughout and is labelled so. The climate that feeds the classifier (monthly
temperature and rain) is being rewritten and is not checked here.

Ledger ids on this page: `BIO-001` to `BIO-100` (see [ledger/BIO.md](ledger/BIO.md)).

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
and so on. Land under ice gets a snow and ice cover whatever the biome, frozen lakes follow the
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
| BIO-026 to BIO-070, BIO-075 to BIO-081, BIO-100 | the placement table, knobs and ice cover | see ledger | band coordinate, blocks | none: game choices | `game-choice` |

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
- Cold deserts (BWk) are placed on the same ramp as hot ones: a cold desert with a mean annual
  temperature below 5 °C sits at the bottom of the hot band (4.05), so vanilla draws it as ordinary
  desert (see the open questions).

## Game choices

Everything in `placement.json`, the preset's `biomes` section, the ice cover and the climate
interface's conventions is a choice, not a measurement: the temperature ramps (BIO-026 to BIO-034),
the glacier cap (BIO-035), the ocean and latitude-only temperature anchors (BIO-036, BIO-037), the
humidity rules (BIO-039 to BIO-048, BIO-053, BIO-054, BIO-100), the knobs (BIO-049 to BIO-062), the
continentalness, erosion and weirdness derivations (BIO-063 to BIO-070), the frozen-ground placement
(BIO-074 to BIO-078: Ben, decisions log 2026-09-29 and 2026-09-30) and the ice cover (BIO-079 to
BIO-084: Ben, 2026-09-30). The GameTest floors (BIO-072) were set 0.1 under the lowest share
measured over ten test planets and catch a broken mapping, not a wrong climate.

## Open questions

- Cold deserts drawn as hot deserts (owner question, STATUS.md "Pending questions"). After the test
  order fix about 1.1 % of land (1,425 cells of 132,955 on the measured planet) is BWk that used to be
  ET or EF, and the desert ramp (BIO-032) puts every BWk with a mean below 5 °C at band 4.05.
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
