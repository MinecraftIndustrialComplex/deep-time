# How tall should a block be? The vertical-scale study

Measured 2026-10-02 for Deep Time, after Ben watched a flyover video and 3D previews of a baked world
and said: "I think we are scaling a bit too much vertically right now. Go get some more data, I want
worlds to feel dramatic and look cool but right now they feel a bit cartoonishly streched" (decisions
log, 2026-10-02). It changes no defaults. The pictures are in `review/out/vertical-scale/index.html`
(not committed; regenerate with §8).

**Tags.** **[C]** computed by me from a baked world or a generated Minecraft world (method in §8).
**[V]** read in this repository's code (file and line given). **[M]** from memory, not checked
against a source. **[I]** my inference, with the reasoning given.

---

## 0. The answer in plain words

- **The stretch is real and measurable.** At 20 m per block (15× exaggeration), 42 % of the land of
  a planet baked on today's code is drawn steeper than 45° and 23 % steeper than 60°. Vanilla 1.21.1
  has 15 % and 7 %; the MIC pack (Terralith with Tectonic) 15 % and 6 %. The bake Ben watched was
  gentler (25 % and 11 %), so today's code is steeper than what he saw. Every real slope over 3.8° is
  drawn steeper than 45°.
- **What the references have instead.** The pack's drama is height and broad relief at vanilla's
  slopes: land up to 209 blocks above the sea, relief of 137 blocks within 256 at the 90th
  percentile, coasts with mountains on them. Vanilla's is local cliffs.
- **Where the scales land** (today's code): 30 m per block keeps the pack's relief with 1.7 times
  its steep land; **35 m matches the references' share of land over 60° (7 %), keeps relief near
  vanilla's (108 against 115 within 256 blocks) and puts the highest ground 158 blocks above the sea**;
  40 m matches their share over 45° exactly but has less relief than either. The bake Ben saw would
  match at 30 m. A re-bake at 40 m agrees with redrawing the 20 m bake to within 0.9 blocks RMS, so
  these numbers stand for real worlds.
- **Recommendation: 35 m per block (8.6×)**, with 40 m as the calmer alternative. It needs no change
  to the world's height range or to the simulation; the costs are flatter lowlands (median land 17
  blocks above the sea instead of 32), rock layers 43 % thinner in blocks (14 % of today's thinnest
  layers merge away), ore bodies shorter, and a look at four settings that were tuned to fight the 15×
  stretch. A non-linear mapping is not worth it: it leaves coasts as steep as today. Changing the
  horizontal scale instead would shrink the planet.
- **Not fixed by any scale:** planet B's continental margins are steep in metres (63 % of the steep
  coast's box is over 45° even at 40 m), and both bakes carry one-block-wide needle spires. Both are
  simulation questions.

---

## 1. How the vertical scale works today [V]

### 1.1 One setting, one division

The vertical scale is already a setting: `scale.m_per_block_v` (`crates/deeptime-core/src/params.rs`,
`ScaleParams`, default 20 from `d_m_per_block_v`), written as 20.0 in all twelve built-in presets and
exposed in the editor as "Vertical scale per block" (`crates/deeptime-viewer/assets/editor_fields.json`,
range 1 to 200 m). The horizontal scale is its sibling `scale.m_per_block_h` (300). The simulation
runs in metres throughout; the scale enters only where metres become blocks, always as

    y = sea_level_y + (h − sea level in metres) / m_per_block_v

- **The bake's terrain.** `bake/terrain.rs` (the L3 `elevation` layer, stored in sixteenths of a
  block) and the amplifier's writer (`surface/amplify.rs`, `PackSink` with `cfg.mv`), which streams
  the finished terrain; the game reads that layer as block heights and never sees metres
  (`neoforge/.../worlddata/WorldData.java`: height = elevation + detail amplitude × noise).
- **The build-limit squeeze.** `BuildFit` (`params.rs`): terrain above a knee at
  `max_y − 16 − build_limit_band_blocks` = y 575 is compressed smoothly into the 48 blocks under
  y 623. The knee is fixed in blocks, so it sits at (575 − 63) × m_per_block_v above the sea:
  10.2 km at 20 m, 20.5 km at 40 m. No Earth-like planet reaches it at 20 m (the highest points of
  the two planets below are 5.4 and 7.0 km), so today the squeeze does nothing on Earth-like worlds.
- **Sea level** is a block height (y 63) and does not move with the scale; the sea level in metres
  comes from the simulation.
- **Rock layers.** `structure/bake_horizons.rs:762` uses `m_per_block_v / feature_scale.strata`, and
  every unit's thickness in metres is divided by it (l.116). Every baked horizon is at least one block
  thick: a thinner one merges into its thinner neighbour, and at most 32 horizons are kept per column
  (`structure/params.rs`, `horizons`). Metamorphic rock keeps at least 8 blocks and plutonic rock 16
  (`structure/columns.rs:238,241`): fixed in blocks.
- **Volcanoes and impact craters** are built in metres on the simulated surface and converted by the
  same division (`surface/volcanoes/mod.rs:1459`, `surface/impacts/mod.rs`), so a cone's height and its
  crater scale with the setting. The cone's footprint is `arc_radius_km / flank_steepness`, and
  `flank_steepness` defaults to 0.5, chosen because "heights are drawn 15 times larger than widths"
  (the editor's hint, `editor_fields.json`): a game choice made against today's scale.
- **Plutons and ore bodies** take their depths and vertical sizes in metres
  (`structure/plutons.rs:348`, `deposits/geometry.rs:57,115,164`), with floors in blocks
  (`min_y + 4`, `min_y + 16`). `deposits/survival.rs:131` drops a body whose top would be under
  `min_y + 4`: the reachable depth in metres is (surface − floor) × m_per_block_v.
- **Climate.** The lapse rate reads metres back from blocks, `(y − sea_level_y) × m_per_block_v`
  (`deeptime-format/src/climate.rs:148`, `ClimateLayers.java:153`), so snowlines follow real heights
  at any scale.
- **Regolith** (`bake/surface.rs`) in metres, capped at 15.9 blocks by its u8 layer.

### 1.2 What is fixed in blocks and would not rescale

These were tuned by eye at 20 m per block; at another scale their size in metres changes:

| Thing | Value in blocks | Where |
|---|---|---|
| The amplifier's detail term: cap | 6 blocks (relief share 0.12 rescales) | `surface/detail.rs:58,98` |
| Character pass: terraces, largest change, coast band | 8, 60, 6 blocks | `surface/amplify/character.rs:98–106` |
| Folds, faults, stocks, dykes, plugs | sizes in blocks by design ("true in block space"); turned into metres by × m_per_block_v | `structure/deform/params.rs`, `deform/mod.rs:105,253,275` |
| River channels and lakes | depth 0.5·Q^0.4, 1 to 12 blocks; freeboard 1 | `hydro/mod.rs:254,392` |
| Thinnest rock unit; metamorphic, plutonic minimum | 1, 8, 16 blocks | `bake_horizons.rs`, `structure/columns.rs` |
| Caves, aquifers, deepslate, bedrock, the lava floor (10 and 54 above the floor), magma lenses | vanilla's density functions and Deep Time's block constants | `noise_settings/planet.json`, `LavaRules.java:29`, `Magma.java:42–56` |
| World range | y −352 to 639 (the −512 floor decided 2026-10-02 is not on master yet) | `dimension_type/planet_overworld.json` |

Three game choices are metre-side multipliers made to fight the 15× stretch, and would want
re-tuning with the scale: volcano `flank_steepness` 0.5, impact `depth_scale` 0.5
(`surface/impacts/params.rs:94`) and the glacial `valley_width` 3 (`surface/glacial.rs:174`). The
decided fold `vertical_scale` of 4 (log 2026-10-02) is not built yet. One constant assumes 20 m:
`VISIBLE_RELIEF_M = 40` in `surface/impacts/mod.rs:138` ("two blocks"). One place hard-codes sea level
63 in a conversion: `deposits/mod.rs:1184`. The rest of the hard-coded 20s are defaults in viewer,
test and synthetic-world helpers (`keyframes.rs` `ViewScale::default`, `satellite.rs`, `dummy.rs`,
`Manifest.java` `DEFAULT_M_PER_BLOCK_V`).

### 1.3 What changing the default touches

The default in `params.rs`, the twelve preset files, the editor's hint, and the tests that pin
20 m (`bake_horizons.rs`, `amplify/character.rs`, `amplify/tests.rs`, `plutons.rs`,
`deposits/geometry.rs`, `params.rs:1380`, Java `GeneratorGameTests`, `NetworkGameTests`). No
migration is needed (no compatibility before a public release). The world range need not change:
the decision log fixes y −352 to 639 for every world until 1.0, and every candidate below fits in it
with room to spare.

## 2. Two baked planets, measured at each scale [C]

**The planets.** Both are the quick 16k preset (16,384 blocks round, 300 m per block across), seed 2,
supercontinent start, 1,000 Myr, baked at 20 m per block:

- **Planet A**, the bake behind the recent previews (workbench run of 2026-10-02 06:46 UTC, an earlier
  code revision: stratovolcanoes up to 3 km on 14 km × 2 = 28 km radii). Highest point 7.0 km, land
  17 % of the planet, land elevation median 530 m and 90th percentile 1,414 m.
- **Planet B**, the same preset re-baked on today's master (00ef214c). Highest point 5.2 km (young
  landscape), land 21 %, median 645 m, 90th percentile 2,266 m. It is the rougher of the two by a
  wide margin, so the code has made worlds steeper since the previews.

**How another scale is measured.** The baked terrain height H is redrawn as
`63 + (H − 63) × 20 / m` for m metres per block: the same planet in metres, drawn at m. That is a
stand-in for re-baking, and I checked it: planet B baked again at 40 m differs from planet B at 20 m
redrawn at 40 by 0.03 blocks at the median land column, 0.9 blocks RMS, 4.3 blocks at the 99th
percentile. The re-bake is slightly rougher at the block scale, because the block-fixed terms of §1.2
(the detail cap, the character pass) do not shrink: 17.7 % of land steeper than 45° against 16.4 %
redrawn, drops of 4 blocks or more 0.90 % against 0.51 %. So the redrawn numbers below understate a
real re-bake's steepness by about one percentage point at 40 m.

**What is measured** (every land column inside ±75°, about 17 million on planet A and 20 million on
B; definitions in `tools/review/vertical_scale.py`'s header): the slope between neighbouring columns
as an angle in block space (central difference over 2 blocks, so it comes in steps of half a block per
block), the share of land steeper than 45°, 60° and 75°; the "hillside" slope over 16 blocks, which is
smoother; the relief (highest minus lowest ground) within squares of 16, 64 and 256 blocks; and the
coast: the highest ground within 8 blocks of a column on the open sea's shore.

| Planet, m per block (exaggeration) | Land > 45° | > 60° | > 75° | Hillside median / p90 | Relief 64 (p90) | Relief 256 (p90) | Coast within 8 blocks (p90 / p99) | Land height p99 / max (blocks above sea) |
|---|---|---|---|---|---|---|---|---|
| A, 20 (15×), today | 25.1 % | 11.3 % | 1.8 % | 20.8° / 55.8° | 90 | 146 | 20 / 62 | 143 / 281 |
| A, 25 (12×) | 18.4 % | 7.2 % | 0.9 % | 17.4° / 49.6° | 72 | 117 | 15 / 49 | 114 / 225 |
| A, 30 (10×) | 13.7 % | 4.8 % | 0.5 % | 14.0° / 44.1° | 59 | 98 | 12 / 41 | 95 / 188 |
| A, 35 (8.6×) | 10.4 % | 3.2 % | 0.2 % | 11.2° / 39.4° | 51 | 84 | 9 / 35 | 81 / 161 |
| A, 40 (7.5×) | 8.0 % | 2.2 % | 0.1 % | 10.6° / 35.5° | 44 | 73 | 7 / 30 | 71 / 141 |
| A, 50 (6×) | 4.9 % | 1.1 % | 0.04 % | 8.0° / 29.5° | 35 | 59 | 4 / 24 | 56 / 113 |
| A, 60 (5×) | 3.1 % | 0.6 % | 0.01 % | 7.1° / 24.5° | 29 | 49 | 3 / 20 | 47 / 94 |
| A, 80 (3.75×) | 1.3 % | 0.2 % | 0 | 5.1° / 18.6° | 21 | 37 | 1 / 15 | 35 / 71 |
| B, 20 (15×), today | 42.1 % | 22.6 % | 3.9 % | 34.6° / 65.7° | 128 | 190 | 24 / 67 | 179 / 276 |
| B, 25 (12×) | 33.1 % | 15.3 % | 1.8 % | 28.3° / 60.4° | 102 | 152 | 19 / 54 | 143 / 221 |
| B, 30 (10×) | 26.0 % | 10.4 % | 0.9 % | 23.8° / 55.6° | 85 | 126 | 15 / 45 | 119 / 184 |
| B, 35 (8.6×) | 20.6 % | 7.1 % | 0.4 % | 20.6° / 51.1° | 72 | 108 | 12 / 38 | 102 / 158 |
| B, 40 (7.5×) | 16.4 % | 4.9 % | 0.2 % | 17.4° / 46.9° | 63 | 95 | 10 / 33 | 89 / 138 |
| B, 40, re-baked | 17.7 % | 5.7 % | 0.5 % | 17.4° / 47.9° | 65 | 99 | 10 / 35 | 89 / 143 |
| B, 50 (6×) | 10.4 % | 2.4 % | 0.1 % | 14.0° / 40.4° | 50 | 75 | 7 / 27 | 71 / 111 |
| B, 60 (5×) | 6.7 % | 1.2 % | 0.02 % | 11.2° / 34.9° | 41 | 63 | 5 / 22 | 59 / 92 |
| B, 80 (3.75×) | 2.9 % | 0.3 % | 0 | 8.0° / 26.7° | 31 | 47 | 3 / 16 | 44 / 69 |

Relief within 16 blocks (median / p90 / p99): A 9 / 30 / 67 at 20 m, 4 / 15 / 33 at 40 m; B 15 / 44 / 80
at 20 m, 7 / 21 / 39 at 40 m. The share of land columns level with all four neighbours rises from
31 % (A) and 20 % (B) at 20 m to 52 % and 39 % at 40 m and 73 % and 62 % at 80 m.

In real terms the planets are not steep. Planet A's 90th-percentile slope between columns, 61° at
15×, is a real slope of 6.9° over 600 m; its 99th percentile, 77.5°, is 16.8°. Everything steeper than
about 3.8° real is drawn steeper than 45° at 20 m per block (5.7° at 30 m, 7.6° at 40 m, 9.5° at
50 m). That, more than any one feature, is the stretched look: ordinary hill country is drawn as walls.

## 3. Against vanilla and the pack's terrain [C]

**The references.** Five squares of 1,536 × 1,536 blocks (centres 0,0 and ±5,000 on each axis,
about 9,400 chunks each, 47,000 chunks per world) generated by a headless 1.21.1 server with Chunky,
level type `minecraft:normal`, seed 20261002: once with vanilla's generator (the smoke set
`standalone`) and once with the MIC pack's (Terralith 2.6.2 with Tectonic 3.0.28, which reshapes the
terrain itself: its `global_terrain.vertical_scale` 1.125, world top y 320). Terrain tops were read
from the region files with `deeptime-inspect heights` (trees, plants, snow layers and fluids
skipped) and measured with the same code as the planets, without wrapping and away from the squares'
edges. 11.8 million columns each. The squares sit around spawn, so they hold more land (vanilla 58 %)
than a planet does; the statistics are over land only.

| World | Land > 45° | > 60° | > 75° | Hillside median / p90 | Relief 64 (p90) | Relief 256 (p90) | Coast within 8 blocks (p90 / p99) | Land height p99 / max | Drops ≥ 8 blocks |
|---|---|---|---|---|---|---|---|---|---|
| Vanilla 1.21.1 | 15.1 % | 7.0 % | 1.7 % | 14.5° / 41.6° | 54 | 115 | 14 / 44 | 75 / 137 | 0.99 % |
| The MIC pack (Terralith + Tectonic) | 14.8 % | 6.4 % | 1.4 % | 12.7° / 41.8° | 53 | 137 | 41 / 163 | 161 / 209 | 0.59 % |
| Planet A at 35 m | 10.4 % | 3.2 % | 0.2 % | 11.2° / 39.4° | 51 | 84 | 9 / 35 | 81 / 161 | 0.01 % |
| Planet B at 35 m | 20.6 % | 7.1 % | 0.4 % | 20.6° / 51.1° | 72 | 108 | 12 / 38 | 102 / 158 | 0.02 % |
| Planet A at 30 m | 13.7 % | 4.8 % | 0.5 % | 14.0° / 44.1° | 59 | 98 | 12 / 41 | 95 / 188 | 0.03 % |
| Planet B at 40 m | 16.4 % | 4.9 % | 0.2 % | 17.4° / 46.9° | 63 | 95 | 10 / 33 | 89 / 138 | 0.01 % |
| Planet B at 20 m (today) | 42.1 % | 22.6 % | 3.9 % | 34.6° / 65.7° | 128 | 190 | 24 / 67 | 179 / 276 | 0.37 % |

The charts on the review page plot every measure against the scale with the references as lines.

**Where Deep Time sits.**

- **Slopes.** Vanilla and the pack agree almost exactly on how steep their land is: 15 % of land
  steeper than 45°, 6.4 to 7 % steeper than 60°, hillsides of 13 to 15° at the median and 42° at the
  90th percentile. Today's planet B is 2.8 times vanilla on the 45° share and 3.2 times on the 60°
  share; planet A, the one Ben saw, 1.7 and 1.6 times. Planet B comes down to the references at
  40 m per block (16.4 %, or 17.7 % re-baked; 4.9 %); planet A at 30 m (13.7 %, 4.8 %). At 35 m
  they straddle them: A 10.4 % and B 20.6 % over 45°, A 3.2 % and B 7.1 % over 60°.
- **Relief and height.** The pack is where the drama is: relief within 256 blocks of 137 at the 90th
  percentile (vanilla 115), coasts with ground 41 blocks above the sea within 8 blocks at the 90th
  percentile and 163 at the 99th (Tectonic's coastal mountains; vanilla 14 and 44), land up to 209
  blocks above the sea (vanilla 137). It gets that height with the same slopes as vanilla, so its
  mountains are broad. Planet B at 40 m has relief 95 and a highest point 138 blocks above the sea
  (vanilla's 137); at 35 m 108 and 158; at 30 m 126 and 184, near the pack's relief but with 1.8
  times its steep land.
- **Mountain regions.** The highest 96 km box of each reference has 35 % (vanilla, peak 134 blocks
  above the sea) and 29 % (the pack, peak 209) of its ground steeper than 45°. Planet B's range box
  has 61 % at 20 m, 34 % at 30 m, 24 % at 35 m and 18 % at 40 m (peaks 239, 159, 137 and 120 blocks).
  So for the mountains alone, 30 to 35 m matches the references, and 40 m is gentler than their
  mountains.
- **Coasts.** Every Deep Time scale from 30 m up has lower coasts than the pack (whose coasts are
  dramatic by design) and from 35 m up lower than vanilla at the 90th percentile.

**A difference in kind.** Vanilla's drama is local: 1 % of its land is beside a drop of 8 blocks or
more (cliff faces, jagged peaks, river banks) against 0.01 to 0.4 % for Deep Time at any scale, and
its steepest hundredth is as steep as Deep Time's at 20 m (1.7 % over 75°). Deep Time's drama is
broad: whole mountainsides and margins at one steep angle. So at a scale where the broad statistics
match vanilla, Deep Time reads smoother up close; the amplifier's character pass (cliff bands on
hard rock, sharp young peaks, `amplify.ruggedness`) is the place to add local drama back, in blocks,
independent of the scale.

## 4. Pictures [C]

`review/out/vertical-scale/index.html` (planet B, with the references) and `a/index.html` (planet A),
not committed. For each planet the same five places at 20, 30, 35, 40, 50, 60 and 80 m per block and
through the two non-linear mappings of §6.2, side by side, each an oblique view from the south of a
320 × 320-block box (96 km), one pillar per column, colours by real elevation, with the numbers
beside each picture; and a west-to-east section of each place drawn one block to one block, every
scale on one plot. The places, chosen by `vertical_scale.py places`:

- the stratovolcano that stands highest above the ring at its base radius, and a second one;
- the highest 64-block cell of land away from volcanoes (a range or collision belt);
- the 128-block cell with most shoreline under ground 16 or more blocks above the sea (a steep
  coast and continental margin);
- the 128-block cell with most land 12 or more blocks below a closing over 33 blocks (incised
  valleys).

The flanks, as the steeper side of the section between 20 % and 80 % of the height:

| Place | 20 m | 30 m | 35 m | 40 m | 50 m | 60 m | 80 m |
|---|---|---|---|---|---|---|---|
| A, stratovolcano on a high base (peak 281 blocks above the sea at 20 m) | 79° | 73° | 71° | 68° | 63° | 59° | 52° |
| A, second stratovolcano (133 blocks) | 51° | 40° | 36° | 32° | 27° | 23° | 17° |
| B, stratovolcano (197 blocks) | 74° | 67° | 63° | 60° | 54° | 50° | 41° |
| B, second stratovolcano (157 blocks) | 55° | 44° | 40° | 36° | 30° | 26° | 20° |

And the share of each 96 km box steeper than 45°:

| Place | 20 m | 30 m | 35 m | 40 m | 50 m | 60 m | 80 m |
|---|---|---|---|---|---|---|---|
| B, range (peak 239 blocks at 20 m) | 61 % | 34 % | 24 % | 18 % | 10 % | 6 % | 2 % |
| B, steep coast and margin | 84 % | 74 % | 68 % | 63 % | 52 % | 41 % | 24 % |
| B, incised valleys | 52 % | 34 % | 28 % | 23 % | 16 % | 11 % | 5 % |
| A, range | 54 % | 39 % | 34 % | 31 % | 25 % | 20 % | 13 % |
| A, steep coast and margin | 71 % | 50 % | 41 % | 35 % | 25 % | 18 % | 10 % |
| Vanilla, its highest box (peak 134) | 35 % | | | | | | |
| The pack, its highest box (peak 209) | 29 % | | | | | | |

Two things the pictures show that the statistics do not. First, the steep coast of planet B stays a
wall at every scale: 63 % of its box is steeper than 45° even at 40 m. Its margin is steep in metres
(the simulation's shelf edge, not the drawing), so the vertical scale cannot fix it alone. Second,
both bakes carry single-column spires, needles 20 to 50 blocks tall one block wide, on steep ground
(visible in the coast and range pictures of planet A); they shrink with the scale but stay needles.
They look like an amplifier artefact and are worth a separate look.

## 5. What each choice costs [C, from §1 and the baked rock layers]

| At m per block | 20 | 30 | 35 | 40 | 50 | 60 | 80 |
|---|---|---|---|---|---|---|---|
| Vertical exaggeration | 15× | 10× | 8.6× | 7.5× | 6× | 5× | 3.75× |
| Mount Everest, 8,849 m: blocks above the sea | 442 | 295 | 253 | 221 | 177 | 147 | 111 |
| Planet A's highest point, 7.0 km (y) | 351 (y 414) | 234 (297) | 201 (264) | 176 (239) | 141 (204) | 117 (180) | 88 (151) |
| A 3 km volcano: blocks tall | 150 | 100 | 86 | 75 | 60 | 50 | 38 |
| A real 6° volcano flank is drawn at | 58° | 46° | 42° | 38° | 32° | 28° | 21° |
| A real 18° slope is drawn at | 78° | 73° | 70° | 68° | 63° | 58° | 51° |
| The real slope drawn at 45° | 3.8° | 5.7° | 6.7° | 7.6° | 9.5° | 11.3° | 14.9° |
| A 1 km canyon: blocks deep | 50 | 33 | 29 | 25 | 20 | 17 | 12.5 |
| A 100 m river valley: blocks deep | 5 | 3.3 | 2.9 | 2.5 | 2 | 1.7 | 1.2 |
| A 100 m rock layer: blocks thick | 5 | 3.3 | 2.9 | 2.5 | 2 | 1.7 | 1.2 |
| Baked rock units under one block (sampled) | 0 % | 14 % | 14 % | 14 % | 27 % | 27 % | 34 % |
| Median baked rock unit: blocks thick | 9 | 6 | 5.1 | 4.5 | 3.6 | 3 | 2.3 |
| Ore 1.5 km down: blocks below the surface | 75 | 50 | 43 | 38 | 30 | 25 | 19 |
| Abyssal plain, 4 km deep (y) | −137 | −70 | −51 | −37 | −17 | −4 | 13 |
| Trench, 10 km deep (y) | −437 | −270 | −223 | −187 | −137 | −104 | −62 |
| Depth from sea level to the floor at y −352 | 8.3 km | 12.5 km | 14.5 km | 16.6 km | 20.8 km | 24.9 km | 33.2 km |

**World height.** Nothing needs to change. The range is fixed at y −352 to 639 until 1.0 (decision
table); every scale from 30 m up fits the tallest Earth-like terrain far under the top and the deepest
trench above y −352. The decision of 2026-10-02 to lower the floor to −512 was made because trenches
reached y −412 to −472 at 20 m per block; at 30 m and above they reach about −270, so that change is
not needed if the scale moves, and the world keeps 160 fewer blocks of solid rock under every column.
At 40 m the terrain uses about y −190 to 240 and leaves 400 blocks of air above it for building.

**Rock layers.** From 1,500 random land columns of planet A, 22,324 baked units between the top block
and the floor (median 13 per column): 14 % are one block thick at 20 m, 27 % at most two, 34 % at most
three. At 30 or 40 m the one-block units fall under a block and merge into their neighbours; at 50 or
60 m the two-block ones too; at 80 m a third of today's units are gone. The thin units are the
interesting ones: chalk, tuff, ironstone, limestone and diamictite have a median of one block today,
coal, mudstone, anhydrite, red beds and halite two to three, so at 40 m they become rarer and thinner
first. Units that were already under 20 m thick merged at today's scale and are not counted. The
existing setting `scale.feature_scale.strata` divides the scale for the rock alone
(`bake_horizons.rs:762`): at 40 m with strata 2, every layer keeps today's thickness in blocks, at the
cost of exposing fewer layers on a hillside of the same height in blocks.

**Mining and ore.** The world's depth in blocks does not change (about 430 blocks from a lowland
surface to the floor); it holds twice the depth in metres at 40 m (16.6 km below the sea instead of
8.3), so ore that formed deep (orogenic gold, porphyry roots) is inside the world more often, and
every ore body is nearer the surface in blocks and half as tall (`deposits/geometry.rs:57`:
`size_mult_v / m_per_block_v`). `deposits.size_mult_v` or `feature_scale.deposits` can restore the
drawn height. Caves, aquifers and the lava floor stay as they are in blocks.

**Lowlands.** The cost of a larger number is flatness where people live: on planet B the median land
column is 32 blocks above the sea at 20 m, 15 at 40 m, 6 at 80 m; a 100 m river valley is 2.5 blocks
deep at 40 m. River channels (1 to 12 blocks deep, fixed in blocks) become relatively deeper against
their valleys. The climate follows real heights, so snowlines come down in blocks (3 km is y 213 at
20 m, y 138 at 40 m), which looks more like vanilla's snowy peaks.

**Settings tuned against 15×.** At 7.5× some of today's compensations become unnecessary or
excessive: `volcanoes.flank_steepness` 0.5 (cones twice as wide as real "because heights are drawn 15
times larger"), impacts `depth_scale` 0.5, glacial `valley_width` 3, the folds' decided
`vertical_scale` 4. Each would want one look at 40 m; flank steepness 1 (the real profile) at 40 m
draws a real 6° flank at 38°, steeper than today's widened cones at 20 m and still well short of a
spike.

## 6. Other ways to get there

### 6.1 A different horizontal scale

The stretch is the ratio of the two scales, 300 / 20. Halving the horizontal scale to 150 m per block
at 20 m up gives the same slopes as 40 m up, but makes the 16k planet half as large in kilometres
(2,458 km round), doubles the width in blocks of every feature drawn in metres (a 28 km volcano becomes
187 blocks in radius), and halves what the same simulation covers. The vertical scale is the cheaper
lever: one division in the bake, no change to the simulation or to how far apart things are.

### 6.2 A non-linear vertical mapping [C]

"True scale near sea level, compressed above": metres above the sea map to blocks with 20 (or 30) m per
block near the sea and 60 (or 80) m per block above about 1 (or 1.5) km, blended smoothly. Measured on
both planets:

| Planet B | Land > 45° | > 60° | Hillside p90 | Relief 64 (p90) | Coast (p90 / p99) | Land height p99 / max |
|---|---|---|---|---|---|---|
| 20 → 60 m above 1 km | 26.3 % | 10.0 % | 55.0° | 72 | 23 / 54 | 94 / 126 |
| 30 → 80 m above 1.5 km | 17.9 % | 5.3 % | 48.4° | 59 | 15 / 41 | 76 / 101 |
| linear 30 m, for comparison | 26.0 % | 10.4 % | 55.6° | 85 | 15 / 45 | 119 / 184 |
| linear 40 m | 16.4 % | 4.9 % | 46.9° | 63 | 10 / 33 | 89 / 138 |

It compresses the mountains but not the coasts: the coast numbers of the 20 → 60 mapping are today's
(23 against 24 blocks at the 90th percentile), because coasts sit where the mapping is still 20 m. The
slope statistics come out like a linear 30 m (20 → 60) or 40 m (30 → 80), with lower peaks. Against
real shapes it is wrong in a visible way: a cone rising from the sea is drawn steep at its foot and
gentle at its top (convex, a dome), where a real stratovolcano is gentle at the foot and steep at the
top (concave); see the non-linear pictures and the volcano sections. For the rock, every conversion
from metres to blocks (about a hundred lines in some 25 files, plus the climate's inverse in Rust and Java)
would go through the mapping and its inverse; a bed of 100 m would be 5 blocks thick near the sea and
under 2 at 3 km, and an ore body's drawn height would depend on its elevation. Much more work for a
result between two linear scales, and worse on coasts and cones.

### 6.3 Fixing features one at a time at 20 m

Widening cones (the question that started this), widening glacial valleys and lowering fold
amplitudes each fix one feature and leave the rest stretched: ranges, valleys and margins are not
features with a width setting, and they are most of the 42 % of planet B's land that is drawn steeper
than 45°.

### 6.4 The margins themselves

Planet B's steep coast stays steep at every scale (§4). A real continental slope averages about 4°
[M], drawn at 46° at 15× and 28° at 7.5×; the baked margins are steeper than that in metres. That is
a simulation question for the margin and flexure work, not for the drawing.

## 7. Options for Ben

Each option changes `scale.m_per_block_v` in the presets and the code default, nothing else
until the follow-ups below; the world range stays y −352 to 639.

1. **35 m per block (8.6×). Recommended.** On today's code (planet B) the share of land steeper than
   60° equals the references' (7.1 % against 6.4 to 7.0 %), relief within 256 blocks is near
   vanilla's (108 against 115), mountain regions are a little gentler than the references' mountains
   (24 % of the range box over 45°, against 29 to 35 %), and the highest ground stands 158 blocks above
   the sea (vanilla 137, the pack 209). The share over 45° is still 1.4 times the references'
   (20.6 %, about 22 % re-baked): the land stays a little steeper than vanilla, which is the drama
   Ben asked to keep. Planet A, the bake he saw, lands just under the references at 35 m, so the
   scale is right for both bakes the code has produced, not only one. Everest would be 253 blocks
   tall; a 3 km volcano 86.
2. **40 m per block (7.5×).** The calmest that still matches: planet B's slopes then equal the
   references' planet-wide (16.4 %, 17.7 % re-baked, against 15 %), but its relief (95) and its
   mountain regions (18 % of the range box over 45°) are below both references, and planet A becomes
   clearly tamer than vanilla (8 % of land over 45°). Choose this if 35 m still reads stretched in the
   game.
3. **30 m per block (10×).** Relief and peak heights like the pack's (126 and 184 on planet B), but
   1.7 times the references' steep land and 1.5 times their land over 60°. The mountains alone match
   the references best here (34 % of the range box), so this is the dramatic end; it is still
   noticeably stretched on today's code.
4. **Keep 20 m and fix features one at a time** (wider cones, wider glacial valleys, lower folds).
   Leaves ranges, valleys and margins stretched: 42 % of planet B's land over 45°, three times the
   references. Not recommended.
5. **A non-linear mapping** (§6.2). Not recommended: it leaves coasts as steep as today, draws cones
   upside down in profile, and touches every metres-to-blocks conversion.

**The honest trade-off.** A larger number makes the lowlands flatter (the median land column 32
blocks above the sea today on planet B, 17 at 35 m), river valleys and canyons shallower, rock layers
thinner (14 % of today's units fall under a block at 30 to 40 m and merge away) and ore bodies
shorter; and the two planets show that the right number moves with the code: planet B, two code
revisions after planet A, needs about 10 m more per block for the same slopes. A scale picked now
will want one more look when the plate and margin work lands.

**Follow-ups if the scale moves** (none done here): look again at the settings tuned against 15×
(`volcanoes.flank_steepness` 0.5, impacts `depth_scale` 0.5, `surface.glacial.valley_width` 3, the
folds' `vertical_scale` 4 still to be built) and at `VISIBLE_RELIEF_M`; decide whether rock layers
follow the scale or keep today's thickness through `scale.feature_scale.strata`; the floor at −512
is not needed from 30 m up; the editor's hint for the slider; and two problems the scale does not
fix, the steep margins (§6.4) and the needle spires (§4).

## 8. Method and reproduction

Everything ran on the test machine; nothing booted Minecraft except the two reference worlds (under
the shared server lock). The scripts are `tools/review/vertical-scale.sh` and
`tools/review/vertical_scale.py` (committed), and `deeptime-inspect heights` (committed with them).

1. `vertical-scale.sh planet <world-data> <name>`: the baked terrain height and lake level of every
   column, from `deeptime heightmap` in strips of 1,024 rows.
2. `vertical-scale.sh run 40` re-bakes the quick 16k preset at 40 m per block (seed 2, supercontinent,
   1,000 Myr); `vertical_scale.py compare` checks the redrawing against it.
3. The references: `tools/smoke/run.py --profile standalone|mic --fresh --level-type minecraft:normal
   --chunky 768 --chunky-at 5000,0,768 --chunky-at -5000,0,768 --chunky-at 0,5000,768 --chunky-at
   0,-5000,768`, then `deeptime-inspect heights` on each square and `vertical_scale.py refjoin`.
4. `vertical_scale.py stats` (scales 20 to 80 and the two mappings `knee:20:60:1000:300` and
   `knee:30:80:1500:400`), `places`, `strata` (1,500 land columns through `deeptime inspect-column`),
   `render`, `charts` and `page`.

## 9. What I could not verify

- **Two planets, one preset, one seed.** A and B differ more from each other than neighbouring
  scales do; other seeds, starts and lengths will sit elsewhere. The recommendation is made on B,
  today's code.
- **The references are five squares near spawn of one seed.** Vanilla and pack terrain vary a lot by
  region; another seed would move the reference lines by a few points.
- **Statistics are not the feel.** Nobody has walked a world at 40 m per block. The pictures are a
  heightmap renderer, not the game: no trees, no biome surface, no caves opening on hillsides.
- **The redrawing is a stand-in for re-baking.** Checked at one scale (40 m) on one planet; a real
  re-bake is about one point steeper on the steep shares (§2).
- **Rock layers** were sampled on planet A only; units already merged at 20 m are invisible to the
  count.
- **Real-world angles** (6° stratovolcano average, 18° slope, 4° continental slope) are from memory,
  not checked against a source.
- **The needle spires** (§4) were seen in the pictures and not traced to their cause.
