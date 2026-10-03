# World scale, planet constants and the bake

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

A Deep Time planet is small. Its circumference is 16,384 or 32,768 blocks, and one block stands for 300 m across and 20 m up, so the 16k planet is about 4,915 km round, an eighth of Earth's 40,075 km, and everything is stretched 15 times vertically. These are game choices, made so that a planet fits in a Minecraft world and a mountain range fits in its build height; they bound how literally any physical number can be taken. This page lists those choices, the planet's constants (gravity, day, tilt, sunlight, ocean share), the timeline and resolutions, and what the bake does when it turns the simulation's metres into blocks.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [SCL](ledger/SCL.md) | world scale | SCL-001 to SCL-021 (21) |
| [SCL](ledger/SCL.md) | planet constants | SCL-022 to SCL-031 (10) |
| [SCL](ledger/SCL.md) | timeline | SCL-032 to SCL-041 (10) |
| [SCL](ledger/SCL.md) | resolution | SCL-042 to SCL-051 (10) |
| [BAK](ledger/BAK.md) | bake terrain | BAK-001 to BAK-013 (13) |
| [BAK](ledger/BAK.md) | spawn placement | BAK-014 to BAK-016 (3) |
| [BAK](ledger/BAK.md) | satellite view | BAK-017 to BAK-032 (16) |
| [BAK](ledger/BAK.md) | synthetic worlds | BAK-033 to BAK-038 (6) |
| [BAK](ledger/BAK.md) | far terrain | BAK-039 to BAK-040 (2) |

Rows by status: `cited-unverified` 6, `calibrated` 2, `game-choice` 71, `uncited` 12.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/SCL.md](extraction-notes/SCL.md).
- Every key of the default preset with its owning subsystem: `extraction-notes/preset-coverage.md`.
- Work package in the verification plan: WP-12 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The projection (SCL-049) is Lambert's cylindrical equal-area projection, true on the equator
  (Ben, decisions log 2026-10-02: "Lambert for now"); it replaced the equirectangular frame. It is
  computed in one module per language: Rust `deeptime_format::projection`, Java
  `worlddata/Projection`, the pages' `projection.js` (`docs/design/equal-area-projection.md` §2).
  Every cell has the same true area (SCL-052, tested), so map shares are area shares; the grids
  run from z = −Zo, the multiple of 64 past the pole (2,624 at C = 16,384, 1024 × 328 cells at
  L1), and map pictures are π : 1 (1024 × 326). World-data format 0.4.

- The preset's `pole_cap_deg` is read by nothing but validation; the game reads its own value from the world preset file (extraction notes SCL, flag 1).
- A run may be set 4,600 Myr long while the planet's age defaults to 4.57 Gyr (flag 8).
