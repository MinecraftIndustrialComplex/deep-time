# Rock classes, metamorphism and intrusions

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Deep Time knows 62 classes of rock: sedimentary (laid down as sediment), igneous (cooled from melt: lavas at the surface, granite-like plutons at depth), metamorphic (changed by heat and pressure) and a few special hosts. Buried rock is heated along a geothermal gradient (temperature rising with depth) and changes grade: shale becomes slate, then phyllite, schist and gneiss. Bodies of magma that freeze underground become plutons. Each class carries a density and how easily it erodes, and each maps to Minecraft blocks, with stone and deepslate standing in for the common ones.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [RCK](ledger/RCK.md) | rock classes | RCK-001 to RCK-027 (27) |
| [RCK](ledger/RCK.md) | metamorphism | RCK-028 to RCK-055 (28) |
| [RCK](ledger/RCK.md) | plutons and intrusions | RCK-056 to RCK-073 (18) |
| [RCK](ledger/RCK.md) | rock to blocks | RCK-074 to RCK-084 (11) |

Rows by status: `cited-unverified` 18, `calibrated` 1, `game-choice` 26, `uncited` 39.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/RCK.md](extraction-notes/RCK.md).
- Rock vocabulary: MIC's rock-tag datapack README (outside this repository).
- Work package in the verification plan: WP-7 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Every erodibility and diffusivity value in the 62 rock-class files is called "judgment, relative to granite"; 41 of the 62 densities have no source (extraction notes RCK, flag 5).
- A stock is coded with a radius of 2 to 10 km while the same comment defines a stock as under 100 km2 (flag 1).
- The pluton thickness law cited to Cruden 1998 is not in that paper's abstract, which says granites 10 to 100 km wide are at most 3 km thick; the code gives 4.1 km at 25 km and 8.3 km at 80 km (RCK-061, MISMATCH at abstract level).
