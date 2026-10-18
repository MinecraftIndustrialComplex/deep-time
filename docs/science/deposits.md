# Ore deposits, coal and petroleum

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)`. Every row of the
two ledgers was read against its source in the project's source library (package WP-9/10). Of 351
active rows, 77 are `verified`, 37 rest on an abstract, 20 name a source nobody could read in full,
121 are declared game or design choices and 96 have no source. The code was corrected where it
contradicted its sources: the secular clocks (below) were never applied to a real history, so no
banded iron formation could form.

Ledger ids on this page: `DEP-001` to `DEP-211` ([ledger/DEP.md](ledger/DEP.md)) and `ORE-001` to
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
   tile a basin, touching but not overlapping.
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
   as a co-product had satisfied the copper essential. The spawn is no longer drawn toward
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
- **Bare rock where it crops out** (DEP-211). Over an exposed cap the soil goes too: every solid
  block from the ground down to the cap's bottom becomes the gossan's rock. Cook and Porter
  describe deposits "represented at surface by heavily iron stained leached capping outcrops", so
  a cap that reaches the ground is rock at the ground. It is a game rule that the whole soil
  profile goes: the sources say nothing of the soil over a cap, and the generator's surface rules
  cover stone with grass and dirt. The cap is placed where the simulation's surface was, and
  the generated terrain differs by a block or three (the amplifier's relief), so "exposed" means
  the ground is no more than four blocks above the cap's top; a cap buried deeper (one cut by
  erosion and then covered, placed under the surface that cut it) is left under its cover.

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
| DEP-211 | Exposed gossan crops out as bare rock | solid blocks from the ground to the cap's bottom become its rock when the ground is at most 4 blocks above its top | blocks | none (game choice, coordinator 2026-10-17) | `game-choice` |
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
region, of copper in every 8,192-block one), so a game world's nearest iron and copper deposit is closer
than these figures; that was not measured.

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
- The gossan: thick as the water table is deep within 2 blocks and 300 m, and bare rock where it
  crops out (coordinator's calls, decisions log 2026-10-17; DEP-209 to DEP-211). The cap's
  thickness follows the sources for leached caps; the two-block floor and the bare outcrop are
  the project's.
- Partial placer shedding (Ben, 2026-10-05, "Reword check + partial shedding"): a porphyry the surface
  only cuts sheds a placer in proportion to the share of its volume eroded (DEP-207), with a floor
  at the smallest recorded placer (DEP-208). On Earth a placer's gold does come from the erosion of
  older lodes, partly eroded ones included, and the placers carry gold of porphyry and epithermal
  origin (Chapman & Mortensen, found in a search, not read); no source gives a quantitative relation
  between a lode's eroded fraction and the placer below it, so the linear law is the project's.
- No spawn selection by essentials (Ben, 2026-10-05; BAK-015).
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
