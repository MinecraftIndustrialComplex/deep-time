# Glacial erosion and landforms

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Where the climate holds ice on land, glaciers grind the ground: they widen river valleys into U-shaped troughs, cut fjords at the coast, hollow out cirques near the peaks and leave till (unsorted debris, lithified as diamictite) where they melt. The mod places ice above the equilibrium line altitude (the height above which snow survives the summer), erodes under it with a sliding law, and has the amplifier draw the troughs and aretes at block scale.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [GLA](ledger/GLA.md) | glacial erosion | GLA-001 to GLA-039 (39) |
| [GLA](ledger/GLA.md) | glacial landforms | GLA-040 to GLA-052 (13) |
| [GLA](ledger/GLA.md) | glacial block-scale shapes | GLA-053 to GLA-066 (14) |
| [GLA](ledger/GLA.md) | ice sheets (`climate/sheet.rs`, off by default; see [climate.md](climate.md)) | GLA-067 to GLA-076 (10) |

Rows by status: `cited-unverified` 25, `from-memory` 1, `calibrated` 1, `game-choice` 29, `uncited` 20.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/AMP.md](extraction-notes/AMP.md).
- Work package in the verification plan: WP-6 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The relation between precipitation and summer temperature at the equilibrium line (Ohmura, Kasser & Funk 1992) was cited from memory. Stage 1 fetched the paper: the coefficients match (P = 645 + 296 T + 9 T^2, 70 glaciers), but Ohmura's T is the free-atmosphere summer temperature at the equilibrium line, and the code extrapolates the fit below its data; 85.5 % of polar-ice-climate cells come out glaciated, some with under 30 mm of precipitation a year.
- The erosion constant is described as giving 10 to 100 mm/yr under fast glaciers, but the fastest cell in the runs is about 1.1 to 1.4 mm/yr (extraction agent's arithmetic, flag 7).
- The erosion constant K_g = 1e-5 is paired with exponent 1; the published pairs read in stage 1 are 1e-4 with exponent 1 and 1e-5 with exponent 2 (GLA-002, PARTIAL).
