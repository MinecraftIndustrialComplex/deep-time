# Köppen classes and biomes

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The finished climate is sorted into Köppen classes, a standard scheme that names climates from monthly temperature and rain (for example Af, tropical rainforest; BWh, hot desert; ET, tundra). Each class, with altitude and wetness, is then mapped to Minecraft biomes, including those of biome mods. The Köppen thresholds are a scientific claim and are ledgered as such; the mapping from class to biome is a game choice, and every constant in it is configurable. The page also covers ice and snow in the game and the climate interface other mods read.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [BIO](ledger/BIO.md) | koppen classification | BIO-001 to BIO-020 (20) |
| [BIO](ledger/BIO.md) | biome mapping | BIO-021 to BIO-077 (57) |
| [BIO](ledger/BIO.md) | ice and snow in the game | BIO-078 to BIO-087 (10) |
| [BIO](ledger/BIO.md) | climate API | BIO-088 to BIO-099 (12) |

Rows by status: `cited-unverified` 24, `calibrated` 1, `game-choice` 66, `uncited` 8.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/MOI.md](extraction-notes/MOI.md).
- Stage 1 spot checks: [spot-checks.md](spot-checks.md), section "Moisture, Köppen classes and biomes".
- Work package in the verification plan: WP-11 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Stage 1 read Peel, Finlayson & McMahon 2007, Table 1, in full text. Every coded threshold matches it except the order of tests: Peel says "the B climates must be identified first", and the code tests the polar group E first, so cold dry cells come out as tundra or ice cap where Peel gives desert or steppe (spot check BIO-002, MISMATCH). Where a cell passes both the dry-summer and the dry-winter test, Peel picks by which season is wetter and the code always picks dry-summer (BIO-013).
- The dryness classes are labelled "UNEP"; UNEP 1997 uses 0.03, 0.2, 0.5 and 0.65, and the code's anchors 1, 2 and 5 are its own (BIO-038).
- The mapping from class to biome is a game choice throughout; every constant in it is a setting.
