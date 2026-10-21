# Ore deposits, coal and petroleum

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)`. Every row of the
two ledgers was read against its source in the project's source library (package WP-9/10). Of 351
active rows, 77 are `verified`, 37 rest on an abstract, 20 name a source nobody could read in full,
121 are declared game or design choices and 96 have no source. The code was corrected where it
contradicted its sources: the secular clocks (below) were never applied to a real history, so no
banded iron formation could form.

Ledger ids on this page: `DEP-001` to `DEP-213` ([ledger/DEP.md](ledger/DEP.md)) and `ORE-001` to
`ORE-269` ([ledger/ORE.md](ledger/ORE.md)); rows marked `merged` are owned by the row they name.

Changed on 2026-10-02 by the owner's answers on deposits (decisions log 2026-10-02): inherited
banded iron formation lies in a field of basins covering Earth's share of the stable interiors;
deposits are richer, the richness setting reaches the blocks, rich cores carry vanilla's raw-ore
blocks, and the background scatter is thinner; the supergene rain floor is 0 and the coal field
P10 area about 6,000 km². Section "Richness against scatter" has the targets and the measurements.

Changed on 2026-10-16 by the owner's answer on placer blocks (decisions log 2026-10-16): in a pack
without a gold ore for gravel (no Excavated Variants), a placer's gold in a gravel or sand bar is
vanilla's suspicious gravel or sand, brushed out as gold nuggets. Section "Placer gold as panning"
has the rules and the compatibility notes.

Changed on 2026-10-17 by the coordinator's call on the gossan (decisions log 2026-10-17, "gossan
stays at the surface"): the iron cap over an exposed porphyry is as thick as the water table is
deep (at most 300 m, never under two blocks) and crops out as bare ironstone instead of being
dressed in soil. Section "The gossan" has the rules, the sources and the limits.

Changed on 2026-10-17 by the coordinator's call on Lootr (decisions log 2026-10-17): the nuggets
are rolled when the block is placed and the block entity holds them with no loot table, so a
per-player loot mod (Lootr, in MIC) leaves the block alone and each block is brushed once, by
whoever gets there first.

## What you see in the world

Ore in a Deep Time world comes in two kinds. **Deposits** are bodies of a known size and shape
placed where the simulated history made the conditions for them. A thin **background scatter** of
the same ores fills the rest of the rock at about a twentieth of vanilla's coal and iron (the
setting `deposits.scatter_mult`, 0.5 of the scatter rules; a tenth before 2026-10-02). A chunk of
a deposit holds more of its ore than a chunk of a vanilla world and 60 to 400 times the scatter,
and the richest zone of a metal deposit carries **raw-ore blocks** among its ore, as vanilla's
copper and iron ore veins do (`raw_copper_block`, `raw_iron_block`, a pack's own through
`#c:storage_blocks/raw_<metal>`). On the quick 16k planet measured below, deposits hold 98 % of
the coal, 57 % of the zinc, 48 % of the gold, 21 % of the copper and 3 % of the iron (before the
change 96, 23, 14, 7 and 0 %); the rest is scatter.

The deposit families are:

- **Porphyry copper** (copper with molybdenum and gold): a tall shell of copper ore in and around
  a granite-like intrusion, under or near old volcanic arcs where plates converged. A median one
  holds 400 to 450 copper ore blocks. Where erosion has cut into one, a **supergene blanket** of
  richer copper can lie at the old water table, and **placer gold** gravel lies in the rivers
  downstream. Where one reaches the surface it is capped by a **gossan**, a rust-coloured sheet
  of ironstone (Create's ochrum in MIC) that shows as bare rock on the ground, two blocks thick or
  more, with a little gold in it.
- **Mississippi Valley-type zinc and lead** in limestone and dolostone of platforms near old
  mountain belts, and **sedimentary exhalative (sedex) zinc and lead** in black shale of rift basins.
- **Banded iron formation**: sheets of iron ore in a few old basins of the stable continental
  interiors, as Earth's Hamersley, Transvaal and Lake Superior basins. They cover Earth's share of
  the interiors, 1.4 %: on a quick 16k planet that is one basin and usually one large iron
  deposit (see "Inherited iron-formation basins").
- **Coal seam fields**: stacks of coal seams a block or more thick, tens of kilometres across,
  in the sedimentary basins of the time after land plants appeared.
- **Petroleum traps**: oil in anticlines and salt domes above a source rock that was buried deep
  enough. These place no blocks; they feed the Create Diesel Generators oil chunks.
- Impact-related types (nickel, diamond, uranium, hydrothermal copper) are documented on
  [impacts.md](impacts.md).

At 300 m per block a real deposit would be tiny (a median porphyry's 240 Mt of ore is 49 blocks of
rock), so every body is enlarged by a fixed factor per type to be visible and minable.

## The model in plain words

The deposit stage runs once, after the whole history has been simulated. It reads the history as
a series of frames, one every few million years, each holding about 30 fields per cell (arc magma,
convergence rate, uplift, heat flow, deposition, sediment facies, burial temperature, climate, the
host rock in the top 2 km) and three global clocks.

1. **Favourability** (the mineral-systems approach). For each deposit type, each of four
   components (a source of metal, a driver, a pathway, a trap) is the product of simple
   piecewise-linear curves over named fields. The four are multiplied, so a missing process gives
   zero. The result is multiplied by a host-rock score and by the type's clocks. A cell's
   favourability over time is integrated as a hazard: a cell favourable for 100 Myr forms deposits
   throughout them.
2. **Secular clocks** (`deposits/clocks.rs`, rewritten in stage 2). Three switches follow Earth's
   history, dated by the planet's age: sediment-hosted zinc-lead only after 2.02 Ga (ORE-014), coal
   only after land plants, from the base of the Devonian at 419 Ma (ORE-015), and Superior-type
   banded iron formation only from 2.6 to 1.85 Ga (ORE-016). A frame that straddles a date carries
   the share of its time inside the window. The surface stage asks the same land-plant date when it
   decides whether a step laid down peat (coal facies). When a run starts after the iron-formation
   window, the stable continental interiors can inherit banded iron formation in their basement,
   and the basement then counts as its host where it lies within the top 2 km (ORE-020). Not every
   interior: at the run's first step the stable continental interiors are dealt a field of
   basins covering `deposits.inherited_bif_share` of them (5 %, a game choice of the owner's; Earth's share is 1.4 %, ORE-261), each
   about `deposits.bif_basin_km2` (100,000 km², Hamersley's size, ORE-263), coherent ellipses with
   rough edges (ORE-264); the basins ride with their crust and hold iron formation where their
   crust is a stable interior.
3. **Tracts and counts** (the USGS three-part method). Connected cells above a type's favourability
   threshold form permissive tracts. The number of deposits that **survive to the present** is the
   type's deposit density times the permissive area (`N* = ρ/10⁵ · ΣA`, DEP-166). For porphyry
   copper ρ is the mean of the USGS distribution (17.8 per 100,000 km²); for the other types it is
   a single estimate (the P50). Only survivors whose top lies within an assessment depth count
   (1 km, from USGS Circular 1178; 1,800 m for coal, from Circular 891).
4. **Formation and preservation.** The engine forms more deposits than survive, spread over the
   tracts by area, favourability and province endowment, and in time by the hazard. Each formed
   deposit is replayed through the history of its column: buried by sediment, uplifted and eroded.
   A deposit eroded away is lost (a porphyry leaves placer gold downstream); one cut into is
   truncated (and a cut porphyry also sheds placer gold downstream, in proportion to the share of
   it that erosion took: DEP-207); one buried below the build range is dropped; one deeper than the assessment depth
   but inside the build range is placed but not counted. The fate follows the order of events:
   the deepest the surface ever cut towards the body decides it, so a deposit exhumed through and
   buried again is gone, and one cut and then covered is placed under its old erosion surface
   (an unconformity). The column's history is the crust's own, followed frame by frame with the
   plate (2026-10-05, [erosion-and-sediment.md](erosion-and-sediment.md#the-erosion-and-burial-audit-2026-10-05)). The formation rate is chosen so that the
   expected counted survivors equal N\*.
5. **Positions.** Within a tract, deposits are drawn in clusters (a Thomas process: camps of
   deposits) at least the USGS same-deposit distance apart (2 km for porphyry). Coal fields instead
   tile a basin, touching but not overlapping. A type that lies in a bed (MVT, since 2026-10-19) is
   placed only where the column holds a horizon of its host rock, inside that horizon, and never
   on oceanic crust; a placer only on land or on a shelf the sea has drowned since it formed
   (see "Hosts, crust and placers").
6. **Sizes and grades.** Tonnage is a lognormal through the USGS grade-tonnage percentiles
   (P90, P50, P10); grades are drawn with the published correlations (a Gaussian copula).
7. **Bodies.** Each deposit becomes zoned bodies (porphyry: ore core, phyllic and propylitic halo,
   lithocap, gossan; coal: one body per seam) sized from tonnage and rock density and enlarged by
   the type's size multipliers (the gossan's thickness is not a size multiple: see "The gossan"). In a zone a host block becomes ore with the zone's density times
   the richness settings (ORE-262, at most 0.9), and a share of the ore blocks of the richest zone
   is the material's raw-ore block (ORE-266), so the richness reaches the blocks.
8. **Petroleum** follows its own path: Lopatin's time-temperature index (TTI) accumulates in each
   source-rock layer as it is buried and heated; oil is generated between TTI 15 and 160 (Waples
   1980) above 80 °C, migrates up to 50 km to the nearest trap with a seal, and charges it in order.
9. **Playability.** Regional minimums for essential materials can add the smallest plausible
   deposit of the best type; placer gold is moved, at the bake, into the nearest real river channel
   within 30 km as gravel bars. A site counts toward an essential only when the material is its
   primary product (DEP-206): on the real planet of 2026-10-04 a 0.07 Mt impact deposit with copper
   as a co-product had satisfied the copper essential; lead, a co-product of every type that makes
   it, is the one essential that counts co-products (ORE-273). An essential whose material has no
   ore block in the running game (zinc without Create, lead without a lead mod) is left out of the
   world's content and forces nothing. The spawn is no longer drawn toward
   essentials (Ben, 2026-10-05, "I don't believe we need spawn selectoin since we have some passive
   iron gen like vanilla"): it is the dry, flat land nearest (0, 0) (BAK-015).

## The gossan

A porphyry whose top the surface has reached is oxidised: the sulphides it holds, mostly pyrite,
make sulphuric acid that dissolves the copper and carries it down to the water table, and the iron
stays behind as a rust-coloured, porous, hematite, goethite and jarosite rock, the **leached cap**
or gossan (SIR 2010-5070-B pp. 66 to 67, 120). The copper redeposits in a richer blanket at the top
of the water table (the supergene blanket above), so the cap ends where the blanket begins.

What the sources say about how thick it is:

| Source | Statement | Where |
|---|---|---|
| SIR 2010-5070-B (John et al. 2010) | "The thickness of leached capping is highly variable and ranges from 0 to several hundred meters in most PCDs" | p. 4, again at pp. 64 to 65 |
| same | supergene-enriched zones "can range from a few meters to more than 200 meters in thickness and generally have a hematitic leached cap as much as tens of meters in thickness" | p. 67 |
| same | the blanket forms "at the top of the water table", and leaching needs the drawdown of the water table to outrun erosion | pp. 120 to 121 |
| Cook and Porter 2005 | Inspiration, Arizona: the "remnant leached cap varied from 0 to 300 m in thickness" | PDF p. 19 |
| same | Tyrone, New Mexico: a blanket "from a few metres to more than 100 m in thickness" under "a comparable remnant thickness of leached capping (60 to 150 m thick)" | PDF p. 26 |
| same | Silver Bell, El Tiro pit: a "60 m thick" blanket "overlain by a 30 m thick remnant leached capping"; older "hematite capping exposed on ridges and slopes" | PDF p. 14 |
| same | a deposit "represented at surface by heavily iron stained leached capping outcrops" | PDF p. 24 |

So a cap is tens of metres where the table is shallow and the climate wet, and up to a few hundred
where it is deep and dry. The model takes the cap's thickness from the column's own water table
(DEP-209): the surface stage's `deeptime:water_table_depth_m`, which is 5 m plus 150 m times
(1 − min(1, precipitation over potential evaporation)) plus 5 % of the local relief, so a humid
valley has a thin cap and an arid plateau a thick one; the supergene child already reads the same
column. It is at most 300 m (Inspiration's largest, and the data file's placeholder for "several
hundred", DEP-205). At 35 m per block that is 2 to 9 blocks.

Two rules are the project's, not the sources' (coordinator's calls, decisions log 2026-10-17):

- **Never under two blocks** (DEP-210). The cap was 40 m, 1.1 blocks at 35 m per block, a single
  sheet. The soil rule (grass and three blocks of dirt) painted over it, so a probe found ironstone
  in the body and the player never saw any. Two blocks is the least that reads as an outcrop.
- **Bare rock where it crops out** (DEP-211). Over an exposed cap the soil goes too: in every
  column of its footprint the top blocks of the ground, as many as the cap is thick (at least two),
  become the gossan's rock. Cook and Porter describe deposits "represented at surface by heavily
  iron stained leached capping outcrops", so a cap that reaches the ground is rock at the ground.
  It is a game rule that the whole soil profile goes: the sources say nothing of the soil over a
  cap, and the generator's surface rules cover stone with grass and dirt. Two things make this a
  rule about the body and not about each column. The body is a flat sheet at the surface the
  simulation had at its centre, while the terrain the generator builds is that surface plus the
  amplifier's relief (13 blocks either way across one footprint in the exit run), so a cap cut to
  the sheet is buried on every rise and hangs over every dip: the cap therefore follows the ground.
  And whether a cap is exposed is decided once, at the body's centre: the generated ground there
  is no more than six blocks above the cap's top. A cap that a younger deposit buried again (placed
  under the surface that cut it) is left under its cover; the exit run measured the gap between
  the cap's top and the ground at the centre as 1 to 3 blocks for the exposed ones and 14 to 37
  for the buried ones, and six is above the first and well under the second. One exception: a
  block inside a placer's body (loose rock that carries ore) keeps its gravel or sand, because the
  ore step puts a placer's gold only into loose blocks and the cap would leave it none.
- **A leached cap holds no copper** (DEP-212, coordinator's call, decisions log 2026-10-18). Leached
  means the copper was dissolved out and carried down to the supergene blanket at the top of the
  water table (SIR 2010-5070-B p. 67, p. 120). Silver Bell's 30 m remnant leached capping
  "averaged 300 ppm Cu" (Cook and Porter 2005, PDF p. 14), about 0.03 % against the porphyry's
  median grade of 0.44 % Cu (DEP-002): a cap is depleted, and the game reads depleted as none. The
  ore step therefore places none of the porphyry's ore in the cap's blocks. It matters in the pack
  with Excavated Variants, where the ore block is named for its host rock: the skin repainted the
  ground over the ore body, the ore step took the painted block for host rock and put the copper
  block of that rock (the red-sandstone copper, where the pack's ironstone is the fallback red
  sandstone) into a cap that should hold none. Two kinds of ore stay, because leaching removes copper and not these: the
  gossan's own gold (its `gossan_gold` zone, density 0.02, the data file's "carries gold": an
  iron cap over a gold-bearing system is where prospectors looked for gold) and a placer's gold
  (its gravel is not the cap; the skin leaves it alone). Only the cap's blocks are ore-free, not
  the whole gossan body: the body's cells below the cap, where the supergene blanket forms, are
  drawn as before.
- **The cap is named by a window, not read back** (DEP-213). The ore step cannot read the skin's
  blocks, because the carvers and the features before it have already changed the column the skin
  scanned (a cave mouth under a cap, a ravine cut through it). The rule is a pure window of the
  column instead: from the ground the terrain model gives (the noise cells' interpolated surface,
  the one the water pass reads as the column's ground), `ground + 3` down to
  `ground - thickness - 2`, three blocks wider than the cap each way. The skin hangs the cap from
  the ground it finds, held to the model's ground within three blocks, so every block it repaints
  is inside the window. The window over-excludes (a block three blocks above the cap's top is
  ore-free too, wet or dry), which costs a thin skin of the porphyry's ore and is the price of a
  rule that needs no block read. In the test world the model agreed with the generated ground to
  within three blocks in all 1,631 capped columns, so the hold never acted.

## Equations and constants

The main constants by family; the ledgers hold every row.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| DEP-001 | Porphyry Cu tonnage P90 / P50 / P10 | 33 / 240 / 1,500 (n 422) | Mt | Singer, Berger & Moring 2008, OFR 2008-1155, Table 1 | `verified` |
| DEP-002 | Porphyry Cu grade P90 / P50 / P10 | 0.24 / 0.44 / 0.75 | % Cu | same, Table 1 | `verified` |
| DEP-007 | Correlation of tonnage and gold grade | r = −0.28 (n 256) | – | same, text p. 8 | `verified` |
| DEP-011 | Porphyry deposit density P90 / P50 / P10 | 4 / 15 / 35 | per 100,000 km² of exposed permissive rock | Singer, Berger, Menzie & Berger 2005 | `abstract-only` |
| DEP-147 | Mean of that density (split lognormal) | 17.8 | per 100,000 km² | the project's arithmetic on the three percentiles | `verified` (arithmetic) |
| DEP-013 | Same-deposit distance | 2,000 | m | OFR 2008-1155, rules | `verified` |
| DEP-017 | Porphyry emplacement depth (body centre) | 1,500 | m | OFR 2008-1155 data file, Cu subtype median | `verified` |
| DEP-010 | Porphyry ore density | 2.7 | t/m³ | none (placeholder) | `uncited` |
| DEP-040 | Supergene: eligible precipitation | 0 to 1,500 (the floor was 100 until 2026-10-02) | mm/yr | SIR 2010-5070-B p. 68: enrichment from tropical to hyper-arid climates | `verified` |
| DEP-041 | Supergene: minimum stable time | 0.5 | Myr | Sillitoe 2005 | `abstract-only` |
| DEP-043 | Supergene enrichment factor | 2 to 3 × | – | SIR 2010-5070-B after Sillitoe 2005 | `abstract-only` |
| DEP-050 | Placer gold tonnage P90 / P50 / P10 | 0.022 / 1.1 / 50 (n 65) | Mt of gravel | Bulletin 1693 model 39a, Fig. 196 | `verified` |
| DEP-051 | Placer gold grade | 0.084 / 0.2 / 0.48 | g/t | same, Fig. 197 | `verified` |
| DEP-207 | Placer shed by a cut porphyry | tonnage = size draw × eroded fraction (1 − fraction left); gold cap × the same fraction | Mt of gravel | none found (game choice, Ben 2026-10-05) | `game-choice` |
| DEP-208 | Smallest placer shed by a cut porphyry | dropped below 0.022 Mt of gravel (the model's P90, DEP-050) | Mt | the data's own smallest size | `game-choice` |
| DEP-209 | Gossan (leached cap) thickness | water table depth (`deeptime:water_table_depth_m`), at least the file's 40 m, at most 300 m | m | SIR 2010-5070-B pp. 65, 67, 120; Cook and Porter 2005 (Inspiration 0 to 300 m) | `verified` (the ranges and the water-table link); the mapping from depth to thickness is the project's |
| DEP-210 | Least gossan thickness | 2 blocks (70 m at 35 m per block) | blocks | none (game choice, coordinator 2026-10-17) | `game-choice` |
| DEP-211 | Exposed gossan crops out as bare rock | the top blocks of the ground (as many as the cap is thick, at least 2) in the cap's footprint become its rock when the ground at its centre is at most 6 blocks above its top | blocks | none (game choice, coordinator 2026-10-17; the 6 is the implementing agent's) | `game-choice` |
| DEP-212 | A leached cap holds no ore | no ore of the porphyry's bodies in the cap's window; the gossan's own gold and a placer's gold stay | blocks | SIR 2010-5070-B pp. 67, 120; Cook and Porter 2005 PDF p. 14 (Silver Bell 30 m cap, 300 ppm Cu) | `game-choice` (coordinator 2026-10-18; the sources support a depleted cap) |
| DEP-213 | Cap held to the terrain model's ground | the skin's cap within 3 blocks of the model's ground; the ore-free window 3 blocks beyond the cap each way | blocks | none (game choice, implementing agent 2026-10-18) | `game-choice` |
| DEP-062 | Banded iron formation tonnage | 11 / 170 / 2,400 (n 66) | Mt | Bulletin 1693 model 34a, Fig. 172 | `verified` |
| DEP-063 | Banded iron formation grade | 30 / 53 / 66 | % Fe | same, Fig. 173 | `verified` |
| DEP-065 | Banded iron formation density | 1 / 4 / 12 | per 100,000 km² | none (placeholder) | `uncited` |
| DEP-074 | MVT tonnage (OFR 2009-1252 CAam) | 0.93 / 10 / 86 (n 132) | Mt | Singer, Berger & Moring 2009, Table 2 | `verified` |
| DEP-075 | MVT zinc grade | 1.7 / 5.1 / 12 | % Zn | same | `verified` |
| DEP-080 | MVT density | 1 / 5 / 20 | per 100,000 km² | none (placeholder) | `uncited` |
| DEP-091, ORE-001 | Reach of basin brines and orogen-driven flow | 600 | km | Leach et al. 2010, SIR 2010-5070-A | `verified` |
| DEP-102 | Sedex tonnage (OFR 2009-1252 SHam) | 5 / 25 / 300 (n 25) | Mt | Singer, Berger & Moring 2009, Table 2 | `verified` |
| DEP-107 | Sedex density | 1 / 5 / 12.5 | per 100,000 km² | 1998 USGS assessment tracts, summed by the project | `verified` (data and arithmetic) |
| DEP-154 | Assessment depth | 1,000 (coal 1,800) | m | Circular 1178 printed p. 5; Circular 891 Table 3 | `verified` |
| ORE-014 | Sediment-hosted Zn–Pb after | 2.02 | Ga | Leach et al. 2010, Econ. Geol. | `abstract-only` |
| ORE-015 | Coal-forming land plants after | 0.419 | Ga | Nelsen et al. 2016; ICS chart 2023/09 | `verified` |
| ORE-016 | Banded iron formation window | 2.6 to 1.85 (was 2.65 to 1.85) | Ga | Bekker et al. 2010 | `abstract-only` |
| ORE-020 | Inherited BIF in the basins under a stable interior | basin × craton (frame mean) × the share of the window before the run | 0–1 | plan §8.6 | `game-choice` |
| ORE-261 | Share of the stable interiors that are iron-formation basins (`deposits.inherited_bif_share`) | 0.05 (a game choice, owner 2026-10-02; Earth's sourced share is 0.014; was 0, the interim) | 0–1 | Earth: 0.75 million km² of 2.6–1.8 Ga basins / 53.5 million km² of shields and cratons = 0.014 (see below) | `game-choice` (default); the Earth reference is `verified` (arithmetic) |
| ORE-263 | Typical basin area (`deposits.bif_basin_km2`) | 100,000 | km² | median of seven Earth basins (Hamersley about 100,000, WAEPA1974) | `verified` |
| ORE-264 | Basin field shape | ellipses of elongation 1–3, area lognormal σ 0.5, edges ±0.25 of the radius, none under a quarter basin | – | none | `game-choice` |
| ORE-262 | Richness acts on the blocks | zone density × richness ≤ 0.9; raw share × richness ≤ 0.5 | probability | none | `game-choice` |
| ORE-265 | Background scatter multiplier (`deposits.scatter_mult`) | 0.5 | × the scatter rules | none | `game-choice` |
| ORE-266 | Zone ore densities and raw-ore shares | porphyry core Cu 0.6 (raw 0.05), placer Au 0.25, BIF ore 0.6 (raw 0.04), MVT core Zn 0.65 (raw 0.04), sedex Zn 0.55 (raw 0.03), supergene blanket 0.7 (raw 0.08), coal 0.6 | probability | vanilla OreVeinifier: ore 0.1–0.3 of a vein's solid blocks, raw blocks 0.02 of its ore | `game-choice` |
| ORE-267 | Placer gold is recovered as flakes and nuggets by washing the gravel (the basis of the panning rule) | gold nuggets from washed gravel | – | none read (general knowledge) | `from-memory` |
| ORE-268 | Gold nuggets from one brushed placer block | uniform 2 to 5 (mean 3.5) | nuggets | none (a game count) | `game-choice` |
| ORE-269 | Placer gold in gravel or sand without a gold ore for the rock | suspicious gravel or sand holding the table's roll for its seed, with no loot table (so Lootr leaves it); only over support; only gold | – | Ben, decisions log 2026-10-16; coordinator, 2026-10-17 (Lootr); 1.21.1 sources for the falling-block rule and the roll; Lootr 1.11.38.125 decompiled | `game-choice` |
| ORE-258 | Earth's age for the clocks | 4.57 | Gyr | as SCL-039 | `uncited` |
| ORE-117 | Coal field area P90 / P50 / P10 | 300 / 1,000 / 6,000 (P10 3,000 until 2026-10-02) | km² | the project's estimate; Witbank 5,680, Highveld 7,000 km² | `uncited` |
| ORE-127 | Coal field tonnage P90 / P50 / P10 | 3,870 / 23,000 / 193,000 (was 3,800 / 19,800 / 115,000) | Mt | area × cumulative thickness × 1.32 t/m³, by simulation | `uncited` |
| ORE-119 | Coal field density P90 / P50 / P10 | 4 / 14 / 40 (P50 used) | per 100,000 km² | the project's estimate, re-derived in stage 2 | `uncited` |
| ORE-128 | Cumulative minable coal per field | 5 / 14 / 60 | m | the project's judgement | `uncited` |
| ORE-130 | Density of bituminous coal in the ground | 1.32 | t/m³ | Circular 891, Table 2 | `verified` |
| ORE-108 to ORE-112 | Peat mire: P/PET > 1.2, T > 5 °C, slope < 0.002, deposition 5 to 150 m/Myr | – | – | design values | `uncited` |
| ORE-158, ORE-159 | TTI: each Myr counts 2^⌊(T − 100)/10⌋ | – | Myr | Waples 1980 | `abstract-only` |
| ORE-160 to ORE-162 | Oil generation onset / peak / end | TTI 15 / 75 / 160 | – | Waples 1980 | `abstract-only` |
| ORE-185 | Oil field size, median | 12 MMBO (1.64 Mt) | – | USGS DDS-60, Niger Delta unit 71920101 data form | `verified` |
| ORE-186 | Oil field size P90 / P10 | 2.4 / 89 MMBO | – | the same unit's histogram, read off the figure | `cited-unverified` |
| ORE-207 | Placer transport distance | 5 | km | none | `uncited` |

The survivor formula (DEP-166), as coded:

    N* = count_mult · ρ / 10⁵ · Σ_i A_i
    λ_i = N* · φ_i / Σ_j φ_j · p̄_j,    φ_i = A_i · F̄_i · E_i

`N*` the expected survivors on the planet; `ρ` the type's density (deposits per 100,000 km²);
`A_i` the area of permissive tract i (km²); `F̄_i` its mean favourability; `E_i` its province
endowment; `p̄_i` the chance that a deposit formed in tract i survives within the assessment depth;
`λ_i` the expected number formed in tract i. Then `Σ λ_i p̄_i = N*`.

The tonnage law (DEP-155): `T = exp(μ + σ z)` with `μ = ln P50`, `σ = (ln P50 − ln P90)/1.2816`
below the median and `(ln P10 − ln P50)/1.2816` above it, `z` a standard normal draw.

The clock (ORE-259, ORE-260): Earth's age before present at time `t` (years) of a planet whose age
at the end of the run is `a₀` is `ga(t) = 4.57 − (a₀ + t/10⁹)`; a frame's clock is the share of
its span whose `ga` lies in the window.

## Earth against the mod

Earth's grade-tonnage figures are the USGS models in the table above: the mod draws from them
directly, so a deposit's tonnage and grade match the model by construction. What can differ is
how many deposits a planet has and where.

**Deposits per planet, before and after the stage 2 fixes.** One run each of `earthlike_quick_16k`
with `--start scattered_continents --length-myr 1000` on master 29d493f (before) and on the WP-9/10
branch (after), 2026-10-02, `deeptime run … --threads 4`. Sites are deposits placed in the world;
counted are survivors within the assessment depth; target is N\*.

| Type | Sites before | Sites after | Counted before / after | Target N\* before / after | Permissive area before / after, km² |
|---|---|---|---|---|---|
| porphyry Cu-Mo-Au | 14 | 17 | 9 / 9 | 5.8 / 13.5 (capped: 8.2 expected) | 32,434 / 76,185 |
| supergene Cu blanket (child) | 1 | 2 | – | – | – |
| placer Au (child) | 31 | 97 | – | – | – |
| banded iron formation | **0** | **412** | 0 / 165 | 0 / 97.1 (capped: 41.1 expected) | 0 / 2,428,339 |
| coal seam field | 79 | 53 | 51 / 32 | 59.1 / 38.2 | 422,047 / 272,509 |
| MVT Zn-Pb | 107 | 93 | 40 / 39 | 38.8 / 31.9 | 775,958 / 638,263 |
| sedex Zn-Pb | 2 | 6 | 0 / 0 | 0.24 / 1.01 | 4,705 / 20,223 |
| petroleum trap | 10 | 10 | 10 / 10 | 8.2 / 8.1 | 40,863 / 40,640 |
| impact types (Ni-Cu-PGE, hydrothermal, diamond, basin U) | 0, 22, 1, 17 | 0, 44, 0, 25 | – | – | – |

The two runs are not the same history: the fixes change the strata (no coal facies before the
Devonian), which changes erosion, and the runs are not bit-reproducible. The impact types, which
the fixes do not touch, show the size of that run-to-run variation (hydrothermal sites 22 against
44). Two changes are the fixes' own: **banded iron formation** goes from none to 412 sites (2.4
million km² of stable continental interior becomes permissive, and its target of 97 survivors is
capped because only 2.1 % of what forms survives within 1 km), and **coal** loses the first 581
Myr of the run: its permissive area falls by a third and its target from 59 to 38 fields.

The run above had inherited BIF in every stable interior (the setting at 1). The built-in presets
then shipped `deposits.inherited_bif_share` at 0 (ORE-261; since 2026-10-02 0.05). The same command on the branch with that
default (a8b90db, 2026-10-02) gives: porphyry 8 sites (3 counted, target 5.3, capped), supergene 1,
placer 40, **banded iron formation 0**, coal 76 (54 counted, target 46.0, permissive 328,344 km²),
MVT 87 (35 counted), sedex 2, petroleum 9, impact types 0, 16, 0 and 8. Coal still forms only after
419 Ma; the differences from the two runs above are run-to-run variation.

**Earth figures read in stage 2**, for comparison with what a planet holds:

| Quantity | Earth | Source of the Earth figure |
|---|---|---|
| Porphyry deposit density | 2 to 128 per 100,000 km² of exposed permissive rock; P90 / P50 / P10 4 / 15 / 35; Nevada 15, south-eastern Arizona 71 and 120 | Singer et al. 2005 abstract; Singer 2001, PP 1640-A p. A8 (full text) |
| Porphyries that survive | about 125,895 formed in the Phanerozoic, 47,789 remain (38 %) | Kesler & Wilkinson 2008, abstract |
| Porphyry footprint | ore area median 0.6 km²; hypogene ore generally 1 to 1.5 km tall or less | SIR 2010-5070-B Table E1 and p. 25 (full text) |
| Supergene blanket | enrichment up to 3 × (Andes), about 8 × (south-western North America); enriched zone seldom over 200 m, at most 750 m | SIR 2010-5070-B pp. 25, 65–66 (full text) |
| Placer gold | median 1.1 Mt of gravel at 0.2 g/t, about 220 kg of gold | Bulletin 1693 model 39a (rendered figure) |
| Sediment-hosted Zn–Pb ages | MVT and clastic-dominated first at 2.02 Ga; sedex 1.8–1.4 Ga and 550–300 Ma; more than 80 % of MVT in Phanerozoic rocks | Leach et al. 2010 abstract; SIR 2010-5070-N p. 8; SIR 2010-5070-A p. 6 |
| MVT district | Pine Point: 60 ore bodies in 1,600 km²; ore events 50 to 250 kyr | SIR 2010-5070-A (full text) |
| Iron formation | Superior-type first at ca. 2.6 Ga, giant BIF 2.6–2.4 Ga, gone at ca. 1.85 Ga; primary iron formation 30 to 35 % Fe | Bekker et al. 2010 abstract; OFR 95-831 pp. 210, 254 |
| Coal fields per area | Powder River 21, Sydney 14, Appalachian 13, South Africa 20, Illinois Basin 2.3 named fields per 100,000 km² of coal-bearing area | USGS PP 1625-A, PP 1625-D, PP 1708 D.1, Averitt 1975, Bioregional Assessments, Hancox & Götz 2014 (full texts; counts and areas read, arithmetic the project's) |
| Coal per area | Powder River Wyodak-Anderson about 23 Mt/km²; Illinois about 1.4 to 1.9 | PP 1625-A chap. PN; Averitt 1975 (full text) |
| Oil fields | Niger Delta unit: undiscovered fields 1 / 12 / 1,500 MMBO (min / median / max); world units: about 45 % have a median undiscovered field of 1 to 10 MMBO | USGS DDS-60 data form and chapter AR (full text) |


**Inherited iron-formation basins** (2026-10-02, the owner's "Earth-like patch field"). Earth's
iron-formation basins of 2.6 to 1.8 Ga, read in the source library: Hamersley about 100,000 km²
(WAEPA1974; once about 150,000, MarshSchlinger1980 after Trendall 1973), the Transvaal platform
190,000 preserved of perhaps 600,000 (Beukes1987, abstract; the carbonate platform under the
Kuruman and Penge iron formations), the Lake Superior region 220,000 (Pufahl1996 after Morey 1983;
the region enclosing the iron ranges), Kursk 120,000 and the Quadrilátero Ferrífero 7,000
(Wikipedia), Nabberu 60,000 (Bunting1986) and the Labrador Trough about 50,000 (850 km long and up
to 160 km wide; the project's estimate): about 0.75 million km² (0.71 to 1.39 million with the
original extents). Earth's shields and cratons cover 53.5 million km² (the provinces of
Hasterok et al. 2022, areas computed from the open shapefiles), so the basins are 1.4 % of them
(1.3 to 2.6 %; 1.0 % of the 73 million km² of Archean and Early Proterozoic crust of Abbott et
al. 2000). The basin size is the median of the seven, 100,000 km².

**The default is 5 %, not Earth's 1.4 %** (the owner's choice, 2026-10-02, "5 % of stable interiors").
A 16k planet has about 1/44 of Earth's area, so at Earth's share it holds about one iron-formation
site, often under the sea; at 5 % it holds a handful (about 15 sites, around 6 on land in the run
below). Earth's 1.4 % stays as the reference value in ORE-261; the editor slider sets any share.

Measured on the quick 16k planet (`earthlike_quick_16k`, `scattered_continents`, 1,000 Myr,
`tools/review/deposit-richness-runs.sh`, 2026-10-02): the first step's stable continental
interiors were dealt one basin, 2.0 % of them (the share is met to within one basin). It made 8
BIF deposits in 34 tracts (8,405 km² permissive, where the basement lies within the top 2 km),
of which the planet preserved one (0.9 % of what forms survives 1,000 Myr of cratonic erosion), a
large one: 11,283 iron ore and 305 raw iron blocks, 13 chunks. With the share at 0.05 (two basins)
the same planet holds 15 BIF sites, 6 counted. Before, with every stable interior permissive (the
setting at 1, stage 2), it held 412. On Earth the share holds about 66 deposits (Bulletin 1693
model 34a, n = 66); scaled to the quick planet's 2.4 million km² of stable interior that is about
three, so one is low but of the right order; a 32k planet has four times the area.

## Richness against scatter

The owner's answer of 2026-10-02 ("Rich Deposits, Thinner Scatter, Also make deposits use some of
the vanilla ore vein features"): the richness setting reaches the blocks, deposits are clearly
richer than the background, the background is thinner, and deposit bodies carry vanilla's ore-vein
raw blocks. The deposit audit of 2026-10-01 had found the metal deposits to be noise beside the
scatter: a chunk of a porphyry, placer or MVT body held about half of what a vanilla chunk holds
of its ore, and the planet's copper was 93 % scatter.

**Targets** (set by the agent from what the audit measured):

1. A chunk of a deposit body holds at least as much ore-equivalent (a raw block counts nine, the
   raw ore it crafts into) as a chunk of a vanilla world holds of that ore, and at least 50 times
   the background scatter of a chunk.
2. The background scatter is half of what it was: about 5 % of vanilla's coal and 7 % of its iron
   per chunk (from 10 and 13 %).
3. Raw-ore blocks in the richest zone of each metal deposit, at 2 to 4 times a vanilla vein's share
   (vanilla: 2 % of a vein's ore blocks), and none in placers and coal.
4. The early game keeps its ore: a player still finds a few dozen blocks of coal, copper and iron
   near the surface within a short walk of any point.

**Measured** with `deeptime deposit-audit` (the bake evaluated as the mod's features do; the
access probe is new) on the quick 16k planet, `scattered_continents`, 1,000 Myr, before (master
e50291c7) and after (this change), 2026-10-02. The two runs' histories are the same to the
deposit (the same 6 porphyries, 121 MVT bodies); vanilla figures are the audit's 1,024-chunk
sample of 2026-10-01 (vanilla copper includes the large copper veins under dripstone caves, so it
is a generous yardstick):

| type | ore-equivalent per chunk of body, before → after | × vanilla's chunk | × scatter's chunk | median deposit, ore-equivalent | raw blocks, median site |
|---|---|---|---|---|---|
| porphyry Cu | 151 → 278 | 0.52 → 0.95 (copper 293; 3.6 × vanilla iron's 77) | 20 → 71 | 906 → 1,669 | 0 → 23 |
| placer Au | 10.6 → 32.5 | 0.41 → 1.27 | 10 → 58 | 54 → 153 | – |
| MVT Zn | 39 → 84 | 0.51 → 1.11 | 19 → 80 | 123 → 263 | 0 → 2 |
| sedex Zn | 238 → 444 | 3.1 → 5.8 | 114 → 422 | 2,140 → 3,999 (one site) | 0 → 88 |
| BIF Fe | none → 1,056 | 13.6 | 207 | 13,723 (one site) | 305 |
| coal | 1,951 → 2,018 | 19 → 20 | 189 → 389 | 123,999 → 127,356 | – |

| scatter, per chunk | before | after | share of vanilla after |
|---|---|---|---|
| coal | 10.3 | 5.2 | 5 % |
| iron | 10.1 | 5.1 | 7 % |
| copper | 7.7 | 3.9 | 1 % (of vanilla's generous figure) |
| gold | 1.08 | 0.56 | 2 % |
| zinc | 2.1 | 1.05 | 1 % |

The planet's ore in deposits (ore-equivalent): coal 96 → 98 %, zinc 23 → 57 %, gold 14 → 48 %,
copper 7 → 21 %, iron 0 → 3 % (one BIF deposit). Deposits are rare, so on a planet most copper and
iron stay scatter; where a deposit is, its chunks are worth 50 to 400 of the background's.

**Real blocks.** The same box of 256 chunks (x 5,296 to 5,551, z 1,010 to 1,265: porphyry site 23
and the placer bars of site 22) generated by the game on the standalone set (Deep Time and Create)
before and after, and every block counted (`tools/review/deposit-richness-runs.sh census`):
copper ore in deposit bodies 31,717 → 55,685 plus 870 `raw_copper_block`; gold in bodies 1,477 →
2,022 (the placer's five bars in the box 196 → 658); scattered coal 2,878 → 1,433, iron 2,350 →
1,147, copper 1,781 → 882, gold 262 → 111, zinc 468 → 256. The game places what the audit models
(the audit predicts 898 raw blocks for the whole site).

**Early game.** From 150 random land points, how far a player ranges (the half-width of a square of
chunks around the point) to gather 24 ore-equivalent within 48 blocks under the ground: coal 24 →
40 blocks (median; p90 40 → 56), copper 56 → 72 (72 → 88), iron 56 → 72 (72 → 88). The deposits do
not move these medians (they are too rare); halving the scatter widens the square by about √2. The
essentials (`deeptime:essentials`) still guarantee an exposed iron and coal deposit in every
4,096-block region (no longer within a set distance of the spawn: 2026-10-05). The probe assumes every block in the
square is found, so it compares worlds; it is not a mining time.

**Deposits within reach.** Of the 269 sites of the measured planet 196 lie under the sea (land is
0.197 of it). Counting the sites not under the sea that hold at least 64 ore-equivalent, and
taking the mean distance from a land point to the nearest as 0.5 / √(sites per land area) (an
estimate from the site list, not a probe): coal 26 → 25 deposits, about 430 → 440 blocks; copper
one porphyry before and after, about 2,200 blocks; iron none before and none after (the one
iron-formation site is under the sea; at a basin share of 0.05, 6 land deposits, about 900
blocks). The measured runs carry no essentials (the built-in presets list none; a world made in
the game takes `deeptime:essentials`: a site of iron and of coal in every 4,096-block land
region, of copper in every 8,192-block one, and since 2026-10-20 of zinc and lead in every
8,192-block one, ORE-273), so a game world's nearest iron and copper deposit is closer
than these figures; that was not measured. Zinc and lead were measured with their essentials:
see "The zinc and lead floor" below.

## Placer gold as panning

*Ben, decisions log 2026-10-16: "Could we use suspicious gravel w/ a loot table set to a gold
nugget?" Ledger ORE-267 (`from-memory`), ORE-268 and ORE-269 (`game-choice`).*

On Earth a placer's gold is recovered by washing the gravel: the gold is much denser than the
sand and gravel it lies in, so panning and sluicing leave it behind as flakes and small nuggets
(ORE-267, from memory: no source for this was opened in this work). In a Deep Time world the
placer's gold in a **gravel** bar is therefore, where the pack has no gold ore for gravel,
vanilla's `minecraft:suspicious_gravel` (and in a **sand** cell `minecraft:suspicious_sand`), and
brushing it is the panning. Deep Time ships no block of its own: each placer cell holds the gold
nuggets that the loot table `deeptime:archaeology/placer_gold` gives it, one stack of 2 to 5
(uniform, mean 3.5, about 0.39 of an ingot; ORE-268, a game choice), rolled with a seed that is a
hash of the world seed and the block's position, so the same world gives the same nuggets for the
same brush. Deep Time rolls the table when it places the block, with the seed and the parameters
vanilla's brushing would use, and the block entity holds the nuggets as its item and no loot
table. A median placer of about 150 ore blocks therefore holds about 500 nuggets, about 55 ingots,
if every block is brushed.

The rules (ORE-269):

- **Only where the pack has no gold ore for the rock.** Where Excavated Variants has a gold ore in
  gravel or sand (MIC's `deep-time` pack branch; MIC's master pins no Excavated Variants on
  2026-10-17, so MIC pans), or a pack lists a gold block in `#c:ores/gold` and
  `#c:ores_in_ground/gravel` (or `/sand`), that ore is placed as before and nothing changes.
- **Only gold, only in gravel and sand cells.** Silver, the placer's by-product zone, keeps the
  ore block. A cell that is not gravel or sand (the soil the surface rules left over the bar, a
  rock bench) keeps today's ore block (a stone ore in soil), and the raw-ore draw of a rich core
  still comes first.
- **On support.** Suspicious gravel and sand fall like plain gravel and sand when the block below
  is air, fire, a liquid or a replaceable plant. A cell with such a block under it keeps the plain
  host block and no gold; so a bar's overhanging edge, or a bar over a cave, holds less gold than
  its ore draw said. Bars under water stand on the bed and are supported.
- **One brushing, for whoever gets there first** (coordinator's call, decisions log 2026-10-17).
  A placer is a finite natural deposit. Lootr (in MIC; version 1.11.38.125 read decompiled)
  replaces every block in `#lootr:convert/gravels` or `/sands` (vanilla's suspicious gravel and
  sand) whose brushable block entity has a loot table with its own block, which every player can
  brush once, as soon as the block's chunk and the 24 around it are loaded; it skips a block
  entity without a loot table, which is why the gold is rolled at placement. Nothing in the pack's
  Lootr config is needed. The GameTest server enables every datapack it finds, Lootr's opt-in
  `lootr_no_suspicious_blocks` too, which empties both tags, so there Lootr converts no suspicious
  block at all; `placerBarHoldsPannedGold` shows with a chest control that Lootr ran on the bar's
  chunks, and checks the opt-out itself as the absence of a loot table on every placer block. A MIC
  world has that pack off: a feature pack is selected only when a world or server asks for it, and
  MIC asks nowhere. The roll leaves out NeoForge's global loot modifiers (it runs on a
  world-generation thread), which brushing a block with a loot table would apply; a pack's
  modifier that adds to archaeology loot therefore does not reach placer gold.
- **Compatibility for packs without Excavated Variants.** Nothing is needed from the pack: the
  loot table is in the mod's data, and brushing needs only vanilla's brush. A pack that wants
  another gold-bearing gravel adds its block to `#c:ores/gold` and `#c:ores_in_ground/gravel`, or
  overrides `data/deeptime/loot_table/archaeology/placer_gold.json` to change the count (for
  chunks generated after the change: the nuggets are in the block once it is placed). The
  `deeptime-inspect` tool counts ore blocks and does not count suspicious blocks as ore.

## Hosts, crust and placers (carbonate and ore-deposit task, 2026-10-19)

**What was wrong.** The review of run 1a156f22b66 and six planets of master b438d47a
(`earthlike_quick_16k`, 1,000 Myr, supercontinent and scattered continents, seeds 1 to 3,
`deposit-audit`) found three kinds of misplaced deposit:

- **MVT with no carbonate.** Of 1,003 MVT sites, none had limestone or dolostone at its centre
  in the drawn column (630 sat in shale, 157 in sandstone) and 308 lay on oceanic crust. The host
  score read the history's host fields (`deeptime:host/<class>`, the share of the top 2 km at each frame,
  `deposits/rules.rs:222` on master), a time average that a carbonate long since eroded or thinned
  below a block still satisfied; the body was then painted dolostone wherever it formed
  (`geometry.rock_class` in `mvt_zn_pb.json:65`, applied at `geometry.rs:181` and `:485`), and the
  crust test was a thickness curve (20 to 30 km, `mvt_zn_pb.json:21`) that thick oceanic crust passes.
- **Placers at sea.** Of 452 placers, 102 lay on oceanic crust and 190 deeper than the shelf
  (200 m): a placer was spawned where its eroded parent stood, with no test of where that is now
  (`spawn_placer`, `deposits/mod.rs:3051` on master).
- **Impact ore everywhere a crater was.** A median 153 impact-basin uranium sites per planet
  (62 to 359) and 75 hydrothermal Cu-Zn-Au sites (43 to 541). Basin uranium had no uranium
  source: its source term was the crater basin itself (`impact_basin_u.json` on master: source `impact_basin`, driver
  deposition, trap reservoir quality), so a crater on oceanic basalt made uranium as readily as
  one in granite; the hydrothermal type's source was the hydrothermal strength alone. Their
  densities (50 and 100 per 100,000 km² of crater, p50) were placeholders about 10 to 25 times
  Earth's inventory (IMP-143).

**What changed.**

- **In-stratum hosts** (ORE-271, DEP-088, DEP-089). A type with `hosts.in_stratum` lies in a host
  horizon the column really holds. Of the units the world draws (`structure::Cover`, the same
  smoothed stacks as the bake, read through the world's folds and faults as the strata pass reads
  them), those of a host class are candidates, each cut to its blocks under the column's top four
  solid blocks (the soil, which hosts no ore in the game; the first ensemble without this cut put
  most bodies there, a median 0 ore blocks per MVT site), above the floor's last four, and to its
  part at least as old as the ore. MVT ore is epigenetic: its brines replace a carbonate that is
  already there, so a unit laid after the ore formed cannot hold it. The strata log's keys date a
  unit (the log is age-linear), and a unit the ore's age falls inside keeps its blocks below that
  age. Nearest the type's formation depth first, the first candidate is taken that overlaps the
  same unit, in a host class, in each column within two strata cells (32 blocks) of the site,
  each column on its own, so a dipping unit passes. The lateral test is there because the column
  rule interfingers a facies change block by block: a one-block limestone stringer that holds in
  the site's column alone is gone a few blocks away, and a body wider than that sits mostly in
  shale. The body's centre is the unit's middle, its half height at most the unit's (a body the
  cap thins spreads along the unit and keeps its volume, so the ore it draws matches the site's
  tonnage), its dip 0, and no erosion cut is applied to it. The site's centre is the block
  holding the body's centre (rounding had put the centre of a one-block horizon in the block
  above it, and x and z a column off the one the host was read in), projected through that
  block's centre. The bake raises it with the rock at its centre, as it raises every body
  (`key_shift`, STR-056). The projection to Petroff-Guyou moves it with its host horizon instead
  of with the drawn ground: the world's stacks are raised to the drawn ground by the bake's rule
  (the thickest rock unit near the surface takes up the raise and the beds over it rise whole),
  and the body rises by the same `key_shift` on those stacks. It is clipped four blocks under
  the world's final ground (the young landscape and the amplifier run after the
  deposit stage, so that ground can lie lower than the one the stage kept the body under). A
  site with no such horizon places nothing, nor does one on crust that is oceanic now
  (`plates/crust_type`); `survives()` decides the same way as `materialise()`, and the guarantee
  step that forces one deposit of a type skips such cells (an exposed essential's body must still
  reach the exposed band). MVT is the one such type: its hosts are limestone and dolostone
  (dolostone weighted 1.5, after Leach et al. 2010: "More MVT deposits are hosted in dolostone
  rather than limestone"), not chalk (pelagic, no platform) or marble, and the body keeps its host
  rock (no `rock_class`). The census counts the drops (`no_host`, `oceanic`).
- **Placers stay on land and the shelf** (DEP-214). A placer is not placed on oceanic crust, nor
  where the ground lies deeper under the present sea than the sea's lowest stand since the placer
  formed (the surface stage now keeps its sea level per step, `SurfaceState.sea_history`). A bench
  drowned by a later rise, as the gold of the Bering shelf off Nome (to about 18 m of water, ARDF
  NM253; the Last Glacial Maximum's fall was about 130 m, Clark et al. 2009), is kept within that
  range. The parent link is unchanged; the census counts the drops as `offshore`.
- **Impact metals need a source, a fluid and a trap** (IMP-142, IMP-143, ORE-039, ORE-040,
  ORE-042). See [impacts.md](impacts.md#impact-deposits-need-a-source).

**Measured.** The six quick planets of erosion-and-sediment.md "Carbonate on the sea floor"
(master b438d47a against the branch at 686a01cf, `deeptime deposit-audit` with `--soil 4`,
`tools/review/carbonate-table.py`); sites per planet are medians with the range, the rest sums
over the six:

| measure | master | branch |
|---|---|---|
| MVT sites per planet | 159.5 [91, 253] | 5 [0, 23] |
| MVT centred in limestone or dolostone | 0 of 1,003 | 50 of 51 (limestone; one centre block reads shale) |
| MVT on oceanic crust | 308 | 0 |
| MVT ore-body blocks in a host horizon | 0 % | 99.9 % |
| placer gold sites per planet | 75.5 [61, 90] | 65 [41, 90] |
| placers on crust the bake marks oceanic | 102 | 13 |
| placers deeper than the shelf (200 m) | 190 | 3 |
| porphyry Cu-Mo-Au sites per planet | 15 [2, 28] | 24.5 [8, 56] |
| coal seam fields per planet | 71 [62, 94] | 94 [68, 115] |
| petroleum traps per planet | 17 [10, 29] | 15.5 [9, 28] |

MVT now forms only where a limestone horizon runs through the ground about the site, and the
planets have little of it (carbonate is 0.7 % of the land's top blocks), so the count fell about
thirtyfold; no dolostone host was found, because the planets lay almost none. On the full 16k
planet (`earthlike_full_16k`, scattered continents, seed 1) MVT went from 168 sites to 7, all in
limestone, and the deposits' share of the world's zinc from 0.80 to 0.07 (lead from 14,636 ore
blocks to 281): zinc now comes mostly from the pack's scatter. The 13 placers the audit counts on
oceanic crust read the bake's crust-age layer at its nearest sample; 10 of them stand on land
(rivers and lakes 46 to 1,293 m above the sea) and 3 at the coast within 85 m under it, where the
simulation's crust, which the rule reads, is continental. The 3 below the shelf lie on
continental crust where the drawn ground is 7 to 28 blocks under the sea; the rule reads the
simulation cell's height, not the drawn column. Porphyry, coal and petroleum move within the
planets' spread; their rules did not change.

**After the review (2026-10-20).** An Opus review of the branch raised these; each was checked
against the code before it was fixed.

- The placer's depth counted the sea level twice. `deeptime:elevation_m` is already relative to
  the sea of its time, so the depth under the present sea is minus the present elevation
  (DEP-214). A NaN crust type had read as oceanic (`as u8` gives 0); it is now no crust type.
- The lateral host test cut the candidate to the blocks every column of the reach held at once,
  which a dipping unit fails; each column is now asked on its own for the same unit. A host
  younger than the ore was accepted; a unit is now cut to its part at least as old as the ore.
- The projection kept a stratabound body at its y but not under the final ground; it now clips it
  there. A site was projected through its block's corner, which could land a block off; now
  through the block's centre.
- Keeping the body's y through the projection was itself wrong once the ocean crust branch
  (master c24ff793) moved bodies by `key_shift` in the bake: the projection raises the world's
  stacks again to the drawn ground, and where that raise lifted the host the body stayed under
  it. The body now rises by `key_shift` on the raised world stacks (no double lift: the bake's
  lift is applied once, and the projection's shift is only its own raise). A test carries 300
  bodies set in a 6 to 10 block limestone through the bake's lift and crest raise and then the
  projection's raise, and finds every one still in limestone; keeping the y leaves some out.
- The guarantee pass read a candidate's host fit for each essential, with a new deform scratch
  each time; the fits are now cached per type, exposure and cell, with one scratch. An exposed
  essential's fit whose body top lies below the exposed band is no fit, so it stays exposed.
- The vertical cap took volume from a body without taking tonnage from its site; the body now
  widens to keep its volume.
- `stratabound_sites` in the projection asks only the built-in types. Not a defect: the core has
  only built-in types, and the filter reads their `hosts.in_stratum`.

**The zinc and lead floor (2026-10-20).** Once MVT formed only in a real limestone horizon, the
deposits held little of the world's zinc and lead, and a world without SEDEX had no lead at all
(the pack's scatter had zinc, no lead). The coordinator's call was a floor of two parts: a galena
scatter shaped like the zinc scatter (ORE-272: 0.5 clusters of 5 blocks per chunk, mostly in
carbonate and shale), and zinc and lead essentials (ORE-273: a site of each in every 8,192-block
land region, copper's region, not necessarily exposed). Zinc counts primary products only (MVT,
SEDEX, the impact hydrothermal Cu-Zn-Au); lead counts co-products too, because every type that
makes lead makes it as a co-product. The guarantee places the smallest plausible deposit of the
region's most favourable producer, an in-stratum type only where its column holds a host horizon
(ORE-271). A game without Create (zinc) or without a lead mod leaves the essential out of the
world's content (`EssentialsBundle.placeable`, logged), so nothing is forced that could not be
placed.

Measured with the essentials and scatter each build ships (`tools/review/carbonate-ensemble.sh`
on the VM with each build's content bundle, `deeptime deposit-audit` per world,
`tools/review/essentials-table.py <dir> master branch`): master 5dc06974 against the branch at
f546d9bc, six quick planets (`earthlike_quick_16k`, 1,000 Myr, supercontinent and scattered
continents, seeds 1 to 3; medians with the range) and the full 16k planet (scattered continents,
seed 1). Master carries no zinc or lead essential and no lead scatter.

| measure | master, quick | branch, quick | master, full | branch, full |
|---|---|---|---|---|
| zinc ore blocks per land chunk: deposits | 3.35 [2.38, 7.26] | 0.29 [0.001, 1.67] | 4.41 | 0.43 |
| zinc: scatter | 1.09 [1.04, 1.12] | 1.07 [1.04, 1.13] | 1.06 | 1.09 |
| zinc: all | 4.46 [3.42, 8.34] | 1.35 [1.12, 2.80] | 5.47 | 1.51 |
| lead ore blocks per land chunk: deposits | 0.17 [0.09, 0.40] | 0.021 [0, 0.125] | 0.25 | 0.031 |
| lead: scatter | 0 | 0.26 [0.25, 0.30] | 0 | 0.27 |
| lead: all | 0.17 [0.09, 0.40] | 0.30 [0.26, 0.39] | 0.25 | 0.30 |
| sites counting toward zinc (guaranteed) | 330 [146, 607] (0) | 5 [2, 9] (0 to 2) | 299 (0) | 3 (1) |
| sites counting toward lead (guaranteed) | 128 [89, 233] (0) | 3 [2, 9] (0 to 2) | 197 (0) | 2 (1) |
| 8,192-block land regions with a zinc source | 2 of 2 on all six | 2 of 2 on four, 1 of 2 on two | 2 of 2 | 2 of 2 |
| 2,048-block land regions with a zinc source | 11 to 20 of 19 to 21 | 2 to 4 of 20 or 21 | 14 of 19 | 3 of 21 |
| spawn to the nearest zinc source (blocks) | 657 [98, 1,070] | 1,610 [548, 3,310] | 399 | 3,420 |
| spawn to the nearest lead source (blocks) | 776 [98, 1,500] | 1,800 [548, 3,540] | 399 | 3,420 |
| access p50, zinc (blocks; points of 150 beyond 1,536) | 624 [472, 712] (0 to 10) | 1,220 [1,130, 1,290] (11 to 46) | 520 | 1,130 |
| access p50, lead | 816 [696, 1,180] (13 to 53) | 1,340 [1,300, 1,510] (45 to 77) | 776 | 1,530 |
| MVT sites, natural + guaranteed | 133 [98, 242] + 0 | 0 or 1 + 0 or 1 | 206 + 0 | 0 + 1 |
| SEDEX sites, natural + guaranteed | 3 [1, 8] + 0 | 1.5 [0, 9] + 0 or 1 | 1 + 0 | 1 + 0 |

What it says:

- **The deposits' zinc fell about tenfold and their lead about eightfold**, because MVT, which
  made most of both, now forms only in a real limestone horizon (ORE-271) and almost none does on
  these planets: 0 or 1 natural MVT site on the branch against 98 to 242 on master. It is not the
  facies change A8 (f546d9bc): the build before it (b2f5c381) had 0 [0, 4] natural sites on the
  same six planets. Nor is it the review's host rules (A4, A5, 160b5389): built just before them
  (34e7e88e), whose simulation is b2f5c381's to the drawn count (364 and 327 MVT drawn), the
  pairwise overlap and the host age move single sites both ways, supercontinent 2 from 1 site to
  0 and scattered continents 1 from 0 to 1 (the pairwise test passes a dipping unit, the age test
  drops a younger host). The host rule made the fall (159.5 to 5 sites a planet at 686a01cf,
  above); the rest came with master 5dc06974 and the tracer's fixes, which left the planets about
  half the neritic and periplatform carbonate (erosion-and-sediment.md "After the review"), and
  the two are not separated.
- **The floor holds the totals, not the sources.** The scatter is now most of the world's zinc
  (1.07 of 1.35 blocks per land chunk) and nearly all of its lead (0.26 of 0.30); lead per land chunk rose
  (0.17 to 0.30), since the galena scatter outweighs what the deposits lost, and zinc fell to
  about a third. But the sources thinned from hundreds to a handful, so the spawn's nearest zinc
  source went from 657 to 1,610 blocks (the full planet's from 399 to 3,420, its sources from 299
  to 3) and the access probe's median doubled, with up to a third of its points finding no
  zinc's need, and up to half no lead's, within 1,536 blocks.
- **The regions are the simulation's.** The guarantee forced 0 to 2 deposits a planet and left
  every region of its grid with a source, but two of the six quick planets show one of the two
  8,192-block world regions without one. The guarantee counts regions in the simulation's
  Lambert frame; the world is Petroff-Guyou, whose orientation the bake picks later from the final
  land, so a region of the simulation is not a region of the world. Iron shows it too, on master
  as on the branch: its 8 guaranteed sites, one in each of the simulation's 4,096-block regions,
  land in 4 or 5 of the world's 8.
- **A guaranteed SEDEX held no ore** on 3 of the 4 quick planets that forced one, and most
  guaranteed iron bodies held none either (17 of 48 held ore on the branch, 20 of 46 on master). Two
  causes. A guaranteed body was placed against the deposit stage's ground, and the soil's top 4
  blocks hold no ore: SEDEX's 100 m formation depth is under 3 blocks at 35 m a block, and an
  exposed body's top was at the ground, so a thin body lay wholly in the soil. Since 14e4ae7d its
  top lies under the soil (ORE-076). The other cause is outside the deposit stage, and it is the
  larger: the world's ground is not the stage's (the drawn and amplified terrain, and since the
  ocean-crust merge the bake's lift and crest raise, which move every body with its stratigraphic
  key, STR-056), and 31 of the branch's 48 guaranteed iron centres lie above the world's ground.
  On the merged build 869e65e7's quick planet (scattered continents seed 1) all 8 guaranteed iron
  centres lie 2 to 47 blocks above it and the SEDEX 64, and none of the 9 holds ore. With the soil
  floor (14e4ae7d, supercontinent 2 and scattered continents 1), 13 of the 16 guaranteed iron
  centres and the one SEDEX still lie above the ground (the SEDEX 61 blocks), and 3 of the 16 iron
  bodies hold ore: a body the bake lifted out of the ground is beyond the stage's reach.

**The guaranteed deposits in the world as generated (2026-10-20).** The last two findings above
are one problem: the guarantee promised a source of each essential in every land region of the
world, and the world it reasoned about was the deposit stage's, not the one the player walks. A
probe of master 3ecea5b9's quick planet (scattered continents seed 1) read every guaranteed site by its
columns (`deeptime inspect-column` at the centre): of its 8 guaranteed iron bodies 6 lay under the
world's sea and none held ore, and the SEDEX held none either. Three causes, three fixes, and no
change to the world data (the guaranteed sites are known from the event log, the rest are manifest
notes):

- **The regions were the simulation's** (DEP-220). The deposit stage now chooses the world's λc
  itself, with the bake's rule (SCL-087: the four cone points as far from land as possible) on its
  present L1 land, and counts each essential's regions on the world's own frame: a 16k world is
  13,312 by 6,657 blocks, so its 4,096-block regions are 4 by 2 with the last column 1,024 wide and
  the rows split at z = 768, and its 8,192-block regions 2 by 1. The projection draws the world
  with the stage's λc (`notes.bake.projection.lambda_c.chosen_by` = `deposits`) when the preset
  leaves λc to the run.
- **Ties went to the region's edge** (DEP-222). Where every cell of a region scored alike (no
  favourability anywhere, common for iron), the first land cell in index order won: at the
  region's top edge, often the pole or a coast, and the present L1 coast is not the world's, which
  the young landscape and the amplifier redraw. Candidates now come from cells at least 128 blocks
  (two L1 cells at 16k) from the present sea first, any land cell when a region has none, and a
  tie goes to the cell nearest (0, 0) in the world, where the player starts.
- **The body was placed against the stage's ground** (DEP-221, ORE-280). The bake lifts the strata
  and the bodies with them where the final terrain stands above the history's surface (STR-052,
  STR-053, STR-056), and the projection moves each body with the change of the drawn height
  between its Lambert and world columns, so a body set just under the stage's ground ends where
  the bake's arithmetic puts it, most often above the world's ground. A pass after the projection
  now reads the finished folder as the game does and moves each guaranteed site's bodies together,
  by whole blocks, so the top of its ore lies at the top rock block over its centre (the world's
  ground less 4 blocks of soil on dry land, the bed under water) for an exposed essential (iron,
  coal), and no higher than it for the others (a body the history buried deeper stays). An
  in-stratum body (MVT) stays in its host horizon, where the bake and the projection keep it
  (ORE-271), and a placer where `placer_channels` settled it; natural bodies keep their physics.

GUARANTEED_MEASUREMENTS

## Simplifications and limits

- **One density per type, flat over the tract.** Singer et al. 2005 found porphyry density falls
  as the permissive area grows; the engine applies one value to every tract. The value is about
  right for tracts of 40,000 to 80,000 km² and too high for much larger ones.
- **Densities that are placeholders.** No USGS density exists for MVT, banded iron formation or
  petroleum; their 5, 4 and 20 per 100,000 km² are placeholders. The sedex density is the project's
  sum of the 1998 assessment's tract estimates, a lower bound.
- **Proxy grade-tonnage models.** MVT uses the carbonate-hosted amagmatic group of OFR 2009-1252
  (its large end, 86 Mt, is twice the MVT-only model's 42 Mt); sedex uses the shale-hosted
  amagmatic group (25 of 179 sedex deposits); banded iron formation is one model for Superior and
  Algoma types, and its 53 % median iron is enriched ore, not primary iron formation.
- **The gossan is one rock and one thickness rule.** Real caps are hematitic over chalcocite-rich
  protore, goethitic over chalcopyrite and limonitic over pyrite (SIR p. 120); the model has one
  class, ironstone. Its thickness follows the present water table, not the history of the
  water table's descent, which is what actually sets a cap (Cook and Porter 2005 describe
  several cycles of enrichment and leaching at Tyrone), and its base is flat. Caps that no longer exist (eroded after the table fell)
  and caps over a body that erosion only cut are not modelled apart.
- **The clocks are Earth's dates.** They are applied by the planet's age; a planet younger than
  Earth sits earlier in Earth's history. The dates are earliest known occurrences, not physical
  limits.
- **Inherited iron formation in dealt basins.** The plate model does not date continental crust,
  so the basins are dealt at the run's first step over the stable continental interiors of that
  moment and ride with their crust; crust made later holds none. On Earth the basins lie on
  Archean and Paleoproterozoic cratons, which the model does not know apart.
- **No banded iron formation is laid down during the window.** A run that spans 2.6 to 1.85 Ga
  opens the clock, but no process lays down iron-formation rock on the shelves, so the host score
  is zero and none forms then.
- **Coal is a seam field, not a mine block.** No source counts minable seam fields with a common
  rule; the coal field's area, density and cumulative thickness are the project's estimates from
  named fields of five basins (re-derived in stage 2).
- **Petroleum is oil only**, generated from one TTI window with a hard 80 °C floor and one province's
  field sizes (the Niger Delta's) used everywhere; there is no gas window, no carrier beds and no
  fill-and-spill.
- **Grade and blocks.** Ore blocks follow volume and the zone density times the richness; the
  site's grade is listed for prospecting and does not set the density itself (a 0.4 % copper
  porphyry and a 0.2 g/t gold placer both place a game's worth of ore).
- **The coal footprint.** The engine scales a field's three axes with the cube root of its
  tonnage, so the larger tonnage P10 of 2026-10-02 gives a footprint P10 of about 4,560 km², not
  the 6,000 km² of the area distribution.
- **A guaranteed body is anchored at its centre column.** The anchor moves the whole body by one
  vertical shift read over the site's centre; elsewhere the body keeps its shape, not the ground's,
  so on a slope one edge of a wide sheet can lie in the soil and the other under more cover.
- **Natural sources are judged at the stage's ground.** A natural site counts toward an exposed
  essential when its top lies within 16 blocks of the deposit stage's surface, not the world's,
  and only guaranteed bodies are moved; the audit's `_with_ore` counts say what the world holds.
- **The world's regions are not all alike.** A 16k Petroff-Guyou world is 13,312 blocks round, so
  its last column of 4,096-block regions is 1,024 blocks wide and its last 8,192-block column 5,120;
  each still needs its own source.

## Game choices

- The deposit definition for coal (one minable seam field) and adjoining, stacked fields (Ben,
  decisions log 2026-09-29).
- The mean density for porphyry and only survivors within the assessment depth counting (Ben,
  decisions log 2026-09-29, "Use the USGS mean density").
- The size multipliers (porphyry 12 × horizontally, placer 12 and 17.5, MVT 12 and 5.25, sedex 12 and
  7, banded iron formation 6 and 7, coal 1 and 35), so bodies are visible at 300 m per block
  (ORE-053, ORE-055, DEP-195, DEP-127 and the type files; agents, build log 2026-09-29 and
  2026-10-01). The vertical multipliers and coal's seam spacing (`spacing_mult_v` 10.5, ORE-145)
  are 1.75 times the values tuned at 20 m per block, so at 35 m per block bodies keep the height
  in blocks they had (Ben, decisions log 2026-10-03, "Keep today's block height").
- Placer gold moved into a real river channel within 30 km at the bake (Ben, 2026-10-01).
- The gossan: thick as the water table is deep within 2 blocks and 300 m, bare rock where it
  crops out, and holding none of the porphyry's ore (coordinator's calls, decisions log 2026-10-17
  and 2026-10-18; DEP-209 to DEP-213). The cap's thickness and its being leached follow the
  sources for leached caps; the two-block floor, the bare outcrop and "none" for "depleted" are
  the project's.
- Partial placer shedding (Ben, 2026-10-05, "Reword check + partial shedding"): a porphyry the surface
  only cuts sheds a placer in proportion to the share of its volume eroded (DEP-207), with a floor
  at the smallest recorded placer (DEP-208). On Earth a placer's gold does come from the erosion of
  older lodes, partly eroded ones included, and the placers carry gold of porphyry and epithermal
  origin (Chapman & Mortensen, found in a search, not read); no source gives a quantitative relation
  between a lode's eroded fraction and the placer below it, so the linear law is the project's.
- No spawn selection by essentials (Ben, 2026-10-05; BAK-015).
- The essentials guarantee in the world as generated: regions on the world's own frame with λc
  chosen at the deposit stage (DEP-220), candidates 128 blocks inland first and nearest (0, 0) on a
  tie (DEP-222), and guaranteed bodies anchored after the projection at the top rock block (DEP-221,
  ORE-280) (coordinator's calls under Ben's rule of 2026-10-16, decisions log 2026-10-20).
- Placer gold as panning in packs without Excavated Variants: suspicious gravel or sand holding
  2 to 5 gold nuggets rolled from a loot table, only gold, only over support (Ben, 2026-10-16:
  ORE-267 to ORE-269; the block is Ben's, the count and the rules around it are the agent's);
  rolled at placement with no loot table left, so one brushing per block and no per-player loot
  under Lootr (coordinator's call, decisions log 2026-10-17).
- Richer deposits, thinner scatter and raw-ore cores (Ben, 2026-10-02: ORE-262, ORE-265, ORE-266;
  the values are the agent's, against the targets above); inherited iron formation in a basin
  field (Ben, 2026-10-02: ORE-261, ORE-263, ORE-264), at 5 % of the stable interiors rather than
  Earth's 1.4 % (Ben, 2026-10-02, "5 % of stable interiors").
- Province endowment strength and size, camp spacings, zoning fractions and zone densities
  (design values per the type files).
- The favourability curves of every type: the concepts are sourced (for example Superior-type iron
  formation on passive-margin shelves, Bekker et al. 2010), the numbers are design.
- Tying the secular clocks to the planet's age (ORE-259; WP-9/10 agent, 2026-10-02).

## Open questions

- **How many iron-formation sites.** At Earth's share (1.4 %) the quick 16k planet holds one BIF
  deposit; the owner's answer expected a handful to a few tens per planet, which a share of about
  5 % gives (15 sites) (STATUS.md, "Pending questions").
- **Supergene minimum stable time**, 0.5 Myr, is Sillitoe's floor; his typical minimum is 3 to
  9 Myr (kept, the owner's answer of 2026-10-02). The rain floor is now 0.
- **Porphyry density and tract size** (above; flat density kept, the owner's answer of 2026-10-02).
- **The MVT clock.** One clock gates MVT and sedex at 2.02 Ga; Leach et al. 2010 say MVT was minor
  until the late Neoproterozoic and SIR 2010-5070-N finds no sedex older than 1.8 Ga.
- **MVT integration time** is 5 Myr, the top of Garven's modelled range; dated ore events last 50
  to 250 kyr.
- **Oil-window thresholds.** TTI 15 / 75 / 160 come from Waples 1980's abstract; the USGS reports
  in the library use TTI 10 for the onset and 160 or 180 for the end.
- **Placer transport** of 5 km has no source; the nearest figures are tin within 8 km and diamonds
  up to 1,000 km (Bulletin 1693).
- Not readable in full: Waples 1980, Bekker et al. 2010, Leach et al. 2010 (Economic Geology),
  Singer et al. 2005, Sillitoe 2005, Kesler & Wilkinson 2006 and 2008, Garven & Freeze 1984
  part 2, Magoon & Dow 1994, Dyman et al. 2002, Heckel 1986, Rasbury et al. 1998.

## Sources

How each was read in stage 2 (full entries in [bibliography.md](bibliography.md)):

- Read in full in the source library: Singer2008a (OFR 2008-1155 and its database), John2010
  (SIR 2010-5070-B), Cox1986 (Bulletin 1693; grade-tonnage figures read on rendered pages),
  Singer2009 (OFR 2009-1252 and its database), Leach2010a (SIR 2010-5070-A), Emsbo2016
  (SIR 2010-5070-N), Cannon1995 (OFR 95-831), NMRA1998data, Schmidt2007 (SIR 2007-5039), Singer2001
  (PP 1640-A), USGS2000a (Circular 1178), Wood1983 (Circular 891), Averitt1975 (Bulletin 1412),
  Ellis1999 (PP 1625-A), Hatch2002 (PP 1625-D), Ruppert2014 (PP 1708 D.1), Trippi2011
  (OFR 2011-1296), SAIMM-coalfields, Hancox2014, BAP2018, Smolinski2014, Nelsen2016, ICS2023,
  Tuttle2000 (DDS-60 data form), USGS2000b (DDS-60 chapters), Moore2003, Nuccio1994, Nuccio2003,
  Dubiel2003, Spencer1988, Higley2005, Newell1997, Acklam, Hormann1993 (preprint), Blackman2021,
  AbramowitzStegun1964, Cooke2005 (author's copy), Bliss1992. Added 2026-10-17 for the gossan:
  CookPorter2005 (open author copy, pp. 207 to 242; the three leached-cap thicknesses above) and
  John2010 pp. 65 to 67 and 120 re-read. Added 2026-10-02 for the
  iron-formation basins: WAEPA1974_HamersleyIronProvince (excerpt), MarshSchlinger1980,
  Pufahl1996, Bunting1986 (excerpts), Abbott2000, Hasterok2022 (areas computed from its open
  data); and MinecraftSources (vanilla's OreVeinifier) for the raw-ore share.
- Abstract only: Singer2005, Singer1993, Singer1995, Singer2008b, Kesler2006, Kesler2008,
  Agterberg1995, Bekker2010, Leach2010b, Garven1985, Garven1984 (part 1), Sillitoe2005,
  SillitoeMcKee1996, Richards2003, Robert2005, Garnett2005, Hoggard2020, Waples1980, Bohacs1997,
  Ryer1980, Hronsky2008, Skirrow2019, McCuaig2010, Sillitoe2010, Seedorff2005.
- Secondary only: Butterworth2016 (a search summary of the abstract); Wikipedia's Kursk Magnetic
  Anomaly and Iron Quadrangle pages and the Quebec survey's Kaniapiskau Supergroup entry
  (2026-10-02). Abstract only (2026-10-02): Beukes1987, Klein2005.
- Not readable: Root1992, Thomas1949, Lehmann2020, Magoon1994, Dyman2002, Heckel1986, Rasbury1998,
  Hampson1999.
