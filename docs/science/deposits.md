# Ore deposits, coal and petroleum

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)`. Every row of the
two ledgers was read against its source in the project's source library (package WP-9/10). Of 351
active rows, 77 are `verified`, 37 rest on an abstract, 20 name a source nobody could read in full,
121 are declared game or design choices and 96 have no source. The code was corrected where it
contradicted its sources: the secular clocks (below) were never applied to a real history, so no
banded iron formation could form.

Ledger ids on this page: `DEP-001` to `DEP-205` ([ledger/DEP.md](ledger/DEP.md)) and `ORE-001` to
`ORE-261` ([ledger/ORE.md](ledger/ORE.md)); rows marked `merged` are owned by the row they name.

## What you see in the world

Ore in a Deep Time world comes in two kinds. **Deposits** are bodies of a known size and shape
placed where the simulated history made the conditions for them. A thin **background scatter** of
the same ores fills the rest of the rock at about a tenth of vanilla's amounts. On the two planets
the deposit audit measured, deposits held 85 to 94 % of the coal blocks, 11 to 35 % of the zinc,
3 to 6 % of the gold and 1 % of the copper; the rest is scatter (build log 2026-10-01, "Deposit
audit"; before the stage 2 fixes, when no iron deposit formed).

The deposit families are:

- **Porphyry copper** (copper with molybdenum and gold): a tall shell of copper ore in and around
  a granite-like intrusion, under or near old volcanic arcs where plates converged. A median one
  holds 400 to 450 copper ore blocks. Where erosion has cut into one, a **supergene blanket** of
  richer copper can lie at the old water table, and **placer gold** gravel lies in the rivers
  downstream.
- **Mississippi Valley-type zinc and lead** in limestone and dolostone of platforms near old
  mountain belts, and **sedimentary exhalative (sedex) zinc and lead** in black shale of rift basins.
- **Banded iron formation**: sheets of iron ore in the old stable interiors of continents. Before
  stage 2 none ever formed. Now it can, but the built-in presets set the inherited share to 0
  until the owner decides how much there should be, so a default world still has none (see
  "Open questions").
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
   and the basement then counts as its host where it lies within the top 2 km (ORE-020). How much
   of an interior holds it is the preset setting `deposits.inherited_bif_share` (ORE-261), 0 in the
   built-in presets for now.
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
   truncated; one buried below the build range is dropped; one deeper than the assessment depth
   but inside the build range is placed but not counted. The formation rate is chosen so that the
   expected counted survivors equal N\*.
5. **Positions.** Within a tract, deposits are drawn in clusters (a Thomas process: camps of
   deposits) at least the USGS same-deposit distance apart (2 km for porphyry). Coal fields instead
   tile a basin, touching but not overlapping.
6. **Sizes and grades.** Tonnage is a lognormal through the USGS grade-tonnage percentiles
   (P90, P50, P10); grades are drawn with the published correlations (a Gaussian copula).
7. **Bodies.** Each deposit becomes zoned bodies (porphyry: ore core, phyllic and propylitic halo,
   lithocap, gossan; coal: one body per seam) sized from tonnage and rock density and enlarged by
   the type's size multipliers. In a zone a host block becomes ore with a fixed probability; the
   grade does not reach the blocks (see "Open questions").
8. **Petroleum** follows its own path: Lopatin's time-temperature index (TTI) accumulates in each
   source-rock layer as it is buried and heated; oil is generated between TTI 15 and 160 (Waples
   1980) above 80 °C, migrates up to 50 km to the nearest trap with a seal, and charges it in order.
9. **Playability.** Regional minimums for essential materials can add the smallest plausible
   deposit of the best type; placer gold is moved, at the bake, into the nearest real river channel
   within 30 km as gravel bars.

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
| DEP-040 | Supergene: eligible precipitation | 100 to 1,500 | mm/yr | SIR 2010-5070-B p. 68 (one Andean stage) | `cited-unverified` |
| DEP-041 | Supergene: minimum stable time | 0.5 | Myr | Sillitoe 2005 | `abstract-only` |
| DEP-043 | Supergene enrichment factor | 2 to 3 × | – | SIR 2010-5070-B after Sillitoe 2005 | `abstract-only` |
| DEP-050 | Placer gold tonnage P90 / P50 / P10 | 0.022 / 1.1 / 50 (n 65) | Mt of gravel | Bulletin 1693 model 39a, Fig. 196 | `verified` |
| DEP-051 | Placer gold grade | 0.084 / 0.2 / 0.48 | g/t | same, Fig. 197 | `verified` |
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
| ORE-020 | Inherited BIF in the stable interiors | craton share × setting × the share of the window before the run | 0–1 | plan §8.6 | `game-choice` |
| ORE-261 | Inherited BIF setting (`deposits.inherited_bif_share`) | 0 in the built-in presets (interim) | 0–1 | none; the owner's question | `game-choice` |
| ORE-258 | Earth's age for the clocks | 4.57 | Gyr | as SCL-039 | `uncited` |
| ORE-117 | Coal field area P90 / P50 / P10 | 300 / 1,000 / 3,000 | km² | the project's estimate | `uncited` |
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
- **The clocks are Earth's dates.** They are applied by the planet's age; a planet younger than
  Earth sits earlier in Earth's history. The dates are earliest known occurrences, not physical
  limits.
- **Inherited iron formation everywhere stable.** The plate model does not date continental
  crust, so every stable continental interior is treated as old enough to hold inherited banded
  iron formation in its basement. On Earth only the Archean and Paleoproterozoic cratons do.
- **No banded iron formation is laid down during the window.** A run that spans 2.6 to 1.85 Ga
  opens the clock, but no process lays down iron-formation rock on the shelves, so the host score
  is zero and none forms then.
- **Coal is a seam field, not a mine block.** No source counts minable seam fields with a common
  rule; the coal field's area, density and cumulative thickness are the project's estimates from
  named fields of five basins (re-derived in stage 2).
- **Petroleum is oil only**, generated from one TTI window with a hard 80 °C floor and one province's
  field sizes (the Niger Delta's) used everywhere; there is no gas window, no carrier beds and no
  fill-and-spill.
- **Grade never reaches the blocks.** Ore blocks follow volume and a fixed zone density; the
  richness setting changes the listed grade only (owner question pending since 2026-10-01).

## Game choices

- The deposit definition for coal (one minable seam field) and adjoining, stacked fields (Ben,
  decisions log 2026-09-29).
- The mean density for porphyry and only survivors within the assessment depth counting (Ben,
  decisions log 2026-09-29, "Use the USGS mean density").
- The size multipliers (porphyry 12 × horizontally, placer 12 and 10, MVT 12 and 3, sedex 12 and 4,
  banded iron formation 6 and 4, coal 1 and 20), so bodies are visible at 300 m per block
  (ORE-053, DEP-195, DEP-127 and the type files; agents, build log 2026-09-29 and 2026-10-01).
- Placer gold moved into a real river channel within 30 km at the bake (Ben, 2026-10-01).
- Province endowment strength and size, camp spacings, zoning fractions and zone densities
  (design values per the type files).
- The favourability curves of every type: the concepts are sourced (for example Superior-type iron
  formation on passive-margin shelves, Bekker et al. 2010), the numbers are design.
- Tying the secular clocks to the planet's age (ORE-259; WP-9/10 agent, 2026-10-02).

## Open questions

- **How much banded iron formation.** With inherited iron formation in every stable interior, the
  1,000 Myr run above forms 412 banded iron formation sites (165 counted within 1 km), against 17 porphyries. Whether to keep the whole stable
  interior as permissive, limit it, or wait for a plate model that dates crust is an owner
  question (STATUS.md, "Pending questions").
- **The richness setting** does not reach the blocks (owner question pending since 2026-10-01).
- **Supergene enrichment** is allowed only with at least 100 mm/yr of rain, taken from one stage of
  the central Andes; the same source says enrichment happens from tropical to hyper-arid climates.
  Its minimum stable time, 0.5 Myr, is Sillitoe's floor; his typical minimum is 3 to 9 Myr.
- **Porphyry density and tract size** (above).
- **Coal field size.** The P10 of 3,000 km² is below several real named fields (Gillette 4,150 km²,
  Witbank 5,680 km², Highveld 7,000 km²).
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
  AbramowitzStegun1964, Cooke2005 (author's copy), Bliss1992.
- Abstract only: Singer2005, Singer1993, Singer1995, Singer2008b, Kesler2006, Kesler2008,
  Agterberg1995, Bekker2010, Leach2010b, Garven1985, Garven1984 (part 1), Sillitoe2005,
  SillitoeMcKee1996, Richards2003, Robert2005, Garnett2005, Hoggard2020, Waples1980, Bohacs1997,
  Ryer1980, Hronsky2008, Skirrow2019, McCuaig2010, Sillitoe2010, Seedorff2005.
- Secondary only: Butterworth2016 (a search summary of the abstract).
- Not readable: Root1992, Thomas1949, Lehmann2020, Magoon1994, Dyman2002, Heckel1986, Rasbury1998,
  Hampson1999.
