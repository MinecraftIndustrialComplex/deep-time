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
| [BAK](ledger/BAK.md) | far terrain | BAK-039 to BAK-042, BAK-048, BAK-050 (6) |
| [BAK](ledger/BAK.md) | Distant Horizons curvature | BAK-043 to BAK-047 (5) |

Rows by status: `verified` 2, `cited-unverified` 6, `calibrated` 2, `game-choice` 77, `uncited` 12.

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
  Measured on the quick 16k planet (scattered continents, 500 Myr, seed 1, 2026-10-03, STATUS build
  log): the map's share of land is 0.200 against a true share of 0.201, of ice 0.097 against 0.095,
  and of the band poleward of 60° 0.135 against the sphere's 0.134. On the equirectangular map they
  were 0.146 against 0.192 (land), 0.274 against 0.094 (ice) and 0.334 (the band).

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

**Settings tuned against 15×, re-checked** (all kept but the deposits' vertical multipliers; the
planet in metres is unchanged by keeping them, and the drawn shapes calm down with the world, as Ben
chose on 2026-10-03, "Keep values, calmer shapes"):

| Setting | At 20 m (15×) | At 35 m (8.6×), value kept | To keep the 20 m drawing |
|---|---|---|---|
| `volcanoes.flank_steepness` 0.5 (VOL-024) | a real 6° flank drawn at 38° | drawn at 24°; cones the same in metres, arc spacing still fits | 0.875 |
| `impacts.depth_scale` 0.5 (IMP-130) | a simple bowl 1.5 times as deep as wide | 0.86 times | 0.875 |
| `surface.glacial.valley_width` 3 (GLA-029) | a 2 km trough 600 m deep: 20 blocks wide, 30 deep | 20 wide, 17 deep | 1.7 |
| `volcanoes.flood_basalt_step_m` 80 m (VOL-036) | 4-block steps | 2.3-block steps | 140 m |
| build-limit band 48 blocks, knee y 575 (ISO-031) | knee 10.2 km above the sea | 17.9 km; no Earth-like planet reaches it | unchanged (fixed in blocks) |
| deposit `size_mult_v`, `spacing_mult_v` (per type; ORE-055, ORE-145, DEP-127, DEP-195, IMP-110/115/118/123) | as tuned | **changed**: × 1.75 (Ben, 2026-10-03, "Keep today's block height"), so ore bodies and seam gaps are as tall in blocks as at 20 m | × 1.75 (applied) |

The folds' vertical stretch, `structure.deformation.true_height_share` 4/15 (the share Ben's "4×"
was on 20 m blocks, DEF-011), keeps the structures' height in metres at any vertical scale, not
their look in blocks: at 35 m they are drawn 2.29 times their 1:1 shape instead of 4, calmer like
every other drawn shape. Ben kept the value (decisions log 2026-10-03).

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
−352; master has since moved it to −512, which 35 m does not need, and Ben kept it there (decisions log
2026-10-03, "Keep −512"). The sea floor's 1st percentile is
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

**Ore after "Keep today's block height"** (2026-10-03; quick 16k preset, seed 2, master 6f72f0dd,
which has the plate work, so the planets differ from the ones above; `deeptime deposit-audit`). The
vertical multipliers × 1.75 bring the ore back to about the 20 m figures of the same master:

| Planet | 20 m | 35 m, multipliers as tuned | 35 m, × 1.75 |
|---|---|---|---|
| Supercontinent, 1,000 Myr: sites | 264 | 291 | 291 |
| ore blocks in deposits | 9.05 million | 4.66 million | 8.09 million |
| median porphyry, ore blocks | 2,351 | 1,964 | 4,876 |
| Scattered, 500 Myr: sites | 143 | 149 | 149 |
| ore blocks in deposits | 6.46 million | 3.57 million | 5.77 million |
| median porphyry, ore blocks | 1,694 | 983 | 1,685 |

Coal is most of the ore (97 % and 95 %). The supercontinent planet has only 7 porphyries, so its
median moves with which body is fourth. Sites do not change: no body was dropped or kept because of
its taller drawing.

**Answered** (decisions log 2026-10-03): rock layers keep their real thickness
(`scale.feature_scale.strata` stays 1); ore bodies keep today's height in blocks (the vertical
multipliers × 1.75, above); the drawing settings keep their values; the floor stays at y −512.
**Still open** (STATUS.md "Pending questions"): plutons with a floor, whether the column builder
carries plutonic rock to the world's floor where a pluton's layer is the lowest stored one.

## Distant Horizons far terrain and Toroidal World's seam

Far terrain (P3-04) gives Distant Horizons its distant LODs from the baked world data instead of
from chunks generated far away. Nothing physical is decided here; the rows (BAK-039 to BAK-042,
BAK-048 and BAK-050) are game choices about how far terrain stays the same terrain as the chunks.

- **The seam.** Every far column folds x into the planet's lap [-C/2, C/2), the period at which
  the world data wraps and at which Toroidal World loops the level, so a section west of x = C/2,
  east of it or across it draws what the chunks there have (BAK-042). What far terrain does not
  draw is what is random per chunk on top of the column rule: trees, plants, coral reefs and the
  carvers' cuts. The checks read the ground under the first three (a mangrove's roots too, since
  2026-10-17: they are part of the tree, not of the ground) and count a column a carver cut
  below far terrain's ground apart (at most one in eight); on the MIC set Terralith's canyon, which
  reaches from y −20 to 70, cut a sea bed two blocks under the water down to y −2 (2026-10-03).
- **The top block** (BAK-048). A chunk's top blocks come from the pack's surface rule, which runs
  over the generated chunk; far terrain has no chunk, so a table of soils by biome
  (`far_terrain/surface.json`) gives them. Since 2026-10-18 far terrain first runs, in the game's own
  surface-rule code, the leading rules of the level's surface rule that ask nothing of a chunk (the
  biome, the block's height, the water over it, noise, the random state), and the table stands for
  the rest. It does so on dry ground, on the sea bed and on a lake's bed: the water pass fills a
  lake after the surface rule, so the rule saw the ground dry, and keeps the bed as the rule painted
  it, grass turned to dirt (`WaterBodies.underWater`, which the table's lake beds share); a river's
  bed is the pass's own sand or gravel. The first rule that reads the chunk ends that part:
  vanilla's second rule, `above_preliminary_surface`, reads the chunk's noise, so a vanilla world
  runs only its bedrock floor there and its top blocks are the table's. Expanded Ecosphere paints
  before that rule: dead brain coral on every warm ocean floor from y 58 to 62 and sand on lukewarm
  ones, which the seam test met when the planet of 2026-10-18 put the seam in warm ocean (43 of 64
  columns west of it and 44 east were coral where far terrain drew sand). Run on the sea bed, the
  rule left 9 of 128 columns west and 16 east coral where far terrain drew sand, all of them lake
  beds at the coast; run on lake beds too, it left none, and 126 of 128 columns a side had the same
  top block. The table follows vanilla's rule where it can (since 2026-10-18 a lukewarm ocean's dry
  ground is grass, as vanilla paints it, and its floor under water sand; only a warm ocean's dry
  ground is sand). On the same planet 8 of 128 columns west of the seam and 16 east (standalone)
  were dirt where far terrain drew sand, lukewarm sea floors 3 to 7 blocks under the water at the
  coast. That was put down to the noise aquifer leaving them dry when the rule ran, but vanilla's
  rule paints them dirt with the water over them too (BAK-050, read in its 1.21.1 source,
  `SurfaceRuleData.overworldLike`): a floor whose water starts at most 6 + surface depth blocks
  above it (`waterStartCheck(-6, -1)`, `UNDER_FLOOR`) takes the rule vanilla paints under a dry
  floor's top, dirt in an ocean or a lukewarm ocean, and only a deeper floor reaches the sea-bed
  rule, sand in a lukewarm ocean and gravel in most others. Painted dry, the floor is grass, which
  the water pass turns to dirt, so in vanilla's rule the aquifer changes nothing there. Since
  2026-10-20 far terrain runs those two conditions in the game's code on the column's own surface
  depth and draws the table's lake bed (the dry top as water leaves it) inside the band and its sea
  bed below. The planet of 2026-10-20 (PLT-293) put the seam in a shallow lukewarm sea and the test
  failed on it before the change: 76 of 108 columns east of the seam dirt near and sand far
  (standalone), 47 dirt, 14 clay and 7 gravel with MIC. With the band drawn, 112 of 112 columns west
  and 110 of 112 east have the same top block before features (standalone) and 92 of 112 and 77 of
  108 with MIC, where Terralith's rule paints gravel, clay and sand under shallow water from noise
  past `above_preliminary_surface`. After features vanilla's sand, clay and gravel disks cover part
  of those dirt floors (83 and 55 of 112 the same, logged only): they are features, which far
  terrain does not draw. Only a floor on the top block's agreement is asserted.
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
  on 64 blocks either side of the seam. The planet-wide test reads 128 scattered columns with their
  features; the seam test reads each chunk at the carvers, before any feature (Deep Time's terrain
  is complete by then), and logs the same columns after their features without asserting them.
  Its 16 columns a side lie 4 blocks apart, so one patch of a feature covers a quarter of a side
  or more: on the planet of P1-26's rain (2026-10-17) the equator west of the seam, sea on the
  planet before, became land, and there mangrove trees stood on 4 of the 16 columns (standalone)
  and Terralith's basalt cliffs stood basalt columns 5 to 10 blocks over the ground on 9 (MIC).
  Features over the planet are the planet-wide test's to bound; what the seam test checks is the
  fold. Read before features it found one real difference, which was fixed: Expanded Ecosphere's
  surface rule cut a black beach east of the seam from y 65 to 67 down to 62 (BIO-156). The seam
  can lie in the sea, as it did on the planet of 2026-10-18 (its four fixed rows held 3 dry
  columns west of the seam and 15 east), so the test adds up to eight rows, searched up to 7/8 of
  the way to the world's z edge (the poles on a Lambert world, the turning edges on a Petroff–Guyou
  one), chosen for the dry ground each side still lacks, and asserts that each side compares at least 16 dry columns and holds their ground
  within a block in nine of ten (BAK-042). Until 2026-10-19 it took four rows within C/8 of the
  equator, which on that day's planet held only 14 dry columns east of the seam; the wider search
  found 24. A side whose rows hold fewer than 16 dry columns must compare every one of them: on
  the planet of master 7d60fc4a the west of the seam is sea to 7/8 of the edges, so that side
  compares its sea bed only.
  `tools/smoke/run.py --dh` has Distant Horizons generate
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

## Distant Horizons world curvature

Ben, decisions log 2026-10-03: "btw, DH has world curvature settings, can you make it so those
default to values that make sense for the given world a player is in (dependent upon the world
size generated using our mod.)" In a Deep Time level the client's Distant Horizons (DH) curvature
is the planet's own; elsewhere DH's value is its own. The geometry is BAK-043 and BAK-044, the
choices BAK-045 to BAK-047. Sources: the DH 3.3.1 jar (the MIC pack's pin), read on the test VM
with `javap` and by opening its shaders and lang file; DH is LGPL-3.0 (the jar's `LICENSE.LESSER.txt`
and `neoforge.mods.toml`), so reading it and calling its API is fine and nothing was copied.

### What DH offers (3.3.1)

- **The setting.** `Config.Client.Advanced.Graphics.Experimental.earthCurveRatio`, shown as "Earth
  Curve Ratio" under DH's experimental graphics options. An `Integer` from -5,000 to 5,000, default
  0. It is a **client** setting: the shader that reads it runs only on a client, and the value
  lives in the player's DH config file.
- **Units.** The ratio of Earth's size to the curved world's. The terrain shader draws a sphere of
  radius `6,371,000 / ratio` blocks (DH's own table: 1 is 6,371,000 blocks, 100 is 63,710, 10,000
  is 637.1). 0 is flat; a negative ratio bends the ground up into a bowl. DH accepts 50 to 5,000
  (spheres of 127,420 down to 1,274 blocks) and its change listener rewrites 1 to 49 to 50 and
  saves it, which would put a value in the player's file, so Deep Time never offers one.
- **What it bends.** Only DH's own level-of-detail meshes (its terrain vertex shader). Minecraft's
  chunks are drawn flat. At DH's near edge, where its meshes meet the chunks, the bent ground sits
  below the flat chunk edge by the drop at that distance: 7.1 blocks at 192 and 12.6 at 256 on a
  16k planet, half that at 32k. Not yet seen on a screen (below).
- **The API.** `DhApi.Delayed.configs.graphics().earthCurvatureRatio()` returns an
  `IDhApiConfigValue<Integer>` (read in 3.3.1, API 7.1.0; `DhApiBindingTest` finds the same members in the newest jar, 3.3.3, API 7.2.0). It has two
  layers: `getTrueValue()` is what the player's file and settings screen hold, `getApiValue()` is
  an override held in memory, `getValue()` is the override when set and the true value otherwise
  (the shader reads this, every frame), `setValue(n)` sets only the override, `clearValue()` removes
  it, `getCanBeOverrodeByApi()` says whether the entry allows it. Nothing on the API reaches the
  file. While an override is set, DH's settings screen locks the entry ("API LOCK: this option is
  controlled by another mod via DH's API so it cannot be changed via the UI or config file").
  `DhApi.Delayed.configs` is null until DH has started its API, so a mod polls for it.

### How Deep Time sets it

- **Planet size.** Already on the client: `WorldInfoPayload` carries the circumference C in blocks
  (the GameTest `worldInfoFollowsThePlayer` checks it), and `ClientPlanetState.circumference(level)`
  answers 0 for any level that is not the planet the server last described. No new payload.
- **The rule** (`DhCurvature`, `client/DhCurvatureClient`, once a second on the client tick, only
  when DH is installed; DH's API is bound by name like the rest of the compat): in a planet level
  with the setting on, put `ratioFor(C)` on the API layer; anywhere else clear what Deep Time put
  there. The player's own value wins: if `getTrueValue()` is not the default 0, nothing is set (a
  value typed into DH's screen later takes Deep Time's off within a second). Another mod's override
  is left alone, and DH's "not overridable" answer is respected. Leaving the server clears at once.
  The player's config file is never written, so a vanilla world is never curved and nothing needs
  restoring after a crash. The one thing that cannot be told apart is a player who typed 0 on purpose:
  0 is DH's default, so it counts as "untouched", and they use the switch.
- **The switch.** `config/deeptime-client.toml`, `dhAutoCurvature` (default true), with a screen
  under Mods, Deep Time, Config (lang keys `deeptime.configuration.*`).

### The arithmetic

Radius `R = C / 2 pi`; ratio `= round(6,371,000 / R)`; the drawn sphere is then `6,371,000 / ratio`
(within half a ratio step, at most 0.04 %). Drop at horizontal distance `d` (DH's shader moves the
vertex to y += (cos phi - 1)(R + y), phi = d / (R + y); at sea level `R (1 - cos(d / R))`).

| World | C (blocks) | R = C / 2 pi | 6,371,000 / R | ratio set | sphere drawn | drop at 256 | at 512 | at 1,024 |
|---|---|---|---|---|---|---|---|---|
| 16k | 16,384 | 2,607.6 | 2,443.25 | 2,443 | 2,607.9 | 12.56 | 50.10 | 198.5 |
| 32k | 32,768 | 5,215.2 | 1,221.62 | 1,222 | 5,213.6 | 6.28 | 25.11 | 100.2 |

Both are inside DH's 50 to 5,000. Doubling the world halves the ratio and the drop (a parabola,
`d^2 / 2R`, to 1.3 % at 1,024 on 16k). DH's range covers circumferences from 8,006 to 800,603
blocks; below, the strongest curve (5,000) is used, above, flat (BAK-046).

### What this does not do: the vertical exaggeration

A block is 300 m across and 35 m up (SCL-001, SCL-002), so the ground of a planet drawn to scale
falls 300 / 35 = 8.57 times as far, in blocks, as an isotropic sphere of the same circumference:
107.6 blocks at 256 on 16k (3,767 m, a real 77 km view on the planet), 429.5 at 512, 1,701 at 1,024.
That sphere is 304 blocks across at 16k, ratio 20,942, which DH cannot draw (limit 5,000, 1,274
blocks, 25.7 blocks of drop at 256). Deep Time uses the planet's radius in blocks, as asked: a world
that curves like a sphere of its own circumference, not one that matches its mountains' scale
(BAK-045; STATUS.md, pending question).

### What was checked

Unit tests (`DhCurvatureTest`): the conversion against the geometry, the two world sizes, the
parabola, the limits (never a value in 1 to 49), and the rule against a model of DH's two layers
(vanilla flat, set, resize, hand back, the player's value kept and later released, another mod's
override kept, "not overridable"). `DhApiBindingTest` runs the same controller through the real
`IDhApiConfigValue`, `IDhApiGraphicsConfig` and `IDhApiConfig` interfaces of the pinned and the
newest DH jars. **Not checked:** a real client. The GameTests have none, and the software-GL
client has not made DH request LODs (see above), so how the bent LODs meet the flat chunks has
not been seen.
