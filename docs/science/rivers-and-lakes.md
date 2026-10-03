# Rivers, lakes and runoff

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Rain that does not evaporate runs downhill, gathers into rivers and fills hollows as lakes. The mod routes water over the finished terrain, sizes each lake from a water balance of its basin (inflow against evaporation: a lake may be full to its outlet, stand below its rim, shrink to a salt flat, or be dry), freezes lakes in cold climates, and sizes river channels from their discharge with hydraulic-geometry laws (width and depth grow as powers of discharge). Rivers and lakes are far wider in blocks than a true scale would give, so that they exist at 300 m per block; that enlargement is a game choice.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [HYD](ledger/HYD.md) | rivers | HYD-001 to HYD-027 (27) |
| [HYD](ledger/HYD.md) | runoff and evaporation | HYD-028 to HYD-046 (19) |
| [HYD](ledger/HYD.md) | lakes | HYD-047 to HYD-065 (19) |
| [HYD](ledger/HYD.md) | water in the game | HYD-066 to HYD-077 (12) |

Rows by status: `cited-unverified` 14, `from-memory` 3, `calibrated` 1, `game-choice` 37, `uncited` 22.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/HYD.md](extraction-notes/HYD.md).
- Audit of grid artifacts in rivers: `docs/research/notes/grid-artifact-audit.md`.
- Work package in the verification plan: WP-5 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- River sinuosity (channel length over straight-line length) is 1.035 at the median in the audited world; the audit's Earth comparison ("commonly 1.1 to 1.5 and over") was written from memory.
- Leopold & Maddock 1953 was read in stage 1: the exponents 0.5 (width) and 0.4 (depth) are in it, as "representative" downstream values for a discharge of given frequency. The coefficients are the code's own.
- Two runoff laws coexist: hydrology uses the corrected cold-climate law, the young landscape and the erosion stage still use the old one (extraction notes HYD, flag 5).
- Holdridge defines biotemperature with temperatures above 30 degC counted as zero as well as those below 0 degC; the code applies only the lower limit (spot check HYD-030, MISMATCH).
- A lake in the game is a game-side placement of the hydrology's level: every lake column is cut to its bed like a river channel, because the generated ground between noise-cell corners can stand a block or two above the rule's bilinear ground and left one-block-deep lake margins dry (HYD-077). A frozen lake's ice is its surface, not ground, so the snow and ice cover leaves it alone (BIO-084).
