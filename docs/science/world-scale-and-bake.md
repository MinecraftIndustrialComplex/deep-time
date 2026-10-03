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
| [BAK](ledger/BAK.md) | far terrain | BAK-039 to BAK-042 (4) |

Rows by status: `cited-unverified` 6, `calibrated` 2, `game-choice` 73, `uncited` 12.

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

## Distant Horizons far terrain and Toroidal World's seam

Far terrain (P3-04) gives Distant Horizons its distant LODs from the baked world data instead of
from chunks generated far away. Nothing physical is decided here; the rows (BAK-039 to BAK-042)
are game choices about how far terrain stays the same terrain as the chunks.

- **The seam.** Every far column folds x into the planet's lap [-C/2, C/2), the period at which
  the world data wraps and at which Toroidal World loops the level, so a section west of x = C/2,
  east of it or across it draws what the chunks there have (BAK-042).
- **Toroidal World's guard** (Ben, 2026-10-02: "Mixin around TW's guard"). TW 0.19.3 (MIT) refuses
  every Distant Horizons world generator but DH's own on a level it wraps, from DH 3.3.1 on: TW
  makes the seam seamless by reshaping the level's chunk generator and folding DH's section keys,
  and a foreign generator would not know the seam. Deep Time's generator reads the same wrapped
  world data the chunk generator does, so a mixin into TW's `DhGenerators.requireOwn` lets it
  through, and only it, when the level's TW loop equals the world data's circumference and Z
  does not loop (BAK-041). TW's key folding, retrieval, seam ranking and LOD-tree mixins are not
  touched. The mixin is optional: it applies only when TW and DH are installed and TW's method is
  as checked, and logs whether it applied.
- **How it is checked.** GameTests compare far terrain with generated chunks over the planet and
  on 64 blocks either side of the seam; `tools/smoke/run.py --dh` has Distant Horizons generate
  and store sections beside the spawn and centred on the seam, then compares what it stores with
  far terrain and far terrain with the chunks generated afterwards on 160 blocks either side.
  `tools/client/dh-client-check.sh` joins a real client (software GL on the test machine) and
  checks that its Distant Horizons requests reach the generator on both sides of the seam and
  that the sections arrive in the client's own database. Its first complete run (2026-10-03) did
  not pass: the client joined and DH's networking came up, but it sent no generation request, so
  far terrain reaching a player is not verified yet (STATUS.md build log).
- **What is not covered.** Distant Horizons' `/dh pregen` cannot cross the seam with Toroidal World
  0.19.3: DH files a pending section under its raw position, TW answers it under the folded one, and
  the pregeneration stops (STATUS.md, pending questions). Players' clients do not use it. The
  GameTest server cannot load Distant Horizons (it casts the server to a dedicated one), so the
  GameTests check far terrain without it and the smoke check covers Distant Horizons.
