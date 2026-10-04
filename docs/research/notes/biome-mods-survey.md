# Biome mods for Deep Time: a survey, and the gaps Terralith leaves

Date: 2026-10-03. Research note (a dated snapshot; see the index's reading rules).

Question (Ben, decisions log 2026-10-03): "Leaning towards terralith rn, though go do a bit more
research. Also, check if there are any major gaps that terralith might not have that we would want /
need."

## Short answer

- **Terralith is the right primary target.** Among the NeoForge 1.21.1 biome mods it is the only one
  that combines all of: vanilla blocks only, a large realistic biome set, a published parameter list
  that Deep Time's biome source reads as it is, active maintenance, and a pack MIC already ships. Its
  realistic biomes are named after geology (Shield, Yosemite, White Cliffs, Basalt Cliffs, Yellowstone,
  Caldera, Volcanic Peaks, Bryce Canyon), which suits a world that knows its geology: Deep Time can
  place them where the rock and the provinces match, as it is about to do for the volcanic ones.
- **Its gaps that matter are few, and most are not biomes.** Measured on a real 16k planet, the
  largest problems are placement, not missing biomes: monsoon forest (Am) comes out 56 % brushland,
  cold steppe (BSk) never reaches Terralith's own `steppe`, Mediterranean classes come out as plains,
  and 1.35 % of the land gets Terralith's fantasy biomes. Those are cheap fixes on Deep Time's side
  (a Terralith-tuned placement table and a veto by tag).
- **The real biome gaps** are peatland (bog, fen, muskeg) for cold wet lowland, a true tundra, tropical
  dry forest, tidal flats, salt flats and karst. Salt flats and hot springs are already decided as
  surface features; karst, lava fields and tidal flats fit the same pattern; peatland is the one gap
  that is both large in area and not a surface feature.
- **No other mod is a better primary.** Biomes O' Plenty and the rest add hundreds of custom blocks;
  Expanded Ecosphere is the most realistic and covers almost every gap, but its biomes do not reach a
  Deep Time world as packaged, its licence forbids derivatives, and it replaces the whole overworld;
  Wilder Wild has no NeoForge 1.21.1 build. Regions Unexplored is the best companion to Terralith
  (same library, MIT, a cold fen with peat blocks).

The recommended target set, the fixes and their costs are in sections 7 to 9; the questions for Ben
are in section 10.

## 1. How a biome mod reaches a Deep Time world

This decides what "compatible" means, so it comes first.

Deep Time's world preset (`data/deeptime/worldgen/world_preset/planet.json`) gives the overworld the
generator `deeptime:geologic` with the biome source `{"type": "minecraft:multi_noise", "preset":
"minecraft:overworld"}`. Deep Time computes the six climate coordinates of each column from its own
simulation (`placement.json`, `bake/biomes.rs`) and asks that biome source for the biome. The surface
blocks come from the pack's own overworld surface rule (`DelegatingRuleSource`). So a mod's biomes
appear in a Deep Time world in exactly these ways:

| Mechanism | Mods | Reaches a Deep Time world? |
|---|---|---|
| Replaces the `multi_noise_biome_source_parameter_list/overworld` entry (Lithostitched's `lithostitched:biomes` field) | Terralith, CliffTree | Yes: the preset reads it. Two such mods collide (the last pack wins). |
| Replaces vanilla biomes inside the list by its own noise and ranges (Lithostitched) | Regions Unexplored | Yes: wherever the vanilla parent biome is picked. |
| TerraBlender regions | Biomes O' Plenty, BWG (which also swaps vanilla swamps inside its regions), Nature's Spirit, Nyctophobia | Yes: Deep Time deals one region to each geologic province (P1-10), so a mod's biomes fill whole provinces. |
| Lithostitched biome injectors | Spawn (islands) | Probably (Lithostitched wraps the biome source); not tested. |
| Biolith | Let's Do BloomingNature | Not tested. |
| Overrides `dimension/overworld.json` | Expanded Ecosphere, CliffTree (also) | **No, not as packaged**: the preset defines its own overworld stem. Which wins at world creation was not tested; either the file is ignored (EE's biomes never appear) or it replaces Deep Time's generator. |
| Overrides `world_preset/normal.json` | Woodlands: Extra | No: only the vanilla Normal preset. |
| Rewrites vanilla biome files only (no new ids) | WWOO, Geophilic | Yes, as restyled vanilla biomes. |

Two consequences carry through the rest of this note:

- A mod's biome lands where its parameter cells meet Deep Time's band coordinates. Deep Time places
  each Köppen class on vanilla's five temperature bands (frozen F, cold C, temperate T, warm W, hot H)
  and five humidity bands (0 driest to 4 wettest). From `placement.json`:

  | Class | Cells it lands in |
  |---|---|
  | Af | W:4 |
  | Am | W:3–4 |
  | Aw, As | W:1–2 |
  | BWh, and BWk above 10 °C | H:0 |
  | BWk 0–10 °C | C:0 |
  | BWk at or below 0 °C; EF barren | F:1 |
  | BSh | W:1 |
  | BSk | C:1 (coldest month ≤ 0 °C) or T:1 |
  | Cs | T:1–2 |
  | Cw, Cf | T:2–4 |
  | Da/Db (s, w, f) | C:2–4 |
  | Dc/Dd | C:3–4 |
  | ET | F:0–1 |
  | EF ice cap | F:0 |

  Inside a cell, continentalness, erosion and weirdness pick among the cell's biomes. Deep Time
  already clamps erosion and weirdness per class (EF uses both), which is the cheapest lever it has.
- Fixes must live on Deep Time's side (its placement table, its own selection, tags), not in edits of
  the mods' files: Terralith's licence forbids publicly distributed modpacks from overriding parts of
  it (section 2.5).

## 2. Terralith 2.6.2 for NeoForge 1.21.1

Read from `Terralith_1.21.1_v2.6.2_Neoforge.jar` in `smoke/mic/server/mods` on the test VM, with
Python `zipfile` (script and output: `deeptime-runs/biome-survey/terralith.{py,out}` on the VM).

### 2.1 What the jar holds

- Mod id `terralith` 2.6.2, authors Starmute, catter1, Apollo, TheKingWhale; `license = "MIT"` in the
  mods.toml, but the jar's `license.txt` is the Stardust Labs License (2.5), which is the licence the
  Modrinth page names. One required dependency: Lithostitched `[1.7.7,)`. 11 Java classes (config and
  the `terralith:config` resource condition); its mixin list is empty.
- The parameter list (`data/minecraft/worldgen/multi_noise_biome_source_parameter_list/overworld.json`)
  has 1,713 entries in Lithostitched's `lithostitched:biomes` field, naming 148 biomes: 53 vanilla and
  95 of Terralith's own (84 surface, 11 cave). It also rewrites 35 vanilla biome files.
- **Blocks: vanilla only.** Every block state named anywhere in its worldgen JSON is in the
  `minecraft:` namespace (179 distinct names); no block of its own, no assets but language files. Its
  surface rule uses 48 vanilla blocks (stone, coarse dirt, gravel, sand, calcite, red sand, clay,
  tuff, blackstone, dripstone, granite, smooth basalt, powder snow, packed ice, mud, terracotta
  colours and so on) and names about 110 biomes.
- Config toggles by resource overlays: custom structures, intro message, Skylands, terrain slabs,
  recipe changes, vanilla stone generation.
- Biome tags it ships include `c:is_magical` (14 biomes: amethyst canyon and rainforest, lavender
  forest and valley, mirage isles, moonlight grove and valley, painted mountains, scarlet mountains,
  the four Skylands, warped mesa) and `c:is_rare` (alpha islands, alpha islands winter, mirage isles,
  the Skylands, warped mesa). These are a ready-made veto list (section 8).

### 2.2 The full biome list with its climate parameters

Bands are the union of a biome's entries (T:humidity digits as in section 1); continentalness uses
vanilla's names; erosion and weirdness are the union of its entries' ranges, so a biome with several
narrow slots shows one wide range. "temp / downfall" are the biome file's own values (they drive
grass colour and snow, not placement).

| Biome | Entries | Bands (T:humidity) | Continentalness | Erosion | Weirdness | temp / downfall | c: tags |
|---|---|---|---|---|---|---|---|
| `alpha_islands` | 1 | C:01234 T:01234 W:01234 H:01234 | mushroom | 0.57..1.00 | -1.00..1.00 | 0.7 / 0.8 | is_rare |
| `alpha_islands_winter` | 1 | F:01234 | mushroom | 0.57..1.00 | -1.00..1.00 | 0.0 / 0.75 | is_rare |
| `alpine_grove` | 35 | F:234 C:01234 T:234 | near, mid, far | -1.00..-0.38 | -0.57..0.57 | -0.2 / 0.8 |  |
| `alpine_highlands` | 28 | C:023 T:0 | ocean, coast, near, mid, far | -1.00..0.40 | -1.00..1.00 | 0.45 / 0.1 |  |
| `amethyst_canyon` | 3 | W:4 | mid, far | 0.40..0.57 | -1.00..1.00 | 0.95 / 0.9 | is_magical |
| `amethyst_rainforest` | 24 | W:4 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0.95 / 0.9 | is_magical |
| `ancient_sands` | 3 | H:0 | mid, far | 0.05..0.40 | -1.00..1.00 | 2 / 0 |  |
| `arid_highlands` | 3 | W:012 | mid, far | 0.05..0.40 | -1.00..1.00 | 1.6 / 0.1 |  |
| `ashen_savanna` | 6 | W:12 | near, mid, far | -1.00..-0.38 | -0.57..-0.05 | 1 / 0 |  |
| `basalt_cliffs` | 1 | W:01234 | ocean, coast | -1.00..-0.38 | -1.00..-0.57 | 0.5 / 0.3 |  |
| `birch_taiga` | 18 | C:23 T:23 | ocean, coast, near, mid, far | -1.00..-0.38 | -1.00..1.00 | 0.22 / 0.2 |  |
| `blooming_plateau` | 4 | C:12 T:12 | mid, far | 0.40..0.57 | -1.00..1.00 | 0.5 / 0.8 |  |
| `blooming_valley` | 6 | C:12 T:12 | mid, far | 0.40..0.57 | -0.27..0.27 | 0.7 / 0.8 |  |
| `brushland` | 38 | W:23 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 1.2 / 0.2 |  |
| `bryce_canyon` | 15 | H:4 | ocean, coast, near, mid, far | -1.00..0.05 | -1.00..1.00 | 2 / 0 |  |
| `caldera` | 6 | C:0123 | near, mid, far | -1.00..-0.38 | -1.00..1.00 | 0.45 / 0.8 |  |
| `cloud_forest` | 1 | C:34 | far | 0.57..1.00 | -1.00..1.00 | 0.25 / 0.8 |  |
| `cold_shrubland` | 27 | F:1 C:0 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0.14 / 0 frozen |  |
| `deep_warm_ocean` | 7 | H:01234 | mushroom, deep ocean | -1.00..1.00 | -1.00..1.00 | 0.5 / 0.5 |  |
| `desert_canyon` | 3 | H:1234 | mid, far | 0.33..0.40 | -1.00..1.00 | 2 / 0 |  |
| `desert_oasis` | 18 | H:234 | ocean, coast, near, mid, far | 0.05..1.00 | -1.00..1.00 | 2 / 0 |  |
| `desert_spires` | 2 | H:01234 | mid | 0.57..1.00 | -1.00..1.00 | 2 / 0 |  |
| `emerald_peaks` | 5 | F:34 | near, mid, far | -1.00..-0.38 | 0.40..0.93 | 0.1 / 0.4 |  |
| `forested_highlands` | 22 | C:034 T:3 | ocean, coast, near, mid, far | -1.00..0.40 | -1.00..1.00 | 0.36 / 0.2 |  |
| `fractured_savanna` | 8 | C:01 T:01 W:01 | mid, far | 0.57..1.00 | -1.00..1.00 | 1.1 / 0 |  |
| `frozen_cliffs` | 4 | F:01234 | ocean, coast | -1.00..-0.38 | -1.00..1.00 | 0 / 0.5 |  |
| `glacial_chasm` | 3 | F:01234 | mid, far | 0.40..1.00 | -1.00..1.00 | 0 / 0.5 |  |
| `granite_cliffs` | 8 | W:01234 H:01234 | ocean, coast | -1.00..-0.22 | -1.00..1.00 | 0.4 / 0.3 |  |
| `gravel_beach` | 4 | C:01234 T:01234 W:0123 | ocean, coast | -0.38..0.05 | -0.40..-0.05 | 0.8 / 0.4 |  |
| `gravel_desert` | 16 | F:012 | ocean, coast, near, mid, far | -1.00..1.00 | -0.05..1.00 | 0.14 / 0.5 frozen |  |
| `haze_mountain` | 2 | C:34 T:34 W:2 | far | 0.57..1.00 | -1.00..1.00 | 0.3 / 0.6 |  |
| `highlands` | 3 | C:12 T:12 | mid, far | 0.05..0.40 | -1.00..1.00 | 0.4 / 0.4 |  |
| `hot_shrubland` | 28 | W:023 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0.8 / 0.4 |  |
| `ice_marsh` | 10 | F:01234 | ocean, coast, near, far | -0.38..1.00 | -1.00..1.00 | 0.14 / 0.9 frozen |  |
| `jungle_mountains` | 4 | C:4 T:4 W:234 | mid, far | 0.57..1.00 | -1.00..1.00 | 0.95 / 0.9 |  |
| `lavender_forest` | 5 | C:4 | ocean, coast, near | -0.38..1.00 | 0.05..1.00 | 0.7 / 0.8 | is_magical |
| `lavender_valley` | 2 | C:34 T:34 | mid, far | 0.40..0.57 | -1.00..1.00 | 0.7 / 0.8 | is_magical |
| `lush_desert` | 6 | H:24 | mid, far | 0.05..0.33 | -1.00..1.00 | 2 / 0 |  |
| `lush_valley` | 3 | C:3 | mid, far | 0.40..0.57 | -1.00..1.00 | 0.4 / 0.8 |  |
| `mirage_isles` | 1 | F:01234 C:01234 T:01234 | mushroom | -0.78..-0.22 | -1.00..1.00 | 0.7 / 0.8 | is_magical, is_rare |
| `moonlight_grove` | 30 | C:4 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0.7 / 0.8 | is_magical |
| `moonlight_valley` | 3 | C:1 | mid, far | 0.05..0.57 | -1.00..1.00 | 0.7 / 0.8 | is_magical |
| `orchid_swamp` | 4 | C:01 T:01 | ocean, coast, near, far | -0.22..1.00 | -1.00..1.00 | 0.8 / 0.9 |  |
| `painted_mountains` | 10 | H:01 | near, mid, far | -1.00..-0.38 | -1.00..1.00 | 1 / 0.3 | is_magical |
| `red_oasis` | 1 | H:01234 | far | -0.22..0.05 | -1.00..1.00 | 2 / 0 |  |
| `rocky_jungle` | 21 | W:34 | near, mid, far | -1.00..0.40 | -1.00..1.00 | 0.95 / 0.9 |  |
| `rocky_mountains` | 21 | C:01234 T:01234 | near, mid, far | -1.00..-0.38 | -0.93..0.93 | 0.3 / 0.3 |  |
| `rocky_shrubland` | 3 | F:0 | mid, far | 0.33..0.40 | -1.00..1.00 | 0.14 / 0 frozen |  |
| `sakura_grove` | 34 | C:4 T:4 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0.7 / 0.8 |  |
| `sakura_valley` | 5 | C:4 T:4 W:4 | mid, far | 0.40..0.57 | -1.00..1.00 | 0.7 / 0.8 |  |
| `sandstone_valley` | 2 | H:01234 | mid, far | 0.40..1.00 | -1.00..1.00 | 2 / 0 |  |
| `savanna_badlands` | 34 | W:012 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 1 / 0 |  |
| `savanna_slopes` | 10 | W:01 | near, mid, far | -1.00..-0.38 | 0.05..1.00 | 1 / 0 |  |
| `scarlet_mountains` | 5 | F:01234 | near, mid, far | -1.00..-0.38 | -0.93..-0.40 | 0.1 / 0.4 | is_magical |
| `shield` | 19 | C:1 | ocean, coast, near, mid, far | -1.00..0.05 | -1.00..1.00 | 0.4 / 0.8 |  |
| `shield_clearing` | 1 | C:1 | mid, far | -1.00..-0.38 | -0.05..0.05 | 0.4 / -0.5 |  |
| `shrubland` | 35 | C:0 T:0 W:3 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 1.2 / 0 |  |
| `siberian_grove` | 7 | C:4 | near, mid, far | -1.00..-0.38 | 0.05..1.00 | 0.13 / 0.8 |  |
| `siberian_taiga` | 65 | F:4 C:4 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0.13 / 1.2 |  |
| `skylands_autumn` | 1 | C:1 T:1 | deep ocean | -0.22..0.05 | 0.19..1.00 | 0.5 / 0.5 | is_magical, is_rare |
| `skylands_spring` | 1 | C:3 T:3 | deep ocean | -0.22..0.05 | 0.19..1.00 | 0.5 / 0.5 | is_magical, is_rare |
| `skylands_summer` | 1 | W:2 | deep ocean | -0.22..0.05 | 0.19..1.00 | 0.5 / 0.5 | is_magical, is_rare |
| `skylands_winter` | 1 | C:2 | deep ocean | -0.22..0.05 | 0.19..1.00 | 0.2 / 0.5 | is_magical, is_rare |
| `snowy_badlands` | 22 | F:01 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | 0 / 0.5 |  |
| `snowy_cherry_grove` | 7 | F:01 | mid, far | -0.78..0.05 | 0.05..1.00 | 0.1 / 0.8 |  |
| `snowy_maple_forest` | 9 | C:0 | near, mid, far | -1.00..-0.38 | -0.57..0.57 | 0.1 / 0.2 |  |
| `snowy_shield` | 9 | C:1 | near, mid, far | -1.00..-0.38 | -0.57..0.57 | 0.1 / 0.8 |  |
| `steppe` | 4 | C:01 T:0 | mid, far | 0.05..0.40 | -1.00..1.00 | 0.4 / -0.5 |  |
| `stony_spires` | 6 | C:234 T:234 W:4 | mid, far | -1.00..1.00 | -1.00..1.00 | 0.7 / 0.3 |  |
| `temperate_highlands` | 25 | C:034 T:34 | ocean, coast, near, mid, far | -1.00..0.40 | -1.00..1.00 | 0.5 / 0.2 |  |
| `tropical_jungle` | 12 | W:34 | ocean, coast, near, mid, far | -1.00..0.05 | -1.00..1.00 | 0.95 / 0.9 |  |
| `valley_clearing` | 2 | C:023 T:023 | mid, far | -1.00..-0.38 | -0.05..0.05 | 0.4 / -0.5 |  |
| `volcanic_crater` | 1 | W:01234 | near, mid, far | -1.00..-0.38 | -1.00..-0.77 | 1 / 0.3 |  |
| `volcanic_peaks` | 3 | W:01234 | near, mid, far | -1.00..-0.38 | -0.77..-0.40 | 1 / 0.3 |  |
| `warm_river` | 8 | H:01234 | ocean, coast, near, mid, far | -1.00..1.00 | -0.05..0.05 | 0.5 / 0.5 |  |
| `warped_mesa` | 1 | W:01234 H:01234 | mushroom | -0.78..-0.22 | -1.00..1.00 | 2 / 0 | is_magical, is_rare |
| `white_cliffs` | 4 | C:01234 | ocean, coast | -1.00..-0.38 | -1.00..1.00 | 0.4 / 0.3 |  |
| `white_mesa` | 5 | W:01 | near, mid, far | -1.00..0.57 | -1.00..1.00 | 2 / 0 |  |
| `windswept_spires` | 4 | C:0134 | mid | 0.57..1.00 | -1.00..1.00 | 0.2 / 0.3 |  |
| `wintry_forest` | 50 | F:234 | ocean, coast, near, mid, far | -1.00..1.00 | -1.00..1.00 | -0.5 / 0.4 |  |
| `wintry_lowlands` | 6 | F:234 | mid, far | 0.05..0.33 | -1.00..1.00 | -0.5 / 0.4 |  |
| `yellowstone` | 6 | C:23 | near, mid, far | -1.00..0.05 | -1.00..1.00 | 0.24775 / 0.8 |  |
| `yosemite_cliffs` | 2 | C:0 T:0 | mid, far | 0.40..0.57 | -1.00..1.00 | 0.375 / 0.8 |  |
| `yosemite_lowlands` | 3 | C:0 T:0 | mid, far | 0.40..0.57 | -0.27..0.27 | 0.25 / 0.8 |  |

Cave biomes (depth above 0, unaffected by Deep Time's surface climate): andesite, deep, diorite,
frostfire, fungal, granite, infested, mantle, thermal and tuff caves, underground jungle.

Two placement facts that matter later: Terralith puts its realistic cold dry land (`cold_shrubland`,
`gravel_desert`) in the upper frozen band and its red `snowy_badlands` in the lower (the EF block of
`placement.json` already steers by this), and it has only one wetland outside vanilla's swamps:
`ice_marsh`, frozen band, swamp slot.

### 2.3 Realistic and fantasy

Of the 84 surface biomes, 14 carry `c:is_magical` and 8 `c:is_rare` (18 distinct; the Skylands and
the special islands only appear at deep-ocean or mushroom-island continentalness). Two more are
unrealistic though untagged: `snowy_badlands` (red terracotta under snow) and `emerald_peaks`
(green-tinted peaks). Stylised but plausible: the sakura and lavender biomes (lavender is tagged
magical), `snowy_cherry_grove`, the `blooming_*` pair, `haze_mountain`. The rest read as realistic,
and many are named for a real place or rock: `shield` (the Canadian Shield), `yosemite_*` (granite),
`white_cliffs` (chalk), `granite_cliffs`, `basalt_cliffs`, `yellowstone`, `caldera`,
`volcanic_peaks` and `volcanic_crater`, `bryce_canyon` (hoodoos in red beds), `painted_mountains`
(the Painted Hills; tagged magical for its colours), `sandstone_valley`, `desert_canyon`,
`glacial_chasm`, `siberian_taiga`, `cloud_forest`.

### 2.4 Maintenance

Modrinth (API, read 2026-10-03): six 1.21.1 NeoForge releases since 2024-07-07; 2.6.2 for 1.21.1 was
published 2026-06-09, a backport of the 26.1 line's 2.6.2. The project is active on the new game
versions (2.6.3 and 2.6.4 for 26.2 in July, 2.6.5 for 26.3 on 2026-09-27). 23.6 million downloads.
Expect 1.21.1 to get backports at the authors' discretion rather than every release.

### 2.5 Licence and pack terms

The Stardust Labs License (`license.txt` in the jar, last altered 2023-02-05; the mods.toml's "MIT" is
not the governing text). What it says, in short:

- Personal use and use on servers are allowed, modified or not, as long as modified versions are not
  distributed.
- Publicly distributed modpacks may include it **unmodified**, whole, with credit and a link. Using a
  modified version in a public modpack needs permission, "including cases where the datapack is
  required to be downloaded separately, with parts being overridden"; the exceptions are adding or
  altering structures and changing loot tables.
- Copying portions of its code into another project is forbidden, except for compatibility addons
  between Stardust Labs packs and other packs.
- A clause forbids using Stardust Labs works as AI training data and to "configure, test, debug, or
  augment any artificial intelligence ... or other generative system".

For Deep Time: it never ships Terralith (it does not); it may ship data that names Terralith's biomes
(tags, its own placement or veto tables), which is compatibility work; it should not ship, and MIC
should not carry, datapack overrides of Terralith's own files (biome JSON, parameter list, or
Lithostitched injectors that replace its biomes), so every fix below acts in Deep Time's selection.
The AI clause is an open question for Ben (section 10, Q5): this project's agents read Terralith's
jar and run GameTests with it.

### 2.6 With Tectonic and Lithostitched in MIC

MIC carries Terralith 2.6.2, Terralith ReStoned 1.3.1, Lithostitched 1.8.0+beta6 and Tectonic 3.0.28.
From the jar study of 2026-09-27 (`mod_compatibility_biomes_ores_fantasy.md`, Key question 2) and
this survey:

- Lithostitched is what makes Terralith's parameter list valid (it adds the `lithostitched:biomes`
  field and merges lists); Deep Time's `preset: minecraft:overworld` therefore reads Terralith's
  1,713 entries. The MIC GameTests and the biome census below confirm that Terralith's biomes appear
  in Deep Time worlds.
- Tectonic touches no biome placement. When Terralith is present it swaps in a combined
  "Terratonic" overworld noise settings file; Deep Time's surface rule delegates to the pack's
  overworld settings, so that combined file's surface rule paints Deep Time's terrain, while its
  terrain density functions do not shape a Deep Time overworld. Its mixins that key on the noise
  settings path `overworld` should not fire for `deeptime:planet` (path `planet`; read, not tested); its snowline mixin (a Y offset on
  biome temperature) applies to every biome, so it moves snow on Deep Time terrain too (not
  measured here). `placement.json`'s comment records that "MIC's Tectonic router stretches erosion",
  which is why some of the EF clamps sit where they do.
- Terralith ReStoned only restores vanilla's andesite, diorite and granite blobs (six placed-feature
  overrides).
- Skylands: their density-function spline does not run in Deep Time terrain, but their four biomes
  stay in the parameter list at deep-ocean continentalness. None appeared on the measured planet
  (section 3); Terralith's config can switch them off for MIC.

## 3. What Terralith does with Deep Time's climate today (measured)

The world: `earthlike_quick_16k`, scattered continents, 500 Myr, seed 1, baked on 2026-10-03 for the
volcanic-biomes task (`deeptime-runs/volcanic-biomes/vb-world` on the VM); `/deeptime probe biomes 16`
through each profile's biome source; 51,860 land columns at a step of 16 blocks; shares by
`tools/review/biomemap_shares.py`. MIC means Terralith; TerraBlender means Biomes O' Plenty with
regions dealt per province. This planet is colder than Earth (E is 29 % of land); the wetland rule
on it still sent part of Dfc to swamp, which the open task "taiga instead, peatland if present"
changes.

| Class (share of land) | MIC: Terralith (top shares) | Vanilla (top shares) | Biomes O' Plenty (top shares) | Mismatch with Terralith |
|---|---|---|---|---|
| Af (10.2 %) | tropical_jungle 23, mangrove_swamp 21, bamboo_jungle 14, sparse_jungle 12, jungle 12, brushland 8 | jungle 30, bamboo_jungle 29, mangrove 22 | bamboo_jungle 17, rainforest 16, jungle 14, mangrove 13, floodplain 13, rocky_rainforest 13 | brushland in rainforest (small) |
| Am (1.9 %) | **brushland 56**, sparse_jungle 19, jungle 10 | sparse_jungle 34, jungle 18, plains 16, forest 16 | sparse_jungle 28, rainforest 16, plains 13, woodland 11 | **monsoon forest drawn as dry scrub** |
| Aw (5.4 %) | savanna 50, brushland 32 | savanna 50, forest 22, plains 12 | savanna 30, scrubland 15, woodland 11 | fine (no dry forest anywhere) |
| As (0.4 %) | brushland 68, savanna 10 | plains 40, forest 35 | plains 34, woodland 25 | fine |
| BWh (2.2 %) | desert 53, eroded_badlands 10, badlands 8, windswept_savanna 7, ancient_sands 6 | desert 78 | desert 44, dryland 23, wasteland 12 | fine |
| BWk (5.8 %) | desert 11, badlands 9, temperate_highlands 8, eroded_badlands 8, snowy_slopes 8, savanna 7, cold_shrubland 6 | plains 31, desert 23, savanna 16 | plains 23, desert 15, savanna 10, dryland 9 | the hot part (above 10 °C) is desert by design |
| BSh (6.8 %) | savanna 58, savanna_badlands 10, arid_highlands 8, white_mesa 8 | savanna 83 | savanna 46, scrubland 23, mediterranean_forest 15 | fine |
| BSk (8.9 %) | plains 43, **shield 10**, **blooming_plateau 8**, savanna 5, highlands 4 | plains 71 | plains 41, prairie 14 | **cold steppe gets a boreal rock forest and a flower plateau; `steppe` 0 %** |
| Csa (0.8 %) | plains 58, blooming_plateau 26 | plains 81 | plains 69, lavender_field 9 | **no Mediterranean scrub** |
| Csb (4.1 %) | plains 56, forest 16, birch_taiga 7 | plains 61, forest 25 | plains 40, forest 13, prairie 11 | **no Mediterranean scrub** |
| Cwa, Cwb (1.2 %) | old-growth birch, birch_taiga, forest, white_cliffs 6 (Cwb) | forest, birch | redwood_forest, mystic_grove (fantasy), forest | white_cliffs off carbonate coasts |
| Cfa (2.8 %) | forest 39, highlands 30, brushland 6 | forest 77 | forest 64, redwood 9 | fine |
| Cfb (10.3 %) | birch_forest 20, forest 14, birch_taiga 12, old-growth birch 11, forested_highlands 6, dark_forest 6, **lavender_valley 4** | birch_forest 33, forest 26 | birch_forest 16, forest 16, shrubland 11, overgrown_greens 11 | fantasy (lavender) |
| Cfc (0.4 %) | dark_forest 33, taiga 12, **moonlight_grove 4** | dark_forest 32 | overgrown_greens 15, maple_woods 13 | fantasy (moonlight) |
| Ds, Dw (1.4 %) | forest, taiga, **white_cliffs 10–39**, alpine_grove, shield 7 | forest, taiga | forest, taiga, coniferous_forest | white_cliffs off carbonate coasts |
| Dfb (3.9 %) | taiga 58, forested_highlands 11, swamp 10, lush_valley 6 | taiga 78, swamp 10 | taiga 62, maple_woods 13, swamp 9 | cold wetland (open task) |
| Dfc (4.1 %) | taiga 34, old_growth_spruce_taiga 30, **moonlight_grove 7**, swamp 6 | old-growth spruce 38, taiga 32, swamp 5 | maple_woods 22, old-growth spruce 22, taiga 15 | fantasy; cold wetland |
| ET (18.6 %) | cold_shrubland 28, snowy_plains 24, gravel_desert 16, snowy_slopes 14, snowy_beach 6, **snowy_badlands 3**, **scarlet_mountains 2** | snowy_plains 77 | cold_desert 51, tundra 17, snowy_plains 13 | no summer tundra; red terracotta and red-leaf mountains |
| EF (10.7 %) | ice_spikes 35, snowy_slopes 26, frozen_peaks 16, gravel_desert 8, frozen_cliffs 3, volcanic_peaks 3 | snowy_plains 29, ice_spikes 29, snowy_slopes 17 | cold_desert 28, tundra 20 | fine (volcanic peaks now tie to real volcanoes) |

Over all land, 44 Terralith biomes appear; the ones tagged `c:is_magical` take 1.35 % of the land
(scarlet mountains 0.51, lavender valley 0.40, moonlight grove 0.29, painted mountains 0.13,
amethyst canyon 0.02, lavender forest under 0.01), `snowy_badlands` another 0.54 % and sakura grove
0.05 %. Terralith's `steppe`, `shrubland`,
`cloud_forest`, `yosemite_*` and `stony_spires` are at or near 0 %: the slots they need (erosion,
continentalness, weirdness) are not where Deep Time puts those classes.

## 4. The alternatives

All facts below are from the jars (downloaded from Modrinth on 2026-10-03 and read with Python
`zipfile`), the Modrinth API (versions, dates, licences, page text) and, for the four code-placed
mods, their published sources on GitHub (BOP branch `1.21.1`, Nature's Spirit `1.21.1`, BWG
`1.21.1`, Regions Unexplored `21.1`). "Surface biomes" counts new biome ids that are not cave,
Nether or End biomes. Blocks are the mod's own block-state files.

| Mod | 1.21.1 NeoForge release | New surface biomes | Own blocks | Placement | Licence and pack terms | Maintenance | Realism |
|---|---|---|---|---|---|---|---|
| **Terralith** | 2.6.2, 2026-06-09 | 84 (incl. 4 Skylands, 4 rare islands) | 0 | Lithostitched parameter list | Stardust Labs License: unmodified in public packs, no overrides of its files (2.5) | active (26.x), 1.21.1 by backport | mostly realistic, 14 tagged magical |
| Biomes O' Plenty | 21.1.0.14, 2026-06-20 (11 releases) | 59 (69 biome files incl. 5 Nether, 3 End, 2 cave) | 429 | TerraBlender: 3 regions (primary, secondary, rare) | All rights reserved; "may include this mod in a Modrinth-hosted modpack" if not rehosted or sold | active (updated 2026-10-02) | mixed; a few fantasy (mystic grove, ominous woods, auroral garden, fungal jungle, origin valley) |
| Regions Unexplored | 0.6.2, 2026-07-12 (11 releases) | about 67, incl. rivers, beaches, 2 seas (78 files incl. 4 cave, 7 Nether) | 859 | Lithostitched: splits vanilla biomes by its own noise and temperature/humidity ranges | MIT | active (repo pushed 2026-08-30); author also writes Lithostitched and Tectonic | mostly realistic; a few fantasy (prismachasm, ashen woodland, alpha grove) |
| Nature's Spirit | 2.2.5, 2025-09-04 (3 releases) | 51 | 773 | TerraBlender: 5 regions | custom licence (GitHub, not read) | slowing on 1.21.1 (no release in 13 months; repo active on 26.1) | realistic, flower-heavy |
| Oh The Biomes We've Gone | 2.6.0, 2026-06-14 (45 releases) | 55, incl. 3 beaches and 2 seas | 1,024 | TerraBlender: 3 regions, plus swapping vanilla swamps | All rights reserved; no pack statement on its page | active (repo pushed 2026-10-03) | mixed; several fantasy (enchanted tangle, weeping witch forest, crag gardens) |
| William Wythers' Overhauled Overworld | 2.6.7, 2026-05-28 | 0 on 1.21.1 (restyles 53 vanilla biomes) | 0 | rewrites vanilla biome files | CC BY-NC-ND 4.0 | active | realistic |
| Expanded Ecosphere (Wythers) | 3.4.7, 2026-07-04 (6 releases) | 189 (incl. 14 cave) | 0 | full `dimension/overworld.json` override (cristel-lib) | CC BY-NC-ND 4.0; packs allowed with credit, no changes except config | active | the most realistic set surveyed |
| Wilder Wild | none: Fabric/Quilt only on 1.21.1 (NeoForge from 26.x) | 27 (30 files incl. 3 caves) | 269 | Fabric; would need Sinytra Connector | FrozenBlock Modding Oasis License (not read) | active | realistic |
| Geophilic | 3.7, 2026-09-24 | 0 (restyles 31 vanilla biomes) | 0 | data-only overlays of vanilla biome files | All rights reserved | active | realistic |
| CliffTree | 3.3 backport, 2026-09-11 | 20, incl. 2 seas (29 files incl. 9 cave) | 0 | full parameter-list override (collides with Terralith) | CC BY-NC-SA 4.0 | active | realistic, vanilla style |
| Spawn | 4.0.8, 2026-09-14 | 10, mostly islands and coasts | 163 | Lithostitched biome injectors (ocean islands) | All rights reserved | active (1.21.1 only) | realistic |
| Let's Do BloomingNature | 1.1.11, 2026-09-18 | 16 | 503 | Biolith | custom; "not intended to be realistic" (its page) | active | stylised |
| Woodlands: Extra | 1.3.3, 2025-02-02 | 42 | 0 | overrides the Normal world preset only | All rights reserved | stalled | realistic |
| Mystic's Biomes, Good Ending, Luminous, Autumnity, Environmental, Nyctophobia | various | 1–7 each | yes | biome modifiers or TerraBlender | various | various | themed (spooky, colourful, autumn) |

Notes per mod, as they bear on Deep Time:

- **Biomes O' Plenty.** The one TerraBlender mod Deep Time already tests (the `-PwithTerraBlender`
  GameTests). Its tables (rows temperature, columns humidity) put `cold_desert` and `tundra` at F:0–1,
  `muskeg` at F:2, `bog` and `wetland` in the cold swamp slot, `marsh` in the temperate swamp slot,
  `bayou` and `floodplain` in the warm and hot swamp slots, `hot_springs` in the frozen swamp slot,
  `mediterranean_forest` at W:0–1, `scrubland` W:0–1, `woodland` W:2, `volcano` and `volcanic_plains`
  at W:3–4, `wasteland` and `dryland` in the hot band. On the measured planet it gave ET a cold desert
  (51 %) and tundra (17 %), the closest anything came to the right polar mix, and Af a floodplain.
- **Regions Unexplored.** Its biomes replace vanilla ones, so they follow whatever list is in place,
  Terralith's included. The placements that matter: `fen` takes part of vanilla's swamp where the
  temperature parameter is −0.45 to −0.1 (the cold band) and humidity above −0.35; `tundra` takes
  snowy plains below humidity −0.1; `shrubland` takes plains below temperature −0.1; `steppe`,
  `prairie` and `dry_bushland` take savanna; `outback`, `joshua_desert` and `saguaro_desert` take
  desert; `bayou` and `marsh` take swamp. It has peat and silt ground blocks (its cave carvers are
  patched to cut them). Being Lithostitched-based and by the author of Lithostitched and Tectonic, it
  is the natural companion to Terralith in MIC.
- **Nature's Spirit.** Placed in code across five regions; realistic but adds few of the gaps
  (`tundra`, `chaparral`, `xeric_plains`, `marsh`, `cypress_fields`); its 1.21.1 line has been quiet
  since 2025-09.
- **Oh The Biomes We've Gone.** Large and colourful; it has `atacama_outback`, `mojave_desert`,
  `canadian_shield`, `crimson_tundra`, `pale_bog`, `dead_sea` (a hot sea) and a `shattered_glacier`;
  1,024 own block states.
- **Expanded Ecosphere.** Biome for biome the best match to Deep Time's needs: `fen`, `snowy_fen`,
  `snowy_bog`, `berry_bog`, `marsh`, `tundra`, `snowy_tundra`, `ice_cap`, `glacial_cliffs`,
  `salt_lakes_white/pink/turquoise`, `danakil_desert`, `guelta`, `billabong`, `pantanal`,
  `flooded_savanna`, `chaparral`, `kwongan_heath`, `scrubland`, `dry_tropical_forest`,
  `dry_tropical_grassland`, `tsingy_forest` (karst pinnacles), `tepui`, `volcano`,
  `tropical_volcano`, `icy_volcano`, `volcanic_crater`, `black_beach`, `mud_pools`,
  `thermal_taiga`, `coastal_mangroves`, `temperate_rainforest`. All vanilla blocks. But it replaces
  the whole overworld dimension, so its biomes do not reach Deep Time's preset as packaged (section 1);
  it extends temperature beyond vanilla's −1 to 1 (`ice_cap` below −1, `tibesti_mountains` and
  `mud_pools` above 1.05), which Deep Time's band coordinates never produce; its licence forbids
  derivatives; it is incompatible in practice with Terralith (both define the overworld biome list)
  and it marks WWOO "discouraged". Making it work would need a Deep Time-side compatibility list (its
  biome ids placed by Deep Time's own coordinates), which is our own file and allowed, plus a test of
  what its dimension override does to a Deep Time world.
- **Wilder Wild.** Not available for NeoForge 1.21.1, so out of scope; noted because it is often
  suggested (cypress wetlands, tundra, temperate rainforest, oasis).
- **WWOO and Geophilic.** No new biomes on 1.21.1; they restyle vanilla ones and would work with
  Deep Time as restyled vanilla biomes. Not a solution to gaps.
- **CliffTree.** Realistic and vanilla-blocked, with `bog` (frozen swamp slot), `tundra` (F:0–1),
  `glacier_valley`, `glacier_cliff`, `kelp_forest`; but its full parameter-list override collides
  with Terralith's, so it is an either-or with Terralith.

## 5. Gaps by climate class

"Best" is the most fitting biome a mod can give that class in the cells of section 1 (from the jars
and sources), whether or not Deep Time reaches it today; Terralith's column also says what the
measurement gave. **Bold** marks the gaps that matter.

| Class | Terralith | Vanilla | BOP | RU | NS | BWG | EE | Gap |
|---|---|---|---|---|---|---|---|---|
| Af | tropical_jungle, rocky_jungle, jungle (measured: good) | jungle, bamboo jungle | rainforest, rocky_rainforest, floodplain | rainforest (replaces jungle) | tropical_woods | tropical_rainforest | tropical_rainforest, flooded_rainforest | none |
| Am | tropical_jungle, sparse jungle exist; **measured: brushland 56 %** | sparse_jungle, jungle | rainforest, woodland | sparse_rainforest | sparse_tropical_woods | jacaranda_jungle | eucalyptus_jungle, sparse_bamboo_jungle | **placement** (fixable in the table) |
| Aw, As | savanna, brushland, hot_shrubland | savanna | woodland, scrubland, old_growth_woodland | dry_bushland, prairie (replace savanna) | oak_savanna, wooded_drylands | baobab_savanna, ironwood_gour | dry_tropical_forest, wooded_savanna, dry_savanna | **tropical dry forest**: Terralith and vanilla have none (moderate) |
| BWh | desert, ancient_sands, sandstone_valley, desert_canyon, oasis | desert, badlands | dryland, wasteland | outback, joshua and saguaro deserts | lively_dunes, stratified_desert | mojave_desert, atacama_outback | deep_desert, cactus_desert, sand_dunes, red_desert | none |
| BWk (0–10 °C) | cold_shrubland, temperate_highlands, steppe, alpine_highlands | plains | cold_desert (F band), dryland | steppe, shrubland | shrubby_highlands | coconino_meadow | cold_plains, tundra | small (rocky cold desert floor) |
| BWk ≤ 0 °C, EF barren | gravel_desert, cold_shrubland | snowy_plains | cold_desert, tundra | tundra | tundra | shattered_glacier, crimson_tundra | snowy_tundra, icy_crags | none with Terralith (gravel desert is a fair polar desert) |
| BSh | savanna, savanna_badlands, arid_highlands, white_mesa | savanna | scrubland, mediterranean_forest | dry_bushland, steppe | xeric_plains | araucaria_savanna | dry_savanna, scrubland | small (thorn scrub) |
| BSk | **`steppe` exists but is never reached (measured 0 %); gets plains, shield, blooming_plateau** | plains | prairie | steppe, prairie (replace savanna), shrubland | prairie | prairie | cold_plains, highland_plains, subtropical_grassland | **placement** (erosion clamp) |
| Cs | `shrubland` exists (T:0); **measured: plains, blooming_plateau** | plains, forest | mediterranean_forest (W band), lavender_field | shrubland, orchard | chaparral (W band) | firecracker_chaparral | chaparral, kwongan_heath, scrubland | **Mediterranean scrub** (fixable by placement for Terralith; none in vanilla) |
| Cw, Cf | forest, birch, forested_highlands, temperate_highlands, highlands | forest, birch, dark forest | redwood, woodland, overgrown_greens | old_growth_forest, deciduous_forest | maple_woodlands, redwood | temperate_grove | ancient beech forests, temperate_rainforest | **temperate rainforest** (Cfb/Cfc coasts; small); fantasy to remove |
| Tropical highland (Cfb/Cwb at low latitude, high and wet) | `cloud_forest`, `haze_mountain` exist (unreached) | none | none | none | none | none | highland_tropical_rainforest, huangshan_highlands | **cloud forest**: Terralith has it; needs a placement rule |
| Da/Db | forest, birch_taiga, forested_highlands, lush_valley | forest, taiga | coniferous_forest, maple_woods, seasonal_forest | maple_forest, willow_forest | aspen_forest, fir_forest | aspen_boreal, maple_taiga | boreal forests, pine_barrens | none |
| Dc/Dd | taiga, siberian_taiga, siberian_grove, old-growth spruce | taiga, old-growth taigas | coniferous_forest, dead_forest, snowy_coniferous_forest | boreal_taiga variants | boreal_taiga | frosted_coniferous_forest | larch_taiga, ancient_taiga | none |
| Cold wetland (Dfb/Dfc/ET low, flat, wet) | **none** (`ice_marsh` is frozen-band, swamp slot) | swamp (being replaced by taiga) | bog, muskeg, wetland | fen (cold swamp, peat blocks) | marsh (temperate only) | none (its `pale_bog` is a warm-slot swap) | fen, snowy_fen, snowy_bog, berry_bog | **peatland**: the largest real gap |
| Temperate and warm wetland | orchid_swamp (C:0–1) | swamp, mangrove swamp | marsh, bayou, floodplain | marsh, bayou, fungal_fen | marsh, bamboo_wetlands | bayou, cypress_swamplands, white_mangrove_marshes | marsh, pantanal, bayou | small (floodplain marsh) |
| ET | cold_shrubland, gravel_desert; **measured also snowy_plains 24 %, snowy_badlands 3 %, scarlet_mountains 2 %** | snowy_plains | tundra, cold_desert | tundra (replaces snowy plains) | tundra | crimson_tundra | tundra, snowy_tundra, crimson_tundra | **summer tundra** (green-brown, not snow) and two unrealistic biomes to remove |
| EF | ice_spikes, frozen_peaks, snowy_slopes, glacial_chasm, gravel_desert + Deep Time's ice cover | ice_spikes, snowy_plains | none special | spires, icy_heights | sleeted_slopes | shattered_glacier | ice_cap (unreachable), glacial_cliffs | none (Deep Time's ice cover does the work) |

## 6. Gaps by geologic setting

What Deep Time knows, and what each source offers for it.

| Setting (Deep Time knows) | Terralith | Vanilla | Others | Gap and nature |
|---|---|---|---|---|
| Volcanoes, calderas, fresh ash, hot-spring fields | `volcanic_peaks`, `volcanic_crater`, `caldera`, `yellowstone`, `ashen_savanna`, `basalt_cliffs` | none | BOP volcano, volcanic_plains, hot_springs; EE volcano, tropical/icy volcano, mud_pools, thermal taiga | none: Terralith is the best set; tie to real volcanoes is decided |
| Lava fields and flood basalts (young) | none (ashen savanna is warm-band ash) | none | BOP volcanic_plains; EE savanna_basaltic_incursions, black_beach | **lava fields**: surface feature |
| Meteor craters | none (Terralith's crater density functions make terrain, not a biome) | none | none | none: Deep Time carves craters; climate biome on the floor is right |
| Salt flats and playas | none | none | EE salt_lakes_*, danakil_desert; BWG dead_sea (sea) | **salt flats**: surface feature (decided 2026-10-03) |
| Carbonate karst | `stony_spires` (tower look), `white_cliffs` (chalk coast) | none | EE tsingy_forest; BOP jade_cliffs (pillars) | **karst** (dolines, pavement, towers): surface feature plus gating `stony_spires` |
| Glaciers and ice sheets | `glacial_chasm`, `frozen_cliffs` + Deep Time's ice cover | ice_spikes, frozen_peaks | EE ice_cap, glacial_cliffs; CliffTree glacier_valley | none |
| Glacial outwash (sandur) in front of ice | `gravel_desert` (F band), `gravel_beach` | windswept_gravelly_hills | none | small: steer proglacial plains to gravel desert, or leave |
| Fjords | coastal cliff biomes | stony_shore | none | none: terrain (glacial troughs drowned by the sea), not a biome |
| Rifts | none | none | none | none: terrain and volcanoes |
| Coasts: cliffs by rock | `granite_cliffs`, `white_cliffs`, `basalt_cliffs`, `frozen_cliffs`, `gravel_beach` | beach, stony_shore | EE calcite_coast, black_beach | **gating**: cliffs by coastal rock (white cliffs now appear on any cold rocky coast) |
| Tidal flats and salt marsh | none | mangrove swamp (tropics only) | Spawn tide_pool; none else | **tidal flats** (temperate): surface feature, or leave |
| Shelves | ocean biomes | ocean biomes | CliffTree kelp_forest | none for this survey |
| Deltas | none | swamp or mangrove through the wetland rule | BOP floodplain; EE pantanal | small: extend the wetland rule to delta cells regardless of aridity |
| Floodplains and riparian forest | none | swamp | BOP floodplain; EE billabong, flooded savanna, river biomes | small to moderate: gallery forest along rivers in dry climates (surface feature) |
| Lakes | none | lake water | none | none: water from Deep Time's hydrology; climate biome on the shore |
| Precambrian shields (province) | `shield`, `snowy_shield`, `shield_clearing` | none | BWG canadian_shield | **gating**: put Shield only on shield provinces (now it lands on cold steppe) |
| Granite plutons, glacial valleys | `yosemite_cliffs`, `yosemite_lowlands` | none | none | **gating**: Yosemite only on exposed granite (now unreached) |
| Red beds, sandstone in arid land; badlands | `bryce_canyon`, `painted_mountains`, `white_mesa`, `sandstone_valley`, `desert_canyon`, `savanna_badlands` | badlands (terracotta bands) | EE red_rock_canyon, badlands_canyon | **gating**; and vanilla-style terracotta bands draw over Deep Time's real strata (to check) |
| Loess | none | none | none | leave (no loess block in vanilla) |

## 7. One fix per major gap

Each gap gets one recommendation. Costs use the scale in section 9.

1. **Fantasy biomes in realistic classes** (1.35 % of land on the test planet). Fix: a veto in
   Deep Time's biome query: a biome in `#c:is_magical`, `#c:is_rare` or a new `#deeptime:biome_veto`
   (for `snowy_badlands`, `emerald_peaks` and other mods' equivalents) is replaced by the pick at
   nearby coordinates that is not vetoed, or by the vanilla biome of the cell. The volcanic tie-in
   (decided 2026-10-03, built on another branch; its census already has `forbidden` and `volcanic`
   checks) needs the same "keep out" machinery. Works for any pack that follows the `c:` tags.
   `painted_mountains` is tagged magical for its colours but is a real landform (the Painted Hills);
   the geology gate of fix 3 can let it back in on red beds.
   Cost **S** on top of the volcanic mechanism, **M** without it.
2. **Monsoon forest drawn as scrub (Am), Mediterranean as plains (Cs), cold steppe missing (BSk),
   tundra as snow (ET).** Fix: a Terralith-tuned `placement.json`, shipped by Deep Time as a
   resource overlay with a `neoforge:mod_loaded terralith` condition (as Tectonic ships its
   Terralith overlay), so vanilla and BOP keep today's table. Contents to measure: Am humidity at
   least 4.05 (W:4: tropical jungle); Cs humidity into band 0 of the temperate band (Terralith's
   `shrubland`, a dry scrub); BSk erosion clamped to 0.05–0.40 (Terralith's `steppe` slot; shares
   that slot with `highlands`, so measure); ET humidity held at band 1 like barren EF (more
   `cold_shrubland`, less `snowy_plains`). Cost **S** (one table, one probe run per variant, the
   census checks), about 1–2 days with measurements.
3. **Geology-named biomes on the wrong ground** (`shield` on cold steppe, `white_cliffs` on any cold
   coast, Yosemite never). Fix: generalise the volcanic tie-in into a small "geology gate" table:
   `shield`/`snowy_shield`/`shield_clearing` only on shield provinces; `yosemite_*` only on exposed
   granite with glacial relief; `white_cliffs` on carbonate coasts, `granite_cliffs` on granite
   coasts, `basalt_cliffs` on basalt coasts; `bryce_canyon`, `painted_mountains`, `white_mesa`,
   `sandstone_valley`, `desert_canyon` on exposed sedimentary rock in dry climates; `stony_spires`
   on humid carbonate. Outside their ground the veto of fix 1 applies; on their ground Deep Time may
   prefer them (as for volcanoes). This is where Terralith beats every alternative: its realistic
   biomes are already named for the geology Deep Time simulates. Cost **M** (a table, the rock and
   province lookups exist, census checks per biome).
4. **Peatland** (cold wet lowland; 66 % of Dfc land on one test planet went to swamp). Fix, already
   half-decided ("taiga instead, peatland if present"): a tag `#deeptime:peatland` with optional
   entries for the known bogs (`regions_unexplored:fen`, `biomesoplenty:bog`, `biomesoplenty:muskeg`,
   `wythers:fen`, `wythers:snowy_bog`, `clifftree:bog`); the wetland rule
   asks the biome source at the wetland coordinates and keeps the answer only if it is in the tag,
   else uses taiga. MIC has no member today (Terralith has none); whether MIC gets one is Q3.
   Cost **S** as part of the wetland task.
5. **Salt flats and playas** (decided: surface feature). Fix: on playa floors (the hydrology already
   shrinks lakes to salt flats), a crust of a `#deeptime:salt_crust` block (first installed of a
   modded salt block, else calcite or white concrete powder) with no vegetation. Cost **S–M**.
6. **Lava fields.** Fix: young lava flows and flood-basalt surfaces get bare basalt, smooth basalt
   and blackstone tops with sparse vegetation for their first few thousand years, as a surface pass
   like the ice cover. Cost **S–M**.
7. **Karst.** Fix: a surface and terrain feature on exposed carbonate: dolines and sinkholes,
   limestone pavement (stone and calcite with moss), cave openings; tower karst by gating
   `stony_spires` (fix 3). Cost **M–L** (it shapes terrain); defer behind the others.
8. **Tropical dry forest (Aw edges) and temperate rainforest (Cfb coasts).** Fix: leave for now.
   No Terralith biome fits; vanilla forest and savanna are acceptable; BOP's woodland covers the dry
   forest where BOP is installed. Revisit only if Ben wants Deep Time biomes (Q1).
9. **Tidal flats.** Fix: leave for now; later a surface feature on low-gradient coasts within the
   preset's tidal range (mud and packed mud, seagrass in the pools). Cost **M** when wanted.
10. **Cloud forest.** Fix: a placement rule for tropical highlands (C classes below 25° latitude,
    high and wet) into Terralith's `cloud_forest` slot (cold band, humidity 3–4, erosion above 0.57,
    far inland), inside the Terralith table of fix 2. Cost **S**, but check the slot's terrain
    assumptions on real mountains first.
11. **Deltas and floodplains.** Fix: let the wetland rule count delta and floodplain cells as wet
    when their river discharge is large, whatever the local rain (the Nile delta is not a desert).
    Cost **S**.
12. **Badlands over real strata.** Fix: check first. If vanilla's and Terralith's badlands surface
    rules paint terracotta bands over Deep Time's strata, restrict badlands biomes to soft
    sedimentary rock (fix 3) so the bands at least sit on sediment. Cost **S** to check.

## 8. Recommended target set

- **Primary: Terralith** (MIC), with Deep Time's veto (fix 1), Terralith-tuned table (fix 2) and
  geology gate (fix 3). This is what Ben leaned to, and the measurements support it.
- **Baseline: vanilla** (standalone), with today's table; it keeps working as the fallback of every
  fix.
- **Kept working: Biomes O' Plenty through TerraBlender**, as now (GameTest profile). Its biomes
  enter through provinces; the veto and peatland tags apply to it too.
- **Supported by tags, not tested: Regions Unexplored, BWG, Nature's Spirit**: they enter through
  the same mechanisms; `#deeptime:peatland`, `#deeptime:salt_crust` and the veto name their ids as
  optional entries.
- **Not supported: Expanded Ecosphere, Woodlands: Extra, CliffTree with Terralith, Wilder Wild** (do
  not reach a Deep Time world, collide with Terralith, or have no NeoForge 1.21.1 build).
- **Surface features over biomes** for salt flats, hot springs (both decided), lava fields and
  karst; **no Deep Time biomes** in this round (Q1).

## 9. Cost estimates

Scale (agent working days, including tests, a GameTest census check, the science page and ledger):
**S** up to 1 day, **M** 2–5 days, **L** 1–3 weeks.

| Fix | Cost | Depends on | What it needs |
|---|---|---|---|
| 1 Veto by tag | S (M alone) | volcanic tie-in | tag, query hook, census check "no vetoed biome on land" |
| 2 Terralith-tuned table | S (1–2 days) | none | overlay with mod-loaded condition, one 16k probe per variant, census floors for MIC |
| 3 Geology gate | M | 1 | gate table (biome → rock class, province class, climate), lookups, per-biome census checks |
| 4 Peatland tag | S | wetland task | tag with optional entries, wetland probe, fallback to taiga |
| 5 Salt crust | S–M | hydrology playas | surface pass, block tag, GameTest |
| 6 Lava fields | S–M | volcano ages | surface pass by age, GameTest |
| 7 Karst | M–L | rock map | terrain shaping, surface pass, gating of `stony_spires` |
| 9 Tidal flats | M | coast and tide data | surface pass on low coasts |
| 10 Cloud forest rule | S | 2 | one table rule, check on mountains |
| 11 Delta and floodplain wetland | S | river discharge | wetland rule change, census |
| 12 Badlands check | S | none | one look in game |
| A Deep Time biome (for example a vanilla-blocked `deeptime:peat_bog`) | M per biome; L for a set of 4–6 | its own placement path | biome JSON (climate, colours, spawns), configured and placed features from vanilla blocks, surface rule entries through the delegate, lang keys, `c:` and `minecraft:` tags (structures, spawns, villagers), placement by Deep Time's own query (it cannot join a pack's parameter list without Lithostitched or TerraBlender), GameTests in all four profiles, Distant Horizons colour check |
| A full Deep Time biome set (every class and setting) | many weeks, plus upkeep | all of the above | not recommended now |

Total for the recommended set (fixes 1–6, 10–12): roughly 2 to 3 weeks of agent time, most of it in
fixes 3, 5 and 6; fixes 1, 2, 4, 10–12 together are under a week.

## 10. Open questions (recommendation first)

1. **Biomes going forward.** (a, recommended) Terralith primary, fixes 1–6 and 10–12, no Deep Time
   biomes this round; (b) as (a) plus one Deep Time biome, `deeptime:peat_bog` (vanilla blocks, M),
   so peatland exists in every pack; (c) a full Deep Time biome set.
2. **Terralith's fantasy biomes in Deep Time worlds.** (a, recommended) veto `#c:is_magical` and
   `#c:is_rare` plus `snowy_badlands` and `emerald_peaks`; (b) keep them (1.35 % of land); (c) veto
   only the Skylands and islands.
3. **Peatland in MIC.** (a, recommended) taiga for now, the `#deeptime:peatland` tag ready, and a
   separate decision on adding Regions Unexplored to MIC (its fen fits, but it also re-skins many
   vanilla biomes and adds 859 block states); (b) add Regions Unexplored to MIC now; (c) build
   `deeptime:peat_bog` (Q1 b).
4. **Geology gate for Terralith's named biomes.** (a, recommended) yes, as one table after the
   volcanic tie-in lands; (b) only the cliffs and Shield; (c) leave Terralith's placement alone.
5. **Stardust Labs' AI clause.** Its licence forbids using its works "to configure, test, debug, or
   augment" an AI or generative system. This project's agents read Terralith's jar and run GameTests
   with it. (a, recommended) Ben asks Stardust Labs for permission (the licence allows individual
   permission) and, until then, agents work from the measured census and the facts already recorded
   rather than reading Terralith's files again; (b) Ben judges the clause does not apply to
   development of a separate mod; (c) stop testing with Terralith.
6. **Expanded Ecosphere.** (a, recommended) not supported; revisit only if Ben wants it in a pack;
   (b) a test of what its dimension override does to a Deep Time world, then a Deep Time-side list
   that places its biomes.

## 11. Method, sources and what was not checked

- **Jars read** (Python `zipfile`, no game launched): Terralith 2.6.2 on the VM
  (`smoke/mic/server/mods`); BOP 21.1.0.14, BWG 2.6.0, Regions Unexplored 0.6.2 (copies from the
  2026-09-27 study); Nature's Spirit 2.2.5, WWOO 2.6.7, Expanded Ecosphere 3.4.7, Geophilic 3.7,
  CliffTree 3.3, Spawn 4.0.8, BloomingNature 1.1.11, Woodlands: Extra 1.3.3, Good Ending, Mystic's
  Biomes, Luminous Overworld, Autumnity, Environmental, Nyctophobia, Biolith (Modrinth CDN,
  2026-10-03), Wilder Wild 4.2.1 for Fabric 1.21.1 (biome list only).
- **Exact band cells** were computed from the parameter lists for Terralith, Expanded Ecosphere and
  CliffTree. For BOP, Nature's Spirit and BWG they were read from the 5×5 tables (temperature ×
  humidity) in their sources, not from a running game; the erosion and weirdness rules of their
  builders were not reproduced, and Nature's Spirit's config switches were taken at their "biome
  enabled" values. Regions Unexplored's placements were read from its default config in the `21.1`
  branch head (2026-08-30), which may differ in detail from the 0.6.2 jar.
- **Measured shares** come from one planet and one seed (section 3); BOP's shares follow the random
  deal of regions to provinces, so they are one draw.
- **Not tested**: what Expanded Ecosphere's dimension override does to a Deep Time world; Spawn's and
  BloomingNature's placement in a Deep Time world; Terralith together with BOP or Regions Unexplored
  in a Deep Time world; Tectonic's snowline shift on Deep Time terrain. Licences not read in full:
  Nature's Spirit's, FrozenBlock's, Let's Do's, BWG's terms beyond "all rights reserved".
- **Modrinth API** (`api.modrinth.com/v2`, 2026-10-03): versions for `game_versions=["1.21.1"]` and
  `loaders=["neoforge"]`, licences, page text; search of 1.21.1 NeoForge worldgen and "biomes"
  projects by downloads, to find newer notable mods (Expanded Ecosphere, Geophilic, Spawn, CliffTree,
  BloomingNature were the notable additions).
- **Sources on GitHub**: Glitchfiend/BiomesOPlenty (`BOPOverworldBiomeBuilder`,
  `BOPSecondaryOverworldBiomeBuilder`, `BOPRareOverworldBiomeBuilder`), Team-Hibiscus/NaturesSpirit
  (`Terra*Parameters`), Potion-Studios/Oh-The-Biomes-Weve-Gone (`BWGTerraBlenderRegion`,
  `BWGBiomeSelectors`), Apollounknowndev/RegionsUnexplored (`RUCommonConfig`, `RURegions`).
- Scratch on the laptop and VM: `deeptime-runs/biome-survey/` (Terralith script and output);
  `deeptime-runs/volcanic-biomes/vb-*-biomemap.json` (the measured biome maps, another task's).
