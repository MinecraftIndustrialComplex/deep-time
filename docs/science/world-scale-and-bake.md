# World scale, planet constants and the bake

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

A Deep Time planet is small. Its circumference is 16,384 or 32,768 blocks, and one block stands for 300 m across and 35 m up (20 m up until 2026-10-02), so the 16k planet is about 4,915 km round, an eighth of Earth's 40,075 km, and everything is stretched 8.6 times vertically (15 times at 20 m). These are game choices, made so that a planet fits in a Minecraft world and a mountain range fits in its build height; they bound how literally any physical number can be taken. This page lists those choices, the planet's constants (gravity, day, tilt, sunlight, ocean share), the timeline and resolutions, and what the bake does when it turns the simulation's metres into blocks.

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

- The projection (SCL-049, equirectangular) is computed in one module per language: Rust
  `deeptime_format::projection`, Java `worlddata/Projection`, the pages' `projection.js`
  (2026-10-02; `docs/design/equal-area-projection.md` §2). Ben reopened the choice of projection
  on 2026-10-02; until he picks, the module is equirectangular and the worlds are what they were.

- The preset's `pole_cap_deg` is read by nothing but validation; the game reads its own value from the world preset file (extraction notes SCL, flag 1).
- A run may be set 4,600 Myr long while the planet's age defaults to 4.57 Gyr (flag 8).

## The vertical scale: 35 m per block (2026-10-02)

**What changed.** `scale.m_per_block_v` is 35 m (it was 20 m): a game choice by Ben (decisions log
2026-10-02, "35 m per block (Recommended)"; ledger SCL-002), made on the measurements of
[the vertical-scale study](../research/notes/vertical-scale-study.md). At 20 m, 300 m across and 20 m
up drew every slope 15 times steeper than it is (SCL-003), and 42 % of a planet's land was steeper
than 45° against 15 % in vanilla 1.21.1 and in the MIC pack's terrain; ordinary hills became walls.
At 35 m the exaggeration is 8.6. One shared default holds it (`deeptime_format::DEFAULT_M_PER_BLOCK_V`,
Java `Manifest.DEFAULT_M_PER_BLOCK_V`), and every built-in preset and the test planet
`planet_world_4k` write 35.0. The simulation runs in metres and is unchanged; only the conversion of
metres to blocks moved, everywhere it happens (the study's §1 lists the places). Two conversions that
assumed 20 m were fixed: the crater visibility threshold is two blocks at the world's scale (IMP-081,
was a fixed 40 m), and the deposits' surface height reads the world's sea level instead of 63.

**What does not scale.** Everything fixed in blocks stays in blocks (study §1.2): the amplifier's
detail cap and character pass, river channels, the thinnest rock unit (1 block), the metamorphic and
plutonic minimum thicknesses (8 and 16 blocks), the build-limit band and its knee (y 575, now 17.9 km
above the sea instead of 10.2 km: ISO-031), caves, aquifers and the lava floor. The world's range
(y −512 to 639 since the floor moved on 2026-10-02) holds 20.1 km below the sea and 20.2 km above it at
35 m (SCL-010; 14.5 km below with the earlier floor at −352).

**Settings tuned against 15×, re-checked** (all kept; the planet in metres is unchanged by keeping
them, and the drawn shapes move the way Ben asked):

| Setting | At 20 m (15×) | At 35 m (8.6×), value kept | To keep the 20 m drawing |
|---|---|---|---|
| `volcanoes.flank_steepness` 0.5 (VOL-024) | a real 6° flank drawn at 38° | drawn at 24°; cones the same in metres, arc spacing still fits | 0.875 |
| `impacts.depth_scale` 0.5 (IMP-130) | a simple bowl 1.5 times as deep as wide | 0.86 times | 0.875 |
| `surface.glacial.valley_width` 3 (GLA-029) | a 2 km trough 600 m deep: 20 blocks wide, 30 deep | 20 wide, 17 deep | 1.7 |
| `volcanoes.flood_basalt_step_m` 80 m (VOL-036) | 4-block steps | 2.3-block steps | 140 m |
| build-limit band 48 blocks, knee y 575 (ISO-031) | knee 10.2 km above the sea | 17.9 km; no Earth-like planet reaches it | unchanged (fixed in blocks) |
| deposit `size_mult_v`, `spacing_mult_v` (per type) | as tuned | ore bodies and seam gaps 43 % shorter in blocks | × 1.75 |

The folds' vertical stretch is built on its own branch as `structure.deformation.true_height_share`
(4/15 of the true height in metres, the share Ben's "4×" was on 20 m blocks): it does not depend on
the vertical scale, and draws structures 2.3 times their 1:1 shape at 35 m.

**Measured** (quick 16k preset, seed 2, test VM, 2026-10-03; each preset baked at 20 m on master
463ee83e and at 35 m on this change; `tools/review/vertical-scale.sh` and `vertical_scale.py stats`,
land within ±75°):

| Planet | Land > 45° | > 60° | > 75° | Hillside p50 / p90 | Relief 256 (p90) | Coast within 8 (p90 / p99) | Land above the sea p50 / p90 / p99 / max, blocks |
|---|---|---|---|---|---|---|---|
| Supercontinent, 1,000 Myr, 20 m | 42.1 % | 22.6 % | 3.9 % | 34.6° / 65.7° | 190 | 24 / 67 | 32 / 117 / 179 / 276 |
| … 35 m | 21.8 % | 7.8 % | 0.72 % | 20.6° / 51.7° | 113 | 12 / 39 | 17 / 66 / 102 / 160 |
| Scattered continents, 500 Myr, 20 m | 25.5 % | 11.3 % | 1.8 % | 20.0° / 55.6° | 162 | 21 / 48 | 28 / 93 / 162 / 250 |
| … 35 m | 11.7 % | 3.8 % | 0.35 % | 11.2° / 40.4° | 95 | 9 / 29 | 16 / 52 / 93 / 151 |
| Vanilla 1.21.1 (study §3) | 15.1 % | 7.0 % | 1.7 % | 14.5° / 41.6° | 115 | 14 / 44 | p99 75, max 137 |
| The MIC pack (study §3) | 14.8 % | 6.4 % | 1.4 % | 12.7° / 41.8° | 137 | 41 / 163 | p99 161, max 209 |

The 35 m bakes are within about one point of the 20 m bakes redrawn at 35 m (20.6 % and 10.6 %
over 45°), as the study's check at 40 m found. Stratovolcanoes (89 to 95 active on land on the supercontinent planet, 43 on the scattered one): the steeper
flank through the summit, between 20 % and 80 % of the height, p50 77° → 68° (supercontinent) and
71° → 60° (scattered); the highest stands 113 to 123 blocks above the sea (198 to 214 at 20 m). The
one visible crater of each planet: 15 → 9 and 11 → 7 blocks deep.

**The deepest ground.** The lowest terrain is y −303 on the supercontinent planet (y −577 at 20 m)
and y −260 on the scattered one (−502); at 35 m no column is below y −336 and two are below −300
(at 20 m 500,227 and 125,646 columns were below the old floor at −352). These bakes ran with the floor at
−352; master has since moved it to −512, which 35 m does not need (STATUS.md, pending question). The sea floor's 1st percentile is
y −164 and −140.

**Rock layers** (`vertical_scale.py strata` and a comparison of the same land columns down to 2 km
below the ground). In the top 2 km a land column holds 4.5 units at 35 m against 6.5 at 20 m on the
supercontinent planet (1,055 columns, −30 %) and 4.0 against 5.3 on the scattered one (1,337 columns, −25 %); the
thin classes go first (mud, sand, saprolite, tuff, ironstone, coal and laterite lose a half to nine
tenths). 14 % (supercontinent) and 11 % (scattered) of the units baked at 20 m were one block thick.
`strata-check` finds the same few grade inversions and slivers, and one new thing: plutonic rock
with other rock under it in 17 % of land columns (0.1 % at 20 m), because the deeper world now
reaches below pluton floors.

**Ore.** Deposit bodies are 43 % shorter in blocks; more survive, because the world holds 14.5 km
below the sea and fewer bodies are dropped as too deep (MVT 50 → 19 dropped on the supercontinent
planet). Sites 505 → 603 and 90 → 117; the median deposit centre is 89 and 50 blocks under the ground
(126 and 60 at 20 m); total ore in deposits 17.9 → 18.4 million blocks (supercontinent) and 5.9 → 3.0
million (scattered); a median porphyry body holds 241 → 155 ore blocks.

**Open questions** (STATUS.md "Pending questions", 2026-10-02): whether rock layers keep their real
thickness or today's thickness in blocks (`scale.feature_scale.strata` 1.75); whether ore bodies keep
today's height in blocks; whether the settings above keep their values; and whether the floor stays at
y −512 or goes back to −352.

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
