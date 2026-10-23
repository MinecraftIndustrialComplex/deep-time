# The young landscape and the amplifier

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The deep-time simulation works on cells several kilometres wide. Two later steps bring the terrain down to the size of a block. The young landscape reruns erosion for the last stretch of history on a finer grid, so that valleys and ridges are sharp where the land is young. The amplifier then adds block-scale form: it carves branching valleys with a stream-power kernel (after Schott and others 2024), limits slopes, and gives mountains a character that depends on their rock and age. Nearly every amplifier constant was tuned by eye to look right and is marked `game-choice`; this page must say so plainly.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [YNG](ledger/YNG.md) | young landscape | YNG-001 to YNG-014 (14) |
| [YNG](ledger/YNG.md) | detail noise | YNG-015 to YNG-021 (7) |
| [YNG](ledger/YNG.md) | sea floor | YNG-022, YNG-023 (2) |
| [AMP](ledger/AMP.md) | amplifier kernels | AMP-001 to AMP-022 (22) |
| [AMP](ledger/AMP.md) | incision | AMP-023 to AMP-033 (11) |
| [AMP](ledger/AMP.md) | mountain character | AMP-034 to AMP-066 (33) |
| [AMP](ledger/AMP.md) | resampling | AMP-067 to AMP-071 (5) |
| [AMP](ledger/AMP.md) | sea floor | AMP-074 (1) |
| [AMP](ledger/AMP.md) | steep-ground texture (the bake's relief pass) | AMP-080 to AMP-084 (5) |
| [AMP](ledger/AMP.md) | lake shores | AMP-090, AMP-091 (2) |

Rows by status (YNG and AMP, recounted 2026-10-22): `verified` 1, `cited-unverified` 8, `calibrated` 3, `game-choice` 71, `uncited` 19.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/AMP.md](extraction-notes/AMP.md), [extraction-notes/HYD.md](extraction-notes/HYD.md).
- Work package in the verification plan: WP-6 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- `coarse_iteration_factor` is coded 2.0 where the build log and plan say 3 (extraction notes AMP, flag 1).
- The thermal slope comment says "Schott: tan 30 degrees = 0.57" and the default is 0.6. In Schott's published shader 0.57 is used only on a non-default branch; the default threshold is a tangent of 0.9 to 1.4 (AMP-034, AMP-038).

## One-cell spires (margins and spires task, 2026-10-03)

**What was wrong.** The vertical-scale study saw needles one block wide and 20 to 50 blocks tall on steep ground.
Counted on the baked terrain (an L3 cell, two blocks, standing above all eight neighbours), master 00ef214c had about
450 land cells more than 8 blocks above all their neighbours on each quick 16k planet and 50 to 60 more than 16
(tallest 31 to 33 blocks at 20 m per block); master f545c0bc 132 and 167 over 8 blocks, 12 and 10 over 16, tallest
42 and 20. All of them were in the elevation layer itself, not the detail noise.

**Cause.** Switching the amplifier's terms off one at a time on one planet's young checkpoint: the mountain
character (`amplify.ruggedness` 0) removed them (over 8 blocks: 450 → 3); within it the gullies (450 → 21); the glacial
pass (450 → 67). Two inputs of the character varied from one cell to the next: (1) the fade over glacial troughs was
taken from each cell's own glacial cut, which jumps between neighbours at the ice margin and over the bed's rills, so
single cells kept the full character between faded ones (removing that fade alone: 777 → 64 on the run that was
used); (2) each gully octave took its direction from a one-cell gradient of the terrain plus the coarser octaves,
which turns freely at summits, saddles and the coarser gullies' crests and axes, so neighbouring cells read
unrelated stripes (with both gone: 3).

**Fix.** The trough fade is a box mean of the cut over three cells (AMP-072); each gully octave takes the fall line
at a quarter of its wavelength from five gradients and fades where they disagree (AMP-073). Test
`surface::amplify::tests::a_glaciated_young_range_has_no_one_cell_spires`: on a young, steep, glaciated range at
ruggedness 2 no cell stands more than 6 blocks (120 m at the test's 20 m per block) above all eight neighbours; the
old code left 14. Meant exceptions, not on that planet: a volcano's summit and an impact crater's central peak.

**Measured** (master f545c0bc before, this change after; land cells over all eight neighbours by more than 8 / 16
blocks, tallest): planet A 132 / 12 / 41.6 → 3 / 0 / 11.6; planet B 167 / 10 / 19.8 → 0 / 0 / 6.0.


## The amplifier on a GPU (2026-10-18)

**What changes.** With a usable GPU (`compute` auto or gpu; `docs/design/gpu-compute.md`) the amplifier's middle
passes (stream-power erosion and its drainage area, the glacial valleys, thermal erosion, deposition) run as compute
shaders on each L3 tile. The laws, constants and iteration counts are the CPU's: no ledger row changes and none is
added. The arithmetic is the CPU's f32, summed over the eight neighbours in the same order, so a pass differs only by
rounding: fused multiply-adds the shader compiler may form, `pow` for exponents other than 1, 1.5 and 2, and the
glacial box mean summed as Kahan-compensated differences from the centre cell instead of in f64.

**What that does to the terrain.** Rounding can flip a branch the CPU takes on a comparison (the steepest receiver,
a positive slope), which moves a little water or sediment to another neighbour. The tolerance test
`surface::amplify::tests::the_gpu_passes_stay_within_their_error_budget_of_the_cpu` holds one glaciated window to:
under 0.1 % of cells more than 1 cm from the CPU's elevation, erosion, sediment and incision cap, a mean difference
under 1 mm, none over 5 m, and the glacial statistics within 0.1 %; through the tiled ladder, under 1 % of L3 cells
more than 1 cm apart. A world made with a GPU is therefore not byte-identical to one made on the CPU, or on another
GPU or driver (Ben, 2026-10-18: worlds are shared as finished folders). The measured figures are in the design note,
section 10.

## Sea-floor walls (sea-floor spikes task, 2026-10-20)

**What Ben saw.** The sea bed looked "very spiky, like a sin wave". The question was whether that is
realistic. In real units it was not, though part of the look is the vertical scale and stays.

**Measured, before.** Ben's play world (`/data/ban/play/be17a7cf`, earthlike_quick_16k), every
deep sea-floor sample (more than 6 blocks, 210 m, under sea level) of the whole world sampled every
2 blocks with each block's own size (`deeptime heightmap --scale`; 183 to 474 m a block on the
Petroff-Guyou map, median 296 m), slopes in real metres (35 m a block up):

| | the play world, 2 blocks (about 590 m) | the same, 6 blocks (about 1.8 km) | ETOPO 2022 60″, whole ocean (1.85 km) | ETOPO 2022 15″, five rough 15° tiles (460 m) |
|---|---|---|---|---|
| slope p50 / p90 / p99 / p99.9 | 2.0 / 6.7 / 28.1 / 40.8° | 1.9 / 6.6 / 24.1 / 37.0° | 0.9 / 4.1 / 12.2 / 19.6° | 0.9 to 2.5 / 3.9 to 9.6 / 12.9 to 21.3 / 24.6 to 31.8° |
| share steeper than 10° / 20° / 30° | 6.3 / 2.3 / 0.75 % | 5.9 / 1.7 / 0.40 % | 1.8 / 0.088 / 0.0036 % | 1.8 to 9.3 / 0.26 to 1.3 / 0.03 to 0.15 % |
| relief RMS in 38 km tiles (high-pass, σ 4.7 km), p50 / p90 / p99 | 63 / 312 / 618 m | 62 / 301 / 583 m | 49 / 148 / 274 m | 41 to 114 / 109 to 232 / 244 to 331 m |

The body of the distribution (the median, the 90th percentile) is within the range of Earth's
rough regions. The tail is not: the planet as a whole had 2 to 9 times the share steeper than 20°
of Earth's roughest 15° tiles (a trench and arc, a slow ridge, a fracture-zone field) and 5 to 25
times the share steeper than 30°; against the whole ocean at 1.85 km, 19 and 110 times. The
detail noise is not the cause: of the 195 m RMS of the high-passed terrain, 7.8 m is the detail
term and the rest the elevation layer itself.

Earth's own figures are a lower bound: about 80 % of the ocean floor in these grids is predicted
from satellite gravity, which resolves half-wavelengths of about 6 km only (Sandwell2022, read), and
satellite bathymetry "is not a valid proxy for multibeam measurements" (BeckerSandwell2008,
abstract). Multibeam surveys of a trench give its flanks a mean slope of 7.7° and 5.9°, accretionary
ridges up to 15° (locally over 20°), and bend-fault scarps 10 to 40° that are 150 to 1,200 m high
(Kermadec, StewartJamieson2018, read). So slopes of 25 to 40° are real at the scale of single
scarps, a few hundred metres across; they are not 2 to 5 km walls standing over 5 to 10 km.

The sea floor's ordinary roughness is gentler still. Abyssal hills, the statistical sea floor of
Goff & Jordan 1988 and Goff 1991 (abstracts only), have RMS heights of 56 to 236 m and
characteristic widths of 1.8 to 8.2 km, the largest on slow ridges (GoffSmithMarks2004, Table 2,
read): slopes of a few degrees, and most hills at or below the 2.4 km L2 cell, so the planet draws
them with its detail noise or not at all, and a 15° failure does not touch them. Seamounts are the
steep exception: SYNBATH fits them with a maximum slope of 0.25, 14° (Sandwell2022, read), Wessel
2001 models them as Gaussians (abstract only), and Smith 1988's height to basal radius of about
0.21, a 12° flank, is from memory (not read).

**Where the walls come from.** Slopes on the simulation's own grids (run-s1: earthlike_quick_16k,
start supercontinent, 1000 Myr, default seed, master of 2026-10-20 morning, which is the play
world's own planet: its young and amplify stages rerun with the fix switched off bake the play
world's terrain to the block, `tools/review/seafloor/compare.py`; `deeptime dump-fields` and
`tools/review/seafloor/stages.py`, ground metres with the Lambert grid's own east-west and
north-south spacing):

| field | grid | share steeper than 20° / 30° | p99.9 | cells with a drop over 1 km / 2 km to a neighbour |
|---|---|---|---|---|
| the plate model's elevation | L1, 4.8 km | 2.0 / 0.68 % | 44° | 4.7 / 1.9 % |
| the surface stage's | L1 | 2.0 / 0.60 % | 42° | 5.0 / 1.8 % |
| the young landscape's | L2, 2.4 km | 2.1 / 0.81 % | 47° | 1.9 / 0.46 % |

The walls are already in the plate model's sea floor, on a 4.8 km grid where Earth would show even
fewer steep cells than at 1.85 km: trenches pulled toward 10 km in a single sample, and strips where
plates and crusts of different ages interleave, 2 to 5 km tall over 5 to 9 km. Nothing after it
relaxed them under the sea: the marine diffusion vanishes below the 60 m wave base, the submarine
slope failure (ERO-168) moves sediment only, and the amplifier's thermal pass held every cell, land
or sea, at the land's talus angle (0.6, 31°, AMP-034). The interpolation from L2 to L3 then drew
each step as a smooth S over two or three L2 cells, which is the "sine wave": across a 384-block
box of the worst sea floor (close-up C, x -5056 to -4672, z -1088 to -704) the floor drops 100 to
130 blocks (3.5 to 4.5 km) over 15 to 20 blocks (4.5 to 6 km) and climbs again, trench after
trench.

**The vertical scale's part.** The game draws 35 m as one block up and about 296 m as one block
across, so a slope looks 8.4 times steeper than it is: 6.8° real is 45° in the game, 15° is 66°,
30° is 78°. Earth's own deep sea floor drawn this way is steeper than 45° on screen over 4.3 % of
its area (the whole ocean at 1.85 km) and over 4.0 to 16.5 % in the five rough tiles at 460 m
(Mariana arc and Philippine Sea 16.5 %, Mid-Atlantic Ridge 13.2 %, Tonga 12.6 %, Pacific fracture
zones 9.7 %, East Pacific Rise flank 4.0 %); the play world had 11.6 %. So most of the "spiky" look
on screen is the exaggeration, and an Earth-like sea floor keeps it: changing it is Ben's decision
(the vertical scale, SCL ledger), not a sea-floor change. What was wrong is the tail: the walls
steeper than 20° and 30° in real units, which on screen are the near-vertical cliffs of 70° to
over 80°.

**Fix.** Two steps, each the sea floor's own critical slope, so the walls fail to a ramp and the
fallen rock builds the toe:

1. The young landscape, before its steps, runs the marine slope failure (`marine::slope_failure`,
   the same mass-conserving rule as ERO-168) on every open-sea cell of L2 with nothing limiting what
   a cell gives (rock as well as sediment): `young.sea_floor_slope_deg`, YNG-022, at 300 iterations
   (YNG-023, numerical: a 3 km step one cell wide, a 5 km one-cell trench and a maze of 2.8 km
   blocks relax to within a degree in 100 to 200, `tools/review/seafloor/relax.py`). L2 is where it
   reaches: a step 2 to 5 km tall needs 8 to 19 km of ramp at 15°, more than the amplifier's
   thermal pass moves matter at L3 (about 2.8 km).
2. The amplifier's thermal pass holds every cell below sea level at `amplify.submarine_slope`
   (AMP-074) instead of the land's `thermal_slope`, with the same ±25 % noise and drama scaling,
   so the L3 interpolation does not steepen the ramps again. A sea cell under more than 60 m of a
   glacier's ice keeps the land's angle: a fjord is a trough the ice cut in rock.

Tests: `surface::young::tests::the_plate_model_sea_floor_fails_to_its_critical_slope` (a 96 x 48
L2 world of land, shelf, a plain at 4.5 km, a one-cell trench to 10 km and a strip of 2 x 2 blocks
1.5 km high: before, its steepest open-sea pair is over 50°; after, at most 15° + 3 %, matter kept,
land untouched, the trench still deeper than 5.5 km) and
`surface::amplify::tests::the_sea_floor_keeps_its_own_critical_slope` (the multiplier cell by cell,
the iced exception, and a sea floor of 700 m knolls one L2 cell across that comes out of the
amplifier at least a fifth gentler than at the land's angle, 22° against 36°, and gentler than the
land's angle allows anywhere, 24° at the bottom of its noise; 24 iterations do not take a 700 m
step all the way to 15°, and on a planet the young landscape's ramps arrive near it already). Each
test's "before" state breaks its own bound: the young test checks its world is over 50° before the
failure runs, and the amplifier test runs the same knolls at the land's angle beside the submarine
one.

**Calibration.** Run-s1's young and amplify stages again from its deposits checkpoint with each
pair of slopes, baked, and surveyed as the play world was (`tools/review/seafloor/calib.sh`; the
projection's central meridian pinned at the 146° the bake chose for this planet, since left free it
moved to 193.25° or 13.5° when a few coastal cells changed). Deep sea floor, at 2 blocks (about
590 m) and, in brackets, 6 blocks (about 1.8 km):

| run | young `sea_floor_slope_deg` | amplify `submarine_slope` | share steeper than 20° | steeper than 30° | p99 / p99.9 | relief RMS p90 / p99 |
|---|---|---|---|---|---|---|
| off (the play world) | 0 | 0.6 (31°, the land's) | 2.29 (1.67) % | 0.75 (0.40) % | 28.1 / 40.8° | 312 / 618 m |
| young only | 15° | 0.6 | 1.10 (0.66) % | 0.21 (0.079) % | 20.6 / 33.5° | 259 / 416 m |
| amplifier only | 0 | 0.268 (15°) | 1.07 (0.78) % | 0.16 (0.10) % | 20.3 / 33.5° | 270 / 477 m |
| 10° | 10° | 0.176 | 0.25 (0.16) % | 0.044 (0.020) % | 13.7 / 25.6° | 198 / 303 m |
| **15° (the default)** | 15° | 0.268 | **0.50 (0.35) %** | **0.096 (0.053) %** | 17.6 / 29.7° | 246 / 391 m |
| 20° | 20° | 0.364 | 1.42 (0.86) % | 0.16 (0.096) % | 21.3 / 33.3° | 279 / 465 m |
| ETOPO 2022 15″, five rough tiles (460 m) | | | 0.26 to 1.32 % | 0.029 to 0.15 % | 12.9 to 21.3 / 24.6 to 31.8° | 109 to 232 / 244 to 331 m |
| ETOPO 2022 60″, whole ocean (1.85 km) | | | (0.088 %) | (0.0036 %) | (12.2 / 19.6°) | (148 / 274 m) |

Each step alone halves the share steeper than 20° (1.10 and 1.07 %, inside the rough tiles' range
near its top) but leaves the share steeper than 30° above it (0.21 and 0.16 % against at most
0.15 %); only the two together bring both into the middle of the range. The young step does more
at 1.8 km (0.66 against 0.78 % steeper than 20°), where the long walls are, and the amplifier more
on the steepest blocks at 590 m, where the interpolation had steepened them.

The rule (the coordinator's call, 2026-10-20): the steepest critical slope that brings the
planet's share steeper than 20° and steeper than 30° into the range of Earth's rough tiles. 15°
does, 20° is just past the top of both ranges, and 10° sits at their bottom. 15° is also the
upper end of the accretionary ridges' slopes on a multibeam-mapped trench (up to 15°, locally over
20°) and twice the trench flanks' mean (7.7° and 5.9°, StewartJamieson2018). The planet as a whole
is held to the rough tiles rather than to the whole ocean because the gridded ocean is a lower
bound (above), and a 4.8 km plate grid draws a planet's sea floor as trenches, ridges and fracture
zones more often than Earth's abyssal plains. The body of the distribution does not move (the
median 2.0°, the 90th percentile 6.7° to 7.0°), nor the share steeper than 45° on screen (11.6 to
12.3 % of the samples).

**What else it changed** (15° against off, the same planet and projection, `compare.py`): mean
depth 2,870 m before and after; the shelf (shallower than 200 m) 7.63 → 7.53 % of the sea; the
hadal sea floor (deeper than 6 km) 1.60 → 1.36 % of the sea and the deepest point 9,944 → 9,889 m,
since the one-sample trenches give rock to their own walls' toes. 0.08 % of the samples went from
land to sea and 0.04 % from sea to land. Land changed by more than 2 blocks on 2.6 % of its samples,
94 % of them within 64 blocks (about 19 km) of the sea and none farther than 256: the young
landscape's 2 Myr of rivers and coasts and the amplifier's drainage over a different shelf; the
young step alone does the same (2.4 %). The amplifier's glacial pass carved 2.7 % more volume
(2.87 → 2.94 x 10⁴ km³) over 15 % more cells; that is the amplifier's step (alone, 20 % more
cells), the young step alone gives 8 % fewer, and it was not traced further.

**Earth's hadal zone is smaller than the planet's, before and after.** Earth's hadal areas total
"over 800,000 km²" (StewartJamieson2018, read), about 0.22 % of an ocean of about 361 million km²
(the ocean's area from memory). The planet's 1.36 % comes from the plate model's trenches and deep
basins; it is not this task's to change and is listed under the open questions.

**On today's planet.** While this work ran, master changed the plate model (continents keep their
land: lower-crustal flow and isostatic settling; master c83f445e). A fresh run on it (the same
preset, start, length and seed; the bake chose λc 113.25, pinned for both arms), its young and
amplify stages rerun with the fix off and at 15° (`calib.sh m-off 0 0.6 113.25 run-m1` and
`calib.sh m-d15 15 0.268 113.25 run-m1`, with a binary of this branch merged with that master):

| run | share steeper than 20° | steeper than 30° | p99 / p99.9 | relief RMS p90 / p99 |
|---|---|---|---|---|
| off | 1.37 (0.83) % | 0.37 (0.15) % | 23.0 / 36.2° | 220 / 495 m |
| 15° | 0.36 (0.21) % | 0.061 (0.029) % | 16.5 / 26.9° | 185 / 341 m |

The new plate model's tail is smaller than the play world's (1.37 against 2.29 % steeper than 20°)
but its share steeper than 30° is still 2.5 times the top of Earth's rough range; with the fix both
shares sit in the lower half of the range and the p99.9 inside it. Its sea is 76 % of the planet
(84 % on run-s1), the shelf 12.2 % of the sea, the mean depth 2,590 m, the hadal sea floor 0.51 →
0.42 % of the sea (Earth about 0.22 %), and the share steeper than 45° on screen 7.6 → 8.1 %. The
deepest point falls from 9,112 to 7,833 m: failing to 15° gives 1.3 km of the deepest trench's
floor to its walls (on run-s1 the deepest point lost 55 m; the trench's width was not measured). Earth's deepest trenches reach about 10.9 km (the Challenger Deep, from
memory), so the planet's deepest point is now shallower than Earth's; a trench drawn wider than one
sample at the source would keep its depth through the failure (open questions).

**Simplifications.** One critical slope for all the sea floor, with no cohesion, failure plane or
runout: real walls stand steeper where the rock is strong (fault scarps 10 to 40°, the inner walls of
trenches, carbonate platform margins) and fail as slumps and debris avalanches that run out tens of
kilometres. Failed rock is drawn as the toe's own column, with no debris or turbidite facies in the
strata. Single scarps a few hundred metres across are below the L2 cell (2.4 km) and are not drawn.

**Game choices.** The iteration count (YNG-023). The fjord exception at the glaciers' minimum ice
(60 m, the glacial shape's `min_ice_m`), so that a trough cut by ice keeps its walls under the
water.

**Not checked.** The walls of the half-turn edge rows (z near ±3,328, where the Petroff-Guyou map is
most anisotropic and a sample's east-west and north-south sizes differ most; the roughest tiles of
the planet are there, and their real slopes are taken with one size per sample); the square steps of
the shallow sea floor that follow L1 cells (close-ups Q and P, 10 to 20 blocks tall, in both runs);
the floating stone shelf and the contour terraces; the Lambert grid's cos φ anisotropy in the
failure itself; carbonate platform walls.

**Open questions.** Whether the plate model should draw its trenches wider than one sample and its
crust strips with gentler edges at the source (the plates' and ocean crust's owners); this fix
relaxes what reaches the young landscape, and on today's planet it costs the deepest trench
1.3 km (9.1 to 7.8 km, against Earth's 10.9). The vertical scale (Ben's).

## Steep ground: the jag and the cliff bands (D3, 2026-10-22)

Ben's decision (decisions log 2026-10-22, D3): "jagged detail and cliff bands on steep ground in the
heightfield, far terrain kept matching, no 3D overhangs".

**What was wrong.** A steep face was drawn as an even stair: the game's surface steps from block
to block by the same amount all the way up, with nothing to break it. The page "Volcanoes: realism
or us?" counted 45 % of the columns stepping 3 or more blocks to a neighbour in the 201 blocks
round one volcano; over the whole
world the share is far lower, 3.8 to 5.4 % of steep land (slope 0.5 or more over 8 blocks) and 6
to 8 % of steep hard rock and steep lava (seeds 1 to 3, below), and about 12 to 15 % of those steps
change from one block to the next (a "broken" face). The mountain character already terraces
steep hard rock in the amplifier (AMP-059), before the projection resamples the terrain to the
world grid; the master column of the census below is what reaches the drawn surface.

**What the bake does now** (`bake::relief::steep_texture`, AMP-080 to AMP-084). Last in the
projection pass, after the cone gullies, on the world's own grid and in world blocks, from the
drawn terrain (`elevation`) and its top rock (`surface/class_top`):

- **The jag.** Steep hard rock gets a ridged noise, (1 − abs(n))² less its mean, of two octaves from
  32 blocks, so the face breaks into knobs and chutes. Its amplitude is 4 blocks per unit of slope,
  at most 8, and comes in between slopes of 0.3 and 1.2 (blocks per block over 16 blocks) and with
  the rock's hardness (AMP-048: none on shale or mudstone, all on granite, gneiss, quartzite and
  basalt). Stronger rock standing in steeper, higher relief is the direction Schmidt & Montgomery
  1995 found (abstract read: "mountain-scale material strength can limit topographic relief"); the
  numbers are the game's.
- **The cliff bands.** Steep lava (basalt, andesite, rhyolite, obsidian: the volcanic classes that
  are neither friable nor porous) gets flat benches with a cliff over the lowest 30 % of each band's
  run, as stacked flows of a flood basalt or a lava plateau stand. The thinnest band is three
  Columbia River basalt flows, 105 m (flows "averaging 30-40 m", Swanson & Wright 1981 in USGS
  Circular 838, read), 3 blocks at 35 m a block; thicker ones are 2, 4 and 8 times that, up to 24
  blocks, chosen so a band spans about 24 blocks of run down its slope. The doublings nest, so a
  bench stays level where the slope changes; one slow noise (512 blocks) shifts all the levels
  together. The bands only raise the ground, at most about 0.71 of a band.
- **Where not.** Nothing within a block of the sea (fading in to 6 blocks over it), on a lake or a
  river, under ice (fading out over 4 blocks of it), or on a young standing volcano's body (the
  cone gullies' calm, VOL-068), whose flanks the gullies draw. A Lambert world gets none.

Everything goes into `elevation`, so the game's near terrain, the far terrain (Distant Horizons)
and the column evaluator draw the same surface; no layer was added (FORMAT.md, "The bake's
projection pass").

**Why 24 blocks of run.** The game takes the terrain's density at corners 4 blocks apart and
interpolates between them. A band 3 blocks high on a slope of 0.5 spans 6 blocks of run, so its
cliff falls between two corners and draws as a zigzag of single corners, not as a cliff (the first
renders of this task). At 24 blocks of run the cliff, about 7 blocks, spans two corners and the
bench four. So the bands are 3 to 24 blocks thick (105 to 840 m), several flows each, not single
flows.

**Measured** (`deeptime heightmap --step 4 --columns` of the whole world and a census of the
surface as the game draws it, H at the 4-block corners and bilinear between, floored; master
against the branch, each branch's `deeptime resume` of the same last checkpoint of
`earthlike_quick_16k`, scattered continents, 500 Myr, seeds 1 to 3, 2026-10-22; masks from master;
scratch scripts build/vrelief/steps.{sh,py}, not committed). Steep: slope 0.5 or more over 8
blocks; hard rock: hardness 0.9 or more; broken: a second difference of 2 or more blocks along x or
z.

| seeds 1 / 2 / 3 | master | branch |
|---|---|---|
| steep hard rock (0.70, 0.54, 0.73 M columns): steps of 3+ | 6.2 / 7.0 / 7.7 % | 11.9 / 10.8 / 15.3 % |
| steep hard rock: steps of 6+ | 0.07 / 0.14 / 0.12 % | 0.70 / 0.56 / 1.20 % |
| steep hard rock: of the 3+ columns, broken | 14 / 14 / 12 % | 27 / 24 / 28 % |
| steep lava (0.38, 0.38, 0.44 M columns): steps of 3+ | 6.2 / 6.0 / 8.2 % | 16.2 / 11.1 / 20.4 % |
| steep lava: steps of 6+ | 0.11 / 0.12 / 0.13 % | 1.24 / 0.71 / 1.93 % |
| steep lava: of the 3+ columns, broken | 15 / 15 / 13 % | 31 / 28 / 32 % |
| steep lava: flat (no step to any neighbour: benches) | 1.6 / 1.5 / 1.6 % | 7.7 / 4.1 / 8.3 % |
| all steep land (7.6, 5.9, 7.4 M columns): steps of 3+ | 3.8 / 5.0 / 5.4 % | 4.7 / 5.6 / 6.6 % |
| all land: steps of 3+ | 1.5 / 1.9 / 2.7 % | 1.9 / 2.1 / 3.3 % |

The pass moved 772k, 398k and 661k of the world's 2-block cells (jagged 759k, 391k, 648k; banded
127k, 77k, 137k), a mean 0.8 to 1.1 blocks each, at most 20.3 blocks up and 4.6 down. Renders,
before and after, shaded relief of a 384-block box and a game-scale oblique of its middle 160
blocks (review/out/vol-relief/pair-d3-s1/, not committed): `lava` (-2048, -1792), a lava range
whose steps of 3+ go from 1.2 to 6.4 % of the box's land, benches and risers plain on its flanks;
`mtn` (-4352, -2176), hard rock that is not lava, a modest jag (0.0 to 0.3 %); `vol-a` and `vol-b`,
two volcanoes with the cone gullies (1.2 to 3.2 % and 0.8 to 1.9 %).

**Cost.** The texture takes 0.20 to 0.39 s of the bake's projection (the cone gullies 0.8 to 1.2
s), 4 threads, on a full 16k world; whole `deeptime resume` runs side by side under the same load
took 1.5 to 2.5 % longer (137.3 against 139.3 s, 134.9 against 138.3, 132.5 against 135.7), the
same peak memory (2.87 GB). The game's per-chunk cost is unchanged: no Java or column-evaluator
code changed, and the column evaluator over the whole world (5.5 M corners) took 31.5 against
31.7 s on seed 2.

**Simplifications.** A heightfield: no overhangs, no arches, no undercut cliffs, and a cliff rises
over at least one spacing of the game's 4-block corners. The jag is noise, not fractures or joints: it
does not follow bedding, faults or the rock's jointing (DiBiase et al. 2018, from memory, tie the
relief of bedrock landscapes to fracture density). The bands are on lava only and run level, not
along the flows' own dips, and they are several flows thick; sedimentary cliff bands (sandstone
over shale) are left to the mountain character's terraces (AMP-059). Cells, not columns: the
texture is drawn on the 2-block cells of `elevation`, so its finest detail is 8 blocks.

**Game choices.** Every number but the flows' thickness (AMP-080 to AMP-083): the jag's strength,
wavelength and slope and hardness ranges, the bands' run, cliff share, blend and level noise, the
fades; the factor of three flows (AMP-084), so the bands start where the game's corners can draw
them. Chosen by the census above and the renders (the agent, D3).

**Open questions.** Whether hard rock should get cliff bands too where it is layered (sandstone,
limestone), and whether the bands should follow the strata's dip instead of running level (they
would then read the structure). Whether the jag should be stronger on glacial headwalls.

## The amplifier's and the bake's memory estimate (amp-est, 2026-10-23)

Not a physical model: what the run's memory estimate (`session::estimate_for`, which the workbench
and the editor hold to a run's cap) counts for the amplifier and the bake, against the counting
allocator's measured peak. No ledger row names these byte constants; they are figures of the memory
model, marked so in the code.

**Before.** On quick 16k at 50 Myr the estimate listed 3,750 MB for a run whose allocator peaked at
2,894 MB (cache on). The amplifier's figure counted W x W/2 cells on every level where the world's
grids have `Projection::grid_rows` rows (909 MiB of cells the run never has on the last level),
and left out `whole_last_level_peak`'s extra (the level the ladder keeps, the preview). Under that
excess the bake's figure was too low: it was the projection pass alone, 452.6 MB, where the pass
held 969.9 MB above the world (the closed basins on the world's ground, HYD-105) and the bake 1,658
MB in all; on the world Ben plays (1000 Myr) 1,877 MB was listed and 2,581 MB held. An honest
amplifier figure alone would have let the editor accept a run whose bake passes its cap.

**Where the "300 MiB" went.** The run held about 300 MiB more than its start plus the whole last
level's peak. (1) Most of it was the allocator's kept blocks (freed large blocks it keeps for
reuse; 655.9 MB at most on quick 16k at 50 Myr, counted peak 2,893.7 MB against a live peak of
2,409.8). They are held under a ceiling the session sets from the estimate and never above the cap
(`alloc::keep_below`), and the run's reservations (`StageContext::reserve`) count live bytes only,
so they cannot carry a run past its cap. (2) A real gap in the run's own reservation: each level
before the last held 10.25 B a cell that `run_tiles` never reserved. The glaciers' surface of the
next level (`LevelSink`'s `zs`, 4 B a cell) was allocated at the first tile, after the tiles'
reservation, and the cores' copy of it (`zs_core`) grew by pushes to the next power of two, 6.25 B a
cell instead of 4 (about 55 MB on quick 16k's middle level, 220 MB on full 32k's). Both are now
allocated at their size before the reservation, so `LEVEL_BYTES_PER_CELL` covers them.

**The amplifier's figure** (`amplify::estimate_under`, the stage's `Stage::estimate_under` for the
run's cap): the young landscape's world on the L2 grid's real rows (`young::FIELD_BYTES_PER_L2_CELL`,
17 f32 fields), the character's fields (`CHARACTER_BYTES_PER_L1_CELL`, 32 B: 25, 31 and 29 B held on
quick 16k at 50 and 1000 Myr and full 32k at 100 Myr), the L2 level, then each level on its real
grid (`GridDesc::for_planet`), deciding whole or tiled as the run does under the run's cap, and the
last level with `whole_last_level_peak` whole, or its largest window and the preview in tiles.

**The bake's figure** (`bake::estimate_bytes`): the largest of its parts, which run one after
another. The strata on L1, `STRATA_BYTES_PER_HORIZON_CELL` (28 B a horizon of a cell; 26.4 and 26.5
B held on quick 16k, 19.8 on full 32k, 32 horizons). The water on the hydrology's grid (L3, halved
while it is over `hydro::MAX_CELLS`: 32k full), `WATER_BYTES_PER_CELL` (28 B) on the cells it drains
plus `WATER_BYTES_PER_L1_CELL` (80 B) for the L1 climate it loads whole (587.2 MB held on quick
16k's 21.5 M cells and 0.34 M L1 cells; 658.4 MB on full 32k's 21.3 M and 1.34 M; the pair is about
7 % above both). The projection pass (`project::estimate_bytes`, Petroff-Guyou only): the grids in
turn on their real rows, the water's layers on the hydrology's grid ("L3h" where it is halved), the
closed basins' ground and flood (`GROUND_BYTES_PER_WORLD_CELL`, 12 B a world cell: 8.0 for the ground
as f64 and 3.0 for the flood, measured on quick 16k and Ben's world) and the relief
(`RELIEF_BYTES_PER_WORLD_CELL`, 6 B: its f32 change and calm flag, 5.0 held).

**The closed basins' flood** (`hydro::closed_basin_on_world_by`, `hydro::sea_reached_by`; HYD-105's
rule unchanged, the world output byte-identical). The flood kept each body's cells and a depth-first
stack as usize, and `sea_reached` pushed every open cell onto its stack at once, all grown by
doubling: 17.1 B a world cell, 29.3 B where the ocean passed 2^24 cells (the doubling's cliff, on
Ben's world). Now a state byte a cell (new, met, done) and a breadth-first front of u32 (each body is
flooded twice, the second time through its own cells to mark it done and open or not), and the sea's
reach floods from one open cell at a time: 3.0 B a world cell on both worlds (67.0 and 67.6 MB). The
bake's time did not grow (22.4 against 25.9 s on quick 16k at 50 Myr, a loaded machine).

**Measured** (`deeptime bench --threads 8 --compute cpu --mem-cap <cap>`, `DEEPTIME_MEM_TRACE`, the
large-block cache off, `DEEPTIME_LARGE_CACHE=0`, so the peak is live bytes; the bake's peak is the
trace's bake phase; MB = 10^6 B; scratch scripts not committed):

| Run (preset, start, length, cap) | Run peak | Estimate | Margin | Bake peak | Bake estimate | Margin |
|---|---|---|---|---|---|---|
| pipeline_quick_16k, scattered_continents, 50 Myr, 4 GiB | 2,411.8 | 2,662.1 | +250.3 (+10.4 %) | 1,345.6 | 1,564.1 | +218.6 (+16.2 %) |
| pipeline_quick_16k, scattered_continents, 1000 Myr, 4 GiB | 3,024.5 | 3,259.8 | +235.2 (+7.8 %) | 1,999.5 | 2,161.8 | +162.3 (+8.1 %) |
| pipeline_full_32k, supercontinent, 100 Myr, 8 GiB | 4,976.1 | 5,607.3 (amplifier 5,242.0) | +631.2 (+12.7 %; amplifier +265.9, +5.3 %) | 3,969.0 | 4,548.5 | +579.5 (+14.6 %) |
| Ben's world (earthlike, Petroff-Guyou C 13312), supercontinent, 1000 Myr, 4 GiB | 3,023.3 | 3,259.8 | +236.5 (+7.8 %) | 2,000.1 | 2,161.8 | +161.7 (+8.1 %) |

Every run's peak is the amplifier's last level; full 32k's estimate peak is the deposits stage's
(5,607.3 MB, 3,431.3 held). With the cache on (the game's default) the kept blocks fill to the
ceiling, so the counted peak sits at the estimate: 2,662.3 against 2,662.1 on quick 16k at 50 Myr,
3,259.0 against 3,259.8 on Ben's world, its bake 2,162.1 against 2,161.8. The 0.2 and 0.3 MB over
are what other threads count in the instant between a block's count and the shed that makes room
for it; they are kept blocks, not live data. Each run's 68 to 87 files and layers have the same
checksums as the same run before the change (the manifests' crc32s, against a build of the old flood).

**The margin's parts.** Of the run's 235 to 266 MB, the world as estimated is 138 MB above the world
the run holds on quick 16k at 50 Myr and 257 MB on full 32k (the stages' retained layers and the
recorded history: at 1000 Myr 629.1 MB listed for 493.0 MB held). So a cap in that band above the
whole path's need is one at which the estimate lists a level in tiles that the run, seeing its real
gauge, keeps whole; the run stays under its cap either way. The editor judges a run at its cap less
`SERVER_OVERHEAD_BYTES` (512 MiB), while the run itself uses the whole cap. Under the editor's 4 GiB
the default quick world now has 1,127 MB between its amplifier's base (2,630.6 MB) and the stage cap
(3,758.1 MB), where it had 40.0 MB.

## Lake shores in the amplifier (savanna pit task, 2026-10-22)

Ben, on deep pits in a savanna plain: "The savana pit was fine, the weird thing was all of the
straight lines, angles, etc. It didn't feel like a natural pit but instead like a glitch."

**What was wrong.** Round each pit the terrain stepped along straight lines that met at right
angles, a staircase with 8-block treads (renders of Ben's world, x -2780 to -2520, z 1040 to 1220,
and of the same seed rebaked on master: the steps sit on the slopes round each closed basin, at
the edge of the young landscape's lake).

**Cause.** The young landscape floods every closed basin to its spill point and hands the lake on
to the amplifier as a mask on its own grid (L2, 8 blocks a cell). The amplifier carried that mask
to each finer level by nearest sampling plus a 3 x 3 dilation (AMP-020 as it was), so on L3 the
lake's edge was a staircase of the L2 grid. Two things stop at that edge: the amplifier's erosion
(lake cells are not eroded) and the mountain character (`character::apply` changes land cells
only), so the edge stood in the terrain as a step. Both were found by reading the fields round the
pit (`deeptime dump-fields` of the young checkpoint) and by rebaking with the character off
(`amplify.ruggedness` 0), which leaves no step on the contour rule's edge.

**Fix.** (1) A fine cell is in the lake when its terrain lies under the highest lake surface among
the coarse corners of its interpolation quad (AMP-090), so the lake's edge is the terrain's own
contour at the lake's level, smooth and of no preferred direction; the levels carry each lake's
surface (`Level::lake_z`, 4 B a cell in place of the 1-B mask, so a level's bytes per cell in the
memory model went from 21 to 24). The windows do not carry it: a window keeps the 1-B lake flag,
and when a tile hands its core on, the next level takes each lake cell's surface from the coarse
level it was sampled from (`Level::lake_surface`, the same quad rule), so a window stays at 79 B
a cell. (The first version, merged 2026-10-22, gave the windows an f32 copy as well, 83 B a cell,
which put `pipeline_quick_16k`'s estimate over the workbench's default 4 GiB cap.) (2) The character
fades in from the shore: none on the first land cells, all of it 8 blocks out, by the Euclidean
distance to the nearest lake cell (AMP-091). With (1) alone the step was gone but a lip of 0.3 to
0.5 block was left along the contour (the change the character made on the first land cell);
with (2) the profile across the shore falls monotonically. Test:
`surface::amplify::character::tests::the_character_fades_in_from_a_lake_shore` (a lake round a
cone's foot: the shore's land cells keep their height, the flank above changes by more than half a
block on average).

**Measured** (the young checkpoint of Ben's seed on master, amplified and baked at the pinned
projection, `scale.lambda_c_deg` 120.25; master, the contour rule alone, and both; window
x -2640 to -2300, z 900 to 1180, and a second pit at x 4300 to 4560, z 3100 to 3300; scratch
scripts not committed). Profile down the slope at x -2476, z 966 to 976, blocks: master 71.7, 71.3,
71.4, 71.6, 71.1, 70.5 (a 0.3-block lip, then the stair); contour rule 71.6, 71.2, 70.8, 70.3, 70.2,
70.5 (a 0.3-block lip); both 71.6, 71.3, 71.1, 70.9, 70.8, 70.6. Against master the whole window
moved by 0.06 block on average (most 4.2, 99th percentile 1.25), the second pit's by 0.25 (most
10.2).

**The plain's terraces are not the same cause.** The contour-like terraces on the flat savanna
round the pits are the game's 1-block steps: on the plain of Ben's world (slope under 0.2, no
steep ground within 12 blocks) floor(bake height + a constant) equals the generated ground on
87.6 % of the cells, and the neighbour steps there are of exactly one block.

**Still there.** The young lake is the whole closed basin to its spill, and the amplifier adds
nothing inside it (no erosion, no character), while the bake's water balance later leaves most of
such a basin dry in a semi-arid climate. So the dry floor and the lower walls of a pit keep the
young landscape's coarser terrain, and the character's fade marks the old waterline as a soft
contour. Making the amplifier's lake the water-balance lake would end both; it is not done here.
