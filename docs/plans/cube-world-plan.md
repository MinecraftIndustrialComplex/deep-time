# Cube world plan

Written 2026-10-22 by a sub-agent of the Deep Time coordinator (session
f981c0d3-5144-49ec-8686-2946d755872c), branch `worktree-agent-a29cad658c6c26bf2`. This is a plan
only: no shipped code changes with it.

**The decision.** Ben, 22 October (decisions log, second 2026-10-22 entry): "Fuck it, remove equal
are restriction and move to a cube." The world moves from Petroff-Guyou to a cube, V1 included.
Equal area is no longer a strict rule: the area per block may vary within a bound on land, and shapes
should be as true as possible. The erosion, river and sediment solvers move onto the cube's faces
with the world, and the climate grid uses the same cube map.

**What the coordinator already decided** (undoable, under Ben's rule of 2026-10-16):

- The map on the cube is picked by the running study (branch `worktree-agent-a996493d601a564e5`,
  `docs/research/notes/mountains-and-the-projection.md`) from measured land numbers.
- The layout is the survey's cross net: four faces round the equator wrapping east-west through
  Toroidal World, one face on each pole.
- The world keeps today's blocks per square km.
- V1 ships on the cube.
- Fix 6 (D1, volcanoes drawn round) is dropped if the chosen map draws shapes true or nearly so.
- The D5 multiplayer world goes ahead on Petroff-Guyou, and another fresh world comes with the cube.

**Revision, 2026-10-22: the study's pick** (`docs/research/notes/mountains-and-the-projection.md`
§5, merged c6a20ce8; taken as the coordinator's call until Ben answers, see section 8's Q4). Where
this block and the text below disagree, this block wins:

- **The world: the conformal cube (τ = 0), turned freely.** Shapes on land are true (stretch at most
  1.02). Land is drawn 0.79 to 1.92 times its true area on the worst of 15 planets. The corners lie
  5.28° to 9.57° out at sea. Fix 6 (D1) is dropped.
- **The simulation: its own cube, the equal-area Moser map (τ = 1) on the polar axis.** The
  simulation's corners cannot be kept at sea, because continents cross every point in 1,000 Myr. On
  the conformal map a corner cell at n = 240 has 0.044 of the mean area (§1.6's "0.16" is a length
  ratio), so explicit steps would be 31 times shorter. Moser's corner cell is 0.91 of the mean. The
  bake composes world block, face (u, v), conformal map, the rotation, the Moser inverse, and then
  the simulation cell.
- **B3 is replaced by the study's §5.4.** A free rotation search (seeded random rotations, then a
  shrinking refinement) maximises the smallest signed distance from the 8 corners to land. The cube's
  24 symmetries then fix the rotation, with the north pole in face N and the most land on F0. The
  manifest carries the rotation as a 3 × 3 matrix in place of `lambda0_deg`. The poles are no longer
  at the face centres: latitude does not run along the net, and -z is not north. The fallback is a
  tilt limit (pole at most 30° from face N's centre: land × 2.05, corners 4.58° out at worst).
- **Phase 0 commits two tables**, the conformal table for the world and the Moser table for the
  simulation, with the conformal corner cells handled (study §5.2: the map behaves like a 4/3 power
  there, and the bicubic may fold).
- **Metres to blocks uses the local σ everywhere.** Land runs from 213 to 331 m per block on the
  worst planet, and the median is about 310 m per block at s = 3,840. The face size stays.
- **Detail noise** is sampled at the world block, not the planet point (study §6). The seamless
  projection survey's line 249 says otherwise and is wrong. G3 picks either a partition of unity
  across turning seams or 3D noise at the planet point. (G3 took a third way, 2026-10-23: value
  noise on a lattice laid on the cube's surface, SCL-215; see G3.)

**How to read this plan.** Each work item has an id (W, M, B, S, G, T, E, D) and a "Check" line,
the acceptance check that closes it. Section 7 orders the items into phases and lanes, and section 8
holds the questions only Ben can answer. Nothing here estimates how long anything takes: scope is
described by what gets built.

## 0. Summary

**What changes.** The world becomes a cross of six square faces of s blocks a side: s = 3,840 at
16k and 7,424 at 32k, both multiples of 256. Four band faces run east-west round the equator and
loop through Toroidal World; one face sits above and one below the band's first face, and the poles
are at their centres. Every seam of the cube that the net cuts is glued by a slide (east-west), a
quarter turn (eight edges) or a half turn (four edges).

Until the wrap engine joins the turning seams, a border 1 block inside the 12 net edges that face
empty space keeps players on the planet. Every point of the planet, both poles included, is reachable
on foot through the band face the polar faces join.

**How the world is made.** One "map slot" turns face coordinates into planet points and back, with
its Jacobian. The candidate maps are conformal, Moser's flow taken part way, and the equal-area Moser
cube. The slot is one table shipped in the code and read identically in Rust, Java and JS. The plan
is written so that any of the three candidates fits it, and section 1.6 lists what each changes.

The bake projects every world cell through the map slot from the simulation's frame. A search over
the cube's turn about the polar axis (λ0) puts the eight cube corners at sea. The simulation's
solvers move from the Lambert raster to the cube's faces with ghost rings and the true metric.

**Phases** (section 7):

| Phase | What lands | Gate |
|---|---|---|
| 0 | The cube's geometry and the map slot in Rust, Java and JS, with the table and the 0.7 format written down | Parity fixture bit-identical in three languages; fold and seam identities; table round trip and convergence |
| 1 | The bake writes cube worlds from today's simulation; synthetic cube worlds for tests | earthlike_quick_16k baked to the cube; land share, corner distances, river and body checks; review page |
| 2 | The game on the cube: readers, generator, the fill outside the planet, borders, spawn, F3, climate API 5, Create World screen, dev map; the Lambert and Petroff-Guyou world shapes deleted | cargo, Gradle, both GameTest batches, smoke 16k and 32k, DH client check, screenshots |
| 3 | Tools, viewer, workbench and the docs on the cube (runs beside phase 2) | viewer checks, preview runs, science pages and ledger |
| 4 | The simulation on cube faces (its own lane from phase 0 on) | Determinism, re-baselined seed tests, Earth checks, A/B time against Lambert, the 900 s gate tracked |
| 5 | V1: the fresh play world on the cube; the Lambert simulation grid deleted | Play-server checks |
| 6 | The wrap engine joins the turning seams (scheduled by Ben, question Q3) | The engine's phases 2 to 5 |

**Questions for Ben** (section 8). Each lists the recommended option first:

1. **The data format 0.7.** Recommended: rasters stored face by face with no void cells, the map
   table shipped in the code, and its hash in the manifest. The alternatives are rasters over the
   whole net box with void cells, or the table carried in every world.
2. **Climate API 5** (the public interface mic-climate reads). Recommended: keep `latitudeDeg`,
   `longitudeDeg`, `projectionKind` (new value `"cube"`) and `circumferenceBlocks` (the loop period
   4s); add `northBearingDeg` and `metresPerBlock`; drop the static Lambert-only
   `latitudeDeg(z, C)`.
3. **When the wrap engine is scheduled.** Recommended: after V1 ships on borders. The alternatives
   are now, in parallel with V1, or after the public release.

## 1. The world

### 1.1 Face size and scale

The face side is s = 256 · round(C / (256 · √(6π))), where C is the preset's notional circumference
(16,384 or 32,768). The planet radius stays C · 300 m / 2π (ledger SCL-004, SCL-005). The six faces
cover the planet's area, so the nominal metres per block is 300 · (C / √(6π)) / s.

The table uses these terms:

- "Today" means the Petroff-Guyou world today.
- "Turning seam length" is the total length of the net edges that join by a turn.

| | 4k test planet | 16k | 32k |
|---|---|---|---|
| Exact side C/√(6π) | 943.4 | 3,773.7 | 7,547.4 |
| Face side s | **1,024** (64 chunks) | **3,840** (240 chunks) | **7,424** (464 chunks) |
| Nominal m per block | 276.4 | 294.8 | 305.0 |
| Today's m per block | 319.2 (b 1,536) | 294.6 (b 6,656) | 300.4 (b 13,056) |
| East-west period 4s | 4,096 | 15,360 | 29,696 |
| Net box 4s × 3s | 4,096 × 3,072 | 15,360 × 11,520 | 29,696 × 22,272 |
| Planet area 6s² (blocks²) | 6.3 M | 88.5 M | 330.7 M |
| Today's area 2b² | 4.7 M | 88.6 M | 340.9 M |
| Turning seam length 6s | 6,144 | 23,040 | 44,544 |

**Why 256.** It is the same quantum as SCL-084, for the same reasons:

- every grid's r (up to 256 blocks per cell) divides s;
- the period 4s is a multiple of 512, which is a region file and what WorldDataRef and Toroidal
  World's loop take;
- the Nether's 1:8 loop (4s/8 = 1,920 at 16k, 120 chunks) stays whole;
- every turn pivot (section 1.2) sits on a chunk corner.

The survey's 3,776 (a multiple of 16) would match the area more closely at 16k, but it breaks the
first two of these.

At 32k the nearest multiple of 256 is 7,424. That is 3% fewer blocks per square km than today's
300.4 m; the next one up, 7,680, would be 3.8% more. Taking the nearest multiple keeps the rule one
line, which is the coordinator's call.

**The 4k test planet.** Today it is Petroff-Guyou with b 1,536 (3,072 × 1,536, z edge 768). It
becomes a cube with s 1,024, so its period changes from 3,072 to 4,096.

**The work.**

- **W1.** Add `face_for(C)` beside `hemisphere_for` in `crates/deeptime-format/src/projection.rs`, with
  `FACE_QUANTUM = 256` (new ledger row; SCL-078's value changes from 3,776 to 3,840). Mirror it in
  Java `WorldDataRef` and in JS.
  Check: a unit test in each language gives 1,024, 3,840 and 7,424 for 4,096, 16,384 and 32,768.

### 1.2 The net

Coordinates use +x east and -z north, as today.

| Face | x range | z range | Centre (blocks) | Planet point at the centre |
|---|---|---|---|---|
| F3 | [-3s/2, -s/2) | [-s/2, s/2) | (-s, 0) | equator, λ0 - 90° |
| F0 | [-s/2, s/2) | [-s/2, s/2) | (0, 0) | equator, λ0 |
| F1 | [s/2, 3s/2) | [-s/2, s/2) | (s, 0) | equator, λ0 + 90° |
| F2 | [3s/2, 5s/2) | [-s/2, s/2) | (2s, 0) | equator, λ0 + 180° |
| N | [-s/2, s/2) | [-3s/2, -s/2) | (0, -s) | north pole |
| S | [-s/2, s/2) | [s/2, 3s/2) | (0, s) | south pole |

The table's planet points use the simulation's longitudes. The world's displayed longitude is 0 at
x = 0, as the dev map's `displayLon` convention is today.

At 16k, s/2 = 1,920, 3s/2 = 5,760 and 5s/2 = 9,600.

```
 x:   -3s/2     -s/2      s/2      3s/2      5s/2
z -3s/2 +--------+--------+--------+--------+
        |  void  |   N    |  void  |  void  |
z  -s/2 +--------+--------+--------+--------+
        |   F3   |   F0   |   F1   |   F2   |   (F2's east edge = F3's west edge, period 4s)
z   s/2 +--------+--------+--------+--------+
        |  void  |   S    |  void  |  void  |
z  3s/2 +--------+--------+--------+--------+
```

**The equator and the poles.**

- The equator is the line z = 0 across the band.
- Walking -z from F0 crosses into N and reaches the north pole at (0, -s). Walking on in the same
  straight line, you are now heading south down the meridian λ0 + 180°.
- The pole is an ordinary point of N. There is no cap and no storm.

**Toroidal World.**

- PlanetShape's loop becomes 4s wide (`LoopSpans.ofWidth(X, 4s/16)`). The Nether keeps 1:8 and Z is
  not looped.
- TW's own canonical window is centred on 0, so its seam (x = ±2s) runs down the middle of F2. TW's
  seam is a pure slide and cannot be seen.
- The fold writes everything in the net window [-3s/2, 5s/2) and adds the 4s slide where TW's
  window cuts F2. `WrapLayouts.cubeCross` already composes the slide for its west edges.
- **Why F0 is centred on x = 0:** the world origin, vanilla's spawn search and the world border's
  centre then sit at the middle of F0 on the equator.

**Seams.** The cube has 12 edges. The net joins 5 of them directly: F3|F0, F0|F1, F1|F2, N|F0 and
S|F0. It cuts the other 7, each into a pair of net edges:

| Seam | Net edges | Motion | Pivot or shift | Length |
|---|---|---|---|---|
| East-west | F2 east (x = 5s/2) and F3 west (x = -3s/2) | slide (Toroidal World) | 4s in x | s |
| NE | N east (x = s/2) and F1 top (z = -s/2) | quarter turn, clockwise from N to F1 | (s/2, -s/2) | s |
| NW | N west (x = -s/2) and F3 top | quarter turn, counter-clockwise from N to F3 | (-s/2, -s/2) | s |
| SE | S east and F1 bottom (z = s/2) | quarter turn, counter-clockwise from S to F1 | (s/2, s/2) | s |
| SW | S west and F3 bottom | quarter turn, clockwise from S to F3 | (-s/2, s/2) | s |
| NN | N top (z = -3s/2) and F2 top | half turn | (s, -s) | s |
| SS | S bottom (z = 3s/2) and F2 bottom | half turn | (s, s) | s |

- "Clockwise" is as seen on the map with north up.
- A player walking east out of N enters F1 heading south.
- All pivots are on chunk corners, because s/2 is a multiple of 128.
- The motions are exact integer block motions (the `iso` triple, see below), so a block and its
  partner across any seam are found without rounding.
- The table matches the survey's `CUBE_SEAMS` (`tools/research/projection-survey/survey.py`) and the
  14 edges of `WrapLayouts.cubeCross`, shifted to this window.

**The void.** The net box is 12s², half of which is not planet. The void is six s × s squares, above
and below F1, F2 and F3. Section 1.4 says what is generated there.

**The corners.** The cube's 8 corners (cone points, 270° round each in the world) sit at latitude
±35.26° and longitude λ0 + 45° + k·90°.

- **The 4 corners of F0** at (±s/2, ±s/2) are inside the world. They are concave corners of the net,
  each with one image, and a player can stand on them.
- **The other 4** (F1/F2/N, F2/F3/N, F1/F2/S, F2/F3/S) lie on the border. Each has two images, at N's
  or S's outer corners and at the F1/F2 or F2/F3 top or bottom corners.

**The work.**

- **W2.** Add a `Cube` geometry type in `deeptime-format` (new module `cube.rs`) with:
  - the face table above;
  - `fold(x, z)`, returning `Face(f, local)` or `Outside(edge, partner block, quarter turns)`;
  - the edge list;
  - the corner list with images;
  - the face adjacency used by rasters and solvers (section 3).

  Java gets the twin `worlddata/CubeLayout.java`, and JS gets `projection.js` `CubeWorld`.
  Check: unit tests in the three languages, run from one shared fixture. They cover:
  - every seam's motion maps the partner edge onto the edge, and applying it twice is the identity
    for half turns (four times for quarter turns);
  - fold(partner) is the identity on 10,000 random outside blocks within s of the planet;
  - the corner images agree;
  - the layout equals `WrapLayouts.cubeCross` after the window shift. WrapTopologyTest already holds
    the Java side of `cubeCross`.

### 1.3 Reach and the border until the wrap engine

**The border.** It runs on the 12 net edges that face the void or the outside:

- N's east, west and top edges;
- S's east, west and bottom edges;
- the top and bottom edges of F1, F2 and F3.

That is 12 segments of s blocks (46,080 blocks at 16k). Each segment keeps today's rules: inset 1
block (SCL-088) and a warning band of 24 blocks (POL-023), with today's bar spacing. F2|F3 is not
bordered, because Toroidal World slides it.

**Reach.** The net is connected through F0, so every point of the planet is reachable on foot,
both poles included. That is better than Petroff-Guyou, where the poles sat on the bordered edges.

The cost is the detour between two partners across a bordered seam:

- **Quarter-turn seams.** Partners at distance t from F0's corner meet only by walking round that
  corner, a path of 2t. The longest is 2s (7,680 blocks at 16k) at the far ends of the seam.
- **Half-turn seams.** The detour runs through N, F0 and F1 (or F3), at most about 2.6s.

Petroff-Guyou's worst detour today is b (6,656).

**What stops at the border.** It is the same list as today: players, mobs that path, items,
projectiles, Sable ships (through `SablePoleCap.tickEdge`, generalised to segments), and Create
trains (borders in V1, Ben's decision).

**Two holes the plan closes:**

- **The corners of the void.** A position just inside a void square, near F0's corner, lies beyond
  two segments at once. The push-back must take the nearest point of the planet, not each segment on
  its own, or a player can slip diagonally past the corner.
- **Arrivals outside the planet.** A Nether portal (or `/tp`, an ender pearl or chorus fruit) can
  land a player outside the planet. The Nether loop is 1:8, but nothing maps Nether z or the void's x
  back onto the planet.

  Today's Petroff-Guyou world has the same hole beyond its north and south edges. A grep for portal
  handling finds nothing: `EdgeBorder` only pushes back.

  The fix: an arrival outside the planet is folded onto its partner block with its heading turned (a
  portal arrival through vanilla's portal search position, other teleports through the border tick).

**The work.**

- **W3.** Generalise `EdgeBorder` from a symmetric |z| stop to the manifest's border segments, with
  nearest-planet-point push-back. `stopZ` and `petroffGuyou()` go.
  Check: GameTests at all 12 segments and at both sides of each of the four F0 corners. A grid of
  start positions inside the void squares near each corner ends on the planet. Sable ships are pushed
  back at a segment and cross the slide.
- **W4.** Fold arrivals outside the planet onto it (portal, teleport).
  Check: a GameTest builds a Nether portal at a Nether position whose overworld image is in the void
  above F1 and arrives on F0 or N. A `/tp` into the void lands on the partner block.

### 1.4 What is generated outside the planet

Players see across the border. Distant Horizons draws far beyond it. The wrap engine's band will
later hold turned copies of the neighbouring face. So everything outside the planet is drawn as the
fold through the nearest planet edge: the copy the wrap engine would show, with block states turned
(vanilla `BlockState.rotate`).

The region outside the planet splits as follows:

- **A void square that touches two faces** (above or below F1 or F3) splits on the diagonal through
  F0's corner, and each half takes the fold of its own edge.
- **The void above F2** is exactly N turned half way round (likewise S below F2).
- **The strips beyond N's top and S's bottom** are F2 turned.
- **Anywhere a single fold does not land on the planet** is deep ocean at sea level. That means
  further than s from every reachable point, past the box's corners.

Petroff-Guyou does the same today beyond its north and south edges (`PetroffGuyou.fold`, with up to
64 reflections). The cube uses one fold plus the ocean fallback.

The work is item G5.

### 1.5 Spawn

The candidates stay those the simulation's `spawn_candidates` and the bake's `project_spawn` give.
They are projected through the map slot. A candidate is passed over if it is:

- within 64 blocks of a border segment (POL-024 generalised), or
- within 256 blocks of any cube corner image. This is a new game choice (coordinator's call): at a
  corner three faces meet at 270°, and on a conformal map the scale runs away there.

All six faces are allowed.

**The work.**

- **W5.** Rework `SpawnHandler` and `project_spawn` to the rules above. The new margin gets a POL row.
  Check: a unit test on synthetic candidates. A GameTest that the world spawn on the 4k cube planet
  keeps both margins.

### 1.6 The map slot and what each candidate changes

**The slot.** It is one interface, implemented once per language and fed by a table:

- `forward(face, u, v) -> unit vector` turns world into planet, with (u, v) in [-1, 1]² on the
  face.
- `inverse(unit) -> (face, u, v)` turns planet into world.
- `jacobian(face, u, v)` gives planet metres per block along the face's two axes as a 2 × 2 matrix.
  For a conformal map it is a scalar times a rotation.

The cube's symmetry reduces every face to one eighth of a face: the triangle 0 ≤ v ≤ u ≤ 1. That
takes exact sign flips and a swap.

- **Forward.** The table holds, on a regular grid over the triangle, the planet point in gnomonic
  coordinates (a = y/x, b = z/x in the face frame). It is evaluated by a Catmull-Rom bicubic.
- **Inverse.** A second table over the gnomonic triangle gives a starting (u, v). A fixed two Newton
  steps on the forward bicubic then finish it.
- **Arithmetic.** Both use only +, ×, IEEE division and square root, in a fixed order with no fused
  multiply-add. Trigonometry, where latitude and longitude are wanted, goes through `portable_math`
  and `PortableMath` and `DTMath` as today. Rust, Java and JS therefore agree bit for bit (Java has
  been strict IEEE by default since Java 17).
- **Edges.** Points on a face edge reduce to the same triangle input from both faces, so the map is
  continuous across every seam. Edge nodes are written exactly (a = 1).

**The candidates.** The study writes them as one family by Moser's flow stopped at time τ: τ = 0 is
the conformal cube and τ = 1 the equal-area Moser cube. Blocks per planet area are τ + (1 - τ)/ρ,
where ρ is the conformal map's planet area per world area (`cube_table.py` on the study's branch).
The quarter-spherical cube (QSC) is ruled out: it has 41° creases along the face diagonals (SCL-082
stays a fallback only).

| | Conformal (τ = 0) | In between (0 < τ < 1) | Equal-area Moser (τ = 1) |
|---|---|---|---|
| Shapes | True everywhere except the 8 corners | Stretched a little, less than Moser | Up to 1.42 near corners, median 1.26 |
| Area per block | 0.79 to unbounded at the corners; 87% within ×1.25, 1% beyond ×2.2 of the mean | Bounded except at the corner itself | Within 0.6% (rms 0.01% on the 513² table) |
| Metres per block | Varies per point (one scale) | Varies per point and direction | Uniform area, varies with direction |
| Jacobian in the slot | Scalar times rotation | Full 2 × 2 | Full 2 × 2 |
| Bake footprint | Circle, radius × local scale | Ellipse | Ellipse of fixed area |
| Solver metric (section 3) | One scale factor per cell; corner cells about 0.16 of the mean area (Putman and Lin), so explicit steps halve | Full metric tensor | Full metric tensor, equal cell areas |
| Corner search (2.3) | Critical: land near a corner is drawn huge | Matters | Least critical |
| Fix 6 (D1) | Dropped | Dropped if shape stretch stays near 1 on land | Kept: volcanoes and craters drawn round in blocks through the inverse Jacobian |
| Every metres-to-blocks conversion | Must use the local scale: rivers, craters, volcanoes, structures, bodies (SCL-091 already does), deposits | As conformal, with direction | Nominal scale is right on average |
| Slopes at the fixed vertical scale | Vary with the local scale (the cost Ben accepted) | Vary less | Vary only with direction |

Under the conformal map the scale drops to 1/2.5 within 5° of a corner, to 1/1.8 within 10° and to
1/1.3 within 20°.

**The work.**

- **M1. The table generator.** Make `tools/cube-map/` a production copy of the study's
  `cube_table.py`, which builds on the survey's `moser.py` and `conformal.py`. It is Python with
  numpy, scipy and mpmath through `nix shell`, and it runs on the VM.

  It writes `crates/deeptime-format/data/cube-map-<name>.bin`: little-endian f64 with a header giving
  the name, n, τ and the generator's commit. It also writes the sha256.

  The table is data, committed once. Nobody has to regenerate it bit for bit, because its hash pins
  the world.

  It enforces the eightfold symmetry by building on the triangle and mirroring its ghost nodes, not
  by trusting a full-face solve.

  Check, all three run on the VM:
  - n = 257, 513 and 1,025 agree to the stated bound;
  - the bicubic's Jacobian determinant is positive everywhere, so there are no fold-overs, including
    within one block of a corner at 32k;
  - the area and shape statistics reproduce the study's numbers for the chosen τ.
- **M2. The slot in Rust.** `deeptime-format::cube_map` with `CubeMap::load(bytes)`, `forward`,
  `inverse` and `jacobian`, checked against the manifest's hash.
  Check:
  - the round trip inverse(forward(p)) returns within 1e-6 block on 10⁶ random points and on every
    face edge;
  - `jacobian` matches finite differences to 1e-6;
  - the two faces' forward values at an edge agree to 1e-15;
  - speed: under 30 ns per forward on the VM (the survey's bilinear lookup measured 13.4 ns).
- **M3. The slot in Java.** `worlddata/CubeMap.java`, which loads the same table from the jar
  resource, plus `WorldProjection.Kind.CUBE`.
  Check: `ProjectionParityTest` on the extended fixture.
- **M4. The slot in JS.** In `crates/deeptime-viewer/assets/projection.js`; the table goes into each
  bundle as a binary file.
  Check: `tools/viewer/projection-check.js` on the extended fixture.
- **M5. The fixture.** Extend `crates/deeptime-format/tests/data/projection_parity.json`, written by
  `DEEPTIME_WRITE_PARITY=1`, with cube forward, inverse, Jacobian and fold samples. They include
  points on every edge, within one block of every corner, and at both poles.
  Check: Rust, Java and JS read the same hex f64 values. Today's fixture has 5,280 values.

## 2. The bake

**The resample stays.** The simulation's cube is in the simulation's frame. The world's cube is the
same cube turned by λ0 about the polar axis, and λ0 is chosen after the run from the final land. So
the bake still resamples, cube to cube:

1. world cell centre;
2. world map forward;
3. turn by -λ0;
4. simulation map inverse;
5. a bilinear sample with ghost rings, or the footprint average below.

`docs/research/notes/sphere-native-grids.md` says a cube simulation needs no resample at the bake.
That overlooks the corner search, and the note gets corrected (D9).

Until phase 4 lands, step 4 is today's Lambert inverse. The bake code takes the simulation grid as
a trait, so both sources work.

**The work.**

- **B1. The world raster on cube faces.** Rewrite `bake/project.rs` (3,077 lines) around the map
  slot. `Mapping`, `resample_grid`, `glue_edge_rows`, `raise_world_tops` and `project_folder` become
  face loops. A world grid of r blocks per cell has n = s/r cells per face.
  Check: on `planet_world_4k` (seed 11) and earthlike_quick_16k, cells either side of every seam
  differ no more than cells inside a face. That uses the 99th percentile of height steps across
  seams against within faces, per grid.
- **B2. Footprint averaging.**
  - A world cell's planet footprint is the image of its square under the composed Jacobian:
    J_sim⁻¹ · R(-λ0) · J_world.
  - Where the footprint spans more than one simulation cell in some direction, the simulation is
    averaged over it. Otherwise it is sampled bilinearly.
  - This replaces the polar widening of SCL-090 (`POLAR_WIDEN_*`, `MAX_STRETCH` 8), which is deleted.

  Check: on a synthetic checkerboard planet at the simulation's resolution, the world shows no
  aliasing beats (the spectral peak beyond the world cell's Nyquist is below 1% of the input's).
- **B3. The corner search** (superseded by the study's §5.4, a free rotation: see the revision block
  at the top). It replaces `choose_lambda_c` (SCL-087).
  - The axis stays, so the band stays on the equator and the poles stay at the face centres. That
    is the coordinator's call: it keeps latitude and the net aligned.
  - It searches λ0 in 0.25° steps over 90°. The corner set repeats every 90°.
  - It maximises the smallest distance from the 8 corners to land, with the 4 reachable corners
    weighted the same as the 4 on the border.
  - The four 90° turns of one λ0 give the same corners but put a different band face at F0. The bake
    picks the turn that puts the most land on F0, the only band face joined to both poles
    (coordinator's call).
  - It records the 8 corners with their distances to land in the manifest and in `bake.stats`.

  Petroff-Guyou had 4 cone points: 39% of its candidates put all of them at sea, and the closest was
  7.0° from land. The cube has 8 at lower latitude, so all-at-sea will be rarer.

  Check: report on earthlike_quick_16k and a 12-seed set:
  - the share of candidates with all 8 at sea;
  - the smallest distance;
  - for a conformal map, the land area within 5° of a corner.
- **B4. Categories and point features.**
  - Categories are taken from the nearest simulation cell through the composed map.
  - Structures (`project_structure`), deposit bodies (`project_body`), sites (`project_site`),
    volcano vents, craters and spawn go through the inverse map.
  - Shapes go through the composed Jacobian, generalising SCL-091 from Petroff's map.
  - A body that touches several faces emits one image per face touched, cut at the face edge.

  Check: a body centred on each seam and on each corner shows the same planet area in total (within
  1%) as one in the middle of a face. A structure whose footprint crosses a seam is placed once, on
  the face holding its centre, and refused if it would cross a turning seam (Create track near turn
  points is already forbidden).
- **B5. Rivers across seams.** `project_rivers` cuts each river polyline on the sphere at the face
  regions (great circles), so the crossings are exact. Where the faces are joined directly in the
  net, the river continues in world blocks with no break. Where the seam is a turning seam, it is
  split with matched endpoint ids and a seam id, so the wrap engine can join the two halves later.
  River width in blocks uses the local scale.
  Check:
  - every river cut at a turning seam has two halves whose endpoints are partners under the fold;
  - no river on F3|F0, F0|F1, F1|F2, N|F0 or S|F0 is cut;
  - total river length on the planet changes by less than 0.5% against today's bake of the same run.
- **B6. Water, ice and closed basins on the world's ground.**
  - The projection pass's lake fill, `closed_basin` redraw (BAK-055, HYD-104, HYD-105, HYD-108),
    `bake/water.rs:83`, `bake/ice.rs:120`, `bake/strata.rs:731` and `deposits/cluster.rs:45-50` all
    stop calling `wrap_i` and `clamp_j`. They use the face adjacency, so a lake can straddle a seam.
  - The first and last rows stop being outlets (POL-021; `hydro/network.rs` "the poles, past the
    hard cap").

  Check: a synthetic basin centred on a turning seam fills to one level on both faces. No lake level
  differs across a seam.
- **B7. Biome layers and the land call on the world's ground.** These are `settle_biome_layers` and
  BIO-170 to 174, BAK-057 to 059 and HYD-106 to 109. They work face by face with ghost rings. The
  volcanic zones (BIO-105, BIO-106, BIO-182) become the shape the map makes of the circle: a circle
  on the conformal map, an ellipse otherwise.
  Check: the biome census tool on earthlike_quick_16k gives the same biome shares as today's
  Petroff-Guyou bake of the same run, within the census's tolerance.
- **B8. Deposits in the world's frame.** DEP-220 to 222 and the anchor top-up move from the
  Petroff-Guyou index frame to face cells:
  - `world_frame`, the guarantee and `inland_cells_scaled`;
  - the playability regions;
  - the guarantee's "land at least 128 world blocks from the sea", measured across seams.

  Check: the deposits census on `deposits_world_4k` (made a cube) keeps every guaranteed essential.
- **B9. The format: 0.7, layout version 2.** It follows question Q1's recommended answer and has no
  migrations (decisions log 2026-10-01). Its parts:
  - **The manifest's `projection`.** `kind "cube"`, `layout_version 2`, `map {name, sha256}`,
    `face_blocks` s and `lambda0_deg`. `faces[]` gives each face's id, net origin, quarter turns and
    the planet point at its centre. `edges[]` gives net edges with kind `slide`, `quarter_turn` or
    `half_turn` and the `iso [k, tx, tz]` the format already uses for cell motions. Then the 8
    `cone_points[]` with their images, whether each is reachable, and its distance to land; the two
    `poles[]`; a `border` with its inset and 12 segments; and a `domain` with the net window and
    `period_x`.
  - **Scale.** `scale.circumference_blocks` is 4s (the loop period, as the Java loop reads it).
    `m_per_block_h` is the nominal √(4πR²/6s²).
  - **Rasters.** They are stored face by face (F0, F1, F2, F3, N, S), each face n × n in its own net
    orientation, so a reader never turns a cell. There are no void cells.
  - **`GridDesc`.** `wraps_x` and `ZEdges {Clamp, HalfTurn}` give way to a face adjacency and one
    `cell(face, i, j)`. An index off the face resolves through the adjacency. At a corner, the
    diagonal ghost copies the neighbour along the adjacent strip (the FV3 rule). `GridDesc::cell` is
    today the only place the half-turn fold lives, so this is one function to replace.
  - **Docs and version.** `FORMAT.md` gets "The world's layout" rewritten. `FORMAT_VERSION` in
    Cargo.toml becomes 0.7.

  Check: the format's round-trip tests; `inspect` reads a 0.7 world; Java refuses 0.6 with today's
  message.
- **B10. The CLI and the dev tools in Rust.**
  - `deeptime project` forces `petroff_guyou` today and should force `cube`.
  - `world_map` draws the net, the border, the 8 corners and both poles.
  - `structure_view`, `structure_sections`, `satellite_view` and `parity` read the world's frame.
    They are Lambert-only today even on Petroff-Guyou worlds, a latent bug.
  - `watch/tiles.rs` draws a 4:3 net pyramid with empty void tiles.
  - `inspect` (`render.rs` seam, `area.rs`) and `deeptime-tiles` (its slope metric per face) follow.
  - `synth` gains cube worlds, so the Java side can test before the bake is done.
  - The viewer's `section.rs:198-200` builds a Lambert projection from the manifest's period, which
    is already wrong on Petroff-Guyou. It reads the manifest's projection instead.

  Check:
  - `deeptime synth --kind cube --size 4096` writes a valid 0.7 world;
  - `world-map` on earthlike_quick_16k shows the net;
  - `tools/review/projection-identity.sh` is rebaselined once on purpose and then holds.

## 3. The simulation's grids

What exists today:

- The simulation is tier B of the Rust sweep. It runs on the x-wrapped Lambert raster, with latitude
  from the row.
- Its GPU kernels (tier C) carry their own wraps in inline WGSL.
- No cubed-sphere code exists. `climate/upwind.rs`'s `Mesh` is the only grid-agnostic piece.
- `tools/research/sphere-grid-bench/` measured cube patches with ghost rings at the Lambert raster's
  cost:
  - routing 1.0 times;
  - diffusion 1.0 to 1.3 times;
  - a true-metric conjugate gradient needed 138 iterations against 785.

**One map, two cubes** (superseded: see the revision block at the top; the simulation takes Moser on the
polar axis, the world the conformal map turned freely). Following the 2026-10-22 log, the simulation's faces use the same map as
the world, from the same table.

The solvers get their metric from the cells' corner points on the sphere (finite volume, the Putman
and Lin form): areas, edge lengths and the 8 neighbour distances. Nothing in a solver depends on
which map made the grid. If the study picks a conformal map and the small corner cells (0.16 of the
mean) make an explicit step too short, the simulation could take a different τ from the same family
with no solver change. That is a coordinator's call at the time, recorded in the ledger.

**Cell counts.** At quick 16k today, L1 is 1,024 × 326 at r 16 (333,824 cells). On the cube at
r 16, n = 240 and 6n² = 345,600 cells, 3.5% more. Every r up to 256 divides s at all three sizes.

**The work.**

- **S1. `core::sphere::cube`.** It holds:
  - faces n × n with a ghost ring of width g (1 for most solvers, 2 where a stencil needs it);
  - storage face by face, the format's order, so the bake and the solvers share layouts;
  - per cell: unit vector, latitude, north bearing, area and the 8 neighbour distances, from the map
    slot;
  - a ghost fill in a fixed order;
  - a 64-bit Hilbert key per face for tiling.

  The latitude-from-row helpers (`lat_of_row`, `row_weights`, `true_cell_m`, `cell_area_km2`,
  `area_share`, `band_weight`, `GridDesc::projection()`) become per-cell lookups.
  Check:
  - the cell areas sum to 4πR² within 1e-9;
  - the ghost fill equals a direct fold on every ghost;
  - each face's neighbour table matches the format's `cell()`.
- **S2. The surface solvers on a neighbour function.** The solvers today take `Dims {w, h, dx,
  wrap_x}` in block metric. They switch to the cube grid and its metric:
  - routing and the parallel pflood;
  - young landscape, marine, spl and spread;
  - sea level;
  - diffusion (both ADI and multigrid);
  - glacial and thermal;
  - advect and amplify (window and tiles);
  - volcanoes, impacts, erosion bands and carbonate.

  `surface/grid.rs`'s `neighbour` (E, NE, N, NW, W, SW, S, SE) keeps its order with ghost-aware
  indices. ADI runs per face with ghost exchange between sweeps (its tridiagonal lines end at face
  edges); multigrid restricts face by face. D8 routing at a corner has 7 distinct neighbours, and the
  diagonal ghost must never make a self-loop.
  Check, per solver:
  - **Rotation.** A cone, a ridge and a basin placed on a face centre, on a seam and on a corner give
    the same answer turned. For example, the routed drainage area of a cone has its 99th percentile
    within 2% of the face-centre case.
  - **Conservation.** Sediment and water mass are conserved to 1e-9 relative over 1,000 steps on a
    closed planet.
- **S3. Climate.** Port the lat-lon row code to the cube:
  - `climate/mod.rs` `Geometry`;
  - `coarse.rs` (its coarse grid becomes a coarse cube);
  - `ebm2d`, `ocean2d`, `moisture`, `sheet` and `percell`;
  - `belts` and `lapse` per cell.

  The per-cell climate P1-26 (branch `worktree-agent-a1d2d85d0bfeddcc8`) is written on Lambert L1.
  Recommended order: P1-26 lands on Lambert first, then the port (coordinator's call).

  The orographic rain is an FFT in x today. It becomes a convolution per face on windows padded from
  the neighbours, with the wind turned into each face's frame.
  Check:
  - Earth's relief gives 13.8 °C mean against Ben's 14.1 ± 0.4 as today, within 0.2 °C of the
    Lambert result;
  - a zonally symmetric planet gives zonal climate (no face pattern beyond 0.1 °C);
  - rain on a ridge across a seam matches the same ridge in a face to 2%.
- **S4. Flexure.** `flexure.rs` (3,016 lines) is a global Fourier-in-x plus cosine solve, the least
  portable piece. It becomes a multigrid biharmonic solve on the faces with the true metric, reusing
  the diffusion multigrid.
  Check:
  - a point load and a line load match the analytic plate (Hetényi) within 2%;
  - on the band, the result matches today's solver on the same load within 5%.
- **S5. Plates L1, the ice sheet and the rest of tier B.**
  - Plates L1 (`plates/l1`) and its GPU kernel move to the cube. L0 stays on its Fibonacci sphere.
  - The ice sheet (`climate/sheet`) moves to the cube.
  - Hydro (`neighbour8`, `network.rs`) moves to the cube.
  - Deposits forecasts move to the cube.

  Check: the plate tests and the playability tests pass after re-baselining (S8).
- **S6. GPU kernels.** These are marine, multigrid, amplify, climate and plates L1. Each kernel runs
  on padded faces, plus one gather kernel per step that fills ghosts from a precomputed source table.
  Check: GPU against CPU on the VM's lavapipe gives the same results as today's tolerance per kernel.
  Real-GPU speed is measured on the laptop in A/B pairs, one bench at a time (the GPU-bench rule).
- **S7. Determinism.** The orders are fixed:
  - faces;
  - ghost fill;
  - tiles;
  - partial sums: per face first, then combined F0, F1, F2, F3, N, S. No rayon float sums (a
    recorded trap).

  Check: two runs on 8 and 1 threads give identical bytes for `earthlike_quick_16k`.
- **S8. Re-baselining.** The seed-specific tests and fixtures are reset once, on purpose: plates,
  playability, the bake tests (`bake/tests.rs` 183-232), `planet_world_4k` and the deposits worlds. The
  memory estimate (`memory_estimate`) and the admission seed table take 6n² cells, not w × h.
  Check:
  - `cargo test --workspace` is green;
  - the memory estimate is within its bound on full 16k (the kept-blocks trap is recorded);
  - the 12-run ensemble's land, crust and sea statistics are within master's seed spread (Mann-Whitney
    per run, the recorded rule).
- **S9. The speed gate.** The gate is full 16k at 8 pinned threads on the VM, 900 s. Master misses it
  today: 1,290 s quiet after the overlap (decisions 2026-10-17 to 19, `docs/design/p1-17-benchmark-
  gate.md`).

  The cube must not make it worse. The check is an A/B pair, cube against Lambert, on the same
  master, run side by side. The 900 s gate stays tracked by P1-17, not by this plan.
  Check: the cube run's wall time is at most the Lambert run's, phase by phase in the profile.

## 4. The game side

The Java sweep found every place that assumes a 2:1 world, a Lambert row, a symmetric |z| or the
Petroff-Guyou kind. Each line below is a place to change, grouped into work items. Paths are under
`neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime/`.

**State, 2026-10-23 (phase 2, stage 1, branch `worktree-agent-a19e057e8770be0a9`; design note
`docs/design/cube-game.md`).** A cube world is playable end to end behind two opt-in level types,
`deeptime:planet_cube` and `deeptime:planet_cube_32k`; Petroff-Guyou is unchanged and nothing of
section 7.4 is deleted yet.

- Done: G1 (readers, the cube manifest through `WorldDataRef`, `WorldProjection.Cube`, the indexes on
  net coordinates, the river file's seam mouth, code 3, SCL-126), G2 (the generator on the cube: period 4s, `zHalf` 3C/8 for the censuses, the
  level types; the static Lambert helpers stay until section 7.4), G4 (block states turned in the
  fill), G5 (`worldgen/CubeFill`: partners' biomes and terrain copied and turned, deep ocean past the
  corners, SCL-210, SCL-211), G7's border part (W3), G8 and W5 (spawn), W3 and W4 (`wrap/CubeBorder`,
  `EdgeBorder.tickCube`, arrivals folded, POL-031), G10's payload and F3 line (face, latitude and
  longitude; bearing and scale wait for G9), G11's `ServerConfig` and `FirstStartGenerator` text,
  G13's level-type registration and the editor's `cube` projection option, and G15's layout, seam,
  corner, fill, biome, ocean, block-state and border GameTests (`CubeWorldGameTests`, on the 4k test
  planet baked as a cube) and the reachability flood fill (`CubeBorderTest`), with the dev planet
  server on that cube (`gametest-planet.sh --boot-only --cube-4k`, its probes on every face against
  `inspect-column`) and the smoke's cube presets at 16k and 32k. Stage 2: G6 (`FarTerrain` folds
  an area off the faces column by column through `CubeLayout.fold`, turns its block states, reads
  the fill's biomes and draws deep ocean past the corners; Distant Horizons' sphere is the
  equal-area radius; the guard checks 4s; `/deeptime probe dh` gives the slide's x, and
  `dh-client-check.sh` and the smoke's `--dh` checks cover both sides of the slide as of Toroidal
  World's seam), G3 (the detail noise on the cube's surface lattice in Rust and Java, the bake's
  relief noises with it; the seam blend SCL-125 is retired for SCL-215), G9 (climate API 5:
  `projectionKind` `"cube"`, the period 4s, latitude and longitude through the map, the new
  `northBearingDeg` and `metresPerBlock` on both sides, the static `latitudeDeg(z, C)` deleted;
  `ClimateApiGameTests.climateApiOnTheCube` and `NorthBearingAndScaleTest`), G10's bearing and
  scale on the F3 line (`north 7.8° · 311 m/block here`; `NetworkGameTests.planetDebugLineOnTheCube`
  beside the Lambert strings' `planetDebugLine`), the rest of G11 (`probe pole` names both poles in
  N and S with the planet distance in km, the walk under the border and whether each is reachable;
  `probe column` gives the fold, north and metres per block; `probe tw` the period 4s, the face side
  and whether Toroidal World wraps at it, which the smoke checks; `sites near` adds km and the walk;
  `CubeBorder.walkDistance`, `WorldProjection.greatCircleBlocks`, `CubeWorldGameTests.cubeCommandProbes`;
  the dedicated-server first start on the cube is G6's 16k smoke run), G12's opt-in part (a cube
  level type takes every built-in preset of its size and bakes it as a cube: `Presets.withProjection`,
  `FirstStartGenerator.presetsFor`, the `server.json` template, `install.sh --cube`; the 16k smoke's
  first start bakes `earthlike_quick_16k` on `deeptime:planet_cube`).
- Left: the rest of G13 and G14, G16, the Sable border, and section 7.4 with what waits for it: G12's
  preset edits (`planet` and `planet_32k` made cubes, `planet_lambert*` deleted, the Rust presets'
  projection keys, the deposits fixtures as cubes), G13's `editor_fields.json` keys, and the
  deletions with the rewrites of the Petroff-Guyou tests they bring.

**The work.**

- **G1. Readers.**
  - `FormatVersion` 0.7.
  - `Manifest`.
  - `WorldLayout`: `LAYOUT_VERSION` 2 and kind `cube`. `fromJson` already throws on an unknown kind.
  - `GridDesc`: face adjacency in place of `ZEdges`.
  - `WorldData` and `RasterSampler`.
  - The spatial indexes `SiteData`, `Structure`, `Rivers`, `Columns`, `Water`, `FarColumn` and
    `ClimateLayers`. They are x-wrap only today; they index by face and cell, and neighbourhood
    queries cross seams by the fold. `SiteData.empty` and `Structure.read`'s default use the Lambert
    frame today, latent bugs that go.
  - `WorldProjection`: `Kind.CUBE`, with `of` building `CubeWorld`.

  Check: JUnit on a synthetic 0.7 world (B10) covers every index's query at a seam and a corner.
- **G2. The generator.**
  - `GeologicChunkGenerator.borderInset`.
  - `WorldDataRef`: projection `cube`, period 4s, a multiple of 512.
  - `WorldDataTerrain`: kind checks; `lakeMaxAbsZ` goes.
  - `TerrainSource.zHalf` goes.
  - `AnalyticTestTerrain`.
  - `PlanetGenerators`: two level types, not four.
  - The static Lambert helpers in `Projection.java` go. They are used by `DeepTimeClimate:139`,
    `BiomeCensus:525`, `GeologyCensus:96`, `GeothermalProbe:85`, `MagmaProbe:100`, `Geothermal:486`,
    `TerrainSource:44`, `WorldDataTerrain:135`, `BeyondPoleDensity:35`, `PoleCapCurve`,
    `GlobeProjection` and `MapWidget`, and each caller moves to the world's projection. `Geothermal`'s
    and `MagmaProbe`'s pole limits are Lambert-only today.

  Check: GameTests generate chunks at every face centre, both poles, every seam and every F0 corner
  without error. `RealPlanetGameTests` runs on the 4k cube test planet.
- **G3. Detail noise.** `DetailNoise`'s 32-block edge blend (SCL-089) is half-turn only today. It
  becomes a blend at every turning seam: within 32 blocks of a seam, a block's noise is mixed with
  the noise at its partner under the seam's motion, by weights that agree at the seam line. Block
  centres map to block centres under the quarter turns about chunk corners. At an F0 corner, the
  three faces' noises are blended by a partition of unity.
  Check: the noise at an outside block equals the noise at its fold partner exactly, for every
  seam. The height step across each seam is within the face's own distribution (as B1).
  Built otherwise (2026-10-23, the agent's design, accepted by the coordinator): no blend. On a cube
  world the noise is value noise on a lattice laid on the cube's surface, m = max(1, round(s/λ))
  cells along every cube edge, so the edges are lattice lines and neighbouring faces share their
  lattice points; a block's centre reaches the surface through the fold. The noise is one function
  on the surface, continuous through every seam and corner, and an outside block equals its fold
  partner bit for bit (SCL-215, FORMAT.md "The detail layer"). The bake's jag and cliff-band
  levels read the same noise, so the bake's seam blend (SCL-125, `seam_blend`) is gone.
  Petroff-Guyou and Lambert keep the plain noise byte for byte.
- **G4. Block states.** Fill copies and later the engine turn stairs, logs, rails and other directional
  blocks with vanilla `BlockState.rotate`.
  Check: a GameTest with a turned structure in the fill matches the original turned.
- **G5. The fill outside the planet** (section 1.4) replaces `BeyondPoleDensity` and the
  Petroff-Guyou reflections loop.
  Check: the seam GameTest (G15) compares a fill chunk against its partner block for block, with
  states turned.
- **G6. Far terrain and Distant Horizons.**
  - `FarTerrain` folds through the cube, not the x wrap, and answers the fill outside the planet.
  - `DhCurvature` and `ClientPlanetState.sphereCircumference` take the manifest's nominal radius. A
    cube has no single sphere in block space, so this is a visual choice: the radius of a sphere
    with the planet's area in blocks, 2,653 at 16k.
  - `TwDhGuard` and `TwDhGuardRule` check width 4s.

  Check: a far terrain GameTest at each seam and outside each bordered edge matches the generator.
  `tools/client/dh-client-check.sh` (made period-aware, T5) passes across the slide.
- **G7. Borders and the old pole code.**
  - `EdgeBorder` is reworked as W3.
  - `PoleCap`, `PoleCapCurve`, `PolarStorm`, `PoleStormPayload`, `SablePoleCap`'s cap and
    `BeyondPoleDensity` belong to the Lambert world and are deleted with it (section 7.4).
    `SablePoleCap.tickEdge` becomes the Sable border.
  - Ledger rows POL-001 to 015 are retired, not deleted.

  Check: as W3. `PoleCapGameTests` is deleted, and `SableSeamGameTests` is rewritten for segments.
- **G8. Spawn.** `SpawnHandler.EDGE_MARGIN` and its |z| test become W5.
- **G9. Climate API 5** (question Q2). `DeepTimeClimate.API_VERSION` goes to 5.

  mic-climate reads `latitudeDeg(level, x, z)`, `circumferenceBlocks(level)` (its client re-mesh
  trigger) and `projectionKind(level)`. Its `PlanetLatitude` says only "petroff_guyou" or "lambert".
  The repo is read-only for this work, so its owner adds "cube". Its test fallback is a straight line in
  z, which still works for its own GameTests.

  The recommended answer:
  - `projectionKind` answers `"cube"`.
  - `circumferenceBlocks` answers 4s.
  - `latitudeDeg` and `longitudeDeg` go through the map slot.
  - Two calls are new: `northBearingDeg(level, x, z)`, the direction of north in the block frame,
    which turns at the quarter-turn seams and inside the polar faces; and `metresPerBlock(level, x,
    z)`, the local scale, nominal on the equal-area map.
  - The static Lambert-only `latitudeDeg(z, C)` is deleted.

  Check: `ClimateApiGameTests` check latitude and longitude at the six face centres, the poles, the 8
  corners (±35.26°) and points across every seam; bearing on each face; scale against the slot's
  Jacobian.
- **G10. The network and F3.**
  - `WorldInfoPayload` carries kind `cube`, s, λ0 and the map's name and hash. The client has the
    table in its jar and shows "map mismatch" instead of numbers if the hash differs.
  - `PlanetDebugLine` (F3) shows face, latitude, longitude, north bearing and local scale.

  Check: a new F3 GameTest in `NetworkGameTests` covers both the cube and the Lambert strings that go.
  Petroff-Guyou has never had an F3 test.
- **G11. Commands and the server.**
  - `DeepTimeCommands` probes: `pole` (both poles are inside N and S), `tw` (period 4s) and `column`.
  - `SiteCommands.distance` (x-wrap only) gives planet distance in km through the map slot, plus the
    walking distance under the border.
  - `ServerConfig:159` (13312 × 6656 hard-coded) reads the layout.
  - `FirstStartGenerator`: `levelTypeFor`, `describe` and `presetsFor`.

  Check: the command GameTests; a dedicated-server first start on the cube in the smoke run.
- **G12. Presets and level types.**
  - World presets `planet` (16k cube) and `planet_32k`.
  - `planet_lambert` and `planet_lambert_32k` are deleted, along with their entry in the `normal.json`
    tag.
  - Rust presets lose `scale.projection` and `lambda_c_deg` and gain `lambda0_deg` (-1 means search).
    That covers earthlike, pipeline and plates at full and quick for 16k and 32k, and
    `tests/presets/planet_world_4k.json`. `deposits_world_4k` and `deposits_world_16k` become cubes.
  - The quick keyframe size `d_keyframe_size` [1024, 326] is a π:1 picture and stays (G13).

  Phase 2 builds the opt-in part only (coordinator's call, 2026-10-23): the level type chooses the
  map, so a cube level type bakes any preset of its notional size as a cube
  (`Presets.withProjection` on a copy, `--set scale.projection=cube` on the CLI line), and
  `presetsFor` lists the built-ins for it. Petroff-Guyou and Lambert keep their path. The renames,
  the deletions, the Rust preset keys and the deposits fixtures wait for section 7.4.

  Check: `Presets` JUnit; `FirstStartGenerator` on both level types.
- **G13. The Create World screen.**
  - `ClientSetup` registers the editor for the two cube level types. Today it registers it only for
    the Petroff-Guyou ones.
  - `Presets.worldCircumference` and `DEFAULT_PROJECTION`.
  - `editor_fields.json` loses `projection`, renames `lambda_c_deg` to `lambda0_deg` and keeps
    `edge_border`. The lang keys and `EditorModel.accept` follow.
  - The timelapse widgets (`MapWidget`, `GlobeWidget`, `GlobeProjection`, `GenerationPage`,
    `TimelapseModel`, `ReviewModel`) draw π:1 Lambert pictures of the simulation's keyframes. The
    keyframe picture stays a π:1 Lambert picture, resampled from the cube by the simulation when it
    writes a keyframe (coordinator's call: keep a lat-lon picture projection for maps), so these
    widgets keep working.
  - The globe adds the cube's edges and corners. `ReviewModel`'s final-world view draws the net with
    the border.

  Check: `EditorModelGameTests`; the Create World walk on the VM (`tools/client/vm-client.sh`) with
  screenshots of the globe and the final world.
- **G14. The dev map** (`devmap/`, never shipped).
  - `PlanetGeometry`:
    - `displayLon`, `simLon`, `seamDistance` and `edgeDistance` work by segments;
    - `poles` gives two points inside N and S;
    - `conePoints` gives 8 corners with their 12 images, not 6 hard-coded points.
  - `MapFrame` covers 4s × 3s with void transparent.
  - `Graticule` is cut at seams.
  - `ViewTransform` wraps 4s.
  - `BaseImage` changes.
  - `ShotDriver` places: spawn, the slide, a quarter-turn border, the north pole, a reachable corner
    and a border corner.

  Check: `PlanetGeometryTest` and `MapModelTest` on a cube manifest; `tools/client/devmap-shots.sh`.
- **G15. New GameTests.**
  - **Seams, bit for bit.** For each of the 7 seams, a chunk outside the planet next to the seam
    equals its partner chunk turned, block for block with states turned. The height step across the
    seam is within the face's distribution.
  - **Corners.** At each F0 corner the three faces meet without holes, a player can stand on the
    corner, and the border closes the void quadrant.
  - **Reachability.**
    - A JUnit flood fill of the net at chunk resolution, under the border, reaches all six faces
      and both poles from spawn.
    - A GameTest teleports to each pole and is not pushed.
  - **Far terrain at seams.** As G6.
  - **Climate API.** As G9.
  - **F3.** As G10.
  - **Portals.** As W4.
  - **Spawn.** As W5.

  The tests whose fixtures change:
  - `SeamlessWorldGameTests` is rewritten for the cube;
  - `RealPlanetGameTests`;
  - `FarTerrainGameTests`, which uses Lambert `poleZ` on the Petroff-Guyou planet today;
  - `EditorModelGameTests`, `ImpactGameTests` and `SeamGameTests`;
  - `WrapEngineGameTests` and `CreateSeamGameTests`, which keep their own layouts.

  The JUnit tests that change: `ProjectionParityTest` (`Kind.parse("cube")` is null today),
  `ColumnParityTest` and `FarColumnTest`.

  The test dimensions:
  - `test_planet_real` becomes the 4k cube, period 4,096.
  - The Gradle fixtures `testWorldPlanet4k` and `testWorldDeposits4kRun` are regenerated as cubes.
  - The Lambert synthetic worlds stay for tests that do not care about the shape, as cubes made by
    `synth`.

  Check: `./gametest.sh` and `./gametest.sh -PwithMic` pass in the coordinator's merge batch.
- **G16. Lava flows across seams** (fix 8's `worldgen/LavaFlows.java`). A flow's step to the next
  column, its reach box and its grid keys go through the cube's fold (W2) in place of the x-only
  `wrap`/`delta`, so a flow that reaches a seam carries on onto the partner face, turned. The metric
  comes from the map slot's Jacobian (M3) through `WorldProjection`.

  Check: `LavaFlowsTest` gains a flow started a few blocks from each kind of seam (slide, quarter
  turn, half turn) whose painted columns match on both sides; far terrain paints what the chunks
  paint across a seam.

## 5. The wrap engine

**What exists** (`docs/design/wrap-engine.md` §10, built 2026-10-04 on branch
`worktree-agent-a4445c63094be0023`):

- Folded block access at the `Level` API.
- A replicated band of turned chunks.
- Entities carried across with heading and velocity turned.
- `CellIsometry` with quarter turns.
- `WrapLayouts.cubeCross` (14 edges) and `cubeCrossCorners`.
- 15 Create seam adapters and their logic in `compat/create/wrap/`.
- 15 GameTests on half and quarter seams.
- A benchmark: `getBlockState` off the band at 25.8 to 26.7 ns against 23.8 ns, and 49.1 ns in the
  band.

It is dev-only (`-Ddeeptime.wrapengine=true`, `./gametest.sh -PwrapEngine`). `WrapEngine.install` has
no caller outside `WrapEngineGameTests`. Ben's choices: "Folded access + band" and "TW slides, engine
turns" (2026-10-03), then "Engine only; pause IP" (2026-10-04).

**What it needs before it can join the cube's turning seams.** This is wrap-engine.md §10.4 and its
phases 2 to 5, made concrete for the cube:

- **E1. A topology from the manifest.** Build `WrapTopology` from layout version 2's edges and corners
  and install it on the overworld in production, with Toroidal World keeping the slide. The engine
  composes the slide where TW's window cuts F2.
- **E2. Block-entity data in the band.**
- **E3. Worldgen across seams (its phase 2).** The fill outside the planet (G5) is already the band's
  content, so the generator side of phase 2 is done by G5. What remains is keeping the band current
  as players build.
- **E4. Entities.** Leashes, ghosts, items and projectiles across turns. At corners, seam links are
  refused from both ends near a corner, as the engine already does. The 8 corners are 270° cone
  points and cannot be seamless locally.
- **E5. The client (its phase 3).** The re-anchor across a turn, so the view does not snap. The
  north bearing (G9) turns with it.
- **E6. Pack compatibility (its phase 4).**
  - TW on the same level.
  - Distant Horizons across turns (LODs of band chunks).
  - Sable through `PhysicsPipeline.teleport(body, position, orientation)`. TW's own Sable bridge
    (`SableRigidShift.requireTranslation`) throws on any turn.
  - Create trains (its phase 4b). Track is already refused near turn points.
- **E7. Hardening (its phase 5).**
- **E8. The detail noise and fill.** G3 and G5 already make the world continuous across turns, so
  joining a seam later changes no terrain.

**Immersive Portals is not the route.** The 2026-10-02 prototype's pack costs
(`docs/research/notes/immersive-portals-pillowcase-prototype.md`):

- Petrolpark, which carries Destroy, MIC Destroy Electric and MIC Petrochem, hard-crashes at mod
  construction. Its access transformer clashes with IP-CE's private `@Overwrite`.
- CBC Modern Warfare 0.0.6v crashes.
- Toroidal World kicks every join, so TW would have to go.
- Sable 2.0.5 crashed four times in about 15 minutes, and ships do not cross same-dimension portals.
- Distant Horizons' LODs did not build (unverified).

IPSable (Apache-2.0, a prerelease) was not tried and lacks DH support. Ben paused IP on 2026-10-04.
SCL-085 (IP's quarter-turn quaternion) is retired with it.

**Until the engine:** the border of W3.

Question Q3 asks Ben when to schedule the engine. The recommendation is after V1, because V1 is
complete and playable without it: every point is reachable.

## 6. Docs and science

The work, each landing with the phase whose code it describes (the rule "Science documentation is
part of done"):

- **D1.** Rewrite `docs/science/seamless-world.md` for the cube, keeping its sections: what you see,
  the model, equations and constants, Earth against the mod, limits, game choices and open questions.
  - **Earth against the mod.** It repeats today's measurements on the cube: land share on the map
    against the true planet (today 0.13174 against 0.13168), the corners' distances to land, and the
    area and shape statistics on land for the chosen τ.
  - **Limits.** It records the corners, the border detours and the conformal scale near corners if
    τ < 1.
- **D2. SCL rows.** "New" rows take SCL-100 onward, leaving a gap after SCL-091 for peers (the ledger
  trap). All rows are in `docs/science/ledger/SCL.md`.

  | Row | Change |
  |---|---|
  | SCL-078 | Value 3,840 and the 256 quantum |
  | SCL-079 | λ0 search, axis kept, F0 by land |
  | SCL-080, SCL-081 | Coded; the study's τ; from `not-yet-coded` to the status the sources allow |
  | SCL-082 | Stays the fallback |
  | SCL-083 to SCL-087, SCL-089 and SCL-090 | Retired with Petroff-Guyou |
  | SCL-088 | Generalised to segments |
  | SCL-091 | Generalised to the composed Jacobian |
  | SCL-085 and SCL-070 to SCL-077 | Retired (the IP prototype) |
  | SCL-049, SCL-050 and SCL-052 | Rewritten when the simulation moves (phase 4) |
  | SCL-004 | Gains the period 4s |
  | New | The table (size, interpolation, Newton steps), the fill outside the planet, the corner search's step and weights, the nominal m per block |
- **D3. POL rows** (`docs/science/ledger/POL.md`).

  | Row | Change |
  |---|---|
  | POL-022 | Poles ordinary ground inside N and S |
  | POL-023 | Segments |
  | POL-024 | Every segment |
  | POL-001 to POL-015 and POL-017 | Retired with the Lambert world |
  | POL-020 and POL-021 | Retired when the Lambert simulation goes |
  | POL-016 | Period 4s |
  | New, from POL-030 | The 256-block corner margin for spawn, the arrival fold |
- **D4. Other ledger rows that name the world's frame.** Re-check BAK-042, BAK-055, BAK-057 to 059,
  HYD-104 to 109, DEP-220 to 222, BIO-105, BIO-106, BIO-159, BIO-170 to 174, BIO-182, VOL-080,
  STR-055 and ORE-271.
- **D5. Science pages that mention Petroff-Guyou or Lambert.** Update:
  - `world-scale-and-bake.md` (m per block, face size);
  - `pole-cap.md` (retired with the Lambert world);
  - `quincuncial-projection.md` (superseded);
  - `young-landscape-and-amplifier.md`, `biomes.md`, `deposits.md`, `rivers-and-lakes.md`,
    `climate.md`, `per-cell-climate.md`, `provinces.md`, `sea-level.md`, `volcanoes-and-magma.md`,
    `strata.md`, `folds-faults-intrusions.md` and `README.md`.
- **D6. Bibliography rows.** Each gets the status its source reading allows, never `verified`
  without reading:
  - Lee 1976 (conformal projections from elliptic functions, the conformal cube);
  - Rančić, Purser and Mesinger 1996 (the conformal cubic grid);
  - Moser 1965 and Dacorogna and Moser 1990 (the flow);
  - Putman and Lin 2007 (finite volume on cubed spheres, the corner cells);
  - Ronchi, Iacono and Paolucci 1996 and Sadourny 1972 (cubed spheres);
  - Roşca and Plonka 2011 (QSC, already read for SCL-082);
  - Hetényi 1946 (plate flexure, for S4).
- **D7. `FORMAT.md`.** "The world's layout", rewritten (B9).
- **D8. Design docs.**
  - `docs/design/wrap-engine.md`: §10.4's "the layout is not chosen ... the bake still writes
    Lambert" is stale. The cube becomes the target layout.
  - `seamless-wrapping.md` and `equal-area-projection.md`: status lines.
  - `docs/deeptime-implementation-plan.md` §5.2 and the glossary's "Block space" still say
    equirectangular or Lambert. They get rewritten to point here; §7.10 to §7.12 get notes.
- **D9. `docs/research/notes/sphere-native-grids.md`.** Correct "no resample at the bake" (λ0, section
  2).
- **D10. A review page for Ben.** The cube world of one planet: the net with border, the corners and
  their distances, the seams, the land share, and the area and shape maps for the chosen τ.

  It is SW-1's page (`tools/review/seamless-world/page.py`, hard-coded 2:1 today) rebuilt for 4:3. It
  is published as a private Spacesheep page through `tools/review/publish.sh`.

## 7. Order of work

### 7.1 Phases

1. **Phase 0: geometry and the map slot.** W1, W2, M1 to M5, and B9's schema written into
   `FORMAT.md` first: the contract both lanes code against.
   - The geometry needs no map choice.
   - M2 to M5 can be built and tested on the survey's equal-area table, and swapped for the study's
     table when it picks τ. Only the data changes.

   Gate:
   - the parity fixture is identical in Rust, Java and JS;
   - W2's fold tests pass;
   - M1's convergence and positivity checks pass;
   - `cargo test`, clippy, fmt and deny pass; `./gradlew build test` passes.
2. **Phase 1: the bake on the cube from today's simulation.** B1 to B10.
   Gate: earthlike_quick_16k and `planet_world_4k` baked to cubes on the VM. B1 to B8's checks pass,
   and D10's review page is published.
3. **Phase 2: the game on the cube.** G1 to G16, W3 to W5, and the deletions of section 7.4.
   Gate:
   - cargo and Gradle pass;
   - the coordinator's GameTest batch (standalone and `-PwithMic`) passes;
   - `tools/smoke/run.py` passes at 16k and 32k, standalone and mic, including a first start;
   - `tools/client/dh-client-check.sh` passes across the slide;
   - the Create World walk's screenshots are taken;
   - `gametest-planet.sh` probes run on the 4k cube.
4. **Phase 3: tools, viewer, workbench and docs.** T1 to T8 and D1 to D9. It runs beside phase 2 and
   starts as soon as M4 and B10 exist.
   Gate: `projection-check.js` and `worldmap-check.js` pass; a workbench preview runs on a cube world;
   the ledger is up to date.
5. **Phase 4: the simulation on the cube.** S1 to S9. It is its own lane from phase 0 on, since it
   needs only M2 and the format's adjacency. When it lands, the bake's source switches from Lambert to
   the cube (B1's trait).
   Gate: S1 to S9's checks; the A/B time pair; the re-baselined ensemble.
6. **Phase 5: V1.**
   - The play server gets its fresh cube world, so the D5 Petroff-Guyou world is replaced (decided:
     "another fresh world comes with the cube"). The coordinator's call is that this waits for
     phases 2 and 4 both, so players get one fresh world, not two.
   - The Lambert simulation grid is deleted.

   Gate: `tools/remote/vm/play-stage.sh` on the cube level type; play checks; the world format
   checked before the jar swap (the recorded deploy trap).
7. **Phase 6: the wrap engine** (E1 to E8). Ben scheduled it now, in parallel with V1 (Q3, 2026-10-22).

### 7.2 Tools, viewer and workbench (phase 3's items)

- **T1. `projection.js`.**
  - `CubeWorld` (M4).
  - `DTProj.world` picks it for `kind === 'cube'`.
  - `tileFrameZ` and the π:1 picture helpers stay for keyframe pictures.
  - `Petroff`, `PetroffGuyou` and `LambertWorld` are deleted with section 7.4.
- **T2. `worldmap.js`.** `frameOf` draws a 4:3 net pyramid with void tiles empty, the border and the
  corners; `Wp`, `zLim` and the 3D click handler work by face.

  Its twin `frameOf` is in mic-viewer's `map.js`, checked by `tools/viewer/worldmap-check.js`.
  mic-viewer is a public repository since 2026-10-21: scan its commits for world names, hosts and
  server data before pushing, then bump Deep Time's pinned rev.
- **T3. `workbench.js`.**
  - `worldOf`, `blockXZ`, `boxBlocks` and `circ` go through the world.
  - The graticule's seam line becomes the face edges.
  - The globe draws the cube's edges and 8 corners.
  - The "N × b world" text goes.
  - The known issue "`worldOf` answers in the Lambert frame until `world.json` loads" is fixed by
    loading the world before the first tap.

  `viewer.js`, `sections.js` and `deposits.js` read the simulation's pictures. They stay Lambert
  pictures until phase 4, then read the keyframe picture (G13).
- **T4. `tools/workbench/preview.py`.**
  - `LEVEL_TYPES` has two entries.
  - `PETROFF_PERIODS` goes.
  - `fold` and `pieces` go through the cube.
  - `pole_z` goes.
  - `chunky_square` stays inside one face and never crosses a turning seam.
  - `latlon_to_block` and `box_on_world` take the map.

  `find_places.py`, `sections_shots.py`, `deposits_shots.py` and `test_preview.py` follow.
- **T5. Period bugs in the test tools.** `tools/smoke/run.py` and `tools/client/dh-client-check.sh`
  use the notional size (16,384) as the period. That is already wrong at 13,312 and would be wrong at
  15,360. Both read the period from the probe or the manifest.
  - `run.py`'s seam probes (`chunky_center("seam")`, `--chunky-seam`, `PROBE_COLUMNS`, the DH pregen
    squares) sit at TW's seam and beside a bordered edge.
  - `install.sh` sets the cube level types and loses `WORLD=lambert`.
- **T6. `tools/planet/run_planet.py`.**
  - Level types become the cube's.
  - `forceload` folds through the cube.
  - `sable_straddle` straddles the slide.
- **T7. Review tools.**
  - `tools/review/seamless-evidence.sh`: a cube world in place of the pg/lambert pair.
  - `seamless-world/page.py` (D10).
  - `phase1_vm.py:170` and `phase1_scale.py` (sizes).
  - `ice-spikes.sh` (level type from `manifest.projection.kind`).
  - The Lambert-L1 scripts `continent-bisect.py`, `margins.py`, `percell_metrics.py` and
    `water_census.py` read the cube grid after phase 4. The water census trap (hydro-grid frame
    coordinates) is recorded.

  `tools/review/world-shape/compute.py` stays as the Python reference for the cross net
  (`cube_cross_edges`, `engine_fold`).
- **T8. `tools/remote/vm/play-stage.sh`.** Level type and period.

### 7.3 Lanes and the branches in flight

**Lanes.** These can run at once (the coordinator's memory: keep three to four Opus agents running):

| Lane | Phases | Starts when |
|---|---|---|
| Format and bake | 0, 1 | Now |
| Game | 2 | `FORMAT.md` 0.7 and `synth --kind cube` exist |
| Tools and docs | 3 | M4 and B10 exist |
| Simulation | 4 | M2 and the adjacency exist |

Each lane takes its own ledger id range so concurrent agents do not collide.

**The branches in flight:**

- **D6, transition width on the model sphere** (`worktree-agent-a0f44b1747562ca67`). It changes
  `plates/transition.rs`, `belts.rs`, `params.rs`, the 12 presets and `editor_fields.json`. It is
  independent of the map and lands first. The plan's preset and editor edits (G12, G13) merge after
  it, with trivial conflicts.
- **D2, D3, D7: gullies, cliffs, craters** (`worktree-agent-a50beab5a56858b5e`). It changes
  `surface/amplify/volcano.rs`, `volcanoes/shape.rs` and `underground.rs`, on the simulation's
  raster. The bake resamples it like everything else, so phases 0 to 3 do not touch it. In phase 4
  its shape code takes the cube's metric (cone radius in metres per cell, not block metric).
- **Fix 8, lava-flow paint** (`worktree-agent-a221f5181dfc5a4b5`). This plan first recommended
  painting on the simulation's raster. By the time it was read, fix 8 was built in the game instead:
  `worldgen/LavaFlows.java` traces flows downhill on the drawn heightfield at chunk time, steps in
  true metres through `WorldProjection`'s metric, and wraps x in one `wrap`/`delta` pair. The
  coordinator's call (2026-10-22): it lands as built and ships on the D5 world, and G16 ports its walk
  to the cube.
- **The projection study** (`worktree-agent-a996493d601a564e5`). It picks τ. Its `cube_table.py`
  becomes M1, and its numbers fill SCL-080 and SCL-081, D1 and the table in section 1.6. Phase 0
  does not wait for it (see phase 0).
- **Fix 6 (D1)** follows section 1.6's row: dropped for conformal or nearly conformal, kept for
  Moser.
- **D5's fresh world on Petroff-Guyou.** It goes ahead as decided. Tag its build commit (for example
  `play-d5`) so a hotfix branch can start there after master moves to 0.7.
- **P1-26 per-cell climate** (`worktree-agent-a1d2d85d0bfeddcc8`). It lands on Lambert, and S3 ports
  it.

### 7.4 What to delete (recommendation, the coordinator's call)

1. **When phase 2 lands, delete both old world shapes.**
   - **The Lambert world.** `scale.projection lambert`; the level types `planet_lambert` and
     `planet_lambert_32k`; the pole cap, polar storm, `BeyondPoleDensity` and `PoleCapCurve`; the
     Lambert branches in `FarTerrain`, `Geothermal`, `MagmaProbe`, `TerrainSource` and
     `WorldDataTerrain`; the static helpers in `Projection.java`.
   - **Petroff-Guyou.** `PetroffGuyou` and `Petroff` in Rust, Java and JS; `NEAR_SIDE`;
     `choose_lambda_c`; SCL-090's widening; Petroff-Guyou's parts of the parity fixture; the SW-1
     tools.
   - **The quincuncial prototype.** `crates/deeptime-quincuncial` and `tools/proto/quincuncial`.

   **Why not keep Petroff-Guyou until the play server moves.** Master at 0.7 cannot read the D5
   world anyway: the format check refuses 0.6, and there is no compatibility before release. Keeping
   the Petroff-Guyou code on master would serve nobody. The play server stays on its own build, with
   hotfixes from the tagged commit.
2. **When phase 4 lands, delete the Lambert simulation grid.** That covers `Dims`' wrap and the
   latitude-from-row helpers. A lat-lon picture projection stays for keyframe pictures and maps.
3. **Keep the research tools** (`tools/research/projection-survey`, `sphere-grid-bench`,
   `mountains-projection`, `tools/review/world-shape`) as research.

## 8. Questions only Ben can answer

**Answered 2026-10-22** (decisions log): Q4 (a), Q1 (a), Q2 (a), and Q3 (b), the wrap engine now, in
parallel with V1. Phase 6 runs as its own lane from now; borders (W3) still ship with phase 2.

Asked in one batch when Ben says he is back (log 2026-10-22: no questions until then). The
recommended option comes first in each. Everything else in this plan is undoable and taken as the
coordinator's call.

1. **The world-data format 0.7 (a data format).**
   - (a) **Recommended.** Rasters stored face by face with no void cells, the map table shipped in
     the code (Rust, jar, viewer), and the manifest naming the map and its sha256. A reader with a
     different table refuses the world.
   - (b) Rasters over the whole 4s × 3s net box with void cells. This is simpler for some tools but
     doubles the disk and memory per raster.
   - (c) The table carried inside every world. Each world is self-contained, but larger, and three
     languages must read the table from the world.
2. **Climate API 5 (a public interface mic-climate reads).**
   - (a) **Recommended.** Keep `latitudeDeg`, `longitudeDeg`, `projectionKind` (new value `"cube"`)
     and `circumferenceBlocks` (now the loop period 4s). Add `northBearingDeg` and `metresPerBlock`.
     Delete the static Lambert-only `latitudeDeg(z, C)`.
   - (b) Only the new kind value, no new calls.
3. **When to build the wrap engine (big scope).**
   - (a) **Recommended.** After V1 ships on borders: every point is reachable without it.
   - (b) Now, in parallel with V1.
   - (c) After the public release.

4. **The map and the turn (asked because it moves the poles off the face centres).**
   - (a) **Recommended (the study).** The conformal cube turned freely: shapes true, land × 1.92 at
     worst, corners 5.3° or more out at sea. North turns across the world and the band faces do not
     hold the equator.
   - (b) The conformal cube with the pole within 30° of face N's centre: land × 2.05 at worst, north
     near -z on the polar face.
   - (c) The equal-area Moser cube on the polar axis: areas exact, poles at the face centres,
     mountains stretched 1.26 times one way (median, up to 1.42), fix 6 kept.

   In every case the simulation runs on its own Moser cube on the polar axis.

## 9. Sources

**Decisions.** `docs/deeptime-design-decisions.md` log entries:

- 2026-10-02 (equal area; IP prototype);
- 2026-10-03 and 2026-10-04 (the wrap engine; Petroff-Guyou; "Engine only; pause IP");
- 2026-10-05 (bordered edges; climate grid);
- 2026-10-16 (Ben's decision rule);
- 2026-10-17 to 19 (the 900 s gate);
- both entries of 2026-10-22.

**Plans and design.**

- `docs/deeptime-implementation-plan.md` §5.2, §7.10 to 7.16, §14 (Q6).
- `docs/design/wrap-engine.md` (§0, §1, §6, §9, §10, §10.4, §11).
- `docs/design/seamless-wrapping.md` (§11 the format 0.5 layout).
- `docs/design/equal-area-projection.md`.
- `docs/design/p1-17-benchmark-gate.md`.
- `docs/design/p1-26-per-cell-climate.md`.

**Research.**

- `docs/research/notes/seamless-projection-survey.md` (the cube, its net, Moser's flow, QSC, §8).
- `sphere-native-grids.md`.
- `immersive-portals-pillowcase-prototype.md`.
- `wrapped_planet_worlds.md`.
- `world-projection-options.md`.
- `rotated-seam-worlds-prior-art.md`.
- `grid-artifact-audit.md`.
- The study's `mountains-and-the-projection.md` (in progress).

**Science.**

- `docs/science/seamless-world.md`, `quincuncial-projection.md` and `pole-cap.md`.
- `docs/science/ledger/SCL.md` (SCL-004, 005, 049, 050, 052, 070 to 091) and `POL.md` (POL-001 to
  024).

**Code** (read for this plan).

- **Rust.**
  - `crates/deeptime-format/src/projection.rs` (`HEMISPHERE_QUANTUM`, `ProjectionKind`, `EdgeSpec`,
    `Fold`, `PetroffGuyou`).
  - `crates/deeptime-core/src/surface/grid.rs`.
  - `crates/deeptime-core/src/bake/project.rs`.
  - `crates/deeptime-core/src/hydro/network.rs`.
  - `crates/deeptime-viewer/src/section.rs:198-200`.
- **Java.**
  - `neoforge/.../wrapengine/WrapLayouts.java`, `WrapTopology.java` and `WrapEngine.java`.
  - `wrap/PlanetShape.java` and `wrap/EdgeBorder.java`.
  - `api/climate/DeepTimeClimate.java`.
  - mic-climate's `seasons/PlanetLatitude.java` (read only).
- **JS and tools.**
  - `crates/deeptime-viewer/assets/projection.js`.
  - `tools/research/projection-survey/survey.py` (`CUBE_SEAMS`), `moser.py` and `conformal.py`.
  - `tools/research/sphere-grid-bench/src/main.rs`.
  - `tools/review/world-shape/compute.py`.
  - The study's `tools/research/mountains-projection/cube_table.py`.

**Literature.** As cited by the survey. Their reading status is the survey's and the bibliography's,
not this plan's:

- Lee 1976;
- Rančić, Purser and Mesinger 1996;
- Moser 1965;
- Dacorogna and Moser 1990;
- Putman and Lin 2007;
- Ronchi, Iacono and Paolucci 1996;
- Sadourny 1972;
- Roşca and Plonka 2011;
- Chan and O'Neill 1975.
