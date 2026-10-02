# Strata

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Strata are the layers of rock under the surface, in the order they were laid down, with gaps (unconformities) where erosion removed part of the record. The mod logs every layer the surface stage deposits, compacts and correlates the layers between neighbouring cells so that they run continuously, and bakes them into the columns the game reads. Layer thicknesses are enlarged to be visible at 20 m per block; that is a game choice. The page also covers the rules that mark petroleum traps and ore host rocks in the column, and the checks that judge whether a column looks like real geology.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [STR](ledger/STR.md) | strata | STR-001 to STR-024 (24) |
| [STR](ledger/STR.md) | petroleum traps and host rocks | STR-025 to STR-043 (19) |
| [STR](ledger/STR.md) | strata realism checks | STR-044 to STR-048 (5) |

Rows by status: `cited-unverified` 4, `calibrated` 2, `game-choice` 37, `uncited` 5.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/RCK.md](extraction-notes/RCK.md).
- Work package in the verification plan: WP-7 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- `realism.rs` still says there are no folds, though folds and faults have landed.
