# Geologic provinces

Status of this page: `stage 1, written with P1-10 on 2026-10-02`. The class names, the three oceanic
age bands and the 150 km map resolution were read in a secondary source (the Wikimedia Commons
description of the USGS map); nothing else here is checked against a source. The rules that assign a
cell to a class are this project's own.

Ledger ids on this page: `PRV-001` to `PRV-031` (see [ledger/PRV.md](ledger/PRV.md)); the old
province scheme of the bake is `BAK-010` to `BAK-013` (superseded, see PRV-028).

## What you see in the world

The planet is cut into provinces: areas of one kind of geologic history. On the Create World
timelapse and in the workbench the "Geologic provinces" layer colours them: shield, platform,
orogen, basin, large igneous province, extended crust and, for the sea floor, three age classes.

In the game the same classification decides where the biome mods' regions go, when TerraBlender is
installed (with Biomes O' Plenty, for example). Each province gets one TerraBlender region, so a
mod's biomes fill whole provinces (a mountain belt, a basin, a stretch of old ocean floor) instead
of a patchwork cut by TerraBlender's own noise. On the 16k test planet (500 Myr, scattered start,
seed 1) there are 43 provinces, each at least 50,000 km^2; a province is a few hundred to a few
thousand blocks across. Without TerraBlender (the MIC pack has none) nothing changes in the game.
Which region a province gets is chance (PRV-030); the class does not decide it.

## The model in plain words

1. **Classify each cell.** After the history is run, every cell of the L1 grid gets one class, by
   the first rule that applies (PRV-006 to PRV-020):
   a mafic volcanic pile much thicker than ocean crust carries is a large igneous province;
   oceanic crust is sea floor in three age classes (under 20 Myr, 20 to 65 Myr, over 65 Myr), or,
   where it is an arc on crust thicker than 9 km, an orogen (island arc);
   continental crust in a collision or arc regime, crust a collision touched in the last 500 Myr
   or a range an Andean-type orogeny lifted in the last 50 Myr to over 1,500 m, is an orogen;
   a rift, or crust thinner than 20 km, is extended crust; a cover of 3 km or more of sediment
   (or 1 km on ground that is sinking faster than 0.03 mm/yr) is a basin; ground with under 200 m
   of cover or a crystalline rock at the surface is shield; the rest is platform.
   Continental crust has no formation age in the plate model, so whatever no orogen rule caught is
   taken to be old crust (PRV-020).
2. **Tidy it like a printed map** (the view): a band of extended crust along every quiet coast,
   a majority filter over half the map's 150 km feature size, and patches smaller than that
   feature dropped into the sea floor around them (PRV-014, PRV-021 to PRV-023). The bake uses the
   same tidied classes.
3. **The bake cuts the classes into provinces** (PRV-028): connected regions of one class, with
   regions under `provinces.min_area_km2` joined to the nearest larger one (PRV-029). The ids go
   in the layer `meta/province`; `notes.bake.meta.provinces` lists them and the class shares.
4. **The game gives each province a TerraBlender region** (PRV-030), once, at the first start
   with TerraBlender installed, and keeps it. Two mixins answer biome queries and chunk
   generation with the region of the column's province instead of TerraBlender's noise.

The preset section `provinces` (editor: Advanced, "Biome placement and regions") sets the minimum
area and the three orogen thresholds.

## Equations and constants

No equation; the rules are thresholds on the plate model's fields. The full list with every
value is [ledger/PRV.md](ledger/PRV.md). The ones P1-10 made settings or added:

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| PRV-001 | the nine classes | shield, platform, orogen, basin, LIP, extended crust, oceanic 0-20, 20-65, >65 Ma | class | USGS world geologic provinces map; legend read on its Commons description | `abstract-only` |
| PRV-002, PRV-003 | oceanic age bands | 20 Myr, 65 Myr | Myr | same legend | `abstract-only` |
| PRV-021 | smallest feature of the map | 1.35 degrees (150 km) | degrees of arc | Commons description: "only features approximately 150 km across and greater" | `abstract-only` |
| PRV-011 | collision belt stays an orogen | 500 Myr (`provinces.orogen_max_age_myr`) | Myr | none; chosen so that the Appalachians and Urals count | `uncited` |
| PRV-012 | Andean range is an orogen | lifted within 50 Myr (`andean_max_age_myr`) and over 1,500 m (`andean_min_elevation_m`) | Myr, m | none; set by running the model until Andean orogeny stopped covering whole continents (87 % of continental cells carried it) | `calibrated` |
| PRV-029 | smallest bake province | 50,000 km^2 (`min_area_km2`) | km^2 | none; "a region the size of a small country" | `game-choice` |
| PRV-030 | province to region | stratified deal by TerraBlender weight | - | none; not a physical claim | `game-choice` |

## Earth against the mod

Class shares of the continental area. "Classified" is the rules' classes before the bake absorbs
small regions; the Earth figures are of that kind.

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| cratons (shield + platform + basin) | about 0.71 | CRUST 2.0 crust types by name, land without Antarctica, derived by an earlier sub-agent (method not recorded); the stage 1 spot check, with its own crude name mapping, got 0.75 | 0.431 | A |
| orogens + arcs | about 0.17 (0.12 + 0.05) | same; the spot check got 0.18 | 0.245 | A |
| extended crust + rifts | about 0.07 | same; the spot check got 0.066 | 0.224 | A |
| large igneous provinces | no figure | not looked for | 0.100 | A |
| orogens + arcs, earlier state of the model | | | 0.421 | B |

Run A: `earthlike_quick_16k`, scattered continents, 500 Myr, preset seed, this branch merged with
master 1eac09f (general flexure on), `deeptime provinces` over the last checkpoint
(`tools/review/p1-10-orogen.sh`), 2026-10-02; one planet. Run B: the same preset on the tree of the
first P1-10 commit (before flexure and the folds stage). The model is chaotic and the platform's
maths library differs, so one planet says little; P1-27's ensemble of six runs gave orogen 0.40 and
cratons 0.38 at an earlier state of the plate model.

After the bake absorbs regions under 50,000 km^2, run A holds (of the continents) orogen 0.285,
extended crust 0.220, large igneous province 0.148, shield 0.170, platform 0.153 and basin 0.024;
43 provinces; 340,730 of 524,288 cells changed class by it. Thin belts and the coastal bands go to
their neighbours, so these shares are not the Earth-comparable ones.

**Why the orogen share is high.** Of the 0.421 in run B, 0.125 was the plate model's own active
collision and arc regime (with its 800 km back-arc halo), 0.166 collision belts of the last
500 Myr (the model collided continents about twice as often as the Earth did: about 10 per 100 Myr
against 3 to 4, per the P1-27 notes) and 0.134 Andean ranges. In run A the active regime is 0.118,
belts 0.042 and Andean ranges 0.060, so the share is 0.245: nearer the Earth's 0.17 but still above
it. The part no threshold reaches is the active regime. Tightening the thresholds (orogen 200 Myr,
Andean 25 Myr and 2,500 m) takes run A from 0.245 to 0.185, at the cost of calling the old belts
of the Appalachian type platform or shield. The defaults are unchanged; the question is with Ben
(STATUS pending questions). Large igneous provinces at 0.10 of the continents are probably high
too (the Earth's are a few per cent); no Earth figure has been looked for.

## Simplifications and limits

- The mod's rules are its own. The USGS map's rules for assigning a cell to a class (its basement
  ages, which basins count) were not found. The map calls its orogens "Phanerozoic" and its basins
  "foreland or intracratonic"; the mod's orogen includes active arcs and recent collisions and its
  basin is a sediment-cover test.
- Continental crust has no age in the plate model, so shield and platform are told apart by cover
  and surface rock, not by Precambrian age (PRV-020).
- Regions are 4-neighbour regions on the equirectangular L1 grid, wider in km at the equator than
  near the poles; the minimum area is counted in map kilometres (cell = 16 x 300 m), not true area.
- 50,000 km^2 is 0.65 % of the 16k planet's 7.7 million km^2, a larger share than on the Earth
  (PRV-029).
- The Earth shares are not a published table. They depend on a mapping from CRUST 2.0 type names
  to classes that nobody published.

## Game choices

- The bake's provinces are the USGS classes (Ben, decisions log 2026-09-29, "Yes, for P1-10";
  PRV-028).
- Minimum province area, the stratified deal of provinces to regions, the code for unclassified
  cells (PRV-029 to PRV-031: the P1-10 agent, STATUS build log 2026-10-01 and its follow-ups).
- The view layer's generalisation and speck sizes (PRV-022 to PRV-027: the P1-27 agent).

## Open questions

- Are the orogen thresholds (500 Myr, 50 Myr, 1,500 m) right, or is the share to be brought down
  through the plate model's collision frequency and halo? With Ben.
- Find the USGS map's own rules or a published table of Earth's province shares (WP-12 in
  [claims.md](claims.md)); until then the Earth column is partial.
- Large igneous province share of 0.10: check against a published LIP area.
- A geologist should look at the platform and shield split, which is by cover thickness.

## Sources

[bibliography.md](bibliography.md): `USGSprov` (the USGS map; read in a secondary source, the
Wikimedia Commons file description, 2026-10-02; the image and the USGS page not read),
`Bassin2000` (CRUST 2.0, used for the Earth shares by an earlier agent and in the stage 1 spot
check; the model's type names, not a published province table), `Mooney1998` (CRUST 5.1, context).
