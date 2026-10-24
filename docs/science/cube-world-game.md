# The game on a cube world

Status of this page: `stage 1 (written with the feature, cube plan phase 2, 2026-10-23; nothing here
is physical, every row is a game choice)`.

Ledger ids on this page: `SCL-210`, `SCL-211`, `SCL-215`, `POL-031`, with `SCL-108`, `SCL-127` and `POL-022`
to `POL-024` from earlier pages (see [claims.md](claims.md)).

## What you see in the world

A cube world (world-data format 0.7, the level types `deeptime:planet_cube` at 16k and
`deeptime:planet_cube_32k`, opt-in on the Create World screen) is the planet drawn on six square
faces of side s blocks (3,840 at 16k, 7,424 at 32k) laid out as a cross: four faces round the
equator, F3, F0, F1 and F2, wrap east to west through Toroidal World with a period of 4s, and one
face lies above F0 (N) and one below it (S). The cube is turned freely on the planet, so the
poles lie somewhere inside N and S, not at their centres, and north is not the -z direction
everywhere.

Six of the cube's twelve edges are joined in the net itself: five with the faces side by side, one
(F2 to F3) by the east-west slide. Each of the other six appears twice in the net, as two
"turning edges" (round N and S, and the tops and bottoms of F1, F2 and F3, twelve in all) where one
face's ground continues on another face turned by a quarter or half turn. Until the wrap engine joins
them, a temporary border stops a player a block inside each turning edge, with a warning on the
action bar within 24 blocks of it. Looking past the border, the land is not cut off: every chunk
off the faces holds a copy of the ground it would join, turned (its partner), so the view across
a turning edge shows the right hills and coasts, but without trees, ores placed as features or
structures. Past the corners of the net, where no face's ground reaches, lies deep ocean 64 blocks
deep.

A player who arrives far outside the planet (a teleport, an ender pearl, a portal exit more than
16 blocks out) is moved to the place the copy under them shows, with their heading turned the same
way; one who walks into the border is put back at the nearest point inside it. Spectators pass.
The world spawn keeps 64 blocks from any turning edge and 256 blocks from any corner of the cube.

## The model in plain words

There is no physical model here: this page is geometry and game rules. The pieces, in the order
the game uses them:

1. **The net and the fold** (`worlddata/CubeLayout`, from phase 0, SCL-108). Every block (x, z) of
   the level is either on a face, in the band beyond one turning edge (the band reaches a face
   width out; where two bands meet, in the square void by each corner of F0, the nearer edge
   wins), or in no band. A block in a band has a partner on the planet: the block the edge's
   motion (a quarter or half turn about a chunk corner, or the slide) takes it to.
2. **The fill off the faces** (`worldgen/CubeFill`, SCL-210). The faces are chunk aligned, so a
   chunk is wholly on a face or wholly off. A chunk on a face is generated as on any world. A chunk
   off the faces is never filled from noise: at the biome step it takes its partners' biomes (a
   motion about a chunk corner maps 4 x 4 biome cells onto biome cells), and at the surface step
   each partner chunk its columns map to is generated in a scratch chunk by the same steps the game
   runs for that chunk (biomes, noise fill, surface rules, Deep Time's surface passes, the air
   carvers, the water's wake). The scratch chunk is placed where Toroidal World generates the
   partner, so it is the same terrain. Its columns are copied block for block, each block state
   turned (stairs, logs, rails face the right way after the turn). The chunk then gets no
   structure starts, features or mobs of its own.
3. **Deep ocean past the corners** (SCL-211). Columns in no band are the level's default rock up
   to 64 blocks below sea level and water above, in the deep ocean biome.
4. **The border** (`wrap/CubeBorder`, `wrap/EdgeBorder`, POL-031 with POL-022 and POL-023). The
   playable area A(d) is the six faces shrunk by d at every turning edge. Each tick a survival,
   adventure or creative player (or the vehicle they ride) outside A(1) is moved to the nearest
   point of A(1.05); a player more than 16 blocks outside the planet is first folded to their
   partner point instead. Within 24 blocks of a turning edge (measured through every east-west
   period image) the action bar warns.
5. **Spawn** (`server/SpawnHandler`, SCL-127, POL-024). A spawn candidate must lie in A(64), 64
   blocks from every turning edge, and 256 blocks from every image of a cube corner in the net.
6. **Far terrain** (`farterrain/FarTerrain`, Distant Horizons' level of detail). An area wholly on
   one face is drawn from the world data as on any world. An area that reaches off the faces is
   drawn column by column at the planet block each column shows (the same fold), its block states
   turned and its biomes those the fill stores, so the far view past a turning edge is the fill's
   ground; columns past the corners are the same deep ocean. Distant Horizons' curvature takes the
   radius of the sphere with the six faces' area, s·sqrt(3/(2π)) (SCL-078: 2,653 blocks at 16k,
   5,130 at 32k), not the period's 4s/(2π), and Toroidal World's guard checks the period 4s.
7. **The detail noise** (`worlddata/DetailNoise`, Rust's `detail::DetailSpec::cube_noise`,
   SCL-215). The game adds a small noise term to the baked ground (the detail layer's amplitude
   times a noise of 32-block wavelength). On a cube world that noise is laid on the cube's surface,
   not on the net: a block's centre goes to its face by the fold (a block off the faces to its
   partner's point), and the noise is value noise on a lattice drawn on the cube, whose lines meet
   the cube's edges (m = max(1, round(s/λ)) cells along each edge, so 32 at s = 1,024). Two faces
   meeting at an edge share the lattice points on it, so the noise runs on through every seam and
   corner with no blend, and a block off the faces reads its partner's value exactly. The bake's
   own relief noises (the jag, the cliff bands' levels) are the same function, so the bake and the
   game agree on the ground at a seam. Petroff-Guyou and Lambert worlds keep the plain noise.
8. **The climate API** (`api/climate/DeepTimeClimate`, version 5, cube plan G9). Other mods read
   the planet's geometry from it on the server and on the client. On a cube world
   `projectionKind` is `"cube"` and `circumferenceBlocks` is the period 4s. `latitudeDeg` and
   `longitudeDeg` go through the fold and the map, so a block off the faces reads its partner's
   place. Two calls are new. `northBearingDeg` gives the direction of north in the block frame,
   in degrees clockwise from -z. It runs smoothly over a face and on into the copy past its
   edge, and goes all the way round each pole. `metresPerBlock` gives the planet's metres per
   block there. Both are NaN past the corners. The old static `latitudeDeg(z, C)` was right on
   Lambert worlds only and is gone.
9. **The F3 line** (`net/PlanetDebugLine`, cube plan G10). On a cube world the debug screen reads
   `Deep Time: 10.144° N 100.920° E · face F0 · north 7.8° · 311 m/block here · <planet> (C 4096,
   276 m/block)`: latitude and longitude, the face (`F1 (folded)` off the faces, for the face the
   block folds onto), north's bearing and the metres a block spans there, the numbers the climate
   API gives, then the nominal scale. Past the corners it says "off the planet", and a client whose
   jar lacks the server's map table says "map mismatch". The client works it out from the planet
   info alone.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-108 | bands beyond the turning edges | a face width deep, nearer edge wins, ties to the earlier edge | blocks | cube-world-plan.md section 1.4 | `game-choice` |
| SCL-210 | the fill off the faces | partners' biomes; partners' terrain copied and turned at the surface step; no starts, features or mobs | rule | docs/design/cube-game.md section 2 | `game-choice` |
| SCL-211 | deep ocean past the corners | OCEAN_DEPTH = 64 below sea level | blocks | docs/design/cube-game.md section 2 | `game-choice` |
| SCL-215 | the detail noise on the cube's surface | value noise on a surface lattice, m = max(1, round(s/λ)) cells per cube edge, fade f²(3 − 2f), octaves as the plain noise | blocks | cube-world-plan.md G3; FORMAT.md "The detail layer" | `game-choice` |
| POL-022 | the stop line's inset | 1 block inside each turning edge; put back 0.05 block inside it | blocks | FORMAT.md "The world's layout" | `game-choice` |
| POL-023 | the warning band | 24 blocks, every 40 ticks | blocks, ticks | code | `game-choice` |
| POL-031 | arrivals folded | FOLD_DEPTH = 16 outside the planet | blocks | docs/design/cube-game.md section 5 | `game-choice` |
| POL-024, SCL-127 | spawn margins | 64 from a turning edge, 256 from a corner image | blocks | cube-world-plan.md B4, W5 | `game-choice` |

The playable area for an inset d, with h = s/2 and x taken into Toroidal World's frame [-2s, 2s):

    A(d) = { |z| <= h - d }  union  { |x| <= h - d and |z| <= 3h - d }  union  { |x| <= h and |z| <= h }

The first set is the equatorial band, the second N and S shrunk at their edges, and the third F0
whole, so that F0's corners (where N's and S's side edges meet F1's and F3's top and bottom edges)
stay inside. A heading of yaw y turns with a motion of k quarter turns as the vector (-sin y, cos y)
does under the motion.

The climate API's north and scale come from the map's Jacobian J at the block: its rows are the
planet's (east, north) metres per block of +x and of +z, through the fold's motion off the faces
(`CubeWorld.jacobian`). North in the block frame is the (dx, dz) that J takes to (0, 1),
normalised. Its bearing is atan2(dx, -dz) in degrees, taken into [0, 360). The metres per block
are sqrt(|det J|), with J taken on the sphere of the six faces' area, whose radius is
m_per_block_h · s · sqrt(3/(2π)) metres (SCL-107). The conformal map
has one scale in every direction, so this is the length of a block both ways. On the equal-area
Lambert and Petroff-Guyou worlds the API gives m_per_block_h everywhere, since a block's area is
the same everywhere there.

## Earth against the mod

Nothing physical is modelled, so there is nothing to compare with Earth. The GameTests
(`CubeWorldGameTests` on the 4k test planet baked as a cube, s = 1,024) check that the fill equals
its partners block for block and biome for biome. On 2026-10-23 (the VM, `./gametest.sh
-PgametestOnly=...` with the seven tests): 29 chunks off the faces, 8,257,536 blocks against their
partners turned, 0 differ, and 256 columns past the corners deep ocean; 16 chunks, 18,432 biome quarts,
0 differ; 99 chunks at the face centres, seams and corners (53 off the faces) generate with no hole
to the floor; a point just inside each turning edge and the fold of the point just outside it are
at most 1.05 blocks apart on the sphere (R 707.6 blocks); the border puts a player in a void square
or past the corners back on the planet (4.55 and 92.7 blocks), folds one 100 blocks past N-east
onto F1 with its yaw turned 90°, and leaves both poles and the face centres alone.

Far terrain (stage 2, 2026-10-23, the 4k cube, `farTerrainMatchesTheCube` with the other 13 cube and
far terrain tests): against the chunks the game generated, 168 columns in the 16 blocks inside the
turning edges (167 at the same ground height, all within 1 block), 168 past them (165 exact, all
within 1), 40 across the slide and 132 across Toroidal World's seam (all exact), and the 4 corners of
a chunk past the net's corners (deep ocean, exact); water and biome agree in every column. At 16k
(the smoke's `--dh` checks on `earthlike_quick_16k_cube.json`, then `dh-client-check.sh`, the VM's
private Xvfb): Distant Horizons held 400 of 400 columns on each side of the slide (x = -5,760)
with ground within 2 blocks of the generated chunk in 390 and 398 of them, water or dry as
generated in all 800 and the biome in all 800; the real client then received 2,001 far sections, 378
and 228 of them in the 2,048 blocks west and east of the slide and 210 and 362 beside the seam.

The surface noise (G3, 2026-10-23, `build/cube2/g3-checks.sh` on the VM): the 14 cube and far
terrain GameTests above pass with it, and `gametest-planet.sh --cube-4k` reads the same ground
height (to 0.01 block) from the game and from the bake's Rust reader in the 10 probe columns on every
face. Its unit tests check that the noise reads one value from both sides of each of the 14 net
edges (the slide and the twelve turning edges) and steps across them no more than across a lattice
line inside a face, that an outside block reads its fold partner's value exactly, and that Rust and
Java give the same pinned values. The seam steps it gives the bake are on the bake page (B1).

## Simplifications and limits

- The copy off the faces has no structures and no features: trees, ore veins placed as features,
  springs and villages stop at the turning edges in the view across them. The wrap engine's band
  (cube plan E3) shows the partner's real chunks instead and supersedes the fill there.
- Terrain that a structure adapts round its pieces (the beardifier) is not copied: the scratch
  chunk does not know the partner's structure starts.
- Lava flows (G16) do not yet cross turning seams.
- Far terrain past a turning edge is the fill's ground, so it has no trees or structures either;
  the area is drawn column by column there (its cost there was not measured apart).
- The surface noise's lattice is square on each face in net blocks, so where the map's local
  scale σ varies across a face the noise's texture is stretched with it on the planet, as the
  plain noise's is on the other projections; in blocks it is the same everywhere.
- Sable ships are not stopped by the cube border yet.

## Game choices

All rows above are game choices made by the cube lane's agent on 2026-10-23 under Ben's rule of
2026-10-16 (undoable choices are the coordinator's call), within Ben's decisions of 2026-10-22 (a
cube world, the conformal cube turned freely, format 0.7, the wrap engine in parallel).

## Open questions

- Whether the fill should place features (trees) from the partner: the wrap engine's band makes
  it moot where the engine runs.
- Whether the two level types replace `deeptime:planet` and `deeptime:planet_32k` (cube plan
  section 7.4, the coordinator's call after phase 2 is gated).
