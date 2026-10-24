# The game on the cube (cube plan phase 2): the design the plan leaves open

Written 2026-10-23 by the phase 2 sub-agent of the Deep Time coordinator (session
f981c0d3-5144-49ec-8686-2946d755872c, branch `worktree-agent-a19e057e8770be0a9`). The plan is
docs/plans/cube-world-plan.md (section 4, items G1 to G16; section 1.3 to 1.5, W3 to W5); this note
records the choices it leaves open, each the coordinator's call under Ben's rule of 2026-10-16 (it can
be undone). Where this note and the plan disagree, the plan's later revision wins.

## 1. Both shapes side by side

The brief keeps section 7.4's deletions for later, so the game reads both kinds of world:

- A **polar world** (Petroff-Guyou or Lambert) says format 0.6 and plays exactly as on master. Every
  change below is behind a cube branch; the Petroff-Guyou parity fixture and the existing GameTests
  are the check that nothing moved.
- A **cube world** says 0.7 and has `projection.kind` `cube`, layout version 2. Java's
  `FormatVersion` mirrors Rust's `Manifest::read`: 0.6 and 0.7 are read, the mismatch is refused both
  ways (`format_version 0.6 on a cube world, which is 0.7`), and the refusal of any other version names
  both. A cube world whose map table the jar does not ship by name and sha256 is refused before
  anything else is read, as in Rust.
- FORMAT.md's "Phase 2 moves everything to 0.7 and drops the polar layouts" waits for the deletions:
  until then 0.6 stays the polar worlds' number and every file header keeps minor 6.

## 2. Readers (G1)

- **Grids.** `GridDesc` gains the cube form `{"name", "kind": "cube", "n", "r"}`: w = n, h = 6n, no
  x wrap, origin (−s/2, −s/2), its circumference 4·n·r (the manifest's 4s), tile gcd(n, 256).
  `ZEdges` stays for the polar grids. `GridDesc.resolveFaceCell` is Rust's `resolve_face_cell` line
  for line (the corner squares take the neighbour the cell is further out of, the other index
  clamped; more than a face out clamps).
- **Samplers.** `RasterSampler` takes a block as on the polar grids and, on a cube grid, reads at the
  block centre folded onto the planet (`CubeLayout.foldPoint`); the fractional cell is
  a = (x − ox)/r − ½. Nearest, bilinear, cover, the column spline and the stack reads are the Rust
  cube branches (`cube_spot`, `get_face`, `bilinear_weights`) in the same order of operations, so the
  column parity fixture holds on a cube world too. Past the net's corners every read is NaN (cover 0).
- **The projection.** `WorldProjection.Cube` wraps `CubeLayout` and `CubeWorld`: period 4s, radius
  s·√(1.5/π), z from −3s/2 to 3s/2, x wrapped into the window [−3s/2, 5s/2) (not the polar
  [−C/2, C/2)), `canonical` the fold, `unit` and `latLon` through the map (NaN past the corners),
  `north` from the map's Jacobian turned by the fold, the index frame origin (−3s/2, −3s/2) and span
  (4s, 3s). `WorldProjection.of` builds it from the layout block.
- **Spatial indexes** (sites, bodies, structure, rivers, the far columns) keep their window-frame
  buckets as the bake writes them. Their buckets take x modulo their own period, so a query at a
  window block (x anywhere in the wrapped window, as Columns.sample and Rust's
  `ColumnWorld::sample` pass it after the plain x wrap by C) finds what Rust's evaluator finds;
  the rasters fold inside the samplers. Neither evaluator folds the block before its queries (the
  same algorithm on both sides keeps parity), so a block outside the planet is not read through
  the readers at all: its blocks are the G5 fill, a copy of the partner chunk. Neighbourhood queries that would reach across a turning seam (a body, a structure
  window or a river channel cut by the seam) are G1's second step: the query point's image under
  each nearby edge's motion is queried too, in Java and in Rust's evaluator together so parity
  holds. Stage 1 leaves them out; the cut is at most the reach of one record, along the seams only.

## 3. The fill outside the planet (G5)

Everything outside the six faces is drawn as the fold through the nearest net edge (plan section
1.4). The fill is built **chunk by chunk as an exact copy of the partner's own generation before
features**, so the seam GameTest can compare block for block and the wrap engine's band (E3) can take
it as its content.

- **Which chunks.** A chunk is *on the planet* when any column is on a face; *wholly outside* when no
  column is. Since every pivot is a chunk corner and every edge lies on a multiple of s/2, a chunk is
  wholly on one face or wholly outside, except the chunks the diagonal of a void square cuts by F0's
  corners; those are outside too (no column is on a face) but their columns have two partners.
- **Biomes (BIOMES stage).** A wholly outside chunk takes its biomes from a scratch chunk built at
  each partner chunk, quart by quart through the edge's motion (quarts map to quarts: the pivots are
  chunk corners). Deep ocean where the fold is `Ocean`.
- **Noise (NOISE stage).** Nothing: the chunk stays empty until its surface stage.
- **Surface (SURFACE stage).** For each (partner chunk, motion) group of its columns a scratch
  `ProtoChunk` at the partner chunk is generated through the generator's own code: the noise fill
  (the generator's `doFill` with a noise chunk whose Beardifier is empty), the biomes, the surface
  rules with a biome manager that answers the scratch's own quarts and the biome source elsewhere
  (vanilla's zoom seed), Deep Time's passes (sea cliffs, strata, water bodies, playa, lava flows,
  magma, geothermal, ice), then the air carvers as vanilla's `applyCarvers` runs them, with the
  carver biome of each chunk within 8 taken from the biome source (what vanilla caches per chunk),
  and `WaterBodies.wake`. Every block of the group's columns is then copied, its state turned by
  `BlockState.rotate` (k = 1 is `COUNTERCLOCKWISE_90`, k = 2 `CLOCKWISE_180`, k = 3 `CLOCKWISE_90`:
  the iso turns the partner's frame onto the outside chunk's), fluid post-processing marks copied
  through the motion, and the world-generation heightmaps primed.
- **Carvers (CARVERS stage).** Nothing for a wholly outside chunk: the copy already holds them.
- **Ocean.** A column whose fold is `Ocean` (more than s out, past the net's corners) is deep ocean:
  the world's default block up to y = sea level − 64 (SCL-211, game-choice), water to sea level,
  biome `minecraft:deep_ocean`.
- **Structures and features.** No structure starts in a wholly outside chunk and no features are
  placed there (SCL-210, game-choice). Features from planet chunks next to a seam still reach one
  chunk out, as vanilla's write radius allows. Mobs are not spawned in wholly outside chunks.
- **What the copy cannot hold.** The terrain a structure adapts round its pieces (villages,
  ancient cities: the Beardifier) is the partner's own and is not copied, since the partner's
  structure starts are not known where the copy is made. Features (trees, ores placed as features,
  springs) are not copied. Both belong to the wrap engine's band (E3), which copies the partner's
  finished chunk once it exists.
- **E3's gate restated.** "Copying a band chunk of untouched ground changes no block" holds for the
  pre-feature terrain; with features it holds only once the engine copies finished chunks. This is a
  question for the coordinator (section 7).

Petroff-Guyou keeps `BeyondPoleDensity` and its reflections: the fill branch runs only on a cube
world.

## 4. Detail noise (G3)

The noise lattice R has period 4s in x, so the slide needs nothing, and R is continuous across the
direct seams (F0 with N, F1, F3 and S; F1 with F2; F2 with F3), which are not cut in the net. Only the
12 turning edges cut it. `DetailNoise.at(x, z)` on a cube world:

- folds the block's centre first, so an outside block's noise is its partner's exactly;
- on a face F, for each turning edge e of F within 32 blocks (SCL-089), mixes in the lattice at the
  point's image under e's motion (`iso_e(p)`, which lies just outside the partner face, where the
  partner's own lattice continues):
  `N = R(p) − Σ_e a_e · (R(p) − R(iso_e(p)))`, with `a_e = ½ · (1 − u_e)`,
  `u_e = min(1, d_e / min(32, g_e))`, d_e the point's distance from e's line and g_e its distance
  along e from e's pivot when that pivot is a corner of F (the quarter turns, about F0's corners), else
  32 (the half turns).

So at a turning seam's line both sides read the same ½ + ½ mix of the two lattices, the weights
run linearly to the own lattice 32 blocks in (one linear blend across the seam, as SCL-089 on
Petroff-Guyou), and near F0's corner the mix fades out towards F0's direct seams (u = 1 at g = 0),
where R is continuous: the only point where the weights jump is the corner itself, a fixed point
of the motion where both lattices agree. Two turning edges of one face meet only at the four corners
on the border (n1, n2, s1, s2), which no player reaches; their mixes add there. Rust's
`DetailSpec::noise` gets the same branch, so the column parity fixture holds on a cube world.

## 5. Border, arrivals and spawn (W3 to W5)

- **W3.** `EdgeBorder` on a cube world stops at the layout's 12 border segments, each `inset_blocks`
  inside its edge line, with the 24-block warning band (POL-023). A position outside the planet or
  inside an inset is moved to the nearest point of the planet that is at least the inset inside every
  segment (the corner of the void squares by F0's corners is a nearest point, not two segment
  pushes). The slide is not bordered.
- **W4.** An arrival outside the planet (a portal, a teleport, a pearl) is folded onto its partner
  block, its heading turned by the motion (`CubeLayout.Iso.point`, yaw + 90° × k). The portal search
  takes the folded position; other arrivals are folded by the border tick.
- **W5.** `SpawnHandler` passes over a candidate within 64 blocks of a border segment (POL-024,
  generalised) or 256 blocks of any image of a cube corner (POL-030, game-choice; the bake drops the
  same, SCL-127).

## 6. Level types and presets (G12 in part)

Alongside the polar presets (the deletions wait): world presets `deeptime:planet_cube` (16k,
s = 3,840) and `deeptime:planet_cube_32k` (s = 7,424), projection `cube`, opt-in, not in the
`normal` tag's default slot. `WorldDataRef` takes projection `cube`, whose circumference is 4s.

## 7. Questions for the coordinator

1. E3's gate (section 3): restate it as "pre-feature terrain" for the fill, with the finished-chunk
   copy in the engine's band? Recommended: yes.
2. Features in the fill: skip them in wholly outside chunks (recommended, stage 1), or decorate an
   outside chunk as its partner through a turned `WorldGenLevel` view (later, if DH's view beyond the
   border needs trees)?
3. The level types' names: `planet_cube` and `planet_cube_32k` until the deletions, then `planet`
   and `planet_32k` (recommended), or another pair?
