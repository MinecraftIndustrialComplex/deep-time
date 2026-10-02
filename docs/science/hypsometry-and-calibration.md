# Hypsometry and calibration targets

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Hypsometry is the distribution of a planet's surface over height: how much is land, how high the land stands, how deep the oceans are. Earth's has two peaks (continents near sea level, ocean floor about 4 km down). The mod's presets are tuned so that an Earth-like run comes out near Earth's figures (about 29 % land, median land height near 430 m, about 10 % of land above 2 km). This page lists every such target, what was tuned to hit it, the Earth figure and where that figure comes from. A calibrated value is honest only if its target is: the targets are ledger rows too.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [HYP](ledger/HYP.md) | hypsometry targets | HYP-001 to HYP-021 (21) |
| [HYP](ledger/HYP.md) | other calibration targets | HYP-022 to HYP-034 (13) |

Rows by status: `cited-unverified` 10, `abstract-only` 1, `calibrated` 18, `game-choice` 1, `uncited` 4.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/ERO.md](extraction-notes/ERO.md).
- Build log: STATUS.md 2026-10-01, "P1-17 hypsometry recalibration"; 2026-09-29 calibration entries.
- Earth relief used: GMT `earth_relief` (SRTM15+, Tozer et al. 2019), at a resolution the documents state three different ways (extraction notes ERO, flag 8).
- Work package in the verification plan: WP-4 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The accepted band for median land height (430 to 640 m) reaches about 50 % above Earth's 427 m.
- Volcano volume is a known contributor to land that stands too high; it was kept as a game choice.
