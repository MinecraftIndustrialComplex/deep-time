# Ore deposits, coal and petroleum

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Ore is placed where the simulated history made the conditions for it, not by random scatter. Each deposit family has a rule built from a source, a driver, a pathway, a trap and preservation (the mineral-systems approach): porphyry copper above subduction zones, MVT and sedex zinc-lead from basin brines, banded iron formation from early oceans, placer gold in rivers below eroded lodes, supergene copper where weathering enriches a porphyry, coal from buried peat, oil and gas from source rock cooked in the oil window and caught in traps. How many deposits form and how large they are come from United States Geological Survey models (deposit densities and grade-tonnage tables). Bodies are then enlarged to be minable at the game's scale, which is a game choice.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [DEP](ledger/DEP.md) | deposit: porphyry Cu | DEP-001 to DEP-034 (34) |
| [DEP](ledger/DEP.md) | deposit: supergene Cu | DEP-035 to DEP-049 (15) |
| [DEP](ledger/DEP.md) | deposit: placer gold | DEP-050 to DEP-061 (12) |
| [DEP](ledger/DEP.md) | deposit: BIF iron | DEP-062 to DEP-073 (12) |
| [DEP](ledger/DEP.md) | deposit: MVT Zn-Pb | DEP-074 to DEP-101 (28) |
| [DEP](ledger/DEP.md) | deposit: sedex Zn-Pb | DEP-102 to DEP-116 (15) |
| [DEP](ledger/DEP.md) | deposit: coal | DEP-117 to DEP-130 (14) |
| [DEP](ledger/DEP.md) | deposit: petroleum | DEP-131 to DEP-144 (14) |
| [DEP](ledger/DEP.md) | deposit density | DEP-145 to DEP-154 (10) |
| [DEP](ledger/DEP.md) | deposit size and grade | DEP-155 to DEP-165 (11) |
| [DEP](ledger/DEP.md) | deposit survival | DEP-166 to DEP-177 (12) |
| [DEP](ledger/DEP.md) | deposit rules | DEP-178 to DEP-191 (14) |
| [DEP](ledger/DEP.md) | deposit playability | DEP-192 to DEP-204 (13) |
| [ORE](ledger/ORE.md) | deposit formation drivers | ORE-001 to ORE-049, ORE-250 to ORE-253, ORE-256 to ORE-257 (55) |
| [ORE](ledger/ORE.md) | deposit geometry | ORE-050 to ORE-076 (27) |
| [ORE](ledger/ORE.md) | deposit clusters and children | ORE-077 to ORE-107 (31) |
| [ORE](ledger/ORE.md) | coal | ORE-108 to ORE-157, ORE-254 (51) |
| [ORE](ledger/ORE.md) | petroleum | ORE-158 to ORE-206, ORE-255 (50) |
| [ORE](ledger/ORE.md) | placer channels | ORE-207 to ORE-221 (15) |
| [ORE](ledger/ORE.md) | spawn and ore blocks | ORE-222 to ORE-249 (28) |

Rows by status: `cited-unverified` 170, `from-memory` 2, `abstract-only` 33, `game-choice` 118, `uncited` 138.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/DEP.md](extraction-notes/DEP.md), [extraction-notes/ORE.md](extraction-notes/ORE.md).
- Research note: `docs/research/notes/ore_genesis_modeling.md`.
- Build log: the deposit entries of 2026-09-28 to 2026-10-01, and "Deposit audit" (2026-10-01).
- Work package in the verification plan: WP-9 and WP-10 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The time windows are documented but not applied on real histories: the banded-iron, oxygen and land-plant clocks are hard-coded, so the limits on BIF, sediment-hosted zinc-lead and coal do nothing (extraction notes ORE, flag 1).
- The richness setting does not reach the blocks (flag 2).
- Coal field sizes and densities are a research agent's own judgement, with basin areas marked "NOT verified".
- The oil-window thresholds (Waples 1980) were taken from an abstract.
- Porphyry density (4 / 15 / 35 per 100,000 km2) counts deposits and prospects per area of exposed permissive rock and falls as tracts grow; the engine applies a flat value to the whole tract. The stage 1 check found the value about right for tracts of the mod's size, by the source's own example. The mean of 17.8 the engine uses is the project's arithmetic, not a number in the source.
- Stage 1 spot checks on the USGS tables (read from the reports): the grade and tonnage percentiles of porphyry copper, banded iron formation, placer gold, MVT and sedex zinc-lead match the tables exactly, and the model numbers cited are right. The Waples oil-window thresholds match the abstract. The Illinois Basin area in the coal derivation (36,800 square miles) is less than half the 80,000 a secondary source gives.
- The 2.02 Ga limit is from Leach et al. 2010 in Economic Geology, a different paper from the USGS report of the same authors and year that the code also cites; and that paper does not support gating MVT deposits on the same clock as sedex.
