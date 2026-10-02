# Geothermal features

Status of this page: `stub`. The ledger rows were written in stage 2, package WP-0 (2026-10-02); the page is to be written from [_template.md](_template.md) by the package that verifies them.

Hot springs, travertine and sinter terraces and fumaroles (vents of gas and steam, shown as stained ground) are built around active volcanoes. The feature was accepted on 2026-10-01 and is on master (plan item P2-14). The geothermal gradient, the rise of temperature with depth that drives metamorphism and the oil window, is a different thing and is ledgered under rock and metamorphism and under deposits.

What the ledger found when it read the feature (details and row ids in [GEO](ledger/GEO.md)):

- Heat comes only from active volcanoes. The plan said "from the heat flow already modelled"; the code does not read any heat-flow field.
- There are no geysers in the code, the hints or the decisions, and spring water is plain water at no recorded temperature, although this page's earlier stub said "geysers". No Earth count or temperature appears in any source for the feature.
- Several hints and format notes disagree with the code (fields per volcano, pool size, the stained share of a terrace, the height above sea level at which a field may be drawn). These are listed as flags in the ledger file.
- The 400 mm rainfall threshold for a hot spring against a fumarole and the 3 degC rule have no source; the second is a vanilla game rule, not physics.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [GEO](ledger/GEO.md) | Geothermal features | GEO-001 to GEO-051 (51) |

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/SCL.md](extraction-notes/SCL.md).
- Work package in the verification plan: WP-16 ([claims.md](claims.md#verification-plan-for-stage-2)).
