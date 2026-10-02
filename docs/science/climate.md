# Climate

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Climate decides temperature, rain, ice and sea level through the planet's history, and with them erosion, glaciers, coal and the biomes of the finished world. Today's code solves an energy balance (sunlight in, heat out, heat carried poleward) in latitude bands, adds wind and rain belts, rain shadows behind mountains, a carbon cycle with a slowly brightening sun, orbital cycles that pace ice ages, land ice and sea ice. Because it works in bands, climate follows latitude far too closely (the design measured latitude explaining 99.6 % of land temperature variance, against 96.8 % on Earth). **This model is about to be replaced** by a per-cell, seasonal, two-dimensional energy balance with a slab ocean and a moisture budget (`docs/design/p1-26-per-cell-climate.md`, approved 2026-10-01). The ledger rows describe the code as it is today and are marked `pending-rewrite`; constants of the new design are `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [CLM](ledger/CLM.md) | energy balance | CLM-001 to CLM-026, CLM-119 to CLM-134, CLM-138 (43) |
| [CLM](ledger/CLM.md) | insolation and orbit | CLM-027 to CLM-044 (18) |
| [CLM](ledger/CLM.md) | carbon cycle | CLM-045 to CLM-080 (36) |
| [CLM](ledger/CLM.md) | deep-time clock | CLM-081 to CLM-091, CLM-136 to CLM-137 (13) |
| [CLM](ledger/CLM.md) | land ice and sea ice | CLM-092 to CLM-112, CLM-135 (22) |
| [CLM](ledger/CLM.md) | mountain glaciers | CLM-113 to CLM-118 (6) |
| [CLM](ledger/CLM.md) | moisture (design only) | CLM-139 to CLM-146 (8) |
| [CLM](ledger/CLM.md) | ice branch: the flowing-sheet switch, present-day climate, supply-limited and seafloor weathering (merged 2026-10-02) | CLM-147 to CLM-162 (16) |
| [GLA](ledger/GLA.md) | the flowing ice sheet (`climate/sheet.rs`, off by default) and the ice mass balance for the hydrology; on the glacial page | GLA-067 to GLA-076 (10) |
| [MOI](ledger/MOI.md) | wind and rain belts | MOI-001 to MOI-033 (33) |
| [MOI](ledger/MOI.md) | orographic rain | MOI-034 to MOI-050 (17) |
| [MOI](ledger/MOI.md) | lapse rate | MOI-051 to MOI-053 (3) |
| [MOI](ledger/MOI.md) | seasons | MOI-054 to MOI-060 (7) |
| [MOI](ledger/MOI.md) | climate snapshot | MOI-061 to MOI-076 (16) |

Rows by status (CLM and MOI): `cited-unverified` 72, `from-memory` 5, `calibrated` 14, `game-choice` 36, `uncited` 111; `pending-rewrite` 209; `not-yet-coded` 28.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/CLM.md](extraction-notes/CLM.md), [extraction-notes/MOI.md](extraction-notes/MOI.md).
- Design: `docs/design/p1-26-per-cell-climate.md` (its section 2 measures the faults of the banded model; its section 9 lists references, six of them marked "cited from memory").
- Session notes: `docs/research/notes/session-polar-climate-bands.md`.
- Work package in the verification plan: WP-14 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Land ice thickness uses a hard-coded gravity of 9.81 m/s2, so it ignores the planet's gravity setting (extraction notes CLM, flag 1).
- Earth-like runs end near 210 to 220 ppm CO2, not the 280 ppm the carbon cycle is balanced for (flag 7).
- Ice sheets overgrow on the banded climate (47 to 52 % of the land under ice on three 500-Myr Earth-like planets, build log 2026-10-01 "Climate ice branch"), so the flowing sheets (`climate.ice_dynamics`, CLM-147, GLA-067 to GLA-076) are off by default and in every built-in preset (decisions log 2026-10-02, "Merge with ice sheets off"); they are turned on once the per-cell climate (P1-26) fixes the cold bias. See "The ice branch, merged 2026-10-02" below.
- Stage 1 spot checks: the CO2 forcing fit (Byrne & Goldblatt 2014) is stated valid for 200 to 10,000 ppm and is used up to 1,000,000 ppm; a snowball state ends at 0.55 bar of CO2 where Hoffman and others 2017 give 0.01 to 0.1 bar; the code's Earth mean temperature of 15.6 degC is above any long-term mean (Berkeley Earth: 14.1 degC for 1951 to 1980).
- The energy-balance constants are a mix: A, B and D attributed to North, Cahalan & Coakley 1981 (not readable: a scan), albedos from two different climlab classes that climlab does not combine.
